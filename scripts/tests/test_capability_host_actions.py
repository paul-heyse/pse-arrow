# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed host kernels, real FunctionTools and independent helper cancellation."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import asyncio
import contextlib
import io
import json
import os
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from fastmcp import Client, FastMCP
from fastmcp.exceptions import ToolError
from fastmcp.tools import FunctionTool
from pydantic import ValidationError

from scripts import active_diagnostics as active
from scripts import experiments, test_archives
from scripts.capabilities import host_actions, host_worker
from scripts.tests.test_capabilities import content

ROOT = Path(__file__).resolve().parents[2]


class HostKernelTests(unittest.TestCase):
    def test_requests_validate_defaults_unknown_fields_and_explicit_empty(self) -> None:
        request = active.ActionRequest(action="stat", output="selected", events=None)
        self.assertEqual(request.seconds, 10.0)
        self.assertIn("events", request.model_fields_set)
        self.assertNotIn(
            "events",
            active.ActionRequest(action="stat", output="selected").model_fields_set,
        )
        self.assertEqual(request.model_dump(exclude_unset=True)["events"], None)
        for fields in (
            {"unknown": "silent"},
            {"output_bytes": True},
            {"seconds": "1"},
            {"seconds": float("nan")},
            {"events": ""},
            {"pid": 1},
            {"command": [""]},
        ):
            with self.subTest(fields=fields), self.assertRaises(ValidationError):
                active.ActionRequest.model_validate(
                    {"action": "stat", "output": "selected", **fields}
                )
        for fields in (
            {"lib": True, "test": "target"},
            {"fixture": ["../escape"]},
            {"filter": ""},
        ):
            with self.subTest(fields=fields), self.assertRaises(ValidationError):
                test_archives.ArchiveRequest.model_validate(
                    {
                        "output": "build/archive",
                        "package": "pse-ids",
                        "filter": "test(control)",
                        **fields,
                    }
                )

    def test_experiment_leaf_help_is_not_consumed_by_outer_dispatch(self) -> None:
        for operation, flag in (
            ("archive", "--package"),
            ("inspect", "archive"),
            ("replay", "--output"),
        ):
            captured = io.StringIO()
            with (
                self.subTest(operation=operation),
                contextlib.redirect_stdout(captured),
                self.assertRaises(SystemExit) as exited,
            ):
                experiments.main([operation, "--help"])
            self.assertEqual(exited.exception.code, 0)
            self.assertIn(flag, captured.getvalue())
            self.assertNotIn("tool-probe", captured.getvalue())

    def test_cli_calls_the_typed_archive_kernel(self) -> None:
        captured = io.StringIO()
        with (
            patch.object(
                test_archives, "archive", return_value={"status": "archived"}
            ) as kernel,
            contextlib.redirect_stdout(captured),
        ):
            self.assertEqual(
                experiments.main(
                    [
                        "archive",
                        "--output",
                        "build/control",
                        "--package",
                        "pse-ids",
                        "--filter",
                        "test(control)",
                    ]
                ),
                0,
            )
        request = kernel.call_args.args[0]
        self.assertIsInstance(request, test_archives.ArchiveRequest)
        self.assertEqual(request.package, "pse-ids")

    def test_interrupted_capture_publishes_and_never_starts_decoder(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "action"

            def capture(
                _command: list[str], destination: Path, _label: str, **_options: object
            ) -> dict:
                (destination / "perf.data").write_bytes(b"partial capture")
                return {
                    "status": "returned",
                    "exit_code": 143,
                    "process": {
                        "forwarded_signal": signal.SIGTERM,
                        "owned_drain": "complete",
                    },
                }

            with (
                patch.object(active, "collect", side_effect=capture) as collect,
                patch.object(
                    active,
                    "selected_tool",
                    return_value={"status": "available", "path": "/selected/perf"},
                ),
            ):
                result = active.execute(
                    active.ActionRequest(
                        action="python-perf",
                        output=str(output),
                        command=[sys.executable, "-c", "pass"],
                    )
                )
            self.assertEqual(collect.call_count, 1)
            self.assertEqual(result["status"], "interrupted")
            self.assertEqual(result["steps"][0]["process"]["owned_drain"], "complete")
            self.assertEqual(
                active.read_action(output, summary_only=True)["status"], "interrupted"
            )

    def test_fixed_worker_refuses_unregistered_operation(self) -> None:
        with self.assertRaisesRegex(ValueError, "unsupported fixed host operation"):
            host_worker.invoke("not-a-shell-router", "{}")

    def test_interrupted_replay_retains_new_invocation_native_drain_and_historical_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = root / "retained"
            archive.mkdir()
            output = root / "replayed"
            invocation = root / "new-invocation.json"
            selected = [
                {
                    "binary_id": "pure::lib",
                    "name": "control",
                    "executable": "/historical/test",
                }
            ]
            manifest = {
                "fixtures": {},
                "filter": "test(control)",
                "selected": selected,
                "libraries": {},
                "producer_invocation": "/original/invocation.json",
                "source_commit": "original-source",
            }
            (archive / "archive.json").write_text(json.dumps(manifest))
            (archive / "checks.json").write_text("historical receipt")
            terminal = {
                "status": "interrupted",
                "exit_code": 143,
                "process": {
                    "forwarded_signal": signal.SIGTERM,
                    "owned_drain": "complete",
                },
            }

            def allocate(path: Path) -> str:
                path.mkdir()
                return "authentic-resource"

            def execute(*_args: object, **_options: object) -> int:
                invocation.with_suffix(".terminal.json").write_text(
                    json.dumps(terminal)
                )
                return 143

            with (
                patch.object(test_archives, "new_output", side_effect=allocate),
                patch.object(
                    test_archives.arrow_validation,
                    "rust_test_environment",
                    return_value={},
                ),
                patch.object(
                    test_archives.test_resources,
                    "reference_report",
                    return_value=contextlib.nullcontext(),
                ),
                patch.object(test_archives, "preflight"),
                patch.object(test_archives, "capture_json", return_value={}),
                patch.object(
                    test_archives.test_run, "rust_selected", return_value=selected
                ),
                patch.object(test_archives, "library_closure", return_value={}),
                patch.object(
                    test_archives.test_resources, "invocation", return_value=invocation
                ),
                patch.object(
                    test_archives.test_run,
                    "rust_report_command",
                    side_effect=lambda _invocation, command: (command, root / "report"),
                ),
                patch.object(
                    test_archives.test_run, "execute_selected", side_effect=execute
                ),
                patch.object(test_archives, "seal") as seal,
            ):
                result = test_archives.replay(archive, output)
            self.assertEqual(
                result["producer_invocation"], manifest["producer_invocation"]
            )
            self.assertEqual(result["invocation"], str(invocation))
            self.assertEqual(result["source_commit"], "original-source")
            self.assertEqual(result["status"], "interrupted")
            self.assertEqual(result["exit_code"], 143)
            self.assertEqual(result["process"], terminal["process"])
            self.assertFalse(result["current_source_qualified"])
            self.assertEqual(json.loads(Path(result["terminal"]).read_text()), terminal)
            receipt = seal.call_args.args[2]
            self.assertFalse(receipt["complete"])
            self.assertFalse(receipt["required_checks_covered"])


class HostToolTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.specs, self.tools = host_actions.bindings(ROOT)

    async def test_actual_functiontools_enforce_strict_unknowns_before_effects(
        self,
    ) -> None:
        self.assertTrue(
            all(isinstance(tool, FunctionTool) for tool in self.tools.values())
        )
        server = FastMCP("host-controls", strict_input_validation=True)
        for tool in self.tools.values():
            server.add_tool(tool)
        with patch.object(host_actions, "run_worker", new_callable=AsyncMock) as worker:
            async with Client(server) as client:
                for request in (
                    {"action": "feasibility", "output": "chosen", "extra": 1},
                    {"action": "stat", "output": "chosen", "seconds": "1"},
                    {"action": "stat", "output": "chosen", "output_bytes": True},
                ):
                    with self.subTest(request=request), self.assertRaises(ToolError):
                        await client.call_tool(
                            "diagnostic_action", {"request": request}
                        )
        worker.assert_not_awaited()
        assert self.tools["diagnostic_action"].annotations is not None
        assert self.tools["diagnostic_action_read"].annotations is not None
        self.assertFalse(self.tools["diagnostic_action"].annotations.read_only_hint)
        self.assertTrue(self.tools["diagnostic_action_read"].annotations.read_only_hint)

    async def test_real_mcp_action_retains_native_result_and_reader(self) -> None:
        output = self.directory / "feasibility"
        server = FastMCP("host-controls", strict_input_validation=True)
        for tool in self.tools.values():
            server.add_tool(tool)
        async with Client(server) as client:
            result = await client.call_tool(
                "diagnostic_action",
                {"request": {"action": "feasibility", "output": str(output)}},
            )
            payload = content(result)["result"]
            self.assertEqual(payload["status"], "observed")
            self.assertEqual(
                payload["target_outcome"],
                "not observed; collection success is independent",
            )
            self.assertIsNone(payload["invocation"])
            read = await client.call_tool(
                "diagnostic_action_read", {"request": {"output": str(output)}}
            )
            self.assertEqual(
                content(read)["result"]["started_monotonic_ns"],
                payload["started_monotonic_ns"],
            )
            with self.assertRaises(ToolError) as refusal:
                await client.call_tool(
                    "diagnostic_action",
                    {"request": {"action": "feasibility", "output": str(output)}},
                )
            self.assertIn("FileExistsError", str(refusal.exception))

    async def test_disappeared_subject_is_native_unavailable_not_transport_success_claim(
        self,
    ) -> None:
        output = self.directory / "stale-subject"
        result = await self.tools["diagnostic_action"].run(
            {
                "request": {
                    "action": "python-snapshot",
                    "output": str(output),
                    "pid": os.getpid(),
                    "start": "0",
                }
            }
        )
        native = content(result)["result"]
        self.assertEqual(native["status"], "unavailable")
        self.assertIn("identity changed", native["error"])
        self.assertEqual(native["steps"], [])

    async def test_archive_reader_continuation_refuses_replaced_manifest(self) -> None:
        directory = self.directory / "archive"
        directory.mkdir()
        (directory / "tests.tar.zst").write_bytes(b"retained")
        manifest = {
            "schema": test_archives.SCHEMA,
            "archive_sha256": test_archives.digest(directory / "tests.tar.zst"),
            "selected": [{"name": f"control-{index}"} for index in range(3)],
            "fixtures": {},
            "source_commit": "historical",
            "native_replay": "unsupported",
            "producer_invocation": "/authentic/invocation.json",
            "claims": "historical only",
            "libraries": {},
        }
        path = directory / "archive.json"
        path.write_text(json.dumps(manifest))
        with patch.object(
            test_archives.test_resources,
            "reference_report",
            return_value=contextlib.nullcontext(),
        ):
            first = await self.tools["experiment_inspect"].run(
                {"request": {"archive": str(directory), "limit": 1}}
            )
            native = content(first)["result"]
            self.assertEqual(native["next_offset"], 1)
            second = await self.tools["experiment_inspect"].run(
                {
                    "request": {
                        "archive": str(directory),
                        "limit": 1,
                        "offset": 1,
                        "manifest_sha256": native["manifest_sha256"],
                    }
                }
            )
            self.assertEqual(
                content(second)["result"]["selected"], [{"name": "control-1"}]
            )
            path.write_text(json.dumps({**manifest, "source_commit": "replaced"}))
            with self.assertRaises(ToolError) as refusal:
                await self.tools["experiment_inspect"].run(
                    {
                        "request": {
                            "archive": str(directory),
                            "offset": 1,
                            "manifest_sha256": native["manifest_sha256"],
                        }
                    }
                )
            self.assertIn("manifest identity changed", str(refusal.exception))

    async def test_actual_cancellation_joins_helper_and_preserves_unrelated_subject(
        self,
    ) -> None:
        output = self.directory / "cancelled"
        ready = self.directory / "helper-ready"
        helper = self.directory / "selected-perf"
        helper.write_text(
            f"#!{sys.executable}\nimport os,sys,time\nfrom pathlib import Path\n"
            "if '--version' in sys.argv: print('selected perf control'); sys.exit(0)\n"
            f"Path({str(ready)!r}).write_text(str(os.getpid()))\n"
            "time.sleep(60)\n"
        )
        helper.chmod(0o700)
        target = subprocess.Popen([sys.executable, "-c", "import time;time.sleep(60)"])
        try:
            task = asyncio.create_task(
                self.tools["diagnostic_action"].run(
                    {
                        "request": {
                            "action": "stat",
                            "output": str(output),
                            "seconds": 60,
                            "perf_binary": str(helper),
                            "command": [sys.executable, "-c", "pass"],
                        }
                    }
                )
            )
            try:
                async with asyncio.timeout(15):
                    while not ready.exists():
                        if task.done():
                            self.fail(str(task.result()))
                        await asyncio.sleep(0.02)
                helper_pid = int(ready.read_text())
                task.cancel()
                with self.assertRaises(asyncio.CancelledError):
                    await asyncio.wait_for(task, timeout=20)
                self.assertIsNone(target.poll())
                with self.assertRaises(ProcessLookupError):
                    os.kill(helper_pid, 0)
                record = active.read_action(output, summary_only=True)
                self.assertEqual(record["status"], "interrupted", record)
                self.assertEqual(
                    record["steps"][0]["process"]["owned_drain"], "complete"
                )
            finally:
                if not task.done():
                    task.cancel()
                    with contextlib.suppress(asyncio.CancelledError):
                        await task
        finally:
            target.terminate()
            target.wait(timeout=5)


if __name__ == "__main__":
    unittest.main()
