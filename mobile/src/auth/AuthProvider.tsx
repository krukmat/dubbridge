import {
  createContext,
  type ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";

import { createGatewayClient } from "../api/client";
import {
  clearAuthSession,
  loadAuthSession,
  saveAuthSession,
  type AuthSession,
} from "./session";
import { readRuntimeConfig } from "../config/env";

export type AuthStatus = "loading" | "authed" | "unauthed";

export type AuthContextValue = {
  sessionRef: string | null;
  status: AuthStatus;
  loginError: string | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  onSessionRotation: (rotation: string | null) => Promise<void>;
};

const missingAuthProviderError =
  "useAuth must be used within an AuthProvider";

const createStubAsync = (_rotation: string | null): Promise<void> =>
  Promise.resolve();

type GatewayClientBundle = {
  client: ReturnType<typeof createGatewayClient>;
};

type AuthSuccessPayload = {
  token: string;
  userId: string;
  workspaceId: string;
};

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

function getGatewayClient(): GatewayClientBundle | null {
  const runtimeConfig = readRuntimeConfig();

  if (!runtimeConfig.ok) {
    return null;
  }

  return {
    client: createGatewayClient({
      gatewayBaseUrl: runtimeConfig.value.gatewayBaseUrl,
    }),
  };
}

function isAuthSuccessPayload(value: unknown): value is AuthSuccessPayload {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const candidate = value as Record<string, unknown>;

  return (
    typeof candidate.token === "string" &&
    candidate.token.trim().length > 0 &&
    typeof candidate.userId === "string" &&
    candidate.userId.trim().length > 0 &&
    typeof candidate.workspaceId === "string" &&
    candidate.workspaceId.trim().length > 0
  );
}

function toAuthSession(payload: AuthSuccessPayload): AuthSession {
  return {
    token: payload.token.trim(),
    userId: payload.userId.trim(),
    workspaceId: payload.workspaceId.trim(),
  };
}

export function AuthProvider({
  children,
}: AuthProviderProps): ReactNode {
  const [session, setSession] = useState<AuthSession | null>(null);
  const [status, setStatus] = useState<AuthStatus>("loading");
  const [loginError, setLoginError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    async function hydrateSession(): Promise<void> {
      try {
        const storedSession = await loadAuthSession();

        if (!isMounted) {
          return;
        }

        if (storedSession === null) {
          setSession(null);
          setStatus("unauthed");
          setLoginError(null);
          return;
        }

        setSession(storedSession);
        setStatus("authed");
        setLoginError(null);
      } catch {
        await clearAuthSession();

        if (!isMounted) {
          return;
        }

        setSession(null);
        setStatus("unauthed");
        setLoginError(null);
      }
    }

    void hydrateSession();

    return () => {
      isMounted = false;
    };
  }, []);

  async function logout(): Promise<void> {
    setSession(null);
    setStatus("unauthed");
    setLoginError(null);
    await clearAuthSession();
  }

  async function login(email: string, password: string): Promise<void> {
    const gateway = getGatewayClient();

    if (gateway === null) {
      setSession(null);
      setStatus("unauthed");
      setLoginError("missing_runtime_config");
      return;
    }

    const loginResult = await gateway.client.post<AuthSuccessPayload>(
      "/auth/login",
      null,
      {
        email: email.trim(),
        password,
      },
    );

    if (!loginResult.ok) {
      await clearAuthSession();
      setSession(null);
      setStatus("unauthed");
      setLoginError(
        loginResult.error.kind === "network"
          ? "network_error"
          : "login_failed",
      );
      return;
    }

    if (!isAuthSuccessPayload(loginResult.value.data)) {
      await clearAuthSession();
      setSession(null);
      setStatus("unauthed");
      setLoginError("login_failed");
      return;
    }

    const nextSession = toAuthSession(loginResult.value.data);
    await saveAuthSession(nextSession);
    setSession(nextSession);
    setStatus("authed");
    setLoginError(null);
  }

  const value: AuthContextValue = {
    sessionRef: session?.token ?? null,
    status,
    loginError,
    login,
    logout,
    onSessionRotation: createStubAsync,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}
