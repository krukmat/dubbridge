// P3 T3a: secure session reference storage primitives
// Pure storage module — no React, no HTTP. Stores only the opaque session reference;
// never persists a JWT or refresh token (ADR-024).

import * as SecureStore from 'expo-secure-store';

const SESSION_KEY = 'dubbridge_session_ref';

// Three base64url segments separated by dots — matches JWT structure.
// Used to guard against accidental JWT persistence in updateSessionRef.
const JWT_PATTERN = /^[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+$/;

export function isJwtLike(value: string): boolean {
  return JWT_PATTERN.test(value);
}

export async function saveSessionRef(ref: string): Promise<void> {
  await SecureStore.setItemAsync(SESSION_KEY, ref);
}

export async function loadSessionRef(): Promise<string | null> {
  return SecureStore.getItemAsync(SESSION_KEY);
}

export async function clearSessionRef(): Promise<void> {
  await SecureStore.deleteItemAsync(SESSION_KEY);
}

export async function updateSessionRef(rotation: string | null): Promise<void> {
  if (rotation === null) return;
  if (isJwtLike(rotation)) return;
  await saveSessionRef(rotation);
}
