// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The relational face of the math IR (blueprint §6.9, §7.1 constraint 1).
//!
//! Filled by packet M-5: `MathRelationSink` and `MathRelationSource` with
//! primitive-typed methods, plus `emit`, `emit_untyped`, `load_untyped` and
//! `load_canonical`.
//!
//! The traits carry primitives rather than row structs because this crate depends on no
//! engine crate: the adapter over the generated Arrow builders lives in `pse-compiler`
//! (`mathir_relations.rs`), which is what keeps `pse-mathir` Arrow-free.

pub mod vec_sink;
