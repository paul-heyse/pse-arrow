// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pure registry code generation outside product dependencies.

pub use pse_schema::*;
pub mod codegen;
pub use codegen::{GeneratedTree, Language, generate};
