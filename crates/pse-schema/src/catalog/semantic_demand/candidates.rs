// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete candidate classification and greatest-rank selection are relational.
use super::{
    Cell, CmpOp, E, P, RegistryBuilder, anti, eq, filter, join, literal, null, present, project,
    rule, same, scan,
};
mod dependencies;
mod parameters;
mod signatures;
mod winners;

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    let base = join(
        scan("inferred.requirement_universe", "requirements"),
        scan("inferred.selection_inventory", "selections"),
        vec![(
            "requirements.property_package_id",
            "selections.property_package_id",
        )],
    );
    let base = filter(
        base,
        E::Or(vec![
            null("selections.property_kind_id"),
            eq(
                E::col("requirements.property_kind_id"),
                E::col("selections.property_kind_id"),
            ),
        ]),
    );
    let scoped = join(
        base.clone(),
        scan("inferred.requirement_scope_keys", "scope_keys"),
        vec![
            ("requirements.requirement_id", "scope_keys.requirement_id"),
            ("selections.scope_kind", "scope_keys.scope_kind"),
            ("selections.scope_ids", "scope_keys.scope_ids"),
        ],
    );
    classify(
        builder,
        "P6.candidate_scope_mismatch",
        difference(base.clone(), scoped.clone()),
        false,
        "scope_mismatch",
        None,
    );
    let method = join(
        scoped.clone(),
        scan("reference.method_specs", "methods"),
        vec![("selections.method_id", "methods.method_id")],
    );
    let family = filter(
        method,
        eq(E::col("selections.family"), E::col("methods.family")),
    );
    classify(
        builder,
        "P6.candidate_family_mismatch",
        difference(scoped, family.clone()),
        false,
        "family_mismatch",
        None,
    );
    let provision = join(
        family.clone(),
        scan("reference.method_provisions", "provisions"),
        vec![
            ("selections.method_id", "provisions.method_id"),
            (
                "requirements.property_kind_id",
                "provisions.property_kind_id",
            ),
        ],
    );
    classify(
        builder,
        "P6.candidate_missing_provision",
        difference(family, provision.clone()),
        false,
        "missing_provision",
        None,
    );
    let signature = signatures::expected(physical_signature(provision.clone()));
    classify(
        builder,
        "P6.candidate_incompatible_signature",
        difference(provision, signature.clone()),
        false,
        "incompatible_signature",
        None,
    );
    let dependent = dependencies::complete(signature.clone());
    classify(
        builder,
        "P6.candidate_missing_dependency",
        difference(signature, dependent.clone()),
        false,
        "incompatible_signature",
        None,
    );
    let signature = dependent;
    let with_parameters = parameters::complete(signature.clone());
    classify(
        builder,
        "P6.candidate_missing_parameter",
        difference(signature, with_parameters.clone()),
        false,
        "missing_parameter",
        None,
    );
    let ranked = ranked(with_parameters.clone());
    // A missing precedence row is a malformed selection policy, not an implicit default.
    classify(
        builder,
        "P6.candidate_missing_rank",
        difference(with_parameters, ranked.clone()),
        false,
        "incompatible_signature",
        None,
    );
    classify(
        builder,
        "P6.candidate_applicable",
        ranked,
        true,
        "applicable",
        Some("precedence.rank"),
    );
    winners::declare(builder);
}
fn physical_signature(input: P) -> P {
    let input = join(
        input,
        scan("reference.property_kinds", "properties"),
        vec![(
            "requirements.property_kind_id",
            "properties.property_kind_id",
        )],
    );
    let input = join(
        input,
        scan("reference.quantity_types", "quantities"),
        vec![("provisions.quantity_type_id", "quantities.quantity_type_id")],
    );
    let input = join(
        input,
        scan("reference.units", "natural"),
        vec![("provisions.natural_unit_id", "natural.unit_id")],
    );
    let input = join(
        input,
        scan("reference.units", "canonical"),
        vec![("quantities.canonical_unit_id", "canonical.unit_id")],
    );
    filter(
        input,
        E::And(vec![
            eq(
                E::col("quantities.quantity_kind_id"),
                E::col("properties.quantity_kind_id"),
            ),
            same("quantities.basis_id", "properties.basis_id"),
            eq(E::col("quantities.shape"), E::col("properties.shape")),
            eq(E::col("provisions.indexed_by"), E::col("properties.shape")),
            eq(E::col("natural.dimension"), E::col("canonical.dimension")),
            E::Or(vec![
                null("natural.reference_state_id"),
                same(
                    "natural.reference_state_id",
                    "quantities.reference_state_id",
                ),
            ]),
        ]),
    )
}
fn ranked(input: P) -> P {
    let input = join(
        input,
        scan("reference.method_precedence", "precedence"),
        vec![
            ("selections.is_default", "precedence.is_default"),
            ("selections.scope_kind", "precedence.scope_kind"),
        ],
    );
    filter(
        input,
        eq(
            E::col("precedence.property_specific"),
            present("selections.property_kind_id"),
        ),
    )
}
fn difference(left: P, right: P) -> P {
    anti(
        left,
        project(
            right,
            vec![
                ("matched_requirement", E::col("requirements.requirement_id")),
                ("matched_selection", E::col("selections.selection_id")),
                ("matched_method", E::col("selections.method_id")),
            ],
        ),
        vec![
            ("requirements.requirement_id", "matched_requirement"),
            ("selections.selection_id", "matched_selection"),
            ("selections.method_id", "matched_method"),
        ],
    )
}
fn classify(
    builder: &mut RegistryBuilder,
    name: &'static str,
    input: P,
    applicable: bool,
    reason: &'static str,
    rank: Option<&'static str>,
) {
    rule(
        builder,
        name,
        2,
        "inferred.potential_method_candidates",
        "provenance.potential_method_candidate_assertions",
        project(
            input,
            vec![
                ("requirement_id", E::col("requirements.requirement_id")),
                ("selection_id", E::col("selections.selection_id")),
                ("method_id", E::col("selections.method_id")),
                ("applicable", E::Lit(Cell::Bool(applicable))),
                ("rank", rank.map_or(E::Lit(Cell::Null), E::col)),
                ("reason", literal(reason)),
                ("derivation_id", E::col("selections.derivation_id")),
            ],
        ),
        true,
    );
}
