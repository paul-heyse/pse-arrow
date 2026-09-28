// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Event bindings select authored expressions and structural facts, never new equations.
use super::*;

/// Zero crossing of an authored scalar member. Tolerance uses its canonical unit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelingDynamicEvent {
    /// Path of the authored scalar member whose zero crossing triggers the event.
    pub guard: String,
    /// Each entry maps a state path to an authored expression with the same quantity type.
    #[serde(default)]
    pub reset: BTreeMap<String, String>,
    /// Stop the integration at the event instead of resetting.
    pub terminal: bool,
    /// Required for resets; terminal events have no successor.
    pub next_mode: Option<String>,
    /// Absolute guard tolerance for ambiguity detection, in the guard's canonical unit.
    pub tolerance: f64,
}
/// One same-layout specialization. The first mode supplies the initial condition.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelingDynamicMode {
    /// Mode name; the first declared mode starts the integration.
    pub name: String,
    /// Boolean facts read by source `when` variants and named `stage` overrides.
    #[serde(default)]
    pub facts: BTreeMap<String, bool>,
    /// Events active in this mode.
    #[serde(default)]
    pub events: Vec<ModelingDynamicEvent>,
}
impl ModelingDynamicMode {
    pub(super) fn extend_bindings(&self, bindings: &mut Bindings) -> Result<(), WorkflowError> {
        for (name, value) in &self.facts {
            if name.starts_with("analysis.") {
                return Err(contract("event modes cannot change the analysis route"));
            }
            bindings.facts.insert(
                name.clone(),
                pse_modeling::specialize::Value::Boolean(*value),
            );
        }
        for event in &self.events {
            bindings.demand.push(event.guard.clone());
            bindings
                .demand
                .extend(event.reset.keys().chain(event.reset.values()).cloned());
        }
        Ok(())
    }
}
impl ModelingPackage {
    /// Bind an authored case to the integrated route with declared same-layout modes.
    pub async fn declared_simulation_modes(
        &self,
        root: SemanticId,
        compiler: Profile,
        profile: native::Profile,
        limits: Limits,
        modes: Vec<ModelingDynamicMode>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        let (bindings, case) = self
            .declared_case(
                root,
                pse_model::generated::enums::ModelingAnalysisRoute::Integrated,
                limits,
                cancel,
            )
            .await?;
        self.prepare_simulation_modes(
            root, root, bindings, limits, case, compiler, profile, modes, cancel,
        )
        .await
    }
    /// Prepare an integrated simulation of one instance under explicit bindings.
    pub async fn prepare_simulation(
        &self,
        root: SemanticId,
        instance: SemanticId,
        bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        compiler: Profile,
        profile: native::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        self.prepare_simulation_modes(
            root,
            instance,
            bindings,
            limits,
            case,
            compiler,
            profile,
            vec![ModelingDynamicMode {
                name: "initial".into(),
                facts: BTreeMap::new(),
                events: vec![],
            }],
            cancel,
        )
        .await
    }
    /// Compile all modes before native admission. Layout, units and state scaling stay fixed.
    pub async fn prepare_simulation_modes(
        &self,
        root: SemanticId,
        instance: SemanticId,
        bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        compiler: Profile,
        profile: native::Profile,
        modes: Vec<ModelingDynamicMode>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        if modes.is_empty()
            || modes.len() > limits.items
            || modes.iter().any(|m| m.name.is_empty())
            || modes.iter().map(|m| &m.name).collect::<BTreeSet<_>>().len() != modes.len()
            || modes
                .iter()
                .try_fold(0usize, |n, m| n.checked_add(m.events.len()))
                .is_none_or(|n| n > limits.items)
        {
            return Err(contract(
                "dynamic modes require unique names and bounded nonempty extent",
            ));
        }
        let names = modes.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
        let mut prepared = None::<ModelingSimulation>;
        let mut identity = FramedHasher::new(pse_ids::Frame::ModelingDynamicModesV1);
        for (index, mode) in modes.iter().enumerate() {
            let mut next = self
                .prepare_simulation_mode(
                    root,
                    instance,
                    bindings.clone(),
                    limits,
                    case.clone(),
                    compiler,
                    profile.clone(),
                    mode,
                    &names,
                    cancel,
                )
                .await?;
            identity.hash(&next.key);
            if let Some(result) = &mut prepared {
                let physical_ports = |model: &ModelingPreparation| {
                    model
                        .compiled()
                        .admitted
                        .case
                        .variables()
                        .iter()
                        .map(|v| (v.port.id, (v.port.quantity, v.port.unit)))
                        .chain(
                            model
                                .compiled()
                                .admitted
                                .case
                                .parameters()
                                .iter()
                                .map(|p| (p.id, (p.quantity, p.unit))),
                        )
                        .collect::<BTreeMap<_, _>>()
                };
                if physical_ports(result.model()) != physical_ports(next.model())
                    || result.contract.states != next.contract.states
                    || result.contract.parameters != next.contract.parameters
                    || result.contract.outputs != next.contract.outputs
                    || result.contract.outputs.iter().any(|id| {
                        result
                            .model()
                            .compiled()
                            .admitted
                            .case
                            .rows()
                            .iter()
                            .find(|r| r.id == *id)
                            .map(|r| r.quantity)
                            != next
                                .model()
                                .compiled()
                                .admitted
                                .case
                                .rows()
                                .iter()
                                .find(|r| r.id == *id)
                                .map(|r| r.quantity)
                    })
                    || result.contract.differential != next.contract.differential
                    || result.contract.quadratures != next.contract.quadratures
                    || result.coordinates != next.coordinates
                    || result.parameters != next.parameters
                {
                    return Err(contract(
                        "dynamic modes require identical state, parameter, output, quadrature and coordinate layouts",
                    ));
                }
                result.bytes = result
                    .bytes
                    .checked_add(next.bytes)
                    .ok_or_else(|| contract("dynamic mode storage extent"))?;
                let programs = result
                    .programs
                    .iter()
                    .cloned()
                    .chain(next.programs.iter().cloned().map(|mut p| {
                        p.mode = index;
                        p
                    }))
                    .collect::<Vec<_>>();
                result.programs = programs.into();
                result.contract.events[index] = std::mem::take(&mut next.contract.events[0]);
                result.modes.extend(next.modes);
            } else {
                prepared = Some(next);
            }
        }
        let mut result = prepared.ok_or_else(|| contract("dynamic mode absent"))?;
        result.key = identity.finish_hash();
        result.contract.identity = result.key;
        result
            .profile
            .validate(&result.contract, &result.parameters)
            .map_err(|e| WorkflowError::Math(e.into()))?;
        Ok(result)
    }
}

type Role = (Function, Vec<SemanticId>, BTreeMap<usize, f64>);
pub(super) fn resolve_events(
    mode: &ModelingDynamicMode,
    names: &[String],
    product: &pse_compiler::workspace::PreparedModeling,
    states: &[SemanticId],
) -> Result<(Vec<native::Event>, Vec<Role>), WorkflowError> {
    let member = |path: &str| -> Result<SemanticId, WorkflowError> {
        if let Some(id) = product.model.paths.get(path) {
            return Ok(*id);
        }
        // An integrated family has one dynamic coordinate. Unindexed family selection
        // is valid only when it resolves to exactly one scalar; never pick a member.
        let prefix = format!("{path}[");
        let mut matches = product
            .model
            .paths
            .iter()
            .filter(|(p, _)| p.starts_with(&prefix))
            .map(|(_, id)| *id);
        let id = matches
            .next()
            .ok_or_else(|| contract(format!("event member path absent: {path}")))?;
        if matches.next().is_some() {
            return Err(contract("event member path is not scalar"));
        }
        Ok(id)
    };
    let mut events = Vec::new();
    let mut roles = Vec::new();
    let mut roots = Vec::new();
    for (index, event) in mode.events.iter().enumerate() {
        let guard = member(&event.guard)?;
        if !event.tolerance.is_finite()
            || event.tolerance <= 0.
            || !matches!(
                product.model.symbols.get(&guard).map(|s| &s.ty),
                Some(pse_modeling::Type::Quantity(_))
            )
        {
            return Err(contract(
                "event requires a physical scalar guard and positive finite tolerance",
            ));
        }
        if event.terminal && (!event.reset.is_empty() || event.next_mode.is_some()) {
            return Err(contract("terminal events cannot reset or change mode"));
        }
        let next_mode = if event.terminal {
            0
        } else {
            names
                .iter()
                .position(|n| Some(n) == event.next_mode.as_ref())
                .ok_or_else(|| contract("event successor mode absent"))?
        };
        roots.push(ModelingOutput::Member(guard).row_id());
        events.push(native::Event {
            id: guard,
            terminal: event.terminal,
            next_mode,
            tolerance: event.tolerance,
            // Authored events are zero crossings in either direction.
            direction: native::Crossing::Either,
        });
        if !event.terminal {
            let mut rows = states
                .iter()
                .map(|id| ModelingOutput::Member(*id).row_id())
                .collect::<Vec<_>>();
            let mut assigned = BTreeSet::new();
            for (state, expression) in &event.reset {
                let state = member(state)?;
                let expression = member(expression)?;
                let at = states
                    .iter()
                    .position(|id| *id == state)
                    .ok_or_else(|| contract("event reset target is not a state"))?;
                if !assigned.insert(state)
                    || product.model.symbols[&state].ty != product.model.symbols[&expression].ty
                {
                    return Err(contract(
                        "event reset requires unique targets with exactly matching physical types",
                    ));
                }
                rows[at] = ModelingOutput::Member(expression).row_id();
            }
            roles.push((Function::Reset(index), rows, BTreeMap::new()));
        }
    }
    if !roots.is_empty() {
        roles.push((Function::Roots, roots, BTreeMap::new()));
    }
    Ok((events, roles))
}
