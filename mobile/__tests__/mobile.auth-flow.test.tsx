import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";
import { makeRedirectUri } from "expo-auth-session";
import * as WebBrowser from "expo-web-browser";
import { Linking } from "react-native";

import { createGatewayClient } from "../src/api/client";
import { RootNavigator } from "../src/navigation/RootNavigator";
import {
  clearSessionRef,
  isJwtLike,
  loadSessionRef,
  saveSessionRef,
  updateSessionRef,
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
  loadSessionRef: jest.fn(),
  saveSessionRef: jest.fn(),
  clearSessionRef: jest.fn(),
  updateSessionRef: jest.fn(),
  isJwtLike: jest.fn(),
}));

jest.mock("../src/api/client", () => ({
  createGatewayClient: jest.fn(),
}));

jest.mock("expo-auth-session", () => ({
  makeRedirectUri: jest.fn(),
}));

jest.mock("expo-web-browser", () => ({
  openAuthSessionAsync: jest.fn(),
}));

const mockLoadSessionRef = loadSessionRef as jest.MockedFunction<typeof loadSessionRef>;
const mockSaveSessionRef = saveSessionRef as jest.MockedFunction<typeof saveSessionRef>;
const mockClearSessionRef = clearSessionRef as jest.MockedFunction<typeof clearSessionRef>;
const mockUpdateSessionRef =
  updateSessionRef as jest.MockedFunction<typeof updateSessionRef>;
const mockIsJwtLike = isJwtLike as jest.MockedFunction<typeof isJwtLike>;
const mockCreateGatewayClient =
  createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;
const mockMakeRedirectUri = makeRedirectUri as jest.MockedFunction<typeof makeRedirectUri>;
const mockOpenAuthSessionAsync =
  WebBrowser.openAuthSessionAsync as jest.MockedFunction<
    typeof WebBrowser.openAuthSessionAsync
  >;
const mockLinkingGetInitialURL = jest.spyOn(Linking, "getInitialURL");
const mockLinkingAddEventListener = jest.spyOn(Linking, "addEventListener");

const REDIRECT_URI = "dubbridge://auth/callback";
const OPAQUE_REF = "opaque-session-abc123";
const HANDOFF_CODE = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNO12345";
const JWT_LIKE =
  "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyIn0.SomeSignatureValue";
const originalE2EEnabled = process.env.EXPO_PUBLIC_E2E_ENABLED;

describe("mobile auth flow integration", () => {
  beforeEach(() => {
    cleanup();
    jest.clearAllMocks();

    mockExtra = {
      dubbridgeEnv: "local",
      gatewayBaseUrl: "http://127.0.0.1:4000",
    };

    mockLoadSessionRef.mockResolvedValue(null);
    mockSaveSessionRef.mockResolvedValue(undefined);
    mockClearSessionRef.mockResolvedValue(undefined);
    mockUpdateSessionRef.mockResolvedValue(undefined);
    mockIsJwtLike.mockImplementation((value: string) => value === JWT_LIKE);
    mockMakeRedirectUri.mockReturnValue(REDIRECT_URI);
    mockOpenAuthSessionAsync.mockResolvedValue({
      type: "success",
      url: `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
    } as Awaited<ReturnType<typeof WebBrowser.openAuthSessionAsync>>);
    mockLinkingGetInitialURL.mockResolvedValue(null);
    mockLinkingAddEventListener.mockImplementation(
      () =>
        ({
          remove: jest.fn(),
        }) as unknown as ReturnType<typeof Linking.addEventListener>,
    );

    const mockClient = {
      get: jest
        .fn()
        .mockResolvedValueOnce({
          ok: true,
          value: {
            data: [
              {
                id: "asset-123",
                title: "Test Video",
                uploader_id: "user-123",
                status: "finalized",
                created_at: "2026-06-07T10:00:00Z",
                updated_at: "2026-06-07T10:05:00Z",
              },
            ],
            sessionRotation: null,
          },
        })
        .mockResolvedValueOnce({
          ok: true,
          value: {
            data: {
              id: "asset-123",
              title: "Test Video",
              uploader_id: "user-123",
              status: "finalized",
              created_at: "2026-06-07T10:00:00Z",
              updated_at: "2026-06-07T10:05:00Z",
            },
            sessionRotation: null,
          },
        }),
      post: jest
        .fn()
        .mockResolvedValueOnce({
          ok: true,
          value: {
            data: {
              session_ref: OPAQUE_REF,
            },
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
    };

    mockCreateGatewayClient.mockReturnValue(
      mockClient as unknown as ReturnType<typeof createGatewayClient>,
    );
  });

  afterEach(() => {
    cleanup();
    mockExtra = {};
    if (originalE2EEnabled === undefined) {
      delete process.env.EXPO_PUBLIC_E2E_ENABLED;
    } else {
      process.env.EXPO_PUBLIC_E2E_ENABLED = originalE2EEnabled;
    }
  });

  describe("T5 HP-1: full mobile login to asset detail flow stays green against the stub", () => {
    it("signs in through the gateway, opens the asset list, and renders asset detail", async () => {
      const view = await render(<RootNavigator />);

      await waitFor(() => {
        expect(view.getByText("Sign in with session gateway")).toBeTruthy();
      });

      await act(async () => {
        fireEvent.press(view.getByText("Sign in with session gateway"));
      });

      await waitFor(() => {
        expect(view.getByText("Mobile home")).toBeTruthy();
      });

      expect(mockSaveSessionRef).toHaveBeenCalledWith(OPAQUE_REF);
      expect(mockSaveSessionRef).not.toHaveBeenCalledWith(JWT_LIKE);
      expect(view.queryByText(OPAQUE_REF)).toBeNull();
      expect(view.queryByText(JWT_LIKE)).toBeNull();

      await act(async () => {
        fireEvent.press(view.getByText("Browse assets"));
      });

      await waitFor(() => {
        expect(view.getByText("Asset list")).toBeTruthy();
        expect(view.getByText("Test Video")).toBeTruthy();
      });

      await act(async () => {
        fireEvent.press(view.getByText("Test Video"));
      });

      await waitFor(() => {
        expect(view.getByText("Asset detail")).toBeTruthy();
        expect(view.getByText("Compliance and consent")).toBeTruthy();
      });
    });
  });

  describe("V5 HP-1: root navigator enters the authed tree from an inbound handoff deep link", () => {
    it("boots directly to the home screen when E2E bootstrap is enabled", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";
      mockLinkingGetInitialURL.mockResolvedValueOnce(
        `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
      );

      const view = await render(<RootNavigator />);

      await waitFor(() => {
        expect(view.getByText("Mobile home")).toBeTruthy();
      });

      expect(mockSaveSessionRef).toHaveBeenCalledWith(OPAQUE_REF);
      expect(view.getByTestId("home-screen")).toBeTruthy();
    });
  });
});
