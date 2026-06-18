import * as FileSystem from "expo-file-system/legacy";

import { createGatewayClient } from "../src/api/client";
import type { MultipartUpload } from "../src/api/client";

jest.mock("expo-file-system/legacy", () => ({
  uploadAsync: jest.fn(),
  FileSystemUploadType: { MULTIPART: "MULTIPART" },
  FileSystemSessionType: { BACKGROUND: "BACKGROUND" },
}));

const BASE_URL = "http://localhost:4000";
const ACCESS_TOKEN = "token-abc";

const SAMPLE_UPLOAD: MultipartUpload = {
  fileUri: "file:///tmp/test.mov",
  fileName: "test.mov",
  mimeType: "video/quicktime",
};

function makeUploadResult(status: number, body: unknown): FileSystem.FileSystemUploadResult {
  return { status, body: JSON.stringify(body), headers: {}, mimeType: null };
}

function makeMockResponse(
  status: number,
  bodyObj: unknown,
): Response {
  const bodyText =
    bodyObj === undefined ? "" : typeof bodyObj === "string" ? bodyObj : JSON.stringify(bodyObj);
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: {
      get(): string | null {
        return null;
      },
    },
    text: () => Promise.resolve(bodyText),
  } as unknown as Response;
}

describe("createGatewayClient", () => {
  let mockFetch: jest.Mock;

  const client = createGatewayClient({ gatewayBaseUrl: BASE_URL, timeoutMs: 5000 });

  beforeEach(() => {
    mockFetch = jest.fn();
    (globalThis as unknown as Record<string, unknown>).fetch = mockFetch;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  it("HP-1: attaches Authorization bearer headers to authenticated GET requests", async () => {
    mockFetch.mockResolvedValueOnce(makeMockResponse(200, [{ id: "a1" }]));

    const result = await client.get<{ id: string }[]>("/api/assets", ACCESS_TOKEN);

    expect(result.ok).toBe(true);
    expect(mockFetch).toHaveBeenCalledWith(
      `${BASE_URL}/api/assets`,
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: `Bearer ${ACCESS_TOKEN}`,
        }),
      }),
    );
  });

  it("HP-2: authenticated responses stay bearer-only and expose null sessionRotation", async () => {
    mockFetch.mockResolvedValueOnce(makeMockResponse(200, { items: [] }));

    const result = await client.get<{ items: unknown[] }>("/api/assets", ACCESS_TOKEN);

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.sessionRotation).toBeNull();
      expect(result.value.data).toEqual({ items: [] });
    }
  });

  it("EC-1: maps 401 to session_expired", async () => {
    mockFetch.mockResolvedValueOnce(makeMockResponse(401, {}));

    const result = await client.get("/api/assets", ACCESS_TOKEN);

    expect(result).toEqual({
      ok: false,
      error: { kind: "session_expired" },
    });
  });

  it("EC-2: omits Authorization when there is no access token", async () => {
    mockFetch.mockResolvedValueOnce(makeMockResponse(200, {}));

    await client.get("/api/public", null);

    const init = mockFetch.mock.calls[0]?.[1] as RequestInit;
    expect((init.headers as Record<string, string>).Authorization).toBeUndefined();
  });

  it("EC-3: multipart uploads also use Authorization bearer headers", async () => {
    (FileSystem.uploadAsync as jest.Mock).mockResolvedValueOnce(
      makeUploadResult(201, { ingest_token: "tok-abc" }),
    );

    const result = await client.postMultipart<{ ingest_token: string }>(
      "/api/ingest",
      ACCESS_TOKEN,
      SAMPLE_UPLOAD,
    );

    expect(result.ok).toBe(true);
    expect(FileSystem.uploadAsync).toHaveBeenCalledWith(
      `${BASE_URL}/api/ingest`,
      SAMPLE_UPLOAD.fileUri,
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: `Bearer ${ACCESS_TOKEN}`,
        }),
      }),
    );
  });
});
