# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cooperating source launch through the runtime command owner."""

from __future__ import annotations

from typing import TYPE_CHECKING

from pse._build import _diagnostic_record_launch

if TYPE_CHECKING:
    from pse.contracts.documents import (
        SourceRecordingLaunchDocumentOutput,
        SourceRecordingLaunchRequest,
        SourceRecordingLaunchRequestOutput,
    )


def record_launch(
    request: SourceRecordingLaunchRequest | SourceRecordingLaunchRequestOutput,
) -> SourceRecordingLaunchDocumentOutput:
    """Launch literal argv and retain independent activation and command outcomes."""
    return _diagnostic_record_launch(request)
