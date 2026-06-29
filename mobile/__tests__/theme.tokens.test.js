const fs = require("fs");
const path = require("path");

const { color } = require("../src/theme");

function hexToRgb(hex) {
  const normalized = hex.replace("#", "");
  return [
    Number.parseInt(normalized.slice(0, 2), 16) / 255,
    Number.parseInt(normalized.slice(2, 4), 16) / 255,
    Number.parseInt(normalized.slice(4, 6), 16) / 255,
  ];
}

function linearize(channel) {
  return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
}

function relativeLuminance(hex) {
  const [r, g, b] = hexToRgb(hex).map(linearize);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrastRatio(foreground, background) {
  const [lighter, darker] = [relativeLuminance(foreground), relativeLuminance(background)].sort(
    (left, right) => right - left,
  );
  return (lighter + 0.05) / (darker + 0.05);
}

describe("T8 palette tokens", () => {
  it("HP-1: dark-canvas palette (S-220/ADR-035) — surface, primary, and semantic values", () => {
    expect(color.canvas).toBe("#141414");
    expect(color.sunken).toBe("#0A0A0A");
    expect(color.border).toBe("#2A2A2A");
    expect(color.primary).toBe("#E50914");
    expect(color.primaryPressed).toBe("#FF3333");

    expect(color.success).toBe("#2DC76D");
    expect(color.warning).toBe("#F5A623");
    expect(color.danger).toBe("#E50914");
    expect(color.info).toBe("#3B9EDB");

    expect(contrastRatio(color.onPrimary, color.primary)).toBeGreaterThanOrEqual(4.5);
  });

  it("HP-2: keeps DESIGN.md synchronized with the shipped runtime tokens", () => {
    const designDoc = fs.readFileSync(path.resolve(__dirname, "../../DESIGN.md"), "utf8");

    expect(designDoc).toContain(`canvas: "${color.canvas}"`);
    expect(designDoc).toContain(`sunken: "${color.sunken}"`);
    expect(designDoc).toContain(`border: "${color.border}"`);
    expect(designDoc).toContain(`primary: "${color.primary}"`);
    expect(designDoc).toContain(`primaryPressed: "${color.primaryPressed}"`);
  });

  it("EC-1: primary reading text (ink900) meets WCAG AA on dark canvas", () => {
    // Full primary/canvas and primaryPressed/primarySubtle contrast cert is a T2 gate.
    // This gate checks the most critical pair: body text on the app background.
    expect(contrastRatio(color.ink900, color.canvas)).toBeGreaterThanOrEqual(4.5);
  });

  it("EC-2: borderStrong is darker than border on dark canvas (higher luminance = lighter)", () => {
    // On a dark canvas both borders are dark; borderStrong is slightly lighter than border.
    expect(relativeLuminance(color.borderStrong)).toBeGreaterThan(relativeLuminance(color.border));
    expect(contrastRatio(color.borderStrong, color.border)).toBeGreaterThanOrEqual(1.2);
  });
});

// T2 — WCAG AA contrast certification suite (S-220)
describe("T2 WCAG AA contrast certification", () => {
  // ── Ink scale ────────────────────────────────────────────────────────────
  it("HP-3: ink900 on canvas meets AA 4.5:1 (primary reading text)", () => {
    expect(contrastRatio(color.ink900, color.canvas)).toBeGreaterThanOrEqual(4.5);
  });

  it("HP-4: ink900 on raised meets AA 4.5:1 (card body text)", () => {
    expect(contrastRatio(color.ink900, color.raised)).toBeGreaterThanOrEqual(4.5);
  });

  it("HP-5: ink900 on sunken meets AA 4.5:1 (recessed surface text)", () => {
    expect(contrastRatio(color.ink900, color.sunken)).toBeGreaterThanOrEqual(4.5);
  });

  // ── Primary accent ───────────────────────────────────────────────────────
  it("HP-6: onPrimary on primary meets AA 4.5:1 (button label on red)", () => {
    expect(contrastRatio(color.onPrimary, color.primary)).toBeGreaterThanOrEqual(4.5);
  });

  it("HP-7: primaryPressed on primarySubtle meets AA 4.5:1 (pressed-state text on subtle bg)", () => {
    expect(contrastRatio(color.primaryPressed, color.primarySubtle)).toBeGreaterThanOrEqual(4.5);
  });

  // ── Semantic — success ───────────────────────────────────────────────────
  it("HP-8: success on successSubtle meets AA 4.5:1", () => {
    expect(contrastRatio(color.success, color.successSubtle)).toBeGreaterThanOrEqual(4.5);
  });

  it("HP-9: successStrong on successSubtle meets AA 4.5:1", () => {
    expect(contrastRatio(color.successStrong, color.successSubtle)).toBeGreaterThanOrEqual(4.5);
  });

  // ── Semantic — warning ───────────────────────────────────────────────────
  it("HP-10: warning on warningSubtle meets AA 4.5:1", () => {
    expect(contrastRatio(color.warning, color.warningSubtle)).toBeGreaterThanOrEqual(4.5);
  });

  it("HP-11: warningStrong on warningSubtle meets AA 4.5:1", () => {
    expect(contrastRatio(color.warningStrong, color.warningSubtle)).toBeGreaterThanOrEqual(4.5);
  });

  // ── Semantic — info ──────────────────────────────────────────────────────
  it("HP-12: info on infoSubtle meets AA 4.5:1", () => {
    expect(contrastRatio(color.info, color.infoSubtle)).toBeGreaterThanOrEqual(4.5);
  });

  it("HP-13: infoStrong on infoSubtle meets AA 4.5:1", () => {
    expect(contrastRatio(color.infoStrong, color.infoSubtle)).toBeGreaterThanOrEqual(4.5);
  });

  // ── Edge cases ───────────────────────────────────────────────────────────

  // EC-3: primary on canvas — 3.84:1. Below 4.5:1 for small text but above
  // 3:1 large-UI threshold (WCAG 1.4.11 non-text / 1.4.3 large text ≥18px bold).
  // primary (#E50914) is ONLY used for: CTA buttons (≥16px/600 = large), icons,
  // and the Netflix-red brand stripe. Never for body or meta text on canvas.
  it("EC-3: primary on canvas meets large-UI threshold 3:1 (CTA/icon use only)", () => {
    expect(contrastRatio(color.primary, color.canvas)).toBeGreaterThanOrEqual(3.0);
  });

  // EC-4: danger === primary (both #E50914) — T0 decision gate resolved.
  // Semantic overlap is accepted; destructive actions are distinguished from
  // primary actions by shape (outlined/ghost vs filled) and label, not hue.
  // This test asserts the deliberate equality so future token changes are explicit.
  it("EC-4: danger equals primary — deliberate semantic overlap (shape/label distinction)", () => {
    expect(color.danger).toBe(color.primary);
    // danger on canvas also clears 3:1 large-UI (same as EC-3)
    expect(contrastRatio(color.danger, color.canvas)).toBeGreaterThanOrEqual(3.0);
  });

  // EC-5: ink500 (muted/meta text) on canvas — must clear AA for body use.
  it("EC-5: ink500 on canvas meets AA 4.5:1 (muted/meta text)", () => {
    expect(contrastRatio(color.ink500, color.canvas)).toBeGreaterThanOrEqual(4.5);
  });
});
