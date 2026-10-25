# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""OPT01 feasibility, using real Monty and original invocation readers.

Run with the locked capability-inquiry group. This module is never imported by
ordinary capability serving. Its small persisted fixtures are reader controls,
not executed Rust tests or scientific qualification.
"""

from __future__ import annotations

import argparse
import asyncio
import hashlib
import importlib.metadata
import json
import logging
import sys
import tempfile
import uuid
from contextvars import ContextVar
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Annotated
from unittest.mock import patch

from fastmcp import Client
from fastmcp.exceptions import ToolError
from fastmcp.experimental.transforms.code_mode import (
    CodeMode,
    DiscoveryToolFactory,
    GetSchemas,
    GetTags,
    GetToolCatalog,
    ListTools,
    MontySandboxProvider,
)
from fastmcp.server.context import (
    Context,  # noqa: TC002 -- framework inspects annotations.
)
from fastmcp.server.middleware import Middleware
from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import BaseModel, Field, JsonValue, TypeAdapter

if TYPE_CHECKING:
    from collections.abc import Awaitable, Callable, Sequence

    from fastmcp.server.middleware import CallNext, MiddlewareContext
    from fastmcp.tools import ToolResult
    from mcp.types import CallToolRequestParams

    from scripts.capabilities.specialists import Specialists

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from scripts import resource_store, run_inspection  # noqa: E402
from scripts.capabilities import server as canonical  # noqa: E402
from scripts.capabilities.model import Capability  # noqa: E402
from scripts.capabilities.rust_references import read as reference_read  # noqa: E402
from scripts.capabilities.rust_test_contracts import InvocationReference  # noqa: E402

JSON = TypeAdapter(JsonValue)
OBJECT = TypeAdapter(dict[str, JsonValue])
ELIGIBLE = frozenset({"rust_test_reference_read", "product_run_read"})
EXECUTE = "inquiry_compose_reads"
LIST = "inquiry_composition_list"
SCHEMAS = "inquiry_composition_schemas"
TAGS = "inquiry_composition_tags"
TYPED = "inquiry_selected_results"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def encoded(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, allow_nan=False).encode()


@dataclass
class Policy:
    eligible: frozenset[str] = ELIGIBLE
    max_calls: int = 8
    max_bytes: int = 1 << 20
    max_final_bytes: int = 64 << 10
    deadline: float = 5
    attempts: int = 0
    transferred: int = 0
    incomplete: bool = False
    records: list[dict[str, JsonValue]] = field(default_factory=list)
    before_dispatch: Callable[[], None] | None = None


ACTIVE: ContextVar[Policy | None] = ContextVar("opt01-read-policy", default=None)


class CompositionAnswer(BaseModel):
    derived: JsonValue
    observations: list[dict[str, JsonValue]]
    complete: bool
    examined_dispatches: int
    callback_bytes: int
    interpretation: str = (
        "Derived answer plus original reader observations; independently observed cuts, "
        "not an atomic snapshot or a new native terminal."
    )


class ReadGuard(Middleware):
    """Enforce the execution closure, independently of discovery or annotations."""

    async def on_call_tool(
        self,
        context: MiddlewareContext[CallToolRequestParams],
        call_next: CallNext[CallToolRequestParams, ToolResult],
    ) -> ToolResult:
        policy = ACTIVE.get()
        if policy is None:
            return await call_next(context)
        name = context.message.name
        policy.attempts += 1
        if name not in policy.eligible or policy.attempts > policy.max_calls:
            policy.incomplete = True
            policy.records.append({"capability": name, "outcome": "dispatch-refused"})
            raise ToolError("opt01.execution_eligibility_or_call_bound")
        if policy.before_dispatch is not None:
            policy.before_dispatch()
        try:
            result = await call_next(context)
        except Exception:
            policy.incomplete = True
            policy.records.append({"capability": name, "outcome": "reader-failed"})
            raise
        structured = JSON.validate_python(result.structured_content)
        size = len(encoded(structured))
        policy.transferred += size
        if policy.transferred > policy.max_bytes:
            policy.incomplete = True
            policy.records.append({"capability": name, "outcome": "bytes-refused"})
            raise ToolError("opt01.cumulative_callback_bytes")
        policy.incomplete |= result.is_error
        policy.records.append(
            {
                "capability": name,
                "outcome": "error" if result.is_error else "observed",
                "structured": structured,
                "meta": JSON.validate_python(result.meta),
                "sha256": hashlib.sha256(encoded(structured)).hexdigest(),
            }
        )
        # Typed refusals are data, not necessarily MCP errors. Retain and label them.
        text = json.dumps(structured)
        if '"detail-refused"' in text or '"not_observed"' in text:
            policy.incomplete = True
        if isinstance(structured, dict):
            answer = structured.get("result")
            if isinstance(answer, dict) and answer.get("complete") is False:
                policy.incomplete = True
        return result


class BoundedMonty(MontySandboxProvider):
    """Use the real framework sandbox; add the full callback-inclusive deadline."""

    def __init__(self) -> None:
        super().__init__(
            limits={"max_memory": 100_000_000, "max_feed_duration_secs": 2}
        )
        self.next_policy: Policy | None = None

    async def run(
        self,
        code: str,
        *,
        inputs: dict[str, JsonValue] | None = None,
        external_functions: dict[str, Callable[..., Awaitable[object]]] | None = None,
    ) -> dict[str, JsonValue]:
        policy = self.next_policy or Policy()
        token = ACTIVE.set(policy)
        wrapped = {}
        for name, callback in (external_functions or {}).items():

            async def checked(
                *args: object,
                _callback: Callable[..., Awaitable[object]] = callback,
                **kwargs: object,
            ) -> object:
                try:
                    return await _callback(*args, **kwargs)
                except Exception:
                    policy.incomplete = True
                    raise

            wrapped[name] = checked
        try:
            async with asyncio.timeout(policy.deadline):
                result = await super().run(
                    code, inputs=inputs, external_functions=wrapped
                )
            result = JSON.validate_python(result)
            if len(encoded(result)) > policy.max_final_bytes:
                raise ToolError("opt01.final_answer_bytes")
            answer = CompositionAnswer(
                derived=result,
                observations=policy.records,
                complete=not policy.incomplete,
                examined_dispatches=policy.attempts,
                callback_bytes=policy.transferred,
            ).model_dump(mode="json")
            if len(encoded(answer)) > policy.max_final_bytes:
                raise ToolError("opt01.final_envelope_bytes")
            return answer
        finally:
            ACTIVE.reset(token)


def exact_factory(factory: DiscoveryToolFactory) -> DiscoveryToolFactory:
    def make(get_catalog: GetToolCatalog) -> Tool:
        async def exact(ctx: Context) -> Sequence[Tool]:
            return [tool for tool in await get_catalog(ctx) if tool.name in ELIGIBLE]

        return factory(exact)

    return make


async def inquiry_server() -> tuple[canonical.Runtime, BoundedMonty]:
    """Register public CodeMode-produced tools through the canonical binding seam."""
    sandbox = BoundedMonty()
    transform = CodeMode(
        sandbox_provider=sandbox,
        discovery_tools=[
            exact_factory(ListTools(name=LIST)),
            exact_factory(GetSchemas(name=SCHEMAS)),
            exact_factory(GetTags(name=TAGS)),
        ],
        execute_tool_name="inquiry_kernel",
        max_tool_calls=8,
    )
    generated = list(await transform.transform_tools([]))
    kernel = generated.pop()

    async def inquiry_compose_reads(
        code: Annotated[str, Field(max_length=16384)],
    ) -> CompositionAnswer:
        """Inquiry only: compose selected original immutable-result reads in Monty."""
        result = await kernel.run({"code": code})
        return CompositionAnswer.model_validate(result.structured_content)

    generated.append(
        Tool.from_function(
            inquiry_compose_reads,
            annotations=ToolAnnotations(readOnlyHint=True, destructiveHint=False),
        )
    )

    def inquiry_selected_results(
        outputs: Annotated[list[str], Field(min_length=1, max_length=3)],
    ) -> CompositionAnswer:
        """Stable typed comparison control, delegating to the same original readers."""
        rows = []
        observations = []
        complete = True
        for output in outputs:
            ref = reference_read(ROOT, output)
            if ref.reference is None:
                complete = False
                rows.append({"outcome": "reference-unavailable"})
                continue
            native = run_inspection.product_result(
                ref.reference.invocation, registry=Path(ref.reference.registry), limit=1
            )
            subjects = native["subjects"]
            rows.append(
                {
                    "identity": subjects[0]["identity"],
                    "outcome": subjects[0]["outcome"],
                    "assurance": native["assurance"],
                }
            )
            complete &= native["selected_unreported_count"] == 0
            observations.append(
                {
                    "invocation": ref.reference.invocation,
                    "registry": ref.reference.registry,
                    "reference_sha256": ref.source_sha256,
                    "terminal_identity": native["terminal_identity"],
                    "selected_count": native["selected_count"],
                    "selected_unreported_count": native["selected_unreported_count"],
                    "owned_drain": native["owned_drain"],
                }
            )
        return CompositionAnswer(
            derived=rows,
            observations=observations,
            complete=complete,
            examined_dispatches=2 * len(outputs),
            callback_bytes=0,
        )

    generated.append(
        Tool.from_function(
            inquiry_selected_results,
            annotations=ToolAnnotations(readOnlyHint=True, destructiveHint=False),
        )
    )
    old = canonical.bindings

    def bindings(
        root: Path, *, owner: Specialists, telemetry: canonical.BackendClient
    ) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
        declarations, tools = old(root, owner=owner, telemetry=telemetry)
        specs = tuple(
            Capability(
                id=tool.name,
                title=tool.name,
                description=tool.description or "",
                domain="execution",
                owner=Path(__file__).relative_to(ROOT).as_posix(),
                source=Path(__file__).relative_to(ROOT).as_posix(),
                guides=("root",),
                exposure="tool",
                effects="Isolated feasibility read composition; never a production binding.",
            )
            for tool in generated
        )
        return (*declarations, *specs), tools | {tool.name: tool for tool in generated}

    with patch.object(canonical, "bindings", bindings):
        runtime = canonical.build(ROOT)
    runtime.server.add_middleware(ReadGuard())
    return runtime, sandbox


def fixtures(directory: Path) -> dict[str, JsonValue]:
    """Publish three small native-format reader controls, with independent expected facts."""
    registry = directory / "registry"
    store = resource_store.FileStore(registry)
    outputs = []
    expected = []
    for label, outcome, assurance, drained in (
        ("pass", "pass", "pass", "observed"),
        ("failure", "failure", "failure", "observed"),
        ("unreported", None, "incomplete", "unobserved"),
    ):
        nonce = uuid.uuid4().hex
        name = "control_" + label
        identity = "pse-inquiry::" + name
        declaration = {
            "version": 2,
            "nonce": nonce,
            "kind": "rust",
            "selected": [{"binary_id": "pse-inquiry", "name": name}],
            "terminal_owner": "opt01-fixture",
        }
        store.create_invocation(declaration)
        with store.invocation_edit(nonce) as state:
            state["collected"] = True
        store.freeze(
            nonce,
            {
                "selection": resource_store.digest(declaration["selected"]),
                "terminal_owner": "opt01-fixture",
                "complete": outcome is not None,
                "outcomes": {} if outcome is None else {identity: outcome},
                "evidence": {"assurance": assurance, "owned_drain": drained},
            },
            declaration=declaration,
        )
        path = registry / "invocations" / (nonce + ".json")
        from scripts.test_resources import invocation_locator  # noqa: PLC0415

        resource_store.publish(path, invocation_locator(declaration))
        output = directory / label
        output.mkdir()
        reference = InvocationReference(
            invocation=str(path),
            registry=str(registry),
            ownership="owned",
            terminal_owner="native runner",
        )
        (output / "invocation-reference.json").write_text(reference.model_dump_json())
        outputs.append(str(output))
        expected.append(
            {
                "identity": identity,
                "outcome": outcome or "unreported",
                "assurance": assurance,
            }
        )
    return OBJECT.validate_python(
        {"outputs": outputs, "expected": expected, "registry": str(registry)}
    )


def code_for(outputs: list[str]) -> str:
    return (
        "rows = []\n"
        f"for output in {outputs!r}:\n"
        "    ref = await call_tool('rust_test_reference_read', {'output': output})\n"
        "    if ref['outcome'] != 'issued':\n"
        "        rows.append({'outcome': 'reference-unavailable'})\n"
        "        continue\n"
        "    r = ref['reference']\n"
        "    answer = await call_tool('product_run_read', {'invocation': r['invocation'], 'registry': r['registry'], 'limit': 1})\n"
        "    a = answer['result']['payload']\n"
        "    rows.append({'identity': a['subjects'][0]['identity'], 'outcome': a['subjects'][0]['outcome'], 'assurance': a['assurance']})\n"
        "return rows"
    )


async def read_value(
    client: Client, name: str, arguments: dict[str, JsonValue]
) -> dict[str, JsonValue]:
    result = await client.call_tool(name, arguments)
    return OBJECT.validate_python(result.structured_content)


async def probe(output: Path) -> None:
    output.mkdir(parents=True, exist_ok=False)
    control = fixtures(output)
    (output / "fixtures.json").write_bytes(encoded(control))
    outputs = TypeAdapter(list[str]).validate_python(control["outputs"])
    expected = control["expected"]
    runtime, sandbox = await inquiry_server()
    cases: dict[str, JsonValue] = {}
    async with Client(runtime.server, timeout=10) as client:
        initial = {tool.name for tool in await client.list_tools()}
        require(EXECUTE not in initial, "composition must start hidden")
        require("capabilities_select" in initial, "bootstrap must remain available")
        await client.call_tool(
            "capabilities_select",
            {"capabilities": sorted(ELIGIBLE | {EXECUTE, LIST, SCHEMAS, TAGS, TYPED})},
        )
        listed = await client.call_tool(LIST)
        text = "\n".join(getattr(block, "text", "") for block in listed.content)
        require(
            all(name in text for name in ELIGIBLE), "exact discovery missed a reader"
        )
        require("capabilities_invoke_action" not in text, "discovery leaked invoker")
        schemas = await client.call_tool(
            SCHEMAS, {"tools": sorted(ELIGIBLE), "detail": "full"}
        )
        require(bool(schemas.content), "missing actual reader schemas")
        await client.call_tool(TAGS)
        cases["bootstrap_preserved_exact_discovery"] = True

        small = []
        small_bytes = 0
        for path in outputs:
            reference = await read_value(
                client, "rust_test_reference_read", {"output": path}
            )
            ref = OBJECT.validate_python(reference["reference"])
            answer = await read_value(
                client,
                "product_run_read",
                {
                    "invocation": ref["invocation"],
                    "registry": ref["registry"],
                    "limit": 1,
                },
            )
            native = OBJECT.validate_python(
                OBJECT.validate_python(answer["result"])["payload"]
            )
            subject = OBJECT.validate_python(
                TypeAdapter(list[JsonValue]).validate_python(native["subjects"])[0]
            )
            small.append(
                {
                    "identity": subject["identity"],
                    "outcome": subject["outcome"],
                    "assurance": native["assurance"],
                }
            )
            small_bytes += len(encoded(reference)) + len(encoded(answer))
        require(
            small == expected,
            "original readers differ from independent fixture expectation",
        )
        composed = await read_value(client, EXECUTE, {"code": code_for(outputs)})
        typed = await read_value(
            client, TYPED, OBJECT.validate_python({"outputs": outputs})
        )
        require(
            composed["derived"] == expected,
            "real Monty composition lost native meanings",
        )
        require(
            typed["derived"] == expected,
            "typed owner composition differs from expected facts",
        )
        require(
            composed["complete"] is False and typed["complete"] is False,
            "unreported evidence became complete",
        )
        require(composed["examined_dispatches"] == 6, "wrong dispatch count")
        cases["three_original_invocations"] = {
            "small_calls": 6,
            "composition_calls": 1,
            "typed_composition_calls": 1,
            "discovery_excluded_from_counts": True,
            "small_result_bytes": small_bytes,
            "composition_result_bytes": len(encoded(composed)),
            "typed_composition_result_bytes": len(encoded(typed)),
            "derived_only_bytes": len(encoded(composed["derived"])),
            "same_independent_expected_facts": True,
            "interpretation": "Control fixture, not benchmark or executed Rust suite; provenance is retained in composed output.",
        }

        for target in (
            "capabilities_invoke_action",
            "capabilities_select",
            "capabilities_reset",
            EXECUTE,
        ):
            code = f"try:\n    await call_tool({target!r}, {{}})\nexcept Exception:\n    pass\nreturn 'caught'"
            answer = await read_value(client, EXECUTE, {"code": code})
            # Some targets fail in CodeMode catalogue lookup before middleware.
            require(
                answer["derived"] == "caught", "sandbox catch control did not execute"
            )
            require(
                answer["complete"] is False or answer["examined_dispatches"] == 0,
                "caught refusal presented as examined success",
            )
        cases["direct_action_controls_and_recursion_refused"] = True

        # A misleading read annotation is insufficient: guard every nested dispatch.
        effects = []

        async def indirect_read() -> str:
            await runtime.server.call_tool("inquiry_effect", {})
            return "unexpected"

        def inquiry_effect() -> str:
            effects.append("effect")
            return "effect"

        runtime.server.add_tool(
            Tool.from_function(
                indirect_read, annotations=ToolAnnotations(readOnlyHint=True)
            )
        )
        runtime.server.add_tool(
            Tool.from_function(
                inquiry_effect, annotations=ToolAnnotations(readOnlyHint=False)
            )
        )
        sandbox.next_policy = Policy(eligible=ELIGIBLE | {"indirect_read"})
        answer = await read_value(
            client,
            EXECUTE,
            {
                "code": "try:\n    await call_tool('indirect_read', {})\nexcept Exception:\n    pass\nreturn 'caught'"
            },
        )
        require(
            answer["complete"] is False and effects == [],
            "indirect action escaped actual-dispatch policy",
        )
        sandbox.next_policy = None
        cases["nested_dispatch_not_read_annotation_enforces_policy"] = True

        missing = await read_value(
            client, EXECUTE, {"code": code_for([str(output / "absent")])}
        )
        require(
            missing["complete"] is False, "typed not-observed was promoted to complete"
        )
        cases["typed_unavailable_retained"] = True

        sandbox.next_policy = Policy(max_bytes=1)
        refusal = await client.call_tool(
            EXECUTE, {"code": code_for(outputs[:1])}, raise_on_error=False
        )
        require(refusal.is_error, "cumulative callback byte limit did not refuse")
        sandbox.next_policy = Policy(max_final_bytes=1)
        refusal = await client.call_tool(
            EXECUTE, {"code": "return 'large'"}, raise_on_error=False
        )
        require(refusal.is_error, "final answer limit did not refuse")
        sandbox.next_policy = None
        cases["callback_and_final_byte_bounds"] = True

        code = (
            "for i in range(9):\n    await call_tool('rust_test_reference_read', {'output': "
            + repr(outputs[0])
            + "})\nreturn 'unexpected'"
        )
        refusal = await client.call_tool(EXECUTE, {"code": code}, raise_on_error=False)
        require(refusal.is_error, "callback count limit did not refuse")
        cases["callback_attempt_bound"] = True

        sandbox.next_policy = Policy(
            before_dispatch=lambda: (_ for _ in ()).throw(
                ToolError("opt01.stale_source")
            )
        )
        refusal = await client.call_tool(
            EXECUTE, {"code": code_for(outputs[:1])}, raise_on_error=False
        )
        require(refusal.is_error, "execution-time stale control did not refuse")
        sandbox.next_policy = None
        cases["execution_time_stale_control"] = (
            "injected prerequisite refusal; real canonical source-fence tests remain separate"
        )

        await client.call_tool("capabilities_reset")
        require(
            EXECUTE
            not in {
                tool.name for tool in await client.list_tools(cache_mode="refresh")
            },
            "reset leaked inquiry",
        )
        cases["reset_hides_inquiry"] = True

    # Exercise real Monty host-callback cancellation, including an async finally.
    entered, drained = asyncio.Event(), asyncio.Event()

    async def waiting() -> str:
        entered.set()
        try:
            await asyncio.Event().wait()
        finally:
            await asyncio.sleep(0)
            drained.set()
        return "unreachable"

    sandbox.next_policy = Policy(deadline=0.1)
    try:
        await sandbox.run(
            "return await waiting()", external_functions={"waiting": waiting}
        )
    except TimeoutError:
        require(
            entered.is_set() and drained.is_set(),
            "deadline returned before real callback finally drained",
        )
    else:
        raise AssertionError("whole lifecycle deadline failed")
    cases["whole_deadline_includes_callback_wait_and_drain"] = True
    sandbox.next_policy = None
    result = {
        "versions": {
            name: importlib.metadata.version(name)
            for name in ("fastmcp", "mcp", "pydantic-monty", "pydantic")
        },
        "cases": cases,
        "scope": "in-process real Monty/public CodeMode generation and canonical dispatch; native-format reader controls, not actual Codex or scientific qualification",
    }
    (output / "result.json").write_bytes(encoded(result))
    print(json.dumps(result, indent=2))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--serve", action="store_true")
    args = parser.parse_args()
    logging.disable(logging.CRITICAL)
    if args.serve:
        runtime, _sandbox = asyncio.run(inquiry_server())
        runtime.server.run(transport="stdio", show_banner=False)
    elif args.output is not None:
        asyncio.run(probe(args.output.resolve()))
    else:
        with tempfile.TemporaryDirectory(prefix="pse-opt01-") as directory:
            asyncio.run(probe(Path(directory) / "evidence"))


if __name__ == "__main__":
    main()
