// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P2 observes duplicate candidate rows and preserves exact diagnostic/provenance keys.
#[path = "support/native_check_execution.rs"]
mod native_check_execution;
#[path = "support/row_key.rs"]
mod row_key;
use datafusion::arrow::array::{Array, RecordBatch};
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SnapshotSession, ThreadBudget, build_candidate_session,
    native_engine_profile,
};
use pse_ids::{CancellationToken, ContentHash, FixedBudget, SemanticId};
use pse_rules::invariants::{InvariantScope, run_invariants};
use pse_schema::{
    Registry,
    model::{Cell, RelationKey},
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

#[expect(clippy::unwrap_used, reason = "fixed test fixture admission")]
fn budget() -> ThreadBudget {
    ThreadBudget {
        pool_threads: NonZeroUsize::new(1).unwrap(),
        target_partitions: NonZeroUsize::new(1).unwrap(),
    }
}
#[expect(clippy::unwrap_used, reason = "fixed test fixture admission")]
fn session(registry: &Arc<Registry>, rows: &BTreeMap<RelationKey, RecordBatch>) -> SnapshotSession {
    build_candidate_session(
        rows.clone(),
        Arc::clone(registry),
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        budget(),
        native_engine_profile(),
    )
    .unwrap()
}
#[expect(clippy::unwrap_used, reason = "fixed test fixture admission")]
fn packages(registry: &Registry, count: usize) -> BTreeMap<RelationKey, RecordBatch> {
    let spec = registry.relation("authored.packages").unwrap();
    let row = vec![
        Cell::Id(SemanticId::from_bytes([7; 16])),
        Cell::text("fixture"),
        Cell::text("1.0.0"),
        Cell::Enum("model"),
        Cell::Enum("explicit"),
        Cell::List(vec![]),
        Cell::Hash(ContentHash::from_bytes([0; 32])),
        Cell::text("Fixture"),
    ];
    BTreeMap::from([(
        spec.key,
        pse_relations::cells::batch_from_cells(registry, spec, &vec![row; count]).unwrap(),
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
    let factory = pse_catalog::session::SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        budget(),
        native_engine_profile(),
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

fn physical_subject(kind: &'static str, member: &Cell, phase: &Cell, law: bool) -> Cell {
    let mut fields = vec![Cell::Enum(kind)];
    for arm in [
        "total",
        "energy",
        "momentum",
        "species",
        "element",
        "phase_species",
    ] {
        fields.push(if arm == kind {
            Cell::Struct(match arm {
                "total" | "energy" | "momentum" => vec![phase.clone()],
                "species" | "element" if !law => vec![member.clone()],
                _ => vec![member.clone(), phase.clone()],
            })
        } else {
            Cell::Null
        });
    }
    Cell::Struct(fields)
}

#[tokio::test]
async fn subject_alternatives_admit_indexed_physics_and_reject_overlapping_coordinates() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let spec = registry
        .relation("authored.template_contribution_contracts")
        .unwrap();
    // Typed alternatives make missing/overlapping arm payloads structural errors.
    // The remaining relational rule forbids reusing one axis for two coordinates.
    let axis = |position| {
        Cell::Struct(vec![
            Cell::Enum("axis"),
            Cell::Null,
            Cell::Struct(vec![Cell::I64(position)]),
        ])
    };
    let fixed = || {
        Cell::Struct(vec![
            Cell::Enum("fixed"),
            Cell::Struct(vec![Cell::Id(SemanticId::NIL)]),
            Cell::Null,
        ])
    };
    let cases = [
        physical_subject("energy", &Cell::Null, &Cell::Null, false),
        physical_subject("energy", &Cell::Null, &axis(0), false),
        physical_subject("energy", &Cell::Null, &fixed(), false),
        physical_subject("species", &axis(0), &Cell::Null, false),
        physical_subject("species", &fixed(), &Cell::Null, false),
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
                Cell::Id(SemanticId::from_bytes(
                    [u8::try_from(index + 1).unwrap(); 16],
                )),
                Cell::List(vec![]),
                Cell::Id(SemanticId::NIL),
                subject,
                Cell::Null,
            ]
        })
        .collect::<Vec<_>>();
    let rows = BTreeMap::from([(
        spec.key,
        pse_relations::cells::batch_from_cells(&registry, spec, &values).unwrap(),
    )]);
    let session = session(&registry, &rows);
    let plan = native_check_execution::violations(&session, spec, "distinct_subject_axes");
    let cancel = CancellationToken::default();
    let result = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let actual = result
        .batches()
        .iter()
        .flat_map(|batch| {
            let ids = batch
                .column(0)
                .as_any()
                .downcast_ref::<datafusion::arrow::array::FixedSizeBinaryArray>()
                .unwrap();
            (0..ids.len())
                .map(|index| ids.value(index).to_vec())
                .collect::<Vec<_>>()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn species_law_accepts_a_fixed_phase_slice_but_rejects_a_free_phase_axis() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let spec = registry.relation("compiled.law_applications").unwrap();
    let values = [None, Some(1)]
        .into_iter()
        .enumerate()
        .map(|(index, phase_axis)| {
            spec.columns
                .iter()
                .map(|column| match column.name() {
                    "application_id" => Cell::Id(SemanticId::from_bytes(
                        [u8::try_from(index + 1).unwrap(); 16],
                    )),
                    "subject" => physical_subject(
                        "species",
                        &Cell::Struct(vec![
                            Cell::Enum("axis"),
                            Cell::Null,
                            Cell::Struct(vec![Cell::I64(0)]),
                        ]),
                        &phase_axis.map_or_else(
                            || {
                                Cell::Struct(vec![
                                    Cell::Enum("fixed"),
                                    Cell::Struct(vec![Cell::Id(SemanticId::NIL)]),
                                    Cell::Null,
                                ])
                            },
                            |position| {
                                Cell::Struct(vec![
                                    Cell::Enum("axis"),
                                    Cell::Null,
                                    Cell::Struct(vec![Cell::I64(position)]),
                                ])
                            },
                        ),
                        true,
                    ),
                    "basis_id" => Cell::Null,
                    "law_family" | "source_family" => Cell::Enum("material"),
                    "subject_projection" => Cell::Enum("identity"),
                    "expansion" => Cell::Enum("conservation"),
                    _ => Cell::Id(SemanticId::NIL),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let rows = BTreeMap::from([(
        spec.key,
        pse_relations::cells::batch_from_cells(&registry, spec, &values).unwrap(),
    )]);
    let session = session(&registry, &rows);
    let plan = native_check_execution::violations(&session, spec, "species_phase_slice");
    let cancel = CancellationToken::default();
    let result = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let invalid = result
        .batches()
        .iter()
        .flat_map(|batch| {
            let ids = batch
                .column(0)
                .as_any()
                .downcast_ref::<datafusion::arrow::array::FixedSizeBinaryArray>()
                .unwrap();
            (0..ids.len())
                .map(|row| ids.value(row).to_vec())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(invalid, vec![vec![2; 16]]);
}

#[tokio::test]
async fn p2_keeps_candidate_duplicates_and_never_invents_execution_or_snapshot_identity() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let rows = packages(&registry, 2);
    let session = session(&registry, &rows);
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
            &[Cell::Id(SemanticId::from_bytes([7; 16]))]
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
        Cell::U64(20),
        Cell::U64(2),
        Cell::Struct(vec![Cell::List(vec![Cell::I64(2)])]),
    ];
    let rows = BTreeMap::from([
        (
            target.key,
            pse_relations::cells::batch_from_cells(
                &registry,
                target,
                &[vec![Cell::U64(1)], vec![Cell::U64(1)]],
            )
            .unwrap(),
        ),
        (
            source.key,
            pse_relations::cells::batch_from_cells(
                &registry,
                source,
                &[
                    vec![
                        Cell::U64(10),
                        Cell::U64(1),
                        Cell::Struct(vec![Cell::List(vec![Cell::I64(0), Cell::I64(1)])]),
                    ],
                    invalid.clone(),
                    invalid,
                    vec![Cell::U64(30), Cell::Null, Cell::Null],
                    vec![Cell::U64(40), Cell::Null, Cell::Struct(vec![Cell::Null])],
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
            &[Cell::U64(value)],
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
