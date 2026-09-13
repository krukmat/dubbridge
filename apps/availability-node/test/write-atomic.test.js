import { test } from "node:test";
import assert from "node:assert/strict";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

async function loadWriteAtomic() {
  const mod = await import("../dist/write_atomic.js");
  return mod;
}

test("HP-1: write_atomic writes content correctly", async () => {
  const { writeFileAtomic } = await loadWriteAtomic();
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "write-atomic-test-"));
  const targetPath = path.join(tmpDir, "test.txt");
  const contentStr = "Hello, World!";
  const content = Buffer.from(contentStr);

  await writeFileAtomic(targetPath, content);

  const written = fs.readFileSync(targetPath, "utf-8");
  assert.equal(written, contentStr);
});

test("HP-2: write_atomic overwrites existing file", async () => {
  const { writeFileAtomic } = await loadWriteAtomic();
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "write-atomic-test-"));
  const targetPath = path.join(tmpDir, "test.txt");

  // Create initial file
  fs.writeFileSync(targetPath, "Initial Content");

  // Overwrite with new content
  const newContentStr = "New Content";
  const newContent = Buffer.from(newContentStr);
  await writeFileAtomic(targetPath, newContent);

  const written = fs.readFileSync(targetPath, "utf-8");
  assert.equal(written, newContentStr);
});

test("EC-1: write_atomic cleans up on failure", async () => {
  const { writeFileAtomic } = await loadWriteAtomic();
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "write-atomic-test-"));

  // Target a path where the parent directory does not exist
  const targetPath = path.join(tmpDir, "missing-subdir", "file.txt");
  const content = Buffer.from("Should not be written");

  // This should fail because 'missing-subdir' does not exist
  await assert.rejects(
    () => writeFileAtomic(targetPath, content),
    (err) => {
      assert.ok(err);
      return true;
    }
  );

  // Assert that no stray temp files were left in the existing tmpDir
  const files = fs.readdirSync(tmpDir);
  assert.deepEqual(files, []);
});
