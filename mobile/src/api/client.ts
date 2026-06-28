import * as FileSystem from "expo-file-system/legacy";

import type { GatewayErrorKind, GatewayResponse, GatewayResult } from "./types";

export type { GatewayErrorKind, GatewayResponse, GatewayResult };

export type ClientConfig = {
  gatewayBaseUrl: string;
  timeoutMs?: number;
};

export type MultipartUpload = {
  fileUri: string;
  fileName: string;
  mimeType: string;
  fields?: Record<string, string>;
};

export type GatewayClient = {
  get<T>(path: string, accessToken: string | null): Promise<GatewayResult<T>>;
  post<T>(path: string, accessToken: string | null, body: unknown): Promise<GatewayResult<T>>;
  postMultipart<T>(
    path: string,
    accessToken: string | null,
    upload: MultipartUpload,
  ): Promise<GatewayResult<T>>;
};

async function parseResponseBody<T>(res: Response): Promise<T> {
  if (res.status === 204) {
    return undefined as T;
  }

  const text = await res.text();
  if (text.trim() === "") {
    return undefined as T;
  }

  return JSON.parse(text) as T;
}

function withBearer(
  headers: Record<string, string>,
  accessToken: string | null,
): Record<string, string> {
  if (accessToken === null) {
    return headers;
  }

  return {
    ...headers,
    Authorization: `Bearer ${accessToken}`,
  };
}

function mapHttpError(status: number) {
  if (status === 401) return { ok: false as const, error: { kind: "session_expired" as const } };
  if (status === 403) return { ok: false as const, error: { kind: "forbidden" as const } };
  return { ok: false as const, error: { kind: "http" as const, status } };
}

function buildJsonHeaders(accessToken: string | null) {
  return withBearer({ "Content-Type": "application/json" }, accessToken);
}

function createAbortGuard(timeoutMs: number) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  return {
    signal: controller.signal,
    clear() {
      clearTimeout(timer);
    },
  };
}

function mapNetworkError(err: unknown) {
  const error = err as Error;
  if (error.name === "AbortError") {
    return { ok: false as const, error: { kind: "network" as const, message: "timeout" } };
  }
  return {
    ok: false as const,
    error: { kind: "network" as const, message: error.message ?? "unknown network error" },
  };
}

async function mapJsonResponse<T>(res: Response): Promise<GatewayResult<T>> {
  if (!res.ok) {
    return mapHttpError(res.status);
  }

  const data = await parseResponseBody<T>(res);
  return { ok: true, value: { data, sessionRotation: null } };
}

function mapUploadStatus<T>(status: number): GatewayResult<T> | null {
  if (status >= 200 && status < 300) return null;
  return mapHttpError(status === 413 ? 413 : status);
}

function createRequest(
  gatewayBaseUrl: string,
  timeoutMs: number,
) {
  return async function request<T>(
    method: string,
    path: string,
    accessToken: string | null,
    body?: unknown,
  ): Promise<GatewayResult<T>> {
    const headers = buildJsonHeaders(accessToken);
    const guard = createAbortGuard(timeoutMs);

    try {
      const res = await fetch(`${gatewayBaseUrl}${path}`, {
        method,
        headers,
        body: body !== undefined ? JSON.stringify(body) : undefined,
        signal: guard.signal,
      });
      return mapJsonResponse<T>(res);
    } catch (err: unknown) {
      return mapNetworkError(err);
    } finally {
      guard.clear();
    }
  };
}

function createMultipartPost(gatewayBaseUrl: string) {
  return async function postMultipart<T>(
    path: string,
    accessToken: string | null,
    upload: MultipartUpload,
  ): Promise<GatewayResult<T>> {
    const headers = withBearer({}, accessToken);

    try {
      const result = await FileSystem.uploadAsync(`${gatewayBaseUrl}${path}`, upload.fileUri, {
        httpMethod: "POST",
        uploadType: FileSystem.FileSystemUploadType.MULTIPART,
        fieldName: "file",
        mimeType: upload.mimeType,
        parameters: { title: upload.fileName, ...upload.fields },
        headers,
        sessionType: FileSystem.FileSystemSessionType.BACKGROUND,
      });
      const statusError = mapUploadStatus<T>(result.status);
      if (statusError) return statusError;

      const data = JSON.parse(result.body) as T;
      return { ok: true, value: { data, sessionRotation: null } };
    } catch (err: unknown) {
      return mapNetworkError(err);
    }
  };
}

export function createGatewayClient(config: ClientConfig): GatewayClient {
  const { gatewayBaseUrl, timeoutMs = 10_000 } = config;
  const request = createRequest(gatewayBaseUrl, timeoutMs);
  const postMultipart = createMultipartPost(gatewayBaseUrl);

  return {
    get<T>(path: string, accessToken: string | null) {
      return request<T>("GET", path, accessToken);
    },
    post<T>(path: string, accessToken: string | null, body: unknown) {
      return request<T>("POST", path, accessToken, body);
    },
    postMultipart,
  };
}
