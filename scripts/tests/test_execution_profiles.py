# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual product deployment operator preserves role values and refuses invalid policy."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from scripts import execution_profiles as profiles
from scripts.generated import deployment

GIB = 1024 ** 3


class ExecutionProfileTests(unittest.TestCase):
    def test_actual_declared_roles_keep_their_original_distinct_scopes(self) -> None:
        expected = {
            "service": (48, 16, 32, None),
            "functional": (56, 16, 32, (24, 8, 16, 2, 4, 1, 30000, 8, 4)),
            "wide": (96, 16, 72, (64, 16, 16, 4, 8, 1, 30000, 8, 4)),
            "timing": (64, 8, 48, (40, 8, 16, 4, 8, 1, 30000, 8, 4)),
            "exclusive": (160, 16, 136, (128, 16, 16, 16, 32, 1, 30000, 8, 4)),
            "reference": (160, 16, 140, (128, 16, 16, 16, 32, 1, 30000, 12, 4)),
            "exclusive-observer": (160, 16, 72, (64, 16, 16, 4, 8, 1, 30000, 8, 64)),
        }
        self.assertEqual({name.value for name in deployment.ProductionProfileName}, set(expected))
        for name, (total, server, worker, execution) in expected.items():
            with self.subTest(profile=name):
                allocation = profiles.profile(name)
                self.assertEqual((allocation["total_memory_bytes"], allocation["server_memory_bytes"],
                                  allocation["native_worker_memory_bytes"]), (total * GIB, server * GIB, worker * GIB))
                self.assertEqual(allocation["native_workers"], 1)
                if execution is None:
                    self.assertNotIn("execution", allocation)
                else:
                    pool, unit, cpus, lanes, jobs, compilers, wait, headroom, observer = execution
                    self.assertEqual(allocation["execution"], {
                        "pool_memory_bytes": pool * GIB, "worker_bytes": unit * GIB,
                        "cpu_threads": cpus, "case_lanes": lanes, "math_jobs": jobs,
                        "compiler_cores": compilers, "admission_wait_ms": wait,
                        "process_headroom_bytes": headroom * GIB, "observer_memory_bytes": observer * GIB,
                    })

    def test_default_service_and_capability_defaults_are_separate_actual_declarations(self) -> None:
        declared = profiles.declarations()
        self.assertIs(declared.default_profile, deployment.ProductionProfileName.SERVICE)
        self.assertEqual(profiles.default_allocation(), profiles.profile("service"))
        self.assertEqual((declared.capabilities.memory_limit_bytes, declared.capabilities.threads,
                          declared.capabilities.max_spill_bytes), (16 * GIB, 16, GIB))

    def test_actual_materialization_validates_overrides_and_rejects_relational_changes(self) -> None:
        allocation = profiles.materialize(4 * GIB, GIB, 2, GIB)
        self.assertEqual(allocation["native_workers"], 2)
        profiles.validate(allocation)
        for changed in (
            {**allocation, "total_memory_bytes": GIB},
            {**allocation, "memory_threshold_bytes": 1},
            {**allocation, "surrealmx_max_async_queue_bytes": 1},
            {**allocation, "native_workers": 0},
        ):
            with self.subTest(changed=changed), self.assertRaises(profiles.DeploymentError):
                profiles.validate(changed)
        with self.assertRaises(profiles.DeploymentError):
            profiles.materialize(2 ** 63, GIB, 1, GIB)

    def test_native_header_and_unknown_fields_refuse_before_state_effects(self) -> None:
        allocation = profiles.materialize(4 * GIB, GIB, 1, GIB)
        valid = {"document_version": 1, "operation": "validate", "allocation": allocation}
        malformed = [
            {key: value for key, value in valid.items() if key != "document_version"},
            {**valid, "document_version": 2}, {**valid, "unexpected": "field"},
        ]
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            for value in malformed:
                result = subprocess.run([str(profiles.operator())], input=json.dumps(value).encode(),
                                        cwd=directory, capture_output=True, check=False)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, b"")
            self.assertEqual(list(directory.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
