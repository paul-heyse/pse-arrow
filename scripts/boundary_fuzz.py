# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite pure inspection-codec exploration; corpus/minimization stays with this run."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import re
import subprocess
import time
from pathlib import Path
from scripts import arrow_validation, cargo_tools, validation
ROOT=Path(__file__).resolve().parents[1]


def main(argv:list[str]|None=None)->int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output",type=Path)
    parser.add_argument("--seconds",type=int,default=60)
    parser.add_argument("--replay",type=Path)
    parser.add_argument("--minimize",type=Path,help="minimize a failing input rather than run a new campaign")
    args=parser.parse_args(argv)
    if not 1<=args.seconds<=600 or (args.replay and args.minimize):
        parser.error("select one operation and a duration from 1 to 600 seconds")
    if os.environ.get("CARGO_ENCODED_RUSTFLAGS") is not None:
        parser.error("cargo-fuzz 0.13.2 sets RUSTFLAGS; CARGO_ENCODED_RUSTFLAGS would suppress its instrumentation; unset it explicitly")
    tool=cargo_tools.selected_identity("cargo-fuzz")
    output=validation.fresh_output(ROOT,args.output,"boundary-fuzz")
    host=next(line.removeprefix("host: ") for line in subprocess.check_output(["rustc","-vV"],text=True).splitlines() if line.startswith("host: "))
    seed=arrow_validation.compose(["cargo","run","--locked","-p","xtask","--features","fuzz-inspection","--bin","inspection-fuzz-seeds","--",str(output/"corpus")])
    check=arrow_validation.resolve(["cargo","check","--locked","-p","xtask","--features","fuzz-inspection","--bin","frozen-inspection-fuzz","--target",host])
    record={"schema":"boundary-fuzz-v1","tool":tool,"compiler":subprocess.check_output(["rustc","-vV"],text=True),
            "host":host,"seed_command":seed,"correctness_command":check.command,"source_inputs":validation.sources(ROOT),
            "environment":validation.relevant_environment(),"requested_seconds":args.seconds,
            "operation":"minimize" if args.minimize else "replay" if args.replay else "campaign",
            "instrumentation":{"tool":"cargo-fuzz 0.13.2","sanitizer":"address","target":host,"profile":"dev", "expected_feedback":"sancov-module level4, inline8bit counters, pc-table, trace-compares", "opaque_native_code":"not qualified"},
            "limits":["pure decoder and member validation; no server/solver","finite exploration is not exhaustive qualification"]}
    validation.write_json(output/"fuzz.json",record)
    with (output/"seed.log").open("w") as log:
        code=subprocess.run(seed,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=False).returncode
    if code:
        record.update(seed_exit=code,status="setup-failed");validation.write_json(output/"fuzz.json",record);return code
    record["seed_controls"] = json.loads((output/"seed-controls.json").read_text())
    record["initial_corpus"] = {path.name:{"bytes":path.stat().st_size,"sha256":hashlib.sha256(path.read_bytes()).hexdigest()} for path in sorted((output/"corpus").iterdir()) if path.is_file()}
    features=",".join(check.command[index+1] for index,arg in enumerate(check.command) if arg in {"--features","-F"})
    command=["cargo","fuzz","tmin" if args.minimize else "run","--fuzz-dir","xtask","--dev","--verbose","--sanitizer","address","--target",host,
             "--features",features,"frozen-inspection-fuzz",str(args.minimize or args.replay or output/"corpus"),"--",
             f"-max_total_time={args.seconds}","-max_len=65536","-timeout=5",f"-artifact_prefix={output}/"]
    if args.replay:
        command.append("-runs=1")
    record["command"]=command
    validation.write_json(output/"fuzz.json",record)
    started=time.monotonic()
    with (output/"fuzzer.log").open("w") as log:
        code=subprocess.run(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,env=dict(os.environ),check=False).returncode
    log=(output/"fuzzer.log").read_text(errors="replace")
    completion=re.search(r"Done (\d+) runs in (\d+) second",log)
    record.update(exit_code=code,status="completed" if code==0 else "inspect-finding-or-tool-failure",unchanged_inputs=record["source_inputs"]==validation.sources(ROOT),
                  elapsed_build_and_execution_seconds=time.monotonic()-started,
                  feedback_observed="inline 8-bit counters" in log and "PC tables" in log,
                  completion={"runs":int(completion[1]),"seconds":int(completion[2])} if completion else None,
                  findings=[path.name for path in output.iterdir() if path.name.startswith(("crash-","timeout-","oom-","leak-","minimized-"))])
    if code==0 and args.replay is None and args.minimize is None and (not record["feedback_observed"] or completion is None):
        record["status"]="instrumentation-or-completion-unavailable"
        code=1
    validation.write_json(output/"fuzz.json",record)
    print(output/"fuzz.json")
    return code


if __name__=="__main__":
    raise SystemExit(main())
