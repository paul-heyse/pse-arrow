// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P2 observes duplicate candidate rows and preserves exact diagnostic/provenance keys.
#[path = "support/native_check_execution.rs"]
mod native_check_execution;
#[path = "support/row_key.rs"]
mod row_key;
use datafusion::arrow::array::RecordBatch;
use pse_columnar::CancellationToken;
use pse_engine::session::{EngineSession, ExecutionSettings, ThreadBudget};
use pse_ids::{ContentHash, SemanticId};
use pse_rules::invariants::{InvariantScope, run_invariants};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

#[expect(clippy::unwrap_used, reason = "fixed test fixture admission")]
fn budget() -> ThreadBudget {
    ThreadBudget {
        pool_threads: NonZeroUsize::new(1).unwrap(),
        target_partitions: NonZeroUsize::new(1).unwrap(),
    }
}
fn session(registry: &Arc<Registry>, rows: &BTreeMap<RelationKey, RecordBatch>) -> EngineSession {
    session_with_observation(
        registry,
        rows,
        pse_engine::session::assurance::ObservationPolicy::default(),
    )
}
#[expect(clippy::unwrap_used, reason = "fixed test fixture admission")]
fn session_with_observation(
    registry: &Arc<Registry>,
    rows: &BTreeMap<RelationKey, RecordBatch>,
    observation: pse_engine::session::assurance::ObservationPolicy,
) -> EngineSession {
    pse_testkit::factory(
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20)),
        ExecutionSettings::default(),
        budget(),
    )
    .map(|factory| factory.with_observation(observation))
    .and_then(|factory| {
        factory.candidate(
            rows.clone(),
            Arc::clone(registry),
            &CancellationToken::new(),
        )
    })
    .unwrap()
}
#[expect(clippy::unwrap_used, reason = "fixed test fixture admission")]
fn packages(registry: &Registry, count: usize) -> BTreeMap<RelationKey, RecordBatch> {
    let spec = registry.relation("authored.packages").unwrap();
    let row = vec![
        serde_json::json!(["id", (SemanticId::from_bytes([7; 16])).to_hex()]),
        serde_json::json!(["text", "fixture"]),
        serde_json::json!(["text", "1.0.0"]),
        serde_json::json!(["enum", "model"]),
        serde_json::json!(["enum", "explicit"]),
        serde_json::json!(["list", []]),
        serde_json::json!(["hash", (ContentHash::from_bytes([0; 32])).to_hex()]),
        serde_json::json!(["text", "Fixture"]),
    ];
    BTreeMap::from([(
        spec.key,
        pse_relations::testing::batch_from_literals(registry, spec, &vec![row; count]).unwrap(),
    )])
}

#[tokio::test]
async fn provider_requirements_execute_the_same_primary_key_obligation_before_scalar_queries() {
    use pse_schema::model::provider::{ProviderPolicy, ProviderScope};
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let required = registry
        .invariants()
        .iter()
        .find(|invariant| {
            invariant.relation == "authored.packages" && invariant.name == "unique:pk"
        })
        .unwrap()
        .id;
    let factory = pse_testkit::factory(
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20)),
        ExecutionSettings::default(),
        budget(),
    )
    .unwrap()
    .with_requirement_planner(Arc::new(pse_rules::invariants::RegistryRequirementPlanner));
    for (count, valid) in [(0, true), (1, true), (2, false)] {
        let cancel = CancellationToken::new();
        let mut policy = ProviderPolicy::new(SemanticId::from_bytes([8; 16]), ProviderScope::Root);
        policy.requirements.insert(required);
        let session = factory
            .candidate(packages(&registry, count), Arc::clone(&registry), &cancel)
            .unwrap()
            .with_policy(policy)
            .unwrap();
        let prepared = session.prepare_sql("SELECT 42", &cancel).await.unwrap();
        assert_eq!(prepared.execute(&cancel).await.is_ok(), valid);
    }
}

fn physical_subject(
    kind: &'static str,
    member: &serde_json::Value,
    phase: &serde_json::Value,
    law: bool,
) -> serde_json::Value {
    let mut fields = vec![serde_json::json!(["enum", kind])];
    for arm in [
        "total",
        "energy",
        "momentum",
        "species",
        "element",
        "phase_species",
    ] {
        fields.push(if arm == kind {
            serde_json::json!([
                "struct",
                match arm {
                    "total" | "energy" | "momentum" => vec![phase.clone()],
                    "species" | "element" if !law => vec![member.clone()],
                    _ => vec![member.clone(), phase.clone()],
                }
            ])
        } else {
            serde_json::json!(["null", null])
        });
    }
    serde_json::json!(["struct", fields])
}

#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "one declared alternative fixture and its refusal matrix"
)]
async fn subject_alternatives_admit_indexed_physics_and_reject_overlapping_coordinates() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let spec = registry
        .relation("authored.template_contribution_contracts")
        .unwrap();
    // Typed alternatives make missing/overlapping arm payloads structural errors.
    // The remaining relational rule forbids reusing one axis for two coordinates.
    let axis = |position| {
        serde_json::json!([
            "struct",
            vec![
                serde_json::json!(["enum", "axis"]),
                serde_json::json!(["null", null]),
                serde_json::json!(["struct", vec![serde_json::json!(["i64", position])]]),
            ]
        ])
    };
    let fixed = || {
        serde_json::json!([
            "struct",
            vec![
                serde_json::json!(["enum", "fixed"]),
                serde_json::json!([
                    "struct",
                    vec![serde_json::json!(["id", (SemanticId::NIL).to_hex()])]
                ]),
                serde_json::json!(["null", null]),
            ]
        ])
    };
    let cases = [
        physical_subject(
            "energy",
            &serde_json::json!(["null", null]),
            &serde_json::json!(["null", null]),
            false,
        ),
        physical_subject(
            "energy",
            &serde_json::json!(["null", null]),
            &axis(0),
            false,
        ),
        physical_subject(
            "energy",
            &serde_json::json!(["null", null]),
            &fixed(),
            false,
        ),
        physical_subject(
            "species",
            &axis(0),
            &serde_json::json!(["null", null]),
            false,
        ),
        physical_subject(
            "species",
            &fixed(),
            &serde_json::json!(["null", null]),
            false,
        ),
        physical_subject("phase_species", &axis(1), &axis(0), false),
        physical_subject("phase_species", &fixed(), &axis(0), false),
        physical_subject("phase_species", &fixed(), &fixed(), false),
        physical_subject("phase_species", &axis(0), &axis(0), false),
    ];
    let expected = std::collections::BTreeSet::from([vec![9; 16]]);
    let values = cases
        .into_iter()
        .enumerate()
        .map(|(index, subject)| {
            vec![
                serde_json::json!([
                    "id",
                    (SemanticId::from_bytes([u8::try_from(index + 1).unwrap(); 16],)).to_hex()
                ]),
                serde_json::json!(["list", []]),
                serde_json::json!(["id", (SemanticId::NIL).to_hex()]),
                subject,
                serde_json::json!(["null", null]),
            ]
        })
        .collect::<Vec<_>>();
    let actual: std::collections::BTreeSet<_> = native_check_execution::invalid_keys(
        &registry,
        spec,
        &values,
        "distinct_subject_axes",
        spec.primary_key[0],
    )
    .into_iter()
    .collect();
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn p2_keeps_candidate_duplicates_and_never_invents_execution_or_snapshot_identity() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let rows = packages(&registry, 2);
    let session = session_with_observation(
        &registry,
        &rows,
        pse_engine::session::assurance::ObservationPolicy::Diagnostic,
    );
    let report = run_invariants(
        &rows,
        &session,
        &registry,
        InvariantScope::Candidate,
        &CancellationToken::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.error_count(), 1);
    let finding = pse_relations::generated::runtime::diagnostics_findings::View::from_checked(
        &report.findings()[0],
    )
    .unwrap();
    let source = registry.relation("authored.packages").unwrap();
    assert_eq!(
        finding
            .row(0)
            .unwrap()
            .evidence
            .row
            .unwrap()
            .row_key
            .as_bytes(),
        row_key::values(
            &registry,
            source.id,
            &[source.column("package_id").unwrap()],
            &[serde_json::json!([
                "id",
                (SemanticId::from_bytes([7; 16])).to_hex()
            ])]
        )
        .await
        .as_bytes()
    );
    let completed = report.completion().unwrap();
    assert!(completed.prepared().observation().physical_plan().is_none());
    assert!(completed.observation().physical_plan().is_some());
    assert!(!completed.physical_plan().name().is_empty());
    let observed = completed
        .prepared()
        .optimized_plan()
        .display_indent()
        .to_string();
    assert!(observed.contains("Aggregate"));
    assert!(observed.contains("pse_row_key"));
    // Both declared package uniqueness and dependency closure execute, including
    // the empty dependency list's valid zero-membership case.
    assert_eq!(report.check_count(), 2);
    let valid = packages(&registry, 1);
    assert!(
        run_invariants(
            &valid,
            &session,
            &registry,
            InvariantScope::Candidate,
            &CancellationToken::default()
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn native_requirement_queries_retain_typed_violations_and_cancellation() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    for (count, valid) in [(1, true), (2, false)] {
        let rows = packages(&registry, count);
        let execution = session(&registry, &rows);
        let report = run_invariants(
            &rows,
            &execution,
            &registry,
            InvariantScope::Candidate,
            &CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(report.error_count() == 0, valid);
        if !valid {
            assert!(!report.findings().is_empty());
        }
        let token = CancellationToken::new();
        token.cancel();
        assert!(
            run_invariants(
                &rows,
                &execution,
                &registry,
                InvariantScope::Candidate,
                &token
            )
            .await
            .is_err()
        );
    }
}

#[tokio::test]
async fn every_registered_invariant_compiles_against_its_exact_candidate_contract() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let rows = registry
        .relations()
        .iter()
        .map(|spec| {
            (
                spec.key,
                RecordBatch::new_empty(Arc::new(
                    pse_schema::arrow::relation_schema(&registry, spec).unwrap(),
                )),
            )
        })
        .collect();
    let session = session(&registry, &rows);
    for invariant in registry.invariants() {
        let inputs = invariant
            .inputs
            .iter()
            .map(|name| registry.relation(name).unwrap().key)
            .collect::<Vec<_>>();
        session
            .bind_declared_query(&invariant.query, &inputs, &CancellationToken::default())
            .await
            .unwrap_or_else(|error| panic!("{}: {error}", invariant.qualified_name()));
    }
}

#[tokio::test]
async fn complete_minimal_model_executes_all_applicable_invariants_with_explicit_empty_bindings() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let mut rows: BTreeMap<_, _> = registry
        .relations()
        .iter()
        .filter(|spec| spec.snapshot_class == pse_schema::model::SnapshotClass::Model)
        .map(|spec| {
            (
                spec.key,
                RecordBatch::new_empty(Arc::new(
                    pse_schema::arrow::relation_schema(&registry, spec).unwrap(),
                )),
            )
        })
        .collect();
    rows.extend(packages(&registry, 1));
    let session = session(&registry, &rows);
    let report = run_invariants(
        &rows,
        &session,
        &registry,
        InvariantScope::Model,
        &CancellationToken::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.error_count(), 0);
    assert!(report.findings().is_empty());
    assert_eq!(
        report.check_count(),
        registry
            .invariants()
            .iter()
            .filter(
                |inv| registry.relation(&inv.relation).unwrap().snapshot_class
                    == pse_schema::model::SnapshotClass::Model
            )
            .count()
            + registry
                .relations()
                .iter()
                .filter(|spec| spec.snapshot_class == pse_schema::model::SnapshotClass::Model)
                .map(|spec| spec.checks.len())
                .sum::<usize>()
    );
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one integrity program exercises three independent violations"
)]
async fn native_integrity_program_reports_duplicates_references_and_nested_ordinal_bounds() {
    use pse_schema::model::{
        Authority, ExtensionUse, FieldContract, Namespace, RelationDecl, SnapshotClass,
    };
    let mut builder = pse_schema::RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    let relation = |name, extra: Vec<FieldContract>| {
        let key = if name == "integrity_source" {
            "__pse_count"
        } else {
            "id"
        };
        let mut columns = vec![FieldContract::key(
            key,
            FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
            "Fixture key.",
        )];
        columns.extend(extra);
        RelationDecl::new(
            Namespace::Authored,
            name,
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Integrity fixture.",
        )
        .pk(&[key])
        .columns(columns)
    };
    builder.declare_relation(relation("integrity_target", vec![]));
    builder.declare_relation(relation(
        "integrity_source",
        vec![
            FieldContract::reference(
                "target",
                FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
                "Actual target key.",
            )
            .with_fk("authored.integrity_target", "id")
            .optional(),
            FieldContract::payload(
                "__pse_integrity_value",
                FieldContract::structure(vec![
                    FieldContract::list(FieldContract::extended(ExtensionUse::OrdinalRef {
                        target: "authored.integrity_target",
                    }))
                    .with_name("items")
                    .with_nullable(true),
                ]),
                "Visible nested ordinal values.",
            )
            .optional(),
        ],
    ));
    let registry = Arc::new(builder.build().unwrap());
    let target = registry.relation("authored.integrity_target").unwrap();
    let source = registry.relation("authored.integrity_source").unwrap();
    let invalid = vec![
        serde_json::json!(["u64", 20]),
        serde_json::json!(["u64", 2]),
        serde_json::json!([
            "struct",
            vec![serde_json::json!([
                "list",
                vec![serde_json::json!(["i64", 2])]
            ])]
        ]),
    ];
    let rows = BTreeMap::from([
        (
            target.key,
            pse_relations::testing::batch_from_literals(
                &registry,
                target,
                &[
                    vec![serde_json::json!(["u64", 1])],
                    vec![serde_json::json!(["u64", 1])],
                ],
            )
            .unwrap(),
        ),
        (
            source.key,
            pse_relations::testing::batch_from_literals(
                &registry,
                source,
                &[
                    vec![
                        serde_json::json!(["u64", 10]),
                        serde_json::json!(["u64", 1]),
                        serde_json::json!([
                            "struct",
                            vec![serde_json::json!([
                                "list",
                                vec![serde_json::json!(["i64", 0]), serde_json::json!(["i64", 1])]
                            ])]
                        ]),
                    ],
                    invalid.clone(),
                    invalid,
                    vec![
                        serde_json::json!(["u64", 30]),
                        serde_json::json!(["null", null]),
                        serde_json::json!(["null", null]),
                    ],
                    vec![
                        serde_json::json!(["u64", 40]),
                        serde_json::json!(["null", null]),
                        serde_json::json!(["struct", vec![serde_json::json!(["null", null])]]),
                    ],
                ],
            )
            .unwrap(),
        ),
    ]);
    let bound_session = session(&registry, &rows);
    let report = run_invariants(
        &rows,
        &bound_session,
        &registry,
        InvariantScope::Candidate,
        &CancellationToken::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.check_count(), 4);
    assert_eq!(report.error_count(), 4);
    let actual = report
        .findings()
        .iter()
        .flat_map(|batch| {
            let view =
                pse_relations::generated::runtime::diagnostics_findings::View::from_checked(batch)
                    .unwrap();
            (0..batch.batch().num_rows())
                .map(|row| {
                    (
                        view.check_id_column().value(row).to_vec(),
                        view.row(row)
                            .unwrap()
                            .evidence
                            .row
                            .unwrap()
                            .row_key
                            .as_bytes()
                            .to_vec(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(actual.len(), 4);
    for invariant in registry
        .invariants()
        .iter()
        .filter(|inv| inv.relation.starts_with("authored.integrity_"))
    {
        let value = if invariant.relation == "authored.integrity_target" {
            1
        } else {
            20
        };
        let key = if invariant.relation == "authored.integrity_target" {
            "id"
        } else {
            "__pse_count"
        };
        let field =
            FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64).with_name(key);
        let token = row_key::values(
            &registry,
            registry.relation(&invariant.relation).unwrap().id,
            &[&field],
            &[serde_json::json!(["u64", value])],
        )
        .await;
        assert!(actual.contains(&(invariant.id.as_bytes().to_vec(), token.as_bytes().to_vec())));
    }
    let mut incomplete = rows.clone();
    incomplete.remove(&target.key);
    let incomplete_session = session(&registry, &incomplete);
    assert!(
        run_invariants(
            &incomplete,
            &incomplete_session,
            &registry,
            InvariantScope::Candidate,
            &CancellationToken::default()
        )
        .await
        .is_err()
    );
}
