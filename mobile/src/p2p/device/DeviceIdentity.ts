import { requireNativeModule } from "expo";
import { Platform } from "react-native";

export type DevicePublicIdentity = Readonly<{
  keyId: string;
  publicKeySpkiBase64: string;
}>;

export type DeviceEnvelope = Readonly<{
  profileVersion: string;
  keyId: string;
  encapsulatedKeyBase64: string;
  ciphertextBase64: string;
  bindingJson: string;
}>;

export interface DeviceIdentity {
  getOrCreateP256Identity(): Promise<DevicePublicIdentity>;
  unwrapEnvelope(envelope: DeviceEnvelope): Promise<string>;
}

type NativeDeviceIdentityModule = {
  getOrCreateP256Identity?: () => Promise<{
    keyId?: unknown;
    publicKeySpkiBase64?: unknown;
  }>;
  unwrapHpkeBaseEnvelope?: (envelope: DeviceEnvelope) => Promise<unknown>;
};

export class DeviceIdentityUnavailableError extends Error {
  constructor(message = "Opaque Android P-256 device identity is unavailable") {
    super(message);
    this.name = "DeviceIdentityUnavailableError";
  }
}

function nativeModule(): NativeDeviceIdentityModule {
  if (Platform.OS !== "android") {
    throw new DeviceIdentityUnavailableError("K1 device identity is Android-only in MVP-0");
  }

  try {
    return requireNativeModule<NativeDeviceIdentityModule>("DubBridgeP2PKeyStore");
  } catch {
    throw new DeviceIdentityUnavailableError();
  }
}

/**
 * K1 native boundary. It deliberately has no JavaScript private-key fallback:
 * the P-256 private key must remain opaque inside Android Keystore.
 */
export class AndroidKeystoreDeviceIdentity implements DeviceIdentity {
  async getOrCreateP256Identity(): Promise<DevicePublicIdentity> {
    const module = nativeModule();
    if (typeof module.getOrCreateP256Identity !== "function") {
      throw new DeviceIdentityUnavailableError();
    }
    const result = await module.getOrCreateP256Identity();
    if (
      typeof result.keyId !== "string" ||
      result.keyId.length === 0 ||
      typeof result.publicKeySpkiBase64 !== "string" ||
      result.publicKeySpkiBase64.length === 0
    ) {
      throw new DeviceIdentityUnavailableError("Native P-256 identity returned an invalid public identity");
    }
    return { keyId: result.keyId, publicKeySpkiBase64: result.publicKeySpkiBase64 };
  }

  async unwrapEnvelope(envelope: DeviceEnvelope): Promise<string> {
    const module = nativeModule();
    if (typeof module.unwrapHpkeBaseEnvelope !== "function") {
      throw new DeviceIdentityUnavailableError("Native HPKE unwrap is unavailable");
    }
    const ckBase64 = await module.unwrapHpkeBaseEnvelope(envelope);
    if (typeof ckBase64 !== "string" || ckBase64.length === 0) {
      throw new DeviceIdentityUnavailableError("Native HPKE unwrap returned invalid key material");
    }
    return ckBase64;
  }
}
