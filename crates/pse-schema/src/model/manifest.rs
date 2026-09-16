// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The `pse.manifest.v2` envelope, declared (blueprint §20.2, ADR-0051).
//!
//! The manifest is the *physical* envelope around the *semantic* membership of §5.3. Its
//! own checksum is held by the ref, never embedded in itself, because a document cannot
//! contain its own digest.
//!
//! This declaration controls generated Rust/Python wire structs and documentation.
//! Checked native bindings preserve Rust role types without repeating wire fields
//! (ADR-0060); catalog admission remains separate from the generated envelope.

use core::fmt;

/// The type of one manifest field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestType {
    /// A JSON string.
    Text,
    /// A JSON number holding an unsigned 32-bit value.
    U32,
    /// A JSON number holding an unsigned 64-bit value.
    U64,
    /// A JSON boolean.
    Bool,
    /// A 32-hex-digit semantic identity.
    Id,
    /// A `blake3:<64 hex>` digest.
    Hash,
    /// An RFC 3339 timestamp.
    Timestamp,
    /// A JSON array.
    List(Box<ManifestType>),
    /// A JSON object with declared members.
    Struct(Vec<ManifestField>),
    /// A member that may be absent or null.
    Optional(Box<ManifestType>),
}

/// An existing Rust representation of a declared manifest wire value (ADR-0060).
/// These bindings select codecs; they never change the underlying wire shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestRustBinding {
    /// `pse_ids::SnapshotKind`, encoded as a declared lowercase kind string.
    SnapshotKind,
    /// `pse_ids::SnapshotId`, encoded as the ordinary prefixed hash string.
    SnapshotId,
    /// `pse_ids::LogicalHash`, encoded as the ordinary prefixed hash string.
    LogicalHash,
    /// `pse_ids::EncodingChecksum`, encoded as the ordinary prefixed hash string.
    EncodingChecksum,
    /// `pse_ids::SchemaVersion`, encoded as an unsigned 32-bit number.
    SchemaVersion,
    /// The catalog's `EncodingFormat`, encoded as its declared string.
    EncodingFormat,
    /// `Vec<pse_ids::SnapshotParent>`, using the generated nested wire object.
    SnapshotParents,
}
impl ManifestRustBinding {
    /// Whether the declared wire shape is exactly supported by this native codec.
    pub(crate) fn accepts(self, ty: &ManifestType) -> bool {
        match self {
            Self::SnapshotKind | Self::EncodingFormat => matches!(ty, ManifestType::Text),
            Self::SnapshotId | Self::LogicalHash | Self::EncodingChecksum => {
                matches!(ty, ManifestType::Hash)
            }
            Self::SchemaVersion => matches!(ty, ManifestType::U32),
            Self::SnapshotParents => {
                let ManifestType::List(child) = ty else {
                    return false;
                };
                let ManifestType::Struct(fields) = child.as_ref() else {
                    return false;
                };
                matches!(fields.as_slice(), [role, id]
                    if role.name == "role" && role.ty == ManifestType::Text && role.rust.is_none()
                    && id.name == "snapshot_id" && id.ty == ManifestType::Hash
                    && id.rust == Some(Self::SnapshotId))
            }
        }
    }
}

impl ManifestType {
    /// A list of `element`.
    pub fn list(element: Self) -> Self {
        Self::List(Box::new(element))
    }

    /// An optional `inner`.
    pub fn optional(inner: Self) -> Self {
        Self::Optional(Box::new(inner))
    }

    /// The rendering used in generated documentation and type names.
    pub fn name(&self) -> String {
        match self {
            Self::Text => "text".to_owned(),
            Self::U32 => "u32".to_owned(),
            Self::U64 => "u64".to_owned(),
            Self::Bool => "bool".to_owned(),
            Self::Id => "semantic_id".to_owned(),
            Self::Hash => "content_hash".to_owned(),
            Self::Timestamp => "ts".to_owned(),
            Self::List(element) => format!("list<{}>", element.name()),
            Self::Struct(members) => {
                let rendered: Vec<String> = members
                    .iter()
                    .map(|member| format!("{}:{}", member.name, member.ty.name()))
                    .collect();
                format!("struct{{{}}}", rendered.join(","))
            }
            Self::Optional(inner) => format!("{}?", inner.name()),
        }
    }
}

impl fmt::Display for ManifestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

/// One manifest field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestField {
    /// The JSON key.
    pub name: &'static str,
    /// The field's type.
    pub ty: ManifestType,
    /// What the field records.
    pub doc: &'static str,
    /// Optional checked native representation; the wire type remains authoritative.
    pub rust: Option<ManifestRustBinding>,
}

impl ManifestField {
    /// A field with the given type.
    pub const fn new(name: &'static str, ty: ManifestType, doc: &'static str) -> Self {
        Self {
            name,
            ty,
            doc,
            rust: None,
        }
    }

    /// Preserve an existing Rust role type through a codec compatible with `ty`.
    #[must_use]
    pub const fn with_rust(mut self, binding: ManifestRustBinding) -> Self {
        self.rust = Some(binding);
        self
    }
}

/// The declared `pse.manifest.v2` envelope (blueprint §20.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestSpec {
    /// The manifest version string, which is also the value of `manifest_version`.
    pub version: &'static str,
    /// The membership profile the manifest describes (blueprint §5.3 step 7).
    pub membership_profile: &'static str,
    /// The fields, in declaration order.
    fields: Vec<ManifestField>,
}

impl ManifestSpec {
    /// The frozen manifest version string.
    pub const VERSION: &'static str = "pse.manifest.v2";

    /// The frozen membership profile string (ADR-0050).
    pub const MEMBERSHIP_PROFILE: &'static str = "pse.snapshot.v2";

    /// A manifest declaration.
    pub const fn new(
        version: &'static str,
        membership_profile: &'static str,
        fields: Vec<ManifestField>,
    ) -> Self {
        Self {
            version,
            membership_profile,
            fields,
        }
    }

    /// The fields, in declaration order.
    pub fn fields(&self) -> &[ManifestField] {
        &self.fields
    }

    /// The field of that name, if the manifest has one.
    pub fn field(&self, name: &str) -> Option<&ManifestField> {
        self.fields.iter().find(|field| field.name == name)
    }
}
