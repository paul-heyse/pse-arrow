// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Versioned boundary documents (ADR-0116 Outcome 6). A Rust-owned document that crosses a
//! process, store or language boundary on its own carries `version: Version<N>`: it encodes
//! as the integer `N`, its JSON Schema is the constant `N`, and decoding any other number
//! is refused, never reinterpreted. A change to a document's encoding is a new `N`.
use std::borrow::Cow;

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
        let error = serde_json::from_str::<Version<2>>("1").unwrap_err().to_string();
        assert!(error.contains("unknown document version 1"), "{error}");
    }
}
