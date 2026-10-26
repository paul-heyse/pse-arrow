# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Frozen receiver launch validates existing native inputs and never prepares them."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import native_cache as cache
from scripts import native_operation as operation
from scripts import receiver_environment as receiver


class ReceiverEnvironmentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.base = self.root / "native"
        self.enterContext(patch.dict(os.environ, {"PSE_NATIVE_CACHE": str(self.base)}))
        identity = {"fixture": "owned native generation"}
        self.prefix = (
            cache.identity_location(self.base, "isolation", identity)
            / "generations/selected"
        )
        for name in cache.ISOLATION_FILES:
            path = self.prefix / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("native artifact " + name)
        (self.prefix / ".complete.json").write_text(
            json.dumps(
                {
                    "version": cache.RECEIPT_VERSION,
                    "identity": identity,
                    "files": cache.installation_files(self.prefix),
                }
            )
        )
        compiler = self.root / "selected-clang"
        compiler.write_text("#!/bin/sh\nexit 91\n")
        compiler.chmod(0o700)
        self.environment = {
            "PSE_NATIVE_CACHE": str(self.base),
            "PSE_ROOT_ISOLATION_DIR": str(self.prefix),
            "CLANG_PATH": str(compiler),
            "CXXFLAGS": "-fno-fast-math",
            "OMP_NUM_THREADS": "16",
            "SYMBOLICA_LICENSE": "setup-secret",
        }
        self.path = self.root / "native-environment.json"

    def capture(self) -> None:
        self.path.write_bytes(receiver.capture(["isolation"], self.environment))
        self.path.chmod(0o600)

    def test_capture_excludes_secrets_and_thread_budgets_and_restore_never_prepares(
        self,
    ) -> None:
        with (
            patch.object(
                operation, "environment", side_effect=AssertionError("setup entered")
            ),
            patch.object(
                cache,
                "compiler_env",
                side_effect=AssertionError("compiler discovery entered"),
            ),
            patch.object(
                cache, "prepare", side_effect=AssertionError("provider builder entered")
            ),
        ):
            self.capture()
            raw = self.path.read_bytes()
            self.assertNotIn(b"setup-secret", raw)
            self.assertNotIn(b"OMP_NUM_THREADS", raw)
            restored = receiver.restore(
                self.path,
                ["isolation"],
                {"OMP_NUM_THREADS": "7", "SYMBOLICA_LICENSE": "invocation-secret"},
            )
        self.assertEqual(restored["OMP_NUM_THREADS"], "7")
        self.assertEqual(restored["SYMBOLICA_LICENSE"], "invocation-secret")
        self.assertEqual(restored["CLANG_PATH"], self.environment["CLANG_PATH"])
        self.assertEqual(restored["CXXFLAGS"], "-fno-fast-math")
        self.assertTrue(list((self.base / ".operations").glob("*.json")))

    def test_absent_compiler_inputs_stay_absent_in_actual_child_environment(
        self,
    ) -> None:
        self.capture()
        absent = (
            "LIBCLANG_PATH",
            "CPLUS_INCLUDE_PATH",
            "BINDGEN_EXTRA_CLANG_ARGS",
            "LD_LIBRARY_PATH",
        )
        caller = {
            **os.environ,
            **dict.fromkeys(absent, "/unadmitted"),
            "CLANG_PATH": "/unadmitted/compiler",
            "CXXFLAGS": "-unadmitted",
            "OMP_NUM_THREADS": "7",
            "SYMBOLICA_LICENSE": "invocation-secret",
        }
        restored = receiver.restore(self.path, ["isolation"], caller)
        child = subprocess.run(
            [
                sys.executable,
                "-I",
                "-B",
                "-c",
                "import json,os; print(json.dumps(dict(os.environ)))",
            ],
            env=restored,
            capture_output=True,
            check=True,
            timeout=10,
        )
        actual = json.loads(child.stdout)
        for name in absent:
            self.assertNotIn(name, actual)
        self.assertEqual(actual["CLANG_PATH"], self.environment["CLANG_PATH"])
        self.assertEqual(actual["CXXFLAGS"], self.environment["CXXFLAGS"])
        self.assertEqual(actual["OMP_NUM_THREADS"], "7")
        self.assertEqual(actual["SYMBOLICA_LICENSE"], "invocation-secret")

    def test_complete_worker_capabilities_verify_all_managed_generations_without_preparation(
        self,
    ) -> None:
        from scripts.surreal_server import WORKER_CAPABILITIES

        environment = dict(self.environment)
        interfaces = {
            "solver": cache.SOLVER_FILES,
            "klu": cache.KLU_FILES,
            "uno": cache.pipeline_files("uno"),
            "petsc": cache.pipeline_files("petsc"),
        }
        prefixes = {}
        for kind, required in interfaces.items():
            identity = {"fixture": kind}
            prefix = (
                cache.identity_location(self.base, kind, identity)
                / "generations/selected"
            )
            for name in required:
                path = prefix / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("native artifact " + name)
            (prefix / ".complete.json").write_text(
                json.dumps(
                    {
                        "version": cache.RECEIPT_VERSION,
                        "identity": identity,
                        "files": cache.installation_files(prefix),
                    }
                )
            )
            prefixes[kind] = prefix
        environment.update(
            {
                "IPOPT_DIR": str(prefixes["solver"]),
                "SCIPOPTDIR": str(prefixes["solver"]),
                "SUITESPARSE_INCLUDE_DIR": str(prefixes["klu"] / "include/suitesparse"),
                "SUITESPARSE_LIBRARY_DIR": str(prefixes["klu"] / "lib"),
                "UNO_DIR": str(prefixes["uno"]),
                "PETSC_DIR": str(prefixes["petsc"]),
            }
        )
        with (
            patch.object(
                operation, "environment", side_effect=AssertionError("setup entered")
            ),
            patch.object(
                cache,
                "compiler_env",
                side_effect=AssertionError("compiler discovery entered"),
            ),
            patch.object(
                cache, "prepare", side_effect=AssertionError("provider builder entered")
            ),
        ):
            self.path.write_bytes(receiver.capture(WORKER_CAPABILITIES, environment))
            self.path.chmod(0o600)
            restored = receiver.restore(self.path, WORKER_CAPABILITIES, environment)
        self.assertEqual(len(receiver.read(self.path).provider_receipts), 5)
        self.assertEqual(
            restored["SUITESPARSE_INCLUDE_DIR"], environment["SUITESPARSE_INCLUDE_DIR"]
        )

    def test_original_receipt_hash_is_not_a_substitute_for_actual_generation_verification(
        self,
    ) -> None:
        self.capture()
        artifact = self.prefix / cache.ISOLATION_FILES[0]
        artifact.chmod(0o600)
        artifact.write_text("changed actual library")
        with self.assertRaisesRegex(ValueError, "corrupt"):
            receiver.restore(self.path, ["isolation"], self.environment)
        self.assertEqual(artifact.read_text(), "changed actual library")

    def test_missing_preparation_wrong_capabilities_unknown_fields_and_versions_refuse(
        self,
    ) -> None:
        with self.assertRaisesRegex(receiver.ReceiverEnvironmentError, "Prepare"):
            receiver.capture(["isolation"], {})
        self.capture()
        with self.assertRaisesRegex(receiver.ReceiverEnvironmentError, "capability"):
            receiver.restore(self.path, ["solver"], self.environment)
        original = json.loads(self.path.read_bytes())
        for changed in (
            {**original, "version": 2},
            {**original, "unknown": 1},
            {key: value for key, value in original.items() if key != "version"},
        ):
            self.path.write_text(json.dumps(changed))
            with self.assertRaises(ValueError):
                receiver.read(self.path)

    def test_changed_receipt_or_injected_budget_cannot_be_reinterpreted(self) -> None:
        self.capture()
        receipt = self.prefix / ".complete.json"
        receipt.chmod(0o600)
        receipt.write_text(receipt.read_text() + "\n")
        with self.assertRaisesRegex(receiver.ReceiverEnvironmentError, "readmit"):
            receiver.restore(self.path, ["isolation"], self.environment)
        self.capture()
        value = json.loads(self.path.read_bytes())
        value["values"]["OMP_NUM_THREADS"] = "1000"
        self.path.write_text(json.dumps(value))
        with self.assertRaisesRegex(receiver.ReceiverEnvironmentError, "capability"):
            receiver.restore(self.path, ["isolation"], self.environment)

    def test_shared_prefix_retains_every_selected_capability_interface(self) -> None:
        environment = {
            **self.environment,
            "IPOPT_DIR": str(self.prefix),
            "SCIPOPTDIR": str(self.prefix),
        }
        with self.assertRaisesRegex(ValueError, "required consumed interfaces"):
            receiver.capture(["solver", "isolation"], environment)
        for name in cache.SOLVER_FILES:
            path = self.prefix / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("solver artifact " + name)
        receipt = self.prefix / ".complete.json"
        value = json.loads(receipt.read_text())
        value["files"] = cache.installation_files(self.prefix)
        receipt.write_text(json.dumps(value))
        raw = receiver.capture(["solver", "isolation"], environment)
        self.assertEqual(len(json.loads(raw)["provider_receipts"]), 1)


if __name__ == "__main__":
    unittest.main()
