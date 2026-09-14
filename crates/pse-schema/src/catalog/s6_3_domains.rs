// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.3 domain.

use super::declarations::{column, relation};
use crate::builder::RegistryBuilder;
use crate::model::{LogicalType as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.3 domain contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_authored_domains(builder);
    declare_authored_domain_members(builder);
    declare_authored_continuous_domains(builder);
    declare_normalized_domain_products(builder);
    declare_inferred_valid_index_tuples(builder);
    declare_compiled_meshes(builder);
    declare_compiled_mesh_nodes(builder);
    declare_compiled_stencils(builder);
    declare_compiled_quadrature_rules(builder);
    declare_node_kind_vocabulary(builder);
}

fn declare_authored_domains(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "domains",
        S::Model,
        &["domain_id"],
        vec![
            column("domain_id", T::id()),
            column("owner_entity_id", T::id()).with_fk("authored.entities", "entity_id"),
            column("kind", T::enumeration("DomainKind")),
            column("continuous", T::Bool),
            column("unit_id", T::id())
                .optional()
                .with_fk("reference.units", "unit_id"),
            column("parent_domain_id", T::id())
                .optional()
                .with_fk("authored.domains", "domain_id"),
            column("doc", T::Text),
        ],
        "blueprint §6.3 domain: domains.",
    );
}

fn declare_authored_domain_members(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "domain_members",
        S::Model,
        &["member_id"],
        vec![
            column("domain_id", T::id()).with_fk("authored.domains", "domain_id"),
            column("member_id", T::id()),
            column("ordinal", T::U32),
            column("label", T::Text),
            column("coordinate", T::F64).optional(),
            column("ref_entity_id", T::id())
                .optional()
                .with_fk("authored.entities", "entity_id"),
        ],
        "blueprint §6.3 domain: domain_members.",
    );
}

fn declare_authored_continuous_domains(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "continuous_domains",
        S::Model,
        &["domain_id"],
        vec![
            column("domain_id", T::id()).with_fk("authored.domains", "domain_id"),
            column("lower", T::F64),
            column("upper", T::F64),
            column("unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("initial_points", T::list(T::F64)),
            column("discretization_policy_id", T::id()).optional(),
        ],
        "blueprint §6.3 domain: continuous_domains.",
    );
}

fn declare_normalized_domain_products(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Normalized,
        "domain_products",
        S::Derived,
        &["product_id"],
        vec![
            column("product_id", T::id()),
            column("domain_ids", T::list(T::id())),
        ],
        "blueprint §6.3 domain: domain_products.",
    );
}

fn declare_inferred_valid_index_tuples(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "valid_index_tuples",
        S::Derived,
        &["product_id", "tuple"],
        vec![
            column("product_id", T::id()).with_fk("normalized.domain_products", "product_id"),
            column("tuple", T::Ext(crate::model::ExtensionUse::IndexTuple)),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.3 domain: valid_index_tuples.",
    );
}

fn declare_compiled_meshes(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "meshes",
        S::Derived,
        &["mesh_id"],
        vec![
            column("mesh_id", T::id()),
            column("domain_id", T::id()),
            column("policy_id", T::id()),
            column("node_count", T::U32),
            column("nodes", T::list(T::F64)),
        ],
        "blueprint §6.3 domain: meshes.",
    );
}

fn declare_compiled_mesh_nodes(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "mesh_nodes",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::id()),
            column("mesh_id", T::id()).with_fk("compiled.meshes", "mesh_id"),
            column("ordinal", T::U32),
            column("coordinate", T::F64),
            column("kind", T::enumeration("NodeKind")),
        ],
        "blueprint §6.3 domain: mesh_nodes.",
    );
}

fn declare_compiled_stencils(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "stencils",
        S::Derived,
        &["stencil_id"],
        vec![
            column("stencil_id", T::id()),
            column("mesh_id", T::id()).with_fk("compiled.meshes", "mesh_id"),
            column("derivative_order", T::U8),
            column("scheme", T::enumeration("DiscretizationScheme")),
            column("node_id", T::id()),
            column("neighbor_node_id", T::id()),
            column("weight", T::F64),
        ],
        "blueprint §6.3 domain: stencils.",
    );
}

fn declare_compiled_quadrature_rules(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "quadrature_rules",
        S::Derived,
        &["rule_id"],
        vec![
            column("rule_id", T::id()),
            column("mesh_id", T::id()).with_fk("compiled.meshes", "mesh_id"),
            column("node_id", T::id()),
            column("weight", T::F64),
        ],
        "blueprint §6.3 domain: quadrature_rules.",
    );
}

fn declare_node_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "NodeKind",
        ["element_boundary", "collocation", "interior"],
    );
}
