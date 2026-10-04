// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Versioned boundary documents (ADR-0116 Outcome 6). A Rust-owned document that crosses a
//! process, store or language boundary on its own carries `version: Version<N>`: it encodes
//! as the integer `N`, its JSON Schema is the constant `N`, and decoding any other number
//! is refused, never reinterpreted. A change to a document's encoding is a new `N`.
use std::borrow::Cow;

/// Decode a standalone current JSON document after admitting its version header.
/// Historical bodies are never decoded with current nested scientific defaults.
///
/// # Errors
/// Missing, malformed or unsupported versions and malformed current bodies are refused.
pub fn decode_versioned<T: serde::de::DeserializeOwned, const N: u32>(
    bytes: &[u8],
) -> Result<T, crate::ModelError> {
    #[derive(serde::Deserialize)]
    struct Header {
        version: u32,
    }
    let header: Header = serde_json::from_slice(bytes)
        .map_err(|error| crate::malformed(&format!("document version header: {error}")))?;
    if header.version != N {
        return Err(crate::malformed(&format!(
            "document version {} is unsupported (current: {N}); explicit readmission is required",
            header.version
        )));
    }
    serde_json::from_slice(bytes)
        .map_err(|error| crate::malformed(&format!("current document body: {error}")))
}

/// Schema projection of serde's closed standard-duration representation.
/// Integer widths and fields come from schemars's Duration projection; serde's
/// standard visitor owns decoding, overflow and unknown-field refusal.
#[derive(Debug)]
pub struct ClosedDuration;
impl schemars::JsonSchema for ClosedDuration {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("ClosedDuration")
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut schema = <std::time::Duration as schemars::JsonSchema>::json_schema(generator);
        schema.insert("additionalProperties".into(), false.into());
        schema
    }
}

/// The version field of a document at version `N`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version<const N: u32>;

impl<const N: u32> Version<N> {
    /// The version number this type encodes and accepts.
    pub const NUMBER: u32 = N;
}

impl<const N: u32> serde::Serialize for Version<N> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(N)
    }
}

impl<'de, const N: u32> serde::Deserialize<'de> for Version<N> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = <u32 as serde::Deserialize>::deserialize(deserializer)?;
        if version == N {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unknown document version {version}; this build reads version {N}"
            )))
        }
    }
}

impl<const N: u32> schemars::JsonSchema for Version<N> {
    fn inline_schema() -> bool {
        true
    }
    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!("Version{N}"))
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({ "type": "integer", "const": N, "default": N })
    }
}

#[cfg(test)]
mod tests {
    use super::Version;

    #[test]
    fn unknown_document_version_refused() {
        assert_eq!(serde_json::to_string(&Version::<2>).unwrap(), "2");
        assert!(serde_json::from_str::<Version<2>>("2").is_ok());
        let error = serde_json::from_str::<Version<2>>("1")
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown document version 1"), "{error}");
    }

    #[test]
    fn version_admission_precedes_current_body_even_when_header_is_last() {
        #[derive(Debug, serde::Deserialize)]
        struct Current {
            #[serde(rename = "version")]
            _version: Version<2>,
            #[serde(rename = "required")]
            _required: u64,
        }
        let historical = br#"{"required":"not a current integer","version":1}"#;
        let error = super::decode_versioned::<Current, 2>(historical).unwrap_err();
        assert!(error.to_string().contains("explicit readmission"));
        assert!(!error.to_string().contains("current integer"));
        assert!(super::decode_versioned::<Current, 2>(br#"{"required":5,"version":2}"#).is_ok());
        assert!(super::decode_versioned::<Current, 2>(br#"{"required":5}"#).is_err());
        assert!(
            super::decode_versioned::<Current, 2>(br#"{"required":5,"version":2,"version":2}"#)
                .is_err()
        );
    }
}
