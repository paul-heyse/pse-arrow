# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Endpoint owners settle independently and retain teardown failures/cancellation."""

# ruff: noqa: PT009, PT027 -- stdlib host controls
from __future__ import annotations

import asyncio
import unittest
from contextlib import asynccontextmanager
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import AsyncMock, patch

from fastmcp import Client

from scripts.capabilities import cleanup, server, specialists

if TYPE_CHECKING:
    from collections.abc import AsyncGenerator


def leaves(error: BaseException) -> list[BaseException]:
    if isinstance(error, BaseExceptionGroup):
        return [leaf for item in error.exceptions for leaf in leaves(item)]
    return [error]


class CleanupTests(unittest.IsolatedAsyncioTestCase):
    async def test_failed_connection_contexts_both_close_and_keep_original_errors(
        self,
    ) -> None:
        exits = []

        @asynccontextmanager
        async def factory(name: str) -> AsyncGenerator[Client]:
            try:
                yield AsyncMock(spec=Client, list_tools=AsyncMock(return_value=[]))
            finally:
                exits.append(name)
                raise RuntimeError(name)

        owner = specialists.Specialists(Path(__file__).resolve().parents[2])
        owner.languages = {
            name: specialists.Connection(lambda selected=name: factory(selected))
            for name in ("python", "rust")
        }
        for connection in owner.languages.values():
            await connection.invoke(AsyncMock(return_value={}))
        with self.assertRaises(BaseExceptionGroup) as raised:
            await owner.close()
        self.assertEqual(set(exits), {"python", "rust"})
        self.assertEqual(
            {str(error) for error in leaves(raised.exception)}, {"python", "rust"}
        )
        self.assertTrue(
            all(
                connection.task is not None and connection.task.done()
                for connection in owner.languages.values()
            )
        )

    async def test_repeated_cancellation_settles_every_owner_before_repropagation(
        self,
    ) -> None:
        started = asyncio.Event()
        release = asyncio.Event()
        settled = []

        async def close(name: str) -> None:
            started.set()
            await release.wait()
            settled.append(name)
            raise ValueError(name)

        closing = asyncio.create_task(
            cleanup.settle(close("telemetry"), close("language"))
        )
        await started.wait()
        for _ in range(2):
            closing.cancel()
            await asyncio.sleep(0)
            self.assertFalse(closing.done())
        release.set()
        with self.assertRaises(asyncio.CancelledError) as raised:
            await closing
        self.assertEqual(set(settled), {"telemetry", "language"})
        self.assertIsInstance(raised.exception.__cause__, BaseExceptionGroup)
        self.assertEqual(len(leaves(raised.exception.__cause__)), 2)

    async def test_actual_endpoint_lifespan_closes_specialists_after_backend_failure(
        self,
    ) -> None:
        backend = AsyncMock()
        backend.close.side_effect = RuntimeError("backend teardown")
        with patch.object(server, "BackendClient", return_value=backend):
            runtime = server.build()
        original_close = runtime.specialists.close
        closed = []

        async def close() -> None:
            await original_close()
            closed.append(True)

        with (
            patch.object(runtime.specialists, "close", side_effect=close),
            self.assertRaises(BaseExceptionGroup) as raised,
        ):
            async with runtime.server._lifespan_manager():  # noqa: SLF001 -- actual endpoint lifecycle teardown
                pass
        self.assertEqual(closed, [True])
        self.assertEqual(
            [str(error) for error in leaves(raised.exception)], ["backend teardown"]
        )
