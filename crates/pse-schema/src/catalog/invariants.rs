// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native relational integrity queries over explicit source bindings.
use super::inv::{declare as invariant, identifier, table};
use crate::{RegistryBuilder, model::InvariantKind};

/// Add mechanical integrity projections and explicitly declared domain contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    builder.derive_integrity();
    entity_registration(builder);
    super::invariant_closure::declare(builder);
}
fn entity_registration(builder: &mut RegistryBuilder) {
    let mut mappings = std::collections::BTreeMap::new();
    for document in builder.declared_documents() {
        for section in &document.sections {
            if let (Some(identity), Some(kind)) = (section.identity_column, section.entity_kind) {
                mappings.insert(
                    section.relation,
                    (
                        identity,
                        kind,
                        section.name_column,
                        section.naming_scope_column,
                    ),
                );
            }
        }
    }
    for (relation, (identity, kind, name, owner)) in mappings {
        let source = table(relation);
        let key = identifier(identity);
        let kind = super::inv::literal(kind);
        invariant(
            builder,
            relation,
            "closure:entity_registered",
            InvariantKind::Closure,
            &[identity],
            format!(
                "SELECT s.{key} FROM {source} s WHERE NOT EXISTS (SELECT 1 FROM authored.entities e WHERE e.entity_id = s.{key} AND e.kind = {kind})"
            ),
            &[relation, "authored.entities"],
            "Every declared identity has an entity row with its declared entity kind.",
        );
        let mut mismatches = Vec::new();
        if let Some(name) = name {
            mismatches.push(format!("s.{} IS DISTINCT FROM e.name", identifier(name)));
        }
        let parent = owner.map_or_else(|| "NULL".into(), |name| format!("s.{}", identifier(name)));
        mismatches.push(format!("{parent} IS DISTINCT FROM e.parent_entity_id"));
        let has_package = builder
            .declared_relations()
            .iter()
            .find(|spec| spec.key.qualified_name() == relation)
            .is_some_and(|spec| {
                spec.columns
                    .iter()
                    .any(|column| column.name() == "package_id")
            });
        let mut owner_join = String::new();
        if has_package {
            mismatches.push("s.package_id IS DISTINCT FROM e.package_id".into());
        } else if let Some(owner) = owner {
            owner_join = format!(
                " JOIN authored.entities p ON s.{} = p.entity_id",
                identifier(owner)
            );
            mismatches.push("p.package_id IS DISTINCT FROM e.package_id".into());
        }
        invariant(
            builder,
            relation,
            "closure:entity_fields",
            InvariantKind::Closure,
            &[identity],
            format!(
                "SELECT s.{key} FROM {source} s JOIN authored.entities e ON s.{key} = e.entity_id{owner_join} WHERE {}",
                mismatches
                    .iter()
                    .map(|predicate| format!("({predicate})"))
                    .collect::<Vec<_>>()
                    .join(" OR ")
            ),
            &[relation, "authored.entities"],
            "Entity names, explicit parents and available declaring-package facts match their actual source declarations.",
        );
    }
}
