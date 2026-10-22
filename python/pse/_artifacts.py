# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent scientific selections with exact attributed lifetimes."""

import attrs

from pse._build import _NativeArtifactSettlement, _NativeScientificArtifact
from pse._inspection import TableStream
from pse.contracts.documents import ArtifactIntent
from pse.contracts.enums import ArtifactOwnerKind, ScientificArtifactKind


@attrs.frozen
class ScientificArtifact:
    """Retained content and original qualification, independent of its producer."""

    _handle: _NativeScientificArtifact

    @property
    def key(self) -> str:
        """Exact original occurrence and settlement identity."""
        return self._handle.key

    @property
    def kind(self) -> ScientificArtifactKind:
        """Scientific product promised by its original promotion owner."""
        return self._handle.kind

    @property
    def original_intent(self) -> ArtifactIntent:
        """Exact original settlement document, also retained by errors and interrupts."""
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
    """Exact recovery intent for release/collection, never content access."""

    _handle: _NativeArtifactSettlement

    @property
    def key(self) -> str:
        """Original occurrence, including a possibly absent or inactive header."""
        return self._handle.key

    def release(self, kind: ArtifactOwnerKind, owner: str) -> None:
        """Release only the explicit owner after immutable original equality checks."""
        self._handle.release(kind, owner)

    def collect(self) -> bool:
        """Settle one page; false leaves original creation/read windows pending."""
        return self._handle.collect()
