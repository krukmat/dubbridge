// P3 T2: typed gateway API client — all authenticated calls go through /api/* proxy

import * as FileSystem from 'expo-file-system/legacy';

import type { GatewayErrorKind, GatewayResponse, GatewayResult } from './types';

export type { GatewayErrorKind, GatewayResponse, GatewayResult };

export type ClientConfig = {
  gatewayBaseUrl: string;
  /** Request timeout in milliseconds. Defaults to 10 000. */
  timeoutMs?: number;
};

export type MultipartUpload = {
  fileUri: string;
  fileName: string;
  mimeType: string;
  fields?: Record<string, string>;
};

export type GatewayClient = {
  get<T>(path: string, sessionRef: string | null): Promise<GatewayResult<T>>;
  post<T>(path: string, sessionRef: string | null, body: unknown): Promise<GatewayResult<T>>;
  postMultipart<T>(path: string, sessionRef: string | null, upload: MultipartUpload): Promise<GatewayResult<T>>;
};

// Three base64url segments separated by dots — reject to prevent accidental JWT persistence.
const JWT_PATTERN = /^[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+$/;

function extractRotation(headers: { get(name: string): string | null }): string | null {
  const value = headers.get('X-Dubbridge-Session');
  if (!value || value.trim() === '') return null;
  const trimmed = value.trim();
  if (JWT_PATTERN.test(trimmed)) return null;
  return trimmed;
}

async function parseResponseBody<T>(res: Response): Promise<T> {
  if (res.status === 204) {
    return undefined as T;
  }

  const text = await res.text();
  if (text.trim() === '') {
    return undefined as T;
  }

  return JSON.parse(text) as T;
}

export function createGatewayClient(config: ClientConfig): GatewayClient {
  const { gatewayBaseUrl, timeoutMs = 10_000 } = config;

  async function request<T>(
    method: string,
    path: string,
    sessionRef: string | null,
    body?: unknown,
  ): Promise<GatewayResult<T>> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (sessionRef !== null) {
      headers['X-Dubbridge-Session'] = sessionRef;
    }

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);

    try {
      const res = await fetch(`${gatewayBaseUrl}${path}`, {
        method,
        headers,
        body: body !== undefined ? JSON.stringify(body) : undefined,
        signal: controller.signal,
      });

      if (res.status === 401) {
        return { ok: false, error: { kind: 'session_expired' } };
      }
      if (res.status === 403) {
        return { ok: false, error: { kind: 'forbidden' } };
      }
      if (!res.ok) {
        return { ok: false, error: { kind: 'http', status: res.status } };
      }

      const sessionRotation = extractRotation(res.headers);
      const data = await parseResponseBody<T>(res);
      return { ok: true, value: { data, sessionRotation } };
    } catch (err: unknown) {
      const error = err as Error;
      if (error.name === 'AbortError') {
        return { ok: false, error: { kind: 'network', message: 'timeout' } };
      }
      return {
        ok: false,
        error: { kind: 'network', message: error.message ?? 'unknown network error' },
      };
    } finally {
      clearTimeout(timer);
    }
  }

  async function postMultipart<T>(
    path: string,
    sessionRef: string | null,
    upload: MultipartUpload,
  ): Promise<GatewayResult<T>> {
    const headers: Record<string, string> = {};
    if (sessionRef !== null) {
      headers['X-Dubbridge-Session'] = sessionRef;
    }

    try {
      const result = await FileSystem.uploadAsync(`${gatewayBaseUrl}${path}`, upload.fileUri, {
        httpMethod: 'POST',
        uploadType: FileSystem.FileSystemUploadType.MULTIPART,
        fieldName: 'file',
        mimeType: upload.mimeType,
        parameters: { title: upload.fileName, ...upload.fields },
        headers,
        sessionType: FileSystem.FileSystemSessionType.BACKGROUND,
      });

      if (result.status === 401) {
        return { ok: false, error: { kind: 'session_expired' } };
      }
      if (result.status === 403) {
        return { ok: false, error: { kind: 'forbidden' } };
      }
      if (result.status < 200 || result.status >= 300) {
        if (result.status === 413) {
          return { ok: false, error: { kind: 'http', status: 413 } };
        }
        return { ok: false, error: { kind: 'http', status: result.status } };
      }

      // expo-file-system does not expose response headers, so no session rotation here.
      const data = JSON.parse(result.body) as T;
      return { ok: true, value: { data, sessionRotation: null } };
    } catch (err: unknown) {
      const error = err as Error;
      return {
        ok: false,
        error: { kind: 'network', message: error.message ?? 'unknown network error' },
      };
    }
  }

  return {
    get<T>(path: string, sessionRef: string | null) {
      return request<T>('GET', path, sessionRef);
    },
    post<T>(path: string, sessionRef: string | null, body: unknown) {
      return request<T>('POST', path, sessionRef, body);
    },
    postMultipart,
  };
}
