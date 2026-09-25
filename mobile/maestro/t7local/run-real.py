#!/usr/bin/env python3
"""S-230-T7local B3->C4 real-stack Maestro runner.

This is certification support only. It uses supported product APIs and read-only
PostgreSQL probes; it never seeds product tables or changes product source.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
T7_DIR = REPO_ROOT / "mobile" / "maestro" / "t7local"
GATEWAY = os.environ.get("T7LOCAL_HOST_GATEWAY_URL", "http://localhost:8082").rstrip("/")
RUN_ID = os.environ.get(
    "T7LOCAL_RUN_ID",
    datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"),
)
PROOF = os.environ.get("T7LOCAL_PROOF_REFERENCE", f"t7local-{RUN_ID}")
FILENAME = os.environ.get("T7LOCAL_FILENAME", f"t7local-{RUN_ID}.mp4")
OWNER = os.environ.get("T7LOCAL_OWNER", "DubBridge T7local")
SOURCE_LANG = os.environ.get("T7LOCAL_SOURCE_LANG", "en")
TARGET_LANG = os.environ.get("T7LOCAL_TARGET_LANG", "es-ES")
SUMMARY_DIR = Path(
    os.environ.get("T7LOCAL_OUTPUT_DIR", f"/tmp/dubbridge-t7local-{RUN_ID}")
)


def blocked(message: str) -> "NoReturn":
    print(f"T7LOCAL_MAESTRO=BLOCKED {message}", file=sys.stderr)
    raise SystemExit(1)


def command(
    args: list[str],
    *,
    env: dict[str, str] | None = None,
    capture: bool = False,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        args,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        capture_output=capture,
        check=False,
    )
    if result.returncode != 0:
        if capture:
            if result.stdout:
                print(result.stdout.rstrip(), file=sys.stderr)
            if result.stderr:
                print(result.stderr.rstrip(), file=sys.stderr)
        blocked(f"command failed ({result.returncode}): {args[0]}")
    return result


def http_json(
    method: str,
    path: str,
    *,
    payload: dict[str, object] | None = None,
    token: str | None = None,
) -> object:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    headers = {"content-type": "application/json"}
    if token:
        headers["authorization"] = f"Bearer {token}"
    request = urllib.request.Request(
        f"{GATEWAY}{path}",
        data=data,
        headers=headers,
        method=method,
    )
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            raw = response.read()
    except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError) as exc:
        blocked(f"{method} {path} failed: {exc}")
    if not raw:
        return {}
    try:
        value = json.loads(raw)
    except json.JSONDecodeError:
        blocked(f"{method} {path} returned invalid JSON")
    return value


def require_object(value: object, context: str) -> dict[str, object]:
    if not isinstance(value, dict):
        blocked(f"{context} returned non-object JSON")
    return value


def require_string(value: object, field: str) -> str:
    if not isinstance(value, str) or not value:
        blocked(f"response missing {field}")
    return value


def check_environment() -> str:
    for name in ("python3", "adb", "maestro", "docker-compose"):
        if shutil.which(name) is None:
            blocked(f"missing dependency: {name}")

    branch = command(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"], capture=True
    ).stdout.strip()
    if branch != "feature/p2p-mvp-core":
        blocked(f"wrong branch: {branch}")

    if command(["git", "status", "--porcelain"], capture=True).stdout.strip():
        blocked("working tree is not clean")

    if os.environ.get("EXPO_PUBLIC_E2E_ENABLED", "false").lower() == "true":
        blocked("EXPO_PUBLIC_E2E_ENABLED must be disabled")

    try:
        urllib.request.urlopen(f"{GATEWAY}/health/ready", timeout=10).read()
    except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError) as exc:
        blocked(f"gateway not ready at {GATEWAY}: {exc}")

    serial = os.environ.get("T7LOCAL_EMULATOR_SERIAL", "")
    if not serial:
        devices = command(["adb", "devices"], capture=True).stdout.splitlines()
        emulators = [
            line.split()[0]
            for line in devices
            if line.startswith("emulator-") and line.rstrip().endswith("\tdevice")
        ]
        if len(emulators) != 1:
            blocked(
                "expected exactly one running emulator or set "
                "T7LOCAL_EMULATOR_SERIAL"
            )
        serial = emulators[0]

    command(["adb", "-s", serial, "get-state"], capture=True)
    package = command(
        ["adb", "-s", serial, "shell", "pm", "path", "com.dubbridge.mobile"],
        capture=True,
    ).stdout.strip()
    if not package:
        blocked("com.dubbridge.mobile is not installed")
    return serial


def resolve_account() -> tuple[str, str, str]:
    email = os.environ.get("T7LOCAL_EMAIL", "")
    password = os.environ.get("T7LOCAL_PASSWORD", "")
    if bool(email) != bool(password):
        blocked("provide both T7LOCAL_EMAIL and T7LOCAL_PASSWORD, or neither")

    if not email:
        email = f"t7local-{RUN_ID}@dubbridge.dev"
        ui_safe_run_id = "".join(char for char in RUN_ID if char.isalnum()) or "run"
        password = f"T7local{ui_safe_run_id}A9zQ7"
        http_json(
            "POST",
            "/auth/register",
            payload={
                "email": email,
                "password": password,
                "workspaceName": f"T7local-{RUN_ID}",
            },
        )

    login = require_object(
        http_json(
            "POST",
            "/auth/login",
            payload={"email": email, "password": password},
        ),
        "POST /auth/login",
    )
    token = require_string(login.get("token"), "token")
    return email, password, token


def create_review_scope(token: str) -> tuple[str, str]:
    org = require_object(
        http_json(
            "POST",
            "/api/orgs",
            token=token,
            payload={"name": f"T7local-{RUN_ID}"},
        ),
        "POST /api/orgs",
    )
    org_id = require_string(org.get("id"), "organization id")

    project = require_object(
        http_json(
            "POST",
            f"/api/orgs/{org_id}/projects",
            token=token,
            payload={"name": f"T7local-{RUN_ID}", "asset_ids": []},
        ),
        "POST project",
    )
    project_id = require_string(project.get("id"), "project id")

    http_json(
        "PUT",
        f"/api/orgs/{org_id}/projects/{project_id}/target-languages",
        token=token,
        payload={
            "source_lang": SOURCE_LANG,
            "target_languages": [TARGET_LANG],
        },
    )
    return org_id, project_id


def maestro_env(serial: str) -> dict[str, str]:
    env = os.environ.copy()
    env["ANDROID_SERIAL"] = serial
    env["MAESTRO_CLI_NO_ANALYTICS"] = "1"

    java17 = Path(
        env.get(
            "T7LOCAL_JAVA_HOME",
            "/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home",
        )
    )
    if java17.exists():
        env["JAVA_HOME"] = str(java17)
        env["PATH"] = f"{java17 / 'bin'}:{env.get('PATH', '')}"
    return env


def reset_maestro_android_transport(serial: str) -> None:
    subprocess.run(
        ["adb", "-s", serial, "forward", "--remove-all"],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    subprocess.run(
        ["adb", "reconnect"],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    command(["adb", "-s", serial, "wait-for-device"], capture=True)


def diagnose_maestro_ui(serial: str, env: dict[str, str], flow: str) -> None:
    hierarchy = subprocess.run(
        ["maestro", "--device", serial, "hierarchy"],
        cwd=REPO_ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if hierarchy.returncode != 0:
        print(
            f"T7LOCAL_UI_DIAGNOSTIC=UNAVAILABLE flow={flow}",
            file=sys.stderr,
        )
        return

    ui = hierarchy.stdout
    for phase in ("authenticated", "persisting", "response_received", "requesting", "error", "idle"):
        if f"Login phase: {phase}" in ui:
            print(
                f"T7LOCAL_LOGIN_PHASE={phase} flow={flow}",
                file=sys.stderr,
            )
            break

    known = (
        (
            "Invalid email or password.",
            "LOGIN_REJECTED credentials rejected by mobile login request",
        ),
        (
            "We could not reach DubBridge. Try again.",
            "LOGIN_NETWORK Android app could not reach configured gateway",
        ),
        (
            "This app is missing its gateway configuration.",
            "LOGIN_CONFIG missing runtime gateway configuration",
        ),
        (
            "We could not securely store your session. Try again.",
            "SESSION_STORAGE secure session persistence failed",
        ),
        (
            "An unexpected sign-in error occurred. Try again.",
            "LOGIN_UNEXPECTED unexpected exception escaped the normal login result path",
        ),
    )
    for needle, diagnosis in known:
        if needle in ui:
            print(
                f"T7LOCAL_UI_DIAGNOSTIC={diagnosis}",
                file=sys.stderr,
            )
            return

    if "home-screen" in ui:
        print(
            f"T7LOCAL_UI_DIAGNOSTIC=HOME_VISIBLE selector timing mismatch flow={flow}",
            file=sys.stderr,
        )
    elif "login-screen" in ui:
        print(
            f"T7LOCAL_UI_DIAGNOSTIC=LOGIN_STILL_VISIBLE no known error copy flow={flow}",
            file=sys.stderr,
        )
    else:
        print(
            f"T7LOCAL_UI_DIAGNOSTIC=UNKNOWN flow={flow}",
            file=sys.stderr,
        )


def maestro(flow: str, values: dict[str, str], serial: str) -> None:
    args = ["maestro", "test"]
    for key, value in values.items():
        args.extend(["-e", f"{key}={value}"])
    args.append(str(T7_DIR / flow))
    env = maestro_env(serial)

    first = subprocess.run(
        args,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if first.returncode == 0:
        if first.stdout:
            print(first.stdout, end="")
        return

    combined = f"{first.stdout}\n{first.stderr}"
    transport_failure = (
        "StatusRuntimeException: UNAVAILABLE" in combined
        or "Command failed (tcp:" in combined
    )
    if not transport_failure:
        if first.stdout:
            print(first.stdout, end="", file=sys.stderr)
        if first.stderr:
            print(first.stderr, end="", file=sys.stderr)
        diagnose_maestro_ui(serial, env, flow)
        blocked(f"Maestro flow failed: {flow}")

    print(
        f"T7LOCAL_MAESTRO_TRANSPORT_RETRY flow={flow}",
        file=sys.stderr,
    )
    reset_maestro_android_transport(serial)

    second = subprocess.run(
        args,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if second.stdout:
        print(second.stdout, end="")
    if second.returncode != 0:
        if second.stderr:
            print(second.stderr, end="", file=sys.stderr)
        second_combined = f"{second.stdout}\n{second.stderr}"
        if not (
            "StatusRuntimeException: UNAVAILABLE" in second_combined
            or "Command failed (tcp:" in second_combined
        ):
            diagnose_maestro_ui(serial, env, flow)
        blocked(f"Maestro transport retry failed: {flow}")


def db_probe(action: str, value: str) -> list[str]:
    result = command(
        [str(T7_DIR / "db-probe.sh"), action, value],
        capture=True,
    )
    lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    if not lines:
        blocked(f"{action} returned no evidence")
    return lines


def main() -> None:
    serial = check_environment()
    email, password, token = resolve_account()
    org_id, project_id = create_review_scope(token)

    prep_env = os.environ.copy()
    prep_env["T7LOCAL_FILENAME"] = FILENAME
    prep_env["T7LOCAL_EMULATOR_SERIAL"] = serial
    command([str(T7_DIR / "prepare-media.sh")], env=prep_env, capture=True)

    maestro(
        "ingest-real.yaml",
        {
            "T7LOCAL_EMAIL": email,
            "T7LOCAL_PASSWORD": password,
            "T7LOCAL_OWNER": OWNER,
            "T7LOCAL_PROOF_REFERENCE": PROOF,
            "T7LOCAL_FILENAME": FILENAME,
        },
        serial,
    )

    c1 = db_probe("wait-c1", PROOF)
    print(c1[0])
    asset_id = c1[-1]

    http_json(
        "POST",
        f"/api/orgs/{org_id}/projects/{project_id}/assets",
        token=token,
        payload={"asset_id": asset_id},
    )

    c2 = db_probe("wait-c2", asset_id)
    print(c2[0])

    review = db_probe("wait-review-task", asset_id)
    print(review[0])
    review_task_id = review[-1]

    maestro(
        "review-publish-real.yaml",
        {
            "T7LOCAL_EMAIL": email,
            "T7LOCAL_PASSWORD": password,
            "T7LOCAL_REVIEW_TASK_ID": review_task_id,
        },
        serial,
    )

    print(db_probe("verify-c3", review_task_id)[0])
    print(db_probe("verify-c4", review_task_id)[0])

    maestro(
        "playback-real.yaml",
        {
            "T7LOCAL_EMAIL": email,
            "T7LOCAL_PASSWORD": password,
            "T7LOCAL_ASSET_ID": asset_id,
        },
        serial,
    )

    SUMMARY_DIR.mkdir(parents=True, exist_ok=True)
    SUMMARY_DIR.chmod(0o700)
    head_sha = command(["git", "rev-parse", "HEAD"], capture=True).stdout.strip()
    summary = SUMMARY_DIR / "summary.env"
    summary.write_text(
        "\n".join(
            [
                f"T7LOCAL_RUN_ID={RUN_ID}",
                f"T7LOCAL_HEAD={head_sha}",
                f"T7LOCAL_EMULATOR_SERIAL={serial}",
                f"T7LOCAL_GATEWAY={GATEWAY}",
                f"T7LOCAL_PROOF_REFERENCE={PROOF}",
                f"T7LOCAL_FILENAME={FILENAME}",
                f"T7LOCAL_ORG_ID={org_id}",
                f"T7LOCAL_PROJECT_ID={project_id}",
                f"T7LOCAL_ASSET_ID={asset_id}",
                f"T7LOCAL_REVIEW_TASK_ID={review_task_id}",
                "B3=PASS",
                "C1=PASS",
                "C2=PASS",
                "C3=PASS",
                "C4=PASS",
                "",
            ]
        ),
        encoding="utf-8",
    )
    summary.chmod(0o600)

    print("T7LOCAL_MAESTRO=PASS")
    print(f"HEAD={head_sha}")
    print(f"RUN_ID={RUN_ID}")
    print(f"ASSET_ID={asset_id}")
    print(f"REVIEW_TASK_ID={review_task_id}")
    print(f"SUMMARY={summary}")


if __name__ == "__main__":
    main()
