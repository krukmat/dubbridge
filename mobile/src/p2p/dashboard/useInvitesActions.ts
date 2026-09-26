import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { createGatewayClient } from "../../api/client";
import { useAuth } from "../../auth/AuthProvider";
import { P2PAudienceService } from "../P2PAudienceService";
import { useP2PService, useP2PSyncController } from "../P2PProvider";
import {
  P2PPlaybackController,
  type P2PPlaybackSession,
} from "../playback/P2PPlaybackController";
import {
  canStartViewerPlayback,
  canStartViewerSync,
  claimInvitationErrorMessage,
  type ViewerInboxProjection,
} from "./InvitesModel";

export function useInvitesActions(gatewayBaseUrl: string, refreshInbox: () => Promise<void>) {
  return {
    ...useClaimAction(gatewayBaseUrl, refreshInbox),
    ...useSyncAction(refreshInbox),
    ...usePlaybackAction(gatewayBaseUrl, refreshInbox),
  };
}

function useClaimAction(gatewayBaseUrl: string, refreshInbox: () => Promise<void>) {
  const auth = useAuth();
  const audience = useMemo(
    () => new P2PAudienceService(createGatewayClient({ gatewayBaseUrl })), [gatewayBaseUrl],
  );
  const submitting = useRef(false);
  const [claimToken, setClaimToken] = useState("");
  const [claimError, setClaimError] = useState<string | null>(null);
  const [isClaiming, setIsClaiming] = useState(false);
  const identity = `${auth.userId ?? ""}:${auth.sessionRef ?? ""}`;
  const identityRef = useRef(identity);
  identityRef.current = identity;
  useEffect(() => {
    submitting.current = false;
    setClaimToken("");
    setClaimError(null);
    setIsClaiming(false);
  }, [identity]);

  const updateClaimToken = useCallback((value: string) => {
    setClaimToken(value);
    setClaimError(null);
  }, []);

  const claim = useCallback(async () => {
    const token = claimToken.trim();
    const accessToken = auth.sessionRef;
    const requestIdentity = identity;
    if (!accessToken || token.length === 0 || submitting.current) return;
    submitting.current = true;
    setIsClaiming(true);
    setClaimError(null);
    try {
      await runClaim(
        audience, accessToken, token, auth, refreshInbox, setClaimToken, setClaimError,
        () => identityRef.current === requestIdentity,
      );
    } finally {
      if (identityRef.current === requestIdentity) {
        submitting.current = false;
        setIsClaiming(false);
      }
    }
  }, [audience, auth, claimToken, identity, refreshInbox]);

  return {
    claimToken, updateClaimToken, claim, claimError, isClaiming,
    canClaim: claimToken.trim().length > 0 && auth.sessionRef !== null && !isClaiming,
  };
}

async function runClaim(
  audience: P2PAudienceService,
  accessToken: string,
  token: string,
  auth: ReturnType<typeof useAuth>,
  refreshInbox: () => Promise<void>,
  setClaimToken: (value: string) => void,
  setClaimError: (value: string | null) => void,
  isCurrent: () => boolean,
) {
  try {
    const result = await audience.claimInvitation(accessToken, token);
    if (!isCurrent()) return;
    if (!result.ok) {
      if (result.error.kind === "session_expired") {
        setClaimToken("");
        await auth.logout();
        return;
      }
      setClaimError(claimInvitationErrorMessage(result.error));
      return;
    }
    await auth.onSessionRotation(result.value.sessionRotation);
    if (!isCurrent()) return;
    setClaimToken("");
    await refreshInbox();
  } catch {
    if (isCurrent()) setClaimError("Could not claim the invitation.");
  }
}

function useSyncAction(refreshInbox: () => Promise<void>) {
  const auth = useAuth();
  const syncController = useP2PSyncController();
  const inFlight = useRef(new Set<string>());
  const [busyInvites, setBusyInvites] = useState<ReadonlySet<string>>(new Set());
  const [syncErrors, setSyncErrors] = useState<Readonly<Record<string, string>>>({});
  const identity = `${auth.userId ?? ""}:${auth.sessionRef ?? ""}`;
  const identityRef = useRef(identity);
  identityRef.current = identity;
  useEffect(() => {
    inFlight.current.clear();
    setBusyInvites(new Set());
    setSyncErrors({});
  }, [identity]);

  const syncInvitation = useCallback(async (projection: ViewerInboxProjection) => {
    const descriptor = projection.item.descriptor;
    const requestIdentity = identity;
    const accountScope = auth.userId ?? null;
    if (!descriptor || !accountScope || !canStartViewerSync(projection, accountScope)) return;
    const key = `${descriptor.publicationId}/${descriptor.lineageId}`;
    if (inFlight.current.has(key)) return;

    inFlight.current.add(key);
    updateBusy(setBusyInvites, projection.item.invitation.id, true);
    setSyncErrors((current) => {
      if (!(projection.item.invitation.id in current)) return current;
      const next = { ...current };
      delete next[projection.item.invitation.id];
      return next;
    });
    try {
      await syncController.startSync(descriptor, accountScope);
    } catch {
      if (identityRef.current !== requestIdentity) return;
      setSyncErrors((current) => ({
        ...current,
        [projection.item.invitation.id]: "Could not sync this invitation.",
      }));
    } finally {
      if (identityRef.current === requestIdentity) {
        await refreshInbox();
        inFlight.current.delete(key);
        updateBusy(setBusyInvites, projection.item.invitation.id, false);
      }
    }
  }, [auth.userId, identity, refreshInbox, syncController]);

  return { syncInvitation, busyInvites, syncErrors };
}

function usePlaybackAction(gatewayBaseUrl: string, refreshInbox: () => Promise<void>) {
  const auth = useAuth();
  const service = useP2PService();
  const syncController = useP2PSyncController();
  const controller = useMemo(
    () => new P2PPlaybackController(
      new P2PAudienceService(createGatewayClient({ gatewayBaseUrl })), service,
    ),
    [gatewayBaseUrl, service],
  );
  const inFlight = useRef(new Set<string>());
  const [busyPlayInvites, setBusyPlayInvites] = useState<ReadonlySet<string>>(new Set());
  const [playErrors, setPlayErrors] = useState<Readonly<Record<string, string>>>({});
  const [playbackSession, setPlaybackSession] = useState<
    Readonly<{ invitationId: string; session: P2PPlaybackSession }> | null
  >(null);
  const identity = `${auth.userId ?? ""}:${auth.sessionRef ?? ""}`;
  const identityRef = useRef(identity);
  identityRef.current = identity;

  useEffect(() => {
    inFlight.current.clear();
    setBusyPlayInvites(new Set());
    setPlayErrors({});
    setPlaybackSession(null);
  }, [identity]);

  const playInvitation = useCallback(async (projection: ViewerInboxProjection) => {
    await runPlayback({
      projection, auth, controller, syncController, refreshInbox,
      identity, identityRef, inFlight, setBusyPlayInvites, setPlayErrors, setPlaybackSession,
    });
  }, [auth, controller, identity, refreshInbox, syncController]);

  return { playInvitation, busyPlayInvites, playErrors, playbackSession, playbackController: controller };
}

async function runPlayback({
  projection, auth, controller, syncController, refreshInbox, identity, identityRef,
  inFlight, setBusyPlayInvites, setPlayErrors, setPlaybackSession,
}: {
  projection: ViewerInboxProjection;
  auth: ReturnType<typeof useAuth>;
  controller: P2PPlaybackController;
  syncController: ReturnType<typeof useP2PSyncController>;
  refreshInbox: () => Promise<void>;
  identity: string;
  identityRef: React.MutableRefObject<string>;
  inFlight: React.MutableRefObject<Set<string>>;
  setBusyPlayInvites: React.Dispatch<React.SetStateAction<ReadonlySet<string>>>;
  setPlayErrors: React.Dispatch<React.SetStateAction<Readonly<Record<string, string>>>>;
  setPlaybackSession: React.Dispatch<React.SetStateAction<
    Readonly<{ invitationId: string; session: P2PPlaybackSession }> | null
  >>;
}) {
  const descriptor = projection.item.descriptor;
  const accessToken = auth.sessionRef;
  const accountScope = auth.userId ?? null;
  if (!descriptor || !accessToken || !accountScope) return;
  if (!canStartViewerPlayback(projection, accountScope)) return;
  const key = `${descriptor.publicationId}/${descriptor.lineageId}`;
  if (inFlight.current.has(key)) return;

  inFlight.current.add(key);
  updateBusy(setBusyPlayInvites, projection.item.invitation.id, true);
  setPlayErrors({});
  try {
    await startCurrentPlayback({
      projection, descriptor, accessToken, accountScope, auth, controller, syncController,
      refreshInbox, identity, identityRef, setPlayErrors, setPlaybackSession,
    });
  } finally {
    if (identityRef.current === identity) {
      inFlight.current.delete(key);
      updateBusy(setBusyPlayInvites, projection.item.invitation.id, false);
    }
  }
}

async function startCurrentPlayback({
  projection, descriptor, accessToken, accountScope, auth, controller, syncController,
  refreshInbox, identity, identityRef, setPlayErrors, setPlaybackSession,
}: {
  projection: ViewerInboxProjection;
  descriptor: NonNullable<ViewerInboxProjection["item"]["descriptor"]>;
  accessToken: string;
  accountScope: string;
  auth: ReturnType<typeof useAuth>;
  controller: P2PPlaybackController;
  syncController: ReturnType<typeof useP2PSyncController>;
  refreshInbox: () => Promise<void>;
  identity: string;
  identityRef: React.MutableRefObject<string>;
  setPlayErrors: React.Dispatch<React.SetStateAction<Readonly<Record<string, string>>>>;
  setPlaybackSession: React.Dispatch<React.SetStateAction<
    Readonly<{ invitationId: string; session: P2PPlaybackSession }> | null
  >>;
}) {
  const current = () => identityRef.current === identity;
  try {
    const handle = await syncController.getVerifiedPackageHandle(descriptor, accountScope);
    if (!current()) return;
    const result = await controller.start(accessToken, projection.item.authorization.id, handle);
    if (!current()) return;
    if (!result.ok) {
      await handlePlaybackFailure(result.error.kind, projection, auth, refreshInbox, setPlayErrors);
      return;
    }
    await auth.onSessionRotation(result.value.sessionRotation);
    if (current()) {
      setPlaybackSession({ invitationId: projection.item.invitation.id, session: result.value.data });
    }
  } catch {
    if (!current()) return;
    setPlayErrors((value) => ({
      ...value,
      [projection.item.invitation.id]: "Could not start verified playback.",
    }));
    await refreshInbox();
  }
}

async function handlePlaybackFailure(
  kind: string,
  projection: ViewerInboxProjection,
  auth: ReturnType<typeof useAuth>,
  refreshInbox: () => Promise<void>,
  setPlayErrors: React.Dispatch<React.SetStateAction<Readonly<Record<string, string>>>>,
) {
  if (kind === "session_expired") {
    await auth.logout();
    return;
  }
  setPlayErrors((value) => ({
    ...value,
    [projection.item.invitation.id]: "Playback authorization is no longer available.",
  }));
  await refreshInbox();
}

function updateBusy(
  setter: React.Dispatch<React.SetStateAction<ReadonlySet<string>>>,
  invitationId: string,
  busy: boolean,
) {
  setter((current) => {
    const next = new Set(current);
    if (busy) next.add(invitationId);
    else next.delete(invitationId);
    return next;
  });
}
