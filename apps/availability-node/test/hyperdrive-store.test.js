import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { openDrive, closeDrive, closeSharedStore } from "../dist/hyperdrive_store.js";

test("HP-1: opening a drive for a new publication_id creates a fresh, ready drive", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-store-hp1-"));
  const opened = await openDrive(root, "11111111-1111-1111-1111-111111111111");

  assert.equal(typeof opened.publicKeyHex, "string");
  assert.equal(opened.publicKeyHex.length, 64);
  assert.equal(opened.drive.key.toString("hex"), opened.publicKeyHex);

  await closeDrive(opened);
  await closeSharedStore(root);
});

test("HP-2: reopening the same publication_id after a simulated restart returns the same drive key and persisted content", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-store-hp2-"));
  const publicationId = "22222222-2222-2222-2222-222222222222";

  const first = await openDrive(root, publicationId);
  await first.drive.put("/manifest.json", Buffer.from("frozen-manifest"));
  await closeDrive(first);
  // Simulate a full process restart: the shared store for this root is
  // closed too, so the next openDrive call must construct a brand-new
  // Corestore instance from disk, not reuse an in-memory one.
  await closeSharedStore(root);

  const second = await openDrive(root, publicationId);

  assert.equal(second.publicKeyHex, first.publicKeyHex);
  const content = await second.drive.get("/manifest.json");
  assert.equal(content?.toString(), "frozen-manifest");

  await closeDrive(second);
  await closeSharedStore(root);
});

test("EC-1: two different publication_ids under the same root open two distinct drives sharing one store", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-store-ec1-"));

  const a = await openDrive(root, "33333333-3333-3333-3333-333333333333");
  const b = await openDrive(root, "44444444-4444-4444-4444-444444444444");

  assert.notEqual(a.publicKeyHex, b.publicKeyHex);

  await closeDrive(a);
  await closeDrive(b);
  await closeSharedStore(root);
});

test("EC-2: an unusable storage root rejects cleanly without leaving a dangling shared store entry", async () => {
  const unusableRoot = "/dev/null/cannot-mkdir-under-a-file";

  await assert.rejects(
    () => openDrive(unusableRoot, "55555555-5555-5555-5555-555555555555"),
    (err) => {
      assert.ok(err instanceof Error);
      return true;
    }
  );

  // A subsequent call against a *valid* root must still work — the failed
  // attempt must not have left a broken entry keyed by some other path, or
  // otherwise corrupted shared process state.
  const validRoot = mkdtempSync(join(tmpdir(), "hd-store-ec2-"));
  const opened = await openDrive(validRoot, "66666666-6666-6666-6666-666666666666");
  assert.equal(opened.publicKeyHex.length, 64);
  await closeDrive(opened);
  await closeSharedStore(validRoot);
});
