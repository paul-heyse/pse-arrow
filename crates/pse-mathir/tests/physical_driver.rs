// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical inference before typed identity, exercised through the actual P10 leaf entry.
use pse_ids::SemanticId;
use pse_mathir::canonicalize::{CanonicalizeInput, Policy, canonicalize};
use pse_mathir::infer::SymbolTypeSource;
use pse_mathir::{ExprGraph, MathIrError, NodeId, Opcode, Payload};
use pse_quantity::standard::{StandardInvariantChecker, ids, standard_registry};
use pse_quantity::{QuantityTypeId, UnitId};
use std::collections::BTreeMap;
#[derive(Default)]
struct Source {
    symbols: BTreeMap<SemanticId, QuantityTypeId>,
    units: BTreeMap<SemanticId, UnitId>,
}
impl SymbolTypeSource for Source {
    fn symbol_type(&self, symbol: SemanticId) -> Option<QuantityTypeId> {
        self.symbols.get(&symbol).copied()
    }
    fn symbol_unit(&self, symbol: SemanticId) -> Option<UnitId> {
        self.units.get(&symbol).copied()
    }
    fn invariant_checker(&self, _node: NodeId) -> &dyn pse_quantity::infer::InvariantChecker {
        &StandardInvariantChecker
    }
}
fn sid(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}
#[expect(
    clippy::unwrap_used,
    reason = "fixture helper asserts graph construction outside individual test bodies"
)]
fn symbol(graph: &mut ExprGraph, source: &mut Source, index: u8, quantity: &str) -> NodeId {
    source.symbols.insert(sid(index), ids::quantity(quantity));
    graph.symbol(sid(index)).unwrap()
}
#[test]
fn actual_composed_quantity_results_and_selections_precede_hashes() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let flow = symbol(&mut graph, &mut source, 1, "molar_flow");
    let enthalpy = symbol(&mut graph, &mut source, 2, "molar_enthalpy.point");
    let fraction = symbol(&mut graph, &mut source, 3, "mole_fraction");
    let gas = symbol(&mut graph, &mut source, 4, "gas_constant");
    let temperature = symbol(&mut graph, &mut source, 5, "temperature.point");
    let pressure = symbol(&mut graph, &mut source, 6, "pressure.absolute");
    let activation = symbol(&mut graph, &mut source, 7, "activation_energy");
    let energy = graph.mul(flow, enthalpy).unwrap();
    let component = graph.mul(flow, fraction).unwrap();
    let rt = graph.mul(gas, temperature).unwrap();
    let density = graph.div(pressure, rt).unwrap();
    let ratio = graph.div(activation, rt).unwrap();
    let exponent = graph
        .insert(Opcode::Exp, Payload::None, &[ratio], None)
        .unwrap();
    let scale = graph
        .insert_typed(
            Opcode::Const,
            Payload::FloatConst {
                value: 1000.,
                unit: ids::unit("K"),
            },
            &[],
            ids::quantity("temperature_scale"),
            None,
        )
        .unwrap();
    let polynomial = graph.div(temperature, scale).unwrap();
    let roots = [energy, component, rt, density, exponent, polynomial];
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &roots, &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    for (root, name) in result.roots().iter().zip([
        "energy_flow",
        "component_flow",
        "molar_energy",
        "molar_density",
        "neutral",
        "neutral",
    ]) {
        assert_eq!(
            result.node(*root).unwrap().quantity_type,
            ids::quantity(name)
        );
    }
    assert_eq!(result.selections().len(), result.len());
    assert!(
        result
            .selections()
            .iter()
            .any(|row| row.operation == Some(ids::operation("flow_times_enthalpy")))
            || result
                .selections()
                .iter()
                .filter(|row| row.operation.is_some())
                .count()
                >= 6
    );
    let mut rows = pse_mathir::relations::VecSink::new();
    pse_mathir::relations::emit(&result, &mut rows).unwrap();
    let reloaded =
        pse_mathir::relations::load_canonical(&rows, result.roots(), &source, &registry).unwrap();
    assert_eq!(result.listing(), reloaded.listing());
    let again = canonicalize(
        CanonicalizeInput::new(&graph, &roots, &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    assert_eq!(result.listing(), again.listing());
}
#[test]
fn one_source_literal_resolves_by_occurrence_before_sharing() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let point = symbol(&mut graph, &mut source, 1, "temperature.point");
    let difference = symbol(&mut graph, &mut source, 2, "temperature.difference");
    let literal = graph.float_const(20., ids::unit("K")).unwrap();
    let residual = graph.sub(point, literal).unwrap();
    let shifted = graph.add(difference, literal).unwrap();
    let roots = [residual, shifted];
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &roots, &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    let literal_types: Vec<_> = result
        .iter()
        .filter_map(|(_, node)| {
            matches!(node.payload, Payload::FloatConst { value: 20., .. })
                .then_some(node.quantity_type)
        })
        .collect();
    assert_eq!(literal_types.len(), 2);
    assert!(literal_types.contains(&ids::quantity("temperature.point")));
    assert!(literal_types.contains(&ids::quantity("temperature.difference")));
    for root in result.roots() {
        assert_eq!(
            result.node(*root).unwrap().quantity_type,
            ids::quantity("temperature.difference")
        );
    }
}
#[test]
fn complete_type_claims_and_missing_compositions_are_rechecked() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let point = symbol(&mut graph, &mut source, 1, "temperature.point");
    let pressure = symbol(&mut graph, &mut source, 2, "pressure.absolute");
    let missing = graph.mul(point, pressure).unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[missing], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
    let claimed = graph
        .insert_typed(
            Opcode::Neg,
            Payload::None,
            &[point],
            ids::quantity("pressure.absolute"),
            None,
        )
        .unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[claimed], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
    let gauge = symbol(&mut graph, &mut source, 3, "pressure.gauge");
    let mixed = graph.sub(pressure, gauge).unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[mixed], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
}
#[test]
fn excluded_failures_remain_guarded_and_unconditional_uses_still_fail() {
    let registry = standard_registry().unwrap();
    let source = Source::default();
    let mut graph = ExprGraph::new();
    let zero = graph.int_const(0).unwrap();
    let one = graph.int_const(1).unwrap();
    let bad = graph.div(one, zero).unwrap();
    let log = graph
        .insert(Opcode::Log, Payload::None, &[zero], None)
        .unwrap();
    for failure in [bad, log] {
        let conditional = graph
            .insert(
                Opcode::Conditional,
                Payload::Conditional { guard: zero.into() },
                &[failure, one],
                None,
            )
            .unwrap();
        let result = canonicalize(
            CanonicalizeInput::new(&graph, &[conditional], &source, &registry),
            Policy::Strict,
        )
        .unwrap();
        assert!(
            result
                .iter()
                .any(|(_, node)| node.opcode == graph.node(failure).unwrap().opcode)
        );
        assert!(matches!(
            canonicalize(
                CanonicalizeInput::new(&graph, &[conditional, failure], &source, &registry),
                Policy::Strict
            ),
            Err(MathIrError::StaticDomain { .. })
        ));
    }
}
#[test]
fn unit_edges_check_actual_direction_and_point_difference_offsets() {
    let registry = standard_registry().unwrap();
    let source = Source::default();
    let mut graph = ExprGraph::new();
    let c = registry
        .unit_by_symbol("degC")
        .or_else(|| registry.unit_by_symbol("°C"))
        .unwrap();
    let point = graph
        .insert_typed(
            Opcode::Const,
            Payload::FloatConst {
                value: 20.,
                unit: c.id,
            },
            &[],
            ids::quantity("temperature.point"),
            None,
        )
        .unwrap();
    let difference = graph
        .insert_typed(
            Opcode::Const,
            Payload::FloatConst {
                value: 20.,
                unit: c.id,
            },
            &[],
            ids::quantity("temperature.difference"),
            None,
        )
        .unwrap();
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[point, difference], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    assert!(
        matches!(result.node(result.roots()[0]).unwrap().payload,Payload::FloatConst{value,..} if value.to_bits()==293.15f64.to_bits())
    );
    assert!(
        matches!(result.node(result.roots()[1]).unwrap().payload,Payload::FloatConst{value,..} if value.to_bits()==20.0f64.to_bits())
    );
    let wrong = graph
        .unit_convert(
            point,
            pse_quantity::UnitConvertSpec {
                from: ids::unit("K"),
                to: c.id,
                scale: 1.,
                offset: -273.15,
            },
        )
        .unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[wrong], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
}
#[test]
fn named_gauge_conversion_is_checked_and_recorded_exactly_once() {
    let registry = standard_registry().unwrap();
    let mut source = Source::default();
    let mut graph = ExprGraph::new();
    let gauge = symbol(&mut graph, &mut source, 1, "pressure.gauge");
    let spec = pse_quantity::UnitConvertSpec {
        from: ids::unit("Pa"),
        to: ids::unit("Pa"),
        scale: 1.,
        offset: 101_325.,
    };
    let edge = graph.unit_convert(gauge, spec).unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[edge], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
    let selections = [pse_mathir::relations::vec_sink::QuantitySelection {
        node: edge,
        operation: None,
        builtin: Some(pse_quantity::infer::BuiltInRule::UnitConvert),
        permutation: vec![],
        conversions: vec![(0, ids::conversion("gauge_to_absolute"))],
        deferred_static_check: false,
    }];
    let roots = [edge];
    let mut input = CanonicalizeInput::new(&graph, &roots, &source, &registry);
    input.selections = &selections;
    let result = canonicalize(input, Policy::Strict).unwrap();
    assert_eq!(
        result.node(result.roots()[0]).unwrap().quantity_type,
        ids::quantity("pressure.absolute")
    );
    assert_eq!(
        result
            .selections()
            .iter()
            .filter(|s| !s.conversions.is_empty())
            .count(),
        1
    );
}

#[test]
fn known_zero_divisor_is_rejected_even_with_a_symbolic_numerator() {
    let registry = standard_registry().unwrap();
    let mut source = Source::default();
    let mut graph = ExprGraph::new();
    let numerator = symbol(&mut graph, &mut source, 1, "neutral");
    let zero = graph.int_const(0).unwrap();
    let bad = graph.div(numerator, zero).unwrap();
    assert!(matches!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[bad], &source, &registry),
            Policy::Strict
        ),
        Err(MathIrError::StaticDomain { .. })
    ));
}

#[test]
fn physical_occurrence_traversal_handles_a_deep_graph_without_recursion() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let mut root = symbol(&mut graph, &mut source, 1, "neutral");
    for _ in 0..5000 {
        root = graph.neg(root).unwrap();
    }
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[root], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    assert_eq!(result.len(), 5001);
    assert_eq!(result.node(result.roots()[0]).unwrap().opcode, Opcode::Neg);
}

#[test]
fn weighted_mean_uses_ordered_actual_weight_facts_even_when_values_are_symbolic() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let temperature = symbol(&mut graph, &mut source, 1, "temperature.point");
    let one = graph.int_const(1).unwrap();
    let negative = graph.int_const(-1).unwrap();
    let mean = graph
        .insert(
            Opcode::WeightedMean,
            Payload::WeightedMean {
                pairs: vec![
                    pse_mathir::WeightedPair {
                        weight: one,
                        value: temperature,
                    },
                    pse_mathir::WeightedPair {
                        weight: negative,
                        value: temperature,
                    },
                ],
                normalization: pse_quantity::WeightNormalization::DivideBySum,
                unit_sum_invariant: None,
            },
            &[],
            None,
        )
        .unwrap();
    assert!(matches!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[mean], &source, &registry),
            Policy::Strict
        ),
        Err(MathIrError::StaticDomain { .. })
    ));
    let good = graph
        .insert(
            Opcode::WeightedMean,
            Payload::WeightedMean {
                pairs: vec![pse_mathir::WeightedPair {
                    weight: one,
                    value: temperature,
                }],
                normalization: pse_quantity::WeightNormalization::DivideBySum,
                unit_sum_invariant: None,
            },
            &[],
            None,
        )
        .unwrap();
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[good], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    assert_eq!(
        result.node(result.roots()[0]).unwrap().quantity_type,
        ids::quantity("temperature.point")
    );
}
