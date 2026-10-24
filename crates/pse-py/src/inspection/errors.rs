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
    /// Original analysis occurrence retained when publication delivery is uncertain.
    #[pyo3(get)]
    analysis_key: Option<String>,
    /// Original scientific artifact occurrence after uncertain delivery.
    #[pyo3(get)]
    artifact_key: Option<String>,
    artifact_intent: Option<pse_operations::canonical_artifacts::Artifact>,
    #[pyo3(get)]
    artifact_owner_kind: Option<String>,
    #[pyo3(get)]
    artifact_owner: Option<String>,
    #[pyo3(get)]
    artifact_read_key: Option<String>,
    /// Original unsettled study occurrence requiring explicit recovery.
    #[pyo3(get)]
    study_key: Option<String>,
    /// Original source problem, retained for explicit read-only settlement.
    #[pyo3(get)]
    source_problem: Option<String>,
    /// Original source operation; absence of its receipt never proves non-issuance.
    #[pyo3(get)]
    source_operation: Option<String>,
    /// True only when the actual source transaction owner proved refusal. False
    /// means refusal is unproved; it does not assert acknowledgement or success.
    #[pyo3(get)]
    source_refusal_proved: bool,
    causes: Vec<DiagnosticCauseDocument>,
    contexts: Vec<DiagnosticContextDocument>,
    related: Vec<DiagnosticReport>,
}
/// Adapters retain source-owned semantic projection without dynamic error classification.
pub(crate) trait ReportSource: TypedDiagnostic {
    fn report_boundary(&self) -> BoundaryDiagnostic;
    fn report_analysis_key(&self) -> Option<String> {
        None
    }
    fn report_artifact_key(&self) -> Option<String> {
        None
    }
    fn report_artifact_intent(&self) -> Option<pse_operations::canonical_artifacts::Artifact> {
        None
    }
    fn report_artifact_cleanup(&self) -> Option<(String, String, Option<String>)> {
        None
    }
    fn report_study_key(&self) -> Option<String> {
        None
    }
    fn report_source_problem(&self) -> Option<String> {
        None
    }
    fn report_source_operation(&self) -> Option<String> {
        None
    }
    fn report_source_refusal_proved(&self) -> bool {
        false
    }
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
    pse_runtime::math::MathRuntimeError,
    pse_admission::authoring_driver::DriverError,
    pse_backend_native::ProblemError,
);
impl ReportSource for pse_runtime::workflow::WorkflowError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        DiagnosticProjection::boundary_diagnostic(self, DiagnosticStage::Workflow)
    }
    fn report_analysis_key(&self) -> Option<String> {
        match self {
            Self::Canonical(error) => error.report_analysis_key(),
            Self::Shared(error) => error.report_analysis_key(),
            Self::ArtifactTransferUnsettled {
                analysis, source, ..
            } => analysis.clone().or_else(|| source.report_analysis_key()),
            Self::StudyCreationUnsettled { source, .. } => source.report_analysis_key(),
            _ => None,
        }
    }
    fn report_artifact_key(&self) -> Option<String> {
        match self {
            Self::Canonical(error) => error.report_artifact_key(),
            Self::Shared(error) => error.report_artifact_key(),
            Self::StudyCreationUnsettled { source, .. } => source.report_artifact_key(),
            Self::ArtifactTransferUnsettled { artifact, .. } => Some(artifact.key.clone()),
            _ => None,
        }
    }
    fn report_artifact_intent(&self) -> Option<pse_operations::canonical_artifacts::Artifact> {
        match self {
            Self::Canonical(error) => error.report_artifact_intent(),
            Self::Shared(error) => error.report_artifact_intent(),
            Self::StudyCreationUnsettled { source, .. } => source.report_artifact_intent(),
            Self::ArtifactTransferUnsettled { artifact, .. } => Some((**artifact).clone()),
            _ => None,
        }
    }
    fn report_artifact_cleanup(&self) -> Option<(String, String, Option<String>)> {
        match self {
            Self::ArtifactTransferUnsettled {
                owner_kind,
                owner,
                read,
                ..
            } => Some((owner_kind.clone(), owner.clone(), read.clone())),
            Self::Shared(error) => error.report_artifact_cleanup(),
            Self::StudyCreationUnsettled { source, .. } => source.report_artifact_cleanup(),
            _ => None,
        }
    }
    fn report_study_key(&self) -> Option<String> {
        match self {
            Self::StudyCreationUnsettled { study, .. } => Some(study.to_string()),
            Self::Shared(error) => error.report_study_key(),
            _ => None,
        }
    }
    fn report_source_problem(&self) -> Option<String> {
        match self {
            Self::Canonical(error) => error.report_source_problem(),
            Self::Shared(error) => error.report_source_problem(),
            Self::StudyCreationUnsettled { source, .. }
            | Self::ArtifactTransferUnsettled { source, .. } => source.report_source_problem(),
            _ => None,
        }
    }
    fn report_source_operation(&self) -> Option<String> {
        match self {
            Self::Canonical(error) => error.report_source_operation(),
            Self::Shared(error) => error.report_source_operation(),
            Self::StudyCreationUnsettled { source, .. }
            | Self::ArtifactTransferUnsettled { source, .. } => source.report_source_operation(),
            _ => None,
        }
    }

    fn report_source_refusal_proved(&self) -> bool {
        match self {
            Self::Canonical(error) => error.report_source_refusal_proved(),
            Self::Shared(error) => error.report_source_refusal_proved(),
            Self::StudyCreationUnsettled { source, .. }
            | Self::ArtifactTransferUnsettled { source, .. } => {
                source.report_source_refusal_proved()
            }
            _ => false,
        }
    }
}
impl ReportSource for pse_runtime::RuntimeError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
}
impl ReportSource for pse_columnar::CanonError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
}
impl ReportSource for pse_operations::canonical::CanonicalError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
    fn report_analysis_key(&self) -> Option<String> {
        match self {
            Self::AnalysisUnsettled { intent, .. } => Some(intent.key.clone()),
            _ => None,
        }
    }
    fn report_artifact_key(&self) -> Option<String> {
        match self {
            Self::ArtifactUnsettled { intent, .. } => Some(intent.key.clone()),
            _ => None,
        }
    }
    fn report_artifact_intent(&self) -> Option<pse_operations::canonical_artifacts::Artifact> {
        match self {
            Self::ArtifactUnsettled { intent, .. } => Some((**intent).clone()),
            _ => None,
        }
    }
    fn report_source_problem(&self) -> Option<String> {
        match self {
            Self::SourceEditUnsettled { problem, .. } | Self::SourceEditRefused { problem, .. } => {
                Some(problem.clone())
            }
            _ => None,
        }
    }
    fn report_source_operation(&self) -> Option<String> {
        match self {
            Self::SourceEditUnsettled { operation, .. }
            | Self::SourceEditRefused { operation, .. } => Some(operation.clone()),
            _ => None,
        }
    }

    fn report_source_refusal_proved(&self) -> bool {
        self.proves_source_refusal()
    }
}
impl ReportSource for pse_runtime::math::portable::PortableError {
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
impl ReportSource for pse_runtime::observation::ObservationError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
}
impl ReportSource for pse_runtime::tooling::review_pack::PackError {
    fn report_boundary(&self) -> BoundaryDiagnostic {
        pse_model::diagnostic::project_typed(self, DiagnosticStage::Workflow)
    }
}
impl ReportSource for pse_runtime::observation_detail::CaptureError {
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
        report.analysis_key = error.report_analysis_key();
        report.artifact_key = error.report_artifact_key();
        report.artifact_intent = error.report_artifact_intent();
        if let Some((kind, owner, read)) = error.report_artifact_cleanup() {
            report.artifact_owner_kind = Some(kind);
            report.artifact_owner = Some(owner);
            report.artifact_read_key = read;
        }
        report.study_key = error.report_study_key();
        report.source_problem = error.report_source_problem();
        report.source_operation = error.report_source_operation();
        report.source_refusal_proved = error.report_source_refusal_proved();
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
            analysis_key: None,
            artifact_key: None,
            artifact_intent: None,
            artifact_owner_kind: None,
            artifact_owner: None,
            artifact_read_key: None,
            study_key: None,
            source_problem: None,
            source_operation: None,
            source_refusal_proved: false,
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
    /// Exact original settlement document; inactive content remains inaccessible.
    #[getter]
    fn artifact_intent(
        &self,
    ) -> Option<DocumentValue<&pse_operations::canonical_artifacts::Artifact>> {
        self.artifact_intent.as_ref().map(DocumentValue)
    }
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

