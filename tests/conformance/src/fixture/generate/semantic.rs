// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Concrete domain examples independent of the relational rule implementation.
use super::{Rows, id, put, row, set};
use pse_schema::{
    Registry,
    model::{Cell, InvariantSpec},
};

pub(super) fn populate(
    registry: &Registry,
    invariant: &InvariantSpec,
    valid: &mut Rows,
    invalid: &mut Rows,
) {
    let relation = invariant.relation.as_str();
    if super::inference::populate(registry, invariant, valid, invalid) {
        return;
    }
    match invariant.name.as_str() {
        "connections_admitted" => {
            valid.insert(relation.to_owned(), vec![]);
        }
        "cardinality:continuous_detail" => {
            put(registry, valid, "authored.continuous_domains", 1);
            *invalid = valid.clone();
            invalid.insert("authored.continuous_domains".to_owned(), vec![]);
        }
        "domain:continuous_context" => {
            put(registry, valid, "authored.domains", 1);
            *invalid = valid.clone();
            set(invalid, "authored.domains", "unit_id", id(2));
        }
        "cardinality:member_ordinal" => {
            let mut second = row(registry, registry.relation(relation).unwrap(), 2);
            second.insert("ordinal".to_owned(), Cell::I64(1));
            valid.get_mut(relation).unwrap().push(second);
            *invalid = valid.clone();
            invalid.get_mut(relation).unwrap()[1].insert("domain_id".to_owned(), id(1));
        }
        "closure:target_port_owner" => {
            put(registry, valid, "authored.instances", 1);
            put(registry, valid, "authored.template_ports", 1);
            set(
                valid,
                relation,
                "member",
                target_member("port", Cell::Struct(vec![id(1), Cell::text("value-1")])),
            );
            *invalid = valid.clone();
            set(
                invalid,
                relation,
                "member",
                target_member(
                    "port",
                    Cell::Struct(vec![id(1), Cell::text("missing port")]),
                ),
            );
        }
        "closure:entity_registered" | "closure:entity_fields" => {
            entity(registry, invariant, valid, invalid);
        }
        "closure:material_system_species_exist" => material(
            registry,
            invariant,
            "species_ids",
            "authored.species",
            valid,
            invalid,
        ),
        "closure:material_system_phases_exist" => material(
            registry,
            invariant,
            "phase_ids",
            "authored.phases",
            valid,
            invalid,
        ),
        "closure:packages.dependencies_resolved" => packages(registry, valid, invalid),
        "closure:stoichiometry_species_in_phase" => stoichiometry(registry, valid, invalid),
        "acyclic:parents" => parents(relation, valid, invalid),
        name if name.starts_with("closure:target_owner:") => {
            target_owner(
                registry,
                relation,
                name.strip_prefix("closure:target_owner:").unwrap(),
                valid,
                invalid,
            );
        }
        name => panic!("missing explicit semantic fixture: {relation}:{name}"),
    }
}
fn parents(relation: &str, valid: &mut Rows, invalid: &mut Rows) {
    let parent = match relation {
        "authored.entities" => "parent_entity_id",
        "authored.instances" => "parent_instance_id",
        "authored.cases" => "parent_case_id",
        _ => panic!("uncovered parent relation {relation}"),
    };
    set(valid, relation, parent, Cell::Null);
    *invalid = valid.clone();
    set(invalid, relation, parent, id(1));
}

fn target_member(kind: &'static str, payload: Cell) -> Cell {
    let mut payload = Some(payload);
    Cell::Struct(
        std::iter::once(Cell::Enum(kind))
            .chain(
                ["symbol", "group", "equation", "port"]
                    .into_iter()
                    .map(|name| {
                        if name == kind {
                            payload.take().expect("one complete target arm")
                        } else {
                            Cell::Null
                        }
                    }),
            )
            .collect(),
    )
}
fn target_owner(
    registry: &Registry,
    relation: &str,
    kind: &str,
    valid: &mut Rows,
    invalid: &mut Rows,
) {
    let (kind, declarations) = match kind {
        "symbol" => ("symbol", "authored.template_symbols"),
        "group" => ("group", "authored.template_symbols"),
        "equation" => ("equation", "authored.template_equations"),
        _ => panic!("uncovered target owner {kind}"),
    };
    set(
        valid,
        relation,
        "member",
        target_member(kind, Cell::Struct(vec![id(1), Cell::Null])),
    );
    put(registry, valid, "authored.instances", 1);
    put(registry, valid, declarations, 1);
    *invalid = valid.clone();
    set(invalid, declarations, "template_id", id(2));
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
        Cell::Enum(section.entity_kind.unwrap()),
    );
    let name = section.name_column.map_or_else(
        || Cell::text("value-1"),
        |column| valid[&invariant.relation][0][column].clone(),
    );
    set(valid, "authored.entities", "name", name);
    let parent = section.naming_scope_column.map_or(Cell::Null, |column| {
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
            Cell::text("different name"),
        );
    } else {
        set(invalid, "authored.entities", "parent_entity_id", id(2));
    }
}
fn material(
    registry: &Registry,
    invariant: &InvariantSpec,
    list: &str,
    target: &str,
    valid: &mut Rows,
    invalid: &mut Rows,
) {
    put(registry, valid, target, 1);
    set(valid, &invariant.relation, list, Cell::List(vec![id(1)]));
    *invalid = valid.clone();
    set(invalid, &invariant.relation, list, Cell::List(vec![id(2)]));
}
fn packages(registry: &Registry, valid: &mut Rows, invalid: &mut Rows) {
    let relation = "authored.packages";
    let target = row(registry, registry.relation(relation).unwrap(), 2);
    valid.get_mut(relation).unwrap().push(target);
    set(
        valid,
        relation,
        "dependencies",
        Cell::List(vec![Cell::Struct(vec![id(2), Cell::text("value-2")])]),
    );
    *invalid = valid.clone();
    set(
        invalid,
        relation,
        "dependencies",
        Cell::List(vec![Cell::Struct(vec![
            id(2),
            Cell::text("missing version"),
        ])]),
    );
}
fn stoichiometry(registry: &Registry, valid: &mut Rows, invalid: &mut Rows) {
    put(registry, valid, "authored.species", 1);
    put(registry, valid, "authored.phases", 1);
    set(
        valid,
        "authored.phases",
        "phase_type",
        Cell::Enum(pse_material::PhaseType::Liquid.as_str()),
    );
    set(
        valid,
        "authored.species",
        "valid_phase_types",
        Cell::List(vec![Cell::Enum(pse_material::PhaseType::Liquid.as_str())]),
    );
    *invalid = valid.clone();
    set(
        invalid,
        "authored.species",
        "valid_phase_types",
        Cell::List(vec![Cell::Enum(pse_material::PhaseType::Vapor.as_str())]),
    );
}
