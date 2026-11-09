// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated projections of the actual scientific operation owners.
#[path = "../generated/scientific-worker/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
pub mod pb;
#[path = "../generated/scientific-worker/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned local validation generator packaging")]
mod validation;
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, reason = "Pinned service generator surface")]
#[allow(clippy::match_single_binding, reason = "Pinned unary service generator emits empty streaming branches")]
pub mod services {
    include!("../generated/scientific-worker/services/pse.scientific_worker.v2.mod.rs");
}
