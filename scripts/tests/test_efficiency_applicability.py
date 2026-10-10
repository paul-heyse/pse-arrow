# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Successful current observations cannot survive changed consumed premises."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from dataclasses import asdict
from pathlib import Path
from unittest.mock import patch

from scripts import test_resources, validation, validation_receipts, validation_scope
from scripts.validation_scope import Gate


class ApplicabilityTests(unittest.TestCase):
    def test_successful_current_command_refuses_changed_consumed_premises(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = (
                "scripts/test_run.py",
                "scripts/test_resources.py",
                "scripts/host_admission.py",
                "scripts/surreal_server.py",
                "benches/benches/modeling_preparation.rs",
            )
            for name in files:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("original")

            def snapshot() -> dict[str, str]:
                return {name: validation_receipts.digest(root / name) for name in files}

            real_execute = validation.execute

            def execute(
                root: Path,
                output: Path,
                name: str,
                _command: list[str],
                env: dict[str, str],
            ) -> dict:
                return real_execute(
                    root,
                    output,
                    name,
                    [sys.executable, "-c", "print('bounded observation')"],
                    env,
                )

            with (
                patch.object(
                    test_resources, "registry", return_value=root / "registry"
                ),
                patch.object(
                    validation,
                    "provenance",
                    side_effect=lambda *_: (snapshot(), root / "target", []),
                ),
                patch.object(validation, "sources", side_effect=lambda *_: snapshot()),
                patch.object(validation, "command_env", return_value={}),
                patch.object(validation, "execute", side_effect=execute),
            ):
                for scope in ("rust-product", "python-product"):
                    gate = Gate("bounded", input_scope=scope)
                    output = root / "build" / scope
                    output.mkdir(parents=True)
                    self.assertEqual(validation.run_gates(root, output, [gate]), 0)
                    report = json.loads((output / "checks.json").read_text())
                    self.assertEqual(report["checks"][0]["exit_code"], 0)
                    self.assertEqual(report["checks"][0]["status"], "passed")
                    declarations = [asdict(gate)]
                    environment = validation.relevant_environment({})

                    def reuse(
                        environment: dict[str, str],
                        output: Path = output,
                        declarations: list[dict] = declarations,
                    ) -> list[dict]:
                        return validation_receipts.reuse_checks(
                            output,
                            snapshot(),
                            environment,
                            declarations,
                            {"bounded"},
                            set(),
                            None,
                        )

                    self.assertEqual(len(reuse(environment)), 1)
                    for name in files:
                        with self.subTest(scope=scope, file=name):
                            path = root / name
                            path.write_text("changed execution premise")
                            with self.assertRaisesRegex(ValueError, "identical inputs"):
                                reuse(environment)
                            path.write_text("original")
                    for key in (
                        "UNO_DIR",
                        "PETSC_DIR",
                        "SUITESPARSE_INCLUDE_DIR",
                        "CC",
                        "CXX",
                        "CMAKE_TOOLCHAIN_FILE",
                        "CFLAGS",
                        "CXXFLAGS",
                        "BINDGEN_EXTRA_CLANG_ARGS",
                    ):
                        with (
                            self.subTest(scope=scope, selector=key),
                            self.assertRaisesRegex(ValueError, "identical inputs"),
                        ):
                            reuse(validation.relevant_environment({key: "changed"}))
                    # An older projection is never relabelled as current evidence.
                    old = output.with_name(scope + "-old-interpretation")
                    old.mkdir()
                    with patch.object(validation_scope, "INPUT_SCOPE_VERSION", 2):
                        self.assertEqual(validation.run_gates(root, old, [gate]), 0)
                    with self.assertRaisesRegex(ValueError, "identical inputs"):
                        validation_receipts.reuse_checks(
                            old,
                            snapshot(),
                            environment,
                            declarations,
                            {"bounded"},
                            set(),
                            None,
                        )

    def test_current_configuration_binds_file_contents_and_directory_generation(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first, second = root / "first", root / "second"
            first.mkdir()
            second.mkdir()
            selected = root / "current"
            selected.symlink_to(first, target_is_directory=True)
            toolchain = root / "toolchain.cmake"
            toolchain.write_text("first configuration")
            environment = {
                "UNO_DIR": str(selected),
                "CMAKE_TOOLCHAIN_FILE": str(toolchain),
            }
            before = validation.relevant_environment(environment)
            toolchain.write_text("second configuration")
            self.assertNotEqual(before, validation.relevant_environment(environment))
            toolchain.write_text("first configuration")
            selected.unlink()
            selected.symlink_to(second, target_is_directory=True)
            self.assertNotEqual(before, validation.relevant_environment(environment))

    def test_compiler_selection_resolves_current_search_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("first", "second"):
                folder = root / name
                folder.mkdir()
                compiler = folder / "cc"
                compiler.write_text("#!/bin/sh\nexit 0\n")
                compiler.chmod(0o700)
            environment = {"CC": "cc", "PATH": str(root / "first")}
            before = validation.relevant_environment(environment)
            # Both previous executables still exist and retain identical bytes.
            # The current configuration selects a different executable.
            after = validation.relevant_environment(
                {**environment, "PATH": str(root / "second")}
            )
            self.assertNotEqual(before, after)


if __name__ == "__main__":
    unittest.main()
