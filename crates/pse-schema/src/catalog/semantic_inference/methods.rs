// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Method names become compatible only through the actual complete provision inventory.
use super::{
    Cell, E, N, P, RegistryBuilder, RuleDecl, RuleHead, S, T, assertion, column, join, project,
    provenance, relation, scan,
};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "method_compatibility",
        S::Derived,
        &["selection_id", "method_id"],
        vec![
            column("selection_id", T::id()),
            column("method_id", T::id()),
            column("compatible", T::native(arrow_schema::DataType::Boolean)),
            column("reason", T::enumeration("MethodCandidateReason")),
            provenance(),
        ],
        "P4 selection family and supported provision validation; P6 checks each actual requirement signature and scope.",
    );
    assertion(
        builder,
        "inferred.method_compatibility",
        "method_compatibility_assertions",
    );
    let selections = scan("normalized.method_selections", "selections");
    let matched = join(
        selections.clone(),
        scan("reference.method_specs", "methods"),
        vec![("selections.method_id", "methods.method_id")],
    );
    let family_matches = E::cmp(
        crate::model::CmpOp::Eq,
        E::col("selections.family"),
        E::col("methods.family"),
    );
    let compatible_family = P::Filter {
        input: Box::new(matched.clone()),
        predicate: family_matches.clone(),
    };
    let family = project(
        compatible_family,
        vec![
            ("selection_id", E::col("selections.selection_id")),
            ("method_id", E::col("selections.method_id")),
            ("property_kind_id", E::col("selections.property_kind_id")),
        ],
    );
    let specific = P::Filter {
        input: Box::new(family.clone()),
        predicate: E::IsNotNull(Box::new(E::col("property_kind_id"))),
    };
    let generic = P::Filter {
        input: Box::new(family.clone()),
        predicate: E::IsNull(Box::new(E::col("property_kind_id"))),
    };
    let provided_specific = join(
        specific,
        scan("reference.method_provisions", "provisions"),
        vec![
            ("method_id", "provisions.method_id"),
            ("property_kind_id", "provisions.property_kind_id"),
        ],
    );
    let provided_generic = join(
        generic,
        scan("reference.method_provisions", "provisions"),
        vec![("method_id", "provisions.method_id")],
    );
    let provided = P::Union(
        vec![provided_specific, provided_generic]
            .into_iter()
            .map(|input| {
                project(
                    input,
                    vec![
                        ("accepted_selection_id", E::col("selection_id")),
                        ("accepted_method_id", E::col("provisions.method_id")),
                    ],
                )
            })
            .collect(),
    );
    emit(
        builder,
        "P4.method_compatible",
        provided.clone(),
        "accepted_selection_id",
        "accepted_method_id",
        true,
        "applicable",
    );
    let missing = P::AntiJoin {
        left: Box::new(family),
        right: Box::new(provided),
        keys: (vec![
            ("selection_id", "accepted_selection_id"),
            ("method_id", "accepted_method_id"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    emit(
        builder,
        "P4.method_missing_provision",
        missing,
        "selection_id",
        "method_id",
        false,
        "missing_provision",
    );
    let wrong_family = P::Filter {
        input: Box::new(matched),
        predicate: E::Not(Box::new(family_matches)),
    };
    emit(
        builder,
        "P4.method_family_mismatch",
        wrong_family,
        "selections.selection_id",
        "selections.method_id",
        false,
        "family_mismatch",
    );
    let missing_method = P::AntiJoin {
        left: Box::new(selections),
        right: Box::new(scan("reference.method_specs", "methods")),
        keys: (vec![("selections.method_id", "methods.method_id")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    };
    emit(
        builder,
        "P4.method_absent",
        missing_method,
        "selections.selection_id",
        "selections.method_id",
        false,
        "missing_provision",
    );
}
fn emit(
    builder: &mut RegistryBuilder,
    name: &'static str,
    input: P,
    selection: &'static str,
    method: &'static str,
    compatible: bool,
    reason: &'static str,
) {
    builder.declare_rule(
        RuleDecl::new(
            name,
            "1",
            0,
            RuleHead::Relation("inferred.method_compatibility".to_owned()),
            P::Union(vec![project(
                input,
                vec![
                    ("selection_id", E::col(selection)),
                    ("method_id", E::col(method)),
                    ("compatible", E::Lit(Cell::Bool(compatible))),
                    ("reason", E::Lit(Cell::Enum(reason))),
                    ("derivation_id", E::col(selection)),
                ],
            )]),
        )
        .assertions("provenance.method_compatibility_assertions")
        .stratified_negation(),
    );
}
