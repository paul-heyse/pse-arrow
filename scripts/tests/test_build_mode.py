# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit uncapped ordinary work retains host ownership and scientific role limits."""

# ruff: noqa: PT009, PT027 -- unittest controls
from __future__ import annotations

import json
import subprocess
import tempfile
import time
import unittest
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable
from pathlib import Path
from unittest.mock import MagicMock, patch

from scripts import systemd_environment
from scripts import operation_lifetime
from scripts import host_admission as host
from scripts import native_operation as operation
from scripts import native_tests, pse_env
from scripts import surreal_server as server


class BuildModeTests(unittest.TestCase):
    def test_local_mode_changes_only_ordinary_defaults_and_preserves_explicit_choices(
        self,
    ) -> None:
        mode = {"PSE_BUILD_MODE": "unbounded"}
        for command in (
            ["cargo", "check"],
            ["just", "unit-package", "pse-runtime"],
            ["just", "codegen"],
            [".venv/bin/python", "-m", "scripts.python_tests", "--unit-only"],
        ):
            with self.subTest(command=command):
                self.assertEqual(
                    pse_env.selected_resource_class(command, mode, native=False),
                    "build",
                )
        for default in ("compile", "exclusive"):
            self.assertEqual(
                pse_env.selected_resource_class(
                    ["bash", "-euo", "pipefail"], mode, native=False, default=default
                ),
                "build",
            )
        for explicit in ("functional", "timing", "reference", "exclusive"):
            self.assertEqual(
                pse_env.selected_resource_class(
                    ["cargo", "build"], mode, native=False, explicit=explicit
                ),
                explicit,
            )
            self.assertEqual(
                pse_env.selected_resource_class(
                    ["cargo", "build"],
                    {**mode, "PSE_RESOURCE_CLASS": explicit},
                    native=False,
                ),
                explicit,
            )
        self.assertEqual(
            pse_env.selected_resource_class(
                ["cargo", "build"], {**mode, "PSE_MEMORY_MAX": "40G"}, native=False
            ),
            "compile",
        )
        self.assertEqual(
            pse_env.selected_resource_class(
                ["cargo", "build"],
                {**mode, operation.BUILD_TIMING: "receipt"},
                native=False,
            ),
            "compile",
        )
        self.assertEqual(
            pse_env.selected_resource_class(
                ["just", "unit-native-selected", "--managed-primary-route"],
                mode,
                native=True,
            ),
            "reference",
        )
        self.assertEqual(
            pse_env.selected_resource_class(["git", "status"], mode, native=False),
            "light",
        )

    def test_build_profile_has_no_memory_budget_and_preserves_timing_isolation(
        self,
    ) -> None:
        with patch.object(
            host, "memory_info", side_effect=AssertionError("no ordinary memory budget")
        ):
            build = host.select("build")
        self.assertEqual(build.memory, 0)
        self.assertTrue(build.unbounded)
        self.assertEqual(len(build.cores), 16)
        self.assertFalse(build.exclusive)
        self.assertNotIn("build", host.policy())
        with self.assertRaises(host.AdmissionError):
            host.select("build", "40G")

        def record(profile: host.Profile) -> dict[str, object]:
            return {
                "lane": profile.lane,
                "slots": profile.slots,
                "exclusive": profile.exclusive,
            }

        self.assertIsNone(host.conflict(build, {"other": record(build)}))
        self.assertIsNone(
            host.conflict(build, {str(index): record(build) for index in range(64)})
        )
        for bounded in (host.select("timing"), host.select("reference")):
            self.assertIsNotNone(host.conflict(build, {"other": record(bounded)}))
            self.assertIsNotNone(host.conflict(bounded, {"other": record(build)}))

    def test_parallel_owners_skip_memory_pressure_and_keep_unknown_lifetimes(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "admission"
            with (
                patch.object(host, "reconcile"),
                patch.object(
                    host,
                    "memory_info",
                    side_effect=AssertionError("no startup memory guard"),
                ),
            ):
                owners = [
                    host.acquire(host.select("build"), directory=directory)
                    for _ in range(17)
                ]
            with host.allocation_metadata(directory) as ledger:
                self.assertEqual(
                    set(ledger["owners"]), {owner.nonce for owner in owners}
                )
                record = ledger["owners"][owners[0].nonce]
                self.assertEqual(record["memory"], 0)
                self.assertEqual(len(record["cores"]), 16)
                host.validate_allocation_record(record)
                with self.assertRaises(host.AdmissionError):
                    host.validate_allocation_record(
                        {**record, "class": "functional", "lane": "functional"}
                    )
                record["units"]["unknown.scope"] = {}
            # An unbound registered lifetime cannot be silently considered drained.
            with patch.object(
                operation_lifetime,
                "unit_observation",
                return_value={"LoadState": "loaded", "ActiveState": "active"},
            ):
                self.assertFalse(owners[0].release())
            with host.allocation_metadata(directory) as ledger:
                self.assertIn(owners[0].nonce, ledger["owners"])

    def test_uncapped_readback_checks_the_selected_group_and_every_ancestor(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            parent, child = root / "psebuild.slice", root / "psebuild.slice/owner.slice"
            child.mkdir(parents=True)
            for directory in (root, parent, child):
                (directory / "memory.max").write_text("max")
            pse_env.require_unbounded_group(child, root=root)
            for directory in (root, parent, child):
                with self.subTest(directory=directory):
                    (directory / "memory.max").write_text("1048576")
                    with self.assertRaises(pse_env.BoundaryError):
                        pse_env.require_unbounded_group(child, root=root)
                    (directory / "memory.max").write_text("max")

    def test_build_parent_and_allocation_emit_no_memory_properties(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            owner = host.Allocation(
                Path(temporary), "a" * 32, host.select("build"), time.monotonic() + 10
            )
            self.assertIn(
                "psebuild.slice", str(pse_env.slice_group(host.allocation_slice(owner)))
            )
            self.assertNotIn(
                "pse.slice", pse_env.slice_group(host.allocation_slice(owner)).parts
            )
            with (
                patch.object(
                    host.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess([], 0),
                ) as run,
                patch.object(systemd_environment, "require_unbounded_slice") as readback,
                patch.object(host, "cpu_set", side_effect=tuple),
            ):
                host.enforce_parent(owner.profile, {"XDG_STATE_HOME": temporary})
                host.enforce_allocation(owner, {})
            commands = [call.args[0] for call in run.call_args_list]
            self.assertEqual(
                commands[0], ["systemctl", "--user", "start", "psebuild.slice"]
            )
            self.assertTrue(any("CPUQuota=1600%" in command for command in commands))
            self.assertFalse(
                any(
                    "MemoryMax=" in item or "MemorySwapMax=" in item
                    for command in commands
                    for item in command
                )
            )
            self.assertEqual(readback.call_count, 2)

    def test_uncapped_child_keeps_handoff_and_drain_and_bounded_scientific_child_cap(
        self,
    ) -> None:
        for cap in (None, 4 << 30):
            owner = MagicMock(spec=host.Allocation)
            owner.profile = host.select("build")
            owner.nonce = "a" * 32
            selected = {
                "unit": "child.scope",
                "group": "/child",
                "invocation": "b" * 32,
            }

            def completed(
                _command: list[str],
                _environment: dict[str, str],
                *,
                on_reaped: Callable[[], None],
                **_options: object,
            ) -> int:
                on_reaped()
                return 0

            with (
                self.subTest(cap=cap),
                patch.object(host, "inherit", return_value=owner),
                patch.object(operation_lifetime, "scope_owner", return_value={"unit": "parent"}),
                patch.object(systemd_environment, "current_memory_ceiling", return_value=cap),
                patch.object(systemd_environment, "require_unbounded_slice") as readback,
                patch.object(
                    operation, "prepare_handoff", return_value=Path("/owned/handoff")
                ) as handoff,
                patch.object(operation, "run", side_effect=completed) as run,
                patch.object(operation, "_record", return_value={"scope": selected}),
                patch.object(
                    operation_lifetime,
                    "drained",
                    return_value=operation_lifetime.DrainObservation(
                        selected, operation_lifetime.DrainPhase.COMPLETE, "controlled drain"
                    ),
                ) as drained,
            ):
                self.assertEqual(
                    operation.run_child_scope(["ordinary-workload"], {}), 0
                )
            command = run.call_args.args[0]
            if cap is None:
                self.assertFalse(
                    any(
                        "MemoryMax=" in item or "MemorySwapMax=" in item
                        for item in command
                    )
                )
            else:
                self.assertIn(f"--property=MemoryMax={cap}", command)
            owner.register.assert_called_once()
            owner.release.assert_not_called()
            handoff.assert_called_once()
            drained.assert_called()
            readback.assert_called_once()

    def test_ordinary_observer_has_no_os_cap_but_keeps_finite_scientific_profile(
        self,
    ) -> None:
        owner = host.Allocation(
            Path("/owned"), "a" * 32, host.select("build"), time.monotonic() + 10
        )
        resources = server.execution_resources("exclusive-observer")
        with (
            patch.object(server, "inherit_host_owner", return_value=owner),
            patch.object(
                server, "selected_slice", return_value=["--slice=owned.slice"]
            ),
            patch.object(
                server, "role_command", side_effect=lambda _allocation, command: command
            ),
            patch.object(pse_env, "require_unbounded_slice") as readback,
            patch.object(pse_env, "current_memory_ceiling", return_value=None),
            patch.object(server, "systemctl") as systemctl,
        ):
            command = server.observer_scope_command(
                Path("/state"), resources, "observer.scope", ["tests"], unbounded=True
            )
            with server.observer_control_limit(
                owner, "observer.scope", "exclusive-observer"
            ):
                pass
            with self.assertRaises(server.SupervisorError):
                server.observer_scope_command(
                    Path("/state"),
                    server.reference_resources(),
                    "observer.scope",
                    ["tests"],
                    unbounded=True,
                )
            with (
                self.assertRaises(server.SupervisorError),
                server.observer_control_limit(owner, "observer.scope", "reference"),
            ):
                pass
            systemctl.assert_not_called()
            self.assertEqual(readback.call_count, 2)
        self.assertFalse(
            any("MemoryMax=" in item or "MemorySwapMax=" in item for item in command)
        )
        self.assertGreater(server.integer(resources["native_worker_memory_bytes"]), 0)
        execution = server.object_mapping(resources["execution"])
        self.assertGreater(server.integer(execution["pool_memory_bytes"]), 0)

    def test_ordinary_execution_placement_records_actual_max_without_fake_ram_cap(
        self,
    ) -> None:
        owner = host.Allocation(
            Path("/owned"), "a" * 32, host.select("build"), time.monotonic() + 10
        )
        resources = server.execution_resources("exclusive-observer")
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            with (
                patch.object(server, "inherit_host_owner", return_value=owner),
                patch.object(server, "physical_cpus", return_value=list(range(16))),
                patch.object(host, "enforce_allocation"),
                patch.object(server, "group_for_slice", return_value=state),
                patch.object(pse_env, "require_unbounded_slice") as readback,
                patch.object(server, "effective_limits", return_value=(None, 16)),
                patch.object(
                    server, "host_memory", return_value=(192 << 30, 100 << 30)
                ),
                patch.object(server.shutil, "which", return_value="taskset"),
                patch.object(server, "systemd_environment", return_value={}),
            ):
                server.ensure_execution_placement(state, resources)
                observed = json.loads((state / "execution-placement.json").read_text())
                self.assertIsNone(observed["memory_max_bytes"])
                self.assertFalse(observed["physically_reserved"])
                readback.assert_called_once()
                with self.assertRaises(server.SupervisorError):
                    server.ensure_execution_placement(
                        state, server.reference_resources()
                    )

    def test_observer_readback_requires_actual_build_owner_for_unlimited_memory(
        self,
    ) -> None:
        resources = server.execution_resources("exclusive-observer")
        owner = host.Allocation(
            Path("/owned"), "a" * 32, host.select("build"), time.monotonic() + 10
        )
        environment = {
            "PSE_TEST_EXECUTION_PROFILE": "exclusive-observer",
            "PSE_SURREAL_STATE": "/state",
        }
        with (
            patch.object(server, "config_for", return_value={"resources": resources}),
            patch.object(server, "effective_limits", return_value=(None, 16)),
            patch.object(
                server,
                "placement_observation",
                return_value={"physical_cpus": list(range(16))},
            ),
            patch.object(server, "execution_slice", return_value="owned.slice"),
            patch.object(
                server, "group_for_slice", return_value=Path("/sys/fs/cgroup")
            ),
            patch.object(server, "role_affinity_ready", return_value=True),
            patch.object(host, "inherit", return_value=owner) as inherited,
            patch.object(pse_env, "require_unbounded_slice") as readback,
        ):
            observed = native_tests.observer_execution(environment)
            self.assertIsNone(observed["memory_max"])
            self.assertEqual(observed["resources"], resources)
            readback.assert_called_once()
            inherited.return_value = None
            with self.assertRaises(ValueError):
                native_tests.observer_execution(environment)
