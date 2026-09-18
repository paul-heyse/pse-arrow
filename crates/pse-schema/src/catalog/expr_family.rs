// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Projections of authored and mathematical schemas into P3's derived namespace.

use crate::builder::RegistryBuilder;
use crate::model::{
    Authority, DerivationGranularity, ExtensionUse, FieldContract, Namespace, SnapshotClass,
};

/// Declares normalized copies and the five parsed expression families.
pub fn declare(builder: &mut RegistryBuilder) {
    let copies: Vec<_> = builder
        .declared_relations()
        .iter()
        .filter(|spec| {
            spec.key.namespace == Namespace::Authored && spec.snapshot_class == SnapshotClass::Model
        })
        .cloned()
        .collect();
    for mut spec in copies {
        spec.key.namespace = Namespace::Normalized;
        spec.authority = Authority::Derived;
        spec.snapshot_class = SnapshotClass::Derived;
        spec.derivation_granularity = Some(DerivationGranularity::Row);
        // Configuration introduces actual child instances and finite domains.
        // Normalized references bind that complete inventory; authored references
        // retain their original source-only targets.
        for column in &mut spec.columns {
            if let Some(fk) = column.fk() {
                let target = match fk.relation {
                    "authored.domains" => "normalized.domains",
                    "authored.instances" => "normalized.instance_bindings",
                    other => other,
                };
                *column = column.clone().with_fk(target, fk.column);
            }
        }
        builder.declare_relation(spec);
    }
    for prefix in ["template", "instance", "display", "contribution", "guard"] {
        declare_expression_family(builder, prefix);
    }
    declare_instantiated(builder);
}

/// P7–P9 keep deferred physical requests, but cannot keep unresolved template bindings.
fn declare_instantiated(builder: &mut RegistryBuilder) {
    let sources: Vec<_> = builder
        .declared_relations()
        .iter()
        .filter(|spec| {
            spec.key.namespace == Namespace::Compiled
                && (spec.key.name.starts_with("math_") || spec.key.name == "kernel_bindings")
        })
        .cloned()
        .collect();
    for mut spec in sources {
        spec.key.namespace = Namespace::Inferred;
        if spec.key.name == "math_expr_nodes" {
            bind_expression(&mut spec);
        }
        if matches!(spec.key.name, "math_equations" | "math_indexed_equations") {
            for column in &mut spec.columns {
                if column.name() == "constraint" {
                    *column = FieldContract::payload(
                        "constraint",
                        super::s6_9_math::equation_constraint("inferred.math_expr_nodes"),
                        "Equation comparison and bound nodes.",
                    );
                }
            }
        }
        builder.declare_relation(spec);
    }
}

/// Copies field contracts from the sole mathematical declaration, changing only the family identity.
fn declare_expression_family(builder: &mut RegistryBuilder, prefix: &str) {
    let names = names_for(prefix);
    let sources: Vec<_> = builder
        .declared_relations()
        .iter()
        .filter(|spec| {
            spec.key.namespace == Namespace::Compiled
                && names.iter().any(|(name, _)| *name == spec.key.name)
        })
        .cloned()
        .collect();
    for mut spec in sources {
        if let Some((_, target)) = names.iter().find(|(name, _)| *name == spec.key.name) {
            spec.key.namespace = Namespace::Normalized;
            spec.key.name = target;
            bind_expression(&mut spec);
            if spec
                .columns
                .first()
                .is_some_and(|column| column.name() == "node_id")
                && spec.columns.iter().any(|column| column.name() == "opcode")
            {
                spec.columns.push(FieldContract::provenance(
                    "derivation_id",
                    FieldContract::id(),
                    "The source-expression derivation.",
                ));
                spec.columns.push(FieldContract::provenance(
                    "source_span",
                    FieldContract::extended(ExtensionUse::SourceSpan),
                    "Exact authored expression byte span.",
                ));
            }
            builder.declare_relation(spec);
        }
    }
}

/// The sole mathematical-to-normalized relation mapping used by generated declarations and adapters.
#[must_use]
pub fn target_name(prefix: &str, source: &str) -> Option<&'static str> {
    names_for(prefix)
        .iter()
        .find(|(name, _)| *name == source)
        .map(|(_, target)| *target)
}
fn names_for(prefix: &str) -> &'static [(&'static str, &'static str)] {
    match prefix {
        "template" => TEMPLATE_NAMES,
        "instance" => INSTANCE_NAMES,
        "display" => DISPLAY_NAMES,
        "contribution" => CONTRIBUTION_NAMES,
        "guard" => GUARD_NAMES,
        _ => &[],
    }
}
const TEMPLATE_NAMES: &[(&str, &str)] = &[("math_expr_nodes", "template_expr_nodes")];
const INSTANCE_NAMES: &[(&str, &str)] = &[("math_expr_nodes", "instance_expr_nodes")];
const DISPLAY_NAMES: &[(&str, &str)] = &[("math_expr_nodes", "display_expr_nodes")];
const CONTRIBUTION_NAMES: &[(&str, &str)] = &[("math_expr_nodes", "contribution_expr_nodes")];
const GUARD_NAMES: &[(&str, &str)] = &[("math_expr_nodes", "guard_expr_nodes")];

fn bind_expression(spec: &mut crate::model::RelationDecl) {
    let target = format!("{}.{}", spec.key.namespace, spec.key.name);
    for column in &mut spec.columns {
        match column.name() {
            "payload" => {
                *column = FieldContract::payload(
                    "payload",
                    super::math_value::payload(&target),
                    "payload",
                );
            }
            "children" => {
                *column = FieldContract::payload(
                    "children",
                    FieldContract::list(
                        FieldContract::nonnegative(i64::MAX).with_fk(&target, "node_id"),
                    ),
                    "children",
                );
            }
            _ => {}
        }
    }
    spec.checks = super::math_value::checks(spec.key.namespace);
}
