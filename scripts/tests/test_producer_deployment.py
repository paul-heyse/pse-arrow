# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Mocked controls for source-reviewed actual deployment capture orchestration."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import producer_deployment as deployment

if TYPE_CHECKING:
    from collections.abc import Callable
    from typing import BinaryIO


class ProducerDeploymentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.output = self.root / "evidence"
        self.environment = {}
        cargo = patch.object(
            deployment.producer_cargo,
            "cargo_executable",
            return_value="/selected/cargo",
        )
        cargo.start()
        self.addCleanup(cargo.stop)
        self.native = self.root / "native-library"
        self.native.write_bytes(b"actual native input fixture")
        self.python_native = self.root / "python-native-library"
        self.python_native.write_bytes(b"actual Python native input fixture")
        common = self.root / "native-inputs.json"
        common.write_text(json.dumps([str(self.native)]))
        python = self.root / "python-native-inputs.json"
        python.write_text(json.dumps([str(self.native), str(self.python_native)]))
        self.environment["PSE_NATIVE_PRODUCER_INPUTS"] = str(common)
        self.environment["PSE_PYTHON_PRODUCER_INPUTS"] = str(python)
        self.declarations = {}
        for name, *_ in deployment.ROLES:
            path = self.root / f"{name}-context.json"
            path.write_text(
                json.dumps(
                    {
                        "capture_actual_build": True,
                        "native_inputs_complete": True,
                        "native_reviewed_source": "c" * 64,
                    }
                )
            )
            self.declarations[name] = path
            self.environment[f"PSE_{name.upper()}_PRODUCER_DECLARATIONS"] = str(path)
        worker = self.root / "target/producer/pse-worker"
        worker.parent.mkdir(parents=True)
        worker.write_bytes(b"actual captured worker fixture")
        self.artifacts = {"worker": worker}
        for role in ("runtime", "python"):
            artifact = self.root / f"{role}-artifact"
            artifact.write_bytes(role.encode())
            self.artifacts[role] = artifact
        self.actual_bind = deployment.bind_python_artifact
        binding = patch.object(deployment, "bind_python_artifact")
        binding.start()
        self.addCleanup(binding.stop)

    def actual_capture(
        self,
        change: Callable[[str, dict[str, object], dict[str, object]], None]
        | None = None,
    ) -> Callable[..., subprocess.CompletedProcess[bytes | str]]:
        def run(
            command: list[str],
            *,
            cwd: Path,
            env: dict[str, str],
            stdout: BinaryIO | int | None = None,
            stderr: int | None = None,
            check: bool,
            text: bool = False,
            capture_output: bool = False,
            pass_fds: tuple[int, ...] = (),
        ) -> subprocess.CompletedProcess[bytes | str]:
            self.assertEqual(cwd, self.root)
            self.assertIsInstance(text, bool)
            if "producer-history-verify" in command:
                self.assertTrue(text)
                self.assertTrue(capture_output)
                name = Path(command[-1]).name.removesuffix(
                    "-build-evidence.json.closure"
                )
                return subprocess.CompletedProcess(
                    command,
                    0,
                    stdout=(
                        Path(command[-1]).parent / f"{name}-fixture-history.json"
                    ).read_text(),
                )
            if command[:2] == ["uv", "sync"]:
                return subprocess.CompletedProcess(command, 0)
            if command[:2] == ["cargo", "build"]:
                self.assertEqual(
                    command[2:],
                    [
                        "-p",
                        "xtask",
                        "--bin",
                        "xtask",
                        "--no-default-features",
                        "--locked",
                    ],
                )
                return subprocess.CompletedProcess(command, 0)
            if "producer-preparation-begin" in command:
                selection = json.loads(
                    Path(command[command.index("--selection") + 1]).read_text()
                )
                self.assertEqual(
                    [item["package"] for item in selection],
                    [role[1] for role in deployment.ROLES],
                )
                self.assertEqual(
                    Path(command[command.index("--directory") + 1]).name, "preparation"
                )
                return subprocess.CompletedProcess(command, 0)
            if Path(command[0]).name == "maturin":
                selected = deployment.read_object(
                    Path(env[deployment.producer_cargo.CAPTURE])
                )
                options = selected["capture_options"]
                if not isinstance(options, list):
                    raise TypeError("installer must select its actual capture options")
                command = ["just", "producer-identity", *options]
                env = {
                    **self.environment,
                    "PYO3_BUILD_EXTENSION_MODULE": "1",
                    deployment.producer_cargo.PREPARATION_GUARD_FD: env[
                        deployment.producer_cargo.PREPARATION_GUARD_FD
                    ],
                }
                for name, value in deployment.producer_cargo.effective_environment(
                    env
                ).items():
                    command.extend(("--environment", f"{name}={value}"))
                (self.output / "python-installer-cargo.status.json").write_text(
                    json.dumps({"returncode": 0, "sealed": True})
                )
            if "verify-deployment-artifact" in command:
                self.assertTrue(text)
                self.assertTrue(capture_output)
                observation = deployment.file_observation(
                    Path(command[command.index("--artifact") + 1])
                )
                return subprocess.CompletedProcess(
                    command, 0, stdout=json.dumps(observation)
                )
            if "observe-deployment" in command:
                Path(command[-1]).write_text(
                    json.dumps({"source": "fresh outer", "build": "fresh context"})
                )
                return subprocess.CompletedProcess(command, 0)
            self.assertEqual(stderr, subprocess.STDOUT)
            self.assertFalse(check)
            output = Path(command[command.index("--output") + 1])
            name = output.stem
            expected_env = dict(self.environment)
            expected_env[deployment.producer_cargo.PREPARATION_GUARD_FD] = env[
                deployment.producer_cargo.PREPARATION_GUARD_FD
            ]
            self.assertEqual(
                pass_fds, (int(env[deployment.producer_cargo.PREPARATION_GUARD_FD]),)
            )
            if name == "python":
                expected_env["PYO3_BUILD_EXTENSION_MODULE"] = "1"
            else:
                expected_env.pop("PYO3_BUILD_EXTENSION_MODULE", None)
                expected_env.pop("PYO3_CONFIG_FILE", None)
            self.assertEqual(env, expected_env)
            role = next(role for role in deployment.ROLES if role[0] == name)
            _, package, target, kind, _ = role
            unit = {
                "key": name + "-selected-unit-key",
                "package_id": "path+workspace://fixture#0.0.1",
                "target_name": target,
                "target_kind": [kind],
                "mode": "build",
                "profile": {"name": "producer"},
                "features": ["native-solvers"],
            }
            receipt: dict[str, object] = {
                "frame": "pse.producer.v1",
                "package": package,
                "identity": str(deployment.ROLES.index(role) + 1) * 64,
                "selected_root": unit["key"],
                "units": [unit],
                "persistent_reuse_eligible": True,
                "reasons": [],
                "receipt_version": 2,
                "native_abi": "reviewed-fixture-abi",
                "deployment": {
                    "artifact": deployment.file_observation(self.artifacts[name])
                },
            }
            evidence: dict[str, object] = {
                "success": True,
                "unit_graph": {
                    "roots": [0],
                    "units": [
                        {
                            "pkg_id": "path+file://fixture#0.0.1",
                            "target": {"name": target, "kind": [kind]},
                            "mode": "build",
                            "profile": unit["profile"],
                            "features": unit["features"],
                        }
                    ],
                },
            }
            if change:
                change(name, receipt, evidence)
            output.write_text(json.dumps(receipt))
            Path(command[command.index("--build-evidence") + 1]).write_text(
                json.dumps(evidence)
            )
            (output.parent / f"{name}-fixture-history.json").write_text(
                json.dumps(
                    {
                        "version": 1,
                        "historical_only": True,
                        "producer_identity": receipt["identity"],
                        "selected_root": receipt["selected_root"],
                        "actual_build_evidence": evidence,
                    }
                )
            )
            if stdout is None or isinstance(stdout, int):
                raise AssertionError("Capture diagnostics require an opened binary log")
            stdout.write(b"actual capture diagnostics\n")
            return subprocess.CompletedProcess(command, 0)

        return run

    def verified_receipts(self) -> dict[str, dict[str, object]]:
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ):
            return deployment.validate_receipts(
                self.root, self.output, self.environment
            )

    def test_capture_orders_actual_targets_and_preserves_safe_declared_environment(
        self,
    ) -> None:
        self.environment.update(
            TZ="UTC",
            SOURCE_DATE_EPOCH="0",
            LIBCLANG_PATH="selected-libclang",
            CFLAGS_x86_64_unknown_linux_gnu="-O2",
            LLVM_CONFIG_PATH="selected-llvm",
            PSE_SCCACHE_BINARY="selected-cache-binary",
            PYO3_BUILD_EXTENSION_MODULE="",
            PYO3_CONFIG_FILE=str(self.python_native),
            SYMBOLICA_LICENSE="private-secret",
            SURREAL_PASS=str(self.root / "unused-surreal-credential"),
        )
        before = {name: path.read_bytes() for name, path in self.declarations.items()}
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ) as run:
            environment = deployment.capture(self.root, self.output, self.environment)
        commands = [
            call.args[0] for call in run.call_args_list if call.args[0][0] == "just"
        ]
        all_commands = [call.args[0] for call in run.call_args_list]
        self.assertEqual(
            all_commands[0], ["uv", "sync", "--locked", "--no-install-project"]
        )
        self.assertEqual(all_commands[1][:2], ["cargo", "build"])
        self.assertIn("producer-preparation-begin", all_commands[2])
        self.assertTrue((self.output / "preparation/COMPLETE").is_file())
        self.assertEqual(len(commands), 2)  # Python is selected by Maturin's child.
        options = deployment.read_object(self.output / "python-installer-cargo.json")[
            "capture_options"
        ]
        self.assertIsInstance(options, list)
        commands.append(["just", "producer-identity", *options])
        self.assertEqual(
            [command[command.index("--package") + 1] for command in commands],
            ["pse-runtime", "xtask", "pse-py"],
        )
        for command, role in zip(commands, deployment.ROLES, strict=True):
            name, _, _, _, target_args = role
            self.assertEqual(command[:2], ["just", "producer-identity"])
            self.assertEqual(command[command.index("--profile") + 1], "producer")
            self.assertIn(target_args[0], command)
            if len(target_args) > 1:
                self.assertEqual(
                    command[command.index(target_args[0]) + 1], target_args[1]
                )
            self.assertEqual(
                command[command.index("--features") + 1],
                "native-solvers,force-validate"
                if name == "python"
                else "native-solvers,pse-relations/force-validate",
            )
            self.assertNotIn("--refresh-executors", command)
            self.assertEqual(
                command[command.index("--preparation") + 1],
                str(self.output / "preparation"),
            )
            self.assertNotIn("--outer-source", command)
            self.assertNotIn("--outer-build", command)
            if name == "python":
                self.assertNotIn("--environment", command)
                installer = next(
                    call
                    for call in run.call_args_list
                    if Path(call.args[0][0]).name == "maturin"
                )
                self.assertEqual(
                    installer.kwargs["env"]["PYO3_CONFIG_FILE"], str(self.python_native)
                )
                self.assertEqual(
                    installer.kwargs["env"]["CARGO"],
                    str(self.root / "scripts/producer-cargo"),
                )
            else:
                self.assertIn("TZ=UTC", command)
                self.assertIn("SOURCE_DATE_EPOCH=0", command)
                self.assertIn("CFLAGS_x86_64_unknown_linux_gnu=-O2", command)
                self.assertIn("LLVM_CONFIG_PATH=selected-llvm", command)
                self.assertIn("PSE_SCCACHE_BINARY=selected-cache-binary", command)
                self.assertFalse(
                    any(
                        item.startswith("PYO3_BUILD_EXTENSION_MODULE=")
                        for item in command
                    )
                )
                self.assertFalse(
                    any(item.startswith("PYO3_CONFIG_FILE=") for item in command)
                )
            self.assertNotIn("private-secret", " ".join(command))
            self.assertNotIn("private-password", " ".join(command))
            self.assertEqual(self.declarations[name].read_bytes(), before[name])
        self.assertNotIn(str(self.python_native), commands[0])
        self.assertNotIn(str(self.python_native), commands[1])
        self.assertIn(str(self.python_native), commands[2])
        self.assertEqual(
            environment, deployment.deployment_environment(self.root, self.output)
        )
        self.assertEqual(
            environment["PSE_PRODUCER_RECEIPT"], str(self.output / "worker.json")
        )
        self.assertFalse((self.output / "python-attestation.json").exists())
        self.assertEqual(
            set(self.verified_receipts()),
            {"runtime", "worker", "python"},
        )

    def test_python_native_inputs_can_use_common_reviewed_list(self) -> None:
        self.environment.pop("PSE_PYTHON_PRODUCER_INPUTS")
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ) as run:
            deployment.capture(self.root, self.output, self.environment)
        for call in run.call_args_list:
            if call.args[0][0] != "just":
                continue
            self.assertIn(str(self.native), call.args[0])
            self.assertNotIn(str(self.python_native), call.args[0])

    def test_missing_role_context_refuses_before_any_capture(self) -> None:
        self.environment.pop("PSE_WORKER_PRODUCER_DECLARATIONS")
        with (
            patch.object(deployment.subprocess, "run") as run,
            self.assertRaisesRegex(ValueError, "PSE_WORKER_PRODUCER_DECLARATIONS"),
        ):
            deployment.capture(self.root, self.output, self.environment)
        run.assert_not_called()
        self.assertTrue((self.output / "deployment-error.log").is_file())

    def test_unsupported_cargo_selection_refuses_before_any_role_build_or_install(
        self,
    ) -> None:
        with (
            patch.object(
                deployment.producer_cargo,
                "cargo_executable",
                side_effect=ValueError("unsupported explicit Cargo invocation"),
            ),
            patch.object(deployment.subprocess, "run") as run,
            self.assertRaisesRegex(ValueError, "unsupported explicit Cargo"),
        ):
            deployment.capture(self.root, self.output, self.environment)
        run.assert_not_called()

    def test_unreviewed_or_graph_only_context_is_not_refreshed(self) -> None:
        path = self.declarations["runtime"]
        path.write_text(
            json.dumps({"capture_actual_build": False, "native_inputs_complete": True})
        )
        before = path.read_bytes()
        with (
            patch.object(deployment.subprocess, "run") as run,
            self.assertRaisesRegex(ValueError, "source-bound"),
        ):
            deployment.capture(self.root, self.output, self.environment)
        run.assert_not_called()
        self.assertEqual(path.read_bytes(), before)

    def test_missing_native_input_refuses_before_capture(self) -> None:
        self.native.unlink()
        with (
            patch.object(deployment.subprocess, "run") as run,
            self.assertRaisesRegex(ValueError, "native input file is missing"),
        ):
            deployment.capture(self.root, self.output, self.environment)
        run.assert_not_called()

    def test_stale_output_is_refused_without_overwriting(self) -> None:
        self.output.mkdir()
        sentinel = self.output / "runtime.json"
        sentinel.write_bytes(b"prior capture")
        with (
            patch.object(deployment.subprocess, "run") as run,
            self.assertRaises(FileExistsError),
        ):
            deployment.capture(self.root, self.output, self.environment)
        run.assert_not_called()
        self.assertEqual(sentinel.read_bytes(), b"prior capture")
        self.assertEqual(list(self.output.iterdir()), [sentinel])

    def test_ineligible_receipt_stops_order_and_retains_diagnostics(self) -> None:
        def change(
            name: str, receipt: dict[str, object], _evidence: dict[str, object]
        ) -> None:
            if name == "worker":
                receipt["persistent_reuse_eligible"] = False
                receipt["reasons"] = ["native closure changed"]

        with (
            patch.object(
                deployment.subprocess, "run", side_effect=self.actual_capture(change)
            ) as run,
            self.assertRaisesRegex(ValueError, "ineligible"),
        ):
            deployment.capture(self.root, self.output, self.environment)
        self.assertEqual(
            len([call for call in run.call_args_list if call.args[0][0] == "just"]), 2
        )
        self.assertFalse((self.output / "python.json").exists())
        self.assertEqual(
            (self.output / "worker.log").read_bytes(), b"actual capture diagnostics\n"
        )
        self.assertTrue((self.output / "worker-build-evidence.json").is_file())

    def test_changed_selected_role_and_missing_root_are_refused(self) -> None:
        for field, value in (
            ("package", "pse-py"),
            ("frame", "wrong"),
            ("selected_root", "missing"),
        ):
            with self.subTest(field=field):
                output = self.root / field

                def change(
                    _name: str,
                    receipt: dict[str, object],
                    _evidence: dict[str, object],
                    *,
                    field: str = field,
                    value: str = value,
                ) -> None:
                    receipt[field] = value

                with (
                    patch.object(
                        deployment.subprocess,
                        "run",
                        side_effect=self.actual_capture(change),
                    ),
                    self.assertRaises(ValueError),
                ):
                    deployment.capture(self.root, output, self.environment)

    def test_actual_graph_root_must_match_receipt_role(self) -> None:
        def change(
            _name: str, _receipt: dict[str, object], evidence: dict[str, object]
        ) -> None:
            evidence["unit_graph"] = {
                "roots": [0],
                "units": [
                    {
                        "target": {"name": "pse-worker", "kind": ["bin"]},
                        "mode": "build",
                        "profile": {"name": "producer"},
                        "features": ["native-solvers"],
                    }
                ],
            }

        with (
            patch.object(
                deployment.subprocess, "run", side_effect=self.actual_capture(change)
            ),
            self.assertRaisesRegex(ValueError, "actual selected build root"),
        ):
            deployment.capture(self.root, self.output, self.environment)

    def test_final_validation_rereads_earlier_role_receipts(self) -> None:
        def change(
            name: str, _receipt: dict[str, object], _evidence: dict[str, object]
        ) -> None:
            if name == "python":
                path = self.output / "worker.json"
                worker = json.loads(path.read_text())
                worker["package"] = "pse-runtime"
                path.write_text(json.dumps(worker))

        with (
            patch.object(
                deployment.subprocess, "run", side_effect=self.actual_capture(change)
            ) as run,
            self.assertRaisesRegex(ValueError, "worker receipt has the wrong"),
        ):
            deployment.capture(self.root, self.output, self.environment)
        self.assertEqual(
            [
                Path(call.args[0][0]).name
                for call in run.call_args_list
                if "producer-history-verify" not in call.args[0]
            ],
            ["uv", "cargo", "xtask", "just", "just", "maturin"],
        )
        self.assertTrue((self.output / "deployment-error.log").is_file())

    def test_artifact_replacement_and_historical_shape_refuse_before_role_decoding(
        self,
    ) -> None:
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ):
            deployment.capture(self.root, self.output, self.environment)
        receipt = deployment.read_object(self.output / "worker.json")
        self.artifacts["worker"].write_bytes(b"replacement at same path")
        with self.assertRaisesRegex(ValueError, "artifact changed"):
            self.verified_receipts()
        receipt["deployment"] = "historical shape cannot be current-decoded"
        for version in (1, 2.0, True, None):
            receipt["receipt_version"] = version
            with self.assertRaisesRegex(ValueError, "interpretation"):
                deployment.receipt_artifact(receipt, "worker")

    def test_role_artifacts_are_independent_and_outer_is_fresh_context(self) -> None:
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ):
            deployment.capture(self.root, self.output, self.environment)
        associations = self.verified_receipts()
        self.assertEqual(len({record["sha256"] for record in associations.values()}), 3)
        self.assertTrue((self.output / "outer-observation.json").is_file())
        self.assertEqual(
            json.loads((self.output / "observed-artifacts.json").read_text()),
            associations,
        )

    def test_role_validation_uses_protected_context_after_legacy_evidence_disappears(
        self,
    ) -> None:
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ):
            deployment.capture(self.root, self.output, self.environment)
        for role, *_ in deployment.ROLES:
            (self.output / f"{role}-build-evidence.json").unlink()
        self.assertEqual(set(self.verified_receipts()), {"runtime", "worker", "python"})
        receipt = deployment.read_object(self.output / "worker.json")
        receipt["identity"] = "f" * 64
        (self.output / "worker.json").write_text(json.dumps(receipt))
        with self.assertRaisesRegex(
            ValueError, "protected historical evidence differs"
        ):
            self.verified_receipts()

    def test_protected_reader_identity_and_completion_precede_current_artifact_checks(
        self,
    ) -> None:
        receipt = {
            "receipt_version": 2,
            "identity": "a" * 64,
            "selected_root": "selected",
        }
        history = {
            "version": 1,
            "historical_only": True,
            "producer_identity": receipt["identity"],
            "selected_root": "selected",
            "actual_build_evidence": {"success": True},
        }
        for changed in (
            {"version": True},
            {"version": 2},
            {"historical_only": False},
            {"producer_identity": "b" * 64},
            {"selected_root": "other"},
            {"actual_build_evidence": None},
            {"actual_build_evidence": {"success": False}},
        ):
            with (
                self.subTest(changed=changed),
                patch.object(
                    deployment.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(
                        [], 0, stdout=json.dumps({**history, **changed})
                    ),
                ),
                patch.object(
                    deployment,
                    "receipt_artifact",
                    side_effect=AssertionError(
                        "current artifact checked before protected context"
                    ),
                ),
                self.assertRaises(ValueError),
            ):
                deployment.load_history(
                    self.root, self.output, "worker", self.environment, receipt
                )
        with (
            patch.object(
                deployment.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 1, stdout="", stderr="historical seal differs"
                ),
            ),
            self.assertRaisesRegex(ValueError, "historical seal differs"),
        ):
            deployment.load_history(
                self.root, self.output, "worker", self.environment, receipt
            )

    def capture_shared_contracts(
        self, contracts: dict[str, list[dict[str, object]]]
    ) -> None:
        def change(
            name: str,
            receipt: dict[str, object],
            _evidence: dict[str, object],
        ) -> None:
            units = receipt["units"]
            if not isinstance(units, list):
                raise TypeError("Expected actual unit fixture list")
            units.extend(
                {
                    "key": f"{name}-bitflags-{index}",
                    "package_id": "registry+https://github.com/rust-lang/crates.io-index#bitflags@2.9.4",
                    "target_name": "bitflags",
                    "target_kind": ["lib"],
                    "mode": "build",
                    "platform": None,
                    **contract,
                }
                for index, contract in enumerate(contracts[name])
            )

        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture(change)
        ):
            deployment.capture(self.root, self.output, self.environment)

    def shared_contract(self) -> dict[str, object]:
        return {
            "profile": {
                "name": "producer",
                "opt_level": "3",
                "debug_assertions": False,
            },
            "features": ["std"],
            "dependencies": {},
        }

    def test_same_role_variants_are_retained_as_complete_contract_sets(self) -> None:
        standard = self.shared_contract()
        serde = {
            **standard,
            "features": ["serde", "std"],
            "dependencies": {"serde_core:serde-unit": "serde-unit"},
        }
        self.capture_shared_contracts(
            {
                "runtime": [standard, serde, standard],
                "worker": [serde, standard],
                "python": [standard, serde],
            }
        )
        self.assertEqual(
            set(self.verified_receipts()),
            {"runtime", "worker", "python"},
        )

    def test_extra_root_helper_context_is_a_compatible_superset(self) -> None:
        standard = self.shared_contract()
        helper = {**standard, "profile": {"name": "producer", "opt_level": "0"}}
        self.capture_shared_contracts(
            {"runtime": [standard], "worker": [standard, helper], "python": [standard]}
        )
        self.assertEqual(len(self.verified_receipts()), 3)

    def test_shared_single_variant_contract_changes_refuse(self) -> None:
        for field, value in (
            ("profile", {"name": "producer", "opt_level": "0"}),
            ("features", ["serde", "std"]),
            ("dependencies", {"serde_core:other-unit": "other-unit"}),
        ):
            with self.subTest(field=field):
                self.output = self.root / f"conflicting-{field}"
                standard = self.shared_contract()
                with self.assertRaisesRegex(
                    ValueError, "incompatible shared unit contracts"
                ):
                    self.capture_shared_contracts(
                        {
                            "runtime": [standard],
                            "worker": [standard],
                            "python": [{**standard, field: value}],
                        }
                    )

    def test_overlap_without_subset_refuses_across_all_roles(self) -> None:
        standard = self.shared_contract()
        serde = {**standard, "features": ["serde", "std"]}
        alternate = {**standard, "features": ["bytemuck", "std"]}
        with self.assertRaisesRegex(ValueError, "incompatible shared unit contracts"):
            self.capture_shared_contracts(
                {
                    "runtime": [standard, serde],
                    "worker": [standard],
                    "python": [standard, alternate],
                }
            )

    def test_failed_actual_capture_stops_and_keeps_command_log(self) -> None:
        actual = self.actual_capture()

        def failed(
            command: list[str], **kwargs: object
        ) -> subprocess.CompletedProcess[bytes]:
            if command[:2] != ["just", "producer-identity"]:
                return actual(command, **kwargs)
            log = kwargs["stdout"]
            if not hasattr(log, "write"):
                self.fail("capture must preserve the diagnostic stream")
            log.write(b"source changed during actual build\n")
            return subprocess.CompletedProcess([], 1)

        with (
            patch.object(deployment.subprocess, "run", side_effect=failed) as run,
            self.assertRaisesRegex(ValueError, "actual producer capture failed"),
        ):
            deployment.capture(self.root, self.output, self.environment)
        self.assertEqual(run.call_count, 4)
        self.assertIn("source changed", (self.output / "runtime.log").read_text())
        self.assertTrue((self.output / "preparation/FAILED").is_file())

    def test_competing_strict_operation_refuses_before_dependency_effects(self) -> None:
        import fcntl

        active = self.root / "active"
        active.mkdir(mode=0o700)
        with deployment.own_preparation(self.root, active) as guard:
            guard.seek(0)
            original = guard.read()
            with (
                patch.object(deployment.subprocess, "run") as child,
                self.assertRaises(BlockingIOError),
            ):
                deployment.capture(self.root, self.output, self.environment)
            child.assert_not_called()
            guard.seek(0)
            self.assertEqual(guard.read(), original)
            # A failure by a competing owner never unlocks the active lifetime.
            with (self.root / "target/.producer-preparation.lock").open("r+b") as other:
                with self.assertRaises(BlockingIOError):
                    fcntl.flock(other, fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_actual_elf_installation_accepts_only_reviewed_rpath_replay(self) -> None:
        installed = self.root / "python/pse/_native.so"
        installed.parent.mkdir(parents=True)
        original = self.artifacts["python"]
        shutil.copyfile("/usr/bin/true", original)
        subprocess.run(
            ["patchelf", "--set-rpath", "/original", str(original)], check=True
        )
        shutil.copyfile(original, installed)
        subprocess.run(["patchelf", "--remove-rpath", str(installed)], check=True)
        subprocess.run(
            [
                "patchelf",
                "--force-rpath",
                "--set-rpath",
                "/original:/native:/native",
                str(installed),
            ],
            check=True,
        )
        self.output.mkdir()
        receipt = {
            "receipt_version": 2,
            "identity": "a" * 64,
            "selected_root": "python-fixture-root",
            "deployment": {
                "artifact": deployment.file_observation(original),
                "built_artifact": deployment.file_observation(original),
            },
        }
        path = self.output / "python.json"
        path.write_text(json.dumps(receipt))
        (self.output / "python-build-evidence.json").write_text(
            json.dumps(
                {
                    "stdout": json.dumps(
                        {
                            "reason": "build-script-executed",
                            "linked_libs": ["dylib=actual"],
                            "linked_paths": ["native=/native", "native=/native"],
                        }
                    )
                }
            )
        )
        actual_check_output = subprocess.check_output
        actual_run = subprocess.run
        history_evidence = {
            "success": True,
            **deployment.read_object(self.output / "python-build-evidence.json"),
        }
        (self.output / "python-build-evidence.json").unlink()

        def run(command: list[str], **kwargs: object) -> subprocess.CompletedProcess:
            if "producer-history-verify" in command:
                return subprocess.CompletedProcess(
                    command,
                    0,
                    stdout=json.dumps(
                        {
                            "version": 1,
                            "historical_only": True,
                            "producer_identity": receipt["identity"],
                            "selected_root": receipt["selected_root"],
                            "actual_build_evidence": history_evidence,
                        }
                    ),
                )
            return actual_run(command, **kwargs)

        def observed(
            command: list[str],
            *,
            text: bool,
            cwd: Path | None = None,
            env: dict[str, str] | None = None,
        ) -> str:
            self.assertTrue(text)
            if command[0] == str(self.root / ".venv/bin/python"):
                return json.dumps(str(installed))
            return actual_check_output(command, text=True, cwd=cwd, env=env)

        # The fixture tests byte-exact installation replay on actual ELF files.
        # Import/code mapping and consumed-input guards have separate real controls.
        with (
            patch.object(deployment.subprocess, "check_output", side_effect=observed),
            patch.object(deployment.subprocess, "run", side_effect=run),
            patch.object(deployment, "verify_artifact"),
        ):
            self.actual_bind(self.root, self.output, self.environment)
            rebound = deployment.read_object(path)
            association = rebound["deployment"]
            if not isinstance(association, dict):
                raise TypeError("Binding must retain an artifact association")
            self.assertEqual(rebound["identity"], receipt["identity"])
            self.assertEqual(
                association["artifact"],
                deployment.file_observation(installed),
            )
            self.assertEqual(
                association["built_artifact"],
                deployment.file_observation(original),
            )
            installation = deployment.read_object(
                self.output / "python-installation.json"
            )
            self.assertEqual(
                installation["raw_artifact"], deployment.file_observation(original)
            )
            self.assertEqual(
                installation["installed_artifact"],
                deployment.file_observation(installed),
            )
            transformation = installation["transformation"]
            self.assertIsInstance(transformation, dict)
            self.assertEqual(
                transformation["replayed_sha256"],
                deployment.file_observation(installed)["sha256"],
            )
            path.write_text(json.dumps(receipt))
            subprocess.run(
                ["patchelf", "--add-needed", "unreviewed.so", str(installed)],
                check=True,
            )
            with self.assertRaisesRegex(ValueError, "beyond.*RPATH-only"):
                self.actual_bind(self.root, self.output, self.environment)

    def test_deployment_environment_is_pure_and_selects_role_paths(self) -> None:
        environment = deployment.deployment_environment(self.root, self.output)
        self.assertFalse(self.output.exists())
        self.assertEqual(
            environment["PSE_RUNTIME_PRODUCER_RECEIPT"],
            str(self.output / "runtime.json"),
        )
        self.assertEqual(
            environment["PSE_WORKER_PRODUCER_RECEIPT"], str(self.output / "worker.json")
        )
        self.assertEqual(
            environment["PSE_PYTHON_PRODUCER_RECEIPT"], str(self.output / "python.json")
        )
        self.assertEqual(
            environment["PSE_WORKER_BINARY"],
            str(self.root / "target/producer/pse-worker"),
        )
        self.assertEqual(
            environment["PSE_PYTHON_DEPLOYMENT_ATTESTATION"],
            str(self.output / "python-attestation.json"),
        )
        self.assertEqual(
            environment, deployment.deployment_environment(self.root, Path("evidence"))
        )


if __name__ == "__main__":
    unittest.main()
