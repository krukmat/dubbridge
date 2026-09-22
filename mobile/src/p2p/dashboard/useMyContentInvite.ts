import * as Clipboard from "expo-clipboard";
import { useCallback, useMemo, useState } from "react";

import { createGatewayClient } from "../../api/client";
import type { P2pOwnerContent } from "../../api/p2pDashboard";
import { useAuth } from "../../auth/AuthProvider";
import { P2PAudienceService } from "../P2PAudienceService";
import {
  canCreateP2pInvite,
  p2pInviteErrorMessage,
  shouldRefreshAfterInviteError,
} from "./MyContentModel";

export type MyContentInviteState =
  | { kind: "idle" }
  | { kind: "creating"; assetId: string }
  | {
      kind: "created";
      assetId: string;
      invitationId: string;
      token: string;
      copied: boolean;
      copyError: string | null;
    }
  | { kind: "error"; assetId: string; message: string };

export function useMyContentInvite(
  gatewayBaseUrl: string,
  refreshOwnerContent: () => Promise<void>,
) {
  const auth = useAuth();
  const audience = useMemo(
    () => new P2PAudienceService(createGatewayClient({ gatewayBaseUrl })),
    [gatewayBaseUrl],
  );
  const [inviteState, setInviteState] = useState<MyContentInviteState>({ kind: "idle" });

  const createInvite = useCallback(
    async (content: P2pOwnerContent) => {
      if (!canCreateP2pInvite(content)) {
        setInviteState({
          kind: "error",
          assetId: content.assetId,
          message: "This content is not eligible for an invite.",
        });
        return;
      }
      if (!auth.sessionRef) {
        setInviteState({
          kind: "error",
          assetId: content.assetId,
          message: "Session unavailable.",
        });
        return;
      }

      setInviteState({ kind: "creating", assetId: content.assetId });
      const result = await audience.createInvitation(auth.sessionRef, content.assetId);
      if (result.ok) {
        await auth.onSessionRotation(result.value.sessionRotation);
        setInviteState({
          kind: "created",
          assetId: content.assetId,
          invitationId: result.value.data.invitation.id,
          token: result.value.data.token,
          copied: false,
          copyError: null,
        });
        return;
      }

      if (result.error.kind === "session_expired") {
        setInviteState({ kind: "idle" });
        await auth.logout();
        return;
      }

      if (shouldRefreshAfterInviteError(result.error)) {
        await refreshOwnerContent();
      }
      setInviteState({
        kind: "error",
        assetId: content.assetId,
        message: p2pInviteErrorMessage(result.error),
      });
    },
    [
      audience,
      auth.logout,
      auth.onSessionRotation,
      auth.sessionRef,
      refreshOwnerContent,
    ],
  );

  const copyInvite = useCallback(async () => {
    if (inviteState.kind !== "created") return;
    const { invitationId, token } = inviteState;
    try {
      await Clipboard.setStringAsync(token);
      setInviteState((current) =>
        current.kind === "created" && current.invitationId === invitationId
          ? { ...current, copied: true, copyError: null }
          : current,
      );
    } catch {
      setInviteState((current) =>
        current.kind === "created" && current.invitationId === invitationId
          ? { ...current, copied: false, copyError: "Could not copy the invite token." }
          : current,
      );
    }
  }, [inviteState]);

  const dismissInvite = useCallback(() => {
    setInviteState({ kind: "idle" });
  }, []);

  return { inviteState, createInvite, copyInvite, dismissInvite };
}
