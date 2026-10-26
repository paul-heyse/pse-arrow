// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Responsibility-owned development storage and its generation contributions.
pub mod development;
/// Private generated semantic values, native codecs and columnar contracts.
#[cfg(feature = "evidence-tools")]
#[rustfmt::skip]
pub mod generated;
