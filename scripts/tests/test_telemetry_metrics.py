# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent host populations, explicit SDK policy and cheap snapshots."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import copy
import io
import os
import sys
import tempfile
import time
import unittest
from dataclasses import asdict
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from opentelemetry.exporter.otlp.proto.http.metric_exporter import OTLPMetricExporter
from opentelemetry.metrics import CallbackOptions
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import (
    AggregationTemporality,
    InMemoryMetricReader,
    MetricExportResult,
    PeriodicExportingMetricReader,
    Sum,
)

from scripts import (
    pressure_observation,
    telemetry_metrics,
)
from scripts.tests.telemetry_operational_acceptance import validate_points, validate_sdk
from scripts.validation_scope import Gate

if TYPE_CHECKING:
    from opentelemetry.sdk.metrics.export import MetricsData

    from scripts.telemetry_investigation import QueryAnswer


class HostMetricTests(unittest.TestCase):
    def setUp(self) -> None:
        self.reader = InMemoryMetricReader(
            preferred_temporality=telemetry_metrics.cumulative_temporality()
        )
        self.provider = MeterProvider(
            metric_readers=[self.reader], views=telemetry_metrics.sdk_views()
        )
        self.addCleanup(self.provider.shutdown)
        self.host = telemetry_metrics.HostMetrics(
            self.provider.get_meter(telemetry_metrics.METER_NAME)
        )

    def collect(self) -> dict:
        result = self.reader.get_metrics_data()
        if result is None:
            return {}
        return {
            metric.name: metric.data
            for resource in result.resource_metrics
            for scope in resource.scope_metrics
            for metric in scope.metrics
        }

    def test_ambient_delta_does_not_override_selected_export_policy(self) -> None:
        captured: list[MetricsData] = []

        class CaptureExporter(OTLPMetricExporter):
            def export(
                self,
                metrics_data: MetricsData,
                timeout_millis: float | None = 10000,
                **kwargs: object,
            ) -> MetricExportResult:
                del timeout_millis, kwargs
                captured.append(metrics_data)
                return MetricExportResult.SUCCESS

        with patch.dict(
            os.environ, {"OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE": "DELTA"}
        ):
            exporter = CaptureExporter(
                preferred_temporality=telemetry_metrics.cumulative_temporality()
            )
            reader = PeriodicExportingMetricReader(
                exporter, export_interval_millis=3_600_000
            )
            provider = MeterProvider(metric_readers=[reader], shutdown_on_exit=False)
        self.addCleanup(provider.shutdown)
        counter = provider.get_meter("selected-temporality").create_counter(
            "fixture.events"
        )
        counter.add(3)
        self.assertTrue(provider.force_flush())
        self.assertTrue(provider.force_flush())
        totals = [
            metric.data
            for capture in captured
            for resource in capture.resource_metrics
            for scope in resource.scope_metrics
            for metric in scope.metrics
            if metric.name == "fixture.events"
        ]
        self.assertEqual(len(totals), 2)
        for total in totals:
            assert isinstance(total, Sum)
            self.assertEqual(
                total.aggregation_temporality, AggregationTemporality.CUMULATIVE
            )
            self.assertEqual(total.data_points[0].value, 3)

    def test_metrics_survive_unsampled_trace_context(self) -> None:
        from opentelemetry.sdk.trace import TracerProvider  # noqa: PLC0415
        from opentelemetry.sdk.trace.sampling import ALWAYS_OFF  # noqa: PLC0415

        provider = TracerProvider(sampler=ALWAYS_OFF)
        self.addCleanup(provider.shutdown)
        with provider.get_tracer("host-test").start_as_current_span(
            "unsampled"
        ) as span:
            self.assertFalse(span.is_recording())
            self.host.engineering_stages.add(
                1, {"stage": "execution", "status": "passed", "failure_class": "none"}
            )
        self.assertEqual(
            self.collect()["pse.engineering.stages"].data_points[0].value, 1
        )

    def test_real_failed_child_and_assessment_populations(self) -> None:
        from scripts import (
            validation,
        )

        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(telemetry_metrics, "host_metrics", return_value=self.host),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            root = Path(directory)
            result = validation.execute(
                root,
                root,
                "check",
                [sys.executable, "-c", "raise SystemExit(3)"],
                dict(os.environ),
            )
            self.assertEqual(result["status"], "failed")
            self.assertTrue((root / "check.log").is_file())
            result.update(
                evidence_kind="executed",
                invocation=asdict(Gate("check", input_scope="rust-product")),
                mode="native-force-validate",
                phase="functional",
                selected=[{"name": name} for name in ("a", "b", "c", "d")],
                results=[
                    {"status": s} for s in ("passed", "failure", "not_run", "skipped")
                ],
            )
            reused = {
                **result,
                "gate": "old-pass",
                "status": "passed",
                "evidence_kind": "unchanged-input-reuse",
                "origin": "/exact/authentic/origin",
            }
            receipt = {
                "scope": [
                    asdict(Gate(name))
                    for name in ("check", "old-pass", "never-attempted")
                ],
                "checks": [result, reused],
            }
            self.host.assessment_finished(receipt)
        data = self.collect()
        populations = {}
        for point in data["pse.assessment.gates"].data_points:
            key = point.attributes["population"]
            populations[key] = populations.get(key, 0) + point.value
        self.assertEqual(
            populations,
            {"declared": 3, "executed": 1, "outcome": 1, "reused": 1, "unattempted": 1},
        )
        cases = {
            (p.attributes["population"], p.attributes["status"]): p.value
            for p in data["pse.assessment.cases"].data_points
        }
        self.assertEqual(cases[("selected", "unknown")], 4)
        self.assertEqual(
            sum(v for (population, _), v in cases.items() if population == "executed"),
            2,
        )
        self.assertEqual(cases[("outcome", "not_run")], 1)
        self.assertEqual(data["pse.assessment.duration"].data_points[0].count, 1)
        self.assertEqual(
            data["pse.engineering.stages"].data_points[0].attributes["failure_class"],
            "compile",
        )

    def test_interruption_setup_and_teardown_are_distinct_stage_facts(self) -> None:
        with self.host.stage("setup"):
            pass
        with self.assertRaises(KeyboardInterrupt), self.host.stage("teardown"):
            raise KeyboardInterrupt
        points = self.collect()["pse.engineering.stages"].data_points
        self.assertEqual(
            {
                (
                    p.attributes["stage"],
                    p.attributes["status"],
                    p.attributes["failure_class"],
                )
                for p in points
            },
            {("setup", "passed", "none"), ("teardown", "interrupted", "interruption")},
        )

    def test_pressure_snapshot_omits_absent_expired_and_invalid_fields_without_io(
        self,
    ) -> None:
        now = time.monotonic_ns()
        identity = pressure_observation.Identity("boot-one", None, None, None)
        host = pressure_observation.Sample(
            "partial",
            identity,
            now - 100,
            now,
            time.time_ns(),
            {
                "cpu.pressure": pressure_observation.Reading(
                    "available",
                    {"some.avg10": 2.0, "some.avg60": float("nan"), "full.avg10": 99.0},
                ),
                "io.pressure": pressure_observation.unavailable("unreadable"),
            },
        )
        missing = pressure_observation.Sample(
            "unavailable", None, now, now, time.time_ns(), {}
        )
        self.assertNotIn("pse.host.pressure", self.collect())
        self.host.publish_pressure(pressure_observation.Capture(host, missing))
        with patch.object(
            pressure_observation,
            "sample",
            side_effect=AssertionError("callback attempted IO"),
        ):
            first = self.collect()["pse.host.pressure"].data_points
            second = self.collect()["pse.host.pressure"].data_points
        self.assertEqual(
            [(dict(p.attributes), p.value) for p in first],
            [
                (
                    {
                        "scope": "host",
                        "resource": "cpu",
                        "stall": "some",
                        "window": "avg10",
                    },
                    2.0,
                )
            ],
        )
        self.assertEqual(first[0].value, second[0].value)
        with patch.object(
            telemetry_metrics.time, "monotonic_ns", return_value=now + 31_000_000_000
        ):
            self.assertNotIn("pse.host.pressure", self.collect())
        restarted = pressure_observation.Sample(
            "observed",
            pressure_observation.Identity("boot-two", None, None, None),
            now,
            now,
            time.time_ns(),
            {
                "cpu.pressure": pressure_observation.Reading(
                    "available", {"some.avg10": 7.0}
                )
            },
        )
        self.host.publish_pressure(pressure_observation.Capture(restarted, missing))
        self.assertEqual(self.collect()["pse.host.pressure"].data_points[0].value, 7.0)
        self.assertEqual(host.identity, identity)
        self.assertEqual(host.started_monotonic_ns, now - 100)

    def test_explicit_capture_publishes_real_source_without_changing_its_identity(
        self,
    ) -> None:
        with patch.object(telemetry_metrics, "host_metrics", return_value=self.host):
            captured = pressure_observation.capture(None)
        self.assertIsNone(captured.selected.identity)
        self.assertEqual(captured.selected.status, "unavailable")
        if captured.host.status == "unavailable":
            self.assertNotIn("pse.host.pressure", self.collect())
        else:
            self.assertIsNotNone(captured.host.identity)
            self.assertGreaterEqual(
                captured.host.finished_monotonic_ns, captured.host.started_monotonic_ns
            )
            observations = self.host.pressure_observations(CallbackOptions())
            self.assertTrue(
                all(
                    item.attributes is not None and item.attributes["scope"] == "host"
                    for item in observations
                )
            )


class OperationalAcceptanceOracleTests(unittest.TestCase):
    """The manual driver's oracle rejects plausible wrong transport populations."""

    def fixture(self) -> dict:
        common = {"start_time_unix_nano": 1_000_000, "time_unix_nano": 2_000_000}
        return {
            "sampler": "always_off",
            "service_instance_id": "fixture-one",
            "configuration_id": "configuration-one",
            "spans": [{"span_id": str(index), "sampled": False} for index in range(3)],
            "exported_span_ids": [],
            "metric_exports": [
                {
                    "name": "pse.engineering.stages",
                    "data": {
                        "aggregation_temporality": 2,
                        "data_points": [
                            {
                                **common,
                                "value": 3,
                                "attributes": {
                                    "stage": "execution",
                                    "status": "passed",
                                    "failure_class": "none",
                                },
                            }
                        ],
                    },
                },
                {
                    "name": "pse.engineering.stage.duration",
                    "data": {
                        "aggregation_temporality": 2,
                        "data_points": [
                            {
                                **common,
                                "count": 3,
                                "sum": 0.0211,
                                "attributes": {
                                    "status": "passed",
                                    "failure_class": "none",
                                },
                                "explicit_bounds": [
                                    0.001,
                                    0.01,
                                    0.1,
                                    1.0,
                                    10.0,
                                    60.0,
                                    300.0,
                                    1800.0,
                                ],
                                "bucket_counts": [1, 1, 1, 0, 0, 0, 0, 0, 0],
                            }
                        ],
                    },
                },
            ],
        }

    def answer(
        self, rows: list[dict], stream: str = "pse_engineering_stages"
    ) -> QueryAnswer:
        from scripts.telemetry_investigation import QueryAnswer  # noqa: PLC0415

        return QueryAnswer.model_validate(
            {
                "request": {
                    "signal": "metrics",
                    "metric": stream,
                    "service_instance_id": "fixture-one",
                    "start_time_us": 0,
                    "end_time_us": 3000,
                },
                "status": "available",
                "source": {
                    "configuration_id": "configuration-one",
                    "backend_version": "1.1.2",
                    "backend_process": {"pid": 1, "start": "fixture"},
                    "stream": stream,
                    "signal": "metrics",
                    "start_time_us": 0,
                    "end_time_us": 3000,
                    "retrieved_at_ns": 3_000_000,
                },
                "records": rows,
            }
        )

    def row(self) -> dict:
        return {
            "service_instance_id": "fixture-one",
            "start_time": "1000000",
            "_timestamp": 2000,
            "aggregation_temporality": "AGGREGATION_TEMPORALITY_CUMULATIVE",
            "value": 3,
            "stage": "execution",
            "status": "passed",
            "failure_class": "none",
        }

    def test_literal_sdk_and_backend_oracles_reject_changed_totals_and_reset(
        self,
    ) -> None:
        produced = self.fixture()
        self.assertEqual(
            validate_sdk(produced),
            {
                "pse.engineering.stages": 1_000_000,
                "pse.engineering.stage.duration": 1_000_000,
            },
        )
        validate_points(
            self.answer([self.row()]),
            produced,
            "pse_engineering_stages",
            exact_timestamps=True,
        )
        for changes in (
            {"value": 6},
            {"start_time": "999999"},
            {"_timestamp": 2001},
            {"fixture_invocation_id": "forbidden-id"},
        ):
            with self.subTest(changes=changes), self.assertRaises(RuntimeError):
                validate_points(
                    self.answer([{**self.row(), **changes}]),
                    produced,
                    "pse_engineering_stages",
                    exact_timestamps=True,
                )
        incorrect = copy.deepcopy(produced)
        incorrect["metric_exports"][1]["data"]["data_points"][0]["bucket_counts"] = [
            0,
            2,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
        ]
        with self.assertRaises(RuntimeError):
            validate_sdk(incorrect)

    def test_backend_oracle_requires_all_cumulative_buckets(self) -> None:
        # The backend stores cumulative buckets; SDK buckets are disjoint.
        rows = [
            {**self.row(), "le": label, "value": value}
            for label, value in [
                ("0.001", 1),
                ("0.01", 2),
                ("0.1", 3),
                ("1", 3),
                ("10", 3),
                ("60", 3),
                ("300", 3),
                ("1800", 3),
                ("inf", 3),
            ]
        ]
        validate_points(
            self.answer(rows, "pse_engineering_stage_duration_bucket"),
            self.fixture(),
            "pse_engineering_stage_duration_bucket",
            exact_timestamps=True,
        )
        with self.assertRaises(RuntimeError):
            validate_points(
                self.answer(rows[:-1], "pse_engineering_stage_duration_bucket"),
                self.fixture(),
                "pse_engineering_stage_duration_bucket",
            )


if __name__ == "__main__":
    unittest.main()
