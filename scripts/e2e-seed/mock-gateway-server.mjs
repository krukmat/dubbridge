#!/usr/bin/env node
// Minimal mock gateway for E2E screenshot suite (S-200 credential auth).
// Endpoints:
//   GET  /health/ready     — health-check gate
//   POST /auth/login       — email/password → bearer token
//   POST /auth/register    — registration → bearer token
//   POST /e2e/seed         — set per-phase asset_seed / ingest_seed mode
//   GET  /api/*            — mocked API responses gated by Authorization: Bearer

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

// E2E credential constants — used by seed-and-run.sh and the mock /auth/login handler.
export const E2E_EMAIL = "e2e@dubbridge.dev";
export const E2E_PASSWORD = "e2etestpass123";
export const E2E_BEARER_TOKEN = "e2e-bearer-token";

// Session used for the non-member EC-2 / SC-MEMBER-2 fixture.
export const NON_MEMBER_SESSION = "e2e-non-member-session";
export const NON_MEMBER_BEARER = "e2e-non-member-bearer";

// Session used for the non-reviewer EC-2 / review fixture.
export const NON_REVIEWER_SESSION = "e2e-non-reviewer-session";
export const NON_REVIEWER_BEARER = "e2e-non-reviewer-bearer";

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

const PLAYBACK_TOKEN = "e2e-playback-token";

function sendText(
  res,
  status,
  body,
  contentType = "text/plain; charset=utf-8",
) {
  res.writeHead(status, { "content-type": contentType });
  res.end(body);
}

function sendBytes(res, status, body, contentType) {
  res.writeHead(status, { "content-type": contentType });
  res.end(body);
}

function sendJson(res, status, body) {
  res.writeHead(status, { "content-type": "application/json; charset=utf-8" });
  res.end(JSON.stringify(body));
}

async function readBody(req) {
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  return Buffer.concat(chunks).toString("utf8");
}

function extractBearer(req) {
  const auth = req.headers["authorization"] ?? "";
  if (auth.startsWith("Bearer ")) return auth.slice(7).trim();
  return null;
}

function hasAuth(req) {
  const token = extractBearer(req);
  return typeof token === "string" && token.trim().length > 0;
}

function getAssetSeedMode(req, tokenModes) {
  const token = extractBearer(req);
  if (!token) return "default";
  return tokenModes.get(token)?.assetSeed ?? "default";
}

export function createMockGatewayServer({
  host = DEFAULT_HOST,
  port = DEFAULT_PORT,
  logger = console,
} = {}) {
  let boundPort = port;
  // In-memory bearer token → seed mode mapping
  const tokenModes = new Map();
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

    // S-200 bearer auth — relay login (mock issues a fixed E2E token)
    if (req.method === "POST" && url.pathname === "/auth/login") {
      let payload;
      try { payload = JSON.parse(await readBody(req)); } catch {
        return sendJson(res, 400, { error: "invalid_json" });
      }
      const { email, password } = payload ?? {};
      if (!email || !password) return sendJson(res, 400, { error: "missing_fields" });
      if (email !== E2E_EMAIL || password !== E2E_PASSWORD) {
        return sendJson(res, 401, { error: "invalid_credentials" });
      }
      // Preserve any mode pre-seeded via /e2e/seed (e.g. ingest_seed=no_rights):
      // the Maestro phase seeds the mode BEFORE launching the app, and the app
      // logs in on launch. Overwriting here would wipe the seeded mode.
      if (!tokenModes.has(E2E_BEARER_TOKEN)) {
        tokenModes.set(E2E_BEARER_TOKEN, { assetSeed: "default" });
      }
      logger.log?.(`[mock-gateway] login ok -> token=${E2E_BEARER_TOKEN}`);
      return sendJson(res, 200, {
        token: E2E_BEARER_TOKEN,
        userId: "00000000-0000-0000-0000-000000000001",
        workspaceId: "00000000-0000-0000-0000-000000000002",
      });
    }

    // S-200 bearer auth — relay register
    if (req.method === "POST" && url.pathname === "/auth/register") {
      let payload;
      try { payload = JSON.parse(await readBody(req)); } catch {
        return sendJson(res, 400, { error: "invalid_json" });
      }
      const { email, password, workspaceName } = payload ?? {};
      if (!email || !password || !workspaceName) return sendJson(res, 400, { error: "missing_fields" });
      // Preserve any mode pre-seeded via /e2e/seed (see /auth/login note).
      if (!tokenModes.has(E2E_BEARER_TOKEN)) {
        tokenModes.set(E2E_BEARER_TOKEN, { assetSeed: "default" });
      }
      logger.log?.(`[mock-gateway] register ok -> token=${E2E_BEARER_TOKEN}`);
      return sendJson(res, 201, {
        token: E2E_BEARER_TOKEN,
        userId: "00000000-0000-0000-0000-000000000001",
        workspaceId: "00000000-0000-0000-0000-000000000002",
      });
    }

    // E2E seed control — set asset/ingest mode for a token without going through the UI
    if (req.method === "POST" && url.pathname === "/e2e/seed") {
      const assetSeed = url.searchParams.get("asset_seed") === "empty" ? "empty" : "default";
      const ingestSeed = url.searchParams.get("ingest_seed") === "no_rights" ? "no_rights" : undefined;
      tokenModes.set(E2E_BEARER_TOKEN, { assetSeed, ingestSeed });
      logger.log?.(`[mock-gateway] e2e/seed asset_seed=${assetSeed} ingest_seed=${ingestSeed}`);
      return sendJson(res, 200, { ok: true, asset_seed: assetSeed });
    }

    if (url.pathname.startsWith("/api/") && !hasAuth(req)) {
      return sendJson(res, 401, { error: "missing_auth" });
    }

    if (req.method === "GET" && url.pathname === "/api/assets") {
      const assetSeed = getAssetSeedMode(req, tokenModes);
      return sendJson(res, 200, assetSeed === "empty" ? [] : SEED_ASSETS);
    }

    const assetMatch = url.pathname.match(/^\/api\/assets\/([^/]+)$/);
    if (req.method === "GET" && assetMatch) {
      const asset = SEED_ASSETS.find((seed) => seed.id === assetMatch[1]);
      if (!asset) return sendJson(res, 404, { error: "asset_not_found" });
      return sendJson(res, 200, asset);
    }

    const playbackGrantMatch = url.pathname.match(/^\/api\/assets\/([^/]+)\/playback-grants$/);
    if (req.method === "POST" && playbackGrantMatch) {
      const asset = SEED_ASSETS.find((seed) => seed.id === playbackGrantMatch[1]);
      if (!asset) {
        return sendJson(res, 404, { error: "asset_not_found" });
      }
      if (asset.status !== "finalized") {
        return sendJson(res, 409, { error: "asset_not_ready" });
      }
      return sendJson(res, 201, { grant_id: `grant-${asset.id}` });
    }

    const playbackManifestMatch = url.pathname.match(
      /^\/api\/assets\/([^/]+)\/playback\/([^/]+)\/manifest$/,
    );
    if (req.method === "GET" && playbackManifestMatch) {
      const asset = SEED_ASSETS.find((seed) => seed.id === playbackManifestMatch[1]);
      if (!asset) {
        return sendJson(res, 404, { error: "asset_not_found" });
      }
      const manifest = [
        "#EXTM3U",
        "#EXT-X-VERSION:3",
        "#EXT-X-TARGETDURATION:6",
        "#EXT-X-MEDIA-SEQUENCE:0",
        "#EXTINF:6.0,",
        `http://${req.headers.host}/api/assets/${asset.id}/playback/segments/segment-00000.ts?token=${PLAYBACK_TOKEN}`,
        "#EXT-X-ENDLIST",
      ].join("\n");
      return sendText(res, 200, manifest, "application/vnd.apple.mpegurl");
    }

    const playbackSegmentMatch = url.pathname.match(
      /^\/api\/assets\/([^/]+)\/playback\/segments\/([^/]+)$/,
    );
    if (req.method === "GET" && playbackSegmentMatch) {
      const asset = SEED_ASSETS.find((seed) => seed.id === playbackSegmentMatch[1]);
      if (!asset) {
        return sendJson(res, 404, { error: "asset_not_found" });
      }
      if (url.searchParams.get("token") !== PLAYBACK_TOKEN) {
        return sendJson(res, 403, { error: "forbidden" });
      }
      return sendBytes(
        res,
        200,
        Buffer.from("DUBBRIDGE_PLAYBACK_E2E_SEGMENT"),
        "video/mp2t",
      );
    }

    if (req.method === "POST" && url.pathname === "/api/ingest") {
      const ingestToken = randomUUID();
      const tokenMode = tokenModes.get(extractBearer(req));
      if (tokenMode?.ingestSeed === "no_rights") {
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

    const bearer = extractBearer(req);
    const isNonMember = bearer === NON_MEMBER_BEARER;
    const isNonReviewer = bearer === NON_REVIEWER_BEARER;

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
