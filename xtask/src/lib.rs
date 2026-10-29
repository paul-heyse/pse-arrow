// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Responsibility-owned development storage and its generation contributions.
#![recursion_limit = "512"]
#![allow(
    clippy::disallowed_types,
    reason = "xtask's developer-only library preserves heterogeneous external tool errors, matching its command boundary"
)]
pub mod development;
/// Native Markdown and authored document metadata readers.
pub mod documentation;
/// Private generated semantic values, native codecs and columnar contracts.
#[cfg(feature = "evidence-tools")]
#[rustfmt::skip]
pub mod generated;
