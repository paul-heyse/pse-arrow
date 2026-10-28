# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Process systems engineering with independent durable-control imports.

Scientific exports load their qualified native image when first consumed. Importing
``pse.control`` or its generated documents neither loads that image nor prepares science.
"""

import importlib
import importlib.metadata
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pse._scientific import *  # noqa: F403 -- the explicit scientific export owner supplies static types

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
    "GeneratedDesign",
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
    "PetscSettings",
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
    "host_operation_catalogue",
    "registry_table",
]

try:
    __version__: str | None = importlib.metadata.version("pse-arrow")
except importlib.metadata.PackageNotFoundError:
    __version__ = None


def __getattr__(name: str) -> object:
    """Resolve declared scientific exports without intercepting submodule imports."""
    if (name not in __all__ and name not in ('SCHEMA_SET_FINGERPRINT', 'extension_types', 'native_version')) or name == "__version__":
        raise AttributeError(f"module 'pse' has no attribute {name!r}")
    value = getattr(importlib.import_module("pse._scientific"), name)
    globals()[name] = value
    return value


def __dir__() -> list[str]:
    return sorted(set(globals()) | set(__all__))
