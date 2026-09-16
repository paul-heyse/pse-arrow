// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Scalar mathematical facts feed the same native execution program.
#![allow(
    clippy::unwrap_used,
    reason = "independent mathematical fixture and assertions"
)]
use datafusion::{
    arrow::{
        array::{Float64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    },
    common::Column,
    execution::session_state::SessionStateBuilder,
    logical_expr::col,
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_mathir::{
    ExprGraph, NodeId, Opcode, Payload, QuantityTypeId, WeightNormalization, WeightedPair,
};
use pse_numerics::{NumericsError, scalar_math::ScalarMath};
use pse_quantity::{
    DimensionVector, IndexSet, QuantityAdditionKind, QuantityKind, QuantityKindId,
    QuantityRegistry, QuantityRegistryBuilder, QuantityType, QuantityTypeKey, ScaleKind, Unit,
    UnitId,
};
use std::{collections::BTreeMap, sync::Arc};

fn registry() -> QuantityRegistry {
    let id = SemanticId::NIL;
    let mut builder = QuantityRegistryBuilder::new();
    builder
        .unit(Unit {
            id: UnitId::from_id(id),
            symbol: "1".into(),
            dimension: DimensionVector::DIMENSIONLESS,
            scale_to_canonical: 1.0,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state: None,
        })
        .kind(QuantityKind {
            id: QuantityKindId::from_id(id),
            dimension: DimensionVector::DIMENSIONLESS,
            extensive: false,
            addition_kind: QuantityAdditionKind::Additive,
        })
        .quantity_type(QuantityType {
            id: QuantityTypeId::from_id(id),
            key: QuantityTypeKey {
                kind: QuantityKindId::from_id(id),
                basis: None,
                reference_state: None,
                scale_kind: ScaleKind::Point,
                shape: vec![],
                subject_kind: None,
            },
            canonical_unit: UnitId::from_id(id),
            nominal_magnitude: None,
        });
    builder.build().unwrap()
}
fn canonical(mut graph: ExprGraph, roots: &[NodeId]) -> pse_mathir::CanonicalGraph {
    let ids = graph.iter().map(|(id, _)| id).collect::<Vec<_>>();
    for id in &ids {
        graph
            .set_quantity_type(*id, QuantityTypeId::from_id(SemanticId::NIL))
            .unwrap();
    }
    pse_mathir::number_typed_graph(
        &graph,
        roots,
        &ids.into_iter().map(|id| (id, IndexSet::new())).collect(),
        &registry(),
    )
    .unwrap()
}
fn schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![Field::new("x", DataType::Float64, false)]))
}
fn number(batch: &RecordBatch, column: usize) -> f64 {
    batch
        .column(column)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap()
        .value(0)
}

#[test]
fn ordered_weighted_math_executes_after_the_mathematical_graph_is_released() {
    let symbol = SemanticId::from_bytes([1; 16]);
    let mut graph = ExprGraph::new();
    let x = graph.symbol(symbol).unwrap();
    let two = graph.int_const(2).unwrap();
    let three = graph.int_const(3).unwrap();
    let square = graph.pow(x, two).unwrap();
    let root = graph
        .insert(
            Opcode::WeightedMean,
            Payload::WeightedMean {
                pairs: vec![
                    WeightedPair {
                        weight: two,
                        value: square,
                    },
                    WeightedPair {
                        weight: three,
                        value: x,
                    },
                ],
                normalization: WeightNormalization::DivideBySum,
                unit_sum_invariant: None,
            },
            &[],
            None,
        )
        .unwrap();
    let graph = canonical(graph, &[root]);
    let symbols = BTreeMap::from([(symbol, col("x"))]);
    let budget = FixedBudget::new(32 << 20);
    let program = ScalarMath {
        graph: &graph,
        roots: graph.roots(),
        symbols: &symbols,
    }
    .compile(
        &[Column::from_name("x")],
        schema(),
        &SessionStateBuilder::new_with_default_features().build(),
        budget.clone(),
        CancellationToken::new(),
    )
    .unwrap();
    drop((graph, symbols));
    let output = program
        .evaluate(
            &RecordBatch::try_new(schema(), vec![Arc::new(Float64Array::from(vec![4.0]))]).unwrap(),
        )
        .unwrap();
    assert!((number(&output, 0) - 8.8).abs() < 1e-12);
    assert!((number(&output, 1) - 3.8).abs() < 1e-12);
    drop((program, output));
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn missing_case_bindings_and_inexact_integer_conversion_are_explicit() {
    let mut graph = ExprGraph::new();
    let root = graph.symbol(SemanticId::NIL).unwrap();
    let graph = canonical(graph, &[root]);
    let symbols = BTreeMap::new();
    let state = SessionStateBuilder::new_with_default_features().build();
    assert!(matches!(
        ScalarMath {
            graph: &graph,
            roots: graph.roots(),
            symbols: &symbols
        }
        .compile(
            &[],
            schema(),
            &state,
            FixedBudget::new(1 << 20),
            CancellationToken::new()
        ),
        Err(NumericsError::Input { .. })
    ));
    let mut graph = ExprGraph::new();
    let root = graph.int_const((1i64 << 53) + 1).unwrap();
    let graph = canonical(graph, &[root]);
    assert!(matches!(
        ScalarMath {
            graph: &graph,
            roots: graph.roots(),
            symbols: &symbols
        }
        .compile(
            &[],
            schema(),
            &state,
            FixedBudget::new(1 << 20),
            CancellationToken::new()
        ),
        Err(NumericsError::Unsupported { .. })
    ));
}
