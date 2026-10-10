// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public finite modeling packages are immutable inputs to the existing compiler service.
// An unscoped caller has no scientific operation clock to inherit. Its shared pure loader
// still needs a finite lifetime; this omission policy is separate from SDK request and
// native solver limits. Scoped operations always retain their original absolute clock.
const UNSCOPED_PREPARATION_LIMIT: std::time::Duration = std::time::Duration::from_secs(600);

fn preparation_deadline(cancel: &crate::CancelSource) -> Result<std::time::Instant, WorkflowError> {
    cancel.deadline().map_or_else(
        || {
            std::time::Instant::now()
                .checked_add(UNSCOPED_PREPARATION_LIMIT)
                .ok_or_else(|| {
                    crate::math::MathRuntimeError::Limit("preparation deadline extent").into()
                })
        },
        Ok,
    )
}
mod canonical;
pub(super) mod cases;
mod conformance;
pub(super) mod declared;
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
pub use dynamics::{ModelingSimulation, ModelingTrajectory};
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
    ModelingNonlinearPolicy,
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
pub(super) mod results;
mod views;
use super::{PhysicalContext, Runtime, WorkflowError, contract, relation};
use crate::math::{Workspace, modeling::ModelingPreparation};
pub use analysis_tables::ModelingNativeAnalysis;
pub use cases::{ModelingObservations, ModelingSolvePreparation, StartSource};
use pse_authoring::language::Declaration;
use pse_compiler::workspace::{CompilerContext, WorkspaceLimits};
use pse_ids::SemanticId;
use pse_modeling::{Bindings, DeclarationId, InstanceId, Limits, PhysicalScope};
use pse_relations::columnar::RelationRow;
pub use results::{ModelingCheck, ModelingReport, ModelingResult};
#[cfg(test)]
use std::sync::Mutex;
use std::{collections::BTreeMap, sync::Arc};

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
    pub(in crate::workflow) runtime: Runtime,
    pub(in crate::workflow) revision: canonical::SourceRevision,
    accelerators: Arc<pse_math::implicit::accelerators::Accelerators>,
    providers: Arc<BTreeMap<String, pse_kernels::Registration>>,
    pub(in crate::workflow) physical: PhysicalContext,
    pub(in crate::workflow) quantities: Arc<pse_quantity::QuantityRegistry>,
}
/// Process-local request context; scientific eligibility is independently rechecked.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SelectedRequest {
    problem: String,
    roots: Vec<DeclarationId>,
    physical: pse_ids::ContentHash,
    providers: Vec<(String, pse_kernels::ProviderKey, pse_ids::ContentHash)>,
    validation: usize,
    registry: usize,
}
impl SelectedRequest {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.problem.capacity()
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
    pub(crate) dependencies: Arc<pse_operations::canonical_selection::SelectedDependencies>,
    pub(crate) qualification: Arc<pse_operations::canonical_selection::SelectionDependencyReceipt>,
    pub(crate) versions: Arc<BTreeMap<String, String>>,
    pub(crate) metadata: Arc<pse_columnar::AllocationLease>,
    pub(crate) _validation_owner: Arc<pse_engine::session::EngineFactory>,
    pub(crate) _registry_owner: Arc<pse_schema::Registry>,
}
impl SelectedAdmission {
    pub(crate) fn retained_bytes(&self) -> usize {
        self.revision.retained_bytes()
            + self.qualification.retained_bytes()
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
    _validation_owner: Arc<pse_engine::session::EngineFactory>,
    _registry_owner: Arc<pse_schema::Registry>,
}
impl PackageAdmission {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.revision.canonical.problem.capacity()
            + self.revision.canonical.key.capacity()
            + 256
    }
}
impl ModelingPackage {
    pub(crate) fn admission(&self) -> Result<PackageAdmission, WorkflowError> {
        Ok(PackageAdmission {
            revision: self.revision.clone(),
            accelerators: self.accelerators.clone(),
            providers: self.providers.clone(),
            _lease: self.runtime.shared.math().reserve(
                "modeling:package-admission",
                size_of::<PackageAdmission>()
                    + 256
                    + self
                        .providers
                        .keys()
                        .map(|name| name.capacity() + size_of::<pse_kernels::Registration>() + 128)
                        .sum::<usize>(),
            )?,
            _validation_owner: self.runtime.sessions.clone(),
            _registry_owner: self.runtime.registry.clone(),
        })
    }
}
impl Runtime {
    pub(crate) fn package_from_admission(
        &self,
        admitted: PackageAdmission,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        Ok(ModelingPackage {
            runtime: self.clone(),
            revision: admitted.revision,
            accelerators: admitted.accelerators,
            providers: admitted.providers,
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
    super::FitDeclarations,
    crate::authoring_driver::document::Batches,
    Arc<pse_modeling::document::DocumentInventory>,
    Arc<pse_columnar::AllocationLease>,
);
/// The package data documents of a closure and the package of each text document, which a
/// dataset's document path resolves in (ADR-0125).
fn data_documents(
    documents: &crate::authoring_driver::document::OwnedDocumentSet,
) -> pse_modeling::document::DocumentInventory {
    let mut inventory = pse_modeling::document::DocumentInventory::default();
    for bundle in documents.bundles() {
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
    documents: &crate::authoring_driver::document::OwnedDocumentSet,
    registry: &pse_schema::Registry,
    physical: &PhysicalContext,
    workspace_bytes: usize,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<DocumentCompilerContext, WorkflowError> {
    use pse_model::HeapUsage;
    documents.validate_registry(registry)?;
    cancel
        .checkpoint()
        .map_err(pse_relations::RelationError::from)
        .map_err(relation)?;
    let (count, bytes) = documents
        .bundles()
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
    let mut headers = documents
        .bundles()
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
    let batches = crate::authoring_driver::p1::source_context_batches(
        documents.bundles(),
        registry,
        &headers,
        limits,
    )?;
    let mut rows = Vec::with_capacity(count);
    rows.extend(
        documents
            .bundles()
            .iter()
            .flat_map(|bundle| &bundle.documents)
            .filter_map(|document| document.modeling_rows())
            .flat_map(|rows| rows.iter().cloned()),
    );
    validate_import_versions(&rows, documents)?;
    let scope = physical_scope(documents, physical);
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
        super::FitDeclarations::from_batches(&batches)?,
        context,
        Arc::new(data_documents(documents)),
        lease,
    ))
}
impl Runtime {
    /// Persist native-admitted authored source once; later preparations select immutable records.
    pub async fn modeling_from_documents(
        &self,
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        physical: PhysicalContext,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        let token = cancel.token();
        token.checkpoint().map_err(pse_engine::EngineError::from)?;
        let validation = self.sessions.validation_context(&self.registry)?;
        documents.validate_context(&self.registry, &validation, &self.shared.pool(), &token)?;
        let (rows, scope, fits, sources, inventory, _input_lease) = document_inputs(
            documents,
            &self.registry,
            &physical,
            self.shared.budget().math.workspace_bytes,
            &self.shared.pool(),
            &token,
        )?;
        token.checkpoint().map_err(pse_engine::EngineError::from)?;
        // Publication is an issued canonical effect; finish its owner before
        // observing cancellation again. Pure validation uses the caller token.
        let package = self
            .persist_modeling(
                rows,
                physical,
                scope,
                inventory,
                sources,
                fits,
                BTreeMap::new(),
            )
            .await?;
        token.checkpoint().map_err(pse_engine::EngineError::from)?;
        Ok(package)
    }
    /// Persist registry-generated rows without serializing through a text document.
    pub async fn modeling_package(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.modeling_package_registered(rows, physical, BTreeMap::new())
            .await
    }
    /// Native capabilities remain explicitly supplied services; source names load no code.
    pub async fn modeling_package_registered(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
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
        )
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
        sources: crate::authoring_driver::document::Batches,
        fits: super::fitting::FitDeclarations,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        let revision = canonical::persist(
            self, rows, &physical, scope, &documents, &sources, &fits, None,
        )
        .await?;
        self.package_from_canonical(revision, physical, providers)
    }
    fn package_from_canonical(
        &self,
        revision: canonical::SourceRevision,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        Ok(ModelingPackage {
            runtime: self.clone(),
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
        let source = canonical::SourceRevision::open(self, revision).await?;
        self.package_from_canonical(source, physical, providers)
    }
}
/// Which documents see the physical names (ADR-0123 Outcome 6): those of packages whose
/// manifest depends on, or is, the package that declared them. A physical context admitted
/// without documents names no declaring package, so every row sees its names.
fn physical_scope(
    documents: &crate::authoring_driver::document::OwnedDocumentSet,
    physical: &PhysicalContext,
) -> PhysicalScope {
    let Some(declaring) = &physical.package else {
        return PhysicalScope::default();
    };
    PhysicalScope {
        package: Some(declaring.name.clone()),
        documents: Some(
            documents
                .bundles()
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
    documents: &crate::authoring_driver::document::OwnedDocumentSet,
) -> Result<(), WorkflowError> {
    let owners = documents
        .bundles()
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
    ) -> Result<Workspace, crate::math::MathRuntimeError> {
        self.runtime.shared.math().workspace(
            compiler_context(&self.physical, &self.providers),
            WorkspaceLimits::default(),
        )
    }

    /// Explicit full source inventory; ordinary preparation reads only its dependency closure.
    pub async fn declarations(&self) -> Result<OwnedDeclarations, WorkflowError> {
        self.declaration_inventory().await
    }
    /// Exact immutable canonical revision, independent of semantic compiler source hashes.
    pub fn canonical_revision(&self) -> &pse_model::generated::runtime::canonical_revisions::Row {
        &self.revision.canonical
    }
    /// Generate full source tables only for an explicit export/publication request.
    pub async fn source_tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        self.encode_source_tables().await
    }
    async fn encode_source_tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        use pse_model::HeapUsage;
        use pse_relations::generated::authored::modeling_declarations as wire;
        let (source_rows, _, _, document_sources, _source_owners) = self.full_source().await?;
        let registry = &self.runtime.registry;
        let validation = self.runtime.validation_context()?;
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
        let _scratch = self
            .runtime
            .shared
            .math()
            .reserve("modeling:source-export", bytes)?;
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
                .retained(
                    &self.runtime.shared.pool(),
                    &pse_columnar::CancellationToken::new(),
                )
                .map_err(relation)?,
        );
        Ok(tables)
    }
    /// Prepare predecessor-ordered conditional initialization under the analysis's solve
    /// profile and the supplied continuation stages, without changing the original
    /// specification. The profile is admitted in Rust before any native work (F26).
    #[cfg(feature = "solver-kinsol")]
    pub async fn prepare_block_initialization(
        &self,
        a: &ModelingAnalysis,
        stages: Vec<BTreeMap<SemanticId, f64>>,
        cancel: &crate::CancelSource,
    ) -> Result<super::PreparedInitializationStrategy, WorkflowError> {
        let profile = crate::math::initialization::InitializationProfile {
            solver: a.solver.clone(),
            stages,
        };
        let resolved = self
            .resolve_case(
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
        let prepared = self
            .runtime
            .native()
            .prepare_modeling_initialization(
                self.numerical_workspace()?,
                resolved.model.case,
                a.compiler,
                resolved.numerical,
                cancel,
            )
            .await?;
        prepared.validate_profile(&resolved.model.values, &profile)?;
        Ok(super::PreparedInitializationStrategy {
            runtime: self.runtime.clone(),
            values: resolved.model.values,
            providers: resolved.providers,
            prepared,
            profile,
        })
    }
    /// Prepare declared topology independently of the selected case's numerical incidence.
    pub async fn prepare_flow(
        &self,
        analysis: &ModelingAnalysis,
        selection: pse_compiler::workspace::ModelingFlowSelection,
        cancel: &crate::CancelSource,
    ) -> Result<crate::math::flows::PreparedFlow, WorkflowError> {
        let deadline = preparation_deadline(cancel)?;
        let scoped_driver = cancel.with_deadline(Some(deadline));
        let cancel = &scoped_driver;
        let selected = self
            .checked_selection_scoped(&[analysis.root], cancel, deadline)
            .await?;
        let prepared = (|| {
            let workspace = self.pure_workspace()?;
            let revision = selected.admission.revision.clone();
            Ok::<_, WorkflowError>((workspace, revision))
        })();
        let (workspace, revision) = match prepared {
            Ok(value) => value,
            Err(error) => {
                let _ = self
                    .runtime
                    .canonical
                    .store()
                    .release(selected.read.selection())
                    .await;
                return Err(error);
            }
        };
        let result = self
            .runtime
            .shared
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
        let release = self
            .runtime
            .canonical
            .store()
            .release(selected.read.selection())
            .await;
        let model = result?;
        release?;
        Ok(self
            .runtime
            .native()
            .prepare_modeling_flow(model, self.quantities.clone(), selection, cancel)
            .await?)
    }
    /// Replace source declarations while preserving which documents see the physical
    /// names and the package data documents. No old document text is retained as the source
    /// of the edited IR.
    pub async fn with_declarations(&self, rows: Vec<Declaration>) -> Result<Self, WorkflowError> {
        let (_, scope, mut documents, mut sources, _source_owners) = self.full_source().await?;
        Arc::make_mut(&mut documents).field_spans.clear();
        if let Some(batch) =
            sources.get_mut(&pse_relations::generated::authored::documents::RELATION_ID)
        {
            use pse_relations::generated::authored::documents as wire;
            let rows = wire::Row::rows(batch).map_err(relation)?;
            let validation = self.runtime.validation_context()?;
            let mut builder =
                wire::Builder::with_registry(&self.runtime.registry, rows.len(), &validation)
                    .map_err(relation)?;
            for mut row in rows {
                row.source_text = None;
                builder.push(row).map_err(relation)?;
            }
            *batch = builder.finish().map_err(relation)?;
        }
        let revision = canonical::persist(
            &self.runtime,
            rows,
            &self.physical,
            scope,
            &documents,
            &sources,
            &Default::default(),
            Some(&self.revision),
        )
        .await?;
        Ok(Self {
            revision,
            ..self.clone()
        })
    }
    /// Finite K3 admission and inspection; solver orchestration belongs to the subsequent lowering packets.
    pub async fn prepare(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPreparation, WorkflowError> {
        let deadline = preparation_deadline(cancel)?;
        let mut selected = self
            .checked_selection_scoped(&[root], cancel, deadline)
            .await?;
        let result = self
            .prepare_selected(
                &mut selected,
                root,
                instance,
                bindings,
                limits,
                cancel,
                deadline,
            )
            .await;
        let release = self
            .runtime
            .canonical
            .store()
            .release(selected.read.selection())
            .await;
        let result = result?;
        release?;
        Ok(result)
    }
}

#[cfg(test)]
mod reuse_tests;

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
        let tables = package.source_tables().await.unwrap();
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
        let spec = runtime.registry.relation_by_id(wire::RELATION_ID).unwrap();
        let table = |rows: Vec<Declaration>| {
            let mut b = wire::Builder::with_registry(
                &runtime.registry,
                rows.len(),
                &runtime.validation_context().unwrap(),
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
            let exported = package.source_tables().await.unwrap();
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
        assert!(package.source_tables().await.is_err());
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
        let bundle = crate::authoring_driver::document::load_package_documents_owned(
            &texts,
            &rt.registry,
            pse_authoring::ParseBudget::default(),
            &pool,
            &token,
            &rt.validation_context().unwrap(),
        )
        .unwrap();
        let documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
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
            &rt.registry,
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
                &rt.registry,
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
        let revised = package.with_declarations(updated).await.unwrap();
        let changed = revised
            .prepare(
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
    ) -> Result<crate::authoring_driver::document::OwnedDocumentSet, WorkflowError> {
        let token = pse_columnar::CancellationToken::new();
        let pool = rt.shared.pool();
        let bundle = crate::authoring_driver::document::load_package_documents_owned(
            texts,
            &rt.registry,
            pse_authoring::ParseBudget::default(),
            &pool,
            &token,
            &rt.validation_context().unwrap(),
        )?;
        Ok(
            crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
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
                        pse_quantity::PhysicalName::QuantityType(QuantityTypeId::from_id(
                            row.quantity_type_id,
                        )),
                    )
                })
            })
            .chain(document.reference_states.into_iter().map(|row| {
                (
                    row.name,
                    pse_quantity::PhysicalName::ReferenceState(
                        pse_quantity::ReferenceStateId::from_id(row.reference_state_id),
                    ),
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
                    .declarations()
                    .await?
                    .iter()
                    .find(|row| row.name == "Root")
                    .unwrap()
                    .declaration_id;
                package
                    .selected_revision(root, &crate::CancelSource::new())
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
            crate::authoring_driver::document::load_package_documents_owned(
                &BTreeMap::from([
                    ("package.toml".into(), manifest.into_bytes()),
                    ("models/kernel.pse".into(), source.as_bytes().to_vec()),
                ]),
                &rt.registry,
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
                &rt.validation_context().unwrap(),
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
            let documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                parts, &pool, &token,
            )
            .unwrap();
            let package = rt
                .modeling_from_documents(&documents, physical.clone(), &crate::CancelSource::new())
                .await?;
            // Draft persistence is structural; these controls demand the authored
            // importing scope/function before checking its scientific obligations.
            let declarations = package.declarations().await?;
            for declaration in declarations
                .iter()
                .filter(|row| row.name == "Root" || row.name == "f" || row.value.import.is_some())
            {
                package
                    .selected_revision(declaration.declaration_id, &crate::CancelSource::new())
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
            crate::authoring_driver::document::load_package_documents_owned(
                &BTreeMap::from([
                    ("package.toml".into(), manifest.into_bytes()),
                    ("models/kernel.pse".into(), source.as_bytes().to_vec()),
                ]),
                &rt.registry,
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
                &rt.validation_context().unwrap(),
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
            let documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
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
            let bundle = crate::authoring_driver::document::load_package_documents_owned(
                &texts,
                &rt.registry,
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
                &rt.validation_context().unwrap(),
            )
            .unwrap();
            let documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
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
