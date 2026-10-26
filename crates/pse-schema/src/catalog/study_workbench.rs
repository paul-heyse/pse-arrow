// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Indexed compact study selection; exact result and source owners remain authoritative.
use super::substrate::NativeIndex;
/// Maximum raw compact bytes in one native outcome/capture page.
pub const STUDY_PAGE_BYTES: usize = 2 * 1024 * 1024;
/// Compact settled outcomes in one bounded native response.
pub const STUDY_OUTCOME_PAGE_ROWS: usize = 128;
/// Frozen compact cohort members in one bounded native response/cleanup page.
pub const STUDY_CAPTURE_PAGE_ROWS: usize = 256;
/// Maximum complete cohort population admitted by the qualified native capture.
pub const STUDY_COHORT_MAX_MEMBERS: usize = 10_000;
/// Useful outcome rows packed across native metadata pages before publication.
pub const STUDY_SUMMARY_BUFFER_ROWS: usize = 1024;
/// Compact encoded outcome bytes retained before requesting another page.
pub const STUDY_SUMMARY_BUFFER_BYTES: usize = 1024 * 1024;

/// Native access paths for source, physical context, lineage and outcome selection.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex {
        name: "capture_protection_study",
        table: "canonical_study_capture_protections",
        fields: &["study", "key"],
        unique: false,
    },
    NativeIndex {
        name: "capture_header_order",
        table: "canonical_study_capture_headers",
        fields: &["capture", "ordinal"],
        unique: true,
    },
    NativeIndex {
        name: "capture_member_order",
        table: "canonical_study_capture_members",
        fields: &["capture", "ordinal"],
        unique: true,
    },
    NativeIndex {
        name: "capture_manifest_run",
        table: "canonical_study_capture_protections",
        fields: &["run", "key"],
        unique: false,
    },
    NativeIndex {
        name: "capture_source_problem",
        table: "canonical_study_capture_protections",
        fields: &["problem", "revision"],
        unique: false,
    },
    NativeIndex {
        name: "capture_protection_cleanup",
        table: "canonical_study_capture_protections",
        fields: &["capture", "key"],
        unique: false,
    },
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
