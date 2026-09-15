import type { GatewayClient, GatewayResult } from "./client";

export type P2pReadyDescriptor = Readonly<{
  descriptorVersion: string;
  assetId: string;
  publicationId: string;
  lineageId: string;
  manifestVersion: string;
  manifestDigestSha256: string;
  externalPublicationId: string;
  ckWrapRef: string;
  kekId: string;
  kekVersion: number;
  readyAt: string;
}>;

export type P2pDevice = Readonly<{
  id: string;
  keyId: string;
  createdAtUnix: number;
}>;

export type P2pInvitationStatus = "pending" | "claimed" | "expired" | "revoked";

export type P2pInvitation = Readonly<{
  id: string;
  assetId: string;
  publicationId: string;
  lineageId: string;
  ownerSubjectId: string;
  status: P2pInvitationStatus;
  expiresAtUnix: number;
  claimedAtUnix: number | null;
}>;

export type P2pAuthorization = Readonly<{
  id: string;
  invitationId: string;
  assetId: string;
  publicationId: string;
  lineageId: string;
  viewerSubjectId: string;
  deviceId: string;
  expiresAtUnix: number;
}>;

export type P2pDeviceEnvelope = Readonly<{
  profileVersion: string;
  keyId: string;
  encapsulatedKeyBase64: string;
  ciphertextBase64: string;
  bindingJson: string;
}>;

export type P2pClaim = Readonly<{
  invitation: P2pInvitation;
  authorization: P2pAuthorization;
  descriptor: P2pReadyDescriptor;
}>;

type ApiDevice = {
  id: string;
  key_id: string;
  created_at_unix: number;
};

type ApiInvitation = {
  id: string;
  asset_id: string;
  publication_id: string;
  lineage_id: string;
  owner_subject_id: string;
  status: P2pInvitationStatus;
  expires_at_unix: number;
  claimed_at_unix: number | null;
};

type ApiAuthorization = {
  id: string;
  invitation_id: string;
  asset_id: string;
  publication_id: string;
  lineage_id: string;
  viewer_subject_id: string;
  device_id: string;
  expires_at_unix: number;
};

type ApiDeviceEnvelope = {
  profile_version: string;
  key_id: string;
  encapsulated_key_base64: string;
  ciphertext_base64: string;
  binding_json: string;
};

type ApiReadyDescriptor = {
  descriptor_version: string;
  asset_id: string;
  publication_id: string;
  lineage_id: string;
  manifest_version: string;
  manifest_digest_sha256: string;
  external_publication_id: string;
  ck_wrap_ref: string;
  kek_id: string;
  kek_version: number;
  ready_at: string;
};

type ApiInvitationCreated = {
  invitation: ApiInvitation;
  token: string;
};

type ApiClaim = {
  invitation: ApiInvitation;
  authorization: ApiAuthorization;
  descriptor: ApiReadyDescriptor;
};

function mapResult<A, B>(result: GatewayResult<A>, map: (value: A) => B): GatewayResult<B> {
  if (!result.ok) return result;
  return {
    ok: true,
    value: {
      data: map(result.value.data),
      sessionRotation: result.value.sessionRotation,
    },
  };
}

function mapDevice(value: ApiDevice): P2pDevice {
  return { id: value.id, keyId: value.key_id, createdAtUnix: value.created_at_unix };
}

function mapInvitation(value: ApiInvitation): P2pInvitation {
  return {
    id: value.id,
    assetId: value.asset_id,
    publicationId: value.publication_id,
    lineageId: value.lineage_id,
    ownerSubjectId: value.owner_subject_id,
    status: value.status,
    expiresAtUnix: value.expires_at_unix,
    claimedAtUnix: value.claimed_at_unix,
  };
}

function mapAuthorization(value: ApiAuthorization): P2pAuthorization {
  return {
    id: value.id,
    invitationId: value.invitation_id,
    assetId: value.asset_id,
    publicationId: value.publication_id,
    lineageId: value.lineage_id,
    viewerSubjectId: value.viewer_subject_id,
    deviceId: value.device_id,
    expiresAtUnix: value.expires_at_unix,
  };
}

function mapDeviceEnvelope(value: ApiDeviceEnvelope): P2pDeviceEnvelope {
  return {
    profileVersion: value.profile_version,
    keyId: value.key_id,
    encapsulatedKeyBase64: value.encapsulated_key_base64,
    ciphertextBase64: value.ciphertext_base64,
    bindingJson: value.binding_json,
  };
}

function mapDescriptor(value: ApiReadyDescriptor): P2pReadyDescriptor {
  return {
    descriptorVersion: value.descriptor_version,
    assetId: value.asset_id,
    publicationId: value.publication_id,
    lineageId: value.lineage_id,
    manifestVersion: value.manifest_version,
    manifestDigestSha256: value.manifest_digest_sha256,
    externalPublicationId: value.external_publication_id,
    ckWrapRef: value.ck_wrap_ref,
    kekId: value.kek_id,
    kekVersion: value.kek_version,
    readyAt: value.ready_at,
  };
}

export async function registerP2pDevice(
  client: GatewayClient,
  accessToken: string,
  keyId: string,
  publicKeySpkiBase64: string,
): Promise<GatewayResult<P2pDevice>> {
  const result = await client.post<ApiDevice>("/api/p2p/devices", accessToken, {
    key_id: keyId,
    public_key_spki_base64: publicKeySpkiBase64,
  });
  return mapResult(result, mapDevice);
}

export async function createP2pInvitation(
  client: GatewayClient,
  accessToken: string,
  assetId: string,
  ttlSeconds?: number,
): Promise<GatewayResult<{ invitation: P2pInvitation; token: string }>> {
  const result = await client.post<ApiInvitationCreated>(
    `/api/assets/${encodeURIComponent(assetId)}/p2p/invitations`,
    accessToken,
    ttlSeconds === undefined ? {} : { ttl_seconds: ttlSeconds },
  );
  return mapResult(result, (value) => ({ invitation: mapInvitation(value.invitation), token: value.token }));
}

export async function claimP2pInvitation(
  client: GatewayClient,
  accessToken: string,
  token: string,
  deviceId: string,
): Promise<GatewayResult<P2pClaim>> {
  const result = await client.post<ApiClaim>("/api/p2p/invitations/claim", accessToken, {
    token,
    device_id: deviceId,
  });
  return mapResult(result, (value) => ({
    invitation: mapInvitation(value.invitation),
    authorization: mapAuthorization(value.authorization),
    descriptor: mapDescriptor(value.descriptor),
  }));
}

export async function listP2pInvitations(
  client: GatewayClient,
  accessToken: string,
): Promise<GatewayResult<P2pInvitation[]>> {
  const result = await client.get<ApiInvitation[]>("/api/p2p/invitations", accessToken);
  return mapResult(result, (values) => values.map(mapInvitation));
}

export async function getP2pAuthorization(
  client: GatewayClient,
  accessToken: string,
  authorizationId: string,
): Promise<GatewayResult<P2pAuthorization>> {
  const result = await client.get<ApiAuthorization>(
    `/api/p2p/authorizations/${encodeURIComponent(authorizationId)}`,
    accessToken,
  );
  return mapResult(result, mapAuthorization);
}

export async function getP2pDeviceEnvelope(
  client: GatewayClient,
  accessToken: string,
  authorizationId: string,
): Promise<GatewayResult<P2pDeviceEnvelope>> {
  const result = await client.get<ApiDeviceEnvelope>(
    `/api/p2p/authorizations/${encodeURIComponent(authorizationId)}/device-envelope`,
    accessToken,
  );
  return mapResult(result, mapDeviceEnvelope);
}
