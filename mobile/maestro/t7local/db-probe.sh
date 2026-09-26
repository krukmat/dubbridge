#!/usr/bin/env bash
# Read-only PostgreSQL evidence probes for S-230-T7local.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
compose_file="$repo_root/infra/local/docker-compose.yml"
timeout_seconds="${T7LOCAL_DB_TIMEOUT:-240}"
poll_seconds="${T7LOCAL_DB_POLL_SECONDS:-2}"

usage() {
  cat <<'EOF'
Usage:
  db-probe.sh wait-c1 <proof_reference>
  db-probe.sh wait-c2 <asset_id>
  db-probe.sh wait-review-task <asset_id>
  db-probe.sh review-context <review_task_id>
  db-probe.sh verify-c3 <review_task_id>
  db-probe.sh verify-c4 <review_task_id>
EOF
}

psql_query() {
  local sql="$1"
  docker-compose -f "$compose_file" exec -T postgres     psql -U dubbridge -d dubbridge -At -v ON_ERROR_STOP=1 -c "$sql"
}

require_safe_ref() {
  local value="$1"
  [[ "$value" =~ ^[A-Za-z0-9._:-]+$ ]] || {
    echo "unsafe proof reference" >&2
    exit 2
  }
}

require_uuid() {
  local value="$1"
  [[ "$value" =~ ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$ ]] || {
    echo "invalid UUID: $value" >&2
    exit 2
  }
}

deadline_epoch() {
  echo $(( $(date +%s) + timeout_seconds ))
}

wait_c1() {
  local proof="$1"
  require_safe_ref "$proof"
  local deadline row asset_id status rights_count source_count
  deadline="$(deadline_epoch)"
  while (( $(date +%s) <= deadline )); do
    row="$(psql_query "
      SELECT a.id::text || '|' || a.status || '|' ||
             (SELECT count(*) FROM rights_records r2 WHERE r2.asset_id = a.id)::text || '|' ||
             (SELECT count(*) FROM artifact_records ar
                WHERE ar.asset_id = a.id AND ar.parent_artifact_id IS NULL)::text
      FROM assets a
      JOIN rights_records rr ON rr.asset_id = a.id
      WHERE rr.proof_reference = '$proof'
      ORDER BY rr.created_at DESC
      LIMIT 1;
    ")"
    if [[ -n "$row" ]]; then
      IFS='|' read -r asset_id status rights_count source_count <<<"$row"
      if [[ "$status" == "finalized" && "$rights_count" -ge 1 && "$source_count" -ge 1 ]]; then
        echo "C1=PASS asset_id=$asset_id status=$status rights=$rights_count source_artifacts=$source_count"
        echo "$asset_id"
        return 0
      fi
    fi
    sleep "$poll_seconds"
  done
  echo "C1=BLOCKED proof_reference=$proof row=${row:-none}" >&2
  return 1
}

wait_c2() {
  local asset_id="$1"
  require_uuid "$asset_id"
  local deadline status detail counts probe manifest segments
  deadline="$(deadline_epoch)"
  while (( $(date +%s) <= deadline )); do
    status="$(psql_query "SELECT COALESCE(status, '') FROM asset_preparation_status WHERE asset_id = '$asset_id'::uuid;")"
    if [[ "$status" == "failed" ]]; then
      detail="$(psql_query "SELECT COALESCE(error_detail, '') FROM asset_preparation_status WHERE asset_id = '$asset_id'::uuid;")"
      echo "C2=BLOCKED asset_id=$asset_id preparation=failed detail=$detail" >&2
      return 1
    fi
    if [[ "$status" == "ready" ]]; then
      counts="$(psql_query "
        SELECT
          COALESCE(SUM(CASE WHEN kind = 'probe_metadata' THEN 1 ELSE 0 END),0)::text || '|' ||
          COALESCE(SUM(CASE WHEN kind = 'hls_manifest' THEN 1 ELSE 0 END),0)::text || '|' ||
          COALESCE(SUM(CASE WHEN kind = 'hls_segment' THEN 1 ELSE 0 END),0)::text
        FROM artifact_records
        WHERE asset_id = '$asset_id'::uuid AND parent_artifact_id IS NOT NULL;
      ")"
      IFS='|' read -r probe manifest segments <<<"$counts"
      if [[ "$probe" -ge 1 && "$manifest" -ge 1 && "$segments" -ge 1 ]]; then
        echo "C2=PASS asset_id=$asset_id preparation=ready probe=$probe hls_manifest=$manifest hls_segments=$segments"
        return 0
      fi
      echo "C2=BLOCKED asset_id=$asset_id preparation=ready incomplete_artifacts=$counts" >&2
      return 1
    fi
    sleep "$poll_seconds"
  done
  echo "C2=BLOCKED asset_id=$asset_id preparation=${status:-missing} timeout=${timeout_seconds}s" >&2
  return 1
}

wait_review_task() {
  local asset_id="$1"
  require_uuid "$asset_id"
  local deadline task_id
  deadline="$(deadline_epoch)"
  while (( $(date +%s) <= deadline )); do
    task_id="$(psql_query "
      SELECT id::text
      FROM review_tasks
      WHERE asset_id = '$asset_id'::uuid
      ORDER BY created_at DESC, id DESC
      LIMIT 1;
    ")"
    if [[ -n "$task_id" ]]; then
      echo "REVIEW_TASK=PASS asset_id=$asset_id review_task_id=$task_id"
      echo "$task_id"
      return 0
    fi
    sleep "$poll_seconds"
  done
  local transcription subtitle translation
  transcription="$(psql_query "
    SELECT status || ':' || COALESCE(error_detail, '')
    FROM asset_transcription_status
    WHERE asset_id = '$asset_id'::uuid;
  ")"
  subtitle="$(psql_query "
    SELECT status || ':' || COALESCE(error_detail, '')
    FROM asset_subtitle_status
    WHERE asset_id = '$asset_id'::uuid;
  ")"
  translation="$(psql_query "
    SELECT COALESCE(
      string_agg(
        target_language_id::text || ':' || status || ':' || COALESCE(error_detail, ''),
        ';' ORDER BY target_language_id::text
      ),
      ''
    )
    FROM asset_translation_status
    WHERE asset_id = '$asset_id'::uuid;
  ")"
  echo "REVIEW_TASK=BLOCKED asset_id=$asset_id timeout=${timeout_seconds}s transcription=${transcription:-missing} subtitle=${subtitle:-missing} translation=${translation:-missing}" >&2
  return 1
}

review_context() {
  local task_id="$1"
  require_uuid "$task_id"
  local row org_name run_id asset_id org_id project_id
  row="$(psql_query "
    SELECT o.name || '|' || rt.asset_id::text || '|' || rt.org_id::text || '|' || rt.project_id::text
    FROM review_tasks rt
    JOIN organizations o ON o.id = rt.org_id
    WHERE rt.id = '$task_id'::uuid
    LIMIT 1;
  ")"
  if [[ -z "$row" ]]; then
    echo "REVIEW_CONTEXT=BLOCKED review_task_id=$task_id missing" >&2
    return 1
  fi
  IFS='|' read -r org_name asset_id org_id project_id <<<"$row"
  if [[ "$org_name" != T7local-* ]]; then
    echo "REVIEW_CONTEXT=BLOCKED review_task_id=$task_id unexpected_org_name=$org_name" >&2
    return 1
  fi
  run_id="${org_name#T7local-}"
  if [[ -z "$run_id" ]]; then
    echo "REVIEW_CONTEXT=BLOCKED review_task_id=$task_id missing_run_id" >&2
    return 1
  fi
  echo "REVIEW_CONTEXT=PASS review_task_id=$task_id run_id=$run_id asset_id=$asset_id org_id=$org_id project_id=$project_id"
  echo "$run_id|$asset_id|$org_id|$project_id"
}

verify_c3() {
  local task_id="$1"
  require_uuid "$task_id"
  local row
  row="$(psql_query "
    SELECT verdict || '|' || happened_at::text
    FROM review_decisions
    WHERE review_task_id = '$task_id'::uuid
    ORDER BY happened_at DESC, id DESC
    LIMIT 1;
  ")"
  if [[ "${row%%|*}" == "approved" ]]; then
    echo "C3=PASS review_task_id=$task_id decision=$row"
    return 0
  fi
  echo "C3=BLOCKED review_task_id=$task_id latest_decision=${row:-none}" >&2
  return 1
}

verify_c4() {
  local task_id="$1"
  require_uuid "$task_id"
  local row
  row="$(psql_query "
    SELECT state || '|' || published_at::text
    FROM publications
    WHERE review_task_id = '$task_id'::uuid
    LIMIT 1;
  ")"
  if [[ "${row%%|*}" == "published" ]]; then
    echo "C4_DB=PASS review_task_id=$task_id publication=$row"
    return 0
  fi
  echo "C4_DB=BLOCKED review_task_id=$task_id publication=${row:-none}" >&2
  return 1
}

[[ $# -ge 2 ]] || { usage >&2; exit 2; }
command="$1"
value="$2"

case "$command" in
  wait-c1) wait_c1 "$value" ;;
  wait-c2) wait_c2 "$value" ;;
  wait-review-task) wait_review_task "$value" ;;
  review-context) review_context "$value" ;;
  verify-c3) verify_c3 "$value" ;;
  verify-c4) verify_c4 "$value" ;;
  *) usage >&2; exit 2 ;;
esac
