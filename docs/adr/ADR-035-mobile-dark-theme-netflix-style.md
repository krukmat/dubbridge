---
type: ADR
title: "ADR-035: Mobile dark-theme visual identity — Netflix-style dark canvas"
status: Accepted
---

# ADR-035: Mobile dark-theme visual identity — Netflix-style dark canvas

- **Status:** Accepted
- **Date:** 2026-06-29
- **Deciders:** DubBridge platform team
- **Closes:** S-220 design-direction decision
- **Amends:** ADR-029 (visual identity section only — product-surface decision unchanged)

## Context

S-215 delivered a commercial palette recalibration that moved the app toward a more
confident SaaS register (neutral canvas, saturated teal). After reviewing the
refreshed screenshots, the team finds the ink-on-light theme still reads as an
operational utility tool rather than a content-confident product surface.

The reference direction requested is a **Netflix-style dark canvas**: deep neutral
background, high-contrast text, a bold single accent (red or red-adjacent), and
content thumbnails/posters as the dominant visual element on each screen. This is
a common pattern in professional media tooling (Frame.io, Blackbird, Avid MediaCentral)
and is appropriate for a product whose primary objects are video assets.

This direction requires a deliberate decision because it materially changes:

1. The token system in `mobile/src/theme/tokens.ts` (canvas, ink scale, primary accent)
2. The component library (backgrounds, borders, badge tones — all currently light-mode)
3. DESIGN.md (the agent-readable color block governs future work)
4. Contrast certification (WCAG AA must be re-verified on every interactive pair)
5. ADR-029's implicit light-mode assumption in the "restrained ink + teal" visual language

## Decision

### D1 — Adopt a dark canvas as the default mobile theme

Replace the current light canvas (`#F7F8FA`) with a deep neutral dark canvas
(`#141414`) as the default app background. This is a product-level decision, not
a system dark-mode follow: the app ships dark by default regardless of OS setting.

### D2 — Replace teal with a bold red accent

Replace `primary` (`#097F67`) with a confident red accent (`#E50914`, Netflix
signature, or a DubBridge-calibrated derivative). The accent must clear WCAG AA
(4.5:1) against the dark canvas for interactive text labels, and 3:1 for large
UI elements (buttons, badges).

### D3 — Invert the ink scale

The current ink scale (ink900 darkest → ink300 lightest, designed for dark-on-light)
is inverted: the lightest values become primary reading text on the dark canvas.
New scale anchors:

| Token | Current (light) | New (dark) |
|---|---|---|
| `ink900` | `#0F1B22` | `#F5F5F5` (primary reading text) |
| `ink700` | `#243640` | `#EBEBEB` |
| `ink500` | `#4A5A63` | `#B3B3B3` |
| `ink400` | `#647079` | `#8C8C8C` |
| `ink300` | `#8A949B` | `#666666` |

Exact values are subject to contrast verification during S-220 implementation.

### D4 — Dark surface scale

| Token | Current | New |
|---|---|---|
| `canvas` | `#F7F8FA` | `#141414` |
| `raised` | `#FFFFFF` | `#1F1F1F` |
| `sunken` | `#EEF0F4` | `#0A0A0A` |
| `border` | `#E1E5EC` | `#2A2A2A` |
| `borderStrong` | `#C2CDC8` | `#3D3D3D` |

### D5 — Semantic colors are recalibrated, not removed

`success`, `warning`, `danger`, `info` and their subtle/strong variants remain.
Their values shift to dark-on-dark appropriate tones. The governance and compliance
UI must retain clear semantic color signals regardless of theme.

### D6 — WCAG AA is a hard gate

No token value ships without a verified contrast ratio. The S-220 implementation
must include a test file (`theme.tokens.test.js`) that asserts every interactive
pair (foreground on background) clears WCAG AA. This extends the T8 contract
established in S-215.

### D7 — DESIGN.md is the sync boundary

All shipped token values must be mirrored in the DESIGN.md color block before
S-220 closes. Agents reading DESIGN.md must see the current dark-theme values.

### D8 — Component library is light-mode-unaware after S-220

After S-220 lands, no component in `mobile/src/components/` may reference a
light-canvas assumption (white backgrounds, dark borders as primary contrast
signals). The theme token system is the only source of surface color.

## Consequences

### Accepted regressions

- All current screenshot baselines become stale on S-220 merge and must be
  refreshed in the same slice.
- The Gemma Reviewer will surface contrast findings during S-220 review; the
  implementation agent must adjudicate each one against WCAG AA before closing.
- The "ink + teal" visual identity described in ADR-029 is superseded for the
  mobile surface. ADR-029's product-surface decision (mobile as sole authenticated
  UI) is unchanged.

### Non-regressions

- Governance, compliance, and audit surfaces remain fully operational.
- No navigation structure, route, testID, or API contract changes.
- ADR-031 (JWT transport), ADR-030 (publication gate), ADR-008 (rights fail-closed)
  are unaffected.

### Open follow-ups recorded

| ID | Need |
|---|---|
| X-S-220-1 | OS-level dark/light mode toggle (ship dark by default; honor system preference later) |
| X-S-220-2 | Poster/thumbnail contrast on dark canvas (media cards need a minimum background scrim when poster images are light) |
| X-S-220-3 | Semantic badge legibility audit — `successSubtle`/`warningSubtle`/`dangerSubtle` are currently tinted-light; new dark equivalents need design sign-off |

## Related

- `ADR-029-mobile-as-sole-authenticated-product-surface.md`
- `DESIGN.md`
- `mobile/src/theme/tokens.ts`
- `docs/plan/s-220-mobile-dark-theme.md`
- `docs/plan/s-215-mobile-streaming-organization-pass.md`
