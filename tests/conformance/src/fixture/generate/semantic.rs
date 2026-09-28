// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Concrete domain examples independent of the relational rule implementation.
use super::{Rows, id, put, row, set};
use pse_schema::{Registry, model::InvariantSpec};

pub(super) fn populate(
    registry: &Registry,
    invariant: &InvariantSpec,
    valid: &mut Rows,
    invalid: &mut Rows,
) {
    let relation = invariant.relation.as_str();
    match invariant.name.as_str() {
        "closure:entity_registered" | "closure:entity_fields" => {
            entity(registry, invariant, valid, invalid);
        }
        "closure:packages.dependencies_resolved" => packages(registry, valid, invalid),
        "acyclic:parents" => parents(relation, valid, invalid),
        name => panic!("missing explicit semantic fixture: {relation}:{name}"),
    }
}
fn parents(relation: &str, valid: &mut Rows, invalid: &mut Rows) {
    let parent = match relation {
        "authored.entities" => "parent_entity_id",
        _ => panic!("uncovered parent relation {relation}"),
    };
    set(valid, relation, parent, serde_json::json!(["null", null]));
    *invalid = valid.clone();
    set(invalid, relation, parent, id(1));
}

fn entity(registry: &Registry, invariant: &InvariantSpec, valid: &mut Rows, invalid: &mut Rows) {
    let section = registry
        .documents()
        .iter()
        .flat_map(|document| &document.sections)
        .find(|section| section.relation == invariant.relation && section.identity_column.is_some())
        .expect("identity declared by document");
    put(registry, valid, "authored.entities", 1);
    let identity = valid[&invariant.relation][0][section.identity_column.unwrap()].clone();
    set(valid, "authored.entities", "entity_id", identity);
    set(
        valid,
        "authored.entities",
        "kind",
        serde_json::json!(["enum", section.entity_kind.unwrap()]),
    );
    let name = section.name_column.map_or_else(
        || serde_json::json!(["text", "value-1"]),
        |column| valid[&invariant.relation][0][column].clone(),
    );
    set(valid, "authored.entities", "name", name);
    let parent = section
        .naming_scope_column
        .map_or(serde_json::json!(["null", null]), |column| {
            valid[&invariant.relation][0][column].clone()
        });
    set(valid, "authored.entities", "parent_entity_id", parent);
    *invalid = valid.clone();
    if invariant.name == "closure:entity_registered" {
        invalid.insert("authored.entities".to_owned(), vec![]);
    } else if section.name_column.is_some() {
        set(
            invalid,
            "authored.entities",
            "name",
            serde_json::json!(["text", "different name"]),
        );
    } else {
        set(invalid, "authored.entities", "parent_entity_id", id(2));
    }
}

fn packages(registry: &Registry, valid: &mut Rows, invalid: &mut Rows) {
    let relation = "authored.packages";
    let target = row(registry, registry.relation(relation).unwrap(), 2);
    valid.get_mut(relation).unwrap().push(target);
    set(
        valid,
        relation,
        "dependencies",
        serde_json::json!([
            "list",
            vec![serde_json::json!([
                "struct",
                vec![id(2), serde_json::json!(["text", "value-2"])]
            ])]
        ]),
    );
    *invalid = valid.clone();
    set(
        invalid,
        relation,
        "dependencies",
        serde_json::json!([
            "list",
            vec![serde_json::json!([
                "struct",
                vec![id(2), serde_json::json!(["text", "missing version"]),]
            ])]
        ]),
    );
}
