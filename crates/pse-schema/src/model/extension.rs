// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The eleven `pse.*` Arrow extension types and their metadata shapes
//! (blueprint §4.4, revision 6).
//!
//! Every extension has a standard storage type so that an unaware consumer reads it
//! safely: a Python reader without the registered class sees `FixedSizeBinary(16)`, not a
//! failure. That is also why the loss is asymmetric and recorded — the Rust admission
//! layer errors on an unregistered `pse.*` name, the Python one degrades (§21.1).
//!
//! This table is the single authority. `pse-relations` generates its `ExtensionType`
//! implementations from it, the Python contract package generates its `pyarrow`
//! classes from it, and the DataFusion session registers exactly these names.

use core::fmt;

use arrow_schema::{DataType, Field, Fields, TimeUnit};

/// What an extension's `ARROW:extension:metadata` looks like (blueprint §4.4 revision 6).
///
/// Every shape carries `v`, which is the extension-type migration mechanism (DM-44,
/// DM-51): a factory accepts the generations it knows and rejects the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtensionMetadataShape {
    /// `{"v":1}`.
    VersionOnly,
    /// `{"v":1,"enum_id":"<32 hex>"}`.
    Enum,
    /// `{"v":1,"target_relation_id":"<32 hex>"}`.
    OrdinalRef,
}

impl ExtensionMetadataShape {
    /// The JSON key this shape adds beside `v`, if any.
    pub const fn id_key(self) -> Option<&'static str> {
        match self {
            Self::VersionOnly => None,
            Self::Enum => Some("enum_id"),
            Self::OrdinalRef => Some("target_relation_id"),
        }
    }
}

impl fmt::Display for ExtensionMetadataShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::VersionOnly => "version_only",
            Self::Enum => "enum",
            Self::OrdinalRef => "ordinal_ref",
        })
    }
}

/// One `pse.*` extension type: its name, its storage and its metadata contract.
#[derive(Clone, Copy, Debug)]
pub struct ExtensionTypeSpec {
    /// The `ARROW:extension:name` value, always `pse.<something>`.
    pub name: &'static str,
    /// The declared storage. A function rather than a constant because
    /// `arrow_schema::DataType` owns `Arc`s and cannot be built in a `const`.
    pub storage: fn() -> DataType,
    /// The shape of `ARROW:extension:metadata`.
    pub metadata: ExtensionMetadataShape,
    /// The generation the platform writes. Readers accept the generations they know.
    pub metadata_version: u32,
    /// What a value of this type means.
    pub doc: &'static str,
}

impl ExtensionTypeSpec {
    /// The declared storage, built fresh.
    pub fn storage(&self) -> DataType {
        (self.storage)()
    }
}

/// The storage of `pse.semantic_id`.
fn semantic_id_storage() -> DataType {
    DataType::FixedSizeBinary(16)
}

/// The storage of `pse.content_hash`.
fn content_hash_storage() -> DataType {
    DataType::FixedSizeBinary(32)
}

/// The storage of `pse.dimension_vector`: rational exponents over the eight base
/// dimensions (length, mass, time, temperature, amount, current, luminous intensity,
/// currency).
fn dimension_vector_storage() -> DataType {
    let member = DataType::Struct(Fields::from(vec![
        Field::new("num", DataType::Int16, false),
        Field::new("den", DataType::Int16, false),
    ]));
    DataType::FixedSizeList(Field::new_list_field(member, false).into(), 8)
}

/// The storage of `pse.quantity_value`: a value in a heterogeneous column that carries its
/// own contract.
fn quantity_value_storage() -> DataType {
    DataType::Struct(Fields::from(vec![
        Field::new("value", DataType::Float64, false),
        Field::new("quantity_type_id", DataType::FixedSizeBinary(16), false),
        Field::new("unit_id", DataType::FixedSizeBinary(16), false),
    ]))
}

/// The storage of `pse.bound`: an explicit bound, never a NaN or null sentinel
/// (blueprint §7.6).
///
/// `kind` carries the meaning, so `value` is null exactly when `kind = unbounded`. That is
/// not the sentinel §7.6 bans: the banned form is a null (or `±inf`, or NaN) that *means*
/// unbounded while `kind` says nothing. Here `kind` is non-null and decides, and a
/// non-nullable `value` would force a fabricated number into every unbounded row.
fn bound_storage() -> DataType {
    DataType::Struct(Fields::from(vec![
        Field::new(
            "kind",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            false,
        ),
        Field::new("value", DataType::Float64, true),
    ]))
}

/// The storage of `pse.index_tuple`: ordered domain member identities.
fn index_tuple_storage() -> DataType {
    DataType::List(Field::new_list_field(DataType::FixedSizeBinary(16), false).into())
}

/// The storage of `pse.ordinal_ref`: an artifact-local reference whose target relation is
/// named by the metadata.
fn ordinal_ref_storage() -> DataType {
    DataType::UInt64
}

/// The storage of `pse.source_span`: authoring provenance.
fn source_span_storage() -> DataType {
    DataType::Struct(Fields::from(vec![
        Field::new("document_id", DataType::FixedSizeBinary(16), false),
        Field::new("start", DataType::UInt32, false),
        Field::new("end", DataType::UInt32, false),
    ]))
}

/// The storage of `pse.enum`: a closed dictionary whose codes are presentation only.
fn enum_storage() -> DataType {
    DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8))
}

/// The storage of `pse.expr_dsl`: authored expression text.
fn expr_dsl_storage() -> DataType {
    DataType::Utf8
}

/// The storage of `pse.target_path`: an authored selector path.
fn target_path_storage() -> DataType {
    DataType::Utf8
}

/// The canonical Arrow storage of a `ts` logical type (blueprint §4.5).
pub(crate) fn timestamp_storage() -> DataType {
    DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into()))
}

/// The eleven extension types of blueprint §4.4, in the order the table lists them.
///
/// The order is part of the contract: `docs/generated/extension_types.md` and the
/// generated registration list follow it, and a registration that replaced an earlier one
/// would be invisible in an unordered set.
pub const EXTENSION_TYPES: [ExtensionTypeSpec; 11] = [
    ExtensionTypeSpec {
        name: "pse.semantic_id",
        storage: semantic_id_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "128-bit semantic identity (blueprint §5.1).",
    },
    ExtensionTypeSpec {
        name: "pse.content_hash",
        storage: content_hash_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "A BLAKE3 digest identifying an immutable version (blueprint §5.3).",
    },
    ExtensionTypeSpec {
        name: "pse.dimension_vector",
        storage: dimension_vector_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "Rational exponents over the eight base dimensions (blueprint §8.1).",
    },
    ExtensionTypeSpec {
        name: "pse.quantity_value",
        storage: quantity_value_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "A value in a heterogeneous column with an explicit quantity contract.",
    },
    ExtensionTypeSpec {
        name: "pse.bound",
        storage: bound_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "An explicit bound, `finite` or `unbounded`; never an infinity or a null sentinel.",
    },
    ExtensionTypeSpec {
        name: "pse.index_tuple",
        storage: index_tuple_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "Ordered domain member identities (blueprint §5.1).",
    },
    ExtensionTypeSpec {
        name: "pse.ordinal_ref",
        storage: ordinal_ref_storage,
        metadata: ExtensionMetadataShape::OrdinalRef,
        metadata_version: 1,
        doc: "An artifact-local reference; the metadata names the target relation.",
    },
    ExtensionTypeSpec {
        name: "pse.source_span",
        storage: source_span_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "Authoring provenance: a document and a byte range (blueprint §11).",
    },
    ExtensionTypeSpec {
        name: "pse.enum",
        storage: enum_storage,
        metadata: ExtensionMetadataShape::Enum,
        metadata_version: 1,
        doc: "A closed enumeration; the metadata carries the `enum_id`.",
    },
    ExtensionTypeSpec {
        name: "pse.expr_dsl",
        storage: expr_dsl_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "Authored expression text: the only authored form of an expression (§7.7).",
    },
    ExtensionTypeSpec {
        name: "pse.target_path",
        storage: target_path_storage,
        metadata: ExtensionMetadataShape::VersionOnly,
        metadata_version: 1,
        doc: "An authored selector path; P1 resolves it to identities at commit (§6.10).",
    },
];
