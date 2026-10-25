# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Settle independent endpoint resource owners before reporting teardown failure."""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Awaitable


async def settle(*owners: Awaitable[None]) -> None:
    """Cancellation of the observer never abandons an already selected cleanup."""
    joined = asyncio.gather(*owners, return_exceptions=True)
    cancellation: asyncio.CancelledError | None = None
    while not joined.done():
        try:
            await asyncio.shield(joined)
        except asyncio.CancelledError as error:
            cancellation = error
    failures = [value for value in joined.result() if isinstance(value, BaseException)]
    failure = (
        BaseExceptionGroup("Endpoint resource cleanup failed", failures)
        if failures
        else None
    )
    if cancellation is not None:
        raise cancellation from failure
    if failure is not None:
        raise failure
