// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit physical constants and parameterized conversion contracts.
use pse_ids::SemanticId;
use pse_mathir::canonicalize::{CanonicalizeInput, Policy, canonicalize};
use pse_mathir::infer::{KernelContract, KernelPort, SymbolTypeSource};
use pse_mathir::payload::AffineTerm;
use pse_mathir::relations::vec_sink::KernelBinding;
use pse_mathir::{ExprGraph, NodeId, Opcode, Payload};
use pse_quantity::standard::{ids, standard_registry};
use pse_quantity::{ConversionId, ConversionKind, ConversionRule, InputConversion, QuantityTypeId};
use std::collections::BTreeMap;

#[derive(Default)]
struct Source {
    symbols: BTreeMap<SemanticId, QuantityTypeId>,
    kernels: BTreeMap<SemanticId, KernelContract>,
    conversions: BTreeMap<(NodeId, u16, ConversionId), SemanticId>,
}
impl SymbolTypeSource for Source {
    fn symbol_type(&self, symbol: SemanticId) -> Option<QuantityTypeId> {
        self.symbols.get(&symbol).copied()
    }
    fn kernel_contract(&self, kernel: SemanticId) -> Option<&KernelContract> {
        self.kernels.get(&kernel)
    }
    fn conversion_binding(
        &self,
        node: NodeId,
        operand: u16,
        conversion: ConversionId,
    ) -> Option<SemanticId> {
        self.conversions.get(&(node, operand, conversion)).copied()
    }
}
fn sid(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}

#[test]
fn affine_constants_keep_explicit_point_difference_meaning_and_round_trip() {
    let registry = standard_registry().unwrap();
    let source = Source {
        symbols: BTreeMap::from([(sid(1), ids::quantity("temperature.difference"))]),
        ..Source::default()
    };
    let mut graph = ExprGraph::new();
    let difference = graph.symbol(sid(1)).unwrap();
    let mut roots = vec![];
    for (value, quantity) in [
        (273.15, Some(ids::quantity("temperature.point"))),
        (0.0, Some(ids::quantity("temperature.point"))),
        (-0.0, None),
    ] {
        roots.push(
            graph
                .insert(
                    Opcode::Affine,
                    Payload::Affine {
                        constant: value,
                        constant_quantity_type: quantity,
                        constant_unit: quantity.map(|_| ids::unit("K")),
                        terms: vec![AffineTerm {
                            coefficient: 1.0,
                            child: difference,
                        }],
                    },
                    &[difference],
                    None,
                )
                .unwrap(),
        );
    }
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &roots, &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    for (root, expected) in result.roots().iter().zip([
        "temperature.point",
        "temperature.point",
        "temperature.difference",
    ]) {
        assert_eq!(
            result.node(*root).unwrap().quantity_type,
            ids::quantity(expected)
        );
    }
    assert!(matches!(result.node(result.roots()[2]).unwrap().payload,
        Payload::Affine {constant, constant_quantity_type: None, constant_unit: None, ..}
        if constant.to_bits() == (-0.0f64).to_bits()));
    let mut rows = pse_mathir::relations::VecSink::new();
    pse_mathir::relations::emit(&result, &mut rows).unwrap();
    let loaded =
        pse_mathir::relations::load_canonical(&rows, result.roots(), &source, &registry).unwrap();
    assert_eq!(result.listing(), loaded.listing());
}

#[test]
fn affine_constants_reject_missing_pair_and_noncanonical_representation() {
    let registry = standard_registry().unwrap();
    let source = Source::default();
    let mut graph = ExprGraph::new();
    for (value, quantity, unit) in [
        (1.0, None, None),
        (0.0, Some(ids::quantity("temperature.point")), None),
        (0.0, None, Some(ids::unit("K"))),
    ] {
        assert!(
            graph
                .insert(
                    Opcode::Affine,
                    Payload::Affine {
                        constant: value,
                        constant_quantity_type: quantity,
                        constant_unit: unit,
                        terms: vec![],
                    },
                    &[],
                    None
                )
                .is_err()
        );
    }
    let root = graph
        .insert(
            Opcode::Affine,
            Payload::Affine {
                constant: 20.0,
                constant_quantity_type: Some(ids::quantity("temperature.point")),
                constant_unit: Some(ids::unit("degC")),
                terms: vec![],
            },
            &[],
            None,
        )
        .unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[root], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
}

#[test]
fn parameterized_conversion_binds_actual_input_and_parameter_then_round_trips() {
    let standard = standard_registry().unwrap();
    let mut builder = standard.to_builder();
    let conversion = ids::conversion("activity_normalization");
    builder.conversion(ConversionRule {
        id: conversion,
        from: ids::quantity("mole_fraction"),
        to: ids::quantity("neutral"),
        kind: ConversionKind::Kernel,
        kernel: Some(sid(2)),
        required_parameters: vec!["scale".into()],
        scale: None,
        offset: None,
    });
    let mut operation = standard.operations_for(Opcode::Sin).next().unwrap().clone();
    operation.id = ids::operation("sin_normalized_activity");
    operation.input_conversions.push(InputConversion {
        operand: 0,
        conversion,
    });
    builder.operation(operation);
    let registry = builder.build().unwrap();
    let port = |name: &str, quantity: &str| KernelPort {
        name: name.into(),
        quantity_type: ids::quantity(quantity),
        unit: ids::unit("1"),
        shape: vec![],
    };
    let mut source = Source {
        symbols: BTreeMap::from([(sid(1), ids::quantity("mole_fraction"))]),
        kernels: BTreeMap::from([(
            sid(2),
            KernelContract {
                inputs: vec![port("fraction", "mole_fraction")],
                outputs: vec![port("activity", "neutral")],
                parameters: vec![port("scale", "neutral")],
            },
        )]),
        ..Source::default()
    };
    let mut graph = ExprGraph::new();
    let fraction = graph.symbol(sid(1)).unwrap();
    let root = graph
        .insert(Opcode::Sin, Payload::None, &[fraction], None)
        .unwrap();
    source.conversions.insert((root, 0, conversion), sid(3));
    let bindings = BTreeMap::from([(
        sid(3),
        KernelBinding {
            binding: sid(3),
            kernel: sid(2),
            scope: sid(4),
            parameters: vec![("scale".into(), None, Some(2.0), Some(ids::unit("1")))],
            inputs: vec![("fraction".into(), fraction)],
        },
    )]);
    let roots = [root];
    let mut input = CanonicalizeInput::new(&graph, &roots, &source, &registry);
    input.kernel_bindings = &bindings;
    let result = canonicalize(input, Policy::Strict).unwrap();
    let call = result
        .iter()
        .find(|(_, node)| node.opcode == Opcode::KernelCall)
        .unwrap()
        .0;
    let selection = result
        .selections()
        .iter()
        .find(|row| row.node == call)
        .unwrap();
    assert_eq!(selection.conversions, [(0, conversion)]);
    assert_eq!(
        result.node(result.roots()[0]).unwrap().quantity_type,
        ids::quantity("neutral")
    );
    let mut rows = pse_mathir::relations::VecSink::new();
    pse_mathir::relations::emit(&result, &mut rows).unwrap();
    let loaded =
        pse_mathir::relations::load_canonical(&rows, result.roots(), &source, &registry).unwrap();
    assert_eq!(result.listing(), loaded.listing());
    for wrong in 0..6 {
        let mut corrupt = bindings.clone();
        let binding = corrupt.get_mut(&sid(3)).unwrap();
        match wrong {
            0 => binding.parameters[0].0 = "unrelated".into(),
            1 => binding.parameters[0].2 = Some(f64::NAN),
            2 => binding.parameters[0].3 = Some(ids::unit("K")),
            3 => binding.inputs[0].1 = root,
            4 => binding.inputs[0].0 = "unrelated".into(),
            _ => binding.binding = sid(99),
        }
        let mut invalid = input;
        invalid.kernel_bindings = &corrupt;
        assert!(canonicalize(invalid, Policy::Strict).is_err());
    }
    source.conversions.clear();
    let mut missing = CanonicalizeInput::new(&graph, &roots, &source, &registry);
    missing.kernel_bindings = &bindings;
    assert!(canonicalize(missing, Policy::Strict).is_err());
}

#[test]
fn pending_unit_request_resolves_only_after_complete_point_or_difference_context() {
    let registry = standard_registry().unwrap();
    let source = Source::default();
    for (quantity, expected) in [
        ("temperature.point", 283.15_f64),
        ("temperature.difference", 10.0_f64),
    ] {
        let mut graph = ExprGraph::new();
        let literal = graph.float_const(10., ids::unit("degC")).unwrap();
        let root = graph
            .insert_typed(
                Opcode::UnitConvert,
                Payload::PendingUnitConvert { to: ids::unit("K") },
                &[literal],
                ids::quantity(quantity),
                None,
            )
            .unwrap();
        let output = canonicalize(
            CanonicalizeInput::new(&graph, &[root], &source, &registry),
            Policy::Strict,
        )
        .unwrap();
        let root = output.node(output.roots()[0]).unwrap();
        assert!(
            matches!(root.payload, Payload::FloatConst { value, unit } if value.to_bits() == expected.to_bits() && unit == ids::unit("K"))
        );
        assert_eq!(root.quantity_type, ids::quantity(quantity));
    }
    let mut graph = ExprGraph::new();
    let literal = graph.float_const(10., ids::unit("degC")).unwrap();
    let root = graph
        .insert(
            Opcode::UnitConvert,
            Payload::PendingUnitConvert { to: ids::unit("K") },
            &[literal],
            None,
        )
        .unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[root], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
}

#[test]
fn pending_smoothing_units_round_trip_and_recompute_from_actual_changed_definitions() {
    use pse_mathir::relations::{VecSink, emit, emit_untyped, load_canonical, load_untyped};
    let standard = standard_registry().unwrap();
    let source = Source {
        symbols: BTreeMap::from([
            (sid(1), ids::quantity("temperature.point")),
            (sid(2), ids::quantity("temperature.point")),
        ]),
        ..Source::default()
    };
    let mut graph = ExprGraph::new();
    let right = graph.symbol(sid(2)).unwrap();
    let left = graph.symbol(sid(1)).unwrap();
    let root = graph
        .insert(
            Opcode::SmoothMax,
            Payload::PendingSmoothOp {
                eps: 2.0,
                unit: ids::unit("declared_tolerance"),
            },
            &[right, left],
            None,
        )
        .unwrap();
    let mut rows = VecSink::new();
    emit_untyped(&graph, &[root], &mut rows).unwrap();
    let loaded = load_untyped(&rows, &[root]).unwrap();
    assert!(
        matches!(loaded.graph.node(loaded.roots[0]).unwrap().payload, Payload::PendingSmoothOp { eps: 2.0, unit } if unit == ids::unit("declared_tolerance"))
    );
    for (scale, expected) in [(0.5, 1.0_f64), (0.25, 0.5)] {
        let mut builder = standard.to_builder();
        let mut declared = standard.unit(ids::unit("degF")).unwrap().clone();
        declared.id = ids::unit("declared_tolerance");
        declared.symbol = "declared_tolerance".into();
        declared.scale_to_canonical = scale;
        // The unit identity and origin stay fixed while the actual declared scale changes.
        builder.unit(declared);
        let registry = builder.build().unwrap();
        let result = canonicalize(
            CanonicalizeInput::new(&loaded.graph, &loaded.roots, &source, &registry),
            Policy::Strict,
        )
        .unwrap();
        let node = result.node(result.roots()[0]).unwrap();
        assert!(
            matches!(node.payload, Payload::SmoothOp { eps } if eps.to_bits() == expected.to_bits())
        );
        assert_eq!(node.quantity_type, ids::quantity("temperature.point"));
        for (child, expected_symbol) in node.children.iter().zip([sid(2), sid(1)]) {
            assert!(
                matches!(result.node(*child).unwrap().payload, Payload::SymbolRef { symbol: pse_mathir::ValueRef::ActualSymbol(id) } if id == expected_symbol)
            );
        }
        let mut resolved = VecSink::new();
        emit(&result, &mut resolved).unwrap();
        let reopened = load_canonical(&resolved, result.roots(), &source, &registry).unwrap();
        assert_eq!(result.listing(), reopened.listing());
    }
    let bad = graph
        .insert(
            Opcode::SmoothAbs,
            Payload::PendingSmoothOp {
                eps: 1.0,
                unit: ids::unit("Pa"),
            },
            &[left],
            None,
        )
        .unwrap();
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[bad], &source, &standard),
            Policy::Strict
        )
        .is_err()
    );
    for eps in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert!(
            graph
                .insert(
                    Opcode::SmoothAbs,
                    Payload::PendingSmoothOp {
                        eps,
                        unit: ids::unit("K")
                    },
                    &[left],
                    None
                )
                .is_err()
        );
    }
}
