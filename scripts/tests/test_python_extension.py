# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual feature report and correctness build selection controls."""
# ruff: noqa: PT009, PT027 -- standard-library process boundary controls

from __future__ import annotations

import io
import json
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
    artifact_publication,
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

    def test_stage_composition_has_independent_operations_and_preserves_failures(
        self,
    ) -> None:
        stages = [
            "environment_ensure",
            "extension_prepare",
            "stubs_prepare",
            "requested_artifact",
        ]
        for operation, expected in (
            ("environment", ["environment_ensure"]),
            ("prepare", ["extension_prepare"]),
            ("stubs", ["stubs_prepare"]),
            ("stubs-check", ["stubs_prepare"]),
            ("admit", ["requested_artifact"]),
            ("sync", stages),
        ):
            events = []
            with self.subTest(operation=operation):
                self.enterContext(
                    patch.object(
                        python_extension.subprocess, "check_call", return_value=0
                    )
                )
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
                                    if _name == "requested_artifact"
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


class PublicationPreparationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.enterContext(patch.object(python_extension, "ROOT", self.root))
        self.alias = self.root / "python/pse/_native.abi3.so"
        self.alias.parent.mkdir(parents=True)
        self.alias.write_bytes(b"completed installed image")
        (self.root / "pyproject.toml").write_text(
            '[tool.maturin]\nbindings="pyo3"\nmanifest-path="crates/pse-py/Cargo.toml"\n'
        )
        self.environment = {
            "UV_PROJECT_ENVIRONMENT": str(self.root / ".venv"),
            "VIRTUAL_ENV": str(self.root / ".venv"),
        }
        self.enterContext(
            patch.object(
                python_extension,
                "validated_maturin_command",
                side_effect=lambda command, *_args: list(command),
            )
        )
        self.enterContext(
            patch.object(
                arrow_validation,
                "resolve",
                side_effect=lambda command, **_kwargs: SimpleNamespace(
                    command=list(command)
                ),
            )
        )

    def prepare(self, *, profile: str = "release") -> python_extension.StageResult:
        with (
            patch.object(python_extension.subprocess, "call", return_value=0),
            patch.object(
                python_extension,
                "_capture_candidate_features",
                return_value=["force-validate"],
            ),
        ):
            return python_extension.extension_prepare("dev", profile, self.environment)

    def test_explicit_preparation_publishes_candidate_and_cargo_owns_rebuild_choice(
        self,
    ) -> None:
        first = self.prepare()
        second = self.prepare()
        self.assertEqual(first.status, "prepared")
        self.assertEqual(second.status, "prepared")
        self.assertNotEqual(first.artifact, second.artifact)
        self.assertIsNotNone(first.artifact)
        assert first.artifact is not None
        self.assertEqual(
            Path(first.artifact.binary).read_bytes(), b"completed installed image"
        )
        publication = python_extension.read_publication("extension", self.environment)
        assert publication is not None
        self.assertEqual(publication.artifact, second.artifact)

    def test_publication_selection_ignores_source_alias_and_compiler_environment_changes(
        self,
    ) -> None:
        prepared = self.prepare()
        assert prepared.artifact is not None
        self.alias.unlink()
        (self.root / "changed.rs").write_text("new source for a future explicit build")
        with patch.object(
            python_extension.subprocess,
            "check_output",
            side_effect=AssertionError("consumer import"),
        ):
            actual = python_extension.currency_check_installed(
                {**self.environment, "RUSTFLAGS": "later build flags"}, mode="dev"
            )
            self.assertEqual(actual.state, "current")
            self.assertEqual(actual.artifact, prepared.artifact)
            self.assertEqual(actual.profile, "release")
            self.assertEqual(
                python_extension.capture(self.environment),
                prepared.artifact.model_dump(),
            )
        self.assertEqual(
            python_extension.currency_check(
                self.environment, mode="native", profile="release"
            ).state,
            "unsupported",
        )
        self.assertEqual(
            python_extension.currency_check(
                self.environment, mode="dev", profile="other"
            ).state,
            "unsupported",
        )

    def test_failed_install_does_not_touch_previous_publication(self) -> None:
        self.prepare()
        path = python_extension.stage_receipt("extension", self.environment)
        previous = path.read_bytes()
        with (
            patch.object(python_extension.subprocess, "call", return_value=17),
            patch.object(artifact_publication, "publish") as publish,
        ):
            failed = python_extension.extension_prepare(
                "dev", "release", self.environment
            )
        self.assertEqual(failed.exit_code, 17)
        publish.assert_not_called()
        self.assertEqual(path.read_bytes(), previous)

    def test_candidate_probe_loads_exact_image_once_and_checks_requested_contract(
        self,
    ) -> None:
        candidate = self.root / "private/_native.abi3.so"
        report: dict[str, object] = {
            "prefix": str(self.root / ".venv"),
            "package": str(self.root / "python/pse/__init__.py"),
            "native": str(candidate),
            "features": ["force-validate"],
        }
        with patch.object(
            python_extension.subprocess, "check_output", return_value=json.dumps(report)
        ) as probe:
            self.assertEqual(
                python_extension._capture_candidate_features(  # noqa: SLF001 -- exercise the constructor probe
                    self.environment, candidate, validated=True, native=False
                ),
                ["force-validate"],
            )
        self.assertEqual(probe.call_count, 1)
        self.assertEqual(
            probe.call_args.kwargs["env"]["_PSE_NATIVE_PUBLISHED_IMAGE"], str(candidate)
        )
        for features in (
            [],
            ["force-validate", "force-validate"],
            ["force-validate", {}],
            ["force-validate", "native-solvers"],
        ):
            report["features"] = features
            with (
                self.subTest(features=features),
                patch.object(
                    python_extension.subprocess,
                    "check_output",
                    return_value=json.dumps(report),
                ),
                self.assertRaises(ValueError),
            ):
                python_extension._capture_candidate_features(  # noqa: SLF001 -- exercise the constructor probe
                    self.environment, candidate, validated=True, native=False
                )

    def test_explicit_import_observation_uses_issued_image_without_rehashing(
        self,
    ) -> None:
        prepared = self.prepare()
        assert prepared.artifact is not None
        with patch.object(
            python_extension.subprocess, "check_call", return_value=0
        ) as imported:
            result = python_extension.run_stages(
                "admit", "dev", "release", self.environment
            )
        self.assertEqual(result.stages[0].artifact, prepared.artifact)
        self.assertEqual(imported.call_count, 1)
        self.assertEqual(
            imported.call_args.kwargs["env"]["_PSE_NATIVE_PUBLISHED_IMAGE"],
            prepared.artifact.binary,
        )

    def test_explicit_stages_refuse_incompatible_contract_before_import_or_generator(
        self,
    ) -> None:
        path = python_extension.stage_receipt("extension", self.environment)
        published = artifact_publication.publish(
            path,
            kind="extension",
            source=self.alias,
            mode="native",
            profile="producer",
            validated=True,
            interpreter=str(python_extension.interpreter(self.environment)),
            features=lambda _candidate: ["force-validate", "native-solvers"],
        )
        original = path.read_bytes()
        for stage in ("admit", "stubs", "stubs-check"):
            for mode, profile in (
                ("dev", "producer"),
                ("native", "dev"),
                ("dev", "dev"),
            ):
                with (
                    self.subTest(stage=stage, mode=mode, profile=profile),
                    patch.object(python_extension.subprocess, "check_call") as imported,
                    patch.object(python_extension.subprocess, "call") as generated,
                    patch.object(python_extension, "preparation_inputs") as inputs,
                    self.assertRaisesRegex(
                        ValueError, "requested .*incompatible-publication"
                    ),
                ):
                    python_extension.run_stages(stage, mode, profile, self.environment)
                imported.assert_not_called()
                generated.assert_not_called()
                inputs.assert_not_called()
                self.assertEqual(path.read_bytes(), original)
        self.assertEqual(
            python_extension.capture(self.environment), published.artifact.model_dump()
        )
        interactive = artifact_publication.publish(
            path,
            kind="extension",
            source=self.alias,
            mode="dev",
            profile="dev",
            validated=False,
            interpreter=str(python_extension.interpreter(self.environment)),
            features=lambda _candidate: [],
        )
        for stage in ("admit", "stubs", "stubs-check"):
            with (
                self.subTest(stage=stage),
                patch.object(python_extension.subprocess, "check_call") as imported,
                patch.object(python_extension.subprocess, "call") as generated,
                self.assertRaisesRegex(ValueError, "validated=True"),
            ):
                python_extension.run_stages(stage, "dev", "dev", self.environment)
            imported.assert_not_called()
            generated.assert_not_called()
        self.assertEqual(
            python_extension.read_publication("extension", self.environment),
            interactive,
        )

    def test_explicit_compatible_stage_contract_matches_result_and_child_selection(
        self,
    ) -> None:
        for mode, profile, validated in (
            ("dev", "dev", False),
            ("native", "producer", True),
        ):
            published = artifact_publication.publish(
                python_extension.stage_receipt("extension", self.environment),
                kind="extension",
                source=self.alias,
                mode=mode,
                profile=profile,
                validated=validated,
                interpreter=str(python_extension.interpreter(self.environment)),
                features=lambda _candidate, validated=validated, mode=mode: (
                    (["force-validate"] if validated else [])
                    + (["native-solvers"] if mode == "native" else [])
                ),
            )
            for stage in ("admit", "stubs", "stubs-check"):
                with (
                    self.subTest(stage=stage, mode=mode),
                    patch.object(
                        python_extension.subprocess, "check_call", return_value=0
                    ) as imported,
                    patch.object(
                        python_extension.subprocess, "call", return_value=0
                    ) as generated,
                    patch.object(
                        python_extension, "preparation_inputs", return_value=None
                    ),
                ):
                    result = python_extension.run_stages(
                        stage, mode, profile, self.environment, validated=validated
                    )
                self.assertEqual(
                    (result.mode, result.profile, result.validated),
                    (mode, profile, validated),
                )
                self.assertEqual(result.stages[-1].artifact, published.artifact)
                child = imported if stage == "admit" else generated
                self.assertEqual(
                    child.call_args.kwargs["env"]["_PSE_NATIVE_PUBLISHED_IMAGE"],
                    published.artifact.binary,
                )

    def test_ordinary_sync_uses_new_publication_instead_of_inherited_strict_selection(
        self,
    ) -> None:
        strict_path = self.root / "strict/python-publication.json"
        strict = artifact_publication.publish(
            strict_path,
            kind="extension",
            source=self.alias,
            mode="native",
            profile="producer",
            validated=True,
            interpreter=str(python_extension.interpreter(self.environment)),
            features=lambda _candidate: ["force-validate", "native-solvers"],
        )
        environment = {**self.environment, "_PSE_NATIVE_PUBLICATION": str(strict_path)}
        with (
            patch.object(
                python_extension,
                "environment_ensure",
                return_value=python_extension.StageResult(
                    stage="environment", status="reused"
                ),
            ),
            patch.object(python_extension.subprocess, "call", return_value=0),
            patch.object(
                python_extension,
                "_capture_candidate_features",
                return_value=["force-validate"],
            ),
            patch.object(
                python_extension,
                "stubs_prepare",
                return_value=python_extension.StageResult(
                    stage="stubs", status="prepared"
                ),
            ) as stubs,
        ):
            result = python_extension.run_stages("sync", "dev", "release", environment)
        ordinary = python_extension.read_publication("extension", self.environment)
        assert ordinary is not None
        self.assertEqual(result.stages[-1].artifact, ordinary.artifact)
        self.assertNotEqual(ordinary.generation, strict.generation)
        self.assertNotIn("_PSE_NATIVE_PUBLICATION", stubs.call_args.args[2])
        self.assertEqual(
            python_extension.read_publication("extension", environment), strict
        )
        self.assertEqual(environment["_PSE_NATIVE_PUBLICATION"], str(strict_path))

    def test_child_selection_carries_only_issued_image_and_stub_scope_stays_independent(
        self,
    ) -> None:
        prepared = self.prepare()
        assert prepared.artifact is not None
        selected = python_extension.execution_environment(
            self.environment, prepared.artifact.model_dump()
        )
        self.assertEqual(
            selected["_PSE_NATIVE_PUBLISHED_IMAGE"], prepared.artifact.binary
        )
        self.assertNotIn("PSE_NATIVE_EXPECTED_SHA256", selected)
        with (
            patch.object(
                python_extension,
                "preparation_inputs",
                return_value={"stub_inputs": "independent"},
            ),
            patch.object(
                python_extension.subprocess, "call", return_value=0
            ) as generator,
        ):
            self.assertEqual(
                python_extension.stubs_prepare(
                    "dev", "release", self.environment
                ).status,
                "prepared",
            )
            self.assertEqual(
                python_extension.stubs_prepare(
                    "dev", "release", self.environment
                ).status,
                "reused",
            )
        self.assertEqual(generator.call_count, 1)
        self.assertEqual(
            generator.call_args.kwargs["env"]["_PSE_NATIVE_PUBLISHED_IMAGE"],
            prepared.artifact.binary,
        )


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
