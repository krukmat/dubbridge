import { useCallback, useEffect, useMemo, useState } from "react";

import { createGatewayClient } from "../../api/client";
import { useAuth } from "../../auth/AuthProvider";
import { P2PDashboardService } from "../P2PDashboardService";
import {
  myContentErrorMessage,
  toMyContentViewState,
  type MyContentViewState,
} from "./MyContentModel";

export function useMyContentState(gatewayBaseUrl: string) {
  const auth = useAuth();
  const dashboard = useMemo(
    () => new P2PDashboardService(createGatewayClient({ gatewayBaseUrl })),
    [gatewayBaseUrl],
  );
  const [viewState, setViewState] = useState<MyContentViewState>({ kind: "loading" });

  const load = useCallback(async () => {
    if (!auth.sessionRef) {
      setViewState({ kind: "error", message: "Session unavailable." });
      return;
    }
    const result = await dashboard.listOwnerContent(auth.sessionRef);
    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setViewState(toMyContentViewState(result.value.data));
      return;
    }
    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }
    setViewState({ kind: "error", message: myContentErrorMessage(result.error) });
  }, [auth.logout, auth.onSessionRotation, auth.sessionRef, dashboard]);

  useEffect(() => {
    void load();
  }, [load]);

  const retry = useCallback(() => {
    setViewState({ kind: "loading" });
    void load();
  }, [load]);

  return { viewState, retry, refresh: load };
}
