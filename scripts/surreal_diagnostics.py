# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected observations and explicit owned database diagnostic restarts."""

from __future__ import annotations

import argparse
import json
import os
import selectors
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import msgspec

from scripts import storage_health
from scripts import surreal_server as server
from scripts.surreal_telemetry import capture_operation


def syntax_capture(
    command: list[str], sql: str, environment: dict[str, str], deadline: float
) -> dict[str, object]:
    """Bound each native syntax check's aggregate output while draining its exact child."""
    with tempfile.TemporaryFile() as source, selectors.DefaultSelector() as streams:
        source.write(sql.encode())
        source.seek(0)
        server.remaining(deadline)
        child = subprocess.Popen(
            command,
            stdin=source,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=environment,
        )
        output = {"stdout": bytearray(), "stderr": bytearray()}
        count = 0
        try:
            for name, stream in (("stdout", child.stdout), ("stderr", child.stderr)):
                if stream is None:
                    raise server.SupervisorError(
                        "Syntax checker output stream unavailable"
                    )
                streams.register(stream, selectors.EVENT_READ, name)
            while streams.get_map():
                for key, _ in streams.select(server.remaining(deadline)):
                    block = os.read(key.fd, 64 << 10)
                    server.remaining(deadline)
                    if not block:
                        streams.unregister(key.fileobj)
                        continue
                    count += len(block)
                    if count > 1 << 20:
                        raise server.SupervisorError(
                            "Syntax checker output exceeds its bound"
                        )
                    output[key.data].extend(block)
            code = child.wait(timeout=server.remaining(deadline))
            return {
                "exit_code": code,
                **{name: raw.decode(errors="replace") for name, raw in output.items()},
            }
        finally:
            if child.poll() is None:
                child.kill()
            child.wait(timeout=5)
            for stream in (child.stdout, child.stderr):
                if stream is not None:
                    stream.close()


def exact_parser(
    state: Path, description: dict[str, object], deadline: float
) -> dict[str, object]:
    """Use the selected server binary as syntax authority, with original source context."""
    config = server.config_for(state, deadline=deadline)
    selected = server.object_mapping(config["server"])
    binary = Path(str(selected["binary"]))
    if server.file_digest(binary) != selected["binary_sha256"]:
        raise server.SupervisorError(
            "Selected syntax oracle binary changed; readmit explicitly"
        )
    query = description.get("query", description)
    query = server.object_mapping(query)
    sql = query.get("sql")
    if not isinstance(sql, str) or len(sql.encode()) > 1 << 20:
        raise server.SupervisorError("Owned query description lacks bounded SQL")
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("SURREAL_", "DATABASE_"))
    }
    result = syntax_capture(
        [str(binary), "validate", "--stdin"], sql, environment, deadline
    )
    status = "valid" if result["exit_code"] == 0 else "invalid"
    if "Use of `mod::` requires Surrealism modules" in str(result["stdout"]):
        status = "unsupported_capability"
    return {
        "schema": "selected-query-feedback-v1",
        "binary_sha256": selected["binary_sha256"],
        "source": query.get("source"),
        "parameters": query.get("parameters"),
        "exact_parser": {
            "status": status,
            **result,
            "capability_scope": "validate --stdin uses default parser capabilities; Surrealism mod:: is unavailable here even when enabled on the selected server",
        },
        "advisory": {
            "status": "unavailable",
            "decision": "no-adopt",
            "version": "0.6.3",
            "revision": "5aa6a12a29b7def0e7df662c09ac9203e66f5033",
            "reason": "The qualified canonical producer corpus is incompatible: 0.6.3 rejects namespaced fn/RETURN blocks and generated INLINE EDGES/REFERENCES schema; workspace-only checking can report zero despite those parse losses. Semantic assistance is unavailable for this producer; revisit after grammar support and context coverage are demonstrated",
        },
    }


def parser() -> argparse.ArgumentParser:
    command = argparse.ArgumentParser(description=__doc__)
    commands = command.add_subparsers(dest="command", required=True)
    health = commands.add_parser(
        "health",
        help="Inspect selected storage/host health without repair, or read a supplied snapshot",
    )
    selection = health.add_mutually_exclusive_group(required=True)
    selection.add_argument("--state", type=Path)
    selection.add_argument("--snapshot", type=Path)
    health.add_argument(
        "--output", type=Path, help="Retain original versioned health snapshot"
    )
    metrics = commands.add_parser(
        "metrics",
        help="Observe fixed selected /metrics with internal VIEWER authentication",
    )
    metrics.add_argument("--state", type=Path, required=True)
    metrics.add_argument("--output", type=Path)
    compare = commands.add_parser(
        "compare-metrics", help="Compare two retained like-service snapshots"
    )
    compare.add_argument("before", type=Path)
    compare.add_argument("after", type=Path)
    audit = commands.add_parser(
        "mcp-audit",
        help="Read selected safe audit metadata from its current bounded private log",
    )
    audit.add_argument("--state", type=Path, required=True)
    logs = commands.add_parser(
        "log-config", help="Inspect or explicitly restart selected diagnostic logging"
    )
    logs.add_argument("--state", type=Path, required=True)
    logs.add_argument("--restart", choices=("mcp-audit", "baseline"))
    traces = commands.add_parser(
        "trace-capture",
        help="Restart temporary loopback telemetry, run one literal command, restore and drain",
    )
    traces.add_argument("--state", type=Path, required=True)
    traces.add_argument("--output", type=Path, required=True)
    traces.add_argument("operation", nargs=argparse.REMAINDER)
    check = commands.add_parser(
        "check-query",
        help="Validate a production query description using the selected exact engine",
    )
    check.add_argument("--state", type=Path, required=True)
    check.add_argument("--description", type=Path, required=True)
    return command


def main(arguments: list[str] | None = None) -> int:
    args = parser().parse_args(arguments)
    try:
        deadline = time.monotonic() + 15
        if args.command == "health":
            snapshot = (
                storage_health.read(args.snapshot)
                if args.snapshot is not None
                else storage_health.capture(args.state, deadline)
            )
            if args.output is not None:
                server.write_json(args.output, msgspec.to_builtins(snapshot))
            print(
                json.dumps(
                    storage_health.presentation(snapshot),
                    sort_keys=True,
                    allow_nan=False,
                )
            )
            return 0
        if args.command == "trace-capture":
            operation = (
                args.operation[1:] if args.operation[:1] == ["--"] else args.operation
            )
            result = capture_operation(args.state, args.output, operation)
            print(
                json.dumps(
                    {
                        "artifact": str(args.output / "capture.json"),
                        "exported_bytes": result["exported_bytes"],
                        "spans": len(server.object_inventory(result["spans"])),
                        "workload": result["workload"],
                    }
                )
            )
            return server.integer(result["workload"])
        if args.command == "metrics":
            result = server.metric_snapshot(args.state, deadline)
            if args.output:
                server.write_json(args.output, result)
                print(
                    json.dumps(
                        {
                            "artifact": str(args.output),
                            "identity": result["identity"],
                            "families": len(server.object_mapping(result["families"])),
                        }
                    )
                )
                return 0
        elif args.command == "compare-metrics":
            result = server.compare_metric_snapshots(
                server.read_json(args.before), server.read_json(args.after)
            )
        elif args.command == "mcp-audit":
            result = server.mcp_audit_records(args.state, deadline)
        elif args.command == "log-config":
            result = (
                server.restart_log_configuration(
                    args.state, args.restart, time.monotonic() + 120
                )
                if args.restart
                else server.inspect_log_configuration(args.state, deadline)
            )
        else:
            result = exact_parser(
                args.state, server.read_json(args.description), deadline
            )
        print(json.dumps(result, sort_keys=True))
        return (
            1
            if (
                "exact_parser" in result
                and server.object_mapping(result["exact_parser"])["status"] != "valid"
            )
            else 0
        )
    except (
        server.SupervisorError,
        OSError,
        ValueError,
        KeyError,
        subprocess.SubprocessError,
    ):
        print(
            "surreal-diagnostics: selected observation unavailable; verify owned service, private inputs and exact binary",
            file=sys.stderr,
        )
        return 125


if __name__ == "__main__":
    raise SystemExit(main())
