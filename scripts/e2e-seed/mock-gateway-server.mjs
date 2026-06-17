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

export const SEED_ORG = {
  id: "org-seed-1",
  name: "E2E Org",
  viewer_role: "owner",
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

export const SEED_PROJECT = {
  id: "project-seed-1",
  org_id: "org-seed-1",
  name: "E2E Project",
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

export const SEED_MEMBER = {
  org_id: "org-seed-1",
  subject_id: "e2e-user",
  role: "owner",
  joined_at: "2026-01-01T00:00:00Z",
};

export const SEED_AUDIT_EVENTS = [
  {
    id: "audit-seed-1",
    asset_id: "asset-seed-1",
    event_kind: "ingest_finalized",
    ingest_token: null,
    recording_session_id: null,
    platform_ingest_session_id: null,
    detail: "seeded mobile compliance event",
    happened_at: "2026-01-01T10:00:00Z",
  },
];

export const SEED_RIGHTS_RECORDS = [
  {
    id: "rights-seed-1",
    asset_id: "asset-seed-1",
    owner: "E2E Org",
    license_type: "owned",
    source_type: "direct_upload",
    proof_reference: "proof://e2e-owned",
    created_at: "2026-01-01T09:00:00Z",
  },
];

// Session used for the non-member EC-2 / SC-MEMBER-2 fixture.
export const NON_MEMBER_SESSION = "e2e-non-member-session";

// Session used for the non-reviewer EC-2 / review fixture.
export const NON_REVIEWER_SESSION = "e2e-non-reviewer-session";

export const SEED_REVIEW_TASK = {
  id: "review-task-seed-1",
  org_id: "org-seed-1",
  project_id: "project-seed-1",
  asset_id: "asset-seed-1",
  target_language_id: "lang-seed-1",
  assignee_subject_id: "e2e-user",
  state: "pending",
  created_at: "2026-06-13T10:00:00Z",
  updated_at: "2026-06-13T10:00:00Z",
  assigned_at: "2026-06-13T10:00:00Z",
};

export const SEED_NOTIFICATION = {
  id: "notification-seed-1",
  recipient_subject_id: "e2e-user",
  notification_kind: "review_task_decided",
  ref_entity_type: "review_task",
  ref_entity_id: "review-task-seed-1",
  actor_subject_id: "e2e-user",
  read_at: null,
  created_at: "2026-06-13T10:00:00Z",
};

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
  // In-memory ingest token mode: token → "no_rights" | undefined
  const ingestTokenModes = new Map();
  const consentRows = [];
  // In-memory review / notification state — mutable per-request
  let reviewTaskStore = { ...SEED_REVIEW_TASK };
  const notificationStore = [{ ...SEED_NOTIFICATION }];
  const pushTokenStore = [];

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
      const ingestSeed = url.searchParams.get("ingest_seed") === "no_rights" ? "no_rights" : undefined;
      handoffStore.set(handoffCode, { sessionRef, assetSeed, ingestSeed });
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
      sessionModes.set(sessionInfo.sessionRef, {
        assetSeed: sessionInfo.assetSeed,
        ingestSeed: sessionInfo.ingestSeed,
      });
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
      const ingestToken = randomUUID();
      const sessionMode = sessionModes.get(req.headers["x-dubbridge-session"]);
      if (sessionMode?.ingestSeed === "no_rights") {
        ingestTokenModes.set(ingestToken, "no_rights");
      }
      return sendJson(res, 201, { ingest_token: ingestToken });
    }

    if (req.method === "POST" && /^\/api\/ingest\/[^/]+\/rights$/.test(url.pathname)) {
      return sendJson(res, 200, {});
    }

    if (req.method === "POST" && /^\/api\/ingest\/[^/]+\/finalize$/.test(url.pathname)) {
      const token = url.pathname.split("/")[3];
      if (ingestTokenModes.get(token) === "no_rights") {
        return sendJson(res, 422, { error: "rights_required" });
      }
      return sendJson(res, 201, SEED_ASSETS[0]);
    }

    const complianceMatch = url.pathname.match(/^\/api\/assets\/([^/]+)\/(audit|rights|consents)$/);
    if (req.method === "GET" && complianceMatch) {
      const [, assetId, resource] = complianceMatch;
      if (assetId !== SEED_ASSETS[0].id) {
        return sendJson(res, 403, { error: "asset_not_found" });
      }
      if (resource === "audit") {
        return sendJson(res, 200, { asset_id: assetId, events: SEED_AUDIT_EVENTS });
      }
      if (resource === "rights") {
        return sendJson(res, 200, { asset_id: assetId, entries: SEED_RIGHTS_RECORDS });
      }
      return sendJson(res, 200, {
        asset_id: assetId,
        current_status: consentRows.at(-1)?.status ?? null,
        rows: consentRows,
      });
    }

    if (req.method === "POST" && url.pathname === "/api/consents") {
      const request = JSON.parse((await readBody(req)) || "{}");
      if (request.status === "grant" && !request.evidence_ref) {
        return sendJson(res, 422, { error: "evidence reference is required" });
      }
      const row = {
        id: `consent-seed-${consentRows.length + 1}`,
        asset_id: request.asset_id,
        scope: request.scope,
        status: request.status,
        evidence_ref: request.evidence_ref ?? null,
        granted_by: "00000000-0000-0000-0000-000000000001",
        happened_at: `2026-01-01T1${consentRows.length + 1}:00:00Z`,
      };
      consentRows.push(row);
      return sendJson(res, 201, {
        asset_id: row.asset_id,
        scope: row.scope,
        current_status: row.status,
        happened_at: row.happened_at,
      });
    }

    // -------------------------------------------------------------------------
    // Workspace routes (S-100-T7) — in-memory fixtures, session-gated
    // Non-member sessions (NON_MEMBER_SESSION) are rejected on org-scoped routes.
    // -------------------------------------------------------------------------

    const isNonMember =
      req.headers["x-dubbridge-session"] === NON_MEMBER_SESSION;
    const isNonReviewer =
      req.headers["x-dubbridge-session"] === NON_REVIEWER_SESSION;

    // GET /api/orgs — list orgs for the session
    if (req.method === "GET" && url.pathname === "/api/orgs") {
      return sendJson(res, 200, [SEED_ORG]);
    }

    // POST /api/orgs — create org (returns seed for determinism)
    if (req.method === "POST" && url.pathname === "/api/orgs") {
      return sendJson(res, 201, SEED_ORG);
    }

    // Org-scoped routes — deny non-members with 403
    const orgMatch = url.pathname.match(/^\/api\/orgs\/([^/]+)(\/.*)?$/);
    if (orgMatch) {
      if (isNonMember) {
        return sendJson(res, 403, { error: "forbidden" });
      }

      const orgId = orgMatch[1];
      const subPath = orgMatch[2] ?? "";

      // GET /api/orgs/{orgId}/members
      if (req.method === "GET" && subPath === "/members") {
        return sendJson(res, 200, [SEED_MEMBER]);
      }

      // POST /api/orgs/{orgId}/members
      if (req.method === "POST" && subPath === "/members") {
        return sendJson(res, 201, SEED_MEMBER);
      }

      // GET /api/orgs/{orgId}/projects
      if (req.method === "GET" && subPath === "/projects") {
        return sendJson(res, 200, [SEED_PROJECT]);
      }

      // POST /api/orgs/{orgId}/projects
      if (req.method === "POST" && subPath === "/projects") {
        return sendJson(res, 201, SEED_PROJECT);
      }

      // GET /api/orgs/{orgId}/projects/{projectId}
      const projectDetailMatch = subPath.match(/^\/projects\/([^/]+)$/);
      if (req.method === "GET" && projectDetailMatch) {
        const projectId = projectDetailMatch[1];
        if (projectId !== SEED_PROJECT.id) {
          return sendJson(res, 404, { error: "project_not_found" });
        }
        return sendJson(res, 200, {
          ...SEED_PROJECT,
          assets: SEED_ASSETS,
          target_languages: [],
        });
      }

      // GET /api/orgs/{orgId}/projects/{projectId}/review-tasks — reviewer queue
      const reviewQueueMatch = subPath.match(/^\/projects\/([^/]+)\/review-tasks$/);
      if (req.method === "GET" && reviewQueueMatch) {
        const projectId = reviewQueueMatch[1];
        if (isNonReviewer) {
          return sendJson(res, 403, { error: "forbidden" });
        }
        return sendJson(res, 200, {
          org_id: orgId,
          project_id: projectId,
          tasks: projectId === SEED_PROJECT.id ? [reviewTaskStore] : [],
        });
      }

      // POST /api/orgs/{orgId}/projects/{projectId}/review-tasks/{id}/decision
      const reviewDecisionMatch = subPath.match(
        /^\/projects\/([^/]+)\/review-tasks\/([^/]+)\/decision$/,
      );
      if (req.method === "POST" && reviewDecisionMatch) {
        if (isNonReviewer) {
          return sendJson(res, 403, { error: "forbidden" });
        }
        const taskId = reviewDecisionMatch[2];
        if (taskId !== reviewTaskStore.id) {
          return sendJson(res, 404, { error: "review_task_not_found" });
        }
        let body;
        try {
          body = JSON.parse(await readBody(req));
        } catch {
          return sendJson(res, 400, { error: "invalid_json" });
        }
        const verdict = body?.verdict;
        if (verdict !== "approved" && verdict !== "rejected") {
          return sendJson(res, 422, { error: "invalid_verdict" });
        }
        reviewTaskStore = { ...reviewTaskStore, state: verdict, updated_at: new Date().toISOString() };
        notificationStore.push({
          id: `notification-seed-${notificationStore.length + 2}`,
          recipient_subject_id: reviewTaskStore.assignee_subject_id ?? "e2e-user",
          notification_kind: "review_task_decided",
          ref_entity_type: "review_task",
          ref_entity_id: reviewTaskStore.id,
          actor_subject_id: "e2e-user",
          read_at: null,
          created_at: new Date().toISOString(),
        });
        return sendJson(res, 200, {
          review_task_id: taskId,
          state: reviewTaskStore.state,
          sessionRotation: null,
        });
      }

      // POST /api/orgs/{orgId}/projects/{projectId}/review-tasks/{id}/publish
      const reviewPublishMatch = subPath.match(
        /^\/projects\/([^/]+)\/review-tasks\/([^/]+)\/publish$/,
      );
      if (req.method === "POST" && reviewPublishMatch) {
        const taskId = reviewPublishMatch[2];
        if (taskId !== reviewTaskStore.id) {
          return sendJson(res, 404, { error: "review_task_not_found" });
        }
        if (reviewTaskStore.state !== "approved") {
          return sendJson(res, 409, { error: "review_not_approved" });
        }
        const now = new Date().toISOString();
        notificationStore.push({
          id: `notification-seed-${notificationStore.length + 2}`,
          recipient_subject_id: reviewTaskStore.assignee_subject_id ?? "e2e-user",
          notification_kind: "review_task_published",
          ref_entity_type: "review_task",
          ref_entity_id: reviewTaskStore.id,
          actor_subject_id: "e2e-user",
          read_at: null,
          created_at: now,
        });
        return sendJson(res, 201, {
          review_task_id: taskId,
          status: "published",
          published_by: "e2e-user",
          published_at: now,
          sessionRotation: null,
        });
      }

      void orgId;
    }

    // -------------------------------------------------------------------------
    // Notification routes (S-160-T4c / T4d) — session-gated, caller-scoped
    // -------------------------------------------------------------------------

    // GET /api/notifications — list caller's notifications
    if (req.method === "GET" && url.pathname === "/api/notifications") {
      const callerRows = notificationStore.filter(
        (n) => n.recipient_subject_id === "e2e-user",
      );
      return sendJson(res, 200, { notifications: callerRows });
    }

    // POST /api/notifications/mark-read
    if (req.method === "POST" && url.pathname === "/api/notifications/mark-read") {
      let body;
      try {
        body = JSON.parse(await readBody(req));
      } catch {
        return sendJson(res, 400, { error: "invalid_json" });
      }
      const ids = Array.isArray(body?.ids) ? body.ids : [];
      const now = new Date().toISOString();
      for (const n of notificationStore) {
        if (ids.includes(n.id) && n.recipient_subject_id === "e2e-user") {
          n.read_at = now;
        }
      }
      return sendJson(res, 200, {});
    }

    // POST /api/notifications/push-tokens — register push token
    if (req.method === "POST" && url.pathname === "/api/notifications/push-tokens") {
      let body;
      try {
        body = JSON.parse(await readBody(req));
      } catch {
        return sendJson(res, 400, { error: "invalid_json" });
      }
      if (!body?.token || typeof body.token !== "string" || body.token.trim() === "") {
        return sendJson(res, 422, { error: "token_required" });
      }
      if (body.platform !== "ios" && body.platform !== "android") {
        return sendJson(res, 422, { error: "invalid_platform" });
      }
      const existing = pushTokenStore.find((t) => t.device_token === body.token);
      if (existing) {
        return sendJson(res, 409, { error: "duplicate_token" });
      }
      pushTokenStore.push({
        device_token: body.token,
        platform: body.platform,
        subject_id: "e2e-user",
      });
      return sendJson(res, 201, {});
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
  const port = process.env.GATEWAY_PORT ? parseInt(process.env.GATEWAY_PORT, 10) : DEFAULT_PORT;
  const server = createMockGatewayServer({ port });
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
