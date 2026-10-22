# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit disposable supervisor log restart and metadata-only audit control."""

from __future__ import annotations

import argparse
import json
import socket
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

from scripts import surreal_server as server

ROOT = Path(__file__).resolve().parents[2]


def mcp_call(state: Path) -> None:
    deadline = time.monotonic() + 15
    selected = server.selected_service(state, deadline)
    config = server.config_for(state)
    headers = {**selected.authorization(), "Content-Type": "application/json", "Accept": "application/json, text/event-stream"}
    def request(method: str, params: dict, identity: int) -> dict:
        operation = urllib.request.Request(f"http://127.0.0.1:{selected.port}/mcp", data=json.dumps({"jsonrpc":"2.0", "id":identity, "method":method, "params":params}).encode(), headers=headers)
        with urllib.request.urlopen(operation, timeout=server.remaining(deadline)) as response:  # noqa: S310 -- exact frozen fixture-owned loopback endpoint
            if session := response.headers.get("MCP-Session-Id"):
                headers["MCP-Session-Id"] = session
            raw = response.read((1 << 20)+1).decode()
        if len(raw) > 1 << 20:
            raise server.SupervisorError("Fixture MCP response exceeds its bound")
        messages = [line[5:].strip() for line in raw.splitlines() if line.startswith("data:")]
        return json.loads(messages[-1] if messages else raw)
    initialized = request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{}, "clientInfo":{"name":"pse-audit-fixture", "version":"1"}}, 1)
    headers["MCP-Protocol-Version"] = initialized["result"]["protocolVersion"]
    reply = request("tools/call", {"name":"query", "arguments":{"namespace": config["namespace"], "database": config["database"], "query":"RETURN 'query-payload-must-not-appear-in-audit'; RETURN mod::pse::study::readiness({generation:0dec,cancelled:false,rows:[],items:[]});"}}, 2)
    if "error" in reply or reply.get("result", {}).get("isError"):
        raise server.SupervisorError("Fixture MCP audit read failed")
    selected.recheck(deadline)


def run(state: Path, evidence: Path, initializer: Path, worker: Path) -> None:
    if state.exists() or evidence.exists():
        raise server.SupervisorError("Logging control requires fresh owned state and evidence")
    evidence.mkdir(mode=0o700, parents=True)
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    args = server.parser().parse_args(["setup", "--state", str(state), "--port", str(port), "--no-resident", "--tooling-fixture", "--memory-mib", "3072", "--server-memory-mib", "2048", "--native-workers", "1", "--native-worker-memory-mib", "1024", "--worker-executable", str(worker)])
    server.setup(args)
    active = False
    try:
        server.initialize(state, [str(initializer), "canonical-init", str(state)])
        server.start(state, server.config_for(state))
        active = True
        candidate = server.restart_log_configuration(state, "mcp-audit", time.monotonic()+120)
        mcp_call(state)
        until = time.monotonic()+5
        while True:
            audit = server.mcp_audit_records(state, until)
            if audit["records"]:
                break
            time.sleep(min(0.02, server.remaining(until)))
        assert all(record["tool"] == "query" and record["outcome"] == "ok" and record["time_ms"] is not None for record in audit["records"])
        assert "query-payload-must-not-appear" not in json.dumps(audit)
        restored = server.restart_log_configuration(state, "baseline", time.monotonic()+120)
        mcp_call(state)
        after = server.mcp_audit_records(state, time.monotonic()+5)
        assert len(after["records"]) == len(audit["records"])
        capture=evidence/"trace"
        code=subprocess.call([sys.executable,"-m","scripts.surreal_diagnostics","trace-capture","--state",str(state),"--output",str(capture),"--",sys.executable,"-m","scripts.tests.surreal_logging_check","--state",str(state),"--probe-only"],cwd=ROOT)
        assert code==0
        traces=server.read_json(capture/"capture.json")
        assert traces["exported_bytes"]>0 and traces["workload"]==0
        assert any(span["name"]=="invoke_with_timeout" for span in traces["spans"])
        assert server.config_for(state).get("diagnostic_telemetry") is None
        assert server.log_configuration(server.config_for(state))==server.LOG_CONFIGURATIONS["baseline"]
        server.write_json(evidence / "qualified.json", {"audit":audit, "candidate":candidate, "restored":restored, "baseline_records":len(after["records"]), "scope":"disposable exact supervisor restart and scoped MCP metadata only"})
    finally:
        if active:
            server.close_context_admission(state)
            server.stop(state, server.config_for(state))
        server.write_json(evidence / "cleanup.json", {"active": server.active(state), "admission": server.config_for(state)["admission"]})


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--evidence", type=Path)
    parser.add_argument("--initializer", type=Path)
    parser.add_argument("--worker", type=Path)
    parser.add_argument("--probe-only", action="store_true")
    args = parser.parse_args()
    if args.probe_only:
        mcp_call(args.state)
    else:
        if args.evidence is None or args.initializer is None or args.worker is None:
            parser.error("explicit evidence, captured canonical initializer and immutable worker are required")
        run(args.state, args.evidence, args.initializer, args.worker)


if __name__ == "__main__":
    main()
