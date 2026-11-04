// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated projections of the actual engineering request/response owners.
#[path = "../../generated/engineering/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
pub mod pb;
#[path = "../../generated/engineering/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator spelling and packaging modules")]
mod validation;
/// Generated typed service and independent client; importing this starts no listener.
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, reason = "Pinned service generator surface")]
#[allow(clippy::match_single_binding, reason = "Pinned service generator emits empty streaming branches")]
pub mod services {
    include!("../../generated/engineering/services/pse.engineering.v1.mod.rs");
}
/// Complete immutable descriptor identity for the actual compiled service projection.
pub const DESCRIPTORS: &[u8] = include_bytes!("../../generated/engineering/descriptor.bin");
