// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P4 evaluates declared finite feature/material/capability rules and typed guards.
mod augmented;
pub mod predicates;
pub(crate) use augmented::evaluate_augmented;

use crate::AlgorithmOutput;
use crate::{AlgorithmContext, AlgorithmInputs, CompilerError};
use pse_rules::strata::{
    LocatedRuleInput, RuleBindings, RuleInputLocation, StratumLimits, StratumOutcome,
};
use pse_schema::{
    Registry,
    model::{AlgorithmSpec, RelationKey, RuleSpec},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU32,
};

/// Executable capability closure bound to its registered complete pass contract.
#[derive(Debug)]
pub struct P4 {
    spec: AlgorithmSpec,
}
impl P4 {
    /// Bind the exact registered P4 declaration.
    /// # Errors
    /// The declaration is unavailable.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        Ok(Self {
            spec: registry
                .algorithm("P4@1")
                .ok_or_else(|| invalid("P4 declaration missing"))?
                .clone(),
        })
    }
}
impl crate::Algorithm for P4 {
    fn spec(&self) -> &AlgorithmSpec {
        &self.spec
    }
    fn run<'a>(
        &'a self,
        ctx: &'a AlgorithmContext<'a>,
        inputs: &'a AlgorithmInputs,
    ) -> pse_catalog::provider::BoxFut<'a, Result<AlgorithmOutput, CompilerError>> {
        Box::pin(async move {
            inputs.validate(&self.spec, ctx.registry)?;
            let mut outcome = execute_program(&self.spec, ctx, inputs).await?;
            let predicate = predicates::evaluate(ctx, inputs, &outcome).await?;
            outcome.derivations.extend(predicate.derivations);
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let spec = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| invalid("P4 output undeclared"))?;
                    let batch = if let Some(input) = predicate.relations.get(&spec.key) {
                        input.checked().clone()
                    } else {
                        outcome.completed.relation(spec.key)?.checked().clone()
                    };
                    Ok((port.port.clone(), batch))
                })
                .collect::<Result<_, CompilerError>>()?;
            Ok(AlgorithmOutput {
                outputs: ports,
                findings: Vec::new(),
                derivations: outcome.derivations,
                plans: Vec::new(),
            })
        })
    }
}

/// Execute the complete producer inventory selected by a pass's declared head outputs.
/// Every external scan is paired with its actual immutable snapshot member.
/// # Errors
/// Missing bindings, unfinished closure, conflicts or rule admission/execution failures.
pub(crate) async fn execute_program(
    spec: &AlgorithmSpec,
    ctx: &AlgorithmContext<'_>,
    inputs: &AlgorithmInputs,
) -> Result<StratumOutcome, CompilerError> {
    let session = ctx.session;
    execute_selected_program(spec, spec, ctx, inputs, session, &BTreeMap::new()).await
}

pub(crate) async fn execute_selected_program(
    selection: &AlgorithmSpec,
    spec: &AlgorithmSpec,
    ctx: &AlgorithmContext<'_>,
    inputs: &AlgorithmInputs,
    session: &pse_catalog::session::SnapshotSession,
    locations: &BTreeMap<RelationKey, RuleInputLocation>,
) -> Result<StratumOutcome, CompilerError> {
    let declared = selection
        .outputs
        .iter()
        .map(|port| port.relation.as_str())
        .collect::<BTreeSet<_>>();
    let rules = ctx
        .registry
        .rules()
        .iter()
        .filter(|rule| declared.contains(rule.head.as_str()))
        .cloned()
        .collect::<Vec<RuleSpec>>();
    let heads = rules
        .iter()
        .map(|rule| rule.head.as_str())
        .collect::<BTreeSet<_>>();
    let mut outputs = BTreeSet::<RelationKey>::new();
    let mut bindings = BTreeMap::new();
    for name in ["inferred.rule_outcomes", "provenance.rule_support_edges"] {
        outputs.insert(
            ctx.registry
                .relation(name)
                .ok_or_else(|| invalid("rule evidence contract missing"))?
                .key,
        );
    }
    for rule in &rules {
        for name in [Some(rule.head.as_str()), rule.assertion_relation.as_deref()]
            .into_iter()
            .flatten()
        {
            outputs.insert(
                ctx.registry
                    .relation(name)
                    .ok_or_else(|| invalid("rule head/assertion contract missing"))?
                    .key,
            );
        }
        let mut ports = BTreeMap::new();
        for input in &rule.inputs {
            let name = input.relation.as_str();
            let port = input.port;
            let relation = ctx
                .registry
                .relation(name)
                .ok_or_else(|| invalid("rule scan undeclared"))?;
            let location = if heads.contains(name) {
                RuleInputLocation::Workspace
            } else if let Some(location) = locations.get(&relation.key) {
                location.clone()
            } else {
                let input_port = spec
                    .inputs
                    .iter()
                    .find(|input| input.relation == name)
                    .ok_or_else(|| invalid(format!("pass has no input for rule scan {name}")))?;
                let bound = inputs
                    .port(&input_port.port)
                    .and_then(Option::as_ref)
                    .ok_or_else(|| invalid("rule requires a complete present input binding"))?;
                RuleInputLocation::Facts(std::sync::Arc::clone(bound.relation()?))
            };
            ports.insert(
                port.to_owned(),
                LocatedRuleInput {
                    relation: relation.key,
                    location,
                },
            );
        }
        bindings.insert(rule.id, RuleBindings { ports });
    }
    let retained = session
        .input_keys()
        .filter(|key| !outputs.contains(key))
        .collect();
    let session = session.select_inputs(&retained, ctx.cancel)?;
    Ok(pse_rules::strata::execute_strata(
        &rules,
        &bindings,
        &session,
        ctx.registry,
        &outputs,
        StratumLimits {
            max_rounds: NonZeroU32::new(u32::MAX)
                .ok_or_else(|| invalid("invalid finite closure limit"))?,
        },
        ctx.cancel,
    )
    .await?)
}
pub(super) fn invalid(reason: impl Into<String>) -> CompilerError {
    pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.into(),
    }
    .into()
}
