// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Completion owns qualification and lineage. Exporters only copy this product.
use super::{RunReport, RunRequest, RunResult, WorkflowError};
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
        match &self.request {
            RunRequest::Modeling(requests) => {
                for (ordinal, request) in requests.iter().enumerate() {
                    let step = ordinal as i64;
                    let solved = request.model.model.solved();
                    let declaration = request.model.case.compiled().plan.structure();
                    let outcome = match &self.report {
                        Ok(RunReport::Modeling(r)) => r.get(ordinal).map(|r| &r.outcome),
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
                        step,
                        model_id: Some(solved.model()),
                        revision: Some(request.source.revision.identity()),
                        case_id: solved.case(),
                        instance_id: Some(solved.instance()),
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
                        objective_sense: declaration.objective().map(|o| match o.sense {
                            pse_math::binding::ObjectiveSense::Minimize => {
                                NativeObjectiveSense::Minimize
                            }
                            pse_math::binding::ObjectiveSense::Maximize => {
                                NativeObjectiveSense::Maximize
                            }
                        }),
                        objective_quantity_id: declaration.objective().map(|o| o.quantity.as_id()),
                        validation_error: native
                            .and_then(|r| r.validation_failure().map(ToString::to_string)),
                        error,
                        transformation: native
                            .and_then(|r| r.preprocessing.as_ref().map(|p| p.transformation)),
                    });

                    let preparation = request
                        .solve
                        .preparation_identity()
                        .map_err(crate::math::MathRuntimeError::from)?;
                    let selected = request
                        .solve
                        .request_identity()
                        .map_err(crate::math::MathRuntimeError::from)?;
                    let profile = crate::math::solves::profile_key(&request.profile)
                        .map_err(crate::math::MathRuntimeError::from)?;
                    let mut identity = FramedHasher::new(pse_ids::Frame::CompletedRequestV2);
                    identity
                        .hash(&request.source.revision.identity())
                        .id(&solved.instance().as_id())
                        .hash(&preparation)
                        .hash(&selected)
                        .hash(&profile)
                        .hash(&request.solve.numerics().key);
                    // A result is traceable to what seeded it and to native state it reused:
                    // the previous step's seed and the reuse of retained native state enter
                    // its lineage identity (F25).
                    frame_start(&mut identity, native);
                    let mut actual_environment =
                        FramedHasher::new(pse_ids::Frame::CompletedEnvironmentV1);
                    actual_environment
                        .hash(&environment)
                        .hash(&pse_buildinfo::BUILD_IDENTITY);
                    if let Some(native) = native {
                        actual_environment.str(native.backend.as_str());
                        for (name, value) in &native.provenance {
                            actual_environment.str(name).str(value);
                        }
                    }
                    let lineage = solved.lineage();
                    product.lineage.push(run_lineage::Row {
                        run_id: self.run_id,
                        step,
                        model_id: lineage.model_id,
                        revision: request.source.revision.identity(),
                        case_id: lineage.case_id,
                        instance_id: lineage.instance_id,
                        fit_id: lineage.fit_id,
                        request_identity: identity.finish_hash(),
                        preparation_identity: preparation,
                        profile_identity: profile,
                        numerical_identity: request.solve.numerics().key,
                        physical_identity: request.source.physical.key,
                        environment_identity: actual_environment.finish_hash(),
                    });
                }
            }
            RunRequest::Simulation(p) => {
                let trajectory = match &self.report {
                    Ok(RunReport::Simulation(r)) => Some(r.as_ref()),
                    _ => None,
                };
                let r = trajectory.map(|t| t.report.as_ref());
                let profile = super::dynamics::profile_identity(p.profile());
                let mut row = header(
                    self,
                    ComputationKind::Simulation,
                    p.source.revision.identity(),
                    profile,
                );
                row.state = if r.is_some() {
                    NativeRunState::Native
                } else {
                    NativeRunState::Rejected
                };
                row.trajectory_termination = r.map(|r| r.termination);
                row.backend = p.profile().resolved_method().ok().map(|m| match m {
                    pse_backend_native::dynamics::Method::Idas => NativeBackend::Idas,
                    _ => NativeBackend::Diffsol,
                });
                row.candidate_available = r.is_some_and(|r| !r.samples.is_empty());
                row.completed_time = r.map(|r| r.completed_time).filter(|v| v.is_finite());
                row.completed_samples = r.map(|r| r.samples.len() as i64);
                row.feasible = trajectory
                    .filter(|t| t.checks_complete)
                    .map(|t| t.checks.iter().all(|c| c.satisfied));
                row.qualification = if trajectory.is_some_and(|t| t.accepted) {
                    NativeQualification::Feasible
                } else {
                    NativeQualification::Unqualified
                };
                row.validation_error =
                    trajectory.and_then(|t| t.validation_error.as_ref().map(ToString::to_string));
                row.error = row
                    .error
                    .or_else(|| r.and_then(|r| r.error.as_ref().map(ToString::to_string)));
                product.computation = Some(row);
                let mut actual_environment =
                    FramedHasher::new(pse_ids::Frame::CompletedEnvironmentV1);
                actual_environment
                    .hash(&environment)
                    .hash(&pse_buildinfo::BUILD_IDENTITY);
                #[cfg(feature = "solver-diffsol")]
                if r.is_some() {
                    actual_environment.str(&pse_backend_native::dynamics::settings_identity(
                        p.profile(),
                    ));
                }
                let lineage = p.solved.lineage();
                product.lineage.push(run_lineage::Row {
                    run_id: self.run_id,
                    step: 0,
                    model_id: lineage.model_id,
                    revision: p.source.revision.identity(),
                    case_id: lineage.case_id,
                    instance_id: lineage.instance_id,
                    fit_id: lineage.fit_id,
                    request_identity: p.identity(),
                    preparation_identity: p.identity(),
                    profile_identity: profile,
                    numerical_identity: p.numerics().key,
                    physical_identity: p.source.physical.key,
                    environment_identity: actual_environment.finish_hash(),
                });
            }
            RunRequest::Fit(p) => {
                let r = match &self.report {
                    Ok(RunReport::Fit(r)) => Some(r.as_ref()),
                    _ => None,
                };
                let native = r.and_then(|r| r.solve.as_ref());
                let physical = p.source.physical.key;
                let p = &p.problem;
                let mut row = header(self, ComputationKind::Fit, p.source_identity, p.profile_key);
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
                row.validation_error = native
                    .and_then(|s| s.validation_failure().map(ToString::to_string))
                    .or_else(|| r.and_then(|r| r.validation_error.as_ref().map(|d| d.to_string())));
                row.error = row
                    .error
                    .or_else(|| r.and_then(|r| r.diagnostic.as_ref().map(ToString::to_string)));
                product.computation = Some(row);
                let mut actual_environment =
                    FramedHasher::new(pse_ids::Frame::CompletedEnvironmentV1);
                actual_environment
                    .hash(&environment)
                    .hash(&pse_buildinfo::BUILD_IDENTITY);
                if let Some(native) = native {
                    actual_environment.str(native.backend.as_str());
                    for (name, value) in &native.provenance {
                        actual_environment.str(name).str(value);
                    }
                }
                let lineage = p.lineage.lineage();
                product.lineage.push(run_lineage::Row {
                    run_id: self.run_id,
                    step: 0,
                    model_id: lineage.model_id,
                    revision: p.source_identity,
                    case_id: lineage.case_id,
                    instance_id: lineage.instance_id,
                    fit_id: lineage.fit_id,
                    request_identity: p.key,
                    preparation_identity: p.key,
                    profile_identity: p.profile_key,
                    numerical_identity: p.numerics.key,
                    physical_identity: physical,
                    environment_identity: actual_environment.finish_hash(),
                });
            }
        }
        Ok(product)
    }
}
/// Frame the start a native step actually used: its submitted seed by content (never by
/// the run that produced it), the predecessor step, a partial start, and whether the step
/// reused retained native state.
fn frame_start(h: &mut FramedHasher, native: Option<&pse_backend_native::solve::SolveReport>) {
    let Some(native) = native else {
        h.bool(false);
        return;
    };
    h.bool(true).bool(native.evidence.reused_native_state);
    let Some(receipt) = &native.start_receipt else {
        h.bool(false);
        return;
    };
    h.bool(true).bool(receipt.submitted);
    h.u64(receipt.previous_attempt.map_or(0, |a| a as u64 + 1));
    match receipt.seed.as_ref().filter(|_| receipt.submitted) {
        Some(seed) => {
            h.bool(true).hash(&seed.content_key());
        }
        None => {
            h.bool(false);
        }
    }
    let partial = receipt.sparse_seed.as_ref();
    h.u64(partial.map_or(0, |p| p.len() as u64));
    for (id, value) in partial.into_iter().flatten() {
        h.id(id).u64(value.to_bits());
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
