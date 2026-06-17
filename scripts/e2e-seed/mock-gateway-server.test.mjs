import test from "node:test";
import assert from "node:assert/strict";

import {
  SEED_ASSETS,
  SEED_ORG,
  SEED_PROJECT,
  SEED_MEMBER,
  NON_MEMBER_SESSION,
  NON_REVIEWER_SESSION,
  SEED_AUDIT_EVENTS,
  SEED_RIGHTS_RECORDS,
  SEED_REVIEW_TASK,
  createMockGatewayServer,
} from "./mock-gateway-server.mjs";

const SESSION_HEADERS = {
  "x-dubbridge-session": "e2e-session-ref",
};

function createSilentLogger() {
  return {
    log() {},
  };
}

async function withServer(fn) {
  const server = createMockGatewayServer({
    port: 0,
    logger: createSilentLogger(),
  });
  const binding = await server.start();

  try {
    await fn(binding);
  } finally {
    await server.close();
  }
}

test("mock gateway preserves the mobile handoff redemption flow", async () => {
  await withServer(async ({ baseUrl }) => {
    const issueResponse = await fetch(`${baseUrl}/e2e/issue-handoff`, {
      method: "POST",
    });

    assert.equal(issueResponse.status, 200);
    const issuePayload = await issueResponse.json();
    const handoffCode = issuePayload.auth.handoff_code;
    assert.equal(typeof handoffCode, "string");
    assert.ok(handoffCode.startsWith("e2e-handoff-"));

    const redeemResponse = await fetch(`${baseUrl}/auth/mobile/session`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({ handoff_code: handoffCode }),
    });

    assert.equal(redeemResponse.status, 200);
    const redeemPayload = await redeemResponse.json();
    assert.equal(typeof redeemPayload.session_ref, "string");
    assert.ok(redeemPayload.session_ref.startsWith("e2e-session-"));
  });
});

test("mock gateway can seed an empty asset list for a redeemed E2E session", async () => {
  await withServer(async ({ baseUrl }) => {
    const issueResponse = await fetch(`${baseUrl}/e2e/issue-handoff?asset_seed=empty`, {
      method: "POST",
    });

    assert.equal(issueResponse.status, 200);
    const issuePayload = await issueResponse.json();
    assert.equal(issuePayload.meta.asset_seed, "empty");

    const redeemResponse = await fetch(`${baseUrl}/auth/mobile/session`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({ handoff_code: issuePayload.auth.handoff_code }),
    });

    assert.equal(redeemResponse.status, 200);
    const redeemPayload = await redeemResponse.json();

    const assetsResponse = await fetch(`${baseUrl}/api/assets`, {
      headers: {
        "x-dubbridge-session": redeemPayload.session_ref,
      },
    });

    assert.equal(assetsResponse.status, 200);
    assert.deepEqual(await assetsResponse.json(), []);
  });
});

test("mock gateway rejects api requests without a session header", async () => {
  await withServer(async ({ baseUrl }) => {
    const requests = [
      fetch(`${baseUrl}/api/assets`),
      fetch(`${baseUrl}/api/ingest`, { method: "POST" }),
      fetch(`${baseUrl}/api/not-implemented`),
    ];

    for (const response of await Promise.all(requests)) {
      assert.equal(response.status, 401);
      assert.deepEqual(await response.json(), { error: "missing_session" });
    }
  });
});

test("mock gateway serves seed asset fixtures with a session header", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(`${baseUrl}/api/assets`, {
      headers: SESSION_HEADERS,
    });

    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), SEED_ASSETS);
  });
});

test("mock gateway serves a single seed asset by id", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(`${baseUrl}/api/assets/${SEED_ASSETS[0].id}`, {
      headers: SESSION_HEADERS,
    });

    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), SEED_ASSETS[0]);
  });
});

test("mock gateway returns 404 for an unknown asset id", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(`${baseUrl}/api/assets/not-a-seed`, {
      headers: SESSION_HEADERS,
    });

    assert.equal(response.status, 404);
    assert.deepEqual(await response.json(), { error: "asset_not_found" });
  });
});

test("mock gateway returns 422 on finalize when ingest_seed=no_rights", async () => {
  await withServer(async ({ baseUrl }) => {
    // Issue a handoff with ingest_seed=no_rights
    const issueResponse = await fetch(`${baseUrl}/e2e/issue-handoff?ingest_seed=no_rights`, {
      method: "POST",
    });
    assert.equal(issueResponse.status, 200);
    const issuePayload = await issueResponse.json();

    // Redeem the handoff code
    const redeemResponse = await fetch(`${baseUrl}/auth/mobile/session`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ handoff_code: issuePayload.auth.handoff_code }),
    });
    assert.equal(redeemResponse.status, 200);
    const { session_ref } = await redeemResponse.json();

    // Create an ingest session — the token is now flagged as no_rights
    const ingestResponse = await fetch(`${baseUrl}/api/ingest`, {
      method: "POST",
      headers: { "x-dubbridge-session": session_ref },
    });
    assert.equal(ingestResponse.status, 201);
    const { ingest_token } = await ingestResponse.json();

    // Finalize should return 422
    const finalizeResponse = await fetch(`${baseUrl}/api/ingest/${ingest_token}/finalize`, {
      method: "POST",
      headers: { "x-dubbridge-session": session_ref },
    });
    assert.equal(finalizeResponse.status, 422);
    assert.deepEqual(await finalizeResponse.json(), { error: "rights_required" });
  });
});

// ---------------------------------------------------------------------------
// Workspace route tests (S-100-T7)
// ---------------------------------------------------------------------------

test("mock gateway lists seed org for any authenticated session", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(`${baseUrl}/api/orgs`, {
      headers: SESSION_HEADERS,
    });

    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), [SEED_ORG]);
  });
});

test("mock gateway creates org and returns seed shape", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(`${baseUrl}/api/orgs`, {
      method: "POST",
      headers: { ...SESSION_HEADERS, "content-type": "application/json" },
      body: JSON.stringify({ name: "New Org" }),
    });

    assert.equal(response.status, 201);
    assert.deepEqual(await response.json(), SEED_ORG);
  });
});

test("mock gateway serves compliance ledgers and append-only consent mutations", async () => {
  await withServer(async ({ baseUrl }) => {
    const auditResponse = await fetch(`${baseUrl}/api/assets/${SEED_ASSETS[0].id}/audit`, { headers: SESSION_HEADERS });
    const rightsResponse = await fetch(`${baseUrl}/api/assets/${SEED_ASSETS[0].id}/rights`, { headers: SESSION_HEADERS });
    assert.deepEqual((await auditResponse.json()).events, SEED_AUDIT_EVENTS);
    assert.deepEqual((await rightsResponse.json()).entries, SEED_RIGHTS_RECORDS);

    const grantResponse = await fetch(`${baseUrl}/api/consents`, {
      method: "POST",
      headers: { ...SESSION_HEADERS, "content-type": "application/json" },
      body: JSON.stringify({
        asset_id: SEED_ASSETS[0].id,
        scope: "voice_clone",
        status: "grant",
        evidence_ref: "proof://voice",
      }),
    });
    assert.equal(grantResponse.status, 201);

    const revokeResponse = await fetch(`${baseUrl}/api/consents`, {
      method: "POST",
      headers: { ...SESSION_HEADERS, "content-type": "application/json" },
      body: JSON.stringify({
        asset_id: SEED_ASSETS[0].id,
        scope: "voice_clone",
        status: "revoke",
        evidence_ref: null,
      }),
    });
    assert.equal(revokeResponse.status, 201);

    const ledgerResponse = await fetch(`${baseUrl}/api/assets/${SEED_ASSETS[0].id}/consents`, { headers: SESSION_HEADERS });
    const ledger = await ledgerResponse.json();
    assert.equal(ledger.current_status, "revoke");
    assert.deepEqual(ledger.rows.map((row) => row.status), ["grant", "revoke"]);
  });
});

test("mock gateway lists members for org owner session", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(`${baseUrl}/api/orgs/${SEED_ORG.id}/members`, {
      headers: SESSION_HEADERS,
    });

    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), [SEED_MEMBER]);
  });
});

test("mock gateway lists projects for org member session", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(`${baseUrl}/api/orgs/${SEED_ORG.id}/projects`, {
      headers: SESSION_HEADERS,
    });

    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), [SEED_PROJECT]);
  });
});

test("mock gateway returns project detail with linked assets", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(
      `${baseUrl}/api/orgs/${SEED_ORG.id}/projects/${SEED_PROJECT.id}`,
      { headers: SESSION_HEADERS },
    );

    assert.equal(response.status, 200);
    const body = await response.json();
    assert.equal(body.id, SEED_PROJECT.id);
    assert.equal(body.org_id, SEED_ORG.id);
    assert.deepEqual(body.assets, SEED_ASSETS);
    assert.deepEqual(body.target_languages, []);
  });
});

test("mock gateway returns 404 for unknown project id", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(
      `${baseUrl}/api/orgs/${SEED_ORG.id}/projects/does-not-exist`,
      { headers: SESSION_HEADERS },
    );

    assert.equal(response.status, 404);
    assert.deepEqual(await response.json(), { error: "project_not_found" });
  });
});

test("mock gateway denies non-member session on org-scoped routes (SC-MEMBER-2)", async () => {
  await withServer(async ({ baseUrl }) => {
    const nonMemberHeaders = { "x-dubbridge-session": NON_MEMBER_SESSION };
    const routes = [
      fetch(`${baseUrl}/api/orgs/${SEED_ORG.id}/members`, { headers: nonMemberHeaders }),
      fetch(`${baseUrl}/api/orgs/${SEED_ORG.id}/projects`, { headers: nonMemberHeaders }),
      fetch(`${baseUrl}/api/orgs/${SEED_ORG.id}/projects/${SEED_PROJECT.id}`, {
        headers: nonMemberHeaders,
      }),
    ];

    for (const response of await Promise.all(routes)) {
      assert.equal(response.status, 403);
      assert.deepEqual(await response.json(), { error: "forbidden" });
    }
  });
});

test("mock gateway preserves existing 401 gate: workspace routes without session return 401", async () => {
  await withServer(async ({ baseUrl }) => {
    const routes = [
      fetch(`${baseUrl}/api/orgs`),
      fetch(`${baseUrl}/api/orgs/${SEED_ORG.id}/projects`),
    ];

    for (const response of await Promise.all(routes)) {
      assert.equal(response.status, 401);
      assert.deepEqual(await response.json(), { error: "missing_session" });
    }
  });
});

// ---------------------------------------------------------------------------

test("mock gateway returns happy-path ingest shapes", async () => {
  await withServer(async ({ baseUrl }) => {
    const ingestResponse = await fetch(`${baseUrl}/api/ingest`, {
      method: "POST",
      headers: SESSION_HEADERS,
    });

    assert.equal(ingestResponse.status, 201);
    const ingestPayload = await ingestResponse.json();
    assert.match(
      ingestPayload.ingest_token,
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
    );

    const rightsResponse = await fetch(
      `${baseUrl}/api/ingest/${ingestPayload.ingest_token}/rights`,
      {
        method: "POST",
        headers: SESSION_HEADERS,
      },
    );

    assert.equal(rightsResponse.status, 200);
    assert.deepEqual(await rightsResponse.json(), {});

    const finalizeResponse = await fetch(
      `${baseUrl}/api/ingest/${ingestPayload.ingest_token}/finalize`,
      {
        method: "POST",
        headers: SESSION_HEADERS,
      },
    );

    assert.equal(finalizeResponse.status, 201);
    assert.deepEqual(await finalizeResponse.json(), SEED_ASSETS[0]);
  });
});

// ---------------------------------------------------------------------------
// Review route tests (S-160-T8 — HP-1, EC-1, EC-2)
// ---------------------------------------------------------------------------

test("mock gateway returns review queue for reviewer session (HP-1 / SC-REVIEW-1)", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(
      `${baseUrl}/api/orgs/${SEED_ORG.id}/projects/${SEED_PROJECT.id}/review-tasks`,
      { headers: SESSION_HEADERS },
    );

    assert.equal(response.status, 200);
    const body = await response.json();
    assert.equal(body.org_id, SEED_ORG.id);
    assert.equal(body.project_id, SEED_PROJECT.id);
    assert.equal(body.tasks.length, 1);
    assert.equal(body.tasks[0].id, SEED_REVIEW_TASK.id);
    assert.equal(body.tasks[0].state, "pending");
  });
});

test("mock gateway rejects review queue without session (EC-1 / SC-REVIEW-1)", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(
      `${baseUrl}/api/orgs/${SEED_ORG.id}/projects/${SEED_PROJECT.id}/review-tasks`,
    );
    assert.equal(response.status, 401);
  });
});

test("mock gateway denies non-reviewer decide attempt (EC-2)", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(
      `${baseUrl}/api/orgs/${SEED_ORG.id}/projects/${SEED_PROJECT.id}/review-tasks/${SEED_REVIEW_TASK.id}/decision`,
      {
        method: "POST",
        headers: { "x-dubbridge-session": NON_REVIEWER_SESSION, "content-type": "application/json" },
        body: JSON.stringify({ verdict: "approved", comment: null }),
      },
    );
    assert.equal(response.status, 403);
  });
});

test("mock gateway approve→publish flow emits notifications (HP-1 / SC-REVIEW-2, SC-PUBLISH-1)", async () => {
  await withServer(async ({ baseUrl }) => {
    // Approve
    const decisionResponse = await fetch(
      `${baseUrl}/api/orgs/${SEED_ORG.id}/projects/${SEED_PROJECT.id}/review-tasks/${SEED_REVIEW_TASK.id}/decision`,
      {
        method: "POST",
        headers: { ...SESSION_HEADERS, "content-type": "application/json" },
        body: JSON.stringify({ verdict: "approved", comment: "LGTM" }),
      },
    );
    assert.equal(decisionResponse.status, 200);
    const decisionBody = await decisionResponse.json();
    assert.equal(decisionBody.state, "approved");

    // Publish
    const publishResponse = await fetch(
      `${baseUrl}/api/orgs/${SEED_ORG.id}/projects/${SEED_PROJECT.id}/review-tasks/${SEED_REVIEW_TASK.id}/publish`,
      {
        method: "POST",
        headers: { ...SESSION_HEADERS, "content-type": "application/json" },
        body: JSON.stringify({}),
      },
    );
    assert.equal(publishResponse.status, 201);
    const publishBody = await publishResponse.json();
    assert.equal(publishBody.status, "published");

    // Notifications emitted
    const notifResponse = await fetch(`${baseUrl}/api/notifications`, { headers: SESSION_HEADERS });
    assert.equal(notifResponse.status, 200);
    const notifBody = await notifResponse.json();
    assert.ok(notifBody.notifications.length >= 2, "expected at least 2 notifications after approve+publish");
  });
});

test("mock gateway refuses publish on pending task (EC-1 / SC-PUBLISH-2)", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(
      `${baseUrl}/api/orgs/${SEED_ORG.id}/projects/${SEED_PROJECT.id}/review-tasks/${SEED_REVIEW_TASK.id}/publish`,
      {
        method: "POST",
        headers: { ...SESSION_HEADERS, "content-type": "application/json" },
        body: JSON.stringify({}),
      },
    );
    assert.equal(response.status, 409);
    const body = await response.json();
    assert.equal(body.error, "review_not_approved");
  });
});

// ---------------------------------------------------------------------------
// Notification route tests (S-160-T8)
// ---------------------------------------------------------------------------

test("mock gateway lists seeded notification for authenticated session", async () => {
  await withServer(async ({ baseUrl }) => {
    const response = await fetch(`${baseUrl}/api/notifications`, { headers: SESSION_HEADERS });
    assert.equal(response.status, 200);
    const body = await response.json();
    assert.ok(Array.isArray(body.notifications));
    assert.ok(body.notifications.length >= 1);
    assert.equal(body.notifications[0].ref_entity_type, "review_task");
  });
});

test("mock gateway mark-read updates read_at for caller notifications", async () => {
  await withServer(async ({ baseUrl }) => {
    const listBefore = await fetch(`${baseUrl}/api/notifications`, { headers: SESSION_HEADERS });
    const { notifications } = await listBefore.json();
    const ids = notifications.map((n) => n.id);

    const markResponse = await fetch(`${baseUrl}/api/notifications/mark-read`, {
      method: "POST",
      headers: { ...SESSION_HEADERS, "content-type": "application/json" },
      body: JSON.stringify({ ids }),
    });
    assert.equal(markResponse.status, 200);

    const listAfter = await fetch(`${baseUrl}/api/notifications`, { headers: SESSION_HEADERS });
    const afterBody = await listAfter.json();
    for (const n of afterBody.notifications) {
      if (ids.includes(n.id)) {
        assert.ok(n.read_at !== null, `expected read_at to be set for id=${n.id}`);
      }
    }
  });
});

test("mock gateway push-token registration returns 201 and 409 on duplicate", async () => {
  await withServer(async ({ baseUrl }) => {
    const token = `ExponentPushToken[e2e-${Date.now()}]`;

    const firstResponse = await fetch(`${baseUrl}/api/notifications/push-tokens`, {
      method: "POST",
      headers: { ...SESSION_HEADERS, "content-type": "application/json" },
      body: JSON.stringify({ token, platform: "ios" }),
    });
    assert.equal(firstResponse.status, 201);

    const dupeResponse = await fetch(`${baseUrl}/api/notifications/push-tokens`, {
      method: "POST",
      headers: { ...SESSION_HEADERS, "content-type": "application/json" },
      body: JSON.stringify({ token, platform: "ios" }),
    });
    assert.equal(dupeResponse.status, 409);
  });
});

test("mock gateway rejects push-token with missing or invalid platform", async () => {
  await withServer(async ({ baseUrl }) => {
    const badPlatform = await fetch(`${baseUrl}/api/notifications/push-tokens`, {
      method: "POST",
      headers: { ...SESSION_HEADERS, "content-type": "application/json" },
      body: JSON.stringify({ token: "tok", platform: "web" }),
    });
    assert.equal(badPlatform.status, 422);

    const emptyToken = await fetch(`${baseUrl}/api/notifications/push-tokens`, {
      method: "POST",
      headers: { ...SESSION_HEADERS, "content-type": "application/json" },
      body: JSON.stringify({ token: "   ", platform: "android" }),
    });
    assert.equal(emptyToken.status, 422);
  });
});
