from __future__ import annotations

import json
import gemma_local
from ckg_adapter import CKGAdapterError, CodebaseMemoryCLIAdapter
from ckg_manifest import CKGContextManifest, derive_worktree_identity, source_sha256


def _render_entries(entries):
    blocks = []
    for entry in entries:
        path = json.dumps(entry["path"], ensure_ascii=False)
        if entry.get("missing"):
            blocks.append(f"--- {path} (missing; creation allowed) ---")
        else:
            blocks.append(
                f"--- BEGIN {path} ---\n{entry['content']}\n--- END {path} ---"
            )
    return "\n\n".join(blocks) if blocks else "(none)"


class ContextProvider:
    def render_initial(self):
        raise NotImplementedError

    def render_refresh(self, reason):
        raise NotImplementedError

    def manifest(self):
        return None


class LegacyContextProvider(ContextProvider):
    def __init__(self, card, file_tools):
        self.card = card
        self.file_tools = file_tools

    def _render(self):
        return _render_entries(self.file_tools.preload_context(self.card.allowed_paths))

    def render_initial(self):
        return self._render()

    def render_refresh(self, reason):
        return self._render()


class CKGContextProvider(ContextProvider):
    def __init__(
        self,
        *,
        card,
        worktree_dir,
        boundary,
        file_tools,
        retrieval_budget_tokens,
        budget_details,
        adapter=None,
        boundary_error=Exception,
        manifest_path=None,
    ):
        self.card = card
        self.worktree_dir = worktree_dir
        self.boundary = boundary
        self.file_tools = file_tools
        self.retrieval_budget_tokens = max(0, int(retrieval_budget_tokens))
        self.budget_details = budget_details
        self.adapter = adapter or CodebaseMemoryCLIAdapter()
        self.boundary_error = boundary_error
        self.manifest_path = manifest_path
        self._manifest = None
        self._last_render = None

    def manifest(self):
        return self._manifest.as_dict() if self._manifest else None

    def _current_source(self, candidate):
        content = self.file_tools.read_checked(candidate.path)
        if candidate.start_line and candidate.end_line and candidate.end_line >= candidate.start_line:
            lines = content.splitlines(keepends=True)
            start = max(0, candidate.start_line - 1)
            end = min(len(lines), candidate.end_line)
            if start < end:
                return "".join(lines[start:end]), "worktree"
        return content, "worktree"

    def _authorize(self, candidates):
        selected = []
        gaps = []
        for candidate in candidates:
            try:
                self.boundary.check_path(candidate.path)
            except self.boundary_error as exc:
                gaps.append(
                    {
                        "path": candidate.path,
                        "symbol": candidate.symbol,
                        "reason": "outside_allowed_paths",
                        "detail": str(exc),
                    }
                )
                continue
            selected.append(candidate)
        return selected, gaps

    def _resolve(self):
        identity = derive_worktree_identity(self.worktree_dir)
        discovery = self.adapter.discover(self.card.spec, self.worktree_dir)
        authorized, gaps = self._authorize(discovery["candidates"])
        coverage = self.adapter.coverage(
            discovery["project"], sorted({candidate.path for candidate in authorized})
        )
        if coverage["status"] != "verified":
            manifest = CKGContextManifest(
                work_item_id=self.card.task_id,
                capsule_hash=getattr(self.card, "capsule_hash", None),
                source_state=identity,
                backend=self.adapter.backend_name,
                graph_revision=identity.base_revision,
                coverage=coverage["status"],
                anchors=[
                    {"path": path, "reason": "explicit_task_anchor"}
                    for path in discovery["anchors"].get("paths", [])
                ]
                + [
                    {"symbol": symbol, "reason": "explicit_task_anchor"}
                    for symbol in discovery["anchors"].get("symbols", [])
                ],
                scope_gaps=gaps,
                budget=dict(self.budget_details),
                notes=["graph coverage incomplete; legacy source fallback required"],
            )
            self._manifest = manifest
            if self.manifest_path:
                manifest.write(self.manifest_path)
            raise CKGAdapterError(
                f"CKG coverage is {coverage['status']}; use source fallback"
            )
        entries = []
        manifest_selection = []
        used_tokens = 0
        for candidate in authorized:
            try:
                content, context_source = self._current_source(candidate)
            except (OSError, ValueError, RuntimeError):
                continue
            tokens = gemma_local.estimate_text_tokens(content)
            if used_tokens + tokens > self.retrieval_budget_tokens:
                continue
            used_tokens += tokens
            entries.append({"path": candidate.path, "missing": False, "content": content})
            manifest_selection.append(
                {
                    "path": candidate.path,
                    "symbol": candidate.symbol,
                    "mode": "region" if candidate.start_line and candidate.end_line else "body_or_file",
                    "reason": candidate.reason,
                    "priority": candidate.priority,
                    "context_source": context_source,
                    "source_sha256": source_sha256(content),
                    "estimated_tokens": tokens,
                }
            )
        if not entries:
            raise CKGAdapterError("CKG produced no authorized context within retrieval budget")
        manifest = CKGContextManifest(
            work_item_id=self.card.task_id,
            capsule_hash=getattr(self.card, "capsule_hash", None),
            source_state=identity,
            backend=self.adapter.backend_name,
            graph_revision=identity.base_revision,
            coverage=coverage["status"],
            anchors=[
                {"path": path, "reason": "explicit_task_anchor"}
                for path in discovery["anchors"].get("paths", [])
            ]
            + [
                {"symbol": symbol, "reason": "explicit_task_anchor"}
                for symbol in discovery["anchors"].get("symbols", [])
            ],
            scope_gaps=gaps,
            selection=manifest_selection,
            budget={**self.budget_details, "selected_context_tokens": used_tokens},
        )
        self._manifest = manifest
        if self.manifest_path:
            manifest.write(self.manifest_path)
        return _render_entries(entries)

    def render_initial(self):
        self._last_render = self._resolve()
        return self._last_render

    def render_refresh(self, reason):
        self._last_render = self._resolve()
        if self._manifest:
            self._manifest.notes.append(f"refresh_reason={reason}")
            if self.manifest_path:
                self._manifest.write(self.manifest_path)
        return self._last_render


class FallbackContextProvider(ContextProvider):
    def __init__(self, primary, fallback):
        self.primary = primary
        self.fallback = fallback
        self.last_fallback_reason = None
        self._active = primary

    def _call(self, method, *args):
        try:
            value = getattr(self.primary, method)(*args)
            self._active = self.primary
            return value
        except (CKGAdapterError, OSError, ValueError, RuntimeError) as exc:
            self.last_fallback_reason = str(exc)
            self._active = self.fallback
            return getattr(self.fallback, method)(*args)

    def render_initial(self):
        return self._call("render_initial")

    def render_refresh(self, reason):
        return self._call("render_refresh", reason)

    def manifest(self):
        manifest = self.primary.manifest()
        if manifest is not None and self.last_fallback_reason:
            manifest = dict(manifest)
            notes = list(manifest.get("notes", []))
            notes.append(f"legacy_fallback={self.last_fallback_reason}")
            manifest["notes"] = notes
        return manifest
