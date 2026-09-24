// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared-parameter experiment compilation. Trial values never enter Salsa queries.
mod oracle;
mod results;
use super::{FitDeclaration, ModelRevision, PreparedSimulation, WorkflowError, contract, math};
use crate::math::{ExecutableCase, solves::SolverProfile};
use pse_backend_native::{
    self as native, OracleContract, Variable,
    solve::{Backend, HessianMode, SolveIntent, SolverSelection},
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, Port};
use pse_math::binding::CaseValues;
use pse_model::SemanticFrame;
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
    Transient(PreparedSimulation),
}
/// Immutable fitting product; mutable evaluators and native sessions are attempt-owned.
#[derive(Clone, Debug)]
pub(crate) struct FitProblem {
    pub(crate) revision: ModelRevision,
    pub(crate) declaration: FitDeclaration,
    pub(crate) profile: FitProfile,
    pub(crate) key: ContentHash,
    pub(crate) profile_key: ContentHash,
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
        let route = if problem.contract.variables.is_empty() {
            native::routing::Route::Constant
        } else {
            let backend = match problem.profile.solver.selection {
                SolverSelection::Auto if cfg!(feature = "solver-ipopt") => Backend::Ipopt,
                SolverSelection::Auto => Backend::Pounce,
                SolverSelection::Explicit(b @ (Backend::Ipopt | Backend::Pounce)) => b,
                _ => return Err(contract("fitting requires a native continuous NLP backend")),
            };
            if !backend.available() {
                return Err(contract("selected fitting adapter is not linked"));
            }
            native::routing::Route::Native(backend)
        };
        crate::math::solves::admit_profile(&problem.profile.solver, route)
            .map_err(crate::math::MathRuntimeError::from)?;
        Ok(PreparedFit { problem, route })
    }

    async fn prepare_fit_problem(
        &self,
        id: SemanticId,
        profile: FitProfile,
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
            let experiment = if let Some(dynamic) = e.dynamic_id {
                let mut integration = profile
                    .simulations
                    .get(&e.experiment_id)
                    .cloned()
                    .ok_or_else(|| contract("transient experiment profile missing"))?;
                let mut times = local_bindings
                    .iter()
                    .filter(|o| o.included)
                    .map(|o| {
                        o.time
                            .ok_or_else(|| contract("transient observation needs elapsed seconds"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if times.iter().any(|t| !t.is_finite()) {
                    return Err(contract("nonfinite observation time"));
                }
                times.sort_by(f64::total_cmp);
                times.dedup();
                if !times.is_empty() {
                    integration.samples = times;
                }
                let dynamic_source = self
                    .0
                    .sources
                    .dynamics
                    .iter()
                    .find(|d| d.dynamic_id == dynamic)
                    .ok_or_else(|| contract("unknown dynamic experiment"))?;
                integration.sensitivities = !dynamic_source.parameters.is_empty();
                if d.parameters.iter().any(|p| {
                    source.parameters().iter().any(|s| s.id == p.symbol_id)
                        && !dynamic_source.parameters.contains(&p.symbol_id)
                }) {
                    return Err(contract(
                        "fitted dynamic parameters must be selected sensitivity coordinates",
                    ));
                }
                if !integration.changes.is_empty()
                    || dynamic_source.modes.iter().any(|m| !m.events.is_empty())
                {
                    return Err(contract("fitting admits smooth transient experiments only"));
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
                Experiment::Transient(simulation)
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
                        if binding.time.is_some() {
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
                    time: binding.time,
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
                        .sources
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
        profile
            .solver
            .tolerances
            .validate(vars.len(), rows.len())
            .map_err(crate::math::MathRuntimeError::from)?;
        if let Some(scales) = &profile.solver.scaling {
            scales
                .validate(vars.len(), rows.len())
                .map_err(crate::math::MathRuntimeError::from)?;
        }
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
        h.hash(&source).hash(&profile_key);
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
