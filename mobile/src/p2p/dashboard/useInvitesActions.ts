import { useCallback, useMemo, useRef, useState } from "react";

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

  const updateClaimToken = useCallback((value: string) => {
    setClaimToken(value);
    setClaimError(null);
  }, []);

  const claim = useCallback(async () => {
    const token = claimToken.trim();
    const accessToken = auth.sessionRef;
    if (!accessToken || token.length === 0 || submitting.current) return;
    submitting.current = true;
    setIsClaiming(true);
    setClaimError(null);
    try {
      await runClaim(audience, accessToken, token, auth, refreshInbox, setClaimToken, setClaimError);
    } finally {
      submitting.current = false;
      setIsClaiming(false);
    }
  }, [audience, auth, claimToken, refreshInbox]);

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
) {
  try {
    const result = await audience.claimInvitation(accessToken, token);
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
    setClaimToken("");
    await refreshInbox();
  } catch {
    setClaimError("Could not claim the invitation.");
  }
}

function useSyncAction(refreshInbox: () => Promise<void>) {
  const auth = useAuth();
  const syncController = useP2PSyncController();
  const inFlight = useRef(new Set<string>());
  const [busyInvites, setBusyInvites] = useState<ReadonlySet<string>>(new Set());
  const [syncErrors, setSyncErrors] = useState<Readonly<Record<string, string>>>({});

  const syncInvitation = useCallback(async (projection: ViewerInboxProjection) => {
    const descriptor = projection.item.descriptor;
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
      setSyncErrors((current) => ({
        ...current,
        [projection.item.invitation.id]: "Could not sync this invitation.",
      }));
    } finally {
      await refreshInbox();
      inFlight.current.delete(key);
      updateBusy(setBusyInvites, projection.item.invitation.id, false);
    }
  }, [auth.userId, refreshInbox, syncController]);

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

  const playInvitation = useCallback(async (projection: ViewerInboxProjection) => {
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
      const handle = await syncController.getVerifiedPackageHandle(descriptor, accountScope);
      const result = await controller.start(accessToken, projection.item.authorization.id, handle);
      if (!result.ok) {
        if (result.error.kind === "session_expired") {
          await auth.logout();
          return;
        }
        setPlayErrors((current) => ({
          ...current,
          [projection.item.invitation.id]: "Playback authorization is no longer available.",
        }));
        await refreshInbox();
        return;
      }
      await auth.onSessionRotation(result.value.sessionRotation);
      setPlaybackSession({ invitationId: projection.item.invitation.id, session: result.value.data });
    } catch {
      setPlayErrors((current) => ({
        ...current,
        [projection.item.invitation.id]: "Could not start verified playback.",
      }));
      await refreshInbox();
    } finally {
      inFlight.current.delete(key);
      updateBusy(setBusyPlayInvites, projection.item.invitation.id, false);
    }
  }, [auth, controller, refreshInbox, syncController]);

  return { playInvitation, busyPlayInvites, playErrors, playbackSession, playbackController: controller };
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
