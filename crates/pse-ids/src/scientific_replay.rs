// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opaque authority for replaying an immutable, admitted scientific description.
//! Serialization of scientific DTOs never confers this authority.
use crate::{ContentHash, Frame, FramedHasher, IdError};
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeSet, sync::Arc};

mod structural_v1;
pub use structural_v1::{
    Counts as NativeMembershipCounts, Error as NativeMembershipError,
    Limits as NativeMembershipLimits,
};

/// The compiler-owned admitted description wire interpretation.
pub const ADMITTED_RECIPE_INTERPRETATION: &str = "pse.admitted-body.v3";
/// Prerequisite binary interpretation; this does not activate a production reader.
pub const NATIVE_RECIPE_INTERPRETATION: &str = "pse.admitted-body.v4";
/// Logical admitted-description bound, independent of storage RPC block sizes.
/// Consumers reserve actual decode/index scratch before parsing; this is not a memory allowance.
pub const MAX_ADMITTED_RECIPE_BYTES: usize = 64 * 1024 * 1024;
/// Exact framed identity of the compiler-owned admitted description.
pub fn admitted_recipe_payload_hash(payload: &[u8]) -> ContentHash {
    let mut hash = FramedHasher::new(Frame::MathAdmittedRecipeV1);
    hash.str(ADMITTED_RECIPE_INTERPRETATION).part(payload);
    hash.finish_hash()
}

/// Exact identity of the future binary interpretation, distinct from active JSON v3.
/// Hashing bytes establishes no qualification, admission or record authority.
pub fn native_recipe_payload_hash(payload: &[u8]) -> ContentHash {
    let mut hash = FramedHasher::new(Frame::MathAdmittedRecipeV1);
    hash.str(NATIVE_RECIPE_INTERPRETATION).part(payload);
    hash.finish_hash()
}

/// Original audited receiving boundary, never evidence that origin was authenticated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeRecipeSource {
    /// Eligible canonical compiler product, qualified by the original runtime owner.
    CanonicalProduct,
    /// Isolated fixture record origin; never production qualification.
    #[cfg(any(test, feature = "fixtures"))]
    Fixture,
}
impl NativeRecipeSource {
    fn domain(self) -> &'static str {
        match self {
            Self::CanonicalProduct => "qualified-recipe",
            #[cfg(any(test, feature = "fixtures"))]
            Self::Fixture => "admitted-record-fixture",
        }
    }
}

/// Original qualification identities supplied by an audited receiving owner.
/// These data alone cannot mint authority.
#[derive(Clone, Copy, Debug)]
pub struct NativeRecipeScope {
    /// Selected receiving premise whose authentication remains the caller's duty.
    pub source: NativeRecipeSource,
    /// Verified exact immutable binary-payload digest.
    pub payload_digest: ContentHash,
    /// Original qualified product identity; isolated fixtures use a test-case label.
    pub qualification: ContentHash,
    /// Original selected request identity.
    pub request: ContentHash,
}

/// Count actual native serialization under an explicit normalization allowance.
/// Only the count/helper scratch is covered here; decoded roots and receiving work
/// need their own original admission. This returns data and no authority.
/// # Errors
/// Unsupported normalization, exhausted allowances or serialization failures refuse.
pub fn count_native_membership<T: ?Sized + Serialize>(
    root: &T,
    limits: NativeMembershipLimits,
    count_scratch: usize,
) -> Result<NativeMembershipCounts, IdError> {
    structural_v1::count(root, limits, count_scratch).map_err(Into::into)
}

impl NativeMembershipCounts {
    /// Simultaneous index construction plus its shared immutable backing header.
    /// Caller/root ownership, decoder, validator, native adoption, receipt copies,
    /// reconstruction, allocator-private overhead and RSS are outside this extent.
    /// # Errors
    /// Checked extent overflow refuses before allocation.
    pub fn authority_scratch_bytes(self) -> Result<usize, IdError> {
        self.index_scratch_bytes()?
            .checked_add(size_of::<NativeRecordIndex>() + 2 * size_of::<usize>())
            .ok_or(IdError::ScientificMembershipLimit {
                resource: "authority extent overflow",
            })
    }
}

/// A qualified compiler-issued recipe. It cannot be cloned or deserialized.
///
/// ```
/// let payload: serde_json::Value = serde_json::from_slice(b"{}").unwrap();
/// assert!(payload.is_object());
/// ```
///
/// ```compile_fail,E0277
/// let _: pse_ids::scientific_replay::QualifiedScientificRecipe =
///     serde_json::from_slice(b"{}").unwrap();
/// ```
/// ```
/// fn borrow(permit: &pse_ids::scientific_replay::QualifiedScientificRecipe)
///     -> &pse_ids::scientific_replay::QualifiedScientificRecipe { permit }
/// ```
///
/// ```compile_fail,E0308
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
    /// ```
    /// use pse_ids::{ContentHash, scientific_replay::QualifiedScientificRecipe};
    /// let _mint: unsafe fn(Vec<u8>, ContentHash, ContentHash, ContentHash) -> _ =
    ///     QualifiedScientificRecipe::from_canonical_product;
    /// ```
    ///
    /// ```compile_fail,E0133
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
        Self::qualified(
            payload,
            expected_digest,
            qualification,
            request,
            "qualified-recipe",
        )
    }
    /// Issue replay records for an isolated immutable scientific test fixture.
    ///
    /// # Safety
    /// The caller must own the complete fixture record population and keep its
    /// replay authority inside the isolated test. Authentic and deliberately corrupt
    /// fixture records may exercise admission/refusal; neither results nor scope
    /// labels qualify a production product. This constructor must never receive
    /// external product data or supply publication/readiness authority. Integrity
    /// hashing alone is not admission. Select only through test dependencies.
    ///
    /// # Errors
    /// Oversized, malformed or digest-mismatching descriptors are refused.
    #[cfg(any(test, feature = "fixtures"))]
    #[allow(
        unsafe_code,
        reason = "isolated scientific-record fixture authority; unavailable in the default product API"
    )]
    pub unsafe fn from_admitted_record_fixture(
        payload: Vec<u8>,
        expected_digest: ContentHash,
        fixture_case: ContentHash,
        request: ContentHash,
    ) -> Result<Self, IdError> {
        Self::qualified(
            payload,
            expected_digest,
            fixture_case,
            request,
            "admitted-record-fixture",
        )
    }
    fn qualified(
        payload: Vec<u8>,
        expected_digest: ContentHash,
        qualification: ContentHash,
        request: ContentHash,
        source: &str,
    ) -> Result<Self, IdError> {
        Self::scratch_bytes(payload.len())?;
        if admitted_recipe_payload_hash(&payload) != expected_digest {
            return Err(IdError::ScientificRecipeDigest);
        }
        let json: Value =
            serde_json::from_slice(&payload).map_err(|_| IdError::ScientificRecipeEncoding)?;
        let mut scope = FramedHasher::new(Frame::MathReplayAuthorityV1);
        scope
            .str(source)
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
                nodes: RecordIndex::Json(Arc::new(nodes)),
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
    /// Qualified immutable product or authenticated private execution identity.
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
    nodes: RecordIndex,
    qualification: ContentHash,
    request: ContentHash,
    binding: ContentHash,
}
#[derive(Clone, Debug)]
enum RecordIndex {
    Json(Arc<BTreeSet<ContentHash>>),
    Native(Arc<NativeRecordIndex>),
}
#[derive(Debug)]
struct NativeRecordIndex {
    root: ContentHash,
    nodes: Vec<ContentHash>,
    limits: NativeMembershipLimits,
    count_scratch: usize,
    query_scratch: usize,
}
impl RecordAuthority {
    /// Index one borrowed native root under its original authenticated scope.
    /// No caller-supplied index is accepted, and no JSON value tree is constructed.
    ///
    /// # Safety
    /// Only the audited monomorphic compiler canonical receiver may call
    /// this primitive. They must authenticate the original runtime provenance,
    /// qualification, request, physical context and eligible producer;
    /// verify the exact immutable bytes against `scope.payload_digest` under the
    /// binary interpretation; decode/adopt those same bytes once into a private
    /// native Recipe; index a borrow of that root here; then consume that SAME
    /// root for strict restoration with complete original receipt drain. They
    /// must not accept a caller-native-root or caller index. Arbitrary hashes,
    /// local validation and successful decoding cannot satisfy this premise.
    ///
    /// The same immutable, deterministic source must have produced `counts`.
    /// Admit count/index/query buffers before their allocations, and separately
    /// admit decode, complete validation, adoption, root and receipt ownership,
    /// restoration, cancellation and original operation/process settlement.
    /// These traversal extents do not establish that enclosing F03 admission.
    /// With the nondefault fixture feature, isolated tests may instead index their
    /// complete owned fixture under `NativeRecipeSource::Fixture`. Such records
    /// must never enter a product workflow or imply canonical qualification.
    ///
    /// # Errors
    /// Changed source shape, unsupported normalization and exhausted or failed
    /// preadmitted allocations refuse without publishing partial authority.
    #[allow(
        unsafe_code,
        reason = "ADR-0245 audited exact-native-root authority premise; no unsafe memory operation"
    )]
    pub unsafe fn from_authenticated_native_root<T: ?Sized + Serialize>(
        root: &T,
        scope: NativeRecipeScope,
        counts: NativeMembershipCounts,
        index_scratch: usize,
        query_scratch: usize,
    ) -> Result<Self, IdError> {
        if counts.authority_scratch_bytes()? > index_scratch {
            return Err(IdError::ScientificMembershipLimit {
                resource: "preadmitted authority bytes",
            });
        }
        let count_scratch = counts.limits.count_scratch_bytes()?;
        if count_scratch > query_scratch {
            return Err(IdError::ScientificMembershipLimit {
                resource: "preadmitted query normalization bytes",
            });
        }
        let mut binding = FramedHasher::new(Frame::MathReplayAuthorityV1);
        binding
            .str(scope.source.domain())
            .hash(&scope.payload_digest)
            .hash(&scope.qualification)
            .hash(&scope.request);
        let binding = binding.finish_hash();
        let index = structural_v1::build_index(root, binding, counts, index_scratch)?;
        Ok(Self {
            nodes: RecordIndex::Native(Arc::new(NativeRecordIndex {
                root: index.root,
                nodes: index.nodes,
                limits: counts.limits,
                count_scratch,
                query_scratch,
            })),
            qualification: scope.qualification,
            request: scope.request,
            binding,
        })
    }
    /// Whether the complete serialized owning DTO is present in the qualified recipe.
    /// Object-key order does not affect membership; array order and values do.
    pub fn admits<T: Serialize>(&self, record: &T) -> Result<bool, IdError> {
        match &self.nodes {
            RecordIndex::Json(nodes) => {
                let value =
                    serde_json::to_value(record).map_err(|_| IdError::ScientificRecipeEncoding)?;
                Ok(nodes.contains(&structural_hash(&value, self.binding, None)))
            }
            RecordIndex::Native(index) => {
                let counts = structural_v1::count(record, index.limits, index.count_scratch)?;
                let digest =
                    structural_v1::query_hash(record, self.binding, counts, index.query_scratch)?;
                Ok(digest == index.root || index.nodes.binary_search(&digest).is_ok())
            }
        }
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
    fn native_limits() -> NativeMembershipLimits {
        NativeMembershipLimits {
            nodes: 10_000,
            key_bytes: 1 << 20,
            helper_bytes: 4096,
            depth: 256,
        }
    }
    #[allow(
        unsafe_code,
        reason = "Isolated source-owned structural authority fixture, not a runtime receiving gate"
    )]
    fn native_fixture<T: Serialize>(
        value: &T,
        scope: NativeRecipeScope,
        query_bytes: Option<usize>,
    ) -> RecordAuthority {
        let limits = native_limits();
        let counts =
            count_native_membership(value, limits, limits.count_scratch_bytes().unwrap()).unwrap();
        // SAFETY: this private test owns the complete immutable source and scope.
        unsafe {
            RecordAuthority::from_authenticated_native_root(
                value,
                scope,
                counts,
                counts.authority_scratch_bytes().unwrap(),
                query_bytes.unwrap_or(counts.query_scratch_bytes().unwrap()),
            )
        }
        .unwrap()
    }
    fn native_scope() -> NativeRecipeScope {
        NativeRecipeScope {
            source: NativeRecipeSource::Fixture,
            payload_digest: ContentHash::from_bytes([11; 32]),
            qualification: ContentHash::from_bytes([12; 32]),
            request: ContentHash::from_bytes([13; 32]),
        }
    }
    #[test]
    fn native_membership_authority_retains_v1_queries_and_original_fixture_scope() {
        let value = serde_json::json!({"records": [{"a": [1,2]}], "empty": null});
        let original = native_scope();
        let baseline = native_fixture(&value, original, None);
        assert_ne!(
            NativeRecipeSource::Fixture.domain(),
            NativeRecipeSource::CanonicalProduct.domain()
        );
        assert!(baseline.admits(&value).unwrap());
        assert!(baseline.admits(&serde_json::json!({"a":[1,2]})).unwrap());
        assert!(!baseline.admits(&serde_json::json!({"a":[2,1]})).unwrap());
        assert!(
            !baseline
                .admits(&serde_json::json!({"foreign": 31337}))
                .unwrap()
        );
        assert_eq!(baseline.request(), original.request);
        assert_eq!(baseline.qualification(), original.qualification);
        for scope in [
            NativeRecipeScope {
                request: ContentHash::from_bytes([14; 32]),
                ..original
            },
            NativeRecipeScope {
                qualification: ContentHash::from_bytes([15; 32]),
                ..original
            },
            NativeRecipeScope {
                payload_digest: ContentHash::from_bytes([16; 32]),
                ..original
            },
        ] {
            let changed = native_fixture(&value, scope, None);
            assert_ne!(changed.binding, baseline.binding);
            let (RecordIndex::Native(left), RecordIndex::Native(right)) =
                (&baseline.nodes, &changed.nodes)
            else {
                panic!("native backends")
            };
            assert_ne!(left.root, right.root);
        }
        assert_ne!(
            native_recipe_payload_hash(b"same"),
            admitted_recipe_payload_hash(b"same")
        );
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "Isolated traversal preadmission refusal fixture"
    )]
    fn native_membership_budget_profile_and_query_failures_remain_typed() {
        let value = serde_json::json!({"a":[1,2,3],"b":{"c":false}});
        let limits = native_limits();
        assert!(matches!(
            count_native_membership(&value, limits, 0),
            Err(IdError::ScientificMembershipLimit { .. })
        ));
        let counts =
            count_native_membership(&value, limits, limits.count_scratch_bytes().unwrap()).unwrap();
        assert!(matches!(
            // SAFETY: isolated source fixture; too-small budget must refuse atomically.
            unsafe {
                RecordAuthority::from_authenticated_native_root(
                    &value,
                    native_scope(),
                    counts,
                    counts.authority_scratch_bytes().unwrap() - 1,
                    counts.query_scratch_bytes().unwrap(),
                )
            },
            Err(IdError::ScientificMembershipLimit { .. })
        ));
        let native = native_fixture(&value, native_scope(), None);
        assert!(matches!(
            native.admits(&i128::MAX),
            Err(IdError::ScientificMembershipProfile { .. })
        ));
        let bounded = native_fixture(
            &value,
            native_scope(),
            Some(limits.count_scratch_bytes().unwrap()),
        );
        assert!(matches!(
            bounded.admits(&value),
            Err(IdError::ScientificMembershipLimit { .. })
        ));
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "controlled fixture simulates a qualified compiler/store boundary"
    )]
    fn scientific_replay_authority_binds_exact_records() {
        let bytes = br#"{"receipt":{"branches":[2],"request":"a"},"other":[]}"#.to_vec();
        // SAFETY: pure authority fixture explicitly supplies its complete trusted record.
        let permit = unsafe {
            QualifiedScientificRecipe::from_admitted_record_fixture(
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
            QualifiedScientificRecipe::from_admitted_record_fixture(
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
