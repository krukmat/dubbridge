// P3 T3a: TDD tests for session storage primitives (expo-secure-store, JWT guard, rotation)
// All expo-secure-store calls are mocked — no real Keychain/Keystore access.

import * as SecureStore from 'expo-secure-store';
import {
  saveSessionRef,
  loadSessionRef,
  clearSessionRef,
  updateSessionRef,
  isJwtLike,
} from '../src/auth/session';

jest.mock('expo-secure-store', () => ({
  setItemAsync: jest.fn(),
  getItemAsync: jest.fn(),
  deleteItemAsync: jest.fn(),
}));

const SESSION_KEY = 'dubbridge_session_ref';
const OPAQUE_REF = 'opaque-session-abc123';
// Three base64url segments — matches JWT structure
const JWT_LIKE = 'eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyIn0.SomeSignatureValue';

describe('session storage primitives', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  // ── isJwtLike ─────────────────────────────────────────────────────────────
  describe('isJwtLike', () => {
    it('returns true for a three-segment base64url string', () => {
      expect(isJwtLike(JWT_LIKE)).toBe(true);
    });

    it('returns true for a minimal three-segment value', () => {
      expect(isJwtLike('aaa.bbb.ccc')).toBe(true);
    });

    it('returns false for an opaque session reference', () => {
      expect(isJwtLike(OPAQUE_REF)).toBe(false);
    });

    it('returns false for a two-segment string', () => {
      expect(isJwtLike('aaa.bbb')).toBe(false);
    });

    it('returns false for a four-segment string', () => {
      expect(isJwtLike('aaa.bbb.ccc.ddd')).toBe(false);
    });

    it('returns false for an empty string', () => {
      expect(isJwtLike('')).toBe(false);
    });
  });

  // ── saveSessionRef ────────────────────────────────────────────────────────
  describe('saveSessionRef', () => {
    it('calls setItemAsync with the session key and ref', async () => {
      (SecureStore.setItemAsync as jest.Mock).mockResolvedValueOnce(undefined);

      await saveSessionRef(OPAQUE_REF);

      expect(SecureStore.setItemAsync).toHaveBeenCalledWith(SESSION_KEY, OPAQUE_REF);
      expect(SecureStore.setItemAsync).toHaveBeenCalledTimes(1);
    });
  });

  // ── loadSessionRef ────────────────────────────────────────────────────────
  describe('loadSessionRef', () => {
    it('returns the stored value when present', async () => {
      (SecureStore.getItemAsync as jest.Mock).mockResolvedValueOnce(OPAQUE_REF);

      const result = await loadSessionRef();

      expect(result).toBe(OPAQUE_REF);
      expect(SecureStore.getItemAsync).toHaveBeenCalledWith(SESSION_KEY);
    });

    it('returns null when the key is absent', async () => {
      (SecureStore.getItemAsync as jest.Mock).mockResolvedValueOnce(null);

      const result = await loadSessionRef();

      expect(result).toBeNull();
    });

    // Acceptance criterion: stored value must NOT be JWT-like
    it('returns a value that is NOT JWT-like', async () => {
      (SecureStore.getItemAsync as jest.Mock).mockResolvedValueOnce(OPAQUE_REF);

      const result = await loadSessionRef();

      expect(result).not.toBeNull();
      if (result !== null) {
        expect(isJwtLike(result)).toBe(false);
      }
    });
  });

  // ── clearSessionRef ───────────────────────────────────────────────────────
  describe('clearSessionRef', () => {
    it('calls deleteItemAsync with the session key', async () => {
      (SecureStore.deleteItemAsync as jest.Mock).mockResolvedValueOnce(undefined);

      await clearSessionRef();

      expect(SecureStore.deleteItemAsync).toHaveBeenCalledWith(SESSION_KEY);
      expect(SecureStore.deleteItemAsync).toHaveBeenCalledTimes(1);
    });
  });

  // ── updateSessionRef ──────────────────────────────────────────────────────
  describe('updateSessionRef', () => {
    it('is a no-op when rotation is null', async () => {
      await updateSessionRef(null);

      expect(SecureStore.setItemAsync).not.toHaveBeenCalled();
    });

    // Acceptance criterion: JWT-like value must not be persisted
    it('is a no-op when rotation is JWT-like → setItemAsync NOT called', async () => {
      await updateSessionRef(JWT_LIKE);

      expect(SecureStore.setItemAsync).not.toHaveBeenCalled();
    });

    it('is a no-op when rotation is a minimal three-segment JWT-like value', async () => {
      await updateSessionRef('hdr.payload.sig');

      expect(SecureStore.setItemAsync).not.toHaveBeenCalled();
    });

    it('saves the ref when rotation is a valid opaque reference', async () => {
      (SecureStore.setItemAsync as jest.Mock).mockResolvedValueOnce(undefined);

      await updateSessionRef(OPAQUE_REF);

      expect(SecureStore.setItemAsync).toHaveBeenCalledWith(SESSION_KEY, OPAQUE_REF);
      expect(SecureStore.setItemAsync).toHaveBeenCalledTimes(1);
    });
  });
});
