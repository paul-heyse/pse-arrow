# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated snapshot/protocol controls; never run a compiler or benchmark."""
# ruff: noqa: PT009, PT027 -- stdlib setup runner must work before pytest is installed

from __future__ import annotations

import json
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import build_measurements, native_operation, validation


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

    def test_native_operation_precedes_snapshot_and_preserves_arguments(self) -> None:
        with (
            patch.object(native_operation, "owner_record", return_value=None),
            patch.object(build_measurements.os, "execvpe") as execute,
        ):
            build_measurements.ensure_capability_operation(
                Path("/checkout"), native=True, workflow=False
            )
        self.assertEqual(
            execute.call_args.args[1][:3],
            ["/checkout/scripts/pse-env", "--native", "--"],
        )
        self.assertEqual(
            execute.call_args.args[2]["PSE_NATIVE_CAPABILITIES"],
            "solver,klu,isolation,uno,petsc",
        )
        with (
            patch.object(native_operation, "owner_record", return_value=None),
            self.assertRaisesRegex(ValueError, "before snapshot"),
        ):
            build_measurements.capability_environment({}, native=True, workflow=False)

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

    def test_actual_unified_feature_mode_must_match_the_build_label(self) -> None:
        unit = {"target": {"name": "arrow_data"}, "features": ["force_validate"]}
        build_measurements.require_validation_mode([unit], True)
        build_measurements.require_validation_mode([{**unit, "features": []}], False)
        for units, expected in [
            ([unit], False),
            ([], True),
            ([unit, {**unit, "features": []}], True),
        ]:
            with self.assertRaises(ValueError):
                build_measurements.require_validation_mode(units, expected)


if __name__ == "__main__":
    unittest.main()
