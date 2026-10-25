# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual feature report and correctness build selection controls."""
# ruff: noqa: PT009, PT027 -- standard-library process boundary controls

from __future__ import annotations

import hashlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from types import SimpleNamespace
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import (
    arrow_validation,
    native_cache,
    native_operation,
    native_tests,
    python_extension,
)
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
                "prefix": str(root / ".venv"),
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

    def test_stage_composition_has_independent_operations_and_preserves_failures(
        self,
    ) -> None:
        stages = ["environment_ensure", "extension_prepare", "stubs_prepare", "capture"]
        for operation, expected in (
            ("environment", ["environment_ensure"]),
            ("prepare", ["extension_prepare"]),
            ("stubs", ["stubs_prepare"]),
            ("stubs-check", ["stubs_prepare"]),
            ("admit", ["capture"]),
            ("sync", stages),
        ):
            events = []
            with self.subTest(operation=operation):
                for name in stages:
                    self.enterContext(
                        patch.object(
                            python_extension,
                            name,
                            side_effect=lambda *_args, _name=name, _events=events, **_kwargs: (
                                _events.append(_name)
                                or (
                                    {
                                        "binary": "/observed/native.so",
                                        "sha256": "observed",
                                        "features": [],
                                    }
                                    if _name == "capture"
                                    else python_extension.StageResult(
                                        stage=(
                                            "environment"
                                            if _name == "environment_ensure"
                                            else "prepare"
                                            if _name == "extension_prepare"
                                            else "stubs"
                                        ),
                                        status="prepared",
                                    )
                                )
                            ),
                        )
                    )
                self.assertEqual(
                    python_extension.run_stages(
                        operation, "dev", "producer", {}
                    ).exit_code,
                    0,
                )
                self.assertEqual(events, expected)
        with (
            patch.object(
                python_extension,
                "environment_ensure",
                return_value=python_extension.StageResult(
                    stage="environment", status="reused"
                ),
            ),
            patch.object(
                python_extension,
                "extension_prepare",
                return_value=python_extension.StageResult(
                    stage="prepare", status="failed", exit_code=17
                ),
            ),
            patch.object(python_extension, "stubs_prepare") as stubs,
        ):
            self.assertEqual(
                python_extension.run_stages("sync", "dev", "dev", {}).exit_code, 17
            )
            stubs.assert_not_called()

    def test_stage_progress_uses_actual_selected_starts_and_results(self) -> None:
        events = []
        failed = python_extension.StageResult(
            stage="prepare", status="failed", exit_code=17
        )
        with patch.object(python_extension, "extension_prepare", return_value=failed):
            result = python_extension.run_stages(
                "prepare",
                "dev",
                "dev",
                {},
                on_stage=lambda stage, result: events.append((stage, result)),
            )
        self.assertEqual(events, [("extension", None), ("extension", failed)])
        self.assertEqual(result.exit_code, 17)
        events.clear()
        with self.assertRaises(ValueError):
            python_extension.run_stages(
                "unknown",
                "dev",
                "dev",
                {},
                on_stage=lambda stage, result: events.append((stage, result)),
            )
        self.assertFalse(events)

    def test_resolved_cli_prints_typed_result_and_maps_failure_without_relaunch(
        self,
    ) -> None:
        result = python_extension.PreparationResult(
            operation="prepare",
            mode="native",
            profile="producer",
            validated=False,
            stages=[
                python_extension.StageResult(
                    stage="prepare", status="failed", exit_code=17
                )
            ],
            exit_code=17,
        )
        output = io.StringIO()
        with (
            patch.object(python_extension, "run_stages", return_value=result) as run,
            patch.object(python_extension, "launch") as launch,
            redirect_stdout(output),
        ):
            code = python_extension.main(
                ["prepare", "native", "producer", "--resolved", "--interactive"]
            )
        self.assertEqual(code, 17)
        self.assertEqual(
            python_extension.PreparationResult.model_validate_json(output.getvalue()),
            result,
        )
        self.assertEqual(run.call_args.args[:3], ("prepare", "native", "producer"))
        self.assertFalse(run.call_args.kwargs["validated"])
        launch.assert_not_called()

    def test_environment_check_preserves_the_separately_installed_project(self) -> None:
        for statuses, count in (([0], 1), ([1, 0], 2), ([1, 17], 2)):
            with patch.object(
                python_extension.subprocess, "call", side_effect=statuses
            ) as run:
                self.assertEqual(
                    python_extension.environment_ensure({}).exit_code, statuses[-1]
                )
            self.assertEqual(run.call_count, count)
            for call in run.call_args_list:
                self.assertIn("--inexact", call.args[0])
                self.assertIn("--locked", call.args[0])
                self.assertIn("--no-install-project", call.args[0])
            self.assertIn("--check", run.call_args_list[0].args[0])

    def test_selected_environment_is_shared_by_interpreter_maturin_and_launcher(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            selected = Path(directory) / "alternate"
            environment = {
                "UV_PROJECT_ENVIRONMENT": str(selected),
                "VIRTUAL_ENV": str(selected),
            }
            with (
                patch("scripts.pse_env.compose", return_value=environment),
                patch.object(
                    python_extension.subprocess, "call", return_value=0
                ) as run,
            ):
                self.assertEqual(
                    python_extension.launch("sync", "native", "producer"), 0
                )
            command = run.call_args.args[0]
            self.assertIn(sys.executable, command)
            self.assertFalse((selected / "bin/python").exists())
            self.assertNotIn(str(selected / "bin/python"), command)
            self.assertIn("--native=solver,klu,isolation,uno,petsc", command)
            self.assertEqual(run.call_args.kwargs["env"], environment)
            for validated in (True, False):
                command = python_extension.develop_command(
                    python_extension.ROOT,
                    "native",
                    "producer",
                    environment=environment,
                    validated=validated,
                )
                self.assertEqual(command[0], str(selected / "bin/maturin"))
                self.assertEqual(
                    command[command.index("--features") + 1],
                    "force-validate,native-solvers" if validated else "native-solvers",
                )


class PreparationCurrencyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = self.enterContext(tempfile.TemporaryDirectory())
        self.root = Path(self.directory)
        self.enterContext(patch.object(python_extension, "ROOT", self.root))
        self.enterContext(patch.object(native_tests, "ROOT", self.root))
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        (self.root / "pyproject.toml").write_text(
            '[tool.maturin]\nbindings="pyo3"\nmanifest-path="crates/pse-py/Cargo.toml"\n'
        )
        for name, body in (
            ("crates/pse-py/src/lib.rs", "module"),
            ("xtask/src/codegen/python_stubs.rs", "generator"),
            ("python/pse/module.py", "pure python"),
        ):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(body)
        self.binary = self.root / "python/pse/_native.abi3.so"
        self.binary.write_bytes(b"installed artifact fixture")
        self.environment = {
            "UV_PROJECT_ENVIRONMENT": str(Path(sys.executable).parent.parent),
            "VIRTUAL_ENV": str(Path(sys.executable).parent.parent),
            "PATH": os.environ["PATH"],
        }
        self.enterContext(
            patch.object(
                python_extension, "tool_identity", return_value={"tool": "selected"}
            )
        )
        self.enterContext(
            patch.object(
                python_extension,
                "capture",
                return_value={
                    "binary": str(self.binary),
                    "sha256": "installed-transformed-bytes",
                    "features": ["force-validate"],
                },
            )
        )
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

    def inputs(self, stage: str = "python-extension") -> dict[str, object] | None:
        return python_extension.preparation_inputs(
            stage, "dev", "dev", self.environment, validated=True
        )

    def record_extension(self) -> None:
        python_extension.publish_receipt(
            python_extension.stage_receipt("extension", self.environment),
            {
                "inputs": self.inputs(),
                "artifact": python_extension.capture(self.environment),
            },
        )

    def currency(self) -> python_extension.ExtensionCurrency:
        return python_extension.currency_check(self.environment, mode="dev")

    def test_read_only_currency_reuses_owner_basis_without_install_or_publication(
        self,
    ) -> None:
        self.record_extension()
        with (
            patch.object(python_extension.subprocess, "call") as install,
            patch.object(python_extension, "publish_receipt") as publish,
        ):
            first = self.currency()
            second = self.currency()
        self.assertEqual(first.state, "current")
        self.assertEqual(first, second)
        assert first.artifact is not None
        self.assertEqual(first.artifact.sha256, "installed-transformed-bytes")
        self.assertTrue(first.receipt_recorded)
        self.assertIsNotNone(first.source_basis)
        install.assert_not_called()
        publish.assert_not_called()

    def test_installed_currency_uses_owner_profile_without_guessing_dev(self) -> None:
        with patch.object(python_extension.subprocess, "call", return_value=0):
            python_extension.extension_prepare("dev", "release", self.environment)
        self.assertEqual(self.currency().state, "stale")
        current = python_extension.currency_check_installed(
            self.environment, mode="dev"
        )
        self.assertEqual(current.state, "current")
        self.assertEqual(current.profile, "release")

    def test_installed_currency_rejects_malformed_profile_without_probe_or_import(
        self,
    ) -> None:
        self.record_extension()
        path = python_extension.stage_receipt("extension", self.environment)
        previous = python_extension.read_receipt(path)
        inputs = previous.get("inputs")
        assert isinstance(inputs, dict)
        for profile in (None, 7, "", "secret/path", "x" * 65):
            inputs["profile"] = profile
            python_extension.publish_receipt(path, previous)
            with (
                patch.object(python_extension, "preparation_inputs") as probe,
                patch.object(python_extension, "capture") as imported,
            ):
                result = python_extension.currency_check_installed(
                    self.environment, mode="dev"
                )
            self.assertEqual(result.state, "unknown")
            self.assertIsNone(result.profile)
            self.assertNotIn("secret/path", result.model_dump_json())
            probe.assert_not_called()
            imported.assert_not_called()

    def test_explicit_preparation_then_fresh_check_recovers_current_currency(
        self,
    ) -> None:
        with patch.object(
            python_extension.subprocess, "call", return_value=0
        ) as install:
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).status,
                "prepared",
            )
            self.assertEqual(self.currency().state, "current")
            (self.root / "crates/pse-py/src/lib.rs").write_text("new implementation")
            self.assertEqual(self.currency().state, "stale")
            self.assertEqual(install.call_count, 1)
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).status,
                "prepared",
            )
            self.assertEqual(self.currency().state, "current")
        self.assertEqual(install.call_count, 2)

    def test_same_schema_rust_change_is_stale_before_import_and_does_not_cache(
        self,
    ) -> None:
        self.record_extension()
        current = self.currency()
        path = self.root / "crates/pse-py/src/lib.rs"
        path.write_text("same schema; changed implementation")
        with patch.object(python_extension, "capture") as imported:
            stale = self.currency()
        self.assertEqual(stale.state, "stale")
        self.assertEqual(stale.cause, "inputs-changed")
        self.assertIsNotNone(stale.input_changes)
        assert stale.input_changes is not None
        self.assertEqual(stale.input_changes.sections, ("files",))
        self.assertEqual(stale.input_changes.source_keys, ("crates/pse-py/src/lib.rs",))
        self.assertTrue(stale.input_changes.complete)
        self.assertNotEqual(stale.source_basis, current.source_basis)
        imported.assert_not_called()
        path.write_text("module")
        self.assertEqual(self.currency().state, "current")

    def test_relevant_configuration_and_artifact_replacement_refuse_currency(
        self,
    ) -> None:
        self.record_extension()
        (self.root / ".cargo").mkdir()
        (self.root / ".cargo/config.toml").write_text(
            '[build]\nrustflags=["--cfg=changed"]'
        )
        self.assertEqual(self.currency().cause, "inputs-changed")
        (self.root / ".cargo/config.toml").unlink()
        with patch.object(
            python_extension,
            "capture",
            return_value={
                "binary": str(self.binary),
                "sha256": "replaced",
                "features": ["force-validate"],
            },
        ):
            replaced = self.currency()
        self.assertEqual(replaced.state, "stale")
        self.assertEqual(replaced.cause, "artifact-changed")

    def test_replacement_during_capture_refuses_the_import_window(self) -> None:
        self.record_extension()
        record = python_extension.read_receipt(
            python_extension.stage_receipt("extension", self.environment)
        )

        def changed(*_args: object, **_kwargs: object) -> object:
            self.binary.write_bytes(b"replacement artifact fixture")
            return record["artifact"]

        with patch.object(python_extension, "capture", side_effect=changed):
            result = self.currency()
        self.assertEqual(result.state, "stale")
        self.assertEqual(result.cause, "artifact-changed")

    def test_missing_and_unsupported_evidence_never_imports_or_claims_current(
        self,
    ) -> None:
        with (
            patch.object(python_extension, "preparation_inputs") as inputs,
            patch.object(python_extension, "capture") as imported,
        ):
            self.assertEqual(self.currency().state, "unknown")
            python_extension.publish_receipt(
                python_extension.stage_receipt("extension", self.environment),
                {
                    "inputs": {"schema": "future-inputs"},
                    "artifact": {
                        "binary": "/installed/native.so",
                        "sha256": "recorded",
                        "features": [],
                    },
                },
            )
            self.assertEqual(self.currency().state, "unsupported")
        inputs.assert_not_called()
        imported.assert_not_called()
        self.record_extension()
        self.environment["PYO3_CROSS_LIB_DIR"] = "private unsupported selection"
        result = self.currency()
        self.assertEqual(result.state, "unsupported")
        self.assertNotIn("private unsupported selection", result.model_dump_json())

    def test_unavailable_inputs_and_imports_redact_owner_errors(self) -> None:
        self.record_extension()
        with patch.object(
            python_extension,
            "preparation_inputs",
            side_effect=OSError("private tool path"),
        ):
            unavailable = self.currency()
        self.assertEqual(unavailable.state, "unknown")
        self.assertEqual(unavailable.cause, "inputs-unavailable")
        self.assertNotIn("private tool path", unavailable.model_dump_json())
        with patch.object(
            python_extension, "capture", side_effect=ValueError("private import input")
        ):
            unavailable = self.currency()
        self.assertEqual(unavailable.cause, "import-unavailable")
        self.assertNotIn("private import input", unavailable.model_dump_json())

    def test_native_currency_never_prepares_unadmitted_providers(self) -> None:
        self.record_extension()
        with (
            patch.object(native_operation, "remembered", return_value=None),
            patch.object(python_extension.subprocess, "check_output") as probe,
            patch.object(python_extension, "capture") as imported,
            patch.object(native_operation, "environment") as prepare,
        ):
            result = python_extension.currency_check(self.environment, mode="native")
        self.assertEqual(result.state, "unsupported")
        probe.assert_not_called()
        imported.assert_not_called()
        prepare.assert_not_called()

    def test_preparation_observation_disables_rustup_installation_without_changing_basis(
        self,
    ) -> None:
        original = python_extension.subprocess.check_output
        with patch.object(
            python_extension.subprocess, "check_output", wraps=original
        ) as probe:
            result = self.inputs()
        self.assertIsNotNone(result)
        assert result is not None
        self.assertTrue(probe.call_args_list)
        for call in probe.call_args_list:
            if call.args[0][0] != "git":
                self.assertEqual(call.kwargs["env"]["RUSTUP_AUTO_INSTALL"], "0")
        self.assertNotIn("RUSTUP_AUTO_INSTALL", self.environment)
        environment = result["environment"]
        assert isinstance(environment, dict)
        self.assertNotIn("RUSTUP_AUTO_INSTALL", environment)

    def test_real_source_currency_separates_pure_python_stub_and_rust_inputs(
        self,
    ) -> None:
        before = self.inputs()
        stub_before = self.inputs("python-stubs")
        (self.root / "python/pse/module.py").write_text("changed pure Python")
        self.assertEqual(before, self.inputs())
        self.assertEqual(stub_before, self.inputs("python-stubs"))
        (self.root / "xtask/src/codegen/python_stubs.rs").write_text(
            "changed generator"
        )
        self.assertEqual(before, self.inputs())
        self.assertNotEqual(stub_before, self.inputs("python-stubs"))
        (self.root / "crates/pse-py/src/lib.rs").write_text("changed Rust")
        self.assertNotEqual(before, self.inputs())
        previous = self.inputs()
        added = self.root / "crates/pse-py/src/new.rs"
        added.write_text("new member")
        self.assertNotEqual(previous, self.inputs())
        added.unlink()
        self.assertEqual(previous, self.inputs())

    def test_completed_install_receipt_skips_repeat_but_unknown_currency_prepares(
        self,
    ) -> None:
        with patch.object(
            python_extension.subprocess, "call", return_value=0
        ) as installer:
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(installer.call_count, 1)
            (self.root / "python/pse/module.py").write_text("changed pure Python")
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(installer.call_count, 1)
            (self.root / "crates/pse-py/src/lib.rs").write_text("changed Rust")
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(installer.call_count, 2)
            with patch.object(
                python_extension, "preparation_inputs", return_value=None
            ):
                for _ in range(2):
                    self.assertEqual(
                        python_extension.extension_prepare(
                            "dev", "dev", self.environment
                        ).exit_code,
                        0,
                    )
            self.assertEqual(installer.call_count, 4)

    def test_ineligible_reuse_never_imports_the_replaced_binary(self) -> None:
        installed: dict[str, object] = {
            "binary": "/installed/native.so",
            "sha256": "installed-transformed-bytes",
            "features": ["force-validate"],
        }
        events: list[str] = []

        def install(*_args: object, **_kwargs: object) -> int:
            events.append("install")
            return 0

        def capture(*_args: object, **_kwargs: object) -> dict[str, object]:
            events.append("capture")
            return installed

        with (
            patch.object(python_extension.subprocess, "call", side_effect=install),
            patch.object(python_extension, "capture", side_effect=capture),
        ):
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(events, ["install", "capture"])
            events.clear()
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(events, ["capture"], "eligible reuse still checks bytes")
            events.clear()
            (self.root / "crates/pse-py/src/lib.rs").write_text("changed Rust")
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(events, ["install", "capture"])
            events.clear()
            with patch.object(
                python_extension, "preparation_inputs", return_value=None
            ):
                self.assertEqual(
                    python_extension.extension_prepare(
                        "dev", "dev", self.environment
                    ).exit_code,
                    0,
                )
            self.assertEqual(events, ["install", "capture"])

    def test_owner_results_separate_install_reuse_and_check_without_publication(
        self,
    ) -> None:
        with patch.object(python_extension.subprocess, "call", return_value=0):
            prepared = python_extension.extension_prepare(
                "dev", "dev", self.environment
            )
            reused = python_extension.extension_prepare("dev", "dev", self.environment)
            self.assertEqual(prepared.status, "prepared")
            self.assertTrue(prepared.published)
            self.assertIsNotNone(prepared.artifact)
            self.assertEqual(reused.status, "reused")
            self.assertFalse(reused.published)
            self.assertEqual(prepared.artifact, reused.artifact)
            receipt = python_extension.stage_receipt("stubs", self.environment)
            self.assertFalse(receipt.exists())
            checked = python_extension.stubs_prepare(
                "dev", "dev", self.environment, check=True
            )
            self.assertEqual(checked.stage, "stubs-check")
            self.assertEqual(checked.status, "checked")
            self.assertFalse(checked.published)
            self.assertIsNone(checked.receipt)
            self.assertFalse(receipt.exists())
            published = python_extension.stubs_prepare("dev", "dev", self.environment)
            previous = receipt.read_bytes()
            self.assertTrue(published.published)
            python_extension.stubs_prepare("dev", "dev", self.environment, check=True)
            self.assertEqual(receipt.read_bytes(), previous)

    def test_failed_install_has_no_import_and_changed_inputs_cannot_publish(
        self,
    ) -> None:
        with (
            patch.object(python_extension.subprocess, "call", return_value=17),
            patch.object(python_extension, "capture") as capture,
        ):
            self.assertEqual(
                python_extension.extension_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                17,
            )
        capture.assert_not_called()
        self.assertFalse(
            python_extension.stage_receipt("extension", self.environment).exists()
        )

        def install(*_args: object, **_kwargs: object) -> int:
            (self.root / "crates/pse-py/src/lib.rs").write_text("changed during build")
            return 0

        with (
            patch.object(python_extension.subprocess, "call", side_effect=install),
            self.assertRaisesRegex(ValueError, "inputs changed during preparation"),
        ):
            python_extension.extension_prepare("dev", "dev", self.environment)
        self.assertFalse(
            python_extension.stage_receipt("extension", self.environment).exists()
        )

    def test_currency_change_diagnostics_only_render_bounded_safe_input_keys(
        self,
    ) -> None:
        private_value = "do-not-render-source-or-environment-value"
        previous: dict[str, object] = {
            "files": {"crates/a.rs": "old"},
            "configuration": {str(self.root / ".cargo/config.toml"): "old"},
            "environment": {"RUSTFLAGS": "old"},
        }
        current: dict[str, object] = {
            "files": {"crates/a.rs": private_value, "new-missing.rs": None},
            "configuration": {str(self.root / ".cargo/config.toml"): private_value},
            "environment": {
                "RUSTFLAGS": private_value,
                "NEW_BUILD_INPUT": private_value,
            },
        }
        result = python_extension.changed_inputs(previous, current)
        self.assertEqual(result.sections, ("configuration", "environment", "files"))
        self.assertEqual(result.source_keys, ("crates/a.rs", "new-missing.rs"))
        self.assertEqual(result.configuration_keys, (".cargo/config.toml",))
        self.assertEqual(result.environment_keys, ("NEW_BUILD_INPUT", "RUSTFLAGS"))
        self.assertTrue(result.complete)
        self.assertNotIn(private_value, result.model_dump_json())
        limited = python_extension.changed_inputs(
            {},
            {
                "environment": {
                    **{f"BUILD_{index}": private_value for index in range(40)},
                    "UNSAFE\nKEY": private_value,
                },
                "configuration": {"/outside/private-secret-path": private_value},
            },
        )
        self.assertEqual(len(limited.environment_keys), 32)
        self.assertFalse(limited.complete)
        self.assertNotIn(private_value, limited.model_dump_json())
        self.assertNotIn("private-secret-path", limited.model_dump_json())

    def test_actual_observer_keys_do_not_invalidate_build_but_unknown_selectors_do(
        self,
    ) -> None:
        observer_keys = {
            "PYTEST_CURRENT_TEST",
            "PYTEST_VERSION",
            "PYTEST_XDIST_WORKER",
            "PYTEST_XDIST_WORKER_COUNT",
            "PYTEST_XDIST_TESTRUNUID",
            "PSE_NATIVE_EXPECTED_BINARY",
            "PSE_NATIVE_EXPECTED_FEATURES",
            "PSE_NATIVE_EXPECTED_SHA256",
            "PSE_TEST_ENUMERATION",
        }
        first = python_extension.preparation_environment_identity(self.environment)
        self.assertEqual(
            first,
            python_extension.preparation_environment_identity(
                {**self.environment, **dict.fromkeys(observer_keys, "observer-only")}
            ),
        )
        for key in (
            "RUSTFLAGS",
            "PYO3_CONFIG_FILE",
            "PYTEST_UNKNOWN_BUILD_INPUT",
            "PSE_NATIVE_UNKNOWN_BUILD_INPUT",
            "CUSTOM_COMPILER_SELECTOR",
        ):
            with self.subTest(key=key):
                self.assertNotEqual(
                    first,
                    python_extension.preparation_environment_identity(
                        {**self.environment, key: "actual-selection"}
                    ),
                )

    def test_session_transport_changes_reuse_preparation_but_unknown_build_inputs_invalidate(
        self,
    ) -> None:
        first = {
            **self.environment,
            **dict.fromkeys(
                python_extension.PREPARATION_SESSION_ENVIRONMENT, "first-session"
            ),
        }
        next_call = {
            **self.environment,
            **dict.fromkeys(
                python_extension.PREPARATION_SESSION_ENVIRONMENT, "next-session"
            ),
        }
        self.assertNotEqual(
            native_tests.rust_environment_identity(first),
            native_tests.rust_environment_identity(next_call),
        )
        self.assertEqual(
            python_extension.preparation_environment_identity(first),
            python_extension.preparation_environment_identity(next_call),
        )
        with patch.object(
            python_extension.subprocess, "call", return_value=0
        ) as prepare:
            for environment in (first, next_call):
                self.assertEqual(
                    python_extension.extension_prepare(
                        "dev", "dev", environment
                    ).exit_code,
                    0,
                )
                self.assertEqual(
                    python_extension.stubs_prepare("dev", "dev", environment).exit_code,
                    0,
                )
            self.assertEqual(
                prepare.call_count,
                2,
                "one install and one stub generation cover both sessions",
            )
            for name in (
                "PYO3_CONFIG_FILE",
                "PYO3_ENVIRONMENT_SIGNATURE",
                "RUSTFLAGS",
                "CARGO_ENCODED_RUSTFLAGS",
                "PKG_CONFIG_PATH",
                "LD_LIBRARY_PATH",
                "CODEX_UNKNOWN_BUILD_INPUT",
                "DIRENV_UNKNOWN_BUILD_INPUT",
                "UNRECOGNIZED_BUILD_INPUT",
            ):
                with self.subTest(name=name):
                    changed = {**next_call, name: "changed-build-selection"}
                    self.assertNotEqual(
                        python_extension.preparation_environment_identity(next_call),
                        python_extension.preparation_environment_identity(changed),
                    )
            changed = {**next_call, "UNRECOGNIZED_BUILD_INPUT": "changed"}
            self.assertEqual(
                python_extension.extension_prepare("dev", "dev", changed).exit_code, 0
            )
            self.assertEqual(
                python_extension.stubs_prepare("dev", "dev", changed).exit_code, 0
            )
            self.assertEqual(prepare.call_count, 4)

    def test_configuration_tools_pyo3_and_installed_replacement_invalidate_currency(
        self,
    ) -> None:
        before = self.inputs()
        config = self.root / "pyo3.txt"
        config.write_text("abi3=true")
        self.environment["PYO3_CONFIG_FILE"] = str(config)
        configured = self.inputs()
        self.assertNotEqual(before, configured)
        config.write_text("abi3=false")
        self.assertNotEqual(configured, self.inputs())
        with patch.object(
            python_extension, "tool_identity", return_value={"tool": "replaced"}
        ):
            self.assertNotEqual(before, self.inputs())
        with patch.object(
            python_extension.subprocess, "call", return_value=0
        ) as installer:
            python_extension.extension_prepare("dev", "dev", self.environment)
            with patch.object(
                python_extension,
                "capture",
                return_value={
                    "binary": "/installed/native.so",
                    "sha256": "replacement",
                    "features": ["force-validate"],
                },
            ):
                python_extension.extension_prepare("dev", "dev", self.environment)
            self.assertEqual(installer.call_count, 2)
        self.environment["PYO3_CROSS_LIB_DIR"] = "/untracked/cross"
        self.assertIsNone(self.inputs())

    def test_native_currency_consumes_only_admitted_managed_generation_identities(
        self,
    ) -> None:
        environment = dict(self.environment)
        for name in (
            "IPOPT_DIR",
            "UNO_DIR",
            "PETSC_DIR",
            "PSE_ROOT_ISOLATION_DIR",
            "SUITESPARSE_LIBRARY_DIR",
        ):
            path = self.root / name
            path.mkdir()
            (path / ".complete.json").write_text('{"version":2}')
            environment[name] = str(
                path / "lib" if name == "SUITESPARSE_LIBRARY_DIR" else path
            )
        with patch.object(
            native_operation,
            "remembered",
            return_value={"managed": True, "identity": {"generation": "selected"}},
        ):
            first = python_extension.preparation_inputs(
                "python-extension", "native", "dev", environment, validated=True
            )
            self.assertIsNotNone(first)
            (self.root / "IPOPT_DIR/.complete.json").write_text(
                '{"version":2,"changed":true}'
            )
            self.assertNotEqual(
                first,
                python_extension.preparation_inputs(
                    "python-extension", "native", "dev", environment, validated=True
                ),
            )
        with patch.object(
            native_operation, "remembered", return_value={"managed": False}
        ):
            self.assertIsNone(
                python_extension.preparation_inputs(
                    "python-extension", "native", "dev", environment, validated=True
                )
            )

    def test_native_install_receipt_reuses_authenticated_unscoped_admission_across_operations(
        self,
    ) -> None:
        environment = dict(self.environment)
        prefixes = []
        for name in (
            "IPOPT_DIR",
            "UNO_DIR",
            "PETSC_DIR",
            "PSE_ROOT_ISOLATION_DIR",
            "SUITESPARSE_LIBRARY_DIR",
        ):
            prefix = self.root / name
            prefix.mkdir()
            (prefix / ".complete.json").write_text(
                '{"version":2,"identity":{"generation":"sealed"}}'
            )
            prefixes.append(prefix)
            environment[name] = str(
                prefix / "lib" if name == "SUITESPARSE_LIBRARY_DIR" else prefix
            )
        with (
            patch.object(native_operation, "scope_owner", return_value=None),
            patch.object(
                native_cache,
                "installation_files",
                side_effect=AssertionError("already admitted prefix rescanned"),
            ),
            patch.object(
                python_extension.subprocess, "call", return_value=0
            ) as installer,
        ):
            for _ in range(2):
                with native_operation.Operation(self.root / "native-cache"):
                    for prefix in prefixes:
                        native_operation.admit(
                            "prefix:" + str(prefix),
                            {"managed": True, "identity": {"generation": "sealed"}},
                        )
                    self.assertIsNone(native_operation.current())
                    self.assertEqual(
                        python_extension.extension_prepare(
                            "native", "dev", environment
                        ).exit_code,
                        0,
                    )
                    self.assertTrue(
                        python_extension.stage_receipt(
                            "extension", environment
                        ).is_file()
                    )
        self.assertEqual(installer.call_count, 1)

    def test_stub_currency_binds_generator_binary_and_output_membership(self) -> None:
        with patch.object(
            python_extension.subprocess, "call", return_value=0
        ) as generator:
            for _ in range(2):
                self.assertEqual(
                    python_extension.stubs_prepare(
                        "dev", "dev", self.environment
                    ).exit_code,
                    0,
                )
            self.assertEqual(generator.call_count, 1)
            (self.root / "xtask/src/codegen/python_stubs.rs").write_text(
                "changed generator"
            )
            self.assertEqual(
                python_extension.stubs_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(generator.call_count, 2)
            output = (
                self.root
                / "docs/generated/source-relationships/native-stubs/extra.json"
            )
            output.parent.mkdir(parents=True)
            output.write_text("extra")
            self.assertEqual(
                python_extension.stubs_prepare(
                    "dev", "dev", self.environment
                ).exit_code,
                0,
            )
            self.assertEqual(generator.call_count, 3)
            self.assertEqual(
                python_extension.stubs_prepare(
                    "dev", "dev", self.environment, check=True
                ).exit_code,
                0,
            )
            self.assertIn("--check", generator.call_args.args[0])


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

    def test_unknown_configuration_and_multiphase_selection_refuse_graph_assurance(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "pyproject.toml").write_text(
                '[tool.maturin]\nbindings="pyo3"\ntarget="unsupported-target"\n'
            )
            with self.assertRaisesRegex(
                arrow_validation.ValidationError, "unsupported Maturin configuration"
            ):
                python_extension.validated_maturin_command(
                    ["maturin", "develop"], {}, root=root
                )
            (root / "pyproject.toml").write_text(
                '[tool.maturin]\nbindings="pyo3"\ncompatibility="linux"\nskip-auditwheel=true\n[tool.maturin.sbom]\nrust=false\n'
            )
            command = python_extension.maturin_cargo_command(
                ["maturin", "develop"], root
            )
            self.assertIn("--lib", command)
        with self.assertRaisesRegex(arrow_validation.ValidationError, "multiphase"):
            python_extension.validated_maturin_command(
                ["maturin", "develop"], {"MATURIN_PGO": "1"}
            )

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
            patch.object(python_extension, "preparation_inputs", return_value=None),
            patch.object(python_extension, "capture", side_effect=ValueError("absent")),
            patch.object(
                python_extension,
                "capture_build_graph",
                side_effect=arrow_validation.ValidationError("uncovered"),
            ),
            self.assertRaisesRegex(arrow_validation.ValidationError, "uncovered"),
        ):
            python_extension.run_stages(
                "sync",
                "dev",
                "dev",
                {"VIRTUAL_ENV": str(python_extension.ROOT / ".venv")},
            )
        self.assertEqual(child.call_count, 1)
        self.assertEqual(child.call_args.args[0][:2], ["uv", "sync"])
        self.assertIn("--check", child.call_args.args[0])

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
