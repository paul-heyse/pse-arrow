// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical owned analysis requests. Native preparation owns all eligibility and iteration.
use super::*;
use pse_backend_native::solve::SolveReport;
use pse_runtime::math::solves::{Outcome, SequenceReport};

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
            "decisions":d.decisions.iter().map(|d|serde_json::json!({"id":d.id,"cost":d.cost,"policy":format!("{:?}",d.policy)})).collect::<Vec<_>>()
        }))
    }
    fn select_tears(
        &self,
        py: Python<'_>,
        method: &str,
        settings: &SolveSettings,
    ) -> PyResult<NativeStrategyResult> {
        use pse_runtime::math::flows::TearMethod;
        let method = match method {
            "highs" => TearMethod::Highs,
            "unweighted_heuristic" => TearMethod::UnweightedHeuristic,
            _ => return Err(invalid(py, "unknown tear method")),
        };
        if settings.profile.controls.start != pse_backend_native::solve::StartPolicy::NoPriorStart
            || settings.profile.controls.reuse != pse_backend_native::solve::ReusePolicy::Fresh
        {
            return Err(invalid(
                py,
                "tear selection has no retained seed or allocation",
            ));
        }
        let handle = py
            .detach(|| {
                let _enter = self.owner.executor.enter();
                self.math.select_tears(
                    self.inner.clone(),
                    method,
                    settings.profile.controls.clone(),
                )
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
    Cone(native::PreparedConic),
    #[cfg(feature = "native-solvers")]
    Recycle(native::PreparedRecycle),
    #[cfg(feature = "native-solvers")]
    Initialization(native::PreparedInitializationStrategy),
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
    #[getter]
    fn routes(&self) -> PyResult<Vec<String>> {
        match &self.inner {
            Strategy::Cone(p) => Ok(vec![format!("{:?}", p.solve().route())]),
            #[cfg(feature = "native-solvers")]
            Strategy::Recycle(_) => Ok(vec!["Kinsol.FixedPoint".into()]),
            #[cfg(feature = "native-solvers")]
            Strategy::Initialization(p) => p
                .strategies()
                .map(|routes| routes.iter().map(|r| format!("{r:?}")).collect())
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
    Cone(Box<SequenceReport>),
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
    fn attempts(&self) -> Vec<NativeAttempt> {
        let reports: Vec<&SolveReport> = match self.inner.as_ref() {
            StrategyResult::Tears(r) => r.attempt.iter().collect(),
            StrategyResult::Cone(r) => r
                .outcomes
                .iter()
                .filter_map(|o| {
                    if let Outcome::Native(r) = o {
                        Some(r.as_ref())
                    } else {
                        None
                    }
                })
                .collect(),
            #[cfg(feature = "native-solvers")]
            StrategyResult::Recycle(r) => vec![&r.report],
            #[cfg(feature = "native-solvers")]
            StrategyResult::Initialization(r) => r
                .attempts
                .iter()
                .filter_map(|a| a.result.as_ref().ok().map(AsRef::as_ref))
                .collect(),
        };
        reports
            .into_iter()
            .map(|r| NativeAttempt { inner: r.clone() })
            .collect()
    }
    fn failures(&self) -> Vec<(usize, String)> {
        match self.inner.as_ref() {
            StrategyResult::Tears(_) => vec![],
            StrategyResult::Cone(r) => r
                .outcomes
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    if let Outcome::Rejected(e) = o {
                        Some((i, e.to_string()))
                    } else {
                        None
                    }
                })
                .collect(),
            #[cfg(feature = "native-solvers")]
            StrategyResult::Initialization(r) => r
                .attempts
                .iter()
                .enumerate()
                .filter_map(|(i, a)| a.result.as_ref().err().map(|e| (i, e.to_string())))
                .collect(),
            #[cfg(feature = "native-solvers")]
            StrategyResult::Recycle(_) => vec![],
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
/// Owned native attempt, retaining stop, qualification, original values and bounded history separately.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeAttempt {
    inner: SolveReport,
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
    #[getter]
    fn validation_error(&self) -> Option<&str> {
        self.inner.validation_error.as_deref()
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
                .map(ProgressEvent)
                .collect(),
            self.inner.dropped_events,
        )
    }
    fn metrics(&self) -> ProgressEvent {
        ProgressEvent(pse_backend_native::solve::Event {
            phase: "final".into(),
            elapsed: Duration::ZERO,
            values: self.inner.metrics.clone(),
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
        self.inner.start_receipt.as_ref().map(|s| document(serde_json::json!({
            "previous_attempt":s.previous_attempt,"seed":s.seed.as_ref().map(|s|s.snapshot()),
            "sparse_seed":s.sparse_seed,"transformations":s.transformations,"submitted":s.submitted,
        })))
    }
}
