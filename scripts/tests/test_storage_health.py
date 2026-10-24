# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Original storage/host health, independent unavailable observations and inert actions."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import io
import json
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

import msgspec

from scripts import evidence_sources
from scripts import storage_health as health
from scripts import surreal_diagnostics as diagnostics
from scripts import surreal_server as server

BOOT = "a0a0a0a0-0000-4000-8000-000000000000"


def fixture(state: Path, *, available: bool = True) -> health.HealthSnapshot:
    """A supplied original-owner record, not a derived evidence projection."""
    identity = health.ServiceIdentity(
        str(state), "selected-instance", "a" * 32, 9000, "b" * 64
    )
    fields = {
        name: health.Metric("unavailable", [], "unregistered")
        for name in server.MX_METRIC_FIELDS
    }
    if available:
        for name, value in {
            "active_readers": 2,
            "oldest_pinned_version": 11,
            "async_queue_depth": 3,
            "async_queue_bytes": 4096,
            "poisoned": 1,
            "approx_data_bytes": 999,
        }.items():
            fields[name] = health.Metric(
                "approximate" if name == "approx_data_bytes" else "observed",
                [health.MetricSample('{namespace="shared"}', value)],
            )
    configuration = {
        name: health.unavailable("not observed") for name in health.CONFIGURATION_FIELDS
    }
    configuration["pending_maintenance"] = health.Observation("available", "none")
    return health.HealthSnapshot(
        health.SCHEMA,
        str(state),
        100,
        200,
        health.ProcessIdentity(BOOT, 30, "1000"),
        {name: health.Observation("available", 100) for name in health.HOST_FIELDS},
        health.Service("available", identity)
        if available
        else health.Service("unavailable", reason="service absent"),
        configuration,
        fields,
        110 if available else None,
        120 if available else None,
        health.CgroupObservation(
            "partial" if available else "unavailable",
            health.CgroupIdentity(BOOT, "/owned/service", 20, "a" * 32)
            if available
            else None,
            {name: health.unavailable("not observed") for name in health.CGROUP_FIELDS},
            [
                health.ProcessObservation(
                    health.ProcessIdentity(BOOT, 40, "2000"),
                    health.Observation("approximate", 200, "kernel VmRSS"),
                )
            ]
            if available
            else [],
        ),
        "partial",
        "observed" if available else "unavailable",
        "unknown",
        "selected-service-cgroup" if available else "unknown",
    )


class StorageHealthTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.state = self.root / "absent-service"

    def save(self, record: health.HealthSnapshot) -> Path:
        path = self.root / "health.json"
        path.write_bytes(msgspec.json.encode(record))
        return path

    def selected_service(self) -> server.SelectedService:
        return server.SelectedService(
            self.state,
            9000,
            {"instance_id": "selected-instance", "server": {"binary_sha256": "b" * 64}},
            "credential-digest",
            "proof-digest",
            "a" * 32,
            "fixture-selection-secret",
        )

    def test_representative_original_gauges_preserve_labels_precision_and_limits(
        self,
    ) -> None:
        record = health.read(self.save(fixture(self.state)))
        self.assertEqual(record.metrics["async_queue_depth"].samples[0].value, 3)
        self.assertEqual(record.metrics["oldest_pinned_version"].samples[0].value, 11)
        self.assertEqual(record.metrics["poisoned"].samples[0].value, 1)
        self.assertEqual(record.metrics["approx_data_bytes"].status, "approximate")
        self.assertEqual(record.metrics["live_keys"].status, "unavailable")
        self.assertEqual(
            record.metrics["active_readers"].samples[0].labels, '{namespace="shared"}'
        )
        presented = health.presentation(record)
        self.assertEqual(
            [a["action"] for a in presented["owner_actions"]], ["inspect-service"]
        )
        self.assertIn(
            "Zero async queue depth does not establish synchronous durability",
            " ".join(presented["limits"]),
        )
        self.assertEqual(record.scientific_outcome, "unknown")
        # A zero value is observed, while an absent pin remains unavailable.
        fields = health.metric_fields(
            {
                "families": server.selected_metric_values(
                    server.MX_METRIC_PREFIX + "async_queue_depth 0\n"
                )["families"]
            }
        )
        self.assertEqual(fields["async_queue_depth"].samples[0].value, 0)
        self.assertEqual(fields["oldest_pinned_version"].status, "unavailable")

    def test_unavailable_service_keeps_real_independent_host_observations_without_repairs(
        self,
    ) -> None:
        with (
            patch.object(
                server, "config_for", side_effect=server.SupervisorError("absent")
            ),
            patch.object(
                server, "selected_service", side_effect=server.SupervisorError("absent")
            ),
            patch.object(
                server,
                "metric_snapshot",
                side_effect=AssertionError("unverified metrics"),
            ),
            patch.object(server, "start", side_effect=AssertionError("hidden start")),
            patch.object(
                server, "recover_file_owners", side_effect=AssertionError("hidden GC")
            ),
            patch.object(
                server,
                "recover_maintenance",
                side_effect=AssertionError("hidden repair"),
            ),
            patch.object(
                server, "write_json", side_effect=AssertionError("hidden write")
            ),
        ):
            record = health.capture(self.state, time.monotonic() + 5)
        self.assertEqual(record.service.status, "unavailable")
        self.assertEqual(record.tool_outcome, "partial")
        self.assertEqual(record.service_outcome, "unavailable")
        self.assertEqual(record.scientific_outcome, "unknown")
        self.assertIsNotNone(record.observer)
        self.assertEqual(record.host["MemTotal_bytes"].status, "available")
        total = record.host["MemTotal_bytes"].value
        assert type(total) is int
        self.assertGreater(total, 0)
        self.assertEqual(record.host["filesystem_available_bytes"].status, "available")
        self.assertEqual(record.correspondence, "unknown")
        self.assertIsNotNone(record.host_pressure)
        self.assertEqual(record.host_pressure.identity.group, None)
        self.assertIn("cpu.pressure", record.host_pressure.fields)
        self.assertFalse(self.state.exists())
        self.assertEqual(health.read(self.save(record)), record)

    def test_pressure_serialization_refuses_different_service_cgroup(self) -> None:
        from scripts import pressure_observation as pressure

        record = fixture(self.state)
        record.cgroup.pressure = pressure.Sample(
            "partial",
            pressure.Identity(BOOT, "/owned/service", 1, 20),
            10,
            20,
            150,
            {
                "cpu.pressure": pressure.Reading(
                    "unavailable", reason="permission denied"
                )
            },
        )
        self.assertEqual(health.read(self.save(record)), record)
        record.cgroup.pressure.identity.inode = 21
        with self.assertRaisesRegex(ValueError, "pressure differs"):
            health.read(self.save(record))

    def test_failed_service_control_keeps_independent_host_observation(self) -> None:
        with (
            patch.object(
                server, "config_for", side_effect=server.SupervisorError("absent")
            ),
            patch.object(
                server,
                "selected_service",
                side_effect=subprocess.TimeoutExpired("systemctl", 1),
            ),
        ):
            record = health.capture(self.state, time.monotonic() + 5)
        self.assertEqual(record.service.status, "unavailable")
        self.assertEqual(record.host["MemTotal_bytes"].status, "available")
        self.assertEqual(record.correspondence, "unknown")
        self.assertFalse(self.state.exists())

    def original_metrics(self, identity: dict | None = None) -> dict:
        return {
            "schema": "selected-native-metrics-v1",
            "identity": identity
            or msgspec.to_builtins(fixture(self.state).service.identity),
            "started_unix_ns": time.time_ns(),
            "finished_unix_ns": time.time_ns(),
            **server.selected_metric_values(server.MX_METRIC_PREFIX + "poisoned 1\n"),
        }

    def test_capture_checks_actual_service_identity_and_nonfinite_observations(
        self,
    ) -> None:
        for variation in (
            "foreign_state",
            "foreign_instance",
            "foreign_invocation",
            "nonfinite",
            "missing_time",
        ):
            with self.subTest(variation=variation):

                def metrics(*_args: object, variation: str = variation) -> dict:
                    original = self.original_metrics()
                    if variation == "foreign_state":
                        original["identity"]["state"] = str(self.root / "other")
                    elif variation == "foreign_instance":
                        original["identity"]["instance_id"] = "other"
                    elif variation == "foreign_invocation":
                        original["identity"]["invocation"] = "c" * 32
                    elif variation == "nonfinite":
                        original["families"][server.MX_METRIC_PREFIX + "poisoned"][
                            "samples"
                        ][0]["value"] = float("nan")
                    else:
                        original.pop("started_unix_ns")
                    return original

                with (
                    patch.object(
                        server,
                        "config_for",
                        return_value={
                            "instance_id": "selected-instance",
                            "resources": {},
                            "accepting_writes": True,
                        },
                    ),
                    patch.object(
                        server,
                        "database_endpoint",
                        return_value="mem://configured?aol=sync&sync=every",
                    ),
                    patch.object(
                        server, "selected_service", return_value=self.selected_service()
                    ),
                    patch.object(server.SelectedService, "recheck"),
                    patch.object(server, "metric_snapshot", side_effect=metrics),
                    patch.object(
                        health,
                        "selected_cgroup",
                        return_value=fixture(self.state).cgroup,
                    ),
                    patch.object(
                        health,
                        "process_identity",
                        return_value=fixture(self.state).observer,
                    ),
                ):
                    record = health.capture(self.state, time.monotonic() + 5)
                self.assertEqual(record.service.status, "available")
                self.assertEqual(
                    record.service.identity, fixture(self.state).service.identity
                )
                self.assertTrue(
                    all(f.status == "unavailable" for f in record.metrics.values())
                )
                self.assertEqual(record.host["MemTotal_bytes"].status, "available")

    def test_available_capture_retains_configured_persistence_and_independent_outcomes(
        self,
    ) -> None:
        config = {
            "instance_id": "selected-instance",
            "resources": {},
            "accepting_writes": False,
        }
        with (
            patch.object(server, "config_for", return_value=config),
            patch.object(
                server,
                "database_endpoint",
                return_value="mem://configured?aol=sync&sync=every",
            ),
            patch.object(
                server, "selected_service", return_value=self.selected_service()
            ),
            patch.object(server.SelectedService, "recheck"),
            patch.object(
                server,
                "metric_snapshot",
                side_effect=lambda *_: self.original_metrics(),
            ),
            patch.object(
                health, "selected_cgroup", return_value=fixture(self.state).cgroup
            ),
            patch.object(
                health, "process_identity", return_value=fixture(self.state).observer
            ),
        ):
            record = health.capture(self.state, time.monotonic() + 5)
        self.assertEqual(record.service_outcome, "observed")
        self.assertEqual(record.metrics["poisoned"].samples[0].value, 1)
        self.assertEqual(
            record.configuration["configured_endpoint"].value,
            "mem://configured?aol=sync&sync=every",
        )
        self.assertEqual(record.scientific_outcome, "unknown")

    def test_oversized_actual_metric_response_retains_independent_verified_service(
        self,
    ) -> None:
        config = {
            "instance_id": "selected-instance",
            "resources": {},
            "accepting_writes": True,
        }
        response = MagicMock()
        response.status = 200
        received = 0

        def read1(maximum: int) -> bytes:
            nonlocal received
            received += maximum
            return b"x" * maximum

        response.read1.side_effect = read1
        connection = MagicMock()
        connection.getresponse.return_value = response
        with (
            patch.object(server, "config_for", return_value=config),
            patch.object(server, "database_endpoint", return_value="mem://configured"),
            patch.object(
                server, "selected_service", return_value=self.selected_service()
            ),
            patch.object(server.SelectedService, "recheck") as recheck,
            patch.object(server.http.client, "HTTPConnection", return_value=connection),
            patch.object(
                server,
                "selected_metric_values",
                side_effect=AssertionError("truncated prefix parsed"),
            ),
            patch.object(
                health, "selected_cgroup", return_value=fixture(self.state).cgroup
            ),
            patch.object(
                health, "process_identity", return_value=fixture(self.state).observer
            ),
        ):
            record = health.capture(self.state, time.monotonic() + 5)
        self.assertEqual(received, (4 << 20) + 1)
        connection.close.assert_called_once()
        recheck.assert_called_once()
        self.assertEqual(record.service.status, "available")
        self.assertEqual(record.service.identity, fixture(self.state).service.identity)
        self.assertEqual(record.service_outcome, "observed")
        self.assertEqual(record.cgroup, fixture(self.state).cgroup)
        self.assertEqual(record.correspondence, "selected-service-cgroup")
        self.assertEqual(record.tool_outcome, "partial")
        self.assertEqual(record.scientific_outcome, "unknown")
        self.assertIsNone(record.metric_started_unix_ns)
        self.assertIsNone(record.metric_finished_unix_ns)
        for field in record.metrics.values():
            self.assertEqual(field.status, "unavailable")
            self.assertEqual(field.samples, [])
            self.assertEqual(
                field.reason, "Selected native metrics exceeds its response bound"
            )
        self.assertEqual(health.read(self.save(record)), record)

    def test_failed_metric_capture_cannot_retain_changed_service_identity(self) -> None:
        config = {
            "instance_id": "selected-instance",
            "resources": {},
            "accepting_writes": True,
        }
        with (
            patch.object(server, "config_for", return_value=config),
            patch.object(server, "database_endpoint", return_value="mem://configured"),
            patch.object(
                server, "selected_service", return_value=self.selected_service()
            ),
            patch.object(
                server.SelectedService,
                "recheck",
                side_effect=server.SupervisorError("service changed"),
            ),
            patch.object(
                server,
                "metric_snapshot",
                side_effect=server.SupervisorError(
                    "Selected native metrics exceeds its response bound"
                ),
            ),
            patch.object(
                health,
                "selected_cgroup",
                side_effect=AssertionError("changed service joined"),
            ),
        ):
            record = health.capture(self.state, time.monotonic() + 5)
        self.assertEqual(record.service.status, "unavailable")
        self.assertIsNone(record.service.identity)
        self.assertEqual(record.service_outcome, "unavailable")
        self.assertEqual(record.correspondence, "unknown")
        self.assertTrue(
            all(field.status == "unavailable" for field in record.metrics.values())
        )
        self.assertEqual(record.host["MemTotal_bytes"].status, "available")

    def test_changed_cgroup_incarnation_retains_unknown_correspondence(self) -> None:
        identity = fixture(self.state).service.identity
        assert identity is not None
        with (
            patch.object(
                server,
                "storage_unit_observation",
                return_value={"InvocationID": "c" * 32, "ControlGroup": "/foreign"},
            ),
            patch.object(
                health.host_admission,
                "group_identity",
                side_effect=AssertionError("foreign cgroup read"),
            ),
        ):
            group = health.selected_cgroup(self.state, identity, time.monotonic() + 5)
        self.assertEqual(group.status, "unavailable")
        self.assertIsNone(group.identity)
        self.assertEqual(group.processes, [])

    def test_service_process_membership_rss_and_lifetime_are_original_kernel_observations(
        self,
    ) -> None:
        identity = fixture(self.state).service.identity
        assert identity is not None
        group = "/owned/service"
        files = {
            "/sys/fs/cgroup/owned/service/memory.current": "4096",
            "/sys/fs/cgroup/owned/service/memory.max": "max",
            "/sys/fs/cgroup/owned/service/memory.peak": "8192",
            "/sys/fs/cgroup/owned/service/memory.swap.current": "0",
            "/sys/fs/cgroup/owned/service/cgroup.procs": "40",
            "/proc/40/status": "Name: surreal\nVmRSS: 23 kB\n",
        }
        process = health.ProcessIdentity(BOOT, 40, "2000")
        for changed in (False, True):
            with self.subTest(changed=changed):
                with (
                    patch.object(
                        server,
                        "storage_unit_observation",
                        return_value={
                            "InvocationID": identity.invocation,
                            "ControlGroup": group,
                        },
                    ),
                    patch.object(
                        health.host_admission, "group_identity", return_value=20
                    ),
                    patch.object(health.host_admission, "boot", return_value=BOOT),
                    patch.object(
                        health.native_operation, "process_group", return_value=group
                    ),
                    patch.object(
                        health,
                        "process_identity",
                        side_effect=[
                            process,
                            health.ProcessIdentity(BOOT, 40, "2001")
                            if changed
                            else process,
                        ],
                    ),
                    patch.object(
                        Path,
                        "read_text",
                        autospec=True,
                        side_effect=lambda path: files[str(path)],
                    ),
                ):
                    observed = health.selected_cgroup(
                        self.state, identity, time.monotonic() + 5
                    )
                self.assertEqual(observed.fields["memory.current"].value, 4096)
                self.assertEqual(observed.fields["memory.max"].value, "max")
                if changed:
                    self.assertEqual(observed.status, "partial")
                    self.assertEqual(observed.processes, [])
                else:
                    self.assertEqual(observed.status, "observed")
                    self.assertEqual(observed.processes[0].identity, process)
                    self.assertEqual(observed.processes[0].rss_bytes.value, 23 * 1024)
                    self.assertEqual(
                        observed.processes[0].rss_bytes.status, "approximate"
                    )

    def test_offline_codec_refuses_foreign_inconsistent_nonfinite_and_unknown_records(
        self,
    ) -> None:
        original = msgspec.to_builtins(fixture(self.state))
        mutations = [
            lambda r: r.update(schema="other-v1"),
            lambda r: r["service"]["identity"].update(state="/foreign"),
            lambda r: r["cgroup"]["identity"].update(invocation="c" * 32),
            lambda r: r["metrics"]["approx_data_bytes"].update(status="observed"),
            lambda r: r["metrics"]["poisoned"]["samples"][0].update(value=float("inf")),
            lambda r: r["metrics"]["poisoned"]["samples"][0].update(value=True),
            lambda r: r["metrics"]["poisoned"].update(status="unavailable"),
            lambda r: r.update(scientific_outcome="pass"),
            lambda r: r.update(command=["rm", "-rf", "/"]),
            lambda r: r["host"].pop("MemTotal_bytes"),
        ]
        for mutate in mutations:
            value = json.loads(json.dumps(original))
            mutate(value)
            path = self.root / "invalid.json"
            path.write_text(json.dumps(value))
            with self.assertRaises((ValueError, msgspec.DecodeError)):
                health.read(path)

    def test_health_cli_offline_reads_only_supplied_snapshot_and_does_not_execute_suggestions(
        self,
    ) -> None:
        record = fixture(self.state)
        record.configuration["pending_maintenance"] = health.Observation(
            "available", "rebuild:disposing"
        )
        original = self.save(record)
        with (
            patch.object(
                server, "config_for", side_effect=AssertionError("live owner read")
            ),
            patch.object(
                server, "metric_snapshot", side_effect=AssertionError("network")
            ),
            patch.object(
                server,
                "recover_maintenance",
                side_effect=AssertionError("hidden repair"),
            ),
            patch.object(
                diagnostics.subprocess,
                "Popen",
                side_effect=AssertionError("shell launch"),
            ),
            contextlib.redirect_stdout(io.StringIO()) as output,
        ):
            self.assertEqual(
                diagnostics.main(["health", "--snapshot", str(original)]), 0
            )
        value = json.loads(output.getvalue())
        self.assertEqual(value["snapshot"]["schema"], health.SCHEMA)
        action = value["owner_actions"][1]
        self.assertEqual(action["action"], "recover-maintenance")
        self.assertEqual(
            action["command"],
            ["just", "surreal", "recover-maintenance", "--state", str(self.state)],
        )
        self.assertIn("exact effect settlement", action["effect"])
        self.assertFalse(self.state.exists())

    def test_stale_suggestion_enters_original_owner_and_cannot_replace_current_preconditions(
        self,
    ) -> None:
        record = fixture(self.state)
        record.configuration["pending_maintenance"] = health.Observation(
            "available", "rebuild:disposing"
        )
        action = health.owner_actions(record)[1]
        self.assertEqual(action["command"][2], "recover-maintenance")
        with (
            patch.object(
                server, "lifecycle_reservation", return_value=contextlib.nullcontext()
            ) as reserve,
            patch.object(
                server, "config_for", return_value={"surrealkit_rollout_pending": True}
            ) as current,
            patch.object(
                server,
                "maintenance_payloads",
                side_effect=AssertionError("stale action passed preconditions"),
            ),
            self.assertRaisesRegex(
                server.SupervisorError, "exact rollout owner and settlement"
            ),
        ):
            server.recover_maintenance(self.state, [])
        reserve.assert_called_once_with(self.state)
        current.assert_called_once_with(self.state)

    def test_evidence_adapter_uses_original_identity_links_without_live_service_or_science_inference(
        self,
    ) -> None:
        for available in (True, False):
            with self.subTest(available=available):
                record = fixture(self.state, available=available)
                source = self.save(record)
                with patch.object(
                    server, "metric_snapshot", side_effect=AssertionError("live lookup")
                ):
                    projection = evidence_sources.storage_health_snapshot(source)
                self.assertEqual(projection["codec"], health.SCHEMA)
                facts = {f["member"]: f for f in projection["facts"]}
                self.assertEqual(facts["science"]["status"], "unknown")
                self.assertEqual(
                    facts["service"]["status"],
                    "observed" if available else "unavailable",
                )
                self.assertEqual(
                    facts["metric:approx_data_bytes"]["status"],
                    "approximate" if available else "unavailable",
                )
                self.assertEqual(
                    facts["link:storage_health"]["link_target"],
                    facts["health"]["detail_ref"],
                )
                self.assertEqual(
                    facts["health"]["detail_ref"],
                    health.presentation(record)["reference"],
                )
                links = [
                    f["link_relation"]
                    for f in projection["facts"]
                    if f["kind"] == "declared_link"
                ]
                self.assertNotIn("invocation", links)
                self.assertEqual("service" in links, available)
                self.assertEqual("process" in links, available)
                self.assertFalse(self.state.exists())


if __name__ == "__main__":
    unittest.main()
