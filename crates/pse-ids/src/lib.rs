// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Identity, framing and accounted allocation (blueprint §5.1, §5.3,
//! §14.3).
//!
//! The only crate in the workspace allowed to depend on `blake3` (governance
//! `blake3_owner`). A second crate hashing on its own would invent a second identity
//! contract, and nothing would tell you which of the two was authoritative — so the
//! dependency itself is fenced rather than the convention documented.
//!
//! # The three hashing contracts, and why they are three
//!
//! | Contract | Keyed? | Framing | Answers |
//! |---|---|---|---|
//! | [`mod@derive`] | `new_derive_key(context)` | every part length-prefixed | "which entity is this?" |
//! | [`canon`] logical hashing | plain | [`FrameSink`], mixed fixed and length-prefixed | "do these two relations mean the same thing?" |
//! | [`encoding`] | plain | none; the finished bytes | "is this stored object intact?" |
//!
//! ADR-0045 separates the second and third because `pse.canon.v1` conflated them and could
//! then neither survive an IPC-file/Parquet round trip nor detect a truncated object under
//! a plausible name. [`id::LogicalHash`], [`id::EncodingChecksum`]
//! are newtypes over the same 32 bytes for the same reason: the compiler refuses the
//! substitution the design refuses.
//!
//! # What never enters a hash
//!
//! A `Debug` or display rendering of anything. Three libraries in the dependency set —
//! Arrow `Schema` metadata, `datafusion-proto` field metadata, egglog's `TermDag` — leak
//! hash-container iteration order through such a rendering, so a value hashed that way is
//! reproducible only by accident (§5.3 step 8). A governance grep enforces this over
//! `crates/pse-ids`; ordered output uses `BTreeMap` or a sorted `Vec` throughout.
//!
//! # Layout
//!
//! - [`id`] — the identity types and their textual forms.
//! - [`mod@derive`] — keyed, framed derivation of IDs and digests (§5.1).
//! - [`float`] — the ADR-0030 hashing-path float rules.
//! - [`frame`] — the unkeyed framing of the canonical preimages (§5.3 steps 4 and 7).
//! - [`encoding`] — encoded-artifact checksums (§5.3 step 6).
//! - [`contract`] — what the canonicalizer may consume: layouts, field paths, envelope.
//! - [`resource`] — the reserve-before-allocate interface (§14.3, ADR-0046).
//! - [`canon`] — the `pse.canon.v2` constants; the canonicalizer itself is packet B-canon.
//! - [`error`] — every error enum with its §23.2 class.

pub mod canon;
pub mod contract;
pub mod derive;
pub mod encoding;
pub mod error;
pub mod float;
pub mod frame;
pub mod id;
pub mod owned_buffer;
pub mod resource;
pub mod validation_extent;
pub use validation_extent::validation_extent;

pub use crate::canon::{CANON_VERSION, IPC_ALIGNMENT, IPC_METADATA_VERSION};
pub use crate::canon::{CanonicalOutput, CanonicalizeOptions, canonicalize, logical_hash};
pub use crate::contract::{CanonicalContract, DictKey, Envelope, FieldPath, Layout};
pub use crate::derive::{
    FramedHasher, IndexTuple, connection_equation_id, derive_hash, derive_id,
    discretized_symbol_id, equation_instance_id, law_term_id, mesh_node_id, named_id,
    symbol_instance_id,
};
pub use crate::encoding::{EncodingHasher, encoding_checksum};
pub use crate::error::{CanonError, EnvelopeBound, IdError, ReserveError};
pub use crate::float::{
    CANONICAL_F32_NAN_BITS, CANONICAL_F64_NAN_BITS, canonical_f32_bits, canonical_f64_bits,
};
pub use crate::frame::FrameSink;
pub use crate::id::{
    ContentHash, EncodingChecksum, LogicalHash, Ordinal, SchemaVersion, SemanticId,
};
pub use crate::resource::{
    CancellationToken, FixedBudget, MemoryReserver, Reservation, ReservationLease,
};
