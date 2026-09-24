// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Semantic identity and canonical hashing; native representations live in pse-columnar.
pub mod derive;
pub mod encoding;
pub mod error;
pub mod float;
pub mod frame;
pub mod id;
pub mod preimage;
pub use derive::{FramedHasher, derive_hash, derive_id, named_id};
pub use encoding::{EncodingHasher, encoding_checksum};
pub use error::IdError;
pub use float::{
    CANONICAL_F32_NAN_BITS, CANONICAL_F64_NAN_BITS, canonical_f32_bits, canonical_f64_bits,
};
pub use frame::FrameSink;
pub use id::{ContentHash, EncodingChecksum, LogicalHash, SchemaVersion, SemanticId};

/// Declaration-key steps in authored instance paths.
pub mod source_path;
