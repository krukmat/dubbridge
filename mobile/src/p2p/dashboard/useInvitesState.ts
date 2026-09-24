import { useCallback, useEffect, useMemo, useState } from "react";

import { createGatewayClient } from "../../api/client";
import type { P2pInboxItem } from "../../api/p2pDashboard";
import { useAuth } from "../../auth/AuthProvider";
import { P2PDashboardService } from "../P2PDashboardService";
import { useP2PSyncController } from "../P2PProvider";
import type { P2pSyncSnapshot } from "../sync/SyncState";
import {
  invitesErrorMessage,
  shouldReadViewerSyncState,
  toInvitesViewState,
  type InvitesViewState,
} from "./InvitesModel";

type InboxWithSnapshot = Readonly<{
  item: P2pInboxItem;
  snapshot: P2pSyncSnapshot | null;
}>;

export function useInvitesState(gatewayBaseUrl: string) {
  const auth = useAuth();
  const syncController = useP2PSyncController();
  const dashboard = useMemo(
    () => new P2PDashboardService(createGatewayClient({ gatewayBaseUrl })),
    [gatewayBaseUrl],
  );
  const [viewState, setViewState] = useState<InvitesViewState>({ kind: "loading" });

  const load = useCallback(async () => {
    if (!auth.sessionRef || !auth.userId) {
      setViewState({ kind: "error", message: "Session unavailable." });
      return;
    }

    const result = await dashboard.listInbox(auth.sessionRef);
    if (!result.ok) {
      if (result.error.kind === "session_expired") {
        await auth.logout();
        return;
      }
      setViewState({ kind: "error", message: invitesErrorMessage(result.error) });
      return;
    }

    await auth.onSessionRotation(result.value.sessionRotation);
    try {
      const joined = await Promise.all(
        result.value.data.map((item) => readLocalSnapshot(item, auth.userId!, syncController)),
      );
      setViewState(toInvitesViewState(joined, auth.userId));
    } catch {
      setViewState({
        kind: "error",
        message: "Could not read local P2P availability.",
      });
    }
  }, [
    auth.logout,
    auth.onSessionRotation,
    auth.sessionRef,
    auth.userId,
    dashboard,
    syncController,
  ]);

  useEffect(() => {
    void load();
  }, [load]);

  const retry = useCallback(() => {
    setViewState({ kind: "loading" });
    void load();
  }, [load]);

  return { viewState, retry, refresh: load };
}

async function readLocalSnapshot(
  item: P2pInboxItem,
  accountScope: string,
  syncController: ReturnType<typeof useP2PSyncController>,
): Promise<InboxWithSnapshot> {
  if (!shouldReadViewerSyncState(item, accountScope) || item.descriptor === null) {
    return { item, snapshot: null };
  }

  const snapshot = await syncController.getSyncState({
    accountScope,
    publicationId: item.descriptor.publicationId,
    lineageId: item.descriptor.lineageId,
  });
  return { item, snapshot };
}
