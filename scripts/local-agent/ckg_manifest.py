from __future__ import annotations

import hashlib
import json
import os
import subprocess
from dataclasses import dataclass, field


@dataclass(frozen=True)
class WorktreeIdentity:
    base_revision: str | None
    dirty: bool
    state_hash: str

    def as_dict(self):
        return {
            "base_revision": self.base_revision,
            "dirty": self.dirty,
            "state_hash": self.state_hash,
        }


def _git(worktree_dir, *args):
    completed = subprocess.run(
        ["git", "-C", worktree_dir, *args],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=10,
        check=False,
    )
    if completed.returncode != 0:
        return None
    return completed.stdout


def derive_worktree_identity(worktree_dir):
    revision = _git(worktree_dir, "rev-parse", "HEAD")
    base_revision = revision.strip() if revision else None
    status = _git(worktree_dir, "status", "--porcelain=v1", "--untracked-files=all")
    diff = _git(worktree_dir, "diff", "--binary", "HEAD")
    staged = _git(worktree_dir, "diff", "--binary", "--cached", "HEAD")
    payload = "\n".join(
        [
            base_revision or "",
            status or "",
            diff or "",
            staged or "",
        ]
    ).encode("utf-8", errors="replace")
    state_hash = hashlib.sha256(payload).hexdigest()
    return WorktreeIdentity(
        base_revision=base_revision,
        dirty=bool(status and status.strip()),
        state_hash=state_hash,
    )


def source_sha256(content):
    return hashlib.sha256(content.encode("utf-8")).hexdigest()


@dataclass
class CKGContextManifest:
    work_item_id: str
    capsule_hash: str | None
    source_state: WorktreeIdentity
    backend: str
    graph_revision: str | None = None
    coverage: str = "unknown"
    anchors: list[dict] = field(default_factory=list)
    scope_gaps: list[dict] = field(default_factory=list)
    selection: list[dict] = field(default_factory=list)
    budget: dict = field(default_factory=dict)
    notes: list[str] = field(default_factory=list)

    def as_dict(self):
        return {
            "schema": "ckg-context-manifest-v1",
            "task": {
                "work_item_id": self.work_item_id,
                "capsule_hash": self.capsule_hash,
            },
            "source_state": self.source_state.as_dict(),
            "graph": {
                "backend": self.backend,
                "graph_revision": self.graph_revision,
                "coverage": self.coverage,
            },
            "anchors": list(self.anchors),
            "scope_gaps": list(self.scope_gaps),
            "selection": list(self.selection),
            "budget": dict(self.budget),
            "notes": list(self.notes),
        }

    def write(self, path):
        os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
        temp = f"{path}.tmp"
        with open(temp, "w", encoding="utf-8") as handle:
            json.dump(self.as_dict(), handle, ensure_ascii=False, indent=2, sort_keys=True)
            handle.write("\n")
        os.replace(temp, path)
