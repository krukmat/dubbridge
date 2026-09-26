import { test } from "node:test";
import assert from "node:assert/strict";
import { withPublicationLock } from "../dist/publication_lock.js";

function deferred() {
  let resolve;
  const promise = new Promise((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

test("HP-1: two concurrent calls with the same key run strictly one at a time, in call order", async () => {
  const order = [];
  const first = deferred();

  const callA = withPublicationLock("pub-1", async () => {
    order.push("a-start");
    await first.promise;
    order.push("a-end");
    return "a";
  });

  // Give callA a chance to actually start (acquire the lock) before callB
  // is issued, without resolving it yet.
  await new Promise((r) => setTimeout(r, 10));

  const callB = withPublicationLock("pub-1", async () => {
    order.push("b-start");
    order.push("b-end");
    return "b";
  });

  // callB must not have started yet: the lock is still held by callA.
  await new Promise((r) => setTimeout(r, 10));
  assert.deepEqual(order, ["a-start"]);

  first.resolve();
  const [resultA, resultB] = await Promise.all([callA, callB]);

  assert.equal(resultA, "a");
  assert.equal(resultB, "b");
  assert.deepEqual(order, ["a-start", "a-end", "b-start", "b-end"]);
});

test("HP-2: calls with different keys run fully in parallel, not serialized", async () => {
  const order = [];
  const firstKeyGate = deferred();

  const callX = withPublicationLock("pub-x", async () => {
    order.push("x-start");
    await firstKeyGate.promise;
    order.push("x-end");
    return "x";
  });

  await new Promise((r) => setTimeout(r, 10));

  // A different key must start immediately, even though pub-x's call is
  // still in flight and hasn't released its lock yet.
  const callY = withPublicationLock("pub-y", async () => {
    order.push("y-start");
    order.push("y-end");
    return "y";
  });

  const resultY = await callY;
  assert.equal(resultY, "y");
  assert.deepEqual(order, ["x-start", "y-start", "y-end"]);

  firstKeyGate.resolve();
  const resultX = await callX;
  assert.equal(resultX, "x");
});

test("EC-1: a throwing call releases the lock so the next queued call still runs", async () => {
  const order = [];

  const callA = withPublicationLock("pub-err", async () => {
    order.push("a-start");
    throw new Error("boom");
  });
  // Attach a handler in the same tick to avoid a spurious
  // unhandledRejection warning; the real assertion happens below via
  // assert.rejects on the same promise.
  callA.catch(() => {});

  await new Promise((r) => setTimeout(r, 5));

  const callB = withPublicationLock("pub-err", async () => {
    order.push("b-start");
    return "b";
  });

  await assert.rejects(callA, /boom/);
  const resultB = await callB;

  assert.equal(resultB, "b");
  assert.deepEqual(order, ["a-start", "b-start"]);
});

test("EC-2: three queued calls for the same key run in strict FIFO order", async () => {
  const order = [];

  const make = (label) =>
    withPublicationLock("pub-fifo", async () => {
      order.push(label);
      await new Promise((r) => setTimeout(r, 5));
      return label;
    });

  const results = await Promise.all([make("first"), make("second"), make("third")]);

  assert.deepEqual(results, ["first", "second", "third"]);
  assert.deepEqual(order, ["first", "second", "third"]);
});

test("EC-3: a mid-queue throw does not disrupt the calls queued before or after it", async () => {
  const order = [];

  const callA = withPublicationLock("pub-mid-throw", async () => {
    order.push("a-start");
    await new Promise((r) => setTimeout(r, 5));
    order.push("a-end");
    return "a";
  });

  const callB = withPublicationLock("pub-mid-throw", async () => {
    order.push("b-start");
    throw new Error("mid-queue boom");
  });
  callB.catch(() => {});

  const callC = withPublicationLock("pub-mid-throw", async () => {
    order.push("c-start");
    return "c";
  });

  const resultA = await callA;
  await assert.rejects(callB, /mid-queue boom/);
  const resultC = await callC;

  assert.equal(resultA, "a");
  assert.equal(resultC, "c");
  assert.deepEqual(order, ["a-start", "a-end", "b-start", "c-start"]);
});
