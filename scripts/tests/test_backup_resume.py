# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The original lifecycle/configuration owner resumes one exact stopped backup."""

from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import backup_image
from scripts import surreal_server as server


class BackupResumeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.state = self.root / "source"
        self.state.mkdir(mode=0o700)
        (self.state / "database").mkdir(mode=0o700)
        for name, value in (
            ("database/data", b"stopped database image"),
            ("credentials.json", b"private credentials"),
        ):
            path = self.state / name
            path.write_bytes(value)
            path.chmod(0o600)
        self.config = {
            "instance_id": "original-service",
            "interpretation": "canonical-v10",
            "accepting_writes": False,
            "admission": "quiesced",
            "payload_frozen": True,
        }
        server.write_json(self.state / "config.json", self.config)
        self.destination = self.root / "backup"
        self.enterContext(
            patch.object(
                server,
                "lifecycle_reservation",
                side_effect=lambda *_args, **_options: contextlib.nullcontext(),
            )
        )
        self.enterContext(
            patch.object(server.service_borrowers, "service_registry_binding")
        )
        self.enterContext(
            patch.object(
                server,
                "config_for",
                side_effect=lambda path, **_options: server.read_json(
                    path / "config.json"
                ),
            )
        )
        self.enterContext(patch.object(server, "require_private_service_context"))
        self.enterContext(patch.object(server, "all_contexts_drained"))
        self.enterContext(patch.object(server, "active", return_value=False))
        self.enterContext(patch.object(server, "stop"))
        self.enterContext(patch.object(server, "owned_generations", return_value=set()))
        self.freeze = self.enterContext(
            patch.object(server, "freeze_payloads", return_value=[])
        )
        self.capture = self.enterContext(
            patch.object(
                server,
                "capture_application_inventory",
                return_value={"semantic": "original"},
            )
        )
        self.helper = self.root / "original-helper"
        self.helper.write_bytes(b"#!/bin/sh\nexit 0\n")
        self.helper.chmod(0o700)
        self.enterContext(
            patch.object(
                server,
                "admitted_payload_verifier",
                return_value=server.payload_verifier(self.helper),
            )
        )
        self.enterContext(
            patch.object(
                server,
                "verify_payload_inventory",
                side_effect=AssertionError(
                    "duplicate immediate inventory verification"
                ),
            )
        )
        self.payload = self.enterContext(
            patch.object(
                server,
                "copy_payload_inventory",
                side_effect=server.SupervisorError("interrupted native copy"),
            )
        )

    def backup(self, destination: Path | None = None) -> dict[str, object]:
        return server.backup(
            self.state,
            server.read_json(self.state / "config.json"),
            destination or self.destination,
        )

    def interrupted(self) -> dict[str, object]:
        with self.assertRaisesRegex(server.SupervisorError, "interrupted native"):
            self.backup()
        return server.read_json(self.state / "config.json")

    def explicit_helper(self, command: str, helper: Path) -> int:
        arguments = [
            command,
            "--state",
            str(self.state),
            "--payload-verifier",
            str(helper),
        ]
        if command == "backup":
            arguments += ["--destination", str(self.destination), "--drained"]
        with contextlib.redirect_stdout(io.StringIO()):
            return server.dispatch(server.parser().parse_args(arguments))

    def test_backup_cli_different_helper_path_refuses_without_changing_original_config(
        self,
    ) -> None:
        self.interrupted()
        original = (self.state / "config.json").read_bytes()
        other = self.root / "same-bytes-other-helper"
        other.write_bytes(self.helper.read_bytes())
        other.chmod(0o700)
        with self.assertRaisesRegex(
            server.SupervisorError, "original exact native payload helper"
        ):
            self.explicit_helper("backup", other)
        self.assertEqual((self.state / "config.json").read_bytes(), original)
        self.payload.side_effect = None
        self.backup()
        self.assertTrue((self.destination / "backup.json").is_file())
        self.freeze.assert_called_once()

    def test_maintenance_recovery_different_helper_refuses_before_configuration_or_cutover(
        self,
    ) -> None:
        self.interrupted()
        original = (self.state / "config.json").read_bytes()
        other = self.root / "different-helper"
        other.write_bytes(b"#!/bin/sh\nexit 2\n")
        other.chmod(0o700)
        with (
            patch.object(server, "maintenance_payloads") as effects,
            self.assertRaisesRegex(
                server.SupervisorError, "original exact native payload helper"
            ),
        ):
            self.explicit_helper("recover-maintenance", other)
        effects.assert_not_called()
        self.assertEqual((self.state / "config.json").read_bytes(), original)
        self.payload.side_effect = None
        self.backup()
        self.assertTrue((self.destination / "backup.json").is_file())

    def test_same_helper_path_changed_bytes_refuse_without_rewriting_pending_configuration(
        self,
    ) -> None:
        self.interrupted()
        original = (self.state / "config.json").read_bytes()
        original_helper = self.helper.read_bytes()
        self.helper.write_bytes(b"#!/bin/sh\nexit 3\n")
        for command in ("backup", "recover-maintenance"):
            with (
                self.subTest(command=command),
                self.assertRaisesRegex(
                    server.SupervisorError, "original exact native payload helper"
                ),
            ):
                self.explicit_helper(command, self.helper)
            self.assertEqual((self.state / "config.json").read_bytes(), original)
            self.assertEqual(self.helper.read_bytes(), b"#!/bin/sh\nexit 3\n")
        # Restore only this test's deliberately changed executable, then exercise
        # the original unchanged backup request. Production never repairs it.
        self.helper.write_bytes(original_helper)
        self.payload.side_effect = None
        self.backup()
        self.assertTrue((self.destination / "backup.json").is_file())

    def test_equal_helper_selection_does_not_write_before_original_backup_resume(
        self,
    ) -> None:
        self.interrupted()
        original = (self.state / "config.json").read_bytes()
        resume = server.backup

        def original_resume(
            state: Path, config: dict[str, object], destination: Path
        ) -> dict[str, object]:
            self.assertEqual((state / "config.json").read_bytes(), original)
            return resume(state, config, destination)

        self.payload.side_effect = None
        with patch.object(server, "backup", side_effect=original_resume):
            self.assertEqual(self.explicit_helper("backup", self.helper), 0)
        self.assertNotIn("backup_pending", server.read_json(self.state / "config.json"))
        self.assertTrue((self.destination / "backup.json").is_file())
        self.freeze.assert_called_once()

    def test_equal_helper_never_grants_maintenance_cutover_or_changes_original_backup(
        self,
    ) -> None:
        self.interrupted()
        original = (self.state / "config.json").read_bytes()
        with (
            patch.object(server, "maintenance_payloads") as effects,
            self.assertRaisesRegex(server.SupervisorError, "original backup resume"),
        ):
            self.explicit_helper("recover-maintenance", self.helper)
        effects.assert_not_called()
        self.assertEqual((self.state / "config.json").read_bytes(), original)
        self.payload.side_effect = None
        self.backup()

    def test_equal_helper_cannot_bypass_malformed_pending_or_changed_source_identity(
        self,
    ) -> None:
        interrupted = self.interrupted()
        original = (self.state / "config.json").read_bytes()
        pending = server.object_mapping(interrupted["backup_pending"])
        for invalid in (
            None,
            {},
            {**pending, "version": 2},
            {**pending, "unknown": True},
        ):
            selected = {**interrupted, "backup_pending": invalid}
            server.write_json(self.state / "config.json", selected)
            retained = (self.state / "config.json").read_bytes()
            for command in ("backup", "recover-maintenance"):
                with (
                    self.subTest(command=command, pending=invalid),
                    self.assertRaises(server.SupervisorError),
                ):
                    self.explicit_helper(command, self.helper)
                self.assertEqual((self.state / "config.json").read_bytes(), retained)
        for change in (
            {"instance_id": "replacement"},
            {"resources": {"changed": "allocation"}},
        ):
            server.write_json(self.state / "config.json", {**interrupted, **change})
            retained = (self.state / "config.json").read_bytes()
            for command in ("backup", "recover-maintenance"):
                with (
                    self.subTest(command=command, change=change),
                    self.assertRaisesRegex(
                        server.SupervisorError, "source configuration differs"
                    ),
                ):
                    self.explicit_helper(command, self.helper)
                self.assertEqual((self.state / "config.json").read_bytes(), retained)
        # Test-only fixture restoration demonstrates that neither refusal rewrote
        # the original request or its archive. It is not a production repair path.
        (self.state / "config.json").write_bytes(original)
        self.payload.side_effect = None
        self.backup()
        self.assertTrue((self.destination / "backup.json").is_file())

    def test_normal_command_resumes_original_snapshot_without_thaw_or_recapture(
        self,
    ) -> None:
        interrupted = self.interrupted()
        original = (self.destination / "database/data").stat().st_ino
        self.assertEqual(interrupted["backup_pending"]["phase"], "payload")
        self.payload.side_effect = None
        self.backup()
        selected = server.read_json(self.state / "config.json")
        self.assertNotIn("backup_pending", selected)
        self.assertFalse(selected["accepting_writes"])
        self.assertTrue(selected["payload_frozen"])
        self.assertEqual((self.destination / "database/data").stat().st_ino, original)
        self.freeze.assert_called_once()
        self.capture.assert_called_once()
        manifest = server.read_json(self.destination / "backup.json")
        self.assertEqual(manifest["application_inventory"], {"semantic": "original"})
        self.assertNotIn(
            "backup_pending", server.read_json(self.destination / "config.json")
        )

    def test_partial_metadata_phase_is_resumable_without_named_partial_files(
        self,
    ) -> None:
        original = backup_image.copy
        interrupted = False

        def copy(source: Path, target: Path, expected: str) -> None:
            nonlocal interrupted
            if target == self.destination / "database/data" and not interrupted:
                interrupted = True
                raise InterruptedError("metadata process interruption")
            original(source, target, expected)

        with (
            patch.object(server.backup_image, "copy", side_effect=copy),
            self.assertRaises(InterruptedError),
        ):
            self.backup()
        self.assertFalse((self.destination / "database/data").exists())
        selected = server.read_json(self.state / "config.json")
        self.assertEqual(selected["backup_pending"]["phase"], "metadata")
        self.payload.side_effect = None
        self.backup()
        self.assertTrue((self.destination / "backup.json").is_file())
        self.freeze.assert_called_once()

    def test_source_snapshot_changes_refuse_and_pending_owner_survives(self) -> None:
        self.interrupted()
        (self.state / "database/data").write_bytes(b"changed frozen source")
        with self.assertRaisesRegex(server.SupervisorError, "source image differs"):
            self.backup()
        self.assertIn("backup_pending", server.read_json(self.state / "config.json"))

    def test_unknown_destination_content_is_retained(self) -> None:
        self.interrupted()
        unknown = self.destination / "unrelated"
        unknown.write_bytes(b"retain")
        unknown.chmod(0o600)
        with self.assertRaisesRegex(server.SupervisorError, "Unknown or changed"):
            self.backup()
        self.assertEqual(unknown.read_bytes(), b"retain")
        self.assertFalse((self.destination / "backup.json").exists())

    def test_replaced_destination_directory_and_other_targets_are_not_adopted(
        self,
    ) -> None:
        self.interrupted()
        with self.assertRaisesRegex(server.SupervisorError, "destination differs"):
            self.backup(self.root / "other")
        self.destination.rename(self.root / "original-destination")
        self.destination.mkdir(mode=0o700)
        with self.assertRaisesRegex(server.SupervisorError, "destination differs"):
            self.backup()
        self.assertEqual(list(self.destination.iterdir()), [])

    def test_pending_owner_blocks_start_readmission_reconfiguration_and_cutover(
        self,
    ) -> None:
        selected = self.interrupted()
        for operation in (
            lambda: server.start(self.state, selected),
            lambda: server._readmit_supervisor(self.state),
            lambda: server.reconfigure(
                self.state, selected, server.argparse.Namespace()
            ),
            lambda: server.private_offline_state(self.state, selected),
        ):
            with self.assertRaisesRegex(server.SupervisorError, "[Bb]ackup|backup"):
                operation()
        self.assertIn("backup_pending", server.read_json(self.state / "config.json"))

    def test_unrelated_nonempty_destination_refuses_before_freezing_source(
        self,
    ) -> None:
        self.destination.mkdir(mode=0o700)
        (self.destination / "unrelated").write_bytes(b"retain")
        with self.assertRaises(server.SupervisorError):
            self.backup()
        self.freeze.assert_not_called()
        self.assertEqual((self.destination / "unrelated").read_bytes(), b"retain")


if __name__ == "__main__":
    unittest.main()
