import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { access } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { createServer as createTcpServer } from "node:net";
import { createServer as createHttpServer } from "node:http";
import path from "node:path";
import { homedir } from "node:os";

import { createMockOAuthServer } from "./mock-oauth-server.mjs";
import {
  assertOpaqueOnlySeedPayload,
  ensureGatewayReady,
  verifySingleUseRedemption,
} from "./mint-handoff-code.mjs";

const REPO_ROOT = fileURLToPath(new URL("../../", import.meta.url));
const SEED_SCRIPT_PATH = fileURLToPath(
  new URL("./mint-handoff-code.mjs", import.meta.url),
);
const GATEWAY_BINARY_PATH = path.join(
  REPO_ROOT,
  "target",
  "debug",
  "dubbridge-gateway",
);
const CARGO_BIN =
  process.env.CARGO_BIN ?? path.join(homedir(), ".cargo", "bin", "cargo");

let gatewayBinaryPromise;

test("assertOpaqueOnlySeedPayload rejects access_token and JWT-like values", () => {
  assert.throws(
    () => assertOpaqueOnlySeedPayload({ auth: { access_token: "secret" } }),
    /access_token must not appear/,
  );
  assert.throws(
    () =>
      assertOpaqueOnlySeedPayload({
        auth: { handoff_code: "eyJhbGciOiJIUzI1NiJ9.payload.signature" },
      }),
    /JWT-like value/,
  );
});

test(
  "V4b HP-1: seed CLI emits only handoff output and the code redeems once",
  { timeout: 120_000 },
  async () => {
    const mockServer = createMockOAuthServer({
      port: 0,
      logger: {
        info() {},
      },
    });
    const binding = await mockServer.start();
    const gatewayPort = await reservePort();
    const gateway = await startGatewayProcess({
      gatewayPort,
      mockBaseUrl: binding.baseUrl,
    });

    try {
      const gatewayBaseUrl = `http://127.0.0.1:${gatewayPort}`;
      await ensureGatewayReady(gatewayBaseUrl);

      const cliResult = await runCommand(process.execPath, [
        SEED_SCRIPT_PATH,
        "--gateway-base-url",
        gatewayBaseUrl,
        "--auth-code",
        "cli-seed-code",
      ]);

      assert.equal(cliResult.exitCode, 0, cliResult.stderr);
      assert.equal(cliResult.stderr, "");
      assert.doesNotMatch(cliResult.stdout, /access_token|refresh_token/);

      const payload = JSON.parse(cliResult.stdout);
      assert.equal(typeof payload.auth?.handoff_code, "string");
      assert.equal(payload.auth.handoff_code.length, 43);
      assert.equal(
        payload.auth.bootstrap_deeplink,
        `dubbridge://auth/callback?handoff_code=${payload.auth.handoff_code}`,
      );
      assert.equal(payload.meta.gateway_base_url, gatewayBaseUrl);
      assertOpaqueOnlySeedPayload(payload);

      const secondCliResult = await runCommand(process.execPath, [
        SEED_SCRIPT_PATH,
        "--gateway-base-url",
        gatewayBaseUrl,
        "--auth-code",
        "cli-seed-code-2",
      ]);
      assert.equal(secondCliResult.exitCode, 0, secondCliResult.stderr);

      const secondPayload = JSON.parse(secondCliResult.stdout);
      assert.notEqual(
        secondPayload.auth.handoff_code,
        payload.auth.handoff_code,
        "seed must mint a fresh handoff_code on each run",
      );

      const verification = await verifySingleUseRedemption({
        gatewayBaseUrl,
        handoffCode: payload.auth.handoff_code,
      });
      assert.deepEqual(verification, {
        first_redeem_status: 200,
        second_redeem_status: 401,
      });
    } finally {
      await gateway.stop();
      await mockServer.close();
    }
  },
);

test(
  "ensureGatewayReady rejects non-gateway health payloads",
  async () => {
    const port = await reservePort();
    const server = await startJsonServer(port, {
      "/health/ready": {
        status: 200,
        body: {
          id: "expo-manifest",
          status: "not-a-gateway",
        },
      },
    });

    try {
      await assert.rejects(
        () => ensureGatewayReady(`http://127.0.0.1:${port}`),
        /expected gateway readiness response/,
      );
    } finally {
      await new Promise((resolve, reject) => {
        server.close((error) => {
          if (error) {
            reject(error);
            return;
          }

          resolve();
        });
      });
    }
  },
);

async function resolveGatewayBinary() {
  if (!gatewayBinaryPromise) {
    gatewayBinaryPromise = (async () => {
      try {
        await access(GATEWAY_BINARY_PATH);
        return GATEWAY_BINARY_PATH;
      } catch {
        const buildResult = await runCommand(CARGO_BIN, [
          "build",
          "-p",
          "dubbridge-gateway",
        ]);
        assert.equal(buildResult.exitCode, 0, buildResult.stderr);
        await access(GATEWAY_BINARY_PATH);
        return GATEWAY_BINARY_PATH;
      }
    })();
  }

  return gatewayBinaryPromise;
}

async function startGatewayProcess({ gatewayPort, mockBaseUrl }) {
  const binaryPath = await resolveGatewayBinary();
  const stdout = [];
  const stderr = [];
  const child = spawn(binaryPath, [], {
    cwd: REPO_ROOT,
    env: {
      ...process.env,
      DUBBRIDGE_ENV: "local",
      DUBBRIDGE_GATEWAY__PORT: String(gatewayPort),
      DUBBRIDGE_GATEWAY__UPSTREAM_API_BASE_URL: "http://127.0.0.1:18080",
      DUBBRIDGE_GATEWAY__OAUTH__AUTHORIZATION_URL: `${mockBaseUrl}/oauth/authorize`,
      DUBBRIDGE_GATEWAY__OAUTH__TOKEN_URL: `${mockBaseUrl}/oauth/token`,
      DUBBRIDGE_GATEWAY__OAUTH__REDIRECT_URL: `http://127.0.0.1:${gatewayPort}/auth/callback`,
      DUBBRIDGE_REDIS_URL: "redis://127.0.0.1:6379",
      RUST_LOG: "warn",
    },
    stdio: ["ignore", "pipe", "pipe"],
  });

  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  child.stdout.on("data", (chunk) => stdout.push(chunk));
  child.stderr.on("data", (chunk) => stderr.push(chunk));

  try {
    await waitFor(async () => {
      await ensureGatewayReady(`http://127.0.0.1:${gatewayPort}`);
    });
  } catch (error) {
    child.kill("SIGTERM");
    throw new Error(
      `gateway failed to start: ${error instanceof Error ? error.message : String(error)}\n${stderr.join("")}${stdout.join("")}`,
    );
  }

  return {
    async stop() {
      if (child.exitCode !== null) {
        return;
      }

      child.kill("SIGTERM");
      await waitForProcessExit(child);
    },
  };
}

async function runCommand(command, args) {
  const stdout = [];
  const stderr = [];

  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: REPO_ROOT,
      stdio: ["ignore", "pipe", "pipe"],
    });

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", reject);
    child.on("exit", (exitCode) => {
      resolve({
        exitCode,
        stdout: stdout.join(""),
        stderr: stderr.join(""),
      });
    });
  });
}

async function reservePort() {
  const server = createTcpServer();

  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("failed to reserve a local TCP port"));
        return;
      }

      const { port } = address;
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }

        resolve(port);
      });
    });
  });
}

async function waitFor(work, { attempts = 100, delayMs = 100 } = {}) {
  let lastError;

  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      return await work();
    } catch (error) {
      lastError = error;
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }
  }

  throw lastError ?? new Error("waitFor timed out");
}

async function waitForProcessExit(child, timeoutMs = 10_000) {
  await Promise.race([
    new Promise((resolve) => child.once("exit", resolve)),
    new Promise((_, reject) =>
      setTimeout(() => {
        child.kill("SIGKILL");
        reject(new Error("process did not exit before timeout"));
      }, timeoutMs),
    ),
  ]);
}

async function startJsonServer(port, routes) {
  const server = await new Promise((resolve, reject) => {
    const instance = createHttpServer((request, response) => {
      const route = routes[request.url];

      if (!route) {
        response.writeHead(404, { "content-type": "application/json" });
        response.end(JSON.stringify({ error: "not_found" }));
        return;
      }

      response.writeHead(route.status, {
        "content-type": "application/json; charset=utf-8",
      });
      response.end(JSON.stringify(route.body));
    });
    instance.once("error", reject);
    instance.listen(port, "127.0.0.1", () => resolve(instance));
  });

  return server;
}
