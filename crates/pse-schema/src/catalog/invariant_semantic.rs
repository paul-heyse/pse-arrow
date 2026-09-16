// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declarative conditional shapes consumed by semantic compilation (blueprint §6.15).
use super::inv::{declare as invariant, filter, project, scan};
use crate::RegistryBuilder;
use crate::model::{Cell, CmpOp, InvariantKind, RuleExpr as E};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    check(
        builder,
        "normalized.expression_sources",
        "owner_alternative",
        E::Or(vec![
            E::And(vec![
                eq("owner_kind", "template"),
                present("owner_template_id"),
                absent("owner_instance_id"),
            ]),
            E::And(vec![
                eq("owner_kind", "instance"),
                absent("owner_template_id"),
                present("owner_instance_id"),
            ]),
        ]),
    );
    check(
        builder,
        "authored.template_domain_bindings",
        "source_alternative",
        E::Or(vec![
            E::And(vec![
                eq("source", "domain"),
                present("domain_id"),
                absent("parameter_name"),
            ]),
            E::And(vec![
                eq("source", "parameter"),
                absent("domain_id"),
                present("parameter_name"),
            ]),
            E::And(vec![
                E::Or(vec![
                    eq("source", "species"),
                    eq("source", "phase"),
                    eq("source", "phase_species"),
                    eq("source", "element"),
                ]),
                absent("domain_id"),
                absent("parameter_name"),
            ]),
        ]),
    );
    check(
        builder,
        "inferred.method_resolutions",
        "winner_alternative",
        E::Or(vec![
            E::And(vec![
                eq("status", "resolved"),
                present("method_id"),
                eq("realization", "equation_template"),
                present("template_id"),
            ]),
            E::And(vec![
                eq("status", "resolved"),
                present("method_id"),
                eq("realization", "kernel"),
                absent("template_id"),
            ]),
            E::And(vec![
                E::Or(vec![eq("status", "unresolved"), eq("status", "ambiguous")]),
                absent("method_id"),
                absent("realization"),
                absent("template_id"),
            ]),
        ]),
    );
    check(
        builder,
        "reference.method_specs",
        "realization_alternative",
        E::Or(vec![
            E::And(vec![
                eq("realization", "equation_template"),
                present("template_id"),
                absent("kernel_id"),
            ]),
            E::And(vec![
                eq("realization", "kernel"),
                absent("template_id"),
                present("kernel_id"),
            ]),
        ]),
    );
    check(
        builder,
        "reference.method_provisions",
        "output_alternative",
        E::Or(vec![
            E::And(vec![
                eq("output_kind", "template_symbol"),
                present("symbol_decl_id"),
                absent("kernel_output_ordinal"),
            ]),
            E::And(vec![
                eq("output_kind", "kernel_output"),
                absent("symbol_decl_id"),
                present("kernel_output_ordinal"),
            ]),
        ]),
    );
    check(
        builder,
        "compiled.method_realizations",
        "output_alternative",
        E::Or(vec![
            E::And(vec![
                eq("output_kind", "template_symbol"),
                present("template_instance_id"),
                present("output_symbol_id"),
                absent("kernel_binding_id"),
                absent("kernel_output_ordinal"),
            ]),
            E::And(vec![
                eq("output_kind", "kernel_output"),
                absent("template_instance_id"),
                absent("output_symbol_id"),
                present("kernel_binding_id"),
                present("kernel_output_ordinal"),
            ]),
        ]),
    );
    for relation in [
        "authored.template_contribution_contracts",
        "compiled.contributions",
        "compiled.law_applications",
    ] {
        check(
            builder,
            relation,
            "subject_alternative",
            E::And(vec![
                E::Not(Box::new(E::And(vec![
                    present("subject_axis"),
                    present("phase_axis"),
                    E::cmp(CmpOp::Eq, E::col("subject_axis"), E::col("phase_axis")),
                ]))),
                E::Or(vec![
                    E::And(vec![
                        E::Or(vec![
                            eq("subject_kind", "total"),
                            eq("subject_kind", "energy"),
                            eq("subject_kind", "momentum"),
                        ]),
                        absent("subject_id"),
                        absent("subject_axis"),
                        coordinate("phase_id", "phase_axis", false),
                    ]),
                    E::And(vec![
                        E::Or(vec![
                            eq("subject_kind", "species"),
                            eq("subject_kind", "element"),
                        ]),
                        coordinate("subject_id", "subject_axis", true),
                        absent("phase_id"),
                        absent("phase_axis"),
                    ]),
                    E::And(vec![
                        eq("subject_kind", "phase_species"),
                        coordinate("subject_id", "subject_axis", true),
                        coordinate("phase_id", "phase_axis", true),
                    ]),
                ]),
            ]),
        );
    }
    check(
        builder,
        "compiled.law_participation",
        "decision_alternative",
        E::Or(vec![
            E::And(vec![
                eq("decision", "included"),
                eq("reason", "matched"),
                E::InList {
                    expr: Box::new(E::col("sign")),
                    list: vec![Cell::I64(-1), Cell::I64(1)],
                },
            ]),
            E::And(vec![
                eq("decision", "excluded"),
                E::Not(Box::new(eq("reason", "matched"))),
                E::cmp(CmpOp::Eq, E::col("sign"), E::Lit(Cell::I64(0))),
                absent("conversion_id"),
            ]),
        ]),
    );
    check(
        builder,
        "inferred.tear_candidates",
        "nonnegative_cost",
        E::cmp(CmpOp::GtEq, E::col("cost"), E::Lit(Cell::F64(0.0))),
    );
}
fn coordinate(fixed: &'static str, axis: &'static str, required: bool) -> E {
    let alternatives = E::Or(vec![
        E::And(vec![present(fixed), absent(axis)]),
        E::And(vec![absent(fixed), present(axis)]),
    ]);
    if required {
        alternatives
    } else {
        E::Or(vec![
            alternatives,
            E::And(vec![absent(fixed), absent(axis)]),
        ])
    }
}

fn eq(name: &'static str, value: &'static str) -> E {
    E::cmp(CmpOp::Eq, E::col(name), E::Lit(Cell::Enum(value)))
}
fn absent(name: &'static str) -> E {
    E::IsNull(Box::new(E::col(name)))
}
fn present(name: &'static str) -> E {
    E::IsNotNull(Box::new(E::col(name)))
}
fn check(builder: &mut RegistryBuilder, relation: &'static str, name: &'static str, valid: E) {
    let Some(keys) = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == relation)
        .and_then(|spec| spec.primary_key.clone())
    else {
        return;
    };
    // Unknown is invalid at this total admission boundary as well as false.
    let invalid = E::Not(Box::new(E::IsTrue(Box::new(valid))));
    invariant(
        builder,
        relation,
        name,
        InvariantKind::Check,
        &keys,
        project(filter(scan(relation, "subject"), invalid), &keys),
        "The selected semantic alternative has exactly its declared payload.",
    );
}
