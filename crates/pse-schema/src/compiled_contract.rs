// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact declaration values for generated adapter admission. These are lossless values,
//! not digests: equality compares every contract fact that generated code compiled.

use crate::model::{
    Cell, EnumMember, EnumSpec, ExtensionTypeSpec, ExtensionUse, FieldContract, RelationKey,
    RelationSpec,
};
use crate::{Registry, SchemaError};
use arrow_schema::{DataType, Field};

/// The complete compiled relation declaration, including resolved enum and extension
/// contracts. Fingerprints are deliberately excluded: they may reject an incompatible
/// declaration early, but cannot establish equivalence.
///
/// # Errors
/// A referenced declaration is absent or its Arrow storage is unsupported.
pub fn relation(reg: &Registry, spec: &RelationSpec) -> Result<Cell, SchemaError> {
    let RelationSpec {
        id,
        key,
        authority,
        snapshot_class,
        derivation_granularity,
        stability,
        primary_key,
        columns,
        checks: _, // The complete native checks are in relation schema metadata below.
        doc,
        fingerprint: _,
    } = spec;
    Ok(Cell::Struct(vec![
        Cell::text("compiled_relation_v1"),
        Cell::Id(*id),
        relation_key(*key),
        Cell::text(authority.as_str()),
        Cell::text(snapshot_class.as_str()),
        derivation_granularity.map_or(Cell::Null, |value| Cell::text(value.as_str())),
        Cell::text(stability.as_str()),
        Cell::List(primary_key.iter().map(|name| Cell::text(*name)).collect()),
        Cell::List(
            columns
                .iter()
                .map(|value| column(reg, value))
                .collect::<Result<_, _>>()?,
        ),
        Cell::text(*doc),
        metadata_value(
            crate::arrow::relation_schema(reg, spec)?
                .metadata()
                .iter()
                .filter(|(key, _)| key.as_str() != crate::arrow::KEY_CONTRACT_FINGERPRINT),
        ),
    ]))
}

fn relation_key(key: RelationKey) -> Cell {
    let RelationKey {
        namespace,
        name,
        version,
    } = key;
    Cell::Struct(vec![
        Cell::text(namespace.as_str()),
        Cell::text(name),
        Cell::U64(u64::from(version)),
    ])
}

fn target(reg: &Registry, name: &str) -> Result<Cell, SchemaError> {
    let spec = reg.relation(name).ok_or_else(|| missing(name))?;
    Ok(Cell::Struct(vec![
        Cell::Id(spec.id),
        relation_key(spec.key),
    ]))
}

fn column(reg: &Registry, column: &FieldContract) -> Result<Cell, SchemaError> {
    Ok(Cell::Struct(vec![
        field_value(column.field())?,
        field_value(&crate::arrow::field_for(reg, column)?)?,
        logical(reg, column)?,
        column
            .fk()
            .map(|fk| target(reg, fk.relation))
            .transpose()?
            .unwrap_or(Cell::Null),
    ]))
}

fn logical(reg: &Registry, ty: &FieldContract) -> Result<Cell, SchemaError> {
    Ok(Cell::Struct(vec![
        ty.extension()
            .map(|use_| extension(reg, &use_))
            .transpose()?
            .unwrap_or(Cell::Null),
        Cell::List(
            ty.children()
                .iter()
                .map(|child| {
                    if ty.extension().is_some() {
                        storage_field(reg, child)
                    } else {
                        column(reg, child)
                    }
                })
                .collect::<Result<_, _>>()?,
        ),
    ]))
}

// Composite extensions already own exact physical child fields. Do not rebind them
// as independent unbound declarations. Include referenced member domains as well as
// the stored native fields so a domain change invalidates the compiled contract.
fn storage_field(reg: &Registry, field: &FieldContract) -> Result<Cell, SchemaError> {
    let domain = field
        .field()
        .metadata()
        .get(crate::arrow::KEY_ENUM)
        .map(|id| {
            reg.enums()
                .iter()
                .find(|spec| spec.id.to_hex() == *id)
                .map(enumeration)
                .ok_or_else(|| missing(id))
        })
        .transpose()?
        .unwrap_or(Cell::Null);
    Ok(Cell::Struct(vec![
        field_value(field.field())?,
        domain,
        Cell::List(
            field
                .children()
                .iter()
                .map(|child| storage_field(reg, child))
                .collect::<Result<_, _>>()?,
        ),
    ]))
}

fn extension(reg: &Registry, use_: &ExtensionUse<'_>) -> Result<Cell, SchemaError> {
    let ExtensionTypeSpec {
        name,
        storage: layout,
        metadata,
        metadata_version,
        doc,
    } = use_.spec();
    let (parameter, identity) = match use_ {
        ExtensionUse::Enum(name) => {
            let spec = reg.enum_spec(name).ok_or_else(|| missing(name))?;
            (enumeration(spec), Some(spec.id))
        }
        ExtensionUse::OrdinalRef { target: name } => {
            let spec = reg.relation(name).ok_or_else(|| missing(name))?;
            (target(reg, name)?, Some(spec.id))
        }
        ExtensionUse::SemanticId
        | ExtensionUse::ContentHash
        | ExtensionUse::DimensionVector
        | ExtensionUse::QuantityValue
        | ExtensionUse::Bound
        | ExtensionUse::IndexTuple
        | ExtensionUse::SourceSpan
        | ExtensionUse::ExprDsl
        | ExtensionUse::TargetPath => (Cell::Null, None),
    };
    Ok(Cell::Struct(vec![
        Cell::text("extension"),
        Cell::text(use_.name()),
        Cell::text(*name),
        storage(&layout())?,
        Cell::text(metadata.to_string()),
        Cell::U64(u64::from(*metadata_version)),
        Cell::text(crate::ext_metadata::canonical(
            *metadata,
            *metadata_version,
            identity,
        )),
        parameter,
        Cell::text(*doc),
    ]))
}

fn enumeration(spec: &EnumSpec) -> Cell {
    let EnumSpec {
        id,
        name,
        idaes_source,
        members,
    } = spec;
    Cell::Struct(vec![
        Cell::Id(*id),
        Cell::text(*name),
        idaes_source.map_or(Cell::Null, Cell::text),
        Cell::List(
            members
                .iter()
                .map(|member| {
                    let EnumMember {
                        name,
                        idaes_name,
                        deprecated,
                        doc,
                    } = member;
                    Cell::Struct(vec![
                        Cell::text(*name),
                        idaes_name.map_or(Cell::Null, Cell::text),
                        Cell::Bool(*deprecated),
                        Cell::text(*doc),
                    ])
                })
                .collect(),
        ),
    ])
}

fn storage(ty: &DataType) -> Result<Cell, SchemaError> {
    Ok(Cell::text(crate::model::render_data_type(ty)?))
}

fn field_value(field: &Field) -> Result<Cell, SchemaError> {
    Ok(Cell::Struct(vec![
        Cell::text(field.name()),
        storage(field.data_type())?,
        Cell::Bool(field.is_nullable()),
        metadata_value(field.metadata()),
    ]))
}

fn metadata_value<'a>(values: impl IntoIterator<Item = (&'a String, &'a String)>) -> Cell {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_by_key(|(key, _)| *key);
    Cell::List(
        values
            .into_iter()
            .map(|(key, value)| Cell::Struct(vec![Cell::text(key), Cell::text(value)]))
            .collect(),
    )
}

fn missing(reference: &str) -> SchemaError {
    SchemaError::UnknownReference {
        context: "generated compiled contract".to_owned(),
        reference: reference.to_owned(),
    }
}
