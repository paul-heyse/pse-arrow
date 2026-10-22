# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cargo records and actual child streams retain their exact evidence scope."""
# ruff: noqa: PT009

from __future__ import annotations

import contextlib
import io
import json
import os
import sys
import tempfile
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from scripts import build_observation


class BuildObservationTests(unittest.TestCase):
    def test_actual_cold_build_rebuild_and_concurrent_artifact_producers(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "src").mkdir()
            (root / "Cargo.toml").write_text(
                '[package]\nname="pse-build-observation-fixture"\nversion="0.0.0"\nedition="2024"\n'
            )
            (root / "src/main.rs").write_text('fn main() { println!("first"); }\n')
            (root / "build.rs").write_text(
                "fn main() { std::thread::sleep(std::time::Duration::from_secs(1)); }\n"
            )
            environment = {
                **os.environ,
                "XDG_STATE_HOME": str(root / "state"),
                "RUSTC_WRAPPER": "",
                "RUSTC_WORKSPACE_WRAPPER": "",
                "PSE_COMPILER_CACHE_BYPASS_REASON": "private cold fixture explicitly disables wrappers",
            }
            command = [
                "cargo",
                "build",
                "--offline",
                "--manifest-path",
                str(root / "Cargo.toml"),
                "--message-format",
                "json",
            ]
            output, errors = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
                with ThreadPoolExecutor(max_workers=2) as pool:
                    codes = list(
                        pool.map(
                            lambda _index: build_observation.run_command(
                                command, env=environment
                            ),
                            range(2),
                        )
                    )
                self.assertEqual(codes, [0, 0])
                (root / "src/main.rs").write_text('fn main() { println!("second"); }\n')
                self.assertEqual(
                    build_observation.run_command(command, env=environment), 0
                )
            stages = build_observation.observations(
                root / "state/pse-arrow/host-admission"
            )
            self.assertEqual(len(stages), 3)
            self.assertTrue(all(item["exit_code"] == 0 for item in stages))
            self.assertTrue(any(item["compiled_artifacts"] > 0 for item in stages))
            self.assertTrue(any(item["fresh_artifacts"] > 0 for item in stages))
            self.assertTrue(
                all(item["cargo_lock_owner"] == "unknown" for item in stages)
            )
            self.assertIn("Blocking waiting for file lock", errors.getvalue())

    def test_real_child_json_freshness_is_not_a_cache_hit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            environment = {**os.environ, "XDG_STATE_HOME": temporary}
            output, errors = io.StringIO(), io.StringIO()
            command = [
                sys.executable,
                "-c",
                "import json,sys; print(json.dumps({'reason':'compiler-artifact','fresh':True})); print(json.dumps({'reason':'compiler-artifact','fresh':False})); print(json.dumps({'reason':'build-finished','success':False})); print('diagnostic',file=sys.stderr); sys.exit(7)",
            ]
            with contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
                self.assertEqual(
                    build_observation.run_command(command, env=environment), 7
                )
            stages = build_observation.observations(
                Path(temporary) / "pse-arrow/host-admission"
            )
            self.assertEqual(len(stages), 1)
            self.assertEqual(stages[0]["fresh_artifacts"], 1)
            self.assertEqual(stages[0]["compiled_artifacts"], 1)
            self.assertEqual(stages[0]["cache"]["hits"], "unknown")
            self.assertFalse(stages[0]["last_cargo_record"]["success"])
            self.assertEqual(stages[0]["exit_code"], 7)
            self.assertIn("diagnostic", errors.getvalue())
            self.assertEqual(len(output.getvalue().splitlines()), 3)

    def test_lock_and_cached_build_script_records_do_not_invent_current_stage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            observation = build_observation.BuildObservation(
                os.getpid(), {**os.environ, "XDG_STATE_HOME": temporary}
            )
            observation.message(
                b"Blocking waiting for file lock on artifact directory", "stderr"
            )
            observation.message(
                json.dumps(
                    {"reason": "build-script-executed", "package_id": "fixture"}
                ).encode(),
                "stdout",
            )
            self.assertEqual(observation.value["stage"], "cargo-lock-wait")
            self.assertEqual(observation.value["cargo_lock_owner"], "unknown")
            self.assertEqual(observation.value["cargo_records"], 1)
            self.assertEqual(
                observation.value["last_cargo_record"]["reason"],
                "build-script-executed",
            )


if __name__ == "__main__":
    unittest.main()
