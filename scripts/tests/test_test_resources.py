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
        resource = resources.register_report(directory, root=self.root)
        resources.finish_report(
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
        resources.finish_report(
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
        resources.finish_report(
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
            resources.finish_report(
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
        resources.finish_report(
            resource, {"complete": True, "required_checks_covered": True}
        )
        return resource, directory

    def test_report_compaction_refuses_symlinked_artifact_ancestor(self) -> None:
        resource, directory = self.report("ancestor")
        nested = directory / "logs"
        nested.mkdir()
        (directory / "run.log").rename(nested / "run.log")
        resources.finish_report(
            resource, {"complete": True, "required_checks_covered": True}
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
