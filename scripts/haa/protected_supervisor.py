"""Owner-channel composition of protected task and plan lifecycle.

No worker-callable approval, arbitrary privileged launch or CLI activation.
The deployment starts the worker under its own identity, using a fixed
worker command and scratch. This supervisor accepts only a live session,
a kernel-authenticated proposal, and a real trusted verifier.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from threading import RLock

from scripts.haa.protected_session import ProtectedSession
from scripts.haa.protected_plan_entry import PlanCloseController
from scripts.haa.verification_backend import ExactTreeVerifier


class SupervisorError(RuntimeError):
    pass


@dataclass(frozen=True)
class WorkerInvocation:
    argv: tuple[str,...]
    cwd: Path


def worker_invocation(*, layout, python: Path, socket_path: Path,
                      card: Path, output: Path, path: str):
    """Generate fixed worker argv; no credentials, approval or shell args."""
    scratch=layout.scratch
    for value in (card,output):
        if (not value.is_absolute() or value!=value.resolve()
                or not value.is_relative_to(scratch)):
            raise SupervisorError("WORKER_PATH_OUTSIDE_SCRATCH")
    if (not python.is_absolute() or python!=python.resolve()
            or not socket_path.is_absolute() or not path
            or any(p in {"",".","..",".git",".agent"} for p in path.split("/"))
            or "\\" in path):
        raise SupervisorError("INVALID_WORKER_INVOCATION")
    # -I ignores ambient PYTHONPATH; bootstrap only the protected code root.
    bootstrap=("import sys; sys.path.insert(0,sys.argv.pop(1)); "
               "from scripts.haa.worker_runner import main; raise SystemExit(main())")
    return WorkerInvocation((str(python),"-I","-c",bootstrap,str(layout.install),
        "--socket",str(socket_path),"--scratch",str(scratch),"--card",str(card),
        "--out",str(output),"--path",path),scratch)


class ProtectedSupervisor:
    """Single supervisor owns task and closure serialization.

    Call in the executor UID only. Owner control is separate from the
    worker socket. Deployment must exclude all competing target writers.
    """
    def __init__(self, *, layout, approved, plan, verifier: ExactTreeVerifier,
                 plan_validation):
        if not isinstance(verifier,ExactTreeVerifier) or verifier.repo!=layout.target:
            raise SupervisorError("EXACT_TREE_VERIFIER_REQUIRED")
        self.layout=layout
        self.plan=plan
        self.verifier=verifier
        self.lock=RLock()
        self.active=None
        self.failed=False
        self.closure=PlanCloseController(layout=layout,approved=approved,journal=plan,
            supervisor_lock=self.lock,assert_no_active_tasks=self.assert_no_active_tasks,
            verify_plan=plan_validation)

    def assert_no_active_tasks(self):
        if self.active is not None or self.failed:
            raise SupervisorError("SUPERVISOR_NOT_QUIESCENT")

    def begin(self, session: ProtectedSession):
        with self.lock:
            self.assert_no_active_tasks()
            if not isinstance(session,ProtectedSession) or session.layout!=self.layout:
                raise SupervisorError("SESSION_LAYOUT_MISMATCH")
            if self.plan.read_snapshot().workflow_state!="ACTIVE":
                raise SupervisorError("PLAN_NOT_ACTIVE")
            self.active=session
            try:
                return session.request()
            except BaseException:
                self.abort()
                raise

    def poll(self):
        with self.lock:
            if self.active is None or self.failed:
                raise SupervisorError("NO_ACTIVE_SESSION")
            return self.active.poll()

    def execute(self,listener,*,duration_seconds=60):
        """Requires already approved session; worker runs separately.

        A successful scratch process is not proof. Only a broker-accepted
        edit and verified protected commit can complete the task.
        """
        with self.lock:
            if self.active is None or self.failed:
                raise SupervisorError("NO_ACTIVE_SESSION")
            session=self.active
            try:
                session.activate()
                session.serve_worker(listener,duration_seconds=duration_seconds)
                edit=session._accepted_edit
                if edit is None:
                    raise SupervisorError("NO_ACCEPTED_EDIT")
                result=session.complete_task(plan=self.plan,verifier=self.verifier,
                                             paths=(edit[0],))
                self.active=None
                return result
            except BaseException:
                self.abort()
                raise
            finally:
                listener.close()

    def abort(self):
        with self.lock:
            self.failed=True
            if self.active is not None:
                session=self.active
                self.active=None
                session.close()
