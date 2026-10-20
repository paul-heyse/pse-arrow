// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

// PyO3's `Ungil`/Send proofs include nested workflow observation wrappers and exceed
// the default compiler depth. Match the runtime's finite checking depth without boxing
// its futures or weakening auto-trait requirements.
#![recursion_limit = "512"]
#![allow(
    unsafe_code,
    reason = "pyo3 CPython ABI glue (blueprint §21) and ADR-0164 controlled deployment qualification"
)]

//! pyo3 + pyo3-arrow extension module (cdylib) that is the whole Python boundary
//! (blueprint §3.2, §21).
//!
//! The module is imported as `pse._native`; nothing else in the Python package is
//! allowed to import it directly (ast-grep rule `no-direct-native-import`).
//!
//! Exposes build provenance and exact canonical scientific inspection. Named table
//! streams retain protected source ownership through Arrow C Stream consumers.

use pyo3::prelude::*;

mod diagnostics;
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
    pse_relations::generated::REGISTRY_FINGERPRINT.to_prefixed()
}

/// Features of this actual extension binary, independent of its launch environment.
#[pyfunction]
fn compiled_features() -> Vec<String> {
    [
        ("compiled-evaluator", cfg!(feature = "compiled-evaluator")),
        ("force-validate", cfg!(feature = "force-validate")),
        ("native-solvers", cfg!(feature = "native-solvers")),
        ("solver-diffsol", cfg!(feature = "solver-diffsol")),
        ("solver-highs", cfg!(feature = "solver-highs")),
        ("solver-ipopt", cfg!(feature = "solver-ipopt")),
        ("solver-kinsol", cfg!(feature = "solver-kinsol")),
        ("solver-pounce", cfg!(feature = "solver-pounce")),
        ("solver-scip", cfg!(feature = "solver-scip")),
    ]
    .into_iter()
    .filter(|&(_, enabled)| enabled)
    .map(|(name, _)| name.to_owned())
    .collect()
}

/// Immutable admitted data inspection and build provenance.
#[pymodule]
mod _native {
    #[pymodule_export]
    use super::{
        build_info, compiled_features,
        diagnostics::{
            NativeCaptureSession, NativeDetailSession, NativeDiagnostics, NativeLaunchedCommand,
            NativeOperationObservations, diagnostic_analyze, diagnostic_capture,
            diagnostic_capture_arrow, diagnostic_compare_invocations, diagnostic_detail_arrow,
            diagnostic_detail_records, diagnostic_explain_invocation, diagnostic_export_pack,
            diagnostic_inspect, diagnostic_inspect_frozen, diagnostic_inspect_image,
            diagnostic_inventory_from_evidence, diagnostic_inventory_page,
            diagnostic_launch_capture, diagnostic_list, diagnostic_operation_id,
            diagnostic_operation_text, diagnostic_pack_analysis, diagnostic_pack_evidence,
            diagnostic_query_specification, diagnostic_read_checkpoint, diagnostic_read_context,
            diagnostic_read_detail, diagnostic_read_evidence, diagnostic_read_handoff,
            diagnostic_read_invocation, diagnostic_render_analysis, diagnostic_review_arrow,
            diagnostic_review_pack,
        },
        identities::{
            content_hash_from_prefixed, content_hash_to_prefixed, semantic_id_from_hex,
            semantic_id_to_hex,
        },
        inspection::{
            CacheSettings, DiagnosticReport, EngineSettings, InspectionError, TableStream,
            registry_table,
        },
        registry_fingerprint,
        workflow::{
            ModelingDiagnosticSettings, ModelingLimits, NativeAnalysis, NativeArtifactSettlement,
            NativeAttempt, NativeLiveSimulationCursor, NativeLiveSimulationEvent, NativeLocalStudy,
            NativeModelingConformance, NativeModelingDiagnosticSamples, NativeModelingDiagnostics,
            NativeModelingElasticAttempt, NativeModelingInitialization,
            NativeModelingInitializationAttempt, NativeModelingKnowledge,
            NativeModelingNativeAnalysis, NativeModelingNonlinearExplanation,
            NativeModelingPackage, NativeModelingResult, NativeModelingTrajectory,
            NativePhysicalContext, NativePreparedFlow, NativePreparedOperation,
            NativePreparedStrategy, NativeProgressStream, NativeRunHandle, NativeRunResult,
            NativeRuntime, NativeScientificArtifact, NativeScientificReplay, NativeStart,
            NativeStoredResult, NativeStrategyAttempt, NativeStrategyResult, NativeStudyHandle,
            NativeStudyReport, SimulationSettings,
        },
    };

    #[pymodule_export]
    #[expect(non_upper_case_globals, reason = "Python module version convention")]
    const __version__: &str = pse_buildinfo::VERSION;
}
