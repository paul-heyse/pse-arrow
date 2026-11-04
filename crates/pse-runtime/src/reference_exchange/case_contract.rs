// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared case correspondence; numerical candidates are exchanged separately.
use crate::workflow::PhysicalQuantityDescription;
use pse_contract::boundary_projection::{Direction, LocalRule};
use pse_ids::SemanticId;
use pse_model::{document::Version, reference_exchange::ReferenceMeaning};

const KEY: LocalRule = LocalRule::Utf8Bytes { min: 1, max: 1024 };
const POPULATION: LocalRule = LocalRule::Length {
    min: 1,
    max: 65_536,
};

pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Independently agreed scalar coordinate and its original physical interpretation.
    pub struct ReferenceObservableContract {
        projection: { name: "ReferenceObservableContract", rust_path: crate::reference_exchange::ReferenceObservableContract,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Stable output coordinate in the entire required comparison population.
        pub output: SemanticId => { tag: 1, direction: Direction::Output, rules: [] };
        /// Scientific reference coordinate, independent of a rendered native report label.
        pub key: String => { tag: 2, direction: Direction::Output, rules: [KEY] };
        /// Actual quantity/unit/basis/datum owner, without a second metadata projection.
        pub physical: PhysicalQuantityDescription => { tag: 3, direction: Direction::Output, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Original fixed input; it is never an expected answer or a transferred solution.
    pub struct ReferenceFixedInput {
        projection: { name: "ReferenceFixedInput", rust_path: crate::reference_exchange::ReferenceFixedInput,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Scientific input coordinate in the agreed case specification.
        pub key: String => { tag: 1, direction: Direction::Output, rules: [KEY] };
        /// Original canonical scalar bits, before any independent calculation.
        pub value: f64 => { tag: 2, direction: Direction::Output, rules: [] };
        /// Exact original physical interpretation of this input.
        pub physical: PhysicalQuantityDescription => { tag: 3, direction: Direction::Output, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Exact original artifact used in establishing scientific correspondence.
    pub struct ReferenceInputArtifact {
        projection: { name: "ReferenceInputArtifact", rust_path: crate::reference_exchange::ReferenceInputArtifact,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Agreed checkout-relative artifact; the foreign consumer independently selects its allowlist.
        pub path: String => { tag: 1, direction: Direction::Output, rules: [LocalRule::Utf8Bytes { min: 1, max: 4096 }] };
        /// SHA256 of original bytes, distinct from native semantic/content hashing.
        pub sha256: pse_model::Bytes => { tag: 2, direction: Direction::Output, rules: [LocalRule::Length { min: 32, max: 32 }] };
    }
}

pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Complete comparison contract sent before an independent reference calculation.
    /// This carries inputs and correspondence, while candidate values stay in a separate envelope.
    pub struct ReferenceCaseContract {
        projection: { name: "ReferenceCaseContract", rust_path: crate::reference_exchange::ReferenceCaseContract,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Case-contract interpretation; scientific formulation has its separate identity below.
        pub version: Version<1> => { tag: 1, direction: Direction::Output, rules: [LocalRule::Required] };
        /// Selected independently implemented case, with attributable refusal for unknown names.
        pub case_name: String => { tag: 2, direction: Direction::Output, rules: [KEY] };
        /// Exact source, formulation, physical and configuration correspondence.
        pub meaning: ReferenceMeaning => { tag: 3, direction: Direction::Output, rules: [LocalRule::Required] };
        /// Complete required scalar coordinates; duplicate identities are a correspondence error.
        pub observables: Vec<ReferenceObservableContract> => { tag: 4, direction: Direction::Output, rules: [POPULATION] };
        /// Selected original fixed inputs, excluding numerical starts and candidate values.
        pub specification: Vec<ReferenceFixedInput> => { tag: 5, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 65_536 }] };
        /// Complete selected source/data artifact population for the independent constructor.
        pub artifacts: Vec<ReferenceInputArtifact> => { tag: 6, direction: Direction::Output, rules: [LocalRule::Length { min: 1, max: 1024 }] };
    }
}
