# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Specialist lifetime, real language contract and selected-generation controls."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import asyncio
import json
import tempfile
import unittest
from contextlib import asynccontextmanager
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, Mock, patch

from fastmcp import Client, FastMCP

from scripts import semantic_edits, semantic_navigation
from scripts.capabilities import specialists


class ConnectionTests(unittest.IsolatedAsyncioTestCase):
    async def test_reuses_task_owned_context_and_cancel_only_request(self):
        entered = []
        exited = []
        client = SimpleNamespace(list_tools=AsyncMock(return_value=[]))
        @asynccontextmanager
        async def factory():
            entered.append(asyncio.current_task())
            try:
                yield client
            finally:
                exited.append(asyncio.current_task())
        owner = specialists.Connection(factory)
        self.assertFalse(entered)
        async def first(selected, tools):
            self.assertIs(selected, client)
            return 1
        self.assertEqual(await owner.invoke(first), 1)
        started = asyncio.Event()
        cancelled = asyncio.Event()
        async def waiting(selected, tools):
            started.set()
            try:
                await asyncio.Event().wait()
            finally:
                cancelled.set()
        request = asyncio.create_task(owner.invoke(waiting))
        await started.wait()
        request.cancel()
        with self.assertRaises(asyncio.CancelledError):
            await request
        await cancelled.wait()
        self.assertEqual(await owner.invoke(first), 1)
        self.assertEqual(len(entered), 1)
        self.assertEqual(client.list_tools.await_count, 1)
        await owner.close()
        self.assertEqual(exited, entered)
        with self.assertRaisesRegex(ValueError, "closed"):
            await owner.invoke(first)

    async def test_startup_failure_retains_cause_and_closes_owner(self):
        @asynccontextmanager
        async def factory():
            raise ValueError("selected backend unavailable")
            yield
        owner = specialists.Connection(factory)
        with self.assertRaisesRegex(ValueError, "selected backend unavailable"):
            await owner.invoke(AsyncMock())
        await owner.close()


class SpecialistTests(unittest.IsolatedAsyncioTestCase):
    async def test_catalogue_construction_is_source_only_and_closed(self):
        root = Path(__file__).resolve().parents[2]
        with patch.object(semantic_navigation, "connection") as connect, patch.object(
            specialists.surreal_server, "selected_service"
        ) as selected:
            owner = specialists.Specialists(root)
            specs, tools = specialists.bindings(root, owner=owner)
            self.assertEqual(len(specs), len(tools))
            families = {spec.id: spec.family for spec in specs}
            self.assertEqual(families["navigation_hover"], "language-navigation")
            self.assertEqual(families["navigation_rename_preview"], "controlled-edits")
            self.assertEqual(families["edit_preview_read"], "controlled-edits")
            self.assertNotIn("preview", tools["edit_preview_apply"].parameters["properties"])
            self.assertEqual(families["database_query"], "database-tooling")
            self.assertFalse(owner.languages)
            self.assertIsNone(owner.database)
            connect.assert_not_called()
            selected.assert_not_called()
            self.assertEqual(tools["navigation_hover"].parameters["properties"]["line"]["minimum"], 1)
            self.assertNotIn("name", tools["database_query"].parameters["properties"])
            self.assertNotIn("credentials", json.dumps([tool.parameters for tool in tools.values()]))
            await owner.close()

    async def test_database_fence_rechecked_before_backend_and_secrets_not_returned(self):
        root = Path(__file__).resolve().parents[2]
        owner = specialists.Specialists(root)
        selected = Mock(port=1234, selection_fence="a" * 64)
        selected.identity = {"invocation": "actual-generation", "state": "/private/state"}
        selected.authorization.return_value = {"Authorization": "must-not-leak"}
        with patch.object(specialists.surreal_server, "selected_service", return_value=selected):
            observed = await owner.select_database()
            self.assertEqual(observed["selection"], "a" * 64)
            self.assertNotIn("must-not-leak", json.dumps(observed))
        with self.assertRaisesRegex(ValueError, "fence differs"):
            await owner.database_call("b" * 64, "query", {"query": "RETURN 1"})
        self.assertIsNone(owner.database)
        selected.recheck.side_effect = specialists.surreal_server.SupervisorError("Selected service changed during observation")
        with self.assertRaisesRegex(specialists.surreal_server.SupervisorError, "changed"):
            await owner.database_call("a" * 64, "query", {"query": "RETURN 1"})
        selected.authorization.assert_not_called()
        await owner.close()

    async def test_transport_error_is_distinct_and_credentials_are_redacted(self):
        root = Path(__file__).resolve().parents[2]
        owner = specialists.Specialists(root)
        secret = "must-not-leak-password"
        token = specialists.base64.b64encode(("pse-selection:" + secret).encode()).decode()
        selected = Mock(selection_fence="a" * 64)
        selected.authorization.return_value = {"Authorization": "Basic " + token}
        owner.selected = selected
        owner.database_call = AsyncMock(side_effect=RuntimeError("connection failed: Basic " + token + " password=" + secret))
        _, tools = specialists.bindings(root, owner=owner)
        with self.assertRaisesRegex(Exception, "pse.specialist.transport_failed") as caught:
            await tools["database_info"].run({"selection": "a" * 64})
        self.assertNotIn(token, str(caught.exception))
        self.assertNotIn(secret, str(caught.exception))
        self.assertIn("RuntimeError", str(caught.exception))
        await owner.close()

    async def test_real_python_calls_reuse_bridge_and_protected_edit_preview(self):
        with tempfile.TemporaryDirectory(prefix="pse-specialist-test-") as directory:
            root = Path(directory)
            source = root / "sample.py"
            original = ("def sample(value: int) -> int:\n    return value + 1\nresult = sample(2)\n"
                        + "# " + "unchanged source " * 1500 + "💡" * 3000 + "\n")
            source.write_text(original)
            self.assertGreater(len(source.read_bytes()), 20 << 10)
            owner = specialists.Specialists(root, fixture=True)
            _, tools = specialists.bindings(root, owner=owner)
            server = FastMCP("specialist-test", strict_input_validation=True)
            for tool in tools.values():
                server.add_tool(tool)
            try:
                async with Client(server) as client:
                    symbols = await client.call_tool("navigation_symbols", {"language": "python", "file_path": "sample.py"})
                    payload = symbols.structured_content["payload"]
                    self.assertFalse(payload["native"]["isError"], payload)
                    self.assertTrue(payload["source_unchanged"])
                    task = owner.languages["python"].task
                    hover = await client.call_tool("navigation_hover", {
                        "language": "python", "file_path": "sample.py", "line": 3, "character": 11,
                        "source_sha256": payload["source_sha256"]})
                    self.assertFalse(hover.structured_content["payload"]["native"]["isError"])
                    self.assertIs(owner.languages["python"].task, task)
                    wrong = await client.call_tool("navigation_hover", {
                        "language": "python", "file_path": "sample.py", "line": 3, "character": 11,
                        "source_sha256": "0" * 64}, raise_on_error=False)
                    self.assertTrue(wrong.is_error)
                    self.assertIn("source version differs", wrong.content[0].text)
                    response = await client.call_tool("navigation_rename_preview", {
                        "language": "python", "file_path": "sample.py", "line": 1, "character": 5,
                        "new_name": "renamed", "owned": ["sample.py"]})
                    result = response.structured_content["payload"]
                    self.assertFalse(result["native"]["isError"], result)
                    self.assertIsNotNone(result["preview"], result)
                    self.assertTrue(response.structured_content["complete"], response.structured_content)
                    handle = result["preview"]["preview_sha256"]
                    self.assertNotIn("before", result["preview"]["files"][0])
                    self.assertNotIn("changes", result["native"]["result"])
                    # Traverse the original large Unicode text over the real MCP boundary.
                    for side, expected in (("before", original), ("after", original.replace("sample", "renamed"))):
                        offset = 0
                        chunks = []
                        while True:
                            detail = await client.call_tool("edit_preview_read", {
                                "preview_sha256": handle, "file_index": 0,
                                "side": side, "offset": offset, "limit": 4096})
                            self.assertTrue(detail.structured_content["complete"], detail.structured_content)
                            window = detail.structured_content["payload"]
                            self.assertIsInstance(window["text"], str)
                            chunks.append(window["text"])
                            if window["next_offset"] is None:
                                break
                            self.assertGreater(window["next_offset"], offset)
                            offset = window["next_offset"]
                        self.assertEqual("".join(chunks), expected)
                    self.assertEqual(source.read_text(), original)
                    applied = await client.call_tool("edit_preview_apply", {
                        "preview_sha256": handle, "receipt": "receipt.json"})
                    self.assertTrue(applied.structured_content["payload"]["complete"])
                    self.assertIn("renamed", source.read_text())
                    reconciled = await client.call_tool("edit_preview_reconcile", {"preview_sha256": handle})
                    self.assertTrue(reconciled.structured_content["payload"]["complete"])
                    self.assertEqual(reconciled.structured_content["payload"]["preview_sha256"], handle)
                    self.assertEqual(source.read_text(), original.replace("sample", "renamed"))
                    self.assertEqual(json.loads((root / "receipt.json").read_text())["preview_sha256"], handle)
            finally:
                await owner.close()
            self.assertTrue(task.done())

    async def test_database_tool_contract_keeps_native_error_and_reuses_selected_client(self):
        root = Path(__file__).resolve().parents[2]
        backend = FastMCP("selected-viewer-fixture")
        calls = []
        @backend.tool()
        def info(target: str | None = None, namespace: str | None = None,
                 database: str | None = None) -> dict:
            calls.append((target, namespace, database))
            return {"native": "schema", "target": target}
        @backend.tool()
        def query(query: str, parameters: dict | None = None,
                  namespace: str | None = None, database: str | None = None) -> dict:
            from fastmcp.exceptions import ToolError
            raise ToolError("NotAllowed: VIEWER cannot mutate canonical project records")
        selected = Mock(port=1234, selection_fence="a" * 64)
        selected.identity = {"invocation": "selected-generation"}
        selected.authorization.return_value = {"Authorization": "private"}
        owner = specialists.Specialists(root)
        owner.selected = selected
        _, tools = specialists.bindings(root, owner=owner)
        entered = []
        @asynccontextmanager
        async def client_factory(transport, **kwargs):
            entered.append(transport)
            async with Client(backend) as client:
                yield client
        try:
            with patch.object(specialists, "Client", side_effect=client_factory):
                first = await tools["database_info"].run({"selection": "a" * 64, "target": "root"})
                second = await tools["database_info"].run({"selection": "a" * 64, "target": "root"})
                refused = await tools["database_query"].run({"selection": "a" * 64, "query": "DELETE canonical_record"})
            self.assertEqual(len(entered), 1)
            self.assertEqual(calls, [("root", None, None), ("root", None, None)])
            self.assertFalse(first.structured_content["payload"]["native"]["isError"])
            self.assertTrue(second.structured_content["payload"]["selection_unchanged"])
            native = refused.structured_content["payload"]["native"]
            self.assertTrue(native["isError"])
            self.assertIn("NotAllowed", str(native))
            self.assertNotIn("private", json.dumps(refused.structured_content))
        finally:
            await owner.close()
        self.assertTrue(owner.database.task.done())

    async def test_partial_native_edit_retains_output_and_refuses_preview(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "sample.py").write_text("x = 1\n")
            owner = specialists.Specialists(root)
            native = {"isError": False, "result": {"changes": [], "dropped": {"out_of_workspace": 1}}}
            owner.language = AsyncMock(return_value={"native": native, "source_unchanged": True})
            _, tools = specialists.bindings(root, owner=owner)
            result = await tools["navigation_rename_preview"].run({
                "language": "python", "file_path": "sample.py", "line": 1, "character": 1,
                "new_name": "other", "owned": ["sample.py"]})
            payload = result.structured_content["payload"]
            self.assertEqual(payload["native"], native)
            self.assertIsNone(payload["preview"])
            self.assertIn("incomplete server result", payload["preview_refusal"])
            self.assertFalse(result.structured_content["complete"])
            await owner.close()

    async def test_exact_preview_handles_reject_foreign_unknown_tampered_and_raw_requests(self):
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as foreign:
            root = Path(directory)
            source = root / "sample.py"
            source.write_text("x=1\n")
            owner = specialists.Specialists(root)
            other = specialists.Specialists(root)
            _, tools = specialists.bindings(root, owner=owner)
            preview = semantic_edits.replacements(root, semantic_edits.capture(root, ["sample.py"]),
                                                  {"sample.py": "x=2\n"}, producer="test")
            foreign_preview = {**preview, "root": foreign}
            with self.assertRaisesRegex(ValueError, "another checkout"):
                owner.retain_preview(foreign_preview)
            handle = owner.retain_preview(preview)["preview_sha256"]
            for refused in ("0" * 64, ("1" if handle[0] != "1" else "2") + handle[1:]):
                with self.assertRaisesRegex(Exception, "unknown edit preview digest"):
                    await tools["edit_preview_apply"].run({"preview_sha256": refused, "receipt": "receipt.json"})
            with self.assertRaisesRegex(ValueError, "unknown edit preview digest"):
                other.retained_preview(handle)
            self.assertFalse((root / "receipt.json").exists())
            self.assertEqual(source.read_text(), "x=1\n")
            # Mutating a caller-owned summary or original dict cannot mutate retention.
            preview["files"][0]["after"] = "injected text"
            self.assertEqual(owner.retained_preview(handle)["files"][0]["after"], "x=2\n")
            # Corrupting the private retained bytes is detected before source/receipt effects.
            owner._previews[handle] = b"{}"
            with self.assertRaisesRegex(Exception, "digest mismatch"):
                await tools["edit_preview_apply"].run({"preview_sha256": handle, "receipt": "receipt.json"})
            self.assertFalse((root / "receipt.json").exists())
            self.assertEqual(source.read_text(), "x=1\n")
            server = FastMCP("exact-preview-input", strict_input_validation=True)
            server.add_tool(tools["edit_preview_apply"])
            async with Client(server) as client:
                raw = await client.call_tool("edit_preview_apply", {"preview": preview, "receipt": "receipt.json"}, raise_on_error=False)
                self.assertTrue(raw.is_error)
            await owner.close()
            await other.close()
            self.assertFalse(owner._previews)
            self.assertEqual(owner._preview_bytes, 0)
            with self.assertRaisesRegex(Exception, "closed"):
                await tools["edit_preview_read"].run({"preview_sha256": handle})

    async def test_preview_capacity_refusal_preserves_issued_handles_and_has_no_effects(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "sample.py"
            source.write_text("x=1\n")
            cut = semantic_edits.capture(root, ["sample.py"])
            first = semantic_edits.replacements(root, cut, {"sample.py": "x=2\n"}, producer="test")
            second = semantic_edits.replacements(root, cut, {"sample.py": "x=3\n"}, producer="test")
            owner = specialists.Specialists(root, preview_capacity=1)
            retained = owner.retain_preview(first)
            self.assertEqual(owner.retain_preview(first), retained)
            with self.assertRaisesRegex(specialists.PreviewCapacityError, "no existing handle was released"):
                owner.retain_preview(second)
            self.assertEqual(source.read_text(), "x=1\n")
            _, tools = specialists.bindings(root, owner=owner)
            native = {"isError": False, "result": {"changes": [{"uri": source.as_uri(), "edits": [
                {"range": {"start": {"line": 1, "character": 3}, "end": {"line": 1, "character": 4}}, "new_text": "3"}]}]}}
            owner.language = AsyncMock(return_value={"native": native, "source_unchanged": True})
            with self.assertRaisesRegex(Exception, "PreviewCapacityError"):
                await tools["navigation_rename_preview"].run({"language": "python", "file_path": "sample.py", "line": 1,
                    "character": 1, "new_name": "other", "owned": ["sample.py"]})
            self.assertEqual(source.read_text(), "x=1\n")
            applied = await tools["edit_preview_apply"].run({"preview_sha256": retained["preview_sha256"], "receipt": "receipt.json"})
            self.assertTrue(applied.structured_content["payload"]["complete"])
            self.assertEqual(source.read_text(), "x=2\n")
            await owner.close()
            # Aggregate byte capacity is independent from the count limit.
            first_bytes = len(json.dumps(first, sort_keys=True).encode("utf-8"))
            byte_owner = specialists.Specialists(root, preview_byte_capacity=first_bytes)
            byte_handle = byte_owner.retain_preview(first)["preview_sha256"]
            with self.assertRaisesRegex(specialists.PreviewCapacityError, "capacity exhausted"):
                byte_owner.retain_preview(second)
            self.assertEqual(byte_owner.retained_preview(byte_handle), first)
            await byte_owner.close()

    async def test_retained_preview_stale_basis_and_protected_receipt_still_refuse_before_effects(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "sample.py").write_text("x=1\n")
            (root / "consumer.py").write_text("import sample\n")
            owner = specialists.Specialists(root)
            _, tools = specialists.bindings(root, owner=owner)
            preview = semantic_edits.replacements(root, semantic_edits.capture(root, ["sample.py", "consumer.py"]),
                                                  {"sample.py": "x=2\n"}, producer="test")
            handle = owner.retain_preview(preview)["preview_sha256"]
            for receipt in ("sample.py", "consumer.py", "target/receipt.json"):
                with self.assertRaisesRegex(Exception, "overwrite|protected"):
                    await tools["edit_preview_apply"].run({"preview_sha256": handle, "receipt": receipt})
            (root / "consumer.py").write_text("# concurrent edit\n")
            with self.assertRaisesRegex(Exception, "stale preview basis"):
                await tools["edit_preview_apply"].run({"preview_sha256": handle, "receipt": "receipt.json"})
            self.assertFalse((root / "receipt.json").exists())
            self.assertEqual((root / "sample.py").read_text(), "x=1\n")
            state = await tools["edit_preview_reconcile"].run({"preview_sha256": handle})
            self.assertFalse(state.structured_content["payload"]["complete"])
            self.assertEqual(state.structured_content["payload"]["files"][0]["state"], "unchanged")
            self.assertEqual(state.structured_content["payload"]["basis"][0]["state"], "diverged")
            (root / "consumer.py").write_text("import sample\n")
            (root / "sample.py").write_text("x=7\n")
            with self.assertRaisesRegex(Exception, "stale preview basis"):
                await tools["edit_preview_apply"].run({"preview_sha256": handle, "receipt": "receipt.json"})
            self.assertFalse((root / "receipt.json").exists())
            original = await tools["edit_preview_read"].run({"preview_sha256": handle, "file_index": 0})
            self.assertEqual(original.structured_content["payload"]["text"], "x=1\n")
            self.assertEqual((root / "sample.py").read_text(), "x=7\n")
            await owner.close()

    async def test_retained_preview_partial_write_receipt_can_be_reconciled(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("a.py", "b.py"):
                (root / name).write_text("x=1\n")
            owner = specialists.Specialists(root)
            _, tools = specialists.bindings(root, owner=owner)
            preview = semantic_edits.replacements(root, semantic_edits.capture(root, ["a.py", "b.py"]),
                                                  {"a.py": "x=2\n", "b.py": "x=2\n"}, producer="test")
            handle = owner.retain_preview(preview)["preview_sha256"]
            write = semantic_edits._write
            def interrupted(path, content):
                if path.name == "b.py":
                    raise OSError("injected second-write interruption")
                write(path, content)
            with patch.object(semantic_edits, "_write", side_effect=interrupted), self.assertRaisesRegex(Exception, "second-write interruption"):
                await tools["edit_preview_apply"].run({"preview_sha256": handle, "receipt": "receipt.json"})
            receipt = json.loads((root / "receipt.json").read_text())
            self.assertFalse(receipt["complete"])
            self.assertEqual(receipt["preview_sha256"], handle)
            state = await tools["edit_preview_reconcile"].run({"preview_sha256": handle})
            self.assertEqual([file["state"] for file in state.structured_content["payload"]["files"]], ["applied", "unchanged"])
            await owner.close()


if __name__ == "__main__":
    unittest.main()
