// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Frozen hexadecimal vectors for every hashing contract this crate owns
//! (blueprint §5.1, §5.3; ADR-0007, ADR-0030, ADR-0045).
//!
//! These are the contract, not a regression net. A change to any value below is a change
//! to identity: every stored `logical_hash` and derived entity ID computed
//! under the old value becomes unreachable, which is why ADR-0045 needed a `v2` rather
//! than a repair of `v1`. When one of these assertions fails the question is never "what
//! is the new value" but "which contract changed, and where is its decision record".
//!
//! Every input is a literal byte pattern or a literal string, so an independent
//! implementation can reproduce the whole table from the recorded rules — keyed
//! `derive_key`, `u64` little-endian length prefixes, first 16 XOF bytes for an ID — with
//! nothing from this repository but the numbers.

use pse_ids::{
    CANONICAL_F32_NAN_BITS, CANONICAL_F64_NAN_BITS, Frame, FramedHasher, SemanticId,
    canonical_f32_bits, canonical_f64_bits, derive_hash, derive_id, encoding_checksum, named_id,
};

// ------------------------------------------------------- frozen identity vectors --

/// `derive_id("pse:named:v1", [[0x00; 16], "pse.schema"])`, which is also
/// `named_id(SemanticId::NIL, "pse.schema")`: the registry package ID.
const REGISTRY_PACKAGE_ID: &str = "a409aa6be295e9f374cf4b829f193f49";

/// `named_id(REGISTRY_PACKAGE_ID, "relation:authored.stoichiometry@1")`.
const STOICHIOMETRY_RELATION_ID: &str = "4af05b3e65e854a58198776b3cddaa8d";

/// `derive_hash("pse:registry:v1", [b"a", b"bc"])`.
const REGISTRY_HASH_A_BC: &str = "e1745e73a5c5b0a583f4cf13428c2e7696e97101b6a4abb64a8ddfb83969a8b6";

/// `derive_id("pse.quantity.unit-product.v1", [2u64 LE, [0x01; 16], 1i16 LE, 1i16 LE,
/// [0x02; 16], -1i16 LE, 1i16 LE])`: the product of unit `0x01…` and the inverse of
/// unit `0x02…` (ADR-0124). `pse_quantity::unit::unit_product_id` frames exactly these
/// parts.
const UNIT_PRODUCT_ID: &str = "b65b26cc780fb634de036e1195b0af54";

/// `derive_hash(frame, [b"pse"])` for each frame variant Plan 23 KR3 added because its
/// preimage changed (ADR-0123 Outcome 8, DP-24): the structured source revision, and every
/// frame over canonical DSL spellings, whose unit literals print as canonical products
/// since ADR-0124. The source revision's preimage layout is pinned beside its derivation
/// (`pse_runtime::math::modeling::source_revision`).
/// `ModelingFiniteFunctionV2` was one of them until KR4 replaced it (below).
const STRUCTURED_IR_FRAMES: [(Frame, &str, &str); 7] = [
    (
        Frame::ModelingSourceRevisionV2,
        "pse.modeling.source-revision.v2",
        "c3010bcfb34611415f9c0ae3ab5c73f613b662b487166178438e733c4ff2653e",
    ),
    (
        Frame::ModelingDispatchBodyV2,
        "pse.modeling.dispatch-body.v2",
        "c66db81d5fbbd304c0d42959b432acd177062882366c6c2a9caf3ff22cadac20",
    ),
    (
        Frame::ModelingContinuityV2,
        "pse.modeling.continuity.v2",
        "b0ca5cab65746987fcd342b5324b658ade81aab12a3afde94b8de3d10788045d",
    ),
    (
        Frame::ModelingDefiniteIntegralV2,
        "pse.modeling.definite-integral.v2",
        "8d13406a0ebb9964552fd1f913071fdad985a16898132ef4ec350db304c0f335",
    ),
    (
        Frame::ModelingConsumerBodyV2,
        "pse.modeling.consumer-body.v2",
        "5610f7db50b756a20b7a5253db954773c1028e8d1506b0a035138b0d5b179e59",
    ),
    (
        Frame::ModelingImplicitResidualV2,
        "pse.modeling.implicit-residual.v2",
        "b60062d9ef33991edb88ead0ccd261a0e13316aeeeba1a6262d8943c83a5abd8",
    ),
    (
        Frame::MathTypedDefinitionV3,
        "pse.math.typed-definition.v3",
        "ab5d37e55b80a098e63ea38db0d0106cd5d1dba776b1d088482abfe924d1aaa7",
    ),
];

/// `derive_hash(frame, [b"pse"])` for each frame variant Plan 23 KR4 added (ADR-0123
/// Outcome 2, DP-24): the keyed entity identity, and the coordinate and function
/// specialization frames, whose preimages now frame an enumeration member by identity.
const ENTITY_RECORD_FRAMES: [(Frame, &str, &str); 3] = [
    (
        Frame::ModelingKeyedEntityV1,
        "pse.modeling.keyed-entity.v1",
        "f3d2e7de042febbb62d1c903e4e85b3b50bfe077afc12dee8fe59183d7490307",
    ),
    (
        Frame::ModelingCoordinateV2,
        "pse.modeling.coordinate.v2",
        "efff8910923a963ad18268dc4dfb8ba9238d08a9c32dbcfdc183e441ff22495f",
    ),
    (
        Frame::ModelingFiniteFunctionV3,
        "pse.modeling.finite-function.v3",
        "3ddbbe58c760b5d63816de0e66aeea4ec8968bb59d1e5a52b2a067109821ef6f",
    ),
];

/// `derive_id("pse.modeling.keyed-entity.v1", [[0x01; 16], 2u64 LE, "entity", [0x02; 16],
/// "int", 1i64 LE])`: the identity of a row whose key-declaring kind is `0x01…` and whose
/// two keys are the entity `0x02…` and the integer 1. `pse_modeling` frames exactly these
/// parts (`entity::keyed_identity`).
const KEYED_ENTITY_ID: &str = "fc6aa818a9891d4188e2058bca760892";

/// `encoding_checksum(b"pse")`: plain, unkeyed BLAKE3 over three bytes.
const ENCODING_CHECKSUM_PSE: &str =
    "b183159a276933fcc7170f73b6e21ff751344a2769b9669736ff4ffa47c689ee";

// --------------------------------------------------------------------- the table --

#[test]
fn the_registry_package_id_is_frozen() {
    let package = named_id(SemanticId::NIL, "pse.schema");
    assert_eq!(package.to_hex(), REGISTRY_PACKAGE_ID);

    // `named_id` is exactly `derive_id(NAMED, [package bytes, name bytes])`.
    assert_eq!(
        derive_id(Frame::NamedV1, &[SemanticId::NIL.as_bytes(), b"pse.schema"]),
        package
    );

    assert_eq!(
        named_id(package, "relation:authored.stoichiometry@1").to_hex(),
        STOICHIOMETRY_RELATION_ID
    );
}

#[test]
fn the_registry_digest_is_frozen() {
    assert_eq!(
        derive_hash(Frame::RegistryV1, &[b"a", b"bc"]).to_hex(),
        REGISTRY_HASH_A_BC
    );

    // The framing this table depends on: a different split is a different digest.
    assert_ne!(
        derive_hash(Frame::RegistryV1, &[b"ab", b"c"]).to_hex(),
        REGISTRY_HASH_A_BC
    );
}

#[test]
fn the_unit_product_identity_is_frozen() {
    let parts: &[&[u8]] = &[
        &2_u64.to_le_bytes(),
        &[0x01; 16],
        &1_i16.to_le_bytes(),
        &1_i16.to_le_bytes(),
        &[0x02; 16],
        &(-1_i16).to_le_bytes(),
        &1_i16.to_le_bytes(),
    ];
    assert_eq!(
        derive_id(Frame::QuantityUnitProductV1, parts).to_hex(),
        UNIT_PRODUCT_ID
    );
    assert_eq!(
        Frame::QuantityUnitProductV1.as_str(),
        "pse.quantity.unit-product.v1"
    );
}

#[test]
fn the_structured_ir_frame_variants_are_frozen() {
    for (frame, spelling, vector) in STRUCTURED_IR_FRAMES {
        assert_eq!(frame.as_str(), spelling);
        assert_eq!(
            derive_hash(frame, &[b"pse"]).to_hex(),
            vector,
            "{spelling}"
        );
    }
}

#[test]
fn the_entity_record_frame_variants_are_frozen() {
    for (frame, spelling, vector) in ENTITY_RECORD_FRAMES {
        assert_eq!(frame.as_str(), spelling);
        assert_eq!(
            derive_hash(frame, &[b"pse"]).to_hex(),
            vector,
            "{spelling}"
        );
    }
    let parts: &[&[u8]] = &[
        &[0x01; 16],
        &2_u64.to_le_bytes(),
        b"entity",
        &[0x02; 16],
        b"int",
        &1_i64.to_le_bytes(),
    ];
    assert_eq!(
        derive_id(Frame::ModelingKeyedEntityV1, parts).to_hex(),
        KEYED_ENTITY_ID
    );
    // The framed hasher writes the same parts.
    let mut h = FramedHasher::new(Frame::ModelingKeyedEntityV1);
    h.id(&SemanticId::from_bytes([0x01; 16]))
        .u64(2)
        .str("entity")
        .id(&SemanticId::from_bytes([0x02; 16]))
        .str("int")
        .part(&1_i64.to_le_bytes());
    assert_eq!(h.finish_id().to_hex(), KEYED_ENTITY_ID);
}

#[test]
fn a_derived_id_is_the_prefix_of_the_derived_hash_under_one_context() {
    // BLAKE3's `finalize` is the first 32 bytes of its extendable output, so a 128-bit ID
    // is the first 16 bytes of the 256-bit digest under the same context and parts. This
    // is a property of the algorithm, recorded so a future implementation does not "fix"
    // one of the two functions into disagreement.
    let parts: &[&[u8]] = &[b"a", b"bc"];
    let id = derive_id(Frame::RegistryV1, parts);
    assert_eq!(
        id.to_hex(),
        REGISTRY_HASH_A_BC
            .get(..32)
            .expect("a 64-digit frozen vector")
    );
}

#[test]
fn the_framed_hasher_reproduces_the_frozen_vectors() {
    let mut hasher = FramedHasher::new(Frame::NamedV1);
    hasher.id(&SemanticId::NIL).str("pse.schema");
    assert_eq!(hasher.finish_id().to_hex(), REGISTRY_PACKAGE_ID);

    let mut digest = FramedHasher::new(Frame::RegistryV1);
    digest.part(b"a").part(b"bc");
    assert_eq!(digest.finish_hash().to_hex(), REGISTRY_HASH_A_BC);
}

#[test]
fn the_derive_key_contexts_are_frozen() {
    assert_eq!(Frame::NamedV1.as_str(), "pse:named:v1");
    assert_eq!(Frame::RegistryV1.as_str(), "pse:registry:v1");
    assert_eq!(Frame::SettingsV1.as_str(), "pse:settings:v1");
}

/// The frame spellings captured at the start of Plan 22 B3a, before the catalog existed.
const CAPTURED_FRAME_SPELLINGS: &str = include_str!("frame_spellings.txt");

#[test]
fn frame_spellings_unchanged() {
    // Every frame in the catalog is a spelling that was in use before it, and every
    // spelling that was in use is in the catalog: no identity moved to a new context.
    let captured: std::collections::BTreeSet<&str> = CAPTURED_FRAME_SPELLINGS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let cataloged: std::collections::BTreeSet<&str> =
        Frame::ALL.iter().map(|frame| frame.as_str()).collect();
    assert_eq!(
        cataloged.difference(&captured).collect::<Vec<_>>(),
        Vec::<&&str>::new(),
        "frames that were not in use before the catalog"
    );
    assert_eq!(
        captured.difference(&cataloged).collect::<Vec<_>>(),
        Vec::<&&str>::new(),
        "spellings in use before the catalog that it does not declare"
    );
    assert_eq!(
        cataloged.len(),
        Frame::ALL.len(),
        "a spelling is declared twice"
    );
}

#[test]
fn the_encoding_checksum_is_frozen() {
    assert_eq!(encoding_checksum(b"pse").0.to_hex(), ENCODING_CHECKSUM_PSE);
    assert_eq!(
        encoding_checksum(b"pse").0.to_prefixed(),
        format!("blake3:{ENCODING_CHECKSUM_PSE}")
    );

    // The logical and encoded roles never share a value by construction: the checksum of
    // three bytes is not the identity of anything derived under a key.
    assert_ne!(encoding_checksum(b"pse").0.to_hex(), REGISTRY_HASH_A_BC);
}

#[test]
fn the_canonical_float_bits_are_frozen() {
    assert_eq!(CANONICAL_F64_NAN_BITS, 0x7ff8_0000_0000_0000);
    assert_eq!(CANONICAL_F32_NAN_BITS, 0x7fc0_0000);

    // Every NaN payload and sign collapses to the one pattern (ADR-0030).
    for bits in [
        0x7ff8_0000_0000_0000_u64,
        0x7ff8_0000_0000_0001,
        0x7ff0_0000_0000_0001,
        0xfff8_0000_0000_0000,
        0xfff8_dead_beef_cafe,
        0xffff_ffff_ffff_ffff,
    ] {
        let value = f64::from_bits(bits);
        assert!(value.is_nan(), "{bits:#018x} is not a NaN");
        assert_eq!(canonical_f64_bits(value), CANONICAL_F64_NAN_BITS);
    }
    for bits in [
        0x7fc0_0000_u32,
        0x7fc0_0001,
        0x7f80_0001,
        0xffc0_0000,
        0xffff_ffff,
    ] {
        let value = f32::from_bits(bits);
        assert!(value.is_nan(), "{bits:#010x} is not a NaN");
        assert_eq!(canonical_f32_bits(value), CANONICAL_F32_NAN_BITS);
    }

    // Signed zero survives, and the two zeros stay apart.
    assert_eq!(canonical_f64_bits(-0.0), 0x8000_0000_0000_0000);
    assert_eq!(canonical_f64_bits(0.0), 0x0000_0000_0000_0000);
    assert_eq!(canonical_f32_bits(-0.0), 0x8000_0000);
    assert_eq!(canonical_f32_bits(0.0), 0x0000_0000);

    // A few ordinary values, to pin that nothing else is touched.
    assert_eq!(canonical_f64_bits(1.0), 0x3ff0_0000_0000_0000);
    assert_eq!(canonical_f64_bits(f64::INFINITY), 0x7ff0_0000_0000_0000);
    assert_eq!(canonical_f64_bits(f64::NEG_INFINITY), 0xfff0_0000_0000_0000);
    assert_eq!(canonical_f32_bits(1.0), 0x3f80_0000);
}
