# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Recovery selection and actual reference placement acceptance controls."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import surreal_server as server
from scripts.tests import canonical_recovery_check as recovery
from scripts.tests import canonical_recovery_science as science


class ScientificRecoveryControls(unittest.TestCase):
    def test_retained_science_detects_content_changes_with_unchanged_keys(self) -> None:
        import pyarrow as pa  # noqa: PLC0415

        manifest = pa.table(
            {
                "key": ["attempt"],
                "digest": ["original-digest"],
                "descriptors": [b"exact-descriptors"],
            }
        )
        variables = pa.table({"symbol_id": [b"original-symbol"], "value": [2.0]})
        checks = pa.table(
            {"key": ["original-check"], "satisfied": [True], "value": [2.0]}
        )
        saved = science.result_snapshot(manifest, variables, checks)
        self.assertEqual(saved["manifest_digest"], "original-digest")
        science.check_retained_snapshot(
            saved, science.result_snapshot(manifest, variables, checks)
        )
        mutations = (
            (
                pa.table(
                    {
                        "key": ["attempt"],
                        "digest": ["changed-digest"],
                        "descriptors": [b"exact-descriptors"],
                    }
                ),
                variables,
                checks,
            ),
            (
                pa.table(
                    {
                        "key": ["attempt"],
                        "digest": ["original-digest"],
                        "descriptors": [b"changed-descriptors"],
                    }
                ),
                variables,
                checks,
            ),
            (
                manifest,
                pa.table({"symbol_id": [b"original-symbol"], "value": [3.0]}),
                checks,
            ),
            (
                manifest,
                variables,
                pa.table(
                    {"key": ["original-check"], "satisfied": [True], "value": [3.0]}
                ),
            ),
            (
                manifest,
                variables,
                pa.table(
                    {"key": ["original-check"], "satisfied": [False], "value": [2.0]}
                ),
            ),
        )
        for changed in mutations:
            with (
                self.subTest(snapshot=changed),
                self.assertRaisesRegex(
                    RuntimeError, "retained result contents changed"
                ),
            ):
                science.check_retained_snapshot(
                    saved, science.result_snapshot(*changed)
                )

    def test_retained_science_compares_float_bits_and_ignores_transport_chunking(
        self,
    ) -> None:
        import pyarrow as pa  # noqa: PLC0415

        manifest = pa.table({"key": ["attempt"], "digest": ["digest"]})
        checks = pa.table({"satisfied": [True]})
        positive = pa.table({"value": [0.0, 2.0]})
        negative = pa.table({"value": [-0.0, 2.0]})
        saved = science.result_snapshot(manifest, positive, checks)
        with self.assertRaisesRegex(RuntimeError, "retained result contents changed"):
            science.check_retained_snapshot(
                saved, science.result_snapshot(manifest, negative, checks)
            )
        paged = pa.concat_tables([positive.slice(0, 1), positive.slice(1, 1)])
        science.check_retained_snapshot(
            saved, science.result_snapshot(manifest, paged, checks)
        )

    def test_cli_forwards_science_profile_and_retained_evidence_directory(self) -> None:
        with (
            patch(
                "sys.argv",
                [
                    "recovery",
                    "binary",
                    "--scientific",
                    "--profile-state",
                    "profile",
                    "--directory",
                    "evidence",
                ],
            ),
            patch.object(recovery, "journey") as journey,
        ):
            recovery.main()
        journey.assert_called_once_with(
            Path("binary"),
            Path("profile"),
            scientific=True,
            output_directory=Path("evidence"),
        )

    def test_preserved_inventory_contains_actual_authored_and_physical_inputs(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            inputs = Path(directory) / "inputs"
            inventory = science.prepare_inputs(inputs)
            self.assertIn("model/models/length.pse", inventory)
            self.assertIn("physical/materials/physical.yaml", inventory)
            self.assertIn("physical/package.toml", inventory)
            self.assertEqual(
                science.documents(inputs / "model")["models/length.pse"], science.SOURCE
            )
            self.assertEqual(server.input_inventory(inputs), inventory)
            (inputs / "model/models/length.pse").write_text(science.SOURCE + "\n")
            self.assertNotEqual(server.input_inventory(inputs), inventory)

    def test_science_child_uses_explicit_phase_inputs_and_exact_receipt(self) -> None:
        command = recovery.science_command(
            Path("state"), Path("preserved/inputs"), Path("receipt.json"), "rebuild"
        )
        self.assertEqual(
            command[1:4], ["-m", "scripts.tests.canonical_recovery_science", "--state"]
        )
        self.assertEqual(
            command[4:],
            [
                "state",
                "--inputs",
                "preserved/inputs",
                "--receipt",
                "receipt.json",
                "--phase",
                "rebuild",
            ],
        )

    def test_science_refuses_unspecified_or_lower_profile_before_setup(self) -> None:
        with patch.object(server, "setup") as setup:
            with self.assertRaisesRegex(server.SupervisorError, "explicit original"):
                recovery.journey(Path("unused"), scientific=True)
            setup.assert_not_called()

    def test_original_native_and_physical_acceptance_refuses_missing_or_false_evidence(
        self,
    ) -> None:
        science.check_solution(True, "ipopt", [2.0], [True], 1e-6)
        for usable, backend, values, checks in (
            (False, "ipopt", [2.0], [True]),
            (True, "other", [2.0], [True]),
            (True, "ipopt", [2.0], []),
            (True, "ipopt", [2.0], [False]),
            (True, "ipopt", [], [True]),
            (True, "ipopt", [float("nan")], [True]),
            (True, "ipopt", [float("inf")], [True]),
            (True, "ipopt", [3.0], [True]),
        ):
            with (
                self.subTest(
                    usable=usable, backend=backend, values=values, checks=checks
                ),
                self.assertRaisesRegex(RuntimeError, "scientific acceptance"),
            ):
                science.check_solution(usable, backend, values, checks, 1e-6)

    def test_old_credentials_need_positive_current_authentication_around_refusal(
        self,
    ) -> None:
        with (
            patch.object(server, "config_for", return_value={}),
            patch.object(server, "lifecycle_reservation"),
            patch.object(server, "private_offline_state"),
            patch.object(server, "read_json", return_value={"username": "fresh"}),
            patch.object(
                server,
                "maintenance_query",
                side_effect=[{}, server.SupervisorError("authentication refused"), {}],
            ) as query,
        ):
            recovery.refuse_old_credentials(Path("state"), {"username": "stale"})
            self.assertEqual(query.call_count, 3)
        with (
            patch.object(server, "config_for", return_value={}),
            patch.object(server, "lifecycle_reservation"),
            patch.object(server, "private_offline_state"),
            patch.object(server, "read_json", return_value={"username": "fresh"}),
            patch.object(
                server,
                "maintenance_query",
                side_effect=server.SupervisorError("unavailable"),
            ),
            self.assertRaisesRegex(server.SupervisorError, "unavailable"),
        ):
            recovery.refuse_old_credentials(Path("state"), {"username": "stale"})

    def test_successful_stale_authentication_fails_acceptance(self) -> None:
        with (
            patch.object(server, "config_for", return_value={}),
            patch.object(server, "lifecycle_reservation"),
            patch.object(server, "private_offline_state"),
            patch.object(server, "read_json", return_value={"username": "fresh"}),
            patch.object(server, "maintenance_query", return_value={}),
            self.assertRaisesRegex(
                server.SupervisorError, "Original root credentials admitted"
            ),
        ):
            recovery.refuse_old_credentials(Path("state"), {"username": "stale"})


class CanonicalRecoveryProfileTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.state = self.root / "state"
        self.worker = self.root / "worker"
        self.worker.write_bytes(b"selected receiver fixture")
        self.worker.chmod(0o700)

    def initialized(self, reference: bool) -> dict:
        arguments = (
            [
                "--execution-profile",
                "plan28-reference",
                "--worker-executable",
                str(self.worker),
            ]
            if reference
            else recovery.profile_arguments(None)
        )
        with (
            patch.object(
                server,
                "install",
                return_value={
                    "version": "3.3.0",
                    "binary": "/unused",
                    "archive_sha256": "fixture",
                },
            ),
            patch.object(server, "public_status", return_value={}),
        ):
            server.setup(
                server.parser().parse_args(
                    [
                        "setup",
                        "--state",
                        str(self.state),
                        "--interpretation",
                        server.SUBSTRATE_INTERPRETATION,
                        *arguments,
                    ]
                )
            )
        config = server.config_for(self.state)
        config["admission"] = "quiesced"
        config["accepting_writes"] = False
        server.write_json(self.state / "config.json", config)
        return config

    def test_reference_selection_carries_actual_receiver_without_numeric_overrides(
        self,
    ) -> None:
        config = self.initialized(True)
        arguments = recovery.profile_arguments(self.state)
        selected = server.parser().parse_args(["setup", *arguments])
        self.assertEqual(selected.execution_profile, "plan28-reference")
        self.assertEqual(
            str(selected.worker_executable),
            config["primary_receiver"]["worker_executable"],
        )
        self.assertEqual(selected.version, "v3.3.0")
        self.assertTrue(
            all(
                value is None
                for value in (
                    selected.memory_mib,
                    selected.server_memory_mib,
                    selected.native_workers,
                    selected.native_worker_memory_mib,
                )
            )
        )
        Path(config["primary_receiver"]["worker_executable"]).write_bytes(
            b"changed receiver bytes"
        )
        with self.assertRaisesRegex(
            server.SupervisorError, "Immutable supervisor/receiver closure changed"
        ):
            recovery.profile_arguments(self.state)

    def test_numeric_profile_selection_retains_legacy_allocations_and_version(
        self,
    ) -> None:
        config = self.initialized(False)
        selected = server.parser().parse_args(
            ["setup", *recovery.profile_arguments(self.state)]
        )
        self.assertIsNone(selected.execution_profile)
        self.assertIsNone(selected.worker_executable)
        self.assertEqual(selected.version, "v3.3.0")
        allocation = server.resources(
            selected.memory_mib * server.MIB,
            selected.server_memory_mib * server.MIB,
            selected.native_workers,
            selected.native_worker_memory_mib * server.MIB,
        )
        self.assertEqual(allocation, config["resources"])

    def test_reduced_reference_recovery_profile_is_refused_without_mutation(
        self,
    ) -> None:
        config = self.initialized(True)
        saved = (self.state / "config.json").read_bytes()
        credentials = (self.state / "credentials.json").read_bytes()
        with (
            server.state_lock(self.state),
            patch.object(
                server,
                "reconfigure",
                side_effect=AssertionError("recovery control mutated allocation"),
            ),
            patch.object(
                recovery,
                "reference_recovery_receiver",
                wraps=recovery.reference_recovery_receiver,
            ) as validate,
        ):
            recovery.refuse_reduced_recovery_profile(self.state, config, 8192)
        validate.assert_called_once()
        self.assertEqual(
            validate.call_args.args[0]["resources"]["server_memory_bytes"],
            8192 * server.MIB,
        )
        self.assertEqual(config["resources"], server.reference_resources())
        self.assertEqual((self.state / "config.json").read_bytes(), saved)
        self.assertEqual((self.state / "credentials.json").read_bytes(), credentials)

    def test_reference_refusal_rejects_mutating_error_paths(self) -> None:
        config = self.initialized(True)

        def mutate_then_refuse(*_args: object) -> None:
            (self.state / "credentials.json").write_bytes(b"changed credentials")
            raise server.SupervisorError(
                "Recovery requires the original exact reference allocation"
            )

        with (
            patch.object(
                recovery, "reference_recovery_receiver", side_effect=mutate_then_refuse
            ),
            self.assertRaisesRegex(
                server.SupervisorError, "mutated recovery state or credentials"
            ),
        ):
            recovery.refuse_reduced_recovery_profile(self.state, config, 8192)

    def test_legacy_reconfiguration_still_accepts_lower_then_original_cap(self) -> None:
        config = self.initialized(False)
        identity = recovery.recovery_identity(config)
        credentials = (self.state / "credentials.json").read_bytes()
        with patch.object(
            server,
            "systemctl",
            return_value=subprocess.CompletedProcess(
                [],
                0,
                "LoadState=not-found\nActiveState=inactive\nControlGroup=\nMemoryMax=infinity\n",
                "",
            ),
        ):
            for mib in (512, 1024):
                with server.state_lock(self.state):
                    server.reconfigure(
                        self.state,
                        config,
                        server.parser().parse_args(
                            [
                                "reconfigure",
                                "--state",
                                str(self.state),
                                "--server-memory-mib",
                                str(mib),
                            ]
                        ),
                    )
                self.assertEqual(
                    config["resources"]["server_memory_bytes"], mib * server.MIB
                )
                self.assertEqual(recovery.recovery_identity(config), identity)
                self.assertEqual(
                    (self.state / "credentials.json").read_bytes(), credentials
                )
                self.assertFalse(config["accepting_writes"])


class CanonicalRecoveryPlacementTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.state = self.root / "state"
        self.selected = self.root / "reference.slice"
        self.actual = self.selected / "server.service"
        self.actual.mkdir(parents=True)
        (self.actual / "memory.max").write_text(str(16 * server.GIB))
        (self.actual / "cgroup.controllers").write_text("memory pids\n")
        (self.selected / "memory.max").write_text(str(160 * server.GIB))
        (self.selected / "cpu.max").write_text("1600000 100000\n")
        self.enterContext(
            patch.object(server, "group_for_slice", return_value=self.selected)
        )

    def verify(self, actual: Path | None = None) -> None:
        recovery.verify_server_placement(
            self.state,
            server.reference_resources(),
            str((actual or self.actual).relative_to(self.root)),
            self.root,
        )

    def test_reference_accepts_actual_inherited_quota_and_exact_memory_caps(
        self,
    ) -> None:
        self.assertFalse((self.actual / "cpu.max").exists())
        self.verify()

    def test_reference_refuses_wrong_selected_limits_or_narrower_ancestor(self) -> None:
        for field, value in (
            ("cpu.max", "800000 100000\n"),
            ("memory.max", str(159 * server.GIB)),
            ("cpu.max", "max 100000\n"),
        ):
            path = self.selected / field
            original = path.read_text()
            path.write_text(value)
            with (
                self.subTest(field=field, value=value),
                self.assertRaisesRegex(
                    server.SupervisorError, "ancestor memory/CPU enforcement"
                ),
            ):
                self.verify()
            path.write_text(original)
        (self.root / "cpu.max").write_text("800000 100000\n")
        with self.assertRaisesRegex(
            server.SupervisorError, "ancestor memory/CPU enforcement"
        ):
            self.verify()

    def test_reference_refuses_a_service_outside_its_selected_ancestor(self) -> None:
        other = self.root / "unrelated.service"
        other.mkdir()
        (other / "memory.max").write_text(str(16 * server.GIB))
        with self.assertRaisesRegex(
            server.SupervisorError, "outside its selected execution ancestor"
        ):
            self.verify(other)

    def test_reference_refuses_wrong_actual_server_cap(self) -> None:
        (self.actual / "memory.max").write_text(str(8 * server.GIB))
        with self.assertRaisesRegex(server.SupervisorError, "kernel MemoryMax differs"):
            self.verify()


if __name__ == "__main__":
    unittest.main()
