# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Mocked controls for source-reviewed actual deployment capture orchestration."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import json
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
        self.outer = {"source": "blake3:" + "a" * 64, "build": "blake3:" + "b" * 64}
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

    def actual_capture(
        self,
        change: Callable[[str, dict[str, object], dict[str, object]], None]
        | None = None,
    ) -> Callable[..., subprocess.CompletedProcess[bytes]]:
        def run(
            command: list[str],
            *,
            cwd: Path,
            env: dict[str, str],
            stdout: BinaryIO,
            stderr: int,
            check: bool,
        ) -> subprocess.CompletedProcess[bytes]:
            self.assertEqual(cwd, self.root)
            self.assertEqual(stderr, subprocess.STDOUT)
            self.assertFalse(check)
            output = Path(command[command.index("--output") + 1])
            name = output.stem
            expected_env = dict(self.environment)
            if name == "python":
                expected_env["PYO3_BUILD_EXTENSION_MODULE"] = "1"
            else:
                expected_env.pop("PYO3_BUILD_EXTENSION_MODULE", None)
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
                "outer_attestation": None if name == "runtime" else dict(self.outer),
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
            stdout.write(b"actual capture diagnostics\n")
            return subprocess.CompletedProcess(command, 0)

        return run

    def test_capture_orders_actual_targets_and_preserves_safe_declared_environment(
        self,
    ) -> None:
        self.environment.update(
            TZ="UTC",
            SOURCE_DATE_EPOCH="0",
            LIBCLANG_PATH="selected-libclang",
            CFLAGS_x86_64_unknown_linux_gnu="-O2",
            LLVM_CONFIG_PATH="selected-llvm",
            PYO3_BUILD_EXTENSION_MODULE="",
            SYMBOLICA_LICENSE="private-secret",
            SURREAL_PASS=str(self.root / "unused-surreal-credential"),
        )
        before = {name: path.read_bytes() for name, path in self.declarations.items()}
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ) as run:
            environment = deployment.capture(self.root, self.output, self.environment)
        commands = [call.args[0] for call in run.call_args_list]
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
            self.assertNotIn("--outer-source", command)
            self.assertNotIn("--outer-build", command)
            self.assertIn("TZ=UTC", command)
            self.assertIn("SOURCE_DATE_EPOCH=0", command)
            self.assertIn("CFLAGS_x86_64_unknown_linux_gnu=-O2", command)
            self.assertIn("LLVM_CONFIG_PATH=selected-llvm", command)
            if name == "python":
                self.assertIn("PYO3_BUILD_EXTENSION_MODULE=1", command)
            else:
                self.assertFalse(
                    any(
                        item.startswith("PYO3_BUILD_EXTENSION_MODULE=")
                        for item in command
                    )
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
            deployment.validate_receipts(self.root, self.output), self.outer
        )

    def test_python_native_inputs_can_use_common_reviewed_list(self) -> None:
        self.environment.pop("PSE_PYTHON_PRODUCER_INPUTS")
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ) as run:
            deployment.capture(self.root, self.output, self.environment)
        for call in run.call_args_list:
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
        self.assertEqual(run.call_count, 2)
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
        self.assertEqual(run.call_count, 3)
        self.assertTrue((self.output / "deployment-error.log").is_file())

    def test_outer_mismatch_and_missing_worker_attestation_are_refused(self) -> None:
        for value in (
            None,
            {"source": "blake3:" + "d" * 64, "build": self.outer["build"]},
        ):
            with self.subTest(value=value):
                output = self.root / ("absent" if value is None else "mismatch")

                def change(
                    name: str,
                    receipt: dict[str, object],
                    _evidence: dict[str, object],
                    *,
                    value: dict[str, str] | None = value,
                ) -> None:
                    if name == "worker":
                        receipt["outer_attestation"] = value

                with (
                    patch.object(
                        deployment.subprocess,
                        "run",
                        side_effect=self.actual_capture(change),
                    ),
                    self.assertRaisesRegex(ValueError, "attestation"),
                ):
                    deployment.capture(self.root, output, self.environment)

    def test_missing_actual_worker_binary_prevents_success(self) -> None:
        (self.root / "target/producer/pse-worker").unlink()
        with (
            patch.object(
                deployment.subprocess, "run", side_effect=self.actual_capture()
            ) as run,
            self.assertRaisesRegex(ValueError, "worker binary is missing"),
        ):
            deployment.capture(self.root, self.output, self.environment)
        self.assertEqual(run.call_count, 3)
        self.assertTrue((self.output / "deployment-error.log").is_file())

    def test_independent_outer_observation_is_required_to_match(self) -> None:
        with patch.object(
            deployment.subprocess, "run", side_effect=self.actual_capture()
        ):
            deployment.capture(self.root, self.output, self.environment)
        deployment.validate_receipts(self.root, self.output, expected_outer=self.outer)
        with self.assertRaisesRegex(ValueError, "independently observed"):
            deployment.validate_receipts(
                self.root,
                self.output,
                expected_outer={
                    "source": self.outer["source"],
                    "build": "blake3:" + "e" * 64,
                },
            )

    def test_failed_actual_capture_stops_and_keeps_command_log(self) -> None:
        def failed(
            _command: list[str], **kwargs: object
        ) -> subprocess.CompletedProcess[bytes]:
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
        self.assertEqual(run.call_count, 1)
        self.assertIn("source changed", (self.output / "runtime.log").read_text())

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
