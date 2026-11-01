// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored wire contracts. Native context checks and CPU settlement retain their owners.
/// Generated typed messages and direct reflection over the authored wire schema.
#[path = "generated/code-intelligence/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator public surface, authored wire enum spellings and generated accessor documentation")]
pub mod pb;
#[path = "generated/code-intelligence/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator spelling and private packaging modules")]
mod validation;
/// Generated service contracts and reusable typed client bindings; no listener is started.
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, reason = "Pinned service generator public surface and generated helpers")]
#[allow(clippy::match_single_binding, reason = "Pinned service generator emits an unimplemented match arm for absent unary or streaming methods")]
pub mod services {
    include!("generated/code-intelligence/services/pse.code_intelligence.v1.mod.rs");
}

/// Complete source-aware descriptors; the embedded runtime pool is a derived projection.
pub const DESCRIPTORS: &[u8] = include_bytes!("generated/code-intelligence/descriptor.bin");

mod local;
pub use local::{DeclarationProjection, api_declaration, project_declaration, source_declaration};
