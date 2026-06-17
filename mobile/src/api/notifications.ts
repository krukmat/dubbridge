import type { GatewayClient, GatewayResult } from './client';

export type NotificationItem = {
  id: string;
  kind: string;
  ref_entity_type: string;
  ref_entity_id: string;
  actor_subject_id: string | null;
  read_at: string | null;
  created_at: string;
};

export type NotificationListResponse = {
  notifications: NotificationItem[];
};

export function listNotifications(
  client: GatewayClient,
  sessionRef: string | null,
): Promise<GatewayResult<NotificationListResponse>> {
  return client.get<NotificationListResponse>('/api/notifications', sessionRef);
}

export function markNotificationsRead(
  client: GatewayClient,
  sessionRef: string | null,
  ids: string[],
): Promise<GatewayResult<void>> {
  return client.post<void>('/api/notifications/mark-read', sessionRef, { ids });
}

export async function registerPushToken(
  client: GatewayClient,
  sessionRef: string,
  token: string,
  platform: string,
): Promise<GatewayResult<void>> {
  return client.post<void>('/api/notifications/push-tokens', sessionRef, { token, platform });
}
