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

    def render_current(self, reason=None):
        raise NotImplementedError

    def render_refresh(self, reason, hints=None):
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

    def render_current(self, reason=None):
        return self._render()

    def render_refresh(self, reason, hints=None):
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
        self._graph_state_hash = None
        self._selected_candidates = []

    def manifest(self):
        return self._manifest.as_dict() if self._manifest else None

    def _current_source(self, candidate):
        content = self.file_tools.read_checked(candidate.path)
        if (
            candidate.start_line
            and candidate.end_line
            and candidate.end_line >= candidate.start_line
        ):
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

    @staticmethod
    def _anchors(discovery):
        return [
            {"path": path, "reason": "explicit_task_anchor"}
            for path in discovery["anchors"].get("paths", [])
        ] + [
            {"symbol": symbol, "reason": "explicit_task_anchor"}
            for symbol in discovery["anchors"].get("symbols", [])
        ]

    def _retrieval_text(self, repair_hints=None):
        acceptance = json.dumps(
            self.card.acceptance_tests, ensure_ascii=False, separators=(",", ":")
        )
        parts = [f"{self.card.spec}\n\nAcceptance criteria/tests:\n{acceptance}"]
        if repair_hints:
            edited_paths = repair_hints.get("edited_paths") or []
            if edited_paths:
                parts.append(
                    "Repair edited paths:\n"
                    + "\n".join(str(path) for path in edited_paths[:20])
                )
            diagnostic = repair_hints.get("diagnostic_summary") or ""
            if diagnostic:
                # Diagnostics are already deterministically compacted by the
                # session loop. Bound once more before sending them to CBM so
                # a custom caller cannot turn repair retrieval into a log dump.
                parts.append("Repair diagnostics:\n" + str(diagnostic)[:6000])
        return "\n\n".join(parts)

    def _render_candidates(self, candidates):
        entries = []
        used_candidates = []
        used_tokens = 0
        for candidate in candidates:
            try:
                content, _context_source = self._current_source(candidate)
            except (OSError, ValueError, RuntimeError):
                continue
            tokens = gemma_local.estimate_text_tokens(content)
            if used_tokens + tokens > self.retrieval_budget_tokens:
                continue
            used_tokens += tokens
            entries.append(
                {"path": candidate.path, "missing": False, "content": content}
            )
            used_candidates.append(candidate)
        if not entries:
            raise CKGAdapterError(
                "CKG produced no authorized context within retrieval budget"
            )
        return _render_entries(entries), used_candidates, used_tokens

    def _resolve(self, *, force_refresh=False, repair_hints=None):
        identity = derive_worktree_identity(self.worktree_dir)
        discovery = self.adapter.discover(
            self._retrieval_text(repair_hints),
            self.worktree_dir,
            force_refresh=force_refresh,
        )
        # discover() indexes the exact task worktree. After a repair mutation,
        # force_refresh=True performs an incremental re-index before traversal.
        if self._graph_state_hash is None or force_refresh:
            self._graph_state_hash = identity.state_hash
        authorized, gaps = self._authorize(discovery["candidates"])
        coverage = self.adapter.coverage(
            discovery["project"], sorted({candidate.path for candidate in authorized})
        )
        notes = []
        if identity.state_hash != self._graph_state_hash:
            notes.append(
                "worktree changed after graph snapshot; current worktree source remains authoritative"
            )
        if coverage["status"] != "verified":
            manifest = CKGContextManifest(
                work_item_id=self.card.task_id,
                capsule_hash=getattr(self.card, "capsule_hash", None),
                source_state=identity,
                backend=self.adapter.backend_name,
                graph_revision=self._graph_state_hash,
                coverage=coverage["status"],
                anchors=self._anchors(discovery),
                scope_gaps=gaps,
                budget=dict(self.budget_details),
                notes=notes
                + ["graph coverage incomplete; legacy source fallback required"],
            )
            self._manifest = manifest
            if self.manifest_path:
                manifest.write(self.manifest_path)
            raise CKGAdapterError(
                f"CKG coverage is {coverage['status']}; use source fallback"
            )

        rendered, used_candidates, used_tokens = self._render_candidates(authorized)
        manifest_selection = []
        for candidate in used_candidates:
            content, context_source = self._current_source(candidate)
            manifest_selection.append(
                {
                    "path": candidate.path,
                    "symbol": candidate.symbol,
                    "mode": (
                        "region"
                        if candidate.start_line and candidate.end_line
                        else "body_or_file"
                    ),
                    "reason": candidate.reason,
                    "priority": candidate.priority,
                    "context_source": context_source,
                    "source_sha256": source_sha256(content),
                    "estimated_tokens": gemma_local.estimate_text_tokens(content),
                }
            )

        self._selected_candidates = used_candidates
        manifest = CKGContextManifest(
            work_item_id=self.card.task_id,
            capsule_hash=getattr(self.card, "capsule_hash", None),
            source_state=identity,
            backend=self.adapter.backend_name,
            graph_revision=self._graph_state_hash,
            coverage=coverage["status"],
            anchors=self._anchors(discovery),
            scope_gaps=gaps,
            selection=manifest_selection,
            budget={
                **self.budget_details,
                "selected_context_tokens": used_tokens,
            },
            notes=notes,
        )
        self._manifest = manifest
        if self.manifest_path:
            manifest.write(self.manifest_path)
        return rendered

    def render_initial(self):
        self._last_render = self._resolve()
        return self._last_render

    def render_current(self, reason=None):
        """Re-read the current selected source without re-querying/re-indexing CKG."""
        if not self._selected_candidates:
            if self._last_render is not None:
                return self._last_render
            return self.render_initial()
        rendered, _used_candidates, _used_tokens = self._render_candidates(
            self._selected_candidates
        )
        self._last_render = rendered
        return rendered

    def render_refresh(self, reason, hints=None):
        self._last_render = self._resolve(
            force_refresh=True,
            repair_hints=hints or {},
        )
        if self._manifest:
            self._manifest.notes.append(f"refresh_reason={reason}")
            edited_paths = (hints or {}).get("edited_paths") or []
            if edited_paths:
                self._manifest.notes.append(
                    "repair_edited_paths=" + ",".join(str(path) for path in edited_paths[:20])
                )
            if self.manifest_path:
                self._manifest.write(self.manifest_path)
        return self._last_render


class FallbackContextProvider(ContextProvider):
    def __init__(self, primary, fallback):
        self.primary = primary
        self.fallback = fallback
        self.last_fallback_reason = None
        self._active = primary

    def _call(self, method, *args, **kwargs):
        try:
            value = getattr(self.primary, method)(*args, **kwargs)
            self._active = self.primary
            return value
        except (CKGAdapterError, OSError, ValueError, RuntimeError) as exc:
            self.last_fallback_reason = str(exc)
            self._active = self.fallback
            return getattr(self.fallback, method)(*args, **kwargs)

    def render_initial(self):
        return self._call("render_initial")

    def render_current(self, reason=None):
        # Preserve whichever provider is active. In particular, a legacy
        # fallback should not retry CBM on every edit after the initial failure.
        try:
            return self._active.render_current(reason)
        except (CKGAdapterError, OSError, ValueError, RuntimeError) as exc:
            self.last_fallback_reason = str(exc)
            self._active = self.fallback
            return self.fallback.render_current(reason)

    def render_refresh(self, reason, hints=None):
        # A repair is the deliberate retry/re-index point, so prefer primary
        # again and retain the existing fail-safe fallback behavior.
        return self._call("render_refresh", reason, hints=hints)

    def manifest(self):
        manifest = self.primary.manifest()
        if manifest is not None and self.last_fallback_reason:
            manifest = dict(manifest)
            notes = list(manifest.get("notes", []))
            notes.append(f"legacy_fallback={self.last_fallback_reason}")
            manifest["notes"] = notes
        return manifest
