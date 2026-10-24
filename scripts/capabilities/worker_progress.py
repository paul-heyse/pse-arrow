# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Private foreground observations; native owners retain identity and settlement."""

from __future__ import annotations

import asyncio
import json
import os
import tempfile
from pathlib import Path
from typing import Literal

from pydantic import Field

from scripts.capabilities.model import Contract

MAX_PROGRESS_BYTES = 16 * 1024
MAX_REFERENCES = 8
POLL_SECONDS = 0.25


class OwnerReference(Contract):
    kind: Literal["invocation", "assessment", "diagnostic", "archive"]
    locator: str = Field(min_length=1, max_length=4096)
    registry: str | None = Field(default=None, max_length=4096)
    ownership: Literal["owned", "borrowed"] | None = None


class WorkerProgress(Contract):
    version: Literal[1] = 1
    operation: str = Field(min_length=1, max_length=96)
    sequence: int = Field(ge=1)
    stage: Literal["environment", "extension", "stubs", "admit", "import", "request", "assessment", "execution", "native", "archive", "diagnostic"]
    state: Literal["started", "completed", "failed"]
    references: tuple[OwnerReference, ...] = Field(default=(), max_length=MAX_REFERENCES)
    references_complete: bool = True


class ProgressReporter:
    """One request-private latest snapshot, never a durable run/status authority."""

    def __init__(self, path: Path, operation: str):
        self.path = path
        self.operation = operation
        self.sequence = 0
        self.references: list[OwnerReference] = []
        self.references_complete = True

    def publish(
        self,
        stage: str,
        state: Literal["started", "completed", "failed"],
        *,
        reference: OwnerReference | None = None,
    ) -> None:
        if reference is not None and reference not in self.references:
            if len(self.references) < MAX_REFERENCES:
                self.references.append(reference)
            else:
                self.references_complete = False
        self.sequence += 1
        value = WorkerProgress.model_validate({
            "operation": self.operation, "sequence": self.sequence,
            "stage": stage, "state": state, "references": tuple(self.references),
            "references_complete": self.references_complete,
        })
        raw = value.model_dump_json().encode()
        if len(raw) > MAX_PROGRESS_BYTES:
            # Keep the stable original reference. Additional child identities stay
            # available through that original owner's bounded readers.
            value = value.model_copy(update={"references": tuple(self.references[:1]), "references_complete": False})
            raw = value.model_dump_json().encode()
        if len(raw) > MAX_PROGRESS_BYTES:
            return
        try:
            with tempfile.NamedTemporaryFile(dir=self.path.parent, prefix=".progress-", delete=False) as stream:
                temporary = Path(stream.name)
                try:
                    stream.write(raw)
                    stream.flush()
                    temporary.replace(self.path)
                finally:
                    temporary.unlink(missing_ok=True)
        except OSError:
            # Loss of this optional observation cannot fail an admitted kernel.
            pass


def read(path: Path, operation: str) -> WorkerProgress | None:
    """Read a bounded complete atomic snapshot from this exact private exchange."""
    try:
        with path.open("rb") as source:
            raw = source.read(MAX_PROGRESS_BYTES + 1)
    except FileNotFoundError:
        return None
    if len(raw) > MAX_PROGRESS_BYTES:
        raise ValueError("worker progress exceeds its bound")
    value = WorkerProgress.model_validate_json(raw)
    if value.operation != operation:
        raise ValueError("worker progress belongs to another operation")
    return value


class ProgressObserver:
    """Coalesce request-local snapshots; notification loss is not kernel failure."""

    def __init__(self, path: Path, operation: str):
        from fastmcp.server.dependencies import get_context  # noqa: PLC0415 -- optional request context.

        self.path = path
        self.operation = operation
        self.sequence = 0
        self.latest: WorkerProgress | None = None
        self.omissions: list[str] = []
        try:
            self.context = get_context()
        except RuntimeError:
            self.context = None

    async def update(self) -> None:
        try:
            value = read(self.path, self.operation)
        except (OSError, ValueError):
            if "worker progress unavailable" not in self.omissions:
                self.omissions.append("worker progress unavailable")
            return
        if value is None or value.sequence <= self.sequence:
            return
        self.sequence = value.sequence
        self.latest = value
        if self.context is not None:
            try:
                await asyncio.wait_for(self.context.report_progress(
                    progress=value.sequence, total=None, message=value.model_dump_json(),
                ), timeout=0.5)
            except asyncio.CancelledError:
                raise
            except Exception:
                if "progress notification unavailable" not in self.omissions:
                    self.omissions.append("progress notification unavailable")
                self.context = None

    async def until(self, waiting: asyncio.Task[int]) -> None:
        while not waiting.done():
            await self.update()
            await asyncio.wait({waiting}, timeout=POLL_SECONDS)
        await self.update()

    def recovery(self) -> dict[str, object]:
        return {
            "references": [reference.model_dump(mode="json") for reference in self.latest.references] if self.latest else [],
            "observation_omissions": self.omissions,
            "settlement": "Original owner readers retain terminal and drain authority; progress is observation only.",
        }
