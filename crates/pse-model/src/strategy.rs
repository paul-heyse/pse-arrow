// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared finite numerical composition, independent of mathematical/native execution.
//! Applicability and actual support belong to the producers named by the consumed hashes.
//! A declaration grants numerical operations and transitions; it grants no scientific result.
pub use crate::generated::enums::{
    NativeBackend as Backend, NativeStartPolicy as StartPolicy,
    NumericalAccuracyClass as AccuracyClass, NumericalBranchPolicy as BranchKind,
    NumericalMechanism as MechanismKind, NumericalPhase as Phase, NumericalPosition as Position,
    NumericalScope as Scope, NumericalStartOrigin as StartOrigin,
    NumericalTransition as Transition,
};
use crate::{HeapUsage, ModelError};
use pse_ids::ContentHash;

/// Effective admitted native settings, interpreted by their existing native owner.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ProfileRef {
    /// Explicit implementation; changing it requires another declared reference.
    pub backend: Backend,
    /// Complete effective method/settings identity, including set versus unset options.
    pub key: ContentHash,
}
/// Entry intent and separately declared permissions for later starts in this task.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StartRules {
    /// Existing request's entry policy; a strategy cannot reinterpret its precedence.
    pub policy: StartPolicy,
    /// Permitted replacement origins. A fresh task-produced proposal may precede
    /// a non-explicit entry; inherited products still obey the entry policy.
    pub recovery: Vec<StartOrigin>,
}
impl StartRules {
    /// Origin permission only. Compatibility, result permission and explicit identity are
    /// independently checked by the start owner; an accepted label cannot establish them.
    pub fn permits_entry(&self, origin: StartOrigin, inherited: bool) -> bool {
        match self.policy {
            StartPolicy::Explicit => matches!(origin, StartOrigin::Explicit),
            StartPolicy::NoPriorStart => {
                !inherited
                    && (matches!(origin, StartOrigin::Specification)
                        || self.recovery.contains(&origin)
                            && !matches!(origin, StartOrigin::Explicit))
            }
            StartPolicy::PreviousAccepted => {
                matches!(origin, StartOrigin::Accepted)
                    || !inherited
                        && (matches!(origin, StartOrigin::Specification)
                            || self.recovery.contains(&origin)
                                && !matches!(origin, StartOrigin::Explicit))
            }
        }
    }
    /// NoPriorStart also excludes inherited products on later rungs. A product freshly
    /// produced in this task may serve a separately declared recovery origin.
    pub fn permits_recovery(&self, origin: StartOrigin, inherited: bool) -> bool {
        !(self.policy == StartPolicy::NoPriorStart && inherited) && self.recovery.contains(&origin)
    }
}
/// Explicit originating path/sheet and the consumed transport/orientation witnesses.
/// Supplying identities is not proof: the responsible path owner validates the products.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ConnectedPath {
    /// Original connected path identity.
    pub path: ContentHash,
    /// Selected root sheet; a shared regime label alone is insufficient.
    pub sheet: ContentHash,
    /// Transport evidence between the source and target bindings.
    pub transport: ContentHash,
    /// Path tangent/transport orientation evidence.
    pub orientation: ContentHash,
}
/// Root-selection meaning retained by every proposal and correction.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct BranchPolicy {
    /// Any qualified root, or continuation on a declared connected sheet.
    pub kind: BranchKind,
    /// Required precisely for the connected policy.
    pub connected: Option<ConnectedPath>,
}
impl BranchPolicy {
    /// A task imposing no connected-path obligation.
    pub const fn any_qualified() -> Self {
        Self {
            kind: BranchKind::AnyQualified,
            connected: None,
        }
    }
    /// Refuse an unbound connected policy or a contradictory unused binding.
    pub fn validate(&self) -> Result<(), ModelError> {
        if matches!(
            (self.kind, self.connected),
            (BranchKind::AnyQualified, None) | (BranchKind::Connected, Some(_))
        ) {
            Ok(())
        } else {
            Err(crate::malformed(
                "branch policy contradicts its connected-path binding",
            ))
        }
    }
}
/// Additional finite work allowances. None imposes no additional counter cap under the
/// enclosing finite ExecutionScope; it never denotes an observed zero or a new deadline.
/// Every supplied counter is finite; Some(0) expressly prohibits that operation.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct WorkLimits {
    /// Maximum executions, including the first attempt. A task requires at least one.
    pub attempts: u64,
    /// Evaluations admitted by the charging owner, including rejected trials.
    pub evaluations: Option<u64>,
    /// Native/library iterations, where the operation exposes them.
    pub iterations: Option<u64>,
    /// Numerical factor constructions, distinct from structural analysis/setup reuse.
    pub factorizations: Option<u64>,
    /// Verification/proof operations accounted by their owning verifier.
    pub proof_steps: Option<u64>,
}
impl WorkLimits {
    /// Validate the enclosing execution allowance; zero work counters remain meaningful.
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.attempts == 0 {
            Err(crate::malformed(
                "numerical task requires a positive finite attempt allowance",
            ))
        } else {
            Ok(())
        }
    }
}
/// Actual observations. Unknown counters remain None; these are neither allowances nor
/// reservations. Inclusive nested callback work is reported by its one charging owner.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct WorkObservation {
    /// Executions actually started.
    pub attempts: u64,
    /// Observed total evaluations, or unavailable measurement.
    pub evaluations: Option<u64>,
    /// Observed iterations, or unavailable measurement.
    pub iterations: Option<u64>,
    /// Observed numerical factor constructions, or unavailable measurement.
    pub factorizations: Option<u64>,
    /// Observed proof operations, or unavailable measurement.
    pub proof_steps: Option<u64>,
}
/// A work event with one owner; the effectful ledger prevents duplicate charging.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct WorkCharge {
    /// Preparation/native/assessment work remains distinguishable.
    pub phase: Phase,
    /// Task, durable occurrence, mechanism or attempt allowance being charged.
    pub scope: Scope,
    /// Stable identity of the actual operation owning these inclusive counters.
    pub charging_owner: ContentHash,
    /// Actual observations, retaining unknown counters.
    pub observed: WorkObservation,
}
/// Consumed accuracy of one operation. Its product identity is declared by the math owner
/// and includes the actual output/action, derivative order/source and coordinate meaning.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AccuracyDemand {
    /// Exact operation/product demanded; no capability table is inferred from this hash.
    pub product: ContentHash,
    /// Coordinate/error normalization in which the allowance is stated.
    pub normalization: ContentHash,
    /// Finite nonnegative error allowance; zero demands exact evidence.
    pub allowance: f64,
    /// Evidence class required by this operation, separate from original physical tolerances.
    pub class: AccuracyClass,
}
impl AccuracyDemand {
    /// Accuracy requirements cannot ask unresolved evidence to establish a bound.
    pub fn validate(&self) -> Result<(), ModelError> {
        if !self.allowance.is_finite()
            || self.allowance < 0.0
            || self.class == AccuracyClass::Unresolved
        {
            Err(crate::malformed(
                "consumed accuracy requires a finite allowance and established evidence class",
            ))
        } else {
            Ok(())
        }
    }
}
/// Actual error evidence; exact derivative provenance alone does not establish zero error.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AccuracyEvidence {
    /// Exact output/action and derivative product covered by the observation.
    pub product: ContentHash,
    /// Error normalization covered by the observation.
    pub normalization: ContentHash,
    /// Certification, numerical estimate, or unresolved evidence.
    pub class: AccuracyClass,
    /// Bound/estimate when available, retaining unknown rather than inventing zero.
    pub error: Option<f64>,
}
impl AccuracyEvidence {
    /// Check the consumed contract without converting estimated evidence into certification.
    pub fn satisfies(&self, demand: &AccuracyDemand) -> bool {
        demand.validate().is_ok()
            && self.product == demand.product
            && self.normalization == demand.normalization
            && match demand.class {
                AccuracyClass::Certified => self.class == AccuracyClass::Certified,
                AccuracyClass::Estimated => matches!(
                    self.class,
                    AccuracyClass::Certified | AccuracyClass::Estimated
                ),
                AccuracyClass::Unresolved => false,
            }
            && self
                .error
                .is_some_and(|error| error.is_finite() && error >= 0.0 && error <= demand.allowance)
    }
}
/// Actual consumed dependencies of a semantic product. Omitted fields were not consumed;
/// producer ownership determines compatibility, including lawful transport between values.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SemanticProductKey {
    /// Original immutable structural identity.
    pub structure: ContentHash,
    /// Bound original problem identity.
    pub binding: ContentHash,
    /// Frozen numerical policy, when consumed.
    pub numerical_policy: Option<ContentHash>,
    /// Coordinate normalization, when consumed.
    pub normalization: Option<ContentHash>,
    /// Point actually consumed, distinct from a future proposal's target.
    pub point: Option<ContentHash>,
    /// Parameter values actually consumed.
    pub parameters: Option<ContentHash>,
    /// Original/derived correspondence, when consumed.
    pub derivation: Option<ContentHash>,
    /// Actual selected sheet/branch evidence, when consumed.
    pub branch: Option<ContentHash>,
    /// Accuracy/order/source demand actually consumed.
    pub accuracy: Option<ContentHash>,
}
/// A real produced numerical product with the exact source and point it covers.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductEvidence {
    /// Source, binding, normalization, point and parameter dependencies actually consumed.
    pub source: SemanticProductKey,
    /// Actual derivative order; exact derivative provenance is separate from error class.
    pub derivative_order: u8,
    /// Selected branch meaning, validated by the producing owner.
    pub branch: BranchPolicy,
    /// Actual bound/estimate produced, never a preparation capability label.
    pub accuracy: AccuracyEvidence,
}
/// The consumer's exact dependency contract. No field is inferred from capabilities.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductDemand {
    /// The immutable dependencies and actual point the consumer requires.
    pub source: SemanticProductKey,
    /// Required derivative order.
    pub derivative_order: u8,
    /// Required branch meaning.
    pub branch: BranchPolicy,
    /// Required product/error normalization and evidence class.
    pub accuracy: AccuracyDemand,
}
/// A producer's finite output obligation, before the output point or receipt exists.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductionDemand {
    /// Frozen producer dependencies. An absent point names a point selected during execution.
    pub source: SemanticProductKey,
    /// The action order this producer must actually establish.
    pub derivative_order: u8,
    /// Original branch permission required of the output.
    pub branch: BranchPolicy,
    /// Finite error allowance in the source's declared normalization.
    pub allowance: f64,
    /// Required evidence class; support never supplies the future receipt.
    pub class: AccuracyClass,
}
impl ProductionDemand {
    /// Validate production permission independently of a future output certificate.
    pub fn validate(&self) -> Result<(), ModelError> {
        self.branch.validate()?;
        if self.source.normalization.is_none()
            || !self.allowance.is_finite()
            || self.allowance < 0.0
            || self.class == AccuracyClass::Unresolved
        {
            return Err(crate::malformed(
                "production requires normalization, finite allowance and established evidence class",
            ));
        }
        Ok(())
    }
    /// Check actual evidence after production without loosening its frozen dependencies.
    pub fn admits(&self, evidence: &ProductEvidence) -> bool {
        let mut source = self.source;
        if source.point.is_none() {
            source.point = evidence.source.point;
        }
        if source.accuracy.is_none() {
            source.accuracy = evidence.source.accuracy;
        }
        self.validate().is_ok()
            && source == evidence.source
            && self.derivative_order == evidence.derivative_order
            && self.branch == evidence.branch
            && evidence.accuracy.satisfies(&AccuracyDemand {
                product: evidence.accuracy.product,
                normalization: self
                    .source
                    .normalization
                    .unwrap_or(evidence.accuracy.normalization),
                allowance: self.allowance,
                class: self.class,
            })
    }
}
impl ProductEvidence {
    /// Exact dependency match followed by the owned accuracy check.
    pub fn satisfies(&self, demand: &ProductDemand) -> bool {
        self.source == demand.source
            && self.derivative_order == demand.derivative_order
            && self.branch == demand.branch
            && self.branch.validate().is_ok()
            && self.source.point.is_some()
            && self.source.normalization == Some(self.accuracy.normalization)
            && self.accuracy.satisfies(&demand.accuracy)
    }
}
/// Input requirements are consumed before use; output contracts grant bounded production,
/// never possession of the future output's certificate.
#[derive(
    Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct OperationContract {
    /// Actual prerequisites consumed by this particular operation.
    pub inputs: Vec<ProductDemand>,
    /// Products this operation may attempt to refine under the supplied demands.
    pub outputs: Vec<ProductionDemand>,
}

/// A native payload needs stronger compatibility than its semantic point.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct NativeProductKey {
    /// Complete semantic dependencies consumed by this payload.
    pub semantic: SemanticProductKey,
    /// Actual native method/effective settings.
    pub profile: ProfileRef,
    /// Native/provider source identity.
    pub source: ContentHash,
    /// Native build/ABI identity.
    pub build: ContentHash,
    /// Native layout/sparsity/order identity.
    pub layout: ContentHash,
}
/// One operation description. Applicability, supplied derivatives and method mathematics
/// remain with its support producers; this declaration owns bounded composition only.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mechanism {
    /// Declared numerical operation, from the shared registry vocabulary.
    pub kind: MechanismKind,
    /// Preparation, execution or a later recovery rung.
    pub position: Position,
    /// Failure of required admission ends this task; optional absence can preserve its base.
    pub required: bool,
    /// Explicit effective native profile; absent lets the contextual route resolve the base.
    pub profile: Option<ProfileRef>,
    /// Exact producer-owned support contracts required by this operation.
    pub support: Vec<ContentHash>,
    /// Exact prerequisites and bounded production obligations of this operation only.
    #[serde(default)]
    pub operation: OperationContract,
    /// Additional local finite allowances under the enclosing task scope.
    pub limits: WorkLimits,
    /// Permitted proposal origins for this mechanism; entry rules still take precedence.
    pub starts: Vec<StartOrigin>,
    /// Declared transitions. Terminal stops remain obligatory, regardless of numerical history.
    pub transitions: Vec<Transition>,
}
/// Registry-owned requested numerical composition.
pub use crate::generated::enums::NumericalCompositionPolicy as CompositionPolicy;
/// Hard caller constraints retained by automatic and explicitly declared composition.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompositionRequest {
    /// Automatic resolution or explicit declaration binding.
    pub policy: CompositionPolicy,
    /// Original root-selection contract.
    pub branch: BranchPolicy,
    /// Additional finite counters; absent retains the admitted caller controls.
    pub limits: Option<WorkLimits>,
    /// Permitted later start origins. Empty grants no replacement origin.
    pub recovery: Vec<StartOrigin>,
}
impl Default for CompositionRequest {
    fn default() -> Self {
        Self {
            policy: CompositionPolicy::Auto,
            branch: BranchPolicy::any_qualified(),
            limits: None,
            recovery: Vec::new(),
        }
    }
}
impl CompositionRequest {
    /// Validate immutable requested constraints before resolving capabilities.
    pub fn validate(&self) -> Result<(), ModelError> {
        self.branch.validate()?;
        if let Some(limits) = self.limits {
            limits.validate()?;
        }
        unique(&self.recovery, "duplicate composition recovery origin")
    }
}
/// Immutable ordered declaration. Ordering affects composition and identity; no backend
/// ranking or universal direct-first escalation is implied. With no request, the runtime
/// constructs direct(start, limits) from its existing admitted controls and enclosing scope.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NumericalStrategy {
    /// Entry intent and explicit later-start permissions.
    pub start: StartRules,
    /// Original root-selection contract.
    pub branch: BranchPolicy,
    /// Ordered composition. At least one execution operation is required.
    pub mechanisms: Vec<Mechanism>,
    /// Existing admitted task/occurrence finite limits, never refreshed at a new rung.
    pub limits: WorkLimits,
}
/// Current strategy wire document. Historical bodies are not silently reinterpreted.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NumericalStrategyDocument {
    /// Exact current document version, checked before decoding its body.
    pub version: crate::document::Version<3>,
    /// Validated finite operation declaration.
    pub strategy: NumericalStrategy,
}
impl NumericalStrategyDocument {
    /// Construct the current document only from a valid declaration.
    pub fn new(strategy: NumericalStrategy) -> Result<Self, ModelError> {
        strategy.validate()?;
        Ok(Self {
            version: crate::document::Version,
            strategy,
        })
    }
}
impl NumericalStrategy {
    /// Complete minimal strategy: one original execution, no preparation or automatic recovery.
    /// Limits come from admitted caller controls; this constructor invents no numerical ceiling.
    pub fn direct(start: StartPolicy, limits: WorkLimits) -> Self {
        let starts = match start {
            StartPolicy::NoPriorStart => vec![StartOrigin::Specification],
            StartPolicy::Explicit => vec![StartOrigin::Explicit],
            StartPolicy::PreviousAccepted => {
                vec![StartOrigin::Accepted, StartOrigin::Specification]
            }
        };
        Self {
            start: StartRules {
                policy: start,
                recovery: Vec::new(),
            },
            branch: BranchPolicy::any_qualified(),
            mechanisms: vec![Mechanism {
                kind: MechanismKind::Direct,
                position: Position::Execution,
                required: true,
                profile: None,
                support: Vec::new(),
                operation: OperationContract::default(),
                limits,
                starts,
                transitions: vec![Transition::Finish, Transition::Stop],
            }],
            limits,
        }
    }
    /// Validate declaration meaning only; resolution binds actual support and scoped resources.
    pub fn validate(&self) -> Result<(), ModelError> {
        self.limits.validate()?;
        self.branch.validate()?;
        unique(&self.start.recovery, "duplicate recovery start origin")?;
        if !self.mechanisms.iter().any(|mechanism| {
            mechanism.position == Position::Execution && mechanism.limits.attempts > 0
        }) {
            return Err(crate::malformed(
                "numerical strategy requires an execution mechanism",
            ));
        }
        for mechanism in &self.mechanisms {
            if mechanism.required {
                mechanism.limits.validate()?;
            }
            unique(&mechanism.support, "duplicate required support contract")?;
            unique(&mechanism.starts, "duplicate mechanism start origin")?;
            unique(&mechanism.transitions, "duplicate numerical transition")?;
            if !mechanism.transitions.contains(&Transition::Stop) {
                return Err(crate::malformed(
                    "every numerical mechanism must permit terminal stop",
                ));
            }
            if mechanism.position == Position::Execution && mechanism.starts.is_empty() {
                return Err(crate::malformed(
                    "execution mechanism has no admitted start origins",
                ));
            }
            for input in &mechanism.operation.inputs {
                input.accuracy.validate()?;
                input.branch.validate()?;
                if input.source.point.is_none()
                    || input.source.normalization != Some(input.accuracy.normalization)
                {
                    return Err(crate::malformed(
                        "operation input must name its actual point and normalization",
                    ));
                }
            }
            for output in &mechanism.operation.outputs {
                output.validate()?;
            }
        }
        Ok(())
    }
    /// Complete declaration identity; invalid declarations never receive an admitted key.
    pub fn key(&self) -> Result<ContentHash, ModelError> {
        self.validate()?;
        pse_ids::document::of(pse_ids::Frame::NumericalStrategyV3, self)
            .map_err(|error| ModelError::Malformed(error.to_string()))
    }
}

fn unique<T: Ord>(values: &[T], reason: &str) -> Result<(), ModelError> {
    let mut seen = std::collections::BTreeSet::new();
    if values.iter().any(|value| !seen.insert(value)) {
        Err(crate::malformed(reason))
    } else {
        Ok(())
    }
}
impl HeapUsage for NumericalStrategy {
    fn heap_bytes(&self) -> usize {
        self.start
            .recovery
            .capacity()
            .saturating_mul(size_of::<StartOrigin>())
            .saturating_add(
                self.mechanisms.iter().fold(
                    self.mechanisms
                        .capacity()
                        .saturating_mul(size_of::<Mechanism>()),
                    |bytes, mechanism| {
                        bytes
                            .saturating_add(
                                mechanism
                                    .operation
                                    .inputs
                                    .capacity()
                                    .saturating_mul(size_of::<ProductDemand>()),
                            )
                            .saturating_add(
                                mechanism
                                    .operation
                                    .outputs
                                    .capacity()
                                    .saturating_mul(size_of::<ProductionDemand>()),
                            )
                            .saturating_add(
                                mechanism
                                    .support
                                    .capacity()
                                    .saturating_mul(size_of::<ContentHash>()),
                            )
                            .saturating_add(
                                mechanism
                                    .starts
                                    .capacity()
                                    .saturating_mul(size_of::<StartOrigin>()),
                            )
                            .saturating_add(
                                mechanism
                                    .transitions
                                    .capacity()
                                    .saturating_mul(size_of::<Transition>()),
                            )
                    },
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    fn limits() -> WorkLimits {
        WorkLimits {
            attempts: 1,
            evaluations: None,
            iterations: Some(50),
            factorizations: None,
            proof_steps: None,
        }
    }
    #[test]
    fn direct_strategy_is_complete_and_has_no_implicit_preparation_or_retry() {
        for policy in [
            StartPolicy::NoPriorStart,
            StartPolicy::Explicit,
            StartPolicy::PreviousAccepted,
        ] {
            let strategy = NumericalStrategy::direct(policy, limits());
            strategy.validate().unwrap();
            assert_eq!(strategy.mechanisms.len(), 1);
            assert_eq!(strategy.mechanisms[0].kind, MechanismKind::Direct);
            assert_eq!(strategy.mechanisms[0].position, Position::Execution);
            assert!(strategy.mechanisms[0].required);
            assert_eq!(
                strategy.mechanisms[0].transitions,
                vec![Transition::Finish, Transition::Stop]
            );
            assert_eq!(strategy.limits, limits());
            assert!(strategy.start.recovery.is_empty());
            assert!(strategy.mechanisms[0].profile.is_none());
            assert!(
                strategy
                    .mechanisms
                    .iter()
                    .all(|mechanism| mechanism.operation.inputs.is_empty()
                        && mechanism.operation.outputs.is_empty())
            );
        }
    }
    #[test]
    fn entry_start_precedence_excludes_inherited_and_explicit_substitution() {
        let mut rules = StartRules {
            policy: StartPolicy::NoPriorStart,
            recovery: vec![StartOrigin::Accepted, StartOrigin::Predicted],
        };
        assert!(rules.permits_entry(StartOrigin::Specification, false));
        assert!(!rules.permits_entry(StartOrigin::Specification, true));
        assert!(!rules.permits_entry(StartOrigin::Accepted, true));
        assert!(rules.permits_entry(StartOrigin::Predicted, false));
        assert!(!rules.permits_entry(StartOrigin::Predicted, true));
        assert!(!rules.permits_entry(StartOrigin::Auxiliary, false));
        assert!(rules.permits_recovery(StartOrigin::Accepted, false));
        assert!(!rules.permits_recovery(StartOrigin::Accepted, true));
        assert!(!rules.permits_recovery(StartOrigin::Surrogate, false));
        rules.policy = StartPolicy::Explicit;
        assert!(rules.permits_entry(StartOrigin::Explicit, true));
        assert!(!rules.permits_entry(StartOrigin::Predicted, false));
        assert!(!rules.permits_entry(StartOrigin::Accepted, true));
        assert!(!rules.permits_entry(StartOrigin::Specification, false));
        rules.policy = StartPolicy::PreviousAccepted;
        assert!(rules.permits_entry(StartOrigin::Accepted, true));
        assert!(rules.permits_entry(StartOrigin::Specification, false));
        assert!(!rules.permits_entry(StartOrigin::Auxiliary, true));
    }
    #[test]
    fn declarations_refuse_missing_execution_terminal_stop_and_zero_task_allowance() {
        let direct = NumericalStrategy::direct(StartPolicy::NoPriorStart, limits());
        let mut missing = direct.clone();
        missing.mechanisms.clear();
        assert!(missing.validate().is_err());
        let mut preparation_only = direct.clone();
        preparation_only.mechanisms[0].position = Position::Preparation;
        assert!(preparation_only.validate().is_err());
        let mut no_stop = direct.clone();
        no_stop.mechanisms[0].transitions = vec![Transition::Recover];
        assert!(no_stop.validate().is_err());
        let mut no_budget = direct.clone();
        no_budget.limits.attempts = 0;
        assert!(no_budget.validate().is_err());
        let mut no_origin = direct;
        no_origin.mechanisms[0].starts.clear();
        assert!(no_origin.validate().is_err());
    }
    #[test]
    fn optional_empty_slice_preserves_base_and_unknown_work_is_not_observed_zero() {
        let mut strategy = NumericalStrategy::direct(StartPolicy::NoPriorStart, limits());
        let mut optional = strategy.mechanisms[0].clone();
        optional.kind = MechanismKind::KktPredictor;
        optional.position = Position::Preparation;
        optional.required = false;
        optional.limits.attempts = 0;
        optional.limits.evaluations = Some(0);
        optional.transitions = vec![Transition::Continue, Transition::Stop];
        strategy.mechanisms.insert(0, optional);
        strategy.validate().unwrap();
        strategy.mechanisms[0].required = true;
        assert!(strategy.validate().is_err());
        let unknown = WorkObservation {
            attempts: 1,
            evaluations: None,
            iterations: Some(0),
            factorizations: None,
            proof_steps: None,
        };
        let value = serde_json::to_value(unknown).unwrap();
        assert!(value["evaluations"].is_null());
        assert_eq!(value["iterations"], 0);
    }
    #[test]
    fn connected_branch_requires_its_bound_witnesses() {
        let mut branch = BranchPolicy {
            kind: BranchKind::Connected,
            connected: None,
        };
        assert!(branch.validate().is_err());
        branch.connected = Some(ConnectedPath {
            path: hash(1),
            sheet: hash(2),
            transport: hash(3),
            orientation: hash(4),
        });
        branch.validate().unwrap();
        branch.kind = BranchKind::AnyQualified;
        assert!(branch.validate().is_err());
    }
    #[test]
    fn accuracy_requires_matching_product_normalization_class_and_finite_error() {
        let mut demand = AccuracyDemand {
            product: hash(1),
            normalization: hash(2),
            allowance: 1e-6,
            class: AccuracyClass::Certified,
        };
        let mut evidence = AccuracyEvidence {
            product: hash(1),
            normalization: hash(2),
            class: AccuracyClass::Estimated,
            error: Some(1e-8),
        };
        assert!(!evidence.satisfies(&demand));
        evidence.class = AccuracyClass::Certified;
        assert!(evidence.satisfies(&demand));
        for error in [
            None,
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(-1.0),
            Some(1e-4),
        ] {
            evidence.error = error;
            assert!(!evidence.satisfies(&demand));
        }
        evidence.error = Some(1e-8);
        evidence.normalization = hash(3);
        assert!(!evidence.satisfies(&demand));
        evidence.normalization = hash(2);
        evidence.product = hash(3);
        assert!(!evidence.satisfies(&demand));
        evidence.product = hash(1);
        demand.class = AccuracyClass::Estimated;
        assert!(evidence.satisfies(&demand));
        for allowance in [f64::NAN, f64::INFINITY, -1.0] {
            demand.allowance = allowance;
            assert!(demand.validate().is_err());
        }
        demand.allowance = 1e-6;
        demand.class = AccuracyClass::Unresolved;
        assert!(demand.validate().is_err());
    }
    #[test]
    fn complete_strategy_identity_preserves_profiles_permissions_and_absent_counters() {
        let strategy = NumericalStrategy::direct(StartPolicy::NoPriorStart, limits());
        let key = strategy.key().unwrap();
        let mut explicit = strategy.clone();
        explicit.start.policy = StartPolicy::Explicit;
        explicit.mechanisms[0].starts = vec![StartOrigin::Explicit];
        assert_ne!(key, explicit.key().unwrap());
        let mut profiled = strategy.clone();
        profiled.mechanisms[0].profile = Some(ProfileRef {
            backend: Backend::Ipopt,
            key: hash(1),
        });
        assert_ne!(key, profiled.key().unwrap());
        let profiled_key = profiled.key().unwrap();
        profiled.mechanisms[0].profile.as_mut().unwrap().key = hash(2);
        assert_ne!(profiled_key, profiled.key().unwrap());
        let mut prohibited = strategy.clone();
        prohibited.limits.evaluations = Some(0);
        assert_ne!(key, prohibited.key().unwrap());
        let decoded: NumericalStrategy =
            serde_json::from_value(serde_json::to_value(&strategy).unwrap()).unwrap();
        assert_eq!(strategy, decoded);
        assert_eq!(key, decoded.key().unwrap());
    }
}

#[cfg(test)]
mod recovery_entry_tests {
    use super::*;
    #[test]
    fn fresh_declared_recovery_origin_can_enter_without_rewriting_explicit_precedence() {
        for policy in [
            StartPolicy::NoPriorStart,
            StartPolicy::PreviousAccepted,
            StartPolicy::Explicit,
        ] {
            let rules = StartRules {
                policy,
                recovery: vec![
                    StartOrigin::Predicted,
                    StartOrigin::Auxiliary,
                    StartOrigin::ModifiedSpecification,
                    StartOrigin::Surrogate,
                ],
            };
            for origin in &rules.recovery {
                assert_eq!(
                    rules.permits_entry(*origin, false),
                    policy != StartPolicy::Explicit
                );
                assert!(!rules.permits_entry(*origin, true));
            }
            assert_eq!(
                rules.permits_entry(StartOrigin::Explicit, false),
                policy == StartPolicy::Explicit
            );
        }
    }
}
