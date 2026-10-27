// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared deployment identity and narrow typed configuration projections.

use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Maximum encoded deployment configuration, including immutable module source receipts.
pub const CONFIGURATION_FILE_BYTES: u64 = 256 * 1024;

/// Storage identity shared by connection and native execution consumers.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ServiceIdentity {
    /// Current service configuration interpretation.
    pub profile_version: Version<3>,
    /// Selected authenticated transport endpoint.
    pub endpoint: String,
    /// Selected canonical namespace.
    pub namespace: String,
    /// Exactly selected database.
    pub database: String,
    /// Enclosing storage service for an immutable receiver context.
    #[serde(default)]
    pub service_state: Option<PathBuf>,
    /// Immutable registered receiver generation, when selected.
    #[serde(default)]
    pub receiver_generation: Option<String>,
}

/// Connection-owned fields; native execution does not interpret these settings.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalConnectionSettings {
    /// Accepted scientific substrate interpretation.
    pub schema_interpretation: String,
    /// Exact transport contract envelope.
    pub max_message_bytes: usize,
    /// Private authentication record selected by this deployment.
    pub credentials_file: String,
}

/// Deserialize an actual owner's declared fields from a larger configuration.
///
/// Filtering occurs in the streaming serde map, so duplicate selected keys remain
/// visible to the strict owner. This never infers fields or constructs a second
/// declaration of the surrounding service configuration.
/// # Errors
/// Missing/duplicate selected fields, malformed selected values or trailing input.
pub fn projection<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, serde_json::Error> {
    let mut input = serde_json::Deserializer::from_slice(bytes);
    let value = T::deserialize(OwnedFields(&mut input))?;
    input.end()?;
    Ok(value)
}

struct OwnedFields<D>(D);
impl<'de, D: serde::Deserializer<'de>> serde::Deserializer<'de> for OwnedFields<D> {
    type Error = D::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_any(visitor)
    }
    fn deserialize_struct<V: serde::de::Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_map(ProjectionVisitor { fields, visitor })
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map enum identifier ignored_any
    }
}
struct ProjectionVisitor<V> {
    fields: &'static [&'static str],
    visitor: V,
}
impl<'de, V: serde::de::Visitor<'de>> serde::de::Visitor<'de> for ProjectionVisitor<V> {
    type Value = V::Value;
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.visitor.expecting(formatter)
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        self.visitor.visit_map(SelectedMap {
            fields: self.fields,
            map,
        })
    }
}
struct SelectedMap<A> {
    fields: &'static [&'static str],
    map: A,
}
impl<'de, A: serde::de::MapAccess<'de>> serde::de::MapAccess<'de> for SelectedMap<A> {
    type Error = A::Error;
    fn next_key_seed<K: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        while let Some(key) = self.map.next_key::<String>()? {
            if self.fields.contains(&key.as_str()) {
                return seed
                    .deserialize(serde::de::value::StringDeserializer::<Self::Error>::new(
                        key,
                    ))
                    .map(Some);
            }
            self.map.next_value::<serde::de::IgnoredAny>()?;
        }
        Ok(None)
    }
    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        self.map.next_value_seed(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projections_share_one_identity_without_erasing_duplicate_fields() {
        let source = br#"{"profile_version":3,"endpoint":"grpc://127.0.0.1:8000","namespace":"canonical","database":"selected","resources":{"other":"owner"},"schema_interpretation":"v1","max_message_bytes":1024,"credentials_file":"credentials.json"}"#;
        let identity: ServiceIdentity = projection(source).unwrap();
        let connection: CanonicalConnectionSettings = projection(source).unwrap();
        assert_eq!(identity.database, "selected");
        assert_eq!(connection.max_message_bytes, 1024);
        assert!(projection::<ServiceIdentity>(br#"{"profile_version":3,"endpoint":"one","endpoint":"two","namespace":"canonical","database":"selected"}"#).is_err());
        assert!(projection::<ServiceIdentity>(br#"{"profile_version":2,"endpoint":"one","namespace":"canonical","database":"selected"}"#).is_err());
    }
}

/// Canonical initialization artifact capability; description does not establish effects.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CanonicalInitializationPhase {
    /// The artifact describes its supported operation only.
    Description,
    /// Canonical schema initialization completed through the ordinary store owner.
    Initialized,
}
/// Typed initializer acknowledgment; no stdout phrase grants operation authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalInitializationReport {
    /// Initializer protocol interpretation.
    pub version: Version<1>,
    /// Actual supported initialization operation.
    pub canonical_initialize: bool,
    /// Description and completed initialization have distinct meanings.
    pub phase: CanonicalInitializationPhase,
}

/// Durable service-owned backup phase; completion never reopens application admission.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BackupPhase {
    /// Copy the exact stopped metadata image into reserved empty coordinates.
    Metadata,
    /// Reverify/copy the original frozen physical payload inventory.
    Payload,
    /// The exact complete backup manifest was published.
    Published,
}

/// Existing lifecycle/configuration owner's explicit backup-resume binding.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BackupOperation {
    /// Interpretation of this standalone pending owner.
    pub version: Version<1>,
    /// Never-reused operation identity deriving its private retained snapshot directory.
    pub nonce: String,
    /// Original explicitly selected destination.
    pub destination: PathBuf,
    /// Original destination filesystem device.
    pub destination_device: u64,
    /// Original destination directory inode, never adopted by path alone.
    pub destination_inode: u64,
    /// Exact source service incarnation.
    pub source_instance_id: String,
    /// Semantic configuration image before adding this pending owner.
    pub source_configuration_sha256: String,
    /// Exact immutable draft backup manifest, including the original frozen inventories.
    pub manifest_sha256: String,
    /// Original native payload checkpoint, once that physical phase was selected.
    pub payload_checkpoint: Option<PathBuf>,
    /// Only this original pending owner can advance the backup.
    pub phase: BackupPhase,
}
