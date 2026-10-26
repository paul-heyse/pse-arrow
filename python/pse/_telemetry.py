# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit embedded-host composition. Importing pse installs no SDK providers."""

from __future__ import annotations

from typing import TYPE_CHECKING, cast

import msgspec

from pse._build import _telemetry_initialize, _telemetry_shutdown, _telemetry_status
from pse.contracts.documents import SdkLossHealth

if TYPE_CHECKING:
    from collections.abc import Callable


class NativeTelemetryLifetime:
    """Borrow the host incarnation and retain explicit native shutdown ownership."""

    def __init__(self, activation: str) -> None:
        self.activation = msgspec.json.decode(activation, type=dict[str, str | None])
        self.closed = False
        self._final_health: dict[str, object] | None = None

    def shutdown(self) -> None:
        if not self.closed:
            self.closed = True
            if self.activation.get("status") in {"initialized", "disabled"}:
                try:
                    _telemetry_shutdown()
                finally:
                    self._final_health = self._read_loss_health()

    @staticmethod
    def _read_loss_health() -> dict[str, object] | None:
        snapshot = msgspec.json.decode(_telemetry_status(), type=SdkLossHealth | None)
        return (
            cast("dict[str, object]", msgspec.to_builtins(snapshot))
            if snapshot is not None
            else None
        )

    def loss_health(self) -> dict[str, object] | None:
        """Read only an existing native observer; shutdown retains its last snapshot."""
        return self._final_health if self.closed else self._read_loss_health()


def initialize(
    settings: str, carrier_injector: Callable[[], dict[str, str]]
) -> NativeTelemetryLifetime:
    """The native owner samples the injector at execution ingress before detach."""
    return NativeTelemetryLifetime(_telemetry_initialize(settings, carrier_injector))
