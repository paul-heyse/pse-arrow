# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit embedded-host composition. Importing pse installs no SDK providers."""

from __future__ import annotations

from collections.abc import Callable, Mapping

import msgspec

from pse._build import _telemetry_initialize, _telemetry_shutdown


class NativeTelemetryLifetime:
    """Borrow the host incarnation and retain explicit native shutdown ownership."""

    def __init__(self, activation: str) -> None:
        self.activation = msgspec.json.decode(activation, type=dict[str, str | None])
        self.closed = False

    def shutdown(self) -> None:
        if not self.closed:
            self.closed = True
            if self.activation.get("status") in {"initialized", "disabled"}:
                _telemetry_shutdown()


def initialize(settings: Mapping[str, object], carrier_injector: Callable[[], dict[str, str]]) -> NativeTelemetryLifetime:
    """The native owner samples the injector at execution ingress before detach."""
    return NativeTelemetryLifetime(_telemetry_initialize(msgspec.json.encode(dict(settings)).decode(), carrier_injector))
