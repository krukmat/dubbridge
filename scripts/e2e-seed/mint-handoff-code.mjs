#!/usr/bin/env node

const DEFAULT_GATEWAY_BASE_URL =
  process.env.DUBBRIDGE_E2E_GATEWAY_BASE_URL ?? "http://127.0.0.1:8081";
const DEFAULT_RETURN_URI =
  process.env.DUBBRIDGE_E2E_RETURN_URI ?? "dubbridge://auth/callback";
const DEFAULT_AUTH_CODE =
  process.env.DUBBRIDGE_E2E_AUTH_CODE ?? "seed-auth-code";

const FORBIDDEN_OUTPUT_KEYS = new Set(["access_token", "refresh_token"]);
const JWT_LIKE_PATTERN =
  /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/;

export function buildBootstrapDeepLink(returnUri, handoffCode) {
  const deepLink = new URL(returnUri);
  deepLink.searchParams.set("handoff_code", handoffCode);
  return deepLink.toString();
}

export function buildSeedPayload({
  gatewayBaseUrl,
  returnUri,
  handoffCode,
}) {
  const payload = {
    auth: {
      handoff_code: handoffCode,
      bootstrap_deeplink: buildBootstrapDeepLink(returnUri, handoffCode),
    },
    meta: {
      gateway_base_url: gatewayBaseUrl,
      return_uri: returnUri,
    },
  };

  assertOpaqueOnlySeedPayload(payload);
  return payload;
}

export function assertOpaqueOnlySeedPayload(value, path = "$") {
  if (Array.isArray(value)) {
    value.forEach((entry, index) =>
      assertOpaqueOnlySeedPayload(entry, `${path}[${index}]`),
    );
    return;
  }

  if (value && typeof value === "object") {
    for (const [key, entry] of Object.entries(value)) {
      if (FORBIDDEN_OUTPUT_KEYS.has(key)) {
        throw new Error(`${path}.${key} must not appear in seed output`);
      }

      assertOpaqueOnlySeedPayload(entry, `${path}.${key}`);
    }
    return;
  }

  if (typeof value !== "string") {
    return;
  }

  if (JWT_LIKE_PATTERN.test(value)) {
    throw new Error(`${path} must not contain a JWT-like value`);
  }
}

function isRedirectStatus(status) {
  return status >= 300 && status < 400;
}

async function readJsonResponse(response) {
  const text = await response.text();
  if (!text) {
    return null;
  }

  try {
    return JSON.parse(text);
  } catch (error) {
    throw new Error(
      `expected JSON response from ${response.url}, got: ${text.slice(0, 200)}`,
    );
  }
}

export async function ensureGatewayReady(gatewayBaseUrl) {
  const healthUrl = new URL("/health/ready", gatewayBaseUrl);
  const response = await fetch(healthUrl, {
    headers: { accept: "application/json" },
  });

  if (!response.ok) {
    throw new Error(
      `gateway readiness check failed at ${healthUrl}: status ${response.status}`,
    );
  }

  const body = await readJsonResponse(response);
  if (body?.service !== "gateway" || body?.status !== "ready") {
    throw new Error(
      `expected gateway readiness response at ${healthUrl}, got ${JSON.stringify(body)}`,
    );
  }
}

export async function mintHandoffSeed({
  gatewayBaseUrl = DEFAULT_GATEWAY_BASE_URL,
  returnUri = DEFAULT_RETURN_URI,
  authCode = DEFAULT_AUTH_CODE,
} = {}) {
  await ensureGatewayReady(gatewayBaseUrl);

  const loginUrl = new URL("/auth/login", gatewayBaseUrl);
  loginUrl.searchParams.set("return_uri", returnUri);

  const loginResponse = await fetch(loginUrl, {
    method: "GET",
    redirect: "manual",
  });

  if (!isRedirectStatus(loginResponse.status)) {
    throw new Error(
      `expected login redirect from ${loginUrl}, got status ${loginResponse.status}`,
    );
  }

  const loginLocation = loginResponse.headers.get("location");
  if (!loginLocation) {
    throw new Error("gateway login redirect did not set a location header");
  }

  const loginState = extractStateFromAuthorizationRedirect(loginLocation);
  const callbackUrl = new URL("/auth/callback", gatewayBaseUrl);
  callbackUrl.searchParams.set("code", authCode);
  callbackUrl.searchParams.set("state", loginState);

  const callbackResponse = await fetch(callbackUrl, {
    method: "GET",
    redirect: "manual",
  });

  if (!isRedirectStatus(callbackResponse.status)) {
    throw new Error(
      `expected callback redirect from ${callbackUrl}, got status ${callbackResponse.status}`,
    );
  }

  if (callbackResponse.headers.get("set-cookie")) {
    throw new Error("mobile callback must not emit cookies");
  }

  const callbackLocation = callbackResponse.headers.get("location");
  if (!callbackLocation) {
    throw new Error("gateway callback redirect did not set a location header");
  }

  const handoffCode = extractHandoffCodeFromMobileRedirect(
    callbackLocation,
    returnUri,
  );

  return buildSeedPayload({
    gatewayBaseUrl,
    returnUri,
    handoffCode,
  });
}

export function extractStateFromAuthorizationRedirect(location) {
  const url = new URL(location);
  const state = url.searchParams.get("state")?.trim();

  if (!state) {
    throw new Error("authorization redirect did not include a non-empty state");
  }

  return state;
}

export function extractHandoffCodeFromMobileRedirect(location, returnUri) {
  const callbackUrl = new URL(location);
  const expectedReturnUrl = new URL(returnUri);
  expectedReturnUrl.search = "";
  expectedReturnUrl.hash = "";
  callbackUrl.hash = "";

  const comparableCallbackUrl = new URL(callbackUrl.toString());
  comparableCallbackUrl.search = "";

  if (comparableCallbackUrl.toString() !== expectedReturnUrl.toString()) {
    throw new Error(
      `expected callback redirect to return to ${expectedReturnUrl}, got ${comparableCallbackUrl}`,
    );
  }

  const pairs = [...callbackUrl.searchParams.entries()];
  if (pairs.length !== 1 || pairs[0][0] !== "handoff_code") {
    throw new Error("mobile callback must return only handoff_code");
  }

  const handoffCode = pairs[0][1].trim();
  if (!handoffCode) {
    throw new Error("mobile callback returned an empty handoff_code");
  }

  return handoffCode;
}

export async function redeemHandoffCode({ gatewayBaseUrl, handoffCode }) {
  const response = await fetch(new URL("/auth/mobile/session", gatewayBaseUrl), {
    method: "POST",
    headers: {
      "content-type": "application/json",
      accept: "application/json",
    },
    body: JSON.stringify({ handoff_code: handoffCode }),
  });

  return {
    status: response.status,
    body: await readJsonResponse(response),
  };
}

export async function verifySingleUseRedemption({
  gatewayBaseUrl,
  handoffCode,
}) {
  const first = await redeemHandoffCode({ gatewayBaseUrl, handoffCode });

  if (first.status !== 200) {
    throw new Error(
      `expected first handoff redemption to succeed, got status ${first.status}`,
    );
  }

  const firstKeys = Object.keys(first.body ?? {});
  if (firstKeys.length !== 1 || firstKeys[0] !== "session_ref") {
    throw new Error("first redemption must return only session_ref");
  }

  const sessionRef = first.body?.session_ref?.trim();
  if (!sessionRef) {
    throw new Error("first redemption returned an empty session_ref");
  }

  assertOpaqueOnlySeedPayload({ session_ref: sessionRef });

  const second = await redeemHandoffCode({ gatewayBaseUrl, handoffCode });

  if (second.status !== 401) {
    throw new Error(
      `expected second handoff redemption to return 401, got ${second.status}`,
    );
  }

  if (second.body?.session_ref) {
    throw new Error("second redemption must not return session_ref");
  }

  return {
    first_redeem_status: first.status,
    second_redeem_status: second.status,
  };
}

function parseArgs(argv) {
  const options = {
    authCode: DEFAULT_AUTH_CODE,
    gatewayBaseUrl: DEFAULT_GATEWAY_BASE_URL,
    returnUri: DEFAULT_RETURN_URI,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];

    if (argument === "--help") {
      options.help = true;
      continue;
    }

    if (argument === "--gateway-base-url") {
      options.gatewayBaseUrl = argv[index + 1];
      index += 1;
      continue;
    }

    if (argument === "--return-uri") {
      options.returnUri = argv[index + 1];
      index += 1;
      continue;
    }

    if (argument === "--auth-code") {
      options.authCode = argv[index + 1];
      index += 1;
      continue;
    }

    throw new Error(`unrecognized argument: ${argument}`);
  }

  return options;
}

function printHelp() {
  console.error(`Usage: node scripts/e2e-seed/mint-handoff-code.mjs [options]

Options:
  --gateway-base-url <url>  Gateway base URL (default: ${DEFAULT_GATEWAY_BASE_URL})
  --return-uri <uri>        Mobile return URI (default: ${DEFAULT_RETURN_URI})
  --auth-code <code>        OAuth code value passed to /auth/callback (default: ${DEFAULT_AUTH_CODE})
  --help                    Show this help message
`);
}

function formatError(error) {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

export async function runCli(argv = process.argv.slice(2)) {
  const options = parseArgs(argv);

  if (options.help) {
    printHelp();
    return 0;
  }

  const payload = await mintHandoffSeed(options);
  process.stdout.write(`${JSON.stringify(payload, null, 2)}\n`);
  return 0;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runCli().then(
    (code) => {
      process.exit(code);
    },
    (error) => {
      const message = formatError(error);
      process.stderr.write(`[mint-handoff-code] ${message}\n`);
      process.exit(1);
    },
  );
}
