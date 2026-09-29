#!/usr/bin/env bash
set -euo pipefail

HOST="${DO_WORDPRESS_SSH_HOST:-46.101.217.151}"
USER="${DO_WORDPRESS_SSH_USER:-root}"
PORT="${DO_WORDPRESS_SSH_PORT:-22}"
OUT="${DO_WORDPRESS_BACKUP_DIR:-/tmp/dubbridge-t6d2a-wordpress-backup}"
REMOTE_TMP="${DO_WORDPRESS_REMOTE_TMP:-/tmp/dubbridge-t6d2a-wordpress-backup}"
SSH_TARGET="${USER}@${HOST}"
INTERACTIVE="${DO_WORDPRESS_SSH_INTERACTIVE:-0}"

SSH_OPTS=(-p "${PORT}" -o ConnectTimeout=15 -o StrictHostKeyChecking=accept-new)
if [[ "${INTERACTIVE}" == "1" ]]; then
  SSH_OPTS+=(-o BatchMode=no -o PreferredAuthentications=publickey,password)
else
  SSH_OPTS+=(-o BatchMode=yes)
fi

fail() {
  echo "T6D2A_BACKUP=BLOCKED reason=$1" >&2
  exit 2
}

command -v ssh >/dev/null 2>&1 || fail "missing-ssh"
command -v shasum >/dev/null 2>&1 || fail "missing-shasum"

rm -rf "${OUT}"
mkdir -p "${OUT}"

echo "T6D2A_TARGET=${SSH_TARGET}"

set +e
ssh_probe_output="$(ssh "${SSH_OPTS[@]}" "${SSH_TARGET}" 'true' 2>&1)"
ssh_probe_status=$?
set -e

if [[ "${ssh_probe_status}" -ne 0 ]]; then
  printf '%s\n' "${ssh_probe_output}" >&2

  if grep -Eqi 'Permission denied|publickey|authentication failed' <<<"${ssh_probe_output}"; then
    fail "ssh-authentication-failed"
  elif grep -Eqi 'Connection timed out|Operation timed out|No route to host|Connection refused|Could not resolve hostname' <<<"${ssh_probe_output}"; then
    fail "ssh-transport-unreachable"
  else
    fail "ssh-probe-failed"
  fi
fi

echo "T6D2A_SSH=PASS"

ssh "${SSH_OPTS[@]}" "${SSH_TARGET}" "sh -s -- '${REMOTE_TMP}'" <<'REMOTE'
set -eu
OUT="$1"
rm -rf "$OUT"
mkdir -p "$OUT"

WP_ROOT=""
for d in /var/www/html /var/www/wordpress /srv/www/wordpress /opt/wordpress; do
  if [ -f "$d/wp-config.php" ]; then
    WP_ROOT="$d"
    break
  fi
done

if [ -z "$WP_ROOT" ]; then
  WP_CONFIG="$(find /var/www /srv/www /opt -maxdepth 4 -type f -name wp-config.php 2>/dev/null | head -n 1 || true)"
  [ -n "$WP_CONFIG" ] || { echo "REMOTE_BACKUP=BLOCKED reason=wp-config-not-found" >&2; exit 21; }
  WP_ROOT="$(dirname "$WP_CONFIG")"
fi

printf "%s\n" "$WP_ROOT" > "$OUT/wp-root.txt"

command -v wp >/dev/null 2>&1 || {
  echo "REMOTE_BACKUP=BLOCKED reason=wp-cli-missing" >&2
  exit 22
}

wp --allow-root --path="$WP_ROOT" db export "$OUT/wordpress.sql" >/dev/null
[ -s "$OUT/wordpress.sql" ] || { echo "REMOTE_BACKUP=BLOCKED reason=empty-db-export" >&2; exit 23; }

tar -C "$(dirname "$WP_ROOT")" -czf "$OUT/wordpress-files.tar.gz" "$(basename "$WP_ROOT")"
[ -s "$OUT/wordpress-files.tar.gz" ] || { echo "REMOTE_BACKUP=BLOCKED reason=empty-files-archive" >&2; exit 24; }

{
  echo "hostname=$(hostname)"
  echo "wp_root=$WP_ROOT"
  echo "created_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} > "$OUT/manifest.env"

if command -v systemctl >/dev/null 2>&1; then
  systemctl list-unit-files --type=service 2>/dev/null | grep -E "nginx|apache|mysql|mariadb|php.*fpm" > "$OUT/services.txt" || true
fi

if command -v docker >/dev/null 2>&1; then
  docker ps --format "{{.ID}} {{.Image}} {{.Names}} {{.Ports}}" > "$OUT/docker-ps.txt" 2>/dev/null || true
fi

(cd "$OUT" && sha256sum wordpress.sql wordpress-files.tar.gz > SHA256SUMS)
echo "REMOTE_BACKUP=PASS"
REMOTE

ssh "${SSH_OPTS[@]}" "${SSH_TARGET}" "tar -C '$(dirname "${REMOTE_TMP}")' -czf - '$(basename "${REMOTE_TMP}")'" > "${OUT}/remote-backup.tar.gz" || fail "download-failed"
tar -xzf "${OUT}/remote-backup.tar.gz" -C "${OUT}"
rm -f "${OUT}/remote-backup.tar.gz"

LOCAL_DIR="${OUT}/$(basename "${REMOTE_TMP}")"
[[ -s "${LOCAL_DIR}/wordpress.sql" ]] || fail "missing-db-export"
[[ -s "${LOCAL_DIR}/wordpress-files.tar.gz" ]] || fail "missing-files-archive"
[[ -f "${LOCAL_DIR}/SHA256SUMS" ]] || fail "missing-checksums"

while read -r hash file; do
  actual="$(shasum -a 256 "${LOCAL_DIR}/${file}" | awk '{print $1}')"
  [[ "${actual}" == "${hash}" ]] || fail "checksum-verification-failed"
done < "${LOCAL_DIR}/SHA256SUMS"

cat > "${OUT}/restore-manifest.txt" <<EOF
host=${HOST}
ssh_user=${USER}
local_backup=${LOCAL_DIR}
db_export=${LOCAL_DIR}/wordpress.sql
files_archive=${LOCAL_DIR}/wordpress-files.tar.gz
checksums=${LOCAL_DIR}/SHA256SUMS
EOF

echo "T6D2A_DB_EXPORT=PASS"
echo "T6D2A_FILES_ARCHIVE=PASS"
echo "T6D2A_CHECKSUMS=PASS"
echo "T6D2A_RESTORE_MANIFEST=${OUT}/restore-manifest.txt"
echo "T6D2A_BACKUP_DIR=${OUT}"
echo "T6D2A_BACKUP=PASS"
