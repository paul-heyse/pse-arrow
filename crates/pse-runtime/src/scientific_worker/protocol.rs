// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual scientific worker operation owners and source-derived wire metadata.
use pse_ids::{ContentHash, SemanticId};
use pse_model::boundary_projection::{Direction, LocalRule};

pse_model::projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Exact attempt and child generation; this carries no owner-sealed proof.
    pub struct Handle {
        projection: { name: "Handle", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Original mutable scientific attempt.
        pub attempt: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
        /// Fresh explicitly launched child session.
        pub session: SemanticId => { tag: 2, direction: Direction::Both, rules: [] };
        /// Fresh immutable prepared demand in that session.
        pub generation: SemanticId => { tag: 3, direction: Direction::Both, rules: [] };
    }
}
pse_model::projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Ordered scalar physical port, resolved through actual quantity/unit identities.
    pub struct Port {
        projection: { name: "Port", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Semantic scalar port identity.
        pub id: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
        /// Complete physical quantity identity.
        pub quantity: SemanticId => { tag: 2, direction: Direction::Both, rules: [] };
        /// Actual numerical representation unit.
        pub unit: SemanticId => { tag: 3, direction: Direction::Both, rules: [] };
    }
}
pse_model::projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Complete scalar-only descriptor of the selected water implementation.
    pub struct Spec {
        projection: { name: "Spec", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Scientific implementation identity.
        pub id: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
        /// Algorithm and interpretation revision.
        pub revision: ContentHash => { tag: 2, direction: Direction::Both, rules: [] };
        /// Exact released parameter data identity.
        pub data: ContentHash => { tag: 3, direction: Direction::Both, rules: [] };
        /// Input scalar order consumed by the native owner.
        pub inputs: Vec<Port> => { tag: 4, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 64 }] };
        /// Output scalar order declared by the native owner.
        pub outputs: Vec<Port> => { tag: 5, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 64 }] };
        /// Declared derivative provenance, independent of order.
        pub derivative_source: String => { tag: 6, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 64 }] };
        /// Maximum available partial derivative order.
        pub derivatives: u32 => { tag: 7, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 0, max: 2 }] };
        /// Maximum admitted smoothness order.
        pub smoothness: u32 => { tag: 8, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 0, max: 2 }] };
    }
}
pse_model::projection_enum! {
    #[derive(Clone, Debug, PartialEq, Eq)]
    /// All kernel product kinds; transport availability does not imply native support.
    pub enum Demand {
        projection: { name: "Demand", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Complete output-major values and partial arrays.
        CompleteJets => 1;
        /// Ordered input column subset and its second-order square.
        SelectedColumns(Vec<u64>) => 2;
        /// Number of direction-major forward seeds.
        JacobianAction(u64) => 3;
        /// Number of selected-output cotangents.
        TransposeAction(u64) => 4;
        /// Number of directions for one weighted second product.
        WeightedSecondAction(u64) => 5;
    }
}
pse_model::projection_struct! {
    #[derive(Clone, Debug)]
    /// One immutable preparation, carrying selected science and original remaining time.
    pub struct Prepare {
        projection: { name: "Prepare", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Selected major protocol contract.
        pub version: u32 => { tag: 1, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: 1 }] };
        /// Parent-issued fresh context reference.
        pub handle: Handle => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Complete expected physical and algorithm descriptor.
        pub spec: Spec => { tag: 3, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Expected native image, data, fitted constants and datum configuration.
        pub configuration: ContentHash => { tag: 4, direction: Direction::Input, rules: [] };
        /// Explicit v1 selected registry. A custom physical interpretation refuses.
        pub physical_registry: String => { tag: 5, direction: Direction::Input, rules: [LocalRule::Length { min: 1, max: 64 }] };
        /// Explicit homogeneous branch; normal native calls never impose phase.
        pub regime: String => { tag: 6, direction: Direction::Input, rules: [LocalRule::Length { min: 1, max: 64 }] };
        /// Ordered unique selected output ordinals.
        pub outputs: Vec<u64> => { tag: 7, direction: Direction::Input, rules: [LocalRule::Length { min: 1, max: 64 }] };
        /// Requested derivative order.
        pub order: u32 => { tag: 8, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 0, max: 2 }] };
        /// Exact requested product; actions retain honest support/refusal.
        pub demand: Demand => { tag: 9, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Remaining original scope lifetime; absent means no scientific deadline.
        pub remaining_ns: Option<u64> => { tag: 10, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
        /// Admitted complete numerical result storage, in bytes.
        pub result_limit: u64 => { tag: 11, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: 1048576 }] };
    }
}
pse_model::projection_struct! {
    #[derive(Clone, Debug)]
    /// Invocation-local numerical inputs; descriptors are already prepared.
    pub struct Evaluate {
        projection: { name: "Evaluate", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact current prepared attempt/session/generation.
        pub handle: Handle => { tag: 1, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Monotonic nonzero invocation identity; repeated dispatch refuses.
        pub invocation: u64 => { tag: 2, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
        /// Ordered canonical physical input bits.
        pub inputs: Vec<f64> => { tag: 3, direction: Direction::Input, rules: [LocalRule::Length { min: 1, max: 64 }] };
        /// Direction-major input seeds.
        pub directions: Vec<f64> => { tag: 4, direction: Direction::Input, rules: [LocalRule::Length { min: 0, max: 65536 }] };
        /// Cotangent-major selected-output weights.
        pub cotangents: Vec<f64> => { tag: 5, direction: Direction::Input, rules: [LocalRule::Length { min: 0, max: 65536 }] };
        /// One weight per selected output for weighted second actions.
        pub weights: Vec<f64> => { tag: 6, direction: Direction::Input, rules: [LocalRule::Length { min: 0, max: 64 }] };
    }
}
pse_model::projection_enum! {
    #[derive(Clone, Debug)]
    /// Distinct lifecycle and scientific operations on one authenticated channel.
    pub enum Request {
        projection: { name: "Request", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Reconstruct and admit one immutable demand before effects.
        Prepare(Prepare) => 1;
        /// Execute one coherent scientific invocation without automatic replay.
        Evaluate(Evaluate) => 2;
        /// Cancel only this prepared handle and retire its settled state.
        Stop(Handle) => 3;
        /// Destroy only this handle's settled native state.
        Close(Handle) => 4;
        /// Close admission and cancel/drain every handle on this authenticated session.
        Shutdown => 5;
    }
}
pse_model::projection_struct! {
    #[derive(Clone, Debug)]
    /// One complete selected numerical product; no linearization proof is exported.
    pub struct Values {
        projection: { name: "Values", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact completing attempt/session/generation.
        pub handle: Handle => { tag: 1, direction: Direction::Output, rules: [] };
        /// Exact completing invocation.
        pub invocation: u64 => { tag: 2, direction: Direction::Output, rules: [] };
        /// Requested physical values in selected output order.
        pub values: Vec<f64> => { tag: 3, direction: Direction::Output, rules: [LocalRule::Length { min: 1, max: 65536 }] };
        /// First partials or actions in the demand's declared order.
        pub jacobian: Vec<f64> => { tag: 4, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 65536 }] };
        /// Second partials or actions in the demand's declared order.
        pub hessians: Vec<f64> => { tag: 5, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 65536 }] };
    }
}
pse_model::projection_enum! {
    #[derive(Clone, Debug)]
    /// Owner-attributed outcomes; rendered text never selects a diagnostic class.
    pub enum Failure {
        projection: { name: "Failure", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Original cooperative stop.
        Cancelled => 1;
        /// Original absolute deadline expiry.
        Deadline => 2;
        /// Original allocation refusal and its detail.
        Resource(String) => 3;
        /// Rejected immutable contract or capability.
        Contract(String) => 4;
        /// Recoverable native trial rejection.
        Trial(String) => 5;
        /// Physical domain refusal with exact canonical coordinate bits.
        OutsideEnvelope {
            /// Physical coordinate selected by the scientific owner.
            axis: String => { tag: 1, direction: Direction::Output, rules: [LocalRule::Length { min: 1, max: 256 }] };
            /// Attempted canonical value.
            value: f64 => { tag: 2, direction: Direction::Output, rules: [] };
            /// Inclusive lower bound.
            lower: f64 => { tag: 3, direction: Direction::Output, rules: [] };
            /// Inclusive upper bound.
            upper: f64 => { tag: 4, direction: Direction::Output, rules: [] };
        } => 6;
        /// Terminal native or infrastructure refusal.
        Terminal(String) => 7;
        /// Dispatched effects have no authenticated complete response.
        UnknownCompletion(String) => 8;
    }
}
pse_model::projection_enum! {
    #[derive(Clone, Debug)]
    /// Complete operation outcome on the original authenticated session.
    pub enum Response {
        projection: { name: "Response", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// The actual reconstructed native owner admitted this demand.
        Prepared(Handle) => 1;
        /// The exact invocation produced a complete numerical product.
        Values(Values) => 2;
        /// A typed refusal, without a readable partial result.
        Failure(Failure) => 3;
        /// Native state was destroyed; process settlement remains the parent's duty.
        Closed(Handle) => 4;
        /// All session-native state has joined; the parent still owns process/I/O settlement.
        Shutdown => 5;
    }
}
