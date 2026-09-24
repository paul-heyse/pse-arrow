// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Signed physical declarations project into the existing compiler and independent observations.
use super::{ModelDeclaration, SourceDeclarations, WorkflowError, contract};
use pse_ids::SemanticId;
use pse_relations::generated::{
    authored::{computation_models as model, physical_balances as wire},
    enums::BalanceRole,
};
use std::collections::{BTreeMap, BTreeSet};
/// The registry owns balance declarations, including provenance and explicit tolerances.
pub type BalanceDeclaration = wire::Row;
pub(crate) fn sign(role: BalanceRole) -> f64 {
    match role {
        BalanceRole::Inlet
        | BalanceRole::Generation
        | BalanceRole::WorkIn
        | BalanceRole::InternalIn => 1.0,
        BalanceRole::Outlet
        | BalanceRole::Consumption
        | BalanceRole::WorkOut
        | BalanceRole::InternalOut => -1.0,
    }
}
pub(crate) fn mode_row(id: SemanticId, mode: usize) -> SemanticId {
    if mode == 0 {
        id
    } else {
        pse_ids::named_id(id, &format!("mode.{mode}"))
    }
}
/// Charge mode expansion before allocating it; authored bytes alone do not bound this product.
pub(crate) fn projection_bytes(sources: &SourceDeclarations) -> usize {
    sources.balances.iter().fold(0usize, |bytes, balance| {
        let modes = if balance.accumulation.is_some() {
            sources
                .dynamics
                .iter()
                .filter(|d| d.case_id == balance.case_id)
                .map(|d| d.modes.len())
                .max()
                .unwrap_or(1)
        } else {
            1
        };
        let terms = balance.terms.iter().fold(0usize, |count, term| {
            count.saturating_add(if term.mode.is_some() { 1 } else { modes })
        });
        bytes
            .saturating_add(modes.saturating_mul(size_of::<
                model::AuthoredComputationModelsFieldCasesItemRowsItem,
            >()))
            .saturating_add(terms.saturating_mul(size_of::<
                model::AuthoredComputationModelsFieldCasesItemInstancesItemContributionsItem,
            >()))
    })
}
/// Derive equations only in the transient compiler projection. The authored model retains terms.
pub(crate) fn project(
    model: &mut ModelDeclaration,
    sources: &SourceDeclarations,
) -> Result<(), WorkflowError> {
    let mut transfers: BTreeMap<SemanticId, Vec<_>> = BTreeMap::new();
    let mut accumulations = BTreeSet::new();
    for b in &sources.balances {
        if b.terms.is_empty()
            || b.provenance.trim().is_empty()
            || !b.tolerance.is_finite()
            || b.tolerance <= 0.0
            || b.integral_tolerance
                .is_some_and(|v| !v.is_finite() || v <= 0.0)
            || b.accumulation.is_some() != b.integral_tolerance.is_some()
            || b.impulses.iter().any(|i| !i.value.is_finite())
            || b.impulses
                .iter()
                .map(|i| i.event_id)
                .collect::<BTreeSet<_>>()
                .len()
                != b.impulses.len()
            || (b.accumulation.is_none() && !b.impulses.is_empty())
        {
            return Err(contract(
                "physical balance provenance, tolerance, accumulation or impulse contract",
            ));
        }
        let case = model
            .cases
            .iter_mut()
            .find(|c| c.case_id == b.case_id)
            .ok_or_else(|| contract("balance case missing"))?;
        if case.rows.iter().any(|r| r.row_id == b.balance_id) {
            return Err(contract(
                "balance equation must derive solely from contributions",
            ));
        }
        if let Some(state) = b.accumulation {
            if !accumulations.insert((b.case_id, state)) {
                return Err(contract("multiple balance owners for one conserved state"));
            }
            let dynamics: Vec<_> = sources
                .dynamics
                .iter()
                .filter(|d| d.case_id == b.case_id)
                .collect();
            if dynamics.len() != 1
                || !dynamics[0]
                    .states
                    .iter()
                    .any(|s| s.symbol_id == state && s.differential)
                || b.impulses.iter().any(|i| {
                    !dynamics[0]
                        .modes
                        .iter()
                        .flat_map(|m| &m.events)
                        .any(|e| e.event_id == i.event_id && !e.terminal)
                })
            {
                return Err(contract(
                    "balance accumulation requires one declared differential state and actual nonterminal event impulses",
                ));
            }
            let index = dynamics[0]
                .states
                .iter()
                .position(|s| s.symbol_id == state)
                .ok_or_else(|| contract("accumulation state"))?;
            if dynamics[0]
                .modes
                .iter()
                .any(|m| m.rhs_rows.get(index) != Some(&b.balance_id))
            {
                return Err(contract(
                    "conserved state rate must derive from its balance contributions in every mode",
                ));
            }
        }
        let modes = if b.accumulation.is_some() {
            sources
                .dynamics
                .iter()
                .find(|d| d.case_id == b.case_id)
                .map_or(1, |d| d.modes.len())
        } else {
            1
        };
        let mut ids = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        for t in &b.terms {
            if !ids.insert(t.source_id)
                || !outputs.insert((t.instance_id, t.output, t.mode))
                || t.output < 0
                || t.mode
                    .is_some_and(|m| b.accumulation.is_none() || m < 0 || m as usize >= modes)
            {
                return Err(contract(
                    "duplicate balance source/output or invalid ordinal",
                ));
            }
            let internal = matches!(t.role, BalanceRole::InternalIn | BalanceRole::InternalOut);
            if internal != t.transfer_id.is_some() {
                return Err(contract("internal transfer requires explicit identity"));
            }
            if let Some(id) = t.transfer_id {
                transfers.entry(id).or_default().push((
                    b.case_id,
                    b.quantity_id,
                    t.instance_id,
                    t.output,
                    t.mode,
                    sign(t.role),
                ));
            }
            let instance = case
                .instances
                .iter_mut()
                .find(|i| i.instance_id == t.instance_id)
                .ok_or_else(|| contract("balance source instance missing"))?;
            let definition = model
                .definitions
                .iter()
                .find(|d| d.definition_id == instance.definition_id)
                .ok_or_else(|| contract("balance definition missing"))?;
            if t.output as usize >= definition.sources.len() {
                return Err(contract("balance source output missing"));
            }
            for mode in 0..modes {
                if t.mode.is_none_or(|m| m as usize == mode) {
                    instance.contributions.push(model::AuthoredComputationModelsFieldCasesItemInstancesItemContributionsItem { output:t.output, row_id:Some(mode_row(b.balance_id,mode)), scale:sign(t.role) });
                }
            }
        }
        for mode in 0..modes {
            if !b
                .terms
                .iter()
                .any(|t| t.mode.is_none_or(|m| m as usize == mode))
            {
                return Err(contract(
                    "physical balance mode has no declared contributions",
                ));
            }
            let mut seen = BTreeSet::new();
            if b.terms
                .iter()
                .filter(|t| t.mode.is_none_or(|m| m as usize == mode))
                .any(|t| !seen.insert((t.instance_id, t.output)))
            {
                return Err(contract("overlapping physical contribution modes"));
            }
            case.rows
                .push(model::AuthoredComputationModelsFieldCasesItemRowsItem {
                    row_id: mode_row(b.balance_id, mode),
                    quantity_id: b.quantity_id,
                    lower: b.accumulation.is_none().then_some(0.0),
                    upper: b.accumulation.is_none().then_some(0.0),
                });
        }
    }
    for entries in transfers.values() {
        if entries.len() != 2
            || entries[0].0 != entries[1].0
            || entries[0].1 != entries[1].1
            || entries[0].2 != entries[1].2
            || entries[0].3 != entries[1].3
            || entries[0].4 != entries[1].4
            || entries[0].5 != -entries[1].5
        {
            return Err(contract(
                "internal transfers require two opposite roles over the identical typed source output",
            ));
        }
    }
    Ok(())
}
/// Independently sum declared source values; never use a solver residual as conservation evidence.
pub(crate) fn closure(
    b: &BalanceDeclaration,
    values: &[pse_math::assembly::OutputValue],
) -> Result<f64, WorkflowError> {
    let values: BTreeMap<_, _> = values
        .iter()
        .map(|v| ((v.instance, v.output), v.value))
        .collect();
    let mut sum = 0.0;
    for term in &b.terms {
        let value = values
            .get(&(term.instance_id, term.output as usize))
            .ok_or_else(|| contract("physical source output unavailable"))?;
        sum += sign(term.role) * value;
    }
    if !sum.is_finite() {
        return Err(contract("nonfinite physical closure"));
    }
    Ok(sum)
}

/// One independently computed physical check, separate from mathematical fit quality.
#[derive(Clone, Debug)]
pub struct BalanceCheck {
    /// Original balance declaration.
    pub balance: SemanticId,
    /// Experiment ordinal in the source fit declaration.
    pub experiment: usize,
    /// Completed sample ordinal; zero for steady states.
    pub sample: usize,
    /// Physical elapsed time, absent for steady states.
    pub time: Option<f64>,
    /// Closure in the declared canonical quantity, or an attributable evaluation failure.
    pub value: Result<f64, String>,
}
pub(crate) fn dynamic_closure(
    balance: &pse_backend_native::dynamics::Balance,
    index: usize,
    report: &pse_backend_native::dynamics::Report,
    sample: &pse_backend_native::dynamics::Sample,
) -> Result<f64, WorkflowError> {
    let value = sample
        .balance_integrals
        .get(index)
        .zip(report.consistent_initial.get(balance.state))
        .zip(sample.state.get(balance.state))
        .map(|((integral, initial), state)| {
            let impulse: f64 = report
                .events
                .iter()
                .filter(|e| e.time <= sample.time && e.after.is_some())
                .filter_map(|e| e.event.and_then(|id| balance.impulses.get(&id)))
                .sum();
            (state - initial) * balance.scale - integral - impulse
        });
    value
        .filter(|v| v.is_finite())
        .ok_or_else(|| contract("native integrated physical output missing"))
}

impl super::RunResult {
    pub(super) fn physical_checks(
        &self,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::physical_checks as checks;
        let mut out =
            checks::Builder::with_registry(&self.runtime.registry, 0).map_err(super::relation)?;
        let mut push = |revision: &super::ModelRevision,
                        b: &BalanceDeclaration,
                        step: usize,
                        sample: usize,
                        time: Option<f64>,
                        result: Result<f64, WorkflowError>| {
            let quantity = if let Some(state) = b.accumulation {
                revision.0.cases[&b.case_id].cases[&b.case_id]
                    .structure
                    .variables()
                    .iter()
                    .find(|v| v.port.id == state)
                    .ok_or_else(|| contract("conserved result state missing"))?
                    .port
                    .quantity
            } else {
                b.quantity_id.into()
            };
            let unit = revision
                .0
                .physical
                .quantities
                .quantity_type(quantity)
                .map_err(super::math)?
                .canonical_unit;
            let tolerance = b.integral_tolerance.unwrap_or(b.tolerance);
            out.push(checks::Row {
                run_id: self.run_id,
                step: step as i64,
                sample: sample as i64,
                balance_id: b.balance_id,
                quantity_id: quantity.as_id(),
                unit_id: unit.as_id(),
                time,
                closure: result.as_ref().ok().copied(),
                tolerance,
                accepted: result.as_ref().ok().map(|v| v.abs() <= tolerance),
                error: result.err().map(|e| e.to_string()),
                provenance: b.provenance.clone(),
            })
            .map_err(super::relation)
        };
        match &self.request {
            super::RunRequest::Solves(steps) => {
                for (i, s) in steps.iter().enumerate() {
                    let observation = match &self.report {
                        Ok(super::RunReport::Solves(r)) => match r.outcomes.get(i) {
                            Some(crate::math::solves::Outcome::Native(n)) => n.observation.as_ref(),
                            Some(crate::math::solves::Outcome::Constant(c)) => Some(&c.observation),
                            _ => None,
                        },
                        _ => None,
                    };
                    for b in s
                        .revision
                        .0
                        .sources
                        .balances
                        .iter()
                        .filter(|b| b.case_id == s.case)
                    {
                        let value = if b.accumulation.is_some() {
                            Err(contract("dynamic accumulation requires simulation"))
                        } else {
                            observation
                                .ok_or_else(|| contract("physical observation unavailable"))
                                .and_then(|o| closure(b, &o.sources))
                        };
                        push(&s.revision, b, i, 0, None, value)?;
                    }
                }
            }
            super::RunRequest::Simulation(p) => {
                for (j, balance) in p.contract.balances.iter().enumerate() {
                    let b = p
                        .revision
                        .0
                        .sources
                        .balances
                        .iter()
                        .find(|b| b.balance_id == balance.id)
                        .ok_or_else(|| contract("balance source missing"))?;
                    if let Ok(super::RunReport::Simulation(r)) = &self.report {
                        for (i, s) in r.samples.iter().enumerate() {
                            let value = dynamic_closure(balance, j, r, s);
                            push(&p.revision, b, 0, i, Some(s.time), value)?;
                        }
                        if r.samples.is_empty() {
                            push(
                                &p.revision,
                                b,
                                0,
                                0,
                                None,
                                Err(contract("no completed physical samples")),
                            )?;
                        }
                    } else {
                        push(
                            &p.revision,
                            b,
                            0,
                            0,
                            None,
                            Err(contract("physical trajectory unavailable")),
                        )?;
                    }
                }
            }
            super::RunRequest::Fit(p) => {
                for (ei, experiment) in p.problem.declaration.experiments.iter().enumerate() {
                    for b in p
                        .problem
                        .revision
                        .0
                        .sources
                        .balances
                        .iter()
                        .filter(|b| b.case_id == experiment.case_id)
                    {
                        let values = match &self.report {
                            Ok(super::RunReport::Fit(r)) => r
                                .physical
                                .iter()
                                .filter(|v| v.experiment == ei && v.balance == b.balance_id)
                                .collect::<Vec<_>>(),
                            _ => vec![],
                        };
                        if values.is_empty() {
                            push(
                                &p.problem.revision,
                                b,
                                ei,
                                0,
                                None,
                                Err(contract("physical fit observation unavailable")),
                            )?;
                        }
                        for v in values {
                            push(
                                &p.problem.revision,
                                b,
                                ei,
                                v.sample,
                                v.time,
                                v.value.clone().map_err(contract),
                            )?;
                        }
                    }
                }
            }
        }
        out.finish().map_err(super::relation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contribution_authority_derives_equations_and_independent_closure() {
        let mut model = super::super::tests::declaration();
        let case = &mut model.cases[0];
        let instance = case.instances[0].instance_id;
        let row = case.rows.remove(0);
        case.instances[0].contributions.clear();
        let b = BalanceDeclaration {
            model_id: model.model_id,
            case_id: case.case_id,
            balance_id: row.row_id,
            quantity_id: row.quantity_id,
            accumulation: None,
            tolerance: 1e-5,
            integral_tolerance: None,
            provenance: "unit physical declaration".into(),
            terms: vec![wire::AuthoredPhysicalBalancesFieldTermsItem {
                source_id: instance,
                role: BalanceRole::Outlet,
                mode: None,
                transfer_id: None,
                instance_id: instance,
                output: 0,
            }],
            impulses: vec![],
        };
        let sources = SourceDeclarations {
            balances: vec![b.clone()],
            ..Default::default()
        };
        project(&mut model, &sources).unwrap();
        assert_eq!(model.cases[0].rows[0].lower, Some(0.0));
        assert_eq!(model.cases[0].instances[0].contributions[0].scale, -1.0);
        assert_eq!(
            closure(
                &b,
                &[pse_math::assembly::OutputValue {
                    instance,
                    output: 0,
                    value: 3.0
                }]
            )
            .unwrap(),
            -3.0
        );
        assert!(closure(&b, &[]).is_err());
        assert!(project(&mut model, &sources).is_err()); // No duplicate equation authority.
        let mut bad = b.clone();
        bad.terms[0].role = BalanceRole::InternalIn;
        let mut model = super::super::tests::declaration();
        model.cases[0].rows.clear();
        model.cases[0].instances[0].contributions.clear();
        assert!(
            project(
                &mut model,
                &SourceDeclarations {
                    balances: vec![bad],
                    ..Default::default()
                }
            )
            .is_err()
        );
        let mut raw = super::super::tests::declaration();
        raw.cases[0].rows.clear();
        raw.cases[0].instances[0].contributions.clear();
        // Mode-dependent fluxes derive separate rows while retaining one balance owner.
        let state = raw.cases[0].variables[0].port.symbol_id;
        let mut b = b;
        b.accumulation = Some(state);
        b.integral_tolerance = Some(1e-5);
        let mut terms = b.terms.clone();
        terms[0].source_id = pse_ids::named_id(instance, "mode-one");
        terms[0].mode = Some(1);
        terms[0].role = BalanceRole::Inlet;
        b.terms[0].mode = Some(0);
        b.terms.extend(terms);
        let dynamic = serde_json::from_value(serde_json::json!({
            "dynamic_id": instance, "model_id": raw.model_id, "case_id": b.case_id,
            "time_id": state, "states":[{"symbol_id":state,"differential":true,
            "initial_row":b.balance_id,"offset":0.0,"scale":1.0,"residual_scale":1.0}],
            "parameters":[],"outputs":[], "modes":[
                {"rhs_rows":[b.balance_id],"events":[]},
                {"rhs_rows":[b.balance_id],"events":[]}
            ]
        }))
        .unwrap();
        let sources = SourceDeclarations {
            balances: vec![b.clone()],
            dynamics: vec![dynamic],
            ..Default::default()
        };
        let extent = projection_bytes(&sources);
        assert!(extent >= 2 * size_of::<model::AuthoredComputationModelsFieldCasesItemRowsItem>());
        let mut projected = raw.clone();
        project(&mut projected, &sources).unwrap();
        assert_eq!(projected.cases[0].rows.len(), 2);
        let contributions = &projected.cases[0].instances[0].contributions;
        assert_eq!(contributions[0].row_id, Some(b.balance_id));
        assert_eq!(contributions[0].scale, -1.0);
        assert_eq!(contributions[1].row_id, Some(mode_row(b.balance_id, 1)));
        assert_eq!(contributions[1].scale, 1.0);
        let mut duplicate = sources.clone();
        duplicate.balances[0].balance_id = pse_ids::named_id(b.balance_id, "duplicate");
        duplicate.balances.push(b);
        assert!(project(&mut raw, &duplicate).is_err());
    }
}
