// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durable studies from the runtime's side (Plan 22 O7), against isolated PostgreSQL 18
//! databases: a worker in this process runs the points and the finalization. The
//! two-process journey is `tests/worker.rs` (`study_parallel_workers_publish_once`).
use super::durable_tests::quick;
use super::worker_tests::{ipopt, job_durable, sources};
use super::*;
use datafusion::arrow::array::RecordBatch;
use pse_columnar::CancellationToken;
use pse_operations::{
    jobs::{JobState, RetryPolicy},
    lifecycle::AttemptState,
    streams::ProgressValue,
    testing::TestDatabase,
};
use std::{collections::BTreeMap, time::Duration};

/// Two cases: `Root` takes its right-hand side from parameter `a`, `Other` has another
/// structure, so a seed of `Root` cannot start it.
const TWO_CASES: &str = r#"package algebraic {
def Root {
    param a:Scalar = 4;
    var x:Scalar;
    eq square:x*x==a;
    annotation start x(1);
    annotation bounds x(0,10);
    annotation report x("root");
    annotation check x(x>1);
}
def Other {
    var y:Scalar;
    var z:Scalar;
    eq sum:y+z==3;
    eq difference:y-z==1;
    annotation start y(1);
    annotation start z(1);
    annotation report y("y");
}
}"#;

fn operations(runtime: &Runtime) -> &Operations {
    match runtime.durability() {
        Durability::Durable(operations) => operations,
        Durability::Ephemeral => panic!("ephemeral"),
    }
}

/// The declaration of a named case in `source`'s package.
async fn case(
    runtime: &Runtime,
    source: &str,
    name: &str,
) -> pse_model::generated::identities::DeclarationId {
    let (physical, modeling) = sources(source);
    let cancel = crate::CancelSource::new();
    let context = runtime
        .physical_from_sources(&physical, &cancel)
        .await
        .unwrap();
    let package = runtime
        .package_from_sources(std::slice::from_ref(&modeling), context)
        .unwrap();
    package
        .declarations()
        .iter()
        .find(|d| d.name == name)
        .unwrap()
        .declaration_id
}

fn point(
    case: pse_model::generated::identities::DeclarationId,
    a: Option<f64>,
    predecessor: Option<u32>,
) -> StudyPoint {
    StudyPoint {
        case,
        overlay: PointOverlay {
            values: a
                .map(|a| BTreeMap::from([("a".to_owned(), a)]))
                .unwrap_or_default(),
            parameters: BTreeMap::new(),
        },
        predecessor,
    }
}

fn plan(source: &str, points: Vec<StudyPoint>) -> StudyPlan {
    let (physical, modeling) = sources(source);
    StudyPlan {
        sources: PackageSources {
            physical,
            modeling: vec![modeling],
        },
        route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
        settings: ipopt(),
        points,
        retry: RetryPolicy::ONCE,
        priority: 0,
    }
}

fn directory() -> (tempfile::TempDir, url::Url) {
    let directory = tempfile::tempdir().unwrap();
    let root = url::Url::from_directory_path(directory.path()).unwrap();
    (directory, root)
}

/// Run jobs in this process until none is available; the results of the tries that ran.
async fn drain(runtime: &Runtime) -> Vec<(Processed, Option<Arc<RunResult>>)> {
    let mut ran = Vec::new();
    loop {
        let (processed, result) = runtime.work_once_with_result().await.unwrap();
        if matches!(processed, Processed::Idle) {
            return ran;
        }
        ran.push((processed, result));
    }
}

fn start_values(record: &DurableRecord) -> BTreeMap<String, ProgressValue> {
    record
        .progress
        .as_ref()
        .unwrap()
        .iter()
        .find(|event| event.phase == "job.start")
        .unwrap()
        .values
        .clone()
}

async fn rows(publication: &pse_catalog::delta::publication::Publication, sql: &str) -> usize {
    publication
        .session()
        .sql(sql, &CancellationToken::new())
        .await
        .unwrap()
        .iter()
        .map(RecordBatch::num_rows)
        .sum()
}

/// A dependent point is not claimable before its predecessor completed and then starts
/// from the predecessor's stored solution; a dependent whose case has other coordinates
/// starts from its authored start and records why; a point whose predecessor failed never
/// runs.
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn predecessor_waits_and_seeds() {
    use pse_operations::studies::StudyPointState as P;
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker-a", quick()).await;
    let (_directory, root) = directory();
    let workspace = runtime.register_workspace("studies", root).await.unwrap();
    let root_case = case(&runtime, TWO_CASES, "Root").await;
    let other_case = case(&runtime, TWO_CASES, "Other").await;
    let handle = runtime
        .start_study(
            &workspace,
            plan(
                TWO_CASES,
                vec![
                    point(root_case, None, None),
                    point(root_case, Some(9.), Some(0)),
                    point(root_case, Some(-1.), None),
                    point(root_case, Some(16.), Some(2)),
                    point(other_case, None, Some(0)),
                ],
            ),
        )
        .await
        .unwrap();
    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Open);
    assert_eq!(
        status
            .points
            .iter()
            .map(|p| p.job_state)
            .collect::<Vec<_>>(),
        [
            JobState::Queued,
            JobState::Waiting,
            JobState::Queued,
            JobState::Waiting,
            JobState::Waiting
        ]
    );

    let ran = drain(&runtime).await;
    let status = handle.status().await.unwrap();
    assert_eq!(
        status.points.iter().map(|p| p.state).collect::<Vec<_>>(),
        [
            P::Completed,
            P::Completed,
            P::Failed,
            P::Cancelled,
            P::Completed
        ]
    );
    assert_eq!(status.state, StudyState::Published);
    assert_eq!(status.attempt_state, AttemptState::Partial);
    assert_eq!(
        status.points[3].error.as_deref(),
        Some("predecessor point 2 failed")
    );
    let record = |point: usize| {
        ran.iter()
            .find_map(|(processed, result)| match processed {
                Processed::Ran { record, .. }
                    if record.attempt_id == status.points[point].attempt_id =>
                {
                    Some((record.clone(), result.clone()))
                }
                _ => None,
            })
            .unwrap()
    };
    // Point 1 ran after point 0 and started from point 0's stored solution.
    let (zero, _) = record(0);
    let (one, one_result) = record(1);
    let [(0, seed)] = zero.solutions[..] else {
        panic!("{:?}", zero.solutions)
    };
    let start = start_values(&one);
    assert_eq!(
        start["requested"],
        ProgressValue::Text("predecessor".into())
    );
    assert_eq!(start["solution"], ProgressValue::Text(seed.to_string()));
    let RunRequest::Modeling(steps) = one_result.unwrap().request().clone() else {
        panic!("a modeling run")
    };
    assert!(steps[0].starts.values().any(|s| *s
        == StartSource::Stored {
            solution: seed.as_id()
        }));
    // Point 4's case has other coordinates: it starts fresh, saying why.
    let (four, _) = record(4);
    let start = start_values(&four);
    assert_eq!(
        start["requested"],
        ProgressValue::Text("predecessor".into())
    );
    let ProgressValue::Text(fresh) = &start["fresh"] else {
        panic!("{start:?}")
    };
    assert!(fresh.contains("predecessor point 0"), "{fresh}");
    // The point cancelled behind its failed predecessor never ran.
    assert_eq!(status.points[3].attempt_state, AttemptState::Cancelled);
    assert!(!ran.iter().any(|(processed, _)| matches!(
        processed,
        Processed::Ran { record, .. } if record.attempt_id == status.points[3].attempt_id
    )));
    drop(ran);
    drop(runtime);
    database.remove().await.unwrap();
}

/// A failed point contributes no members and does not stop the others: the study's one
/// publication holds the summary of every point and the members of every completed point,
/// under the intent registered at creation, and names the study's own attempt.
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn failed_point_does_not_contaminate() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker-a", quick()).await;
    let (_directory, root) = directory();
    let workspace = runtime.register_workspace("studies", root).await.unwrap();
    let root_case = case(&runtime, TWO_CASES, "Root").await;
    let handle = runtime
        .start_study(
            &workspace,
            plan(
                TWO_CASES,
                vec![
                    point(root_case, Some(4.), None),
                    point(root_case, Some(-1.), None),
                    point(root_case, Some(25.), None),
                ],
            ),
        )
        .await
        .unwrap();
    let status = handle.status().await.unwrap();
    let intent = operations(&runtime)
        .store()
        .catalog()
        .intent(status.publication_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(intent.attempt_id, status.attempt_id);
    drain(&runtime).await;
    let published = handle.wait(Duration::from_millis(10)).await.unwrap();
    assert_eq!(published.publication_id, status.publication_id);
    assert_eq!(published.attempt_id, status.attempt_id);
    assert_eq!(
        runtime.head(workspace.workspace_id).await.unwrap(),
        Some(published.publication_id)
    );
    let record = operations(&runtime)
        .store()
        .catalog()
        .publication(published.publication_id)
        .await
        .unwrap()
        .unwrap();
    // Every member lies under the study's intent; the failed point wrote none.
    assert!(
        record
            .members
            .iter()
            .all(|m| m.table_uri.starts_with(&intent.member_prefix))
    );
    let catalogs: std::collections::BTreeSet<_> = record
        .members
        .iter()
        .map(|m| m.catalog_name.as_str())
        .collect();
    assert_eq!(
        catalogs.into_iter().collect::<Vec<_>>(),
        ["point_0", "point_2", "study"]
    );
    let cancel = CancellationToken::new();
    let reader = runtime
        .open(published.publication_id, &cancel)
        .await
        .unwrap();
    let publication = reader.publication();
    assert_eq!(
        rows(publication, "SELECT * FROM study.runtime.study_outcomes").await,
        3
    );
    assert_eq!(
        rows(
            publication,
            "SELECT * FROM study.runtime.study_outcomes WHERE member_catalog IS NULL AND error IS NOT NULL"
        )
        .await,
        1
    );
    let point_tables = record
        .members
        .iter()
        .filter(|m| m.catalog_name == "point_0")
        .count();
    assert!(point_tables > 0);
    assert_eq!(
        record
            .members
            .iter()
            .filter(|m| m.catalog_name == "point_2")
            .count(),
        point_tables
    );
    drop(reader);
    drop(runtime);
    database.remove().await.unwrap();
}

/// Cancelling a study before any point ran cancels every point; the study concludes as
/// cancelled and still publishes its summary.
#[tokio::test]
async fn study_cancel_stops_pending_points() {
    use pse_operations::studies::StudyPointState as P;
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker-a", quick()).await;
    let (_directory, root) = directory();
    let workspace = runtime.register_workspace("studies", root).await.unwrap();
    let root_case = case(&runtime, TWO_CASES, "Root").await;
    let handle = runtime
        .start_study(
            &workspace,
            plan(
                TWO_CASES,
                vec![
                    point(root_case, None, None),
                    point(root_case, Some(9.), Some(0)),
                    point(root_case, Some(16.), None),
                ],
            ),
        )
        .await
        .unwrap();
    let cancelled = handle.cancel().await.unwrap();
    assert_eq!(cancelled.cancelled, [0, 1, 2]);
    assert!(cancelled.concluded);
    let ran = drain(&runtime).await;
    // Only the finalization ran.
    assert_eq!(ran.len(), 1);
    let status = handle.status().await.unwrap();
    assert_eq!(
        status.points.iter().map(|p| p.state).collect::<Vec<_>>(),
        [P::Cancelled, P::Cancelled, P::Cancelled]
    );
    assert_eq!(status.attempt_state, AttemptState::Cancelled);
    let published = handle.result().await.unwrap().unwrap();
    let record = operations(&runtime)
        .store()
        .catalog()
        .publication(published.publication_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.members.len(), 1);
    assert_eq!(record.members[0].catalog_name, "study");
    // The study is listed with its state.
    let listed = runtime.studies(&StudyFilter::newest(10)).await.unwrap();
    assert_eq!(listed.batch().num_rows(), 1);
    drop(ran);
    drop(runtime);
    database.remove().await.unwrap();
}
