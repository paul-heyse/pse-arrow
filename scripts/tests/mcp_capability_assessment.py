# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit real-STDIO capability census and source acceptance; never run on import.

Run through scripts/pse-env with --output selecting a fresh campaign directory.
This driver records independent assertions separately from successful delivery.
It neither repairs the endpoint nor invokes an effectful operation.
"""

from __future__ import annotations

import argparse
import asyncio
import hashlib
import importlib.metadata
import json
import mimetypes
import os
import subprocess
import sys
import time
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

from fastmcp import Client
from fastmcp.client.transports import StdioTransport
from jsonschema import Draft202012Validator
from pydantic import JsonValue, TypeAdapter

if TYPE_CHECKING:
    from collections.abc import Awaitable, Callable

ROOT = Path(__file__).resolve().parents[2]
OBJECT = TypeAdapter(dict[str, JsonValue])
BOOTSTRAP = {
    "capabilities_domains",
    "capabilities_workflows",
    "capabilities_describe",
    "capabilities_select",
    "capabilities_reset",
    "capabilities_invoke_read",
    "capabilities_invoke_action",
    "list_resources",
    "read_resource",
    "list_prompts",
    "get_prompt",
}


def mapping(value: object) -> dict[str, JsonValue]:
    return OBJECT.validate_python(value)


def sequence(value: JsonValue) -> list[JsonValue]:
    if not isinstance(value, list):
        raise TypeError("Expected a JSON array")
    return value


def integer(value: JsonValue) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise TypeError("Expected an exact JSON integer")
    return value


def text(value: JsonValue) -> str:
    if not isinstance(value, str):
        raise TypeError("Expected JSON text")
    return value


def require(condition: object, reason: str) -> None:
    if not condition:
        raise AssertionError(reason)


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")


def payload(value: dict[str, JsonValue]) -> dict[str, JsonValue]:
    selected = mapping(value["result"]) if set(value) == {"result"} else value
    return mapping(selected["payload"]) if "payload" in selected else selected


def input_refused(
    error: bool, answer: dict[str, JsonValue], content: list[JsonValue]
) -> bool:
    if error:
        if len(content) != 1:
            return False
        block = mapping(content[0])
        if block.get("type") != "text":
            return False
        try:
            diagnostic = mapping(json.loads(text(block["text"])))
        except (ValueError, TypeError):
            return False
        return (
            diagnostic.get("code") == "pse.capabilities.input_refused"
            and diagnostic.get("stage") == "input_validation"
        )
    return (
        payload(answer).get("outcome") == "malformed_invocation"
        and payload(answer).get("code") == "pse.native.host_envelope_refused"
    )


def source_fact(identity: str, selected: dict) -> None:
    """Expected facts use original bytes, small known fixtures and explicit intent."""
    if identity == "workspace_task_context":
        fact = selected["sources"][0]
        require(
            fact["path"] == "scripts/capabilities/server.py"
            and fact["digest"]
            == hashlib.sha256((ROOT / fact["path"]).read_bytes()).hexdigest(),
            "task source fact differs",
        )
    elif identity == "workspace_inventory":
        require(
            selected["items"][0]["identity"] == "activity"
            and selected["items"][0]["source"] == "justfile",
            "known first recipe differs",
        )
        require(
            len(selected["items"]) == 2 and selected["next_offset"] == 2,
            "inventory page differs",
        )
    elif identity == "workspace_recipe_describe":
        require(
            selected["name"] == "capabilities"
            and selected["source"]["captured_sha256"]
            == hashlib.sha256((ROOT / "justfile").read_bytes()).hexdigest(),
            "recipe source differs",
        )
    elif identity == "workspace_producer_relationships":
        rows = selected["items"]
        require(
            {row["declaration_kind"] for row in rows}
            == {"pyo3_module", "pyo3_metadata", "pyo3_exception_supplement"},
            "three original declarations differ",
        )
        require(
            all(
                row["generator_path"] == "xtask/src/codegen/python_stubs.rs"
                and row["output_path"] == "python/pse/_native.pyi"
                and row["contract_sections"] == ["21.5"]
                for row in rows
            ),
            "producer correspondence differs",
        )
    elif identity == "workspace_concepts_query":
        item = selected["items"][0]
        require(
            item["id"] == "document:docs/README.md"
            and item["native_version"]
            == hashlib.sha256((ROOT / "docs/README.md").read_bytes()).hexdigest(),
            "document identity differs",
        )
    elif identity == "workspace_integration_view":
        require(
            selected["task"] == "mcp-assessment" and selected["checkout"] == str(ROOT),
            "integration context differs",
        )
        require(
            all(
                row["classification"] == "unknown" and not row["claims"]
                for row in selected["changes"]
            ),
            "unclaimed changes acquired ownership",
        )
    elif identity == "document_scope_query":
        item = selected["items"][0]
        require(
            item["native_id"] == "I01"
            and item["raw_status"] == "resolved"
            and item["state"] == "complete",
            "original plan status differs",
        )
        require(
            "| I01 | resolved |" in (ROOT / item["document"]).read_text(),
            "native status row missing",
        )
    elif identity == "environment_explain":
        require(
            selected["checkout"] == str(ROOT)
            and selected["interpreter"] == str(ROOT / ".venv/bin/python"),
            "selected placement differs",
        )
    elif identity == "storage_observe":
        item = next(row for row in selected["items"] if row["name"] == "target")
        info = (ROOT / "target").stat()
        require(
            item["identity"] == [info.st_dev, info.st_ino, info.st_uid]
            and item["reclaimable_bytes"] is None,
            "storage identity or reclamation differs",
        )
        require(
            selected["reclamation_authorized"] is False, "observation granted disposal"
        )
    elif identity == "knowledge_source_corpus":
        fact = selected["sources"][0]
        original = (ROOT / "docs/README.md").read_bytes()
        require(
            fact["id"] == "docs/README.md"
            and fact["version_digest"] == hashlib.sha256(original).hexdigest(),
            "corpus source differs",
        )
        require(
            selected["excerpts"][0]["text"].strip() in original.decode(),
            "excerpt not original bytes",
        )
        require(
            selected["excerpt_page"]["limit"] == 1
            and selected["excerpt_page"]["next_offset"] == 1,
            "excerpt continuation differs",
        )
    elif identity == "library_source_packet":
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        require(
            selected["declaration"] == manifest["workspace"]["dependencies"]["arrow"],
            "dependency declaration differs",
        )
        packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
        require(
            [row["version"] for row in selected["resolved"]]
            == [row["version"] for row in packages if row["name"] == "arrow"],
            "lock version differs",
        )
    elif identity == "native_contract_schema":
        original = (ROOT / "docs/generated/schema/authoring.schema.json").read_bytes()
        require(
            selected["identity"]["sha256"] == hashlib.sha256(original).hexdigest()
            and selected["schema"]["title"] == json.loads(original)["title"],
            "authoring schema differs",
        )
    elif identity == "rust_test_describe":
        result = selected["result"]
        require(
            result["command"]
            == [
                "cargo",
                "nextest",
                "run",
                "--no-fail-fast",
                "-p",
                "pse-ids",
                "--lib",
                "--locked",
                "-E",
                "test(=float::tests::signed_zero_is_preserved_and_distinguished)",
            ],
            "declared test command differs",
        )
        require(
            result["compiled_or_enumerated"] is False, "description implied execution"
        )
    elif identity == "assessment_select":
        names = [row["name"] for row in selected["result"]["checks"]]
        require(
            names
            == [
                "governance-tests",
                "codegen-all-check",
                "python-contracts-check",
                "python-stubs-check",
                "codegen-capabilities-check",
                "codegen-capability-cli-check",
                "family-check",
            ],
            "governance selection differs",
        )
    elif identity == "capability_readiness":
        require(
            selected["capability"] == "native_model_inspect"
            and selected["operation_exercised"]["value"] is None
            and selected["import_compatible"]["value"] is None,
            "passive readiness claimed live evidence",
        )
    elif identity == "capability_explain":
        require(
            selected["input_schema_valid"] is True
            and selected["admission"] == "not_attempted"
            and not selected["errors"],
            "explanation admitted work or rejected valid input",
        )
    else:
        raise ValueError("no independent source fact supplied")


class Campaign:
    def __init__(self, output: Path) -> None:
        output.mkdir(parents=True, exist_ok=False)
        self.output = output
        self.events: list[dict[str, JsonValue]] = []
        self.cases: list[dict[str, JsonValue]] = []
        self.descriptions: dict[str, dict[str, JsonValue]] = {}

    async def call(
        self, client: Client, name: str, arguments: dict[str, JsonValue]
    ) -> tuple[bool, dict[str, JsonValue]]:
        started = time.time_ns()
        result = await client.call_tool(name, arguments, raise_on_error=False)
        structured = mapping(result.structured_content or {})
        event = mapping(
            {
                "index": len(self.events),
                "tool": name,
                "arguments": arguments,
                "is_error": result.is_error,
                "structured": structured,
                "content": [item.model_dump(mode="json") for item in result.content],
                "started_ns": started,
                "finished_ns": time.time_ns(),
            }
        )
        self.events.append(event)
        with (self.output / "events.jsonl").open("a") as stream:
            stream.write(json.dumps(event, allow_nan=False) + "\n")
        return result.is_error, structured

    async def selected(
        self, client: Client, capability: str, arguments: dict[str, JsonValue]
    ) -> dict[str, JsonValue]:
        error, _ = await self.call(
            client, "capabilities_select", {"capabilities": [capability]}
        )
        require(not error, "selection refused")
        error, answer = await self.call(
            client,
            "capabilities_invoke_read",
            {
                "capability": capability,
                "arguments": arguments,
            },
        )
        require(not error, "selected read refused")
        description = self.descriptions[capability]
        schema = description.get("output_schema")
        if schema is not None:
            Draft202012Validator(mapping(schema)).validate(answer)
        return answer

    async def case(
        self,
        identity: str,
        capabilities: list[str],
        operation: Callable[[], Awaitable[None]],
    ) -> None:
        first = len(self.events)
        result: dict[str, JsonValue] = {
            "id": identity,
            "capabilities": list(capabilities),
            "layer": "real-stdio",
            "first_event": first,
            "status": "passed",
        }
        try:
            await operation()
        except Exception as error:
            result.update(
                status="failed", error=type(error).__name__, reason=str(error)
            )
        result["last_event"] = len(self.events) - 1
        self.cases.append(result)
        write(self.output / "cases.json", self.cases)

    async def census(self, client: Client) -> None:
        initial = await client.list_tools(cache_mode="refresh")
        write(
            self.output / "bootstrap.json",
            [item.model_dump(mode="json") for item in initial],
        )
        require(
            {item.name for item in initial} == BOOTSTRAP,
            "initial tools differ from eleven bootstrap tools",
        )
        error, domains = await self.call(client, "capabilities_domains", {"limit": 100})
        require(
            not error and domains["next_offset"] is None, "domain census incomplete"
        )
        error, workflows = await self.call(
            client, "capabilities_workflows", {"limit": 100}
        )
        require(
            not error and workflows["next_offset"] is None, "workflow census incomplete"
        )
        entries: list[JsonValue] = []
        for domain_value in sequence(domains["items"]):
            domain = mapping(domain_value)
            offset = 0
            while True:
                error, page = await self.call(
                    client,
                    "capabilities_domains",
                    {
                        "domain": domain["id"],
                        "offset": offset,
                        "limit": 7,
                    },
                )
                require(not error, "domain page refused")
                entries.extend(sequence(page["items"]))
                if page["next_offset"] is None:
                    break
                offset = integer(page["next_offset"])
        identities = [str(mapping(row)["id"]) for row in entries]
        require(
            len(set(identities)) == len(identities), "duplicate capability identities"
        )
        for identity in identities:
            error, description = await self.call(
                client,
                "capabilities_describe",
                {
                    "capability": identity,
                    "include_schema": True,
                },
            )
            require(not error, "description refused: " + identity)
            self.descriptions[identity] = description
            require(
                description["callable"]
                == (mapping(description["capability"])["exposure"] == "tool"),
                "callability differs from exposure",
            )
            if description["callable"]:
                Draft202012Validator.check_schema(mapping(description["input_schema"]))
                if description["output_schema"] is not None:
                    Draft202012Validator.check_schema(
                        mapping(description["output_schema"])
                    )
        write(
            self.output / "catalogue.json",
            {
                "domains": domains,
                "workflows": workflows,
                "entries": entries,
                "descriptions": self.descriptions,
            },
        )
        self.domains = domains
        self.workflows = workflows

    async def guidance(self, client: Client) -> None:
        seen: set[str] = set()
        protocol_reads = []
        for domain in sequence(self.domains["items"]):
            error, selected = await self.call(
                client, "capabilities_select", {"domains": [mapping(domain)["id"]]}
            )
            require(not error, "domain selection refused")
            for reference_value in sequence(selected["instruction_resources"]):
                reference = mapping(reference_value)
                uri = str(reference["uri"])
                if uri in seen:
                    continue
                seen.add(uri)
                error, response = await self.call(client, "read_resource", {"uri": uri})
                require(not error, "guide resource refused")
                body = mapping(json.loads(str(response["result"])))
                original = (ROOT / str(reference["path"])).read_bytes()
                require(
                    body["complete"] is True and body["content"] == original.decode(),
                    "guide content differs from original",
                )
                require(
                    body["digest"] == hashlib.sha256(original).hexdigest(),
                    "guide digest differs",
                )
                manifest = reference.get("supporting_manifest")
                if manifest is not None:
                    error, response = await self.call(
                        client, "read_resource", {"uri": manifest}
                    )
                    require(not error, "skill manifest refused")
                    package = mapping(json.loads(str(response["result"])))
                    require(package["complete"] is True, "skill manifest incomplete")
                    require(
                        package["package_digest"] == reference["package_digest"],
                        "skill package identity differs",
                    )
                    original_digest = hashlib.sha256()
                    for member_value in sequence(package["files"]):
                        member = mapping(member_value)
                        relative = Path(str(reference["path"])).parent / str(
                            member["path"]
                        )
                        require(
                            ".." not in relative.parts,
                            "manifest path escapes owned package",
                        )
                        raw = (ROOT / relative).read_bytes()
                        encoded = relative.as_posix().encode()
                        original_digest.update(len(encoded).to_bytes(8, "big"))
                        original_digest.update(encoded)
                        original_digest.update(hashlib.sha256(raw).digest())
                        require(
                            member["bytes"] == len(raw)
                            and member["digest"] == hashlib.sha256(raw).hexdigest(),
                            "manifest member differs from original bytes",
                        )
                        error, response = await self.call(
                            client, "read_resource", {"uri": member["uri"]}
                        )
                        require(not error, "supporting file refused")
                        selected_text = str(response["result"])
                        if member["uri"] == uri:
                            selected_text = str(
                                mapping(json.loads(selected_text))["content"]
                            )
                        require(
                            selected_text.encode() == raw,
                            "captured member content differs",
                        )
                        native = await client.read_resource(str(member["uri"]))
                        require(len(native) == 1, "native resource split selected file")
                        expected_mime = (
                            "application/json"
                            if member["uri"] == uri
                            else mimetypes.guess_type(str(relative))[0] or "text/plain"
                        )
                        require(
                            native[0].mime_type == expected_mime,
                            "captured member MIME differs",
                        )
                        protocol_reads.append(
                            {
                                "uri": member["uri"],
                                "mime_type": native[0].mime_type,
                                "bytes": len(raw),
                                "sha256": hashlib.sha256(raw).hexdigest(),
                            }
                        )
                    require(
                        package["package_digest"] == original_digest.hexdigest(),
                        "captured package digest differs from original file population",
                    )
        write(self.output / "guidance.json", {"guides_checked": sorted(seen)})
        write(self.output / "protocol-resources.json", protocol_reads)

    async def selection(self, client: Client) -> None:
        await self.call(client, "capabilities_reset", {})
        error, _ = await self.call(
            client,
            "capabilities_invoke_read",
            {"capability": "workspace_task_context", "arguments": {}},
        )
        require(error, "hidden read unexpectedly executed")
        args: dict[str, JsonValue] = {"capabilities": ["workspace_task_context"]}
        error, first = await self.call(client, "capabilities_select", args)
        require(
            not error and first["changed"] is True, "first selection did not replace"
        )
        error, repeat = await self.call(client, "capabilities_select", args)
        require(
            not error and repeat["changed"] is False, "repeat selection not idempotent"
        )
        error, _ = await self.call(
            client, "capabilities_select", {"capabilities": ["assessment_missing_id"]}
        )
        require(error, "unknown selection accepted")
        error, answer = await self.call(
            client,
            "capabilities_invoke_read",
            {
                "capability": "workspace_task_context",
                "arguments": {"paths": ["scripts/capabilities/server.py"]},
            },
        )
        require(not error, "unknown selection did not preserve the original view")
        source = mapping(sequence(payload(answer)["sources"])[0])
        require(
            source["digest"]
            == hashlib.sha256(
                (ROOT / "scripts/capabilities/server.py").read_bytes()
            ).hexdigest(),
            "source digest differs from independent read",
        )
        error, _ = await self.call(
            client,
            "capabilities_invoke_read",
            {
                "capability": "workspace_task_context",
                "arguments": {"unknown_canary_argument": "not-to-be-echoed"},
            },
        )
        require(error, "unknown field accepted")
        require(
            "not-to-be-echoed" not in json.dumps(self.events[-1]["content"]),
            "refusal echoed supplied value",
        )
        error, _ = await self.call(
            client,
            "capabilities_invoke_read",
            {"capability": "rust_test_run", "arguments": {}},
        )
        require(error, "read invoker accepted action")
        error, _ = await self.call(
            client,
            "capabilities_invoke_action",
            {"capability": "workspace_task_context", "arguments": {}},
        )
        require(error, "action invoker accepted read")

    async def workflow(self, client: Client) -> None:
        for row_value in sequence(self.workflows["items"]):
            row = mapping(row_value)
            error, selected = await self.call(
                client, "capabilities_select", {"workflows": [row["id"]]}
            )
            require(not error, "workflow selection refused")
            expected: set[str] = set()
            for step_value in sequence(row["steps"]):
                step = mapping(step_value)
                if step["default"]:
                    expected.update(
                        str(item) for item in sequence(step["capabilities"])
                    )
            require(
                {text(item) for item in sequence(selected["capabilities"])} == expected,
                "workflow defaults differ from original declaration",
            )
            error, _ = await self.call(
                client, "get_prompt", {"name": "workflow_" + str(row["id"])}
            )
            require(not error, "workflow prompt refused")
            error, _ = await self.call(
                client,
                "capabilities_select",
                {"workflow_steps": {str(row["id"]): ["missing_step"]}},
            )
            require(error, "unknown optional workflow step accepted")

    async def sources(self, client: Client) -> None:
        cases: list[tuple[str, dict[str, JsonValue]]] = [
            ("workspace_task_context", {"paths": ["scripts/capabilities/server.py"]}),
            ("workspace_inventory", {"limit": 2}),
            ("workspace_recipe_describe", {"recipe": "capabilities"}),
            (
                "workspace_producer_relationships",
                {"paths": ["python/pse/_native.pyi"], "limit": 5},
            ),
            (
                "workspace_concepts_query",
                {
                    "request": {
                        "concept": "document",
                        "identity": "docs/README.md",
                        "limit": 1,
                    }
                },
            ),
            (
                "workspace_integration_view",
                {"request": {"task": "mcp-assessment", "limit": 5}},
            ),
            (
                "document_scope_query",
                {
                    "path": "docs/plans/52-agent-effectiveness-and-engineering-workflows.md",
                    "limit": 2,
                },
            ),
            ("environment_explain", {}),
            ("storage_observe", {}),
            (
                "knowledge_source_corpus",
                {
                    "concept": "document",
                    "identity": "docs/README.md",
                    "excerpt_limit": 1,
                },
            ),
            ("library_source_packet", {"package": "arrow"}),
            ("native_contract_schema", {"slug": "authoring"}),
            (
                "rust_test_describe",
                {
                    "request": {
                        "package": "pse-ids",
                        "expression": "test(=float::tests::signed_zero_is_preserved_and_distinguished)",
                    }
                },
            ),
            ("assessment_select", {"request": {"group": "governance"}}),
            ("capability_readiness", {"capability": "native_model_inspect"}),
            (
                "capability_explain",
                {"capability": "workspace_task_context", "arguments": {"paths": []}},
            ),
        ]
        for identity, arguments in cases:

            async def run(
                identity: str = identity, arguments: dict[str, JsonValue] = arguments
            ) -> None:
                answer = await self.selected(client, identity, arguments)
                require(bool(answer), "empty response")
                selected = payload(answer)
                source_fact(identity, selected)

            await self.case("source-" + identity, [identity], run)

    async def strict_inputs(self, client: Client) -> None:
        """Malformed envelopes never supply a valid effectful request."""
        checked: list[str] = []
        for domain in sequence(self.domains["items"]):
            error, selection = await self.call(
                client, "capabilities_select", {"domains": [mapping(domain)["id"]]}
            )
            require(not error, "strict-input domain selection refused")
            tools = {
                tool.name: tool
                for tool in await client.list_tools(cache_mode="refresh")
            }
            for identity_value in sequence(selection["callable_tools"]):
                identity = text(identity_value)
                tool = tools[identity]
                assert tool.annotations is not None, (
                    "callable missing effect annotations"
                )
                wrapper = (
                    "capabilities_invoke_read"
                    if tool.annotations.read_only_hint is True
                    else "capabilities_invoke_action"
                )
                error, answer = await self.call(
                    client,
                    wrapper,
                    {
                        "capability": identity,
                        "arguments": {
                            "assessment_unknown_field": "private-canary-do-not-echo"
                        },
                    },
                )
                # NativeTool reports original validation as a typed native answer;
                # protocol delivery can succeed while admission is refused.
                refused = input_refused(
                    error, answer, sequence(self.events[-1]["content"])
                )
                require(refused, "malformed request unexpectedly admitted: " + identity)
                require(
                    "private-canary-do-not-echo"
                    not in json.dumps(self.events[-1]["content"]),
                    "refusal echoed supplied value",
                )
                checked.append(identity)
        expected = {key for key, row in self.descriptions.items() if row["callable"]}
        require(
            set(checked) == expected,
            "strict-input coverage differs from live callable population",
        )
        write(
            self.output / "strict-inputs.json",
            {
                "checked": sorted(checked),
                "scope": "invalid envelopes only; no valid action admitted",
            },
        )

    async def run(self) -> int:
        write(
            self.output / "environment.json",
            {
                "revision": subprocess.check_output(
                    ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
                ).strip(),
                "python": sys.version,
                "fastmcp": importlib.metadata.version("fastmcp"),
                "jsonschema": importlib.metadata.version("jsonschema"),
                "root": str(ROOT),
                "baseline_failures": 0,
            },
        )
        transport = StdioTransport(
            command=str(ROOT / ".venv/bin/python"),
            args=["-m", "scripts.capabilities"],
            cwd=str(ROOT),
            env=dict(os.environ),
            keep_alive=False,
        )
        async with Client(transport, timeout=120) as client:
            await self.case("T01-census", [], lambda: self.census(client))
            if self.descriptions:
                await self.case("T03-selection", [], lambda: self.selection(client))
                await self.case("T04-guidance", [], lambda: self.guidance(client))
                await self.case("T03-workflows", [], lambda: self.workflow(client))
                await self.sources(client)
                await self.case(
                    "T05-every-binding-strict-input",
                    [],
                    lambda: self.strict_inputs(client),
                )
            await self.call(client, "capabilities_reset", {})
            require(
                {item.name for item in await client.list_tools(cache_mode="refresh")}
                == BOOTSTRAP,
                "reset did not restore bootstrap",
            )
        summary = {
            "status": "failed"
            if any(row["status"] == "failed" for row in self.cases)
            else "passed",
            "cases": self.cases,
            "events": len(self.events),
            "baseline_failures": 0,
            "scope": "real STDIO census, guidance, selection and source reads; not native or actual-host qualification",
            "capability_count": len(self.descriptions),
            "callable_count": sum(
                row["callable"] is True for row in self.descriptions.values()
            ),
            "endpoint": "closed normally after reset",
        }
        write(self.output / "lane.json", summary)
        print(json.dumps(summary, sort_keys=True))
        return int(summary["status"] == "failed")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    return asyncio.run(Campaign(arguments.output.resolve()).run())


if __name__ == "__main__":
    raise SystemExit(main())
