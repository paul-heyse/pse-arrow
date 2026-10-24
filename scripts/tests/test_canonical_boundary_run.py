# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Focused driver controls; no live service, Cargo build or native import."""
# ruff: noqa: PT009, PT027 -- stdlib host-tool controls

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from scripts.tests import canonical_boundary_run as boundary
from scripts.tests import plan30_context_overlap_run as overlap
from scripts.tests import plan50_database_fixture as database_fixture


class RestartPeerTests(unittest.TestCase):
    def test_overlap_composes_the_durable_disposed_resource_and_frozen_result(self) -> None:
        record = {"invocation": "authentic-child", "disposition": "pass", "cleanup": "removed"}
        result = {"invocation": "authentic-child", "terminal_identity": "frozen",
                  "terminal": {"complete": True}, "assurance": "pass", "subjects": [],
                  "resources": [{"resource": "disposed"}]}
        with (
            patch.object(overlap.test_resources, "storage") as store,
            patch.object(overlap.test_resources, "entry_facts", return_value=record),
            patch.object(overlap.run_inspection, "product_result", return_value=result) as read,
            patch.object(overlap.test_resources, "resource_status") as live,
        ):
            facts, invocation = overlap.peer_result("disposed")
        store.return_value.resource_snapshot.assert_called_once_with("disposed")
        read.assert_called_once_with("authentic-child")
        live.assert_not_called()
        self.assertEqual(facts, record)
        self.assertEqual(invocation["terminal_identity"], "frozen")

    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.state = self.directory / "live"
        self.control = self.state / "restart-control"
        self.process = Mock(spec=subprocess.Popen)
        self.process.poll.return_value = None
        self.database = "canonical_test_" + "1" * 32
        self.ready = {"state": str(self.state), "database": self.database}
        self.down = {"database": self.database, "error": "actual closed server"}
        self.complete = {
            "database": self.database, "same_store": True,
            "same_attempt": True, "next_sequence": 2,
        }

    def test_actual_stop_precedes_down_ack_and_restart_precedes_complete(self) -> None:
        events = []

        def wait(directory, name, process, deadline):
            self.assertEqual(directory, self.control)
            self.assertIs(process, self.process)
            events.append(name)
            return {
                "ready.json": self.ready,
                "down-observed.json": self.down,
                "complete.json": self.complete,
            }[name]

        with (
            patch.object(boundary, "wait_record", side_effect=wait),
            patch.object(boundary.server, "config_for", return_value={}),
            patch.object(boundary.server, "stop", side_effect=lambda *_, **__: events.append("stop")) as stop,
            patch.object(boundary.server, "active", return_value=False),
            patch.object(boundary.server, "start", side_effect=lambda *_, **__: events.append("start")),
            patch.object(boundary.server, "unit_name", return_value="owned-fresh.service"),
            patch.object(boundary.native_operation, "unit_observation", side_effect=[
                {"ActiveState": "active", "InvocationID": "original"},
                {"ActiveState": "active", "InvocationID": "restarted"},
            ]),
            patch.object(boundary.server, "write_json", side_effect=lambda path, _: events.append(path.name)),
        ):
            boundary.restart_handshake(self.state, self.control, self.process, time.monotonic() + 5)
        stop.assert_called_once_with(self.state, {}, abrupt=True)
        self.assertEqual(events, [
            "ready.json", "stop", "stopped.json", "down-observed.json",
            "start", "restarted.json", "complete.json",
        ])

    def test_foreign_service_peer_cannot_grant_stop_authority(self) -> None:
        self.ready["state"] = str(self.directory / "selected-main-service")
        with (
            patch.object(boundary, "wait_record", return_value=self.ready),
            patch.object(boundary.server, "stop") as stop,
        ):
            with self.assertRaisesRegex(RuntimeError, "another service"):
                boundary.restart_handshake(self.state, self.control, self.process, time.monotonic() + 5)
        stop.assert_not_called()

    def test_missing_peer_is_bounded_and_child_exit_is_not_an_ack(self) -> None:
        with self.assertRaises(TimeoutError):
            boundary.wait_record(self.control, "ready.json", self.process, time.monotonic() - 1)
        self.process.poll.return_value = 0
        self.process.returncode = 0
        with self.assertRaisesRegex(RuntimeError, "exited before"):
            boundary.wait_record(self.control, "ready.json", self.process, time.monotonic() + 5)

    def test_live_service_does_not_publish_stopped_ack(self) -> None:
        with (
            patch.object(boundary, "wait_record", return_value=self.ready),
            patch.object(boundary.server, "config_for", return_value={}),
            patch.object(boundary.server, "stop"),
            patch.object(boundary.server, "active", return_value=True),
            patch.object(boundary.server, "unit_name", return_value="owned-fresh.service"),
            patch.object(boundary.native_operation, "unit_observation", return_value={"ActiveState": "active", "InvocationID": "original"}),
            patch.object(boundary.server, "write_json") as publish,
            patch.object(boundary.server, "start") as start,
        ):
            with self.assertRaisesRegex(RuntimeError, "did not stop"):
                boundary.restart_handshake(self.state, self.control, self.process, time.monotonic() + 5)
        publish.assert_not_called()
        start.assert_not_called()

    def test_invalid_down_observation_never_starts_service(self) -> None:
        with (
            patch.object(boundary, "wait_record", side_effect=[self.ready, {"database": self.database}]),
            patch.object(boundary.server, "config_for", return_value={}),
            patch.object(boundary.server, "stop"),
            patch.object(boundary.server, "active", return_value=False),
            patch.object(boundary.server, "unit_name", return_value="owned-fresh.service"),
            patch.object(boundary.native_operation, "unit_observation", return_value={"ActiveState": "active", "InvocationID": "original"}),
            patch.object(boundary.server, "write_json"),
            patch.object(boundary.server, "start") as start,
        ):
            with self.assertRaisesRegex(RuntimeError, "actual closed"):
                boundary.restart_handshake(self.state, self.control, self.process, time.monotonic() + 5)
        start.assert_not_called()

    def test_rust_peer_owns_a_distinct_authentic_selected_invocation(self) -> None:
        selected = self.directory / "selected.json"
        selected.write_text("{}")
        execution = self.directory / "execution.json"
        parent = str(self.directory / "python-parent.json")
        child = self.directory / "rust-child.json"
        declaration = {
            "provenance": str(self.directory / "native.json"),
            "selected": str(selected), "worker": str(self.directory / "worker"),
            "arguments": ["prepared"], "parent_invocation": parent,
        }
        catalog = [{"name": boundary.RESTART_TEST, "binary_id": "pse-operations::lib"}]
        with (
            patch.dict(os.environ, {boundary.test_resources.MARKER: parent}),
            patch.object(boundary.server, "read_json", return_value=declaration),
            patch.object(boundary.native_tests, "rust_fixture_bindings", return_value={}),
            patch.object(boundary.validation_receipts, "native_inventory", return_value={}),
            patch.object(boundary.test_run, "rust_selected", return_value=catalog),
            patch.object(boundary.test_resources, "invocation", return_value=child) as invocation,
            patch.object(boundary.test_run, "run_rust", return_value=0) as run,
            patch.object(boundary.server, "file_digest", return_value="a" * 64),
            patch.object(boundary.server, "write_json") as observe,
        ):
            self.assertEqual(boundary.rust_child(execution), 0)
        invocation.assert_called_once_with("rust", catalog)
        self.assertEqual(run.call_args.kwargs["env"][boundary.test_resources.MARKER], str(child))
        self.assertEqual(observe.call_args.args[1]["parent"], parent)
        self.assertEqual(observe.call_args.args[1]["child"], str(child))

    def test_context_overlap_selects_the_pytest_component_peer(self) -> None:
        (self.directory / "python-native.json").write_text("{}")
        with (
            patch.object(sys, "argv", ["overlap", "python-child", str(self.directory)]),
            patch.dict(os.environ, {"PSE_TEST_EXECUTION_PROFILE": "exclusive-observer", "PSE_TEST_REQUIRED_ROLES": "context-overlap"}),
            patch.object(overlap.host_admission, "inherit", return_value=object()),
            patch.object(overlap.native_tests, "python_native_binary", return_value=self.directory / "native.so"),
            patch.object(overlap.producer_deployment, "file_observation", return_value={}),
            patch.object(overlap.test_run, "run_python", return_value=0) as run,
        ):
            self.assertEqual(overlap.main(), 0)
            self.assertEqual(os.environ.get("PSE_TEST_REQUIRED_ROLES"), "context-overlap")
        command = run.call_args.args[0]
        self.assertIn(str(overlap.ROOT / "scripts/tests/test_plan30_context_overlap.py"), command)
        self.assertEqual(command[command.index("-m", 3) + 1], "component")
        self.assertEqual(command[command.index("-n") + 1], "0")

    def test_database_driver_runs_api_and_operator_as_separate_selections(self) -> None:
        tool = self.directory / "xtask"
        tool.touch()
        arguments = [
            "database-fixture", "--query-only", "--state", str(self.state),
            "--tool-root", str(self.directory), "--evidence", str(self.directory / "evidence"),
            "--initializer", str(tool),
        ]
        with (
            patch.object(sys, "argv", arguments),
            patch.object(database_fixture.subprocess, "call", side_effect=[0, 7]) as run,
        ):
            self.assertEqual(database_fixture.main(), 7)
        self.assertEqual(run.call_count, 2)
        api, operator = [call.args[0] for call in run.call_args_list]
        self.assertIn("test(canonical_candidates_inspection) | test(canonical_graph_explanation)", api)
        self.assertIn("scripts.tests.canonical_boundary_run", operator)
        self.assertIn("operator", operator)
        self.assertEqual(operator[-2:], ["--executor", str(tool)])
        self.assertNotIn("PSE_CANONICAL_TOOLS", run.call_args_list[0].kwargs["env"])
        with (
            patch.object(sys, "argv", arguments),
            patch.object(database_fixture.subprocess, "call", return_value=3) as run,
        ):
            self.assertEqual(database_fixture.main(), 3)
        run.assert_called_once()

    def test_selected_operator_missing_prerequisite_fails_within_budget(self) -> None:
        environment = dict(os.environ)
        environment.pop("PSE_CANONICAL_TOOLS", None)
        result = subprocess.run(
            [sys.executable, "-m", "scripts.tests.canonical_inspection_check",
             "--state", str(self.state), "--database", self.database,
             "--kind", "graph", "--request", str(self.directory / "missing-request.json"),
             "--evidence", str(self.directory / "evidence.json")],
            cwd=boundary.ROOT, env=environment, capture_output=True, text=True,
            timeout=5, check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires PSE_CANONICAL_TOOLS", result.stderr)
        self.assertFalse((self.directory / "evidence.json").exists())
        self.assertFalse(self.state.exists())

    def test_controls_register_before_creation_and_keep_distinct_child_receipt(self) -> None:
        directory = self.directory / "new-journey"
        state = directory / "live"
        control = state / "restart-control"
        initializer = self.directory / "target/debug/xtask"
        initializer.parent.mkdir(parents=True)
        initializer.write_text("captured initializer")
        initializer.chmod(0o700)
        events = []

        def register(payload):
            self.assertEqual(payload["directory"], str(control))
            self.assertFalse(control.exists())
            events.append("registered")
            return "1" * 32

        def handshake(*_):
            self.assertTrue(control.is_dir())
            events.append("created")

        def setup(_):
            self.assertNotIn("CMAKE_TOOLCHAIN_FILE", os.environ)
            state.mkdir()

        self.process.wait.return_value = 0
        self.process.poll.return_value = 0
        parent = str(self.directory / "parent-invocation.json")
        peer = {
            "parent": parent, "child": str(self.directory / "child-invocation.json"),
            "terminal": str(self.directory / "child-terminal.json"),
            "terminal_sha256": "a" * 64,
        }
        with (
            patch.dict(os.environ, {boundary.test_resources.MARKER: parent}),
            patch.object(boundary, "ROOT", self.directory),
            patch.object(boundary.reference_run, "freeze_initializer", return_value=initializer),
            patch.object(boundary.native_operation, "environment", side_effect=lambda _, env: {**env, "CMAKE_TOOLCHAIN_FILE": "selected-native-toolchain"}),
            patch.object(boundary.native_tests, "ordinary_rust_capture", return_value=(["prepared"], self.directory / "worker")),
            patch.object(boundary.server, "setup", side_effect=setup),
            patch.object(boundary.server, "initialize"),
            patch.object(boundary.server, "config_for", return_value={}),
            patch.object(boundary.server, "start"),
            patch.object(boundary.server, "stop") as stop,
            patch.object(boundary.test_resources, "require_effect", create=True) as require,
            patch.object(boundary.test_resources, "register", side_effect=register),
            patch.object(boundary.test_resources, "record_drain") as drain,
            patch.object(boundary.test_resources, "read_invocation", return_value={"kind": "rust"}),
            patch.object(boundary.test_resources, "observe_peer") as peer_observation,
            patch.object(boundary.test_resources, "observe_cleanup") as cleanup_observation,
            patch.object(boundary.server, "write_json"),
            patch.object(boundary.server, "read_json", return_value=peer),
            patch.object(boundary.server, "file_digest", return_value="a" * 64),
            patch.object(boundary.subprocess, "Popen", return_value=self.process) as launch,
            patch.object(boundary, "restart_handshake", side_effect=handshake),
        ):
            boundary.restart_journey(directory, "selected-component-peer")
        self.assertEqual([call.kwargs["effect"] for call in require.call_args_list], ["canonical", "native"])
        self.assertEqual(events, ["registered", "created"])
        self.assertNotIn(boundary.test_resources.MARKER, launch.call_args.kwargs["env"])
        self.assertEqual(launch.call_args.kwargs["env"]["PSE_SURREAL_STATE"], str(state))
        self.assertEqual(launch.call_args.kwargs["env"]["CMAKE_TOOLCHAIN_FILE"], "selected-native-toolchain")
        stop.assert_called_once_with(state, {})
        drain.assert_called_once_with("1" * 32)
        peer_observation.assert_called_once_with({"test": "selected-component-peer"}, Path(peer["child"]), role="external-restart")
        cleanup_observation.assert_called_once_with({"test": "selected-component-peer"}, [])

    def test_undeclared_lifecycle_demand_refuses_before_owned_creation(self) -> None:
        directory = self.directory / "never-created"
        with (
            patch.object(boundary.test_resources, "require_effect", create=True,
                         side_effect=RuntimeError("undeclared canonical demand")),
            patch.object(boundary.native_tests, "ordinary_rust_capture") as capture,
            patch.object(boundary.server, "setup") as setup,
        ):
            with self.assertRaisesRegex(RuntimeError, "undeclared canonical"):
                boundary.restart_journey(directory, "unadmitted-test")
        self.assertFalse(directory.exists())
        capture.assert_not_called()
        setup.assert_not_called()

    def test_restart_cannot_use_an_existing_directory(self) -> None:
        with patch.object(boundary.native_tests, "ordinary_rust_capture") as capture:
            with self.assertRaisesRegex(ValueError, "fresh owned"):
                boundary.restart_journey(self.directory, "component-peer")
        capture.assert_not_called()


if __name__ == "__main__":
    unittest.main()
