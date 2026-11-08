// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Request files, process signals and presentation for the shared project handler.
use super::operator_error::Result;
use clap::Args;
use pse_operations::canonical::CanonicalStore;
use pse_runtime::CancelSource;
#[cfg(test)]
use pse_runtime::workflow::{
    PreparationDiscoveryRequest, ProjectDraftDocuments, ResearchReadRequest, StudyCohortRequest,
    StudyWorkbenchRequest,
};
#[cfg(test)]
use std::collections::BTreeMap;
use std::{path::PathBuf, time::Instant};

pub(super) use pse_runtime::host_operations::project::PROJECT_DOCUMENT_BYTES;
#[cfg(test)]
use pse_runtime::host_operations::project::ProjectAction as Action;
use pse_runtime::host_operations::project::{
    PROJECT_OPERATOR_DEFAULTS, ProjectOperationRequest as Request, ProjectOperatorDefaults,
    handler::{
        execute_project_operation, operational_project_operation, project_operator_attestation,
    },
    project_operator_budget,
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
fn settings(args: &RuntimeArgs) -> Result<pse_runtime::ResourceBudget> {
    Ok(project_operator_budget(
        args.spill_dir.clone(),
        ProjectOperatorDefaults {
            memory_limit_bytes: args.memory_limit_bytes,
            threads: args.threads,
            max_spill_bytes: args.max_spill_bytes,
        },
        args.batch_size,
    )?)
}
pub(crate) async fn run(
    store: &CanonicalStore,
    args: RuntimeArgs,
    deadline: Instant,
) -> Result<serde_json::Value> {
    let request: Request = super::project_tools::read_json(&args.request)?;
    if let Some(response) =
        operational_project_operation(store, &request, args.memory_limit_bytes).await?
    {
        return Ok(response);
    }
    let budget = settings(&args)?;
    let attestation = project_operator_attestation()?;
    let cancel = CancelSource::new().with_deadline(Some(deadline));
    let signal = super::signals::ProcessInterrupt::new(cancel.clone())?;
    let result = execute_project_operation(store, request, budget, attestation, &cancel).await;
    signal.finish().await;
    result.map_err(Into::into)
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
            "version": 2,
            "action": {"kind": "project_import_issue", "input": "selected-pack", "title": "Imported"}
        })).unwrap();
        assert!(matches!(request.action,
            Action::ProjectImportIssue { input, branch_name, maximum_document_bytes, .. }
                if input == std::path::Path::new("selected-pack") && branch_name == "main" && maximum_document_bytes == PROJECT_DOCUMENT_BYTES
        ));
        assert!(serde_json::from_value::<Request>(serde_json::json!({
            "version": 1,
            "action": {"kind": "project_import_issue", "input": "selected-pack", "title": "Imported"}
        })).is_err());
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "the disposable native CLI fixture requires exact successful setup and inspected outcomes"
    )]
    use super::*;
    #[tokio::test]
    async fn project_cli_runtime_import_fork_compare_and_passive_discovery() {
        let store = pse_operations::native_fixture::canonical_fixture_store().unwrap();
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
        // Operational browsing consumes the same library handler without validating
        // unused scientific settings or admitting supplied physical documents.
        let operational_request = Request {
            version: Default::default(),
            physical: Some(ProjectDraftDocuments {
                version: Default::default(),
                packages: vec![],
            }),
            action: Action::StudyWorkbench {
                request: StudyWorkbenchRequest {
                    filter: Default::default(),
                    maximum: 4,
                    cursor: None,
                },
            },
        };
        let direct_page = operational_project_operation(&store, &operational_request, 8usize << 30)
            .await
            .unwrap()
            .unwrap();
        let path = directory.path().join("operational-request.json");
        std::fs::write(&path, serde_json::to_vec(&operational_request).unwrap()).unwrap();
        let cli_page = run(
            &store,
            RuntimeArgs {
                request: path,
                memory_limit_bytes: 8usize << 30,
                threads: 0,
                spill_dir: directory.path().join("absent-scientific-spill"),
                max_spill_bytes: 0,
                batch_size: 0,
            },
            Instant::now() + std::time::Duration::from_secs(60),
        )
        .await
        .unwrap();
        assert_eq!(direct_page, cli_page);
        let cohort = execute(Action::StudyCohort {
            output: None,
            maximum_bytes: pse_runtime::host_operations::project::cohort_export_bytes(),
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
