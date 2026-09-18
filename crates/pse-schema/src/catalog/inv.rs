// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native invariant declarations bound by the actual DataFusion session.
use crate::{
    RegistryBuilder,
    model::{InvariantDecl, InvariantKind},
};

pub(crate) fn identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
pub(crate) fn table(name: &str) -> String {
    name.split('.')
        .map(identifier)
        .collect::<Vec<_>>()
        .join(".")
}
pub(crate) fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
pub(crate) fn columns(keys: &[&str], alias: &str) -> String {
    keys.iter()
        .map(|key| format!("{alias}.{}", identifier(key)))
        .collect::<Vec<_>>()
        .join(", ")
}

#[expect(
    clippy::too_many_arguments,
    reason = "declaration fields are independent invariant contract dimensions"
)]
pub(crate) fn declare(
    builder: &mut RegistryBuilder,
    relation: &str,
    name: &str,
    kind: InvariantKind,
    keys: &[&'static str],
    query: impl Into<String>,
    inputs: &[&str],
    doc: &'static str,
) {
    let mut inputs = inputs
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    inputs.sort();
    inputs.dedup();
    let declaration = InvariantDecl::error(relation, name, kind, query, inputs, keys.to_vec(), doc);
    if !builder.declared_invariants().contains(&declaration) {
        builder.declare_invariant(declaration);
    }
}
