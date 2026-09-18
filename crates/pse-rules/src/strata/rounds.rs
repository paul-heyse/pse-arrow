// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One prepared native program per stratum; execution only swaps settled epochs.

use super::{
    RuleBindings,
    native_state::{State, empty, role, round_scan, spec},
    native_support, relational,
};
use crate::{
    RuleError,
    errmap::{engine, internal},
    plan::{PortBinding, compile_query, delta::input_deltas, trace::compile_support},
};
use datafusion_common::TableReference;
use datafusion_expr::{LogicalPlan, LogicalPlanBuilder};
use pse_catalog::session::{ReusableComputation, SnapshotSession, round::RoundInputs};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry,
    model::{RelationKey, RelationSpec, RuleSpec},
};
use std::collections::BTreeMap;

struct Query {
    evolving: bool,
    candidate_role: String,
    support_role: String,
    assertion: RelationKey,
    empty_candidate: FieldCheckedBatch,
    empty_support: FieldCheckedBatch,
    probe: ReusableComputation,
    candidates: ReusableComputation,
    support: Option<ReusableComputation>,
}
struct Head {
    key: RelationKey,
    assertion: RelationKey,
    merged: ReusableComputation,
    added: ReusableComputation,
    facts: ReusableComputation,
    delta: ReusableComputation,
}
/// Only this owner may advance the private round inputs. Every execution method
/// drains completely, and a failed execution cannot be resumed or replanned.
pub(super) struct Program {
    inputs: RoundInputs,
    queries: Vec<Query>,
    heads: Vec<Head>,
    support: ReusableComputation,
    added_support: ReusableComputation,
}
impl Program {
    pub(super) async fn prepare(
        active: &[&RuleSpec],
        bindings: &BTreeMap<SemanticId, RuleBindings>,
        state: &State,
        session: &SnapshotSession,
        registry: &Registry,
        cancel: &CancellationToken,
    ) -> Result<Self, RuleError> {
        let support_spec = registry
            .relation("provenance.rule_support_edges")
            .ok_or_else(|| internal("support declaration absent"))?;
        let mut roles = initial_roles(state);
        let mut targets: BTreeMap<RelationKey, Vec<String>> = BTreeMap::new();
        let mut support_roles = vec![];
        for rule in active {
            let head = registry
                .relation(rule.head.as_str())
                .ok_or_else(|| internal("head absent"))?;
            let assertion = spec(registry, state.heads[&head.key])?;
            for index in 0..rule.queries.len() {
                let name = format!("__pse_candidate:{}:{index}", rule.id);
                roles.insert(name.clone(), empty(registry, assertion)?);
                targets.entry(head.key).or_default().push(name);
                let name = format!("__pse_witness:{}:{index}", rule.id);
                roles.insert(name.clone(), empty(registry, support_spec)?);
                support_roles.push(name);
            }
        }
        let (session, inputs) = session.with_round_inputs(state.facts.clone(), roles, cancel)?;
        let deltas = active
            .iter()
            .map(|rule| {
                let target = registry
                    .relation(rule.head.as_str())
                    .ok_or_else(|| internal("head absent"))?;
                Ok((
                    TableReference::partial(target.key.namespace.as_str(), target.key.name),
                    session.scan_role(&role("delta", target.key))?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, RuleError>>()?;
        let queries =
            prepare_queries(active, bindings, state, &session, registry, cancel, &deltas).await?;
        let mut heads = vec![];
        for (key, incoming) in targets {
            let head = spec(registry, key)?;
            let assertion = spec(registry, state.heads[&key])?;
            let current = session.scan_role(&role("assertions", key))?;
            let scans = incoming
                .iter()
                .map(|name| session.scan_role(name))
                .collect::<Result<Vec<_>, _>>()?;
            let merged = relational::union_assertions(
                std::iter::once(current.clone()).chain(scans),
                assertion,
            )?;
            let next_assertions = session.scan_role(&role("next_assertions", key))?;
            let columns = assertion
                .columns
                .iter()
                .filter(|column| column.name() != "assertion_id")
                .map(pse_schema::model::FieldContract::name)
                .collect::<Vec<_>>();
            let added = relational::difference(next_assertions.clone(), current, &columns)?;
            let facts = relational::facts(next_assertions, head, registry)?;
            let delta = relational::difference(
                session.scan_role(&role("next_facts", key))?,
                round_scan(&session, key)?,
                &relational::semantic_columns(head),
            )?;
            heads.push(Head {
                key,
                assertion: assertion.key,
                merged: prepare_relation(merged, assertion, &session, registry, cancel).await?,
                added: prepare_probe(added, &session, cancel).await?,
                facts: prepare_relation(facts, head, &session, registry, cancel).await?,
                delta: prepare_relation(delta, head, &session, registry, cancel).await?,
            });
        }
        let current = session.scan_role("__pse_support")?;
        let support_scans = support_roles
            .iter()
            .map(|name| session.scan_role(name))
            .collect::<Result<Vec<_>, _>>()?;
        let merged = relational::union(std::iter::once(current.clone()).chain(support_scans))?;
        let columns = support_spec
            .columns
            .iter()
            .filter(|column| column.name() != "edge_id")
            .map(pse_schema::model::FieldContract::name)
            .collect::<Vec<_>>();
        let added =
            relational::difference(session.scan_role("__pse_next_support")?, current, &columns)?;
        Ok(Self {
            inputs,
            queries,
            heads,
            support: prepare_relation(merged, support_spec, &session, registry, cancel).await?,
            added_support: prepare_probe(added, &session, cancel).await?,
        })
    }
    async fn execute_queries(
        &mut self,
        state: &mut State,
        registry: &Registry,
        cancel: &CancellationToken,
        ordinal: u32,
    ) -> Result<(), RuleError> {
        let support_spec = registry
            .relation("provenance.rule_support_edges")
            .ok_or_else(|| internal("support absent"))?;
        for query in &mut self.queries {
            if (ordinal != 1 && !query.evolving)
                || execute_empty(&mut query.probe, state, cancel).await?
            {
                self.inputs
                    .replace(&query.candidate_role, query.empty_candidate.clone())
                    .map_err(engine)?;
                self.inputs
                    .replace(&query.support_role, query.empty_support.clone())
                    .map_err(engine)?;
                continue;
            }
            let candidates = execute_relation(
                &mut query.candidates,
                spec(registry, query.assertion)?,
                state,
                registry,
                cancel,
            )
            .await?;
            self.inputs
                .replace(&query.candidate_role, candidates)
                .map_err(engine)?;
            if let Some(program) = &mut query.support {
                let support =
                    execute_relation(program, support_spec, state, registry, cancel).await?;
                self.inputs
                    .replace(&query.support_role, support)
                    .map_err(engine)?;
            }
        }
        Ok(())
    }
    pub(super) async fn execute(
        &mut self,
        state: &mut State,
        registry: &Registry,
        cancel: &CancellationToken,
        ordinal: u32,
    ) -> Result<bool, RuleError> {
        let started = std::time::Instant::now();
        self.execute_queries(state, registry, cancel, ordinal)
            .await?;
        tracing::debug!(
            round = ordinal,
            elapsed_seconds = started.elapsed().as_secs_f64(),
            "finite rule candidates and support executed"
        );
        let started = std::time::Instant::now();
        let support_spec = registry
            .relation("provenance.rule_support_edges")
            .ok_or_else(|| internal("support absent"))?;
        let mut changed = false;
        let mut next_facts = state.facts.clone();
        let mut next_assertions = state.assertions.clone();
        let mut next_deltas = state
            .heads
            .keys()
            .map(|key| Ok((*key, empty(registry, spec(registry, *key)?)?)))
            .collect::<Result<BTreeMap<_, _>, RuleError>>()?;
        for head in &mut self.heads {
            let merged = execute_relation(
                &mut head.merged,
                spec(registry, head.assertion)?,
                state,
                registry,
                cancel,
            )
            .await?;
            self.inputs
                .replace(&role("next_assertions", head.key), merged.clone())
                .map_err(engine)?;
            changed |= !execute_empty(&mut head.added, state, cancel).await?;
            let facts = execute_relation(
                &mut head.facts,
                spec(registry, head.key)?,
                state,
                registry,
                cancel,
            )
            .await?;
            self.inputs
                .replace(&role("next_facts", head.key), facts.clone())
                .map_err(engine)?;
            let delta = execute_relation(
                &mut head.delta,
                spec(registry, head.key)?,
                state,
                registry,
                cancel,
            )
            .await?;
            changed |= delta.batch().num_rows() != 0;
            next_assertions.insert(head.key, merged);
            next_facts.insert(head.key, facts);
            next_deltas.insert(head.key, delta);
        }
        tracing::debug!(
            round = ordinal,
            elapsed_seconds = started.elapsed().as_secs_f64(),
            "finite rule heads merged"
        );
        let started = std::time::Instant::now();
        let support =
            execute_relation(&mut self.support, support_spec, state, registry, cancel).await?;
        self.inputs
            .replace("__pse_next_support", support.clone())
            .map_err(engine)?;
        changed |= !execute_empty(&mut self.added_support, state, cancel).await?;
        for (key, input) in &next_facts {
            self.inputs
                .replace(&key.qualified_name(), input.clone())
                .map_err(engine)?;
        }
        for (key, input) in &next_assertions {
            self.inputs
                .replace(&role("assertions", *key), input.clone())
                .map_err(engine)?;
        }
        for (key, input) in &next_deltas {
            self.inputs
                .replace(&role("delta", *key), input.clone())
                .map_err(engine)?;
        }
        self.inputs
            .replace("__pse_support", support.clone())
            .map_err(engine)?;
        state.facts = next_facts;
        state.assertions = next_assertions;
        state.deltas = next_deltas;
        state.support = support;
        tracing::debug!(
            round = ordinal,
            elapsed_seconds = started.elapsed().as_secs_f64(),
            "finite rule support merged and epoch advanced"
        );
        Ok(changed)
    }
}
fn initial_roles(state: &State) -> BTreeMap<String, FieldCheckedBatch> {
    let mut roles = BTreeMap::from([
        ("__pse_support".into(), state.support.clone()),
        ("__pse_next_support".into(), state.support.clone()),
    ]);
    for (key, value) in &state.assertions {
        roles.insert(role("assertions", *key), value.clone());
        roles.insert(role("next_assertions", *key), value.clone());
    }
    for (key, value) in &state.facts {
        roles.insert(role("next_facts", *key), value.clone());
    }
    for (key, value) in &state.deltas {
        roles.insert(role("delta", *key), value.clone());
    }
    roles
}

async fn prepare_queries(
    active: &[&RuleSpec],
    bindings: &BTreeMap<SemanticId, RuleBindings>,
    state: &State,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
    deltas: &BTreeMap<TableReference, LogicalPlan>,
) -> Result<Vec<Query>, RuleError> {
    let support_spec = registry
        .relation("provenance.rule_support_edges")
        .ok_or_else(|| internal("support declaration absent"))?;
    let mut queries = vec![];
    for rule in active {
        let ports = bindings
            .get(&rule.id)
            .ok_or_else(|| internal("rule ports absent"))?;
        let binding = PortBinding {
            ports: ports
                .ports
                .iter()
                .map(|(name, input)| (name.clone(), input.relation))
                .collect(),
        };
        let target = registry
            .relation(rule.head.as_str())
            .ok_or_else(|| internal("head absent"))?;
        let assertion = spec(registry, state.heads[&target.key])?;
        let evolving = rule.inputs.iter().any(|input| {
            active
                .iter()
                .any(|producer| producer.head == input.relation)
        });
        for (index, query) in rule.queries.iter().enumerate() {
            let compiled = compile_query(rule, query, &binding, session, registry, cancel).await?;
            let variants = input_deltas(&compiled, deltas)?;
            let variants = if variants.is_empty() {
                vec![compiled.clone()]
            } else {
                variants
            };
            let candidates = variants
                .into_iter()
                .map(|variant| relational::assertions(variant.plan, rule, query.truth, registry))
                .collect::<Result<Vec<_>, _>>()?;
            let candidates = relational::union_assertions(candidates, assertion)?;
            let support = compile_support(&compiled.plan, rule, registry, session, cancel)?
                .into_iter()
                .map(|trace| {
                    native_support::plan(trace, rule, query.truth, ports, session, registry)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let support = if support.is_empty() {
                None
            } else {
                tracing::debug!(rule = %rule.id, query = index, branches = support.len(), "preparing native rule support");
                Some(
                    prepare_relation(
                        relational::union(support)?,
                        support_spec,
                        session,
                        registry,
                        cancel,
                    )
                    .await?,
                )
            };
            queries.push(Query {
                evolving,
                candidate_role: format!("__pse_candidate:{}:{index}", rule.id),
                support_role: format!("__pse_witness:{}:{index}", rule.id),
                assertion: assertion.key,
                empty_candidate: empty(registry, assertion)?,
                empty_support: empty(registry, support_spec)?,
                probe: prepare_probe(compiled.plan, session, cancel).await?,
                candidates: prepare_relation(candidates, assertion, session, registry, cancel)
                    .await?,
                support,
            });
        }
    }
    Ok(queries)
}

async fn prepare_relation(
    plan: LogicalPlan,
    target: &RelationSpec,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<ReusableComputation, RuleError> {
    let plan = relational::ordered(plan, target)?;
    let plan = pse_catalog::session::output::declare_relation_output(plan, registry, target)
        .map_err(engine)?;
    Ok(session
        .prepare_rule_plan(plan, cancel)?
        .prepare_reusable(cancel)
        .await?)
}
async fn prepare_probe(
    plan: LogicalPlan,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<ReusableComputation, RuleError> {
    let plan = LogicalPlanBuilder::from(plan)
        .limit(0, Some(1))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    Ok(session
        .prepare_rule_plan(plan, cancel)?
        .prepare_reusable(cancel)
        .await?)
}
async fn execute_empty(
    program: &mut ReusableComputation,
    state: &mut State,
    cancel: &CancellationToken,
) -> Result<bool, RuleError> {
    let completed = program.execute(cancel).await?;
    state.plans.push(completed.observation().clone());
    Ok(completed
        .batches()
        .iter()
        .all(|batch| batch.num_rows() == 0))
}
async fn execute_relation(
    program: &mut ReusableComputation,
    target: &RelationSpec,
    state: &mut State,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<FieldCheckedBatch, RuleError> {
    let completed = program.execute(cancel).await?;
    state.plans.push(completed.observation().clone());
    Ok(completed.into_checked_relation(registry, target, cancel)?)
}
