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
    CacheSettings,
    DiagnosticReport,
    EngineSettings,
    InspectionError,
    ModelingDiagnosticSettings,
    ModelingLimits,
    NativeAttempt,
    NativeStrategyAttempt,
    OperationalStore,
    SimulationSettings,
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
    ModelingKnowledge,
    ModelingNativeAnalysis,
    ModelingNonlinearExplanation,
    ModelingPackage,
    ModelingResult,
    ModelingTrajectory,
    StudyReport,
)
from pse._runs import (
    Completion,
    PreparedOperation,
    ProgressStream,
    PublicationAttempt,
    PublicationTicket,
    Published,
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
    BuildInfo,
    CacheReport,
    ClarabelSettings,
    ConformanceControls,
    ConicRequest,
    DiagnosticAnnotationDocument,
    DiagnosticCauseDocument,
    DiagnosticContextDocument,
    DiagnosticNoteDocument,
    DiagnosticSamplesControls,
    DiagnosticSpanDocument,
    DiffsolSettings,
    EligibilityDocument,
    ExportReceipt,
    FitPreparationDocument,
    FitProfileDocument,
    FlowConnectionDocument,
    FlowGraphDocument,
    FlowSelectionDocument,
    HighsSettings,
    IdasSettings,
    IncumbentDocument,
    IneligibleDocument,
    InitializationDocument,
    InitializationOverrides,
    InventoryControls,
    IpoptSettings,
    JacobianDiagnosticControls,
    KinsolSettings,
    KnowledgeControls,
    LinearDiagnosticControls,
    ModelingDiagnosticPolicy,
    ModelingInspection,
    ModelingNonlinearPolicy,
    NumericalStrategy,
    PounceSettings,
    ProgressControls,
    ProgressEventDocument,
    ProgressMetricDocument,
    PublicationSettlement,
    PublicationSettlementCommitted,
    PublicationSettlementConflict,
    PublicationSettlementProvedNoncommit,
    PublicationSettlementUnresolved,
    PureConformanceControls,
    RecycleRequest,
    ResourceConsumer,
    ResourceReport,
    RouteDocument,
    RunControls,
    ScipSettings,
    SolveControls,
    SolveSettings,
    SourceLocation,
    StudyRunControls,
    StudySubmitControls,
    StudyWaitControls,
    TableName,
    TearSelectionDocument,
    WarmStartSnapshot,
)

__all__ = [
    "AdjointSettings",
    "BackendSettings",
    "BuildInfo",
    "CacheReport",
    "CacheSettings",
    "ClarabelSettings",
    "Completion",
    "ConformanceControls",
    "ConicRequest",
    "DiagnosticAnnotationDocument",
    "DiagnosticCauseDocument",
    "DiagnosticContextDocument",
    "DiagnosticNoteDocument",
    "DiagnosticReport",
    "DiagnosticSamplesControls",
    "DiagnosticSpanDocument",
    "DiffsolSettings",
    "EligibilityDocument",
    "EngineSettings",
    "ExportReceipt",
    "FieldTransfer",
    "FitPreparationDocument",
    "FitProfileDocument",
    "FlowConnectionDocument",
    "FlowGraphDocument",
    "FlowSelectionDocument",
    "HighsSettings",
    "IdasSettings",
    "IncumbentDocument",
    "IneligibleDocument",
    "InitializationDocument",
    "InitializationOverrides",
    "InspectionError",
    "InventoryControls",
    "IpoptSettings",
    "JacobianDiagnosticControls",
    "KinsolSettings",
    "KnowledgeControls",
    "LinearDiagnosticControls",
    "ModelingConformance",
    "ModelingDiagnosticPolicy",
    "ModelingDiagnosticSamples",
    "ModelingDiagnosticSettings",
    "ModelingDiagnostics",
    "ModelingElasticAttempt",
    "ModelingInitialization",
    "ModelingInitializationAttempt",
    "ModelingInspection",
    "ModelingKnowledge",
    "ModelingLimits",
    "ModelingNativeAnalysis",
    "ModelingNonlinearExplanation",
    "ModelingNonlinearPolicy",
    "ModelingPackage",
    "ModelingResult",
    "ModelingTrajectory",
    "NativeAttempt",
    "NativeStrategyAttempt",
    "NumericalStrategy",
    "OperationalAttempt",
    "OperationalJob",
    "OperationalStore",
    "OperationalStudy",
    "PhysicalContext",
    "PounceSettings",
    "PreparedFlow",
    "PreparedOperation",
    "PreparedStrategy",
    "ProgressControls",
    "ProgressEventDocument",
    "ProgressMetricDocument",
    "ProgressStream",
    "Publication",
    "PublicationAttempt",
    "PublicationSettlement",
    "PublicationSettlementCommitted",
    "PublicationSettlementConflict",
    "PublicationSettlementProvedNoncommit",
    "PublicationSettlementUnresolved",
    "PublicationTicket",
    "Published",
    "PureConformanceControls",
    "RecycleRequest",
    "ResourceConsumer",
    "ResourceReport",
    "RouteDocument",
    "RunControls",
    "RunHandle",
    "RunResult",
    "Runtime",
    "ScipSettings",
    "SimulationSettings",
    "SolveControls",
    "SolveSettings",
    "SolverCapability",
    "SourceLocation",
    "StrategyResult",
    "StudyCancel",
    "StudyHandle",
    "StudyPointStatus",
    "StudyReport",
    "StudyRunControls",
    "StudyStatus",
    "StudySubmitControls",
    "StudyWaitControls",
    "TableName",
    "TableStream",
    "TearSelectionDocument",
    "WarmStartSnapshot",
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
