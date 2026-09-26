import type { GatewayClient, GatewayResult } from "../api/client";
import {
  claimP2pInvitation,
  createP2pInvitation,
  getP2pAuthorization,
  getP2pDeviceEnvelope,
  listP2pInvitations,
  registerP2pDevice,
  type P2pAuthorization,
  type P2pClaim,
  type P2pDevice,
  type P2pInvitation,
} from "../api/p2p";
import {
  AndroidKeystoreDeviceIdentity,
  type DeviceIdentity,
} from "./device/DeviceIdentity";

export class P2PAudienceService {
  constructor(
    private readonly client: GatewayClient,
    private readonly deviceIdentity: DeviceIdentity = new AndroidKeystoreDeviceIdentity(),
  ) {}

  async ensureDevice(accessToken: string): Promise<GatewayResult<P2pDevice>> {
    const identity = await this.deviceIdentity.getOrCreateP256Identity();
    return registerP2pDevice(
      this.client,
      accessToken,
      identity.keyId,
      identity.publicKeySpkiBase64,
    );
  }

  createInvitation(
    accessToken: string,
    assetId: string,
    ttlSeconds?: number,
  ): Promise<GatewayResult<{ invitation: P2pInvitation; token: string }>> {
    return createP2pInvitation(this.client, accessToken, assetId, ttlSeconds);
  }

  async claimInvitation(
    accessToken: string,
    token: string,
  ): Promise<GatewayResult<P2pClaim>> {
    const device = await this.ensureDevice(accessToken);
    if (!device.ok) return device;
    return claimP2pInvitation(this.client, accessToken, token, device.value.data.id);
  }

  listInvitations(accessToken: string): Promise<GatewayResult<P2pInvitation[]>> {
    return listP2pInvitations(this.client, accessToken);
  }

  getAuthorization(
    accessToken: string,
    authorizationId: string,
  ): Promise<GatewayResult<P2pAuthorization>> {
    return getP2pAuthorization(this.client, accessToken, authorizationId);
  }

  /**
   * Fetches the current O3-authorized K1 envelope and asks the opaque device
   * identity to unwrap it. The returned CK is transient session material: this
   * service never persists or logs it and callers must consume it immediately.
   */
  async getTransientContentKey(
    accessToken: string,
    authorizationId: string,
  ): Promise<GatewayResult<string>> {
    const result = await getP2pDeviceEnvelope(this.client, accessToken, authorizationId);
    if (!result.ok) return result;

    const envelope = result.value.data;
    const ckBase64 = await this.deviceIdentity.unwrapEnvelope({
      profileVersion: envelope.profileVersion,
      keyId: envelope.keyId,
      encapsulatedKeyBase64: envelope.encapsulatedKeyBase64,
      ciphertextBase64: envelope.ciphertextBase64,
      bindingJson: envelope.bindingJson,
    });

    return {
      ok: true,
      value: {
        data: ckBase64,
        sessionRotation: result.value.sessionRotation,
      },
    };
  }
}
