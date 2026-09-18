// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native finite predicate arguments and typed local graph evaluation.
mod coordinates;
pub(crate) mod inventory;
mod scalar;
use super::invalid;
use crate::{
    AlgorithmContext, AlgorithmInputs, CompilerError,
    mathir_relations::{domain::DomainValue, syntax},
    passes::{
        native_outputs::{self, OutputRows, SourceRole, Sources},
        native_rows::Keyed,
    },
};
pub use coordinates::IndexEvaluator;
pub(crate) use coordinates::SourceGraph;
pub(crate) use inventory::Inventory;
use inventory::{Inputs, Support};
use pse_ids::SemanticId;
use pse_mathir::{NodeId, Payload, ValueRef};
use pse_relations::{
    RecordBatch,
    generated::{enums::TruthValue, inferred as i, normalized as n},
};
use pse_rules::strata::{
    LocatedRuleInput, RuleInputLocation, StratumOutcome, native_input::NativeInput,
};
use pse_schema::model::{AlgorithmSpec, RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Native predicate results retain their actual argument and source owners.
#[derive(Debug)]
pub struct PredicateOutput {
    /// Completed outcomes and ordered axes.
    pub relations: BTreeMap<RelationKey, Arc<NativeInput>>,
    /// Exact source derivations from those completed results.
    pub derivations: Vec<RecordBatch>,
}
/// The four knowledge states, evaluated independently of nullable storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truth {
    /// Positive evidence only.
    True,
    /// Negative evidence only.
    False,
    /// Neither positive nor negative evidence.
    Unknown,
    /// Both positive and negative evidence.
    Conflict,
}
impl Truth {
    fn bits(self) -> (bool, bool) {
        match self {
            Self::True => (true, false),
            Self::False => (false, true),
            Self::Unknown => (false, false),
            Self::Conflict => (true, true),
        }
    }
    fn from_bits(yes: bool, no: bool) -> Self {
        match (yes, no) {
            (true, false) => Self::True,
            (false, true) => Self::False,
            (false, false) => Self::Unknown,
            (true, true) => Self::Conflict,
        }
    }
    fn and(self, other: Self) -> Self {
        let (a, b) = self.bits();
        let (c, d) = other.bits();
        Self::from_bits(a && c, b || d)
    }
    fn or(self, other: Self) -> Self {
        let (a, b) = self.bits();
        let (c, d) = other.bits();
        Self::from_bits(a || c, b && d)
    }
    fn not(self) -> Self {
        let (a, b) = self.bits();
        Self::from_bits(b, a)
    }
}
impl From<bool> for Truth {
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }
}

/// Evaluate each actual source and finite free-index tuple once.
/// # Errors
/// Malformed/cyclic graphs, invalid finite domains, native failures or resource refusal.
pub async fn evaluate(
    ctx: &AlgorithmContext<'_>,
    inputs: &AlgorithmInputs,
    heads: &StratumOutcome,
) -> Result<PredicateOutput, CompilerError> {
    let mut rows = inputs.checked_rows(ctx.registry)?;
    let mut sources = Sources::from_inputs(inputs, ctx.registry)?;
    let mut added = BTreeMap::new();
    for key in heads.completed.keys() {
        let result = heads.completed.relation(key)?;
        rows.insert(key, result.checked().clone());
        added.insert(key, result.checked().clone());
        let port = sources
            .get(&key)
            .map_or_else(|| key.qualified_name(), |(port, _)| port.clone());
        sources.insert(
            key,
            (
                port,
                LocatedRuleInput {
                    relation: key,
                    location: RuleInputLocation::Completed(result),
                },
            ),
        );
    }
    let pass = ctx
        .registry
        .algorithm("P4@1")
        .ok_or_else(|| invalid("P4 declaration absent"))?;
    let base = ctx.session;
    let selected = base
        .input_keys()
        .filter(|key| !added.contains_key(key))
        .collect();
    let session = base
        .select_inputs(&selected, ctx.cancel)?
        .with_checked_workspace(added, ctx.cancel)?;
    evaluate_rows(ctx, pass, &rows, &sources, &session).await
}

#[expect(
    clippy::too_many_lines,
    reason = "evaluate_rows keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(crate) async fn evaluate_rows(
    ctx: &AlgorithmContext<'_>,
    pass: &AlgorithmSpec,
    rows: &Inputs,
    sources: &Sources,
    session: &pse_catalog::session::SnapshotSession,
) -> Result<PredicateOutput, CompilerError> {
    let mut inventory = Inventory::load(
        rows,
        ctx.registry,
        session,
        ctx.reserver,
        ctx.cancel,
        ctx.physical()?,
    )
    .await?;
    let mut output = OutputRows::new(ctx.registry, ctx.reserver, ctx.cancel)?;
    output.ensure::<i::predicate_outcomes::Row>()?;
    output.ensure::<i::predicate_axes::Row>()?;
    // Absence can change an outcome without supplying a positive source row.
    for relation in [
        n::config_values::RELATION_KEY,
        i::instance_features::RELATION_KEY,
        n::domain_members::RELATION_KEY,
        n::instance_domain_bindings::RELATION_KEY,
    ] {
        let (port, _) = sources
            .get(&relation)
            .ok_or_else(|| invalid("predicate read scope needs one exact source role"))?;
        output.read_scope(
            i::predicate_outcomes::RELATION_KEY,
            SourceRole {
                relation,
                port: port.clone(),
            },
        )?;
    }
    let mut graphs = BTreeMap::new();
    for source in inventory.sources.clone() {
        ctx.cancel.checkpoint()?;
        let source_id = source.row.source_id;
        let predicates = inventory
            .predicates
            .iter()
            .filter(|row| row.row.source_id == source_id)
            .map(|row| (row.row.predicate_id, row.clone()))
            .collect::<BTreeMap<_, _>>();
        let count = inventory
            .predicates
            .iter()
            .filter(|row| row.row.source_id == source_id)
            .count();
        if predicates.len() != count {
            return Err(invalid("predicate source repeats its node identity"));
        }
        if predicates.is_empty() {
            continue;
        }
        let family = source.row.family.as_str();
        if !graphs.contains_key(family) {
            let graph_source = crate::mathir_relations::RelationSource::from_checked(
                rows,
                ctx.registry,
                crate::mathir_relations::SourceFamily::Normalized { prefix: family },
            )?;
            let extent = graph_source
                .batches()
                .values()
                .try_fold(0usize, |size, input| {
                    size.checked_add(pse_ids::validation_extent(input.batch())?)
                        .ok_or_else(|| invalid("predicate graph extent overflow"))
                })?;
            inventory.arguments.reserve(
                extent
                    .checked_mul(8)
                    .ok_or_else(|| invalid("predicate graph extent overflow"))?,
            )?;
            graphs.insert(
                family.to_owned(),
                pse_mathir::relations::load_untyped(&graph_source, &[])?,
            );
        }
        let graph = graphs
            .get(family)
            .ok_or_else(|| invalid("predicate graph missing"))?;
        let instances = inventory.source_instances(&source.row, ctx.cancel).await?;
        for (predicate_id, predicate) in &predicates {
            let mut free = BTreeSet::new();
            collect_predicate(
                *predicate_id,
                &predicates,
                graph,
                &mut free,
                &mut BTreeSet::new(),
            )?;
            let mut axes = free
                .into_iter()
                .map(|id| inventory.binder(source_id, id).cloned())
                .collect::<Result<Vec<_>, _>>()?;
            axes.sort_by_key(|binder| {
                (
                    binder.row.position.is_none(),
                    binder.row.position,
                    binder.row.bound_index_id,
                )
            });
            for (position, binder) in axes.iter().enumerate() {
                let support = BTreeSet::from([
                    inventory.origin(&source)?,
                    inventory.origin(predicate)?,
                    inventory.origin(binder)?,
                ]);
                output.push(
                    i::predicate_axes::Row {
                        source_id,
                        predicate_id: *predicate_id,
                        position: i64::try_from(position)
                            .map_err(|_| invalid("predicate axis count exceeds UInt16"))?,
                        bound_index_id: binder.row.bound_index_id,
                        derivation_id: SemanticId::NIL,
                    },
                    &sources.locate(support)?,
                )?;
            }
            for instance in &instances {
                let mut base_support =
                    BTreeSet::from([inventory.origin(&source)?, inventory.origin(instance)?]);
                let mut domains = Vec::with_capacity(axes.len());
                for binder in &axes {
                    base_support.insert(inventory.origin(binder)?);
                    let domain = inventory.domain(
                        &instance.row,
                        &binder.row.domain.domain_ref()?,
                        &mut base_support,
                    )?;
                    inventory.members(domain, &mut base_support)?;
                    domains.push(domain);
                }
                for (tuple, member_support) in inventory.tuples(&domains, ctx.cancel).await? {
                    ctx.cancel.checkpoint()?;
                    let bindings = axes
                        .iter()
                        .zip(&tuple)
                        .map(|(binder, member)| (binder.row.bound_index_id, *member))
                        .collect();
                    let mut support = base_support.clone();
                    support.extend(member_support);
                    let mut evaluation = scalar::Evaluation {
                        inventory: &inventory,
                        graph,
                        instance: &instance.row,
                        source_id,
                        bindings: &bindings,
                        support: &mut support,
                        cancel: ctx.cancel,
                    };
                    let truth = evaluate_predicate(
                        *predicate_id,
                        &predicates,
                        &mut evaluation,
                        &mut BTreeSet::new(),
                    )?;
                    output.push(
                        i::predicate_outcomes::Row {
                            instance_id: instance.row.instance_id,
                            source_id,
                            predicate_id: *predicate_id,
                            index: tuple,
                            outcome: match truth {
                                Truth::True => TruthValue::True,
                                Truth::False => TruthValue::False,
                                Truth::Unknown => TruthValue::Unknown,
                                Truth::Conflict => TruthValue::Conflict,
                            },
                            derivation_id: SemanticId::NIL,
                        },
                        &sources.locate(support)?,
                    )?;
                }
            }
        }
    }
    let relations =
        native_outputs::materialize(output.finish()?, sources, pass, session, ctx.cancel).await?;
    let mut derivations = Vec::new();
    for relation in relations.values() {
        derivations.push(relation.derivations().clone().into_batch());
    }
    Ok(PredicateOutput {
        relations,
        derivations,
    })
}
fn collect_predicate(
    id: i64,
    predicates: &BTreeMap<i64, Keyed<n::predicate_nodes::Row>>,
    graph: &pse_mathir::relations::LoadedMath,
    free: &mut BTreeSet<SemanticId>,
    stack: &mut BTreeSet<i64>,
) -> Result<(), CompilerError> {
    if !stack.insert(id) {
        return Err(invalid("predicate graph cycle"));
    }
    let row = &predicates
        .get(&id)
        .ok_or_else(|| invalid("predicate reference absent"))?
        .row;
    for node in syntax::math_dependencies(&row.value)? {
        collect_node(graph, NodeId(node), free)?;
    }
    for child in syntax::predicate_dependencies(&row.value)? {
        collect_predicate(child, predicates, graph, free, stack)?;
    }
    stack.remove(&id);
    Ok(())
}
fn collect_node(
    graph: &pse_mathir::relations::LoadedMath,
    old: NodeId,
    free: &mut BTreeSet<SemanticId>,
) -> Result<(), CompilerError> {
    let node = *graph
        .node_mapping
        .get(&old)
        .ok_or_else(|| invalid("predicate arithmetic operand absent"))?;
    collect_free_indices(&graph.graph, node, free)
}
fn collect_free_indices(
    graph: &pse_mathir::ExprGraph,
    root: NodeId,
    free: &mut BTreeSet<SemanticId>,
) -> Result<(), CompilerError> {
    let mut pending = vec![(root, BTreeSet::new())];
    let mut visited = BTreeSet::new();
    while let Some((id, mut bound)) = pending.pop() {
        if !visited.insert((id, bound.clone())) {
            continue;
        }
        let node = graph.node(id)?;
        match &node.payload {
            Payload::SymbolRef {
                symbol: ValueRef::Index(index),
            } => {
                if !bound.contains(&index.as_id()) {
                    free.insert(index.as_id());
                }
            }
            Payload::Gather { coordinate_map, .. } => {
                free.extend(
                    coordinate_map
                        .iter()
                        .map(|(index, _)| index.as_id())
                        .filter(|index| !bound.contains(index)),
                );
            }
            Payload::Reduction { bound_index, .. }
            | Payload::Broadcast { bound_index, .. }
            | Payload::Integral { bound_index, .. } => {
                bound.insert(bound_index.as_id());
            }
            _ => {}
        }
        pending.extend(
            node.children
                .iter()
                .copied()
                .chain(node.payload.referenced_nodes())
                .map(|child| (child, bound.clone())),
        );
    }
    Ok(())
}

fn evaluate_predicate(
    id: i64,
    predicates: &BTreeMap<i64, Keyed<n::predicate_nodes::Row>>,
    eval: &mut scalar::Evaluation<'_, '_>,
    stack: &mut BTreeSet<i64>,
) -> Result<Truth, CompilerError> {
    eval.cancel.checkpoint()?;
    if !stack.insert(id) {
        return Err(invalid("predicate graph cycle"));
    }
    let source = predicates
        .get(&id)
        .ok_or_else(|| invalid("predicate reference absent"))?;
    eval.support.insert(eval.inventory.origin(source)?);
    let row = &source.row;
    let truth = match row.value.selected()? {
        syntax::PredicateSelected::Boolean(value) => Truth::from(value.value),
        syntax::PredicateSelected::Null => Truth::Unknown,
        syntax::PredicateSelected::Atom(value) => {
            eval.expression(NodeId(value.expression))?.truth()?
        }
        syntax::PredicateSelected::Compare(value) => {
            let [left, right] = value.operands.as_slice() else {
                return Err(invalid("comparison requires exactly two ordered operands"));
            };
            let left = eval.operand(left)?;
            let right = eval.operand(right)?;
            scalar::compare(
                &left,
                &right,
                value.comparison.as_str(),
                eval.inventory.physical,
            )?
        }
        syntax::PredicateSelected::In(value) => {
            let member = eval.expression(NodeId(value.expression))?;
            eval.contains(&value.domain, &member)?
        }
        syntax::PredicateSelected::And(value) => {
            let left = evaluate_predicate(value.left, predicates, eval, stack)?;
            let right = evaluate_predicate(value.right, predicates, eval, stack)?;
            left.and(right)
        }
        syntax::PredicateSelected::Or(value) => {
            let left = evaluate_predicate(value.left, predicates, eval, stack)?;
            let right = evaluate_predicate(value.right, predicates, eval, stack)?;
            left.or(right)
        }
        syntax::PredicateSelected::Not(value) => {
            evaluate_predicate(value.predicate, predicates, eval, stack)?.not()
        }
    };
    stack.remove(&id);
    Ok(truth)
}

#[cfg(test)]
mod free_index_tests {
    use super::*;
    use pse_mathir::{DomainRef, ExprGraph};
    use pse_quantity::{BoundIndexId, DomainId, Opcode, ReductionKind};

    #[test]
    fn gather_coordinates_are_free_only_outside_their_lexical_binder() {
        let index = BoundIndexId::from_id(SemanticId::from_bytes([1; 16]));
        let mut graph = ExprGraph::new();
        let gather = graph
            .insert(
                Opcode::Gather,
                Payload::Gather {
                    group: SemanticId::from_bytes([2; 16]),
                    coordinate_map: vec![(index, 0)],
                },
                &[],
                None,
            )
            .unwrap();
        let reduction = graph
            .insert(
                Opcode::SumOver,
                Payload::Reduction {
                    kind: ReductionKind::Sum,
                    domain: DomainRef::Actual(DomainId::from_id(SemanticId::from_bytes([3; 16]))),
                    bound_index: index,
                    filter: None,
                },
                &[gather],
                None,
            )
            .unwrap();
        let mut free = BTreeSet::new();
        collect_free_indices(&graph, gather, &mut free).unwrap();
        assert_eq!(free, BTreeSet::from([index.as_id()]));
        let mut closed = BTreeSet::new();
        collect_free_indices(&graph, reduction, &mut closed).unwrap();
        assert!(closed.is_empty());
        // A shared node can occur both under the binder and outside it.
        let shared = graph
            .insert(Opcode::Add, Payload::None, &[gather, reduction], None)
            .unwrap();
        collect_free_indices(&graph, shared, &mut closed).unwrap();
        assert_eq!(closed, free);
    }
}
