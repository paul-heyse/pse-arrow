// @generated from emitted MessageName and Validate implementations; do not edit.
macro_rules! dispatch {
    ($name:expr, $callback:ident, $arguments:tt) => {
        match $name { "pse.code_intelligence.v1.AnswerDemand" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::AnswerDemand
        > $arguments), "pse.code_intelligence.v1.ApiDefinition" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::ApiDefinition
        > $arguments), "pse.code_intelligence.v1.ApiFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::ApiFact >
        $arguments), "pse.code_intelligence.v1.ApiQuery" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::ApiQuery >
        $arguments), "pse.code_intelligence.v1.BooleanGroup" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::BooleanGroup
        > $arguments), "pse.code_intelligence.v1.CompilerBodyFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerBodyFact
        > $arguments), "pse.code_intelligence.v1.CompilerCallee" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerCallee
        > $arguments), "pse.code_intelligence.v1.CompilerCaptureFact" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerCaptureFact
        > $arguments), "pse.code_intelligence.v1.CompilerCleanupFact" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerCleanupFact
        > $arguments), "pse.code_intelligence.v1.CompilerFacts" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerFacts
        > $arguments), "pse.code_intelligence.v1.CompilerInitialization" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerInitialization
        > $arguments), "pse.code_intelligence.v1.CompilerSavedField" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerSavedField
        > $arguments), "pse.code_intelligence.v1.CompilerSourceFile" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerSourceFile
        > $arguments), "pse.code_intelligence.v1.CompilerSourceLocation" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerSourceLocation
        > $arguments), "pse.code_intelligence.v1.CompilerSuspensionVariant" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::CompilerSuspensionVariant
        > $arguments), "pse.code_intelligence.v1.ContextGuard" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::ContextGuard
        > $arguments), "pse.code_intelligence.v1.Control" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Control >
        $arguments), "pse.code_intelligence.v1.Count" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Count >
        $arguments), "pse.code_intelligence.v1.Event" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Event >
        $arguments), "pse.code_intelligence.v1.Failure" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Failure >
        $arguments), "pse.code_intelligence.v1.GuardedReference" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::GuardedReference
        > $arguments), "pse.code_intelligence.v1.HistoryQuery" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::HistoryQuery
        > $arguments), "pse.code_intelligence.v1.Location" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Location >
        $arguments), "pse.code_intelligence.v1.NativeAdjustmentFact" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAdjustmentFact
        > $arguments), "pse.code_intelligence.v1.NativeAnalysisBackend" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend
        > $arguments), "pse.code_intelligence.v1.NativeAnalysisBackend.Alternative1" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::native_analysis_backend::Alternative1
        > $arguments), "pse.code_intelligence.v1.NativeAnalysisBackend.Alternative2" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::native_analysis_backend::Alternative2
        > $arguments), "pse.code_intelligence.v1.NativeAnalysisContext" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisContext
        > $arguments), "pse.code_intelligence.v1.NativeAnswer" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnswer
        > $arguments), "pse.code_intelligence.v1.NativeBodyObservation" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeBodyObservation
        > $arguments), "pse.code_intelligence.v1.NativeBoundFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeBoundFact
        > $arguments), "pse.code_intelligence.v1.NativeByteLocation" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation
        > $arguments), "pse.code_intelligence.v1.NativeCallableCandidateFact" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCallableCandidateFact
        > $arguments), "pse.code_intelligence.v1.NativeCallableContextFact" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCallableContextFact
        > $arguments), "pse.code_intelligence.v1.NativeCallableObligationFact" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCallableObligationFact
        > $arguments), "pse.code_intelligence.v1.NativeCallableSubstitutionFact" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCallableSubstitutionFact
        > $arguments), "pse.code_intelligence.v1.NativeCallableSubstitutions" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCallableSubstitutions
        > $arguments), "pse.code_intelligence.v1.NativeCancellationObservation" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCancellationObservation
        > $arguments), "pse.code_intelligence.v1.NativeCandidateHandle" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCandidateHandle
        > $arguments), "pse.code_intelligence.v1.NativeCaptureFact" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCaptureFact
        > $arguments), "pse.code_intelligence.v1.NativeCaptureUsages" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCaptureUsages
        > $arguments), "pse.code_intelligence.v1.NativeCaptureUseFact" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCaptureUseFact
        > $arguments), "pse.code_intelligence.v1.NativeCompatibilityObservation" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCompatibilityObservation
        > $arguments), "pse.code_intelligence.v1.NativeCompatibilityQuery" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeCompatibilityQuery
        > $arguments), "pse.code_intelligence.v1.NativeConfiguration" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConfiguration
        > $arguments), "pse.code_intelligence.v1.NativeConflict" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConflict
        > $arguments), "pse.code_intelligence.v1.NativeControl" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeControl
        > $arguments), "pse.code_intelligence.v1.NativeDependencyReference" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyReference
        > $arguments), "pse.code_intelligence.v1.NativeDependencyUsageAnswer" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageAnswer
        > $arguments), "pse.code_intelligence.v1.NativeDependencyUsageQuery" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageQuery
        > $arguments), "pse.code_intelligence.v1.NativeDocumentationLinkObservation" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDocumentationLinkObservation
        > $arguments), "pse.code_intelligence.v1.NativeEditCandidate" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeEditCandidate
        > $arguments), "pse.code_intelligence.v1.NativeEvent" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeEvent
        > $arguments), "pse.code_intelligence.v1.NativeFailureContext" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeFailureContext
        > $arguments), "pse.code_intelligence.v1.NativeFileOperation" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeFileOperation
        > $arguments), "pse.code_intelligence.v1.NativeGraphAnswer" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphAnswer
        > $arguments), "pse.code_intelligence.v1.NativeGraphDistance" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphDistance
        > $arguments), "pse.code_intelligence.v1.NativeGraphEdge" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphEdge
        > $arguments), "pse.code_intelligence.v1.NativeGraphNode" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphNode
        > $arguments), "pse.code_intelligence.v1.NativeGraphQuery" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphQuery
        > $arguments), "pse.code_intelligence.v1.NativeGraphReached" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphReached
        > $arguments), "pse.code_intelligence.v1.NativeGraphRole" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphRole
        > $arguments), "pse.code_intelligence.v1.NativeGraphTraversal" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphTraversal
        > $arguments), "pse.code_intelligence.v1.NativeGraphWitness" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeGraphWitness
        > $arguments), "pse.code_intelligence.v1.NativeImportTarget" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeImportTarget
        > $arguments), "pse.code_intelligence.v1.NativeImportabilityFact" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeImportabilityFact
        > $arguments), "pse.code_intelligence.v1.NativeInvestigationFacts" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeInvestigationFacts
        > $arguments), "pse.code_intelligence.v1.NativeLayoutFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLayoutFact
        > $arguments), "pse.code_intelligence.v1.NativeLayoutFieldFact" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLayoutFieldFact
        > $arguments), "pse.code_intelligence.v1.NativeLazyCandidate" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLazyCandidate
        > $arguments), "pse.code_intelligence.v1.NativeLibraryProfile" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLibraryProfile
        > $arguments), "pse.code_intelligence.v1.NativeLocation" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLocation
        > $arguments), "pse.code_intelligence.v1.NativeObservedFacts" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeObservedFacts
        > $arguments), "pse.code_intelligence.v1.NativeOccurrenceFile" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeOccurrenceFile
        > $arguments), "pse.code_intelligence.v1.NativeOccurrenceScope" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeOccurrenceScope
        > $arguments), "pse.code_intelligence.v1.NativeOmittedRelationship" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeOmittedRelationship
        > $arguments), "pse.code_intelligence.v1.NativeOriginObservation" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeOriginObservation
        > $arguments), "pse.code_intelligence.v1.NativePackageContext" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageContext
        > $arguments), "pse.code_intelligence.v1.NativePackageContextAnswer" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageContextAnswer
        > $arguments), "pse.code_intelligence.v1.NativePackageReply" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageReply
        > $arguments), "pse.code_intelligence.v1.NativePackageRequest" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageRequest
        > $arguments), "pse.code_intelligence.v1.NativePathBuf" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf
        > $arguments), "pse.code_intelligence.v1.NativePlaceholder" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePlaceholder
        > $arguments), "pse.code_intelligence.v1.NativePosition" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePosition
        > $arguments), "pse.code_intelligence.v1.NativePredicateFact" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePredicateFact
        > $arguments), "pse.code_intelligence.v1.NativePreview" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePreview
        > $arguments), "pse.code_intelligence.v1.NativeProducerContext" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeProducerContext
        > $arguments), "pse.code_intelligence.v1.NativeProtocolLocation" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeProtocolLocation
        > $arguments), "pse.code_intelligence.v1.NativeQuery" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeQuery
        > $arguments), "pse.code_intelligence.v1.NativeRange" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeRange
        > $arguments), "pse.code_intelligence.v1.NativeReceiverAdjustments" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeReceiverAdjustments
        > $arguments), "pse.code_intelligence.v1.NativeRelationshipFact" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeRelationshipFact
        > $arguments), "pse.code_intelligence.v1.NativeRequest" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeRequest
        > $arguments), "pse.code_intelligence.v1.NativeRunnableTarget" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeRunnableTarget
        > $arguments), "pse.code_intelligence.v1.NativeSection" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSection
        > $arguments), "pse.code_intelligence.v1.NativeSessionStatus" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus
        > $arguments), "pse.code_intelligence.v1.NativeSnapshotUpdate" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSnapshotUpdate
        > $arguments), "pse.code_intelligence.v1.NativeSourceHash" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSourceHash
        > $arguments), "pse.code_intelligence.v1.NativeSourceIdentity" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSourceIdentity
        > $arguments), "pse.code_intelligence.v1.NativeSourceText" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSourceText
        > $arguments), "pse.code_intelligence.v1.NativeSymbolFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSymbolFact
        > $arguments), "pse.code_intelligence.v1.NativeTargetFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeTargetFact
        > $arguments), "pse.code_intelligence.v1.NativeTestCandidate" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeTestCandidate
        > $arguments), "pse.code_intelligence.v1.NativeTypeContext" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeTypeContext
        > $arguments), "pse.code_intelligence.v1.NativeTypeFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeTypeFact
        > $arguments), "pse.code_intelligence.v1.NativeTypedFacts" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeTypedFacts
        > $arguments), "pse.code_intelligence.v1.NativeVisibilityQuery" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeVisibilityQuery
        > $arguments), "pse.code_intelligence.v1.NativeWorkspaceAnswer" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeWorkspaceAnswer
        > $arguments), "pse.code_intelligence.v1.NativeWorkspaceQuery" => Some($callback
        :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeWorkspaceQuery
        > $arguments), "pse.code_intelligence.v1.Negation" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Negation >
        $arguments), "pse.code_intelligence.v1.Observation" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Observation
        > $arguments), "pse.code_intelligence.v1.OriginalEvidence" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::OriginalEvidence
        > $arguments), "pse.code_intelligence.v1.PlatformPathBytes" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes
        > $arguments), "pse.code_intelligence.v1.PlatformPathBytes.Alternative1" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::platform_path_bytes::Alternative1
        > $arguments), "pse.code_intelligence.v1.PlatformPathBytes.Alternative2" =>
        Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::platform_path_bytes::Alternative2
        > $arguments), "pse.code_intelligence.v1.Predicate" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Predicate >
        $arguments), "pse.code_intelligence.v1.PredicateFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PredicateFact
        > $arguments), "pse.code_intelligence.v1.Refine" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Refine >
        $arguments), "pse.code_intelligence.v1.Refinement" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Refinement >
        $arguments), "pse.code_intelligence.v1.Related" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Related >
        $arguments), "pse.code_intelligence.v1.Request" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Request >
        $arguments), "pse.code_intelligence.v1.SourceFact" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::SourceFact >
        $arguments), "pse.code_intelligence.v1.SourceSelector" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::SourceSelector
        > $arguments), "pse.code_intelligence.v1.StructuredQuery" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::StructuredQuery
        > $arguments), "pse.code_intelligence.v1.SymbolQuery" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::SymbolQuery
        > $arguments), "pse.code_intelligence.v1.SyntaxQuery" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::SyntaxQuery
        > $arguments), "pse.code_intelligence.v1.Terminal" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Terminal >
        $arguments), "pse.code_intelligence.v1.TextQuery" => Some($callback :: <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::TextQuery >
        $arguments), "pse.code_intelligence.v1.ValidationDisclosure" => Some($callback ::
        <
        $crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::ValidationDisclosure
        > $arguments), _ => None }
    };
}
pub(crate) use dispatch;
