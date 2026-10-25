# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Host telemetry observations; no scientific or native checkpoint schema copies."""

from __future__ import annotations

from typing import Literal

from pydantic import Field

from scripts.capabilities.read_contracts import (
    AnswerState,
    Contract,
    DetailRefused,
    OwnerRecord,
    ReadAnswer,
    finish_read,
)
from scripts.telemetry_investigation import QueryAnswer
from scripts.telemetry_selection import SelectionPage


class SignalSettings(Contract):
    enabled: bool
    endpoint: str | None
    header_names: list[str]
    timeout_ms: int


class Signals(Contract):
    traces: SignalSettings
    logs: SignalSettings
    metrics: SignalSettings


class ConfigurationPayload(OwnerRecord):
    schema_version: int
    disabled: bool
    selection: str
    signals: Signals
    transport: str
    resource: dict[str, str]
    sdk_versions: dict[str, str | None]
    lifecycle_owner: str
    sampler: str
    sampler_argument: float
    max_queue_size: int
    max_export_batch_size: int
    configuration_id: str | None
    delivery: str


class LocalProducer(OwnerRecord):
    status: str
    lifecycle_owner: str
    configuration: ConfigurationPayload | None = None
    export_attempts: OwnerRecord
    delivery: str
    queue_occupancy: str
    dropped_records: str


class ServiceObservation(OwnerRecord):
    status: str
    state: str | None = None
    reason: str | None = None
    configuration: OwnerRecord | None = None
    services: dict[str, OwnerRecord] = Field(default_factory=dict)


class CollectorSample(Contract):
    name: str
    labels: dict[str, str]
    value: float


class PipelineStage(OwnerRecord):
    status: str
    observed_at_ns: int | None = None
    reason: str | None = None
    http_status: int | None = None
    source: str | None = None
    samples: list[CollectorSample] | None = None
    configuration_id: str | None = None
    process: OwnerRecord | None = None
    query_visibility: str | None = None


class HealthPayload(Contract):
    observed_at_ns: int
    service: ServiceObservation
    producer: LocalProducer
    delivery: OwnerRecord
    source_coverage: str
    pipeline: PipelineStage | dict[str, PipelineStage]


class TraceSource(OwnerRecord):
    configuration_id: str | None
    backend: Literal["openobserve"]
    backend_version: str | None
    signal: Literal["traces"]
    stream: str
    start_time_us: int
    end_time_us: int
    retrieved_at_ns: int
    retention_days: int | None
    source_cut: str
    backend_process: OwnerRecord | None = None


class TraceRecord(Contract):
    trace_id: str | None
    span_id: str | None
    parent_span_id: str | None
    parent_trace_id: str | None
    service_instance_id: str | None
    operation_name: str | None
    timestamp_us: int | None


class TracePayload(Contract):
    status: Literal["unavailable", "stale", "partial", "available", "empty"]
    reason: str | None = None
    http_status: int | None = None
    source: TraceSource | None = None
    records: list[TraceRecord]
    complete: Literal[False]
    returned_rows: int | None = None
    row_limit: int | None = None
    coverage: str | None = None
    population_total: None = None
    missing_record_meaning: str | None = None


class TelemetryHealthResult(ReadAnswer):
    operation: Literal["telemetry_health"] = "telemetry_health"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: HealthPayload


class TelemetryConfigurationResult(ReadAnswer):
    operation: Literal["telemetry_configuration"] = "telemetry_configuration"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: ConfigurationPayload


class TelemetryTraceResult(ReadAnswer):
    operation: Literal["telemetry_trace_lookup"] = "telemetry_trace_lookup"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: TracePayload


class TelemetryQueryResult(ReadAnswer):
    operation: Literal["telemetry_query"] = "telemetry_query"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: QueryAnswer


def query(value: QueryAnswer) -> TelemetryQueryResult | DetailRefused:
    return finish_read(
        TelemetryQueryResult,
        value,
        population="unavailable"
        if value.status in {"unavailable", "stale", "unsupported"}
        else "partial",
        reasons=[value.reason or "bounded_live_selection", *value.limits],
    )


class TelemetrySelectionResult(ReadAnswer):
    operation: Literal["telemetry_selection_read"] = "telemetry_selection_read"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: SelectionPage


def selection(value: SelectionPage) -> TelemetrySelectionResult | DetailRefused:
    return finish_read(
        TelemetrySelectionResult,
        value,
        population="partial",
        reasons=[value.interpretation, *value.missing_context],
    )


class WatchPayload(Contract):
    status: str
    reason: str | None = None
    rules: list[OwnerRecord]
    records: list[OwnerRecord]
    history: list[OwnerRecord] = Field(default_factory=list)
    complete: Literal[False]
    source: OwnerRecord | None = None
    delivery_stream: str | None = None
    automatic_intervention: Literal[False] = False


class TelemetryWatchResult(ReadAnswer):
    operation: Literal["telemetry_watch_read"] = "telemetry_watch_read"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: WatchPayload


def watch(value: object) -> TelemetryWatchResult | DetailRefused:
    selected = WatchPayload.model_validate(value)
    return finish_read(
        TelemetryWatchResult,
        selected,
        population="unavailable" if selected.status == "unavailable" else "partial",
        reasons=[
            "Backend-native schedule/history and local delivery; no automatic intervention. Quiet or missing history does not establish health."
        ],
    )


def health(value: object) -> TelemetryHealthResult | DetailRefused:
    selected = HealthPayload.model_validate(value)
    return finish_read(
        TelemetryHealthResult,
        selected,
        population="partial",
        reasons=[
            selected.source_coverage,
            "Producer state belongs to this calling MCP process, not an observed native target. Native target observations remain at native_diagnostic_inspect; native results and settlement retain their original authority.",
        ],
    )


def configuration(value: object) -> TelemetryConfigurationResult | DetailRefused:
    return finish_read(
        TelemetryConfigurationResult,
        ConfigurationPayload.model_validate(value),
        population="partial",
        reasons=[
            "Redacted effective calling-process settings are configuration, not SDK activation, native target settings or delivery evidence.",
        ],
    )


def trace(value: object) -> TelemetryTraceResult | DetailRefused:
    selected = TracePayload.model_validate(value)
    return finish_read(
        TelemetryTraceResult,
        selected,
        population="unavailable"
        if selected.status in {"unavailable", "stale"}
        else "partial",
        reasons=[
            selected.reason or selected.coverage or "bounded_live_selection",
            "Live bounded lookup has no atomic snapshot, pagination or population completeness guarantee. Missing records do not establish execution, loss, sampling or expiry; native results and settlement remain authoritative.",
        ],
    )
