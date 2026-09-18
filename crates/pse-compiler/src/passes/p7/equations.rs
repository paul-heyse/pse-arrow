// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid};
use crate::{CompilerError, mathir_relations::syntax::EquationSelected};
use pse_ids::{IndexTuple, SemanticId, equation_instance_id};
use pse_mathir::{
    NodeId, Opcode, Payload,
    equation::{EquationRecord, FreeIndex},
};
use pse_quantity::BoundIndexId;
use pse_relations::generated::{compiled, inferred, normalized};
use pse_schema::math::Sense;
use pse_templates::InstantiationEnvironment;
use std::collections::BTreeSet;

struct Leaf {
    id: i64,
    left: NodeId,
    right: NodeId,
    sense: Sense,
    guards: Vec<(i64, bool)>,
}

impl Realizer<'_> {
    pub(super) fn equation(
        &mut self,
        instance: &inferred::instances::Row,
        declaration: SemanticId,
        name: &str,
        axes: &[String],
        has_filter: bool,
        source: &normalized::expression_sources::Row,
    ) -> Result<(), CompilerError> {
        let env = self.prepare(instance, source)?;
        let leaves = self.equation_leaves(source, &env)?;
        let domains = self.domains(instance.instance_id, axes)?;
        let product = self.product(&domains)?;
        let free_indices = self.equation_indices(source.source_id, &domains, &env)?;
        let authored_filter = if has_filter {
            let filter_source = self.source(
                source.source_relation_id,
                "equation_decl_id",
                declaration,
                "filter",
            )?;
            let filter_env = self.prepare(instance, &filter_source)?;
            if filter_env.free_indices != env.free_indices {
                return Err(invalid(
                    "filter source binders differ from the exact equation declaration",
                ));
            }
            let first_new = self.graph.len();
            let filter = filter_env.predicate_guard(
                filter_source.source_id,
                filter_source.root_id,
                &mut self.graph,
                &env.free_indices,
            )?;
            for ordinal in first_new..self.graph.len() {
                self.node_support
                    .entry(NodeId(
                        i64::try_from(ordinal).map_err(|_| invalid("graph ordinal overflow"))?,
                    ))
                    .or_default()
                    .extend(self.active_support.iter().copied());
            }
            Some(filter)
        } else {
            None
        };
        let parent = equation_instance_id(instance.instance_id, declaration, IndexTuple::EMPTY);
        for leaf in leaves {
            let indexed = !leaf.guards.is_empty();
            let id = if indexed {
                pse_ids::named_id(
                    parent,
                    &format!(
                        "pse:equation-branch:v1:{}:{}",
                        source.source_id.to_hex(),
                        leaf.id
                    ),
                )
            } else {
                parent
            };
            let filter =
                self.branch_filter(source.source_id, &leaf.guards, authored_filter, &env)?;
            if indexed {
                self.append("compiled.equation_branches", compiled::equation_branches::Row {
                    indexed_equation_id: id, parent_indexed_equation_id: parent, instance_id: instance.instance_id,
                    equation_decl_id: declaration, source_id: source.source_id, equation_node_id: leaf.id,
                    branch_guards: leaf.guards.iter().map(|(predicate_id, expected)| compiled::equation_branches::CompiledEquationBranchesFieldBranchGuardsItem {
                        source_id: source.source_id, predicate_id: *predicate_id, expected: *expected,
                    }).collect(), derivation_id: Self::derivation(instance.instance_id, source.source_id, "equation_branch"),
                })?;
            }
            self.equation_support
                .entry(id)
                .or_default()
                .extend(self.active_support.iter().copied());
            self.equations.push(EquationRecord {
                indexed_equation_id: id,
                owner_instance: instance.instance_id,
                equation_decl: Some(declaration),
                qualified_name: if indexed {
                    format!("{}.{name}:branch:{}", instance.path, leaf.id)
                } else {
                    format!("{}.{name}", instance.path)
                },
                product: Some(product),
                filter,
                body: leaf.left,
                sense: leaf.sense,
                lower: matches!(leaf.sense, Sense::Ge).then_some(leaf.right),
                upper: (!matches!(leaf.sense, Sense::Ge)).then_some(leaf.right),
                free_indices: free_indices.clone(),
                residual_quantity_type: None,
                law_instance: None,
                derivation: Self::derivation(instance.instance_id, source.source_id, "equation"),
            });
        }
        Ok(())
    }

    fn equation_indices(
        &self,
        source: SemanticId,
        domains: &[pse_quantity::DomainId],
        env: &InstantiationEnvironment,
    ) -> Result<Vec<FreeIndex>, CompilerError> {
        let mut indices = self
            .inventory
            .indices
            .iter()
            .filter(|row| row.source_id == source && row.position.is_some())
            .collect::<Vec<_>>();
        indices.sort_by_key(|row| row.position);
        if indices.len() != domains.len() {
            return Err(invalid(
                "equation source axes differ from its exact declaration",
            ));
        }
        indices
            .into_iter()
            .zip(domains)
            .enumerate()
            .map(|(position, (index, domain))| {
                let binding = env
                    .binders
                    .get(&BoundIndexId::from_id(index.bound_index_id))
                    .ok_or_else(|| invalid("equation free binder absent"))?;
                if index.position.and_then(|value| usize::try_from(value).ok()) != Some(position)
                    || binding.domain != *domain
                {
                    return Err(invalid("equation source axis/domain mismatch"));
                }
                Ok(FreeIndex {
                    bound_index: binding.bound_index,
                    domain: *domain,
                    position: u16::try_from(position)
                        .map_err(|_| invalid("equation axis overflow"))?,
                })
            })
            .collect()
    }

    fn equation_leaves(
        &mut self,
        source: &normalized::expression_sources::Row,
        env: &InstantiationEnvironment,
    ) -> Result<Vec<Leaf>, CompilerError> {
        let nodes = self
            .inventory
            .equation_nodes
            .iter()
            .filter(|row| row.source_id == source.source_id)
            .map(|row| (row.equation_id, row.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut tasks = vec![(source.root_id, Vec::new(), BTreeSet::new())];
        let mut leaves = Vec::new();
        while let Some((id, guards, mut ancestors)) = tasks.pop() {
            self.cancel.checkpoint()?;
            if !ancestors.insert(id) {
                return Err(invalid("conditional equation source graph is cyclic"));
            }
            let node = nodes
                .get(&id)
                .ok_or_else(|| invalid("equation source node absent"))?;
            match node.value.selected()? {
                EquationSelected::Relation(value) => {
                    let roots = [value.left, value.right];
                    let mapped = self.instantiate_roots(source, &roots, env)?;
                    let sense = Sense::parse(value.sense.as_str())
                        .ok_or_else(|| invalid("equation sense unknown"))?;
                    leaves.push(Leaf {
                        id,
                        left: mapped[0],
                        right: mapped[1],
                        sense,
                        guards,
                    });
                }
                EquationSelected::Conditional(value) => {
                    let guard = value.guard;
                    let yes = value.then;
                    let no = value.otherwise;
                    if let Some(value) = env.predicates.get(&(source.source_id, guard)) {
                        tasks.push((if *value { yes } else { no }, guards, ancestors));
                    } else if env.predicate_masks.contains_key(&(source.source_id, guard)) {
                        let mut rejected = guards.clone();
                        rejected.push((guard, false));
                        let mut accepted = guards;
                        accepted.push((guard, true));
                        tasks.push((no, rejected, ancestors.clone()));
                        tasks.push((yes, accepted, ancestors));
                    } else {
                        return Err(pse_templates::TemplateError::GuardUndecidable {
                            instance: env.instance,
                            source_id: source.source_id,
                            predicate: guard,
                        }
                        .into());
                    }
                }
            }
        }
        Ok(leaves)
    }

    fn branch_filter(
        &mut self,
        source: SemanticId,
        guards: &[(i64, bool)],
        authored: Option<NodeId>,
        env: &InstantiationEnvironment,
    ) -> Result<Option<NodeId>, CompilerError> {
        if guards.is_empty() {
            return Ok(authored);
        }
        let quantity = self.boolean_types_for_instance(env.instance)?;
        let zero = self.graph.insert_typed(
            Opcode::Const,
            Payload::IntConst { value: 0 },
            &[],
            quantity,
            Some(env.instance),
        )?;
        let one = self.graph.insert_typed(
            Opcode::Const,
            Payload::IntConst { value: 1 },
            &[],
            quantity,
            Some(env.instance),
        )?;
        let mut result = authored.unwrap_or(one);
        for (predicate, expected) in guards.iter().rev() {
            let guard =
                env.predicate_guard(source, *predicate, &mut self.graph, &env.free_indices)?;
            let children = if *expected {
                [result, zero]
            } else {
                [zero, result]
            };
            result = self.graph.insert(
                Opcode::Conditional,
                Payload::Conditional {
                    guard: guard.into(),
                },
                &children,
                Some(env.instance),
            )?;
        }
        for node in [zero, one, result] {
            self.node_support
                .entry(node)
                .or_default()
                .extend(self.active_support.iter().copied());
        }
        Ok(Some(result))
    }
}
