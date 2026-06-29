---
version: alpha
name: DubBridge Mobile
description: Agent-readable design intent for the first-party DubBridge mobile app. Mirrors the shipped React Native token system; does not replace runtime styling in mobile/src/theme/tokens.ts.
colors:
  ink900: "#F5F5F5"
  ink700: "#E0E0E0"
  ink500: "#A8A8A8"
  ink400: "#737373"
  ink300: "#4D4D4D"
  canvas: "#141414"
  raised: "#1F1F1F"
  sunken: "#0A0A0A"
  border: "#2A2A2A"
  borderStrong: "#3D3D3D"
  primary: "#E50914"
  primaryPressed: "#FF3333"
  primarySubtle: "#2A0608"
  onPrimary: "#FFFFFF"
  success: "#2DC76D"
  successSubtle: "#0D2E1A"
  successStrong: "#1FA855"
  warning: "#F5A623"
  warningSubtle: "#2E1F04"
  warningStrong: "#D4891A"
  danger: "#E50914"
  dangerSubtle: "#2A0608"
  dangerPressed: "#B8000B"
  info: "#3B9EDB"
  infoSubtle: "#071622"
  infoStrong: "#4BAEE5"
typography:
  display:
    fontFamily: System
    fontSize: 32px
    fontWeight: 700
    lineHeight: 1.1875
  title:
    fontFamily: System
    fontSize: 24px
    fontWeight: 700
    lineHeight: 1.25
  heading:
    fontFamily: System
    fontSize: 19px
    fontWeight: 700
    lineHeight: 1.3158
  body:
    fontFamily: System
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.5
  body-strong:
    fontFamily: System
    fontSize: 16px
    fontWeight: 600
    lineHeight: 1.5
  button:
    fontFamily: System
    fontSize: 16px
    fontWeight: 600
    lineHeight: 1.25
  meta:
    fontFamily: System
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.3846
  label:
    fontFamily: System
    fontSize: 12px
    fontWeight: 700
    lineHeight: 1.3333
    letterSpacing: 0.5px
rounded:
  sm: 6px
  md: 8px
  lg: 12px
  pill: 999px
spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 20px
  xxl: 24px
  xxxl: 32px
components:
  screen:
    backgroundColor: "{colors.canvas}"
    padding: "{spacing.xxl}"
  screen-header-kicker:
    textColor: "{colors.primary}"
    typography: "{typography.label}"
  screen-header-title:
    textColor: "{colors.ink900}"
    typography: "{typography.display}"
  screen-header-copy:
    textColor: "{colors.ink500}"
    typography: "{typography.body}"
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.onPrimary}"
    typography: "{typography.button}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.xl}"
  button-secondary:
    backgroundColor: "{colors.primarySubtle}"
    textColor: "{colors.primaryPressed}"
    typography: "{typography.button}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.xl}"
  button-danger:
    backgroundColor: "{colors.danger}"
    textColor: "{colors.onPrimary}"
    typography: "{typography.button}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.xl}"
  card:
    backgroundColor: "{colors.raised}"
    rounded: "{rounded.lg}"
    padding: "{spacing.lg}"
  panel:
    backgroundColor: "{colors.raised}"
    rounded: "{rounded.lg}"
    padding: "{spacing.lg}"
  badge:
    rounded: "{rounded.pill}"
    padding: "{spacing.xs}"
    typography: "{typography.label}"
  state-view:
    backgroundColor: "{colors.raised}"
    rounded: "{rounded.lg}"
    padding: "{spacing.xl}"
  video-player:
    backgroundColor: "{colors.ink900}"
    rounded: "{rounded.lg}"
---

# DubBridge Mobile DESIGN.md

## Overview

DubBridge Mobile is a professional, operational mobile workspace for governed media
review. It should feel calm, trustworthy, and precise rather than expressive,
playful, or marketing-led. The visual language is restrained on purpose: one teal
accent, ink-forward text, near-white surfaces, compact but breathable spacing, and
clear state feedback.

This is not a landing page system. It is an authenticated product surface for
repeated work: scanning lists, opening detail views, reviewing compliance status,
loading playback, making publication decisions, and recovering cleanly from loading
or failure states.

This file is an agent-readable design-intent contract. The runtime source of truth
remains `mobile/src/theme/tokens.ts`; if this file and the TypeScript tokens ever
drift, the shipped tokens win until the docs are synchronized.

## Colors

The palette is rooted in a dark canvas and a single Netflix-red accent (ADR-035).

- **Ink:** `ink900`, `ink700`, `ink500`, `ink400`, and `ink300` provide the full
  text hierarchy on dark surfaces. Values are inverted from the light palette —
  `ink900` is near-white (`#F5F5F5`). Use them for title, body, metadata, and
  disabled text before considering any semantic color.
- **Surface:** `canvas` (`#141414`) is the app background, `raised` (`#1F1F1F`)
  is the primary panel/card surface, and `sunken` (`#0A0A0A`) is the quiet inset
  surface for secondary emphasis.
- **Primary:** `primary` (`#E50914`) is the Netflix-red brand/action accent.
  `primaryPressed`, `primarySubtle`, and `onPrimary` define its interactive states.
- **Semantic:** `success`, `warning`, `danger`, and `info` are used sparingly for
  status communication, badges, destructive actions, and error states. Values are
  brightened for legibility on dark backgrounds. WCAG AA certification in T2.

Prefer calm contrast over visual noise. The app should read as operational and
legible first, branded second.

## Typography

DubBridge Mobile uses the platform native sans stack through React Native's default
`System` family. Hierarchy comes from size, weight, and spacing rather than from
mixing decorative typefaces.

- **Display:** screen titles only.
- **Title:** large object titles, such as key asset headings.
- **Heading:** section and card titles.
- **Body / body-strong:** explanatory copy and emphasized body copy.
- **Button:** action labels.
- **Meta:** identifiers, timestamps, secondary facts, and supporting detail.
- **Label:** uppercase kicker, eyebrow, field label, and compact status text.

Typography should feel deliberate and tight, but not compressed. Avoid oversized
hero typography inside operational surfaces.

## Layout

Layout is mobile-first and safe-area aware. Screens sit on `canvas` with real
top/bottom inset padding and a compact spacing scale. The system is dense enough for
work, but never cramped.

- Use `Screen` for page framing and safe-area padding.
- Use `ScreenHeader` for the opening hierarchy: kicker, title, optional copy.
- Group related information into `Panel` blocks.
- Use full-width buttons when the action is primary to the current task.
- Allow empty, error, and loading states to center cleanly instead of clinging to
  the top edge of the screen.

Primary mobile flows should remain thumb-friendly, scannable, and stable across
loading and retry cycles.

## Elevation & Depth

Depth is intentionally minimal.

- Tappable cards use one soft elevation level.
- Static panels rely on border separation rather than shadow.
- Video and playback surfaces may use a darker background for media framing, but
  they still sit inside the same overall surface hierarchy.

Avoid stacking multiple decorative depth treatments. If a surface is not tappable,
it usually should not float.

## Shapes

Corners are modest and utilitarian.

- `sm` and `md` support fields and buttons.
- `lg` supports cards, panels, and media frames.
- `pill` supports badges and compact metadata capsules.

Rounded corners should soften the interface without making it toy-like. Keep the
shape language consistent across buttons, panels, states, and media containers.

## Components

The component vocabulary is intentionally small and should stay small.

- **Screen:** safe-area aware canvas wrapper for every screen.
- **ScreenHeader:** consistent title block with a primary-colored kicker.
- **Button:** one action primitive with `primary`, `secondary`, and `danger`
  variants; disabled and loading states are part of the contract.
- **Card:** raised, tappable container for navigation and selectable items.
- **Panel:** static information container for grouped details and actions.
- **Badge:** compact semantic status pill; subtle background, strong foreground.
- **StateView:** standard loading, empty, and error treatment with optional retry.
- **PlaybackStateView / VideoPlayer:** media surfaces should feel native to the
  system, not like an embedded third-party widget bolted onto the screen.

Compose screens from these primitives before inventing a new surface. New mobile UI
work should preserve existing `testID`s and behavior while staying visually inside
this vocabulary.

## Do's and Don'ts

Do:

- Use the shipped token values from `mobile/src/theme/tokens.ts`.
- Keep the dark-canvas + Netflix-red identity intact (ADR-035).
- Prefer clear operational copy over engineering-harness language.
- Use semantic colors for state meaning, not for decoration.
- Keep primary actions obvious and secondary actions quiet.
- Preserve usability when playback, loading, or network states fail.

Don't:

- Introduce a second accent palette or screen-local hex values.
- Build marketing-style hero layouts, splashy gradients, or decorative chrome into
  authenticated mobile work surfaces.
- Nest cards inside cards or create fake depth with multiple shadow layers.
- Replace existing primitives with one-off containers unless the task explicitly
  changes the design system.
- Treat UI visibility as an authorization boundary.
- Let media playback surfaces break the surrounding design language.
