# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Terminal outcome, protected references and crash-resumable cleanup controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import fcntl
import json
import os
import subprocess
import tempfile
import threading
import unittest
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import surreal_server

if TYPE_CHECKING:
    from collections.abc import Generator
from unittest.mock import patch

from scripts import test_resources as resources


class TestResourceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        registry = patch.object(
            resources, "registry", return_value=self.root / "registry"
        )
        registry.start()
        self.addCleanup(registry.stop)
        placement = patch.object(resources.host, "inherit", return_value=None)
        placement.start()
        self.addCleanup(placement.stop)
        self.state = self.root / "state"
        self.state.mkdir(mode=0o700)
        self.config: dict[str, object] = {
            "instance_id": "owned-service",
            "namespace": "pse",
            "admission": "open",
            "accepting_writes": True,
        }
        config = patch("scripts.surreal_server.config_for", return_value=self.config)
        config.start()
        self.addCleanup(config.stop)
        self.invocation = resources.invocation(
            "python", [{"name": "test_module.py::test_assert_after_exit"}]
        )
        environment = patch.dict("os.environ", {resources.MARKER: str(self.invocation)})
        environment.start()
        self.addCleanup(environment.stop)

    def finish_report(
        self,
        resource: str,
        outcome: dict,
        *,
        roles: dict[str, str] | None = None,
        required_provenance: list[str] | None = None,
    ) -> None:
        directory = Path(resources.resource_status(resource)["state"])
        receipt = {
            "version": 5,
            "input_coverage": True,
            "source_unchanged": True,
            "provenance_errors": [],
            "scope": [{"name": "fixture-control"}],
            "checks": [{"gate": "fixture-control", "status": "passed"}],
            **outcome,
        }
        (directory / "checks.json").write_text(json.dumps(receipt))
        if roles is None:
            roles = {"checks.json": "receipt"}
            if (directory / "run.log").exists():
                roles["run.log"] = "scratch"
        (directory / "fixture-source.json").write_text('{"scope":"filesystem-control"}')
        roles["fixture-source.json"] = "provenance"
        resources.finish_report(
            resource,
            receipt,
            roles=roles,
            required_provenance=required_provenance or ["fixture-source.json"],
        )

    def register(self) -> str:
        return resources.register(
            {
                "state": str(self.state),
                "database": "canonical_test_" + "a" * 32,
                "test": "test_module.py::test_assert_after_exit",
            }
        )

    def test_codec_registration_refuses_live_lifecycle_before_publication(self) -> None:

        with surreal_server.lifecycle_reservation(self.state):
            with self.assertRaisesRegex(
                surreal_server.SupervisorError, "lifecycle operation"
            ):
                self.register()
            self.assertEqual(resources.resource_status(), {})

    def test_codec_registration_refuses_closed_service_before_publication(self) -> None:

        for admission, accepting in (("quiescing", True), ("open", False)):
            with self.subTest(admission=admission, accepting=accepting):
                self.config.update(admission=admission, accepting_writes=accepting)
                with self.assertRaisesRegex(
                    surreal_server.SupervisorError, "admission is closed"
                ):
                    self.register()
                self.assertEqual(resources.resource_status(), {})

    def test_codec_registration_rechecks_snapshot_inside_admission(self) -> None:

        for change in (
            {"instance_id": "new-generation"},
            {"namespace": "new_namespace"},
            {"endpoint": "ws://127.0.0.1:2"},
        ):
            updated = {**self.config, **change}
            with (
                self.subTest(change=change),
                patch.object(
                    surreal_server,
                    "config_for",
                    side_effect=[dict(self.config), updated, updated],
                ),
            ):
                with self.assertRaisesRegex(
                    resources.ResourceError, "configuration changed"
                ):
                    self.register()
                self.assertEqual(resources.resource_status(), {})

    def test_codec_publication_holds_service_gate_without_startup_ipc(self) -> None:

        metadata = resources.host.metadata

        @contextlib.contextmanager
        def gated(
            directory: Path,
        ) -> Generator[resources.host.MetadataLedger, None, None]:
            descriptor = os.open(self.state / ".supervisor.lock", os.O_RDWR)
            try:
                with self.assertRaises(BlockingIOError):
                    fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                os.close(descriptor)
            with metadata(directory) as ledger:
                yield ledger

        with (
            patch.object(resources.host, "metadata", side_effect=gated),
            patch.object(surreal_server, "start") as start,
        ):
            resource = self.register()
        start.assert_not_called()
        self.assertEqual(
            resources.resource_status(resource)["service_generation"], "owned-service"
        )

    def test_demand_startup_precedes_borrow_publication_and_service_gate(self) -> None:

        self.config.update(profile_version=2, parked=False)

        def startup(state: Path, _config: dict[str, object]) -> None:
            self.assertEqual(resources.resource_status(), {})
            descriptor = os.open(
                state / ".supervisor.lock", os.O_CREAT | os.O_RDWR, 0o600
            )
            try:
                fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                os.close(descriptor)

        with (
            patch.object(surreal_server, "ready", return_value=False),
            patch.object(surreal_server, "start", side_effect=startup) as start,
        ):
            resource = self.register()
        start.assert_called_once()
        self.assertFalse(resources.resource_status(resource)["drained"])

    def test_parked_demand_without_actual_owner_never_publishes_borrow(self) -> None:

        self.config.update(profile_version=2, parked=True)
        with (
            patch.object(surreal_server, "unpark_service") as unpark,
            self.assertRaisesRegex(resources.ResourceError, "actual admitted owner"),
        ):
            self.register()
        unpark.assert_not_called()
        self.assertEqual(resources.resource_status(), {})

    def passed_resource(self) -> str:
        resource = self.register()
        resources.record_drain(resource)
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "pass"},
            complete=True,
            terminal_owner="runner",
        )
        return resource

    def test_cleanup_claim_refuses_lifecycle_without_marking_removing(self) -> None:

        resource = self.passed_resource()
        with (
            surreal_server.lifecycle_reservation(self.state),
            patch.object(resources, "remove_owned") as remove,
            self.assertRaisesRegex(
                surreal_server.SupervisorError, "lifecycle operation"
            ),
        ):
            resources.reclaim(resource)
        remove.assert_not_called()
        self.assertEqual(resources.resource_status(resource)["cleanup"], "retained")

    def test_cleanup_claim_refuses_stale_configuration(self) -> None:

        resource = self.passed_resource()
        updated = {**self.config, "instance_id": "changed-generation"}
        with (
            patch.object(
                surreal_server,
                "config_for",
                side_effect=[dict(self.config), updated, updated],
            ),
            patch.object(resources, "remove_owned") as remove,
            self.assertRaisesRegex(
                resources.ResourceError, "cleanup ownership changed"
            ),
        ):
            resources.reclaim(resource)
        remove.assert_not_called()
        self.assertEqual(resources.resource_status(resource)["cleanup"], "retained")

    def test_cleanup_rechecks_new_reference_under_admission(self) -> None:

        resource = self.passed_resource()
        admission = surreal_server.context_admission

        @contextlib.contextmanager
        def reference_arrives(state: Path) -> Generator[None, None, None]:
            with admission(state):
                resources.retain_reference(resource, "new-consumer", "f" * 64)
                yield

        with (
            patch.object(
                surreal_server, "context_admission", side_effect=reference_arrives
            ),
            patch.object(resources, "remove_owned") as remove,
        ):
            self.assertEqual(
                resources.reclaim(resource), "protected reference or active borrow"
            )
        remove.assert_not_called()
        self.assertEqual(resources.resource_status(resource)["cleanup"], "retained")

    def test_live_cleaner_prevents_lifecycle_after_short_claim_gate_released(
        self,
    ) -> None:

        resource = self.passed_resource()

        def remove(record: resources.ResourceRecord) -> None:
            self.assertTrue(record["drained"])
            self.assertEqual(resources.resource_status(resource)["cleanup"], "removing")
            self.assertTrue(resources.borrower_alive(record["cleaner"]))
            descriptor = os.open(self.state / ".supervisor.lock", os.O_RDWR)
            try:
                fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                os.close(descriptor)
            with (
                surreal_server.lifecycle_reservation(self.state),
                self.assertRaisesRegex(surreal_server.SupervisorError, "cleanup"),
            ):
                surreal_server.require_borrowers_drained(self.state)

        with patch.object(resources, "remove_owned", side_effect=remove):
            self.assertEqual(resources.reclaim(resource), "removed")

    def test_evidence_cleanup_never_enters_service_admission(self) -> None:

        directory = self.root / "build" / "report"
        directory.mkdir(parents=True)
        (directory / "run.log").write_text("disposable fixture output")
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource, {"complete": True, "required_checks_covered": True}
        )
        with patch.object(
            surreal_server,
            "context_admission",
            side_effect=AssertionError("evidence has no service"),
        ):
            self.assertEqual(resources.reclaim(resource), "removed")

    def test_collection_catalog_is_frozen_to_actual_collector(self) -> None:

        names = ["test_module.py::test_assert_after_exit"]
        self.assertTrue(resources.publish_python_collection(self.invocation, names))
        owner = resources.read_invocation(self.invocation)["collection_owner"]
        self.assertEqual(owner["pid"], __import__("os").getpid())
        with self.assertRaisesRegex(resources.ResourceError, "frozen catalog"):
            resources.publish_python_collection(self.invocation, [])
        with (
            patch.object(resources.os, "getpid", return_value=owner["pid"] + 1),
            patch.object(
                resources.operation, "start_identity", return_value="nested-generation"
            ),
        ):
            self.assertFalse(
                resources.publish_python_collection(self.invocation, names)
            )
            with self.assertRaisesRegex(resources.ResourceError, "outside"):
                resources.publish_python_collection(
                    self.invocation, ["unselected::child"]
                )
        self.assertEqual(
            resources.read_invocation(self.invocation)["selected"], [{"name": names[0]}]
        )

    def test_exact_cli_prompts_parse_complete_json(self) -> None:
        self.assertEqual(
            resources.cli_json("pse> [null]\n\npse> ", namespace="pse"), [None]
        )
        self.assertEqual(
            resources.cli_json(
                'pse/fixture> [["value"]]\n\npse/fixture> ',
                namespace="pse",
                database="fixture",
            ),
            [["value"]],
        )
        self.assertEqual(resources.cli_json("> [null]\n\n> "), [None])
        self.assertEqual(resources.cli_json("[null]\n", namespace="pse"), [None])

    def test_cli_prompt_parser_never_discards_unknown_output(self) -> None:

        for output in (
            "other> [null]\n\nother> ",
            "pse> [null]\nerror\npse> ",
            "pse> [null]\n[null]\npse> ",
            "pse/other> [null]\npse/other> ",
        ):
            with self.subTest(output=output), self.assertRaises(json.JSONDecodeError):
                resources.cli_json(output, namespace="pse")
        self.assertEqual(
            resources.cli_json('pse> ["pse> error"]\npse> ', namespace="pse"),
            ["pse> error"],
        )
        with self.assertRaises(resources.ResourceError):
            resources.cli_json("[null]", database="fixture")

    def test_cleanup_requires_exact_complete_success_without_diagnostics(self) -> None:

        credentials = self.root / "credentials.json"
        credentials.write_text('{"username":"control","password":"not-a-live-secret"}')
        config = {
            "credentials_file": str(credentials),
            "server": {"binary": "/unused/control-cli"},
            "endpoint": "ws://127.0.0.1:1",
            "namespace": "pse",
        }
        record = {"state": str(self.state), "database": "canonical_test_" + "a" * 32}
        for stdout, stderr, code in (
            ("pse> [null, null]\npse> ", "", 0),
            ("pse> [null]\npse> ", "query failed", 0),
            ("pse> [null]\npse> ", "", 1),
        ):
            with (
                self.subTest(stdout=stdout, stderr=stderr, code=code),
                patch.object(
                    resources.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess([], code, stdout, stderr),
                ),
                self.assertRaises(resources.ResourceError),
            ):
                resources.remove_database(record, config)
        with patch.object(
            resources.subprocess,
            "run",
            return_value=subprocess.CompletedProcess([], 0, "pse> [null]\n\npse> ", ""),
        ):
            resources.remove_database(record, config)

    def test_failure_after_drop_retains_database_and_controls(self) -> None:
        resource = self.register()
        resources.record_drain(resource)
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "failure"},
            complete=True,
            terminal_owner="runner",
        )
        with patch.object(resources, "remove_owned") as removal:
            self.assertEqual(resources.reclaim(resource), "retention pin")
            removal.assert_not_called()
        self.assertEqual(resources.resource_status(resource)["disposition"], "failure")

    def test_operator_pin_survives_successful_terminal_disposition(self) -> None:
        resource = self.register()
        resources.pin(resource, True)
        resources.record_drain(resource)
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "pass"},
            complete=True,
            terminal_owner="runner",
        )
        with patch.object(resources, "remove_owned") as removal:
            self.assertEqual(resources.reclaim(resource), "retention pin")
            removal.assert_not_called()
        resources.pin(resource, False)
        with patch.object(resources, "remove_owned") as removal:
            self.assertEqual(resources.reclaim(resource), "removed")
            removal.assert_called_once()

    def test_missing_collection_or_terminal_never_infers_pass(self) -> None:
        resource = self.register()
        resources.record_drain(resource)
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "pass"},
            complete=False,
            terminal_owner="runner",
        )
        self.assertEqual(
            resources.resource_status(resource)["disposition"], "incomplete"
        )
        self.assertEqual(resources.reclaim(resource), "retention pin")

    def test_only_reconciled_pass_and_actual_drain_can_remove(self) -> None:
        resource = self.register()
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "pass"},
            complete=True,
            terminal_owner="runner",
        )
        self.assertEqual(resources.reclaim(resource), "fixture borrowers not drained")
        resources.record_drain(
            resource,
            units=[{"unit": "owned.scope", "group": "/owned", "invocation": "b" * 32}],
        )
        with patch.object(resources.operation, "drained", return_value=False):
            self.assertEqual(
                resources.reclaim(resource), "supervised descendant not drained"
            )
        with (
            patch.object(resources.operation, "drained", return_value=True),
            patch.object(resources, "remove_owned") as removal,
        ):
            self.assertEqual(resources.reclaim(resource), "removed")
            self.assertEqual(resources.reclaim(resource), "removed")
            removal.assert_called_once()

    def test_reference_acquisition_precedes_read_and_reference_retains_origin(
        self,
    ) -> None:
        resource = self.register()
        resources.record_drain(resource)
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "pass"},
            complete=True,
            terminal_owner="runner",
        )
        with resources.borrow(resource):
            self.assertEqual(
                resources.reclaim(resource), "protected reference or active borrow"
            )
            resources.retain_reference(resource, "new-receipt", "c" * 64)
        self.assertEqual(
            resources.reclaim(resource), "protected reference or active borrow"
        )
        resources.pin(resource, False)
        self.assertEqual(
            resources.reclaim(resource), "protected reference or active borrow"
        )
        with self.assertRaisesRegex(resources.ResourceError, "digest changed"):
            resources.release_reference(resource, "new-receipt", "d" * 64)
        self.assertEqual(
            resources.reclaim(resource), "protected reference or active borrow"
        )
        resources.release_reference(resource, "new-receipt", "c" * 64)
        with patch.object(resources, "remove_owned") as removal:
            self.assertEqual(resources.reclaim(resource), "removed")
            removal.assert_called_once()

    def test_cleanup_error_pins_and_explicit_release_resumes_same_owned_removal(
        self,
    ) -> None:
        resource = self.register()
        resources.record_drain(resource)
        resources.pin(resource, False)
        with (
            patch.object(resources, "remove_owned", side_effect=OSError("interrupted")),
            self.assertRaises(OSError),
        ):
            resources.reclaim(resource)
        self.assertTrue(resources.resource_status(resource)["pin"])
        resources.pin(resource, False)
        with patch.object(resources, "remove_owned") as removal:
            self.assertEqual(resources.reclaim(resource), "removed")
            removal.assert_called_once()
        with self.assertRaises(resources.ResourceError), resources.borrow(resource):
            pass

    def test_unassociated_bare_tool_starts_incomplete_and_pinned(self) -> None:
        with patch.dict("os.environ", {}, clear=True):
            resource = self.register()
        record = resources.resource_status(resource)
        self.assertIsNone(record["invocation"])
        self.assertTrue(record["pin"])
        self.assertEqual(record["disposition"], "incomplete")

    def test_wrong_terminal_owner_and_duplicate_identity_refuse(self) -> None:
        self.register()
        with self.assertRaises(resources.ResourceError):
            self.register()
        with self.assertRaises(resources.ResourceError):
            resources.finalize(
                self.invocation, {}, complete=True, terminal_owner="assessment"
            )

    def test_concurrent_reclaim_has_one_effect_owner(self) -> None:

        resource = self.register()
        resources.record_drain(resource)
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "pass"},
            complete=True,
            terminal_owner="runner",
        )
        entered = threading.Event()
        release = threading.Event()
        results = []

        def removal(_record: resources.ResourceRecord) -> None:
            entered.set()
            self.assertTrue(release.wait(2))

        with patch.object(resources, "remove_owned", side_effect=removal) as remove:
            thread = threading.Thread(
                target=lambda: results.append(resources.reclaim(resource))
            )
            thread.start()
            self.assertTrue(entered.wait(2))
            self.assertEqual(resources.reclaim(resource), "active cleanup owner")
            release.set()
            thread.join(2)
            self.assertFalse(thread.is_alive())
            self.assertEqual(results, ["removed"])
            remove.assert_called_once()

    def test_origin_borrow_transfers_reference_before_cleanup(self) -> None:
        directory = self.root / "build" / "origin"
        directory.mkdir(parents=True)
        (directory / "checks.json").write_text('{"version":5}')
        (directory / "scope.json").write_text("{}")
        (directory / "run.log").write_text("owned bulky output")
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource, {"complete": True, "required_checks_covered": True}
        )
        with resources.reference_report(directory, self.root / "build" / "consumer"):
            self.assertEqual(
                resources.reclaim(resource), "protected reference or active borrow"
            )
        self.assertEqual(
            resources.reclaim(resource), "protected reference or active borrow"
        )
        self.assertTrue((directory / "run.log").exists())
        self.assertTrue(resources.resource_status(resource)["references"])

    def test_report_compaction_preserves_receipt_and_unknown_history(self) -> None:
        directory = self.root / "build" / "owned"
        directory.mkdir(parents=True)
        (directory / "checks.json").write_text("{}")
        (directory / "selected.json").write_text("{}")
        (directory / "run.log").write_text("owned bulky output")
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource, {"complete": True, "required_checks_covered": True}
        )
        self.assertEqual(resources.reclaim(resource), "removed")
        self.assertTrue((directory / "checks.json").exists())
        self.assertTrue((directory / "selected.json").exists())
        self.assertFalse((directory / "run.log").exists())
        self.assertIsNone(resources.report_resource(self.root / "old-history"))

    def test_automatic_report_compaction_waits_for_actual_group_drain(self) -> None:
        directory = self.root / "build" / "drain-report"
        directory.mkdir(parents=True)
        (directory / "checks.json").write_text("{}")
        (directory / "run.log").write_text("owned bulky output")
        resource = resources.register_report(directory, root=self.root)
        owner = resources.host.acquire(
            resources.host.select("light"), directory=self.root / "host"
        )
        owner.register("owned-report.scope")
        with resources.host.allocation_metadata(owner.directory) as ledger:
            ledger["owners"][owner.nonce]["units"]["owned-report.scope"] = {
                "group": "/owned",
                "invocation": "a" * 32,
            }
        with patch.object(resources.host, "inherit", return_value=owner):
            self.finish_report(
                resource, {"complete": True, "required_checks_covered": True}
            )
        with patch.object(resources.operation, "drained", return_value=False):
            self.assertEqual(resources.reclaim_reports(), [])
            self.assertTrue((directory / "run.log").exists())
        with patch.object(resources.operation, "drained", return_value=True):
            self.assertEqual(resources.reclaim_reports(), [])
            self.assertFalse((directory / "run.log").exists())
        self.assertTrue((directory / "checks.json").exists())

    def report(self, name: str) -> tuple[str, Path]:
        directory = self.root / "build" / name
        directory.mkdir(parents=True)
        (directory / "checks.json").write_text("{}")
        (directory / "run.log").write_text("owned output")
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource, {"complete": True, "required_checks_covered": True}
        )
        return resource, directory

    def test_report_compaction_refuses_symlinked_artifact_ancestor(self) -> None:
        directory = self.root / "build" / "ancestor"
        directory.mkdir(parents=True)
        nested = directory / "logs"
        nested.mkdir()
        (nested / "run.log").write_text("owned output")
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource,
            {"complete": True, "required_checks_covered": True},
            roles={"checks.json": "receipt", "logs/run.log": "scratch"},
        )
        external = self.root / "external"
        nested.rename(external)
        nested.symlink_to(external, target_is_directory=True)
        with self.assertRaisesRegex(resources.ResourceError, "ownership changed"):
            resources.reclaim(resource)
        self.assertEqual((external / "run.log").read_text(), "owned output")
        self.assertTrue(resources.resource_status(resource)["pin"])

    def test_report_compaction_refuses_changed_report_directory_identity(self) -> None:
        resource, directory = self.report("replacement")
        retained = self.root / "retained"
        directory.rename(retained)
        directory.mkdir()
        (directory / "run.log").write_text("owned output")
        with self.assertRaisesRegex(resources.ResourceError, "ownership changed"):
            resources.reclaim(resource)
        self.assertTrue((directory / "run.log").exists())
        self.assertTrue((retained / "run.log").exists())

    def test_report_compaction_refuses_symlinked_report_ancestor(self) -> None:
        resource, directory = self.report("root-ancestor")
        build = self.root / "build"
        external = self.root / "external-build"
        build.rename(external)
        build.symlink_to(external, target_is_directory=True)
        with self.assertRaisesRegex(resources.ResourceError, "ownership changed"):
            resources.reclaim(resource)
        self.assertTrue((external / directory.name / "run.log").exists())

    def test_cleanup_skips_known_references_and_borrows(self) -> None:
        reports = sorted(self.report(f"protected-{index}") for index in range(5))
        with resources.resource_metadata() as ledger:
            for index, (resource, _directory) in enumerate(reports[:4]):
                if index % 2:
                    ledger["owners"][resource]["references"] = {"protected": "owner"}
                else:
                    ledger["owners"][resource]["borrows"] = {
                        "protected": {
                            "pid": os.getpid(),
                            "start": resources.operation.start_identity(os.getpid()),
                        }
                    }
        self.assertEqual(resources.reclaim_reports(), [])
        self.assertFalse((reports[4][1] / "run.log").exists())
        self.assertTrue(
            all(
                (directory / "run.log").exists() for _resource, directory in reports[:4]
            )
        )

    def test_bounded_cleanup_rotates_past_undrained_reports(self) -> None:
        reports = sorted(self.report(f"rotation-{index}") for index in range(5))
        with resources.resource_metadata() as ledger:
            for resource, _directory in reports[:4]:
                ledger["owners"][resource]["units"] = [{"unit": "retained.scope"}]
        with patch.object(resources.operation, "drained", return_value=False) as drain:
            self.assertEqual(resources.reclaim_reports(), [])
            self.assertEqual(drain.call_count, 4)
            self.assertTrue((reports[4][1] / "run.log").exists())
            self.assertEqual(resources.reclaim_reports(), [])
            self.assertFalse((reports[4][1] / "run.log").exists())
        self.assertTrue(
            all(
                (directory / "run.log").exists() for _resource, directory in reports[:4]
            )
        )

    def test_manifest_preserves_provenance_receipts_and_unknown_bytes(self) -> None:
        directory = self.root / "build" / "semantics"
        directory.mkdir(parents=True)
        for name in (
            "summary.md",
            "source.diff",
            "untracked-source.tar.gz",
            "raw.xml",
            "undeclared.log",
            "scratch.json",
        ):
            (directory / name).write_text(name)
        resource = resources.register_report(directory, root=self.root)
        roles = {
            "checks.json": "receipt",
            "summary.md": "receipt",
            "source.diff": "provenance",
            "untracked-source.tar.gz": "provenance",
            "raw.xml": "evidence",
            "scratch.json": "scratch",
        }
        self.finish_report(
            resource, {"complete": True, "required_checks_covered": True}, roles=roles
        )
        manifest = resources.resource_status(resource)["artifact_manifest"]
        self.assertEqual(manifest["version"], 1)
        self.assertEqual(manifest["entries"]["undeclared.log"]["role"], "unknown")
        self.assertEqual(resources.reclaim(resource), "removed")
        self.assertFalse((directory / "scratch.json").exists())
        for name in roles.keys() - {"scratch.json"} | {"undeclared.log"}:
            self.assertTrue((directory / name).exists(), name)

    def test_no_scratch_keeps_exact_evidence_without_hot_cleanup_work(self) -> None:
        directory = self.root / "build" / "all-retained"
        directory.mkdir(parents=True)
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource, {"complete": True, "required_checks_covered": True}
        )
        self.assertEqual(resources.reclaim(resource), "no disposable report artifacts")
        self.assertEqual(resources.resource_status(resource)["cleanup"], "compacted")
        self.assertNotIn(resource, resources.resource_status())
        self.assertEqual(resources.report_resource(directory), resource)
        self.assertTrue((directory / "checks.json").exists())

    def test_completed_reports_have_exact_lookup_and_live_reference_promotion(
        self,
    ) -> None:
        completed = []
        for index in range(24):
            directory = self.root / "build" / f"completed-{index}"
            directory.mkdir(parents=True)
            (directory / "run.log").write_text("disposable")
            resource = resources.register_report(directory, root=self.root)
            self.finish_report(
                resource, {"complete": True, "required_checks_covered": True}
            )
            self.assertEqual(resources.reclaim(resource), "removed")
            completed.append((resource, directory))
        self.assertEqual(resources.resource_status(), {})
        resource, directory = completed[7]
        self.assertEqual(resources.report_resource(directory), resource)
        with resources.borrow(resource):
            self.assertEqual(set(resources.resource_status()), {resource})
            self.assertTrue(resources.resource_status(resource)["borrows"])
        self.assertEqual(resources.resource_status(), {})
        resources.retain_reference(resource, "current-consumer", "a" * 64)
        self.assertEqual(set(resources.resource_status()), {resource})
        resources.release_reference(resource, "current-consumer", "a" * 64)
        self.assertEqual(resources.resource_status(), {})
        with patch.object(
            resources, "private_record", wraps=resources.private_record
        ) as read:
            self.assertEqual(resources.reclaim_reports(), [])
            self.assertEqual(resources.resource_status(), {})
            read.assert_not_called()
        self.assertTrue((directory / "checks.json").exists())
        self.assertFalse((directory / "run.log").exists())
        self.assertEqual(resources.reclaim(resource), "compacted")

    def test_missing_provenance_never_becomes_successful_disposal(self) -> None:
        for field, value in (
            ("input_coverage", False),
            ("source_unchanged", False),
            ("provenance_errors", ["missing snapshot"]),
        ):
            with self.subTest(field=field):
                directory = self.root / "build" / field
                directory.mkdir(parents=True)
                (directory / "run.log").write_text("scratch")
                resource = resources.register_report(directory, root=self.root)
                self.finish_report(
                    resource,
                    {"complete": True, "required_checks_covered": True, field: value},
                )
                self.assertEqual(
                    resources.resource_status(resource)["disposition"], "pass"
                )
                self.assertEqual(resources.reclaim(resource), "retention pin")
                resources.pin(resource, False)
                self.assertEqual(
                    resources.reclaim(resource), "unverified report provenance"
                )
                self.assertTrue((directory / "run.log").exists())

    def test_claimed_success_without_covered_qualified_scope_stays_failed(self) -> None:
        directory = self.root / "build" / "false-success"
        directory.mkdir(parents=True)
        (directory / "run.log").write_text("scratch")
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource, {"complete": True, "required_checks_covered": True, "checks": []}
        )
        self.assertEqual(resources.resource_status(resource)["disposition"], "failure")
        self.assertEqual(resources.reclaim(resource), "retention pin")

    def test_claimed_provenance_without_required_snapshot_stays_protected(self) -> None:
        directory = self.root / "build" / "missing-required-provenance"
        directory.mkdir(parents=True)
        (directory / "run.log").write_text("scratch")
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource,
            {"complete": True, "required_checks_covered": True},
            required_provenance=["missing-source.snapshot"],
        )
        self.assertFalse(resources.resource_status(resource)["provenance_verified"])
        resources.pin(resource, False)
        self.assertEqual(resources.reclaim(resource), "unverified report provenance")
        self.assertTrue((directory / "run.log").exists())

    def test_manifest_cannot_be_recomputed_to_downgrade_retained_bytes(self) -> None:
        resource, directory = self.report("sealed")
        original = resources.resource_status(resource)["artifact_manifest"]
        receipt = json.loads((directory / "checks.json").read_text())
        with self.assertRaisesRegex(resources.ResourceError, "already sealed"):
            resources.finish_report(
                resource,
                receipt,
                roles={"checks.json": "receipt", "run.log": "scratch"},
                required_provenance=["fixture-source.json"],
            )
        self.assertEqual(
            resources.resource_status(resource)["artifact_manifest"], original
        )

    def test_changed_retained_provenance_prevents_any_scratch_removal(self) -> None:
        directory = self.root / "build" / "changed-provenance"
        directory.mkdir(parents=True)
        (directory / "run.log").write_text("scratch")
        (directory / "source.diff").write_text("authored source")
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource,
            {"complete": True, "required_checks_covered": True},
            roles={
                "checks.json": "receipt",
                "run.log": "scratch",
                "source.diff": "provenance",
            },
        )
        (directory / "source.diff").write_text("changed authored source")
        with self.assertRaisesRegex(resources.ResourceError, "ownership changed"):
            resources.reclaim(resource)
        self.assertTrue((directory / "run.log").exists())
        self.assertTrue(resources.resource_status(resource)["pin"])

    def test_same_byte_replacement_is_not_the_sealed_artifact(self) -> None:
        resource, directory = self.report("same-bytes")
        original = directory / "run.log"
        original.rename(directory / "held-original")
        original.write_text("owned output")
        with self.assertRaisesRegex(resources.ResourceError, "ownership changed"):
            resources.reclaim(resource)
        self.assertTrue(original.exists())

    def test_manifestless_record_stays_protected_after_manual_release(self) -> None:
        resource, directory = self.report("old-policy")
        with resources.resource_metadata() as ledger:
            del ledger["owners"][resource]["artifact_manifest"]
            del ledger["owners"][resource]["artifact_policy_version"]
        resources.pin(resource, False)
        self.assertEqual(
            resources.reclaim(resource), "missing producer artifact manifest"
        )
        self.assertTrue((directory / "run.log").exists())

    def test_registered_unsealed_report_cannot_supply_reused_evidence(self) -> None:
        directory = self.root / "build" / "active-origin"
        directory.mkdir(parents=True)
        (directory / "checks.json").write_text("{}")
        resource = resources.register_report(directory, root=self.root)
        with (
            self.assertRaisesRegex(resources.ResourceError, "sealed receipt"),
            resources.reference_report(directory, self.root / "consumer"),
        ):
            self.fail("unsealed origin was exposed")
        self.assertEqual(resources.resource_status(resource)["borrows"], {})
        self.assertEqual(resources.resource_status(resource)["references"], {})

    def test_consumed_evidence_cannot_be_declared_scratch(self) -> None:
        directory = self.root / "build" / "consumed"
        directory.mkdir(parents=True)
        (directory / "run.log").write_text("consumed log")
        resource = resources.register_report(directory, root=self.root)
        with self.assertRaisesRegex(resources.ResourceError, "consumed evidence"):
            self.finish_report(
                resource,
                {
                    "complete": True,
                    "required_checks_covered": True,
                    "checks": [
                        {
                            "gate": "fixture-control",
                            "status": "passed",
                            "log": "run.log",
                        }
                    ],
                },
            )
        self.assertTrue(resources.resource_status(resource)["pin"])

    def test_unknown_history_is_displayable_but_cannot_supply_reuse(self) -> None:
        directory = self.root / "build" / "unregistered-history"
        directory.mkdir(parents=True)
        (directory / "checks.json").write_text("{}")
        with resources.borrow_report(directory):
            self.assertEqual((directory / "checks.json").read_text(), "{}")
        with (
            self.assertRaisesRegex(resources.ResourceError, "sealed receipt"),
            resources.reference_report(directory, None),
        ):
            self.fail("unregistered origin was exposed")

    def test_registered_legacy_origin_cannot_bypass_sealed_reuse(self) -> None:
        resource, directory = self.report("legacy-reuse")
        with resources.resource_metadata() as ledger:
            del ledger["owners"][resource]["artifact_manifest"]
            del ledger["owners"][resource]["artifact_policy_version"]
        with resources.borrow_report(directory):
            self.assertTrue((directory / "checks.json").is_file())
        with (
            self.assertRaisesRegex(resources.ResourceError, "sealed receipt"),
            resources.reference_report(directory, None),
        ):
            self.fail("legacy origin was exposed")
        self.assertEqual(resources.resource_status(resource)["borrows"], {})
        self.assertEqual(resources.resource_status(resource)["references"], {})

    def test_interrupted_scratch_cleanup_resumes_without_losing_retained_files(
        self,
    ) -> None:
        directory = self.root / "build" / "interrupted-scratch"
        directory.mkdir(parents=True)
        for name in ("first.tmp", "second.tmp"):
            (directory / name).write_text(name)
        resource = resources.register_report(directory, root=self.root)
        self.finish_report(
            resource,
            {"complete": True, "required_checks_covered": True},
            roles={
                "checks.json": "receipt",
                "first.tmp": "scratch",
                "second.tmp": "scratch",
            },
        )
        unlink = resources.os.unlink

        def interrupted(name: str, *, dir_fd: int | None = None) -> None:
            unlink(name, dir_fd=dir_fd)
            raise KeyboardInterrupt

        with (
            patch.object(resources.os, "unlink", side_effect=interrupted),
            self.assertRaises(KeyboardInterrupt),
        ):
            resources.reclaim(resource)
        self.assertEqual(resources.resource_status(resource)["cleanup"], "removing")
        with patch.object(resources, "borrower_alive", return_value=False):
            self.assertEqual(resources.reclaim(resource), "removed")
        self.assertTrue((directory / "checks.json").exists())
        self.assertFalse((directory / "first.tmp").exists())
        self.assertFalse((directory / "second.tmp").exists())

    def test_legacy_report_without_directory_identity_remains_pinned(self) -> None:
        resource, directory = self.report("unknown-identity")
        with resources.resource_metadata() as ledger:
            del ledger["owners"][resource]["directory_identity"]
        with self.assertRaisesRegex(resources.ResourceError, "ownership changed"):
            resources.reclaim(resource)
        self.assertTrue((directory / "run.log").exists())
        self.assertTrue(resources.resource_status(resource)["pin"])


if __name__ == "__main__":
    unittest.main()
