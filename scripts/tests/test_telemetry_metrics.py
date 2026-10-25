# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent host populations, explicit SDK policy and cheap snapshots."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import io
import os
import sys
import tempfile
import time
import unittest
from dataclasses import asdict
from pathlib import Path
from unittest.mock import patch

from opentelemetry.metrics import CallbackOptions
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import AggregationTemporality, InMemoryMetricReader

from scripts import build_observation, pressure_observation, telemetry_metrics, validation
from scripts.validation_scope import Gate


class HostMetricTests(unittest.TestCase):
    def setUp(self) -> None:
        self.reader = InMemoryMetricReader(preferred_temporality=telemetry_metrics.cumulative_temporality())
        self.provider = MeterProvider(metric_readers=[self.reader], views=telemetry_metrics.sdk_views())
        self.addCleanup(self.provider.shutdown)
        self.host = telemetry_metrics.HostMetrics(self.provider.get_meter(telemetry_metrics.METER_NAME))

    def collect(self) -> dict:
        result = self.reader.get_metrics_data()
        if result is None:
            return {}
        return {metric.name: metric.data for resource in result.resource_metrics for scope in resource.scope_metrics for metric in scope.metrics}

    def test_cumulative_counts_histogram_boundaries_and_producer_cardinality(self) -> None:
        for index in range(100):
            self.host.build_record("compiler-artifact", index % 2 == 0)
            self.host.build_stage("arbitrary-id-" + str(index))
        self.host.build_record("not-a-cargo-record")
        # Test filtering itself as well as the producer vocabulary. The discarded
        # value must not create separate SDK aggregations.
        for index in range(30):
            self.host.build_records.add(1, {"record": "compiler-message", "invocation": str(index)})
        for duration in (0.001, 0.0011, 0.01, 1801.0):
            self.host.build_started()
            self.host.build_finished(duration, 0)
        first = self.collect()
        second = self.collect()
        self.assertEqual(first["pse.build.records"].aggregation_temporality, AggregationTemporality.CUMULATIVE)
        self.assertEqual(sum(p.value for p in first["pse.build.records"].data_points), 130)
        self.assertEqual(sum(p.value for p in second["pse.build.records"].data_points), 130)
        self.assertEqual(len(first["pse.build.records"].data_points), 2)
        self.assertEqual({k for p in first["pse.build.records"].data_points for k in p.attributes}, {"record"})
        self.assertEqual({p.attributes["freshness"]: p.value for p in first["pse.build.artifacts"].data_points}, {"fresh": 50, "nonfresh": 50})
        stages = first["pse.build.stage.observations"].data_points
        self.assertEqual([(dict(p.attributes), p.value) for p in stages], [({"stage": "unknown"}, 100)])
        self.assertEqual(first["pse.build.active"].data_points[0].value, 0)
        histogram = first["pse.build.duration"].data_points[0]
        self.assertEqual(histogram.explicit_bounds, telemetry_metrics.DURATION_BUCKETS)
        self.assertEqual(histogram.count, 4)
        self.assertAlmostEqual(histogram.sum, 1801.0121)
        self.assertEqual(histogram.bucket_counts, (1, 2, 0, 0, 0, 0, 0, 0, 1))

    def test_ambient_delta_does_not_override_selected_export_policy(self) -> None:
        from opentelemetry.exporter.otlp.proto.http.metric_exporter import OTLPMetricExporter  # noqa: PLC0415

        with patch.dict(os.environ, {"OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE": "DELTA"}):
            exporter = OTLPMetricExporter(preferred_temporality=telemetry_metrics.cumulative_temporality())
        self.addCleanup(exporter.shutdown)
        self.assertTrue(all(value == AggregationTemporality.CUMULATIVE for value in exporter._preferred_temporality.values()))

    def test_metrics_survive_unsampled_trace_context(self) -> None:
        from opentelemetry.sdk.trace import TracerProvider  # noqa: PLC0415
        from opentelemetry.sdk.trace.sampling import ALWAYS_OFF  # noqa: PLC0415

        provider = TracerProvider(sampler=ALWAYS_OFF)
        self.addCleanup(provider.shutdown)
        with provider.get_tracer("host-test").start_as_current_span("unsampled") as span:
            self.assertFalse(span.is_recording())
            self.host.build_record("build-finished")
        self.assertEqual(self.collect()["pse.build.records"].data_points[0].value, 1)

    def test_real_failed_child_build_and_assessment_populations(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.object(telemetry_metrics, "host_metrics", return_value=self.host), contextlib.redirect_stdout(io.StringIO()):
            root = Path(directory)
            result = validation.execute(root, root, "check", [sys.executable, "-c", "raise SystemExit(3)"], dict(os.environ))
            self.assertEqual(result["status"], "failed")
            self.assertTrue((root / "check.log").is_file())
            result.update(evidence_kind="executed", invocation=asdict(Gate("check", input_scope="rust-product")), mode="native-force-validate", phase="functional", selected=[{"name": name} for name in ("a", "b", "c", "d")], results=[{"status": s} for s in ("passed", "failure", "not_run", "skipped")])
            reused = {**result, "gate": "old-pass", "status": "passed", "evidence_kind": "unchanged-input-reuse", "origin": "/exact/authentic/origin"}
            receipt = {"scope": [asdict(Gate(name)) for name in ("check", "old-pass", "never-attempted")], "checks": [result, reused]}
            self.host.assessment_finished(receipt)
            command = [sys.executable, "-c", 'import json; print(json.dumps({"reason":"compiler-artifact","fresh":True})); print(json.dumps({"reason":"build-finished","success":False})); raise SystemExit(5)']
            self.assertEqual(build_observation.run_command(command, env={**os.environ, "XDG_STATE_HOME": directory}), 5)
            records = build_observation.observations(root / "pse-arrow/host-admission")
            self.assertEqual(records[0]["fresh_artifacts"], 1)
            self.assertEqual(records[0]["cache"]["hits"], "unknown")
        data = self.collect()
        populations = {}
        for point in data["pse.assessment.gates"].data_points:
            key = point.attributes["population"]
            populations[key] = populations.get(key, 0) + point.value
        self.assertEqual(populations, {"declared": 3, "executed": 1, "outcome": 1, "reused": 1, "unattempted": 1})
        cases = {(p.attributes["population"], p.attributes["status"]): p.value for p in data["pse.assessment.cases"].data_points}
        self.assertEqual(cases[("selected", "unknown")], 4)
        self.assertEqual(sum(v for (population, _), v in cases.items() if population == "executed"), 2)
        self.assertEqual(cases[("outcome", "not_run")], 1)
        self.assertEqual(data["pse.assessment.duration"].data_points[0].count, 1)
        self.assertEqual(data["pse.build.duration"].data_points[0].attributes["failure_class"], "compile")
        self.assertEqual(data["pse.engineering.stages"].data_points[0].attributes["failure_class"], "compile")

    def test_interruption_setup_and_teardown_are_distinct_stage_facts(self) -> None:
        with self.host.stage("setup"):
            pass
        with self.assertRaises(KeyboardInterrupt), self.host.stage("teardown"):
            raise KeyboardInterrupt
        points = self.collect()["pse.engineering.stages"].data_points
        self.assertEqual({(p.attributes["stage"], p.attributes["status"], p.attributes["failure_class"]) for p in points}, {("setup", "passed", "none"), ("teardown", "interrupted", "interruption")})

    def test_pressure_snapshot_omits_absent_expired_and_invalid_fields_without_io(self) -> None:
        now = time.monotonic_ns()
        identity = pressure_observation.Identity("boot-one", None, None, None)
        host = pressure_observation.Sample("partial", identity, now - 100, now, time.time_ns(), {"cpu.pressure": pressure_observation.Reading("available", {"some.avg10": 2.0, "some.avg60": float("nan"), "full.avg10": 99.0}), "io.pressure": pressure_observation.unavailable("unreadable")})
        missing = pressure_observation.Sample("unavailable", None, now, now, time.time_ns(), {})
        self.assertNotIn("pse.host.pressure", self.collect())
        self.host.publish_pressure(pressure_observation.Capture(host, missing))
        with patch.object(pressure_observation, "sample", side_effect=AssertionError("callback attempted IO")):
            first = self.collect()["pse.host.pressure"].data_points
            second = self.collect()["pse.host.pressure"].data_points
        self.assertEqual([(dict(p.attributes), p.value) for p in first], [({"scope": "host", "resource": "cpu", "stall": "some", "window": "avg10"}, 2.0)])
        self.assertEqual(first[0].value, second[0].value)
        with patch.object(telemetry_metrics.time, "monotonic_ns", return_value=now + 31_000_000_000):
            self.assertNotIn("pse.host.pressure", self.collect())
        restarted = pressure_observation.Sample("observed", pressure_observation.Identity("boot-two", None, None, None), now, now, time.time_ns(), {"cpu.pressure": pressure_observation.Reading("available", {"some.avg10": 7.0})})
        self.host.publish_pressure(pressure_observation.Capture(restarted, missing))
        self.assertEqual(self.collect()["pse.host.pressure"].data_points[0].value, 7.0)
        self.assertEqual(host.identity, identity)
        self.assertEqual(host.started_monotonic_ns, now - 100)

    def test_explicit_capture_publishes_real_source_without_changing_its_identity(self) -> None:
        with patch.object(telemetry_metrics, "host_metrics", return_value=self.host):
            captured = pressure_observation.capture(None)
        self.assertIsNone(captured.selected.identity)
        self.assertEqual(captured.selected.status, "unavailable")
        if captured.host.status == "unavailable":
            self.assertNotIn("pse.host.pressure", self.collect())
        else:
            self.assertIsNotNone(captured.host.identity)
            self.assertGreaterEqual(captured.host.finished_monotonic_ns, captured.host.started_monotonic_ns)
            observations = self.host.pressure_observations(CallbackOptions())
            self.assertTrue(all(item.attributes is not None and item.attributes["scope"] == "host" for item in observations))


if __name__ == "__main__":
    unittest.main()
