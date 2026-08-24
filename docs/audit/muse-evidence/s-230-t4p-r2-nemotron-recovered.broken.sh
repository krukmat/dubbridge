# Full-pipeline case for scripts/test-production-images.sh.
# This module intentionally relies on the parent harness for strict mode,
# TEMP_DIR, case dispatch, and process-wide cleanup.

contract_full-pipeline() {
    echo "Contract check for full-pipeline case"
    if [ ! -f "apps/api/Dockerfile" ]; then
        echo "ERROR: apps/api/Dockerfile not found" >&2
        return 1
    fi
    if [ ! -f "apps/worker-runner/Dockerfile" ]; then
        echo "ERROR: apps/worker-runner/Dockerfile not found" >&2
        return 1
    fi
    if ! grep -q 'ENTRYPOINT \["/app/dubbridge-api"\]" apps/api/Dockerfile; then
        echo "ERROR: ENTRYPOINT ["/app/dubbridge-api"] not found in apps/api/Dockerfile" >&2
        return 1
    fi
    if ! grep -q 'ENTRYPOINT \["/app/dubbridge-worker-runner"\]" apps/worker-runner/Dockerfile; then
        echo "ERROR: ENTRYPOINT ["/app/dubbridge-worker-runner"] not found in apps/worker-runner/Dockerfile" >&2
        return 1
    fi
    echo "Contract check passed for full-pipeline"
    return 0
}

run_full-pipeline() {
    echo "Run check for full-pipeline case"
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi
    if ! command -v curl >/dev/null 2>&1; then
        echo "ERROR: curl not found on PATH" >&2
        return 1
    fi
    if ! command -v python3 >/dev/null 2>&1; then
        echo "ERROR: python3 not found on PATH" >&2
        return 1
    fi

    dep_container="${DUBBRIDGE_TEST_DEPENDENCY_CONTAINER:-local-postgres-1}"
    dep_running=$(docker inspect -f '{{.State.Running}}' "$dep_container" 2>/dev/null) ; if [ "$dep_running" != "true" ]; then echo "ERROR: dependency container '$dep_container' is not running — bring up infra/local/docker-compose.yml first" >&2; return 1; fi
    test_network=$(docker inspect -f '{{range $k, $v := .NetworkSettings.Networks}}{{$k}}{{end}}' "$dep_container" 2>/dev/null | head -n 1)
    if [ -z "$test_network" ]; then
        echo "ERROR: could not resolve network for dependency container '$dep_container'" >&2
        return 1
    fi

    api_image="${DUBBRIDGE_API_IMAGE_TAG:-dubbridge-api-t4c:test}"
    worker_image="${DUBBRIDGE_WORKER_IMAGE_TAG:-dubbridge-worker-runner-t4i:test}"

     # \(\| true\) on each step: under \( -e \), a failure inside a RETURN trap
     # \(e.g. stopping a --rm container that already self-removed\) would abort
     # the whole script and clobber the real exit status of run_full-pipeline().
    cleanup_full_pipeline() {
        docker stop dubbridge-worker-full-pipeline-test >/dev/null 2>&1 || true
        docker stop dubbridge-api-full-pipeline-test >/dev/null 2>&1 || true
     }
     trap cleanup_full_pipeline RETURN

    docker run -d --rm --network "$test_network" -p 8090:8080 \
         -e DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@postgres:5432/dubbridge \
         -e DUBBRIDGE_REDIS_URL=redis://redis:6379 \
         -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000 \
         -e DUBBRIDGE_ENV=local \
         -e DUBBRIDGE_STORAGE__ACCESS_KEY_ID=dubbridge \
         -e DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=dubbridge123 \
         -e DUBBRIDGE_STORAGE__REGION=us-east-1 \
         -e DUBBRIDGE_STORAGE__BACKEND=s3 \
         -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \
         --name dubbridge-api-full-pipeline-test \
         "$api_image"

    api_live_ok=0
    for i in $(seq 1 30); do
        if curl -sf -o /dev/null http://localhost:8090/health/live; then
            api_live_ok=1
            break
        fi
        sleep 1
    done
    if [ "$api_live_ok" -ne 1 ]; then
        echo "ERROR: api /health/live did not become ready within timeout" >&2
        return 1
    fi

    api_ready_ok=0
    for i in $(seq 1 30); do
        if curl -sf -o /dev/null http://localhost:8090/health/ready; then
            api_ready_ok=1
            break
        fi
        sleep 1
    done
    if [ "$api_ready_ok" -ne 1 ]; then
        echo "ERROR: api /health/ready did not become ready within timeout" >&2
        return 1
    fi

    docker run -d --rm --network "$test_network" \
         -e DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@postgres:5432/dubbridge \
         -e DUBBRIDGE_REDIS_URL=redis://redis:6379 \
         -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000 \
         -e DUBBRIDGE_ENV=local \
         -e DUBBRIDGE_STORAGE__ACCESS_KEY_ID=dubbridge \
         -e DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=dubbridge123 \
         -e DUBBRIDGE_STORAGE__REGION=us-east-1 \
         -e DUBBRIDGE_STORAGE__BACKEND=s3 \
         -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \
         --name dubbridge-worker-full-pipeline-test \
         "$worker_image"

    sleep 3
    worker_running=$(docker inspect -f '{{.State.Running}}' dubbridge-worker-full-pipeline-test 2>/dev/null)
    if [ "$worker_running" != "true" ]; then
        echo "ERROR: worker-runner container did not stay running" >&2
        return 1
    fi

    email="t4o-$(date +%s)@example.com"
    register_output=$(curl -sf -X POST http://localhost:8090/auth/register \
         -H "Content-Type: application/json" \
         -d "{\"email\":\"${email}\",\"password\":\"t4o-password-1\",\"workspaceName\":\"t4o-workspace\"}") || {
        echo "ERROR: HP-1 FAILED: auth/register did not return success" >&2
        return 1
     }
    token=$(echo "$register_output" | python3 -c 'import json,sys; print(json.load(sys.stdin)["token"])')
    if [ -z "$token" ]; then
        echo "ERROR: HP-1 FAILED: could not extract bearer token from register response" >&2
        echo "$register_output" >&2
        return 1
    fi

    fixture_wav="${TEMP_DIR:-/tmp}/t4o-fixture.wav"
    python3 -c "
import struct, wave
with wave.open('${fixture_wav}', 'wb') as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(16000)
    w.writeframes(struct.pack('<8000h', *([0] * 8000)))
"

    ingest_output=$(curl -sf -X POST http://localhost:8090/ingest \
         -H "Authorization: Bearer ${token}" \
         -F "file=@${fixture_wav}" \
         -F "title=t4o-full-pipeline-fixture") || {
        echo "ERROR: HP-1 FAILED: POST /ingest did not return success" >&2
        return 1
     }
    ingest_token=$(echo "$ingest_output" | python3 -c 'import json,sys; print(json.load(sys.stdin)["ingest_token"])')
    if [ -z "$ingest_token" ]; then
        echo "ERROR: HP-1 FAILED: could not extract ingest_token from ingest response" >&2
        echo "$ingest_output" >&2
        return 1
    fi

    rights_output=$(curl -sf -X POST "http://localhost:8090/ingest/${ingest_token}/rights" \
         -H "Authorization: Bearer ${token}" \
         -H "Content-Type: application/json" \
         -d '{"owner":"t4o-test-owner","license_type":"all_rights_reserved","source_type":"direct_upload","proof_reference":"t4o-full-pipeline-test"}') || {
        echo "ERROR: HP-1 FAILED: POST /ingest/{token}/rights did not return success" >&2
        return 1
     }
    if ! echo "$rights_output" | grep -q "rights_recorded"; then
        echo "ERROR: HP-1 FAILED: rights submission did not report rights_recorded" >&2
        echo "$rights_output" >&2
        return 1
    fi

    finalize_output=$(curl -sf -X POST "http://localhost:8090/ingest/${ingest_token}/finalize" \
         -H "Authorization: Bearer ${token}") || {
        echo "ERROR: HP-1 FAILED: POST /ingest/{token}/finalize did not return success" >&2
        return 1
     }
    asset_id=$(echo "$finalize_output" | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
    if [ -z "$asset_id" ]; then
        echo "ERROR: HP-1 FAILED: could not extract asset id from finalize response" >&2
        echo "$finalize_output" >&2
        return 1
    fi
    if ! echo "$asset_id" | grep -qE '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$'; then
        echo "ERROR: HP-1 FAILED: asset id from finalize response is not a well-formed UUID" >&2
        echo "$finalize_output" >&2
        return 1
    fi

    transcription_status=""
    for i in $(seq 1 60); do
        transcription_status=$(docker exec "$dep_container" psql -U dubbridge -d dubbridge -tAc "SELECT status FROM asset_transcription_status WHERE asset_id = '${asset_id}';" 2>/dev/null | tr -d '[:space:]')
        if [ "$transcription_status" = "ready" ] || [ "$transcription_status" = "failed" ]; then
            break
        fi
        sleep 2
    done

    if [ "$transcription_status" != "ready" ]; then
        echo "ERROR: HP-1 FAILED: asset_transcription_status did not reach 'ready' within timeout (last observed: '${transcription_status}')" >&2
        return 1
    fi

     # EC-1: finalize an already-finalized ingest token must fail, not silently succeed
    ec1_exit=0
    ec1_output=$(curl -sf -X POST "http://localhost:8090/ingest/${ingest_token}/finalize" \
         -H "Authorization: Bearer ${token}" 2>&1) || ec1_exit=$?
    if [ "$ec1_exit" -eq 0 ]; then
        echo "ERROR: EC-1 FAILED: re-finalizing an already-finalized ingest token succeeded" >&2
        echo "$ec1_output" >&2
        return 1
    fi

    echo "Run check passed for full-pipeline"
    return 0
}

