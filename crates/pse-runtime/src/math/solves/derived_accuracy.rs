// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A complete selected reconstruction's actual coordinate coverage, before publication.
use super::*;
use pse_math::{engineering_accuracy::GoalResult, implicit::ProofInterval};
use pse_model::{
    engineering_accuracy::{AccuracyGoal, BoundGoal, OutputEvidence},
    generated::enums::{
        AccuracyEvidenceInterpretation, AccuracyEvidenceMethod, AccuracyGoalSubject,
        AccuracyObservation, AccuracyUnavailableReason, NumericalTarget,
    },
    strategy::{AccuracyClass, SemanticProductKey},
};

/// Sealed receipt issued only after an actual complete selected-root reconstruction.
/// Its box covers every original free coordinate; partial reduced points cannot issue it.
#[derive(Debug)]
pub(crate) struct CertifiedReconstructionPoint {
    original_source: SemanticProductKey,
    source: SemanticProductKey,
    validity: pse_ids::ContentHash,
    point_product: pse_ids::ContentHash,
    #[cfg(feature = "solver-root-isolation")]
    point_accuracy: AccuracyEvidence,
    coordinates: Vec<(pse_ids::SemanticId, f64, ProofInterval)>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
#[cfg(any(feature = "solver-root-isolation", test))]
fn positive_upper_product(a: f64, b: f64) -> f64 {
    if a == 0. || b == 0. {
        0.
    } else {
        (a * b).next_up()
    }
}
impl CertifiedReconstructionPoint {
    pub(super) fn issue(
        service: &MathService,
        prepared: &PreparedDerived,
        point: &math::ReconstructionObservation,
        realization: pse_ids::ContentHash,
    ) -> Result<Option<Arc<Self>>, MathRuntimeError> {
        if prepared.original.numerics.policy.goals.is_empty() || !prepared.complete_reconstruction()
        {
            return Ok(None);
        }
        let Some(reduced) = &prepared.reduced else {
            return Ok(None);
        };
        let Some(error) = point.accuracy.error.filter(|e| e.is_finite() && *e >= 0.) else {
            return Ok(None);
        };
        if point.accuracy.class != AccuracyClass::Certified
            || point.accuracy.normalization != prepared.physical.normalization()
            || !reduced.contract.retained().is_empty()
            || point.values.len() != prepared.physical.coordinates().len()
            || point.values.len() != prepared.original.normalization.variables.len()
        {
            return Ok(None);
        }
        let bytes = point
            .values
            .len()
            .checked_mul(size_of::<(pse_ids::SemanticId, f64, ProofInterval)>())
            .and_then(|bytes| bytes.checked_add(size_of::<Self>() + 128))
            .ok_or_else(|| ProblemError::memory("certified reconstruction receipt extent"))?;
        let owner = service.reserve("math:certified-reconstruction-point", bytes)?;
        let mut coordinates = Vec::with_capacity(point.values.len());
        for ((coordinate, value), scale) in prepared
            .physical
            .coordinates()
            .iter()
            .zip(&point.values)
            .zip(&prepared.original.normalization.variables)
        {
            if !value.is_finite() || !scale.is_finite() || *scale <= 0. {
                return Err(ProblemError::Contract(
                    "certified reconstruction physical coordinates".into(),
                )
                .into());
            }
            let radius = if error == 0. {
                0.
            } else {
                (error * scale).next_up()
            };
            let interval = if radius == 0. {
                ProofInterval {
                    lower: *value,
                    upper: *value,
                }
            } else {
                ProofInterval {
                    lower: (value - radius).next_down(),
                    upper: (value + radius).next_up(),
                }
            };
            if !interval.valid() {
                return Ok(None);
            }
            coordinates.push((coordinate.id, *value, interval));
        }
        let original_source = prepared.original.semantic_point_key(&point.values)?;
        let source = SemanticProductKey {
            derivation: Some(prepared.family.key()),
            branch: Some(realization),
            accuracy: Some(point.accuracy.product),
            ..original_source
        };
        let mut validity = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        validity
            .str("actual-complete-selected-reconstruction")
            .hash(&reduced.contract.validity())
            .hash(&realization)
            .hash(&point.accuracy.product)
            .hash(&point.accuracy.normalization)
            .f64(error);
        for (id, value, interval) in &coordinates {
            validity
                .id(id)
                .f64(*value)
                .f64(interval.lower)
                .f64(interval.upper);
        }
        Ok(Some(Arc::new(Self {
            original_source,
            source,
            validity: validity.finish_hash(),
            point_product: point.accuracy.product,
            #[cfg(feature = "solver-root-isolation")]
            point_accuracy: point.accuracy,
            coordinates,
            _owner: owner,
        })))
    }
    /// Exact producer and selected sheet dependencies, including the observed point.
    #[cfg(feature = "solver-root-isolation")]
    pub(crate) fn source(&self) -> SemanticProductKey {
        self.source
    }
    #[cfg(feature = "solver-root-isolation")]
    pub(crate) fn validity(&self) -> pse_ids::ContentHash {
        self.validity
    }
    pub(crate) fn bound_goal(&self, goal: &AccuracyGoal) -> BoundGoal {
        let mut product = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        product
            .str("complete-reconstruction-selected-output")
            .id(&goal.target_id)
            .hash(&self.validity)
            .hash(&self.point_product);
        let mut unit = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        unit.str("physical-output-error")
            .id(&goal.quantity_id)
            .id(&goal.unit_id);
        BoundGoal {
            declaration: goal.clone(),
            source: self.source,
            product: product.finish_hash(),
            normalization: unit.finish_hash(),
        }
    }
    fn matches(&self, original: &PreparedSolve, values: &CaseValues) -> Result<bool, ProblemError> {
        let Representation::Algebraic(case) = &original.representation else {
            return Ok(false);
        };
        let columns = case.prepared.compiled().plan.columns();
        if columns.len() != self.coordinates.len()
            || columns
                .iter()
                .zip(&self.coordinates)
                .any(|(id, (expected, point, _))| {
                    id != expected
                        || values
                            .scalars
                            .get(id)
                            .is_none_or(|value| value.to_bits() != point.to_bits())
                })
        {
            return Ok(false);
        }
        // Fixed dependencies, normalization and numerical context must still be frozen.
        for (id, value) in &case.values.scalars {
            if !columns.contains(id)
                && values
                    .scalars
                    .get(id)
                    .is_none_or(|actual| actual.to_bits() != value.to_bits())
            {
                return Ok(false);
            }
        }
        let point = self
            .coordinates
            .iter()
            .map(|(_, point, _)| *point)
            .collect::<Vec<_>>();
        Ok(original.semantic_point_key(&point)? == self.original_source)
    }
    /// Allocate this box only after original scientific admission; the caller owns its
    /// transient storage and any subsequent source evaluator work.
    #[cfg(feature = "solver-root-isolation")]
    pub(crate) fn coordinate_box(
        &self,
        original: &PreparedSolve,
        values: &CaseValues,
    ) -> Result<Option<Vec<ProofInterval>>, ProblemError> {
        if !self.matches(original, values)? {
            return Ok(None);
        }
        Ok(Some(
            self.coordinates
                .iter()
                .map(|(_, _, interval)| *interval)
                .collect(),
        ))
    }
    /// Classify only directly reconstructed variables. Authored functions require a
    /// separate interval operation over this complete box; point gradients do not suffice.
    pub(crate) fn assess_coordinates(
        &self,
        original: &PreparedSolve,
        values: &CaseValues,
    ) -> Result<Option<Vec<GoalResult>>, ProblemError> {
        if !self.matches(original, values)? {
            return Ok(None);
        }
        Ok(Some(original.numerics.policy.goals.iter().map(|goal| {
            if goal.subject != AccuracyGoalSubject::SelectedOutput || goal.observation != AccuracyObservation::Steady
                || goal.target_kind != NumericalTarget::Variable {
                return GoalResult::unavailable(goal.clone(), AccuracyUnavailableReason::UnsupportedObservation);
            }
            let Some((_, value, interval)) = self.coordinates.iter().find(|(id, _, _)| *id == goal.target_id) else {
                return GoalResult::unavailable(goal.clone(), AccuracyUnavailableReason::Unsupported);
            };
            let bound = self.bound_goal(goal);
            let radius = (value - interval.lower).abs().max((interval.upper - value).abs());
            let error = if radius == 0. { 0. } else { radius.next_up() };
            GoalResult::assess(&bound, Some(OutputEvidence { target: goal.target_id, target_kind: goal.target_kind,
                quantity: goal.quantity_id, unit: goal.unit_id, observation: goal.observation, time: goal.time,
                value: Some(*value), accuracy: AccuracyEvidence { product: bound.product, normalization: bound.normalization,
                    class: AccuracyClass::Certified, error: Some(error) }, source: self.source, validity: Some(self.validity),
                interpretation: AccuracyEvidenceInterpretation::OutputError, method: AccuracyEvidenceMethod::CertifiedEnclosure,
                interval: Some((interval.lower, interval.upper)),
                limitation: "complete selected-root reconstruction; original checks assessed independently".into(), }))
        }).collect()))
    }
    /// Demand on the actual normalized reconstructed point from named consuming
    /// outputs. Direct projection has an exact scale gain; authored outputs consume
    /// the interval library's uniform First influence across this same certified box.
    /// This allocates work only and grants no output or original-model permission.
    #[cfg(feature = "solver-root-isolation")]
    pub(super) fn refinement_demand(
        &self,
        service: &MathService,
        original: &PreparedSolve,
        values: &CaseValues,
        scope: &ExecutionScope,
        budget: &Arc<WorkerBudget>,
        execution: &Execution,
    ) -> Result<Option<pse_model::strategy::AccuracyDemand>, ProblemError> {
        use native::root_isolation::{Ibex, PointArithmeticEvidence};
        use pse_math::engineering_accuracy::{Contribution, allocate, refinement_allowance};
        if !self.matches(original, values)? {
            return Ok(None);
        }
        execution.check()?;
        let observable = original.numerics.policy.goals.iter().any(|goal| {
            goal.refine
                && goal.target_kind == NumericalTarget::Observable
                && goal.subject == AccuracyGoalSubject::SelectedOutput
                && goal.observation == AccuracyObservation::Steady
        });
        let observed = if observable {
            original.evaluated_observable_outputs(service, values, scope, budget, execution)?
        } else {
            None
        };
        let uniform = if observed.is_some() {
            let Some(selected) = original.selected_output_program() else {
                return Ok(None);
            };
            if selected
                .executable
                .assembly
                .columns()
                .iter()
                .zip(&self.coordinates)
                .any(|(id, (coordinate, _, _))| id != coordinate)
                || selected.executable.assembly.columns().len() != self.coordinates.len()
            {
                return Ok(None);
            }
            let available = budget.capacity().saturating_sub(budget.used());
            if available < 256 {
                return Ok(None);
            }
            let projection_charge = budget
                .charge(available)
                .map_err(MathRuntimeError::into_problem)?;
            let program = match selected
                .executable
                .assembly
                .point_arithmetic_program_for_order(
                    values,
                    DerivativeOrder::First,
                    available / 256,
                    &execution.cancel,
                ) {
                Ok(Some(program)) => program,
                Ok(None) => return Ok(None),
                Err(pse_math::factorable::FactorableError::Math(error)) => return Err(error.into()),
                Err(_) => return Ok(None),
            };
            drop(projection_charge);
            let _program = budget
                .charge(program.retained_bytes())
                .map_err(MathRuntimeError::into_problem)?;
            let bytes = match Ibex
                .point_arithmetic_workspace_bytes_for_order(&program, DerivativeOrder::First)
            {
                Ok(bytes) if bytes <= budget.capacity().saturating_sub(budget.used()) => bytes,
                _ => return Ok(None),
            };
            let _workspace = budget
                .charge(bytes)
                .map_err(MathRuntimeError::into_problem)?;
            let region = self
                .coordinates
                .iter()
                .map(|(_, _, interval)| *interval)
                .collect::<Vec<_>>();
            match Ibex.enclose_arithmetic_box_with_execution(
                &program,
                &region,
                DerivativeOrder::First,
                bytes,
                execution,
            )? {
                PointArithmeticEvidence::Enclosed { jacobian, .. } => {
                    Some((program, jacobian, _program, _workspace))
                }
                _ => None,
            }
        } else {
            None
        };
        let mut requested: Option<pse_model::strategy::AccuracyDemand> = None;
        for goal in &original.numerics.policy.goals {
            if goal.subject != AccuracyGoalSubject::SelectedOutput
                || goal.observation != AccuracyObservation::Steady
            {
                continue;
            }
            let (value, gain, floor) = match goal.target_kind {
                NumericalTarget::Variable => {
                    let Some(column) = self
                        .coordinates
                        .iter()
                        .position(|(id, _, _)| *id == goal.target_id)
                    else {
                        continue;
                    };
                    (
                        self.coordinates[column].1,
                        original.normalization.variables[column],
                        0.,
                    )
                }
                NumericalTarget::Observable if goal.refine => {
                    let (Some(observed), Some((program, jacobian, _, _)), Some(selected)) =
                        (&observed, &uniform, original.selected_output_program())
                    else {
                        continue;
                    };
                    let Some(output) = observed.outputs.get(&goal.target_id) else {
                        continue;
                    };
                    let Some(row_id) = selected.rows.get(&goal.target_id) else {
                        continue;
                    };
                    let Some(row) = selected
                        .executable
                        .assembly
                        .structure()
                        .rows()
                        .iter()
                        .position(|row| row.id == *row_id)
                    else {
                        continue;
                    };
                    let output_index = program.rows[row].value;
                    let mut gain = 0.;
                    for (column, scale) in original.normalization.variables.iter().enumerate() {
                        let entry = jacobian[output_index * self.coordinates.len() + column];
                        let term = positive_upper_product(
                            entry.lower.abs().max(entry.upper.abs()),
                            *scale,
                        );
                        if term > 0. {
                            gain = (gain + term).next_up();
                        }
                    }
                    (output.value, gain, output.uncertainty)
                }
                _ => continue,
            };
            let bound = self.bound_goal(goal);
            // Direct declared resolutions keep their previous operational tightness;
            // decision slack and authored functions request refinement only when allowed.
            let allowance = if goal.refine {
                refinement_allowance(&bound, value)
            } else {
                goal.resolution
                    .filter(|_| goal.target_kind == NumericalTarget::Variable)
            };
            let Some(allowance) = allowance else {
                continue;
            };
            let allocation = allocate(
                allowance,
                AccuracyClass::Certified,
                floor,
                &[Contribution {
                    evidence: self.point_accuracy,
                    gain,
                    gain_class: AccuracyClass::Certified,
                    reducible: true,
                }],
            )?;
            for demand in allocation.demands {
                if demand.allowance > 0.
                    && requested
                        .as_ref()
                        .is_none_or(|old| demand.allowance < old.allowance)
                {
                    requested = Some(demand);
                }
            }
        }
        execution.check()?;
        Ok(requested)
    }
    #[cfg(not(feature = "solver-root-isolation"))]
    pub(super) fn refinement_demand(
        &self,
        _service: &MathService,
        _original: &PreparedSolve,
        _values: &CaseValues,
        _scope: &ExecutionScope,
        _budget: &Arc<WorkerBudget>,
        _execution: &Execution,
    ) -> Result<Option<pse_model::strategy::AccuracyDemand>, ProblemError> {
        Ok(None)
    }
}

#[cfg(test)]
mod gain_tests {
    use super::*;
    #[test]
    fn engineering_accuracy_reconstruction_tiny_uniform_gain_retains_material_point_error_demand() {
        use pse_math::engineering_accuracy::{Contribution, allocate};
        // The uniform physical derivative and physical coordinate scale are both
        // positive, while their binary64 product underflows. A large actual
        // normalized point error still makes this contribution material.
        let gain = positive_upper_product(1e-200, 1e-200);
        assert_eq!(1e-200 * 1e-200, 0.);
        assert!(gain > 0. && gain.is_finite());
        assert_eq!(positive_upper_product(0., 1e-200), 0.);
        let evidence = AccuracyEvidence {
            product: pse_ids::ContentHash::from_bytes([1; 32]),
            normalization: pse_ids::ContentHash::from_bytes([2; 32]),
            class: AccuracyClass::Certified,
            error: Some(1e308),
        };
        let allocation = allocate(
            1e-94,
            AccuracyClass::Certified,
            0.,
            &[Contribution {
                evidence,
                gain,
                gain_class: AccuracyClass::Certified,
                reducible: true,
            }],
        )
        .unwrap();
        assert!(allocation.error.unwrap() > 1e-94);
        assert_eq!(allocation.demands.len(), 1);
        assert!(allocation.demands[0].allowance > 0. && allocation.demands[0].allowance < 1e308);
    }
}
