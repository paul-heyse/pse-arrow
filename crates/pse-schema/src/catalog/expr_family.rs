// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Projections of authored and mathematical schemas into P3's derived namespace.

use crate::builder::RegistryBuilder;
use crate::model::{
    Authority, ColumnSpec, DerivationGranularity, ExtensionUse, LogicalType, Namespace,
    RelationDecl, SnapshotClass,
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
        builder.declare_relation(spec);
    }
    for prefix in ["template", "instance", "display", "contribution", "guard"] {
        declare_expression_family(builder, prefix);
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
            project_domain_reference(&mut spec);
            project_value_reference(&mut spec);
            project_guard_reference(&mut spec);
            project_conversion_request(&mut spec);
            project_gather_request(&mut spec);
            project_smoothing_request(&mut spec);
            spec.key.namespace = Namespace::Normalized;
            spec.key.name = target;
            if spec
                .columns
                .first()
                .is_some_and(|column| column.name == "node_id")
                && spec.columns.iter().any(|column| column.name == "opcode")
            {
                spec.columns.push(ColumnSpec::provenance(
                    "derivation_id",
                    LogicalType::id(),
                    "The source-expression derivation.",
                ));
                spec.columns.push(ColumnSpec::provenance(
                    "source_span",
                    LogicalType::Ext(ExtensionUse::SourceSpan),
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
fn project_value_reference(spec: &mut RelationDecl) {
    if spec.key.name != "math_symbol_refs" {
        return;
    }
    for column in &mut spec.columns {
        if column.name == "symbol_id" {
            column.nullable = true;
        }
    }
    spec.columns.extend([
        ColumnSpec::payload(
            "kind",
            LogicalType::enumeration("NormalizedReferenceKind"),
            "Exact reference alternative.",
        ),
        ColumnSpec::reference(
            "template_id",
            LogicalType::id(),
            "Composite template owner.",
        )
        .optional(),
        ColumnSpec::label(
            "name",
            LogicalType::Text,
            "Composite declaration or domain name.",
        )
        .optional(),
        ColumnSpec::reference("domain_id", LogicalType::id(), "Actual domain value.").optional(),
        ColumnSpec::reference(
            "bound_index_id",
            LogicalType::id(),
            "Explicit lexical index binding.",
        )
        .optional(),
    ]);
}
fn project_conversion_request(spec: &mut RelationDecl) {
    if spec.key.name != "math_unit_converts" {
        return;
    }
    for column in &mut spec.columns {
        if matches!(column.name, "from_unit_id" | "scale" | "offset") {
            column.nullable = true;
        }
    }
    spec.columns.push(ColumnSpec::payload(
        "conversion_state",
        LogicalType::enumeration("UnitConversionState"),
        "Exclusive resolved coefficients or pending target-only request.",
    ));
}

fn project_smoothing_request(spec: &mut RelationDecl) {
    if spec.key.name != "math_smooth_ops" {
        return;
    }
    spec.columns.extend([
        ColumnSpec::payload(
            "epsilon_state",
            LogicalType::enumeration("SmoothingEpsilonState"),
            "Exclusive canonical-coordinate or pending declared-unit tolerance.",
        ),
        ColumnSpec::reference(
            "eps_unit_id",
            LogicalType::id(),
            "Actual unit of a pending tolerance.",
        )
        .optional(),
    ]);
}

fn project_gather_request(spec: &mut RelationDecl) {
    if spec.key.name != "math_gathers" {
        return;
    }
    for column in &mut spec.columns {
        if column.name == "coordinate_map" {
            column.nullable = true;
        }
    }
    spec.columns.extend([
        ColumnSpec::payload(
            "gather_state",
            LogicalType::enumeration("GatherState"),
            "Exclusive resolved lexical map or pending ordered index expressions.",
        ),
        ColumnSpec::reference(
            "index_nodes",
            LogicalType::list(LogicalType::U64),
            "Ordered graph references awaiting actual template/member resolution.",
        )
        .optional(),
    ]);
}

fn project_guard_reference(spec: &mut RelationDecl) {
    let (actual, source, predicate) = match spec.key.name {
        "math_conditionals" => ("guard_node_id", "guard_source_id", "guard_predicate_id"),
        "math_reductions" | "math_integrals" => {
            ("filter_node_id", "filter_source_id", "filter_predicate_id")
        }
        _ => return,
    };
    for column in &mut spec.columns {
        if column.name == actual {
            column.nullable = true;
        }
    }
    spec.columns.extend([
        ColumnSpec::reference(source, LogicalType::id(), "Normalized predicate source.").optional(),
        ColumnSpec::reference(predicate, LogicalType::U64, "Normalized predicate ordinal.")
            .optional(),
    ]);
}

/// Normalization preserves explicit template keys until instance binding resolves them.
fn project_domain_reference(spec: &mut RelationDecl) {
    let actual = match spec.key.name {
        "math_reductions" | "math_broadcasts" | "math_integrals" => "domain_id",
        "math_derivatives" => "wrt_domain_id",
        _ => return,
    };
    for column in &mut spec.columns {
        if column.name == actual {
            column.nullable = true;
        }
    }
    spec.columns.extend([
        ColumnSpec::reference(
            "template_id",
            LogicalType::id(),
            "Owner of an unresolved template domain.",
        )
        .optional()
        .with_fk("authored.templates", "template_id"),
        ColumnSpec::label(
            "domain_name",
            LogicalType::Text,
            "Exact unresolved local template domain name.",
        )
        .optional(),
    ]);
}

const TEMPLATE_NAMES: &[(&str, &str)] = &[
    ("math_expr_nodes", "template_expr_nodes"),
    ("math_expr_args", "template_expr_args"),
    ("math_symbol_refs", "template_expr_symbol_refs"),
    ("math_float_constants", "template_expr_float_constants"),
    ("math_int_constants", "template_expr_int_constants"),
    ("math_affine", "template_expr_affine"),
    ("math_weighted_means", "template_expr_weighted_means"),
    ("math_reductions", "template_expr_reductions"),
    ("math_gathers", "template_expr_gathers"),
    ("math_broadcasts", "template_expr_broadcasts"),
    ("math_derivatives", "template_expr_derivatives"),
    ("math_integrals", "template_expr_integrals"),
    ("math_smooth_ops", "template_expr_smooth_ops"),
    ("math_conditionals", "template_expr_conditionals"),
    ("math_kernel_calls", "template_expr_kernel_calls"),
    ("math_implicit_refs", "template_expr_implicit_refs"),
    ("math_unit_converts", "template_expr_unit_converts"),
    ("math_piecewise_linear", "template_expr_piecewise_linear"),
];

const INSTANCE_NAMES: &[(&str, &str)] = &[
    ("math_expr_nodes", "instance_expr_nodes"),
    ("math_expr_args", "instance_expr_args"),
    ("math_symbol_refs", "instance_expr_symbol_refs"),
    ("math_float_constants", "instance_expr_float_constants"),
    ("math_int_constants", "instance_expr_int_constants"),
    ("math_affine", "instance_expr_affine"),
    ("math_weighted_means", "instance_expr_weighted_means"),
    ("math_reductions", "instance_expr_reductions"),
    ("math_gathers", "instance_expr_gathers"),
    ("math_broadcasts", "instance_expr_broadcasts"),
    ("math_derivatives", "instance_expr_derivatives"),
    ("math_integrals", "instance_expr_integrals"),
    ("math_smooth_ops", "instance_expr_smooth_ops"),
    ("math_conditionals", "instance_expr_conditionals"),
    ("math_kernel_calls", "instance_expr_kernel_calls"),
    ("math_implicit_refs", "instance_expr_implicit_refs"),
    ("math_unit_converts", "instance_expr_unit_converts"),
    ("math_piecewise_linear", "instance_expr_piecewise_linear"),
];

const DISPLAY_NAMES: &[(&str, &str)] = &[
    ("math_expr_nodes", "display_expr_nodes"),
    ("math_expr_args", "display_expr_args"),
    ("math_symbol_refs", "display_expr_symbol_refs"),
    ("math_float_constants", "display_expr_float_constants"),
    ("math_int_constants", "display_expr_int_constants"),
    ("math_affine", "display_expr_affine"),
    ("math_weighted_means", "display_expr_weighted_means"),
    ("math_reductions", "display_expr_reductions"),
    ("math_gathers", "display_expr_gathers"),
    ("math_broadcasts", "display_expr_broadcasts"),
    ("math_derivatives", "display_expr_derivatives"),
    ("math_integrals", "display_expr_integrals"),
    ("math_smooth_ops", "display_expr_smooth_ops"),
    ("math_conditionals", "display_expr_conditionals"),
    ("math_kernel_calls", "display_expr_kernel_calls"),
    ("math_implicit_refs", "display_expr_implicit_refs"),
    ("math_unit_converts", "display_expr_unit_converts"),
    ("math_piecewise_linear", "display_expr_piecewise_linear"),
];

const CONTRIBUTION_NAMES: &[(&str, &str)] = &[
    ("math_expr_nodes", "contribution_expr_nodes"),
    ("math_expr_args", "contribution_expr_args"),
    ("math_symbol_refs", "contribution_expr_symbol_refs"),
    ("math_float_constants", "contribution_expr_float_constants"),
    ("math_int_constants", "contribution_expr_int_constants"),
    ("math_affine", "contribution_expr_affine"),
    ("math_weighted_means", "contribution_expr_weighted_means"),
    ("math_reductions", "contribution_expr_reductions"),
    ("math_gathers", "contribution_expr_gathers"),
    ("math_broadcasts", "contribution_expr_broadcasts"),
    ("math_derivatives", "contribution_expr_derivatives"),
    ("math_integrals", "contribution_expr_integrals"),
    ("math_smooth_ops", "contribution_expr_smooth_ops"),
    ("math_conditionals", "contribution_expr_conditionals"),
    ("math_kernel_calls", "contribution_expr_kernel_calls"),
    ("math_implicit_refs", "contribution_expr_implicit_refs"),
    ("math_unit_converts", "contribution_expr_unit_converts"),
    (
        "math_piecewise_linear",
        "contribution_expr_piecewise_linear",
    ),
];

const GUARD_NAMES: &[(&str, &str)] = &[
    ("math_expr_nodes", "guard_expr_nodes"),
    ("math_expr_args", "guard_expr_args"),
    ("math_symbol_refs", "guard_expr_symbol_refs"),
    ("math_float_constants", "guard_expr_float_constants"),
    ("math_int_constants", "guard_expr_int_constants"),
    ("math_affine", "guard_expr_affine"),
    ("math_weighted_means", "guard_expr_weighted_means"),
    ("math_reductions", "guard_expr_reductions"),
    ("math_gathers", "guard_expr_gathers"),
    ("math_broadcasts", "guard_expr_broadcasts"),
    ("math_derivatives", "guard_expr_derivatives"),
    ("math_integrals", "guard_expr_integrals"),
    ("math_smooth_ops", "guard_expr_smooth_ops"),
    ("math_conditionals", "guard_expr_conditionals"),
    ("math_kernel_calls", "guard_expr_kernel_calls"),
    ("math_implicit_refs", "guard_expr_implicit_refs"),
    ("math_unit_converts", "guard_expr_unit_converts"),
    ("math_piecewise_linear", "guard_expr_piecewise_linear"),
];
