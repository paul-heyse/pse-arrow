// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Indexed compact study selection; exact result and source owners remain authoritative.
use super::substrate::NativeIndex;
/// Native access paths for source, physical context, lineage and outcome selection.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex {
        name: "study_source_order",
        table: "canonical_studies",
        fields: &["revision", "key"],
        unique: false,
    },
    NativeIndex {
        name: "study_problem_order",
        table: "canonical_studies",
        fields: &["problem", "key"],
        unique: false,
    },
    NativeIndex {
        name: "study_physical_order",
        table: "canonical_studies",
        fields: &["physical_revision", "key"],
        unique: false,
    },
    NativeIndex {
        name: "study_context_order",
        table: "canonical_studies",
        fields: &["context", "key"],
        unique: false,
    },
    NativeIndex {
        name: "study_outcome_runs",
        table: "canonical_runs",
        fields: &["terminal_class", "key"],
        unique: false,
    },
    NativeIndex {
        name: "study_research_revision",
        table: "canonical_research_references",
        fields: &["revision", "key"],
        unique: false,
    },
];
