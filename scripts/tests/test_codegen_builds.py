# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise codegen recipe selections at their actual graph-assurance boundary."""
# ruff: noqa: PT009 -- standard-library process boundary controls

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts.arrow_validation import resolve
from scripts.tests.test_arrow_validation import Cargo, graph, package, unit

ROOT = Path(__file__).resolve().parents[2]


class CodegenBuildTests(unittest.TestCase):
    def test_recipes_preserve_selected_generators_features_and_explicit_binary(
        self,
    ) -> None:
        cases = {
            "codegen-relations-check": [
                ("relations", "physical-admission,force-validate")
            ],
            "codegen-rust-contracts": [("rust-contracts", None)],
            "codegen-rust-contracts-check": [("rust-contracts", None)],
            "codegen-physical": [("physical", "physical-admission,force-validate")],
            "codegen-physical-check": [
                ("physical", "physical-admission,force-validate")
            ],
            "codegen-python-check": [("python", None)],
            "codegen-docs-check": [("docs", None)],
            "codegen-surreal-check": [("surreal", None)],
            "codegen-bindgen-check": [("bindgen", "native-abi")],
            "codegen-schemas-check": [("schemas", "product-schemas,force-validate")],
            "codegen-all-check": [(None, "full-codegen,force-validate")],
            "codegen": [
                ("rust-contracts", None),
                ("surreal", None),
                (None, "full-codegen,force-validate"),
            ],
            "codegen-contracts": [
                ("rust-contracts", None),
                ("python", None),
                ("docs", None),
                ("surreal", None),
            ],
            "python-stubs": [("python-stubs", None)],
            "python-stubs-check": [("python-stubs", None)],
        }
        for recipe, expected in cases.items():
            with (
                self.subTest(recipe=recipe),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory)
                (root / "justfile").write_text((ROOT / "justfile").read_text())
                (root / ".python-version").write_text(
                    (ROOT / ".python-version").read_text()
                )
                (root / ".venv/bin").mkdir(parents=True)
                (root / ".venv/bin/python").symlink_to(sys.executable)
                scripts = root / "scripts"
                scripts.mkdir()
                (scripts / "__init__.py").write_text("")
                capabilities = scripts / "capabilities"
                capabilities.mkdir()
                (capabilities / "__init__.py").write_text("")
                (capabilities / "reference.py").write_text(
                    "from pathlib import Path\n"
                    "Path('reference-published').write_text('published')\n"
                )
                (scripts / "pse-env").write_text(
                    'printf "%s\\n" "$@" >> environment-arguments.txt\n'
                    'while [[ "$1" != -- ]]; do shift; done\nshift\nexec "$@"\n'
                )
                (scripts / "pse-env").chmod(0o700)
                (scripts / "doctor.py").write_text("print('dev')\n")
                (scripts / "arrow_validation.py").write_text(
                    "import json, pathlib, sys\n"
                    "with pathlib.Path('commands.jsonl').open('a') as output:\n"
                    "    output.write(json.dumps(sys.argv[1:]) + '\\n')\n"
                )
                child = subprocess.run(
                    ["just", recipe],
                    cwd=root,
                    env={**os.environ, "UV_PROJECT_ENVIRONMENT": str(root / ".venv")},
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(child.returncode, 0, child.stderr)
                self.assertEqual(
                    (root / "reference-published").exists(),
                    recipe in {"codegen", "codegen-contracts"},
                )
                if recipe == "codegen-all-check":
                    environment_arguments = (
                        (root / "environment-arguments.txt").read_text().splitlines()
                    )
                    self.assertEqual(
                        environment_arguments[:3],
                        [
                            "--recipe-default-class",
                            "compile",
                            "--native=compiler,solver",
                        ],
                    )
                commands = [
                    json.loads(line)
                    for line in (root / "commands.jsonl").read_text().splitlines()
                ]
                self.assertEqual(len(commands), len(expected))
                for command, (generator, features) in zip(
                    commands, expected, strict=True
                ):
                    self.assertEqual(command[:3], ["run", "--bin", "xtask"])
                    self.assertEqual(command[command.index("-p") + 1], "xtask")
                    self.assertIn("--locked", command)
                    args = command[command.index("--") + 1 :]
                    if generator == "python-stubs":
                        self.assertEqual(args[0], generator)
                    elif generator is not None:
                        self.assertEqual(args[:3], ["codegen", "--only", generator])
                    else:
                        self.assertEqual(
                            args,
                            [
                                "codegen",
                                *(["--check"] if recipe.endswith("-check") else []),
                            ],
                        )
                    if features is None:
                        self.assertNotIn("--features", command)
                    else:
                        self.assertEqual(
                            command[command.index("--features") + 1], features
                        )
                    if recipe.endswith("-check"):
                        self.assertIn("--check", args)

    def test_bootstrap_graph_keeps_only_selected_xtask_binary(self) -> None:
        cargo = Cargo(
            [
                package("xtask", "force-validate"),
                package("arrow-array", "force_validate"),
            ],
            graph([unit("xtask", [1]), unit("arrow-array")]),
            graph(
                [
                    unit("xtask", [1], features=["force-validate"]),
                    unit("arrow-array", features=["force_validate"]),
                ]
            ),
        )
        command = [
            "cargo",
            "run",
            "--bin",
            "xtask",
            "-p",
            "xtask",
            "--no-default-features",
            "--locked",
            "--",
            "codegen",
            "--only",
            "rust-contracts",
        ]
        resolved = resolve(command, run=cargo)
        self.assertEqual(resolved.packages, ["xtask"])
        self.assertEqual(resolved.command[-3:], ["codegen", "--only", "rust-contracts"])
        self.assertNotIn("pse-relations/force-validate", resolved.command)
