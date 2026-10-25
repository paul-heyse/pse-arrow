# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit real-STDIO response-loss and bounded action acceptance.

The relay drops only the completed acknowledgement for one selected action. It
never retries the action, changes its implementation, or invents settlement.
"""

from __future__ import annotations

import argparse
import asyncio
import contextlib
import hashlib
import json
import os
import subprocess
import sys
import threading
import time
from pathlib import Path

from fastmcp import Client
from fastmcp.client.transports import StdioTransport

ROOT = Path(__file__).resolve().parents[2]
TEST = "test(=float::tests::signed_zero_is_preserved_and_distinguished)"


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")


def require(condition: object, reason: str) -> None:
    if not condition:
        raise AssertionError(reason)


def object_value(value: object) -> dict:
    if not isinstance(value, dict):
        raise TypeError("expected object")
    return value


def unwrap(value: object) -> dict:
    selected = object_value(value)
    if set(selected) == {"result"}:
        selected = object_value(selected["result"])
    return object_value(selected.get("payload", selected))


def retained_lost_ack(output: Path) -> dict:
    lost = json.loads((output / "dropped-response.json").read_text())
    original = json.loads((output / "original-terminal.json").read_text())
    product = json.loads((output / "native-result.json").read_text())
    reference = json.loads((output / "original-reference.json").read_text())
    watched = json.loads((output / "detached-watch.json").read_text())
    require(lost["relay"]["calls"] == 1, "action replay detected")
    require(
        original["status"] == "passed" and original["exit_code"] == 0,
        "original producer did not pass",
    )
    require(
        len(original["results"]) == 1 and original["results"][0]["status"] == "passed",
        "original one-test result differs",
    )
    require(
        product["assurance"] == "pass" and product["selected_count"] == 1,
        "managed reader did not preserve original assurance",
    )
    require(
        product["selected_unreported_count"] == 0 and not product["report_errors"],
        "managed original population incomplete",
    )
    require(
        watched["selection"]["kind"] == "run", "detached watch selected another kind"
    )
    require(
        watched["selection"]["handle"] == Path(reference["invocation"]).stem,
        "detached watch selected another invocation",
    )
    require(
        watched["selection"]["registry"] == reference["registry"],
        "detached watch selected another registry",
    )
    snapshot = watched["snapshot"]
    require(
        snapshot["result_reference"]["run_path"] == reference["invocation"],
        "detached result reference differs from original",
    )
    require(
        snapshot["result_reference"]["registry"] == reference["registry"],
        "detached result reference registry differs",
    )
    require(
        snapshot["run"]["assurance"] == "pass"
        and snapshot["run"]["selected_count"] == 1,
        "detached snapshot changed original assurance/population",
    )
    require(
        not Path(f"/proc/{lost['relay']['endpoint_pid']}").exists(),
        "old endpoint process remains",
    )
    return {
        "id": "lost-ack",
        "status": "passed",
        "action_calls": 1,
        "native_tests": 1,
        "native_passed": 1,
        "caller_timeout": True,
        "old_endpoint_reaped": True,
        "detached_snapshot_bound": True,
        "original_owned_drain": product["owned_drain"],
        "capabilities": [
            "rust_test_run",
            "rust_test_reference_read",
            "product_run_read",
            "workload_watch",
        ],
    }


def retained_assessment(output: Path) -> dict:
    compilation = json.loads((output / "compilation.json").read_text())["result"]
    assessment = json.loads((output / "assessment.json").read_text())["result"]
    read = json.loads((output / "assessment-read.json").read_text())
    for result in (compilation, assessment):
        require(
            result["complete"] and result["required_checks_covered"],
            "original action coverage incomplete",
        )
        require(
            result["exit_code"] == 0 and not result["provenance_errors"],
            "original action failed",
        )
        require(
            all(check["status"] == "passed" for check in result["checks"]),
            "original check did not pass",
        )
    expected = {"fmt-rust-check", "lint-toml"}
    require(
        read["run_path"] == str(output / "assessment"),
        "assessment reader selected another output",
    )
    require(
        read["assessment"]["complete"] is True
        and read["assessment"]["required_checks_covered"] is True,
        "assessment reader changed completeness/coverage",
    )
    require(
        read["assessment"]["input_coverage"] is False,
        "assessment reader promoted narrow input coverage",
    )
    require(
        len(read["checks"]) == 2
        and {row["record"]["gate"] for row in read["checks"]} == expected,
        "assessment reader returned another gate population",
    )
    require(
        all(
            row["qualified"] is True
            and row["record"]["status"] == "passed"
            and row["record"]["exit_code"] == 0
            for row in read["checks"]
        ),
        "assessment reader changed gate qualification/outcome",
    )
    return {
        "id": "bounded-actions",
        "status": "passed",
        "assessment_reader_bound": True,
        "expected_gates": sorted(expected),
        "input_coverage": False,
        "capabilities": ["compilation_check", "assessment_run", "assessment_read"],
        "scope": "selected force-validated library compile plus two original fmt-check gates; not scientific or full-product qualification",
    }


def retained_diagnostic(output: Path) -> dict:
    action_path = output / "gdb-owned/action.json"
    raw = action_path.read_bytes()
    record = json.loads(raw)
    read = json.loads((output / "diagnostic-read.json").read_text())["result"]
    require(
        read["action_reference"]["path"] == str(action_path),
        "diagnostic reader selected another action",
    )
    require(
        read["action_reference"]["sha256"] == hashlib.sha256(raw).hexdigest(),
        "diagnostic reader action digest differs from original bytes",
    )
    for observed in (record, read):
        require(observed["status"] == "collected", "GDB capture/read did not collect")
        require(
            observed["target_outcome"] == [{"exit_code": 0}],
            "owned target outcome differs",
        )
        require(
            observed["steps"][0]["process"]["owned_drain"] == "complete",
            "diagnostic helper drain incomplete",
        )
    return {
        "id": "owned-diagnostic",
        "status": "passed",
        "reader_original_sha256_checked": True,
        "capabilities": ["diagnostic_action", "diagnostic_action_read"],
        "target_exit": 0,
        "owned_drain": "complete",
    }


def relay(output: Path) -> int:
    child = subprocess.Popen(
        [str(ROOT / ".venv/bin/python"), "-m", "scripts.capabilities"],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr,
    )
    child_input, child_output = child.stdin, child.stdout
    if child_input is None or child_output is None:
        raise RuntimeError("missing child pipes")
    state: dict = {
        "calls": 0,
        "action_id": None,
        "dropped": False,
        "endpoint_pid": child.pid,
    }
    write(output / "relay-start.json", state)

    def incoming() -> None:
        for line in sys.stdin.buffer:
            message = json.loads(line)
            if message.get("method") == "tools/call":
                params = message.get("params", {})
                if (
                    params.get("name") == "capabilities_invoke_action"
                    and params.get("arguments", {}).get("capability") == "rust_test_run"
                ):
                    state["calls"] += 1
                    state["action_id"] = message["id"]
                    write(output / "relay-action-count.json", state)
            child_input.write(line)
            child_input.flush()
        child_input.close()

    thread = threading.Thread(target=incoming, daemon=True)
    thread.start()
    try:
        for line in child_output:
            message = json.loads(line)
            if (
                state["action_id"] is not None
                and message.get("id") == state["action_id"]
                and not state["dropped"]
            ):
                state["dropped"] = True
                write(
                    output / "dropped-response.json",
                    {"relay": state, "response": message},
                )
                continue
            sys.stdout.buffer.write(line)
            sys.stdout.buffer.flush()
        return child.wait(timeout=20)
    finally:
        if child.poll() is None:
            child.terminate()
            child.wait(timeout=20)
        write(output / "relay-exit.json", {**state, "endpoint_exit": child.returncode})


class Acceptance:
    def __init__(self, output: Path) -> None:
        self.output = output.resolve()
        self.output.mkdir(parents=True, exist_ok=False)
        self.calls: list[dict] = []
        self.results: list[dict] = []

    async def invoke(
        self, client: Client, capability: str, arguments: dict, *, action: bool = False
    ) -> dict:
        described = await client.call_tool(
            "capabilities_describe", {"capability": capability, "include_schema": True}
        )
        require(not described.is_error, "description refused")
        await client.call_tool("capabilities_select", {"capabilities": [capability]})
        response = await client.call_tool(
            "capabilities_invoke_action" if action else "capabilities_invoke_read",
            {"capability": capability, "arguments": arguments},
            raise_on_error=False,
        )
        row = {
            "capability": capability,
            "arguments": arguments,
            "action": action,
            "is_error": response.is_error,
            "structured": response.structured_content,
            "content": [block.model_dump(mode="json") for block in response.content],
        }
        self.calls.append(row)
        write(self.output / "calls.json", self.calls)
        require(not response.is_error, "operation refused: " + capability)
        return unwrap(response.structured_content)

    def transport(self) -> StdioTransport:
        return StdioTransport(
            command=str(ROOT / ".venv/bin/python"),
            args=["-m", "scripts.capabilities"],
            cwd=str(ROOT),
            env=dict(os.environ),
            keep_alive=False,
        )

    async def lost_ack(self) -> None:
        out = self.output / "lost-ack"
        out.mkdir()
        rust_output = out / "rust-test"
        transport = StdioTransport(
            command=str(ROOT / ".venv/bin/python"),
            args=[
                "-m",
                "scripts.tests.mcp_interruption_acceptance",
                "--relay",
                "--output",
                str(out),
            ],
            cwd=str(ROOT),
            env=dict(os.environ),
            keep_alive=False,
        )
        request = {
            "request": {
                "package": "pse-ids",
                "kind": "unit",
                "expression": TEST,
                "output": str(rust_output),
            }
        }
        write(out / "authorized-request.json", request)
        async with Client(transport, timeout=180) as client:
            await client.call_tool(
                "capabilities_describe",
                {"capability": "rust_test_run", "include_schema": True},
            )
            await client.call_tool(
                "capabilities_select", {"capabilities": ["rust_test_run"]}
            )
            task = asyncio.create_task(
                client.call_tool(
                    "capabilities_invoke_action",
                    {"capability": "rust_test_run", "arguments": request},
                )
            )
            deadline = time.monotonic() + 150
            while not (out / "dropped-response.json").is_file():
                require(
                    time.monotonic() < deadline,
                    "original action never returned to response-loss relay",
                )
                require(
                    not task.done(),
                    "client received action before controlled response loss",
                )
                await asyncio.sleep(0.05)
            lost = json.loads((out / "dropped-response.json").read_text())
            require(lost["relay"]["calls"] == 1, "action replay detected")
            require(not task.done(), "dropped response reached client")
            timed_out = False
            try:
                await asyncio.wait_for(task, timeout=0.2)
            except TimeoutError:
                timed_out = True
            require(timed_out, "lost acknowledgement did not leave caller uncertain")
        reference = json.loads((rust_output / "invocation-reference.json").read_text())
        write(out / "original-reference.json", reference)
        async with Client(self.transport(), timeout=120) as fresh:
            issued = await self.invoke(
                fresh, "rust_test_reference_read", {"output": str(rust_output)}
            )
            require(issued["outcome"] == "issued", "fresh original reference missing")
            require(
                issued["reference"] == reference,
                "fresh client returned another invocation",
            )
            arguments = {
                "invocation": reference["invocation"],
                "registry": reference["registry"],
                "limit": 10,
                "tail": 0,
            }
            product = await self.invoke(fresh, "product_run_read", arguments)
            observed = await self.invoke(
                fresh,
                "workload_watch",
                {
                    "kind": "run",
                    "handle": reference["invocation"],
                    "registry": reference["registry"],
                    "max_wait": 0,
                },
            )
            await fresh.call_tool("capabilities_reset", {})
        write(out / "native-result.json", product)
        write(out / "detached-watch.json", observed)
        write(
            out / "host-ready.json",
            {
                "reader": "product_run_read",
                "arguments": arguments,
                "action_count": 1,
                "requested_test": TEST,
            },
        )
        terminal = Path(reference["invocation"]).with_suffix(".terminal.json")
        require(terminal.is_file(), "original terminal missing")
        original = json.loads(terminal.read_text())
        write(out / "original-terminal.json", original)
        pid = lost["relay"]["endpoint_pid"]
        deadline = time.monotonic() + 10
        while Path(f"/proc/{pid}").exists() and time.monotonic() < deadline:
            await asyncio.sleep(0.05)
        require(not Path(f"/proc/{pid}").exists(), "old endpoint process remains")
        self.results.append(retained_lost_ack(out))

    async def bounded_actions(self) -> None:
        async with Client(self.transport(), timeout=600) as client:
            compilation = await self.invoke(
                client,
                "compilation_check",
                {
                    "request": {
                        "package": "pse-ids",
                        "target": "library",
                        "output": str(self.output / "compilation"),
                    }
                },
                action=True,
            )
            write(self.output / "compilation.json", compilation)
            assessment = await self.invoke(
                client,
                "assessment_run",
                {
                    "request": {
                        "group": "fmt-check",
                        "output": str(self.output / "assessment"),
                    }
                },
                action=True,
            )
            write(self.output / "assessment.json", assessment)
            read = await self.invoke(
                client,
                "assessment_read",
                {"directory": str(self.output / "assessment"), "tail": 0},
            )
            write(self.output / "assessment-read.json", read)
            await client.call_tool("capabilities_reset", {})
        self.results.append(retained_assessment(self.output))

    async def diagnostic(self) -> None:
        output = self.output / "gdb-owned"
        async with Client(self.transport(), timeout=120) as client:
            captured = await self.invoke(
                client,
                "diagnostic_action",
                {
                    "request": {
                        "action": "gdb-launch",
                        "output": str(output),
                        "seconds": 15.0,
                        "command": [
                            str(ROOT / ".venv/bin/python"),
                            "-c",
                            "print('mcp-owned-diagnostic-control')",
                        ],
                    }
                },
                action=True,
            )
            write(self.output / "diagnostic.json", captured)
            read = await self.invoke(
                client,
                "diagnostic_action_read",
                {"request": {"output": str(output), "tail": 0, "summary_only": True}},
            )
            write(self.output / "diagnostic-read.json", read)
            await client.call_tool("capabilities_reset", {})
        self.results.append(retained_diagnostic(self.output))

    async def cancellation(self) -> None:
        """A synthetic profiler stalls a real admitted helper; no science is simulated."""
        ready = self.output / "profiler-ready"
        profiler = self.output / "selected-perf"
        profiler.write_text(
            f"#!{sys.executable}\nimport os,sys,time\nfrom pathlib import Path\nif '--version' in sys.argv: print('selected perf control'); sys.exit(0)\nPath({str(ready)!r}).write_text(str(os.getpid()))\ntime.sleep(60)\n"
        )
        profiler.chmod(0o700)
        subject = subprocess.Popen([sys.executable, "-c", "import time;time.sleep(60)"])
        request = {
            "request": {
                "action": "stat",
                "output": str(self.output / "cancelled"),
                "seconds": 60.0,
                "perf_binary": str(profiler),
                "command": [sys.executable, "-c", "pass"],
            }
        }
        write(self.output / "authorized-request.json", request)
        try:
            async with Client(self.transport(), timeout=120) as client:
                task = asyncio.create_task(
                    self.invoke(client, "diagnostic_action", request, action=True)
                )
                deadline = time.monotonic() + 20
                while not ready.is_file():
                    require(
                        not task.done(),
                        "diagnostic helper ended before cancellation control",
                    )
                    require(
                        time.monotonic() < deadline, "diagnostic helper never started"
                    )
                    await asyncio.sleep(0.05)
                helper_pid = int(ready.read_text())
                task.cancel()
                with contextlib.suppress(asyncio.CancelledError):
                    await task
                deadline = time.monotonic() + 20
                record_path = self.output / "cancelled/action.json"
                while not record_path.is_file() or Path(f"/proc/{helper_pid}").exists():
                    require(
                        time.monotonic() < deadline,
                        "original cancelled helper did not settle",
                    )
                    await asyncio.sleep(0.05)
                require(
                    subject.poll() is None, "cancelled helper stopped unrelated subject"
                )
            async with Client(self.transport(), timeout=120) as fresh:
                read = await self.invoke(
                    fresh,
                    "diagnostic_action_read",
                    {
                        "request": {
                            "output": str(self.output / "cancelled"),
                            "tail": 0,
                            "summary_only": True,
                        }
                    },
                )
                write(self.output / "cancelled-read.json", read)
                await fresh.call_tool("capabilities_reset", {})
            record = json.loads(record_path.read_text())
            require(
                record["status"] == "interrupted",
                "original action did not retain interruption",
            )
            require(
                record["steps"][0]["process"]["owned_drain"] == "complete",
                "cancelled helper original drain incomplete",
            )
            self.results.append(
                {
                    "id": "cancelled-diagnostic",
                    "status": "passed",
                    "capabilities": ["diagnostic_action", "diagnostic_action_read"],
                    "helper_reaped": True,
                    "unrelated_subject_survived": True,
                    "fixture": "synthetic selected profiler; real helper admission/cancellation/drain",
                }
            )
        finally:
            if subject.poll() is None:
                subject.terminate()
            subject.wait(timeout=10)

    async def run(self) -> int:
        for name, operation in (
            ("lost-ack", self.lost_ack),
            ("bounded-actions", self.bounded_actions),
            ("owned-diagnostic", self.diagnostic),
        ):
            try:
                await operation()
            except Exception as error:
                self.results.append(
                    {
                        "id": name,
                        "status": "failed",
                        "error": type(error).__name__,
                        "reason": str(error),
                    }
                )
            write(
                self.output / "lane.json",
                {
                    "baseline_failures": 0,
                    "cases": self.results,
                    "status": "failed"
                    if any(row["status"] != "passed" for row in self.results)
                    else "passed",
                },
            )
        print(json.dumps(self.results))
        return int(any(row["status"] != "passed" for row in self.results))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--relay", action="store_true")
    parser.add_argument("--reassess-lost-ack", action="store_true")
    parser.add_argument("--reassess-existing", action="store_true")
    parser.add_argument("--diagnostic-only", action="store_true")
    parser.add_argument("--cancellation-only", action="store_true")
    args = parser.parse_args()
    if args.relay:
        return relay(args.output)
    if args.reassess_existing:
        output = args.output.resolve()
        cases = [
            retained_lost_ack(output / "run-1/lost-ack"),
            retained_assessment(output / "run-1"),
            retained_diagnostic(output / "diagnostic-1"),
        ]
        result = {
            "status": "passed",
            "cases": cases,
            "scope": "additional assertions over retained reader responses and original artifacts only; no action or backend query",
        }
        write(output / "reader-reassessment.json", result)
        lane = json.loads((output / "lane.json").read_text())
        updated = {row["id"]: row for row in cases}
        lane["cases"] = [{**row, **updated.get(row["id"], {})} for row in lane["cases"]]
        lane["reader_reassessment"] = str(output / "reader-reassessment.json")
        write(output / "lane.json", lane)
        print(json.dumps(result))
        return 0
    if args.reassess_lost_ack:
        result = retained_lost_ack(args.output)
        write(args.output / "assessment-corrected.json", result)
        print(json.dumps(result))
        return 0
    if args.diagnostic_only or args.cancellation_only:
        acceptance = Acceptance(args.output)
        try:
            asyncio.run(
                acceptance.diagnostic()
                if args.diagnostic_only
                else acceptance.cancellation()
            )
        except Exception as error:
            acceptance.results.append(
                {
                    "status": "failed",
                    "error": type(error).__name__,
                    "reason": str(error),
                }
            )
        status = (
            "failed"
            if any(row["status"] != "passed" for row in acceptance.results)
            else "passed"
        )
        write(
            acceptance.output / "lane.json",
            {"status": status, "cases": acceptance.results},
        )
        print(json.dumps(acceptance.results))
        return int(status != "passed")
    return asyncio.run(Acceptance(args.output).run())


if __name__ == "__main__":
    raise SystemExit(main())
