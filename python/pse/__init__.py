# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Arrow-native process systems engineering core (import name ``pse``).

The distribution is ``pse-arrow``; the import name is ``pse``. Everything that
computes lives in the Rust extension ``pse._native``; this package is the typed,
contract-enforcing boundary around it (blueprint §21).

Import registers the declared Arrow extension types and checks package/native
version and generated registry identity. Exhaustive annotation checks run through
``just python-contracts-check``; dynamic converter hooks check their own classes.

Settings are the generated msgspec document types of ``pse.contracts.documents``;
native entry points receive their JSON encoding (ADR-0116).

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
    Incumbent,
    InspectionError,
    ModelingDiagnosticSettings,
    ModelingEventSettings,
    ModelingFixturePolicy,
    ModelingLimits,
    ModelingModeSettings,
    NativeAttempt,
    NativeEligibility,
    NativeIneligible,
    NativeRoute,
    NativeStrategyAttempt,
    OperationalStore,
    ProgressEvent,
    ResourceConsumer,
    ResourceReport,
    SimulationSettings,
    TableName,
    _check_native_compatibility,
    build_info,
    native_version,
)
from pse._inspection import Publication, TableStream, open_export
from pse._modeling import (
    ModelingConformance,
    ModelingDiagnostics,
    ModelingDiagnosticSamples,
    ModelingElasticAttempt,
    ModelingInitialization,
    ModelingInitializationAttempt,
    ModelingNativeAnalysis,
    ModelingNonlinearExplanation,
    ModelingPackage,
    ModelingResult,
    ModelingStudy,
    ModelingTrajectory,
)
from pse._runs import (
    ExportReceipt,
    PreparedOperation,
    ProgressStream,
    PublicationAttempt,
    PublicationCommitted,
    PublicationConflict,
    PublicationNoncommit,
    PublicationSettlement,
    PublicationTicket,
    PublicationUnresolved,
    Published,
    RunCompletion,
    RunHandle,
    RunResult,
    StudyCancel,
    StudyHandle,
    StudyPointStatus,
    StudyStatus,
    Workspace,
)
from pse._strategies import PreparedFlow, PreparedStrategy, StrategyResult
from pse._transfer import FieldTransfer
from pse._workflow import (
    OperationalAttempt,
    OperationalJob,
    OperationalStudy,
    PhysicalContext,
    Runtime,
    SolverCapability,
)
from pse.contracts import extension_types
from pse.contracts.documents import (
    AdjointSettings,
    BackendSettings,
    ClarabelSettings,
    DiffsolSettings,
    HighsSettings,
    IdasSettings,
    IpoptSettings,
    KinsolSettings,
    PounceSettings,
    ScipSettings,
    SolveControls,
    SolveSettings,
)

__all__ = [
    "AdjointSettings",
    "BackendSettings",
    "BuildInfo",
    "CacheReport",
    "CacheSettings",
    "ClarabelSettings",
    "DiagnosticAnnotation",
    "DiagnosticCause",
    "DiagnosticContext",
    "DiagnosticNote",
    "DiagnosticObservation",
    "DiagnosticReport",
    "DiagnosticSourceLocation",
    "DiagnosticSpan",
    "DiffsolSettings",
    "EngineSettings",
    "ExportReceipt",
    "FieldTransfer",
    "HighsSettings",
    "IdasSettings",
    "Incumbent",
    "InspectionError",
    "IpoptSettings",
    "KinsolSettings",
    "ModelingConformance",
    "ModelingDiagnosticSamples",
    "ModelingDiagnosticSettings",
    "ModelingDiagnostics",
    "ModelingElasticAttempt",
    "ModelingEventSettings",
    "ModelingFixturePolicy",
    "ModelingInitialization",
    "ModelingInitializationAttempt",
    "ModelingLimits",
    "ModelingModeSettings",
    "ModelingNativeAnalysis",
    "ModelingNonlinearExplanation",
    "ModelingPackage",
    "ModelingResult",
    "ModelingStudy",
    "ModelingTrajectory",
    "NativeAttempt",
    "NativeEligibility",
    "NativeIneligible",
    "NativeRoute",
    "NativeStrategyAttempt",
    "OperationalAttempt",
    "OperationalJob",
    "OperationalStore",
    "OperationalStudy",
    "PhysicalContext",
    "PounceSettings",
    "PreparedFlow",
    "PreparedOperation",
    "PreparedStrategy",
    "ProgressEvent",
    "ProgressStream",
    "Publication",
    "PublicationAttempt",
    "PublicationCommitted",
    "PublicationConflict",
    "PublicationNoncommit",
    "PublicationSettlement",
    "PublicationTicket",
    "PublicationUnresolved",
    "Published",
    "ResourceConsumer",
    "ResourceReport",
    "RunCompletion",
    "RunHandle",
    "RunResult",
    "Runtime",
    "ScipSettings",
    "SimulationSettings",
    "SolveControls",
    "SolveSettings",
    "SolverCapability",
    "StrategyResult",
    "StudyCancel",
    "StudyHandle",
    "StudyPointStatus",
    "StudyStatus",
    "TableName",
    "TableStream",
    "Workspace",
    "__version__",
    "build_info",
    "open_export",
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
