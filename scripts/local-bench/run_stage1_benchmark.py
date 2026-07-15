#!/usr/bin/env python3
"""T7 orchestrator: runs the ADR-036 benchmark corpus (scripts/local-bench/cards/*.json)
through the local-agent runner (scripts/local-agent/run_local_task.py) against a real
Ollama binding, one isolated git worktree per card.

The runner's `test_runner` callback is operator-controlled (it runs the card's own
verify_commands, e.g. `make qa-mobile`), not model-controlled — it does not go through
boundary.check_command, which exists to restrict the untrusted model's own run_command
tool calls, not the harness that grades the model's work afterward.
"""

import argparse
import json
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "local-agent"))
import run_local_task as rlt
import boundary as boundary_mod

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
VERIFY_TIMEOUT_SECONDS = 600


def load_cards(cards_dir):
    cards = []
    for name in sorted(os.listdir(cards_dir)):
        if name.endswith(".json"):
            with open(os.path.join(cards_dir, name), encoding="utf-8") as f:
                cards.append(json.load(f))
    return cards


def make_test_runner(verify_commands):
    def test_runner(worktree_dir):
        outputs = []
        for cmd in verify_commands:
            try:
                completed = subprocess.run(
                    cmd,
                    shell=True,
                    cwd=worktree_dir,
                    capture_output=True,
                    text=True,
                    timeout=VERIFY_TIMEOUT_SECONDS,
                )
            except subprocess.TimeoutExpired as exc:
                outputs.append(f"$ {cmd}\n[TIMEOUT after {VERIFY_TIMEOUT_SECONDS}s]")
                return {"passed": False, "output": "\n\n".join(outputs), "error": str(exc)}

            outputs.append(
                f"$ {cmd}\n(exit {completed.returncode})\n{completed.stdout}\n{completed.stderr}"
            )
            if completed.returncode != 0:
                return {"passed": False, "output": "\n\n".join(outputs)}

        return {"passed": True, "output": "\n\n".join(outputs)}

    return test_runner


def setup_worktree(card_id, base_dir):
    worktree_path = os.path.join(base_dir, f"bench-{card_id.lower()}")
    branch = f"bench/{card_id.lower()}-{int(time.time())}"
    subprocess.run(
        ["git", "worktree", "add", "-b", branch, worktree_path, "HEAD"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return worktree_path, branch


def teardown_worktree(worktree_path, branch):
    subprocess.run(
        ["git", "worktree", "remove", "--force", worktree_path],
        cwd=REPO_ROOT,
        capture_output=False,
        check=False,
    )
    subprocess.run(
        ["git", "branch", "-D", branch],
        cwd=REPO_ROOT,
        capture_output=True,
        check=False,
    )


def run_card(card, base_dir, out_dir, host, model, keep_worktree=False):
    worktree_path, branch = setup_worktree(card["task_id"], base_dir)
    out_path = os.path.join(out_dir, f"{card['task_id']}.transcript.json")
    test_runner = make_test_runner(card["verify_commands"])

    session_start = time.monotonic()
    try:
        exit_code = rlt.main(
            [
                "--card", _write_temp_card(card, base_dir),
                "--worktree", worktree_path,
                "--out", out_path,
                "--host", host,
                "--model", model,
            ],
            test_runner=test_runner,
        )
    finally:
        elapsed = time.monotonic() - session_start
        if not keep_worktree:
            teardown_worktree(worktree_path, branch)

    with open(out_path, encoding="utf-8") as f:
        transcript = json.load(f)

    return {
        "task_id": card["task_id"],
        "category": card["category"],
        "exit_code": exit_code,
        "status": transcript["status"],
        "elapsed_s": round(elapsed, 1),
        "transcript_path": out_path,
    }


def _write_temp_card(card, base_dir):
    rlt_card = {
        "task_id": card["task_id"],
        "spec": card["spec"] + "\n\nAllowed paths: " + ", ".join(card["allowed_paths"]),
        "acceptance_tests": card["acceptance_tests"],
        "allowed_paths": card["allowed_paths"],
    }
    if "rri" in card:
        rlt_card["rri"] = card["rri"]
    if "band" in card:
        rlt_card["band"] = card["band"]
    path = os.path.join(base_dir, f"{card['task_id']}.card.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(rlt_card, f)
    return path


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cards-dir", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "cards"))
    parser.add_argument("--out-dir", required=True)
    parser.add_argument("--work-dir", required=True, help="Scratch dir for worktrees + temp card files.")
    parser.add_argument("--only", nargs="*", help="Task IDs to run (default: all cards found).")
    parser.add_argument("--host", default=os.environ.get("OLLAMA_HOST", "http://localhost:11434"))
    parser.add_argument("--model", default="qwen3.6:35b-a3b")
    parser.add_argument("--keep-worktree", action="store_true")
    args = parser.parse_args(argv)

    os.makedirs(args.out_dir, exist_ok=True)
    os.makedirs(args.work_dir, exist_ok=True)

    cards = load_cards(args.cards_dir)
    if args.only:
        cards = [c for c in cards if c["task_id"] in args.only]

    results = []
    for card in cards:
        print(f"[T7] running {card['task_id']} ({card['category']})...", file=sys.stderr)
        card_start = time.monotonic()
        try:
            result = run_card(card, args.work_dir, args.out_dir, args.host, args.model, args.keep_worktree)
        except Exception as exc:  # noqa: BLE001 - deliberate: isolate one card's
            # T7d-fix: found live when MC-01's malformed argv crashed
            # run_local_task.py with an uncaught exception that propagated
            # straight through run_card() (its own worktree-teardown `finally`
            # still ran, so no orphaned worktree/branch) and killed this whole
            # loop, silently discarding every subsequent card's result. This
            # is deliberately a broad except: it is the last line of defense
            # against failure modes not already converted into a structured
            # result by run_local_task.py itself, and must never let one
            # card's harness-level failure erase the rest of the batch.
            result = {
                "task_id": card["task_id"],
                "category": card["category"],
                "exit_code": None,
                "status": "harness_crash",
                "elapsed_s": round(time.monotonic() - card_start, 1),
                "error": str(exc),
            }
            print(f"[T7] {card['task_id']}: harness_crash ({exc})", file=sys.stderr)
        else:
            print(f"[T7] {card['task_id']}: {result['status']} in {result['elapsed_s']}s", file=sys.stderr)
        results.append(result)

    summary_path = os.path.join(args.out_dir, "summary.json")
    with open(summary_path, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"[T7] summary written to {summary_path}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
