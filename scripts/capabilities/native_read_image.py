# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Retain the original admitted image witness; a later receipt cannot bless an old import."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from scripts.python_extension import ExtensionArtifact, ExtensionCurrency


class ReadImageChangedError(ValueError):
    """A retained worker must retire before another receiver can enter."""


@dataclass(frozen=True)
class LoadedReadImage:
    checkout: str
    interpreter: str
    artifact: ExtensionArtifact
    stamp: tuple[int, int, int, int, int]
    source_basis: str
    mode: str
    profile: str
    validated: bool

    @classmethod
    def admit(cls, current: ExtensionCurrency) -> LoadedReadImage:
        if (
            current.state != "current"
            or current.artifact is None
            or current.artifact_stamp is None
            or current.source_basis is None
            or current.profile is None
            or current.validated is not True
            or "force-validate" not in current.artifact.features
            or (
                (current.mode == "native")
                != ("native-solvers" in current.artifact.features)
            )
        ):
            raise ReadImageChangedError(
                "native read lacks current producing and installed image evidence"
            )
        return cls(
            current.checkout,
            current.interpreter,
            current.artifact.model_copy(deep=True),
            current.artifact_stamp,
            current.source_basis,
            current.mode,
            current.profile,
            current.validated,
        )

    def context_identity(self, supplied_context: str) -> str:
        """Bind actual producing/provider/preparation/image evidence to retained objects."""
        return hashlib.sha256(
            json.dumps(
                {
                    "supplied_context": supplied_context,
                    "checkout": self.checkout,
                    "interpreter": self.interpreter,
                    "source_basis": self.source_basis,
                    "artifact": self.artifact.model_dump(mode="json"),
                    "stamp": self.stamp,
                    "mode": self.mode,
                    "profile": self.profile,
                    "validated": self.validated,
                },
                sort_keys=True,
                allow_nan=False,
            ).encode()
        ).hexdigest()

    def require_current(self, current: ExtensionCurrency) -> None:
        observed = self.admit(current)
        if observed != self:
            raise ReadImageChangedError(
                "native read producing, provider, preparation or image premises changed"
            )

    def require_loaded(self, loaded_file: str) -> None:
        from scripts.python_extension import (
            artifact_stamp,
        )

        try:
            unchanged = (
                Path(loaded_file).resolve(strict=True) == Path(self.artifact.binary)
                and artifact_stamp(self.artifact) == self.stamp
            )
        except (OSError, ValueError):
            unchanged = False
        if not unchanged:
            raise ReadImageChangedError(
                "native read loaded image differs from its original witness"
            )
