#!/usr/bin/env sh
# Screenshot environment — source this before starting Metro for the screenshot suite.
#
# Usage (from repo root):
#   . mobile/maestro/screenshot-env.sh
#   cd mobile && npx expo start --port 8082 --clear
#
# Port map:
#   apps/gateway  :8081  (adb reverse tcp:8081 tcp:8081)
#   apps/api      :8080  (adb reverse tcp:8080 tcp:8080)
#   Metro         :8082  (adb reverse tcp:8082 tcp:8082)
#   mock-oauth    :9000  (host-only; gateway contacts it directly)
#
# gatewayBaseUrl=http://localhost:8081 works on the emulator because
# adb reverse maps emulator localhost:8081 -> host gateway :8081.

export DUBBRIDGE_ENV=local
export EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://localhost:8081
export EXPO_PUBLIC_E2E_ENABLED=true
