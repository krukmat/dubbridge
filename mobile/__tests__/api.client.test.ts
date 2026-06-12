// P3 T2: unit tests for the typed gateway API client (stubbed fetch, no real network)

import { createGatewayClient } from '../src/api/client';
import type { MultipartUpload } from '../src/api/client';
import * as FileSystem from 'expo-file-system/legacy';

jest.mock('expo-file-system/legacy', () => ({
  uploadAsync: jest.fn(),
  FileSystemUploadType: { MULTIPART: 'MULTIPART' },
  FileSystemSessionType: { BACKGROUND: 'BACKGROUND' },
}));

const BASE_URL = 'http://localhost:4000';
const SESSION_REF = 'opaque-session-abc123';

const SAMPLE_UPLOAD: MultipartUpload = {
  fileUri: 'file:///tmp/test.mov',
  fileName: 'test.mov',
  mimeType: 'video/quicktime',
};

function makeUploadResult(status: number, body: unknown): FileSystem.FileSystemUploadResult {
  return { status, body: JSON.stringify(body), headers: {}, mimeType: null };
}

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

  // ── postMultipart: HP-1 ───────────────────────────────────────────────────
  describe('postMultipart HP-1: calls FileSystem.uploadAsync and returns parsed body', () => {
    it('calls uploadAsync with MULTIPART type and returns typed data', async () => {
      (FileSystem.uploadAsync as jest.Mock).mockResolvedValueOnce(
        makeUploadResult(201, { ingest_token: 'tok-abc' })
      );

      const result = await client.postMultipart<{ ingest_token: string }>(
        '/api/ingest',
        SESSION_REF,
        SAMPLE_UPLOAD,
      );

      expect(result.ok).toBe(true);
      if (result.ok) {
        expect(result.value.data.ingest_token).toBe('tok-abc');
        expect(result.value.sessionRotation).toBeNull();
      }

      expect(FileSystem.uploadAsync).toHaveBeenCalledWith(
        `${BASE_URL}/api/ingest`,
        SAMPLE_UPLOAD.fileUri,
        expect.objectContaining({
          uploadType: FileSystem.FileSystemUploadType.MULTIPART,
          fieldName: 'file',
          mimeType: SAMPLE_UPLOAD.mimeType,
          parameters: expect.objectContaining({ title: SAMPLE_UPLOAD.fileName }),
        }),
      );
    });
  });

  // ── postMultipart: HP-2 ───────────────────────────────────────────────────
  describe('postMultipart HP-2: attaches X-Dubbridge-Session header', () => {
    it('forwards session ref in the headers option', async () => {
      (FileSystem.uploadAsync as jest.Mock).mockResolvedValueOnce(
        makeUploadResult(201, { ingest_token: 'tok-xyz' })
      );

      await client.postMultipart('/api/ingest', SESSION_REF, SAMPLE_UPLOAD);

      expect(FileSystem.uploadAsync).toHaveBeenCalledWith(
        expect.any(String),
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({ 'X-Dubbridge-Session': SESSION_REF }),
        }),
      );
    });
  });

  // ── postMultipart: HP-3 ───────────────────────────────────────────────────
  describe('postMultipart HP-3: null sessionRef omits session header', () => {
    it('does not attach X-Dubbridge-Session when sessionRef is null', async () => {
      (FileSystem.uploadAsync as jest.Mock).mockResolvedValueOnce(
        makeUploadResult(201, { ingest_token: 'tok-pub' })
      );

      await client.postMultipart('/api/ingest', null, SAMPLE_UPLOAD);

      const calledOptions = (FileSystem.uploadAsync as jest.Mock).mock.calls[0]?.[2] as Record<string, unknown>;
      const sentHeaders = calledOptions?.headers as Record<string, string> | undefined;
      expect(sentHeaders?.['X-Dubbridge-Session']).toBeUndefined();
    });
  });

  // ── postMultipart: EC-1 ───────────────────────────────────────────────────
  describe('postMultipart EC-1: 401 → session_expired', () => {
    it('returns session_expired on 401', async () => {
      (FileSystem.uploadAsync as jest.Mock).mockResolvedValueOnce(makeUploadResult(401, {}));

      const result = await client.postMultipart('/api/ingest', SESSION_REF, SAMPLE_UPLOAD);

      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.kind).toBe('session_expired');
      }
    });
  });

  // ── postMultipart: EC-2 ───────────────────────────────────────────────────
  describe('postMultipart EC-2: 403 → forbidden', () => {
    it('returns forbidden on 403', async () => {
      (FileSystem.uploadAsync as jest.Mock).mockResolvedValueOnce(makeUploadResult(403, {}));

      const result = await client.postMultipart('/api/ingest', SESSION_REF, SAMPLE_UPLOAD);

      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.kind).toBe('forbidden');
      }
    });
  });

  // ── postMultipart: EC-3 ───────────────────────────────────────────────────
  describe('postMultipart EC-3: network error → network kind', () => {
    it('returns network error on uploadAsync rejection', async () => {
      (FileSystem.uploadAsync as jest.Mock).mockRejectedValueOnce(new Error('Upload network failure'));

      const result = await client.postMultipart('/api/ingest', SESSION_REF, SAMPLE_UPLOAD);

      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.kind).toBe('network');
        if (result.error.kind === 'network') {
          expect(result.error.message).toBe('Upload network failure');
        }
      }
    });
  });

  // ── postMultipart: EC-4 ───────────────────────────────────────────────────
  describe('postMultipart EC-4: extra fields forwarded in parameters', () => {
    it('merges upload.fields into parameters alongside title', async () => {
      (FileSystem.uploadAsync as jest.Mock).mockResolvedValueOnce(
        makeUploadResult(201, { ingest_token: 'tok-fields' })
      );

      const uploadWithFields: MultipartUpload = {
        ...SAMPLE_UPLOAD,
        fields: { custom: 'value' },
      };

      await client.postMultipart('/api/ingest', SESSION_REF, uploadWithFields);

      expect(FileSystem.uploadAsync).toHaveBeenCalledWith(
        expect.any(String),
        expect.any(String),
        expect.objectContaining({
          parameters: expect.objectContaining({ title: SAMPLE_UPLOAD.fileName, custom: 'value' }),
        }),
      );
    });
  });

  // ── postMultipart: EC-5 ───────────────────────────────────────────────────
  describe('postMultipart EC-5: non-2xx (413) → http kind', () => {
    it('returns http error with status 413', async () => {
      (FileSystem.uploadAsync as jest.Mock).mockResolvedValueOnce(makeUploadResult(413, {}));

      const result = await client.postMultipart('/api/ingest', SESSION_REF, SAMPLE_UPLOAD);

      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.kind).toBe('http');
        if (result.error.kind === 'http') {
          expect(result.error.status).toBe(413);
        }
      }
    });
  });
});
