// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Focused native endpoint, topology and query-plan controls for Plan 35.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "isolated native fixtures and independent assertions fail on unexpected results"
)]

use crate::{
    canonical::{
        CanonicalError, CanonicalOptions, CanonicalStore, ObjectEdit, Revision, bounded_query,
    },
    canonical_analyses::{AnalysisEdge, AnalysisNode, seal_analysis_request},
    canonical_codec as codec,
    canonical_execution::RunRequest,
    canonical_studies::{NewOccurrence, point_key},
    generated::surreal as wire,
};
use pse_model::{
    generated::runtime::{
        canonical_analysis_inputs, canonical_memberships, canonical_problem_runs,
        canonical_run_sources, canonical_versions,
    },
    study::{ActionKind, Dependency, OccurrenceKey, PointPolicy, SeedNeed, StartPolicy},
};
use surrealdb::types::{NotFoundError, Object, ToSql, Value};

async fn fixture() -> CanonicalStore {
    let state =
        std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
    let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_capabilities_{}",
        uuid::Uuid::new_v4().simple()
    );
    crate::testing::canonical_fixture_with_options(&options, true).unwrap()
}

fn edit(version: &str, logical: &str, scope: &str, kind: &str, supplier: bool) -> ObjectEdit {
    ObjectEdit {
        logical: logical.into(),
        scope: scope.into(),
        name: logical.into(),
        version: Some(canonical_versions::Row {
            key: version.into(),
            logical: logical.into(),
            kind: kind.into(),
            payload: logical.as_bytes().to_vec().into(),
            interpretation: wire::INTERPRETATION.into(),
        }),
        references: if supplier {
            vec![("target-scope".into(), "target-name".into())]
        } else {
            Vec::new()
        },
    }
}

fn request(revision: &Revision, key: &str) -> RunRequest {
    RunRequest {
        key: key.into(),
        revision: revision.clone(),
        sources: Vec::new(),
        request: vec![1],
        source_selection: vec![2],
        attestation: vec![3],
    }
}

struct Endpoint {
    table: &'static str,
    key: String,
    row: Object,
}
async fn endpoint(store: &CanonicalStore, table: &'static str, key: &str) -> Endpoint {
    let row: Option<Object> = store.db.select((table, key)).await.unwrap();
    let mut row = row.unwrap();
    row.remove("id");
    Endpoint {
        table,
        key: key.into(),
        row,
    }
}
impl Endpoint {
    fn renamed(&self, key: &str) -> Object {
        let mut row = self.row.clone();
        row.insert("key", Value::String(key.into()));
        // These independent native endpoint fixtures must also satisfy the
        // existing unique indexes; they confer no operational/scientific authority.
        match self.table {
            "canonical_revisions" => {
                row.insert("operation", Value::String(key.into()));
                row.insert("problem", Value::String(key.into()));
            }
            "canonical_runs" => {
                row.insert("problem", Value::String(key.into()));
            }
            "canonical_attempts" => {
                row.insert("run", Value::String(key.into()));
            }
            _ => {}
        }
        row
    }
}
struct RelationCase {
    table: &'static str,
    source_field: &'static str,
    target_field: &'static str,
    source: Endpoint,
    target: Endpoint,
    row: Object,
}
impl RelationCase {
    fn row(&self, key: &str, source: &str, target: &str) -> Object {
        let mut row = self.row.clone();
        row.insert("id", codec::record_link(self.table, key));
        row.insert("key", Value::String(key.into()));
        row.insert(self.source_field, Value::String(source.into()));
        row.insert(self.target_field, Value::String(target.into()));
        row.insert("in", codec::record_link(self.source.table, source));
        row.insert("out", codec::record_link(self.target.table, target));
        row
    }
}

#[tokio::test]
async fn selected_enforced_relations_refuse_each_missing_endpoint_and_accept_same_transaction_creation()
 {
    let store = fixture().await;
    let revision = store
        .edit(
            "problem",
            None,
            "source",
            &[edit("version", "root", "scope", "leaf", false)],
        )
        .await
        .unwrap();
    store.begin_run(&request(&revision, "run")).await.unwrap();
    let mut analysis = store
        .new_analysis(
            &revision,
            "endpoint-control:v1",
            vec![1],
            "input".into(),
            2,
            1,
        )
        .await
        .unwrap();
    let nodes = [
        AnalysisNode {
            key: "node-a".into(),
            analysis: analysis.key.clone(),
            semantic: "a".into(),
            kind: "variable".into(),
        },
        AnalysisNode {
            key: "node-b".into(),
            analysis: analysis.key.clone(),
            semantic: "b".into(),
            kind: "row".into(),
        },
    ];
    let edge = AnalysisEdge {
        key: "edge".into(),
        analysis: analysis.key.clone(),
        source: nodes[0].key.clone(),
        target: nodes[1].key.clone(),
        kind: "incidence".into(),
        evidence_artifact: None, evidence_block: None, evidence_row: None, evidence_field: None,
    };
    seal_analysis_request(
        &mut analysis,
        std::slice::from_ref(&revision),
        &[],
        &nodes,
        std::slice::from_ref(&edge),
    );
    store
        .persist_analysis(
            &analysis,
            std::slice::from_ref(&revision),
            &[],
            &nodes,
            std::slice::from_ref(&edge),
        )
        .await
        .unwrap();
    let mut artifact=store.new_artifact(&revision,"endpoint-fixture",vec![],1,1).await.unwrap();
    let block=crate::canonical_artifacts::artifact_block(&artifact.key,0,"fixture",vec![1],"original",1);
    crate::canonical_artifacts::seal_artifact_request(&mut artifact,std::slice::from_ref(&revision),&[],std::slice::from_ref(&block),"user","fixture");
    store.persist_artifact(&artifact,std::slice::from_ref(&revision),&[],std::slice::from_ref(&block),"user","fixture").await.unwrap();
    let cases = [
        RelationCase {
            table: "canonical_memberships",
            source_field: "problem",
            target_field: "version",
            source: endpoint(&store, "canonical_problems", "problem").await,
            target: endpoint(&store, "canonical_version_manifests", "version").await,
            row: wire::encode_canonical_memberships(&canonical_memberships::Row {
                key: "control".into(),
                problem: "problem".into(),
                scope: "scope".into(),
                name: "root".into(),
                logical: "root".into(),
                version: "version".into(),
                from_sequence: revision.sequence,
                to_sequence: None,
            })
            .unwrap(),
        },
        RelationCase {
            table: "canonical_analysis_edges",
            source_field: "source",
            target_field: "target",
            source: endpoint(&store, "canonical_analysis_nodes", "node-a").await,
            target: endpoint(&store, "canonical_analysis_nodes", "node-b").await,
            row: wire::encode_canonical_analysis_edges(&edge).unwrap(),
        },
        RelationCase {
            table: "canonical_analysis_inputs",
            source_field: "analysis",
            target_field: "artifact",
            source: endpoint(&store, "canonical_analyses", &analysis.key).await,
            target: endpoint(&store, "canonical_artifacts", &artifact.key).await,
            row: wire::encode_canonical_analysis_inputs(&canonical_analysis_inputs::Row {
                key: "control".into(),
                analysis: analysis.key.clone(),
                artifact: artifact.key.clone(),
            })
            .unwrap(),
        },
        RelationCase {
            table: "canonical_problem_runs",
            source_field: "problem",
            target_field: "run",
            source: endpoint(&store, "canonical_problems", "problem").await,
            target: endpoint(&store, "canonical_runs", "run").await,
            row: wire::encode_canonical_problem_runs(&canonical_problem_runs::Row {
                key: "control".into(),
                problem: "problem".into(),
                run: "run".into(),
            })
            .unwrap(),
        },
        RelationCase {
            table: "canonical_run_sources",
            source_field: "run",
            target_field: "revision",
            source: endpoint(&store, "canonical_runs", "run").await,
            target: endpoint(&store, "canonical_revisions", &revision.key).await,
            row: wire::encode_canonical_run_sources(&canonical_run_sources::Row {
                key: "control".into(),
                run: "run".into(),
                revision: revision.key.clone(),
            })
            .unwrap(),
        },
    ];
    for case in cases {
        for missing_source in [true, false] {
            let key = format!(
                "{}-missing-{}",
                case.table,
                if missing_source { "in" } else { "out" }
            );
            let source = if missing_source {
                key.as_str()
            } else {
                &case.source.key
            };
            let target = if missing_source {
                &case.target.key
            } else {
                key.as_str()
            };
            let error = bounded_query(
                store
                    .db
                    .query(format!("INSERT RELATION INTO {} $row;", case.table))
                    .bind(("row", case.row(&key, source, target))),
            )
            .await
            .unwrap_err();
            let CanonicalError::Driver(error) = error else {
                panic!("unexpected native refusal: {error}")
            };
            let missing_table = if missing_source {
                case.source.table
            } else {
                case.target.table
            };
            let expected_id = codec::record_link(missing_table, &key).to_sql();
            assert!(
                matches!(error.not_found_details(), Some(NotFoundError::Record { id }) if id == &expected_id),
                "{} must identify the absent endpoint natively: {error}",
                case.table
            );
            let saved: Option<Object> = store.db.select((case.table, key.as_str())).await.unwrap();
            assert!(
                saved.is_none(),
                "failed endpoint admission must not create an edge"
            );
        }
        let key = format!("{}-same-transaction", case.table);
        let source = format!("{key}-source");
        let target = format!("{key}-target");
        bounded_query(store.db.query(format!(
            "BEGIN; CREATE type::record($source_table,$source_key) CONTENT $source; CREATE type::record($target_table,$target_key) CONTENT $target; INSERT RELATION INTO {} $row; COMMIT;", case.table
        )).bind(("source_table", case.source.table)).bind(("source_key", source.clone())).bind(("source", case.source.renamed(&source)))
            .bind(("target_table", case.target.table)).bind(("target_key", target.clone())).bind(("target", case.target.renamed(&target)))
            .bind(("row", case.row(&key, &source, &target)))).await.unwrap();
        let saved: Option<Object> = store.db.select((case.table, key.as_str())).await.unwrap();
        let saved = saved.unwrap();
        assert_eq!(
            saved.get("in"),
            Some(&codec::record_link(case.source.table, &source))
        );
        assert_eq!(
            saved.get("out"),
            Some(&codec::record_link(case.target.table, &target))
        );
    }
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn reverse_study_dag_crosses_native_ingestion_page_without_requiring_future_endpoints() {
    let store = fixture().await;
    let revision = store.edit("problem", None, "source", &[]).await.unwrap();
    // Authoring order is independent of dependency order. The first transaction
    // stores a dependency on an occurrence whose endpoint arrives in page two.
    let points = (0..65)
        .map(|key| NewOccurrence {
            policy: PointPolicy {
                key: OccurrenceKey(key),
                dependencies: if key == 64 {
                    Vec::new()
                } else {
                    vec![Dependency::Ordering(OccurrenceKey(key + 1))]
                },
                start: StartPolicy::Fresh,
                seed_need: SeedNeed::NotNeeded,
                attempt_limit: 1,
            },
            descriptor: vec![42],
            run: request(&revision, &format!("point-{key}")),
        })
        .collect::<Vec<_>>();
    let parent = request(&revision, "parent");
    let study = store
        .create_study("reverse", &parent, &[9], &points, &|| false)
        .await
        .unwrap()
        .unwrap();
    assert!(study.active && !study.cancelled);
    assert_eq!(study.point_count, 65);
    assert_eq!(study.next_ordinal, 65);
    let first = store.study_point_page("reverse", None).await.unwrap();
    assert_eq!(first.len(), 64);
    let second = store
        .study_point_page("reverse", Some(first.last().unwrap().ordinal))
        .await
        .unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].ordinal, 64);
    let boundary = store
        .study_scope(&point_key("reverse", OccurrenceKey(63)))
        .await
        .unwrap();
    assert_eq!(boundary.predecessors().len(), 1);
    assert_eq!(boundary.predecessors()[0].occurrence, 64);
    assert!(matches!(
        boundary.action(None).unwrap().kind,
        ActionKind::Wait(_)
    ));
    let last = store
        .study_scope(&point_key("reverse", OccurrenceKey(64)))
        .await
        .unwrap();
    assert!(last.predecessors().is_empty());
    assert!(matches!(
        last.action(None).unwrap().kind,
        ActionKind::Start(_)
    ));
    assert_eq!(
        store
            .create_study("reverse", &parent, &[9], &points, &|| false)
            .await
            .unwrap()
            .unwrap(),
        study
    );
    let mut response = bounded_query(
        store
            .db
            .query("SELECT * FROM canonical_study_dependencies WHERE study='reverse';"),
    )
    .await
    .unwrap();
    let edges: Vec<Object> = response.take(0).unwrap();
    assert_eq!(edges.len(), 64);
    store.remove_isolated_fixture().await.unwrap();
}

fn plan_nodes<'a>(value: &'a Value, nodes: &mut Vec<&'a Object>) {
    match value {
        Value::Object(object) => {
            if object.contains_key("operator") || object.contains_key("operation") {
                nodes.push(object);
            }
            for value in object.values() {
                plan_nodes(value, nodes);
            }
        }
        Value::Array(array) => {
            for value in array.iter() {
                plan_nodes(value, nodes);
            }
        }
        _ => {}
    }
}
fn operator(node: &Object) -> &str {
    match node.get("operator").or_else(|| node.get("operation")) {
        Some(Value::String(name)) => name,
        _ => panic!("native plan operator missing: {node:?}"),
    }
}
fn plan_index(node: &Object) -> Option<&str> {
    // The 3.3 planner emits attributes.index. The retained legacy explanation
    // surface emits detail.plan.index; both are native evidence, never inferred.
    if let Some(Value::Object(attributes)) = node.get("attributes")
        && let Some(Value::String(index)) = attributes.get("index")
    {
        return Some(index);
    }
    if let Some(Value::Object(detail)) = node.get("detail")
        && let Some(Value::Object(plan)) = detail.get("plan")
        && let Some(Value::String(index)) = plan.get("index")
    {
        return Some(index);
    }
    None
}
fn assert_native_plan(plan: &Value, indexed: Option<&str>, noindex: bool) {
    let mut nodes = Vec::new();
    plan_nodes(plan, &mut nodes);
    assert!(
        !nodes.is_empty(),
        "native EXPLAIN must return an actual plan: {plan:?}"
    );
    if noindex {
        assert!(
            nodes.iter().any(|node| matches!(
                operator(node),
                "TableScan" | "Iterate Table" | "Iterate Table Keys"
            )),
            "WITH NOINDEX must execute the table-scan control: {plan:?}"
        );
        assert!(
            !nodes.iter().any(|node| plan_index(node).is_some()),
            "WITH NOINDEX must not acquire a secondary index: {plan:?}"
        );
    } else {
        assert!(
            nodes.iter().any(|node| plan_index(node).is_some()),
            "canonical predicates must expose an existing indexed access path: {plan:?}"
        );
        if let Some(indexed) = indexed {
            assert!(
                nodes.iter().any(|node| plan_index(node) == Some(indexed)),
                "the existing-index control must use {indexed}: {plan:?}"
            );
        }
    }
}
fn assert_executed_cardinality(plan: &Value, rows: usize) {
    // At 3.3, trailing EXPLAIN FULL invokes AnalyzePlan and reports total_rows.
    // Legacy Fetch.count is also post-LIMIT. Neither is a count of examined rows.
    if let Value::Object(object) = plan
        && let Some(total) = object.get("total_rows")
    {
        assert_eq!(codec::decode_int(total.clone()).unwrap(), rows as i64);
        let mut nodes = Vec::new();
        plan_nodes(plan, &mut nodes);
        assert!(
            nodes.iter().any(|node| node.contains_key("metrics")),
            "EXPLAIN FULL must execute and report native operator metrics"
        );
        return;
    }
    let mut nodes = Vec::new();
    plan_nodes(plan, &mut nodes);
    let fetch = nodes
        .iter()
        .find(|node| operator(node) == "Fetch")
        .expect("EXPLAIN FULL must report execution cardinality");
    let Some(Value::Object(detail)) = fetch.get("detail") else {
        panic!("native Fetch detail missing")
    };
    assert_eq!(
        codec::decode_int(detail.get("count").unwrap().clone()).unwrap(),
        rows as i64
    );
}

async fn observe_query(store: &CanonicalStore, revision: &Revision, query: &str) -> Value {
    let mut response = bounded_query(
        store
            .db
            .query(query.to_owned())
            .bind(("problem", revision.problem.clone()))
            .bind(("scope", "selected"))
            .bind(("after", ""))
            .bind(("sequence", codec::encode_uint(revision.sequence).unwrap()))
            .bind(("source_kind", "leaf"))
            .bind(("target_scope", "target-scope"))
            .bind(("target_name", "target-name")),
    )
    .await
    .unwrap();
    response.take(response.num_statements() - 1).unwrap()
}

#[tokio::test]
async fn native_canonical_interval_dereference_and_supplier_plans_report_access_and_output_separately()
 {
    let store = fixture().await;
    let initial = (0..192)
        .map(|index| {
            edit(
                &format!("version-{index:03}"),
                &format!("object-{index:03}"),
                if index < 96 { "selected" } else { "noise" },
                "leaf",
                index % 2 == 0,
            )
        })
        .collect::<Vec<_>>();
    let old = store
        .edit("plan-problem", None, "plan-source", &initial)
        .await
        .unwrap();
    let replacements = (0..16)
        .map(|index| {
            edit(
                &format!("replacement-{index:03}"),
                &format!("object-{index:03}"),
                "selected",
                "other",
                index % 2 == 0,
            )
        })
        .collect::<Vec<_>>();
    let revision = store
        .edit(
            "plan-problem",
            Some(&old.key),
            "plan-replacement",
            &replacements,
        )
        .await
        .unwrap();
    let mut artifact=store.new_artifact(&revision,"endpoint-fixture",vec![],1,1).await.unwrap();
    let block=crate::canonical_artifacts::artifact_block(&artifact.key,0,"fixture",vec![1],"original",1);
    crate::canonical_artifacts::seal_artifact_request(&mut artifact,std::slice::from_ref(&revision),&[],std::slice::from_ref(&block),"user","fixture");
    store.persist_artifact(&artifact,std::slice::from_ref(&revision),&[],std::slice::from_ref(&block),"user","fixture").await.unwrap();
    let cases = [
        (
            "membership-interval",
            "",
            "",
            "",
            "membership_scope_pages",
            64usize,
        ),
        (
            "membership-kind-dereference",
            "",
            " AND ($source_kind = NONE OR (out.kind = $source_kind AND out.closed = true))",
            "",
            "membership_scope_pages",
            64,
        ),
        (
            "inverse-supplier-membership",
            "LET $suppliers = SELECT VALUE source_version FROM canonical_edges WHERE target_scope=$target_scope AND target_name=$target_name TIMEOUT 5s;",
            " AND ($source_kind = NONE OR (out.kind = $source_kind AND out.closed = true))",
            " AND version IN $suppliers",
            "membership_scope_pages",
            40,
        ),
    ];
    for (label, prefix, kind, suppliers, index, count) in cases {
        let mut expected = None;
        for (hint, noindex, required_index) in [
            ("", false, None),
            (" WITH INDEX membership_scope_pages", false, Some(index)),
            (" WITH NOINDEX", true, None),
        ] {
            // This is the actual production interval/dereference shape from
            // canonical_selection, with only an explicit bounded diagnostic timeout.
            let query = format!(
                "{prefix} SELECT * FROM canonical_memberships{hint} WHERE problem=$problem AND scope=$scope AND key>$after AND from_sequence<=$sequence AND (to_sequence=NONE OR to_sequence>$sequence){kind}{suppliers} ORDER BY key LIMIT 64 TIMEOUT 5s"
            );
            let result = observe_query(&store, &revision, &format!("{query};")).await;
            let Value::Array(rows) = &result else {
                panic!("native SELECT must return rows: {result:?}")
            };
            assert_eq!(rows.len(), count, "{label}{hint}");
            if let Some(expected) = &expected {
                assert_eq!(
                    &result, expected,
                    "access-path controls must retain exact ordering and interval semantics"
                );
            } else {
                expected = Some(result.clone());
            }
            let mut previous_key: Option<String> = None;
            for row in rows.iter() {
                let Value::Object(row) = row else {
                    panic!("membership row is not native object")
                };
                let member = wire::decode_canonical_memberships(row.clone()).unwrap();
                assert_eq!(member.problem, revision.problem);
                assert_eq!(member.scope, "selected");
                assert!(
                    previous_key
                        .as_ref()
                        .is_none_or(|previous| previous < &member.key)
                );
                previous_key = Some(member.key.clone());
                assert!(member.from_sequence <= revision.sequence);
                assert!(member.to_sequence.is_none_or(|end| end > revision.sequence));
                if !kind.is_empty() {
                    assert!(!member.version.starts_with("replacement-"));
                }
            }
            let plan = observe_query(&store, &revision, &format!("{query} EXPLAIN;")).await;
            let full = observe_query(&store, &revision, &format!("{query} EXPLAIN FULL;")).await;
            assert_native_plan(&plan, required_index, noindex);
            assert_native_plan(&full, required_index, noindex);
            assert_executed_cardinality(&full, rows.len());
            // Preserve native collectors/sorts, scan access, records_fetched and
            // all other reported metrics. LIMIT/total_rows/Fetch.count establish
            // output cardinality; they do not establish total examined work.
            eprintln!(
                "{label}{hint}: fixture_memberships=208 limit=64 output_rows={} examined_rows=not_established explain={} explain_full={}",
                rows.len(),
                plan.to_sql(),
                full.to_sql()
            );
        }
    }
    // Inspect the exact inverse-supplier access independently: the membership
    // query's LET result is not an explanation of the edge lookup itself.
    for (hint, noindex, required) in [
        ("", false, None),
        (" WITH INDEX edge_target", false, Some("edge_target")),
        (" WITH NOINDEX", true, None),
    ] {
        let query = format!(
            "SELECT VALUE source_version FROM canonical_edges{hint} WHERE target_scope=$target_scope AND target_name=$target_name TIMEOUT 5s"
        );
        let result = observe_query(&store, &revision, &format!("{query};")).await;
        let Value::Array(rows) = &result else {
            panic!("inverse supplier query must return native rows")
        };
        assert_eq!(
            rows.len(),
            104,
            "historical immutable supplier versions remain candidates until interval selection"
        );
        let plan = observe_query(&store, &revision, &format!("{query} EXPLAIN;")).await;
        let full = observe_query(&store, &revision, &format!("{query} EXPLAIN FULL;")).await;
        assert_native_plan(&plan, required, noindex);
        assert_native_plan(&full, required, noindex);
        assert_executed_cardinality(&full, rows.len());
        eprintln!(
            "inverse-supplier-edge{hint}: output_rows={} examined_rows=not_established explain={} explain_full={}",
            rows.len(),
            plan.to_sql(),
            full.to_sql()
        );
    }
    store.remove_isolated_fixture().await.unwrap();
}
