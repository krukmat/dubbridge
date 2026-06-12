export type GatewayErrorKind =
  | { kind: 'session_expired' }
  | { kind: 'forbidden' }
  | { kind: 'network'; message: string }
  | { kind: 'http'; status: number };

export type GatewayResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: GatewayErrorKind };
