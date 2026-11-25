"""Synthetic deterministic runner probe, never an authority source.

Execute only under dedicated worker UID 472, with a scratch fixture
named haa-delta-fixture. Uses the same worker proposal path and actual
runner main, real file tools, Git scope check and pinned command runner.
Only the model response and audit destination are test fixtures.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
from unittest.mock import patch
from scripts.haa.worker_runner import main as worker_main

ROOT=Path(__file__).resolve().parents[2]
FIXTURE=Path("/Library/DubBridgeHAA/scratch/haa-delta-fixture")
CONTENT="synthetic approved candidate\n"


def fixture_runner(argv):
    sys.path.insert(0,str(ROOT/"scripts/local-agent"))
    import run_local_task as runner
    def tool(name,args):
        return {"tool_calls":[{"function":{"name":name,"arguments":json.dumps(args)}}]}
    responses=iter([tool("write_file",{"path":"README.md","content":CONTENT}),
                    tool("finish",{})])
    with patch.object(runner.gemma_local,"append_audit_log",return_value=None):
        return runner.main([*argv,"--context-provider","legacy"],
                           chat_fn=lambda messages:next(responses))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--socket",type=Path,required=True)
    args=p.parse_args()
    if (os.getuid()!=472 or os.geteuid()!=472 or FIXTURE!=FIXTURE.resolve()
            or (FIXTURE/"repo").resolve()!=FIXTURE/"repo"):
        raise SystemExit("SYNTHETIC_WORKER_REQUIRED")
    card=FIXTURE/"card.json"
    payload=json.loads(card.read_text())
    # Fixed synthetic contract, never an actual task/routing downgrade.
    if (payload.get("task_id")!="haa-delta-fixture"
            or payload.get("allowed_paths")!=["README.md"]
            or payload.get("verification_commands")!=[
                {"id":"check","criterion_ids":["AC-1"],"argv":["/usr/bin/true"]}]
            or payload.get("band")!="Moderate"):
        raise SystemExit("SYNTHETIC_CARD_REQUIRED")
    return worker_main(["--socket",str(args.socket),"--scratch",str(FIXTURE/"repo"),
        "--card",str(card),"--out",str(FIXTURE/"out.json"),"--path","README.md"],
        run_scratch=fixture_runner)


if __name__=="__main__":
    raise SystemExit(main())
