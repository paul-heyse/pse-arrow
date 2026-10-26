# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Supervise native byte progress without imposing a whole-inventory clock."""

from __future__ import annotations

import contextlib
import hashlib
import json
import os
import subprocess
import time
from typing import TYPE_CHECKING

import msgspec

from scripts import host_metadata, resource_store
from scripts.generated import deployment

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path
    from typing import BinaryIO


class RecoveryError(ValueError):
    """Native physical recovery remains closed at its retained checkpoint."""


def checkpoint_path(
    command: Sequence[str], inventory_sha256: str, helper_sha256: str
) -> Path:
    key = hashlib.sha256(
        json.dumps(
            [list(command), inventory_sha256, helper_sha256], separators=(",", ":")
        ).encode()
    ).hexdigest()
    directory = host_metadata.root_path().parent / "payload-recovery"
    resource_store.protected(directory)
    return directory / (key + ".json")


def observation(path: Path) -> deployment.RecoveryCheckpoint | None:

    try:
        descriptor = os.open(
            path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
        )
    except FileNotFoundError:
        return None
    try:
        information = os.fstat(descriptor)
        host_metadata.validate_private_file(information, "native recovery progress")
        if information.st_size > 64 * 1024:
            raise RecoveryError(
                "Native recovery checkpoint exceeds its bounded envelope"
            )
        with os.fdopen(descriptor, "rb", closefd=False) as source:
            raw = source.read(64 * 1024 + 1)
        # A persisted version is mandatory even if generated producers provide a default.
        value = json.loads(raw)
        if not isinstance(value, dict) or "version" not in value:
            raise RecoveryError(
                "Native recovery checkpoint omitted its persisted version"
            )
        return msgspec.json.decode(raw, type=deployment.RecoveryCheckpoint)
    except (msgspec.DecodeError, ValueError) as error:
        raise RecoveryError("Native recovery checkpoint is malformed") from error
    finally:
        os.close(descriptor)


def run(
    command: Sequence[str],
    request: BinaryIO,
    response: BinaryIO,
    errors: BinaryIO,
    checkpoint: Path,
    *,
    deadline: float | None = None,
    stall_seconds: float = 120,
) -> int:
    """Retain caller cancellation/budget and diagnose absence of actual native progress."""
    if not 0 < stall_seconds < float("inf"):
        raise RecoveryError(
            "Native recovery stall interval must be finite and positive"
        )
    host_metadata.remaining(deadline, 1)
    previous = observation(checkpoint)
    original_attempt = previous.attempt if previous is not None else None
    progress: tuple[str, int] | None = None
    last_progress = time.monotonic()
    with subprocess.Popen(
        [*command, "--checkpoint", str(checkpoint)],
        stdin=request,
        stdout=response,
        stderr=errors,
    ) as child:
        try:
            while child.poll() is None:
                host_metadata.remaining(deadline, 1)
                observed = observation(checkpoint)
                if observed is not None and observed.attempt != original_attempt:
                    selected = (observed.attempt, observed.sequence)
                    if progress is not None and (
                        selected[0] != progress[0] or selected[1] < progress[1]
                    ):
                        raise RecoveryError(
                            "Native recovery progress authority changed during the operation"
                        )
                    if selected != progress:
                        progress = selected
                        last_progress = time.monotonic()
                if time.monotonic() - last_progress >= stall_seconds:
                    phase = (
                        observed.phase.value
                        if observed is not None
                        else "awaiting-native-binding"
                    )
                    raise RecoveryError(
                        f"Native recovery stalled during {phase}; checkpoint={checkpoint}; explicit resume required"
                    )
                with contextlib.suppress(subprocess.TimeoutExpired):
                    child.wait(timeout=host_metadata.remaining(deadline, 0.1))
            if child.returncode == 0:
                final = observation(checkpoint)
                if (
                    final is None
                    or final.attempt == original_attempt
                    or final.phase.value != "complete"
                ):
                    raise RecoveryError(
                        f"Native recovery omitted complete physical settlement; checkpoint={checkpoint}"
                    )
            return child.returncode
        finally:
            # The helper owns no descendants. Cancellation settles this exact child
            # before its surrounding lifecycle owner can release the frozen operation.
            if child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait()
