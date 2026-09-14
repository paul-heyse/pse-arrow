// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Small constructors for explicit catalog declarations; no schema inference.

use crate::builder::RegistryBuilder;
use crate::model::{
    Authority, ColumnRole, ColumnSpec, DerivationGranularity, EnumDecl, EnumMember, LogicalType,
    Namespace, RelationDecl, SnapshotClass,
};

pub(super) fn relation(
    builder: &mut RegistryBuilder,
    namespace: Namespace,
    name: &'static str,
    class: SnapshotClass,
    keys: &[&'static str],
    mut columns: Vec<ColumnSpec>,
    doc: &'static str,
) {
    let authority = match namespace {
        Namespace::Authored => Authority::Authored,
        Namespace::Reference => Authority::Reference,
        _ => Authority::Derived,
    };
    for column in &mut columns {
        if keys.contains(&column.name) {
            column.role = ColumnRole::Key;
        }
    }
    let mut declaration = RelationDecl::new(namespace, name, 1, authority, class, doc)
        .pk(keys)
        .columns(columns);
    if authority == Authority::Derived {
        declaration = declaration.granularity(DerivationGranularity::Row);
    }
    builder.declare_relation(declaration);
}

pub(super) fn column(name: &'static str, logical_type: LogicalType) -> ColumnSpec {
    ColumnSpec::payload(name, logical_type, name)
}

pub(super) fn enumeration(
    builder: &mut RegistryBuilder,
    name: &'static str,
    members: impl IntoIterator<Item = &'static str>,
) {
    builder.declare_enum(EnumDecl::platform(
        name,
        members
            .into_iter()
            .map(|member| EnumMember::new(member, member))
            .collect(),
    ));
}

pub(super) fn structure(fields: Vec<(&'static str, LogicalType)>) -> LogicalType {
    LogicalType::Struct(
        fields
            .into_iter()
            .map(|(name, kind)| (name, kind, false))
            .collect(),
    )
}
