import { test } from "node:test";
import assert from "node:assert/strict";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

async function loadContainment() {
  const mod = await import("../dist/containment.js");
  return mod;
}

test("HP-S3-1: a relative path with no symlinks resolves successfully", async () => {
  const { verifyContainedRealpath } = await loadContainment();
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "containment-test-"));
  try {
    fs.mkdirSync(path.join(dir, "sub"));
    const result = verifyContainedRealpath(dir, "sub");
    assert.strictEqual(result, path.join(dir, "sub"));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("HP-S3-2: a relative path whose final component does not exist yet still resolves", async () => {
  const { verifyContainedRealpath } = await loadContainment();
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "containment-test-"));
  try {
    const result = verifyContainedRealpath(dir, "not-yet-created/file.txt");
    assert.strictEqual(result, path.join(dir, "not-yet-created/file.txt"));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-S3-1: a symlink pointing outside root is rejected", async () => {
  const { verifyContainedRealpath, ContainmentError } = await loadContainment();
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "containment-test-"));
  const outside = fs.mkdtempSync(path.join(os.tmpdir(), "containment-test-"));
  try {
    fs.writeFileSync(path.join(outside, "secret.txt"), "x");
    fs.symlinkSync(path.join(outside, "secret.txt"), path.join(dir, "link"));
    assert.throws(() => verifyContainedRealpath(dir, "link"), ContainmentError);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
    fs.rmSync(outside, { recursive: true, force: true });
  }
});

test("EC-S3-2: a dangling symlink is rejected even though its target does not exist", async () => {
  const { verifyContainedRealpath, ContainmentError } = await loadContainment();
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "containment-test-"));
  try {
    fs.symlinkSync(path.join(dir, "nonexistent-target"), path.join(dir, "danglink"));
    assert.throws(() => verifyContainedRealpath(dir, "danglink"), ContainmentError);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-S3-3: a non-existent root throws ContainmentError, not a raw fs error", async () => {
  const { verifyContainedRealpath, ContainmentError } = await loadContainment();
  const nonExistentRoot = path.join(os.tmpdir(), "containment-test-does-not-exist-" + Date.now());
  assert.throws(() => verifyContainedRealpath(nonExistentRoot, "anything"), ContainmentError);
});

test("EC-S3-4: a symlinked directory earlier in the path escapes containment even without a direct component symlink", async () => {
  const { verifyContainedRealpath, ContainmentError } = await loadContainment();
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "containment-test-"));
  const outside = fs.mkdtempSync(path.join(os.tmpdir(), "containment-test-"));
  try {
    fs.mkdirSync(path.join(dir, "real-sub"));
    fs.mkdirSync(path.join(outside, "nested"));
    fs.writeFileSync(path.join(outside, "nested", "file.txt"), "x");
    fs.symlinkSync(path.join(outside, "nested"), path.join(dir, "alias"));
    assert.throws(() => verifyContainedRealpath(dir, "alias/file.txt"), ContainmentError);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
    fs.rmSync(outside, { recursive: true, force: true });
  }
});
