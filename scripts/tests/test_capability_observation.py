# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Interface correlation preserves native truth and detached owner lifetimes."""

# ruff: noqa: PT009, PT027 -- stdlib endpoint controls
from __future__ import annotations

import asyncio
import contextlib
import hashlib
import io
import json
import tempfile
import threading
import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client, FastMCP
from fastmcp.exceptions import NotFoundError, ResourceError
from fastmcp.prompts import PromptResult
from fastmcp.resources import ResourceResult
from fastmcp.server.transforms import PromptsAsTools, ResourcesAsTools
from fastmcp.tools import ToolResult
from mcp.types import CallToolResult, TextContent, TextResourceContents

from scripts import (
    activity_watch,
    evidence_detail,
    run_inspection,
    test_resources,
    test_run,
)
from scripts.capabilities import observation
from scripts.capabilities.responses import project
from scripts.capabilities.responses import bounded
from scripts.capabilities.server import Runtime, build

ROOT = Path(__file__).resolve().parents[2]


def object_value(value: object) -> dict:
    if not isinstance(value, dict):
        raise TypeError("expected an actual object")
    return value


def diagnostics(output: io.StringIO) -> list[dict]:
    return [
        json.loads(line)
        for line in output.getvalue().splitlines()
        if line.startswith('{"code":"pse.capabilities.interface_')
    ]


class ObservationTests(unittest.IsolatedAsyncioTestCase):
    def runtime(self) -> Runtime:
        runtime = build(ROOT)
        if not any(
            isinstance(item, observation.Observation)
            for item in runtime.server.middleware
        ):
            observation.install(runtime.server, revision=runtime.catalogue.revision)
        return runtime

    async def test_failed_composed_interaction_has_one_safe_diagnostic_and_no_handle(
        self,
    ) -> None:
        runtime = self.runtime()
        output = io.StringIO()
        async with Client(runtime.server) as client:
            await client.call_tool(
                "capabilities_select", {"capabilities": ["product_run_read"]}
            )
            with contextlib.redirect_stderr(output):
                result = await client.call_tool(
                    "capabilities_invoke_read",
                    {
                        "capability": "product_run_read",
                        "arguments": {
                            "invocation": "secret-supplied-handle",
                            "registry": "/missing/secret-registry",
                        },
                    },
                    raise_on_error=False,
                )
        self.assertTrue(result.is_error)
        records = diagnostics(output)
        self.assertEqual(len(records), 1)
        record = records[0]
        self.assertEqual(record["native_reference_count"], 0)
        self.assertEqual(
            [call["interface"] for call in record["calls"]], ["failed", "failed"]
        )
        self.assertEqual(record["calls"][1]["parent"], 0)
        self.assertEqual(record["calls"][1]["stage"], "adapter")
        self.assertEqual(record["loaded_revision"], runtime.catalogue.revision)
        self.assertNotIn("secret", json.dumps(record))
        self.assertNotIn("native_references", record)
        self.assertEqual(record["transport_delivery"], "unobserved")

    async def test_refusal_partial_and_existing_metadata_survive_nested_dispatch(
        self,
    ) -> None:
        server = FastMCP("native-observation-test", strict_input_validation=True)
        payload = bounded(
            {
                "outcome": "domain_refusal",
                "complete": False,
                "code": "pse.native.scientific_refusal",
                "message": "sensitive-source-body",
                "omissions": ["native cause detail bounded"],
                "nested": {"invocation": "invented", "registry": "invented"},
            },
            "native_source_admit",
        ).model_dump(mode="json")
        original = ToolResult(
            structured_content=payload, meta={"owner": {"identity": "native-refusal"}}
        )
        expected = original.structured_content

        @server.tool(name="native_source_admit")
        async def native() -> ToolResult:
            return original

        @server.tool(name="capabilities_invoke_action")
        async def invoke() -> ToolResult:
            return await server.call_tool("native_source_admit", {})

        observation.install(server, revision="test-loaded-revision")
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            result = await server.call_tool("capabilities_invoke_action", {})
        self.assertIs(result, original)
        self.assertIs(result.structured_content, expected)
        metadata = object_value(result.meta)
        self.assertEqual(metadata["owner"], {"identity": "native-refusal"})
        correlation = object_value(metadata[observation.META_KEY])
        self.assertEqual(correlation["calls"][0]["interface"], "returned")
        signals = correlation["calls"][1]["native"]
        self.assertEqual(signals["outcome"], "domain_refusal")
        self.assertFalse(signals["projection_complete"])
        self.assertGreater(signals["omission_count"], 0)
        self.assertEqual(correlation["native_references"], [])
        self.assertEqual(len(diagnostics(output)), 1)
        self.assertNotIn("sensitive-source-body", output.getvalue())
        self.assertEqual(correlation["trace"]["end_to_end_continuity"], "unestablished")
        self.assertEqual(correlation["trace"]["export"], "unverified")

    async def test_wire_metadata_merge_and_native_refusal_are_independent(self) -> None:
        server = FastMCP("wire-observation-test")
        native = bounded(
            {
                "outcome": "incomplete_observation",
                "complete": False,
                "omissions": ["narrow original window"],
                "chunks": [],
            },
            "native_result_read",
        ).model_dump(mode="json")

        @server.tool(name="native_result_read")
        async def read() -> ToolResult:
            return ToolResult.from_mcp_result(
                CallToolResult(
                    content=[],
                    structured_content=native,
                    _meta={"native": "retained"},
                    is_error=True,
                )
            )

        observation.install(server, revision="wire-revision")
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            async with Client(server) as client:
                result = await client.call_tool(
                    "native_result_read", {}, raise_on_error=False
                )
        self.assertTrue(result.is_error)
        self.assertEqual(result.structured_content, native)
        self.assertEqual(object_value(result.meta)["native"], "retained")
        metadata = object_value(object_value(result.meta)[observation.META_KEY])
        self.assertEqual(metadata["calls"][0]["interface"], "error_result")
        self.assertEqual(
            metadata["calls"][0]["native"]["outcome"], "incomplete_observation"
        )
        self.assertEqual(len(diagnostics(output)), 1)

    async def test_provider_refusal_logs_no_user_supplied_name_or_request_values(
        self,
    ) -> None:
        server = FastMCP("lookup-observation-test")
        observation.install(server, revision="test-revision")
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            async with Client(server) as client:
                result = await client.call_tool(
                    "secret-credential-as-tool-name",
                    {"password": "do-not-log"},
                    raise_on_error=False,
                )
        self.assertTrue(result.is_error)
        record = diagnostics(output)[0]
        self.assertEqual(record["calls"][0]["stage"], "provider")
        self.assertNotIn("secret", json.dumps(record))
        self.assertNotIn("password", json.dumps(record))
        self.assertNotIn("do-not-log", json.dumps(record))

    async def test_cancelled_watch_detaches_and_later_reader_authenticates_owner(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            registry = Path(directory) / "registry"
            with patch.object(test_resources, "registry", return_value=registry):
                invocation = test_resources.invocation(
                    "python", [{"name": "suite::one"}, {"name": "suite::two"}]
                )
            runtime = self.runtime()
            started, released, finished = (
                threading.Event(),
                threading.Event(),
                threading.Event(),
            )
            owner_watch = activity_watch.watch

            def watch(selected: dict, **_arguments: object) -> dict:
                started.set()
                try:
                    if not released.wait(5):
                        raise RuntimeError("test failed to release detached owner read")
                    return owner_watch(selected, max_wait=0)
                finally:
                    finished.set()

            output = io.StringIO()
            try:
                with (
                    patch.object(activity_watch, "watch", side_effect=watch),
                    contextlib.redirect_stderr(output),
                ):
                    async with Client(runtime.server) as client:
                        await client.call_tool(
                            "capabilities_select",
                            {"capabilities": ["workload_watch", "product_run_read"]},
                        )
                        pending = asyncio.create_task(
                            client.call_tool(
                                "capabilities_invoke_read",
                                {
                                    "capability": "workload_watch",
                                    "arguments": {
                                        "kind": "run",
                                        "handle": invocation.stem,
                                        "registry": str(registry),
                                    },
                                },
                            )
                        )
                        self.assertTrue(await asyncio.to_thread(started.wait, 3))
                        pending.cancel()
                        with self.assertRaises(asyncio.CancelledError):
                            await pending
                        # The subject remains live/open after its observer disconnects.
                        self.assertEqual(
                            run_inspection.product_result(
                                str(invocation), registry=registry
                            )["terminal_status"],
                            "missing",
                        )
                        for _ in range(100):
                            if any(
                                record["code"].endswith("observer_detached")
                                for record in diagnostics(output)
                            ):
                                break
                            await asyncio.sleep(0.01)
                        detached = [
                            record
                            for record in diagnostics(output)
                            if record["code"].endswith("observer_detached")
                        ]
                        self.assertEqual(len(detached), 1)
                        self.assertEqual(detached[0]["native_reference_count"], 0)
                        # An independent native owner settles partial results later.
                        with patch.object(
                            test_resources, "registry", return_value=registry
                        ):
                            test_run.finish(
                                invocation,
                                {
                                    "exit_code": 130,
                                    "results": [
                                        {"nodeid": "suite::one", "status": "passed"}
                                    ],
                                    "report_errors": ["interrupted"],
                                    "owned_drain": "unresolved",
                                },
                                "runner",
                            )
                        released.set()
                        self.assertTrue(await asyncio.to_thread(finished.wait, 3))
                        later = await client.call_tool(
                            "capabilities_invoke_read",
                            {
                                "capability": "product_run_read",
                                "arguments": {
                                    "invocation": invocation.stem,
                                    "registry": str(registry),
                                    "limit": 1,
                                },
                            },
                        )
                        native = object_value(
                            object_value(later.structured_content)["payload"]
                        )
                        self.assertFalse(native["terminal"]["complete"])
                        self.assertEqual(native["workload_exit"], 130)
                        self.assertEqual(native["owned_drain"], "unresolved")
                        reference = object_value(
                            object_value(later.meta)[observation.META_KEY]
                        )["native_references"][0]
                        self.assertEqual(reference["capability"], "product_run_read")
                        self.assertEqual(reference["arguments"]["tail"], 0)
                        with patch.object(
                            run_inspection,
                            "artifact_observation",
                            side_effect=AssertionError(
                                "metadata correlation must not hydrate protected data"
                            ),
                        ):
                            authentic = evidence_detail.invocation_detail(
                                reference["arguments"]["invocation"],
                                registry=Path(reference["arguments"]["registry"]),
                                subject=None,
                                tail=0,
                            )
                        self.assertEqual(
                            authentic["terminal_identity"], native["terminal_identity"]
                        )
                        self.assertEqual(
                            authentic["selected_unreported"], ["suite::two"]
                        )
            finally:
                released.set()
                await asyncio.to_thread(finished.wait, 5)

    async def test_host_partial_outcome_and_observation_budget_do_not_rewrite_schema(
        self,
    ) -> None:
        server = FastMCP("bounded-observation-test")
        payload = project(
            {
                "invocation": "/native/invocations/" + "a" * 32 + ".json",
                "catalogue_reader": {
                    "capability": "product_run_read",
                    "invocation": "/native/invocations/" + "a" * 32 + ".json",
                    "registry": "/native",
                },
                "exit_code": 1,
                "complete": False,
                "omissions": ["selected population bounded"],
            },
            "rust_test_run",
        ).model_dump(mode="json")

        @server.tool(name="rust_test_run")
        async def native() -> ToolResult:
            return ToolResult(structured_content=payload, meta={"owner": "preserve"})

        @server.tool(name="capabilities_invoke_action")
        async def invoke(depth: int = 0) -> ToolResult:
            return (
                await server.call_tool(
                    "capabilities_invoke_action", {"depth": depth - 1}
                )
                if depth
                else await server.call_tool("rust_test_run", {})
            )

        observation.install(server, revision="bounded-revision")
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            result = await server.call_tool("capabilities_invoke_action", {"depth": 20})
        self.assertEqual(result.structured_content, payload)
        metadata = object_value(object_value(result.meta)[observation.META_KEY])
        self.assertLessEqual(
            len(json.dumps(metadata, ensure_ascii=True).encode()),
            observation.MAX_META_BYTES,
        )
        self.assertEqual(len(metadata["calls"]), observation.MAX_FRAMES)
        self.assertGreater(metadata["omitted_calls"], 0)
        self.assertEqual(len(metadata["native_references"]), 1)
        self.assertEqual(object_value(result.meta)["owner"], "preserve")
        self.assertEqual(len(diagnostics(output)), 1)
        self.assertEqual(
            metadata["native_references"][0]["arguments"]["invocation"],
            payload["result"]["invocation"],
        )
        self.assertEqual(
            metadata["calls"][0]["operation_sha256"],
            hashlib.sha256(b"capabilities_invoke_action").hexdigest(),
        )

    async def test_resource_prompt_and_projection_metadata_keep_one_outer_record(
        self,
    ) -> None:
        server = FastMCP("content-observation-test")

        @server.resource("pse://guides/{selection}", name="guide_template")
        async def guide(selection: str) -> ResourceResult:
            if selection == "fail-secret":
                raise ResourceError(
                    "pse.capabilities.guidance_unavailable: raw-secret-source"
                )
            return ResourceResult(
                "sensitive-source-body", meta={"resource_owner": "retained"}
            )

        @server.prompt(name="workflow_fixture")
        async def workflow(token: str) -> PromptResult:
            return PromptResult(
                "sensitive-prompt-body",
                meta={"prompt_owner": "retained", "token_length": len(token)},
            )

        server.add_transform(ResourcesAsTools(server))
        server.add_transform(PromptsAsTools(server))
        observation.install(
            server,
            revision="content-revision",
            operation_names=frozenset({"read_resource", "get_prompt"}),
            resource_names={"pse://guides/{selection}": "guide_template"},
            prompt_names=frozenset({"workflow_fixture"}),
        )
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            async with Client(server) as client:
                resource = await client.read_resource_mcp("pse://guides/request-secret")
                self.assertEqual(
                    object_value(resource.meta)["resource_owner"], "retained"
                )
                resource_content = resource.contents[0]
                if not isinstance(resource_content, TextResourceContents):
                    raise TypeError("expected text resource content")
                self.assertEqual(resource_content.text, "sensitive-source-body")
                self.assertEqual(
                    object_value(resource.meta)[observation.META_KEY]["calls"][0][
                        "component"
                    ],
                    "guide_template",
                )
                prompt = await client.get_prompt(
                    "workflow_fixture", {"token": "secret-token"}
                )
                self.assertEqual(object_value(prompt.meta)["prompt_owner"], "retained")
                prompt_content = prompt.messages[0].content
                if not isinstance(prompt_content, TextContent):
                    raise TypeError("expected text prompt content")
                self.assertEqual(prompt_content.text, "sensitive-prompt-body")
                self.assertEqual(
                    object_value(prompt.meta)[observation.META_KEY]["calls"][0][
                        "component"
                    ],
                    "workflow_fixture",
                )
                resource_projection = await client.call_tool(
                    "read_resource", {"uri": "pse://guides/request-secret"}
                )
                resource_calls = object_value(resource_projection.meta)[
                    observation.META_KEY
                ]["calls"]
                self.assertEqual(
                    [call["component_kind"] for call in resource_calls],
                    ["tool", "resource"],
                )
                self.assertEqual(resource_calls[0]["operation"], "read_resource")
                self.assertEqual(resource_calls[1]["component"], "guide_template")
                prompt_projection = await client.call_tool(
                    "get_prompt",
                    {
                        "name": "workflow_fixture",
                        "arguments": {"token": "secret-token"},
                    },
                )
                prompt_calls = object_value(prompt_projection.meta)[
                    observation.META_KEY
                ]["calls"]
                self.assertEqual(
                    [call["component_kind"] for call in prompt_calls],
                    ["tool", "prompt"],
                )
                refused = await client.call_tool(
                    "read_resource",
                    {"uri": "pse://guides/fail-secret"},
                    raise_on_error=False,
                )
                self.assertTrue(refused.is_error)
        records = diagnostics(output)
        self.assertEqual(len(records), 5)
        self.assertEqual(len(records[-1]["calls"]), 2)
        self.assertEqual(records[-1]["calls"][1]["stage"], "interface")
        for record in records:
            serialized = json.dumps(record)
            self.assertNotIn("secret", serialized)
            self.assertNotIn("sensitive", serialized)
            self.assertNotIn("pse://guides/", serialized)
            self.assertEqual(
                record["native_transport"], "server_unobserved_client_errors"
            )

    async def test_unknown_content_components_log_only_hashes_and_registered_names_are_readable(
        self,
    ) -> None:
        server = FastMCP("component-names-test")

        @server.tool(name="registered_local_owner")
        async def owner() -> dict[str, str]:
            return {"status": "interface only"}

        observation.install(
            server,
            revision="names-revision",
            operation_names=frozenset({"registered_local_owner"}),
        )
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            result = await server.call_tool("registered_local_owner", {})
            with self.assertRaises(NotFoundError):
                await server.read_resource("pse://unknown/secret-param")
            with self.assertRaises(NotFoundError):
                await server.render_prompt("secret-unknown-prompt", {})
        self.assertEqual(
            object_value(result.meta)[observation.META_KEY]["calls"][0]["operation"],
            "registered_local_owner",
        )
        records = diagnostics(output)
        self.assertEqual(len(records), 3)
        for record in records[1:]:
            self.assertNotIn("secret", json.dumps(record))
            self.assertEqual(record["calls"][0]["stage"], "provider")
            self.assertNotIn("component", record["calls"][0])

    async def test_oversized_identity_never_falls_back_to_default_registry_and_reference_budget_is_explicit(
        self,
    ) -> None:
        server = FastMCP("reference-bounds-test")

        @server.tool(name="product_run_read")
        async def read(index: int, oversized: bool = False) -> ToolResult:
            return ToolResult(
                structured_content=bounded(
                    {
                        "kind": "product-test",
                        "invocation": f"{index:032x}",
                        "registry": "/"
                        + "r" * (2049 if oversized else 1900)
                        + str(index),
                    },
                    "product_run_read",
                ).model_dump(mode="json")
            )

        @server.tool(name="capabilities_invoke_read")
        async def invoke() -> ToolResult:
            result = None
            for index in range(8):
                result = await server.call_tool("product_run_read", {"index": index})
            assert result is not None
            return result

        observation.install(server, revision="reference-bounds-revision")
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            oversized = await server.call_tool(
                "product_run_read", {"index": 1, "oversized": True}
            )
            result = await server.call_tool("capabilities_invoke_read", {})
        self.assertGreater(
            len(object_value(oversized.structured_content)["payload"]["registry"]), 2048
        )
        self.assertEqual(
            object_value(oversized.meta)[observation.META_KEY]["native_references"], []
        )
        metadata = object_value(object_value(result.meta)[observation.META_KEY])
        self.assertGreater(metadata["omitted_references"], 0)
        self.assertLessEqual(
            len(json.dumps(metadata, ensure_ascii=True).encode()),
            observation.MAX_META_BYTES,
        )
        self.assertEqual(
            len(metadata["native_references"]) + metadata["omitted_references"], 8
        )
        for reference in metadata["native_references"]:
            self.assertEqual(len(reference["arguments"]["invocation"]), 32)
            self.assertGreater(len(reference["arguments"]["registry"]), 1900)
        self.assertEqual(len(diagnostics(output)), 2)


if __name__ == "__main__":
    unittest.main()
