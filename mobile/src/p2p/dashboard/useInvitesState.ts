import { useCallback, useEffect, useMemo, useRef, useState } from "react";

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

export function useInvitesState(gatewayBaseUrl: string) {
  const auth = useAuth();
  const syncController = useP2PSyncController();
  const dashboard = useMemo(
    () => new P2PDashboardService(createGatewayClient({ gatewayBaseUrl })),
    [gatewayBaseUrl],
  );
  const [viewState, setViewState] = useState<InvitesViewState>({ kind: "loading" });
  const identity = `${auth.userId ?? ""}:${auth.sessionRef ?? ""}`;
  const identityRef = useRef(identity);
  identityRef.current = identity;

  const load = useCallback(async () => {
    const requestIdentity = `${auth.userId ?? ""}:${auth.sessionRef ?? ""}`;
    const isCurrent = () => identityRef.current === requestIdentity;
    if (!auth.sessionRef || !auth.userId) {
      if (isCurrent()) setViewState({ kind: "error", message: "Session unavailable." });
      return;
    }

    const result = await dashboard.listInbox(auth.sessionRef);
    if (!result.ok) {
      if (result.error.kind === "session_expired") {
        if (isCurrent()) await auth.logout();
        return;
      }
      if (isCurrent()) setViewState({ kind: "error", message: invitesErrorMessage(result.error) });
      return;
    }

    if (!isCurrent()) return;
    await auth.onSessionRotation(result.value.sessionRotation);
    if (!isCurrent()) return;
    try {
      const joined = await Promise.all(
        result.value.data.map((item) => readLocalSnapshot(item, auth.userId!, syncController)),
      );
      if (isCurrent()) setViewState(toInvitesViewState(joined, auth.userId));
    } catch {
      if (isCurrent()) {
        setViewState({
          kind: "error",
          message: "Could not read local P2P availability.",
        });
      }
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
    setViewState({ kind: "loading" });
    void load();
  }, [identity, load]);

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
): Promise<Readonly<{ item: P2pInboxItem; snapshot: P2pSyncSnapshot | null }>> {
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
