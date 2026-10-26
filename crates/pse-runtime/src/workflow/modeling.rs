// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public finite modeling packages are immutable inputs to the existing compiler service.
mod bound;
mod fit_declarations;
pub use fit_declarations::{FitDeclaration, FitDeclarations};
mod canonical;
pub(crate) use canonical::reload::RetainedIngress;
pub(crate) use canonical::{AdmittedModelRevision, LocalSourcePremises};
pub(in crate::workflow) mod cases;
mod conformance;
pub(super) mod declared;
pub(crate) mod engagement;
mod path_pipeline;
mod preparation;
pub use declared::{DeclaredExecution, DeclaredProcedure, InitializationOverrides};
mod knowledge;
mod pure;
pub use conformance::{
    ModelingConformanceCheck, ModelingConformancePolicy, ModelingConformanceReport,
    ModelingFixtureSelection,
};
pub use knowledge::ModelingKnowledge;
pub use pure::conform_pure_documents;
pub(super) mod dynamics;
#[cfg(feature = "solver-idas")]
pub use dynamics::consistent::ConsistentInitializationResult;
pub use dynamics::{
    ModelingSimulation, ModelingTrajectory, SimulationSampleAttempt, SimulationSampleObserver,
    with_simulation_sample_wait,
};
#[cfg(test)]
mod certificate_tests;
#[cfg(test)]
#[cfg(all(
    feature = "solver-scip",
    feature = "solver-ipopt",
    feature = "solver-highs"
))]
mod commitment_tests;
#[cfg(test)]
#[cfg(feature = "solver-highs")]
mod convexity_tests;
mod diagnostics;
#[cfg(test)]
#[cfg(feature = "solver-highs")]
mod forms_tests;
#[cfg(test)]
#[cfg(all(feature = "solver-scip", feature = "solver-ipopt"))]
mod global_tests;
mod implicit;
#[cfg(test)]
#[cfg(all(
    feature = "solver-ipopt",
    feature = "solver-highs",
    feature = "solver-pounce"
))]
mod pounce_convex_tests;
#[cfg(all(test, feature = "solver-kinsol", feature = "solver-root-isolation"))]
mod pr_jacobian_tests;
#[cfg(all(test, feature = "solver-kinsol", feature = "solver-root-isolation"))]
mod recycle_tests;
#[cfg(all(test, feature = "solver-kinsol"))]
mod root_response_tests;
#[cfg(test)]
#[cfg(all(feature = "solver-ipopt", feature = "solver-highs"))]
pub(in crate::workflow) mod sensitivity_tests;
#[cfg(all(test, feature = "solver-pounce"))]
mod work_admission_tests;
pub use diagnostics::{
    DiagnosticSampleStop, ElasticObservation, ModelingDiagnosticPolicy,
    ModelingDiagnosticPreparation, ModelingDiagnosticSamples, ModelingDiagnostics,
    ModelingElasticAttempt, ModelingInfeasibilityCertificate, ModelingNonlinearExplanation,
    ModelingNonlinearPolicy, ModelingSourceMap, ModelingSourceMapDocument,
};
#[cfg(feature = "solver-highs")]
pub use diagnostics::{ModelingJacobianOptimization, ModelingLinearDiagnostics};
mod engines;
#[cfg(feature = "solver-petsc")]
mod petsc_pipeline;
pub use engines::{
    DiscreteInitialization, ModelingAnalysis, ModelingInitialization,
    ModelingInitializationAttempt, ModelingInitializationReport, ModelingInitializationStep,
};
pub(super) mod analysis_tables;
pub(super) mod assessment;
pub(super) mod documents;
mod project_comparison;
mod project_preparation;
pub(super) mod results;
mod views;
use super::{PhysicalContext, Runtime, SourceDelta, WorkflowError, contract, relation};
use crate::math::{Workspace, modeling::ModelingPreparation};
pub use analysis_tables::ModelingNativeAnalysis;
pub use cases::{ModelingObservations, ModelingSolvePreparation, StartSource};
pub use project_comparison::{
    ModelingComparisonObservation, ModelingComparisonRequest, ProjectChangeImpact,
    ProjectComparison, ProjectComparisonSelection, ProjectDocumentComposition,
    ProjectDocumentSource, ProjectMergeChoice, ProjectMergeConflict, ProjectMergeConflictKind,
    ProjectMergeIntent, ProjectMergeMode, ProjectMergePreview, ProjectMergeProposal,
    ProjectMergeResolution, ProjectObjectChange, ProjectSourceDetail, ProjectSourceObject,
};
pub use project_preparation::{
    PreparationDiscoveryReport, PreparationDiscoveryRequest, PreparationEligibility,
    PreparationProductKind, ResidentPreparation,
};
use pse_authoring::language::Declaration;
use pse_compiler::workspace::{CompilerContext, WorkspaceLimits};
use pse_ids::SemanticId;
use pse_modeling::{Bindings, DeclarationId, InstanceId, Limits, PhysicalScope};
use pse_relations::columnar::RelationRow;
pub use results::{ModelingCheck, ModelingReport, ModelingResult};
#[cfg(test)]
use std::sync::Mutex;
use std::{collections::BTreeMap, sync::Arc};

#[cfg(feature = "solver-kinsol")]
enum InitializationStageValues<'a> {
    Physical(&'a super::InitializationStages),
    #[cfg(test)]
    Canonical(Vec<BTreeMap<SemanticId, f64>>),
}

#[cfg(test)]
fn source_map_extent(tables: usize) -> Result<usize, WorkflowError> {
    tables
        .checked_mul(
            size_of::<pse_relations::columnar::FieldCheckedBatch>() + size_of::<SemanticId>() + 128,
        )
        .and_then(|n| {
            n.checked_add(
                size_of::<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>>() + 128,
            )
        })
        .ok_or_else(|| contract("source export map extent"))
}
/// Immutable source revision sharing one deployment-owned compiler workspace.
#[derive(Clone, Debug)]
pub struct ModelingPackage {
    /// Exact authenticated research inputs consumed by ordinary execution.
    pub(in crate::workflow) research_source: Option<super::ResearchSource>,
    /// Frozen historical engineering selection, independent of scientific content identity.
    pub(in crate::workflow) project_context:
        Option<pse_operations::canonical_contexts::ContextSelection>,
    /// Escaped positional replay metadata retains its receiving allocation owner.
    pub(in crate::workflow) replay_owner: Option<Arc<pse_columnar::AllocationLease>>,
    /// Protection/admission lease over exact immutable views and allocations; no execution services.
    /// Preparation and joined effects still receive their explicit caller capabilities.
    pub(in crate::workflow) held_preparation: Option<Arc<engagement::HeldPreparationScope>>,

    pub(in crate::workflow) revision: canonical::SourceRevision,
    accelerators: Arc<pse_math::implicit::accelerators::Accelerators>,
    providers: Arc<BTreeMap<String, pse_kernels::Registration>>,
    pub(in crate::workflow) physical: PhysicalContext,
    pub(in crate::workflow) quantities: Arc<pse_quantity::QuantityRegistry>,
}
/// Process-local request context; scientific eligibility is independently rechecked.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SelectedRequest {
    roots: Vec<DeclarationId>,
    physical: pse_ids::ContentHash,
    physical_owners: (usize, usize),
    providers: Vec<(String, pse_kernels::ProviderKey, pse_ids::ContentHash)>,
    validation: usize,
    registry: usize,
}
impl SelectedRequest {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.roots.capacity() * size_of::<DeclarationId>()
            + self.providers.capacity()
                * size_of::<(String, pse_kernels::ProviderKey, pse_ids::ContentHash)>()
            + self
                .providers
                .iter()
                .map(|(name, _, _)| name.capacity())
                .sum::<usize>()
            + 128
    }
}
/// Immutable checked payload only: no protected read, attempt or compiler workspace.
#[derive(Debug)]
pub(crate) struct SelectedAdmission {
    pub(crate) request: Arc<SelectedRequest>,
    pub(crate) revision: crate::math::modeling::ModelingRevision,
    pub(crate) dependencies: Arc<LocalSourcePremises>,
    pub(crate) versions: Arc<BTreeMap<String, String>>,
    pub(crate) metadata: Arc<pse_columnar::AllocationLease>,
    pub(crate) _validation_owner: Arc<pse_relations::validate::ValidationContext>,
}
impl SelectedAdmission {
    pub(crate) fn retained_bytes(&self) -> usize {
        self.revision.retained_bytes()
            + self.dependencies.retained_bytes()
            + self.request.retained_bytes()
            + size_of::<Self>()
            + 256
            + self
                .versions
                .iter()
                .map(|(k, v)| k.capacity() + v.capacity() + 128)
                .sum::<usize>()
    }
}
/// Explicit immutable source inventory, retaining deployment memory charges
/// until the generated declaration rows are no longer held by the caller.
#[derive(Debug)]
pub struct OwnedDeclarations {
    rows: Vec<Declaration>,
    _leases: Vec<Arc<pse_columnar::AllocationLease>>,
}
impl std::ops::Deref for OwnedDeclarations {
    type Target = [Declaration];
    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

/// Immutable admitted package payload; excludes runtime services and mutable workspaces.
#[derive(Clone, Debug)]
pub(crate) struct PackageAdmission {
    revision: canonical::SourceRevision,
    accelerators: Arc<pse_math::implicit::accelerators::Accelerators>,
    providers: Arc<BTreeMap<String, pse_kernels::Registration>>,
    _lease: Arc<pse_columnar::AllocationLease>,
    _validation_owner: Arc<pse_relations::validate::ValidationContext>,
}
impl PackageAdmission {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.revision.revision_id().len()
            + self.revision.admitted.retained_bytes()
            + 256
    }
}
impl ModelingPackage {
    /// Receiving services must retain the actual registry/native validation bindings
    /// that admitted this source. Independent caller scopes over those bindings are valid.
    pub(in crate::workflow) fn check_service_bindings(
        &self,
        services: &crate::workflow::PreparationServices,
    ) -> Result<(), crate::math::MathRuntimeError> {
        if !self
            .revision
            .admitted
            .validation_context()
            .same_bindings(&services.admission_context())
        {
            return Err(pse_backend_native::ProblemError::Contract(
                "scientific source requires its original registry and native validation bindings"
                    .into(),
            )
            .into());
        }
        Ok(())
    }

    /// Original admitted physical meaning, reusable by ordinary draft and study operations.
    pub fn physical_context(&self) -> &PhysicalContext {
        &self.physical
    }

    pub(crate) fn admission(
        &self,
        services: &crate::workflow::PreparationServices,
    ) -> Result<PackageAdmission, WorkflowError> {
        self.check_service_bindings(services)?;
        Ok(PackageAdmission {
            revision: self.revision.clone(),
            accelerators: self.accelerators.clone(),
            providers: self.providers.clone(),
            _lease: services.math().reserve(
                "modeling:package-admission",
                size_of::<PackageAdmission>()
                    + 2 * size_of::<usize>()
                    + 256
                    + self
                        .providers
                        .keys()
                        .map(|name| name.capacity() + size_of::<pse_kernels::Registration>() + 128)
                        .sum::<usize>(),
            )?,
            _validation_owner: services.validation_context(),
        })
    }
}
impl crate::workflow::SourceServices {
    pub(crate) fn package_from_admission(
        &self,
        admitted: Arc<PackageAdmission>,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        Ok(ModelingPackage {
            research_source: None,
            project_context: None,
            replay_owner: None,
            held_preparation: None,

            revision: admitted.revision.clone(),
            accelerators: admitted.accelerators.clone(),
            providers: admitted.providers.clone(),
            quantities: physical.quantities.clone(),
            physical,
        })
    }
}
fn compiler_context(
    physical: &PhysicalContext,
    providers: &BTreeMap<String, pse_kernels::Registration>,
) -> CompilerContext {
    CompilerContext {
        quantities: physical.quantities.clone(),
        preconditions: physical.preconditions.clone(),

        providers: providers
            .iter()
            .map(|(name, p)| {
                (
                    name.clone(),
                    pse_compiler::typed_math::ProviderCall {
                        descriptor: p.descriptor(),
                        output: 0,
                    },
                )
            })
            .collect(),
    }
}
/// Typed declarations, physical scope, fit data, source context, data inventory
/// and ownership of the compiler-input copy (ADR-0125).
type DocumentCompilerContext = (
    Vec<Declaration>,
    PhysicalScope,
    FitDeclarations,
    pse_admission::authoring_driver::document::Batches,
    Arc<pse_modeling::document::DocumentInventory>,
    Arc<pse_columnar::AllocationLease>,
);
/// The package data documents of a closure and the package of each text document, which a
/// dataset's document path resolves in (ADR-0125).
fn data_documents(
    bundles: &[pse_admission::authoring_driver::document::DocumentBundle],
) -> pse_modeling::document::DocumentInventory {
    let mut inventory = pse_modeling::document::DocumentInventory::default();
    for bundle in bundles {
        for document in &bundle.documents {
            for (path, span) in document.spans.iter() {
                if let Some(field) = path.strip_prefix("/modeling-fields/")
                    && let Some((declaration, role)) = field.split_once('/')
                    && let Ok(declaration) = SemanticId::parse_hex(declaration)
                {
                    inventory
                        .field_spans
                        .entry(declaration.into())
                        .or_default()
                        .insert(role.into(), span);
                }
            }

            match document.data() {
                Some(data) => {
                    inventory.documents.insert(document.id, Arc::clone(data));
                }
                None => {
                    inventory
                        .packages
                        .insert(document.id, bundle.package.package_id.as_id());
                }
            }
        }
    }
    inventory
}
fn document_inputs(
    documents: &pse_admission::authoring_driver::document::OwnedDocumentSet,
    registry: &pse_schema::Registry,
    physical: &PhysicalContext,
    workspace_bytes: usize,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<DocumentCompilerContext, WorkflowError> {
    documents.validate_registry(registry)?;
    document_inputs_bundles(
        documents.bundles(),
        registry,
        physical,
        workspace_bytes,
        pool,
        cancel,
    )
}
fn document_inputs_bundles(
    bundles: &[pse_admission::authoring_driver::document::DocumentBundle],
    registry: &pse_schema::Registry,
    physical: &PhysicalContext,
    workspace_bytes: usize,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<DocumentCompilerContext, WorkflowError> {
    use pse_model::HeapUsage;
    cancel
        .checkpoint()
        .map_err(pse_relations::RelationError::from)
        .map_err(relation)?;
    let (count, bytes) = bundles
        .iter()
        .flat_map(|bundle| &bundle.documents)
        .filter_map(|document| document.modeling_rows())
        .flat_map(|rows| rows.iter())
        .try_fold(
            (0usize, size_of::<Vec<Declaration>>()),
            |(count, n), row| Some((count.checked_add(1)?, n.checked_add(row.owned_bytes())?)),
        )
        .ok_or_else(|| contract("typed document input extent"))?;
    let allocation =
        pse_columnar::MemoryConsumer::new("modeling:typed-document-inputs").register(pool);
    allocation
        .try_grow(bytes)
        .map_err(pse_columnar::CanonError::from)
        .map_err(pse_relations::RelationError::from)
        .map_err(relation)?;
    let lease = pse_columnar::AllocationLease::new(allocation);
    let mut headers = bundles
        .iter()
        .map(|b| b.package.clone())
        .collect::<Vec<_>>();
    cancel
        .checkpoint()
        .map_err(pse_relations::RelationError::from)
        .map_err(relation)?;
    // A manifest may depend on the package the physical context was admitted from without
    // repeating its documents (ADR-0123 Outcome 6).
    if let Some(declaring) = &physical.package
        && !headers.iter().any(|h| h.package_id.as_id() == declaring.id)
    {
        headers.push(declaring.header.clone());
    }
    let limits = pse_authoring::p0::GraphLimits {
        nodes: headers.len(),
        edges: workspace_bytes / 128,
    };
    let batches = pse_admission::authoring_driver::p1::source_context_batches(
        bundles, registry, &headers, limits,
    )?;
    let mut rows = Vec::with_capacity(count);
    rows.extend(
        bundles
            .iter()
            .flat_map(|bundle| &bundle.documents)
            .filter_map(|document| document.modeling_rows())
            .flat_map(|rows| rows.iter().cloned()),
    );
    validate_import_versions(&rows, bundles)?;
    let scope = physical_scope(bundles, physical);
    let context = [
        pse_relations::generated::authored::packages::RELATION_ID,
        pse_relations::generated::authored::documents::RELATION_ID,
    ]
    .into_iter()
    .filter_map(|id| batches.get(&id).map(|batch| (id, batch.clone())))
    .collect();
    Ok((
        rows,
        scope,
        FitDeclarations::from_batches(&batches)?,
        context,
        Arc::new(data_documents(bundles)),
        lease,
    ))
}
impl crate::workflow::SourceServices {
    /// Persist native-admitted authored source once; later preparations select immutable records.
    pub async fn modeling_from_documents(
        &self,
        documents: &pse_admission::authoring_driver::document::OwnedDocumentSet,
        physical: PhysicalContext,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        let activity = self.preparation().admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                let token = cancel.token();
                token.checkpoint().map_err(pse_engine::EngineError::from)?;
                let validation = self.preparation().admission_context();
                documents.validate_context(
                    self.preparation().registry(),
                    &validation,
                    &self.preparation().pool(),
                    &token,
                )?;
                let (rows, scope, fits, sources, inventory, _input_lease) = document_inputs(
                    documents,
                    self.preparation().registry(),
                    &physical,
                    self.preparation().math().policy().workspace_bytes,
                    &self.preparation().pool(),
                    &token,
                )?;
                token.checkpoint().map_err(pse_engine::EngineError::from)?;
                // Pure validation retains the caller's cancellation and clock.
                // Once publication is issued, finish its canonical effect owner.
                let package = self
                    .persist_modeling(
                        rows,
                        physical,
                        scope,
                        inventory,
                        sources,
                        fits,
                        BTreeMap::new(),
                        Some(documents),
                        cancel,
                    )
                    .await?;
                let _ = canonical::reload::retain_initial(self.preparation(), &package, documents);
                // An acknowledged source effect is the outcome even if cancellation
                // arrived during publication. The Python recovery callback retains it.
                Ok(package)
            })
            .await
    }
    /// Fork one already admitted exact source closure into its own writable source problem.
    pub async fn project_fork(
        &self,
        project: &pse_operations::canonical_projects::Project,
        branch: &str,
        problem: &str,
        operation: &str,
        source: &ModelingPackage,
    ) -> Result<pse_operations::canonical::Revision, WorkflowError> {
        let activity = self.preparation().admit_activity()?;
        activity
            .scope(async {
                let store = self.canonical_store()?;
                let origin = source.recorded_revision()?;
                if source.source_issuer() != Some(store.issuing_store()?) {
                    return Err(contract(
                        "project fork issuer differs from its original admitted source",
                    ));
                }
                // Authenticate this runtime's exact source before trusting local payload/edge copies.
                let actual = store
                    .revision(&origin.key)
                    .await?
                    .ok_or_else(|| contract("project fork source unavailable"))?;
                if actual != *origin {
                    return Err(contract("project fork source authority differs"));
                }
                let protection = store
                    .protect(actual.clone(), std::time::Duration::from_secs(3600))
                    .await?;
                // Recover the original membership generation from this immutable fork receipt,
                // rather than substitute a later project observation on replay.
                let mut original_project = project.clone();
                if let Some(ack) = store.project_acknowledgement(operation).await? {
                    if ack.kind != "fork"
                        || ack.project != project.key
                        || ack.branch.as_deref() != Some(branch)
                    {
                        return Err(
                            pse_operations::canonical::CanonicalError::OperationReused.into()
                        );
                    }
                    original_project.generation = ack
                        .generation
                        .checked_sub(1)
                        .ok_or_else(|| contract("fork receipt generation invalid"))?;
                }
                let intent = store.project_fork_intent(
                    &original_project,
                    branch,
                    problem,
                    &actual,
                    operation,
                )?;
                let edits = source
                    .revision
                    .admitted
                    .objects
                    .values()
                    .map(|object| object.edit())
                    .collect::<Vec<_>>();
                let result = store.edit_project(&intent, &edits).await;
                let release = protection.finish().await;
                let revision = result?;
                release?;
                Ok(revision)
            })
            .await
    }
    /// Persist registry-generated rows without serializing through a text document.
    pub async fn modeling_package(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        let activity = self.preparation().admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                self.modeling_package_registered(rows, physical, BTreeMap::new())
                    .await
            })
            .await
    }
    /// Native capabilities remain explicitly supplied services; source names load no code.
    pub async fn modeling_package_registered(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        let activity = self.preparation().admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                let scope = PhysicalScope {
                    package: physical.package.as_ref().map(|p| p.name.clone()),
                    documents: None,
                };
                self.persist_modeling(
                    rows,
                    physical,
                    scope,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    providers,
                    None,
                    &crate::CancelSource::new(),
                )
                .await
            })
            .await
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one authored ingress persists declarations with their physical scope, document provenance, source leases, fit declarations and provider registrations"
    )]
    async fn persist_modeling(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
        scope: PhysicalScope,
        documents: Arc<pse_modeling::document::DocumentInventory>,
        sources: pse_admission::authoring_driver::document::Batches,
        fits: FitDeclarations,
        providers: BTreeMap<String, pse_kernels::Registration>,
        loader: Option<&pse_admission::authoring_driver::document::OwnedDocumentSet>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        let revision = canonical::persist(
            self,
            canonical::CanonicalIngress {
                rows,
                physical: &physical,
                physical_scope: scope,
                documents: &documents,
                sources: &sources,
                fits: &fits,
                loader,
            },
            None,
            cancel,
        )
        .await?;
        self.package_from_revision(revision, physical, providers)
    }
    fn package_from_revision(
        &self,
        revision: canonical::SourceRevision,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        Ok(ModelingPackage {
            research_source: None,
            project_context: None,
            replay_owner: None,
            held_preparation: None,

            revision,
            accelerators: Arc::new(pse_math::implicit::accelerators::Accelerators::standard()),
            providers: Arc::new(providers),
            quantities: physical.quantities.clone(),
            physical,
        })
    }
    /// Reopen an exact canonical revision after restart; no authored text or package hydration.
    pub async fn modeling_revision(
        &self,
        revision: pse_model::generated::runtime::canonical_revisions::Row,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        let activity = self.preparation().admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                let source = canonical::SourceRevision::open(self, revision).await?;
                let package = self.package_from_revision(source, physical, providers)?;
                package
                    .delta_parent(self.preparation(), &crate::CancelSource::new())
                    .await?;
                Ok(package)
            })
            .await
    }
}
/// Which documents see the physical names (ADR-0123 Outcome 6): those of packages whose
/// manifest depends on, or is, the package that declared them. A physical context admitted
/// without documents names no declaring package, so every row sees its names.
fn physical_scope(
    bundles: &[pse_admission::authoring_driver::document::DocumentBundle],
    physical: &PhysicalContext,
) -> PhysicalScope {
    let Some(declaring) = &physical.package else {
        return PhysicalScope::default();
    };
    PhysicalScope {
        package: Some(declaring.name.clone()),
        documents: Some(
            bundles
                .iter()
                .filter(|bundle| {
                    bundle.package.package_id.as_id() == declaring.id
                        || bundle
                            .package
                            .dependencies
                            .iter()
                            .any(|dependency| dependency.package_id.as_id() == declaring.id)
                })
                .flat_map(|bundle| bundle.documents.iter().map(|document| document.id))
                .collect(),
        ),
    }
}
fn validate_import_versions(
    rows: &[Declaration],
    bundles: &[pse_admission::authoring_driver::document::DocumentBundle],
) -> Result<(), WorkflowError> {
    let owners = bundles
        .iter()
        .flat_map(|bundle| {
            bundle
                .documents
                .iter()
                .map(move |doc| (doc.id, &bundle.package))
        })
        .collect::<BTreeMap<_, _>>();
    let packages = rows
        .iter()
        .filter(|row| {
            row.value.kind == pse_model::generated::enums::ModelingDeclarationKind::Package
        })
        .map(|row| (row.name.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for row in rows {
        if let Some(import) = &row.value.import {
            let target = packages
                .get(row.name.as_str())
                .ok_or_else(|| contract("imported modeling package absent"))?;
            let source = owners
                .get(&row.document_id)
                .ok_or_else(|| contract("import source document absent"))?;
            let target = owners
                .get(&target.document_id)
                .ok_or_else(|| contract("import target document absent"))?;
            // ADR-0123 Outcome 7: the import's typed requirement admits the target's
            // version, and the importing manifest depends on the target by identity with a
            // requirement that admits it too.
            if !pse_authoring::language::requirement_admits(&import.version, &target.version) {
                return Err(contract(
                    "modeling import differs from admitted package version",
                ));
            }
            if source.package_id != target.package_id
                && !source.dependencies.iter().any(|dependency| {
                    dependency.package_id == target.package_id
                        && pse_authoring::language::requirement_admits(
                            &dependency.version_req,
                            &target.version,
                        )
                })
            {
                return Err(contract(
                    "modeling import requires an exact manifest dependency",
                ));
            }
        }
    }
    Ok(())
}
impl ModelingPackage {
    #[cfg(all(test, feature = "native-solvers"))]
    pub(in crate::workflow) fn distinct_provider_owner_for_test(&self) -> Self {
        let mut distinct = self.clone();
        distinct.providers = Arc::new((*self.providers).clone());
        distinct
    }

    /// Select the immutable capability inventory for subsequent attempts. Existing
    /// attempts retain their admitted factories; source strings never load code.
    pub fn with_accelerators(
        mut self,
        accelerators: pse_math::implicit::accelerators::Accelerators,
    ) -> Self {
        self.accelerators = Arc::new(accelerators);
        self
    }

    /// Attempt-local numerical compilation from already admitted preparations.
    /// Immutable mathematical artifacts remain shared through the service cache.
    pub(in crate::workflow) fn numerical_workspace(
        &self,
        services: &crate::workflow::PreparationServices,
    ) -> Result<Workspace, crate::math::MathRuntimeError> {
        self.check_service_bindings(services)?;
        services.math().workspace(
            compiler_context(&self.physical, &self.providers),
            WorkspaceLimits::default(),
        )
    }

    /// Explicit full source inventory; ordinary preparation reads only its dependency closure.
    pub async fn declarations(
        &self,
        services: &crate::workflow::PreparationServices,
    ) -> Result<OwnedDeclarations, WorkflowError> {
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                self.declaration_inventory(services).await
            })
            .await
    }
    /// Immutable issuer of the original admitted canonical source, independent of selected services.
    pub fn source_issuer(&self) -> Option<&str> {
        self.revision.issuer.as_deref()
    }

    /// Exact immutable canonical revision, independent of semantic compiler source hashes.
    pub fn canonical_revision(
        &self,
    ) -> Option<&pse_model::generated::runtime::canonical_revisions::Row> {
        self.revision.origin()
    }
    /// Opaque exact parent token, distinct from semantic content and durable origin.
    pub fn revision_id(&self) -> &str {
        self.revision.revision_id()
    }
    pub(crate) fn recorded_revision(
        &self,
    ) -> Result<&pse_model::generated::runtime::canonical_revisions::Row, WorkflowError> {
        self.canonical_revision()
            .ok_or(WorkflowError::RecordingUnavailable)
    }
    pub(crate) fn source_attribution(&self) -> &str {
        self.canonical_revision()
            .map_or_else(|| self.revision_id(), |origin| origin.key.as_str())
    }
    /// Generate full source tables only for an explicit export/publication request.
    pub async fn source_tables(
        &self,
        services: &crate::workflow::PreparationServices,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                self.encode_source_tables(services).await
            })
            .await
    }
    async fn encode_source_tables(
        &self,
        services: &crate::workflow::PreparationServices,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        use pse_model::HeapUsage;
        use pse_relations::generated::authored::modeling_declarations as wire;
        let (source_rows, _, _, document_sources, _source_owners) =
            self.full_source(services).await?;
        let registry = services.registry();
        let validation = services.admission_context();
        let bytes = source_rows
            .iter()
            .map(HeapUsage::owned_bytes)
            .try_fold(4096usize, usize::checked_add)
            .and_then(|n| {
                self.physical
                    .sources
                    .values()
                    .chain(document_sources.values())
                    .try_fold(n, |n, b| {
                        n.checked_add(b.batch().get_array_memory_size().checked_mul(8)?)
                    })
            })
            .ok_or_else(|| contract("source export extent"))?;
        let _scratch = services.math().reserve("modeling:source-export", bytes)?;
        let mut tables = document_sources.clone();
        let mut physical = Vec::new();
        macro_rules! merge_context {
            ($module:ident, $key:expr, $batch:expr) => {{
                use pse_relations::generated::authored::$module as context;
                let mut rows = BTreeMap::new();
                for source in document_sources
                    .get(&context::RELATION_ID)
                    .into_iter()
                    .chain(std::iter::once($batch))
                {
                    for row in context::Row::rows(source).map_err(relation)? {
                        let key = ($key)(&row);
                        if let Some(old) = rows.insert(key, row.clone())
                            && old != row
                        {
                            return Err(contract(
                                "conflicting physical and modeling package context",
                            ));
                        }
                    }
                }
                let mut builder =
                    context::Builder::with_registry(registry, rows.len(), &validation)
                        .map_err(relation)?;
                for row in rows.into_values() {
                    builder.push(row).map_err(relation)?;
                }
                tables.insert(context::RELATION_ID, builder.finish().map_err(relation)?);
            }};
        }
        for (key, batch) in &self.physical.sources {
            let id = registry
                .relation(&key.qualified_name())
                .ok_or_else(|| contract("physical source relation absent"))?
                .id;
            if id == wire::RELATION_ID {
                physical = wire::Row::rows(batch).map_err(relation)?;
            } else if key.qualified_name() == "authored.documents" {
                merge_context!(
                    documents,
                    |r: &pse_relations::generated::authored::documents::Row| r.document_id,
                    batch
                );
            } else if key.qualified_name() == "authored.packages" {
                merge_context!(
                    packages,
                    |r: &pse_relations::generated::authored::packages::Row| r.package_id,
                    batch
                );
            } else {
                tables.insert(id, batch.clone());
            }
        }
        let mut rows = BTreeMap::new();
        for row in physical.iter().chain(&source_rows) {
            if let Some(old) = rows.insert(row.declaration_id, row)
                && old != row
            {
                return Err(contract(
                    "conflicting physical and model source declarations",
                ));
            }
        }
        let mut declarations =
            wire::Builder::with_registry(registry, rows.len(), &validation).map_err(relation)?;
        for row in rows.into_values() {
            declarations.push(row.clone()).map_err(relation)?;
        }
        tables.insert(
            wire::RELATION_ID,
            declarations
                .finish()
                .map_err(relation)?
                .retained(&services.pool(), &pse_columnar::CancellationToken::new())
                .map_err(relation)?,
        );
        Ok(tables)
    }
    /// Prepare predecessor-ordered conditional initialization under the analysis's solve
    /// profile and the supplied continuation stages, without changing the original
    /// specification. The profile is admitted in Rust before any native work (F26).
    #[cfg(all(test, feature = "solver-kinsol"))]
    pub(in crate::workflow) async fn prepare_block_initialization(
        &self,
        services: &crate::workflow::PreparationServices,
        a: &ModelingAnalysis,
        stages: Vec<BTreeMap<SemanticId, f64>>,
        cancel: &crate::CancelSource,
    ) -> Result<super::PreparedInitializationStrategy, WorkflowError> {
        self.check_service_bindings(services)?;
        self.prepare_initialization_stages(
            services,
            a,
            InitializationStageValues::Canonical(stages),
            cancel,
        )
        .await
    }
    /// Admit sparse physical stage coordinates against the actual resolved case.
    #[cfg(feature = "solver-kinsol")]
    pub async fn prepare_block_initialization_physical(
        &self,
        services: &crate::workflow::PreparationServices,
        a: &ModelingAnalysis,
        stages: &super::InitializationStages,
        cancel: &crate::CancelSource,
    ) -> Result<super::PreparedInitializationStrategy, WorkflowError> {
        self.check_service_bindings(services)?;
        let mut prepared = self
            .prepare_initialization_stages(
                services,
                a,
                InitializationStageValues::Physical(stages),
                cancel,
            )
            .await?;
        prepared
            .prepared
            .retain_physical_input(Arc::new(stages.clone()));
        Ok(prepared)
    }
    #[cfg(feature = "solver-kinsol")]
    async fn prepare_initialization_stages(
        &self,
        services: &crate::workflow::PreparationServices,
        a: &ModelingAnalysis,
        stages: InitializationStageValues<'_>,
        cancel: &crate::CancelSource,
    ) -> Result<super::PreparedInitializationStrategy, WorkflowError> {
        let preparation_driver = services.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                let resolved = self
                    .resolve_case(
                        services,
                        a.root,
                        a.instance,
                        a.bindings.clone(),
                        a.limits,
                        a.case.clone(),
                        a.order,
                        a.compiler,
                        a.solver.clone(),
                        a.numerical.clone(),
                        Default::default(),
                        false,
                        cancel,
                    )
                    .await?;
                let stages = match stages {
                    InitializationStageValues::Physical(stages) => {
                        let structure = resolved.model.case.compiled().plan.structure();
                        let ports = structure
                            .variables()
                            .iter()
                            .map(|variable| variable.port.clone())
                            .chain(structure.parameters().iter().cloned())
                            .collect::<Vec<_>>();
                        stages.admit(&self.quantities, &ports)?
                    }
                    #[cfg(test)]
                    InitializationStageValues::Canonical(stages) => stages,
                };
                let profile = crate::math::initialization::InitializationProfile {
                    solver: a.solver.clone(),
                    stages,
                };
                let prepared = services
                    .math()
                    .prepare_modeling_initialization(
                        self.numerical_workspace(services)?,
                        resolved.model.case,
                        a.compiler,
                        resolved.numerical,
                        cancel,
                    )
                    .await?;
                prepared.validate_profile(&resolved.model.values, &profile)?;
                Ok(super::PreparedInitializationStrategy {
                    review_source: self.clone(),

                    values: resolved.model.values,
                    providers: resolved.providers,
                    prepared,
                    profile,
                })
            })
            .await
    }
    /// Prepare declared topology independently of the selected case's numerical incidence.
    pub async fn prepare_flow(
        &self,
        services: &crate::workflow::PreparationServices,
        analysis: &ModelingAnalysis,
        selection: pse_compiler::workspace::ModelingFlowSelection,
        cancel: &crate::CancelSource,
    ) -> Result<crate::math::flows::PreparedFlow, WorkflowError> {
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                let deadline = services.math().preparation_deadline(cancel)?;
                let scoped_driver = cancel.with_deadline(Some(deadline));
                let cancel = &scoped_driver;
                let selected = self
                    .checked_selection_scoped(services, &[analysis.root], cancel, deadline)
                    .await?;
                let prepared = (|| {
                    let workspace = self.pure_workspace(services)?;
                    let revision = selected.admission.revision.clone();
                    Ok::<_, WorkflowError>((workspace, revision))
                })();
                let (workspace, revision) = prepared?;
                let result = services
                    .math()
                    .prepare_semantic_modeling_revision(
                        workspace,
                        revision,
                        analysis.root,
                        analysis.instance,
                        analysis.bindings.clone(),
                        analysis.limits,
                        cancel,
                    )
                    .await;
                let model = result?;
                Ok(services
                    .math()
                    .prepare_modeling_flow(model, self.quantities.clone(), selection, cancel)
                    .await?)
            })
            .await
    }
    /// Replace source declarations while preserving which documents see the physical
    /// names and the package data documents. No old document text is retained as the source
    /// of the edited IR.
    pub async fn with_declarations(
        &self,
        services: &crate::workflow::SourceServices,
        rows: Vec<Declaration>,
    ) -> Result<Self, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let activity = services.preparation().admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                let cancel = crate::CancelSource::new();
                let parent = self.delta_parent(services.preparation(), &cancel).await?;
                let old = parent
                    .rows
                    .iter()
                    .map(|row| (row.declaration_id, row))
                    .collect::<BTreeMap<_, _>>();
                let new = rows
                    .iter()
                    .map(|row| (row.declaration_id, row))
                    .collect::<BTreeMap<_, _>>();
                if new.len() != rows.len() {
                    return Err(contract("complete declarations repeat an identity"));
                }
                let delta = SourceDelta {
                    parent: self.revision_id().into(),
                    additions: rows
                        .iter()
                        .filter(|row| !old.contains_key(&row.declaration_id))
                        .cloned()
                        .collect(),
                    replacements: rows
                        .iter()
                        .filter(|row| old.get(&row.declaration_id).is_some_and(|old| *old != *row))
                        .cloned()
                        .collect(),
                    removals: old
                        .keys()
                        .filter(|id| !new.contains_key(id))
                        .copied()
                        .collect(),
                    ..Default::default()
                };
                self.apply_delta_parent_ordered(
                    services,
                    delta,
                    parent,
                    &cancel,
                    None,
                    Some(rows.iter().map(|row| row.declaration_id).collect()),
                )
                .await
            })
            .await
    }
    /// Finite K3 admission and inspection; solver orchestration belongs to the subsequent lowering packets.
    pub async fn prepare(
        &self,
        services: &crate::workflow::PreparationServices,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPreparation, WorkflowError> {
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("modeling.prepare");
                let deadline = services.math().preparation_deadline(cancel)?;
                let phase = cancel.child_with_deadline(Some(deadline));
                preparation::scoped(&phase, deadline, async {
                    let original = engagement::OriginalPreparationInputs::capture(
                        services, root, instance, &bindings, limits,
                    )?;
                    if let Some(scope) = &self.held_preparation {
                        let mut model = scope
                            .prepare(services, self, root, instance, bindings, limits, &phase)
                            .await?;
                        model.original = Some(original);
                        return Ok(model);
                    }
                    let engagement = self
                        .engage(services, root, instance, bindings, limits, &phase)
                        .await?;
                    let mut model = engagement.model().clone();
                    model.original = Some(original);
                    engagement.close().await?;
                    Ok(model)
                })
                .await
            })
            .await
    }
}

#[cfg(test)]
mod reuse_tests;
#[cfg(test)]
mod service_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn checked_source_export_keeps_storage_and_map_charges_after_packages_drop() {
        use pse_relations::generated::authored::modeling_declarations as wire;
        let runtime = super::super::tests::runtime();
        let pool = runtime.shared.pool();
        let rows = pse_authoring::language::parse(
            "package escaped {}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let package = runtime
            .modeling_package(rows, super::super::tests::physical())
            .await
            .unwrap();
        let tables = package
            .source_tables(&runtime.preparation_services())
            .await
            .unwrap();
        let escaped = tables[&wire::RELATION_ID].clone();
        let map_bytes = source_map_extent(tables.len()).unwrap();
        drop(tables);
        drop(package);
        drop(runtime);
        assert!(
            pool.reserved() > map_bytes,
            "escaped checked table retains map, storage and buffer charges"
        );
        let retained = pool.reserved();
        let clone = escaped.clone();
        assert_eq!(
            pool.reserved(),
            retained,
            "checked clones allocate no new column vectors"
        );
        drop(escaped);
        assert_eq!(pool.reserved(), retained);
        assert!(
            wire::Row::rows(&clone)
                .unwrap()
                .iter()
                .any(|row| row.name == "escaped")
        );
        drop(clone);
        assert_eq!(
            pool.reserved(),
            0,
            "last checked export releases all surviving source claims"
        );
    }

    #[tokio::test]
    async fn source_export_merges_physical_modeling_rows_and_rejects_conflicts() {
        use pse_relations::generated::authored::modeling_declarations as wire;
        let runtime = super::super::tests::runtime();
        let parse = |text| {
            pse_authoring::language::parse(
                text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                Default::default(),
            )
            .unwrap()
        };
        let rows = parse("package application {}");
        let mut package = runtime
            .modeling_package(rows.clone(), super::super::tests::physical())
            .await
            .unwrap();
        let spec = runtime
            .registry()
            .relation_by_id(wire::RELATION_ID)
            .unwrap();
        let table = |rows: Vec<Declaration>| {
            let mut b = wire::Builder::with_registry(
                runtime.registry(),
                rows.len(),
                &runtime.validation_context(),
            )
            .unwrap();
            for row in rows {
                b.push(row).unwrap();
            }
            b.finish().unwrap()
        };
        for physical in [vec![], rows.clone(), parse("package physical {}")] {
            package
                .physical
                .sources
                .insert(spec.key, table(physical.clone()));
            let exported = package
                .source_tables(&runtime.preparation_services())
                .await
                .unwrap();
            let retained = wire::Row::rows(&exported[&wire::RELATION_ID]).unwrap();
            let expected = rows
                .iter()
                .chain(&physical)
                .map(|r| r.declaration_id)
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(
                retained
                    .iter()
                    .map(|r| r.declaration_id)
                    .collect::<std::collections::BTreeSet<_>>(),
                expected
            );
        }
        let mut conflict = rows.clone();
        conflict[0].name = "conflicting_name".into();
        package.physical.sources.insert(spec.key, table(conflict));
        assert!(
            package
                .source_tables(&runtime.preparation_services(),)
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn generic_documents_prepare_revisions_and_cancel_without_poisoning() {
        let rt = super::super::tests::runtime();
        let physical = super::super::tests::physical();
        let rows = pse_authoring::language::parse(
            "package p { def Root { param p: Scalar = 2; var x: Scalar; eq e: x*p == 6; } }",
            SemanticId::from_bytes([77; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let source = pse_authoring::language::render(&rows).unwrap();
        let texts = BTreeMap::from([
            (
                "package.toml".into(),
                include_bytes!("../../../../tests/fixtures/packages/minimal_explicit/package.toml")
                    .to_vec(),
            ),
            ("models/kernel.pse".into(), source.into_bytes()),
        ]);
        let token = pse_columnar::CancellationToken::new();
        let pool = rt.shared.pool();
        let bundle = pse_admission::authoring_driver::document::load_package_documents_owned(
            &texts,
            rt.registry(),
            pse_authoring::ParseBudget::default(),
            &pool,
            &token,
            &rt.validation_context(),
        )
        .unwrap();
        let documents =
            pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                vec![bundle],
                &pool,
                &token,
            )
            .unwrap();
        let expected = documents
            .bundles()
            .iter()
            .flat_map(|bundle| &bundle.documents)
            .filter_map(|document| document.modeling_rows())
            .flat_map(|rows| rows.iter().cloned())
            .collect::<Vec<_>>();
        assert!(
            documents
                .bundles()
                .iter()
                .all(|bundle| !bundle.batches.contains_key(
                    &pse_relations::generated::authored::modeling_declarations::RELATION_ID,
                ))
        );
        let retained = pool.reserved();
        let inputs = document_inputs(
            &documents,
            rt.registry(),
            &physical,
            rt.shared.budget().math.workspace_bytes,
            &pool,
            &token,
        )
        .unwrap();
        assert_eq!(inputs.0, expected);
        assert!(pool.reserved() > retained);
        drop(inputs);
        assert_eq!(pool.reserved(), retained);
        let small: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1));
        assert!(
            document_inputs(
                &documents,
                rt.registry(),
                &physical,
                rt.shared.budget().math.workspace_bytes,
                &small,
                &token
            )
            .is_err()
        );
        assert_eq!(small.reserved(), 0);
        let package = rt
            .modeling_from_documents(&documents, physical, &crate::CancelSource::new())
            .await
            .unwrap();
        let original = package
            .prepare(
                &rt.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(original.compiled().model.symbols.len(), 2);
        let mut updated = rows;
        updated
            .iter_mut()
            .find(|r| r.value.binding.is_some() && r.name == "p")
            .unwrap()
            .value
            .binding
            .as_mut()
            .unwrap()
            .expression = Some("3".into());
        let revised = package
            .with_declarations(&rt.source_services(), updated)
            .await
            .unwrap();
        let changed = revised
            .prepare(
                &rt.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert!(!original.compiled().admitted.bodies.is_empty());
        for (key, body) in &original.compiled().admitted.bodies {
            let revised_body = &changed.compiled().admitted.bodies[key];
            assert_eq!(body.spec().key(), revised_body.spec().key());
            assert!(
                Arc::ptr_eq(body.math(), revised_body.math()),
                "a default input edit preserves the admitted scientific mathematics"
            );
        }
        let initial = |model: &ModelingPreparation| {
            let symbol = model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path == "p" || symbol.lineage.path.ends_with(".p"))
                .unwrap();
            match symbol.initial.as_ref().unwrap() {
                pse_modeling::specialize::Value::Number { bits, .. } => f64::from_bits(*bits),
                other => panic!("expected the authored scalar input, got {other:?}"),
            }
        };
        assert_eq!(initial(&original), 2.0);
        assert_eq!(initial(&changed), 3.0);
        assert_ne!(
            original.compiled().model.symbols,
            changed.compiled().model.symbols
        );
        let cancel = crate::CancelSource::new();
        cancel.cancel();
        assert!(
            revised
                .prepare(
                    &rt.preparation_services(),
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    &cancel
                )
                .await
                .is_err()
        );
        let restored = package
            .prepare(
                &rt.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(original.compiled().model, restored.compiled().model);
        let retained = pool.reserved();
        drop(restored);
        drop(original);
        drop(changed);
        // Caller departure does not release prepared bases retained by the service.
        // Explicitly retire optional program retention before asserting final release.
        rt.clear_program_cache();
        assert!(pool.reserved() < retained);
    }
}

#[cfg(test)]
mod import_tests {
    use super::*;
    use pse_quantity::QuantityTypeId;

    /// The physical primitives fixture's documents by relative path.
    fn primitives() -> BTreeMap<String, Vec<u8>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/packages/physical-primitives");
        [
            "package.toml",
            "materials/physical.yaml",
            "materials/time.yaml",
        ]
        .into_iter()
        .map(|path| (path.to_owned(), std::fs::read(root.join(path)).unwrap()))
        .collect()
    }
    #[tokio::test]
    async fn pure_document_conformance_uses_admitted_typed_rows() {
        let rt = super::super::tests::runtime();
        let manifest = include_str!("../../../../tests/fixtures/packages/minimal_named/package.toml")
            .replace("dependencies = []", r#"dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#);
        let documents = BTreeMap::from([
            ("package.toml".to_owned(), manifest.into_bytes()),
            ("models/analytic.pse".to_owned(), b"package p { fn cube(x:Scalar)->Scalar=x*x*x; test analytic fixture { dof 0; route steady; procedure check; } { expect cube(2)==8 tolerance 1e-12; } }".to_vec()),
        ]);
        let report = conform_pure_documents(
            vec![documents],
            primitives(),
            rt.shared.budget().clone(),
            ModelingFixtureSelection::Package,
            4,
            16,
            crate::workflow::PreparationSettings::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
        use pse_model::generated::enums::{
            ModelingConformanceKind as Kind, ModelingConformanceStatus as Status,
        };
        assert!(report.complete);
        let expectation = report
            .checks
            .iter()
            .find(|check| check.kind == Kind::Expectation && check.deviation.is_some())
            .unwrap();
        assert_eq!(expectation.status, Status::Passed);
        assert_eq!(expectation.deviation, Some(0.0));
        assert_eq!(expectation.tolerance, Some(1e-12));
        // The fixture has no validity envelope or conservation closure; their
        // explicit NotApplicable inventory rows accompany the evaluated checks.
        assert!(
            report
                .checks
                .iter()
                .all(|check| matches!(check.status, Status::Passed | Status::NotApplicable)),
            "{:?}",
            report.checks
        );
    }
    fn load(
        rt: &Runtime,
        texts: &BTreeMap<String, Vec<u8>>,
    ) -> Result<pse_admission::authoring_driver::document::OwnedDocumentSet, WorkflowError> {
        let token = pse_columnar::CancellationToken::new();
        let pool = rt.shared.pool();
        let bundle = pse_admission::authoring_driver::document::load_package_documents_owned(
            texts,
            rt.registry(),
            pse_authoring::ParseBudget::default(),
            &pool,
            &token,
            &rt.validation_context(),
        )?;
        Ok(
            pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                vec![bundle],
                &pool,
                &token,
            )?,
        )
    }

    /// ADR-0123 Outcome 6: quantity types are named once, in the physical document. The
    /// reference inventory names exactly the former manifest aliases; a manifest can no
    /// longer declare a name; a name declared twice is refused at physical admission.
    #[tokio::test]
    async fn quantity_names_declared_once_in_physical_document() {
        let rt = super::super::tests::runtime();
        let token = pse_columnar::CancellationToken::new();
        let physical = rt
            .physical_from_documents(&load(&rt, &primitives()).unwrap(), &token)
            .await
            .unwrap();
        let named = |name| physical.quantities().physical_name(name);
        assert_eq!(
            named("Scalar"),
            Some(pse_quantity::PhysicalName::QuantityType(
                QuantityTypeId::from_id(SemanticId::from_bytes([0x1f; 16]))
            ))
        );
        assert!(named("Length").is_some() && named("Time").is_some());
        // Compare the generated reference inventory to the authoritative document's
        // exact names and identities, including independently named reference states.
        let standard = pse_quantity::standard::standard_registry().unwrap();
        let document: pse_authoring::generated::documents::MaterialsDocument =
            serde_saphyr::from_str(include_str!(
                "../../../../packages/reference/physical/materials/physical.yaml"
            ))
            .unwrap();
        let declared: BTreeMap<_, _> = document
            .quantity_types
            .into_iter()
            .filter_map(|row| {
                row.name.map(|name| {
                    (
                        name,
                        pse_quantity::PhysicalName::QuantityType(row.quantity_type_id),
                    )
                })
            })
            .chain(document.reference_states.into_iter().map(|row| {
                (
                    row.name,
                    pse_quantity::PhysicalName::ReferenceState(row.reference_state_id),
                )
            }))
            .collect();
        let generated: BTreeMap<_, _> = standard
            .physical_names()
            .map(|(name, identity)| (name.to_owned(), identity))
            .collect();
        assert_eq!(generated, declared);
        assert_eq!(
            standard.physical_name("MolarCp"),
            Some(pse_quantity::PhysicalName::QuantityType(
                QuantityTypeId::from_id(
                    SemanticId::parse_hex("cd653ba98fa94d16b5d66b363f21c3d6").unwrap()
                )
            ))
        );
        // A manifest declares no physical names.
        let mut aliased = BTreeMap::from([(
            "package.toml".to_owned(),
            (include_str!("../../../../tests/fixtures/packages/minimal_named/package.toml")
                .to_owned()
                + "\n[[quantity_aliases]]\nname = \"Scalar\"\nquantity_type_id = \"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f\"\n")
                .into_bytes(),
        )]);
        let refused = load(&rt, &aliased).unwrap_err().to_string();
        assert!(refused.contains("quantity_aliases"), "{refused}");
        aliased.clear();
        // Naming a second type `Scalar` in the physical document is refused.
        let mut twice = primitives();
        let time = twice.get_mut("materials/time.yaml").unwrap();
        *time = String::from_utf8_lossy(time)
            .replacen("\"name\": \"Time\"", "\"name\": \"Scalar\"", 1)
            .into_bytes();
        let refused = rt
            .physical_from_documents(&load(&rt, &twice).unwrap(), &token)
            .await
            .unwrap_err()
            .to_string();
        assert!(refused.contains("declared more than once"), "{refused}");
    }

    /// ADR-0123 Outcome 6: a package sees the physical names, unqualified or qualified by
    /// the declaring package, exactly when its manifest depends on that package.
    #[tokio::test]
    async fn quantity_name_requires_manifest_dependency() {
        let rt = super::super::tests::runtime();
        let token = pse_columnar::CancellationToken::new();
        let physical = rt
            .physical_from_documents(&load(&rt, &primitives()).unwrap(), &token)
            .await
            .unwrap();
        let manifest =
            include_str!("../../../../tests/fixtures/packages/minimal_named/package.toml");
        let dependent = manifest.replace(
            "dependencies = []",
            r#"dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#,
        );
        let package = |manifest: &str, source: &str| {
            load(
                &rt,
                &BTreeMap::from([
                    ("package.toml".to_owned(), manifest.as_bytes().to_vec()),
                    ("models/root.pse".to_owned(), source.as_bytes().to_vec()),
                ]),
            )
            .unwrap()
        };
        for source in [
            "package app { def Root { var x: Scalar; } }",
            "package app { def Root { var x: \"physical-primitives\".Scalar; } }",
        ] {
            let selected = async |manifest: &str| {
                let package = rt
                    .modeling_from_documents(
                        &package(manifest, source),
                        physical.clone(),
                        &crate::CancelSource::new(),
                    )
                    .await?;
                let root = package
                    .declarations(&rt.preparation_services())
                    .await?
                    .iter()
                    .find(|row| row.name == "Root")
                    .unwrap()
                    .declaration_id;
                package
                    .selected_revision(
                        &rt.preparation_services(),
                        root,
                        &crate::CancelSource::new(),
                    )
                    .await
            };
            assert!(selected(&dependent).await.is_ok(), "{source}");
            let refused = selected(manifest).await.unwrap_err().to_string();
            assert!(refused.contains("unknown type"), "{source}: {refused}");
        }
    }
    #[tokio::test]
    async fn modeling_closure_refuses_missing_conflicting_and_cyclic_dependencies() {
        let rt = super::super::tests::runtime();
        let physical = super::super::tests::physical();
        let token = pse_columnar::CancellationToken::new();
        let pool = rt.shared.pool();
        let package = |id: u8, name: &str, dependencies: &str, source: &str| {
            let manifest = format!(
                "[package]\nid=\"{}\"\nname=\"{name}\"\nversion=\"1.0.0\"\nkind=\"model\"\nid_policy=\"named\"\ndependencies=[{dependencies}]\ndoc=\"synthetic closure\"\n",
                SemanticId::from_bytes([id; 16])
            );
            pse_admission::authoring_driver::document::load_package_documents_owned(
                &BTreeMap::from([
                    ("package.toml".into(), manifest.into_bytes()),
                    ("models/kernel.pse".into(), source.as_bytes().to_vec()),
                ]),
                rt.registry(),
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
                &rt.validation_context(),
            )
            .unwrap()
        };
        let dependency = |id, major| {
            format!(
                "{{package_id=\"{}\",version_req={{operator=\"exact\",major={major},minor=0,patch=0}}}}",
                SemanticId::from_bytes([id; 16])
            )
        };
        let library = package(
            81,
            "lib",
            "",
            "package lib {fn twice(x:Scalar)->Scalar=2*x;}",
        );
        let application = package(
            82,
            "app",
            &dependency(81, 1),
            "package app {use lib @\"1.0.0\"; def Root {let result:Scalar=lib.twice(2);}}",
        );
        let admit = async |parts| {
            let documents =
                pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                    parts, &pool, &token,
                )
                .unwrap();
            let package = rt
                .modeling_from_documents(&documents, physical.clone(), &crate::CancelSource::new())
                .await?;
            // Draft persistence is structural; these controls demand the authored
            // importing scope/function before checking its scientific obligations.
            let declarations = package.declarations(&rt.preparation_services()).await?;
            for declaration in declarations
                .iter()
                .filter(|row| row.name == "Root" || row.name == "f" || row.value.import.is_some())
            {
                package
                    .selected_revision(
                        &rt.preparation_services(),
                        declaration.declaration_id,
                        &crate::CancelSource::new(),
                    )
                    .await?;
            }
            Ok::<_, WorkflowError>(package)
        };
        assert!(
            admit(vec![library.clone(), application.clone()])
                .await
                .is_ok()
        );
        assert!(admit(vec![application.clone()]).await.is_err());
        assert!(
            admit(vec![library.clone(), library.clone(), application.clone()])
                .await
                .is_err()
        );
        let cycle = package(
            81,
            "lib",
            &dependency(82, 1),
            "package lib {fn twice(x:Scalar)->Scalar=2*x;}",
        );
        assert!(admit(vec![cycle, application.clone()]).await.is_err());
        for (dependency_text, source_text) in [
            (dependency(81, 2), "package app {}"),
            (String::new(), "package app {use lib @\"1.0.0\";}"),
            (
                dependency(81, 1),
                "package app {fn f(x:Scalar)->Scalar=lib.twice(x);}",
            ),
        ] {
            let bad = package(82, "app", &dependency_text, source_text);
            assert!(admit(vec![library.clone(), bad]).await.is_err());
        }
    }
    /// ADR-0123 Outcome 7: an import resolves its target package by identity. A manifest
    /// dependency on another package at the same version grants nothing; the dependency on
    /// the target's identity must carry a requirement that admits the target's version, as
    /// the import's own typed requirement must.
    #[tokio::test]
    async fn import_requires_dependency_by_identity() {
        let rt = super::super::tests::runtime();
        let physical = super::super::tests::physical();
        let token = pse_columnar::CancellationToken::new();
        let pool = rt.shared.pool();
        let package = |id: u8, name: &str, dependencies: &[(u8, i64)], source: &str| {
            let dependencies = dependencies
                .iter()
                .map(|(id, major)| {
                    format!(
                        "{{package_id=\"{}\",version_req={{operator=\"exact\",major={major},minor=0,patch=0}}}}",
                        SemanticId::from_bytes([*id; 16])
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let manifest = format!(
                "[package]\nid=\"{}\"\nname=\"{name}\"\nversion=\"1.0.0\"\nkind=\"model\"\nid_policy=\"named\"\ndependencies=[{dependencies}]\ndoc=\"synthetic identity\"\n",
                SemanticId::from_bytes([id; 16])
            );
            pse_admission::authoring_driver::document::load_package_documents_owned(
                &BTreeMap::from([
                    ("package.toml".into(), manifest.into_bytes()),
                    ("models/kernel.pse".into(), source.as_bytes().to_vec()),
                ]),
                rt.registry(),
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
                &rt.validation_context(),
            )
            .unwrap()
        };
        let library = package(
            91,
            "lib",
            &[],
            "package lib {fn twice(x:Scalar)->Scalar=2*x;}",
        );
        let other = package(93, "other", &[], "package other {}");
        let application = |dependencies: &[(u8, i64)]| {
            package(
                92,
                "app",
                dependencies,
                "package app {use lib @\"1.0.0\"; def Root {let result:Scalar=lib.twice(2);}}",
            )
        };
        let admit = async |parts| {
            let documents =
                pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                    parts, &pool, &token,
                )
                .unwrap();
            rt.modeling_from_documents(&documents, physical.clone(), &crate::CancelSource::new())
                .await
        };
        assert!(
            admit(vec![
                library.clone(),
                other.clone(),
                application(&[(91, 1)])
            ])
            .await
            .is_ok()
        );
        // The same version, depended on under another identity, is not the target.
        let refused = admit(vec![
            library.clone(),
            other.clone(),
            application(&[(93, 1)]),
        ])
        .await
        .unwrap_err()
        .to_string();
        assert!(refused.contains("exact manifest dependency"), "{refused}");
        // The typed requirement is checked against the manifest version.
        assert!(
            admit(vec![library, other, application(&[(91, 2)])])
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn modeling_document_import_checks_the_admitted_version() {
        let rt = super::super::tests::runtime();
        let physical = super::super::tests::physical();
        for (version, valid) in [("1.0.0", true), ("2.0.0", false)] {
            let source = format!(
                "package library {{ fn f(x:Scalar)->Scalar=x; }} package p {{ use library @ \"{version}\"; def Root {{ var x:Scalar; eq e:library.f(x)==0; }} }}"
            );
            let source = pse_authoring::language::assign_ids(
                &source,
                SemanticId::NIL,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let texts = BTreeMap::from([
                (
                    "package.toml".into(),
                    include_bytes!(
                        "../../../../tests/fixtures/packages/minimal_explicit/package.toml"
                    )
                    .to_vec(),
                ),
                ("models/kernel.pse".into(), source.into_bytes()),
            ]);
            let token = pse_columnar::CancellationToken::new();
            let pool = rt.shared.pool();
            let bundle = pse_admission::authoring_driver::document::load_package_documents_owned(
                &texts,
                rt.registry(),
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
                &rt.validation_context(),
            )
            .unwrap();
            let documents =
                pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                    vec![bundle],
                    &pool,
                    &token,
                )
                .unwrap();
            assert_eq!(
                rt.modeling_from_documents(
                    &documents,
                    physical.clone(),
                    &crate::CancelSource::new()
                )
                .await
                .is_ok(),
                valid
            );
        }
    }
}

#[cfg(test)]
mod applicability_tests;

impl Runtime {
    /// Supply the composition root's explicit source capabilities.
    pub(crate) fn package_from_admission(
        &self,
        admitted: Arc<PackageAdmission>,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.source_services()
            .package_from_admission(admitted, physical)
    }
    /// Supply the composition root's explicit source capabilities.
    pub async fn modeling_from_documents(
        &self,
        documents: &pse_admission::authoring_driver::document::OwnedDocumentSet,
        physical: PhysicalContext,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.source_services()
            .modeling_from_documents(documents, physical, cancel)
            .await
    }
    /// Supply the composition root's explicit source capabilities.
    pub async fn project_fork(
        &self,
        project: &pse_operations::canonical_projects::Project,
        branch: &str,
        problem: &str,
        operation: &str,
        source: &ModelingPackage,
    ) -> Result<pse_operations::canonical::Revision, WorkflowError> {
        self.source_services()
            .project_fork(project, branch, problem, operation, source)
            .await
    }
    /// Supply the composition root's explicit source capabilities.
    pub async fn modeling_package(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.source_services()
            .modeling_package(rows, physical)
            .await
    }
    /// Supply the composition root's explicit source capabilities.
    pub async fn modeling_package_registered(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.source_services()
            .modeling_package_registered(rows, physical, providers)
            .await
    }

    /// Supply the composition root's explicit source capabilities.
    pub async fn modeling_revision(
        &self,
        revision: pse_model::generated::runtime::canonical_revisions::Row,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.source_services()
            .modeling_revision(revision, physical, providers)
            .await
    }
}
