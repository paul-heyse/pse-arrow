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
from unittest.mock import patch

from fastmcp import Client

from scripts.capabilities import native, native_worker
from scripts.capabilities.cleanup import settle
from scripts.capabilities.native_contracts import Resources
from scripts.capabilities.native_read_context import NativeReadContext
from scripts.capabilities.native_schema import JSON_OBJECT, SchemaSet
from scripts.capabilities.server import build
from scripts.tests.native_read_adverse import (
    AuthenticatedReadWorker,
    ImageSelection,
    PublishedReadGenerations,
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
    def test_changed_compiled_effect_fact_refuses_before_runtime_acquisition(
        self,
    ) -> None:
        import pse  # noqa: PLC0415 -- explicit installed catalogue control
        from pse import codec  # noqa: PLC0415 -- original JSON boundary codec

        changed = JSON_OBJECT.validate_json(
            codec.encode_json(pse.host_operation_catalogue())
        )
        operations = changed["operations"]
        assert isinstance(operations, list)
        operation = operations[0]
        assert isinstance(operation, dict)
        facts = operation["effect_facts"]
        assert isinstance(facts, dict)
        facts["read_only"] = not facts["read_only"]
        supplied = source_request()
        supplied["schema_set_fingerprint"] = SchemaSet(ROOT).fingerprint
        supplied["request"] = {"document_version": 1, "offset": 0, "limit": 1}
        context = NativeReadContext("changed-compiled-effect-control")
        try:
            with (
                tempfile.TemporaryDirectory() as temporary,
                patch.object(pse, "host_operation_catalogue", return_value=changed),
                patch.object(pse, "Runtime") as acquire,
            ):
                result = native_worker.execute(
                    "model_declarations",
                    supplied,
                    scratch=Path(temporary),
                    context=context,
                )
                self.assertEqual(
                    result["code"], "pse.native.operation_catalogue_mismatch"
                )
                acquire.assert_not_called()
        finally:
            context.close()

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

    async def test_mutable_installation_alias_does_not_revoke_published_attachment(
        self,
    ) -> None:
        # The root authenticates only the test-owned mutable alias being replaced.
        # Production consumers use the already-issued immutable generation.
        selected_image = ImageSelection.selected(ROOT)
        endpoint = build(ROOT)
        tool = endpoint.catalogue.tools["native_model_declarations"]
        assert isinstance(tool, native.NativeTool)
        lifetime = tool.read_lifetime()
        async with Client(endpoint.server) as client:
            selected, _first, session = await self.first_page(client, lifetime)
            process = session._process  # noqa: SLF001 -- actual process lifetime oracle
            assert process is not None
            with replaced_image(selected_image):
                self.assertNotEqual(
                    image_stamp(selected_image.binary)[:2], selected_image.stamp[:2]
                )
                result = await client.call_tool("native_model_declarations", selected)
                self.assertFalse(result.is_error)
                self.assertEqual(
                    payload(result.structured_content)["outcome"], "observed"
                )
                self.assertFalse(session.retired)
                self.assertIsNone(process.returncode)
                self.assertEqual(len(owned_sessions(lifetime)), 1)
                # A distinct fresh worker attaches the published image even while
                # the mutable installation alias contains a deliberate bad image.
                fresh = build(ROOT)
                fresh_tool = fresh.catalogue.tools["native_model_declarations"]
                assert isinstance(fresh_tool, native.NativeTool)
                async with Client(fresh.server) as new_client:
                    await self.first_page(new_client, fresh_tool.read_lifetime())
            self.assertEqual(
                image_stamp(selected_image.binary)[:2], selected_image.stamp[:2]
            )
        self.assertEqual(process.returncode, 0)


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
        sessions = [session for session in owned_sessions(lifetime) if not session.retired]
        self.assertEqual(len(sessions), 1)
        return arguments, first, sessions[0]

    async def test_actual_publication_replacement_retains_old_cursor_then_drains_and_attaches_new_generation(self) -> None:
        selection = CanonicalReadSelection.selected()
        workers = []
        with tempfile.TemporaryDirectory(prefix="pse-two-read-generations-") as directory:
            generations = PublishedReadGenerations(ROOT, Path(directory))
            first = await asyncio.to_thread(generations.publish)
            with patch.dict(os.environ, {"_PSE_NATIVE_PUBLICATION": str(generations.receipt)}):
                endpoint = build(ROOT)
                tool = endpoint.catalogue.tools["native_result_inspect"]
                assert isinstance(tool, native.NativeTool)
                lifetime = tool.read_lifetime()
                try:
                    async with Client(endpoint.server) as client:
                        arguments, _page, old = await self.first_result_page(client, lifetime, selection)
                        old_worker = AuthenticatedReadWorker(old)
                        workers.append(old_worker)
                        old_worker.require_image(first)
                        second = await asyncio.to_thread(generations.publish)
                        self.assertNotEqual(first.generation, second.generation)
                        self.assertNotEqual(first.artifact.binary, second.artifact.binary)
                        self.assertEqual(first.artifact.features, second.artifact.features)
                        old_worker.require_image(first)
                        advanced = payload((await client.call_tool("native_result_inspect", arguments)).structured_content)
                        self.assertEqual(advanced["outcome"], "observed")
                        document = advanced["document"]
                        assert isinstance(document, dict)
                        self.assertEqual(document["offset"], 1)
                        self.assertEqual(owned_sessions(lifetime), [old])
                        old_worker.require_image(first)
                        await old.close()
                        await old_worker.require_settled()
                        process = old._process  # noqa: SLF001 -- actual normal EOF owner
                        assert process is not None
                        self.assertEqual(process.returncode, 0)
                        refused = payload((await client.call_tool("native_result_inspect", arguments)).structured_content)
                        self.assertEqual(refused["code"], "pse.native.retired_session_cursor")
                        self.assertEqual(owned_sessions(lifetime), [old])
                        _arguments, _page, new = await self.first_result_page(client, lifetime, selection)
                        new_worker = AuthenticatedReadWorker(new)
                        workers.append(new_worker)
                        self.assertNotEqual(new.incarnation, old.incarnation)
                        self.assertNotEqual(new_worker.pid, old_worker.pid)
                        new_worker.require_image(second)
                    await new_worker.require_settled()
                    self.assertTrue(lifetime.closed)
                    async with Client(endpoint.server) as client:
                        successor = tool.read_lifetime()
                        self.assertIsNot(successor, lifetime)
                        _arguments, _page, attached = await self.first_result_page(client, successor, selection)
                        attached_worker = AuthenticatedReadWorker(attached)
                        workers.append(attached_worker)
                        self.assertNotEqual(attached.incarnation, new.incarnation)
                        attached_worker.require_image(second)
                    await attached_worker.require_settled()
                finally:
                    await native.close_bound(endpoint.catalogue.tools)
                    for worker in workers:
                        worker.close()

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
    async def test_foreground_cli_two_generations_keep_successive_pages_then_eof_drains_and_reconnects(self) -> None:
        selection = CanonicalReadSelection.selected()
        with tempfile.TemporaryDirectory(prefix="pse-cli-read-generations-") as directory:
            generations = PublishedReadGenerations(ROOT, Path(directory))
            first = await asyncio.to_thread(generations.publish)
            second = None
            for attachment in range(2):
                with tempfile.TemporaryFile() as errors:
                    process = await asyncio.create_subprocess_exec(
                        str(ROOT / ".venv/bin/python"), "-m", "scripts.capability_cli", "session",
                        cwd=ROOT, env={**os.environ, "OTEL_SDK_DISABLED": "true", "_PSE_NATIVE_PUBLICATION": str(generations.receipt)},
                        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=errors,
                    )
                    assert process.stdin is not None
                    assert process.stdout is not None
                    worker = None
                    closing = None
                    try:
                        connected = json.loads(await process.stdout.readline())
                        self.assertEqual(connected["session"], "connected")
                        arguments = copy.deepcopy(selection.arguments)
                        request = arguments["request"]
                        assert isinstance(request, dict)
                        request["view"] = {"kind": "schema", "offset": 0, "limit": 1}
                        request["cursor"] = None

                        async def page(request_id: str) -> dict[str, object]:
                            assert process.stdin is not None
                            assert process.stdout is not None
                            process.stdin.write((json.dumps({"id": request_id, "capability": "native_result_inspect", "arguments": arguments}) + "\n").encode())
                            await process.stdin.drain()
                            reply = json.loads(await process.stdout.readline())
                            self.assertEqual(reply["id"], request_id)
                            self.assertFalse(reply["result"]["is_error"])
                            original = payload(reply["result"]["structured_content"])
                            self.assertEqual(original["outcome"], "observed")
                            document = original["document"]
                            assert isinstance(document, dict)
                            return document

                        original = await page("first")
                        worker = AuthenticatedReadWorker.from_foreground(generations.receipt, process.pid)
                        issued = first if attachment == 0 else second
                        assert issued is not None
                        worker.require_image(issued)
                        if attachment == 0:
                            second = await asyncio.to_thread(generations.publish)
                            self.assertNotEqual(second.generation, first.generation)
                        request["cursor"] = JSON_OBJECT.validate_python(original["next_cursor"])
                        advanced = await page("next")
                        self.assertEqual(advanced["offset"], 1)
                        worker.require_image(issued)
                        again = AuthenticatedReadWorker.from_foreground(generations.receipt, process.pid)
                        try:
                            self.assertEqual(again.pid, worker.pid)
                        finally:
                            again.close()
                        if attachment == 0:
                            # Hold the authenticated receiver past the SDK's two-second
                            # disconnect grace. The explicit handshake must keep both
                            # foreground transport and native allocation alive.
                            await worker.stop()
                            process.stdin.close()
                            closing = asyncio.create_task(process.communicate())
                            await asyncio.sleep(2.1)
                            self.assertFalse(closing.done())
                            worker.require_charged()
                    finally:
                        if worker is not None:
                            worker.resume()
                        process.stdin.close()
                        if closing is None:
                            await process.communicate()
                        else:
                            await closing
                        try:
                            self.assertEqual(process.returncode, 0)
                            if worker is not None:
                                await worker.require_settled()
                        finally:
                            if worker is not None:
                                worker.close()

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
