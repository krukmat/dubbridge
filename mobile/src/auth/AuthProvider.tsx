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

type AuthStateControls = {
  setSession: (session: AuthSession | null) => void;
  setStatus: (status: AuthStatus) => void;
  setLoginError: (error: string | null) => void;
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

function resetAuthState(controls: AuthStateControls) {
  controls.setSession(null);
  controls.setStatus("unauthed");
  controls.setLoginError(null);
}

function acceptStoredSession(
  storedSession: AuthSession | null,
  controls: AuthStateControls,
) {
  if (storedSession === null) {
    resetAuthState(controls);
    return;
  }

  controls.setSession(storedSession);
  controls.setStatus("authed");
  controls.setLoginError(null);
}

async function hydrateStoredSession(
  controls: AuthStateControls,
  isMounted: () => boolean,
) {
  try {
    const storedSession = await loadAuthSession();
    if (!isMounted()) return;
    acceptStoredSession(storedSession, controls);
  } catch {
    await clearAuthSession();
    if (!isMounted()) return;
    resetAuthState(controls);
  }
}

function useHydratedAuthState(
  setSession: AuthStateControls["setSession"],
  setStatus: AuthStateControls["setStatus"],
  setLoginError: AuthStateControls["setLoginError"],
) {
  useEffect(() => {
    let mounted = true;
    const isMounted = () => mounted;
    void hydrateStoredSession({ setSession, setStatus, setLoginError }, isMounted);
    return () => {
      mounted = false;
    };
  }, [setLoginError, setSession, setStatus]);
}

async function submitLogin(
  email: string,
  password: string,
  controls: AuthStateControls,
): Promise<void> {
  const gateway = getGatewayClient();

  if (gateway === null) {
    resetAuthState(controls);
    controls.setLoginError("missing_runtime_config");
    return;
  }

  const loginResult = await gateway.client.post<AuthSuccessPayload>("/auth/login", null, {
    email: email.trim(),
    password,
  });

  if (!loginResult.ok || !isAuthSuccessPayload(loginResult.value.data)) {
    await clearAuthSession();
    resetAuthState(controls);
    controls.setLoginError(loginResult.ok ? "login_failed" : loginErrorKind(loginResult.error.kind));
    return;
  }

  const nextSession = toAuthSession(loginResult.value.data);
  await saveAuthSession(nextSession);
  controls.setSession(nextSession);
  controls.setStatus("authed");
  controls.setLoginError(null);
}

function loginErrorKind(kind: "session_expired" | "forbidden" | "http" | "network") {
  return kind === "network" ? "network_error" : "login_failed";
}

export function AuthProvider({
  children,
}: AuthProviderProps): ReactNode {
  const [session, setSession] = useState<AuthSession | null>(null);
  const [status, setStatus] = useState<AuthStatus>("loading");
  const [loginError, setLoginError] = useState<string | null>(null);
  const controls: AuthStateControls = { setSession, setStatus, setLoginError };
  useHydratedAuthState(setSession, setStatus, setLoginError);

  async function logout(): Promise<void> {
    resetAuthState(controls);
    await clearAuthSession();
  }

  async function login(email: string, password: string): Promise<void> {
    await submitLogin(email, password, controls);
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
