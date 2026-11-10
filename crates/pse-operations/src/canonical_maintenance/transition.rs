// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Serializable original transition intent. An intent never grants maintenance authority.

use super::{CanonicalError, Marker, invalid, is_sha256, sha256};
use serde::{Deserialize, Serialize};

/// Exact selected treatment of existing data; no destructive default exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransitionDataPolicy {
    /// Replace explicitly disposable state from its admitted authoritative inputs.
    Reconstruct {
        /// Frozen reconstruction input identity.
        authoritative_inputs_sha256: String,
    },
    /// Preserve original meaning using one complete source-to-target mapping.
    Preserve {
        /// Frozen conversion/backfill profile; a scalar example is not a full mapping.
        conversion_profile_sha256: String,
    },
}

/// Original issuer, predecessor and effective inputs retained across uncertain replies.
/// Deserialization supplies no execution capability; revalidation against the live owner is required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SchemaTransitionIntent {
    pub(crate) operation: String,
    pub(crate) namespace: String,
    pub(crate) database: String,
    pub(crate) predecessor: Marker,
    pub(crate) source_epoch: String,
    pub(crate) installed_catalog_sha256: String,
    pub(crate) target_interpretation: String,
    pub(crate) target_schema_digest: String,
    pub(crate) target_module_sha256: String,
    pub(crate) manifest_sha256: String,
    pub(crate) tool_input_sha256: String,
    pub(crate) tool_binary_sha256: String,
    pub(crate) data_policy: TransitionDataPolicy,
}

impl SchemaTransitionIntent {
    pub(crate) fn checked(self) -> Result<Self, CanonicalError> {
        self.validate()?;
        Ok(self)
    }

    /// Stable original operation. Recovery must not mint another identity.
    pub fn operation(&self) -> &str {
        &self.operation
    }
    /// Original persistent database issuer.
    pub fn issuer(&self) -> &str {
        &self.predecessor.issuing_store
    }
    /// Exact observed epoch; static schema identity alone cannot bind maintenance.
    pub fn source_epoch(&self) -> &str {
        &self.source_epoch
    }
    /// Selected data treatment, not evidence that its effects have occurred.
    pub fn data_policy(&self) -> &TransitionDataPolicy {
        &self.data_policy
    }
    /// Exact frozen manifest identity.
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }
    /// Target policy artifact, distinct from the still-installed original artifact.
    pub fn target_module_sha256(&self) -> &str {
        &self.target_module_sha256
    }
    /// Validate a retained document without contacting or mutating a deployment.
    pub fn validate(&self) -> Result<(), CanonicalError> {
        let policy_hash = match &self.data_policy {
            TransitionDataPolicy::Reconstruct {
                authoritative_inputs_sha256,
            } => authoritative_inputs_sha256,
            TransitionDataPolicy::Preserve {
                conversion_profile_sha256,
            } => conversion_profile_sha256,
        };
        if [
            &self.operation,
            &self.source_epoch,
            &self.predecessor.issuing_store,
        ]
        .into_iter()
        .any(|value| {
            !uuid::Uuid::parse_str(value).is_ok_and(|id| !id.is_nil() && id.to_string() == *value)
        }) || [&self.namespace, &self.database].into_iter().any(|name| {
            name.is_empty()
                || name.len() > 128
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        }) || self.predecessor.interpretation.is_empty()
            || self.target_interpretation.is_empty()
            || [
                &self.installed_catalog_sha256,
                &self.predecessor.schema_digest,
                &self.predecessor.study_module_sha256,
                &self.target_schema_digest,
                &self.target_module_sha256,
                &self.manifest_sha256,
                &self.tool_input_sha256,
                &self.tool_binary_sha256,
                policy_hash,
            ]
            .into_iter()
            .any(|digest| !is_sha256(digest))
        {
            return Err(invalid(
                "transition intent differs from exact original inputs",
            ));
        }
        Ok(())
    }
    /// Immutable request fingerprint; not a data witness or an activation receipt.
    pub fn fingerprint(&self) -> Result<String, CanonicalError> {
        self.validate()?;
        let bytes =
            serde_json::to_vec(self).map_err(|_| invalid("transition intent encoding failed"))?;
        Ok(sha256(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn intent() -> SchemaTransitionIntent {
        SchemaTransitionIntent {
            operation: uuid::Uuid::new_v4().to_string(),
            namespace: "ns".into(),
            database: "db".into(),
            predecessor: Marker {
                interpretation: "source-v1".into(),
                schema_digest: sha256(b"source-schema"),
                study_module_sha256: sha256(b"source-module"),
                issuing_store: uuid::Uuid::new_v4().to_string(),
            },
            source_epoch: uuid::Uuid::new_v4().to_string(),
            installed_catalog_sha256: sha256(b"observed-catalog"),
            target_interpretation: "target-v2".into(),
            target_schema_digest: sha256(b"target-schema"),
            target_module_sha256: sha256(b"target-module"),
            manifest_sha256: sha256(b"effective-inputs"),
            tool_input_sha256: sha256(b"tool-inputs"),
            tool_binary_sha256: sha256(b"tool-binary"),
            data_policy: TransitionDataPolicy::Preserve {
                conversion_profile_sha256: sha256(b"complete-conversion-profile"),
            },
        }
        .checked()
        .unwrap()
    }
    #[test]
    fn transition_intent_roundtrip_binds_original_effects_and_artifacts() {
        let original = intent();
        let encoded = serde_json::to_vec(&original).unwrap();
        let retained: SchemaTransitionIntent = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(original, retained);
        assert_eq!(
            original.fingerprint().unwrap(),
            retained.fingerprint().unwrap()
        );
        let mut changed = retained.clone();
        changed.target_module_sha256 = sha256(b"substituted-module");
        assert_ne!(
            changed.fingerprint().unwrap(),
            original.fingerprint().unwrap()
        );
        changed = retained.clone();
        changed.manifest_sha256 = sha256(b"substituted-effective-inputs");
        assert_ne!(
            changed.fingerprint().unwrap(),
            original.fingerprint().unwrap()
        );
        changed = retained;
        changed.predecessor.issuing_store = uuid::Uuid::new_v4().to_string();
        assert_ne!(
            changed.fingerprint().unwrap(),
            original.fingerprint().unwrap()
        );
    }
    #[test]
    fn transition_intent_has_no_implicit_destructive_policy_or_nil_owner() {
        let original = intent();
        let mut document = serde_json::to_value(&original).unwrap();
        document.as_object_mut().unwrap().remove("data_policy");
        assert!(serde_json::from_value::<SchemaTransitionIntent>(document).is_err());
        let mut changed = original.clone();
        changed.operation = uuid::Uuid::nil().to_string();
        assert!(changed.validate().is_err());
        changed.operation = format!("urn:uuid:{}", original.operation);
        assert!(
            changed.validate().is_err(),
            "alternate UUID spellings must not create another original operation key"
        );
        changed = original;
        changed.data_policy = TransitionDataPolicy::Reconstruct {
            authoritative_inputs_sha256: "unfrozen".into(),
        };
        assert!(changed.validate().is_err());
    }
}
