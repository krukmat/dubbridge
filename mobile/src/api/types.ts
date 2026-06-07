// P3 T2: typed gateway response and error contracts for the mobile API client

export type GatewayErrorKind =
  | { kind: 'session_expired' }
  | { kind: 'forbidden' }
  | { kind: 'network'; message: string }
  | { kind: 'http'; status: number };

export type GatewayResponse<T> = {
  data: T;
  /** Rotated opaque session reference from X-Dubbridge-Session, or null if absent/rejected. */
  sessionRotation: string | null;
};

export type GatewayResult<T> =
  | { ok: true; value: GatewayResponse<T> }
  | { ok: false; error: GatewayErrorKind };
