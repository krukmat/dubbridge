import { act, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import { RootNavigator } from "../src/navigation/RootNavigator";
import {
  clearAuthSession,
  loadAuthSession,
  saveAuthSession,
  type AuthSession,
} from "../src/auth/session";

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

let mockExtra: {
  dubbridgeEnv?: string | null;
  gatewayBaseUrl?: string | null;
} = {};

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

jest.mock("../src/auth/session", () => ({
  loadAuthSession: jest.fn(),
  saveAuthSession: jest.fn(),
  clearAuthSession: jest.fn(),
}));

jest.mock("../src/api/client", () => ({
  createGatewayClient: jest.fn(),
}));

jest.mock("../src/push/registerPush", () => ({
  registerPush: jest.fn().mockResolvedValue(undefined),
}));

jest.mock("expo-notifications", () => ({
  addNotificationResponseReceivedListener: jest.fn(() => ({
    remove: jest.fn(),
  })),
}));

jest.mock("../src/api/notifications", () => ({
  listNotifications: jest.fn().mockResolvedValue({
    ok: true,
    value: { data: { notifications: [] }, sessionRotation: null },
  }),
  markNotificationsRead: jest.fn().mockResolvedValue({ ok: true, value: { sessionRotation: null } }),
  registerPushToken: jest.fn().mockResolvedValue({ ok: true, value: { sessionRotation: null } }),
}));

const mockLoadAuthSession =
  loadAuthSession as jest.MockedFunction<typeof loadAuthSession>;
const mockSaveAuthSession =
  saveAuthSession as jest.MockedFunction<typeof saveAuthSession>;
const mockClearAuthSession =
  clearAuthSession as jest.MockedFunction<typeof clearAuthSession>;
const mockCreateGatewayClient =
  createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;

const LOGIN_SESSION: AuthSession = {
  token: "token-abc",
  userId: "user-123",
  workspaceId: "workspace-456",
};

const ASSET_SUMMARY = {
  id: "asset-123",
  title: "Test Video",
  uploader_id: "user-123",
  status: "finalized",
  created_at: "2026-06-07T10:00:00Z",
  updated_at: "2026-06-07T10:05:00Z",
};

describe("mobile auth flow integration", () => {
  beforeEach(() => {
    jest.clearAllMocks();

    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };

    mockLoadAuthSession.mockResolvedValue(null);
    mockSaveAuthSession.mockResolvedValue(undefined);
    mockClearAuthSession.mockResolvedValue(undefined);

    const mockClient = {
      get: jest
        .fn()
        // HomeScreen: GET /api/assets (dashboard aggregate)
        .mockResolvedValueOnce({
          ok: true,
          value: { data: [ASSET_SUMMARY], sessionRotation: null },
        })
        // AssetListScreen: GET /api/assets
        .mockResolvedValueOnce({
          ok: true,
          value: { data: [ASSET_SUMMARY], sessionRotation: null },
        })
        // AssetDetailScreen: GET /api/assets/{id}
        .mockResolvedValueOnce({
          ok: true,
          value: { data: ASSET_SUMMARY, sessionRotation: null },
        }),
      post: jest
        .fn()
        .mockResolvedValueOnce({
          ok: true,
          value: {
            data: LOGIN_SESSION,
            sessionRotation: null,
          },
        })
        .mockResolvedValue({
          ok: true,
          value: {
            data: {},
            sessionRotation: null,
          },
        }),
      postMultipart: jest.fn(),
    };

    mockCreateGatewayClient.mockReturnValue(
      mockClient as unknown as ReturnType<typeof createGatewayClient>,
    );
  });

  it("HP-1 + HP-2 + EC-1: bearer login reaches home and asset detail without any browser handoff", async () => {
    const view = await render(<RootNavigator />);

    await waitFor(() => {
      expect(view.getByTestId("login-email-input")).toBeTruthy();
    });

    await act(async () => {
      fireEvent.changeText(view.getByTestId("login-email-input"), "user@example.com");
      fireEvent.changeText(view.getByTestId("login-password-input"), "password-123456");
    });

    await waitFor(() => {
      expect(view.getByTestId("login-submit-button").props.accessibilityState.disabled).toBe(false);
    });

    await act(async () => {
      fireEvent.press(view.getByTestId("login-submit-button"));
    });

    await waitFor(() => {
      expect(view.getByTestId("home-screen")).toBeTruthy();
    });

    expect(mockSaveAuthSession).toHaveBeenCalledWith(LOGIN_SESSION);

    const mockClient = mockCreateGatewayClient.mock.results[0]?.value as {
      post: jest.Mock;
    };
    expect(mockClient.post).toHaveBeenCalledWith(
      "/auth/login",
      null,
      {
        email: "user@example.com",
        password: "password-123456",
      },
    );

    await act(async () => {
      fireEvent.press(view.getByText("Browse assets"));
    });

    await waitFor(() => {
      expect(view.getByTestId("asset-list-screen")).toBeTruthy();
      expect(view.getByText("Test Video")).toBeTruthy();
    });

    await act(async () => {
      fireEvent.press(view.getByText("Test Video"));
    });

    await waitFor(() => {
      expect(view.getByTestId("asset-detail-screen")).toBeTruthy();
      expect(view.getByText("Compliance and consent")).toBeTruthy();
    });
  });
});
