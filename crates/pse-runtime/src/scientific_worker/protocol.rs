// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual scientific worker operation owners and source-derived wire metadata.
use pse_contract::boundary_projection::{Direction, LocalRule};
use pse_ids::{ContentHash, SemanticId};

/// Conservative effects of the multiplexed authenticated Exchange operation.
/// Preparation, numerical work and lifecycle control are effectful. Duplicate refusal
/// is not a promise that an independent client can recover an original result by replay.
/// Individual native operations retain their own identity, admission and settlement.
pub const EXCHANGE_EFFECTS: pse_contract::operation_effects::EffectFacts =
    pse_contract::operation_effects::EffectFacts {
        scope: pse_contract::operation_effects::EffectScope::NativeExecution,
        read_only: false,
        idempotent: false,
        destructive: false,
    };

#[cfg(any(test, feature = "property-coolprop"))]
pub(super) const FAILURE_DEPTH: usize = 32;
#[cfg(any(test, feature = "property-coolprop"))]
pub(super) const FAILURE_NODES: usize = 1024;
#[cfg(any(test, feature = "property-coolprop"))]
const FAILURE_RETAINED_TEXT: usize = 64 << 10;

pse_contract::projection_struct! {
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
pse_contract::projection_enum! {
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
pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// One immutable preparation, carrying selected science and original remaining time.
    pub struct Prepare {
        projection: { name: "Prepare", version: 2, direction: Direction::Input, reserved_tags: [5], reserved_names: ["physical_registry"] };
        /// Selected major protocol contract.
        pub version: u32 => { tag: 1, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 2, max: 2 }] };
        /// Parent-issued fresh context reference.
        pub handle: Handle => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Complete expected physical and algorithm descriptor.
        pub spec: pse_kernels::portable::ProviderRecord => { tag: 3, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Expected native image, data, fitted constants and datum configuration.
        pub configuration: ContentHash => { tag: 4, direction: Direction::Input, rules: [] };
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
        /// Actual unadmitted physical declarations; the native builder establishes membership.
        pub registry: pse_quantity::QuantityRegistryBuilder => { tag: 12, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Expected interpretation of the actual declarations, checked after native reconstruction.
        pub physical: ContentHash => { tag: 13, direction: Direction::Input, rules: [] };
    }
}
pse_contract::projection_struct! {
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
pse_contract::projection_enum! {
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
        /// Execute one native group; raw point cells undergo independent scientific admission.
        EvaluateBatch(EvaluateBatch) => 6;
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Prepared native capability for the exact reconstructed immutable demand.
    pub struct Prepared {
        projection: { name: "Prepared", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact admitted context.
        pub handle: Handle => { tag: 1, direction: Direction::Output, rules: [] };
        /// Native owner supports a genuine group for this demand; scalar support grants nothing.
        pub batch_supported: bool => { tag: 2, direction: Direction::Output, rules: [] };
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Original identified raw point; uint64 cells preserve every IEEE bit until point admission.
    pub struct BatchPoint {
        projection: { name: "BatchPoint", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Unique nonzero group-local identity, checked by the native envelope owner.
        pub id: SemanticId => { tag: 1, direction: Direction::Input, rules: [] };
        /// Exact ordered input bits, including independently refused nonfinite coordinates.
        pub inputs: Vec<u64> => { tag: 2, direction: Direction::Input, rules: [LocalRule::Length { min: 0, max: 64 }] };
        /// Exact direction seed bits.
        pub directions: Vec<u64> => { tag: 3, direction: Direction::Input, rules: [LocalRule::Length { min: 0, max: 65536 }] };
        /// Exact output cotangent bits.
        pub cotangents: Vec<u64> => { tag: 4, direction: Direction::Input, rules: [LocalRule::Length { min: 0, max: 65536 }] };
        /// Exact weighted-second seed bits.
        pub weights: Vec<u64> => { tag: 5, direction: Direction::Input, rules: [LocalRule::Length { min: 0, max: 64 }] };
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// One genuine group on one immutable native demand, without replay.
    pub struct EvaluateBatch {
        projection: { name: "EvaluateBatch", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact prepared context.
        pub handle: Handle => { tag: 1, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Shares the scalar invocation sequence; repeated/stale operations refuse.
        pub invocation: u64 => { tag: 2, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
        /// Ordered identified points; framing bounds never grant scientific permission.
        pub points: Vec<BatchPoint> => { tag: 3, direction: Direction::Input, rules: [LocalRule::Length { min: 1, max: 4096 }] };
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Complete numerical group point; exact extents are checked against the prepared demand.
    pub struct BatchProduct {
        projection: { name: "BatchProduct", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Selected physical values.
        pub values: Vec<f64> => { tag: 1, direction: Direction::Output, rules: [LocalRule::Length { min: 1, max: 65536 }] };
        /// Prepared first product.
        pub jacobian: Vec<f64> => { tag: 2, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 65536 }] };
        /// Prepared second product.
        pub hessians: Vec<f64> => { tag: 3, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 65536 }] };
    }
}
pse_contract::projection_enum! {
    #[derive(Clone, Debug)]
    /// A scientific point refusal leaves its sibling products readable.
    pub enum BatchResult {
        projection: { name: "BatchResult", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Owner-completed demanded product.
        Product(BatchProduct) => 1;
        /// Original point refusal; never inferred from a group-wide C error code.
        Refused(Failure) => 2;
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Exactly one attributable outcome for an original point.
    pub struct BatchOutcome {
        projection: { name: "BatchOutcome", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Original group-local point identity.
        pub id: SemanticId => { tag: 1, direction: Direction::Output, rules: [] };
        /// Independent complete product or original refusal.
        pub result: BatchResult => { tag: 2, direction: Direction::Output, rules: [] };
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Attributable completion of the exact original native group.
    pub struct Batch {
        projection: { name: "Batch", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Completing context.
        pub handle: Handle => { tag: 1, direction: Direction::Output, rules: [] };
        /// Completing shared invocation sequence identity.
        pub invocation: u64 => { tag: 2, direction: Direction::Output, rules: [] };
        /// Original order with exactly one outcome per point.
        pub points: Vec<BatchOutcome> => { tag: 3, direction: Direction::Output, rules: [LocalRule::Length { min: 1, max: 4096 }] };
    }
}
pse_contract::projection_struct! {
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
pse_contract::projection_enum! {
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
        /// Every failed provider retirement, with original admitted keys.
        ProviderRetirement {
            /// Independent failures in attempted order, never deduplicated.
            failures: Vec<KeyedFailure> => { tag: 1, direction: Direction::Output, rules: [LocalRule::Length { min: 1, max: 64 }] };
        } => 9;
        /// Original provider operation and independently failed cleanup.
        ProviderSettlement {
            /// Original scientific operation failure.
            operation: Box<Failure> => { tag: 1, direction: Direction::Output, rules: [LocalRule::Required] };
            /// Independently attempted retirement failure.
            cleanup: Box<Failure> => { tag: 2, direction: Direction::Output, rules: [LocalRule::Required] };
        } => 10;
        /// A caught arbitrary provider finisher panic.
        ProviderRetirementPanic(String) => 11;
        /// An opaque source observation; it never reconstructs a native owner or error.
        Diagnostic(Box<pse_model::diagnostic::BoundaryDiagnostic>) => 12;
        /// Original worker action and all independently failed physical teardowns.
        Retirement {
            /// Complete ordered child failures under a nonrecoverable aggregate.
            causes: Vec<Failure> => { tag: 1, direction: Direction::Output, rules: [LocalRule::Length { min: 1, max: 128 }] };
        } => 13;
        /// A child-native lane or original provider finish boundary panicked.
        NativePanic(String) => 14;
        /// A known recoverable singular scientific trial.
        Singular(String) => 15;
        /// An owner-certified derivative context changed.
        LinearizationChanged => 16;
        /// Known recoverable crossing of an owner-bound regime.
        RegimeCrossing {
            /// Original regime selector identity.
            selector: SemanticId => { tag: 1, direction: Direction::Output, rules: [] };
            /// Regime bound by the original derivative request.
            bound: SemanticId => { tag: 2, direction: Direction::Output, rules: [] };
            /// Regime selected by the refused trial.
            selected: SemanticId => { tag: 3, direction: Direction::Output, rules: [] };
        } => 17;
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// One original provider-bank retirement occurrence.
    pub struct KeyedFailure {
        projection: { name: "KeyedFailure", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact admitted bank membership key, never recomputed from a descriptor.
        pub key: ContentHash => { tag: 1, direction: Direction::Output, rules: [] };
        /// Original known failure shape or its explicitly opaque source observation.
        pub cause: Box<Failure> => { tag: 2, direction: Direction::Output, rules: [LocalRule::Required] };
    }
}
impl Failure {
    /// Semantic restoration has its own finite tree/retention allowance, after
    /// the generated codec's independent frame, nesting and allocation limits.
    #[cfg(any(test, feature = "property-coolprop"))]
    pub(super) fn within_limits(&self) -> bool {
        use pse_model::HeapUsage as _;
        let mut pending = vec![(self, 0usize)];
        let mut nodes = FAILURE_NODES;
        let mut text = FAILURE_RETAINED_TEXT;
        while let Some((failure, depth)) = pending.pop() {
            if depth > FAILURE_DEPTH || nodes == 0 {
                return false;
            }
            nodes -= 1;
            let bytes = match failure {
                Self::ProviderRetirement { failures } => {
                    if failures.is_empty() || failures.len() > 64 {
                        return false;
                    }
                    pending.extend(
                        failures
                            .iter()
                            .map(|failure| (failure.cause.as_ref(), depth + 1)),
                    );
                    0
                }
                Self::ProviderSettlement { operation, cleanup } => {
                    pending.push((operation, depth + 1));
                    pending.push((cleanup, depth + 1));
                    0
                }
                Self::Retirement { causes } => {
                    if causes.is_empty() || causes.len() > 128 {
                        return false;
                    }
                    pending.extend(causes.iter().map(|cause| (cause, depth + 1)));
                    0
                }
                Self::Diagnostic(diagnostic) => {
                    let mut observed = vec![(diagnostic.as_ref(), depth + 1)];
                    while let Some((diagnostic, depth)) = observed.pop() {
                        if depth > FAILURE_DEPTH || nodes == 0 {
                            return false;
                        }
                        nodes -= 1;
                        if diagnostic.causes.len() > 128 {
                            return false;
                        }
                        observed.extend(diagnostic.causes.iter().map(|cause| (cause, depth + 1)));
                    }
                    diagnostic.heap_bytes()
                }
                Self::Resource(value)
                | Self::Contract(value)
                | Self::Trial(value)
                | Self::Terminal(value)
                | Self::UnknownCompletion(value)
                | Self::ProviderRetirementPanic(value)
                | Self::NativePanic(value)
                | Self::Singular(value) => value.len(),
                Self::OutsideEnvelope { axis, .. } => axis.len(),
                Self::Cancelled
                | Self::Deadline
                | Self::LinearizationChanged
                | Self::RegimeCrossing { .. } => 0,
            };
            let Some(remaining) = text.checked_sub(bytes) else {
                return false;
            };
            text = remaining;
        }
        true
    }
}
pse_contract::projection_enum! {
    #[derive(Clone, Debug)]
    /// Complete operation outcome on the original authenticated session.
    pub enum Response {
        projection: { name: "Response", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// The actual reconstructed native owner admitted this demand.
        Prepared(Prepared) => 1;
        /// The exact invocation produced a complete numerical product.
        Values(Values) => 2;
        /// A typed refusal, without a readable partial result.
        Failure(Failure) => 3;
        /// Native state was destroyed; process settlement remains the parent's duty.
        Closed(Handle) => 4;
        /// All session-native state has joined; the parent still owns process/I/O settlement.
        Shutdown => 5;
        /// One complete attributable native group, including independent point refusals.
        Batch(Batch) => 6;
    }
}
