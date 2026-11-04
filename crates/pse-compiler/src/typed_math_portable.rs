// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Strict direct-graph recipes. Native/library values are reconstructed by the one
//! admitted-fact consumer; no source parser, resolver, inference or inventory fallback.
use super::providers::SelectedProvidersRecord;
use super::*;
use pse_modeling::resolved::{ResolvedRecord, SupportRecord, SymbolIx};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
const INTERPRETATION: &str = pse_ids::scientific_replay::ADMITTED_RECIPE_INTERPRETATION;
const MAX_PAYLOAD: usize = pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES;
#[derive(Default)]
struct ByteCount(usize);
impl std::io::Write for ByteCount {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("portable encoded extent overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    interpretation: String,
    semantic_identity: Option<pse_ids::roles::SemanticBodyHash>,
    syntax: ResolvedRecord,
    support: SupportRecord,
    providers: SelectedProvidersRecord,
    inputs: Vec<SymbolIx>,
    physical: ContentHash,
    structure: ContentHash,
    slots: usize,
    occurrences: usize,
    outputs: Vec<QuantityTypeId>,
    checked_members: BTreeSet<SemanticId>,
    checked_sources: BTreeSet<SemanticId>,
    occurrence_tokens: BTreeSet<SemanticId>,
    receipts: pse_quantity::resolved::receipts::ReceiptStream,
    proofs: pse_math::typed::receipts::ProofStream,
    spec: BodySpec,
}
fn contract(error: impl std::fmt::Display) -> MathError {
    MathError::Contract(error.to_string())
}
fn oversized(required: usize) -> MathError {
    MathError::ByteLimit {
        resource: "portable body payload",
        required,
        available: MAX_PAYLOAD,
    }
}
fn check_payload(payload: &[u8]) -> Result<(), MathError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(oversized(payload.len()));
    }
    Ok(())
}
impl Recipe {
    fn encode(&self) -> Result<Arc<[u8]>, MathError> {
        let mut count = ByteCount::default();
        serde_json::to_writer(&mut count, self).map_err(contract)?;
        if count.0 > MAX_PAYLOAD {
            return Err(oversized(count.0));
        }
        let mut bytes = Vec::with_capacity(count.0);
        serde_json::to_writer(&mut bytes, self).map_err(contract)?;
        if bytes.len() != count.0 {
            return Err(contract("portable encoded extent changed"));
        }
        Ok(bytes.into())
    }
    fn decode(payload: &[u8]) -> Result<Self, MathError> {
        check_payload(payload)?;
        // Interpret the bounded header before decoding representation-specific native records.
        // An older shape is incompatible data, not corruption of the current representation.
        #[derive(Deserialize)]
        struct Header {
            interpretation: String,
        }
        let header: Header = serde_json::from_slice(payload).map_err(contract)?;
        if header.interpretation != INTERPRETATION {
            return Err(contract("unsupported admitted recipe interpretation"));
        }
        let value: Self = serde_json::from_slice(payload).map_err(contract)?;
        if value.physical != value.spec.physical || value.structure != value.spec.structure {
            return Err(contract(
                "portable interpretation or admitted specification differs",
            ));
        }
        if value.inputs.len() > value.slots
            || value.inputs.len() > value.occurrences
            || value.occurrences == 0
        {
            return Err(MathError::Limit("portable direct input profile"));
        }
        Ok(value)
    }
}
#[test]
#[cfg(test)]
fn historical_recipe_interpretation_refuses_before_decoding_old_provider_shape() {
    let historical = br#"{"interpretation":"pse.admitted-body.v2","providers":{"spec":{"shapes":[],"derivatives":0}}}"#;
    assert!(
        matches!(Recipe::decode(historical), Err(MathError::Contract(reason))
        if reason == "unsupported admitted recipe interpretation")
    );
}
#[test]
#[cfg(test)]
fn native_body_spec_preserves_exact_recipe_json_layout_and_strict_fields() {
    let hash = |byte| ContentHash::from_bytes([byte; 32]);
    let spec = BodySpec {
        definition: hash(1),
        structure: hash(2),
        physical: hash(3),
        admissions: hash(4),
        providers: vec![hash(5), hash(6)],
        policy: hash(7),
    };
    // Independently spell the retired six-field recipe record's exact emitted order.
    let expected = format!(
        "{{\"definition\":\"{}\",\"structure\":\"{}\",\"physical\":\"{}\",\"admissions\":\"{}\",\"providers\":[\"{}\",\"{}\"],\"policy\":\"{}\"}}",
        hash(1).to_prefixed(),
        hash(2).to_prefixed(),
        hash(3).to_prefixed(),
        hash(4).to_prefixed(),
        hash(5).to_prefixed(),
        hash(6).to_prefixed(),
        hash(7).to_prefixed(),
    );
    assert_eq!(serde_json::to_vec(&spec).unwrap(), expected.as_bytes());
    assert_eq!(serde_json::from_str::<BodySpec>(&expected).unwrap(), spec);
    let unknown = expected.strip_suffix('}').unwrap().to_owned() + ",\"extra\":0}";
    assert!(serde_json::from_str::<BodySpec>(&unknown).is_err());
}
/// Capture complete owner records transiently, then retain only the exact encoded allocation.
pub(super) fn capture(
    request: &Request<'_>,
    body: &AdmittedBody,
    receipts: pse_quantity::resolved::receipts::ReceiptStream,
    proofs: pse_math::typed::receipts::ProofStream,
) -> Result<Arc<[u8]>, MathError> {
    let syntax =
        ResolvedRecord::capture(request.syntax, request.limits.occurrences).map_err(contract)?;
    let recipe = Recipe {
        interpretation: INTERPRETATION.into(),
        semantic_identity: None,
        syntax,
        support: SupportRecord::capture(request.support).map_err(contract)?,
        providers: SelectedProvidersRecord::capture(request.providers),
        inputs: request.inputs.to_vec(),
        physical: request.physical,
        structure: request.structure,
        slots: request.limits.slots,
        occurrences: request.limits.occurrences,
        outputs: body.quantities.clone(),
        checked_members: body.math.checked_members().clone(),
        checked_sources: body.math.checked_sources().clone(),
        occurrence_tokens: body.math.occurrence_tokens().clone(),
        receipts,
        proofs,
        spec: body.spec.clone(),
    };
    recipe.encode()
}
/// Attach the final compiler-issued semantic identity. The immutable core remains exact;
/// decode and output allocations are both bounded by the fixed recipe byte admission.
pub(super) fn attach_identity(
    payload: &[u8],
    identity: pse_ids::roles::SemanticBodyHash,
) -> Result<Arc<[u8]>, MathError> {
    let mut recipe = Recipe::decode(payload)?;
    recipe.semantic_identity = Some(identity);
    recipe.encode()
}
/// Versioned payload digest, qualified by the canonical product owner before replay.
pub fn portable_payload_hash(payload: &[u8]) -> ContentHash {
    pse_ids::scientific_replay::admitted_recipe_payload_hash(payload)
}
/// Reconstruct an exact caller-qualified immutable product. Restored model/provider
/// evidence cannot observe or manufacture a mutable preparation context.
pub fn reconstruct_portable(
    permit: &pse_ids::scientific_replay::QualifiedScientificRecipe,
    registry: &QuantityRegistry,
    cancelled: &Arc<AtomicBool>,
) -> Result<AdmittedBody, MathError> {
    let payload = permit.payload();
    let recipe = Recipe::decode(payload)?;
    let expected = &recipe.spec;
    let limits = BodyLimits {
        slots: recipe.slots,
        occurrences: recipe.occurrences,
    };
    let mut body =
        pse_quantity::resolved::receipts::replay(&recipe.receipts, permit.records(), || {
            let syntax = recipe
                .syntax
                .restore(limits.occurrences)
                .map_err(contract)?;
            let support = recipe.support.restore(&syntax).map_err(contract)?;
            let providers = recipe.providers.restore(&syntax, &support)?;
            let request = Request {
                definition: SemanticId::NIL,
                syntax: &syntax,
                inputs: &recipe.inputs,
                support: &support,
                providers: &providers,
                physical: recipe.physical,
                structure: recipe.structure,
                limits,
            };
            pse_math::typed::receipts::replay(&recipe.proofs, permit.records(), || {
                request.admit_inner(registry, &pse_quantity::infer::NoInvariantFacts, cancelled)
            })
        })?;
    if &body.spec != expected
        || body.quantities != recipe.outputs
        || body.math.checked_members() != &recipe.checked_members
        || body.math.checked_sources() != &recipe.checked_sources
        || body.math.occurrence_tokens() != &recipe.occurrence_tokens
    {
        return Err(contract(
            "reconstructed direct body differs from its exact admitted specification or source-role inventory",
        ));
    }
    body.semantic_identity = recipe.semantic_identity;
    // The authority session has drained. Retain only immutable exact bytes, no permit,
    // mutable context, deserialized syntax/support pool or reconstruction authority.
    body.portable = Some(Arc::from(payload));
    Ok(body)
}
#[cfg(test)]
#[allow(
    unsafe_code,
    reason = "controlled compiler fixtures qualify authentic and deliberately corrupt immutable recipes"
)]
pub(super) fn reconstruct_fixture(
    payload: &[u8],
    digest: ContentHash,
    spec: &BodySpec,
    registry: &QuantityRegistry,
    cancelled: &Arc<AtomicBool>,
) -> Result<AdmittedBody, MathError> {
    // SAFETY: private immutable compiler/store test fixture; public callers cannot issue this permit.
    let permit = unsafe {
        pse_ids::scientific_replay::QualifiedScientificRecipe::from_canonical_product(
            payload.to_vec(),
            digest,
            ContentHash::from_bytes([1; 32]),
            ContentHash::from_bytes([2; 32]),
        )
    }
    .map_err(contract)?;
    let body = reconstruct_portable(&permit, registry, cancelled)?;
    if body.spec() != spec {
        return Err(contract("fixture admitted specification differs"));
    }
    Ok(body)
}

#[cfg(test)]
#[test]
#[allow(
    unsafe_code,
    reason = "controlled compiler fixture qualifies actual admitted support solely to test live-context refusal"
)]
fn region_composition_refuses_qualified_restored_support_as_live_context() {
    use pse_modeling::resolved::{RegionOutput, RegionRequest, SelectedRegion};
    let inputs = crate::authored_transfer_tests::context();
    let scope = pse_modeling::PhysicalScope::default();
    let context = pse_modeling::TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: &inputs.quantities,
        preconditions: &inputs.preconditions,
        scope: &scope,
    };
    let declarations = pse_authoring::language::parse(
        "package p {def Root {var x:Scalar; eq e:x==2;}}",
        SemanticId::from_bytes([146; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = pse_modeling::check(&declarations, &context).unwrap();
    let model = pse_modeling::specialize(
        &package,
        root,
        pse_modeling::specialize::root_instance(root),
        &pse_modeling::Bindings::default(),
        pse_modeling::Limits::default(),
    )
    .unwrap();
    let relation = model.equation_roots[&model.equations[0].id];
    let mut region = model
        .prepare_region(
            RegionRequest {
                outputs: &[RegionOutput::RelationResidual(relation)],
                validity: &[],
            },
            &context,
        )
        .unwrap();
    let declaration = declarations
        .iter()
        .find(|row| row.name == "x")
        .unwrap()
        .declaration_id;
    let ports = vec![
        model
            .symbols
            .values()
            .find(|symbol| {
                symbol.lineage.declaration == declaration
                    && symbol.lineage.instance == pse_modeling::specialize::root_instance(root)
            })
            .unwrap(),
    ];
    assert!(SelectedRegion::combine(std::slice::from_ref(&region), &ports, &context).is_ok());
    let record = SupportRecord::capture(&region.support).unwrap();
    use pse_contract::boundary_projection::{ConstructionRole, ProjectionGraph, ProjectionRecord};
    let original_record = serde_json::to_vec(&record).unwrap();
    let record = SupportRecord::from_projection_fields(record.into_projection_fields()).unwrap();
    assert_eq!(serde_json::to_vec(&record).unwrap(), original_record);
    let graph = ProjectionGraph::of::<SupportRecord>().unwrap();
    for name in [
        "ModelingSupportRecord",
        "ModelingSupportNodeRecord",
        "ModelingBodyInputRecord",
        "ModelingDerivedRelationship",
        "PhysicalOutputBoundaryRecord",
    ] {
        assert_eq!(
            graph.definitions()[name].construction,
            ConstructionRole::Data
        );
    }
    let (_, stream) =
        pse_quantity::resolved::receipts::capture(|| Ok::<_, pse_quantity::QuantityError>(()))
            .unwrap();
    let payload = serde_json::to_vec(&(&stream, &record)).unwrap();
    let digest = pse_ids::scientific_replay::admitted_recipe_payload_hash(&payload);
    // SAFETY: This private compiler fixture retains the complete immutable record
    // of actual successful source admission. It cannot issue production authority.
    let recipe = unsafe {
        pse_ids::scientific_replay::QualifiedScientificRecipe::from_admitted_process_message(
            payload,
            digest,
            ContentHash::from_bytes([1; 32]),
            ContentHash::from_bytes([2; 32]),
        )
    }
    .unwrap();
    region.support = pse_quantity::resolved::receipts::replay(&stream, recipe.records(), || {
        Ok::<_, pse_quantity::QuantityError>(record.restore(&region.syntax))
    })
    .unwrap()
    .unwrap();
    assert!(region.support.premises().is_err());
    let error = SelectedRegion::combine(&[region], &ports, &context).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("qualified restored support cannot supply live preparation context"),
        "{error}"
    );
}

#[cfg(test)]
#[path = "typed_math_quantity_record_tests.rs"]
mod quantity_record_tests;
