import {
  AndroidKeystoreDeviceIdentity,
  DeviceIdentityUnavailableError,
  type DeviceEnvelope,
} from "../../src/p2p/device/DeviceIdentity";

const envelope: DeviceEnvelope = {
  profileVersion: "p2p-k1-hpke-v1",
  keyId: "dubbridge-p2p-k1-v1",
  encapsulatedKeyBase64: "encapsulated",
  ciphertextBase64: "ciphertext",
  bindingJson: "{}",
};

describe("K1 Android device identity boundary", () => {
  it("fails closed when the native module cannot be resolved", async () => {
    const identity = new AndroidKeystoreDeviceIdentity({
      platformOs: "android",
      requireModule: () => {
        throw new Error("native module missing");
      },
    });

    await expect(identity.getOrCreateP256Identity()).rejects.toBeInstanceOf(
      DeviceIdentityUnavailableError,
    );
    await expect(identity.unwrapEnvelope(envelope)).rejects.toBeInstanceOf(
      DeviceIdentityUnavailableError,
    );
  });

  it("fails closed outside Android instead of introducing a software identity", async () => {
    const requireModule = jest.fn();
    const identity = new AndroidKeystoreDeviceIdentity({
      platformOs: "ios",
      requireModule,
    });

    await expect(identity.getOrCreateP256Identity()).rejects.toMatchObject({
      name: "DeviceIdentityUnavailableError",
      message: "K1 device identity is Android-only in MVP-0",
    });
    expect(requireModule).not.toHaveBeenCalled();
  });

  it("has no software fallback when native methods are absent", async () => {
    const identity = new AndroidKeystoreDeviceIdentity({
      platformOs: "android",
      requireModule: () => ({}),
    });

    await expect(identity.getOrCreateP256Identity()).rejects.toMatchObject({
      name: "DeviceIdentityUnavailableError",
    });
    await expect(identity.unwrapEnvelope(envelope)).rejects.toMatchObject({
      name: "DeviceIdentityUnavailableError",
      message: "Native HPKE unwrap is unavailable",
    });
  });

  it("rejects malformed native public identity instead of inventing one in JS", async () => {
    const identity = new AndroidKeystoreDeviceIdentity({
      platformOs: "android",
      requireModule: () => ({
        getOrCreateP256Identity: jest.fn().mockResolvedValue({
          keyId: "",
          publicKeySpkiBase64: "",
        }),
      }),
    });

    await expect(identity.getOrCreateP256Identity()).rejects.toMatchObject({
      name: "DeviceIdentityUnavailableError",
      message: "Native P-256 identity returned an invalid public identity",
    });
  });

  it("returns only native-produced public identity and transient CK material", async () => {
    const getOrCreateP256Identity = jest.fn().mockResolvedValue({
      keyId: "dubbridge-p2p-k1-v1",
      publicKeySpkiBase64: "native-spki",
    });
    const unwrapHpkeBaseEnvelope = jest.fn().mockResolvedValue("transient-ck");
    const identity = new AndroidKeystoreDeviceIdentity({
      platformOs: "android",
      requireModule: () => ({
        getOrCreateP256Identity,
        unwrapHpkeBaseEnvelope,
      }),
    });

    await expect(identity.getOrCreateP256Identity()).resolves.toEqual({
      keyId: "dubbridge-p2p-k1-v1",
      publicKeySpkiBase64: "native-spki",
    });
    await expect(identity.unwrapEnvelope(envelope)).resolves.toBe("transient-ck");
    expect(unwrapHpkeBaseEnvelope).toHaveBeenCalledWith(envelope);
  });
});
