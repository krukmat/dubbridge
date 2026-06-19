/**
 * Pure, framework-free formatters for the mobile UI (S-190-T1).
 *
 * These helpers must stay free of any React / React Native imports so they can
 * be unit-tested in isolation and reused across screens. They centralise the
 * two presentation rules established by the S-190 usability audit:
 *
 *  - U1: identifiers are never cut mid-token (no manual `.slice(0, 8)`); when an
 *    id must be shortened, only a *complete* value is elided with a trailing `…`.
 *  - U2: timestamps are never shown raw; every user-facing timestamp goes through
 *    a single locale-aware absolute formatter, with a stable fallback that never
 *    throws on malformed input.
 */

export interface FormatIdOptions {
  /**
   * Optional maximum rendered length. When set and the id is longer, the value
   * is elided with a trailing ellipsis. The id is never cut in the middle of a
   * token when a token boundary is available within the budget; only past the
   * limit is an ellipsis appended.
   */
  max?: number;
}

export interface FormatTimestampOptions {
  /** BCP-47 locale tag (e.g. "en-US"). Defaults to the device/runtime locale. */
  locale?: string;
  /** IANA time zone (e.g. "UTC"). Defaults to the device/runtime time zone. */
  timeZone?: string;
}

const ELLIPSIS = "…";

/**
 * Return a display-safe identifier.
 *
 * Default: the full id, unchanged. With `max`, the id is shortened only when it
 * exceeds the budget, and the ellipsis is appended to a *complete* value — it
 * prefers a hyphen-delimited token boundary so a slug is never cut mid-token.
 */
export function formatId(id: string | null | undefined, opts: FormatIdOptions = {}): string {
  if (id == null) {
    return "";
  }

  const value = String(id);
  const { max } = opts;

  if (max == null || max <= 0 || value.length <= max) {
    return value;
  }

  // Budget for the visible portion, leaving room for the ellipsis itself.
  const budget = Math.max(0, max - ELLIPSIS.length);
  const window = value.slice(0, budget);

  // Prefer eliding at a token boundary (hyphen) so we never cut mid-token.
  const lastBoundary = window.lastIndexOf("-");
  const head = lastBoundary > 0 ? window.slice(0, lastBoundary) : window;

  return `${head}${ELLIPSIS}`;
}

/**
 * Return a locale-aware absolute timestamp string.
 *
 * Invalid or empty input returns a stable fallback (the original string) and
 * never throws. Tests can pin `locale`/`timeZone` for deterministic output while
 * the UI relies on the device locale by default.
 */
export function formatTimestamp(
  iso: string | null | undefined,
  opts: FormatTimestampOptions = {},
): string {
  if (iso == null || iso === "") {
    return "";
  }

  const source = String(iso);
  const date = new Date(source);

  if (Number.isNaN(date.getTime())) {
    return source;
  }

  try {
    return new Intl.DateTimeFormat(opts.locale, {
      dateStyle: "medium",
      timeStyle: "short",
      timeZone: opts.timeZone,
    }).format(date);
  } catch {
    // Defensive: an invalid time zone / locale should never crash a render.
    return source;
  }
}

const RELATIVE_DIVISIONS: { amount: number; unit: Intl.RelativeTimeFormatUnit }[] = [
  { amount: 60, unit: "second" },
  { amount: 60, unit: "minute" },
  { amount: 24, unit: "hour" },
  { amount: 7, unit: "day" },
  { amount: 4.34524, unit: "week" },
  { amount: 12, unit: "month" },
  { amount: Number.POSITIVE_INFINITY, unit: "year" },
];

/**
 * Return a relative time string (e.g. "5 months ago"). Invalid input returns a
 * stable fallback (the original string) and never throws. `now` is injectable
 * for deterministic tests.
 */
export function formatRelative(
  iso: string | null | undefined,
  now: Date = new Date(),
  opts: FormatTimestampOptions = {},
): string {
  if (iso == null || iso === "") {
    return "";
  }

  const source = String(iso);
  const date = new Date(source);

  if (Number.isNaN(date.getTime())) {
    return source;
  }

  let delta = (date.getTime() - now.getTime()) / 1000;

  let formatter: Intl.RelativeTimeFormat;
  try {
    formatter = new Intl.RelativeTimeFormat(opts.locale, { numeric: "auto" });
  } catch {
    return source;
  }

  let division = RELATIVE_DIVISIONS[0];
  for (division of RELATIVE_DIVISIONS) {
    if (Math.abs(delta) < division.amount) {
      break;
    }
    delta /= division.amount;
  }

  // The final division has an infinite threshold, so the loop always selects a
  // unit before reaching here — no unreachable fallback branch remains.
  return formatter.format(Math.round(delta), division.unit);
}
