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

use pse_ids::derive::context;
use pse_ids::{
    CANONICAL_F32_NAN_BITS, CANONICAL_F64_NAN_BITS, FramedHasher, IndexTuple, Ordinal, SemanticId,
    canonical_f32_bits, canonical_f64_bits, connection_equation_id, derive_hash, derive_id,
    discretized_symbol_id, encoding_checksum, equation_instance_id, law_term_id, mesh_node_id,
    named_id, symbol_instance_id,
};

// ---------------------------------------------------------------- fixed inputs --

/// Four literal identities: `A = [0x01; 16]` … `D = [0x04; 16]`.
const A: SemanticId = SemanticId::from_bytes([0x01; 16]);
/// See [`A`].
const B: SemanticId = SemanticId::from_bytes([0x02; 16]);
/// See [`A`].
const C: SemanticId = SemanticId::from_bytes([0x03; 16]);
/// See [`A`].
const D: SemanticId = SemanticId::from_bytes([0x04; 16]);

/// The fixed index tuple `(C, D)`.
const INDEX: IndexTuple<'static> = IndexTuple(&[C, D]);

/// The fixed ordinal used by the connection and mesh vectors.
const ORDINAL: Ordinal = Ordinal(7);

// ------------------------------------------------------- frozen identity vectors --

/// `derive_id("pse:named:v1", [[0x00; 16], "pse.schema"])`, which is also
/// `named_id(SemanticId::NIL, "pse.schema")`: the registry package ID.
const REGISTRY_PACKAGE_ID: &str = "a409aa6be295e9f374cf4b829f193f49";

/// `named_id(REGISTRY_PACKAGE_ID, "relation:authored.stoichiometry@1")`.
const STOICHIOMETRY_RELATION_ID: &str = "4af05b3e65e854a58198776b3cddaa8d";

/// `derive_hash("pse:registry:v1", [b"a", b"bc"])`.
const REGISTRY_HASH_A_BC: &str = "e1745e73a5c5b0a583f4cf13428c2e7696e97101b6a4abb64a8ddfb83969a8b6";

/// `symbol_instance_id(A, B, (C, D))`, context `pse:symbol:v1`.
const SYMBOL_INSTANCE: &str = "70dde4aa3dfe04a4886a7b59df389070";

/// `equation_instance_id(A, B, (C, D))`, context `pse:equation:v1`.
const EQUATION_INSTANCE: &str = "a56c9c4f2710d4ee6160789909a9c33a";

/// `law_term_id(A, B, (C, D))`, context `pse:term:v1`.
const LAW_TERM: &str = "bbacd7698075d85247763217db444ee3";

/// `connection_equation_id(A, Ordinal(7), (C, D))`, context `pse:conn:v1`.
const CONNECTION_EQUATION: &str = "19ae7d5c7570254f981df87da0451540";

/// `mesh_node_id(A, B, Ordinal(7))`, context `pse:node:v1`.
const MESH_NODE: &str = "eb00e870f92170e9ac71856dca721e01";

/// `discretized_symbol_id(A, B)`, context `pse:symbol:v1` with two parts.
const DISCRETIZED_SYMBOL: &str = "7d39e0dbf3fea05cb118af04f299257a";

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
        derive_id(context::NAMED, &[SemanticId::NIL.as_bytes(), b"pse.schema"]),
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
        derive_hash(context::REGISTRY, &[b"a", b"bc"]).to_hex(),
        REGISTRY_HASH_A_BC
    );

    // The framing this table depends on: a different split is a different digest.
    assert_ne!(
        derive_hash(context::REGISTRY, &[b"ab", b"c"]).to_hex(),
        REGISTRY_HASH_A_BC
    );
}

#[test]
fn a_derived_id_is_the_prefix_of_the_derived_hash_under_one_context() {
    // BLAKE3's `finalize` is the first 32 bytes of its extendable output, so a 128-bit ID
    // is the first 16 bytes of the 256-bit digest under the same context and parts. This
    // is a property of the algorithm, recorded so a future implementation does not "fix"
    // one of the two functions into disagreement.
    let parts: &[&[u8]] = &[b"a", b"bc"];
    let id = derive_id(context::REGISTRY, parts);
    assert_eq!(
        id.to_hex(),
        REGISTRY_HASH_A_BC
            .get(..32)
            .expect("a 64-digit frozen vector")
    );
}

#[test]
fn every_derived_entity_constructor_is_frozen() {
    let table: Vec<(&str, String, &str)> = vec![
        (
            "symbol_instance_id",
            symbol_instance_id(A, B, INDEX).to_hex(),
            SYMBOL_INSTANCE,
        ),
        (
            "equation_instance_id",
            equation_instance_id(A, B, INDEX).to_hex(),
            EQUATION_INSTANCE,
        ),
        ("law_term_id", law_term_id(A, B, INDEX).to_hex(), LAW_TERM),
        (
            "connection_equation_id",
            connection_equation_id(A, ORDINAL, INDEX).to_hex(),
            CONNECTION_EQUATION,
        ),
        (
            "mesh_node_id",
            mesh_node_id(A, B, ORDINAL).to_hex(),
            MESH_NODE,
        ),
        (
            "discretized_symbol_id",
            discretized_symbol_id(A, B).to_hex(),
            DISCRETIZED_SYMBOL,
        ),
    ];
    for (name, actual, frozen) in table {
        assert_eq!(actual, frozen, "{name} changed the identity contract");
    }
}

#[test]
fn the_typed_constructors_agree_with_their_raw_derivations() {
    // The typed constructors are the readable form of §5.1's formulas, not a second
    // derivation: each one must equal `derive_id` over the same context and parts.
    assert_eq!(
        symbol_instance_id(A, B, INDEX),
        derive_id(
            context::SYMBOL,
            &[A.as_bytes(), B.as_bytes(), &INDEX.to_bytes()]
        )
    );
    assert_eq!(
        equation_instance_id(A, B, INDEX),
        derive_id(
            context::EQUATION,
            &[A.as_bytes(), B.as_bytes(), &INDEX.to_bytes()]
        )
    );
    assert_eq!(
        law_term_id(A, B, INDEX),
        derive_id(
            context::TERM,
            &[A.as_bytes(), B.as_bytes(), &INDEX.to_bytes()]
        )
    );
    assert_eq!(
        connection_equation_id(A, ORDINAL, INDEX),
        derive_id(
            context::CONN,
            &[A.as_bytes(), &ORDINAL.to_le_bytes(), &INDEX.to_bytes()]
        )
    );
    assert_eq!(
        mesh_node_id(A, B, ORDINAL),
        derive_id(
            context::NODE,
            &[A.as_bytes(), B.as_bytes(), &ORDINAL.to_le_bytes()]
        )
    );
    assert_eq!(
        discretized_symbol_id(A, B),
        derive_id(context::SYMBOL, &[A.as_bytes(), B.as_bytes()])
    );
}

#[test]
fn the_framed_hasher_reproduces_the_frozen_vectors() {
    let mut hasher = FramedHasher::new(context::NAMED);
    hasher.id(&SemanticId::NIL).str("pse.schema");
    assert_eq!(hasher.finish_id().to_hex(), REGISTRY_PACKAGE_ID);

    let mut digest = FramedHasher::new(context::REGISTRY);
    digest.part(b"a").part(b"bc");
    assert_eq!(digest.finish_hash().to_hex(), REGISTRY_HASH_A_BC);
}

#[test]
fn the_derive_key_contexts_are_frozen() {
    assert_eq!(context::NAMED, "pse:named:v1");
    assert_eq!(context::SYMBOL, "pse:symbol:v1");
    assert_eq!(context::EQUATION, "pse:equation:v1");
    assert_eq!(context::TERM, "pse:term:v1");
    assert_eq!(context::CONN, "pse:conn:v1");
    assert_eq!(context::NODE, "pse:node:v1");
    assert_eq!(context::REGISTRY, "pse:registry:v1");
    assert_eq!(context::STAGE_KEY, "pse:stage_key:v1");
    assert_eq!(context::SETTINGS, "pse:settings:v1");
    assert_eq!(context::MATHIR_NODE, "pse:mathir:node:v1");
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
