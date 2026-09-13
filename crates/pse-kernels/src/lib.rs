// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `KernelSpec` registry and the built-in property, transport and reaction kernels
//! (blueprint §3.2, §9).
//!
//! Kernel bodies are written against the `num-dual` 0.15 trait shape
//! (`DualNum<Primitive = f64>`); a kernel never reads engine configuration
//! (blueprint §9, §18.2, §24.1 `config_options` ban).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
