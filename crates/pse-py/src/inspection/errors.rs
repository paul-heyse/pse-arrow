// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::tuple::Tuple;
use crate::documents::DocumentValue;
use crate::enums::EnumValue;
use miette::Diagnostic;
use pse_diagnostics::TypedDiagnostic;
use pse_model::diagnostic::{
    BoundaryDiagnostic, DiagnosticProjection, DiagnosticStage, SourceLocation,
};
use pse_model::generated::enums::{
    DiagnosticCode, DiagnosticRule, DiagnosticSeverity, FailureClass, NativeBoundaryClass,
};
use pse_runtime::workflow::{DiagnosticCauseDocument, DiagnosticContextDocument};
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

/// Source-owned observation values projected through the generated document codec.
pub(crate) struct ObservedValues(Option<BoundaryDiagnostic>);
impl<'py> IntoPyObject<'py> for ObservedValues {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;
    const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr = pyo3::type_hint_subscript!(
        pyo3::type_hint_identifier!("builtins", "dict"),
        pyo3::type_hint_identifier!("builtins", "str"),
        pyo3::type_hint_identifier!("pse.contracts.documents", "Observation")
    );
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        match self.0 {
            Some(boundary) => DocumentValue(boundary)
                .into_pyobject(py)?
                .getattr("observations"),
            None => Ok(pyo3::types::PyDict::new(py).into_any()),
        }
    }
}

/// Structured native diagnostics, separate from the transport's textual message.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticReport {
    boundary: Option<BoundaryDiagnostic>,
    code: Option<String>,
    #[pyo3(get)]
    message: String,
    #[pyo3(get)]
    help: Option<String>,
    causes: Vec<DiagnosticCauseDocument>,
    contexts: Vec<DiagnosticContextDocument>,
    related: Vec<DiagnosticReport>,
}
/// Adapters retain source-owned semantic projection without dynamic error classification.
pub(crate) trait ReportSource: TypedDiagnostic {
    fn report_boundary(&self) -> BoundaryDiagnostic;
    fn report_contexts(&self) -> Vec<DiagnosticContextDocument> {
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
    fn report_contexts(&self) -> Vec<DiagnosticContextDocument> {
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
            report.causes.push(DiagnosticCauseDocument {
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
            causes.push(DiagnosticCauseDocument {
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
    #[getter]
    fn code(&self) -> Option<EnumValue<DiagnosticCode>> {
        self.boundary.as_ref().map(|boundary| boundary.code.into())
    }
    #[getter]
    fn reported_code(&self) -> Option<&str> {
        self.code.as_deref()
    }
    /// Complete typed envelope encoded by the generated BoundaryDiagnostic document contract.
    #[getter]
    fn envelope(&self) -> Option<DocumentValue<BoundaryDiagnostic>> {
        self.boundary.clone().map(DocumentValue)
    }
    #[getter]
    fn failure_class(&self) -> Option<EnumValue<FailureClass>> {
        self.boundary
            .as_ref()
            .map(|boundary| boundary.code.class().into())
    }
    #[getter]
    fn boundary_class(&self) -> Option<EnumValue<NativeBoundaryClass>> {
        self.boundary.as_ref().map(|b| b.class.into())
    }
    /// Error, warning or information; a warning never makes a model invalid.
    #[getter]
    fn severity(&self) -> Option<EnumValue<DiagnosticSeverity>> {
        self.boundary.as_ref().map(|b| b.severity.into())
    }
    #[getter]
    fn stage(&self) -> Option<EnumValue<DiagnosticStage>> {
        self.boundary.as_ref().map(|b| b.stage.into())
    }
    #[getter]
    fn rule(&self) -> Option<EnumValue<DiagnosticRule>> {
        self.boundary.as_ref().map(|b| b.rule.into())
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
    fn source_locations(&self) -> Tuple<DocumentValue<SourceLocation>> {
        Tuple(
            self.boundary
                .iter()
                .flat_map(|boundary| &boundary.locations)
                .cloned()
                .map(DocumentValue)
                .collect(),
        )
    }
    #[getter]
    fn observations(&self) -> ObservedValues {
        ObservedValues(self.boundary.clone())
    }
    #[getter]
    fn causes(&self) -> Tuple<DocumentValue<DiagnosticCauseDocument>> {
        Tuple(self.causes.iter().cloned().map(DocumentValue).collect())
    }
    #[getter]
    fn contexts(&self) -> Tuple<DocumentValue<DiagnosticContextDocument>> {
        Tuple(self.contexts.iter().cloned().map(DocumentValue).collect())
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
        let decoded: BoundaryDiagnostic = report.envelope().unwrap().0;
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
            report.failure_class().map(|value| value.0),
            Some(DiagnosticCode::NativeReuse.class())
        );
        assert_eq!(report.related.len(), 1);
        let decoded: BoundaryDiagnostic = report.envelope().unwrap().0;
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
