# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual product deployment operator preserves role values and refuses invalid policy."""

from __future__ import annotations

import copy
import json
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

import pytest

from scripts import execution_profiles as profiles
from scripts import host_admission as host
from scripts.generated import deployment

GIB = 1024**3


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
        assert {name.value for name in deployment.ProductionProfileName} == set(
            expected
        )
        for name, (total, server, worker, execution) in expected.items():
            with self.subTest(profile=name):
                allocation = profiles.profile(name)
                assert (
                    allocation["total_memory_bytes"],
                    allocation["server_memory_bytes"],
                    allocation["native_worker_memory_bytes"],
                ) == (total * GIB, server * GIB, worker * GIB)
                assert allocation["native_workers"] == 1
                if execution is None:
                    assert "execution" not in allocation
                else:
                    (
                        pool,
                        unit,
                        cpus,
                        lanes,
                        jobs,
                        compilers,
                        wait,
                        headroom,
                        observer,
                    ) = execution
                    assert allocation["execution"] == {
                        "pool_memory_bytes": pool * GIB,
                        "worker_bytes": unit * GIB,
                        "cpu_threads": cpus,
                        "case_lanes": lanes,
                        "math_jobs": jobs,
                        "compiler_cores": compilers,
                        "admission_wait_ms": wait,
                        "process_headroom_bytes": headroom * GIB,
                        "observer_memory_bytes": observer * GIB,
                    }

    def test_default_service_and_capability_defaults_are_separate_actual_declarations(
        self,
    ) -> None:
        declared = profiles.declarations()
        assert declared.default_profile is deployment.ProductionProfileName.REFERENCE
        assert profiles.default_allocation() == profiles.profile("reference")
        assert (
            declared.capabilities.memory_limit_bytes,
            declared.capabilities.threads,
            declared.capabilities.max_spill_bytes,
        ) == (128 * GIB, 16, GIB)

    def test_actual_materialization_validates_overrides_and_rejects_relational_changes(
        self,
    ) -> None:
        allocation = profiles.materialize(4 * GIB, GIB, 2, GIB)
        assert allocation["native_workers"] == 2
        profiles.validate(allocation)
        for changed in (
            {**allocation, "total_memory_bytes": GIB},
            {**allocation, "memory_threshold_bytes": 1},
            {**allocation, "surrealmx_max_async_queue_bytes": 1},
            {**allocation, "native_workers": 0},
        ):
            with (
                self.subTest(changed=changed),
                pytest.raises(profiles.DeploymentError),
            ):
                profiles.validate(changed)
        with pytest.raises(profiles.DeploymentError):
            profiles.materialize(2**63, GIB, 1, GIB)

    def test_changed_declaration_reaches_actual_operator_and_host_consumer(
        self,
    ) -> None:
        declaration = profiles.ROOT / ".config/agent-capacity.toml"
        original = declaration.read_text()
        marker = '[[production.profiles]]\nname = "functional"\n'
        assert original.count(marker) == 1
        prefix, remaining = original.split(marker, 1)
        functional, separator, suffix = remaining.partition("[[production.profiles]]")
        assert functional.count("admission_wait_ms = 30000") == 1
        changed = (
            prefix
            + marker
            + functional.replace(
                "admission_wait_ms = 30000", "admission_wait_ms = 12345", 1
            )
            + separator
            + suffix
        )
        expected_declaration = copy.deepcopy(tomllib.loads(original))
        selected = next(
            profile
            for profile in expected_declaration["production"]["profiles"]
            if profile["name"] == "functional"
        )
        selected["execution"]["admission_wait_ms"] = 12345
        assert tomllib.loads(changed) == expected_declaration
        operator = profiles.operator().absolute()
        original_profile = profiles.profile("functional")
        expected_profile = copy.deepcopy(original_profile)
        expected_execution = expected_profile["execution"]
        assert isinstance(expected_execution, dict)
        expected_execution["admission_wait_ms"] = 12345
        real_run = profiles.subprocess.run
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / ".config").mkdir(mode=0o700)
            (root / ".config/agent-capacity.toml").write_text(changed)
            with (
                patch.object(profiles, "ROOT", root),
                patch.object(profiles, "operator", return_value=operator),
                patch.object(profiles, "_REQUESTS", {}),
                patch.object(profiles, "_VALIDATED", set()),
                patch.object(profiles.subprocess, "run", wraps=real_run) as calls,
            ):
                actual = profiles.profile("functional")
                assert actual == expected_profile
                total, server, workers, worker = (
                    actual[key]
                    for key in (
                        "total_memory_bytes",
                        "server_memory_bytes",
                        "native_workers",
                        "native_worker_memory_bytes",
                    )
                )
                assert isinstance(total, int)
                assert isinstance(server, int)
                assert isinstance(workers, int)
                assert isinstance(worker, int)
                materialized = profiles.materialize(
                    total, server, workers, worker, actual["execution"]
                )
                assert materialized == actual
                assert host.execution("functional") == expected_execution
                assert host.execution("functional")["admission_wait_ms"] == 12345
                with patch.object(profiles, "_VALIDATED", set()):
                    profiles.validate_configuration(
                        {
                            "profile_version": 3,
                            "endpoint": "grpc://127.0.0.1:18080",
                            "namespace": "fixture",
                            "database": "canonical",
                            "credentials_file": str(root / "credentials.json"),
                            "max_message_bytes": 8 << 20,
                            "schema_interpretation": "fixture",
                            "resources": materialized,
                        }
                    )
                operations = {
                    json.loads(call.kwargs["input"])["operation"]
                    for call in calls.call_args_list
                    if call.args and call.args[0] == [str(operator)]
                }
                assert {"profile", "materialize", "validate"} <= operations
        assert declaration.read_text() == original

    def test_native_header_and_unknown_fields_refuse_before_state_effects(self) -> None:
        allocation = profiles.materialize(4 * GIB, GIB, 1, GIB)
        valid = {
            "document_version": 1,
            "operation": "validate",
            "allocation": allocation,
        }
        malformed = [
            {key: value for key, value in valid.items() if key != "document_version"},
            {**valid, "document_version": 2},
            {**valid, "unexpected": "field"},
        ]
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            for value in malformed:
                result = subprocess.run(
                    [str(profiles.operator())],
                    input=json.dumps(value).encode(),
                    cwd=directory,
                    capture_output=True,
                    check=False,
                )
                assert result.returncode != 0
                assert result.stdout == b""
            assert list(directory.iterdir()) == []


if __name__ == "__main__":
    unittest.main()
