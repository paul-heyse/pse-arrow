// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public finite modeling packages are immutable inputs to the existing compiler service.
pub(super) mod cases;
mod conformance;
pub(super) mod declared;
mod path_pipeline;
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
use crate::math::{
    Workspace,
    modeling::{ModelingPreparation, ModelingRevision},
};
pub use analysis_tables::ModelingNativeAnalysis;
pub use cases::{ModelingObservations, ModelingSolvePreparation, StartSource};
use pse_authoring::language::Declaration;
use pse_compiler::workspace::{CompilerContext, WorkspaceLimits};
use pse_ids::SemanticId;
use pse_modeling::{Bindings, DeclarationId, InstanceId, Limits, PhysicalScope};
use pse_relations::columnar::RelationRow;
pub use results::{ModelingCheck, ModelingReport, ModelingResult};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
};

#[derive(Debug)]
struct SourceBatchIdentity {
    rows: usize,
    schema: Weak<datafusion::arrow::datatypes::Schema>,
    columns: Vec<Weak<dyn datafusion::arrow::array::Array>>,
}
impl SourceBatchIdentity {
    fn new(batch: &pse_relations::columnar::FieldCheckedBatch) -> Self {
        let batch = batch.batch();
        Self {
            rows: batch.num_rows(),
            schema: Arc::downgrade(batch.schema_ref()),
            columns: batch.columns().iter().map(Arc::downgrade).collect(),
        }
    }
    fn matches(&self, batch: &pse_relations::columnar::FieldCheckedBatch) -> bool {
        let batch = batch.batch();
        self.rows == batch.num_rows()
            && Weak::ptr_eq(&self.schema, &Arc::downgrade(batch.schema_ref()))
            && self.columns.len() == batch.num_columns()
            && self
                .columns
                .iter()
                .zip(batch.columns())
                .all(|(a, b)| Weak::ptr_eq(a, &Arc::downgrade(b)))
    }
}
#[derive(Debug)]
struct SourceExport {
    revision: pse_ids::roles::SourceRevisionHash,
    documents: Weak<crate::authoring_driver::document::Batches>,
    physical: pse_ids::ContentHash,
    physical_sources: BTreeMap<pse_schema::model::RelationKey, SourceBatchIdentity>,
    registry: Weak<pse_schema::Registry>,
    validation: Weak<pse_engine::session::EngineFactory>,
    pool: Weak<dyn pse_columnar::MemoryPool>,
    tables: BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>,
    _metadata: Arc<pse_columnar::AllocationLease>,
}
impl SourceExport {
    fn export_tables(
        &self,
        runtime: &Runtime,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        let owner = runtime.shared.math().reserve(
            "modeling:source-export-map",
            source_map_extent(self.tables.len())?,
        )?;
        let mut tables = self.tables.clone();
        for table in tables.values_mut() {
            *table = table.clone().with_export_owner(owner.clone());
        }
        Ok(tables)
    }
    fn matches(&self, package: &ModelingPackage) -> bool {
        self.revision == package.revision.identity()
            && Weak::ptr_eq(&self.documents, &Arc::downgrade(&package.document_sources))
            && self.physical == package.physical.key
            && Weak::ptr_eq(&self.registry, &Arc::downgrade(&package.runtime.registry))
            && Weak::ptr_eq(&self.validation, &Arc::downgrade(&package.runtime.sessions))
            && Weak::ptr_eq(&self.pool, &Arc::downgrade(&package.runtime.shared.pool()))
            && self.physical_sources.len() == package.physical.sources.len()
            && self.physical_sources.iter().all(|(key, cached)| {
                package
                    .physical
                    .sources
                    .get(key)
                    .is_some_and(|current| cached.matches(current))
            })
    }
}
pub(in crate::workflow) fn source_map_extent(tables: usize) -> Result<usize, WorkflowError> {
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
    pub(in crate::workflow) workspace: Workspace,
    pub(in crate::workflow) revision: ModelingRevision,
    document_sources: Arc<crate::authoring_driver::document::Batches>,
    source_export: Arc<Mutex<Option<SourceExport>>>,
    pub(in crate::workflow) fit_declarations: Arc<super::fitting::FitDeclarations>,
    accelerators: Arc<pse_math::implicit::accelerators::Accelerators>,
    providers: Arc<BTreeMap<String, pse_kernels::Registration>>,
    pub(in crate::workflow) physical: PhysicalContext,
    pub(in crate::workflow) quantities: Arc<pse_quantity::QuantityRegistry>,
}
/// Immutable admitted package payload; excludes runtime services and mutable workspaces.
#[derive(Clone, Debug)]
pub(crate) struct PackageAdmission {
    revision: ModelingRevision,
    sources: Arc<crate::authoring_driver::document::Batches>,
    fits: Arc<super::fitting::FitDeclarations>,
    accelerators: Arc<pse_math::implicit::accelerators::Accelerators>,
    providers: Arc<BTreeMap<String, pse_kernels::Registration>>,
    _lease: Arc<pse_columnar::AllocationLease>,
    _validation_owner: Arc<pse_engine::session::EngineFactory>,
    _registry_owner: Arc<pse_schema::Registry>,
}
impl PackageAdmission {
    pub(crate) fn retained_bytes(&self) -> usize {
        use pse_model::HeapUsage;
        self.revision.retained_bytes()
            + self.fits.fits.owned_bytes()
            + self
                .sources
                .values()
                .map(|source| source.batch().get_array_memory_size() + 128)
                .sum::<usize>()
            + size_of::<Self>()
            + 256
    }
}
impl ModelingPackage {
    pub(crate) fn admission(&self) -> Result<PackageAdmission, WorkflowError> {
        Ok(PackageAdmission {
            revision: self.revision.clone(),
            sources: self.document_sources.clone(),
            fits: self.fit_declarations.clone(),
            accelerators: self.accelerators.clone(),
            providers: self.providers.clone(),
            _lease: self.runtime.shared.math().reserve(
                "modeling:package-admission",
                size_of::<PackageAdmission>()
                    + 256
                    + self.document_sources.len() * 128
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
        let service = self.shared.math();
        let workspace = service.workspace(
            compiler_context(&physical, &admitted.providers),
            WorkspaceLimits::default(),
        )?;
        Ok(ModelingPackage {
            runtime: self.clone(),
            workspace,
            revision: admitted.revision,
            document_sources: admitted.sources,
            source_export: Default::default(),
            fit_declarations: admitted.fits,
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
/// Declarations, their physical-name scope, fit data, source batches decoded from the
/// documents and the package data documents (ADR-0125).
type DocumentCompilerContext = (
    Vec<Declaration>,
    PhysicalScope,
    super::FitDeclarations,
    crate::authoring_driver::document::Batches,
    Arc<pse_modeling::document::DocumentInventory>,
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
) -> Result<DocumentCompilerContext, WorkflowError> {
    documents.validate_registry(registry)?;
    let mut headers = documents
        .bundles()
        .iter()
        .map(|b| b.package.clone())
        .collect::<Vec<_>>();
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
    let batches = crate::authoring_driver::p1::source_batches(
        documents.bundles(),
        registry,
        &headers,
        limits,
    )?;
    use pse_relations::generated::authored::modeling_declarations as wire;
    let batch = batches
        .get(&wire::RELATION_ID)
        .ok_or_else(|| contract("documents contain no modeling declarations"))?;
    let rows = wire::Row::rows(batch).map_err(relation)?;
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
    ))
}
impl Runtime {
    /// Load the generated generic IR directly from admitted `.pse` documents.
    pub fn modeling_from_documents(
        &self,
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        let validation = self.sessions.validation_context(&self.registry)?;
        documents.validate_context(
            &self.registry,
            &validation,
            &pse_columnar::CancellationToken::new(),
        )?;
        let (rows, scope, data, sources, inventory) = document_inputs(
            documents,
            &self.registry,
            &physical,
            self.shared.budget().math.workspace_bytes,
        )?;
        let mut package = self
            .modeling_package_scoped(rows, physical, scope, inventory, BTreeMap::new())?
            .with_fit_declarations(data)?;
        let pool = self.shared.pool();
        let cancel = pse_columnar::CancellationToken::new();
        package.document_sources = Arc::new(
            sources
                .into_iter()
                .map(|(id, b)| {
                    b.retained(&pool, &cancel)
                        .map(|b| (id, b))
                        .map_err(relation)
                })
                .collect::<Result<_, _>>()?,
        );
        Ok(package)
    }
    /// Admit generated declarations without serializing through a text document. Rows
    /// admitted without manifests see the physical names everywhere.
    pub fn modeling_package(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.modeling_package_registered(rows, physical, BTreeMap::new())
    }
    /// Supply native capabilities explicitly; declaration strings never instantiate implementations.
    pub fn modeling_package_registered(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        let scope = PhysicalScope {
            package: physical.package.as_ref().map(|p| p.name.clone()),
            documents: None,
        };
        self.modeling_package_scoped(rows, physical, scope, Default::default(), providers)
    }
    fn modeling_package_scoped(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
        scope: PhysicalScope,
        documents: Arc<pse_modeling::document::DocumentInventory>,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        let service = self.shared.math();
        let inputs = compiler_context(&physical, &providers);
        let workspace = service.workspace(inputs, WorkspaceLimits::default())?;
        let revision =
            service.modeling_revision(&workspace, rows, scope, documents, &physical.key)?;
        Ok(ModelingPackage {
            runtime: self.clone(),
            workspace,
            revision,
            document_sources: Default::default(),
            source_export: Default::default(),
            fit_declarations: Arc::new(super::fitting::FitDeclarations::default()),
            accelerators: Arc::new(pse_math::implicit::accelerators::Accelerators::standard()),
            providers: Arc::new(providers),
            physical: physical.clone(),
            quantities: physical.quantities,
        })
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

    /// Original registry-generated source declarations, retaining their immutable revision owner.
    pub fn declarations(&self) -> &[Declaration] {
        self.revision.declarations()
    }
    /// Merge retained physical declarations with this source by semantic identity.
    /// Physical entity kinds share the modeling relation; an empty physical selection
    /// must never replace the model, and conflicting declarations cannot be published.
    pub(in crate::workflow) fn source_tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        let mut cache = self
            .source_export
            .lock()
            .map_err(|_| WorkflowError::Internal("modeling source export cache poisoned".into()))?;
        if let Some(export) = cache.as_ref().filter(|export| export.matches(self)) {
            return export.export_tables(&self.runtime);
        }
        // A changed input cannot reuse this product. Release its cache-only claims
        // before constructing the replacement; escaped tables retain their own owners.
        *cache = None;
        let mut tables = self.encode_source_tables()?;
        let pool = self.runtime.shared.pool();
        let cancel = pse_columnar::CancellationToken::new();
        for table in tables.values_mut() {
            *table = table.retained(&pool, &cancel).map_err(relation)?;
        }
        // Checked storage accounts its own shared column vectors. This grant covers
        // only the cache map and its exact-input identity inventory.
        let metadata = source_map_extent(tables.len())?
            .checked_add(size_of::<SourceExport>() + 256)
            .and_then(|n| {
                self.physical.sources.values().try_fold(n, |n, table| {
                    table
                        .batch()
                        .num_columns()
                        .checked_mul(size_of::<Weak<dyn datafusion::arrow::array::Array>>())
                        .and_then(|columns| n.checked_add(columns))
                        .and_then(|n| {
                            n.checked_add(
                                size_of::<SourceBatchIdentity>()
                                    + size_of::<pse_schema::model::RelationKey>()
                                    + 128,
                            )
                        })
                })
            })
            .ok_or_else(|| contract("source export cache extent"))?;
        let metadata = self
            .runtime
            .shared
            .math()
            .reserve("modeling:source-export-cache", metadata)?;
        let export = SourceExport {
            revision: self.revision.identity(),
            documents: Arc::downgrade(&self.document_sources),
            physical: self.physical.key,
            physical_sources: self
                .physical
                .sources
                .iter()
                .map(|(key, batch)| (*key, SourceBatchIdentity::new(batch)))
                .collect(),
            registry: Arc::downgrade(&self.runtime.registry),
            validation: Arc::downgrade(&self.runtime.sessions),
            pool: Arc::downgrade(&pool),
            _metadata: metadata,
            tables,
        };
        let tables = export.export_tables(&self.runtime)?;
        *cache = Some(export);
        Ok(tables)
    }
    fn encode_source_tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        use pse_model::HeapUsage;
        use pse_relations::generated::authored::modeling_declarations as wire;
        let registry = &self.runtime.registry;
        let validation = self.runtime.validation_context()?;
        let bytes = self
            .declarations()
            .iter()
            .map(HeapUsage::owned_bytes)
            .try_fold(4096usize, usize::checked_add)
            .and_then(|n| {
                self.physical
                    .sources
                    .values()
                    .chain(self.document_sources.values())
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
        let mut tables = self.document_sources.as_ref().clone();
        let mut physical = Vec::new();
        macro_rules! merge_context {
            ($module:ident, $key:expr, $batch:expr) => {{
                use pse_relations::generated::authored::$module as context;
                let mut rows = BTreeMap::new();
                for source in self
                    .document_sources
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
        for row in physical.iter().chain(self.declarations()) {
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
                self.workspace.clone(),
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
        let model = self
            .runtime
            .shared
            .math()
            .prepare_semantic_modeling_revision(
                self.workspace.clone(),
                self.revision.clone(),
                analysis.root,
                analysis.instance,
                analysis.bindings.clone(),
                analysis.limits,
                cancel,
            )
            .await?;
        Ok(self
            .runtime
            .native()
            .prepare_modeling_flow(model, self.quantities.clone(), selection, cancel)
            .await?)
    }
    /// Replace source declarations while preserving which documents see the physical
    /// names and the package data documents. No old document text is retained as the source
    /// of the edited IR.
    pub fn with_declarations(&self, rows: Vec<Declaration>) -> Result<Self, WorkflowError> {
        let revision = self.runtime.shared.math().modeling_revision(
            &self.workspace,
            rows,
            self.revision.physical_scope().clone(),
            self.revision.documents().clone(),
            &self.physical.key,
        )?;
        Ok(Self {
            // Direct IR edits provide no replacement document bytes. Preserve no
            // stale source text as the declaration of the new revision.
            document_sources: Default::default(),
            source_export: Default::default(),
            runtime: self.runtime.clone(),
            workspace: self.workspace.clone(),
            revision,
            fit_declarations: self.fit_declarations.clone(),
            accelerators: self.accelerators.clone(),
            providers: self.providers.clone(),
            physical: self.physical.clone(),
            quantities: self.quantities.clone(),
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
        Ok(self
            .runtime
            .shared
            .math()
            .prepare_modeling_revision(
                self.workspace.clone(),
                self.revision.clone(),
                root,
                instance,
                bindings,
                limits,
                cancel,
            )
            .await?)
    }
}

#[cfg(test)]
mod reuse_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_export_clones_reuse_owned_buffers_and_changed_inputs_rebuild() {
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
        let package = runtime
            .modeling_package(
                parse("package application {}"),
                super::super::tests::physical(),
            )
            .unwrap();
        let pool = runtime.shared.pool();
        let first = package.source_tables().unwrap();
        let retained = pool.reserved();
        let cloned = package.clone();
        let second = cloned.source_tables().unwrap();
        assert_eq!(
            pool.reserved() - retained,
            source_map_extent(second.len()).unwrap(),
            "only the new escaping map is charged; storage and buffers remain shared"
        );
        assert!(Arc::ptr_eq(
            &first[&wire::RELATION_ID].batch().columns()[0],
            &second[&wire::RELATION_ID].batch().columns()[0],
        ));
        let replacement = package
            .with_declarations(parse("package replacement {}"))
            .unwrap();
        let changed = replacement.source_tables().unwrap();
        let rows = wire::Row::rows(&changed[&wire::RELATION_ID]).unwrap();
        assert!(rows.iter().any(|row| row.name == "replacement"));
        assert!(!rows.iter().any(|row| row.name == "application"));

        // Another validation factory/pool cannot inherit this export's admission.
        let mut relocated = cloned;
        relocated.runtime = super::super::tests::runtime();
        let other = relocated.source_tables().unwrap();
        assert!(!Arc::ptr_eq(
            &first[&wire::RELATION_ID].batch().columns()[0],
            &other[&wire::RELATION_ID].batch().columns()[0],
        ));
        drop((package, replacement, relocated, second, changed, other));
        assert!(
            wire::Row::rows(&first[&wire::RELATION_ID])
                .unwrap()
                .iter()
                .any(|row| row.name == "application")
        );
    }

    #[test]
    fn checked_source_export_keeps_storage_and_map_charges_after_packages_drop() {
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
            .unwrap();
        let tables = package.source_tables().unwrap();
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
    async fn checked_source_table_outlives_joined_result_with_accounted_metadata() {
        use pse_relations::generated::authored::modeling_declarations as wire;
        let runtime = super::super::tests::runtime();
        let pool = runtime.shared.pool();
        let rows = pse_authoring::language::parse(
            "package escaped { def Root { param x:Scalar=2; eq fixed:x*x==4; } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, super::super::tests::physical())
            .unwrap();
        let mut compiler = super::super::tests::compiler_profile();
        compiler.assembly.worker_bytes = 2 << 20;
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                pse_kernels::DerivativeOrder::Value,
                compiler,
                super::super::tests::profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        assert!(result.usable());
        let escaped = result.table("authored.modeling_declarations").unwrap();
        drop(result);
        drop(prepared);
        drop(package);
        drop(runtime);
        assert!(pool.reserved() > source_map_extent(1).unwrap());
        assert!(
            wire::Row::rows(&escaped)
                .unwrap()
                .iter()
                .any(|row| row.name == "Root")
        );
        drop(escaped);
        assert_eq!(
            pool.reserved(),
            0,
            "checked source export was the last retained run owner"
        );
    }

    #[test]
    fn source_export_resource_failure_can_retry_without_poisoning_clones() {
        let runtime = super::super::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package retry {}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let package = runtime
            .modeling_package(rows, super::super::tests::physical())
            .unwrap();
        let pool = runtime.shared.pool();
        let available = runtime.shared.budget().memory_limit_bytes.get() - pool.reserved();
        let held = runtime
            .shared
            .math()
            .reserve("source-export-test:occupied", available)
            .unwrap();
        assert!(package.source_tables().is_err());
        drop(held);
        let first = package.clone().source_tables().unwrap();
        let second = package.source_tables().unwrap();
        assert!(!first.is_empty());
        assert_eq!(
            first.keys().collect::<Vec<_>>(),
            second.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn source_export_merges_physical_modeling_rows_and_rejects_conflicts() {
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
            let exported = package.source_tables().unwrap();
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
        assert!(package.source_tables().is_err());
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
        let package = rt.modeling_from_documents(&documents, physical).unwrap();
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
        let revised = package.with_declarations(updated).unwrap();
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
        assert!(pse_math::SharedAllocation::ptr_eq(
            &original.compiled().admitted,
            &changed.compiled().admitted
        ));
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
            assert!(
                rt.modeling_from_documents(&package(&dependent, source), physical.clone())
                    .is_ok(),
                "{source}"
            );
            let refused = rt
                .modeling_from_documents(&package(manifest, source), physical.clone())
                .unwrap_err()
                .to_string();
            assert!(refused.contains("unknown type"), "{source}: {refused}");
        }
    }
    #[test]
    fn modeling_closure_refuses_missing_conflicting_and_cyclic_dependencies() {
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
        let admit = |parts| {
            let documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                parts, &pool, &token,
            )
            .unwrap();
            rt.modeling_from_documents(&documents, physical.clone())
        };
        assert!(admit(vec![library.clone(), application.clone()]).is_ok());
        assert!(admit(vec![application.clone()]).is_err());
        assert!(admit(vec![library.clone(), library.clone(), application.clone()]).is_err());
        let cycle = package(
            81,
            "lib",
            &dependency(82, 1),
            "package lib {fn twice(x:Scalar)->Scalar=2*x;}",
        );
        assert!(admit(vec![cycle, application.clone()]).is_err());
        for (dependency_text, source_text) in [
            (dependency(81, 2), "package app {}"),
            (String::new(), "package app {use lib @\"1.0.0\";}"),
            (
                dependency(81, 1),
                "package app {fn f(x:Scalar)->Scalar=lib.twice(x);}",
            ),
        ] {
            let bad = package(82, "app", &dependency_text, source_text);
            assert!(admit(vec![library.clone(), bad]).is_err());
        }
    }
    /// ADR-0123 Outcome 7: an import resolves its target package by identity. A manifest
    /// dependency on another package at the same version grants nothing; the dependency on
    /// the target's identity must carry a requirement that admits the target's version, as
    /// the import's own typed requirement must.
    #[test]
    fn import_requires_dependency_by_identity() {
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
        let admit = |parts| {
            let documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                parts, &pool, &token,
            )
            .unwrap();
            rt.modeling_from_documents(&documents, physical.clone())
        };
        assert!(
            admit(vec![
                library.clone(),
                other.clone(),
                application(&[(91, 1)])
            ])
            .is_ok()
        );
        // The same version, depended on under another identity, is not the target.
        let refused = admit(vec![
            library.clone(),
            other.clone(),
            application(&[(93, 1)]),
        ])
        .unwrap_err()
        .to_string();
        assert!(refused.contains("exact manifest dependency"), "{refused}");
        // The typed requirement is checked against the manifest version.
        assert!(admit(vec![library, other, application(&[(91, 2)])]).is_err());
    }
    #[test]
    fn modeling_document_import_checks_the_admitted_version() {
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
                rt.modeling_from_documents(&documents, physical.clone())
                    .is_ok(),
                valid
            );
        }
    }
}

#[cfg(test)]
mod applicability_tests;
