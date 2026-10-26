# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Installed extension and actual endpoint controls; run after explicit native sync.

No ordinary service is started. These source/physical pages exercise real native owners,
not solver convergence, canonical storage or a performance claim.
"""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import asyncio
import copy
import json
import os
import tempfile
import unittest
from contextlib import suppress
from pathlib import Path
from typing import TYPE_CHECKING

from fastmcp import Client

from scripts.capabilities import native, native_worker
from scripts.capabilities.cleanup import settle
from scripts.capabilities.native_contracts import Resources
from scripts.capabilities.native_read_context import NativeReadContext
from scripts.capabilities.native_schema import SchemaSet
from scripts.capabilities.server import build
from scripts.tests.native_read_adverse import (
    AuthenticatedReadWorker,
    ImageSelection,
    image_stamp,
    replaced_image,
)
from scripts.tests.native_read_canonical import CanonicalReadSelection
from scripts.tests.test_capability_native import source_request

if TYPE_CHECKING:
    from pydantic import JsonValue

    from scripts.capabilities.native_read_sessions import ReadSession, ReadSessions

ROOT = Path(__file__).resolve().parents[2]


def payload(value: object) -> dict[str, object]:
    while isinstance(value, dict) and set(value) == {"result"}:
        value = value["result"]
    if not isinstance(value, dict):
        raise TypeError("missing native endpoint response")
    selected = value.get("payload", value)
    if not isinstance(selected, dict):
        raise TypeError("missing original native payload")
    return selected


def owned_sessions(lifetime: ReadSessions) -> list[ReadSession]:
    """Observe exact test-owned workers without adding a product inspection API."""
    groups = lifetime._sessions  # noqa: SLF001 -- test-owned workers
    return [session for group in groups.values() for session in group]


class ActualNativeContextTests(unittest.TestCase):
    def test_actual_extension_retains_runtime_physical_and_modeling_package_between_pages(
        self,
    ) -> None:
        supplied = source_request()
        supplied["schema_set_fingerprint"] = SchemaSet(ROOT).fingerprint
        supplied["request"] = {"document_version": 1, "offset": 0, "limit": 1}
        context = NativeReadContext("explicit-actual-extension-context-control")
        with tempfile.TemporaryDirectory() as temporary:
            try:
                first = native_worker.execute(
                    "model_declarations",
                    supplied,
                    scratch=Path(temporary),
                    context=context,
                )
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
                supplied["request"] = {
                    "document_version": 1,
                    "offset": offset,
                    "limit": 1,
                }
                second = native_worker.execute(
                    "model_declarations",
                    supplied,
                    scratch=Path(temporary),
                    context=context,
                )
                self.assertEqual(second["outcome"], "observed")
                self.assertIs(context._runtime, runtime)  # noqa: SLF001 -- compare exact retained native owner
                self.assertIs(next(iter(context._physical.values())), physical[0])  # noqa: SLF001 -- compare exact retained physical owner
                self.assertIs(next(iter(context._modeling.values())), packages[0])  # noqa: SLF001 -- compare exact retained source preparation
            finally:
                context.close()


class ActualNativeEndpointTests(unittest.IsolatedAsyncioTestCase):
    async def test_both_dispatch_routes_reuse_actual_worker_and_endpoint_eof_drains(
        self,
    ) -> None:
        endpoint = build(ROOT)
        tool = endpoint.catalogue.tools["native_model_declarations"]
        assert isinstance(tool, native.NativeTool)
        lifetime = tool.read_lifetime()
        selected = source_request()
        selected["request"] = {"document_version": 1, "offset": 0, "limit": 1}
        sessions = []
        async with Client(endpoint.server) as client:
            await client.call_tool(
                "capabilities_select", {"capabilities": ["native_model_declarations"]}
            )
            first_result = await client.call_tool("native_model_declarations", selected)
            self.assertFalse(first_result.is_error)
            first = payload(first_result.structured_content)
            self.assertEqual(first["outcome"], "observed")
            document = first["document"]
            assert isinstance(document, dict)
            offset = document.get("next_offset")
            self.assertIsInstance(offset, int)
            assert isinstance(offset, int)
            sessions = owned_sessions(lifetime)
            self.assertEqual(len(sessions), 1)
            incarnation = sessions[0].incarnation
            selected["request"] = {"document_version": 1, "offset": offset, "limit": 1}
            second_result = await client.call_tool(
                "capabilities_invoke_action",
                {"capability": "native_model_declarations", "arguments": selected},
            )
            self.assertFalse(second_result.is_error)
            second = payload(second_result.structured_content)
            self.assertEqual(second["outcome"], "observed")
            retained = owned_sessions(lifetime)
            self.assertEqual(retained, sessions)
            self.assertEqual(retained[0].incarnation, incarnation)
            self.assertFalse(retained[0].retired)
        for session in sessions:
            self.assertTrue(session.retired)
            process = session._process  # noqa: SLF001 -- actual physical process settlement oracle
            self.assertIsNotNone(process)
            assert process is not None
            self.assertEqual(process.returncode, 0)


class ActualNativeAdverseTests(unittest.IsolatedAsyncioTestCase):
    async def first_page(
        self, client: Client, lifetime: ReadSessions
    ) -> tuple[dict[str, JsonValue], dict[str, object], ReadSession]:
        selected = source_request()
        selected["request"] = {"document_version": 1, "offset": 0, "limit": 1}
        await client.call_tool(
            "capabilities_select", {"capabilities": ["native_model_declarations"]}
        )
        result = await client.call_tool("native_model_declarations", selected)
        self.assertFalse(result.is_error)
        first = payload(result.structured_content)
        self.assertEqual(first["outcome"], "observed")
        document = first["document"]
        assert isinstance(document, dict)
        assert isinstance(document.get("next_offset"), int)
        selected["request"] = {
            "document_version": 1,
            "offset": document["next_offset"],
            "limit": 1,
        }
        sessions = owned_sessions(lifetime)
        self.assertEqual(len(sessions), 1)
        return selected, first, sessions[0]

    async def test_stopped_actual_worker_keeps_queued_native_owner_and_allocation_until_eof_drain(
        self,
    ) -> None:
        endpoint = build(ROOT)
        tool = endpoint.catalogue.tools["native_model_declarations"]
        assert isinstance(tool, native.NativeTool)
        lifetime = tool.read_lifetime()
        client = Client(endpoint.server)
        await client.__aenter__()
        worker: AuthenticatedReadWorker | None = None
        observer = None
        closing = None
        owned = None
        session = None
        try:
            selected, _first, session = await self.first_page(client, lifetime)
            worker = AuthenticatedReadWorker(session)
            await worker.stop()
            observer = asyncio.create_task(
                client.call_tool("native_model_declarations", selected)
            )

            async def queued() -> None:
                while worker is not None and worker.queued_bytes() == 0:
                    await asyncio.sleep(0.01)

            await asyncio.wait_for(queued(), timeout=10)
            # The real request is in the stopped worker's actual pipe. This proves
            # queued native-owner custody, not entry into Rust or a numerical solver.
            pending = tuple(lifetime._pending)  # noqa: SLF001 -- original owned task survives observer detachment
            self.assertEqual(len(pending), 1)
            owned = pending[0]
            self.assertFalse(owned.done())
            observer.cancel()
            with self.assertRaises(asyncio.CancelledError):
                await observer
            self.assertFalse(owned.cancelled())
            worker.require_charged()
            closing = asyncio.create_task(client.__aexit__(None, None, None))
            await asyncio.sleep(0.1)
            self.assertFalse(
                closing.done(),
                "endpoint EOF returned before its stopped native owner could drain",
            )
            self.assertFalse(owned.done())
            worker.require_charged()
        finally:
            # Always resume exactly the authenticated pidfd before awaiting cleanup,
            # including failed assertions and repeated test cancellation.
            if worker is not None:
                worker.resume()
            if observer is not None and not observer.done():
                observer.cancel()
                with suppress(asyncio.CancelledError):
                    await observer

            async def close_client() -> None:
                if closing is None:
                    await client.__aexit__(None, None, None)
                else:
                    await closing

            try:
                await settle(close_client(), lifetime.close())
                if owned is not None:
                    completed = await owned
                    self.assertEqual(completed["outcome"], "observed")
                if session is not None:
                    process = session._process  # noqa: SLF001 -- physical supervised process exit oracle
                    assert process is not None
                    self.assertEqual(process.returncode, 0)
                if worker is not None:
                    self.assertTrue(worker.exited())
            finally:
                if worker is not None:
                    worker.close()

    async def test_owned_installed_image_replacement_retires_actual_session_and_restores_original_inode(
        self,
    ) -> None:
        # Root selects exact fresh bits and holds producers for this sequential test.
        # Absence of that explicit selection is a failure, never a skipped control.
        selected_image = ImageSelection.selected(ROOT)
        endpoint = build(ROOT)
        tool = endpoint.catalogue.tools["native_model_declarations"]
        assert isinstance(tool, native.NativeTool)
        lifetime = tool.read_lifetime()
        async with Client(endpoint.server) as client:
            selected, _first, session = await self.first_page(client, lifetime)
            process = session._process  # noqa: SLF001 -- compare actual admitted supervised process
            assert process is not None
            with replaced_image(selected_image):
                self.assertNotEqual(
                    image_stamp(selected_image.binary)[:2], selected_image.stamp[:2]
                )
                result = await client.call_tool("native_model_declarations", selected)
                self.assertFalse(result.is_error)
                refused = payload(result.structured_content)
                self.assertIn(
                    refused["outcome"], {"stale_interface", "prerequisite_unavailable"}
                )
                self.assertIn(
                    refused["code"],
                    {
                        "pse.native.artifact_not_current",
                        "pse.native.retained_read_context_changed",
                    },
                )
                self.assertTrue(session.retired)
                self.assertEqual(process.returncode, 0)
            self.assertEqual(
                image_stamp(selected_image.binary)[:2], selected_image.stamp[:2]
            )
            selected["request"] = {"document_version": 1, "offset": 0, "limit": 1}
            restored = payload(
                (
                    await client.call_tool("native_model_declarations", selected)
                ).structured_content
            )
            self.assertEqual(restored["outcome"], "observed")
            peers = [peer for group in lifetime._sessions.values() for peer in group]  # noqa: SLF001 -- fresh original-image admission follows exact prior worker drain
            self.assertEqual(len(peers), 2)
            self.assertIs(peers[0], session)
            self.assertNotEqual(peers[1].incarnation, session.incarnation)


class ActualCanonicalReadAdverseTests(unittest.IsolatedAsyncioTestCase):
    async def first_result_page(
        self, client: Client, lifetime: ReadSessions, selection: CanonicalReadSelection
    ) -> tuple[dict[str, JsonValue], dict[str, object], ReadSession]:
        arguments = copy.deepcopy(selection.arguments)
        request = arguments["request"]
        assert isinstance(request, dict)
        request["view"] = {"kind": "schema", "offset": 0, "limit": 1}
        request["cursor"] = None
        await client.call_tool(
            "capabilities_select", {"capabilities": ["native_result_inspect"]}
        )
        result = await client.call_tool("native_result_inspect", arguments)
        self.assertFalse(result.is_error)
        first = payload(result.structured_content)
        self.assertEqual(first["outcome"], "observed")
        document = first["document"]
        assert isinstance(document, dict)
        self.assertEqual(document["kind"], "schema")
        self.assertEqual(document["offset"], 0)
        cursor = document["next_cursor"]
        self.assertIsInstance(cursor, dict)
        assert isinstance(cursor, dict)
        request["cursor"] = cursor
        sessions = owned_sessions(lifetime)
        self.assertEqual(len(sessions), 1)
        return arguments, first, sessions[0]

    async def test_existing_canonical_cursor_preserves_native_basis_and_known_retired_session_fence(
        self,
    ) -> None:
        selection = CanonicalReadSelection.selected()
        endpoint = build(ROOT)
        tool = endpoint.catalogue.tools["native_result_inspect"]
        assert isinstance(tool, native.NativeTool)
        lifetime = tool.read_lifetime()
        async with Client(endpoint.server) as client:
            arguments, _first, session = await self.first_result_page(
                client, lifetime, selection
            )
            second_result = await client.call_tool(
                "capabilities_invoke_read",
                {"capability": "native_result_inspect", "arguments": arguments},
            )
            self.assertFalse(second_result.is_error)
            second = payload(second_result.structured_content)
            self.assertEqual(second["outcome"], "observed")
            document = second["document"]
            assert isinstance(document, dict)
            self.assertEqual(document["kind"], "schema")
            self.assertEqual(document["offset"], 1)
            peers = [peer for group in lifetime._sessions.values() for peer in group]  # noqa: SLF001 -- actual worker/session reuse oracle
            self.assertEqual(peers, [session])
            changed = copy.deepcopy(arguments)
            changed_request = changed["request"]
            assert isinstance(changed_request, dict)
            changed_request.update(start=1, end=2)
            refusal = payload(
                (
                    await client.call_tool("native_result_inspect", changed)
                ).structured_content
            )
            self.assertEqual(refusal["code"], "pse.native.changed_session_cursor_basis")
            unknown = copy.deepcopy(arguments)
            unknown_request = unknown["request"]
            assert isinstance(unknown_request, dict)
            unknown_cursor = unknown_request["cursor"]
            assert isinstance(unknown_cursor, dict)
            unknown_cursor["issuing_store"] = "deliberately-different-original-store"
            refused_by_native = payload(
                (
                    await client.call_tool("native_result_inspect", unknown)
                ).structured_content
            )
            self.assertEqual(refused_by_native["outcome"], "observed")
            refusal_document = refused_by_native["document"]
            assert isinstance(refusal_document, dict)
            self.assertEqual(
                (refusal_document["kind"], refusal_document["reason"]),
                ("refused", "cursor_changed"),
            )
            # Retire exactly this authenticated idle context through its normal EOF
            # owner, without killing a process or modifying canonical data.
            await session.close()
            process = session._process  # noqa: SLF001 -- actual original supervised receiver drain
            assert process is not None
            self.assertEqual(process.returncode, 0)
            retired = payload(
                (
                    await client.call_tool("native_result_inspect", arguments)
                ).structured_content
            )
            self.assertEqual(retired["code"], "pse.native.retired_session_cursor")
            peers = [peer for group in lifetime._sessions.values() for peer in group]  # noqa: SLF001 -- refusal cannot create a replacement receiver
            self.assertEqual(peers, [session])
        # A fresh original-owner admission with no old session cursor remains readable.
        fresh = build(ROOT)
        async with Client(fresh.server) as client:
            tool = fresh.catalogue.tools["native_result_inspect"]
            assert isinstance(tool, native.NativeTool)
            await self.first_result_page(client, tool.read_lifetime(), selection)

    async def test_independent_public_reader_protection_survives_queued_observer_cancel_and_endpoint_eof(
        self,
    ) -> None:
        import pse  # noqa: PLC0415 -- explicit installed native/public canonical control

        selection = CanonicalReadSelection.selected()
        original = selection.arguments["request"]
        assert isinstance(original, dict)
        run, attempt, relation = (
            original["run"],
            original["attempt"],
            original["relation"],
        )
        assert isinstance(run, str)
        assert isinstance(attempt, str)
        assert isinstance(relation, str)
        limits = Resources()
        with tempfile.TemporaryDirectory(
            prefix="pse-original-reader-control-"
        ) as temporary:
            runtime = pse.Runtime(
                pse.EngineSettings(
                    memory_limit_bytes=limits.memory_limit_bytes,
                    threads=limits.threads,
                    spill_dir=temporary,
                    max_spill_bytes=limits.max_spill_bytes,
                    batch_size=8192,
                ),
                substrate=selection.state,
                database=selection.database,
            )
            stream = None
            guard = None
            worker = None
            observer = None
            closing = None
            owned = None
            client = None
            lifetime = None
            batch = None
            expected = None

            def protected_reclaim_refusal() -> None:
                # Only the expected refusal is authorized on this shared fixture.
                # Reader checks precede study retirement and no successful reclamation
                # is ever requested after these original public readers close.
                with self.assertRaises(pse.InspectionError) as captured:
                    runtime.reclaim_run_results(run)
                self.assertIn(
                    "run_results_have_protected_readers", str(captured.exception)
                )

            try:
                stream = runtime.results(run, attempt, relation, start=0, end=1)
                protected_reclaim_refusal()
                batch = stream.next_batch()
                self.assertIsNotNone(batch)
                assert batch is not None
                expected = batch.to_pylist()
                self.assertGreater(batch.num_rows, 0)
                # This separate unconsumed public reader owns unread source protection.
                # The already copied Arrow batch is an independent lifetime witness.
                guard = runtime.results(run, attempt, relation, start=0, end=1)
                protected_reclaim_refusal()
                endpoint = build(ROOT)
                tool = endpoint.catalogue.tools["native_result_inspect"]
                assert isinstance(tool, native.NativeTool)
                lifetime = tool.read_lifetime()
                client = Client(endpoint.server)
                await client.__aenter__()
                arguments, _first, session = await self.first_result_page(
                    client, lifetime, selection
                )
                worker = AuthenticatedReadWorker(session)
                await worker.stop()
                observer = asyncio.create_task(
                    client.call_tool("native_result_inspect", arguments)
                )

                async def queued() -> None:
                    while worker is not None and worker.queued_bytes() == 0:
                        await asyncio.sleep(0.01)

                await asyncio.wait_for(queued(), timeout=10)
                pending = tuple(lifetime._pending)  # noqa: SLF001 -- original queued native owner is independent of public stream protection
                self.assertEqual(len(pending), 1)
                owned = pending[0]
                observer.cancel()
                with self.assertRaises(asyncio.CancelledError):
                    await observer
                closing = asyncio.create_task(client.__aexit__(None, None, None))
                await asyncio.sleep(0.1)
                self.assertFalse(closing.done())
                worker.require_charged()
                # The original reader, not a cross-page transaction or the stopped
                # worker, supplies this native protection/refusal evidence.
                protected_reclaim_refusal()
            finally:
                if worker is not None:
                    worker.resume()
                if observer is not None and not observer.done():
                    observer.cancel()
                    with suppress(asyncio.CancelledError):
                        await observer

                async def close_client() -> None:
                    if closing is not None:
                        await closing
                    elif client is not None:
                        await client.__aexit__(None, None, None)

                async def close_lifetime() -> None:
                    if lifetime is not None:
                        await lifetime.close()

                try:
                    await settle(close_client(), close_lifetime())
                    if owned is not None:
                        completed = await owned
                        self.assertEqual(completed["outcome"], "observed")
                    if worker is not None:
                        self.assertTrue(worker.exited())
                    if guard is not None:
                        protected_reclaim_refusal()
                finally:
                    if worker is not None:
                        worker.close()
                    if guard is not None:
                        guard.close()
                    if stream is not None:
                        stream.close()
                    runtime.close()
            assert batch is not None
            self.assertEqual(batch.to_pylist(), expected)
        # No successful reclaim follows reader release: preserve the shared original run.


class ActualNativeCliSessionTests(unittest.IsolatedAsyncioTestCase):
    async def test_foreground_cli_session_follows_actual_page_then_eof_closes_endpoint(
        self,
    ) -> None:
        with tempfile.TemporaryFile() as errors:
            process = await asyncio.create_subprocess_exec(
                str(ROOT / ".venv/bin/python"),
                "-m",
                "scripts.capability_cli",
                "session",
                cwd=ROOT,
                env={**os.environ, "OTEL_SDK_DISABLED": "true"},
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
                stderr=errors,
            )
            assert process.stdin is not None
            assert process.stdout is not None
            try:
                connected = json.loads(await process.stdout.readline())
                self.assertEqual(connected["session"], "connected")
                selected = source_request()
                selected["request"] = {"document_version": 1, "offset": 0, "limit": 1}
                process.stdin.write(
                    (
                        json.dumps(
                            {
                                "id": "first",
                                "capability": "native_model_declarations",
                                "arguments": selected,
                            }
                        )
                        + "\n"
                    ).encode()
                )
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
                selected["request"] = {
                    "document_version": 1,
                    "offset": offset,
                    "limit": 1,
                }
                process.stdin.write(
                    (
                        json.dumps(
                            {
                                "id": "next",
                                "capability": "native_model_declarations",
                                "arguments": selected,
                            }
                        )
                        + "\n"
                    ).encode()
                )
                await process.stdin.drain()
                second = json.loads(await process.stdout.readline())
                self.assertEqual(second["id"], "next")
                self.assertFalse(second["result"]["is_error"])
                self.assertEqual(
                    payload(second["result"]["structured_content"])["outcome"],
                    "observed",
                )
            finally:
                # EOF is the foreground lifetime boundary. Do not kill the process
                # or infer native drain from an abandoned client observer.
                process.stdin.close()
                await process.communicate()
            self.assertEqual(process.returncode, 0)


if __name__ == "__main__":
    unittest.main()
