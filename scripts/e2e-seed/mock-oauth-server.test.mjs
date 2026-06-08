import test from "node:test";
import assert from "node:assert/strict";

import {
  buildFixtureTokenSet,
  createMockOAuthServer,
} from "./mock-oauth-server.mjs";

test("buildFixtureTokenSet returns a deterministic default token set", () => {
  assert.deepEqual(buildFixtureTokenSet({}), {
    access_token: "fixture-access-token",
    refresh_token: "fixture-refresh-token",
    expires_in: 3600,
    token_type: "Bearer",
  });
});

test("mock oauth server serves health and deterministic token responses", async () => {
  const server = createMockOAuthServer({
    port: 0,
    logger: {
      info() {},
    },
  });
  const binding = await server.start();

  try {
    const healthResponse = await fetch(`${binding.baseUrl}/health/live`);
    assert.equal(healthResponse.status, 200);
    assert.deepEqual(await healthResponse.json(), { ok: true });

    const tokenResponse = await fetch(`${binding.baseUrl}/oauth/token`, {
      method: "POST",
      headers: {
        "content-type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({
        grant_type: "authorization_code",
        code: "code-123",
        redirect_uri: "http://localhost:8081/auth/callback",
        client_id: "dubbridge-web-local",
      }),
    });

    assert.equal(tokenResponse.status, 200);
    assert.deepEqual(await tokenResponse.json(), buildFixtureTokenSet({}));
  } finally {
    await server.close();
  }
});

test("mock oauth server rejects unsupported grant types", async () => {
  const server = createMockOAuthServer({
    port: 0,
    logger: {
      info() {},
    },
  });
  const binding = await server.start();

  try {
    const tokenResponse = await fetch(`${binding.baseUrl}/oauth/token`, {
      method: "POST",
      headers: {
        "content-type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({
        grant_type: "client_credentials",
      }),
    });

    assert.equal(tokenResponse.status, 400);
    assert.deepEqual(await tokenResponse.json(), {
      error: "unsupported_grant_type",
      error_description: "expected authorization_code or refresh_token",
    });
  } finally {
    await server.close();
  }
});
