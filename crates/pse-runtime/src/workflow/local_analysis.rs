// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Publication of qualified square responses and quantities derived from KKT-point analysis (Plan 22 S1; ADR-0118
//! item 10; PS-12): one `local_validity` row per requested quantity, certified or withheld
//! with its reason, and the data rows of the certified ones. The backend's typed reasons
//! map onto the registry `WithheldReason` here, at the publication boundary.
use super::{
    WorkflowError, contract, relation,
    uncertainty::{self, Jacobian, Propagated, Upstream},
};
use pse_backend_native::{
    kkt::{Curvature, KktPoint, Licq, Parametric, Unavailable, Withheld},
    solve::SolveReport,
};
use pse_ids::SemanticId;
use pse_math::binding::CaseStructure;
use pse_quantity::QuantityRegistry;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{
        enums::{DerivedQuantity, NumericalTarget, WithheldReason},
        identities::RunId,
        runtime::{
            local_validity as validity, parametric_sensitivities as sensitivities,
            propagated_covariances as propagated, reduced_hessians as hessians,
        },
        structures::LocalValidity,
    },
};
use std::collections::BTreeMap;

/// The registry reason of a withheld quantity and its typed cause.
pub(super) fn reason(withheld: &Withheld) -> WithheldReason {
    match withheld {
        Withheld::NoCandidate => WithheldReason::NoCandidate,
        Withheld::Multipliers | Withheld::Analysis(Unavailable::Multipliers) => {
            WithheldReason::MultipliersUnrecovered
        }
        Withheld::Complementarity => WithheldReason::ComplementarityFailed,
        Withheld::Unqualified(_) | Withheld::Analysis(Unavailable::Infeasible) => {
            WithheldReason::NotStationary
        }
        Withheld::Analysis(_) => WithheldReason::AnalysisUnavailable,
        Withheld::Licq { .. } => WithheldReason::LicqFailed,
        Withheld::WeaklyActive { .. } => WithheldReason::WeaklyActive,
        Withheld::SecondOrder(_) => WithheldReason::SecondOrderFailed,
        Withheld::Backsolve => WithheldReason::BacksolveFailed,
    }
}

/// The validity record of one quantity: its outcome, whether it is conditional on a
/// discrete assignment, and the verdicts of the point it was read from.
pub(super) fn record<T>(
    outcome: Result<&T, (WithheldReason, String)>,
    point: Option<&KktPoint>,
    conditional: bool,
) -> LocalValidity {
    let (certified, reason, detail) = match outcome {
        Ok(_) => (true, None, None),
        Err((reason, detail)) => (false, Some(reason), Some(detail)),
    };
    LocalValidity {
        certified,
        reason,
        detail,
        conditional,
        licq: point.map(|p| p.licq == Licq::Independent),
        strict_complementarity: point.map(|p| p.weakly_active() == 0),
        second_order: point.map(|p| p.curvature == Curvature::Sufficient),
        weakly_active: point.and_then(|p| i64::try_from(p.weakly_active()).ok()),
        condition_1norm: point.and_then(|p| p.condition_1norm),
        residual: point.and_then(|p| p.residual),
        root_rank: None,
        root_rank_cutoff: None,
        root_rank_relative_cutoff: None,
        root_backward_error: None,
        root_backward_error_limit: None,
        root_neighborhood: None,
    }
}

/// The step's inputs to the local-analysis relations.
pub(super) struct Step<'a> {
    pub run_id: RunId,
    pub step: i64,
    /// The requested parameters and whether the reduced Hessian was requested.
    pub request: &'a crate::math::settings::SensitivityRequest,
    /// The native report, when the step ran one.
    pub report: Option<&'a SolveReport>,
    /// Whether the step observed a candidate at all.
    pub candidate: bool,
    /// The solved structure, for units.
    pub structure: &'a CaseStructure,
    pub quantities: &'a QuantityRegistry,
}

/// Builders of the four local-analysis relations over a run's steps.
pub(super) struct Rows {
    validity: validity::Builder,
    sensitivities: sensitivities::Builder,
    hessians: hessians::Builder,
    propagated: propagated::Builder,
}
impl Rows {
    pub(super) fn new(
        registry: &pse_schema::Registry,
        validation: &pse_relations::validate::ValidationContext,
    ) -> Result<Self, WorkflowError> {
        Ok(Self {
            validity: validity::Builder::with_registry(registry, 0, validation)
                .map_err(relation)?,
            sensitivities: sensitivities::Builder::with_registry(registry, 0, validation)
                .map_err(relation)?,
            hessians: hessians::Builder::with_registry(registry, 0, validation)
                .map_err(relation)?,
            propagated: propagated::Builder::with_registry(registry, 0, validation)
                .map_err(relation)?,
        })
    }
    /// The rows of one step that requested sensitivities.
    pub(super) fn push(&mut self, step: &Step<'_>) -> Result<(), WorkflowError> {
        if let Some(root) = step.report.and_then(|r| r.evidence.root_response.as_ref()) {
            return self.root_response(step, root);
        }
        let evidence = step.report.and_then(|r| r.evidence.sensitivity.as_ref());
        // Quantities read from multipliers conditional on a discrete assignment hold under
        // that commitment, which the step's `solve_runs` row states (ADR-0118 items 4, 9).
        let conditional = step
            .report
            .and_then(|r| r.candidate.as_ref())
            .is_some_and(|c| c.commitment.is_some());
        let absent = || {
            if step.candidate {
                (
                    WithheldReason::NoLocalAnalysis,
                    "the route ran no KKT-point analysis at its candidate".to_owned(),
                )
            } else {
                (
                    WithheldReason::NoCandidate,
                    Withheld::NoCandidate.to_string(),
                )
            }
        };
        let withheld = |w: &Withheld| (reason(w), w.to_string());
        let point = evidence.and_then(|p| p.point.as_ref());
        let sensitivity = evidence.map_or_else(
            || Err(absent()),
            |p| p.sensitivities.as_ref().map_err(withheld),
        );
        self.validity
            .push(validity::Row {
                run_id: step.run_id,
                step: step.step,
                quantity: DerivedQuantity::ParametricSensitivity,
                validity: record(sensitivity.clone(), point, conditional),
            })
            .map_err(relation)?;
        let hessian = step.request.reduced_hessian.then(|| {
            evidence.map_or_else(
                || Err(absent()),
                |p| match &p.reduced_hessian {
                    Some(Ok(h)) => Ok(h),
                    Some(Err(w)) => Err(withheld(w)),
                    None => Err(absent()),
                },
            )
        });
        if let Some(hessian) = &hessian {
            self.validity
                .push(validity::Row {
                    run_id: step.run_id,
                    step: step.step,
                    quantity: DerivedQuantity::ReducedHessian,
                    validity: record(
                        hessian.as_ref().map(|h| *h).map_err(Clone::clone),
                        point,
                        conditional,
                    ),
                })
                .map_err(relation)?;
        }
        // The requested propagation holds while the sensitivities it reads do (S4).
        if let Some(propagation) = &step.request.propagation {
            let jacobian = match (&sensitivity, step.report, evidence) {
                (Ok(s), Some(report), Some(parametric)) => {
                    Ok(jacobian(report, parametric, s, &propagation.outputs)?)
                }
                (Err((reason, detail)), ..) => Err(Upstream {
                    quantity: DerivedQuantity::ParametricSensitivity,
                    reason: *reason,
                    detail: detail.clone(),
                }),
                (Ok(_), ..) => return Err(contract("certified sensitivities without a report")),
            };
            let result = uncertainty::propagate(
                Ok(&propagation.covariance),
                jacobian.as_ref().map_err(Clone::clone),
            )?;
            self.propagate(step, result, point, conditional)?;
        }
        let (Some(report), Some(parametric)) = (step.report, evidence) else {
            return Ok(());
        };
        if let Ok(s) = &parametric.sensitivities {
            self.sensitivities(step, report, parametric, s)?;
        }
        if let Some(Ok(h)) = hessian {
            self.hessian(step, parametric, h)?;
        }
        Ok(())
    }
    fn root_response(
        &mut self,
        step: &Step<'_>,
        root: &Result<
            pse_backend_native::square_response::Response,
            pse_backend_native::square_response::Withheld,
        >,
    ) -> Result<(), WorkflowError> {
        use pse_backend_native::square_response::Withheld as W;
        let outcome = root.as_ref().map_err(|w| {
            (
                match w {
                    W::NoCandidate => WithheldReason::NoCandidate,
                    W::Infeasible(_) => WithheldReason::NotFeasible,
                    W::Structural(_) => WithheldReason::StructuralUnavailable,
                    W::Neighborhood(_) => WithheldReason::NeighborhoodUnavailable,
                    W::Rank { .. } => WithheldReason::RankDeficient,
                    W::Numerical(_) => WithheldReason::BacksolveFailed,
                    W::Memory => WithheldReason::AnalysisUnavailable,
                },
                w.to_string(),
            )
        });
        let mut validity = record(outcome.clone(), None, false);
        if let Ok(response) = &outcome {
            let e = &response.evidence;
            validity.root_rank = i64::try_from(e.rank).ok();
            validity.root_rank_cutoff = Some(e.cutoff);
            validity.root_rank_relative_cutoff = Some(e.relative_cutoff);
            validity.root_backward_error = Some(e.backward_error);
            validity.root_backward_error_limit = Some(e.backward_error_limit);
            validity.root_neighborhood = Some(e.neighborhood.to_owned());
        }
        if let Err(W::Rank { rank, cutoff, .. }) = root {
            validity.root_rank = i64::try_from(*rank).ok();
            validity.root_rank_cutoff = Some(*cutoff);
            validity.root_rank_relative_cutoff =
                Some(pse_backend_native::square_response::DEFAULT_RELATIVE_RANK_CUTOFF);
        }
        self.validity
            .push(validity::Row {
                run_id: step.run_id,
                step: step.step,
                quantity: DerivedQuantity::ParametricSensitivity,
                validity,
            })
            .map_err(relation)?;
        if let Ok(response) = root {
            let units = Units::of(step)?;
            for (j, parameter) in response.parameters.iter().enumerate() {
                for (i, state) in response.states.iter().enumerate() {
                    self.sensitivities
                        .push(sensitivities::Row {
                            run_id: step.run_id,
                            step: step.step,
                            parameter_id: *parameter,
                            target_kind: NumericalTarget::Variable,
                            target_id: *state,
                            parameter_unit_id: units.parameter(*parameter)?,
                            target_unit_id: units.variable(*state)?,
                            primal: Some(response.values[(i, j)]),
                            dual: None,
                        })
                        .map_err(relation)?;
                }
            }
        }
        Ok(())
    }
    fn sensitivities(
        &mut self,
        step: &Step<'_>,
        report: &SolveReport,
        parametric: &Parametric,
        s: &pse_backend_native::kkt::Sensitivities,
    ) -> Result<(), WorkflowError> {
        let units = Units::of(step)?;
        for (k, parameter) in parametric.parameters.iter().enumerate() {
            let parameter_unit_id = units.parameter(*parameter)?;
            let mut push = |target_kind, target_id, target_unit_id, primal, dual| {
                self.sensitivities
                    .push(sensitivities::Row {
                        run_id: step.run_id,
                        step: step.step,
                        parameter_id: *parameter,
                        target_kind,
                        target_id,
                        parameter_unit_id,
                        target_unit_id,
                        primal,
                        dual,
                    })
                    .map_err(relation)
            };
            for (j, id) in report.variables.iter().enumerate() {
                push(
                    NumericalTarget::Variable,
                    *id,
                    units.variable(*id)?,
                    Some(s.primal[k][j]),
                    Some(s.bounds[k][j]),
                )?;
            }
            for (r, id) in report.rows.iter().enumerate() {
                push(
                    NumericalTarget::Row,
                    *id,
                    units.row(*id)?,
                    None,
                    Some(s.rows[k][r]),
                )?;
            }
            push(
                NumericalTarget::Objective,
                SemanticId::NIL,
                units.objective()?,
                Some(s.objective[k]),
                None,
            )?;
        }
        Ok(())
    }
    fn hessian(
        &mut self,
        step: &Step<'_>,
        parametric: &Parametric,
        h: &pse_backend_native::kkt::ReducedHessian,
    ) -> Result<(), WorkflowError> {
        let units = Units::of(step)?;
        self.hessians
            .push(hessians::Row {
                run_id: step.run_id,
                step: step.step,
                parameters: parametric.parameters.clone(),
                parameter_units: parametric
                    .parameters
                    .iter()
                    .map(|p| units.parameter(*p))
                    .collect::<Result<_, _>>()?,
                objective_unit_id: units.objective()?,
                coordinate_scales: h.coordinate_scales.clone(),
                objective_scale: h.objective_scale,
                values: h.values.clone(),
                normalized: h.normalized.clone(),
                eigenvalues: h.eigenvalues.clone(),
                eigenvectors: h.eigenvectors.clone(),
            })
            .map_err(relation)
    }
    /// A propagated covariance's validity row and, when certified, its data row; the
    /// outputs' units are read from `step`.
    fn propagate(
        &mut self,
        step: &Step<'_>,
        result: Result<Propagated, Upstream>,
        point: Option<&KktPoint>,
        conditional: bool,
    ) -> Result<(), WorkflowError> {
        self.validity
            .push(validity::Row {
                run_id: step.run_id,
                step: step.step,
                quantity: DerivedQuantity::PropagatedCovariance,
                validity: record(
                    result
                        .as_ref()
                        .map_err(|u| (WithheldReason::UpstreamWithheld, u.to_string())),
                    point,
                    conditional,
                ),
            })
            .map_err(relation)?;
        if let Ok(propagated) = result {
            let units = Units::of(step)?;
            let output_units = propagated
                .outputs
                .iter()
                .map(|id| units.variable(*id))
                .collect::<Result<_, _>>()?;
            self.propagated
                .push(propagated::Row {
                    run_id: step.run_id,
                    step: step.step,
                    covariance_run_id: propagated.covariance_run,
                    parameters: propagated.parameters,
                    outputs: propagated.outputs,
                    output_units,
                    values: propagated.values,
                })
                .map_err(relation)?;
        }
        Ok(())
    }
    /// The finished relations.
    pub(super) fn finish(self) -> Result<[(SemanticId, FieldCheckedBatch); 4], WorkflowError> {
        Ok([
            (
                validity::RELATION_ID,
                self.validity.finish().map_err(relation)?,
            ),
            (
                sensitivities::RELATION_ID,
                self.sensitivities.finish().map_err(relation)?,
            ),
            (
                hessians::RELATION_ID,
                self.hessians.finish().map_err(relation)?,
            ),
            (
                propagated::RELATION_ID,
                self.propagated.finish().map_err(relation)?,
            ),
        ])
    }
}

/// `J` of a propagation from a step's certified sensitivities: `dx/dp` of each output
/// variable over the request's parameters.
fn jacobian(
    report: &SolveReport,
    parametric: &Parametric,
    s: &pse_backend_native::kkt::Sensitivities,
    outputs: &[SemanticId],
) -> Result<Jacobian, WorkflowError> {
    let columns = outputs
        .iter()
        .map(|id| {
            report
                .variables
                .iter()
                .position(|v| v == id)
                .ok_or_else(|| {
                    contract(format!("propagation output {id} is not a solved variable"))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let n = parametric.parameters.len();
    Ok(Jacobian {
        outputs: outputs.to_vec(),
        parameters: parametric.parameters.clone(),
        values: columns
            .iter()
            .flat_map(|j| (0..n).map(move |k| s.primal[k][*j]))
            .collect(),
    })
}

/// The physical unit of every coordinate a sensitivity row names.
struct Units<'a> {
    structure: &'a CaseStructure,
    quantities: &'a QuantityRegistry,
    variables: BTreeMap<SemanticId, SemanticId>,
}
impl<'a> Units<'a> {
    fn of(step: &Step<'a>) -> Result<Self, WorkflowError> {
        Ok(Self {
            structure: step.structure,
            quantities: step.quantities,
            variables: step
                .structure
                .variables()
                .iter()
                .map(|v| (v.port.id, v.port.unit.as_id()))
                .chain(
                    step.structure
                        .parameters()
                        .iter()
                        .map(|p| (p.id, p.unit.as_id())),
                )
                .collect(),
        })
    }
    fn variable(&self, id: SemanticId) -> Result<SemanticId, WorkflowError> {
        self.variables
            .get(&id)
            .copied()
            .ok_or_else(|| contract(format!("no unit for coordinate {id}")))
    }
    fn parameter(&self, id: SemanticId) -> Result<SemanticId, WorkflowError> {
        self.variable(id)
    }
    fn canonical(
        &self,
        quantity: pse_quantity::QuantityTypeId,
    ) -> Result<SemanticId, WorkflowError> {
        Ok(self
            .quantities
            .quantity_type(quantity)
            .map_err(super::math)?
            .canonical_unit
            .as_id())
    }
    fn row(&self, id: SemanticId) -> Result<SemanticId, WorkflowError> {
        let row = self
            .structure
            .rows()
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| contract(format!("no row {id}")))?;
        self.canonical(row.quantity)
    }
    fn objective(&self) -> Result<SemanticId, WorkflowError> {
        let objective = self
            .structure
            .objective()
            .ok_or_else(|| contract("a sensitivity differentiates an objective"))?;
        self.canonical(objective.quantity)
    }
}
