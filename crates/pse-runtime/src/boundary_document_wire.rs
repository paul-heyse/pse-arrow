// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Standalone document envelopes; nested domain values retain their own encoding.

use std::marker::PhantomData;

use pse_model::document::Version;
use serde::de::{DeserializeSeed, Error as _, IntoDeserializer, MapAccess, Visitor};

/// Actual wire owner for an exported object whose domain body is unversioned.
///
/// The separate header preserves domain fields already named `version`, including
/// build versions and diagnostic format versions. The body is deliberately not a
/// second field declaration.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Versioned<T, const N: u32> {
    /// Required standalone encoding version.
    pub document_version: Version<N>,
    /// The actual domain owner's fields, at their existing names.
    #[serde(flatten)]
    pub body: T,
}

impl<T: schemars::JsonSchema, const N: u32> schemars::JsonSchema for Versioned<T, N> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("Versioned{}V{N}", T::schema_name()).into()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        format!(
            "pse_runtime::boundary_documents::Versioned<{}, {N}>",
            T::schema_id()
        )
        .into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // The pinned schemars flatten derive intentionally removes the inner
        // unknown-field policy (_private::json_schema_for_flatten, 1.2.2). This
        // wire owner's streaming deserializer preserves that policy instead.
        let mut schema = T::json_schema(generator);
        let header = <Version<N> as schemars::JsonSchema>::json_schema(generator).to_value();
        add_header(
            schema.ensure_object(),
            &header,
            false,
            generator.definitions(),
            &mut std::collections::BTreeSet::new(),
        );
        schema
    }
}

fn add_header(
    object: &mut serde_json::Map<String, serde_json::Value>,
    header: &serde_json::Value,
    alternative: bool,
    definitions: &serde_json::Map<String, serde_json::Value>,
    active: &mut std::collections::BTreeSet<String>,
) {
    // A composition alternative may reference a strict object. Inline that actual
    // schema only here, so its own closure sees the new envelope header. Nested
    // properties keep their unchanged domain-body references.
    if let Some(reference) = object
        .get("$ref")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.strip_prefix("#/$defs/"))
        .map(|value| value.replace("~1", "/").replace("~0", "~"))
        && active.insert(reference.clone())
    {
        if let Some(serde_json::Value::Object(referenced)) = definitions.get(&reference) {
            let siblings = std::mem::replace(object, referenced.clone());
            object.extend(siblings.into_iter().filter(|(key, _)| key != "$ref"));
        }
        // Recursive properties are intentionally not followed; composition cycles
        // remain explicit references for the generator's closed schema admission.
        add_header_contents(object, header, alternative, definitions, active);
        active.remove(&reference);
        return;
    }
    add_header_contents(object, header, alternative, definitions, active);
}
fn add_header_contents(
    object: &mut serde_json::Map<String, serde_json::Value>,
    header: &serde_json::Value,
    alternative: bool,
    definitions: &serde_json::Map<String, serde_json::Value>,
    active: &mut std::collections::BTreeSet<String>,
) {
    if alternative && let Some(serde_json::Value::String(title)) = object.get_mut("title") {
        title.push_str("Document");
    }
    for keyword in ["oneOf", "anyOf", "allOf"] {
        if let Some(serde_json::Value::Array(variants)) = object.get_mut(keyword) {
            for variant in variants {
                if let Some(variant) = variant.as_object_mut() {
                    add_header(variant, header, true, definitions, active);
                }
            }
        }
    }
    let properties = object
        .entry("properties")
        .or_insert_with(|| serde_json::json!({}));
    if let Some(properties) = properties.as_object_mut() {
        properties.insert("document_version".into(), header.clone());
    }
    let required = object
        .entry("required")
        .or_insert_with(|| serde_json::json!([]));
    if let Some(required) = required.as_array_mut() {
        required.push("document_version".into());
    }
}

impl<T, const N: u32> Versioned<T, N> {
    /// Wrap a domain owner without copying its payload.
    pub fn new(body: T) -> Self {
        Self {
            document_version: Version,
            body,
        }
    }
}

// Serde's derived flatten deserializer does not support deny_unknown_fields on
// the flattened body (https://serde.rs/attr-flatten.html). Forward the map into
// the owner's actual deserializer instead. No Value map can erase duplicate keys.
struct BodyAccess<'a, A, const N: u32> {
    map: &'a mut A,
    header: bool,
}

impl<'de, A: MapAccess<'de>, const N: u32> MapAccess<'de> for &mut BodyAccess<'_, A, N> {
    type Error = A::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        while let Some(key) = self.map.next_key::<String>()? {
            if key == "document_version" {
                if self.header {
                    return Err(A::Error::duplicate_field("document_version"));
                }
                self.header = true;
                let _: Version<N> = self.map.next_value()?;
            } else {
                return seed.deserialize(key.into_deserializer()).map(Some);
            }
        }
        Ok(None)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        self.map.next_value_seed(seed)
    }
}

impl<'de, T: serde::Deserialize<'de>, const N: u32> serde::Deserialize<'de> for Versioned<T, N> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T, const N: u32>(PhantomData<T>);
        impl<'de, T: serde::Deserialize<'de>, const N: u32> Visitor<'de> for ObjectVisitor<T, N> {
            type Value = Versioned<T, N>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a standalone object document at version {N}")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut body = BodyAccess::<_, N> {
                    map: &mut map,
                    header: false,
                };
                let value =
                    T::deserialize(serde::de::value::MapAccessDeserializer::new(&mut body))?;
                if (&mut body).next_key::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(A::Error::custom("document body left unread fields"));
                }
                if !body.header {
                    return Err(A::Error::missing_field("document_version"));
                }
                Ok(Versioned::new(value))
            }
        }
        deserializer.deserialize_map(ObjectVisitor::<T, N>(PhantomData))
    }
}

/// Actual wire owner for a non-object exported value, such as a unit enum.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VersionedValue<T, const N: u32> {
    /// Required standalone encoding version.
    pub document_version: Version<N>,
    /// The actual scalar or collection owner, retaining its original encoding.
    pub value: T,
}

impl<T, const N: u32> VersionedValue<T, N> {
    /// Wrap a scalar owner without altering its meaning.
    pub fn new(value: T) -> Self {
        Self {
            document_version: Version,
            value,
        }
    }
}

/// Check the required current envelope header before interpreting the domain body.
///
/// # Errors
/// A missing, duplicate, malformed or unsupported header is refused.
pub fn check_header<const N: u32>(
    bytes: &[u8],
) -> Result<(), pse_model::document::JsonDecodeError> {
    check_required_header(bytes, "document_version", N)
}

/// Validate one actual owner-declared required header directly on the original bytes.
/// Duplicate keys are refused before any body or Python value conversion.
///
/// # Errors
/// The document is malformed or its required interpretation is missing/unsupported.
pub fn check_required_header(
    bytes: &[u8],
    field: &'static str,
    expected: u32,
) -> Result<(), pse_model::document::JsonDecodeError> {
    pse_model::document::check_required_header(bytes, field, expected)
}

/// The actual standalone encoding selected once by the boundary catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Wire {
    /// The unchanged body encoding, including any owner-declared `version` field.
    Native,
    /// An object body exported through its version-one flattened wire owner.
    FlatV1,
    /// A scalar body exported through its version-one value wire owner.
    ValueV1,
}

impl Wire {
    /// Serialize through the selected actual wire owner, borrowing retained custody.
    ///
    /// # Errors
    /// The actual body serializer refuses its encoding.
    pub fn to_vec<T: serde::Serialize + ?Sized>(
        self,
        value: &T,
    ) -> Result<Vec<u8>, serde_json::Error> {
        match self {
            Self::Native => serde_json::to_vec(value),
            Self::FlatV1 => serde_json::to_vec(&Versioned::<_, 1>::new(value)),
            Self::ValueV1 => serde_json::to_vec(&VersionedValue::<_, 1>::new(value)),
        }
    }

    /// Decode an envelope header before the unchanged actual domain body.
    /// Native owners retain their existing version-first entry points.
    ///
    /// # Errors
    /// A required header or the body's actual serde contract refuses the value.
    pub fn decode_slice<T: serde::de::DeserializeOwned>(
        self,
        bytes: &[u8],
    ) -> Result<T, pse_model::document::JsonDecodeError> {
        match self {
            Self::Native => pse_model::document::decode_json(bytes),
            Self::FlatV1 => {
                check_header::<1>(bytes)?;
                pse_model::document::decode_json::<Versioned<T, 1>>(bytes)
                    .map(|document| document.body)
            }
            Self::ValueV1 => {
                check_header::<1>(bytes)?;
                pse_model::document::decode_json::<VersionedValue<T, 1>>(bytes)
                    .map(|document| document.value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_required_headers_refuse_original_duplicate_keys_before_body() {
        for field in ["version", "document_version"] {
            for (first, second) in [("1", "1"), ("1", "2"), ("2", "1")] {
                let bytes = format!(
                    "{{\"{field}\":{first},\"body\":{{\"version\":4}},\"{field}\":{second}}}"
                );
                assert!(check_required_header(bytes.as_bytes(), field, 1).is_err());
            }
            assert!(
                check_required_header(
                    format!("{{\"{field}\":1,\"nested\":{{\"{field}\":2}}}}").as_bytes(),
                    field,
                    1
                )
                .is_ok()
            );
            assert!(
                check_required_header(
                    format!("{{\"nested\":{{\"{field}\":1}}}}").as_bytes(),
                    field,
                    1
                )
                .is_err()
            );
            for value in ["true", "1.0", "-1", "null", "\"1\""] {
                assert!(
                    check_required_header(format!("{{\"{field}\":{value}}}").as_bytes(), field, 1)
                        .is_err()
                );
            }
        }
    }

    #[test]
    fn standalone_wire_decode_reports_nested_paths_and_keeps_trailing_refusal() {
        use pse_model::document::{DecodeCategory, DecodePathSegment, DecodeStage};
        #[derive(Debug, serde::Deserialize)]
        struct Nested {
            #[serde(rename = "rows")]
            _rows: Vec<Strict>,
        }
        let bytes = br#"{"document_version":1,"rows":[{"value":"private-value"}]}"#;
        let error = Wire::FlatV1.decode_slice::<Nested>(bytes).unwrap_err();
        assert_eq!(error.location().stage, DecodeStage::Body);
        assert_eq!(error.location().category, DecodeCategory::Data);
        assert_eq!(
            error.location().path,
            vec![
                DecodePathSegment::Map { key: "rows".into() },
                DecodePathSegment::Sequence { index: 0 },
                DecodePathSegment::Map {
                    key: "value".into()
                },
            ]
        );
        assert!(!error.to_string().contains("private-value"));
        for (wire, bytes) in [
            (Wire::Native, br#"{"value":7} null"#.as_slice()),
            (
                Wire::FlatV1,
                br#"{"document_version":1,"value":7} null"#.as_slice(),
            ),
            (
                Wire::ValueV1,
                br#"{"document_version":1,"value":{"value":7}} null"#.as_slice(),
            ),
        ] {
            let error = wire.decode_slice::<Strict>(bytes).unwrap_err();
            assert_eq!(error.location().stage, DecodeStage::Trailing);
            assert_eq!(error.location().category, DecodeCategory::Syntax);
        }
    }

    #[test]
    fn standalone_registered_tagged_owner_preserves_schema_closure_and_nested_body() {
        use crate::boundary_documents::BoundaryDocument;
        use pse_operations::canonical_history::{HistoryObligation, HistoryOwner, HistoryPurpose};
        let value = HistoryOwner::Run("original-owner".into());
        let bytes = HistoryOwner::WIRE.to_vec(&value).unwrap();
        assert_eq!(
            HistoryOwner::WIRE
                .decode_slice::<HistoryOwner>(&bytes)
                .unwrap(),
            value
        );
        let roots = crate::boundary_documents::schemas();
        let root = roots
            .iter()
            .find(|root| {
                root.slug == "history-owner"
                    && root.direction == crate::boundary_documents::Direction::Input
            })
            .unwrap();
        let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let branch = root.schema["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|branch| branch["properties"]["kind"]["const"] == "run")
            .unwrap();
        assert_eq!(branch["additionalProperties"], false);
        for key in wire.as_object().unwrap().keys() {
            assert!(branch["properties"].get(key).is_some(), "{key}");
        }
        for key in branch["required"].as_array().unwrap() {
            assert!(wire.get(key.as_str().unwrap()).is_some());
        }
        let nested = HistoryObligation {
            key: "obligation".into(),
            owner: value,
            purpose: HistoryPurpose::Audit,
            attribution: "caller".into(),
        };
        let encoded: serde_json::Value =
            serde_json::from_slice(&HistoryObligation::WIRE.to_vec(&nested).unwrap()).unwrap();
        assert_eq!(encoded["document_version"], 1);
        assert!(encoded["owner"].get("document_version").is_none());
        assert_eq!(encoded["purpose"], "audit");
        assert!(root.version.is_some());
    }

    #[test]
    fn standalone_referenced_registered_alternative_retains_its_own_closure() {
        let mut schema =
            schemars::schema_for!(pse_operations::canonical_history::HistoryOwner).to_value();
        let actual = schema["oneOf"].as_array_mut().unwrap().remove(0);
        let definitions = serde_json::Map::from_iter([("ActualHistoryAlternative".into(), actual)]);
        let mut reference = serde_json::json!({"$ref":"#/$defs/ActualHistoryAlternative"})
            .as_object()
            .unwrap()
            .clone();
        let header = schemars::schema_for!(Version<1>).to_value();
        add_header(
            &mut reference,
            &header,
            true,
            &definitions,
            &mut std::collections::BTreeSet::new(),
        );
        assert!(reference.get("$ref").is_none());
        assert_eq!(reference["additionalProperties"], false);
        assert_eq!(reference["properties"]["document_version"]["const"], 1);
        assert!(
            reference["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == "document_version")
        );
        let expected = &definitions["ActualHistoryAlternative"]["properties"];
        for (key, value) in expected.as_object().unwrap() {
            assert_eq!(&reference["properties"][key], value);
        }
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    struct Strict {
        value: u64,
        #[serde(default)]
        optional: bool,
    }

    fn read<T: serde::de::DeserializeOwned>(
        bytes: &[u8],
    ) -> Result<T, pse_model::document::JsonDecodeError> {
        check_header::<1>(bytes)?;
        pse_model::document::decode_json(bytes)
    }

    #[test]
    fn standalone_wrapper_preserves_actual_strict_defaults_and_duplicate_refusal() {
        let value: Versioned<Strict, 1> = read(br#"{"value":7,"document_version":1}"#).unwrap();
        assert_eq!(
            value.body,
            Strict {
                value: 7,
                optional: false
            }
        );
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            serde_json::json!({"document_version":1,"value":7,"optional":false})
        );
        let body = Strict {
            value: 7,
            optional: false,
        };
        let bytes = Wire::FlatV1.to_vec(&body).unwrap();
        assert_eq!(Wire::FlatV1.decode_slice::<Strict>(&bytes).unwrap(), body);
        assert_eq!(
            Wire::Native.to_vec(&body).unwrap(),
            serde_json::to_vec(&body).unwrap()
        );
        for invalid in [
            br#"{"document_version":1,"value":7,"unknown":true}"#.as_slice(),
            br#"{"document_version":1,"value":7,"value":8}"#,
            br#"{"document_version":1,"document_version":1,"value":7}"#,
            br#"{"value":7}"#,
        ] {
            assert!(read::<Versioned<Strict, 1>>(invalid).is_err());
        }
    }

    #[test]
    fn standalone_wrapper_checks_header_before_body_and_preserves_version_named_members() {
        let error =
            read::<Versioned<Strict, 1>>(br#"{"value":"historical shape","document_version":2}"#)
                .unwrap_err();
        assert!(
            error
                .inner()
                .to_string()
                .contains("unsupported document_version 2")
        );
        assert_eq!(
            error.location().stage,
            pse_model::document::DecodeStage::Header
        );
        assert!(!error.to_string().contains("historical shape"));
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct Build {
            version: String,
        }
        let value: Versioned<Build, 1> =
            read(br#"{"version":"0.1.0","document_version":1}"#).unwrap();
        assert_eq!(value.body.version, "0.1.0");
    }

    #[test]
    fn standalone_wrapper_keeps_permissive_and_tagged_owner_contracts() {
        #[derive(Debug, serde::Deserialize)]
        struct Permissive {
            value: u64,
        }
        let value: Versioned<Permissive, 1> =
            read(br#"{"value":7,"unknown":true,"document_version":1}"#).unwrap();
        assert_eq!(value.body.value, 7);
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Tagged {
            Selected { value: u64 },
        }
        let value: Versioned<Tagged, 1> =
            read(br#"{"kind":"selected","document_version":1,"value":7}"#).unwrap();
        assert_eq!(value.body, Tagged::Selected { value: 7 });
        assert!(
            read::<Versioned<Tagged, 1>>(
                br#"{"kind":"selected","document_version":1,"value":7,"unknown":true}"#
            )
            .is_err()
        );
    }

    #[test]
    fn standalone_wrapper_schema_and_scalar_share_the_actual_wire_owner() {
        let schema = schemars::schema_for!(Versioned<Strict, 1>).to_value();
        assert_eq!(schema["properties"]["document_version"]["const"], 1);
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == "document_version")
        );
        assert!(schema["properties"].get("body").is_none());
        assert_eq!(schema["additionalProperties"], false);
        let scalar =
            VersionedValue::<_, 1>::new(pse_operations::canonical_history::HistoryPurpose::Audit);
        assert_eq!(
            serde_json::to_value(&scalar).unwrap(),
            serde_json::json!({"document_version":1,"value":"audit"})
        );
        let bytes = Wire::ValueV1.to_vec(&scalar.value).unwrap();
        assert_eq!(
            Wire::ValueV1
                .decode_slice::<pse_operations::canonical_history::HistoryPurpose>(&bytes)
                .unwrap(),
            scalar.value
        );
        assert!(
            read::<VersionedValue<pse_operations::canonical_history::HistoryPurpose, 1>>(
                br#"{"document_version":1,"value":"audit","extra":0}"#
            )
            .is_err()
        );
    }

    #[test]
    fn standalone_wrapper_preserves_nested_bodies_and_tagged_schema_closure() {
        #[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct Parent {
            nested: Strict,
        }
        let document = Versioned::<_, 1>::new(Parent {
            nested: Strict {
                value: 9,
                optional: false,
            },
        });
        let bytes = serde_json::to_vec(&document).unwrap();
        let encoded: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            encoded["nested"],
            serde_json::json!({"value":9,"optional":false})
        );
        let decoded: Versioned<Parent, 1> = read(&bytes).unwrap();
        assert_eq!(decoded.body.nested.value, 9);
        assert!(
            serde_json::from_slice::<Versioned<Strict, 1>>(br#"{"document_version":2,"value":9}"#)
                .is_err()
        );
        #[derive(serde::Serialize, schemars::JsonSchema)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Choice {
            First { value: u64 },
            Second {},
        }
        for choice in [Choice::First { value: 3 }, Choice::Second {}] {
            let value = serde_json::to_value(Versioned::<_, 1>::new(choice)).unwrap();
            assert_eq!(value["document_version"], 1);
        }
        let schema = schemars::schema_for!(Versioned<Choice, 1>).to_value();
        assert_eq!(schema["properties"]["document_version"]["const"], 1);
        for variant in schema["oneOf"].as_array().unwrap() {
            assert_eq!(variant["additionalProperties"], false);
            assert_eq!(variant["properties"]["document_version"]["const"], 1);
            assert!(
                variant["required"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|name| name == "document_version")
            );
        }
    }
}
