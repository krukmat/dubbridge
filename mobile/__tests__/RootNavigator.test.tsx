import { cleanup, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import * as notifications from "../src/api/notifications";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { RootNavigator } from "../src/navigation/RootNavigator";

let mockExtra: {
  dubbridgeEnv?: unknown;
  gatewayBaseUrl?: unknown;
} = {};

let mockAuthValue: AuthContextValue;

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

jest.mock("../src/auth/AuthProvider", () => ({
  AuthProvider: ({ children }: { children: React.ReactNode }) => children,
  useAuth: () => mockAuthValue,
}));

jest.mock("../src/api/client", () => ({
  createGatewayClient: jest.fn(),
}));

jest.mock("../src/api/notifications", () => ({
  listNotifications: jest.fn(),
}));

jest.mock("../src/push/registerPush", () => ({
  registerPush: jest.fn().mockResolvedValue(undefined),
}));

const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;
const mockListNotifications = notifications.listNotifications as jest.MockedFunction<typeof notifications.listNotifications>;

jest.mock("expo-notifications", () => ({
  addNotificationResponseReceivedListener: jest.fn(() => ({
    remove: jest.fn(),
  })),
}));

jest.mock("../src/screens/ReviewInboxScreen", () => ({
  ReviewInboxScreen: () => null,
}));

jest.mock("../src/screens/ReviewDetailScreen", () => ({
  ReviewDetailScreen: () => null,
}));

jest.mock("expo-constants", () => ({
  __esModule: true,
  default: {
    get expoConfig() {
      return {
        extra: mockExtra,
      };
    },
  },
}));

describe("RootNavigator", () => {
  beforeEach(() => {
    const mockClient = { get: jest.fn(), post: jest.fn(), postMultipart: jest.fn() };
    mockClient.get.mockResolvedValue({ ok: true, value: { data: [], sessionRotation: null } });
    mockCreateGatewayClient.mockReturnValue(mockClient as any);
    mockListNotifications.mockResolvedValue({ ok: true, value: { data: { notifications: [] }, sessionRotation: null } });

    mockAuthValue = {
      sessionRef: null,
      status: "unauthed",
      loginError: null,
      login: jest.fn().mockResolvedValue(undefined),
      logout: jest.fn().mockResolvedValue(undefined),
      onSessionRotation: jest.fn().mockResolvedValue(undefined),
    };
  });

  afterEach(() => {
    cleanup();
    mockExtra = {};
  });

  it("HP-1: renders the bearer login entry screen when unauthenticated", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };

    const view = await render(<RootNavigator />);

    expect(view.getByTestId("login-screen")).toBeTruthy();
    expect(view.getByTestId("login-email-input")).toBeTruthy();
    expect(view.getByTestId("login-password-input")).toBeTruthy();
    expect(view.getByTestId("login-submit-button")).toBeTruthy();
  });

  it("HP-2: renders the authenticated home tree when bearer auth is present", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      sessionRef: "token-abc",
      status: "authed",
    };

    const view = await render(<RootNavigator />);

    expect(view.getByTestId("home-screen")).toBeTruthy();
    expect(view.getByText("Your workspace")).toBeTruthy();
    // AccountSection renders after dashboard data loads (ready state)
    await waitFor(() => expect(view.getByTestId("home-sign-out")).toBeTruthy());
  });

  it("EC-1: renders a config error when the gateway URL is missing", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: null,
    };

    const view = await render(<RootNavigator />);

    expect(view.getByTestId("config-error-screen")).toBeTruthy();
    expect(
      view.getByText(
        "Missing gateway base URL. Set EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL or DUBBRIDGE_GATEWAY_URL.",
      ),
    ).toBeTruthy();
  });

  it("EC-2: loading auth still stays on the unauthenticated navigator", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      status: "loading",
    };

    const view = await render(<RootNavigator />);

    expect(view.getByTestId("login-screen")).toBeTruthy();
    expect(view.queryByTestId("home-screen")).toBeNull();
  });

  it("EC-3: login failures render the generic credential error on the unauthenticated tree", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      loginError: "login_failed",
    };

    const view = await render(<RootNavigator />);

    expect(view.getByTestId("login-error-text")).toBeTruthy();
    expect(view.getByText("Invalid email or password.")).toBeTruthy();
  });
});
