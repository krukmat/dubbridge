#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

# Case registry - bash 3.2 compatible (no associative arrays)
CASE_LIST="self-check api gateway migration worker asr"

# Cleanup machinery
TEMP_DIR=""
cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT INT TERM

usage() {
    cat >&2 <<'EOF'
Usage: scripts/test-production-images.sh <mode> <case>

Modes:
  contract - Validate image contract
  run - Run image test

Cases:
  self-check - Verify script preconditions (no Docker)
EOF
    exit 1
}

# Case membership check
case_exists() {
    local needle="$1"
    local c
    local old_ifs="$IFS"
    IFS=' '
    for c in $CASE_LIST; do
        if [[ "$c" == "$needle" ]]; then
            IFS="$old_ifs"
            return 0
        fi
    done
    IFS="$old_ifs"
    return 1
}

# self-check case functions
contract_self-check() {
    echo "Contract check for self-check case"
    # Verify required commands are on PATH
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi
    if ! command -v jq >/dev/null 2>&1; then
        echo "ERROR: jq not found on PATH" >&2
        return 1
    fi
    # Print bash version
    echo "Bash version: $BASH_VERSION"
    return 0
}

run_self-check() {
    echo "Run check for self-check case"
    # Verify required commands are on PATH
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi
    if ! command -v jq >/dev/null 2>&1; then
        echo "ERROR: jq not found on PATH" >&2
        return 1
    fi
    # Print bash version
    echo "Bash version: $BASH_VERSION"
    return 0
}

contract_api() {
    echo "Contract check for api case"
    # Verify Dockerfile exists
    if [ ! -f "apps/api/Dockerfile" ]; then
        echo "ERROR: apps/api/Dockerfile not found" >&2
        return 1
    fi
    # Verify ENTRYPOINT
    if ! grep -q 'ENTRYPOINT \["/app/dubbridge-api"\]' "apps/api/Dockerfile"; then
        echo "ERROR: ENTRYPOINT [\"/app/dubbridge-api\"] not found in Dockerfile" >&2
        return 1
    fi
    # Verify EXPOSE
    if ! grep -q 'EXPOSE 8080' "apps/api/Dockerfile"; then
        echo "ERROR: EXPOSE 8080 not found in Dockerfile" >&2
        return 1
    fi
    echo "Contract check passed for api"
    return 0
}

run_api() {
    echo "Run check for api case"
    # Verify required commands are on PATH
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi
    if ! command -v curl >/dev/null 2>&1; then
        echo "ERROR: curl not found on PATH" >&2
        return 1
    fi

    # Define dependency container
    dep_container="${DUBBRIDGE_TEST_DEPENDENCY_CONTAINER:-local-postgres-1}"

    # Verify dependency is running
    dep_running=$(docker inspect -f '{{.State.Running}}' "$dep_container" 2>/dev/null)
    if [ "$dep_running" != "true" ]; then
        echo "ERROR: dependency container '$dep_container' is not running — bring up infra/local/docker-compose.yml first" >&2
        return 1
    fi

    # Resolve network dynamically
    test_network=$(docker inspect -f '{{range $k, $v := .NetworkSettings.Networks}}{{$k}}{{end}}' "$dep_container" 2>/dev/null | head -n 1)
    if [ -z "$test_network" ]; then
        echo "ERROR: could not resolve network for dependency container '$dep_container'" >&2
        return 1
    fi

    # Define cleanup function
    # `|| true` on each step: under `set -e`, a failure inside a RETURN trap
    # (e.g. stopping a --rm container that already self-removed) would abort
    # the whole script and clobber the real exit status of run_api().
    cleanup_api() {
        docker stop dubbridge-api-contract-test >/dev/null 2>&1 || true
        docker start "$dep_container" >/dev/null 2>&1 || true
    }

    # Register trap for cleanup
    trap cleanup_api RETURN

    # Start API container
    api_image="${DUBBRIDGE_API_IMAGE_TAG:-dubbridge-api-t4c:test}"
    docker run -d --rm --network "$test_network" -p 8080:8080 \
        -e DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@postgres:5432/dubbridge \
        -e DUBBRIDGE_REDIS_URL=redis://redis:6379 \
        -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000 \
        -e DUBBRIDGE_ENV=local \
        -e AWS_ACCESS_KEY_ID=dubbridge \
        -e AWS_SECRET_ACCESS_KEY=dubbridge123 \
        -e AWS_REGION=us-east-1 \
        -e DUBBRIDGE_STORAGE__BACKEND=local_fs \
        -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \
        --name dubbridge-api-contract-test \
        "$api_image"

    # Poll /health/live
    live_ok=0
    for i in $(seq 1 30); do
        if curl -sf -o /dev/null http://localhost:8080/health/live; then
            live_ok=1
            break
        fi
        sleep 1
    done

    if [ "$live_ok" -ne 1 ]; then
        echo "ERROR: /health/live did not become ready within timeout" >&2
        return 1
    fi

    # Poll /health/ready
    ready_ok=0
    for i in $(seq 1 30); do
        if curl -sf -o /dev/null http://localhost:8080/health/ready; then
            ready_ok=1
            break
        fi
        sleep 1
    done

    if [ "$ready_ok" -ne 1 ]; then
        echo "ERROR: /health/ready did not become ready within timeout" >&2
        return 1
    fi

    # EC-1: Stop dependency and verify readiness degrades
    docker stop "$dep_container" >/dev/null 2>&1

    # Re-check /health/ready (should fail)
    if curl -sf -o /dev/null http://localhost:8080/health/ready; then
        echo "ERROR: EC-1 FAILED: readiness did not degrade after dependency stop" >&2
        return 1
    fi

    # Re-check /health/live (should still pass)
    if ! curl -sf -o /dev/null http://localhost:8080/health/live; then
        echo "ERROR: EC-1 FAILED: liveness incorrectly depends on downstream dependency" >&2
        return 1
    fi

    echo "Run check passed for api"
    return 0
}

contract_gateway() {
    echo "Contract check for gateway case"
    if [ ! -f "apps/gateway/Dockerfile" ]; then
        echo "ERROR: apps/gateway/Dockerfile not found" >&2
        return 1
    fi

    if ! grep -q 'ENTRYPOINT \["/app/dubbridge-gateway"\]' apps/gateway/Dockerfile; then
        echo "ERROR: ENTRYPOINT [\"/app/dubbridge-gateway\"] not found in Dockerfile" >&2
        return 1
    fi

    if ! grep -q 'EXPOSE 8081' apps/gateway/Dockerfile; then
        echo "ERROR: EXPOSE 8081 not found in Dockerfile" >&2
        return 1
    fi

    echo "Contract check passed for gateway"
    return 0
}

run_gateway() {
    echo "Run check for gateway case"
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi
    if ! command -v curl >/dev/null 2>&1; then
        echo "ERROR: curl not found on PATH" >&2
        return 1
    fi

    api_image="${DUBBRIDGE_API_IMAGE_TAG:-dubbridge-api-t4c:test}"
    gateway_image="${DUBBRIDGE_GATEWAY_IMAGE_TAG:-dubbridge-gateway-t4d:test}"

    # `|| true` on each step: under `set -e`, a failure inside a RETURN trap
    # (e.g. stopping a --rm container that already self-removed) would abort
    # the whole script and clobber the real exit status of run_gateway().
    cleanup_gateway() {
        docker stop dubbridge-gateway-contract-test >/dev/null 2>&1 || true
        docker stop dubbridge-api-gateway-dep >/dev/null 2>&1 || true
    }
    trap cleanup_gateway RETURN

    # Start API dependency container. --network host means bridge-network
    # service names (postgres/redis/minio) are unreachable; use localhost
    # against the docker-compose published host ports instead.
    docker run -d --rm --network host \
        -e DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@localhost:5432/dubbridge \
        -e DUBBRIDGE_REDIS_URL=redis://localhost:6379 \
        -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://localhost:9000 \
        -e DUBBRIDGE_ENV=local \
        -e AWS_ACCESS_KEY_ID=dubbridge \
        -e AWS_SECRET_ACCESS_KEY=dubbridge123 \
        -e AWS_REGION=us-east-1 \
        -e DUBBRIDGE_STORAGE__BACKEND=local_fs \
        -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \
        --name dubbridge-api-gateway-dep \
        "$api_image"

    # Poll API /health/live
    api_live_ok=0
    for i in $(seq 1 30); do
        if curl -sf -o /dev/null http://localhost:8080/health/live; then
            api_live_ok=1
            break
        fi
        sleep 1
    done
    if [ "$api_live_ok" -ne 1 ]; then
        echo "ERROR: API dependency /health/live did not become ready within timeout" >&2
        return 1
    fi

    # Poll API /health/ready
    api_ready_ok=0
    for i in $(seq 1 30); do
        if curl -sf -o /dev/null http://localhost:8080/health/ready; then
            api_ready_ok=1
            break
        fi
        sleep 1
    done
    if [ "$api_ready_ok" -ne 1 ]; then
        echo "ERROR: API dependency /health/ready did not become ready within timeout" >&2
        return 1
    fi

    # Start gateway container
    docker run -d --rm --network host \
        -e DUBBRIDGE_ENV=local \
        --name dubbridge-gateway-contract-test \
        "$gateway_image"

    # Poll gateway /health/live
    gw_live_ok=0
    for i in $(seq 1 30); do
        if curl -sf -o /dev/null http://localhost:8081/health/live; then
            gw_live_ok=1
            break
        fi
        sleep 1
    done
    if [ "$gw_live_ok" -ne 1 ]; then
        echo "ERROR: gateway /health/live did not become ready within timeout" >&2
        return 1
    fi

    # Poll gateway /health/ready
    gw_ready_ok=0
    for i in $(seq 1 30); do
        if curl -sf -o /dev/null http://localhost:8081/health/ready; then
            gw_ready_ok=1
            break
        fi
        sleep 1
    done
    if [ "$gw_ready_ok" -ne 1 ]; then
        echo "ERROR: gateway /health/ready did not become ready within timeout" >&2
        return 1
    fi

    # EC-1: Stop API dependency and verify gateway readiness degrades
    docker stop dubbridge-api-gateway-dep >/dev/null 2>&1

    # Re-check gateway /health/ready (should fail)
    if curl -sf -o /dev/null http://localhost:8081/health/ready; then
        echo "ERROR: EC-1 FAILED: gateway readiness did not degrade after API dependency stop" >&2
        return 1
    fi

    # Re-check gateway /health/live (should still pass)
    if ! curl -sf -o /dev/null http://localhost:8081/health/live; then
        echo "ERROR: EC-1 FAILED: gateway liveness incorrectly depends on API dependency" >&2
        return 1
    fi

    echo "Run check passed for gateway"
    return 0
}

contract_migration() {
    echo "Contract check for migration case"
    if [ ! -f "apps/cli/Dockerfile" ]; then
        echo "ERROR: apps/cli/Dockerfile not found" >&2
        return 1
    fi
    if ! grep -q 'ENTRYPOINT \["/app/dubbridge-cli"\]' "apps/cli/Dockerfile"; then
        echo "ERROR: ENTRYPOINT [\"/app/dubbridge-cli\"] not found in Dockerfile" >&2
        return 1
    fi
    if ! grep -q 'sqlx::migrate' apps/cli/src/main.rs; then
        echo "ERROR: sqlx::migrate! macro call not found in apps/cli/src/main.rs" >&2
        return 1
    fi
    echo "Contract check passed for migration"
    return 0
}

run_migration() {
    echo "Run check for migration case"
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi
    dep_container="${DUBBRIDGE_TEST_DEPENDENCY_CONTAINER:-local-postgres-1}"
    dep_running=$(docker inspect -f '{{.State.Running}}' "$dep_container" 2>/dev/null) ; if [ "$dep_running" != "true" ]; then echo "ERROR: dependency container '$dep_container' is not running — bring up infra/local/docker-compose.yml first" >&2; return 1; fi
    test_network=$(docker inspect -f '{{range $k, $v := .NetworkSettings.Networks}}{{$k}}{{end}}' "$dep_container" 2>/dev/null | head -n 1) ; if [ -z "$test_network" ]; then echo "ERROR: could not resolve network for dependency container '$dep_container'" >&2; return 1; fi
    cli_image="${DUBBRIDGE_CLI_IMAGE_TAG:-dubbridge-cli:t4f-test}"
    test_db="t4g_contract_test"
    cleanup_migration() {
        # `|| true` on each step: under `set -e`, a failure inside a RETURN
        # trap (e.g. dropping a database that was never created) would abort
        # the whole script and clobber the real exit status of run_migration().
        docker exec "$dep_container" psql -U dubbridge -d dubbridge -c "DROP DATABASE IF EXISTS $test_db;" >/dev/null 2>&1 || true
    }
    trap cleanup_migration RETURN
    docker exec "$dep_container" psql -U dubbridge -d dubbridge -c "CREATE DATABASE $test_db;" >/dev/null 2>&1 || {
      echo "ERROR: could not create test database '$test_db'" >&2
      return 1
    }
    hp1_output=$(docker run --rm --network "$test_network" -e DUBBRIDGE_ENV=local -e DUBBRIDGE_DATABASE_URL="postgres://dubbridge:dubbridge@${dep_container}:5432/${test_db}" "$cli_image" 2>&1)
    hp1_exit=$?
    if [ "$hp1_exit" -ne 0 ]; then
        echo "ERROR: HP-1 FAILED: migration run against empty DB exited $hp1_exit" >&2
        echo "$hp1_output" >&2
        return 1
    fi
    if ! echo "$hp1_output" | grep -q "migrations applied successfully"; then
        echo "ERROR: HP-1 FAILED: success log line not found in migration output" >&2
        echo "$hp1_output" >&2
        return 1
    fi
    ec1_exit=0
    ec1_output=$(docker run --rm -e DUBBRIDGE_ENV=local -e DUBBRIDGE_DATABASE_URL="postgres://dubbridge:dubbridge@nonexistent-host-unreachable:5432/${test_db}" "$cli_image" 2>&1) || ec1_exit=$?
    if [ "$ec1_exit" -eq 0 ]; then
        echo "ERROR: EC-1 FAILED: migration run against unreachable DB exited 0" >&2
        return 1
    fi
    if echo "$ec1_output" | grep -q "migrations applied successfully"; then
        echo "ERROR: EC-1 FAILED: success log line present despite unreachable DB" >&2
        return 1
    fi
    echo "Run check passed for migration"
    return 0
}

contract_worker() {
    echo "Contract check for worker case"
    if [ ! -f "apps/worker-runner/Dockerfile" ]; then
        echo "ERROR: apps/worker-runner/Dockerfile not found" >&2
        return 1
    fi
    if ! grep -q 'ENTRYPOINT \["/app/dubbridge-worker-runner"\]' "apps/worker-runner/Dockerfile"; then
        echo "ERROR: ENTRYPOINT [\"/app/dubbridge-worker-runner\"] not found in Dockerfile" >&2
        return 1
    fi
    if ! grep -qE '^\s*ffmpeg\s*\\?\s*$' "apps/worker-runner/Dockerfile"; then
        echo "ERROR: ffmpeg package install not found in Dockerfile" >&2
        return 1
    fi
    echo "Contract check passed for worker"
    return 0
}

run_worker() {
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi

    worker_image="${DUBBRIDGE_WORKER_IMAGE_TAG:-dubbridge-worker-runner-t4i:test}"

    # HP-1
    hp1_exit=0
    hp1_output=$(docker run --rm --entrypoint /bin/sh "$worker_image" -c 'ls -la /app/dubbridge-worker-runner' 2>&1) || hp1_exit=$?
    if [ "$hp1_exit" -ne 0 ] || ! echo "$hp1_output" | grep -qE '^-rwx'; then
        echo "ERROR: HP-1 FAILED: executable permission check failed" >&2
        echo "$hp1_output" >&2
        return 1
    fi

    hp1_exit=0
    hp1_output=$(docker run --rm --entrypoint ffmpeg "$worker_image" -version 2>&1) || hp1_exit=$?
    if [ "$hp1_exit" -ne 0 ]; then
        echo "ERROR: HP-1 FAILED: ffmpeg version check failed" >&2
        echo "$hp1_output" >&2
        return 1
    fi

    hp1_exit=0
    hp1_output=$(docker run --rm --entrypoint ffprobe "$worker_image" -version 2>&1) || hp1_exit=$?
    if [ "$hp1_exit" -ne 0 ]; then
        echo "ERROR: HP-1 FAILED: ffprobe version check failed" >&2
        echo "$hp1_output" >&2
        return 1
    fi

    # EC-1
    ec1_exit=0
    ec1_output=$(docker run --rm --entrypoint /bin/sh "$worker_image" -c 'rm /usr/bin/ffmpeg && ffmpeg -version' 2>&1) || ec1_exit=$?
    if [ "$ec1_exit" -eq 0 ]; then
        echo "ERROR: EC-1 FAILED: invocation succeeded after removing ffmpeg" >&2
        echo "$ec1_output" >&2
        return 1
    fi

    echo "Run check passed for worker"
    return 0
}

contract_asr() {
    echo "Contract check for asr case"
    if [ ! -f "apps/worker-runner/Dockerfile" ]; then
        echo "ERROR: apps/worker-runner/Dockerfile not found" >&2
        return 1
    fi
    if ! grep -q 'faster-whisper==1.1.0' "workers/asr-worker-py/requirements.txt"; then
        echo "ERROR: faster-whisper==1.1.0 pin not found in workers/asr-worker-py/requirements.txt" >&2
        return 1
    fi
    if ! grep -q 'ENV DUBBRIDGE_ASR_WORKER_PATH=' "apps/worker-runner/Dockerfile"; then
        echo "ERROR: DUBBRIDGE_ASR_WORKER_PATH env var not found in Dockerfile" >&2
        return 1
    fi
    if ! grep -q 'ENV DUBBRIDGE_ASR_WORKER_PYTHON=' "apps/worker-runner/Dockerfile"; then
        echo "ERROR: DUBBRIDGE_ASR_WORKER_PYTHON env var not found in Dockerfile" >&2
        return 1
    fi
    if ! grep -q 'ENV ASR_MODEL_SIZE=small' "apps/worker-runner/Dockerfile"; then
        echo "ERROR: default ASR_MODEL_SIZE=small not found in Dockerfile" >&2
        return 1
    fi
    echo "Contract check passed for asr"
    return 0
}

run_asr() {
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi

    asr_image="${DUBBRIDGE_ASR_WORKER_IMAGE_TAG:-dubbridge-worker-runner-t4k:test}"

    # Generate a real (silent) WAV inside the container first — main.py
    # checks os.path.exists() before touching ASR_MODEL_SIZE, so a
    # nonexistent path would short-circuit both HP-1 and EC-1 on
    # audio_not_found before the behavior under test ever runs.
    gen_wav='python3 -c "
import struct, wave
with wave.open(\"/tmp/t4l-silence.wav\", \"wb\") as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(16000)
    w.writeframes(struct.pack(\"<8000h\", *([0] * 8000)))
"'

    # HP-1: dependency and protocol resolution end to end (no live
    # transcription — that needs network access to fetch model weights,
    # unavailable in this harness; matches T4k's HP-1 evidence scope).
    # Confirming dependency resolution via direct imports/version pin is a
    # deliberately narrower check than a real transcription pass.
    hp1_exit=0
    hp1_output=$(docker run --rm --entrypoint /bin/sh "$asr_image" -c "
        set -e
        python3 -c 'import faster_whisper' &&
        python3 -c 'from faster_whisper import WhisperModel' &&
        test -f \"\$DUBBRIDGE_ASR_WORKER_PATH\" &&
        [ \"\$DUBBRIDGE_ASR_WORKER_PYTHON\" = python3 ] &&
        pip3 show faster-whisper | grep -q '^Version: 1.1.0\$' &&
        $gen_wav &&
        echo '{\"job_id\":\"t4l-hp1\",\"audio_uri\":\"file:///tmp/t4l-silence.wav\"}' | python3 \"\$DUBBRIDGE_ASR_WORKER_PATH\"
    " 2>&1) || hp1_exit=$?
    if [ "$hp1_exit" -ne 0 ]; then
        echo "ERROR: HP-1 FAILED: dependency/protocol resolution check failed" >&2
        echo "$hp1_output" >&2
        return 1
    fi
    if ! echo "$hp1_output" | grep -q '"status": "ok"'; then
        echo "ERROR: HP-1 FAILED: no successful protocol response in output" >&2
        echo "$hp1_output" >&2
        return 1
    fi

    # EC-1: invalid ASR_MODEL_SIZE must fail closed, never fall back to
    # large-v3 or return a false-ready result.
    ec1_exit=0
    ec1_output=$(docker run --rm -e ASR_MODEL_SIZE=not-a-real-invalid-model-xyz --entrypoint /bin/sh "$asr_image" -c "
        $gen_wav &&
        echo '{\"job_id\":\"t4l-ec1\",\"audio_uri\":\"file:///tmp/t4l-silence.wav\"}' | python3 \"\$DUBBRIDGE_ASR_WORKER_PATH\"
    " 2>&1) || ec1_exit=$?
    if [ "$ec1_exit" -eq 0 ]; then
        echo "ERROR: EC-1 FAILED: invocation succeeded with an invalid ASR_MODEL_SIZE" >&2
        echo "$ec1_output" >&2
        return 1
    fi
    if ! echo "$ec1_output" | grep -q '"error_code": "transcription_failed"'; then
        echo "ERROR: EC-1 FAILED: expected transcription_failed error_code not found" >&2
        echo "$ec1_output" >&2
        return 1
    fi

    echo "Run check passed for asr"
    return 0
}

# Main execution
main() {
    # Validation 1: Exactly 2 positional arguments
    if [[ $# -ne 2 ]]; then
        usage
    fi

    local mode="$1"
    local case_name="$2"

    # Validation 2: Mode must be contract or run
    if [[ "$mode" != "contract" && "$mode" != "run" ]]; then
        usage
    fi

    # Validation 3: Case must exist in registry
    if ! case_exists "$case_name"; then
        usage
    fi

    # Dispatch to function
    local func_name="${mode}_${case_name}"
    if ! declare -F "$func_name" >/dev/null 2>&1; then
        echo "ERROR: Function $func_name not found" >&2
        exit 1
    fi

    "$func_name"
}

main "$@"
