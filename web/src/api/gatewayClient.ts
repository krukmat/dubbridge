import type { GatewayErrorKind, GatewayResult } from './types';

export type { GatewayErrorKind, GatewayResult };

export type GatewayClient = {
  get<T>(path: string): Promise<GatewayResult<T>>;
  post<T>(path: string, body: unknown): Promise<GatewayResult<T>>;
  put<T>(path: string, body: unknown): Promise<GatewayResult<T>>;
};

// Three base64url segments — reject to prevent accidental JWT persistence.
const JWT_PATTERN = /^[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+$/;

function isBearerToken(value: string): boolean {
  return JWT_PATTERN.test(value.trim());
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<GatewayResult<T>> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  try {
    const res = await fetch(path, {
      method,
      headers,
      credentials: 'include',
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });

    // Guard: never persist a bearer token received from the server.
    const auth = res.headers.get('Authorization') ?? '';
    if (isBearerToken(auth)) {
      return { ok: false, error: { kind: 'network', message: 'unexpected bearer token in response' } };
    }

    if (res.status === 401) {
      return { ok: false, error: { kind: 'session_expired' } };
    }
    if (res.status === 403) {
      return { ok: false, error: { kind: 'forbidden' } };
    }
    if (!res.ok) {
      return { ok: false, error: { kind: 'http', status: res.status } };
    }

    const data = (await res.json()) as T;
    return { ok: true, value: data };
  } catch (err: unknown) {
    const error = err as Error;
    return { ok: false, error: { kind: 'network', message: error.message ?? 'unknown network error' } };
  }
}

export function createGatewayClient(): GatewayClient {
  return {
    get<T>(path: string) {
      return request<T>('GET', path);
    },
    post<T>(path: string, body: unknown) {
      return request<T>('POST', path, body);
    },
    put<T>(path: string, body: unknown) {
      return request<T>('PUT', path, body);
    },
  };
}

export const gatewayClient = createGatewayClient();
