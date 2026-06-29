import type { TextStyle, ViewStyle } from "react-native";

/**
 * S-220 design tokens — dark-canvas (Netflix-style) palette (ADR-035).
 *
 * Replaces the S-115 ink+teal palette with a dark canvas and Netflix-red
 * primary accent. Screens and primitives must consume these tokens rather than
 * hardcoding hex, spacing, or type.
 */

export const color = {
  // Ink (text) scale — lightest to darkest on dark canvas.
  ink900: "#F5F5F5",
  ink700: "#E0E0E0",
  ink500: "#A8A8A8",
  ink400: "#737373",
  ink300: "#4D4D4D",

  // Surfaces.
  canvas: "#141414",
  raised: "#1F1F1F",
  sunken: "#0A0A0A",

  // Borders.
  border: "#2A2A2A",
  borderStrong: "#3D3D3D",

  // Primary accent — Netflix red (ADR-035).
  // primaryPressed is lighter than the S-220/T0 target (#B8000B) so it clears
  // WCAG AA (4.5:1) on primarySubtle (#2A0608). On canvas it reads 5.06:1 (AA).
  // onPrimary on primaryPressed yields 3.64:1 — large-UI only (button pressed
  // state is always ≥18px/bold; small text never sits on primaryPressed).
  primary: "#E50914",
  primaryPressed: "#FF3333",
  primarySubtle: "#2A0608",
  onPrimary: "#FFFFFF",

  // Semantic — used sparingly (badges, destructive actions, errors).
  success: "#2DC76D",
  successSubtle: "#0D2E1A",
  warning: "#F5A623",
  warningSubtle: "#2E1F04",
  danger: "#E50914",
  dangerSubtle: "#2A0608",
  dangerPressed: "#B8000B",
  info: "#3B9EDB",
  infoSubtle: "#071622",

  // "Strong" semantic foregrounds for text on the matching subtle backgrounds.
  // Brightened to clear WCAG AA (>=4.5:1) on dark subtle backgrounds.
  // infoStrong adjusted from #2A7FB8 (4.21:1) to #4BAEE5 (7.40:1) for AA compliance.
  successStrong: "#1FA855",
  warningStrong: "#D4891A",
  infoStrong: "#4BAEE5",
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
