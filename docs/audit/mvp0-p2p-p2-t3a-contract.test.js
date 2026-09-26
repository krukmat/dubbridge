const assert = require("node:assert/strict");
const { readFile } = require("node:fs/promises");
const test = require("node:test");

async function context() {
  const contract = await import("../../apps/availability-node/dist/contract.js");
  const fixture = JSON.parse(
    await readFile("docs/fixtures/mvp0-p2p-publication-contract-v1.json", "utf8"),
  ).availability_node;
  return { contract, fixture };
}

function rejectsWith(contract, operation, code) {
  assert.throws(
    operation,
    (error) => error instanceof contract.PublicationContractError && error.code === code,
  );
}

test("HP-1 frozen request and evidence round-trip", async () => {
  const { contract, fixture } = await context();
  const request = contract.parsePublicationRequest(
    fixture.request,
    fixture.request.publication_id,
  );
  assert.deepEqual(request, fixture.request);
  assert.deepEqual(
    contract.parsePublicationResponse(
      fixture.success_first_publish.http_status,
      fixture.success_first_publish.body,
      request,
    ),
    { status: 201, evidence: fixture.success_first_publish.body },
  );
  assert.deepEqual(
    contract.parsePublicationResponse(200, fixture.success_first_publish.body, request),
    { status: 200, evidence: fixture.success_first_publish.body },
  );
  for (const { http_status: status, code } of fixture.errors) {
    assert.deepEqual(
      contract.parsePublicationResponse(status, contract.createPublicationError(code).body),
      contract.createPublicationError(code),
    );
  }
});

test("EC-1 secret fields and unsafe package references fail closed", async () => {
  const { contract, fixture } = await context();
  for (const field of fixture.secret_deny_list) {
    rejectsWith(
      contract,
      () => contract.parsePublicationRequest(
        { ...fixture.request, [field]: "redacted" },
        fixture.request.publication_id,
      ),
      "invalid_contract",
    );
  }
  for (const package_ref of [
    "", "/absolute", "back\\slash", ".", "..", "a/./b", "a/../b",
    "a//b", "a/", "C:/drive", "a\u0000b", "a\u001fb", "cafe\u0301/file",
  ]) {
    rejectsWith(
      contract,
      () => contract.parsePublicationRequest({ ...fixture.request, package_ref }),
      "package_invalid",
    );
  }
  rejectsWith(
    contract,
    () => contract.parsePublicationRequest({
      ...fixture.request,
      publication_id: "AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA",
    }),
    "invalid_contract",
  );
});
