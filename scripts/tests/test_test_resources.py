# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Terminal outcome, protected references and crash-resumable cleanup controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import fcntl
import json
import os
import tempfile
import threading
import unittest
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import surreal_server

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping, Sequence
from unittest.mock import patch

from scripts import test_resources as resources


def drain_fixture(complete: bool) -> resources.operation.DrainObservation:
    return resources.operation.DrainObservation(
        None,
        resources.operation.DrainPhase.COMPLETE
        if complete
        else resources.operation.DrainPhase.POPULATED,
        "controlled observation",
    )


from scripts import assessment_store
from scripts.tests import assessment_fixture

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
            "python",
            [{"name": "test_module.py::test_assert_after_exit"}],
            requirements={
                "test_module.py::test_assert_after_exit": {
                    "effects": ["canonical", "native"],
                    "fixtures": [],
                    "capacity": "canonical",
                    "roles": [],
                }
            },
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
            "version": 6,
            "input_coverage": True,
            "source_unchanged": True,
            "provenance_errors": [],
            "scope": [{"name": "fixture-control"}],
            "checks": [{"gate": "fixture-control", "status": "passed"}],
            **outcome,
        }
        assessment_fixture.publish(directory, receipt)
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
            {"endpoint": "grpc://127.0.0.1:2"},
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

        registration = resources.resource_store.FileStore.register

        def gated(
            store: resources.resource_store.FileStore,
            identity: Sequence[str],
            facts: dict,
            *,
            invocation: str | None,
            declaration: Mapping[str, object] | None = None,
            deadline: float | None = None,
        ) -> str:
            descriptor = os.open(self.state / ".supervisor.lock", os.O_RDWR)
            try:
                with self.assertRaises(BlockingIOError):
                    fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                os.close(descriptor)
            return registration(
                store,
                identity,
                facts,
                invocation=invocation,
                declaration=declaration,
                deadline=deadline,
            )

        with (
            patch.object(resources.resource_store.FileStore, "register", gated),
            patch.object(surreal_server, "start") as start,
        ):
            resource = self.register()
        start.assert_not_called()
        self.assertEqual(
            resources.resource_status(resource)["service_generation"], "owned-service"
        )

    def test_demand_startup_precedes_borrow_publication_and_service_gate(self) -> None:

        self.config.update(profile_version=3, parked=False)

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

        self.config.update(profile_version=3, parked=True)
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

    def test_reclaim_one_removal_clock_covers_slow_effect_and_metadata_settlement(
        self,
    ) -> None:
        resource = self.passed_resource()
        now = [100.0]

        def remove(
            record: resources.ResourceRecord, *, deadline: float | None = None
        ) -> None:
            self.assertEqual(record["cleanup"], "removing")
            self.assertEqual(
                deadline, 190.0, "retain the existing finite removal allowance"
            )
            assert deadline is not None
            now[0] += 6.0
            self.assertEqual(resources.host.remaining(deadline, 90), 84.0)

        with (
            patch.object(resources.time, "monotonic", side_effect=lambda: now[0]),
            patch.object(
                resources, "resource_update", wraps=resources.resource_update
            ) as updates,
            patch.object(resources, "remove_owned", side_effect=remove) as removal,
        ):
            self.assertEqual(resources.reclaim(resource), "removed")
            removal.assert_called_once()
            self.assertEqual(
                [call.kwargs["deadline"] for call in updates.call_args_list],
                [190.0, 190.0],
                "claim and final settlement consume the same effect clock",
            )
        entry = resources.storage().lookup(resource)
        self.assertEqual(entry.phase, "disposed")
        self.assertEqual(resources.reclaim(resource), "removed")

    def test_reclaim_preserves_explicit_deadline_through_effect_and_settlement(
        self,
    ) -> None:
        resource = self.passed_resource()
        now = [100.0]
        original_deadline = 105.0

        def remove(
            record: resources.ResourceRecord, *, deadline: float | None = None
        ) -> None:
            self.assertEqual(record["cleanup"], "removing")
            self.assertIs(deadline, original_deadline)
            now[0] += 6.0

        with (
            patch.object(resources.time, "monotonic", side_effect=lambda: now[0]),
            patch.object(
                resources, "resource_update", wraps=resources.resource_update
            ) as updates,
            patch.object(resources, "remove_owned", side_effect=remove) as removal,
            self.assertRaisesRegex(resources.host.AdmissionError, "deadline exhausted"),
        ):
            resources.reclaim(resource, deadline=original_deadline)
        removal.assert_called_once()
        self.assertEqual(len(updates.call_args_list), 2)
        for call in updates.call_args_list:
            self.assertIs(call.kwargs["deadline"], original_deadline)
        record = resources.resource_status(resource)
        self.assertEqual(record["cleanup"], "removing")
        self.assertTrue(
            record["pin"], "an expired original clock cannot acknowledge disposal"
        )

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
        def reference_arrives(
            state: Path,
            *,
            deadline: float | None = None,
            wait_for_lifecycle: bool = True,
        ) -> Generator[None, None, None]:
            with admission(
                state, deadline=deadline, wait_for_lifecycle=wait_for_lifecycle
            ):
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

        def remove(
            record: resources.ResourceRecord, *, deadline: float | None = None
        ) -> None:
            self.assertIsNotNone(deadline)
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
            with self.assertRaisesRegex(
                resources.ResourceError, "outside|requirement declarations"
            ):
                resources.publish_python_collection(
                    self.invocation, ["unselected::child"]
                )
        self.assertEqual(
            resources.read_invocation(self.invocation)["selected"], [{"name": names[0]}]
        )

    def test_collected_membership_does_not_republish_catalogue_or_requirements(self) -> None:
        image = resources.storage().invocation_snapshot(self.invocation.stem)
        descriptor = resources.storage().path("invocation-inputs", image["descriptor"])
        before = (descriptor.read_bytes(), descriptor.stat().st_mtime_ns)
        publish = resources.resource_store.publish
        writes = []
        def observe(path: Path, value: object) -> None:
            writes.append(path)
            publish(path, value)
        with patch.object(resources.resource_store, "publish", side_effect=observe):
            self.register()
        self.assertFalse(any(path.parent.name in {"invocation-inputs", "invocation-staging"} for path in writes))
        compact = resources.resource_store.read(resources.storage().path("invocation-state", self.invocation.stem))
        self.assertNotIn("declaration", compact)
        self.assertNotIn("requirements", compact)
        self.assertNotIn("selected", json.loads(self.invocation.read_text()))
        self.assertEqual(before, (descriptor.read_bytes(), descriptor.stat().st_mtime_ns))
        with self.assertRaisesRegex(resources.ResourceError, "immutable"), resources.storage().invocation_edit(self.invocation.stem) as state:
            state["requirements"] = {}

    def test_service_cohort_covers_other_invocation_and_retained_database_reader(self) -> None:
        first = self.register()
        resources.record_drain(first)
        second = resources.invocation("python", [{"name": "other::test"}])
        with patch.dict(os.environ, {resources.MARKER: str(second)}):
            with resources.borrow(first):
                with patch.object(resources.storage().__class__, "active_ids", side_effect=AssertionError("global scan")):
                    cohort = resources.service_resources(self.state)
                    self.assertIn(first, cohort)
                    with surreal_server.lifecycle_reservation(self.state), self.assertRaisesRegex(surreal_server.SupervisorError, "protected borrowers"):
                        surreal_server.require_borrowers_drained(self.state)
        # A foreign empty registry must not establish service quiet.
        with patch.object(resources, "registry", return_value=self.root / "foreign-registry"), self.assertRaisesRegex(resources.ResourceError, "coverage differs"):
            resources.service_resources(self.state)

    def test_projection_writes_only_membership_transitions_and_scratch_is_separate(self) -> None:
        resource, directory = self.report("indexed-scratch")
        store = resources.storage()
        self.assertIn(resource, store.access_ids("scratch-candidates"))
        with resources.borrow(resource):
            self.assertNotIn(resource, store.access_ids("scratch-candidates"))
        self.assertIn(resource, store.access_ids("scratch-candidates"))
        protected = self.root / "build" / "protected-only"
        protected.mkdir(parents=True)
        (protected / "run.log").write_text("protected evidence")
        sealed = resources.register_report(protected, root=self.root)
        self.finish_report(sealed, {"complete": True, "required_checks_covered": True}, roles={"checks.json": "receipt", "run.log": "evidence"})
        self.assertNotIn(sealed, store.access_ids("scratch-candidates"))
        publish = resources.resource_store.publish
        writes = []
        def observe(path: Path, value: object) -> None:
            writes.append(path)
            publish(path, value)
        with patch.object(resources.resource_store, "publish", side_effect=observe), resources.resource_update(sealed):
            pass
        self.assertFalse(any(path.parent.name == "active" for path in writes))
        with patch.object(store.__class__, "active_ids", side_effect=AssertionError("global scan")):
            self.assertEqual(resources.reclaim_reports(), [])
        self.assertFalse((directory / "run.log").exists())
        self.assertTrue((protected / "run.log").exists())

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

    def test_namespace_sql_preserves_exact_context_and_explicit_statement_errors(
        self,
    ) -> None:
        config = {"credentials_file": str(self.state / "credentials.json")}
        query = "// retained comment\nRETURN '// literal';"
        ok = {"status": "OK", "result": None}
        error = {"status": "ERR", "result": "fixture refusal"}
        with patch.object(
            surreal_server, "administrative_results", return_value=[ok]
        ) as transport:
            self.assertEqual(resources.namespace_sql(config, query), [None])
            self.assertEqual(transport.call_args.args[1], query)
            self.assertFalse(transport.call_args.kwargs["select_context"])
            self.assertIsNone(transport.call_args.kwargs["namespace"])
        ack = {"status": "OK", "result": {"namespace": "owned", "database": None}}
        with patch.object(
            surreal_server, "administrative_results", return_value=[ack, ok, error]
        ) as transport:
            self.assertEqual(
                resources.namespace_sql(config, query, namespace="owned"),
                [None, "fixture refusal"],
            )
            self.assertEqual(transport.call_args.args[1], "USE NS `owned`;\n" + query)
            self.assertFalse(transport.call_args.kwargs["select_context"])
            self.assertIsNone(transport.call_args.kwargs["namespace"])
        with patch.object(
            surreal_server, "administrative_results", return_value=[ok]
        ) as transport:
            resources.namespace_sql(
                config, query, namespace="owned", database="fixture"
            )
            self.assertEqual(transport.call_args.args[1], query)
            self.assertEqual(transport.call_args.kwargs["namespace"], "owned")
            self.assertEqual(transport.call_args.kwargs["database"], "fixture")
            self.assertTrue(transport.call_args.kwargs["select_context"])

    def test_namespace_only_requires_its_exact_selection_acknowledgment(self) -> None:
        config = {"credentials_file": str(self.state / "credentials.json")}
        for acknowledgment in (
            {"namespace": "other", "database": None},
            {"namespace": "owned", "database": "implicit"},
            None,
        ):
            with (
                patch.object(
                    surreal_server,
                    "administrative_results",
                    return_value=[
                        {"status": "OK", "result": acknowledgment},
                        {"status": "OK", "result": None},
                    ],
                ),
                self.assertRaises(resources.ResourceError),
            ):
                resources.namespace_sql(config, "RETURN 1;", namespace="owned")
        with (
            patch.object(surreal_server, "administrative_results") as transport,
            self.assertRaises(resources.ResourceError),
        ):
            resources.namespace_sql(config, "RETURN 1;", database="fixture")
        transport.assert_not_called()

    def test_cleanup_requires_exact_complete_root_acknowledgment(self) -> None:

        credentials = self.root / "credentials.json"
        credentials.write_text('{"username":"control","password":"not-a-live-secret"}')
        config = {
            "credentials_file": str(credentials),
            "server": {"binary": "/unused/control-cli"},
            "endpoint": "grpc://127.0.0.1:1",
            "namespace": "pse",
        }
        record = {"state": str(self.state), "database": "canonical_test_" + "a" * 32}
        for values in ([None, None], ["query failed"], []):
            with (
                self.subTest(values=values),
                patch.object(resources, "namespace_sql", return_value=values),
                self.assertRaises(resources.ResourceError),
            ):
                resources.remove_database(record, config)
        with patch.object(resources, "namespace_sql", return_value=[None]):
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
        with patch.object(
            resources.operation, "drained", return_value=drain_fixture(False)
        ):
            self.assertEqual(
                resources.reclaim(resource),
                "supervised descendant not drained: "
                + drain_fixture(False).diagnostic(),
            )
        with (
            patch.object(
                resources.operation, "drained", return_value=drain_fixture(True)
            ),
            patch.object(resources, "remove_owned") as removal,
        ):
            self.assertEqual(resources.reclaim(resource), "removed")
            self.assertEqual(resources.reclaim(resource), "removed")
            removal.assert_called_once()

    def test_accepted_creation_borrower_survives_freeze_and_late_settlement(
        self,
    ) -> None:
        resource = self.register()
        before = resources.resource_status(resource)["creation_borrower"]
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "pass"},
            complete=True,
            terminal_owner="runner",
        )
        self.assertEqual(
            resources.resource_status(resource)["creation_borrower"], before
        )
        self.assertEqual(resources.reclaim(resource), "fixture borrowers not drained")
        with (
            patch.object(resources, "borrower_alive", return_value=False),
            self.assertRaisesRegex(
                surreal_server.SupervisorError, "accepted fixture creation"
            ),
        ):
            surreal_server.require_borrowers_drained(self.state)
        with self.assertRaisesRegex(
            resources.ResourceError, "Invocation membership is closed"
        ):
            resources.register(
                {
                    "state": str(self.state),
                    "database": "canonical_test_" + "b" * 32,
                    "test": "test_module.py::test_assert_after_exit",
                }
            )
        resources.record_drain(resource)
        self.assertNotIn("creation_borrower", resources.resource_status(resource))
        with patch.object(resources, "remove_owned") as remove:
            self.assertEqual(resources.reclaim(resource), "removed")
            remove.assert_called_once()

    def test_creation_effect_already_accepted_settles_after_freeze(self) -> None:
        entered, release = threading.Event(), threading.Event()
        result, errors = [], []

        def create(*_args: object) -> None:
            entered.set()
            if not release.wait(2):
                raise TimeoutError("test effect release")

        def register() -> None:
            try:
                result.append(
                    resources.register(
                        {
                            "state": str(self.state),
                            "database": "canonical_test_" + "b" * 32,
                            "test": "test_module.py::test_assert_after_exit",
                            "execution_profile": "exclusive-observer",
                        }
                    )
                )
            except Exception as error:
                errors.append(error)

        with patch.object(surreal_server, "register_context", side_effect=create):
            thread = threading.Thread(target=register)
            thread.start()
            try:
                self.assertTrue(entered.wait(2))
                accepted = next(iter(resources.resource_status()))
                resources.finalize(
                    self.invocation,
                    {"test_module.py::test_assert_after_exit": "pass"},
                    complete=True,
                    terminal_owner="runner",
                )
                self.assertIn("creation_borrower", resources.resource_status(accepted))
                self.assertEqual(
                    resources.reclaim(accepted), "fixture borrowers not drained"
                )
            finally:
                release.set()
                thread.join(2)
        self.assertEqual(errors, [])
        self.assertEqual(result, [accepted])
        resources.record_drain(accepted)
        self.assertNotIn("creation_borrower", resources.resource_status(accepted))

    def test_partial_projection_resumes_same_terminal_and_preserves_manual_pin(
        self,
    ) -> None:
        first = self.register()
        second = resources.register(
            {
                "state": str(self.state),
                "database": "canonical_test_" + "b" * 32,
                "test": "test_module.py::test_assert_after_exit",
            }
        )
        saved = resources.resource_store.FileStore._save  # noqa: SLF001 -- inject projection publication failure
        projected = []

        def fail_second(
            store: resources.resource_store.FileStore,
            entry: resources.resource_store.Entry,
        ) -> None:
            if entry.facts.get("terminal_identity"):
                if projected:
                    raise OSError("projection interrupted")
                projected.append(entry.resource)
            saved(store, entry)

        with (
            patch.object(resources.resource_store.FileStore, "_save", new=fail_second),
            self.assertRaisesRegex(OSError, "projection interrupted"),
        ):
            resources.finalize(
                self.invocation,
                {"test_module.py::test_assert_after_exit": "pass"},
                complete=True,
                terminal_owner="runner",
                terminal={"workload_exit": 0},
            )
        self.assertEqual(len(projected), 1)
        resources.pin(projected[0], True)
        resources.resume_projection(self.invocation, terminal_owner="runner")
        self.assertTrue(resources.resource_status(projected[0])["pin"])
        for resource in (first, second):
            self.assertEqual(resources.resource_status(resource)["disposition"], "pass")
            self.assertIn("creation_borrower", resources.resource_status(resource))
        with self.assertRaisesRegex(resources.ResourceError, "another immutable"):
            resources.finalize(
                self.invocation,
                {"test_module.py::test_assert_after_exit": "failure"},
                complete=True,
                terminal_owner="runner",
                terminal={"workload_exit": 1},
            )

    def projection_members(self, count: int) -> tuple[Path, list[str]]:
        path = resources.invocation(
            "python",
            [{"name": f"projection.py::point_{index}"} for index in range(count)],
            requirements={
                f"projection.py::point_{index}": {
                    "effects": ["canonical"],
                    "fixtures": [],
                    "capacity": "canonical",
                    "roles": [],
                }
                for index in range(count)
            },
        )
        with patch.dict(os.environ, {resources.MARKER: str(path)}):
            members = [
                resources.register(
                    {
                        "state": str(self.state),
                        "database": f"canonical_test_projection_{index}_{path.stem}",
                        "test": f"projection.py::point_{index}",
                    }
                )
                for index in range(count)
            ]
        return path, members

    def test_finite_projection_operations_preserve_mixed_retention_beyond_one_clock(
        self,
    ) -> None:
        path, members = self.projection_members(8)
        outcomes = {
            f"projection.py::point_{index}": ("pass", "failure", "incomplete")[
                index % 3
            ]
            for index in range(len(members))
        }
        resources.record_drain(members[0])
        resources.record_drain(members[6])
        resources.pin(members[3], True)
        now = [100.0]
        saved = resources.resource_store.FileStore._save  # noqa: SLF001 -- observe exact durable projection work

        def save(
            store: resources.resource_store.FileStore,
            entry: resources.resource_store.Entry,
        ) -> None:
            saved(store, entry)
            now[0] += 2

        with (
            resources.borrow(members[6]) as borrower,
            patch.object(
                resources.resource_store.time, "monotonic", side_effect=lambda: now[0]
            ),
            patch.object(resources.resource_store.FileStore, "_save", new=save),
        ):
            eligible = resources.finalize(
                path, outcomes, complete=True, terminal_owner="runner"
            )
            self.assertGreater(now[0] - 100, 5)
            self.assertEqual(set(eligible), {members[0], members[6]})
            for index, resource in enumerate(members):
                record = resources.resource_status(resource)
                self.assertEqual(
                    record["disposition"], outcomes[f"projection.py::point_{index}"]
                )
                self.assertEqual(record["pin"], index % 3 != 0 or index == 3)
                if index not in {0, 6}:
                    self.assertIn("creation_borrower", record)
                    self.assertFalse(record["drained"])
            self.assertIn(borrower, resources.resource_status(members[6])["borrows"])
            with patch.object(resources, "remove_owned") as remove:
                self.assertEqual(
                    resources.reclaim(members[6]),
                    "protected reference or active borrow",
                )
                remove.assert_not_called()
        state = resources.resource_store.read(
            resources.storage().path("invocation-state", path.stem)
        )
        self.assertEqual(state["projection"], "complete")
        self.assertNotIn("terminal", state)
        self.assertEqual(resources.storage().invocation_snapshot(path.stem)["terminal"]["outcomes"], outcomes)

    def test_resume_settles_long_completed_prefix_without_replacing_it(self) -> None:
        path, members = self.projection_members(14)
        saved = resources.resource_store.FileStore._save  # noqa: SLF001 -- interrupt exact projection publication
        last = max(members)

        def fail_last(
            store: resources.resource_store.FileStore,
            entry: resources.resource_store.Entry,
        ) -> None:
            if entry.resource == last:
                raise OSError("last projection interrupted")
            saved(store, entry)

        with (
            patch.object(resources.resource_store.FileStore, "_save", new=fail_last),
            self.assertRaisesRegex(OSError, "last projection interrupted"),
        ):
            resources.finalize(
                path,
                {
                    f"projection.py::point_{index}": "pass"
                    for index in range(len(members))
                },
                complete=True,
                terminal_owner="runner",
            )
        prefix = [resource for resource in members if resource != last]
        resources.pin(prefix[0], True)
        store = resources.storage()
        before = {
            resource: (
                store.path("resources", resource).stat().st_ino,
                store.path("resources", resource).stat().st_mtime_ns,
            )
            for resource in prefix
        }
        settled = resources.resource_store.FileStore._settle_unchanged  # noqa: SLF001 -- observe resumed durability settlement
        now = [100.0]

        def settle(
            selected: resources.resource_store.FileStore,
            entry: resources.resource_store.Entry,
            deadline: float,
        ) -> bool:
            result = settled(selected, entry, deadline)
            now[0] += 0.6
            return result

        with (
            patch.object(
                resources.resource_store.time, "monotonic", side_effect=lambda: now[0]
            ),
            patch.object(
                resources.resource_store.FileStore, "_settle_unchanged", new=settle
            ),
            patch.object(
                resources.resource_store.FileStore,
                "_save",
                autospec=True,
                side_effect=saved,
            ) as writes,
        ):
            resources.resume_projection(path, terminal_owner="runner")
        self.assertGreater(now[0] - 100, 5)
        self.assertEqual(
            [call.args[1].resource for call in writes.call_args_list], [last]
        )
        for resource in prefix:
            stat = store.path("resources", resource).stat()
            self.assertEqual((stat.st_ino, stat.st_mtime_ns), before[resource])
        self.assertTrue(resources.resource_status(prefix[0])["pin"])
        self.assertEqual(resources.resource_status(last)["disposition"], "pass")

    def test_projection_keeps_explicit_and_newly_observed_deadlines(self) -> None:
        for dynamic in (False, True):
            with self.subTest(dynamic=dynamic):
                self.assert_projection_deadline_case(dynamic)

    def assert_projection_deadline_case(self, dynamic: bool) -> None:
        path, members = self.projection_members(5)
        now = [100.0]
        observed: list[float | None] = [None]
        saved = resources.resource_store.FileStore._save  # noqa: SLF001 -- account for controlled projection work

        def save(
            store: resources.resource_store.FileStore,
            entry: resources.resource_store.Entry,
        ) -> None:
            saved(store, entry)
            now[0] += 3

        def frozen() -> None:
            if dynamic:
                observed[0] = 105.0

        def deadline() -> float | None:
            for family, key in [
                ("invocations", path.stem),
                *[("resources", resource) for resource in members],
            ]:
                descriptor = os.open(
                    resources.storage().root / "locks" / family / key,
                    os.O_RDONLY | os.O_NOFOLLOW,
                )
                try:
                    fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
                finally:
                    os.close(descriptor)
            # A later owner observation cannot renew the first clock.
            return observed[0] if now[0] <= 100 else 200.0

        with (
            patch.object(
                resources.resource_store.time, "monotonic", side_effect=lambda: now[0]
            ),
            patch.object(resources.resource_store.FileStore, "_save", new=save),
            self.assertRaisesRegex(resources.host.AdmissionError, "deadline exhausted"),
        ):
            resources.finalize(
                path,
                {
                    f"projection.py::point_{index}": "pass"
                    for index in range(len(members))
                },
                complete=True,
                terminal_owner="runner",
                deadline=None if dynamic else 105.0,
                deadline_supplier=deadline if dynamic else None,
                on_frozen=frozen,
            )
        state = resources.resource_store.read(
            resources.storage().path("invocation-state", path.stem)
        )
        self.assertEqual(state["phase"], "frozen")
        self.assertNotIn("projection", state)
        projected = sum(
            "terminal_identity" in resources.resource_status(resource)
            for resource in members
        )
        self.assertGreater(projected, 0)
        self.assertLess(projected, len(members))

    def test_projection_resume_refuses_missing_or_contradictory_authority(self) -> None:
        for contradiction in ("missing", "identity", "disposition"):
            with self.subTest(contradiction=contradiction):
                path, members = self.projection_members(1)
                resources.finalize(
                    path,
                    {"projection.py::point_0": "pass"},
                    complete=True,
                    terminal_owner="runner",
                )
                authority = resources.storage().path("resources", members[0])
                if contradiction == "missing":
                    authority.unlink()
                else:
                    record = resources.resource_store.read(authority)
                    record["facts"][
                        "terminal_identity"
                        if contradiction == "identity"
                        else "disposition"
                    ] = "b" * 64 if contradiction == "identity" else "failure"
                    resources.resource_store.publish(authority, record)
                with self.assertRaises((FileNotFoundError, resources.ResourceError)):
                    resources.resume_projection(path, terminal_owner="runner")

    def test_collection_between_terminal_read_and_freeze_refuses_stale_composition(
        self,
    ) -> None:
        path = resources.invocation("python", [])
        freeze = resources.resource_store.FileStore.freeze

        def collect_then_freeze(
            store: resources.resource_store.FileStore,
            nonce: str,
            terminal: dict,
            *,
            declaration: Mapping[str, object] | None = None,
            deadline: float | None = None,
            on_frozen: Callable[[], None] | None = None,
        ) -> list[str]:
            resources.publish_python_requirements(
                path,
                {
                    "collected.py::test_case": {
                        "effects": [],
                        "fixtures": [],
                        "capacity": "local",
                        "roles": [],
                    }
                },
            )
            resources.publish_python_collection(path, ["collected.py::test_case"])
            return freeze(
                store,
                nonce,
                terminal,
                declaration=declaration,
                deadline=deadline,
                on_frozen=on_frozen,
            )

        with (
            patch.object(
                resources.resource_store.FileStore, "freeze", new=collect_then_freeze
            ),
            self.assertRaisesRegex(
                resources.ResourceError, "terminal selection authority"
            ),
        ):
            resources.finalize(path, {}, complete=False, terminal_owner="runner")
        nonce = resources.read_invocation(path)["nonce"]
        state = resources.resource_store.read(
            resources.storage().path("invocation-state", nonce)
        )
        self.assertEqual(state["phase"], "open")
        self.assertNotIn("terminal", state)
        resources.finalize(path, {}, complete=False, terminal_owner="runner")
        self.assertEqual(resources.resume_projection(path, terminal_owner="runner"), [])

    def test_missing_authority_with_dangling_access_paths_never_proves_disposal(
        self,
    ) -> None:
        resource = self.register()
        store = resources.storage()
        identity = store.lookup(resource).identity
        store.path("resources", resource).unlink()
        self.assertTrue(store.path("active", resource).exists())
        self.assertTrue(
            store.path("identities", resources.resource_store.digest(identity)).exists()
        )
        with (
            patch.object(resources, "remove_owned") as remove,
            self.assertRaisesRegex(
                resources.ResourceError, "disposal remains unproved"
            ),
        ):
            resources.reclaim(resource)
        remove.assert_not_called()

    def test_closed_service_defers_cleanup_until_same_generation_reopens(self) -> None:
        resource = self.register()
        resources.record_drain(resource)
        resources.finalize(
            self.invocation,
            {"test_module.py::test_assert_after_exit": "pass"},
            complete=True,
            terminal_owner="runner",
        )
        before = resources.resource_status(resource)
        with patch.object(resources, "remove_owned") as removal:
            for admission, accepting in (("quiesced", False), ("open", False)):
                with self.subTest(admission=admission, accepting=accepting):
                    self.config.update(admission=admission, accepting_writes=accepting)
                    self.assertEqual(
                        resources.reclaim(resource),
                        "service context admission is closed",
                    )
                    self.assertEqual(resources.resource_status(resource), before)
                    removal.assert_not_called()
            self.config.update(admission="open", accepting_writes=True)
            self.assertEqual(resources.reclaim(resource), "removed")
            removal.assert_called_once()

    def test_service_closing_during_cleanup_admission_still_refuses_removal(
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
        before = resources.resource_status(resource)
        admission = surreal_server.context_admission

        @contextlib.contextmanager
        def close_before_admission(
            state: Path,
            *,
            deadline: float | None = None,
            wait_for_lifecycle: bool = True,
        ) -> Generator[None, None, None]:
            self.config.update(admission="quiesced", accepting_writes=False)
            with admission(
                state, deadline=deadline, wait_for_lifecycle=wait_for_lifecycle
            ):
                yield

        with (
            patch.object(surreal_server, "context_admission", close_before_admission),
            patch.object(resources, "remove_owned") as removal,
            self.assertRaisesRegex(
                surreal_server.SupervisorError, "admission is closed"
            ),
        ):
            resources.reclaim(resource)
        removal.assert_not_called()
        self.assertEqual(resources.resource_status(resource), before)

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

        def removal(
            _record: resources.ResourceRecord, *, deadline: float | None = None
        ) -> None:
            self.assertIsNotNone(deadline)
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

    def test_report_borrow_without_overall_deadline_survives_long_read(self) -> None:
        for retained in (False, True):
            with self.subTest(retained=retained):
                resource, directory = self.report(f"long-read-{retained}")
                now = [100.0]
                consumer = self.root / "consumer"
                guard = (
                    resources.reference_report(directory, consumer)
                    if retained
                    else resources.borrow_report(directory)
                )
                with patch.object(
                    resources.time, "monotonic", side_effect=lambda now=now: now[0]
                ):
                    with guard:
                        now[0] += 6.0
                        self.assertTrue(resources.resource_status(resource)["borrows"])
                    record = resources.resource_status(resource)
                self.assertEqual(record["borrows"], {})
                self.assertEqual(bool(record["references"]), retained)

    def test_report_borrow_preserves_explicit_deadline_after_long_read(self) -> None:
        for retained in (False, True):
            with self.subTest(retained=retained):
                resource, directory = self.report(f"bounded-read-{retained}")
                now = [100.0]
                consumer = self.root / "consumer"
                guard = (
                    resources.reference_report(directory, consumer, deadline=105.0)
                    if retained
                    else resources.borrow_report(directory, deadline=105.0)
                )
                with (
                    patch.object(
                        resources.time, "monotonic", side_effect=lambda now=now: now[0]
                    ),
                    self.assertRaisesRegex(
                        resources.host.AdmissionError, "deadline exhausted"
                    ),
                    guard,
                ):
                    now[0] += 6.0
                record = resources.resource_status(resource)
                self.assertTrue(record["borrows"])
                self.assertEqual(bool(record["references"]), retained)

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
        with patch.object(
            resources.operation, "drained", return_value=drain_fixture(False)
        ):
            self.assertEqual(resources.reclaim_reports(), [])
            self.assertTrue((directory / "run.log").exists())
        with patch.object(
            resources.operation, "drained", return_value=drain_fixture(True)
        ):
            self.assertEqual(resources.reclaim_reports(), [])
            self.assertFalse((directory / "run.log").exists())
        self.assertTrue((directory / "checks.json").exists())

    def test_cleanup_sweep_retains_rejected_service_and_continues(self) -> None:
        first, _ = self.report("retired-service")
        second, _ = self.report("next-report")
        with patch.object(
            resources,
            "reclaim",
            side_effect=[
                surreal_server.SupervisorError("retired service profile"),
                "removed",
            ],
        ) as reclaim:
            errors = resources.reclaim_reports()
        self.assertEqual(reclaim.call_count, 2)
        self.assertEqual(len(errors), 1)
        self.assertIn("retired service profile", errors[0])
        self.assertEqual(
            {call.args[0] for call in reclaim.call_args_list}, {first, second}
        )
        self.assertNotEqual(resources.resource_status(first).get("cleanup"), "removed")

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
        for index, (resource, _directory) in enumerate(reports[:4]):
            with resources.resource_update(resource) as record:
                if index % 2:
                    record["references"] = {"protected": "owner"}
                else:
                    record["borrows"] = {
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
        created = dict(self.report(f"rotation-{index}") for index in range(5))
        reports = [(resource, created[resource]) for resource in resources.storage().access_ids("scratch-candidates")]
        for resource, _directory in reports[:4]:
            with resources.resource_update(resource) as record:
                record["units"] = [{"unit": "retained.scope"}]
        with patch.object(
            resources.operation, "drained", return_value=drain_fixture(False)
        ) as drain:
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
            resources.resource_store.FileStore,
            "_entry",
            side_effect=AssertionError("cold history was read"),
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
        receipt = assessment_store.export(directory)
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
        with resources.resource_update(resource) as record:
            del record["artifact_manifest"]
            del record["artifact_policy_version"]
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
        with resources.resource_update(resource) as record:
            del record["artifact_manifest"]
            del record["artifact_policy_version"]
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
        with resources.resource_update(resource) as record:
            del record["directory_identity"]
        with self.assertRaisesRegex(resources.ResourceError, "ownership changed"):
            resources.reclaim(resource)
        self.assertTrue((directory / "run.log").exists())
        self.assertTrue(resources.resource_status(resource)["pin"])


if __name__ == "__main__":
    unittest.main()
