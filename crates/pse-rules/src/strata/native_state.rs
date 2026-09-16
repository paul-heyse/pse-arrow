// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed immutable round inventories; all row-state operations are native plans.

use super::relational;
use crate::{RuleError, errmap::internal};
use datafusion::arrow::array::{RecordBatch, StringArray};
use datafusion_expr::{LogicalPlan, LogicalPlanBuilder};
use pse_catalog::session::{PlanObservation, PreparedComputation, SnapshotSession};
use pse_ids::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry,
    model::{ConflictPolicy, RelationKey, RelationSpec, RuleSpec},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(super) struct State {
    pub(super) heads: BTreeMap<RelationKey, RelationKey>,
    pub(super) assertions: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(super) facts: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(super) deltas: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(super) support: FieldCheckedBatch,
    pub(super) plans: Vec<PlanObservation>,
}

pub(super) fn role(kind: &str, key: RelationKey) -> String {
    format!("__pse_{kind}:{}", key.qualified_name())
}

pub(super) fn spec(registry: &Registry, key: RelationKey) -> Result<&RelationSpec, RuleError> {
    registry
        .relation(&key.qualified_name())
        .filter(|spec| spec.key == key)
        .ok_or_else(|| internal("finite state relation declaration is absent"))
}

fn empty(registry: &Registry, spec: &RelationSpec) -> Result<FieldCheckedBatch, RuleError> {
    let schema = pse_schema::arrow::relation_schema(registry, spec)
        .map_err(|error| internal(error.to_string()))?;
    Ok(FieldCheckedBatch::admit(
        registry,
        spec,
        RecordBatch::new_empty(Arc::new(schema)),
    )?)
}

impl State {
    pub(super) fn new(registry: &Registry, rules: &[RuleSpec]) -> Result<Self, RuleError> {
        let mut heads = BTreeMap::new();
        let mut assertions = BTreeMap::new();
        let mut facts = BTreeMap::new();
        for rule in rules {
            let head = registry
                .relation(rule.head.relation())
                .ok_or_else(|| internal("head absent"))?;
            let assertion = registry
                .relation(
                    rule.assertion_relation
                        .as_deref()
                        .ok_or_else(|| internal("assertion absent"))?,
                )
                .ok_or_else(|| internal("assertion absent"))?;
            if heads
                .insert(head.key, assertion.key)
                .is_some_and(|old| old != assertion.key)
            {
                return Err(internal(
                    "one head must share one typed assertion projection",
                ));
            }
            facts.insert(head.key, empty(registry, head)?);
            assertions.insert(head.key, empty(registry, assertion)?);
        }
        let support = empty(
            registry,
            registry
                .relation("provenance.rule_support_edges")
                .ok_or_else(|| internal("support relation absent"))?,
        )?;
        Ok(Self {
            heads,
            assertions,
            deltas: facts.clone(),
            facts,
            support,
            plans: vec![],
        })
    }

    pub(super) fn workspace(
        &self,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, RuleError> {
        let current = session.with_checked_workspace(self.facts.clone(), cancel)?;
        let mut named = BTreeMap::new();
        for (key, batch) in &self.assertions {
            named.insert(role("assertions", *key), batch.clone());
        }
        for (key, batch) in &self.deltas {
            named.insert(role("delta", *key), batch.clone());
        }
        named.insert("__pse_support".to_owned(), self.support.clone());
        Ok(current.with_checked_role_inputs(named, cancel)?)
    }

    pub(super) async fn execute_relation(
        &mut self,
        plan: LogicalPlan,
        target: &RelationSpec,
        session: &SnapshotSession,
        registry: &Registry,
        cancel: &CancellationToken,
    ) -> Result<FieldCheckedBatch, RuleError> {
        let plan = relational::ordered(plan, target)?;
        let plan = pse_catalog::session::output::declare_relation_output(plan, registry, target)
            .map_err(crate::errmap::engine)?;
        let completed = session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?;
        self.plans.push(completed.prepared().observation().clone());
        Ok(completed.into_checked_relation(registry, target, cancel)?)
    }

    pub(super) async fn check_empty(
        &mut self,
        plan: LogicalPlan,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<bool, RuleError> {
        let prepared = empty_probe(plan, session, cancel)?;
        self.completed_empty(prepared, cancel).await
    }

    pub(super) async fn check_immutable_empty(
        &mut self,
        plan: LogicalPlan,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<bool, RuleError> {
        let prepared = empty_probe(plan, session, cancel)?;
        if prepared.contains_volatile_expression() {
            return Ok(false);
        }
        self.completed_empty(prepared, cancel).await
    }

    async fn completed_empty(
        &mut self,
        prepared: PreparedComputation,
        cancel: &CancellationToken,
    ) -> Result<bool, RuleError> {
        let completed = prepared.execute(cancel).await?;
        self.plans.push(completed.prepared().observation().clone());
        Ok(completed
            .batches()
            .iter()
            .all(|batch| batch.num_rows() == 0))
    }

    pub(super) async fn merge_round(
        &mut self,
        candidates: BTreeMap<RelationKey, Vec<LogicalPlan>>,
        support: Vec<LogicalPlan>,
        round: &SnapshotSession,
        registry: &Registry,
        cancel: &CancellationToken,
    ) -> Result<bool, RuleError> {
        let mut changed = false;
        let mut next_facts = self.facts.clone();
        let mut next_deltas = BTreeMap::new();
        let mut next_assertions = self.assertions.clone();
        for (head_key, incoming) in candidates {
            let head = spec(registry, head_key)?;
            let assertion_spec = spec(registry, self.heads[&head_key])?;
            let current = round.scan_role(&role("assertions", head_key))?;
            let merged = relational::union_assertions(
                std::iter::once(current.clone()).chain(incoming),
                assertion_spec,
            )?;
            let assertion_columns = assertion_spec
                .columns
                .iter()
                .filter(|column| column.name() != "assertion_id")
                .map(pse_schema::model::FieldContract::name)
                .collect::<Vec<_>>();
            let merged = self
                .execute_relation(merged, assertion_spec, round, registry, cancel)
                .await?;
            let bound = round.with_checked_role_inputs(
                BTreeMap::from([(role("next_assertions", head_key), merged.clone())]),
                cancel,
            )?;
            let actual_assertions = bound.scan_role(&role("next_assertions", head_key))?;
            let added =
                relational::difference(actual_assertions.clone(), current, &assertion_columns)?;
            changed |= !self.check_empty(added, &bound, cancel).await?;
            let facts = self
                .execute_relation(
                    relational::facts(actual_assertions, head, registry)?,
                    head,
                    &bound,
                    registry,
                    cancel,
                )
                .await?;
            let bound = bound.with_checked_role_inputs(
                BTreeMap::from([(role("next_facts", head_key), facts.clone())]),
                cancel,
            )?;
            let delta = relational::difference(
                bound.scan_role(&role("next_facts", head_key))?,
                round_scan(round, head_key)?,
                &relational::semantic_columns(head),
            )?;
            let delta = self
                .execute_relation(delta, head, &bound, registry, cancel)
                .await?;
            changed |= delta.batch().num_rows() != 0;
            next_assertions.insert(head_key, merged);
            next_facts.insert(head_key, facts);
            next_deltas.insert(head_key, delta);
        }
        for key in self.heads.keys() {
            if !next_deltas.contains_key(key) {
                next_deltas.insert(*key, empty(registry, spec(registry, *key)?)?);
            }
        }
        if !support.is_empty() {
            let current = round.scan_role("__pse_support")?;
            let merged = relational::union(std::iter::once(current.clone()).chain(support))?;
            let target = registry
                .relation("provenance.rule_support_edges")
                .ok_or_else(|| internal("support relation absent"))?;
            let columns = target
                .columns
                .iter()
                .filter(|column| column.name() != "edge_id")
                .map(pse_schema::model::FieldContract::name)
                .collect::<Vec<_>>();
            let merged = self
                .execute_relation(merged, target, round, registry, cancel)
                .await?;
            let bound = round.with_checked_role_inputs(
                BTreeMap::from([("__pse_next_support".to_owned(), merged.clone())]),
                cancel,
            )?;
            changed |= !self
                .check_empty(
                    relational::difference(
                        bound.scan_role("__pse_next_support")?,
                        current,
                        &columns,
                    )?,
                    &bound,
                    cancel,
                )
                .await?;
            self.support = merged;
        }
        self.facts = next_facts;
        self.deltas = next_deltas;
        self.assertions = next_assertions;
        Ok(changed)
    }

    pub(super) async fn complete_stratum(
        &mut self,
        rules: &[&RuleSpec],
        registry: &Registry,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<(), RuleError> {
        let current = self.workspace(session, cancel)?;
        let heads = rules
            .iter()
            .map(|rule| rule.head.relation())
            .collect::<BTreeSet<_>>();
        for name in heads {
            let head = registry
                .relation(name)
                .ok_or_else(|| internal("head absent"))?;
            let assertions = current.scan_role(&role("assertions", head.key))?;
            if !self
                .check_empty(
                    relational::identity_collisions(assertions.clone(), "assertion_id")?,
                    &current,
                    cancel,
                )
                .await?
            {
                return Err(internal(
                    "distinct exact assertion tuples share one label identity",
                ));
            }
            let conflicts = relational::conflicts(assertions, head, registry)?;
            if rules.iter().any(|rule| {
                rule.head.relation() == name && rule.conflict_policy == ConflictPolicy::Reject
            }) {
                let key = LogicalPlanBuilder::from(conflicts)
                    .sort(
                        head.primary_key
                            .iter()
                            .map(|name| datafusion_expr::col(*name).sort(true, true)),
                    )
                    .map_err(crate::errmap::engine)?
                    .project(vec![relational::key(head).alias("key")])
                    .and_then(|builder| builder.limit(0, Some(1)))
                    .and_then(LogicalPlanBuilder::build)
                    .map_err(crate::errmap::engine)?;
                let completed = current
                    .prepare_rule_plan(key, cancel)?
                    .execute(cancel)
                    .await?;
                self.plans.push(completed.prepared().observation().clone());
                if let Some(batch) = completed
                    .batches()
                    .iter()
                    .find(|batch| batch.num_rows() != 0)
                {
                    let keys = batch
                        .column(0)
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .ok_or_else(|| internal("conflict diagnostic is not text"))?;
                    let key = keys.value(0).to_owned();
                    let assertions = self
                        .conflict_assertions(head, &key, &current, cancel)
                        .await?;
                    return Err(RuleError::Conflict {
                        stratum: rules[0].stratum,
                        relation: name.to_owned(),
                        key,
                        assertions,
                    });
                }
            } else {
                let accepted = relational::difference(
                    round_scan(&current, head.key)?,
                    conflicts,
                    &head.primary_key,
                )?;
                let accepted = self
                    .execute_relation(accepted, head, &current, registry, cancel)
                    .await?;
                self.facts.insert(head.key, accepted);
            }
        }
        if !self
            .check_empty(
                relational::identity_collisions(current.scan_role("__pse_support")?, "edge_id")?,
                &current,
                cancel,
            )
            .await?
        {
            return Err(internal(
                "distinct exact support tuples share one label identity",
            ));
        }
        Ok(())
    }

    async fn conflict_assertions(
        &mut self,
        head: &RelationSpec,
        key: &str,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<Vec<String>, RuleError> {
        let plan = LogicalPlanBuilder::from(session.scan_role(&role("assertions", head.key))?)
            .filter(relational::key(head).eq(datafusion_expr::lit(key)))
            .and_then(|builder| {
                builder.sort(vec![datafusion_expr::col("assertion_id").sort(true, true)])
            })
            .and_then(|builder| builder.limit(0, Some(3)))
            .and_then(|builder| {
                builder.project(vec![
                    relational::frame(
                        ["rule_id", "truth"]
                            .into_iter()
                            .chain(relational::semantic_columns(head))
                            .map(datafusion_expr::col)
                            .collect(),
                    )
                    .alias("assertion"),
                ])
            })
            .and_then(LogicalPlanBuilder::build)
            .map_err(crate::errmap::engine)?;
        let completed = session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?;
        self.plans.push(completed.observation().clone());
        let mut assertions = vec![];
        for batch in completed.batches() {
            let values = batch
                .column(0)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| internal("conflict assertion diagnostic is not text"))?;
            assertions.extend(values.iter().flatten().map(str::to_owned));
        }
        Ok(assertions)
    }
}

fn empty_probe(
    plan: LogicalPlan,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<PreparedComputation, RuleError> {
    let plan = LogicalPlanBuilder::from(plan)
        .limit(0, Some(1))
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::errmap::engine)?;
    Ok(session.prepare_rule_plan(plan, cancel)?)
}

pub(super) fn round_scan(
    session: &SnapshotSession,
    key: RelationKey,
) -> Result<LogicalPlan, RuleError> {
    LogicalPlanBuilder::scan(
        session.table_reference(&key)?,
        session.table_source(&key)?,
        None,
    )
    .and_then(LogicalPlanBuilder::build)
    .map_err(crate::errmap::engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_candidate_probe_requires_immutable_expressions_before_elimination() {
        use pse_catalog::session::{
            ExecutionSettings, ThreadBudget, build_candidate_session, native_engine_profile,
        };
        let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
        let one = std::num::NonZeroUsize::new(1).unwrap();
        let session = build_candidate_session(
            BTreeMap::new(),
            registry.clone(),
            Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
            pse_ids::FixedBudget::new(128 << 20),
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: one,
                target_partitions: one,
            },
            native_engine_profile(),
        )
        .unwrap();
        let cancel = CancellationToken::new();
        let mut state = State::new(&registry, &[]).unwrap();
        let volatile = LogicalPlanBuilder::empty(false)
            .project([session
                .scalar_function("random")
                .unwrap()
                .call(vec![])
                .alias("value")])
            .unwrap()
            .build()
            .unwrap();
        assert!(
            !state
                .check_immutable_empty(volatile, &session, &cancel)
                .await
                .unwrap()
        );
        assert!(
            state.plans.is_empty(),
            "a volatile emptiness probe must not execute"
        );
        let immutable = LogicalPlanBuilder::empty(false)
            .project([datafusion_expr::lit(1_i64).alias("value")])
            .unwrap()
            .build()
            .unwrap();
        assert!(
            state
                .check_immutable_empty(immutable, &session, &cancel)
                .await
                .unwrap()
        );
        assert_eq!(
            state.plans.len(),
            1,
            "empty output has an actual completed native observation"
        );
    }
}
