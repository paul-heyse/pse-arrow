// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared typed project operations, using the original native workflow owners.
//!
//! The JSON values here are the existing native operator presentation. Generated
//! service responses select their typed owner results at their own boundary.
use super::{ProjectAction as Action, ProjectOperationRequest as Request};
use crate::{
    CancelSource, SharedRuntime,
    workflow::{
        CanonicalDeployment, ModelingPackage, OuterAttestation, PhysicalContext,
        ProjectBundleReader, ProjectDraftDocuments, Runtime,
    },
};
use pse_operations::{canonical::CanonicalStore, canonical_contexts::ContextSelection};
use serde::Serialize;
use std::{collections::BTreeMap, io::Read, path::PathBuf, sync::Arc};

mod error;
pub use error::ProjectOperationError;
use error::{Context, Result, ensure};

fn value<T: Serialize>(value: T) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(value)?)
}
fn boxed<'a, F: Future<Output = Result<serde_json::Value>> + Send + 'a>(
    shared: &SharedRuntime,
    operation: F,
) -> Result<std::pin::Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a>>> {
    let bytes = size_of::<F>()
        .checked_add(512)
        .context("operator future extent overflow")?;
    let owner = datafusion::execution::memory_pool::MemoryConsumer::new("project:operation-future")
        .register(&shared.pool());
    owner.try_grow(bytes)?;
    Ok(Box::pin(async move {
        let _owner = owner;
        operation.await
    }))
}
fn package(value: &ModelingPackage) -> Result<serde_json::Value> {
    #[derive(Serialize)]
    struct Selected<'a> {
        revision_id: &'a str,
        canonical_revision: Option<&'a str>,
        canonical_problem: Option<&'a str>,
        context: Option<&'a ContextSelection>,
    }
    self::value(Selected {
        revision_id: value.revision_id(),
        canonical_revision: value.canonical_revision().map(|r| r.key.as_str()),
        canonical_problem: value.canonical_revision().map(|r| r.problem.as_str()),
        context: value.project_context(),
    })
}
fn documents(
    runtime: &Runtime,
    shared: &SharedRuntime,
    input: &ProjectDraftDocuments,
    cancel: &CancelSource,
) -> Result<pse_admission::authoring_driver::document::OwnedDocumentSet> {
    let registry = pse_schema::shared_registry()?;
    let pool = shared.pool();
    let token = cancel.token();
    let validation = runtime.validation_context();
    let bundles = input
        .packages
        .iter()
        .map(|sources| {
            pse_admission::authoring_driver::document::load_package_documents_owned(
                sources,
                &registry,
                Default::default(),
                &pool,
                &token,
                &validation,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
            bundles, &pool, &token,
        )?,
    )
}

/// Admit physical documents through the same owned document loader used by the CLI.
/// # Errors
/// Preserves original source decoding, physical admission and resource failures.
pub async fn admit_project_physical(
    runtime: &Runtime,
    shared: &SharedRuntime,
    input: &ProjectDraftDocuments,
    cancel: &CancelSource,
) -> std::result::Result<PhysicalContext, ProjectOperationError> {
    let result: Result<PhysicalContext> = async {
        Ok(runtime
            .physical_from_documents(&documents(runtime, shared, input, cancel)?, &cancel.token())
            .await?)
    }
    .await;
    result.map_err(ProjectOperationError::action)
}

/// Admit exact authored source against the caller's already admitted physical context.
/// # Errors
/// Preserves the original loader and modeling admission failures without JSON conversion.
pub async fn admit_project_documents(
    runtime: &Runtime,
    shared: &SharedRuntime,
    input: &ProjectDraftDocuments,
    physical: PhysicalContext,
    cancel: &CancelSource,
) -> std::result::Result<ModelingPackage, ProjectOperationError> {
    let result: Result<ModelingPackage> = async {
        Ok(runtime
            .modeling_from_documents(
                &documents(runtime, shared, input, cancel)?,
                physical,
                cancel,
            )
            .await?)
    }
    .await;
    result.map_err(ProjectOperationError::action)
}

fn file_documents(
    runtime: &Runtime,
    shared: &SharedRuntime,
    roots: &[PathBuf],
    cancel: &CancelSource,
) -> Result<pse_admission::authoring_driver::document::OwnedDocumentSet> {
    ensure!(
        (1..=16).contains(&roots.len()),
        "select 1..=16 explicit package directories"
    );
    let registry = pse_schema::shared_registry()?;
    let validation = runtime.validation_context();
    let token = cancel.token();
    let mut bundles = Vec::with_capacity(roots.len());
    for root in roots {
        token.checkpoint()?;
        // The existing package loader owns declared paths, byte/row budgets,
        // symlink refusal, original Parquet decoding and physical admission.
        bundles.push(pse_admission::authoring_driver::document::load_package(
            root,
            &registry,
            Default::default(),
            &validation,
        )?);
    }
    Ok(
        pse_admission::authoring_driver::document::load_bundles_owned(
            &bundles,
            &registry,
            &shared.pool(),
            &token,
            &validation,
        )?,
    )
}
async fn selected(
    runtime: &Runtime,
    store: &CanonicalStore,
    key: &str,
    physical: &Option<PhysicalContext>,
) -> Result<ModelingPackage> {
    let revision = store
        .revision(key)
        .await?
        .context("exact source revision is unavailable")?;
    Ok(runtime
        .modeling_revision(
            revision,
            physical
                .clone()
                .context("physical documents required for scientific action")?,
            BTreeMap::new(),
        )
        .await?)
}
fn require_physical(physical: &Option<PhysicalContext>) -> Result<PhysicalContext> {
    physical
        .clone()
        .context("physical documents required for scientific action")
}
fn bytes(path: &std::path::Path, limit: usize) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    ensure!(file.metadata()?.is_file(), "regular body file required");
    let limit = u64::try_from(limit)?
        .checked_add(1)
        .context("body transport allowance overflow")?;
    let mut content = Vec::new();
    file.take(limit).read_to_end(&mut content)?;
    ensure!(
        u64::try_from(content.len())? < limit,
        "body exceeds runtime transport allowance"
    );
    Ok(content)
}
/// Execute an operational-only action without building a scientific runtime.
///
/// `None` leaves the original request untouched for scientific dispatch.
/// # Errors
/// Returns the original store, response reservation or presentation failure.
pub async fn operational_project_operation(
    store: &CanonicalStore,
    request: &Request,
    memory_limit_bytes: usize,
) -> std::result::Result<Option<serde_json::Value>, ProjectOperationError> {
    let Action::StudyWorkbench { request: browse } = &request.action else {
        return Ok(None);
    };
    let action: Result<serde_json::Value> = async {
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(memory_limit_bytes));
        let response =
            pse_columnar::MemoryConsumer::new("project:study-workbench-response").register(&pool);
        response.try_grow(pse_operations::canonical::MetadataResponse::Page.allocation_bytes())?;
        pse_operations::canonical::response_ownership_scope(Arc::new(response), async {
            value(store.control_study_workbench_page(browse.clone()).await?)
        })
        .await
    }
    .await;
    action.map(Some).map_err(ProjectOperationError::action)
}

/// Run one original request with explicit configured resources and native settlement.
///
/// Operational browsing returns before constructing sessions or a scientific runtime.
/// The caller owns process signals and the canonical connection; this owner closes the
/// runtime and joins its original native lanes before returning, retaining both failures.
/// # Errors
/// Returns the original action failure, native drain failure, or both independently.
pub async fn execute_project_operation(
    store: &CanonicalStore,
    request: Request,
    budget: crate::ResourceBudget,
    attestation: OuterAttestation,
    cancel: &CancelSource,
) -> std::result::Result<serde_json::Value, ProjectOperationError> {
    if let Some(response) =
        operational_project_operation(store, &request, budget.memory_limit_bytes.get()).await?
    {
        return Ok(response);
    }
    let build: Result<(Arc<SharedRuntime>, Runtime)> = async {
        let shared = SharedRuntime::build(budget)?;
        let registry = pse_schema::shared_registry()?;
        let sessions =
            Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
        let runtime = Runtime::from_shared(
            shared.clone(),
            sessions.bind_model(registry)?,
            CanonicalDeployment::new(store.clone(), attestation, None),
        )?;
        Ok((shared, runtime))
    }
    .await;
    let (shared, runtime) = build.map_err(ProjectOperationError::action)?;
    let result = dispatch(&runtime, &shared, store, request, cancel).await;
    runtime.close();
    let cleanup = shared.math().shutdown_native_lanes().await;
    error::settle_action(result, cleanup)
}

/// Dispatch against an already owned shared scientific runtime.
///
/// A service can reuse its existing runtime and canonical connection. The caller retains
/// the original operation's lifetime and must close and drain those owners explicitly.
/// # Errors
/// Returns original admission, source/context, workflow, resource and output failures.
pub async fn dispatch_project_operation(
    runtime: &Runtime,
    shared: &SharedRuntime,
    store: &CanonicalStore,
    request: Request,
    cancel: &CancelSource,
) -> std::result::Result<serde_json::Value, ProjectOperationError> {
    if let Some(response) =
        operational_project_operation(store, &request, shared.budget().memory_limit_bytes.get())
            .await?
    {
        return Ok(response);
    }
    dispatch(runtime, shared, store, request, cancel)
        .await
        .map_err(ProjectOperationError::action)
}

async fn dispatch(
    runtime: &Runtime,
    shared: &SharedRuntime,
    store: &CanonicalStore,
    request: Request,
    cancel: &CancelSource,
) -> Result<serde_json::Value> {
    let memory_limit_bytes = shared.budget().memory_limit_bytes.get();
    let descriptive = matches!(&request.action,Action::ProjectClosure{request} | Action::ProjectExport{request,..} if request.descriptive_only);
    let physical = if let Some(input) = request.physical.filter(|_| !descriptive) {
        Some(admit_project_physical(runtime, shared, &input, cancel).await?)
    } else {
        None
    };
    let effects = request.action.effect_facts();
    tracing::debug!(target: "pse.project.operator", scope = ?effects.scope, read_only = effects.read_only,
            idempotent = effects.idempotent, destructive = effects.destructive, "selected original project operation effects");
    // Erase each operation's deeply nested source/preparation future separately.
    // A generic action must not reserve every workflow's stack frame at once.
    let operation: std::pin::Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + '_>> =
        match request.action {
            Action::ProjectClosure { request } => boxed(shared, async move {
                if request.descriptive_only {
                    value(runtime.project_closure_metadata(&request, cancel).await?)
                } else {
                    value(
                        runtime
                            .project_closure(&request, require_physical(&physical)?, cancel)
                            .await?,
                    )
                }
            })?,
            Action::ProjectExport {
                destination,
                request,
            } => boxed(shared, async move {
                if request.descriptive_only {
                    value(
                        runtime
                            .project_export_metadata(&destination, &request, cancel)
                            .await?,
                    )
                } else {
                    value(
                        runtime
                            .project_export(
                                &destination,
                                &request,
                                require_physical(&physical)?,
                                cancel,
                            )
                            .await?,
                    )
                }
            })?,
            Action::ProjectImportIssue {
                input,
                title,
                branch_name,
                maximum_document_bytes,
            } => boxed(shared, async move {
                cancel.token().checkpoint()?;
                let bundle = ProjectBundleReader::open(&input, maximum_document_bytes)?;
                cancel.token().checkpoint()?;
                value(
                    runtime
                        .project_import_issue(&bundle, &title, &branch_name)
                        .await?,
                )
            })?,
            Action::ProjectImportStatus { intent } => boxed(shared, async move {
                value(runtime.project_import_status(&intent).await?)
            })?,
            Action::ProjectImport {
                input,
                intent,
                maximum_document_bytes,
            } => boxed(shared, async move {
                cancel.token().checkpoint()?;
                let bundle = ProjectBundleReader::open(&input, maximum_document_bytes)?;
                cancel.token().checkpoint()?;
                value(runtime.project_import(&bundle, &intent, cancel).await?)
            })?,
            Action::ProjectImportRecover { intent, original } => boxed(shared, async move {
                value(
                    runtime
                        .project_import_recover(&intent, &original, cancel)
                        .await?,
                )
            })?,
            Action::ProjectImportOpen { intent } => boxed(shared, async move {
                package(&runtime.project_import_open(&intent, cancel).await?)
            })?,
            Action::LifecycleSelect { target } => boxed(shared, async move {
                value(runtime.lifecycle_select(&target).await?)
            })?,
            Action::LifecycleExplain { selection } => boxed(shared, async move {
                value(runtime.lifecycle_explain(&selection).await?)
            })?,
            Action::LifecycleRelease { selection } => boxed(shared, async move {
                value(runtime.lifecycle_release(&selection).await?)
            })?,
            Action::ResearchDatasetCreate { request } => boxed(shared, async move {
                value(runtime.research_dataset_create(&request, cancel).await?)
            })?,
            Action::ResearchSelectionCreate { request } => boxed(shared, async move {
                value(runtime.research_selection_create(&request, cancel).await?)
            })?,
            Action::ResearchRead { request } => boxed(shared, async move {
                value(runtime.research_read(&request).await?)
            })?,
            Action::ResearchDatasetOpen { key, context } => boxed(shared, async move {
                package(
                    &runtime
                        .research_dataset_open(&key, &context, cancel)
                        .await?,
                )
            })?,
            Action::ResearchSelectionOpen { key } => boxed(shared, async move {
                let (source, fit) = runtime.research_selection_open(&key, cancel).await?;
                Ok(serde_json::json!({"package": package(&source)?, "fit": fit}))
            })?,
            Action::ResearchPublicationPreview { request } => boxed(shared, async move {
                value(
                    runtime
                        .research_publication_preview(&request, cancel)
                        .await?,
                )
            })?,
            Action::ResearchPublish { intent } => boxed(shared, async move {
                value(runtime.research_publish(&intent, cancel).await?)
            })?,
            Action::ResearchPublicationOpen { key } => boxed(shared, async move {
                package(&runtime.research_publication_open(&key, cancel).await?)
            })?,
            Action::StudyWorkbench { .. } => {
                return Err(
                    "operational browse was not dispatched before scientific construction".into(),
                );
            }
            Action::StudyCohort {
                request,
                output,
                maximum_bytes,
            } => boxed(shared, async move {
                let cohort = runtime.freeze_study_cohort(request).await?;
                let bytes = if let Some(output) = &output {
                    Some(
                        cohort
                            .export_pages(output, maximum_bytes, &cancel.token())
                            .await?,
                    )
                } else {
                    None
                };
                Ok(serde_json::json!({"receipt":cohort.receipt(),"output":output,"bytes":bytes}))
            })?,
            Action::StudyAnalyze {
                request,
                kpi,
                output,
            } => boxed(shared, async move {
                value(
                    runtime
                        .export_study_analysis(request, &kpi, &output, &cancel.token())
                        .await?,
                )
            })?,
            Action::StudyResult {
                request,
                member,
                relation,
                start,
                end,
                output,
                maximum_bytes,
            } => boxed(shared, async move {
                let cohort = runtime.freeze_study_cohort(request).await?;
                let mut reader = cohort
                    .result(member, &relation, start, end, cancel.token())
                    .await?;
                let bytes = reader
                    .export_ipc_bounded(&output, maximum_bytes, &cancel.token())
                    .await?;
                Ok(
                    serde_json::json!({"cohort": cohort.receipt(), "member": member, "output": output, "bytes": bytes}),
                )
            })?,
            Action::StudyObserve {
                request,
                cursor,
                delay_ms,
            } => boxed(shared, async move {
                let mut observer = runtime.observe_study(request, cursor).await?;
                let page = if let Some(ms) = delay_ms {
                    ensure!(ms <= 60_000, "observation delay exceeds 60 seconds");
                    observer
                        .follow(std::time::Duration::from_millis(ms))
                        .await?
                } else {
                    observer.poll().await?
                };
                observer.close();
                value(page)
            })?,
            Action::Import { documents: input } => boxed(shared, async move {
                package(
                    &admit_project_documents(
                        runtime,
                        shared,
                        &input,
                        require_physical(&physical)?,
                        cancel,
                    )
                    .await?,
                )
            })?,
            Action::ImportFiles {
                packages,
                physical_packages,
            } => boxed(shared, async move {
                let physical_documents =
                    file_documents(runtime, shared, &physical_packages, cancel)?;
                let selected_physical = runtime
                    .physical_from_documents(&physical_documents, &cancel.token())
                    .await?;
                let sources = file_documents(runtime, shared, &packages, cancel)?;
                package(
                    &runtime
                        .modeling_from_documents(&sources, selected_physical, cancel)
                        .await?,
                )
            })?,
            Action::Load { revision } => boxed(shared, async move {
                package(&selected(runtime, store, &revision, &physical).await?)
            })?,
            Action::Select { request } => boxed(shared, async move {
                value(
                    runtime
                        .project_select(&request, require_physical(&physical)?, cancel)
                        .await?,
                )
            })?,
            Action::OpenSelection { selection } => boxed(shared, async move {
                package(
                    &runtime
                        .project_open_selection(&selection, require_physical(&physical)?, cancel)
                        .await?,
                )
            })?,
            Action::Fork {
                project,
                branch,
                problem,
                operation,
                source,
            } => boxed(shared, async move {
                {
                    let source = selected(runtime, store, &source, &physical).await?;
                    value(
                        runtime
                            .project_fork(&project, &branch, &problem, &operation, &source)
                            .await?
                            .key,
                    )
                }
            })?,
            Action::DraftRead { key } => boxed(shared, async move {
                value(runtime.project_draft_bytes(&key, cancel).await?.to_vec())
            })?,
            Action::DraftIssue {
                operation,
                project,
                branch,
                key,
                base,
                expected,
                input,
            } => boxed(shared, async move {
                {
                    let content = bytes(&input, memory_limit_bytes / 4)?;
                    value(
                        runtime
                            .project_draft_save_intent(
                                &operation, &project, &branch, &key, &base, expected, &content,
                            )
                            .await?,
                    )
                }
            })?,
            Action::DraftSave { intent, input } => boxed(shared, async move {
                {
                    let content = bytes(&input, memory_limit_bytes / 4)?;
                    value(runtime.project_save_draft_intent(&intent, &content).await?)
                }
            })?,
            Action::DraftAdmit { operation, key } => boxed(shared, async move {
                package(
                    &runtime
                        .project_admit_draft(&operation, &key, require_physical(&physical)?, cancel)
                        .await?,
                )
            })?,
            Action::DraftAdmitVersion { version } => boxed(shared, async move {
                package(
                    &runtime
                        .project_admit_draft_version(&version, require_physical(&physical)?, cancel)
                        .await?,
                )
            })?,
            Action::PublicationIssue {
                project,
                branch,
                expected,
                draft,
                operation,
            } => boxed(shared, async move {
                value(store.project_publication_intent(
                    &project,
                    &branch,
                    &expected,
                    draft.as_ref(),
                    &operation,
                )?)
            })?,
            Action::DraftPublish { intent } => boxed(shared, async move {
                package(
                    &runtime
                        .project_publish_draft(&intent, require_physical(&physical)?, cancel)
                        .await?,
                )
            })?,
            Action::Compare {
                left,
                right,
                source_only,
                right_physical,
            } => boxed(shared, async move {
                {
                    let left = selected(runtime, store, &left, &physical).await?;
                    let right_physical = if let Some(input) = right_physical {
                        Some(
                            runtime
                                .physical_from_documents(
                                    &documents(runtime, shared, &input, cancel)?,
                                    &cancel.token(),
                                )
                                .await?,
                        )
                    } else {
                        physical.clone()
                    };
                    let right = selected(runtime, store, &right, &right_physical).await?;
                    if source_only {
                        value(
                            left.project_compare_source(&runtime.source_services(), &right, cancel)
                                .await?,
                        )
                    } else {
                        value(
                            left.project_compare(&runtime.source_services(), &right, cancel)
                                .await?,
                        )
                    }
                }
            })?,
            Action::SourceDetail { revision, logical } => boxed(shared, async move {
                {
                    let source = selected(runtime, store, &revision, &physical).await?;
                    value(
                        source
                            .project_source_detail(&runtime.source_services(), &logical, cancel)
                            .await?,
                    )
                }
            })?,
            Action::MergePreview {
                base,
                target,
                proposal,
                resolutions,
            } => boxed(shared, async move {
                {
                    let base = selected(runtime, store, &base, &physical).await?;
                    let target = selected(runtime, store, &target, &physical).await?;
                    let proposal = selected(runtime, store, &proposal, &physical).await?;
                    value(
                        runtime
                            .project_merge_preview(&base, &target, &proposal, &resolutions, cancel)
                            .await?,
                    )
                }
            })?,
            Action::MergeIssue {
                publication,
                base,
                target,
                proposal,
                resolutions,
            } => boxed(shared, async move {
                {
                    let base = selected(runtime, store, &base, &physical).await?;
                    let target = selected(runtime, store, &target, &physical).await?;
                    let proposal = selected(runtime, store, &proposal, &physical).await?;
                    value(
                        runtime
                            .project_merge_intent(
                                publication,
                                &base,
                                &target,
                                &proposal,
                                resolutions,
                                cancel,
                            )
                            .await?,
                    )
                }
            })?,
            Action::MergeDraftIssue {
                publication,
                base,
                target,
                version,
                resolutions,
            } => boxed(shared, async move {
                {
                    let base = selected(runtime, store, &base, &physical).await?;
                    let target = selected(runtime, store, &target, &physical).await?;
                    value(
                        runtime
                            .project_merge_draft_intent(
                                publication,
                                &base,
                                &target,
                                &version,
                                resolutions,
                                cancel,
                            )
                            .await?,
                    )
                }
            })?,
            Action::MergePublish { intent } => boxed(shared, async move {
                package(
                    &runtime
                        .project_publish_merge(&intent, require_physical(&physical)?, cancel)
                        .await?,
                )
            })?,
            Action::UndoIssue {
                publication,
                target,
                restore,
            } => boxed(shared, async move {
                {
                    let target = selected(runtime, store, &target, &physical).await?;
                    let restore = selected(runtime, store, &restore, &physical).await?;
                    value(
                        runtime
                            .project_undo_intent(publication, &target, &restore, cancel)
                            .await?,
                    )
                }
            })?,
            Action::ContextRecord {
                operation,
                project,
                scope,
                revision,
                effective_from_us,
                effective_to_us,
                corrects,
                deleted,
            } => boxed(shared, async move {
                {
                    let source = selected(runtime, store, &revision, &physical).await?;
                    value(
                        runtime
                            .project_record_context(
                                &operation,
                                &project,
                                &scope,
                                &source,
                                effective_from_us,
                                effective_to_us,
                                corrects.as_deref(),
                                deleted,
                            )
                            .await?,
                    )
                }
            })?,
            Action::ContextCut { project } => boxed(shared, async move {
                value(store.project_context_cut(&project).await?)
            })?,
            Action::ContextSelect {
                cut,
                scope,
                effective_at_us,
            } => boxed(shared, async move {
                value(
                    store
                        .select_project_context(&cut, &scope, effective_at_us)
                        .await?,
                )
            })?,
            Action::ContextLoad { selection } => boxed(shared, async move {
                package(
                    &runtime
                        .project_context_package(&selection, require_physical(&physical)?, cancel)
                        .await?,
                )
            })?,
            Action::ContextReopen { selection } => boxed(shared, async move {
                package(&runtime.project_context_reopen(&selection, cancel).await?)
            })?,
            Action::PreparationDiscovery { revision, request } => boxed(shared, async move {
                {
                    let source = selected(runtime, store, &revision, &physical).await?;
                    value(
                        source
                            .preparation_discovery(&runtime.source_services(), &request, cancel)
                            .await?,
                    )
                }
            })?,
        };
    operation.await
}
