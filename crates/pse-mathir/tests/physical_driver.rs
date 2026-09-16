// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical inference before typed identity, exercised through the actual P10 leaf entry.
use pse_ids::SemanticId;
use pse_mathir::canonicalize::{
    CanonicalizeInput, Policy, RootEnvironment, canonicalize, canonicalize_with_environments,
};
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
fn registered_division_supplies_the_unique_literal_operand_contract() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let temperature = symbol(&mut graph, &mut source, 1, "temperature.point");
    let scale = graph.float_const(1000., ids::unit("K")).unwrap();
    let ratio = graph.div(temperature, scale).unwrap();
    let two = graph.int_const(2).unwrap();
    let squared = graph.pow(ratio, two).unwrap();
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[ratio, squared], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    let root = result.node(result.roots()[0]).unwrap();
    assert_eq!(root.quantity_type, ids::quantity("neutral"));
    let denominator = result.node(root.children[1]).unwrap();
    assert_eq!(
        denominator.quantity_type,
        ids::quantity("temperature_scale")
    );
    assert!(matches!(
        denominator.payload,
        Payload::FloatConst { value: 1000., .. }
    ));
    let squared = result.node(result.roots()[1]).unwrap();
    assert_eq!(squared.opcode, Opcode::Pow);
    assert_eq!(squared.quantity_type, ids::quantity("neutral"));
    // The same untyped node remains ambiguous without that operation context.
    assert!(matches!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[scale], &source, &registry),
            Policy::Strict
        ),
        Err(MathIrError::Quantity {
            source: pse_quantity::QuantityError::AmbiguousLiteral { .. },
            ..
        })
    ));
}

#[test]
fn multiplication_keeps_ambiguous_literals_and_explicit_contracts() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let neutral = symbol(&mut graph, &mut source, 1, "neutral");
    let ambiguous = graph.float_const(20., ids::unit("K")).unwrap();
    // Neutral scaling admits several Kelvin types; neither operand position may pick one.
    for operands in [[neutral, ambiguous], [ambiguous, neutral]] {
        let product = graph.mul(operands[0], operands[1]).unwrap();
        assert!(matches!(
            canonicalize(
                CanonicalizeInput::new(&graph, &[product], &source, &registry),
                Policy::Strict
            ),
            Err(MathIrError::Quantity {
                source: pse_quantity::QuantityError::AmbiguousLiteral { .. },
                ..
            })
        ));
    }
    let explicit = graph
        .insert_typed(
            Opcode::Const,
            Payload::FloatConst {
                value: 20.,
                unit: ids::unit("K"),
            },
            &[],
            ids::quantity("temperature.difference"),
            None,
        )
        .unwrap();
    let product = graph.mul(explicit, neutral).unwrap();
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[product], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    assert_eq!(
        result.node(result.roots()[0]).unwrap().quantity_type,
        ids::quantity("temperature.difference")
    );
    let temperature = symbol(&mut graph, &mut source, 2, "temperature.point");
    let invalid_ratio = graph.div(temperature, explicit).unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[invalid_ratio], &source, &registry),
            Policy::Strict,
        )
        .is_err()
    );
}

#[test]
fn declared_result_constrains_a_literal_product_with_a_constant_sibling() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let source = Source::default();
    let kelvin = graph.float_const(20., ids::unit("K")).unwrap();
    let two = graph.int_const(2).unwrap();
    let product = graph.mul(kelvin, two).unwrap();
    let result = canonicalize_with_environments(
        CanonicalizeInput::new(&graph, &[product], &source, &registry),
        &[RootEnvironment {
            expected: Some(ids::quantity("temperature.difference")),
            ..RootEnvironment::default()
        }],
        Policy::Strict,
    )
    .unwrap();
    let root = result.node(result.roots()[0]).unwrap();
    assert_eq!(root.quantity_type, ids::quantity("temperature.difference"));
    assert!(matches!(
        root.payload,
        Payload::FloatConst { value: 40., .. }
    ));
}

#[test]
fn reference_points_require_difference_corrections_in_composed_expressions() {
    let registry = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let reference = symbol(&mut graph, &mut source, 1, "molar_enthalpy.point");
    let correction = symbol(&mut graph, &mut source, 2, "neutral");
    let unit = graph.float_const(1., ids::unit("J/mol")).unwrap();
    let increment = graph.mul(correction, unit).unwrap();
    let added = graph.add(reference, increment).unwrap();
    let subtracted = graph.sub(reference, increment).unwrap();
    let environment = RootEnvironment {
        expected: Some(ids::quantity("molar_enthalpy.point")),
        ..RootEnvironment::default()
    };
    let result = canonicalize_with_environments(
        CanonicalizeInput::new(&graph, &[added, subtracted], &source, &registry),
        &[environment.clone(), environment],
        Policy::Strict,
    )
    .unwrap();
    for root in result.roots() {
        let root = result.node(*root).unwrap();
        assert_eq!(root.quantity_type, ids::quantity("molar_enthalpy.point"));
        assert_eq!(
            result.node(root.children[1]).unwrap().quantity_type,
            ids::quantity("molar_enthalpy.difference")
        );
    }
    let invalid = graph.add(reference, reference).unwrap();
    assert!(matches!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[invalid], &source, &registry),
            Policy::Strict
        ),
        Err(MathIrError::Quantity {
            source: pse_quantity::QuantityError::Incompatible {
                reason: pse_quantity::IncompatibilityReason::PointPlusPoint,
                ..
            },
            ..
        })
    ));
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
