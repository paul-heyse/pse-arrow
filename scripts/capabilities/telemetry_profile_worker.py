# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One native-worker action, borrowing only the original capture owner."""

from __future__ import annotations

import signal
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from types import FrameType

    from pse.contracts.documents import CaptureDocumentOutput, CheckpointIdentity


def _same_target(document: CaptureDocumentOutput, expected: CheckpointIdentity) -> bool:
    if not document.targets:
        return False
    target = document.targets[0]
    return (
        target.process == expected.process
        and target.locator.pid == expected.locator.pid
        and expected.locator.start_ticks is not None
        and target.locator.start_ticks == expected.locator.start_ticks
    )


def capture(supplied: dict[str, object]) -> dict[str, object]:
    from pse import (  # noqa: PLC0415 -- only the selected native worker imports science
        codec,
        diagnostics,
    )
    from pse.contracts import documents  # noqa: PLC0415
    from scripts.capabilities.native_contracts import (  # noqa: PLC0415 -- original request owner
        ProfileCaptureRequest,
    )
    from scripts.capabilities.native_worker import decoded  # noqa: PLC0415
    from scripts.capabilities.telemetry_profiles import (  # noqa: PLC0415
        CaptureContext,
    )

    selected = ProfileCaptureRequest.model_validate(supplied, strict=True)
    request = decoded(selected.request, documents.CaptureRequest)
    expected = decoded(selected.expected, documents.CheckpointIdentity)
    # This is original native admission against the same image it returns.
    # CaptureRequest itself has no atomic expected-incarnation field. Its own
    # checkpoint/OperationId checks and CPU liveness remain independently active.
    diagnostics.read_checkpoint(Path(request.checkpoint), expected=expected)
    source = selected.suspect.source
    if source is None or selected.suspect.status not in {"available", "partial"}:
        raise ValueError("an original telemetry source is required")
    context = CaptureContext(
        request=selected.suspect.request,
        source=source,
        status=selected.suspect.status,
        meaning=selected.suspect.meaning,
    )
    previous = signal.getsignal(signal.SIGTERM)
    stopping = False

    def interrupt(_signum: int, _frame: FrameType | None) -> None:
        nonlocal stopping
        if not stopping:
            stopping = True
            # PyO3 join observes this signal, requests native capture stop and
            # drains the capture before returning the interpreter exception.
            raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, interrupt)
    try:
        with diagnostics.capture(request) as session:
            if not _same_target(session.snapshot(), expected):
                raise ValueError(
                    "capture target differs from admitted process incarnation"
                )
            try:
                document = session.wait()
            except KeyboardInterrupt:
                document = session.stop()
            if not _same_target(document, expected):
                raise ValueError(
                    "capture target changed after checkpoint identity admission"
                )
    finally:
        signal.signal(signal.SIGTERM, previous)
    return {
        "outcome": "observed",
        "document": codec.document_builtins(document),
        "telemetry_context": context.model_dump(mode="json"),
    }
