import { registerPush } from '../src/push/registerPush';
import { createGatewayClient } from '../src/api/client';
import * as Notifications from 'expo-notifications';
import { Platform } from 'react-native';

jest.mock('expo-notifications', () => ({
  requestPermissionsAsync: jest.fn(),
  getExpoPushTokenAsync: jest.fn(),
}));

jest.mock('react-native', () => ({
  Platform: { OS: 'ios' },
}));

const BASE_URL = 'http://localhost:4000';
const SESSION_REF = 'opaque-session-abc123';
const EXPO_TOKEN = 'ExponentPushToken[xxxxxxxxxxxxxxxxxxxxxx]';

function makeMockResponse(status: number, bodyObj: unknown = {}): Response {
  const bodyText =
    bodyObj === undefined ? '' : typeof bodyObj === 'string' ? bodyObj : JSON.stringify(bodyObj);
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: { get: () => null },
    json: () => Promise.resolve(bodyObj),
    text: () => Promise.resolve(bodyText),
  } as unknown as Response;
}

describe('registerPush', () => {
  let mockFetch: jest.Mock;
  const client = createGatewayClient({ gatewayBaseUrl: BASE_URL, timeoutMs: 5000 });
  const mockRequestPermissions = Notifications.requestPermissionsAsync as jest.Mock;
  const mockGetToken = Notifications.getExpoPushTokenAsync as jest.Mock;

  beforeEach(() => {
    mockFetch = jest.fn();
    (globalThis as unknown as Record<string, unknown>).fetch = mockFetch;
    jest.clearAllMocks();
  });

  it('HP-1: registers token successfully on 201', async () => {
    mockRequestPermissions.mockResolvedValue({ status: 'granted' });
    mockGetToken.mockResolvedValue({ data: EXPO_TOKEN });
    mockFetch.mockResolvedValue(makeMockResponse(201));

    await registerPush(client, SESSION_REF);

    expect(mockFetch).toHaveBeenCalledTimes(1);
    const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
    expect(url).toBe(`${BASE_URL}/api/notifications/push-tokens`);
    expect(init.method).toBe('POST');
    const body = JSON.parse(init.body as string) as { token: string; platform: string };
    expect(body.token).toBe(EXPO_TOKEN);
    expect(body.platform).toBe('ios');
  });

  it('EC-1: returns early without API call when permission denied', async () => {
    mockRequestPermissions.mockResolvedValue({ status: 'denied' });

    await registerPush(client, SESSION_REF);

    expect(mockFetch).not.toHaveBeenCalled();
    expect(mockGetToken).not.toHaveBeenCalled();
  });

  it('EC-2: returns early without API call when sessionRef is null', async () => {
    await registerPush(client, null);

    expect(mockFetch).not.toHaveBeenCalled();
    expect(mockRequestPermissions).not.toHaveBeenCalled();
  });

  it('HP-2: treats 409 conflict as success (idempotent)', async () => {
    mockRequestPermissions.mockResolvedValue({ status: 'granted' });
    mockGetToken.mockResolvedValue({ data: EXPO_TOKEN });
    mockFetch.mockResolvedValue(makeMockResponse(409, { error: 'token already registered' }));

    await expect(registerPush(client, SESSION_REF)).resolves.toBeUndefined();
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it('EC-3: returns early on session_expired without throwing', async () => {
    mockRequestPermissions.mockResolvedValue({ status: 'granted' });
    mockGetToken.mockResolvedValue({ data: EXPO_TOKEN });
    mockFetch.mockResolvedValue(makeMockResponse(401));

    await expect(registerPush(client, SESSION_REF)).resolves.toBeUndefined();
  });

  it('EC-4: swallows network error without throwing', async () => {
    mockRequestPermissions.mockResolvedValue({ status: 'granted' });
    mockGetToken.mockResolvedValue({ data: EXPO_TOKEN });
    mockFetch.mockRejectedValue(new Error('network failure'));

    await expect(registerPush(client, SESSION_REF)).resolves.toBeUndefined();
  });

  it('EC-5: swallows getExpoPushTokenAsync failure without throwing', async () => {
    mockRequestPermissions.mockResolvedValue({ status: 'granted' });
    mockGetToken.mockRejectedValue(new Error('device not registered'));

    await expect(registerPush(client, SESSION_REF)).resolves.toBeUndefined();
    expect(mockFetch).not.toHaveBeenCalled();
  });
});
