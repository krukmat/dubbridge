import { Platform } from 'react-native';
import * as Notifications from 'expo-notifications';

import type { GatewayClient } from '../api/client';
import { registerPushToken } from '../api/notifications';

export async function registerPush(
  client: GatewayClient,
  sessionRef: string | null,
): Promise<void> {
  if (sessionRef === null) {
    return;
  }

  const { status } = await Notifications.requestPermissionsAsync();
  if (status !== 'granted') {
    return;
  }

  let token: Notifications.ExpoPushToken;
  try {
    token = await Notifications.getExpoPushTokenAsync();
  } catch (err) {
    console.warn('[registerPush] failed to obtain Expo push token', err);
    return;
  }

  const os = Platform.OS;
  if (os !== 'ios' && os !== 'android') {
    return;
  }
  const platform: 'ios' | 'android' = os;
  const result = await registerPushToken(client, sessionRef, token.data, platform);

  if (!result.ok) {
    if (result.error.kind === 'session_expired') {
      return;
    }
    if (result.error.kind === 'http' && result.error.status === 409) {
      return;
    }
    console.warn('[registerPush] registration failed', result.error);
  }
}
