// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable study definition and durable publication through the existing worker owners.
use super::durable_tests::quick;
use super::worker_tests::{ipopt, job_durable, sources};
use super::*;
use pse_model::study::*;
use pse_operations::{lifecycle::AttemptState, testing::TestDatabase};
use std::time::Duration;

const CASES: &str = r#"package algebraic {
def Root { var x:Scalar; eq square:x*x==4; annotation start x(1); annotation bounds x(0,10); annotation report x("root"); annotation check x(x>1); }
def Failed { var x:Scalar; eq square:x*x == -1; annotation start x(1); annotation report x("root"); }
}"#;
fn point(
    case: pse_model::generated::identities::DeclarationId,
    key: u32,
    dependencies: Vec<Dependency>,
    start: StartPolicy,
) -> StudyPoint {
    StudyPoint {
        operation: OperationRequest::DeclaredCase(CaseOperation {
            case,
            route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            settings: ipopt(),
        }),
        preparation: PreparationSettings::default(),
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(key),
            dependencies,
            start,
            attempt_limit: 1,
        },
    }
}
async fn admitted(
    runtime: &Runtime,
    points: impl FnOnce(
        pse_model::generated::identities::DeclarationId,
        pse_model::generated::identities::DeclarationId,
    ) -> Vec<StudyPoint>,
) -> (PackageSources, StudyDefinition) {
    let (physical, modeling) = sources(CASES);
    let cancel = crate::CancelSource::new();
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime
                .physical_from_sources(&physical, &cancel)
                .await
                .unwrap(),
        )
        .unwrap();
    let find = |name: &str| {
        package
            .declarations()
            .iter()
            .find(|declaration| declaration.name == name)
            .unwrap()
            .declaration_id
    };
    let points = points(find("Root"), find("Failed"));
    let definition = package
        .admit_study_points(
            crate::authoring_driver::document::package_checksum(&physical),
            vec![crate::authoring_driver::document::package_checksum(
                &modeling,
            )],
            &points,
            &cancel,
        )
        .await
        .unwrap();
    (
        PackageSources {
            physical,
            modeling: vec![modeling],
        },
        definition,
    )
}
#[test]
fn study_request_codec_unit_excludes_owner_seed_need_and_bare_overlays() {
    let case = pse_model::generated::identities::DeclarationId::from_bytes([4; 16]);
    let request = StudyRequest {
        version: pse_model::document::Version,
        points: vec![point(case, 7, vec![], StartPolicy::Fresh)],
    };
    let encoded = serde_json::to_value(request).unwrap();
    assert!(encoded["points"][0]["policy"].get("seed_need").is_none());
    let decoded: StudyRequest = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
    let mut bare = encoded;
    bare["points"][0]["overlay"] = serde_json::json!({"values":{"x":1}});
    assert!(serde_json::from_value::<StudyRequest>(bare).is_err());
}

#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn immutable_definition_equal_bindings_distinct_occurrences_and_usable_dependency_refusal() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker", quick()).await;
    let directory = tempfile::tempdir().unwrap();
    let workspace = runtime
        .register_workspace(
            "studies",
            url::Url::from_directory_path(directory.path()).unwrap(),
        )
        .await
        .unwrap();
    let (sources, definition) = admitted(&runtime, |root, failed| {
        vec![
            point(root, 2, vec![], StartPolicy::Fresh),
            point(root, 4, vec![], StartPolicy::Fresh),
            point(failed, 8, vec![], StartPolicy::Fresh),
            point(
                root,
                12,
                vec![Dependency::UsableResult(OccurrenceKey(8))],
                StartPolicy::Fresh,
            ),
        ]
    })
    .await;
    assert_eq!(
        definition.points[0].binding_hash,
        definition.points[1].binding_hash
    );
    let handle = runtime
        .start_defined_study(&workspace, sources, definition, RetryPolicy::ONCE, 0)
        .await
        .unwrap();
    while !matches!(runtime.work_once().await.unwrap(), Processed::Idle) {}
    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Published);
    assert_eq!(status.attempt_state, AttemptState::Partial);
    assert_eq!(status.points[3].state, StudyPointState::Failed);
    assert!(
        status.points[3]
            .outcome
            .as_ref()
            .unwrap()
            .diagnostic
            .is_some()
    );
    assert_ne!(status.points[3].state, StudyPointState::Cancelled);
    let published = handle.wait(Duration::from_millis(10)).await.unwrap();
    assert_eq!(published.attempt_id, status.attempt_id);
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn cancellation_preserves_one_outcome_for_every_unattempted_occurrence() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker", quick()).await;
    let directory = tempfile::tempdir().unwrap();
    let workspace = runtime
        .register_workspace(
            "cancelled",
            url::Url::from_directory_path(directory.path()).unwrap(),
        )
        .await
        .unwrap();
    let (sources, definition) = admitted(&runtime, |root, _| {
        vec![
            point(root, 3, vec![], StartPolicy::Fresh),
            point(
                root,
                9,
                vec![Dependency::Ordering(OccurrenceKey(3))],
                StartPolicy::Fresh,
            ),
        ]
    })
    .await;
    let handle = runtime
        .start_defined_study(&workspace, sources, definition, RetryPolicy::ONCE, 0)
        .await
        .unwrap();
    assert!(handle.cancel().await.unwrap().concluded);
    let status = handle.status().await.unwrap();
    assert_eq!(status.points.len(), 2);
    assert_eq!(status.attempt_state, AttemptState::Cancelled);
    assert!(status.points.iter().all(|point| {
        point.state == StudyPointState::Cancelled
            && point
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.attempts.is_empty() && !outcome.scientific.usable)
    }));
    drop(runtime);
    database.remove().await.unwrap();
}
