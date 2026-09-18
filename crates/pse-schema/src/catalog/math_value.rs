// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Coherent mathematical values, shared by every expression family.

use arrow_schema::DataType;
use std::fmt::Write;

use super::declarations::enumeration;
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, TaggedAlternative},
};

pub(super) fn record(fields: Vec<(&str, T)>) -> T {
    T::structure(
        fields
            .into_iter()
            .map(|(name, field)| field.with_name(name))
            .collect(),
    )
}
fn ordinal() -> T {
    T::nonnegative(i64::MAX)
}
fn real() -> T {
    T::native(DataType::Float64)
}
fn text() -> T {
    T::native(DataType::Utf8)
}

pub(super) fn alternatives(kind: &'static str, arms: Vec<(&str, T)>, empty: &[&str]) -> T {
    let mut alternative = TaggedAlternative::new(
        "kind",
        arms.iter()
            .map(|(name, _)| ((*name).to_owned(), (*name).to_owned())),
    );
    for tag in empty {
        alternative = alternative.with_unit(*tag);
    }
    let mut fields = vec![T::enumeration(kind).with_name("kind")];
    fields.extend(
        arms.into_iter()
            .map(|(name, field)| field.with_name(name).optional()),
    );
    T::structure(fields).with_alternative(&alternative)
}

pub(super) fn domain() -> T {
    alternatives(
        "MathDomainKind",
        vec![
            ("actual", record(vec![("domain_id", T::id())])),
            (
                "template",
                record(vec![("template_id", T::id()), ("name", text())]),
            ),
        ],
        &[],
    )
}

fn guard(target: &str) -> T {
    alternatives(
        "MathGuardKind",
        vec![
            (
                "math",
                record(vec![("node_id", ordinal().with_fk(target, "node_id"))]),
            ),
            (
                "predicate",
                record(vec![("source_id", T::id()), ("predicate_id", ordinal())]),
            ),
        ],
        &[],
    )
}

fn reference() -> T {
    alternatives(
        "MathReferenceKind",
        vec![
            ("symbol", record(vec![("symbol_id", T::id())])),
            (
                "template",
                record(vec![
                    ("template_id", T::id()),
                    ("member_kind", T::enumeration("MathTemplateMemberKind")),
                    ("name", text()),
                ]),
            ),
            ("domain", record(vec![("value", domain())])),
            ("index", record(vec![("bound_index_id", T::id())])),
        ],
        &[],
    )
}

/// The node payload is one tagged value, never a collection of independently keyed rows.
#[expect(
    clippy::too_many_lines,
    reason = "one declarative inventory of the complete mathematical payload family"
)]
fn arms(target: &str) -> Vec<(&'static str, T)> {
    vec![
        ("symbol", record(vec![("reference", reference())])),
        (
            "float",
            record(vec![("value", real()), ("unit_id", T::id())]),
        ),
        (
            "integer",
            record(vec![("value", T::native(DataType::Int64))]),
        ),
        (
            "affine",
            record(vec![
                ("constant", real()),
                ("constant_quantity_type_id", T::id().optional()),
                ("constant_unit_id", T::id().optional()),
                ("coefficients", T::list(real())),
            ]),
        ),
        (
            "weighted_mean",
            record(vec![
                (
                    "pairs",
                    T::list(record(vec![
                        ("weight_node_id", ordinal().with_fk(target, "node_id")),
                        ("value_node_id", ordinal().with_fk(target, "node_id")),
                    ])),
                ),
                ("normalization", T::enumeration("WeightNormalization")),
                ("unit_sum_invariant_id", T::id().optional()),
            ]),
        ),
        (
            "reduction",
            record(vec![
                ("reduction_kind", T::enumeration("ReductionKind")),
                ("domain", domain()),
                ("bound_index_id", T::id()),
                ("filter", guard(target).optional()),
            ]),
        ),
        (
            "gather",
            record(vec![
                ("group_id", T::id()),
                (
                    "coordinates",
                    T::list(record(vec![
                        ("bound_index_id", T::id()),
                        ("position", T::nonnegative(i64::from(u16::MAX))),
                    ])),
                ),
            ]),
        ),
        (
            "pending_gather",
            record(vec![
                ("group_id", T::id()),
                ("indices", T::list(ordinal().with_fk(target, "node_id"))),
            ]),
        ),
        (
            "pending_path",
            record(vec![
                ("source_id", T::id()),
                ("path_id", ordinal()),
                ("indices", T::list(ordinal().with_fk(target, "node_id"))),
            ]),
        ),
        (
            "broadcast",
            record(vec![("domain", domain()), ("bound_index_id", T::id())]),
        ),
        (
            "derivative",
            record(vec![
                ("domain", domain()),
                ("order", T::nonnegative(i64::from(u8::MAX))),
            ]),
        ),
        (
            "integral",
            record(vec![
                ("domain", domain()),
                ("bound_index_id", T::id()),
                ("quadrature_policy_id", T::id().optional()),
                ("filter", guard(target).optional()),
            ]),
        ),
        ("smooth", record(vec![("eps", real())])),
        (
            "pending_smooth",
            record(vec![("eps", real()), ("unit_id", T::id())]),
        ),
        ("conditional", record(vec![("guard", guard(target))])),
        (
            "kernel_call",
            record(vec![
                ("binding_id", T::id()),
                ("output_ordinal", T::nonnegative(i64::from(u16::MAX))),
            ]),
        ),
        (
            "implicit_ref",
            record(vec![
                ("system_id", T::id()),
                ("unknown_ordinal", T::nonnegative(i64::from(u16::MAX))),
            ]),
        ),
        (
            "unit_convert",
            record(vec![
                ("scale", real()),
                ("offset", real()),
                ("from_unit_id", T::id()),
                ("to_unit_id", T::id()),
            ]),
        ),
        (
            "pending_unit_convert",
            record(vec![("to_unit_id", T::id())]),
        ),
        (
            "piecewise_linear",
            record(vec![
                (
                    "breakpoints",
                    T::list(record(vec![("x", real()), ("y", real())])),
                ),
                ("input_quantity_type_id", T::id()),
                ("output_quantity_type_id", T::id()),
            ]),
        ),
    ]
}

/// Complete payload declaration, including the payload-free alternative.
pub(super) fn payload(target: &str) -> T {
    alternatives("MathPayloadKind", arms(target), &["none"])
}

pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "MathTemplateMemberKind",
        crate::math::TemplateValueKind::ALL.map(crate::math::TemplateValueKind::as_str),
    );
    enumeration(builder, "MathDomainKind", ["actual", "template"]);
    enumeration(builder, "MathGuardKind", ["math", "predicate"]);
    enumeration(
        builder,
        "MathReferenceKind",
        ["symbol", "template", "domain", "index"],
    );
    enumeration(
        builder,
        "MathPayloadKind",
        std::iter::once("none").chain(
            arms("compiled.math_expr_nodes")
                .into_iter()
                .map(|(name, _)| name),
        ),
    );
}

/// Native row predicates shared by candidate admission and durable Delta CHECK.
pub(super) fn checks(
    namespace: crate::model::Namespace,
) -> std::collections::BTreeMap<String, String> {
    use crate::math::{
        Arity,
        operators::{OPERATOR_TABLE, payload_kinds, reduction_kind},
    };
    use crate::model::Namespace;
    let mut arity = String::from("CASE opcode");
    let mut payload = String::from("CASE opcode");
    for spec in &OPERATOR_TABLE {
        let condition = match spec.arity {
            Arity::Fixed(count) => format!("array_length(children) = {count}"),
            Arity::Payload => "array_length(children) = 0".into(),
            Arity::Variadic => "TRUE".into(),
        };
        let _ = write!(arity, " WHEN '{}' THEN {condition}", spec.opcode.as_str());
        let tags = payload_kinds(spec.opcode)
            .iter()
            .map(|tag| format!("'{tag}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let reduction = reduction_kind(spec.opcode).map_or_else(String::new, |kind| {
            format!(
                " AND payload['reduction']['reduction_kind'] = '{}'",
                kind.as_str()
            )
        });
        let _ = write!(
            payload,
            " WHEN '{}' THEN payload['kind'] IN ({tags}){reduction}",
            spec.opcode.as_str()
        );
    }
    arity.push_str(" ELSE FALSE END");
    payload.push_str(" ELSE FALSE END");
    let mut result = std::collections::BTreeMap::from([
        ("expression_arity".into(), arity),
        ("expression_payload".into(), payload),
        ("affine_children".into(), "payload['kind'] <> 'affine' OR array_length(payload['affine']['coefficients']) = array_length(children)".into()),
    ]);
    if namespace != Namespace::Normalized {
        let mut resolved = vec![
            "payload['kind'] NOT IN ('pending_gather', 'pending_path')".to_owned(),
            "coalesce(payload['symbol']['reference']['kind'], 'symbol') = 'symbol'".to_owned(),
        ];
        for arm in ["reduction", "broadcast", "derivative", "integral"] {
            resolved.push(format!(
                "coalesce(payload['{arm}']['domain']['kind'], 'actual') = 'actual'"
            ));
        }
        for (arm, guard) in [
            ("reduction", "filter"),
            ("integral", "filter"),
            ("conditional", "guard"),
        ] {
            resolved.push(format!(
                "coalesce(payload['{arm}']['{guard}']['kind'], 'math') = 'math'"
            ));
        }
        if namespace == Namespace::Compiled {
            resolved
                .push("payload['kind'] NOT IN ('pending_unit_convert', 'pending_smooth')".into());
        }
        result.insert("expression_resolution".into(), resolved.join(" AND "));
    }
    result
}
