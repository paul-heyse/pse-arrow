// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

// PyO3's `Ungil` proof for the nested inspection closures exceeds rustc's default depth
// of 128; its `recursion_depth_exceeding_limit` future-incompatibility lint asks for more.
#![recursion_limit = "256"]
#![allow(
    unsafe_code,
    reason = "pyo3 expansions carry the CPython ABI glue (blueprint §21)"
)]

//! pyo3 + pyo3-arrow extension module (cdylib) that is the whole Python boundary
//! (blueprint §3.2, §21).
//!
//! The module is imported as `pse._native`; nothing else in the Python package is
//! allowed to import it directly (ast-grep rule `no-direct-native-import`).
//!
//! Exposes build provenance and read-only exact Delta publication inspection. Named table
//! streams preserve the catalog's final-buffer reservations through Arrow C Stream
//! consumers; opening checks exact declarations under one explicit process budget.

use pyo3::prelude::*;

mod documents;
mod enums;
mod identities;
mod inspection;
mod workflow;

/// Captured build provenance projected from its Rust-owned document.
#[pyfunction]
fn build_info() -> documents::DocumentValue<pse_buildinfo::BuildInfo> {
    documents::DocumentValue(pse_buildinfo::BuildInfo::captured())
}

/// Compiled registry identity; does not assemble or inspect the runtime registry.
#[pyfunction]
fn registry_fingerprint() -> String {
    pse_catalog::inspection::REGISTRY_FINGERPRINT.to_prefixed()
}

/// Immutable admitted data inspection and build provenance.
#[pymodule]
mod _native {
    #[pymodule_export]
    use super::{
        build_info,
        identities::{
            content_hash_from_prefixed, content_hash_to_prefixed, semantic_id_from_hex,
            semantic_id_to_hex,
        },
        inspection::{
            CacheSettings, DiagnosticReport, EngineSettings, InspectionError, Publication,
            TableStream, open_export,
        },
        registry_fingerprint,
        workflow::{
            ModelingDiagnosticSettings, ModelingLimits, NativeAttempt, NativeModelingConformance,
            NativeModelingDiagnosticSamples, NativeModelingDiagnostics,
            NativeModelingElasticAttempt, NativeModelingInitialization,
            NativeModelingInitializationAttempt, NativeModelingKnowledge,
            NativeModelingNativeAnalysis, NativeModelingNonlinearExplanation,
            NativeModelingPackage, NativeModelingResult, NativeModelingTrajectory,
            NativePhysicalContext, NativePreparedFlow, NativePreparedOperation,
            NativePreparedStrategy, NativeProgressStream, NativePublicationAttempt,
            NativeRunHandle, NativeRunResult, NativeRuntime, NativeStart, NativeStrategyAttempt,
            NativeStrategyResult, NativeStudyHandle, NativeStudyReport, OperationalStore,
            SimulationSettings,
        },
    };

    #[pymodule_export]
    #[expect(non_upper_case_globals, reason = "Python module version convention")]
    const __version__: &str = pse_buildinfo::VERSION;
}
