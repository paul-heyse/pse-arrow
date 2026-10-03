// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored modes and events select source expressions and structural facts, never new
//! equations (ADR-0119 Outcome 3).
use super::*;
use pse_modeling::specialize::FixtureMode;

impl ModelingPackage {
    /// Prepare an integrated simulation of one instance under explicit bindings, with its
    /// functions compiled to the `derivatives` order: first order for integration and
    /// sensitivities, second order for exact transient Hessians (ADR-0110 item 4). The
    /// instance's fixture declares its modes and events (ADR-0119 Outcome 3): the first mode
    /// starts the integration, and without any one smooth mode integrates. Every mode is
    /// compiled before native admission; layout, units and state scaling stay fixed.
    #[expect(
        clippy::too_many_arguments,
        reason = "the specialization request (root, instance, bindings, limits) travels with the case, profiles and cancellation as independent inputs"
    )]
    pub async fn prepare_simulation(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        compiler: Profile,
        profile: native::Profile,
        derivatives: DerivativeOrder,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        self.prepare_simulation_for(
            root,
            instance,
            bindings,
            limits,
            case,
            compiler,
            profile,
            derivatives,
            &BTreeSet::new(),
            cancel,
        )
        .await
    }
    /// Prepare the mandatory exact transient consumer before choosing a dynamic method.
    #[expect(
        clippy::too_many_arguments,
        reason = "the dynamic request carries its exact consumer selection"
    )]
    pub(in crate::workflow) async fn prepare_simulation_for(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        compiler: Profile,
        profile: native::Profile,
        derivatives: DerivativeOrder,
        exact_parameters: &BTreeSet<SemanticId>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        if bindings
            .facts
            .keys()
            .any(|k| k.namespace() == pse_modeling::analysis::FactNamespace::Analysis)
            && bindings
                .analysis_route()
                .map_err(|e| contract(e.to_string()))?
                != pse_modeling::analysis::Route::Integrated
        {
            return Err(contract(
                "simulation requires the integrated analysis route",
            ));
        }
        let mut bindings = bindings.with_analysis(pse_modeling::analysis::Route::Integrated);
        bindings
            .demand
            .extend(case.values.keys().chain(case.variables.keys()).cloned());
        bindings.demand.sort();
        bindings.demand.dedup();
        let authored = self
            .prepare(root, instance, bindings.clone(), limits, cancel)
            .await?
            .compiled()
            .model
            .fixtures
            .get(&instance)
            .map(|f| f.modes.clone())
            .unwrap_or_default();
        let modes = if authored.is_empty() {
            vec![FixtureMode {
                name: "initial".into(),
                facts: BTreeMap::new(),
                events: vec![],
            }]
        } else {
            authored
        };
        let snapshot = pse_backend_native::execution::Snapshot::observe(
            &pse_backend_native::execution::LINKED,
        );
        self.prepare_simulation_modes(
            root,
            instance,
            bindings,
            limits,
            case,
            compiler,
            profile,
            derivatives,
            &modes,
            exact_parameters,
            &snapshot,
            cancel,
        )
        .await
    }
    /// Compile all modes before native admission. Layout, units and state scaling stay fixed.
    #[expect(
        clippy::too_many_arguments,
        reason = "the specialization request (root, instance, bindings, limits) travels with the case, profiles, modes and cancellation as independent inputs"
    )]
    async fn prepare_simulation_modes(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        compiler: Profile,
        profile: native::Profile,
        derivatives: DerivativeOrder,
        modes: &[FixtureMode],
        exact_parameters: &BTreeSet<SemanticId>,
        snapshot: &pse_backend_native::execution::Snapshot,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        if modes.is_empty()
            || modes.len() > limits.items
            || modes
                .iter()
                .try_fold(0usize, |n, m| n.checked_add(m.events.len()))
                .is_none_or(|n| n > limits.items)
        {
            return Err(contract("dynamic modes require bounded nonempty extent"));
        }
        let names = modes.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
        let mut prepared = None::<ModelingSimulation>;
        let mut identity = FramedHasher::new(pse_ids::Frame::ModelingDynamicModesV1);
        for (index, mode) in modes.iter().enumerate() {
            // A mode's facts select its `when` variants and stage overrides.
            let mut bindings = bindings.clone();
            for (name, value) in &mode.facts {
                bindings.facts.insert(
                    name.clone(),
                    pse_modeling::specialize::Value::Boolean(*value),
                );
            }
            let mut next = self
                .prepare_simulation_mode(
                    root,
                    instance,
                    bindings,
                    limits,
                    case.clone(),
                    compiler,
                    profile.clone(),
                    derivatives,
                    index,
                    &names,
                    exact_parameters,
                    snapshot,
                    cancel,
                )
                .await?;
            identity.hash(&next.key);
            if let Some(result) = &mut prepared {
                let physical_ports = |model: &ModelingPreparation| {
                    model
                        .compiled()
                        .admitted
                        .case()
                        .variables()
                        .iter()
                        .map(|v| (v.port.id, (v.port.quantity, v.port.unit)))
                        .chain(
                            model
                                .compiled()
                                .admitted
                                .case()
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
                            .case()
                            .rows()
                            .iter()
                            .find(|r| r.id == *id)
                            .map(|r| r.quantity)
                            != next
                                .model()
                                .compiled()
                                .admitted
                                .case()
                                .rows()
                                .iter()
                                .find(|r| r.id == *id)
                                .map(|r| r.quantity)
                    })
                    || result.contract.differential != next.contract.differential
                    || result.contract.signs != next.contract.signs
                    || result.contract.quadratures != next.contract.quadratures
                    || result.contract.balances.len() != next.contract.balances.len()
                    || result
                        .contract
                        .balances
                        .iter()
                        .zip(&next.contract.balances)
                        .any(|(a, b)| {
                            a.id != b.id
                                || a.inventory != b.inventory
                                || a.flux != b.flux
                                || a.tolerance != b.tolerance
                                || result.model().compiled().model.inventory_balances[&a.id].ty
                                    != next.model().compiled().model.inventory_balances[&b.id].ty
                        })
                    || result.coordinates != next.coordinates
                    || result.parameters != next.parameters
                {
                    return Err(contract(
                        "dynamic modes require identical state, sign, parameter, output, quadrature and coordinate layouts",
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
                result.contract.events[index] = std::mem::take(&mut next.contract.events[index]);
                for (balance, next_balance) in result
                    .contract
                    .balances
                    .iter_mut()
                    .zip(next.contract.balances)
                {
                    balance.transfers.extend(next_balance.transfers);
                }
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
/// The native events and reset functions of the mode at `index`, from the fixture of that
/// mode's own specialization; a case without authored modes has none.
pub(super) fn resolve_events(
    product: &pse_compiler::workspace::PreparedModeling,
    instance: InstanceId,
    index: usize,
    states: &[SemanticId],
) -> Result<(Vec<native::Event>, Vec<Role>), WorkflowError> {
    let Some(mode) = product
        .model
        .fixtures
        .get(&instance)
        .and_then(|f| f.modes.get(index))
    else {
        return Ok((Vec::new(), Vec::new()));
    };
    let mut events = Vec::new();
    let mut roles = Vec::new();
    let mut roots = Vec::new();
    for (index, event) in mode.events.iter().enumerate() {
        if event.next.is_none() && !event.reset.is_empty() {
            return Err(contract("a terminal event cannot declare resets"));
        }
        roots.push(ModelingOutput::Member(event.guard).row_id());
        events.push(native::Event {
            id: event.guard,
            terminal: event.next.is_none(),
            next_mode: event.next.unwrap_or(0),
            tolerance: event.tolerance,
            direction: event.direction,
        });
        if event.next.is_some() {
            let mut rows = states
                .iter()
                .map(|id| ModelingOutput::Member(*id).row_id())
                .collect::<Vec<_>>();
            for (state, value) in &event.reset {
                if !product.model.derivatives.contains_key(state) {
                    let mut refusal = pse_model::diagnostic::BoundaryDiagnostic::new(
                        pse_model::diagnostic::BoundaryClass::Unsupported,
                        pse_diagnostics::DiagnosticStage::ModelingEventReset,
                        [event.guard, *state],
                        pse_diagnostics::DiagnosticRule::ModelingDynamicAlgebraicResetUnsupported,
                    );
                    refusal.observations.insert("capability".into(), pse_model::diagnostic::Observation::Text(
                        "an authored algebraic-coordinate reset needs a precise inventory-preserving reset transformation".into(),
                    ));
                    return Err(Box::new(refusal).into());
                }
                let at = states
                    .iter()
                    .position(|id| id == state)
                    .ok_or_else(|| contract("event reset target is not a state"))?;
                rows[at] = ModelingOutput::Member(*value).row_id();
            }
            roles.push((Function::Reset(index), rows, BTreeMap::new()));
            let mut transfer_rows = Vec::new();
            let mut constants = BTreeMap::new();
            for balance in product.model.inventory_balances.values() {
                let row = if balance.transfers.contains_key(&event.guard) {
                    ModelingOutput::InventoryTransfer {
                        balance: balance.id,
                        event: event.guard,
                    }
                    .row_id()
                } else {
                    constants.insert(transfer_rows.len(), 0.0);
                    SemanticId::NIL
                };
                transfer_rows.push(row);
            }
            if constants.len() != transfer_rows.len() {
                roles.push((Function::Transfer(index), transfer_rows, constants));
            }
        }
    }
    if !roots.is_empty() {
        roles.push((Function::Roots, roots, BTreeMap::new()));
    }
    Ok((events, roles))
}
