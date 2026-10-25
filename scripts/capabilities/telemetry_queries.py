# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Passive selected telemetry reads through the original bounded host owners."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Annotated

from fastmcp.tools import Tool, ToolResult
from mcp.types import ResourceLink, TextContent, ToolAnnotations
from pydantic import Field, TypeAdapter

from scripts.capabilities import telemetry_contracts as answers
from scripts.capabilities.diagnostics import refusal
from scripts.capabilities.model import Capability
from scripts.capabilities.responses import owner_read
from scripts.telemetry_investigation import (
    BackendClient,
    QueryAnswer,
    QueryRequest,
    schema_description,
)
from scripts.telemetry_selection import (
    OwnerReference,
    ReaderSuggestion,
    SelectionReference,
)

State = Annotated[
    str, Field(min_length=1, max_length=4096, pattern=r"^[^\x00]+$", strict=True)
]
TraceId = Annotated[str, Field(pattern=r"^[0-9a-f]{32}$", strict=True)]
Microseconds = Annotated[int, Field(ge=0, le=(1 << 63) - 1, strict=True)]
Limit = Annotated[int, Field(ge=1, le=1000, strict=True)]


def bindings(
    _root: Path, *, client: BackendClient | None = None
) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    backend = client or BackendClient()

    async def telemetry_query(
        request: QueryRequest, state: State | None = None
    ) -> ToolResult:
        """Read a selected trace/operation/log population or metric points/distribution through the single bounded backend adapter. Select explicit microsecond times and identities. Metrics use the exact backend instrument stream; rate/increase/p95 are estimates, exemplars only related examples. Source/configuration fences and 3s/1MiB bounds apply. Live reads provide no stable pagination. Follow original native readers for scientific results and explicit profiling. No SQL, arbitrary endpoint, service startup or automatic action."""
        try:
            result = answers.query(await backend.query(state, request))
            encoded = result.model_dump(mode="json")
            return ToolResult(
                structured_content={"result": encoded},
                content=[
                    TextContent(
                        type="text",
                        text=f"{request.signal}/{request.kind}: selected observation; inspect structured coverage and source before drawing conclusions.",
                    ),
                    ResourceLink(
                        type="resource_link",
                        uri="pse://telemetry/schema",
                        name="telemetry_query_schema",
                        mimeType="application/json",
                    ),
                ],
            )
        except (OSError, RuntimeError, TypeError, ValueError) as error:
            raise refusal(
                "pse.telemetry.source_unavailable", operation="telemetry_query"
            ) from error

    def telemetry_schema() -> dict[str, object]:
        """Describe exact supported telemetry filters, signal fields, units, aggregation limits and original native readers without contacting a backend."""
        return dict(schema_description())

    async def telemetry_investigation_export(
        destination: State,
        selections: Annotated[list[QueryAnswer], Field(min_length=1, max_length=16)],
        question: Annotated[str, Field(min_length=1, max_length=4096)],
        maximum_output_bytes: Annotated[int, Field(ge=1, le=8 << 20)] = 8 << 20,
        missing_context: Annotated[list[str], Field(max_length=64)] | None = None,
        original_references: Annotated[list[OwnerReference], Field(max_length=32)]
        | None = None,
        next_readers: Annotated[list[ReaderSuggestion], Field(max_length=32)]
        | None = None,
    ) -> ToolResult:
        """Explicitly copy supplied query answers into a fresh finite native member pack; no new query or scientific analysis. Returns a digest-bound reference for stable offline pages. Native extension must already be prepared. Output exhaustion leaves its native incomplete publication visible; never overwrite an existing destination."""
        import anyio  # noqa: PLC0415

        from scripts import telemetry_selection  # noqa: PLC0415

        try:
            reference = await anyio.to_thread.run_sync(
                lambda: telemetry_selection.export(
                    Path(destination),
                    selections,
                    question=question,
                    original_references=original_references,
                    next_readers=next_readers,
                    maximum_output_bytes=maximum_output_bytes,
                    missing_context=missing_context,
                )
            )
            return ToolResult(
                structured_content=reference.model_dump(mode="json"),
                content=[
                    TextContent(
                        type="text",
                        text="Copied investigation published through native custody; live population completeness remains unestablished.",
                    ),
                    ResourceLink(
                        type="resource_link",
                        uri=f"pse://telemetry/selection/{telemetry_selection.resource_reference(reference)}/0/100",
                        name="copied_investigation",
                        mimeType="application/json",
                    ),
                ],
            )
        except (OSError, RuntimeError, TypeError, ValueError) as error:
            raise refusal(
                "pse.telemetry.export_refused",
                operation="telemetry_investigation_export",
            ) from error

    def telemetry_selection_read(
        reference: SelectionReference,
        selection: Annotated[int, Field(ge=0, le=15)] = 0,
        limit: Limit = 100,
        cursor: Annotated[str, Field(max_length=2048)] | None = None,
    ) -> answers.TelemetrySelectionResult | answers.DetailRefused:
        """Read stable pages from one exact copied investigation using original native custody plus the returned member digest. Supply identical reference, selection and limit on continuation. Offline reads use no current clock, backend, source paths or scientific rerun. Copied population completeness remains distinct from live population completeness."""
        from scripts import telemetry_selection  # noqa: PLC0415

        try:
            return answers.selection(
                telemetry_selection.page(
                    reference, selection=selection, limit=limit, cursor=cursor
                )
            )
        except (OSError, RuntimeError, TypeError, ValueError) as error:
            raise refusal(
                "pse.telemetry.selection_refused", operation="telemetry_selection_read"
            ) from error

    def telemetry_watch_read(
        start_time_us: Microseconds,
        end_time_us: Microseconds,
        state: State | None = None,
        limit: Limit = 100,
    ) -> answers.TelemetryWatchResult | answers.DetailRefused:
        """Read fixed native alert conditions, evaluation history and local delivery in an explicit interval up to seven days. Missing data, disabled/backlogged evaluation and unsupported history remain explicit. Backend-down local health remains telemetry_health. No polling scheduler or implicit intervention."""
        from scripts import telemetry_alerts  # noqa: PLC0415

        try:
            return answers.watch(
                telemetry_alerts.read(
                    state,
                    start_time_us=start_time_us,
                    end_time_us=end_time_us,
                    limit=limit,
                )
            )
        except (OSError, RuntimeError, TypeError, ValueError) as error:
            raise refusal(
                "pse.telemetry.source_unavailable", operation="telemetry_watch_read"
            ) from error

    def telemetry_watch_configure(
        enabled: bool = True, state: State | None = None
    ) -> answers.OwnerRecord:
        """Explicitly configure/enable/disable fixed OpenObserve-native conditions and local self-delivery. Requires an owned, prepared backend with narrow loopback delivery configured; no automatic startup, remediation or external messages. Missing source metric schemas are reported and require a later explicit configure call."""
        from scripts import telemetry_alerts  # noqa: PLC0415

        try:
            return answers.OwnerRecord.model_validate(
                telemetry_alerts.configure(state, enabled=enabled)
            )
        except (OSError, RuntimeError, TypeError, ValueError) as error:
            raise refusal(
                "pse.telemetry.watch_configuration_refused",
                operation="telemetry_watch_configure",
            ) from error

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

    async def telemetry_trace_lookup(
        trace_id: TraceId,
        start_time_us: Microseconds,
        end_time_us: Microseconds,
        state: State | None = None,
        limit: Limit = 100,
    ) -> answers.TelemetryTraceResult | answers.DetailRefused:
        """Look up one known nonzero trace ID in an explicit ordered microsecond interval, with 1..1000 rows. The original owner enforces HTTP byte/deadline bounds and backend incarnation/configuration fences. Unavailable, stale, partial and empty observations stay distinct. Live bounded selection has no completeness, pagination or custody guarantee; telemetry is not native result or settlement authority. No SQL, credentials, arbitrary endpoint or service startup is accepted."""
        from scripts import telemetry_query  # noqa: PLC0415 -- bounded read owner.

        try:
            selected = QueryRequest(
                signal="traces",
                trace_id=trace_id,
                start_time_us=start_time_us,
                end_time_us=end_time_us,
                limit=limit,
            )
            return answers.trace(
                telemetry_query.trace_answer(await backend.query(state, selected))
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
        (telemetry_investigation_export, "scripts/telemetry_selection.py"),
        (telemetry_selection_read, "scripts/telemetry_selection.py"),
        (telemetry_watch_read, "scripts/telemetry_alerts.py"),
        (telemetry_watch_configure, "scripts/telemetry_alerts.py"),
    )
    specs = []
    tools: dict[str, Tool] = {}
    for function, source in entries:
        effectful = function.__name__ in {
            "telemetry_investigation_export",
            "telemetry_watch_configure",
        }
        tool = Tool.from_function(
            function,
            annotations=ToolAnnotations(
                readOnlyHint=not effectful, destructiveHint=False, openWorldHint=False
            ),
        )
        if tool.name == "telemetry_query":
            selected_schema = TypeAdapter(
                answers.TelemetryQueryResult | answers.DetailRefused
            ).json_schema(mode="serialization")
            definitions = selected_schema.pop("$defs", {})
            tool = tool.model_copy(
                update={
                    "output_schema": {
                        "type": "object",
                        "$defs": definitions,
                        "properties": {"result": selected_schema},
                        "required": ["result"],
                        "additionalProperties": False,
                    }
                }
            )
        elif tool.name == "telemetry_investigation_export":
            tool = tool.model_copy(
                update={
                    "output_schema": SelectionReference.model_json_schema(
                        mode="serialization"
                    )
                }
            )
        tools[tool.name] = tool
        specs.append(
            Capability(
                invocation_lifetime="single",
                id=tool.name,
                title=tool.name.replace("_", " "),
                description=tool.description or "",
                domain="diagnostics",
                family="telemetry-observation",
                owner=source,
                source=source,
                guides=("diagnostics",),
                exposure="tool",
                effects="Explicit selected local publication/configuration; no scientific execution or automatic intervention"
                if effectful
                else "Passive bounded selected telemetry read; no startup, capture or subject settlement. Offline custody requires the prepared native reader.",
            )
        )
    return tuple(specs), tools
