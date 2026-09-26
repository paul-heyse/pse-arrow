// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dynamic roles project the shared typed compiler; only attempts own native state.
use super::{DynamicDeclaration, ModelRevision, WorkflowError, contract, math};
use crate::math::ExecutableCase;
use pse_backend_native::{
    ProblemError,
    dynamics::{self as native, Function, Oracle},
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, Port};
use pse_math::{assembly::CaseWorker, binding::CaseValues};
use pse_model::SemanticFrame;
use pse_quantity::UnitConvertSpec;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicBool},
};
/// Dynamic controls use the concrete native profile; values are validated at preparation.
pub type SimulationProfile = native::Profile;
#[derive(Clone, Debug)]
pub(crate) struct FunctionProgram {
    mode: usize,
    function: Function,
    case: Arc<ExecutableCase>,
    rows: Vec<usize>,
    scales: Vec<f64>,
    offsets: Vec<f64>,
}
/// Immutable compiled physical simulation, with no integrator or mutable evaluator.
#[derive(Clone, Debug)]
pub struct PreparedSimulation {
    pub(crate) revision: ModelRevision,
    pub(crate) declaration: DynamicDeclaration,
    pub(crate) profile: SimulationProfile,
    pub(crate) numerics: Arc<pse_model::numerics::ResolvedNumericalPolicy>,
    pub(crate) contract: native::Contract,
    pub(crate) programs: Arc<[FunctionProgram]>,
    pub(crate) values: CaseValues,
    pub(crate) conversions: Vec<UnitConvertSpec>,
    pub(crate) time_scale: f64,
    pub(crate) parameters: Vec<f64>,
    pub(crate) output_ports: Vec<Port>,
    pub(crate) parameter_ports: Vec<Port>,
    pub(crate) state_ports: Vec<Port>,
    pub(crate) key: ContentHash,
    pub(crate) profile_key: ContentHash,
    pub(crate) bytes: usize,
}
impl PreparedSimulation {
    /// Rebind authored parameter values and a horizon through the same checked compiler.
    /// Unaffected bodies and executable artifacts remain shared by semantic identity.
    pub async fn rebind(
        &self,
        parameters: &BTreeMap<SemanticId, f64>,
        profile: SimulationProfile,
        compiler: pse_compiler::workspace::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        if parameters
            .iter()
            .any(|(id, value)| !self.declaration.parameters.contains(id) || !value.is_finite())
        {
            return Err(contract(
                "dynamic rebind requires finite declared parameter values",
            ));
        }
        let mut draft = self.revision.edit();
        let case = draft
            .declaration_mut()
            .cases
            .iter_mut()
            .find(|c| c.case_id == self.declaration.case_id)
            .ok_or_else(|| contract("dynamic rebind case is absent"))?;
        for (id, value) in parameters {
            let bound = case
                .values
                .iter_mut()
                .find(|v| v.symbol_id == *id)
                .ok_or_else(|| contract("dynamic rebind parameter has no authored value"))?;
            bound.value = *value;
        }
        draft
            .freeze()?
            .prepare_simulation(self.declaration.dynamic_id, profile, compiler, cancel)
            .await
    }
    /// Complete prepared execution identity, including exact data and effective controls.
    pub fn identity(&self) -> ContentHash {
        self.key
    }
    /// Stated integration/sensitivity controls.
    pub fn profile(&self) -> &SimulationProfile {
        &self.profile
    }
    /// Source model declaration; the compiled projection is never durable authority.
    pub fn declaration(&self) -> &DynamicDeclaration {
        &self.declaration
    }
    pub(crate) fn worker(&self, cancel: Arc<AtomicBool>) -> Result<DynamicWorker, ProblemError> {
        let chain = self
            .declaration
            .states
            .iter()
            .enumerate()
            .map(|(i, s)| s.scale / self.conversions[i].scale)
            .chain(
                self.declaration
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(i, _)| 1.0 / self.conversions[self.declaration.states.len() + i].scale),
            )
            .collect::<Vec<_>>();
        let mut functions = BTreeMap::new();
        for program in self.programs.iter() {
            let providers = self
                .revision
                .0
                .providers
                .values()
                .map(|p| p.registration.clone())
                .map(|r| {
                    r.worker()
                        .map(|w| (r.spec().key(), w))
                        .map_err(|e| ProblemError::Contract(e.to_string()))
                })
                .collect::<Result<_, _>>()?;
            let source = program.case.assembly.jacobian_pattern();
            let mut pairs = Vec::new();
            let mut refill = Vec::new();
            for (c, scale) in chain.iter().enumerate() {
                for k in source.col_range(c) {
                    for (row, &original) in program.rows.iter().enumerate() {
                        if source.row_idx()[k] == original {
                            refill.push((k, pairs.len(), program.scales[row] * scale));
                            pairs.push((row, c));
                        }
                    }
                }
            }
            let jacobian = pse_math::sparse::AssemblyMatrix::new(
                program.rows.len(),
                chain.len(),
                &pairs,
                self.profile.max_cells,
            )?;
            functions.insert(
                (program.mode, program.function),
                FunctionWorker {
                    program: program.clone(),
                    worker: program.case.assembly.worker(providers, cancel.clone()),
                    jacobian,
                    refill,
                    pairs,
                    cache: None,
                    #[cfg(test)]
                    evaluations: 0,
                },
            );
        }
        Ok(DynamicWorker {
            prepared: self.clone(),
            functions,
            values: self.values.clone(),
            cancel,
        })
    }
}
impl ModelRevision {
    /// Prepare a declared semi-explicit dynamic case through the same Salsa/artifact service.
    pub async fn prepare_simulation(
        &self,
        id: SemanticId,
        mut profile: SimulationProfile,
        compiler: pse_compiler::workspace::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedSimulation, WorkflowError> {
        if !cfg!(feature = "solver-diffsol") {
            return Err(contract("Diffsol adapter is not linked"));
        }
        let mut d = self
            .0
            .sources
            .dynamics
            .iter()
            .find(|d| d.dynamic_id == id)
            .cloned()
            .ok_or_else(|| contract("unknown dynamic declaration"))?;
        for b in self
            .0
            .resolved_sources
            .balances
            .iter()
            .filter(|b| b.case_id == d.case_id)
        {
            if let Some(state) = b.accumulation {
                let index = d
                    .states
                    .iter()
                    .position(|s| s.symbol_id == state)
                    .ok_or_else(|| contract("balance state missing"))?;
                for (mode, m) in d.modes.iter_mut().enumerate() {
                    m.rhs_rows[index] = super::balances::mode_row(b.balance_id, mode);
                }
            }
        }
        let inputs = self
            .0
            .cases
            .get(&d.case_id)
            .ok_or_else(|| contract("unknown dynamic function case"))?;
        let source = inputs
            .cases
            .get(&d.case_id)
            .ok_or_else(|| contract("missing dynamic function case"))?
            .structure
            .as_ref();
        let q = self.0.physical.quantities.as_ref();
        if source.objective().is_some() {
            return Err(contract(
                "dynamic function case cannot have an optimization objective",
            ));
        }
        let ports: BTreeMap<_, _> = source
            .variables()
            .iter()
            .map(|v| (v.port.id, v.port.clone()))
            .chain(source.parameters().iter().map(|p| (p.id, p.clone())))
            .collect();
        let get = |id| {
            ports
                .get(&id)
                .cloned()
                .ok_or_else(|| contract("unknown dynamic symbol"))
        };
        let rows: BTreeMap<_, _> = source.rows().iter().map(|r| (r.id, r.quantity)).collect();
        let row_quantity = |id| {
            rows.get(&id)
                .copied()
                .ok_or_else(|| contract("unknown dynamic function output"))
        };
        let time = get(d.time_id)?;
        if !source.parameters().iter().any(|p| p.id == time.id) {
            return Err(contract("dynamic time must be a parameter input"));
        }
        let time_unit = q.unit(time.unit).map_err(math)?;
        let time_dim = pse_quantity::DimensionVector::base(pse_quantity::BaseDimension::Time);
        if time_unit.dimension != time_dim || time_unit.is_affine {
            return Err(contract("dynamic time needs a non-affine time unit"));
        }
        if d.time_origin.is_some_and(|t| !t.is_finite()) {
            return Err(contract("nonfinite model time origin"));
        }
        let time_scale = time_unit.scale_to_canonical;
        let mut states = Vec::new();
        let mut conversions = Vec::new();
        let convert = |p: &Port| -> Result<UnitConvertSpec, WorkflowError> {
            let ty = q.quantity_type(p.quantity).map_err(math)?;
            pse_quantity::convert_spec_for_type(
                q.unit(p.unit).map_err(math)?,
                q.unit(ty.canonical_unit).map_err(math)?,
                &ty.key,
            )
            .map_err(math)
        };
        for s in &d.states {
            let p = get(s.symbol_id)?;
            if !source.variables().iter().any(|v| {
                v.port.id == p.id
                    && !v.fixed
                    && v.domain == pse_math::binding::VariableDomain::Continuous
            }) || !s.offset.is_finite()
                || !positive(s.scale)
                || !positive(s.residual_scale)
            {
                return Err(contract("dynamic state role/scaling"));
            }
            pse_quantity::admission::require_same_contract(
                p.quantity,
                row_quantity(s.initial_row)?,
                q,
            )
            .map_err(math)?;
            conversions.push(convert(&p)?);
            states.push(p);
        }
        if source
            .variables()
            .iter()
            .any(|v| !v.fixed && !d.states.iter().any(|s| s.symbol_id == v.port.id))
        {
            return Err(contract(
                "every free function-case variable must be a dynamic state",
            ));
        }
        let mut parameters = Vec::new();
        let mut parameter_ports = Vec::new();
        for id in &d.parameters {
            let p = get(*id)?;
            if !source.parameters().iter().any(|p| p.id == *id) || *id == d.time_id {
                return Err(contract("dynamic adjustable parameter role"));
            }
            let conversion = convert(&p)?;
            let value = *inputs
                .values
                .get(id)
                .ok_or_else(|| contract("missing dynamic parameter value"))?;
            parameters.push(value * conversion.scale + conversion.offset);
            conversions.push(conversion);
            parameter_ports.push(p);
        }
        let output_ports = d
            .outputs
            .iter()
            .map(|id| {
                let quantity = row_quantity(*id)?;
                Ok(Port {
                    id: *id,
                    quantity,
                    unit: q.quantity_type(quantity).map_err(math)?.canonical_unit,
                })
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        use pse_math::numerics::{SourcedRequirement, TargetSpec};
        use pse_model::generated::enums::{NumericalCoordinates, NumericalSource, NumericalTarget};
        let mut targets = Vec::new();
        let mut declarations = self
            .0
            .resolved_sources
            .numerics
            .iter()
            .filter(|r| r.case_id.is_none_or(|id| id == d.case_id))
            .map(|r| SourcedRequirement {
                source: if r.case_id.is_some() {
                    NumericalSource::Case
                } else {
                    NumericalSource::Model
                },
                declaration: r.clone(),
            })
            .collect::<Vec<_>>();
        for (state, port) in d.states.iter().zip(&states) {
            let unit = q.quantity_type(port.quantity).map_err(math)?.canonical_unit;
            targets.push(TargetSpec {
                id: port.id,
                kind: NumericalTarget::Variable,
                quantity: port.quantity,
                unit,
                integer: false,
                declared_tolerance: None,
            });
            declarations.push(SourcedRequirement {
                source: NumericalSource::Model,
                declaration: pse_model::numerics::NumericalRequirement {
                    requirement_id: pse_ids::named_id(
                        d.dynamic_id,
                        &format!("state-nominal.{}", port.id),
                    ),
                    model_id: d.model_id,
                    case_id: None,
                    target_id: port.id,
                    target_kind: NumericalTarget::Variable,
                    nominal: Some(state.scale),
                    scaling_factor: None,
                    absolute_tolerance: None,
                    relative_tolerance: None,
                    unit_id: Some(unit.as_id()),
                    coordinates: NumericalCoordinates::Physical,
                    priority: i32::MIN,
                    required: true,
                    provenance: "authored dynamic state normalization".into(),
                },
            });
        }
        for row in source.rows() {
            targets.push(TargetSpec {
                id: row.id,
                kind: NumericalTarget::Row,
                quantity: row.quantity,
                unit: q.quantity_type(row.quantity).map_err(math)?.canonical_unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        for output in &output_ports {
            targets.push(TargetSpec {
                id: output.id,
                kind: NumericalTarget::Observable,
                quantity: output.quantity,
                unit: output.unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        for (i, b) in self
            .0
            .resolved_sources
            .balances
            .iter()
            .filter(|b| b.case_id == d.case_id && b.accumulation.is_some())
            .enumerate()
        {
            let port = get(b
                .accumulation
                .ok_or_else(|| contract("missing conserved state"))?)?;
            let unit = q.quantity_type(port.quantity).map_err(math)?.canonical_unit;
            targets.push(TargetSpec {
                id: b.balance_id,
                kind: NumericalTarget::Closure,
                quantity: port.quantity,
                unit,
                integer: false,
                declared_tolerance: b.integral_tolerance,
            });
            targets.push(TargetSpec {
                id: b.balance_id,
                kind: NumericalTarget::Observable,
                quantity: port.quantity,
                unit,
                integer: false,
                declared_tolerance: profile.out_atol.get(i).copied(),
            });
        }
        declarations.extend(self.property_numerics(d.case_id, &targets)?);
        let numerics = Arc::new(
            pse_math::numerics::resolve(q, &targets, &declarations, &profile.numerics)
                .map_err(math)?,
        );
        for state in &mut d.states {
            state.scale = numerics
                .targets
                .iter()
                .find(|t| t.id == state.symbol_id && t.kind == NumericalTarget::Variable)
                .ok_or_else(|| contract("missing dynamic nominal"))?
                .coordinate_scale;
        }
        let integrated = self
            .0
            .resolved_sources
            .balances
            .iter()
            .filter(|b| b.case_id == d.case_id && b.accumulation.is_some())
            .collect::<Vec<_>>();
        if profile.out_atol.len() != integrated.len() {
            return Err(contract(
                "explicit integrated-flux absolute tolerances required",
            ));
        }
        for (atol, balance) in profile.out_atol.iter_mut().zip(&integrated) {
            *atol = numerics
                .targets
                .iter()
                .find(|t| t.id == balance.balance_id && t.kind == NumericalTarget::Observable)
                .ok_or_else(|| contract("missing integrated-flux numerical target"))?
                .budget;
        }
        let mut h = FramedHasher::new("pse.dynamic.source.v1");
        h.hash(&self.identity());
        d.frame(&mut h);
        let balances = self
            .0
            .resolved_sources
            .balances
            .iter()
            .filter(|b| b.case_id == d.case_id && b.accumulation.is_some())
            .map(|b| {
                let state = d
                    .states
                    .iter()
                    .position(|s| Some(s.symbol_id) == b.accumulation)
                    .ok_or_else(|| contract("conserved state missing"))?;
                Ok(native::Balance {
                    id: b.balance_id,
                    state,
                    scale: d.states[state].scale,
                    tolerance: numerics
                        .targets
                        .iter()
                        .find(|t| t.id == b.balance_id && t.kind == NumericalTarget::Closure)
                        .ok_or_else(|| contract("conserved tolerance missing"))?
                        .budget,
                    impulses: b.impulses.iter().map(|i| (i.event_id, i.value)).collect(),
                })
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        let c = native::Contract {
            balances,
            identity: h.finish_hash(),
            states: states.iter().map(|p| p.id).collect(),
            differential: d.states.iter().map(|s| s.differential).collect(),
            parameters: d.parameters.clone(),
            outputs: d.outputs.clone(),
            events: d
                .modes
                .iter()
                .map(|m| {
                    m.events
                        .iter()
                        .map(|e| {
                            Ok(native::Event {
                                id: e.event_id,
                                terminal: e.terminal,
                                next_mode: usize::try_from(e.next_mode)
                                    .map_err(|_| contract("dynamic mode index"))?,
                                tolerance: e.tolerance,
                            })
                        })
                        .collect()
                })
                .collect::<Result<_, WorkflowError>>()?,
        };
        profile
            .validate(&c, &parameters)
            .map_err(|e| WorkflowError::Math(e.into()))?;
        let coordinates: Vec<_> = c.states.iter().chain(&c.parameters).copied().collect();
        let mut programs = Vec::new();
        for (mode, m) in d.modes.iter().enumerate() {
            if m.rhs_rows.len() != d.states.len() {
                return Err(contract("dynamic RHS/state count"));
            }
            for (s, &row) in d.states.iter().zip(&m.rhs_rows) {
                if s.differential {
                    let qty = get(s.symbol_id)?.quantity;
                    let inferred = pse_quantity::infer::infer_with_evidence(
                        &pse_quantity::infer::OpRequest::Derivative {
                            domain_unit: time.unit,
                            domain_kind: pse_quantity::DomainKind::Time,
                            order: 1,
                        },
                        &[pse_quantity::infer::Operand {
                            quantity_type: qty,
                            indices: &pse_quantity::IndexSet::new(),
                        }],
                        q,
                        self.0.physical.preconditions.as_ref(),
                    )
                    .map_err(math)?;
                    pse_quantity::admission::require_same_contract(
                        inferred.result,
                        row_quantity(row)?,
                        q,
                    )
                    .map_err(math)?;
                } else {
                    row_quantity(row)?;
                }
            }
            let mut selections = vec![
                (Function::Rhs, m.rhs_rows.clone()),
                (
                    Function::Initial,
                    d.states.iter().map(|s| s.initial_row).collect(),
                ),
                (Function::Output, d.outputs.clone()),
            ];
            if !c.balances.is_empty() {
                selections.push((
                    Function::BalanceFlux,
                    c.balances
                        .iter()
                        .map(|b| super::balances::mode_row(b.id, mode))
                        .collect(),
                ));
            }
            if !m.events.is_empty() {
                selections.push((
                    Function::Roots,
                    m.events.iter().map(|e| e.guard_row).collect(),
                ));
            }
            for (index, event) in m.events.iter().enumerate() {
                if !event.terminal {
                    if event.reset_rows.len() != d.states.len() {
                        return Err(contract("complete reset state required"));
                    }
                    for (s, row) in states.iter().zip(&event.reset_rows) {
                        pse_quantity::admission::require_same_contract(
                            s.quantity,
                            row_quantity(*row)?,
                            q,
                        )
                        .map_err(math)?;
                    }
                    selections.push((Function::Reset(index), event.reset_rows.clone()));
                }
            }
            for (function, outputs) in selections {
                let selected = outputs
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                let case = self
                    .0
                    .runtime
                    .native()
                    .prepare_functions_revision(
                        self.0.workspace.clone(),
                        inputs.as_ref().clone(),
                        d.case_id,
                        selected,
                        coordinates.clone(),
                        DerivativeOrder::First,
                        compiler,
                        cancel,
                    )
                    .await?;
                let row_indices = outputs
                    .iter()
                    .map(|r| {
                        case.assembly
                            .structure()
                            .rows()
                            .iter()
                            .position(|x| x.id == *r)
                            .ok_or_else(|| contract("compiled output missing"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if function == Function::Rhs {
                    let algebraic = d
                        .states
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| !s.differential)
                        .collect::<Vec<_>>();
                    self.0
                        .runtime
                        .native()
                        .validate_dynamic_partition(
                            self.0.workspace.clone(),
                            inputs.as_ref().clone(),
                            d.case_id,
                            algebraic.iter().map(|(i, _)| m.rhs_rows[*i]).collect(),
                            algebraic.iter().map(|(_, s)| s.symbol_id).collect(),
                            cancel,
                        )
                        .await?;
                }
                if function == Function::Initial {
                    for instance in case.assembly.structure().instances() {
                        let body = &case.assembly.bodies()[&instance.body];
                        if body
                            .support()
                            .controls
                            .iter()
                            .any(|i| states.iter().any(|s| s.id == instance.slots[*i].source()))
                        {
                            return Err(contract("initial guard depends on a state guess"));
                        }
                    }
                    let j = case.assembly.jacobian_pattern();
                    if (0..states.len()).any(|col| !j.col_range(col).is_empty()) {
                        return Err(contract(
                            "initial values may depend on parameters/time, not state guesses",
                        ));
                    }
                }
                let mut scales = vec![1.0; outputs.len()];
                let mut offsets = vec![0.0; outputs.len()];
                match function {
                    Function::BalanceFlux => {
                        for (i, b) in c.balances.iter().enumerate() {
                            let rate = q
                                .quantity_type(row_quantity(super::balances::mode_row(b.id, mode))?)
                                .map_err(math)?;
                            let state = q.quantity_type(states[b.state].quantity).map_err(math)?;
                            scales[i] = q
                                .unit(rate.canonical_unit)
                                .map_err(math)?
                                .scale_to_canonical
                                / q.unit(state.canonical_unit)
                                    .map_err(math)?
                                    .scale_to_canonical;
                        }
                    }
                    Function::Rhs => {
                        for (i, s) in d.states.iter().enumerate() {
                            scales[i] = if s.differential {
                                let rate = q
                                    .quantity_type(row_quantity(m.rhs_rows[i])?)
                                    .map_err(math)?;
                                let state = q.quantity_type(states[i].quantity).map_err(math)?;
                                q.unit(rate.canonical_unit)
                                    .map_err(math)?
                                    .scale_to_canonical
                                    / q.unit(state.canonical_unit)
                                        .map_err(math)?
                                        .scale_to_canonical
                                    / s.scale
                            } else {
                                1.0 / s.residual_scale
                            };
                        }
                    }
                    Function::Initial | Function::Reset(_) => {
                        for (i, s) in d.states.iter().enumerate() {
                            scales[i] = 1.0 / s.scale;
                            offsets[i] = -s.offset / s.scale;
                        }
                    }
                    _ => {}
                }
                programs.push(FunctionProgram {
                    mode,
                    function,
                    case,
                    rows: row_indices,
                    scales,
                    offsets,
                });
            }
        }
        let cells = profile
            .validate(&c, &parameters)
            .map_err(|e| WorkflowError::Math(e.into()))?;
        let bytes = programs.iter().try_fold(
            cells
                .checked_mul(64)
                .and_then(|n| n.checked_add(256 * 1024))
                .ok_or_else(|| contract("dynamic result extent"))?,
            |n, p| {
                n.checked_add(p.case.assembly.numeric_worker_bytes())
                    .ok_or_else(|| contract("dynamic worker extent"))
            },
        )?;
        let profile_key = profile_identity(&profile);
        let mut key = FramedHasher::new("pse.dynamic.prepared.v1");
        key.hash(&c.identity).hash(&profile_key).hash(&numerics.key);
        Ok(PreparedSimulation {
            revision: self.clone(),
            declaration: d,
            profile,
            numerics,
            contract: c,
            programs: programs.into(),
            values: CaseValues {
                scalars: inputs.values.clone(),
            },
            conversions,
            time_scale,
            parameters,
            output_ports,
            parameter_ports,
            state_ports: states,
            key: key.finish_hash(),
            profile_key,
            bytes,
        })
    }
}
#[derive(Debug)]
struct FunctionWorker {
    program: FunctionProgram,
    worker: CaseWorker,
    jacobian: pse_math::sparse::AssemblyMatrix,
    refill: Vec<(usize, usize, f64)>,
    pairs: Vec<(usize, usize)>,
    // Mode/function and provider/build identity are fixed by this worker. Every
    // varying time/state/parameter bit participates, including signed zero.
    cache: Option<(Vec<u64>, native::Evaluation)>,
    #[cfg(test)]
    evaluations: usize,
}
#[derive(Debug)]
pub(crate) struct DynamicWorker {
    prepared: PreparedSimulation,
    functions: BTreeMap<(usize, Function), FunctionWorker>,
    values: CaseValues,
    cancel: Arc<AtomicBool>,
}
impl Oracle for DynamicWorker {
    fn contract(&self) -> &native::Contract {
        &self.prepared.contract
    }
    fn support(&self, mode: usize, function: Function) -> Vec<(usize, usize)> {
        self.functions
            .get(&(mode, function))
            .map_or_else(Vec::new, |w| w.pairs.clone())
    }
    fn evaluate(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        derivatives: bool,
    ) -> Result<native::Evaluation, ProblemError> {
        if self.cancel.load(std::sync::atomic::Ordering::Acquire) {
            return Err(pse_math::MathError::Cancelled.into());
        }
        let d = &self.prepared.declaration;
        let n = d.states.len();
        if state.len() != n
            || parameters.len() != d.parameters.len()
            || !time.is_finite()
            || state.iter().chain(parameters).any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "dynamic binding dimensions or values".into(),
            ));
        }
        let Some(function) = self.functions.get_mut(&(mode, function)) else {
            if function == Function::Roots && mode < d.modes.len() {
                return Ok(native::Evaluation {
                    values: vec![],
                    jacobian: None,
                });
            }
            return Err(ProblemError::Contract(
                "missing compiled dynamic function".into(),
            ));
        };
        let bits = std::iter::once(time)
            .chain(state.iter().copied())
            .chain(parameters.iter().copied())
            .map(f64::to_bits);
        if let Some((key, evaluation)) = &function.cache
            && (!derivatives || evaluation.jacobian.is_some())
            && key.iter().copied().eq(bits.clone())
        {
            let mut result = evaluation.clone();
            if !derivatives {
                result.jacobian = None;
            }
            return Ok(result);
        }
        function.cache = None;
        #[cfg(test)]
        {
            function.evaluations += 1;
        }
        self.values.scalars.insert(
            d.time_id,
            (time - d.time_origin.unwrap_or(0.0)) / self.prepared.time_scale,
        );
        for (i, s) in d.states.iter().enumerate() {
            let c = &self.prepared.conversions[i];
            self.values.scalars.insert(
                s.symbol_id,
                (state[i] * s.scale + s.offset - c.offset) / c.scale,
            );
        }
        for (i, id) in d.parameters.iter().enumerate() {
            let c = &self.prepared.conversions[n + i];
            self.values
                .scalars
                .insert(*id, (parameters[i] - c.offset) / c.scale);
        }
        let p = &function.program;
        let rows = function.worker.constraints(&self.values)?;
        let values = p
            .rows
            .iter()
            .enumerate()
            .map(|(i, &r)| rows[r] * p.scales[i] + p.offsets[i])
            .collect::<Vec<_>>();
        let jacobian = if derivatives {
            let source = function.worker.jacobian(&self.values)?;
            function.jacobian.clear();
            for &(local, target, scale) in &function.refill {
                function.jacobian.add(target, source.val()[local] * scale)?;
            }
            Some(function.jacobian.matrix().clone())
        } else {
            None
        };
        if values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract("nonfinite dynamic values".into()));
        }
        let result = native::Evaluation { values, jacobian };
        function.cache = Some((bits.collect(), result.clone()));
        Ok(result)
    }
}

fn positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}
pub(crate) fn profile_identity(p: &SimulationProfile) -> ContentHash {
    let mut h = FramedHasher::new("pse.dynamic.profile.v1");
    for x in [p.start, p.end, p.rtol, p.initial_step] {
        h.u64(x.to_bits());
    }
    h.hash(&p.numerics.key()).bool(p.out_rtol.is_some());
    if let Some(t) = p.out_rtol {
        h.u64(t.to_bits());
    }
    for values in [&p.samples, &p.atol, &p.parameter_scales, &p.out_atol] {
        h.u64(values.len() as u64);
        for x in values {
            h.u64(x.to_bits());
        }
    }
    h.u64(p.max_steps as u64)
        .u64(p.max_events as u64)
        .u64(p.max_cells as u64)
        .u64(u64::from(p.sensitivities));
    h.u64(p.time_limit.as_secs())
        .u64(u64::from(p.time_limit.subsec_nanos()));
    h.u64(p.changes.len() as u64);
    for c in &p.changes {
        h.u64(c.time.to_bits());
        for v in &c.parameters {
            h.u64(v.to_bits());
        }
    }
    #[cfg(feature = "solver-diffsol")]
    {
        h.str(&pse_backend_native::dynamics::settings_identity(p));
    }
    h.finish_hash()
}

#[cfg(test)]
pub(super) mod tests;
