// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Disposable evidence and knowledge; original source owners retain authority and custody.
pub mod catalog;
#[cfg(feature = "evidence-tools")]
pub mod evidence;
#[cfg(feature = "evidence-tools")]
pub mod evidence_query;
#[cfg(feature = "evidence-tools")]
pub mod knowledge;

/// Resolve the independent private registry, without adding declarations to product schemas.
/// # Errors
/// A malformed responsibility-owned declaration.
pub fn registry() -> Result<&'static pse_schema::Registry, pse_schema::SchemaError> {
    static REGISTRY: std::sync::OnceLock<Result<pse_schema::Registry, pse_schema::SchemaError>> =
        std::sync::OnceLock::new();
    match REGISTRY.get_or_init(catalog::assemble) {
        Ok(registry) => Ok(registry),
        Err(error) => Err(error.clone()),
    }
}

