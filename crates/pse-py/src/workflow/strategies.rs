// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical owned analysis requests. Native preparation owns all eligibility and iteration.
use super::*;
use pse_backend_native::solve::SolveReport;
use pse_runtime::math::solves::{Outcome, StepReport};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AnalysisDocument<T> {
    pub(super) payload: T,
}
fn document(payload: serde_json::Value) -> String {
    serde_json::json!({"payload":payload}).to_string()
}

/// Physically admitted selected-revision graph with authored tear policies.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePreparedFlow {
    pub(super) owner: Arc<runtime::Runtime>,
    pub(super) math: Arc<pse_runtime::math::MathService>,
    pub(super) inner: pse_runtime::math::flows::PreparedFlow,
}
#[pymethods]
impl NativePreparedFlow {
    fn graph_json(&self) -> String {
        let d = self.inner.graph().declaration();
        document(serde_json::json!({
            "identity": self.inner.graph().key(),
            "nodes": d.nodes.iter().map(|n| serde_json::json!({"id":n.id,"ports":n.ports.iter().map(|p|serde_json::json!({"id":p.id,"quantity_id":p.quantity.as_id(),"unit_id":p.unit.as_id()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "connections": d.connections.iter().map(|c|serde_json::json!({"id":c.id,"from":c.from,"to":c.to,"decision":c.decision,"bindings":c.bindings})).collect::<Vec<_>>(),
            "decisions":d.decisions.iter().map(|d|serde_json::json!({"id":d.id,"cost":d.cost,"policy":d.policy})).collect::<Vec<_>>()
        }))
    }
    fn select_tears(
        &self,
        py: Python<'_>,
        method: &str,
        settings: &[u8],
    ) -> PyResult<NativeStrategyResult> {
        let settings = settings::solve_profile(py, settings)?;
        let method: pse_runtime::math::flows::TearMethod =
            settings::named(py, "tear method", method)?;
        let handle = py
            .detach(|| {
                let _enter = self.owner.executor.enter();
                self.math
                    .select_tears(self.inner.clone(), method, settings.controls.clone())
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        let cancel = handle.cancellation();
        let result = blocking(
            py,
            &self.owner,
            async { handle.finish().await.map_err(native::WorkflowError::from) },
            || cancel.cancel(),
        )?;
        Ok(NativeStrategyResult {
            inner: Arc::new(StrategyResult::Tears(Box::new(result))),
        })
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
            Strategy::Recycle(_) => Ok(vec![
                pse_backend_native::routing::Route::Native(
                    pse_backend_native::solve::Backend::Kinsol,
                )
                .into(),
            ]),
            #[cfg(feature = "native-solvers")]
            Strategy::Initialization(p) => p
                .strategies()
                .map(|routes| routes.into_iter().map(Into::into).collect())
                .map_err(|e| Python::attach(|py| errors::diagnostic(py, &e))),
        }
    }
    /// Run one bounded attempt, observing signals while retaining ownership through join.
    fn run(&self, py: Python<'_>) -> PyResult<NativeStrategyResult> {
        macro_rules! run {
            ($p:expr,$variant:ident) => {{
                let handle = py
                    .detach(|| {
                        let _enter = self.owner.executor.enter();
                        $p.start()
                    })
                    .map_err(|e| errors::diagnostic(py, &e))?;
                let cancellation = handle.cancellation();
                blocking(
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
                )?
            }};
        }
        let result = match &self.inner {
            Strategy::Cone(p) => run!(p, Cone),
            #[cfg(feature = "native-solvers")]
            Strategy::Recycle(p) => run!(p, Recycle),
            #[cfg(feature = "native-solvers")]
            Strategy::Initialization(p) => run!(p, Initialization),
        };
        Ok(NativeStrategyResult {
            inner: Arc::new(result),
        })
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
}
#[pymethods]
impl NativeStrategyResult {
    /// Every attempt in execution order, one typed row each: its native report, or the
    /// typed failure that preceded one, so indices are shared by reports and failures.
    fn attempts(&self) -> Vec<NativeStrategyAttempt> {
        let report = |r: &SolveReport| NativeStrategyAttempt {
            report: Some(NativeAttempt { inner: r.clone() }),
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
    fn initialization_json(&self) -> Option<String> {
        #[cfg(feature = "native-solvers")]
        if let StrategyResult::Initialization(r) = self.inner.as_ref() {
            let values = |v: &std::collections::BTreeMap<pse_ids::SemanticId, f64>| {
                v.iter()
                    .map(|(k, v)| (k.to_hex(), *v))
                    .collect::<std::collections::BTreeMap<_, _>>()
            };
            return Some(document(
                serde_json::json!({"original":values(&r.original.scalars),"solved_unknowns":values(&r.values.scalars),"completed_stages":r.completed_stages,"original_bindings_restored":r.original_bindings_restored,"cancelled":r.cancelled,"stages":r.stages.iter().map(|s|serde_json::json!({"stage":s.stage,"completed":s.completed,"overlay":s.overlay.iter().map(|(k,v)|(k.to_hex(),*v)).collect::<std::collections::BTreeMap<_,_>>(),"candidate":values(&s.candidate.scalars)})).collect::<Vec<_>>()}),
            ));
        }
        None
    }
    fn tears_json(&self) -> Option<String> {
        if let StrategyResult::Tears(r) = self.inner.as_ref() {
            return r.selected.as_ref().map(|s| document(serde_json::json!({"decisions":s.decisions,"connections":s.connections,"order":s.order,"cost":s.cost,"method":s.method})));
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
}
#[pymethods]
impl NativeAttempt {
    #[getter]
    fn backend(&self) -> &str {
        self.inner.backend.as_str()
    }
    #[getter]
    fn termination(&self) -> &str {
        self.inner.termination.category.as_str()
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
    fn qualification(&self) -> &str {
        self.inner.qualification.as_str()
    }
    /// Typed independent-validation failure, when validation failed.
    #[getter]
    fn validation_error(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .validation_failure()
            .map(|e| inspection::DiagnosticReport::observe(e))
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
    fn primal(&self) -> Vec<(String, f64)> {
        self.inner.candidate.as_ref().map_or_else(Vec::new, |c| {
            self.inner
                .variables
                .iter()
                .zip(&c.primal)
                .map(|(id, v)| (id.to_hex(), *v))
                .collect()
        })
    }
    fn progress(&self) -> (Vec<ProgressEvent>, u64) {
        (
            self.inner
                .events
                .iter()
                .cloned()
                .map(ProgressEvent::native)
                .collect(),
            self.inner.dropped_events,
        )
    }
    fn metrics(&self) -> ProgressEvent {
        ProgressEvent::native(pse_backend_native::solve::Event {
            phase: "final".into(),
            elapsed: Duration::ZERO,
            values: self.inner.metrics.clone(),
            incumbent: None,
        })
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
    fn start_json(&self) -> Option<String> {
        self.inner
            .start_receipt
            .as_ref()
            .map(|s| document(s.snapshot()))
    }
}
