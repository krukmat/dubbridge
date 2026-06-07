// P3 T2: unit tests for the typed gateway API client (stubbed fetch, no real network)

import { createGatewayClient } from '../src/api/client';

const BASE_URL = 'http://localhost:4000';
const SESSION_REF = 'opaque-session-abc123';

function makeMockResponse(
  status: number,
  bodyObj: unknown,
  responseHeaders: Record<string, string> = {}
): Response {
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: {
      get(name: string): string | null {
        const lower = name.toLowerCase();
        const key = Object.keys(responseHeaders).find(
          (k) => k.toLowerCase() === lower
        );
        return key !== undefined ? (responseHeaders[key] ?? null) : null;
      },
    },
    json: () => Promise.resolve(bodyObj),
  } as unknown as Response;
}

describe('createGatewayClient', () => {
  let mockFetch: jest.Mock;

  // client is stateless config-holder; create once, mock fetch per test
  const client = createGatewayClient({ gatewayBaseUrl: BASE_URL, timeoutMs: 5000 });

  beforeEach(() => {
    mockFetch = jest.fn();
    // patch global fetch for the duration of each test
    (globalThis as unknown as Record<string, unknown>).fetch = mockFetch;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  // ── HP-1 ──────────────────────────────────────────────────────────────────
  describe('HP-1: authenticated GET returns typed data + null rotation', () => {
    it('attaches X-Dubbridge-Session header and returns typed data', async () => {
      const body = [{ id: 'a1', name: 'test asset' }];
      mockFetch.mockResolvedValueOnce(makeMockResponse(200, body));

      const result = await client.get<typeof body>('/api/assets', SESSION_REF);

      expect(result.ok).toBe(true);
      if (result.ok) {
        expect(result.value.data).toEqual(body);
        expect(result.value.sessionRotation).toBeNull();
      }

      // verify session ref was forwarded
      expect(mockFetch).toHaveBeenCalledWith(
        `${BASE_URL}/api/assets`,
        expect.objectContaining({
          headers: expect.objectContaining({
            'X-Dubbridge-Session': SESSION_REF,
          }),
        })
      );
    });
  });

  // ── HP-2 ──────────────────────────────────────────────────────────────────
  describe('HP-2: gateway response with rotation header exposes new session ref', () => {
    it('captures rotated X-Dubbridge-Session from response headers', async () => {
      const newRef = 'rotated-opaque-xyz789';
      mockFetch.mockResolvedValueOnce(
        makeMockResponse(200, { items: [] }, { 'X-Dubbridge-Session': newRef })
      );

      const result = await client.get<{ items: unknown[] }>('/api/assets', SESSION_REF);

      expect(result.ok).toBe(true);
      if (result.ok) {
        expect(result.value.sessionRotation).toBe(newRef);
      }
    });
  });

  // ── EC-1 ──────────────────────────────────────────────────────────────────
  describe('EC-1: 401 → session_expired', () => {
    it('returns session_expired error kind on 401', async () => {
      mockFetch.mockResolvedValueOnce(makeMockResponse(401, {}));

      const result = await client.get('/api/assets', SESSION_REF);

      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.kind).toBe('session_expired');
      }
    });
  });

  // ── EC-2 ──────────────────────────────────────────────────────────────────
  describe('EC-2: AbortError → network timeout', () => {
    it('returns network error with "timeout" message on AbortError', async () => {
      const abortError = new Error('The user aborted a request.');
      abortError.name = 'AbortError';
      mockFetch.mockRejectedValueOnce(abortError);

      const result = await client.get('/api/assets', SESSION_REF);

      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.kind).toBe('network');
        if (result.error.kind === 'network') {
          expect(result.error.message).toBe('timeout');
        }
      }
    });
  });

  // ── EC-3 ──────────────────────────────────────────────────────────────────
  describe('EC-3: missing rotation header → sessionRotation null', () => {
    it('leaves sessionRotation as null when response has no rotation header', async () => {
      mockFetch.mockResolvedValueOnce(makeMockResponse(200, { ok: true }));

      const result = await client.get('/api/data', SESSION_REF);

      expect(result.ok).toBe(true);
      if (result.ok) {
        expect(result.value.sessionRotation).toBeNull();
      }
    });
  });

  // ── EC-4 ──────────────────────────────────────────────────────────────────
  describe('EC-4: JWT-like rotation header is rejected', () => {
    it('rejects a JWT-looking X-Dubbridge-Session, sessionRotation stays null', async () => {
      const jwtLike = 'eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyIn0.SomeSignatureValue';
      mockFetch.mockResolvedValueOnce(
        makeMockResponse(200, { ok: true }, { 'X-Dubbridge-Session': jwtLike })
      );

      const result = await client.get('/api/data', SESSION_REF);

      expect(result.ok).toBe(true);
      if (result.ok) {
        expect(result.value.sessionRotation).toBeNull();
      }
    });
  });

  // ── Additional: 403 ───────────────────────────────────────────────────────
  describe('403 → forbidden', () => {
    it('returns forbidden error kind on 403', async () => {
      mockFetch.mockResolvedValueOnce(makeMockResponse(403, {}));

      const result = await client.get('/api/restricted', SESSION_REF);

      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.kind).toBe('forbidden');
      }
    });
  });

  // ── Additional: generic network error ────────────────────────────────────
  describe('non-abort network error', () => {
    it('returns network error kind with the error message', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network request failed'));

      const result = await client.get('/api/assets', SESSION_REF);

      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.kind).toBe('network');
        if (result.error.kind === 'network') {
          expect(result.error.message).toBe('Network request failed');
        }
      }
    });
  });

  // ── Additional: null sessionRef ───────────────────────────────────────────
  describe('null sessionRef → no X-Dubbridge-Session header', () => {
    it('omits X-Dubbridge-Session when sessionRef is null', async () => {
      mockFetch.mockResolvedValueOnce(makeMockResponse(200, {}));

      await client.get('/api/public', null);

      const calledInit = mockFetch.mock.calls[0]?.[1] as RequestInit;
      const sentHeaders = calledInit?.headers as Record<string, string> | undefined;
      expect(sentHeaders?.['X-Dubbridge-Session']).toBeUndefined();
    });
  });

  // ── Additional: POST with body ────────────────────────────────────────────
  describe('POST serializes body as JSON', () => {
    it('sends body as JSON string', async () => {
      const payload = { name: 'test' };
      mockFetch.mockResolvedValueOnce(makeMockResponse(200, { created: true }));

      const result = await client.post<{ created: boolean }>('/api/assets', SESSION_REF, payload);

      expect(result.ok).toBe(true);
      const calledInit = mockFetch.mock.calls[0]?.[1] as RequestInit;
      expect(calledInit.body).toBe(JSON.stringify(payload));
    });
  });
});
