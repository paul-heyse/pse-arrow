# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded known-record telemetry reads; native results remain authoritative.

This is TEL13's prerequisite slice for pipeline visibility/recovery. It provides
no general SQL surface, metric semantics or snapshot/pagination guarantee.
"""

from __future__ import annotations

import http.client
import json
import re
import time
from typing import TYPE_CHECKING
from urllib.parse import urlsplit

from scripts import telemetry_service

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path

MAX_RESPONSE_BYTES = 1 << 20
TIMEOUT_SECONDS = 3


def request(
    url: str,
    *,
    headers: Mapping[str, str] | None = None,
    body: dict[str, object] | None = None,
) -> tuple[int, bytes]:
    """Read one finite HTTP response, without reporting credentials or server bodies."""
    parts = urlsplit(url)
    if (
        parts.scheme not in {"http", "https"}
        or parts.hostname is None
        or parts.username
        or parts.password
    ):
        raise ValueError("Unsupported telemetry service endpoint")
    import anyio  # noqa: PLC0415 -- only a selected network read needs the async client
    import httpx2  # noqa: PLC0415

    selected_headers = {"Accept-Encoding": "identity", **dict(headers or {})}

    async def read() -> tuple[int, bytes]:
        # Socket inactivity timeouts alone do not bound a slow-drip peer's headers/body.
        with anyio.fail_after(TIMEOUT_SECONDS):
            async with httpx2.AsyncClient(
                timeout=TIMEOUT_SECONDS, trust_env=False, follow_redirects=False
            ) as client:
                async with client.stream(
                    "GET" if body is None else "POST",
                    url,
                    headers=selected_headers,
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

    try:
        return anyio.run(read)
    except (httpx2.HTTPError, TimeoutError) as error:
        raise OSError("Telemetry read failed or exceeded its total deadline") from error


def _answer(
    raw: bytes, limit: int
) -> tuple[dict[str, object], list[dict[str, object]]]:
    value = json.loads(raw)
    if not isinstance(value, dict):
        raise TypeError("Telemetry backend returned an unsupported answer")
    hits = value.get("hits")
    if not isinstance(hits, list):
        raise TypeError("Telemetry backend returned an unsupported answer")
    if len(hits) > limit or any(not isinstance(row, dict) for row in hits):
        raise ValueError("Telemetry backend exceeded or violated the selected rows")
    return value, hits


def lookup_trace(
    state: Path | str | None,
    trace_id: str,
    *,
    start_time_us: int,
    end_time_us: int,
    limit: int = 100,
    environment: Mapping[str, str] | None = None,
) -> dict[str, object]:
    """Find a known trace in an explicit live interval, with truthful finite coverage."""
    if not re.fullmatch(r"[0-9a-f]{32}", trace_id) or trace_id == "0" * 32:
        raise ValueError("Expected a nonzero lower-case 32-digit trace ID")
    if (
        type(start_time_us) is not int
        or type(end_time_us) is not int
        or not 0 <= start_time_us < end_time_us
    ):
        raise ValueError("Expected a positive ordered microsecond interval")
    if type(limit) is not int or not 1 <= limit <= 1000:
        raise ValueError("Trace selection must contain from 1 to 1000 rows")
    inspected = telemetry_service.inspect(state, environment=environment)
    config = inspected.get("configuration")
    if not isinstance(config, dict):
        return {
            "status": "unavailable",
            "reason": "telemetry_not_configured",
            "records": [],
            "complete": False,
        }
    streams = config.get("streams")
    stream = streams.get("traces") if isinstance(streams, dict) else None
    organization = config.get("organization")
    if (
        not isinstance(stream, str)
        or not re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]{0,127}", stream)
        or not isinstance(organization, str)
        or not re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]{0,127}", organization)
    ):
        raise ValueError("Selected telemetry stream configuration is invalid")
    versions = inspected.get("versions")
    backend_version = (
        versions.get("openobserve") if isinstance(versions, dict) else None
    )
    source = {
        "configuration_id": config.get("configuration_id"),
        "backend": "openobserve",
        "backend_version": backend_version,
        "signal": "traces",
        "stream": stream,
        "start_time_us": start_time_us,
        "end_time_us": end_time_us,
        "retrieved_at_ns": time.time_ns(),
        "retention_days": inspected.get("retention_days", config.get("retention_days")),
        "source_cut": "live; no atomic snapshot or continuation guarantee",
    }
    base, headers = telemetry_service.backend_access(state, environment=environment)
    # Both identifiers and the sole literal have closed validated vocabularies.
    sql = f"SELECT trace_id,span_id,reference_parent_span_id,reference_parent_trace_id,service_service_instance_id,operation_name,_timestamp FROM {stream} WHERE trace_id = '{trace_id}' ORDER BY _timestamp,span_id"  # noqa: S608 -- stream identifier and trace literal are validated closed vocabularies above
    try:
        code, raw = request(
            base + f"/api/{organization}/_search?type=traces",
            headers=headers,
            body={
                "query": {
                    "sql": sql,
                    "start_time": start_time_us,
                    "end_time": end_time_us,
                    "from": 0,
                    "size": limit,
                },
                "timeout": TIMEOUT_SECONDS,
            },
        )
        if code != 200:
            return {
                "status": "unavailable",
                "reason": "backend_refused",
                "http_status": code,
                "source": source,
                "records": [],
                "complete": False,
            }
        value, hits = _answer(raw, limit)
        records = [
            {
                "trace_id": row.get("trace_id"),
                "span_id": row.get("span_id"),
                "parent_span_id": row.get("reference_parent_span_id"),
                "parent_trace_id": row.get("reference_parent_trace_id"),
                "service_instance_id": row.get("service_service_instance_id"),
                "operation_name": row.get("operation_name"),
                "timestamp_us": row.get("_timestamp"),
            }
            for row in hits
        ]
        partial = value.get("is_partial") is True or bool(value.get("function_error"))
        return {
            "status": "partial" if partial else ("available" if records else "empty"),
            "source": source,
            "records": records,
            "returned_rows": len(records),
            "row_limit": limit,
            "complete": False,
            "coverage": "backend_partial"
            if partial
            else (
                "row_limit_reached"
                if len(records) == limit
                else "bounded_live_selection"
            ),
            "population_total": None,
            "missing_record_meaning": "not observed in this selection; execution, sampling, loss and expiry are independent",
        }
    except (OSError, http.client.HTTPException, TypeError, ValueError):
        return {
            "status": "unavailable",
            "reason": "backend_read_failed",
            "source": source,
            "records": [],
            "complete": False,
        }
