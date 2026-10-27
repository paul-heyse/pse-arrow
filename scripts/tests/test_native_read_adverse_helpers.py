# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Owned-file replacement controls; no installed extension or service is touched."""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import hashlib
import mmap
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts import host_admission, process_observation
from scripts.tests.native_read_adverse import (
    AuthenticatedReadWorker,
    ImageSelection,
    ReplacementRefusedError,
    image_stamp,
    replaced_image,
)


class ReadWorkerSettlementTests(unittest.TestCase):
    def test_borrowed_parent_stays_charged_and_independent_owner_must_release(
        self,
    ) -> None:
        worker = AuthenticatedReadWorker.__new__(AuthenticatedReadWorker)
        worker.allocation = Path("/owned/allocations/original")
        worker.unit = "original.scope"
        worker.bound = {
            "group": "/owned/original.scope",
            "inode": 321,
            "invocation": "original",
        }
        worker.borrowed_parent = True
        parent = SimpleNamespace(
            directory=worker.allocation.parent, nonce=worker.allocation.name
        )
        record = {"units": {worker.unit: worker.bound}}
        with (
            patch.object(worker, "exited", return_value=True) as exited,
            patch.object(worker, "session_members", return_value=[]) as family,
            patch.object(
                host_admission,
                "readonly_snapshot",
                return_value={"owners": {worker.allocation.name: record}},
            ) as snapshot,
            patch.object(host_admission, "inherit", return_value=parent) as inherit,
            patch.object(process_observation, "group_identity", return_value=321),
        ):
            self.assertTrue(worker.settlement_observed(123))
            inherit.assert_called_once()
            worker.borrowed_parent = False
            self.assertFalse(worker.settlement_observed(123))
            record["units"] = {}
            self.assertTrue(worker.settlement_observed(123))
            worker.borrowed_parent = True
            with self.assertRaisesRegex(ValueError, "parent allocation disappeared"):
                worker.settlement_observed(123)
            snapshot.return_value = {"owners": {}}
            with self.assertRaisesRegex(ValueError, "parent allocation disappeared"):
                worker.settlement_observed(123)
            exited.return_value = False
            snapshot.reset_mock()
            self.assertFalse(worker.settlement_observed(123))
            snapshot.assert_not_called()
            exited.return_value = True
            family.return_value = [999]
            self.assertFalse(worker.settlement_observed(123))
            snapshot.assert_not_called()

    def test_changed_parent_binding_refuses_settlement(self) -> None:
        worker = AuthenticatedReadWorker.__new__(AuthenticatedReadWorker)
        worker.allocation = Path("/owned/allocations/original")
        worker.unit = "original.scope"
        worker.bound = {"group": "/owned/original.scope", "inode": 321}
        worker.borrowed_parent = True
        with (
            patch.object(worker, "exited", return_value=True),
            patch.object(worker, "session_members", return_value=[]),
            patch.object(
                host_admission,
                "readonly_snapshot",
                return_value={
                    "owners": {
                        worker.allocation.name: {
                            "units": {worker.unit: {"group": "/changed"}}
                        }
                    }
                },
            ),
            patch.object(host_admission, "inherit", return_value=None),
            self.assertRaisesRegex(ValueError, "parent allocation changed"),
        ):
            worker.settlement_observed(123)


class OwnedImageReplacementTests(unittest.TestCase):
    def test_original_mapped_inode_and_all_bytes_survive_path_replacement_and_restore(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            binary = Path(temporary) / "_native.so"
            original = b"original-mapped-fixture-bytes" * 128
            binary.write_bytes(original)
            selected = ImageSelection(
                binary, hashlib.sha256(original).hexdigest(), image_stamp(binary)
            )
            with (
                binary.open("rb") as source,
                mmap.mmap(source.fileno(), 0, access=mmap.ACCESS_READ) as mapped,
            ):
                with replaced_image(selected) as controlled:
                    self.assertNotEqual(image_stamp(binary)[:2], selected.stamp[:2])
                    self.assertEqual(
                        image_stamp(controlled.backup)[:2], selected.stamp[:2]
                    )
                    self.assertEqual(mapped[:], original)
                    self.assertNotEqual(binary.read_bytes(), original)
                self.assertEqual(mapped[:], original)
            self.assertEqual(image_stamp(binary)[:2], selected.stamp[:2])
            self.assertEqual(binary.read_bytes(), original)
            self.assertFalse(
                any(
                    path.name.startswith(".pse-read-image-control-")
                    for path in binary.parent.iterdir()
                )
            )

    def test_body_failure_restores_original_and_concurrent_path_mutation_is_never_overwritten(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            binary = Path(temporary) / "_native.so"
            original = b"original-fixture"
            binary.write_bytes(original)
            selected = ImageSelection(
                binary, hashlib.sha256(original).hexdigest(), image_stamp(binary)
            )
            with (
                self.assertRaisesRegex(ValueError, "original-body-failure"),
                replaced_image(selected),
            ):
                raise ValueError("original-body-failure")
            self.assertEqual(binary.read_bytes(), original)
            selected = ImageSelection(
                binary, hashlib.sha256(original).hexdigest(), image_stamp(binary)
            )
            backup: Path | None = None
            with (
                self.assertRaises(ReplacementRefusedError),
                replaced_image(selected) as controlled,
            ):
                backup = controlled.backup
                concurrent = binary.with_name("concurrent-installer-file")
                concurrent.write_bytes(b"concurrent-fixture-installation")
                concurrent.replace(binary)
            self.assertEqual(binary.read_bytes(), b"concurrent-fixture-installation")
            assert backup is not None
            self.assertEqual(backup.read_bytes(), original)
            self.assertEqual(image_stamp(backup)[:2], selected.stamp[:2])


if __name__ == "__main__":
    unittest.main()
