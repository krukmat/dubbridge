import { test } from "node:test";
import assert from "node:assert/strict";

async function loadMtls() {
  const mod = await import("../dist/mtls.js");
  return mod;
}

test("HP: normalizeSha256Fingerprint and createClientFingerprintPolicy accept exact identities", async () => {
  const { normalizeSha256Fingerprint, createClientFingerprintPolicy } = await loadMtls();

  const compact1 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
  const compact2 = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

  // Derive canonical Node representation of the first by uppercasing it and inserting a colon between every byte
  const canonical1 = compact1.toUpperCase().replace(/(.{2})/g, "$1:").replace(/:$/, "");

  // Assert the canonical colon-delimited Node representation and the compact lowercase value normalize to the same compact lowercase fingerprint
  assert.equal(normalizeSha256Fingerprint(canonical1), compact1);
  assert.equal(normalizeSha256Fingerprint(compact1), compact1);

  // Build a policy from both distinct pins and assert it accepts each exact identity
  const policy = createClientFingerprintPolicy([compact1, compact2]);
  assert.equal(policy(compact1), true);
  assert.equal(policy(compact2), true);
  assert.equal(policy(canonical1), true);
});

test("EC: policy rejects undefined, empty string, malformed text, and unlisted valid fingerprint", async () => {
  const { createClientFingerprintPolicy } = await loadMtls();

  const compact1 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
  const compact2 = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
  const compact3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

  const policy = createClientFingerprintPolicy([compact1, compact2]);

  assert.equal(policy(undefined), false);
  assert.equal(policy(""), false);
  assert.equal(policy("malformed"), false);
  assert.equal(policy(compact3), false);
});

test("Configuration fail-closed: createClientFingerprintPolicy throws for empty list, malformed pins, and duplicates", async () => {
  const { createClientFingerprintPolicy } = await loadMtls();

  // Missing configuration
  assert.throws(() => createClientFingerprintPolicy());

  // Empty list
  assert.throws(() => createClientFingerprintPolicy([]));

  // Malformed pins
  assert.throws(() => createClientFingerprintPolicy(["malformed"]));

  // Duplicates after normalization (compact pin plus its colon-delimited equivalent)
  const compact1 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
  const canonical1 = compact1.toUpperCase().replace(/(.{2})/g, "$1:").replace(/:$/, "");
  assert.throws(() => createClientFingerprintPolicy([compact1, canonical1]));
});
