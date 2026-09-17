#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
out_dir="${1:-${repo_root}/tmp/p2p-mtls}"

command -v openssl >/dev/null 2>&1 || {
  echo "openssl is required" >&2
  exit 1
}

mkdir -p "${out_dir}"
chmod 700 "${out_dir}"
work_dir="$(mktemp -d)"
trap 'rm -rf "${work_dir}"' EXIT

ca_key="${out_dir}/ca-key.pem"
ca_cert="${out_dir}/ca.pem"
server_key="${out_dir}/server-key.pem"
server_cert="${out_dir}/server-cert.pem"
client_key="${out_dir}/client-key.pem"
client_cert="${out_dir}/client-cert.pem"
client_identity="${out_dir}/client-identity.pem"
fingerprint_file="${out_dir}/client-fingerprint.txt"

openssl genrsa -out "${ca_key}" 3072 >/dev/null 2>&1
openssl req -x509 -new -sha256 -days 3650 \
  -key "${ca_key}" \
  -subj "/CN=DubBridge Local P2P CA" \
  -out "${ca_cert}"

cat >"${work_dir}/server.ext" <<'EOF'
subjectAltName=DNS:availability-node,DNS:localhost,IP:127.0.0.1
extendedKeyUsage=serverAuth
keyUsage=digitalSignature,keyEncipherment
EOF
openssl genrsa -out "${server_key}" 3072 >/dev/null 2>&1
openssl req -new -key "${server_key}" -subj "/CN=availability-node" -out "${work_dir}/server.csr"
openssl x509 -req -sha256 -days 825 \
  -in "${work_dir}/server.csr" \
  -CA "${ca_cert}" -CAkey "${ca_key}" -CAcreateserial \
  -extfile "${work_dir}/server.ext" \
  -out "${server_cert}" >/dev/null 2>&1

cat >"${work_dir}/client.ext" <<'EOF'
extendedKeyUsage=clientAuth
keyUsage=digitalSignature,keyEncipherment
EOF
openssl genrsa -out "${client_key}" 3072 >/dev/null 2>&1
openssl req -new -key "${client_key}" -subj "/CN=dubbridge-worker-runner-local" -out "${work_dir}/client.csr"
openssl x509 -req -sha256 -days 825 \
  -in "${work_dir}/client.csr" \
  -CA "${ca_cert}" -CAkey "${ca_key}" -CAcreateserial \
  -extfile "${work_dir}/client.ext" \
  -out "${client_cert}" >/dev/null 2>&1

cat "${client_cert}" "${client_key}" >"${client_identity}"
openssl x509 -in "${client_cert}" -noout -fingerprint -sha256 \
  | cut -d= -f2 \
  | tr -d ':' \
  | tr '[:upper:]' '[:lower:]' \
  >"${fingerprint_file}"

chmod 600 "${ca_key}" "${server_key}" "${client_key}" "${client_identity}"
chmod 644 "${ca_cert}" "${server_cert}" "${client_cert}" "${fingerprint_file}"
rm -f "${out_dir}/ca.srl"

cat <<EOF
Local P2P mTLS material generated in:
  ${out_dir}

Client SHA-256 fingerprint:
  $(cat "${fingerprint_file}")

These files are local-only. The default path is under tmp/ and is git-ignored.
EOF
