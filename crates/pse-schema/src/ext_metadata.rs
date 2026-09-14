// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The canonical `ARROW:extension:metadata` strings (blueprint §4.4 revision 6).
//!
//! Three shapes and nothing else: `{"v":1}`, `{"v":1,"enum_id":"<32 hex>"}` and
//! `{"v":1,"target_relation_id":"<32 hex>"}`, with no whitespace and in that key order.
//! A factory rejects any other shape.
//!
//! Built by hand rather than with `serde_json`, for two reasons. The string is a schema
//! metadata value and therefore a canonical-metadata-relation value under §5.3 step 3, so
//! it enters a logical hash; a serializer's key order and number formatting are its own
//! choices, and the one that happens to be right today is not a contract. And
//! `pse-schema` has no `serde_json` dependency: ADR-0049 pins it for the manifest and the
//! `pse-relations` metadata codec, which reads these strings back with a strict parser.

use pse_ids::SemanticId;

use crate::model::extension::ExtensionMetadataShape;

/// The canonical metadata string for `shape` at `version`.
///
/// `id` is required for [`ExtensionMetadataShape::Enum`] and
/// [`ExtensionMetadataShape::OrdinalRef`]; [`SemanticId::NIL`] stands in when it is
/// absent, which only happens if a caller skipped the registry lookup that would have
/// failed first — [`crate::arrow::field_for`] errors with
/// [`crate::SchemaError::UnknownReference`] before it gets here.
///
/// ```
/// use pse_ids::SemanticId;
/// use pse_schema::ext_metadata::canonical;
/// use pse_schema::model::ExtensionMetadataShape;
///
/// assert_eq!(canonical(ExtensionMetadataShape::VersionOnly, 1, None), r#"{"v":1}"#);
/// assert_eq!(
///     canonical(ExtensionMetadataShape::Enum, 1, Some(SemanticId::from_bytes([0xab; 16]))),
///     r#"{"v":1,"enum_id":"abababababababababababababababab"}"#,
/// );
/// ```
pub fn canonical(shape: ExtensionMetadataShape, version: u32, id: Option<SemanticId>) -> String {
    match shape.id_key() {
        None => format!("{{\"v\":{version}}}"),
        Some(key) => {
            let hex = id.unwrap_or(SemanticId::NIL).to_hex();
            format!("{{\"v\":{version},\"{key}\":\"{hex}\"}}")
        }
    }
}

/// The JSON Schema that describes `shape` at `version`.
///
/// Stored in `reference.schema_logical_types.metadata_schema` (blueprint §4.1) and emitted
/// into `docs/generated/schema/`. `additionalProperties: false` is the point: an
/// extension whose metadata grew a key nobody declared is exactly the silent fork the
/// `v` generation counter exists to prevent (DM-44, DM-51).
pub fn json_schema(shape: ExtensionMetadataShape, version: u32) -> String {
    match shape.id_key() {
        None => format!(
            "{{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"v\"],\
             \"properties\":{{\"v\":{{\"const\":{version}}}}}}}"
        ),
        Some(key) => format!(
            "{{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"v\",\"{key}\"],\
             \"properties\":{{\"v\":{{\"const\":{version}}},\
             \"{key}\":{{\"type\":\"string\",\"pattern\":\"^[0-9a-f]{{32}}$\"}}}}}}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_shapes_are_exactly_these_bytes() {
        let id = SemanticId::from_bytes([0x01; 16]);
        assert_eq!(
            canonical(ExtensionMetadataShape::VersionOnly, 1, None),
            "{\"v\":1}"
        );
        assert_eq!(
            canonical(ExtensionMetadataShape::Enum, 1, Some(id)),
            "{\"v\":1,\"enum_id\":\"01010101010101010101010101010101\"}"
        );
        assert_eq!(
            canonical(ExtensionMetadataShape::OrdinalRef, 1, Some(id)),
            "{\"v\":1,\"target_relation_id\":\"01010101010101010101010101010101\"}"
        );
    }

    #[test]
    fn there_is_no_whitespace_anywhere() {
        for shape in [
            ExtensionMetadataShape::VersionOnly,
            ExtensionMetadataShape::Enum,
            ExtensionMetadataShape::OrdinalRef,
        ] {
            let rendered = canonical(shape, 1, Some(SemanticId::NIL));
            assert!(
                !rendered.contains(' '),
                "{rendered} carries whitespace a byte comparison would trip over"
            );
        }
    }

    #[test]
    fn a_later_generation_renders_its_own_number() {
        assert_eq!(
            canonical(ExtensionMetadataShape::VersionOnly, 2, None),
            "{\"v\":2}"
        );
        assert!(json_schema(ExtensionMetadataShape::VersionOnly, 2).contains("\"const\":2"));
    }

    #[test]
    fn the_schema_closes_the_object() {
        for shape in [
            ExtensionMetadataShape::VersionOnly,
            ExtensionMetadataShape::Enum,
            ExtensionMetadataShape::OrdinalRef,
        ] {
            assert!(json_schema(shape, 1).contains("\"additionalProperties\":false"));
        }
        assert!(json_schema(ExtensionMetadataShape::Enum, 1).contains("^[0-9a-f]{32}$"));
    }
}
