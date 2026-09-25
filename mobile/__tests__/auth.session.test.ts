import * as SecureStore from "expo-secure-store";

import {
  AUTH_CLOCK_SKEW_LEEWAY_SECONDS,
  clearAuthSession,
  isStoredAuthSessionValid,
  loadAuthSession,
  saveAuthSession,
  type AuthSession,
} from "../src/auth/session";

jest.mock("expo-secure-store", () => ({
  setItemAsync: jest.fn(),
  getItemAsync: jest.fn(),
  deleteItemAsync: jest.fn(),
}));

const SESSION_KEY = "dubbridge_auth_session";
const SESSION: AuthSession = {
  token: "token-abc",
  userId: "user-123",
  workspaceId: "workspace-456",
};


type Base64Globals = typeof globalThis & {
  btoa: (value: string) => string;
};

function base64UrlJson(value: unknown): string {
  return (globalThis as Base64Globals)
    .btoa(JSON.stringify(value))
    .replace(/=/g, "")
    .replace(/\+/g, "-")
    .replace(/\//g, "_");
}

function tokenWithPayload(payload: unknown): string {
  return `${base64UrlJson({ alg: "HS256", typ: "JWT" })}.${base64UrlJson(payload)}.signature`;
}

function sessionWithToken(token: string): AuthSession {
  return { ...SESSION, token };
}

describe("auth session storage", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it("HP-1: saves the bearer session payload as JSON", async () => {
    (SecureStore.setItemAsync as jest.Mock).mockResolvedValueOnce(undefined);

    await saveAuthSession(SESSION);

    expect(SecureStore.setItemAsync).toHaveBeenCalledWith(
      SESSION_KEY,
      JSON.stringify(SESSION),
    );
  });

  it("HP-2: loads a persisted bearer session payload", async () => {
    (SecureStore.getItemAsync as jest.Mock).mockResolvedValueOnce(
      JSON.stringify(SESSION),
    );

    await expect(loadAuthSession()).resolves.toEqual(SESSION);
  });

  it("EC-1: returns null for an invalid persisted payload shape", async () => {
    (SecureStore.getItemAsync as jest.Mock).mockResolvedValueOnce(
      JSON.stringify({ token: "token-abc", userId: "user-123" }),
    );

    await expect(loadAuthSession()).resolves.toBeNull();
  });

  it("EC-2: returns null for malformed JSON", async () => {
    (SecureStore.getItemAsync as jest.Mock).mockResolvedValueOnce("{bad json");

    await expect(loadAuthSession()).resolves.toBeNull();
  });

  it("HP-3: accepts a stored JWT that is still valid", () => {
    const now = 1_800_000_000;
    const session = sessionWithToken(tokenWithPayload({ exp: now + 300 }));

    expect(isStoredAuthSessionValid(session, now)).toBe(true);
  });

  it("EC-3: rejects a stored JWT expired beyond the configured leeway", () => {
    const now = 1_800_000_000;
    const session = sessionWithToken(
      tokenWithPayload({ exp: now - AUTH_CLOCK_SKEW_LEEWAY_SECONDS - 1 }),
    );

    expect(isStoredAuthSessionValid(session, now)).toBe(false);
  });

  it("EC-4: accepts expiry inside the clock-skew leeway", () => {
    const now = 1_800_000_000;
    const session = sessionWithToken(tokenWithPayload({ exp: now - 10 }));

    expect(isStoredAuthSessionValid(session, now)).toBe(true);
  });

  it("EC-5: rejects malformed JWTs and JWTs without a numeric exp", () => {
    expect(isStoredAuthSessionValid(sessionWithToken("not-a-jwt"))).toBe(false);
    expect(
      isStoredAuthSessionValid(sessionWithToken(tokenWithPayload({ sub: "user-123" }))),
    ).toBe(false);
    expect(
      isStoredAuthSessionValid(sessionWithToken(tokenWithPayload({ exp: "tomorrow" }))),
    ).toBe(false);
  });

  it("EC-6: clears the persisted bearer session", async () => {
    (SecureStore.deleteItemAsync as jest.Mock).mockResolvedValueOnce(undefined);

    await clearAuthSession();

    expect(SecureStore.deleteItemAsync).toHaveBeenCalledWith(SESSION_KEY);
  });
});
