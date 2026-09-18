// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Mechanical integrity declarations expressed directly in native SQL.
use super::RegistryBuilder;
use crate::{
    catalog::inv::{columns, declare as invariant, identifier, table},
    model::{ExtensionUse, FieldContract, InvariantKind, RelationDecl},
};
use arrow_schema::DataType;

pub(super) fn declare(builder: &mut RegistryBuilder) {
    for relation in builder.relations.clone() {
        let Some(keys) = relation.primary_key.as_deref() else {
            continue;
        };
        let name = relation.key.qualified_name();
        let source = table(&name);
        let group = columns(keys, "s");
        let query = if keys.is_empty() {
            format!("SELECT COUNT(*) AS violations FROM {source} s HAVING COUNT(*) > 1")
        } else {
            format!("SELECT {group} FROM {source} s GROUP BY {group} HAVING COUNT(*) > 1")
        };
        invariant(
            builder,
            &name,
            "unique:pk",
            InvariantKind::Unique,
            keys,
            query,
            &[&name],
            "The declared primary key identifies exactly one row.",
        );
        for field in &relation.columns {
            if let Some(fk) = field.fk() {
                let key = identifier(field.name());
                let target = table(fk.relation);
                let target_key = identifier(fk.column);
                let projection = projection(keys, "s");
                invariant(
                    builder,
                    &name,
                    &format!("foreign_key:{}", field.name()),
                    InvariantKind::ForeignKey,
                    keys,
                    format!(
                        "SELECT DISTINCT {projection} FROM {source} s WHERE s.{key} IS NOT NULL AND NOT EXISTS (SELECT 1 FROM {target} t WHERE s.{key} = t.{target_key})"
                    ),
                    &[&name, fk.relation],
                    "Every present foreign-key value resolves to its declared relation and column.",
                );
            }
            if has_ordinal(field) {
                let mut value = "__pse_ordinal_value".to_owned();
                while relation.columns.iter().any(|field| field.name() == value) {
                    value.push('_');
                }
                let prefix = if keys.is_empty() {
                    String::new()
                } else {
                    format!("{}, ", columns(keys, "s"))
                };
                let input = format!(
                    "SELECT {prefix}s.{} AS {} FROM {source} s",
                    identifier(field.name()),
                    identifier(&value)
                );
                ordinals(builder, &relation, field, field.name(), &input, &value);
            }
        }
    }
}
fn projection(keys: &[&str], alias: &str) -> String {
    if keys.is_empty() {
        "TRUE AS singleton".into()
    } else {
        columns(keys, alias)
    }
}
fn has_ordinal(field: &FieldContract) -> bool {
    matches!(field.extension(), Some(ExtensionUse::OrdinalRef { .. }))
        || field.children().iter().any(has_ordinal)
}
fn ordinals(
    builder: &mut RegistryBuilder,
    relation: &RelationDecl,
    field: &FieldContract,
    path: &str,
    input: &str,
    value: &str,
) {
    let name = relation.key.qualified_name();
    let keys = relation.primary_key.as_deref().unwrap_or_default();
    let prefix = if keys.is_empty() {
        String::new()
    } else {
        format!("{}, ", columns(keys, "s"))
    };
    let address = format!("s.{}", identifier(value));
    match (field.extension(), field.data_type()) {
        (Some(ExtensionUse::OrdinalRef { target }), _) => {
            invariant(
                builder,
                &name,
                &format!("ordinal_range:{path}"),
                InvariantKind::Domain,
                keys,
                format!(
                    "SELECT DISTINCT {} FROM ({input}) s WHERE {address} IS NOT NULL AND ({address} < 0 OR {address} >= (SELECT COUNT(*) FROM {}))",
                    projection(keys, "s"),
                    table(target)
                ),
                &[&name, target],
                "Every visible ordinal is within its explicitly declared target relation's row count.",
            );
        }
        (
            None,
            DataType::List(child) | DataType::LargeList(child) | DataType::FixedSizeList(child, _),
        ) => {
            let child = FieldContract::from_field(child.as_ref().clone());
            if has_ordinal(&child) {
                let expanded = format!(
                    "SELECT {prefix}unnest({address}) AS {} FROM ({input}) s",
                    identifier(value)
                );
                ordinals(
                    builder,
                    relation,
                    &child,
                    &format!("{path}[]"),
                    &expanded,
                    value,
                );
            }
        }
        (None, DataType::Struct(children)) => {
            for child in &children {
                let child = FieldContract::from_field(child.as_ref().clone());
                if has_ordinal(&child) {
                    let nested = format!(
                        "SELECT {prefix}get_field({address}, '{}') AS {} FROM ({input}) s WHERE {address} IS NOT NULL",
                        child.name().replace('\'', "''"),
                        identifier(value)
                    );
                    ordinals(
                        builder,
                        relation,
                        &child,
                        &format!("{path}.{}", child.name()),
                        &nested,
                        value,
                    );
                }
            }
        }
        _ => {}
    }
}
