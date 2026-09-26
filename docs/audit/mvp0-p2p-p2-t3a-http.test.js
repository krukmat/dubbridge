const assert = require("node:assert/strict");
const { once } = require("node:events");
const { readFile } = require("node:fs/promises");
const { createServer, request: httpRequest } = require("node:http");
const test = require("node:test");

async function context() {
  const serverModule = await import("../../apps/availability-node/dist/server.js");
  const contract = await import("../../apps/availability-node/dist/contract.js");
  const fixture = JSON.parse(
    await readFile("docs/fixtures/mvp0-p2p-publication-contract-v1.json", "utf8"),
  ).availability_node;
  return { serverModule, contract, fixture };
}

async function send(serverModule, fixture, executor, options = {}) {
  const handler = executor === undefined
    ? serverModule.createPublicationHandler()
    : serverModule.createPublicationHandler(executor);
  const server = createServer(handler);
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  assert.equal(typeof address, "object");
  const body = options.rawBody ?? JSON.stringify(options.body ?? fixture.request);
  const headers = options.headers ?? { "content-type": "application/json; charset=utf-8" };
  try {
    return await new Promise((resolve, reject) => {
      const request = httpRequest({
        host: "127.0.0.1",
        port: address.port,
        method: options.method ?? "PUT",
        path: options.path ?? `/v1/publications/${fixture.request.publication_id}`,
        headers,
      }, (response) => {
        const chunks = [];
        response.on("data", (chunk) => chunks.push(chunk));
        response.on("end", () => resolve({
          status: response.statusCode,
          body: JSON.parse(Buffer.concat(chunks).toString("utf8")),
        }));
      });
      request.on("error", reject);
      request.end(body);
    });
  } finally {
    server.close();
    await once(server, "close");
  }
}

test("HP-2 validated ingress invokes executor once and emits 201 or 200", async () => {
  const { serverModule, fixture } = await context();
  for (const status of [201, 200]) {
    let calls = 0;
    const response = await send(serverModule, fixture, async () => {
      calls += 1;
      return { status, evidence: fixture.success_first_publish.body };
    });
    assert.equal(calls, 1);
    assert.equal(response.status, status);
    assert.deepEqual(response.body, fixture.success_first_publish.body);
  }
});

test("EC-2 invalid ingress and ambiguous executor results fail closed", async () => {
  const { serverModule, contract, fixture } = await context();
  let calls = 0;
  const countingExecutor = async () => {
    calls += 1;
    return { status: 201, evidence: fixture.success_first_publish.body };
  };
  for (const options of [
    { method: "POST" },
    { path: `/v1/publications/${fixture.request.publication_id}?query=forbidden` },
    { rawBody: "{" },
    { body: { ...fixture.request, package_ref: "../escape" } },
    { body: { ...fixture.request, plaintext_ck: "redacted" } },
  ]) {
    const response = await send(serverModule, fixture, countingExecutor, options);
    assert.ok(response.status === 400 || response.status === 422);
  }
  assert.equal(calls, 0);

  for (const executor of [
    async () => { throw new Error("opaque"); },
    async () => ({
      status: 201,
      evidence: { ...fixture.success_first_publish.body, extra: "forbidden" },
    }),
    async () => ({
      status: 201,
      evidence: {
        ...fixture.success_first_publish.body,
        lineage_id: "44444444-4444-4444-8444-444444444444",
      },
    }),
  ]) {
    const response = await send(serverModule, fixture, executor);
    assert.equal(response.status, 503);
    assert.deepEqual(response.body, contract.createPublicationError("publication_unavailable").body);
  }
  assert.equal((await send(serverModule, fixture, undefined)).status, 503);
});
