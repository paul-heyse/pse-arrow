# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cargo-hack adaptation preserves its check scope and Cargo forwarding."""

from __future__ import annotations

import io
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import pytest

from scripts import cargo_feature_check as adapter


def expect_equal(actual: object, expected: object) -> None:
    """Check script results without Python's optimization-removable assert statement."""
    if actual != expected:
        message = f"{actual!r} != {expected!r}"
        raise AssertionError(message)


class CargoFeatureCheckTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.manifest = Path(self.directory.name) / "Cargo.toml"
        self.manifest.write_text('[package]\nname = "selected-package"\n')

    def test_powerset_features_qualified_without_changing_scope_or_flags(self) -> None:
        arguments = [
            "check",
            "--locked",
            "--manifest-path",
            str(self.manifest),
            "--no-default-features",
            "--features",
            "diffsol,default",
        ]
        expected = [
            *arguments[:-1],
            "selected-package/diffsol,selected-package/default",
        ]
        expect_equal(adapter.qualify_features(arguments), expected)
        expect_equal(arguments[-1], "diffsol,default")

    def test_all_feature_spellings_and_optional_controls(self) -> None:
        for feature_arguments, expected in (
            (
                ["--features", "ipopt highs,native-solvers"],
                [
                    "--features",
                    "selected-package/ipopt,selected-package/highs,selected-package/native-solvers",
                ],
            ),
            (
                ["--features=force-validate"],
                ["--features=selected-package/force-validate"],
            ),
            (["-F", "ipopt"], ["-F", "selected-package/ipopt"]),
            (["-Fhighs"], ["-Fselected-package/highs"]),
        ):
            with self.subTest(arguments=feature_arguments):
                prefix = ["check", f"--manifest-path={self.manifest}"]
                expect_equal(
                    adapter.qualify_features(prefix + feature_arguments),
                    prefix + expected,
                )

    def test_qualified_features_remain_qualified(self) -> None:
        arguments = [
            "check",
            "--manifest-path",
            str(self.manifest),
            "--features",
            "ipopt,pse-relations/force-validate,selected-package/highs",
        ]
        expect_equal(
            adapter.qualify_features(arguments)[-1],
            "selected-package/ipopt,pse-relations/force-validate,selected-package/highs",
        )
        qualified = ["check", "--features", "pse-relations/force-validate"]
        expect_equal(adapter.qualify_features(qualified), qualified)

    def test_no_features_and_other_commands_pass_through_exactly(self) -> None:
        for arguments in (
            [],
            ["--version"],
            ["metadata", "--format-version", "1"],
            ["locate-project"],
            ["check", "--locked", "--no-default-features"],
            ["check", "--all-features"],
            ["check", "--features", ""],
            ["build", "--features", "ipopt"],
            ["check", "--", "--features", "ipopt"],
        ):
            with self.subTest(arguments=arguments):
                expect_equal(adapter.qualify_features(arguments), arguments)

    def test_missing_selected_manifest_or_package_refused_clearly(self) -> None:
        with pytest.raises(ValueError, match="selected --manifest-path"):
            adapter.qualify_features(["check", "--features", "ipopt"])
        self.manifest.write_text("[workspace]\n")
        with pytest.raises(ValueError, match=r"lacks package\.name"):
            adapter.qualify_features(
                ["check", "--manifest-path", str(self.manifest), "-Fipopt"]
            )
        with pytest.raises(ValueError, match="requires a value"):
            adapter.qualify_features(["check", "--features"])

    def test_forwarding_uses_cargo_environment_and_inherits_process(self) -> None:
        arguments = [
            "adapter",
            "check",
            "--manifest-path",
            str(self.manifest),
            "-Fipopt",
        ]
        with (
            patch.object(adapter.sys, "argv", arguments),
            patch.dict(os.environ, {"CARGO": "/existing/pinned/cargo"}),
            patch.object(
                adapter,
                "compose",
                side_effect=lambda command, **_: [
                    *command,
                    "--features",
                    "selected-package/force-validate",
                ],
            ) as compose,
            patch.object(
                adapter.subprocess, "run", return_value=SimpleNamespace(returncode=0)
            ) as execute,
            patch.object(adapter.shutil, "which") as lookup,
        ):
            expect_equal(adapter.main(), 0)
        execute.assert_called_once_with(
            [
                "/existing/pinned/cargo",
                "check",
                "--manifest-path",
                str(self.manifest),
                "-Fselected-package/ipopt",
                "--features",
                "selected-package/force-validate",
            ],
            check=False,
        )
        compose.assert_called_once()
        lookup.assert_not_called()

    def test_path_fallback_and_configuration_failure_status(self) -> None:
        with (
            patch.object(adapter.sys, "argv", ["adapter", "--version"]),
            patch.dict(os.environ, {}, clear=True),
            patch.object(adapter.shutil, "which", return_value="/existing/cargo"),
            patch.object(
                adapter.subprocess, "run", return_value=SimpleNamespace(returncode=0)
            ) as execute,
        ):
            expect_equal(adapter.main(), 0)
        execute.assert_called_once_with(["/existing/cargo", "--version"], check=False)
        stderr = io.StringIO()
        with (
            patch.object(adapter.sys, "argv", ["adapter", "check", "-Fipopt"]),
            patch.object(adapter.subprocess, "run") as execute,
            redirect_stderr(stderr),
        ):
            expect_equal(adapter.main(), 2)
        execute.assert_not_called()
        expect_equal("selected --manifest-path" in stderr.getvalue(), True)

    def test_process_status_and_output_preserved_without_running_cargo(self) -> None:
        fake_cargo = Path(self.directory.name) / "fake-cargo"
        fake_cargo.write_text(
            f"#!{sys.executable}\nimport sys\nprint('native stdout')\nprint('native stderr', file=sys.stderr)\nsys.exit(23)\n"
        )
        fake_cargo.chmod(493)
        result = subprocess.run(
            [sys.executable, adapter.__file__, "--version"],
            env={**os.environ, "CARGO": str(fake_cargo)},
            capture_output=True,
            text=True,
            check=False,
        )
        expect_equal(result.returncode, 23)
        expect_equal(result.stdout, "native stdout\n")
        expect_equal(result.stderr, "native stderr\n")


if __name__ == "__main__":
    unittest.main()
