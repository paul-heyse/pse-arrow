# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Retained image witness controls; these do not claim actual extension loading."""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import hashlib
import sys
import tempfile
import unittest
from pathlib import Path

from scripts import python_extension
from scripts.capabilities.native_read_image import LoadedReadImage, ReadImageChanged


class ReadImageTests(unittest.TestCase):
    def current(self, root: Path) -> python_extension.ExtensionCurrency:
        binary = root / "_native.so"
        if not binary.exists():
            binary.write_bytes(b"original-installed-image")
        artifact = python_extension.ExtensionArtifact(binary=str(binary),
            sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), features=["force-validate", "native-solvers"])
        return python_extension.ExtensionCurrency(state="current", checkout=str(root),
            interpreter=sys.executable, mode="native", profile="dev", validated=True,
            source_basis="original-producing-provider-preparation-basis", artifact=artifact,
            artifact_stamp=python_extension.artifact_stamp(artifact), receipt_recorded=True,
            cause="matching-inputs-and-artifact", remedy="Explicit preparation only.")

    def test_original_loaded_image_cannot_borrow_changed_receipt_or_interpreter(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            current = self.current(Path(temporary))
            image = LoadedReadImage.admit(current)
            image.require_current(current)
            assert current.artifact is not None
            image.require_loaded(current.artifact.binary)
            for changed in ({"source_basis": "changed-consumed-source"},
                            {"interpreter": "changed-interpreter"}, {"profile": "changed-profile"}):
                with self.assertRaises(ReadImageChanged):
                    image.require_current(current.model_copy(update=changed))
            self.assertNotEqual(image.context_identity("same-supplied-context"), image.context_identity("changed-supplied-context"))

    def test_image_replacement_inside_import_window_or_between_requests_refuses(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            current = self.current(root)
            image = LoadedReadImage.admit(current)
            replacement = root / "replacement.so"
            replacement.write_bytes(b"replacement-installed-image")
            replacement.replace(root / "_native.so")
            assert current.artifact is not None
            with self.assertRaises(ReadImageChanged):
                image.require_loaded(current.artifact.binary)
            newly_current = self.current(root)
            with self.assertRaises(ReadImageChanged):
                image.require_current(newly_current)
            self.assertNotEqual(image.context_identity("supplied"), LoadedReadImage.admit(newly_current).context_identity("supplied"))


if __name__ == "__main__":
    unittest.main()
