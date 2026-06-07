import { cleanup, fireEvent, render } from "@testing-library/react-native";

import type { AuthContextValue } from "../src/auth/AuthProvider";
import { RootNavigator } from "../src/navigation/RootNavigator";

let mockExtra: {
  dubbridgeEnv?: unknown;
  gatewayBaseUrl?: unknown;
} = {};

let mockAuthValue: AuthContextValue;

jest.mock("../src/auth/AuthProvider", () => ({
  AuthProvider: ({ children }: { children: React.ReactNode }) => children,
  useAuth: () => mockAuthValue,
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
  afterEach(() => {
    cleanup();
    mockExtra = {};
  });

  beforeEach(() => {
    mockAuthValue = {
      sessionRef: null,
      status: "unauthed",
      loginError: null,
      login: jest.fn().mockResolvedValue(undefined),
      logout: jest.fn().mockResolvedValue(undefined),
      onSessionRotation: jest.fn().mockResolvedValue(undefined),
    };
  });

  it("renders the unauthenticated entry screen when runtime config is valid", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };

    const view = await render(<RootNavigator />);

    expect(view.getByText("DubBridge mobile")).toBeTruthy();
    expect(view.getByText("Sign in with session gateway")).toBeTruthy();
    expect(view.getByTestId("login-screen")).toBeTruthy();
  });

  it("renders the unauthenticated entry screen while auth is loading", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      status: "loading",
    };

    const view = await render(<RootNavigator />);

    expect(view.getByText("DubBridge mobile")).toBeTruthy();
    expect(view.getByText("Sign in with session gateway")).toBeTruthy();
    expect(view.getByTestId("login-screen")).toBeTruthy();
  });

  it("renders the authenticated home screen when auth status is authed", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      sessionRef: "opaque-session-abc123",
      status: "authed",
    };

    const view = await render(<RootNavigator />);

    expect(view.getByText("Mobile home")).toBeTruthy();
    expect(view.getByText("local")).toBeTruthy();
    expect(view.getByText("http://127.0.0.1:4000")).toBeTruthy();
    expect(view.getByText("Browse assets")).toBeTruthy();
    expect(view.getByText("Sign out")).toBeTruthy();
    expect(view.getByTestId("home-screen")).toBeTruthy();
  });

  it("renders a clear configuration error when the gateway URL is missing", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: null,
    };

    const view = await render(<RootNavigator />);

    expect(view.getByText("Configuration required")).toBeTruthy();
    expect(
      view.getByText(
        "Missing gateway base URL. Set EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL or DUBBRIDGE_GATEWAY_URL.",
      ),
    ).toBeTruthy();
    expect(view.getByTestId("config-error-screen")).toBeTruthy();
  });

  it("fails closed to the config error screen when Expo extra values are not strings", async () => {
    mockExtra = {
      dubbridgeEnv: { value: "local" },
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };

    const view = await render(<RootNavigator />);

    expect(view.getByText("Configuration required")).toBeTruthy();
    expect(
      view.getByText(
        "Missing DUBBRIDGE_ENV. Expected one of: local, staging, production.",
      ),
    ).toBeTruthy();
    expect(view.getByTestId("config-error-screen")).toBeTruthy();
  });

  it("wires the login screen button to auth.login()", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };

    const view = await render(<RootNavigator />);

    fireEvent.press(view.getByText("Sign in with session gateway"));

    expect(mockAuthValue.login).toHaveBeenCalledTimes(1);
  });

  it("wires the home screen button to auth.logout()", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      sessionRef: "opaque-session-abc123",
      status: "authed",
    };

    const view = await render(<RootNavigator />);

    fireEvent.press(view.getByText("Sign out"));

    expect(mockAuthValue.logout).toHaveBeenCalledTimes(1);
  });
});
