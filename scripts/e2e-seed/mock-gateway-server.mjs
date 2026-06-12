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

const HOST = "127.0.0.1";
const PORT = 8081;

// In-memory handoff store: code → session_ref (single-use)
const handoffStore = new Map();

function sendJson(res, status, body) {
  res.writeHead(status, { "content-type": "application/json; charset=utf-8" });
  res.end(JSON.stringify(body));
}

async function readBody(req) {
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  return Buffer.concat(chunks).toString("utf8");
}

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
    handoffStore.set(handoffCode, sessionRef);
    console.log(`[mock-gateway] issued handoff_code=${handoffCode}`);
    return sendJson(res, 200, {
      auth: {
        handoff_code: handoffCode,
        bootstrap_deeplink: `dubbridge://auth/callback?handoff_code=${handoffCode}`,
      },
      meta: { gateway_base_url: `http://localhost:${PORT}` },
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

    const sessionRef = handoffStore.get(code);
    if (!sessionRef) {
      console.log(`[mock-gateway] handoff_code not found: ${code}`);
      return sendJson(res, 401, { error: "invalid_handoff_code" });
    }

    handoffStore.delete(code);
    console.log(`[mock-gateway] redeemed handoff_code=${code} -> session_ref=${sessionRef}`);
    return sendJson(res, 200, { session_ref: sessionRef });
  }

  sendJson(res, 404, { error: "not_found", path: url.pathname });
});

server.listen(PORT, HOST, () => {
  console.log(`[mock-gateway] listening on http://${HOST}:${PORT}`);
});

process.on("SIGINT", () => { server.close(); process.exit(0); });
process.on("SIGTERM", () => { server.close(); process.exit(0); });
