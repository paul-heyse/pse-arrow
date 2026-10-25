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

from pse._analyses import Analysis
from pse._artifacts import (
    ArtifactOwnerKind,
    ArtifactSettlement,
    ScientificArtifact,
    ScientificArtifactKind,
)
from pse._build import (
    CacheSettings,
    DiagnosticReport,
    EngineSettings,
    InspectionError,
    ModelingDiagnosticSettings,
    ModelingLimits,
    NativeAttempt,
    NativeStrategyAttempt,
    SimulationSettings,
    _check_native_compatibility,
    build_info,
    native_version,
)
from pse._inspection import TableStream, registry_table
from pse._modeling import (
    CompactStudyResult,
    LocalStudy,
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
    LiveSimulationCursor,
    LiveSimulationEvent,
    PreparedOperation,
    ProgressStream,
    RunHandle,
    RunResult,
    ScientificReplay,
    StoredResult,
    StudyCancel,
    StudyHandle,
    StudyPointStatus,
    StudyResults,
)
from pse._strategies import PreparedFlow, PreparedStrategy, StrategyResult
from pse._study_workbench import FrozenStudyCohort, StudyKpiAnalysis, StudyObserver
from pse._transfer import FieldTransfer
from pse._workflow import (
    PhysicalContext,
    ProjectBundle,
    Runtime,
    SolverCapability,
)
from pse.contracts import extension_types
from pse.contracts.documents import (
    SCHEMA_SET_FINGERPRINT,
    AdjointSettings,
    AnalysisControls,
    BackendSettings,
    BlockSolveOverride,
    BuildInfo,
    CacheReport,
    ClarabelSettings,
    Completion,
    CompositionRequest,
    ConformanceControls,
    ConicRequest,
    DiagnosticAnnotationDocument,
    DiagnosticCauseDocument,
    DiagnosticContextDocument,
    DiagnosticNoteDocument,
    DiagnosticSamplesControls,
    DiagnosticSpanDocument,
    DiffsolSettings,
    EffortPolicy,
    EligibilityDocument,
    FitExecutionDocument,
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
    IpoptSettings,
    JacobianDiagnosticControls,
    KinsolSettings,
    KnowledgeControls,
    LazyStudyDefinition,
    LinearDiagnosticControls,
    ModelingDiagnosticPolicy,
    ModelingInspection,
    ModelingInspectionRequest,
    ModelingNonlinearPolicy,
    NumericalStrategy,
    NumericalStrategyDocument,
    PhysicalFitRequest,
    PounceSettings,
    ProgressEventDocument,
    ProgressMetricDocument,
    PureConformanceControls,
    RecycleRequest,
    RecycleRestartRequest,
    ResourceConsumer,
    ResourceReport,
    ResultReclamationPage,
    RouteDocument,
    RunControls,
    ScipSettings,
    SelectedStudyVariant,
    SolveControls,
    SolveSettings,
    SourceChoice,
    SourceLocation,
    StudyInputs,
    StudyInputsExplicit,
    StudyInputsLazy,
    StudyRunControls,
    StudyStatus,
    StudyVariants,
    StudyWaitControls,
    TableName,
    TearSelectionDocument,
    WarmStartSnapshot,
)
from pse.diagnostics import CaptureSession, Diagnostics, OperationObservations

__all__ = [
    "AdjointSettings",
    "Analysis",
    "AnalysisControls",
    "ArtifactOwnerKind",
    "ArtifactSettlement",
    "BackendSettings",
    "BlockSolveOverride",
    "BuildInfo",
    "CacheReport",
    "CacheSettings",
    "CaptureSession",
    "ClarabelSettings",
    "CompactStudyResult",
    "Completion",
    "CompositionRequest",
    "ConformanceControls",
    "ConicRequest",
    "DiagnosticAnnotationDocument",
    "DiagnosticCauseDocument",
    "DiagnosticContextDocument",
    "DiagnosticNoteDocument",
    "DiagnosticReport",
    "DiagnosticSamplesControls",
    "DiagnosticSpanDocument",
    "Diagnostics",
    "DiffsolSettings",
    "EffortPolicy",
    "EligibilityDocument",
    "EngineSettings",
    "FieldTransfer",
    "FitExecutionDocument",
    "FitProfileDocument",
    "FlowConnectionDocument",
    "FlowGraphDocument",
    "FlowSelectionDocument",
    "FrozenStudyCohort",
    "HighsSettings",
    "IdasSettings",
    "IncumbentDocument",
    "IneligibleDocument",
    "InitializationDocument",
    "InitializationOverrides",
    "InspectionError",
    "IpoptSettings",
    "JacobianDiagnosticControls",
    "KinsolSettings",
    "KnowledgeControls",
    "LazyStudyDefinition",
    "LinearDiagnosticControls",
    "LiveSimulationCursor",
    "LiveSimulationEvent",
    "LocalStudy",
    "ModelingConformance",
    "ModelingDiagnosticPolicy",
    "ModelingDiagnosticSamples",
    "ModelingDiagnosticSettings",
    "ModelingDiagnostics",
    "ModelingElasticAttempt",
    "ModelingInitialization",
    "ModelingInitializationAttempt",
    "ModelingInspection",
    "ModelingInspectionRequest",
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
    "NumericalStrategyDocument",
    "OperationObservations",
    "PhysicalContext",
    "PhysicalFitRequest",
    "PounceSettings",
    "PreparedFlow",
    "PreparedOperation",
    "PreparedStrategy",
    "ProgressEventDocument",
    "ProgressMetricDocument",
    "ProgressStream",
    "ProjectBundle",
    "PureConformanceControls",
    "RecycleRequest",
    "RecycleRestartRequest",
    "ResourceConsumer",
    "ResourceReport",
    "ResultReclamationPage",
    "RouteDocument",
    "RunControls",
    "RunHandle",
    "RunResult",
    "Runtime",
    "ScientificArtifact",
    "ScientificArtifactKind",
    "ScientificReplay",
    "ScipSettings",
    "SelectedStudyVariant",
    "SimulationSettings",
    "SolveControls",
    "SolveSettings",
    "SolverCapability",
    "SourceChoice",
    "SourceLocation",
    "StoredResult",
    "StrategyResult",
    "StudyCancel",
    "StudyHandle",
    "StudyInputs",
    "StudyInputsExplicit",
    "StudyInputsLazy",
    "StudyKpiAnalysis",
    "StudyObserver",
    "StudyPointStatus",
    "StudyReport",
    "StudyResults",
    "StudyRunControls",
    "StudyStatus",
    "StudyVariants",
    "StudyWaitControls",
    "TableName",
    "TableStream",
    "TearSelectionDocument",
    "WarmStartSnapshot",
    "__version__",
    "build_info",
    "registry_table",
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
_check_native_compatibility(__version__, SCHEMA_SET_FINGERPRINT)
extension_types.register_all()
