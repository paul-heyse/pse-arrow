// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned Python handles over the generic kernel; no numerical semantics cross the boundary.
use super::*;
use pse_ids::SemanticId;
use pse_modeling::{DeclarationId, InstanceId};
use std::collections::BTreeMap;

/// A modeling declaration (a case, test or definition) named by its hex identity.
fn declaration(py: Python<'_>, text: &str) -> PyResult<DeclarationId> {
    id(py, text).map(DeclarationId::from)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FlowSelectionDocument {
    nodes: Vec<InstanceId>,
    connections: Vec<FlowConnectionDocument>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FlowConnectionDocument {
    connection: SemanticId,
    group: SemanticId,
    cost: f64,
    policy: pse_runtime::math::flows::Policy,
}
fn flow_selection(
    py: Python<'_>,
    bytes: &[u8],
    allowance: usize,
) -> PyResult<pse_runtime::math::flows::ModelingFlowSelection> {
    if bytes.len() > allowance / 4 {
        return Err(invalid(py, "flow selection exceeds workspace allowance"));
    }
    let wire = serde_json::from_slice::<strategies::AnalysisDocument<FlowSelectionDocument>>(bytes)
        .map_err(|e| invalid(py, e.to_string()))?
        .payload;
    let mut nodes = std::collections::BTreeSet::new();
    for node in wire.nodes {
        if !nodes.insert(node) {
            return Err(invalid(py, "duplicate selected flow node"));
        }
    }
    let mut connections = BTreeMap::new();
    for c in wire.connections {
        use pse_runtime::math::flows::Decision;
        if connections
            .insert(
                c.connection,
                Decision {
                    id: c.group,
                    cost: c.cost,
                    policy: c.policy,
                },
            )
            .is_some()
        {
            return Err(invalid(py, "duplicate selected connection"));
        }
    }
    Ok(pse_runtime::math::flows::ModelingFlowSelection { nodes, connections })
}

/// The registry relation of a qualified table name; each result decides whether it holds
/// that relation.
fn relation(py: Python<'_>, name: &str) -> PyResult<SemanticId> {
    pse_schema::registry()
        .map_err(|e| errors::diagnostic(py, &e))?
        .relation(name)
        .map(|spec| spec.id)
        .ok_or_else(|| invalid(py, "unknown result relation"))
}

pub(super) fn from_documents(
    runtime: &NativeRuntime,
    py: Python<'_>,
    documents: Vec<BTreeMap<String, String>>,
    physical: &NativePhysicalContext,
) -> PyResult<NativeModelingPackage> {
    let cancel = CancelSource::new();
    let inner = blocking(
        py,
        &runtime.owner,
        async {
            let pool = runtime.owner.shared.pool();
            let token = cancel.token();
            let bundles = documents
                .iter()
                .map(|documents| {
                    pse_runtime::authoring_driver::document::load_package_texts_owned(
                        documents,
                        &runtime.owner.registry,
                        pse_authoring::ParseBudget::default(),
                        &pool,
                        &token,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let owned =
                pse_runtime::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                    bundles, &pool, &token,
                )?;
            runtime
                .inner
                .modeling_from_documents(&owned, physical.inner.clone())
        },
        || cancel.cancel(),
    )?;
    Ok(NativeModelingPackage {
        owner: runtime.owner.clone(),
        inner,
        limits: Default::default(),
        sources: Some(Arc::new(native::PackageSources {
            physical: physical.documents.as_ref().clone(),
            modeling: documents,
        })),
    })
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingPackage {
    owner: Arc<runtime::Runtime>,
    inner: native::ModelingPackage,
    limits: pse_modeling::Limits,
    /// The authored documents the package was admitted from, which a durable study stores
    /// for its workers; none once the package was changed in memory.
    sources: Option<Arc<native::PackageSources>>,
}
/// Fixture-local solve and derivative inspection choices over the shared harness.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct ModelingFixturePolicy {
    inner: native::ModelingFixturePolicy,
}
#[pymethods]
impl ModelingFixturePolicy {
    #[new]
    #[pyo3(signature=(settings=None, *, derivative_step=None, derivative_tolerance=None, derivative_cells=None))]
    fn new(
        py: Python<'_>,
        settings: Option<&[u8]>,
        derivative_step: Option<f64>,
        derivative_tolerance: Option<f64>,
        derivative_cells: Option<usize>,
    ) -> PyResult<Self> {
        let settings = settings
            .map(|s| settings::solve_profile(py, s))
            .transpose()?;
        let derivatives = if derivative_step.is_some()
            || derivative_tolerance.is_some()
            || derivative_cells.is_some()
        {
            let policy = pse_backend_native::derivative_diagnostics::Policy {
                perturbation: derivative_step.unwrap_or(1e-6),
                relative_tolerance: derivative_tolerance.unwrap_or(1e-4),
                maximum_cells: derivative_cells.unwrap_or(100_000),
            };
            policy.allowance().map_err(|e| errors::diagnostic(py, &e))?;
            Some(policy)
        } else {
            None
        };
        Ok(Self {
            inner: native::ModelingFixturePolicy {
                solver: settings,
                derivatives,
            },
        })
    }
}
/// Explicit source expansion limits, independent of runtime memory and native work budgets.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ModelingLimits {
    limits: pse_modeling::Limits,
}
#[pymethods]
impl ModelingLimits {
    #[new]
    #[pyo3(signature=(*, depth=None, items=None, members=None, body_occurrences=None, body_slots=None))]
    fn new(
        py: Python<'_>,
        depth: Option<usize>,
        items: Option<usize>,
        members: Option<usize>,
        body_occurrences: Option<usize>,
        body_slots: Option<usize>,
    ) -> PyResult<Self> {
        let default = pse_modeling::Limits::default();
        let limits = pse_modeling::Limits {
            depth: depth.unwrap_or(default.depth),
            items: items.unwrap_or(default.items),
            members: members.unwrap_or(default.members),
            body_occurrences,
            body_slots,
        };
        if limits.depth == 0
            || limits.items == 0
            || limits.members == 0
            || limits.body_occurrences == Some(0)
            || limits.body_slots == Some(0)
        {
            return Err(invalid(py, "modeling expansion limits must be positive"));
        }
        Ok(Self { limits })
    }
    #[getter]
    fn depth(&self) -> usize {
        self.limits.depth
    }
    #[getter]
    fn items(&self) -> usize {
        self.limits.items
    }
    #[getter]
    fn members(&self) -> usize {
        self.limits.members
    }
    #[getter]
    fn body_occurrences(&self) -> Option<usize> {
        self.limits.body_occurrences
    }
    #[getter]
    fn body_slots(&self) -> Option<usize> {
        self.limits.body_slots
    }
}
#[pymethods]
impl NativeModelingPackage {
    fn with_declarations(&self, py: Python<'_>, source: &[u8]) -> PyResult<Self> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Edit {
            declarations: Vec<pse_authoring::language::Declaration>,
        }
        if source.len() > self.owner.shared.budget().math.workspace_bytes / 2 {
            return Err(invalid(py, "declaration edit exceeds workspace allowance"));
        }
        let edit =
            serde_json::from_slice::<Edit>(source).map_err(|e| invalid(py, e.to_string()))?;
        let inner = py
            .detach(|| self.inner.with_declarations(edit.declarations))
            .map_err(|e| errors::diagnostic(py, &e))?;
        // Edited declarations are not its authored documents: no durable study from it.
        Ok(Self {
            owner: self.owner.clone(),
            inner,
            limits: self.limits,
            sources: None,
        })
    }
    fn with_fit_data(&self, py: Python<'_>, source: &[u8]) -> PyResult<Self> {
        if source.len() > self.owner.shared.budget().math.workspace_bytes / 2 {
            return Err(invalid(py, "fit source extent"));
        }
        let data: native::FitData =
            serde_json::from_slice(source).map_err(|e| invalid(py, e.to_string()))?;
        let inner = self
            .inner
            .clone()
            .with_fit_data(data)
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self {
            owner: self.owner.clone(),
            inner,
            limits: self.limits,
            sources: None,
        })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one parameter per argument of the Python method signature"
    )]
    #[pyo3(signature=(fit_id, settings, simulations, *, rank_tolerance=1e-8, max_cells=1000000, derivatives="responses", uncertainty=None))]
    fn prepare_fit(
        &self,
        py: Python<'_>,
        fit_id: &str,
        settings: &[u8],
        simulations: Vec<(String, PyRef<'_, SimulationSettings>)>,
        rank_tolerance: f64,
        max_cells: usize,
        derivatives: &str,
        uncertainty: Option<&[u8]>,
    ) -> PyResult<NativePreparedOperation> {
        let settings = settings::solve_profile(py, settings)?;
        // The typed `fit-uncertainty` document (Plan 22 S3).
        let uncertainty = uncertainty
            .map(serde_json::from_slice::<native::FitUncertainty>)
            .transpose()
            .map_err(|e| invalid(py, e.to_string()))?;
        let fit = id(py, fit_id).map(pse_model::generated::identities::FitId::from)?;
        let count = simulations.len();
        // Experiment settings are keyed by the experiment's instance.
        let simulations = simulations
            .into_iter()
            .map(|(key, v)| id(py, &key).map(|key| (InstanceId::from(key), v.profile.clone())))
            .collect::<PyResult<BTreeMap<_, _>>>()?;
        if simulations.len() != count {
            return Err(invalid(py, "duplicate experiment settings"));
        }
        let cancel = CancelSource::new();
        let profile = native::FitProfile {
            solver: settings.clone(),
            simulations,
            rank_tolerance,
            max_cells,
            derivatives: settings::named(py, "fit derivatives", derivatives)?,
            uncertainty,
        };
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .prepare_fit(fit, profile, Default::default(), self.limits, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner: PreparedOperation::Fit(Box::new(inner)),
        })
    }

    fn with_limits(&self, limits: &ModelingLimits) -> Self {
        // Workers apply default limits: a package with its own limits runs no durable study.
        Self {
            owner: self.owner.clone(),
            inner: self.inner.clone(),
            limits: limits.limits,
            sources: None,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one parameter per argument of the Python method signature"
    )]
    #[pyo3(signature=(case_id,settings,nominals,*,penalty_tolerance,maximum_attempts,time_limit))]
    fn explain_nonlinear(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        nominals: BTreeMap<String, f64>,
        penalty_tolerance: f64,
        maximum_attempts: usize,
        time_limit: f64,
    ) -> PyResult<NativeModelingNonlinearExplanation> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let nominals = nominals
            .into_iter()
            .map(|(key, value)| Ok((id(py, &key)?, value)))
            .collect::<PyResult<_>>()?;
        let time_limit = Duration::try_from_secs_f64(time_limit)
            .map_err(|_| invalid(py, "explanation time limit must be finite and positive"))?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_analysis(
                        root,
                        pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                self.inner
                    .explain_nonlinear(
                        &analysis,
                        native::ModelingNonlinearPolicy {
                            nominals,
                            penalty_tolerance,
                            maximum_attempts,
                            time_limit,
                        },
                        &cancel,
                    )
                    .await
            },
            || cancel.cancel(),
        )?;
        Ok(NativeModelingNonlinearExplanation {
            inner: Arc::new(inner),
        })
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one parameter per argument of the Python method signature"
    )]
    fn diagnose_samples(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        diagnostics: &ModelingDiagnosticSettings,
        samples: Vec<(String, BTreeMap<String, f64>)>,
        maximum_samples: usize,
        time_limit: f64,
    ) -> PyResult<NativeModelingDiagnosticSamples> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let duration = Duration::try_from_secs_f64(time_limit)
            .map_err(|_| invalid(py, "diagnostic time limit must be finite and positive"))?;
        let samples = samples
            .into_iter()
            .map(|(name, values)| {
                Ok((
                    id(py, &name)?,
                    values
                        .into_iter()
                        .map(|(key, value)| Ok((id(py, &key)?, value)))
                        .collect::<PyResult<BTreeMap<_, _>>>()?,
                ))
            })
            .collect::<PyResult<Vec<_>>>()?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_analysis(
                        root,
                        pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                let prepared = self.inner.prepare_diagnostics(&analysis, &cancel).await?;
                let samples = samples
                    .into_iter()
                    .map(|(name, overrides)| {
                        let mut values = prepared.model.values.clone();
                        values.scalars.extend(overrides);
                        (name, values)
                    })
                    .collect();
                self.inner
                    .diagnose_samples(
                        prepared,
                        samples,
                        diagnostics.policy.clone(),
                        analysis.compiler,
                        maximum_samples,
                        duration,
                        &cancel,
                    )
                    .await
            },
            || cancel.cancel(),
        )?;
        Ok(NativeModelingDiagnosticSamples {
            inner: Arc::new(inner),
        })
    }
    /// The case's authored fixture declares its modes, events and scheduled inputs.
    fn prepare_simulation(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &SimulationSettings,
    ) -> PyResult<NativePreparedOperation> {
        let root = declaration(py, case_id)?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner.declared_simulation(
                root,
                Default::default(),
                Some(settings.profile.clone()),
                self.limits,
                &cancel,
            ),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner: PreparedOperation::Simulation(Box::new(inner)),
        })
    }
    fn simulate(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &SimulationSettings,
    ) -> PyResult<NativeModelingTrajectory> {
        let operation = self.prepare_simulation(py, case_id, settings)?;
        let PreparedOperation::Simulation(prepared) = operation.inner else {
            return Err(invalid(py, "simulation preparation mismatch"));
        };
        let cancel = CancelSource::new();
        let inner = blocking(py, &self.owner, prepared.run(&cancel), || cancel.cancel())?;
        Ok(NativeModelingTrajectory {
            inner: Arc::new(inner),
        })
    }
    fn diagnose(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        diagnostics: &ModelingDiagnosticSettings,
    ) -> PyResult<NativeModelingDiagnostics> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_analysis(
                        root,
                        pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                let prepared = self.inner.prepare_diagnostics(&analysis, &cancel).await?;
                let values = prepared.model.values.clone();
                self.inner
                    .diagnose_case(
                        prepared,
                        values,
                        diagnostics.policy.clone(),
                        analysis.compiler,
                        &cancel,
                    )
                    .await
            },
            || cancel.cancel(),
        )?;
        Ok(NativeModelingDiagnostics {
            inner: Arc::new(inner),
        })
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one parameter per argument of the Python method signature"
    )]
    #[pyo3(signature=(case_id,settings,*,rays=false,iis=false,ranging=false,relaxation=None,lower_penalties=None,upper_penalties=None,row_penalties=None,maximum_entries=100_000))]
    fn diagnose_linear(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        rays: bool,
        iis: bool,
        ranging: bool,
        relaxation: Option<(f64, f64, f64)>,
        lower_penalties: Option<BTreeMap<String, f64>>,
        upper_penalties: Option<BTreeMap<String, f64>>,
        row_penalties: Option<BTreeMap<String, f64>>,
        maximum_entries: usize,
    ) -> PyResult<NativeModelingNativeAnalysis> {
        let settings = settings::solve_profile(py, settings)?;
        #[cfg(not(feature = "solver-highs"))]
        {
            let _ = (
                case_id,
                settings,
                rays,
                iis,
                ranging,
                relaxation,
                lower_penalties,
                upper_penalties,
                row_penalties,
                maximum_entries,
            );
            Err(invalid(py, "HiGHS capability is not installed"))
        }
        #[cfg(feature = "solver-highs")]
        {
            use pse_backend_native::highs::diagnostics::{Penalties, Request};
            let root = declaration(py, case_id)?;
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                async {
                    let analysis = self
                        .inner
                        .declared_analysis(
                            root,
                            pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await?;
                    let prepared = self.inner.prepare_diagnostics(&analysis, &cancel).await?;
                    let plan = &prepared.model.case.compiled().plan;
                    let rows = plan
                        .structure()
                        .rows()
                        .iter()
                        .map(|r| r.id)
                        .collect::<Vec<_>>();
                    let align=|values:Option<BTreeMap<String,f64>>,ids:&[SemanticId]|->Result<Option<Vec<f64>>,native::WorkflowError>{
                    values.map(|values|{
                        let values=values.into_iter().map(|(key,v)|SemanticId::parse_hex(&key).map(|id|(id,v)).map_err(|e|native::WorkflowError::Contract(e.to_string()))).collect::<Result<BTreeMap<_,_>,_>>()?;
                        if values.len()!=ids.len(){return Err(native::WorkflowError::Contract("local penalties require every source coordinate exactly once".into()));}
                        ids.iter().map(|id|values.get(id).copied().ok_or_else(||native::WorkflowError::Contract("local penalty source coordinate absent".into()))).collect()
                    }).transpose()
                };
                    if relaxation.is_none()
                        && (lower_penalties.is_some()
                            || upper_penalties.is_some()
                            || row_penalties.is_some())
                    {
                        return Err(native::WorkflowError::Contract(
                            "local penalties require explicit global relaxation penalties".into(),
                        ));
                    }
                    let relaxation = relaxation
                        .map(|(lower, upper, row)| {
                            let finite = |v: f64| {
                                pse_model::scalars::FiniteBound::try_new(v).map_err(|e| {
                                    native::WorkflowError::Contract(format!(
                                        "global relaxation penalty: {e}"
                                    ))
                                })
                            };
                            Ok::<_, native::WorkflowError>(Penalties {
                                global: [finite(lower)?, finite(upper)?, finite(row)?],
                                lower: align(lower_penalties, plan.columns())?,
                                upper: align(upper_penalties, plan.columns())?,
                                rows: align(row_penalties, &rows)?,
                            })
                        })
                        .transpose()?;
                    self.inner
                        .diagnose_linear(
                            prepared,
                            Request {
                                rays,
                                iis,
                                ranging,
                                relaxation,
                                // The fixed-LP, basis-inverse, presolve and cut-pool views
                                // are not projected to Python yet (A5).
                                ..Request::default()
                            },
                            settings.controls.clone(),
                            maximum_entries,
                            &cancel,
                        )
                        .await?
                        .into_export()
                },
                || cancel.cancel(),
            )?;
            Ok(NativeModelingNativeAnalysis {
                inner: Arc::new(inner),
            })
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one parameter per argument of the Python method signature"
    )]
    #[pyo3(signature=(case_id,settings,*,maximum_rows=32,maximum_entries=100_000,maximum_attempts=64,multiplier_bound=10.0,tolerance=1e-7,rank_relative=1e-8))]
    fn diagnose_jacobian(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        maximum_rows: usize,
        maximum_entries: usize,
        maximum_attempts: usize,
        multiplier_bound: f64,
        tolerance: f64,
        rank_relative: f64,
    ) -> PyResult<NativeModelingNativeAnalysis> {
        let settings = settings::solve_profile(py, settings)?;
        #[cfg(not(feature = "solver-highs"))]
        {
            let _ = (
                case_id,
                settings,
                maximum_rows,
                maximum_entries,
                maximum_attempts,
                multiplier_bound,
                tolerance,
                rank_relative,
            );
            Err(invalid(py, "HiGHS capability is not installed"))
        }
        #[cfg(feature = "solver-highs")]
        {
            let root = declaration(py, case_id)?;
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                async {
                    let analysis = self
                        .inner
                        .declared_analysis(
                            root,
                            pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await?;
                    let prepared = self.inner.prepare_diagnostics(&analysis, &cancel).await?;
                    let values = prepared.model.values.clone();
                    self.inner
                        .diagnose_jacobian_optimization(
                            prepared,
                            values,
                            pse_backend_native::jacobian_diagnostics::Policy {
                                maximum_rows,
                                maximum_entries,
                                maximum_attempts,
                                multiplier_bound,
                                tolerance,
                                rank_relative,
                                // The MILPs are bounded by the deadline; the node
                                // budget is not projected to Python yet (A5).
                                maximum_nodes: None,
                            },
                            settings.controls.clone(),
                            &cancel,
                        )
                        .await?
                        .into_export()
                },
                || cancel.cancel(),
            )?;
            Ok(NativeModelingNativeAnalysis {
                inner: Arc::new(inner),
            })
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one parameter per argument of the Python method signature"
    )]
    #[pyo3(signature=(case_id, settings, *, stages=Vec::new(), homotopy=false, initial_step=0.25, minimum_step=1e-6, growth=1.5, maximum_attempts=128, time_limit=60.0, discrete="refuse", discrete_values=BTreeMap::new()))]
    fn initialize(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        stages: Vec<String>,
        homotopy: bool,
        initial_step: f64,
        minimum_step: f64,
        growth: f64,
        maximum_attempts: usize,
        time_limit: f64,
        discrete: &str,
        discrete_values: BTreeMap<String, f64>,
    ) -> PyResult<NativeModelingInitialization> {
        use pse_model::generated::enums::ModelingDiscreteInitialization as Discrete;
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let duration = Duration::try_from_secs_f64(time_limit)
            .map_err(|_| invalid(py, "initialization time limit must be finite and positive"))?;
        let discrete = match discrete.parse::<Discrete>() {
            Ok(Discrete::FixAt) => native::DiscreteInitialization::FixAt(discrete_values),
            Ok(Discrete::FixAtStart) if discrete_values.is_empty() => {
                native::DiscreteInitialization::FixAtStart
            }
            Ok(Discrete::Refuse) if discrete_values.is_empty() => {
                native::DiscreteInitialization::Refuse
            }
            _ => {
                return Err(invalid(
                    py,
                    "discrete is refuse, fix_at_start or fix_at; values are declared only with fix_at",
                ));
            }
        };
        let policy = native::ModelingInitialization {
            stages,
            homotopy,
            initial_step,
            minimum_step,
            growth,
            maximum_attempts,
            time_limit: duration,
            discrete,
        };
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_analysis(
                        root,
                        pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                self.inner
                    .initialize_model(&analysis, policy, &cancel)
                    .await
            },
            || cancel.cancel(),
        )?;
        Ok(NativeModelingInitialization {
            inner: Arc::new(inner),
        })
    }
    #[pyo3(signature=(case_ids, settings, *, predecessors=Vec::new(), maximum_points=1024))]
    fn study(
        &self,
        py: Python<'_>,
        case_ids: Vec<String>,
        settings: &[u8],
        predecessors: Vec<Option<usize>>,
        maximum_points: usize,
    ) -> PyResult<NativeModelingStudy> {
        let settings = settings::solve_profile(py, settings)?;
        if maximum_points == 0
            || maximum_points > 4096
            || case_ids.len() > maximum_points
            || !predecessors.is_empty() && predecessors.len() != case_ids.len()
        {
            return Err(invalid(
                py,
                "invalid bounded study extent or predecessor list",
            ));
        }
        if predecessors
            .iter()
            .enumerate()
            .any(|(i, p)| p.is_some_and(|j| j >= i))
        {
            return Err(invalid(
                py,
                "study predecessor must identify an earlier point",
            ));
        }
        let ids = case_ids
            .iter()
            .map(|v| declaration(py, v))
            .collect::<PyResult<Vec<_>>>()?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let mut points = Vec::new();
                for (index, root) in ids.into_iter().enumerate() {
                    let analysis = self
                        .inner
                        .declared_analysis(
                            root,
                            pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await
                        .map_err(Arc::new);
                    points.push(native::ModelingStudyPoint {
                        analysis,
                        predecessor: predecessors.get(index).copied().flatten(),
                    });
                }
                self.inner.study(points, maximum_points, &cancel).await
            },
            || cancel.cancel(),
        )?;
        Ok(NativeModelingStudy {
            inner: Arc::new(inner),
        })
    }
    /// Start a durable study of authored cases in `runtime`'s operational store (Plan 22
    /// O7): workers run the points, and the study publishes once in `workspace` (a
    /// workspace JSON document). `overlays` holds each point's encoded `PointOverlay`.
    #[expect(
        clippy::too_many_arguments,
        reason = "one study: store, workspace, points, settings, dependencies, overlays and job policy"
    )]
    #[pyo3(signature=(runtime, workspace, case_ids, settings, *, predecessors=Vec::new(), overlays=Vec::new(), max_tries=1, priority=0))]
    fn start_study(
        &self,
        py: Python<'_>,
        runtime: &NativeRuntime,
        workspace: &[u8],
        case_ids: Vec<String>,
        settings: &[u8],
        predecessors: Vec<Option<u32>>,
        overlays: Vec<Vec<u8>>,
        max_tries: u32,
        priority: i32,
    ) -> PyResult<NativeStudyHandle> {
        let sources = self.sources.as_ref().ok_or_else(|| {
            invalid(
                py,
                "a durable study runs the package's authored documents; this package was changed in memory",
            )
        })?;
        if !predecessors.is_empty() && predecessors.len() != case_ids.len()
            || !overlays.is_empty() && overlays.len() != case_ids.len()
        {
            return Err(invalid(
                py,
                "a study's predecessors and overlays name every point or none",
            ));
        }
        let workspace: native::Workspace =
            serde_json::from_slice(workspace).map_err(|e| invalid(py, e.to_string()))?;
        let settings = settings::solve_settings(py, settings)?;
        let points = case_ids
            .iter()
            .enumerate()
            .map(|(index, case)| {
                Ok(native::StudyPoint {
                    case: declaration(py, case)?,
                    overlay: overlays
                        .get(index)
                        .map(|overlay| settings::point_overlay(py, overlay))
                        .transpose()?
                        .unwrap_or_default(),
                    predecessor: predecessors.get(index).copied().flatten(),
                })
            })
            .collect::<PyResult<Vec<_>>>()?;
        let plan = native::StudyPlan {
            sources: sources.as_ref().clone(),
            route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            settings,
            points,
            retry: native::RetryPolicy {
                max_tries,
                ..native::RetryPolicy::ONCE
            },
            priority,
        };
        let inner = blocking(
            py,
            &runtime.owner,
            runtime.inner.start_study(&workspace, plan),
            || {},
        )?;
        Ok(NativeStudyHandle {
            owner: runtime.owner.clone(),
            inner,
        })
    }
    fn inspect(&self, py: Python<'_>, case_id: &str, settings: &[u8]) -> PyResult<Vec<u8>> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let cancel = CancelSource::new();
        let model = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_analysis(
                        root,
                        pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                self.inner
                    .prepare(
                        analysis.root,
                        analysis.instance,
                        analysis.bindings,
                        analysis.limits,
                        &cancel,
                    )
                    .await
            },
            || cancel.cancel(),
        )?;
        py.detach(|| {
            let source=&model.compiled().model;
            let lineage=|l:&pse_modeling::specialize::Lineage|serde_json::json!({"declaration":l.declaration,"instance":l.instance,"path":l.path,"demand":l.demand,"default_owner":l.default_owner,"is_override":l.is_override,"presets":l.presets});
            serde_json::to_vec(&serde_json::json!({"payload":{
                "instances":source.instances.values().map(|i|serde_json::json!({"id":i.id,"definition":i.definition,"parent":i.parent,"path":i.path,"members":i.members})).collect::<Vec<_>>(),
                "members":source.symbols.values().map(|s|serde_json::json!({"id":s.id,"role":s.role,"lineage":lineage(&s.lineage)})).collect::<Vec<_>>(),
                "ports":source.ports.values().map(|p|serde_json::json!({"id":p.id,"symbol":p.symbol,"lineage":lineage(&p.lineage)})).collect::<Vec<_>>(),
                "connections":source.connections.values().map(|c|serde_json::json!({"id":c.id,"from":c.from,"to":c.to,"lineage":lineage(&c.lineage)})).collect::<Vec<_>>()
            }}))
        }).map_err(|e|invalid(py,e.to_string()))
    }
    fn prepare_flow(
        &self,
        py: Python<'_>,
        case_id: &str,
        selection: &[u8],
        settings: &[u8],
    ) -> PyResult<NativePreparedFlow> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let selection = flow_selection(
            py,
            selection,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_analysis(
                        root,
                        pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                self.inner.prepare_flow(&analysis, selection, &cancel).await
            },
            || cancel.cancel(),
        )?;
        Ok(NativePreparedFlow {
            owner: self.owner.clone(),
            math: self.owner.shared.math().clone(),
            inner,
        })
    }
    fn prepare_recycle(
        &self,
        py: Python<'_>,
        case_id: &str,
        selection: &[u8],
        request: &[u8],
        settings: &[u8],
    ) -> PyResult<NativePreparedStrategy> {
        let settings = settings::solve_profile(py, settings)?;
        #[cfg(feature = "native-solvers")]
        {
            let root = declaration(py, case_id)?;
            let selection = flow_selection(
                py,
                selection,
                self.owner.shared.budget().math.workspace_bytes,
            )?;
            if request.len() > self.owner.shared.budget().math.workspace_bytes / 4 {
                return Err(invalid(py, "recycle request exceeds workspace allowance"));
            }
            let request = serde_json::from_slice::<
                strategies::AnalysisDocument<native::RecycleRequest>,
            >(request)
            .map_err(|e| invalid(py, e.to_string()))?
            .payload;
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                async {
                    let analysis = self
                        .inner
                        .declared_analysis(
                            root,
                            pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await?;
                    self.inner
                        .prepare_recycle(&analysis, selection, request, &cancel)
                        .await
                },
                || cancel.cancel(),
            )?;
            Ok(NativePreparedStrategy {
                owner: self.owner.clone(),
                inner: strategies::Strategy::Recycle(inner),
            })
        }
        #[cfg(not(feature = "native-solvers"))]
        {
            let _ = (case_id, selection, request, settings);
            Err(invalid(py, "KINSOL strategy workflow is not linked"))
        }
    }
    fn prepare_block_initialization(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        stages: Vec<BTreeMap<String, f64>>,
    ) -> PyResult<NativePreparedStrategy> {
        let settings = settings::solve_profile(py, settings)?;
        #[cfg(feature = "native-solvers")]
        {
            // Admission of the profile and stages is native (`validate_profile`).
            let root = declaration(py, case_id)?;
            let stages = stages
                .into_iter()
                .map(|s| {
                    s.into_iter()
                        .map(|(k, v)| id(py, &k).map(|k| (k, v)))
                        .collect::<PyResult<_>>()
                })
                .collect::<PyResult<_>>()?;
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                async {
                    let analysis = self
                        .inner
                        .declared_analysis(
                            root,
                            pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await?;
                    self.inner
                        .prepare_block_initialization(&analysis, stages, &cancel)
                        .await
                },
                || cancel.cancel(),
            )?;
            Ok(NativePreparedStrategy {
                owner: self.owner.clone(),
                inner: strategies::Strategy::Initialization(inner),
            })
        }
        #[cfg(not(feature = "native-solvers"))]
        {
            let _ = (case_id, settings, stages);
            Err(invalid(py, "initialization workflow is not linked"))
        }
    }
    fn declarations(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        py.detach(|| serde_json::to_vec(self.inner.declarations()))
            .map_err(|e| invalid(py, e.to_string()))
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one parameter per argument of the Python method signature"
    )]
    #[pyo3(signature=(settings, *, maximum_fixtures=1024, maximum_checks=16384, derivative_cells=100000, derivative_step=1e-6, derivative_tolerance=1e-4, fixture_policies=None))]
    fn conform(
        &self,
        py: Python<'_>,
        settings: &[u8],
        maximum_fixtures: usize,
        maximum_checks: usize,
        derivative_cells: usize,
        derivative_step: f64,
        derivative_tolerance: f64,
        fixture_policies: Option<BTreeMap<String, Py<ModelingFixturePolicy>>>,
    ) -> PyResult<NativeModelingConformance> {
        let settings = settings::solve_profile(py, settings)?;
        let cancel = CancelSource::new();
        let fixture_policies = fixture_policies
            .unwrap_or_default()
            .into_iter()
            .map(|(key, policy)| Ok((declaration(py, &key)?, policy.borrow(py).inner.clone())))
            .collect::<PyResult<BTreeMap<_, _>>>()?;
        let policy = native::ModelingConformancePolicy {
            fixture_policies,
            compiler: Default::default(),
            solver: settings.clone(),
            numerical: Default::default(),
            limits: self.limits,
            derivatives: pse_backend_native::derivative_diagnostics::Policy {
                perturbation: derivative_step,
                relative_tolerance: derivative_tolerance,
                maximum_cells: derivative_cells,
            },
            maximum_fixtures,
            maximum_checks,
        };
        let inner = blocking(py, &self.owner, self.inner.conform(policy, &cancel), || {
            cancel.cancel()
        })?;
        Ok(NativeModelingConformance {
            inner: Arc::new(inner),
        })
    }
    #[pyo3(signature=(case_id, settings, *, route="steady"))]
    fn prepare_solve(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        route: &str,
    ) -> PyResult<NativePreparedOperation> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let route = route
            .parse()
            .map_err(|_| invalid(py, "unknown modeling analysis route"))?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_analysis(
                        root,
                        route,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                self.inner.prepare_analysis(&analysis, &cancel).await
            },
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner: PreparedOperation::Modeling(Box::new(inner)),
        })
    }
    #[pyo3(signature=(case_id, settings, *, route="steady"))]
    fn solve_case(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        route: &str,
    ) -> PyResult<NativeModelingResult> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let route = route
            .parse()
            .map_err(|_| invalid(py, "unknown modeling analysis route"))?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_analysis(
                        root,
                        route,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                let prepared = self.inner.prepare_analysis(&analysis, &cancel).await?;
                self.inner
                    .solve_case(prepared, analysis.compiler, &cancel)
                    .await
            },
            || cancel.cancel(),
        )?;
        Ok(NativeModelingResult {
            inner: Arc::new(inner),
        })
    }
}

#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingNonlinearExplanation {
    inner: Arc<native::ModelingNonlinearExplanation>,
}
#[pymethods]
impl NativeModelingNonlinearExplanation {
    fn findings(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.findings_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }

    #[getter]
    fn complete(&self) -> bool {
        self.inner.complete
    }
    #[getter]
    fn stop(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .stop
            .as_ref()
            .map(|error| inspection::DiagnosticReport::observe(error))
    }
    fn candidate_rows(&self) -> Vec<String> {
        self.inner
            .candidate_rows
            .iter()
            .map(|id| id.to_hex())
            .collect()
    }
    fn background_variables(&self) -> Vec<String> {
        self.inner
            .background_variables
            .iter()
            .map(|id| id.to_hex())
            .collect()
    }
    fn attempts(&self) -> Vec<NativeModelingElasticAttempt> {
        (0..self.inner.attempts.len())
            .map(|index| NativeModelingElasticAttempt {
                owner: self.inner.clone(),
                index,
            })
            .collect()
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingElasticAttempt {
    owner: Arc<native::ModelingNonlinearExplanation>,
    index: usize,
}
#[pymethods]
impl NativeModelingElasticAttempt {
    /// Registry name of the attempt's observation.
    #[getter]
    fn observation(&self) -> &'static str {
        self.owner.attempts[self.index].observation.as_str()
    }
    #[getter]
    fn penalty(&self) -> Option<f64> {
        self.owner.attempts[self.index].penalty
    }
    fn omitted(&self) -> Vec<String> {
        self.owner.attempts[self.index]
            .omitted
            .iter()
            .map(|id| id.to_hex())
            .collect()
    }
    fn result(&self) -> Option<NativeModelingResult> {
        self.owner.attempts[self.index]
            .result
            .as_ref()
            .ok()
            .map(|r| NativeModelingResult {
                inner: Arc::new(r.clone()),
            })
    }
    fn failure(&self) -> Option<inspection::DiagnosticReport> {
        self.owner.attempts[self.index]
            .diagnostic()
            .as_ref()
            .map(|error| inspection::DiagnosticReport::observe(error))
    }
}

/// Explicit profile data, with native validation and bounded decoding.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingTrajectory {
    inner: Arc<native::ModelingTrajectory>,
}
#[pymethods]
impl NativeModelingTrajectory {
    #[getter]
    fn accepted(&self) -> bool {
        self.inner.accepted
    }
    #[getter]
    fn checks_complete(&self) -> bool {
        self.inner.checks_complete
    }
    #[getter]
    fn validation_error(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .validation_error
            .as_ref()
            .map(|e| inspection::DiagnosticReport::observe(e))
    }

    #[getter]
    fn termination(&self) -> &'static str {
        self.inner.report.termination.as_str()
    }
    #[getter]
    fn completed_time(&self) -> f64 {
        self.inner.report.completed_time
    }
    #[getter]
    fn samples(&self) -> usize {
        self.inner.report.samples.len()
    }
    fn failure(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .diagnostic()
            .as_ref()
            .map(|e| inspection::DiagnosticReport::observe(e))
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        let id = relation(py, name)?;
        py.detach(|| {
            self.inner.tables().and_then(|mut tables| {
                tables.remove(&id).ok_or_else(|| {
                    native::WorkflowError::Contract("trajectory table absent".into())
                })
            })
        })
        .map(inspection::TableStream::from_batch)
        .map_err(|e| errors::diagnostic(py, &e))
    }
}

/// Explicit profile data, with native validation and bounded decoding.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct ModelingDiagnosticSettings {
    policy: native::ModelingDiagnosticPolicy,
}
#[pymethods]
impl ModelingDiagnosticSettings {
    #[staticmethod]
    fn from_json(py: Python<'_>, source: &str) -> PyResult<Self> {
        if source.len() > 1 << 20 {
            return Err(invalid(py, "diagnostic profile extent"));
        }
        let policy: native::ModelingDiagnosticPolicy =
            serde_json::from_str(source).map_err(|e| invalid(py, e.to_string()))?;
        policy.validate().map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self { policy })
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        serde_json::to_string(&self.policy).map_err(|e| invalid(py, e.to_string()))
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingDiagnostics {
    inner: Arc<native::ModelingDiagnostics>,
}
/// An owned generated diagnostic transport and its unchanged native attempts.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingNativeAnalysis {
    inner: Arc<native::ModelingNativeAnalysis>,
}
#[pymethods]
impl NativeModelingNativeAnalysis {
    #[getter]
    fn relation(&self) -> &str {
        self.inner.relation
    }
    #[getter]
    fn attempt_count(&self) -> usize {
        self.inner.attempts.len()
    }
    fn attempt(&self, py: Python<'_>, index: usize) -> PyResult<NativeAttempt> {
        self.inner
            .attempts
            .get(index)
            .cloned()
            .map(|inner| NativeAttempt { inner })
            .ok_or_else(|| invalid(py, "native diagnostic attempt index"))
    }
    fn table(&self) -> inspection::TableStream {
        inspection::TableStream::from_batch(self.inner.table.clone())
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingDiagnosticSamples {
    inner: Arc<native::ModelingDiagnosticSamples>,
}
#[pymethods]
impl NativeModelingDiagnosticSamples {
    fn findings(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.findings_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }

    #[getter]
    fn unattempted(&self) -> usize {
        self.inner.unattempted
    }
    /// Registry name of the stop.
    #[getter]
    fn stop(&self) -> &'static str {
        self.inner.stop.as_str()
    }
    fn ids(&self) -> Vec<String> {
        self.inner
            .outcomes
            .iter()
            .map(|(id, _)| id.to_hex())
            .collect()
    }
    fn result(&self, py: Python<'_>, index: usize) -> PyResult<Option<NativeModelingDiagnostics>> {
        let (_, result) = self
            .inner
            .outcomes
            .get(index)
            .ok_or_else(|| invalid(py, "diagnostic sample index"))?;
        Ok(result.as_ref().ok().map(|inner| NativeModelingDiagnostics {
            inner: inner.clone(),
        }))
    }
    fn failure(
        &self,
        py: Python<'_>,
        index: usize,
    ) -> PyResult<Option<inspection::DiagnosticReport>> {
        let (_, result) = self
            .inner
            .outcomes
            .get(index)
            .ok_or_else(|| invalid(py, "diagnostic sample index"))?;
        Ok(result
            .as_ref()
            .err()
            .map(|error| inspection::DiagnosticReport::observe(error)))
    }
}
#[pymethods]
impl NativeModelingDiagnostics {
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        let id = relation(py, name)?;
        let mut tables = py
            .detach(|| self.inner.tables())
            .map_err(|e| errors::diagnostic(py, &e))?;
        tables
            .remove(&id)
            .map(inspection::TableStream::from_batch)
            .ok_or_else(|| invalid(py, "diagnostic table absent"))
    }

    #[getter]
    fn complete(&self) -> bool {
        self.inner.complete
    }
    #[getter]
    fn profile(&self) -> String {
        self.inner.profile.clone()
    }
    #[getter]
    fn rank(&self) -> Option<usize> {
        self.inner.matrix.as_ref().map(|m| m.rank)
    }
    #[getter]
    fn cutoff(&self) -> Option<f64> {
        self.inner.matrix.as_ref().map(|m| m.cutoff)
    }
    fn findings(&self) -> Vec<inspection::DiagnosticReport> {
        self.inner
            .findings
            .iter()
            .map(|d| inspection::DiagnosticReport::observe(d))
            .collect()
    }
    fn statistics(&self) -> BTreeMap<String, usize> {
        self.inner.statistics.clone()
    }
    fn coordinates(&self) -> (Vec<String>, Vec<String>) {
        (
            self.inner.rows.iter().map(|v| v.to_hex()).collect(),
            self.inner.columns.iter().map(|v| v.to_hex()).collect(),
        )
    }
    fn singular_modes(&self) -> Vec<(f64, Vec<f64>, Vec<f64>)> {
        self.inner
            .matrix
            .iter()
            .flat_map(|m| &m.modes)
            .map(|m| (m.value, m.left.clone(), m.right.clone()))
            .collect()
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingInitialization {
    inner: Arc<native::ModelingInitializationReport>,
}
#[pymethods]
impl NativeModelingInitialization {
    fn findings(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.findings_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }

    #[getter]
    fn completed(&self) -> bool {
        self.inner.completed
    }
    #[getter]
    fn failure(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .failure
            .as_ref()
            .map(|e| inspection::DiagnosticReport::observe(e))
    }
    fn committed_values(&self) -> Option<BTreeMap<String, f64>> {
        self.inner
            .committed
            .as_ref()
            .map(|v| v.iter().map(|(id, value)| (id.to_hex(), *value)).collect())
    }
    fn attempts(&self) -> Vec<NativeModelingInitializationAttempt> {
        (0..self.inner.attempts.len())
            .map(|index| NativeModelingInitializationAttempt {
                owner: self.inner.clone(),
                index,
            })
            .collect()
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingInitializationAttempt {
    owner: Arc<native::ModelingInitializationReport>,
    index: usize,
}
#[pymethods]
impl NativeModelingInitializationAttempt {
    /// Registry name of the attempted step's kind.
    #[getter]
    fn kind(&self) -> &'static str {
        self.owner.attempts[self.index].step.kind().as_str()
    }
    #[getter]
    fn stage(&self) -> Option<String> {
        match &self.owner.attempts[self.index].step {
            native::ModelingInitializationStep::Stage(name) => Some(name.clone()),
            _ => None,
        }
    }
    #[getter]
    fn fraction(&self) -> Option<f64> {
        match self.owner.attempts[self.index].step {
            native::ModelingInitializationStep::Homotopy(value) => Some(value),
            _ => None,
        }
    }
    #[getter]
    fn accepted(&self) -> bool {
        self.owner.attempts[self.index].accepted()
    }
    #[getter]
    fn interruption(&self) -> Option<inspection::DiagnosticReport> {
        self.owner.attempts[self.index]
            .interruption
            .as_ref()
            .map(|e| inspection::DiagnosticReport::observe(e))
    }
    #[getter]
    fn preparation_error(&self) -> Option<inspection::DiagnosticReport> {
        self.owner.attempts[self.index]
            .result
            .as_ref()
            .err()
            .map(|e| inspection::DiagnosticReport::observe(e.as_ref()))
    }
    fn result(&self) -> Option<NativeModelingResult> {
        self.owner.attempts[self.index]
            .result
            .as_ref()
            .ok()
            .map(|r| NativeModelingResult {
                inner: Arc::new(r.clone()),
            })
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingStudy {
    inner: Arc<native::ModelingStudyReport>,
}
#[pymethods]
impl NativeModelingStudy {
    fn findings(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.findings_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }

    #[getter]
    fn unattempted(&self) -> usize {
        self.inner.unattempted
    }
    #[getter]
    fn count(&self) -> usize {
        self.inner.outcomes.len()
    }
    fn result(&self, py: Python<'_>, index: usize) -> PyResult<Option<NativeModelingResult>> {
        let result = self
            .inner
            .outcomes
            .get(index)
            .ok_or_else(|| invalid(py, "study point index outside report"))?;
        Ok(result.as_ref().ok().map(|r| NativeModelingResult {
            inner: Arc::new(r.clone()),
        }))
    }
    fn failure(
        &self,
        py: Python<'_>,
        index: usize,
    ) -> PyResult<Option<inspection::DiagnosticReport>> {
        let result = self
            .inner
            .outcomes
            .get(index)
            .ok_or_else(|| invalid(py, "study point index outside report"))?;
        Ok(match result {
            Err(error) => Some(inspection::DiagnosticReport::observe(error)),
            Ok(result) => result
                .diagnostic()
                .as_ref()
                .map(|e| inspection::DiagnosticReport::observe(e)),
        })
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingConformance {
    inner: Arc<native::ModelingConformanceReport>,
}
#[pymethods]
impl NativeModelingConformance {
    #[staticmethod]
    #[pyo3(signature=(documents, physical, settings, *, maximum_fixtures=1024, maximum_checks=16384, limits=None))]
    fn pure(
        py: Python<'_>,
        documents: Vec<BTreeMap<String, String>>,
        physical: BTreeMap<String, String>,
        settings: &inspection::EngineSettings,
        maximum_fixtures: usize,
        maximum_checks: usize,
        limits: Option<&ModelingLimits>,
    ) -> PyResult<Self> {
        let budget = settings.resource_budget().clone();
        let executor = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(budget.threads.pool_threads.get())
            .enable_all()
            .build()
            .map_err(|e| invalid(py, e.to_string()))?;
        let cancel = CancelSource::new();
        let inner = blocking_on(
            py,
            &executor,
            native::conform_pure_documents(
                documents,
                physical,
                budget,
                Arc::new(pse_rules::invariants::RegistryRequirementPlanner),
                maximum_fixtures,
                maximum_checks,
                limits.map_or_else(Default::default, |v| v.limits),
                &cancel,
            ),
            || cancel.cancel(),
        )?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
    #[getter]
    fn passed(&self) -> bool {
        self.inner.passed()
    }
    #[getter]
    fn complete(&self) -> bool {
        self.inner.complete
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn findings(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.findings_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn failure(&self, py: Python<'_>, ordinal: usize) -> PyResult<inspection::DiagnosticReport> {
        self.inner
            .failures
            .get(ordinal)
            .map(|error| inspection::DiagnosticReport::observe(error))
            .ok_or_else(|| invalid(py, "conformance failure ordinal outside report"))
    }
    fn trajectory(&self, py: Python<'_>, fixture_id: &str) -> PyResult<NativeModelingTrajectory> {
        self.inner
            .trajectories
            .get(&declaration(py, fixture_id)?)
            .map(|value| NativeModelingTrajectory {
                inner: Arc::new(value.clone()),
            })
            .ok_or_else(|| invalid(py, "fixture has no integrated trajectory"))
    }
    fn initialization(
        &self,
        py: Python<'_>,
        fixture_id: &str,
    ) -> PyResult<NativeModelingInitialization> {
        self.inner
            .initializations
            .get(&declaration(py, fixture_id)?)
            .map(|value| NativeModelingInitialization {
                inner: Arc::new(value.clone()),
            })
            .ok_or_else(|| invalid(py, "fixture has no initialization report"))
    }
    fn fixtures(&self) -> Vec<String> {
        self.inner
            .fixtures()
            .iter()
            .map(|id| id.as_id().to_hex())
            .collect()
    }
    fn fixture_statuses(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.fixture_statuses_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn result(&self, py: Python<'_>, fixture_id: &str) -> PyResult<NativeModelingResult> {
        let identity = declaration(py, fixture_id)?;
        let inner = self.inner.results.get(&identity).ok_or_else(|| {
            invalid(
                py,
                "fixture has no completed solve; inspect conformance checks",
            )
        })?;
        Ok(NativeModelingResult {
            inner: Arc::new(inner.clone()),
        })
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingResult {
    inner: Arc<native::ModelingResult>,
}
#[pymethods]
impl NativeModelingResult {
    #[getter]
    fn accepted(&self) -> bool {
        self.inner.accepted
    }
    #[getter]
    fn run_id(&self) -> String {
        self.inner.run_id.as_id().to_hex()
    }
    #[getter]
    fn validation_error(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .validation_error
            .as_ref()
            .map(|e| inspection::DiagnosticReport::observe(e))
    }
    /// Registry run state of the outcome.
    #[getter]
    fn outcome_kind(&self) -> &'static str {
        self.inner.outcome.state().as_str()
    }
    fn attempt(&self) -> Option<NativeAttempt> {
        match &self.inner.outcome {
            pse_runtime::math::solves::Outcome::Native(report) => Some(NativeAttempt {
                inner: report.as_ref().clone(),
            }),
            _ => None,
        }
    }
    fn failure(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .diagnostic()
            .as_ref()
            .map(|e| inspection::DiagnosticReport::observe(e))
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        let relation = relation(py, name)?;
        py.detach(|| {
            self.inner.tables().and_then(|mut tables| {
                tables.remove(&relation).ok_or_else(|| {
                    native::WorkflowError::Contract("modeling result relation absent".into())
                })
            })
        })
        .map(inspection::TableStream::from_batch)
        .map_err(|e| errors::diagnostic(py, &e))
    }
}
