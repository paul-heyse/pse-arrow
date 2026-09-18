// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::float_cmp,
    clippy::panic,
    clippy::unwrap_used,
    reason = "test fixture construction and exact independent value assertions"
)]
//! Actual indexed P3–P8 conservation, physical coefficient provenance and refusal cases.
#[path = "support/conservation_source.rs"]
mod conservation_source;
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;

use conservation_source::{Balance, Options, documents, id};
use native_pipeline::Values;
use pse_relations::generated::{
    compiled::law_participation::{self, CompiledLawParticipationFieldDecisionSelected},
    enums::ContributionSign,
};
use pse_schema::{Registry, model::Cell};
use std::collections::{BTreeMap, BTreeSet};

fn math_nodes(
    registry: &Registry,
    snapshot: &Values,
) -> Vec<pse_relations::generated::inferred::math_expr_nodes::Row> {
    let batch = source(snapshot, "inferred", "math_expr_nodes")
        .unwrap()
        .batch();
    pse_relations::generated::inferred::math_expr_nodes::View::try_from_batch_with_registry(
        registry, batch,
    )
    .unwrap()
    .rows()
    .unwrap()
}

fn rows(registry: &Registry, snapshot: &Values, name: &str) -> Vec<Vec<Cell>> {
    let spec = registry.relation(name).unwrap();
    let batch = source(snapshot, spec.key.namespace.as_str(), spec.key.name)
        .unwrap()
        .batch();
    pse_relations::cells::cells_from_batch(registry, spec, batch).unwrap()
}
fn source<'a>(
    values: &'a Values,
    namespace: &str,
    name: &str,
) -> Option<&'a pse_relations::columnar::FieldCheckedBatch> {
    native_pipeline::relation(values, namespace, name)
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
async fn run(options: &Options) -> (native_pipeline::Fixture, Values) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("pse_compiler=info,pse_rules=debug")
        .with_ansi(false)
        .try_init();
    let fixture = native_pipeline::Fixture::new();
    let source = documents(&fixture.registry, options);
    let model = fixture.source(source);
    let result = fixture
        .evaluate(model, "P8")
        .await
        .unwrap_or_else(|error| panic!("actual P3–P8 conservation pipeline: {error}"));
    let stage = result;
    (fixture, stage)
}
fn indexed_contract(
    registry: &Registry,
    stage: &Values,
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
    let partition = law_participation::View::from_checked(
        source(stage, "compiled", "law_participation").unwrap(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(
        partition.len(),
        2,
        "every actual contribution classified exactly once"
    );
    assert_eq!(
        partition
            .iter()
            .map(|row| match row.decision.selected().unwrap() {
                CompiledLawParticipationFieldDecisionSelected::Included(value) => value.sign,
                CompiledLawParticipationFieldDecisionSelected::Excluded(value) => {
                    panic!("expected an included conservation contribution, got {value:?}")
                }
            })
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([ContributionSign::Negative, ContributionSign::Positive]),
        "both actual contributions are included, with opposite conservation signs"
    );
    assert_eq!(
        math_nodes(registry, stage)
            .iter()
            .filter(|row| row.payload.reduction.is_some())
            .count(),
        reductions
    );
}

async fn conservation_contract(balance: Balance, kinds: &[&'static str], reductions: usize) {
    let (fixture, stage) = run(&Options::new(balance)).await;
    indexed_contract(&fixture.registry, &stage, kinds, reductions);
    assert!(
        rows(
            &fixture.registry,
            &stage,
            "compiled.element_projection_coefficients"
        )
        .is_empty()
    );
}
#[tokio::test]
async fn component_total_reduces_only_phase() {
    conservation_contract(Balance::ComponentTotal, &["species"], 2).await;
}
#[tokio::test]
async fn component_phase_preserves_both_axes() {
    conservation_contract(Balance::ComponentPhase, &["phase", "species"], 0).await;
}
#[tokio::test]
async fn energy_conservation_uses_declared_physical_contracts() {
    conservation_contract(Balance::Energy, &[], 2).await;
}
#[tokio::test]
async fn pressure_equality_uses_declared_physical_contracts() {
    conservation_contract(Balance::Pressure, &[], 0).await;
}
fn coefficient_values(registry: &Registry, stage: &Values) -> Vec<f64> {
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
    let constants = math_nodes(registry, stage);
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
                .find(|row| &Cell::I64(row.node_id) == node)
                .unwrap();
            let value = field(
                registry,
                "compiled.element_projection_coefficients",
                coefficient,
                "value",
            );
            assert_eq!(
                &Cell::F64(constant.payload.float.as_ref().unwrap().value),
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
async fn element_molar_projection(count: f64) {
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
    let reopened = fixture.roundtrip(&stage).await;
    assert_eq!(
        coefficient_values(&fixture.registry, &reopened),
        values,
        "actual source replay preserves complete admitted values"
    );
}
#[tokio::test]
async fn element_molar_projection_retains_element_axis() {
    element_molar_projection(2.0).await;
}
#[tokio::test]
async fn element_molar_projection_uses_changed_composition() {
    element_molar_projection(3.0).await;
}
#[tokio::test]
async fn element_mass_projection_uses_actual_kg_per_mole_weights() {
    let (fixture, stage) = run(&Options::new(Balance::ElementMass)).await;
    indexed_contract(&fixture.registry, &stage, &["element"], 4);
    let values = coefficient_values(&fixture.registry, &stage);
    assert_eq!(values.iter().filter(|value| **value == 0.0).count(), 4);
    let expected: BTreeMap<_, f64> =
        BTreeMap::from([(id(100), 2.0 / 0.002_016), (id(101), 2.0 / 0.031_998)]);
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
    let fixture = native_pipeline::Fixture::new();
    let mut options = Options::new(Balance::ElementMass);
    options.molecular_weight = false;
    let source = documents(&fixture.registry, &options);
    let model = fixture.source(source);
    let error = fixture.evaluate(model, "P8").await.unwrap_err();
    assert!(error.to_string().contains("molecular weight"), "{error}");
}
#[tokio::test]
async fn an_absent_explicit_balance_binding_is_not_inferred_from_a_template_name() {
    let fixture = native_pipeline::Fixture::new();
    let mut options = Options::new(Balance::ComponentTotal);
    options.unsupported = true;
    let source = documents(&fixture.registry, &options);
    let model = fixture.source(source);
    assert!(
        fixture.evaluate(model, "P8").await.is_err(),
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
async fn fixed_species_selects_actual_phase_members() {
    let mut options = Options::new(Balance::ComponentPhase);
    options.fixed_subject = true;
    let (fixture, stage) = run(&options).await;
    indexed_contract(&fixture.registry, &stage, &["phase"], 0);
    assert_eq!(
        rows(&fixture.registry, &stage, "compiled.group_projections").len(),
        2
    );
}

#[tokio::test]
async fn scalar_energy_broadcast_is_explicit() {
    let (fixture, stage) = run(&Options::new(Balance::EnergyBroadcast)).await;
    indexed_contract(&fixture.registry, &stage, &["phase"], 0);
    let nodes = math_nodes(&fixture.registry, &stage)
        .into_iter()
        .map(|row| (row.node_id, row))
        .collect::<BTreeMap<_, _>>();
    let equations = pse_relations::generated::inferred::math_indexed_equations::View::try_from_batch_with_registry(
        &fixture.registry, source(&stage, "inferred", "math_indexed_equations").unwrap().batch(),
    ).unwrap().rows().unwrap();
    let equation = &equations[0];
    let count_broadcasts = |root| {
        let mut pending = vec![root];
        let mut reachable = BTreeSet::new();
        while let Some(node) = pending.pop() {
            if reachable.insert(node) {
                pending.extend(&nodes[&node].children);
            }
        }
        reachable
            .iter()
            .filter(|node| nodes[node].payload.broadcast.is_some())
            .count()
    };
    assert_eq!(
        count_broadcasts(equation.body_node_id),
        2,
        "one explicit scalar-to-Phase broadcast for each signed source"
    );
    let bound = equation.constraint.single.as_ref().unwrap().node_id;
    assert_eq!(
        count_broadcasts(bound),
        1,
        "the equality's typed zero has the same Phase shape"
    );
    let bound = &nodes[&bound];
    assert!(bound.payload.broadcast.is_some());
    let zero = &nodes[&bound.children[0]];
    assert_eq!(zero.payload.float.as_ref().unwrap().value, 0.0);
}
