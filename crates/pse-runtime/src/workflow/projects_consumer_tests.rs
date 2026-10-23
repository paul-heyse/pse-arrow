// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One real occurrence proves editing cancellation and compensating publication custody.
//! Public operation futures are boxed so this compound consumer does not sum large polling frames.
#![allow(
    clippy::unwrap_used,
    reason = "Consumer fixture setup and cancellation settlement require every exact original acknowledgement"
)]
use super::*;
use pse_model::study::{OccurrenceKey, StartPolicy};
use std::collections::BTreeMap;

async fn recorded(runtime: &Runtime, package: &ModelingPackage) -> StudyHandle {
    let cancel = crate::CancelSource::new();
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Fixed")
        .unwrap()
        .declaration_id;
    let point = StudyPoint {
        operation: OperationRequest::DeclaredCase(CaseOperation {
            case: root,
            route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            settings: crate::math::settings::SolveSettings::default(),
        }),
        preparation: PreparationSettings {
            compiler: tests::compiler_profile(),
            ..Default::default()
        },
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(0),
            dependencies: vec![],
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        },
    };
    let local = package
        .admit_study(
            &StudyInputs::Explicit(vec![point]),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    let definition = Box::pin(runtime.record_study(&local, &cancel))
        .await
        .unwrap();
    assert_eq!(
        definition.points[0].operation.source,
        OperationSource::of(package)
    );
    Box::pin(runtime.start_recorded_study(definition, &cancel))
        .await
        .unwrap()
}
async fn cancel_and_settle(runtime: &Runtime, handle: &StudyHandle) {
    handle.cancel().await.unwrap();
    for _ in 0..4 {
        if handle.status().await.unwrap().state == StudyState::Concluded {
            return;
        }
        Box::pin(runtime.work_once()).await.unwrap();
    }
    assert_eq!(
        handle.status().await.unwrap().state,
        StudyState::Concluded,
        "one cancelled occurrence must settle without a campaign"
    );
}
#[tokio::test]
async fn project_admitted_draft_run_cancellation_and_undo_preserve_original_recorded_science() {
    let runtime = durable_tests::durable_runtime();
    let cancel = crate::CancelSource::new();
    let (physical_documents, original_documents) = tests::sources(
        "package algebraic { def Fixed { param x:Scalar=2; eq fixed:x==2; annotation report x(\"value\"); } }",
    );
    let physical = Box::pin(runtime.physical_from_sources(&physical_documents, &cancel))
        .await
        .unwrap();
    let source = Box::pin(runtime.package_from_sources(
        std::slice::from_ref(&original_documents),
        physical.clone(),
        &cancel,
    ))
    .await
    .unwrap();
    let store = runtime.canonical_store().unwrap();
    store
        .create_project("create", "project", "One editing occurrence")
        .await
        .unwrap();
    let project = store.project("project").await.unwrap().unwrap();
    let base = Box::pin(runtime.project_fork(&project, "main", "main-source", "fork", &source))
        .await
        .unwrap();
    let original =
        Box::pin(runtime.modeling_revision(base.clone(), physical.clone(), BTreeMap::new()))
            .await
            .unwrap();
    let project = store.project("project").await.unwrap().unwrap();
    let branch = store.project_branch("main").await.unwrap().unwrap();
    let declarations = original.declarations().await.unwrap();
    let mut parameter = declarations
        .iter()
        .find(|row| row.name == "x")
        .unwrap()
        .clone();
    parameter.value.binding.as_mut().unwrap().expression = Some("3".into());
    let mut equation = declarations
        .iter()
        .find(|row| row.name == "fixed")
        .unwrap()
        .clone();
    equation.value.equation.as_mut().unwrap().expression = "x==3".into();
    let content = runtime
        .project_draft_delta(
            &original,
            SourceDelta {
                parent: original.revision_id().into(),
                replacements: vec![parameter, equation],
                ..Default::default()
            },
        )
        .unwrap();
    let body = serde_json::to_vec(&content).unwrap();
    Box::pin(
        runtime.project_save_draft("save", &project, &branch, "draft", &base.key, None, &body),
    )
    .await
    .unwrap();
    let admitted =
        Box::pin(runtime.project_admit_draft("admit", "draft", physical.clone(), &cancel))
            .await
            .unwrap();
    assert!(
        admitted.canonical_revision().is_none(),
        "raw admission creates no branch publication"
    );
    let handle = Box::pin(recorded(&runtime, &admitted)).await;
    let old_key = handle.study_id().to_string();
    assert!(
        store.canonical_study(&old_key).await.unwrap().is_some(),
        "a real recorded request exists before cancellation"
    );
    cancel_and_settle(&runtime, &handle).await;
    let old = store.canonical_study(&old_key).await.unwrap().unwrap();
    let old_run = store.canonical_run(&old.run).await.unwrap().unwrap();
    assert!(old.cancelled);
    assert_eq!(
        &*Box::pin(runtime.project_draft_bytes("draft", &cancel))
            .await
            .unwrap(),
        body.as_slice()
    );
    assert_eq!(
        store
            .project_branch_revision("main")
            .await
            .unwrap()
            .unwrap()
            .key,
        base.key
    );
    let draft = store.project_draft("draft").await.unwrap().unwrap();
    let publication = store
        .project_publication_intent(&project, &branch, &base.key, Some(&draft), "publish")
        .unwrap();
    let published =
        Box::pin(runtime.project_publish_draft(&publication, physical.clone(), &cancel))
            .await
            .unwrap();
    let project = store.project("project").await.unwrap().unwrap();
    let branch = store.project_branch("main").await.unwrap().unwrap();
    let undo = store
        .project_publication_intent(&project, &branch, "publish", None, "undo")
        .unwrap();
    let undo = Box::pin(runtime.project_undo_intent(undo, &published, &original, &cancel))
        .await
        .unwrap();
    let compensated = Box::pin(runtime.project_publish_merge(&undo, physical, &cancel))
        .await
        .unwrap();
    assert_eq!(compensated.canonical_revision().unwrap().key, "undo");
    assert_eq!(
        store
            .project_branch_revision("main")
            .await
            .unwrap()
            .unwrap()
            .key,
        "undo"
    );
    let new_handle = Box::pin(recorded(&runtime, &compensated)).await;
    let new = store
        .canonical_study(&new_handle.study_id().to_string())
        .await
        .unwrap()
        .unwrap();
    let old_after = store.canonical_study(&old_key).await.unwrap().unwrap();
    let old_run_after = store.canonical_run(&old.run).await.unwrap().unwrap();
    assert_eq!(old_after.revision, old.revision);
    assert_eq!(old_after.metadata, old.metadata);
    assert_eq!(old_run_after.revision, old_run.revision);
    assert_eq!(old_run_after.source_selection, old_run.source_selection);
    assert_ne!(
        new.revision, old.revision,
        "new work has a new exact recorded source"
    );
    let new_point = pse_operations::canonical_studies::point_key(
        &new_handle.study_id().to_string(),
        OccurrenceKey(0),
    );
    let new_point = store
        .canonical_study_point(&new_point)
        .await
        .unwrap()
        .unwrap();
    let (stored_operation, _, _): (StudyOperation, pse_ids::roles::BindingHash, AdmittedBinding) =
        serde_json::from_slice(new_point.descriptor.as_slice()).unwrap();
    assert_eq!(
        stored_operation.source,
        OperationSource::of(&compensated),
        "the actual recorded point consumes precisely the selected compensating head"
    );
    let original_hash = OperationSource::of(&original).revision;
    let compensated_hash = OperationSource::of(&compensated).revision;
    assert_eq!(
        compensated_hash, original_hash,
        "compensating publication restores original authored meaning"
    );
    assert_ne!(OperationSource::of(&admitted).revision, compensated_hash);
    assert_eq!(
        &*Box::pin(runtime.project_draft_bytes("draft", &cancel))
            .await
            .unwrap(),
        body.as_slice()
    );
    cancel_and_settle(&runtime, &new_handle).await;
}
