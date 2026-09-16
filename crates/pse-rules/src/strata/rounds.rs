// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native candidate programs over exact current and per-occurrence delta bindings.

use super::{
    RuleBindings,
    native_state::{State, role},
    native_support, relational,
};
use crate::{
    RuleError,
    errmap::internal,
    plan::{
        PortBinding, compile_bound, delta::input_deltas, outcomes::candidates as candidate_plans,
        trace::compile_support_bound,
    },
};
use datafusion_common::TableReference;
use datafusion_expr::LogicalPlan;
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_schema::{
    Registry,
    model::{RelationKey, RuleSpec},
};
use std::collections::BTreeMap;

pub(super) struct Round<'a> {
    pub(super) active: &'a [&'a RuleSpec],
    pub(super) bindings: &'a BTreeMap<SemanticId, RuleBindings>,
    pub(super) session: &'a SnapshotSession,
    pub(super) registry: &'a Registry,
    pub(super) cancel: &'a CancellationToken,
    pub(super) ordinal: u32,
    pub(super) max_rounds: u32,
}

impl Round<'_> {
    pub(super) async fn execute(&self, state: &mut State) -> Result<bool, RuleError> {
        let mut candidates: BTreeMap<RelationKey, Vec<LogicalPlan>> = BTreeMap::new();
        let mut supports = vec![];
        let mut execution = self.session.clone();
        let deltas = self.delta_sources()?;
        for rule in self.active {
            tracing::debug!(
                rule = rule.name,
                round = self.ordinal,
                "rule candidates started"
            );
            self.cancel
                .checkpoint()
                .map_err(pse_catalog::CatalogError::from)?;
            let evolving = rule.plan.dependencies().iter().any(|(relation, _, _)| {
                self.active
                    .iter()
                    .any(|producer| producer.head.relation() == *relation)
            });
            // Fixed input scopes already produced their complete assertions and
            // witnesses in round one. Do not rebuild their private intermediates.
            if self.ordinal != 1 && !evolving {
                continue;
            }
            let (ports, binding, target) = self.rule_bindings(rule)?;
            let (native, extended, observations) = crate::plan::closure::prepare(
                rule,
                &binding,
                &execution,
                self.registry,
                crate::plan::closure::RoundScope {
                    max_rounds: self.max_rounds,
                    ordinal: self.ordinal,
                    evolving,
                },
                self.cancel,
            )
            .await?;
            execution = extended;
            state.plans.extend(observations);
            let nested_input = native.has_results() && evolving;
            for (truth, plan) in candidate_plans(&rule.plan) {
                let mut candidate = (*rule).clone();
                candidate.plan = plan;
                let compiled =
                    compile_bound(&candidate, &binding, &execution, self.registry, &native)
                        .map_err(|source| RuleError::Execution {
                            rule: rule.qualified_name(),
                            phase: "candidate construction",
                            source: Box::new(source),
                        })?;
                let variants = input_deltas(&compiled, &deltas)?;
                let variants = if variants.is_empty() {
                    if self.ordinal != 1 && !nested_input {
                        continue;
                    }
                    vec![compiled.clone()]
                } else {
                    variants
                };
                if self
                    .candidate_is_empty(state, &compiled, &execution, rule)
                    .await?
                {
                    continue;
                }
                for variant in variants {
                    candidates
                        .entry(target.key)
                        .or_default()
                        .push(relational::assertions(
                            variant.plan,
                            rule,
                            truth,
                            self.registry,
                        )?);
                }
                // Every source witness is retained, including an additional support for
                // a fact already present. These are native joins over the current facts;
                // candidate insertion alone uses one input-delta occurrence per variant.
                for trace in
                    compile_support_bound(&candidate, &binding, &execution, self.registry, &native)?
                {
                    supports.push(native_support::plan(
                        trace,
                        rule,
                        truth,
                        ports,
                        &execution,
                        self.registry,
                    )?);
                }
            }
        }
        tracing::info!(
            round = self.ordinal,
            heads = candidates.len(),
            support_plans = supports.len(),
            "finite rule round merging"
        );
        state
            .merge_round(candidates, supports, &execution, self.registry, self.cancel)
            .await
    }

    fn delta_sources(&self) -> Result<BTreeMap<TableReference, LogicalPlan>, RuleError> {
        let deltas = self
            .active
            .iter()
            .map(|rule| {
                let target = self
                    .registry
                    .relation(rule.head.relation())
                    .ok_or_else(|| internal("active head absent"))?;
                Ok((
                    self.session.table_reference(&target.key)?,
                    self.session.scan_role(&role("delta", target.key))?,
                ))
            })
            .collect::<Result<BTreeMap<TableReference, LogicalPlan>, RuleError>>()?;
        Ok(deltas)
    }

    async fn candidate_is_empty(
        &self,
        state: &mut State,
        compiled: &crate::plan::CompiledRule,
        execution: &SnapshotSession,
        rule: &RuleSpec,
    ) -> Result<bool, RuleError> {
        for (check, reason) in &compiled.checks {
            if !state
                .check_empty(check.clone(), execution, self.cancel)
                .await?
            {
                return Err(internal(reason.clone()));
            }
        }
        // No assertion or support edge can exist when the complete current
        // candidate is empty. Establish that with the native query before
        // constructing its witness joins. Checking only the delta would lose
        // additional witnesses for facts that are already present.
        state
            .check_immutable_empty(compiled.plan.clone(), execution, self.cancel)
            .await
            .map_err(|source| RuleError::Execution {
                rule: rule.qualified_name(),
                phase: "complete candidate emptiness",
                source: Box::new(source),
            })
    }

    fn rule_bindings(
        &self,
        rule: &RuleSpec,
    ) -> Result<(&RuleBindings, PortBinding, &pse_schema::model::RelationSpec), RuleError> {
        let ports = self
            .bindings
            .get(&rule.id)
            .ok_or_else(|| internal("rule ports absent"))?;
        let binding = PortBinding {
            ports: ports
                .ports
                .iter()
                .map(|(name, input)| (name.clone(), input.relation))
                .collect(),
        };
        let target = self
            .registry
            .relation(rule.head.relation())
            .ok_or_else(|| internal("rule head absent"))?;
        Ok((ports, binding, target))
    }
}
