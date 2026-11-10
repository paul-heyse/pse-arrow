// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original frozen population coordinates, independent of execution allocation.
use super::super::{
    BindingAssignment, BindingQuantity, BindingTarget, PointOverlay, WorkflowError, contract,
};
use super::admission::LawCoordinate;
use super::*;
use pse_ids::SemanticId;
use pse_math::integration::{Constructible, Grid, MonteCarloRng, Sample};
use pse_model::generated::identities::StudyId;
use pse_model::scalars::FiniteBound;

/// Complete original attribution of one independently counted scientific occurrence.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectationSample {
    /// Original global ordinal, unaffected by equal realized values.
    pub ordinal: u64,
    /// Frozen proposal iteration.
    pub iteration: usize,
    /// Original independent random stream/lane.
    pub lane: usize,
    /// Ordinal within this lane's retained stream.
    pub lane_ordinal: usize,
    /// Unit-cube coordinates in original continuous coordinate order.
    pub base: Vec<f64>,
    /// Original authored category identities in categorical coordinate order.
    pub categories: Vec<SemanticId>,
    /// Product of admitted target category masses; continuous target density is one.
    pub target_mass: f64,
    /// Inverse full proposal probability/density; fixed target sampling uses one.
    pub proposal_weight: f64,
    /// Canonical physically typed assignments before the optional joint transform.
    pub base_overlay: PointOverlay,
    /// Original physically admitted member assignments after the transform.
    pub overlay: PointOverlay,
    pub(super) proposal: Option<Sample<f64>>,
}

/// One immutable child population. Its scientific child never accepts later samples.
#[derive(Clone, Debug)]
pub struct ExpectationIteration {
    /// Original expectation parent.
    pub parent: StudyId,
    /// Caller-issued original finite child, recorded before execution.
    pub child: StudyId,
    /// Frozen iteration ordinal.
    pub iteration: usize,
    /// Complete original ordered sample attribution.
    pub samples: std::sync::Arc<[ExpectationSample]>,
    pub(super) request: Vec<u8>,
    pub(super) rng_after: Vec<[u8; 32]>,
    pub(super) _owner: std::sync::Arc<pse_columnar::AllocationLease>,
}

/// Original physical observations selected from the exact child occurrence.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberObservation {
    /// Exact original finite child inventory used for reduction.
    pub child: StudyId,
    /// Original global sample ordinal, not a realized value or binding hash.
    pub ordinal: u64,
    /// Canonical physical output magnitudes in the admitted demand order.
    pub values: Vec<Option<f64>>,
    /// Explicit scientific classification; operational outcomes do not supply an indicator.
    pub scientific_failure: Option<bool>,
    /// Cancellation, infrastructure loss or unsupported interpretation remains incomplete.
    pub operational_incomplete: bool,
}

fn invalid_pushforward(
    admitted: &AdmittedExpectation,
    iteration: &ExpectationIteration,
    sample: &ExpectationSample,
    cause: WorkflowError,
) -> WorkflowError {
    use pse_model::diagnostic::{
        BoundaryClass, BoundaryDiagnostic, DiagnosticRule, DiagnosticStage, Observation,
        PhysicalObservation,
    };
    let sources = admitted
        .request
        .transform
        .as_ref()
        .map(|transform| vec![transform.operation.root.as_id()])
        .unwrap_or_default();
    let mut diagnostic = BoundaryDiagnostic::new(
        BoundaryClass::InvalidModel,
        DiagnosticStage::StudyAdmission,
        sources,
        DiagnosticRule::StudyPolicyAdmission,
    );
    diagnostic.causes.push(cause.boundary_diagnostic());
    for (name, value) in [
        ("expectation_parent", iteration.parent.to_string()),
        ("expectation_proposed_child", iteration.child.to_string()),
        ("sample_ordinal", sample.ordinal.to_string()),
        ("sample_iteration", sample.iteration.to_string()),
        ("sample_lane", sample.lane.to_string()),
        (
            "law_refusal",
            "invalid declared pushforward at a positive-mass base point".into(),
        ),
    ] {
        diagnostic
            .observations
            .insert(name.into(), Observation::Text(value));
    }
    diagnostic.observations.insert(
        "target_mass".into(),
        Observation::number(sample.target_mass),
    );
    diagnostic.observations.insert(
        "proposal_weight".into(),
        Observation::number(sample.proposal_weight),
    );
    for (index, value) in sample.base.iter().enumerate() {
        diagnostic.observations.insert(
            format!("base_coordinate_{index}"),
            Observation::number(*value),
        );
    }
    for (index, category) in sample.categories.iter().enumerate() {
        diagnostic.observations.insert(
            format!("category_{index}"),
            Observation::Text(category.to_string()),
        );
    }
    for (index, assignment) in sample.base_overlay.assignments.iter().enumerate() {
        diagnostic.observations.insert(
            format!("base_physical_{index}"),
            Observation::Physical(PhysicalObservation {
                magnitude: assignment.value.magnitude.into_inner(),
                quantity: assignment.value.quantity.as_id(),
                unit: assignment.value.unit.as_id(),
                context: admitted.request.source.physical_context,
            }),
        );
    }
    if sample.overlay != sample.base_overlay {
        for (index, assignment) in sample.overlay.assignments.iter().enumerate() {
            diagnostic.observations.insert(
                format!("realized_physical_{index}"),
                Observation::Physical(PhysicalObservation {
                    magnitude: assignment.value.magnitude.into_inner(),
                    quantity: assignment.value.quantity.as_id(),
                    unit: assignment.value.unit.as_id(),
                    context: admitted.request.source.physical_context,
                }),
            );
        }
    }
    WorkflowError::ScientificRefusal(Box::new(diagnostic))
}

impl ExpectationIteration {
    /// Admit the joint pushforward and exact finite ordinary child through existing owners.
    /// A caller records this immutable child under `self.child` before issuing any member.
    pub async fn admit_child(
        &mut self,
        admitted: &AdmittedExpectation,
        services: &super::super::ExecutionServices,
        cancel: &crate::CancelSource,
    ) -> Result<super::super::LocalStudy, WorkflowError> {
        if self.request
            != serde_json::to_vec(&admitted.request).map_err(|error| contract(error.to_string()))?
        {
            return Err(contract(
                "expectation child belongs to another original scientific request",
            ));
        }
        let preparation = services.preparation();
        if let Some(transform) = &admitted.request.transform {
            let model = admitted
                .transform_model
                .as_ref()
                .ok_or_else(|| contract("expectation original transform preparation absent"))?;
            let program = admitted
                .original
                .prepare_selected_observation(
                    preparation,
                    &transform.operation,
                    &admitted.request.preparation,
                    None,
                    cancel,
                )
                .await?;
            let bindings = self
                .samples
                .iter()
                .map(|sample| {
                    let identity = SemanticId::from_bytes(u128::from(sample.ordinal).to_le_bytes());
                    admitted
                        .original
                        .admit_overlay(model.compiled(), &sample.base_overlay)
                        .map(|binding| (identity, binding))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let population = program
                .evaluate_batch(preparation, bindings, cancel)
                .await?;
            for sample_index in 0..self.samples.len() {
                cancel
                    .token()
                    .checkpoint()
                    .map_err(pse_engine::EngineError::from)?;
                let values = match population.canonical(sample_index) {
                    Some(values) => values,
                    None => {
                        let cause = match &population.original.points()[sample_index].outcome {
                            pse_math::batch::PointOutcome::Failed(error) => {
                                WorkflowError::from(pse_model::diagnostic::project_typed(
                                    error,
                                    pse_diagnostics::DiagnosticStage::Evaluation,
                                ))
                            }
                            pse_math::batch::PointOutcome::Canceled
                            | pse_math::batch::PointOutcome::NotExecuted => {
                                return Err(crate::math::MathRuntimeError::Cancelled.into());
                            }
                            pse_math::batch::PointOutcome::Succeeded => contract(
                                "expectation joint transform has no original physical value",
                            ),
                        };
                        return Err(invalid_pushforward(
                            admitted,
                            self,
                            &self.samples[sample_index],
                            cause,
                        ));
                    }
                };
                let mut assignments = Vec::with_capacity(transform.outputs.len());
                for output in &transform.outputs {
                    let index = population.members.iter().position(|member| *member == output.output).ok_or_else(|| contract("expectation joint transform did not supply its selected physical output"))?;
                    let value = *values
                        .get(index)
                        .ok_or_else(|| contract("expectation joint transform output extent"))?;
                    let symbol = model
                        .compiled()
                        .model
                        .symbols
                        .get(&output.output)
                        .ok_or_else(|| {
                            contract("expectation joint transform output source absent")
                        })?;
                    let physical = symbol
                        .ty
                        .quantity_scheme()
                        .ok_or_else(|| {
                            contract("expectation joint transform output is not physical")
                        })?
                        .resolve_contract_with_evidence(
                            &admitted.original.quantities,
                            &std::collections::BTreeMap::new(),
                            admitted.original.physical.preconditions.as_ref(),
                        )
                        .map_err(|error| {
                            WorkflowError::from(pse_model::diagnostic::project_typed(
                                &error,
                                pse_diagnostics::DiagnosticStage::StudyAdmission,
                            ))
                        })?;
                    let quantity = physical.named_id().ok_or_else(|| {
                        contract(
                            "expectation joint transform output has no declared physical quantity",
                        )
                    })?;
                    let unit = admitted
                        .original
                        .quantities
                        .quantity_type(quantity)
                        .map_err(|error| super::super::math(pse_math::MathError::Quantity(error)))?
                        .canonical_unit;
                    assignments.push(BindingAssignment {
                        target: output.target.clone(),
                        value: BindingQuantity {
                            quantity,
                            unit,
                            magnitude: FiniteBound::try_new(value).map_err(|_| {
                                contract("expectation joint transform output is not finite")
                            })?,
                        },
                    });
                }
                std::sync::Arc::make_mut(&mut self.samples)[sample_index].overlay =
                    PointOverlay { assignments };
            }
        }
        for sample in self.samples.iter() {
            cancel
                .token()
                .checkpoint()
                .map_err(pse_engine::EngineError::from)?;
            if let Err(cause) = admitted.validate_realized(&sample.overlay) {
                return Err(if admitted.request.transform.is_some() {
                    invalid_pushforward(admitted, self, sample, cause)
                } else {
                    cause
                });
            }
        }
        let operation = match &admitted.request.member {
            ExpectationMember::Case(case) => {
                super::super::OperationRequest::DeclaredCase(case.clone())
            }
            ExpectationMember::Observation(operation) => {
                super::super::OperationRequest::SelectedObservation(operation.clone())
            }
        };
        let points = self
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| {
                Ok(super::super::StudyPoint {
                    operation: operation.clone(),
                    preparation: admitted.request.preparation.clone(),
                    overlay: sample.overlay.clone(),
                    policy: super::super::StudyPointPolicy {
                        key: pse_model::study::OccurrenceKey(
                            u32::try_from(index)
                                .map_err(|_| contract("expectation child occurrence index"))?,
                        ),
                        dependencies: vec![],
                        start: pse_model::study::StartPolicy::Fresh,
                        attempt_limit: 1,
                    },
                })
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        admitted
            .original
            .admit_study(
                services,
                &super::super::StudyInputs::Explicit(points),
                &super::super::StudyOutputDemand::Full {},
                cancel,
            )
            .await
    }
}

fn assignment(
    member: SemanticId,
    quantity: pse_quantity::QuantityTypeId,
    unit: pse_quantity::UnitId,
    value: f64,
) -> Result<BindingAssignment, WorkflowError> {
    Ok(BindingAssignment {
        target: BindingTarget::Member(member),
        value: BindingQuantity {
            quantity,
            unit,
            magnitude: FiniteBound::try_new(value)
                .map_err(|_| contract("expectation realized coordinate is nonfinite"))?,
        },
    })
}

pub(super) fn draw(
    admitted: &AdmittedExpectation,
    rng: &mut MonteCarloRng,
    grid: Option<&mut Grid<f64>>,
    ordinal: u64,
    iteration: usize,
    lane: usize,
    lane_ordinal: usize,
) -> Result<ExpectationSample, WorkflowError> {
    let mut sample = Sample::new();
    let adaptive = if let Some(grid) = grid {
        grid.sample(rng, &mut sample);
        true
    } else {
        false
    };
    let proposal_weight = if adaptive { sample.get_weight() } else { 1. };
    if !proposal_weight.is_finite() || proposal_weight <= 0. {
        return Err(contract(
            "expectation proposal has no finite positive inverse density",
        ));
    }
    let mut discrete = Vec::new();
    let mut continuous = Vec::new();
    if adaptive {
        let mut selected = &sample;
        loop {
            match selected {
                Sample::Discrete(_, index, child) => {
                    discrete.push(*index);
                    let Some(child) = child else {
                        break;
                    };
                    selected = child;
                }
                Sample::Continuous(_, values) => {
                    continuous.extend_from_slice(values);
                    break;
                }
                _ => {
                    return Err(contract(
                        "expectation proposal has an unsupported sample shape",
                    ));
                }
            }
        }
    }
    let mut base = Vec::new();
    let mut categories = Vec::new();
    let mut assignments = Vec::new();
    let mut target_mass = 1_f64;
    let mut continuous_index = 0;
    let mut discrete_index = 0;
    for coordinate in admitted.coordinates.iter() {
        match coordinate {
            LawCoordinate::Uniform {
                member,
                quantity,
                unit,
                lower,
                upper,
            } => {
                let u = if adaptive {
                    *continuous.get(continuous_index).ok_or_else(|| {
                        contract("expectation continuous sample dimension differs")
                    })?
                } else {
                    f64::new_sample_unit(rng)
                };
                continuous_index += 1;
                base.push(u);
                // Convex interpolation avoids an overflowing finite endpoint difference.
                let value = (1. - u) * lower + u * upper;
                assignments.push(assignment(*member, *quantity, *unit, value)?);
            }
            LawCoordinate::Categorical {
                member,
                quantity,
                unit,
                categories: population,
            } => {
                let positive = population
                    .iter()
                    .filter(|(_, _, mass)| *mass > 0.)
                    .collect::<Vec<_>>();
                let chosen = if adaptive {
                    let index = *discrete.get(discrete_index).ok_or_else(|| {
                        contract("expectation categorical sample dimension differs")
                    })?;
                    discrete_index += 1;
                    *positive
                        .get(index)
                        .ok_or_else(|| contract("expectation active category mapping differs"))?
                } else {
                    let value = f64::new_sample_unit(rng);
                    let mut sum = 0.;
                    let mut selected = positive
                        .last()
                        .copied()
                        .ok_or_else(|| contract("expectation positive category absent"))?;
                    for category in &positive {
                        sum += category.2;
                        if value < sum {
                            selected = category;
                            break;
                        }
                    }
                    selected
                };
                categories.push(chosen.0);
                target_mass *= chosen.2;
                assignments.push(assignment(*member, *quantity, *unit, chosen.1)?);
            }
        }
    }
    if !target_mass.is_finite() || target_mass <= 0. {
        return Err(contract("expectation target mass is not representable"));
    }
    let overlay = PointOverlay { assignments };
    Ok(ExpectationSample {
        ordinal,
        iteration,
        lane,
        lane_ordinal,
        base,
        categories,
        target_mass,
        proposal_weight,
        base_overlay: overlay.clone(),
        overlay,
        proposal: adaptive.then_some(sample),
    })
}
