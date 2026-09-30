#!/usr/bin/env bash
set -euo pipefail

fail(){ echo "CONTABO_C2=BLOCKED reason=$1" >&2; exit 2; }

command -v cntb >/dev/null 2>&1 || fail "missing-contabo-cli"

echo "CONTABO_PRODUCT_ID=V154"
echo "CONTABO_PRODUCT=Cloud VPS 6"
echo "CONTABO_TARGET_CPU=6"
echo "CONTABO_TARGET_RAM_GB=12"
echo "CONTABO_TARGET_DISK_GB=200"
echo "CONTABO_REGION=EU"
echo "CONTABO_PERIOD_MONTHS=1"

echo "CONTABO_IMAGES_BEGIN"
cntb get images
echo "CONTABO_IMAGES_END"

echo "CONTABO_SSH_SECRETS_BEGIN"
cntb get secrets --type ssh
echo "CONTABO_SSH_SECRETS_END"

echo "CONTABO_C2_PREFLIGHT=PASS"
