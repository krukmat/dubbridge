#!/usr/bin/env bash
set -euo pipefail

REV="${1:-}"
REGISTRY="${2:-}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT}/artifacts/releases/${REV}"

fail() {
  echo "DO_RELEASE=BLOCKED reason=$1" >&2
  exit 2
}

[[ "${REV}" =~ ^[0-9a-f]{40}$ ]] || fail "REV-must-be-full-git-sha"
[[ -n "${REGISTRY}" ]] || fail "missing-registry"
command -v docker >/dev/null 2>&1 || fail "missing-docker"
command -v jq >/dev/null 2>&1 || fail "missing-jq"
docker buildx version >/dev/null 2>&1 || fail "missing-docker-buildx"
git -C "${ROOT}" cat-file -e "${REV}^{commit}" 2>/dev/null || fail "unknown-revision"

CURRENT="$(git -C "${ROOT}" rev-parse HEAD)"
[[ "${CURRENT}" == "${REV}" ]] || fail "checkout-does-not-match-REV"
[[ -z "$(git -C "${ROOT}" status --porcelain --untracked-files=no)" ]] || fail "dirty-tracked-worktree"

mkdir -p "${OUT_DIR}"

services=(
  "api:apps/api/Dockerfile:dubbridge-api"
  "gateway:apps/gateway/Dockerfile:dubbridge-gateway"
  "migration:apps/cli/Dockerfile:dubbridge-migration"
  "worker:apps/worker-runner/Dockerfile:dubbridge-worker-runner"
  "availability:apps/availability-node/Dockerfile:dubbridge-availability-node"
)

echo "DO_RELEASE_BUILD_START=${REV}"

for spec in "${services[@]}"; do
  IFS=: read -r service dockerfile image <<<"${spec}"
  tag="${REGISTRY}/${image}:${REV}"
  echo "DO_RELEASE_BUILD=${service}"
  docker buildx build --platform linux/amd64 --file "${ROOT}/${dockerfile}" --tag "${tag}" --push "${ROOT}" >/dev/null

  digest="$(docker buildx imagetools inspect "${tag}" --format '{{json .Manifest}}' | jq -r '.digest // empty')"
  [[ "${digest}" =~ ^sha256:[0-9a-f]{64}$ ]] || fail "digest-resolution-${service}"
  printf '%s@%s\n' "${REGISTRY}/${image}" "${digest}" > "${OUT_DIR}/${service}.ref"
done

API_REF="$(cat "${OUT_DIR}/api.ref")"
GATEWAY_REF="$(cat "${OUT_DIR}/gateway.ref")"
MIGRATION_REF="$(cat "${OUT_DIR}/migration.ref")"
WORKER_REF="$(cat "${OUT_DIR}/worker.ref")"
AVAILABILITY_REF="$(cat "${OUT_DIR}/availability.ref")"

compose_hash="$(
  DUBBRIDGE_API_IMAGE="${API_REF}"   DUBBRIDGE_GATEWAY_IMAGE="${GATEWAY_REF}"   DUBBRIDGE_MIGRATION_IMAGE="${MIGRATION_REF}"   DUBBRIDGE_WORKER_IMAGE="${WORKER_REF}"   DUBBRIDGE_AVAILABILITY_IMAGE="${AVAILABILITY_REF}"   DUBBRIDGE_DATABASE_URL=x   DUBBRIDGE_REDIS_URL=x   DUBBRIDGE_STORAGE__ACCESS_KEY_ID=x   DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=x   DUBBRIDGE_AUTH__JWT_SECRET=x   DUBBRIDGE_P2P_KEK_HEX=x   DUBBRIDGE_P2P_KEK_ID=x   DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET=x   DUBBRIDGE_TRANSLATION_API_URL=x   DUBBRIDGE_TRANSLATION_API_KEY=x   DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS=x   DUBBRIDGE_P2P_MTLS_DIR=/tmp/p2p-mtls   docker compose -f "${ROOT}/infra/production/docker-compose.yml" config   | shasum -a 256 | awk '{print $1}'
)"

created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
release_id="${REV:0:12}-${compose_hash:0:12}"

jq -n   --arg release_id "${release_id}"   --arg created_at "${created_at}"   --arg git_sha "${REV}"   --arg compose_sha256 "${compose_hash}"   --arg registry "${REGISTRY}"   --arg api "${API_REF}"   --arg gateway "${GATEWAY_REF}"   --arg migration "${MIGRATION_REF}"   --arg worker "${WORKER_REF}"   --arg availability "${AVAILABILITY_REF}"   '{schema_version:1,release_id:$release_id,created_at:$created_at,git_sha:$git_sha,compose_sha256:$compose_sha256,registry:$registry,images:{api:$api,gateway:$gateway,migration:$migration,worker_runner:$worker,availability_node:$availability}}'   > "${OUT_DIR}/release.json"

jq -e '[.images[] | test("@sha256:[0-9a-f]{64}$")] | all' "${OUT_DIR}/release.json" >/dev/null || fail "manifest-not-digest-pinned"

echo "DO_RELEASE_ID=${release_id}"
echo "DO_RELEASE_MANIFEST=${OUT_DIR}/release.json"
echo "DO_RELEASE=PASS"
