import test from "node:test";
import assert from "node:assert/strict";

import {
  SEED_ASSETS,
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
