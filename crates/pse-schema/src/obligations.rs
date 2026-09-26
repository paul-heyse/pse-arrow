// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One declaration-owned interpretation of keys and visible reference occurrences.
use crate::{
    SchemaError,
    model::{ExtensionUse, FieldContract, ReferenceContract, RelationDecl, TaggedAlternative},
};
use arrow_schema::DataType;
use std::collections::BTreeMap;

/// One visible occurrence, preserving collection correlation and ordered key mappings.
#[derive(Clone, Debug)]
pub struct ReferenceObligation {
    /// Structural path shared with native occurrence lowering; [] denotes one collection.
    pub path: Vec<String>,
    /// Reference meaning normalized once from the declaration.
    pub reference: ReferenceContract,
    /// SQL selection retaining root keys and one occurrence as __pse_value.
    pub input: String,
}
/// An ordinal occurrence bound to one explicitly selected relation's cardinality.
#[derive(Clone, Debug)]
pub struct OrdinalObligation {
    /// Structural occurrence path.
    pub path: Vec<String>,
    /// Qualified target relation.
    pub target: String,
    /// SQL occurrence selection.
    pub input: String,
}
/// Compiled declaration product shared by publication and invariant generation.
#[derive(Clone, Debug)]
pub struct RelationObligations {
    /// Private occurrence column chosen outside the authored field namespace.
    pub value_column: String,
    /// Ordered root key columns; empty means singleton.
    pub primary_key: Vec<&'static str>,
    /// Correlated scalar and nested references.
    pub references: Vec<ReferenceObligation>,
    /// Scalar and nested ordinal references.
    pub ordinals: Vec<OrdinalObligation>,
    /// Canonical native row checks, without presentation differences.
    pub checks: BTreeMap<String, String>,
}
impl RelationObligations {
    pub(crate) fn compile(relation: &RelationDecl) -> Result<Self, SchemaError> {
        let mut value_column = "__pse_value".to_owned();
        while relation
            .columns
            .iter()
            .any(|field| field.name() == value_column)
        {
            value_column.push('_');
        }
        let value = crate::catalog::inv::identifier(&value_column);
        let mut result = Self {
            value_column,
            primary_key: relation.primary_key.clone().unwrap_or_default(),
            references: vec![],
            ordinals: vec![],
            checks: relation
                .checks
                .iter()
                .map(|(name, sql)| {
                    Ok((name.clone(), crate::fingerprint::canonical_sql(sql, true)?))
                })
                .collect::<Result<_, SchemaError>>()?,
        };
        let source = crate::catalog::inv::table(&relation.key.qualified_name());
        for field in &relation.columns {
            let input = format!(
                "SELECT {}s.{} AS {value} FROM {source} s",
                result.prefix(),
                crate::catalog::inv::identifier(field.name())
            );
            result.descend(field, vec![field.name().into()], input)?;
        }
        Ok(result)
    }
    fn prefix(&self) -> String {
        if self.primary_key.is_empty() {
            String::new()
        } else {
            format!("{}, ", crate::catalog::inv::columns(&self.primary_key, "s"))
        }
    }
    fn descend(
        &mut self,
        field: &FieldContract,
        path: Vec<String>,
        input: String,
    ) -> Result<(), SchemaError> {
        let value = crate::catalog::inv::identifier(&self.value_column);
        let input = format!("SELECT * FROM ({input}) s WHERE s.{value} IS NOT NULL");
        if !matches!(field.data_type(), DataType::Dictionary(..))
            && let Some(reference) = ReferenceContract::for_contract(field)?
        {
            self.references.push(ReferenceObligation {
                path: path.clone(),
                reference,
                input: input.clone(),
            });
        }
        if let Some(ExtensionUse::OrdinalRef { target }) = field.extension() {
            self.ordinals.push(OrdinalObligation {
                path: path.clone(),
                target: target.into(),
                input: input.clone(),
            });
        }
        let prefix = self.prefix();
        match field.data_type() {
            DataType::Struct(children) => {
                let alternative = TaggedAlternative::from_field(field.field())?;
                for child in &children {
                    let mut guard = String::new();
                    if let Some(alternative) = &alternative
                        && alternative.payloads().contains(child.name().as_str())
                    {
                        let tags = alternative
                            .arms
                            .iter()
                            .filter(|(_, arm)| arm.as_deref() == Some(child.name()))
                            .map(|(tag, _)| crate::catalog::inv::literal(tag))
                            .collect::<Vec<_>>()
                            .join(", ");
                        guard = format!(
                            " WHERE get_field(s.{value}, {}) IN ({tags})",
                            crate::catalog::inv::literal(&alternative.discriminator)
                        );
                    }
                    let nested = format!(
                        "SELECT {prefix}get_field(s.{value}, {}) AS {value} FROM ({input}) s{guard}",
                        crate::catalog::inv::literal(child.name())
                    );
                    let mut child_path = path.clone();
                    child_path.push(child.name().clone());
                    self.descend(
                        &FieldContract::from_field(child.as_ref().clone()),
                        child_path,
                        nested,
                    )?;
                }
            }
            DataType::List(child)
            | DataType::LargeList(child)
            | DataType::FixedSizeList(child, _)
            | DataType::ListView(child)
            | DataType::LargeListView(child) => {
                let nested =
                    format!("SELECT {prefix}unnest(s.{value}) AS {value} FROM ({input}) s");
                let mut path = path;
                path.push("[]".into());
                self.descend(
                    &FieldContract::from_field(child.as_ref().clone()),
                    path,
                    nested,
                )?;
            }
            DataType::Map(entries, _) => {
                let DataType::Struct(fields) = entries.data_type() else {
                    return Err(crate::checks::invalid(
                        field.name(),
                        "map entries require a struct",
                    ));
                };
                let fresh = |suffix| {
                    let mut name = format!("{}{suffix}", self.value_column);
                    while self.primary_key.contains(&name.as_str()) {
                        name.push('_');
                    }
                    crate::catalog::inv::identifier(&name)
                };
                let key = fresh("_key");
                let item = fresh("_item");
                // Paired UNNEST preserves occurrence correlation. At this pin,
                // map_entries constructs canonical key/value fields and panics
                // when the admitted map retains different names or metadata.
                let expanded = format!(
                    "SELECT {prefix}unnest(map_keys(s.{value})) AS {key}, unnest(map_values(s.{value})) AS {item} FROM ({input}) s"
                );
                let names = fields
                    .iter()
                    .zip([key, item])
                    .map(|(field, native)| {
                        format!("{}, s.{native}", crate::catalog::inv::literal(field.name()))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let nested =
                    format!("SELECT {prefix}named_struct({names}) AS {value} FROM ({expanded}) s");
                let mut path = path;
                path.push("[]".into());
                self.descend(
                    &FieldContract::from_field(entries.as_ref().clone()),
                    path,
                    nested,
                )?;
            }
            DataType::Dictionary(_, kind) => {
                let nested = format!(
                    "SELECT {prefix}arrow_cast(s.{value}, {}) AS {value} FROM ({input}) s",
                    crate::catalog::inv::literal(&kind.to_string())
                );
                let decoded =
                    FieldContract::from_field(field.field().clone().with_data_type(*kind));
                self.descend(&decoded, path, nested)?;
            }
            DataType::RunEndEncoded(_, child) => {
                let nested = format!(
                    "SELECT {prefix}arrow_cast(s.{value}, {}) AS {value} FROM ({input}) s",
                    crate::catalog::inv::literal(&child.data_type().to_string())
                );
                self.descend(
                    &FieldContract::from_field(child.as_ref().clone()),
                    path,
                    nested,
                )?;
            }
            DataType::Union(children, _) => {
                for (_, child) in children.iter() {
                    let nested = format!(
                        "SELECT {prefix}union_extract(s.{value}, {}) AS {value} FROM ({input}) s",
                        crate::catalog::inv::literal(child.name())
                    );
                    let mut path = path.clone();
                    path.push(child.name().clone());
                    self.descend(
                        &FieldContract::from_field(child.as_ref().clone()),
                        path,
                        nested,
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
