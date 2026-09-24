import type { P2pInboxItem } from "../../api/p2pDashboard";
import type { BadgeTone } from "../../components";
import type { P2pSyncSnapshot } from "../sync/SyncState";

export type ViewerProductState =
  | "pending"
  | "syncing"
  | "sync_error"
  | "available"
  | "expired";

export type ViewerProductAction =
  | "refresh"
  | "sync"
  | "retry_sync"
  | "play"
  | "none";

export type ViewerInboxProjection = Readonly<{
  item: P2pInboxItem;
  state: ViewerProductState;
  action: ViewerProductAction;
  snapshot: P2pSyncSnapshot | null;
}>;

export type InvitesViewState =
  | { kind: "loading" }
  | { kind: "ready"; invitations: ViewerInboxProjection[] }
  | { kind: "empty" }
  | { kind: "error"; message: string };

export const VIEWER_STATE_LABELS: Record<ViewerProductState, string> = {
  pending: "Pending",
  syncing: "Syncing",
  sync_error: "Sync error",
  available: "Available",
  expired: "Expired",
};

export const VIEWER_STATE_TONES: Record<ViewerProductState, BadgeTone> = {
  pending: "info",
  syncing: "info",
  sync_error: "danger",
  available: "success",
  expired: "warning",
};

export function hasExactViewerDescriptor(item: P2pInboxItem): boolean {
  const descriptor = item.descriptor;
  const invitation = item.invitation;
  const authorization = item.authorization;
  return (
    descriptor !== null &&
    authorization.invitationId === invitation.id &&
    authorization.assetId === invitation.assetId &&
    authorization.publicationId === invitation.publicationId &&
    authorization.lineageId === invitation.lineageId &&
    descriptor.assetId === authorization.assetId &&
    descriptor.publicationId === authorization.publicationId &&
    descriptor.lineageId === authorization.lineageId
  );
}

export function shouldReadViewerSyncState(
  item: P2pInboxItem,
  accountScope: string,
): boolean {
  return (
    item.authorization.viewerSubjectId === accountScope &&
    item.authorizationActive &&
    item.invitation.status !== "expired" &&
    item.invitation.status !== "revoked" &&
    hasExactViewerDescriptor(item)
  );
}

export function projectViewerInboxItem(
  item: P2pInboxItem,
  snapshot: P2pSyncSnapshot | null,
  accountScope: string,
): ViewerInboxProjection | null {
  if (item.authorization.viewerSubjectId !== accountScope) return null;
  if (isAccessInactive(item)) {
    return { item, state: "expired", action: "none", snapshot };
  }
  if (!hasExactViewerDescriptor(item)) {
    return { item, state: "pending", action: "refresh", snapshot: null };
  }
  return projectLocalSnapshot(item, snapshot);
}

function isAccessInactive(item: P2pInboxItem): boolean {
  return (
    !item.authorizationActive ||
    item.invitation.status === "expired" ||
    item.invitation.status === "revoked"
  );
}

function projectLocalSnapshot(
  item: P2pInboxItem,
  snapshot: P2pSyncSnapshot | null,
): ViewerInboxProjection {
  if (snapshot === null || snapshot.phase === "IDLE" || snapshot.phase === "CANCELLED") {
    return { item, state: "pending", action: "sync", snapshot };
  }
  if (snapshot.phase === "FAILED") {
    return { item, state: "sync_error", action: "retry_sync", snapshot };
  }
  if (isVerifiedReady(snapshot)) {
    return { item, state: "available", action: "play", snapshot };
  }
  return { item, state: "syncing", action: "none", snapshot };
}

function isVerifiedReady(snapshot: P2pSyncSnapshot): boolean {
  return (
    snapshot.phase === "READY" &&
    snapshot.manifestVerified &&
    snapshot.packageVerified &&
    isProgressComplete(snapshot)
  );
}

export function toInvitesViewState(
  items: readonly { item: P2pInboxItem; snapshot: P2pSyncSnapshot | null }[],
  accountScope: string,
): InvitesViewState {
  const invitations = items
    .map(({ item, snapshot }) => projectViewerInboxItem(item, snapshot, accountScope))
    .filter((value): value is ViewerInboxProjection => value !== null);

  return invitations.length === 0
    ? { kind: "empty" }
    : { kind: "ready", invitations };
}

export function invitesErrorMessage(error: {
  kind: string;
  message?: string;
  status?: number;
}): string {
  if (error.kind === "network") return error.message ?? "Network request failed.";
  if (error.kind === "forbidden") return "You do not have access to P2P invitations.";
  return error.status
    ? `Request failed with status ${error.status}.`
    : "Could not load P2P invitations.";
}

export function claimInvitationErrorMessage(error: {
  kind: string;
  message?: string;
  status?: number;
}): string {
  if (error.kind === "network") {
    return "Network request failed. Try claiming the invitation again.";
  }
  if (error.kind === "forbidden") {
    return "This invitation cannot be claimed by this account.";
  }
  if (error.kind === "http" && error.status === 404) {
    return "Invitation token is invalid or no longer available.";
  }
  if (error.kind === "http" && error.status === 409) {
    return "Invitation has already been claimed.";
  }
  if (error.kind === "http" && error.status === 410) {
    return "Invitation has expired.";
  }
  return error.status
    ? `Invitation claim failed with status ${error.status}.`
    : "Could not claim the invitation.";
}

function isProgressComplete(snapshot: P2pSyncSnapshot): boolean {
  const progress = snapshot.progress;
  return (
    progress.totalFiles > 0 &&
    progress.filesCompleted === progress.totalFiles &&
    progress.bytesCompleted === progress.totalBytes
  );
}
