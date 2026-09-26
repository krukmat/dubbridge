import * as SecureStore from "expo-secure-store";

const SESSION_KEY = "dubbridge_auth_session";


/**
 * Mobile hydration leeway mirrors the POC auth profile. The client only uses
 * this to avoid restoring a clearly expired bearer; the API remains the
 * authority for signature and authorization checks.
 */
export const AUTH_CLOCK_SKEW_LEEWAY_SECONDS = 30;

type Base64Globals = typeof globalThis & {
  atob?: (encoded: string) => string;
};

function decodeBase64UrlJson(value: string): unknown | null {
  const base64 = value.replace(/-/g, "+").replace(/_/g, "/");
  const remainder = base64.length % 4;
  const padded = remainder === 0 ? base64 : base64 + "=".repeat(4 - remainder);
  const decode = (globalThis as Base64Globals).atob;

  if (typeof decode !== "function") return null;

  try {
    return JSON.parse(decode(padded)) as unknown;
  } catch {
    return null;
  }
}

function readPositiveIntegerClaim(
  payload: unknown,
  claim: string,
): number | null {
  if (typeof payload !== "object" || payload === null) return null;

  const value = (payload as Record<string, unknown>)[claim];
  return typeof value === "number" && Number.isSafeInteger(value) && value > 0
    ? value
    : null;
}

function readJwtExpirySeconds(token: string): number | null {
  const parts = token.split(".");
  if (parts.length !== 3 || parts[1].length === 0) return null;

  return readPositiveIntegerClaim(decodeBase64UrlJson(parts[1]), "exp");
}

export function isStoredAuthSessionValid(
  session: AuthSession,
  nowSeconds = Math.floor(Date.now() / 1000),
  clockSkewLeewaySeconds = AUTH_CLOCK_SKEW_LEEWAY_SECONDS,
): boolean {
  const expiry = readJwtExpirySeconds(session.token);
  if (expiry === null) {
    return false;
  }

  const leeway = Math.max(0, clockSkewLeewaySeconds);
  return nowSeconds <= expiry + leeway;
}

export type AuthSession = {
  token: string;
  userId: string;
  workspaceId: string;
};

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isAuthSession(value: unknown): value is AuthSession {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const candidate = value as Record<string, unknown>;

  return (
    isNonEmptyString(candidate.token) &&
    isNonEmptyString(candidate.userId) &&
    isNonEmptyString(candidate.workspaceId)
  );
}

export async function saveAuthSession(session: AuthSession): Promise<void> {
  await SecureStore.setItemAsync(SESSION_KEY, JSON.stringify(session));
}

export async function loadAuthSession(): Promise<AuthSession | null> {
  const rawValue = await SecureStore.getItemAsync(SESSION_KEY);

  if (rawValue === null) {
    return null;
  }

  try {
    const parsed = JSON.parse(rawValue) as unknown;
    if (!isAuthSession(parsed)) {
      return null;
    }

    return {
      token: parsed.token.trim(),
      userId: parsed.userId.trim(),
      workspaceId: parsed.workspaceId.trim(),
    };
  } catch {
    return null;
  }
}

export async function clearAuthSession(): Promise<void> {
  await SecureStore.deleteItemAsync(SESSION_KEY);
}
