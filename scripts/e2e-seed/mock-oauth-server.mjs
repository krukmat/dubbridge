#!/usr/bin/env node

import http from "node:http";

const DEFAULT_HOST = "127.0.0.1";
const DEFAULT_PORT = 9000;

export function buildFixtureTokenSet(env = process.env) {
  return {
    access_token:
      env.DUBBRIDGE_E2E_FIXTURE_ACCESS_TOKEN ?? "fixture-access-token",
    refresh_token:
      env.DUBBRIDGE_E2E_FIXTURE_REFRESH_TOKEN ?? "fixture-refresh-token",
    expires_in: Number.parseInt(
      env.DUBBRIDGE_E2E_FIXTURE_EXPIRES_IN ?? "3600",
      10,
    ),
    token_type: env.DUBBRIDGE_E2E_FIXTURE_TOKEN_TYPE ?? "Bearer",
  };
}

async function readRequestBody(request) {
  const chunks = [];

  for await (const chunk of request) {
    chunks.push(chunk);
  }

  return Buffer.concat(chunks).toString("utf8");
}

function sendJson(response, statusCode, payload) {
  response.writeHead(statusCode, {
    "content-type": "application/json; charset=utf-8",
    "cache-control": "no-store",
  });
  response.end(JSON.stringify(payload));
}

function sendText(response, statusCode, body) {
  response.writeHead(statusCode, {
    "content-type": "text/plain; charset=utf-8",
    "cache-control": "no-store",
  });
  response.end(body);
}

export function createMockOAuthServer({
  host = DEFAULT_HOST,
  port = DEFAULT_PORT,
  fixture = buildFixtureTokenSet(),
  logger = console,
} = {}) {
  const server = http.createServer(async (request, response) => {
    const url = new URL(request.url ?? "/", `http://${request.headers.host}`);

    if (request.method === "GET" && url.pathname === "/health/live") {
      return sendJson(response, 200, { ok: true });
    }

    if (request.method === "GET" && url.pathname === "/oauth/authorize") {
      return sendText(
        response,
        200,
        "DubBridge mock OAuth server: use the gateway redirect state and call /auth/callback directly.",
      );
    }

    if (request.method === "POST" && url.pathname === "/oauth/token") {
      const body = await readRequestBody(request);
      const params = new URLSearchParams(body);
      const grantType = params.get("grant_type");

      if (grantType !== "authorization_code" && grantType !== "refresh_token") {
        return sendJson(response, 400, {
          error: "unsupported_grant_type",
          error_description: "expected authorization_code or refresh_token",
        });
      }

      if (grantType === "authorization_code") {
        if (!params.get("code") || !params.get("redirect_uri")) {
          return sendJson(response, 400, {
            error: "invalid_request",
            error_description: "code and redirect_uri are required",
          });
        }
      }

      if (grantType === "refresh_token" && !params.get("refresh_token")) {
        return sendJson(response, 400, {
          error: "invalid_request",
          error_description: "refresh_token is required",
        });
      }

      logger.info?.(
        `[mock-oauth-server] ${grantType} request accepted for client_id=${params.get("client_id") ?? "<missing>"}`,
      );
      return sendJson(response, 200, fixture);
    }

    return sendJson(response, 404, {
      error: "not_found",
      error_description: `${request.method} ${url.pathname} is not implemented`,
    });
  });

  return {
    fixture,
    host,
    async start() {
      await new Promise((resolve, reject) => {
        server.once("error", reject);
        server.listen(port, host, resolve);
      });

      const address = server.address();

      if (!address || typeof address === "string") {
        throw new Error("mock oauth server failed to resolve its bound address");
      }

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
  const fixture = buildFixtureTokenSet();
  const server = createMockOAuthServer({ fixture });
  const binding = await server.start();

  console.log(
    `[mock-oauth-server] listening on ${binding.baseUrl} with deterministic fixture token set`,
  );

  const shutdown = async () => {
    await server.close();
    process.exit(0);
  };

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error("[mock-oauth-server] failed to start", error);
    process.exit(1);
  });
}
