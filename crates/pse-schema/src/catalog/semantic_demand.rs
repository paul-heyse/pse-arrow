// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite demand closure and complete method selection (blueprint §6.15.3).

mod candidates;
mod checks;
mod closure;
mod declaration_checks;
mod declarations;
mod framing;
mod read_failures;
mod states;

use super::declarations::{column, relation};
use crate::{
    RegistryBuilder,
    model::{
        Authority, Cell, CmpOp, DerivationGranularity, FieldContract, FieldContract as T,
        Namespace as N, NullEquality, RelationDecl, RuleDecl, RuleExpr as E, RuleHead,
        RulePlan as P, SnapshotClass as S,
    },
};

/// Declare the complete potential-key substrate, stratified selection and positive demand closure.
pub fn declare(builder: &mut RegistryBuilder) {
    declarations::declare(builder);
    framing::declare(builder);
    states::declare(builder);
    candidates::declare(builder);
    closure::declare(builder);
    checks::declare(builder);
    read_failures::declare(builder);
    declaration_checks::declare(builder);
}
fn provenance() -> FieldContract {
    FieldContract::provenance(
        "derivation_id",
        T::id(),
        "Actual rule or structural source derivation.",
    )
}
fn index() -> T {
    T::extended(crate::model::ExtensionUse::IndexTuple)
}
fn scan(relation: &str, port: &'static str) -> P {
    P::Scan {
        relation: relation.to_owned(),
        port,
    }
}
fn project(input: P, columns: Vec<(&str, E)>) -> P {
    P::Project {
        input: Box::new(input),
        columns: (columns)
            .into_iter()
            .map(|(name, expression)| (name.to_owned().into(), expression))
            .collect(),
    }
}
fn filter(input: P, predicate: E) -> P {
    P::Filter {
        input: Box::new(input),
        predicate,
    }
}
fn join(left: P, right: P, keys: Vec<(&'static str, &'static str)>) -> P {
    P::EquiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: (keys)
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
        null_equality: NullEquality::NullEqualsNothing,
    }
}
fn anti(left: P, right: P, keys: Vec<(&'static str, &'static str)>) -> P {
    P::AntiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: (keys)
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    }
}
fn eq(left: E, right: E) -> E {
    E::cmp(CmpOp::Eq, left, right)
}
fn literal(value: &'static str) -> E {
    E::Lit(Cell::Enum(value))
}
fn null(name: &'static str) -> E {
    E::IsNull(Box::new(E::col(name)))
}
fn present(name: &'static str) -> E {
    E::IsNotNull(Box::new(E::col(name)))
}
fn same(left: &'static str, right: &'static str) -> E {
    E::IsNotDistinctFrom(Box::new(E::col(left)), Box::new(E::col(right)))
}
fn rule(
    builder: &mut RegistryBuilder,
    name: &'static str,
    stratum: u16,
    head: &'static str,
    assertion: &'static str,
    plan: P,
    negative: bool,
) {
    let spec = RuleDecl::new(
        name,
        "1",
        50 + stratum,
        RuleHead::Relation(head.to_owned()),
        plan,
    )
    .assertions(assertion);
    builder.declare_rule(if negative {
        spec.stratified_negation()
    } else {
        spec
    });
}
fn assertion(builder: &mut RegistryBuilder, head: &str, name: &'static str) {
    if let Some(spec) = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == head)
    {
        let columns = crate::model::rule::assertion_columns(&spec.columns);
        builder.declare_relation(
            RelationDecl::new(
                N::Provenance,
                name,
                1,
                Authority::Derived,
                S::Derived,
                "Mechanically projected P6 rule assertions.",
            )
            .pk(&["assertion_id"])
            .columns(columns)
            .granularity(DerivationGranularity::Rule),
        );
    }
}
