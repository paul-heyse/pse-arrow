// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned Python handles over the generic kernel; no numerical semantics cross the boundary.
use super::*;
use crate::enums::EnumValue;
use crate::identities::{HashValue, Identity};
use pse_ids::SemanticId;
use pse_model::generated::enums::{
    ModelingDiagnosticSampleStop, ModelingElasticObservation, ModelingInitializationStep,
    NativeRunState, TrajectoryTermination,
};
use pse_modeling::DeclarationId;
use std::collections::BTreeMap;

/// The one declaration document, retaining its source and prepaid copy through Python conversion.
pub(crate) struct DeclarationExport {
    document: native::DeclarationInventory,
    _source: native::OwnedDeclarations,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl serde::Serialize for DeclarationExport {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.document.serialize(serializer)
    }
}
documents::projection!(DeclarationExport => native::DeclarationInventory);

/// Positional native transport for the public Python `SingularMode` record:
/// value, left/right vectors, optional left/right residuals, normalization and bound.
type SingularModeTuple = (f64, Vec<f64>, Vec<f64>, Option<f64>, Option<f64>, f64, f64);

/// Preserve the registry declaration role; contextual membership remains runtime admission.
fn declaration(value: Identity<DeclarationId>) -> DeclarationId {
    value.0
}

/// A decoded covariance document carries no process-local statistical authority.
/// Recover it only from the retained actual fit, checking the complete document.
fn covariance_source(
    py: Python<'_>,
    settings: &mut pse_runtime::math::settings::SolveSettings,
    source: Option<&NativeRunResult>,
) -> PyResult<()> {
    if let Some(source) = source {
        let covariance = settings
            .sensitivity
            .as_mut()
            .and_then(|request| request.propagation.as_mut())
            .map(|propagation| &mut propagation.covariance)
            .ok_or_else(|| {
                invalid(
                    py,
                    "a covariance source requires a covariance propagation request",
                )
            })?;
        source
            .inner
            .reconcile_parameter_covariance(covariance)
            .map_err(|error| errors::diagnostic(py, &error))?;
    }
    Ok(())
}
/// Use the complete canonical preparation document or the existing Rust defaults.
fn preparation_settings(
    value: Option<documents::DocumentInput<native::PreparationSettings>>,
    limits: pse_modeling::Limits,
) -> native::PreparationSettings {
    value.map_or_else(
        || native::PreparationSettings::with_limits(limits),
        |value| value.0,
    )
}
/// Decode the flow selection through its Rust-owned document contract.
fn flow_selection(
    py: Python<'_>,
    bytes: &[u8],
    allowance: usize,
) -> PyResult<pse_runtime::math::flows::ModelingFlowSelection> {
    let document: pse_runtime::math::flows::FlowSelectionDocument =
        documents::decode_document(py, "flow selection", bytes, allowance / 4)?;
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
    let inner = source_settlement::blocking_source(
        py,
        &runtime.owner,
        runtime.inner.scope_activity(async {
            let pool = runtime.owner.shared.pool();
            let token = cancel.token();
            let validation = runtime.inner.admission_context();
            let bundles = documents
                .iter()
                .map(|documents| {
                    pse_admission::authoring_driver::document::load_package_documents_owned(
                        documents,
                        runtime.owner.model.registry(),
                        pse_authoring::ParseBudget::default(),
                        &pool,
                        &token,
                        &validation,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let owned =
                pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                    bundles, &pool, &token,
                )?;
            runtime
                .inner
                .modeling_from_documents(&owned, physical.inner.clone(), &cancel)
                .await
        }),
        || cancel.cancel(),
    )?;
    Ok(NativeModelingPackage {
        owner: runtime.owner.clone(),
        inner,
        limits: Default::default(),
    })
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelingPackage {
    owner: Arc<runtime::Runtime>,
    pub(super) inner: native::ModelingPackage,
    limits: pse_modeling::Limits,
}
impl NativeModelingPackage {
    pub(super) fn from_revision(
        owner: Arc<runtime::Runtime>,
        inner: native::ModelingPackage,
    ) -> Self {
        Self {
            owner,
            inner,
            limits: Default::default(),
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
    fn revision_id(&self) -> String {
        self.inner.revision_id().to_owned()
    }
    #[getter]
    fn physical_context(&self) -> NativePhysicalContext {
        NativePhysicalContext { inner: self.inner.physical_context().clone() }
    }
    /// Exact immutable modeling and physical source identity for lazy study inputs.
    #[getter]
    fn operation_source(&self) -> documents::DocumentValue<native::OperationSource> {
        documents::DocumentValue(native::OperationSource::of(&self.inner))
    }
    #[getter]
    fn canonical_revision(&self) -> Option<String> {
        self.inner
            .canonical_revision()
            .map(|origin| origin.key.clone())
    }
    #[getter]
    fn canonical_problem(&self) -> Option<String> {
        self.inner
            .canonical_revision()
            .map(|origin| origin.problem.clone())
    }
    fn with_declarations(&self, py: Python<'_>, source: &[u8]) -> PyResult<Self> {
        let edit: native::DeclarationEdit = documents::decode_document(
            py,
            "declaration edit",
            source,
            self.owner.shared.budget().math.workspace_bytes / 2,
        )?;
        let inner = source_settlement::blocking_source(
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
        })
    }
    fn with_delta(&self, py: Python<'_>, source: &[u8]) -> PyResult<Self> {
        let delta: native::SourceDelta = documents::decode_document(
            py,
            "source delta",
            source,
            self.owner.shared.budget().math.workspace_bytes / 2,
        )?;
        let cancel = CancelSource::new();
        let inner = source_settlement::blocking_source(
            py,
            &self.owner,
            self.inner.with_delta(delta, &cancel),
            || cancel.cancel(),
        )?;
        Ok(Self {
            owner: self.owner.clone(),
            inner,
            limits: self.limits,
        })
    }
    fn reload_from_parent(
        &self,
        py: Python<'_>,
        documents: Vec<BTreeMap<String, DocumentContent>>,
    ) -> PyResult<Self> {
        let documents = documents
            .into_iter()
            .map(document_bytes)
            .collect::<Vec<_>>();
        let cancel = CancelSource::new();
        let inner = source_settlement::blocking_source(
            py,
            &self.owner,
            self.inner.reload_from_parent(&documents, &cancel),
            || cancel.cancel(),
        )?;
        Ok(Self {
            owner: self.owner.clone(),
            inner,
            limits: self.limits,
        })
    }
    fn with_fit_declarations(&self, py: Python<'_>, source: &[u8]) -> PyResult<Self> {
        if source.len() > self.owner.shared.budget().math.workspace_bytes / 2 {
            return Err(invalid(py, "fit source extent"));
        }
        let data: native::FitDeclarations = documents::decode_document(
            py,
            "operation",
            source,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let inner = source_settlement::blocking_source(
            py,
            &self.owner,
            self.inner.clone().with_fit_declarations(data),
            || {},
        )?;
        Ok(Self {
            owner: self.owner.clone(),
            inner,
            limits: self.limits,
        })
    }

    fn prepare_fit(
        &self,
        py: Python<'_>,
        fit_id: Identity<pse_model::generated::identities::FitId>,
        request: documents::DocumentInput<native::PhysicalFitRequest>,
    ) -> PyResult<NativePreparedOperation> {
        let request = request.0;
        let fit = fit_id.0;
        let cancel = CancelSource::new();
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let inner = blocking(
            py,
            &self.owner,
            self.inner.prepare_operation(
                &native::OperationSource::of(&self.inner),
                &preparation,
                native::RuntimeOperationRequest::Study {
                    operation: Box::new(native::OperationRequest::Fit(native::FitOperation {
                        fit,
                        settings: request,
                    })),
                    output: native::StudyOutputDemand::Full {},
                    admitted_horizon: None,
                },
                None,
                &cancel,
            ),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner,
        })
    }

    fn with_limits(&self, limits: &ModelingLimits) -> Self {
        // Workers apply default limits: a package with its own limits runs no durable study.
        Self {
            owner: self.owner.clone(),
            inner: self.inner.clone(),
            limits: limits.limits,
        }
    }

    #[pyo3(signature=(case_id, settings, policy))]
    fn explain_nonlinear(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        policy: &[u8],
    ) -> PyResult<NativeModelingNonlinearExplanation> {
        let settings = settings.0;
        let root = declaration(case_id);
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let cancel = CancelSource::new();
        let policy: native::ModelingNonlinearPolicy = documents::decode_document(
            py,
            "nonlinear explanation",
            policy,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .explain_case_request(root, &preparation, settings, policy, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeModelingNonlinearExplanation {
            inner: Arc::new(inner),
        })
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "PyO3 receiver/interpreter plus explicit source, samples, diagnostic policy, controls and original context are separate boundary meanings"
    )]
    #[pyo3(signature=(case_id, settings, diagnostics, samples, *, controls=None, origin=None))]
    fn diagnose_samples(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        diagnostics: &ModelingDiagnosticSettings,
        samples: physical_input::PhysicalInput,
        controls: Option<&[u8]>,
        origin: Option<documents::DocumentInput<native::DiagnosticOrigin>>,
    ) -> PyResult<NativeModelingDiagnosticSamples> {
        let settings = settings.0;
        let root = declaration(case_id);
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let cancel = CancelSource::new();
        let controls: native::DiagnosticSamplesControls = documents::controls(
            py,
            "diagnostic sample controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let samples = self
            .inner
            .snapshot_diagnostic_samples(samples.0.into_reader()?)
            .map_err(|error| errors::diagnostic(py, &error))?;
        let inner = blocking(
            py,
            &self.owner,
            self.inner.diagnose_samples_request(
                root,
                &preparation,
                settings,
                samples,
                diagnostics.policy.clone(),
                controls,
                origin.map(|value| value.0),
                &cancel,
            ),
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
        case_id: Identity<DeclarationId>,
        settings: &SimulationSettings,
    ) -> PyResult<NativePreparedOperation> {
        let root = declaration(case_id);
        let cancel = CancelSource::new();
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let inner = blocking(
            py,
            &self.owner,
            self.inner.prepare_operation(
                &native::OperationSource::of(&self.inner),
                &preparation,
                native::RuntimeOperationRequest::Study {
                    operation: Box::new(native::OperationRequest::Simulation(
                        native::SimulationOperation {
                            case: root,
                            settings: Some(settings.request.clone()),
                        },
                    )),
                    output: native::StudyOutputDemand::Full {},
                    admitted_horizon: None,
                },
                None,
                &cancel,
            ),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner,
        })
    }
    fn simulate(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: &SimulationSettings,
    ) -> PyResult<NativeModelingTrajectory> {
        let root = declaration(case_id);
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .simulate_case_request(root, &preparation, settings.request.clone(), &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeModelingTrajectory {
            inner: Arc::new(inner),
        })
    }
    fn diagnostic_source_map(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        maximum_bytes: usize,
        time_limit: f64,
    ) -> PyResult<documents::DocumentValue<native::ModelingSourceMap>> {
        let settings = settings.0;
        let root = declaration(case_id);
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let cancel = CancelSource::new();
        blocking(
            py,
            &self.owner,
            self.inner.source_map_request(
                root,
                &preparation,
                settings,
                maximum_bytes,
                time_limit,
                &cancel,
            ),
            || cancel.cancel(),
        )
        .map(documents::DocumentValue)
    }
    #[pyo3(signature=(case_id, settings, diagnostics, *, origin=None))]
    fn diagnose(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        diagnostics: &ModelingDiagnosticSettings,
        origin: Option<documents::DocumentInput<native::DiagnosticOrigin>>,
    ) -> PyResult<NativeModelingDiagnostics> {
        let settings = settings.0;
        let root = declaration(case_id);
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner.diagnose_case_request(
                root,
                &preparation,
                settings,
                diagnostics.policy.clone(),
                origin.map(|value| value.0),
                &cancel,
            ),
            || cancel.cancel(),
        )?;
        Ok(NativeModelingDiagnostics {
            inner: Arc::new(inner),
        })
    }
    #[pyo3(signature=(case_id, settings, *, controls=None, origin=None))]
    fn diagnose_linear(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        controls: Option<&[u8]>,
        origin: Option<documents::DocumentInput<native::DiagnosticOrigin>>,
    ) -> PyResult<NativeModelingNativeAnalysis> {
        let settings = settings.0;
        let controls: native::LinearDiagnosticControls = documents::controls(
            py,
            "linear diagnostic controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        #[cfg(not(feature = "solver-highs"))]
        {
            let _ = (case_id, settings, controls, origin);
            Err(invalid(py, "HiGHS capability is not installed"))
        }
        #[cfg(feature = "solver-highs")]
        {
            let root = declaration(case_id);
            let preparation = native::PreparationSettings::with_limits(self.limits);
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                self.inner.diagnose_linear_request(
                    root,
                    &preparation,
                    settings,
                    controls,
                    origin.map(|value| value.0),
                    &cancel,
                ),
                || cancel.cancel(),
            )?;
            Ok(NativeModelingNativeAnalysis {
                inner: Arc::new(inner),
            })
        }
    }
    #[pyo3(signature=(case_id, settings, *, controls=None, origin=None))]
    fn diagnose_jacobian(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        controls: Option<&[u8]>,
        origin: Option<documents::DocumentInput<native::DiagnosticOrigin>>,
    ) -> PyResult<NativeModelingNativeAnalysis> {
        let settings = settings.0;
        let controls: native::JacobianDiagnosticControls = documents::controls(
            py,
            "Jacobian diagnostic controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        #[cfg(not(feature = "solver-highs"))]
        {
            let _ = (case_id, settings, controls, origin);
            Err(invalid(py, "HiGHS capability is not installed"))
        }
        #[cfg(feature = "solver-highs")]
        {
            let root = declaration(case_id);
            let preparation = native::PreparationSettings::with_limits(self.limits);
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                self.inner.diagnose_jacobian_request(
                    root,
                    &preparation,
                    settings,
                    controls,
                    origin.map(|value| value.0),
                    &cancel,
                ),
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
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        overrides: Option<&[u8]>,
    ) -> PyResult<NativeModelingInitialization> {
        let settings = settings.0;
        let root = declaration(case_id);
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let cancel = CancelSource::new();
        let overrides: native::InitializationOverrides = documents::controls(
            py,
            "initialization",
            overrides,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .initialize_case_request(root, &preparation, settings, overrides, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeModelingInitialization {
            inner: Arc::new(inner),
        })
    }
    /// Admit local scientific intent without publication prerequisites.
    fn admit_study(&self, py: Python<'_>, request: &[u8]) -> PyResult<NativeLocalStudy> {
        let request = documents::decode_document::<native::StudyRequest>(
            py,
            "operation",
            request,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .admit_study(&request.inputs, &request.output, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeLocalStudy {
            inner,
            owner: self.owner.clone(),
        })
    }
    /// Execute using the owned admitted model and physical context.
    #[pyo3(signature=(local, *, controls=None))]
    fn study(
        &self,
        py: Python<'_>,
        local: &NativeLocalStudy,
        controls: Option<&[u8]>,
    ) -> PyResult<NativeStudyReport> {
        let native::StudyRunControls { maximum_points } = documents::controls(
            py,
            "study",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner.study(&local.inner, maximum_points, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeStudyReport {
            inner: Arc::new(inner),
            owner: local.owner.clone(),
        })
    }
    fn preparation_discovery(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::PreparationDiscoveryRequest>,
    ) -> PyResult<documents::DocumentValue<native::PreparationDiscoveryReport>> {
        let cancel = CancelSource::new();
        let report = blocking(
            py,
            &self.owner,
            self.inner.preparation_discovery(&request.0, &cancel),
            || cancel.cancel(),
        )?;
        Ok(documents::DocumentValue(report))
    }

    fn inspect(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
    ) -> PyResult<documents::DocumentValue<native::ModelingInspection>> {
        let settings = settings.0;
        let root = declaration(case_id);
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let cancel = CancelSource::new();
        let inspection = blocking(
            py,
            &self.owner,
            self.inner
                .inspect_case_request(root, &preparation, settings, &cancel),
            || cancel.cancel(),
        )?;
        Ok(documents::DocumentValue(inspection))
    }

    fn prepare_flow(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        selection: &[u8],
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
    ) -> PyResult<NativePreparedFlow> {
        let settings = settings.0;
        let root = declaration(case_id);
        let preparation = native::PreparationSettings::with_limits(self.limits);
        let cancel = CancelSource::new();
        let selection = flow_selection(
            py,
            selection,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .prepare_flow_request(root, &preparation, settings, selection, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedFlow {
            owner: self.owner.clone(),
            math: self.owner.shared.math().clone(),
            package: self.inner.clone(),
            inner,
        })
    }
    fn prepare_recycle(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        selection: &[u8],
        request: &[u8],
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
    ) -> PyResult<NativePreparedStrategy> {
        let settings = settings.0;
        #[cfg(feature = "native-solvers")]
        {
            let root = declaration(case_id);
            let preparation = native::PreparationSettings::with_limits(self.limits);
            let selection = flow_selection(
                py,
                selection,
                self.owner.shared.budget().math.workspace_bytes,
            )?;
            let request: native::RecycleRequest = documents::decode_document(
                py,
                "recycle",
                request,
                self.owner.shared.budget().math.workspace_bytes / 4,
            )?;
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                self.inner.prepare_recycle_request(
                    root,
                    &preparation,
                    settings,
                    selection,
                    request,
                    &cancel,
                ),
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
            Err(invalid(py, "recycle workflow is not linked"))
        }
    }
    fn prepare_block_initialization(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        stages: physical_input::PhysicalInput,
    ) -> PyResult<NativePreparedStrategy> {
        let settings = settings.0;
        #[cfg(feature = "native-solvers")]
        {
            let root = declaration(case_id);
            let preparation = native::PreparationSettings::with_limits(self.limits);
            let stages = self
                .inner
                .snapshot_initialization_stages(stages.0.into_reader()?)
                .map_err(|error| errors::diagnostic(py, &error))?;
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                self.inner.prepare_initialization_request(
                    root,
                    &preparation,
                    settings,
                    stages,
                    &cancel,
                ),
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
    fn declarations(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<DeclarationExport>> {
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
        Ok(documents::DocumentValue(DeclarationExport {
            document: native::DeclarationInventory {
                declarations: declarations.to_vec(),
            },
            _source: declarations,
            _owner: pse_columnar::AllocationLease::new(scratch),
        }))
    }
    /// Compact logical-object source comparison, with explicit source-only context handling.
    #[pyo3(signature=(other,*,source_only=false))]
    fn project_compare(
        &self,
        py: Python<'_>,
        other: &NativeModelingPackage,
        source_only: bool,
    ) -> PyResult<documents::DocumentValue<native::ProjectComparison>> {
        let cancel = CancelSource::new();
        let result = if source_only {
            blocking(
                py,
                &self.owner,
                self.inner.project_compare_source(&other.inner, &cancel),
                || cancel.cancel(),
            )?
        } else {
            blocking(
                py,
                &self.owner,
                self.inner.project_compare(&other.inner, &cancel),
                || cancel.cancel(),
            )?
        };
        Ok(documents::DocumentValue(result))
    }
    /// Selected source metadata and typed values; large data remains an exact identity.
    fn project_source_detail(
        &self,
        py: Python<'_>,
        logical: Vec<String>,
    ) -> PyResult<Vec<documents::DocumentValue<native::ProjectSourceDetail>>> {
        let cancel = CancelSource::new();
        let result = blocking(
            py,
            &self.owner,
            self.inner.project_source_detail(&logical, &cancel),
            || cancel.cancel(),
        )?;
        Ok(result.into_iter().map(documents::DocumentValue).collect())
    }
    /// Explicit complete source Arrow export, independent of routine preparation.
    fn source_tables(
        &self,
        py: Python<'_>,
    ) -> PyResult<BTreeMap<Identity<SemanticId>, inspection::TableStream>> {
        let tables = blocking(py, &self.owner, self.inner.source_tables(), || {})?;
        tables
            .into_iter()
            .map(|(relation, batch)| {
                inspection::TableStream::from_batch(batch)
                    .map(|stream| (Identity(relation), stream))
                    .map_err(|error| errors::diagnostic(py, &error))
            })
            .collect()
    }
    #[pyo3(signature=(owner_id=None, *, controls=None))]
    fn knowledge(
        &self,
        py: Python<'_>,
        owner_id: Option<Identity<DeclarationId>>,
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
        let selected = owner_id.map(declaration);
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
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        controls: Option<&[u8]>,
        preparation: Option<documents::DocumentInput<native::PreparationSettings>>,
    ) -> PyResult<NativeModelingConformance> {
        let preparation = preparation_settings(preparation, self.limits);
        let controls: native::ConformanceControls = documents::controls(
            py,
            "conformance controls",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let settings = settings.0;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .conform_request(&preparation, settings, controls, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeModelingConformance {
            inner: Arc::new(inner),
        })
    }
    #[pyo3(signature=(case_id, settings, *, preparation=None, covariance_source=None))]
    fn prepare_solve(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        preparation: Option<documents::DocumentInput<native::PreparationSettings>>,
        covariance_source: Option<&NativeRunResult>,
    ) -> PyResult<NativePreparedOperation> {
        let preparation = preparation_settings(preparation, self.limits);
        let mut settings = settings.0;
        self::covariance_source(py, &mut settings, covariance_source)?;
        let root = declaration(case_id);
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner.prepare_operation(
                &native::OperationSource::of(&self.inner),
                &preparation,
                native::RuntimeOperationRequest::DeclaredCase {
                    case: root,
                    settings: Box::new(settings),
                },
                None,
                &cancel,
            ),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner,
        })
    }
    /// Apply typed transient coordinates without exporting or editing source declarations.
    #[pyo3(signature=(case_id, overlay, settings, *, preparation=None, covariance_source=None))]
    fn solve_bound(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        overlay: &[u8],
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        preparation: Option<documents::DocumentInput<native::PreparationSettings>>,
        covariance_source: Option<&NativeRunResult>,
    ) -> PyResult<NativeModelingResult> {
        let allowance = self.owner.shared.budget().math.workspace_bytes;
        let preparation = preparation_settings(preparation, self.limits);
        let overlay: native::PointOverlay =
            documents::decode(py, "point overlay", overlay, allowance / 4)?;
        let mut settings = settings.0;
        self::covariance_source(py, &mut settings, covariance_source)?;
        let root = declaration(case_id);
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .execute_bound_request(root, &overlay, &preparation, settings, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeModelingResult {
            inner: Arc::new(inner),
        })
    }
    #[pyo3(signature=(case_id, settings, *, preparation=None, covariance_source=None))]
    fn solve_case(
        &self,
        py: Python<'_>,
        case_id: Identity<DeclarationId>,
        settings: documents::DocumentInput<pse_runtime::math::settings::SolveSettings>,
        preparation: Option<documents::DocumentInput<native::PreparationSettings>>,
        covariance_source: Option<&NativeRunResult>,
    ) -> PyResult<NativeModelingResult> {
        let preparation = preparation_settings(preparation, self.limits);
        let mut settings = settings.0;
        self::covariance_source(py, &mut settings, covariance_source)?;
        let root = declaration(case_id);
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .execute_case_request(root, &preparation, settings, &cancel),
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
    fn candidate_rows(&self) -> Vec<Identity<SemanticId>> {
        self.inner
            .candidate_rows
            .iter()
            .copied()
            .map(Identity)
            .collect()
    }
    fn background_variables(&self) -> Vec<Identity<SemanticId>> {
        self.inner
            .background_variables
            .iter()
            .copied()
            .map(Identity)
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
    fn omitted(&self) -> Vec<Identity<SemanticId>> {
        self.owner.attempts[self.index]
            .omitted
            .iter()
            .copied()
            .map(Identity)
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
        self.inner.report().coverage.produced()
    }
    fn failure(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .diagnostic()
            .as_ref()
            .map(inspection::DiagnosticReport::observe)
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.clone().into_table_cursor(name, 1024))
            .map(inspection::TableStream::from_result)
            .map_err(|error| errors::diagnostic(py, &error))
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
            documents::decode_document(py, "diagnostic profile", source.as_bytes(), 1 << 20)?;
        policy.validate().map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self { policy })
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        String::from_utf8(
            <native::ModelingDiagnosticPolicy as documents::DocumentType>::WIRE
                .to_vec(&self.policy)
                .map_err(|error| invalid(py, error.to_string()))?,
        )
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
    fn source_revision(&self) -> HashValue {
        HashValue(self.inner.source_revision().as_id())
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
    fn origin(&self) -> Option<documents::DocumentValue<native::DiagnosticOrigin>> {
        self.inner.origin.clone().map(documents::DocumentValue)
    }
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
            .map(|inner| NativeAttempt {
                inner,
                primal_coordinates: self.inner.primal_coordinates.get(index).cloned(),
            })
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
    #[getter]
    fn origin(&self) -> Option<documents::DocumentValue<native::DiagnosticOrigin>> {
        self.inner.origin.clone().map(documents::DocumentValue)
    }
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
    fn ids(&self) -> Vec<Identity<SemanticId>> {
        self.inner
            .outcomes
            .iter()
            .map(|(id, _)| Identity(*id))
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
    #[getter]
    fn origin(&self) -> Option<documents::DocumentValue<native::DiagnosticOrigin>> {
        self.inner.origin.clone().map(documents::DocumentValue)
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.clone().into_table_cursor(name, 1024))
            .map(inspection::TableStream::from_result)
            .map_err(|error| errors::diagnostic(py, &error))
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
    fn coordinates(&self) -> (Vec<Identity<SemanticId>>, Vec<Identity<SemanticId>>) {
        (
            self.inner.rows.iter().copied().map(Identity).collect(),
            self.inner.columns.iter().copied().map(Identity).collect(),
        )
    }
    fn singular_modes(&self) -> Vec<SingularModeTuple> {
        self.inner
            .matrix
            .iter()
            .flat_map(|m| &m.modes)
            .map(|m| {
                (
                    m.value,
                    m.left.clone(),
                    m.right.clone(),
                    m.left_residual,
                    m.right_residual,
                    m.normalization_error,
                    m.residual_bound,
                )
            })
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
    fn committed_values(&self) -> Option<BTreeMap<Identity<SemanticId>, f64>> {
        self.inner.committed.as_ref().map(|v| {
            v.iter()
                .map(|(id, value)| (Identity(*id), *value))
                .collect()
        })
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
/// Nonserializable retained scientific authority for a local study.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeLocalStudy {
    pub(super) inner: native::LocalStudy,
    pub(super) owner: Arc<runtime::Runtime>,
}
#[pymethods]
impl NativeLocalStudy {
    #[pyo3(signature=(*, controls=None))]
    fn execute(&self, py: Python<'_>, controls: Option<&[u8]>) -> PyResult<NativeStudyReport> {
        let native::StudyRunControls { maximum_points } = documents::controls(
            py,
            "study",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner.execute(maximum_points, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeStudyReport {
            inner: Arc::new(inner),
            owner: self.owner.clone(),
        })
    }
    #[getter]
    fn points(&self) -> Vec<documents::DocumentValue<&native::StudyPointDefinition>> {
        self.inner
            .points
            .iter()
            .map(documents::DocumentValue)
            .collect()
    }
    #[getter]
    fn variants(&self) -> Option<documents::DocumentValue<&native::LazyStudyDefinition>> {
        self.inner.variants.as_ref().map(documents::DocumentValue)
    }
    #[getter]
    fn modeling_revision(&self) -> String {
        self.inner.modeling_revision.clone()
    }
}

#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeStudyReport {
    pub(super) inner: Arc<native::StudyReport>,
    pub(super) owner: Arc<runtime::Runtime>,
}
#[pymethods]
impl NativeStudyReport {
    /// Explicit selected diagnostics at the actual recorded study's locator.
    fn diagnostic_selection(
        &self,
        py: Python<'_>,
    ) -> PyResult<Option<documents::DocumentValue<native::ManagedStudyDiagnosticsDocument>>> {
        blocking(py, &self.owner, self.inner.diagnostic_selection(), || {})
            .map(|document| document.map(documents::DocumentValue))
    }
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
    fn preparations(&self) -> documents::DocumentValue<&pse_runtime::math::PreparationCounts> {
        documents::DocumentValue(&self.inner.preparations)
    }
    #[getter]
    fn conclusion(&self) -> documents::DocumentValue<&native::StudyConclusion> {
        documents::DocumentValue(&self.inner.decision.conclusion)
    }
    fn outcome(
        &self,
        py: Python<'_>,
        index: usize,
    ) -> PyResult<documents::DocumentValue<&native::PointOutcome>> {
        let outcome = self
            .inner
            .outcomes
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        Ok(documents::DocumentValue(outcome))
    }
    fn is_compact(&self, py: Python<'_>, index: usize) -> PyResult<bool> {
        let result = self
            .inner
            .results
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        Ok(matches!(
            result,
            Some(native::StudyOccurrenceResult::Compact(_))
        ))
    }
    fn compact_completion(
        &self,
        py: Python<'_>,
        index: usize,
    ) -> PyResult<documents::DocumentValue<&native::Completion>> {
        let result = self
            .inner
            .results
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        let Some(native::StudyOccurrenceResult::Compact(result)) = result else {
            return Err(invalid(py, "occurrence did not select compact output"));
        };
        let completion = result
            .completion()
            .map_err(|error| errors::diagnostic(py, error))?;
        Ok(documents::DocumentValue(completion))
    }
    fn compact_table_names(&self, py: Python<'_>, index: usize) -> PyResult<Vec<String>> {
        let result = self
            .inner
            .results
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        let Some(native::StudyOccurrenceResult::Compact(result)) = result else {
            return Err(invalid(py, "occurrence did not select compact output"));
        };
        result
            .table_names()
            .map(|names| names.into_iter().map(str::to_owned).collect())
            .map_err(|error| errors::diagnostic(py, error))
    }
    fn compact_table(
        &self,
        py: Python<'_>,
        index: usize,
        name: &str,
    ) -> PyResult<inspection::TableStream> {
        let result = self
            .inner
            .results
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        let Some(native::StudyOccurrenceResult::Compact(result)) = result else {
            return Err(invalid(py, "occurrence did not select compact output"));
        };
        py.detach(|| result.clone().into_table_cursor(name))
            .map(inspection::TableStream::from_result)
            .map_err(|error| errors::diagnostic(py, error.as_ref()))
    }
    fn result(&self, py: Python<'_>, index: usize) -> PyResult<Option<NativeRunResult>> {
        let result = self
            .inner
            .results
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        match result {
            Some(native::StudyOccurrenceResult::Ephemeral(inner)) => Ok(Some(NativeRunResult {
                inner: inner.clone(),
                owner: self.owner.clone(),
            })),
            Some(native::StudyOccurrenceResult::Compact(_)) => Err(invalid(
                py,
                "compact occurrence uses original completion and selected detail APIs",
            )),
            Some(native::StudyOccurrenceResult::Retained { .. }) => Err(invalid(
                py,
                "retained occurrence uses its stored result handle",
            )),
            None => Ok(None),
        }
    }
    fn stored_result(&self, py: Python<'_>, index: usize) -> PyResult<Option<NativeStoredResult>> {
        let result = self
            .inner
            .results
            .get(index)
            .ok_or_else(|| invalid(py, "study occurrence outside report"))?;
        Ok(result.as_ref().and_then(|result| {
            result
                .stored_keys()
                .map(|(run, attempt)| NativeStoredResult {
                    inner: self.inner.runtime().clone(),
                    owner: self.owner.clone(),
                    run_id: result.run_id(),
                    run: run.into(),
                    attempt: attempt.into(),
                })
        }))
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
        preparation: Option<documents::DocumentInput<native::PreparationSettings>>,
    ) -> PyResult<Self> {
        let preparation = preparation_settings(
            preparation,
            limits.map_or_else(Default::default, |value| value.limits),
        );
        let controls: native::PureConformanceControls = documents::controls(
            py,
            "pure conformance controls",
            controls,
            settings.resource_budget().math.workspace_bytes,
        )?;
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
            native::conform_pure_request(
                documents.into_iter().map(document_bytes).collect(),
                document_bytes(physical),
                budget,
                controls,
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
    fn trajectory(
        &self,
        py: Python<'_>,
        fixture_id: Identity<DeclarationId>,
    ) -> PyResult<NativeModelingTrajectory> {
        self.inner
            .trajectories
            .get(&declaration(fixture_id))
            .map(|value| NativeModelingTrajectory {
                inner: Arc::new(value.clone()),
            })
            .ok_or_else(|| invalid(py, "fixture has no integrated trajectory"))
    }
    fn initialization(
        &self,
        py: Python<'_>,
        fixture_id: Identity<DeclarationId>,
    ) -> PyResult<NativeModelingInitialization> {
        self.inner
            .initializations
            .get(&declaration(fixture_id))
            .map(|value| NativeModelingInitialization {
                inner: Arc::new(value.clone()),
            })
            .ok_or_else(|| invalid(py, "fixture has no initialization report"))
    }
    fn fixtures(&self) -> Vec<Identity<DeclarationId>> {
        self.inner
            .fixtures()
            .iter()
            .copied()
            .map(Identity)
            .collect()
    }
    fn fixture_statuses(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.fixture_statuses_table())
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, &e))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn result(
        &self,
        py: Python<'_>,
        fixture_id: Identity<DeclarationId>,
    ) -> PyResult<NativeModelingResult> {
        let identity = declaration(fixture_id);
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
    /// Exact selected parent/context and submitted attribution for a transient solve.
    fn bound_source(&self) -> Option<documents::DocumentValue<&native::AdmittedBinding>> {
        self.inner
            .prepared
            .bound_source
            .as_ref()
            .map(|binding| documents::DocumentValue(&***binding))
    }
    #[getter]
    fn binding_identity(&self) -> Option<HashValue> {
        self.inner
            .prepared
            .bound_source
            .as_ref()
            .map(|binding| HashValue(binding.identity().as_id()))
    }
    #[getter]
    fn accepted(&self) -> bool {
        self.inner.accepted
    }
    #[getter]
    fn run_id(&self) -> Identity<pse_model::generated::identities::RunId> {
        Identity(self.inner.run_id)
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
    fn attempt(&self, py: Python<'_>) -> PyResult<Option<NativeAttempt>> {
        match &self.inner.outcome {
            pse_runtime::math::solves::Outcome::Native(report) => Ok(Some(NativeAttempt {
                inner: report.as_ref().clone(),
                primal_coordinates: Some(
                    self.inner
                        .prepared
                        .primal_coordinates()
                        .map_err(|error| errors::diagnostic(py, &error))?,
                ),
            })),
            _ => Ok(None),
        }
    }
    fn failure(&self) -> Option<inspection::DiagnosticReport> {
        self.inner
            .diagnostic()
            .as_ref()
            .map(inspection::DiagnosticReport::observe)
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.clone().into_table_cursor(name, 1024))
            .map(inspection::TableStream::from_result)
            .map_err(|error| errors::diagnostic(py, &error))
    }
}
