#!/usr/bin/env bash
# Prepare one real local MP4 for T7local and expose it to Android DocumentsUI.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
compose_file="$repo_root/infra/local/docker-compose.yml"
filename="${T7LOCAL_FILENAME:-}"

if [[ -z "$filename" ]]; then
  echo "T7LOCAL_FILENAME is required" >&2
  exit 2
fi
if [[ ! "$filename" =~ ^[A-Za-z0-9._-]+\.mp4$ ]]; then
  echo "T7LOCAL_FILENAME must be a simple .mp4 basename" >&2
  exit 2
fi

command -v docker-compose >/dev/null 2>&1 || { echo "docker-compose is required" >&2; exit 2; }
command -v adb >/dev/null 2>&1 || { echo "adb is required" >&2; exit 2; }

serial="${T7LOCAL_EMULATOR_SERIAL:-}"
if [[ -z "$serial" ]]; then
  serial="$(adb devices | awk '/emulator-[0-9]+[[:space:]]+device/{print $1; exit}')"
fi
[[ -n "$serial" ]] || { echo "no running Android emulator found" >&2; exit 2; }

container_id="$(docker-compose -f "$compose_file" ps -q worker-runner)"
[[ -n "$container_id" ]] || { echo "worker-runner container is not running" >&2; exit 2; }

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/dubbridge-t7local-media.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT

container_path="/tmp/$filename"
speech_path="/tmp/t7local-speech.wav"
host_path="$tmp_dir/$filename"

# T7local exercises the real ASR -> subtitle -> translation path, so the
# fixture must contain intelligible speech rather than a synthetic tone.
# espeak-ng provides deterministic local speech without an external service.
docker-compose -f "$compose_file" exec -T worker-runner \
  espeak-ng -v en-us -s 135 -w "$speech_path" \
  "DubBridge local transcription test. This media contains real spoken words."

# H.264/AAC is representative of the normal preparation pipeline. The local
# worker image is the canonical ffmpeg-bearing runtime for this task.
docker-compose -f "$compose_file" exec -T worker-runner \
  ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i "testsrc=size=320x180:rate=12" \
    -i "$speech_path" \
    -shortest -c:v libx264 -pix_fmt yuv420p -c:a aac -movflags +faststart \
    "$container_path"

docker-compose -f "$compose_file" exec -T worker-runner rm -f "$speech_path"

docker cp "$container_id:$container_path" "$host_path" >/dev/null
adb -s "$serial" push "$host_path" "/sdcard/Download/$filename" >/dev/null
adb -s "$serial" shell am broadcast   -a android.intent.action.MEDIA_SCANNER_SCAN_FILE   -d "file:///sdcard/Download/$filename" >/dev/null 2>&1 || true

echo "T7LOCAL_MEDIA_READY=$filename"
echo "T7LOCAL_EMULATOR_SERIAL=$serial"
