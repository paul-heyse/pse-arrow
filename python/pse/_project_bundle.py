# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Offline reader of an exact copied project closure."""

from pathlib import Path

import attrs

from pse._build import _NativeProjectBundle
from pse.contracts.documents import ProjectManifestOutput
from pse.generated.project_operation_defaults import DEFAULT_DOCUMENT_BYTES


@attrs.frozen(init=False)
class ProjectBundle:
    """Offline reader of a copied, digest-checked selected project closure."""

    _handle: _NativeProjectBundle

    def __init__(
        self, path: Path, *, maximum_document_bytes: int = DEFAULT_DOCUMENT_BYTES
    ) -> None:
        object.__setattr__(
            self,
            "_handle",
            _NativeProjectBundle(path, maximum_document_bytes=maximum_document_bytes),
        )

    @property
    def manifest(self) -> ProjectManifestOutput:
        """Read the original manifest without connecting to a service."""
        return self._handle.manifest

    @property
    def digest(self) -> str:
        """Exact copied manifest digest used to bind an import intent."""
        return self._handle.digest

    def member_bytes(
        self, member: str, *, maximum_bytes: int = DEFAULT_DOCUMENT_BYTES
    ) -> bytes:
        """Inspect one bounded selected member, verifying its consumed bytes."""
        return self._handle.member_bytes(member, maximum_bytes=maximum_bytes)
