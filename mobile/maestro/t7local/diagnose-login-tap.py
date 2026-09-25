#!/usr/bin/env python3
"""B1 diagnostic: isolate Maestro submit-tap delivery on the real login screen.

This script does not modify product state beyond normal login attempts. It runs
the same real login preparation before each probe and compares four Maestro tap
strategies with an ADB coordinate tap control.
"""

from __future__ import annotations

import importlib.util
import os
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


T7_DIR = Path(__file__).resolve().parent
REPO_ROOT = T7_DIR.parents[2]
RUNNER_PATH = T7_DIR / "run-real.py"
RUN_ID = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
OUTPUT_DIR = Path(
    os.environ.get(
        "T7LOCAL_LOGIN_TAP_OUTPUT_DIR",
        f"/tmp/dubbridge-t7local-login-tap-{RUN_ID}",
    )
)
PHASES = (
    "authenticated",
    "persisting",
    "response_received",
    "requesting",
    "error",
    "idle",
)


def load_runner() -> Any:
    spec = importlib.util.spec_from_file_location("t7local_runner", RUNNER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {RUNNER_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


runner = load_runner()


def run(
    args: list[str],
    *,
    env: dict[str, str] | None = None,
    check: bool = False,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=check,
    )


def write_text(name: str, value: str) -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUTPUT_DIR / name).write_text(value, encoding="utf-8")


def maestro_test(
    flow: Path,
    env: dict[str, str],
    values: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    args = ["maestro", "test"]
    for key, value in (values or {}).items():
        args.extend(["-e", f"{key}={value}"])
    args.append(str(flow))
    return run(args, env=env)


def hierarchy(serial: str, env: dict[str, str]) -> str:
    result = run(["maestro", "--device", serial, "hierarchy"], env=env)
    return result.stdout if result.returncode == 0 else ""


def observable_state(ui: str) -> str:
    if "home-screen" in ui:
        return "home"
    for phase in PHASES:
        if f"Login phase: {phase}" in ui:
            return f"phase:{phase}"
    if "login-screen" in ui:
        return "login:no-phase"
    return "unknown"


def wait_for_probe_state(serial: str, env: dict[str, str], label: str) -> str:
    last_ui = ""
    last_state = "unknown"
    deadline = time.monotonic() + 6.0
    sample = 0
    while time.monotonic() < deadline:
        last_ui = hierarchy(serial, env)
        last_state = observable_state(last_ui)
        write_text(f"{label}-hierarchy-{sample:02d}.txt", last_ui)
        if last_state == "home":
            return last_state
        if last_state not in ("phase:idle", "login:no-phase", "unknown"):
            return last_state
        sample += 1
        time.sleep(0.35)
    return last_state


def dump_android_ui(serial: str) -> str:
    run(
        [
            "adb",
            "-s",
            serial,
            "shell",
            "uiautomator",
            "dump",
            "/sdcard/t7local-login-tap.xml",
        ]
    )
    result = run(
        [
            "adb",
            "-s",
            serial,
            "exec-out",
            "cat",
            "/sdcard/t7local-login-tap.xml",
        ]
    )
    return result.stdout


def parse_bounds(raw: str) -> tuple[int, int, int, int]:
    root = ET.fromstring(raw)
    candidates = []
    fallback = []
    for node in root.iter("node"):
        resource_id = node.attrib.get("resource-id", "")
        content_desc = node.attrib.get("content-desc", "")
        text = node.attrib.get("text", "")
        clickable = node.attrib.get("clickable", "") == "true"
        bounds = node.attrib.get("bounds", "")
        if not bounds:
            continue
        if resource_id.endswith("login-submit-button"):
            candidates.append(node)
        elif clickable and (content_desc == "Sign in" or text == "Sign in"):
            fallback.append(node)

    nodes = candidates or fallback
    if not nodes:
        raise RuntimeError("could not resolve login submit bounds from Android UI hierarchy")

    match = re.fullmatch(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]", nodes[0].attrib["bounds"])
    if match is None:
        raise RuntimeError(f"unexpected bounds: {nodes[0].attrib['bounds']}")
    return tuple(int(value) for value in match.groups())


def android_focus_snapshot(serial: str, label: str) -> None:
    window = run(
        ["adb", "-s", serial, "shell", "dumpsys", "window", "windows"]
    ).stdout
    ime = run(
        ["adb", "-s", serial, "shell", "dumpsys", "input_method"]
    ).stdout
    write_text(f"{label}-window.txt", window)
    write_text(f"{label}-input-method.txt", ime)

    focus_lines = [
        line.strip()
        for line in window.splitlines()
        if "mCurrentFocus" in line or "mFocusedApp" in line
    ]
    ime_lines = [
        line.strip()
        for line in ime.splitlines()
        if any(
            key in line
            for key in (
                "mServedView",
                "mNextServedView",
                "mInputShown",
                "mIsInputViewShown",
            )
        )
    ]
    write_text(
        f"{label}-focus-summary.txt",
        "\n".join(focus_lines + ime_lines) + "\n",
    )


def clear_logcat(serial: str) -> None:
    run(["adb", "-s", serial, "logcat", "-c"])


def save_logcat(serial: str, label: str) -> None:
    logs = run(["adb", "-s", serial, "logcat", "-d", "-v", "time"]).stdout
    write_text(f"{label}-logcat.txt", logs)


def prepare(
    serial: str,
    env: dict[str, str],
    email: str,
    password: str,
    label: str,
) -> tuple[int, int]:
    prep = maestro_test(
        T7_DIR / "login-tap-prepare-real.yaml",
        env,
        {
            "T7LOCAL_EMAIL": email,
            "T7LOCAL_PASSWORD": password,
        },
    )
    write_text(f"{label}-prepare-stdout.txt", prep.stdout)
    write_text(f"{label}-prepare-stderr.txt", prep.stderr)
    if prep.returncode != 0:
        raise RuntimeError(
            f"prepare failed for {label}; see {OUTPUT_DIR}/{label}-prepare-*.txt"
        )

    ui = hierarchy(serial, env)
    write_text(f"{label}-prepared-hierarchy.txt", ui)
    state = observable_state(ui)
    if state != "phase:idle":
        raise RuntimeError(f"unexpected prepared login state for {label}: {state}")

    xml = dump_android_ui(serial)
    write_text(f"{label}-uiautomator.xml", xml)
    x1, y1, x2, y2 = parse_bounds(xml)
    center = ((x1 + x2) // 2, (y1 + y2) // 2)
    write_text(
        f"{label}-bounds.txt",
        f"bounds=[{x1},{y1}][{x2},{y2}]\ncenter={center[0]},{center[1]}\n",
    )
    android_focus_snapshot(serial, label)
    return center


def write_probe_flow(label: str, command: str) -> Path:
    path = OUTPUT_DIR / f"{label}.yaml"
    path.write_text(
        "appId: com.dubbridge.mobile\n"
        f"name: T7local login tap probe {label}\n"
        "---\n"
        f"{command}\n",
        encoding="utf-8",
    )
    return path


def run_maestro_probe(
    serial: str,
    env: dict[str, str],
    label: str,
    command: str,
) -> str:
    clear_logcat(serial)
    flow = write_probe_flow(label, command)
    result = maestro_test(flow, env)
    write_text(f"{label}-tap-stdout.txt", result.stdout)
    write_text(f"{label}-tap-stderr.txt", result.stderr)
    state = wait_for_probe_state(serial, env, label)
    save_logcat(serial, label)
    return state


def run_adb_probe(
    serial: str,
    env: dict[str, str],
    label: str,
    x: int,
    y: int,
) -> str:
    clear_logcat(serial)
    tap = run(["adb", "-s", serial, "shell", "input", "tap", str(x), str(y)])
    write_text(f"{label}-tap-stdout.txt", tap.stdout)
    write_text(f"{label}-tap-stderr.txt", tap.stderr)
    state = wait_for_probe_state(serial, env, label)
    save_logcat(serial, label)
    return state


def classify(results: dict[str, str]) -> str:
    maestro = {
        name: state
        for name, state in results.items()
        if name != "adb_absolute"
    }
    maestro_pass = {
        name: state
        for name, state in maestro.items()
        if state != "phase:idle" and state != "login:no-phase" and state != "unknown"
    }
    adb_pass = results.get("adb_absolute") not in (
        "phase:idle",
        "login:no-phase",
        "unknown",
        None,
    )

    if not maestro_pass and adb_pass:
        return "MAESTRO_INJECTION_PATH"
    if maestro_pass and adb_pass:
        if "id_default" not in maestro_pass:
            return "MAESTRO_SELECTOR_OR_DEFAULT_TARGETING"
        return "MAESTRO_TAP_VARIANT_DEPENDENT"
    if not adb_pass:
        return "APP_OR_GEOMETRY_NOT_REPRODUCED"
    return "INCONCLUSIVE"


def main() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    OUTPUT_DIR.chmod(0o700)

    serial = runner.check_environment()
    email, password, _token = runner.resolve_account()
    env = runner.maestro_env(serial)

    probes: list[tuple[str, str | None]] = [
        (
            "id_default",
            "- tapOn:\n"
            "    id: login-submit-button\n"
            "    enabled: true",
        ),
        (
            "text_below_password",
            "- tapOn:\n"
            '    text: "Sign in"\n'
            '    below: "Password"',
        ),
        (
            "id_relative_center",
            "- tapOn:\n"
            "    id: login-submit-button\n"
            '    point: "50%,50%"',
        ),
        ("absolute_center", None),
        ("adb_absolute", None),
    ]

    results: dict[str, str] = {}
    reference_center: tuple[int, int] | None = None

    for label, command in probes:
        center = prepare(serial, env, email, password, label)
        if reference_center is None:
            reference_center = center

        if label == "adb_absolute":
            results[label] = run_adb_probe(
                serial,
                env,
                label,
                center[0],
                center[1],
            )
            continue

        if label == "absolute_center":
            command = (
                "- tapOn:\n"
                f'    point: "{center[0]},{center[1]}"'
            )

        if command is None:
            raise RuntimeError(f"missing Maestro command for {label}")
        results[label] = run_maestro_probe(serial, env, label, command)

    diagnosis = classify(results)
    summary_lines = [
        f"B1_RUN_ID={RUN_ID}",
        f"B1_EMULATOR_SERIAL={serial}",
        f"B1_REFERENCE_CENTER={reference_center[0]},{reference_center[1]}"
        if reference_center
        else "B1_REFERENCE_CENTER=unavailable",
    ]
    summary_lines.extend(
        f"B1_{name.upper()}={state}" for name, state in results.items()
    )
    summary_lines.append(f"B1_DIAGNOSIS={diagnosis}")
    summary_lines.append(f"B1_EVIDENCE={OUTPUT_DIR}")

    summary = "\n".join(summary_lines) + "\n"
    write_text("summary.env", summary)
    print(summary, end="")


if __name__ == "__main__":
    main()
