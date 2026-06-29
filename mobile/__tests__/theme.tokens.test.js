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
  it("HP-1: refreshes the commercial palette without changing the semantic accent system", () => {
    expect(color.canvas).toBe("#F7F8FA");
    expect(color.sunken).toBe("#EEF0F4");
    expect(color.border).toBe("#E1E5EC");
    expect(color.primary).toBe("#097F67");
    expect(color.primaryPressed).toBe("#0A745E");

    expect(color.success).toBe("#1A7F5A");
    expect(color.warning).toBe("#9A6B12");
    expect(color.danger).toBe("#B3261E");
    expect(color.info).toBe("#1D5E84");

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

  it("EC-1: primarySubtle surfaces keep readable action contrast with the pressed teal foreground", () => {
    expect(contrastRatio(color.primaryPressed, color.primarySubtle)).toBeGreaterThanOrEqual(4.5);
  });

  it("EC-2: borderStrong stays visually stronger than the softened border", () => {
    expect(relativeLuminance(color.borderStrong)).toBeLessThan(relativeLuminance(color.border));
    expect(contrastRatio(color.borderStrong, color.border)).toBeGreaterThanOrEqual(1.2);
  });
});
