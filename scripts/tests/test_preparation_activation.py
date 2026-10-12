# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Preservation, interruption and exact-resume controls for the offline owner."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import preparation_activation as activation
from scripts import surreal_server as server


class PreparationActivationTests(unittest.TestCase):
    def test_frozen_binding_compares_bytes_and_never_replaces_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "binding.json"
            activation.frozen_json(path, {"original": "identity"})
            identity = path.stat().st_ino
            activation.frozen_json(path, {"original": "identity"})
            self.assertEqual(path.stat().st_ino, identity)
            with self.assertRaisesRegex(server.SupervisorError, "binding differs"):
                activation.frozen_json(path, {"original": "changed"})
            self.assertEqual(server.read_json(path), {"original": "identity"})

    def test_interrupted_manifest_publication_retains_orphan_and_can_resume(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "binding.json"
            with (
                patch.object(
                    activation.os, "fsync", side_effect=OSError("interrupted")
                ),
                self.assertRaisesRegex(OSError, "interrupted"),
            ):
                activation.frozen_json(path, {"original": "identity"})
            self.assertFalse(path.exists())
            orphans = list(path.parent.glob(".activation-pending-*"))
            self.assertEqual(len(orphans), 1)
            raw = orphans[0].read_bytes()
            activation.frozen_json(path, {"original": "identity"})
            self.assertEqual(orphans[0].read_bytes(), raw)
            self.assertEqual(server.read_json(path), {"original": "identity"})

    def test_backup_verification_refuses_changed_missing_and_added_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            backup = archive / "backup"
            backup.mkdir()
            source = backup / "payload"
            source.write_bytes(b"original")
            server.write_json(
                backup / "backup.json",
                {"files": {"payload": server.file_digest(source)}},
            )
            binding = {"backup_digest": server.file_digest(backup / "backup.json")}
            activation.verify_backup(archive, binding)
            source.write_bytes(b"changed")
            with self.assertRaisesRegex(server.SupervisorError, "backup bytes"):
                activation.verify_backup(archive, binding)
            source.write_bytes(b"original")
            source.unlink()
            with self.assertRaisesRegex(server.SupervisorError, "inventory changed"):
                activation.verify_backup(archive, binding)
            source.write_bytes(b"original")
            (backup / "extra").write_bytes(b"unknown")
            with self.assertRaisesRegex(server.SupervisorError, "inventory changed"):
                activation.verify_backup(archive, binding)

    def test_backup_symlink_never_substitutes_for_original_material(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            backup = archive / "backup"
            backup.mkdir()
            source = archive / "outside"
            source.write_bytes(b"original")
            (backup / "payload").symlink_to(source)
            server.write_json(
                backup / "backup.json",
                {"files": {"payload": server.file_digest(source)}},
            )
            with self.assertRaisesRegex(server.SupervisorError, "backup bytes"):
                activation.verify_backup(
                    archive,
                    {"backup_digest": server.file_digest(backup / "backup.json")},
                )

    def test_history_page_owner_checks_actual_hashes_and_freezes_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            pages = archive / "native-history"
            pages.mkdir()
            page = pages / "source-00000000.json"
            page.write_bytes(b"original-scientific-payload")
            descriptor = {
                "name": page.name,
                "bytes": page.stat().st_size,
                "sha256": server.file_digest(page),
                "digest": "native-semantic-digest",
                "rows": 1,
            }
            manifest = {"pages": [descriptor], "digest": "native-manifest-digest"}
            server.write_json(archive / "native-snapshot.json", {"history": manifest})
            activation.verify_history_archive(archive)
            owner = archive / "native-history-source-owner.json"
            original = owner.read_bytes()
            activation.verify_history_archive(archive)
            self.assertEqual(owner.read_bytes(), original)
            page.write_bytes(b"changed--scientific-payload")
            with self.assertRaisesRegex(server.SupervisorError, "page bytes changed"):
                activation.verify_history_archive(archive)
            descriptor["sha256"] = server.file_digest(page)
            descriptor["bytes"] = page.stat().st_size
            server.write_json(archive / "native-snapshot.json", {"history": manifest})
            with self.assertRaisesRegex(server.SupervisorError, "binding differs"):
                activation.verify_history_archive(archive)
            self.assertEqual(owner.read_bytes(), original)

    def test_history_page_missing_symlink_traversal_and_oversized_refuse(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            pages = archive / "native-history"
            pages.mkdir()
            payload = archive / "original"
            payload.write_bytes(b"unchanged")
            descriptor: dict[str, object] = {
                "name": "source-00000000.json",
                "bytes": 9,
                "sha256": server.file_digest(payload),
                "digest": "native",
                "rows": 1,
            }
            manifest = {"pages": [descriptor], "digest": "native"}
            snapshot = archive / "native-snapshot.json"
            server.write_json(snapshot, {"history": manifest})
            with self.assertRaisesRegex(server.SupervisorError, "page bytes changed"):
                activation.verify_history_archive(archive)
            (pages / "source-00000000.json").symlink_to(payload)
            with self.assertRaisesRegex(server.SupervisorError, "page bytes changed"):
                activation.verify_history_archive(archive)
            (archive / "native-history-source-owner.json").unlink()
            descriptor["name"] = "../original"
            server.write_json(snapshot, {"history": manifest})
            with self.assertRaisesRegex(server.SupervisorError, "identity or bound"):
                activation.verify_history_archive(archive)
            (archive / "native-history-source-owner.json").unlink()
            descriptor.update(name="source-00000000.json", bytes=4194305)
            server.write_json(snapshot, {"history": manifest})
            with self.assertRaisesRegex(server.SupervisorError, "identity or bound"):
                activation.verify_history_archive(archive)
            self.assertEqual(payload.read_bytes(), b"unchanged")

    def test_history_derived_page_is_checked_before_any_native_effect(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            pages = archive / "native-history"
            pages.mkdir()
            page = pages / "derive-00000000.json"
            page.write_bytes(b"typed-locators")
            manifest = {
                "pages": [
                    {
                        "name": page.name,
                        "bytes": page.stat().st_size,
                        "sha256": server.file_digest(page),
                        "digest": "native",
                        "rows": 1,
                    }
                ],
                "digest": "native",
            }
            server.write_json(archive / "native-history-derived.json", manifest)
            activation.verify_history_archive(archive)
            page.unlink()
            with self.assertRaisesRegex(server.SupervisorError, "page bytes changed"):
                activation.verify_history_archive(archive)

    def test_archive_authentication_work_scales_with_pages_not_rpc_count(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            pages = archive / "native-history"
            pages.mkdir()
            backup = archive / "backup"
            backup.mkdir()
            (backup / "original").write_bytes(b"preserved physical backup")
            server.write_json(
                backup / "backup.json",
                {"files": {"original": server.file_digest(backup / "original")}},
            )
            bridge = archive / "bridge"
            bridge.write_bytes(b"executable" * 100000)
            bridge.chmod(0o700)
            (archive / "credentials.json").write_bytes(b"private")
            config: dict[str, object] = {
                "instance_id": "fixture",
                "namespace": "ns",
                "database": "db",
                "endpoint": "local",
                "server": {},
                "resources": {},
            }
            physical = activation.physical_binding(archive, config, bridge)
            bridge_identity = activation.bridge_file_identity(bridge)
            binding = {
                "backup_digest": server.file_digest(backup / "backup.json"),
                "physical": physical,
            }
            # Reuse one bounded buffer while building a genuinely paged archive.
            payload = b"x" * 262145
            descriptors = []
            for ordinal in range(64):
                page = pages / f"source-{ordinal:08}.json"
                page.write_bytes(payload)
                descriptors.append(
                    {
                        "name": page.name,
                        "bytes": len(payload),
                        "sha256": server.file_digest(page),
                        "digest": "native",
                        "rows": 1,
                    }
                )
            server.write_json(
                archive / "native-snapshot.json",
                {"history": {"pages": descriptors, "digest": "native"}},
            )
            actual_digest = server.file_digest
            hashed: list[Path] = []

            def counted_digest(path: Path) -> str:
                hashed.append(path)
                return actual_digest(path)

            with patch.object(server, "file_digest", side_effect=counted_digest):
                activation.authenticate_archive_boundary(
                    archive,
                    binding,
                    {"event": "authorize", "phase": "archive:source"},
                    bridge,
                )
                for step in range(5000):
                    activation.authenticate_archive_boundary(
                        archive,
                        binding,
                        {
                            "event": "authorize",
                            "step": step,
                            "phase": "readback:canonical_result_rows",
                        },
                    )
                    self.assertEqual(
                        activation.bridge_file_identity(bridge), bridge_identity
                    )
                    self.assertEqual(
                        activation.physical_binding(
                            archive,
                            config,
                            bridge,
                            bridge_digest=str(physical["bridge_digest"]),
                        ),
                        physical,
                    )
                self.assertEqual(sum(path.parent == pages for path in hashed), 64)
                self.assertEqual(hashed.count(bridge), 1)
                activation.authenticate_archive_boundary(
                    archive,
                    binding,
                    {"event": "authorize", "phase": "archive:final"},
                    bridge,
                )
                self.assertEqual(sum(path.parent == pages for path in hashed), 128)
                self.assertEqual(hashed.count(bridge), 2)
            # A change between phase boundaries must prevent marker authorization.
            (pages / "source-00000063.json").write_bytes(b"y" + payload[1:])
            with self.assertRaisesRegex(server.SupervisorError, "page bytes changed"):
                activation.authenticate_archive_boundary(
                    archive, binding, {"event": "authorize", "phase": "archive:final"}
                )
            (pages / "source-00000063.json").write_bytes(payload)
            (backup / "original").write_bytes(b"changed physical backup")
            with self.assertRaisesRegex(server.SupervisorError, "backup bytes changed"):
                activation.authenticate_archive_boundary(
                    archive, binding, {"event": "authorize", "phase": "archive:final"}
                )

    def test_verified_bridge_detects_in_place_write_with_restored_mtime_and_replacement(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            bridge = Path(directory) / "bridge"
            bridge.write_bytes(b"original-executable")
            bridge.chmod(0o700)
            original = activation.bridge_file_identity(bridge)
            original_stat = bridge.stat()
            bridge.write_bytes(b"modified-executable")
            os.utime(bridge, ns=(original_stat.st_atime_ns, original_stat.st_mtime_ns))
            modified = activation.bridge_file_identity(bridge)
            self.assertEqual(modified[:5], original[:5])
            self.assertNotEqual(modified[5], original[5])
            replacement = bridge.with_name("replacement")
            replacement.write_bytes(b"modified-executable")
            replacement.chmod(0o700)
            os.utime(
                replacement, ns=(original_stat.st_atime_ns, original_stat.st_mtime_ns)
            )
            replacement.replace(bridge)
            self.assertNotEqual(activation.bridge_file_identity(bridge)[1], modified[1])
            bridge.unlink()
            bridge.symlink_to("replacement")
            with self.assertRaisesRegex(
                server.SupervisorError, "bridge identity changed"
            ):
                activation.bridge_file_identity(bridge)

    def test_borrower_snapshot_includes_retained_evidence_and_excludes_other_owners(
        self,
    ) -> None:
        selected = Path("/selected")
        records = {
            "retained-export": {
                "kind": "evidence",
                "state": str(selected),
                "drained": True,
            },
            "receiver": {"state": str(selected / ".receivers" / "one")},
            "other": {"state": "/other"},
        }
        with patch.object(
            activation.test_resources, "resource_status", return_value=records
        ):
            frozen = activation.borrowers(selected)
        self.assertIn("retained-export", frozen)
        self.assertIn("receiver", frozen)
        self.assertNotIn("other", frozen)

    def test_unknown_partial_archive_and_closed_admission_refusal_have_no_native_effect(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            state = root / "state"
            archive = root / "archive"
            state.mkdir()
            archive.mkdir()
            (archive / "unknown").write_bytes(b"retained")
            bridge = root / "bridge"
            bridge.write_text("unused")
            bridge.chmod(0o700)
            with (
                patch.object(
                    server,
                    "lifecycle_reservation",
                    return_value=contextlib.nullcontext(),
                ),
                patch.object(server, "config_for", return_value={}),
                patch.object(server, "private_offline_state"),
                patch.object(activation.subprocess, "Popen") as launch,
                self.assertRaisesRegex(
                    server.SupervisorError, "Unknown activation history"
                ),
            ):
                activation.execute(state, "original-migration", archive, bridge)
            launch.assert_not_called()
            self.assertEqual((archive / "unknown").read_bytes(), b"retained")

    def test_interrupt_stays_closed_and_same_identity_resume_reuses_original_backup(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            state = root / "state"
            archive = root / "archive"
            state.mkdir()
            archive.mkdir()
            config: dict[str, object] = {
                "accepting_writes": False,
                "admission": "quiesced",
                "instance_id": "fixture",
                "namespace": "ns",
                "database": "db",
                "endpoint": "ws://127.0.0.1:1",
                "server": {},
                "resources": {},
            }
            server.write_json(state / "config.json", config)
            server.write_json(state / "credentials.json", {"fixture": "private"})
            bridge = root / "bridge"
            # A real short-lived child exercises pipe settlement and failure. It
            # performs no database operation; authentic server controls are separate.
            bridge.write_text(
                f'#!{sys.executable}\nimport sys\nprint(\'{{"event":"authorize","step":1}}\',flush=True)\nanswer=sys.stdin.readline()\nsys.exit(7)\n'
            )
            bridge.chmod(0o700)
            backup_calls: list[Path] = []

            def backup(
                _state: Path, _config: dict[str, object], destination: Path
            ) -> dict[str, object]:
                backup_calls.append(destination)
                destination.mkdir()
                (destination / "raw-original").write_bytes(b"unchanged-original")
                server.write_json(
                    destination / "backup.json",
                    {
                        "files": {
                            "raw-original": server.file_digest(
                                destination / "raw-original"
                            )
                        }
                    },
                )
                return {}

            with (
                patch.object(
                    server,
                    "lifecycle_reservation",
                    side_effect=lambda *_a, **_k: contextlib.nullcontext(),
                ),
                patch.object(
                    server,
                    "config_for",
                    side_effect=lambda p: server.read_json(p / "config.json"),
                ),
                patch.object(server, "private_offline_state"),
                patch.object(server, "_backup", side_effect=backup),
                patch.object(server, "maintenance_endpoint"),
                patch.object(server, "require_maintenance_owner"),
                patch.object(server, "listener_ready", return_value=True),
                patch.object(
                    activation,
                    "borrowers",
                    return_value={"retained-export": "unchanged"},
                ),
            ):
                for _ in range(2):
                    with self.assertRaisesRegex(server.SupervisorError, "interrupted"):
                        activation.execute(state, "original-migration", archive, bridge)
                    current = server.read_json(state / "config.json")
                    self.assertIs(current["accepting_writes"], False)
                    self.assertEqual(current["admission"], "validation_required")
                with self.assertRaisesRegex(
                    server.SupervisorError, "different preparation activation"
                ):
                    activation.execute(state, "changed-migration", archive, bridge)
            self.assertEqual(len(backup_calls), 1)
            self.assertEqual(backup_calls[0].parent, archive)
            self.assertEqual(
                (archive / "backup" / "raw-original").read_bytes(),
                b"unchanged-original",
            )
            self.assertEqual(
                server.read_json(archive / "operation.json")["migration"],
                "original-migration",
            )


if __name__ == "__main__":
    unittest.main()
