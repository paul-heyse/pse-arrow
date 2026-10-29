// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Responsibility-owned Markdown and strict document metadata interpretation.
pub mod adr;
pub mod headings;
pub mod lifecycle;
pub mod site;
pub mod metadata;
pub mod register;
#[cfg(test)]
mod metadata_tests;
