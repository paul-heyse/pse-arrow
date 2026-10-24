# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Resolve private native-host telemetry launch settings through the shared owner."""

from __future__ import annotations

from typing import TYPE_CHECKING

from scripts.telemetry_config import resolve_process

if TYPE_CHECKING:
    from pathlib import Path

SETTINGS = "PSE_NATIVE_TELEMETRY_SETTINGS"
STATUS = "PSE_TELEMETRY_CONFIGURATION_STATUS"


def configure(root: Path, environment: dict[str, str]) -> None:
    """Passive resolution only; no SDK installation or scientific/service setup."""
    try:
        settings = resolve_process(
            "pse-native-host", root=root, environment=environment
        )
    except (OSError, ValueError, UnicodeError):
        environment.pop(SETTINGS, None)
        environment[STATUS] = "initialization_refused"
        return
    wire = settings.wire
    environment[STATUS] = "disabled" if wire.disabled else "configured"
    if wire.disabled:
        environment.pop(SETTINGS, None)
    else:
        # Private carrier is omitted by pse-env --print and --explain.
        # The receiving Rust executable mints its own process incarnation.
        environment[SETTINGS] = settings.native_json()
