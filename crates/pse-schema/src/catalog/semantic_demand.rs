// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite demand closure and complete method selection (blueprint §6.15.3).
mod checks;
mod declaration_checks;
mod declarations;
mod framing;
mod read_failures;
use super::declarations::{column, relation};
use crate::{
    RegistryBuilder,
    model::{
        Authority, DerivationGranularity, FieldContract, FieldContract as T, Namespace as N,
        RelationDecl, SnapshotClass as S,
    },
};
/// Declare the complete potential-key substrate, stratified selection and positive demand closure.
pub fn declare(builder: &mut RegistryBuilder) {
    declarations::declare(builder);
    framing::declare(builder);
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
