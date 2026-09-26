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
  const identity = authIdentity(auth.userId, auth.sessionRef);
  const identityRef = useRef(identity);
  identityRef.current = identity;

  const load = useCallback(async () => {
    const requestIdentity = authIdentity(auth.userId, auth.sessionRef);
    const isCurrent = () => identityRef.current === requestIdentity;
    await loadInvites({
      auth, dashboard, syncController, isCurrent, setViewState,
    });
  }, [auth, dashboard, syncController]);

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

async function loadInvites({
  auth,
  dashboard,
  syncController,
  isCurrent,
  setViewState,
}: {
  auth: ReturnType<typeof useAuth>;
  dashboard: P2PDashboardService;
  syncController: ReturnType<typeof useP2PSyncController>;
  isCurrent: () => boolean;
  setViewState: React.Dispatch<React.SetStateAction<InvitesViewState>>;
}) {
  if (!auth.sessionRef || !auth.userId) {
    commitState(isCurrent, setViewState, { kind: "error", message: "Session unavailable." });
    return;
  }
  const result = await dashboard.listInbox(auth.sessionRef);
  if (!result.ok) {
    await handleInboxFailure(result.error, auth, isCurrent, setViewState);
    return;
  }
  if (!isCurrent()) return;
  await auth.onSessionRotation(result.value.sessionRotation);
  if (!isCurrent()) return;
  await projectInbox(result.value.data, auth.userId, syncController, isCurrent, setViewState);
}

async function handleInboxFailure(
  error: { kind: string; message?: string; status?: number },
  auth: ReturnType<typeof useAuth>,
  isCurrent: () => boolean,
  setViewState: React.Dispatch<React.SetStateAction<InvitesViewState>>,
) {
  if (!isCurrent()) return;
  if (error.kind === "session_expired") {
    await auth.logout();
    return;
  }
  setViewState({ kind: "error", message: invitesErrorMessage(error) });
}

async function projectInbox(
  items: P2pInboxItem[],
  accountScope: string,
  syncController: ReturnType<typeof useP2PSyncController>,
  isCurrent: () => boolean,
  setViewState: React.Dispatch<React.SetStateAction<InvitesViewState>>,
) {
  try {
    const joined = await Promise.all(
      items.map((item) => readLocalSnapshot(item, accountScope, syncController)),
    );
    commitState(isCurrent, setViewState, toInvitesViewState(joined, accountScope));
  } catch {
    commitState(isCurrent, setViewState, {
      kind: "error",
      message: "Could not read local P2P availability.",
    });
  }
}

function commitState(
  isCurrent: () => boolean,
  setViewState: React.Dispatch<React.SetStateAction<InvitesViewState>>,
  state: InvitesViewState,
) {
  if (isCurrent()) setViewState(state);
}

function authIdentity(userId: string | null | undefined, sessionRef: string | null) {
  return `${userId ?? ""}:${sessionRef ?? ""}`;
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
