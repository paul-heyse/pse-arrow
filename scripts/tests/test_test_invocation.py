# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent selection/configuration controls for the shared native request."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import tomlkit

from scripts import native_tests, test_invocation, test_run


class InvocationTests(unittest.TestCase):
    def test_python_aliases_and_opaque_selection_share_the_interpreter(self) -> None:
        for option in (["-m", "component"], ["--markexpr=component"], ["-mcomponent"]):
            request = test_invocation.python(
                [*option, "-k", "selected", "--", "-m", "opaque.py"]
            )
            self.assertEqual(request.marker, "component")
            self.assertEqual(request.forwarded, ("-k", "selected"))
            self.assertEqual(request.opaque, ("--", "-m", "opaque.py"))
        with self.assertRaisesRegex(ValueError, "unsupported pytest scope"):
            test_invocation.python(["--unknown-selection"])

    def test_profile_aliases_and_environment_have_one_effective_identity(self) -> None:
        expected = native_tests.rust_selection_identity(
            ["--profile", "local", "-E", "test(real)"]
        )
        for arguments, environment in (
            (["-P", "local", "--filterset=test(real)"], {}),
            (["-Plocal", "--filter-expr", "test(real)"], {}),
            (["-Etest(real)"], {"NEXTEST_PROFILE": "local"}),
            (["-P", "local", "-E", "test(real)"], {"NEXTEST_PROFILE": "other"}),
        ):
            with self.subTest(arguments=arguments, environment=environment):
                self.assertEqual(
                    native_tests.rust_selection_identity(arguments, environment),
                    expected,
                )

    def test_opaque_arguments_cannot_change_profiles_or_compilation(self) -> None:
        request = test_invocation.rust(
            ["-p", "pure", "--", "-P", "opaque", "--features", "opaque"], {}
        )
        self.assertEqual(request.nextest_profile, "default")
        self.assertEqual(request.opaque, ("--", "-P", "opaque", "--features", "opaque"))
        self.assertNotIn("opaque", request.graph())
        self.assertEqual(request.listing()[-5:], list(request.opaque))

    def test_unsupported_scope_and_missing_values_refuse_purely(self) -> None:
        for arguments in (
            ["--future-selection", "x"],
            ["-P"],
            ["--features", "--"],
            ["--config-file="],
        ):
            with self.subTest(arguments=arguments), self.assertRaises(ValueError):
                test_invocation.rust(arguments, {})

    def test_historical_target_remap_survives_native_report_configuration(self) -> None:
        request = test_invocation.rust(
            [
                "--binaries-metadata",
                "/archive/binaries.json",
                "--cargo-metadata",
                "/archive/cargo.json",
                "--target-dir-remap",
                "/extracted/target",
                "--workspace-remap",
                "/extracted/workspace",
                "-E",
                "test(real)",
            ],
            {},
        )
        self.assertEqual(request.values("--target-dir-remap"), ("/extracted/target",))
        self.assertIn("--target-dir-remap", request.listing())
        self.assertIn("--target-dir-remap", request.preparation()[0])
        self.assertNotIn("--target-dir-remap", request.graph())

    def test_report_copy_preserves_custom_repository_and_lower_priority_tool_config(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repository = root / "repository.toml"
            original = '# owner\n[profile."special profile"]\nretries = 2 # retained\n[profile."special profile".junit]\npath = "prior.xml" # report\n'
            repository.write_text(original)
            tool = root / "tool.toml"
            tool.write_text('[profile."special profile"]\nfail-fast = false\n')
            invocation = root / "invocation.json"
            command, report = test_run.rust_report_command(
                invocation,
                [
                    "cargo",
                    "nextest",
                    "run",
                    "--config-file",
                    str(repository),
                    "--tool-config-file",
                    f"base:{tool}",
                    "--",
                    "--config-file=opaque",
                ],
                {"NEXTEST_PROFILE": "special profile"},
            )
            self.assertEqual(repository.read_text(), original)
            self.assertIn(f"base:{tool}", command)
            self.assertEqual(command[-2:], ["--", "--config-file=opaque"])
            copied = invocation.with_suffix(".toml").read_text()
            self.assertIn("# owner", copied)
            self.assertIn("retries = 2 # retained", copied)
            profile = tomlkit.parse(copied)["profile"]["special profile"]
            self.assertEqual(profile["junit"]["path"], str(report))
            self.assertEqual(
                native_tests.rust_selection_identity(command[3:]),
                native_tests.rust_selection_identity(
                    [
                        "-P",
                        "special profile",
                        "--config-file",
                        str(repository),
                        "--tool-config-file",
                        f"base:{tool}",
                        "--",
                        "--config-file=opaque",
                    ]
                ),
            )


if __name__ == "__main__":
    unittest.main()
