// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P2 observes duplicate candidate rows and preserves exact diagnostic/provenance keys.
use datafusion::arrow::array::RecordBatch;
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SnapshotSession, ThreadBudget, build_candidate_session,
    phase0_reference_profile,
};
use pse_catalog::store::membership::SemanticValidator;
use pse_ids::{CancellationToken, ContentHash, FixedBudget, SemanticId};
use pse_rules::{
    invariants::{InvariantScope, RunEvidence, run_invariants, run_invariants_with_evidence},
    plan::{PortBinding, compile},
    validator::InvariantValidator,
};
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
        phase0_reference_profile(),
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
async fn p2_keeps_candidate_duplicates_and_never_invents_execution_or_snapshot_identity() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let rows = packages(&registry, 2);
    let session = session(&registry, &rows);
    let report = run_invariants(
        &rows,
        &session,
        &registry,
        InvariantScope::Candidate,
        None,
        &CancellationToken::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.error_count, 1);
    let finding = pse_relations::cells::decode_columns(&registry, &report.findings[0]).unwrap();
    assert_eq!(finding[0][1], Cell::Null);
    let provenance =
        pse_relations::cells::decode_columns(&registry, &report.derivations[0]).unwrap();
    assert_eq!(provenance[0][6], Cell::Null);
    assert_eq!(provenance[0][7], Cell::Null);
    let Cell::Text(key) = &provenance[0][2] else {
        panic!("typed key")
    };
    let rule = registry
        .rules()
        .iter()
        .find(|rule| rule.name == "unique:pk:authored.packages")
        .unwrap();
    assert_eq!(
        pse_rules::derivations::decode_key(rule, &registry, key).unwrap(),
        vec![Cell::Id(SemanticId::from_bytes([7; 16]))]
    );
    let real = ContentHash::from_bytes([9; 32]);
    let with_evidence = run_invariants_with_evidence(
        &rows,
        &session,
        &registry,
        InvariantScope::Candidate,
        None,
        Some(RunEvidence { fingerprint: real }),
        &CancellationToken::default(),
    )
    .await
    .unwrap();
    assert_eq!(
        pse_relations::cells::decode_columns(&registry, &with_evidence.derivations[0]).unwrap()[0]
            [7],
        Cell::Hash(real)
    );
    assert_eq!(with_evidence.error_count, 1);
    let valid = packages(&registry, 1);
    assert!(
        run_invariants(
            &valid,
            &session,
            &registry,
            InvariantScope::Candidate,
            None,
            &CancellationToken::default()
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn catalog_validator_executes_candidate_rules_and_retains_typed_violations() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let validator = InvariantValidator::new(
        Arc::clone(&registry),
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        budget(),
        phase0_reference_profile(),
    );
    validator
        .validate(
            &registry,
            &packages(&registry, 1),
            &CancellationToken::default(),
        )
        .await
        .unwrap();
    let error = validator
        .validate(
            &registry,
            &packages(&registry, 2),
            &CancellationToken::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(
        miette::Diagnostic::code(&error).unwrap().to_string(),
        "validation::invariant"
    );
    let token = CancellationToken::default();
    token.cancel();
    assert!(
        validator
            .validate(&registry, &packages(&registry, 1), &token)
            .await
            .is_err()
    );
}

#[test]
fn every_registered_invariant_compiles_against_its_exact_candidate_contract() {
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
        let rule = registry.rule(&invariant.rule).unwrap();
        let mut binding = PortBinding::default();
        for (relation, port, _) in rule.plan.dependencies() {
            binding
                .ports
                .insert(port.to_owned(), registry.relation(relation).unwrap().key);
        }
        compile(rule, &binding, &session, &registry)
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
        None,
        &CancellationToken::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.error_count, 0);
    assert!(report.findings.is_empty());
    assert_eq!(
        report.explain.len(),
        registry
            .invariants()
            .iter()
            .filter(
                |inv| registry.relation(&inv.relation).unwrap().snapshot_class
                    == pse_schema::model::SnapshotClass::Model
            )
            .count()
    );
}
