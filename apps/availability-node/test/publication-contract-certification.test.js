import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const fixtureUrl = new URL(
  "../../../docs/fixtures/mvp0-p2p-publication-contract-v1.json",
  import.meta.url
);
const fixture = JSON.parse(readFileSync(fixtureUrl, "utf8")).availability_node;

async function contract() {
  return import("../dist/contract.js");
}

function expectContractError(assertion, expectedCode) {
  assert.throws(assertion, (error) => {
    assert.equal(error?.name, "PublicationContractError");
    assert.equal(error?.code, expectedCode);
    return true;
  });
}

test("T3d contract certification: frozen request and success response round-trip", async () => {
  const { parsePublicationRequest, parsePublicationResponse } = await contract();
  const request = parsePublicationRequest(
    fixture.request,
    fixture.request.publication_id
  );
  const response = parsePublicationResponse(
    fixture.success_first_publish.http_status,
    fixture.success_first_publish.body,
    request
  );

  assert.equal(response.status, fixture.success_first_publish.http_status);
  assert.deepEqual(response.evidence, fixture.success_first_publish.body);
});

test("T3d secret-deny certification: frozen forbidden publication fields fail closed", async () => {
  const { parsePublicationRequest } = await contract();

  for (const field of fixture.secret_deny_list) {
    const candidate = { ...fixture.request, [field]: "must-not-cross-boundary" };
    expectContractError(
      () => parsePublicationRequest(candidate, fixture.request.publication_id),
      "invalid_contract"
    );
  }
});

test("T3d traversal certification: package_ref cannot escape the ciphertext root", async () => {
  const { parsePublicationRequest } = await contract();
  const invalidRefs = [
    "../outside",
    "packages/../outside",
    "/absolute/path",
    "packages\\outside",
    "packages//outside",
    "C:/outside",
  ];

  for (const packageRef of invalidRefs) {
    expectContractError(
      () =>
        parsePublicationRequest(
          { ...fixture.request, package_ref: packageRef },
          fixture.request.publication_id
        ),
      "package_invalid"
    );
  }
});

test("T3d response certification: success evidence must match the requested lineage and digest", async () => {
  const { parsePublicationRequest, parsePublicationResponse } = await contract();
  const request = parsePublicationRequest(
    fixture.request,
    fixture.request.publication_id
  );

  const wrongLineage = {
    ...fixture.success_first_publish.body,
    lineage_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  };
  expectContractError(
    () => parsePublicationResponse(201, wrongLineage, request),
    "invalid_contract"
  );

  const wrongDigest = {
    ...fixture.success_first_publish.body,
    manifest_digest_sha256: "f".repeat(64),
  };
  expectContractError(
    () => parsePublicationResponse(201, wrongDigest, request),
    "invalid_contract"
  );
});
