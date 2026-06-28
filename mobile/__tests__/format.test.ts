import { formatId, formatRelative, formatStatusLabel, formatTimestamp } from "../src/format";

describe("formatId", () => {
  it("HP-1: returns the full id unchanged by default", () => {
    expect(formatId("asset-seed-1")).toBe("asset-seed-1");
    expect(formatId("project-seed-1")).toBe("project-seed-1");
  });

  it("returns the full id when shorter than or equal to max", () => {
    expect(formatId("asset-1", { max: 20 })).toBe("asset-1");
    expect(formatId("asset-1", { max: 7 })).toBe("asset-1");
  });

  it("elides only past the limit and never cuts mid-token", () => {
    const out = formatId("asset-seed-1234567890", { max: 12 });
    expect(out.endsWith("…")).toBe(true);
    // Never the raw mid-token cut the audit flagged.
    expect(out).not.toBe("asset-se");
    // The visible portion ends on a complete token boundary.
    expect(out).toBe("asset-seed…");
  });

  it("falls back to a windowed elide when no token boundary fits the budget", () => {
    const out = formatId("abcdefghijklmnop", { max: 6 });
    expect(out.endsWith("…")).toBe(true);
    expect(out.length).toBeLessThanOrEqual(6);
  });

  it("EC-1: empty input returns empty string without throwing", () => {
    expect(formatId("", { max: 8 })).toBe("");
  });

  it("EC-1: null / undefined return empty string", () => {
    expect(formatId(null)).toBe("");
    expect(formatId(undefined)).toBe("");
  });

  it("treats non-positive max as no limit", () => {
    expect(formatId("asset-seed-1", { max: 0 })).toBe("asset-seed-1");
    expect(formatId("asset-seed-1", { max: -5 })).toBe("asset-seed-1");
  });
});

describe("formatTimestamp", () => {
  it("HP-2: returns a locale absolute string with no raw ISO marker", () => {
    const out = formatTimestamp("2026-01-01T11:00:00Z", {
      locale: "en-US",
      timeZone: "UTC",
    });
    expect(out).not.toMatch(/\d{4}-\d{2}-\d{2}T.*Z/);
    expect(out).toMatch(/2026/);
    expect(out).toMatch(/Jan/);
  });

  it("is deterministic for a pinned locale and time zone", () => {
    const out = formatTimestamp("2026-01-01T11:00:00Z", {
      locale: "en-US",
      timeZone: "UTC",
    });
    expect(out).toBe("Jan 1, 2026, 11:00 AM");
  });

  it("EC-2: malformed timestamp returns the original string, no throw", () => {
    expect(formatTimestamp("not-a-date")).toBe("not-a-date");
  });

  it("empty / null / undefined return empty string", () => {
    expect(formatTimestamp("")).toBe("");
    expect(formatTimestamp(null)).toBe("");
    expect(formatTimestamp(undefined)).toBe("");
  });

  it("returns the original string when the time zone is invalid", () => {
    expect(formatTimestamp("2026-01-01T11:00:00Z", { timeZone: "Not/AZone" })).toBe(
      "2026-01-01T11:00:00Z",
    );
  });
});

describe("formatStatusLabel", () => {
  it("HP-3: maps known domain statuses to product labels", () => {
    expect(formatStatusLabel("finalized")).toBe("Ready");
    expect(formatStatusLabel("in_review")).toBe("In review");
    expect(formatStatusLabel("approved")).toBe("Approved");
  });

  it("supports consent-specific labels without leaking raw enum values", () => {
    expect(formatStatusLabel("grant", "consent")).toBe("Active");
    expect(formatStatusLabel("revoke", "consent")).toBe("Inactive");
  });

  it("EC-3: unknown values degrade to a humanized fallback", () => {
    expect(formatStatusLabel("needs_manual_check")).toBe("Needs Manual Check");
    expect(formatStatusLabel("")).toBe("");
    expect(formatStatusLabel(null)).toBe("");
  });
});

describe("formatRelative", () => {
  const now = new Date("2026-06-01T00:00:00Z");

  it("formats a past timestamp as a relative string", () => {
    expect(formatRelative("2026-01-01T00:00:00Z", now, { locale: "en-US" })).toBe(
      "5 months ago",
    );
  });

  it("formats a recent past timestamp in smaller units", () => {
    expect(formatRelative("2026-05-31T23:59:30Z", now, { locale: "en-US" })).toBe(
      "30 seconds ago",
    );
  });

  it("formats a future timestamp", () => {
    expect(formatRelative("2026-06-01T01:00:00Z", now, { locale: "en-US" })).toBe(
      "in 1 hour",
    );
  });

  it("malformed input returns the original string", () => {
    expect(formatRelative("not-a-date", now)).toBe("not-a-date");
  });

  it("returns the original string when the locale is invalid", () => {
    expect(formatRelative("2026-01-01T00:00:00Z", now, { locale: "not a locale" })).toBe(
      "2026-01-01T00:00:00Z",
    );
  });

  it("empty / null / undefined return empty string", () => {
    expect(formatRelative("", now)).toBe("");
    expect(formatRelative(null, now)).toBe("");
    expect(formatRelative(undefined, now)).toBe("");
  });
});
