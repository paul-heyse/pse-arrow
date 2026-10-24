# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded interface correlation, borrowing native identities without reading detail.

FastMCP owns protocol/provider spans. These diagnostics describe component boundaries,
not response delivery, scientific success, or application-to-database continuity.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import re
import sys
import time
from contextlib import suppress
from contextvars import ContextVar
from dataclasses import dataclass, field
from pathlib import PurePath
from typing import TYPE_CHECKING, TypeVar
from uuid import uuid4

from fastmcp.exceptions import NotFoundError, PromptError, ResourceError, ToolError
from fastmcp.exceptions import ValidationError as FrameworkValidationError
from fastmcp.prompts import PromptResult
from fastmcp.resources import ResourceResult
from fastmcp.server.middleware import Middleware
from fastmcp.tools import ToolResult
from mcp.types import CallToolResult
from opentelemetry.trace import get_current_span
from pydantic import ValidationError

from scripts.capabilities.native_contracts import BINDINGS

if TYPE_CHECKING:
    from collections.abc import Mapping

    from fastmcp import FastMCP
    from fastmcp.server.middleware import CallNext, MiddlewareContext
    from mcp.types import (
        CallToolRequestParams,
        GetPromptRequestParams,
        ReadResourceRequestParams,
    )
    from pydantic import JsonValue

META_KEY = "pse.observation"
MessageT = TypeVar("MessageT")
ResultT = TypeVar("ResultT", ToolResult, ResourceResult, PromptResult)
MAX_FRAMES = 16
MAX_REFERENCES = 8
MAX_META_BYTES = 16384
NATIVE_TOOLS = frozenset("native_" + item.operation for item in BINDINGS)
ASSESSMENT_TOOLS = frozenset(
    {
        "assessment_run",
        "compilation_check",
        "contracts_check",
        "contracts_generate",
        "documentation_build",
    }
)
OWNER_TOOLS = (
    NATIVE_TOOLS
    | ASSESSMENT_TOOLS
    | {
        "product_run_read",
        "workload_watch",
        "assessment_read",
        "rust_test_run",
        "experiment_archive",
        "experiment_replay",
        "diagnostic_action",
        "diagnostic_action_read",
    }
)
INTERFACE_TOOLS = frozenset(
    {
        "capabilities_domains",
        "capabilities_workflows",
        "capabilities_describe",
        "capabilities_select",
        "capabilities_reset",
        "capabilities_invoke_read",
        "capabilities_invoke_action",
    }
)


def _object(value: object) -> dict:
    return value if isinstance(value, dict) else {}


def _text(value: object) -> str | None:
    # Omit an oversized identity entirely: truncating it would invent a handle.
    return value if isinstance(value, str) and 0 < len(value) <= 2048 else None


def _product(value: dict) -> dict[str, JsonValue] | None:
    invocation = _text(value.get("invocation"))
    registry = _text(value.get("registry"))
    if invocation is None or (
        registry is None and not PurePath(invocation).is_absolute()
    ):
        # A nonce without its issuing registry cannot be converted into an
        # authentic read by selecting the observer's current/default registry.
        return None
    arguments: dict[str, JsonValue] = {"invocation": invocation, "tail": 0}
    if registry is not None:
        arguments["registry"] = registry
    return {
        "owner": "scripts/run_inspection.py",
        "capability": "product_run_read",
        "arguments": arguments,
    }


def _native(
    name: str, content: object
) -> tuple[dict[str, JsonValue], list[dict[str, JsonValue]]]:
    """Select only declared owner envelopes/fields, never scan arbitrary payloads."""
    envelope = _object(content)
    # These typed reader unions have one framework-owned serialization wrapper.
    # Unwrap only that exact published shape, never an arbitrary nested payload.
    if name in {"product_run_read", "assessment_read", "workload_watch"} and set(
        envelope
    ) == {"result"}:
        envelope = _object(envelope["result"])
    operation = {
        "experiment_archive": "archive",
        "experiment_replay": "replay",
        "diagnostic_action": "diagnostic",
    }.get(name, name)
    if envelope.get("operation") != operation:
        return {}, []
    read = "payload" in envelope and "complete" in envelope
    host = "result" in envelope and "detail_complete" in envelope
    payload = _object(envelope.get("payload" if read else "result"))
    if not (read or host):
        return {}, []
    signals: dict[str, JsonValue] = {}
    references: list[dict[str, JsonValue]] = []
    if name not in OWNER_TOOLS:
        return {}, []
    signals["projection_complete"] = (
        envelope.get("complete" if read else "detail_complete") is True
    )
    omissions = envelope.get("omissions")
    signals["omission_count"] = len(omissions) if isinstance(omissions, list) else 0
    # These are owner-reported signals, not a synthesized semantic pass/fail.
    for key in (
        "outcome",
        "status",
        "phase",
        "terminal_status",
        "assurance",
        "owned_drain",
    ):
        value = payload.get(key)
        if isinstance(value, str) and re.fullmatch(r"[A-Za-z0-9_. -]{1,96}", value):
            signals[key] = value
    for key in (
        "exit_code",
        "workload_exit",
        "complete",
        "required_checks_covered",
        "input_coverage",
        "terminal_observed",
    ):
        value = payload.get(key)
        if isinstance(value, (bool, int)):
            signals[key] = value

    product: dict | None = None
    if name == "product_run_read" and payload.get("kind") == "product-test":
        product = payload
    elif name == "workload_watch":
        candidate = _object(_object(payload.get("snapshot")).get("run"))
        if candidate.get("kind") == "product-test":
            product = candidate
    elif name == "rust_test_run":
        reader = _object(payload.get("catalogue_reader"))
        if reader.get("capability") == "product_run_read" and reader.get(
            "invocation"
        ) == payload.get("invocation"):
            product = reader
    elif name in {"experiment_archive", "experiment_replay"}:
        product = {
            "invocation": payload.get(
                "producer_invocation" if name == "experiment_archive" else "invocation"
            )
        }
    elif name in {"diagnostic_action", "diagnostic_action_read"}:
        association = _object(payload.get("invocation"))
        product = {
            "invocation": association.get("identity"),
            "registry": association.get("registry"),
        }
        output = _text(payload.get("output"))
        if output is not None:
            references.append(
                {
                    "owner": "scripts/active_diagnostics.py",
                    "capability": "diagnostic_action_read",
                    "arguments": {"request": {"output": output, "summary_only": True}},
                }
            )
    if product is not None:
        reference = _product(product)
        if reference is not None:
            references.append(reference)
    if name in ASSESSMENT_TOOLS:
        reader = _object(payload.get("reader"))
        directory = _text(reader.get("directory"))
        if reader.get("capability") == "assessment_read" and directory is not None:
            references.append(
                {
                    "owner": "scripts/validation_result.py",
                    "capability": "assessment_read",
                    "arguments": {"directory": directory, "tail": 0},
                }
            )
    elif name == "assessment_read":
        directory = _text(payload.get("run_path"))
        if directory is not None:
            references.append(
                {
                    "owner": "scripts/validation_result.py",
                    "capability": "assessment_read",
                    "arguments": {"directory": directory, "tail": 0},
                }
            )
    return signals, references


def _failure(error: Exception) -> tuple[str, str]:
    if isinstance(error, NotFoundError):
        return "provider", "pse.capabilities.provider_unavailable"
    if isinstance(error, (ValidationError, FrameworkValidationError)):
        return "adapter", "pse.capabilities.input_refused"
    if isinstance(error, (ToolError, ResourceError, PromptError)):
        try:
            value = json.loads(str(error)) if len(str(error)) <= 16384 else {}
        except ValueError:
            value = {}
        code = _object(value).get("code")
        if code is None:
            match = re.match(
                r"^(pse\.capabilities\.[a-z0-9_.]{1,100}):", str(error)[:128]
            )
            code = match.group(1) if match else None
        if isinstance(code, str) and re.fullmatch(r"pse\.[a-z0-9_.]{1,120}", code):
            stage = (
                "native"
                if code.startswith("pse.native.")
                else "interface"
                if code.startswith("pse.capabilities.")
                else "adapter"
            )
            return stage, code
        return "adapter", "pse.capabilities.tool_refused"
    return "unknown", "pse.capabilities.tool_failed"


def _trace() -> dict[str, JsonValue]:
    span = get_current_span()
    context = span.get_span_context()
    value: dict[str, JsonValue] = {
        "recording": span.is_recording(),
        "export": "unverified",
        "end_to_end_continuity": "unestablished",
    }
    if context.is_valid:
        value.update(
            trace_id=f"{context.trace_id:032x}", span_id=f"{context.span_id:016x}"
        )
    else:
        value["context"] = "unavailable"
    return value


@dataclass
class _Request:
    correlation: str
    revision: str
    request_id_sha256: str | None
    trace: dict[str, JsonValue]
    frames: list[dict[str, JsonValue]] = field(default_factory=list)
    references: list[dict[str, JsonValue]] = field(default_factory=list)
    omitted_frames: int = 0
    omitted_references: int = 0

    def metadata(self) -> dict[str, JsonValue]:
        references: list[JsonValue] = list(self.references)
        calls: list[JsonValue] = list(self.frames)
        value: dict[str, JsonValue] = {
            "correlation": self.correlation,
            "loaded_revision": self.revision,
            "request_id_sha256": self.request_id_sha256,
            "calls": calls,
            "native_references": references,
            "omitted_calls": self.omitted_frames,
            "omitted_references": self.omitted_references,
            "trace": self.trace,
            "transport_delivery": "unobserved",
            "native_transport": "server_unobserved_client_errors",
            "native_timing": "unobserved",
            "reference_scope": "Declared supported owner output fields only; missing or oversized identities are not inferred.",
        }
        omitted_references = self.omitted_references
        omitted_calls = self.omitted_frames
        while len(json.dumps(value, ensure_ascii=True).encode()) > MAX_META_BYTES:
            if references:
                references.pop()
                omitted_references += 1
                value["omitted_references"] = omitted_references
            elif calls:
                calls.pop()
                omitted_calls += 1
                value["omitted_calls"] = omitted_calls
            else:
                break
        return value


class Observation(Middleware):
    """One request record across nested invoke dispatch; no subject lifetime ownership."""

    def __init__(
        self,
        revision: str,
        *,
        operation_names: frozenset[str] = frozenset(),
        resource_names: Mapping[str, str] | None = None,
        prompt_names: frozenset[str] = frozenset(),
    ) -> None:
        if not revision or len(revision) > 128:
            raise ValueError("bounded loaded interface revision required")
        self.revision = revision
        self.operation_names = operation_names | OWNER_TOOLS | INTERFACE_TOOLS
        self.prompt_names = prompt_names
        self.resource_names = dict(resource_names or {})
        if any(
            len(name) > 128 or not name
            for name in (
                *self.operation_names,
                *self.prompt_names,
                *self.resource_names.values(),
            )
        ):
            raise ValueError("bounded registered component names required")
        self._active: ContextVar[tuple[_Request, int | None] | None] = ContextVar(
            "pse_interface_observation", default=None
        )

    def _component_name(self, kind: str, identity: str) -> str | None:
        if kind == "tool":
            return identity if identity in self.operation_names else None
        if kind == "prompt":
            return identity if identity in self.prompt_names else None
        for template, name in self.resource_names.items():
            # Labels come only from declarations. No resolved URI or template
            # parameter is exposed, including for missing/failed resource reads.
            pattern = re.escape(template)
            pattern = re.sub(r"\\\{[^{}]+\\\}", "[^/?#]+", pattern)
            if re.fullmatch(pattern, identity):
                return name
        return None

    async def on_call_tool(
        self,
        context: MiddlewareContext[CallToolRequestParams],
        call_next: CallNext[CallToolRequestParams, ToolResult],
    ) -> ToolResult:
        return await self._observe(context, call_next, "tool", context.message.name)

    async def on_read_resource(
        self,
        context: MiddlewareContext[ReadResourceRequestParams],
        call_next: CallNext[ReadResourceRequestParams, ResourceResult],
    ) -> ResourceResult:
        return await self._observe(context, call_next, "resource", context.message.uri)

    async def on_get_prompt(
        self,
        context: MiddlewareContext[GetPromptRequestParams],
        call_next: CallNext[GetPromptRequestParams, PromptResult],
    ) -> PromptResult:
        return await self._observe(context, call_next, "prompt", context.message.name)

    async def _observe(
        self,
        context: MiddlewareContext[MessageT],
        call_next: CallNext[MessageT, ResultT],
        kind: str,
        identity: str,
    ) -> ResultT:
        parent = self._active.get()
        root = parent is None
        if parent is None:
            ctx = context.fastmcp_context
            request_id = ctx.origin_request_id if ctx is not None else None
            state = _Request(
                uuid4().hex,
                self.revision,
                hashlib.sha256(request_id.encode()).hexdigest()
                if request_id is not None
                else None,
                _trace(),
            )
            parent_index = None
        else:
            state, parent_index = parent
        index = len(state.frames) if len(state.frames) < MAX_FRAMES else None
        frame: dict[str, JsonValue] = {
            "call": index,
            "parent": parent_index,
            # Tool names can themselves be untrusted failed lookup input.
            "operation_sha256": hashlib.sha256(identity.encode()).hexdigest(),
            "component_kind": kind,
            "interface": "entered",
        }
        component = self._component_name(kind, identity)
        if component is not None:
            frame["operation" if kind == "tool" else "component"] = component
        if index is not None:
            state.frames.append(frame)
        else:
            state.omitted_frames += 1
        token = self._active.set((state, index))
        started = time.monotonic()
        span = get_current_span()
        if span.is_recording():
            span.set_attribute("pse.interface.correlation", state.correlation)
            span.set_attribute("pse.interface.loaded_revision", self.revision)
        result: ResultT | None = None
        try:
            result = await call_next(context)
            frame["interface"] = (
                "error_result"
                if isinstance(result, ToolResult) and result.is_error
                else "returned"
            )
            if isinstance(result, ToolResult):
                signals, references = _native(identity, result.structured_content)
                if signals:
                    frame["native"] = signals
                for reference in references:
                    if reference in state.references:
                        continue
                    if len(state.references) < MAX_REFERENCES:
                        state.references.append(reference)
                    else:
                        state.omitted_references += 1
        except asyncio.CancelledError:
            frame.update(
                interface="observer_detached",
                stage="interface",
                code="pse.capabilities.observer_detached",
            )
            raise
        except Exception as error:
            stage, code = _failure(error)
            frame.update(interface="failed", stage=stage, code=code)
            raise
        else:
            return result
        finally:
            frame["interface_elapsed_ms"] = round(
                (time.monotonic() - started) * 1000, 3
            )
            self._active.reset(token)
            if root:
                metadata = state.metadata()
                if result is not None and frame["interface"] in {
                    "returned",
                    "error_result",
                }:
                    # Preserve the exact native payload/content and existing metadata.
                    merged = dict(result.meta or {})
                    if META_KEY in merged:
                        # A result owner's key is never replaced on collision.
                        frame["metadata_attachment"] = "key_collision"
                    else:
                        merged[META_KEY] = metadata
                        result.meta = merged
                        if isinstance(result, ToolResult):
                            wire = result.to_mcp_result()
                            if isinstance(wire, CallToolResult):
                                wire.meta = merged
                # Logs deliberately exclude arguments, result values, native paths,
                # exception text and caller-supplied protocol request IDs.
                diagnostic = {
                    "code": "pse.capabilities.interface_" + str(frame["interface"]),
                    "correlation": state.correlation,
                    "loaded_revision": state.revision,
                    "request_id_sha256": state.request_id_sha256,
                    "calls": [
                        {key: value for key, value in call.items() if key != "native"}
                        for call in state.frames
                    ],
                    "native_reference_count": len(state.references),
                    "omitted_calls": state.omitted_frames,
                    "trace": state.trace,
                    "transport_delivery": "unobserved",
                    "native_transport": "server_unobserved_client_errors",
                }
                # A detached diagnostic sink cannot change native results.
                with suppress(OSError, ValueError):
                    print(
                        json.dumps(
                            diagnostic, ensure_ascii=True, separators=(",", ":")
                        ),
                        file=sys.stderr,
                    )


def install(
    server: FastMCP,
    *,
    revision: str,
    operation_names: frozenset[str] = frozenset(),
    resource_names: Mapping[str, str] | None = None,
    prompt_names: frozenset[str] = frozenset(),
) -> None:
    """Attach after composition so this middleware sees actual canonical dispatch."""
    server.add_middleware(
        Observation(
            revision,
            operation_names=operation_names,
            resource_names=resource_names,
            prompt_names=prompt_names,
        )
    )
