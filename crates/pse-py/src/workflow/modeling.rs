// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned Python handles over the generic kernel; no numerical semantics cross the boundary.
use super::*;
use crate::enums::EnumValue;
use pse_ids::SemanticId;
use pse_model::generated::enums::{
    ModelingDiagnosticSampleStop, ModelingElasticObservation, ModelingInitializationStep,
    NativeRunState, TrajectoryTermination,
};
use pse_modeling::DeclarationId;
use std::collections::BTreeMap;

/// A modeling declaration (a case, test or definition) named by its hex identity.
fn declaration(py: Python<'_>, text: &str) -> PyResult<DeclarationId> {
    id(py, text).map(DeclarationId::from)
}
/// Use the complete canonical preparation document or the existing Rust defaults.
fn preparation_settings(
    py: Python<'_>,
    bytes: Option<&[u8]>,
    limits: pse_modeling::Limits,
    allowance: usize,
) -> PyResult<native::PreparationSettings> {
    match bytes {
        Some(bytes) => documents::decode(py, "preparation settings", bytes, allowance / 4),
        None => Ok(native::PreparationSettings {
            limits,
            ..Default::default()
        }),
    }
}
/// Decode the flow selection through its Rust-owned document contract.
fn flow_selection(
    py: Python<'_>,
    bytes: &[u8],
    allowance: usize,
) -> PyResult<pse_runtime::math::flows::ModelingFlowSelection> {
    let document: pse_runtime::math::flows::FlowSelectionDocument =
        documents::decode(py, "flow selection", bytes, allowance / 4)?;
    pse_runtime::math::flows::ModelingFlowSelection::try_from(document)
        .map_err(|error| errors::diagnostic(py, &pse_runtime::math::MathRuntimeError::from(error)))
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
    documents: Vec<BTreeMap<String, DocumentContent>>,
    physical: &NativePhysicalContext,
) -> PyResult<NativeModelingPackage> {
    let documents = documents
        .into_iter()
        .map(document_bytes)
        .collect::<Vec<_>>();
    let cancel = CancelSource::new();
    let inner = blocking(
        py,
        &runtime.owner,
        async {
            let pool = runtime.owner.shared.pool();
            let token = cancel.token();
            let validation = runtime
                .owner
                .sessions
                .validation_context(&runtime.owner.registry)?;
            let bundles = documents
                .iter()
                .map(|documents| {
                    pse_runtime::authoring_driver::document::load_package_documents_owned(
                        documents,
                        &runtime.owner.registry,
                        pse_authoring::ParseBudget::default(),
                        &pool,
                        &token,
                        &validation,
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
                .await
        },
        || cancel.cancel(),
    )?;
    Ok(NativeModelingPackage {
        owner: runtime.owner.clone(),
        inner,
        limits: Default::default(),
        physical_sources: Some(physical.documents.clone()),
    })
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingPackage {
    owner: Arc<runtime::Runtime>,
    inner: native::ModelingPackage,
    limits: pse_modeling::Limits,
    /// Shared whole physical prerequisite; modeling sources live only in the canonical store.
    physical_sources: Option<Arc<BTreeMap<String, Vec<u8>>>>,
}
impl NativeModelingPackage {
    pub(super) fn from_revision(
        owner: Arc<runtime::Runtime>,
        inner: native::ModelingPackage,
        physical_sources: Arc<BTreeMap<String, Vec<u8>>>,
    ) -> Self {
        Self {
            owner,
            inner,
            limits: Default::default(),
            physical_sources: Some(physical_sources),
        }
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
    #[getter]
    fn canonical_revision(&self) -> String {
        self.inner.canonical_revision().key.clone()
    }
    #[getter]
    fn canonical_problem(&self) -> String {
        self.inner.canonical_revision().problem.clone()
    }
    fn with_declarations(&self, py: Python<'_>, source: &[u8]) -> PyResult<Self> {
        let edit: native::DeclarationEdit = documents::decode(
            py,
            "declaration edit",
            source,
            self.owner.shared.budget().math.workspace_bytes / 2,
        )?;
        let inner = blocking(
            py,
            &self.owner,
            self.inner.with_declarations(edit.declarations),
            || {},
        )?;
        // The changed declarations now have their own immutable canonical revision.
        Ok(Self {
            owner: self.owner.clone(),
            inner,
            limits: self.limits,
            physical_sources: self.physical_sources.clone(),
        })
    }
    fn with_fit_declarations(&self, py: Python<'_>, source: &[u8]) -> PyResult<Self> {
        if source.len() > self.owner.shared.budget().math.workspace_bytes / 2 {
            return Err(invalid(py, "fit source extent"));
        }
        let data: native::FitDeclarations = documents::decode(
            py,
            "operation",
            source,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let inner = blocking(
            py,
            &self.owner,
            self.inner.clone().with_fit_declarations(data),
            || {},
        )?;
        Ok(Self {
            owner: self.owner.clone(),
            inner,
            limits: self.limits,
            physical_sources: self.physical_sources.clone(),
        })
    }

    fn prepare_fit(
        &self,
        py: Python<'_>,
        fit_id: &str,
        request: &[u8],
    ) -> PyResult<NativePreparedOperation> {
        let request: native::FitPreparationDocument = documents::decode(
            py,
            "fit preparation",
            request,
            self.owner.shared.budget().math.workspace_bytes / 2,
        )?;
        let profile = request
            .profile()
            .map_err(|error| errors::diagnostic(py, &error))?;
        let fit = id(py, fit_id).map(pse_model::generated::identities::FitId::from)?;
        let cancel = CancelSource::new();
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
            physical_sources: None,
        }
    }

    #[pyo3(signature=(case_id, settings, policy))]
    fn explain_nonlinear(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        policy: &[u8],
    ) -> PyResult<NativeModelingNonlinearExplanation> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let policy: native::ModelingNonlinearPolicy = documents::decode(
            py,
            "nonlinear explanation",
            policy,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let analysis = self
                    .inner
                    .declared_execution(
                        root,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?
                    .analysis;
                self.inner
                    .explain_nonlinear(&analysis, policy, &cancel)
                    .await
            },
            || cancel.cancel(),
        )?;
        Ok(NativeModelingNonlinearExplanation {
            inner: Arc::new(inner),
        })
    }
    #[pyo3(signature=(case_id, settings, diagnostics, samples, *, controls=None))]
    fn diagnose_samples(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        diagnostics: &ModelingDiagnosticSettings,
        samples: Vec<(String, BTreeMap<String, f64>)>,
        controls: Option<&[u8]>,
    ) -> PyResult<NativeModelingDiagnosticSamples> {
        let native::DiagnosticSamplesControls {
            maximum_samples,
            time_limit,
        } = documents::controls(
            py,
            "diagnostic sample controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
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
                    .declared_execution(
                        root,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?
                    .analysis;
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
                    .declared_execution(
                        root,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?
                    .analysis;
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
    #[pyo3(signature=(case_id, settings, *, controls=None))]
    fn diagnose_linear(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        controls: Option<&[u8]>,
    ) -> PyResult<NativeModelingNativeAnalysis> {
        let controls: native::LinearDiagnosticControls = documents::controls(
            py,
            "linear diagnostic controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let settings = settings::solve_profile(py, settings)?;
        #[cfg(not(feature = "solver-highs"))]
        {
            let _ = (case_id, settings, controls);
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
                        .declared_execution(
                            root,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await?
                        .analysis;
                    let prepared = self.inner.prepare_diagnostics(&analysis, &cancel).await?;
                    let plan = &prepared.model.case.compiled().plan;
                    let rows = plan
                        .structure()
                        .rows()
                        .iter()
                        .map(|r| r.id)
                        .collect::<Vec<_>>();
                    let maximum_entries = controls.maximum_entries;
                    let request = controls.request(plan.columns(), &rows)?;
                    self.inner
                        .diagnose_linear(
                            prepared,
                            request,
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
    #[pyo3(signature=(case_id, settings, *, controls=None))]
    fn diagnose_jacobian(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        controls: Option<&[u8]>,
    ) -> PyResult<NativeModelingNativeAnalysis> {
        let controls: native::JacobianDiagnosticControls = documents::controls(
            py,
            "Jacobian diagnostic controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let settings = settings::solve_profile(py, settings)?;
        #[cfg(not(feature = "solver-highs"))]
        {
            let _ = (case_id, settings, controls);
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
                        .declared_execution(
                            root,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await?
                        .analysis;
                    let prepared = self.inner.prepare_diagnostics(&analysis, &cancel).await?;
                    let values = prepared.model.values.clone();
                    self.inner
                        .diagnose_jacobian_optimization(
                            prepared,
                            values,
                            controls.policy(),
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
    #[pyo3(signature=(case_id, settings, *, overrides=None))]
    fn initialize(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        overrides: Option<&[u8]>,
    ) -> PyResult<NativeModelingInitialization> {
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let overrides: native::InitializationOverrides = documents::controls(
            py,
            "initialization",
            overrides,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let execution = self
                    .inner
                    .declared_execution(
                        root,
                        Default::default(),
                        settings,
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?;
                self.inner
                    .initialize_declared(&execution, overrides, &cancel)
                    .await
            },
            || cancel.cancel(),
        )?;
        Ok(NativeModelingInitialization {
            inner: Arc::new(inner),
        })
    }
    /// Admit the generated request once under this immutable selected revision.
    fn admit_study(&self, py: Python<'_>, request: &[u8]) -> PyResult<Vec<u8>> {
        let request = documents::decode_versioned::<native::StudyRequest, 4>(
            py,
            "operation",
            request,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let physical=self.physical_sources.as_deref().ok_or_else(||invalid(py,"study admission requires its admitted physical source documents"))?;
        let cancel = CancelSource::new();
        let definition = blocking(
            py,
            &self.owner,
            self.inner
                .admit_study_sources(physical, &request.points, &cancel),
            || cancel.cancel(),
        )?;
        documents::encode(py, &definition)
    }
    /// Execute the same admitted document accepted by the durable adapter.
    #[pyo3(signature=(definition, *, controls=None))]
    fn study(
        &self,
        py: Python<'_>,
        definition: &[u8],
        controls: Option<&[u8]>,
    ) -> PyResult<NativeStudyReport> {
        let native::StudyRunControls { maximum_points } = documents::controls(
            py,
            "study",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let definition = documents::decode::<native::StudyDefinition>(
            py,
            "operation",
            definition,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner.study(&definition, maximum_points, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeStudyReport {
            inner: Arc::new(inner),
            owner: self.owner.clone(),
        })
    }
    /// Persist the admitted document and reopen its exact canonical modeling revision.
    #[pyo3(signature=(runtime, definition))]
    fn start_study(
        &self,
        py: Python<'_>,
        runtime: &NativeRuntime,
        definition: &[u8],
    ) -> PyResult<NativeStudyHandle> {
        let physical_sources = self.physical_sources.as_ref().ok_or_else(|| {
            invalid(
                py,
                "a durable study requires the shared physical source prerequisite",
            )
        })?;
        let definition = documents::decode::<native::StudyDefinition>(
            py,
            "operation",
            definition,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let inner = blocking(
            py,
            &runtime.owner,
            runtime.inner.start_defined_study(
                physical_sources.as_ref().clone(),
                definition,
            ),
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
        let execution = blocking(
            py,
            &self.owner,
            async {
                self.inner
                    .declared_execution(
                        root,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await
            },
            || cancel.cancel(),
        )?;
        documents::encode(py, &execution.inspection())
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
                    .declared_execution(
                        root,
                        Default::default(),
                        settings.clone(),
                        Default::default(),
                        self.limits,
                        &cancel,
                    )
                    .await?
                    .analysis;
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
            let request: native::RecycleRequest = documents::decode(
                py,
                "recycle",
                request,
                self.owner.shared.budget().math.workspace_bytes / 4,
            )?;
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                async {
                    let analysis = self
                        .inner
                        .declared_execution(
                            root,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await?
                        .analysis;
                    self.inner
                        .prepare_recycle(&analysis, selection, request, &cancel)
                        .await
                },
                || cancel.cancel(),
            )?;
            Ok(NativePreparedStrategy {
                owner: self.owner.clone(),
                inner: strategies::Strategy::Recycle(Box::new(inner)),
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
                        .declared_execution(
                            root,
                            Default::default(),
                            settings.clone(),
                            Default::default(),
                            self.limits,
                            &cancel,
                        )
                        .await?
                        .analysis;
                    self.inner
                        .prepare_block_initialization(&analysis, stages, &cancel)
                        .await
                },
                || cancel.cancel(),
            )?;
            Ok(NativePreparedStrategy {
                owner: self.owner.clone(),
                inner: strategies::Strategy::Initialization(Box::new(inner)),
            })
        }
        #[cfg(not(feature = "native-solvers"))]
        {
            let _ = (case_id, settings, stages);
            Err(invalid(py, "initialization workflow is not linked"))
        }
    }
    fn declarations(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        use pse_model::HeapUsage;
        let declarations = blocking(py, &self.owner, self.inner.declarations(), || {})?;
        let bytes = declarations
            .iter()
            .try_fold(4096usize, |bytes, row| bytes.checked_add(row.owned_bytes()))
            .and_then(|bytes| bytes.checked_mul(8))
            .ok_or_else(|| invalid(py, "declaration export extent"))?;
        let pool = self.owner.shared.pool();
        let scratch =
            pse_columnar::MemoryConsumer::new("modeling:python-declaration-export").register(&pool);
        scratch
            .try_grow(bytes)
            .map_err(|error| errors::diagnostic(py, &pse_columnar::CanonError::from(error)))?;
        documents::encode(
            py,
            &native::DeclarationInventory {
                declarations: declarations.to_vec(),
            },
        )
    }
    /// Explicit complete source Arrow export, independent of routine preparation.
    fn source_tables(&self, py: Python<'_>) -> PyResult<BTreeMap<String, inspection::TableStream>> {
        let tables = blocking(py, &self.owner, self.inner.source_tables(), || {})?;
        tables
            .into_iter()
            .map(|(relation, batch)| {
                inspection::TableStream::from_batch(batch)
                    .map(|stream| (relation.to_string(), stream))
                    .map_err(|error| errors::diagnostic(py, &error))
            })
            .collect()
    }
    #[pyo3(signature=(owner_id=None, *, controls=None))]
    fn knowledge(
        &self,
        py: Python<'_>,
        owner_id: Option<&str>,
        controls: Option<&[u8]>,
    ) -> PyResult<NativeModelingKnowledge> {
        let native::KnowledgeControls {
            maximum_cells,
            maximum_bytes,
        } = documents::controls(
            py,
            "knowledge controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let selected = owner_id.map(|id| declaration(py, id)).transpose()?;
        let cancel = pse_columnar::CancellationToken::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .knowledge(selected, maximum_cells, maximum_bytes, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeModelingKnowledge {
            inner: Arc::new(inner),
        })
    }
    #[pyo3(signature=(settings, *, controls=None, preparation=None))]
    fn conform(
        &self,
        py: Python<'_>,
        settings: &[u8],
        controls: Option<&[u8]>,
        preparation: Option<&[u8]>,
    ) -> PyResult<NativeModelingConformance> {
        let preparation = preparation_settings(
            py,
            preparation,
            self.limits,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let controls: native::ConformanceControls = documents::controls(
            py,
            "conformance controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let fixtures = controls
            .selection()
            .map_err(|error| errors::diagnostic(py, &error))?;
        let native::ConformanceControls {
            maximum_fixtures,
            maximum_checks,
            derivative_cells,
            derivative_step,
            derivative_tolerance,
            diagnostics,
            ..
        } = controls;
        let settings = settings::solve_profile(py, settings)?;
        let cancel = CancelSource::new();
        let policy = native::ModelingConformancePolicy {
            compiler: preparation.compiler,
            solver: settings.clone(),
            numerical: Default::default(),
            limits: preparation.limits,
            derivatives: pse_backend_native::derivative_diagnostics::Policy {
                perturbation: derivative_step,
                relative_tolerance: derivative_tolerance,
                maximum_cells: derivative_cells,
            },
            maximum_fixtures,
            maximum_checks,
            fixtures,
            diagnostics,
        };
        let inner = blocking(py, &self.owner, self.inner.conform(policy, &cancel), || {
            cancel.cancel()
        })?;
        Ok(NativeModelingConformance {
            inner: Arc::new(inner),
        })
    }
    #[pyo3(signature=(case_id, settings, *, preparation=None))]
    fn prepare_solve(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        preparation: Option<&[u8]>,
    ) -> PyResult<NativePreparedOperation> {
        let preparation = preparation_settings(
            py,
            preparation,
            self.limits,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let execution = self
                    .inner
                    .declared_execution(
                        root,
                        preparation.compiler,
                        settings.clone(),
                        Default::default(),
                        preparation.limits,
                        &cancel,
                    )
                    .await?;
                self.inner.prepare_declared(&execution, &cancel).await
            },
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner: PreparedOperation::Modeling(Box::new(inner)),
        })
    }
    #[pyo3(signature=(case_id, settings, *, preparation=None))]
    fn solve_case(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &[u8],
        preparation: Option<&[u8]>,
    ) -> PyResult<NativeModelingResult> {
        let preparation = preparation_settings(
            py,
            preparation,
            self.limits,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let settings = settings::solve_profile(py, settings)?;
        let root = declaration(py, case_id)?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let execution = self
                    .inner
                    .declared_execution(
                        root,
                        preparation.compiler,
                        settings.clone(),
                        Default::default(),
                        preparation.limits,
                        &cancel,
                    )
                    .await?;
                self.inner.execute_declared(&execution, &cancel).await
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
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
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
            .map(inspection::DiagnosticReport::observe)
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
    fn observation(&self) -> EnumValue<ModelingElasticObservation> {
        self.owner.attempts[self.index].observation.into()
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
            .map(inspection::DiagnosticReport::observe)
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
        self.inner.accepted()
    }
    #[getter]
    fn checks_complete(&self) -> bool {
        self.inner.checks_complete()
    }
    #[getter]
    fn validation_error(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .validation_error()
            .map(inspection::DiagnosticReport::observe)
    }

    #[getter]
    fn termination(&self) -> EnumValue<TrajectoryTermination> {
        self.inner.report().termination.into()
    }
    #[getter]
    fn completed_time(&self) -> f64 {
        self.inner.report().completed_time
    }
    #[getter]
    fn samples(&self) -> usize {
        self.inner.report().samples.len()
    }
    fn failure(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .diagnostic()
            .as_ref()
            .map(inspection::DiagnosticReport::observe)
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table(name))
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
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
        let policy: native::ModelingDiagnosticPolicy =
            documents::decode(py, "diagnostic profile", source.as_bytes(), 1 << 20)?;
        policy.validate().map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self { policy })
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        String::from_utf8(documents::encode(py, &self.policy)?)
            .map_err(|error| invalid(py, error.to_string()))
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingKnowledge {
    inner: Arc<native::ModelingKnowledge>,
}
#[pymethods]
impl NativeModelingKnowledge {
    #[getter]
    fn source_revision(&self) -> String {
        self.inner.source_revision().to_prefixed()
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        inspection::TableStream::from_batch(self.inner.table().clone())
            .map_err(|e| errors::diagnostic(py, &e))
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
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        inspection::TableStream::from_batch(self.inner.table.clone())
            .map_err(|e| errors::diagnostic(py, &e))
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
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }

    #[getter]
    fn unattempted(&self) -> usize {
        self.inner.unattempted
    }
    /// Registry name of the stop.
    #[getter]
    fn stop(&self) -> EnumValue<ModelingDiagnosticSampleStop> {
        self.inner.stop.into()
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
            .map(inspection::DiagnosticReport::observe))
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
            .ok_or_else(|| invalid(py, "diagnostic table absent"))?
            .map_err(|e| errors::diagnostic(py, &e))
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
            .map(inspection::DiagnosticReport::observe)
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
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
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
            .map(inspection::DiagnosticReport::observe)
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
    fn kind(&self) -> EnumValue<ModelingInitializationStep> {
        self.owner.attempts[self.index].step.kind().into()
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
            .map(inspection::DiagnosticReport::observe)
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
pub(crate) struct NativeStudyReport {
    inner: Arc<native::StudyReport>,
    owner: Arc<runtime::Runtime>,
}
#[pymethods]
impl NativeStudyReport {
    fn findings(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.findings_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    #[getter]
    fn unattempted(&self) -> usize {
        self.inner
            .outcomes
            .iter()
            .filter(|point| point.attempts.is_empty())
            .count()
    }
    #[getter]
    fn count(&self) -> usize {
        self.inner.outcomes.len()
    }
    #[getter]
    fn preparations(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        documents::encode(py, &self.inner.preparations)
    }
    #[getter]
    fn conclusion(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        documents::encode(py, &self.inner.decision.conclusion)
    }
    fn outcome(&self, py: Python<'_>, index: usize) -> PyResult<Vec<u8>> {
        let outcome = self
            .inner
            .outcomes
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        documents::encode(py, outcome)
    }
    fn result(&self, py: Python<'_>, index: usize) -> PyResult<Option<NativeRunResult>> {
        let result = self
            .inner
            .results
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        match result {
            Some(native::StudyOccurrenceResult::Ephemeral(inner))=>Ok(Some(NativeRunResult{owner:self.owner.clone(),inner:inner.clone()})),
            Some(native::StudyOccurrenceResult::Retained{..})=>Err(invalid(py,"retained occurrence uses its stored result handle")),
            None=>Ok(None),
        }
    }
    fn stored_result(&self,py:Python<'_>,index:usize)->PyResult<Option<NativeStoredResult>>{
        let result=self.inner.results.get(index).ok_or_else(||invalid(py,"study occurrence outside report"))?;
        Ok(result.as_ref().and_then(|result|result.stored_keys().map(|(run,attempt)|NativeStoredResult{inner:self.inner.runtime().clone(),owner:self.owner.clone(),run_id:result.run_id(),run:run.into(),attempt:attempt.into()})))
    }
    fn failure(
        &self,
        py: Python<'_>,
        index: usize,
    ) -> PyResult<Option<inspection::DiagnosticReport>> {
        let outcome = self
            .inner
            .outcomes
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        Ok(outcome
            .diagnostic
            .as_ref()
            .map(inspection::DiagnosticReport::observe))
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
    #[pyo3(signature=(documents, physical, settings, *, controls=None, limits=None, preparation=None))]
    fn pure(
        py: Python<'_>,
        documents: Vec<BTreeMap<String, DocumentContent>>,
        physical: BTreeMap<String, DocumentContent>,
        settings: &inspection::EngineSettings,
        controls: Option<&[u8]>,
        limits: Option<&ModelingLimits>,
        preparation: Option<&[u8]>,
    ) -> PyResult<Self> {
        let preparation = preparation_settings(
            py,
            preparation,
            limits.map_or_else(Default::default, |value| value.limits),
            settings.resource_budget().math.workspace_bytes,
        )?;
        let controls: native::PureConformanceControls = documents::controls(
            py,
            "pure conformance controls",
            controls,
            settings.resource_budget().math.workspace_bytes,
        )?;
        let selection = controls
            .selection()
            .map_err(|error| errors::diagnostic(py, &error))?;
        let native::PureConformanceControls {
            maximum_fixtures,
            maximum_checks,
            ..
        } = controls;
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
                documents.into_iter().map(document_bytes).collect(),
                document_bytes(physical),
                budget,
                selection,
                maximum_fixtures,
                maximum_checks,
                preparation,
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
    /// Whether the run executed a fixture selection, which assesses no package coverage.
    #[getter]
    fn selected(&self) -> bool {
        matches!(
            self.inner.selection,
            native::ModelingFixtureSelection::Selected(_)
        )
    }
    fn table(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn findings(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.findings_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    /// Retained original route/structure facts, including refused fixtures.
    fn admission(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        let id = relation(py, name)?;
        py.detach(|| {
            self.inner.admission_tables().and_then(|mut tables| {
                tables.remove(&id).ok_or_else(|| {
                    native::WorkflowError::Input("conformance admission table absent".into())
                })
            })
        })
        .map(inspection::TableStream::from_batch)
        .map_err(|e| errors::diagnostic(py, &e))?
        .map_err(|e| errors::diagnostic(py, &e))
    }
    /// The oracle parity report projected from the run's checks (Plan 23 H6).
    fn parity(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.parity_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn failure(&self, py: Python<'_>, ordinal: usize) -> PyResult<inspection::DiagnosticReport> {
        self.inner
            .failures
            .get(ordinal)
            .map(inspection::DiagnosticReport::observe)
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
            .map_err(|e| errors::diagnostic(py, &e))?
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
            .map(inspection::DiagnosticReport::observe)
    }
    /// Registry run state of the outcome.
    #[getter]
    fn outcome_kind(&self) -> EnumValue<NativeRunState> {
        self.inner.outcome.state().into()
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
            .map(inspection::DiagnosticReport::observe)
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        let relation = relation(py, name)?;
        py.detach(|| {
            self.inner.tables().and_then(|mut tables| {
                tables.remove(&relation).ok_or_else(|| {
                    native::WorkflowError::Input("modeling result relation absent".into())
                })
            })
        })
        .map(inspection::TableStream::from_batch)
        .map_err(|e| errors::diagnostic(py, &e))?
        .map_err(|e| errors::diagnostic(py, &e))
    }
}
