# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit research versions and publication over original scientific owners."""
from pse._build import _NativeRuntime
from pse._modeling import ModelingPackage
from pse.contracts.documents import (
    ContextSelection,
    ContextSelectionOutput,
    ResearchDatasetOutput,
    ResearchDatasetRequest,
    ResearchPublicationIntent,
    ResearchPublicationIntentOutput,
    ResearchPublication,
    ResearchPublicationPreviewOutput,
    ResearchPublicationRequest,
    ResearchReadRequest,
    ResearchReadResultOutput,
    ResearchSelection,
    ResearchSelectionRequest,
)


class ResearchRuntime:
    """Typed facade composed into the ordinary Runtime; no separate engine."""
    _handle: _NativeRuntime

    def research_dataset_create(self, request: ResearchDatasetRequest) -> ResearchDatasetOutput:
        """Admit original typed attribute references into an immutable version."""
        return self._handle.research_dataset_create(request)

    def research_selection_create(self, request: ResearchSelectionRequest) -> ResearchSelection:
        """Freeze complete original observations, exclusions and calibration context."""
        return self._handle.research_selection_create(request)

    def research_read(self, request: ResearchReadRequest) -> ResearchReadResultOutput:
        """Read bounded metadata without granting measurement or result lifetime."""
        return self._handle.research_read(request)

    def research_dataset_open(self, key: str, context: ContextSelection | ContextSelectionOutput) -> ModelingPackage:
        """Reopen selected original typed data through source protections."""
        return ModelingPackage(self._handle.research_dataset_open(key, context))

    def research_selection_open(self, key: str) -> ModelingPackage:
        """Reopen exact fitting inputs carrying their original research ancestry."""
        return ModelingPackage(self._handle.research_selection_open(key))

    def research_publication_preview(self, request: ResearchPublicationRequest) -> ResearchPublicationPreviewOutput:
        """Inspect an admitted authored delta from original qualified fitted cells."""
        return self._handle.research_publication_preview(request)

    def research_publish(self, intent: ResearchPublicationIntent | ResearchPublicationIntentOutput) -> ResearchPublication:
        """Settle or execute one retained exact source publication intent."""
        return self._handle.research_publish(intent)

    def research_publication_open(self, key: str) -> ModelingPackage:
        """Open the admitted parameter source with ancestry for downstream studies."""
        return ModelingPackage(self._handle.research_publication_open(key))
