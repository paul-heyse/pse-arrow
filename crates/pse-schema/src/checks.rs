// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cross-cutting registry checks that need the whole assembled registry.
//!
//! The per-declaration checks live in [`crate::builder`], where the declaration that
//! fails is still in hand. This module is for the ones that are only decidable once
//! everything is declared — rule stratification against the writers of each relation
//! (blueprint §14.2 rule 2, [`crate::SchemaError::RuleStratification`]) and the stage-graph
//! reachability rules of §14.1 beyond one pass's own ports.
//!
//! Every declared rule and pass participates, including custom registry fixtures.

use std::collections::{BTreeMap, BTreeSet};

use crate::builder::Registry;
use crate::error::SchemaError;
use crate::model::{
    Cell, DependencyMode, ExtensionUse, LogicalType, NegationPolicy, PortSource, RelationDecl,
    RuleHead,
};

/// Admit a declaration's literal against its resolved logical contract before it can
/// become a migration default or a context-resolved expression value.
pub(crate) fn cell_value(
    value: &Cell,
    ty: &LogicalType,
    nullable: bool,
    registry: &Registry,
) -> bool {
    if matches!(value, Cell::Null) {
        return nullable;
    }
    match (value, ty) {
        (Cell::Bool(_), LogicalType::Bool)
        | (Cell::F64(_), LogicalType::F64)
        | (Cell::I64(_), LogicalType::I64 | LogicalType::Timestamp)
        | (Cell::U64(_), LogicalType::U64 | LogicalType::Ext(ExtensionUse::OrdinalRef { .. }))
        | (
            Cell::Text(_),
            LogicalType::Text | LogicalType::Ext(ExtensionUse::ExprDsl | ExtensionUse::TargetPath),
        )
        | (Cell::Id(_), LogicalType::Ext(ExtensionUse::SemanticId))
        | (Cell::Hash(_), LogicalType::Ext(ExtensionUse::ContentHash)) => true,
        (Cell::I64(value), LogicalType::I32) => i32::try_from(*value).is_ok(),
        (Cell::U64(value), LogicalType::U8) => u8::try_from(*value).is_ok(),
        (Cell::U64(value), LogicalType::U16) => u16::try_from(*value).is_ok(),
        (Cell::U64(value), LogicalType::U32) => u32::try_from(*value).is_ok(),
        (Cell::Enum(value), LogicalType::Ext(ExtensionUse::Enum(name))) => registry
            .enum_spec(name)
            .is_some_and(|spec| spec.members.iter().any(|member| member.name == *value)),
        (Cell::List(values), LogicalType::List(element)) => values
            .iter()
            .all(|value| cell_value(value, element, false, registry)),
        (Cell::List(values), LogicalType::FixedList(element, width)) => {
            usize::try_from(*width).ok() == Some(values.len())
                && values
                    .iter()
                    .all(|value| cell_value(value, element, false, registry))
        }
        (Cell::Struct(values), LogicalType::Struct(fields)) => {
            values.len() == fields.len()
                && values
                    .iter()
                    .zip(fields)
                    .all(|(value, (_, ty, nullable))| cell_value(value, ty, *nullable, registry))
        }
        (Cell::List(values), LogicalType::Ext(ExtensionUse::IndexTuple)) => {
            values.iter().all(|value| matches!(value, Cell::Id(_)))
        }
        _ => false,
    }
}

pub(crate) fn invalid(context: impl Into<String>, reason: impl Into<String>) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: context.into(),
        reason: reason.into(),
    }
}

pub(crate) fn relation_declaration(decl: &RelationDecl) -> Result<(), SchemaError> {
    let context = decl.key.to_string();
    if decl.primary_key.is_empty() {
        return Err(invalid(
            &context,
            "a relation requires a nonempty primary key",
        ));
    }
    let mut columns = BTreeSet::new();
    for column in &decl.columns {
        if !columns.insert(column.name) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "column",
                name: format!("{context}.{}", column.name),
            });
        }
        logical_type(&column.logical_type, &format!("{context}.{}", column.name))?;
    }
    let mut keys = BTreeSet::new();
    for key in &decl.primary_key {
        if !keys.insert(*key) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "primary key column",
                name: format!("{context}.{key}"),
            });
        }
        let column = decl
            .columns
            .iter()
            .find(|column| column.name == *key)
            .ok_or_else(|| SchemaError::UnknownReference {
                context: format!("primary key of {context}"),
                reference: (*key).to_owned(),
            })?;
        if column.nullable || !key_type(&column.logical_type) {
            return Err(invalid(
                format!("primary key {context}.{key}"),
                "key columns must be nonnullable identities, enums, ordinals, booleans, timestamps or text",
            ));
        }
    }
    Ok(())
}

/// Key equality must be defined by an admitted scalar contract; floating-point and
/// quantity/composite payloads cannot silently acquire identity semantics.
pub(crate) fn key_type(ty: &LogicalType) -> bool {
    ty.admits_exact_key()
}

fn logical_type(ty: &LogicalType, context: &str) -> Result<(), SchemaError> {
    match ty {
        LogicalType::FixedList(_, width) if *width <= 0 => {
            return Err(invalid(context, "fixed-list width must be positive"));
        }
        LogicalType::List(element) | LogicalType::FixedList(element, _) => {
            logical_type(element, context)?;
        }
        LogicalType::Struct(children) => {
            let mut names = BTreeSet::new();
            for (name, child, _) in children {
                if !names.insert(name) {
                    return Err(SchemaError::DuplicateDeclaration {
                        kind: "struct field",
                        name: format!("{context}.{name}"),
                    });
                }
                logical_type(child, &format!("{context}.{name}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Runs every whole-registry check.
///
/// # Errors
///
/// [`SchemaError::RuleStratification`], [`SchemaError::StageGraph`] or
/// [`SchemaError::InvalidDeclaration`] when a whole-registry contract fails.
pub(crate) fn run(registry: &Registry) -> Result<(), SchemaError> {
    if let Some(manifest) = registry.manifest() {
        manifest_fields(manifest.fields(), "manifest")?;
    }
    for rule in registry.rules() {
        let dependencies = rule.plan.dependencies();
        if dependencies
            .iter()
            .any(|(_, _, mode)| *mode == DependencyMode::Negate)
            && rule.negation != NegationPolicy::Stratified
        {
            return Err(invalid(
                rule.qualified_name(),
                "anti-join requires declared stratified negation",
            ));
        }
        for (relation, _, mode) in dependencies {
            for writer in registry.rules().iter().filter(
                |writer| matches!(&writer.head, RuleHead::Relation(target) if target == relation),
            ) {
                if mode == DependencyMode::Negate && writer.stratum >= rule.stratum {
                    return Err(SchemaError::RuleStratification {
                        rule: rule.qualified_name(),
                        relation: relation.to_owned(),
                    });
                }
                if mode == DependencyMode::Read && writer.stratum > rule.stratum {
                    return Err(invalid(
                        rule.qualified_name(),
                        format!(
                            "reads {relation} before writer {} settles",
                            writer.qualified_name()
                        ),
                    ));
                }
            }
        }
    }
    let mut completed = BTreeSet::new();
    let mut pending: BTreeMap<_, _> = registry
        .passes()
        .iter()
        .map(|pass| (pass.id, pass))
        .collect();
    while !pending.is_empty() {
        let ready: Vec<_> = pending
            .iter()
            .filter(|(_, pass)| {
                pass.inputs.iter().all(|input| match input.source {
                    PortSource::Pinned => true,
                    PortSource::Derived { pass: name, .. } => registry
                        .pass(name)
                        .is_some_and(|producer| completed.contains(&producer.id)),
                })
            })
            .map(|(id, _)| *id)
            .collect();
        if ready.is_empty() {
            return Err(SchemaError::StageGraph {
                reason: "the declared producer graph contains a cycle".to_owned(),
            });
        }
        for id in ready {
            pending.remove(&id);
            completed.insert(id);
        }
    }
    Ok(())
}

fn manifest_fields(
    fields: &[crate::model::ManifestField],
    context: &str,
) -> Result<(), SchemaError> {
    let mut names = BTreeSet::new();
    for field in fields {
        if !names.insert(field.name) {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "manifest field",
                name: format!("{context}.{}", field.name),
            });
        }
        manifest_type(&field.ty, &format!("{context}.{}", field.name))?;
    }
    Ok(())
}

fn manifest_type(ty: &crate::model::ManifestType, context: &str) -> Result<(), SchemaError> {
    match ty {
        crate::model::ManifestType::Struct(fields) => manifest_fields(fields, context),
        crate::model::ManifestType::List(element)
        | crate::model::ManifestType::Optional(element) => manifest_type(element, context),
        _ => Ok(()),
    }
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
                if columns.iter().any(|column| column.name == *name) {
                    return Err(invalid(&context, format!("column {name} already exists")));
                }
                let column = target.column(name).ok_or_else(|| {
                    invalid(&context, format!("new column {name} is absent from target"))
                })?;
                if !cell_value(default, &column.logical_type, column.nullable, registry) {
                    return Err(invalid(
                        &context,
                        format!("default for {name} violates its target type/nullability"),
                    ));
                }
                columns.push(column.clone());
            }
            MigrationStep::DropColumn(name) => {
                let index = columns
                    .iter()
                    .position(|column| column.name == *name)
                    .ok_or_else(|| invalid(&context, format!("column {name} does not exist")))?;
                columns.remove(index);
            }
            MigrationStep::RenameColumn { from, to } => {
                if columns.iter().any(|column| column.name == *to) {
                    return Err(invalid(
                        &context,
                        format!("rename target {to} already exists"),
                    ));
                }
                let column = columns
                    .iter_mut()
                    .find(|column| column.name == *from)
                    .ok_or_else(|| {
                        invalid(&context, format!("rename source {from} does not exist"))
                    })?;
                column.name = to;
            }
            MigrationStep::ChangeNullable { name, nullable } => {
                let column = columns
                    .iter_mut()
                    .find(|column| column.name == *name)
                    .ok_or_else(|| invalid(&context, format!("column {name} does not exist")))?;
                column.nullable = *nullable;
            }
        }
    }
    if columns.len() != target.columns.len()
        || columns.iter().zip(&target.columns).any(|(actual, target)| {
            actual.name != target.name
                || actual.logical_type != target.logical_type
                || actual.nullable != target.nullable
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
                section.expression_owner_column,
                section.expression_fields,
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
                    .and_then(|column| column.fk)
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
    document_grammars(section, relation)?;
    if let Some(identity) = section.identity_column
        && (relation.primary_key != [identity]
            || !relation
                .column(identity)
                .is_some_and(|column| column.logical_type == LogicalType::id() && !column.nullable))
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
        && !relation
            .column(name)
            .is_some_and(|column| column.logical_type == LogicalType::Text && !column.nullable)
    {
        return Err(invalid_document(
            section.relation,
            "name must be a nonnullable text column",
        ));
    }
    if let Some(scope) = section.naming_scope_column
        && !relation
            .column(scope)
            .is_some_and(|column| column.logical_type == LogicalType::id() && column.fk.is_some())
    {
        return Err(invalid_document(
            section.relation,
            "scope must be a semantic-ID foreign key",
        ));
    }
    let has_dsl = relation.columns.iter().any(|column| {
        let mut nested = Vec::new();
        column.logical_type.walk(&mut nested);
        nested
            .into_iter()
            .any(|ty| ty == LogicalType::Ext(ExtensionUse::ExprDsl))
    });
    if has_dsl != section.expression_owner_column.is_some() {
        return Err(invalid_document(
            section.relation,
            "every DSL-bearing section requires exactly one explicit expression owner",
        ));
    }
    if let Some(owner) = section.expression_owner_column
        && !relation.column(owner).is_some_and(|column| {
            column.logical_type == LogicalType::id()
                && !column.nullable
                && column.fk.is_some_and(|fk| {
                    fk.relation == "authored.templates" && fk.column == "template_id"
                })
        })
    {
        return Err(invalid_document(
            section.relation,
            "expression owner must be an explicit nonnullable template identity foreign key",
        ));
    }
    Ok(())
}

fn document_grammars(
    section: &crate::model::DocumentSection,
    relation: &crate::model::RelationSpec,
) -> Result<(), SchemaError> {
    let mut expected = BTreeSet::new();
    for column in &relation.columns {
        dsl_field_paths(&column.logical_type, column.name, &mut expected);
    }
    let declared = section
        .expression_fields
        .iter()
        .map(|(path, _)| (*path).to_owned())
        .collect::<BTreeSet<_>>();
    if declared.len() != section.expression_fields.len() || declared != expected {
        return Err(invalid_document(
            section.relation,
            "DSL grammar mapping must cover every exact exposed DSL leaf once",
        ));
    }
    Ok(())
}
fn dsl_field_paths(ty: &LogicalType, path: &str, paths: &mut BTreeSet<String>) {
    match ty {
        LogicalType::Ext(ExtensionUse::ExprDsl) => {
            paths.insert(path.to_owned());
        }
        LogicalType::List(inner) | LogicalType::FixedList(inner, _) => {
            dsl_field_paths(inner, &format!("{path}[]"), paths);
        }
        LogicalType::Struct(fields) => {
            for (name, ty, _) in fields {
                dsl_field_paths(ty, &format!("{path}.{name}"), paths);
            }
        }
        _ => {}
    }
}
