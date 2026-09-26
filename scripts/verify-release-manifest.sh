#!/usr/bin/env bash
set -euo pipefail
manifest="${1:?usage: verify-release-manifest.sh <release.json>}"
command -v jq >/dev/null 2>&1 || { echo "RELEASE_VERIFY=BLOCKED reason=missing-jq" >&2; exit 2; }
jq -e '.schema_version == 1 and (.release_id|type=="string" and length>0) and (.created_at|type=="string" and length>0) and (.git_sha|test("^[0-9a-f]{40}$")) and (.compose_sha256|test("^[0-9a-f]{64}$")) and (.images|length==5) and ([.images[]|test("@sha256:[0-9a-f]{64}$")]|all)' "${manifest}" >/dev/null
echo "RELEASE_VERIFY=PASS"
