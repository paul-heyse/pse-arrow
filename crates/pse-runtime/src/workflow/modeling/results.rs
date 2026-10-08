// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Knowledge checks supplement original-space solver qualification without rewriting termination.
use super::cases::ModelingSolvePreparation;
use super::*;
use crate::math::solves::Outcome;
use pse_compiler::workspace::{ModelingHint, ModelingOutput, ObjectiveBound, Profile};
use pse_math::binding::CaseValues;
use pse_model::generated::identities::RunId;
use pse_modeling::annotation::AnnotationValue;
use std::{collections::BTreeSet, sync::Arc};

use pse_model::generated::enums::{ModelingCheckBasis as Basis, ModelingCheckKind as CheckKind};
/// Registry-owned check and report rows are also the public Rust values.
pub use pse_model::generated::runtime::modeling_checks::Row as ModelingCheck;
pub use pse_model::generated::runtime::modeling_reports::Row as ModelingReport;

/// Native temporal closure is evaluated in the original physical inventory space.
/// The native report retains the corresponding point's actual mode and segment facts.
pub(super) fn temporal_closure_check(
    run_id: RunId,
    source: DeclarationId,
    target: SemanticId,
    point_index: usize,
    time: f64,
    residual: f64,
    tolerance: f64,
) -> ModelingCheck {
    ModelingCheck {
        step: 0,
        run_id,
        sample_index: point_index as i64,
        time: Some(time),
        target_id: target,
        source_id: source,
        kind: CheckKind::Closure,
        value: residual,
        tolerance: Some(tolerance),
        satisfied: residual.abs() <= tolerance,
        within_validity: None,
        extrapolation_allowed: None,
        basis: Basis::Point,
        layer: None,
        claim_id: None,
        claim_owner: None,
        claim_owner_lineage: Vec::new(),
        coverage_id: None,
        evidence_id: None,
        form_id: None,
        call_id: None,
        selected_records: Vec::new(),
        dependencies: Vec::new(),
        input_values: Vec::new(),
        applicability_outcome: None,
        applicability_basis: None,
        permission_ids: Vec::new(),
        unknown_allowed: None,
        observation_instance: None,
        applicability_required: None,
        applicability_reason: None,
        applicability_permissions: Vec::new(),
    }
}
/// An original coordinate initial row remains required after composite stock lowering.
pub(super) fn temporal_initial_check(
    run_id: RunId,
    source: DeclarationId,
    target: SemanticId,
    time: f64,
    residual: f64,
    tolerance: f64,
) -> ModelingCheck {
    let mut check = temporal_closure_check(run_id, source, target, 0, time, residual, tolerance);
    check.kind = CheckKind::OriginalEquation;
    check
}
fn report_transfer_context(
    ty: &pse_modeling::Type,
) -> Result<Option<pse_model::generated::structures::ModelingTransferContext>, WorkflowError> {
    use pse_model::generated::{
        enums::ModelingTransferDirection as Direction, structures::ModelingTransferContext,
    };
    use pse_modeling::{BoundaryRef, PhysicalRefinement, TransferDirection};
    match ty.physical_refinement() {
        None => Ok(None),
        Some(PhysicalRefinement::Transfer {
            boundary:
                BoundaryRef::Bound {
                    instance,
                    declaration,
                    coordinates,
                },
            direction,
        }) => Ok(Some(ModelingTransferContext {
            instance: *instance,
            boundary: *declaration,
            coordinates: coordinates
                .iter()
                .map(pse_modeling::specialize::Value::identity)
                .collect(),
            direction: match direction {
                TransferDirection::Into => Direction::Into,
                TransferDirection::OutOf => Direction::OutOf,
            },
        })),
        Some(PhysicalRefinement::Transfer {
            boundary: BoundaryRef::Declared(_),
            ..
        }) => Err(contract("reported transfer requires an actual bound owner")),
        Some(_) => Err(contract(
            "reported value requires a reconstructed physical contract",
        )),
    }
}
/// Completed solve and explicit model-level qualification. Diagnostic evidence never
/// changes the native report or turns a failed local solve into infeasibility proof.
#[derive(Clone, Debug)]
pub struct ModelingResult(Arc<ModelingResultData>);
/// Immutable result storage shared with every retained public handle.
#[derive(Debug)]
pub struct ModelingResultData {
    pub outcome: Outcome,
    /// Ordered shared numerical operations; absent only for a producer not yet traced.
    pub strategy: Option<crate::math::solves::StrategyTrace>,
    pub values: CaseValues,
    pub checks: Vec<ModelingCheck>,
    pub reports: Vec<ModelingReport>,
    pub run_id: RunId,
    /// Actual zero-based original attempt supplied by the completion owner.
    pub(in crate::workflow) original_attempt: usize,
    runtime: Runtime,
    /// Projection of `completion`; never decided separately.
    pub accepted: bool,
    /// The §16.6 candidate-use completion shared with the published assessment.
    pub(in crate::workflow) completion: crate::workflow::numerics::Completed,
    pub validation_error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    pub prepared: ModelingSolvePreparation,
    _owner: Arc<pse_columnar::AllocationLease>,
    _native_owner: Arc<pse_columnar::AllocationLease>,
}
impl std::ops::Deref for ModelingResult {
    type Target = ModelingResultData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
fn stamp_start(outcome: &mut Outcome, run: RunId, attempt: usize) {
    if let Outcome::Native(native) = outcome {
        if let Some(seed) = &mut native.warm_start {
            seed.origin = Some(pse_backend_native::solve::SeedOrigin {
                run: Some(run),
                attempt,
            });
        }
        if let Some(receipt) = &mut native.start_receipt
            && let Some(previous) = receipt.previous_attempt
            && let Some(seed) = &mut receipt.seed
        {
            seed.origin = Some(pse_backend_native::solve::SeedOrigin {
                run: Some(run),
                attempt: previous,
            });
        }
    }
}
impl ModelingResult {
    /// Bind an original-permitted candidate as the starting point of a scalar path.
    /// Auxiliary reports cannot establish this originating scientific point.
    /// # Errors
    /// Missing candidate, original permission, source binding or a valid orientation.
    pub fn path_start(
        &self,
        orientation: Vec<f64>,
    ) -> Result<crate::math::solves::paths::PathStart, WorkflowError> {
        let report = match &self.outcome {
            Outcome::Native(report) => report,
            _ => {
                return Err(contract(
                    "path origin requires an original native candidate",
                ));
            }
        };
        let candidate = report
            .candidate
            .as_ref()
            .ok_or_else(|| contract("path origin requires a candidate"))?;
        let start = crate::math::solves::paths::PathStart::admitted(
            &self.prepared.solve,
            candidate.primal.clone(),
            orientation,
            self.completion.decision.clone(),
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        let owner = self.runtime.native().reserve(
            "modeling:path-origin",
            start
                .retained_bytes()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        Ok(start.with_owner(owner))
    }
    /// Retain this original-permitted point as a sample for the shared secant predictor.
    /// A seed-only result cannot establish an accepted sequence anchor.
    /// # Errors
    /// No original permission/candidate or invalid parameter/semantic coordinates.
    pub fn prediction_anchor(
        &self,
        parameter: f64,
    ) -> Result<crate::math::prediction::Anchor, WorkflowError> {
        let report = match &self.outcome {
            Outcome::Native(report) => report,
            _ => {
                return Err(contract(
                    "prediction anchor requires an original native candidate",
                ));
            }
        };
        let candidate = report
            .candidate
            .as_ref()
            .ok_or_else(|| contract("prediction anchor requires a candidate"))?;
        let key = self
            .prepared
            .solve
            .semantic_point_key(&candidate.primal)
            .map_err(crate::math::MathRuntimeError::from)?;
        let anchor = crate::math::prediction::Anchor::admitted(
            key,
            report.variables.clone(),
            candidate.primal.clone(),
            parameter,
            self.completion.decision.clone(),
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        let owner = self.runtime.native().reserve(
            "modeling:prediction-anchor",
            anchor
                .retained_bytes()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        Ok(anchor.with_owner(owner))
    }
    /// Produce a bounded secant start from two independently original-permitted samples.
    /// Exactly one authored parameter may vary; fixed scientific and numerical meaning is
    /// checked against both samples. The extrapolation is limited to one observed interval.
    /// # Errors
    /// Changed fixed dependencies, unrelated or coincident samples, unavailable permission,
    /// a connected branch request without transport, or a nonfinite proposal.
    pub fn secant_prediction(
        &self,
        older: &Self,
        target: &ModelingSolvePreparation,
        branch: pse_model::strategy::BranchPolicy,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<crate::math::prediction::Proposal, WorkflowError> {
        let (history, parameter) = self.secant_product(older, target)?;
        let selected = crate::math::prediction::select(
            crate::math::prediction::SelectionRequest {
                mechanism: crate::math::prediction::ProposalMechanism::Secant {
                    history: &history,
                    parameter,
                    scaled_step_limit: 1.,
                },
                permission: &self.completion.decision,
                source: history.source(),
                target: target
                    .solve
                    .original_identity()
                    .map_err(crate::math::MathRuntimeError::from)?,
                branch,
            },
            execution,
        )
        .map_err(|error| crate::math::MathRuntimeError::from(error.into_problem()))?;
        let crate::math::prediction::SelectedProposal::Secant { proposal } = selected else {
            return Err(contract("secant selector returned another producer"));
        };
        let owner = self.runtime.native().reserve(
            "modeling:secant-proposal",
            proposal
                .retained_bytes()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        Ok(proposal.with_owner(owner))
    }
    fn secant_product(
        &self,
        older: &Self,
        target: &ModelingSolvePreparation,
    ) -> Result<(crate::math::prediction::SecantHistory, f64), WorkflowError> {
        self.prediction_sample()?
            .secant_product(&older.prediction_sample()?, target)
    }
    /// Use the same original-qualified proposal owner for retained and in-memory points.
    pub(crate) fn available_prediction(
        &self,
        older: Option<&Self>,
        target: &ModelingSolvePreparation,
        branch: pse_model::strategy::BranchPolicy,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<crate::math::prediction::Proposal, WorkflowError> {
        self.prediction_sample()?.available_prediction(
            older.map(Self::prediction_sample).transpose()?.as_ref(),
            target,
            branch,
            execution,
        )
    }
    pub(crate) fn portable_prediction(&self) -> Result<Option<PortablePrediction>, WorkflowError> {
        if !self.completion.decision.permits_use() {
            return Ok(None);
        }
        let Outcome::Native(report) = &self.outcome else {
            return Ok(None);
        };
        let Some(candidate) = &report.candidate else {
            return Ok(None);
        };
        let key = self
            .prepared
            .solve
            .semantic_point_key(&candidate.primal)
            .map_err(crate::math::MathRuntimeError::from)?;
        let root =
            self.root_predictor()
                .ok()
                .map(|predictor| {
                    let values = report.observation.as_ref().ok_or_else(|| {
                        contract("qualified root factor lost its original values")
                    })?;
                    Ok::<_, WorkflowError>(PortableRootPoint {
                        key: predictor.factor().key(),
                        values: values.values.iter().map(|value| value.to_bits()).collect(),
                    })
                })
                .transpose()?;
        Ok(Some(PortablePrediction {
            key,
            coordinates: report.variables.clone(),
            primal: candidate
                .primal
                .iter()
                .map(|value| value.to_bits())
                .collect(),
            permission: self.completion.decision.clone(),
            root,
        }))
    }
    fn prediction_sample(&self) -> Result<PredictionSample, WorkflowError> {
        let receipt = self
            .portable_prediction()?
            .ok_or_else(|| contract("prediction needs an original-permitted native point"))?;
        Ok(PredictionSample {
            prepared: self.prepared.clone(),
            runtime: self.runtime.clone(),
            point: Arc::new(pse_columnar::Leased::new(
                Arc::new(receipt),
                self._native_owner.clone(),
            )),
            root: self.root_predictor(),
        })
    }
    /// Track a genuinely demanded parameter change with a retained original KKT factor.
    /// Coverage and work remain library observations; this endpoint needs original correction.
    /// # Errors
    /// Source permission, exact factor point, fixed meaning or bounded native operation refusal.
    pub fn activity_prediction(
        &self,
        target: &crate::math::solves::PreparedSolve,
        advance: &pse_backend_native::kkt::Advance,
        segments: usize,
        limits: pse_backend_native::kkt::activity::Limits,
        branch: pse_model::strategy::BranchPolicy,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<
        (
            crate::math::prediction::Proposal,
            pse_backend_native::kkt::path::PathPrediction,
        ),
        WorkflowError,
    > {
        let parameters = target
            .related_target_parameters(&self.prepared.solve, advance.parameters())
            .map_err(crate::math::MathRuntimeError::from)?;
        let Outcome::Native(report) = &self.outcome else {
            return Err(contract(
                "activity prediction requires an original native candidate",
            ));
        };
        let candidate = report
            .candidate
            .as_ref()
            .ok_or_else(|| contract("activity prediction requires a candidate"))?;
        if report.variables != advance.variables() || report.rows != advance.rows() {
            return Err(contract(
                "activity factor original inventories differ from the permitted source",
            ));
        }
        let source = self
            .prepared
            .solve
            .semantic_point_key(&candidate.primal)
            .map_err(crate::math::MathRuntimeError::from)?;
        let selected = crate::math::prediction::select(
            crate::math::prediction::SelectionRequest {
                mechanism: crate::math::prediction::ProposalMechanism::Activity {
                    advance,
                    parameters: &parameters,
                    segments,
                    limits,
                },
                permission: &self.completion.decision,
                source,
                target: target
                    .original_identity()
                    .map_err(crate::math::MathRuntimeError::from)?,
                branch,
            },
            execution,
        )
        .map_err(|error| crate::math::MathRuntimeError::from(error.into_problem()))?;
        let crate::math::prediction::SelectedProposal::Activity {
            proposal,
            path,
            fallback,
        } = selected
        else {
            return Err(contract("activity selector returned another producer"));
        };
        if fallback.is_some() {
            return Err(contract(
                "explicit activity request unexpectedly selected a fallback",
            ));
        }
        let owner = self.runtime.native().reserve(
            "modeling:activity-proposal",
            proposal
                .retained_bytes()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        Ok((proposal.with_owner(owner), path))
    }
    /// Consume an explicitly demanded convex QP path in named original coordinates.
    /// The native producer admits the unchanged QP family; its endpoint remains a start
    /// requiring target screening and independent original correction.
    /// # Errors
    /// Missing original permission, source/coordinates mismatch or native path refusal.
    pub fn qp_prediction(
        &self,
        target: &crate::math::solves::PreparedSolve,
        request: pse_backend_native::kkt::path::qp::Request<'_>,
        branch: pse_model::strategy::BranchPolicy,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<
        (
            crate::math::prediction::Proposal,
            pse_backend_native::kkt::path::qp::Outcome,
        ),
        WorkflowError,
    > {
        let Outcome::Native(report) = &self.outcome else {
            return Err(contract(
                "QP prediction requires an original native candidate",
            ));
        };
        let candidate = report
            .candidate
            .as_ref()
            .ok_or_else(|| contract("QP prediction requires a candidate"))?;
        execution
            .check()
            .map_err(crate::math::MathRuntimeError::from)?;
        let (mut previous, _previous_owner) = self
            .prepared
            .solve
            .convex_qp_problem(self.runtime.native(), execution)?;
        let (next, _next_owner) = target.convex_qp_problem(self.runtime.native(), execution)?;
        // The library QP objective excludes authored constants. Use the native evaluator
        // in that convention directly, avoiding cancellation from subtracting a constant.
        previous.objective_constant = 0.0;
        if previous
            .contract
            .variables
            .iter()
            .map(|v| v.id)
            .ne(report.variables.iter().copied())
            || previous.contract.variables.iter().map(|v| v.id).ne(next
                .contract
                .variables
                .iter()
                .map(|v| v.id))
            || previous.contract.rows != next.contract.rows
            || request.source.x.len() != candidate.primal.len()
            || request
                .source
                .x
                .iter()
                .zip(&candidate.primal)
                .any(|(a, b)| a.to_bits() != b.to_bits())
            || !request.source.obj.is_finite()
            || request.source.obj
                != previous.sense.sign() * previous.objective_at(&candidate.primal)
        {
            return Err(contract(
                "QP prediction source must be the actual original candidate in the same named original coordinates",
            ));
        }
        let triplets = request
            .previous
            .h
            .values()
            .len()
            .checked_add(request.previous.a.values().len())
            .and_then(|n| n.checked_add(request.target.h.values().len()))
            .and_then(|n| n.checked_add(request.target.a.values().len()))
            .ok_or_else(|| contract("QP projection comparison extent"))?;
        let comparison_bytes = triplets
            .checked_mul(256)
            .and_then(|n| {
                request
                    .previous
                    .n
                    .checked_add(request.previous.m)
                    .and_then(|extent| extent.checked_mul(64))
                    .and_then(|extent| n.checked_add(extent))
            })
            .ok_or_else(|| contract("QP projection comparison bytes"))?;
        let _comparison_owner = self
            .runtime
            .native()
            .reserve("modeling:qp-coefficient-comparison", comparison_bytes)?;
        fn sparse_value(
            matrix: &faer::sparse::SparseColMat<usize, f64>,
            row: usize,
            col: usize,
        ) -> f64 {
            let pattern = matrix.symbolic();
            let start = pattern.col_ptr()[col];
            let end = pattern.col_ptr()[col + 1];
            pattern.row_idx()[start..end]
                .binary_search(&row)
                .map_or(0.0, |index| matrix.val()[start + index])
        }
        fn same_sparse(
            actual: &faer::sparse::SparseColMat<usize, f64>,
            expected: &faer::sparse::SparseColMat<usize, f64>,
            sign: f64,
        ) -> bool {
            let all = |matrix: &faer::sparse::SparseColMat<usize, f64>,
                       other: &faer::sparse::SparseColMat<usize, f64>,
                       factor: f64| {
                (0..matrix.ncols()).all(|col| {
                    (matrix.symbolic().col_ptr()[col]..matrix.symbolic().col_ptr()[col + 1]).all(
                        |index| {
                            matrix.val()[index]
                                == factor * sparse_value(other, matrix.row_idx()[index], col)
                        },
                    )
                })
            };
            all(actual, expected, sign) && all(expected, actual, sign)
        }
        fn matches_projection(
            submitted: &pse_backend_native::kkt::path::qp::QpProblem<'_>,
            original: &pse_backend_native::CoefficientProblem,
        ) -> bool {
            use pse_backend_native::kkt::path::qp::{lower_bound_present, upper_bound_present};
            if submitted.validate().is_err()
                || submitted.n != original.contract.variables.len()
                || submitted.m != original.contract.rows.len()
            {
                return false;
            }
            let lower = |actual: f64, expected: f64| {
                if expected.is_finite() {
                    lower_bound_present(actual) && actual == expected
                } else {
                    !actual.is_nan() && !lower_bound_present(actual)
                }
            };
            let upper = |actual: f64, expected: f64| {
                if expected.is_finite() {
                    upper_bound_present(actual) && actual == expected
                } else {
                    !actual.is_nan() && !upper_bound_present(actual)
                }
            };
            if submitted
                .g
                .iter()
                .zip(&original.objective)
                .any(|(actual, expected)| *actual != original.sense.sign() * expected)
                || original.bounds.iter().enumerate().any(|(index, (l, u))| {
                    !lower(submitted.bl[index], *l) || !upper(submitted.bu[index], *u)
                })
                || original
                    .contract
                    .variables
                    .iter()
                    .enumerate()
                    .any(|(index, v)| {
                        !lower(submitted.xl[index], v.lower) || !upper(submitted.xu[index], v.upper)
                    })
            {
                return false;
            }
            // Faer canonicalizes the caller's triplet storage, including duplicates.
            // Mathematical expression extraction and row-bound shifting remain compiler/native owned.
            let index = |value: i32| {
                value
                    .checked_sub(1)
                    .and_then(|value| usize::try_from(value).ok())
            };
            let mut h = Vec::with_capacity(submitted.h.values().len().saturating_mul(2));
            for ((row, col), value) in submitted
                .h
                .irows()
                .iter()
                .zip(submitted.h.jcols())
                .zip(submitted.h.values())
            {
                let Some((row, col)) = index(*row).zip(index(*col)) else {
                    return false;
                };
                if row >= submitted.n || col >= submitted.n || !value.is_finite() {
                    return false;
                }
                h.push(faer::sparse::Triplet::new(row, col, *value));
                if row != col {
                    h.push(faer::sparse::Triplet::new(col, row, *value));
                }
            }
            let Ok(h) =
                faer::sparse::SparseColMat::try_new_from_triplets(submitted.n, submitted.n, &h)
            else {
                return false;
            };
            if let Some(expected) = &original.hessian {
                if !same_sparse(&h, expected, original.sense.sign()) {
                    return false;
                }
            } else if h.val().iter().any(|value| *value != 0.0) {
                return false;
            }
            let mut a = Vec::with_capacity(submitted.a.values().len());
            for ((row, col), value) in submitted
                .a
                .irows()
                .iter()
                .zip(submitted.a.jcols())
                .zip(submitted.a.values())
            {
                let Some((row, col)) = index(*row).zip(index(*col)) else {
                    return false;
                };
                if row >= submitted.m || col >= submitted.n || !value.is_finite() {
                    return false;
                }
                a.push(faer::sparse::Triplet::new(row, col, *value));
            }
            let Ok(a) =
                faer::sparse::SparseColMat::try_new_from_triplets(submitted.m, submitted.n, &a)
            else {
                return false;
            };
            same_sparse(&a, &original.constraints, 1.0)
        }
        execution
            .check()
            .map_err(crate::math::MathRuntimeError::from)?;
        if !matches_projection(request.previous, &previous)
            || !matches_projection(request.target, &next)
        {
            return Err(contract(
                "QP path matrices, objective coefficients or bounds differ from the compiler's actual source or target projection",
            ));
        }
        execution
            .check()
            .map_err(crate::math::MathRuntimeError::from)?;
        let source = self
            .prepared
            .solve
            .semantic_point_key(&candidate.primal)
            .map_err(crate::math::MathRuntimeError::from)?;
        let selected = crate::math::prediction::select(
            crate::math::prediction::SelectionRequest {
                mechanism: crate::math::prediction::ProposalMechanism::Qp {
                    request,
                    coordinates: &report.variables,
                },
                permission: &self.completion.decision,
                source,
                target: target
                    .original_identity()
                    .map_err(crate::math::MathRuntimeError::from)?,
                branch,
            },
            execution,
        )
        .map_err(|error| crate::math::MathRuntimeError::from(error.into_problem()))?;
        let crate::math::prediction::SelectedProposal::Qp { proposal, outcome } = selected else {
            return Err(contract("QP selector returned another producer"));
        };
        let owner = self.runtime.native().reserve(
            "modeling:qp-proposal",
            proposal
                .retained_bytes()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        Ok((proposal.with_owner(owner), *outcome))
    }
    /// A fresh sparse Root predictor is exposed only under composed original permission.
    /// Its own numerical action remains estimated start evidence.
    /// # Errors
    /// Original permission, native candidate or the requested fresh factor is unavailable.
    pub fn root_predictor(
        &self,
    ) -> Result<
        pse_backend_native::square_response::SparsePredictor,
        pse_backend_native::square_response::Withheld,
    > {
        use pse_backend_native::square_response::Withheld;
        if !self.completion.decision.permits_use() {
            return Err(Withheld::Infeasible(
                "original completion does not permit prediction reuse".into(),
            ));
        }
        let Outcome::Native(report) = &self.outcome else {
            return Err(Withheld::NoCandidate);
        };
        report.evidence.root_predictor.clone().unwrap_or_else(|| {
            Err(Withheld::Neighborhood(
                "a fresh sparse Root predictor was not requested".into(),
            ))
        })
    }
    /// Produce a start for an admitted related target, retaining actual source/target
    /// identities. The target must subsequently screen it and perform original correction.
    /// # Errors
    /// Changed fixed dependencies, unavailable original permission or numerical action.
    pub fn root_prediction(
        &self,
        target: &crate::math::solves::PreparedSolve,
        branch: pse_model::strategy::BranchPolicy,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<
        (
            crate::math::prediction::Proposal,
            pse_backend_native::square_response::ActionEvidence,
        ),
        WorkflowError,
    > {
        let predictor = self.root_predictor().map_err(|error| match error {
            pse_backend_native::square_response::Withheld::Cause(cause) => {
                crate::math::MathRuntimeError::Solve(pse_backend_native::ProblemError::Math(
                    pse_math::MathError::Typed {
                        retained: cause.retained_bytes(),
                        cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause),
                    },
                ))
            }
            error => crate::math::MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Unsupported(error.to_string()),
            ),
        })?;
        let parameters = target
            .related_target_parameters(&self.prepared.solve, predictor.parameters())
            .map_err(crate::math::MathRuntimeError::from)?;
        let selected = crate::math::prediction::select(
            crate::math::prediction::SelectionRequest {
                mechanism: crate::math::prediction::ProposalMechanism::Root {
                    predictor: &predictor,
                    parameters: &parameters,
                },
                permission: &self.completion.decision,
                source: predictor.factor().key(),
                target: target
                    .original_identity()
                    .map_err(crate::math::MathRuntimeError::from)?,
                branch,
            },
            execution,
        )
        .map_err(|error| crate::math::MathRuntimeError::from(error.into_problem()))?;
        let (proposal, work) = match selected {
            crate::math::prediction::SelectedProposal::Root { proposal, work } => (*proposal, work),
            _ => {
                return Err(contract("root selector returned a different producer"));
            }
        };
        let owner = self.runtime.native().reserve(
            "modeling:root-proposal",
            proposal
                .retained_bytes()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        Ok((proposal.with_owner(owner), work))
    }
    /// Structured reason this original-model result was not accepted.
    pub fn diagnostic(&self) -> Option<pse_model::diagnostic::BoundaryDiagnostic> {
        use pse_model::diagnostic::{BoundaryClass as C, BoundaryDiagnostic as D, Observation};
        if self.accepted {
            return None;
        }
        if let Outcome::Native(native) = &self.outcome
            && let Some(cause) = native.callback_failure()
        {
            let mut diagnostic = super::super::diagnostics::observed(
                cause,
                pse_diagnostics::DiagnosticStage::Native,
            );
            diagnostic
                .observations
                .insert("phase".into(), Observation::Text("callback".into()));
            return Some(diagnostic);
        }
        if let Outcome::Native(native) = &self.outcome
            && let Some(cause) = native.validation_failure()
        {
            let mut diagnostic = super::super::diagnostics::observed(
                cause,
                pse_diagnostics::DiagnosticStage::Native,
            );
            diagnostic
                .observations
                .insert("phase".into(), Observation::Text("validation".into()));
            return Some(diagnostic);
        }
        if let Some(error) = &self.validation_error {
            return Some(error.clone());
        }
        if let Outcome::Rejected(error) = &self.outcome {
            return Some(
                WorkflowError::Math(crate::math::MathRuntimeError::Shared(error.clone()))
                    .boundary_diagnostic(),
            );
        }
        // A least-infeasible stop names the original rows it leaves violated (ADR-0109
        // item 3) beside the unsatisfied knowledge checks.
        let violated = match &self.outcome {
            Outcome::Native(native) => native
                .least_infeasible
                .as_ref()
                .map(|label| label.violated.iter().map(|v| v.id).collect::<Vec<_>>())
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        // A global infeasibility conclusion that a known point contradicts is its own
        // outcome, never an infeasibility (Plan 23 H10, PS-10).
        let contradiction = match &self.outcome {
            Outcome::Native(native) => native.evidence.contradiction.as_deref(),
            _ => None,
        };
        let mut result = D::new(
            C::TrialRejected,
            pse_diagnostics::DiagnosticStage::ModelingQualification,
            self.checks
                .iter()
                .filter(|c| !c.satisfied)
                .map(|c| c.source_id.as_id())
                .chain(violated),
            if contradiction.is_some() {
                pse_diagnostics::DiagnosticRule::ModelingQualificationInfeasibilityContradicted
            } else {
                pse_diagnostics::DiagnosticRule::ModelingQualificationRejected
            },
        );
        if let Some(witness) = contradiction {
            // The finding names the members the contradicting point assigns, so a fixture
            // expecting it names them as its lineage (Plan 23 H5).
            result
                .sources
                .extend(witness.quality.bounds.iter().map(|v| v.id));
            result.sources.sort_unstable();
            result.sources.dedup();
            result.observations.insert(
                "witness".into(),
                Observation::Text(witness.witness.as_str().into()),
            );
            if let Some(ordinal) = witness.ordinal {
                result.observations.insert(
                    "witness_ordinal".into(),
                    Observation::Integer(i64::try_from(ordinal).unwrap_or(i64::MAX)),
                );
            }
            // The witness's worst residual relative to its tolerance, at most one.
            result.observations.insert(
                "witness_violation".into(),
                Observation::number(witness.quality.normalized_max),
            );
        }
        if let Outcome::Native(native) = &self.outcome {
            result.class = super::super::numerics::native_diagnostic_class(
                native.termination.category,
                contradiction.is_some(),
            );
            // Registry names, never Rust `Debug` output (F30).
            result.observations.insert(
                "termination".into(),
                Observation::Text(native.termination.category.as_str().into()),
            );
            result.observations.insert(
                "qualification".into(),
                Observation::Text(native.qualification.as_str().into()),
            );
            result.observations.insert(
                "candidate_use".into(),
                Observation::Text(self.completion.decision.usability.as_str().into()),
            );
            result.observations.insert(
                "candidate_reason".into(),
                Observation::Text(self.completion.decision.reason()),
            );
        }
        Some(result)
    }
}
pub(in crate::workflow) fn result_bytes(
    prepared: &ModelingSolvePreparation,
) -> Result<usize, WorkflowError> {
    let report_strings = prepared
        .model
        .model
        .compiled()
        .model
        .annotations
        .iter()
        .try_fold(0usize, |n, a| {
            let label = if let AnnotationValue::Report(label) = &a.value {
                label.len()
            } else {
                0
            };
            n.checked_add(label.checked_add(a.lineage.path.len())?.checked_mul(2)?)
        })
        .ok_or_else(|| contract("model report string extent"))?;
    let bytes = prepared
        .model
        .values
        .scalars
        .len()
        .checked_mul(128)
        .and_then(|n| n.checked_add(report_strings))
        .and_then(|n| n.checked_add(prepared.model.model.compiled().model.annotations.len() * 256))
        .and_then(|n| n.checked_add(prepared.model.model.compiled().admitted.outputs.len() * 128))
        .ok_or_else(|| contract("modeling result extent"))?;
    use pse_model::HeapUsage;
    let accuracy = prepared
        .solve
        .numerics()
        .policy
        .goals
        .iter()
        .try_fold(0usize, |n, goal| {
            n.checked_add(size_of::<pse_math::engineering_accuracy::GoalResult>())?
            .checked_add(size_of::<pse_model::engineering_accuracy::OutputEvidence>())?
            .checked_add(goal.heap_bytes())?
            .checked_add(prepared.model.case.compiled().plan.structure().rows().len()
                .checked_mul(size_of::<pse_math::engineering_accuracy::ResidualRowDemand>())?)?
            // Producer limitations are bounded literals; reserve the longest class
            // description and its copied completion projection once per retained goal.
            .checked_add(1024)
        })
        .ok_or_else(|| contract("modeling accuracy result extent"))?;
    bytes
        .checked_add(accuracy)
        .ok_or_else(|| contract("modeling result extent"))
}
impl ModelingResult {
    #[expect(
        clippy::too_many_arguments,
        reason = "result assembly retains separate preparation, attempt identity, outcome, original assessment, completion permission, native ownership and strategy observations"
    )]
    pub(in crate::workflow) fn from_assessment(
        prepared: ModelingSolvePreparation,
        run_id: RunId,
        attempt: usize,
        mut outcome: Outcome,
        point: assessment::AssessedPoint,
        completion: crate::workflow::numerics::Completed,
        native_owner: Arc<pse_columnar::AllocationLease>,
        strategy: Option<crate::math::solves::StrategyTrace>,
    ) -> Self {
        stamp_start(&mut outcome, run_id, attempt);
        Self(Arc::new(ModelingResultData {
            strategy,
            runtime: prepared.source.runtime.clone(),
            prepared,
            run_id,
            original_attempt: attempt,
            outcome,
            values: point.values,
            checks: point.checks,
            reports: point.reports,
            accepted: completion.permits_use(),
            completion,
            validation_error: point.error,
            _owner: point.owner,
            _native_owner: native_owner,
        }))
    }
}
impl ModelingPackage {
    /// Execute one prepared case, join native destruction, then check knowledge obligations.
    pub async fn solve_case(
        &self,
        prepared: ModelingSolvePreparation,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        let mut prepared = prepared;
        prepared.compiler = compiler;
        let handle = prepared.start()?;
        let wait = handle.wait();
        tokio::pin!(wait);
        let joined = tokio::select! {
            result = &mut wait => result?,
            () = cancel.cancelled() => {handle.cancel();wait.await?}
        };
        match &joined.report {
            Ok(super::super::RunReport::Modeling(results)) => results
                .first()
                .cloned()
                .ok_or_else(|| contract("authored solve completed without a result")),
            Err(error) => Err(WorkflowError::Shared(error.clone())),
            _ => Err(contract("authored solve report mismatch")),
        }
    }
}
/// A step's certified dual bound on its authored objective (ADR-0119 Outcome 5): a bound
/// the backend established over the declared box, in the objective's orientation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::workflow) struct CertifiedBound {
    /// The dual bound, in the objective's canonical units.
    pub value: f64,
    /// The authored orientation it bounds: a lower bound when minimizing.
    pub sense: pse_math::binding::ObjectiveSense,
}
impl CertifiedBound {
    /// The certified bound of a step, present only when its assurance is `global_bound` or
    /// `exact_certificate` and the native dual bound is finite. A check never starts a solve
    /// to obtain one.
    pub(in crate::workflow) fn of(outcome: &Outcome) -> Option<Self> {
        use pse_backend_native::solve::Assurance;
        let Outcome::Native(native) = outcome else {
            return None;
        };
        if !matches!(
            native.termination.assurance,
            Assurance::GlobalBound | Assurance::ExactCertificate
        ) {
            return None;
        }
        let global = native.evidence.global.as_ref()?;
        Some(Self {
            value: global.dual_bound.filter(|v| v.is_finite())?,
            sense: global.sense,
        })
    }
    /// Whether this bound can establish `check`: it bounds the objective on the side the
    /// check constrains.
    fn establishes(self, check: ObjectiveBound) -> bool {
        use pse_math::binding::ObjectiveSense as Sense;
        matches!(
            (check, self.sense),
            (ObjectiveBound::Lower { .. }, Sense::Minimize)
                | (ObjectiveBound::Upper { .. }, Sense::Maximize)
        )
    }
}
/// One `annotation check` at a step: at the point, or, for an objective-bound check with a
/// certified bound, against that bound. Returns the indicator value and its basis.
fn check_value(
    point: f64,
    objective_bound: Option<(ObjectiveBound, f64)>,
    certified: Option<CertifiedBound>,
) -> (f64, Basis) {
    match (objective_bound, certified) {
        (Some((check, compared)), Some(bound)) if bound.establishes(check) => (
            if check.holds(bound.value, compared) {
                1.
            } else {
                0.
            },
            Basis::GlobalBound,
        ),
        _ => (point, Basis::Point),
    }
}
/// Shared demand and interpretation for steady candidates and trajectory samples.
pub(in crate::workflow) type AssessmentScope = BTreeSet<(SemanticId, DeclarationId)>;
/// Each obligation is indivisible even when its value, bounds or tolerance depend
/// on different inputs. Consumers may postpone a whole obligation, never one term.
pub(in crate::workflow) fn assessment_units(
    product: &pse_compiler::workspace::PreparedModeling,
) -> BTreeMap<(SemanticId, DeclarationId), BTreeSet<SemanticId>> {
    let mut units = BTreeMap::<_, BTreeSet<_>>::new();
    for output in &product.admitted.outputs {
        let key = match output {
            ModelingOutput::Test { id, .. } => {
                Some((*id, product.model.expectations[id].lineage.declaration))
            }
            ModelingOutput::Hint {
                target,
                declaration,
                kind:
                    ModelingHint::Check
                    | ModelingHint::ObjectiveBound(_)
                    | ModelingHint::ValidLower
                    | ModelingHint::ValidUpper,
            } => Some((*target, *declaration)),
            ModelingOutput::OriginalEquation(id) => {
                Some((*id, product.model.elastic[id].original.lineage.declaration))
            }
            ModelingOutput::Contribution { accumulator, .. } => Some((
                *accumulator,
                product.model.closures[accumulator].lineage.declaration,
            )),
            _ => None,
        };
        if let Some(key) = key {
            units.entry(key).or_default().insert(output.row_id());
        }
    }
    for a in &product.model.annotations {
        if matches!(
            a.value,
            AnnotationValue::Report(_) | AnnotationValue::Valid { .. }
        ) {
            units
                .entry((a.target, a.lineage.declaration))
                .or_default()
                .insert(ModelingOutput::Member(a.target).row_id());
        }
    }
    // The original numerical demand carries scientific evidence even when it has no
    // separate expectation or range annotation. Observe its active execution paths.
    if product
        .model
        .functions
        .values()
        .any(|f| !f.applicability_uses.is_empty())
    {
        for output in &product.admitted.outputs {
            let key = match output {
                ModelingOutput::Equation { id, .. } => product
                    .model
                    .equations
                    .iter()
                    .find(|row| row.id == *id)
                    .map(|row| (*id, row.lineage.declaration)),
                ModelingOutput::Member(id) => product
                    .model
                    .symbols
                    .get(id)
                    .map(|symbol| (*id, symbol.lineage.declaration)),
                _ => None,
            };
            if let Some(key) = key {
                units.entry(key).or_default().insert(output.row_id());
            }
        }
    }
    units
}
pub(in crate::workflow) fn observation_rows(
    product: &pse_compiler::workspace::PreparedModeling,
    _values: &CaseValues,
) -> Result<BTreeSet<SemanticId>, WorkflowError> {
    Ok(assessment_units(product).into_values().flatten().collect())
}
/// Assess the observed point. An objective-bound check reads `certified`, the step's
/// certified dual bound, when the step has one (ADR-0119 Outcome 5).
#[expect(
    clippy::too_many_arguments,
    reason = "assessment reads the run, product, values and observations under the numerical policy, registry, report selection, scope and the step's certified bound"
)]
pub(in crate::workflow) fn assess_observations(
    run_id: RunId,
    product: &pse_compiler::workspace::PreparedModeling,
    values: &CaseValues,
    observed: &BTreeMap<SemanticId, f64>,
    applicability: &[pse_model::applicability::Observation],
    numerics: &pse_model::numerics::ResolvedNumericalPolicy,
    quantities: &pse_quantity::QuantityRegistry,
    include_reports: bool,
    scope: Option<&AssessmentScope>,
    certified: Option<CertifiedBound>,
) -> Result<(Vec<ModelingCheck>, Vec<ModelingReport>), WorkflowError> {
    let selected = |target, source| scope.is_none_or(|ids| ids.contains(&(target, source)));
    let expected = product
        .model
        .expectations
        .values()
        .filter(|t| selected(t.id, t.lineage.declaration))
        .map(|t| t.id)
        .collect();
    let mut checks = product
        .assess_expectations_for(observed, &expected)
        .map_err(|e| contract(e.to_string()))?
        .into_iter()
        .map(|t| ModelingCheck {
            step: 0,
            run_id,
            sample_index: 0,
            time: None,
            target_id: t.id,
            source_id: t.declaration,
            kind: CheckKind::Expectation,
            value: (t.actual - t.expected).abs(),
            tolerance: Some(t.tolerance),
            satisfied: t.passed,
            within_validity: None,
            extrapolation_allowed: None,
            basis: Basis::Point,
            layer: None,
            claim_id: None,
            claim_owner: None,
            claim_owner_lineage: Vec::new(),
            coverage_id: None,
            evidence_id: None,
            form_id: None,
            call_id: None,
            selected_records: Vec::new(),
            dependencies: Vec::new(),
            input_values: Vec::new(),
            applicability_outcome: None,
            applicability_basis: None,
            permission_ids: Vec::new(),
            unknown_allowed: None,
            observation_instance: None,
            applicability_required: None,
            applicability_reason: None,
            applicability_permissions: Vec::new(),
        })
        .collect::<Vec<_>>();
    // The compiler's typed classification of objective-bound checks, with the compared
    // side observed at this point (it depends on no decision).
    let objective_bounds = product
        .admitted
        .outputs
        .iter()
        .filter_map(|output| match output {
            ModelingOutput::Hint {
                target,
                declaration,
                kind: ModelingHint::ObjectiveBound(check),
            } if selected(*target, *declaration) => Some(
                observed
                    .get(&output.row_id())
                    .map(|compared| ((*target, *declaration), (*check, *compared)))
                    .ok_or_else(|| contract("objective-bound check side absent")),
            ),
            _ => None,
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut magnitudes = BTreeMap::new();
    for output in &product.admitted.outputs {
        match output {
            ModelingOutput::Hint {
                target,
                declaration,
                kind: ModelingHint::Check,
            } if selected(*target, *declaration) => {
                let (value, basis) = check_value(
                    observed[&output.row_id()],
                    objective_bounds.get(&(*target, *declaration)).copied(),
                    certified,
                );
                checks.push(ModelingCheck {
                    step: 0,
                    run_id,
                    sample_index: 0,
                    time: None,
                    target_id: *target,
                    source_id: *declaration,
                    kind: CheckKind::Check,
                    value,
                    tolerance: None,
                    satisfied: value == 1.,
                    within_validity: None,
                    extrapolation_allowed: None,
                    basis,
                    layer: None,
                    claim_id: None,
                    claim_owner: None,
                    claim_owner_lineage: Vec::new(),
                    coverage_id: None,
                    evidence_id: None,
                    form_id: None,
                    call_id: None,
                    selected_records: Vec::new(),
                    dependencies: Vec::new(),
                    input_values: Vec::new(),
                    applicability_outcome: None,
                    applicability_basis: None,
                    permission_ids: Vec::new(),
                    unknown_allowed: None,
                    observation_instance: None,
                    applicability_required: None,
                    applicability_reason: None,
                    applicability_permissions: Vec::new(),
                });
            }
            ModelingOutput::Contribution {
                accumulator,
                contribution,
            } if selected(
                *accumulator,
                product.model.closures[accumulator].lineage.declaration,
            ) =>
            {
                magnitudes.insert(*contribution, observed[&output.row_id()]);
            }
            ModelingOutput::OriginalEquation(id)
                if selected(*id, product.model.elastic[id].original.lineage.declaration) =>
            {
                let original = &product.model.elastic[id].original;
                let pse_authoring::dsl::EquationKind::Relation { sense, .. } =
                    original.equation.kind
                else {
                    return Err(contract("original elastic equation shape"));
                };
                let value = observed[&output.row_id()];
                let residual = match sense {
                    pse_authoring::dsl::EquationSense::Eq => value.abs(),
                    pse_authoring::dsl::EquationSense::Le => value.max(0.),
                    pse_authoring::dsl::EquationSense::Ge => (-value).max(0.),
                };
                let tolerance = numerics
                    .targets
                    .iter()
                    .find(|t| {
                        t.id == *id && t.kind == pse_model::generated::enums::NumericalTarget::Row
                    })
                    .ok_or_else(|| contract("original row budget absent"))?
                    .budget;
                checks.push(ModelingCheck {
                    step: 0,
                    run_id,
                    sample_index: 0,
                    time: None,
                    target_id: *id,
                    source_id: original.lineage.declaration,
                    kind: CheckKind::OriginalEquation,
                    value: residual,
                    tolerance: Some(tolerance),
                    satisfied: residual <= tolerance,
                    within_validity: None,
                    extrapolation_allowed: None,
                    basis: Basis::Point,
                    layer: None,
                    claim_id: None,
                    claim_owner: None,
                    claim_owner_lineage: Vec::new(),
                    coverage_id: None,
                    evidence_id: None,
                    form_id: None,
                    call_id: None,
                    selected_records: Vec::new(),
                    dependencies: Vec::new(),
                    input_values: Vec::new(),
                    applicability_outcome: None,
                    applicability_basis: None,
                    permission_ids: Vec::new(),
                    unknown_allowed: None,
                    observation_instance: None,
                    applicability_required: None,
                    applicability_reason: None,
                    applicability_permissions: Vec::new(),
                });
            }
            _ => {}
        }
    }
    let closures = product
        .model
        .closures
        .values()
        .filter(|c| selected(c.id, c.lineage.declaration))
        .map(|c| c.id)
        .collect();
    for closure in product
        .model
        .assess_closures(&magnitudes, &closures, &cases::closure_budgets(numerics))
        .map_err(|e| contract(e.to_string()))?
    {
        if let Some(satisfied) = closure.satisfied {
            checks.push(ModelingCheck {
                step: 0,
                run_id,
                sample_index: 0,
                time: None,
                target_id: closure.accumulator,
                source_id: product.model.closures[&closure.accumulator]
                    .lineage
                    .declaration,
                kind: CheckKind::Closure,
                value: closure.net,
                tolerance: Some(closure.tolerance),
                satisfied,
                within_validity: None,
                extrapolation_allowed: None,
                basis: Basis::Point,
                layer: None,
                claim_id: None,
                claim_owner: None,
                claim_owner_lineage: Vec::new(),
                coverage_id: None,
                evidence_id: None,
                form_id: None,
                call_id: None,
                selected_records: Vec::new(),
                dependencies: Vec::new(),
                input_values: Vec::new(),
                applicability_outcome: None,
                applicability_basis: None,
                permission_ids: Vec::new(),
                unknown_allowed: None,
                observation_instance: None,
                applicability_required: None,
                applicability_reason: None,
                applicability_permissions: Vec::new(),
            });
        }
    }
    let value = |target: SemanticId| -> Result<f64, WorkflowError> {
        values
            .scalars
            .get(&target)
            .copied()
            .or_else(|| {
                product
                    .admitted
                    .outputs
                    .iter()
                    .find(|o| matches!(o,ModelingOutput::Member(id) if *id==target))
                    .and_then(|o| observed.get(&o.row_id()).copied())
            })
            .ok_or_else(|| contract("model check/report target absent"))
    };
    checks.extend(applicability_checks(run_id, applicability));
    let mut reports = Vec::new();
    let mut reported = BTreeSet::new();
    for a in &product.model.annotations {
        if !selected(a.target, a.lineage.declaration) {
            continue;
        }
        match &a.value {
            AnnotationValue::Report(label) if include_reports => {
                if !reported.insert((a.target, a.lineage.declaration)) {
                    return Err(contract("duplicate report source/target"));
                }
                let symbol = product
                    .model
                    .symbols
                    .get(&a.target)
                    .ok_or_else(|| contract("reported target is not a scalar member"))?;
                let transfer_context = report_transfer_context(&symbol.ty)?;
                let scheme = symbol
                    .ty
                    .quantity_scheme()
                    .ok_or_else(|| contract("reported target has no physical contract"))?;
                let quantity = scheme
                    .resolve(quantities, &BTreeMap::new())
                    .map_err(|e| contract(e.to_string()))?;
                let unit = quantities
                    .quantity_type(quantity)
                    .map_err(|e| contract(e.to_string()))?
                    .canonical_unit;
                reports.push(ModelingReport {
                    step: 0,
                    run_id,
                    target_id: a.target,
                    source_id: a.lineage.declaration,
                    label: label.clone(),
                    path: symbol.lineage.path.clone(),
                    quantity_id: quantity.as_id(),
                    unit_id: unit.as_id(),
                    transfer_context,
                    value: value(a.target)?,
                });
            }
            // Every validity check names its layer (ADR-0123 Outcome 4): an annotated
            // closure range, or a data envelope whose consumer selected extrapolation, with
            // the envelope's relation or kind as its source.
            AnnotationValue::Valid { layer, .. } => {
                let endpoint = |kind| -> Result<f64, WorkflowError> {
                    product.admitted.outputs.iter().find(|o|matches!(o,ModelingOutput::Hint{target,declaration,kind:k} if *target==a.target && *declaration==a.lineage.declaration && *k==kind)).and_then(|o|observed.get(&o.row_id()).copied()).ok_or_else(||contract("validity endpoint absent"))
                };
                let lo = endpoint(ModelingHint::ValidLower)?;
                let hi = endpoint(ModelingHint::ValidUpper)?;
                let v = value(a.target)?;
                if lo > hi {
                    return Err(contract("reversed validity interval"));
                }
                let inside = v >= lo && v <= hi;
                checks.push(ModelingCheck {
                    step: 0,
                    run_id,
                    sample_index: 0,
                    time: None,
                    target_id: a.target,
                    source_id: a.lineage.declaration,
                    kind: CheckKind::Validity,
                    value: v,
                    tolerance: None,
                    satisfied: inside,
                    within_validity: Some(inside),
                    extrapolation_allowed: None,
                    basis: Basis::Point,
                    layer: Some(*layer),
                    claim_id: None,
                    claim_owner: None,
                    claim_owner_lineage: Vec::new(),
                    coverage_id: None,
                    evidence_id: None,
                    form_id: None,
                    call_id: None,
                    selected_records: Vec::new(),
                    dependencies: Vec::new(),
                    input_values: Vec::new(),
                    applicability_outcome: None,
                    applicability_basis: None,
                    permission_ids: Vec::new(),
                    unknown_allowed: None,
                    observation_instance: None,
                    applicability_required: None,
                    applicability_reason: None,
                    applicability_permissions: Vec::new(),
                });
            }
            _ => {}
        }
    }
    Ok((checks, reports))
}

impl ModelingResult {
    /// Encode generated contracts with the shared reserve-before-growth columnar owner.
    /// Returned columns may outlive the model, result and source runtime handle.
    pub fn tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        use pse_model::HeapUsage;
        let pool = self.runtime.shared.pool();
        let cancel = pse_columnar::CancellationToken::new();
        let scratch = self.runtime.shared.math().reserve(
            "modeling:result-row-copy",
            self.checks
                .iter()
                .map(HeapUsage::owned_bytes)
                .chain(self.reports.iter().map(HeapUsage::owned_bytes))
                .max()
                .unwrap_or(0),
        )?;
        let validation = self.runtime.validation_context()?;
        let mut columns = pse_relations::columnar::Collection::new(
            &self.runtime.registry,
            &pool,
            &cancel,
            &validation,
        );
        columns.ensure::<ModelingCheck>().map_err(relation)?;
        columns.ensure::<ModelingReport>().map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::solve_strategy_events::Row>()
            .map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::solve_strategy_products::Row>()
            .map_err(relation)?;
        if let Some(strategy) = &self.strategy {
            for row in strategy
                .product_rows(self.run_id, 0)
                .map_err(crate::math::MathRuntimeError::from)?
            {
                columns.push(row).map_err(relation)?;
            }
            for row in strategy
                .rows(self.run_id, 0)
                .map_err(crate::math::MathRuntimeError::from)?
            {
                columns.push(row).map_err(relation)?;
            }
        }
        columns
            .ensure::<pse_model::generated::runtime::modeling_findings::Row>()
            .map_err(relation)?;
        // The failure first, then the informational bound tightenings (ADR-0103 item 4).
        let tightenings = self
            .prepared
            .model
            .tightenings
            .iter()
            .map(pse_modeling::DomainTightening::boundary_diagnostic);
        for (ordinal, finding) in self.diagnostic().into_iter().chain(tightenings).enumerate() {
            columns
                .push(analysis_tables::finding_row(
                    self.run_id,
                    ordinal as i64,
                    &finding,
                ))
                .map_err(relation)?;
        }
        for row in &self.checks {
            columns.push(row.clone()).map_err(relation)?;
        }
        for row in &self.reports {
            columns.push(row.clone()).map_err(relation)?;
        }
        let result = columns.finish().map_err(relation)?;
        drop(scratch);
        Ok(result
            .into_values()
            .map(|batch| (batch.relation_id(), batch))
            .collect())
    }
}

/// Project demanded evidence once into the registry-owned numerical check relation.
pub(super) fn applicability_checks(
    run_id: RunId,
    applicability: &[pse_model::applicability::Observation],
) -> Vec<ModelingCheck> {
    applicability.iter().map(|observation| {
        let claim = &observation.claim;
        let source = DeclarationId::from(claim.id.unwrap_or(claim.form));
        ModelingCheck {
            run_id,step:0,sample_index:0,time:None,target_id:observation_identity(observation),source_id:source,
            kind:CheckKind::Applicability,value:if observation.admitted {1.} else {0.},tolerance:None,
            satisfied:observation.admitted,within_validity:None,extrapolation_allowed:Some(observation.extrapolation_allowed),
            basis:Basis::Point,layer:Some(claim.layer),claim_id:claim.id,claim_owner:Some(claim.owner),claim_owner_lineage:claim.owner_lineage.clone(),coverage_id:claim.coverage,
            evidence_id:claim.evidence,form_id:Some(claim.form),call_id:Some(claim.call),selected_records:claim.records.clone(),dependencies:claim.dependencies.clone(),
            input_values:observation.inputs.iter().map(|input|pse_model::generated::runtime::modeling_checks::RuntimeModelingChecksFieldInputValuesItem {name:input.name.clone(),value:input.value,quantity_type:input.quantity_type}).collect(),
            applicability_outcome:Some(observation.outcome),applicability_basis:claim.basis,
            permission_ids:observation.permissions.iter().map(|p|p.id).collect(),unknown_allowed:Some(observation.unknown_allowed),
            observation_instance:observation.instance,applicability_required:Some(observation.required),applicability_reason:claim.reason.clone(),
            applicability_permissions:observation.permissions.iter().map(|p|pse_model::generated::runtime::modeling_checks::RuntimeModelingChecksFieldApplicabilityPermissionsItem {permission_id:p.id,scope:p.scope,target_kind:p.target_kind,targets:p.targets.clone(),allow_unknown:p.allow_unknown,allow_extrapolation:p.allow_extrapolation}).collect(),
        }
    }).collect()
}

fn observation_identity(observation: &pse_model::applicability::Observation) -> SemanticId {
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ModelingApplicabilityObservationV2);
    h.id(&observation.claim.call)
        .bool(observation.claim.id.is_some())
        .id(&observation.claim.id.unwrap_or(SemanticId::NIL))
        .bool(observation.instance.is_some())
        .id(&observation.instance.unwrap_or(SemanticId::NIL))
        .bool(observation.required)
        .id(&observation.claim.owner)
        .u64(observation.claim.owner_lineage.len() as u64);
    for owner in &observation.claim.owner_lineage {
        h.id(owner);
    }
    h.u64(observation.inputs.len() as u64);
    for input in &observation.inputs {
        h.str(&input.name)
            .id(&input.quantity_type)
            .u64(input.value.to_bits());
    }
    h.finish_id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_compiler::workspace::ModelingCaseBindings;
    use pse_relations::columnar::RelationRow;
    #[tokio::test]
    async fn report_transfer_context_retains_actual_owner_order_and_direction() {
        use pse_modeling::specialize::Value;
        use pse_modeling::{BoundaryRef, PhysicalRefinement, TransferDirection, Type};
        let instance = SemanticId::from_bytes([1; 16]).into();
        let declaration = SemanticId::from_bytes([2; 16]).into();
        let coordinates = vec![Value::Integer(7), Value::Text("hot".into())];
        let quantity = pse_quantity::scheme::Scheme::Concrete(SemanticId::NIL.into());
        let ty = Type::RefinedQuantity {
            quantity: quantity.clone(),
            refinement: PhysicalRefinement::Transfer {
                boundary: BoundaryRef::Bound {
                    instance,
                    declaration,
                    coordinates: coordinates.clone(),
                },
                direction: TransferDirection::OutOf,
            },
        };
        let context = report_transfer_context(&ty).unwrap().unwrap();
        assert_eq!(context.instance, instance);
        assert_eq!(context.boundary, declaration);
        assert_eq!(
            context.coordinates,
            coordinates.iter().map(Value::identity).collect::<Vec<_>>()
        );
        assert_eq!(
            context.direction,
            pse_model::generated::enums::ModelingTransferDirection::OutOf
        );
        let unbound = Type::RefinedQuantity {
            quantity: quantity.clone(),
            refinement: PhysicalRefinement::Transfer {
                boundary: BoundaryRef::Declared(declaration),
                direction: TransferDirection::Into,
            },
        };
        assert!(report_transfer_context(&unbound).is_err());
        assert!(
            report_transfer_context(&Type::Quantity(quantity))
                .unwrap()
                .is_none()
        );
        let report = ModelingReport {
            run_id: SemanticId::NIL.into(),
            step: 0,
            target_id: SemanticId::NIL,
            source_id: declaration,
            label: "heat".into(),
            path: "unit.heat".into(),
            quantity_id: SemanticId::NIL,
            unit_id: SemanticId::NIL,
            transfer_context: Some(context),
            value: -3.,
        };
        let registry = pse_schema::registry().unwrap();
        let validation = pse_relations::validate::ValidationContext::local(registry).unwrap();
        let mut builder = ModelingReport::builder(registry, 1, &validation).unwrap();
        ModelingReport::push(&mut builder, report.clone()).unwrap();
        let rows = ModelingReport::finish(builder).unwrap();
        assert_eq!(ModelingReport::rows(&rows).unwrap(), vec![report]);
    }
    fn step(assurance: pse_backend_native::solve::Assurance, dual_bound: Option<f64>) -> Outcome {
        use pse_backend_native::{self as native, solve::*};
        let id = SemanticId::from_bytes([1; 16]);
        let contract = native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![native::Variable {
                id,
                lower: -2.0,
                upper: 2.0,
            }],
            rows: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let mut report = SolveReport::new(
            Backend::Scip,
            &contract,
            NativeTermination {
                code: 0,
                name: "fixture".into(),
                message: None,
                category: Termination::Success,
                assurance,
            },
            &Execution::new(Arc::default(), &Controls::default()),
        );
        report.evidence.global = Some(GlobalEvidence {
            fidelity: pse_math::factorable::Fidelity::Exact,
            domain: pse_ids::ContentHash::from_bytes([2; 32]),
            sense: pse_math::binding::ObjectiveSense::Minimize,
            feasibility: 1e-9,
            gap_relative: 1e-6,
            gap_absolute: 1e-9,
            dual_bound,
            primal_bound: Some(0.0),
            gap: Some(0.0),
            nodes: 1,
            readback: true,
            dual: BoundSource::ExactExport,
            primal: PrimalSource::Backend,
            infeasible: false,
            exact: assurance == Assurance::ExactCertificate,
        });
        Outcome::Native(Box::new(report))
    }
    #[cfg_attr(
        not(feature = "native-solvers"),
        ignore = "needs the linked native solvers"
    )]
    #[tokio::test]
    async fn objective_bound_check_uses_certified_bound() {
        use pse_backend_native::solve::Assurance;
        let runtime = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        // A tangent-plane-style stability check: the minimized objective stays above -c.
        let source = "package p { def Root { param c: Scalar = 0.5; var x: Scalar; let f: Scalar = x*x; eq g: x <= 2; annotation bounds x(-2, 2); annotation start x(1); annotation objective f(minimize); annotation check f(f > -c); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let cancel = crate::CancelSource::new();
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                pse_kernels::DerivativeOrder::First,
                super::super::super::tests::compiler_profile(),
                crate::math::solves::SolverProfile {
                    intent: pse_backend_native::solve::SolveIntent::Optimize,
                    ..super::super::super::tests::profile()
                },
                crate::math::solves::NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = prepared.model.model.compiled();
        let row = |wanted: fn(&ModelingHint) -> bool| {
            product
                .admitted
                .outputs
                .iter()
                .find(|o| matches!(o, ModelingOutput::Hint { kind, .. } if wanted(kind)))
                .map(ModelingOutput::row_id)
                .unwrap()
        };
        let check = row(|k| *k == ModelingHint::Check);
        let side = row(|k| matches!(k, ModelingHint::ObjectiveBound(_)));
        // The point indicator and the compared side, as the step's observation program
        // would report them.
        let assess = |point: f64, outcome: &Outcome| {
            let observed = BTreeMap::from([(check, point), (side, -0.5)]);
            let (checks, _) = assess_observations(
                RunId::from_bytes([7; 16]),
                product,
                &prepared.model.values,
                &observed,
                &[],
                prepared.solve.numerics(),
                &prepared.source.quantities,
                false,
                None,
                CertifiedBound::of(outcome),
            )
            .unwrap();
            let [check] = checks.as_slice() else {
                panic!("one check expected, got {checks:?}");
            };
            (check.satisfied, check.value, check.basis)
        };
        // A certified dual bound below -c: the global minimum may violate the check, so it
        // fails although the local point satisfies it.
        assert_eq!(
            assess(1.0, &step(Assurance::GlobalBound, Some(-0.75))),
            (false, 0.0, Basis::GlobalBound)
        );
        // A certified bound above -c establishes it over the box, whatever the point says.
        assert_eq!(
            assess(0.0, &step(Assurance::GlobalBound, Some(1e-9))),
            (true, 1.0, Basis::GlobalBound)
        );
        assert_eq!(
            assess(0.0, &step(Assurance::ExactCertificate, Some(0.0))),
            (true, 1.0, Basis::GlobalBound)
        );
        // Without a certified bound the check is evaluated at the point, and says so.
        for outcome in [
            step(Assurance::LocalStationary, Some(1e-9)),
            step(Assurance::NativeOptimal, Some(1e-9)),
            step(Assurance::GlobalBound, None),
            step(Assurance::GlobalBound, Some(f64::NEG_INFINITY)),
        ] {
            assert_eq!(assess(1.0, &outcome), (true, 1.0, Basis::Point));
            assert_eq!(assess(0.0, &outcome), (false, 0.0, Basis::Point));
        }
        // The bound establishes only a check on the side it bounds.
        let lower = ObjectiveBound::Lower { strict: true };
        let upper = ObjectiveBound::Upper { strict: true };
        let minimum = CertifiedBound {
            value: 1.0,
            sense: pse_math::binding::ObjectiveSense::Minimize,
        };
        assert_eq!(
            check_value(0.0, Some((lower, 0.5)), Some(minimum)),
            (1.0, Basis::GlobalBound)
        );
        assert_eq!(
            check_value(0.0, Some((upper, 2.0)), Some(minimum)),
            (0.0, Basis::Point)
        );
        assert_eq!(check_value(1.0, None, Some(minimum)), (1.0, Basis::Point));
    }
}

/// Portable original coordinates and permissions; factors are rebuilt by their library owner.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortablePrediction {
    pub(crate) key: pse_model::strategy::SemanticProductKey,
    pub(crate) coordinates: Vec<SemanticId>,
    pub(crate) primal: Vec<u64>,
    pub(crate) permission: crate::workflow::numerics::CandidateDecision,
    pub(crate) root: Option<PortableRootPoint>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortableRootPoint {
    pub(crate) key: pse_model::strategy::SemanticProductKey,
    pub(crate) values: Vec<u64>,
}
/// The single proposal selection adapter accepts both live and reopened scientific points.
pub(crate) struct PredictionSample {
    pub(crate) prepared: ModelingSolvePreparation,
    pub(crate) runtime: Runtime,
    pub(crate) point: Arc<pse_columnar::Leased<PortablePrediction>>,
    pub(crate) root: Result<
        pse_backend_native::square_response::SparsePredictor,
        pse_backend_native::square_response::Withheld,
    >,
}
impl PredictionSample {
    fn root_predictor(
        &self,
    ) -> Result<
        pse_backend_native::square_response::SparsePredictor,
        pse_backend_native::square_response::Withheld,
    > {
        self.root.clone()
    }
    fn prediction_anchor(
        &self,
        parameter: f64,
    ) -> Result<crate::math::prediction::Anchor, WorkflowError> {
        let point = &self.point;
        let primal = point
            .primal
            .iter()
            .copied()
            .map(f64::from_bits)
            .collect::<Vec<_>>();
        let expected = self
            .prepared
            .solve
            .semantic_point_key(&primal)
            .map_err(crate::math::MathRuntimeError::from)?;
        if expected != point.key
            || self.prepared.model.case.compiled().plan.columns() != point.coordinates.as_slice()
        {
            return Err(contract(
                "retained prediction differs from its original source coordinates",
            ));
        }
        let anchor = crate::math::prediction::Anchor::admitted(
            point.key,
            point.coordinates.clone(),
            primal,
            parameter,
            point.permission.clone(),
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        let owner = self.runtime.native().reserve(
            "modeling:prediction-anchor",
            anchor
                .retained_bytes()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        Ok(anchor.with_owner(owner))
    }
    fn secant_product(
        &self,
        older: &Self,
        target: &ModelingSolvePreparation,
    ) -> Result<(crate::math::prediction::SecantHistory, f64), WorkflowError> {
        let changed = self
            .prepared
            .model
            .case
            .compiled()
            .plan
            .structure()
            .parameters()
            .iter()
            .filter_map(|parameter| {
                let new = self.prepared.model.values.scalars.get(&parameter.id)?;
                let old = older.prepared.model.values.scalars.get(&parameter.id)?;
                (new != old).then_some((parameter.id, *old, *new))
            })
            .collect::<Vec<_>>();
        let [(parameter, old, new)] = changed.as_slice() else {
            return Err(crate::math::MathRuntimeError::from(
                pse_backend_native::ProblemError::Unsupported(
                    "secant requires exactly one changing authored parameter".into(),
                ),
            )
            .into());
        };
        self.prepared
            .solve
            .related_target_parameters(&older.prepared.solve, &[(*parameter, *old)])
            .map_err(crate::math::MathRuntimeError::from)?;
        let values = target
            .solve
            .related_target_parameters(&self.prepared.solve, &[(*parameter, *new)])
            .map_err(crate::math::MathRuntimeError::from)?;
        let history = crate::math::prediction::SecantHistory::new(
            older.prediction_anchor(*old)?,
            self.prediction_anchor(*new)?,
            (new - old).abs(),
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        Ok((history, values[0].1))
    }
    /// Select only actually qualified original products for a related Study target.
    /// The shared selector owns preference; every selected endpoint remains a screened start.
    pub(crate) fn available_prediction(
        &self,
        older: Option<&Self>,
        target: &ModelingSolvePreparation,
        branch: pse_model::strategy::BranchPolicy,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<crate::math::prediction::Proposal, WorkflowError> {
        use crate::math::prediction::{ProposalMechanism, SelectedProposal, SelectionRequest};
        let recoverable = |error: &WorkflowError| {
            matches!(
                error.boundary_diagnostic().class,
                pse_model::diagnostic::BoundaryClass::Unsupported
                    | pse_model::diagnostic::BoundaryClass::Incompatible
                    | pse_model::diagnostic::BoundaryClass::Numerical
            )
        };
        let root = match self.root_predictor() {
            Ok(predictor) => Some(predictor),
            Err(pse_backend_native::square_response::Withheld::Cause(cause)) => {
                let error: WorkflowError = crate::math::MathRuntimeError::from(
                    pse_backend_native::ProblemError::Math(pse_math::MathError::Typed {
                        retained: cause.retained_bytes(),
                        cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause),
                    }),
                )
                .into();
                if !recoverable(&error) {
                    return Err(error);
                }
                None
            }
            Err(_) => None,
        };
        let root_parameters = root
            .as_ref()
            .map(|predictor| {
                target
                    .solve
                    .related_target_parameters(&self.prepared.solve, predictor.parameters())
            })
            .transpose()
            .map_err(crate::math::MathRuntimeError::from)?;
        let secant = match older
            .map(|older| self.secant_product(older, target))
            .transpose()
        {
            Ok(product) => product,
            Err(error) if recoverable(&error) => None,
            Err(error) => return Err(error),
        };
        // Zeroth reuse still validates every changed authored parameter and fixed dependency.
        let parameters = self
            .prepared
            .model
            .case
            .compiled()
            .plan
            .structure()
            .parameters()
            .iter()
            .filter_map(|parameter| {
                self.prepared
                    .model
                    .values
                    .scalars
                    .get(&parameter.id)
                    .map(|value| (parameter.id, *value))
            })
            .collect::<Vec<_>>();
        target
            .solve
            .related_target_parameters(&self.prepared.solve, &parameters)
            .map_err(crate::math::MathRuntimeError::from)?;
        let anchor = self.prediction_anchor(0.)?;
        let identity = target
            .solve
            .original_identity()
            .map_err(crate::math::MathRuntimeError::from)?;
        let mut available = Vec::new();
        let permission = &self.point.permission;
        if let (Some(predictor), Some(parameters)) = (root.as_ref(), root_parameters.as_ref()) {
            available.push(SelectionRequest {
                mechanism: ProposalMechanism::Root {
                    predictor,
                    parameters,
                },
                permission,
                source: predictor.factor().key(),
                target: identity,
                branch,
            });
        }
        if let Some((history, parameter)) = secant.as_ref() {
            available.push(SelectionRequest {
                mechanism: ProposalMechanism::Secant {
                    history,
                    parameter: *parameter,
                    scaled_step_limit: 1.,
                },
                permission,
                source: history.source(),
                target: identity,
                branch,
            });
        }
        available.push(SelectionRequest {
            mechanism: ProposalMechanism::Zeroth { anchor: &anchor },
            permission,
            source: anchor.source(),
            target: identity,
            branch,
        });
        let selected = crate::math::prediction::select_available(available, execution)
            .map_err(|error| crate::math::MathRuntimeError::from(error.into_problem()))?;
        let proposal = match selected {
            SelectedProposal::Root { proposal, .. }
            | SelectedProposal::Secant { proposal }
            | SelectedProposal::Zeroth { proposal } => *proposal,
            _ => {
                return Err(contract(
                    "Study proposal selection returned an unavailable producer",
                ));
            }
        };
        let owner = self.runtime.native().reserve(
            "modeling:selected-proposal",
            proposal
                .retained_bytes()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        Ok(proposal.with_owner(owner))
    }
}
