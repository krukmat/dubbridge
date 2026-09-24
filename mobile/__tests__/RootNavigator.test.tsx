import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import * as notifications from "../src/api/notifications";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { RootNavigator } from "../src/navigation/RootNavigator";

let mockExtra: {
  dubbridgeEnv?: unknown;
  gatewayBaseUrl?: unknown;
} = {};

let mockAuthValue: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };

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

const mockP2PSyncController = {
  getSyncState: jest.fn(),
  startSync: jest.fn(),
  getVerifiedPackageHandle: jest.fn(),
};
const mockP2PService = {};
jest.mock("../src/p2p/P2PProvider", () => ({
  useP2PSyncController: () => mockP2PSyncController,
  useP2PService: () => mockP2PService,
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
    mockClient = { get: jest.fn(), post: jest.fn(), postMultipart: jest.fn() };
    mockClient.get.mockResolvedValue({ ok: true, value: { data: [], sessionRotation: null } });
    mockCreateGatewayClient.mockReturnValue(mockClient as any);
    mockListNotifications.mockResolvedValue({ ok: true, value: { data: { notifications: [] }, sessionRotation: null } });

    mockAuthValue = {
      sessionRef: null,
      userId: null,
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

  it("P6.T1.F: navigates Home → My Content → Back and remounts without reviving the raw token", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      sessionRef: "token-abc",
      status: "authed",
    };
    mockClient.get.mockImplementation((path: string) => {
      if (path === "/api/p2p/content") {
        return Promise.resolve({
          ok: true,
          value: {
            data: [
              {
                asset_id: "asset-ready",
                title: "Ready package",
                publication_id: "pub-ready",
                lineage_id: "lineage-ready",
                state: "ready",
                descriptor: {
                  descriptor_version: "p2p-ready-v1",
                  asset_id: "asset-ready",
                  publication_id: "pub-ready",
                  lineage_id: "lineage-ready",
                  manifest_version: "p2p-manifest-v1",
                  manifest_digest_sha256: "a".repeat(64),
                  external_publication_id: "hyperdrive:ready",
                  ck_wrap_ref: "k1:ready",
                  kek_id: "kek-1",
                  kek_version: 1,
                  ready_at: "2026-09-22T18:00:00Z",
                },
              },
            ],
            sessionRotation: null,
          },
        });
      }
      return Promise.resolve({ ok: true, value: { data: [], sessionRotation: null } });
    });
    mockClient.post.mockResolvedValue({
      ok: true,
      value: {
        data: {
          invitation: {
            id: "invite-1",
            asset_id: "asset-ready",
            publication_id: "pub-ready",
            lineage_id: "lineage-ready",
            owner_subject_id: "owner-1",
            status: "pending",
            expires_at_unix: 2_000_000_000,
            claimed_at_unix: null,
          },
          token: "navigation-secret",
        },
        sessionRotation: null,
      },
    });

    const view = await render(<RootNavigator />);
    await waitFor(() => expect(view.getByTestId("home-open-my-content")).toBeTruthy());

    fireEvent.press(view.getByTestId("home-open-my-content"));
    await waitFor(() => expect(view.getByTestId("my-content-screen")).toBeTruthy());
    expect(
      mockClient.get.mock.calls.filter(([path]) => path === "/api/p2p/content"),
    ).toHaveLength(1);

    await act(async () => {
      fireEvent.press(view.getByTestId("my-content-create-invite-asset-ready"));
    });
    await waitFor(() => expect(view.getByText("navigation-secret")).toBeTruthy());

    fireEvent.press(view.getByTestId("my-content-back"));
    await waitFor(() => expect(view.getByTestId("home-screen")).toBeTruthy());
    expect(view.queryByText("navigation-secret")).toBeNull();

    fireEvent.press(view.getByTestId("home-open-my-content"));
    await waitFor(() => expect(view.getByTestId("my-content-screen")).toBeTruthy());
    expect(view.queryByText("navigation-secret")).toBeNull();
    expect(
      mockClient.get.mock.calls.filter(([path]) => path === "/api/p2p/content"),
    ).toHaveLength(2);
    expect(mockClient.post).toHaveBeenCalledTimes(1);
  });

  it("P6.T1.F: authenticated route disappears when auth state becomes unauthenticated", async () => {
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
    await waitFor(() => expect(view.getByTestId("home-open-my-content")).toBeTruthy());
    fireEvent.press(view.getByTestId("home-open-my-content"));
    await waitFor(() => expect(view.getByTestId("my-content-screen")).toBeTruthy());

    mockAuthValue = {
      ...mockAuthValue,
      sessionRef: null,
      status: "unauthed",
    };
    await act(async () => {
      view.rerender(<RootNavigator />);
    });

    await waitFor(() => expect(view.getByTestId("login-screen")).toBeTruthy());
    expect(view.queryByTestId("my-content-screen")).toBeNull();
  });


  it("P6.T2.G: navigates Home → Invites → Back and re-entry refetches the authoritative inbox", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      sessionRef: "token-abc",
      userId: "viewer-1",
      status: "authed",
    };
    mockClient.get.mockImplementation((path: string) => Promise.resolve({
      ok: true,
      value: { data: [], sessionRotation: null },
    }));

    const view = await render(<RootNavigator />);
    await waitFor(() => expect(view.getByTestId("home-open-invites")).toBeTruthy());

    fireEvent.press(view.getByTestId("home-open-invites"));
    await waitFor(() => expect(view.getByTestId("invites-screen")).toBeTruthy());
    expect(
      mockClient.get.mock.calls.filter(([path]) => path === "/api/p2p/inbox"),
    ).toHaveLength(1);

    fireEvent.press(view.getByTestId("invites-back"));
    await waitFor(() => expect(view.getByTestId("home-screen")).toBeTruthy());

    fireEvent.press(view.getByTestId("home-open-invites"));
    await waitFor(() => expect(view.getByTestId("invites-screen")).toBeTruthy());
    expect(
      mockClient.get.mock.calls.filter(([path]) => path === "/api/p2p/inbox"),
    ).toHaveLength(2);
  });

  it("P6.T2.G: Invites route disappears when auth becomes unauthenticated", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };
    mockAuthValue = {
      ...mockAuthValue,
      sessionRef: "token-abc",
      userId: "viewer-1",
      status: "authed",
    };

    const view = await render(<RootNavigator />);
    await waitFor(() => expect(view.getByTestId("home-open-invites")).toBeTruthy());
    fireEvent.press(view.getByTestId("home-open-invites"));
    await waitFor(() => expect(view.getByTestId("invites-screen")).toBeTruthy());

    mockAuthValue = {
      ...mockAuthValue,
      sessionRef: null,
      userId: null,
      status: "unauthed",
    };
    await act(async () => {
      view.rerender(<RootNavigator />);
    });

    await waitFor(() => expect(view.getByTestId("login-screen")).toBeTruthy());
    expect(view.queryByTestId("invites-screen")).toBeNull();
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
