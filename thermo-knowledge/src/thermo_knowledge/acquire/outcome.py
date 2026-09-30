# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What an acquisition kind reports back to the runner after filling `<work>/tree/`."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class Outcome:
    """The resolved pin and the facts `ACQUISITION.json` records besides the file inventory."""

    pin: str
    resolved: str | None
    urls: list[str]
    tool_versions: dict[str, str] = field(default_factory=dict)
    details: dict[str, str] = field(default_factory=dict)
