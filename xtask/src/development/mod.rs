// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Disposable evidence and knowledge; original source owners retain authority and custody.
pub mod audit;
pub mod catalog;
pub mod configuration;
#[cfg(feature = "package-fixtures")]
pub mod conformance;
#[cfg(feature = "evidence-tools")]
pub mod evidence;
#[cfg(feature = "evidence-tools")]
pub mod evidence_query;
#[cfg(feature = "evidence-tools")]
pub mod knowledge;
pub mod repository_tools;
#[cfg(feature = "package-fixtures")]
pub mod thermodynamic_campaign;
#[cfg(feature = "evidence-tools")]
pub mod workspace_sources;

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
