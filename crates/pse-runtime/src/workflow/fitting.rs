// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared-parameter experiment compilation. Trial values never enter Salsa queries.
mod oracle;
mod results;
use super::{FitDeclaration, ModelRevision, PreparedSimulation, WorkflowError, contract, math};
use crate::math::{ExecutableCase, solves::SolverProfile};
use pse_backend_native::{
    self as native, OracleContract, Variable,
    solve::{HessianMode, SolveIntent},
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, Port};
use pse_math::{
    binding::CaseValues,
    normalization::Normalization,
    numerics::{SourcedRequirement, TargetSpec},
};
use pse_model::SemanticFrame;
use pse_model::{
    generated::enums::{NumericalCoordinates, NumericalSource, NumericalTarget},
    numerics::{NumericalRequirement, ResolvedNumericalPolicy},
};
use pse_quantity::UnitId;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
/// Controls for the ordinary NLP and explicit dynamic experiment profiles.
#[derive(Clone, Debug)]
pub struct FitProfile {
    /// Ordinary native NLP controls and original-space quality tolerances.
    pub solver: SolverProfile,
    /// Smooth integration controls keyed by experiment identity.
    pub simulations: BTreeMap<SemanticId, super::SimulationProfile>,
    /// Relative local response singular-value cutoff; not a confidence level.
    pub rank_tolerance: f64,
    /// Hard cap for dense local response/rank storage and composed derivatives.
    pub max_cells: usize,
}
#[derive(Clone, Debug)]
struct Measurement {
    id: SemanticId,
    experiment: usize,
    row: usize,
    time: Option<f64>,
    #[cfg_attr(
        not(feature = "solver-diffsol"),
        expect(
            dead_code,
            reason = "prepared sample binding is consumed by the linked dynamic fit oracle"
        )
    )]
    sample_index: Option<usize>,
    included: bool,
    value: Option<f64>,
    sigma: Option<f64>,
    importance: f64,
    port: Port,
}
#[derive(Clone, Debug)]
struct Steady {
    case: Arc<ExecutableCase>,
    values: CaseValues,
    coordinates: Vec<(SemanticId, usize)>,
    constraints: Vec<(usize, usize)>,
    local_states: usize,
}
#[derive(Clone, Debug)]
enum Experiment {
    Steady(Steady),
    Transient(Box<PreparedSimulation>),
}
/// Immutable fitting product; mutable evaluators and native sessions are attempt-owned.
#[derive(Clone, Debug)]
pub(crate) struct FitProblem {
    pub(crate) revision: ModelRevision,
    pub(crate) declaration: FitDeclaration,
    pub(crate) profile: FitProfile,
    pub(crate) key: ContentHash,
    pub(crate) profile_key: ContentHash,
    pub(crate) numerics: Arc<ResolvedNumericalPolicy>,
    pub(crate) normalization: Normalization,
    pub(crate) tolerances: native::quality::Tolerances,
    pub(crate) bytes: usize,
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    initial: Vec<f64>,
    parameter_ports: Vec<Port>,
    parameter_columns: Vec<Option<usize>>,
    experiments: Vec<Experiment>,
    measurements: Vec<Measurement>,
}
/// Fitting mathematics with an admitted execution route.
#[derive(Clone, Debug)]
pub struct PreparedFit {
    pub(crate) problem: FitProblem,
    route: native::routing::Route,
}
/// Joined native fit plus independently re-evaluated physical predictions.
#[derive(Debug)]
pub struct FitReport {
    /// Physical source/accumulation checks independent of the NLP and response rank.
    pub physical: Vec<super::BalanceCheck>,
    /// Ordinary native NLP report, absent for all-fixed evaluation.
    pub solve: Option<native::solve::SolveReport>,
    /// Independent original constraint and bound quality, including all-fixed evaluation.
    pub quality: Option<native::quality::Quality>,
    /// Independently evaluated steady physical constraints in admitted order.
    pub constraint_values: Vec<f64>,
    /// Fresh weighted least-squares objective, if evaluated.
    pub objective: Option<f64>,
    /// Complete original coordinate vector, absent without a candidate.
    pub candidate: Option<Vec<f64>>,
    /// Predictions in binding order, including excluded observations when evaluable.
    pub predictions: Vec<Option<f64>>,
    /// Local response derivatives in observation by free-parameter order.
    pub responses: Option<faer::Mat<f64>>,
    /// Singular values of the weighted, parameter-scaled response Jacobian.
    pub singular_values: Vec<f64>,
    /// Local numerical column rank, when independently qualified.
    pub rank: Option<usize>,
    /// Why prediction or local sensitivity qualification is unavailable.
    pub diagnostic: Option<String>,
}
impl FitReport {
    /// A stationary feasible estimate with locally identifiable free parameters.
    /// This establishes neither global optimality nor a statistical confidence interval.
    pub fn estimate_qualified(&self) -> bool {
        self.solve.as_ref().is_some_and(|s| {
            matches!(
                s.qualification,
                native::solve::Qualification::Stationary
                    | native::solve::Qualification::OptimalWithinTolerance
            )
        }) && self
            .quality
            .as_ref()
            .is_some_and(native::quality::Quality::feasible)
            && self
                .responses
                .as_ref()
                .is_some_and(|j| j.ncols() > 0 && self.rank == Some(j.ncols()))
    }
    /// Spectral condition estimate of the weighted, parameter-scaled response at the candidate.
    pub fn response_condition(&self) -> Option<f64> {
        let largest = self.singular_values.first()?;
        let smallest = self.singular_values.last()?;
        let ratio = largest / smallest;
        (*smallest > 0.0 && ratio.is_finite()).then_some(ratio)
    }
}
impl PreparedFit {
    /// Complete immutable source/execution identity.
    pub fn identity(&self) -> ContentHash {
        self.problem.key
    }
    /// Admitted execution route, including direct constant evaluation.
    pub fn route(&self) -> native::routing::Route {
        self.route
    }
    /// Original generated fit declaration.
    pub fn declaration(&self) -> &FitDeclaration {
        &self.problem.declaration
    }
}
fn alias(experiment: SemanticId, source: SemanticId) -> SemanticId {
    let mut h = FramedHasher::new("pse.fit.coordinate.v1");
    h.id(&experiment).id(&source);
    h.finish_id()
}
impl ModelRevision {
    /// Compile simultaneous steady and smooth transient experiments through the shared compiler.
    pub async fn prepare_fit(
        &self,
        id: SemanticId,
        profile: FitProfile,
        compiler: pse_compiler::workspace::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedFit, WorkflowError> {
        let problem = self
            .prepare_fit_problem(id, profile, compiler, cancel)
            .await?;
        let facts = native::routing::oracle_facts(
            &problem.contract,
            true,
            problem.bounds.iter().all(|(a, b)| a.is_finite() && a == b),
        );
        let route = native::routing::Requirements {
            available: Some(crate::math::solves::ALGEBRAIC_BACKENDS),
            facts: &facts,
            intent: problem.profile.solver.intent,
            convex: false,
            controls: &problem.profile.solver.controls,
        }
        .select(problem.profile.solver.selection)
        .map_err(crate::math::MathRuntimeError::from)?;
        crate::math::solves::admit_profile(&problem.profile.solver, route)
            .map_err(crate::math::MathRuntimeError::from)?;
        Ok(PreparedFit { problem, route })
    }

    async fn prepare_fit_problem(
        &self,
        id: SemanticId,
        mut profile: FitProfile,
        compiler: pse_compiler::workspace::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<FitProblem, WorkflowError> {
        let d = self
            .0
            .sources
            .fits
            .iter()
            .find(|d| d.fit_id == id)
            .cloned()
            .ok_or_else(|| contract("unknown fitting declaration"))?;
        profile
            .solver
            .controls
            .validate()
            .map_err(crate::math::MathRuntimeError::from)?;
        if profile.solver.controls.start != native::solve::StartPolicy::NoPriorStart
            || profile.solver.controls.reuse != native::solve::ReusePolicy::Fresh
        {
            return Err(contract(
                "fitting consumes declared parameter guesses; retained allocation and external seeds require a separate fitting-start contract",
            ));
        }
        if profile.solver.intent != SolveIntent::Optimize
            || !profile.rank_tolerance.is_finite()
            || profile.rank_tolerance <= 0.0
            || profile.rank_tolerance >= 1.0
            || profile.max_cells == 0
            || d.experiments.is_empty()
            || d.parameters.is_empty()
        {
            return Err(contract("fit intent, rank policy or inventories"));
        }
        if d.parameters
            .iter()
            .map(|p| p.symbol_id)
            .collect::<BTreeSet<_>>()
            .len()
            != d.parameters.len()
            || d.experiments
                .iter()
                .map(|e| e.experiment_id)
                .collect::<BTreeSet<_>>()
                .len()
                != d.experiments.len()
            || d.observations
                .iter()
                .map(|o| o.observation_id)
                .collect::<BTreeSet<_>>()
                .len()
                != d.observations.len()
        {
            return Err(contract(
                "duplicate fit parameter, experiment or observation binding",
            ));
        }
        let q = &self.0.physical.quantities;
        let mut targets = Vec::new();
        let mut declarations = Vec::new();
        let mut vars = Vec::new();
        let mut initial = Vec::new();
        let mut parameter_columns = Vec::new();
        let mut parameter_ports = Vec::new();
        for p in &d.parameters {
            let mut found = None;
            for e in &d.experiments {
                let inputs = self
                    .0
                    .cases
                    .get(&e.case_id)
                    .ok_or_else(|| contract("unknown fit case"))?;
                let source = &inputs.cases[&e.case_id].structure;
                if let Some(port) = source.parameters().iter().find(|v| v.id == p.symbol_id) {
                    if let Some(old) = &found {
                        if old != port {
                            return Err(contract("shared parameter quantity or unit mismatch"));
                        }
                    } else {
                        found = Some(port.clone());
                    }
                }
            }
            let port =
                found.ok_or_else(|| contract("fitted symbol must be an experiment parameter"))?;
            if !p.value.is_finite()
                || !p.scale.is_finite()
                || p.scale <= 0.0
                || p.lower.is_some_and(|v| !v.is_finite() || v > p.value)
                || p.upper.is_some_and(|v| !v.is_finite() || v < p.value)
                || (!p.fixed && p.lower.is_some() && p.lower == p.upper)
            {
                return Err(contract("parameter bounds, scale or fixed decision"));
            }
            targets.push(TargetSpec {
                id: p.symbol_id,
                kind: NumericalTarget::Variable,
                quantity: port.quantity,
                unit: port.unit,
                integer: false,
                declared_tolerance: None,
            });
            declarations.push(SourcedRequirement {
                source: NumericalSource::Model,
                declaration: NumericalRequirement {
                    requirement_id: pse_ids::named_id(
                        d.fit_id,
                        &format!("nominal.{}", p.symbol_id),
                    ),
                    model_id: d.model_id,
                    case_id: None,
                    target_id: p.symbol_id,
                    target_kind: NumericalTarget::Variable,
                    nominal: Some(p.scale),
                    scaling_factor: None,
                    absolute_tolerance: None,
                    relative_tolerance: None,
                    unit_id: Some(port.unit.as_id()),
                    coordinates: NumericalCoordinates::Physical,
                    priority: 0,
                    required: true,
                    provenance: "authored fitting parameter nominal".into(),
                },
            });
            parameter_columns.push(if p.fixed {
                None
            } else {
                let i = vars.len();
                vars.push(Variable {
                    id: p.symbol_id,
                    lower: p.lower.unwrap_or(f64::NEG_INFINITY),
                    upper: p.upper.unwrap_or(f64::INFINITY),
                });
                initial.push(p.value);
                Some(i)
            });
            parameter_ports.push(port);
        }
        let second = profile.solver.controls.hessian == HessianMode::Exact;
        if second && d.experiments.iter().any(|e| e.dynamic_id.is_some()) {
            return Err(contract(
                "transient fitting requires limited-memory Hessians",
            ));
        }
        let order = if second {
            DerivativeOrder::Second
        } else {
            DerivativeOrder::First
        };
        let mut experiments = Vec::new();
        let mut bounds = Vec::new();
        let mut rows = Vec::new();
        let mut measurements = Vec::new();
        let mut bytes = profile
            .solver
            .controls
            .report_allowance()
            .map_err(crate::math::MathRuntimeError::from)?
            .checked_add(256 * 1024)
            .ok_or_else(|| contract("fit report extent"))?;
        for (ei, e) in d.experiments.iter().enumerate() {
            let inputs = self
                .0
                .cases
                .get(&e.case_id)
                .ok_or_else(|| contract("unknown experiment case"))?;
            let source = &inputs.cases[&e.case_id].structure;
            if e.dynamic_id.is_none() {
                let mut local_targets = source
                    .variables()
                    .iter()
                    .filter(|v| !v.fixed)
                    .map(|v| TargetSpec {
                        id: v.port.id,
                        kind: NumericalTarget::Variable,
                        quantity: v.port.quantity,
                        unit: v.port.unit,
                        integer: false,
                        declared_tolerance: None,
                    })
                    .collect::<Vec<_>>();
                for row in source
                    .rows()
                    .iter()
                    .filter(|r| r.lower.is_finite() || r.upper.is_finite())
                {
                    local_targets.push(TargetSpec {
                        id: row.id,
                        kind: NumericalTarget::Row,
                        quantity: row.quantity,
                        unit: q.quantity_type(row.quantity).map_err(math)?.canonical_unit,
                        integer: false,
                        declared_tolerance: None,
                    });
                }
                for mut requirement in self.property_numerics(e.case_id, &local_targets)? {
                    requirement.declaration.target_id =
                        alias(e.experiment_id, requirement.declaration.target_id);
                    requirement.declaration.requirement_id =
                        alias(e.experiment_id, requirement.declaration.requirement_id);
                    declarations.push(requirement);
                }
            }

            if source.objective().is_some() {
                return Err(contract(
                    "fit experiment must expose physical constraints and outputs without an objective",
                ));
            }
            let outputs = source
                .rows()
                .iter()
                .filter(|r| {
                    r.lower.is_finite()
                        || r.upper.is_finite()
                        || d.observations.iter().any(|o| {
                            o.experiment_id == e.experiment_id && o.included && o.output_id == r.id
                        })
                })
                .map(|r| r.id)
                .collect::<Vec<_>>();
            let local_bindings = d
                .observations
                .iter()
                .filter(|o| o.experiment_id == e.experiment_id)
                .collect::<Vec<_>>();
            let mut bound_times = BTreeMap::new();
            let experiment = if let Some(dynamic) = e.dynamic_id {
                let mut integration = profile
                    .simulations
                    .get(&e.experiment_id)
                    .cloned()
                    .ok_or_else(|| contract("transient experiment profile missing"))?;
                let dynamic_source = self
                    .0
                    .sources
                    .dynamics
                    .iter()
                    .find(|d| d.dynamic_id == dynamic)
                    .ok_or_else(|| contract("unknown dynamic experiment"))?;
                for o in &local_bindings {
                    let raw = o
                        .time
                        .ok_or_else(|| contract("transient observation needs a time coordinate"))?;
                    let scale = if let Some(id) = o.time_unit_id {
                        let unit = q.unit(UnitId::from_id(id)).map_err(math)?;
                        if unit.is_affine
                            || unit.dimension
                                != pse_quantity::DimensionVector::base(
                                    pse_quantity::BaseDimension::Time,
                                )
                        {
                            return Err(contract(
                                "observation time requires a non-affine time unit",
                            ));
                        }
                        unit.scale_to_canonical
                    } else {
                        1.0
                    };
                    let time = super::time::observation(
                        raw,
                        scale,
                        o.time_basis.unwrap_or(
                            pse_relations::generated::enums::ObservationTimeBasis::Elapsed,
                        ),
                        dynamic_source.time_origin.unwrap_or(0.0),
                        integration.start,
                    )?;
                    bound_times.insert(o.observation_id, time);
                }
                let mut times: Vec<_> = local_bindings
                    .iter()
                    .filter(|o| o.included)
                    .map(|o| bound_times[&o.observation_id])
                    .collect();
                times.sort_by(f64::total_cmp);
                times.dedup();
                if !times.is_empty() {
                    integration.samples = times;
                }
                integration.sensitivities = !dynamic_source.parameters.is_empty();
                if d.parameters.iter().any(|p| {
                    source.parameters().iter().any(|s| s.id == p.symbol_id)
                        && !dynamic_source.parameters.contains(&p.symbol_id)
                }) {
                    return Err(contract(
                        "fitted dynamic parameters must be selected sensitivity coordinates",
                    ));
                }
                let simulation = self
                    .prepare_simulation(dynamic, integration, compiler, cancel)
                    .await?;
                if simulation.declaration.case_id != e.case_id {
                    return Err(contract("experiment/dynamic case mismatch"));
                }
                bytes = bytes
                    .checked_add(simulation.bytes)
                    .ok_or_else(|| contract("fit storage extent"))?;
                Experiment::Transient(Box::new(simulation))
            } else {
                if profile.simulations.contains_key(&e.experiment_id) {
                    return Err(contract("steady experiment has an integration profile"));
                }
                let mut coordinates = Vec::new();
                let mut values = CaseValues {
                    scalars: inputs.values.clone(),
                };
                for v in source.variables().iter().filter(|v| !v.fixed) {
                    if v.domain != pse_math::binding::VariableDomain::Continuous {
                        return Err(contract("fitting requires continuous state variables"));
                    }
                    targets.push(TargetSpec {
                        id: alias(e.experiment_id, v.port.id),
                        kind: NumericalTarget::Variable,
                        quantity: v.port.quantity,
                        unit: v.port.unit,
                        integer: false,
                        declared_tolerance: None,
                    });
                    let col = vars.len();
                    vars.push(Variable {
                        id: alias(e.experiment_id, v.port.id),
                        lower: v.lower.unwrap_or(f64::NEG_INFINITY),
                        upper: v.upper.unwrap_or(f64::INFINITY),
                    });
                    initial.push(
                        *values
                            .scalars
                            .get(&v.port.id)
                            .ok_or_else(|| contract("missing initial state"))?,
                    );
                    coordinates.push((v.port.id, col));
                }
                let local_states = coordinates.len();
                for (p, col) in d.parameters.iter().zip(&parameter_columns) {
                    if source.parameters().iter().any(|v| v.id == p.symbol_id) {
                        values.scalars.insert(p.symbol_id, p.value);
                        if let Some(c) = col {
                            coordinates.push((p.symbol_id, *c));
                        }
                    }
                }
                let case = self
                    .0
                    .runtime
                    .native()
                    .prepare_functions_revision(
                        self.0.workspace.clone(),
                        inputs.as_ref().clone(),
                        e.case_id,
                        outputs.clone(),
                        coordinates.iter().map(|v| v.0).collect(),
                        order,
                        compiler,
                        cancel,
                    )
                    .await?;
                let mut constraints = Vec::new();
                for (i, r) in case.assembly.structure().rows().iter().enumerate() {
                    if r.lower.is_finite() || r.upper.is_finite() {
                        targets.push(TargetSpec {
                            id: alias(e.experiment_id, r.id),
                            kind: NumericalTarget::Row,
                            quantity: r.quantity,
                            unit: q
                                .quantity_type(r.quantity)
                                .map_err(|e| contract(e.to_string()))?
                                .canonical_unit,
                            integer: false,
                            declared_tolerance: None,
                        });
                        constraints.push((i, rows.len()));
                        rows.push(alias(e.experiment_id, r.id));
                        bounds.push((r.lower, r.upper));
                    }
                }
                bytes = bytes
                    .checked_add(case.assembly.numeric_worker_bytes())
                    .ok_or_else(|| contract("fit worker extent"))?;
                Experiment::Steady(Steady {
                    case,
                    values,
                    coordinates,
                    constraints,
                    local_states,
                })
            };
            for binding in local_bindings {
                let observation = self
                    .0
                    .sources
                    .observations
                    .iter()
                    .find(|o| o.observation_id == binding.observation_id)
                    .ok_or_else(|| contract("missing authored observation"))?;
                let (row, port) = match &experiment {
                    Experiment::Steady(s) => {
                        if binding.time.is_some()
                            || binding.time_basis.is_some()
                            || binding.time_unit_id.is_some()
                        {
                            return Err(contract("steady observation has elapsed time"));
                        }
                        let r = source
                            .rows()
                            .iter()
                            .find(|r| r.id == binding.output_id)
                            .ok_or_else(|| contract("unknown observed row"))?;
                        let i = s
                            .case
                            .assembly
                            .structure()
                            .rows()
                            .iter()
                            .position(|r| r.id == binding.output_id)
                            .unwrap_or(usize::MAX);
                        (
                            i,
                            Port {
                                id: r.id,
                                quantity: r.quantity,
                                unit: q.quantity_type(r.quantity).map_err(math)?.canonical_unit,
                            },
                        )
                    }
                    Experiment::Transient(s) => {
                        let i = s
                            .output_ports
                            .iter()
                            .position(|p| p.id == binding.output_id)
                            .ok_or_else(|| contract("unknown transient observed output"))?;
                        (i, s.output_ports[i].clone())
                    }
                };
                let ty = q.quantity_type(port.quantity).map_err(math)?;
                let conversion = pse_quantity::convert_spec_for_type(
                    q.unit(UnitId::from_id(observation.unit_id)).map_err(math)?,
                    q.unit(port.unit).map_err(math)?,
                    &ty.key,
                )
                .map_err(math)?;
                let value = observation
                    .value
                    .map(|v| v * conversion.scale + conversion.offset);
                let sigma = observation.std_dev.map(|s| s * conversion.scale.abs());
                if !binding.importance.is_finite()
                    || binding.importance <= 0.0
                    || value.is_some_and(|v| !v.is_finite())
                    || sigma.is_some_and(|s| !s.is_finite() || s <= 0.0)
                    || sigma.is_some_and(|s| !(binding.importance.sqrt() / s).is_finite())
                    || (binding.included && (value.is_none() || sigma.is_none()))
                {
                    return Err(contract(
                        "included observations require finite values, positive difference-unit standard deviations and importance",
                    ));
                }
                measurements.push(Measurement {
                    id: binding.observation_id,
                    experiment: ei,
                    row,
                    time: bound_times.get(&binding.observation_id).copied(),
                    sample_index: match &experiment {
                        Experiment::Transient(s) if binding.included => Some(
                            s.profile
                                .samples
                                .binary_search_by(|t| {
                                    t.total_cmp(&bound_times[&binding.observation_id])
                                })
                                .map_err(|_| contract("unbound observation sample"))?,
                        ),
                        _ => None,
                    },
                    included: binding.included,
                    value,
                    sigma,
                    importance: binding.importance,
                    port,
                });
            }
            experiments.push(experiment);
        }
        if measurements.len() != d.observations.len()
            || !measurements.iter().any(|o| o.included)
            || profile.simulations.keys().any(|id| {
                !d.experiments
                    .iter()
                    .any(|e| e.experiment_id == *id && e.dynamic_id.is_some())
            })
        {
            return Err(contract(
                "fit observations or integration profile ownership",
            ));
        }
        let physical_cells =
            experiments
                .iter()
                .enumerate()
                .try_fold(0usize, |count, (i, experiment)| {
                    let samples = match experiment {
                        Experiment::Steady(_) => 1,
                        Experiment::Transient(s) => s.profile.samples.len(),
                    };
                    let balances = self
                        .0
                        .resolved_sources
                        .balances
                        .iter()
                        .filter(|b| b.case_id == d.experiments[i].case_id)
                        .count();
                    count
                        .checked_add(
                            samples
                                .checked_mul(balances)
                                .ok_or_else(|| contract("fit physical check extent"))?,
                        )
                        .ok_or_else(|| contract("fit physical check extent"))
                })?;
        let cells = vars
            .len()
            .checked_mul(vars.len())
            .and_then(|v| v.checked_add(vars.len().checked_mul(measurements.len() + rows.len())?))
            .and_then(|v| v.checked_add(physical_cells))
            .ok_or_else(|| contract("fit dense diagnostic extent"))?;
        if cells > profile.max_cells {
            return Err(contract("fit derivative/diagnostic cell allowance"));
        }
        bytes = bytes
            .checked_add(
                physical_cells
                    .checked_mul(512)
                    .ok_or_else(|| contract("fit physical report storage"))?,
            )
            .ok_or_else(|| contract("fit physical report storage"))?
            .checked_add(
                cells
                    .checked_mul(64)
                    .ok_or_else(|| contract("fit derivative storage"))?,
            )
            .ok_or_else(|| contract("fit storage"))?;
        let neutral = q
            .neutral_dimensionless()
            .ok_or_else(|| contract("fitting requires the bound neutral quantity"))?;
        targets.push(TargetSpec {
            id: SemanticId::NIL,
            kind: NumericalTarget::Objective,
            quantity: neutral,
            unit: q
                .quantity_type(neutral)
                .map_err(|e| contract(e.to_string()))?
                .canonical_unit,
            integer: false,
            declared_tolerance: None,
        });
        declarations.extend(
            self.0
                .resolved_sources
                .numerics
                .iter()
                .filter(|r| {
                    r.target_kind == NumericalTarget::Variable
                        && d.parameters.iter().any(|p| p.symbol_id == r.target_id)
                        && r.case_id
                            .is_none_or(|id| d.experiments.iter().any(|e| e.case_id == id))
                })
                .cloned()
                .map(|declaration| SourcedRequirement {
                    source: if declaration.case_id.is_some() {
                        NumericalSource::Case
                    } else {
                        NumericalSource::Model
                    },
                    declaration,
                }),
        );
        for experiment in &d.experiments {
            for source in self
                .0
                .resolved_sources
                .numerics
                .iter()
                .filter(|r| r.case_id.is_none_or(|c| c == experiment.case_id))
            {
                if !matches!(
                    source.target_kind,
                    NumericalTarget::Variable | NumericalTarget::Row
                ) {
                    continue;
                }
                let mut declaration = source.clone();
                let target = alias(experiment.experiment_id, source.target_id);
                if !targets
                    .iter()
                    .any(|t| t.id == target && t.kind == source.target_kind)
                {
                    continue;
                }
                declaration.target_id = target;
                declaration.requirement_id = alias(experiment.experiment_id, source.requirement_id);
                declarations.push(SourcedRequirement {
                    source: if source.case_id.is_some() {
                        NumericalSource::Case
                    } else {
                        NumericalSource::Model
                    },
                    declaration,
                });
            }
        }
        for b in self
            .0
            .resolved_sources
            .balances
            .iter()
            .filter(|b| d.experiments.iter().any(|e| e.case_id == b.case_id))
        {
            let quantity = if let Some(state) = b.accumulation {
                self.0.cases[&b.case_id].cases[&b.case_id]
                    .structure
                    .variables()
                    .iter()
                    .find(|v| v.port.id == state)
                    .ok_or_else(|| contract("fit conserved state missing"))?
                    .port
                    .quantity
            } else {
                b.quantity_id.into()
            };
            targets.push(TargetSpec {
                id: b.balance_id,
                kind: NumericalTarget::Closure,
                quantity,
                unit: q.quantity_type(quantity).map_err(math)?.canonical_unit,
                integer: false,
                declared_tolerance: if b.accumulation.is_some() {
                    b.integral_tolerance
                } else {
                    Some(b.tolerance)
                },
            });
        }
        for r in self.0.resolved_sources.numerics.iter().filter(|r| {
            r.target_kind == NumericalTarget::Closure
                && r.case_id
                    .is_none_or(|id| d.experiments.iter().any(|e| e.case_id == id))
        }) {
            declarations.push(SourcedRequirement {
                source: if r.case_id.is_some() {
                    NumericalSource::Case
                } else {
                    NumericalSource::Model
                },
                declaration: r.clone(),
            });
        }
        let numerics = Arc::new(
            pse_math::numerics::resolve(q, &targets, &declarations, &profile.solver.numerics)
                .map_err(math)?,
        );
        let columns: Vec<_> = vars.iter().map(|v| v.id).collect();
        let normalization = Normalization::from_policy(&numerics, &columns, &rows).map_err(math)?;
        let tolerances = native::quality::Tolerances::from_policy(&numerics, &columns, &rows)
            .map_err(crate::math::MathRuntimeError::from)?;
        if profile.solver.controls.accuracy != native::solve::Accuracy::default() {
            return Err(contract(
                "fitting accuracy is owned by the numerical policy",
            ));
        }
        profile.solver.controls.accuracy =
            native::solve::Accuracy::resolve(&numerics.policy, &tolerances, &normalization)
                .map_err(crate::math::MathRuntimeError::from)?;
        if vars.is_empty()
            && matches!(&profile.solver.presolve, native::presolve::Policy::Explicit { required, .. } if !required.is_empty())
        {
            return Err(contract(
                "all-fixed fitting evaluates directly and cannot apply required native presolve passes",
            ));
        }
        let mut h = FramedHasher::new("pse.fit.source.v1");
        h.hash(&self.identity());
        d.frame(&mut h);
        let source = h.finish_hash();
        let mut h = FramedHasher::new("pse.fit.profile.v1");
        h.hash(
            &crate::math::solves::profile_key(&profile.solver)
                .map_err(crate::math::MathRuntimeError::from)?,
        );
        h.u64(profile.rank_tolerance.to_bits())
            .u64(profile.max_cells as u64);
        for (id, p) in &profile.simulations {
            h.id(id).hash(&super::dynamics::profile_identity(p));
        }
        let profile_key = h.finish_hash();
        let mut h = FramedHasher::new("pse.fit.prepared.v1");
        h.hash(&source).hash(&profile_key).hash(&numerics.key);
        let contract = OracleContract {
            identity: source,
            variables: vars,
            rows,
            derivatives: order,
            smoothness: order,
        };
        Ok(FitProblem {
            revision: self.clone(),
            declaration: d,
            profile,
            key: h.finish_hash(),
            profile_key,
            numerics,
            normalization,
            tolerances,
            bytes,
            contract,
            bounds,
            initial,
            parameter_ports,
            parameter_columns,
            experiments,
            measurements,
        })
    }
}
