// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Provenance selects actual typed arguments and graph-node source occurrences.
use super::{Realizer, Support, invalid};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::NodeId;
use pse_relations::generated::{inferred as i, normalized as n};
use pse_schema::model::RelationKey;
use std::collections::{BTreeMap, BTreeSet};

impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "context keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn context(
        &mut self,
        instance: SemanticId,
        source: Option<SemanticId>,
    ) -> Result<(), CompilerError> {
        let actual = self
            .inventory
            .instances
            .iter()
            .find(|row| row.instance_id == instance)
            .ok_or_else(|| invalid("provenance instance absent"))?;
        let template = actual.template_id;
        let domains = self
            .inventory
            .domain_bindings
            .iter()
            .filter(|row| row.instance_id == instance)
            .map(|row| row.domain_id)
            .collect::<BTreeSet<_>>();
        let products = self
            .inventory
            .products
            .iter()
            .filter(|row| row.domain_ids.iter().all(|id| domains.contains(id)))
            .map(|row| row.product_id)
            .collect::<BTreeSet<_>>();
        let symbols = self
            .inventory
            .symbols
            .iter()
            .filter(|row| row.template_id == template)
            .map(|row| row.symbol_decl_id)
            .collect::<BTreeSet<_>>();
        let contributions = self
            .inventory
            .contributions
            .iter()
            .filter(|row| row.template_id == template)
            .map(|row| row.contribution_decl_id)
            .collect::<BTreeSet<_>>();
        let mut support = BTreeSet::new();
        macro_rules! select {
            ($field:ident, $relation:expr, $row:ident, $condition:expr) => {
                for (position, $row) in self.inventory.$field.iter().enumerate() {
                    if $condition {
                        support.insert(($relation, position));
                    }
                }
            };
        }
        select!(
            instances,
            i::instances::RELATION_KEY,
            row,
            row.instance_id == instance
        );
        select!(
            prospective,
            n::instance_bindings::RELATION_KEY,
            row,
            row.instance_id == instance
        );
        select!(
            domain_bindings,
            n::instance_domain_bindings::RELATION_KEY,
            row,
            row.instance_id == instance
        );
        select!(
            feature_values,
            i::instance_features::RELATION_KEY,
            row,
            row.instance_id == instance
        );
        select!(
            configuration,
            n::config_values::RELATION_KEY,
            row,
            row.owner_id == instance
        );
        select!(
            templates,
            n::templates::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            symbols,
            n::template_symbols::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            parameters,
            n::template_params::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            features,
            n::template_features::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            ports,
            n::template_ports::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            submodels,
            n::template_submodels::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            equations,
            n::template_equations::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            bodies,
            n::template_symbol_expressions::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            contributions,
            n::template_contributions::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            scopes,
            n::template_scopes::RELATION_KEY,
            row,
            row.template_id == template
        );
        select!(
            contracts,
            n::template_symbol_contracts::RELATION_KEY,
            row,
            symbols.contains(&row.symbol_decl_id)
        );
        select!(
            derivatives,
            n::template_derivatives::RELATION_KEY,
            row,
            symbols.contains(&row.symbol_decl_id)
        );
        select!(
            contribution_contracts,
            n::template_contribution_contracts::RELATION_KEY,
            row,
            contributions.contains(&row.contribution_decl_id)
        );
        select!(
            sources,
            n::expression_sources::RELATION_KEY,
            row,
            Some(row.source_id) == source
        );
        select!(
            indices,
            n::expression_index_bindings::RELATION_KEY,
            row,
            Some(row.source_id) == source
        );
        select!(
            equation_nodes,
            n::equation_nodes::RELATION_KEY,
            row,
            Some(row.source_id) == source
        );
        select!(
            paths,
            n::expression_paths::RELATION_KEY,
            row,
            Some(row.source_id) == source
        );
        select!(
            predicate_axes,
            i::predicate_axes::RELATION_KEY,
            row,
            Some(row.source_id) == source
        );
        select!(
            predicates,
            i::predicate_outcomes::RELATION_KEY,
            row,
            Some(row.source_id) == source && row.instance_id == instance
        );
        select!(
            targets,
            i::path_targets::RELATION_KEY,
            row,
            Some(row.source_id) == source && row.requester_instance_id == instance
        );
        select!(
            domains,
            n::domains::RELATION_KEY,
            row,
            domains.contains(&row.domain_id)
        );
        select!(
            members,
            n::domain_members::RELATION_KEY,
            row,
            domains.contains(&row.domain_id)
        );
        select!(
            products,
            n::domain_products::RELATION_KEY,
            row,
            products.contains(&row.product_id)
        );
        select!(
            tuples,
            i::valid_index_tuples::RELATION_KEY,
            row,
            products.contains(&row.product_id)
        );
        select!(
            scope_bindings,
            i::scope_bindings::RELATION_KEY,
            row,
            row.owner_instance_id == instance
        );
        if let Some(evaluator) = &self.coordinate_evaluator {
            for predicate in &evaluator.inventory.predicates {
                if Some(predicate.row.source_id) == source {
                    support.insert(
                        self.source_position(n::predicate_nodes::RELATION_KEY, &predicate.key)?,
                    );
                }
            }
        }
        for key in crate::quantity_relations::inventory::input_keys(self.registry) {
            let keys = self
                .inventory
                .source_keys
                .get(&key)
                .ok_or_else(|| invalid(format!("physical source projection {key} absent")))?;
            support.extend((0..keys.len()).map(|position| (key, position)));
        }
        if support.is_empty() {
            return Err(invalid("realization context has no actual supporting rows"));
        }
        self.active_support = support;
        Ok(())
    }

    pub(super) fn source_position(
        &self,
        relation: RelationKey,
        key: &pse_ids::ContentHash,
    ) -> Result<Support, CompilerError> {
        self.inventory
            .source_keys
            .get(&relation)
            .and_then(|keys| keys.iter().position(|actual| actual == key))
            .map(|position| (relation, position))
            .ok_or_else(|| invalid("source key is outside its actual native projection"))
    }

    pub(super) fn source_node_evidence(
        &mut self,
        source: &n::expression_sources::Row,
        mapped: &BTreeMap<NodeId, NodeId>,
    ) -> Result<(), CompilerError> {
        let family = self
            .families
            .get(source.family.as_str())
            .ok_or_else(|| invalid("source graph family absent"))?;
        let evaluator = self
            .coordinate_evaluator
            .as_ref()
            .ok_or_else(|| invalid("shared coordinate/source evaluator absent"))?;
        let mut occurrences = Vec::new();
        for ((name, old), origins) in &evaluator.inventory.node_origins {
            if name != source.family.as_str() {
                continue;
            }
            if let Some(output) = family
                .node_mapping
                .get(old)
                .and_then(|node| mapped.get(node))
            {
                for (relation, key) in origins {
                    occurrences.push((*output, self.source_position(*relation, key)?));
                }
            }
        }
        for (output, source) in occurrences {
            self.node_support.entry(output).or_default().insert(source);
        }
        Ok(())
    }

    pub(super) fn remember_symbol(&mut self, symbol: SemanticId) {
        self.symbol_support
            .entry(symbol)
            .or_default()
            .extend(self.active_support.iter().copied());
    }
}
