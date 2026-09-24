// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.3 domain.

use super::declarations::{column, relation, relation_version};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.3 domain contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_authored_domains(builder);
    declare_authored_domain_members(builder);
    declare_authored_continuous_domains(builder);
    declare_normalized_domain_products(builder);
    declare_inferred_valid_index_tuples(builder);
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
            column("continuous", T::native(arrow_schema::DataType::Boolean)),
            column("unit_id", T::id())
                .optional()
                .with_fk("reference.units", "unit_id"),
            column("parent_domain_id", T::id())
                .optional()
                .with_fk("authored.domains", "domain_id"),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
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
            column("ordinal", T::nonnegative(i64::from(u32::MAX))),
            column("label", T::native(arrow_schema::DataType::Utf8)),
            column("coordinate", T::native(arrow_schema::DataType::Float64)).optional(),
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
            column("lower", T::native(arrow_schema::DataType::Float64)),
            column("upper", T::native(arrow_schema::DataType::Float64)),
            column("unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column(
                "initial_points",
                T::list(T::native(arrow_schema::DataType::Float64)),
            ),
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
    relation_version(
        builder,
        N::Inferred,
        "valid_index_tuples",
        2,
        S::Derived,
        &["product_id", "tuple"],
        vec![
            column("product_id", T::id()).with_fk("normalized.domain_products", "product_id"),
            column("tuple", T::extended(crate::model::ExtensionUse::IndexTuple)),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.3 domain: valid_index_tuples.",
    );
}

