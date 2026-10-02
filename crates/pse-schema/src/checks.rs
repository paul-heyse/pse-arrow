// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cross-cutting registry checks that need the whole assembled registry.
//!
//! The per-declaration checks live in [`crate::builder`], where the declaration that
//! fails is still in hand. This module is for the ones that are only decidable once
//! everything is declared: migration and document references.

use std::collections::{BTreeMap, BTreeSet};

use crate::builder::Registry;
use crate::error::SchemaError;
use crate::model::{FieldContract, RelationDecl};

/// Admit a declaration's literal against its resolved logical contract before it can
/// become a migration default or a context-resolved expression value.
pub(crate) fn invalid(context: impl Into<String>, reason: impl Into<String>) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: context.into(),
        reason: reason.into(),
    }
}

pub(crate) fn relation_declaration(decl: &RelationDecl) -> Result<(), SchemaError> {
    crate::arrow::validate_delta_properties(&decl.delta_properties)?;
    let context = decl.key.to_string();
    if decl
        .checks
        .iter()
        .any(|(name, sql)| name.is_empty() || sql.trim().is_empty())
    {
        return Err(invalid(
            &context,
            "native check names and expressions must be nonempty",
        ));
    }
    let primary_key = decl.primary_key.as_ref().ok_or_else(|| {
        invalid(
            &context,
            "a relation requires an explicit primary key declaration",
        )
    })?;
    let mut columns = BTreeSet::new();
    for column in &decl.columns {
        if !columns.insert(column.name()) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "column",
                name: format!("{context}.{}", column.name()),
            });
        }
        logical_type(
            &column.value_type(),
            &format!("{context}.{}", column.name()),
        )?;
    }
    let mut keys = BTreeSet::new();
    for key in primary_key {
        if !keys.insert(*key) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "primary key column",
                name: format!("{context}.{key}"),
            });
        }
        let column = decl
            .columns
            .iter()
            .find(|column| column.name() == *key)
            .ok_or_else(|| SchemaError::UnknownReference {
                context: format!("primary key of {context}"),
                reference: (*key).to_owned(),
            })?;
        if column.nullable() || !key_type(&column.value_type()) {
            return Err(invalid(
                format!("primary key {context}.{key}"),
                "key columns must be nonnullable identities, enums, ordinals, booleans, timestamps or text",
            ));
        }
    }
    constraint_declarations(decl, &context)
}

/// Unique keys and table-level references name distinct, existing, exact-key columns.
/// Their targets resolve at assembly, once every relation is declared.
fn constraint_declarations(decl: &RelationDecl, context: &str) -> Result<(), SchemaError> {
    let mut names = BTreeSet::new();
    let keys = decl
        .unique_keys
        .iter()
        .map(|key| (key.name, &key.columns, "unique key"));
    let references = decl
        .foreign_keys
        .iter()
        .map(|reference| (reference.name, &reference.columns, "foreign key"));
    for (name, columns, kind) in keys.chain(references) {
        let label = format!("{kind} {context}:{name}");
        if !names.insert((kind, name)) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "key constraint",
                name: label,
            });
        }
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(invalid(
                &label,
                "constraint names are snake-case identifiers",
            ));
        }
        let mut seen = BTreeSet::new();
        if columns.is_empty() || !columns.iter().all(|column| seen.insert(*column)) {
            return Err(invalid(&label, "a key names one or more distinct columns"));
        }
        for column in columns.iter() {
            let field = decl
                .columns
                .iter()
                .find(|field| field.name() == *column)
                .ok_or_else(|| SchemaError::UnknownReference {
                    context: label.clone(),
                    reference: (*column).to_owned(),
                })?;
            if !key_type(&field.value_type()) {
                return Err(invalid(&label, "key columns must have exact-key types"));
            }
        }
    }
    for reference in &decl.foreign_keys {
        if reference.columns.len() != reference.target_columns.len() {
            return Err(invalid(
                format!("foreign key {context}:{}", reference.name),
                "a reference pairs each local column with one target column",
            ));
        }
    }
    Ok(())
}

/// Key equality must be defined by an admitted scalar contract; floating-point and
/// quantity/composite payloads cannot silently acquire identity semantics.
pub(crate) fn key_type(ty: &FieldContract) -> bool {
    ty.admits_exact_key()
}

fn logical_type(ty: &FieldContract, context: &str) -> Result<(), SchemaError> {
    if matches!(ty.data_type(), arrow_schema::DataType::FixedSizeList(_, width) if width < 0) {
        return Err(invalid(context, "fixed-list width cannot be negative"));
    }
    if let Some(use_) = ty.extension() {
        if ty.data_type() != use_.spec().storage() {
            return Err(invalid(
                context,
                "domain extension storage differs from its declaration",
            ));
        }
        return Ok(());
    }
    let mut names = BTreeSet::new();
    for child in ty.children() {
        if !names.insert(child.name().to_owned()) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "struct field",
                name: format!("{context}.{}", child.name()),
            });
        }
        logical_type(&child, &format!("{context}.{}", child.name()))?;
    }
    Ok(())
}

pub(crate) fn migration(
    spec: &crate::model::MigrationSpec,
    registry: &Registry,
) -> Result<(), SchemaError> {
    use crate::model::MigrationStep;
    let context = spec.qualified_name();
    if spec.from_version >= spec.to_version {
        return Err(invalid(
            &context,
            "a migration must advance the relation version",
        ));
    }
    let version = |number| {
        registry
            .relations()
            .iter()
            .find(|relation| {
                relation.qualified_name() == spec.relation && relation.key.version == number
            })
            .ok_or_else(|| SchemaError::UnknownReference {
                context: context.clone(),
                reference: format!("{}@{number}", spec.relation),
            })
    };
    let source = version(spec.from_version)?;
    let target = version(spec.to_version)?;
    let mut columns = source.columns.clone();
    for step in &spec.steps {
        match step {
            MigrationStep::AddColumn { name, default } => {
                if columns.iter().any(|column| column.name() == *name) {
                    return Err(invalid(&context, format!("column {name} already exists")));
                }
                let column = target.column(name).ok_or_else(|| {
                    invalid(&context, format!("new column {name} is absent from target"))
                })?;
                if FieldContract::from_field(default.field().as_ref().clone()) != *column {
                    return Err(invalid(
                        &context,
                        format!("default for {name} does not carry its exact target field"),
                    ));
                }
                columns.push(column.clone());
            }
            MigrationStep::DropColumn(name) => {
                let index = columns
                    .iter()
                    .position(|column| column.name() == *name)
                    .ok_or_else(|| invalid(&context, format!("column {name} does not exist")))?;
                columns.remove(index);
            }
            MigrationStep::RenameColumn { from, to } => {
                if columns.iter().any(|column| column.name() == *to) {
                    return Err(invalid(
                        &context,
                        format!("rename target {to} already exists"),
                    ));
                }
                let column = columns
                    .iter_mut()
                    .find(|column| column.name() == *from)
                    .ok_or_else(|| {
                        invalid(&context, format!("rename source {from} does not exist"))
                    })?;
                *column = column.clone().with_name(*to);
            }
            MigrationStep::ChangeNullable { name, nullable } => {
                let column = columns
                    .iter_mut()
                    .find(|column| column.name() == *name)
                    .ok_or_else(|| invalid(&context, format!("column {name} does not exist")))?;
                *column = column.clone().with_nullable(*nullable);
            }
            MigrationStep::RecodeDomain { name, mapping } => {
                let index = columns
                    .iter()
                    .position(|column| column.name() == *name)
                    .ok_or_else(|| invalid(&context, "recoding source column is absent"))?;
                let target_column = target
                    .column(name)
                    .ok_or_else(|| invalid(&context, "recoding target column is absent"))?;
                mapping.validate_fields(
                    &[crate::arrow::field_for(registry, &columns[index])?],
                    &[crate::arrow::field_for(registry, target_column)?],
                    false,
                )?;
                columns[index] = target_column.clone();
            }
            MigrationStep::MapReferenceKey {
                columns: names,
                mapping,
            } => {
                let mut selected = BTreeSet::new();
                let indices = names
                    .iter()
                    .map(|name| {
                        if !selected.insert(*name) {
                            return Err(invalid(&context, "mapping column is duplicated"));
                        }
                        columns
                            .iter()
                            .position(|column| column.name() == *name)
                            .ok_or_else(|| invalid(&context, "mapping source column is absent"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let target_columns = names
                    .iter()
                    .map(|name| {
                        target
                            .column(name)
                            .ok_or_else(|| invalid(&context, "mapping target column is absent"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                mapping.validate_fields(
                    &indices
                        .iter()
                        .map(|index| crate::arrow::field_for(registry, &columns[*index]))
                        .collect::<Result<Vec<_>, _>>()?,
                    &target_columns
                        .iter()
                        .map(|column| crate::arrow::field_for(registry, column))
                        .collect::<Result<Vec<_>, _>>()?,
                    true,
                )?;
                for (index, target_column) in indices.into_iter().zip(target_columns) {
                    columns[index] = target_column.clone();
                }
            }
        }
    }
    if columns.len() != target.columns.len()
        || columns.iter().zip(&target.columns).any(|(actual, target)| {
            actual.name() != target.name()
                || crate::fingerprint::semantic_field(actual.field()).ok()
                    != crate::fingerprint::semantic_field(target.field()).ok()
                || actual.nullable() != target.nullable()
        })
    {
        return Err(invalid(
            context,
            "migration steps do not produce the declared target column order, types and nullability",
        ));
    }
    Ok(())
}

/// Validate explicit authoring projection semantics without inferring identities.
pub(crate) fn documents(
    documents: &[crate::model::DocumentSpec],
    registry: &Registry,
) -> Result<(), SchemaError> {
    let mut mappings = BTreeMap::new();
    for document in documents {
        for section in &document.sections {
            let projection = (
                section.identity_column,
                section.entity_kind,
                section.name_column,
                section.naming_scope_column,
            );
            if mappings
                .insert(section.relation, projection)
                .is_some_and(|prior| prior != projection)
            {
                return Err(invalid_document(
                    section.relation,
                    "conflicting identity projection across documents",
                ));
            }
        }
    }
    for document in documents {
        for section in &document.sections {
            document_section(section, registry)?;
            if let Some(scope) = section.naming_scope_column {
                let owner = registry
                    .relation(section.relation)
                    .and_then(|relation| relation.column(scope))
                    .and_then(FieldContract::fk)
                    .ok_or_else(|| {
                        invalid_document(
                            section.relation,
                            "naming scope must be an explicit foreign key",
                        )
                    })?;
                if !mappings.get(owner.relation).is_some_and(|projection| {
                    projection.0 == Some(owner.column) && projection.1.is_some()
                }) {
                    return Err(invalid_document(
                        section.relation,
                        "naming scope must target a declared entity identity",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn invalid_document(relation: &str, detail: &str) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: format!("document projection {relation}"),
        reason: detail.to_owned(),
    }
}

fn document_section(
    section: &crate::model::DocumentSection,
    registry: &Registry,
) -> Result<(), SchemaError> {
    let relation = registry
        .relation(section.relation)
        .ok_or_else(|| invalid_document(section.relation, "unknown relation"))?;
    if let Some(identity) = section.identity_column
        && (relation.primary_key != [identity]
            || !relation.column(identity).is_some_and(|column| {
                column.value_type() == FieldContract::id() && !column.nullable()
            }))
    {
        return Err(invalid_document(
            section.relation,
            "identity must be the sole nonnullable semantic-ID primary key",
        ));
    }
    if section.entity_kind.is_some() != section.name_column.is_some()
        || (section.entity_kind.is_some() && section.identity_column.is_none())
        || (section.naming_scope_column.is_some() && section.entity_kind.is_none())
    {
        return Err(invalid_document(
            section.relation,
            "entity kind, identity, name and scope are inconsistent",
        ));
    }
    if let Some(kind) = section.entity_kind
        && !registry
            .enum_spec("EntityKind")
            .is_some_and(|spec| spec.members.iter().any(|member| member.name == kind))
    {
        return Err(invalid_document(
            section.relation,
            "unknown EntityKind member",
        ));
    }
    if let Some(name) = section.name_column
        && !relation.column(name).is_some_and(|column| {
            column.value_type() == FieldContract::native(arrow_schema::DataType::Utf8)
                && !column.nullable()
        })
    {
        return Err(invalid_document(
            section.relation,
            "name must be a nonnullable text column",
        ));
    }
    if let Some(scope) = section.naming_scope_column
        && !relation.column(scope).is_some_and(|column| {
            column.value_type() == FieldContract::id() && column.fk().is_some()
        })
    {
        return Err(invalid_document(
            section.relation,
            "scope must be a semantic-ID foreign key",
        ));
    }
    Ok(())
}
