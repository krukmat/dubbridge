#!/usr/bin/env node
// Minimal mock gateway for E2E screenshot suite.
// Provides only the endpoints the mobile bootstrap flow needs:
//   GET  /health/ready         — seed CLI health-check gate
//   POST /auth/mobile/session  — handoff code → session_ref exchange
//
// The handoff store is in-memory. Use /e2e/issue-handoff to pre-seed a code
// so that mint-handoff-code.mjs (or Maestro) can redeem it.

import http from "node:http";
import { randomUUID } from "node:crypto";

const DEFAULT_HOST = "127.0.0.1";
const DEFAULT_PORT = 8081;

export const SEED_ASSETS = [
  {
    id: "asset-seed-1",
    title: "Demo Reel 2026",
    uploader_id: "e2e-user",
    status: "finalized",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "asset-seed-2",
    title: "Interview Selects",
    uploader_id: "e2e-user",
    status: "finalized",
    created_at: "2026-01-02T00:00:00Z",
    updated_at: "2026-01-02T00:00:00Z",
  },
];

function sendJson(res, status, body) {
  res.writeHead(status, { "content-type": "application/json; charset=utf-8" });
  res.end(JSON.stringify(body));
}

async function readBody(req) {
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  return Buffer.concat(chunks).toString("utf8");
}

function hasSession(req) {
  const session = req.headers["x-dubbridge-session"];
  return typeof session === "string" && session.trim().length > 0;
}

function getAssetSeedMode(req, sessionModes) {
  const session = req.headers["x-dubbridge-session"];
  if (typeof session !== "string") {
    return "default";
  }

  return sessionModes.get(session)?.assetSeed ?? "default";
}

export function createMockGatewayServer({
  host = DEFAULT_HOST,
  port = DEFAULT_PORT,
  logger = console,
} = {}) {
  let boundPort = port;
  // In-memory handoff store: code → seeded session info (single-use)
  const handoffStore = new Map();
  const sessionModes = new Map();

  const server = http.createServer(async (req, res) => {
    const url = new URL(req.url ?? "/", `http://${req.headers.host}`);

    if (req.method === "GET" && url.pathname === "/health/ready") {
      return sendJson(res, 200, { service: "gateway", status: "ready" });
    }

    if (req.method === "GET" && url.pathname === "/health/live") {
      return sendJson(res, 200, { service: "gateway", status: "live" });
    }

    // Pre-seed a handoff code (used by mint-handoff-code.mjs substitute)
    if (req.method === "POST" && url.pathname === "/e2e/issue-handoff") {
      const sessionRef = `e2e-session-${randomUUID()}`;
      const handoffCode = `e2e-handoff-${randomUUID()}`;
      const assetSeed = url.searchParams.get("asset_seed") === "empty" ? "empty" : "default";
      handoffStore.set(handoffCode, { sessionRef, assetSeed });
      logger.log?.(`[mock-gateway] issued handoff_code=${handoffCode}`);
      return sendJson(res, 200, {
        auth: {
          handoff_code: handoffCode,
          bootstrap_deeplink: `dubbridge://auth/callback?handoff_code=${handoffCode}`,
        },
        meta: {
          gateway_base_url: `http://localhost:${boundPort}`,
          asset_seed: assetSeed,
        },
      });
    }

    // Mobile session redemption
    if (req.method === "POST" && url.pathname === "/auth/mobile/session") {
      let payload;
      try {
        payload = JSON.parse(await readBody(req));
      } catch {
        return sendJson(res, 400, { error: "invalid_json" });
      }

      const code = typeof payload?.handoff_code === "string" ? payload.handoff_code.trim() : null;
      if (!code) return sendJson(res, 400, { error: "missing_handoff_code" });

      const sessionInfo = handoffStore.get(code);
      if (!sessionInfo) {
        logger.log?.(`[mock-gateway] handoff_code not found: ${code}`);
        return sendJson(res, 401, { error: "invalid_handoff_code" });
      }

      handoffStore.delete(code);
      sessionModes.set(sessionInfo.sessionRef, { assetSeed: sessionInfo.assetSeed });
      logger.log?.(
        `[mock-gateway] redeemed handoff_code=${code} -> session_ref=${sessionInfo.sessionRef}`,
      );
      return sendJson(res, 200, { session_ref: sessionInfo.sessionRef });
    }

    if (url.pathname.startsWith("/api/") && !hasSession(req)) {
      return sendJson(res, 401, { error: "missing_session" });
    }

    if (req.method === "GET" && url.pathname === "/api/assets") {
      const assetSeed = getAssetSeedMode(req, sessionModes);
      return sendJson(res, 200, assetSeed === "empty" ? [] : SEED_ASSETS);
    }

    const assetMatch = url.pathname.match(/^\/api\/assets\/([^/]+)$/);
    if (req.method === "GET" && assetMatch) {
      const asset = SEED_ASSETS.find((seed) => seed.id === assetMatch[1]);
      if (!asset) return sendJson(res, 404, { error: "asset_not_found" });
      return sendJson(res, 200, asset);
    }

    if (req.method === "POST" && url.pathname === "/api/ingest") {
      return sendJson(res, 201, { ingest_token: randomUUID() });
    }

    if (req.method === "POST" && /^\/api\/ingest\/[^/]+\/rights$/.test(url.pathname)) {
      return sendJson(res, 200, {});
    }

    if (req.method === "POST" && /^\/api\/ingest\/[^/]+\/finalize$/.test(url.pathname)) {
      return sendJson(res, 201, SEED_ASSETS[0]);
    }

    sendJson(res, 404, { error: "not_found", path: url.pathname });
  });

  return {
    host,
    async start() {
      await new Promise((resolve, reject) => {
        server.once("error", reject);
        server.listen(port, host, resolve);
      });

      const address = server.address();
      if (!address || typeof address === "string") {
        throw new Error("mock gateway server failed to resolve its bound address");
      }

      boundPort = address.port;

      return {
        host,
        port: address.port,
        baseUrl: `http://${host}:${address.port}`,
      };
    },
    async close() {
      await new Promise((resolve, reject) => {
        server.close((error) => {
          if (error) {
            reject(error);
            return;
          }

          resolve();
        });
      });
    },
  };
}

async function main() {
  const server = createMockGatewayServer();
  const binding = await server.start();

  console.log(`[mock-gateway] listening on ${binding.baseUrl}`);

  const shutdown = async () => {
    await server.close();
    process.exit(0);
  };

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error("[mock-gateway] failed to start", error);
    process.exit(1);
  });
}
