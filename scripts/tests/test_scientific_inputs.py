# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Invocation capture and conservative scientific dependency reach controls."""

# ruff: noqa: PT009, PT027 -- stdlib functional controls
from __future__ import annotations

import contextlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import affected, coverage_correspondence
from scripts import scientific_inputs as inputs


class ScientificInputTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.destination = self.root / "captured.json"
        self.declaration = {
            "package_inventory": "packages/reference",
            "families": [
                {"name": "math", "consumers": ["pse-compiler"]},
                {"name": "flow", "consumers": ["pse-runtime"]},
            ],
        }
        # A small Rust-capture wire artifact. Dependency interpretation remains
        # owned by the real loader; the Python adapter only consumes its facts.
        self.capsule = {
            "schema": "scientific-inputs-v1",
            "files": {
                "packages/reference/a/models/law.pse": list(b"old law"),
                "packages/reference/a/data/table.parquet": [1, 2, 3],
                "packages/reference/shared/models/imported.pse": list(b"import"),
                "packages/reference/b/models/flow.pse": list(b"flow"),
                "tests/fixtures/scientific/substitute.pse": list(b"synthetic"),
            },
            "families": {
                "math": {
                    "packages": ["packages/reference/a", "packages/reference/shared"],
                    "documents": ["packages/reference/a/models/law.pse"],
                    "substitutions": ["tests/fixtures/scientific/substitute.pse"],
                    "projection": "synthetic-declarations",
                },
                "flow": {
                    "packages": ["packages/reference/b", "packages/reference/shared"],
                    "documents": ["packages/reference/b/models/flow.pse"],
                    "substitutions": [],
                    "projection": "authentic-transport",
                },
            },
            "directories": {
                "packages/reference/a/models": ["law.pse"],
                "packages/reference/a/models/absent.pse": None,
            },
            "package_headers": [],
        }
        catalog = patch.object(inputs, "catalog", return_value=self.declaration)
        catalog.start()
        self.addCleanup(catalog.stop)

    @staticmethod
    def inventory(package: str, selected: bool = True) -> dict:
        return {
            "rust-suites": {
                "selected": {
                    "binary-id": f"{package}::lib",
                    "testcases": {
                        "control": {
                            "filter-match": {
                                "status": "matches" if selected else "mismatch"
                            }
                        }
                    },
                }
            }
        }

    def emit(self, destination: Path, _environment: dict, **_kwargs: object) -> Path:
        destination.write_text(json.dumps(self.capsule))
        return destination

    def test_selected_consumer_captures_once_and_binds_exact_retained_input(
        self,
    ) -> None:
        environment = {"BUILD_INPUT": "selected"}
        with patch.object(inputs, "capture", side_effect=self.emit) as capture:
            first = inputs.prepare(
                self.inventory("pse-compiler"), self.destination, environment
            )
            second = inputs.prepare(
                self.inventory("pse-compiler"), self.root / "unused.json", environment
            )
        capture.assert_called_once()
        self.assertEqual(first, {inputs.ROLE: str(self.destination)})
        self.assertEqual(second, first)
        self.assertEqual(environment, {"BUILD_INPUT": "selected", **first})
        self.assertFalse((self.root / "unused.json").exists())

    def test_unselected_or_unrelated_consumer_has_no_capture_or_environment_effect(
        self,
    ) -> None:
        for inventory in (
            self.inventory("pse-compiler", False),
            self.inventory("pse-ids"),
        ):
            environment = {
                inputs.ROLE: "/unused/producer/input",
                "BUILD_INPUT": "selected",
            }
            original = environment.copy()
            with (
                patch.object(inputs, "capture") as capture,
                patch.object(inputs, "read") as read,
            ):
                self.assertEqual(
                    inputs.prepare(inventory, self.destination, environment), {}
                )
            self.assertEqual(environment, original)
            capture.assert_not_called()
            read.assert_not_called()
            self.assertFalse(self.destination.exists())

    def test_supplied_capsule_is_validated_without_live_recapture(self) -> None:
        self.emit(self.destination, {})
        environment = {inputs.ROLE: str(self.destination)}
        # No source package exists in this temporary root. Only captured bytes
        # remain, including the old authored source and synthetic provenance.
        with patch.object(
            inputs, "capture", side_effect=AssertionError("live recapture")
        ):
            self.assertEqual(
                inputs.prepare(
                    self.inventory("pse-runtime"), self.root / "unused", environment
                ),
                environment,
            )
        self.assertEqual(inputs.read(self.destination)["files"], self.capsule["files"])
        self.destination.write_text('{"schema":"other"}')
        with self.assertRaises(ValueError):
            inputs.prepare(
                self.inventory("pse-runtime"), self.root / "unused", environment
            )

    def test_python_controller_covers_targeted_consumers_and_parent_directories(
        self,
    ) -> None:
        selections = [
            [],
            ["."],
            [str(inputs.ROOT)],
            [str(inputs.ROOT.parent)],
            ["python/pse/tests"],
            [str(inputs.ROOT / "python")],
            ["python/pse/tests/test_modeling_kernel.py::test_reference"],
            [str(inputs.ROOT / "python/pse/tests/test_studies.py")],
            ["python/pse/tests/test_plan14_acceptance.py"],
            ["python/pse/parity/tests/test_flash.py"],
            ["--parity"],
        ]
        for arguments in selections:
            environment = {}
            with (
                self.subTest(arguments=arguments),
                patch.object(
                    inputs,
                    "capture",
                    side_effect=lambda *_: self.emit(self.destination, {}),
                ) as capture,
            ):
                inputs.prepare_python(arguments, environment)
                inputs.prepare_python(arguments, environment)
                capture.assert_called_once()
            self.assertEqual(environment, {inputs.ROLE: str(self.destination)})
        with patch.object(inputs, "capture") as capture:
            inputs.prepare_python(["scripts/tests/test_scientific_inputs.py"], {})
            inputs.prepare_python(["python/pse/tests/test_codec.py"], {})
        capture.assert_not_called()

    def test_python_controller_validates_explicit_capsule_without_recapture(
        self,
    ) -> None:
        self.destination.write_text('{"schema":"other"}')
        environment = {inputs.ROLE: str(self.destination)}
        with (
            patch.object(inputs, "capture") as capture,
            self.assertRaises(ValueError),
        ):
            inputs.prepare_python(
                ["python/pse/tests/test_modeling_kernel.py"], environment
            )
        capture.assert_not_called()

    def test_run_controller_forwards_literal_arguments_and_prepared_environment(
        self,
    ) -> None:
        arguments = [
            "python",
            "-m",
            "pse.conformance",
            "--report-dir",
            "a path;literal",
        ]
        with (
            patch.object(sys, "argv", ["scientific_inputs", "run", "--", *arguments]),
            patch.dict(inputs.os.environ, {"SELECTED": "value"}, clear=True),
            patch.object(
                inputs,
                "capture",
                side_effect=lambda *_: self.emit(self.destination, {}),
            ) as capture,
            patch.object(
                inputs.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(arguments, 9),
            ) as run,
        ):
            self.assertEqual(inputs.main(), 9)
        capture.assert_called_once()
        run.assert_called_once_with(
            arguments,
            env={"SELECTED": "value", inputs.ROLE: str(self.destination)},
            check=False,
        )

    def test_run_controller_reuses_supplied_capsule(self) -> None:
        self.emit(self.destination, {})
        with (
            patch.object(sys, "argv", ["scientific_inputs", "run", "consumer"]),
            patch.dict(
                inputs.os.environ, {inputs.ROLE: str(self.destination)}, clear=True
            ),
            patch.object(inputs, "capture") as capture,
            patch.object(
                inputs.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as run,
        ):
            self.assertEqual(inputs.main(), 0)
        capture.assert_not_called()
        self.assertEqual(
            run.call_args.kwargs["env"][inputs.ROLE], str(self.destination)
        )

    def test_capture_runs_loader_once_with_selected_environment_and_validates_its_output(
        self,
    ) -> None:
        environment = {"BUILD_INPUT": "selected"}

        def tool(command: list[str], **kwargs: object) -> subprocess.CompletedProcess:
            self.assertEqual(kwargs["cwd"], self.root)
            self.assertEqual(kwargs["env"], environment)
            self.assertTrue(kwargs["check"])
            self.assertIn("physical-admission,force-validate", command)
            self.assertEqual(
                command[-3:],
                [
                    "scientific-inputs",
                    str(self.root / inputs.CATALOG),
                    str(self.destination),
                ],
            )
            self.emit(self.destination, environment)
            return subprocess.CompletedProcess(command, 0)

        with patch.object(inputs.subprocess, "run", side_effect=tool) as run:
            self.assertEqual(
                inputs.capture(self.destination, environment, root=self.root),
                self.destination,
            )
            with self.assertRaises(ValueError):
                inputs.capture(self.destination, environment, root=self.root)
        self.assertEqual(run.call_count, 1)
        self.destination.unlink()
        with (
            patch.object(
                inputs.subprocess,
                "run",
                side_effect=lambda *_a, **_k: self.destination.write_text(
                    '{"schema":"other"}'
                ),
            ),
            self.assertRaises(ValueError),
        ):
            inputs.capture(self.destination, environment, root=self.root)

    def test_read_refuses_unknown_schema_and_missing_dependency_maps(self) -> None:
        for value, error in [
            ({"schema": "other"}, ValueError),
            (
                {"schema": "scientific-inputs-v1", "files": [], "families": {}},
                TypeError,
            ),
            ({"schema": "scientific-inputs-v1", "files": {}}, TypeError),
        ]:
            with self.subTest(value=value), self.assertRaises(error):
                self.destination.write_text(json.dumps(value))
                inputs.read(self.destination)

    def test_affected_uses_captured_import_data_and_synthetic_edges(self) -> None:
        cases = [
            ("packages/reference/a/models/law.pse", ["pse-compiler"]),
            ("packages/reference/a/data/table.parquet", ["pse-compiler"]),
            (
                "packages/reference/shared/models/imported.pse",
                ["pse-compiler", "pse-runtime"],
            ),
            ("tests/fixtures/scientific/substitute.pse", ["pse-compiler"]),
        ]
        for path, consumers in cases:
            with (
                self.subTest(path=path),
                patch.object(inputs, "capture", side_effect=self.emit) as capture,
            ):
                result = inputs.affected([("M", path)], root=self.root)
                self.assertEqual(result["consumers"], consumers)
                self.assertEqual(result["unknown"], [])
                capture.assert_called_once()

    def test_new_deleted_missing_lookup_and_unknown_package_edges_broaden(self) -> None:
        paths = [
            ("A", "packages/reference/a/models/new.pse"),
            ("D", "packages/reference/a/models/deleted.pse"),
            ("A", "packages/reference/a/models/absent.pse"),
            ("M", "packages/reference/a/package.toml"),
            ("M", "packages/reference/unknown/models/law.pse"),
        ]
        for status, path in paths:
            with (
                self.subTest(status=status, path=path),
                patch.object(inputs, "capture", side_effect=self.emit),
            ):
                result = inputs.affected([(status, path)], root=self.root)
                self.assertEqual(result["consumers"], ["pse-compiler", "pse-runtime"])
                self.assertEqual(result["unknown"], [path])
        with patch.object(
            inputs, "capture", side_effect=subprocess.CalledProcessError(1, "capture")
        ):
            result = inputs.affected([("D", paths[1][1])], root=self.root)
            self.assertEqual(result["consumers"], ["pse-compiler", "pse-runtime"])
            self.assertIn("reason", result)
        with patch.object(inputs, "capture") as capture:
            self.assertEqual(
                inputs.affected([("M", "docs/example.md")], root=self.root),
                {"consumers": [], "families": [], "unknown": []},
            )
            capture.assert_not_called()

    def test_coverage_overlay_cannot_remove_scientific_dependency_selection(
        self,
    ) -> None:
        package_roots = {
            "pse-compiler": Path("crates/pse-compiler"),
            "pse-runtime": Path("crates/pse-runtime"),
        }
        output = io.StringIO()
        with (
            patch.object(inputs, "capture", side_effect=self.emit),
            patch.object(
                affected,
                "changed_files",
                return_value=[("M", "packages/reference/a/models/law.pse")],
            ),
            patch.object(
                affected,
                "workspace_packages",
                return_value=(package_roots, {"pse-compiler": {"pse-runtime"}}),
            ),
            patch.object(affected, "compose", side_effect=lambda command: command),
            patch.object(
                coverage_correspondence,
                "inspect",
                return_value={"status": "available", "observed_tests": []},
            ),
            contextlib.redirect_stdout(output),
        ):
            self.assertEqual(
                affected.main(["--json", "--coverage", str(self.root / "coverage")]), 0
            )
        result = json.loads(output.getvalue())
        self.assertEqual(result["selected_packages"], ["pse-compiler", "pse-runtime"])
        self.assertEqual(result["scientific_inputs"]["consumers"], ["pse-compiler"])
        self.assertIn("rdeps(pse-compiler)", result["command"])
        self.assertEqual(result["coverage"]["observed_tests"], [])


if __name__ == "__main__":
    unittest.main()
