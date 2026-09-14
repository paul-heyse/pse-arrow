// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Derived identity: the keyed, framed hashing of blueprint §5.1.
//!
//! `blake3_128(context ‖ parts)` in the blueprint denotes exactly this:
//!
//! 1. `blake3::Hasher::new_derive_key(context)` — a *keyed* hash, not a plain one.
//!    `derive_key` and a truncated plain hash produce different bytes, so the choice is
//!    frozen together with the `v1` context strings in [`context`].
//! 2. Every part is framed as its `u64` little-endian byte length followed by its bytes.
//!    IDs, hashes and integers are parts too and carry a length like everything else, so
//!    `["ab", "c"]` and `["a", "bc"]` cannot collide.
//! 3. A 128-bit identity is the first 16 bytes of `finalize_xof()`: a defined digest of
//!    BLAKE3's extendable output, not a truncation of an unrelated hash. A 256-bit hash is
//!    `finalize()`.
//!
//! Consequence, and the reason any of this is worth the framing: re-running compilation on
//! an unchanged snapshot reproduces identical IDs, and changing mesh resolution changes
//! only the mesh-dependent ones.

use crate::id::{ContentHash, Ordinal, SemanticId};

/// The frozen `derive_key` context strings (blueprint §5.1, §5.3, ADR-0050).
///
/// A context string is part of the identity contract: changing one changes every ID
/// derived under it, so a new meaning gets a new string rather than a new meaning for an
/// old one. The `v1` suffix versions the *derivation*, not the relation schema.
pub mod context {
    /// Named-policy entity identity: `named_id(package_id, qualified_name)`.
    pub const NAMED: &str = "pse:named:v1";
    /// A symbol instance, and a discretized symbol, of a compiled model.
    pub const SYMBOL: &str = "pse:symbol:v1";
    /// An equation instance of a compiled model.
    pub const EQUATION: &str = "pse:equation:v1";
    /// A law-expanded contribution term.
    pub const TERM: &str = "pse:term:v1";
    /// A connection equation.
    pub const CONN: &str = "pse:conn:v1";
    /// A mesh node.
    pub const NODE: &str = "pse:node:v1";
    /// Registry identity and the registry fingerprint.
    pub const REGISTRY: &str = "pse:registry:v1";
    /// A pass stage memo key (blueprint §14.3).
    pub const STAGE_KEY: &str = "pse:stage_key:v1";
    /// An engine settings or profile digest (blueprint §14.3, §23.2).
    pub const SETTINGS: &str = "pse:settings:v1";
    /// A math IR structural node hash (blueprint §7.4, ADR-0047).
    pub const MATHIR_NODE: &str = "pse:mathir:node:v1";
}

/// Frames one part into a hasher: `u64` little-endian length, then the bytes.
///
/// `usize` is at most 64 bits on every supported target, so the saturating fallback is
/// unreachable; it exists because the crate's panic policy has no room for an `expect`
/// that only a 128-bit address space could reach.
fn put_part(hasher: &mut blake3::Hasher, part: &[u8]) {
    let len = u64::try_from(part.len()).unwrap_or(u64::MAX);
    hasher.update(&len.to_le_bytes());
    hasher.update(part);
}

/// Derives a 128-bit semantic ID under `context` from framed `parts` (blueprint §5.1).
///
/// ```
/// use pse_ids::{derive::context, derive_id, SemanticId};
///
/// // Framing is what keeps a split from being a collision.
/// assert_ne!(
///     derive_id(context::NAMED, &[b"ab", b"c"]),
///     derive_id(context::NAMED, &[b"a", b"bc"]),
/// );
/// ```
pub fn derive_id(context: &'static str, parts: &[&[u8]]) -> SemanticId {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    for part in parts {
        put_part(&mut hasher, part);
    }
    let mut bytes = [0_u8; SemanticId::WIDTH];
    hasher.finalize_xof().fill(&mut bytes);
    SemanticId::from_bytes(bytes)
}

/// Derives a 256-bit content hash under `context` from framed `parts`.
///
/// Used for registry fingerprints, stage keys, settings digests and math IR node hashes —
/// every keyed digest that is not an entity identity.
pub fn derive_hash(context: &'static str, parts: &[&[u8]]) -> ContentHash {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    for part in parts {
        put_part(&mut hasher, part);
    }
    ContentHash::from_bytes(*hasher.finalize().as_bytes())
}

/// An incremental [`derive_id`] / [`derive_hash`] for callers that build a preimage from
/// heterogeneous components.
///
/// Every method contributes exactly one framed part, so a `FramedHasher` sequence and the
/// equivalent `parts` slice produce the same digest. Integers are contributed as their
/// little-endian bytes *inside* a length-prefixed part, which is what keeps this
/// equivalent to the slice form rather than a second, subtly different framing.
///
/// ```
/// use pse_ids::{derive::context, derive_hash, FramedHasher};
///
/// let mut hasher = FramedHasher::new(context::SETTINGS);
/// hasher.str("target_partitions").u64(8);
///
/// let expected = derive_hash(context::SETTINGS, &[b"target_partitions", &8_u64.to_le_bytes()]);
/// assert_eq!(hasher.finish_hash(), expected);
/// ```
#[derive(Clone, Debug)]
pub struct FramedHasher {
    hasher: blake3::Hasher,
}

impl FramedHasher {
    /// Opens a hasher keyed with `context`.
    pub fn new(context: &'static str) -> Self {
        Self {
            hasher: blake3::Hasher::new_derive_key(context),
        }
    }

    /// One part: the bytes, length-prefixed.
    pub fn part(&mut self, bytes: &[u8]) -> &mut Self {
        put_part(&mut self.hasher, bytes);
        self
    }

    /// One part: the UTF-8 bytes of `text`, length-prefixed.
    pub fn str(&mut self, text: &str) -> &mut Self {
        self.part(text.as_bytes())
    }

    /// One part: two little-endian bytes, length-prefixed.
    pub fn u16(&mut self, v: u16) -> &mut Self {
        self.part(&v.to_le_bytes())
    }

    /// One part: four little-endian bytes, length-prefixed.
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.part(&v.to_le_bytes())
    }

    /// One part: eight little-endian bytes, length-prefixed.
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.part(&v.to_le_bytes())
    }

    /// One part: a single byte, `1` for true and `0` for false, length-prefixed.
    pub fn bool(&mut self, v: bool) -> &mut Self {
        self.part(&[u8::from(v)])
    }

    /// One part: the sixteen raw bytes of an identity, length-prefixed.
    pub fn id(&mut self, id: &SemanticId) -> &mut Self {
        self.part(id.as_bytes())
    }

    /// One part: the thirty-two raw bytes of a digest, length-prefixed.
    pub fn hash(&mut self, hash: &ContentHash) -> &mut Self {
        self.part(hash.as_bytes())
    }

    /// Finishes as a 256-bit content hash.
    pub fn finish_hash(self) -> ContentHash {
        ContentHash::from_bytes(*self.hasher.finalize().as_bytes())
    }

    /// Finishes as a 128-bit semantic ID: the first sixteen bytes of the extendable output.
    pub fn finish_id(self) -> SemanticId {
        let mut bytes = [0_u8; SemanticId::WIDTH];
        self.hasher.finalize_xof().fill(&mut bytes);
        SemanticId::from_bytes(bytes)
    }
}

/// The index tuple that distinguishes instances of one declaration (blueprint §5.1).
///
/// It contributes **one** part whose bytes are the members' raw bytes concatenated, so the
/// tuple's arity is carried by the part's length and a member boundary is never ambiguous
/// (every member is exactly [`SemanticId::WIDTH`] bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexTuple<'a>(pub &'a [SemanticId]);

impl IndexTuple<'_> {
    /// The empty tuple: a declaration with no free index.
    pub const EMPTY: IndexTuple<'static> = IndexTuple(&[]);

    /// The concatenated raw member bytes, which is the single part this tuple frames as.
    pub fn to_bytes(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.0.len() * SemanticId::WIDTH);
        for member in self.0 {
            out.extend_from_slice(member.as_bytes());
        }
        out
    }

    /// The number of members.
    pub fn len(self) -> usize {
        self.0.len()
    }

    /// Whether the tuple has no members.
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }
}

/// The identity of a named-policy entity: `blake3_128("pse:named:v1" ‖ package_id ‖ name)`.
///
/// Named policy is for reference packages whose qualified names *are* the public contract
/// (units, elements, constants, property kinds, standard templates and methods). Under it
/// a rename is by definition a new entity, which is why `change_ops.op = rename` is
/// rejected for such a package and the old name becomes a `reference.aliases` row.
///
/// ```
/// use pse_ids::{named_id, SemanticId};
///
/// let package = named_id(SemanticId::NIL, "pse.schema");
/// // A rename under the named policy is a different entity, by construction.
/// assert_ne!(named_id(package, "unit:kelvin"), named_id(package, "unit:degK"));
/// ```
pub fn named_id(package_id: SemanticId, qualified_name: &str) -> SemanticId {
    derive_id(
        context::NAMED,
        &[package_id.as_bytes(), qualified_name.as_bytes()],
    )
}

/// A symbol instance of a compiled model (blueprint §5.1).
pub fn symbol_instance_id(
    instance_id: SemanticId,
    symbol_decl_id: SemanticId,
    index: IndexTuple<'_>,
) -> SemanticId {
    derive_id(
        context::SYMBOL,
        &[
            instance_id.as_bytes(),
            symbol_decl_id.as_bytes(),
            &index.to_bytes(),
        ],
    )
}

/// An equation instance of a compiled model (blueprint §5.1).
pub fn equation_instance_id(
    instance_id: SemanticId,
    equation_decl_id: SemanticId,
    index: IndexTuple<'_>,
) -> SemanticId {
    derive_id(
        context::EQUATION,
        &[
            instance_id.as_bytes(),
            equation_decl_id.as_bytes(),
            &index.to_bytes(),
        ],
    )
}

/// A law-expanded contribution term (blueprint §5.1, §10).
pub fn law_term_id(
    law_instance_id: SemanticId,
    contribution_id: SemanticId,
    index: IndexTuple<'_>,
) -> SemanticId {
    derive_id(
        context::TERM,
        &[
            law_instance_id.as_bytes(),
            contribution_id.as_bytes(),
            &index.to_bytes(),
        ],
    )
}

/// A connection equation (blueprint §5.1).
///
/// The member ordinal is a *declared* position within the connection, not a row position,
/// and frames as its eight little-endian bytes.
pub fn connection_equation_id(
    connection_id: SemanticId,
    member_ordinal: Ordinal,
    index: IndexTuple<'_>,
) -> SemanticId {
    derive_id(
        context::CONN,
        &[
            connection_id.as_bytes(),
            &member_ordinal.to_le_bytes(),
            &index.to_bytes(),
        ],
    )
}

/// A mesh node of a discretized domain (blueprint §5.1, §12).
pub fn mesh_node_id(
    domain_id: SemanticId,
    policy_id: SemanticId,
    node_ordinal: Ordinal,
) -> SemanticId {
    derive_id(
        context::NODE,
        &[
            domain_id.as_bytes(),
            policy_id.as_bytes(),
            &node_ordinal.to_le_bytes(),
        ],
    )
}

/// A symbol discretized onto a mesh node (blueprint §5.1).
///
/// Shares the `pse:symbol:v1` context with [`symbol_instance_id`]; the two cannot collide
/// because they frame a different number of parts.
pub fn discretized_symbol_id(parent_symbol_id: SemanticId, node_id: SemanticId) -> SemanticId {
    derive_id(
        context::SYMBOL,
        &[parent_symbol_id.as_bytes(), node_id.as_bytes()],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: SemanticId = SemanticId::from_bytes([0x01; 16]);
    const B: SemanticId = SemanticId::from_bytes([0x02; 16]);
    const C: SemanticId = SemanticId::from_bytes([0x03; 16]);
    const D: SemanticId = SemanticId::from_bytes([0x04; 16]);

    #[test]
    fn framing_distinguishes_a_split_that_concatenation_would_not() {
        assert_ne!(
            derive_id(context::REGISTRY, &[b"ab", b"c"]),
            derive_id(context::REGISTRY, &[b"a", b"bc"])
        );
        assert_ne!(
            derive_hash(context::REGISTRY, &[b"ab", b"c"]),
            derive_hash(context::REGISTRY, &[b"a", b"bc"])
        );
    }

    #[test]
    fn an_empty_part_is_not_no_part() {
        assert_ne!(
            derive_hash(context::REGISTRY, &[b"a", b""]),
            derive_hash(context::REGISTRY, &[b"a"])
        );
    }

    #[test]
    fn a_derived_id_is_the_first_sixteen_bytes_of_the_extendable_output() {
        let parts: &[&[u8]] = &[b"pse.schema", b"relation:authored.packages@1"];
        let mut hasher = blake3::Hasher::new_derive_key(context::NAMED);
        for part in parts {
            put_part(&mut hasher, part);
        }
        let mut expected = [0_u8; 16];
        hasher.finalize_xof().fill(&mut expected);

        assert_eq!(derive_id(context::NAMED, parts).as_bytes(), &expected);
    }

    #[test]
    fn a_changed_context_changes_every_derived_value() {
        let parts: &[&[u8]] = &[b"x"];
        assert_ne!(
            derive_id(context::SYMBOL, parts),
            derive_id(context::EQUATION, parts)
        );
        assert_ne!(
            derive_hash(context::REGISTRY, parts),
            derive_hash(context::SETTINGS, parts)
        );
    }

    #[test]
    fn a_keyed_digest_is_not_a_plain_one() {
        let mut plain = blake3::Hasher::new();
        put_part(&mut plain, b"x");
        assert_ne!(
            derive_hash(context::REGISTRY, &[b"x"]).as_bytes(),
            plain.finalize().as_bytes()
        );
    }

    #[test]
    fn the_framed_hasher_equals_the_slice_form_on_the_same_parts() {
        let mut framed = FramedHasher::new(context::STAGE_KEY);
        framed
            .str("P2")
            .u32(1)
            .id(&A)
            .hash(&ContentHash::from_bytes([0x09; 32]))
            .u64(64)
            .u16(7)
            .bool(true)
            .part(b"tail");

        let expected = derive_hash(
            context::STAGE_KEY,
            &[
                b"P2",
                &1_u32.to_le_bytes(),
                A.as_bytes(),
                &[0x09; 32],
                &64_u64.to_le_bytes(),
                &7_u16.to_le_bytes(),
                &[1_u8],
                b"tail",
            ],
        );
        assert_eq!(framed.finish_hash(), expected);
    }

    #[test]
    fn the_framed_hasher_and_derive_id_agree_on_identity_too() {
        let mut framed = FramedHasher::new(context::NAMED);
        framed.id(&SemanticId::NIL).str("pse.schema");
        assert_eq!(framed.finish_id(), named_id(SemanticId::NIL, "pse.schema"));
    }

    #[test]
    fn a_derived_id_is_the_prefix_of_the_derived_hash_under_the_same_context() {
        let parts: &[&[u8]] = &[b"a", b"bc"];
        let id = derive_id(context::REGISTRY, parts);
        let hash = derive_hash(context::REGISTRY, parts);
        assert_eq!(id.as_bytes(), &hash.as_bytes()[..SemanticId::WIDTH]);
    }

    #[test]
    fn an_index_tuple_frames_as_one_concatenated_part() {
        assert_eq!(IndexTuple(&[C, D]).to_bytes().len(), 32);
        assert_eq!(IndexTuple::EMPTY.to_bytes(), Vec::<u8>::new());
        assert!(IndexTuple::EMPTY.is_empty());
        assert_eq!(IndexTuple(&[C, D]).len(), 2);

        // The tuple is one part, so a different arity is a different value.
        assert_ne!(
            symbol_instance_id(A, B, IndexTuple(&[C])),
            symbol_instance_id(A, B, IndexTuple(&[C, D]))
        );
        // ... and member order is identity, not a set.
        assert_ne!(
            symbol_instance_id(A, B, IndexTuple(&[C, D])),
            symbol_instance_id(A, B, IndexTuple(&[D, C]))
        );
    }

    #[test]
    fn the_typed_constructors_do_not_collide_with_each_other() {
        let index = IndexTuple(&[C, D]);
        let derived = vec![
            symbol_instance_id(A, B, index),
            equation_instance_id(A, B, index),
            law_term_id(A, B, index),
            connection_equation_id(A, Ordinal(7), index),
            mesh_node_id(A, B, Ordinal(7)),
            discretized_symbol_id(A, B),
            named_id(A, "x"),
        ];
        let mut unique = derived.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), derived.len());
    }

    #[test]
    fn an_ordinal_is_part_of_a_connection_equation_identity() {
        assert_ne!(
            connection_equation_id(A, Ordinal(0), IndexTuple::EMPTY),
            connection_equation_id(A, Ordinal(1), IndexTuple::EMPTY)
        );
        assert_ne!(
            mesh_node_id(A, B, Ordinal(0)),
            mesh_node_id(A, B, Ordinal(1))
        );
    }
}
