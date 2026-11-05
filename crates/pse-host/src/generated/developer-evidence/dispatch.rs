// @generated from emitted MessageName and Validate implementations; do not edit.
macro_rules! dispatch {
    ($name:expr, $callback:ident, $arguments:tt) => {
        match $name { "pse.developer.evidence.v1.Array16U8" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8
        > $arguments), "pse.developer.evidence.v1.Array32U8" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array32U8
        > $arguments), "pse.developer.evidence.v1.Array3Array16U8" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array3Array16U8
        > $arguments), "pse.developer.evidence.v1.CapturedWorkflowDirectoryManifest" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowDirectoryManifest
        > $arguments), "pse.developer.evidence.v1.CapturedWorkflowEvidence" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowEvidence
        > $arguments), "pse.developer.evidence.v1.CapturedWorkflowReport" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowReport
        > $arguments), "pse.developer.evidence.v1.CliInvocationId" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId
        > $arguments), "pse.developer.evidence.v1.ContentIdentity" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ContentIdentity
        > $arguments), "pse.developer.evidence.v1.DiagnosticFacts" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticFacts
        > $arguments), "pse.developer.evidence.v1.DiagnosticLabel" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLabel
        > $arguments), "pse.developer.evidence.v1.DiagnosticLocation" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLocation
        > $arguments), "pse.developer.evidence.v1.DiagnosticObservation" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation
        > $arguments), "pse.developer.evidence.v1.DiagnosticObservation.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative1
        > $arguments), "pse.developer.evidence.v1.DiagnosticObservation.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative2
        > $arguments), "pse.developer.evidence.v1.DiagnosticObservation.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative3
        > $arguments), "pse.developer.evidence.v1.DiagnosticObservation.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative4
        > $arguments), "pse.developer.evidence.v1.DiagnosticObservation.Alternative5" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative5
        > $arguments), "pse.developer.evidence.v1.DiagnosticObservation.Alternative6" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative6
        > $arguments), "pse.developer.evidence.v1.DiagnosticObservation.Alternative7" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative7
        > $arguments), "pse.developer.evidence.v1.DiagnosticOperandContract" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticOperandContract
        > $arguments), "pse.developer.evidence.v1.DiagnosticRecovery" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecovery
        > $arguments), "pse.developer.evidence.v1.DiagnosticRecoveryCause" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause
        > $arguments), "pse.developer.evidence.v1.DiagnosticRecoveryCause.Alternative1"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative1
        > $arguments), "pse.developer.evidence.v1.DiagnosticRecoveryCause.Alternative2"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative2
        > $arguments), "pse.developer.evidence.v1.DiagnosticRecoveryCause.Alternative3"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative3
        > $arguments), "pse.developer.evidence.v1.DiagnosticRecoveryCause.Alternative4"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative4
        > $arguments), "pse.developer.evidence.v1.DiagnosticRecoveryCause.Alternative5"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative5
        > $arguments), "pse.developer.evidence.v1.DiagnosticRecoveryCause.Alternative6"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative6
        > $arguments), "pse.developer.evidence.v1.ErrorRecord" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord
        > $arguments), "pse.developer.evidence.v1.FailurePhase" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailurePhase
        > $arguments), "pse.developer.evidence.v1.FailurePhase.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_phase::Alternative1
        > $arguments), "pse.developer.evidence.v1.FailurePhase.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_phase::Alternative2
        > $arguments), "pse.developer.evidence.v1.FailurePhase.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_phase::Alternative3
        > $arguments), "pse.developer.evidence.v1.FailurePhase.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_phase::Alternative4
        > $arguments), "pse.developer.evidence.v1.FailurePhase.Alternative5" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_phase::Alternative5
        > $arguments), "pse.developer.evidence.v1.FailurePhase.Alternative6" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_phase::Alternative6
        > $arguments), "pse.developer.evidence.v1.FailurePhase.Alternative7" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_phase::Alternative7
        > $arguments), "pse.developer.evidence.v1.FailureReason" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative1
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative2
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative3
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative4
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative5" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative5
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative6" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative6
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative7" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative7
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative8" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative8
        > $arguments), "pse.developer.evidence.v1.FailureReason.Alternative9" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative9
        > $arguments), "pse.developer.evidence.v1.FailureState" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureState
        > $arguments), "pse.developer.evidence.v1.FailureState.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_state::Alternative1
        > $arguments), "pse.developer.evidence.v1.FailureState.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_state::Alternative2
        > $arguments), "pse.developer.evidence.v1.FailureState.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_state::Alternative3
        > $arguments), "pse.developer.evidence.v1.FailureState.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_state::Alternative4
        > $arguments), "pse.developer.evidence.v1.FailureState.Alternative5" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_state::Alternative5
        > $arguments), "pse.developer.evidence.v1.InputOrigin" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InputOrigin
        > $arguments), "pse.developer.evidence.v1.ManagerSettlement" => Some($callback ::
        <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement
        > $arguments), "pse.developer.evidence.v1.ManagerSettlement.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::manager_settlement::Alternative1
        > $arguments), "pse.developer.evidence.v1.ManagerSettlement.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::manager_settlement::Alternative2
        > $arguments), "pse.developer.evidence.v1.ManagerSettlement.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::manager_settlement::Alternative3
        > $arguments), "pse.developer.evidence.v1.ManagerSettlement.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::manager_settlement::Alternative4
        > $arguments), "pse.developer.evidence.v1.NativePath" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath
        > $arguments), "pse.developer.evidence.v1.NativePath.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::native_path::Alternative1
        > $arguments), "pse.developer.evidence.v1.NativePath.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::native_path::Alternative2
        > $arguments), "pse.developer.evidence.v1.NativeWorkflowReport" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativeWorkflowReport
        > $arguments), "pse.developer.evidence.v1.NextAction" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction
        > $arguments), "pse.developer.evidence.v1.NextAction.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::next_action::Alternative1
        > $arguments), "pse.developer.evidence.v1.NextAction.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::next_action::Alternative2
        > $arguments), "pse.developer.evidence.v1.NonFinite" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NonFinite
        > $arguments), "pse.developer.evidence.v1.SemanticRpcDisposition" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::SemanticRpcDisposition
        > $arguments), "pse.developer.evidence.v1.SemanticRpcDisposition.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::semantic_rpc_disposition::Alternative1
        > $arguments), "pse.developer.evidence.v1.SemanticRpcDisposition.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::semantic_rpc_disposition::Alternative2
        > $arguments), "pse.developer.evidence.v1.SemanticRpcDisposition.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::semantic_rpc_disposition::Alternative3
        > $arguments), "pse.developer.evidence.v1.SemanticRpcDisposition.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::semantic_rpc_disposition::Alternative4
        > $arguments), "pse.developer.evidence.v1.SemanticRpcDisposition.Alternative5" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::semantic_rpc_disposition::Alternative5
        > $arguments), "pse.developer.evidence.v1.SemanticRpcDisposition.Alternative6" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::semantic_rpc_disposition::Alternative6
        > $arguments), "pse.developer.evidence.v1.SemanticRpcDisposition.Alternative7" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::semantic_rpc_disposition::Alternative7
        > $arguments), "pse.developer.evidence.v1.SemanticRpcObservation" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::SemanticRpcObservation
        > $arguments), "pse.developer.evidence.v1.TestIdentity" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity
        > $arguments), "pse.developer.evidence.v1.ValidationCorrection" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ValidationCorrection
        > $arguments), "pse.developer.evidence.v1.ValidationCoverage" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ValidationCoverage
        > $arguments), "pse.developer.evidence.v1.ValidationCoverage.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_coverage::Alternative1
        > $arguments), "pse.developer.evidence.v1.ValidationCoverage.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_coverage::Alternative2
        > $arguments), "pse.developer.evidence.v1.ValidationCoverage.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_coverage::Alternative3
        > $arguments), "pse.developer.evidence.v1.ValidationCoverage.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_coverage::Alternative4
        > $arguments), "pse.developer.evidence.v1.ValidationDisclosure" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ValidationDisclosure
        > $arguments), "pse.developer.evidence.v1.ValidationExposure" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ValidationExposure
        > $arguments), "pse.developer.evidence.v1.ValidationExposure.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_exposure::Alternative1
        > $arguments), "pse.developer.evidence.v1.ValidationExposure.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_exposure::Alternative2
        > $arguments), "pse.developer.evidence.v1.ValidationExposure.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_exposure::Alternative3
        > $arguments), "pse.developer.evidence.v1.ValidationOutcome" => Some($callback ::
        <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ValidationOutcome
        > $arguments), "pse.developer.evidence.v1.ValidationOutcome.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_outcome::Alternative1
        > $arguments), "pse.developer.evidence.v1.ValidationOutcome.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_outcome::Alternative2
        > $arguments), "pse.developer.evidence.v1.ValidationOutcome.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_outcome::Alternative3
        > $arguments), "pse.developer.evidence.v1.ValidationOutcome.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_outcome::Alternative4
        > $arguments), "pse.developer.evidence.v1.ValidationPathElement" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ValidationPathElement
        > $arguments), "pse.developer.evidence.v1.ValidationSelector" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ValidationSelector
        > $arguments), "pse.developer.evidence.v1.ValidationSelector.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_selector::Alternative1
        > $arguments), "pse.developer.evidence.v1.ValidationSelector.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::validation_selector::Alternative2
        > $arguments), "pse.developer.evidence.v1.ValidationViolation" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ValidationViolation
        > $arguments), "pse.developer.evidence.v1.WorkflowAction" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction
        > $arguments), "pse.developer.evidence.v1.WorkflowAction.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_action::Alternative1
        > $arguments), "pse.developer.evidence.v1.WorkflowAction.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_action::Alternative2
        > $arguments), "pse.developer.evidence.v1.WorkflowAction.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_action::Alternative3
        > $arguments), "pse.developer.evidence.v1.WorkflowAction.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_action::Alternative4
        > $arguments), "pse.developer.evidence.v1.WorkflowCapturedFile" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowCapturedFile
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceDirectoryProfile" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowEvidenceDirectoryProfile
        > $arguments),
        "pse.developer.evidence.v1.WorkflowEvidenceDirectoryProfile.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_directory_profile::Alternative1
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceFile" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowEvidenceFile
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceOmission" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowEvidenceOmission
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceOmission.Alternative1"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_omission::Alternative1
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceOmission.Alternative2"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_omission::Alternative2
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceOmission.Alternative3"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_omission::Alternative3
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceOmission.Alternative4"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_omission::Alternative4
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowEvidenceRole
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative1
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative2
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative3
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative4
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative5" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative5
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative6" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative6
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative7" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative7
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative8" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative8
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRole.Alternative9" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_role::Alternative9
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRoot" => Some($callback
        :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowEvidenceRoot
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRoot.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_root::Alternative1
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceRoot.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_root::Alternative2
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceScope" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowEvidenceScope
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceScope.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_scope::Alternative1
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceScope.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_scope::Alternative2
        > $arguments), "pse.developer.evidence.v1.WorkflowEvidenceScope.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_evidence_scope::Alternative3
        > $arguments), "pse.developer.evidence.v1.WorkflowOutcome" => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome
        > $arguments), "pse.developer.evidence.v1.WorkflowOutcome.Alternative1" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative1
        > $arguments), "pse.developer.evidence.v1.WorkflowOutcome.Alternative2" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative2
        > $arguments), "pse.developer.evidence.v1.WorkflowOutcome.Alternative3" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative3
        > $arguments), "pse.developer.evidence.v1.WorkflowOutcome.Alternative4" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative4
        > $arguments), "pse.developer.evidence.v1.WorkflowOutcome.Alternative5" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative5
        > $arguments), "pse.developer.evidence.v1.WorkflowOutcome.Alternative6" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative6
        > $arguments), "pse.developer.evidence.v1.WorkflowOutcome.Alternative7" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative7
        > $arguments), "pse.developer.evidence.v1.WorkflowReportObservation" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation
        > $arguments), "pse.developer.evidence.v1.WorkflowReportObservation.Field10Value"
        => Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_report_observation::Field10Value
        > $arguments), "pse.developer.evidence.v1.WorkflowReportReference" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportReference
        > $arguments), "pse.developer.evidence.v1.WorkflowTerminalRecord" =>
        Some($callback :: <
        $crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord
        > $arguments), _ => None }
    };
}
pub(crate) use dispatch;
