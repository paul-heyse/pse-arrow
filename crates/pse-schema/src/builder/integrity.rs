// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One mechanical projection of declared integrity into ordinary rule programs.

use super::RegistryBuilder;
use crate::model::{
    AggregateEmptyPolicy, AggregateNullPolicy, Cell, CmpOp, EmptyListPolicy, ExtensionUse,
    FieldContract, InvariantDecl, InvariantKind, NullEquality, NullListPolicy, RelationDecl,
    RuleAggregate, RuleAggregateFn, RuleDecl, RuleExpr, RuleHead, RulePlan,
};
use arrow_schema::DataType;
use std::{borrow::Cow, collections::BTreeSet};

type Name = Cow<'static, str>;

pub(super) fn declare(builder: &mut RegistryBuilder) {
    for relation in builder.relations.clone() {
        if relation.primary_key.is_none() {
            // The relation declaration validator reports the missing identity.
            continue;
        }
        let mut used = relation
            .columns
            .iter()
            .map(|column| column.name().to_owned())
            .collect();
        let count_name = fresh(&mut used, "__pse_count");
        let names = Names {
            value: fresh(&mut used, "__pse_integrity_value"),
            item: fresh(&mut used, "__pse_integrity_item"),
            count: fresh(&mut used, "__pse_integrity_count"),
            left: fresh(&mut used, "__pse_integrity_left"),
            right: fresh(&mut used, "__pse_integrity_right"),
        };
        let name = relation.key.qualified_name();
        let repeated = filter(
            RulePlan::Aggregate {
                input: Box::new(scan(&name, "subject")),
                group: keys(&relation).iter().map(|name| (*name).into()).collect(),
                aggregates: vec![count(count_name.clone())],
            },
            RuleExpr::cmp(
                CmpOp::Gt,
                RuleExpr::col(count_name),
                RuleExpr::Lit(Cell::U64(1)),
            ),
        );
        install(
            builder,
            &relation,
            "unique:pk",
            InvariantKind::Unique,
            project(repeated, keys(&relation), None),
            "The declared primary key identifies exactly one row.",
        );
        for column in &relation.columns {
            if let Some(fk) = column.fk() {
                let missing = RulePlan::AntiJoin {
                    left: Box::new(filter(
                        scan(&name, "subject"),
                        present(column.name().to_owned().into()),
                    )),
                    right: Box::new(scan(fk.relation, "referenced")),
                    keys: vec![(column.name().to_owned().into(), fk.column.to_owned().into())],
                };
                install(
                    builder,
                    &relation,
                    &format!("foreign_key:{}", column.name()),
                    InvariantKind::ForeignKey,
                    RulePlan::Distinct(Box::new(project(missing, keys(&relation), None))),
                    "Every present foreign-key value resolves to its declared relation and column.",
                );
            }
            if has_ordinal(&column.value_type()) {
                let input = project(
                    scan(&name, "subject"),
                    keys(&relation),
                    Some((names.value.clone(), RuleExpr::col(column.name().to_owned()))),
                );
                ordinals(
                    builder,
                    &relation,
                    &column.value_type(),
                    column.name(),
                    input,
                    &names,
                );
            }
        }
    }
}

// Aliases belong to this finite plan construction, never to the declared row model.
struct Names {
    value: Name,
    item: Name,
    count: Name,
    left: Name,
    right: Name,
}
fn fresh(used: &mut BTreeSet<String>, stem: &str) -> Name {
    let mut name = stem.to_owned();
    while used.contains(&name) {
        name.push('_');
    }
    used.insert(name.clone());
    name.into()
}

fn has_ordinal(ty: &FieldContract) -> bool {
    matches!(ty.extension(), Some(ExtensionUse::OrdinalRef { .. }))
        || ty.children().iter().any(has_ordinal)
}

fn ordinals(
    builder: &mut RegistryBuilder,
    relation: &RelationDecl,
    ty: &FieldContract,
    path: &str,
    input: RulePlan,
    names: &Names,
) {
    match (ty.extension(), ty.data_type()) {
        (Some(ExtensionUse::OrdinalRef { target }), _) => {
            ordinal_range(builder, relation, target, path, input, names);
        }
        (None, DataType::List(item) | DataType::FixedSizeList(item, _))
            if has_ordinal(&FieldContract::from_field((*item).clone())) =>
        {
            let expanded = RulePlan::Unnest {
                input: Box::new(input),
                column: names.value.clone(),
                value_name: names.item.clone(),
                null_list: NullListPolicy::NoMembers,
                empty_list: EmptyListPolicy::NoMembers,
            };
            let projected = project(
                expanded,
                keys(relation),
                Some((names.value.clone(), RuleExpr::col(names.item.clone()))),
            );
            ordinals(
                builder,
                relation,
                &FieldContract::from_field((*item).clone()),
                &format!("{path}[]"),
                projected,
                names,
            );
        }
        (None, DataType::Struct(fields)) => {
            for field in &fields {
                let name = field.name();
                let child = &FieldContract::from_field((**field).clone());
                if has_ordinal(child) {
                    let projected = project(
                        input.clone(),
                        keys(relation),
                        Some((
                            names.value.clone(),
                            RuleExpr::Field {
                                expr: Box::new(RuleExpr::col(names.value.clone())),
                                name: name.to_owned().into(),
                            },
                        )),
                    );
                    ordinals(
                        builder,
                        relation,
                        child,
                        &format!("{path}.{name}"),
                        projected,
                        names,
                    );
                }
            }
        }
        _ => {}
    }
}

fn ordinal_range(
    builder: &mut RegistryBuilder,
    relation: &RelationDecl,
    target: &str,
    path: &str,
    input: RulePlan,
    names: &Names,
) {
    let visible = filter(input, present(names.value.clone()));
    let mut left_columns = keys(relation)
        .iter()
        .map(|name| ((*name).into(), RuleExpr::col(*name)))
        .collect::<Vec<_>>();
    left_columns.extend([
        (names.value.clone(), RuleExpr::col(names.value.clone())),
        (names.left.clone(), RuleExpr::Lit(Cell::Bool(true))),
    ]);
    let cardinality = RulePlan::Aggregate {
        input: Box::new(scan(target, "ordinal_target")),
        group: vec![],
        aggregates: vec![count(names.count.clone())],
    };
    let joined = RulePlan::EquiJoin {
        left: Box::new(RulePlan::Project {
            input: Box::new(visible),
            columns: left_columns,
        }),
        right: Box::new(RulePlan::Project {
            input: Box::new(cardinality),
            columns: vec![
                (names.count.clone(), RuleExpr::col(names.count.clone())),
                (names.right.clone(), RuleExpr::Lit(Cell::Bool(true))),
            ],
        }),
        keys: vec![(names.left.clone(), names.right.clone())],
        null_equality: NullEquality::NullEqualsNothing,
    };
    let invalid = filter(
        joined,
        RuleExpr::cmp(
            CmpOp::GtEq,
            RuleExpr::col(names.value.clone()),
            RuleExpr::col(names.count.clone()),
        ),
    );
    install(
        builder,
        relation,
        &format!("ordinal_range:{path}"),
        InvariantKind::Domain,
        RulePlan::Distinct(Box::new(project(invalid, keys(relation), None))),
        "Every visible ordinal is below its explicitly declared target relation's row count.",
    );
}

fn install(
    builder: &mut RegistryBuilder,
    relation: &RelationDecl,
    name: &str,
    kind: InvariantKind,
    plan: RulePlan,
    doc: &'static str,
) {
    let relation_name = relation.key.qualified_name();
    let rule_name = format!("{name}:{relation_name}");
    let rule = RuleDecl::new(
        rule_name.clone(),
        "1",
        1,
        RuleHead::Violations {
            of: relation_name.clone(),
            key_columns: keys(relation).to_vec(),
        },
        plan,
    )
    .stratified_negation();
    let invariant = InvariantDecl::error(relation_name, name, kind, format!("{rule_name}@1"), doc);
    // Repeated projection during catalog/pass assembly is exact structural agreement.
    // A same-name different declaration is appended and rejected by normal admission.
    if !builder.rules.contains(&rule) {
        builder.rules.push(rule);
    }
    if !builder.invariants.contains(&invariant) {
        builder.invariants.push(invariant);
    }
}
fn scan(relation: impl Into<String>, port: &'static str) -> RulePlan {
    RulePlan::Scan {
        relation: relation.into(),
        port,
    }
}
fn present(name: Name) -> RuleExpr {
    RuleExpr::IsNotNull(Box::new(RuleExpr::col(name)))
}
fn filter(input: RulePlan, predicate: RuleExpr) -> RulePlan {
    RulePlan::Filter {
        input: Box::new(input),
        predicate,
    }
}
fn project(input: RulePlan, keys: &[&'static str], value: Option<(Name, RuleExpr)>) -> RulePlan {
    let mut columns = keys
        .iter()
        .map(|key| ((*key).into(), RuleExpr::col(*key)))
        .collect::<Vec<_>>();
    columns.extend(value);
    RulePlan::Project {
        input: Box::new(input),
        columns,
    }
}
fn count(output_name: Name) -> RuleAggregate {
    RuleAggregate {
        function: RuleAggregateFn::Count,
        input: None,
        output_name,
        order_by: vec![],
        null_policy: AggregateNullPolicy::Reject,
        empty_policy: AggregateEmptyPolicy::Zero,
    }
}

fn keys(relation: &RelationDecl) -> &[&'static str] {
    relation.primary_key.as_deref().unwrap_or_default()
}
