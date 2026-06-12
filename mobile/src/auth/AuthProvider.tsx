import {
  createContext,
  type ReactNode,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import Constants from "expo-constants";
import { makeRedirectUri } from "expo-auth-session";
import * as WebBrowser from "expo-web-browser";
import { Linking } from "react-native";

import { createGatewayClient } from "../api/client";
import {
  clearSessionRef,
  isJwtLike,
  loadSessionRef,
  saveSessionRef,
  updateSessionRef,
} from "./session";
import { readRuntimeConfig } from "../config/env";

export type AuthStatus = "loading" | "authed" | "unauthed";

export type AuthContextValue = {
  sessionRef: string | null;
  status: AuthStatus;
  loginError: string | null;
  login: () => Promise<void>;
  logout: () => Promise<void>;
  onSessionRotation: (rotation: string | null) => Promise<void>;
};

const missingAuthProviderError =
  "useAuth must be used within an AuthProvider";

const createStubAsync = (): Promise<void> => Promise.resolve();

export const AuthContext =
  createContext<AuthContextValue | undefined>(undefined);

export function useAuth(): AuthContextValue {
  const authContext = useContext(AuthContext);

  if (authContext === undefined) {
    throw new Error(missingAuthProviderError);
  }

  return authContext;
}

type AuthProviderProps = {
  children: ReactNode;
};

function getGatewayClient() {
  const runtimeConfig = readRuntimeConfig();

  if (!runtimeConfig.ok) {
    return null;
  }

  return {
    client: createGatewayClient({
      gatewayBaseUrl: runtimeConfig.value.gatewayBaseUrl,
    }),
    gatewayBaseUrl: runtimeConfig.value.gatewayBaseUrl,
  };
}

function isE2EBootstrapEnabled(): boolean {
  if (!__DEV__) {
    return false;
  }

  if (process.env.EXPO_PUBLIC_E2E_ENABLED === "true") {
    return true;
  }

  const extra =
    Constants.expoConfig?.extra ??
    (Constants.manifest as { extra?: { e2eEnabled?: unknown } } | null)?.extra;

  return extra?.e2eEnabled === true || extra?.e2eEnabled === "true";
}

function getHandoffCodeFromCallbackUrl(url: string): string | null {
  try {
    const parsedUrl = new URL(url);

    if (parsedUrl.protocol !== "dubbridge:") {
      return null;
    }

    const normalizedPath = parsedUrl.pathname.replace(/^\/+/, "");
    const route =
      parsedUrl.host.length > 0
        ? `${parsedUrl.host}/${normalizedPath}`
        : normalizedPath;

    if (route !== "auth/callback") {
      return null;
    }

    const handoffCode = parsedUrl.searchParams.get("handoff_code")?.trim();

    return handoffCode ? handoffCode : null;
  } catch {
    return null;
  }
}

export function AuthProvider({
  children,
}: AuthProviderProps): ReactNode {
  const [sessionRef, setSessionRef] = useState<string | null>(null);
  const [status, setStatus] = useState<AuthStatus>("loading");
  const [loginError, setLoginError] = useState<string | null>(null);
  const statusRef = useRef<AuthStatus>("loading");
  const lastHandledBootstrapUrlRef = useRef<string | null>(null);
  const lastRedeemedHandoffCodeRef = useRef<string | null>(null);
  const pendingBootstrapUrlRef = useRef<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    async function hydrateSession(): Promise<void> {
      const storedSessionRef = await loadSessionRef();

      if (!isMounted) {
        return;
      }

      if (storedSessionRef === null) {
        setSessionRef(null);
        setStatus("unauthed");
        setLoginError(null);
        return;
      }

      if (isJwtLike(storedSessionRef)) {
        await clearSessionRef();

        if (!isMounted) {
          return;
        }

        setSessionRef(null);
        setStatus("unauthed");
        setLoginError(null);
        return;
      }

      setSessionRef(storedSessionRef);
      setStatus("authed");
      setLoginError(null);
    }

    void hydrateSession();

    return () => {
      isMounted = false;
    };
  }, []);

  async function redeemHandoffCode(
    handoffCode: string,
    gateway = getGatewayClient(),
  ): Promise<void> {
    if (lastRedeemedHandoffCodeRef.current === handoffCode) {
      return;
    }

    lastRedeemedHandoffCodeRef.current = handoffCode;
    setLoginError(null);

    if (gateway === null) {
      setStatus("unauthed");
      setSessionRef(null);
      setLoginError("missing_runtime_config");
      return;
    }

    const redeemResult = await gateway.client.post<{ session_ref: string }>(
      "/auth/mobile/session",
      null,
      { handoff_code: handoffCode },
    );

    if (!redeemResult.ok) {
      setStatus("unauthed");
      setSessionRef(null);
      setLoginError(
        redeemResult.error.kind === "session_expired"
          ? "session_expired"
          : "login_failed",
      );
      return;
    }

    const nextSessionRef = redeemResult.value.data.session_ref;

    if (!nextSessionRef || isJwtLike(nextSessionRef)) {
      setStatus("unauthed");
      setSessionRef(null);
      setLoginError("invalid_session_ref");
      return;
    }

    await saveSessionRef(nextSessionRef);
    setSessionRef(nextSessionRef);
    setStatus("authed");
    setLoginError(null);
  }

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  async function redeemPendingBootstrapUrl(): Promise<void> {
    if (!isE2EBootstrapEnabled() || statusRef.current !== "unauthed") {
      return;
    }

    const pendingUrl = pendingBootstrapUrlRef.current;

    if (!pendingUrl || lastHandledBootstrapUrlRef.current === pendingUrl) {
      return;
    }

    const handoffCode = getHandoffCodeFromCallbackUrl(pendingUrl);

    pendingBootstrapUrlRef.current = null;

    if (!handoffCode) {
      return;
    }

    lastHandledBootstrapUrlRef.current = pendingUrl;
    await redeemHandoffCode(handoffCode);
  }

  async function queueBootstrapUrl(url: string | null): Promise<void> {
    if (!url || statusRef.current === "authed") {
      return;
    }

    const handoffCode = getHandoffCodeFromCallbackUrl(url);

    if (
      !handoffCode ||
      lastHandledBootstrapUrlRef.current === url ||
      pendingBootstrapUrlRef.current === url
    ) {
      return;
    }

    pendingBootstrapUrlRef.current = url;
    await redeemPendingBootstrapUrl();
  }

  useEffect(() => {
    if (!isE2EBootstrapEnabled() || status !== "unauthed") {
      return;
    }

    void redeemPendingBootstrapUrl();
  }, [status]);

  useEffect(() => {
    if (!isE2EBootstrapEnabled()) {
      return;
    }

    let isMounted = true;

    void Linking.getInitialURL().then((url) => {
      if (!isMounted) {
        return;
      }

      void queueBootstrapUrl(url);
    });

    const subscription = Linking.addEventListener("url", ({ url }) => {
      void queueBootstrapUrl(url);
    });

    return () => {
      isMounted = false;
      subscription.remove();
    };
  }, []);

  async function logout(): Promise<void> {
    const previousSessionRef = sessionRef;

    setSessionRef(null);
    setStatus("unauthed");
    setLoginError(null);

    await clearSessionRef();

    if (previousSessionRef === null) {
      return;
    }

    const gateway = getGatewayClient();

    if (gateway === null) {
      return;
    }

    await gateway.client.post("/auth/logout", previousSessionRef, {});
  }

  async function login(): Promise<void> {
    const gateway = getGatewayClient();

    if (gateway === null) {
      setStatus("unauthed");
      setSessionRef(null);
      setLoginError("missing_runtime_config");
      return;
    }

    const redirectUri = makeRedirectUri({
      scheme: "dubbridge",
      path: "auth/callback",
    });
    const loginUrl = new URL("/auth/login", gateway.gatewayBaseUrl);
    loginUrl.searchParams.set("return_uri", redirectUri);

    const result = await WebBrowser.openAuthSessionAsync(
      loginUrl.toString(),
      redirectUri,
    );

    if (result.type !== "success") {
      return;
    }

    const handoffCode = new URL(result.url).searchParams.get("handoff_code");

    if (!handoffCode) {
      setStatus("unauthed");
      setSessionRef(null);
      setLoginError("missing_handoff_code");
      return;
    }

    await redeemHandoffCode(handoffCode, gateway);
  }

  async function onSessionRotation(rotation: string | null): Promise<void> {
    if (rotation === null || isJwtLike(rotation)) {
      return;
    }

    await updateSessionRef(rotation);
    setSessionRef(rotation);
    setStatus("authed");
    setLoginError(null);
  }

  const value: AuthContextValue = {
    sessionRef,
    status,
    loginError,
    login,
    logout,
    onSessionRotation,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}
