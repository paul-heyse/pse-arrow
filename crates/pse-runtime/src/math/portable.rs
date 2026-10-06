// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical, qualified admitted-body publication and strict numerical reconstruction.
//! A missing eligible product remains distinct from successful reconstruction.
use pse_compiler::{
    typed_math::{AdmittedBody, portable_payload_hash, reconstruct_portable},
    workspace::{CompilerContext, physical_identity},
};
use pse_ids::{ContentHash, Frame, FramedHasher, roles::SemanticBodyHash};
use pse_operations::{
    canonical::{CanonicalError, CanonicalStore},
    canonical_selection::SelectedRead,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::AtomicBool};

const INTERPRETATION: &str = "pse.runtime.admitted-body-product.v1";
/// Refusals preserve scientific reconstruction and storage failures distinctly.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum PortableError {
    /// Concrete canonical storage failure.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Store(#[from] CanonicalError),
    /// Incomplete or incompatible scientific construction receipts.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Math(#[from] pse_math::MathError),
    /// Deployment/product interpretation cannot establish persistent reuse.
    #[error("portable admitted product: {0}")]
    #[diagnostic(code(runtime::infrastructure))]
    Qualification(String),
}
impl pse_diagnostics::TypedDiagnostic for PortableError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        match self {
            Self::Store(error) => error.diagnostic_code(),
            Self::Math(error) => error.diagnostic_code(),
            Self::Qualification(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        }
    }
    fn diagnostic_facts(&self) -> pse_diagnostics::DiagnosticFacts {
        match self {
            Self::Store(error) => error.diagnostic_facts(),
            Self::Math(error) => error.diagnostic_facts(),
            Self::Qualification(reason) => {
                let mut facts = pse_diagnostics::DiagnosticFacts {
                    rule: Some(pse_diagnostics::DiagnosticRule::MathLibrary),
                    ..Default::default()
                };
                facts.observations.insert(
                    "portable_refusal".into(),
                    pse_diagnostics::DiagnosticObservation::Text(reason.clone()),
                );
                facts
            }
        }
    }
    fn diagnostic_children(
        &self,
    ) -> Option<Box<dyn Iterator<Item = &dyn pse_diagnostics::TypedDiagnostic> + '_>> {
        let child: &dyn pse_diagnostics::TypedDiagnostic = match self {
            Self::Store(error) => error,
            Self::Math(error) => error,
            Self::Qualification(_) => return None,
        };
        Some(Box::new(std::iter::once(child)))
    }
}
/// Relevant producer identity from an operator-owned, verified deployment receipt.
/// It is separate from a whole-build attestation and has no deserializer.
#[derive(Clone, Debug)]
pub struct QualifiedProducer {
    identity: ContentHash,
}
impl QualifiedProducer {
    /// Read the output of the production-unit identity tool at the deployment boundary.
    /// The operator owns the receipt's source/input qualification. Incomplete receipts
    /// return `None`, allowing normal execution without persistent reuse.
    /// # Safety
    /// The bytes must be the actual current capture from the controlled deployment
    /// identity tool, with its reviewed completeness declarations. The caller must
    /// supply the real linked executable source/build attestations, not hashes chosen
    /// to make an arbitrary JSON claim match. This asserts deployment provenance;
    /// matching JSON fields alone cannot establish a qualified producer.
    ///
    /// ```compile_fail
    /// use pse_runtime::math::portable::QualifiedProducer;
    /// let id=pse_ids::ContentHash::from_bytes([0;32]);
    /// let _=QualifiedProducer::from_deployment_receipt(b"{}",id,id);
    /// ```
    /// # Errors
    /// Wrong interpretation, malformed identity or contradictory eligibility evidence.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 controlled deployment producer qualification mint"
    )]
    pub unsafe fn from_deployment_receipt(
        bytes: &[u8],
        expected_source: ContentHash,
        expected_build: ContentHash,
    ) -> Result<Option<Self>, PortableError> {
        #[derive(Deserialize)]
        struct Receipt {
            frame: String,
            identity: String,
            persistent_reuse_eligible: bool,
            reasons: Vec<String>,
            outer_attestation: Option<Attestation>,
        }
        #[derive(Deserialize)]
        struct Attestation {
            source: ContentHash,
            build: ContentHash,
        }
        let receipt: Receipt = serde_json::from_slice(bytes)
            .map_err(|error| PortableError::Qualification(error.to_string()))?;
        if receipt.frame != Frame::ProducerV1.as_str() {
            return Err(PortableError::Qualification(
                "unsupported producer interpretation".into(),
            ));
        }
        if !receipt.persistent_reuse_eligible {
            return Ok(None);
        }
        if !receipt.reasons.is_empty() {
            return Err(PortableError::Qualification(
                "eligible producer retains unresolved input evidence".into(),
            ));
        }
        let Some(attestation) = receipt.outer_attestation else {
            return Ok(None);
        };
        if attestation.source != expected_source || attestation.build != expected_build {
            return Ok(None);
        }
        let identity = ContentHash::parse_hex(&receipt.identity)
            .map_err(|error| PortableError::Qualification(error.to_string()))?;
        Ok(Some(Self { identity }))
    }
    /// Exact relevant producer identity; eligibility is not a property of global builds.
    pub fn identity(&self) -> ContentHash {
        self.identity
    }
    fn key(&self) -> String {
        format!(
            "{}{}",
            pse_operations::canonical_selection::SCIENTIFIC_PRODUCER_PREFIX,
            self.identity.to_hex()
        )
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    interpretation: String,
    semantic_identity: SemanticBodyHash,
    physical: ContentHash,
    spec: pse_math::binding::BodySpec,
    payload_hash: ContentHash,
}
// Keep the compiler payload as opaque bytes. Nesting it as a JSON number array
// would duplicate and expand the same semantic description at the transport boundary.
const ENVELOPE_MAGIC: &[u8] = b"pse-admitted-body-product-v1\0";
// Framing metadata is bounded independently of the scientific recipe and storage wire blocks.
const MAX_METADATA_BYTES: usize = 1024 * 1024;
const MAX_PRODUCT_BYTES: usize = ENVELOPE_MAGIC.len()
    + 4
    + MAX_METADATA_BYTES
    + pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES;
fn pack(envelope: &Envelope, payload: &[u8]) -> Result<Vec<u8>, PortableError> {
    let metadata = serde_json::to_vec(envelope)
        .map_err(|error| PortableError::Qualification(error.to_string()))?;
    let length = u32::try_from(metadata.len()).map_err(|_| {
        PortableError::Qualification("portable metadata exceeds frame bound".into())
    })?;
    let total = ENVELOPE_MAGIC
        .len()
        .saturating_add(4)
        .saturating_add(metadata.len())
        .saturating_add(payload.len());
    if metadata.len() > MAX_METADATA_BYTES
        || payload.len() > pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES
    {
        return Err(PortableError::Qualification(
            "portable envelope exceeds its logical extent".into(),
        ));
    }
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(ENVELOPE_MAGIC);
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(&metadata);
    bytes.extend_from_slice(payload);
    Ok(bytes)
}
// Validate framing and exact bounded slices before parsing metadata or constructing record authority.
fn split_frame(bytes: &[u8]) -> Result<(&[u8], &[u8]), PortableError> {
    if bytes.len() > MAX_PRODUCT_BYTES {
        return Err(PortableError::Qualification(
            "portable envelope exceeds its logical extent".into(),
        ));
    }
    let bytes = bytes.strip_prefix(ENVELOPE_MAGIC).ok_or_else(|| {
        PortableError::Qualification("unsupported portable envelope framing".into())
    })?;
    let length = bytes
        .get(..4)
        .ok_or_else(|| PortableError::Qualification("truncated portable metadata length".into()))?;
    let length = u32::from_be_bytes(
        length
            .try_into()
            .map_err(|_| PortableError::Qualification("invalid portable metadata length".into()))?,
    ) as usize;
    if length > MAX_METADATA_BYTES {
        return Err(PortableError::Qualification(
            "portable metadata exceeds frame bound".into(),
        ));
    }
    let data = &bytes[4..];
    let metadata = data
        .get(..length)
        .ok_or_else(|| PortableError::Qualification("truncated portable metadata".into()))?;
    let payload = &data[length..];
    pse_ids::scientific_replay::QualifiedScientificRecipe::scratch_bytes(payload.len())
        .map_err(|error| PortableError::Qualification(error.to_string()))?;
    Ok((metadata, payload))
}
fn unpack(bytes: &[u8]) -> Result<(Envelope, &[u8]), PortableError> {
    let (metadata, payload) = split_frame(bytes)?;
    let envelope = serde_json::from_slice(metadata)
        .map_err(|error| PortableError::Qualification(error.to_string()))?;
    Ok((envelope, payload))
}
fn reserve_portable(
    service: &super::MathService,
    label: &'static str,
    bytes: usize,
) -> Result<Arc<pse_columnar::AllocationLease>, PortableError> {
    service.reserve(label, bytes).map_err(|error| {
        let (retained, cause) = match error.retained_bytes() {
            Ok(retained) => (retained, pse_model::diagnostic::DiagnosticCause::new(error)),
            Err(refusal) => (
                refusal.retained_bytes(),
                pse_model::diagnostic::DiagnosticCause::new(refusal),
            ),
        };
        PortableError::Math(pse_math::MathError::Typed { retained, cause })
    })
}
/// Exact portable product key admitted under the selected revision's durable root.
#[derive(Clone, Debug)]
pub struct PublishedBody {
    key: String,
    semantic_identity: SemanticBodyHash,
}
impl PublishedBody {
    /// Immutable canonical product identity.
    pub fn key(&self) -> &str {
        &self.key
    }
    /// Compiler-issued body identity authenticated by the published product.
    pub fn semantic_identity(&self) -> SemanticBodyHash {
        self.semantic_identity
    }
}
fn request(identity: SemanticBodyHash, physical: ContentHash) -> Vec<u8> {
    let mut hash = FramedHasher::new(Frame::MathAdmittedRecipeV1);
    hash.str(INTERPRETATION)
        .str("request")
        .hash(&identity.as_id())
        .hash(&physical);
    hash.finish_hash().as_bytes().to_vec()
}
fn encode(
    body: &AdmittedBody,
    physical: ContentHash,
) -> Result<(Envelope, Vec<u8>), PortableError> {
    if body.spec().physical != physical {
        return Err(PortableError::Qualification(
            "admitted body physical context differs from deployment".into(),
        ));
    }
    let semantic_identity = body.semantic_identity().ok_or_else(|| {
        PortableError::Qualification("body lacks compiler-issued selected semantic identity".into())
    })?;
    let payload = body.portable_payload()?;
    Ok((
        Envelope {
            interpretation: INTERPRETATION.into(),
            semantic_identity,
            physical,
            spec: body.spec().clone(),
            payload_hash: portable_payload_hash(&payload),
        },
        payload,
    ))
}
/// Publish one admitted body and the selected read premises in the canonical substrate.
/// The storage owner establishes the immutable product/root. Selection protection stays
/// live until the caller explicitly releases it after publishing the complete selection.
/// # Errors
/// Unsealed body, incomplete receipts, changed physical context or guarded store refusal.
pub async fn publish_body(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &SelectedRead,
    producer: &QualifiedProducer,
    body: &AdmittedBody,
    context: &CompilerContext,
) -> Result<PublishedBody, PortableError> {
    publish_with_producer_key(service, store, read, producer.key(), body, context).await
}
/// Persist admitted meaning when producer eligibility is incomplete. The outer build
/// is recorded as unqualified provenance and never used as a relevant producer key.
pub async fn publish_unqualified_body(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &SelectedRead,
    outer_build: ContentHash,
    body: &AdmittedBody,
    context: &CompilerContext,
) -> Result<PublishedBody, PortableError> {
    publish_with_producer_key(
        service,
        store,
        read,
        format!("pse.unqualified-producer.v1:{}", outer_build.to_hex()),
        body,
        context,
    )
    .await
}
#[allow(
    unsafe_code,
    reason = "ADR-0164 compiler-issued scientific description publication in reserved namespace"
)]
async fn publish_with_producer_key(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &SelectedRead,
    producer: String,
    body: &AdmittedBody,
    context: &CompilerContext,
) -> Result<PublishedBody, PortableError> {
    let extent = body.portable_payload_bytes()?;
    // Both encoded recipe and packed envelope coexist until storage takes ownership.
    let _scratch = reserve_portable(
        service,
        "math:qualified-recipe-encode",
        extent
            .saturating_mul(3)
            .saturating_add(MAX_METADATA_BYTES.saturating_mul(3))
            .saturating_add(read.publication_scratch_bytes()?),
    )?;
    let physical = physical_identity(&context.quantities, &context.preconditions);
    let (envelope, body_payload) = encode(body, physical)?;
    let request = request(envelope.semantic_identity, physical);
    let payload = pack(&envelope, &body_payload)?;
    drop(body_payload);
    let mut hash = FramedHasher::new(Frame::MathAdmittedRecipeV1);
    hash.str(INTERPRETATION)
        .str("product")
        .part(&request)
        .str(&producer)
        .part(&payload);
    let key = hash.finish_hash().to_hex();
    let revision = read.selection().revision();
    let product = pse_model::generated::runtime::canonical_products::Row {
        key: key.clone(),
        problem: revision.problem.clone(),
        revision: revision.key.clone(),
        request: request.into(),
        payload: payload.into(),
        dependencies: Vec::new().into(),
        producer,
        interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
    };
    let key = if product
        .producer
        .starts_with(pse_operations::canonical_selection::SCIENTIFIC_PRODUCER_PREFIX)
    {
        // SAFETY: encode accepts only an AdmittedBody with a private compiler seal
        // emitted by normal scientific admission or previously qualified replay.
        // The caller's QualifiedProducer carries deployment eligibility; decoded DTOs
        // cannot obtain AdmittedBody or enter this reserved publication path.
        unsafe { store.publish_scientific_product(read, product) }.await?
    } else {
        store.publish_product(read, product).await?
    };
    Ok(PublishedBody {
        key,
        semantic_identity: envelope.semantic_identity,
    })
}
/// Return only a dependency-qualified persisted product, reconstructed from its actual
/// concrete receipts. A hit with corrupt or incomplete receipts is an error; it never
/// runs normal admission and is never reported as a reconstruction success.
/// # Errors
/// Storage/protection failure, incompatible envelope, missing receipts or native error.
#[allow(
    unsafe_code,
    reason = "ADR-0164 sole production storage-to-science authority mint after canonical qualification"
)]
pub async fn reuse_body(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &mut SelectedRead,
    producer: &QualifiedProducer,
    identity: SemanticBodyHash,
    context: &CompilerContext,
    cancelled: &Arc<AtomicBool>,
) -> Result<Option<AdmittedBody>, PortableError> {
    let physical = physical_identity(&context.quantities, &context.preconditions);
    let mut after = String::new();
    loop {
        let Some(candidate) = store
            .product_candidate(
                read.selection(),
                &request(identity, physical),
                &producer.key(),
                &after,
            )
            .await?
        else {
            return Ok(None);
        };
        let (product, _product_scratch) =
            qualify_candidate(service, store, read, &candidate).await?;
        let Some(product) = product else {
            after = candidate.key().to_owned();
            continue;
        };
        // Reserve before envelope/recipe parsing or record-index allocation. The lease
        // remains live through strict reconstruction; decoded records never outlive it.
        let (metadata, payload) = split_frame(product.payload())?;
        let scratch =
            pse_ids::scientific_replay::QualifiedScientificRecipe::scratch_bytes(payload.len())
                .map_err(|e| PortableError::Qualification(e.to_string()))?
                .saturating_add(metadata.len().saturating_mul(256));
        let _scratch = reserve_portable(service, "math:qualified-recipe-replay", scratch)?;
        let (envelope, body_payload) = unpack(product.payload())?;
        if envelope.interpretation != INTERPRETATION
            || envelope.semantic_identity != identity
            || envelope.physical != physical
            || envelope.spec.physical != physical
        {
            return Err(PortableError::Qualification(
                "persisted body differs from qualified selected request".into(),
            ));
        }
        pse_math::initialize()?;
        let qualification = ContentHash::parse_hex(product.key())
            .map_err(|e| PortableError::Qualification(e.to_string()))?;
        let selected_request = ContentHash::try_from_slice(&request(identity, physical))
            .map_err(|e| PortableError::Qualification(e.to_string()))?;
        // SAFETY: ReusableProduct is issued only by the protected canonical selection
        // after complete dependency/eligible-producer/interpretation qualification.
        // This controlled scientific publication role emits compiler-sealed descriptions;
        // arbitrary generic product writers must not impersonate that deployment role.
        let permit = unsafe {
            pse_ids::scientific_replay::QualifiedScientificRecipe::from_canonical_product(
                body_payload.to_vec(),
                envelope.payload_hash,
                qualification,
                selected_request,
            )
        }
        .map_err(|e| PortableError::Qualification(e.to_string()))?;
        let body = reconstruct_portable(&permit, &context.quantities, cancelled)?;
        if body.spec() != &envelope.spec {
            return Err(PortableError::Qualification(
                "reconstructed body differs from qualified envelope specification".into(),
            ));
        }
        if body.semantic_identity() != Some(identity) {
            return Err(PortableError::Qualification(
                "reconstructed body lacks exact sealed semantic identity".into(),
            ));
        }
        return Ok(Some(body));
    }
}

// A store-issued descriptor is the only path into body hydration. Keep this lease
// together with the qualified payload until its scientific reconstruction finishes.
async fn qualify_candidate(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &mut SelectedRead,
    candidate: &pse_operations::canonical_selection::ProductCandidate,
) -> Result<
    (
        Option<pse_operations::canonical_selection::ReusableProduct>,
        Arc<pse_columnar::AllocationLease>,
    ),
    PortableError,
> {
    let lease = reserve_portable(
        service,
        "math:qualified-product-hydration",
        candidate.retained_bytes(),
    )?;
    let product = store.qualify_product(read, candidate).await?;
    Ok((product, lease))
}

#[cfg(test)]
mod portable_frame_tests {
    use super::*;
    #[test]
    fn portable_envelope_over_wire_block_preserves_hash_and_refuses_forged_extent() {
        let payload = vec![b'x'; 3 * 1024 * 1024 + 1];
        let id = ContentHash::from_bytes([1; 32]);
        let envelope = Envelope {
            interpretation: INTERPRETATION.into(),
            semantic_identity: SemanticBodyHash::from_id(id),
            physical: id,
            spec: pse_math::binding::BodySpec {
                definition: id,
                structure: id,
                physical: id,
                admissions: id,
                providers: Vec::new(),
                policy: id,
            },
            payload_hash: portable_payload_hash(&payload),
        };
        let frame = pack(&envelope, &payload).unwrap();
        assert!(frame.len() > 3 * 1024 * 1024);
        let (restored, bytes) = unpack(&frame).unwrap();
        assert_eq!(bytes, payload);
        assert_eq!(restored.payload_hash, portable_payload_hash(bytes));
        let mut forged = frame;
        let offset = ENVELOPE_MAGIC.len();
        forged[offset..offset + 4]
            .copy_from_slice(&((MAX_METADATA_BYTES + 1) as u32).to_be_bytes());
        assert!(split_frame(&forged).is_err());
        assert!(split_frame(&forged[..offset + 3]).is_err());
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_portable_body_tests {
    use super::*;
    use pse_compiler::workspace::{CompilerWorkspace, ModelingBodyRetention, WorkspaceLimits};
    use pse_modeling::{Bindings, InstanceId, Limits, PhysicalScope};
    use pse_operations::canonical::{CanonicalOptions, ObjectEdit};
    use std::{collections::BTreeMap, path::Path, sync::Mutex, time::Duration};
    #[allow(
        unsafe_code,
        reason = "controlled deployment fixtures test refusal and outer-binding semantics"
    )]
    fn deployment_fixture(
        bytes: &[u8],
        source: ContentHash,
        build: ContentHash,
    ) -> Result<Option<QualifiedProducer>, PortableError> {
        // SAFETY: isolated fixture supplies a complete controlled receipt and exact
        // fixture attestations; no production completeness is asserted by these tests.
        unsafe { QualifiedProducer::from_deployment_receipt(bytes, source, build) }
    }
    #[test]
    fn canonical_portable_body_errors_preserve_typed_identity() {
        use pse_diagnostics::{DiagnosticCode, TypedDiagnostic, diagnostic_leaves};
        let store = PortableError::Store(CanonicalError::PayloadLimit);
        assert_eq!(
            store.diagnostic_code(),
            Some(DiagnosticCode::RuntimeInfrastructure)
        );
        assert_eq!(diagnostic_leaves(&store).len(), 1);
        let math = pse_math::MathError::Contract("receipt mismatch".into());
        let expected = math.diagnostic_code();
        let error = PortableError::Math(math);
        assert_eq!(error.diagnostic_code(), expected);
        assert_eq!(diagnostic_leaves(&error)[0].diagnostic_code(), expected);
        assert_eq!(
            PortableError::Qualification("producer ineligible".into()).diagnostic_code(),
            Some(DiagnosticCode::RuntimeInfrastructure)
        );
    }
    #[tokio::test]
    async fn canonical_portable_body_large_scientific_recipe_persists_and_reconnects() {
        let state = std::env::var("PSE_SURREAL_STATE")
            .expect("canonical portable unit supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_portable_large_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        // One ordinary semantic body retains a balanced consumed arithmetic tree and
        // its physical/proof receipts. Independent equations produce separate bodies.
        let mut terms = vec![String::from("(x-x)"); 1024];
        while terms.len() > 1 {
            terms = terms
                .chunks_exact(2)
                .map(|pair| format!("({}+{})", pair[0], pair[1]))
                .collect();
        }
        let text = format!(
            "package p {{def Root {{var x:Scalar;eq residual:x+{}==2;}}}}",
            terms.pop().unwrap()
        );
        let revision = store
            .edit(
                "problem",
                None,
                "authored-initial",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("root-v1", &text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(3600))
                .await
                .unwrap(),
        );
        store
            .resolve_names(&mut read, "p", &["Root".into()])
            .await
            .unwrap();
        let context = context();
        let body = admitted_with_limits(
            context.clone(),
            &text,
            Limits {
                body_occurrences: Some(262_144),
                body_slots: Some(65_536),
                ..Limits::default()
            },
        );
        let identity = body.semantic_identity().unwrap();
        let spec = body.spec().clone();
        let payload_bytes = body.portable_payload_bytes().unwrap();
        assert!(payload_bytes > 3 * 1024 * 1024);
        let service = crate::math::tests::limited_service(8usize << 30);
        let producer = producer(68);
        publish_body(&service, &store, &read, &producer, &body, &context)
            .await
            .unwrap();
        store.release(read.selection()).await.unwrap();
        drop(body);
        drop(text);
        drop(read);
        drop(store);
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.open().await.unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(revision, Duration::from_secs(3600))
                .await
                .unwrap(),
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let restored = reuse_body(
            &service, &store, &mut read, &producer, identity, &context, &cancel,
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(restored.spec(), &spec);
        let value_support = restored
            .math()
            .compile(
                &[0],
                &[],
                pse_kernels::DerivativeOrder::Value,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap();
        assert_eq!(value_support.input_formals().len(), 1);
        let coordinate = value_support.input_formals()[0];
        let compiled = restored
            .math()
            .compile(
                &[0],
                &[coordinate],
                pse_kernels::DerivativeOrder::First,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap();
        let values = vec![9.0; compiled.input_formals().len()];
        let jet = compiled
            .worker()
            .evaluate(
                &values,
                pse_kernels::DerivativeOrder::First,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(jet.values, vec![7.0]);
        assert_eq!(jet.jacobian, vec![1.0]);
        store.release(read.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_portable_body_candidate_reserves_before_protected_hydration() {
        use pse_diagnostics::{DiagnosticCode, TypedDiagnostic};
        let state = std::env::var("PSE_SURREAL_STATE")
            .expect("canonical portable unit supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_portable_budget_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let revision = store
            .edit(
                "problem",
                None,
                "authored-initial",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("root-v1", text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(revision, Duration::from_secs(60))
                .await
                .unwrap(),
        );
        store
            .resolve_names(&mut read, "p", &["Root".into()])
            .await
            .unwrap();
        let context = context();
        let body = admitted(context.clone(), text);
        let service = crate::math::tests::service();
        let producer = producer(67);
        let published = publish_body(&service, &store, &read, &producer, &body, &context)
            .await
            .unwrap();
        let candidate = store
            .product_candidate(
                read.selection(),
                &request(
                    body.semantic_identity().unwrap(),
                    physical_identity(&context.quantities, &context.preconditions),
                ),
                &producer.key(),
                "",
            )
            .await
            .unwrap()
            .unwrap();
        assert!(candidate.retained_bytes() > 1024);
        // Header discovery stays valid, while protected payload hydration will now
        // find corruption. A tiny budget must refuse before reading that native block.
        store.corrupt_product_block(published.key()).await.unwrap();
        let tiny = crate::math::tests::limited_service(1024);
        let identity = body.semantic_identity().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let error = reuse_body(
            &tiny, &store, &mut read, &producer, identity, &context, &cancel,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error.diagnostic_code(),
            Some(DiagnosticCode::RuntimeResourceLimit)
        );
        assert_eq!(
            pse_model::diagnostic::project_typed(
                &error,
                pse_diagnostics::DiagnosticStage::ModelingAdmission
            )
            .class,
            pse_model::diagnostic::BoundaryClass::ResourceLimit
        );
        let error = reuse_body(
            &service, &store, &mut read, &producer, identity, &context, &cancel,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(error, PortableError::Store(CanonicalError::Configuration(ref detail))
            if detail.contains("block identity/length/digest mismatch"))
        );
        store.release(read.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[derive(Debug, Default)]
    struct Capture(Mutex<Vec<Arc<AdmittedBody>>>);
    impl ModelingBodyRetention for Capture {
        fn generation(&self) -> u64 {
            0
        }
        fn get(
            &self,
            _: SemanticBodyHash,
        ) -> Result<Option<Arc<AdmittedBody>>, pse_math::MathError> {
            Ok(None)
        }
        fn retain(
            &self,
            _: u64,
            _: SemanticBodyHash,
            body: Arc<AdmittedBody>,
        ) -> Result<Arc<AdmittedBody>, pse_math::MathError> {
            self.0.lock().unwrap().push(body.clone());
            Ok(body)
        }
    }
    fn context() -> CompilerContext {
        CompilerContext {
            quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
            preconditions: Arc::new(
                pse_quantity::PhysicalPreconditions::new(
                    pse_quantity::generated::standard_preconditions(),
                )
                .unwrap(),
            ),
            providers: BTreeMap::new(),
        }
    }
    fn admitted(context: CompilerContext, text: &str) -> Arc<AdmittedBody> {
        admitted_with_limits(context, text, Limits::default())
    }
    fn admitted_with_limits(
        context: CompilerContext,
        text: &str,
        limits: Limits,
    ) -> Arc<AdmittedBody> {
        let declarations = pse_authoring::language::parse(
            text,
            pse_ids::SemanticId::from_bytes([88; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let capture = Arc::new(Capture::default());
        let mut compiler = CompilerWorkspace::new(context, WorkspaceLimits::default()).unwrap();
        compiler.attach_body_retention(capture.clone()).unwrap();
        compiler
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let prepared = compiler
            .prepare_modeling_cancellable(
                root,
                InstanceId::from_id(pse_ids::SemanticId::NIL),
                Bindings::default(),
                limits,
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let body = capture
            .0
            .lock()
            .unwrap()
            .iter()
            .find(|body| body.has_portable_payload())
            .unwrap()
            .clone();
        drop(prepared);
        drop(compiler);
        drop(capture);
        body
    }
    // A named fixture qualification exercises consumer mechanics only. It makes no
    // claim that the real deployment's producer inputs have been completely qualified.
    fn producer(byte: u8) -> QualifiedProducer {
        QualifiedProducer {
            identity: ContentHash::from_bytes([byte; 32]),
        }
    }
    #[derive(Debug)]
    struct RefuseFreshAdmission {
        inner: crate::math::retention::CanonicalBodyRetention,
        wanted: SemanticBodyHash,
        restored: Arc<Mutex<Option<Arc<AdmittedBody>>>>,
    }
    impl ModelingBodyRetention for RefuseFreshAdmission {
        fn generation(&self) -> u64 {
            self.inner.generation()
        }
        fn get(
            &self,
            key: SemanticBodyHash,
        ) -> Result<Option<Arc<AdmittedBody>>, pse_math::MathError> {
            let value = self.inner.get(key)?;
            if key == self.wanted {
                *self.restored.lock().unwrap() = value.clone();
            }
            Ok(value)
        }
        fn retain(
            &self,
            _: u64,
            _: SemanticBodyHash,
            _: Arc<AdmittedBody>,
        ) -> Result<Arc<AdmittedBody>, pse_math::MathError> {
            Err(pse_math::MathError::Contract(
                "fresh admission forbidden in persisted-hit control".into(),
            ))
        }
    }
    #[tokio::test]
    async fn canonical_portable_body_normal_preparation_hits_before_admission_after_reconnect() {
        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_normal_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = Arc::new(CanonicalStore::connect(&options).await.unwrap());
        store.create().await.unwrap();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let revision = store
            .edit(
                "normal",
                None,
                "source",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("normal-v1", text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let context = context();
        let wanted = admitted(context.clone(), text).semantic_identity().unwrap();
        let mut selected = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        store
            .resolve_names(&mut selected, "p", &["Root".into()])
            .await
            .unwrap();
        selected
            .interpretation(
                "physical".into(),
                physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
            )
            .unwrap();
        let read = Arc::new(Mutex::new(selected));
        let service = crate::math::tests::service();
        let workspace = service
            .canonical_workspace(
                context.clone(),
                WorkspaceLimits::default(),
                store.clone(),
                read.clone(),
                Some(producer(65)),
                ContentHash::from_bytes([66; 32]),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let prepare = move |compiler: &mut CompilerWorkspace| {
            let declarations = pse_authoring::language::parse(
                text,
                pse_ids::SemanticId::from_bytes([88; 16]),
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = declarations
                .iter()
                .find(|v| v.name == "Root")
                .unwrap()
                .declaration_id;
            compiler
                .publish_modeling(declarations, PhysicalScope::default())
                .unwrap();
            compiler.prepare_modeling_cancellable(
                root,
                InstanceId::from_id(pse_ids::SemanticId::NIL),
                Bindings::default(),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
        };
        tokio::task::spawn_blocking(move || prepare(&mut workspace.compiler.lock().unwrap()))
            .await
            .unwrap()
            .unwrap();
        let protection = read.lock().unwrap().selection().clone();
        store.release(&protection).await.unwrap();
        drop(read);
        drop(service);
        drop(store);
        let store = Arc::new(CanonicalStore::connect(&options).await.unwrap());
        store.open().await.unwrap();
        let mut selected = SelectedRead::new(
            store
                .protect(revision, Duration::from_secs(60))
                .await
                .unwrap(),
        );
        selected
            .interpretation(
                "physical".into(),
                physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
            )
            .unwrap();
        let read = Arc::new(Mutex::new(selected));
        let service = crate::math::tests::service();
        let restored = Arc::new(Mutex::new(None));
        let attachment = RefuseFreshAdmission {
            inner: crate::math::retention::CanonicalBodyRetention {
                memory: crate::math::retention::BodyRetention(Arc::downgrade(&service)),
                store: store.clone(),
                read: read.clone(),
                producer: Some(producer(65)),
                outer_build: ContentHash::from_bytes([66; 32]),
                inputs: context.clone(),
                cancelled: Arc::new(AtomicBool::new(false)),
                handle: tokio::runtime::Handle::current(),
            },
            wanted,
            restored: restored.clone(),
        };
        let mut workspace = CompilerWorkspace::new(context, WorkspaceLimits::default()).unwrap();
        workspace
            .attach_body_retention(Arc::new(attachment))
            .unwrap();
        tokio::task::spawn_blocking(move || prepare(&mut workspace))
            .await
            .unwrap()
            .unwrap();
        let body = restored
            .lock()
            .unwrap()
            .take()
            .expect("normal body lookup restored persisted product before admission");
        let cancel = Arc::new(AtomicBool::new(false));
        let compiled = body
            .math()
            .compile(
                &[0],
                &[0],
                pse_kernels::DerivativeOrder::First,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap();
        let mut worker = compiled.worker();
        let jet = worker
            .evaluate(
                &[9.0],
                pse_kernels::DerivativeOrder::First,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(9.0 - jet.values[0] / jet.jacobian[0], 2.0);
        let protection = read.lock().unwrap().selection().clone();
        store.release(&protection).await.unwrap();
        drop(body);
        drop(read);
        drop(service);
        store.remove_isolated_fixture().await.unwrap();
    }
    fn version(key: &str, text: &str) -> pse_model::generated::runtime::canonical_versions::Row {
        pse_model::generated::runtime::canonical_versions::Row {
            key: key.into(),
            logical: "root".into(),
            kind: "authored-modeling".into(),
            payload: text.as_bytes().to_vec().into(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        }
    }
    #[tokio::test]
    async fn canonical_portable_body_reconnect_reuses_ordinary_scalar_solve_with_actual_tool_fixture_receipt()
     {
        let state = std::env::var("PSE_SURREAL_STATE")
            .expect("canonical portable unit supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_portable_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let revision = store
            .edit(
                "problem",
                None,
                "authored-initial",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("root-v1", text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let context = context();
        let body = admitted(context.clone(), text);
        let identity = body.semantic_identity().unwrap();
        // The recipe runs the actual production identity tool over a finite Rust-only
        // fixture. Its eligible receipt is evidence for this integration fixture;
        // it is explicitly not real scientific-kernel deployment qualification.
        let receipt_path = std::env::var("PSE_PRODUCER_FIXTURE_RECEIPT")
            .expect("portable recipe supplies actual tool fixture capture");
        let bytes = std::fs::read(receipt_path).unwrap();
        let receipt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(receipt["package"], "producer-fixture");
        assert_eq!(receipt["persistent_reuse_eligible"], true);
        assert!(receipt["reasons"].as_array().unwrap().is_empty());
        assert!(!receipt["units"].as_array().unwrap().is_empty());
        let source =
            serde_json::from_value(receipt["outer_attestation"]["source"].clone()).unwrap();
        let build = serde_json::from_value(receipt["outer_attestation"]["build"].clone()).unwrap();
        let qualified = deployment_fixture(&bytes, source, build)
            .unwrap()
            .expect("actual finite tool fixture is eligible");
        assert_eq!(
            qualified.identity().to_hex(),
            receipt["identity"].as_str().unwrap()
        );
        let mut read = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        assert_eq!(
            store
                .resolve_names(&mut read, "p", &["Root".into()])
                .await
                .unwrap()
                .len(),
            1
        );
        read.interpretation(
            "physical".into(),
            physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
        )
        .unwrap();
        let service = crate::math::tests::service();
        let published = publish_body(&service, &store, &read, &qualified, &body, &context)
            .await
            .unwrap();
        assert_eq!(published.semantic_identity(), identity);
        let outer_build = ContentHash::from_bytes([67; 32]);
        let unqualified =
            publish_unqualified_body(&service, &store, &read, outer_build, &body, &context)
                .await
                .unwrap();
        let unqualified_key = format!("pse.unqualified-producer.v1:{}", outer_build.to_hex());
        let candidate = store
            .product_candidate(
                read.selection(),
                &request(
                    identity,
                    physical_identity(&context.quantities, &context.preconditions),
                ),
                &unqualified_key,
                "",
            )
            .await
            .unwrap()
            .unwrap();
        let (unqualified_product, _product_lease) =
            qualify_candidate(&service, &store, &mut read, &candidate)
                .await
                .unwrap();
        let unqualified_product = unqualified_product.unwrap();
        assert_eq!(unqualified_product.key(), unqualified.key());
        assert_eq!(unqualified_product.producer(), unqualified_key);
        assert!(
            reuse_body(
                &crate::math::tests::service(),
                &store,
                &mut read,
                &producer(67),
                identity,
                &context,
                &Arc::new(AtomicBool::new(false))
            )
            .await
            .unwrap()
            .is_none()
        );
        store.release(read.selection()).await.unwrap();
        // Drop the native source product and the original authenticated transport.
        drop(body);
        drop(read);
        drop(store);
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.open().await.unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        read.interpretation(
            "physical".into(),
            physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
        )
        .unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let rebuilt = reuse_body(
            &crate::math::tests::service(),
            &store,
            &mut read,
            &qualified,
            identity,
            &context,
            &cancel,
        )
        .await
        .unwrap()
        .expect("qualified persisted body");
        assert_eq!(rebuilt.semantic_identity(), Some(identity));
        let compiled = rebuilt
            .math()
            .compile(
                &[0],
                &[0],
                pse_kernels::DerivativeOrder::First,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap();
        let mut x = 9.0;
        let mut worker = compiled.worker();
        for _ in 0..3 {
            let result = worker
                .evaluate(
                    &[x],
                    pse_kernels::DerivativeOrder::First,
                    &mut BTreeMap::new(),
                    &cancel,
                )
                .unwrap();
            if result.values[0].abs() < 1e-12 {
                break;
            }
            x -= result.values[0] / result.jacobian[0];
        }
        assert_eq!(x, 2.0);
        // A different relevant producer has no compatible persisted product.
        assert!(
            reuse_body(
                &crate::math::tests::service(),
                &store,
                &mut read,
                &producer(45),
                identity,
                &context,
                &cancel
            )
            .await
            .unwrap()
            .is_none()
        );
        store.release(read.selection()).await.unwrap();
        let changed = store
            .edit(
                "problem",
                Some(&revision.key),
                "authored-change",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version(
                        "root-v2",
                        "package p {def Root {var x:Scalar;eq residual:x==3;}}",
                    )),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let mut changed_read = SelectedRead::new(
            store
                .protect(changed, Duration::from_secs(60))
                .await
                .unwrap(),
        );
        changed_read
            .interpretation(
                "physical".into(),
                physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
            )
            .unwrap();
        assert!(
            reuse_body(
                &crate::math::tests::service(),
                &store,
                &mut changed_read,
                &qualified,
                identity,
                &context,
                &cancel
            )
            .await
            .unwrap()
            .is_none()
        );
        store.release(changed_read.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[test]
    fn canonical_portable_body_incomplete_producer_refuses_persistent_reuse() {
        let receipt=serde_json::to_vec(&serde_json::json!({"frame":Frame::ProducerV1.as_str(),"identity":ContentHash::from_bytes([1;32]).to_hex(),"persistent_reuse_eligible":false,"reasons":["undeclared build-script input"]})).unwrap();
        assert!(
            deployment_fixture(
                &receipt,
                ContentHash::from_bytes([2; 32]),
                ContentHash::from_bytes([3; 32])
            )
            .unwrap()
            .is_none()
        );
        let contradictory=serde_json::to_vec(&serde_json::json!({"frame":Frame::ProducerV1.as_str(),"identity":ContentHash::from_bytes([1;32]).to_hex(),"persistent_reuse_eligible":true,"reasons":["unknown native bytes"]})).unwrap();
        assert!(
            deployment_fixture(
                &contradictory,
                ContentHash::from_bytes([2; 32]),
                ContentHash::from_bytes([3; 32])
            )
            .is_err()
        );
    }
    #[test]
    fn canonical_portable_body_producer_requires_current_outer_binding() {
        let source = ContentHash::from_bytes([2; 32]);
        let build = ContentHash::from_bytes([3; 32]);
        let identity = ContentHash::from_bytes([4; 32]);
        let receipt=serde_json::to_vec(&serde_json::json!({"frame":Frame::ProducerV1.as_str(),"identity":identity.to_hex(),"persistent_reuse_eligible":true,"reasons":[],"outer_attestation":{"source":source,"build":build}})).unwrap();
        assert_eq!(
            deployment_fixture(&receipt, source, build)
                .unwrap()
                .unwrap()
                .identity(),
            identity
        );
        assert!(
            deployment_fixture(&receipt, ContentHash::from_bytes([5; 32]), build)
                .unwrap()
                .is_none()
        );
        assert!(
            deployment_fixture(&receipt, source, ContentHash::from_bytes([6; 32]))
                .unwrap()
                .is_none()
        );
        let refreshed_source = ContentHash::from_bytes([7; 32]);
        let refreshed=serde_json::to_vec(&serde_json::json!({"frame":Frame::ProducerV1.as_str(),"identity":identity.to_hex(),"persistent_reuse_eligible":true,"reasons":[],"outer_attestation":{"source":refreshed_source,"build":build}})).unwrap();
        assert_eq!(
            deployment_fixture(&refreshed, refreshed_source, build)
                .unwrap()
                .unwrap()
                .identity(),
            identity
        );
    }

    #[test]
    fn canonical_portable_body_envelope_preserves_opaque_payload_and_refuses_truncation() {
        let hash = ContentHash::from_bytes([1; 32]);
        let envelope = Envelope {
            interpretation: INTERPRETATION.into(),
            semantic_identity: SemanticBodyHash::from(hash),
            physical: hash,
            spec: pse_math::binding::BodySpec {
                definition: hash,
                structure: hash,
                physical: hash,
                admissions: hash,
                providers: Vec::new(),
                policy: hash,
            },
            payload_hash: hash,
        };
        let payload = [0, 255, 128, 1, 0, 42];
        let packed = pack(&envelope, &payload).unwrap();
        let (decoded, raw) = unpack(&packed).unwrap();
        assert_eq!(decoded.spec, envelope.spec);
        assert_eq!(raw, payload);
        assert!(unpack(b"unknown-format").is_err());
        assert!(unpack(ENVELOPE_MAGIC).is_err());
        let mut short = ENVELOPE_MAGIC.to_vec();
        short.extend_from_slice(&u32::MAX.to_be_bytes());
        assert!(unpack(&short).is_err());
        assert!(
            pack(
                &envelope,
                &vec![0; pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES + 1]
            )
            .is_err()
        );
    }
    #[tokio::test]
    #[allow(
        unsafe_code,
        reason = "controlled corrupt-store fixture deliberately crosses publication trust boundary"
    )]
    async fn canonical_portable_body_corrupt_hit_refuses_normal_admission_fallback() {
        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_corrupt_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = Arc::new(CanonicalStore::connect(&options).await.unwrap());
        store.create().await.unwrap();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let context = context();
        let body = admitted(context.clone(), text);
        let identity = body.semantic_identity().unwrap();
        let revision = store
            .edit(
                "corrupt",
                None,
                "source",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("corrupt-v1", text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let mut selected = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        store
            .resolve_names(&mut selected, "p", &["Root".into()])
            .await
            .unwrap();
        let physical = physical_identity(&context.quantities, &context.preconditions);
        selected
            .interpretation("physical".into(), physical.to_prefixed())
            .unwrap();
        // Generic substrate admission establishes immutable dependency-qualified bytes;
        // this corrupt fixture deliberately lacks a valid scientific-owner envelope.
        // SAFETY: isolated corrupt-store fixture deliberately tests rejection at scientific decoding.
        unsafe {
            store.publish_scientific_product(
                &selected,
                pse_model::generated::runtime::canonical_products::Row {
                    key: "corrupt-recipe-fixture".into(),
                    problem: revision.problem.clone(),
                    revision: revision.key.clone(),
                    request: request(identity, physical).into(),
                    payload: b"corrupt scientifically typed bytes".to_vec().into(),
                    dependencies: Vec::new().into(),
                    producer: producer(68).key(),
                    interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
                },
            )
        }
        .await
        .unwrap();
        drop(body);
        let read = Arc::new(Mutex::new(selected));
        let service = crate::math::tests::service();
        let restored = Arc::new(Mutex::new(None));
        let attachment = RefuseFreshAdmission {
            inner: crate::math::retention::CanonicalBodyRetention {
                memory: crate::math::retention::BodyRetention(Arc::downgrade(&service)),
                store: store.clone(),
                read: read.clone(),
                producer: Some(producer(68)),
                outer_build: ContentHash::from_bytes([69; 32]),
                inputs: context.clone(),
                cancelled: Arc::new(AtomicBool::new(false)),
                handle: tokio::runtime::Handle::current(),
            },
            wanted: identity,
            restored: restored.clone(),
        };
        let mut compiler = CompilerWorkspace::new(context, WorkspaceLimits::default()).unwrap();
        compiler
            .attach_body_retention(Arc::new(attachment))
            .unwrap();
        let error = tokio::task::spawn_blocking(move || {
            let declarations = pse_authoring::language::parse(
                text,
                pse_ids::SemanticId::from_bytes([88; 16]),
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = declarations
                .iter()
                .find(|v| v.name == "Root")
                .unwrap()
                .declaration_id;
            compiler
                .publish_modeling(declarations, PhysicalScope::default())
                .unwrap();
            compiler
                .prepare_modeling_cancellable(
                    root,
                    InstanceId::from_id(pse_ids::SemanticId::NIL),
                    Bindings::default(),
                    Limits::default(),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap_err()
        })
        .await
        .unwrap();
        use pse_model::diagnostic::DiagnosticProjection;
        fn has_refusal(diagnostic: &pse_model::diagnostic::BoundaryDiagnostic) -> bool {
            diagnostic.observations.values().any(|value|matches!(value,pse_model::diagnostic::Observation::Text(detail) if detail.contains("unsupported portable envelope framing"))) || diagnostic.causes.iter().any(has_refusal)
        }
        let diagnostic =
            error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::ModelingAdmission);
        assert!(has_refusal(&diagnostic), "{diagnostic:?}");
        assert!(restored.lock().unwrap().is_none());
        let protection = read.lock().unwrap().selection().clone();
        store.release(&protection).await.unwrap();
        drop(read);
        drop(service);
        store.remove_isolated_fixture().await.unwrap();
    }
}
