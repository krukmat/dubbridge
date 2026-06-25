---
version: alpha
name: DubBridge Mobile
description: Agent-readable design intent for the first-party DubBridge mobile app. Mirrors the shipped React Native token system; does not replace runtime styling in mobile/src/theme/tokens.ts.
colors:
  ink900: "#0F1B22"
  ink700: "#243640"
  ink500: "#4A5A63"
  ink400: "#647079"
  ink300: "#8A949B"
  canvas: "#F4F7F6"
  raised: "#FFFFFF"
  sunken: "#EAF0EE"
  border: "#D8E0DD"
  borderStrong: "#C2CDC8"
  primary: "#127C68"
  primaryPressed: "#0E6353"
  primarySubtle: "#E2EFEB"
  onPrimary: "#F7FBF9"
  success: "#1A7F5A"
  successSubtle: "#E3F2EA"
  successStrong: "#0F5C40"
  warning: "#9A6B12"
  warningSubtle: "#F6ECD6"
  warningStrong: "#6E4C0D"
  danger: "#B3261E"
  dangerSubtle: "#F7E4E2"
  dangerPressed: "#8F1E18"
  info: "#1D5E84"
  infoSubtle: "#E1ECF3"
  infoStrong: "#16486A"
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

The palette is rooted in deep ink neutrals and a single teal accent.

- **Ink:** `ink900`, `ink700`, `ink500`, `ink400`, and `ink300` provide the full
  text hierarchy. Use them for title, body, metadata, and disabled text before
  considering any semantic color.
- **Surface:** `canvas` is the app background, `raised` is the primary panel/card
  surface, and `sunken` is the quiet inset/tinted surface for secondary emphasis.
- **Primary:** `primary` is the only brand/action accent. `primaryPressed`,
  `primarySubtle`, and `onPrimary` define its interactive states.
- **Semantic:** `success`, `warning`, `danger`, and `info` are used sparingly for
  status communication, badges, destructive actions, and error states.

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
- Keep the "ink + teal" identity intact.
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
