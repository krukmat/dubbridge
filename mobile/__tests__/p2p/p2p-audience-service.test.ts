import type { GatewayClient } from "../../src/api/client";
import { P2PAudienceService } from "../../src/p2p/P2PAudienceService";
import type {
  DeviceEnvelope,
  DeviceIdentity,
  DevicePublicIdentity,
} from "../../src/p2p/device/DeviceIdentity";

function client(overrides: Partial<GatewayClient>): GatewayClient {
  return {
    get: jest.fn(),
    post: jest.fn(),
    postMultipart: jest.fn(),
    ...overrides,
  } as GatewayClient;
}

function deviceIdentity(overrides: Partial<DeviceIdentity> = {}): DeviceIdentity {
  return {
    getOrCreateP256Identity: jest.fn<Promise<DevicePublicIdentity>, []>().mockResolvedValue({
      keyId: "android-key-1",
      publicKeySpkiBase64: "public-spki-only",
    }),
    unwrapEnvelope: jest.fn<Promise<string>, [DeviceEnvelope]>().mockResolvedValue("ck-base64"),
    ...overrides,
  };
}

describe("P2PAudienceService K1 boundary", () => {
  it("registers only the opaque public device identity", async () => {
    const post = jest.fn().mockResolvedValue({
      ok: true,
      value: {
        data: { id: "device-1", key_id: "android-key-1", created_at_unix: 42 },
        sessionRotation: null,
      },
    });
    const service = new P2PAudienceService(client({ post }), deviceIdentity());

    const result = await service.ensureDevice("access");

    expect(result.ok).toBe(true);
    expect(post).toHaveBeenCalledWith("/api/p2p/devices", "access", {
      key_id: "android-key-1",
      public_key_spki_base64: "public-spki-only",
    });
    expect(JSON.stringify(post.mock.calls)).not.toContain("private");
  });

  it("fetches the authorized envelope and delegates unwrap to the opaque identity", async () => {
    const get = jest.fn().mockResolvedValue({
      ok: true,
      value: {
        data: {
          profile_version: "p2p-k1-hpke-v1",
          key_id: "android-key-1",
          encapsulated_key_base64: "enc-base64",
          ciphertext_base64: "cipher-base64",
          binding_json: "{\"authorization_id\":\"auth-1\"}",
        },
        sessionRotation: null,
      },
    });
    const unwrapEnvelope = jest.fn<Promise<string>, [DeviceEnvelope]>().mockResolvedValue("ck-base64");
    const identity = deviceIdentity({ unwrapEnvelope });
    const service = new P2PAudienceService(client({ get }), identity);

    const result = await service.getTransientContentKey("access", "auth-1");

    expect(get).toHaveBeenCalledWith("/api/p2p/authorizations/auth-1/device-envelope", "access");
    expect(unwrapEnvelope).toHaveBeenCalledWith({
      profileVersion: "p2p-k1-hpke-v1",
      keyId: "android-key-1",
      encapsulatedKeyBase64: "enc-base64",
      ciphertextBase64: "cipher-base64",
      bindingJson: "{\"authorization_id\":\"auth-1\"}",
    });
    expect(result).toEqual({
      ok: true,
      value: { data: "ck-base64", sessionRotation: null },
    });
  });

  it("does not attempt device unwrap when envelope authorization fails", async () => {
    const get = jest.fn().mockResolvedValue({
      ok: false,
      error: { kind: "http", status: 404 },
    });
    const unwrapEnvelope = jest.fn<Promise<string>, [DeviceEnvelope]>().mockResolvedValue("ck-base64");
    const service = new P2PAudienceService(
      client({ get }),
      deviceIdentity({ unwrapEnvelope }),
    );

    const result = await service.getTransientContentKey("access", "auth-1");

    expect(result).toEqual({ ok: false, error: { kind: "http", status: 404 } });
    expect(unwrapEnvelope).not.toHaveBeenCalled();
  });
});
