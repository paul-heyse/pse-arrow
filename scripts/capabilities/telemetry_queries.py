# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Passive selected telemetry reads through the original bounded host owners."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Annotated

from fastmcp.tools import Tool, ToolResult
from mcp.types import ResourceLink, TextContent, ToolAnnotations
from pydantic import Field, TypeAdapter

from scripts.capabilities import telemetry_contracts as answers
from scripts.capabilities.diagnostics import refusal
from scripts.capabilities.model import Capability
from scripts.capabilities.responses import owner_read
from scripts.telemetry_investigation import BackendClient, QueryRequest, schema_description

if TYPE_CHECKING:
    from pathlib import Path


State = Annotated[
    str, Field(min_length=1, max_length=4096, pattern=r"^[^\x00]+$", strict=True)
]
TraceId = Annotated[str, Field(pattern=r"^[0-9a-f]{32}$", strict=True)]
Microseconds = Annotated[int, Field(ge=0, le=(1 << 63) - 1, strict=True)]
Limit = Annotated[int, Field(ge=1, le=1000, strict=True)]


def bindings(_root: Path, *, client: BackendClient | None = None) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    backend = client or BackendClient()

    async def telemetry_query(request: QueryRequest, state: State | None = None) -> ToolResult:
        """Read a selected trace/operation/log population or metric points/distribution through the single bounded backend adapter. Select explicit microsecond times and identities. Metrics use the exact backend instrument stream; rate/increase/p95 are estimates, exemplars only related examples. Source/configuration fences and 3s/1MiB bounds apply. Live reads provide no stable pagination. Follow original native readers for scientific results and explicit profiling. No SQL, arbitrary endpoint, service startup or automatic action."""
        try:
            result = answers.query(await backend.query(state, request))
            encoded = result.model_dump(mode="json")
            return ToolResult(
                structured_content={"result": encoded},
                content=[TextContent(type="text", text=f"{request.signal}/{request.kind}: selected observation; inspect structured coverage and source before drawing conclusions."),
                         ResourceLink(type="resource_link", uri="pse://telemetry/schema", name="telemetry_query_schema", mimeType="application/json")],
            )
        except (OSError, RuntimeError, TypeError, ValueError) as error:
            raise refusal("pse.telemetry.source_unavailable", operation="telemetry_query") from error

    def telemetry_schema() -> dict[str, object]:
        """Describe exact supported telemetry filters, signal fields, units, aggregation limits and original native readers without contacting a backend."""
        return dict(schema_description())
    def telemetry_health(
        state: State | None = None,
    ) -> answers.TelemetryHealthResult | answers.DetailRefused:
        """Passively read the selected pipeline's owned service health and collector delivery counters. Producer state is this calling MCP process, never a native target's metrics. Each HTTP read retains its owner's total deadline and byte bound. No install, start, capture, SDK activation or native import; native results/settlement remain authoritative. Use native_diagnostic_inspect for target checkpoint runtime_metrics, preserving its target identity and registration coverage."""
        from scripts import telemetry  # noqa: PLC0415 -- selected read owner only.

        try:
            return owner_read(
                "telemetry_health",
                lambda: telemetry.health(state),
                project=answers.health,
            )
        except (RuntimeError, TypeError, ValueError) as error:
            raise refusal(
                "pse.telemetry.source_unavailable", operation="telemetry_health"
            ) from error

    def telemetry_configuration(
        state: State | None = None,
    ) -> answers.TelemetryConfigurationResult | answers.DetailRefused:
        """Read redacted effective telemetry settings for this calling MCP process. Optional state selects persisted local configuration; environment precedence stays with the host resolver. Endpoint paths and header values remain redacted. Configuration establishes neither SDK activation, target settings nor delivery; no service startup or native import."""
        from scripts import telemetry_config  # noqa: PLC0415 -- passive resolver.

        environment = (
            None if state is None else {**os.environ, "PSE_TELEMETRY_STATE": state}
        )
        try:
            return owner_read(
                "telemetry_configuration",
                lambda: telemetry_config.resolve_process(
                    "pse-capabilities", environment=environment
                ).effective(),
                project=answers.configuration,
            )
        except (RuntimeError, TypeError, ValueError) as error:
            raise refusal(
                "pse.telemetry.source_unavailable",
                operation="telemetry_configuration",
            ) from error

    def telemetry_trace_lookup(
        trace_id: TraceId,
        start_time_us: Microseconds,
        end_time_us: Microseconds,
        state: State | None = None,
        limit: Limit = 100,
    ) -> answers.TelemetryTraceResult | answers.DetailRefused:
        """Look up one known nonzero trace ID in an explicit ordered microsecond interval, with 1..1000 rows. The original owner enforces HTTP byte/deadline bounds and backend incarnation/configuration fences. Unavailable, stale, partial and empty observations stay distinct. Live bounded selection has no completeness, pagination or custody guarantee; telemetry is not native result or settlement authority. No SQL, credentials, arbitrary endpoint or service startup is accepted."""
        from scripts import telemetry_query  # noqa: PLC0415 -- bounded read owner.

        try:
            return owner_read(
                "telemetry_trace_lookup",
                lambda: telemetry_query.lookup_trace(
                    state,
                    trace_id,
                    start_time_us=start_time_us,
                    end_time_us=end_time_us,
                    limit=limit,
                ),
                project=answers.trace,
            )
        except (RuntimeError, TypeError, ValueError) as error:
            raise refusal(
                "pse.telemetry.source_unavailable", operation="telemetry_trace_lookup"
            ) from error

    entries = (
        (telemetry_health, "scripts/telemetry.py"),
        (telemetry_configuration, "scripts/telemetry_config.py"),
        (telemetry_trace_lookup, "scripts/telemetry_query.py"),
        (telemetry_query, "scripts/telemetry_investigation.py"),
        (telemetry_schema, "scripts/telemetry_investigation.py"),
    )
    specs = []
    tools: dict[str, Tool] = {}
    for function, source in entries:
        tool = Tool.from_function(
            function,
            annotations=ToolAnnotations(
                readOnlyHint=True, destructiveHint=False, openWorldHint=False
            ),
        )
        if tool.name == "telemetry_query":
            selected_schema = TypeAdapter(answers.TelemetryQueryResult | answers.DetailRefused).json_schema(mode="serialization")
            definitions = selected_schema.pop("$defs", {})
            tool = tool.model_copy(update={"output_schema": {"type": "object", "$defs": definitions, "properties": {"result": selected_schema}, "required": ["result"], "additionalProperties": False}})
        tools[tool.name] = tool
        specs.append(
            Capability(
                id=tool.name,
                title=tool.name.replace("_", " "),
                description=tool.description or "",
                domain="diagnostics",
                family="telemetry-observation",
                owner=source,
                source=source,
                guides=("diagnostics",),
                exposure="tool",
                effects="Passive bounded selected telemetry read; no startup, capture, native import or subject settlement",
            )
        )
    return tuple(specs), tools
