// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-generated passive evidence. No decoded value constructs a performing owner.

/// Generated data and request-local reflection over the native source declarations.
#[path = "generated/developer-evidence/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
pub mod pb;
#[path = "generated/developer-evidence/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator surface")]
mod validation;

use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
use pse_contract_proto::support::*;
#[path = "generated/developer-evidence/native-adapters.rs"]
mod adapters;
pub(crate) use adapters::*;

#[path = "generated/developer-evidence/native-policy.rs"]
/// Source-derived coverage; acquired descriptors cannot nominate their own native authority.
pub mod source_policy;

/// Exact compiled descriptor artifact; a different acquired artifact stays a separate world.
pub const DESCRIPTORS: &[u8] = include_bytes!("generated/developer-evidence/descriptor.bin");

/// Native source coverage, independent of the acquired descriptor's surviving options.
pub(crate) struct Policy;
impl pse_contract::metadata::InterpretationPolicy for Policy {
    fn name(&self) -> &str { "pse.developer.evidence.v1/native" }
    fn supports_interpretation(&self, subject: &str, version: u32) -> bool {
        source_policy::INTERPRETATIONS.contains(&(subject, version))
    }
}
impl pse_contract_proto::DescriptorPolicy for Policy {
    fn native_selection(&self) -> Option<pse_contract_proto::NativeSelection<'_>> {
        Some(source_policy::NATIVE_SELECTION)
    }
}
