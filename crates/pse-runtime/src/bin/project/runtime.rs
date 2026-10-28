// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One typed operator adapter to existing source, admission and preparation owners.
use super::operator_error::{Context, Result, ensure};
use clap::Args;
use pse_operations::{canonical::CanonicalStore, canonical_contexts::ContextSelection};
#[cfg(all(test, feature = "canonical-tests"))]
use pse_runtime::workflow::{
    PreparationDiscoveryRequest, ResearchReadRequest, StudyCohortRequest, StudyWorkbenchRequest,
};
use pse_runtime::{
    CancelSource, SharedRuntime,
    workflow::{
        CanonicalDeployment, ModelingPackage, OuterAttestation, PhysicalContext,
        ProjectBundleReader, ProjectDraftDocuments, Runtime,
    },
};
use serde::Serialize;
use std::{collections::BTreeMap, io::Read, path::PathBuf, sync::Arc, time::Instant};

pub(super) use pse_runtime::host_operations::project::PROJECT_DOCUMENT_BYTES;
use pse_runtime::host_operations::project::{
    PROJECT_OPERATOR_DEFAULTS, ProjectAction as Action, ProjectOperationRequest as Request,
};

#[derive(Debug, Args)]
pub(crate) struct RuntimeArgs {
    /// Versioned typed action JSON. Scientific selection uses exact original revisions.
    #[arg(long)]
    request: PathBuf,
    /// Accounted runtime capacity, independent of the environment's OS build policy.
    #[arg(long,default_value_t=PROJECT_OPERATOR_DEFAULTS.memory_limit_bytes)]
    memory_limit_bytes: usize,
    #[arg(long, default_value_t = PROJECT_OPERATOR_DEFAULTS.threads)]
    threads: usize,
    #[arg(long)]
    spill_dir: PathBuf,
    #[arg(long,default_value_t=PROJECT_OPERATOR_DEFAULTS.max_spill_bytes)]
    max_spill_bytes: u64,
    #[arg(long, default_value_t = pse_runtime::host_operations::project::default_batch_size())]
    batch_size: usize,
}
macro_rules! field {
    (memory_limit_bytes,$args:ident) => {
        $args.memory_limit_bytes
    };
    (threads,$args:ident) => {
        $args.threads
    };
    (spill_dir,$args:ident) => {
        $args.spill_dir.clone()
    };
    (max_spill_bytes,$args:ident) => {
        $args.max_spill_bytes
    };
    (batch_size,$args:ident) => {
        $args.batch_size
    };
    ($name:ident,$args:ident) => {
        None
    };
}
fn settings(args: &RuntimeArgs) -> Result<pse_runtime::ResourceBudget> {
    macro_rules! input { ($budget:ident; $($name:ident:$ty:ty,$hint:literal $(,$default:expr)? =>$out:ty,$read:expr;)*)=>{
        pse_runtime::settings::EngineSettingsInput{$($name:field!($name,args),)*}
    }; }
    Ok(pse_runtime::engine_settings_fields!(input, unused).resolve(None)?)
}
fn value<T: Serialize>(value: T) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(value)?)
}
fn boxed<'a, F: Future<Output = Result<serde_json::Value>> + 'a>(
    shared: &SharedRuntime,
    operation: F,
) -> Result<std::pin::Pin<Box<dyn Future<Output = Result<serde_json::Value>> + 'a>>> {
    let bytes = size_of::<F>()
        .checked_add(512)
        .context("operator future extent overflow")?;
    let owner =
        datafusion::execution::memory_pool::MemoryConsumer::new("project:cli-operation-future")
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
pub(crate) async fn run(
    store: &CanonicalStore,
    args: RuntimeArgs,
    deadline: Instant,
) -> Result<serde_json::Value> {
    let request: Request = super::project_tools::read_json(&args.request)?;
    let _version = request.version;
    if let Action::StudyWorkbench { request: browse } = &request.action {
        let pool: Arc<dyn pse_columnar::MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(args.memory_limit_bytes));
        let response=pse_columnar::MemoryConsumer::new("project:study-workbench-response").register(&pool);
        response.try_grow(pse_operations::canonical::MetadataResponse::Page.allocation_bytes())?;
        return pse_operations::canonical::response_ownership_scope(Arc::new(response),async {
            value(store.control_study_workbench_page(browse.clone()).await?)
        }).await;
    }
    let budget = settings(&args)?;
    let shared = SharedRuntime::build(budget)?;
    let registry = pse_schema::shared_registry()?;
    let sessions = Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
    let mut artifact = std::fs::File::open(std::env::current_exe()?)?;
    let mut digest = pse_ids::EncodingHasher::new();
    std::io::copy(&mut artifact, &mut digest)?;
    let runtime = Runtime::from_shared(
        shared.clone(),
        sessions.bind_model(registry)?,
        CanonicalDeployment::new(
            store.clone(),
            OuterAttestation {
                source: None,
                build: digest.finish().content_hash(),
            },
            None,
        ),
    )?;
    let cancel = CancelSource::new().with_deadline(Some(deadline));
    let signal_cancel = cancel.clone();
    let signal = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            signal_cancel.cancel();
        }
    });
    let result: Result<serde_json::Value> = async {
        let runtime = &runtime;
        let shared = &shared;
        let cancel = &cancel;
        let descriptive=matches!(&request.action,Action::ProjectClosure{request} | Action::ProjectExport{request,..} if request.descriptive_only);
        let physical = if let Some(input) = request.physical.filter(|_|!descriptive) {
            Some(
                runtime
                    .physical_from_documents(
                        &documents(runtime, shared, &input, cancel)?,
                        &cancel.token(),
                    )
                    .await?,
            )
        } else {
            None
        };
        let effects = request.action.effect_facts();
        tracing::debug!(target: "pse.project.operator", scope = ?effects.scope, read_only = effects.read_only,
            idempotent = effects.idempotent, destructive = effects.destructive, "selected original project operation effects");
        // Erase each operation's deeply nested source/preparation future separately.
        // A generic action must not reserve every workflow's stack frame at once.
        let operation: std::pin::Pin<Box<dyn Future<Output = Result<serde_json::Value>> + '_>> =
            match request.action {
                Action::ProjectClosure { request } => boxed(shared, async move {
                    if request.descriptive_only {value(runtime.project_closure_metadata(&request,cancel).await?)} else {value(runtime.project_closure(&request, require_physical(&physical)?, cancel).await?)}
                })?,
                Action::ProjectExport { destination, request } => boxed(shared, async move {
                    if request.descriptive_only {value(runtime.project_export_metadata(&destination,&request,cancel).await?)} else {value(runtime.project_export(&destination, &request, require_physical(&physical)?, cancel).await?)}
                })?,
                Action::ProjectImportIssue { input, title, branch_name, maximum_document_bytes } => boxed(shared, async move {
                    cancel.token().checkpoint()?;
                    let bundle = ProjectBundleReader::open(&input, maximum_document_bytes)?;
                    cancel.token().checkpoint()?;
                    value(runtime.project_import_issue(&bundle, &title, &branch_name).await?)
                })?,
                Action::ProjectImportStatus { intent } => boxed(shared, async move {
                    value(runtime.project_import_status(&intent).await?)
                })?,
                Action::ProjectImport { input, intent, maximum_document_bytes } => boxed(shared, async move {
                    cancel.token().checkpoint()?;
                    let bundle = ProjectBundleReader::open(&input, maximum_document_bytes)?;
                    cancel.token().checkpoint()?;
                    value(runtime.project_import(&bundle, &intent, cancel).await?)
                })?,
                Action::ProjectImportRecover { intent, original } => boxed(shared, async move {
                    value(runtime.project_import_recover(&intent, &original, cancel).await?)
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
                    package(&runtime.research_dataset_open(&key, &context, cancel).await?)
                })?,
                Action::ResearchSelectionOpen { key } => boxed(shared, async move {
                    let (source, fit) = runtime.research_selection_open(&key, cancel).await?;
                    Ok(serde_json::json!({"package": package(&source)?, "fit": fit}))
                })?,
                Action::ResearchPublicationPreview { request } => boxed(shared, async move {
                    value(runtime.research_publication_preview(&request, cancel).await?)
                })?,
                Action::ResearchPublish { intent } => boxed(shared, async move {
                    value(runtime.research_publish(&intent, cancel).await?)
                })?,
                Action::ResearchPublicationOpen { key } => boxed(shared, async move {
                    package(&runtime.research_publication_open(&key, cancel).await?)
                })?,
                Action::StudyWorkbench { .. } => return Err("operational browse was not dispatched before scientific construction".into()),
                Action::StudyCohort { request,output,maximum_bytes } => boxed(shared, async move {
                    let cohort=runtime.freeze_study_cohort(request).await?;
                    let bytes=if let Some(output)=&output {Some(cohort.export_pages(output,maximum_bytes,&cancel.token()).await?)}else {None};
                    Ok(serde_json::json!({"receipt":cohort.receipt(),"output":output,"bytes":bytes}))
                })?,
                Action::StudyAnalyze { request, kpi, output } => boxed(shared, async move {
                    value(runtime.export_study_analysis(request, &kpi, &output, &cancel.token()).await?)
                })?,
                Action::StudyResult { request, member, relation, start, end, output, maximum_bytes } => boxed(shared, async move {
                    let cohort = runtime.freeze_study_cohort(request).await?;
                    let mut reader = cohort.result(member, &relation, start, end, cancel.token()).await?;
                    let bytes = reader.export_ipc_bounded(&output, maximum_bytes, &cancel.token()).await?;
                    Ok(serde_json::json!({"cohort": cohort.receipt(), "member": member, "output": output, "bytes": bytes}))
                })?,
                Action::StudyObserve { request, cursor, delay_ms } => boxed(shared, async move {
                    let mut observer = runtime.observe_study(request, cursor).await?;
                    let page = if let Some(ms) = delay_ms {
                        ensure!(ms <= 60_000, "observation delay exceeds 60 seconds");
                        observer.follow(std::time::Duration::from_millis(ms)).await?
                    } else { observer.poll().await? };
                    observer.close();
                    value(page)
                })?,
                Action::Import { documents: input } => boxed(shared, async move {
                    package(
                        &runtime
                            .modeling_from_documents(
                                &documents(runtime, shared, &input, cancel)?,
                                require_physical(&physical)?,
                                cancel,
                            )
                            .await?,
                    )
                })?,
                Action::ImportFiles { packages, physical_packages } => boxed(shared, async move {
                    let physical_documents = file_documents(runtime, shared, &physical_packages, cancel)?;
                    let selected_physical = runtime.physical_from_documents(&physical_documents, &cancel.token()).await?;
                    let sources = file_documents(runtime, shared, &packages, cancel)?;
                    package(&runtime.modeling_from_documents(&sources, selected_physical, cancel).await?)
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
                            .project_open_selection(
                                &selection,
                                require_physical(&physical)?,
                                cancel,
                            )
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
                        let content = bytes(&input, args.memory_limit_bytes / 4)?;
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
                        let content = bytes(&input, args.memory_limit_bytes / 4)?;
                        value(runtime.project_save_draft_intent(&intent, &content).await?)
                    }
                })?,
                Action::DraftAdmit { operation, key } => boxed(shared, async move {
                    package(
                        &runtime
                            .project_admit_draft(
                                &operation,
                                &key,
                                require_physical(&physical)?,
                                cancel,
                            )
                            .await?,
                    )
                })?,
                Action::DraftAdmitVersion { version } => boxed(shared, async move {
                    package(
                        &runtime
                            .project_admit_draft_version(
                                &version,
                                require_physical(&physical)?,
                                cancel,
                            )
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
                            value(left.project_compare_source(&runtime.source_services(), &right, cancel).await?)
                        } else {
                            value(left.project_compare(&runtime.source_services(), &right, cancel).await?)
                        }
                    }
                })?,
                Action::SourceDetail { revision, logical } => boxed(shared, async move {
                    {
                        let source = selected(runtime, store, &revision, &physical).await?;
                        value(source.project_source_detail(&runtime.source_services(), &logical, cancel).await?)
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
                                .project_merge_preview(
                                    &base,
                                    &target,
                                    &proposal,
                                    &resolutions,
                                    cancel,
                                )
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
                            .project_context_package(
                                &selection,
                                require_physical(&physical)?,
                                cancel,
                            )
                            .await?,
                    )
                })?,
                Action::ContextReopen { selection } => boxed(shared, async move {
                    package(&runtime.project_context_reopen(&selection, cancel).await?)
                })?,
                Action::PreparationDiscovery { revision, request } => boxed(shared, async move {
                    {
                        let source = selected(runtime, store, &revision, &physical).await?;
                        value(source.preparation_discovery(&runtime.source_services(), &request, cancel).await?)
                    }
                })?,
            };
        operation.await
    }
    .await;
    signal.abort();
    let _ = signal.await;
    runtime.close();
    shared.math().shutdown_native_lanes().await;
    drop(runtime);
    drop(shared);
    result
}

#[cfg(test)]
mod transport_tests {
    #![allow(
        clippy::unwrap_used,
        reason = "typed CLI request controls require exact successful decoding"
    )]
    use super::*;

    #[test]
    fn project_bundle_issue_defaults_remain_typed_and_finite() {
        let request: Request = serde_json::from_value(serde_json::json!({
            "version": 1,
            "action": {"kind": "project_import_issue", "input": "selected-pack", "title": "Imported"}
        })).unwrap();
        assert!(matches!(request.action,
            Action::ProjectImportIssue { input, branch_name, maximum_document_bytes, .. }
                if input == std::path::Path::new("selected-pack") && branch_name == "main" && maximum_document_bytes == PROJECT_DOCUMENT_BYTES
        ));
    }

    #[test]
    fn lifecycle_selection_uses_the_exact_typed_owner_target() {
        let action: Action = serde_json::from_value(serde_json::json!({
            "kind": "lifecycle_select", "target": {"kind": "source_history", "key": "original-revision"}
        })).unwrap();
        assert!(
            matches!(action, Action::LifecycleSelect { target } if target.key == "original-revision")
        );
        assert!(serde_json::from_value::<Action>(serde_json::json!({
            "kind": "lifecycle_select", "target": {"kind": "source_history", "key": "original-revision"}, "query": "arbitrary"
        })).is_err());
        assert!(
            serde_json::from_value::<Action>(
                serde_json::json!({"kind": "shell", "command": ["arbitrary"]})
            )
            .is_err()
        );
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "the disposable native CLI fixture requires exact successful setup and inspected outcomes"
    )]
    use super::*;
    #[tokio::test]
    async fn project_cli_runtime_import_fork_compare_and_passive_discovery() {
        let store = pse_operations::testing::canonical_fixture_store().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();
        let physical_root = root.join("tests/fixtures/packages/physical-primitives");
        let physical = ProjectDraftDocuments {
            version: Default::default(),
            packages: vec![
                [
                    "package.toml",
                    "materials/physical.yaml",
                    "materials/time.yaml",
                ]
                .into_iter()
                .map(|path| {
                    (
                        path.into(),
                        std::fs::read(physical_root.join(path)).unwrap(),
                    )
                })
                .collect(),
            ],
        };
        let mut source = std::fs::read_to_string(
            root.join("tests/fixtures/packages/minimal_named/package.toml"),
        )
        .unwrap();
        source=source.replace("dependencies = []",r#"dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#);
        let docs=ProjectDraftDocuments{version:Default::default(),packages:vec![BTreeMap::from([("package.toml".into(),source.into_bytes()),("models/root.pse".into(),b"package minimal_named {param k:Scalar=1; def Root {var x:Scalar; eq e:x*x==k;}}".to_vec())])]};
        let execute = async |action: Action| {
            let path = directory.path().join("request.json");
            std::fs::write(
                &path,
                serde_json::to_vec(&Request {
                    version: Default::default(),
                    physical: Some(physical.clone()),
                    action,
                })
                .unwrap(),
            )
            .unwrap();
            run(
                &store,
                RuntimeArgs {
                    request: path,
                    memory_limit_bytes: 8usize << 30,
                    threads: 2,
                    spill_dir: directory.path().into(),
                    max_spill_bytes: 1 << 30,
                    batch_size: 8192,
                },
                Instant::now() + std::time::Duration::from_secs(60),
            )
            .await
            .unwrap()
        };
        let source_root = directory.path().join("batch-source");
        std::fs::create_dir_all(&source_root).unwrap();
        for (relative, body) in &docs.packages[0] {
            let path = source_root.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        }
        let imported = execute(Action::ImportFiles {
            packages: vec![source_root],
            physical_packages: vec![physical_root],
        })
        .await;
        let original = imported["canonical_revision"].as_str().unwrap().to_owned();
        store
            .create_project("cli-create", "cli-project", "CLI integration")
            .await
            .unwrap();
        let project = store.project("cli-project").await.unwrap().unwrap();
        let fork = execute(Action::Fork {
            project,
            branch: "cli-main".into(),
            problem: "cli-problem".into(),
            operation: "cli-fork".into(),
            source: original.clone(),
        })
        .await;
        let fork = fork.as_str().unwrap().to_owned();
        let compared = execute(Action::Compare {
            left: original,
            right: fork.clone(),
            source_only: false,
            right_physical: None,
        })
        .await;
        assert!(compared["semantic_context_matches"].as_bool().unwrap());
        assert!(compared["changes"].as_array().unwrap().is_empty());
        let revision = store.revision(&fork).await.unwrap().unwrap();
        let pin = store
            .protect(revision, std::time::Duration::from_secs(60))
            .await
            .unwrap();
        let members = store.membership_page(&pin, None, "").await.unwrap();
        let root = pse_ids::SemanticId::parse_hex(
            members
                .iter()
                .find(|row| row.name == "Root")
                .unwrap()
                .logical
                .strip_prefix("modeling:")
                .unwrap(),
        )
        .unwrap();
        store.release(&pin).await.unwrap();
        let discovery = execute(Action::PreparationDiscovery {
            revision: fork.clone(),
            request: PreparationDiscoveryRequest {
                root: pse_model::generated::identities::DeclarationId::from_id(root),
                instance: pse_model::generated::identities::InstanceId::from_id(root),
                bindings: Default::default(),
                limits: Default::default(),
                durable_after: None,
            },
        })
        .await;
        assert_eq!(discovery["durable_eligibility"], "unchecked");
        assert!(discovery["resident"].as_array().unwrap().is_empty());
        let loaded = execute(Action::Load { revision: fork }).await;
        assert_eq!(loaded["canonical_problem"], "cli-problem");
        let page = execute(Action::StudyWorkbench {
            request: StudyWorkbenchRequest {
                filter: Default::default(),
                maximum: 4,
                cursor: None,
            },
        })
        .await;
        assert!(page["headers"].as_array().unwrap().is_empty());
        let cohort = execute(Action::StudyCohort {
            output: None,
            maximum_bytes: cohort_export_bytes(),
            request: StudyCohortRequest {
                filter: Default::default(),
                maximum_studies: 4,
                maximum_members: 4,
            },
        })
        .await;
        assert_eq!(cohort["receipt"]["complete"], true);
        assert_eq!(cohort["receipt"]["member_count"], 0);
        assert!(cohort.get("members").is_none());
        let research = execute(Action::ResearchRead {
            request: ResearchReadRequest::Datasets {
                project: "cli-project".into(),
                after: String::new(),
                limit: 4,
            },
        })
        .await;
        assert!(research["rows"].as_array().unwrap().is_empty());
        let meta = store
            .project_metadata(
                pse_operations::canonical_projects::ProjectMetadataKind::Projects,
                "",
                None,
            )
            .await
            .unwrap();
        assert_eq!(meta.projects.len(), 1);
        let frozen = meta.projects[0].clone();
        store
            .project_navigate(
                "cli-rename",
                &frozen,
                &pse_operations::canonical_projects::ProjectNavigation::ProjectTitle {
                    name: "CLI renamed".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(
            store.project("cli-project").await.unwrap().unwrap().title,
            "CLI renamed"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
}
