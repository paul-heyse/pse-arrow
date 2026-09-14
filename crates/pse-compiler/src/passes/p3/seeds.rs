// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Property demands retain exact declaration and logical-scope ownership.

use super::{MathRows, append, invalid};
use crate::CompilerError;
use pse_authoring::{
    document::{Rows, binding::SourceExpression},
    targets::TargetContext,
};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{authored, reference};
use pse_schema::{Registry, model::Cell};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Demands {
    mapping: BTreeMap<SemanticId, authored::template_symbol_properties::Row>,
    requirements: Vec<authored::template_property_requirements::Row>,
    guards: BTreeMap<SemanticId, (SemanticId, u64)>,
    scopes: BTreeSet<SemanticId>,
    properties: BTreeSet<SemanticId>,
    rows: Vec<Vec<Cell>>,
}
fn decode<T>(
    rows: &Rows,
    id: SemanticId,
    convert: fn(Vec<Cell>) -> Result<T, pse_relations::RelationError>,
) -> Result<Vec<T>, CompilerError> {
    rows.get(&id)
        .into_iter()
        .flatten()
        .cloned()
        .map(convert)
        .collect::<Result<_, _>>()
        .map_err(CompilerError::from)
}
impl Demands {
    pub(super) fn new(rows: &Rows) -> Result<Self, CompilerError> {
        let mappings = decode(
            rows,
            authored::template_symbol_properties::RELATION_ID,
            authored::template_symbol_properties::Row::from_cells,
        )?;
        let mut mapping = BTreeMap::new();
        for value in mappings {
            if mapping.insert(value.symbol_decl_id, value).is_some() {
                return Err(invalid("duplicate symbol property mapping"));
            }
        }
        Ok(Self {
            mapping,
            requirements: decode(
                rows,
                authored::template_property_requirements::RELATION_ID,
                authored::template_property_requirements::Row::from_cells,
            )?,
            guards: BTreeMap::new(),
            scopes: scope_inventory(rows)?,
            properties: decode(
                rows,
                reference::property_kinds::RELATION_ID,
                reference::property_kinds::Row::from_cells,
            )?
            .into_iter()
            .map(|row| row.property_kind_id)
            .collect(),
            rows: Vec::new(),
        })
    }
    pub(super) fn source(
        &mut self,
        source: &SourceExpression,
        source_id: SemanticId,
        lowered: &mut super::lower::Lowered,
        offset: u64,
        cancel: &CancellationToken,
        context: &TargetContext,
    ) -> Result<(), CompilerError> {
        if source.relation_id == authored::template_property_requirements::RELATION_ID {
            let [Cell::Id(requirement)] = source.row_key.as_slice() else {
                return Err(invalid("opaque property source key malformed"));
            };
            if lowered.syntax != "predicate" {
                return Err(invalid("opaque property guard must be a predicate"));
            }
            if self
                .guards
                .insert(*requirement, (source_id, lowered.root))
                .is_some()
            {
                return Err(invalid("duplicate opaque property guard"));
            }
        }
        if self.mapping.is_empty() {
            return Ok(());
        }
        for (symbol_id, guard) in super::demand_paths::collect(lowered, source_id, offset, cancel)?
        {
            let Some(mapping) = self.mapping.get(&symbol_id) else {
                continue;
            };
            self.check(mapping.scope_selector_id, mapping.property_kind_id)?;
            let symbols = context
                .symbols
                .iter()
                .filter(|symbol| symbol.symbol_decl_id == symbol_id)
                .collect::<Vec<_>>();
            let [symbol] = symbols.as_slice() else {
                return Err(invalid("property mapping symbol is absent or ambiguous"));
            };
            let index = if symbol.indexed_by.is_empty() {
                Cell::List(vec![])
            } else {
                Cell::Null
            };
            let id = pse_ids::named_id(
                source_id,
                &format!("property-symbol:{}:guard:{guard:?}", symbol_id.to_hex()),
            );
            self.rows.push(vec![
                Cell::Id(id),
                Cell::Id(source_id),
                Cell::Enum("expression"),
                Cell::Id(mapping.scope_selector_id),
                Cell::Id(mapping.property_kind_id),
                index,
                guard.map_or(Cell::Null, Cell::U64),
                Cell::Id(pse_ids::named_id(id, "pass:P3@1")),
                guard.map_or(Cell::Null, |_| Cell::Id(source_id)),
                Cell::Id(symbol_id),
            ]);
        }
        Ok(())
    }
    fn check(&self, scope: SemanticId, property: SemanticId) -> Result<(), CompilerError> {
        if self.scopes.contains(&scope) && self.properties.contains(&property) {
            Ok(())
        } else {
            Err(invalid(
                "property demand requires an actual scope and property kind",
            ))
        }
    }
    pub(super) fn finish(
        mut self,
        context: &TargetContext,
        output: &mut MathRows,
        registry: &Registry,
        cancel: &CancellationToken,
    ) -> Result<(), CompilerError> {
        for requirement in &self.requirements {
            cancel.checkpoint()?;
            self.check(requirement.scope_selector_id, requirement.property_kind_id)?;
            let guard = self.guards.get(&requirement.requirement_id).copied();
            if requirement.guard.is_some() != guard.is_some() {
                return Err(invalid("opaque demand guard source is incomplete"));
            }
            let tuples = tuples(requirement, context, cancel)?;
            for (ordinal, tuple) in tuples.into_iter().enumerate() {
                let id = pse_ids::named_id(
                    requirement.requirement_id,
                    &format!("property-index:{ordinal}"),
                );
                self.rows.push(vec![
                    Cell::Id(id),
                    Cell::Id(requirement.requirement_id),
                    Cell::Enum("opaque_operation"),
                    Cell::Id(requirement.scope_selector_id),
                    Cell::Id(requirement.property_kind_id),
                    Cell::List(tuple.into_iter().map(Cell::Id).collect()),
                    guard.map_or(Cell::Null, |(_, id)| Cell::U64(id)),
                    Cell::Id(pse_ids::named_id(id, "pass:P3@1")),
                    guard.map_or(Cell::Null, |(id, _)| Cell::Id(id)),
                    Cell::Null,
                ]);
            }
        }
        append(
            output,
            registry,
            "normalized.property_demand_seeds",
            self.rows,
        )
    }
}
fn tuples(
    requirement: &authored::template_property_requirements::Row,
    context: &TargetContext,
    cancel: &CancellationToken,
) -> Result<Vec<Vec<SemanticId>>, CompilerError> {
    let mut axes = Vec::new();
    let mut names = BTreeSet::new();
    let mut count = 1_usize;
    for binding in &requirement.index_domain_bindings {
        if !names.insert(&binding.index_name) {
            return Err(invalid("duplicate opaque index binding"));
        }
        let domains = context
            .domains
            .iter()
            .filter(|domain| domain.domain_id == binding.domain_id)
            .collect::<Vec<_>>();
        let [domain] = domains.as_slice() else {
            return Err(invalid("opaque index requires exactly one actual domain"));
        };
        let templates = context
            .template_domains
            .iter()
            .filter(|row| {
                row.template_id == requirement.template_id && row.name == binding.index_name
            })
            .collect::<Vec<_>>();
        let [template] = templates.as_slice() else {
            return Err(invalid(
                "opaque index name is not a declared template domain",
            ));
        };
        if template.kind != domain.kind || template.continuous != domain.continuous {
            return Err(invalid("opaque domain binding changes axis contract"));
        }
        let mut members = context
            .members
            .iter()
            .filter(|row| row.domain_id == binding.domain_id)
            .collect::<Vec<_>>();
        members.sort_by_key(|row| (row.ordinal, row.member_id));
        if members.is_empty()
            || members
                .iter()
                .map(|row| row.member_id)
                .collect::<BTreeSet<_>>()
                .len()
                != members.len()
            || members
                .iter()
                .map(|row| row.ordinal)
                .collect::<BTreeSet<_>>()
                .len()
                != members.len()
        {
            return Err(invalid(
                "opaque index domain has no finite unique member inventory",
            ));
        }
        count = count
            .checked_mul(members.len())
            .filter(|count| *count <= 65_536)
            .ok_or_else(|| invalid("property demand expansion exceeds finite budget"))?;
        axes.push(
            members
                .into_iter()
                .map(|row| row.member_id)
                .collect::<Vec<_>>(),
        );
    }
    let mut tuples = vec![Vec::new()];
    for axis in axes {
        let mut next = Vec::new();
        for tuple in tuples {
            for member in &axis {
                cancel.checkpoint()?;
                let mut value = tuple.clone();
                value.push(*member);
                next.push(value);
            }
        }
        tuples = next;
    }
    Ok(tuples)
}
fn scope_inventory(rows: &Rows) -> Result<BTreeSet<SemanticId>, CompilerError> {
    let scopes = decode(
        rows,
        authored::scopes::RELATION_ID,
        authored::scopes::Row::from_cells,
    )?;
    let terms = decode(
        rows,
        authored::selector_terms::RELATION_ID,
        authored::selector_terms::Row::from_cells,
    )?;
    let mut result = BTreeSet::new();
    for scope in scopes {
        if !result.insert(scope.scope_id) {
            return Err(invalid("duplicate logical scope"));
        }
        let owned = terms
            .iter()
            .filter(|term| term.scope_id == scope.scope_id)
            .collect::<Vec<_>>();
        let roots = owned
            .iter()
            .filter(|term| term.parent_term_id.is_none())
            .collect::<Vec<_>>();
        let [root] = roots.as_slice() else {
            return Err(invalid("scope must have one explicit selector root"));
        };
        if root.term_id != scope.root_term_id {
            return Err(invalid("scope selector root differs"));
        }
        let ids = owned
            .iter()
            .map(|term| term.term_id)
            .collect::<BTreeSet<_>>();
        if ids.len() != owned.len() {
            return Err(invalid("duplicate selector term identity"));
        }
        let mut visited = BTreeSet::new();
        let mut pending = vec![scope.root_term_id];
        while let Some(id) = pending.pop() {
            if !visited.insert(id) {
                return Err(invalid("cyclic selector graph"));
            }
            for child in owned.iter().filter(|term| term.parent_term_id == Some(id)) {
                pending.push(child.term_id);
            }
        }
        if visited != ids {
            return Err(invalid("dangling or disconnected selector term"));
        }
    }
    if terms.iter().any(|term| !result.contains(&term.scope_id)) {
        return Err(invalid("selector term has no owning scope"));
    }
    Ok(result)
}
