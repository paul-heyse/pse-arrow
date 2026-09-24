# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise environment edits and rollback entirely inside disposable directories."""
# ruff: noqa: PT009, PT027 -- stdlib bootstrap controls

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import llvm_system as llvm


class LlvmSystemTests(unittest.TestCase):
    def test_environment_preserves_unrelated_settings_and_is_idempotent(self) -> None:
        original = '# existing comment\nPATH="/usr/local/bin:/usr/bin:/bin"\nLANG="en_US.UTF-8"\n'
        result = llvm.environment_text(original)
        self.assertIn('LANG="en_US.UTF-8"', result)
        self.assertTrue(result.startswith("# existing comment\n"))
        self.assertIn(
            'PATH="/opt/llvm-current/bin:/usr/local/bin:/usr/bin:/bin"', result
        )
        self.assertEqual(llvm.environment_text(result), result)
        self.assertNotIn("LD_LIBRARY_PATH", result)
        self.assertNotIn("CC=", result)

    def test_duplicate_environment_authority_is_refused(self) -> None:
        with self.assertRaisesRegex(ValueError, "multiple PATH"):
            llvm.environment_text('PATH="/one"\nPATH="/two"\n')

    def test_root_required_before_apply_mutates_anything(self) -> None:
        with (
            patch.object(llvm.os, "geteuid", return_value=1000),
            patch.object(llvm, "stage") as stage,
        ):
            with self.assertRaisesRegex(ValueError, "requires root"):
                llvm.apply(Path("unused"), Path("/opt/unused"), "23.1.2")
            stage.assert_not_called()

    def test_staging_refuses_existing_destination(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            self.assertRaisesRegex(ValueError, "already exists"),
        ):
            llvm.stage(Path("unused"), Path(directory), "23.1.2")

    def test_rollback_restores_files_symlinks_modes_and_absence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            environment = root / "environment"
            environment.write_text('PATH="/original"\n')
            environment.chmod(0o640)
            selector = root / "current"
            selector.symlink_to("prior-installation")
            service = root / "service.conf"
            links = (root / "clang", root / "clang++", root / "llvm-config")
            with (
                patch.object(llvm, "SELECTOR", selector),
                patch.object(llvm, "ENVIRONMENT", environment),
                patch.object(llvm, "SERVICE_ENVIRONMENT", service),
                patch.object(llvm, "LINKS", links),
                patch.object(llvm.os, "geteuid", return_value=0),
                patch.object(llvm, "require_trusted_backup"),
            ):
                saved = root / "backup"
                manifest = llvm.backup(saved)
                for record in manifest["records"]:
                    path = Path(record["path"])
                    if path in (environment, service):
                        llvm.atomic_text(path, "new contents\n")
                    else:
                        llvm.atomic_link(path, Path("/new-installation"))
                    record["after"] = llvm.fingerprint(path)
                (saved / "rollback.json").write_text(json.dumps(manifest))
                llvm.rollback(saved)
                self.assertEqual(environment.read_text(), 'PATH="/original"\n')
                self.assertEqual(environment.stat().st_mode & 0o777, 0o640)
                self.assertEqual(selector.readlink(), Path("prior-installation"))
                self.assertFalse(service.exists())
                self.assertTrue(all(not path.is_symlink() for path in links))
                # A later administrator change must prevent every rollback write.
                environment.write_text("later administrator change\n")
                with self.assertRaisesRegex(ValueError, "changed since migration"):
                    llvm.rollback(saved)
                self.assertEqual(
                    environment.read_text(), "later administrator change\n"
                )

    def test_backup_directory_rejects_shared_write_access(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            Path(directory).chmod(0o777)
            with self.assertRaisesRegex(ValueError, "root-owned"):
                llvm.require_trusted_backup(Path(directory))


if __name__ == "__main__":
    unittest.main()
