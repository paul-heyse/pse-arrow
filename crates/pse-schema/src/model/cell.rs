// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registry's own row model.
//!
//! The registry is stored as relations (`reference.schema_*`, blueprint §4.1), so it has
//! to be able to produce its own rows. [`Cell`] is that row model, and it is deliberately
//! the *only* one: a second struct mirroring a relation is the failure
//! `tests/governance/tests/no_shadow_structs.rs` exists to catch, and the registry would
//! be the worst place to start one.

use pse_ids::{ContentHash, FramedHasher, SemanticId, canonical_f64_bits};

/// One value in a registry row.
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    /// An absent value in a nullable column.
    Null,
    /// A `bool` column.
    Bool(bool),
    /// An `i64` or `i32` column.
    I64(i64),
    /// A `u8`, `u16`, `u32` or `u64` column.
    U64(u64),
    /// An `f64` column.
    F64(f64),
    /// A `text` column.
    Text(String),
    /// A `pse.semantic_id` column.
    Id(SemanticId),
    /// A `pse.content_hash` column.
    Hash(ContentHash),
    /// A `pse.enum` column, holding the member name. The declaration ordinal is presentation
    /// only and never appears here (blueprint §4.5).
    Enum(&'static str),
    /// A `list<T>` or `fixed_list<T,N>` column.
    List(Vec<Cell>),
    /// A `struct{...}` column, in declared child order.
    Struct(Vec<Cell>),
}

/// The one-byte tags that frame a [`Cell`] into a registry fingerprint.
///
/// The tag is what keeps `Text("a")` and `Enum("a")` apart: their bytes are identical, and
/// without the tag a relation that turned a label column into an enumeration would keep
/// its fingerprint while changing its contract.
mod tag {
    /// [`super::Cell::Null`].
    pub(super) const NULL: u8 = 0x00;
    /// [`super::Cell::Bool`].
    pub(super) const BOOL: u8 = 0x01;
    /// [`super::Cell::I64`].
    pub(super) const I64: u8 = 0x02;
    /// [`super::Cell::U64`].
    pub(super) const U64: u8 = 0x03;
    /// [`super::Cell::F64`].
    pub(super) const F64: u8 = 0x04;
    /// [`super::Cell::Text`].
    pub(super) const TEXT: u8 = 0x05;
    /// [`super::Cell::Id`].
    pub(super) const ID: u8 = 0x06;
    /// [`super::Cell::Hash`].
    pub(super) const HASH: u8 = 0x07;
    /// [`super::Cell::Enum`].
    pub(super) const ENUM: u8 = 0x08;
    /// [`super::Cell::List`].
    pub(super) const LIST: u8 = 0x09;
    /// [`super::Cell::Struct`].
    pub(super) const STRUCT: u8 = 0x0a;
}

impl Cell {
    /// Decode the tagged literal without losing integer ranges, float bits or nested shapes.
    /// Enum spellings resolve to this registry; use field admission for the expected enum.
    ///
    /// # Errors
    /// Invalid JSON, unknown tags, invalid ranges or undeclared enum member spellings.
    pub fn from_literal_spec(
        text: &str,
        registry: &crate::Registry,
    ) -> Result<Self, crate::SchemaError> {
        super::cell_codec::parse(text, registry)
    }

    /// The value's concrete variant, including the distinction between text and enum.
    const fn literal_kind(&self) -> CellKind {
        match self {
            Self::Null => CellKind::Null,
            Self::Bool(_) => CellKind::Bool,
            Self::I64(_) => CellKind::I64,
            Self::U64(_) => CellKind::U64,
            Self::F64(_) => CellKind::F64,
            Self::Text(_) => CellKind::Text,
            Self::Id(_) => CellKind::Id,
            Self::Hash(_) => CellKind::Hash,
            Self::Enum(_) => CellKind::Enum,
            Self::List(_) => CellKind::List,
            Self::Struct(_) => CellKind::Struct,
        }
    }

    /// A lossless, tagged JSON literal, used by migration plan text. Float payloads use
    /// hexadecimal bits to preserve signed zero, nonfinite values and NaN payloads.
    pub fn literal_spec(&self) -> String {
        let value = match self {
            Self::Null => "null".to_owned(),
            Self::Bool(value) => value.to_string(),
            Self::I64(value) => value.to_string(),
            Self::U64(value) => value.to_string(),
            Self::F64(value) => format!("\"{:016x}\"", value.to_bits()),
            Self::Text(value) => json_string(value),
            Self::Enum(value) => json_string(value),
            Self::Id(value) => json_string(&value.to_hex()),
            Self::Hash(value) => json_string(&value.to_string()),
            Self::List(values) | Self::Struct(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Self::literal_spec)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        };
        format!("[\"{}\",{value}]", self.literal_kind().as_str())
    }
    /// A text cell from anything that renders as one.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// An optional text cell: [`Cell::Null`] when absent.
    pub fn opt_text(value: Option<&str>) -> Self {
        value.map_or(Self::Null, Self::text)
    }

    /// An optional identity cell: [`Cell::Null`] when absent.
    pub fn opt_id(value: Option<SemanticId>) -> Self {
        value.map_or(Self::Null, Self::Id)
    }

    /// An optional enumeration cell: [`Cell::Null`] when absent.
    pub fn opt_enum(value: Option<&'static str>) -> Self {
        value.map_or(Self::Null, Self::Enum)
    }

    /// Contributes this cell to `hasher` (blueprint §5.1 framing, ADR-0050).
    ///
    /// One framed part per scalar — a tag byte followed by the typed bytes — and, for the
    /// composites, a tag part, a framed length and then the elements. `f64` goes through
    /// [`canonical_f64_bits`] so that a NaN in a default value cannot make a registry
    /// fingerprint depend on which NaN the compiler produced (ADR-0030).
    pub(crate) fn frame(&self, hasher: &mut FramedHasher) {
        match self {
            Self::Null => {
                hasher.part(&[tag::NULL]);
            }
            Self::Bool(value) => {
                hasher.part(&[tag::BOOL, u8::from(*value)]);
            }
            Self::I64(value) => {
                hasher.part(&tagged(tag::I64, &value.to_le_bytes()));
            }
            Self::U64(value) => {
                hasher.part(&tagged(tag::U64, &value.to_le_bytes()));
            }
            Self::F64(value) => {
                hasher.part(&tagged(tag::F64, &canonical_f64_bits(*value).to_le_bytes()));
            }
            Self::Text(value) => {
                hasher.part(&tagged(tag::TEXT, value.as_bytes()));
            }
            Self::Id(value) => {
                hasher.part(&tagged(tag::ID, value.as_bytes()));
            }
            Self::Hash(value) => {
                hasher.part(&tagged(tag::HASH, value.as_bytes()));
            }
            Self::Enum(value) => {
                hasher.part(&tagged(tag::ENUM, value.as_bytes()));
            }
            Self::List(elements) => {
                hasher.part(&[tag::LIST]);
                frame_sequence(elements, hasher);
            }
            Self::Struct(children) => {
                hasher.part(&[tag::STRUCT]);
                frame_sequence(children, hasher);
            }
        }
    }
}

fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            character if character < '\u{20}' => {
                use std::fmt::Write as _;
                let _ = write!(result, "\\u{:04x}", u32::from(character));
            }
            character => result.push(character),
        }
    }
    result.push('"');
    result
}

/// A tag byte followed by `bytes`, as one part's payload.
fn tagged(tag: u8, bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 1);
    out.push(tag);
    out.extend_from_slice(bytes);
    out
}

/// A framed element count followed by the elements.
///
/// The saturating conversion is unreachable on every supported target; it exists because
/// the crate's panic policy has no room for an `expect` only a 128-bit address space could
/// reach.
fn frame_sequence(elements: &[Cell], hasher: &mut FramedHasher) {
    hasher.u64(u64::try_from(elements.len()).unwrap_or(u64::MAX));
    for element in elements {
        element.frame(hasher);
    }
}

/// The declared `CellKind` wire vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CellKind {
    /// `null`.
    Null,
    /// `bool`.
    Bool,
    /// `i64`.
    I64,
    /// `u64`.
    U64,
    /// `f64`.
    F64,
    /// `text`.
    Text,
    /// `id`.
    Id,
    /// `hash`.
    Hash,
    /// `enum`.
    Enum,
    /// `list`.
    List,
    /// `struct`.
    Struct,
}
impl CellKind {
    /// The wire spelling.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool => "bool",
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::F64 => "f64",
            Self::Text => "text",
            Self::Id => "id",
            Self::Hash => "hash",
            Self::Enum => "enum",
            Self::List => "list",
            Self::Struct => "struct",
        }
    }
}

#[cfg(test)]
mod tests {
    use pse_ids::derive::context;

    use super::*;

    /// The digest of one cell on its own.
    fn digest(cell: &Cell) -> ContentHash {
        let mut hasher = FramedHasher::new(context::REGISTRY);
        cell.frame(&mut hasher);
        hasher.finish_hash()
    }

    #[test]
    fn every_cell_shape_frames_distinctly() {
        let cells = [
            Cell::Null,
            Cell::Bool(false),
            Cell::Bool(true),
            Cell::I64(1),
            Cell::U64(1),
            Cell::F64(1.0),
            Cell::text("a"),
            Cell::Id(SemanticId::from_bytes([0x01; 16])),
            Cell::Hash(ContentHash::from_bytes([0x01; 32])),
            Cell::Enum("a"),
            Cell::List(vec![Cell::text("a")]),
            Cell::Struct(vec![Cell::text("a")]),
        ];
        let mut digests: Vec<ContentHash> = cells.iter().map(digest).collect();
        let before = digests.len();
        digests.sort_unstable_by_key(|hash| *hash.as_bytes());
        digests.dedup();
        assert_eq!(digests.len(), before, "two cell shapes share a framing");
    }

    #[test]
    fn a_text_cell_and_an_enum_cell_with_the_same_bytes_differ() {
        assert_ne!(digest(&Cell::text("molar")), digest(&Cell::Enum("molar")));
    }

    #[test]
    fn nesting_is_not_flattening() {
        assert_ne!(
            digest(&Cell::List(vec![Cell::List(vec![Cell::U64(1)])])),
            digest(&Cell::List(vec![Cell::U64(1)])),
        );
        assert_ne!(
            digest(&Cell::List(vec![Cell::text("ab"), Cell::text("c")])),
            digest(&Cell::List(vec![Cell::text("a"), Cell::text("bc")])),
        );
    }

    #[test]
    fn nan_does_not_leak_its_bit_pattern() {
        let one = f64::from_bits(0x7ff8_0000_0000_0001);
        let other = f64::from_bits(0xfff8_0000_0000_0000);
        assert!(one.is_nan() && other.is_nan());
        assert_eq!(digest(&Cell::F64(one)), digest(&Cell::F64(other)));
        assert_ne!(digest(&Cell::F64(0.0)), digest(&Cell::F64(-0.0)));
    }
}
