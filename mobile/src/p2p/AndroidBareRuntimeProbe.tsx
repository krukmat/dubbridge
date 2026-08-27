import { useEffect } from "react";
import { Platform } from "react-native";

import { useP2PService } from "./P2PProvider";

export function AndroidBareRuntimeProbe({ enabled }: { enabled: boolean }) {
  const service = useP2PService();

  useEffect(() => {
    if (!enabled || Platform.OS !== "android") return;

    let released = false;

    void (async () => {
      try {
        await service.initialize();
        if (released) return;
        const result = await service.ping();
        if (released) return;
        console.warn(`[Bare runtime probe] ping=${result}`);
      } catch (error) {
        const message = error instanceof Error ? error.message : "unknown Bare runtime failure";
        console.error(`[Bare runtime probe] ${message}`);
      } finally {
        if (!released) {
          try {
            await service.shutdown();
            console.warn("[Bare runtime probe] shutdown=complete");
          } catch (error) {
            const message = error instanceof Error ? error.message : "unknown Bare runtime shutdown failure";
            console.error(`[Bare runtime probe] ${message}`);
          }
        }
      }
    })();

    return () => {
      released = true;
      void service.shutdown().catch((error: unknown) => {
        const message = error instanceof Error ? error.message : "unknown Bare runtime shutdown failure";
        console.error(`[Bare runtime probe] ${message}`);
      });
    };
  }, [enabled, service]);

  return null;
}
