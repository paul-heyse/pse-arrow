# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Deliberate wrong-shape invocation for runtime boundary refusal controls."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable


def invalid_call(
    target: Callable[..., object], *args: object, **kwargs: object
) -> object:
    """Pass the selected invalid values unchanged to the actual public boundary."""
    return target(*args, **kwargs)
