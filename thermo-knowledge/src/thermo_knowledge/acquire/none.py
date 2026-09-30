# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kind `none`: the source is recorded as not acquired, with its reason."""

from __future__ import annotations

from thermo_knowledge.acquire.lock import LockEntry
from thermo_knowledge.acquire.manifest import NoneSpec


def lock_entry(spec: NoneSpec, retrieved: str) -> LockEntry:
    """The lock's record of a source that is not acquired: no pin, no files, the reason."""
    return LockEntry("none", None, retrieved, 0, 0, None, reason=spec.reason)
