// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Scope construction is an ordinary native rule program with exact source lineage.

use super::{
    Cell, E, N, P, RegistryBuilder, RuleDecl, RuleHead, S, T, assertion, column, equals, join,
    project, provenance, relation, scan,
};
use crate::model::InvariantKind;

pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (head, assertions) in [
        ("inferred.scope_candidates", "scope_candidate_assertions"),
        ("inferred.selector_contexts", "selector_context_assertions"),
    ] {
        assertion(builder, head, assertions);
    }
    relation(
        builder,
        N::Inferred,
        "selector_parameter_targets",
        S::Derived,
        &["scope_id", "node_id"],
        vec![
            column("scope_id", T::id()),
            column("node_id", T::id()),
            column("target_entity_id", T::id()),
            provenance(),
        ],
        "Actual typed configured instance targets for each relative selector context.",
    );
    assertion(
        builder,
        "inferred.selector_parameter_targets",
        "selector_parameter_target_assertions",
    );
    scopes(builder);
    parameter_targets(builder);
    contexts(builder);
    obligations(builder);
}

fn scopes(builder: &mut RegistryBuilder) {
    let global = anti(
        scan("normalized.scopes", "scopes"),
        scan("normalized.template_scopes", "declarations"),
        vec![("scopes.scope_id", "declarations.scope_id")],
    );
    emit(
        builder,
        "P5.global_scope_candidates",
        6,
        "inferred.scope_candidates",
        "provenance.scope_candidate_assertions",
        global,
        vec![
            ("scope_id", E::col("scopes.scope_id")),
            ("scope_decl_id", E::col("scopes.scope_id")),
            ("owner_instance_id", E::Lit(Cell::Null)),
            ("derivation_id", E::col("scopes.scope_id")),
        ],
    );
    let relative = join(
        join(
            scan("normalized.scopes", "scopes"),
            scan("normalized.template_scopes", "declarations"),
            vec![("scopes.scope_id", "declarations.scope_id")],
        ),
        scan("normalized.instance_bindings", "instances"),
        vec![("declarations.template_id", "instances.template_id")],
    );
    emit(
        builder,
        "P5.relative_scope_candidates",
        6,
        "inferred.scope_candidates",
        "provenance.scope_candidate_assertions",
        relative,
        vec![
            (
                "scope_id",
                E::call(
                    "pse_named_id",
                    vec![
                        E::col("instances.instance_id"),
                        concat(vec![
                            E::Lit(Cell::text("pse:scope:v1:")),
                            hex(E::col("scopes.scope_id")),
                        ]),
                    ],
                    T::id(),
                    false,
                ),
            ),
            ("scope_decl_id", E::col("scopes.scope_id")),
            ("owner_instance_id", E::col("instances.instance_id")),
            ("derivation_id", E::col("scopes.scope_id")),
        ],
    );
}

fn nodes() -> P {
    join(
        scan("inferred.scope_candidates", "scopes"),
        scan("normalized.selector_nodes", "nodes"),
        vec![("scopes.scope_decl_id", "nodes.scope_decl_id")],
    )
}

fn parameter_targets(builder: &mut RegistryBuilder) {
    let source = filter(
        nodes(),
        equals(E::col("nodes.op"), Cell::Enum("instance_parameter")),
    );
    let source = join(
        source,
        scan("normalized.instance_bindings", "owner"),
        vec![("scopes.owner_instance_id", "owner.instance_id")],
    );
    let declarations = filter(
        scan("normalized.template_params", "parameters"),
        E::IsNull(Box::new(E::col("parameters.enum_id"))),
    );
    let source = join(
        source,
        declarations,
        vec![
            ("owner.template_id", "parameters.template_id"),
            ("nodes.parameter_name", "parameters.name"),
        ],
    );
    let logical = filter(
        scan("reference.schema_logical_types", "logical"),
        equals(E::col("logical.name"), Cell::text("semantic_id")),
    );
    let source = join(
        source,
        logical,
        vec![("parameters.logical_type_id", "logical.logical_type_id")],
    );
    let values = filter(
        scan("normalized.config_values", "values"),
        E::And(vec![
            equals(E::col("values.category"), Cell::Enum("parameter")),
            equals(field("values.value", "kind"), Cell::Enum("semantic_id")),
            E::IsNotNull(Box::new(field("values.value", "semantic_id"))),
        ]),
    );
    let values = project(
        values,
        vec![
            ("value_owner_id", E::col("values.owner_id")),
            ("value_name", E::col("values.name")),
            ("value_target_id", field("values.value", "semantic_id")),
        ],
    );
    let source = join(
        source,
        values,
        vec![
            ("owner.instance_id", "value_owner_id"),
            ("nodes.parameter_name", "value_name"),
        ],
    );
    let source = join(
        source,
        scan("normalized.instance_bindings", "target"),
        vec![("value_target_id", "target.instance_id")],
    );
    emit(
        builder,
        "P5.selector_parameter_target",
        7,
        "inferred.selector_parameter_targets",
        "provenance.selector_parameter_target_assertions",
        source,
        vec![
            ("scope_id", E::col("scopes.scope_id")),
            ("node_id", E::col("nodes.node_id")),
            ("target_entity_id", E::col("target.instance_id")),
            ("derivation_id", E::col("nodes.node_id")),
        ],
    );
}

fn contexts(builder: &mut RegistryBuilder) {
    let ordinary = filter(
        nodes(),
        E::Not(Box::new(E::InList {
            expr: Box::new(E::col("nodes.op")),
            list: vec![Cell::Enum("self"), Cell::Enum("instance_parameter")],
        })),
    );
    context(
        builder,
        "P5.selector_absolute_context",
        ordinary,
        E::col("nodes.entity_id"),
    );
    context(
        builder,
        "P5.selector_self_context",
        filter(nodes(), equals(E::col("nodes.op"), Cell::Enum("self"))),
        E::col("scopes.owner_instance_id"),
    );
    let parameter = filter(
        nodes(),
        equals(E::col("nodes.op"), Cell::Enum("instance_parameter")),
    );
    let keys = vec![
        ("scopes.scope_id", "targets.scope_id"),
        ("nodes.node_id", "targets.node_id"),
    ];
    let targets = || scan("inferred.selector_parameter_targets", "targets");
    context(
        builder,
        "P5.selector_parameter_context",
        join(parameter.clone(), targets(), keys.clone()),
        E::col("targets.target_entity_id"),
    );
    context(
        builder,
        "P5.selector_parameter_missing",
        anti(parameter, targets(), keys),
        E::Lit(Cell::Null),
    );
}

fn context(builder: &mut RegistryBuilder, name: &'static str, input: P, target: E) {
    emit(
        builder,
        name,
        8,
        "inferred.selector_contexts",
        "provenance.selector_context_assertions",
        input,
        vec![
            ("scope_id", E::col("scopes.scope_id")),
            ("node_id", E::col("nodes.node_id")),
            ("op", E::col("nodes.op")),
            ("left_node_id", E::col("nodes.left_node_id")),
            ("right_node_id", E::col("nodes.right_node_id")),
            ("target_entity_id", target),
            ("target_kind", E::col("nodes.entity_kind")),
            ("constant", E::col("nodes.constant")),
            ("derivation_id", E::col("nodes.node_id")),
        ],
    );
}

fn obligations(builder: &mut RegistryBuilder) {
    let keys = ["scope_id", "node_id"];
    for (name, invalid, message) in [
        (
            "relative_target_present",
            E::And(vec![
                E::InList {
                    expr: Box::new(E::col("op")),
                    list: vec![Cell::Enum("self"), Cell::Enum("instance_parameter")],
                },
                E::IsNull(Box::new(E::col("target_entity_id"))),
            ]),
            "A relative selector requires an actual, uniquely configured instance target.",
        ),
        (
            "kind_has_actual_universe",
            E::And(vec![
                equals(E::col("op"), Cell::Enum("kind_is")),
                E::Not(Box::new(E::IsTrue(Box::new(E::InList {
                    expr: Box::new(E::col("target_kind")),
                    list: vec![Cell::Enum("instance"), Cell::Enum("port")],
                })))),
            ]),
            "The selected entity kind must have an explicitly admitted finite universe.",
        ),
    ] {
        super::super::inv::declare(
            builder,
            "inferred.selector_contexts",
            name,
            InvariantKind::Check,
            &keys,
            project(
                filter(scan("inferred.selector_contexts", "subject"), invalid),
                keys.iter().map(|name| (*name, E::col(*name))).collect(),
            ),
            message,
        );
    }
}

fn hex(value: E) -> E {
    total(E::call(
        "encode",
        vec![value, E::Lit(Cell::text("hex"))],
        T::native(arrow_schema::DataType::Utf8),
        true,
    ))
}
fn concat(values: Vec<E>) -> E {
    total(E::call(
        "concat",
        values,
        T::native(arrow_schema::DataType::Utf8),
        true,
    ))
}
fn total(value: E) -> E {
    E::call(
        "coalesce",
        vec![value, E::Lit(Cell::text(""))],
        T::native(arrow_schema::DataType::Utf8),
        false,
    )
}
fn field(column: &'static str, name: &'static str) -> E {
    E::Field {
        expr: Box::new(E::col(column)),
        name: name.into(),
    }
}
fn filter(input: P, predicate: E) -> P {
    P::Filter {
        input: Box::new(input),
        predicate,
    }
}
fn anti(left: P, right: P, keys: Vec<(&'static str, &'static str)>) -> P {
    P::AntiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: keys
            .into_iter()
            .map(|(l, r)| (l.into(), r.into()))
            .collect(),
    }
}
fn emit(
    builder: &mut RegistryBuilder,
    name: &'static str,
    stratum: u16,
    head: &'static str,
    assertions: &'static str,
    input: P,
    columns: Vec<(&str, E)>,
) {
    builder.declare_rule(
        RuleDecl::new(
            name,
            "1",
            stratum,
            RuleHead::Relation(head.to_owned()),
            P::Union(vec![project(input, columns)]),
        )
        .assertions(assertions)
        .stratified_negation(),
    );
}
