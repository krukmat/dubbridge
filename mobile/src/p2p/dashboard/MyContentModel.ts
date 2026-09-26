import type { P2pOwnerContent, P2pOwnerContentState } from "../../api/p2pDashboard";
import type { BadgeTone } from "../../components";

export type MyContentViewState =
  | { kind: "loading" }
  | { kind: "ready"; content: P2pOwnerContent[] }
  | { kind: "empty" }
  | { kind: "error"; message: string };

export const MY_CONTENT_STATE_LABELS: Record<P2pOwnerContentState, string> = {
  processing: "Processing",
  ready: "Ready",
  failed: "Failed",
};

export const MY_CONTENT_STATE_TONES: Record<P2pOwnerContentState, BadgeTone> = {
  processing: "info",
  ready: "success",
  failed: "danger",
};

export function canCreateP2pInvite(content: P2pOwnerContent): boolean {
  const descriptor = content.descriptor;
  return (
    content.state === "ready" &&
    descriptor !== null &&
    descriptor.assetId === content.assetId &&
    descriptor.publicationId === content.publicationId &&
    descriptor.lineageId === content.lineageId
  );
}

export function toMyContentViewState(content: P2pOwnerContent[]): MyContentViewState {
  return content.length === 0 ? { kind: "empty" } : { kind: "ready", content };
}

export function myContentErrorMessage(error: {
  kind: string;
  message?: string;
  status?: number;
}): string {
  if (error.kind === "network") return error.message ?? "Network request failed.";
  if (error.kind === "forbidden") return "You do not have access to P2P content.";
  return error.status ? `Request failed with status ${error.status}.` : "Could not load P2P content.";
}


export function p2pInviteErrorMessage(error: {
  kind: string;
  message?: string;
  status?: number;
}): string {
  if (error.kind === "network") return "Network request failed. Try creating the invite again.";
  if (error.kind === "forbidden") return "Invite creation is not allowed for this content.";
  if (error.kind === "http" && (error.status === 404 || error.status === 409)) {
    return "This content is no longer eligible for an invite.";
  }
  return error.status
    ? `Invite creation failed with status ${error.status}.`
    : "Could not create the invite.";
}

export function shouldRefreshAfterInviteError(error: {
  kind: string;
  status?: number;
}): boolean {
  return (
    error.kind === "forbidden" ||
    (error.kind === "http" && (error.status === 404 || error.status === 409))
  );
}
