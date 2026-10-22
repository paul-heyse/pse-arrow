# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded actual operator consumer while the canonical fixture remains owned."""
from __future__ import annotations

import argparse
import json
import os
import time
from pathlib import Path

from scripts import surreal_diagnostics as diagnostics, surreal_server as server


def run(state:Path,database:str,kind:str,request:Path,evidence:Path) -> None:
    initializer=Path(os.environ["PSE_CANONICAL_TOOLS"])
    if not initializer.is_absolute() or not os.access(initializer,os.X_OK):
        raise ValueError("Actual operator control requires its captured canonical-tools artifact")
    selected=server.read_json(request)
    command=[str(initializer),"canonical-"+kind+"-inspect","--state",str(state),"--database",database]
    if kind=="graph":
        command.extend(("--request",str(request),"--experiment"))
    else:
        command.extend(("--study",str(selected["study"]),"--after",str(selected["after"]),"--experiment"))
    observed=diagnostics.syntax_capture(command,"",dict(os.environ),time.monotonic()+30)
    if observed["exit_code"]!=0:
        server.write_json(evidence,{"outcome":"failed","exit_code":observed["exit_code"],"stderr":observed["stderr"]})
        raise ValueError("Actual canonical inspection operator failed; retained bounded diagnostics")
    result=json.loads(str(observed["stdout"]))
    server.write_json(evidence,result)
    if kind=="graph":
        graph=result["neighborhood"]
        assert graph["anchor"]==selected["anchor"] and graph["scope"]==selected["scope"]
        assert len(graph["edges"])==1 and graph["complete"] is False and graph["continuation"] is not None
        assert result["physical"]["experimental_execution"] is True
    else:
        assert result["database"]==database and result["query"]["readiness_index"]=="study_ready_candidates"
        assert "study_ready_candidates" in json.dumps(result["observations"]["readiness_static"])
        samples=[metric for metric in result["statistics"]["metrics"] if metric["name"]=="canonical.query.latest_sample"]
        assert len(samples)==1
        latest=samples[0]["latest"]
        assert latest["kind"]=="Text"
        sample=json.loads(latest["value"])["s"]
        assert set(sample)=={"q","s","ns","ok"}
        assert sample["q"]>0 and sample["s"]==1 and sample["ok"] is True
        assert isinstance(sample["ns"],int) and sample["ns"]>=0


def main() -> None:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state",type=Path,required=True)
    parser.add_argument("--database",required=True)
    parser.add_argument("--kind",choices=("graph","query"),required=True)
    parser.add_argument("--request",type=Path,required=True)
    parser.add_argument("--evidence",type=Path,required=True)
    args=parser.parse_args()
    run(args.state,args.database,args.kind,args.request,args.evidence)


if __name__=="__main__":
    main()
