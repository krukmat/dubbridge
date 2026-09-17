import assert from "node:assert/strict";
import test from "node:test";

import { loadAvailabilityNodeConfig } from "../dist/bootstrap.js";

function baseEnv() {
  return {
    DUBBRIDGE_P2P_CIPHERTEXT_ROOT: "/tmp/packages",
    DUBBRIDGE_P2P_AVAILABILITY_DRIVE_ROOT: "/tmp/drives",
    DUBBRIDGE_P2P_AVAILABILITY_INDEX_ROOT: "/tmp/index",
    DUBBRIDGE_P2P_AVAILABILITY_SERVER_KEY_PEM: "/tmp/server-key.pem",
    DUBBRIDGE_P2P_AVAILABILITY_SERVER_CERT_PEM: "/tmp/server-cert.pem",
    DUBBRIDGE_P2P_AVAILABILITY_CA_PEM: "/tmp/ca.pem",
    DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS: "aa:bb, ccdd",
  };
}

test("bootstrap config uses private-local defaults and preserves required roots", () => {
  const config = loadAvailabilityNodeConfig(baseEnv());

  assert.equal(config.bindHost, "127.0.0.1");
  assert.equal(config.port, 8443);
  assert.equal(config.packageRoot, "/tmp/packages");
  assert.equal(config.driveStorageRoot, "/tmp/drives");
  assert.equal(config.indexRoot, "/tmp/index");
  assert.equal(config.hyperswarmJoinTimeoutMs, 15_000);
  assert.deepEqual(config.allowedClientFingerprints, ["aa:bb", "ccdd"]);
});

test("bootstrap config accepts explicit container bind and port", () => {
  const config = loadAvailabilityNodeConfig({
    ...baseEnv(),
    DUBBRIDGE_P2P_AVAILABILITY_BIND_HOST: "0.0.0.0",
    DUBBRIDGE_P2P_AVAILABILITY_PORT: "9443",
    DUBBRIDGE_P2P_HYPERSWARM_JOIN_TIMEOUT_MS: "25000",
  });

  assert.equal(config.bindHost, "0.0.0.0");
  assert.equal(config.port, 9443);
  assert.equal(config.hyperswarmJoinTimeoutMs, 25_000);
});

test("bootstrap config fails closed when required trust/package config is absent", () => {
  const env = baseEnv();
  delete env.DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS;

  assert.throws(
    () => loadAvailabilityNodeConfig(env),
    /DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS is required/
  );
});

test("bootstrap config rejects invalid ports", () => {
  assert.throws(
    () =>
      loadAvailabilityNodeConfig({
        ...baseEnv(),
        DUBBRIDGE_P2P_AVAILABILITY_PORT: "65536",
      }),
    /outside the supported range/
  );
});
