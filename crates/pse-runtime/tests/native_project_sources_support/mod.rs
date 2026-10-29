// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public source/project history consumers within the caller's private native service.

use futures_util::{FutureExt, future::BoxFuture};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_ids::ContentHash;
use pse_operations::canonical_projects::{
    ProjectMetadataKind, ProjectNavigation, ProjectPublication,
};
use pse_runtime::{
    CancelSource,
    workflow::{
        ModelingPackage, PhysicalContext, ProjectDraftDelta, ProjectReference,
        ProjectSelectRequest, Runtime, SourceDelta,
    },
};
use std::{collections::BTreeMap, path::Path};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const PROJECT: &str = "source-history-project";
const BRANCH: &str = "source-history-main";
const PROBLEM: &str = "source-history-owner";
const DRAFT: &str = "source-history-draft";

fn documents() -> Result<BTreeMap<String, Vec<u8>>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let header: toml::Value = toml::from_str(&std::fs::read_to_string(
        root.join("physical-primitives/package.toml"),
    )?)?;
    let version: Vec<i64> = header["package"]["version"]
        .as_str()
        .unwrap()
        .split('.')
        .map(|part| part.parse().unwrap())
        .collect();
    assert_eq!(version.len(), 3);
    let mut manifest: toml::Value = toml::from_str(&std::fs::read_to_string(
        root.join("minimal_explicit/package.toml"),
    )?)?;
    manifest["package"]["id_policy"] = "named".into();
    manifest["package"]["name"] = "native.project.sources".into();
    let requirement = toml::Table::from_iter([
        ("operator".into(), "exact".into()),
        ("major".into(), version[0].into()),
        ("minor".into(), version[1].into()),
        ("patch".into(), version[2].into()),
    ]);
    manifest["package"]["dependencies"] = toml::Value::Array(vec![
        toml::Table::from_iter([
            ("package_id".into(), header["package"]["package_id"].clone()),
            ("version_req".into(), requirement.into()),
        ])
        .into(),
    ]);
    Ok(BTreeMap::from([
        (
            "package.toml".into(),
            toml::to_string(&manifest)?.into_bytes(),
        ),
        (
            "models/model.pse".into(),
            b"package project_source {param value:Scalar=1;}".to_vec(),
        ),
    ]))
}

fn initial<'a>(
    runtime: &'a Runtime,
    physical: PhysicalContext,
) -> BoxFuture<'a, Result<ModelingPackage>> {
    async move {
        let sources = documents()?;
        let preparation = runtime.preparation_services();
        let pool = preparation.pool();
        let cancel = CancelSource::new();
        let token = cancel.token();
        let validation = runtime.validation_context();
        let documents = OwnedDocumentSet::try_from_bundles(
            vec![load_package_documents_owned(
                &sources,
                runtime.registry(),
                Default::default(),
                &pool,
                &token,
                &validation,
            )?],
            &pool,
            &token,
        )?;
        Ok(Box::pin(runtime.modeling_from_documents(&documents, physical, &cancel)).await?)
    }
    .boxed()
}

fn value<'a>(
    runtime: &'a Runtime,
    package: &'a ModelingPackage,
    expected: u32,
) -> BoxFuture<'a, Result<()>> {
    async move {
        let declarations = package
            .declarations(&runtime.preparation_services())
            .await?;
        let parameter = declarations.iter().find(|row| row.name == "value").unwrap();
        assert_eq!(
            parameter
                .value
                .binding
                .as_ref()
                .unwrap()
                .expression
                .as_deref(),
            Some(expected.to_string().as_str())
        );
        Ok(())
    }
    .boxed()
}

fn typed_value<'a>(
    runtime: &'a Runtime,
    base: &'a ModelingPackage,
    next: u32,
) -> BoxFuture<'a, Result<ProjectDraftDelta>> {
    async move {
        let declarations = base.declarations(&runtime.preparation_services()).await?;
        let mut parameter = declarations
            .iter()
            .find(|row| row.name == "value")
            .unwrap()
            .clone();
        parameter.value.binding.as_mut().unwrap().expression = Some(next.to_string());
        Ok(runtime.project_draft_delta(
            base,
            SourceDelta {
                parent: base.revision_id().into(),
                replacements: vec![parameter],
                ..Default::default()
            },
        )?)
    }
    .boxed()
}

fn save<'a>(
    runtime: &'a Runtime,
    operation: &'a str,
    draft: &'a str,
    base: &'a str,
    expected: Option<u64>,
    bytes: &'a [u8],
) -> BoxFuture<'a, Result<()>> {
    async move {
        let store = runtime.canonical_store()?;
        let project = store.project(PROJECT).await?.unwrap();
        let branch = store.project_branch(BRANCH).await?.unwrap();
        let intent =
            Box::pin(runtime.project_draft_save_intent(
                operation, &project, &branch, draft, base, expected, bytes,
            ))
            .await?;
        let first = Box::pin(runtime.project_save_draft_intent(&intent, bytes)).await?;
        let replay = Box::pin(runtime.project_save_draft_intent(&intent, bytes)).await?;
        assert_eq!(
            first, replay,
            "the exact raw-custody intent retains one acknowledgement"
        );
        Ok(())
    }
    .boxed()
}

async fn publication(
    runtime: &Runtime,
    draft: &str,
    base: &str,
    operation: &str,
) -> Result<ProjectPublication> {
    let store = runtime.canonical_store()?;
    let project = store.project(PROJECT).await?.unwrap();
    let branch = store.project_branch(BRANCH).await?.unwrap();
    let draft = store.project_draft(draft).await?.unwrap();
    Ok(store.project_publication_intent(&project, &branch, base, Some(&draft), operation)?)
}

async fn navigate(runtime: &Runtime, operation: &str, action: ProjectNavigation) -> Result<()> {
    let store = runtime.canonical_store()?;
    let project = store.project(PROJECT).await?.unwrap();
    store.project_navigate(operation, &project, &action).await?;
    Ok(())
}

/// Exercise source custody/publication/history independently of the study's project.
/// The caller owns service shutdown and the supplied runtime/physical closure.
pub(crate) async fn check(runtime: &Runtime, physical: PhysicalContext) -> Result<()> {
    let cancel = CancelSource::new();
    let store = runtime.canonical_store()?;
    let source = initial(runtime, physical.clone()).await?;
    value(runtime, &source, 1).await?;
    store
        .create_project("source-history-create", PROJECT, "Native source history")
        .await?;
    let project = store.project(PROJECT).await?.unwrap();
    let base =
        Box::pin(runtime.project_fork(&project, BRANCH, PROBLEM, "source-history-fork", &source))
            .await?;
    assert_eq!(base.problem, PROBLEM);
    assert_eq!(
        store.project_branch(BRANCH).await?.unwrap().problem,
        PROBLEM
    );
    let selected = Box::pin(runtime.project_select(
        &ProjectSelectRequest {
            version: Default::default(),
            reference: ProjectReference::Branch { key: BRANCH.into() },
        },
        physical.clone(),
        &cancel,
    ))
    .await?;
    assert_eq!(selected.project, PROJECT);
    assert_eq!(selected.problem, PROBLEM);
    assert_eq!(selected.revision, base.key);
    let original =
        Box::pin(runtime.project_open_selection(&selected, physical.clone(), &cancel)).await?;
    value(runtime, &original, 1).await?;

    // Raw custody accepts bytes without claiming an admitted model or publishing
    // a branch. Both the empty and malformed immutable versions remain available.
    save(
        runtime,
        "source-history-raw-empty",
        DRAFT,
        &base.key,
        None,
        b"",
    )
    .await?;
    assert!(
        Box::pin(runtime.project_draft_bytes(DRAFT, &cancel))
            .await?
            .is_empty()
    );
    save(
        runtime,
        "source-history-raw-invalid",
        DRAFT,
        &base.key,
        Some(1),
        b"invalid [[",
    )
    .await?;
    assert_eq!(
        &*Box::pin(runtime.project_draft_bytes(DRAFT, &cancel)).await?,
        b"invalid [["
    );
    assert!(
        Box::pin(runtime.project_admit_draft(
            "source-history-admit-invalid",
            DRAFT,
            physical.clone(),
            &cancel
        ))
        .await
        .is_err()
    );
    let diagnostics = store
        .project_metadata(
            ProjectMetadataKind::Diagnostics,
            "source-history-raw-invalid",
            None,
        )
        .await?;
    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert!(!diagnostics.diagnostics[0].admitted);
    assert_eq!(
        diagnostics.diagnostics[0].version,
        "source-history-raw-invalid"
    );
    let invalid = publication(runtime, DRAFT, &base.key, "source-history-publish-invalid").await?;
    assert!(
        Box::pin(runtime.project_publish_draft(&invalid, physical.clone(), &cancel))
            .await
            .is_err()
    );
    assert!(
        store
            .project_acknowledgement("source-history-publish-invalid")
            .await?
            .is_none()
    );
    assert_eq!(
        store.project_branch_revision(BRANCH).await?.unwrap().key,
        base.key
    );
    navigate(
        runtime,
        "source-history-savepoint",
        ProjectNavigation::Savepoint {
            key: "source-history-invalid-point".into(),
            name: "raw invalid text".into(),
            draft: DRAFT.into(),
            version: "source-history-raw-invalid".into(),
            generation: 2,
        },
    )
    .await?;

    // Native typed edits capture their actual original source, parent and
    // physical premises. Admission is local and leaves the published head alone.
    let typed = typed_value(runtime, &original, 2).await?;
    let body = serde_json::to_vec(&typed)?;
    save(
        runtime,
        "source-history-raw-valid",
        DRAFT,
        &base.key,
        Some(2),
        &body,
    )
    .await?;
    let admitted = Box::pin(runtime.project_admit_draft(
        "source-history-admit-valid",
        DRAFT,
        physical.clone(),
        &cancel,
    ))
    .await?;
    assert!(admitted.canonical_revision().is_none());
    value(runtime, &admitted, 2).await?;
    assert_eq!(
        store.project_branch_revision(BRANCH).await?.unwrap().key,
        base.key
    );
    let valid_version = store
        .project_draft_version("source-history-raw-valid")
        .await?
        .unwrap();
    let intent = publication(runtime, DRAFT, &base.key, "source-history-publish-valid").await?;
    let published =
        Box::pin(runtime.project_publish_draft(&intent, physical.clone(), &cancel)).await?;
    let revision = published.canonical_revision().unwrap();
    assert_eq!(revision.key, "source-history-publish-valid");
    assert_eq!(revision.problem, PROBLEM);
    value(runtime, &published, 2).await?;
    assert_eq!(
        store
            .project_acknowledgement("source-history-publish-valid")
            .await?
            .unwrap()
            .revision
            .as_deref(),
        Some(revision.key.as_str())
    );
    let reopened =
        Box::pin(runtime.modeling_revision(revision.clone(), physical.clone(), BTreeMap::new()))
            .await?;
    value(runtime, &reopened, 2).await?;
    let published_selection = Box::pin(runtime.project_select(
        &ProjectSelectRequest {
            version: Default::default(),
            reference: ProjectReference::Branch { key: BRANCH.into() },
        },
        physical.clone(),
        &cancel,
    ))
    .await?;

    // Accepted immutable intent settles before any mutable draft read: later
    // malformed custody cannot invalidate or replay the accepted source effect.
    save(
        runtime,
        "source-history-advance-draft",
        DRAFT,
        &revision.key,
        Some(3),
        b"later invalid body",
    )
    .await?;
    let replayed =
        Box::pin(runtime.project_publish_draft(&intent, physical.clone(), &cancel)).await?;
    assert_eq!(replayed.canonical_revision().unwrap().key, revision.key);
    value(runtime, &replayed, 2).await?;
    let versioned =
        Box::pin(runtime.project_admit_draft_version(&valid_version, physical.clone(), &cancel))
            .await?;
    value(runtime, &versioned, 2).await?;
    assert!(versioned.canonical_revision().is_none());
    assert_eq!(
        &*Box::pin(runtime.project_draft_version_bytes("source-history-raw-valid", &cancel))
            .await?,
        body.as_slice()
    );
    navigate(
        runtime,
        "source-history-restore",
        ProjectNavigation::Restore {
            key: "source-history-invalid-point".into(),
            draft: DRAFT.into(),
            generation: 4,
        },
    )
    .await?;
    let restored = store.project_draft(DRAFT).await?.unwrap();
    assert_eq!(restored.head.generation, 5);
    assert_eq!(restored.version.key, "source-history-raw-invalid");
    assert_eq!(
        &*Box::pin(runtime.project_draft_bytes(DRAFT, &cancel)).await?,
        b"invalid [["
    );
    assert!(
        Box::pin(runtime.project_draft_version_bytes("source-history-raw-empty", &cancel))
            .await?
            .is_empty()
    );

    // A second real admitted publication moves the branch to value3. Earlier
    // branch selections and the bookmark still reopen their exact original science.
    let third = serde_json::to_vec(&typed_value(runtime, &published, 3).await?)?;
    save(
        runtime,
        "source-history-raw-third",
        "source-history-third-draft",
        &revision.key,
        None,
        &third,
    )
    .await?;
    let third_intent = publication(
        runtime,
        "source-history-third-draft",
        &revision.key,
        "source-history-publish-third",
    )
    .await?;
    let third =
        Box::pin(runtime.project_publish_draft(&third_intent, physical.clone(), &cancel)).await?;
    value(runtime, &third, 3).await?;
    let latest = third.canonical_revision().unwrap();
    assert_eq!(
        store.project_branch_revision(BRANCH).await?.unwrap().key,
        latest.key
    );
    navigate(
        runtime,
        "source-history-rename",
        ProjectNavigation::BranchName {
            key: BRANCH.into(),
            name: "renamed source branch".into(),
        },
    )
    .await?;
    navigate(
        runtime,
        "source-history-bookmark",
        ProjectNavigation::Bookmark {
            key: "source-history-baseline".into(),
            name: "original value1".into(),
            revision: base.key.clone(),
        },
    )
    .await?;
    let bookmark = Box::pin(runtime.project_select(
        &ProjectSelectRequest {
            version: Default::default(),
            reference: ProjectReference::Bookmark {
                key: "source-history-baseline".into(),
            },
        },
        physical.clone(),
        &cancel,
    ))
    .await?;
    assert_eq!(bookmark.revision, base.key);
    for (selection, expected) in [(&selected, 1), (&published_selection, 2), (&bookmark, 1)] {
        let retained =
            Box::pin(runtime.project_open_selection(selection, physical.clone(), &cancel)).await?;
        assert_eq!(
            retained.canonical_revision().unwrap().key,
            selection.revision
        );
        value(runtime, &retained, expected).await?;
    }
    let head = Box::pin(runtime.project_select(
        &ProjectSelectRequest {
            version: Default::default(),
            reference: ProjectReference::Branch { key: BRANCH.into() },
        },
        physical.clone(),
        &cancel,
    ))
    .await?;
    assert_eq!(head.revision, latest.key);
    assert_eq!(
        store.project_branch(BRANCH).await?.unwrap().name,
        "renamed source branch"
    );

    // Keep native authority and physical source mismatches explicit. No second
    // runtime, altered physical registry or frontend reconstruction supplies trust.
    let mut wrong_issuer = selected.clone();
    wrong_issuer.issuing_store.push_str("-foreign");
    assert!(
        Box::pin(runtime.project_open_selection(&wrong_issuer, physical.clone(), &cancel))
            .await
            .is_err()
    );
    let mut wrong_physical = selected.clone();
    wrong_physical.physical_context = ContentHash::from_bytes([0x91; 32]);
    assert_ne!(wrong_physical.physical_context, physical.identity());
    assert!(
        Box::pin(runtime.project_open_selection(&wrong_physical, physical.clone(), &cancel))
            .await
            .is_err()
    );
    for (operation, key, mut content) in [
        (
            "source-history-wrong-issuer-save",
            "source-history-wrong-issuer",
            typed.clone(),
        ),
        (
            "source-history-wrong-physical-save",
            "source-history-wrong-physical",
            typed.clone(),
        ),
    ] {
        if key.ends_with("issuer") {
            content.issuing_store.push_str("-foreign");
        } else {
            content.source.physical_context = ContentHash::from_bytes([0x91; 32]);
        }
        let bytes = serde_json::to_vec(&content)?;
        save(runtime, operation, key, &base.key, None, &bytes).await?;
        assert!(
            Box::pin(runtime.project_admit_draft_version(
                &store.project_draft(key).await?.unwrap().version,
                physical.clone(),
                &cancel
            ))
            .await
            .is_err()
        );
        assert_eq!(
            &*Box::pin(runtime.project_draft_bytes(key, &cancel)).await?,
            bytes.as_slice()
        );
    }
    save(
        runtime,
        "source-history-wrong-parent-save",
        "source-history-wrong-parent",
        &latest.key,
        None,
        &body,
    )
    .await?;
    assert!(
        Box::pin(runtime.project_admit_draft(
            "source-history-wrong-parent-admit",
            "source-history-wrong-parent",
            physical,
            &cancel
        ))
        .await
        .is_err()
    );
    assert_eq!(
        &*Box::pin(runtime.project_draft_bytes("source-history-wrong-parent", &cancel)).await?,
        body.as_slice()
    );
    assert_eq!(
        store.project_branch_revision(BRANCH).await?.unwrap().key,
        latest.key
    );
    Ok(())
}
