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
        format!("pse_runtime::boundary_documents::Versioned<{}, {N}>", T::schema_id()).into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // The pinned schemars flatten derive intentionally removes the inner
        // unknown-field policy (_private::json_schema_for_flatten, 1.2.2). This
        // wire owner's streaming deserializer preserves that policy instead.
        let mut schema = T::json_schema(generator);
        let header = <Version<N> as schemars::JsonSchema>::json_schema(generator).to_value();
        add_header(schema.ensure_object(), &header, false);
        schema
    }
}

fn add_header(object: &mut serde_json::Map<String, serde_json::Value>, header: &serde_json::Value, alternative: bool) {
    if alternative && let Some(serde_json::Value::String(title)) = object.get_mut("title") {
        title.push_str("Document");
    }
    for keyword in ["oneOf", "anyOf", "allOf"] {
        if let Some(serde_json::Value::Array(variants)) = object.get_mut(keyword) {
            for variant in variants {
                if let Some(variant) = variant.as_object_mut() {
                    add_header(variant, header, true);
                }
            }
        }
    }
    let properties = object.entry("properties").or_insert_with(|| serde_json::json!({}));
    if let Some(properties) = properties.as_object_mut() {
        properties.insert("document_version".into(), header.clone());
    }
    let required = object.entry("required").or_insert_with(|| serde_json::json!([]));
    if let Some(required) = required.as_array_mut() {
        required.push("document_version".into());
    }
}

impl<T, const N: u32> Versioned<T, N> {
    /// Wrap a domain owner without copying its payload.
    pub fn new(body: T) -> Self {
        Self { document_version: Version, body }
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

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> {
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

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
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
                let mut body = BodyAccess::<_, N> { map: &mut map, header: false };
                let value = T::deserialize(serde::de::value::MapAccessDeserializer::new(&mut body))?;
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
        Self { document_version: Version, value }
    }
}

/// Check the required current envelope header before interpreting the domain body.
///
/// # Errors
/// A missing, duplicate, malformed or unsupported header is refused.
pub fn check_header<const N: u32>(bytes: &[u8]) -> Result<(), serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct Header<const N: u32> {
        #[serde(rename = "document_version")]
        _version: Version<N>,
    }
    serde_json::from_slice::<Header<N>>(bytes).map(|_| ())
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
    pub fn to_vec<T: serde::Serialize + ?Sized>(self, value: &T) -> Result<Vec<u8>, serde_json::Error> {
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
    pub fn from_slice<T: serde::de::DeserializeOwned>(self, bytes: &[u8]) -> Result<T, serde_json::Error> {
        match self {
            Self::Native => serde_json::from_slice(bytes),
            Self::FlatV1 => {
                check_header::<1>(bytes)?;
                serde_json::from_slice::<Versioned<T, 1>>(bytes).map(|document| document.body)
            }
            Self::ValueV1 => {
                check_header::<1>(bytes)?;
                serde_json::from_slice::<VersionedValue<T, 1>>(bytes).map(|document| document.value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    struct Strict {
        value: u64,
        #[serde(default)]
        optional: bool,
    }

    fn read<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, serde_json::Error> {
        check_header::<1>(bytes)?;
        serde_json::from_slice(bytes)
    }

    #[test]
    fn standalone_wrapper_preserves_actual_strict_defaults_and_duplicate_refusal() {
        let value: Versioned<Strict, 1> = read(br#"{"value":7,"document_version":1}"#).unwrap();
        assert_eq!(value.body, Strict { value: 7, optional: false });
        assert_eq!(serde_json::to_value(value).unwrap(), serde_json::json!({"document_version":1,"value":7,"optional":false}));
        let body = Strict { value: 7, optional: false };
        let bytes = Wire::FlatV1.to_vec(&body).unwrap();
        assert_eq!(Wire::FlatV1.from_slice::<Strict>(&bytes).unwrap(), body);
        assert_eq!(Wire::Native.to_vec(&body).unwrap(), serde_json::to_vec(&body).unwrap());
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
        let error = read::<Versioned<Strict, 1>>(br#"{"value":"historical shape","document_version":2}"#).unwrap_err();
        assert!(error.to_string().contains("unknown document version 2"));
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct Build { version: String }
        let value: Versioned<Build, 1> = read(br#"{"version":"0.1.0","document_version":1}"#).unwrap();
        assert_eq!(value.body.version, "0.1.0");
    }

    #[test]
    fn standalone_wrapper_keeps_permissive_and_tagged_owner_contracts() {
        #[derive(Debug, serde::Deserialize)]
        struct Permissive { value: u64 }
        let value: Versioned<Permissive, 1> = read(br#"{"value":7,"unknown":true,"document_version":1}"#).unwrap();
        assert_eq!(value.body.value, 7);
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Tagged { Selected { value: u64 } }
        let value: Versioned<Tagged, 1> = read(br#"{"kind":"selected","document_version":1,"value":7}"#).unwrap();
        assert_eq!(value.body, Tagged::Selected { value: 7 });
        assert!(read::<Versioned<Tagged, 1>>(br#"{"kind":"selected","document_version":1,"value":7,"unknown":true}"#).is_err());
    }

    #[test]
    fn standalone_wrapper_schema_and_scalar_share_the_actual_wire_owner() {
        let schema = schemars::schema_for!(Versioned<Strict, 1>).to_value();
        assert_eq!(schema["properties"]["document_version"]["const"], 1);
        assert!(schema["required"].as_array().unwrap().iter().any(|field| field == "document_version"));
        assert!(schema["properties"].get("body").is_none());
        assert_eq!(schema["additionalProperties"], false);
        let scalar = VersionedValue::<_, 1>::new(pse_operations::canonical_history::HistoryPurpose::Audit);
        assert_eq!(serde_json::to_value(&scalar).unwrap(), serde_json::json!({"document_version":1,"value":"audit"}));
        let bytes = Wire::ValueV1.to_vec(&scalar.value).unwrap();
        assert_eq!(Wire::ValueV1.from_slice::<pse_operations::canonical_history::HistoryPurpose>(&bytes).unwrap(), scalar.value);
        assert!(read::<VersionedValue<pse_operations::canonical_history::HistoryPurpose, 1>>(br#"{"document_version":1,"value":"audit","extra":0}"#).is_err());
    }

    #[test]
    fn standalone_wrapper_preserves_nested_bodies_and_tagged_schema_closure() {
        #[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct Parent { nested: Strict }
        let document = Versioned::<_, 1>::new(Parent { nested: Strict { value: 9, optional: false } });
        let bytes = serde_json::to_vec(&document).unwrap();
        let encoded: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(encoded["nested"], serde_json::json!({"value":9,"optional":false}));
        let decoded: Versioned<Parent, 1> = read(&bytes).unwrap();
        assert_eq!(decoded.body.nested.value, 9);
        assert!(serde_json::from_slice::<Versioned<Strict, 1>>(br#"{"document_version":2,"value":9}"#).is_err());
        #[derive(serde::Serialize, schemars::JsonSchema)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Choice { First { value: u64 }, Second {} }
        for choice in [Choice::First { value: 3 }, Choice::Second {}] {
            let value = serde_json::to_value(Versioned::<_, 1>::new(choice)).unwrap();
            assert_eq!(value["document_version"], 1);
        }
        let schema = schemars::schema_for!(Versioned<Choice, 1>).to_value();
        assert_eq!(schema["properties"]["document_version"]["const"], 1);
        for variant in schema["oneOf"].as_array().unwrap() {
            assert_eq!(variant["additionalProperties"], false);
            assert_eq!(variant["properties"]["document_version"]["const"], 1);
            assert!(variant["required"].as_array().unwrap().iter().any(|name| name == "document_version"));
        }
    }
}
