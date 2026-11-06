// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked numerical Root capture and passive metadata restoration. Origin belongs
//! to the private runtime issuer and authenticated attempt, never these raw records.
use super::*;

pse_contract::projection_struct! {
/// Original numerical Root definition. A matching attribution checksum detects
/// inconsistent data; it cannot authenticate a caller-controlled replacement.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootDefinitionSnapshot {
    projection: { name: "MathRootDefinitionSnapshot", rust_path: crate::implicit::transfer::RootDefinitionSnapshot, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Fixed private numerical Root profile, independent of wire recursion allowance.
    pub version: u32 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: 1 }] };
    /// Original implicit system identity.
    pub id: SemanticId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original residual content identity, including original residual offsets.
    pub identity: ContentHash => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Unknowns in original leading-formal order; this profile requires finite intervals.
    pub unknowns: Vec<Unknown> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original equation identities in selected output order.
    pub rows: Vec<SemanticId> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original independent parameter population after the leading unknown formals.
    pub inputs: usize => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original finite additive residual offsets in equation order, preserving bits.
    pub residual_offsets: Vec<f64> => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Original resolved derivative facts; these do not grant later source preparation.
    pub requirements: pse_kernels::DerivativeRequirements => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Original native unknown product order; the fixed transfer profile is First.
    pub native_order: DerivativeOrder => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Compact CSC column pointers, starting zero and ending at row_idx.len().
    pub col_ptr: Vec<usize> => { tag: 10, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Strictly increasing row indices within every original CSC column.
    pub row_idx: Vec<usize> => { tag: 11, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Checksum of the complete original attribution framing, sealed by the parent issuer.
    pub attribution_commitment: ContentHash => { tag: 12, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Complete passive capture of an actual held RootWorker and its original metadata.
/// Decoding or consistently replacing these records establishes no attempt origin.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootExecutionSnapshot {
    projection: { name: "MathRootExecutionSnapshot", rust_path: crate::implicit::transfer::RootExecutionSnapshot, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original numerical Root signature, CSC and attribution checksum.
    pub definition: RootDefinitionSnapshot => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Exact complete original maps, not a filtered token subset.
    pub attribution: crate::ExecutionAttributionSnapshot => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Exact cold numerical template from the actual held RootWorker.
    pub numerical: crate::transfer::ExecutionSnapshot => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
}
}

/// Populations admitted before original metadata capture. Backing is a checked Rust
/// upper bound, not allocator usable size, RSS or a wire/stack allowance.
#[derive(Clone, Debug)]
pub struct RootCaptureExtents {
    /// Actual numerical-template populations and requested backing.
    pub numerical: crate::transfer::TransferExtents,
    /// Requested Root definition and full original attribution backing upper bound.
    pub metadata_bytes: usize,
    /// Combined passive output backing upper bound before capture effects.
    pub capture_bytes: usize,
    /// Actual original attribution entries across all three maps.
    pub attribution_entries: usize,
    /// Actual original UTF-8 string bytes to clone.
    pub attribution_text_bytes: usize,
}
/// Derived Root metadata/frame construction demand; input ownership is already
/// admitted by the boundary and must remain live while moved into the Root.
#[derive(Clone, Debug)]
pub struct RootRestoreExtents {
    /// Checked numerical demand from actual owned input, including retained capacities.
    pub numerical: crate::transfer::TransferExtents,
    /// Existing owned metadata backing upper bound retained across construction.
    pub metadata_input_bytes: usize,
    /// New Root Arc, owner anchors and freshly allocated argument/cache backing.
    pub construction_bytes: usize,
    /// Admitted identity validation scratch released before final publication.
    pub validation_bytes: usize,
    /// Combined requested backing bound including still-live metadata/numerical input.
    pub restore_peak_bytes: usize,
    /// Actual square unknown/equation population.
    pub unknowns: usize,
    /// Actual compact numerical input population.
    pub numerical_inputs: usize,
    /// Actual original sparse CSC entries, including conservative extras.
    pub csc_entries: usize,
}

#[cfg(test)]
mod tests {
    #[test]
    fn root_transfer_records_are_passive_and_reuse_actual_native_owner_records() {
        use super::*;
        use pse_contract::boundary_projection::{
            ConstructionRole, LocalRule, ProjectionGraph, Shape,
        };
        let graph = ProjectionGraph::of::<RootExecutionSnapshot>().unwrap();
        for name in [
            "MathRootExecutionSnapshot",
            "MathRootDefinitionSnapshot",
            "MathExecutionAttributionSnapshot",
            "KernelDerivativeRequirements",
            "ImplicitUnknown",
            "MathExecutionSnapshot",
        ] {
            assert_eq!(
                graph.definitions()[name].construction,
                ConstructionRole::Data
            );
        }
        let Shape::Record(fields) = &graph.definitions()["MathRootDefinitionSnapshot"].shape else {
            panic!("definition record");
        };
        assert_eq!(
            fields
                .iter()
                .find(|field| field.name == "version")
                .unwrap()
                .rules,
            [LocalRule::UnsignedRange { min: 1, max: 1 }]
        );
        assert!(
            fields
                .iter()
                .any(|field| field.name == "attribution_commitment")
        );
    }
}
