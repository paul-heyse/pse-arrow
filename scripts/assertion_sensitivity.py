# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded cargo-mutants consumer through existing correctness and native owners."""
from __future__ import annotations
import argparse
import json
import os
import subprocess
from pathlib import Path
from scripts import arrow_validation, cargo_tools, native_tests, validation

ROOT=Path(__file__).resolve().parents[1]


def classification(outcome: str) -> str:
    return {"CaughtMutant":"caught", "MissedMutant":"missed", "Unviable":"unviable", "UnviableMutant":"unviable",
            "Timeout":"timeout", "Success":"baseline-passed", "Failure":"setup-failed"}.get(outcome,"unknown")


def run(path: str, output: Path, arguments: list[str], pattern: str | None, base: str | None, native: bool) -> int:
    target=(ROOT/path).resolve()
    target.relative_to(ROOT)
    if not target.is_file() or target.suffix!=".rs" or any(part in {"generated","tests","benches"} for part in target.relative_to(ROOT).parts):
        raise ValueError("select an explicit existing production Rust file; test-only diffs need their production target")
    output=validation.fresh_output(ROOT,output,"sensitivity")
    tool=cargo_tools.selected_identity("cargo-mutants")
    selection=native_tests.rust_command("run",arguments) if native else ["cargo","nextest","run","--locked",*arguments]
    resolution=arrow_validation.resolve(selection)
    cargo_arguments=resolution.command[3:]
    command=["cargo","mutants","--test-tool","nextest","--baseline","run","--copy-vcs=true",
             "--file",path,"--output",str(output),"--no-shuffle","--timeout","60","--build-timeout","900"]
    if pattern:
        command += ["--re",pattern]
    if base:
        patch=subprocess.check_output(["git","diff","--no-ext-diff","--no-renames",base,"--",path],cwd=ROOT)
        if not patch:
            raise ValueError("selected production target has no changes; omit --base for an explicit whole-file challenge")
        (output/"selected.diff").write_bytes(patch)
        command += ["--in-diff",str(output/"selected.diff")]
    command += ["--",*cargo_arguments]
    record={"schema":"assertion-sensitivity-v1","tool":tool,"command":command,"target":path,
            "correctness_command":resolution.command,"inputs":validation.sources(ROOT),
            "environment":validation.relevant_environment(),"native_requested":native,
            "limits":["a survivor is not automatically equivalent","bounded selected predicates only; no repository score"]}
    validation.write_json(output/"selection.json",record)
    code=subprocess.run(command,cwd=ROOT,env=dict(os.environ),check=False).returncode
    reports=list(output.glob("**/outcomes.json"))
    outcome=None
    if len(reports)==1:
        outcome=json.loads(reports[0].read_text())
    rows=outcome.get("outcomes",[]) if isinstance(outcome,dict) else []
    baseline=[row for row in rows if row.get("scenario")=="Baseline" or row.get("scenario")=={"Baseline":None}]
    # Keep original tool fields; unknown protocol shapes never become successful mutation evidence.
    record.update(exit_code=code,raw_outcomes=str(reports[0]) if len(reports)==1 else None,
        outcomes=[{"classification":classification(row.get("summary","")),"original":row} for row in rows],
        baseline_status="passed" if any(row.get("summary")=="Success" for row in baseline) else "inspect-original-tool-report",
        equivalent="not-inferred")
    validation.write_json(output/"sensitivity.json",record)
    print(output/"sensitivity.json")
    return code


def main(argv:list[str]|None=None)->int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path")
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--pattern",help="bounded cargo-mutants regex selection")
    parser.add_argument("--base",help="optional production diff selection")
    parser.add_argument("--native",action="store_true",help="compose selected native test root features; launch through pse-env --native")
    import sys
    supplied=list(sys.argv[1:] if argv is None else argv)
    boundary=supplied.index("--") if "--" in supplied else len(supplied)
    args=parser.parse_args(supplied[:boundary])
    arguments=supplied[boundary+1:]
    return run(args.path,args.output,arguments,args.pattern,args.base,args.native)


if __name__=="__main__":
    raise SystemExit(main())
