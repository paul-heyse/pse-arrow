# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit OPT01/OPT02 inquiry, never imported by ordinary capability serving.

The controls exercise real Monty and canonical dispatch. ``--host`` runs a fresh
Codex against a test-only canonical server and preserves raw bidirectional wire
traffic. Native study observations remain separate from protocol/host evidence.
No production composition, task extension or scheduler is installed.
"""

from __future__ import annotations

import argparse
import asyncio
import contextlib
import hashlib
import json
import os
import socket
import subprocess
import sys
import threading
import time
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from fastmcp import Client, FastMCP
from fastmcp.client.transports import StdioTransport
from fastmcp.exceptions import ToolError
from fastmcp.server.extensions import (
    MethodBinding,
    ServerExtension,
    read_client_extension_settings,
)
from fastmcp.server.middleware import Middleware
from fastmcp.tools import Tool
from fastmcp.utilities.tasks import TASKS_EXTENSION_ID
from fastmcp_tasks import call_tool_task
from fastmcp_tasks.models import (
    CancelTaskResult,
    CreateTaskResult,
    GetTaskParams,
    GetTaskResult,
)
from mcp.shared.exceptions import MCPError
from mcp.types import ToolAnnotations
from mcp_types.version import MODERN_PROTOCOL_VERSIONS
from pydantic import (
    JsonValue,  # noqa: TC002 -- FunctionTool resolves return annotations at registration.
)

from scripts import codex_session
from scripts.capabilities import coherence
from scripts.tests import capability_composition_inquiry as composition
from scripts.tests.capability_codex_check import drain
from scripts.tests.capability_tasks_inquiry import get_task

if TYPE_CHECKING:
    from collections.abc import Mapping
    from typing import BinaryIO

    from fastmcp.server.context import Context
    from fastmcp.server.extensions import ToolCallContinuation, ToolCallOutcome
    from fastmcp.server.middleware import CallNext, MiddlewareContext
    from fastmcp.tools import ToolResult
    from mcp.server.context import ServerRequestContext
    from mcp_types import CallToolRequestParams

    import pse

ROOT = Path(__file__).resolve().parents[2]
SERVER = "pse-capabilities"
F01_INPUTS = "inquiry_comparison_inputs"
F01_COMPARE = "native_evidence_compare"
PREPARE = "python_extension_native_stage"
PREPARATION = {"request": {"stage": "prepare", "profile": "dev", "validated": True}}
BOOTSTRAP = frozenset(
    {
        "capabilities_domains",
        "capabilities_workflows",
        "capabilities_describe",
        "capabilities_select",
        "capabilities_reset",
        "capabilities_invoke_read",
        "list_resources",
        "read_resource",
        "list_prompts",
        "get_prompt",
    }
)


class InquiryHostGuard(Middleware):
    """The authorized host journey has an exact read closure, including wrappers."""

    def __init__(
        self, eligible: frozenset[str], *, prepare_native: bool = False
    ) -> None:
        self.eligible = (
            eligible
            | BOOTSTRAP
            | {
                composition.EXECUTE,
                composition.LIST,
                composition.SCHEMAS,
                composition.TAGS,
                composition.TYPED,
                "inquiry_native_study_projection",
            }
        )
        self.prepare_native = prepare_native
        self.preparations = 0

    async def on_call_tool(
        self,
        context: MiddlewareContext[CallToolRequestParams],
        call_next: CallNext[CallToolRequestParams, ToolResult],
    ) -> ToolResult:
        name = context.message.name
        arguments = context.message.arguments or {}
        if name in {"capabilities_invoke_action", PREPARE}:
            target = (
                arguments.get("capability")
                if name == "capabilities_invoke_action"
                else name
            )
            request = (
                arguments.get("arguments")
                if name == "capabilities_invoke_action"
                else arguments
            )
            if (
                not self.prepare_native
                or composition.ACTIVE.get() is not None
                or target != PREPARE
                or request != PREPARATION
            ):
                raise ToolError("opt01.only_explicit_native_prepare_setup_authorized")
            if name == PREPARE:
                self.preparations += 1
                if self.preparations != 1:
                    raise ToolError("opt01.native_prepare_must_not_be_replayed")
            return await call_next(context)
        if context.message.name not in self.eligible:
            raise ToolError("opt01.host_inquiry_exact_read_closure")
        return await call_next(context)


def mapping(value: object) -> dict[str, object]:
    if not isinstance(value, dict):
        raise TypeError("expected an object in retained inquiry evidence")
    return value


def sequence(value: object, message: str) -> list[object]:
    if not isinstance(value, list):
        raise TypeError(message)
    return value


def string(value: object, message: str) -> str:
    if not isinstance(value, str):
        raise TypeError(message)
    return value


def require(value: bool, message: str) -> None:
    if not value:
        raise AssertionError(message)


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + "\n")


def load_control(fixture: Path) -> dict[str, object]:
    source = fixture / "comparison-inputs.json"
    if not source.is_file():
        value = mapping(json.loads((fixture / "fixtures.json").read_text()))
        if (fixture / "native-study.json").is_file():
            value["_native_projection_expected"] = True
        return value
    value = mapping(json.loads(source.read_text()))
    require(
        "expected" not in value,
        "independent expected answer must have separate custody",
    )
    return {
        **value,
        "expected": mapping(
            json.loads((fixture / "comparison-expected.json").read_text())
        ),
    }


async def selected(client: Client) -> None:
    await client.call_tool(
        "capabilities_select",
        {
            "capabilities": sorted(
                composition.ELIGIBLE
                | {
                    composition.EXECUTE,
                    composition.LIST,
                    composition.SCHEMAS,
                    composition.TAGS,
                    composition.TYPED,
                }
            )
        },
    )


async def controls(output: Path) -> dict[str, object]:
    """Independent refusal/source/concurrency controls, not actual-host evidence."""
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    control = composition.fixtures(output / "fixtures")
    write(output / "fixtures.json", control)
    paths = sequence(control["outputs"], "fixture paths")
    require(
        isinstance(paths, list) and all(isinstance(p, str) for p in paths),
        "fixture paths",
    )
    runtime, sandbox = await composition.inquiry_server()
    cases: dict[str, object] = {}
    async with Client(runtime.server, timeout=15) as client:
        await selected(client)
        # Both calls must overlap before either reader is allowed to finish.
        entered: set[str] = set()
        both = asyncio.Event()

        async def inquiry_yield_reader(label: str) -> dict[str, str]:
            entered.add(label)
            if entered == {"limited", "independent"}:
                both.set()
            await asyncio.wait_for(both.wait(), 3)
            await asyncio.sleep(0)
            return {"label": label}

        runtime.server.add_tool(
            Tool.from_function(
                inquiry_yield_reader,
                annotations=ToolAnnotations(readOnlyHint=True),
            )
        )

        async def overlapping(label: str, maximum: int) -> dict[str, object]:
            policy = composition.Policy(
                eligible=frozenset({"inquiry_yield_reader"}),
                max_calls=maximum,
            )
            sandbox.next_policy = policy
            code = (
                f"first = await call_tool('inquiry_yield_reader', {{'label': {label!r}}})\n"
                "try:\n"
                f"    second = await call_tool('inquiry_yield_reader', {{'label': {label!r}}})\n"
                "except Exception:\n    second = 'refused'\n"
                "return [first, second]"
            )
            try:
                return mapping(
                    await composition.read_value(
                        client, composition.EXECUTE, {"code": code}
                    )
                )
            finally:
                sandbox.next_policy = None

        limited, independent = await asyncio.gather(
            overlapping("limited", 1),
            overlapping("independent", 2),
        )
        require(
            limited["complete"] is False and independent["complete"] is True,
            "concurrent policy completeness leaked",
        )
        require(
            limited["examined_dispatches"] == independent["examined_dispatches"] == 2,
            "concurrent callback counters leaked",
        )
        require(
            limited["derived"] == [{"label": "limited"}, "refused"],
            "limited request acquired a peer's policy",
        )
        require(
            independent["derived"]
            == [{"label": "independent"}, {"label": "independent"}],
            "independent request acquired a peer's refusal",
        )
        for label, answer in (("limited", limited), ("independent", independent)):
            for record in sequence(answer["observations"], "callback observations"):
                row = mapping(record)
                if row.get("outcome") == "observed":
                    structured = mapping(row["structured"])
                    require(
                        structured == {"label": label},
                        "callback provenance crossed requests",
                    )
        cases["concurrent_request_policy_isolation"] = {
            "limited": limited,
            "independent": independent,
        }

        # Use the production Sources/Interface/RestartMiddleware implementation
        # against copied real source bytes. Never mutate the shared checkout.
        source_root = output / "source-tree"
        for relative in runtime.interface.sources.sources:
            source = ROOT / relative
            if source.is_file():
                destination = source_root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(source.read_bytes())
        sources = coherence.Sources(source_root, runtime.interface.sources.sources)
        interface = coherence.Interface(sources, runtime.catalogue)
        runtime.server.add_middleware(coherence.RestartMiddleware(interface))
        target = "scripts/run_inspection.py"
        require(
            target in interface.operations["product_run_read"],
            "source fence omitted real reader dependency",
        )
        before = hashlib.sha256((source_root / target).read_bytes()).hexdigest()
        attempts = 0

        def change_selected_source() -> None:
            nonlocal attempts
            attempts += 1
            if attempts == 2:
                with (source_root / target).open("ab") as stream:
                    stream.write(b"\n# isolated inquiry source-change control\n")

        sandbox.next_policy = composition.Policy(before_dispatch=change_selected_source)
        code = composition.code_for([str(paths[0])])
        code = (
            "try:\n"
            + "\n".join("    " + line for line in code.splitlines()[:-1])
            + "\nexcept Exception:\n    pass\nreturn 'caught'"
        )
        answer = mapping(
            await composition.read_value(client, composition.EXECUTE, {"code": code})
        )
        sandbox.next_policy = None
        require(
            answer["complete"] is False and answer["examined_dispatches"] == 2,
            "real changed source became complete after caught refusal",
        )
        status = interface.status("product_run_read")
        require(
            status["restart_required"] is True
            and target
            in sequence(status["changed_serving_sources"], "changed serving sources"),
            "canonical source fence did not identify changed reader",
        )
        cases["changed_callback_source"] = {
            "answer": answer,
            "source_status": status,
            "before_sha256": before,
            "after_sha256": hashlib.sha256(
                (source_root / target).read_bytes()
            ).hexdigest(),
            "scope": "production fence on copied real source closure; shared checkout unchanged",
        }

    # Caught failure does not erase partiality, even after callback cleanup.
    drained = asyncio.Event()

    async def fails() -> str:
        try:
            raise ValueError("independent callback failure")
        finally:
            drained.set()

    sandbox.next_policy = composition.Policy()
    answer = await sandbox.run(
        "try:\n    await fails()\nexcept Exception:\n    pass\nreturn 'caught'",
        external_functions={"fails": fails},
    )
    require(
        answer["complete"] is False and drained.is_set(),
        "caught callback failure lost partiality/drain",
    )
    cases["caught_callback_failure"] = answer

    entered_cleanup, cleanup_finished = asyncio.Event(), asyncio.Event()

    async def cleanup_failure() -> None:
        entered_cleanup.set()
        try:
            await asyncio.Event().wait()
        finally:
            cleanup_finished.set()
            raise RuntimeError("independent cleanup failure")

    sandbox.next_policy = composition.Policy(deadline=0.05)
    try:
        await sandbox.run(
            "return await cleanup_failure()",
            external_functions={"cleanup_failure": cleanup_failure},
        )
    except (TimeoutError, RuntimeError, ToolError) as error:
        require(
            entered_cleanup.is_set() and cleanup_finished.is_set(),
            "cleanup was not attempted",
        )
        cases["deadline_cleanup_failure"] = {
            "outcome": "refused",
            "error_type": type(error).__name__,
            "cleanup_attempted": True,
            "successful_cleanup_claim": False,
        }
    else:
        raise AssertionError("cleanup failure produced a complete successful answer")
    callback_done = asyncio.Event()

    async def slow_cleanup() -> str:
        try:
            await asyncio.Event().wait()
        finally:
            await asyncio.sleep(0.08)
            callback_done.set()
        return "unreachable"

    cancellation_deadline = 0.03
    sandbox.next_policy = composition.Policy(deadline=cancellation_deadline)
    started = time.monotonic()
    try:
        await sandbox.run(
            "return await slow_cleanup()",
            external_functions={"slow_cleanup": slow_cleanup},
        )
    except TimeoutError:
        elapsed = time.monotonic() - started
        require(
            callback_done.is_set() and elapsed > cancellation_deadline,
            "cooperative cleanup did not retain its actual deadline overshoot",
        )
        cases["deadline_is_cancellation_not_hard_return_bound"] = {
            "configured_cancellation_seconds": cancellation_deadline,
            "observed_return_seconds": elapsed,
            "callback_cleanup_finished": True,
            "interpretation": "Original framework joins cooperative cleanup after cancellation; this is no hard return or detached native drain guarantee",
        }
    else:
        raise AssertionError("callback-inclusive cancellation deadline did not expire")
    sandbox.next_policy = None
    for label, code, fragment in (
        ("live_intermediate_memory_bound", "return [0] * 100000000", "memory"),
        ("sandbox_compute_bound", "while True:\n    pass", "time"),
    ):
        try:
            await sandbox.run(code)
        except Exception as error:
            require(
                fragment in str(error).lower(),
                "sandbox refused for an unexpected reason: " + str(error),
            )
            cases[label] = {
                "outcome": "refused",
                "error_type": type(error).__name__,
                "message": str(error),
            }
        else:
            raise AssertionError(label + " did not refuse")
    result: dict[str, object] = {
        "status": "passed",
        "baseline": "zero failures",
        "cases": cases,
        "scope": "in-process controls; not Codex, scientific execution or production adoption",
    }
    write(output / "result.json", result)
    return result


def validate_provenance(answer: dict[str, object], expected: list[object]) -> None:
    """A model summary cannot replace retained original callback observations."""
    observations = sequence(
        answer.get("observations"), "composition omitted recoverable observations"
    )
    native_rows = {}
    for value in observations:
        row = mapping(value)
        require(
            row.get("outcome") == "observed",
            "three-way comparison has a refused callback",
        )
        structured = mapping(row.get("structured"))
        require(
            row.get("sha256")
            == hashlib.sha256(composition.encoded(structured)).hexdigest(),
            "callback bytes disagree with retained provenance digest",
        )
        if row.get("capability") == "product_run_read":
            payload = mapping(mapping(structured.get("result")).get("payload"))
            require(
                isinstance(payload.get("terminal_identity"), str),
                "missing original terminal identity",
            )
            require(
                isinstance(payload.get("registry"), str), "missing original registry"
            )
            subjects = sequence(
                payload.get("subjects"), "unexpected selected native population"
            )
            require(
                isinstance(subjects, list) and len(subjects) == 1,
                "unexpected selected native population",
            )
            subject = mapping(subjects[0])
            fact = {
                "identity": subject.get("identity"),
                "outcome": subject.get("outcome"),
                "assurance": payload.get("assurance"),
            }
            identity = fact["identity"]
            require(isinstance(identity, str), "subject omitted its native identity")
            require(
                identity not in native_rows or native_rows[identity] == fact,
                "repeated native observations disagree",
            )
            native_rows[identity] = fact
    require(
        list(native_rows.values()) == expected,
        "composition's retained native facts differ from independent oracle",
    )
    require(
        answer.get("derived") == expected,
        "composition's derived answer changed native facts",
    )
    require(
        answer.get("complete") is False,
        "unreported population was promoted to complete",
    )


class StudyReader:
    """Read/cancel one exact acknowledged native identity through public APIs."""

    def __init__(self, config: dict[str, object]) -> None:
        import pse  # noqa: PLC0415 -- selected owner loads only at this boundary
        from pse.contracts.identities import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            StudyId,
        )
        from pse.contracts.values import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            SemanticId,
        )

        self.config = config
        self.runtime = pse.Runtime(
            pse.EngineSettings(
                memory_limit_bytes=16 << 30,
                threads=16,
                spill_dir=str(config["spill"]),
                max_spill_bytes=1 << 30,
                batch_size=32,
            ),
            substrate=str(config["state"]),
            database=str(config["database"]),
        )
        try:
            self.handle = self.runtime.study(
                StudyId(SemanticId.from_hex(str(config["study_id"])))
            )
        except BaseException:
            self.runtime.close()
            raise

    def snapshot(self) -> dict[str, object]:
        from pse import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            codec,
        )

        status = mapping(json.loads(codec.encode_json(self.handle.status())))
        result = self.handle.result()
        return {
            "study_id": self.handle.study_id.to_hex(),
            "native_status": status,
            "native_result": None
            if result is None
            else json.loads(codec.encode_json(result)),
            "native_drain": "not_established_by_protocol_projection",
        }

    def cancel(self) -> dict[str, object]:
        from pse import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            codec,
        )

        return mapping(json.loads(codec.encode_json(self.handle.cancel())))

    def close(self) -> None:
        self.runtime.close()


class StudyProjection(ServerExtension):
    """Test-only protocol view; no creation, scheduling, retry or result authority."""

    identifier = TASKS_EXTENSION_ID

    def __init__(self, reader: StudyReader) -> None:
        self.reader = reader
        self.identity = str(reader.config["study_id"])
        self.created = str(reader.config["created_at"])

    def methods(self) -> list[MethodBinding]:
        return [
            MethodBinding(
                method="tasks/get",
                params_type=GetTaskParams,
                handler=self.read,
                protocol_versions=frozenset(MODERN_PROTOCOL_VERSIONS),
            ),
            MethodBinding(
                method="tasks/cancel",
                params_type=GetTaskParams,
                handler=self.cancel,
                protocol_versions=frozenset(MODERN_PROTOCOL_VERSIONS),
            ),
        ]

    def authorized(self, context: ServerRequestContext, task_id: str) -> None:
        if task_id != self.identity:
            raise MCPError(
                code=-32602,
                message="Select the exact acknowledged native study identity",
            )
        if read_client_extension_settings(context, TASKS_EXTENSION_ID) is None:
            raise MCPError(code=-32021, message="Tasks capability not declared")

    async def read(
        self, ctx: ServerRequestContext, params: GetTaskParams
    ) -> GetTaskResult:
        self.authorized(ctx, params.task_id)
        snapshot = await asyncio.to_thread(self.reader.snapshot)
        terminal = snapshot["native_result"] is not None
        return GetTaskResult(
            task_id=self.identity,
            status="completed" if terminal else "working",
            created_at=self.created,
            last_updated_at=datetime.now(UTC).isoformat(),
            ttl_ms=None,
            result={"content": [], "structuredContent": snapshot, "isError": False}
            if terminal
            else None,
            status_message="Original native lifecycle; protocol completion does not certify scientific success or drain",
        )

    async def cancel(
        self, ctx: ServerRequestContext, params: GetTaskParams
    ) -> CancelTaskResult:
        self.authorized(ctx, params.task_id)
        original = await asyncio.to_thread(self.reader.cancel)
        destination = self.reader.config.get("cancel_receipt")
        if isinstance(destination, str):
            write(Path(destination), original)
        return CancelTaskResult()

    async def intercept_tool_call(
        self,
        params: CallToolRequestParams,
        context: Context,
        call_next: ToolCallContinuation,
    ) -> ToolCallOutcome:
        if params.name != "inquiry_native_study_projection":
            return await call_next()
        request = context.request_context
        if request is None or request.protocol_version not in MODERN_PROTOCOL_VERSIONS:
            raise MCPError(
                code=-32021,
                message="Modern tasks protocol required; use original native handle",
            )
        if context.client_extension_settings(TASKS_EXTENSION_ID) is None:
            raise MCPError(code=-32021, message="Tasks capability not declared")
        return CreateTaskResult(
            task_id=self.identity,
            status="working",
            created_at=self.created,
            last_updated_at=datetime.now(UTC).isoformat(),
            ttl_ms=None,
        )


def negotiation(wire: Path) -> dict[str, object]:
    requests = [
        mapping(json.loads(line))
        for line in (wire / "requests.jsonl").read_text().splitlines()
    ]
    responses = [
        mapping(json.loads(line))
        for line in (wire / "responses.jsonl").read_text().splitlines()
    ]
    initialize = [row for row in requests if row.get("method") == "initialize"]
    require(len(initialize) == 1, "expected exactly one captured MCP initialization")
    response = next(
        (
            row
            for row in responses
            if row.get("id") == initialize[0].get("id") and "result" in row
        ),
        None,
    )
    require(response is not None, "MCP initialization response missing")
    calls = [row for row in requests if row.get("method") == "tools/call"]
    task_calls = [
        row
        for row in calls
        if mapping(row.get("params")).get("name") == "inquiry_native_study_projection"
    ]
    return {
        "initialization_request": initialize[0],
        "initialization_response": response,
        "task_projection_requests": task_calls,
        "task_methods": [
            row for row in requests if str(row.get("method", "")).startswith("tasks/")
        ],
        "interpretation": "raw actual-host negotiation; extension initialization alone is not per-request task consent",
    }


def assess_native_host_negotiation(
    wire: Path, control: dict[str, object]
) -> dict[str, object]:
    observed = negotiation(wire)
    if control.get("_native_projection_expected") is True:
        task_calls = observed["task_projection_requests"]
        require(
            isinstance(task_calls, list) and len(task_calls) == 1,
            "host did not make exactly one authorized native projection request",
        )
        protocol = mapping(mapping(observed["initialization_response"])["result"])[
            "protocolVersion"
        ]
        if protocol not in MODERN_PROTOCOL_VERSIONS:
            require(
                not observed["task_methods"],
                "legacy host unexpectedly dispatched modern task methods",
            )
            observed["native_projection_acceptance"] = (
                "unsupported by actually negotiated legacy protocol; no native start or retry"
            )
        else:
            observed["native_projection_acceptance"] = (
                "modern protocol; inspect captured task consent/result independently"
            )
    return observed


def host_calls(path: Path) -> list[dict[str, object]]:
    """Capture a complete fresh host turn without accepting alternate tools."""
    started: dict[str, dict[str, object]] = {}
    completed: set[str] = set()
    calls = []
    thread = terminal = False
    total = 0
    for line in path.read_text().splitlines():
        total += len(line.encode())
        require(total <= 64 << 20, "host evidence exceeds parser bound")
        event = mapping(json.loads(line))
        kind = event.get("type")
        if kind == "thread.started":
            require(not thread, "host reused multiple threads")
            thread = True
        if kind == "turn.completed":
            terminal = True
        require(kind not in {"turn.failed", "error"}, "host turn failed")
        item = event.get("item")
        if not isinstance(item, dict) or item.get("type") in {
            "reasoning",
            "agent_message",
        }:
            continue
        require(
            item.get("type") == "mcp_tool_call" and item.get("server") == SERVER,
            "host used a tool outside the single inquiry endpoint",
        )
        identity = string(item.get("id"), "host call has no identity")
        if kind == "item.started":
            require(
                identity not in started and identity not in completed,
                "host duplicate start",
            )
            started[identity] = item
        elif kind == "item.completed":
            old = started.pop(identity, None)
            if old is None:
                raise AssertionError("host unmatched completion")
            require(
                identity not in completed,
                "host unmatched completion",
            )
            require(
                all(
                    old.get(key) == item.get(key)
                    for key in ("server", "tool", "arguments")
                ),
                "host changed call arguments",
            )
            calls.append(item)
            completed.add(identity)
    require(
        thread and terminal and not started,
        "host evidence is not a complete fresh turn",
    )
    return calls


def operation(call: dict[str, object]) -> str:
    name = call.get("tool")
    if name in {"capabilities_invoke_read", "capabilities_invoke_action"}:
        name = mapping(call.get("arguments")).get("capability")
    return string(name, "host call lacks an operation")


def structured(call: dict[str, object]) -> dict[str, object]:
    envelope = mapping(call.get("result"))
    return mapping(
        envelope.get("structured_content", envelope.get("structuredContent"))
    )


def comparison_document(value: dict[str, object]) -> dict[str, object]:
    payload = mapping(value.get("payload"))
    require(
        payload.get("outcome") == "observed",
        "original comparison owner did not return an observed document",
    )
    return mapping(payload.get("document"))


def comparison_essential(document: dict[str, object]) -> dict[str, object]:
    return {
        name: document[name]
        for name in (
            "cut",
            "baseline_sources",
            "candidate_sources",
            "baseline_denominator",
            "candidate_denominator",
            "summary",
        )
    }


def native_setup_observation(
    calls: list[dict[str, object]],
    comparisons: list[dict[str, object]],
    observed: list[dict[str, object]],
    preparation: dict[str, object],
) -> dict[str, object]:
    """Preserve an exact current or controlled-stale basis around one authorized setup."""
    request = mapping(preparation["arguments"])
    require(
        request.get("arguments") == PREPARATION,
        "host changed the authorized setup stage",
    )
    prepared = structured(preparation)
    require(
        prepared.get("exit_code") == 0
        and prepared.get("operation") == "prepare"
        and prepared.get("mode") == "native"
        and prepared.get("profile") == "dev"
        and prepared.get("validated") is True,
        "original preparation owner did not report the exact successful setup",
    )
    index = calls.index(preparation)
    before = [row for row in comparisons if calls.index(row) < index]
    after = [row for row in observed if calls.index(row) > index]
    require(
        len(before) == 1 and bool(after),
        "setup requires one original before observation and an observed after comparison",
    )
    basis = before[0]
    current = basis in observed
    if not current:
        refusal = mapping(structured(basis).get("payload"))
        require(
            refusal.get("code") == "pse.native.artifact_not_current",
            "setup followed an unrelated native refusal",
        )
    return {
        "before": structured(basis),
        "before_state": "observed_current" if current else "artifact_not_current",
        "original_preparation_owner": prepared,
        "after": structured(after[0]),
        "interpretation": "Explicit authorized setup outside read-only CodeMode; no scientific action or automatic replay",
    }


def assess_f01_host(output: Path, control: dict[str, object]) -> dict[str, object]:
    """Check native owner agreement and original cuts, with raw callback custody."""
    calls = host_calls(output / "events.jsonl")
    expected = mapping(control["expected"])
    essential = comparison_essential(expected)
    all_direct = [row for row in calls if operation(row) == F01_COMPARE]
    direct = [
        row
        for row in all_direct
        if mapping(structured(row).get("payload")).get("outcome") == "observed"
    ]
    require(bool(direct), "host omitted actual typed native composition")
    for row in direct:
        require(
            comparison_document(structured(row)) == expected,
            "host native comparison changed the expected original cut",
        )
    preparation_calls = [row for row in calls if operation(row) == PREPARE]
    setup = None
    if preparation_calls:
        require(len(preparation_calls) == 1, "native setup action was replayed")
        preparation = preparation_calls[0]
        setup = native_setup_observation(calls, all_direct, direct, preparation)
    requested_small = sequence(
        control.get("small_reads"),
        "authentic comparison has no supplied small-reader baseline",
    )
    require(
        isinstance(requested_small, list) and bool(requested_small),
        "authentic comparison has no supplied small-reader baseline",
    )
    for small in requested_small:
        selected_read = mapping(small)
        matches = [
            row
            for row in calls
            if operation(row) == selected_read["capability"]
            and (
                mapping(row.get("arguments")).get("arguments")
                if row.get("tool") == "capabilities_invoke_read"
                else row.get("arguments")
            )
            == selected_read["arguments"]
        ]
        require(bool(matches), "host omitted an exact original small-reader question")
    composed = []
    for row in calls:
        if operation(row) != composition.EXECUTE:
            continue
        answer = structured(row)
        if answer.get("derived") != essential:
            continue
        records = sequence(
            answer.get("observations"), "native composition omitted callback provenance"
        )
        typed_records = []
        input_records = []
        for value in records:
            record = mapping(value)
            require(
                record.get("outcome") == "observed",
                "native composition has a refused callback",
            )
            original = mapping(record.get("structured"))
            require(
                record.get("sha256")
                == hashlib.sha256(composition.encoded(original)).hexdigest(),
                "native composition callback provenance digest changed",
            )
            if record.get("capability") == F01_COMPARE:
                typed_records.append(comparison_document(original))
                require(
                    record.get("arguments") == control["comparison"],
                    "CodeMode changed the original native question",
                )
            if record.get("capability") == F01_INPUTS:
                input_records.append(original)
        require(
            typed_records == [expected] and len(input_records) == 1,
            "CodeMode has no recoverable original input and native result",
        )
        require(
            mapping(input_records[0]["inputs"])
            == {key: value for key, value in control.items() if key != "expected"},
            "CodeMode's original inputs changed",
        )
        composed.append(row)
    require(
        bool(composed),
        "host did not produce an independently checked native CodeMode answer",
    )
    write(output / "negotiation.json", negotiation(output / "wire"))
    sizes = {
        "small": sum(
            len(composition.encoded(row["result"]))
            for row in calls
            if operation(row) in {mapping(s)["capability"] for s in requested_small}
        ),
        "typed": sum(len(composition.encoded(row["result"])) for row in direct),
        "codemode": sum(len(composition.encoded(row["result"])) for row in composed),
    }
    return {
        "status": "passed",
        "baseline": "zero failures",
        "calls": len(calls),
        "observed_result_bytes": sizes,
        "scope": "actual fresh Codex and canonical native F01 owner over authentic supplied original evidence; no productivity or latency benchmark",
        "original_cut": expected["cut"],
        "callback_provenance": "full original input, request arguments and native response retained",
        "explicit_native_setup": setup,
    }


def proxy(output: Path, fixture: Path) -> int:
    """Transparent disposable STDIO endpoint capture, including initialization."""
    output.mkdir(parents=True, exist_ok=False)
    process = subprocess.Popen(
        [
            sys.executable,
            "-m",
            "scripts.tests.plan52_conditional_acceptance",
            "--serve",
            str(fixture),
        ],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=(output / "server-stderr.log").open("wb"),
        start_new_session=True,
    )
    assert process.stdin is not None
    assert process.stdout is not None

    def requests() -> None:
        with (output / "requests.jsonl").open("wb") as capture:
            try:
                for line in sys.stdin.buffer:
                    capture.write(line)
                    capture.flush()
                    process.stdin.write(line)
                    process.stdin.flush()
            except (BrokenPipeError, OSError):
                pass
            finally:
                with contextlib.suppress(OSError):
                    process.stdin.close()

    threading.Thread(target=requests, daemon=True).start()
    try:
        with (output / "responses.jsonl").open("wb") as capture:
            for line in process.stdout:
                capture.write(line)
                capture.flush()
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()
        return process.wait(timeout=20)
    finally:
        drain(process)


async def serve(fixture: Path) -> None:
    inputs = fixture / "comparison-inputs.json"
    additional = ()
    eligible = composition.ELIGIBLE
    if inputs.is_file():
        original = inputs.read_bytes()
        original_digest = hashlib.sha256(original).hexdigest()
        source = composition.OBJECT.validate_json(original)
        require(
            "expected" not in source,
            "independent expected answer cannot be exposed to the host",
        )

        def inquiry_comparison_inputs() -> dict[str, JsonValue]:
            """Read retained original inquiry inputs and source references; no build, store or scientific execution."""
            if inputs.read_bytes() != original:
                raise ToolError("opt01.original_inquiry_inputs_changed")
            return {"inputs": source, "input_sha256": original_digest}

        additional = (
            Tool.from_function(
                inquiry_comparison_inputs,
                annotations=ToolAnnotations(readOnlyHint=True, destructiveHint=False),
            ),
        )
        eligible |= {F01_INPUTS, F01_COMPARE}
    runtime, _sandbox = await composition.inquiry_server(
        eligible=eligible, additional_tools=additional
    )
    runtime.server.add_middleware(
        InquiryHostGuard(
            eligible, prepare_native=(fixture / "prepare-native-authorized").is_file()
        )
    )
    # This test server is built from the current canonical bindings. It has no
    # alternate scientific interpreter and no production extension registration.
    native_config = fixture / "native-study.json"
    reader = None
    if native_config.is_file():
        reader = StudyReader(mapping(json.loads(native_config.read_text())))
        runtime.server.add_extension(StudyProjection(reader))

        async def inquiry_native_study_projection() -> dict[str, JsonValue]:
            """Inquiry only: negotiated task view over the exact existing native study; never starts work."""
            raise AssertionError("task extension must intercept")

        runtime.server.add_tool(
            Tool.from_function(
                inquiry_native_study_projection,
                annotations=ToolAnnotations(readOnlyHint=True, destructiveHint=False),
            )
        )
    capture_directory = fixture / "private-child-diagnostics"
    diagnostic_capture = contextlib.ExitStack()
    if capture_directory.is_dir():
        from scripts.capabilities import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            native as native_binding,
        )

        original_run = native_binding.subprocess.run
        write(
            capture_directory / "endpoint-environment.json",
            {
                "keys": sorted(os.environ),
                "linked_library_path_present": bool(os.environ.get("LD_LIBRARY_PATH")),
                "host_allocation_present": bool(os.environ.get("PSE_HOST_ALLOCATION")),
                "native_operation_present": bool(
                    os.environ.get("PSE_NATIVE_OPERATION")
                ),
            },
        )

        def capture_worker(
            command: list[str],
            *,
            input: bytes,
            stdout: BinaryIO,
            stderr: BinaryIO,
            cwd: Path,
            env: Mapping[str, str],
            check: bool,
        ) -> subprocess.CompletedProcess[bytes]:
            completed = original_run(
                command,
                input=input,
                stdout=stdout,
                stderr=stderr,
                cwd=cwd,
                env=env,
                check=check,
            )
            if "scripts.capabilities.native_worker" in command:
                stderr.seek(0)
                raw = stderr.read((1 << 20) + 1)
                stderr_path = capture_directory / "native-worker-stderr.log"
                stderr_path.write_bytes(raw)
                stderr_path.chmod(0o600)
                write(
                    capture_directory / "worker.json",
                    {
                        "command": command,
                        "exit_code": completed.returncode,
                        "stderr_bytes": len(raw),
                        "environment_keys": sorted(env),
                    },
                )
            return completed

        diagnostic_capture.enter_context(
            patch.object(native_binding.subprocess, "run", side_effect=capture_worker)
        )
    try:
        await runtime.server.run_async(transport="stdio", show_banner=False)
    finally:
        diagnostic_capture.close()
        if reader is not None:
            reader.close()


def native_runtime(config: dict[str, object]) -> pse.Runtime:
    import pse  # noqa: PLC0415 -- selected owner loads only at this boundary

    return pse.Runtime(
        pse.EngineSettings(
            memory_limit_bytes=16 << 30,
            threads=16,
            spill_dir=str(config["spill"]),
            max_spill_bytes=1 << 30,
            batch_size=32,
        ),
        substrate=str(config["state"]),
        database=str(config["database"]),
    )


def native_fixture(output: Path) -> dict[str, object]:
    """Own one disposable service using the established fixture lifetime owner."""
    from scripts import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        surreal_server as server,
    )
    from scripts import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        surrealism_module,
    )
    from scripts.tests.telemetry_database_acceptance import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        initialize_canonical,
        owned_fixture,
        record_ownership,
    )

    initializer = ROOT / "target/debug/xtask"
    preflight = subprocess.run(
        [str(initializer), "canonical-init", "--help"],
        cwd=ROOT,
        capture_output=True,
        timeout=15,
        check=False,
    )
    if preflight.returncode != 0:
        raise RuntimeError(
            "Current target/debug/xtask lacks nonmutating canonical-init capability; no fixture provisioned"
        )
    output = output.resolve()
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    fixtures = output / "fixtures"
    fixtures.mkdir(mode=0o700)
    write(
        output / "initializer-preflight.json",
        {
            "command": [str(initializer), "canonical-init", "--help"],
            "exit_code": preflight.returncode,
            "effects": "none; CLI help only",
        },
    )
    result: dict[str, object] = {"status": "failed", "baseline": "zero failures"}
    try:
        with owned_fixture(fixtures, output / "retirement") as directory:
            state = directory / "live"
            with socket.socket() as listener:
                listener.bind(("127.0.0.1", 0))
                port = listener.getsockname()[1]
            minimum = (
                8
                * server.integer(surrealism_module.LIMITS["max_memory_bytes"])
                * server.integer(surrealism_module.LIMITS["max_pool_size"])
                // server.MIB
            )
            args = server.parser().parse_args(
                [
                    "setup",
                    "--state",
                    str(state),
                    "--port",
                    str(port),
                    "--no-resident",
                    "--interpretation",
                    server.SUBSTRATE_INTERPRETATION,
                    "--memory-mib",
                    str(minimum + 512),
                    "--server-memory-mib",
                    str(minimum),
                    "--native-workers",
                    "1",
                    "--native-worker-memory-mib",
                    "512",
                ]
            )
            server.setup(args)
            try:
                initialize_canonical(state)
                server.start(state, server.config_for(state))
                record_ownership(directory, server.unit_name(state))
                database = str(server.config_for(state)["database"])
                write(
                    output / "active-fixture.json",
                    {
                        "state": str(state),
                        "database": database,
                        "unit": server.unit_name(state),
                        "owner_pid": os.getpid(),
                    },
                )
                result = asyncio.run(native(output / "journey", state, database))
                controls = composition.fixtures(output / "host-reader-controls")
                write(output / "journey/fixtures.json", controls)
                host_result = host(
                    output / "codex-native-projection", output / "journey", 600
                )
                require(
                    host_result["status"] == "passed",
                    "fresh Codex native-projection inquiry failed",
                )
                result["actual_codex"] = host_result
                # The worker owner supplies physical settlement independently of
                # protocol/native cancellation acknowledgement and metadata.
                units = server.drain_context(state, database)
                write(
                    output / "native-drain.json",
                    {
                        "status": "passed",
                        "original_owner_units": units,
                        "state": str(state),
                        "database": database,
                    },
                )
                record_ownership(directory, server.unit_name(state))
            finally:
                server.stop(state, server.config_for(state))
        result["fixture_retirement"] = str(output / "retirement/passed.json")
    except BaseException as error:
        result.update(
            status="failed", error_type=type(error).__name__, error=str(error)
        )
        raise
    finally:
        write(output / "acceptance.json", result)
    return result


def start_native(config: dict[str, object]) -> tuple[pse.Runtime, pse.StudyHandle]:
    """Exercise the original public recorded-definition/start APIs, once."""
    import pse  # noqa: PLC0415 -- selected owner loads only at this boundary
    from pse.tests.study_fixtures import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        point,
    )
    from pse.tests.study_fixtures import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        request as study_request,
    )
    from pse.tests.test_studies import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        _package,
    )

    runtime = native_runtime(config)
    try:
        package, case = _package(
            runtime,
            source='package opt02 { def Root { param x:Scalar=2; annotation report x("constant"); } }',
        )
        local = package.admit_study(study_request(point(case, pse.SolveSettings(), 0)))
        definition = runtime.record_study(local)
        handle = runtime.start_study(definition)
    except BaseException:
        runtime.close()
        raise
    else:
        return runtime, handle


def projection_transport(fixture: Path, *, lost_start: bool = False) -> StdioTransport:
    return StdioTransport(
        command=sys.executable,
        args=[
            "-m",
            "scripts.tests.plan52_conditional_acceptance",
            "--serve-lost-start" if lost_start else "--serve",
            str(fixture),
        ],
        cwd=str(ROOT),
        env=dict(os.environ),
        keep_alive=False,
    )


async def lost_start_server(fixture: Path) -> None:
    config = mapping(json.loads((fixture / "connection.json").read_text()))
    server = FastMCP("opt02-disposable-lost-start-endpoint")

    @server.tool
    async def inquiry_start_then_lose_ack() -> str:
        """Inquiry only: admit exactly once, then lose this disposable endpoint before acknowledgement."""
        with (fixture / "oracle-start-entries.jsonl").open("a") as entries:
            entries.write(
                json.dumps(
                    {"endpoint_pid": os.getpid(), "phase": "native-start-entered"}
                )
                + "\n"
            )
            entries.flush()
            os.fsync(entries.fileno())
        _runtime, handle = await asyncio.to_thread(start_native, config)
        # Independent observer custody, never returned or used as a hidden client
        # recovery key. The harness reads this only to settle its owned fixture.
        write(
            fixture / "oracle-native-identity.json",
            {
                **config,
                "study_id": handle.study_id.to_hex(),
                "created_at": datetime.now(UTC).isoformat(),
            },
        )
        os._exit(17)

    await server.run_async(transport="stdio", show_banner=False)


async def native(output: Path, state: Path, database: str) -> dict[str, object]:
    """Queued/pre-native cancellation and real lost-ack controls; no retry."""
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    spill = output / "spill"
    spill.mkdir()
    config: dict[str, object] = {
        "state": str(state.resolve()),
        "database": database,
        "spill": str(spill),
        "cancel_receipt": str(output / "native-cancel-receipt.json"),
    }
    runtime, handle = await asyncio.to_thread(start_native, config)
    config.update(
        study_id=handle.study_id.to_hex(), created_at=datetime.now(UTC).isoformat()
    )
    write(output / "native-study.json", config)
    try:
        reader = StudyReader(config)
    except BaseException:
        try:
            handle.cancel()
            runtime.work()
        finally:
            runtime.close()
        raise
    cases: dict[str, object] = {}
    try:
        queued = await asyncio.to_thread(reader.snapshot)
        points = sequence(
            mapping(queued["native_status"])["points"],
            "unexpected native queued population",
        )
        require(
            isinstance(points, list) and len(points) == 1,
            "unexpected native queued population",
        )
        require(
            all(
                mapping(point)["attempt"] is None and not mapping(point)["settled"]
                for point in points
            ),
            "native work started before queued observation",
        )
        require(
            queued["native_result"] is None, "queued native study already has a result"
        )
        cases["queued"] = queued
        write(output / "native-observations.json", cases)
        async with Client(
            projection_transport(output), mode="auto", timeout=60
        ) as first:
            task = await call_tool_task(first, "inquiry_native_study_projection")
            require(
                task.task_id == str(config["study_id"]),
                "projection changed acknowledged native identity",
            )
            observed = await get_task(first, str(config["study_id"]))
            require(
                observed.status == "working", "queued task projection claimed terminal"
            )
        async with Client(
            projection_transport(output), mode="auto", timeout=60
        ) as fresh:
            reopened = await get_task(fresh, str(config["study_id"]))
            require(
                reopened.status == "working",
                "fresh endpoint lost original native queued status",
            )
            task = await call_tool_task(fresh, "inquiry_native_study_projection")
            await task.cancel()
        cases["fresh_endpoint_queued_recovery"] = {
            "original_study_id": config["study_id"],
            "status": reopened.status,
        }
        write(output / "native-observations.json", cases)
        cancellation = mapping(
            json.loads((output / "native-cancel-receipt.json").read_text())
        )
        require(
            mapping((await asyncio.to_thread(reader.snapshot))["native_status"])[
                "cancelled"
            ]
            is True,
            "task cancellation acknowledgement did not reach original native owner",
        )
        # Original worker authority finalizes the cancelled occurrences. This is
        # not a host task state update and does not dispatch a replacement study.
        actions = await asyncio.to_thread(runtime.work)
        terminal = await asyncio.to_thread(reader.snapshot)
        require(
            terminal["native_result"] is not None,
            "cancelled native queue did not finalize",
        )
        require(
            terminal["study_id"] == queued["study_id"],
            "recovery changed native identity",
        )
        async with Client(
            projection_transport(output), mode="auto", timeout=60
        ) as fresh:
            concluded = await get_task(fresh, str(config["study_id"]))
            require(
                concluded.status == "completed",
                "terminal task projection did not retain original result",
            )
            require(
                mapping(concluded.result)["structuredContent"] == terminal,
                "projection changed native result",
            )
        cases["pre_native_cancellation_and_terminal_recovery"] = {
            "native_cancel_ack": cancellation,
            "original_worker_actions": actions,
            "original_terminal": terminal,
            "protocol_status": concluded.status,
            "interpretation": "completed projection of native cancellation; not scientific success or proof of physical drain",
        }
        write(output / "native-observations.json", cases)
    finally:
        try:
            if handle.result() is None:
                handle.cancel()
                runtime.work()
            write(output / "cleanup-native-snapshot.json", reader.snapshot())
        finally:
            reader.close()
            runtime.close()

    lost = output / "lost-start-ack"
    lost.mkdir()
    write(
        lost / "connection.json",
        {key: config[key] for key in ("state", "database", "spill")},
    )
    failure = None
    try:
        async with Client(
            projection_transport(lost, lost_start=True), timeout=120
        ) as client:
            await client.call_tool("inquiry_start_then_lose_ack")
    except Exception as error:
        failure = {"type": type(error).__name__, "message": str(error)[:2000]}
    require(failure is not None, "lost endpoint unexpectedly acknowledged native start")
    require(
        (lost / "oracle-native-identity.json").is_file(),
        "endpoint failed before authentic native admission",
    )
    entries = [
        mapping(json.loads(line))
        for line in (lost / "oracle-start-entries.jsonl").read_text().splitlines()
    ]
    require(
        len(entries) == 1,
        "lost acknowledgement caused more than one observed native start entry",
    )
    # The client path stops at unknown, with zero repeated start calls. Only the
    # independent fixture owner gets the oracle identity for inspection/cleanup.
    client_outcome = {
        "effect": "unknown",
        "native_identity_received": False,
        "start_calls": len(entries),
        "automatic_retry": False,
        "owner_lookup_by_request": "missing from current public Runtime API",
        "transport_failure": failure,
    }
    write(lost / "client-outcome.json", client_outcome)
    oracle = StudyReader(
        mapping(json.loads((lost / "oracle-native-identity.json").read_text()))
    )
    try:
        original = await asyncio.to_thread(oracle.snapshot)
        cancel = await asyncio.to_thread(oracle.cancel)
        await asyncio.to_thread(oracle.runtime.work)
        settled = await asyncio.to_thread(oracle.snapshot)
        require(
            settled["native_result"] is not None,
            "independent lost-ack fixture did not settle",
        )
        cases["authentic_lost_start_ack"] = {
            "client": client_outcome,
            "independent_oracle": original,
            "owner_cleanup_cancel": cancel,
            "owner_cleanup_terminal": settled,
            "client_recovery_established": False,
        }
    finally:
        oracle.close()
    result: dict[str, object] = {
        "status": "passed",
        "baseline": "zero failures",
        "cases": cases,
        "native_started_control": "not_run: requires independent authentic native-entry owner",
        "scope": "public Runtime durable queued/cancelled studies and fresh STDIO projection; no production task adoption, solver-entry or database restart claim",
    }
    write(output / "native-result.json", result)
    return result


def assess_host(output: Path, control: dict[str, object]) -> dict[str, object]:
    """Assess retained raw host evidence; no additional model call or execution."""
    if "comparison" in control:
        return assess_f01_host(output, control)
    calls = host_calls(output / "events.jsonl")
    operations = {operation(row) for row in calls}
    require(
        {composition.EXECUTE, composition.TYPED, composition.LIST, composition.SCHEMAS}
        <= operations,
        "host did not exercise the requested three-way inquiry",
    )
    # Numerical sizes count observed serialized MCP results, never estimated
    # token counts. Keep full original results and raw protocol traffic.
    sizes = {"small": 0, "typed": 0, "codemode": 0}
    full_comparisons = 0
    for row in calls:
        name = operation(row)
        if name in composition.ELIGIBLE:
            sizes["small"] += len(composition.encoded(row.get("result")))
        elif name in {composition.EXECUTE, composition.TYPED}:
            category = "typed" if name == composition.TYPED else "codemode"
            sizes[category] += len(composition.encoded(row.get("result")))
            envelope = mapping(row.get("result"))
            structured = envelope.get(
                "structured_content", envelope.get("structuredContent")
            )
            if isinstance(structured, dict) and isinstance(
                structured.get("derived"), list
            ):
                require(
                    structured["derived"] == control["expected"],
                    "host typed/CodeMode facts differ from oracle",
                )
                if name == composition.EXECUTE:
                    validate_provenance(
                        structured,
                        sequence(control["expected"], "reader oracle population"),
                    )
                    full_comparisons += 1
    require(
        full_comparisons >= 1, "host omitted a complete recoverable CodeMode comparison"
    )
    observed_negotiation = assess_native_host_negotiation(output / "wire", control)
    write(output / "negotiation.json", observed_negotiation)
    return {
        "status": "passed",
        "baseline": "zero failures",
        "calls": len(calls),
        "observed_result_bytes": sizes,
        "scope": "actual fresh Codex control-reader inquiry; native F01 and scientific recovery separate",
    }


def host(
    output: Path, fixture: Path, timeout: int, *, prepare_native: bool = False
) -> dict[str, object]:
    output.mkdir(parents=True, exist_ok=False)
    control = load_control(fixture)
    prompt = (
        "Run this authorized inquiry using only pse-capabilities MCP tools; no shell, edits, actions, or native execution. "
        "This is an immutable-reader control, not an executed Rust suite. Read the bootstrap instructions. "
        "Select rust_test_reference_read, product_run_read, inquiry_composition_list, inquiry_composition_schemas, "
        "inquiry_compose_reads and inquiry_selected_results. Discover the exact composition tools and schemas. "
        "Compare these same three native-format reader controls in three ways: small original reader calls, "
        "inquiry_selected_results, and one useful Python CodeMode program that branches on unreported outcomes "
        "and returns rows in input order with exactly identity, outcome, assurance. Full callback provenance is automatic. "
        "Use the published schemas; product_run_read's native document is under result.payload. "
        "Do not state that unknown/unreported evidence passed. CodeMode accepts Python async statements and return. "
        "Explicitly call inquiry_compose_reads with a program trying capabilities_reset and catching its refusal; "
        "confirm this gives no complete examined result. End with your supported recommendation and limitations. "
        "Exact control output paths: " + json.dumps(control.get("outputs"))
    )
    if "comparison" in control:
        prompt = (
            "Run this authorized immutable evidence inquiry using only pse-capabilities MCP tools. No shell, edits, "
            "actions, builds, service startup or scientific execution. This uses genuine retained evidence with its original conditions. "
            "Read bootstrap instructions, describe/select inquiry_comparison_inputs, native_evidence_compare, "
            "inquiry_composition_list, inquiry_composition_schemas and inquiry_compose_reads. "
            "Use inquiry_comparison_inputs to obtain the original native request/snapshots/observations and small_reads. "
            "For each small_reads entry, describe/select its exact capability and call it with those exact arguments. "
            "Answer the stated question from those original small reads, preserving missing/partial sources. "
            "Then call actual native_evidence_compare with the exact comparison object. "
            "Finally discover exact composition tools/schemas and run a useful Python CodeMode program that obtains "
            "inquiry_comparison_inputs, calls native_evidence_compare with inputs.comparison, and returns exactly the native document's "
            "cut, baseline_sources, candidate_sources, baseline_denominator, candidate_denominator and summary fields. "
            "The native document is under payload.document; retain full provenance (automatic wrapper). "
            "Do not recreate pairing rules in Python, infer source freshness, or infer invocation membership from timing. "
            "Try capabilities_reset once inside another CodeMode program catching refusal; report its completeness truthfully. "
            "End with the supported comparison, limitations and whether typed/small reads or optional CodeMode is justified. "
            "No production adoption is authorized."
        )
        if prepare_native:
            (fixture / "prepare-native-authorized").touch(exist_ok=True)
            prompt = prompt.replace(
                "No shell, edits, actions, builds, service startup or scientific execution.",
                "No shell, edits, service startup or scientific execution. One exact native preparation setup action is explicitly authorized below.",
            )
            prompt = prompt.replace(
                "Then call actual native_evidence_compare with the exact comparison object.",
                "First call native_evidence_compare with the exact comparison object and retain its original before-preparation observation. It may already be current, or may report the exact artifact_not_current currency refusal; do not invent staleness. "
                "Then describe/select python_extension_native_stage and invoke capabilities_invoke_action EXACTLY ONCE with "
                "capability=python_extension_native_stage, arguments={request:{stage:prepare,profile:dev,validated:true}}. "
                "This explicitly authorized setup is outside CodeMode; it may prepare/install the native extension for this exact selected environment. "
                "Wait for its original owner result, then call native_evidence_compare again with the exact original comparison object. "
                "Do not replay setup or scientific work. Retain the actual before-current or controlled-stale basis, original preparation and after-current native comparison evidence.",
            )
    diagnostic = (fixture / "private-child-diagnostics").is_dir()
    if diagnostic:
        prompt = (
            "Perform one authorized immutable-read transport diagnostic using only pse-capabilities MCP tools. "
            "No shell, edits, actions, builds or service startup. Describe/select inquiry_comparison_inputs and "
            "native_evidence_compare. Read inquiry_comparison_inputs and invoke native_evidence_compare EXACTLY ONCE "
            "with its exact comparison object. Report the actual result/refusal, preserving outcome uncertainty. "
            "Do not retry the comparison or invoke CodeMode, further native tools or any action."
        )
    if (fixture / "native-study.json").is_file():
        prompt += (
            " Also invoke inquiry_native_study_projection exactly once to test real task negotiation over an already "
            "acknowledged native identity. If unsupported, report the refusal and use the original native handle; "
            "do not retry initiation, cancel, start or replay any work. If tasks are supported, inspect its original result."
        )
    command, description = codex_session.session_description(
        state=None,
        database=False,
        rust=False,
        python=False,
        require_database=False,
        require_rust=False,
        arguments=[
            "-c",
            "mcp_servers.pse-capabilities.args="
            + json.dumps(
                [
                    "-m",
                    "scripts.tests.plan52_conditional_acceptance",
                    "--proxy",
                    str(output / "wire"),
                    "--fixture",
                    str(fixture),
                ]
            ),
            "-a",
            "never",
            "--disable",
            "shell_tool",
            "exec",
            "--json",
            "--ephemeral",
            "--sandbox",
            "read-only",
            "--color",
            "never",
            "-",
        ],
    )
    if prepare_native:
        position = command.index("-a")
        command[position:position] = [
            "-c",
            'mcp_servers.pse-capabilities.tools.capabilities_invoke_action.approval_mode="approve"',
        ]
    if (fixture / "native-study.json").is_file() or "comparison" in control:
        # A disposable native projection imports the linked extension directly;
        # ordinary canonical serving stays source-only. Use the established
        # native environment owner instead of recording secret-bearing env.
        position = command.index("-a")
        command[position:position] = [
            "-c",
            "mcp_servers.pse-capabilities.command="
            + json.dumps(str(ROOT / "scripts/pse-env")),
            "-c",
            "mcp_servers.pse-capabilities.args="
            + json.dumps(
                [
                    "--native",
                    "--",
                    str(ROOT / ".venv/bin/python"),
                    "-m",
                    "scripts.tests.plan52_conditional_acceptance",
                    "--proxy",
                    str(output / "wire"),
                    "--fixture",
                    str(fixture),
                ]
            ),
        ]
        description["inquiry_endpoint_environment"] = (
            "Explicit prepared native attachment owned by scripts/pse-env --native; ordinary production endpoint unchanged"
        )
    write(output / "session.json", {"command": command, "description": description})
    (output / "prompt.txt").write_text(prompt)
    outcome: dict[str, object] = {"status": "failed", "baseline": "zero failures"}
    process = None
    try:
        with (
            (output / "events.jsonl").open("wb") as events,
            (output / "stderr.log").open("wb") as errors,
        ):
            process = subprocess.Popen(
                command,
                cwd=ROOT,
                stdin=subprocess.PIPE,
                stdout=events,
                stderr=errors,
                start_new_session=True,
            )
            try:
                process.communicate(prompt.encode(), timeout=timeout)
            except subprocess.TimeoutExpired:
                drain(process)
                raise TimeoutError(
                    "fresh host inquiry expired; retain evidence before any replay"
                ) from None
        require(process.returncode == 0, "host process exited unsuccessfully")
        if diagnostic:
            calls = host_calls(output / "events.jsonl")
            require(
                sum(operation(call) == F01_COMPARE for call in calls) == 1,
                "diagnostic host did not perform exactly one immutable native comparison",
            )
            outcome.update(
                status="passed",
                scope="actual Codex fixed-child transport diagnostic only; no native comparison acceptance",
            )
        else:
            outcome.update(assess_host(output, control))
    except (
        OSError,
        ValueError,
        AssertionError,
        TimeoutError,
        subprocess.SubprocessError,
    ) as error:
        outcome["error"] = str(error)
    finally:
        if process is not None:
            drain(process)
        write(output / "outcome.json", outcome)
    return outcome


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--controls", type=Path)
    parser.add_argument("--host", type=Path)
    parser.add_argument("--assess-host", type=Path)
    parser.add_argument("--fixture", type=Path)
    parser.add_argument("--proxy", type=Path)
    parser.add_argument("--serve", type=Path)
    parser.add_argument("--serve-lost-start", type=Path)
    parser.add_argument("--native", type=Path)
    parser.add_argument("--native-fixture", type=Path)
    parser.add_argument("--state", type=Path)
    parser.add_argument("--database")
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--prepare-native", action="store_true")
    args = parser.parse_args()
    if args.serve is not None:
        asyncio.run(serve(args.serve))
        return 0
    if args.serve_lost_start is not None:
        asyncio.run(lost_start_server(args.serve_lost_start))
        return 0
    if args.proxy is not None and args.fixture is not None:
        return proxy(args.proxy, args.fixture)
    if args.controls is not None:
        result = asyncio.run(controls(args.controls))
    elif args.assess_host is not None and args.fixture is not None:
        control = load_control(args.fixture)
        result = assess_host(args.assess_host, control)
        write(args.assess_host / "reassessed-outcome.json", result)
    elif args.host is not None and args.fixture is not None:
        result = host(
            args.host, args.fixture, args.timeout, prepare_native=args.prepare_native
        )
    elif (
        args.native is not None and args.state is not None and args.database is not None
    ):
        result = asyncio.run(native(args.native, args.state, args.database))
    elif args.native_fixture is not None:
        result = native_fixture(args.native_fixture)
    else:
        parser.error("select --controls or --host with --fixture")
    print(json.dumps(result, sort_keys=True))
    return 0 if result["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
