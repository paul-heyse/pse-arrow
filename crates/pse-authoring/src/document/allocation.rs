// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Preallocation extents for the owned loader; no extent establishes model validity.

use crate::AuthoringError;
use pse_ids::{CancellationToken, MemoryReserver, Reservation, ReservationLease};
use std::sync::Arc;

pub(super) struct Allocation<'a> {
    pub(super) cancel: &'a CancellationToken,
    reservation: Box<dyn Reservation>,
}
impl<'a> Allocation<'a> {
    pub(super) fn new(reserver: &'a dyn MemoryReserver, cancel: &'a CancellationToken) -> Self {
        Self {
            cancel,
            reservation: reserver.open("authoring:owned-documents"),
        }
    }
    pub(super) fn grow(&mut self, bytes: usize) -> Result<(), AuthoringError> {
        self.cancel.checkpoint()?;
        self.reservation.try_grow(bytes)?;
        self.cancel.checkpoint()?;
        Ok(())
    }
    pub(super) fn size(&self) -> usize {
        self.reservation.size()
    }
    pub(super) fn retain(&mut self, start: usize, bytes: usize) -> Result<(), AuthoringError> {
        let target = add(start, bytes)?;
        if target > self.size() {
            return Err(super::load::contract(
                None,
                "owned loader preallocation extent was insufficient",
            ));
        }
        self.reservation.shrink(self.size() - target);
        Ok(())
    }
    pub(super) fn finish(self) -> Arc<ReservationLease> {
        ReservationLease::new(self.reservation)
    }
}

pub(super) fn add(a: usize, b: usize) -> Result<usize, AuthoringError> {
    a.checked_add(b).ok_or(AuthoringError::Budget {
        limit: "allocation extent",
        allowed: usize::MAX as u64,
        needed: u64::MAX,
    })
}
pub(super) fn mul(a: usize, b: usize) -> Result<usize, AuthoringError> {
    a.checked_mul(b).ok_or(AuthoringError::Budget {
        limit: "allocation extent",
        allowed: usize::MAX as u64,
        needed: u64::MAX,
    })
}

/// One full pinned Rust B-tree node per entry, including edges and node header.
/// This intentionally bounds underfilled nodes, not only amortized payload size.
pub(super) const fn map_entry<K, V>() -> usize {
    11 * (size_of::<K>() + size_of::<V>()) + 16 * size_of::<usize>()
}

use super::{Document, value::Value};
use crate::{ParseBudget, SourceSpan, dsl};
use pse_schema::{
    Registry,
    model::{DocumentSection, ExtensionUse, FieldContract},
};
use serde_saphyr::Spanned;

/// Lexical upper bound, including punctuation-created empty/implicit nodes. Each
/// scalar needs a non-whitespace run or punctuation; every collection, implicit
/// null, TOML dotted table and array-table level has a punctuation witness. We
/// count *all* ASCII punctuation (even inside strings/comments), so this is an
/// overestimate, never a tokenizer that could discard a YAML/TOML feature.
fn lexical_nodes(text: &str) -> Result<usize, AuthoringError> {
    let mut tokens = 0;
    let mut in_word = false;
    for byte in text.bytes() {
        if byte.is_ascii_whitespace() {
            in_word = false;
        } else if byte.is_ascii_punctuation() {
            tokens = add(tokens, 1)?;
            in_word = false;
        } else {
            if !in_word {
                tokens = add(tokens, 1)?;
            }
            in_word = true;
        }
    }
    add(mul(tokens, 2)?, 1)
}

pub(super) fn yaml_node_limit(text: &str, budget: &ParseBudget) -> Result<usize, AuthoringError> {
    let limit = usize::try_from(budget.max_bytes)
        .unwrap_or(usize::MAX)
        .min(250_000);
    if text.as_bytes().contains(&b'*') {
        Ok(limit)
    } else {
        Ok(limit.min(lexical_nodes(text)?))
    }
}

pub(super) fn parser_extent(
    text: &str,
    budget: &ParseBudget,
    yaml: bool,
) -> Result<usize, AuthoringError> {
    let limit = usize::try_from(budget.max_bytes).unwrap_or(usize::MAX);
    // An asterisk can introduce alias replay. Do not estimate expanded nodes from
    // source length in that case: use the parser's enforced expanded-node budget.
    let aliases = yaml && text.as_bytes().contains(&b'*');
    let nodes = if yaml {
        yaml_node_limit(text, budget)?
    } else {
        lexical_nodes(text)?
    };
    let scalars = if aliases { limit } else { text.len() };
    // Value Vec growth, map duplicate-key inventory, TOML intermediate tree,
    // Spanned wrappers and visitor transients can coexist. Each term names its
    // allocation type; source bytes alone are not the allocation authority.
    let node = add(
        mul(4, size_of::<(Spanned<String>, Spanned<Value>)>())?,
        add(
            map_entry::<String, ()>(),
            if yaml {
                0
            } else {
                map_entry::<String, toml::Value>()
            },
        )?,
    )?;
    add(16_384, add(mul(nodes, node)?, mul(scalars, 8)?)?)
}

/// Existing allocations only; used solely to shrink an already proven preflight.
/// Every String/Vec capacity is retained, including unused capacity.
pub(super) fn value_retained(value: &Value) -> Result<usize, AuthoringError> {
    let children = match value {
        Value::Text(text) => text.capacity(),
        Value::List(values) => {
            let mut bytes = mul(values.capacity(), size_of::<Spanned<Value>>())?;
            for value in values {
                bytes = add(bytes, value_retained(&value.value)?)?;
            }
            bytes
        }
        Value::Map(values) => {
            let mut bytes = mul(
                values.capacity(),
                size_of::<(Spanned<String>, Spanned<Value>)>(),
            )?;
            for (key, value) in values {
                bytes = add(
                    bytes,
                    add(key.value.capacity(), value_retained(&value.value)?)?,
                )?;
            }
            bytes
        }
        _ => 0,
    };
    add(size_of::<Value>(), children)
}

/// Exact path byte lengths are computed without creating any paths. A separate
/// node-sized claim covers B-tree growth, while four path copies cover escaping,
/// recursive temporary paths and the final /@key entry simultaneously.
pub(super) fn span_extent(value: &Value, parent: usize) -> Result<usize, AuthoringError> {
    let mut bytes = add(map_entry::<String, SourceSpan>(), mul(parent, 4)?)?;
    match value {
        Value::List(values) => {
            for (ordinal, value) in values.iter().enumerate() {
                let digits = if ordinal == 0 {
                    1
                } else {
                    ordinal.ilog10() as usize + 1
                };
                bytes = add(
                    bytes,
                    span_extent(&value.value, add(parent, add(digits, 1)?)?)?,
                )?;
            }
        }
        Value::Map(values) => {
            for (key, value) in values {
                let escaped = key.value.bytes().try_fold(0_usize, |sum, byte| {
                    add(sum, if matches!(byte, b'~' | b'/') { 2 } else { 1 })
                })?;
                let path = add(parent, add(1, escaped)?)?;
                bytes = add(
                    bytes,
                    add(map_entry::<String, SourceSpan>(), mul(add(path, 5)?, 4)?)?,
                )?;
                bytes = add(bytes, span_extent(&value.value, path)?)?;
            }
        }
        _ => {}
    }
    Ok(bytes)
}

/// Visit actual document rows without a temporary row-reference vector.
pub(super) fn visit_rows(
    document: &Document,
    mut visit: impl FnMut(&DocumentSection, &Value) -> Result<(), AuthoringError>,
) -> Result<(), AuthoringError> {
    for section in &document.declaration.sections {
        if let Some(value) = document.value.get(section.key) {
            if section.repeated {
                if let Value::List(values) = value {
                    for value in values {
                        visit(section, &value.value)?;
                    }
                }
            } else {
                visit(section, value)?;
            }
        }
    }
    Ok(())
}

pub(super) fn hydration_extent(
    documents: &[Document],
    package_name: &str,
    registry: &Registry,
) -> Result<usize, AuthoringError> {
    let mut extent = 0;
    let mut entities = 0;
    let mut names = package_name.len();
    for document in documents {
        extent = add(extent, mul(value_retained(&document.value)?, 4)?)?;
        visit_rows(document, |section, value| {
            let spec = registry
                .relation(section.relation)
                .ok_or_else(|| super::load::contract(None, "undeclared document relation"))?;
            // Pending row, visited/identity/name indices and generated
            // entity output all coexist. Context can inject package IDs and a
            // three-member source span for each declared column, at most.
            extent = add(extent, super::hydrate::pending_extent())?;
            extent = add(extent, map_entry::<usize, ()>())?;
            for column in &spec.columns {
                extent = add(
                    extent,
                    add(
                        mul(16, size_of::<(Spanned<String>, Spanned<Value>)>())?,
                        add(column.name().len(), 256)?,
                    )?,
                )?;
            }
            if section.entity_kind.is_some() {
                entities = add(entities, 1)?;
                let name = section
                    .name_column
                    .and_then(|key| value.get(key))
                    .and_then(Value::text)
                    .unwrap_or("");
                names = add(names, add(name.len(), 1)?)?;
            }
            Ok(())
        })?;
    }
    // Any acyclic parent path contains each actual entity name at most once.
    // This accounts the potentially quadratic complete qualified-name inventory
    // before identity resolution; cyclic owners are still semantically refused.
    let entity = add(
        map_entry::<pse_ids::SemanticId, pse_relations::generated::authored::entities::Row>(),
        map_entry::<String, ()>(),
    )?;
    add(extent, mul(entities, add(entity, mul(names, 4)?)?)?)
}

pub(super) fn row_extent(
    document: &Document,
    registry: &Registry,
) -> Result<usize, AuthoringError> {
    let mut extent = mul(value_retained(&document.value)?, 4)?;
    visit_rows(document, |section, value| {
        let relation = registry
            .relation(section.relation)
            .ok_or_else(|| super::load::contract(None, "undeclared document relation"))?;
        extent = add(
            extent,
            map_entry::<pse_ids::SemanticId, pse_relations::columnar::FieldCheckedBatch>(),
        )?;
        for column in &relation.columns {
            let child = value.get(column.name());
            extent = add(extent, field_extent(child, &column.value_type())?)?;
            if let Some(child) = child {
                extent = add(
                    extent,
                    expression_extent(
                        child,
                        &column.value_type(),
                        add(section.key.len(), add(column.name().len(), 24)?)?,
                    )?,
                )?;
            }
        }
        Ok(())
    })?;
    Ok(extent)
}

fn field_extent(value: Option<&Value>, ty: &FieldContract) -> Result<usize, AuthoringError> {
    // Generated DTO slots, Arrow builder capacity growth, offsets and validity
    // can coexist before the final checked columns replace their temporary inputs.
    let mut bytes = mul(8, size_of::<Value>())?;
    match (value, ty.extension(), ty.data_type()) {
        (Some(Value::Text(text)), _, _) => bytes = add(bytes, mul(add(text.len(), 64)?, 8)?)?,
        (
            Some(Value::List(values)),
            None,
            datafusion::arrow::datatypes::DataType::List(child)
            | datafusion::arrow::datatypes::DataType::FixedSizeList(child, _),
        ) => {
            let child = &FieldContract::from_field((*child).clone());
            for value in values {
                bytes = add(bytes, field_extent(Some(&value.value), child)?)?;
            }
        }
        (
            Some(value @ Value::Map(_)),
            None,
            datafusion::arrow::datatypes::DataType::Struct(fields),
        ) => {
            for field in &fields {
                let name = field.name();
                let child = &FieldContract::from_field((**field).clone());
                bytes = add(bytes, field_extent(value.get(name), child)?)?;
            }
        }
        // Extension DTOs are declared fixed shapes, or one scalar/list. Their
        // actual source Value already carries every nested slot and string.
        (Some(value), Some(_), _) => bytes = add(bytes, mul(value_retained(value)?, 8)?)?,
        _ => {}
    }
    Ok(bytes)
}

fn expression_extent(
    value: &Value,
    ty: &FieldContract,
    path: usize,
) -> Result<usize, AuthoringError> {
    let mut bytes = mul(add(path, 32)?, 8)?;
    match (value, ty.extension(), ty.data_type()) {
        (Value::Text(text), Some(ExtensionUse::ExprDsl), _) => {
            // A token witnesses every AST node (with at most a wrapper plus a
            // boxed/list node). Names/unit strings together are bounded by the
            // source; nested syntax does not duplicate complete source suffixes.
            let ast = size_of::<dsl::Expr>()
                + size_of::<dsl::Predicate>()
                + size_of::<dsl::Equation>()
                + size_of::<dsl::PathSegment>()
                + size_of::<dsl::NamedArg>();
            let token = size_of::<(&str, dsl::Span, usize)>();
            bytes = add(
                bytes,
                add(
                    mul(lexical_nodes(text)?, mul(4, add(ast, token)?)?)?,
                    mul(text.len(), 8)?,
                )?,
            )?;
        }
        (
            Value::List(values),
            None,
            datafusion::arrow::datatypes::DataType::List(child)
            | datafusion::arrow::datatypes::DataType::FixedSizeList(child, _),
        ) => {
            let child = &FieldContract::from_field((*child).clone());
            for value in values {
                bytes = add(
                    bytes,
                    expression_extent(&value.value, child, add(path, 24)?)?,
                )?;
            }
        }
        (value @ Value::Map(_), None, datafusion::arrow::datatypes::DataType::Struct(fields)) => {
            for field in &fields {
                let name = field.name();
                let child = &FieldContract::from_field((**field).clone());
                if let Some(value) = value.get(name) {
                    bytes = add(
                        bytes,
                        expression_extent(value, child, add(path, add(name.len(), 1)?)?)?,
                    )?;
                }
            }
        }
        _ => {}
    }
    Ok(bytes)
}

pub(super) fn registry_extent(registry: &Registry) -> Result<usize, AuthoringError> {
    let mut bytes = mul(
        registry.relations().len(),
        size_of::<pse_schema::model::RelationSpec>(),
    )?;
    for relation in registry.relations() {
        bytes = add(bytes, mul(relation.primary_key.len(), size_of::<&str>())?)?;
        bytes = add(
            bytes,
            mul(relation.columns.len(), size_of::<FieldContract>())?,
        )?;
        for column in &relation.columns {
            bytes = add(bytes, column.field().size())?;
        }
    }
    bytes = add(
        bytes,
        mul(
            registry.enums().len(),
            size_of::<pse_schema::model::EnumSpec>(),
        )?,
    )?;
    for enumeration in registry.enums() {
        bytes = add(
            bytes,
            mul(
                enumeration.members.len(),
                size_of::<pse_schema::model::EnumMember>(),
            )?,
        )?;
    }
    Ok(bytes)
}
fn source_value<'a>(mut value: &'a Value, path: &str) -> Option<&'a Value> {
    for part in path.split('/').skip(1) {
        value = match value {
            Value::List(values) => &values.get(part.parse::<usize>().ok()?)?.value,
            Value::Map(_) => value.get(&part.replace("~1", "/").replace("~0", "~"))?,
            _ => return None,
        };
    }
    Some(value)
}

/// Retained-capacity accounting can only shrink the completed preflight; it is
/// never used to grow after construction. B-tree nodes remain conservatively full.
pub(super) fn bundle_retained(bundle: &super::DocumentBundle) -> Result<usize, AuthoringError> {
    let mut bytes = mul(bundle.documents.capacity(), size_of::<Document>())?;
    for document in &bundle.documents {
        bytes = add(
            bytes,
            add(document.path.capacity(), document.text.capacity())?,
        )?;
        bytes = add(bytes, value_retained(&document.value)?)?;
        bytes = add(bytes, value_retained(&document.syntax)?)?;
        for path in document.expressions.keys() {
            let value = source_value(&document.value, path).ok_or_else(|| {
                super::load::contract(None, "cached expression source path absent")
            })?;
            bytes = add(
                bytes,
                add(
                    map_entry::<String, super::binding::ParsedExpression>(),
                    expression_extent(
                        value,
                        &FieldContract::extended(ExtensionUse::ExprDsl),
                        path.capacity(),
                    )?,
                )?,
            )?;
        }

        bytes = add(bytes, document.spans.retained_extent()?)?;
        bytes = add(
            bytes,
            mul(
                document.declaration.sections.capacity(),
                size_of::<DocumentSection>(),
            )?,
        )?;
    }
    for batch in bundle.batches.values() {
        bytes = add(bytes, batch.batch().get_array_memory_size())?;
    }
    let package = &bundle.package;
    bytes = add(
        bytes,
        add(
            package.name.capacity(),
            add(package.version.capacity(), package.doc.capacity())?,
        )?,
    )?;
    bytes = add(
        bytes,
        mul(
            package.dependencies.capacity(),
            size_of::<
                pse_relations::generated::authored::packages::AuthoredPackagesFieldDependenciesItem,
            >(),
        )?,
    )?;
    for dependency in &package.dependencies {
        bytes = add(bytes, dependency.version_req.capacity())?;
    }
    Ok(bytes)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_preflight_accounts_complete_escaped_ancestor_paths() {
        let mut text = "0".to_owned();
        for _ in 0..30 {
            text = format!(
                "{{ '{}~/{text_key}': {text} }}",
                "a".repeat(128),
                text_key = "b".repeat(128)
            );
        }
        let budget = ParseBudget::default();
        let id = pse_ids::SemanticId::NIL;
        let (tree, spans) = super::super::value::parse_yaml(&text, id, &budget).unwrap();
        let parser = parser_extent(&text, &budget, true).unwrap();
        let paths = span_extent(&tree.value, 0).unwrap();
        assert!(paths > text.len() * 10);
        assert!(paths >= spans.retained_extent().unwrap());
        let shared = pse_ids::FixedBudget::new(parser + paths - 1);
        let cancel = CancellationToken::new();
        {
            let mut allocation = Allocation::new(shared.as_ref(), &cancel);
            let error = super::super::value::parse_yaml_accounted(
                &text,
                id,
                &budget,
                Some(&mut allocation),
            )
            .unwrap_err();
            assert!(matches!(error, AuthoringError::Resource(_)), "{error}");
            assert_eq!(
                shared.reserved(),
                parser,
                "only the earlier parser phase was reserved; no path construction starts"
            );
        }
        assert_eq!(shared.reserved(), 0);
    }
}
