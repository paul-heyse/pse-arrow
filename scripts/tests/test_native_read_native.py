# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Installed extension and actual endpoint controls; run after explicit native sync.

No ordinary service is started. These source/physical pages exercise real native owners,
not solver convergence, canonical storage or a performance claim.
"""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import asyncio
import json
import os
import tempfile
import unittest
from pathlib import Path

from fastmcp import Client

from scripts.capabilities import native, native_worker
from scripts.capabilities.native_read_context import NativeReadContext
from scripts.capabilities.native_schema import SchemaSet
from scripts.capabilities.server import build
from scripts.tests.test_capability_native import source_request

ROOT = Path(__file__).resolve().parents[2]


def payload(value: object) -> dict[str, object]:
    while isinstance(value, dict) and set(value) == {"result"}:
        value = value["result"]
    if not isinstance(value, dict):
        raise AssertionError("missing native endpoint response")
    selected = value.get("payload", value)
    if not isinstance(selected, dict):
        raise AssertionError("missing original native payload")
    return selected


class ActualNativeContextTests(unittest.TestCase):
    def test_actual_extension_retains_runtime_physical_and_modeling_package_between_pages(self) -> None:
        supplied = source_request()
        supplied["schema_set_fingerprint"] = SchemaSet(ROOT).fingerprint
        supplied["request"] = {"document_version": 1, "offset": 0, "limit": 1}
        context = NativeReadContext("explicit-actual-extension-context-control")
        with tempfile.TemporaryDirectory() as temporary:
            try:
                first = native_worker.execute("model_declarations", supplied, scratch=Path(temporary), context=context)
                self.assertEqual(first["outcome"], "observed")
                # Test-only inspection of actual object references; no observer was
                # added to the production protocol for qualification.
                runtime = context._runtime  # noqa: SLF001 -- actual native object identity oracle
                physical = tuple(context._physical.values())  # noqa: SLF001 -- actual native object identity oracle
                packages = tuple(context._modeling.values())  # noqa: SLF001 -- actual native object identity oracle
                self.assertIsNotNone(runtime)
                self.assertEqual(len(physical), 1)
                self.assertEqual(len(packages), 1)
                document = first["document"]
                assert isinstance(document, dict)
                offset = document.get("next_offset")
                self.assertIsInstance(offset, int)
                assert isinstance(offset, int)
                supplied["request"] = {"document_version": 1, "offset": offset, "limit": 1}
                second = native_worker.execute("model_declarations", supplied, scratch=Path(temporary), context=context)
                self.assertEqual(second["outcome"], "observed")
                self.assertIs(context._runtime, runtime)  # noqa: SLF001 -- compare exact retained native owner
                self.assertIs(next(iter(context._physical.values())), physical[0])  # noqa: SLF001 -- compare exact retained physical owner
                self.assertIs(next(iter(context._modeling.values())), packages[0])  # noqa: SLF001 -- compare exact retained source preparation
            finally:
                context.close()


class ActualNativeEndpointTests(unittest.IsolatedAsyncioTestCase):
    async def test_both_dispatch_routes_reuse_actual_worker_and_endpoint_eof_drains(self) -> None:
        endpoint = build(ROOT)
        tool = endpoint.catalogue.tools["native_model_declarations"]
        assert isinstance(tool, native.NativeTool)
        lifetime = tool.read_lifetime()
        selected = source_request()
        selected["request"] = {"document_version": 1, "offset": 0, "limit": 1}
        sessions = []
        async with Client(endpoint.server) as client:
            await client.call_tool("capabilities_select", {"capabilities": ["native_model_declarations"]})
            first_result = await client.call_tool("native_model_declarations", selected)
            self.assertFalse(first_result.is_error)
            first = payload(first_result.structured_content)
            self.assertEqual(first["outcome"], "observed")
            document = first["document"]
            assert isinstance(document, dict)
            offset = document.get("next_offset")
            self.assertIsInstance(offset, int)
            assert isinstance(offset, int)
            sessions = [session for group in lifetime._sessions.values() for session in group]  # noqa: SLF001 -- actual supervised owner oracle, no production observer
            self.assertEqual(len(sessions), 1)
            incarnation = sessions[0].incarnation
            selected["request"] = {"document_version": 1, "offset": offset, "limit": 1}
            second_result = await client.call_tool("capabilities_invoke_action", {
                "capability": "native_model_declarations", "arguments": selected})
            self.assertFalse(second_result.is_error)
            second = payload(second_result.structured_content)
            self.assertEqual(second["outcome"], "observed")
            retained = [session for group in lifetime._sessions.values() for session in group]  # noqa: SLF001 -- exact foreground owner identity
            self.assertEqual(retained, sessions)
            self.assertEqual(retained[0].incarnation, incarnation)
            self.assertFalse(retained[0].retired)
        for session in sessions:
            self.assertTrue(session.retired)
            process = session._process  # noqa: SLF001 -- actual physical process settlement oracle
            self.assertIsNotNone(process)
            assert process is not None
            self.assertEqual(process.returncode, 0)


class ActualNativeCliSessionTests(unittest.IsolatedAsyncioTestCase):
    async def test_foreground_cli_session_follows_actual_page_then_eof_closes_endpoint(self) -> None:
        with tempfile.TemporaryFile() as errors:
            process = await asyncio.create_subprocess_exec(
                str(ROOT / ".venv/bin/python"), "-m", "scripts.capability_cli", "session",
                cwd=ROOT, env={**os.environ, "OTEL_SDK_DISABLED": "true"},
                stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=errors,
            )
            assert process.stdin is not None and process.stdout is not None
            try:
                connected = json.loads(await process.stdout.readline())
                self.assertEqual(connected["session"], "connected")
                selected = source_request()
                selected["request"] = {"document_version": 1, "offset": 0, "limit": 1}
                process.stdin.write((json.dumps({"id": "first", "capability": "native_model_declarations", "arguments": selected}) + "\n").encode())
                await process.stdin.drain()
                first = json.loads(await process.stdout.readline())
                self.assertEqual(first["id"], "first")
                self.assertFalse(first["result"]["is_error"])
                original = payload(first["result"]["structured_content"])
                self.assertEqual(original["outcome"], "observed")
                document = original["document"]
                assert isinstance(document, dict)
                offset = document.get("next_offset")
                self.assertIsInstance(offset, int)
                assert isinstance(offset, int)
                selected["request"] = {"document_version": 1, "offset": offset, "limit": 1}
                process.stdin.write((json.dumps({"id": "next", "capability": "native_model_declarations", "arguments": selected}) + "\n").encode())
                await process.stdin.drain()
                second = json.loads(await process.stdout.readline())
                self.assertEqual(second["id"], "next")
                self.assertFalse(second["result"]["is_error"])
                self.assertEqual(payload(second["result"]["structured_content"])["outcome"], "observed")
            finally:
                # EOF is the foreground lifetime boundary. Do not kill the process
                # or infer native drain from an abandoned client observer.
                process.stdin.close()
                await process.communicate()
            self.assertEqual(process.returncode, 0)


if __name__ == "__main__":
    unittest.main()
