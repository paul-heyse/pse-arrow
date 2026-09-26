// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::tuple::Tuple;
use miette::Diagnostic;
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
}

/// Structured native diagnostics, separate from the transport's textual message.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticReport {
    boundary: Option<pse_model::diagnostic::BoundaryDiagnostic>,
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
impl DiagnosticReport {
    pub(crate) fn observe(error: &(dyn Diagnostic + 'static)) -> Self {
        let source: &(dyn std::error::Error + 'static) = error;
        let contexts = match source.downcast_ref::<pse_engine::EngineError>() {
            Some(pse_engine::EngineError::Engine(native)) => {
                native.observations().into_iter().map(Into::into).collect()
            }
            _ => Vec::new(),
        };
        let mut report = Self::record(error);
        report.contexts = contexts;
        let mut current = Some(source);
        while let Some(cause) = current {
            if let Some(boundary) =
                cause.downcast_ref::<pse_model::diagnostic::BoundaryDiagnostic>()
            {
                report.boundary = Some(boundary.clone());
                break;
            }
            if let Some(workflow) = cause.downcast_ref::<pse_runtime::workflow::WorkflowError>() {
                report.boundary = Some(workflow.boundary_diagnostic());
                break;
            }
            current = cause.source();
        }
        match source.downcast_ref::<pse_engine::EngineError>() {
            Some(pse_engine::EngineError::Multiple { errors }) => {
                report.related = errors.iter().map(|error| Self::observe(error)).collect();
            }
            Some(pse_engine::EngineError::Semantic(error)) => {
                report.related = vec![Self::observe(error.as_ref())];
            }
            _ => {}
        }
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
    #[getter]
    fn boundary_class(&self) -> Option<&str> {
        self.boundary.as_ref().map(|b| b.class.as_str())
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
                    };
                    match value {
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

pub(crate) fn diagnostic(py: Python<'_>, error: &(dyn Diagnostic + 'static)) -> PyErr {
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

pub(super) fn message(error: &dyn Diagnostic) -> String {
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
