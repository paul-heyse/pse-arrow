// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.4 material.

use super::declarations::{column, relation, structure};
use crate::builder::RegistryBuilder;
use crate::model::{LogicalType as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.4 material contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_reference_elements(builder);
    declare_authored_species(builder);
    declare_authored_species_elements(builder);
    declare_authored_phases(builder);
    declare_authored_phase_species(builder);
    declare_authored_henry_declarations(builder);
    declare_authored_material_systems(builder);
    declare_inferred_phase_species(builder);
    declare_authored_reactions(builder);
    declare_authored_stoichiometry(builder);
    declare_authored_reaction_methods(builder);
    declare_authored_reaction_packages(builder);
    declare_authored_parameter_values(builder);
    declare_reaction_kind_vocabulary(builder);
}

fn declare_reference_elements(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "elements",
        S::Model,
        &["element_id"],
        vec![
            column("element_id", T::id()),
            column("symbol", T::Text),
            column("name", T::Text),
            column("atomic_mass", T::F64),
        ],
        "blueprint §6.4 material: elements.",
    );
}

fn declare_authored_species(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "species",
        S::Model,
        &["species_id"],
        vec![
            column("species_id", T::id()),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("name", T::Text),
            column("formula", T::Text).optional(),
            column("mw", T::F64).optional(),
            column("component_type", T::enumeration("ComponentType")),
            column("charge", T::I64),
            column(
                "dissociation_species",
                T::list(structure(vec![
                    ("species_id", T::id()),
                    ("coefficient", T::F64),
                ])),
            )
            .optional(),
            column("valid_phase_types", T::list(T::enumeration("PhaseType"))).optional(),
            column("doc", T::Text),
        ],
        "blueprint §6.4 material: species.",
    );
}

fn declare_authored_species_elements(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "species_elements",
        S::Model,
        &["species_id", "element_id"],
        vec![
            column("species_id", T::id()).with_fk("authored.species", "species_id"),
            column("element_id", T::id()).with_fk("reference.elements", "element_id"),
            column("count", T::F64),
        ],
        "blueprint §6.4 material: species_elements.",
    );
}

fn declare_authored_phases(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "phases",
        S::Model,
        &["phase_id"],
        vec![
            column("phase_id", T::id()),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("name", T::Text),
            column("phase_type", T::enumeration("PhaseType")),
            column("is_solvent_phase", T::Bool),
            column("doc", T::Text),
        ],
        "blueprint §6.4 material: phases.",
    );
}

fn declare_authored_phase_species(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "phase_species",
        S::Model,
        &["phase_id", "species_id"],
        vec![
            column("phase_id", T::id()).with_fk("authored.phases", "phase_id"),
            column("species_id", T::id()).with_fk("authored.species", "species_id"),
        ],
        "blueprint §6.4 material: phase_species.",
    );
}

fn declare_authored_henry_declarations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "henry_declarations",
        S::Model,
        &["species_id", "phase_id"],
        vec![
            column("species_id", T::id()).with_fk("authored.species", "species_id"),
            column("phase_id", T::id()).with_fk("authored.phases", "phase_id"),
            column("henry_type", T::enumeration("HenryType")),
            column("method_id", T::id()),
        ],
        "blueprint §6.4 material: henry_declarations.",
    );
}

fn declare_authored_material_systems(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "material_systems",
        S::Model,
        &["material_system_id"],
        vec![
            column("material_system_id", T::id()),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("name", T::Text),
            column("species_ids", T::list(T::id())),
            column("phase_ids", T::list(T::id())),
            column("doc", T::Text),
        ],
        "blueprint §6.4 material: material_systems.",
    );
}

fn declare_inferred_phase_species(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "phase_species",
        S::Derived,
        &["material_system_id", "phase_id", "species_id"],
        vec![
            column("material_system_id", T::id()),
            column("phase_id", T::id()),
            column("species_id", T::id()),
            column("henry", T::Bool),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.4 material: phase_species.",
    );
}

fn declare_authored_reactions(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "reactions",
        S::Model,
        &["reaction_id"],
        vec![
            column("reaction_id", T::id()),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("name", T::Text),
            column("kind", T::enumeration("ReactionKind")),
            column("basis", T::enumeration("BasisKind")),
            column("concentration_form", T::enumeration("ConcentrationForm")).optional(),
            column("reaction_phase_id", T::id()).optional(),
            column("doc", T::Text),
        ],
        "blueprint §6.4 material: reactions.",
    );
}

fn declare_authored_stoichiometry(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "stoichiometry",
        S::Model,
        &["reaction_id", "phase_id", "species_id"],
        vec![
            column("reaction_id", T::id()).with_fk("authored.reactions", "reaction_id"),
            column("phase_id", T::id()).with_fk("authored.phases", "phase_id"),
            column("species_id", T::id()).with_fk("authored.species", "species_id"),
            column("coefficient", T::F64),
        ],
        "blueprint §6.4 material: stoichiometry.",
    );
}

fn declare_authored_reaction_methods(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "reaction_methods",
        S::Model,
        &["reaction_id"],
        vec![
            column("reaction_id", T::id()).with_fk("authored.reactions", "reaction_id"),
            column("rate_form_method_id", T::id()).optional(),
            column("rate_constant_method_id", T::id()).optional(),
            column("equilibrium_form_method_id", T::id()).optional(),
            column("equilibrium_constant_method_id", T::id()).optional(),
            column("heat_of_reaction_method_id", T::id()).optional(),
        ],
        "blueprint §6.4 material: reaction_methods.",
    );
}

fn declare_authored_reaction_packages(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "reaction_packages",
        S::Model,
        &["reaction_package_id"],
        vec![
            column("reaction_package_id", T::id()),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("property_package_id", T::id()),
            column("reaction_ids", T::list(T::id())),
            column("unit_set_id", T::id()),
            column(
                "default_arguments",
                T::list(structure(vec![("key", T::Text), ("value", T::Text)])),
            ),
            column("doc", T::Text),
        ],
        "blueprint §6.4 material: reaction_packages.",
    );
}

fn declare_authored_parameter_values(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "parameter_values",
        S::Model,
        &["owner_entity_id", "parameter_kind", "index"],
        vec![
            column("owner_entity_id", T::id()).with_fk("authored.entities", "entity_id"),
            column("parameter_kind", T::Text),
            column("index", T::Ext(crate::model::ExtensionUse::IndexTuple)),
            column("value", T::F64),
            column("unit_id", T::id()),
            column("source", T::Text).optional(),
            column("std_dev", T::F64).optional(),
            column("estimable", T::Bool),
        ],
        "blueprint §6.4 material: parameter_values.",
    );
}

fn declare_reaction_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ReactionKind",
        pse_material::enums::ReactionKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}
