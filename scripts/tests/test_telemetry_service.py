# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Telemetry configuration publication and exact native lifecycle ownership."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import telemetry_service as service
from scripts.tests import telemetry_pipeline_acceptance as acceptance

if TYPE_CHECKING:
    from collections.abc import Mapping


def mapping(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or any(not isinstance(key, str) for key in value):
        raise AssertionError("Expected a string-keyed object")
    return {key: item for key, item in value.items() if isinstance(key, str)}


def text(value: object) -> str:
    if not isinstance(value, str):
        raise TypeError("Expected a string")
    return value


class TelemetryServiceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.state = self.root / "state"
        self.binary = self.root / "native-fixture"
        self.binary.write_text("#!/bin/sh\nexit 0\n")
        self.binary.chmod(0o700)
        self.binaries = dict.fromkeys(service.TOOLS, self.binary)
        self.environment = {
            "HOME": str(self.root),
            "XDG_CONFIG_HOME": str(self.root / "config"),
        }
        self.calls: list[tuple[str, ...]] = []

    def systemctl(
        self, *arguments: str, **_options: object
    ) -> subprocess.CompletedProcess[str]:
        self.calls.append(arguments)
        return subprocess.CompletedProcess(
            arguments, 0, "LoadState=not-found\nActiveState=inactive\nMainPID=0\n", ""
        )

    def setup(
        self,
        *,
        ports: Mapping[str, int] | None = None,
        organization: str = "default",
        streams: Mapping[str, str] | None = None,
        retention_days: int = 7,
    ) -> dict[str, object]:
        with patch.object(service, "_systemctl", self.systemctl):
            return service.setup(
                self.state,
                environment=self.environment,
                binaries=self.binaries,
                ports=ports,
                organization=organization,
                streams=streams,
                retention_days=retention_days,
            )

    def test_setup_publishes_validated_selection_and_private_credentials(self) -> None:
        result = self.setup()
        public = json.loads((self.state / "configuration.json").read_text())
        self.assertEqual(public["schema_version"], 1)
        self.assertEqual(public["streams"], service.DEFAULT_STREAMS)
        self.assertEqual(public["otlp_http_endpoint"], "http://127.0.0.1:4318")
        self.assertEqual(result["configuration_id"], public["configuration_id"])
        self.assertEqual(
            mapping(result["endpoints"])["collector_health"], "http://127.0.0.1:13133/"
        )
        self.assertEqual(
            mapping(result["endpoints"])["collector_metrics"],
            "http://127.0.0.1:8888/metrics",
        )
        self.assertNotIn("Authorization", json.dumps(result))
        self.assertEqual((self.state / "service.env").stat().st_mode & 0o777, 0o600)
        base, headers = service.backend_access(self.state)
        self.assertEqual(base, public["backend_http_endpoint"])
        self.assertTrue(headers["Authorization"].startswith("Basic "))
        self.assertTrue(all(call[0] == "show" for call in self.calls))
        config = service._configuration(self.state)  # noqa: SLF001 -- verify sealed ownership records directly
        generation = Path(str(config["generation"]))
        pipeline = json.loads((generation / "collector.yaml").read_text())
        for signal in service.DEFAULT_STREAMS:
            exporter = pipeline["exporters"][f"otlp_http/{signal}"]
            self.assertEqual(exporter["sending_queue"]["storage"], "file_storage")
            self.assertEqual(exporter["sending_queue"]["queue_size"], 1000)
            self.assertEqual(exporter["retry_on_failure"]["max_elapsed_time"], "300s")
            self.assertEqual(
                exporter["headers"]["Authorization"],
                "${env:PSE_TELEMETRY_AUTHORIZATION}",
            )
        self.assertIn(
            'ZO_COMPACT_DATA_RETENTION_DAYS="7"',
            (self.state / "service.env").read_text(),
        )

    def test_rejected_collector_config_never_activates_producer_selection(self) -> None:
        self.binary.write_text("#!/bin/sh\nexit 1\n")
        with self.assertRaisesRegex(
            service.TelemetryServiceError, "Collector rejected"
        ):
            self.setup()
        self.assertFalse((self.state / "configuration.json").exists())
        self.assertFalse((self.state / "service.json").exists())

    def test_inspection_is_passive_and_absent_selection_creates_nothing(self) -> None:
        with patch.object(
            service,
            "_systemctl",
            side_effect=AssertionError("unexpected systemd operation"),
        ):
            self.assertEqual(service.inspect(self.state)["status"], "unconfigured")
        self.assertFalse(self.state.exists())
        self.setup()
        self.calls.clear()
        with patch.object(service, "_systemctl", self.systemctl):
            service.inspect(self.state)
        self.assertEqual([call[0] for call in self.calls], ["show", "show"])

    def test_exact_installer_is_shared_and_never_called_by_setup(self) -> None:
        with patch.object(
            service.diagnostic_trace, "install", return_value=self.binary
        ) as install:
            self.assertEqual(set(service.install(self.environment)), set(service.TOOLS))
        self.assertEqual(
            [call.args[0] for call in install.call_args_list],
            ["otelcol-contrib", "openobserve"],
        )
        with patch.object(
            service.diagnostic_trace,
            "install",
            side_effect=AssertionError("implicit install"),
        ):
            self.setup()

    def test_units_bind_persistent_paths_and_native_supervision(self) -> None:
        self.setup()
        config = service._configuration(self.state)  # noqa: SLF001 -- verify sealed ownership records directly
        self.assertEqual(config["state"], str(self.state))
        for binary in mapping(config["binaries"]).values():
            self.assertTrue(Path(text(mapping(binary)["path"])).is_absolute())
        for unit in mapping(config["units"]).values():
            unit_text = (Path(text(config["generation"])) / text(unit)).read_text()
            self.assertNotIn("surreal", unit_text)
            self.assertNotIn("MemoryMax", unit_text)
            self.assertIn("Restart=on-failure", unit_text)
            self.assertIn("WantedBy=default.target", unit_text)
            self.assertIn(f"WorkingDirectory={self.state}\n", unit_text)
            self.assertIn(f"EnvironmentFile={self.state / 'service.env'}\n", unit_text)

    def test_changed_generation_or_public_selection_refuses_control(self) -> None:
        self.setup()
        public = self.state / "configuration.json"
        record = json.loads(public.read_text())
        record["backend_http_endpoint"] = "http://127.0.0.1:9999"
        public.write_text(json.dumps(record))
        with (
            patch.object(
                service,
                "_systemctl",
                side_effect=AssertionError("control must refuse first"),
            ),
            self.assertRaisesRegex(
                service.TelemetryServiceError, "public configuration identity"
            ),
        ):
            service.stop(self.state)

    def test_public_credentials_or_binary_mutation_are_rejected(self) -> None:
        self.setup()
        credentials = self.state / "service.env"
        credentials.chmod(0o644)
        with self.assertRaisesRegex(service.TelemetryServiceError, "permissions"):
            service.backend_access(self.state)
        credentials.chmod(0o600)
        self.binary.write_text("changed")
        with self.assertRaisesRegex(service.TelemetryServiceError, "binary changed"):
            service.backend_access(self.state)

    def test_generated_password_always_satisfies_pinned_backend_policy(self) -> None:
        with patch.object(service.secrets, "token_urlsafe", return_value="a" * 43):
            self.setup()
        lines = dict(
            line.split("=", 1)
            for line in (self.state / "service.env").read_text().splitlines()
        )
        password = json.loads(lines["ZO_ROOT_USER_PASSWORD"])
        self.assertGreaterEqual(len(password), 8)
        self.assertLessEqual(len(password), 128)
        for pattern in (r"[a-z]", r"[A-Z]", r"[0-9]", r"[^a-zA-Z0-9]"):
            self.assertRegex(password, pattern)
        self.assertNotIn(password, json.dumps(service.inspect(self.state)))

    def test_owner_mismatch_never_stops_any_unit(self) -> None:
        self.setup()
        ownership = self.state / "service.json"
        record = json.loads(ownership.read_text())
        record["owner"] = "another-service"
        ownership.write_text(json.dumps(record))
        self.calls.clear()
        with (
            patch.object(service, "_systemctl", self.systemctl),
            self.assertRaisesRegex(service.TelemetryServiceError, "ownership differs"),
        ):
            service.stop(self.state)
        self.assertEqual(self.calls, [])

    def test_setup_preserves_nonempty_and_existing_state(self) -> None:
        self.setup()
        credentials = (self.state / "service.env").read_bytes()
        with self.assertRaisesRegex(
            service.TelemetryServiceError, "already configured"
        ):
            self.setup()
        self.assertEqual((self.state / "service.env").read_bytes(), credentials)
        foreign = self.root / "unrelated"
        foreign.mkdir(mode=0o700)
        sentinel = foreign / "retained-data"
        sentinel.write_text("preserve")
        with self.assertRaisesRegex(service.TelemetryServiceError, "empty owned state"):
            service.setup(foreign, binaries=self.binaries)
        self.assertEqual(sentinel.read_text(), "preserve")

    def test_enable_and_disable_only_change_explicit_login_selection(self) -> None:
        self.setup()
        self.calls.clear()
        with patch.object(service, "_systemctl", self.systemctl):
            service.enable(self.state, environment=self.environment)
            service.disable(self.state, environment=self.environment)
        operations = [call[0] for call in self.calls]
        self.assertIn("enable", operations)
        self.assertIn("disable", operations)
        self.assertNotIn("start", operations)
        self.assertNotIn("stop", operations)
        self.assertTrue((self.state / "configuration.json").is_file())

    def test_symlink_foreign_state_and_retention_are_refused(self) -> None:
        self.state.symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(service.TelemetryServiceError, "symlinks"):
            self.setup()
        self.state.unlink()
        with self.assertRaisesRegex(service.TelemetryServiceError, "three days"):
            self.setup(retention_days=1)
        with self.assertRaisesRegex(service.TelemetryServiceError, "distinct integers"):
            self.setup(ports={"otlp_http": 5080})
        self.state.mkdir(mode=0o755)
        with self.assertRaisesRegex(service.TelemetryServiceError, "private directory"):
            self.setup()

    def test_listener_collision_starts_neither_service(self) -> None:
        with socket.socket() as occupied:
            occupied.bind(("127.0.0.1", 0))
            port = occupied.getsockname()[1]
            self.setup(ports={"otlp_http": port})
            self.calls.clear()
            with (
                patch.object(service, "_systemctl", self.systemctl),
                self.assertRaisesRegex(
                    service.TelemetryServiceError, "listener collision"
                ),
            ):
                service.start(self.state, environment=self.environment)
        self.assertNotIn("start", [call[0] for call in self.calls])
        self.assertNotIn("enable", [call[0] for call in self.calls])

    def test_stop_refuses_changed_invocation_before_effect(self) -> None:
        self.setup()
        self.calls.clear()
        count = 0

        def changed(
            *arguments: str, **_options: object
        ) -> subprocess.CompletedProcess[str]:
            nonlocal count
            self.calls.append(arguments)
            count += 1
            invocation = "a" * 32 if count <= 2 else "b" * 32
            return subprocess.CompletedProcess(
                arguments,
                0,
                f"LoadState=loaded\nActiveState=active\nMainPID=123\nInvocationID={invocation}\n",
                "",
            )

        with (
            patch.object(service, "_systemctl", changed),
            patch.object(service, "_checked_process", return_value={"pid": 123}),
            self.assertRaisesRegex(
                service.TelemetryServiceError, "incarnation changed"
            ),
        ):
            service.stop(self.state)
        self.assertNotIn("stop", [call[0] for call in self.calls])

    def test_current_process_checks_executable_cgroup_and_incarnation(self) -> None:
        group = Path("/proc/self/cgroup").read_text().strip().split("::", 1)[1]
        unit = self.root / Path(group).name
        unit.write_text("fixture unit")
        config: dict[str, object] = {
            "owner_id": "selected-owner",
            "generation": str(self.root),
            "units": {"collector": unit.name},
            "binaries": {
                "collector": {
                    "path": str(Path(sys.executable).resolve()),
                    "identity": service._file_identity(Path(sys.executable).resolve()),  # noqa: SLF001 -- authenticate real executable identity
                }
            },
        }
        group = Path("/proc/self/cgroup").read_text().strip().split("::", 1)[1]
        observed = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "MainPID": str(os.getpid()),
            "InvocationID": "a" * 32,
            "ControlGroup": group,
            "FragmentPath": str(unit),
            "Description": "PSE telemetry collector selected-owner",
        }
        process = service._checked_process(config, "collector", observed)  # noqa: SLF001 -- exact owner verification seam
        if process is None:
            self.fail("The current process must be observed")
        self.assertEqual(process["pid"], os.getpid())
        ticks = process["start_ticks"]
        if not isinstance(ticks, int):
            self.fail("Process start ticks must be an integer")
        self.assertGreater(ticks, 0)
        for key, wrong in (
            ("Description", "foreign owner"),
            ("InvocationID", ""),
            ("ControlGroup", "/foreign"),
        ):
            with (
                self.subTest(key=key),
                self.assertRaises(service.TelemetryServiceError),
            ):
                service._checked_process(config, "collector", observed | {key: wrong})  # noqa: SLF001 -- reject changed ownership evidence

    def test_unit_paths_escape_specifiers_and_execution_variables(self) -> None:
        escaped = service._unit_quote('/fixture/a $b %c "d"')  # noqa: SLF001 -- exercise private unit-path escaping boundary
        self.assertEqual(
            escaped,
            '"/fixture/a $b %%c \\"d\\""',
        )
        self.assertIn("$$b", service._unit_quote("/fixture/$b", execution=True))  # noqa: SLF001 -- exercise private unit-path escaping boundary
        with self.assertRaisesRegex(
            service.TelemetryServiceError, "Invalid service path"
        ):
            service._unit_quote("/fixture/bad\npath")  # noqa: SLF001 -- exercise private unit-path escaping boundary

    def test_collector_internal_reader_pushes_to_existing_ingress(self) -> None:
        config = service.collector_configuration(
            self.state, service.DEFAULT_PORTS, "default", service.DEFAULT_STREAMS
        )
        readers = mapping(mapping(mapping(config["service"])["telemetry"])["metrics"])[
            "readers"
        ]
        if not isinstance(readers, list):
            self.fail("Internal readers must be a list")
        periodic = mapping(mapping(readers[0])["periodic"])
        self.assertEqual(periodic["interval"], 10000)
        self.assertEqual(
            mapping(periodic["exporter"])["otlp"],
            {"protocol": "grpc", "endpoint": "http://127.0.0.1:4317"},
        )
        self.assertEqual(
            mapping(
                mapping(mapping(mapping(readers[1])["pull"])["exporter"])["prometheus"]
            )["port"],
            8888,
        )
        self.assertEqual(set(mapping(config["receivers"])), {"otlp"})

    def test_reconfiguration_preserves_credentials_data_and_sealed_generation(
        self,
    ) -> None:
        before = self.setup()
        original = service._configuration(self.state)  # noqa: SLF001 -- Authenticate sealed fixture generation records and recovery effects.
        old_files = {
            path: (self.state / path).read_bytes()
            for path in mapping(original["files"])
        }
        sentinel = self.state / "openobserve-data" / "retained"
        sentinel.write_bytes(b"keep diagnostic data")
        with patch.object(service, "_systemctl", self.systemctl):
            after = service.reconfigure(self.state, environment=self.environment)
        self.assertNotEqual(before["configuration_id"], after["configuration_id"])
        self.assertEqual(after["retention_days"], 7)
        self.assertEqual(sentinel.read_bytes(), b"keep diagnostic data")
        self.assertEqual(
            old_files,
            {
                path: (self.state / path).read_bytes()
                for path in mapping(original["files"])
            },
        )
        selected = service._configuration(self.state)  # noqa: SLF001 -- Authenticate sealed fixture generation records and recovery effects.
        new_env = dict(
            line.split("=", 1)
            for line in Path(text(selected["environment_file"]))
            .read_text()
            .splitlines()
        )
        old_env = dict(
            line.split("=", 1)
            for line in (self.state / "service.env").read_text().splitlines()
        )
        self.assertEqual(
            new_env["PSE_TELEMETRY_AUTHORIZATION"],
            old_env["PSE_TELEMETRY_AUTHORIZATION"],
        )
        self.assertEqual(
            json.loads((self.state / "reconfiguration.json").read_text())["phase"],
            "activated",
        )
        self.assertNotIn("start", [call[0] for call in self.calls])

    def test_reconfiguration_validation_failure_leaves_previous_selection(self) -> None:
        self.setup()
        before = (self.state / "service.json").read_bytes()
        self.calls.clear()
        with (
            patch.object(service, "_systemctl", self.systemctl),
            patch.object(
                service.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 1),
            ),
            self.assertRaisesRegex(
                service.TelemetryServiceError, "prior generation remains"
            ),
        ):
            service.reconfigure(self.state, environment=self.environment)
        self.assertEqual((self.state / "service.json").read_bytes(), before)
        self.assertNotIn("stop", [call[0] for call in self.calls])

    def test_reconfiguration_failed_publication_restores_previous_selection(
        self,
    ) -> None:
        self.setup()
        before = service._configuration(self.state)  # noqa: SLF001 -- Authenticate sealed fixture generation records and recovery effects.
        published = []

        def fail_candidate(config: Mapping[str, object], _environment: object) -> None:
            published.append(config["configuration_id"])
            if config["configuration_id"] != before["configuration_id"]:
                raise service.TelemetryServiceError("fixture activation failed")

        with (
            patch.object(service, "_systemctl", self.systemctl),
            patch.object(service, "_publish_units", fail_candidate),
            self.assertRaisesRegex(
                service.TelemetryServiceError, "fixture activation failed"
            ),
        ):
            service.reconfigure(self.state, environment=self.environment)
        self.assertEqual(service._configuration(self.state), before)  # noqa: SLF001 -- Authenticate sealed fixture generation records and recovery effects.
        self.assertEqual(published[-1], before["configuration_id"])
        self.assertEqual(
            json.loads((self.state / "reconfiguration.json").read_text())["phase"],
            "rolled_back",
        )

    def test_explicit_recovery_repairs_interrupted_public_pointer(self) -> None:
        self.setup()
        before = service._configuration(self.state)  # noqa: SLF001 -- Authenticate sealed fixture generation records and recovery effects.
        with patch.object(service, "_systemctl", self.systemctl):
            service.reconfigure(self.state, environment=self.environment)
        (self.state / "configuration.json").write_text("{}")
        with patch.object(service, "_systemctl", self.systemctl):
            result = service.recover_previous_configuration(
                self.state, environment=self.environment
            )
        self.assertEqual(result["configuration_id"], before["configuration_id"])
        self.assertEqual(service._configuration(self.state), before)  # noqa: SLF001 -- Authenticate sealed fixture generation records and recovery effects.


class CollectorStopFixtureTests(unittest.TestCase):
    def test_stop_waits_for_authenticated_inactive_or_failed_settlement(self) -> None:
        for terminal in ("inactive", "failed"):
            with (
                self.subTest(terminal=terminal),
                tempfile.TemporaryDirectory() as directory,
            ):
                fixture = acceptance.Fixture(Path(directory) / "state", 2)
                before = {
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "MainPID": "99999999",
                    "InvocationID": "a" * 32,
                    "ControlGroup": "/pse-fixture-absent",
                }
                pending = {**before, "ActiveState": "deactivating"}
                after = {
                    **before,
                    "ActiveState": terminal,
                    "MainPID": "0",
                    "ControlGroup": "",
                }
                process = {
                    "pid": 99999999,
                    "start_ticks": 123,
                    "invocation_id": "a" * 32,
                }
                with (
                    patch.object(
                        service,
                        "_configuration",
                        return_value={"units": {"collector": "owned.service"}},
                    ),
                    patch.object(
                        service,
                        "_observe",
                        side_effect=[
                            before,
                            pending,
                            {**pending, "MainPID": "0"},
                            after,
                        ],
                    ),
                    patch.object(
                        service,
                        "_checked_process",
                        side_effect=[process, process, None],
                    ),
                    patch.object(service, "_systemctl") as control,
                ):
                    self.assertEqual(fixture.unit_action("collector", "stop"), process)
                control.assert_called_once_with("stop", "owned.service")

    def test_stop_refuses_unknown_or_replacement_incarnation(self) -> None:
        for changes in (
            {"LoadState": "unavailable"},
            {"InvocationID": "b" * 32},
            {"MainPID": "99999998"},
            {"ControlGroup": "/replacement"},
        ):
            with (
                self.subTest(changes=changes),
                tempfile.TemporaryDirectory() as directory,
            ):
                fixture = acceptance.Fixture(Path(directory) / "state", 2)
                before = {
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "MainPID": "99999999",
                    "InvocationID": "a" * 32,
                    "ControlGroup": "/pse-fixture-absent",
                }
                after = {**before, "ActiveState": "inactive", "MainPID": "0", **changes}
                with (
                    patch.object(
                        service,
                        "_configuration",
                        return_value={"units": {"collector": "owned.service"}},
                    ),
                    patch.object(service, "_observe", side_effect=[before, after]),
                    patch.object(
                        service,
                        "_checked_process",
                        return_value={"pid": 99999999, "start_ticks": 123},
                    ),
                    patch.object(service, "_systemctl"),
                    self.assertRaisesRegex(
                        acceptance.AcceptanceError, "fixture.stop_identity_changed"
                    ),
                ):
                    fixture.unit_action("collector", "stop")


if __name__ == "__main__":
    unittest.main()
