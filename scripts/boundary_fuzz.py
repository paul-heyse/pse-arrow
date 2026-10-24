# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite pure inspection-codec exploration; corpus/minimization stays with this run."""
from __future__ import annotations
import argparse
import json
import os
import subprocess
from pathlib import Path
from scripts import arrow_validation, cargo_tools, validation
ROOT=Path(__file__).resolve().parents[1]


def main(argv:list[str]|None=None)->int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--seconds",type=int,default=60)
    parser.add_argument("--replay",type=Path)
    parser.add_argument("--minimize",type=Path,help="minimize a failing input rather than run a new campaign")
    args=parser.parse_args(argv)
    if not 1<=args.seconds<=600 or (args.replay and args.minimize):
        parser.error("select one operation and a duration from 1 to 600 seconds")
    tool=cargo_tools.selected_identity("cargo-fuzz")
    output=validation.fresh_output(ROOT,args.output,"boundary-fuzz")
    host=next(line.removeprefix("host: ") for line in subprocess.check_output(["rustc","-vV"],text=True).splitlines() if line.startswith("host: "))
    seed=arrow_validation.compose(["cargo","run","--locked","-p","xtask","--features","fuzz-inspection","--bin","inspection-fuzz-seeds","--",str(output/"corpus")])
    check=arrow_validation.resolve(["cargo","check","--locked","-p","xtask","--features","fuzz-inspection","--bin","frozen-inspection-fuzz","--target",host])
    record={"schema":"boundary-fuzz-v1","tool":tool,"compiler":subprocess.check_output(["rustc","-vV"],text=True),
            "host":host,"seed_command":seed,"correctness_command":check.command,"source_inputs":validation.sources(ROOT),
            "limits":["pure decoder and member validation; no server/solver","finite exploration is not exhaustive qualification"]}
    validation.write_json(output/"fuzz.json",record)
    code=subprocess.run(seed,cwd=ROOT,check=False).returncode
    if code:
        record.update(seed_exit=code,status="setup-failed");validation.write_json(output/"fuzz.json",record);return code
    features="xtask/fuzz-inspection,xtask/force-validate"
    command=["cargo","fuzz","tmin" if args.minimize else "run","--fuzz-dir","xtask","--dev","--target",host,
             "--features",features,"frozen-inspection-fuzz",str(args.minimize or args.replay or output/"corpus"),"--",
             f"-max_total_time={args.seconds}","-max_len=65536","-timeout=5",f"-artifact_prefix={output}/"]
    if args.replay:
        command.append("-runs=1")
    record["command"]=command
    validation.write_json(output/"fuzz.json",record)
    with (output/"fuzzer.log").open("w") as log:
        code=subprocess.run(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,env=dict(os.environ),check=False).returncode
    record.update(exit_code=code,status="completed" if code==0 else "inspect-finding-or-tool-failure",unchanged_inputs=record["source_inputs"]==validation.sources(ROOT))
    validation.write_json(output/"fuzz.json",record)
    print(output/"fuzz.json")
    return code


if __name__=="__main__":
    raise SystemExit(main())
