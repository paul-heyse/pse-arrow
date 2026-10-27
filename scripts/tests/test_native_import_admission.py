# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native images enter a fresh worker only after currency admission."""

# ruff: noqa: PT009 -- stdlib host controls
from __future__ import annotations

import hashlib
import os
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from scripts import python_extension, telemetry_process
from scripts.capabilities import execution_basis, native_worker


class NativeImportTests(unittest.TestCase):
    def test_already_loaded_image_cannot_borrow_a_new_current_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "_native.so"
            binary.write_bytes(b"current image")
            artifact = python_extension.ExtensionArtifact(
                binary=str(binary),
                sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                features=["force-validate", "native-solvers"],
            )
            current = python_extension.ExtensionCurrency(
                state="current",
                checkout=directory,
                interpreter=sys.executable,
                mode="native",
                profile="dev",
                validated=True,
                generation="selected-generation",
                artifact=artifact,
                receipt_recorded=True,
                cause="published-compatible-artifact",
                remedy="Explicit preparation only.",
            )
            with (
                patch.dict(
                    sys.modules, {"pse._native": types.ModuleType("pse._native")}
                ),
                patch.dict(
                    os.environ, {execution_basis.ENVIRONMENT: "/private/admission.json"}
                ),
                patch.object(native_worker, "extension_currency", return_value=current),
                patch.object(telemetry_process, "activate_native") as activate,
            ):
                result = native_worker.execute(
                    "diagnostic_checkpoint",
                    {
                        "schema_set_fingerprint": "same-schema",
                        "checkpoint": "/original/checkpoint",
                    },
                    scratch=Path(directory),
                )
            self.assertEqual(result["code"], "pse.native.unbound_loaded_artifact")
            activate.assert_not_called()

    def test_lazy_native_observation_retains_original_shutdown_owner(self) -> None:
        settings = Mock()
        settings.native_json.return_value = "selected settings"
        native = Mock()
        native.activation = {"status": "initialized"}
        facade = Mock()
        facade.initialize.return_value = native
        owner = telemetry_process.ProcessLifetime(settings, "initialized")
        with (
            patch.object(telemetry_process._STATE, "active", owner),  # noqa: SLF001 -- exercise the original private lifecycle or admission boundary
            patch.object(
                telemetry_process.importlib, "import_module", return_value=facade
            ),
        ):
            telemetry_process.activate_native()
            telemetry_process.activate_native()
            self.assertIs(owner.native, native)
            facade.initialize.assert_called_once()
            owner.shutdown()
            telemetry_process.activate_native()
            facade.initialize.assert_called_once()
        native.shutdown.assert_called_once()
