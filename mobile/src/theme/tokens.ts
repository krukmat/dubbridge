import type { TextStyle, ViewStyle } from "react-native";

/**
 * S-115 design tokens — the single source of visual truth for the mobile app.
 *
 * One restrained "ink + teal" palette (ADR-029 mobile surface). Screens and
 * primitives must consume these tokens rather than hardcoding hex, spacing, or
 * type. See docs/plan/s-115-mobile-ux-foundation.md for the locked spec.
 */

export const color = {
  // Ink (text) scale — darkest to lightest.
  ink900: "#0F1B22",
  ink700: "#243640",
  ink500: "#4A5A63",
  ink400: "#647079",
  ink300: "#8A949B",

  // Surfaces.
  canvas: "#F4F7F6",
  raised: "#FFFFFF",
  sunken: "#EAF0EE",

  // Borders.
  border: "#D8E0DD",
  borderStrong: "#C2CDC8",

  // Primary accent (the single brand accent).
  primary: "#127C68",
  primaryPressed: "#0E6353",
  primarySubtle: "#E2EFEB",
  onPrimary: "#F7FBF9",

  // Semantic — used sparingly (badges, destructive actions, errors).
  success: "#1A7F5A",
  successSubtle: "#E3F2EA",
  warning: "#9A6B12",
  warningSubtle: "#F6ECD6",
  danger: "#B3261E",
  dangerSubtle: "#F7E4E2",
  dangerPressed: "#8F1E18",
  info: "#1D5E84",
  infoSubtle: "#E1ECF3",

  // "Strong" semantic foregrounds for text on the matching subtle backgrounds.
  // Darkened to clear WCAG AA (>=4.5:1) for small badge labels.
  successStrong: "#0F5C40",
  warningStrong: "#6E4C0D",
  infoStrong: "#16486A",
} as const;

export const space = {
  xs: 4,
  sm: 8,
  md: 12,
  lg: 16,
  xl: 20,
  xxl: 24,
  xxxl: 32,
} as const;

export const radius = {
  sm: 6,
  md: 8,
  lg: 12,
  pill: 999,
} as const;

/** One soft elevation level for tappable cards. Static panels use border only. */
export const elevation = {
  card: {
    shadowColor: color.ink900,
    shadowOpacity: 0.06,
    shadowRadius: 12,
    shadowOffset: { width: 0, height: 4 },
    elevation: 2,
  } satisfies ViewStyle,
} as const;

/**
 * Typography scale. Each entry is a ready-to-spread RN TextStyle fragment
 * (color is applied by the consumer so the same scale works on light/tinted
 * surfaces). `label` is the uppercase kicker/eyebrow/field-label style.
 */
export const type = {
  display: { fontSize: 32, fontWeight: "700", lineHeight: 38 },
  title: { fontSize: 24, fontWeight: "700", lineHeight: 30 },
  heading: { fontSize: 19, fontWeight: "700", lineHeight: 25 },
  body: { fontSize: 16, fontWeight: "400", lineHeight: 24 },
  bodyStrong: { fontSize: 16, fontWeight: "600", lineHeight: 24 },
  button: { fontSize: 16, fontWeight: "600", lineHeight: 20 },
  meta: { fontSize: 13, fontWeight: "400", lineHeight: 18 },
  label: {
    fontSize: 12,
    fontWeight: "700",
    lineHeight: 16,
    letterSpacing: 0.5,
    textTransform: "uppercase",
  },
} as const satisfies Record<string, TextStyle>;

/** Shared TextInput styling so forms stay consistent without a separate primitive. */
export const fieldStyle: TextStyle = {
  borderWidth: 1,
  borderColor: color.borderStrong,
  borderRadius: radius.md,
  backgroundColor: color.raised,
  paddingHorizontal: space.lg,
  paddingVertical: space.md,
  fontSize: 16,
  color: color.ink900,
};

export const theme = { color, space, radius, elevation, type, fieldStyle } as const;

export type Theme = typeof theme;
