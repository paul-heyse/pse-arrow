// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::tuple::Tuple;
use miette::Diagnostic;
use pse_diagnostics::TypedDiagnostic;
use pse_model::diagnostic::{BoundaryDiagnostic, DiagnosticProjection, DiagnosticStage};
use pyo3::{exceptions::PyException, prelude::*};

// This declaration drives both the exception's runtime attributes and the narrow
// stub supplement for PyO3's native-exception introspection gap under abi3-py311.
macro_rules! inspection_exception {
    ($name:ident, $base:ty, { $($field:ident: $ty:ty),* $(,)? }) => {
        pyo3::create_exception!(_native, $name, $base);
        struct Details { $( $field: $ty, )* }
        impl Details {
            fn attach(self, value: &Bound<'_, pyo3::types::PyAny>) -> PyResult<()> {
                $( value.setattr(stringify!($field), self.$field)?; )*
                Ok(())
            }
        }
    };
}
inspection_exception!(InspectionError, PyException, {
    report: DiagnosticReport,
});

/// An authored occurrence and its optional UTF-8 source interval.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticSourceLocation(pse_model::diagnostic::SourceLocation);
#[pymethods]
impl DiagnosticSourceLocation {
    #[getter]
    fn source(&self) -> String {
        self.0.source.to_hex()
    }
    #[getter]
    fn revision(&self) -> Option<String> {
        self.0.revision.map(|revision| revision.to_hex())
    }
    #[getter]
    fn path(&self) -> &str {
        &self.0.path
    }
    #[getter]
    fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }
    #[getter]
    fn start(&self) -> Option<u32> {
        self.0.start
    }
    #[getter]
    fn end(&self) -> Option<u32> {
        self.0.end
    }
}

/// One original source in a native error chain.
#[pyclass(frozen, skip_from_py_object, get_all, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticCause {
    message: String,
}

/// One native leaf and its ordered execution contexts.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticContext {
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    message: String,
    contexts: Vec<String>,
    diagnostics: Vec<DiagnosticAnnotation>,
}
#[pymethods]
impl DiagnosticContext {
    #[getter]
    fn contexts(&self) -> Tuple<String> {
        Tuple(self.contexts.clone())
    }
    #[getter]
    fn diagnostics(&self) -> Tuple<DiagnosticAnnotation> {
        Tuple(self.diagnostics.clone())
    }
}

impl From<pse_columnar::Observation<'_>> for DiagnosticContext {
    fn from(item: pse_columnar::Observation<'_>) -> Self {
        Self {
            code: item.code.to_string(),
            message: item
                .domain_cause
                .map_or_else(|| item.cause.to_string(), ToString::to_string),
            contexts: item.contexts.into_iter().map(str::to_owned).collect(),
            diagnostics: item
                .diagnostics
                .into_iter()
                .cloned()
                .map(DiagnosticAnnotation)
                .collect(),
        }
    }
}

/// Native query coordinates; lines and columns use DataFusion's source convention.
#[pyclass(frozen, skip_from_py_object, get_all, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticSpan {
    start_line: u64,
    start_column: u64,
    end_line: u64,
    end_column: u64,
}
impl From<datafusion::common::Span> for DiagnosticSpan {
    fn from(span: datafusion::common::Span) -> Self {
        Self {
            start_line: span.start.line,
            start_column: span.start.column,
            end_line: span.end.line,
            end_column: span.end.column,
        }
    }
}
/// A native help or note and its optional source span.
#[pyclass(frozen, skip_from_py_object, get_all, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticNote {
    message: String,
    span: Option<DiagnosticSpan>,
}
/// One native DataFusion diagnostic, preserving grouped notes, helps and source spans.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticAnnotation(datafusion::common::Diagnostic);
#[pymethods]
impl DiagnosticAnnotation {
    #[getter]
    fn kind(&self) -> &'static str {
        match self.0.kind {
            datafusion::common::diagnostic::DiagnosticKind::Error => "error",
            datafusion::common::diagnostic::DiagnosticKind::Warning => "warning",
        }
    }
    #[getter]
    fn message(&self) -> &str {
        &self.0.message
    }
    #[getter]
    fn span(&self) -> Option<DiagnosticSpan> {
        self.0.span.map(Into::into)
    }
    #[getter]
    fn notes(&self) -> Tuple<DiagnosticNote> {
        Tuple(
            self.0
                .notes
                .iter()
                .map(|note| DiagnosticNote {
                    message: note.message.clone(),
                    span: note.span.map(Into::into),
                })
                .collect(),
        )
    }
    #[getter]
    fn helps(&self) -> Tuple<DiagnosticNote> {
        Tuple(
            self.0
                .helps
                .iter()
                .map(|note| DiagnosticNote {
                    message: note.message.clone(),
                    span: note.span.map(Into::into),
                })
                .collect(),
        )
    }
}

/// One typed observed value associated with a boundary refusal.
#[pyclass(frozen, skip_from_py_object, get_all, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticObservation {
    name: String,
    kind: String,
    real: Option<f64>,
    integer: Option<i64>,
    boolean: Option<bool>,
    text: Option<String>,
    nonfinite: Option<String>,
    quantity: Option<String>,
    unit: Option<String>,
    context: Option<String>,
    contracts: Vec<(String, Vec<(String, String, String)>)>,
}

/// Structured native diagnostics, separate from the transport's textual message.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticReport {
    boundary: Option<BoundaryDiagnostic>,
    #[pyo3(get)]
    code: Option<String>,
    #[pyo3(get)]
    message: String,
    #[pyo3(get)]
    help: Option<String>,
    causes: Vec<DiagnosticCause>,
    contexts: Vec<DiagnosticContext>,
    related: Vec<DiagnosticReport>,
}
/// Adapters retain source-owned semantic projection without dynamic error classification.
pub(crate) trait ReportSource: TypedDiagnostic {
    fn report_boundary(&self) -> BoundaryDiagnostic;
    fn report_contexts(&self) -> Vec<DiagnosticContext> {
        Vec::new()
    }
    fn report_related(&self) -> Option<Vec<DiagnosticReport>> {
        None
    }
}
macro_rules! semantic_report_source {
    ($($source:ty),* $(,)?) => {$(
        impl ReportSource for $source {
            fn report_boundary(&self) -> BoundaryDiagnostic {
                DiagnosticProjection::boundary_diagnostic(self, DiagnosticStage::Workflow)
            }
        }
    )*};
}
semantic_report_source!(
    BoundaryDiagnostic,
    pse_runtime::workflow::WorkflowError,
    pse_runtime::math::MathRuntimeError,
    pse_runtime::authoring_driver::DriverError,
    pse_backend_native::ProblemError,
);
impl ReportSource for pse_runtime::RuntimeError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
}
impl ReportSource for pse_engine::EngineError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
    fn report_contexts(&self) -> Vec<DiagnosticContext> {
        match self {
            Self::Engine(native) => native.observations().into_iter().map(Into::into).collect(),
            _ => Vec::new(),
        }
    }
    fn report_related(&self) -> Option<Vec<DiagnosticReport>> {
        match self {
            Self::Multiple { errors } => {
                Some(errors.iter().map(DiagnosticReport::observe).collect())
            }
            Self::Semantic(error) => Some(vec![DiagnosticReport::observe(error.as_ref())]),
            _ => None,
        }
    }
}
impl ReportSource for pse_schema::SchemaError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
}
impl ReportSource for dyn TypedDiagnostic + Send + Sync {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
}
impl DiagnosticReport {
    pub(crate) fn observe<E: ReportSource + ?Sized>(error: &E) -> Self {
        let boundary = error.report_boundary();
        let mut report = Self::from_boundary(boundary);
        report.message = error.to_string();
        report.help = error.help().map(|help| help.to_string());
        report.contexts = error.report_contexts();
        if let Some(related) = error.report_related() {
            report.related = related;
        }
        report.causes = Vec::new();
        let mut source = error.source();
        while let Some(cause) = source {
            report.causes.push(DiagnosticCause {
                message: cause.to_string(),
            });
            source = cause.source();
        }
        report
    }
    fn from_boundary(boundary: BoundaryDiagnostic) -> Self {
        let mut report = Self::record(&boundary);
        report.code = Some(boundary.code.as_str().to_owned());
        report.related = boundary
            .causes
            .iter()
            .cloned()
            .map(Self::from_boundary)
            .collect();
        report.boundary = Some(boundary);
        report
    }
    fn record(error: &dyn Diagnostic) -> Self {
        let mut causes = Vec::new();
        let mut source = error.source();
        while let Some(cause) = source {
            causes.push(DiagnosticCause {
                message: cause.to_string(),
            });
            source = cause.source();
        }
        Self {
            boundary: None,
            code: error.code().map(|code| code.to_string()),
            message: error.to_string(),
            help: error.help().map(|help| help.to_string()),
            causes,
            contexts: Vec::new(),
            related: error
                .related()
                .into_iter()
                .flatten()
                .map(Self::record)
                .collect(),
        }
    }
}
#[pymethods]
impl DiagnosticReport {
    /// Complete typed envelope encoded by the generated BoundaryDiagnostic document contract.
    #[getter]
    fn envelope(&self) -> PyResult<Option<Vec<u8>>> {
        self.boundary
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
    }
    #[getter]
    fn failure_class(&self) -> Option<&str> {
        self.boundary
            .as_ref()
            .map(|boundary| boundary.code.class().as_str())
    }
    #[getter]
    fn boundary_class(&self) -> Option<&str> {
        self.boundary.as_ref().map(|b| b.class.as_str())
    }
    /// Error, warning or information; a warning never makes a model invalid.
    #[getter]
    fn severity(&self) -> Option<&str> {
        self.boundary.as_ref().map(|b| b.severity.as_str())
    }
    #[getter]
    fn stage(&self) -> Option<&str> {
        self.boundary.as_ref().map(|b| b.stage.as_str())
    }
    #[getter]
    fn rule(&self) -> Option<&str> {
        self.boundary.as_ref().map(|b| b.rule.as_str())
    }
    #[getter]
    fn source_ids(&self) -> Tuple<String> {
        Tuple(
            self.boundary
                .iter()
                .flat_map(|b| &b.sources)
                .map(ToString::to_string)
                .collect(),
        )
    }
    #[getter]
    fn source_locations(&self) -> Tuple<DiagnosticSourceLocation> {
        Tuple(
            self.boundary
                .iter()
                .flat_map(|boundary| &boundary.locations)
                .cloned()
                .map(DiagnosticSourceLocation)
                .collect(),
        )
    }
    #[getter]
    fn observations(&self) -> Tuple<DiagnosticObservation> {
        use pse_model::diagnostic::Observation;
        Tuple(
            self.boundary
                .iter()
                .flat_map(|b| &b.observations)
                .map(|(name, value)| {
                    let mut observation = DiagnosticObservation {
                        name: name.clone(),
                        kind: String::new(),
                        real: None,
                        integer: None,
                        boolean: None,
                        text: None,
                        nonfinite: None,
                        quantity: None,
                        unit: None,
                        context: None,
                        contracts: Vec::new(),
                    };
                    match value {
                        Observation::Missing => observation.kind = "missing".into(),
                        Observation::Nonfinite(v) => {
                            observation.kind = "nonfinite".into();
                            observation.nonfinite = Some(match v {
                                pse_model::diagnostic::NonfiniteObservation::Nan => "nan",
                                pse_model::diagnostic::NonfiniteObservation::PositiveInfinity => "positive_infinity",
                                pse_model::diagnostic::NonfiniteObservation::NegativeInfinity => "negative_infinity",
                            }.into());
                        }
                        Observation::Physical(physical) => {
                            observation.kind = "physical".into();
                            observation.real = Some(physical.magnitude);
                            observation.quantity = Some(physical.quantity.to_hex());
                            observation.unit = Some(physical.unit.to_hex());
                            observation.context = Some(physical.context.to_hex());
                        }
                        Observation::Contracts(contracts) => {
                            observation.kind = "contracts".into();
                            observation.contracts = contracts.iter().map(|contract| (
                                pse_ids::SemanticId::from_bytes(contract.quantity).to_hex(),
                                contract.indices.iter().map(|axis| (
                                    pse_ids::SemanticId::from_bytes(axis[0]).to_hex(),
                                    pse_ids::SemanticId::from_bytes(axis[1]).to_hex(),
                                    pse_ids::SemanticId::from_bytes(axis[2]).to_hex(),
                                )).collect(),
                            )).collect();
                        }
                        Observation::Real(v) => {
                            observation.kind = "real".into();
                            observation.real = Some(*v);
                        }
                        Observation::Integer(v) => {
                            observation.kind = "integer".into();
                            observation.integer = Some(*v);
                        }
                        Observation::Boolean(v) => {
                            observation.kind = "boolean".into();
                            observation.boolean = Some(*v);
                        }
                        Observation::Text(v) => {
                            observation.kind = "text".into();
                            observation.text = Some(v.clone());
                        }
                    }
                    observation
                })
                .collect(),
        )
    }
    #[getter]
    fn causes(&self) -> Tuple<DiagnosticCause> {
        Tuple(self.causes.clone())
    }
    #[getter]
    fn contexts(&self) -> Tuple<DiagnosticContext> {
        Tuple(self.contexts.clone())
    }
    #[getter]
    fn related(&self) -> Tuple<DiagnosticReport> {
        Tuple(self.related.clone())
    }
}

pub(crate) fn diagnostic<E: ReportSource + ?Sized>(py: Python<'_>, error: &E) -> PyErr {
    let result = InspectionError::new_err(message(error));
    let report = DiagnosticReport::observe(error);
    let mut cause = None;
    for item in report.causes.iter().rev() {
        let next = PyException::new_err(item.message.clone());
        next.set_cause(py, cause);
        cause = Some(next);
    }
    result.set_cause(py, cause);
    if let Err(error) = (Details { report }).attach(result.value(py).as_any()) {
        return error;
    }
    result
}

pub(super) fn message<E: Diagnostic + ?Sized>(error: &E) -> String {
    let mut message = error
        .code()
        .map_or_else(|| error.to_string(), |code| format!("[{code}] {error}"));
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

pub(crate) fn invalid(reason: &str) -> pse_engine::EngineError {
    pse_engine::EngineError::ConfigInvalid {
        key: "pse.inspection".to_owned(),
        reason: reason.to_owned(),
    }
}

pub(super) fn closed() -> pse_engine::EngineError {
    pse_engine::EngineError::Admission {
        path: "inspection handle".to_owned(),
        reason: "handle is closed".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::diagnostic::{
        BoundaryClass, DiagnosticCode, DiagnosticRule, Observation, PhysicalObservation,
        SourceLocation,
    };

    #[test]
    fn schema_report_preserves_registry_diagnostic_code_and_envelope() {
        let error = pse_schema::SchemaError::UnknownReference {
            context: "result relation".into(),
            reference: "runtime.unknown".into(),
        };
        let report = DiagnosticReport::observe(&error);
        assert_eq!(
            report.code.as_deref(),
            Some(DiagnosticCode::SchemaUnknownReference.as_str())
        );
        let decoded: BoundaryDiagnostic =
            serde_json::from_slice(&report.envelope().unwrap().unwrap()).unwrap();
        assert_eq!(decoded.code, DiagnosticCode::SchemaUnknownReference);
        assert_eq!(decoded.stage, DiagnosticStage::Workflow);
    }

    #[test]
    fn native_report_preserves_authoritative_code_and_complete_envelope() {
        let source = pse_ids::SemanticId::from_bytes([4; 16]);
        let revision = pse_ids::ContentHash::from_bytes([5; 32]);
        let mut boundary = BoundaryDiagnostic::new(
            BoundaryClass::Incompatible,
            DiagnosticStage::Test,
            [source],
            DiagnosticRule::NativeReuse,
        );
        boundary.locations.push(SourceLocation {
            source,
            revision: Some(revision),
            path: "source.pse".into(),
            name: None,
            start: None,
            end: None,
        });
        boundary.observations.insert(
            "physical".into(),
            Observation::Physical(PhysicalObservation {
                magnitude: 2.,
                quantity: source,
                unit: source,
                context: revision,
            }),
        );
        boundary
            .observations
            .insert("unavailable".into(), Observation::Missing);
        boundary.causes.push(BoundaryDiagnostic::new(
            BoundaryClass::Internal,
            DiagnosticStage::Evaluation,
            [source],
            DiagnosticRule::WorkflowPanic,
        ));
        let report = DiagnosticReport::observe(&boundary);
        assert_eq!(
            report.code.as_deref(),
            Some(DiagnosticCode::NativeReuse.as_str())
        );
        assert_eq!(
            report.failure_class(),
            Some(DiagnosticCode::NativeReuse.class().as_str())
        );
        assert_eq!(report.related.len(), 1);
        let decoded: BoundaryDiagnostic =
            serde_json::from_slice(&report.envelope().unwrap().unwrap()).unwrap();
        assert_eq!(decoded.locations[0].revision, Some(revision));
        assert_eq!(decoded.causes[0].code, DiagnosticCode::WorkflowPanic);
        assert!(
            matches!(&decoded.observations["physical"], Observation::Physical(physical) if physical.context == revision)
        );
        assert!(matches!(
            decoded.observations["unavailable"],
            Observation::Missing
        ));
    }
}
