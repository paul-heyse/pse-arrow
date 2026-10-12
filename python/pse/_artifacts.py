# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent scientific selections with exact attributed lifetimes."""

from typing import Literal

import attrs

from pse._build import _NativeScientificArtifact, _NativeArtifactSettlement
from pse._inspection import TableStream

ScientificArtifactKind = Literal["final-state", "trajectory", "local-analysis"]
ArtifactOwnerKind = Literal["user", "project", "audit"]


@attrs.frozen
class ScientificArtifact:
    """Retained content and original qualification, independent of its producer."""

    _handle: _NativeScientificArtifact

    @property
    def key(self) -> str:
        """Exact original occurrence and settlement identity."""
        return self._handle.key

    @property
    def kind(self) -> str:
        """Scientific product promised by its original promotion owner."""
        return self._handle.kind

    @property
    def original_intent(self) -> bytes:
        """Exact original settlement bytes, also retained by errors and interrupts."""
        return self._handle.original_intent

    def metadata(self) -> TableStream:
        """Observe active metadata, original provenance and member extent."""
        return TableStream(self._handle.metadata())

    def block(self, ordinal: int) -> TableStream:
        """Read one independent IPC member; seed capsules use artifact starts."""
        return TableStream(self._handle.block(ordinal))

    def retain(self, kind: ArtifactOwnerKind, owner: str) -> None:
        """Acquire a distinct owner identity; a released identity cannot resurrect."""
        self._handle.retain(kind, owner)

    def release(self, kind: ArtifactOwnerKind, owner: str) -> None:
        """Irreversibly release only this exact owner identity."""
        self._handle.release(kind, owner)

    def collect(self) -> bool:
        """Settle one bounded page using the original immutable creation intent."""
        return self._handle.collect()


@attrs.frozen
class ArtifactSettlement:
    """Exact original recovery intent; grants release/collection, never content access."""

    _handle: _NativeArtifactSettlement

    @property
    def key(self) -> str:
        """Original occurrence, including a possibly absent or inactive header."""
        return self._handle.key

    def release(self, kind: ArtifactOwnerKind, owner: str) -> None:
        """Release only the explicit owner after immutable original equality checks."""
        self._handle.release(kind, owner)

    def collect(self) -> bool:
        """Settle one page; false leaves the original finite creation/read windows pending."""
        return self._handle.collect()
