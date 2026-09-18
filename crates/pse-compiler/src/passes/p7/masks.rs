// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid};
use crate::CompilerError;
use pse_ids::{IndexTuple, SemanticId, symbol_instance_id};
use pse_mathir::{Opcode, Payload};
use pse_quantity::{BoundIndexId, QuantityTypeKey, ScaleKind};
use pse_relations::generated::{
    compiled,
    enums::{BoundKind, SolverVariableType, SymbolRole, VariableLifecycle, VariableSemanticRole},
    extension_values::Bound,
    inferred,
};
use pse_templates::{GroupBinding, InstantiationEnvironment, PredicateMask};
use std::collections::{BTreeMap, BTreeSet};

impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "bind_masks keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn bind_masks(
        &mut self,
        instance: &inferred::instances::Row,
        source: SemanticId,
        env: &mut InstantiationEnvironment,
    ) -> Result<(), CompilerError> {
        let predicate_ids = self
            .inventory
            .predicate_axes
            .iter()
            .filter(|axis| axis.source_id == source)
            .map(|axis| axis.predicate_id)
            .collect::<BTreeSet<_>>();
        for predicate in predicate_ids {
            let mut rows = self
                .inventory
                .predicate_axes
                .iter()
                .filter(|axis| axis.source_id == source && axis.predicate_id == predicate)
                .collect::<Vec<_>>();
            rows.sort_by_key(|row| row.position);
            let mut domains = Vec::new();
            let mut coordinates = Vec::new();
            for (position, row) in rows.into_iter().enumerate() {
                if usize::try_from(row.position).ok() != Some(position) {
                    return Err(invalid(
                        "predicate axes are not a complete ordered coordinate map",
                    ));
                }
                let source_binding = BoundIndexId::from_id(row.bound_index_id);
                let binding = env
                    .binders
                    .get(&source_binding)
                    .ok_or_else(|| invalid("predicate axis source binding absent"))?;
                domains.push(binding.domain);
                coordinates.push(source_binding);
            }
            let product = self.product(&domains)?;
            let outcomes = self
                .inventory
                .predicates
                .iter()
                .filter(|outcome| {
                    outcome.instance_id == instance.instance_id
                        && outcome.source_id == source
                        && outcome.predicate_id == predicate
                })
                .cloned()
                .collect::<Vec<_>>();
            if outcomes
                .iter()
                .any(|row| !matches!(row.outcome.as_str(), "true" | "false"))
            {
                continue;
            }
            let expected = self
                .inventory
                .tuples
                .iter()
                .filter(|row| row.product_id == product)
                .map(|row| row.tuple.clone())
                .collect::<BTreeSet<_>>();
            let actual = outcomes
                .iter()
                .map(|row| row.index.clone())
                .collect::<BTreeSet<_>>();
            if actual != expected || actual.len() != outcomes.len() {
                return Err(invalid(
                    "predicate mask outcomes do not cover exact actual finite domain tuples",
                ));
            }
            let (scalar, shaped) = self.boolean_types(&domains)?;
            let group = pse_ids::named_id(
                instance.instance_id,
                &format!("pse:predicate-mask:v1:{}:{predicate}", source.to_hex()),
            );
            let mut members = BTreeMap::new();
            for outcome in outcomes {
                self.check_tuple(&domains, &outcome.index)?;
                let symbol =
                    symbol_instance_id(instance.instance_id, group, IndexTuple(&outcome.index));
                let value = outcome.outcome.as_str() == "true";
                let node = self.graph.insert_typed(
                    Opcode::Const,
                    Payload::IntConst {
                        value: i64::from(value),
                    },
                    &[],
                    scalar,
                    Some(instance.instance_id),
                )?;
                self.node_support
                    .entry(node)
                    .or_default()
                    .extend(self.active_support.iter().copied());
                let unit = self.physical.quantity_type(scalar)?.canonical_unit;
                let symbol_row = compiled::symbols::Row {
                    symbol_id: symbol,
                    ordinal: 0,
                    owner_instance_id: instance.instance_id,
                    symbol_decl_id: group,
                    qualified_name: format!(
                        "{}.predicate:{}:{predicate}:{:?}",
                        instance.path,
                        source.to_hex(),
                        outcome.index
                    ),
                    index: outcome.index.clone(),
                    quantity_type_id: scalar.as_id(),
                    unit_id: unit.as_id(),
                    role: SymbolRole::Expression,
                    solver_type: SolverVariableType::Binary,
                    semantic_role: VariableSemanticRole::ReportingOnly,
                    lifecycle: VariableLifecycle::GeneratedSemantic,
                    default_lower: Bound {
                        kind: BoundKind::Unbounded,
                        value: None,
                    },
                    default_upper: Bound {
                        kind: BoundKind::Unbounded,
                        value: None,
                    },
                    default_initial: None,
                    derivation_id: Self::derivation(
                        instance.instance_id,
                        source,
                        "predicate_member",
                    ),
                };
                self.remember_symbol(symbol_row.symbol_id);
                let key = (instance.instance_id, group, outcome.index.clone());
                if self
                    .symbols
                    .insert(key, symbol_row.clone())
                    .is_some_and(|previous| previous != symbol_row)
                {
                    return Err(invalid("actual predicate symbol member values conflict"));
                }
                self.append(
                    "compiled.symbol_expressions",
                    compiled::symbol_expressions::Row {
                        symbol_id: symbol,
                        node_id: node.0,
                        derivation_id: Self::derivation(
                            instance.instance_id,
                            source,
                            "predicate_body",
                        ),
                    },
                )?;
                self.append(
                    "compiled.expression_roots",
                    compiled::expression_roots::Row {
                        owner_id: symbol,
                        role: pse_relations::generated::enums::ExpressionRootRole::SymbolExpression,
                        ordinal: 0,
                        node_id: node.0,
                        derivation_id: Self::derivation(
                            instance.instance_id,
                            source,
                            "predicate_body_root",
                        ),
                    },
                )?;
                self.append(
                    "compiled.predicate_mask_members",
                    compiled::predicate_mask_members::Row {
                        group_id: group,
                        index: outcome.index.clone(),
                        symbol_id: symbol,
                        node_id: node.0,
                        value,
                        derivation_id: Self::derivation(
                            instance.instance_id,
                            source,
                            "predicate_member",
                        ),
                    },
                )?;
                self.append(
                    "compiled.symbol_group_members",
                    compiled::symbol_group_members::Row {
                        group_id: group,
                        tuple: outcome.index.clone(),
                        symbol_id: symbol,
                    },
                )?;
                members.insert(outcome.index, symbol);
            }
            self.append(
                "compiled.predicate_masks",
                compiled::predicate_masks::Row {
                    group_id: group,
                    instance_id: instance.instance_id,
                    source_id: source,
                    predicate_id: predicate,
                    product_id: product,
                    quantity_type_id: shaped.as_id(),
                    derivation_id: Self::derivation(instance.instance_id, source, "predicate_mask"),
                },
            )?;
            self.append(
                "compiled.symbol_groups",
                compiled::symbol_groups::Row {
                    group_id: group,
                    owner_instance_id: instance.instance_id,
                    name: format!("predicate:{}:{predicate}", source.to_hex()),
                    product_id: product,
                },
            )?;
            env.predicate_masks.insert(
                (source, predicate),
                PredicateMask {
                    group: GroupBinding {
                        group,
                        axes: domains,
                        members,
                    },
                    coordinates,
                },
            );
        }
        Ok(())
    }
    pub(super) fn boolean_types_for_instance(
        &self,
        instance: SemanticId,
    ) -> Result<pse_quantity::QuantityTypeId, CompilerError> {
        self.inventory
            .instances
            .iter()
            .find(|row| row.instance_id == instance)
            .ok_or_else(|| invalid("Boolean occurrence instance missing"))?;
        Ok(self.boolean_types(&[])?.0)
    }
    fn boolean_types(
        &self,
        domains: &[pse_quantity::DomainId],
    ) -> Result<(pse_quantity::QuantityTypeId, pse_quantity::QuantityTypeId), CompilerError> {
        // The native physical inventory has already resolved the complete selected
        // package context and rejected conflicting definitions. Consumers need not
        // redeclare the context owned by their physical dependency package.
        let boolean = self.boolean_kind.ok_or_else(|| {
            invalid("predicate requires an explicit selected mathematical context")
        })?;
        let mut key = QuantityTypeKey {
            kind: boolean,
            basis: None,
            reference_state: None,
            scale_kind: ScaleKind::Point,
            shape: Vec::new(),
            subject_kind: None,
        };
        let scalar = self.physical.resolve_key(&key)?;
        key.shape = domains
            .iter()
            .map(|domain| {
                self.domain_facts
                    .get(domain)
                    .map(|facts| facts.kind)
                    .ok_or_else(|| invalid("predicate domain facts absent"))
            })
            .collect::<Result<_, _>>()?;
        Ok((scalar, self.physical.resolve_key(&key)?))
    }
}
