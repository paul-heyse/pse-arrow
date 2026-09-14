// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed extension reconstruction from the single registry extension declaration.

mod formatter;

pub use formatter::{PseFormatterFactory, create_owned_formatter};

use arrow_schema::extension::ExtensionType;
use arrow_schema::{ArrowError, DataType, Field};
use pse_ids::SemanticId;
use pse_schema::model::{EXTENSION_TYPES, ExtensionMetadataShape, ExtensionTypeSpec};
use serde::Deserialize;

/// The exact versioned extension metadata admitted by blueprint §4.4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtMetadata {
    /// Metadata generation.
    pub v: u32,
    /// The declared enum identity, for `pse.enum` only.
    pub enum_id: Option<SemanticId>,
    /// The target relation identity, for `pse.ordinal_ref` only.
    pub target_relation_id: Option<SemanticId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionWire {
    v: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumWire {
    v: u32,
    enum_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinalWire {
    v: u32,
    target_relation_id: String,
}

impl ExtMetadata {
    /// Parses one of the three closed shapes, rejecting duplicate fields and IDs that
    /// do not use the declared lowercase hexadecimal spelling.
    ///
    /// # Errors
    /// [`ArrowError::InvalidArgumentError`] for malformed or unsupported metadata.
    pub fn parse(spec: &ExtensionTypeSpec, text: &str) -> Result<Self, ArrowError> {
        let parse_error = |error: serde_json::Error| invalid(&error.to_string());
        let (v, enum_id, target_relation_id) = match spec.metadata {
            ExtensionMetadataShape::VersionOnly => {
                let value: VersionWire = serde_json::from_str(text).map_err(parse_error)?;
                (value.v, None, None)
            }
            ExtensionMetadataShape::Enum => {
                let value: EnumWire = serde_json::from_str(text).map_err(parse_error)?;
                (value.v, Some(parse_id(&value.enum_id)?), None)
            }
            ExtensionMetadataShape::OrdinalRef => {
                let value: OrdinalWire = serde_json::from_str(text).map_err(parse_error)?;
                (value.v, None, Some(parse_id(&value.target_relation_id)?))
            }
        };
        let value = Self {
            v,
            enum_id,
            target_relation_id,
        };
        value.check(spec)?;
        Ok(value)
    }

    /// The canonical string written by the registry's own metadata encoder.
    pub fn canonical_json(&self, spec: &ExtensionTypeSpec) -> String {
        pse_schema::ext_metadata::canonical(
            spec.metadata,
            self.v,
            self.enum_id.or(self.target_relation_id),
        )
    }

    fn check(&self, spec: &ExtensionTypeSpec) -> Result<(), ArrowError> {
        let valid_shape = match spec.metadata {
            ExtensionMetadataShape::VersionOnly => {
                self.enum_id.is_none() && self.target_relation_id.is_none()
            }
            ExtensionMetadataShape::Enum => {
                self.enum_id.is_some() && self.target_relation_id.is_none()
            }
            ExtensionMetadataShape::OrdinalRef => {
                self.enum_id.is_none() && self.target_relation_id.is_some()
            }
        };
        if self.v != spec.metadata_version || !valid_shape {
            return Err(invalid(
                "extension metadata has an unsupported version or shape",
            ));
        }
        Ok(())
    }
}

fn parse_id(text: &str) -> Result<SemanticId, ArrowError> {
    let id = SemanticId::parse_hex(text).map_err(|error| invalid(&error.to_string()))?;
    if id.to_hex() != text {
        return Err(invalid(
            "extension identities require 32 lowercase hexadecimal digits",
        ));
    }
    Ok(id)
}

fn invalid(reason: &str) -> ArrowError {
    ArrowError::InvalidArgumentError(reason.to_owned())
}

/// A platform extension bound to its authoritative registry-table entry.
pub trait PseExtension: ExtensionType {
    /// The single extension declaration driving storage and metadata validation.
    const SPEC: &'static ExtensionTypeSpec;
}

macro_rules! extension {
    ($name:ident, $ordinal:expr) => {
        #[doc = concat!("Typed reconstruction of `", stringify!($name), "` under blueprint §4.4.")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name(ExtMetadata);
        impl PseExtension for $name {
            const SPEC: &'static ExtensionTypeSpec = &EXTENSION_TYPES[$ordinal];
        }
        impl ExtensionType for $name {
            const NAME: &'static str = EXTENSION_TYPES[$ordinal].name;
            type Metadata = ExtMetadata;
            fn metadata(&self) -> &Self::Metadata {
                &self.0
            }
            fn serialize_metadata(&self) -> Option<String> {
                Some(self.0.canonical_json(Self::SPEC))
            }
            fn deserialize_metadata(metadata: Option<&str>) -> Result<Self::Metadata, ArrowError> {
                ExtMetadata::parse(
                    Self::SPEC,
                    metadata.ok_or_else(|| invalid("extension metadata is required"))?,
                )
            }
            fn supports_data_type(&self, data_type: &DataType) -> Result<(), ArrowError> {
                if data_type == &Self::SPEC.storage() {
                    Ok(())
                } else {
                    Err(invalid(
                        "extension storage differs from its declared contract",
                    ))
                }
            }
            fn try_new(data_type: &DataType, metadata: Self::Metadata) -> Result<Self, ArrowError> {
                metadata.check(Self::SPEC)?;
                let extension = Self(metadata);
                extension.supports_data_type(data_type)?;
                Ok(extension)
            }
        }
    };
}

extension!(PseSemanticId, 0);
extension!(PseContentHash, 1);
extension!(PseDimensionVector, 2);
extension!(PseQuantityValue, 3);
extension!(PseBound, 4);
extension!(PseIndexTuple, 5);
extension!(PseOrdinalRef, 6);
extension!(PseSourceSpan, 7);
extension!(PseEnum, 8);
extension!(PseExprDsl, 9);
extension!(PseTargetPath, 10);

/// Invokes the typed, non-panicking Arrow factory for a claimed platform extension.
///
/// # Errors
/// [`ArrowError::InvalidArgumentError`] for unknown names, storage or metadata.
pub fn validate_extension(field: &Field) -> Result<(), ArrowError> {
    macro_rules! check {
        ($ty:ty) => {
            field.try_extension_type::<$ty>().map(|_| ())
        };
    }
    match field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .map(String::as_str)
    {
        Some(PseSemanticId::NAME) => check!(PseSemanticId),
        Some(PseContentHash::NAME) => check!(PseContentHash),
        Some(PseDimensionVector::NAME) => check!(PseDimensionVector),
        Some(PseQuantityValue::NAME) => check!(PseQuantityValue),
        Some(PseBound::NAME) => check!(PseBound),
        Some(PseIndexTuple::NAME) => check!(PseIndexTuple),
        Some(PseOrdinalRef::NAME) => check!(PseOrdinalRef),
        Some(PseSourceSpan::NAME) => check!(PseSourceSpan),
        Some(PseEnum::NAME) => check!(PseEnum),
        Some(PseExprDsl::NAME) => check!(PseExprDsl),
        Some(PseTargetPath::NAME) => check!(PseTargetPath),
        _ => Err(invalid("extension name is not registered")),
    }
}
