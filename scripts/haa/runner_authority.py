"""W3 candidate trusted-runner seam. No CLI, environment or production activation.

This is a cooperative trusted-controller library, not an OS sandbox.
Call only in a controller isolated from agent code. Existing runner CLI
remains unchanged until deployment can prevent direct bypass.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from threading import RLock
from typing import Any, Callable, Protocol

from scripts.haa.authority_lease import LeaseStore
from scripts.haa.lease_guard import evaluate_lease_operation
from scripts.haa.live_verify import TrustedTask, checked_bound_card, _git
from scripts.haa.contracts import build_task_scope, digest_json


class RunnerAuthorityError(RuntimeError):
    pass


class IsolationBoundary(Protocol):
    """Trusted deployment must implement this using an actual host boundary.

    A positive Python return value is not proof of isolation. No production
    implementation is shipped here; never inject an agent-provided object.
    """
    def run(self, argv: tuple[str, ...], *, cwd: Path) -> Any: ...


@dataclass(frozen=True)
class RunnerScope:
    task: TrustedTask
    lease_id: str | None = None


class RunnerAuthority:
    """Serialize cooperative operations; recheck the pinned card every time.

    Test/read/patch/commit effects are trusted controller functions, never
    callbacks supplied by agent/model output. Test subprocesses must pass
    through the separately verified IsolationBoundary.
    """
    def __init__(self, *, scope: RunnerScope, leases: LeaseStore,
                 existing_boundary: Any, isolated_commands: IsolationBoundary | None = None):
        self.scope = scope
        self.leases = leases
        self.existing = existing_boundary
        self.isolated_commands = isolated_commands
        self._lock = RLock()
        self._closed = False
        self._started = False
        self._root = scope.task.repo_root.resolve()
        self._initial_head = None
        self._branch = None

    def _card(self):
        if self._closed:
            raise RunnerAuthorityError("SESSION_CLOSED")
        task = self.scope.task
        card = checked_bound_card(task, self._root, required=True)
        if Path(_git(self._root, "rev-parse", "--show-toplevel")).resolve() != self._root:
            raise RunnerAuthorityError("WORKTREE_MISMATCH")
        origin = _git(self._root, "remote", "get-url", "origin")
        if origin not in {
            f"https://github.com/{task.repository}", f"https://github.com/{task.repository}.git",
            f"git@github.com:{task.repository}.git", f"ssh://git@github.com/{task.repository}.git",
        }:
            raise RunnerAuthorityError("ORIGIN_MISMATCH")
        if not card.plan:
            raise RunnerAuthorityError("PLAN_REQUIRED")
        return card

    def _check(self, operation: str, paths: tuple[str, ...] = ()):
        card = self._card()
        task = self.scope.task
        if operation not in {"edit", "test", "review", "commit"}:
            raise RunnerAuthorityError("PROTECTED_OR_UNKNOWN_OPERATION")
        if operation not in task.capabilities:
            raise RunnerAuthorityError("CAPABILITY_NOT_APPROVED")
        if card.rri > 25:
            if not self.scope.lease_id:
                raise RunnerAuthorityError("WAITING_FOR_HAA")
            lease = self.leases.load(self.scope.lease_id)
            if not lease.grant_digest:
                raise RunnerAuthorityError("VERIFIED_GRANT_EVIDENCE_REQUIRED")
            decision = evaluate_lease_operation(
                store=self.leases, lease_id=self.scope.lease_id,
                trusted_task=task, operation=operation, touched_paths=paths,
            )
            if decision.outcome != "ALLOW":
                raise RunnerAuthorityError(decision.reason)
        else:
            if self.scope.lease_id:
                raise RunnerAuthorityError("LOW_BAND_LEASE_MISMATCH")
            if (_git(self._root, "symbolic-ref", "--quiet", "--short", "HEAD") != self._branch
                    or _git(self._root, "rev-parse", "--verify", "HEAD") != self._initial_head):
                raise RunnerAuthorityError("LOW_BAND_GIT_STATE_CHANGED")
            # Existing HITL and scope checks still apply. Low-band does not
            # grant new commit/privileged authority.
            if operation == "commit":
                raise RunnerAuthorityError("COMMIT_REQUIRES_SEPARATE_CONTROLLER")
        return card

    def preflight(self):
        """Call before model invocation; no side effect or approval acquisition."""
        with self._lock:
            if self._started:
                raise RunnerAuthorityError("SESSION_ALREADY_STARTED")
            card = self._card()
            self._branch = _git(self._root, "symbolic-ref", "--quiet", "--short", "HEAD")
            self._initial_head = _git(self._root, "rev-parse", "--verify", "HEAD")
            if not self.scope.task.capabilities:
                raise RunnerAuthorityError("NO_CAPABILITIES")
            # Verify the first intended capability; edit/commit guards
            # require an actual path, so validate read-only start via a
            # permitted operation plus exact card scope when applicable.
            if card.rri > 25:
                if not self.scope.lease_id:
                    raise RunnerAuthorityError("WAITING_FOR_HAA")
                lease = self.leases.load(self.scope.lease_id)
                if not lease.grant_digest:
                    raise RunnerAuthorityError("VERIFIED_GRANT_EVIDENCE_REQUIRED")
                if (lease.session_id != self.scope.task.session_id
                        or lease.rri != card.rri or lease.task_id != card.task_id):
                    raise RunnerAuthorityError("LEASE_BINDING_MISMATCH")
                operation = next((v for v in ("test", "review") if v in self.scope.task.capabilities), None)
                if operation is None:
                    raise RunnerAuthorityError("PREFLIGHT_CAPABILITY_REQUIRED")
                self._check(operation)
            else:
                self._check(next((v for v in ("test", "review", "edit") if v in self.scope.task.capabilities), "unknown"))
            self._started = True

    def _require_started(self):
        if not self._started or self._closed:
            raise RunnerAuthorityError("SESSION_NOT_ACTIVE")

    def edit(self, path: str, effect: Callable[[], Any]):
        """Trusted file tool supplies the effect, not the model."""
        with self._lock:
            self._require_started()
            self.existing.check_write(path)
            self._check("edit", (path,))
            return effect()

    def review(self, effect: Callable[[], Any]):
        with self._lock:
            self._require_started()
            self._check("review")
            return effect()

    def test(self, argv: list[str]):
        with self._lock:
            self._require_started()
            self.existing.check_command(argv)
            card = self._check("test")
            if tuple(argv) not in {tuple(command.argv) for command in card.verification_commands}:
                raise RunnerAuthorityError("COMMAND_NOT_PINNED")
            if self.isolated_commands is None:
                raise RunnerAuthorityError("COMMAND_ISOLATION_UNAVAILABLE")
            return self.isolated_commands.run(tuple(argv), cwd=self._root)

    def close(self):
        """Revocation on completion, cancellation or exception; no new request."""
        with self._lock:
            self._closed = True
            if self.scope.lease_id is not None:
                self.leases.revoke(self.scope.lease_id)

    def __enter__(self):
        try:
            self.preflight()
            return self
        except BaseException:
            self.close()
            raise

    def __exit__(self, *_exc):
        self.close()
        return False
