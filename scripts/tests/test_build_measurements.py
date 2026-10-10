# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated snapshot/protocol controls; never run a compiler or benchmark."""
# ruff: noqa: PT009, PT027 -- stdlib setup runner must work before pytest is installed

from __future__ import annotations

import json
import os
import signal
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import build_measurements, host_admission, native_operation, validation

if TYPE_CHECKING:
    from collections.abc import Callable


class BuildMeasurementTests(unittest.TestCase):
    @staticmethod
    def build_source_fixture(root: Path) -> dict[str, str]:
        files = {
            "crates/pse-compiler/src/physical_identity.rs": "h.u64(v.to_bits());\n",
            "crates/pse-compiler/src/lib.rs": "// Original public API.\n",
            "docs/dev/documentation.md": "Original documentation.\r\n",
        }
        for name, content in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        return files

    def test_three_edit_cycles_observe_only_snapshot_mutations_and_restoration(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(validation, "SOURCE_PATHS", (".",)),
        ):
            root = Path(directory) / "original"
            root.mkdir()
            self.build_source_fixture(root)
            subprocess.run(["git", "init", "--quiet"], cwd=root, check=True)
            subprocess.run(["git", "add", "--all"], cwd=root, check=True)
            index = (root / ".git/index").read_bytes()
            original = validation.sources(root)
            output = Path(directory) / "output"
            output.mkdir()
            source = build_measurements.snapshot(root, output)
            calls = []
            edits = {
                "private-edit": "crates/pse-compiler/src/physical_identity.rs",
                "public-edit": "crates/pse-compiler/src/lib.rs",
                "unrelated-edit": "docs/dev/documentation.md",
            }

            def sample(name: str) -> None:
                current = validation.sources(source)
                changed = {path for path in current if current[path] != original[path]}
                expected = {
                    path for kind, path in edits.items() if name.startswith(kind + "-")
                }
                self.assertEqual(changed, expected, name)
                self.assertEqual(validation.sources(root), original)
                calls.append(name)

            build_measurements.build_phases(source, 3, sample, screen=False)
            expected = ["cold"]
            for repetition in range(3):
                expected.extend(
                    f"{kind}-{repetition}"
                    for kind in (
                        "warm",
                        "private-edit",
                        "restore-private",
                        "public-edit",
                        "restore-public",
                        "unrelated-edit",
                        "restore-unrelated",
                    )
                )
            self.assertEqual(calls, expected)
            self.assertEqual(validation.sources(source), original)
            self.assertEqual(validation.sources(root), original)
            self.assertEqual((root / ".git/index").read_bytes(), index)

    def test_unrelated_edit_is_restored_when_measurement_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            files = self.build_source_fixture(source)

            def sample(name: str) -> None:
                if name == "unrelated-edit-0":
                    self.assertNotEqual(
                        (source / "docs/dev/documentation.md").read_bytes(),
                        files["docs/dev/documentation.md"].encode(),
                    )
                    raise RuntimeError("retained failed build")

            with self.assertRaisesRegex(RuntimeError, "retained failed build"):
                build_measurements.build_phases(source, 3, sample, screen=False)
            for name, original in files.items():
                self.assertEqual((source / name).read_bytes(), original.encode())

    def test_screen_mode_has_only_cold_and_three_warm_observations(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            calls = []
            build_measurements.build_phases(
                Path(directory), 3, calls.append, screen=True
            )
        self.assertEqual(calls, ["cold", "warm-0", "warm-1", "warm-2"])

    def test_default_target_does_not_prepare_unconsumed_native_capabilities(
        self,
    ) -> None:
        original = {"PATH": "/selected/tools", "CARGO_BUILD_JOBS": "2"}
        with patch.object(subprocess, "check_output") as command:
            actual = build_measurements.capability_environment(
                original, native=False, workflow=False
            )
        command.assert_not_called()
        self.assertEqual(actual, original)
        actual["new"] = "isolated"
        self.assertNotIn("new", original)

    def test_native_and_workflow_targets_prepare_their_linked_capabilities(
        self,
    ) -> None:
        for native, workflow in [(True, False), (False, True), (True, True)]:
            with (
                patch.object(
                    native_operation, "owner_record", return_value=Path("/operation")
                ),
                patch.object(
                    native_operation,
                    "environment",
                    return_value={"IPOPT_DIR": "/native/solver", "FLAGS": "x=y"},
                ) as command,
            ):
                actual = build_measurements.capability_environment(
                    {"PATH": "/tools"},
                    native=native,
                    workflow=workflow,
                )
            self.assertEqual(actual["IPOPT_DIR"], "/native/solver")
            self.assertEqual(actual["FLAGS"], "x=y")
            self.assertEqual(
                command.call_args.args,
                (["solver", "klu", "isolation", "uno", "petsc"], {"PATH": "/tools"}),
            )

    def test_selected_native_setup_failure_prevents_measurement(self) -> None:
        failure = subprocess.CalledProcessError(1, ["native-capability-setup"])
        with (
            patch.object(
                native_operation, "owner_record", return_value=Path("/operation")
            ),
            patch.object(native_operation, "environment", side_effect=failure),
            self.assertRaises(subprocess.CalledProcessError),
        ):
            build_measurements.capability_environment({}, native=True, workflow=False)

    def test_preparation_times_actual_admission_and_preserves_preexisting_evidence(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            owner = output / "operation.json"
            prior = {
                "version": native_operation.VERSION,
                "scope": {"unit": "owned", "group": "/owned", "invocation": "nonce"},
                "admissions": {
                    "capability:solver": {"inputs": {"IPOPT_DIR": "/solver"}}
                },
                "generations": ["/native/known-generation"],
            }
            owner.write_text(json.dumps(prior))
            calls = []

            def snapshot(_root: Path, _output: Path) -> Path:
                calls.append("snapshot")
                return output / "source"

            def admission(
                _requested: list[str], original: dict[str, str]
            ) -> dict[str, str]:
                calls.append("admission")
                updated = {
                    **prior,
                    "admissions": {
                        **prior["admissions"],
                        "capability:petsc": {"inputs": {"PETSC_DIR": "/petsc"}},
                    },
                    "generations": [*prior["generations"], "/native/petsc-generation"],
                }
                owner.write_text(json.dumps(updated))
                return {**original, "IPOPT_DIR": "/solver", "PETSC_DIR": "/petsc"}

            conditions: dict[str, object] = {
                "compiler_cache_selection": "on",
                "cold_compiler_cache": True,
            }
            with (
                patch.object(native_operation, "owner_record", return_value=owner),
                patch.object(native_operation, "environment", side_effect=admission),
                patch.object(build_measurements, "snapshot", side_effect=snapshot),
                patch.object(
                    build_measurements.time, "monotonic", side_effect=[10, 13, 20, 25]
                ),
                patch.object(build_measurements, "measure") as cargo,
            ):
                source, env, receipt = build_measurements.prepare_inputs(
                    output,
                    output,
                    {"PATH": "/tools"},
                    native=True,
                    workflow=False,
                    conditions=conditions,
                )
            cargo.assert_not_called()
            self.assertEqual(source, output / "source")
            self.assertEqual(env["PETSC_DIR"], "/petsc")
            self.assertEqual(calls, ["snapshot", "admission"])
            self.assertEqual(
                receipt["phases"],
                {
                    "snapshot": {"wall_seconds": 3, "status": "passed"},
                    "capability_environment": {"wall_seconds": 5, "status": "passed"},
                },
            )
            self.assertEqual(receipt["conditions"], conditions)
            operation = receipt["native_operation"]
            if not isinstance(operation, dict):
                self.fail("preparation operation evidence is not an object")
            self.assertTrue(operation["preexisting_at_measurement_target_entry"])
            self.assertEqual(operation["before"], prior)
            after = operation["after"]
            before = operation["before"]
            excludes = receipt["excludes"]
            if not isinstance(after, dict) or not isinstance(before, dict):
                self.fail("operation observations are not objects")
            if not isinstance(excludes, list):
                self.fail("preparation exclusions are not a list")
            self.assertIn("capability:petsc", after["admissions"])
            self.assertNotIn("capability:petsc", before["admissions"])
            self.assertIn("before target entry", excludes[0])
            self.assertEqual(
                json.loads((output / "preparation.json").read_text()), receipt
            )

    def test_failed_admission_keeps_its_timing_without_any_cargo_sample(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            with (
                patch.object(native_operation, "owner_record", return_value=None),
                patch.object(
                    build_measurements, "snapshot", return_value=output / "source"
                ),
                patch.object(
                    build_measurements,
                    "capability_environment",
                    side_effect=ValueError("admission refused"),
                ),
                patch.object(
                    build_measurements.time, "monotonic", side_effect=[10, 12, 20, 26]
                ),
                patch.object(build_measurements, "measure") as cargo,
                self.assertRaisesRegex(ValueError, "admission refused"),
            ):
                build_measurements.prepare_inputs(
                    output,
                    output,
                    {},
                    native=True,
                    workflow=False,
                    conditions={},
                )
            cargo.assert_not_called()
            receipt = json.loads((output / "preparation.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertEqual(receipt["phases"]["snapshot"]["wall_seconds"], 2)
            self.assertEqual(
                receipt["phases"]["capability_environment"],
                {
                    "wall_seconds": 6,
                    "status": "failed",
                    "error_type": "ValueError",
                },
            )

    def test_summary_keeps_cargo_sample_seconds_distinct_from_entry_phases(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            source = output / "source"
            source.mkdir()
            (source / "rust-toolchain.toml").write_text(
                '[toolchain]\nchannel="pinned-test"\n'
            )
            (output / "original-source.json").write_text("{}")

            def sample_builds(
                _source: Path,
                _count: int,
                sample: Callable[[str], None],
                *,
                screen: bool,
            ) -> None:
                self.assertTrue(screen)
                for name in ("cold", "warm-0", "warm-1", "warm-2"):
                    sample(name)

            with (
                patch.object(
                    build_measurements.sys,
                    "argv",
                    [
                        "build-measurements",
                        str(output),
                        "--screen",
                        "--cache",
                        "off",
                        "--second-worktree",
                    ],
                ),
                patch.object(
                    build_measurements, "ensure_capability_operation", return_value=None
                ),
                patch.object(native_operation, "owner_record", return_value=None),
                patch.object(validation, "fresh_output", return_value=output),
                patch.object(validation, "sources", return_value={}),
                patch.object(build_measurements, "snapshot", return_value=source),
                patch.object(
                    build_measurements, "capability_environment", return_value={}
                ),
                patch.object(
                    build_measurements.build_environment, "configure", return_value={}
                ),
                patch.object(
                    build_measurements.build_environment,
                    "effective_flags",
                    return_value=[],
                ),
                patch.object(
                    build_measurements.subprocess,
                    "check_output",
                    return_value="test host",
                ),
                patch.object(
                    build_measurements.shutil,
                    "disk_usage",
                    return_value=type("Disk", (), {"free": 1 << 60})(),
                ),
                patch.object(
                    build_measurements, "build_phases", side_effect=sample_builds
                ),
                patch.object(
                    build_measurements, "measure", return_value={"wall_seconds": 7}
                ) as cargo,
                patch.object(
                    build_measurements.time,
                    "monotonic",
                    side_effect=[100, 101, 104, 105, 110, 120, 123, 150],
                ),
            ):
                build_measurements.main()
            self.assertEqual(cargo.call_count, 5)
            summary = json.loads((output / "summary.json").read_text())
            self.assertEqual(summary["builds"]["cold"]["wall_seconds"], 7)
            self.assertEqual(summary["statistics"]["warm"]["median"], 7)
            self.assertEqual(
                summary["preparation"]["phases"]["snapshot"]["wall_seconds"], 3
            )
            self.assertEqual(
                summary["preparation"]["phases"]["capability_environment"][
                    "wall_seconds"
                ],
                5,
            )
            self.assertEqual(
                summary["preparation"]["phases"]["second_snapshot"]["wall_seconds"], 3
            )
            self.assertEqual(summary["builds"]["second-worktree"]["wall_seconds"], 7)
            self.assertEqual(summary["target_operation"]["wall_seconds"], 50)
            self.assertFalse(summary["target_operation"]["setup_inclusive"])

    def test_native_operation_precedes_snapshot_and_preserves_arguments(self) -> None:
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(native_operation, "owner_record", return_value=None),
            patch.object(validation, "fresh_output", return_value=Path("/output")),
            patch.object(native_operation, "run", return_value=0) as execute,
            patch.object(build_measurements, "complete_operation", return_value=0),
        ):
            build_measurements.ensure_capability_operation(
                Path("/checkout"), Path("/output"), native=True, workflow=False
            )
        self.assertEqual(
            execute.call_args.args[0][:3],
            ["/checkout/scripts/pse-env", "--native", "--"],
        )
        self.assertEqual(
            execute.call_args.args[1]["PSE_NATIVE_CAPABILITIES"],
            "solver,klu,isolation,uno,petsc",
        )
        self.assertEqual(
            execute.call_args.args[1][native_operation.BUILD_TIMING],
            "/output/native-operation.json",
        )
        with (
            patch.object(native_operation, "owner_record", return_value=None),
            self.assertRaisesRegex(ValueError, "before snapshot"),
        ):
            build_measurements.capability_environment({}, native=True, workflow=False)

    def test_compiler_launch_is_supervised_without_native_setup(self) -> None:
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(native_operation, "owner_record", return_value=None),
            patch.object(validation, "fresh_output", return_value=Path("/output")),
            patch.object(native_operation, "run", return_value=42) as execute,
            patch.object(build_measurements, "complete_operation") as complete,
        ):
            status = build_measurements.ensure_capability_operation(
                Path("/checkout"), Path("/output"), native=False, workflow=False
            )
        self.assertEqual(status, 42)
        self.assertEqual(
            execute.call_args.args[0][:2], ["/checkout/scripts/pse-env", "--"]
        )
        self.assertNotIn(native_operation.BUILD_TIMING, execute.call_args.args[1])
        self.assertNotIn("PSE_NATIVE_CAPABILITIES", execute.call_args.args[1])
        complete.assert_not_called()

    def test_existing_operation_remains_explicitly_target_only(self) -> None:
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(native_operation, "owner_record", return_value=Path("/owner")),
            patch.object(native_operation, "run") as execute,
            patch.object(validation, "fresh_output") as fresh,
        ):
            self.assertIsNone(
                build_measurements.ensure_capability_operation(
                    Path("/checkout"), Path("/output"), native=True, workflow=False
                )
            )
        execute.assert_not_called()
        fresh.assert_not_called()

    def test_complete_operation_uses_native_boundaries_and_verified_drain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            owner = {
                "scope": {"unit": "native", "group": "/group", "invocation": "nonce"}
            }
            native = {
                "setup_started_monotonic": 12,
                "setup_finished_monotonic": 19,
                "setup_status": "passed",
                "child_started_monotonic": 20,
                "child_finished_monotonic": 50,
                "child_exit_code": 0,
                "owner": owner,
                "host_allocation": {"path": "/allocations/selected"},
            }
            (output / "native-operation.json").write_text(json.dumps(native))
            (output / "summary.json").write_text(
                json.dumps({"builds": {"cold": {"wall_seconds": 7}}})
            )
            with (
                patch.object(native_operation, "drained", return_value=True) as drain,
                patch.object(
                    host_admission, "readonly_snapshot", return_value={"owners": {}}
                ),
                patch.object(build_measurements.time, "monotonic", return_value=55),
            ):
                self.assertEqual(
                    build_measurements.complete_operation(output, 10, 0), 0
                )
            drain.assert_called_once_with(owner)
            receipt = json.loads((output / "complete-operation.json").read_text())
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(receipt["wall_seconds"], 45)
            self.assertTrue(receipt["setup_inclusive"])
            self.assertEqual(receipt["phases"]["native_setup"]["wall_seconds"], 7)
            self.assertEqual(receipt["phases"]["measurement_child"]["wall_seconds"], 30)
            self.assertEqual(receipt["phases"]["final_drain"]["wall_seconds"], 5)
            summary = json.loads((output / "summary.json").read_text())
            self.assertEqual(summary["builds"]["cold"]["wall_seconds"], 7)
            self.assertEqual(summary["complete_operation"], receipt)

    def test_failed_drain_has_no_complete_wall_time_and_preserves_child_failure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            native = {
                "setup_started_monotonic": 12,
                "setup_finished_monotonic": 19,
                "setup_status": "passed",
                "child_started_monotonic": 20,
                "child_finished_monotonic": 50,
                "child_exit_code": 0,
                "owner": {"scope": None},
                "host_allocation": {"path": "/allocations/selected"},
            }
            (output / "native-operation.json").write_text(json.dumps(native))
            with (
                patch.object(native_operation, "drained", return_value=False),
                patch.object(
                    host_admission, "readonly_snapshot", return_value={"owners": {}}
                ),
                patch.object(build_measurements.time, "monotonic", return_value=55),
            ):
                for status, expected in ((0, 125), (42, 42), (143, 143)):
                    self.assertEqual(
                        build_measurements.complete_operation(output, 10, status),
                        expected,
                    )
            receipt = json.loads((output / "complete-operation.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertNotIn("wall_seconds", receipt)
            self.assertEqual(receipt["phases"]["final_drain"]["status"], "failed")
            self.assertNotIn("wall_seconds", receipt["phases"]["final_drain"])

    def test_failed_initial_setup_is_retained_without_child_or_success_claim(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            native = {
                "setup_started_monotonic": 12,
                "setup_finished_monotonic": 19,
                "setup_status": "failed",
                "error_type": "ValueError",
                "owner": {"scope": None},
                "host_allocation": {"path": "/allocations/selected"},
            }
            (output / "native-operation.json").write_text(json.dumps(native))
            with (
                patch.object(native_operation, "drained", return_value=True),
                patch.object(
                    host_admission, "readonly_snapshot", return_value={"owners": {}}
                ),
                patch.object(build_measurements.time, "monotonic", return_value=21),
            ):
                self.assertEqual(
                    build_measurements.complete_operation(output, 10, 125), 125
                )
            receipt = json.loads((output / "complete-operation.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertEqual(receipt["phases"]["native_setup"]["wall_seconds"], 7)
            self.assertNotIn("measurement_child", receipt["phases"])
            self.assertEqual(receipt["phases"]["final_drain"]["wall_seconds"], 2)

    def test_drained_main_scope_cannot_complete_with_live_registered_descendants(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            native = {
                "setup_started_monotonic": 12,
                "setup_finished_monotonic": 19,
                "setup_status": "passed",
                "child_started_monotonic": 20,
                "child_finished_monotonic": 50,
                "child_exit_code": 0,
                "owner": {"scope": {}},
                "host_allocation": {"path": "/allocations/selected"},
            }
            retained = {"units": {"surviving-worker": {"group": "/live"}}}
            (output / "native-operation.json").write_text(json.dumps(native))
            with (
                patch.object(native_operation, "drained", return_value=True),
                patch.object(
                    host_admission,
                    "readonly_snapshot",
                    return_value={"owners": {"selected": retained}},
                ),
                patch.object(
                    host_admission, "drained", return_value=False
                ) as host_drain,
                patch.object(build_measurements.time, "monotonic", return_value=55),
            ):
                self.assertEqual(
                    build_measurements.complete_operation(output, 10, 0), 125
                )
            host_drain.assert_called_once_with(retained)
            receipt = json.loads((output / "complete-operation.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertNotIn("wall_seconds", receipt)
            self.assertTrue(receipt["drain_observation"]["native_scope_drained"])
            self.assertFalse(receipt["drain_observation"]["host_allocation_drained"])

    def test_native_owner_times_failed_setup_before_any_child(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            owner = output / "owner.json"
            owner.write_text(
                json.dumps(
                    {
                        "version": native_operation.VERSION,
                        "admissions": {},
                        "generations": [],
                    }
                )
            )
            timing = output / "timing.json"
            allocation = type(
                "Allocation",
                (),
                {
                    "directory": output,
                    "nonce": "allocation",
                    "profile": type(
                        "Profile",
                        (),
                        {"name": "light", "memory": 512 << 20, "cores": (0,)},
                    )(),
                },
            )()
            with (
                patch.object(host_admission, "inherit", return_value=allocation),
                patch.object(native_operation, "current", return_value=owner),
                patch.object(
                    native_operation,
                    "_environment",
                    side_effect=ValueError("setup refused"),
                ),
                patch.object(native_operation.time, "monotonic", side_effect=[10, 17]),
                patch.object(native_operation, "run") as child,
                self.assertRaisesRegex(ValueError, "setup refused"),
            ):
                native_operation.environment(
                    ["solver"], {native_operation.BUILD_TIMING: str(timing)}
                )
            child.assert_not_called()
            receipt = json.loads(timing.read_text())
            self.assertEqual(receipt["setup_started_monotonic"], 10)
            self.assertEqual(receipt["setup_finished_monotonic"], 17)
            self.assertEqual(receipt["setup_status"], "failed")
            self.assertEqual(receipt["requested_capabilities"], ["solver"])

    def test_native_owner_child_timing_keeps_signal_status_and_removes_marker(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            owner = output / "owner.json"
            owner.write_text(
                json.dumps(
                    {
                        "version": native_operation.VERSION,
                        "admissions": {},
                        "generations": ["generation"],
                    }
                )
            )
            timing = output / "timing.json"
            timing.write_text(json.dumps({"setup_status": "passed"}))
            child = type(
                "Child",
                (),
                {"returncode": -signal.SIGTERM, "wait": lambda _self: -signal.SIGTERM},
            )()
            with (
                patch.object(native_operation, "current", return_value=owner),
                patch.object(
                    native_operation.subprocess, "Popen", return_value=child
                ) as launch,
                patch.object(native_operation.time, "monotonic", side_effect=[20, 50]),
            ):
                status = native_operation.run(
                    ["child"],
                    {native_operation.BUILD_TIMING: str(timing), "PRESERVE": "yes"},
                )
            self.assertEqual(status, 143)
            self.assertEqual(launch.call_args.kwargs["env"], {"PRESERVE": "yes"})
            receipt = json.loads(timing.read_text())
            self.assertEqual(receipt["child_started_monotonic"], 20)
            self.assertEqual(receipt["child_finished_monotonic"], 50)
            self.assertEqual(receipt["child_exit_code"], 143)
            self.assertEqual(receipt["owner"]["generations"], ["generation"])

    def test_profile_candidate_is_available_to_nested_cargo_commands(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            (source / ".cargo").mkdir()
            config = source / ".cargo/config.toml"
            config.write_text('[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\n')
            build_measurements.profile_override(source, 1)
            parsed = tomllib.loads(config.read_text())
            self.assertEqual(
                parsed["target"]["x86_64-unknown-linux-gnu"]["linker"], "clang"
            )
            self.assertEqual(parsed["profile"]["dev"]["package"]["*"], {"opt-level": 1})
            with self.assertRaisesRegex(ValueError, "profile policy"):
                build_measurements.profile_override(source, 2)

    def test_snapshot_keeps_dirty_deleted_untracked_ignored_tracked_modes_and_symlinks(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(validation, "SOURCE_PATHS", (".",)),
        ):
            root = Path(directory) / "original"
            root.mkdir()
            subprocess.run(["git", "init", "--quiet"], cwd=root, check=True)
            (root / ".gitignore").write_text("ignored\n")
            (root / "tracked").write_text("before\n")
            (root / "deleted").write_text("gone\n")
            (root / "ignored").write_text("tracked despite ignore\n")
            subprocess.run(["git", "add", "--force", "--all"], cwd=root, check=True)
            index = (root / ".git/index").read_bytes()
            (root / "deleted").unlink()
            (root / "tracked").write_text("after\n")
            (root / "tracked").chmod(0o755)
            (root / "new").write_text("new\n")
            (root / "link").symlink_to("tracked")
            before = validation.sources(root)
            output = Path(directory) / "output"
            output.mkdir()
            copied = build_measurements.snapshot(root, output)
            self.assertEqual((copied / "tracked").read_bytes(), b"after\n")
            self.assertEqual((copied / "tracked").stat().st_mode & 0o777, 0o755)
            self.assertEqual((copied / "link").readlink(), Path("tracked"))
            self.assertEqual(
                (copied / "ignored").read_bytes(), b"tracked despite ignore\n"
            )
            self.assertFalse((copied / "deleted").exists())
            self.assertEqual(validation.sources(root), before)
            self.assertEqual((root / ".git/index").read_bytes(), index)
            (copied / "tracked").write_text("private edit\n")
            self.assertEqual((root / "tracked").read_bytes(), b"after\n")

    def test_supported_artifact_messages_distinguish_fresh_rebuilt_and_incomplete_builds(
        self,
    ) -> None:
        artifact = {
            "reason": "compiler-artifact",
            "package_id": "registry+example#crate@1",
            "target": {"name": "crate", "kind": ["lib"]},
            "profile": {"test": False},
            "features": ["a"],
            "fresh": False,
        }
        finished = {"reason": "build-finished", "success": True}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cargo.jsonl"
            path.write_text(
                "\n".join(
                    map(json.dumps, [artifact, {**artifact, "fresh": True}, finished])
                )
            )
            result = build_measurements.artifacts(path)
            self.assertEqual(result["rebuilt_artifacts"], 1)
            self.assertEqual(result["fresh_artifacts"], 1)
            for messages in (
                [artifact],
                [artifact, {**finished, "success": False}],
                [finished],
                [artifact, finished, finished],
            ):
                path.write_text("\n".join(map(json.dumps, messages)))
                with self.assertRaises(ValueError):
                    build_measurements.artifacts(path)

    def test_new_cache_counters_are_counted_from_zero(self) -> None:
        self.assertEqual(
            build_measurements.counter_delta({"hits": {}}, {"hits": {"Rust": 2}}),
            {"hits": {"Rust": 2}},
        )

    def test_selected_roots_and_named_mixed_native_workloads(self) -> None:
        select = build_measurements.selected_packages
        self.assertEqual(
            select(None, native=False, workload="compiler"), ("pse-compiler",)
        )
        self.assertEqual(
            select(None, native=False, workload="compiler-relations"),
            ("pse-compiler", "pse-relations"),
        )
        self.assertEqual(
            select(
                ["pse-math", "pse-compiler", "pse-math"],
                native=False,
                workload="compiler",
            ),
            ("pse-math", "pse-compiler"),
        )
        self.assertEqual(
            select(None, native=True, workload="compiler"),
            build_measurements.NATIVE_PACKAGES,
        )
        for packages, native, workload in [
            (["pse-math"], True, "compiler"),
            (None, True, "compiler-relations"),
            (["pse-math"], False, "compiler-relations"),
        ]:
            with self.assertRaises(ValueError):
                select(packages, native=native, workload=workload)

    def test_selected_measurement_uses_snapshot_resolver_without_adding_roots(
        self,
    ) -> None:
        source, env = Path("/snapshot"), {"CARGO_BUILD_JOBS": "4"}
        cargo = ["rustup", "run", "pinned", "cargo"]
        calls = []
        package = {"id": "compiler", "name": "pse-compiler", "features": {}}
        unit = {
            "pkg_id": "compiler",
            "target": {
                "name": "compiler",
                "kind": ["lib"],
                "src_path": "/snapshot/compiler.rs",
            },
            "features": [],
            "dependencies": [],
            "platform": None,
            "mode": "test",
        }

        def capture(command: list[str], **kwargs: object) -> bytes:
            calls.append(command)
            self.assertEqual(kwargs, {"cwd": source, "env": env})
            return json.dumps(
                {"packages": [package]}
                if "metadata" in command
                else {"version": 1, "units": [unit], "roots": [0]}
            ).encode()

        with patch.object(subprocess, "check_output", side_effect=capture):
            for mode, targets in [("build", ["--tests"]), ("test", ["--lib"])]:
                args = [mode, "-p", "pse-compiler", *targets, "--locked"]
                actual = build_measurements.correctness_command(
                    source, cargo, args, env
                )
                self.assertEqual(actual, [*cargo, *args])
        self.assertEqual(len(calls), 4)
        self.assertTrue(all(command[:4] == cargo for command in calls))
        self.assertTrue(all("pse-relations" not in command for command in calls))

    def test_measurement_resolver_keeps_selected_arrow_owner_and_refuses_unvalidated_leaf(
        self,
    ) -> None:
        source, env = Path("/snapshot"), {}
        packages = [
            {"id": "owner", "name": "owner", "features": {"force-validate": []}},
            {"id": "arrow", "name": "arrow-data", "features": {"force_validate": []}},
        ]

        def graph(validated: bool) -> dict[str, object]:
            return {
                "version": 1,
                "roots": [0],
                "units": [
                    {
                        "pkg_id": "owner",
                        "target": {
                            "name": "owner",
                            "kind": ["lib"],
                            "src_path": "/owner.rs",
                        },
                        "features": [],
                        "dependencies": [{"index": 1}],
                        "platform": None,
                        "mode": "build",
                    },
                    {
                        "pkg_id": "arrow",
                        "target": {
                            "name": "arrow_data",
                            "kind": ["lib"],
                            "src_path": "/arrow.rs",
                        },
                        "features": ["force_validate"] if validated else [],
                        "dependencies": [],
                        "platform": None,
                        "mode": "build",
                    },
                ],
            }

        args = ["build", "--tests", "-p", "owner", "--locked"]
        for valid in (True, False):
            responses = [
                json.dumps(value).encode()
                for value in ({"packages": packages}, graph(False), graph(valid))
            ]
            with patch.object(subprocess, "check_output", side_effect=responses):
                if valid:
                    actual = build_measurements.correctness_command(
                        source, ["cargo"], args, env
                    )
                    self.assertEqual(
                        actual, ["cargo", *args, "--features", "owner/force-validate"]
                    )
                else:
                    with self.assertRaisesRegex(
                        build_measurements.arrow_validation.ValidationError,
                        "arrow-data lacks force_validate",
                    ):
                        build_measurements.correctness_command(
                            source, ["cargo"], args, env
                        )


if __name__ == "__main__":
    unittest.main()
