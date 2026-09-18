// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Indexed realization, exhaustive law participation and graph roots.
use super::{
    ExtensionUse, N, RegistryBuilder, S, T, column, derived, enumeration, index, relation,
};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    derived(
        builder,
        N::Compiled,
        "predicate_masks",
        &["group_id"],
        vec![
            column("group_id", T::id()),
            column("instance_id", T::id()),
            column("source_id", T::id()),
            column("predicate_id", T::nonnegative(i64::MAX)),
            column("product_id", T::id()),
            column("quantity_type_id", T::id()),
        ],
        "A Boolean expression group derived from every actual finite predicate outcome; no authored symbol is fabricated.",
    );
    derived(
        builder,
        N::Compiled,
        "predicate_mask_members",
        &["group_id", "index"],
        vec![
            column("group_id", T::id()).with_fk("compiled.predicate_masks", "group_id"),
            column("index", index()),
            column("symbol_id", T::id()),
            column("node_id", T::nonnegative(i64::MAX)),
            column("value", T::native(arrow_schema::DataType::Boolean)),
        ],
        "Exact predicate outcome to constant-expression symbol correspondence, checked again at P10.",
    );
    relation(
        builder,
        N::Authored,
        "template_symbol_expressions",
        S::Model,
        &["symbol_decl_id"],
        vec![
            column("symbol_decl_id", T::id())
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("expression", T::extended(ExtensionUse::ExprDsl)),
        ],
        "The sole expression-role symbol body; owner and role must match the symbol declaration.",
    );
    derived(
        builder,
        N::Compiled,
        "symbol_expressions",
        &["symbol_id"],
        vec![
            column("symbol_id", T::id()),
            column("node_id", T::nonnegative(i64::MAX)),
        ],
        "Actual expression symbol body, rooted and physically checked at P10.",
    );
    enumeration(
        builder,
        "ContributionSubjectKind",
        [
            "total",
            "species",
            "phase_species",
            "element",
            "energy",
            "momentum",
        ],
    );
    enumeration(builder, "ParticipationDecision", ["included", "excluded"]);
    enumeration(builder, "TearMethod", ["feedback_arc_set"]);
    enumeration(
        builder,
        "ExpressionRootRole",
        [
            "symbol_expression",
            "contribution",
            "display",
            "method_output",
            "guard",
        ],
    );
    let mut contributions = vec![
        column("contribution_id", T::id()),
        column("contribution_decl_id", T::id()),
        column("owner_instance_id", T::id()),
        column("scope_id", T::id()),
        column("product_id", T::id()),
        column("source_id", T::id()),
        column("expression_root", T::nonnegative(i64::MAX)),
        column("law_family", T::enumeration("LawFamily")),
        column("quantity_type_id", T::id()),
        column("basis_id", T::id()).optional(),
        column("orientation", T::enumeration("Orientation")),
        column("transfer_connection_id", T::id()).optional(),
    ];
    contributions.push(column(
        "subject",
        super::conservation_values::subject(false),
    ));
    derived(
        builder,
        N::Compiled,
        "contributions",
        &["contribution_id"],
        contributions,
        "Instantiated contribution with complete physical subject and domain contract.",
    );
    let mut laws = vec![
        column("application_id", T::id()),
        column("law_instance_decl_id", T::id()),
        column("law_template_id", T::id()),
        column("owner_instance_id", T::id()),
        column("scope_id", T::id()),
        column("product_id", T::id()),
        column("law_family", T::enumeration("LawFamily")),
        column("quantity_type_id", T::id()),
        column("basis_id", T::id()).optional(),
    ];
    laws.push(column("subject", super::conservation_values::subject(true)));
    laws.extend([
        column("source_family", T::enumeration("LawFamily")),
        column("subject_projection", T::enumeration("LawSubjectProjection")),
        column("expansion", T::enumeration("LawExpansion")),
    ]);
    derived(
        builder,
        N::Compiled,
        "law_applications",
        &["application_id"],
        laws,
        "Bound conservation law, preserving its scope, subject, basis and indexed shape.",
    );
    derived(
        builder,
        N::Compiled,
        "law_participation",
        &["application_id", "contribution_id"],
        vec![
            column("application_id", T::id()),
            column("contribution_id", T::id()),
            column("decision", super::conservation_values::participation()),
        ],
        "Disjoint exhaustive candidate partition; conversions and exclusions are actual witnesses.",
    );
    derived(
        builder,
        N::Inferred,
        "tear_candidates",
        &["connection_id"],
        vec![
            column("connection_id", T::id()),
            column("edge_group_id", T::id()),
            column("cost", T::native(arrow_schema::DataType::Float64)),
            column("chosen", T::native(arrow_schema::DataType::Boolean)),
            column("method", T::enumeration("TearMethod")),
            column("ordinal", T::nonnegative(i64::MAX)),
        ],
        "Deterministic heuristic edge groups; actual chosen removals must make topology acyclic.",
    );
    derived(
        builder,
        N::Compiled,
        "expression_roots",
        &["owner_id", "role", "ordinal"],
        vec![
            column("owner_id", T::id()),
            column("role", T::enumeration("ExpressionRootRole")),
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("node_id", T::nonnegative(i64::MAX)),
        ],
        "Explicit non-equation expression occurrence roots preserved through P10.",
    );
}
