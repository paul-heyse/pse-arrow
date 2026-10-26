# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Focused driver controls; no live service, Cargo build or native import."""
# ruff: noqa: PT009, PT027 -- stdlib host-tool controls

from __future__ import annotations

import contextlib
import copy
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import Mock, patch

from scripts import operation_lifetime, resource_store
from scripts.tests import canonical_boundary_run as boundary
from scripts.tests import plan30_context_overlap_run as overlap
from scripts.tests import plan50_database_fixture as database_fixture

if TYPE_CHECKING:
    from collections.abc import Generator


class RestartPeerTests(unittest.TestCase):
    def test_overlap_composes_the_durable_disposed_resource_and_frozen_result(
        self,
    ) -> None:
        record = {
            "invocation": "authentic-child",
            "disposition": "pass",
            "cleanup": "removed",
        }
        result = {
            "invocation": "authentic-child",
            "terminal_identity": "frozen",
            "terminal": {"complete": True},
            "assurance": "pass",
            "subjects": [],
            "resources": [{"resource": "disposed"}],
        }
        with (
            patch.object(overlap.test_resources, "storage") as store,
            patch.object(overlap.test_resources, "entry_facts", return_value=record),
            patch.object(
                overlap.run_inspection, "product_result", return_value=result
            ) as read,
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
        self.process.pid = 321
        self.lifecycle_deadline = boundary.server.lifecycle_deadline
        self.options = {
            "controls_resource": "controls",
            "test_identity": "selected-python-peer",
            "selection": {"instance_id": "service-generation", "namespace": "pse"},
        }
        for name in ("lifecycle_deadline", "lifecycle_reservation"):
            patcher = patch.object(
                boundary.server,
                name,
                side_effect=lambda *_, **__: contextlib.nullcontext(),
            )
            patcher.start()
            self.addCleanup(patcher.stop)
        self.database = "canonical_test_" + "1" * 32
        self.ready = {"state": str(self.state), "database": self.database}
        self.down = {"database": self.database, "error": "actual closed server"}
        self.complete = {
            "database": self.database,
            "same_store": True,
            "same_attempt": True,
            "next_sequence": 2,
        }

    def test_actual_stop_precedes_down_ack_and_restart_precedes_complete(self) -> None:
        events = []

        def wait(
            directory: Path,
            name: str,
            process: subprocess.Popen[bytes],
            _deadline: float,
        ) -> dict:
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
            patch.object(boundary, "restart_cohort"),
            patch.object(boundary.server, "config_for", return_value={}),
            patch.object(
                boundary.server,
                "stop",
                side_effect=lambda *_, **__: events.append("stop"),
            ) as stop,
            patch.object(boundary.server, "active", return_value=False),
            patch.object(
                boundary.server,
                "start",
                side_effect=lambda *_, **__: events.append("start"),
            ),
            patch.object(
                boundary.server, "unit_name", return_value="owned-fresh.service"
            ),
            patch.object(
                operation_lifetime,
                "unit_observation",
                side_effect=[
                    {"ActiveState": "active", "InvocationID": "original"},
                    {"ActiveState": "active", "InvocationID": "restarted"},
                ],
            ),
            patch.object(
                boundary.server,
                "write_json",
                side_effect=lambda path, _: events.append(path.name),
            ),
        ):
            boundary.restart_handshake(
                self.state,
                self.control,
                self.process,
                time.monotonic() + 5,
                **self.options,
            )
        stop.assert_called_once_with(self.state, {}, abrupt=True)
        self.assertEqual(
            events,
            [
                "ready.json",
                "stop",
                "stopped.json",
                "down-observed.json",
                "start",
                "restarted.json",
                "complete.json",
            ],
        )

    def test_foreign_service_peer_cannot_grant_stop_authority(self) -> None:
        self.ready["state"] = str(self.directory / "selected-main-service")
        with (
            patch.object(boundary, "wait_record", return_value=self.ready),
            patch.object(boundary.server, "stop") as stop,
            self.assertRaisesRegex(RuntimeError, "another service"),
        ):
            boundary.restart_handshake(
                self.state,
                self.control,
                self.process,
                time.monotonic() + 5,
                **self.options,
            )
        stop.assert_not_called()

    def test_missing_peer_is_bounded_and_child_exit_is_not_an_ack(self) -> None:
        with self.assertRaises(TimeoutError):
            boundary.wait_record(
                self.control, "ready.json", self.process, time.monotonic() - 1
            )
        self.process.poll.return_value = 0
        self.process.returncode = 0
        with self.assertRaisesRegex(RuntimeError, "exited before"):
            boundary.wait_record(
                self.control, "ready.json", self.process, time.monotonic() + 5
            )

    def test_live_service_does_not_publish_stopped_ack(self) -> None:
        with (
            patch.object(boundary, "restart_cohort"),
            patch.object(boundary, "wait_record", return_value=self.ready),
            patch.object(boundary.server, "config_for", return_value={}),
            patch.object(boundary.server, "stop"),
            patch.object(boundary.server, "active", return_value=True),
            patch.object(
                boundary.server, "unit_name", return_value="owned-fresh.service"
            ),
            patch.object(
                operation_lifetime,
                "unit_observation",
                return_value={"ActiveState": "active", "InvocationID": "original"},
            ),
            patch.object(boundary.server, "write_json") as publish,
            patch.object(boundary.server, "start") as start,
            self.assertRaisesRegex(RuntimeError, "did not stop"),
        ):
            boundary.restart_handshake(
                self.state,
                self.control,
                self.process,
                time.monotonic() + 5,
                **self.options,
            )
        publish.assert_not_called()
        start.assert_not_called()

    def test_invalid_down_observation_never_starts_service(self) -> None:
        with (
            patch.object(boundary, "restart_cohort"),
            patch.object(
                boundary,
                "wait_record",
                side_effect=[self.ready, {"database": self.database}],
            ),
            patch.object(boundary.server, "config_for", return_value={}),
            patch.object(boundary.server, "stop"),
            patch.object(boundary.server, "active", return_value=False),
            patch.object(
                boundary.server, "unit_name", return_value="owned-fresh.service"
            ),
            patch.object(
                operation_lifetime,
                "unit_observation",
                return_value={"ActiveState": "active", "InvocationID": "original"},
            ),
            patch.object(boundary.server, "write_json"),
            patch.object(boundary.server, "start") as start,
            self.assertRaisesRegex(RuntimeError, "actual closed"),
        ):
            boundary.restart_handshake(
                self.state,
                self.control,
                self.process,
                time.monotonic() + 5,
                **self.options,
            )
        start.assert_not_called()

    def test_rust_peer_owns_a_distinct_authentic_selected_invocation(self) -> None:
        selected = self.directory / "selected.json"
        selected.write_text("{}")
        execution = self.directory / "execution.json"
        parent = str(self.directory / "python-parent.json")
        child = self.directory / "rust-child.json"
        declaration = {
            "provenance": str(self.directory / "native.json"),
            "selected": str(selected),
            "worker": str(self.directory / "worker"),
            "arguments": ["prepared"],
            "parent_invocation": parent,
        }
        catalog = [{"name": boundary.RESTART_TEST, "binary_id": "pse-operations::lib"}]
        with (
            patch.dict(os.environ, {boundary.test_resources.MARKER: parent}),
            patch.object(boundary.server, "read_json", return_value=declaration),
            patch.object(
                boundary.native_tests, "rust_fixture_bindings", return_value={}
            ),
            patch.object(
                boundary.validation_receipts, "native_inventory", return_value={}
            ),
            patch.object(boundary.test_run, "rust_selected", return_value=catalog),
            patch.object(
                boundary.test_resources, "invocation", return_value=child
            ) as invocation,
            patch.object(boundary.test_run, "run_rust", return_value=0) as run,
            patch.object(boundary.server, "file_digest", return_value="a" * 64),
            patch.object(boundary.server, "write_json") as observe,
        ):
            self.assertEqual(boundary.rust_child(execution), 0)
        invocation.assert_called_once_with("rust", catalog)
        self.assertEqual(
            run.call_args.kwargs["env"][boundary.test_resources.MARKER], str(child)
        )
        self.assertEqual(observe.call_args.args[1]["parent"], parent)
        self.assertEqual(observe.call_args.args[1]["child"], str(child))
        early = observe.call_args_list[0]
        self.assertEqual(early.args[0], execution.with_name("child-ready.json"))
        self.assertEqual(early.args[1], {"parent": parent, "child": str(child)})

    @contextlib.contextmanager
    def cohort_authority(
        self,
    ) -> Generator[tuple[dict[str, dict], dict[str, dict], dict[str, str], Mock]]:
        registry = self.directory / "registry"
        parent_path = registry / "invocations" / "parent.json"
        child_path = registry / "invocations" / "child.json"
        parent = {
            "kind": "python",
            "nonce": "parent",
            "selected": [{"name": "selected-python-peer"}],
        }
        child = {
            "kind": "rust",
            "nonce": "child",
            "selected": [
                {"binary_id": "pse-operations", "name": boundary.RESTART_TEST}
            ],
        }
        binding = {"parent": str(parent_path), "child": str(child_path)}
        producer = {"pid": self.process.pid, "start": "driver-start", "boot": "boot"}
        snapshots = {
            "parent": {"phase": "open", "declaration": parent},
            "child": {"phase": "open", "declaration": child, "producer": producer},
        }
        shared = {
            "state": str(self.state),
            "service_owner": str(self.state),
            "service_generation": "service-generation",
            "namespace": "pse",
            "cleanup": "retained",
            "drained": False,
            "borrows": {},
            "references": {},
            "pid": 123,
            "start": "fixture-start",
            "boot": "boot",
            "creation_borrower": {"pid": 123, "start": "fixture-start", "boot": "boot"},
        }
        records = {
            "controls": {
                **copy.deepcopy(shared),
                "kind": "controls",
                "directory": str(self.control),
                "database": "canonical_test_" + "2" * 32,
                "invocation": "parent",
                "test": "selected-python-peer",
            },
            "database": {
                **copy.deepcopy(shared),
                "kind": "database",
                "database": self.database,
                "invocation": "child",
                "test": "pse-operations::" + boundary.RESTART_TEST,
            },
        }
        with (
            patch.dict(os.environ, {boundary.test_resources.MARKER: str(parent_path)}),
            patch.object(boundary.server, "service_directory", return_value=self.state),
            patch.object(
                boundary.server, "config_for", return_value=self.options["selection"]
            ),
            patch.object(boundary.test_resources, "registry", return_value=registry),
            patch.object(
                boundary.test_resources,
                "read_invocation",
                side_effect=lambda path: parent if path == parent_path else child,
            ),
            patch.object(
                boundary.test_resources,
                "selected_test",
                return_value=("parent", "selected-python-peer"),
            ),
            patch.object(boundary.test_resources, "storage") as store,
            patch.object(boundary.test_resources, "borrower_alive", return_value=True),
            patch.object(
                boundary.test_resources, "service_resources", return_value=records
            ) as query,
            patch.object(
                boundary,
                "wait_record",
                side_effect=lambda _directory, name, *_: {
                    "ready.json": self.ready,
                    "child-ready.json": binding,
                    "down-observed.json": self.down,
                    "complete.json": self.complete,
                }[name],
            ),
        ):
            store.return_value.invocation_snapshot.side_effect = lambda nonce, **_: (
                snapshots[nonce]
            )
            yield records, snapshots, binding, query

    def test_restart_allows_only_its_authenticated_parent_and_live_child_fixture(
        self,
    ) -> None:
        with self.cohort_authority() as (_, _, _, query):
            boundary.restart_cohort(
                self.state,
                self.control,
                self.process,
                time.monotonic() + 5,
                self.database,
                **self.options,
            )
        self.assertEqual(query.call_args.args, (self.state,))
        self.assertIn("deadline", query.call_args.kwargs)

    def test_restart_reads_actual_filestore_snapshots_under_original_deadline(
        self,
    ) -> None:
        with self.cohort_authority() as (_, snapshots, _, query):
            store = resource_store.FileStore(self.directory / "actual-registry")
            for nonce, snapshot in snapshots.items():
                store.create_invocation(snapshot["declaration"])
                if "producer" in snapshot:
                    with store.invocation_edit(nonce) as image:
                        image["producer"] = snapshot["producer"]
            for expired in (False, True):
                query.reset_mock()
                deadline = time.monotonic() + (-1 if expired else 5)
                with (
                    self.subTest(expired=expired),
                    patch.object(
                        boundary.test_resources, "storage", return_value=store
                    ),
                    patch.object(
                        boundary.server,
                        "lifecycle_deadline",
                        side_effect=self.lifecycle_deadline,
                    ),
                    patch.object(
                        store, "invocation_snapshot", wraps=store.invocation_snapshot
                    ) as snapshot_read,
                    patch.object(
                        boundary.server, "remaining", wraps=boundary.server.remaining
                    ) as remaining,
                ):
                    if expired:
                        with self.assertRaises(boundary.server.DeadlineExpiredError):
                            boundary.restart_cohort(
                                self.state,
                                self.control,
                                self.process,
                                deadline,
                                self.database,
                                **self.options,
                            )
                        snapshot_read.assert_not_called()
                        query.assert_not_called()
                        self.assertEqual(remaining.call_count, 1)
                    else:
                        boundary.restart_cohort(
                            self.state,
                            self.control,
                            self.process,
                            deadline,
                            self.database,
                            **self.options,
                        )
                        self.assertEqual(
                            [call.args for call in snapshot_read.call_args_list],
                            [("parent",), ("child",)],
                        )
                        query.assert_called_once_with(self.state, deadline=deadline)
                        self.assertEqual(remaining.call_count, 4)
                    self.assertTrue(
                        all(
                            call.args == (deadline,)
                            for call in remaining.call_args_list
                        )
                    )

    def test_foreign_creation_retained_reader_and_changed_binding_refuse_before_stop(
        self,
    ) -> None:
        for change in (
            "foreign-creation",
            "foreign-reader",
            "child-reader",
            "retained-reference",
            "wrong-generation",
            "wrong-child",
            "wrong-process",
            "wrong-parent",
            "uncovered",
        ):
            with (
                self.subTest(change=change),
                self.cohort_authority() as (records, snapshots, binding, query),
            ):
                if change in {"foreign-creation", "foreign-reader"}:
                    records["foreign"] = {
                        **copy.deepcopy(records["database"]),
                        "invocation": "another-invocation",
                    }
                    if change == "foreign-reader":
                        records["foreign"].pop("creation_borrower")
                        records["foreign"]["borrows"] = {
                            "reader": {"pid": 999, "start": "reader-start"}
                        }
                elif change == "child-reader":
                    records["database"]["borrows"] = {
                        "reader": {"pid": 999, "start": "reader-start"}
                    }
                elif change == "retained-reference":
                    records["database"]["references"] = {"report": "digest"}
                elif change == "wrong-generation":
                    records["database"]["service_generation"] = (
                        "another-service-generation"
                    )
                elif change == "wrong-child":
                    records["database"]["invocation"] = "another-child"
                elif change == "wrong-process":
                    snapshots["child"]["producer"]["pid"] = 999
                elif change == "wrong-parent":
                    binding["parent"] = "another-parent"
                else:
                    query.side_effect = boundary.test_resources.ResourceError(
                        "unestablished coverage"
                    )
                with (
                    patch.object(boundary.server, "stop") as stop,
                    self.assertRaises(
                        (RuntimeError, boundary.test_resources.ResourceError)
                    ),
                ):
                    boundary.restart_handshake(
                        self.state,
                        self.control,
                        self.process,
                        time.monotonic() + 5,
                        **self.options,
                    )
                stop.assert_not_called()

    def test_guard_and_restart_share_reservation_but_peer_cleanup_runs_after_release(
        self,
    ) -> None:
        reserved = False

        @contextlib.contextmanager
        def reservation(*_: object, **__: object) -> Generator[None]:
            nonlocal reserved
            reserved = True
            try:
                yield
            finally:
                reserved = False

        def assert_reserved(*_: object, **__: object) -> None:
            self.assertTrue(reserved)

        with self.cohort_authority() as (_, _, _, query):
            original_query = query.return_value
            query.side_effect = lambda *_args, **_kwargs: (
                assert_reserved(),
                original_query,
            )[1]
            wait_mock = boundary.wait_record
            if not isinstance(wait_mock, Mock):
                self.fail("cohort fixture must own the wait_record mock")
            wait = wait_mock.side_effect
            if not callable(wait):
                self.fail(
                    "cohort wait_record fixture must provide its callable phase reader"
                )

            def wait_phase(
                directory: Path,
                name: str,
                process: subprocess.Popen[bytes],
                deadline: float,
            ) -> dict:
                if name == "complete.json":
                    self.assertFalse(
                        reserved, "peer fixture cleanup must regain admission"
                    )
                return wait(directory, name, process, deadline)

            with (
                patch.object(
                    boundary.server, "lifecycle_reservation", side_effect=reservation
                ),
                patch.object(boundary, "wait_record", side_effect=wait_phase),
                patch.object(boundary.server, "stop", side_effect=assert_reserved),
                patch.object(boundary.server, "active", return_value=False),
                patch.object(boundary.server, "start", side_effect=assert_reserved),
                patch.object(
                    boundary.server, "unit_name", return_value="owned.service"
                ),
                patch.object(
                    operation_lifetime,
                    "unit_observation",
                    side_effect=[
                        {"ActiveState": "active", "InvocationID": "original"},
                        {"ActiveState": "active", "InvocationID": "restarted"},
                    ],
                ),
                patch.object(boundary.server, "write_json"),
            ):
                boundary.restart_handshake(
                    self.state,
                    self.control,
                    self.process,
                    time.monotonic() + 5,
                    **self.options,
                )
        self.assertFalse(reserved)

    def test_context_overlap_selects_the_pytest_component_peer(self) -> None:
        (self.directory / "python-native.json").write_text("{}")
        with (
            patch.object(sys, "argv", ["overlap", "python-child", str(self.directory)]),
            patch.dict(
                os.environ,
                {
                    "PSE_TEST_EXECUTION_PROFILE": "exclusive-observer",
                    "PSE_TEST_REQUIRED_ROLES": "context-overlap",
                },
            ),
            patch.object(overlap.host_admission, "inherit", return_value=object()),
            patch.object(
                overlap.native_tests,
                "python_native_binary",
                return_value=self.directory / "native.so",
            ),
            patch.object(
                overlap.producer_deployment, "file_observation", return_value={}
            ),
            patch.object(overlap.test_run, "run_python", return_value=0) as run,
        ):
            self.assertEqual(overlap.main(), 0)
            self.assertEqual(
                os.environ.get("PSE_TEST_REQUIRED_ROLES"), "context-overlap"
            )
        command = run.call_args.args[0]
        self.assertIn(
            str(overlap.ROOT / "scripts/tests/test_plan30_context_overlap.py"), command
        )
        self.assertEqual(command[command.index("-m", 3) + 1], "component")
        self.assertEqual(command[command.index("-n") + 1], "0")
        self.assertEqual(
            command[command.index("--basetemp") + 1],
            str(self.directory / "python-tmp"),
        )

    def test_database_driver_runs_api_and_operator_as_separate_selections(self) -> None:
        tool = self.directory / "xtask"
        tool.touch()
        arguments = [
            "database-fixture",
            "--query-only",
            "--state",
            str(self.state),
            "--tool-root",
            str(self.directory),
            "--evidence",
            str(self.directory / "evidence"),
            "--initializer",
            str(tool),
        ]
        with (
            patch.object(sys, "argv", arguments),
            patch.object(
                database_fixture.subprocess, "call", side_effect=[0, 7]
            ) as run,
        ):
            self.assertEqual(database_fixture.main(), 7)
        self.assertEqual(run.call_count, 2)
        api, operator = [call.args[0] for call in run.call_args_list]
        self.assertIn(
            "test(canonical_candidates_inspection) | test(canonical_graph_explanation)",
            api,
        )
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
            [
                sys.executable,
                "-m",
                "scripts.tests.canonical_inspection_check",
                "--state",
                str(self.state),
                "--database",
                self.database,
                "--kind",
                "graph",
                "--request",
                str(self.directory / "missing-request.json"),
                "--evidence",
                str(self.directory / "evidence.json"),
            ],
            cwd=boundary.ROOT,
            env=environment,
            capture_output=True,
            text=True,
            timeout=5,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires PSE_CANONICAL_TOOLS", result.stderr)
        self.assertFalse((self.directory / "evidence.json").exists())
        self.assertFalse(self.state.exists())

    def test_controls_register_before_creation_and_keep_distinct_child_receipt(
        self,
    ) -> None:
        directory = self.directory / "new-journey"
        state = directory / "live"
        control = state / "restart-control"
        initializer = self.directory / "actual-cargo-artifact"
        initializer.write_text("captured initializer")
        initializer.chmod(0o700)
        events = []

        def register(payload: dict[str, object]) -> str:
            self.assertEqual(payload["directory"], str(control))
            self.assertFalse(control.exists())
            events.append("registered")
            return "1" * 32

        def handshake(*_: object, **__: object) -> None:
            self.assertTrue(control.is_dir())
            events.append("created")

        def setup(_: Path) -> None:
            self.assertNotIn("CMAKE_TOOLCHAIN_FILE", os.environ)
            events.append("setup")
            state.mkdir()

        def prepare(extra: list[str], environment: dict[str, str]) -> Path:
            self.assertEqual(extra, [])
            self.assertNotIn("CMAKE_TOOLCHAIN_FILE", environment)
            environment["PREPARATION_PRIVATE_INPUT"] = "not exported"
            events.append("prepare")
            return initializer

        def freeze(source: Path, destination: Path) -> Path:
            self.assertEqual(source, initializer)
            self.assertEqual(destination, directory / "tools.json")
            events.append("freeze")
            return initializer

        def capture(*_: object) -> tuple[list[str], Path]:
            events.append("capture")
            return ["prepared"], self.directory / "worker"

        self.process.wait.return_value = 0
        self.process.poll.return_value = 0
        parent = str(self.directory / "parent-invocation.json")
        peer = {
            "parent": parent,
            "child": str(self.directory / "child-invocation.json"),
            "terminal": str(self.directory / "child-terminal.json"),
            "terminal_sha256": "a" * 64,
        }
        with (
            patch.dict(os.environ, {boundary.test_resources.MARKER: parent}),
            patch.object(
                boundary.native_tests, "canonical_tools_binary", side_effect=prepare
            ),
            patch.object(
                boundary.reference_run, "freeze_initializer", side_effect=freeze
            ),
            patch.object(
                boundary.native_operation,
                "environment",
                side_effect=lambda _, env: {
                    **env,
                    "CMAKE_TOOLCHAIN_FILE": "selected-native-toolchain",
                },
            ),
            patch.object(
                boundary.native_tests,
                "ordinary_rust_capture",
                side_effect=capture,
            ),
            patch.object(boundary.server, "setup", side_effect=setup),
            patch.object(boundary.server, "initialize"),
            patch.object(boundary.server, "config_for", return_value={}),
            patch.object(boundary.server, "start"),
            patch.object(boundary.server, "stop") as stop,
            patch.object(
                boundary.test_resources, "require_effect", create=True
            ) as require,
            patch.object(boundary.test_resources, "register", side_effect=register),
            patch.object(boundary.test_resources, "record_drain") as drain,
            patch.object(
                boundary.test_resources,
                "read_invocation",
                return_value={"kind": "rust"},
            ),
            patch.object(boundary.test_resources, "observe_peer") as peer_observation,
            patch.object(
                boundary.test_resources, "observe_cleanup"
            ) as cleanup_observation,
            patch.object(boundary.server, "write_json"),
            patch.object(boundary.server, "read_json", return_value=peer),
            patch.object(boundary.server, "file_digest", return_value="a" * 64),
            patch.object(
                boundary.subprocess, "Popen", return_value=self.process
            ) as launch,
            patch.object(boundary, "restart_handshake", side_effect=handshake),
        ):
            boundary.restart_journey(directory, "selected-component-peer")
        self.assertEqual(
            [call.kwargs["effect"] for call in require.call_args_list],
            ["canonical", "native"],
        )
        self.assertEqual(
            events, ["prepare", "freeze", "capture", "setup", "registered", "created"]
        )
        self.assertNotIn("PREPARATION_PRIVATE_INPUT", os.environ)
        self.assertFalse((self.directory / "target/debug/xtask").exists())
        self.assertNotIn(boundary.test_resources.MARKER, launch.call_args.kwargs["env"])
        self.assertEqual(
            launch.call_args.kwargs["env"]["PSE_SURREAL_STATE"], str(state)
        )
        self.assertEqual(
            launch.call_args.kwargs["env"]["CMAKE_TOOLCHAIN_FILE"],
            "selected-native-toolchain",
        )
        stop.assert_called_once_with(state, {})
        drain.assert_called_once_with("1" * 32)
        peer_observation.assert_called_once_with(
            {"test": "selected-component-peer"},
            Path(peer["child"]),
            role="external-restart",
        )
        cleanup_observation.assert_called_once_with(
            {"test": "selected-component-peer"}, []
        )

    def test_undeclared_lifecycle_demand_refuses_before_owned_creation(self) -> None:
        directory = self.directory / "never-created"
        with (
            patch.object(
                boundary.test_resources,
                "require_effect",
                create=True,
                side_effect=RuntimeError("undeclared canonical demand"),
            ),
            patch.object(boundary.native_tests, "ordinary_rust_capture") as capture,
            patch.object(boundary.server, "setup") as setup,
            self.assertRaisesRegex(RuntimeError, "undeclared canonical"),
        ):
            boundary.restart_journey(directory, "unadmitted-test")
        self.assertFalse(directory.exists())
        capture.assert_not_called()
        setup.assert_not_called()

    def test_restart_cannot_use_an_existing_directory(self) -> None:
        with (
            patch.object(boundary.native_tests, "ordinary_rust_capture") as capture,
            self.assertRaisesRegex(ValueError, "fresh owned"),
        ):
            boundary.restart_journey(self.directory, "component-peer")
        capture.assert_not_called()


if __name__ == "__main__":
    unittest.main()
