// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Invariant-query projection of the declaration-owned obligation product.
use super::RegistryBuilder;
use crate::{
    catalog::inv::{columns, declare as invariant, identifier, literal, table},
    model::InvariantKind,
};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    for relation in builder.relations.clone() {
        if relation.primary_key.is_none() {
            continue;
        }
        let Some(Ok(product)) = builder.obligations.get(&relation.key).cloned() else {
            continue;
        };
        let keys = &product.primary_key;
        let name = relation.key.qualified_name();
        let source = table(&name);
        let value = format!("s.{}", identifier(&product.value_column));
        let group = columns(keys, "s");
        let projection = if keys.is_empty() {
            "TRUE AS singleton".into()
        } else {
            group.clone()
        };
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
        for key in &relation.unique_keys {
            let group = columns(&key.columns, "u");
            let present = key
                .columns
                .iter()
                .map(|column| format!("u.{} IS NOT NULL", identifier(column)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let joined = key
                .columns
                .iter()
                .map(|column| format!("s.{0} = d.{0}", identifier(column)))
                .collect::<Vec<_>>()
                .join(" AND ");
            invariant(
                builder,
                &name,
                &format!("unique:{}", key.name),
                InvariantKind::Unique,
                keys,
                format!(
                    "SELECT DISTINCT {projection} FROM {source} s JOIN (SELECT {group} FROM {source} u WHERE {present} GROUP BY {group} HAVING COUNT(*) > 1) d ON {joined}"
                ),
                &[&name],
                "A declared unique key identifies at most one row among the rows whose key is present.",
            );
        }
        for reference in &relation.foreign_keys {
            let present = reference
                .columns
                .iter()
                .map(|column| format!("s.{} IS NOT NULL", identifier(column)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let equal = reference
                .columns
                .iter()
                .zip(&reference.target_columns)
                .map(|(local, remote)| {
                    format!("s.{} = t.{}", identifier(local), identifier(remote))
                })
                .collect::<Vec<_>>()
                .join(" AND ");
            let target = table(reference.target);
            invariant(
                builder,
                &name,
                &format!("foreign_key:{}", reference.name),
                InvariantKind::ForeignKey,
                keys,
                format!(
                    "SELECT DISTINCT {projection} FROM {source} s WHERE {present} AND NOT EXISTS (SELECT 1 FROM {target} t WHERE {equal})"
                ),
                &[&name, reference.target],
                "Every reference whose columns are all present resolves to its declared target key.",
            );
        }
        for occurrence in &product.references {
            let reference = &occurrence.reference;
            let values = reference
                .columns
                .iter()
                .map(|mapping| {
                    mapping.source.iter().fold(value.clone(), |value, field| {
                        format!("get_field({value}, {})", literal(field))
                    })
                })
                .collect::<Vec<_>>();
            let present = values
                .iter()
                .map(|value| format!("{value} IS NOT NULL"))
                .collect::<Vec<_>>()
                .join(" AND ");
            let equal = values
                .iter()
                .zip(&reference.columns)
                .map(|(value, mapping)| format!("{value} = t.{}", identifier(&mapping.target)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let target = table(&reference.relation);
            let path = display_path(&occurrence.path);
            invariant(
                builder,
                &name,
                &format!("foreign_key:{path}"),
                InvariantKind::ForeignKey,
                keys,
                format!(
                    "SELECT DISTINCT {projection} FROM ({}) s WHERE {present} AND NOT EXISTS (SELECT 1 FROM {target} t WHERE {equal})",
                    occurrence.input
                ),
                &[&name, &reference.relation],
                "Every present foreign-key value resolves to its declared relation and column.",
            );
        }
        for occurrence in &product.ordinals {
            invariant(
                builder,
                &name,
                &format!("ordinal_range:{}", display_path(&occurrence.path)),
                InvariantKind::Domain,
                keys,
                format!(
                    "SELECT DISTINCT {projection} FROM ({}) s WHERE {value} < 0 OR {value} >= (SELECT COUNT(*) FROM {})",
                    occurrence.input,
                    table(&occurrence.target)
                ),
                &[&name, &occurrence.target],
                "Every visible ordinal is within its explicitly declared target relation's row count.",
            );
        }
    }
}
fn display_path(path: &[String]) -> String {
    path.join(".").replace(".[]", "[]")
}
