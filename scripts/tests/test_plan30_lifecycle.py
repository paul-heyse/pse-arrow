# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Focused controls for cross-context offline actions and portable owned closures."""

from __future__ import annotations

import contextlib
import hashlib
import json
import os
import secrets
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from collections.abc import Generator
from unittest.mock import MagicMock, patch

from scripts import host_admission, native_operation, test_resources
from scripts import surreal_server as server
from scripts.tests import surreal_fixture_check

startup_clock = server._STARTUP  # noqa: SLF001 -- checks restoration of the actual private lifecycle clock


class Plan30LifecycleTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.state = self.root / "state"
        self.state.mkdir()
        self.config = {
            "owner": server.OWNER,
            "profile_version": 2,
            "instance_id": "owned",
            "port": 18240,
            "endpoint": "ws://127.0.0.1:18240",
            "namespace": "pse",
            "database": "canonical",
            "interpretation": "test",
            "schema_interpretation": "test",
            "credentials_file": str(self.state / "credentials.json"),
            "admission": "open",
            "accepting_writes": True,
            "websocket_max_message_bytes": server.MESSAGE_BYTES,
            "max_message_bytes": server.MESSAGE_BYTES,
            "resources": server.resources(
                4 * server.GIB, 2 * server.GIB, 1, 2 * server.GIB
            ),
            "server": {"binary": str(self.root / "surreal"), "binary_sha256": "test"},
            "unit_materialized": True,
        }
        server.write_json(self.state / "config.json", self.config)
        server.write_json(
            self.state / "credentials.json",
            {
                "username": "owner",
                "password": "private",
                "selection_username": "pse-selection",
                "selection_password": 'viewer"\\secret',
            },
        )
        (self.state / "database").mkdir()
        (self.state / "database/fixture").write_bytes(b"immutable operation")
        manager = patch.object(
            server,
            "systemctl",
            return_value=subprocess.CompletedProcess([], 0, "inactive\n", ""),
        )
        manager.start()
        self.addCleanup(manager.stop)
        inactive = patch.object(server, "active", return_value=False)
        inactive.start()
        self.addCleanup(inactive.stop)
        drained = patch.object(server, "workers_drained")
        self.drained = drained.start()
        self.addCleanup(drained.stop)

    def context(self) -> Path:
        directory = self.state / ".receivers/generation/case"
        directory.mkdir(parents=True)
        selected = dict(self.config)
        selected.update(
            database="case",
            service_state=str(self.state),
            receiver_generation="generation",
        )
        server.write_json(directory / "config.json", selected)
        (self.state / ".contexts").mkdir()
        server.write_json(
            self.state / ".contexts/case.json",
            {
                "database": "case",
                "receiver_state": str(directory),
                "receiver_generation": "generation",
            },
        )
        return directory

    def generation(self) -> Path:
        files = {
            "scripts/surreal_server.py": hashlib.sha256(b"supervisor").hexdigest(),
            "scripts/sccache": hashlib.sha256(b"wrapper").hexdigest(),
            "bin/pse-worker": hashlib.sha256(b"worker").hexdigest(),
        }
        identity = hashlib.sha256(
            json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
        ).hexdigest()
        directory = self.state / ".generations" / identity
        (directory / "scripts").mkdir(parents=True)
        (directory / "bin").mkdir()
        (directory / "scripts/surreal_server.py").write_bytes(b"supervisor")
        (directory / "scripts/sccache").write_bytes(b"wrapper")
        (directory / "scripts/sccache").chmod(0o700)
        (directory / "bin/pse-worker").write_bytes(b"worker")
        (directory / "bin/pse-worker").chmod(0o700)
        server.write_json(
            directory / "generation.json",
            {"version": 1, "identity": identity, "files": files},
        )
        return directory

    def reservation(self, path: Path) -> None:
        server.write_json(
            path,
            {"pid": os.getpid(), "start": native_operation.start_identity(os.getpid())},
        )

    def test_frozen_cache_entrypoint_requires_its_execution_mode(self) -> None:
        generation = self.generation()
        server.verify_generation(generation)
        (generation / "scripts/sccache").chmod(0o600)
        with pytest.raises(server.SupervisorError, match="execution mode"):
            server.verify_generation(generation)

    def test_disposable_setup_explicitly_disables_resident_lifetime(self) -> None:
        state = self.root / "disposable"
        args = server.parser().parse_args(
            [
                "setup",
                "--state",
                str(state),
                "--tool-root",
                str(self.root / "tools"),
                "--port",
                "18249",
                "--interpretation",
                "test",
                "--no-resident",
                "--memory-mib",
                "1536",
                "--server-memory-mib",
                "1024",
                "--native-workers",
                "1",
                "--native-worker-memory-mib",
                "512",
            ]
        )
        with (
            patch.object(server, "install", return_value=self.config["server"]),
            patch.object(
                server,
                "publish_generation",
                return_value={"supervisor_script": "frozen"},
            ),
            patch.object(server, "public_status", return_value={}),
        ):
            server.setup(args)
        assert not (server.read_json(state / "config.json")["resident"])

    def test_restart_denial_requires_exact_unit_and_current_control_clock(self) -> None:
        unit = "owned.service"
        rows = [
            {
                "MESSAGE": f"{unit}: Start request repeated too quickly.",
                "__REALTIME_TIMESTAMP": "99000000",
            },
            {
                "MESSAGE": "other.service: Start request repeated too quickly.",
                "__REALTIME_TIMESTAMP": "101000000",
            },
        ]

        def output() -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(
                [], 0, "\n".join(json.dumps(row) for row in rows), ""
            )

        with patch.object(
            surreal_fixture_check.subprocess,
            "run",
            side_effect=lambda *_args, **_kwargs: output(),
        ):
            assert (
                surreal_fixture_check.restart_limit_denial(
                    unit, 100, time.monotonic() + 30
                )
            ) is (None)
            rows.append(
                {
                    "MESSAGE": f"{unit}: Start request repeated too quickly.",
                    "__REALTIME_TIMESTAMP": "102000000",
                }
            )
            denial = surreal_fixture_check.restart_limit_denial(
                unit, 100, time.monotonic() + 30
            )
            assert (denial) is not None

    def test_observer_command_routes_explicit_profile_to_owning_implementation(
        self,
    ) -> None:
        for declared, selected in (
            ("functional", "functional"),
            ("plan28-reference", "reference"),
        ):
            with (
                self.subTest(profile=declared),
                patch.object(server, "observer", return_value=0) as observer,
            ):
                args = server.parser().parse_args(
                    [
                        "observer",
                        "--state",
                        str(self.state),
                        "--execution-profile",
                        declared,
                        "--observer-command",
                        "true",
                    ]
                )
                assert (server.dispatch(args)) == (0)
                observer.assert_called_once_with(self.state, ["true"], profile=selected)

    def test_collected_disposable_unit_reset_does_not_refuse_recovery(self) -> None:
        with patch.object(
            server,
            "systemctl",
            side_effect=[
                subprocess.CompletedProcess([], 1, "", "unit not loaded"),
                subprocess.CompletedProcess(
                    [], 0, "ActiveState=inactive\nControlGroup=\n", ""
                ),
            ],
        ):
            server.reset_failure_window(self.state)
        with (
            patch.object(
                server,
                "systemctl",
                side_effect=[
                    subprocess.CompletedProcess([], 1, "", "denied"),
                    subprocess.CompletedProcess(
                        [], 0, "ActiveState=active\nControlGroup=/live\n", ""
                    ),
                ],
            ),
            pytest.raises(server.SupervisorError, match="unresolved live"),
        ):
            server.reset_failure_window(self.state)

    def test_live_native_free_borrower_refuses_recovery_before_storage_effects(
        self,
    ) -> None:
        record = {"state": str(self.state), "cleanup": "retained", "drained": False}
        with (
            patch.object(
                test_resources, "resource_status", return_value={"borrower": record}
            ),
            patch.object(test_resources, "borrower_alive", return_value=True),
            patch.object(server, "stop") as stop,
            patch.object(server, "administrative_query") as query,
        ):
            with pytest.raises(server.SupervisorError, match="still borrows"):
                server.qualify_recovery(self.state)
            stop.assert_not_called()
            query.assert_not_called()
        assert not ((self.state / "recovery-qualification.json").exists())

    def test_context_borrower_normalizes_service_and_terminal_resources_do_not_block(
        self,
    ) -> None:
        context = self.context()
        record = {"state": str(context), "cleanup": "retained", "drained": False}
        with (
            patch.object(
                test_resources, "resource_status", return_value={"borrower": record}
            ),
            patch.object(test_resources, "borrower_alive", return_value=True),
        ):
            with pytest.raises(server.SupervisorError, match="still borrows"):
                server.all_contexts_drained(self.state)
            record["drained"] = True
            server.all_contexts_drained(self.state)
            record.update(drained=False, cleanup="removed")
            server.all_contexts_drained(self.state)

    def test_live_cleanup_blocks_destructive_lifecycle_after_fixture_drain(
        self,
    ) -> None:
        cleaner = {
            "pid": os.getpid(),
            "start": native_operation.start_identity(os.getpid()),
        }
        record = {
            "state": str(self.state),
            "kind": "database",
            "cleanup": "removing",
            "drained": True,
            "cleaner": cleaner,
        }
        with (
            patch.object(
                test_resources, "resource_status", return_value={"resource": record}
            ),
            patch.object(
                test_resources,
                "borrower_alive",
                side_effect=lambda owner: owner is cleaner,
            ),
        ):
            with pytest.raises(server.SupervisorError, match="live context cleanup"):
                server.all_contexts_drained(self.state)
            record["kind"] = "evidence"
            server.all_contexts_drained(self.state)
            record["kind"] = "controls"
            with pytest.raises(server.SupervisorError, match="live context cleanup"):
                server.all_contexts_drained(self.state)
        with (
            patch.object(
                test_resources, "resource_status", return_value={"resource": record}
            ),
            patch.object(test_resources, "borrower_alive", return_value=False),
        ):
            server.all_contexts_drained(self.state)

    def test_lifecycle_blocks_new_context_before_publication(self) -> None:
        with (
            server.lifecycle_reservation(self.state),
            patch.object(server, "publish_generation") as publish,
        ):
            with pytest.raises(server.SupervisorError, match="lifecycle"):
                server.register_context(
                    self.state, "new_case", "functional", self.root / "worker"
                )
            publish.assert_not_called()
        assert not ((self.state / ".contexts").exists())

    def test_storage_affinity_is_independent_of_execution_role_width(self) -> None:
        binary = Path(self.config["server"]["binary"])
        binary.write_bytes(b"owned storage executable")
        self.config["server"]["binary_sha256"] = server.file_digest(binary)
        self.config.update(log_max_bytes=8 * server.MIB, log_backups=2)
        owner = MagicMock()
        owner.environment.return_value = {}
        owner.profile = host_admission.select("store-functional", str(2 * server.GIB))
        owner.directory = self.root / "admission"
        owner.nonce = "a" * 32
        ledger = {
            "owners": {
                owner.nonce: {
                    "units": {server.unit_name(self.state): {"group": "owned"}}
                }
            }
        }
        launch = {
            "allocation": str(owner.directory / owner.nonce),
            "deadline": time.monotonic() + 40,
        }

        class PlacementObservedError(Exception):
            pass

        for execution in (False, True):
            with self.subTest(execution=execution):
                (self.state / "service-launch.json").unlink(missing_ok=True)
                launch.pop("binding", None)
                self.config["resources"] = server.resources(
                    4 * server.GIB, 2 * server.GIB, 1, 2 * server.GIB
                )
                if execution:
                    self.config["resources"]["execution"] = {"cpu_threads": 2}
                with (
                    patch.object(server, "config_for", return_value=self.config),
                    patch.object(server, "service_allocation", return_value=owner),
                    patch.object(server, "storage_launch", return_value=launch),
                    patch.object(
                        host_admission,
                        "allocation_metadata",
                        return_value=contextlib.nullcontext(ledger),
                    ),
                    patch.object(host_admission, "enforce_parent"),
                    patch.object(
                        host_admission, "cpu_set", return_value=(0, 1, 16, 17)
                    ),
                    patch.object(
                        server, "physical_cpus", return_value=[0, 1]
                    ) as physical,
                    patch.object(server.os, "sched_setaffinity") as affinity,
                    patch.object(
                        server.subprocess, "Popen", side_effect=PlacementObservedError
                    ),
                    pytest.raises(PlacementObservedError),
                ):
                    server.serve(self.state)
                affinity.assert_called_once_with(0, list(owner.profile.cores))
                physical.assert_not_called()

    def test_storage_listener_checks_its_own_kernel_caps_and_identity(self) -> None:
        profile = host_admission.select("store-timing")
        owner = host_admission.Allocation(
            self.root / "admission", "a" * 32, profile, time.monotonic() + 40
        )
        self.config["resources"] = server.execution_resources("timing")
        self.config["service_class"] = "timing"
        group = "/pse.slice/" + server.unit_name(self.state)
        binding = {"group": group, "invocation": "b" * 32, "inode": 42}
        launch = {"allocation": str(owner.directory / owner.nonce), "binding": binding}
        server.write_json(
            self.state / "server-process.json",
            {
                "instance_id": "owned",
                "pid": 123,
                "start": "same-start",
                "allocation": launch["allocation"],
            },
        )
        proc = self.root / "kernel/proc/123"
        (proc / "fd").mkdir(parents=True)
        (proc / "cgroup").write_text("0::" + group + "\n")
        (proc / "fd/3").symlink_to("socket:[42]")
        network = self.root / "kernel/proc/net"
        network.mkdir()
        (network / "tcp").write_text(
            f"header\n0: 0100007F:{self.config['port']:04X} 0 0A 0 0 0 0 0 42\n"
        )
        cgroups = self.root / "kernel/cgroup"
        storage = cgroups / group.lstrip("/")
        storage.mkdir(parents=True)
        (storage / "memory.max").write_text(str(8 * server.GIB))
        (storage / "cpu.max").write_text("800000 100000")

        def path(*parts: object) -> Path:
            selected = Path(*(str(part) for part in parts))
            if selected.is_relative_to("/proc"):
                return self.root / "kernel/proc" / selected.relative_to("/proc")
            if selected.is_relative_to("/sys/fs/cgroup"):
                return cgroups / selected.relative_to("/sys/fs/cgroup")
            return selected

        observed = {
            "ActiveState": "active",
            "ControlGroup": group,
            "InvocationID": binding["invocation"],
        }
        with (
            patch.object(server, "Path", side_effect=path),
            patch.object(native_operation, "start_identity", return_value="same-start"),
            patch.object(
                server, "recorded_storage_owner", return_value=(owner, launch)
            ),
            patch.object(server, "storage_unit_observation", return_value=observed),
            patch.object(host_admission, "group_identity", return_value=42),
            patch.object(
                server, "effective_limits", return_value=(8 * server.GIB, 8.0)
            ),
            patch.object(
                server, "group_for_slice", return_value=cgroups / "pse.slice"
            ) as ancestry,
            patch.object(server, "role_affinity_ready", return_value=True) as affinity,
            patch.object(
                server,
                "execution_slice",
                side_effect=AssertionError("science ancestry is unrelated"),
            ),
        ):
            assert server.owns_listener(self.state, self.config)
            ancestry.assert_called_with("pse.slice")
            assert affinity.call_args.args[2] == list(profile.cores)
            (storage / "cpu.max").write_text("max 100000")
            assert not server.owns_listener(self.state, self.config)
            (storage / "cpu.max").write_text("800000 100000")
            observed["InvocationID"] = "c" * 32
            assert not server.owns_listener(self.state, self.config)

    def test_storage_launch_refuses_changed_generation_or_cap(self) -> None:
        launch = {
            "allocation": str(self.root / "admission" / ("a" * 32)),
            "deadline": time.monotonic() + 40,
            "generation": self.config["instance_id"],
            "supervisor_generation": self.config.get("service_supervisor"),
            "server_generation": self.config["server"],
            "service_class": "functional",
            "server_memory_bytes": 2 * server.GIB,
        }
        server.write_json(self.state / "service-launch.json", launch)
        assert server.storage_launch(self.state, self.config) == launch
        for key, value in (
            ("generation", "other"),
            ("server_memory_bytes", server.GIB),
            ("service_class", "timing"),
            ("server_generation", {"binary_sha256": "other"}),
        ):
            with self.subTest(key=key):
                server.write_json(
                    self.state / "service-launch.json", {**launch, key: value}
                )
                with pytest.raises(server.SupervisorError, match="service generation"):
                    server.storage_launch(self.state, self.config)

    def test_storage_restart_keeps_charged_owner_only_after_predecessor_drains(
        self,
    ) -> None:
        owner = host_admission.Allocation(
            self.root / "admission",
            "a" * 32,
            host_admission.select("store-functional"),
            time.monotonic() + 40,
        )
        group = "/pse.slice/" + server.unit_name(self.state)
        launch = {"binding": {"group": group, "invocation": "b" * 32, "inode": 42}}
        observed = {
            "ActiveState": "activating",
            "ControlGroup": group,
            "InvocationID": "c" * 32,
        }
        directory = self.root / "kernel"
        directory.mkdir()
        storage = directory / group.lstrip("/")
        storage.mkdir(parents=True)
        processes = storage / "cgroup.procs"
        processes.write_text(str(os.getpid()))
        with (
            patch.object(
                server, "recorded_storage_owner", return_value=(owner, launch)
            ),
            patch.object(server, "storage_unit_observation", return_value=observed),
            patch.object(native_operation, "process_group", return_value=group),
            patch.object(server, "Path", return_value=directory),
            patch.object(host_admission, "acquire") as acquire,
        ):
            assert server.resume_storage_allocation(self.state, self.config) is owner
            processes.write_text(f"{os.getpid()}\n123\n")
            with pytest.raises(server.SupervisorError, match="remain undrained"):
                server.resume_storage_allocation(self.state, self.config)
            processes.write_text(str(os.getpid()))
            observed["InvocationID"] = "b" * 32
            with pytest.raises(server.SupervisorError, match="uncertain"):
                server.resume_storage_allocation(self.state, self.config)
            acquire.assert_not_called()

    def test_persistent_storage_unit_materializes_storage_quota_and_ancestry(
        self,
    ) -> None:
        generation = self.generation()
        self.config["service_supervisor"] = {
            "supervisor_script": str(generation / "scripts/surreal_server.py"),
            "supervisor_executable": sys.executable,
        }
        self.config["resources"] = server.execution_resources("timing")
        self.config["service_class"] = "timing"
        for profile in (
            host_admission.select("store-timing"),
            host_admission.select("reference"),
        ):
            with self.subTest(profile=profile.name):
                owner = host_admission.Allocation(
                    self.root / "admission", "a" * 32, profile, time.monotonic() + 40
                )
                with (
                    patch.dict(
                        os.environ, {"XDG_CONFIG_HOME": str(self.root / "units")}
                    ),
                    patch.object(
                        host_admission,
                        "inherit",
                        side_effect=AssertionError("caller placement is unrelated"),
                    ),
                ):
                    server.materialize_service(self.state, self.config, owner)
                unit = (
                    self.root / "units/systemd/user" / server.unit_name(self.state)
                ).read_text()
                expected_slice = (
                    host_admission.allocation_slice(owner)
                    if profile.exclusive
                    else "pse.slice"
                )
                assert f"Slice={expected_slice}\n" in unit
                assert "MemoryMax=8589934592\n" in unit
                assert f"CPUQuota={len(profile.cores) * 100}%\n" in unit

    def test_storage_startup_does_not_materialize_science_roles(self) -> None:
        self.config["resources"] = server.execution_resources("timing")
        self.config["service_class"] = "timing"
        server.write_json(self.state / "config.json", self.config)
        owner = MagicMock()
        owner.profile = host_admission.select("store-timing")
        with (
            patch.object(
                server,
                "ensure_execution_placement",
                side_effect=AssertionError("science is admitted at primary startup"),
            ),
            patch.object(server, "service_allocation", return_value=owner) as admit,
            patch.object(host_admission, "enforce_parent"),
            patch.object(server, "materialize_service") as materialize,
            patch.object(server, "listener_ready", return_value=True),
            patch.object(server, "establish_protocol_readiness"),
        ):
            server.start(self.state, self.config, validation=True)
        assert admit.call_args.args[1] == self.config
        materialize.assert_called_once_with(self.state, self.config, owner)

    def test_absent_storage_owner_is_retained_as_refusal(self) -> None:
        launch = {
            "allocation": str(self.root / "admission" / ("a" * 32)),
            "deadline": time.monotonic() + 40,
            "generation": self.config["instance_id"],
            "supervisor_generation": self.config.get("service_supervisor"),
            "server_generation": self.config["server"],
            "service_class": "functional",
            "server_memory_bytes": 2 * server.GIB,
        }
        server.write_json(self.state / "service-launch.json", launch)
        with (
            patch.object(
                host_admission,
                "allocation_metadata",
                return_value=contextlib.nullcontext({"owners": {}}),
            ),
            patch.object(host_admission, "acquire") as acquire,
            pytest.raises(server.SupervisorError, match="absent or stale"),
        ):
            server.recorded_storage_owner(self.state, self.config)
        acquire.assert_not_called()

    def test_reference_demand_checks_existing_ready_and_stopped_parked_service(
        self,
    ) -> None:
        self.config["resources"] = server.reference_resources()
        for parked in (False, True):
            with self.subTest(parked=parked):
                self.config.update(
                    parked=parked,
                    admission="quiesced" if parked else "open",
                    accepting_writes=not parked,
                )
                server.write_json(self.state / "config.json", self.config)
                with (
                    patch.object(server, "start") as start,
                    patch.object(server, "setup") as setup,
                ):
                    assert (server.reference_state(self.state)) == (self.state)
                start.assert_called_once_with(self.state, self.config)
                setup.assert_not_called()

    def test_reference_demand_propagates_startup_refusal(self) -> None:
        self.config.update(
            resources=server.reference_resources(),
            admission="quiesced",
            accepting_writes=False,
        )
        server.write_json(self.state / "config.json", self.config)
        with (
            patch.object(
                server,
                "start",
                side_effect=server.SupervisorError("owned readiness refused"),
            ),
            pytest.raises(server.SupervisorError, match="owned readiness refused"),
        ):
            server.reference_state(self.state)
        assert not (server.config_for(self.state)["accepting_writes"])

    def test_reference_demand_uses_actual_service_owner_of_selected_receiver(
        self,
    ) -> None:
        self.config["resources"] = server.reference_resources()
        server.write_json(self.state / "config.json", self.config)
        context = self.context()
        with patch.object(server, "start") as start:
            assert (server.reference_state(context)) == (self.state)
        start.assert_called_once_with(self.state, self.config)

    def test_reference_demand_preserves_restored_validation_gate(self) -> None:
        self.config.update(
            resources=server.reference_resources(),
            admission="validation_required",
            accepting_writes=False,
            parked=True,
        )
        server.write_json(self.state / "config.json", self.config)
        with (
            patch.object(server, "service_allocation") as acquire,
            pytest.raises(server.SupervisorError, match="requires validate"),
        ):
            server.reference_state(self.state)
        acquire.assert_not_called()
        assert server.config_for(self.state)["parked"]

    def test_pending_context_primary_prevents_offline_copy(self) -> None:
        context = self.context()
        self.reservation(context / "primary-admission.json")
        with pytest.raises(server.SupervisorError, match="reservation"):
            server.backup(self.state, self.config, self.root / "backup")
        assert not ((self.root / "backup").exists())
        assert not (server.config_for(context)["accepting_writes"])

    def test_live_registered_observer_prevents_offline_copy(self) -> None:
        context = self.context()
        self.reservation(context / "primary-observer.json")
        with pytest.raises(server.SupervisorError, match="reservation"):
            server.backup(self.state, self.config, self.root / "backup")
        assert not ((self.root / "backup").exists())

    def test_every_registered_context_must_be_drained(self) -> None:
        context = self.context()

        def observed(state: Path, _config: dict[str, object]) -> None:
            if state == context:
                raise server.SupervisorError("context receiver still populated")

        self.drained.side_effect = observed
        with pytest.raises(server.SupervisorError, match="populated"):
            server.stop(self.state, self.config)
        assert ([call.args[0] for call in self.drained.call_args_list]) == (
            [self.state, context]
        )

    def test_dead_observer_leader_keeps_recorded_group_without_unit(self) -> None:
        context = self.context()
        server.write_json(
            context / "primary-observer.json",
            {"pid": 99999999, "start": "dead", "group": "/owned-observer.scope"},
        )
        with (
            patch.object(server, "group_populated", return_value=True),
            pytest.raises(server.SupervisorError, match="observer process group"),
        ):
            server.all_contexts_drained(self.state)
        with patch.object(server, "group_populated", return_value=False):
            server.all_contexts_drained(self.state)

    def test_explicit_parking_carries_original_clock_and_restores_after_failure(
        self,
    ) -> None:
        for previous, expected in ((None, 40.0), (25.0, 25.0)):
            with (
                self.subTest(previous=previous),
                patch.object(startup_clock, "deadline", previous, create=True),
            ):

                def failing_park(_state: Path, expected: float = expected) -> None:
                    assert startup_clock.deadline == expected
                    raise server.SupervisorError("bounded park failure")

                with (
                    patch.object(server, "_park_service", side_effect=failing_park),
                    pytest.raises(server.SupervisorError, match="bounded park failure"),
                ):
                    server.park_service(self.state, deadline=40.0)
                assert startup_clock.deadline == previous

    def test_graceful_stop_settles_deactivating_unit_even_when_not_active(self) -> None:
        with patch.object(
            server,
            "systemctl",
            return_value=subprocess.CompletedProcess([], 0, "inactive\n", ""),
        ) as manager:
            server.stop(self.state, self.config)
        assert any(call.args[:1] == ("stop",) for call in manager.call_args_list)
        assert (getattr(startup_clock, "deadline", None)) is (None)

    def test_abrupt_stop_wait_consumes_one_original_ninety_second_drain(self) -> None:
        states = iter(("deactivating\n", "failed\n", "failed\n"))
        clocks = iter((100.0, 100.0, 120.0))
        deadlines: list[float] = []

        def manager(
            *arguments: str, **_options: object
        ) -> subprocess.CompletedProcess[str]:
            deadlines.append(startup_clock.deadline)
            output = next(states) if "--property=ActiveState" in arguments else ""
            return subprocess.CompletedProcess([], 0, output, "")

        with (
            patch.object(server, "active", return_value=True),
            patch.object(server, "systemctl", side_effect=manager),
            patch.object(
                server.time, "monotonic", side_effect=lambda: next(clocks, 121.0)
            ),
            patch.object(server.time, "sleep"),
        ):
            server.stop(self.state, self.config, abrupt=True)
        assert deadlines
        assert (set(deadlines)) == ({190.0})
        assert (getattr(startup_clock, "deadline", None)) is (None)

    def test_backup_restore_owns_generations_and_reroots_contexts(self) -> None:
        context = self.context()
        generation = self.generation()
        receiver = {
            "supervisor_executable": str(Path(sys.executable).resolve()),
            "supervisor_script": str(generation / "scripts/surreal_server.py"),
            "worker_executable": str(generation / "bin/pse-worker"),
            "supervisor_sha256": server.file_digest(
                generation / "scripts/surreal_server.py"
            ),
            "worker_sha256": server.file_digest(generation / "bin/pse-worker"),
        }
        for directory in (self.state, context):
            config = server.config_for(directory)
            config.update(service_supervisor=receiver, primary_receiver=receiver)
            server.write_json(directory / "config.json", config)
        (context / "primary-receiver.json").write_text("obsolete live process receipt")
        backup = self.root / "backup"
        with server.lifecycle_reservation(self.state):
            server.backup(self.state, self.config, backup)
        self.state.rename(self.root / "original-unavailable")
        restored = self.root / "restored"
        server.restore(backup, restored, "test")
        selected = server.receiver_context(restored, "case")
        for directory in (restored, selected):
            config = server.config_for(directory)
            closure = Path(
                server.recorded_primary(config)["supervisor_script"]
            ).parents[1]
            server.verify_generation(closure)
            assert (closure.parent) == (restored / ".generations")
            assert (config["credentials_file"]) == (str(restored / "credentials.json"))
            assert (config["admission"]) == ("validation_required")
            assert not (config["unit_materialized"])
        assert not ((selected / "primary-receiver.json").exists())
        assert ((restored / "database/fixture").read_bytes()) == (
            b"immutable operation"
        )

    def test_restore_rejects_changed_generation_before_creating_state(self) -> None:
        generation = self.generation()
        self.config["service_supervisor"] = {
            "supervisor_script": str(generation / "scripts/surreal_server.py")
        }
        server.write_json(self.state / "config.json", self.config)
        backup = self.root / "backup"
        server.backup(self.state, self.config, backup)
        (backup / ".generations" / generation.name / "bin/pse-worker").write_bytes(
            b"changed"
        )
        with pytest.raises(server.SupervisorError, match="validation"):
            server.restore(backup, self.root / "restored", "test")
        assert not ((self.root / "restored").exists())

    def test_lifecycle_blocks_receiver_reservation_without_clock_leak(self) -> None:
        context = self.context()
        previous = getattr(startup_clock, "deadline", None)
        with (
            server.lifecycle_reservation(self.state),
            pytest.raises(server.SupervisorError, match="lifecycle"),
        ):
            server.ensure_primary(self.state, database="case")
        assert not ((context / "primary-admission.json").exists())
        assert (getattr(startup_clock, "deadline", None)) == (previous)

    def test_explicit_readmit_adds_selection_secret_and_preserves_credentials(
        self,
    ) -> None:
        self.config.update(admission="quiesced", accepting_writes=False)
        server.write_json(self.state / "config.json", self.config)
        old = {"username": "owner", "password": "private"}
        server.write_json(self.state / "credentials.json", old)
        (self.root / "surreal").write_bytes(b"binary")
        with patch.object(server, "publish_generation", return_value={}):
            server.readmit_supervisor(self.state)
        credentials = server.read_json(self.state / "credentials.json")
        assert (credentials["password"]) == (old["password"])
        assert (credentials["selection_username"]) == ("pse-selection")
        assert credentials["selection_password"]
        archives = list((self.state / ".profile-history").glob("*-credentials.json"))
        assert (server.read_json(archives[0])) == (old)

    def test_readiness_installs_viewer_and_checks_every_result(self) -> None:
        successful = subprocess.CompletedProcess(
            [], 0, '> [null, {"namespaces": {}}]\n', ""
        )
        with patch.object(server.subprocess, "run", return_value=successful) as run:
            server.establish_protocol_readiness(
                self.state, self.config, time.monotonic() + 10
            )
        query = run.call_args.kwargs["input"]
        assert ('PASSWORD "viewer\\"\\\\secret" ROLES VIEWER;') in (query)
        assert (run.call_args.kwargs["timeout"]) <= (10)
        assert (run.call_args.kwargs["cwd"]) == (self.state)
        assert self.state.stat().st_mode & 0o077 == 0
        failed = subprocess.CompletedProcess(
            [], 0, '["definition failed", {"namespaces": {}}]', ""
        )
        with (
            patch.object(server.subprocess, "run", return_value=failed),
            pytest.raises(server.SupervisorError, match="readiness failed"),
        ):
            server.establish_protocol_readiness(
                self.state, self.config, time.monotonic() + 10
            )

    def test_ordinary_readiness_does_not_migrate_old_credentials(self) -> None:
        old = {"username": "owner", "password": "private"}
        server.write_json(self.state / "credentials.json", old)
        with (
            patch.object(server.subprocess, "run") as run,
            pytest.raises(server.SupervisorError, match="explicitly readmit"),
        ):
            server.establish_protocol_readiness(
                self.state, self.config, time.monotonic() + 10
            )
        run.assert_not_called()
        assert (server.read_json(self.state / "credentials.json")) == (old)

    def test_administrative_context_override_is_exact_for_both_query_paths(
        self,
    ) -> None:
        response = MagicMock()
        response.__enter__.return_value.read.return_value = (
            b'[{"status":"OK","result":null}]'
        )
        deadline = time.monotonic() + 10
        with patch.object(
            server.urllib.request, "urlopen", return_value=response
        ) as request:
            server.administrative_query(
                self.state,
                "RETURN NONE;",
                deadline,
                namespace="pse_recovery",
                database="recovery_owned",
            )
        headers = dict(request.call_args.args[0].header_items())
        assert headers["Surreal-ns"] == "pse_recovery"
        assert headers["Surreal-db"] == "recovery_owned"
        with patch.object(server.http.client, "HTTPConnection") as connection:
            server.abandon_administrative_response(
                self.state,
                "RETURN NONE;",
                deadline,
                namespace="pse_recovery",
                database="recovery_owned",
            )
        transmitted = connection.return_value.request.call_args.kwargs["headers"]
        assert transmitted["surreal-ns"] == headers["Surreal-ns"]
        assert transmitted["surreal-db"] == headers["Surreal-db"]
        connection.return_value.getresponse.assert_not_called()
        connection.return_value.close.assert_called_once()

    def test_administrative_context_rejects_partial_or_unsafe_identifiers_before_ipc(
        self,
    ) -> None:
        for overrides in (
            {"namespace": "pse_recovery"},
            {"namespace": "pse_recovery", "database": "bad`identifier"},
            {"namespace": "", "database": "owned"},
        ):
            with (
                self.subTest(overrides=overrides),
                patch.object(server.urllib.request, "urlopen") as request,
                patch.object(server.http.client, "HTTPConnection") as connection,
            ):
                with pytest.raises(
                    server.SupervisorError, match="Administrative context"
                ):
                    server.administrative_query(
                        self.state,
                        "RETURN NONE;",
                        time.monotonic() + 10,
                        namespace=overrides.get("namespace"),
                        database=overrides.get("database"),
                    )
                with pytest.raises(
                    server.SupervisorError, match="Administrative context"
                ):
                    server.abandon_administrative_response(
                        self.state,
                        "RETURN NONE;",
                        time.monotonic() + 10,
                        namespace=overrides.get("namespace"),
                        database=overrides.get("database"),
                    )
            request.assert_not_called()
            connection.assert_not_called()

    def test_recovery_provisions_its_own_context_and_preserves_original_clock(
        self,
    ) -> None:
        clocks: list[float] = []
        selected_context: list[tuple[object, object]] = []

        def query(
            _state: Path, sql: str, deadline: float, **options: object
        ) -> list[dict[str, object]]:
            clocks.append(deadline)
            selected_context.append((options["namespace"], options["database"]))
            control = server.read_json(
                next((self.state / ".recovery-controls").glob("*/context.json"))
            )
            if sql.startswith("DEFINE NAMESPACE"):
                assert options["select_context"] is False
                assert "USE NS `pse_recovery`" in sql
                return [
                    {"status": "OK", "result": None},
                    {
                        "status": "OK",
                        "result": {"namespace": "pse_recovery", "database": None},
                    },
                    {"status": "OK", "result": None},
                ]
            acknowledged = {
                "operation": control["operation"],
                "payload": "acknowledged-v1",
            }
            if sql.startswith("UPSERT"):
                return [
                    {"status": "OK", "result": None},
                    {"status": "OK", "result": acknowledged},
                ]
            return [
                {"status": "OK", "result": acknowledged},
                {"status": "OK", "result": None},
            ]

        with (
            patch.object(server, "recovery_generation", return_value="same-generation"),
            patch.object(server, "start") as start,
            patch.object(server, "stop"),
            patch.object(server, "reset_failure_window"),
            patch.object(server, "administrative_query", side_effect=query),
            patch.object(server, "abandon_administrative_response") as abandon,
            patch.object(server, "materialize_service"),
            patch.object(server, "public_status", return_value={"qualified": True}),
        ):
            assert server.qualify_recovery(self.state) == {"qualified": True}
        assert len(set(clocks)) == 1
        assert all(
            call.kwargs["deadline"] == clocks[0] for call in start.call_args_list
        )
        assert len(set(selected_context)) == 1
        namespace, database = selected_context[0]
        assert database != self.config["database"]
        assert namespace == "pse_recovery"
        assert abandon.call_args.kwargs["namespace"] == namespace
        assert abandon.call_args.kwargs["database"] == database
        proof = server.read_json(self.state / "recovery-qualification.json")
        context = server.object_mapping(proof["administrative_context"])
        assert context["database"] == database
        assert server.read_json(Path(str(context["evidence"])))["outcome"] == "passed"

    def test_recovery_incomplete_provisioning_retains_context_without_enabling_restart(
        self,
    ) -> None:
        with (
            patch.object(server, "recovery_generation", return_value="same-generation"),
            patch.object(server, "start"),
            patch.object(server, "stop"),
            patch.object(server, "reset_failure_window"),
            patch.object(
                server,
                "administrative_query",
                return_value=[{"status": "OK", "result": None}],
            ),
            patch.object(server, "abandon_administrative_response") as abandon,
            patch.object(server, "materialize_service") as materialize,
            pytest.raises(server.SupervisorError, match="complete acknowledgments"),
        ):
            server.qualify_recovery(self.state)
        abandon.assert_not_called()
        materialize.assert_not_called()
        assert not server.config_for(self.state)["restart_qualified"]
        assert not (self.state / "recovery-qualification.json").exists()
        control = server.read_json(
            next((self.state / ".recovery-controls").glob("*/context.json"))
        )
        assert control["outcome"] == "incomplete"
        assert control["database"] != self.config["database"]

    def test_recovery_refuses_wrong_context_or_incomplete_statement_acknowledgments(
        self,
    ) -> None:
        selected = {"namespace": "pse_recovery", "database": None}
        malformed: list[list[dict[str, object]]] = [
            [{"status": "OK", "result": None} for _ in range(3)],
            [
                {"status": "OK", "result": None},
                {"status": "OK", "result": {"namespace": "other", "database": None}},
                {"status": "OK", "result": None},
            ],
            [
                {"status": "OK", "result": None},
                {
                    "status": "OK",
                    "result": {"namespace": "pse_recovery", "database": "other"},
                },
                {"status": "OK", "result": None},
            ],
            [
                {"status": "OK"},
                {"status": "OK", "result": selected},
                {"status": "OK", "result": None},
            ],
            [
                {"status": "ERR", "result": None},
                {"status": "OK", "result": selected},
                {"status": "OK", "result": None},
            ],
        ]
        for responses in malformed:
            with (
                self.subTest(responses=responses),
                patch.object(
                    server, "recovery_generation", return_value="same-generation"
                ),
                patch.object(server, "start"),
                patch.object(server, "stop"),
                patch.object(server, "reset_failure_window"),
                patch.object(server, "administrative_query", return_value=responses),
                patch.object(server, "abandon_administrative_response") as abandon,
                patch.object(server, "materialize_service") as materialize,
                pytest.raises(server.SupervisorError, match="complete acknowledgments"),
            ):
                server.qualify_recovery(self.state)
            abandon.assert_not_called()
            materialize.assert_not_called()
            assert not server.config_for(self.state)["restart_qualified"]
            assert not (self.state / "recovery-qualification.json").exists()

    def test_restart_requires_matching_actual_generation_proof(self) -> None:
        generation = self.generation()
        self.config["service_supervisor"] = {
            "supervisor_script": str(generation / "scripts/surreal_server.py")
        }
        binary = self.root / "surreal"
        binary.write_bytes(b"server")
        self.config["server"] = {
            "binary": str(binary),
            "binary_sha256": server.file_digest(binary),
        }
        self.config["restart_qualified"] = True
        assert not (server.recovery_qualified(self.state, self.config))
        fingerprint = server.recovery_generation(self.state, self.config)
        server.write_json(
            self.state / "recovery-qualification.json",
            {"version": 1, "outcome": "passed", "generation": fingerprint},
        )
        # This tests proof association only, not the recovery journey itself.
        assert server.recovery_qualified(self.state, self.config)
        credentials = server.read_json(self.state / "credentials.json")
        credentials["selection_password"] = secrets.token_urlsafe(24)
        server.write_json(self.state / "credentials.json", credentials)
        assert not (server.recovery_qualified(self.state, self.config))
        binary.write_bytes(b"replaced server")
        with pytest.raises(server.SupervisorError, match="artifact changed"):
            server.recovery_generation(self.state, self.config)

    def test_deliberate_start_unparks_but_waits_for_protocol_readiness(self) -> None:
        self.config.update(parked=True, admission="quiesced", accepting_writes=False)
        server.write_json(self.state / "config.json", self.config)

        def readiness(
            state: Path, _config: dict[str, object], _deadline: float
        ) -> None:
            current = server.config_for(state)
            assert not (current["parked"])
            assert not (current["accepting_writes"])

        owner = MagicMock()
        with (
            patch.object(server, "ensure_execution_placement"),
            patch.object(server, "service_allocation", return_value=owner),
            patch.object(host_admission, "enforce_parent"),
            patch.object(server, "materialize_service"),
            patch.object(server, "listener_ready", return_value=True),
            patch.object(
                server, "establish_protocol_readiness", side_effect=readiness
            ) as ready,
        ):
            server.start(self.state, self.config)
        ready.assert_called_once()
        current = server.config_for(self.state)
        assert not (current["parked"])
        assert current["accepting_writes"]
        assert (current["admission"]) == ("open")

    def test_automatic_serve_cannot_unpark_or_acquire_storage(self) -> None:
        self.config.update(parked=True, admission="quiesced", accepting_writes=False)
        server.write_json(self.state / "config.json", self.config)
        with (
            patch.object(server, "service_allocation") as acquire,
            patch.object(server.subprocess, "Popen") as spawn,
        ):
            assert (server.serve(self.state)) == (0)
        acquire.assert_not_called()
        spawn.assert_not_called()
        assert server.config_for(self.state)["parked"]

    def test_unpark_keeps_restored_validation_gate(self) -> None:
        self.config.update(
            parked=True, admission="validation_required", accepting_writes=False
        )
        server.write_json(self.state / "config.json", self.config)
        with pytest.raises(server.SupervisorError, match="requires validate"):
            server.unpark_service(self.state)
        assert server.config_for(self.state)["parked"]
        assert not (server.config_for(self.state)["accepting_writes"])

    def test_worker_launch_and_registration_do_not_hold_metadata_lock(self) -> None:
        lock = server.state_lock
        held = []

        @contextlib.contextmanager
        def tracked(state: Path) -> Generator[None, None, None]:
            with lock(state):
                held.append(state)
                try:
                    yield
                finally:
                    held.pop()

        def outside(
            *_args: object, **_kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            assert not (held)
            return subprocess.CompletedProcess([], 0, "inactive\n", "")

        child = MagicMock()
        child.poll.return_value = None
        child.wait.side_effect = lambda: (
            None if not (held) else pytest.fail("metadata lock remains held"),
            7,
        )[1]
        idle = {"LoadState": "not-found", "ActiveState": "inactive", "ControlGroup": ""}
        active = {"ActiveState": "active", "MemoryMax": str(2 * server.GIB)}
        with (
            patch.object(server, "state_lock", tracked),
            patch.object(server, "systemctl", side_effect=outside),
            patch.object(server, "settled_worker", return_value=idle),
            patch.object(server, "ensure_execution_placement", side_effect=outside),
            patch.object(server, "ready", return_value=True),
            patch.object(server, "worker_scope_command", return_value=["true"]),
            patch.object(server, "worker_environment", return_value={}),
            patch.object(server, "worker_observation", return_value=active),
            patch.object(native_operation, "prepare_handoff", return_value={}),
            patch.object(host_admission, "inherit", return_value=MagicMock()),
            patch.object(
                server.subprocess,
                "Popen",
                side_effect=lambda *_args, **_kwargs: (
                    None if not (held) else pytest.fail("metadata lock remains held"),
                    child,
                )[1],
            ),
        ):
            assert (server.worker(self.state, ["true"])) == (7)
        assert not (list(self.state.glob("worker-admission-*.json")))
