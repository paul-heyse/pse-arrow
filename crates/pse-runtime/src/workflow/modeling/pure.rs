// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Standalone pure conformance shares admission and compiler semantics, without MathService.
use super::conformance::{ModelingFixtureSelection, NO_FIXTURE};
use super::declared::declared_limits;
use super::*;
use pse_columnar::{AllocationLease, MemoryConsumer};
use pse_model::generated::enums::{
    ModelingConformanceKind as Kind, ModelingConformanceStatus as Status,
    ModelingProcedure as Procedure,
};
use std::{collections::BTreeSet, sync::Arc};

/// Explicit ephemeral diagnostic: run exclusively pure authored tests without
/// constructing a normal source package, canonical revision or native-solver services.
/// Physical data still passes through its authoritative relational admission boundary.
#[expect(
    clippy::too_many_arguments,
    reason = "the documents, physical data, budget and planner accompany the fixture selection and caps, limits and cancellation"
)]
pub async fn conform_pure_documents(
    documents: Vec<BTreeMap<String, Vec<u8>>>,
    physical_documents: BTreeMap<String, Vec<u8>>,
    budget: crate::ResourceBudget,
    selection: ModelingFixtureSelection,
    maximum_fixtures: usize,
    maximum_checks: usize,
    preparation: crate::workflow::PreparationSettings,
    cancel: &crate::CancelSource,
) -> Result<ModelingConformanceReport, WorkflowError> {
    if maximum_fixtures == 0
        || maximum_fixtures > 4096
        || maximum_checks == 0
        || maximum_checks > 100_000
    {
        return Err(contract("bounded conformance policy"));
    }
    budget
        .validate()
        .map_err(|e| pse_engine::EngineError::Semantic(Arc::new(e)))?;
    let registry = pse_schema::shared_registry()
        .map_err(pse_relations::RelationError::from)
        .map_err(relation)?;
    let resources = pse_engine::resources::EngineResources::build(
        budget.memory_limit_bytes,
        budget.top_consumers,
        budget.spill_dir.clone(),
        budget.max_temp_dir_bytes,
        budget.cache.clone(),
    )?;
    let sessions = pse_engine::session::EngineFactory::new(
        resources.runtime.clone(),
        resources.pool.clone(),
        budget.execution.clone(),
        budget.threads,
        pse_engine::session::native_engine_profile(),
    )?
    .with_cache_service(resources.caches.clone());
    let model = Arc::new(sessions).bind_model(registry.clone())?;
    let validation = model.validation_context();
    let token = cancel.token();
    let load = |documents: &[BTreeMap<String, Vec<u8>>]| {
        let bundles = documents
            .iter()
            .map(|sources| {
                pse_admission::authoring_driver::document::load_package_documents_owned(
                    sources,
                    &registry,
                    pse_authoring::ParseBudget::default(),
                    &resources.pool,
                    &token,
                    &validation,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
            bundles,
            &resources.pool,
            &token,
        )
    };
    let physical =
        PhysicalContext::from_documents(&load(&[physical_documents])?, &model, &token).await?;
    let (rows, names, _fit_declarations, _sources, data_documents, _input_lease) = document_inputs(
        &load(&documents)?,
        &registry,
        &physical,
        budget.math.workspace_bytes,
        &resources.pool,
        &token,
    )?;
    let reserve =
        |name: &'static str, bytes: usize| -> Result<Arc<AllocationLease>, WorkflowError> {
            let allocation = MemoryConsumer::new(name).register(&resources.pool);
            allocation
                .try_grow(bytes)
                .map_err(pse_columnar::CanonError::from)
                .map_err(pse_relations::RelationError::from)
                .map_err(relation)?;
            Ok(AllocationLease::new(allocation))
        };
    let workspace_owner = reserve(
        "pure-conformance:compiler",
        budget
            .math
            .workspace_bytes
            .checked_add(budget.math.worker_bytes)
            .ok_or_else(|| contract("compiler extent"))?,
    )?;
    // An unknown selection is refused before the compiler workspace or any fixture runs.
    let fixture_ids = selection
        .tests(&rows)?
        .into_iter()
        .map(|r| r.declaration_id)
        .collect::<Vec<_>>();
    let mut report = ModelingConformanceReport::new(
        registry,
        resources.pool.clone(),
        validation,
        &fixture_ids,
        maximum_checks,
    )?;
    report.selection = selection;
    let flag = pse_diagnostics::Cancellation::new(token.is_cancelled());
    let worker_flag = flag.clone();
    let task = tokio::task::spawn_blocking(
        move || -> Result<ModelingConformanceReport, WorkflowError> {
            let _workspace_owner = workspace_owner;
            let inputs = compiler_context(&physical, &BTreeMap::new());
            let mut compiler = pse_compiler::workspace::CompilerWorkspace::new(
                inputs,
                WorkspaceLimits {
                    input_bytes: budget.math.workspace_bytes / 2,
                    retained_bytes: budget.math.workspace_bytes / 2,
                    ..WorkspaceLimits::default()
                },
            )
            .map_err(crate::math::MathRuntimeError::from)?;
            let revision = compiler
                .publish_modeling_with(rows, names, data_documents)
                .map_err(crate::math::MathRuntimeError::from)?;
            let fixtures = report.selection.tests(revision.declarations())?;
            if fixtures.is_empty() {
                report.record_fixture(
                    NO_FIXTURE,
                    Kind::Coverage,
                    Status::Failed,
                    "package contains no authored tests",
                    None,
                    maximum_checks,
                );
            }
            // Every fixture's declared allowances are resolved, and refused, before any
            // fixture runs (ADR-0119).
            let fixture_limits = fixtures
                .iter()
                .map(|row| declared_limits(row, preparation.limits))
                .collect::<Result<Vec<_>, _>>()?;
            let mut covered = BTreeSet::new();
            for (index, (row, limits)) in fixtures.iter().zip(fixture_limits).enumerate() {
                let fixture = row.declaration_id;
                let oracle = revision.oracle(fixture);
                report.note_oracle(oracle, |oracle| revision.release_of(oracle));
                let data = row.value.scope.as_ref().and_then(|s| s.fixture.as_ref());
                if index >= maximum_fixtures || worker_flag.is_cancelled() || !report.complete {
                    report.complete = false;
                    report.record_fixture(
                        fixture,
                        Kind::Preparation,
                        Status::Unattempted,
                        "fixture was not attempted",
                        oracle,
                        maximum_checks,
                    );
                    continue;
                }
                if data.and_then(|f| f.procedure) != Some(Procedure::Check) {
                    report.record_fixture(
                        fixture,
                        Kind::Preparation,
                        Status::Failed,
                        "standalone pure execution requires an explicit procedure check fixture",
                        oracle,
                        maximum_checks,
                    );
                    continue;
                }
                let bindings = Bindings::default();
                let model = match compiler.prepare_modeling_cancellable(
                    fixture,
                    pse_modeling::specialize::root_instance(fixture),
                    bindings.clone(),
                    limits,
                    worker_flag.flag(),
                ) {
                    Ok(model) => model,
                    Err(error) => {
                        report.failed(
                            fixture,
                            Kind::Preparation,
                            &crate::math::MathRuntimeError::from(error).into(),
                            // An expected failure is resolved with the fixture's model
                            // (Plan 23 H5).
                            None,
                            oracle,
                            maximum_checks,
                        );
                        continue;
                    }
                };
                covered.extend(model.model.instances.values().map(|i| i.definition));
                report.note_units(fixture, &model.model);
                let checked = compiler
                    .check_prepared_modeling_point(
                        &model,
                        fixture,
                        pse_modeling::specialize::root_instance(fixture),
                        &pse_math::binding::CaseValues {
                            scalars: BTreeMap::new(),
                        },
                        preparation.compiler,
                        worker_flag.clone(),
                    )
                    .map_err(crate::math::MathRuntimeError::from)
                    .map_err(WorkflowError::from);
                report.pure_result(fixture, &model, checked, maximum_checks);
            }
            report.coverage(revision.declarations(), &covered, maximum_checks);
            Ok(report)
        },
    );
    join_compiler(task, cancel, &flag).await
}

async fn join_compiler<T>(
    task: tokio::task::JoinHandle<Result<T, WorkflowError>>,
    cancel: &crate::CancelSource,
    flag: &pse_diagnostics::Cancellation,
) -> Result<T, WorkflowError> {
    tokio::pin!(task);
    let joined = tokio::select! { value = &mut task => value, () = cancel.cancelled() => {
        flag.cancel(); task.await
    }};
    joined.map_err(|source| crate::math::MathRuntimeError::WorkerJoin {
        operation: "pure compiler",
        source,
    })?
}

#[cfg(test)]
mod compiler_join_tests {
    use super::*;
    #[tokio::test]
    async fn compiler_worker_join_preserves_panic_and_cancellation_and_drains_original_worker() {
        use pse_model::diagnostic::BoundaryClass;
        let cancel = crate::CancelSource::new();
        let flag = pse_diagnostics::Cancellation::new(false);
        let task = tokio::task::spawn_blocking(|| -> Result<(), WorkflowError> {
            panic!("private compiler panic payload")
        });
        let error = join_compiler(task, &cancel, &flag).await.unwrap_err();
        let diagnostic = error.boundary_diagnostic();
        assert_eq!(diagnostic.class, BoundaryClass::Internal);
        assert_eq!(
            diagnostic.code,
            pse_diagnostics::DiagnosticCode::WorkflowPanic
        );
        assert!(!format!("{diagnostic:?}").contains("private compiler panic payload"));
        assert!(
            matches!(error, WorkflowError::Math(crate::math::MathRuntimeError::WorkerJoin { operation: "pure compiler", source }) if source.is_panic())
        );

        let task = tokio::spawn(std::future::pending::<Result<(), WorkflowError>>());
        task.abort();
        let error = join_compiler(task, &cancel, &flag).await.unwrap_err();
        assert_eq!(error.boundary_diagnostic().class, BoundaryClass::Cancelled);

        let drained = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let worker_flag = flag.clone();
        let worker_drained = drained.clone();
        let task = tokio::task::spawn_blocking(move || {
            while !worker_flag.is_cancelled() {
                std::thread::yield_now();
            }
            worker_drained.store(true, std::sync::atomic::Ordering::Release);
            Err::<(), _>(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled,
            ))
        });
        cancel.cancel();
        let error = join_compiler(task, &cancel, &flag).await.unwrap_err();
        assert_eq!(error.boundary_diagnostic().class, BoundaryClass::Cancelled);
        assert!(drained.load(std::sync::atomic::Ordering::Acquire));
    }
}
