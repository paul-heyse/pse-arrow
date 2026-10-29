# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise ADR recipe argument transport in an isolated directory."""

# ruff: noqa: PT009

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts.tests import assessment_fixture

ROOT = Path(__file__).resolve().parents[2]


class RecipeArgumentTests(unittest.TestCase):
    def test_python_stubs_accept_explicit_artifact_without_installed_extension(
        self,
    ) -> None:
        artifact = "library with spaces; $HOME `pwd` $(touch injected).so"
        for recipe, flags in [
            ("python-stubs", []),
            ("python-stubs-check", ["--check"]),
        ]:
            with (
                self.subTest(recipe=recipe),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory)
                environment = self.fixture(root)
                native = root / "scripts/cargo-bin/cargo-pse-correctness"
                native.parent.mkdir()
                native.write_text(
                    "#!" + sys.executable + "\n"
                    "import json, pathlib, sys\n"
                    "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
                    "raise SystemExit(7)\n"
                )
                native.chmod(0o700)
                result = subprocess.run(
                    ["just", recipe, "--extension", artifact],
                    cwd=root,
                    env=environment,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(result.returncode, 7, result.stderr)
                self.assertEqual(
                    json.loads((root / "arguments.json").read_text()),
                    [
                        "run",
                        "--bin",
                        "xtask",
                        "-p",
                        "xtask",
                        "--no-default-features",
                        "--locked",
                        "--",
                        "python-stubs",
                        *flags,
                        "--extension",
                        artifact,
                    ],
                )
                self.assertFalse((root / "injected").exists())

    def test_native_unit_launches_choose_outer_class_and_preserve_filter_arguments(
        self,
    ) -> None:
        literal = 'test(a space; $HOME `pwd` $(touch injected) "quote")'
        cases = [
            (
                "unit-package",
                ["pse-runtime", literal],
                ["unit", "pse-runtime", literal],
            ),
            ("unit-libraries", [literal], ["workspace", literal]),
            (
                "unit-native-selected",
                [literal],
                ["--features", "pse-py/native-solvers", "workspace", literal],
            ),
            (
                "unit-native-package",
                ["pse-runtime", "pse-runtime/solver-kinsol", literal],
                [
                    "--execution-effects",
                    "canonical",
                    "--features",
                    "pse-runtime/solver-kinsol",
                    "unit",
                    "pse-runtime",
                    literal,
                ],
            ),
        ]
        for recipe, arguments, expected in cases:
            with (
                self.subTest(recipe=recipe),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory)
                environment = self.fixture(root)
                (root / "scripts/select.py").write_text(
                    "import json, pathlib, sys\n"
                    "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
                )
                result = subprocess.run(
                    ["just", recipe, *arguments],
                    cwd=root,
                    env=environment,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(
                    json.loads((root / "arguments.json").read_text()), expected
                )
                self.assertEqual(
                    (root / "wrapper.txt").read_text().splitlines()[:2],
                    ["--recipe-default-class", "exclusive"],
                )
                self.assertFalse((root / "injected").exists())

    def test_native_capability_units_delegate_literal_arguments_and_status(
        self,
    ) -> None:
        literal = 'test(a space; $HOME `pwd` $(touch injected) "quote")'
        for expression, expected, status in (
            ("root_response", "test(root_response)", 0),
            (literal, literal, 7),
        ):
            with (
                self.subTest(expression=expression),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory)
                environment = self.fixture(root)
                native = root / "scripts/cargo-bin/cargo-pse-correctness"
                native.parent.mkdir()
                native.write_text(
                    "#!" + sys.executable + "\n"
                    "import json, pathlib, sys\n"
                    "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
                    f"raise SystemExit({status})\n"
                )
                native.chmod(0o700)
                extra = ["--test-threads", "16", "-p", "other-package"]
                result = subprocess.run(
                    [
                        "just",
                        "unit-native-capability-package",
                        "pse-runtime",
                        "pse-runtime/solver-kinsol",
                        "compiler,klu",
                        expression,
                        *extra,
                    ],
                    cwd=root,
                    env=environment,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(result.returncode, status, result.stderr)
                self.assertEqual(
                    json.loads((root / "arguments.json").read_text()),
                    [
                        "nextest",
                        "run",
                        "--no-fail-fast",
                        "-p",
                        "pse-runtime",
                        "--lib",
                        "--locked",
                        "--features",
                        "pse-runtime/solver-kinsol",
                        "-E",
                        expected,
                        *extra,
                    ],
                )
                wrappers = (root / "wrapper.txt").read_text().splitlines()
                self.assertIn("--native=compiler,klu", wrappers)
                self.assertNotIn("exclusive", wrappers)
                self.assertFalse((root / "injected").exists())

    def test_py_test_dispatches_directly_with_store_and_literal_arguments(self) -> None:
        literal = 'a space; $HOME `pwd` $(touch injected) "quote"'
        extra = ["-m", "component", "-k", literal, "--junitxml=chosen report.xml"]
        for status in [0, 7, 143]:
            with (
                self.subTest(status=status),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory)
                env = self.fixture(root)
                (root / ".venv/bin").mkdir(parents=True)
                (root / ".venv/bin/python").symlink_to(sys.executable)
                env["UV_PROJECT_ENVIRONMENT"] = str(root / ".venv")
                env["PSE_SURREAL_STATE"] = str(root / "selected-store")
                env["SELECTED_TEST_STATUS"] = str(status)
                (root / "scripts/python_extension.py").write_text(
                    "import pathlib, sys\n"
                    "assert sys.argv[1:] == ['sync', 'auto']\n"
                    "pathlib.Path('validated-build.txt').write_text('requested')\n"
                )
                (root / "scripts/python_tests.py").write_text(
                    "import json, os, pathlib, sys\n"
                    "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
                    "pathlib.Path('store.txt').write_text(os.environ['PSE_SURREAL_STATE'])\n"
                    "raise SystemExit(int(os.environ['SELECTED_TEST_STATUS']))\n"
                )
                result = subprocess.run(
                    ["just", "py-test", *extra],
                    cwd=root,
                    env=env,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(result.returncode, status, result.stderr)
                self.assertEqual(
                    (root / "validated-build.txt").read_text(), "requested"
                )
                self.assertEqual(
                    json.loads((root / "arguments.json").read_text()), extra
                )
                self.assertEqual(
                    (root / "store.txt").read_text(), env["PSE_SURREAL_STATE"]
                )
                self.assertIn(
                    "--store\n--\n" + str(root / ".venv/bin/python"),
                    (root / "wrapper.txt").read_text(),
                )
                self.assertFalse((root / "injected").exists())

    def fixture(self, root: Path) -> dict[str, str]:
        """Use the complete Just composition, replacing only external effects."""
        (root / "justfile").write_text((ROOT / "justfile").read_text())
        (root / ".python-version").write_text((ROOT / ".python-version").read_text())
        (root / "scripts").mkdir()
        # A regular fixture package must win even when the caller's PYTHONPATH
        # contains the production checkout's regular scripts package.
        (root / "scripts/__init__.py").write_text("")
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
        return {
            **os.environ,
            "PATH": f"{bin_dir}:{os.environ['PATH']}",
            "PSE_TEST_RESOURCE_STATE": str(root / "test-resources"),
        }

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

    def test_composed_result_reads_spaced_path_and_literal_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = self.fixture(root)
            env.pop("UV_PROJECT_ENVIRONMENT", None)
            (root / ".venv/bin").mkdir(parents=True)
            (root / ".venv/bin/python").symlink_to(sys.executable)
            for module in (ROOT / "scripts").glob("*.py"):
                if module.name not in {"__init__.py", "solver-images.py"}:
                    (root / "scripts" / module.name).symlink_to(module)
            run = root / "run with spaces"
            run.mkdir()
            gate = "literal $(touch injected); $HOME"
            assessment_fixture.publish(
                run,
                {
                    "scope": [{"name": gate, "role": "required"}],
                    "checks": [],
                },
            )
            result = subprocess.run(
                ["just", "result", str(run), "--gate", gate, "--json"],
                cwd=root,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            report = json.loads(result.stdout)
            self.assertEqual(report["run_path"], str(run))
            self.assertEqual(report["unattempted_scope"][0]["name"], gate)
            self.assertFalse(report["assessment"]["complete"])
            self.assertFalse((root / "injected").exists())

    def test_adr_arguments_reach_script_unchanged(self) -> None:
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
                root = Path(directory)
                environment = self.fixture(root)
                (root / "scripts/adr.py").write_text(
                    "import json, pathlib, sys\n"
                    "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
                )
                subprocess.run(
                    ["just", recipe, *arguments],
                    cwd=root,
                    env=environment,
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
