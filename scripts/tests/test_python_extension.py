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
from types import SimpleNamespace
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import arrow_validation, python_extension
from scripts.tests.test_arrow_validation import Cargo, graph, package, unit

if TYPE_CHECKING:
    from contextlib import AbstractContextManager


class PythonExtensionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.enterContext(
            patch.object(
                python_extension,
                "validated_maturin_command",
                side_effect=lambda command, *_args, **_kwargs: list(command),
            )
        )
        self.enterContext(
            patch.object(
                arrow_validation,
                "resolve",
                side_effect=lambda command, **_kwargs: SimpleNamespace(command=command),
            )
        )

    def test_diagnostic_wheel_preserves_large_full_debug_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            wheels = root / "build/diagnostic-taskdump/wheels"
            wheels.mkdir(parents=True)
            (wheels / "pse_arrow-fixture.whl").write_bytes(b"selected wheel")
            with (
                patch.object(python_extension, "ROOT", root),
                patch("scripts.build_environment.configure", return_value={}),
                patch.object(
                    python_extension.subprocess, "call", return_value=0
                ) as run,
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
                "package": str(package / "__init__.py"),
                "binary": str(binary),
                "prefix": str(selected),
                "features": ["force-validate"],
            }
            with patch.object(
                python_extension.subprocess,
                "check_output",
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


class InstallerGraphTests(unittest.TestCase):
    def cargo(self, *, validated: bool = True, host_validated: bool = True) -> Cargo:
        return Cargo(
            [
                package("pse-py", "force-validate"),
                package("arrow-array", "force_validate"),
            ],
            graph(
                [
                    unit("pse-py", [1, 2], target="_native"),
                    unit("arrow-array", platform="target"),
                    unit("arrow-array"),
                ]
            ),
            graph(
                [
                    unit(
                        "pse-py", [1, 2], features=["force-validate"], target="_native"
                    ),
                    unit(
                        "arrow-array",
                        features=["force_validate"] if validated else [],
                        platform="target",
                    ),
                    unit(
                        "arrow-array",
                        features=["force_validate"] if host_validated else [],
                    ),
                ]
            ),
        )

    def _graph_capture(self, cargo: Cargo) -> AbstractContextManager[object]:
        return patch.object(
            python_extension,
            "capture_build_graph",
            side_effect=lambda selected, *_args, **_kwargs: cargo(selected),
        )

    def test_installer_proves_target_and_host_leaves_and_preserves_selection(
        self,
    ) -> None:
        for mode in ("dev", "native"):
            with self.subTest(mode=mode):
                command = python_extension.develop_command(
                    python_extension.ROOT, mode, "producer"
                )
                cargo = self.cargo()
                with self._graph_capture(cargo):
                    actual = python_extension.validated_maturin_command(command, {})
                self.assertEqual(
                    actual, [*command, "--features", "pse-py/force-validate"]
                )
                selected = cargo.commands[1]
                self.assertIn("--lib", selected)
                self.assertEqual(
                    selected[selected.index("--manifest-path") + 1],
                    "crates/pse-py/Cargo.toml",
                )
                self.assertEqual(selected[selected.index("--profile") + 1], "producer")
                self.assertEqual(selected.count("--locked"), 1)
                self.assertEqual(
                    selected[selected.index("--features") + 1],
                    "force-validate,native-solvers"
                    if mode == "native"
                    else "force-validate",
                )
                self.assertFalse(
                    any("pse-relations/" in argument for argument in actual)
                )

    def test_missing_target_or_host_validation_refuses_before_installation(
        self,
    ) -> None:
        for target, host in ((False, True), (True, False)):
            with self.subTest(target=target, host=host):
                cargo = self.cargo(validated=target, host_validated=host)
                with (
                    self._graph_capture(cargo),
                    patch.object(python_extension.subprocess, "call") as install,
                    self.assertRaisesRegex(
                        arrow_validation.ValidationError, "lacks force_validate"
                    ),
                ):
                    python_extension.maturin(
                        python_extension.develop_command(
                            python_extension.ROOT, "dev", "dev"
                        )
                    )
                install.assert_not_called()

    def test_uncovered_consumer_refuses_before_installation(self) -> None:
        cargo = Cargo(
            [package("pse-py"), package("arrow-array", "force_validate")],
            graph([unit("pse-py", [1]), unit("arrow-array")]),
        )
        with (
            self._graph_capture(cargo),
            patch.object(python_extension.subprocess, "call") as install,
            self.assertRaisesRegex(arrow_validation.ValidationError, "no explicit"),
        ):
            python_extension.maturin(["maturin", "develop", "--locked"])
        install.assert_not_called()

    def test_unknown_maturin_scope_refuses_before_graph_or_install(self) -> None:
        for argument in ("--unknown-scope", "--manifest-path=other/Cargo.toml"):
            with (
                self.subTest(argument=argument),
                patch.object(python_extension, "capture_build_graph") as capture,
                patch.object(python_extension.subprocess, "call") as install,
                self.assertRaises(arrow_validation.ValidationError),
            ):
                python_extension.maturin(["maturin", "develop", argument])
            capture.assert_not_called()
            install.assert_not_called()

    def test_pure_installer_closure_adds_no_arrow_or_relation_target(self) -> None:
        cargo = Cargo([package("pse-py")], graph([unit("pse-py", target="_native")]))
        command = ["maturin", "develop", "--release", "--skip-install", "--locked"]
        with self._graph_capture(cargo):
            self.assertEqual(
                python_extension.validated_maturin_command(command, {}), command
            )
        self.assertIn("--release", cargo.commands[1])
        self.assertNotIn("--profile", cargo.commands[1])

    def test_diagnostic_installer_keeps_profile_interpreter_and_packaging(self) -> None:
        command = [
            "maturin",
            "build",
            "--locked",
            "--profile",
            "diagnostic-taskdump",
            "--interpreter",
            "selected/bin/python",
            "--features",
            "force-validate,diagnostic-taskdump",
            "--compression-method",
            "stored",
            "--compression-enable-large-file-support",
            "--out",
            "selected/wheels",
        ]
        cargo = self.cargo()
        with self._graph_capture(cargo):
            actual = python_extension.validated_maturin_command(command, {})
        self.assertEqual(actual[: len(command)], command)
        selected = cargo.commands[1]
        self.assertEqual(
            selected[selected.index("--profile") + 1], "diagnostic-taskdump"
        )
        self.assertNotIn("--interpreter", selected)
        self.assertNotIn("--compression-method", selected)

    def test_graph_failure_stops_sync_before_build_and_stub_generation(self) -> None:
        with (
            patch.object(python_extension.subprocess, "call", return_value=0) as child,
            patch.object(
                python_extension,
                "capture_build_graph",
                side_effect=arrow_validation.ValidationError("uncovered"),
            ),
            self.assertRaisesRegex(arrow_validation.ValidationError, "uncovered"),
        ):
            python_extension.sync("dev")
        self.assertEqual(child.call_count, 1)
        self.assertEqual(
            child.call_args.args[0][-4:],
            ["uv", "sync", "--locked", "--no-install-project"],
        )

    def test_graph_capture_uses_installers_checkout_environment_and_wrapper(
        self,
    ) -> None:
        environment = {"PYO3_PYTHON": "selected interpreter"}
        command = ["cargo", "build", "--lib"]
        wrapper = ["scripts/pse-env", "--native=solver", "--"]
        with patch.object(
            python_extension.subprocess,
            "run",
            return_value=SimpleNamespace(returncode=0, stdout=b"graph"),
        ) as run:
            self.assertEqual(
                python_extension.capture_build_graph(
                    command, environment, wrapper=wrapper
                ),
                b"graph",
            )
        run.assert_called_once_with(
            [*wrapper, *command],
            cwd=python_extension.ROOT,
            env=environment,
            capture_output=True,
            check=False,
        )
