# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Original invocation fallback readers do not predict admission or settlement."""
from __future__ import annotations
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from scripts import test_resources
from scripts.capabilities import rust_references
from scripts.capabilities.rust_references import read
from scripts.capabilities.rust_test_contracts import InvocationReference

class ReferenceTests(unittest.TestCase):
    def test_absence_is_unconfirmed_and_real_registry_identity_is_required(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            output = root / 'build/selected'
            self.assertEqual(read(root, str(output)).outcome, 'not_observed')
            output.mkdir(parents=True)
            with patch.dict(os.environ, {'PSE_TEST_RESOURCE_STATE': str(root / 'registry')}), patch.object(test_resources.host, 'inherit', return_value=None):
                invocation = test_resources.invocation('rust', [])
                reference = InvocationReference(invocation=str(invocation), registry=str(root / 'registry'), ownership='owned', terminal_owner='native runner')
                (output / 'invocation-reference.json').write_text(reference.model_dump_json())
                observed = read(root, str(output))
                self.assertEqual(observed.outcome, 'issued')
                self.assertEqual(observed.reference, reference)
                self.assertEqual(observed.reader, 'product_run_read')
                self.assertIsNotNone(observed.source_sha256)
                self.assertIn('do not establish settlement', observed.settlement)
                value = json.loads(reference.model_dump_json())
                value['invocation'] = str(root / 'foreign.json')
                (output / 'invocation-reference.json').write_text(json.dumps(value))
                with self.assertRaisesRegex(ValueError, 'does not belong'):
                    read(root, str(output))

    def test_path_and_record_refusals_do_not_read_unbounded_or_foreign_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            output = root / 'build/selected'
            output.mkdir(parents=True)
            source = output / 'invocation-reference.json'
            source.write_text('x' * 17000)
            with self.assertRaisesRegex(ValueError, 'bounded'):
                read(root, str(output))
            source.unlink()
            source.symlink_to(root / 'missing')
            with self.assertRaises(OSError):
                read(root, str(output))
            with self.assertRaises(ValueError):
                read(root, '../foreign')

    def test_parent_swap_never_opens_outside_reference_and_stats_held_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp, tempfile.TemporaryDirectory() as foreign:
            root = Path(tmp)
            parent = root / "build"
            output = parent / "selected"
            output.mkdir(parents=True)
            outside = Path(foreign) / "selected"
            outside.mkdir()
            outside_source = outside / "invocation-reference.json"
            outside_source.write_bytes(b"outside bytes must never be opened")
            outside_identity = (outside_source.stat().st_dev, outside_source.stat().st_ino)
            with patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(root / "registry")}), patch.object(test_resources.host, "inherit", return_value=None):
                invocation = test_resources.invocation("rust", [])
                reference = InvocationReference(invocation=str(invocation), registry=str(root / "registry"), ownership="owned", terminal_owner="native runner")
                source = output / "invocation-reference.json"
                source.write_text(reference.model_dump_json())
                source_identity = (source.stat().st_dev, source.stat().st_ino)
                native_open = os.open
                captured: list[tuple[int, int]] = []
                swapped = False

                def open_after_parent_swap(
                    path: str | Path, flags: int, mode: int = 0o777,
                    *, dir_fd: int | None = None,
                ) -> int:
                    nonlocal swapped
                    if Path(path).name == "invocation-reference.json":
                        parent.rename(root / "original-build")
                        parent.symlink_to(Path(foreign), target_is_directory=True)
                        swapped = True
                    descriptor = native_open(path, flags, mode, dir_fd=dir_fd)
                    if Path(path).name == "invocation-reference.json":
                        information = os.fstat(descriptor)
                        identity = (information.st_dev, information.st_ino)
                        if identity == outside_identity:
                            os.close(descriptor)
                            self.fail("outside reference was opened through a swapped ancestor")
                        captured.append(identity)
                    return descriptor

                with patch.object(rust_references.os, "open", side_effect=open_after_parent_swap):
                    observed = read(root, str(output))
                self.assertTrue(swapped)
                self.assertEqual(captured, [source_identity])
                self.assertEqual(observed.outcome, "issued")
                self.assertEqual(observed.reference, reference)
                self.assertTrue(parent.is_symlink())

    def test_parent_swap_before_directory_open_refuses_without_opening_reference(self) -> None:
        with tempfile.TemporaryDirectory() as tmp, tempfile.TemporaryDirectory() as foreign:
            root = Path(tmp)
            parent = root / "build"
            (parent / "selected").mkdir(parents=True)
            outside = Path(foreign) / "selected"
            outside.mkdir()
            (outside / "invocation-reference.json").write_bytes(b"outside bytes must never be opened")
            native_open = os.open
            opened_reference: list[str | Path] = []
            swapped = False

            def swap_before_directory_open(
                path: str | Path, flags: int, mode: int = 0o777,
                *, dir_fd: int | None = None,
            ) -> int:
                nonlocal swapped
                if path == "build":
                    parent.rename(root / "original-build")
                    parent.symlink_to(Path(foreign), target_is_directory=True)
                    swapped = True
                if Path(path).name == "invocation-reference.json":
                    opened_reference.append(path)
                return native_open(path, flags, mode, dir_fd=dir_fd)

            with patch.object(rust_references.os, "open", side_effect=swap_before_directory_open):
                with self.assertRaises(OSError):
                    read(root, "build/selected")
            self.assertTrue(swapped)
            self.assertFalse(opened_reference)
