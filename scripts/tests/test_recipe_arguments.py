# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise ADR recipe argument transport in an isolated directory."""

# ruff: noqa: PT009

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class RecipeArgumentTests(unittest.TestCase):
    def fixture(self, root: Path) -> dict[str, str]:
        """Use the complete Just composition, replacing only external effects."""
        (root / "justfile").write_text((ROOT / "justfile").read_text())
        (root / ".python-version").write_text((ROOT / ".python-version").read_text())
        (root / "scripts").mkdir()
        (root / "scripts/pse-env").write_text(
            'printf "%s\\n" "$@" >> wrapper.txt\n'
            'while [[ "$1" != -- ]]; do shift; done\nshift\nexec "$@"\n'
        )
        (root / "scripts/pse-env").chmod(0o700)
        (root / "scripts/solver-images.py").write_text(
            "raise AssertionError('unrelated solver image lookup')\n"
        )
        bin_dir = root / "bin"
        bin_dir.mkdir()
        return {**os.environ, "PATH": f"{bin_dir}:{os.environ['PATH']}"}

    def test_composed_compile_arguments_and_outer_class(self) -> None:
        literal = 'a space; $HOME `pwd` $(touch injected) "quote"'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = self.fixture(root)
            # The recipe's external effect is now the validation composer,
            # rather than a direct Cargo call. Keep this transport fixture local
            # even when assessment supplies an absolute product environment.
            (root / ".venv/bin").mkdir(parents=True)
            (root / ".venv/bin/python").symlink_to(sys.executable)
            env["UV_PROJECT_ENVIRONMENT"] = str(root / ".venv")
            (root / "scripts/arrow_validation.py").write_text(
                "import json, pathlib, sys\n"
                "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
            )
            subprocess.run(
                [
                    "just",
                    "check-package",
                    "pse-structural",
                    "--message-format",
                    literal,
                ],
                cwd=root,
                env=env,
                check=True,
                capture_output=True,
            )
            self.assertEqual(
                json.loads((root / "arguments.json").read_text())[-2:],
                ["--message-format", literal],
            )
            self.assertEqual(
                (root / "wrapper.txt").read_text().splitlines()[:2],
                ["--recipe-default-class", "compile"],
            )
            self.assertFalse((root / "injected").exists())

    def test_every_variadic_binding_uses_positional_transport(self) -> None:
        definitions = json.loads(
            subprocess.check_output(
                ["just", "--dump", "--dump-format", "json"],
                cwd=ROOT,
            )
        )["recipes"]
        for name, recipe in definitions.items():
            variadic = [
                p["name"] for p in recipe["parameters"] if p["kind"] in {"star", "plus"}
            ]
            if not variadic:
                continue
            with self.subTest(recipe=name):
                self.assertIn("positional-arguments", recipe["attributes"])
                for parameter in variadic:
                    self.assertNotIn(
                        [["variable", parameter]],
                        [item for line in recipe["body"] for item in line],
                    )

    def test_codegen_rejects_modes_before_every_stage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = self.fixture(root)
            for arguments in [["--check"], ["--only", "registry"]]:
                result = subprocess.run(
                    ["just", "codegen", *arguments],
                    cwd=root,
                    env=env,
                    capture_output=True,
                    check=False,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((root / "wrapper.txt").exists())
                self.assertFalse((root / "arguments.json").exists())

    def test_doctor_script_interpreter_and_literal_arguments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = self.fixture(root)
            (root / "scripts/doctor.py").write_text(
                "import json, pathlib, sys\n"
                "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
            )
            for recipe, arguments in [
                ("doctor", ["spaced value"]),
                ("doctor-json", ["--format=json"]),
            ]:
                result = subprocess.run(
                    ["just", recipe, *(arguments if recipe == "doctor" else [])],
                    cwd=root,
                    env=env,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(
                    json.loads((root / "arguments.json").read_text()), arguments
                )
                self.assertIn("light", (root / "wrapper.txt").read_text().splitlines())

    def test_composed_activity_survives_unavailable_admission(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = self.fixture(root)
            (root / "scripts/pse-env").write_text("exit 125\n")
            for module in (ROOT / "scripts").glob("*.py"):
                if module.name not in {"activity.py", "solver-images.py"}:
                    (root / "scripts" / module.name).symlink_to(module)
            (root / "scripts/activity.py").write_text(
                (ROOT / "scripts/activity.py").read_text()
            )
            (root / "bin/systemctl").write_text(
                "#!/bin/sh\necho 'user manager unavailable' >&2\nexit 1\n"
            )
            (root / "bin/systemctl").chmod(0o700)
            result = subprocess.run(
                ["just", "activity", "--json"],
                cwd=root,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 1, result.stderr)
            report = json.loads(result.stdout)
            self.assertIn("unavailable", json.dumps(report))
            self.assertFalse((root / "wrapper.txt").exists())

    def test_composed_result_reads_spaced_path_and_literal_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = self.fixture(root)
            env.pop("UV_PROJECT_ENVIRONMENT", None)
            (root / ".venv/bin").mkdir(parents=True)
            (root / ".venv/bin/python").symlink_to(sys.executable)
            for module in (ROOT / "scripts").glob("*.py"):
                if module.name != "solver-images.py":
                    (root / "scripts" / module.name).symlink_to(module)
            run = root / "run with spaces"
            run.mkdir()
            gate = "literal $(touch injected); $HOME"
            (run / "checks.json").write_text(
                json.dumps(
                    {
                        "version": 5,
                        "scope": [{"name": gate, "role": "required"}],
                        "checks": [],
                        "complete": False,
                        "source_unchanged": True,
                        "baseline_failures": 0,
                        "provenance_errors": [],
                    }
                )
            )
            result = subprocess.run(
                ["just", "result", str(run), "--gate", gate, "--json"],
                cwd=root,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads(result.stdout)
            self.assertEqual(report["run_path"], str(run))
            self.assertEqual(report["unattempted_scope"][0]["name"], gate)
            self.assertFalse(report["assessment"]["complete"])
            self.assertFalse((root / "injected").exists())

    def test_adr_arguments_reach_script_unchanged(self) -> None:
        source = (ROOT / "justfile").read_text()
        title = 'Scope with "quotes", $HOME, `pwd`, and $(touch injected)'
        for recipe, arguments, expected in [
            (
                "adr-new",
                ["test-scope", "--title", title],
                ["new", "test-scope", "--title", title],
            ),
            (
                "adr-supersede",
                ["0050", "0051"],
                ["supersede", "0050", "0051"],
            ),
        ]:
            with (
                self.subTest(recipe=recipe),
                tempfile.TemporaryDirectory() as directory,
            ):
                match = re.search(
                    rf"(?m)^\[positional-arguments\]\n{recipe}[^\n]*:\n(?:[ \t]+[^\n]*\n)+",
                    source,
                )
                self.assertIsNotNone(match)
                if match is None:
                    raise AssertionError(f"missing positional recipe {recipe}")
                root = Path(directory)
                (root / "justfile").write_text(match.group())
                (root / "scripts").mkdir()
                (root / "scripts/adr.py").write_text(
                    "import json, pathlib, sys\n"
                    "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
                )
                subprocess.run(
                    ["just", recipe, *arguments],
                    cwd=root,
                    check=True,
                    capture_output=True,
                    text=True,
                )
                self.assertEqual(
                    json.loads((root / "arguments.json").read_text()), expected
                )
                self.assertFalse((root / "injected").exists())


if __name__ == "__main__":
    unittest.main()
