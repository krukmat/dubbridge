import { useEffect } from "react";

import { useAuth } from "../../auth/AuthProvider";
import { ExpoP2pSyncCache } from "../sync/SyncCache";
import { buildP2PVisualFixtureSnapshots } from "./P2PVisualFixtures";

/**
 * Screenshot-only fixture bridge. It writes non-secret P4 lifecycle metadata
 * through the real device cache so InvitesModel still derives product states
 * from its normal backend + local-state inputs.
 */
export function P2PVisualFixtureHarness({ enabled }: { enabled: boolean }) {
  const { userId } = useAuth();

  useEffect(() => {
    if (!enabled || !userId) return;

    let active = true;
    const cache = new ExpoP2pSyncCache();

    void (async () => {
      try {
        for (const snapshot of buildP2PVisualFixtureSnapshots(userId)) {
          if (!active) return;
          await cache.writeSnapshot(snapshot);
        }
      } catch {
        if (active) console.error("[P2P visual fixtures] SEED_FAILED");
      }
    })();

    return () => {
      active = false;
    };
  }, [enabled, userId]);

  return null;
}
