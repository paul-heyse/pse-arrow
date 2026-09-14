// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact declaration values for generated adapter admission. These are lossless values,
//! not digests: equality compares every contract fact that generated code compiled.

use crate::model::{
    Cell, ColumnSpec, EnumMember, EnumSpec, ExtensionTypeSpec, ExtensionUse, ForeignKey,
    LogicalType, QuantityContract, RelationKey, RelationSpec,
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

fn column(reg: &Registry, column: &ColumnSpec) -> Result<Cell, SchemaError> {
    let ColumnSpec {
        name,
        logical_type,
        nullable,
        quantity,
        fk,
        role,
        doc,
    } = column;
    let quantity = match quantity {
        QuantityContract::None => Cell::Struct(vec![Cell::text("none")]),
        QuantityContract::Column(name) => {
            Cell::Struct(vec![Cell::text("column"), Cell::text(*name)])
        }
        QuantityContract::PerRow => Cell::Struct(vec![Cell::text("per_row")]),
    };
    let fk = match fk {
        None => Cell::Null,
        Some(ForeignKey { relation, column }) => Cell::Struct(vec![
            Cell::text(*relation),
            Cell::text(*column),
            target(reg, relation)?,
        ]),
    };
    Ok(Cell::Struct(vec![
        Cell::text(*name),
        logical(reg, logical_type)?,
        Cell::Bool(*nullable),
        quantity,
        fk,
        Cell::text(role.as_str()),
        Cell::text(*doc),
        field_value(&crate::arrow::field_for(reg, column)?)?,
    ]))
}

fn logical(reg: &Registry, ty: &LogicalType) -> Result<Cell, SchemaError> {
    Ok(match ty {
        LogicalType::List(child) => Cell::Struct(vec![Cell::text("list"), logical(reg, child)?]),
        LogicalType::FixedList(child, width) => Cell::Struct(vec![
            Cell::text("fixed_list"),
            Cell::I64(i64::from(*width)),
            logical(reg, child)?,
        ]),
        LogicalType::Struct(fields) => Cell::Struct(vec![
            Cell::text("struct"),
            Cell::List(
                fields
                    .iter()
                    .map(|(name, ty, nullable)| {
                        Ok(Cell::Struct(vec![
                            Cell::text(*name),
                            logical(reg, ty)?,
                            Cell::Bool(*nullable),
                        ]))
                    })
                    .collect::<Result<_, SchemaError>>()?,
            ),
        ]),
        LogicalType::Ext(use_) => extension(reg, use_)?,
        LogicalType::F64
        | LogicalType::I64
        | LogicalType::I32
        | LogicalType::U8
        | LogicalType::U16
        | LogicalType::U32
        | LogicalType::U64
        | LogicalType::Bool
        | LogicalType::Text
        | LogicalType::Timestamp => {
            Cell::Struct(vec![Cell::text(ty.name()), storage(&ty.data_type())?])
        }
    })
}

fn extension(reg: &Registry, use_: &ExtensionUse) -> Result<Cell, SchemaError> {
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
    Ok(match ty {
        DataType::List(field) => Cell::Struct(vec![Cell::text("List"), field_value(field)?]),
        DataType::FixedSizeList(field, length) => Cell::Struct(vec![
            Cell::text("FixedSizeList"),
            field_value(field)?,
            Cell::I64(i64::from(*length)),
        ]),
        DataType::Struct(fields) => Cell::Struct(vec![
            Cell::text("Struct"),
            Cell::List(
                fields
                    .iter()
                    .map(|field| field_value(field))
                    .collect::<Result<_, _>>()?,
            ),
        ]),
        DataType::Dictionary(key, value) => Cell::Struct(vec![
            Cell::text("Dictionary"),
            storage(key)?,
            storage(value)?,
        ]),
        DataType::Timestamp(unit, timezone) => Cell::Struct(vec![
            Cell::text("Timestamp"),
            Cell::text(match unit {
                arrow_schema::TimeUnit::Second => "second",
                arrow_schema::TimeUnit::Millisecond => "millisecond",
                arrow_schema::TimeUnit::Microsecond => "microsecond",
                arrow_schema::TimeUnit::Nanosecond => "nanosecond",
            }),
            timezone
                .as_ref()
                .map_or(Cell::Null, |zone| Cell::text(zone.as_ref())),
        ]),
        DataType::FixedSizeBinary(length) => Cell::Struct(vec![
            Cell::text("FixedSizeBinary"),
            Cell::I64(i64::from(*length)),
        ]),
        DataType::Float64
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Boolean
        | DataType::Utf8 => Cell::text(crate::model::logical_type::render_data_type(ty)?),
        _ => return Err(missing("unsupported compiled contract Arrow storage")),
    })
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
