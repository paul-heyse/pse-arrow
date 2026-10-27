# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Attach an owner-published generation once and retain it through native drain."""

from __future__ import annotations

import hashlib
import json
import os
import sys
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from scripts.python_extension import ExtensionArtifact, ExtensionCurrency


class ReadImageChangedError(ValueError):
    """A requested read cannot use this attached generation."""


@dataclass(frozen=True)
class LoadedReadImage:
    checkout: str
    interpreter: str
    artifact: ExtensionArtifact
    generation: str
    mode: str
    profile: str
    validated: bool

    @classmethod
    def admit(cls, current: ExtensionCurrency) -> LoadedReadImage:
        # Publication established construction validity. This boundary only selects
        # the requested correctness contract; it does not inspect artifact bytes.
        if (
            current.state != "current"
            or current.artifact is None
            or current.generation is None
            or current.profile is None
        ):
            raise ReadImageChangedError("native read lacks a compatible publication")
        return cls(
            current.checkout,
            current.interpreter,
            current.artifact.model_copy(deep=True),
            current.generation,
            current.mode,
            current.profile,
            current.validated,
        )

    def attach(self) -> None:
        """Select the exact issued path before the package's native import gate."""
        if "pse._native" in sys.modules:
            raise ReadImageChangedError("native image already belongs to another attachment")
        os.environ["_PSE_NATIVE_PUBLISHED_IMAGE"] = self.artifact.binary
        try:
            __import__("pse")
        finally:
            os.environ.pop("_PSE_NATIVE_PUBLISHED_IMAGE", None)

    def require(self, *, native: bool, validated: bool) -> None:
        """Compare live request requirements with this in-memory attachment."""
        if (native and self.mode != "native") or (validated and not self.validated):
            raise ReadImageChangedError("requested read needs a different artifact contract")

    def context_identity(self, supplied_context: str) -> str:
        return hashlib.sha256(
            json.dumps(
                {
                    "supplied_context": supplied_context,
                    "checkout": self.checkout,
                    "interpreter": self.interpreter,
                    "generation": self.generation,
                    "mode": self.mode,
                    "profile": self.profile,
                    "validated": self.validated,
                },
                sort_keys=True,
                allow_nan=False,
            ).encode()
        ).hexdigest()
