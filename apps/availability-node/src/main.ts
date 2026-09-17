import {
  loadAvailabilityNodeConfig,
  startAvailabilityNode,
  type AvailabilityNodeRuntime,
} from "./bootstrap.js";

async function main(): Promise<void> {
  const config = loadAvailabilityNodeConfig();
  const runtime = await startAvailabilityNode(config);
  console.info(`Availability Node listening on https://${config.bindHost}:${config.port}`);
  installShutdownHandlers(runtime);
}

function installShutdownHandlers(runtime: AvailabilityNodeRuntime): void {
  let shuttingDown = false;
  const shutdown = async (signal: NodeJS.Signals): Promise<void> => {
    if (shuttingDown) {
      return;
    }
    shuttingDown = true;
    try {
      await runtime.close();
      console.info(`Availability Node stopped after ${signal}`);
      process.exitCode = 0;
    } catch (error) {
      const message = error instanceof Error ? error.message : "unknown shutdown error";
      console.error(`Availability Node shutdown failed: ${message}`);
      process.exitCode = 1;
    }
  };

  process.once("SIGINT", () => {
    void shutdown("SIGINT");
  });
  process.once("SIGTERM", () => {
    void shutdown("SIGTERM");
  });
}

main().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : "unknown startup error";
  console.error(`Availability Node failed to start: ${message}`);
  process.exitCode = 1;
});
