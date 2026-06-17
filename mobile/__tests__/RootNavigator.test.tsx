import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import type { AuthContextValue } from "../src/auth/AuthProvider";
import { RootNavigator } from "../src/navigation/RootNavigator";

let mockExtra: {
  dubbridgeEnv?: unknown;
  gatewayBaseUrl?: unknown;
} = {};

let mockAuthValue: AuthContextValue;
let mockNotificationResponseListener:
  | ((response: {
      notification: { request: { content: { data: Record<string, unknown> } } };
    }) => void)
  | null = null;

const mockDeepLinkTask = {
  id: "task-push-001",
  org_id: "org-001",
  project_id: "proj-001",
  asset_id: "asset-001",
  target_language_id: "lang-001",
  assignee_subject_id: "reviewer-001",
  state: "pending" as const,
  created_at: "2026-06-13T00:00:00Z",
  updated_at: "2026-06-13T00:00:00Z",
  assigned_at: "2026-06-13T00:00:00Z",
};

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

jest.mock("../src/auth/AuthProvider", () => ({
  AuthProvider: ({ children }: { children: React.ReactNode }) => children,
  useAuth: () => mockAuthValue,
}));

jest.mock("../src/push/registerPush", () => ({
  registerPush: jest.fn().mockResolvedValue(undefined),
}));

jest.mock("expo-notifications", () => ({
  addNotificationResponseReceivedListener: jest.fn((listener) => {
    mockNotificationResponseListener = listener;
    return { remove: jest.fn() };
  }),
}));

jest.mock("../src/screens/ReviewInboxScreen", () => ({
  ReviewInboxScreen: ({
    initialTaskId,
    onOpenTask,
  }: {
    initialTaskId?: string | null;
    onOpenTask: (task: typeof mockDeepLinkTask) => void;
  }) => {
    const React = require("react");
    const { Text } = require("react-native");
    React.useEffect(() => {
      if (initialTaskId) {
        onOpenTask({
          id: initialTaskId,
          org_id: "org-001",
          project_id: "proj-001",
          asset_id: "asset-001",
          target_language_id: "lang-001",
          assignee_subject_id: "reviewer-001",
          state: "pending",
          created_at: "2026-06-13T00:00:00Z",
          updated_at: "2026-06-13T00:00:00Z",
          assigned_at: "2026-06-13T00:00:00Z",
        });
      }
    }, [initialTaskId, onOpenTask]);
    return <Text testID="mock-review-inbox">{initialTaskId ?? "review-inbox"}</Text>;
  },
}));

jest.mock("../src/screens/ReviewDetailScreen", () => ({
  ReviewDetailScreen: ({ task }: { task: typeof mockDeepLinkTask }) => {
    const { Text } = require("react-native");
    return <Text testID="review-detail-screen">{task.id}</Text>;
  },
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
    mockNotificationResponseListener = null;
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

    expect(view.getByText("DubBridge")).toBeTruthy();
    expect(view.getByText("Sign in")).toBeTruthy();
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

    expect(view.getByText("DubBridge")).toBeTruthy();
    expect(view.getByText("Sign in")).toBeTruthy();
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

    expect(view.getByText("Your workspace")).toBeTruthy();
    expect(view.getByText("local")).toBeTruthy();
    expect(view.getByText("http://127.0.0.1:4000")).toBeTruthy();
    expect(view.getByText("Browse assets")).toBeTruthy();
    expect(view.getByText("Review inbox")).toBeTruthy();
    expect(view.getByText("Organizations and projects")).toBeTruthy();
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

    fireEvent.press(view.getByText("Sign in"));

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

  it("deep-links a push review_task notification to ReviewDetail when authenticated", async () => {
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

    await waitFor(() =>
      expect(view.getByTestId("home-screen")).toBeTruthy(),
    );

    await act(async () => {
      mockNotificationResponseListener?.({
        notification: {
          request: {
            content: {
              data: {
                ref_entity_type: "review_task",
                ref_entity_id: mockDeepLinkTask.id,
              },
            },
          },
        },
      });
    });

    await waitFor(() =>
      expect(view.getByTestId("review-detail-screen")).toBeTruthy(),
    );
    expect(view.getByText(mockDeepLinkTask.id)).toBeTruthy();
  });

  it("keeps the login screen visible on push tap while logged out", async () => {
    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };

    const view = await render(<RootNavigator />);

    await act(async () => {
      mockNotificationResponseListener?.({
        notification: {
          request: {
            content: {
              data: {
                ref_entity_type: "review_task",
                ref_entity_id: mockDeepLinkTask.id,
              },
            },
          },
        },
      });
    });

    expect(view.getByTestId("login-screen")).toBeTruthy();
    expect(view.queryByTestId("review-detail-screen")).toBeNull();
  });
});
