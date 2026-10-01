// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Revision-bound initialization and admitted explicit or conditional causal units.
use super::*;
use crate::math::{
    ExecutableCase,
    initialization::{
        DeclaredRootReport, InitializationProfile, InitializationReport, PreparedInitialization,
    },
};
use crate::workflow::{ModelingAnalysis, ModelingPackage};
use native::{
    kinsol,
    quality::Tolerances,
    recycle::{CausalMap, CausalUnit},
};
use pse_compiler::workspace::{ModelingFlowSelection, ModelingOutput};
use pse_math::binding::CaseValues;
use std::collections::{BTreeMap, BTreeSet};

/// Immutable original bindings plus a finite declared initialization strategy.
#[derive(Clone, Debug)]
pub struct PreparedInitializationStrategy {
    pub(crate) runtime: Runtime,
    pub(crate) values: CaseValues,
    pub(crate) providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
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
            .strategies(&self.profile.solver.controls, self.profile.solver.selection)
            .map_err(|e| MathRuntimeError::from(e).into())
    }
    /// Execute with this revision's original values and provider bindings.
    pub fn start(&self) -> Result<SolveHandle<InitializationReport>, WorkflowError> {
        Ok(self.runtime.native().initialize(
            self.prepared.clone(),
            self.values.clone(),
            self.providers.clone(),
            self.profile.clone(),
        )?)
    }
}

/// One declared mathematical realization of a causal unit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CausalUnitRealization {
    /// Authored output functions with every free dependency declared at the boundary.
    ExplicitMap,
    /// Original owned residuals, local solved coordinates and one declared root procedure.
    Conditional {
        /// Complete selected original unit equality identities.
        residuals: BTreeSet<SemanticId>,
        /// Remaining local free coordinates after boundary inputs are fixed.
        unknowns: BTreeSet<SemanticId>,
        /// The canonical solver settings document; defaults resolve in its existing owner.
        solver: Box<crate::math::settings::SolveSettings>,
    },
}
/// Explicit causal direction and admitted mathematical realization for one unit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CausalUnitRequest {
    /// Owning selected flow node.
    pub node: SemanticId,
    /// Input ports; their coordinates are owned by the authored port declarations.
    pub inputs: BTreeSet<SemanticId>,
    /// Output ports; their expressions are owned by the authored port declarations.
    pub outputs: BTreeSet<SemanticId>,
    /// A unit equation solve is declared independently of an explicit function map.
    pub realization: CausalUnitRealization,
}
/// Concrete tear witness and causal directions; native KINSOL owns all recycle iteration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecycleRequest {
    /// Exact selected tear decision groups, obtainable through select_tears.
    pub tears: BTreeSet<SemanticId>,
    /// Complete admitted causal unit inventory.
    pub units: Vec<CausalUnitRequest>,
    /// Native Anderson history; zero means unaccelerated fixed point.
    pub anderson: usize,
    /// Native fixed-point damping in (0,1].
    pub damping: pse_model::scalars::Fraction,
}
#[derive(Clone, Debug)]
struct UnitProgram {
    declaration: CausalUnitRequest,
    program: Arc<ExecutableCase>,
    values: CaseValues,
    inputs: Vec<(SemanticId, SemanticId, pse_quantity::UnitConvertSpec)>,
    outputs: Vec<(SemanticId, usize, pse_quantity::UnitConvertSpec)>,
    conditional: Option<crate::math::initialization::PreparedConditionalUnit>,
}
#[derive(Debug)]
struct UnitWorker {
    program: UnitProgram,
    /// The unit evaluator with its share of the declared-root job reservation.
    worker: crate::math::ExecutionWorker,
    input_ids: Vec<SemanticId>,
    output_ids: Vec<SemanticId>,
    service: Arc<crate::math::MathService>,
    providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    budget: Arc<crate::math::WorkerBudget>,
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
        if let Some(stop) = execution.stopped() {
            return Err(native::ProblemError::stopped(
                stop,
                "conditional causal unit stopped",
            ));
        }
        // An evaluation is an immutable-original overlay. Every exit drops the
        // temporary inputs and solved coordinates, including a native refusal.
        let mut trial = self.program.values.clone();
        for (port, symbol, conversion) in &self.program.inputs {
            let value = inputs
                .get(port)
                .ok_or_else(|| native::ProblemError::Contract("missing causal input".into()))?;
            let value = value * conversion.scale + conversion.offset;
            if !value.is_finite() {
                return Err(native::ProblemError::numerical(
                    "nonfinite causal boundary input",
                ));
            }
            trial.scalars.insert(*symbol, value);
        }
        if let Some(conditional) = &self.program.conditional {
            trial = self.service.evaluate_conditional_unit(
                conditional,
                trial,
                &self.providers,
                execution,
                &self.budget,
            )?;
        }
        let values = self.worker.worker().constraints(&trial)?;
        let outputs: BTreeMap<_, _> = self
            .program
            .outputs
            .iter()
            .map(|(port, row, c)| (*port, values[*row] * c.scale + c.offset))
            .collect();
        if outputs.values().any(|v| !v.is_finite()) {
            return Err(native::ProblemError::numerical("nonfinite causal output"));
        }
        Ok(outputs)
    }
}
/// Compiled immutable causal functions and the independently checked selected tear graph.
#[derive(Clone, Debug)]
pub struct PreparedRecycle {
    runtime: Runtime,
    providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    _source: crate::math::modeling::ModelingCasePreparation,
    request: RecycleRequest,
    graph: Arc<pse_structural::flowsheet::FlowGraph>,
    programs: Vec<UnitProgram>,
    contract: native::OracleContract,
    initial: Vec<f64>,
    fixed: BTreeMap<SemanticId, f64>,
    settings: kinsol::Settings,
    controls: Controls,
    /// Stopping budgets resolved from the map's numerical policy.
    accuracy: ResolvedAccuracy,
    tolerances: Tolerances,
    numerics: Arc<pse_model::numerics::ResolvedNumericalPolicy>,
}
impl PreparedRecycle {
    /// Source-owned numerical magnitudes projected onto selected tear coordinates.
    pub fn numerics(&self) -> &pse_model::numerics::ResolvedNumericalPolicy {
        &self.numerics
    }
    /// Exact causal directions and tear decisions selected by the caller.
    pub fn request(&self) -> &RecycleRequest {
        &self.request
    }
    /// Admitted internal scalar-coordinate directions. Aggregate request ports expand
    /// here through their authoritative state specification, while `request` retains
    /// the caller's original material-boundary identities.
    pub fn resolved_units(&self) -> impl Iterator<Item = &CausalUnitRequest> {
        self.programs.iter().map(|p| &p.declaration)
    }
    /// Original local boundaries, compiler matching witnesses and selected root routes,
    /// inspectable before native iteration begins.
    pub fn conditional_units(
        &self,
    ) -> impl Iterator<
        Item = (
            SemanticId,
            &pse_compiler::workspace::PreparedBlock,
            native::routing::Route,
        ),
    > {
        self.programs.iter().filter_map(|p| {
            p.conditional
                .as_ref()
                .map(|c| (p.declaration.node, &c.view, c.route()))
        })
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
        Ok(self.runtime.native().solve_declared_root(
            self.contract.clone(),
            self.initial.clone(),
            self.settings.clone(),
            self.controls.clone(),
            self.accuracy.clone(),
            self.tolerances.clone(),
            move |execution, budget| {
                let mut units: BTreeMap<SemanticId, Box<dyn CausalUnit>> = BTreeMap::new();
                for program in prepared.programs {
                    // Every unit evaluator lives for the whole sweep: each is charged to
                    // the job reservation, and its providers observe the attempt's
                    // cancellation (F31).
                    let worker = prepared
                        .runtime
                        .native()
                        .worker(
                            program.program.clone(),
                            &prepared.providers,
                            execution.cancel.clone(),
                            &budget,
                        )
                        .map_err(MathRuntimeError::into_problem)?;
                    let unit = UnitWorker {
                        input_ids: program.inputs.iter().map(|x| x.0).collect(),
                        output_ids: program.outputs.iter().map(|x| x.0).collect(),
                        program,
                        worker,
                        service: prepared.runtime.native().clone(),
                        providers: prepared.providers.clone(),
                        budget: budget.clone(),
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
impl ModelingPackage {
    /// Admit explicitly directed units from this revision, retaining physical port conversions.
    pub async fn prepare_recycle(
        &self,
        analysis: &ModelingAnalysis,
        selection: ModelingFlowSelection,
        request: RecycleRequest,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedRecycle, WorkflowError> {
        let profile = analysis.solver.clone();
        let compiler = analysis.compiler;
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
        if !matches!(profile.backend, native::execution::BackendSettings::Default) {
            return Err(contract(
                "causal map settings derive from its numerical policy and declared map controls",
            ));
        }
        profile
            .controls
            .validate()
            .map_err(MathRuntimeError::from)?;
        let resolved = self
            .resolve_case(
                analysis.root,
                analysis.instance,
                analysis.bindings.clone(),
                analysis.limits,
                analysis.case.clone(),
                DerivativeOrder::First,
                compiler,
                profile.clone(),
                analysis.numerical.clone(),
                Default::default(),
                false,
                cancel,
            )
            .await?;
        let flow = self
            .runtime
            .native()
            .prepare_modeling_flow(
                resolved.model.model.clone(),
                self.quantities.clone(),
                selection,
                cancel,
            )
            .await?;
        let graph = Arc::new(flow.graph().clone());
        graph
            .witness(&request.tears)
            .map_err(|e| contract(e.to_string()))?;
        let q = &self.quantities;
        let ports: BTreeMap<_, _> = graph
            .declaration()
            .nodes
            .iter()
            .flat_map(|n| n.ports.iter().map(|p| (p.id, p)))
            .collect();
        let mut programs = Vec::new();
        let mut all_inputs = BTreeMap::new();
        let mut all_outputs = BTreeSet::new();
        let units = request
            .units
            .iter()
            .map(|unit| {
                let model = &resolved.model.model.compiled().model;
                Ok(CausalUnitRequest {
                    node: unit.node,
                    inputs: expand_unit_ports(model, &unit.inputs)
                        .map_err(|e| unit_preparation_error(unit, &request, e))?,
                    outputs: expand_unit_ports(model, &unit.outputs)
                        .map_err(|e| unit_preparation_error(unit, &request, e))?,
                    realization: unit.realization.clone(),
                })
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        for unit in &units {
            let quantity_error = |cause: pse_quantity::QuantityError| {
                unit_preparation_error(unit, &request, pse_math::MathError::from(cause).into())
            };
            let node = graph
                .declaration()
                .nodes
                .iter()
                .find(|n| n.id == unit.node)
                .ok_or_else(|| unit_contract(unit, &request, "unknown causal node"))?;
            let inventory: BTreeSet<_> = unit
                .inputs
                .iter()
                .chain(unit.outputs.iter())
                .copied()
                .collect();
            if inventory != node.ports.iter().map(|p| p.id).collect()
                || inventory.len() != unit.inputs.len() + unit.outputs.len()
                || programs
                    .iter()
                    .any(|p: &UnitProgram| p.declaration.node == unit.node)
            {
                return Err(unit_contract(
                    unit,
                    &request,
                    "causal direction must cover every port once and every node once",
                ));
            }
            let source = resolved.model.case.compiled().plan.structure();
            let symbols: BTreeMap<_, _> = source
                .variables()
                .iter()
                .map(|v| (v.port.id, &v.port))
                .chain(source.parameters().iter().map(|p| (p.id, p)))
                .collect();
            let mut inputs = Vec::new();
            let source_ports = &resolved.model.model.compiled().model.ports;
            let input_program = self
                .runtime
                .native()
                .prepare_modeling_functions(
                    self.workspace.clone(),
                    resolved.model.model.clone(),
                    unit.inputs
                        .iter()
                        .map(|p| ModelingOutput::Member(source_ports[p].symbol).row_id())
                        .collect(),
                    Vec::new(),
                    DerivativeOrder::Value,
                    compiler,
                    cancel,
                )
                .await
                .map_err(|cause| unit_preparation_error(unit, &request, cause))?;
            let initial_values = resolved.model.values.clone();
            let initial_inputs = self
                .runtime
                .native()
                .with_owned_worker(
                    input_program.clone(),
                    resolved.providers.clone(),
                    cancel,
                    move |mut worker| Ok(worker.constraints(&initial_values)?),
                )
                .await
                .map_err(|cause| unit_preparation_error(unit, &request, cause))?;
            for port in &unit.inputs {
                let symbol = source_ports[port].symbol;
                let (index, row) = input_program
                    .assembly
                    .structure()
                    .rows()
                    .iter()
                    .enumerate()
                    .find(|(_, r)| r.id == ModelingOutput::Member(symbol).row_id())
                    .ok_or_else(|| {
                        unit_contract(unit, &request, "unknown causal input observation")
                    })?;
                let canonical = q
                    .quantity_type(row.quantity)
                    .map_err(&quantity_error)?
                    .canonical_unit;
                let bound = symbols.get(&symbol);
                if bound.is_none() && matches!(unit.realization, CausalUnitRealization::ExplicitMap)
                {
                    return Err(contract(
                        "an expression input requires a declared conditional unit boundary",
                    ));
                }
                if source.variables().iter().any(|v| {
                    v.port.id == symbol
                        && (v.lower.is_some() || v.upper.is_some() || v.domain.is_integer())
                }) {
                    return Err(unit_preparation_error(
                        unit,
                        &request,
                        native::ProblemError::Unsupported("KINSOL fixed point cannot enforce causal input bounds or integrality; request a constrained simultaneous strategy".into()).into(),
                    ));
                }
                let quantity = bound.map_or(row.quantity, |b| b.quantity);
                let unit_id = bound.map_or(canonical, |b| b.unit);
                pse_quantity::admission::require_same_contract(ports[port].quantity, quantity, q)
                    .map_err(&quantity_error)?;
                let conversion = pse_quantity::convert_spec_for_type(
                    q.unit(ports[port].unit).map_err(&quantity_error)?,
                    q.unit(unit_id).map_err(&quantity_error)?,
                    &q.quantity_type(quantity).map_err(&quantity_error)?.key,
                )
                .map_err(&quantity_error)?;
                inputs.push((
                    *port,
                    if bound.is_some() {
                        symbol
                    } else {
                        pse_ids::named_id(symbol, "conditional-boundary-value")
                    },
                    conversion,
                ));
                let to_port = pse_quantity::convert_spec_for_type(
                    q.unit(canonical).map_err(&quantity_error)?,
                    q.unit(ports[port].unit).map_err(&quantity_error)?,
                    &q.quantity_type(row.quantity).map_err(&quantity_error)?.key,
                )
                .map_err(&quantity_error)?;
                if all_inputs
                    .insert(
                        *port,
                        initial_inputs[index] * to_port.scale + to_port.offset,
                    )
                    .is_some()
                {
                    return Err(unit_contract(unit, &request, "duplicate causal input port"));
                }
            }
            let outputs: Vec<_> = unit
                .outputs
                .iter()
                .map(|port| ModelingOutput::Member(source_ports[port].symbol).row_id())
                .collect();
            let declared: BTreeSet<_> =
                unit.inputs.iter().map(|p| source_ports[p].symbol).collect();
            let required: BTreeSet<_> = unit
                .outputs
                .iter()
                .map(|p| source_ports[p].symbol)
                .collect();
            let conditional =
                match &unit.realization {
                    CausalUnitRealization::ExplicitMap => None,
                    CausalUnitRealization::Conditional {
                        residuals,
                        unknowns,
                        solver,
                    } => Some(
                        self.runtime
                            .native()
                            .prepare_conditional_unit(
                                self.workspace.clone(),
                                resolved.model.model.clone(),
                                resolved.model.case.clone(),
                                unit.node,
                                self.quantities.clone(),
                                declared.clone(),
                                required,
                                residuals.clone(),
                                unknowns.clone(),
                                compiler,
                                solver.as_ref().clone().profile().map_err(|cause| {
                                    conditional_admission(unit, &request, cause)
                                })?,
                                resolved.numerics.clone(),
                                cancel,
                            )
                            .await
                            .map_err(|cause| conditional_admission(unit, &request, cause))?,
                    ),
                };
            let program = self
                .runtime
                .native()
                .prepare_modeling_functions(
                    self.workspace.clone(),
                    resolved.model.model.clone(),
                    outputs,
                    Vec::new(),
                    DerivativeOrder::Value,
                    compiler,
                    cancel,
                )
                .await
                .map_err(|cause| unit_preparation_error(unit, &request, cause))?;
            // Every free dependency of each explicit function must be a declared input.
            let free: BTreeSet<_> = source
                .variables()
                .iter()
                .filter(|v| !v.fixed)
                .map(|v| v.port.id)
                .collect();
            if conditional.is_none()
                && program.assembly.structure().instances().iter().any(|i| {
                    i.contributions.iter().any(|c| {
                        program.assembly.bodies()[&i.body].support().first[c.output]
                            .iter()
                            .any(|slot| {
                                let id = i.slots[*slot].source();
                                free.contains(&id) && !declared.contains(&id)
                            })
                    })
                })
            {
                return Err(contract(
                    "causal function depends on an undeclared free input",
                ));
            }
            let mut mapped = Vec::new();
            for port in &unit.outputs {
                let row = source_ports[port].symbol;
                let (index, source_row) = program
                    .assembly
                    .structure()
                    .rows()
                    .iter()
                    .enumerate()
                    .find(|(_, r)| r.id == ModelingOutput::Member(row).row_id())
                    .ok_or_else(|| unit_contract(unit, &request, "unknown causal output row"))?;
                pse_quantity::admission::require_same_contract(
                    source_row.quantity,
                    ports[port].quantity,
                    q,
                )
                .map_err(&quantity_error)?;
                let canonical = q
                    .quantity_type(source_row.quantity)
                    .map_err(&quantity_error)?
                    .canonical_unit;
                let conversion = pse_quantity::convert_spec_for_type(
                    q.unit(canonical).map_err(&quantity_error)?,
                    q.unit(ports[port].unit).map_err(&quantity_error)?,
                    &q.quantity_type(source_row.quantity)
                        .map_err(&quantity_error)?
                        .key,
                )
                .map_err(&quantity_error)?;
                mapped.push((*port, index, conversion));
                all_outputs.insert(*port);
            }
            programs.push(UnitProgram {
                declaration: unit.clone(),
                program,
                values: resolved.model.values.clone(),
                inputs,
                outputs: mapped,
                conditional,
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
        let mut h = FramedHasher::new(pse_ids::Frame::CausalMapV2);
        h.hash(&self.revision.identity())
            .hash(&resolved.model.case.compiled().plan.structure().key())
            .hash(&resolved.model.values.identity())
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
        let mut projection_source = resolved.numerics.as_ref().clone();
        let missing: BTreeMap<_, _> = tears
            .iter()
            .filter_map(|p| {
                let symbol = resolved.model.model.compiled().model.ports[p].symbol;
                (!projection_source.targets.iter().any(|t| t.id == symbol)).then_some((
                    symbol,
                    pse_math::numerics::TargetSpec {
                        id: symbol,
                        kind: pse_model::generated::enums::NumericalTarget::Observable,
                        quantity: ports[p].quantity,
                        unit: ports[p].unit,
                        integer: false,
                        declared_tolerance: None,
                    },
                ))
            })
            .collect();
        if !missing.is_empty() {
            let additional = pse_math::numerics::resolve(
                q,
                &missing.into_values().collect::<Vec<_>>(),
                &[],
                &projection_source.policy,
            )
            .map_err(crate::workflow::math)?;
            projection_source.targets.extend(additional.targets);
            let mut h = FramedHasher::new(pse_ids::Frame::NumericalProjectionV1);
            h.hash(&projection_source.key).hash(&additional.key);
            projection_source.key = h.finish_hash();
        }
        let targets: Vec<_> = tears
            .iter()
            .zip(&contract.rows)
            .flat_map(|(p, r)| {
                [
                    (pse_model::generated::enums::NumericalTarget::Variable, *p),
                    (pse_model::generated::enums::NumericalTarget::Row, *r),
                ]
                .map(|(kind, id)| pse_math::numerics::TargetProjection {
                    source: resolved.model.model.compiled().model.ports[p].symbol,
                    source_kind: if projection_source.targets.iter().any(|t| {
                        t.id == resolved.model.model.compiled().model.ports[p].symbol
                            && t.kind == pse_model::generated::enums::NumericalTarget::Variable
                    }) {
                        pse_model::generated::enums::NumericalTarget::Variable
                    } else {
                        pse_model::generated::enums::NumericalTarget::Observable
                    },
                    target: pse_math::numerics::TargetSpec {
                        id,
                        kind,
                        quantity: ports[p].quantity,
                        unit: ports[p].unit,
                        integer: false,
                        declared_tolerance: None,
                    },
                })
            })
            .collect();
        let numerics = Arc::new(
            pse_math::numerics::project(q, &projection_source, &targets)
                .map_err(crate::workflow::math)?,
        );
        let ids: Vec<_> = tears.into_iter().collect();
        let tolerances = Tolerances::from_policy(&numerics, &ids, &contract.rows)
            .map_err(MathRuntimeError::from)?;
        let identity = pse_math::normalization::Normalization {
            variables: vec![1.0; ids.len()],
            rows: vec![1.0; ids.len()],
            objective: 1.0,
        };
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &identity)
            .map_err(MathRuntimeError::from)?;
        let settings = kinsol::Settings::from_policy(
            kinsol::Method {
                strategy: kinsol::Strategy::FixedPoint,
                anderson: request.anderson,
                damping: request.damping,
                ..kinsol::Method::default()
            },
            &tolerances,
            &identity,
            accuracy.feasibility,
        );
        settings
            .validate_contract(&contract, kinsol::Strategy::FixedPoint, &BTreeMap::new())
            .map_err(MathRuntimeError::from)?;
        Ok(PreparedRecycle {
            runtime: self.runtime.clone(),
            providers: resolved.providers,
            _source: resolved.model,
            request,
            graph,
            programs,
            contract,
            initial,
            fixed,
            settings,
            controls: profile.controls,
            accuracy,
            tolerances,
            numerics,
        })
    }
}

fn expand_unit_ports(
    model: &pse_modeling::SpecializedModel,
    requested: &BTreeSet<SemanticId>,
) -> Result<BTreeSet<SemanticId>, MathRuntimeError> {
    let mut coordinates = BTreeSet::new();
    for id in requested {
        if let Some(port) = model.material_ports.get(id) {
            if port.coordinates.is_empty()
                || port.coordinates.values().any(|p| !coordinates.insert(*p))
            {
                return Err(pse_math::MathError::Contract(
                    "causal material-boundary inventory has empty or overlapping independent coordinates".into(),
                ).into());
            }
        } else if !model.ports.contains_key(id) || !coordinates.insert(*id) {
            return Err(pse_math::MathError::Contract(
                "causal unit names an unknown or overlapping boundary coordinate".into(),
            )
            .into());
        }
    }
    Ok(coordinates)
}

#[cfg(test)]
pub(in crate::workflow) mod reference_fixture;
#[cfg(test)]
mod tests;

/// The execution boundary supplies identities; callers never parse formatted IDs
/// from the compiler or adapter's message to recover a selected-unit witness.
fn conditional_admission(
    unit: &CausalUnitRequest,
    request: &RecycleRequest,
    cause: MathRuntimeError,
) -> WorkflowError {
    use pse_model::diagnostic::{BoundaryClass as C, Observation};
    let mut diagnostic =
        crate::workflow::diagnostics::observed(&cause, "modeling.conditional_unit.admission");
    if matches!(
        cause,
        MathRuntimeError::Compile(pse_compiler::workspace::CompileError::Missing(_))
    ) {
        diagnostic.class = C::InvalidModel;
    }
    diagnostic.stage = "modeling.conditional_unit.admission".into();
    diagnostic.rule = match diagnostic.class {
        C::InvalidModel => "modeling.conditional_unit.admission.invalid_model",
        C::Unsupported => "modeling.conditional_unit.admission.unsupported",
        C::Cancelled => "modeling.conditional_unit.admission.cancelled",
        C::ResourceLimit => "modeling.conditional_unit.admission.resource_limit",
        C::TrialRejected => "modeling.conditional_unit.admission.trial_rejected",
        C::Nonfinite => "modeling.conditional_unit.admission.nonfinite",
        C::Infrastructure => "modeling.conditional_unit.admission.infrastructure",
        C::Conflict => "modeling.conditional_unit.admission.conflict",
        C::Incompatible => "modeling.conditional_unit.admission.incompatible",
        C::Internal => "modeling.conditional_unit.admission.internal",
        C::Numerical => "modeling.conditional_unit.admission.numerical",
        C::Inconclusive => "modeling.conditional_unit.admission.inconclusive",
    }
    .into();
    diagnostic.sources.push(unit.node);
    diagnostic
        .sources
        .extend(unit.inputs.iter().chain(&unit.outputs).copied());
    if let Some(public) = request.units.iter().find(|u| u.node == unit.node) {
        diagnostic
            .sources
            .extend(public.inputs.iter().chain(&public.outputs).copied());
    }
    if let CausalUnitRealization::Conditional {
        residuals,
        unknowns,
        ..
    } = &unit.realization
    {
        diagnostic
            .sources
            .extend(residuals.iter().chain(unknowns).copied());
    }
    diagnostic.sources.sort_unstable();
    diagnostic.sources.dedup();
    diagnostic
        .observations
        .insert("cause".into(), Observation::Text(cause.to_string()));
    WorkflowError::ConditionalAdmission {
        diagnostic: Box::new(diagnostic),
        cause,
    }
}

fn unit_preparation_error(
    unit: &CausalUnitRequest,
    request: &RecycleRequest,
    cause: MathRuntimeError,
) -> WorkflowError {
    if matches!(unit.realization, CausalUnitRealization::Conditional { .. }) {
        conditional_admission(unit, request, cause)
    } else {
        cause.into()
    }
}

fn unit_contract(
    unit: &CausalUnitRequest,
    request: &RecycleRequest,
    message: &str,
) -> WorkflowError {
    if matches!(unit.realization, CausalUnitRealization::Conditional { .. }) {
        conditional_admission(
            unit,
            request,
            MathRuntimeError::Math(pse_math::MathError::Contract(message.into())),
        )
    } else {
        contract(message)
    }
}
