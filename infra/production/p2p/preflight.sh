#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/infra/production/docker-compose.yml"
CADDY_FILE="$ROOT_DIR/infra/production/Caddyfile"
IMAGE_TEST="$ROOT_DIR/scripts/test-production-images.sh"
MODE="${1:-all}"

TEMP_DIR=""
ENV_FILE=""
MTLS_DIR=""
PROJECT=""
HEAD_SHA=""
AN_CONTAINER=""
COMPOSE_BACKEND=""

log() { printf '%s\n' "$*"; }
die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
need_cmd() { command -v "$1" >/dev/null 2>&1 || die "$1 not found on PATH"; }

detect_compose() {
    if docker compose version >/dev/null 2>&1; then
        COMPOSE_BACKEND="plugin"
        return 0
    fi
    if command -v docker-compose >/dev/null 2>&1 && docker-compose version >/dev/null 2>&1; then
        COMPOSE_BACKEND="standalone"
        return 0
    fi
    die "Docker Compose unavailable: install Compose v2 (docker compose) or docker-compose"
}

compose_with_env_file() {
    local env_file="$1"
    shift
    if [[ "$COMPOSE_BACKEND" == "plugin" ]]; then
        docker compose --project-name "$PROJECT" --env-file "$env_file" \
            -f "$COMPOSE_FILE" "$@"
    elif [[ "$COMPOSE_BACKEND" == "standalone" ]]; then
        docker-compose --project-name "$PROJECT" --env-file "$env_file" \
            -f "$COMPOSE_FILE" "$@"
    else
        die "Docker Compose backend was not initialized"
    fi
}

cleanup() {
    if [[ -n "$PROJECT" ]] && [[ -n "$COMPOSE_BACKEND" ]] && command -v docker >/dev/null 2>&1 && [[ -n "$ENV_FILE" ]]; then
        compose_with_env_file "$ENV_FILE" down --remove-orphans >/dev/null 2>&1 || true
        docker volume ls -q --filter "label=com.docker.compose.project=$PROJECT" \
            | while IFS= read -r volume; do
                [[ -n "$volume" ]] && docker volume rm "$volume" >/dev/null 2>&1 || true
            done
    fi
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT INT TERM

usage() {
    cat <<'EOF'
Usage: infra/production/p2p/preflight.sh [contract|runtime|all]

  contract  Source-level T6p-c checks; Docker is not required.
  runtime   Render/build/runtime/mTLS/persistence/fail-closed checks; Docker required.
  all       contract + runtime (default).

The runtime lane must run from a clean checkout and emits machine-readable
T6PC_* evidence lines including the exact HEAD and local OCI image ID.
EOF
}

extract_service_block() {
    local service="$1"
    awk -v service="$service" '
      $0 == "  " service ":" { in_service=1; print; next }
      in_service && /^  [A-Za-z0-9_-]+:$/ { exit }
      in_service { print }
    ' "$COMPOSE_FILE"
}

contract_checks() {
    [[ -f "$COMPOSE_FILE" ]] || die "$COMPOSE_FILE missing"
    [[ -f "$ROOT_DIR/apps/availability-node/Dockerfile" ]] || die "Availability Node Dockerfile missing"
    [[ -f "$IMAGE_TEST" ]] || die "$IMAGE_TEST missing"

    (cd "$ROOT_DIR" && bash "$IMAGE_TEST" contract availability >/dev/null)

    local an_block
    an_block="$(extract_service_block "availability-node")"
    [[ -n "$an_block" ]] || die "availability-node service missing from production Compose"

    if printf '%s\n' "$an_block" | grep -q '^    ports:'; then
        die "availability-node exposes a host port"
    fi
    printf '%s\n' "$an_block" | grep -q 'p2p-control' \
        || die "availability-node is not attached to p2p-control"
    printf '%s\n' "$an_block" | grep -q 'cpus: "1.0"' \
        || die "Availability Node 1 CPU ceiling missing"
    printf '%s\n' "$an_block" | grep -q 'mem_limit: 1g' \
        || die "Availability Node 1 GiB ceiling missing"

    if grep -q 'env_file:' "$COMPOSE_FILE"; then
        die "broad env_file injection remains in production Compose"
    fi
    if grep -Eq 'availability-node|:8443' "$CADDY_FILE"; then
        die "Caddy exposes or references the Availability Node"
    fi

    grep -q 'p2p-ciphertext-data:/var/lib/dubbridge/p2p-ciphertext:ro' "$COMPOSE_FILE" \
        || die "Availability Node ciphertext mount is not read-only"
    grep -q 'p2p-ciphertext-data:/var/lib/dubbridge/p2p-ciphertext$' "$COMPOSE_FILE" \
        || die "worker ciphertext RW mount missing"

    log "T6PC_CONTRACT=PASS"
}

generate_mtls() {
    MTLS_DIR="$TEMP_DIR/mtls"
    mkdir -p "$MTLS_DIR"

    openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
        -subj "/CN=DubBridge T6p-c Test CA" \
        -keyout "$MTLS_DIR/ca-key.pem" \
        -out "$MTLS_DIR/ca.pem" >/dev/null 2>&1

    openssl req -newkey rsa:2048 -nodes \
        -subj "/CN=availability-node" \
        -keyout "$MTLS_DIR/server-key.pem" \
        -out "$MTLS_DIR/server.csr" >/dev/null 2>&1
    cat >"$MTLS_DIR/server.ext" <<'EOF'
subjectAltName=DNS:availability-node
extendedKeyUsage=serverAuth
EOF
    openssl x509 -req -days 2 \
        -in "$MTLS_DIR/server.csr" \
        -CA "$MTLS_DIR/ca.pem" \
        -CAkey "$MTLS_DIR/ca-key.pem" \
        -CAcreateserial \
        -extfile "$MTLS_DIR/server.ext" \
        -out "$MTLS_DIR/server-cert.pem" >/dev/null 2>&1

    openssl req -newkey rsa:2048 -nodes \
        -subj "/CN=dubbridge-worker" \
        -keyout "$MTLS_DIR/client-key.pem" \
        -out "$MTLS_DIR/client.csr" >/dev/null 2>&1
    cat >"$MTLS_DIR/client.ext" <<'EOF'
extendedKeyUsage=clientAuth
EOF
    openssl x509 -req -days 2 \
        -in "$MTLS_DIR/client.csr" \
        -CA "$MTLS_DIR/ca.pem" \
        -CAkey "$MTLS_DIR/ca-key.pem" \
        -CAcreateserial \
        -extfile "$MTLS_DIR/client.ext" \
        -out "$MTLS_DIR/client-cert.pem" >/dev/null 2>&1
    cat "$MTLS_DIR/client-cert.pem" "$MTLS_DIR/client-key.pem" \
        >"$MTLS_DIR/client-identity.pem"

    openssl req -newkey rsa:2048 -nodes \
        -subj "/CN=dubbridge-rogue-worker" \
        -keyout "$MTLS_DIR/rogue-key.pem" \
        -out "$MTLS_DIR/rogue.csr" >/dev/null 2>&1
    openssl x509 -req -days 2 \
        -in "$MTLS_DIR/rogue.csr" \
        -CA "$MTLS_DIR/ca.pem" \
        -CAkey "$MTLS_DIR/ca-key.pem" \
        -CAcreateserial \
        -extfile "$MTLS_DIR/client.ext" \
        -out "$MTLS_DIR/rogue-cert.pem" >/dev/null 2>&1

    chmod 600 "$MTLS_DIR/"*-key.pem "$MTLS_DIR/client-identity.pem"
    ALLOWED_FINGERPRINT="$(
        openssl x509 -in "$MTLS_DIR/client-cert.pem" -noout -fingerprint -sha256 \
            | sed 's/^[^=]*=//'
    )"
}

write_env_file() {
    ENV_FILE="$TEMP_DIR/t6pc.env"
    cat >"$ENV_FILE" <<EOF
DUBBRIDGE_DATABASE_URL=postgres://unused:unused@db.invalid:5432/dubbridge
DUBBRIDGE_REDIS_URL=redis://redis.invalid:6379
DUBBRIDGE_STORAGE__ACCESS_KEY_ID=t6pc-storage-access
DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=t6pc-storage-secret
DUBBRIDGE_AUTH__JWT_SECRET=t6pc-jwt-secret
DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET=t6pc-oauth-secret
DUBBRIDGE_TRANSLATION_API_URL=https://translation.invalid
DUBBRIDGE_TRANSLATION_API_KEY=t6pc-translation-secret
DUBBRIDGE_P2P_KEK_HEX=0303030303030303030303030303030303030303030303030303030303030303
DUBBRIDGE_P2P_KEK_ID=t6pc-kek
DUBBRIDGE_P2P_KEK_VERSION=1
DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS=$ALLOWED_FINGERPRINT
DUBBRIDGE_P2P_MTLS_DIR=$MTLS_DIR
DUBBRIDGE_IMAGE_REVISION=$HEAD_SHA
DUBBRIDGE_P2P_HYPERSWARM_JOIN_TIMEOUT_MS=15000
DUBBRIDGE_P2P_HTTP_TIMEOUT_SECS=10
DUBBRIDGE_P2P_LEASE_SECS=30
DUBBRIDGE_P2P_RETRY_SECS=5
DUBBRIDGE_P2P_DISPATCH_INTERVAL_MS=1000
DUBBRIDGE_P2P_MAX_ATTEMPTS=5
EOF
}

compose() {
    compose_with_env_file "$ENV_FILE" "$@"
}

expect_config_failure_without() {
    local var="$1"
    local stripped="$TEMP_DIR/no-${var}.env"
    grep -v "^${var}=" "$ENV_FILE" >"$stripped"
    if compose_with_env_file "$stripped" config >/dev/null 2>&1; then
        die "Compose rendered without required $var"
    fi
    log "T6PC_FAIL_CLOSED_${var}=PASS"
}

render_checks() {
    local rendered="$TEMP_DIR/rendered.json"
    compose config --format json >"$rendered"

    jq -e '.services["availability-node"] != null' "$rendered" >/dev/null
    jq -e '((.services["availability-node"].ports // []) | length) == 0' "$rendered" >/dev/null
    jq -e '(.services["availability-node"].networks | has("p2p-control"))' "$rendered" >/dev/null
    jq -e '((.networks["p2p-control"].internal // false) == false)' "$rendered" >/dev/null

    jq -e '
      (.services["availability-node"].environment) as $e |
      ([
        "DUBBRIDGE_DATABASE_URL",
        "DUBBRIDGE_REDIS_URL",
        "DUBBRIDGE_STORAGE__ACCESS_KEY_ID",
        "DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY",
        "DUBBRIDGE_AUTH__JWT_SECRET",
        "DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET",
        "DUBBRIDGE_TRANSLATION_API_URL",
        "DUBBRIDGE_TRANSLATION_API_KEY",
        "DUBBRIDGE_P2P_KEK_HEX",
        "DUBBRIDGE_P2P_KEK_ID",
        "DUBBRIDGE_P2P_KEK_VERSION",
        "DUBBRIDGE_P2P_AVAILABILITY_IDENTITY_PEM"
      ] | all(. as $k | ($e | has($k) | not)))
    ' "$rendered" >/dev/null

    jq -e '
      (.services["worker-runner"].environment) as $e |
      ($e | has("DUBBRIDGE_P2P_KEK_HEX")) and
      ($e | has("DUBBRIDGE_P2P_KEK_ID")) and
      ($e | has("DUBBRIDGE_P2P_KEK_VERSION")) and
      ($e | has("DUBBRIDGE_P2P_AVAILABILITY_IDENTITY_PEM"))
    ' "$rendered" >/dev/null

    jq -e '
      (.services.api.environment) as $e |
      ($e | has("DUBBRIDGE_P2P_KEK_HEX")) and
      ($e | has("DUBBRIDGE_P2P_KEK_ID")) and
      ($e | has("DUBBRIDGE_P2P_KEK_VERSION")) and
      ($e | has("DUBBRIDGE_P2P_AVAILABILITY_IDENTITY_PEM") | not)
    ' "$rendered" >/dev/null

    jq -e '
      .services["availability-node"].volumes |
      any(.target == "/var/lib/dubbridge/p2p-ciphertext" and .read_only == true)
    ' "$rendered" >/dev/null
    jq -e '
      .services["worker-runner"].volumes |
      any(.target == "/var/lib/dubbridge/p2p-ciphertext" and (.read_only // false) == false)
    ' "$rendered" >/dev/null

    expect_config_failure_without DUBBRIDGE_IMAGE_REVISION
    expect_config_failure_without DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS
    expect_config_failure_without DUBBRIDGE_P2P_KEK_HEX

    log "T6PC_RENDER=PASS"
    log "T6PC_SECRET_BOUNDARY=PASS"
}

wait_for_service_health() {
    local service="$1"
    local attempt=0
    local container=""
    local status=""

    while [[ "$attempt" -lt 30 ]]; do
        container="$(compose ps -a -q "$service" 2>/dev/null | tail -n 1 || true)"
        if [[ -n "$container" ]] && docker inspect "$container" >/dev/null 2>&1; then
            AN_CONTAINER="$container"
            status="$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' "$container" 2>/dev/null || true)"
            if [[ "$status" == "healthy" ]]; then
                return 0
            fi
            if [[ "$status" == "exited" || "$status" == "dead" ]]; then
                compose logs --no-color "$service" >&2 || true
                docker inspect "$container" >&2 || true
                return 1
            fi
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    printf 'ERROR: %s did not become healthy; last container=%s status=%s\n' \
        "$service" "${container:-none}" "${status:-unknown}" >&2
    compose ps -a >&2 || true
    compose logs --no-color "$service" >&2 || true
    if [[ -n "$container" ]]; then
        docker inspect "$container" >&2 || true
    fi
    return 1
}

node_probe() {
    local cert="$1"
    local key="$2"
    local expected_code="$3"
    docker run --rm \
        --network "$AN_NETWORK" \
        -v "$MTLS_DIR:/certs:ro" \
        node:22.23.0-bookworm-slim \
        node - "$cert" "$key" "$expected_code" <<'NODE'
const https = require("https");
const fs = require("fs");
const [certPath, keyPath, expected] = process.argv.slice(2);
const options = {
  hostname: "availability-node",
  port: 8443,
  path: "/t6pc-probe",
  method: "GET",
  ca: fs.readFileSync("/certs/ca.pem"),
  cert: fs.readFileSync(certPath),
  key: fs.readFileSync(keyPath),
  minVersion: "TLSv1.3",
  rejectUnauthorized: true,
};
const req = https.request(options, (res) => {
  let body = "";
  res.setEncoding("utf8");
  res.on("data", (chunk) => { body += chunk; });
  res.on("end", () => {
    if (!body.includes(expected)) {
      console.error("unexpected probe response: " + body);
      process.exit(1);
    }
    process.exit(0);
  });
});
req.on("error", (error) => {
  console.error(error.message);
  process.exit(1);
});
req.end();
NODE
}

no_client_cert_probe() {
    if docker run --rm \
        --network "$AN_NETWORK" \
        -v "$MTLS_DIR:/certs:ro" \
        node:22.23.0-bookworm-slim \
        node - <<'NODE'
const https = require("https");
const fs = require("fs");
const req = https.request({
  hostname: "availability-node",
  port: 8443,
  path: "/t6pc-probe",
  method: "GET",
  ca: fs.readFileSync("/certs/ca.pem"),
  minVersion: "TLSv1.3",
  rejectUnauthorized: true,
}, (res) => {
  console.error("unexpected HTTP response without client certificate: " + res.statusCode);
  process.exit(0);
});
req.on("error", () => process.exit(1));
req.end();
NODE
    then
        die "mTLS request without client certificate unexpectedly reached HTTP"
    fi
}

runtime_checks() {
    need_cmd docker
    need_cmd jq
    need_cmd openssl
    need_cmd git
    detect_compose
    log "T6PC_COMPOSE_BACKEND=$COMPOSE_BACKEND"

    cd "$ROOT_DIR"
    HEAD_SHA="$(git rev-parse HEAD)"
    git diff --quiet || die "working tree is dirty; certify an exact committed artifact"
    git diff --cached --quiet || die "index is dirty; certify an exact committed artifact"

    TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/dubbridge-t6pc.XXXXXX")"
    PROJECT="dubbridget6pc${HEAD_SHA:0:8}"

    generate_mtls
    write_env_file
    render_checks

    compose build availability-node
    export DUBBRIDGE_AVAILABILITY_IMAGE_TAG="dubbridge-availability-node:$HEAD_SHA"
    bash "$IMAGE_TEST" contract availability
    bash "$IMAGE_TEST" run availability

    IMAGE_ID="$(docker image inspect --format '{{.Id}}' "$DUBBRIDGE_AVAILABILITY_IMAGE_TAG")"
    [[ "$IMAGE_ID" == sha256:* ]] || die "unexpected local OCI image ID: $IMAGE_ID"
    log "T6PC_HEAD=$HEAD_SHA"
    log "T6PC_IMAGE_ID=$IMAGE_ID"
    log "T6PC_IMAGE_CONTRACT=PASS"

    compose up -d --no-deps availability-node
    wait_for_service_health availability-node || die "Availability Node did not become healthy"
    [[ -n "$AN_CONTAINER" ]] || die "Availability Node container id not found after health check"

    local running_image
    running_image="$(docker inspect --format '{{.Image}}' "$AN_CONTAINER")"
    [[ "$running_image" == "$IMAGE_ID" ]] || die "running image $running_image != built image $IMAGE_ID"

    AN_NETWORK="$(
        docker inspect --format '{{range $name, $network := .NetworkSettings.Networks}}{{println $name}}{{end}}' "$AN_CONTAINER" \
            | grep 'p2p-control$' | head -n 1
    )"
    [[ -n "$AN_NETWORK" ]] || die "Availability Node is not attached to p2p-control"

    local network_count
    network_count="$(docker inspect --format '{{len .NetworkSettings.Networks}}' "$AN_CONTAINER")"
    [[ "$network_count" == "1" ]] || die "Availability Node is attached to unexpected networks"

    local internal_network
    internal_network="$(docker network inspect --format '{{.Internal}}' "$AN_NETWORK")"
    [[ "$internal_network" == "false" ]] || die "p2p-control is Docker-internal; Hyperswarm egress would be blocked"

    local port_binding
    port_binding="$(docker inspect --format '{{with (index .HostConfig.PortBindings "8443/tcp")}}{{json .}}{{else}}null{{end}}' "$AN_CONTAINER")"
    [[ "$port_binding" == "null" ]] || die "Availability Node has a host binding for 8443"

    local node_version
    node_version="$(compose exec -T availability-node node --version)"
    [[ "$node_version" == "v22.23.0" ]] || die "runtime Node version drift: $node_version"

    log "T6PC_PRIVATE_NETWORK=PASS"

    node_probe /certs/client-cert.pem /certs/client-key.pem '"invalid_contract"'
    log "T6PC_MTLS_ALLOWED=PASS"

    node_probe /certs/rogue-cert.pem /certs/rogue-key.pem '"service_identity_rejected"'
    log "T6PC_MTLS_WRONG_FINGERPRINT=PASS"

    no_client_cert_probe
    log "T6PC_MTLS_NO_CLIENT_CERT=PASS"

    local cipher_volume drive_volume index_volume
    cipher_volume="$(
        docker inspect --format '{{range .Mounts}}{{if eq .Destination "/var/lib/dubbridge/p2p-ciphertext"}}{{.Name}}{{end}}{{end}}' "$AN_CONTAINER"
    )"
    drive_volume="$(
        docker inspect --format '{{range .Mounts}}{{if eq .Destination "/var/lib/dubbridge/p2p-drive"}}{{.Name}}{{end}}{{end}}' "$AN_CONTAINER"
    )"
    index_volume="$(
        docker inspect --format '{{range .Mounts}}{{if eq .Destination "/var/lib/dubbridge/p2p-index"}}{{.Name}}{{end}}{{end}}' "$AN_CONTAINER"
    )"
    [[ -n "$cipher_volume" && -n "$drive_volume" && -n "$index_volume" ]] \
        || die "failed to resolve one or more P2P named volumes"

    docker run --rm -v "$cipher_volume:/data" node:22.23.0-bookworm-slim \
        sh -c 'printf "%s\n" t6pc-ciphertext > /data/t6pc-ciphertext-marker'
    compose exec -T availability-node sh -c \
        'test "$(cat /var/lib/dubbridge/p2p-ciphertext/t6pc-ciphertext-marker)" = "t6pc-ciphertext"'

    if compose exec -T availability-node sh -c \
        'touch /var/lib/dubbridge/p2p-ciphertext/t6pc-write-must-fail' >/dev/null 2>&1; then
        die "Availability Node wrote to the ciphertext read-only mount"
    fi

    compose exec -T availability-node sh -c \
        'printf "%s\n" t6pc-drive > /var/lib/dubbridge/p2p-drive/t6pc-drive-marker'
    compose exec -T availability-node sh -c \
        'printf "%s\n" t6pc-index > /var/lib/dubbridge/p2p-index/t6pc-index-marker'

    AN_CONTAINER=""
    compose up -d --no-deps --force-recreate availability-node
    wait_for_service_health availability-node || die "Availability Node did not recover after recreate"
    [[ -n "$AN_CONTAINER" ]] || die "Availability Node container id not found after recreate"

    compose exec -T availability-node sh -c \
        'test "$(cat /var/lib/dubbridge/p2p-ciphertext/t6pc-ciphertext-marker)" = "t6pc-ciphertext"'
    compose exec -T availability-node sh -c \
        'test "$(cat /var/lib/dubbridge/p2p-drive/t6pc-drive-marker)" = "t6pc-drive"'
    compose exec -T availability-node sh -c \
        'test "$(cat /var/lib/dubbridge/p2p-index/t6pc-index-marker)" = "t6pc-index"'

    log "T6PC_PERSISTENCE=PASS"

    if compose run --rm --no-deps \
        -e DUBBRIDGE_P2P_AVAILABILITY_SERVER_KEY_PEM=/no-such/server-key.pem \
        availability-node >/dev/null 2>&1; then
        die "Availability Node started with missing server key"
    fi
    log "T6PC_NEGATIVE_MISSING_SERVER_KEY=PASS"

    log "T6PC=PASS"
}

case "$MODE" in
    contract) contract_checks ;;
    runtime) runtime_checks ;;
    all) contract_checks; runtime_checks ;;
    -h|--help|help) usage ;;
    *) usage >&2; exit 2 ;;
esac
