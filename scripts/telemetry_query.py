# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded known-record telemetry reads; native results remain authoritative.

This is TEL13's prerequisite slice for pipeline visibility/recovery. It provides
no general SQL surface, metric semantics or snapshot/pagination guarantee.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path


def request(
    url: str,
    *,
    headers: Mapping[str, str] | None = None,
    body: dict[str, object] | None = None,
    method: str | None = None,
) -> tuple[int, bytes]:
    """Synchronous entry wrapper over the shared bounded async transport."""
    import anyio  # noqa: PLC0415 -- selected network operation only.

    from scripts.telemetry_investigation import BackendClient  # noqa: PLC0415

    async def read() -> tuple[int, bytes]:
        client = BackendClient()
        try:
            return await client.request(url, headers=headers, body=body, method=method)
        finally:
            await client.close()

    return anyio.run(read)


def trace_answer(value: object) -> dict[str, object]:
    """Project the retained known-trace API from the single query adapter."""
    from scripts.telemetry_investigation import QueryAnswer  # noqa: PLC0415

    selected = QueryAnswer.model_validate(value)
    answer = selected.model_dump(mode="json")
    rows = [
        {
            "trace_id": row.get("trace_id"),
            "span_id": row.get("span_id"),
            "parent_span_id": row.get("parent_span_id"),
            "parent_trace_id": row.get("parent_trace_id"),
            "service_instance_id": row.get("service_instance_id"),
            "operation_name": row.get("operation_name"),
            "timestamp_us": row.get("_timestamp"),
        }
        for row in selected.records
    ]
    return {
        "status": selected.status,
        "reason": selected.reason,
        "source": answer["source"],
        "records": rows,
        "complete": False,
        "returned_rows": selected.returned_rows,
        "row_limit": selected.request.limit,
        "http_status": selected.http_status,
        "coverage": selected.limits[0] if selected.limits else None,
        "population_total": None,
        "missing_record_meaning": "not observed in this selection; execution, sampling, loss and expiry are independent",
    }


def lookup_trace(
    state: Path | str | None,
    trace_id: str,
    *,
    start_time_us: int,
    end_time_us: int,
    limit: int = 100,
    environment: Mapping[str, str] | None = None,
) -> dict[str, object]:
    """Synchronous known-trace entry into the same typed bounded query core."""
    import anyio  # noqa: PLC0415
    from pydantic import ValidationError  # noqa: PLC0415

    from scripts.telemetry_investigation import (  # noqa: PLC0415
        BackendClient,
        QueryRequest,
    )

    try:
        selected = QueryRequest(
            signal="traces",
            trace_id=trace_id,
            start_time_us=start_time_us,
            end_time_us=end_time_us,
            limit=limit,
        )
    except ValidationError as error:
        raise ValueError(
            "Expected a valid typed telemetry selection; supplied values are redacted"
        ) from error

    async def read() -> dict[str, object]:
        client = BackendClient()
        try:
            return trace_answer(
                await client.query(state, selected, environment=environment)
            )
        finally:
            await client.close()

    return anyio.run(read)
