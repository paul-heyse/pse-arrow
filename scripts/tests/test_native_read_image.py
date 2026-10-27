# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""An attached generation consumes publication; it does not repeat construction."""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import python_extension
from scripts.capabilities.native_read_image import (
    LoadedReadImage,
    ReadImageChangedError,
)


class ReadImageTests(unittest.TestCase):
    def current(self) -> python_extension.ExtensionCurrency:
        return python_extension.ExtensionCurrency(
            state="current",
            checkout="/selected",
            interpreter=sys.executable,
            mode="native",
            profile="dev",
            validated=True,
            generation="original-generation",
            artifact=python_extension.ExtensionArtifact(
                binary="/published/original/_native.so",
                sha256="constructor-recorded-identity",
                features=["force-validate", "native-solvers"],
            ),
            receipt_recorded=True,
            cause="published-compatible-artifact",
            remedy="Select a compatible publication and reattach.",
        )

    def test_attachment_uses_issued_contract_without_filesystem_revalidation(
        self,
    ) -> None:
        with patch.object(
            Path, "stat", side_effect=AssertionError("redundant validation")
        ):
            image = LoadedReadImage.admit(self.current())
            image.require(native=True, validated=True)
            identity = image.context_identity("same-request")
            # A later publication does not change this owner's generation or cursors.
            newer = self.current().model_copy(update={"generation": "next-generation"})
            self.assertEqual(identity, image.context_identity("same-request"))
            self.assertNotEqual(
                identity, LoadedReadImage.admit(newer).context_identity("same-request")
            )
            self.assertNotEqual(identity, image.context_identity("different-request"))

    def test_distinct_requested_contract_is_checked_in_memory(self) -> None:
        current = self.current()
        dev = current.model_copy(update={"mode": "dev", "validated": False})
        image = LoadedReadImage.admit(dev)
        image.require(native=False, validated=False)
        for requirements in (
            {"native": True, "validated": False},
            {"native": False, "validated": True},
        ):
            with self.assertRaises(ReadImageChangedError):
                image.require(**requirements)
        for changes in ({"state": "unknown"}, {"generation": None}, {"artifact": None}):
            with self.assertRaises(ReadImageChangedError):
                LoadedReadImage.admit(current.model_copy(update=changes))

    def test_loaded_module_cannot_be_silently_replaced_by_a_new_attachment(
        self,
    ) -> None:
        image = LoadedReadImage.admit(self.current())
        with (
            patch.dict(sys.modules, {"pse._native": object()}),
            self.assertRaises(ReadImageChangedError),
        ):
            image.attach()


if __name__ == "__main__":
    unittest.main()
