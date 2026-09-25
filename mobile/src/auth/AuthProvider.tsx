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
export type LoginPhase =
  | "idle"
  | "requesting"
  | "response_received"
  | "persisting"
  | "authenticated"
  | "error";

export type AuthContextValue = {
  sessionRef: string | null;
  userId?: string | null;
  status: AuthStatus;
  loginError: string | null;
  loginPhase: LoginPhase;
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
  setLoginPhase: (phase: LoginPhase) => void;
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

async function clearPersistedSession(): Promise<boolean> {
  try {
    await clearAuthSession();
    return true;
  } catch {
    return false;
  }
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
    await clearPersistedSession();
    if (!isMounted()) return;
    resetAuthState(controls);
    controls.setLoginError("session_storage_error");
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
  controls.setLoginPhase("requesting");
  const gateway = getGatewayClient();

  if (gateway === null) {
    resetAuthState(controls);
    controls.setLoginError("missing_runtime_config");
    controls.setLoginPhase("error");
    return;
  }

  let loginResult;
  try {
    loginResult = await gateway.client.post<AuthSuccessPayload>("/auth/login", null, {
      email: email.trim(),
      password,
    });
  } catch {
    resetAuthState(controls);
    controls.setLoginError("unexpected_login_error");
    controls.setLoginPhase("error");
    return;
  }

  controls.setLoginPhase("response_received");

  if (!loginResult.ok || !isAuthSuccessPayload(loginResult.value.data)) {
    const storageCleared = await clearPersistedSession();
    resetAuthState(controls);
    controls.setLoginError(
      storageCleared
        ? loginResult.ok
          ? "login_failed"
          : loginErrorKind(loginResult.error.kind)
        : "session_storage_error",
    );
    controls.setLoginPhase("error");
    return;
  }

  const nextSession = toAuthSession(loginResult.value.data);
  controls.setLoginPhase("persisting");
  try {
    await saveAuthSession(nextSession);
  } catch {
    resetAuthState(controls);
    controls.setLoginError("session_storage_error");
    controls.setLoginPhase("error");
    return;
  }

  controls.setSession(nextSession);
  controls.setStatus("authed");
  controls.setLoginError(null);
  controls.setLoginPhase("authenticated");
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
  const [loginPhase, setLoginPhase] = useState<LoginPhase>("idle");
  const controls: AuthStateControls = {
    setSession,
    setStatus,
    setLoginError,
    setLoginPhase,
  };
  useHydratedAuthState(setSession, setStatus, setLoginError);

  async function logout(): Promise<void> {
    resetAuthState(controls);
    if (!(await clearPersistedSession())) {
      controls.setLoginError("session_storage_error");
    }
  }

  async function login(email: string, password: string): Promise<void> {
    await submitLogin(email, password, controls);
  }

  const value: AuthContextValue = {
    sessionRef: session?.token ?? null,
    userId: session?.userId ?? null,
    status,
    loginError,
    loginPhase,
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
