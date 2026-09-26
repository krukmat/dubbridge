import type { GatewayClient, GatewayResult } from "./client";
import type {
  P2pAuthorization,
  P2pInvitation,
  P2pInvitationStatus,
  P2pReadyDescriptor,
} from "./p2p";

export type P2pOwnerContentState = "processing" | "ready" | "failed";

export type P2pOwnerContent = Readonly<{
  assetId: string;
  title: string;
  publicationId: string;
  lineageId: string;
  state: P2pOwnerContentState;
  descriptor: P2pReadyDescriptor | null;
}>;

export type P2pInboxItem = Readonly<{
  invitation: P2pInvitation;
  authorization: P2pAuthorization;
  authorizationActive: boolean;
  descriptor: P2pReadyDescriptor | null;
}>;

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

type ApiOwnerContent = {
  asset_id: string;
  title: string;
  publication_id: string;
  lineage_id: string;
  state: P2pOwnerContentState;
  descriptor: ApiReadyDescriptor | null;
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

type ApiInboxItem = {
  invitation: ApiInvitation;
  authorization: ApiAuthorization;
  authorization_active: boolean;
  descriptor: ApiReadyDescriptor | null;
};

export async function listP2pOwnerContent(
  client: GatewayClient,
  accessToken: string,
): Promise<GatewayResult<P2pOwnerContent[]>> {
  const result = await client.get<ApiOwnerContent[]>("/api/p2p/content", accessToken);
  return mapResult(result, (items) => items.map(mapOwnerContent));
}

export async function listP2pInbox(
  client: GatewayClient,
  accessToken: string,
): Promise<GatewayResult<P2pInboxItem[]>> {
  const result = await client.get<ApiInboxItem[]>("/api/p2p/inbox", accessToken);
  return mapResult(result, (items) => items.map(mapInboxItem));
}

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

function mapOwnerContent(value: ApiOwnerContent): P2pOwnerContent {
  return {
    assetId: value.asset_id,
    title: value.title,
    publicationId: value.publication_id,
    lineageId: value.lineage_id,
    state: value.state,
    descriptor: value.descriptor ? mapDescriptor(value.descriptor) : null,
  };
}

function mapInboxItem(value: ApiInboxItem): P2pInboxItem {
  return {
    invitation: mapInvitation(value.invitation),
    authorization: mapAuthorization(value.authorization),
    authorizationActive: value.authorization_active,
    descriptor: value.descriptor ? mapDescriptor(value.descriptor) : null,
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
