import { act, cleanup, render, waitFor } from "@testing-library/react-native";
import { Text } from "react-native";
import { makeRedirectUri } from "expo-auth-session";
import * as WebBrowser from "expo-web-browser";
import { Linking } from "react-native";

import { createGatewayClient } from "../src/api/client";
import {
  AuthProvider,
  useAuth,
  type AuthContextValue,
} from "../src/auth/AuthProvider";
import {
  clearSessionRef,
  isJwtLike,
  loadSessionRef,
  saveSessionRef,
  updateSessionRef,
} from "../src/auth/session";
import { readRuntimeConfig } from "../src/config/env";

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

let mockExtra: {
  e2eEnabled?: boolean | string;
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

jest.mock("../src/config/env", () => ({
  readRuntimeConfig: jest.fn(),
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

const OPAQUE_REF = "opaque-session-abc123";
const JWT_LIKE = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyIn0.SomeSignatureValue";
const ROTATED_REF = "rotated-session-xyz789";
const REDIRECT_URI = "dubbridge://auth/callback";
const HANDOFF_CODE = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNO12345";

const mockLoadSessionRef = loadSessionRef as jest.MockedFunction<typeof loadSessionRef>;
const mockSaveSessionRef = saveSessionRef as jest.MockedFunction<typeof saveSessionRef>;
const mockClearSessionRef = clearSessionRef as jest.MockedFunction<typeof clearSessionRef>;
const mockUpdateSessionRef = updateSessionRef as jest.MockedFunction<typeof updateSessionRef>;
const mockIsJwtLike = isJwtLike as jest.MockedFunction<typeof isJwtLike>;
const mockReadRuntimeConfig = readRuntimeConfig as jest.MockedFunction<typeof readRuntimeConfig>;
const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;
const mockMakeRedirectUri = makeRedirectUri as jest.MockedFunction<typeof makeRedirectUri>;
const mockOpenAuthSessionAsync =
  WebBrowser.openAuthSessionAsync as jest.MockedFunction<
    typeof WebBrowser.openAuthSessionAsync
  >;
const mockLinkingGetInitialURL = jest.spyOn(Linking, "getInitialURL");
const mockLinkingAddEventListener = jest.spyOn(Linking, "addEventListener");

type MockGatewayClient = {
  post: jest.Mock;
};

function createDeferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });

  return { promise, resolve, reject };
}

let latestAuthValue: AuthContextValue | null = null;
let mockGatewayClient: MockGatewayClient;
let linkingEventListener:
  | ((event: { url: string }) => void | Promise<void>)
  | null = null;
const originalE2EEnabled = process.env.EXPO_PUBLIC_E2E_ENABLED;

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
    linkingEventListener = null;
    mockLinkingGetInitialURL.mockReset();
    mockLinkingAddEventListener.mockReset();

    mockLoadSessionRef.mockResolvedValue(null);
    mockSaveSessionRef.mockResolvedValue(undefined);
    mockClearSessionRef.mockResolvedValue(undefined);
    mockUpdateSessionRef.mockResolvedValue(undefined);
    mockIsJwtLike.mockReturnValue(false);
    mockMakeRedirectUri.mockReturnValue(REDIRECT_URI);
    mockOpenAuthSessionAsync.mockResolvedValue({
      type: "cancel",
    } as Awaited<ReturnType<typeof WebBrowser.openAuthSessionAsync>>);
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
        value: { data: {}, sessionRotation: null },
      }),
    };

    mockCreateGatewayClient.mockReturnValue(
      mockGatewayClient as unknown as ReturnType<typeof createGatewayClient>,
    );
    mockLinkingGetInitialURL.mockResolvedValue(null);
    mockLinkingAddEventListener.mockImplementation((_event, listener) => {
      linkingEventListener = listener as typeof linkingEventListener;
      return {
        remove: jest.fn(() => {
          linkingEventListener = null;
        }),
      } as unknown as ReturnType<typeof Linking.addEventListener>;
    });
    mockExtra = {
      e2eEnabled: false,
    };
    delete process.env.EXPO_PUBLIC_E2E_ENABLED;
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

  describe("T3b-i-β HP-1: valid stored opaque session boots into authed state", () => {
    it("loads a stored ref and exposes authed state", async () => {
      mockLoadSessionRef.mockResolvedValueOnce(OPAQUE_REF);
      mockIsJwtLike.mockReturnValueOnce(false);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:authed")).toBeTruthy();
      });

      expect(view.getByText(`sessionRef:${OPAQUE_REF}`)).toBeTruthy();
      expect(mockLoadSessionRef).toHaveBeenCalledTimes(1);
      expect(mockIsJwtLike).toHaveBeenCalledWith(OPAQUE_REF);
      expect(mockClearSessionRef).not.toHaveBeenCalled();
    });
  });

  describe("T3b-i-β EC-1: absent stored session boots into unauthed state", () => {
    it("becomes unauthed when no session ref is stored", async () => {
      mockLoadSessionRef.mockResolvedValueOnce(null);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      expect(view.getByText("sessionRef:null")).toBeTruthy();
      expect(mockIsJwtLike).not.toHaveBeenCalled();
      expect(mockClearSessionRef).not.toHaveBeenCalled();
    });
  });

  describe("T3b-i-β EC-2: JWT-like stored value is cleared and rejected", () => {
    it("clears the stored value and becomes unauthed", async () => {
      mockLoadSessionRef.mockResolvedValueOnce(JWT_LIKE);
      mockIsJwtLike.mockReturnValueOnce(true);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      expect(view.getByText("sessionRef:null")).toBeTruthy();
      expect(mockIsJwtLike).toHaveBeenCalledWith(JWT_LIKE);
      expect(mockClearSessionRef).toHaveBeenCalledTimes(1);
    });
  });

  describe("T3b-i-β HP-2: onSessionRotation persists and updates in-memory state", () => {
    it("stores the rotated ref and updates the current sessionRef", async () => {
      mockLoadSessionRef.mockResolvedValueOnce(OPAQUE_REF);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:authed")).toBeTruthy();
      });

      await act(async () => {
        await latestAuthValue?.onSessionRotation(ROTATED_REF);
      });

      expect(mockUpdateSessionRef).toHaveBeenCalledWith(ROTATED_REF);
      expect(view.getByText(`sessionRef:${ROTATED_REF}`)).toBeTruthy();
      expect(view.getByText("status:authed")).toBeTruthy();
    });
  });

  describe("T3b-i-β EC-3: onSessionRotation(null) is a no-op", () => {
    it("does not persist or alter the current session", async () => {
      mockLoadSessionRef.mockResolvedValueOnce(OPAQUE_REF);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:authed")).toBeTruthy();
      });

      await act(async () => {
        await latestAuthValue?.onSessionRotation(null);
      });

      expect(mockUpdateSessionRef).not.toHaveBeenCalled();
      expect(view.getByText(`sessionRef:${OPAQUE_REF}`)).toBeTruthy();
    });
  });

  describe("T3b-i-β HP-3: logout clears local auth state and calls gateway logout", () => {
    it("clears the session ref and makes a best-effort gateway logout call", async () => {
      mockLoadSessionRef.mockResolvedValueOnce(OPAQUE_REF);

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

      expect(mockClearSessionRef).toHaveBeenCalledTimes(1);
      expect(mockReadRuntimeConfig).toHaveBeenCalled();
      expect(mockCreateGatewayClient).toHaveBeenCalledWith({
        gatewayBaseUrl: "http://127.0.0.1:4000",
      });
      expect(mockGatewayClient.post).toHaveBeenCalledWith("/auth/logout", OPAQUE_REF, {});
      expect(view.getByText("status:unauthed")).toBeTruthy();
      expect(view.getByText("sessionRef:null")).toBeTruthy();
    });
  });

  describe("T3b-i-β EC-4: gateway logout failure still leaves local auth state cleared", () => {
    it("fails safe to unauthed state even when the gateway logout result is not ok", async () => {
      mockLoadSessionRef.mockResolvedValueOnce(OPAQUE_REF);
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: false,
        error: { kind: "network", message: "timeout" },
      });

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

      expect(mockClearSessionRef).toHaveBeenCalledTimes(1);
      expect(mockGatewayClient.post).toHaveBeenCalledWith("/auth/logout", OPAQUE_REF, {});
      expect(view.getByText("status:unauthed")).toBeTruthy();
      expect(view.getByText("sessionRef:null")).toBeTruthy();
    });
  });

  describe("T3b-ii HP-1 + HP-2: login redeems a handoff code and persists an opaque session", () => {
    it("opens the system browser, redeems the code, and becomes authed", async () => {
      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      mockOpenAuthSessionAsync.mockResolvedValueOnce({
        type: "success",
        url: `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
      } as Awaited<ReturnType<typeof WebBrowser.openAuthSessionAsync>>);
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: true,
        value: {
          data: {
            session_ref: OPAQUE_REF,
          },
          sessionRotation: null,
        },
      });
      mockIsJwtLike.mockReturnValueOnce(false);

      await act(async () => {
        await latestAuthValue?.login();
      });

      expect(mockMakeRedirectUri).toHaveBeenCalledWith({
        scheme: "dubbridge",
        path: "auth/callback",
      });
      expect(mockOpenAuthSessionAsync).toHaveBeenCalledWith(
        `http://127.0.0.1:4000/auth/login?return_uri=${encodeURIComponent(
          REDIRECT_URI,
        )}`,
        REDIRECT_URI,
      );
      expect(mockGatewayClient.post).toHaveBeenCalledWith(
        "/auth/mobile/session",
        null,
        { handoff_code: HANDOFF_CODE },
      );
      expect(mockSaveSessionRef).toHaveBeenCalledWith(OPAQUE_REF);
      expect(mockSaveSessionRef.mock.calls[0]?.[0]).not.toContain(".");
      expect(view.getByText("status:authed")).toBeTruthy();
      expect(view.getByText(`sessionRef:${OPAQUE_REF}`)).toBeTruthy();
      expect(view.getByText("loginError:null")).toBeTruthy();
    });
  });

  describe("T3b-ii EC-1: browser cancel leaves auth state unchanged", () => {
    it("stays unauthed when the auth session result is not success", async () => {
      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      await act(async () => {
        await latestAuthValue?.login();
      });

      expect(mockOpenAuthSessionAsync).toHaveBeenCalledTimes(1);
      expect(mockGatewayClient.post).not.toHaveBeenCalledWith(
        "/auth/mobile/session",
        null,
        expect.anything(),
      );
      expect(mockSaveSessionRef).not.toHaveBeenCalled();
      expect(view.getByText("status:unauthed")).toBeTruthy();
      expect(view.getByText("loginError:null")).toBeTruthy();
    });
  });

  describe("T3b-ii EC-2: missing handoff code surfaces a clear login error", () => {
    it("sets loginError to missing_handoff_code and does not authenticate", async () => {
      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      mockOpenAuthSessionAsync.mockResolvedValueOnce({
        type: "success",
        url: REDIRECT_URI,
      } as Awaited<ReturnType<typeof WebBrowser.openAuthSessionAsync>>);

      await act(async () => {
        await latestAuthValue?.login();
      });

      expect(mockGatewayClient.post).not.toHaveBeenCalledWith(
        "/auth/mobile/session",
        null,
        expect.anything(),
      );
      expect(mockSaveSessionRef).not.toHaveBeenCalled();
      expect(view.getByText("status:unauthed")).toBeTruthy();
      expect(view.getByText("loginError:missing_handoff_code")).toBeTruthy();
    });
  });

  describe("T3b-ii EC-3: 401 from handoff redemption maps to session_expired", () => {
    it("does not authenticate and stores a session_expired loginError", async () => {
      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      mockOpenAuthSessionAsync.mockResolvedValueOnce({
        type: "success",
        url: `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
      } as Awaited<ReturnType<typeof WebBrowser.openAuthSessionAsync>>);
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: false,
        error: { kind: "session_expired" },
      });

      await act(async () => {
        await latestAuthValue?.login();
      });

      expect(mockSaveSessionRef).not.toHaveBeenCalled();
      expect(view.getByText("status:unauthed")).toBeTruthy();
      expect(view.getByText("loginError:session_expired")).toBeTruthy();
    });
  });

  describe("T3b-ii EC-4: JWT-like session_ref from gateway is rejected", () => {
    it("does not persist a JWT-looking session ref", async () => {
      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      mockOpenAuthSessionAsync.mockResolvedValueOnce({
        type: "success",
        url: `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
      } as Awaited<ReturnType<typeof WebBrowser.openAuthSessionAsync>>);
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: true,
        value: {
          data: {
            session_ref: JWT_LIKE,
          },
          sessionRotation: null,
        },
      });
      mockIsJwtLike.mockReturnValueOnce(true);

      await act(async () => {
        await latestAuthValue?.login();
      });

      expect(mockSaveSessionRef).not.toHaveBeenCalled();
      expect(view.getByText("status:unauthed")).toBeTruthy();
      expect(view.getByText("loginError:invalid_session_ref")).toBeTruthy();
    });
  });

  describe("V5 HP-1: dev-gated bootstrap redeems an inbound handoff deep link", () => {
    it("hydrates from an initial callback URL and becomes authed without opening the browser", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";
      mockLinkingGetInitialURL.mockResolvedValueOnce(
        `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
      );
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: true,
        value: {
          data: {
            session_ref: OPAQUE_REF,
          },
          sessionRotation: null,
        },
      });
      mockIsJwtLike.mockReturnValueOnce(false);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:authed")).toBeTruthy();
      });

      expect(mockOpenAuthSessionAsync).not.toHaveBeenCalled();
      expect(mockGatewayClient.post).toHaveBeenCalledWith(
        "/auth/mobile/session",
        null,
        { handoff_code: HANDOFF_CODE },
      );
      expect(mockSaveSessionRef).toHaveBeenCalledWith(OPAQUE_REF);
      expect(view.getByText(`sessionRef:${OPAQUE_REF}`)).toBeTruthy();
    });
  });

  describe("V5 EC-1: bootstrap listener stays inert when the flag is off", () => {
    it("ignores inbound deep links when EXPO_PUBLIC_E2E_ENABLED is not true", async () => {
      mockLinkingGetInitialURL.mockResolvedValueOnce(
        `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
      );

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      expect(mockGatewayClient.post).not.toHaveBeenCalledWith(
        "/auth/mobile/session",
        null,
        { handoff_code: HANDOFF_CODE },
      );
      expect(mockSaveSessionRef).not.toHaveBeenCalled();
    });
  });

  describe("V5 EC-2: JWT-like session_ref from bootstrap is rejected", () => {
    it("does not persist a JWT-looking session ref from an inbound deep link", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";
      mockLinkingGetInitialURL.mockResolvedValueOnce(
        `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
      );
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: true,
        value: {
          data: {
            session_ref: JWT_LIKE,
          },
          sessionRotation: null,
        },
      });
      mockIsJwtLike.mockReturnValueOnce(true);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      expect(mockSaveSessionRef).not.toHaveBeenCalled();
      expect(view.getByText("loginError:invalid_session_ref")).toBeTruthy();
    });
  });

  describe("V5 EC-3: 401 bootstrap redemption stays unauthenticated cleanly", () => {
    it("maps a failed inbound redemption to session_expired without crashing", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";
      mockLinkingGetInitialURL.mockResolvedValueOnce(
        `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
      );
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

      expect(mockSaveSessionRef).not.toHaveBeenCalled();
      expect(view.getByText("loginError:session_expired")).toBeTruthy();
    });
  });

  describe("V5 EC-4: duplicate url events are redeemed only once", () => {
    it("deduplicates the same callback URL across initial and event delivery", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";
      const bootstrapUrl = `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`;
      mockLinkingGetInitialURL.mockResolvedValueOnce(bootstrapUrl);
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: true,
        value: {
          data: {
            session_ref: OPAQUE_REF,
          },
          sessionRotation: null,
        },
      });
      mockIsJwtLike.mockReturnValueOnce(false);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:authed")).toBeTruthy();
      });

      await act(async () => {
        await linkingEventListener?.({ url: bootstrapUrl });
      });

      expect(mockGatewayClient.post).toHaveBeenCalledTimes(1);
    });
  });

  describe("V5 HP-2: live url events redeem after mount", () => {
    it("redeems a callback URL delivered through the runtime listener", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: true,
        value: {
          data: {
            session_ref: OPAQUE_REF,
          },
          sessionRotation: null,
        },
      });
      mockIsJwtLike.mockReturnValueOnce(false);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await waitFor(() => {
        expect(view.getByText("status:unauthed")).toBeTruthy();
      });

      await act(async () => {
        await linkingEventListener?.({
          url: `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`,
        });
      });

      await waitFor(() => {
        expect(view.getByText("status:authed")).toBeTruthy();
      });

      expect(mockGatewayClient.post).toHaveBeenCalledWith(
        "/auth/mobile/session",
        null,
        { handoff_code: HANDOFF_CODE },
      );
      expect(mockSaveSessionRef).toHaveBeenCalledWith(OPAQUE_REF);
    });
  });

  describe("V6 EC-1: bootstrap deep links received during loading are replayed after hydration", () => {
    it("redeems a queued callback URL once status becomes unauthed", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";
      const bootstrapUrl = `${REDIRECT_URI}?handoff_code=${HANDOFF_CODE}`;
      const sessionDeferred = createDeferred<string | null>();
      mockLoadSessionRef.mockImplementationOnce(() => sessionDeferred.promise);
      mockGatewayClient.post.mockResolvedValueOnce({
        ok: true,
        value: {
          data: {
            session_ref: OPAQUE_REF,
          },
          sessionRotation: null,
        },
      });
      mockIsJwtLike.mockReturnValueOnce(false);

      const view = await render(
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>,
      );

      await act(async () => {
        await linkingEventListener?.({ url: bootstrapUrl });
      });

      expect(mockGatewayClient.post).not.toHaveBeenCalled();

      await act(async () => {
        sessionDeferred.resolve(null);
        await sessionDeferred.promise;
      });

      await waitFor(() => {
        expect(view.getByText("status:authed")).toBeTruthy();
      });

      expect(mockGatewayClient.post).toHaveBeenCalledWith(
        "/auth/mobile/session",
        null,
        { handoff_code: HANDOFF_CODE },
      );
      expect(mockSaveSessionRef).toHaveBeenCalledWith(OPAQUE_REF);
    });
  });
});
