// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Completion owns qualification and lineage. Exporters only copy this product.
use super::{RunReport, RunRequest, RunResult, WorkflowError, contract};
use crate::math::solves::Outcome;
use pse_ids::{ContentHash, FramedHasher};
use pse_model::generated::{
    enums::*,
    runtime::{computation_runs, run_lineage, solve_runs},
};

/// Immutable semantic projection shared by Arrow, Python and publication.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Completion {
    /// Final physical closure and usability decisions, never recomputed by an exporter.
    pub assessments: Vec<pse_model::generated::runtime::candidate_assessments::Row>,
    /// Source-attributed final errors and unavailable fit diagnostics.
    pub diagnostics: Vec<pse_model::diagnostic::BoundaryDiagnostic>,
    /// Algebraic outcomes, including requested steps that were not attempted.
    pub solves: Vec<solve_runs::Row>,
    /// Dynamic or fitting outcome, with candidate evidence separate from qualification.
    pub computation: Option<computation_runs::Row>,
    /// Complete request lineage, independent of this execution's unique identity.
    pub lineage: Vec<run_lineage::Row>,
}

impl RunResult {
    /// The joined projection; observing it performs no mathematical work.
    pub fn completion(&self) -> Result<&Completion, &WorkflowError> {
        self.completion.as_ref().map_err(AsRef::as_ref)
    }

    pub(super) fn capture_completion(&self) -> Result<Completion, WorkflowError> {
        let mut product = Completion {
            assessments: self.assessments.clone(),
            diagnostics: self.capture_diagnostics(),
            ..Default::default()
        };
        let environment = pse_math::context()
            .map_err(super::math)?
            .environment
            .identity();
        let mut lineage =
            |revision: &super::ModelRevision,
             case,
             preparation,
             selected_request,
             profile,
             numerical,
             native: Option<&pse_backend_native::solve::SolveReport>| {
                let mut request = FramedHasher::new("pse.completed.request.v1");
                request
                    .hash(&revision.identity())
                    .id(&case)
                    .hash(&preparation)
                    .hash(&selected_request)
                    .hash(&profile)
                    .hash(&numerical);
                let mut actual_environment = FramedHasher::new("pse.completed.environment.v1");
                actual_environment
                    .hash(&environment)
                    .hash(&pse_buildinfo::BUILD_IDENTITY);
                if let Some(native) = native {
                    actual_environment.str(native.backend.as_str());
                    for (name, value) in &native.provenance {
                        actual_environment.str(name).str(value);
                    }
                }
                product.lineage.push(run_lineage::Row {
                    run_id: self.run_id,
                    step: product.lineage.len() as i64,
                    model_id: revision.0.row.model_id,
                    revision: revision.identity(),
                    case_id: case,
                    request_identity: request.finish_hash(),
                    preparation_identity: preparation,
                    profile_identity: profile,
                    numerical_identity: numerical,
                    physical_identity: revision.0.physical.key,
                    environment_identity: actual_environment.finish_hash(),
                });
            };
        match &self.request {
            RunRequest::Solves(steps) => {
                for (ordinal, request) in steps.iter().enumerate() {
                    let declaration = request
                        .revision
                        .0
                        .row
                        .cases
                        .iter()
                        .find(|c| c.case_id == request.case)
                        .ok_or_else(|| contract("selected declaration missing at completion"))?;
                    let outcome = match &self.report {
                        Ok(RunReport::Solves(r)) => r.outcomes.get(ordinal),
                        _ => None,
                    };
                    let native = match outcome {
                        Some(Outcome::Native(r)) => Some(r.as_ref()),
                        _ => None,
                    };
                    let constant = match outcome {
                        Some(Outcome::Constant(r)) => Some(r.as_ref()),
                        _ => None,
                    };
                    let quality = native
                        .and_then(|r| r.quality.as_ref())
                        .or_else(|| constant.map(|r| &r.quality));
                    let observation = native
                        .and_then(|r| r.observation.as_ref())
                        .or_else(|| constant.map(|r| &r.observation));
                    let candidate = native.and_then(|r| r.candidate.as_ref());
                    let error = match (&self.report, outcome) {
                        (Err(e), _) => Some(e.to_string()),
                        (_, Some(Outcome::Rejected(e))) => Some(e.to_string()),
                        _ => None,
                    };
                    product.solves.push(solve_runs::Row {
                        run_id: self.run_id,
                        step: ordinal as i64,
                        model_id: Some(request.revision.0.row.model_id),
                        revision: Some(request.revision.identity()),
                        case_id: Some(request.case),
                        backend: native.map(|r| r.backend),
                        native_code: native.map(|r| r.termination.code),
                        native_status: native.map(|r| r.termination.name.clone()),
                        state: if native.is_some() {
                            NativeRunState::Native
                        } else if error.is_some() {
                            NativeRunState::Rejected
                        } else if constant.is_some() {
                            NativeRunState::ConstantEvaluation
                        } else {
                            NativeRunState::Unattempted
                        },
                        termination: native.map(|r| r.termination.category),
                        assurance: native
                            .map_or(NativeAssurance::None, |r| r.termination.assurance),
                        qualification: native.map_or_else(
                            || {
                                if quality.is_some_and(|q| q.feasible()) {
                                    NativeQualification::Feasible
                                } else {
                                    NativeQualification::Unqualified
                                }
                            },
                            |r| r.qualification,
                        ),
                        candidate_kind: candidate
                            .map(|c| c.kind)
                            .or_else(|| constant.map(|_| NativeCandidateKind::ConstantEvaluation)),
                        feasible: quality.map(|q| q.feasible()),
                        objective: observation
                            .and_then(|o| o.objective)
                            .or_else(|| candidate.and_then(|c| c.objective)),
                        objective_sense: declaration.objective.as_ref().map(|o| o.sense),
                        objective_quantity_id: declaration
                            .objective
                            .as_ref()
                            .map(|o| o.quantity_id),
                        validation_error: native.and_then(|r| r.validation_error.clone()),
                        error,
                        transformation: native
                            .and_then(|r| r.preprocessing.as_ref().map(|p| p.transformation)),
                    });
                    lineage(
                        &request.revision,
                        request.case,
                        request
                            .solve
                            .preparation_identity()
                            .map_err(crate::math::MathRuntimeError::from)?,
                        request
                            .solve
                            .request_identity()
                            .map_err(crate::math::MathRuntimeError::from)?,
                        crate::math::solves::profile_key(&request.profile)
                            .map_err(crate::math::MathRuntimeError::from)?,
                        request.solve.numerics().key,
                        native,
                    );
                }
            }
            RunRequest::Simulation(p) => {
                let r = match &self.report {
                    Ok(RunReport::Simulation(r)) => Some(r.as_ref()),
                    _ => None,
                };
                let mut row = header(
                    self,
                    ComputationKind::Simulation,
                    p.revision.identity(),
                    p.profile_key,
                );
                row.state = if r.is_some() {
                    NativeRunState::Native
                } else {
                    NativeRunState::Rejected
                };
                row.trajectory_termination = r.map(|r| r.termination);
                row.backend = r.map(|_| NativeBackend::Diffsol);
                row.candidate_available = r.is_some_and(|r| !r.samples.is_empty());
                row.completed_time = r.map(|r| r.completed_time).filter(|v| v.is_finite());
                row.completed_samples = r.map(|r| r.samples.len() as i64);
                row.error = row
                    .error
                    .or_else(|| r.and_then(|r| r.error.as_ref().map(ToString::to_string)));
                product.computation = Some(row);
                lineage(
                    &p.revision,
                    p.declaration.dynamic_id,
                    p.key,
                    p.key,
                    p.profile_key,
                    p.numerics.key,
                    None,
                );
            }
            RunRequest::Fit(p) => {
                let r = match &self.report {
                    Ok(RunReport::Fit(r)) => Some(r.as_ref()),
                    _ => None,
                };
                let native = r.and_then(|r| r.solve.as_ref());
                let p = &p.problem;
                let mut row = header(
                    self,
                    ComputationKind::Fit,
                    p.revision.identity(),
                    p.profile_key,
                );
                row.state = if native.is_some() {
                    NativeRunState::Native
                } else if r.is_some() {
                    NativeRunState::ConstantEvaluation
                } else {
                    NativeRunState::Rejected
                };
                row.termination = native.map(|s| s.termination.category);
                row.backend = native.map(|s| s.backend);
                row.native_code = native.map(|s| s.termination.code);
                row.native_status = native.map(|s| s.termination.name.clone());
                row.qualification =
                    native.map_or(NativeQualification::Unqualified, |s| s.qualification);
                row.candidate_kind = native.and_then(|s| s.candidate.as_ref().map(|c| c.kind));
                row.candidate_available = r.is_some_and(|r| r.candidate.is_some());
                if native.is_none() && row.candidate_available {
                    row.candidate_kind = Some(NativeCandidateKind::ConstantEvaluation);
                }
                row.feasible = r.and_then(|r| r.quality.as_ref().map(|q| q.feasible()));
                row.estimate_qualified = r.map(super::FitReport::estimate_qualified);
                row.response_available = r.map(|r| r.responses.is_some());
                row.response_rank = r.and_then(|r| r.rank.map(|v| v as i64));
                row.response_condition = r.and_then(super::FitReport::response_condition);
                row.validation_error = native.and_then(|s| s.validation_error.clone());
                row.error = row.error.or_else(|| r.and_then(|r| r.diagnostic.clone()));
                product.computation = Some(row);
                lineage(
                    &p.revision,
                    p.declaration.fit_id,
                    p.key,
                    p.key,
                    p.profile_key,
                    p.numerics.key,
                    native,
                );
            }
        }
        Ok(product)
    }
}
fn header(
    result: &RunResult,
    kind: ComputationKind,
    source_identity: ContentHash,
    profile_identity: ContentHash,
) -> computation_runs::Row {
    computation_runs::Row {
        run_id: result.run_id,
        kind,
        source_identity,
        profile_identity,
        state: NativeRunState::Rejected,
        termination: None,
        trajectory_termination: None,
        backend: None,
        native_code: None,
        native_status: None,
        qualification: NativeQualification::Unqualified,
        candidate_kind: None,
        candidate_available: false,
        feasible: None,
        completed_time: None,
        completed_samples: None,
        estimate_qualified: None,
        response_available: None,
        response_rank: None,
        response_condition: None,
        validation_error: None,
        error: result.report.as_ref().err().map(ToString::to_string),
    }
}
