# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite exact-parser capture and source-feedback controls."""
# ruff: noqa: PT009, PT027
from __future__ import annotations

import os
import contextlib
import json
import io
import tempfile
from pathlib import Path
import sys
import time
import unittest
from unittest.mock import MagicMock, patch

from scripts import surreal_diagnostics as diagnostics
from scripts import surreal_server as server


class ParserCaptureTests(unittest.TestCase):
    def test_capture_preserves_native_exit_channels_and_ranges(self) -> None:
        result = diagnostics.syntax_capture(
            [sys.executable, "-c", "import sys; print('stdin[1:3]'); print('hint', file=sys.stderr); sys.exit(1)"],
            "SELECT;", dict(os.environ), time.monotonic() + 5,
        )
        self.assertEqual(result, {"exit_code": 1, "stdout": "stdin[1:3]\n", "stderr": "hint\n"})

    def test_output_flood_is_refused_and_exact_child_drains(self) -> None:
        with self.assertRaisesRegex(server.SupervisorError, "output exceeds"):
            diagnostics.syntax_capture([sys.executable, "-c", "import os; os.write(1, b'x' * (2 << 20))"], "", dict(os.environ), time.monotonic() + 5)

    def test_timeout_drains_child_and_expired_clock_precedes_launch(self) -> None:
        with self.assertRaises(server.DeadlineExpiredError):
            diagnostics.syntax_capture([sys.executable, "-c", "import time; time.sleep(30)"], "", dict(os.environ), time.monotonic() + 0.1)
        with patch.object(diagnostics.subprocess, "Popen") as launch, self.assertRaises(server.DeadlineExpiredError):
            diagnostics.syntax_capture([sys.executable], "", dict(os.environ), time.monotonic() - 1)
        launch.assert_not_called()


class LogConfigurationTests(unittest.TestCase):
    def test_inspection_is_separate_and_filters_are_narrow(self) -> None:
        self.assertEqual(server.log_arguments({}), ["--log=warn", "--log-format=text"])
        self.assertEqual(server.log_arguments({"diagnostic_logging": "mcp-audit"}), ["--log=warn,surrealdb::mcp::audit=info", "--log-format=json"])
        with patch.object(server, "lifecycle_reservation") as reserve, self.assertRaises(server.SupervisorError):
            server.restart_log_configuration(Path("/unused"), "trace-everything", time.monotonic()+1)
        reserve.assert_not_called()

    def test_bounded_log_publishes_complete_records_and_redacts_split_secret(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "server.log"
            logger = server.BoundedLog(output, 4096, 1, "split-secret")
            class Chunks(io.BytesIO):
                def __init__(self) -> None:
                    super().__init__()
                    self.chunks = iter((b"prefix split-", b"secret suffix\n", b"last"))
                    self.calls = 0
                def read1(self, _size: int = -1) -> bytes:
                    self.calls += 1
                    if self.calls == 3:
                        self.assert_published()
                    return next(self.chunks, b"")
                def assert_published(self) -> None:
                    if output.read_bytes() != b"prefix [REDACTED] suffix\n":
                        raise AssertionError("complete audit record was buffered")
            logger.capture(Chunks())
            self.assertEqual(output.read_bytes(), b"prefix [REDACTED] suffix\nlast")

    def test_audit_reader_projects_metadata_and_omits_payload(self) -> None:
        selected = MagicMock()
        selected._config = {"log_max_bytes":4096}
        selected.state = Path("/fixture")
        selected.identity = {"invocation":"fixed"}
        raw = json.dumps({"target":"surrealdb::mcp::audit", "fields":{"tool":"query", "outcome":"ok", "time_ms":1, "query":"secret-query", "row":"secret-row"}}).encode()+b"\n"
        with patch.object(server, "selected_service", return_value=selected), patch.object(server, "private_record_bytes", return_value=raw):
            observed = server.mcp_audit_records(Path("/fixture"), time.monotonic()+5)
        self.assertNotIn("secret", json.dumps(observed))
        self.assertEqual(observed["records"][0]["tool"], "query")
        selected.recheck.assert_called_once()

    def test_owned_restart_changes_invocation_and_interruption_remains_pending(self) -> None:
        for failure in (False, True):
            with self.subTest(interrupted=failure), tempfile.TemporaryDirectory() as directory, contextlib.ExitStack() as controls:
                state = Path(directory)
                config = {"instance_id": "selected"}
                before = MagicMock(identity={"instance_id": "selected", "invocation": "before"})
                after = MagicMock(identity={"instance_id": "selected", "invocation": "after"})
                controls.enter_context(patch.object(server, "config_for", return_value=config))
                controls.enter_context(patch.object(server, "lifecycle_reservation", return_value=contextlib.nullcontext()))
                controls.enter_context(patch.object(server, "state_lock", return_value=contextlib.nullcontext()))
                controls.enter_context(patch.object(server, "selected_service", side_effect=[before, after]))
                controls.enter_context(patch.object(server, "storage_unit_observation", return_value={"InvocationID": "before"}))
                controls.enter_context(patch.object(server, "managed_contexts", return_value=[]))
                close = controls.enter_context(patch.object(server, "close_context_admission"))
                controls.enter_context(patch.object(server, "all_contexts_drained"))
                stop = controls.enter_context(patch.object(server, "stop"))
                controls.enter_context(patch.object(server, "readmit_supervisor"))
                controls.enter_context(patch.object(server, "start", side_effect=KeyboardInterrupt if failure else None))
                reconfigure = controls.enter_context(patch.object(server, "reconfigure"))
                if failure:
                    with self.assertRaises(KeyboardInterrupt):
                        server.restart_log_configuration(state, "mcp-audit", time.monotonic()+5)
                else:
                    result = server.restart_log_configuration(state, "baseline", time.monotonic()+5)
                    self.assertEqual(result["phase"], "restored")
                    self.assertNotEqual(result["before"]["invocation"], result["after"]["invocation"])
                stop.assert_called_once()
                reconfigure.assert_not_called()
                receipt = json.loads((state / "diagnostic-log-change.json").read_text())
                if failure:
                    self.assertEqual(receipt["phase"], "starting")
                    self.assertIn("interrupted", receipt["outcome"])
                    self.assertEqual(close.call_count, 2)


class TelemetryTests(unittest.TestCase):
    def test_capture_projects_span_parentage_and_refuses_compressed_frames(self) -> None:
        from scripts.surreal_telemetry import Capture
        from scripts.tests.surreal_mx_check import field
        span=field(1,b"t"*16)+field(2,b"s"*8)+field(4,b"p"*8)+field(5,b"invoke_with_timeout")+field(9,b"private-query")
        body=field(1,field(2,field(2,span)))
        with Capture() as capture:
            capture.observe(b"\0"+len(body).to_bytes(4,"big")+body)
            self.assertEqual(capture.spans,[{"trace_id":(b"t"*16).hex(),"span_id":(b"s"*8).hex(),"parent_span_id":(b"p"*8).hex(),"name":"invoke_with_timeout"}])
            self.assertNotIn("private-query",json.dumps(capture.spans))
            with self.assertRaisesRegex(ValueError,"uncompressed"):
                capture.observe(b"\1\0\0\0\0")
            capture.total=16<<20
            with self.assertRaisesRegex(ValueError,"finite capture"):
                capture.observe(b"x")

    def test_owned_capture_restores_after_success_and_workload_failure(self) -> None:
        from scripts import surreal_telemetry as telemetry
        for failure in (False,True):
            with self.subTest(failed=failure),tempfile.TemporaryDirectory() as directory,contextlib.ExitStack() as controls:
                selected=MagicMock(identity={"invocation":"selected"})
                controls.enter_context(patch.object(server,"selected_service",return_value=selected))
                controls.enter_context(patch.object(server,"config_for",side_effect=[{}, {"diagnostic_telemetry":{"endpoint":"http://127.0.0.1:1","capture_nonce":"a"*32}}]))
                controls.enter_context(patch.object(telemetry.uuid,"uuid4",return_value=MagicMock(hex="a"*32)))
                capture=telemetry.Capture()
                capture.server_address=("127.0.0.1",1)
                controls.enter_context(patch.object(telemetry,"Capture",return_value=capture))
                restart=controls.enter_context(patch.object(server,"restart_diagnostic_configuration",return_value={"phase":"complete"}))
                controls.enter_context(patch.object(telemetry.native_operation,"run_child_scope",side_effect=ValueError("workload failed") if failure else None,return_value=0))
                evidence=Path(directory)/"capture"
                if failure:
                    with self.assertRaisesRegex(ValueError,"workload failed"):
                        telemetry.capture_operation(Path("/fixture"),evidence,["literal","argument"])
                else:
                    self.assertEqual(telemetry.capture_operation(Path("/fixture"),evidence,["literal","argument"])["workload"],0)
                self.assertEqual(restart.call_args_list[-1].args[1],{"diagnostic_telemetry":None})
                observed=json.loads((evidence/"capture.json").read_text())
                self.assertEqual(observed["restoration"],{"phase":"complete"})
                self.assertEqual(observed["command"],["literal","argument"])


if __name__ == "__main__":
    unittest.main()
