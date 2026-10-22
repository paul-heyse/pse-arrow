# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual feature report and correctness build selection controls."""
# ruff: noqa: PT009, PT027 -- standard-library process boundary controls

from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import python_extension


class PythonExtensionTests(unittest.TestCase):
    def test_diagnostic_wheel_preserves_large_full_debug_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            wheels = root / "build/diagnostic-taskdump/wheels"
            wheels.mkdir(parents=True)
            (wheels / "pse_arrow-fixture.whl").write_bytes(b"selected wheel")
            with (
                patch.object(python_extension, "ROOT", root),
                patch("scripts.build_environment.configure", return_value={}),
                patch.object(python_extension.subprocess, "call", return_value=0) as run,
                patch.object(python_extension, "diagnostic_identity", return_value={}),
            ):
                self.assertEqual(python_extension.diagnostic_sync(), 0)
            build = run.call_args_list[1].args[0]
            self.assertIn("--compression-enable-large-file-support", build)
            self.assertEqual(build[build.index("--compression-method") + 1], "stored")
            self.assertNotIn("--strip", build)
            self.assertNotIn("develop", build)

    def test_diagnostic_identity_refuses_a_wheel_with_ordinary_flags(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            selected = Path(directory)
            package = selected / "lib/pse"
            package.mkdir(parents=True)
            binary = package / "_native.so"
            binary.write_bytes(b"diagnostic actual bytes")
            report = {
                "package": str(package / "__init__.py"), "binary": str(binary),
                "prefix": str(selected), "features": ["force-validate"],
            }
            with patch.object(
                python_extension.subprocess, "check_output",
                side_effect=lambda *_args, **_kwargs: json.dumps(report),
            ):
                with self.assertRaisesRegex(ValueError, "lacks actual"):
                    python_extension.diagnostic_identity(selected, {})
                report["features"] = ["force-validate", "diagnostic-taskdump"]
                actual = python_extension.diagnostic_identity(selected, {})
                self.assertEqual(actual["features"], report["features"])
                report["binary"] = str(selected.parent / "foreign.so")
                with self.assertRaisesRegex(ValueError, "foreign"):
                    python_extension.diagnostic_identity(selected, {})

    def test_import_path_flags_and_bytes_belong_to_one_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            package = root / "python/pse"
            package.mkdir(parents=True)
            binary = package / "_native.so"
            binary.write_bytes(b"actual extension bytes")
            report: dict[str, object] = {
                "package": str(package / "__init__.py"),
                "native": str(binary),
                "features": ["force-validate"],
            }
            with (
                patch.object(python_extension, "ROOT", root),
                patch.object(
                    python_extension.subprocess,
                    "check_output",
                    return_value=json.dumps(report),
                ),
            ):
                observed = python_extension.capture({})
                self.assertEqual(observed["features"], ["force-validate"])
                self.assertEqual(
                    observed["sha256"], hashlib.sha256(binary.read_bytes()).hexdigest()
                )
                for features, native in [
                    ([], False),
                    (["native-solvers"], True),
                    (["force-validate"], True),
                    (["force-validate", "force-validate"], False),
                    ([{}], False),
                ]:
                    with self.subTest(features=features, native=native):
                        report["features"] = features
                        with (
                            patch.object(
                                python_extension.subprocess,
                                "check_output",
                                return_value=json.dumps(report),
                            ),
                            self.assertRaises(ValueError),
                        ):
                            python_extension.capture({}, native=native)
                report["features"] = ["force-validate", "native-solvers"]
                with patch.object(
                    python_extension.subprocess,
                    "check_output",
                    return_value=json.dumps(report),
                ):
                    self.assertEqual(
                        python_extension.capture({}, native=True)["features"],
                        report["features"],
                    )
                report["native"] = str(root / "other/_native.so")
                with (
                    patch.object(
                        python_extension.subprocess,
                        "check_output",
                        return_value=json.dumps(report),
                    ),
                    self.assertRaisesRegex(ValueError, "different checkout"),
                ):
                    python_extension.capture({})

    def test_correctness_build_explicitly_selects_validation_and_profile(self) -> None:
        for mode in ["dev", "native"]:
            with (
                self.subTest(mode=mode),
                patch.object(
                    python_extension.subprocess, "call", return_value=0
                ) as run,
            ):
                self.assertEqual(python_extension.sync(mode, "producer"), 0)
                commands = [call.args[0] for call in run.call_args_list]
                self.assertEqual(len(commands), 3)
                build = commands[1]
                self.assertEqual(build[build.index("--profile") + 1], "producer")
                self.assertEqual(
                    build[build.index("--features") + 1],
                    "force-validate,native-solvers"
                    if mode == "native"
                    else "force-validate",
                )
                self.assertEqual(
                    any(argument.startswith("--native=") for argument in build),
                    mode == "native",
                )
                self.assertEqual(commands[2][-1], "python-stubs")

    def test_failed_build_never_runs_following_steps(self) -> None:
        with patch.object(
            python_extension.subprocess, "call", side_effect=[0, 17]
        ) as run:
            self.assertEqual(python_extension.sync("dev"), 17)
        self.assertEqual(run.call_count, 2)
