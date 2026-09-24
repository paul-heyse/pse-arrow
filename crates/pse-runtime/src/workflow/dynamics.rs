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
    pub(crate) contract: native::Contract,
    pub(crate) programs: Vec<FunctionProgram>,
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
        let mut functions = Vec::new();
        for program in &self.programs {
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
            functions.push((
                program.clone(),
                program.case.assembly.worker(providers, cancel.clone()),
            ));
        }
        Ok(DynamicWorker {
            prepared: self.clone(),
            functions,
            values: self.values.clone(),
        })
    }
}
impl ModelRevision {
    /// Prepare a declared semi-explicit dynamic case through the same Salsa/artifact service.
    pub async fn prepare_simulation(
        &self,
        id: SemanticId,
        profile: SimulationProfile,
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
            .sources
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
        let mut h = FramedHasher::new("pse.dynamic.source.v1");
        h.hash(&self.identity());
        d.frame(&mut h);
        let balances = self
            .0
            .sources
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
                    tolerance: b
                        .integral_tolerance
                        .ok_or_else(|| contract("conserved tolerance missing"))?,
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
        key.hash(&c.identity).hash(&profile_key);
        Ok(PreparedSimulation {
            revision: self.clone(),
            declaration: d,
            profile,
            contract: c,
            programs,
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
pub(crate) struct DynamicWorker {
    prepared: PreparedSimulation,
    functions: Vec<(FunctionProgram, CaseWorker)>,
    values: CaseValues,
}
impl Oracle for DynamicWorker {
    fn contract(&self) -> &native::Contract {
        &self.prepared.contract
    }
    fn support(&self, mode: usize, function: Function) -> Vec<(usize, usize)> {
        let Some((p, _)) = self
            .functions
            .iter()
            .find(|(p, _)| p.mode == mode && p.function == function)
        else {
            return vec![];
        };
        let pattern = p.case.assembly.jacobian_pattern();
        let mut out = Vec::new();
        for c in 0..pattern.ncols() {
            for &r in &pattern.row_idx()[pattern.col_range(c)] {
                for (target, &source) in p.rows.iter().enumerate() {
                    if r == source {
                        out.push((target, c));
                    }
                }
            }
        }
        out
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
        let d = &self.prepared.declaration;
        let n = d.states.len();
        if state.len() != n || parameters.len() != d.parameters.len() {
            return Err(ProblemError::Contract("dynamic binding dimensions".into()));
        }
        self.values
            .scalars
            .insert(d.time_id, time / self.prepared.time_scale);
        let mut chain = Vec::with_capacity(n + parameters.len());
        for (i, s) in d.states.iter().enumerate() {
            let c = &self.prepared.conversions[i];
            self.values.scalars.insert(
                s.symbol_id,
                (state[i] * s.scale + s.offset - c.offset) / c.scale,
            );
            chain.push(s.scale / c.scale);
        }
        for (i, id) in d.parameters.iter().enumerate() {
            let c = &self.prepared.conversions[n + i];
            self.values
                .scalars
                .insert(*id, (parameters[i] - c.offset) / c.scale);
            chain.push(1.0 / c.scale);
        }
        let Some((p, worker)) = self
            .functions
            .iter_mut()
            .find(|(p, _)| p.mode == mode && p.function == function)
        else {
            if function == Function::Roots {
                return Ok(native::Evaluation {
                    values: vec![],
                    jacobian: None,
                });
            }
            return Err(ProblemError::Contract(
                "missing compiled dynamic function".into(),
            ));
        };
        let rows = worker.constraints(&self.values)?;
        let values = p
            .rows
            .iter()
            .enumerate()
            .map(|(i, &r)| rows[r] * p.scales[i] + p.offsets[i])
            .collect();
        let jacobian = if derivatives {
            let source = worker.jacobian(&self.values)?;
            let mut triplets = Vec::new();
            for c in 0..source.ncols() {
                for k in source.col_range(c) {
                    let r = source.row_idx()[k];
                    for (i, &row) in p.rows.iter().enumerate() {
                        if row == r {
                            triplets.push(faer::sparse::Triplet::new(
                                i,
                                c,
                                source.val()[k] * p.scales[i] * chain[c],
                            ));
                        }
                    }
                }
            }
            Some(
                faer::sparse::SparseColMat::try_new_from_triplets(
                    p.rows.len(),
                    chain.len(),
                    &triplets,
                )
                .map_err(|e| ProblemError::Contract(e.to_string()))?,
            )
        } else {
            None
        };
        Ok(native::Evaluation { values, jacobian })
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
    for values in [&p.samples, &p.atol, &p.parameter_scales] {
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
