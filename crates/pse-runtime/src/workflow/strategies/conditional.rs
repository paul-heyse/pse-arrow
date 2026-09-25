// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Revision-bound initialization and explicitly declared compiled causal unit maps.
use super::*;
use crate::math::{
    ExecutableCase,
    initialization::{
        DeclaredRootReport, InitializationProfile, InitializationReport, PreparedInitialization,
    },
};
use crate::workflow::ModelRevision;
use native::{
    kinsol,
    quality::Tolerances,
    recycle::{CausalMap, CausalUnit},
};
use pse_math::binding::CaseValues;
use std::collections::{BTreeMap, BTreeSet};

/// Immutable original bindings plus a finite declared initialization strategy.
#[derive(Clone, Debug)]
pub struct PreparedInitializationStrategy {
    pub(crate) revision: ModelRevision,
    pub(crate) case: SemanticId,
    pub(crate) prepared: PreparedInitialization,
    pub(crate) profile: InitializationProfile,
}
impl PreparedInitializationStrategy {
    /// Predecessor-ordered conditional boundaries, inspected before execution.
    pub fn blocks(&self) -> &PreparedInitialization {
        &self.prepared
    }
    /// Chosen routes, without an implicit failure fallback.
    pub fn strategies(&self) -> Result<Vec<native::routing::Route>, WorkflowError> {
        self.prepared
            .strategies(&self.profile.controls, self.profile.selection)
            .map_err(|e| MathRuntimeError::from(e).into())
    }
    /// Execute with this revision's original values and provider bindings.
    pub fn start(&self) -> Result<SolveHandle<InitializationReport>, WorkflowError> {
        let values = CaseValues {
            scalars: self.revision.0.cases[&self.case].values.clone(),
        };
        let providers = self
            .revision
            .0
            .providers
            .values()
            .map(|p| (p.registration.spec().key(), p.registration.clone()))
            .collect();
        Ok(self.revision.0.runtime.native().initialize(
            self.prepared.clone(),
            values,
            providers,
            self.profile.clone(),
        )?)
    }
}

/// Explicit causal direction for one unit; outputs name authored function rows, never residual-to-map inference.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CausalUnitRequest {
    /// Owning selected flow node.
    pub node: SemanticId,
    /// Selected function case in this revision.
    pub case: SemanticId,
    /// Flow input port to authored function coordinate.
    pub inputs: BTreeMap<SemanticId, SemanticId>,
    /// Flow output port to authored function row.
    pub outputs: BTreeMap<SemanticId, SemanticId>,
}
/// Concrete tear witness and causal directions; native KINSOL owns all recycle iteration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecycleRequest {
    /// The selected case supplying the flow projection.
    pub case: SemanticId,
    /// Selected flow identity.
    pub flow: SemanticId,
    /// Exact selected tear decision groups, obtainable through select_tears.
    pub tears: BTreeSet<SemanticId>,
    /// Complete explicit causal unit inventory.
    pub units: Vec<CausalUnitRequest>,
    /// Native Anderson history; zero means unaccelerated fixed point.
    pub anderson: usize,
    /// Native fixed-point damping in (0,1].
    pub damping: f64,
}
#[derive(Clone, Debug)]
struct UnitProgram {
    declaration: CausalUnitRequest,
    program: Arc<ExecutableCase>,
    values: CaseValues,
    inputs: Vec<(SemanticId, SemanticId, pse_quantity::UnitConvertSpec)>,
    outputs: Vec<(SemanticId, usize, pse_quantity::UnitConvertSpec)>,
}
#[derive(Debug)]
struct UnitWorker {
    program: UnitProgram,
    worker: pse_math::assembly::CaseWorker,
    input_ids: Vec<SemanticId>,
    output_ids: Vec<SemanticId>,
}
impl CausalUnit for UnitWorker {
    fn id(&self) -> SemanticId {
        self.program.declaration.node
    }
    fn inputs(&self) -> &[SemanticId] {
        &self.input_ids
    }
    fn outputs(&self) -> &[SemanticId] {
        &self.output_ids
    }
    fn evaluate(
        &mut self,
        inputs: &BTreeMap<SemanticId, f64>,
        execution: &Execution,
    ) -> Result<BTreeMap<SemanticId, f64>, native::ProblemError> {
        if execution.stopped().is_some() {
            return Err(pse_math::MathError::Cancelled.into());
        }
        for (port, symbol, conversion) in &self.program.inputs {
            let value = inputs
                .get(port)
                .ok_or_else(|| native::ProblemError::Contract("missing causal input".into()))?;
            self.program
                .values
                .scalars
                .insert(*symbol, value * conversion.scale + conversion.offset);
        }
        let values = self.worker.constraints(&self.program.values)?;
        Ok(self
            .program
            .outputs
            .iter()
            .map(|(port, row, c)| (*port, values[*row] * c.scale + c.offset))
            .collect())
    }
}
/// Compiled immutable causal functions and the independently checked selected tear graph.
#[derive(Clone, Debug)]
pub struct PreparedRecycle {
    revision: ModelRevision,
    request: RecycleRequest,
    graph: Arc<pse_structural::flowsheet::FlowGraph>,
    programs: Vec<UnitProgram>,
    contract: native::OracleContract,
    initial: Vec<f64>,
    fixed: BTreeMap<SemanticId, f64>,
    settings: kinsol::Settings,
    controls: Controls,
    tolerances: Tolerances,
}
impl PreparedRecycle {
    /// Exact causal directions and tear decisions selected by the caller.
    pub fn request(&self) -> &RecycleRequest {
        &self.request
    }
    /// Independent prerequisite-first acyclic witness.
    pub fn order(&self) -> Result<Vec<SemanticId>, WorkflowError> {
        self.graph
            .witness(&self.request.tears)
            .map_err(|e| contract(e.to_string()))
    }
    /// Execute only the explicitly requested map strategy; no automatic fallback follows failure.
    pub fn start(&self) -> Result<SolveHandle<DeclaredRootReport>, WorkflowError> {
        let prepared = self.clone();
        Ok(self.revision.0.runtime.native().solve_declared_root(
            self.contract.clone(),
            self.initial.clone(),
            self.settings.clone(),
            self.controls.clone(),
            self.tolerances.clone(),
            move |execution| {
                let mut units: BTreeMap<SemanticId, Box<dyn CausalUnit>> = BTreeMap::new();
                for program in prepared.programs {
                    let providers = prepared
                        .revision
                        .0
                        .providers
                        .values()
                        .map(|p| {
                            p.registration
                                .worker()
                                .map(|w| (p.registration.spec().key(), w))
                        })
                        .collect::<Result<_, _>>()
                        .map_err(|e| native::ProblemError::Contract(e.to_string()))?;
                    let worker = program
                        .program
                        .assembly
                        .worker(providers, execution.cancel.clone());
                    let unit = UnitWorker {
                        input_ids: program.inputs.iter().map(|x| x.0).collect(),
                        output_ids: program.outputs.iter().map(|x| x.0).collect(),
                        program,
                        worker,
                    };
                    units.insert(unit.id(), Box::new(unit));
                }
                Ok(kinsol::Function::FixedPoint(Box::new(CausalMap::new(
                    prepared.graph,
                    prepared.request.tears,
                    prepared.contract,
                    units,
                    prepared.fixed,
                    execution,
                )?)))
            },
        )?)
    }
}
impl ModelRevision {
    /// Compile explicitly directed units from this revision, retaining physical port conversions.
    pub async fn prepare_recycle(
        &self,
        request: RecycleRequest,
        mut profile: SolverProfile,
        compiler: pse_compiler::workspace::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedRecycle, WorkflowError> {
        if profile.intent != SolveIntent::Root
            || !matches!(
                profile.selection,
                SolverSelection::Auto | SolverSelection::Explicit(Backend::Kinsol)
            )
            || profile.controls.start != StartPolicy::NoPriorStart
            || profile.controls.threads != 1
        {
            return Err(contract(
                "causal map requires serial KINSOL root intent and original declared guesses",
            ));
        }
        if !matches!(
            profile.backend,
            crate::math::solves::BackendSettings::Default
        ) || profile.controls.accuracy != Accuracy::default()
        {
            return Err(contract(
                "causal map settings derive from its numerical policy and declared map controls",
            ));
        }
        profile
            .controls
            .validate()
            .map_err(MathRuntimeError::from)?;
        let flow = self.prepare_flow(request.case, request.flow).await?;
        let graph = Arc::new(flow.graph().clone());
        graph
            .witness(&request.tears)
            .map_err(|e| contract(e.to_string()))?;
        let q = &self.0.physical.quantities;
        let ports: BTreeMap<_, _> = graph
            .declaration()
            .nodes
            .iter()
            .flat_map(|n| n.ports.iter().map(|p| (p.id, p)))
            .collect();
        let mut programs = Vec::new();
        let mut all_inputs = BTreeMap::new();
        let mut all_outputs = BTreeSet::new();
        for unit in &request.units {
            let node = graph
                .declaration()
                .nodes
                .iter()
                .find(|n| n.id == unit.node)
                .ok_or_else(|| contract("unknown causal node"))?;
            let inventory: BTreeSet<_> = unit
                .inputs
                .keys()
                .chain(unit.outputs.keys())
                .copied()
                .collect();
            if inventory != node.ports.iter().map(|p| p.id).collect()
                || inventory.len() != unit.inputs.len() + unit.outputs.len()
                || programs
                    .iter()
                    .any(|p: &UnitProgram| p.declaration.node == unit.node)
            {
                return Err(contract(
                    "causal direction must cover every port once and every node once",
                ));
            }
            let input = self
                .0
                .cases
                .get(&unit.case)
                .ok_or_else(|| contract("unknown causal function case"))?;
            let source = &input.cases[&unit.case].structure;
            let symbols: BTreeMap<_, _> = source
                .variables()
                .iter()
                .map(|v| (v.port.id, &v.port))
                .chain(source.parameters().iter().map(|p| (p.id, p)))
                .collect();
            let mut inputs = Vec::new();
            for (port, symbol) in &unit.inputs {
                let bound = symbols
                    .get(symbol)
                    .ok_or_else(|| contract("unknown causal input symbol"))?;
                if source
                    .variables()
                    .iter()
                    .any(|v| v.port.id == *symbol && (v.lower.is_some() || v.upper.is_some()))
                {
                    return Err(contract(
                        "KINSOL fixed point cannot enforce causal input bounds; request a constrained simultaneous strategy",
                    ));
                }
                pse_quantity::admission::require_same_contract(
                    ports[port].quantity,
                    bound.quantity,
                    q,
                )
                .map_err(crate::workflow::math)?;
                let conversion = pse_quantity::convert_spec_for_type(
                    q.unit(ports[port].unit).map_err(crate::workflow::math)?,
                    q.unit(bound.unit).map_err(crate::workflow::math)?,
                    &q.quantity_type(bound.quantity)
                        .map_err(crate::workflow::math)?
                        .key,
                )
                .map_err(crate::workflow::math)?;
                inputs.push((*port, *symbol, conversion));
                if all_inputs
                    .insert(
                        *port,
                        (input.values[symbol] - conversion.offset) / conversion.scale,
                    )
                    .is_some()
                {
                    return Err(contract("duplicate causal input port"));
                }
            }
            let outputs: Vec<_> = unit.outputs.values().copied().collect();
            let coordinates = unit.inputs.values().copied().collect();
            let program = self
                .0
                .runtime
                .native()
                .prepare_functions_revision(
                    self.0.workspace.clone(),
                    input.as_ref().clone(),
                    unit.case,
                    outputs,
                    coordinates,
                    DerivativeOrder::First,
                    compiler,
                    cancel,
                )
                .await?;
            // Every free dependency of each explicit function must be a declared input.
            let declared: BTreeSet<_> = unit.inputs.values().copied().collect();
            let free: BTreeSet<_> = source
                .variables()
                .iter()
                .filter(|v| !v.fixed)
                .map(|v| v.port.id)
                .collect();
            if program
                .assembly
                .structure()
                .instances()
                .iter()
                .flat_map(|i| &i.slots)
                .any(|s| free.contains(&s.source()) && !declared.contains(&s.source()))
            {
                return Err(contract(
                    "causal function depends on an undeclared free input",
                ));
            }
            let mut mapped = Vec::new();
            for (port, row) in &unit.outputs {
                let (index, source_row) = program
                    .assembly
                    .structure()
                    .rows()
                    .iter()
                    .enumerate()
                    .find(|(_, r)| r.id == *row)
                    .ok_or_else(|| contract("unknown causal output row"))?;
                pse_quantity::admission::require_same_contract(
                    source_row.quantity,
                    ports[port].quantity,
                    q,
                )
                .map_err(crate::workflow::math)?;
                let canonical = q
                    .quantity_type(source_row.quantity)
                    .map_err(crate::workflow::math)?
                    .canonical_unit;
                let conversion = pse_quantity::convert_spec_for_type(
                    q.unit(canonical).map_err(crate::workflow::math)?,
                    q.unit(ports[port].unit).map_err(crate::workflow::math)?,
                    &q.quantity_type(source_row.quantity)
                        .map_err(crate::workflow::math)?
                        .key,
                )
                .map_err(crate::workflow::math)?;
                mapped.push((*port, index, conversion));
                all_outputs.insert(*port);
            }
            programs.push(UnitProgram {
                declaration: unit.clone(),
                program,
                values: CaseValues {
                    scalars: input.values.clone(),
                },
                inputs,
                outputs: mapped,
            });
        }
        if programs.len() != graph.declaration().nodes.len() {
            return Err(contract("incomplete causal unit inventory"));
        }
        let mut connected = BTreeSet::new();
        let mut tears = BTreeSet::new();
        for edge in &graph.declaration().connections {
            for binding in &graph.bindings()[&edge.id] {
                if !all_outputs.contains(&binding.source)
                    || !all_inputs.contains_key(&binding.target)
                {
                    return Err(contract(
                        "flow must connect an explicit output to an explicit input",
                    ));
                }
                connected.insert(binding.target);
                if request.tears.contains(&edge.decision) {
                    tears.insert(binding.target);
                }
            }
        }
        let fixed = all_inputs
            .iter()
            .filter(|(p, _)| !connected.contains(p))
            .map(|(p, v)| (*p, *v))
            .collect();
        let initial = tears.iter().map(|p| all_inputs[p]).collect();
        let mut h = FramedHasher::new("pse.causal-map.v1");
        h.hash(&self.identity())
            .hash(&graph.key())
            .str(&serde_json::to_string(&request).map_err(|e| contract(e.to_string()))?);
        let contract = native::OracleContract {
            identity: h.finish_hash(),
            variables: tears
                .iter()
                .map(|p| native::Variable {
                    id: *p,
                    lower: f64::NEG_INFINITY,
                    upper: f64::INFINITY,
                })
                .collect(),
            rows: tears
                .iter()
                .map(|p| pse_ids::named_id(*p, "connection-residual"))
                .collect(),
            derivatives: DerivativeOrder::Value,
            smoothness: DerivativeOrder::Value,
        };
        let targets: Vec<_> = tears
            .iter()
            .zip(&contract.rows)
            .flat_map(|(p, r)| {
                [
                    (pse_model::generated::enums::NumericalTarget::Variable, *p),
                    (pse_model::generated::enums::NumericalTarget::Row, *r),
                ]
                .map(|(kind, id)| pse_math::numerics::TargetSpec {
                    id,
                    kind,
                    quantity: ports[p].quantity,
                    unit: ports[p].unit,
                    integer: false,
                    declared_tolerance: None,
                })
            })
            .collect();
        let numerics = pse_math::numerics::resolve(q, &targets, &[], &profile.numerics)
            .map_err(crate::workflow::math)?;
        let ids: Vec<_> = tears.into_iter().collect();
        let tolerances = Tolerances::from_policy(&numerics, &ids, &contract.rows)
            .map_err(MathRuntimeError::from)?;
        let identity = pse_math::normalization::Normalization {
            variables: vec![1.0; ids.len()],
            rows: vec![1.0; ids.len()],
            objective: 1.0,
        };
        profile.controls.accuracy = Accuracy::resolve(&numerics.policy, &tolerances, &identity)
            .map_err(MathRuntimeError::from)?;
        let scales = |t: &[f64]| {
            t.iter()
                .map(|t| profile.controls.accuracy.feasibility / t)
                .collect()
        };
        let settings = kinsol::Settings {
            strategy: kinsol::Strategy::FixedPoint,
            linear: kinsol::Linear::Klu,
            variable_scales: scales(&tolerances.variables),
            residual_scales: scales(&tolerances.rows),
            anderson: request.anderson,
            damping: request.damping,
            setup_interval: 10,
            step_tolerance: profile.controls.accuracy.feasibility,
        };
        settings
            .validate_contract(&contract, kinsol::Strategy::FixedPoint, &BTreeMap::new())
            .map_err(MathRuntimeError::from)?;
        Ok(PreparedRecycle {
            revision: self.clone(),
            request,
            graph,
            programs,
            contract,
            initial,
            fixed,
            settings,
            controls: profile.controls,
            tolerances,
        })
    }
}
