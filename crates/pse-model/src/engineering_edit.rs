// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact-parent authored-file intent; the native loader owns all scientific interpretation.
use pse_contract::boundary_projection::{Direction, LocalRule};
use crate::engineering::{LOCAL_REVISION_RULE, OPERATION_ID_RULE, SOURCE_FILE_PATH_RULE};
use pse_ids::SemanticId;

pse_contract::projection_enum! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Explicit file membership operation; omission never deletes an inherited file.
    pub enum SourceFileChange {
        projection: { name: "SourceFileChange", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Add an absent authored document or scientific data file.
        Add(Vec<u8>) => 1;
        /// Replace an existing file in its original package, preserving loader identity rules.
        Replace(Vec<u8>) => 2;
        /// Explicitly remove an existing file and its loader-owned declarations/fits/data.
        Remove => 3;
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// One explicit edit to an existing package's source inventory.
    pub struct SourceFileEdit {
        projection: { name: "SourceFileEdit", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Existing package identity, never selected by an ambiguous path suffix.
        pub package: SemanticId => { tag: 1, direction: Direction::Input, rules: [] };
        /// Original relative path; ordinary loader path admission remains authoritative.
        pub path: String => { tag: 2, direction: Direction::Input, rules: [SOURCE_FILE_PATH_RULE] };
        /// Explicit add/replace/remove intent.
        pub change: SourceFileChange => { tag: 3, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Atomic exact-parent file editing consumed by the original native reload/delta owner.
    pub struct SourceFileDelta {
        projection: { name: "SourceFileDelta", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact locally admitted receiver token, distinct from semantic source identity.
        pub parent: String => { tag: 1, direction: Direction::Input, rules: [LOCAL_REVISION_RULE] };
        /// Unique explicit file edits; unmentioned packages/files remain inherited.
        pub edits: Vec<SourceFileEdit> => { tag: 2, direction: Direction::Input, rules: [LocalRule::Length { min: 1, max: 1024 }] };
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Idempotent service request for a source-owner exact-parent file delta.
    pub struct EngineeringEditSource {
        projection: { name: "EngineeringEditSource", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original caller-issued edit operation identity.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Original source operation retaining the exact receiver.
        pub source_operation: String => { tag: 2, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Actual authored-file delta; protobuf performs no scientific parsing.
        pub delta: SourceFileDelta => { tag: 3, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}
