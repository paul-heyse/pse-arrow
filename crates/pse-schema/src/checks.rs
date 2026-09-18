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

use arrow_schema::DataType as D;
use std::collections::{BTreeMap, BTreeSet};

use crate::builder::Registry;
use crate::error::SchemaError;
use crate::model::{
    Cell, DependencyMode, ExtensionUse, FieldContract, NegationPolicy, RelationDecl,
};

/// Admit a declaration's literal against its resolved logical contract before it can
/// become a migration default or a context-resolved expression value.
pub(crate) fn cell_value(
    value: &Cell,
    ty: &FieldContract,
    nullable: bool,
    registry: &Registry,
) -> bool {
    if matches!(value, Cell::Null) {
        return nullable;
    }
    match crate::model::IntegerRange::from_field(ty.field()) {
        Err(_) => return false,
        Ok(Some(range)) if !matches!(value, Cell::I64(value) if range.contains(*value)) => {
            return false;
        }
        _ => {}
    }
    match crate::model::TaggedAlternative::from_field(ty.field()) {
        Err(_) => return false,
        Ok(Some(alternative)) if !alternative.accepts(ty.field(), value) => return false,
        _ => {}
    }
    match crate::model::CollectionContract::from_field(ty.field()) {
        Err(_) => return false,
        Ok(Some(contract)) if !matches!(value, Cell::List(values) if contract.accepts(values)) => {
            return false;
        }
        _ => {}
    }
    match (value, ty.extension(), ty.data_type()) {
        (Cell::Enum(value), Some(ExtensionUse::Enum(name)), _) => registry
            .enum_spec(name)
            .is_some_and(|spec| spec.members.iter().any(|member| member.name == *value)),
        (Cell::List(values), Some(ExtensionUse::IndexTuple), _) => {
            values.iter().all(|value| matches!(value, Cell::Id(_)))
        }
        (Cell::Id(_), Some(ExtensionUse::SemanticId), _)
        | (Cell::Hash(_), Some(ExtensionUse::ContentHash), _)
        | (Cell::Bool(_), None, D::Boolean)
        | (Cell::F64(_), None, D::Float64)
        | (Cell::I64(_), None, D::Int64 | D::Timestamp(..))
        | (Cell::U64(_), None, D::UInt64)
        | (Cell::I64(_), Some(ExtensionUse::OrdinalRef { .. }), _)
        | (Cell::Text(_), None, D::Utf8)
        | (Cell::Text(_), Some(ExtensionUse::ExprDsl | ExtensionUse::TargetPath), _) => true,
        (Cell::I64(value), None, D::Int32) => i32::try_from(*value).is_ok(),
        (Cell::U64(value), None, D::UInt8) => u8::try_from(*value).is_ok(),
        (Cell::U64(value), None, D::UInt16) => u16::try_from(*value).is_ok(),
        (Cell::U64(value), None, D::UInt32) => u32::try_from(*value).is_ok(),
        (Cell::List(values), None, D::List(field)) => values.iter().all(|value| {
            cell_value(
                value,
                &FieldContract::from_field((*field).clone()),
                field.is_nullable(),
                registry,
            )
        }),
        (Cell::List(values), None, D::FixedSizeList(field, width)) => {
            usize::try_from(width).ok() == Some(values.len())
                && values.iter().all(|value| {
                    cell_value(
                        value,
                        &FieldContract::from_field((*field).clone()),
                        field.is_nullable(),
                        registry,
                    )
                })
        }
        (Cell::Struct(values), None, D::Struct(fields)) => {
            values.len() == fields.len()
                && values.iter().zip(fields.iter()).all(|(value, field)| {
                    cell_value(
                        value,
                        &FieldContract::from_field((**field).clone()),
                        field.is_nullable(),
                        registry,
                    )
                })
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

/// Runs every whole-registry check.
///
/// # Errors
///
/// [`SchemaError::RuleStratification`] or
/// [`SchemaError::InvalidDeclaration`] when a whole-registry contract fails.
pub(crate) fn run(registry: &Registry) -> Result<(), SchemaError> {
    for rule in registry.rules() {
        if let Some(assertion) = &rule.assertion_relation {
            let head = registry
                .relation(&rule.head)
                .ok_or_else(|| invalid(rule.qualified_name(), "undeclared head"))?;
            let assertion = registry
                .relation(assertion)
                .ok_or_else(|| invalid(rule.qualified_name(), "undeclared assertion relation"))?;
            if assertion.primary_key != ["assertion_id"]
                || assertion.columns != crate::model::rule::assertion_columns(&head.columns)
                || assertion.authority != crate::model::Authority::Derived
            {
                return Err(invalid(
                    rule.qualified_name(),
                    "assertion relation is not the exact declared head projection",
                ));
            }
        }
        for writer in registry
            .rules()
            .iter()
            .filter(|writer| writer.head == rule.head)
        {
            if writer.conflict_policy != rule.conflict_policy {
                return Err(invalid(
                    rule.qualified_name(),
                    "one head cannot mix conflict policies",
                ));
            }
        }
        let dependencies = &rule.inputs;
        if dependencies
            .iter()
            .any(|input| input.mode == DependencyMode::Negate)
            && rule.negation != NegationPolicy::Stratified
        {
            return Err(invalid(
                rule.qualified_name(),
                "anti-join requires declared stratified negation",
            ));
        }
        for input in dependencies {
            let relation = input.relation.as_str();
            let mode = input.mode;
            for writer in registry
                .rules()
                .iter()
                .filter(|writer| writer.head == relation)
            {
                if mode == DependencyMode::Negate && writer.stratum >= rule.stratum {
                    return Err(SchemaError::RuleStratification {
                        rule: rule.qualified_name(),
                        relation: relation.to_owned(),
                    });
                }
                if writer.stratum == rule.stratum
                    && mode == DependencyMode::Read
                    && (writer.conflict_policy == crate::model::ConflictPolicy::Undecided
                        || !rule.monotonic)
                {
                    return Err(invalid(
                        rule.qualified_name(),
                        "same-stratum consumer requires a monotone plan and a reject-conflict head",
                    ));
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
                if !cell_value(default, &column.value_type(), column.nullable(), registry) {
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
        }
    }
    if columns.len() != target.columns.len()
        || columns.iter().zip(&target.columns).any(|(actual, target)| {
            actual.name() != target.name()
                || actual.value_type() != target.value_type()
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
    document_grammars(section, relation)?;
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
    let has_dsl = relation.columns.iter().any(|column| {
        let mut nested = Vec::new();
        column.value_type().walk(&mut nested);
        nested
            .into_iter()
            .any(|ty| matches!(ty.extension(), Some(ExtensionUse::ExprDsl)))
    });
    if has_dsl != section.expression_owner_column.is_some()
        || has_dsl != section.expression_owner_kind.is_some()
    {
        return Err(invalid_document(
            section.relation,
            "every DSL-bearing section requires exactly one explicit expression owner",
        ));
    }
    if let Some(owner) = section.expression_owner_column
        && !relation.column(owner).is_some_and(|column| {
            column.value_type() == FieldContract::id()
                && !column.nullable()
                && column.fk().is_some_and(|fk| {
                    section
                        .expression_owner_kind
                        .is_some_and(|kind| kind.target() == (fk.relation, fk.column))
                })
        })
    {
        return Err(invalid_document(
            section.relation,
            "expression owner must be the explicit nonnullable foreign key for its declared kind",
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
        dsl_field_paths(&column.value_type(), column.name(), &mut expected);
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
fn dsl_field_paths(ty: &FieldContract, path: &str, paths: &mut BTreeSet<String>) {
    if matches!(ty.extension(), Some(ExtensionUse::ExprDsl)) {
        paths.insert(path.to_owned());
        return;
    }
    for child in ty.children() {
        let child_path = match ty.data_type() {
            arrow_schema::DataType::List(_)
            | arrow_schema::DataType::LargeList(_)
            | arrow_schema::DataType::FixedSizeList(..) => format!("{path}[]"),
            _ => format!("{path}.{}", child.name()),
        };
        dsl_field_paths(&child, &child_path, paths);
    }
}
