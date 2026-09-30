# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The effects the acquisition stage depends on, injectable so tests do not sleep or wait."""

from __future__ import annotations

import time
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path

import httpx

from thermo_knowledge import __version__, config

USER_AGENT = f"thermo-knowledge/{__version__} (pse-arrow; rate-limited personal research retrieval)"
"""Names the project and the purpose; carries no personal contact details."""

CONNECT_TIMEOUT_SECONDS = 30.0
READ_TIMEOUT_SECONDS = 120.0


def make_client() -> httpx.Client:
    """An HTTP client that does not follow redirects itself: each kind decides how."""
    return httpx.Client(
        headers={"User-Agent": USER_AGENT},
        timeout=httpx.Timeout(READ_TIMEOUT_SECONDS, connect=CONNECT_TIMEOUT_SECONDS),
        follow_redirects=False,
    )


def utc_now() -> datetime:
    return datetime.now(UTC)


def format_time(moment: datetime) -> str:
    """UTC time as `2026-09-30T12:00:00Z`."""
    return moment.astimezone(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


@dataclass(frozen=True)
class Runtime:
    """Clock, sleep, wall time and HTTP client of one acquisition run."""

    clock: Callable[[], float] = time.monotonic
    sleep: Callable[[float], None] = time.sleep
    now: Callable[[], datetime] = utc_now
    client_factory: Callable[[], httpx.Client] = make_client


@dataclass(frozen=True)
class Context:
    """Where an acquisition run reads manifests' effects from and writes them to."""

    raw_dir: Path = field(default_factory=config.raw_dir)
    repo_root: Path = config.REPO_ROOT
    runtime: Runtime = field(default_factory=Runtime)
