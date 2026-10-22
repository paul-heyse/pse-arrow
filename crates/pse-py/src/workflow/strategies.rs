// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical owned analysis requests. Native preparation owns all eligibility and iteration.
use super::*;
use crate::enums::EnumValue;
use pse_backend_native::solve::SolveReport;
use pse_model::generated::enums::{NativeBackend, NativeQualification, NativeTermination};
use pse_runtime::math::solves::{Outcome, StepReport};

/// Physically admitted selected-revision graph with authored tear policies.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePreparedFlow {
    pub(super) owner: Arc<runtime::Runtime>,
    pub(super) math: Arc<pse_runtime::math::MathService>,
    pub(super) package: native::ModelingPackage,
    pub(super) inner: pse_runtime::math::flows::PreparedFlow,
}
#[pymethods]
impl NativePreparedFlow {
    fn graph(&self) -> documents::DocumentValue<pse_runtime::math::flows::FlowGraphDocument> {
        documents::DocumentValue(self.inner.document())
    }
    fn select_tears(
        &self,
        py: Python<'_>,
        method: EnumValue<pse_model::generated::enums::TearMethod>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
    ) -> PyResult<NativeStrategyResult> {
        let settings = settings.0;
        let method = method.0;
        let handle = py
            .detach(|| {
                let _enter = self.owner.executor.enter();
                self.math
                    .select_tears_request(self.inner.clone(), method, settings)
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        let cancel = handle.cancellation();
        let result = blocking(
            py,
            &self.owner,
            async { handle.finish().await.map_err(native::WorkflowError::from) },
            || cancel.cancel(),
        )?;
        let primal_coordinates = result.attempt.as_ref().map(|_| self.package.auxiliary_primal_coordinates(&result.dimensionless_coordinates))
            .transpose().map_err(|error| errors::diagnostic(py, &error))?;
        NativeStrategyResult::new(py, StrategyResult::Tears(Box::new(result)), &self.owner, primal_coordinates)
    }
}

#[derive(Clone, Debug)]
pub(super) enum Strategy {
    Cone(Box<native::PreparedConic>),
    #[cfg(feature = "native-solvers")]
    Recycle(Box<native::PreparedRecycle>),
    #[cfg(feature = "native-solvers")]
    Initialization(Box<native::PreparedInitializationStrategy>),
}
/// Prepared explicit cone, causal map or conditional initialization.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePreparedStrategy {
    pub(super) owner: Arc<runtime::Runtime>,
    pub(super) inner: Strategy,
}
#[pymethods]
impl NativePreparedStrategy {
    /// Typed routes selected before execution, without implicit failure fallback: the
    /// cone route, the declared map's KINSOL fixed-point route, or one per initialization
    /// block.
    #[getter]
    fn routes(&self) -> PyResult<Vec<NativeRoute>> {
        match &self.inner {
            Strategy::Cone(p) => Ok(vec![p.solve().route().into()]),
            #[cfg(feature = "native-solvers")]
            Strategy::Recycle(prepared) => prepared.route()
                .map(|route| vec![route.into()])
                .map_err(|error| Python::attach(|py| errors::diagnostic(py, &error))),
            #[cfg(feature = "native-solvers")]
            Strategy::Initialization(p) => p
                .strategies()
                .map(|routes| routes.into_iter().map(Into::into).collect())
                .map_err(|e| Python::attach(|py| errors::diagnostic(py, &e))),
        }
    }
    /// Run one bounded attempt; descriptive copying is selected explicitly.
    #[pyo3(signature=(*,review_context=None))]
    fn run(
        &self,
        py: Python<'_>,
        review_context: Option<documents::DocumentInput<native::ReviewContextRequest>>,
    ) -> PyResult<NativeStrategyResult> {
        macro_rules! run {
            ($start:expr,$variant:ident) => {{
                let (handle, operation) = py
                    .detach(|| {
                        let _enter = self.owner.executor.enter();
                        $start
                    })
                    .map_err(|e| errors::diagnostic(py, &e))?;
                let cancellation = handle.cancellation();
                let report = blocking(
                    py,
                    &self.owner,
                    async {
                        handle
                            .finish()
                            .await
                            .map(|report| StrategyResult::$variant(Box::new(report)))
                            .map_err(native::WorkflowError::from)
                    },
                    || cancellation.cancel(),
                )?;
                (report, operation)
            }};
        }
        let (result, _operation): (
            StrategyResult,
            Option<pse_diagnostics::observation::OperationId>,
        ) = match &self.inner {
            Strategy::Cone(p) => run!(p.start().map(|handle| (handle, None)), Cone),
            #[cfg(feature = "native-solvers")]
            Strategy::Recycle(p) => run!(p.start_with_operation(), Recycle),
            #[cfg(feature = "native-solvers")]
            Strategy::Initialization(p) => run!(p.start_with_operation(), Initialization),
        };
        // Mandatory science/event transport owns its allocation before optional
        // descriptive capture can reserve any of the remaining pool.
        let primal_coordinates = match &self.inner {
            Strategy::Cone(prepared) => prepared.primal_coordinates(),
            #[cfg(feature = "native-solvers")]
            Strategy::Recycle(prepared) => prepared.primal_coordinates(),
            #[cfg(feature = "native-solvers")]
            Strategy::Initialization(prepared) => prepared.primal_coordinates(),
        }.map_err(|error| errors::diagnostic(py, &error))?;
        let mut result = NativeStrategyResult::new(py, result, &self.owner, Some(primal_coordinates))?;
        #[cfg(feature = "native-solvers")]
        let selected_context = review_context
            .map(|request| {
                match (&self.inner, result.inner.as_ref()) {
                    #[cfg(feature = "native-solvers")]
                    (
                        Strategy::Initialization(prepared),
                        StrategyResult::Initialization(report),
                    ) => prepared.review_context(&request.0, _operation, report),
                    #[cfg(feature = "native-solvers")]
                    (Strategy::Recycle(prepared), StrategyResult::Recycle(report)) => {
                        prepared.review_context(&request.0, _operation, report)
                    }
                    _ => {
                        return Err(invalid(
                            py,
                            "strategy context has no corresponding original owner",
                        ));
                    }
                }
                .map(Arc::new)
                .map_err(|error| errors::diagnostic(py, &error))
            })
            .transpose();
        #[cfg(not(feature = "native-solvers"))]
        let selected_context: PyResult<Option<Arc<native::ReviewContext>>> =
            if review_context.is_some() {
                Err(invalid(
                    py,
                    "strategy descriptive context is not supplied by this adapter",
                ))
            } else {
                Ok(None)
            };
        match selected_context {
            Ok(context) => result.review_context = context,
            Err(error) => result.review_context_failure = Some(error.to_string()),
        }
        Ok(result)
    }
}
#[derive(Debug)]
enum StrategyResult {
    Tears(Box<pse_runtime::math::flows::TearResult>),
    Cone(Box<StepReport>),
    #[cfg(feature = "native-solvers")]
    Recycle(Box<pse_runtime::math::initialization::DeclaredRootReport>),
    #[cfg(feature = "native-solvers")]
    Initialization(Box<pse_runtime::math::initialization::InitializationReport>),
}
/// Joined native strategy report, including failed stages and immutable original values.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeStrategyResult {
    inner: Arc<StrategyResult>,
    events: Arc<native::StrategyEventExport>,
    primal_coordinates: Option<native::PrimalCoordinates>,
    review_context: Option<Arc<native::ReviewContext>>,
    review_context_failure: Option<String>,
}
impl NativeStrategyResult {
    fn new(py: Python<'_>, inner: StrategyResult, owner: &Arc<runtime::Runtime>, primal_coordinates: Option<native::PrimalCoordinates>) -> PyResult<Self> {
        type Trace = (
            pse_model::generated::identities::RunId,
            usize,
            pse_runtime::math::solves::StrategyTrace,
        );
        let (upper, traces): (usize, Box<dyn Iterator<Item = Trace> + '_>) =
            match &inner {
                StrategyResult::Tears(_) => (0, Box::new(std::iter::empty())),
                StrategyResult::Cone(report) => (
                    1,
                    Box::new(std::iter::once((report.run_id, 0, report.strategy.clone()))),
                ),
                #[cfg(feature = "native-solvers")]
                StrategyResult::Recycle(report) => (
                    1,
                    Box::new(std::iter::once((report.run_id, 0, report.strategy.clone()))),
                ),
                #[cfg(feature = "native-solvers")]
                StrategyResult::Initialization(report) => {
                    (
                        report.attempts.len(),
                        Box::new(report.attempts.iter().enumerate().filter_map(
                            |(step, attempt)| {
                                attempt
                                    .trace
                                    .as_ref()
                                    .map(|trace| (report.run_id, step, trace.clone()))
                            },
                        )),
                    )
                }
            };
        let events = native::StrategyEventExport::new(
            owner.model.clone(),
            owner.shared.clone(),
            upper,
            traces,
        )
        .map_err(|error| errors::diagnostic(py, &error))?;
        Ok(Self {
            inner: Arc::new(inner),
            events: Arc::new(events),
            primal_coordinates,
            review_context: None,
            review_context_failure: None,
        })
    }
}
#[pymethods]
impl NativeStrategyResult {
    /// Return only explicitly selected descriptive facts retained at the original join.
    fn review_context(&self, py: Python<'_>) -> PyResult<documents::DocumentValue<Arc<native::ReviewContext>>> {
        if let Some(error) = &self.review_context_failure {
            return Err(invalid(
                py,
                format!("selected strategy review context unavailable: {error}"),
            ));
        }
        let context = self.review_context.as_ref().ok_or_else(|| {
            invalid(
                py,
                "strategy review context was not selected before execution",
            )
        })?;
        Ok(documents::DocumentValue(context.clone()))
    }
    /// Identity of the actual submitted numerical operation; structural tears have none.
    #[getter]
    fn run_id(&self) -> Option<crate::identities::Identity<pse_model::generated::identities::RunId>> {
        match self.inner.as_ref() {
            StrategyResult::Tears(_) => None,
            StrategyResult::Cone(report) => Some(crate::identities::Identity(report.run_id)),
            #[cfg(feature = "native-solvers")]
            StrategyResult::Recycle(report) => Some(crate::identities::Identity(report.run_id)),
            #[cfg(feature = "native-solvers")]
            StrategyResult::Initialization(report) => Some(crate::identities::Identity(report.run_id)),
        }
    }
    /// Generated observations from retained traces, with their original operation identity.
    fn strategy_events(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.events.clone().into_cursor(1024))
            .map(inspection::TableStream::from_result)
            .map_err(|error| errors::diagnostic(py, &error))
    }
    /// Every attempt in execution order, one typed row each: its native report, or the
    /// typed failure that preceded one, so indices are shared by reports and failures.
    fn attempts(&self) -> Vec<NativeStrategyAttempt> {
        let report = |r: &SolveReport| NativeStrategyAttempt {
            report: Some(NativeAttempt { inner: r.clone(), primal_coordinates: self.primal_coordinates.clone() }),
            ..Default::default()
        };
        match self.inner.as_ref() {
            StrategyResult::Tears(r) => r.attempt.iter().map(report).collect(),
            StrategyResult::Cone(r) => match &r.outcome {
                Outcome::Native(r) => vec![report(r)],
                Outcome::Rejected(e) => vec![NativeStrategyAttempt {
                    failure: Some(inspection::DiagnosticReport::observe(e.as_ref())),
                    ..Default::default()
                }],
                Outcome::Constant(_) => vec![],
            },
            #[cfg(feature = "native-solvers")]
            StrategyResult::Recycle(r) => vec![report(&r.report)],
            #[cfg(feature = "native-solvers")]
            StrategyResult::Initialization(r) => r
                .attempts
                .iter()
                .map(|a| NativeStrategyAttempt {
                    report: a.result.as_ref().ok().map(|r| NativeAttempt {
                        inner: r.as_ref().clone(),
                        primal_coordinates: self.primal_coordinates.clone(),
                    }),
                    failure: a
                        .result
                        .as_ref()
                        .err()
                        .map(|e| inspection::DiagnosticReport::observe(e.as_ref())),
                    route: Some(a.strategy.into()),
                    stage: Some(a.stage),
                    committed: Some(a.committed),
                })
                .collect(),
        }
    }
    /// Only initialization has temporary overlays; their candidates never replace original bindings.
    fn initialization(&self) -> Option<documents::DocumentValue<native::InitializationDocument>> {
        #[cfg(feature = "native-solvers")]
        if let StrategyResult::Initialization(report) = self.inner.as_ref() {
            return Some(documents::DocumentValue(native::InitializationDocument::from(report.as_ref())));
        }
        None
    }
    fn tears(&self) -> Option<documents::DocumentValue<pse_runtime::math::flows::TearSelectionDocument>> {
        if let StrategyResult::Tears(report) = self.inner.as_ref() {
            return report
                .document()
                .map(documents::DocumentValue);
        }
        None
    }
}
/// One strategy attempt: exactly one of its native report and the typed failure that
/// preceded a report; initialization block attempts also carry their stage, route and
/// commit.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug, Default)]
pub(crate) struct NativeStrategyAttempt {
    report: Option<NativeAttempt>,
    failure: Option<inspection::DiagnosticReport>,
    route: Option<NativeRoute>,
    stage: Option<usize>,
    committed: Option<bool>,
}
#[pymethods]
impl NativeStrategyAttempt {
    /// Faithful native report, when one exists.
    #[getter]
    fn report(&self) -> Option<NativeAttempt> {
        self.report.clone()
    }
    /// Typed failure before a native report existed.
    #[getter]
    fn failure(&self) -> Option<inspection::DiagnosticReport> {
        self.failure.clone()
    }
    /// Selected block route of an initialization attempt.
    #[getter]
    fn route(&self) -> Option<NativeRoute> {
        self.route.clone()
    }
    /// Zero-based continuation stage of an initialization attempt.
    #[getter]
    fn stage(&self) -> Option<usize> {
        self.stage
    }
    /// Whether an initialization block committed its coordinates.
    #[getter]
    fn committed(&self) -> Option<bool> {
        self.committed
    }
}
/// Owned native attempt, retaining stop, qualification, original values and bounded history separately.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeAttempt {
    pub(super) inner: SolveReport,
    pub(super) primal_coordinates: Option<native::PrimalCoordinates>,
}
#[pymethods]
impl NativeAttempt {
    #[getter]
    fn backend(&self) -> EnumValue<NativeBackend> {
        self.inner.backend.into()
    }
    #[getter]
    fn termination(&self) -> EnumValue<NativeTermination> {
        self.inner.termination.category.into()
    }
    #[getter]
    fn native_code(&self) -> i64 {
        self.inner.termination.code
    }
    #[getter]
    fn native_status(&self) -> &str {
        &self.inner.termination.name
    }
    #[getter]
    fn qualification(&self) -> EnumValue<NativeQualification> {
        self.inner.qualification.into()
    }
    /// Typed independent-validation failure, when validation failed.
    #[getter]
    fn validation_error(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .validation_failure()
            .map(inspection::DiagnosticReport::observe)
    }
    /// Effective explicit native options and semantic controls, as the adapter recorded them.
    #[pyo3(signature = () -> "dict[str, bool | int | float | str]")]
    fn options<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        use pse_backend_native::solve::OptionValue;
        let dict = pyo3::types::PyDict::new(py);
        for (key, value) in &self.inner.options {
            match value {
                OptionValue::Text(v) => dict.set_item(key, v)?,
                OptionValue::Real(v) => dict.set_item(key, v)?,
                OptionValue::Integer(v) => dict.set_item(key, v)?,
                OptionValue::Bool(v) => dict.set_item(key, v)?,
            }
        }
        Ok(dict)
    }
    #[getter]
    fn normalized_violation(&self) -> Option<f64> {
        self.inner.quality.as_ref().map(|q| q.normalized_max)
    }
    #[getter]
    fn objective(&self) -> Option<f64> {
        self.inner.candidate.as_ref().and_then(|c| c.objective)
    }
    fn primal(&self, py: Python<'_>) -> PyResult<pyo3_arrow::PyRecordBatch> {
        let coordinates = self.primal_coordinates.as_ref().ok_or_else(|| invalid(py, "native primal physical coordinate meaning is unavailable"))?;
        coordinates.table(&self.inner).map(|table| pyo3_arrow::PyRecordBatch::new(table.batch().clone()))
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn progress(&self) -> (Vec<ProgressEvent>, u64) {
        (
            self.inner
                .events
                .iter()
                .cloned()
                .map(|event| documents::DocumentValue(event.into()))
                .collect(),
            self.inner.dropped_events,
        )
    }
    fn metrics(&self) -> ProgressEvent {
        documents::DocumentValue(native::ProgressEventDocument::from(
            pse_backend_native::solve::Event {
                source: None,
                phase: "final".into(),
                elapsed: Duration::ZERO,
                values: self.inner.metrics.clone(),
                incumbent: None,
            },
        ))
    }
    fn provenance(&self) -> std::collections::BTreeMap<String, String> {
        self.inner.provenance.clone()
    }
    fn available_start(&self) -> Option<NativeStart> {
        self.inner
            .warm_start
            .clone()
            .map(|inner| NativeStart { inner })
    }
    /// Actual input seed receipt, distinct from available output starts.
    fn start_receipt(&self) -> Option<documents::DocumentValue<pse_backend_native::solve::WarmStartSnapshot>> {
        self.inner.start_receipt.as_ref().map(|start| documents::DocumentValue(start.snapshot()))
    }
}
