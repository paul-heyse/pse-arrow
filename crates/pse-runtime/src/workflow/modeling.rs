// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public finite modeling packages are immutable inputs to the existing compiler service.
pub(super) mod cases;
mod conformance;
mod pure;
pub use conformance::{
    ModelingConformanceCheck, ModelingConformancePolicy, ModelingConformanceReport,
    ModelingFixtureSelection,
};
pub use pure::conform_pure_documents;
pub(super) mod dynamics;
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
#[cfg(test)]
#[cfg(all(feature = "solver-ipopt", feature = "solver-highs"))]
pub(in crate::workflow) mod sensitivity_tests;
pub use diagnostics::{
    DiagnosticSampleStop, ElasticObservation, ModelingDiagnosticPolicy,
    ModelingDiagnosticPreparation, ModelingDiagnosticSamples, ModelingDiagnostics,
    ModelingElasticAttempt, ModelingInfeasibilityCertificate, ModelingNonlinearExplanation,
    ModelingNonlinearPolicy,
};
#[cfg(feature = "solver-highs")]
pub use diagnostics::{ModelingJacobianOptimization, ModelingLinearDiagnostics};
mod engines;
pub use engines::{
    DiscreteInitialization, ModelingAnalysis, ModelingInitialization,
    ModelingInitializationAttempt, ModelingInitializationReport, ModelingInitializationStep,
    ModelingStudyPoint, ModelingStudyReport,
};
pub(super) mod analysis_tables;
pub(super) mod assessment;
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
use pse_compiler::workspace::{Inputs, WorkspaceLimits};
use pse_ids::SemanticId;
use pse_modeling::{Bindings, DeclarationId, InstanceId, Limits, PhysicalScope};
use pse_relations::columnar::RelationRow;
pub use results::{ModelingCheck, ModelingReport, ModelingResult};
use std::collections::BTreeMap;
/// Immutable source revision sharing one deployment-owned compiler workspace.
#[derive(Clone, Debug)]
pub struct ModelingPackage {
    pub(in crate::workflow) runtime: Runtime,
    pub(in crate::workflow) workspace: Workspace,
    pub(in crate::workflow) revision: ModelingRevision,
    document_sources: std::sync::Arc<crate::authoring_driver::document::Batches>,
    pub(in crate::workflow) fit_data: std::sync::Arc<super::fitting::FitData>,
    accelerators: std::sync::Arc<pse_math::implicit::accelerators::Accelerators>,
    providers: std::sync::Arc<BTreeMap<String, pse_kernels::Registration>>,
    pub(in crate::workflow) physical: PhysicalContext,
    pub(in crate::workflow) quantities: std::sync::Arc<pse_quantity::QuantityRegistry>,
    /// Prepared solver views and observation programs, shared by every analysis (A6).
    pub(in crate::workflow) views: std::sync::Arc<views::Views>,
}
fn compiler_inputs(
    physical: &PhysicalContext,
    providers: &BTreeMap<String, pse_kernels::Registration>,
) -> Inputs {
    Inputs {
        quantities: physical.quantities.clone(),
        preconditions: physical.preconditions.clone(),
        flows: BTreeMap::new(),
        definitions: BTreeMap::new(),
        domains: BTreeMap::new(),
        groups: BTreeMap::new(),
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
        cases: BTreeMap::new(),
        values: BTreeMap::new(),
    }
}
/// Declarations, their physical-name scope, fit data and source batches decoded from the
/// documents.
type DocumentInputs = (
    Vec<Declaration>,
    PhysicalScope,
    super::FitData,
    crate::authoring_driver::document::Batches,
);
fn document_inputs(
    documents: &crate::authoring_driver::document::OwnedDocumentSet,
    registry: &pse_schema::Registry,
    physical: &PhysicalContext,
    workspace_bytes: usize,
) -> Result<DocumentInputs, WorkflowError> {
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
    pse_authoring::p0::resolve_rows(
        &headers,
        pse_authoring::p0::GraphLimits {
            nodes: headers.len(),
            edges: workspace_bytes / 128,
        },
    )
    .map_err(crate::authoring_driver::DriverError::from)?;
    let batches = crate::authoring_driver::p1::source_batches(documents.bundles(), registry)?;
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
        super::FitData::from_batches(&batches)?,
        context,
    ))
}
impl Runtime {
    /// Load the generated generic IR directly from admitted `.pse` documents.
    pub fn modeling_from_documents(
        &self,
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        physical: PhysicalContext,
    ) -> Result<ModelingPackage, WorkflowError> {
        let (rows, scope, data, sources) = document_inputs(
            documents,
            &self.registry,
            &physical,
            self.shared.budget().math.workspace_bytes,
        )?;
        let mut package = self
            .modeling_package_scoped(rows, physical, scope, BTreeMap::new())?
            .with_fit_data(data)?;
        let pool = self.shared.pool();
        let cancel = pse_columnar::CancellationToken::new();
        package.document_sources = std::sync::Arc::new(
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
        self.modeling_package_scoped(rows, physical, scope, providers)
    }
    fn modeling_package_scoped(
        &self,
        rows: Vec<Declaration>,
        physical: PhysicalContext,
        scope: PhysicalScope,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        let service = self.shared.math();
        let inputs = compiler_inputs(&physical, &providers);
        let workspace = service.workspace(inputs, WorkspaceLimits::default())?;
        let revision = service.modeling_revision(&workspace, rows, scope, &physical.key)?;
        Ok(ModelingPackage {
            runtime: self.clone(),
            workspace,
            revision,
            document_sources: Default::default(),
            fit_data: std::sync::Arc::new(super::fitting::FitData::default()),
            accelerators: std::sync::Arc::new(
                pse_math::implicit::accelerators::Accelerators::standard(),
            ),
            providers: std::sync::Arc::new(providers),
            physical: physical.clone(),
            quantities: physical.quantities,
            views: Default::default(),
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
        self.accelerators = std::sync::Arc::new(accelerators);
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
        use pse_model::HeapUsage;
        use pse_relations::generated::authored::modeling_declarations as wire;
        let registry = &self.runtime.registry;
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
                    context::Builder::with_registry(registry, rows.len()).map_err(relation)?;
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
            wire::Builder::with_registry(registry, rows.len()).map_err(relation)?;
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
    /// Build an analysis from a data-authored case/test and its physical fixture.
    #[expect(
        clippy::too_many_arguments,
        reason = "a declared analysis binds its root and route with the compiler, solver and numerical profiles, limits and cancellation"
    )]
    pub async fn declared_analysis(
        &self,
        root: DeclarationId,
        route: pse_model::generated::enums::ModelingAnalysisRoute,
        compiler: pse_compiler::workspace::Profile,
        solver: crate::math::solves::SolverProfile,
        numerical: crate::math::solves::NumericalInputs,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingAnalysis, WorkflowError> {
        let (bindings, case) = self.declared_case(root, route, limits, cancel).await?;
        let order =
            if solver.controls.hessian == pse_backend_native::solve::HessianMode::LimitedMemory {
                pse_kernels::DerivativeOrder::First
            } else {
                pse_kernels::DerivativeOrder::Second
            };
        Ok(ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings,
            limits,
            case,
            order,
            compiler,
            solver,
            numerical,
        })
    }
    pub(in crate::workflow) async fn declared_case(
        &self,
        root: DeclarationId,
        route: pse_model::generated::enums::ModelingAnalysisRoute,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<(Bindings, pse_compiler::workspace::ModelingCaseBindings), WorkflowError> {
        let bindings = Bindings::default().with_analysis(route);
        let instance = pse_modeling::specialize::root_instance(root);
        let prepared = self
            .prepare(root, instance, bindings.clone(), limits, cancel)
            .await?;
        let case = prepared
            .compiled()
            .model
            .fixtures
            .get(&instance)
            .map(pse_compiler::workspace::ModelingCaseBindings::from)
            .unwrap_or_default();
        Ok((bindings, case))
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
            .prepare(
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
    /// names. No old document text is retained as the source of the edited IR.
    pub fn with_declarations(&self, rows: Vec<Declaration>) -> Result<Self, WorkflowError> {
        let revision = self.runtime.shared.math().modeling_revision(
            &self.workspace,
            rows,
            self.revision.physical_scope().clone(),
            &self.physical.key,
        )?;
        Ok(Self {
            // Direct IR edits provide no replacement document bytes. Preserve no
            // stale source text as the declaration of the new revision.
            document_sources: Default::default(),
            runtime: self.runtime.clone(),
            workspace: self.workspace.clone(),
            revision,
            fit_data: self.fit_data.clone(),
            accelerators: std::sync::Arc::new(
                pse_math::implicit::accelerators::Accelerators::standard(),
            ),
            providers: self.providers.clone(),
            physical: self.physical.clone(),
            quantities: self.quantities.clone(),
            views: Default::default(),
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
mod tests {
    use super::*;
    use std::sync::Arc;

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
            let mut b = wire::Builder::with_registry(&runtime.registry, rows.len()).unwrap();
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
                include_str!("../../../../tests/fixtures/packages/minimal_explicit/package.toml").to_owned(),
            ),
            ("models/kernel.pse".into(), source),
        ]);
        let token = pse_columnar::CancellationToken::new();
        let pool = rt.shared.pool();
        let bundle = crate::authoring_driver::document::load_package_texts_owned(
            &texts,
            &rt.registry,
            pse_authoring::ParseBudget::default(),
            &pool,
            &token,
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
        assert!(Arc::ptr_eq(
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
    fn primitives() -> BTreeMap<String, String> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/packages/physical-primitives");
        ["package.toml", "materials/physical.yaml", "materials/time.yaml"]
            .into_iter()
            .map(|path| {
                (
                    path.to_owned(),
                    std::fs::read_to_string(root.join(path)).unwrap(),
                )
            })
            .collect()
    }
    fn load(
        rt: &Runtime,
        texts: &BTreeMap<String, String>,
    ) -> Result<crate::authoring_driver::document::OwnedDocumentSet, WorkflowError> {
        let token = pse_columnar::CancellationToken::new();
        let pool = rt.shared.pool();
        let bundle = crate::authoring_driver::document::load_package_texts_owned(
            texts,
            &rt.registry,
            pse_authoring::ParseBudget::default(),
            &pool,
            &token,
        )?;
        Ok(crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
            vec![bundle],
            &pool,
            &token,
        )?)
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
        // The reference inventory (the generated standard registry) names 44 quantity
        // types (the 42 former manifest aliases plus D0's ChargeNumber and GasConstant)
        // and its four reference states.
        let standard = pse_quantity::standard::standard_registry().unwrap();
        let (types, states): (Vec<_>, Vec<_>) = standard
            .physical_names()
            .partition(|(_, n)| matches!(n, pse_quantity::PhysicalName::QuantityType(_)));
        assert_eq!((types.len(), states.len()), (44, 4));
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
            include_str!("../../../../tests/fixtures/packages/minimal_named/package.toml")
                .to_owned()
                + "\n[[quantity_aliases]]\nname = \"Scalar\"\nquantity_type_id = \"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f\"\n",
        )]);
        let refused = load(&rt, &aliased).unwrap_err().to_string();
        assert!(refused.contains("quantity_aliases"), "{refused}");
        aliased.clear();
        // Naming a second type `Scalar` in the physical document is refused.
        let mut twice = primitives();
        let time = twice.get_mut("materials/time.yaml").unwrap();
        *time = time.replacen("\"name\": \"Time\"", "\"name\": \"Scalar\"", 1);
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
        let manifest = include_str!("../../../../tests/fixtures/packages/minimal_named/package.toml");
        let dependent = manifest.replace(
            "dependencies = []",
            r#"dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#,
        );
        let package = |manifest: &str, source: &str| {
            load(
                &rt,
                &BTreeMap::from([
                    ("package.toml".to_owned(), manifest.to_owned()),
                    ("models/root.pse".to_owned(), source.to_owned()),
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
            crate::authoring_driver::document::load_package_texts_owned(
                &BTreeMap::from([
                    ("package.toml".into(), manifest),
                    ("models/kernel.pse".into(), source.into()),
                ]),
                &rt.registry,
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
            )
            .unwrap()
        };
        let dependency = |id, major| {
            format!(
                "{{package_id=\"{}\",version_req={{operator=\"exact\",major={major},minor=0,patch=0}}}}",
                SemanticId::from_bytes([id; 16])
            )
        };
        let library = package(81, "lib", "", "package lib {fn twice(x:Scalar)->Scalar=2*x;}");
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
            crate::authoring_driver::document::load_package_texts_owned(
                &BTreeMap::from([
                    ("package.toml".into(), manifest),
                    ("models/kernel.pse".into(), source.into()),
                ]),
                &rt.registry,
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
            )
            .unwrap()
        };
        let library = package(91, "lib", &[], "package lib {fn twice(x:Scalar)->Scalar=2*x;}");
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
        assert!(admit(vec![library.clone(), other.clone(), application(&[(91, 1)])]).is_ok());
        // The same version, depended on under another identity, is not the target.
        let refused = admit(vec![library.clone(), other.clone(), application(&[(93, 1)])])
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
                    include_str!("../../../../tests/fixtures/packages/minimal_explicit/package.toml").to_owned(),
                ),
                ("models/kernel.pse".into(), source),
            ]);
            let token = pse_columnar::CancellationToken::new();
            let pool = rt.shared.pool();
            let bundle = crate::authoring_driver::document::load_package_texts_owned(
                &texts,
                &rt.registry,
                pse_authoring::ParseBudget::default(),
                &pool,
                &token,
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
