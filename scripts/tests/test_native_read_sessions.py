# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent foreground ownership, framing and request association controls."""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import asyncio
import hashlib
import io
import struct
import tempfile
import types
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, cast
from unittest.mock import patch

from scripts.capabilities import execution_basis
from scripts.capabilities.native_read_protocol import (
    MAXIMUM_REQUEST_FRAME_BYTES,
    ReadRequest,
    ReadResponse,
    SessionProtocolError,
    decode,
    encoded,
    operation_document,
    read,
)
from scripts.capabilities.native_read_sessions import (
    ReadSession,
    ReadSessions,
    page_basis,
)
from scripts.capabilities.workers import Worker
from scripts.producing_currency import Witness

if TYPE_CHECKING:
    from pydantic import JsonValue

    from scripts.capabilities.native_contracts import Binding


def binding(*, kind: str = "opaque") -> Binding:
    return cast(
        "Binding",
        types.SimpleNamespace(
            operation="fixture_read",
            read_eligibility="exact_context",
            prepare_import=False,
            requirements=(),
            context_fields=("physical", "documents"),
            continuations=(
                types.SimpleNamespace(
                    input=["request", "cursor"], output=["document", "next"], kind=kind
                ),
            ),
        ),
    )


def arguments(cursor: str | int | None = None) -> dict[str, JsonValue]:
    return {
        "physical": "original-physical",
        "documents": "original-source",
        "request": {"query": "original-query", "limit": 3, "cursor": cursor},
    }


def request() -> ReadRequest:
    operation, supplied = "fixture_read", arguments()
    raw = operation_document(operation, supplied)
    return ReadRequest(
        session="a" * 32,
        request="b" * 32,
        context="c" * 64,
        operation=operation,
        arguments=supplied,
        admission=execution_basis.Handoff(
            definition=execution_basis.Definition(
                checkout="/fixture",
                operation=operation,
                wire_revision="d" * 64,
                sources={},
            ),
            worker="native-read",
            operation=operation,
            route_revision="e" * 64,
            request_sha256=hashlib.sha256(raw).hexdigest(),
        ),
    )


class ProtocolTests(unittest.TestCase):
    def test_frames_refuse_extent_duplicates_truncation_and_cross_request_response(
        self,
    ) -> None:
        original = request()
        framed = encoded(original, MAXIMUM_REQUEST_FRAME_BYTES)
        self.assertEqual(
            read(io.BytesIO(framed), ReadRequest, MAXIMUM_REQUEST_FRAME_BYTES), original
        )
        self.assertIsNone(read(io.BytesIO(), ReadRequest, MAXIMUM_REQUEST_FRAME_BYTES))
        with self.assertRaises(EOFError):
            read(io.BytesIO(framed[:-1]), ReadRequest, MAXIMUM_REQUEST_FRAME_BYTES)
        with self.assertRaises(SessionProtocolError):
            read(
                io.BytesIO(struct.pack("!I", MAXIMUM_REQUEST_FRAME_BYTES + 1)),
                ReadRequest,
                MAXIMUM_REQUEST_FRAME_BYTES,
            )
        with self.assertRaises(SessionProtocolError):
            decode(
                b'{"session":"a","session":"b"}',
                ReadRequest,
                MAXIMUM_REQUEST_FRAME_BYTES,
            )
        response = ReadResponse(
            session=original.session,
            request=original.request,
            context=original.context,
            deadline_monotonic_ns=None,
            request_sha256=original.admission.request_sha256,
            result={"outcome": "observed"},
        )
        response.require_request(original)
        for changed in (
            {"request": "f" * 32},
            {"session": "f" * 32},
            {"context": "f" * 64},
            {"request_sha256": "f" * 64},
            {"deadline_monotonic_ns": 123},
        ):
            with self.assertRaises(SessionProtocolError):
                response.model_copy(update=changed).require_request(original)

    def test_original_deadline_and_page_basis_are_preserved(self) -> None:
        original = request().model_copy(update={"deadline_monotonic_ns": 12345})
        with (
            patch(
                "scripts.capabilities.native_read_protocol.time.monotonic_ns",
                return_value=12345,
            ),
            self.assertRaises(SessionProtocolError),
        ):
            original.require_live_deadline()
        self.assertEqual(
            page_basis(binding(), arguments("issued")),
            page_basis(binding(), arguments(None)),
        )
        changed = arguments("issued")
        changed["request"] = {"query": "changed-query", "limit": 3, "cursor": "issued"}
        self.assertNotEqual(
            page_basis(binding(), changed), page_basis(binding(), arguments("issued"))
        )

    def test_fresh_handoff_verifies_request_and_caches_only_unchanged_scoped_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "owner.py"
            source.write_text("value = 1\n")
            worker = Worker("fixture.worker", frozenset({"fixture_read"}))
            with patch.dict(execution_basis.WORKERS, {"native-read": worker}):
                original = request()
                admitted = original.admission.model_copy(
                    update={
                        "definition": original.admission.definition.model_copy(
                            update={
                                "checkout": str(root),
                                "sources": {
                                    "owner.py": hashlib.sha256(
                                        source.read_bytes()
                                    ).hexdigest()
                                },
                            }
                        ),
                        "route_revision": execution_basis.route_revision("native-read"),
                    }
                )
                witness = Witness()
                with patch("hashlib.file_digest", wraps=hashlib.file_digest) as hashed:
                    execution_basis.verify_handoff(
                        root,
                        "native-read",
                        original.operation,
                        original.original_document(),
                        admitted,
                        witness=witness,
                    )
                    execution_basis.verify_handoff(
                        root,
                        "native-read",
                        original.operation,
                        original.original_document(),
                        admitted,
                        witness=witness,
                    )
                    self.assertEqual(hashed.call_count, 1)
                    with self.assertRaises(execution_basis.AdmissionRefusedError):
                        execution_basis.verify_handoff(
                            root,
                            "native-read",
                            original.operation,
                            b"different request",
                            admitted,
                            witness=witness,
                        )
                    source.write_text("value = 2\n")
                    with self.assertRaises(execution_basis.AdmissionRefusedError):
                        execution_basis.verify_handoff(
                            root,
                            "native-read",
                            original.operation,
                            original.original_document(),
                            admitted,
                            witness=witness,
                        )
                    self.assertEqual(hashed.call_count, 2)


class FakeSession(ReadSession):
    def __init__(self, root: Path, identity: str, events: list[str]) -> None:
        super().__init__(root, identity)
        self.events = events
        self.started = asyncio.Event()
        self.release = asyncio.Event()
        self.release.set()
        self.close_started = asyncio.Event()
        self.close_release = asyncio.Event()
        self.close_release.set()
        self.fail_close = False
        self.offset = False

    async def invoke(
        self,
        selected: Binding,
        supplied: dict[str, JsonValue],
        deadline_monotonic_ns: int | None,
    ) -> ReadResponse:
        self.events.append("start:" + self.incarnation)
        self.started.set()
        await self.release.wait()
        self.events.append("native-complete:" + self.incarnation)
        return ReadResponse(
            session=self.incarnation,
            request="a" * 32,
            context=self.identity,
            request_sha256=hashlib.sha256(
                operation_document(selected.operation, supplied)
            ).hexdigest(),
            deadline_monotonic_ns=deadline_monotonic_ns,
            result={
                "outcome": "observed",
                "document": {
                    "next": 10 if self.offset else "issued-" + self.incarnation
                },
            },
        )

    async def close(self) -> None:
        self.close_started.set()
        await self.close_release.wait()
        self.events.append("close:" + self.incarnation)
        self.retired = True
        if self.fail_close:
            raise RuntimeError("original-settlement-failure")


class SessionTests(unittest.IsolatedAsyncioTestCase):
    def manager(
        self, *, offset: bool = False
    ) -> tuple[ReadSessions, list[FakeSession], list[str]]:
        sessions: list[FakeSession] = []
        events: list[str] = []

        def create(root: Path, identity: str) -> FakeSession:
            session = FakeSession(root, identity, events)
            session.offset = offset
            sessions.append(session)
            return session

        return ReadSessions(Path.cwd(), factory=create), sessions, events

    async def test_worker_log_and_directory_failures_all_survive_independent_cleanup(
        self,
    ) -> None:
        attempts: list[str] = []

        class Process:
            stdin = None
            stdout = None

            async def wait(self) -> int:
                attempts.append("process")
                return 7

        class Log:
            def close(self) -> None:
                attempts.append("log")
                raise OSError("original-log-close-failure")

        class Directory:
            def cleanup(self) -> None:
                attempts.append("directory")
                raise OSError("original-directory-cleanup-failure")

        session = ReadSession(Path.cwd(), "context")
        session._process = cast("asyncio.subprocess.Process", Process())  # noqa: SLF001 -- inject independent owned-resource failures
        session._files.callback(Log().close)  # noqa: SLF001 -- inject exact owned log close failure
        session._temporary = cast("tempfile.TemporaryDirectory[str]", Directory())  # noqa: SLF001 -- inject exact owned directory cleanup failure
        with self.assertRaises(BaseExceptionGroup) as original:
            await session.close()
        self.assertEqual(attempts, ["process", "log", "directory"])
        with self.assertRaises(BaseExceptionGroup) as repeated:
            await session.close()
        self.assertIs(repeated.exception, original.exception)
        self.assertEqual(attempts, ["process", "log", "directory"])

    async def test_successive_pages_reuse_context_and_known_retired_cursor_refuses(
        self,
    ) -> None:
        manager, sessions, _events = self.manager()
        first = await manager.run(binding(), arguments())
        document = first["document"]
        assert isinstance(document, dict)
        assert isinstance(document["next"], str)
        cursor = document["next"]
        await manager.run(binding(), arguments(cursor))
        self.assertEqual(len(sessions), 1)
        sessions[0].retired = True
        refused = await manager.run(binding(), arguments(cursor))
        self.assertEqual(refused["code"], "pse.native.retired_session_cursor")
        self.assertEqual(len(sessions), 1)
        # An original portable cursor unknown to this endpoint still reaches its owner.
        await manager.run(binding(), arguments("portable-original-owner-cursor"))
        self.assertEqual(len(sessions), 2)
        await manager.close()

    async def test_known_cursor_cannot_become_unknown_after_context_or_request_basis_change(
        self,
    ) -> None:
        manager, sessions, _events = self.manager()
        first = await manager.run(binding(), arguments())
        document = first["document"]
        assert isinstance(document, dict)
        assert isinstance(document["next"], str)
        cursor = document["next"]
        changed_context = arguments(cursor)
        changed_context["physical"] = "changed-physical"
        refused = await manager.run(binding(), changed_context)
        self.assertEqual(refused["code"], "pse.native.changed_session_cursor_basis")
        changed_basis = arguments(cursor)
        changed_basis["request"] = {
            "query": "another-original-query",
            "limit": 3,
            "cursor": cursor,
        }
        refused = await manager.run(binding(), changed_basis)
        self.assertEqual(refused["code"], "pse.native.changed_session_cursor_basis")
        self.assertEqual(len(sessions), 1)
        sessions[0].retired = True
        refused = await manager.run(binding(), changed_context)
        self.assertEqual(refused["code"], "pse.native.retired_session_cursor")
        await manager.close()

    async def test_equal_offsets_in_independent_contexts_remain_independent_selectors(
        self,
    ) -> None:
        manager, sessions, _events = self.manager(offset=True)
        selected = binding(kind="offset")
        await manager.run(selected, arguments())
        changed = arguments()
        changed["physical"] = "another-original-physical"
        await manager.run(selected, changed)
        self.assertEqual(len(sessions), 2)
        next_page = {
            **changed,
            "request": {"query": "original-query", "limit": 3, "cursor": 10},
        }
        observed = await manager.run(selected, next_page)
        self.assertEqual(observed["outcome"], "observed")
        self.assertEqual(len(sessions), 2)
        sessions[0].retired = True
        refused = await manager.run(selected, arguments(10))
        self.assertEqual(refused["code"], "pse.native.retired_session_cursor")
        await manager.close()

    async def test_busy_independent_reads_get_sibling_workers_without_serialization(
        self,
    ) -> None:
        manager, sessions, _events = self.manager()
        first = await manager.run(binding(), arguments())
        self.assertIn("document", first)
        sessions[0].release.clear()
        active = asyncio.create_task(manager.run(binding(), arguments()))
        while not sessions[0].busy:
            await asyncio.sleep(0)
        second = await manager.run(binding(), arguments())
        self.assertIn("document", second)
        self.assertEqual(len(sessions), 2)
        sessions[0].release.set()
        await active
        await manager.close()

    async def test_observer_cancel_and_endpoint_eof_wait_for_native_completion(
        self,
    ) -> None:
        manager, sessions, events = self.manager()
        await manager.run(binding(), arguments())
        sessions[0].release.clear()
        active = asyncio.create_task(manager.run(binding(), arguments()))
        while not sessions[0].busy:
            await asyncio.sleep(0)
        active.cancel()
        with self.assertRaises(asyncio.CancelledError):
            await active
        closing = asyncio.create_task(manager.close())
        await asyncio.sleep(0)
        self.assertFalse(closing.done())
        self.assertFalse(sessions[0].close_started.is_set())
        sessions[0].release.set()
        await closing
        self.assertLess(
            events.index("native-complete:" + sessions[0].incarnation),
            events.index("close:" + sessions[0].incarnation),
        )

    async def test_independent_teardowns_all_settle_before_cancellation_is_repropagated(
        self,
    ) -> None:
        manager, sessions, events = self.manager()
        await manager.run(binding(), arguments())
        changed = arguments()
        changed["physical"] = "another-original-physical"
        await manager.run(binding(), changed)
        for session in sessions:
            session.close_release.clear()
        sessions[0].fail_close = True
        closing = asyncio.create_task(manager.close())
        await asyncio.gather(*(session.close_started.wait() for session in sessions))
        closing.cancel()
        closing.cancel()
        for session in sessions:
            session.close_release.set()
        with self.assertRaises(asyncio.CancelledError) as captured:
            await closing
        self.assertIsNotNone(captured.exception.__cause__)
        self.assertEqual(sum(event.startswith("close:") for event in events), 2)


if __name__ == "__main__":
    unittest.main()
