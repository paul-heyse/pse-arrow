// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native resolution of the pure target syntax.
mod native;
pub use native::resolve_native;
use pse_authoring::targets::{IndexSelector, TargetPath, TargetRow};
