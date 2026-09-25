#!/usr/bin/env bash
# Local-only P2P certification preflight. Never prints secrets (JWTs,
# passwords, invitation tokens, KEKs, or private-key material) — only pass/
# fail evidence about branch, process, and container state.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
compose_file="${repo_root}/infra/local/docker-compose.yml"
required_branch="feature/p2p-mvp-core"
api_base_url="${DUBBRIDGE_LOCAL_API_URL:-http://localhost:8080}"
runtime_checks="${1:-}"

fail=0
say_ok() { echo "OK   - $*"; }
say_fail() {
  echo "FAIL - $*" >&2
  fail=1
}

compose() {
  docker-compose -f "${compose_file}" "$@"
}

echo "== branch =="
current_branch="$(git -C "${repo_root}" rev-parse --abbrev-ref HEAD)"
if [[ "${current_branch}" == "${required_branch}" ]]; then
  say_ok "current branch is ${required_branch}"
else
  say_fail "current branch is '${current_branch}', expected '${required_branch}'"
fi

echo "== HEAD =="
head_sha="$(git -C "${repo_root}" rev-parse HEAD)"
echo "HEAD SHA: ${head_sha}"

echo "== mTLS material =="
mtls_dir="${repo_root}/tmp/p2p-mtls"
for name in ca.pem server-key.pem server-cert.pem client-identity.pem client-fingerprint.txt; do
  path="${mtls_dir}/${name}"
  if [[ -s "${path}" ]]; then
    say_ok "${name} exists and is non-empty"
  else
    say_fail "${name} is missing or empty at ${path}"
  fi
done

if [[ "${runtime_checks}" == "--with-runtime" ]]; then
  echo "== compose services =="
  ps_output="$(compose ps --format '{{.Name}} {{.State}}' 2>&1 || true)"
  for service in api availability-node worker-runner; do
    if echo "${ps_output}" | grep -qE "^local-${service}-1 running$"; then
      say_ok "${service} container is running"
    else
      say_fail "${service} container is not running (docker-compose ps: '${ps_output}')"
    fi
  done

  echo "== API health =="
  if curl -fsS -o /dev/null "${api_base_url}/health/live"; then
    say_ok "GET /health/live succeeded"
  else
    say_fail "GET /health/live failed against ${api_base_url}"
  fi

  if curl -fsS -o /dev/null "${api_base_url}/health/ready"; then
    say_ok "GET /health/ready succeeded"
  else
    say_fail "GET /health/ready failed against ${api_base_url}"
  fi

  echo "== port 8080 provenance (macOS/lsof only) =="
  if command -v lsof >/dev/null 2>&1; then
    listener_line="$(lsof -nP -iTCP:8080 -sTCP:LISTEN 2>/dev/null | awk 'NR==2 {print $2}' || true)"
    if [[ -z "${listener_line}" ]]; then
      say_fail "no process found listening on TCP 8080 (expected the Docker-proxied api container)"
    else
      listener_pid="${listener_line}"
      listener_full_command="$(ps -p "${listener_pid}" -o command= 2>/dev/null | sed -E 's/[[:space:]]+$//' || true)"
      # Docker's own port-forwarding process varies by backend: Docker Desktop
      # uses com.docker.backend/docker-proxy-style processes; Colima (used on
      # this host) forwards through an `ssh ... colima ...` mux process, never
      # the API binary itself. Match on the full command path/args, not just
      # the bare command name, so the non-Docker-provenance failure mode stays
      # any *other* process owning the socket (e.g. a stray host
      # `dubbridge-api` process left running outside Docker).
      case "${listener_full_command}" in
      *com.docke* | docker* | *docker* | *vpnkit* | *colima* | *lima* )
        say_ok "TCP 8080 listener is Docker-backend-owned (pid ${listener_pid}: ${listener_full_command})"
        ;;
      *)
        say_fail "TCP 8080 is owned by a non-Docker process (pid ${listener_pid}: ${listener_full_command}); stop it before running local P2P certification"
        ;;
      esac
    fi
  else
    say_fail "lsof is not available; cannot verify port 8080 provenance on this host"
  fi

  echo "== worker ffmpeg =="
  if compose exec -T worker-runner ffmpeg -version >/dev/null 2>&1; then
    say_ok "worker-runner container has ffmpeg"
  else
    say_fail "worker-runner container is missing ffmpeg or is not running"
  fi

  # Container stdout is colorized (pretty tracing/pino output), so strip ANSI
  # SGR escape sequences before matching literal substrings.
  strip_ansi() { sed -E $'s/\033\\[[0-9;]*m//g'; }

  echo "== worker P2P publication flag =="
  if compose logs --tail=500 worker-runner 2>&1 | strip_ansi | grep -F "p2p_publication_enabled=true" >/dev/null; then
    say_ok "recent worker-runner logs show p2p_publication_enabled=true"
  else
    say_fail "recent worker-runner logs do not show p2p_publication_enabled=true"
  fi

  echo "== availability node listening =="
  if compose logs --tail=500 availability-node 2>&1 | strip_ansi | grep -F "Availability Node listening" >/dev/null; then
    say_ok "recent availability-node logs show 'Availability Node listening'"
  else
    say_fail "recent availability-node logs do not show 'Availability Node listening'"
  fi
else
  echo "== runtime checks skipped =="
  echo "pass --with-runtime to also check compose services, API health, port 8080 provenance, ffmpeg, and logs"
fi

echo "=================="
if [[ "${fail}" -eq 0 ]]; then
  echo "PREFLIGHT: PASS"
  exit 0
else
  echo "PREFLIGHT: FAIL"
  exit 1
fi
