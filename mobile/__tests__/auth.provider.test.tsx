import { act, cleanup, render, waitFor } from "@testing-library/react-native";
import { Text } from "react-native";

import { createGatewayClient } from "../src/api/client";
import {
  AuthProvider,
  useAuth,
  type AuthContextValue,
} from "../src/auth/AuthProvider";
import {
  clearAuthSession,
  loadAuthSession,
  saveAuthSession,
  type AuthSession,
} from "../src/auth/session";
import { readRuntimeConfig } from "../src/config/env";

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

jest.mock("../src/auth/session", () => ({
  loadAuthSession: jest.fn(),
  saveAuthSession: jest.fn(),
  clearAuthSession: jest.fn(),
}));

jest.mock("../src/config/env", () => ({
  readRuntimeConfig: jest.fn(),
}));

jest.mock("../src/api/client", () => ({
  createGatewayClient: jest.fn(),
}));

const STORED_SESSION: AuthSession = {
  token: "token-abc",
  userId: "user-123",
  workspaceId: "workspace-456",
};

const LOGIN_PAYLOAD = {
  token: "token-login",
  userId: "user-login",
  workspaceId: "workspace-login",
};

const mockLoadAuthSession =
  loadAuthSession as jest.MockedFunction<typeof loadAuthSession>;
const mockSaveAuthSession =
  saveAuthSession as jest.MockedFunction<typeof saveAuthSession>;
const mockClearAuthSession =
  clearAuthSession as jest.MockedFunction<typeof clearAuthSession>;
const mockReadRuntimeConfig =
  readRuntimeConfig as jest.MockedFunction<typeof readRuntimeConfig>;
const mockCreateGatewayClient =
  createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;

type MockGatewayClient = {
  post: jest.Mock;
};

let latestAuthValue: AuthContextValue | null = null;
let mockGatewayClient: MockGatewayClient;

function AuthProbe() {
  const auth = useAuth();
  latestAuthValue = auth;

  return (
    <>
      <Text>{`status:${auth.status}`}</Text>
      <Text>{`sessionRef:${auth.sessionRef ?? "null"}`}</Text>
      <Text>{`loginError:${auth.loginError ?? "null"}`}</Text>
    </>
  );
}

describe("AuthProvider", () => {
  beforeEach(() => {
    latestAuthValue = null;
    jest.clearAllMocks();

    mockLoadAuthSession.mockResolvedValue(null);
    mockSaveAuthSession.mockResolvedValue(undefined);
    mockClearAuthSession.mockResolvedValue(undefined);
    mockReadRuntimeConfig.mockReturnValue({
      ok: true,
      value: {
        dubbridgeEnv: "local",
        gatewayBaseUrl: "http://127.0.0.1:4000",
      },
    });

    mockGatewayClient = {
      post: jest.fn().mockResolvedValue({
        ok: true,
        value: {
          data: LOGIN_PAYLOAD,
          sessionRotation: null,
        },
      }),
    };

    mockCreateGatewayClient.mockReturnValue(
      mockGatewayClient as unknown as ReturnType<typeof createGatewayClient>,
    );
  });

  afterEach(() => {
    cleanup();
  });

  it("HP-1: hydrates a persisted bearer session into authed state", async () => {
    mockLoadAuthSession.mockResolvedValueOnce(STORED_SESSION);

    const view = await render(
      <AuthProvider>
        <AuthProbe />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(view.getByText("status:authed")).toBeTruthy();
    });

    expect(view.getByText(`sessionRef:${STORED_SESSION.token}`)).toBeTruthy();
  });

  it("HP-2: login persists the bearer session and authenticates", async () => {
    const view = await render(
      <AuthProvider>
        <AuthProbe />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(view.getByText("status:unauthed")).toBeTruthy();
    });

    await act(async () => {
      await latestAuthValue?.login("user@example.com", "password-123456");
    });

    expect(mockGatewayClient.post).toHaveBeenCalledWith(
      "/auth/login",
      null,
      {
        email: "user@example.com",
        password: "password-123456",
      },
    );
    expect(mockSaveAuthSession).toHaveBeenCalledWith(LOGIN_PAYLOAD);
    expect(view.getByText("status:authed")).toBeTruthy();
    expect(view.getByText(`sessionRef:${LOGIN_PAYLOAD.token}`)).toBeTruthy();
    expect(view.getByText("loginError:null")).toBeTruthy();
  });

  it("EC-1: invalid credentials stay unauthed and clear persisted state", async () => {
    mockGatewayClient.post.mockResolvedValueOnce({
      ok: false,
      error: { kind: "session_expired" },
    });

    const view = await render(
      <AuthProvider>
        <AuthProbe />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(view.getByText("status:unauthed")).toBeTruthy();
    });

    await act(async () => {
      await latestAuthValue?.login("user@example.com", "wrong-password");
    });

    expect(mockSaveAuthSession).not.toHaveBeenCalled();
    expect(mockClearAuthSession).toHaveBeenCalled();
    expect(view.getByText("status:unauthed")).toBeTruthy();
    expect(view.getByText("loginError:login_failed")).toBeTruthy();
  });

  it("EC-2: malformed stored payload fails closed to unauthed state", async () => {
    mockLoadAuthSession.mockRejectedValueOnce(new Error("secure-store failure"));

    const view = await render(
      <AuthProvider>
        <AuthProbe />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(view.getByText("status:unauthed")).toBeTruthy();
    });

    expect(mockClearAuthSession).toHaveBeenCalledTimes(1);
    expect(view.getByText("sessionRef:null")).toBeTruthy();
  });

  it("EC-3: logout clears local bearer state fail-closed", async () => {
    mockLoadAuthSession.mockResolvedValueOnce(STORED_SESSION);

    const view = await render(
      <AuthProvider>
        <AuthProbe />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(view.getByText("status:authed")).toBeTruthy();
    });

    await act(async () => {
      await latestAuthValue?.logout();
    });

    expect(mockClearAuthSession).toHaveBeenCalled();
    expect(view.getByText("status:unauthed")).toBeTruthy();
    expect(view.getByText("sessionRef:null")).toBeTruthy();
  });
});
