import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  parsePublicationRequest,
  parsePublicationResponse,
  serializePublicationBody,
} from "../src/contract.js";

const fixtureUrl = new URL(
  "../../../docs/fixtures/mvp0-p2p-publication-contract-v1.json",
  import.meta.url,
);
const fixture = JSON.parse(readFileSync(fixtureUrl, "utf8")).availability_node;

function expectRejected(candidate: unknown): void {
  assert.throws(
    () => parsePublicationRequest(candidate, fixture.request.publication_id),
    (error: unknown) => {
      assert.equal((error as { name?: string }).name, "PublicationContractError");
      assert.equal((error as { code?: string }).code, "invalid_contract");
      return true;
    },
  );
}

test("T6d rejects every frozen secret-bearing publication field", () => {
  for (const field of fixture.secret_deny_list as string[]) {
    expectRejected({ ...fixture.request, [field]: "must-not-cross-boundary" });
  }
});

test("T6d publication request remains metadata-only", () => {
  const request = parsePublicationRequest(
    fixture.request,
    fixture.request.publication_id,
  );
  assert.deepEqual(Object.keys(request).sort(), [
    "contract_version",
    "lineage_id",
    "manifest_digest_sha256",
    "manifest_version",
    "package_ref",
    "publication_id",
  ]);
});

test("T6d serialized success evidence cannot carry secret extensions", () => {
  const request = parsePublicationRequest(
    fixture.request,
    fixture.request.publication_id,
  );
  const response = parsePublicationResponse(
    fixture.success_first_publish.http_status,
    fixture.success_first_publish.body,
    request,
  );
  const serialized = serializePublicationBody(response, request);
  const parsed = JSON.parse(serialized) as Record<string, unknown>;

  assert.deepEqual(Object.keys(parsed).sort(), [
    "confirmed_at",
    "contract_version",
    "evidence_id",
    "external_publication_id",
    "lineage_id",
    "manifest_digest_sha256",
    "publication_id",
  ]);
  for (const forbidden of fixture.secret_deny_list as string[]) {
    assert.equal(forbidden in parsed, false);
  }
});
