import * as SecureStore from "expo-secure-store";

const SESSION_KEY = "dubbridge_auth_session";

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
