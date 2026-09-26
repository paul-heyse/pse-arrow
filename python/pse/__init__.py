# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Arrow-native process systems engineering core (import name ``pse``).

The distribution is ``pse-arrow``; the import name is ``pse``. Everything that
computes lives in the Rust extension ``pse._native``; this package is the typed,
contract-enforcing boundary around it (blueprint §21).

Import registers the declared Arrow extension types and checks package/native
version and generated registry identity. Exhaustive annotation checks run through
``just python-contracts-check``; dynamic converter hooks check their own classes.

PEP 563 annotations remain prohibited: boundary validators inspect resolved types.

"""

import importlib.metadata

from pse._build import (
    BuildInfo,
    CacheReport,
    CacheSettings,
    DiagnosticAnnotation,
    DiagnosticCause,
    DiagnosticContext,
    DiagnosticNote,
    DiagnosticObservation,
    DiagnosticReport,
    DiagnosticSourceLocation,
    DiagnosticSpan,
    EngineSettings,
    InspectionError,
    ProgressEvent,
    ResourceConsumer,
    ResourceReport,
    SimulationSettings,
    SolveSettings,
    TableName,
    _check_native_compatibility,
    build_info,
    native_version,
)
from pse._inspection import Publication, TableStream, open
from pse._transfer import FieldTransfer
from pse._workflow import (
    CaseBuilder,
    ModelBuilder,
    ModelRevision,
    PhysicalContext,
    PreparedCase,
    PreparedFlow,
    PreparedOperation,
    PreparedStrategy,
    PublicationAttempt,
    PublicationCommitted,
    PublicationConflict,
    PublicationNoncommit,
    PublicationRequest,
    PublicationRoot,
    PublicationSettlement,
    PublicationTicket,
    PublicationUnresolved,
    RunCompletion,
    RunHandle,
    RunResult,
    Runtime,
    SolverCapability,
    StrategyResult,
)
from pse.contracts import extension_types

__all__ = [
    "BuildInfo",
    "CacheReport",
    "CacheSettings",
    "CaseBuilder",
    "DiagnosticAnnotation",
    "DiagnosticCause",
    "DiagnosticContext",
    "DiagnosticNote",
    "DiagnosticObservation",
    "DiagnosticReport",
    "DiagnosticSourceLocation",
    "DiagnosticSpan",
    "EngineSettings",
    "FieldTransfer",
    "InspectionError",
    "ModelBuilder",
    "ModelRevision",
    "PhysicalContext",
    "PreparedCase",
    "PreparedFlow",
    "PreparedOperation",
    "PreparedStrategy",
    "ProgressEvent",
    "Publication",
    "PublicationAttempt",
    "PublicationCommitted",
    "PublicationConflict",
    "PublicationNoncommit",
    "PublicationRequest",
    "PublicationRoot",
    "PublicationSettlement",
    "PublicationTicket",
    "PublicationUnresolved",
    "ResourceConsumer",
    "ResourceReport",
    "RunCompletion",
    "RunHandle",
    "RunResult",
    "Runtime",
    "SimulationSettings",
    "SolveSettings",
    "SolverCapability",
    "StrategyResult",
    "TableName",
    "TableStream",
    "__version__",
    "build_info",
    "open",
]


def _resolve_version() -> str:
    """Resolve the package version, preferring installed metadata.

    Returns:
        The installed ``pse-arrow`` distribution version, or the version the
        extension was built at when the distribution metadata is absent (a
        source tree on ``sys.path`` beside a built extension).
    """
    try:
        return importlib.metadata.version("pse-arrow")
    except importlib.metadata.PackageNotFoundError:
        return native_version()


#: The one version number: the wheel's, the extension's and the workspace's.
__version__: str = _resolve_version()

# Cheap boundary compatibility; exhaustive annotation lint is an explicit check.
_check_native_compatibility(__version__)
extension_types.register_all()
