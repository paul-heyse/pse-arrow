# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed, bounded OpenObserve observations; no scientific outcome authority."""

from __future__ import annotations

import json
import math
import re
import time
from pathlib import Path
from typing import TYPE_CHECKING, Annotated, Literal, Self
from urllib.parse import urlencode, urlsplit

from pydantic import (
    ConfigDict,
    Field,
    FiniteFloat,
    JsonValue,
    TypeAdapter,
    model_validator,
)

from scripts import telemetry_service
from scripts.capabilities.read_contracts import JSON_RECORD, Contract, OwnerRecord
from scripts.telemetry_metrics import METRIC_DECLARATIONS

if TYPE_CHECKING:
    from collections.abc import Mapping

    import httpx2

MAX_RESPONSE_BYTES = 1 << 20
TIMEOUT_SECONDS = 3
NATIVE_DECLARATIONS = (
    Path(__file__).resolve().parent / "generated" / "native_metric_declarations.json"
)
Identifier = Annotated[str, Field(pattern=r"^[a-zA-Z_][a-zA-Z0-9_]{0,127}$")]
Identity = Annotated[str, Field(pattern=r"^[a-zA-Z0-9_.:-]{1,160}$")]
Trace = Annotated[str, Field(pattern=r"^[0-9a-f]{32}$")]
Micros = Annotated[int, Field(ge=0, le=(1 << 63) - 1)]
QueryKind = Literal["points", "rate", "increase", "p95", "fraction_below", "exemplars"]
Meaning = Literal[
    "observed_points",
    "estimated_rate",
    "estimated_increase",
    "estimated_quantile",
    "estimated_fraction",
    "related_examples",
]
INTERPRETATIONS: dict[QueryKind, Meaning] = {
    "points": "observed_points",
    "rate": "estimated_rate",
    "increase": "estimated_increase",
    "p95": "estimated_quantile",
    "fraction_below": "estimated_fraction",
    "exemplars": "related_examples",
}
METRIC_RECORDS = TypeAdapter(list[dict[str, JsonValue]])
ATTRIBUTE_DOMAINS = TypeAdapter(dict[str, list[str]])


class QueryRequest(Contract):
    signal: Literal["traces", "logs", "metrics"]
    kind: QueryKind = "points"
    start_time_us: Micros
    end_time_us: Micros
    limit: Annotated[int, Field(ge=1, le=1000)] = 100
    trace_id: Trace | None = None
    span_id: Annotated[str, Field(pattern=r"^[0-9a-f]{16}$")] | None = None
    operation_id: Identity | None = None
    service_instance_id: Identity | None = None
    metric: Identifier | None = None
    step_seconds: Annotated[int, Field(ge=1, le=3600)] = 60
    threshold: FiniteFloat | None = None
    metric_filters: Annotated[dict[str, str], Field(max_length=16)] = Field(
        default_factory=dict
    )
    group_by: Annotated[list[str], Field(max_length=16)] = Field(default_factory=list)

    @model_validator(mode="after")
    def selection(self) -> QueryRequest:
        if self.start_time_us >= self.end_time_us:
            raise ValueError("Expected an ordered microsecond interval")
        if self.trace_id == "0" * 32 or self.span_id == "0" * 16:
            raise ValueError("Expected nonzero trace/span identities")
        if self.signal == "metrics":
            if (
                self.metric is None
                or self.trace_id
                or self.span_id
                or self.operation_id
            ):
                raise ValueError(
                    "Metric reads require a metric and bounded series filters"
                )
        elif self.kind != "points" or self.metric is not None:
            raise ValueError("Aggregations are selected metric reads only")
        elif not any((self.trace_id, self.operation_id, self.service_instance_id)):
            raise ValueError("Select a trace, operation or service incarnation")
        if self.signal != "metrics" and (self.metric_filters or self.group_by):
            raise ValueError("Dimensions select metric populations only")
        if self.kind in {"points", "exemplars"} and self.group_by:
            raise ValueError(
                "Grouping applies to metric estimators, not raw points or examples"
            )
        if len(set(self.group_by)) != len(self.group_by):
            raise ValueError("Select each grouping attribute once")
        if self.metric_filters or self.group_by:
            declaration = metric_declaration(self.metric)
            if declaration is None:
                raise ValueError("Dimension selection requires a declared instrument")
            domains = ATTRIBUTE_DOMAINS.validate_python(
                declaration.get("attribute_values", {}), strict=True
            )
            for key in {*self.metric_filters, *self.group_by}:
                if key not in domains:
                    raise ValueError("Attribute is not declared for this instrument")
            for key, value in self.metric_filters.items():
                if value not in domains[key]:
                    raise ValueError(
                        "Value is outside the instrument's finite vocabulary"
                    )
        if (self.kind == "fraction_below") != (self.threshold is not None):
            raise ValueError("A threshold is required only for fraction_below")
        if self.kind != "points" and (
            (self.end_time_us - self.start_time_us) // (self.step_seconds * 1_000_000)
            + 1
            > self.limit
        ):
            raise ValueError("Narrow the interval or increase the selected step")
        return self


class QuerySource(Contract):
    adapter: Literal["openobserve-1.1.2-v1"] = "openobserve-1.1.2-v1"
    backend: Literal["openobserve"] = "openobserve"
    configuration_id: str | None
    backend_version: str | None
    backend_process: OwnerRecord
    stream: str
    signal: str
    start_time_us: int
    end_time_us: int
    retrieved_at_ns: int
    retention_days: int | None = None
    lookback_seconds: int | None = None
    source_interval_start_us: int | None = None
    source_interval_end_us: int | None = None
    source_cut: Literal["bounded_live_observation"] = "bounded_live_observation"


class QueryAnswer(Contract):
    model_config = ConfigDict(allow_inf_nan=False)
    version: Literal[1] = 1
    request: QueryRequest
    status: Literal[
        "available", "empty", "partial", "unavailable", "stale", "unsupported"
    ]
    source: QuerySource | None = None
    records: list[dict[str, JsonValue]] = Field(default_factory=list)
    returned_rows: int = 0
    complete: Literal[False] = False
    meaning: Meaning = "observed_points"
    reason: str | None = None
    http_status: int | None = None
    unit: str | None = None
    limits: list[str] = Field(default_factory=list)
    native_readers: list[str] = Field(
        default_factory=lambda: [
            "native_diagnostic_inspect",
            "native_diagnostic_checkpoint",
        ]
    )


def schema_description() -> dict[str, JsonValue]:
    """Source-only adapter capabilities; no backend start, lookup or SDK import."""
    return JSON_RECORD.validate_python(
        {
            "adapter": "openobserve-1.1.2-v1",
            "request_schema": QueryRequest.model_json_schema(),
            "answer_schema": QueryAnswer.model_json_schema(mode="serialization"),
            "time_units": {
                "requests": "microseconds",
                "promql": "seconds",
                "rates": "per second",
                "duration_metrics": "instrument-declared",
            },
            "field_map": {
                "traces": {
                    "service_instance_id": "service_service_instance_id",
                    "parent_span_id": "reference_parent_span_id",
                    "parent_trace_id": "reference_parent_trace_id",
                },
                "logs": {"service_instance_id": "service_instance_id"},
                "metrics": {
                    "service_instance_id": "service_instance_id",
                    "stream": "exact backend instrument name",
                },
            },
            "bounds": {
                "http_bytes": MAX_RESPONSE_BYTES,
                "deadline_seconds": TIMEOUT_SECONDS,
                "rows": 1000,
            },
            "metric_declarations": metric_declarations(),
            "dimension_selection": "metric_filters uses original declared attribute keys and finite values; group_by uses those same keys for estimators. Empty grouping retains the existing estimator population; raw points/examples are never grouped. Backend label names replace dots with underscores.",
            "continuation": "Only immutable copied selections support stable continuation; live observations do not.",
            "metrics": "Points retain original reset/start fields. Rates, increases and bucket quantiles are estimates; missing native source epochs cannot establish exact interval counts. Rust native exemplars are unsupported; Python pointers are related examples only.",
            "guidance": [
                "telemetry_health",
                "telemetry_query",
                "native_diagnostic_inspect",
                "diagnostic_action",
                "telemetry_investigation_export",
            ],
        }
    )


def metric_declarations() -> list[dict[str, JsonValue]]:
    """Read generated Rust meanings and the actual host declaration, without native import."""
    native = METRIC_RECORDS.validate_json(NATIVE_DECLARATIONS.read_bytes())
    host: list[dict[str, JsonValue]] = [
        {
            "name": item.name,
            "kind": item.kind,
            "monotonic": item.monotonic,
            "exemplars": item.exemplars,
            "unit": item.unit,
            "population": item.population,
            "attributes": list[JsonValue](item.attributes),
            "buckets": list[JsonValue](item.buckets),
            "temporality": item.temporality,
            "reset": item.reset,
            "scope": item.scope,
            "attribute_values": {
                key: list[JsonValue](values)
                for key, values in item.attribute_values().items()
            },
        }
        for item in METRIC_DECLARATIONS
    ]
    return [
        {**item, "backend_name": str(item["name"]).replace(".", "_")}
        for item in [*native, *host]
    ]


def metric_declaration(stream: str | None) -> dict[str, JsonValue] | None:
    """Resolve actual names and declared histogram suffixes for dimension selection."""
    return next(
        (
            item
            for item in metric_declarations()
            if stream
            in (
                {item["backend_name"]}
                | {
                    str(item["backend_name"]) + suffix
                    for suffix in ("_bucket", "_count", "_sum")
                    if item.get("kind") == "histogram"
                }
            )
        ),
        None,
    )


MetricRefusalReason = Literal[
    "metric_instrument_not_declared",
    "metric_unit_not_declared",
    "metric_temporality_not_cumulative",
    "metric_requires_monotonic_counter",
    "metric_requires_monotonic_histogram",
    "metric_histogram_buckets_invalid",
    "metric_exemplars_not_supported",
    "selected_histogram_boundary_not_declared",
]


class MetricQueryRefusalError(ValueError):
    """A declared instrument cannot support the selected estimator."""

    def __init__(self, reason: MetricRefusalReason) -> None:
        self.reason = reason
        super().__init__(reason)


def admit_metric_query(request: QueryRequest) -> dict[str, JsonValue] | None:
    """PromQL estimators require known cumulative instrument semantics.

    Raw point observations may name an unknown stream. Histogram derivative
    stream names do not acquire the parent instrument's estimator semantics.
    """
    if request.signal != "metrics":
        return None
    declaration = next(
        (
            item
            for item in metric_declarations()
            if item["backend_name"] == request.metric
        ),
        None,
    )
    if request.kind == "points":
        return declaration
    if declaration is None:
        raise MetricQueryRefusalError("metric_instrument_not_declared")
    unit = declaration.get("unit")
    if not isinstance(unit, str) or not unit.strip():
        raise MetricQueryRefusalError("metric_unit_not_declared")
    if request.kind == "exemplars":
        if declaration.get("exemplars") is not True:
            raise MetricQueryRefusalError("metric_exemplars_not_supported")
        return declaration
    if declaration.get("temporality") != "cumulative":
        raise MetricQueryRefusalError("metric_temporality_not_cumulative")
    if request.kind in {"rate", "increase"}:
        if (
            declaration.get("kind") not in {"counter", "observable_counter"}
            or declaration.get("monotonic") is not True
        ):
            raise MetricQueryRefusalError("metric_requires_monotonic_counter")
    else:
        if (
            declaration.get("kind") != "histogram"
            or declaration.get("monotonic") is not True
        ):
            raise MetricQueryRefusalError("metric_requires_monotonic_histogram")
        raw_boundaries = declaration.get("buckets")
        if not isinstance(raw_boundaries, list) or not raw_boundaries:
            raise MetricQueryRefusalError("metric_histogram_buckets_invalid")
        boundaries: list[float] = []
        for bound in raw_boundaries:
            if isinstance(bound, bool) or not isinstance(bound, (int, float)):
                raise MetricQueryRefusalError("metric_histogram_buckets_invalid")
            try:
                numeric = float(bound)
            except OverflowError as error:
                raise MetricQueryRefusalError(
                    "metric_histogram_buckets_invalid"
                ) from error
            if not math.isfinite(numeric) or (boundaries and numeric <= boundaries[-1]):
                raise MetricQueryRefusalError("metric_histogram_buckets_invalid")
            boundaries.append(numeric)
        if request.kind == "fraction_below" and request.threshold not in boundaries:
            raise MetricQueryRefusalError("selected_histogram_boundary_not_declared")
    return declaration


class BackendClient:
    """One lazy async client per endpoint lifetime; credentials remain request-local."""

    def __init__(self) -> None:
        self._client: httpx2.AsyncClient | None = None

    async def __aenter__(self) -> Self:
        return self

    async def __aexit__(self, *_exception: object) -> None:
        await self.close()

    async def close(self) -> None:
        client, self._client = self._client, None
        if client is not None:
            await client.aclose()

    async def request(
        self,
        url: str,
        *,
        headers: Mapping[str, str] | None = None,
        body: dict[str, object] | None = None,
        method: str | None = None,
    ) -> tuple[int, bytes]:
        import anyio  # noqa: PLC0415 -- selected network read only
        import httpx2  # noqa: PLC0415

        parts = urlsplit(url)
        if (
            parts.scheme not in {"http", "https"}
            or parts.hostname is None
            or parts.username
            or parts.password
        ):
            raise ValueError("Unsupported telemetry endpoint")
        # URLs and credentials are request-local. Rotating this pool for another
        # selection would close active sibling reads; source fences belong to query().
        if self._client is None:
            self._client = httpx2.AsyncClient(
                timeout=TIMEOUT_SECONDS, trust_env=False, follow_redirects=False
            )
        try:
            with anyio.fail_after(TIMEOUT_SECONDS):
                async with self._client.stream(
                    method or ("GET" if body is None else "POST"),
                    url,
                    headers={"Accept-Encoding": "identity", **dict(headers or {})},
                    json=body,
                ) as response:
                    content = bytearray()
                    async for chunk in response.aiter_raw():
                        if len(content) + len(chunk) > MAX_RESPONSE_BYTES:
                            raise ValueError(
                                "Telemetry response exceeds selected byte bound"
                            )
                        content.extend(chunk)
                    return response.status_code, bytes(content)
        except (httpx2.HTTPError, TimeoutError) as error:
            raise OSError(
                "Telemetry read failed or exceeded its total deadline"
            ) from error

    async def query(
        self,
        state: Path | str | None,
        selected: QueryRequest,
        *,
        environment: Mapping[str, str] | None = None,
    ) -> QueryAnswer:
        import anyio  # noqa: PLC0415 -- native service inspection is blocking

        try:
            declaration = admit_metric_query(selected)
        except MetricQueryRefusalError as refusal:
            return QueryAnswer(
                request=selected,
                status="unsupported",
                reason=refusal.reason,
                meaning=INTERPRETATIONS[selected.kind],
            )
        inspected = await anyio.to_thread.run_sync(
            lambda: telemetry_service.inspect(state, environment=environment)
        )
        config = inspected.get("configuration")
        services = inspected.get("services")
        backend = services.get("openobserve") if isinstance(services, dict) else None
        if not isinstance(config, dict):
            return QueryAnswer(
                request=selected,
                status="unavailable",
                reason="telemetry_not_configured",
            )
        if (
            not isinstance(backend, dict)
            or backend.get("ownership") != "verified"
            or not isinstance(backend.get("process"), dict)
        ):
            return QueryAnswer(
                request=selected,
                status="unavailable",
                reason="selected_backend_not_active_or_owned",
            )
        organization = identifier(config.get("organization"))
        streams = config.get("streams")
        stream = (
            selected.metric
            if selected.signal == "metrics"
            else streams.get(selected.signal)
            if isinstance(streams, dict)
            else None
        )
        stream = identifier(stream)
        versions = inspected.get("versions")
        source = QuerySource.model_validate(
            {
                "configuration_id": config.get("configuration_id"),
                "backend_version": versions.get("openobserve")
                if isinstance(versions, dict)
                else None,
                "backend_process": backend["process"],
                "stream": stream,
                "signal": selected.signal,
                "start_time_us": selected.start_time_us,
                "end_time_us": selected.end_time_us,
                "retrieved_at_ns": time.time_ns(),
                "retention_days": inspected.get(
                    "retention_days", config.get("retention_days")
                ),
                "lookback_seconds": lookback_seconds(selected)
                if selected.kind not in {"points", "exemplars"}
                else None,
                "source_interval_start_us": max(
                    0,
                    selected.start_time_us
                    - (
                        lookback_seconds(selected) * 1_000_000
                        if selected.kind not in {"points", "exemplars"}
                        else 0
                    ),
                ),
                "source_interval_end_us": selected.end_time_us,
            }
        )
        path, body = _backend_request(organization, stream, selected)
        unit = declaration.get("unit") if declaration else None
        unit = unit if isinstance(unit, str) else None
        if selected.kind == "fraction_below":
            unit = "1"
        elif selected.kind == "rate" and unit is not None:
            unit += "/s"
        meaning = INTERPRETATIONS[selected.kind]
        try:
            base, headers = await anyio.to_thread.run_sync(
                lambda: telemetry_service.backend_access(state, environment=environment)
            )
            code, raw = await self.request(
                base + path,
                headers=headers,
                body=body,
            )
            current = await anyio.to_thread.run_sync(
                lambda: telemetry_service.inspect(state, environment=environment)
            )
            current_services = current.get("services")
            current_backend = (
                current_services.get("openobserve")
                if isinstance(current_services, dict)
                else None
            )
            if (
                current.get("configuration") != config
                or not isinstance(current_backend, dict)
                or current_backend.get("ownership") != "verified"
                or current_backend.get("process") != backend["process"]
            ):
                return QueryAnswer(
                    request=selected,
                    status="stale",
                    source=source,
                    reason="selected_backend_changed_during_read",
                )
            if code != 200:
                return QueryAnswer(
                    request=selected,
                    status="unavailable",
                    source=source,
                    reason="backend_refused",
                    http_status=code,
                )
            rows, partial = response_rows(raw, selected)
            return QueryAnswer(
                request=selected,
                status="partial" if partial else "available" if rows else "empty",
                source=source,
                records=[normalize(selected.signal, row) for row in rows],
                returned_rows=len(rows),
                meaning=meaning,
                unit=unit,
                limits=[
                    "backend_partial"
                    if partial
                    else "row_limit_reached"
                    if len(rows) == selected.limit
                    else "bounded_live_selection",
                    "Missing records cannot distinguish nonexecution, sampling, loss or expiry.",
                    "Native source epochs are independent of SDK aggregation start_time; exact interval counts require matching epochs and a first baseline.",
                    "Overflow may preserve overall totals while losing dimension-filter coverage.",
                ],
            )
        except (OSError, TypeError, ValueError):
            return QueryAnswer(
                request=selected,
                status="unavailable",
                source=source,
                reason="backend_read_failed",
            )


def response_rows(
    raw: bytes, selected: QueryRequest
) -> tuple[list[dict[str, JsonValue]], bool]:
    """Validate the selected backend layout at its single normalization boundary."""
    value = JSON_RECORD.validate_json(raw)
    if selected.kind == "points":
        rows = value.get("hits")
    elif value.get("status") == "success":
        data = value.get("data")
        rows = (
            data
            if isinstance(data, list)
            else data.get("result")
            if isinstance(data, dict)
            else None
        )
    else:
        raise ValueError("Unsupported backend answer")
    selected_rows = METRIC_RECORDS.validate_python(rows, strict=True)
    if len(selected_rows) > selected.limit:
        raise ValueError("Backend exceeded the selected row population")
    return selected_rows, value.get("is_partial") is True or bool(
        value.get("function_error")
    )


def backend_request(
    organization: str, stream: str, request: QueryRequest
) -> tuple[str, dict[str, object] | None]:
    """Closed request vocabulary; caller strings never become executable query text."""
    admit_metric_query(request)
    return _backend_request(organization, stream, request)


def _backend_request(
    organization: str, stream: str, request: QueryRequest
) -> tuple[str, dict[str, object] | None]:
    """Lower a semantically admitted question to the selected backend vocabulary."""
    if request.signal == "metrics" and stream != request.metric:
        raise MetricQueryRefusalError("metric_instrument_not_declared")
    predicates = []
    fields = {
        "trace_id": request.trace_id,
        "span_id": request.span_id,
        "pse_operation_id": request.operation_id,
        "service_service_instance_id"
        if request.signal == "traces"
        else "service_instance_id": request.service_instance_id,
    }
    fields.update(
        {key.replace(".", "_"): value for key, value in request.metric_filters.items()}
    )
    for name, value in fields.items():
        if value is not None:
            predicates.append(name + " = '" + value.replace("'", "''") + "'")
    if request.kind == "points":
        where = " WHERE " + " AND ".join(predicates) if predicates else ""
        # Every identifier/literal above belongs to the validated closed vocabulary.
        order = "_timestamp,span_id" if request.signal == "traces" else "_timestamp"
        # Resource/attribute columns exist only after a producer emits them.
        # Optional native-operation and parent fields cannot be unconditional
        # projections of an otherwise valid ordinary SDK trace population.
        sql = f'SELECT * FROM "{stream}"{where} ORDER BY {order}'  # noqa: S608 -- validated identifiers and restricted literals
        return f"/api/{organization}/_search?type={request.signal}", {
            "query": {
                "sql": sql,
                "start_time": request.start_time_us,
                "end_time": request.end_time_us,
                "from": 0,
                "size": request.limit,
            },
            "timeout": TIMEOUT_SECONDS,
        }
    label_values = {
        key.replace(".", "_"): value for key, value in request.metric_filters.items()
    }
    if request.service_instance_id:
        label_values["service_instance_id"] = request.service_instance_id
    labels = (
        "{"
        + ",".join(
            f"{key}={json.dumps(value)}" for key, value in sorted(label_values.items())
        )
        + "}"
        if label_values
        else ""
    )
    groups = [key.replace(".", "_") for key in request.group_by]

    def summed(expression: str, *, histogram: bool = False) -> str:
        keys = [*(["le"] if histogram else []), *groups]
        return (
            f"sum by({','.join(keys)})({expression})" if keys else f"sum({expression})"
        )

    expression = stream + labels
    window = lookback_seconds(request)
    if request.kind in {"rate", "increase"}:
        expression = f"{request.kind}({expression}[{window}s])"
        if groups:
            expression = summed(expression)
    elif request.kind == "p95":
        expression = f"histogram_quantile(0.95,{summed(f'rate({stream}_bucket{labels}[{window}s])', histogram=True)})"
    elif request.kind == "fraction_below":
        bound = format(request.threshold, ".15g")
        bucket_values = {**label_values, "le": bound}
        bucket_labels = (
            "{"
            + ",".join(
                f"{key}={json.dumps(value)}"
                for key, value in sorted(bucket_values.items())
            )
            + "}"
        )
        expression = f"{summed(f'rate({stream}_bucket{bucket_labels}[{window}s])')}/{summed(f'rate({stream}_count{labels}[{window}s])')}"
    parameters: dict[str, str | int | float] = {
        "query": expression,
        "start": request.start_time_us / 1_000_000,
        "end": request.end_time_us / 1_000_000,
    }
    endpoint = "query_exemplars" if request.kind == "exemplars" else "query_range"
    if endpoint == "query_range":
        parameters["step"] = request.step_seconds
    return f"/api/{organization}/prometheus/api/v1/{endpoint}?" + urlencode(
        parameters
    ), None


def lookback_seconds(request: QueryRequest) -> int:
    """Range estimators use one requested-interval lookback at each evaluation point."""
    return max(1, (request.end_time_us - request.start_time_us) // 1_000_000)


def identifier(value: object) -> str:
    if not isinstance(value, str) or not re.fullmatch(
        r"[a-zA-Z_][a-zA-Z0-9_]{0,127}", value
    ):
        raise ValueError("Unsupported backend stream configuration")
    return value


def normalize(signal: str, row: dict[str, JsonValue]) -> dict[str, JsonValue]:
    """Keep raw selected facts plus explicit aliases, including point-specific resets."""
    normalized = dict(row)
    names = (
        {
            "service_service_instance_id": "service_instance_id",
            "reference_parent_span_id": "parent_span_id",
            "reference_parent_trace_id": "parent_trace_id",
        }
        if signal == "traces"
        else {}
    )
    for original, alias in names.items():
        if original in row:
            normalized[alias] = row[original]
    return normalized
