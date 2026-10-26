# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Foreground-owned native objects for one exact immutable read context.

This module imports no product extension. The native receiver supplies typed factories;
we retain their actual objects, never serialized substitutes or a cross-page transaction.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import TypeVar, cast

T = TypeVar("T")


class ReadContextChanged(ValueError):
    """A retained receiver cannot adopt different immutable premises."""


class NativeReadContext:
    """Own one actual Runtime and its physical/modeling products until physical close.

    The session owner serializes access to this particular context. Independent contexts
    have independent owners and remain subject to the ordinary native admission policy.
    """

    def __init__(self, identity: str) -> None:
        if not identity:
            raise ValueError("retained read context requires its admitted identity")
        self.identity = identity
        self._runtime: tuple[str, object] | None = None
        self._close_runtime: Callable[[], None] | None = None
        self._physical: dict[str, object] = {}
        self._modeling: dict[str, object] = {}
        self._closed = False

    @property
    def closed(self) -> bool:
        return self._closed

    def require(self, identity: str) -> None:
        if self._closed or identity != self.identity:
            raise ReadContextChanged("retained native read context changed or retired")

    def runtime(
        self, identity: str, create: Callable[[], T], close: Callable[[T], None]
    ) -> T:
        """Construct once; changed Runtime/deployment/resource identity refuses reuse."""
        self.require(self.identity)
        if self._runtime is not None:
            if self._runtime[0] != identity:
                raise ReadContextChanged("retained native Runtime premises changed")
            return cast("T", self._runtime[1])
        runtime = create()
        self._runtime = (identity, runtime)
        self._close_runtime = lambda: close(runtime)
        return runtime

    def physical(self, identity: str, create: Callable[[], T]) -> T:
        """Reuse the actual physical owner only for identical original source bytes."""
        self.require(self.identity)
        if self._runtime is None:
            raise RuntimeError("physical context requires its owned Runtime")
        if identity not in self._physical:
            self._physical[identity] = create()
        return cast("T", self._physical[identity])

    def modeling(self, identity: str, create: Callable[[], T]) -> T:
        """Reuse actual source/revision preparation under its bound physical context."""
        self.require(self.identity)
        if self._runtime is None:
            raise RuntimeError("modeling context requires its owned Runtime")
        if identity not in self._modeling:
            self._modeling[identity] = create()
        return cast("T", self._modeling[identity])

    def close(self) -> None:
        """Drop retained products, then settle the original Runtime exactly once.

        The foreground worker calls this only after all requests using the objects have
        completed. Closing observation or its pipe does not stand in for this drain.
        """
        if self._closed:
            return
        self._closed = True
        self._modeling.clear()
        self._physical.clear()
        self._runtime = None
        close, self._close_runtime = self._close_runtime, None
        if close is not None:
            close()
