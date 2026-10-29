// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Attach only source evidence supplied by the compiler or native boundary.
use super::{RunReport, RunResult};
use pse_model::diagnostic::{BoundaryDiagnostic, Observation};

use pse_diagnostics::TypedDiagnostic;
use pse_model::diagnostic::{DiagnosticProjection, DiagnosticRule, DiagnosticStage, project_facts};

fn canonical_diagnostic(
    error: &pse_operations::canonical::CanonicalError,
    stage: DiagnosticStage,
) -> BoundaryDiagnostic {
    use pse_operations::canonical::CanonicalError as E;
    let mut diagnostic = pse_model::diagnostic::project_typed(error, stage);
    match error {
        E::SettlementUnresolved {
            owner,
            operation,
            request,
            source,
            readback,
        } => {
            diagnostic
                .observations
                .insert("canonical_owner".into(), Observation::Text((*owner).into()));
            diagnostic.observations.insert(
                "canonical_operation".into(),
                Observation::Text(operation.clone()),
            );
            diagnostic.observations.insert(
                "publication_effect".into(),
                Observation::Text("unresolved".into()),
            );
            diagnostic.observations.insert(
                "immutable_request_bytes".into(),
                Observation::Integer(request.len() as i64),
            );
            diagnostic.causes.push(canonical_diagnostic(source, stage));
            if let Some(readback) = readback {
                diagnostic
                    .causes
                    .push(canonical_diagnostic(readback, stage));
            }
        }
        E::SourceEditUnsettled { source, .. }
        | E::SourceEditRefused { source, .. }
        | E::AnalysisUnsettled { source, .. }
        | E::ArtifactUnsettled { source, .. } => {
            diagnostic.causes.push(canonical_diagnostic(source, stage));
        }
        E::PhysicalClose { source } | E::ProtectionRelease { source } => {
            diagnostic.causes.push(canonical_diagnostic(source, stage));
        }
        _ => {}
    }
    diagnostic
}

impl super::WorkflowError {
    pub(crate) fn strategy_trace(&self) -> Option<&std::sync::Arc<crate::math::strategy::Trace>> {
        match self {
            Self::Math(cause) => cause.strategy_trace(),
            Self::ConditionalAdmission { cause, .. } | Self::ModelingAdmission { cause, .. } => {
                cause.strategy_trace()
            }
            Self::Shared(cause) => cause.strategy_trace(),
            Self::Admission { cause, .. } => cause.strategy_trace(),
            _ => None,
        }
    }
    /// The source owner supplies typed facts and scientific evidence.
    pub fn boundary_diagnostic(&self) -> BoundaryDiagnostic {
        DiagnosticProjection::boundary_diagnostic(self, DiagnosticStage::Workflow)
    }
}
impl DiagnosticProjection for super::WorkflowError {
    fn boundary_diagnostic(&self, stage: DiagnosticStage) -> BoundaryDiagnostic {
        match self {
            Self::Settlement { operation, cleanup } => {
                let mut diagnostic = DiagnosticProjection::boundary_diagnostic(operation.as_ref(), stage);
                diagnostic.causes.push(DiagnosticProjection::boundary_diagnostic(cleanup.as_ref(), stage));
                diagnostic
            }
            Self::Boundary(d) | Self::ScientificRefusal(d) => d.as_ref().clone(),
            Self::Admission { effect, cause } => {
                let mut diagnostic =
                    DiagnosticProjection::boundary_diagnostic(cause.as_ref(), stage);
                diagnostic.observations.insert(
                    "admission_effect".into(),
                    Observation::Text(effect.as_str().into()),
                );
                if let super::AdmissionEffect::SourceAcknowledged { revision } = effect {
                    diagnostic.observations.insert(
                        "source_revision".into(),
                        Observation::Text(revision.clone()),
                    );
                }
                diagnostic
            }
            Self::StudyExecutionUnavailable { study } => {
                let mut diagnostic =
                    project_facts(self.diagnostic_code(), self.diagnostic_facts(), stage);
                diagnostic.rule = DiagnosticRule::WorkflowInternal;
                diagnostic
                    .observations
                    .insert("study_key".into(), Observation::Text(study.to_string()));
                diagnostic
                    .observations
                    .insert("detail".into(), Observation::Text(self.to_string()));
                diagnostic
            }
            Self::ConditionalAdmission { diagnostic, cause }
            | Self::ModelingAdmission { diagnostic, cause } => {
                let mut d = diagnostic.as_ref().clone();
                if d.causes.is_empty() {
                    d.causes.push(cause.as_ref().boundary_diagnostic(stage));
                }
                d
            }
            Self::StudyCreationUnsettled {
                study,
                source,
                cleanup,
            } => {
                let mut diagnostic =
                    DiagnosticProjection::boundary_diagnostic(source.as_ref(), stage);
                diagnostic
                    .observations
                    .insert("study_key".into(), Observation::Text(study.to_string()));
                diagnostic.causes.push(pse_model::diagnostic::project_typed(
                    cleanup.as_ref(),
                    stage,
                ));
                diagnostic
            }
            Self::ArtifactTransferUnsettled {
                analysis,
                artifact,
                owner_kind,
                owner,
                read,
                source,
            } => {
                let mut diagnostic =
                    DiagnosticProjection::boundary_diagnostic(source.as_ref(), stage);
                for (key, value) in [
                    ("artifact_key", Some(artifact.key.clone())),
                    ("analysis_key", analysis.clone()),
                    ("artifact_owner_kind", Some(owner_kind.clone())),
                    ("artifact_owner", Some(owner.clone())),
                    ("artifact_read_key", read.clone()),
                ] {
                    if let Some(value) = value {
                        diagnostic
                            .observations
                            .insert(key.into(), Observation::Text(value));
                    }
                }
                diagnostic
            }
            Self::Math(e) => e.boundary_diagnostic(stage),
            Self::Typed(e) => e.boundary_diagnostic(stage),
            Self::Shared(e) => DiagnosticProjection::boundary_diagnostic(e.as_ref(), stage),
            Self::Authoring(e) => e.boundary_diagnostic(stage),
            Self::Engine(e) => pse_model::diagnostic::project_typed(e, stage),
            Self::SeedRead(e) => pse_model::diagnostic::project_typed(e, stage),
            Self::Canonical(e) => canonical_diagnostic(e, stage),
            Self::ResultBlock(e) => pse_model::diagnostic::project_typed(e, stage),
            Self::DocumentDecode(_) => pse_model::diagnostic::project_typed(self, stage),
            Self::Input(_)
            | Self::Resource(_)
            | Self::Integrity(_)
            | Self::Infrastructure(_)
            | Self::Unsupported(_)
            | Self::RecordingUnavailable
            | Self::CallerClosed
            | Self::Internal(_) => {
                let mut diagnostic =
                    project_facts(self.diagnostic_code(), self.diagnostic_facts(), stage);
                diagnostic.rule = match self {
                    Self::Input(_) | Self::RecordingUnavailable | Self::CallerClosed => {
                        DiagnosticRule::WorkflowInput
                    }
                    Self::Resource(_) => DiagnosticRule::MathLimit,
                    Self::Infrastructure(_) | Self::Integrity(_) => DiagnosticRule::MathLibrary,
                    Self::Unsupported(_) => DiagnosticRule::NativeUnsupported,
                    _ => DiagnosticRule::WorkflowInternal,
                };
                if matches!(self, Self::Integrity(_)) {
                    diagnostic
                        .observations
                        .insert("integrity_failure".into(), Observation::Boolean(true));
                }
                diagnostic
                    .observations
                    .insert("detail".into(), Observation::Text(self.to_string()));
                diagnostic
            }
        }
    }
}
/// Whether a selected immutable admission may retain a permanent scientific denial.
/// This is persistence permission, never authorization to repeat a canonical effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AdmissionDisposition {
    ScientificRefusal,
    TransientOrUnclassified,
    UnsettledEffects,
}
fn deterministic_diagnostic(diagnostic: &BoundaryDiagnostic) -> bool {
    use pse_model::diagnostic::BoundaryClass as C;
    matches!(
        diagnostic.class,
        C::InvalidModel | C::Unsupported | C::Incompatible
    ) && diagnostic.causes.iter().all(deterministic_diagnostic)
}
fn deterministic_math(error: &crate::math::MathRuntimeError) -> bool {
    use crate::math::MathRuntimeError as E;
    use pse_compiler::workspace::CompileError;
    fn modeling(error: &pse_modeling::ModelingError) -> bool {
        use pse_modeling::{ModelingContractReason, ModelingError};
        match error {
            ModelingError::Located { cause, .. } => modeling(cause),
            ModelingError::Contract { reason, .. } => {
                *reason != ModelingContractReason::Unclassified
            }
            _ => deterministic_diagnostic(&error.boundary_diagnostic()),
        }
    }
    match error {
        E::Shared(cause) | E::Strategy { cause, .. } => deterministic_math(cause),
        E::Compile(CompileError::Syntax { .. } | CompileError::ConditionalUnavailable(_)) => {
            deterministic_diagnostic(&error.boundary_diagnostic(DiagnosticStage::ModelingAdmission))
        }
        E::Compile(CompileError::Modeling(cause)) => modeling(cause),
        E::Solve(pse_backend_native::ProblemError::Structural { .. }) => true,
        _ => false,
    }
}
fn deterministic_scientific(error: &super::WorkflowError) -> bool {
    use super::WorkflowError as E;
    match error {
        E::Admission { cause, .. } => deterministic_scientific(cause),
        E::Shared(cause) => deterministic_scientific(cause),
        E::ScientificRefusal(diagnostic) => deterministic_diagnostic(diagnostic),
        E::Math(cause) => deterministic_math(cause),
        E::ConditionalAdmission { cause, .. } | E::ModelingAdmission { cause, .. } => {
            deterministic_math(cause)
        }
        E::Authoring(
            pse_admission::authoring_driver::DriverError::Authoring(_)
            | pse_admission::authoring_driver::DriverError::Rules(_),
        ) => {
            let diagnostic = error.boundary_diagnostic();
            diagnostic.code != pse_diagnostics::DiagnosticCode::AuthoringDocumentIo
                && deterministic_diagnostic(&diagnostic)
        }
        // Legacy string contracts, generic unsupported errors, database/schema reads and
        // opaque typed wrappers do not establish an immutable scientific refusal.
        _ => false,
    }
}
/// Interpret only an actual source-owner refusal proof; statement class and a
/// missing acknowledgement do not establish publication absence.
pub(super) fn publication_failure_effect(error: &super::WorkflowError) -> super::AdmissionEffect {
    match error {
        super::WorkflowError::Canonical(error) if error.proves_source_refusal() => {
            super::AdmissionEffect::NoPublication
        }
        super::WorkflowError::Shared(cause) => publication_failure_effect(cause),
        _ => super::AdmissionEffect::UnresolvedPublication,
    }
}
fn unresolved_admission_effect(error: &super::WorkflowError) -> bool {
    match error {
        super::WorkflowError::Admission { effect, cause } => {
            matches!(effect, super::AdmissionEffect::UnresolvedPublication)
                || unresolved_admission_effect(cause)
        }
        super::WorkflowError::Shared(cause) => unresolved_admission_effect(cause),
        _ => false,
    }
}
pub(super) fn admission_disposition(error: &super::WorkflowError) -> AdmissionDisposition {
    let effect = match error {
        super::WorkflowError::Admission { effect, .. } => effect,
        super::WorkflowError::Shared(cause) => return admission_disposition(cause),
        _ => return AdmissionDisposition::UnsettledEffects,
    };
    if matches!(effect, super::AdmissionEffect::UnresolvedPublication)
        || unresolved_admission_effect(error)
    {
        AdmissionDisposition::UnsettledEffects
    } else if deterministic_scientific(error)
        && deterministic_diagnostic(&error.boundary_diagnostic())
    {
        AdmissionDisposition::ScientificRefusal
    } else {
        AdmissionDisposition::TransientOrUnclassified
    }
}

pub(super) fn observed(
    error: &dyn DiagnosticProjection,
    stage: DiagnosticStage,
) -> BoundaryDiagnostic {
    error.boundary_diagnostic(stage)
}
/// Attribute an escaping workflow operation without replacing its typed error or retry class.
pub(super) fn operation_context(
    error: super::WorkflowError,
    operation: &'static str,
) -> super::WorkflowError {
    let diagnostic = error.boundary_diagnostic();
    use pse_model::diagnostic::BoundaryClass as C;
    match diagnostic.class {
        C::Internal | C::Infrastructure => tracing::error!(
            operation,
            code = diagnostic.code.as_str(),
            class = diagnostic.class.as_str(),
            "workflow operation failed"
        ),
        C::Cancelled => tracing::debug!(
            operation,
            code = diagnostic.code.as_str(),
            class = diagnostic.class.as_str(),
            "workflow operation cancelled"
        ),
        C::ResourceLimit => tracing::warn!(
            operation,
            code = diagnostic.code.as_str(),
            class = diagnostic.class.as_str(),
            "workflow operation allowance refused"
        ),
        _ => tracing::info!(
            operation,
            code = diagnostic.code.as_str(),
            class = diagnostic.class.as_str(),
            "workflow operation refused"
        ),
    }
    #[cfg(feature = "canonical-tests")]
    {
        #[allow(
            clippy::print_stderr,
            reason = "canonical qualification captures escaping operation attribution without a tracing subscriber"
        )]
        {
            eprintln!("canonical workflow operation {operation} failed: {error}");
        }
    }
    error
}
impl RunResult {
    pub(super) fn capture_diagnostics(&self) -> Vec<BoundaryDiagnostic> {
        let mut diagnostics: Vec<BoundaryDiagnostic> = match self.report() {
            Ok(RunReport::Horizon(_)) => self
                .completion
                .as_ref()
                .map_or_else(|_| Vec::new(), |completion| completion.diagnostics.clone()),
            Err(error) => self
                .modeling_failure
                .iter()
                .flat_map(|failure| {
                    failure
                        .completed
                        .iter()
                        .filter_map(|result| result.diagnostic())
                })
                .chain(std::iter::once(error.boundary_diagnostic()))
                .collect(),
            Ok(RunReport::Modeling(r)) => r.iter().filter_map(|r| r.diagnostic()).collect(),
            Ok(RunReport::Simulation(r)) => r.diagnostic().into_iter().collect(),
            #[cfg(feature = "solver-diffsol")]
            Ok(RunReport::Shooting(r)) => r
                .solve
                .iter()
                .filter_map(|s| s.validation_failure())
                .map(|cause| {
                    let mut diagnostic = observed(cause, DiagnosticStage::Shooting);
                    diagnostic.rule = DiagnosticRule::ShootingCandidateValidation;
                    diagnostic
                })
                .chain(r.validation_error.clone())
                .collect(),
            // Stable rule codes with the class derived from each typed cause.
            Ok(RunReport::Fit(r)) => r
                .diagnostic
                .iter()
                .map(|d| {
                    let mut diagnostic = observed(d.cause.as_ref(), DiagnosticStage::Fit);
                    diagnostic.rule = match d.rule {
                        super::FitRule::FinalEvaluation => DiagnosticRule::FitFinalEvaluation,
                        super::FitRule::ObjectiveOverflow => DiagnosticRule::FitObjectiveOverflow,
                        super::FitRule::ResponseRank => DiagnosticRule::FitResponseRank,
                    };
                    diagnostic
                })
                .chain(
                    r.solve
                        .iter()
                        .filter_map(|s| s.validation_failure())
                        .map(|cause| {
                            let mut diagnostic = observed(cause, DiagnosticStage::Fit);
                            diagnostic.rule = DiagnosticRule::FitCandidateValidation;
                            diagnostic
                        }),
                )
                .chain(r.validation_error.clone())
                .collect(),
        };
        let mut seen = Vec::<std::sync::Arc<pse_model::diagnostic::DiagnosticCause>>::new();
        let mut append = |goals: &[pse_math::engineering_accuracy::GoalResult]| {
            for failure in goals.iter().filter_map(|goal| goal.failure.as_ref()) {
                if !seen
                    .iter()
                    .any(|cause| std::sync::Arc::ptr_eq(cause, &failure.cause))
                {
                    diagnostics.push(
                        failure
                            .cause
                            .boundary_diagnostic(DiagnosticStage::ModelingQualification),
                    );
                    seen.push(failure.cause.clone());
                }
            }
        };
        match self.report() {
            Ok(RunReport::Horizon(_)) => {}
            Ok(RunReport::Modeling(results)) => {
                for result in results {
                    append(&result.completion.accuracy);
                }
            }
            Ok(RunReport::Simulation(report)) => append(&report.completion().accuracy),
            #[cfg(feature = "solver-diffsol")]
            Ok(RunReport::Shooting(report)) => append(&report.completion.accuracy),
            Ok(RunReport::Fit(report)) => {
                if let Some(completion) = &report.completion {
                    append(&completion.accuracy);
                }
            }
            Err(_) => {
                if let Some(failure) = &self.modeling_failure {
                    for result in &failure.completed {
                        append(&result.completion.accuracy);
                    }
                }
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::diagnostic::BoundaryClass as Class;
    #[test]
    fn modeling_failure_retains_class_rule_and_source_through_wrappers() {
        let declaration = pse_ids::SemanticId::from_bytes([9; 16]);
        let error = pse_modeling::ModelingError::Unsupported {
            declaration,
            capability: "synthetic capability".into(),
        };
        let wrapped =
            super::super::WorkflowError::Math(crate::math::MathRuntimeError::Compile(error.into()));
        let diagnostic = wrapped.boundary_diagnostic();
        assert_eq!(diagnostic.class, Class::Unsupported);
        assert_eq!(diagnostic.rule, DiagnosticRule::ModelingCapability);
        assert_eq!(diagnostic.sources, vec![declaration]);
        assert!(
            matches!(&diagnostic.observations["detail"], Observation::Text(value) if value == "synthetic capability")
        );
    }
    #[test]
    fn operation_context_preserves_typed_error_and_diagnostic() {
        let error = operation_context(
            super::super::WorkflowError::Canonical(
                pse_operations::canonical::CanonicalError::Timeout,
            ),
            "modeling_revision.context",
        );
        assert!(matches!(
            &error,
            super::super::WorkflowError::Canonical(
                pse_operations::canonical::CanonicalError::Timeout
            )
        ));
        let diagnostic = error.boundary_diagnostic();
        assert_eq!(
            diagnostic.code,
            pse_diagnostics::DiagnosticCode::RuntimeInfrastructure
        );
        assert_eq!(diagnostic.class, Class::Infrastructure);
        assert_eq!(diagnostic.rule, DiagnosticRule::WorkflowUnclassified);
        assert!(matches!(
            &diagnostic.observations["detail"],
            Observation::Text(detail) if detail == "canonical store request deadline expired"
        ));
    }
    #[test]
    fn shared_study_retention_failure_preserves_infrastructure_cause() {
        let cause = std::sync::Arc::new(super::super::WorkflowError::Canonical(
            pse_operations::canonical::CanonicalError::Timeout,
        ));
        let expected = cause.boundary_diagnostic();
        let error = operation_context(
            super::super::WorkflowError::Shared(cause.clone()),
            "study.record_study_attempt.retained_result",
        );
        assert!(matches!(
            &error,
            super::super::WorkflowError::Shared(retained)
                if std::sync::Arc::ptr_eq(retained, &cause)
        ));
        assert_eq!(
            serde_json::to_value(error.boundary_diagnostic()).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        assert_eq!(expected.class, Class::Infrastructure);
    }
    #[test]
    fn adapter_not_linked_is_unsupported() {
        use pse_backend_native::{
            ProblemError,
            routing::Requirements,
            solve::{Backend, Controls, SolveIntent, SolverSelection},
        };
        let facts = pse_backend_native::routing::oracle_facts(
            &pse_backend_native::OracleContract {
                identity: pse_ids::ContentHash::from_bytes([1; 32]),
                variables: vec![pse_backend_native::Variable {
                    id: pse_ids::SemanticId::from_bytes([1; 16]),
                    lower: f64::NEG_INFINITY,
                    upper: f64::INFINITY,
                }],
                rows: vec![pse_ids::SemanticId::from_bytes([2; 16])],
                derivatives: pse_kernels::DerivativeOrder::Second,
                smoothness: pse_kernels::DerivativeOrder::Second,
            },
            false,
            true,
        );
        let controls = Controls::default();
        // No adapter exposed or linked: automatic routing has no eligible route.
        let refused = Requirements {
            table: &pse_backend_native::execution::Table::new(&[]),
            facts: &facts,
            intent: SolveIntent::Root,
            numerical_psd: false,
            least_squares: false,
            controls: &controls,
            settings: &pse_backend_native::execution::BackendSettings::Default,
            sensitivity: false,
            context: pse_backend_native::routing::Context {
                snapshot: pse_backend_native::execution::Snapshot::observe(
                    &pse_backend_native::execution::Table::new(&[]),
                ),
                pending_classes: &[],
                structure: None,
                oracle: None,
                guards: &std::collections::BTreeMap::new(),
                budgets: None,
                coefficients: None,
                cone: None,
                factorable: None,
                certificate: None,
                prepared: &[],
                refusals: &std::collections::BTreeMap::new(),
            },
        };
        let error = refused.select(SolverSelection::Auto).unwrap_err();
        assert!(matches!(error, ProblemError::RouteRefused(_)), "{error:?}");
        let diagnostic = observed(&error, DiagnosticStage::Routing);
        assert_eq!(diagnostic.class, Class::Unsupported);
        assert_eq!(diagnostic.rule, DiagnosticRule::NativeRouteRefused);
        // An explicitly selected adapter outside the linked inventory is unavailable.
        let error = refused
            .select(SolverSelection::Explicit(Backend::Ipopt))
            .unwrap_err();
        assert!(
            matches!(error, ProblemError::RouteRefused(ref decision) if matches!(decision.refusal, Some(pse_backend_native::routing::Refusal::Unavailable(Backend::Ipopt)))),
            "{error:?}"
        );
        assert_eq!(
            observed(&error, DiagnosticStage::Routing).class,
            Class::Unsupported
        );
        // Internal invariants and numerical failures are never an invalid model.
        for (error, class) in [
            (ProblemError::internal("postcondition"), Class::Internal),
            (ProblemError::Cancelled, Class::Cancelled),
            (
                ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    detail: "deadline".into(),
                },
                Class::ResourceLimit,
            ),
            (ProblemError::numerical("factorization"), Class::Numerical),
            (
                ProblemError::Reuse {
                    backend: Backend::Ipopt,
                    refusal: pse_backend_native::ReuseRefusal::DroppedOptions(vec![
                        "mu_init".into(),
                    ]),
                },
                Class::Incompatible,
            ),
            (
                ProblemError::Provider(pse_kernels::ProviderError::Terminal("native".into())),
                Class::Infrastructure,
            ),
        ] {
            assert_eq!(
                observed(&error, DiagnosticStage::Native).class,
                class,
                "{error:?}"
            );
        }
        // A refused reuse names the option keys the step drops (PS-10).
        let refused = observed(
            &ProblemError::Reuse {
                backend: Backend::Ipopt,
                refusal: pse_backend_native::ReuseRefusal::DroppedOptions(vec![
                    "mu_init".into(),
                    "warm_start_bound_push".into(),
                ]),
            },
            DiagnosticStage::Native,
        );
        assert_eq!(refused.rule, DiagnosticRule::NativeReuse);
        assert!(matches!(
            &refused.observations["dropped_options"],
            Observation::Text(keys) if keys == "mu_init,warm_start_bound_push"
        ));
    }
    #[test]
    fn structural_failure_keeps_rows_and_columns() {
        let rows = vec![pse_ids::SemanticId::from_bytes([6; 16])];
        let columns = vec![pse_ids::SemanticId::from_bytes([7; 16])];
        let inner = pse_backend_native::ProblemError::Structural {
            mode: pse_backend_native::structural::Mode::Roots,
            rows: rows.clone(),
            columns: columns.clone(),
        };
        let block = pse_ids::SemanticId::from_bytes([5; 16]);
        // An injected inner solve retains the typed structural witness through MathError.
        let error = pse_math::MathError::Native {
            source_id: block,
            retained: inner.retained_bytes(),
            cause: pse_model::diagnostic::DiagnosticCause::new(inner),
        };
        let d = observed(&error, DiagnosticStage::Implicit);
        assert_eq!(d.class, Class::InvalidModel);
        assert_eq!(d.rule, DiagnosticRule::NativeStructural);
        let mut expected = vec![block, rows[0], columns[0]];
        expected.sort_unstable();
        assert_eq!(d.sources, expected);
    }
    #[test]
    fn unattributed_failure_never_acquires_a_guessed_source() {
        let e = pse_backend_native::ProblemError::Contract("presolve evaluation failed".into());
        assert!(observed(&e, DiagnosticStage::Presolve).sources.is_empty());
    }
    #[test]
    fn derivative_work_limit_retains_quantitative_witness_through_wrappers() {
        let source_id = pse_ids::SemanticId::from_bytes([8; 16]);
        let error = pse_math::MathError::WorkLimit {
            source_id,
            resource: "derivative expansion",
            required: 12,
            available: 3,
            components: 6,
        };
        let wrapped =
            super::super::WorkflowError::Math(crate::math::MathRuntimeError::Compile(error.into()));
        let diagnostic = wrapped.boundary_diagnostic();
        assert_eq!(diagnostic.class, Class::ResourceLimit);
        assert_eq!(diagnostic.rule, DiagnosticRule::MathLimit);
        assert_eq!(diagnostic.sources, vec![source_id]);
        assert!(matches!(
            diagnostic.observations["required_operations"],
            Observation::Integer(12)
        ));
        assert!(matches!(
            diagnostic.observations["available_operations"],
            Observation::Integer(3)
        ));
        assert!(matches!(
            diagnostic.observations["taylor_components"],
            Observation::Integer(6)
        ));
    }
    #[test]
    fn portable_recipe_byte_limit_keeps_facts_through_typed_projection() {
        use pse_diagnostics::{DiagnosticCode, TypedDiagnostic};
        let error = crate::math::portable::PortableError::Math(pse_math::MathError::ByteLimit {
            resource: "portable body payload",
            required: 67_108_865,
            available: 67_108_864,
        });
        assert_eq!(
            error.diagnostic_code(),
            Some(DiagnosticCode::RuntimeResourceLimit)
        );
        let projected =
            pse_model::diagnostic::project_typed(&error, DiagnosticStage::ModelingAdmission);
        assert_eq!(projected.class, Class::ResourceLimit);
        assert_eq!(projected.rule, DiagnosticRule::MathLimit);
        assert!(matches!(
            projected.observations["required_bytes"],
            Observation::Integer(67_108_865)
        ));
        assert!(matches!(
            projected.observations["available_bytes"],
            Observation::Integer(67_108_864)
        ));
        let legacy = pse_math::MathError::Limit("portable body payload");
        let projected =
            pse_model::diagnostic::project_typed(&legacy, DiagnosticStage::ModelingAdmission);
        assert_eq!(projected.rule, DiagnosticRule::MathLimit);
    }
    #[test]
    fn body_slot_limit_retains_required_and_available_slots_through_wrappers() {
        let error = pse_math::MathError::SlotLimit {
            required: 16_385,
            available: 16_384,
        };
        let wrapped =
            super::super::WorkflowError::Math(crate::math::MathRuntimeError::Compile(error.into()));
        let diagnostic = wrapped.boundary_diagnostic();
        assert_eq!(diagnostic.class, Class::ResourceLimit);
        assert_eq!(diagnostic.rule, DiagnosticRule::MathLimit);
        assert!(matches!(
            diagnostic.observations["required_slots"],
            Observation::Integer(16_385)
        ));
        assert!(matches!(
            diagnostic.observations["available_slots"],
            Observation::Integer(16_384)
        ));
    }
    #[test]
    fn instance_and_expression_are_retained_independently() {
        let instance = pse_ids::SemanticId::from_bytes([1; 16]);
        let source_id = pse_ids::SemanticId::from_bytes([2; 16]);
        let e = pse_math::MathError::Instance {
            occurrences: Default::default(),
            checked_sources: Default::default(),
            checked_members: Default::default(),
            instance,
            cause: Box::new(pse_math::MathError::Domain {
                source_id,
                requirement: "positive",
            }),
        };
        let d = observed(&e, DiagnosticStage::Evaluation);
        assert_eq!(d.sources, vec![instance, source_id]);
        assert_eq!(d.class, Class::TrialRejected);
    }
    #[test]
    fn compiler_syntax_retains_definition_and_byte_span() {
        let definition = pse_ids::SemanticId::from_bytes([3; 16]);
        let error = pse_compiler::workspace::CompileError::Syntax {
            definition,
            source_index: 2,
            error: std::sync::Arc::new(pse_authoring::dsl::parse_expr("x + )").unwrap_err()),
        };
        let d = super::super::WorkflowError::Math(crate::math::MathRuntimeError::Compile(error))
            .boundary_diagnostic();
        assert_eq!(d.sources, vec![definition]);
        assert_eq!(d.locations[0].start, Some(4));
        assert_eq!(d.locations[0].end, Some(5));
    }
    #[test]
    fn provider_recoverability_is_preserved_with_its_witness() {
        let d = observed(
            &pse_math::MathError::Provider {
                source_id: pse_ids::SemanticId::from_bytes([4; 16]),
                provider: pse_ids::SemanticId::from_bytes([5; 16]),
                cause: pse_kernels::ProviderError::OutsideEnvelope {
                    axis: "temperature".into(),
                    value: 100.,
                    lower: 200.,
                    upper: 400.,
                },
            },
            DiagnosticStage::Property,
        );
        assert_eq!(d.class, Class::TrialRejected);
        assert_eq!(d.sources.len(), 2);
        assert!(matches!(
            d.observations["provider_recoverable"],
            Observation::Boolean(true)
        ));
        let d = observed(
            &pse_kernels::ProviderError::Terminal("native failure".into()),
            DiagnosticStage::Property,
        );
        assert_eq!(d.class, Class::Infrastructure);
        assert!(matches!(
            d.observations["provider_recoverable"],
            Observation::Boolean(false)
        ));
    }
    #[test]
    fn mixed_aggregate_retains_every_repeated_typed_cause_and_detail() {
        let original = pse_engine::EngineError::Multiple {
            errors: vec![
                pse_engine::EngineError::Internal {
                    message: "first invariant".into(),
                },
                pse_engine::EngineError::Cancelled,
                pse_engine::EngineError::Internal {
                    message: "first invariant".into(),
                },
            ],
        };
        let diagnostic = super::super::WorkflowError::Engine(original).boundary_diagnostic();
        assert_eq!(
            diagnostic.code,
            pse_diagnostics::DiagnosticCode::DiagnosticAggregate
        );
        assert_eq!(diagnostic.rule, DiagnosticRule::DiagnosticAggregate);
        assert_eq!(diagnostic.causes.len(), 3);
        assert_eq!(
            diagnostic.causes[1].code,
            pse_diagnostics::DiagnosticCode::RuntimeCancelled
        );
        for index in [0, 2] {
            assert_eq!(
                diagnostic.causes[index].code,
                pse_diagnostics::DiagnosticCode::InternalInvariant
            );
            assert!(
                matches!(&diagnostic.causes[index].observations["detail"], Observation::Text(detail) if detail.contains("first invariant"))
            );
        }
    }
    #[test]
    fn detailed_taxonomy_survives_syntax_identity_units_evaluation_resource_cancel_panic_and_invariant()
     {
        use super::super::WorkflowError;
        use pse_diagnostics::DiagnosticCode as Code;
        let id = pse_ids::SemanticId::from_bytes([3; 16]);
        let expected = pse_quantity::QuantityTypeId::from_id(id);
        let actual =
            pse_quantity::QuantityTypeId::from_id(pse_ids::SemanticId::from_bytes([4; 16]));
        let cases = [
            (
                WorkflowError::Math(crate::math::MathRuntimeError::Compile(
                    pse_compiler::workspace::CompileError::Syntax {
                        definition: id,
                        source_index: 0,
                        error: std::sync::Arc::new(
                            pse_authoring::dsl::parse_expr("x + )").unwrap_err(),
                        ),
                    },
                )),
                Code::AuthoringParseSyntax,
                Class::InvalidModel,
            ),
            (
                WorkflowError::Math(crate::math::MathRuntimeError::Math(
                    pse_math::MathError::Quantity(pse_quantity::QuantityError::UnknownId {
                        kind: "quantity_type",
                        id,
                    }),
                )),
                Code::QuantityUnknownId,
                Class::InvalidModel,
            ),
            (
                WorkflowError::Math(crate::math::MathRuntimeError::Math(
                    pse_math::MathError::Quantity(pse_quantity::QuantityError::ContractMismatch {
                        component: pse_quantity::ContractComponent::Kind,
                        expected,
                        actual,
                    }),
                )),
                Code::QuantityContractMismatch,
                Class::InvalidModel,
            ),
            (
                WorkflowError::Math(crate::math::MathRuntimeError::Math(
                    pse_math::MathError::Evaluation {
                        source_id: id,
                        order: pse_kernels::DerivativeOrder::Value,
                        detail: "evaluation failed".into(),
                    },
                )),
                Code::MathEvaluation,
                Class::TrialRejected,
            ),
            (
                WorkflowError::Math(crate::math::MathRuntimeError::Limit("work")),
                Code::RuntimeResourceLimit,
                Class::ResourceLimit,
            ),
            (
                WorkflowError::Math(crate::math::MathRuntimeError::Cancelled),
                Code::RuntimeCancelled,
                Class::Cancelled,
            ),
            (
                WorkflowError::Math(crate::math::MathRuntimeError::Panic("caught panic".into())),
                Code::WorkflowPanic,
                Class::Internal,
            ),
            (
                WorkflowError::Internal("postcondition".into()),
                Code::WorkflowInternal,
                Class::Internal,
            ),
        ];
        for (error, code, class) in cases {
            let diagnostic = error.boundary_diagnostic();
            assert_eq!(diagnostic.code, code);
            assert_eq!(diagnostic.class, class);
            let encoded = serde_json::to_vec(&diagnostic).unwrap();
            let decoded: BoundaryDiagnostic = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(decoded.code, code);
            assert_eq!(decoded.failure_class(), code.class());
            assert_eq!(decoded.sources, diagnostic.sources);
        }
    }
    #[test]
    fn unit_refusal_keeps_both_complete_operand_contracts_through_runtime_and_codec() {
        let expected =
            pse_quantity::QuantityTypeId::from_id(pse_ids::SemanticId::from_bytes([5; 16]));
        let actual =
            pse_quantity::QuantityTypeId::from_id(pse_ids::SemanticId::from_bytes([6; 16]));
        let error = super::super::WorkflowError::Math(crate::math::MathRuntimeError::Math(
            pse_math::MathError::Quantity(pse_quantity::QuantityError::ContractMismatch {
                component: pse_quantity::ContractComponent::Basis,
                expected,
                actual,
            }),
        ));
        let diagnostic = error.boundary_diagnostic();
        let encoded = serde_json::to_vec(&diagnostic).unwrap();
        let decoded: BoundaryDiagnostic = serde_json::from_slice(&encoded).unwrap();
        let Observation::Contracts(contracts) = &decoded.observations["operands"] else {
            panic!("operand contracts were flattened")
        };
        assert_eq!(
            contracts.iter().map(|c| c.quantity).collect::<Vec<_>>(),
            vec![*expected.as_bytes(), *actual.as_bytes()]
        );
    }
}

#[cfg(test)]
mod admission_failure_tests {
    use super::*;
    use crate::math::MathRuntimeError;
    use crate::workflow::{AdmissionEffect, WorkflowError};
    use pse_model::diagnostic::BoundaryClass;

    fn scientific() -> WorkflowError {
        crate::workflow::scientific_refusal(
            BoundaryClass::InvalidModel,
            DiagnosticStage::ModelingDynamic,
            DiagnosticRule::ModelingDomain,
            vec![pse_ids::SemanticId::from_bytes([13; 16])],
            "missing immutable initial condition",
        )
    }
    fn at(effect: AdmissionEffect, cause: WorkflowError) -> WorkflowError {
        WorkflowError::Admission {
            effect,
            cause: Box::new(cause),
        }
    }
    #[test]
    fn admission_persists_only_owned_scientific_refusal_with_known_effects() {
        assert_eq!(
            admission_disposition(&scientific()),
            AdmissionDisposition::UnsettledEffects
        );
        for effect in [
            AdmissionEffect::NoPublication,
            AdmissionEffect::SourceAcknowledged {
                revision: "issued-source-revision".into(),
            },
        ] {
            let error = at(effect, scientific());
            assert_eq!(
                admission_disposition(&error),
                AdmissionDisposition::ScientificRefusal
            );
            let diagnostic = error.boundary_diagnostic();
            assert_eq!(diagnostic.class, BoundaryClass::InvalidModel);
            assert_eq!(diagnostic.stage, DiagnosticStage::ModelingDynamic);
            assert_eq!(
                diagnostic.sources,
                vec![pse_ids::SemanticId::from_bytes([13; 16])]
            );
        }
        let acknowledged = at(
            AdmissionEffect::SourceAcknowledged {
                revision: "issued-source-revision".into(),
            },
            scientific(),
        );
        assert!(
            matches!(&acknowledged.boundary_diagnostic().observations["source_revision"], Observation::Text(value) if value == "issued-source-revision")
        );
        let unknown = at(AdmissionEffect::UnresolvedPublication, scientific());
        assert_eq!(
            admission_disposition(&unknown),
            AdmissionDisposition::UnsettledEffects
        );
        assert_eq!(
            unknown.boundary_diagnostic().class,
            BoundaryClass::InvalidModel
        );
        let nested_unknown = at(AdmissionEffect::NoPublication, unknown);
        assert_eq!(
            admission_disposition(&nested_unknown),
            AdmissionDisposition::UnsettledEffects
        );
        assert_eq!(
            nested_unknown.boundary_diagnostic().class,
            BoundaryClass::InvalidModel
        );
    }
    #[test]
    fn capacity_deadline_internal_cancel_and_legacy_strings_never_become_permanent_science() {
        let errors = vec![
            WorkflowError::Resource("lazy descriptor bytes".into()),
            WorkflowError::Math(MathRuntimeError::Limit("compiler capacity")),
            WorkflowError::Math(MathRuntimeError::PreparationExpired),
            WorkflowError::Math(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    detail: "admission deadline".into(),
                },
            )),
            WorkflowError::Math(MathRuntimeError::Internal("poisoned producer".into())),
            WorkflowError::Internal("missing admitted projection".into()),
            WorkflowError::Integrity("corrupt admitted content".into()),
            WorkflowError::Math(MathRuntimeError::Cancelled),
            WorkflowError::Input("legacy ambiguous contract".into()),
            WorkflowError::Unsupported("runtime capability unavailable".into()),
        ];
        for cause in errors {
            let error = at(
                AdmissionEffect::SourceAcknowledged {
                    revision: "issued".into(),
                },
                cause,
            );
            assert_eq!(
                admission_disposition(&error),
                AdmissionDisposition::TransientOrUnclassified,
                "{error:?}"
            );
        }
        let failure = WorkflowError::Canonical(
            pse_operations::canonical::CanonicalError::SourceEditUnsettled {
                problem: "original-problem".into(),
                operation: "original-operation".into(),
                source: Box::new(pse_operations::canonical::CanonicalError::Timeout),
            },
        );
        let error = at(AdmissionEffect::UnresolvedPublication, failure);
        assert_eq!(
            admission_disposition(&error),
            AdmissionDisposition::UnsettledEffects
        );
        let WorkflowError::Admission { cause, .. } = error else {
            panic!("effect wrapper lost")
        };
        assert!(
            matches!(*cause, WorkflowError::Canonical(pse_operations::canonical::CanonicalError::SourceEditUnsettled { problem, operation, source }) if problem == "original-problem" && operation == "original-operation" && matches!(*source, pse_operations::canonical::CanonicalError::Timeout))
        );
    }
    #[test]
    fn source_owner_refusal_proof_remains_distinct_from_class_or_missing_receipt() {
        use pse_operations::canonical::CanonicalError;
        let refused = WorkflowError::Canonical(CanonicalError::SourceEditRefused {
            problem: "issued-problem".into(),
            operation: "issued-operation".into(),
            source: Box::new(CanonicalError::OperationReused),
        });
        assert_eq!(
            publication_failure_effect(&refused),
            AdmissionEffect::NoPublication
        );
        assert_eq!(
            publication_failure_effect(&WorkflowError::Canonical(CanonicalError::Timeout)),
            AdmissionEffect::UnresolvedPublication
        );
        assert_eq!(
            publication_failure_effect(&WorkflowError::Resource("memory".into())),
            AdmissionEffect::UnresolvedPublication
        );
    }
    #[test]
    fn equivalent_poisoned_runtime_and_math_callback_keep_internal_class() {
        let direct = MathRuntimeError::Internal("producer lock poisoned".into());
        let nested = MathRuntimeError::Math(
            MathRuntimeError::Internal("producer lock poisoned".into()).into_math(),
        );
        for error in [direct, nested] {
            let diagnostic = error.boundary_diagnostic(DiagnosticStage::Evaluation);
            assert_eq!(diagnostic.class, BoundaryClass::Internal);
            assert_eq!(
                diagnostic.code,
                pse_diagnostics::DiagnosticCode::InternalInvariant
            );
            assert_eq!(
                crate::math::strategy::runtime_failure(&error),
                pse_model::generated::enums::NumericalAttemptObservation::OperationalFailure
            );
        }
        let integrity =
            WorkflowError::Integrity("retained IPC failed validation".into()).boundary_diagnostic();
        assert_eq!(integrity.class, BoundaryClass::Infrastructure);
        assert!(matches!(
            integrity.observations["integrity_failure"],
            Observation::Boolean(true)
        ));
    }
    #[test]
    fn diagnostic_deadline_overflow_keeps_native_time_cause_and_never_permanent_science() {
        let error = crate::workflow::diagnostic_operation::DiagnosticOperation::new(
            &crate::CancelSource::new(),
            std::time::Duration::MAX,
            None,
        )
        .expect_err("unrepresentable diagnostic deadline must refuse");
        assert!(matches!(&error,
            WorkflowError::Math(MathRuntimeError::Solve(pse_backend_native::ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time, detail,
            })) if detail == "diagnostic deadline extent"));
        let diagnostic = DiagnosticProjection::boundary_diagnostic(
            &error,
            DiagnosticStage::NonlinearExplanation,
        );
        assert_eq!(diagnostic.class, BoundaryClass::ResourceLimit);
        assert_eq!(
            diagnostic.code,
            pse_diagnostics::DiagnosticCode::RuntimeTimeout
        );
        assert_eq!(diagnostic.rule, DiagnosticRule::NativeLimit);
        assert_eq!(diagnostic.stage, DiagnosticStage::NonlinearExplanation);
        assert_eq!(
            admission_disposition(&at(AdmissionEffect::NoPublication, error)),
            AdmissionDisposition::TransientOrUnclassified
        );
    }
    #[test]
    fn pool_capacity_and_pool_internal_failure_have_distinct_meaning() {
        use datafusion::common::DataFusionError;
        for (error, expected) in [
            (
                DataFusionError::ResourcesExhausted("allocation".into()),
                BoundaryClass::ResourceLimit,
            ),
            (
                DataFusionError::Internal("pool invariant".into()),
                BoundaryClass::Internal,
            ),
            (
                DataFusionError::Execution("pool service".into()),
                BoundaryClass::Infrastructure,
            ),
        ] {
            assert_eq!(
                MathRuntimeError::Pool(error)
                    .boundary_diagnostic(DiagnosticStage::Evaluation)
                    .class,
                expected
            );
        }
    }
}

#[cfg(test)]
mod canonical_settlement_projection_tests {
    use super::*;
    use pse_operations::canonical::CanonicalError;
    #[test]
    fn unresolved_canonical_effect_retains_both_causes_without_request_content() {
        let error = super::super::WorkflowError::Canonical(CanonicalError::SettlementUnresolved {
            owner: "research",
            operation: "original-op".into(),
            request: b"private-request".to_vec(),
            source: Box::new(CanonicalError::Timeout),
            readback: Some(Box::new(CanonicalError::IncompleteResponse)),
        });
        let diagnostic = error.boundary_diagnostic();
        assert_eq!(diagnostic.causes.len(), 2);
        assert!(
            matches!(diagnostic.observations.get("publication_effect"), Some(Observation::Text(value)) if value == "unresolved")
        );
        assert!(
            matches!(diagnostic.observations.get("canonical_operation"), Some(Observation::Text(value)) if value == "original-op")
        );
        let document = serde_json::to_string(&diagnostic).unwrap();
        assert!(!document.contains("private-request"));
    }
}
