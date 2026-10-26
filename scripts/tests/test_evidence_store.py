# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Service-only evidence credentials, activity lifetime and explicit native dispatch."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time
import unittest
import uuid
from pathlib import Path
from unittest.mock import patch

from scripts import evidence_store, resource_store, service_borrowers, surreal_server
from scripts.tests.server_receipt_fixture import fixture_build_context, fixture_receipt


class EvidenceStoreTests(unittest.TestCase):
    def setUp(self) -> None:
        fixture_build_context(self)
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.state = Path(temporary.name) / "service"
        self.state.mkdir(mode=0o700)
        self.config: dict[str, object] = {
            "owner": surreal_server.OWNER,
            "profile_version": 3,
            "instance_id": str(uuid.uuid4()),
            "port": 18240,
            "endpoint": "grpc://127.0.0.1:18240",
            "accepting_writes": True,
            "admission": "open",
            "server": {
                **fixture_receipt(),
                "surrealmx_version": surreal_server.surreal_source.ENGINE,
            },
        }
        resource_store.publish(self.state / "config.json", self.config)
        self.deadline = time.monotonic() + 10

    def prepare(self) -> dict[str, object]:
        return evidence_store.prepare(self.state, deadline=self.deadline)

    def test_prepare_is_private_idempotent_and_no_scientific_open(self) -> None:
        with (
            patch.object(
                surreal_server,
                "config_for",
                side_effect=AssertionError("scientific open"),
            ),
            patch.object(
                surreal_server,
                "register_context",
                side_effect=AssertionError("native receiver"),
            ),
        ):
            result = self.prepare()
            credentials = self.state / ".tooling/evidence/credentials.json"
            original = credentials.read_bytes()
            self.assertEqual(result, self.prepare())
            self.assertEqual(credentials.read_bytes(), original)
        self.assertEqual(credentials.stat().st_mode & 0o777, 0o600)
        self.assertNotIn("reader_password", result)
        self.assertEqual(result["namespace"], "pse_tooling")
        self.assertEqual(
            resource_store.read(credentials)["reader_username"], "pse_evidence_reader"
        )
        self.assertEqual(
            resource_store.read(credentials)["importer_username"],
            "pse_evidence_importer",
        )
        self.assertFalse((self.state / ".receivers").exists())

    def test_service_identity_change_does_not_overwrite_selection_or_credentials(
        self,
    ) -> None:
        self.prepare()
        target = self.state / ".tooling/evidence/config.json"
        original = target.read_bytes()
        self.config["instance_id"] = str(uuid.uuid4())
        resource_store.publish(self.state / "config.json", self.config)
        with self.assertRaisesRegex(ValueError, "differs"):
            self.prepare()
        self.assertEqual(target.read_bytes(), original)

    def test_explicit_reselection_preserves_credentials_and_database_boundary(
        self,
    ) -> None:
        self.prepare()
        credentials = self.state / ".tooling/evidence/credentials.json"
        original = credentials.read_bytes()
        self.config["instance_id"] = str(uuid.uuid4())
        resource_store.publish(self.state / "config.json", self.config)
        result = evidence_store.prepare(
            self.state, deadline=self.deadline, replace_selection=True
        )
        self.assertEqual(result["status"], "reselected")
        self.assertEqual(credentials.read_bytes(), original)
        evidence_store.selected_configuration(self.state, self.config)
        target = self.state / ".tooling/evidence/config.json"
        malformed = resource_store.read(target)
        malformed["database"] = "foreign"
        resource_store.publish(target, malformed)
        with self.assertRaisesRegex(ValueError, "ownership"):
            evidence_store.prepare(
                self.state, deadline=self.deadline, replace_selection=True
            )
        self.assertEqual(resource_store.read(target), malformed)

    def test_closed_admission_does_not_prepare_tooling(self) -> None:
        self.config["admission"] = "quiesced"
        resource_store.publish(self.state / "config.json", self.config)
        with self.assertRaisesRegex(surreal_server.SupervisorError, "closed"):
            self.prepare()
        self.assertFalse((self.state / ".tooling").exists())

    def test_active_tooling_blocks_service_drain_without_receiver(self) -> None:
        with (
            surreal_server.tooling_activity(self.state, deadline=self.deadline),
            self.assertRaisesRegex(surreal_server.SupervisorError, "live tooling"),
        ):
            surreal_server.require_tooling_drained(self.state)
        surreal_server.require_tooling_drained(self.state)
        self.assertEqual(list((self.state / ".tooling/.activities").iterdir()), [])

    def test_inherited_descriptor_protects_child_after_parent_activity_closes(
        self,
    ) -> None:
        with surreal_server.tooling_activity(self.state, deadline=self.deadline) as (
            _config,
            activity,
        ):
            child = subprocess.Popen(
                [sys.executable, "-c", "import time; time.sleep(0.3)"],
                pass_fds=(activity,),
            )
        try:
            with self.assertRaisesRegex(surreal_server.SupervisorError, "live tooling"):
                surreal_server.require_tooling_drained(self.state)
        finally:
            child.wait(timeout=5)
        surreal_server.require_tooling_drained(self.state)

    def test_supervisor_drain_observes_service_only_lifetime(self) -> None:
        with (
            surreal_server.tooling_activity(self.state, deadline=self.deadline),
            patch.object(service_borrowers, "obligations", return_value=[]),
            self.assertRaisesRegex(surreal_server.SupervisorError, "live tooling"),
        ):
            surreal_server.require_borrowers_drained(self.state)

    def test_unsafe_configuration_and_unknown_activity_refuse(self) -> None:
        self.prepare()
        credentials = self.state / ".tooling/evidence/credentials.json"
        credentials.chmod(0o644)
        with self.assertRaises(ValueError):
            self.prepare()
        credentials.chmod(0o600)
        (self.state / ".tooling/.activities/unknown").write_text("unknown")
        with self.assertRaisesRegex(surreal_server.SupervisorError, "Unknown"):
            surreal_server.require_tooling_drained(self.state)

    def test_native_dispatch_never_compiles_or_leaks_root_credentials(self) -> None:
        self.prepare()
        tool = self.state / "built-xtask"
        tool.write_text("#!/bin/sh\nexit 0\n")
        tool.chmod(0o700)
        with (
            patch.object(surreal_server, "owns_listener", return_value=True),
            patch.object(
                evidence_store.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 7),
            ) as run,
            patch.dict(
                os.environ,
                {"SURREAL_USER": "root-owner", "SURREAL_PASS": "root-secret"},
            ),
        ):
            self.assertEqual(
                evidence_store.run_native(
                    self.state, tool, ["query", "--limit", "3"], deadline=self.deadline
                ),
                7,
            )
        command = run.call_args.args[0]
        self.assertEqual(
            command[:4], [str(tool), "evidence", "--state", str(self.state)]
        )
        self.assertIn("--timeout-ms", command)
        self.assertNotIn("cargo", command)
        self.assertEqual(command[-3:], ["query", "--limit", "3"])
        self.assertNotIn("SURREAL_USER", run.call_args.kwargs["env"])
        self.assertNotIn("SURREAL_PASS", run.call_args.kwargs["env"])
        self.assertEqual(
            run.call_args.kwargs["env"]["PSE_TOOLING_SERVICE_INSTANCE_ID"],
            self.config["instance_id"],
        )
        self.assertEqual(
            int(run.call_args.kwargs["env"]["PSE_TOOLING_ACTIVITY_FD"]),
            run.call_args.kwargs["pass_fds"][0],
        )
        self.assertEqual(len(run.call_args.kwargs["pass_fds"]), 1)

    def test_foreign_or_unavailable_listener_refuses_before_native_launch(self) -> None:
        self.prepare()
        tool = self.state / "built-xtask"
        tool.write_text("#!/bin/sh\nexit 0\n")
        tool.chmod(0o700)
        with (
            patch.object(surreal_server, "owns_listener", return_value=False),
            patch.object(evidence_store.subprocess, "run") as run,
            self.assertRaisesRegex(ValueError, "listener ownership"),
        ):
            evidence_store.run_native(
                self.state, tool, ["init"], deadline=self.deadline
            )
        run.assert_not_called()

    def test_missing_tool_does_not_initialize_or_start_service(self) -> None:
        with self.assertRaises(OSError):
            evidence_store.run_native(
                self.state,
                self.state / "absent-xtask",
                ["status"],
                deadline=self.deadline,
            )
        self.assertFalse((self.state / ".tooling").exists())

    def test_unprepared_selection_does_not_initialize_on_status(self) -> None:
        with self.assertRaises(OSError):
            evidence_store.selected_configuration(self.state, self.config)
        self.assertFalse((self.state / ".tooling").exists())

    def test_query_default_bound_is_owned_by_native_tool(self) -> None:
        with patch.object(evidence_store, "run_native", return_value=0) as run:
            self.assertEqual(
                evidence_store.main(["--state", str(self.state), "query"]), 0
            )
        self.assertEqual(run.call_args.args[2], ["query"])

    def test_first_generation_page_has_no_continuation(self) -> None:
        with patch.object(evidence_store, "run_native", return_value=0) as run:
            self.assertEqual(
                evidence_store.main(["--state", str(self.state), "generations"]), 0
            )
        self.assertEqual(run.call_args.args[2], ["generations"])

    def test_explicit_query_and_generation_selection_are_forwarded_as_arguments(
        self,
    ) -> None:
        with patch.object(evidence_store, "run_native", return_value=0) as run:
            self.assertEqual(
                evidence_store.main(
                    [
                        "--state",
                        str(self.state),
                        "query",
                        "--limit",
                        "8",
                        "--status",
                        "failure",
                        "--generation",
                        "original_cut",
                    ]
                ),
                0,
            )
        self.assertEqual(
            run.call_args.args[2],
            [
                "query",
                "--limit",
                "8",
                "--status",
                "failure",
                "--generation",
                "original_cut",
            ],
        )
        with patch.object(evidence_store, "run_native", return_value=0) as run:
            self.assertEqual(
                evidence_store.main(
                    [
                        "--state",
                        str(self.state),
                        "generations",
                        "--limit",
                        "3",
                        "--after",
                        "original_cut",
                    ]
                ),
                0,
            )
        self.assertEqual(
            run.call_args.args[2],
            ["generations", "--limit", "3", "--after", "original_cut"],
        )


if __name__ == "__main__":
    unittest.main()
