// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual indexed P3–P8 conservation, physical coefficient provenance and refusal cases.
#[path = "support/conservation_source.rs"]
mod conservation_source;
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
use conservation_source::{Balance, Options, documents, id};
use pse_catalog::Snapshot;
use pse_schema::{Registry, model::Cell};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

fn rows(registry: &Registry, snapshot: &Snapshot, name: &str) -> Vec<Vec<Cell>> {
    let spec = registry.relation(name).unwrap();
    let batch = source(snapshot, spec.key.namespace.as_str(), spec.key.name)
        .unwrap()
        .batch();
    pse_relations::cells::cells_from_batch(registry, spec, batch).unwrap()
}
fn source<'a>(
    snapshot: &'a Snapshot,
    namespace: &str,
    name: &str,
) -> Option<&'a Arc<pse_catalog::LoadedRelation>> {
    snapshot.relation(namespace, name).or_else(|| {
        snapshot
            .parents()
            .values()
            .find_map(|parent| source(parent, namespace, name))
    })
}
fn field<'a>(registry: &Registry, name: &str, row: &'a [Cell], column: &str) -> &'a Cell {
    &row[registry
        .relation(name)
        .unwrap()
        .columns
        .iter()
        .position(|field| field.name() == column)
        .unwrap()]
}
async fn run(options: &Options) -> (native_pipeline::Fixture, Arc<Snapshot>) {
    let mut fixture = native_pipeline::Fixture::new();
    let source = documents(&fixture.registry, options);
    let model = fixture.commit(source).await;
    let result = fixture
        .run(model, "P8")
        .await
        .expect("actual P3–P8 conservation pipeline");
    assert_eq!(result.stages.last().unwrap().pass, "P8");
    let stage = Arc::clone(&result.stages.last().unwrap().snapshot);
    (fixture, stage)
}
fn indexed_contract(
    registry: &Registry,
    stage: &Snapshot,
    kinds: &[&'static str],
    reductions: usize,
) {
    let equations = rows(registry, stage, "inferred.math_indexed_equations");
    assert_eq!(
        equations.len(),
        1,
        "one indexed law, never one scalar equation per tuple"
    );
    let indices = rows(registry, stage, "inferred.math_free_indices");
    let domains = rows(registry, stage, "normalized.domains");
    let actual = indices
        .iter()
        .map(|index| {
            let domain = field(registry, "inferred.math_free_indices", index, "domain_id");
            let row = domains
                .iter()
                .find(|row| field(registry, "normalized.domains", row, "domain_id") == domain)
                .unwrap();
            field(registry, "normalized.domains", row, "kind").literal_spec()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual,
        kinds
            .iter()
            .map(|kind| Cell::Enum(kind).literal_spec())
            .collect()
    );
    assert_eq!(indices.len(), kinds.len(), "all declared free axes survive");
    let partition = rows(registry, stage, "compiled.law_participation");
    assert_eq!(
        partition.len(),
        2,
        "every actual contribution classified exactly once"
    );
    assert!(partition.iter().all(|row| field(
        registry,
        "compiled.law_participation",
        row,
        "decision"
    ) == &Cell::Enum("included")));
    assert_eq!(
        partition
            .iter()
            .map(|row| field(registry, "compiled.law_participation", row, "sign").literal_spec())
            .collect::<BTreeSet<_>>(),
        [Cell::I64(-1), Cell::I64(1)]
            .iter()
            .map(Cell::literal_spec)
            .collect()
    );
    assert_eq!(
        rows(registry, stage, "inferred.math_reductions").len(),
        reductions
    );
}

#[tokio::test]
async fn component_total_reduces_only_phase_and_component_phase_preserves_both_axes() {
    for (balance, kinds, reductions) in [
        (Balance::ComponentTotal, vec!["species"], 2),
        (Balance::ComponentPhase, vec!["phase", "species"], 0),
    ] {
        let (fixture, stage) = run(&Options::new(balance)).await;
        indexed_contract(&fixture.registry, &stage, &kinds, reductions);
        assert!(
            rows(
                &fixture.registry,
                &stage,
                "compiled.element_projection_coefficients"
            )
            .is_empty()
        );
    }
}
#[tokio::test]
async fn energy_conservation_and_pressure_equality_use_declared_physical_contracts() {
    for (balance, reductions) in [(Balance::Energy, 2), (Balance::Pressure, 0)] {
        let (fixture, stage) = run(&Options::new(balance)).await;
        indexed_contract(&fixture.registry, &stage, &[], reductions);
        assert!(
            rows(
                &fixture.registry,
                &stage,
                "compiled.element_projection_coefficients"
            )
            .is_empty()
        );
    }
}
fn coefficient_values(registry: &Registry, stage: &Snapshot) -> Vec<f64> {
    let groups = rows(registry, stage, "compiled.element_projection_groups");
    assert_eq!(
        groups.len(),
        2,
        "each actual stream has its own source correspondence"
    );
    let coefficients = rows(registry, stage, "compiled.element_projection_coefficients");
    assert_eq!(
        coefficients.len(),
        8,
        "two streams times two elements times two species"
    );
    let constants = rows(registry, stage, "inferred.math_float_constants");
    let symbols = rows(registry, stage, "compiled.symbols");
    coefficients
        .iter()
        .map(|coefficient| {
            let node = field(
                registry,
                "compiled.element_projection_coefficients",
                coefficient,
                "node_id",
            );
            let constant = constants
                .iter()
                .find(|row| {
                    field(registry, "inferred.math_float_constants", row, "node_id") == node
                })
                .unwrap();
            let value = field(
                registry,
                "compiled.element_projection_coefficients",
                coefficient,
                "value",
            );
            assert_eq!(
                field(registry, "inferred.math_float_constants", constant, "value"),
                value,
                "actual immutable constant equals source-derived value"
            );
            let symbol = field(
                registry,
                "compiled.element_projection_coefficients",
                coefficient,
                "symbol_id",
            );
            let symbol = symbols
                .iter()
                .find(|row| field(registry, "compiled.symbols", row, "symbol_id") == symbol)
                .unwrap();
            assert_eq!(
                field(registry, "compiled.symbols", symbol, "role"),
                &Cell::Enum("expression")
            );
            assert_eq!(
                field(registry, "compiled.symbols", symbol, "default_initial"),
                &Cell::Null,
                "coefficient is never an initial guess"
            );
            let Cell::F64(value) = value else {
                panic!("typed real coefficient")
            };
            *value
        })
        .collect()
}
#[tokio::test]
async fn element_molar_projection_retains_element_axis_and_recomputes_changed_composition() {
    for count in [2.0, 3.0] {
        let mut options = Options::new(Balance::ElementMolar);
        options.hydrogen_count = count;
        options.molecular_weight = false;
        let (fixture, stage) = run(&options).await;
        indexed_contract(&fixture.registry, &stage, &["element"], 4);
        let values = coefficient_values(&fixture.registry, &stage);
        assert_eq!(
            values.iter().filter(|value| **value == 0.0).count(),
            4,
            "absent cross-species elements are explicit zero coefficients"
        );
        let coefficients = rows(
            &fixture.registry,
            &stage,
            "compiled.element_projection_coefficients",
        );
        for row in coefficients.iter().filter(|row| {
            field(
                &fixture.registry,
                "compiled.element_projection_coefficients",
                row,
                "species_id",
            ) == &Cell::Id(id(100))
        }) {
            let value = field(
                &fixture.registry,
                "compiled.element_projection_coefficients",
                row,
                "value",
            );
            assert!(value == &Cell::F64(0.0) || value == &Cell::F64(count));
        }
        assert_eq!(
            values.iter().filter(|value| **value == count).count(),
            if count == 2.0 { 4 } else { 2 }
        );
        let reopened = fixture
            .catalog
            .read_pinned_manifest(stage.manifest_ref(), &pse_ids::CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            coefficient_values(&fixture.registry, &reopened),
            values,
            "actual source replay preserves complete admitted values"
        );
    }
}
#[tokio::test]
async fn element_mass_projection_uses_actual_kg_per_mole_weights() {
    let (fixture, stage) = run(&Options::new(Balance::ElementMass)).await;
    indexed_contract(&fixture.registry, &stage, &["element"], 4);
    let values = coefficient_values(&fixture.registry, &stage);
    assert_eq!(values.iter().filter(|value| **value == 0.0).count(), 4);
    let expected: BTreeMap<_, f64> =
        BTreeMap::from([(id(100), 2.0 / 0.002016), (id(101), 2.0 / 0.031998)]);
    for row in rows(
        &fixture.registry,
        &stage,
        "compiled.element_projection_coefficients",
    ) {
        let Cell::Id(species) = field(
            &fixture.registry,
            "compiled.element_projection_coefficients",
            &row,
            "species_id",
        ) else {
            panic!("actual species")
        };
        let Cell::F64(value) = field(
            &fixture.registry,
            "compiled.element_projection_coefficients",
            &row,
            "value",
        ) else {
            panic!("real coefficient")
        };
        assert!(*value == 0.0 || value.to_bits() == expected[species].to_bits());
    }
}
#[tokio::test]
async fn mass_projection_missing_molecular_weight_refuses_actual_source() {
    let mut fixture = native_pipeline::Fixture::new();
    let mut options = Options::new(Balance::ElementMass);
    options.molecular_weight = false;
    let source = documents(&fixture.registry, &options);
    let model = fixture.commit(source).await;
    fixture
        .run(Arc::clone(&model), "P7")
        .await
        .expect("same sources fully realize before the unsupported physical conversion");
    let error = fixture.run(model, "P8").await.unwrap_err();
    assert!(
        format!("{error:?}").contains("molecular weight"),
        "{error:?}"
    );
}
#[tokio::test]
async fn an_absent_explicit_balance_binding_is_not_inferred_from_a_template_name() {
    let mut fixture = native_pipeline::Fixture::new();
    let mut options = Options::new(Balance::ComponentTotal);
    options.unsupported = true;
    let source = documents(&fixture.registry, &options);
    let model = fixture.commit(source).await;
    assert!(
        fixture.run(model, "P8").await.is_err(),
        "missing actual law binding cannot compile"
    );
}

#[tokio::test]
async fn fixed_phase_selects_actual_group_members_without_summing_the_other_phase() {
    let mut options = Options::new(Balance::ComponentTotal);
    options.fixed_phase = true;
    let (fixture, stage) = run(&options).await;
    indexed_contract(&fixture.registry, &stage, &["species"], 0);
    let projections = rows(&fixture.registry, &stage, "compiled.group_projections");
    assert_eq!(
        projections.len(),
        2,
        "both actual streams retain explicit source slices"
    );
    let members = rows(&fixture.registry, &stage, "compiled.symbol_group_members");
    for projection in projections {
        let group = field(
            &fixture.registry,
            "compiled.group_projections",
            &projection,
            "group_id",
        );
        let actual = members
            .iter()
            .filter(|row| {
                field(
                    &fixture.registry,
                    "compiled.symbol_group_members",
                    row,
                    "group_id",
                ) == group
            })
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 2, "one actual phase times both species");
        for member in actual {
            let Cell::List(tuple) = field(
                &fixture.registry,
                "compiled.symbol_group_members",
                member,
                "tuple",
            ) else {
                panic!("actual tuple")
            };
            assert_eq!(
                tuple.len(),
                1,
                "fixed phase was removed from the group product"
            );
        }
    }
}

#[tokio::test]
async fn fixed_species_selects_actual_phase_members_and_scalar_energy_broadcast_is_explicit() {
    let mut options = Options::new(Balance::ComponentPhase);
    options.fixed_subject = true;
    let (fixture, stage) = run(&options).await;
    indexed_contract(&fixture.registry, &stage, &["phase"], 0);
    assert_eq!(
        rows(&fixture.registry, &stage, "compiled.group_projections").len(),
        2
    );
    let (fixture, stage) = run(&Options::new(Balance::EnergyBroadcast)).await;
    indexed_contract(&fixture.registry, &stage, &["phase"], 0);
    assert_eq!(
        rows(&fixture.registry, &stage, "inferred.math_broadcasts").len(),
        2,
        "one explicit scalar-to-Phase broadcast for each signed source"
    );
}
