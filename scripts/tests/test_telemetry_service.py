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
from unittest.mock import patch

from scripts import telemetry_service as service


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
        self.environment = {"HOME": str(self.root), "XDG_CONFIG_HOME": str(self.root / "config")}
        self.calls: list[tuple[str, ...]] = []

    def systemctl(self, *arguments: str, **_options: object) -> subprocess.CompletedProcess[str]:
        self.calls.append(arguments)
        return subprocess.CompletedProcess(arguments, 0, "LoadState=not-found\nActiveState=inactive\nMainPID=0\n", "")

    def setup(self, **options: object) -> dict[str, object]:
        with patch.object(service, "_systemctl", self.systemctl):
            return service.setup(self.state, environment=self.environment, binaries=self.binaries, **options)

    def test_setup_publishes_validated_selection_and_private_credentials(self) -> None:
        result = self.setup()
        public = json.loads((self.state / "configuration.json").read_text())
        self.assertEqual(public["schema_version"], 1)
        self.assertEqual(public["streams"], service.DEFAULT_STREAMS)
        self.assertEqual(public["otlp_http_endpoint"], "http://127.0.0.1:4318")
        self.assertEqual(result["configuration_id"], public["configuration_id"])
        self.assertEqual(result["endpoints"]["collector_health"], "http://127.0.0.1:13133/")
        self.assertEqual(result["endpoints"]["collector_metrics"], "http://127.0.0.1:8888/metrics")
        self.assertNotIn("Authorization", json.dumps(result))
        self.assertEqual((self.state / "service.env").stat().st_mode & 0o777, 0o600)
        base, headers = service.backend_access(self.state)
        self.assertEqual(base, public["backend_http_endpoint"])
        self.assertTrue(headers["Authorization"].startswith("Basic "))
        self.assertTrue(all(call[0] == "show" for call in self.calls))
        config = service._configuration(self.state)
        generation = Path(str(config["generation"]))
        pipeline = json.loads((generation / "collector.yaml").read_text())
        for signal in service.DEFAULT_STREAMS:
            exporter = pipeline["exporters"][f"otlp_http/{signal}"]
            self.assertEqual(exporter["sending_queue"]["storage"], "file_storage")
            self.assertEqual(exporter["sending_queue"]["queue_size"], 1000)
            self.assertEqual(exporter["retry_on_failure"]["max_elapsed_time"], "300s")
            self.assertEqual(exporter["headers"]["Authorization"], "${env:PSE_TELEMETRY_AUTHORIZATION}")
        self.assertIn('ZO_COMPACT_DATA_RETENTION_DAYS="7"', (self.state / "service.env").read_text())

    def test_rejected_collector_config_never_activates_producer_selection(self) -> None:
        self.binary.write_text("#!/bin/sh\nexit 1\n")
        with self.assertRaisesRegex(service.TelemetryServiceError, "Collector rejected"):
            self.setup()
        self.assertFalse((self.state / "configuration.json").exists())
        self.assertFalse((self.state / "service.json").exists())

    def test_inspection_is_passive_and_absent_selection_creates_nothing(self) -> None:
        with patch.object(service, "_systemctl", side_effect=AssertionError("unexpected systemd operation")):
            self.assertEqual(service.inspect(self.state)["status"], "unconfigured")
        self.assertFalse(self.state.exists())
        self.setup()
        self.calls.clear()
        with patch.object(service, "_systemctl", self.systemctl):
            service.inspect(self.state)
        self.assertEqual([call[0] for call in self.calls], ["show", "show"])

    def test_exact_installer_is_shared_and_never_called_by_setup(self) -> None:
        with patch.object(service.diagnostic_trace, "install", return_value=self.binary) as install:
            self.assertEqual(set(service.install(self.environment)), set(service.TOOLS))
        self.assertEqual([call.args[0] for call in install.call_args_list], ["otelcol-contrib", "openobserve"])
        with patch.object(service.diagnostic_trace, "install", side_effect=AssertionError("implicit install")):
            self.setup()

    def test_units_bind_persistent_paths_and_native_supervision(self) -> None:
        self.setup()
        config = service._configuration(self.state)
        self.assertEqual(config["state"], str(self.state))
        for binary in config["binaries"].values():
            self.assertTrue(Path(binary["path"]).is_absolute())
        for unit in config["units"].values():
            text = (Path(config["generation"]) / unit).read_text()
            self.assertNotIn("surreal", text)
            self.assertNotIn("MemoryMax", text)
            self.assertIn("Restart=on-failure", text)
            self.assertIn("WantedBy=default.target", text)
            self.assertIn(f"WorkingDirectory={self.state}\n", text)
            self.assertIn(f"EnvironmentFile={self.state / 'service.env'}\n", text)

    def test_changed_generation_or_public_selection_refuses_control(self) -> None:
        self.setup()
        public = self.state / "configuration.json"
        record = json.loads(public.read_text())
        record["backend_http_endpoint"] = "http://127.0.0.1:9999"
        public.write_text(json.dumps(record))
        with patch.object(service, "_systemctl", side_effect=AssertionError("control must refuse first")), self.assertRaisesRegex(service.TelemetryServiceError, "public configuration identity"):
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
        lines = dict(line.split("=", 1) for line in (self.state / "service.env").read_text().splitlines())
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
        with patch.object(service, "_systemctl", self.systemctl), self.assertRaisesRegex(service.TelemetryServiceError, "ownership differs"):
            service.stop(self.state)
        self.assertEqual(self.calls, [])

    def test_setup_preserves_nonempty_and_existing_state(self) -> None:
        self.setup()
        credentials = (self.state / "service.env").read_bytes()
        with self.assertRaisesRegex(service.TelemetryServiceError, "already configured"):
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
            with patch.object(service, "_systemctl", self.systemctl), self.assertRaisesRegex(service.TelemetryServiceError, "listener collision"):
                service.start(self.state, environment=self.environment)
        self.assertNotIn("start", [call[0] for call in self.calls])
        self.assertNotIn("enable", [call[0] for call in self.calls])

    def test_stop_refuses_changed_invocation_before_effect(self) -> None:
        self.setup()
        self.calls.clear()
        count = 0

        def changed(*arguments: str, **_options: object) -> subprocess.CompletedProcess[str]:
            nonlocal count
            self.calls.append(arguments)
            count += 1
            invocation = "a" * 32 if count <= 2 else "b" * 32
            return subprocess.CompletedProcess(arguments, 0, f"LoadState=loaded\nActiveState=active\nMainPID=123\nInvocationID={invocation}\n", "")

        with patch.object(service, "_systemctl", changed), patch.object(service, "_checked_process", return_value={"pid": 123}), self.assertRaisesRegex(service.TelemetryServiceError, "incarnation changed"):
            service.stop(self.state)
        self.assertNotIn("stop", [call[0] for call in self.calls])

    def test_current_process_checks_executable_cgroup_and_incarnation(self) -> None:
        unit = self.root / "fixture.service"
        unit.write_text("fixture unit")
        config = {
            "owner_id": "selected-owner",
            "generation": str(self.root),
            "units": {"collector": unit.name},
            "binaries": {"collector": {"path": str(Path(sys.executable).resolve()), "identity": service._file_identity(Path(sys.executable).resolve())}},
        }
        group = Path("/proc/self/cgroup").read_text().strip().split("::", 1)[1]
        observed = {
            "LoadState": "loaded", "ActiveState": "active", "MainPID": str(os.getpid()),
            "InvocationID": "a" * 32, "ControlGroup": group,
            "FragmentPath": str(unit), "Description": "PSE telemetry collector selected-owner",
        }
        process = service._checked_process(config, "collector", observed)
        self.assertEqual(process["pid"], os.getpid())
        self.assertGreater(process["start_ticks"], 0)
        for key, wrong in (("Description", "foreign owner"), ("InvocationID", ""), ("ControlGroup", "/foreign")):
            with self.subTest(key=key), self.assertRaises(service.TelemetryServiceError):
                service._checked_process(config, "collector", observed | {key: wrong})

    def test_unit_paths_escape_specifiers_and_execution_variables(self) -> None:
        self.assertEqual(service._unit_quote('/tmp/a $b %c "d"'), '"/tmp/a $b %%c \\"d\\""')
        self.assertIn("$$b", service._unit_quote("/tmp/$b", execution=True))
        with self.assertRaisesRegex(service.TelemetryServiceError, "Invalid service path"):
            service._unit_quote("/tmp/bad\npath")


if __name__ == "__main__":
    unittest.main()
