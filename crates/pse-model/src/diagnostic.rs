// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Structured boundary failures retain model identity and observed values.
pub use crate::generated::enums::{
    DiagnosticSeverity as Severity, NativeBoundaryClass as BoundaryClass,
};
pub use pse_diagnostics::{DiagnosticCode, DiagnosticRule, DiagnosticStage, OperandContract};
use pse_ids::{ContentHash, SemanticId};

/// Registry-owned explicit nonfinite numerical failure observations.
pub use crate::generated::enums::DiagnosticNonfiniteObservation as NonfiniteObservation;
/// Finite physical evidence bound to its complete immutable quantity contract.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalObservation {
    /// Finite canonical magnitude.
    #[serde(
        deserialize_with = "finite_number",
        serialize_with = "serialize_finite_number"
    )]
    pub magnitude: f64,
    /// Registered complete quantity contract.
    pub quantity: SemanticId,
    /// Declared canonical unit.
    pub unit: SemanticId,
    /// Immutable interpretation context.
    pub context: ContentHash,
}
/// Evidence distinguishes absence, finite numerical/physical values and nonfinite failures.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Observation {
    /// Evidence was not available.
    Missing,
    /// A finite numerical observation (not necessarily a physical magnitude).
    Real(
        #[serde(
            deserialize_with = "finite_number",
            serialize_with = "serialize_finite_number"
        )]
        f64,
    ),
    /// Explicitly tagged nonfinite numerical failure evidence.
    Nonfinite(NonfiniteObservation),
    /// Finite physical evidence bound to its complete immutable quantity contract.
    Physical(PhysicalObservation),
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
    if value.is_finite() {
        Ok(value)
    } else {
        Err(serde::de::Error::custom(
            "ordinary diagnostic magnitudes must be finite",
        ))
    }
}
fn serialize_finite_number<S: serde::Serializer>(
    value: &f64,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if value.is_finite() {
        serializer.serialize_f64(*value)
    } else {
        Err(serde::ser::Error::custom(
            "nonfinite numerical observations require an explicit tag",
        ))
    }
}
impl Observation {
    /// Capture every IEEE value without losing infinity/NaN in JSON.
    pub fn number(value: f64) -> Self {
        if value.is_finite() {
            Self::Real(value)
        } else {
            Self::Nonfinite(if value.is_nan() {
                NonfiniteObservation::Nan
            } else if value.is_sign_positive() {
                NonfiniteObservation::PositiveInfinity
            } else {
                NonfiniteObservation::NegativeInfinity
            })
        }
    }
}
pub(crate) fn serialize_numeric_observation<S: serde::Serializer>(
    value: &f64,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serde::Serialize::serialize(&Observation::number(*value), serializer)
}
pub(crate) fn deserialize_numeric_observation<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<f64, D::Error> {
    match <Observation as serde::Deserialize>::deserialize(deserializer)? {
        Observation::Real(value) => Ok(value),
        Observation::Nonfinite(NonfiniteObservation::Nan) => Ok(f64::NAN),
        Observation::Nonfinite(NonfiniteObservation::PositiveInfinity) => Ok(f64::INFINITY),
        Observation::Nonfinite(NonfiniteObservation::NegativeInfinity) => Ok(f64::NEG_INFINITY),
        _ => Err(serde::de::Error::custom(
            "scientific numeric inputs require a real or nonfinite observation",
        )),
    }
}
/// Named schema projection of the existing restricted numeric observation wire.
pub(crate) struct NumericObservationSchema;
impl schemars::JsonSchema for NumericObservationSchema {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "NumericObservation".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        numeric_observation_schema(generator)
    }
}
pub(crate) fn numeric_observation_schema(
    generator: &mut schemars::SchemaGenerator,
) -> schemars::Schema {
    schemars::json_schema!({"oneOf":[
        {"type":"object","required":["kind","value"],"properties":{"kind":{"const":"real"},"value":{"type":"number"}},"additionalProperties":false},
        {"type":"object","required":["kind","value"],"properties":{"kind":{"const":"nonfinite"},"value":generator.subschema_for::<NonfiniteObservation>()},"additionalProperties":false}
    ]})
}
impl From<pse_diagnostics::DiagnosticObservation> for Observation {
    fn from(value: pse_diagnostics::DiagnosticObservation) -> Self {
        use pse_diagnostics::DiagnosticObservation as O;
        match value {
            O::Missing => Self::Missing,
            O::Number(v) => Self::number(v),
            O::Integer(v) => Self::Integer(v),
            O::Boolean(v) => Self::Boolean(v),
            O::Text(v) => Self::Text(v),
            O::Contracts(v) => Self::Contracts(v),
        }
    }
}
/// Authored location attached by the owner that has the source map.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
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
#[derive(
    Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
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
#[derive(
    Clone, Debug, thiserror::Error, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[error("{stage}: {rule} ({class:?}; sources {sources:?})")]
#[serde(deny_unknown_fields)]
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
    pub fn with_code(mut self, code: DiagnosticCode) -> Self {
        self.code = code;
        self
    }
    /// Attach the selected immutable revision where the source owner lacked context.
    /// Previously attributed locations keep their original revision.
    #[must_use]
    pub fn with_revision(mut self, revision: ContentHash) -> Self {
        for location in &mut self.locations {
            if location.revision.is_none() {
                location.revision = Some(revision);
            }
        }
        self.causes = self
            .causes
            .into_iter()
            .map(|cause| cause.with_revision(revision))
            .collect();
        self
    }
    /// Coarse semantic class, derived from the detailed code.
    pub const fn failure_class(&self) -> pse_diagnostics::FailureClass {
        self.code.class()
    }
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
            + self
                .causes
                .iter()
                .map(crate::HeapUsage::heap_bytes)
                .sum::<usize>()
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
                            Observation::Contracts(contracts) => {
                                contracts.capacity() * size_of::<OperandContract>()
                                    + contracts
                                        .iter()
                                        .map(|c| c.indices.capacity() * size_of::<[[u8; 16]; 3]>())
                                        .sum::<usize>()
                            }
                            _ => 0,
                        }
                })
                .sum::<usize>()
    }
}
pse_diagnostics::impl_diagnostic! {
    BoundaryDiagnostic,
    code(this) { Some(this.code) },
    forward(_this) {None}, help(_this) {None}, related(this) { if this.causes.is_empty() { None } else { Some(Box::new(this.causes.iter().map(|cause| -> &dyn pse_diagnostics::TypedDiagnostic { cause }))) } }, source(_this) {None}
}

/// Upper owners preserve model-owned scientific payload while lower owners supply facts.
pub trait DiagnosticProjection: pse_diagnostics::TypedDiagnostic {
    /// Project structured evidence without classifying rendered messages or downcasting.
    fn boundary_diagnostic(&self, stage: DiagnosticStage) -> BoundaryDiagnostic {
        let mut diagnostic = project_facts(self.diagnostic_code(), self.diagnostic_facts(), stage);
        diagnostic
            .observations
            .entry("detail".into())
            .or_insert_with(|| Observation::Text(self.to_string()));
        if let Some(children) = self.diagnostic_children() {
            diagnostic.causes = children.map(|child| project_typed(child, stage)).collect();
        }
        diagnostic
    }
}
/// Recursively retain every aggregate occurrence in source order, including mixed codes.
pub fn project_typed(
    error: &dyn pse_diagnostics::TypedDiagnostic,
    stage: DiagnosticStage,
) -> BoundaryDiagnostic {
    let mut diagnostic = project_facts(error.diagnostic_code(), error.diagnostic_facts(), stage);
    diagnostic
        .observations
        .entry("detail".into())
        .or_insert_with(|| Observation::Text(error.to_string()));
    if let Some(children) = error.diagnostic_children() {
        diagnostic.causes = children.map(|child| project_typed(child, stage)).collect();
    }
    diagnostic
}
/// Project one source owner's code/facts. Disposition is independent of coarse semantic class.
pub fn project_facts(
    code: Option<DiagnosticCode>,
    facts: pse_diagnostics::DiagnosticFacts,
    stage: DiagnosticStage,
) -> BoundaryDiagnostic {
    let aggregate = code.is_none();
    let code = code.unwrap_or(DiagnosticCode::DiagnosticAggregate);
    let rule = facts.rule.unwrap_or(if aggregate {
        DiagnosticRule::DiagnosticAggregate
    } else {
        DiagnosticRule::WorkflowUnclassified
    });
    let mut diagnostic = BoundaryDiagnostic::new(
        disposition(code),
        stage,
        facts.sources.into_iter().map(SemanticId::from_bytes),
        rule,
    )
    .with_code(code);
    diagnostic.observations = facts
        .observations
        .into_iter()
        .map(|(key, value)| (key, value.into()))
        .collect();
    diagnostic.locations = facts
        .locations
        .into_iter()
        .map(|location| SourceLocation {
            source: SemanticId::from_bytes(location.source),
            revision: location.revision.map(ContentHash::from_bytes),
            path: location.path,
            name: location.name,
            start: location.start,
            end: location.end,
        })
        .collect();
    diagnostic
}
/// Exhaustive operation-boundary disposition of the detailed semantic code.
pub const fn disposition(code: DiagnosticCode) -> BoundaryClass {
    use pse_diagnostics::FailureClass as C;
    if matches!(
        code,
        DiagnosticCode::AuthoringParseBudget | DiagnosticCode::AuthoringBudget
    ) {
        return BoundaryClass::ResourceLimit;
    }
    match code.class() {
        C::AuthoringParse
        | C::AuthoringReference
        | C::ValidationInvariant
        | C::CompileMath
        | C::CompileDiscretization
        | C::CompileFeature
        | C::CompileLaw
        | C::CompileProperty
        | C::KernelUnboundParameter
        | C::ConfigInvalid
        | C::UserModel => BoundaryClass::InvalidModel,
        C::CapabilityBackend => BoundaryClass::Unsupported,
        C::PlanInitialization
        | C::SolveInfeasible
        | C::SolveLocallyInfeasible
        | C::SolveUnbounded
        | C::SolveLimit
        | C::SolveSolverError => BoundaryClass::Numerical,
        C::SolveEvaluationError => BoundaryClass::TrialRejected,
        C::RuntimeCancelled => BoundaryClass::Cancelled,
        C::RuntimeTimeout | C::RuntimeResourceLimit => BoundaryClass::ResourceLimit,
        C::RuntimeInfrastructure => BoundaryClass::Infrastructure,
        C::InternalInvariant => BoundaryClass::Internal,
    }
}
impl DiagnosticProjection for BoundaryDiagnostic {
    fn boundary_diagnostic(&self, _stage: DiagnosticStage) -> BoundaryDiagnostic {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_codec_preserves_tagged_nonfinite_and_missing_observations() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let encoded = serde_json::to_string(&Observation::number(value)).unwrap();
            assert!(!encoded.contains("null"));
            assert!(matches!(
                serde_json::from_str::<Observation>(&encoded).unwrap(),
                Observation::Nonfinite(_)
            ));
            assert!(serde_json::to_string(&Observation::Real(value)).is_err());
        }
        assert!(matches!(
            serde_json::from_str::<Observation>(r#"{"kind":"missing"}"#).unwrap(),
            Observation::Missing
        ));
        assert!(serde_json::from_str::<Observation>(r#"{"kind":"real","value":null}"#).is_err());
    }
    #[test]
    fn diagnostic_code_is_authoritative_over_boundary_disposition() {
        let d = BoundaryDiagnostic::new(
            BoundaryClass::Incompatible,
            DiagnosticStage::Workflow,
            [],
            DiagnosticRule::NativeReuse,
        )
        .with_code(DiagnosticCode::NativeReuse);
        assert_eq!(
            d.failure_class(),
            pse_diagnostics::FailureClass::CapabilityBackend
        );
        assert_eq!(
            pse_diagnostics::TypedDiagnostic::diagnostic_code(&d),
            Some(DiagnosticCode::NativeReuse)
        );
        let mut json = serde_json::to_value(&d).unwrap();
        json["rule"] = "native.reuze".into();
        assert!(serde_json::from_value::<BoundaryDiagnostic>(json).is_err());
        let mut json = serde_json::to_value(&d).unwrap();
        json["unrecognized_fact"] = serde_json::json!(true);
        assert!(serde_json::from_value::<BoundaryDiagnostic>(json).is_err());
        assert!(
            serde_json::from_value::<SourceLocation>(serde_json::json!({
                "source": SemanticId::NIL, "path": "root.x", "name": null,
                "start": null, "end": null, "unrecognized_fact": true
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<OperandContract>(serde_json::json!({
                "quantity": SemanticId::NIL.as_bytes(), "indices": [], "unrecognized_fact": true
            }))
            .is_err()
        );
    }
    #[test]
    fn closed_taxonomy_roundtrips_every_rule_stage_and_authoritative_code() {
        for &rule in DiagnosticRule::ALL {
            let encoded = serde_json::to_vec(&rule).unwrap();
            assert_eq!(
                serde_json::from_slice::<DiagnosticRule>(&encoded).unwrap(),
                rule
            );
            assert_eq!(
                DiagnosticCode::parse(rule.code().as_str()),
                Some(rule.code())
            );
            let diagnostic = BoundaryDiagnostic::new(
                BoundaryClass::Internal,
                DiagnosticStage::Workflow,
                [],
                rule,
            );
            assert_eq!(diagnostic.failure_class(), rule.code().class());
        }
        for &stage in DiagnosticStage::ALL {
            let encoded = serde_json::to_vec(&stage).unwrap();
            assert_eq!(
                serde_json::from_slice::<DiagnosticStage>(&encoded).unwrap(),
                stage
            );
        }
        for &code in DiagnosticCode::ALL {
            let encoded = serde_json::to_vec(&code).unwrap();
            assert_eq!(
                serde_json::from_slice::<DiagnosticCode>(&encoded).unwrap(),
                code
            );
        }
        assert!(serde_json::from_str::<DiagnosticRule>("\"native.reuze\"").is_err());
        assert!(serde_json::from_str::<DiagnosticStage>("\"made-up-boundary\"").is_err());
        assert!(serde_json::from_str::<DiagnosticCode>("\"unknown.code\"").is_err());
    }
    #[test]
    fn scientific_input_codec_preserves_ieee_evidence_and_rejects_nonnumeric_tags() {
        for value in [3.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let input = crate::applicability::Input {
                name: "temperature".into(),
                value,
                quantity_type: SemanticId::from_bytes([1; 16]),
            };
            let encoded = serde_json::to_vec(&input).unwrap();
            let decoded: crate::applicability::Input = serde_json::from_slice(&encoded).unwrap();
            assert!(if value.is_nan() {
                decoded.value.is_nan()
            } else {
                decoded.value == value
            });
            let mut json = serde_json::to_value(&input).unwrap();
            for bad in [
                serde_json::json!({"kind":"missing"}),
                serde_json::json!({"kind":"text","value":"three"}),
                serde_json::json!({"kind":"boolean","value":true}),
            ] {
                json["value"] = bad;
                assert!(
                    serde_json::from_value::<crate::applicability::Input>(json.clone()).is_err()
                );
            }
            let mut json = serde_json::to_value(&input).unwrap();
            json["unrecognized_fact"] = serde_json::json!(true);
            assert!(serde_json::from_value::<crate::applicability::Input>(json).is_err());
        }
        let schema = schemars::schema_for!(crate::applicability::Input);
        assert_eq!(
            schema.as_value()["properties"]["value"]["$ref"],
            "#/$defs/NumericObservation"
        );
        let variants = &schema.as_value()["$defs"]["NumericObservation"]["oneOf"];
        assert_eq!(variants.as_array().unwrap().len(), 2);
        assert_eq!(variants[0]["properties"]["kind"]["const"], "real");
        assert_eq!(variants[1]["properties"]["kind"]["const"], "nonfinite");
    }
    #[test]
    fn physical_observation_requires_finite_magnitude_and_context() {
        assert!(
            serde_json::from_str::<Observation>(r#"{"kind":"physical","value":{"magnitude":1.0}}"#)
                .is_err()
        );
    }
}

/// Sized owned adapter preserves a source diagnostic across library/error trait boundaries.
#[derive(Debug)]
pub struct DiagnosticCause(Box<dyn DiagnosticProjection + Send + Sync>);
impl DiagnosticCause {
    /// Retain the original typed error, never its rendered string as authority.
    pub fn new(error: impl DiagnosticProjection + Send + Sync + 'static) -> Self {
        Self(Box::new(error))
    }
    /// Borrow the original error for library tooling, without replacing typed projection.
    pub fn as_error(&self) -> &(dyn std::error::Error + 'static) {
        self.0.as_ref()
    }
    /// Borrow the original source owner's projection.
    pub fn as_ref(&self) -> &(dyn DiagnosticProjection + Send + Sync) {
        self.0.as_ref()
    }
}
impl std::fmt::Display for DiagnosticCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
impl std::error::Error for DiagnosticCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}
pse_diagnostics::impl_diagnostic! {
    DiagnosticCause, code(_this) {None}, forward(this) {Some(this.0.as_ref())},
    help(_this) {None}, related(_this) {None}, source(this) {Some(this.0.as_ref())}
}
impl DiagnosticProjection for DiagnosticCause {
    fn boundary_diagnostic(&self, stage: DiagnosticStage) -> BoundaryDiagnostic {
        self.0.boundary_diagnostic(stage)
    }
}
