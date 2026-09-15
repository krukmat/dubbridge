import type { P2pReadyDescriptor } from "../../api/p2p";
import type { P2pSyncSnapshot } from "./SyncState";

export type VerifiedP2pPackageHandle = Readonly<{
  accountScope: string;
  assetId: string;
  publicationId: string;
  lineageId: string;
  manifestDigestSha256: string;
  externalPublicationId: string;
}>;

/**
 * P4 -> P5 handoff. A transport or download success is insufficient: the
 * handle can only be created from the exact account/publication/lineage after
 * the persisted sync state has reached verified READY.
 */
export function createVerifiedPackageHandle(
  descriptor: P2pReadyDescriptor,
  snapshot: P2pSyncSnapshot,
): VerifiedP2pPackageHandle {
  if (
    snapshot.identity.publicationId !== descriptor.publicationId ||
    snapshot.identity.lineageId !== descriptor.lineageId
  ) {
    throw new Error("Verified P2P package identity does not match authoritative descriptor");
  }
  if (
    snapshot.phase !== "READY" ||
    !snapshot.manifestVerified ||
    !snapshot.packageVerified ||
    snapshot.progress.totalFiles <= 0 ||
    snapshot.progress.filesCompleted !== snapshot.progress.totalFiles ||
    snapshot.progress.bytesCompleted !== snapshot.progress.totalBytes
  ) {
    throw new Error("P2P package is not verified READY");
  }
  return {
    accountScope: snapshot.identity.accountScope,
    assetId: descriptor.assetId,
    publicationId: snapshot.identity.publicationId,
    lineageId: snapshot.identity.lineageId,
    manifestDigestSha256: descriptor.manifestDigestSha256,
    externalPublicationId: descriptor.externalPublicationId,
  };
}
