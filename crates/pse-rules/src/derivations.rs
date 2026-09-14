// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reversible diagnostic key references, validated against exact rule head contracts.
use crate::{RuleError, errmap::internal};
use pse_ids::{SemanticId, named_id};
use pse_schema::{
    Registry,
    model::{Cell, ColumnSpec, RelationSpec, RuleHead, RuleSpec},
};

const VERSION: &str = "pse.rule-key.v1";
fn columns<'a>(rule: &RuleSpec, registry: &'a Registry) -> Result<Vec<&'a ColumnSpec>, RuleError> {
    let relation = registry
        .relation(rule.head.relation())
        .ok_or_else(|| internal("key head relation is undeclared"))?;
    let names = match &rule.head {
        RuleHead::Relation(_) => &relation.primary_key,
        RuleHead::Violations { key_columns, .. } => key_columns,
    };
    names
        .iter()
        .map(|name| {
            relation
                .column(name)
                .ok_or_else(|| internal("rule key column undeclared"))
        })
        .collect()
}
fn validate(
    columns: &[&ColumnSpec],
    values: &[Cell],
    registry: &Registry,
) -> Result<(), RuleError> {
    if columns.len() != values.len() {
        return Err(internal(
            "diagnostic key width differs from exact rule head",
        ));
    }
    for (column, value) in columns.iter().zip(values) {
        let field = pse_schema::arrow::field_for(registry, column)
            .map_err(|error| internal(error.to_string()))?;
        pse_relations::cells::array_from_cells(registry, &field, std::slice::from_ref(value))
            .map_err(|error| internal(error.to_string()))?;
    }
    Ok(())
}
/// Encode every ordered key name and typed value after exact field admission.
///
/// # Errors
/// The supplied values do not satisfy the exact declared rule key.
pub fn encode_key(
    rule: &RuleSpec,
    registry: &Registry,
    values: &[Cell],
) -> Result<String, RuleError> {
    let columns = columns(rule, registry)?;
    encode_columns(&columns, registry, values)
}
fn encode_columns(
    columns: &[&ColumnSpec],
    registry: &Registry,
    values: &[Cell],
) -> Result<String, RuleError> {
    validate(columns, values, registry)?;
    let pairs = columns
        .iter()
        .zip(values)
        .map(|(column, value)| {
            let name =
                serde_json::to_string(column.name).map_err(|error| internal(error.to_string()))?;
            Ok(format!("[{name},{}]", value.literal_spec()))
        })
        .collect::<Result<Vec<_>, RuleError>>()?;
    Ok(format!("[\"{VERSION}\",[{}]]", pairs.join(",")))
}
/// Encode an actual relation primary key with the same checked, reversible contract.
///
/// # Errors
/// Missing key columns or a value that does not satisfy the declared column.
pub fn encode_relation_key(
    spec: &RelationSpec,
    registry: &Registry,
    row: &[Cell],
) -> Result<String, RuleError> {
    if row.len() != spec.columns.len() {
        return Err(internal("supporting row width differs from relation"));
    }
    let mut columns = vec![];
    let mut values = vec![];
    for name in &spec.primary_key {
        let index = spec
            .columns
            .iter()
            .position(|column| column.name == *name)
            .ok_or_else(|| internal("support key undeclared"))?;
        columns.push(&spec.columns[index]);
        values.push(row[index].clone());
    }
    encode_columns(&columns, registry, &values)
}

/// Decode diagnostic keys and compare names, order and actual field values with the rule.
///
/// # Errors
/// Unknown version, malformed/truncated key, mismatched names/types, or invalid values.
pub fn decode_key(
    rule: &RuleSpec,
    registry: &Registry,
    text: &str,
) -> Result<Vec<Cell>, RuleError> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|error| internal(error.to_string()))?;
    let root = value
        .as_array()
        .filter(|root| root.len() == 2)
        .ok_or_else(|| internal("key requires version and ordered members"))?;
    if root[0].as_str() != Some(VERSION) {
        return Err(internal("unknown rule-key codec version"));
    }
    let members = root[1]
        .as_array()
        .ok_or_else(|| internal("key members are not an ordered list"))?;
    let columns = columns(rule, registry)?;
    if members.len() != columns.len() {
        return Err(internal("diagnostic key member count differs"));
    }
    let mut values = vec![];
    for (member, column) in members.iter().zip(&columns) {
        let member = member
            .as_array()
            .filter(|member| member.len() == 2)
            .ok_or_else(|| internal("key member requires exact name and value"))?;
        if member[0].as_str() != Some(column.name) {
            return Err(internal("diagnostic key name/order differs from rule head"));
        }
        values.push(
            Cell::from_literal_spec(&member[1].to_string(), registry)
                .map_err(|error| internal(error.to_string()))?,
        );
    }
    validate(&columns, &values, registry)?;
    Ok(values)
}
/// Stable identity over an already validated complete diagnostic key.
pub fn finding_id(invariant: SemanticId, key: &str) -> SemanticId {
    named_id(invariant, key)
}
