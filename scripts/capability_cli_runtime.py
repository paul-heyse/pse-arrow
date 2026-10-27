# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent client runtime: never import the serving-source package for help."""

from __future__ import annotations

import asyncio
import json
import os
import stat
import sys
from contextlib import asynccontextmanager
from pathlib import Path
from typing import TYPE_CHECKING

from pydantic import BaseModel, ConfigDict, Field, JsonValue, TypeAdapter

JSON_VALUE = TypeAdapter(JsonValue)
JSON_REQUEST = TypeAdapter(dict[str, JsonValue])

if TYPE_CHECKING:
    from collections.abc import AsyncGenerator, Mapping

    from fastmcp import Client
    from fastmcp.client.client import CallToolResult


class ClientRefusedError(ValueError):
    """A client contract/lifetime refusal before operation effects."""


class SessionRequest(BaseModel):
    model_config = ConfigDict(strict=True, extra="forbid")
    id: str = Field(min_length=1, max_length=128)
    capability: str
    arguments: dict[str, JsonValue]


def checkout_root() -> Path:
    return Path(__file__).resolve().parents[1]


def result_document(result: CallToolResult) -> dict[str, object]:
    """Preserve the meaningful MCP answer, independently of pretty rendering."""
    return {
        "is_error": result.is_error,
        "structured_content": result.structured_content,
        "content": [
            block.model_dump(mode="json", by_alias=True) for block in result.content
        ],
        "meta": result.meta,
    }


def answer(result: CallToolResult) -> object:
    value = result.structured_content
    if isinstance(value, dict) and set(value) == {"result"}:
        return value["result"]
    return value


def connection(root: Path) -> Client:
    """Import client libraries only when a connection is actually requested."""
    # Offline help must not load client or serving libraries.
    from fastmcp import Client  # noqa: PLC0415
    from fastmcp.client.transports import StdioTransport  # noqa: PLC0415

    return Client(
        StdioTransport(
            command=str(root / ".venv/bin/python"),
            args=["-m", "scripts.capabilities"],
            cwd=str(root),
            env=os.environ.copy(),
            keep_alive=False,
        ),
        timeout=1800,
    )


async def drain_connection(client: Client) -> None:
    """Settle the endpoint before the STDIO SDK begins its bounded disconnect."""
    task = asyncio.create_task(client.call_tool("capabilities_drain", {}, raise_on_error=False))
    cancellation: asyncio.CancelledError | None = None
    while not task.done():
        try:
            await asyncio.shield(task)
        except asyncio.CancelledError as error:
            cancellation = error
        except BaseException:
            # Inspect the original result below so prior observer cancellation
            # and the actual drain failure both survive settlement.
            break
    try:
        result = task.result()
        if result.is_error or answer(result) != {"state": "drained", "native_admission": "closed"}:
            raise ClientRefusedError("Endpoint native drain was not acknowledged; physical settlement is unestablished.")
    except BaseException as error:
        if cancellation is not None:
            raise cancellation from error
        raise
    if cancellation is not None:
        raise cancellation


@asynccontextmanager
async def owned_connection(root: Path) -> AsyncGenerator[Client]:
    async with connection(root) as client:
        original: BaseException | None = None
        try:
            yield client
        except BaseException as error:
            original = error
        try:
            await drain_connection(client)
        except BaseException as error:
            if original is not None:
                raise BaseExceptionGroup("Foreground operation and native drain failed", [original, error]) from None
            raise
        if original is not None:
            raise original


async def invoke_on(
    client: Client,
    capability: str,
    arguments: dict[str, JsonValue],
    expected: Mapping[str, object],
    *,
    retained: bool = False,
) -> dict[str, object]:
    """Compatibility, selection and invocation share this exact Client lifetime."""
    lifetime = expected.get("lifetime")
    if lifetime != "single" and not (retained and lifetime == "endpoint"):
        raise ClientRefusedError(
            "This operation needs its retained endpoint/native route; fresh CLI invocation is unavailable."
        )
    description = await client.call_tool(
        "capabilities_describe",
        {"capability": capability, "include_schema": True},
        raise_on_error=False,
    )
    described = answer(description)
    if description.is_error or not isinstance(described, dict):
        return result_document(description)
    interface = described.get("interface")
    if not isinstance(interface, dict) or interface.get(
        "operation_wire_revision"
    ) != expected.get("wire_revision"):
        raise ClientRefusedError(
            "Selected operation contract changed; regenerate the capability CLI before effects."
        )
    if interface.get("restart_required"):
        raise ClientRefusedError(
            "The loaded operation contract changed; reconnect before invoking this operation."
        )
    selection = await client.call_tool(
        "capabilities_select", {"capabilities": [capability]}, raise_on_error=False
    )
    if selection.is_error:
        return result_document(selection)
    invoker = (
        "capabilities_invoke_read"
        if expected.get("read_only") is True
        else "capabilities_invoke_action"
    )
    result = await client.call_tool(
        invoker,
        {"capability": capability, "arguments": arguments},
        raise_on_error=False,
    )
    return result_document(result)


async def invoke(
    capability: str, arguments: dict[str, JsonValue], expected: Mapping[str, object]
) -> dict[str, object]:
    async with owned_connection(checkout_root()) as client:
        return await invoke_on(client, capability, arguments, expected)


def emit(value: object) -> None:
    print(json.dumps(value, ensure_ascii=False, allow_nan=False), flush=True)


def call(
    capability: str, arguments: dict[str, JsonValue], expected: Mapping[str, object]
) -> None:
    result = asyncio.run(invoke(capability, arguments, expected))
    emit(result)
    if result["is_error"]:
        raise SystemExit(1)


def _unique_object(pairs: list[tuple[str, JsonValue]]) -> dict[str, JsonValue]:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ClientRefusedError(
                "Duplicate JSON object keys are refused; supplied values are redacted."
            )
        result[key] = value
    return result


def parse_value(raw: str) -> JsonValue:
    value = json.loads(raw, object_pairs_hook=_unique_object)
    json.dumps(value, allow_nan=False)
    return JSON_VALUE.validate_python(value, strict=True)


def parse_request(raw: str) -> dict[str, JsonValue]:
    return JSON_REQUEST.validate_python(parse_value(raw), strict=True)


def diagnose() -> dict[str, object]:
    root = checkout_root()
    return {
        "checkout": str(root),
        "interpreter": str(root / ".venv/bin/python"),
        "interpreter_available": (root / ".venv/bin/python").is_file(),
        "generated_cli_available": (
            root / "scripts/generated/capabilities/cli.py"
        ).is_file(),
        "endpoint": "python -m scripts.capabilities",
        "connection": "not_probed",
        "routes": {
            "stale_attachment": "Use a fresh capability CLI connection; refresh the host attachment separately.",
            "startup_failed": "Use the original owner commands and scripts/pse-env --explain; the CLI cannot repair a broken endpoint.",
            "native_unavailable": "Describe readiness and explicitly prepare only the original prerequisite owner.",
            "acknowledgement_lost": "Read the original invocation/result/reconciliation owner. Do not replay the effect.",
        },
    }


async def probe(capability: str | None = None) -> dict[str, object]:
    """Explicit bounded fresh handshake and passive readiness; never prepares or repairs."""
    report = diagnose()
    report["stage"] = "configuration"
    if not report["interpreter_available"]:
        report.update(
            connection="unavailable", classification="missing_checkout_interpreter"
        )
        return report
    try:
        async with asyncio.timeout(15):
            report["stage"] = "endpoint_initialization"
            async with owned_connection(checkout_root()) as client:
                report["stage"] = "bootstrap"
                result = await client.call_tool(
                    "capabilities_domains", {}, raise_on_error=False
                )
                if result.is_error:
                    report.update(
                        connection="refused", original_result=result_document(result)
                    )
                    return report
                value = answer(result)
                interface = value.get("interface") if isinstance(value, dict) else None
                report.update(
                    connection="connected",
                    interface=interface,
                    classification="stale_loaded_interface"
                    if isinstance(interface, dict) and interface.get("restart_required")
                    else "fresh_connection",
                )
                if capability is not None:
                    report["stage"] = "selected_description"
                    description = await client.call_tool(
                        "capabilities_describe",
                        {"capability": capability},
                        raise_on_error=False,
                    )
                    if description.is_error:
                        report["original_result"] = result_document(description)
                        return report
                    detail = answer(description)
                    report["selected_capability"] = (
                        detail.get("capability") if isinstance(detail, dict) else None
                    )
                    selection = await client.call_tool(
                        "capabilities_select",
                        {"capabilities": ["capability_readiness"]},
                        raise_on_error=False,
                    )
                    if selection.is_error:
                        report["original_result"] = result_document(selection)
                        return report
                    report["stage"] = "passive_readiness"
                    ready = await client.call_tool(
                        "capabilities_invoke_read",
                        {
                            "capability": "capability_readiness",
                            "arguments": {"capability": capability},
                        },
                        raise_on_error=False,
                    )
                    report["readiness"] = result_document(ready)
                    report["readiness_interpretation"] = (
                        "Original passive receipt/requirement observations; native import, service availability and operation eligibility may remain unknown."
                    )
    except Exception as error:
        report.update(
            connection="unavailable",
            error_type=type(error).__name__,
            classification="endpoint_initialization_or_read_failed",
        )
    report["effect"] = (
        "One fresh read-only probe; no setup, preparation, action replay or repair of the host attachment."
    )
    return report


def failure(error: Exception) -> int:
    emit(
        {
            "is_error": True,
            "code": "pse.capability_cli.refused"
            if isinstance(error, ClientRefusedError)
            else "pse.capability_cli.connection_failed",
            "message": str(error)
            if isinstance(error, ClientRefusedError)
            else "Connection or input failed; use independent launch diagnosis. Supplied values are redacted.",
            "error_type": type(error).__name__,
            "effect": "No replay or rollback is inferred; inspect original owner references after uncertain delivery.",
        }
    )
    return 1


async def session(operations: Mapping[str, Mapping[str, object]]) -> None:
    """One foreground endpoint; explicit JSONL requests, no detached/durable preview promise."""
    reader = asyncio.StreamReader(limit=16 << 20)
    transport = None
    regular = stat.S_ISREG(os.fstat(sys.stdin.fileno()).st_mode)
    if not regular:
        loop = asyncio.get_running_loop()
        transport, _ = await loop.connect_read_pipe(
            lambda: asyncio.StreamReaderProtocol(reader), sys.stdin
        )
    try:
        async with owned_connection(checkout_root()) as client:
            emit(
                {
                    "session": "connected",
                    "lifetime": "foreground endpoint only",
                    "recovery": "Process death loses endpoint previews. Original native work/receipts have their own custody; never infer an unapplied effect.",
                }
            )
            while True:
                line = (
                    await asyncio.to_thread(sys.stdin.buffer.readline, (16 << 20) + 1)
                    if regular
                    else await reader.readline()
                )
                if not line:
                    break
                if len(line) > 16 << 20:
                    raise ClientRefusedError(
                        "JSONL request exceeds the existing finite native input extent."
                    )
                request_id: str | None = None
                try:
                    request = SessionRequest.model_validate(
                        parse_request(line.decode("utf-8"))
                    )
                    request_id = request.id
                    expected = operations.get(request.capability)
                    if expected is None:
                        raise ClientRefusedError(
                            "Unknown generated operation; describe/regenerate before effects."
                        )
                    result = await invoke_on(
                        client,
                        request.capability,
                        request.arguments,
                        expected,
                        retained=True,
                    )
                    emit({"id": request_id, "result": result})
                except ValueError as error:
                    emit(
                        {
                            "id": request_id,
                            "is_error": True,
                            "code": "pse.capability_cli.session_input_refused",
                            "message": str(error)
                            if isinstance(error, ClientRefusedError)
                            else "Invalid JSONL request; supplied values redacted.",
                        }
                    )
    finally:
        if transport is not None:
            transport.close()
