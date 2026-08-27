import { useEffect } from "react";
import { Platform } from "react-native";

import { BareBridge } from "./bare-bridge";

export function AndroidBareRuntimeProbe({ enabled }: { enabled: boolean }) {
  useEffect(() => {
    if (!enabled || Platform.OS !== "android") return;

    const bridge = new BareBridge();
    let released = false;

    void (async () => {
      try {
        await bridge.initialize();
        const result = await bridge.ping();
        console.warn(`[Bare runtime probe] ping=${result}`);
      } catch (error) {
        const message = error instanceof Error ? error.message : "unknown Bare runtime failure";
        console.error(`[Bare runtime probe] ${message}`);
      } finally {
        if (!released) {
          try {
            await bridge.shutdown();
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
      void bridge.shutdown();
    };
  }, [enabled]);

  return null;
}
