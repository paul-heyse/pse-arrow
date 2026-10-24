// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Versioned boundary documents (ADR-0116 Outcome 6). A Rust-owned document that crosses a
//! process, store or language boundary on its own carries `version: Version<N>`: it encodes
//! as the integer `N`, its JSON Schema is the constant `N`, and decoding any other number
//! is refused, never reinterpreted. A change to a document's encoding is a new `N`.
use std::borrow::Cow;

/// The native decoding phase, separate from authored scientific source locations.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DecodeStage {
    /// Admit the declared standalone interpretation before decoding the body.
    Header,
    /// Interpret the admitted current domain body.
    Body,
    /// Refuse non-whitespace content after the complete document.
    Trailing,
}

/// Original serde_json error classification, without its potentially sensitive message.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DecodeCategory {
    /// The input source could not be read.
    Io,
    /// The original bytes do not form the required JSON syntax.
    Syntax,
    /// The JSON value violates the actual Rust decoder's contract.
    Data,
    /// The input ended before the document was complete.
    Eof,
}

/// A structural segment observed by serde_path_to_error, never parsed from display text.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DecodePathSegment {
    /// A JSON object key. It may be caller-supplied, not a declared schema property.
    Map {
        /// Original key; public diagnostic projections must filter undeclared keys.
        key: String,
    },
    /// A zero-based array position.
    Sequence {
        /// Original array position.
        index: usize,
    },
    /// A variant identified by the actual Serde decoder.
    Variant {
        /// Original decoder-observed variant; apply the same filtering as object keys.
        variant: String,
    },
    /// The adapter could not identify this path segment.
    Unknown,
}

/// How much the decoder could establish about a failure's structural location.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DecodePathStatus {
    /// A tracked path prefix. Custom/untagged decoders may hide the exact failing member.
    TrackedPrefix,
    /// No path was established, or one of its segments is unknown.
    Unavailable,
}

/// Original JSON decode facts, with no authored source identity or offending input value.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DecodeLocation {
    /// Actual native decode phase.
    pub stage: DecodeStage,
    /// Original serde_json classification.
    pub category: DecodeCategory,
    /// Original one-based line; zero remains possible for unlocated custom errors.
    pub line: usize,
    /// Original serde_json column; zero is possible immediately after a newline.
    pub column: usize,
    /// Original tracked structural segments; object keys need a privacy-filtered projection.
    pub path: Vec<DecodePathSegment>,
    /// Precision actually established by the adapter.
    pub path_status: DecodePathStatus,
}

/// A JSON decoder refusal retaining its original library error privately.
#[derive(Debug, thiserror::Error)]
#[error(
    "JSON document decoding refused; correct the declared document or use explicit readmission"
)]
pub struct JsonDecodeError {
    location: DecodeLocation,
    // Intentionally not an Error::source: serde messages can contain supplied values.
    inner: serde_json::Error,
}
pse_diagnostics::impl_diagnostic! {
    JsonDecodeError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::ValidationInvariant) },
    forward(_this) { None },
    help(_this) { Some(Box::new("Inspect the decode location and the selected document contract; unsupported versions require explicit readmission.")) },
    related(_this) { None }, source(_this) { None }
}
impl JsonDecodeError {
    fn tracked(
        stage: DecodeStage,
        path: serde_path_to_error::Path,
        inner: serde_json::Error,
    ) -> Self {
        let path: Vec<_> = path
            .iter()
            .map(|segment| match segment {
                serde_path_to_error::Segment::Seq { index } => {
                    DecodePathSegment::Sequence { index: *index }
                }
                serde_path_to_error::Segment::Map { key } => {
                    DecodePathSegment::Map { key: key.clone() }
                }
                serde_path_to_error::Segment::Enum { variant } => DecodePathSegment::Variant {
                    variant: variant.clone(),
                },
                serde_path_to_error::Segment::Unknown => DecodePathSegment::Unknown,
            })
            .collect();
        Self::at(stage, path, inner)
    }
    fn at(stage: DecodeStage, path: Vec<DecodePathSegment>, inner: serde_json::Error) -> Self {
        let category = match inner.classify() {
            serde_json::error::Category::Io => DecodeCategory::Io,
            serde_json::error::Category::Syntax => DecodeCategory::Syntax,
            serde_json::error::Category::Data => DecodeCategory::Data,
            serde_json::error::Category::Eof => DecodeCategory::Eof,
        };
        let path_status = if path.is_empty()
            || path
                .iter()
                .any(|segment| matches!(segment, DecodePathSegment::Unknown))
        {
            DecodePathStatus::Unavailable
        } else {
            DecodePathStatus::TrackedPrefix
        };
        Self {
            location: DecodeLocation {
                stage,
                category,
                line: inner.line(),
                column: inner.column(),
                path,
                path_status,
            },
            inner,
        }
    }
    /// Structured location facts; map keys must be filtered before a public safe projection.
    pub fn location(&self) -> &DecodeLocation {
        &self.location
    }
    /// Original serde_json error for native diagnosis; its message can contain supplied values.
    pub fn inner(&self) -> &serde_json::Error {
        &self.inner
    }
}

/// Decode the actual current Rust owner from original bytes and refuse trailing content.
///
/// # Errors
/// The actual Serde contract or JSON syntax refuses the original bytes.
pub fn decode_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, JsonDecodeError> {
    struct Value<T>(std::marker::PhantomData<T>);
    impl<'de, T: serde::Deserialize<'de>> serde::de::DeserializeSeed<'de> for Value<T> {
        type Value = T;
        fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<T, D::Error> {
            T::deserialize(deserializer)
        }
    }
    decode_seed(
        bytes,
        Value::<T>(std::marker::PhantomData),
        DecodeStage::Body,
    )
}
fn decode_seed<'de, S: serde::de::DeserializeSeed<'de>>(
    bytes: &'de [u8],
    seed: S,
    stage: DecodeStage,
) -> Result<S::Value, JsonDecodeError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let mut track = serde_path_to_error::Track::new();
    let value = seed
        .deserialize(serde_path_to_error::Deserializer::new(
            &mut deserializer,
            &mut track,
        ))
        .map_err(|inner| JsonDecodeError::tracked(stage, track.path(), inner))?;
    deserializer
        .end()
        .map_err(|inner| JsonDecodeError::at(DecodeStage::Trailing, Vec::new(), inner))?;
    Ok(value)
}

/// Admit a required version on the original byte stream before interpreting the body.
/// Duplicate keys cannot disappear through an intermediate Value map.
///
/// # Errors
/// Missing, duplicate, malformed or unsupported versions and trailing content are refused.
pub fn check_required_header(
    bytes: &[u8],
    field: &'static str,
    expected: u32,
) -> Result<(), JsonDecodeError> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Header {
        field: &'static str,
        expected: u32,
    }
    impl<'de> Visitor<'de> for Header {
        type Value = ();
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                formatter,
                "an object with required {} {}",
                self.field, self.expected
            )
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
            let mut seen = false;
            while let Some(key) = map.next_key::<String>()? {
                if key == self.field {
                    if seen {
                        return Err(A::Error::duplicate_field(self.field));
                    }
                    seen = true;
                    let version = map.next_value::<u32>()?;
                    if version != self.expected {
                        return Err(A::Error::custom(format!(
                            "unsupported {} {version} (current: {}); explicit readmission is required",
                            self.field, self.expected
                        )));
                    }
                } else {
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
            if !seen {
                return Err(A::Error::missing_field(self.field));
            }
            Ok(())
        }
    }
    impl<'de> serde::de::DeserializeSeed<'de> for Header {
        type Value = ();
        fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
            deserializer.deserialize_map(self)
        }
    }
    decode_seed(bytes, Header { field, expected }, DecodeStage::Header)
}

/// Decode a standalone current JSON document after admitting its version header.
/// Historical bodies are never decoded with current nested scientific defaults.
///
/// # Errors
/// Missing, malformed or unsupported versions and malformed current bodies are refused.
pub fn decode_versioned<T: serde::de::DeserializeOwned, const N: u32>(
    bytes: &[u8],
) -> Result<T, JsonDecodeError> {
    check_required_header(bytes, "version", N)?;
    decode_json(bytes)
}

/// Shared schema projection of serde's closed standard-duration representation.
pub use pse_diagnostics::ClosedDuration;

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
    #[test]
    fn structural_decode_preserves_segments_and_original_error_facts_without_values() {
        #[derive(Debug, serde::Deserialize)]
        struct Nested {
            #[serde(rename = "a/b~c")]
            _rows: Vec<std::collections::BTreeMap<String, u64>>,
        }
        let bytes = br#"{
            "a/b~c": [{"sensitive.key": "secret-value"}]
        }"#;
        let error = super::decode_json::<Nested>(bytes).unwrap_err();
        let location = error.location();
        assert_eq!(location.stage, super::DecodeStage::Body);
        assert_eq!(location.category, super::DecodeCategory::Data);
        assert_eq!(location.path_status, super::DecodePathStatus::TrackedPrefix);
        assert_eq!(
            location.path,
            vec![
                super::DecodePathSegment::Map {
                    key: "a/b~c".into()
                },
                super::DecodePathSegment::Sequence { index: 0 },
                super::DecodePathSegment::Map {
                    key: "sensitive.key".into()
                },
            ]
        );
        assert_eq!(location.line, 2);
        assert_eq!(location.line, error.inner().line());
        assert_eq!(location.column, error.inner().column());
        assert!(error.inner().is_data());
        assert!(error.inner().to_string().contains("secret-value"));
        assert!(!error.to_string().contains("secret-value"));
        assert!(!error.to_string().contains("sensitive.key"));
        assert!(std::error::Error::source(&error).is_none());
        assert!(miette::Diagnostic::code(&error).is_some());
    }

    #[test]
    fn structural_decode_rejects_trailing_content_and_retains_syntax_eof_categories() {
        #[derive(Debug, serde::Deserialize)]
        struct Current {
            #[serde(rename = "version")]
            _version: Version<2>,
            #[serde(rename = "value")]
            _value: u64,
        }
        for bytes in [
            br#"{"value":7,"version":2} {"version":2}"#.as_slice(),
            br#"{"value":7,"version":2} false"#,
        ] {
            let error = super::decode_versioned::<Current, 2>(bytes).unwrap_err();
            assert_eq!(error.location().stage, super::DecodeStage::Trailing);
            assert_eq!(error.location().category, super::DecodeCategory::Syntax);
            assert_eq!(
                error.location().path_status,
                super::DecodePathStatus::Unavailable
            );
            assert!(error.location().path.is_empty());
        }
        assert!(super::decode_versioned::<Current, 2>(b"{\"value\":7,\"version\":2} \n\t").is_ok());
        let error = super::decode_json::<Current>(br#"{"version":2,"value":"#).unwrap_err();
        assert_eq!(error.location().category, super::DecodeCategory::Eof);
        assert!(error.inner().is_eof());
        let error = super::decode_json::<Current>(br#"{"version":2,"value":!}"#).unwrap_err();
        assert_eq!(error.location().category, super::DecodeCategory::Syntax);
        assert!(error.inner().is_syntax());
    }

    #[test]
    fn structural_header_admission_refuses_duplicates_before_current_body() {
        #[derive(Debug, serde::Deserialize)]
        struct Current {
            #[serde(rename = "version")]
            _version: Version<2>,
            #[serde(rename = "value")]
            _value: u64,
        }
        for bytes in [
            br#"{"value":"not-current","version":2,"version":2}"#.as_slice(),
            br#"{"value":"not-current","version":1}"#,
            br#"{"value":"not-current"}"#,
            br#"{"value":"not-current","version":"secret"}"#,
        ] {
            let error = super::decode_versioned::<Current, 2>(bytes).unwrap_err();
            assert_eq!(error.location().stage, super::DecodeStage::Header);
            assert!(!error.to_string().contains("not-current"));
            assert!(!error.to_string().contains("secret"));
        }
    }

    #[test]
    fn structural_custom_and_untagged_paths_report_only_the_observed_prefix() {
        fn custom<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
            let _: Vec<u64> = serde::Deserialize::deserialize(deserializer)?;
            Err(serde::de::Error::custom("hidden custom constraint"))
        }
        #[derive(Debug, serde::Deserialize)]
        struct Custom {
            #[serde(rename = "values", deserialize_with = "custom")]
            _values: u64,
        }
        let error = super::decode_json::<Custom>(br#"{"values":[1,2]}"#).unwrap_err();
        assert_eq!(
            error.location().path,
            vec![super::DecodePathSegment::Map {
                key: "values".into()
            }]
        );
        assert_eq!(
            error.location().path_status,
            super::DecodePathStatus::TrackedPrefix
        );
        #[derive(Debug, serde::Deserialize)]
        #[serde(untagged)]
        enum Alternative {
            Number {
                #[serde(rename = "number")]
                _number: u64,
            },
            Flag {
                #[serde(rename = "flag")]
                _flag: bool,
            },
        }
        let error = super::decode_json::<Alternative>(br#"{"number":"secret"}"#).unwrap_err();
        assert!(error.location().path.is_empty());
        assert_eq!(
            error.location().path_status,
            super::DecodePathStatus::Unavailable
        );
    }
}
