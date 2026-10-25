# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded host metric populations and process-owned Python SDK policy.

Exact identities, source intervals and uncertainty remain in the build, assessment
and pressure owners. Instruments aggregate owner facts, never trace-derived work.
Python SDK 1.45.1 Views filter keys but do not limit attribute cardinality: every
dimension below also has a finite producer vocabulary.
"""

from __future__ import annotations

import math
import os
import threading
import time
from contextlib import contextmanager
from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal

from opentelemetry import metrics
from opentelemetry.metrics import Observation

if TYPE_CHECKING:
    from collections.abc import Generator, Iterable

    from opentelemetry.metrics import CallbackOptions, Meter
    from opentelemetry.sdk.metrics.export import AggregationTemporality
    from opentelemetry.sdk.metrics.view import View

    from scripts.pressure_observation import Capture

METER_NAME = "pse.host"
DURATION_BUCKETS = (0.001, 0.01, 0.1, 1.0, 10.0, 60.0, 300.0, 1800.0)
PRESSURE_MAX_AGE_SECONDS = 30.0
STAGES = frozenset({"unknown", "cargo", "compiler", "linker", "build-script", "cache-client", "cargo-lock-wait", "build-finished", "discovery", "setup", "execution", "publication", "teardown", "functional", "performance"})
STATUSES = frozenset({"passed", "failed", "failure", "error", "skipped", "not_run", "blocked", "running", "interrupted", "cancelled", "findings", "unsupported", "unknown"})
MODES = frozenset({"python-setup-unit", "source-bound-production-capture", "python-native", "force-validate-feature-absence", "native-force-validate", "unknown"})
SUITES = frozenset({"rust-product", "python-product", "tooling", "generation", "documentation", "unknown"})
FAILURE_CLASSES = frozenset({"none", "interruption", "spawn", "evidence", "compile", "test", "command", "unknown"}) | STAGES


@dataclass(frozen=True)
class MetricDeclaration:
    name: str
    kind: Literal["counter", "up_down_counter", "histogram", "observable_gauge"]
    unit: str
    population: str
    attributes: tuple[str, ...] = ()
    buckets: tuple[float, ...] = ()
    temporality: str = "cumulative"
    reset: str = "process SDK lifetime"
    scope: str = "performing host owner"


METRIC_DECLARATIONS = (
    MetricDeclaration("pse.build.records", "counter", "{record}", "Recognized Cargo JSON records, once per actual child stream record", ("record",)),
    MetricDeclaration("pse.build.artifacts", "counter", "{artifact}", "Cargo compiler-artifact records with explicit fresh boolean; freshness is not compiler-cache hit/miss", ("freshness",)),
    MetricDeclaration("pse.build.stage.observations", "counter", "{observation}", "Changed observed Cargo stages; sampled proc observations are not a complete timeline", ("stage",)),
    MetricDeclaration("pse.build.active", "up_down_counter", "{build}", "Actual Cargo children observed by this host process, from spawn to settled exit"),
    MetricDeclaration("pse.build.duration", "histogram", "s", "Actual Cargo child lifetime observed by this host process", ("status", "failure_class"), DURATION_BUCKETS),
    MetricDeclaration("pse.assessment.gates", "counter", "{gate}", "Final authentic assessment gate populations: declared, executed, reused, outcome and unattempted remain distinct", ("population", "suite", "stage", "mode", "status", "failure_class")),
    MetricDeclaration("pse.assessment.cases", "counter", "{case}", "Selected cases and newly executed reported case lifecycles; runner errors can precede the test body, and selection never establishes execution", ("population", "suite", "mode", "status")),
    MetricDeclaration("pse.assessment.duration", "histogram", "s", "Newly executed assessment command durations, excluding reused evidence", ("suite", "stage", "mode", "status"), DURATION_BUCKETS),
    MetricDeclaration("pse.engineering.stages", "counter", "{stage}", "Completed performing-owner discovery, setup, execution, publication and teardown stages; repeated checkpoints are repeated publications", ("stage", "status", "failure_class")),
    MetricDeclaration("pse.engineering.stage.duration", "histogram", "s", "Performing-owner stage durations, including failed and interrupted stages", ("stage", "status", "failure_class"), DURATION_BUCKETS),
    MetricDeclaration("pse.host.pressure", "observable_gauge", "%", "Prepared Linux PSI rolling averages for the last explicit capture; unavailable and expired fields omitted", ("scope", "resource", "stall", "window"), temporality="instantaneous", reset="last explicit capture; expires after 30 seconds", scope="host or exact selected cgroup; original incarnation and intervals in pressure capture"),
)


def sdk_views() -> list[View]:
    """Filter before SDK aggregation; exact histograms use declared SI buckets."""
    from opentelemetry.sdk.metrics.view import (  # noqa: PLC0415
        ExplicitBucketHistogramAggregation,
        View,
    )

    return [
        View(
            meter_name=METER_NAME,
            instrument_name=d.name,
            attribute_keys=set(d.attributes),
            aggregation=ExplicitBucketHistogramAggregation(boundaries=d.buckets)
            if d.kind == "histogram" else None,
        )
        for d in METRIC_DECLARATIONS
    ]


def cumulative_temporality() -> dict[type, AggregationTemporality]:
    """Override ambient OTLP DELTA preferences for every supported instrument."""
    from opentelemetry.sdk.metrics import (  # noqa: PLC0415
        Counter, Histogram, ObservableCounter, ObservableGauge,
        ObservableUpDownCounter, UpDownCounter,
    )
    from opentelemetry.sdk.metrics.export import AggregationTemporality  # noqa: PLC0415

    return dict.fromkeys(
        (Counter, UpDownCounter, Histogram, ObservableCounter, ObservableUpDownCounter, ObservableGauge),
        AggregationTemporality.CUMULATIVE,
    )


def bounded(value: object, vocabulary: frozenset[str]) -> str:
    return value if isinstance(value, str) and value in vocabulary else "unknown"


def assessment_dimensions(check: dict) -> dict[str, str]:
    declaration = check.get("invocation", {})
    return {
        "suite": bounded(declaration.get("input_scope"), SUITES),
        "stage": bounded(check.get("phase"), STAGES),
        "mode": bounded(check.get("mode"), MODES),
        "status": bounded(check.get("status"), STATUSES),
    }


def failure_class(check: dict) -> str:
    """Classify only explicit receipt facts; no log-message inference."""
    if check.get("status") in {"interrupted", "cancelled"}:
        return "interruption"
    if check.get("spawn_error"):
        return "spawn"
    if check.get("report_errors"):
        return "evidence"
    if check.get("status") in {"failed", "failure", "error"}:
        # Functional scope alone does not identify a compile or test failure.
        recipe = check.get("invocation", {}).get("recipe") or check.get("gate")
        if recipe in {"check", "clippy-default", "clippy-no-default", "py-sync-native-correctness"}:
            return "compile"
        if recipe in {"test", "native-test", "native-python", "setup-test-report", "feature-absence"}:
            return "test"
        return "command"
    return "none"


class HostMetrics:
    """Direct instruments plus a single bounded prepared pressure selection."""

    def __init__(self, meter: Meter) -> None:
        self._pressure_lock = threading.Lock()
        self._pressure: tuple[tuple[int, Observation], ...] = ()
        declarations = {d.name: d for d in METRIC_DECLARATIONS}
        def counter(name: str) -> metrics.Counter:
            d = declarations[name]
            return meter.create_counter(name, unit=d.unit, description=d.population)
        def histogram(name: str) -> metrics.Histogram:
            d = declarations[name]
            return meter.create_histogram(name, unit=d.unit, description=d.population)
        self.build_records = counter("pse.build.records")
        self.build_artifacts = counter("pse.build.artifacts")
        self.build_stages = counter("pse.build.stage.observations")
        self.build_active = meter.create_up_down_counter("pse.build.active", unit="{build}", description=declarations["pse.build.active"].population)
        self.build_duration = histogram("pse.build.duration")
        self.assessment_gates = counter("pse.assessment.gates")
        self.assessment_cases = counter("pse.assessment.cases")
        self.assessment_duration = histogram("pse.assessment.duration")
        self.engineering_stages = counter("pse.engineering.stages")
        self.engineering_duration = histogram("pse.engineering.stage.duration")
        self.pressure = meter.create_observable_gauge(
            "pse.host.pressure", callbacks=[self.pressure_observations], unit="%",
            description=declarations["pse.host.pressure"].population,
        )

    def build_record(self, reason: str, fresh: object = None) -> None:
        if reason not in {"compiler-artifact", "compiler-message", "build-script-executed", "build-finished"}:
            return
        self.build_records.add(1, {"record": reason})
        if reason == "compiler-artifact" and type(fresh) is bool:
            self.build_artifacts.add(1, {"freshness": "fresh" if fresh else "nonfresh"})

    @contextmanager
    def stage(self, stage: str) -> Generator[dict[str, str]]:
        """The performing scope supplies command outcomes, never a sampled span."""
        started = time.monotonic()
        result = {"stage": bounded(stage, STAGES), "status": "passed", "failure_class": "none"}
        try:
            yield result
        except BaseException as error:
            result.update(status="interrupted" if isinstance(error, (KeyboardInterrupt, SystemExit)) else "failed", failure_class="interruption" if isinstance(error, (KeyboardInterrupt, SystemExit)) else bounded(stage, STAGES))
            raise
        finally:
            dimensions = {"stage": bounded(result.get("stage"), STAGES), "status": bounded(result.get("status"), STATUSES), "failure_class": bounded(result.get("failure_class"), FAILURE_CLASSES)}
            self.engineering_stages.add(1, dimensions)
            self.engineering_duration.record(max(0, time.monotonic() - started), dimensions)

    def build_stage(self, stage: str) -> None:
        self.build_stages.add(1, {"stage": bounded(stage, STAGES)})

    def build_started(self) -> None:
        self.build_active.add(1)

    def build_finished(self, seconds: float, code: int | None) -> None:
        self.build_active.add(-1)
        self.build_duration.record(
            max(0, seconds),
            {"status": "passed" if code == 0 else "interrupted" if code is None or code < 0 else "failed",
             "failure_class": "none" if code == 0 else "interruption" if code is None or code < 0 else "compile"},
        )

    def assessment_finished(self, receipt: dict) -> None:
        """Called once by the performing final-publication owner, never by readers.

        No deduplication ledger: replaying a receipt is explicitly not a producer.
        Failed evidence can still contain executed case outcomes; reused passes
        contribute only to the reused gate population.
        """
        checks = {check["gate"]: check for check in receipt["checks"]}
        for declaration in receipt["scope"]:
            check = checks.get(declaration["name"])
            facts = check or {"invocation": declaration, "status": "not_run", "phase": declaration.get("phase"), "mode": declaration.get("mode")}
            dims = {**assessment_dimensions(facts), "failure_class": failure_class(facts)}
            counter = self.assessment_gates
            counter.add(1, {**dims, "population": "declared"})
            case_dims = {k: dims[k] for k in ("suite", "mode")}
            cases = self.assessment_cases
            if check is not None and not check.get("origin") and check.get("evidence_kind") != "unchanged-input-reuse":
                selected = check.get("selected")
                if isinstance(selected, list) and selected:
                    cases.add(len(selected), {**case_dims, "population": "selected", "status": "unknown"})
            if check is None or check.get("evidence_kind") in {None, "not-run"}:
                counter.add(1, {**dims, "population": "unattempted"})
                continue
            executed = check.get("evidence_kind") == "executed"
            counter.add(1, {**dims, "population": "executed" if executed else "reused"})
            if not executed:
                continue
            counter.add(1, {**dims, "population": "outcome"})
            for result in check.get("results", []):
                status = bounded(result.get("status"), STATUSES)
                # An error is a reported attempted case lifecycle, potentially a
                # setup/teardown error; it does not prove the test body ran.
                cases.add(1, {**case_dims, "population": "outcome", "status": status})
                if status in {"passed", "failure", "error"}:
                    cases.add(1, {**case_dims, "population": "executed", "status": status})
            seconds = check.get("elapsed_seconds")
            if isinstance(seconds, (int, float)) and math.isfinite(seconds) and seconds >= 0:
                self.assessment_duration.record(seconds, {k: dims[k] for k in ("suite", "stage", "mode", "status")})

    def publish_pressure(self, captured: Capture) -> None:
        prepared = []
        for scope in ("host", "selected"):
            sample = getattr(captured, scope)
            if sample.identity is None or sample.status == "unavailable":
                continue
            for resource in ("cpu", "io", "memory"):
                reading = sample.fields.get(resource + ".pressure")
                if reading is None or reading.status != "available":
                    continue
                for stall in ("some", "full"):
                    if scope == "host" and resource == "cpu" and stall == "full":
                        continue
                    for window in ("avg10", "avg60", "avg300"):
                        value = reading.values.get(stall + "." + window)
                        if isinstance(value, (int, float)) and math.isfinite(value) and 0 <= value <= 100:
                            prepared.append((sample.finished_monotonic_ns, Observation(value, {"scope": scope, "resource": resource, "stall": stall, "window": window})))
        with self._pressure_lock:
            self._pressure = tuple(prepared)

    def pressure_observations(self, _options: CallbackOptions) -> Iterable[Observation]:
        # No filesystem, service work, scientific locks or snapshot preparation.
        with self._pressure_lock:
            snapshot = self._pressure
        now = time.monotonic_ns()
        return tuple(value for captured_at, value in snapshot if 0 <= now - captured_at <= PRESSURE_MAX_AGE_SECONDS * 1e9)


_LOCK = threading.Lock()
_HOST: tuple[int, HostMetrics] | None = None


def host_metrics() -> HostMetrics:
    global _HOST  # noqa: PLW0603 -- one process-local instrument set, no lifecycle ownership
    with _LOCK:
        if _HOST is None or _HOST[0] != os.getpid():
            _HOST = (os.getpid(), HostMetrics(metrics.get_meter(METER_NAME)))
        return _HOST[1]
