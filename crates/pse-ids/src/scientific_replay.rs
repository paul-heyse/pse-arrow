// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opaque authority for replaying an immutable, admitted scientific description.
//! Serialization of scientific DTOs never confers this authority.
use crate::{ContentHash, Frame, FramedHasher, IdError};
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeSet, sync::Arc};

/// The compiler-owned admitted description wire interpretation.
pub const ADMITTED_RECIPE_INTERPRETATION: &str = "pse.admitted-body.v2";
/// Logical admitted-description bound, independent of storage RPC block sizes.
/// Consumers reserve actual decode/index scratch before parsing; this is not a memory allowance.
pub const MAX_ADMITTED_RECIPE_BYTES: usize = 64 * 1024 * 1024;
/// Exact framed identity of the compiler-owned admitted description.
pub fn admitted_recipe_payload_hash(payload: &[u8]) -> ContentHash {
    let mut hash = FramedHasher::new(Frame::MathAdmittedRecipeV1);
    hash.str(ADMITTED_RECIPE_INTERPRETATION).part(payload);
    hash.finish_hash()
}

/// A qualified compiler-issued recipe. It cannot be cloned or deserialized.
///
/// ```compile_fail
/// let _: pse_ids::scientific_replay::QualifiedScientificRecipe =
///     serde_json::from_slice(b"{}").unwrap();
/// ```
/// ```compile_fail
/// fn duplicate(permit: &pse_ids::scientific_replay::QualifiedScientificRecipe)
///     -> pse_ids::scientific_replay::QualifiedScientificRecipe { permit.clone() }
/// ```
#[derive(Debug)]
pub struct QualifiedScientificRecipe {
    payload: Vec<u8>,
    qualification: ContentHash,
    request: ContentHash,
    records: RecordAuthority,
}
impl QualifiedScientificRecipe {
    /// Conservative bound for simultaneous JSON, structural index and DTO scratch.
    /// The storage consumer reserves this extent before calling the constructor.
    pub fn scratch_bytes(payload_bytes: usize) -> Result<usize, IdError> {
        if payload_bytes > MAX_ADMITTED_RECIPE_BYTES {
            return Err(IdError::ScientificRecipeLimit);
        }
        Ok(payload_bytes.saturating_mul(256).saturating_add(32 * 1024))
    }
    /// Cross the controlled storage-to-science authority boundary.
    ///
    /// # Safety
    /// These exact bytes must be a compiler-issued immutable description produced
    /// by successful scientific admission. The caller must have retrieved it through
    /// the controlled canonical store with its complete selected dependencies,
    /// current eligible producer and interpretation qualified for this exact request.
    /// `qualification` identifies that immutable qualified product, not a caller's
    /// invented hash. Storage publisher roles must prevent arbitrary DTO publication
    /// from impersonating scientific admission. Reserve `scratch_bytes` before parsing.
    /// A digest supplied by an arbitrary consumer is insufficient to establish this.
    ///
    /// ```compile_fail
    /// use pse_ids::{ContentHash, scientific_replay::QualifiedScientificRecipe};
    /// let id = ContentHash::from_bytes([0; 32]);
    /// let _ = QualifiedScientificRecipe::from_canonical_product(Vec::new(), id, id, id);
    /// ```
    ///
    /// # Errors
    /// Oversized, malformed or digest-mismatching descriptors are refused.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 audited storage-to-science authority mint; no unsafe memory operation"
    )]
    pub unsafe fn from_canonical_product(
        payload: Vec<u8>,
        expected_digest: ContentHash,
        qualification: ContentHash,
        request: ContentHash,
    ) -> Result<Self, IdError> {
        Self::scratch_bytes(payload.len())?;
        if admitted_recipe_payload_hash(&payload) != expected_digest {
            return Err(IdError::ScientificRecipeDigest);
        }
        let json: Value =
            serde_json::from_slice(&payload).map_err(|_| IdError::ScientificRecipeEncoding)?;
        let mut scope = FramedHasher::new(Frame::MathReplayAuthorityV1);
        scope
            .str("qualified-recipe")
            .hash(&expected_digest)
            .hash(&qualification)
            .hash(&request);
        let binding = scope.finish_hash();
        let mut nodes = BTreeSet::new();
        structural_hash(&json, binding, Some(&mut nodes));
        Ok(Self {
            payload,
            qualification,
            request,
            records: RecordAuthority {
                nodes: Arc::new(nodes),
                qualification,
                request,
                binding,
            },
        })
    }
    /// The exact authenticated compiler descriptor bytes.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    /// Store-owned immutable product identity.
    pub fn qualification(&self) -> ContentHash {
        self.qualification
    }
    /// Exact qualified selected request identity.
    pub fn request(&self) -> ContentHash {
        self.request
    }
    /// Restricted authority for records actually present in this recipe.
    pub fn records(&self) -> &RecordAuthority {
        &self.records
    }
}

/// Restricted child authority; copying it cannot authorize another descriptor.
#[derive(Clone, Debug)]
pub struct RecordAuthority {
    nodes: Arc<BTreeSet<ContentHash>>,
    qualification: ContentHash,
    request: ContentHash,
    binding: ContentHash,
}
impl RecordAuthority {
    /// Whether the complete serialized owning DTO is present in the qualified recipe.
    /// Object-key order does not affect membership; array order and values do.
    pub fn admits<T: Serialize>(&self, record: &T) -> Result<bool, IdError> {
        let value = serde_json::to_value(record).map_err(|_| IdError::ScientificRecipeEncoding)?;
        Ok(self
            .nodes
            .contains(&structural_hash(&value, self.binding, None)))
    }
    /// The immutable qualification of all authorized records.
    pub fn qualification(&self) -> ContentHash {
        self.qualification
    }
    /// Exact selected request for which these records were qualified.
    pub fn request(&self) -> ContentHash {
        self.request
    }
}
// Postorder construction visits every node once. Type tags and child hashes frame
// JSON structure only; this is neither a new expression language nor a proof engine.
fn structural_hash(
    value: &Value,
    binding: ContentHash,
    mut nodes: Option<&mut BTreeSet<ContentHash>>,
) -> ContentHash {
    let mut h = FramedHasher::new(Frame::MathReplayAuthorityV1);
    h.str("record").hash(&binding);
    match value {
        Value::Null => {
            h.str("null");
        }
        Value::Bool(v) => {
            h.str("bool").bool(*v);
        }
        Value::Number(v) => {
            h.str("number").str(&v.to_string());
        }
        Value::String(v) => {
            h.str("string").str(v);
        }
        Value::Array(values) => {
            h.str("array").u64(values.len() as u64);
            for v in values {
                h.hash(&structural_hash(v, binding, nodes.as_deref_mut()));
            }
        }
        Value::Object(values) => {
            h.str("object").u64(values.len() as u64);
            let ordered = values.iter().collect::<std::collections::BTreeMap<_, _>>();
            for (key, v) in ordered {
                h.str(key)
                    .hash(&structural_hash(v, binding, nodes.as_deref_mut()));
            }
        }
    }
    let digest = h.finish_hash();
    if let Some(nodes) = nodes {
        nodes.insert(digest);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[allow(
        unsafe_code,
        reason = "controlled fixture simulates a qualified compiler/store boundary"
    )]
    fn scientific_replay_authority_binds_exact_records() {
        let bytes = br#"{"receipt":{"branches":[2],"request":"a"},"other":[]}"#.to_vec();
        // SAFETY: pure authority fixture explicitly supplies its complete trusted record.
        let permit = unsafe {
            QualifiedScientificRecipe::from_canonical_product(
                bytes.clone(),
                admitted_recipe_payload_hash(&bytes),
                ContentHash::from_bytes([1; 32]),
                ContentHash::from_bytes([2; 32]),
            )
        }
        .unwrap();
        assert!(
            permit
                .records()
                .admits(&serde_json::json!({"request":"a","branches":[2]}))
                .unwrap()
        );
        assert!(
            !permit
                .records()
                .admits(&serde_json::json!({"request":"a","branches":[3]}))
                .unwrap()
        );
        assert!(
            !permit
                .records()
                .admits(&serde_json::json!({"request":"b","branches":[2]}))
                .unwrap()
        );
        assert_eq!(permit.records().qualification(), permit.qualification());
        assert_eq!(permit.records().request(), permit.request());
        assert!(QualifiedScientificRecipe::scratch_bytes(MAX_ADMITTED_RECIPE_BYTES + 1).is_err());
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "controlled fixture exercises qualified large compiler-description authority"
    )]
    fn scientific_replay_large_logical_description_preserves_record_binding() {
        let record = serde_json::json!({"path": "x".repeat(3 * 1024 * 1024 + 1)});
        let payload = serde_json::to_vec(&record).unwrap();
        assert!(payload.len() > 3 * 1024 * 1024);
        assert_eq!(
            QualifiedScientificRecipe::scratch_bytes(payload.len()).unwrap(),
            payload.len() * 256 + 32 * 1024
        );
        // SAFETY: isolated trusted compiler/store fixture supplies the complete exact admitted record.
        let permit = unsafe {
            QualifiedScientificRecipe::from_canonical_product(
                payload.clone(),
                admitted_recipe_payload_hash(&payload),
                ContentHash::from_bytes([1; 32]),
                ContentHash::from_bytes([2; 32]),
            )
        }
        .unwrap();
        assert!(permit.records().admits(&record).unwrap());
        assert!(
            !permit
                .records()
                .admits(&serde_json::json!({"path":"x"}))
                .unwrap()
        );
        assert_eq!(
            QualifiedScientificRecipe::scratch_bytes(MAX_ADMITTED_RECIPE_BYTES).unwrap(),
            MAX_ADMITTED_RECIPE_BYTES * 256 + 32 * 1024
        );
        assert!(QualifiedScientificRecipe::scratch_bytes(MAX_ADMITTED_RECIPE_BYTES + 1).is_err());
    }
}
