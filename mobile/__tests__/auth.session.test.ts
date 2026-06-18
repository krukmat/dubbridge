import * as SecureStore from "expo-secure-store";

import {
  clearAuthSession,
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

  it("EC-3: clears the persisted bearer session", async () => {
    (SecureStore.deleteItemAsync as jest.Mock).mockResolvedValueOnce(undefined);

    await clearAuthSession();

    expect(SecureStore.deleteItemAsync).toHaveBeenCalledWith(SESSION_KEY);
  });
});
