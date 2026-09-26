import { useEffect } from "react";
import { Platform } from "react-native";

import { useP2PService } from "../P2PProvider";

export type P2PDevelopmentHarnessFailureCode =
  | "INITIALIZE_FAILED"
  | "PING_FAILED"
  | "SHUTDOWN_FAILED";

function reportFailure(code: P2PDevelopmentHarnessFailureCode): void {
  // The diagnostic is intentionally redacted: remote/runtime error payloads do
  // not become app logs through the development harness.
  console.error(`[P2P development harness] ${code}`);
}

/**
 * Explicit Android-only diagnostic host. It exercises the product runtime seam
 * without making runtime startup part of normal application mounting.
 */
export function P2PDevelopmentHarness({ enabled }: { enabled: boolean }) {
  const service = useP2PService();

  useEffect(() => {
    if (Platform.OS !== "android" || !enabled) return;

    let released = false;

    void (async () => {
      let failureCode: P2PDevelopmentHarnessFailureCode = "INITIALIZE_FAILED";

      try {
        await service.initialize();
        if (released) return;

        failureCode = "PING_FAILED";
        const result = await service.ping();
        if (!released) console.warn(`[P2P development harness] ping=${result}`);
      } catch {
        reportFailure(failureCode);
      } finally {
        if (!released) {
          try {
            await service.shutdown();
            console.warn("[P2P development harness] shutdown=complete");
          } catch {
            reportFailure("SHUTDOWN_FAILED");
          }
        }
      }
    })();

    return () => {
      released = true;
      void service.shutdown().catch(() => reportFailure("SHUTDOWN_FAILED"));
    };
  }, [enabled, service]);

  return null;
}
