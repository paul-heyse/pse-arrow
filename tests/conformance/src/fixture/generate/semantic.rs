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
    match invariant.name.as_str() {
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
            second.insert("ordinal".to_owned(), Cell::U64(1));
            valid.get_mut(relation).unwrap().push(second);
            *invalid = valid.clone();
            invalid.get_mut(relation).unwrap()[1].insert("domain_id".to_owned(), id(1));
        }
        "closure:target_port_owner" => {
            put(registry, valid, "authored.instances", 1);
            put(registry, valid, "authored.template_ports", 1);
            *invalid = valid.clone();
            set(invalid, relation, "port_name", Cell::text("missing port"));
        }
        "closure:entity_registered" | "closure:entity_fields" => {
            entity(registry, invariant, valid, invalid);
        }
        name if name.starts_with("check:terminal_") => terminal(invariant, valid, invalid),
        "check:positive_scale" => positive(invariant, "scale_to_canonical", valid, invalid),
        "check:positive_nominal" => positive(invariant, "nominal_magnitude", valid, invalid),
        "check:ordered_bounds" => {
            set(valid, relation, "lower", Cell::F64(1.0));
            set(valid, relation, "upper", Cell::F64(2.0));
            *invalid = valid.clone();
            set(invalid, relation, "lower", Cell::F64(3.0));
        }
        "check:target_shape" => {
            set(valid, relation, "member_kind", Cell::Enum("symbol"));
            for name in ["equation_decl_id", "port_template_id", "port_name"] {
                set(valid, relation, name, Cell::Null);
            }
            set(valid, relation, "wildcard", Cell::Bool(false));
            *invalid = valid.clone();
            set(invalid, relation, "equation_decl_id", id(1));
        }
        "check:domain_reference" => {
            set(valid, relation, "template_id", Cell::Null);
            set(valid, relation, "domain_name", Cell::Null);
            *invalid = valid.clone();
            set(invalid, relation, "template_id", id(1));
            set(invalid, relation, "domain_name", Cell::text("space"));
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
        "acyclic:parents" => {
            let parent = match relation {
                "authored.entities" => "parent_entity_id",
                "authored.instances" => "parent_instance_id",
                "authored.cases" => "parent_case_id",
                "authored.model_revisions" => "parent_revision_id",
                _ => panic!("uncovered parent relation {relation}"),
            };
            set(valid, relation, parent, Cell::Null);
            *invalid = valid.clone();
            set(invalid, relation, parent, id(1));
        }
        name if name.starts_with("closure:target_owner:") => {
            let column = name.strip_prefix("closure:target_owner:").unwrap();
            let declarations = match column {
                "symbol_decl_id" => "authored.template_symbols",
                "equation_decl_id" => "authored.template_equations",
                _ => panic!("uncovered target owner {column}"),
            };
            put(registry, valid, "authored.instances", 1);
            put(registry, valid, declarations, 1);
            *invalid = valid.clone();
            set(invalid, declarations, "template_id", id(2));
        }
        name => panic!("missing explicit semantic fixture: {relation}:{name}"),
    }
}
fn positive(invariant: &InvariantSpec, column: &str, valid: &mut Rows, invalid: &mut Rows) {
    set(valid, &invariant.relation, column, Cell::F64(1.0));
    *invalid = valid.clone();
    set(invalid, &invariant.relation, column, Cell::F64(0.0));
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

fn terminal(invariant: &InvariantSpec, valid: &mut Rows, invalid: &mut Rows) {
    let relation = invariant.relation.as_str();
    for (column, value) in [
        ("status", Cell::Enum("ok")),
        ("failure_class", Cell::Null),
        ("finding_count", Cell::U64(0)),
        ("findings", Cell::List(vec![])),
        ("snapshot_out", Cell::Null),
        ("duration_ms", Cell::F64(1.0)),
    ] {
        set(valid, relation, column, value);
    }
    let error = vec![
        id(3),
        Cell::Null,
        Cell::Null,
        Cell::Null,
        Cell::Enum("error"),
        Cell::List(vec![]),
        Cell::text("{}"),
        Cell::text("actual execution failure"),
        Cell::List(vec![]),
    ];
    if matches!(
        invariant.name.as_str(),
        "check:terminal_cancel_class"
            | "check:terminal_failed_output"
            | "check:terminal_failure_finding"
            | "check:terminal_finding_origin"
    ) {
        set(valid, relation, "status", Cell::Enum("failed"));
        set(
            valid,
            relation,
            "failure_class",
            Cell::Enum("runtime.infrastructure"),
        );
        set(valid, relation, "finding_count", Cell::U64(1));
        set(
            valid,
            relation,
            "findings",
            Cell::List(vec![Cell::Struct(error.clone())]),
        );
    }
    *invalid = valid.clone();
    match invariant.name.as_str() {
        "check:terminal_count" => set(invalid, relation, "finding_count", Cell::U64(1)),
        "check:terminal_failure_class" => set(
            invalid,
            relation,
            "failure_class",
            Cell::Enum("runtime.infrastructure"),
        ),
        "check:terminal_cancel_class" => set(invalid, relation, "status", Cell::Enum("cancelled")),
        "check:terminal_failed_output" => set(
            invalid,
            relation,
            "snapshot_out",
            Cell::Hash(pse_ids::ContentHash::from_bytes([9; 32])),
        ),
        "check:terminal_failure_finding" => {
            set(invalid, relation, "finding_count", Cell::U64(0));
            set(invalid, relation, "findings", Cell::List(vec![]));
        }
        "check:terminal_duration" => set(invalid, relation, "duration_ms", Cell::F64(-1.0)),
        "check:terminal_finding_origin" => {
            let mut warning = error;
            warning[4] = Cell::Enum("warning");
            set(
                invalid,
                relation,
                "findings",
                Cell::List(vec![Cell::Struct(warning)]),
            );
        }
        name => panic!("missing terminal witness: {name}"),
    }
}
