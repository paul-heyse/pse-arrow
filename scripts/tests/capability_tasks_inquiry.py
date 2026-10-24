# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Plan52o task inquiry: actual protocol, deterministic fixture, authentic FileStore.

Run in the isolated capability-inquiry group. This is not production task
adoption, a scientific study, or evidence that an OS process survives loss of
its owner. Each tiny tool body records fixture effects in the existing FileStore;
no second execution-state store or scheduler is invented. Output explicitly
separates protocol task status from native terminal/drain facts.
"""

from __future__ import annotations

import argparse
import asyncio
import importlib.metadata
import json
import logging
import os
import sys
import time
import uuid
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING

from fastmcp import Client, FastMCP
from fastmcp.client.transports import StdioTransport
from fastmcp.exceptions import ToolError
from fastmcp.server.context import (
    Context,  # noqa: TC002 -- FastMCP resolves injected annotations at registration.
)
from fastmcp.server.extensions import (
    MethodBinding,
    ServerExtension,
    read_client_extension_settings,
)
from fastmcp.utilities.tasks import TASKS_EXTENSION_ID, TaskConfig
from fastmcp_tasks import TasksExtension, call_tool_task
from fastmcp_tasks.client_models import (
    ClientGetTaskResult,
    GetTaskRequest,
    GetTaskRequestParams,
)
from fastmcp_tasks.models import CreateTaskResult, GetTaskParams, GetTaskResult
from mcp.shared.exceptions import MCPError
from mcp_types.version import MODERN_PROTOCOL_VERSIONS

from scripts import resource_store

if TYPE_CHECKING:
    from collections.abc import Awaitable, Sequence

    from fastmcp.server.extensions import ToolCallContinuation, ToolCallOutcome
    from mcp.server.context import ServerRequestContext
    from mcp_types import CallToolRequestParams
    from pydantic import BaseModel

ROOT = Path(__file__).resolve().parents[2]


def object_value(value: object) -> dict:
    if not isinstance(value, dict):
        raise TypeError("Expected actual structured result")
    return value


class Fixture:
    """Tiny producer using actual immutable invocation/result owners."""

    def __init__(self, root: Path, source: str = "fixture-source-v1") -> None:
        self.root = root
        self.source = source
        self.store = resource_store.FileStore(root / "registry")

    def begin(self, label: str) -> str:
        nonce = uuid.uuid4().hex
        self.store.create_invocation(
            {
                "nonce": nonce,
                "selected": [{"fixture": label}],
                "kind": "python",
                "source_revision": self.source,
            }
        )
        return nonce

    def finish(self, nonce: str, outcome: str) -> None:
        declaration = self.store.invocation_snapshot(nonce)["declaration"]
        self.store.freeze(
            nonce,
            {
                "selection": resource_store.digest(declaration["selected"]),
                "outcome": outcome,
                "fixture_cleanup_observed": True,
                "scientific_drain_established": False,
            },
            declaration=declaration,
        )

    def snapshots(self, label: str) -> list[dict]:
        directory = self.root / "registry" / "invocation-state"
        if not directory.exists():
            return []
        return [
            image
            for path in sorted(directory.glob("*.json"))
            if (image := self.store.invocation_snapshot(path.stem))["declaration"][
                "selected"
            ]
            == [{"fixture": label}]
        ]

    async def wait_started(self, label: str) -> dict:
        async with asyncio.timeout(3):
            while not (images := self.snapshots(label)):
                await asyncio.sleep(0.01)
        assert len(images) == 1
        return images[0]

    def release(self, label: str) -> None:
        (self.root / f"{label}.release").touch()


def standard_server(fixture: Fixture, *, child_process: bool = False) -> FastMCP:
    server = FastMCP("plan52o-task-fixture", mask_error_details=False)
    server.add_extension(TasksExtension(name="plan52o-inquiry", concurrency=2))

    async def effect(label: str, ctx: Context, *, gated: bool = False) -> dict:
        nonce = fixture.begin(label)
        try:
            if gated:
                async with asyncio.timeout(5):
                    while not (fixture.root / f"{label}.release").exists():
                        await asyncio.sleep(0.01)
            fixture.finish(nonce, "fixture-pass")
        except asyncio.CancelledError:
            # Deliberately retain a real scheduling window after protocol cancel.
            # This is fixture cleanup, never a claim of native scientific drain.
            await asyncio.sleep(0.06)
            fixture.finish(nonce, "fixture-cancelled")
            raise
        else:
            return {
                "invocation": nonce,
                "source": fixture.source,
                "task_id": ctx.task_id,
            }

    @server.tool(task=TaskConfig(mode="optional"))
    async def optional(label: str, ctx: Context) -> dict:
        return await effect(label, ctx)

    @server.tool(task=TaskConfig(mode="required"))
    async def required(label: str, ctx: Context) -> dict:
        return await effect(label, ctx)

    @server.tool(task=TaskConfig(mode="forbidden"))
    async def forbidden(label: str, ctx: Context) -> dict:
        return await effect(label, ctx)

    @server.tool(task=True)
    async def gated(label: str, ctx: Context) -> dict:
        return await effect(label, ctx, gated=True)

    @server.tool(task=True)
    async def tool_error(label: str) -> str:
        nonce = fixture.begin(label)
        fixture.finish(nonce, "fixture-failed")
        raise ToolError("deliberate fixture tool error")

    @server.tool
    async def foreground_wrapper(target: str, label: str) -> dict:
        result = await server.call_tool(target, {"label": label})
        return {"inner": result.structured_content}

    @server.tool(task=True)
    async def task_wrapper(target: str, label: str, ctx: Context) -> dict:
        result = await server.call_tool(target, {"label": label})
        return {"outer_task_id": ctx.task_id, "inner": result.structured_content}

    if child_process:

        @server.tool
        async def force_endpoint_loss() -> str:
            """Exit this disposable probe child, without Python cleanup."""
            os._exit(17)

    return server


async def get_task(client: Client, task_id: str) -> ClientGetTaskResult:
    return await client.session.send_request(
        GetTaskRequest(params=GetTaskRequestParams(task_id=task_id)),
        ClientGetTaskResult,
    )


async def expected_error(awaitable: Awaitable[object]) -> dict:
    try:
        result = await awaitable
    except (MCPError, ToolError, ValueError) as error:
        return {"type": type(error).__name__, "message": str(error)}
    if getattr(result, "is_error", False):
        return {"type": "tool-result-error", "content": str(result)}
    raise AssertionError(f"Expected refusal, got {result!r}")


async def standard_probe(root: Path) -> dict:
    fixture = Fixture(root)
    server = standard_server(fixture)
    report: dict = {}
    # Anchor models a still-running endpoint. A second client is a fresh session,
    # not a fresh process; the later STDIO probe establishes that distinction.
    async with Client(server, mode="auto", timeout=8) as first:
        assert first.protocol_version in MODERN_PROTOCOL_VERSIONS
        report["modern_protocol"] = first.protocol_version
        task = await call_tool_task(first, "optional", {"label": "modern-optional"})
        result = await task.result()
        assert object_value(result.structured_content)["task_id"] == task.task_id
        assert len(fixture.snapshots("modern-optional")) == 1
        report["optional_task"] = {
            "task_id": task.task_id,
            "result": result.structured_content,
        }

        required = await call_tool_task(first, "required", {"label": "modern-required"})
        await required.result()
        report["required_task"] = (await required.status()).status

        # A helper error is NOT a no-effect guarantee: forbidden runs inline first.
        report["forbidden_helper_error"] = await expected_error(
            call_tool_task(first, "forbidden", {"label": "forbidden-effect"})
        )
        assert len(fixture.snapshots("forbidden-effect")) == 1
        report["forbidden_effects"] = 1

        wrapper = await first.call_tool(
            "foreground_wrapper", {"target": "optional", "label": "inner-optional"}
        )
        assert object_value(wrapper.structured_content)["inner"]["task_id"] is None
        report["inner_optional_task_id"] = None
        report["inner_required_refusal"] = await expected_error(
            first.call_tool(
                "foreground_wrapper", {"target": "required", "label": "inner-required"}
            )
        )
        assert not fixture.snapshots("inner-required")
        outer = await call_tool_task(
            first, "task_wrapper", {"target": "optional", "label": "outer-task"}
        )
        outer_result = await outer.result()
        assert (
            object_value(outer_result.structured_content)["outer_task_id"]
            == outer.task_id
        )
        assert object_value(outer_result.structured_content)["inner"]["task_id"] is None
        report["outer_wrapper_task"] = outer_result.structured_content

        error_task = await call_tool_task(
            first, "tool_error", {"label": "native-failed"}, raise_on_error=False
        )
        error_result = await error_task.result()
        error_status = await error_task.status()
        assert error_status.status == "completed"
        assert error_result.is_error
        assert (
            fixture.snapshots("native-failed")[0]["terminal"]["outcome"]
            == "fixture-failed"
        )
        report["completed_tool_error"] = {
            "protocol_status": error_status.status,
            "tool_is_error": error_result.is_error,
            "native_fixture_outcome": "fixture-failed",
        }

        running = await call_tool_task(first, "gated", {"label": "reconnect"})
        native = await fixture.wait_started("reconnect")
        # Discard the create response at the application boundary: no resubmit.
        # Exact previously issued native identity recovers the original occurrence.
        lost_ack_nonce = native["declaration"]["nonce"]
        async with Client(server, mode="auto", timeout=8) as fresh:
            observed = await get_task(fresh, running.task_id)
            assert observed.status == "working"
            fixture.release("reconnect")
            await running.result()
            recovered = await get_task(fresh, running.task_id)
            assert recovered.status == "completed"
            assert len(fixture.snapshots("reconnect")) == 1
            assert (
                fixture.store.invocation_snapshot(lost_ack_nonce)["phase"] == "frozen"
            )
            report["fresh_session"] = {
                "before": observed.status,
                "after": recovered.status,
                "same_native_invocation": lost_ack_nonce,
                "effects": 1,
                "lost_ack_model": "create response discarded, no transport-fault injection",
            }

        cancel_task = await call_tool_task(first, "gated", {"label": "cancel"})
        await fixture.wait_started("cancel")
        cancel_ack = await cancel_task.cancel()
        after_ack = fixture.snapshots("cancel")[0]
        assert cancel_ack is None
        async with asyncio.timeout(3):
            while fixture.snapshots("cancel")[0]["phase"] != "frozen":
                await asyncio.sleep(0.01)
        cancelled = await cancel_task.status()
        assert cancelled.status == "cancelled"
        report["cancellation"] = {
            "ack": cancel_ack,
            "native_phase_at_ack": after_ack["phase"],
            "protocol_terminal": cancelled.status,
            "fixture_cleanup_final": fixture.snapshots("cancel")[0]["terminal"],
        }

        async with Client(server, mode="legacy", timeout=8) as legacy:
            legacy_result = await legacy.call_tool(
                "optional", {"label": "legacy-optional"}
            )
            assert object_value(legacy_result.structured_content)["task_id"] is None
            report["legacy_protocol"] = legacy.protocol_version
            report["legacy_required_refusal"] = await expected_error(
                legacy.call_tool("required", {"label": "legacy-required"})
            )
            assert not fixture.snapshots("legacy-required")
    return report


class NativeProjection(ServerExtension):
    """Probe public hooks, not an adopted implementation of the tasks protocol.

    No execution is scheduled here. Exact original FileStore identities are
    projected read-only. CreateTaskResult emission is checked over actual STDIO;
    this is not a full task protocol adapter or native execution recovery.
    """

    identifier = TASKS_EXTENSION_ID

    def __init__(self, fixture: Fixture, nonce: str) -> None:
        self.fixture = fixture
        self.nonce = nonce

    def methods(self) -> Sequence[MethodBinding]:
        return [
            MethodBinding(
                method="tasks/get",
                params_type=GetTaskParams,
                handler=self.read,
                protocol_versions=frozenset(MODERN_PROTOCOL_VERSIONS),
            )
        ]

    async def read(self, ctx: ServerRequestContext, params: GetTaskParams) -> BaseModel:
        if read_client_extension_settings(ctx, TASKS_EXTENSION_ID) is None:
            raise MCPError(code=-32021, message="Tasks capability not declared")
        image = self.fixture.store.invocation_snapshot(params.task_id)
        if image["declaration"]["source_revision"] != self.fixture.source:
            raise MCPError(
                code=-32602,
                message="Serving source differs from retained native source",
            )
        assert image["phase"] == "frozen"
        now = datetime.now(UTC).isoformat()
        return GetTaskResult(
            task_id=params.task_id,
            status="completed",
            created_at=now,
            last_updated_at=now,
            ttl_ms=None,
            result={
                "content": [],
                "structuredContent": {
                    "invocation": params.task_id,
                    "terminal_identity": image["terminal_identity"],
                    "native_fixture_outcome": image["terminal"]["outcome"],
                },
                "isError": False,
            },
        )

    async def intercept_tool_call(
        self,
        params: CallToolRequestParams,
        context: Context,
        call_next: ToolCallContinuation,
    ) -> ToolCallOutcome:
        if params.name != "native_claim":
            return await call_next()
        request = context.request_context
        if (
            request is None
            or request.protocol_version not in MODERN_PROTOCOL_VERSIONS
            or context.client_extension_settings(TASKS_EXTENSION_ID) is None
        ):
            raise MCPError(code=-32021, message="Modern tasks opt-in required")
        now = datetime.now(UTC).isoformat()
        return CreateTaskResult(
            task_id=self.nonce,
            status="working",
            created_at=now,
            last_updated_at=now,
            ttl_ms=None,
        )


def native_server(fixture: Fixture, nonce: str) -> FastMCP:
    server = FastMCP("plan52o-native-projection")
    server.add_extension(NativeProjection(fixture, nonce))

    @server.tool
    async def native_claim() -> str:
        raise AssertionError("Interceptor must handle this probe")

    return server


async def native_projection_probe(root: Path) -> dict:
    fixture = Fixture(root)
    nonce = fixture.begin("native-projection")
    fixture.finish(nonce, "fixture-failed")
    before = fixture.store.invocation_snapshot(nonce)
    async with Client(native_server(fixture, nonce), mode="auto", timeout=5) as client:
        observed = await get_task(client, nonce)
        assert observed.status == "completed"
        assert (
            object_value(observed.result)["structuredContent"]["native_fixture_outcome"]
            == "fixture-failed"
        )
        claim = await call_tool_task(client, "native_claim")
        assert claim.task_id == nonce
        claim_result = await claim.result()
        assert object_value(claim_result.structured_content)["invocation"] == nonce
    reopened = Fixture(root)
    async with Client(native_server(reopened, nonce), mode="auto", timeout=5) as fresh:
        recovered = await get_task(fresh, nonce)
        assert recovered.result == observed.result
    changed = Fixture(root, "fixture-source-v2")
    async with Client(native_server(changed, nonce), mode="auto", timeout=5) as fresh:
        source_refusal = await expected_error(get_task(fresh, nonce))
    async with Client(stdio(root, nonce), mode="auto", timeout=8) as wire:
        wire_claim = await call_tool_task(wire, "native_claim")
        wire_result = await wire_claim.result()
        assert object_value(wire_result.structured_content)["invocation"] == nonce
    assert fixture.store.invocation_snapshot(nonce) == before
    return {
        "native_identity": nonce,
        "protocol_read_status": observed.status,
        "fresh_server_recovered_original": True,
        "native_store_unchanged": True,
        "claim_production": {
            "in_memory": "accepted",
            "stdio": "accepted",
            "task_id": nonce,
        },
        "changed_source_refusal": source_refusal,
        "limits": "metadata fixture and public protocol projection; no scheduler, live recovery, cancel, or scientific drain proven",
    }


def stdio(root: Path, nonce: str | None = None) -> StdioTransport:
    serve_args = (
        ["--serve", str(root)]
        if nonce is None
        else ["--serve-native", str(root), nonce]
    )
    return StdioTransport(
        command=sys.executable,
        args=["-m", "scripts.tests.capability_tasks_inquiry", *serve_args],
        cwd=str(ROOT),
        keep_alive=False,
    )


async def endpoint_loss_probe(root: Path) -> dict:
    root.mkdir(parents=True)
    async with Client(stdio(root), mode="auto", timeout=8) as client:
        task = await call_tool_task(client, "optional", {"label": "endpoint-loss"})
        result = await task.result()
        old_id = task.task_id
        nonce = object_value(result.structured_content)["invocation"]
    # Entire STDIO process and its in-memory backend have now exited.
    async with Client(stdio(root), mode="auto", timeout=8) as fresh:
        error = await expected_error(get_task(fresh, old_id))
    fixture = Fixture(root)
    assert fixture.store.invocation_snapshot(nonce)["phase"] == "frozen"
    assert len(fixture.snapshots("endpoint-loss")) == 1
    return {
        "old_protocol_task": old_id,
        "fresh_endpoint_task_refusal": error,
        "retained_native_invocation": nonce,
        "native_phase": "frozen",
        "native_effects": 1,
        "limits": "completed fixture; no running-work continuity claim",
    }


async def forced_endpoint_loss_probe(root: Path) -> dict:
    root.mkdir(parents=True)
    fixture = Fixture(root)
    disconnect: dict | None = None
    async with Client(stdio(root), mode="auto", timeout=8) as client:
        running = await call_tool_task(client, "gated", {"label": "forced-loss"})
        image = await fixture.wait_started("forced-loss")
        task_id = running.task_id
        nonce = image["declaration"]["nonce"]
        try:
            await client.call_tool("force_endpoint_loss")
        except Exception as error:
            disconnect = {"type": type(error).__name__, "message": str(error)}
    assert disconnect is not None
    assert fixture.store.invocation_snapshot(nonce)["phase"] == "open"
    async with Client(stdio(root), mode="auto", timeout=8) as fresh:
        error = await expected_error(get_task(fresh, task_id))
    assert len(fixture.snapshots("forced-loss")) == 1
    return {
        "transport_disconnect": disconnect,
        "fresh_endpoint_task_refusal": error,
        "retained_native_invocation": nonce,
        "native_phase": "open",
        "native_effects": 1,
        "native_terminal_manufactured": False,
        "limits": "forced disposable STDIO child loss; FileStore preserves unresolved fixture evidence, not live execution recovery",
    }


async def main(output: Path) -> None:
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    standard = output / "standard"
    standard.mkdir()
    native = output / "native"
    native.mkdir()
    report: dict[str, object] = {
        "evidence": "Tested: actual FastMCP/MCP wire with deterministic fixture bodies and real FileStore records",
        "versions": {
            name: importlib.metadata.version(name)
            for name in ("fastmcp", "fastmcp-tasks", "mcp", "pydocket")
        },
        "native_projection": await native_projection_probe(native),
        "standard_memory": await standard_probe(standard),
        "endpoint_loss": await endpoint_loss_probe(output / "stdio"),
        "forced_endpoint_loss": await forced_endpoint_loss_probe(
            output / "forced-stdio"
        ),
    }
    report["elapsed_seconds"] = time.monotonic() - started
    report["failure_baseline"] = 0
    report["failures"] = 0
    (output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    logging.disable(logging.CRITICAL)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--serve", type=Path)
    parser.add_argument("--serve-native", nargs=2)
    args = parser.parse_args()
    if args.serve_native is not None:
        native_server(Fixture(Path(args.serve_native[0])), args.serve_native[1]).run(
            show_banner=False
        )
    elif args.serve is not None:
        standard_server(Fixture(args.serve), child_process=True).run(show_banner=False)
    elif args.output is not None:
        asyncio.run(main(args.output))
    else:
        parser.error("--output is required unless --serve is selected")
