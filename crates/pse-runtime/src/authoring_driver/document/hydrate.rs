// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared context and identity injection before generated DTO decoding.

use super::load::{Document, contract};
use super::value::{Value, synthetic};
use crate::authoring_driver::{DriverError, SourceSpan};
use pse_ids::SemanticId;
use pse_relations::generated::{authored, enums::EntityKind, extension_values};
use pse_schema::Registry;
use pse_schema::model::{DocumentKind, DocumentSection, ExtensionUse, FieldContract, SourceColumn};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

struct Pending {
    document: usize,
    section: DocumentSection,
    ordinal: usize,
    value: Value,
    at: SourceSpan,
}

pub(super) fn documents(
    documents: &mut [Document],
    package: &authored::packages::Row,
    registry: &Registry,
) -> Result<Vec<authored::entities::Row>, DriverError> {
    let mut rows = collect(documents)?;
    for row in &mut rows {
        context(row, package, registry)?;
    }
    let mut entities = BTreeMap::<SemanticId, authored::entities::Row>::new();
    let mut names = BTreeSet::new();
    let mut done = BTreeSet::new();
    while done.len() < rows.len() {
        let before = done.len();
        for (index, row) in rows.iter_mut().enumerate() {
            if done.contains(&index) {
                continue;
            }
            let parent = parent(row)?;
            if parent.is_some_and(|id| !entities.contains_key(&id)) {
                continue;
            }
            let prefix = parent
                .and_then(|id| entities.get(&id))
                .map_or(package.name.as_str(), |parent| &parent.qualified_name);
            if let Some(entity) = identity(row, package, prefix, parent)? {
                if names.contains(&entity.qualified_name)
                    || entities.contains_key(&entity.entity_id)
                {
                    return Err(contract(
                        Some(row.at),
                        &format!(
                            "duplicate entity identity or qualified name: {} ({}) in {}",
                            entity.qualified_name, entity.entity_id, row.section.relation,
                        ),
                    ));
                }
                names.insert(entity.qualified_name.clone());
                entities.insert(entity.entity_id, entity);
            }
            done.insert(index);
        }
        if done.len() == before {
            return Err(contract(
                rows.iter()
                    .enumerate()
                    .find(|(index, _)| !done.contains(index))
                    .map(|(_, row)| row.at),
                "missing or cyclic naming-scope owner",
            ));
        }
    }
    for row in rows {
        let document = &mut documents[row.document];
        let section = document
            .value
            .get_mut(row.section.key)
            .ok_or_else(|| contract(Some(row.at), "missing section during hydration"))?;
        if row.section.repeated {
            let Value::List(values) = section else {
                return Err(contract(Some(row.at), "section must be a list"));
            };
            values[row.ordinal].value = row.value;
        } else {
            *section = row.value;
        }
    }
    Ok(entities.into_values().collect())
}

fn collect(documents: &mut [Document]) -> Result<Vec<Pending>, DriverError> {
    let mut rows = Vec::new();
    for (index, document) in documents.iter_mut().enumerate() {
        let Value::Map(keys) = &document.value else {
            return Err(contract(
                Some(SourceSpan::head(document.id)),
                "document must be a section mapping",
            ));
        };
        for (key, _) in keys {
            if !document
                .declaration
                .sections
                .iter()
                .any(|section| section.key == key.value)
            {
                return Err(DriverError::Authoring(
                    pse_authoring::AuthoringError::UnknownKey {
                        at: document
                            .spans
                            .span(&format!("/{}/@key", key.value))
                            .unwrap_or(SourceSpan::head(document.id)),
                        key: key.value.clone(),
                        context: document.declaration.name.to_owned(),
                    },
                ));
            }
        }
        for section in &document.declaration.sections {
            if document.declaration.kind == DocumentKind::PackageHeader && section.key == "package"
            {
                continue;
            }
            let Some(value) = document.value.get(section.key) else {
                continue;
            };
            let values = if section.repeated {
                let Value::List(values) = value else {
                    return Err(contract(
                        document.spans.span(&format!("/{}", section.key)),
                        "section must be a list",
                    ));
                };
                values.iter().map(|value| &value.value).collect::<Vec<_>>()
            } else {
                vec![value]
            };
            for (ordinal, value) in values.into_iter().enumerate() {
                let path = if section.repeated {
                    format!("/{}/{ordinal}", section.key)
                } else {
                    format!("/{}", section.key)
                };
                let at = document
                    .spans
                    .span(&path)
                    .or_else(|| {
                        (document.declaration.kind == DocumentKind::PackageHeader)
                            .then(|| document.spans.span("/package"))
                            .flatten()
                    })
                    .ok_or_else(|| contract(None, "parser did not retain a row span"))?;
                rows.push(Pending {
                    document: index,
                    section: *section,
                    ordinal,
                    value: value.clone(),
                    at,
                });
            }
        }
    }
    Ok(rows)
}

fn context(
    row: &mut Pending,
    package: &authored::packages::Row,
    registry: &Registry,
) -> Result<(), DriverError> {
    if !matches!(row.value, Value::Map(_)) {
        return Err(contract(Some(row.at), "row must be a mapping"));
    }
    if let Some(alias) = row.value.remove("id") {
        let column = row.section.identity_column.ok_or_else(|| {
            contract(
                Some(row.at),
                "id alias is not declared for this relationship",
            )
        })?;
        if row.value.get(column).is_some() {
            return Err(contract(
                Some(row.at),
                "id alias and identity column both supplied",
            ));
        }
        row.value.set(column, alias);
    }
    let spec = registry
        .relation(row.section.relation)
        .ok_or_else(|| contract(Some(row.at), "unknown relation"))?;
    for column in &spec.columns {
        if row.section.source_column(column) == SourceColumn::PackageContext {
            let expected = package.package_id.to_string();
            if let Some(value) = row.value.get(column.name()) {
                let supplied = value
                    .text()
                    .ok_or_else(|| contract(Some(row.at), "package reference must be an ID"))?;
                if crate::authoring_driver::ids::parse_id(supplied, row.at)? != package.package_id {
                    return Err(contract(
                        Some(row.at),
                        "row package context differs from its document",
                    ));
                }
            }
            row.value.set(column.name(), Value::Text(expected));
        }
        if row.section.source_column(column) == SourceColumn::ParserSpan {
            if row.value.get(column.name()).is_some() {
                return Err(contract(
                    Some(row.at),
                    "source spans come from the original parser",
                ));
            }
            row.value.set(column.name(), span_value(row.at));
        }
        if let Some(value) = row.value.get_mut(column.name()) {
            normalize_ids(value, &column.value_type(), row.at)?;
        }
    }
    Ok(())
}

fn parent(row: &Pending) -> Result<Option<SemanticId>, DriverError> {
    let Some(column) = row.section.naming_scope_column else {
        return Ok(None);
    };
    match row.value.get(column) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .text()
            .ok_or_else(|| contract(Some(row.at), "owner must be a semantic ID"))
            .and_then(|text| {
                crate::authoring_driver::ids::parse_id(text, row.at).map_err(Into::into)
            })
            .map(Some),
    }
}

fn identity(
    row: &mut Pending,
    package: &authored::packages::Row,
    prefix: &str,
    parent: Option<SemanticId>,
) -> Result<Option<authored::entities::Row>, DriverError> {
    let Some(column) = row.section.identity_column else {
        return Ok(None);
    };
    let explicit = row
        .value
        .get(column)
        .map(|value| {
            value
                .text()
                .ok_or_else(|| contract(Some(row.at), "identity must be a string"))
        })
        .transpose()?;
    let Some(kind) = row.section.entity_kind else {
        let text = explicit.ok_or_else(|| {
            DriverError::Authoring(pse_authoring::AuthoringError::MissingId {
                at: row.at,
                kind: row.section.relation.to_owned(),
                name: "unnamed semantic record".to_owned(),
            })
        })?;
        let id = crate::authoring_driver::ids::parse_id(text, row.at)?;
        row.value.set(column, Value::Text(id.to_string()));
        return Ok(None);
    };
    let name = row
        .section
        .name_column
        .and_then(|column| row.value.get(column))
        .and_then(Value::text)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| contract(Some(row.at), "entity requires its declared nonempty name"))?
        .to_owned();
    let qualified_name = format!("{prefix}.{name}");
    let id = crate::authoring_driver::ids::entity_id(
        package.id_policy,
        package.package_id,
        explicit,
        &qualified_name,
        row.at,
    )?;
    row.value.set(column, Value::Text(id.to_string()));
    Ok(Some(authored::entities::Row {
        entity_id: id,
        package_id: package.package_id,
        kind: EntityKind::deserialize(&Value::Text(kind.to_owned()))
            .map_err(|error| contract(Some(row.at), &error.to_string()))?,
        name,
        qualified_name,
        parent_entity_id: parent,
        source_span: Some(extension_values::SourceSpan {
            document_id: row.at.document_id,
            start: i64::from(row.at.start),
            end: i64::from(row.at.end),
        }),
    }))
}

fn normalize_ids(value: &mut Value, ty: &FieldContract, at: SourceSpan) -> Result<(), DriverError> {
    match (value, ty.extension(), ty.data_type()) {
        (Value::Text(text), Some(ExtensionUse::SemanticId), _) => {
            *text = crate::authoring_driver::ids::parse_id(text, at)?.to_string();
        }
        (
            Value::List(values),
            None,
            datafusion::arrow::datatypes::DataType::List(element)
            | datafusion::arrow::datatypes::DataType::FixedSizeList(element, _),
        ) => {
            let element = &FieldContract::from_field((*element).clone());
            for value in values {
                normalize_ids(&mut value.value, element, at)?;
            }
        }
        (value @ Value::Map(_), None, datafusion::arrow::datatypes::DataType::Struct(fields)) => {
            for field in &fields {
                let name = field.name();
                let ty = &FieldContract::from_field((**field).clone());
                if let Some(value) = value.get_mut(name) {
                    normalize_ids(value, ty, at)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn span_value(at: SourceSpan) -> Value {
    Value::Map(vec![
        (
            synthetic("document_id".to_owned()),
            synthetic(Value::Text(at.document_id.to_string())),
        ),
        (
            synthetic("start".to_owned()),
            synthetic(Value::U64(u64::from(at.start))),
        ),
        (
            synthetic("end".to_owned()),
            synthetic(Value::U64(u64::from(at.end))),
        ),
    ])
}

pub(super) fn pending_extent() -> usize {
    4 * (size_of::<Pending>() + size_of::<SourceSpan>())
}
