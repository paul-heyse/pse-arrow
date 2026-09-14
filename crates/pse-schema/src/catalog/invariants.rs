// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Relational invariants derived from declarations and explicitly stated domain contracts.
use super::inv::{count, declare as invariant, filter, project, scan};
use crate::RegistryBuilder;
use crate::model::{Cell, CmpOp, InvariantKind, RuleExpr, RulePlan};

/// Project every primary key and foreign key into an executable violating-keys rule.
pub fn declare(builder: &mut RegistryBuilder) {
    let relations = builder.declared_relations().to_vec();
    for relation in &relations {
        let name = relation.key.qualified_name();
        let keys = &relation.primary_key;
        if keys.is_empty() {
            continue;
        }
        let grouped = RulePlan::Aggregate {
            input: Box::new(scan(&name, "subject")),
            group: keys.clone(),
            aggregates: vec![count("__pse_count")],
        };
        let repeated = filter(
            grouped,
            RuleExpr::cmp(
                CmpOp::Gt,
                RuleExpr::Col("__pse_count"),
                RuleExpr::Lit(Cell::U64(1)),
            ),
        );
        invariant(
            builder,
            &name,
            "unique:pk",
            InvariantKind::Unique,
            keys,
            project(repeated, keys),
            "The declared primary key identifies exactly one row.",
        );
        for column in &relation.columns {
            let Some(fk) = column.fk else {
                continue;
            };
            let present = filter(
                scan(&name, "subject"),
                RuleExpr::IsNotNull(Box::new(RuleExpr::Col(column.name))),
            );
            let missing = RulePlan::AntiJoin {
                left: Box::new(present),
                right: Box::new(scan(fk.relation, "referenced")),
                keys: vec![(column.name, fk.column)],
            };
            invariant(
                builder,
                &name,
                &format!("foreign_key:{}", column.name),
                InvariantKind::ForeignKey,
                keys,
                RulePlan::Distinct(Box::new(project(missing, keys))),
                "Every present foreign-key value resolves to its declared relation and column.",
            );
        }
    }
    entity_registration(builder);
    physical_checks(builder);
    target_checks(builder);
    domain_reference_checks(builder);
    super::invariant_closure::declare(builder);
    super::invariant_domain::declare(builder);
}
fn entity_registration(builder: &mut RegistryBuilder) {
    let mut mappings = std::collections::BTreeMap::new();
    for document in builder.declared_documents() {
        for section in &document.sections {
            if let (Some(identity), Some(kind)) = (section.identity_column, section.entity_kind) {
                mappings.insert(
                    section.relation,
                    (
                        identity,
                        kind,
                        section.name_column,
                        section.naming_scope_column,
                    ),
                );
            }
        }
    }
    for (relation, (identity, kind, name, owner)) in mappings {
        let entities = filter(
            scan("authored.entities", "registered"),
            RuleExpr::cmp(
                CmpOp::Eq,
                RuleExpr::Col("kind"),
                RuleExpr::Lit(Cell::Enum(kind)),
            ),
        );
        let missing = RulePlan::AntiJoin {
            left: Box::new(scan(relation, "subject")),
            right: Box::new(entities),
            keys: vec![(identity, "entity_id")],
        };
        invariant(
            builder,
            relation,
            "closure:entity_registered",
            InvariantKind::Closure,
            &[identity],
            project(missing, &[identity]),
            "Every declared identity has an entity row with its declared entity kind.",
        );
        entity_fields(builder, relation, identity, name, owner);
    }
}
fn entity_fields(
    builder: &mut RegistryBuilder,
    relation: &'static str,
    identity: &'static str,
    name: Option<&'static str>,
    owner: Option<&'static str>,
) {
    let registered = RulePlan::Project {
        input: Box::new(scan("authored.entities", "registered")),
        columns: vec![
            ("__entity", RuleExpr::Col("entity_id")),
            ("__entity_package", RuleExpr::Col("package_id")),
            ("__entity_name", RuleExpr::Col("name")),
            ("__entity_parent", RuleExpr::Col("parent_entity_id")),
        ],
    };
    let mut joined = RulePlan::EquiJoin {
        left: Box::new(scan(relation, "subject")),
        right: Box::new(registered),
        keys: vec![(identity, "__entity")],
        null_equality: crate::model::NullEquality::NullEqualsNothing,
    };
    let mut mismatches = vec![];
    let distinct = |left, right| RuleExpr::IsDistinctFrom(Box::new(left), Box::new(right));
    if let Some(name) = name {
        mismatches.push(distinct(
            RuleExpr::Col(name),
            RuleExpr::Col("__entity_name"),
        ));
    }
    mismatches.push(distinct(
        owner.map_or(RuleExpr::Lit(Cell::Null), RuleExpr::Col),
        RuleExpr::Col("__entity_parent"),
    ));
    let has_package = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == relation)
        .is_some_and(|spec| {
            spec.columns
                .iter()
                .any(|column| column.name == "package_id")
        });
    if has_package {
        mismatches.push(distinct(
            RuleExpr::Col("package_id"),
            RuleExpr::Col("__entity_package"),
        ));
    } else if let Some(owner) = owner {
        let parents = RulePlan::Project {
            input: Box::new(scan("authored.entities", "owners")),
            columns: vec![
                ("__owner", RuleExpr::Col("entity_id")),
                ("__owner_package", RuleExpr::Col("package_id")),
            ],
        };
        joined = RulePlan::EquiJoin {
            left: Box::new(joined),
            right: Box::new(parents),
            keys: vec![(owner, "__owner")],
            null_equality: crate::model::NullEquality::NullEqualsNothing,
        };
        mismatches.push(distinct(
            RuleExpr::Col("__owner_package"),
            RuleExpr::Col("__entity_package"),
        ));
    }
    invariant(
        builder,
        relation,
        "closure:entity_fields",
        InvariantKind::Closure,
        &[identity],
        project(filter(joined, RuleExpr::Or(mismatches)), &[identity]),
        "Entity names, explicit parents and available declaring-package facts match their actual source declarations.",
    );
}

fn check(
    builder: &mut RegistryBuilder,
    relation: &str,
    name: &str,
    predicate: RuleExpr,
    doc: &'static str,
) {
    let keys = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == relation)
        .map(|spec| spec.primary_key.clone())
        .unwrap_or_default();
    invariant(
        builder,
        relation,
        name,
        InvariantKind::Check,
        &keys,
        project(filter(scan(relation, "subject"), predicate), &keys),
        doc,
    );
}
fn physical_checks(builder: &mut RegistryBuilder) {
    let zero = || RuleExpr::Lit(Cell::F64(0.0));
    check(
        builder,
        "reference.units",
        "check:positive_scale",
        RuleExpr::cmp(CmpOp::LtEq, RuleExpr::Col("scale_to_canonical"), zero()),
        "Unit representation scale is strictly positive.",
    );
    check(
        builder,
        "reference.quantity_types",
        "check:positive_nominal",
        RuleExpr::And(vec![
            RuleExpr::IsNotNull(Box::new(RuleExpr::Col("nominal_magnitude"))),
            RuleExpr::cmp(CmpOp::LtEq, RuleExpr::Col("nominal_magnitude"), zero()),
        ]),
        "A present nominal magnitude is strictly positive.",
    );
    check(
        builder,
        "authored.continuous_domains",
        "check:ordered_bounds",
        RuleExpr::cmp(CmpOp::GtEq, RuleExpr::Col("lower"), RuleExpr::Col("upper")),
        "Continuous-domain bounds are strictly increasing.",
    );
}
fn target_checks(builder: &mut RegistryBuilder) {
    for relation in [
        "authored.case_spec_targets",
        "authored.case_activation_targets",
        "authored.observation_targets",
    ] {
        let has = |name| RuleExpr::IsNotNull(Box::new(RuleExpr::Col(name)));
        let missing = |name| RuleExpr::IsNull(Box::new(RuleExpr::Col(name)));
        let kind = |name| {
            RuleExpr::cmp(
                CmpOp::Eq,
                RuleExpr::Col("member_kind"),
                RuleExpr::Lit(Cell::Enum(name)),
            )
        };
        let common_symbol = RuleExpr::And(vec![
            RuleExpr::Or(vec![kind("symbol"), kind("group")]),
            has("symbol_decl_id"),
            missing("equation_decl_id"),
            missing("port_template_id"),
            missing("port_name"),
            RuleExpr::Not(Box::new(RuleExpr::Col("wildcard"))),
        ]);
        let equation = RuleExpr::And(vec![
            kind("equation"),
            missing("symbol_decl_id"),
            has("equation_decl_id"),
            missing("port_template_id"),
            missing("port_name"),
            RuleExpr::Not(Box::new(RuleExpr::Col("wildcard"))),
        ]);
        let port = RuleExpr::And(vec![
            kind("port"),
            missing("symbol_decl_id"),
            missing("equation_decl_id"),
            has("port_template_id"),
            has("port_name"),
            RuleExpr::Not(Box::new(RuleExpr::Col("wildcard"))),
        ]);
        let wildcard = RuleExpr::And(vec![
            kind("instance_wildcard"),
            missing("symbol_decl_id"),
            missing("equation_decl_id"),
            missing("port_template_id"),
            missing("port_name"),
            missing("index"),
            RuleExpr::Col("wildcard"),
        ]);
        check(
            builder,
            relation,
            "check:target_shape",
            RuleExpr::Not(Box::new(RuleExpr::Or(vec![
                common_symbol,
                equation,
                port,
                wildcard,
            ]))),
            "Target kind, concrete declaration, port pair, wildcard and optional index agree.",
        );
        target_owner(
            builder,
            relation,
            "symbol_decl_id",
            "authored.template_symbols",
            "symbol_decl_id",
        );
        target_owner(
            builder,
            relation,
            "equation_decl_id",
            "authored.template_equations",
            "equation_decl_id",
        );
        target_port_owner(builder, relation);
    }
}
fn target_port_owner(builder: &mut RegistryBuilder, relation: &str) {
    let keys = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == relation)
        .map(|spec| spec.primary_key.clone())
        .unwrap_or_default();
    let subject = filter(
        scan(relation, "subject"),
        RuleExpr::IsNotNull(Box::new(RuleExpr::Col("port_template_id"))),
    );
    let owners = RulePlan::Project {
        input: Box::new(scan("authored.instances", "owners")),
        columns: vec![
            ("__owner_instance", RuleExpr::Col("instance_id")),
            ("__owner_template", RuleExpr::Col("template_id")),
        ],
    };
    let joined = RulePlan::EquiJoin {
        left: Box::new(subject),
        right: Box::new(owners),
        keys: vec![("instance_id", "__owner_instance")],
        null_equality: crate::model::NullEquality::NullEqualsNothing,
    };
    let ports = RulePlan::Project {
        input: Box::new(scan("authored.template_ports", "ports")),
        columns: vec![
            ("__port_template", RuleExpr::Col("template_id")),
            ("__port_name", RuleExpr::Col("name")),
        ],
    };
    let missing = RulePlan::AntiJoin {
        left: Box::new(joined),
        right: Box::new(ports),
        keys: vec![
            ("port_template_id", "__port_template"),
            ("__owner_template", "__port_template"),
            ("port_name", "__port_name"),
        ],
    };
    invariant(
        builder,
        relation,
        "closure:target_port_owner",
        InvariantKind::Closure,
        &keys,
        project(missing, &keys),
        "Port targets name an actual declared port of the target instance's template.",
    );
}
fn target_owner(
    builder: &mut RegistryBuilder,
    relation: &str,
    column: &'static str,
    declarations: &'static str,
    identity: &'static str,
) {
    let keys = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == relation)
        .map(|spec| spec.primary_key.clone())
        .unwrap_or_default();
    let owners = RulePlan::Project {
        input: Box::new(scan("authored.instances", "owners")),
        columns: vec![
            ("owner_instance", RuleExpr::Col("instance_id")),
            ("owner_template", RuleExpr::Col("template_id")),
        ],
    };
    let subject = filter(
        scan(relation, "subject"),
        RuleExpr::IsNotNull(Box::new(RuleExpr::Col(column))),
    );
    let joined = RulePlan::EquiJoin {
        left: Box::new(subject),
        right: Box::new(owners),
        keys: vec![("instance_id", "owner_instance")],
        null_equality: crate::model::NullEquality::NullEqualsNothing,
    };
    let declared = RulePlan::Project {
        input: Box::new(scan(declarations, "declarations")),
        columns: vec![
            ("decl_identity", RuleExpr::Col(identity)),
            ("decl_template", RuleExpr::Col("template_id")),
        ],
    };
    let wrong = RulePlan::AntiJoin {
        left: Box::new(joined),
        right: Box::new(declared),
        keys: vec![
            (column, "decl_identity"),
            ("owner_template", "decl_template"),
        ],
    };
    invariant(
        builder,
        relation,
        &format!("closure:target_owner:{column}"),
        InvariantKind::Closure,
        &keys,
        project(wrong, &keys),
        "Concrete target declarations belong to the actual target instance's template.",
    );
}

fn domain_reference_checks(builder: &mut RegistryBuilder) {
    let relations = builder.declared_relations().to_vec();
    for spec in relations {
        if spec.key.namespace != crate::model::Namespace::Normalized
            || !spec
                .columns
                .iter()
                .any(|column| column.name == "domain_name")
            || !spec
                .columns
                .iter()
                .any(|column| column.name == "template_id")
        {
            continue;
        }
        let actual = if spec
            .columns
            .iter()
            .any(|column| column.name == "wrt_domain_id")
        {
            "wrt_domain_id"
        } else if spec.columns.iter().any(|column| column.name == "domain_id") {
            "domain_id"
        } else {
            continue;
        };
        let has = |name| RuleExpr::IsNotNull(Box::new(RuleExpr::Col(name)));
        let missing = |name| RuleExpr::IsNull(Box::new(RuleExpr::Col(name)));
        let valid = RuleExpr::Or(vec![
            RuleExpr::And(vec![
                has(actual),
                missing("template_id"),
                missing("domain_name"),
            ]),
            RuleExpr::And(vec![
                missing(actual),
                has("template_id"),
                has("domain_name"),
                RuleExpr::cmp(
                    CmpOp::NotEq,
                    RuleExpr::Col("domain_name"),
                    RuleExpr::Lit(Cell::Text(String::new())),
                ),
            ]),
        ]);
        check(
            builder,
            &spec.key.qualified_name(),
            "check:domain_reference",
            RuleExpr::Not(Box::new(RuleExpr::IsTrue(Box::new(valid)))),
            "A normalized domain reference is exactly one actual ID or complete template/name pair.",
        );
    }
}
