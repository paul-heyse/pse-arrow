// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Structured boundary failures retain model identity and observed values.
pub use crate::generated::enums::{
    DiagnosticSeverity as Severity, NativeBoundaryClass as BoundaryClass,
};
use pse_ids::{ContentHash, SemanticId};
pub use pse_diagnostics::{DiagnosticCode, DiagnosticRule, DiagnosticStage, OperandContract};

/// Explicit IEEE failure observations do not masquerade as physical magnitudes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonfiniteObservation {
    /// Not a number.
    Nan,
    /// Positive infinity.
    PositiveInfinity,
    /// Negative infinity.
    NegativeInfinity,
}
/// Evidence distinguishes absence, finite numerical/physical values and nonfinite failures.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Observation {
    /// Evidence was not available.
    Missing,
    /// A finite numerical observation (not necessarily a physical magnitude).
    Real(#[serde(deserialize_with = "finite_number")] f64),
    /// Explicitly tagged nonfinite numerical failure evidence.
    Nonfinite(NonfiniteObservation),
    /// Finite physical evidence bound to its complete immutable quantity contract.
    Physical {
        /// Finite canonical magnitude.
        #[serde(deserialize_with = "finite_number")]
        magnitude: f64,
        /// Registered complete quantity contract.
        quantity: SemanticId,
        /// Declared canonical unit.
        unit: SemanticId,
        /// Immutable interpretation context.
        context: ContentHash,
    },
    /// Exact counts or native codes.
    Integer(i64),
    /// A checked predicate.
    Boolean(bool),
    /// Native status or explanatory detail.
    Text(String),
    /// Ordered physical operand/free-index contracts, preserved structurally.
    Contracts(Vec<OperandContract>),
}
fn finite_number<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    let value = <f64 as serde::Deserialize>::deserialize(d)?;
    if value.is_finite() { Ok(value) } else { Err(serde::de::Error::custom("ordinary diagnostic magnitudes must be finite")) }
}
impl Observation {
    /// Capture every IEEE value without losing infinity/NaN in JSON.
    pub fn number(value: f64) -> Self {
        if value.is_finite() { Self::Real(value) }
        else { Self::Nonfinite(if value.is_nan() { NonfiniteObservation::Nan }
            else if value.is_sign_positive() { NonfiniteObservation::PositiveInfinity }
            else { NonfiniteObservation::NegativeInfinity }) }
    }
}
impl From<pse_diagnostics::DiagnosticObservation> for Observation {
    fn from(value: pse_diagnostics::DiagnosticObservation) -> Self {
        use pse_diagnostics::DiagnosticObservation as O;
        match value { O::Missing => Self::Missing, O::Number(v) => Self::number(v),
            O::Integer(v) => Self::Integer(v), O::Boolean(v) => Self::Boolean(v),
            O::Text(v) => Self::Text(v), O::Contracts(v) => Self::Contracts(v) }
    }
}
/// Authored location attached by the owner that has the source map.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    /// Original semantic occurrence or instance identity.
    pub source: SemanticId,
    /// Immutable source interpretation context; absence is explicitly unattributed.
    #[serde(default)]
    pub revision: Option<ContentHash>,
    /// Definition or document path, never a backend-local node identifier.
    pub path: String,
    /// Optional authored display name.
    pub name: Option<String>,
    /// UTF-8 starting byte offset, when supplied by the parser/compiler.
    pub start: Option<u32>,
    /// UTF-8 exclusive ending byte offset.
    pub end: Option<u32>,
}
/// What a rejected authored validity predicate guards (ADR-0123 Outcome 4, Plan 23 H5): its
/// layer and the declaration stating it; for the form and data layers the form it belongs
/// to, the parameter sets whose values bound it and the form's declared arguments it
/// constrains; for the closure layer the model members its range bounds. A fixture's
/// expected validity failure is matched against this lineage, not against a rule text.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ValidityLineage {
    /// The form layer (a function's own domain), the data layer (a declared envelope) or
    /// the closure layer (an annotated range).
    pub layer: crate::generated::enums::ModelingValidityLayer,
    /// The form itself, the relation or kind declaration declaring the envelope, or the
    /// closure range's annotation.
    pub source: SemanticId,
    /// The form: the function declaration whose evaluation was rejected; absent on the
    /// closure layer.
    pub form: Option<SemanticId>,
    /// The table rows or entities whose values bound the predicate, in order.
    pub sets: Vec<SemanticId>,
    /// Positions of the constrained arguments among the form's declared arguments.
    pub variables: Vec<u32>,
    /// The model members a closure range bounds; empty on the form and data layers.
    pub members: Vec<SemanticId>,
}
impl ValidityLineage {
    /// Owned heap extent.
    pub fn heap_bytes(&self) -> usize {
        (self.sets.capacity() + self.members.capacity()) * size_of::<SemanticId>()
            + self.variables.capacity() * size_of::<u32>()
    }
}
/// Stable structured cause shared by Rust and the public error projection.
#[derive(Clone, Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
#[error("{stage}: {rule} ({class:?}; sources {sources:?})")]
pub struct BoundaryDiagnostic {
    /// Detailed semantic identity, authoritative over every coarse projection.
    pub code: DiagnosticCode,
    /// Ordered retained cause envelopes, preserving repeated aggregate occurrences.
    #[serde(default)]
    pub causes: Vec<BoundaryDiagnostic>,
    /// Machine-readable disposition; callers never parse the message.
    pub class: BoundaryClass,
    /// Error, warning or information; a warning never makes a model invalid.
    #[serde(default = "error_severity")]
    pub severity: Severity,
    /// Operation boundary, such as selected admission or provider registration.
    pub stage: DiagnosticStage,
    /// All affected authored identities, in deterministic order.
    pub sources: Vec<SemanticId>,
    /// The violated named contract, not a replacement for source identities.
    pub rule: DiagnosticRule,
    /// Values observed while checking the contract.
    pub observations: std::collections::BTreeMap<String, Observation>,
    /// Available source locations. Empty means unattributed, not a guessed source.
    #[serde(default)]
    pub locations: Vec<SourceLocation>,
    /// The lineage of a rejected authored validity predicate; absent on every other finding.
    pub validity: Option<ValidityLineage>,
    /// Complete scientific evidence on demanded paths, including refused claims.
    #[serde(default)]
    pub applicability: Vec<crate::applicability::Observation>,
}
const fn error_severity() -> Severity {
    Severity::Error
}
impl BoundaryDiagnostic {
    /// Construct an attributable boundary error; add observations when available.
    pub fn new(
        class: BoundaryClass,
        stage: DiagnosticStage,
        sources: impl IntoIterator<Item = SemanticId>,
        rule: DiagnosticRule,
    ) -> Self {
        let mut sources: Vec<_> = sources.into_iter().collect();
        sources.sort_unstable();
        sources.dedup();
        Self {
            code: rule.code(),
            causes: Vec::new(),
            class,
            severity: Severity::Error,
            stage,
            sources,
            rule,
            observations: Default::default(),
            locations: Vec::new(),
            validity: None,
            applicability: Vec::new(),
        }
    }
    /// The source error's detailed identity is authoritative; disposition stays separate.
    #[must_use]
    pub fn with_code(mut self, code: DiagnosticCode) -> Self { self.code = code; self }
    /// Coarse semantic class, derived from the detailed code.
    pub const fn failure_class(&self) -> pse_diagnostics::FailureClass { self.code.class() }
    /// The same finding at another severity; the class is unchanged.
    #[must_use]
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
}
impl crate::HeapUsage for BoundaryDiagnostic {
    fn heap_bytes(&self) -> usize {
        self.applicability.capacity() * size_of::<crate::applicability::Observation>()
            + self
                .applicability
                .iter()
                .map(crate::applicability::Observation::retained_bytes)
                .sum::<usize>()
            + self.causes.capacity() * size_of::<Self>()
            + self.causes.iter().map(crate::HeapUsage::heap_bytes).sum::<usize>()
            + self.sources.capacity() * size_of::<SemanticId>()
            + self.locations.capacity() * size_of::<SourceLocation>()
            + self
                .validity
                .as_ref()
                .map_or(0, ValidityLineage::heap_bytes)
            + self
                .locations
                .iter()
                .map(|v| v.path.capacity() + v.name.as_ref().map_or(0, String::capacity))
                .sum::<usize>()
            + self
                .observations
                .iter()
                .map(|(name, value)| {
                    // A conservative BTree node allowance includes its inline key/value.
                    128 + name.capacity()
                        + match value {
                            Observation::Text(value) => value.capacity(),
                            _ => 0,
                        }
                })
                .sum::<usize>()
    }
}
pse_diagnostics::impl_diagnostic! {
    BoundaryDiagnostic,
    code(this) { Some(this.code) },
    forward(_this) {None}, help(_this) {None}, related(this) { if this.causes.is_empty() { None } else { Some(Box::new(this.causes.iter().map(|cause| cause as &dyn pse_diagnostics::TypedDiagnostic))) } }, source(_this) {None}
}

/// Upper owners preserve model-owned scientific payload while lower owners supply facts.
pub trait DiagnosticProjection: pse_diagnostics::TypedDiagnostic {
    /// Project structured evidence without classifying rendered messages or downcasting.
    fn boundary_diagnostic(&self, stage: DiagnosticStage) -> BoundaryDiagnostic {
        project_facts(self.diagnostic_code(), self.diagnostic_facts(), stage)
    }
}
/// Project one source owner's code/facts. Disposition is independent of coarse semantic class.
pub fn project_facts(code: Option<DiagnosticCode>, facts: pse_diagnostics::DiagnosticFacts, stage: DiagnosticStage) -> BoundaryDiagnostic {
    let code = code.unwrap_or(DiagnosticCode::InternalInvariant);
    let rule = facts.rule.unwrap_or(DiagnosticRule::WorkflowUnclassified);
    let mut diagnostic = BoundaryDiagnostic::new(disposition(code), stage,
        facts.sources.into_iter().map(SemanticId::from_bytes), rule).with_code(code);
    diagnostic.observations = facts.observations.into_iter().map(|(key, value)| (key, value.into())).collect();
    diagnostic.locations = facts.locations.into_iter().map(|location| SourceLocation {
        source: SemanticId::from_bytes(location.source), revision: location.revision.map(ContentHash::from_bytes),
        path: location.path, name: location.name, start: location.start, end: location.end,
    }).collect();
    diagnostic
}
/// Exhaustive operation-boundary disposition of the detailed semantic code.
pub const fn disposition(code: DiagnosticCode) -> BoundaryClass {
    use pse_diagnostics::FailureClass as C;
    match code.class() {
        C::AuthoringParse | C::AuthoringReference | C::ValidationInvariant | C::CompileMath | C::CompileDiscretization | C::CompileFeature | C::CompileLaw | C::CompileProperty | C::KernelUnboundParameter | C::ConfigInvalid | C::UserModel => BoundaryClass::InvalidModel,
        C::CapabilityBackend => BoundaryClass::Unsupported,
        C::PlanInitialization | C::SolveInfeasible | C::SolveLocallyInfeasible | C::SolveUnbounded | C::SolveLimit | C::SolveSolverError => BoundaryClass::Numerical,
        C::SolveEvaluationError => BoundaryClass::TrialRejected,
        C::RuntimeCancelled => BoundaryClass::Cancelled,
        C::RuntimeTimeout | C::RuntimeResourceLimit => BoundaryClass::ResourceLimit,
        C::RuntimeInfrastructure => BoundaryClass::Infrastructure,
        C::InternalInvariant => BoundaryClass::Internal,
    }
}
impl DiagnosticProjection for BoundaryDiagnostic {
    fn boundary_diagnostic(&self, _stage: DiagnosticStage) -> BoundaryDiagnostic { self.clone() }
}
