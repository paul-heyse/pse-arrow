// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete physical composition through strict canonicalization (blueprint §8.3).
use pse_ids::SemanticId;
use pse_mathir::{
    ExprGraph, NodeId, Opcode, Payload, WeightedPair,
    canonicalize::{CanonicalizeInput, Policy, canonicalize},
    infer::SymbolTypeSource,
};
use pse_quantity::{
    QuantityTypeId, WeightNormalization,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::collections::BTreeMap;

#[derive(Default)]
struct Source(BTreeMap<SemanticId, QuantityTypeId>);
impl SymbolTypeSource for Source {
    fn symbol_type(&self, symbol: SemanticId) -> Option<QuantityTypeId> {
        self.0.get(&symbol).copied()
    }
    fn invariant_checker(&self, _: NodeId) -> &dyn pse_quantity::infer::InvariantChecker {
        &StandardInvariantChecker
    }
}
#[expect(
    clippy::expect_used,
    reason = "concrete conformance expression construction"
)]
fn symbol(graph: &mut ExprGraph, source: &mut Source, ordinal: u8, name: &str) -> NodeId {
    let id = SemanticId::from_bytes([ordinal; 16]);
    source.0.insert(id, ids::quantity(name));
    graph.symbol(id).expect("symbol occurrence")
}
#[test]
fn heater_compositions_keep_actual_reference_basis_and_point_difference_contracts() {
    let registry = standard_registry().expect("physical fixture admitted");
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let flow = symbol(&mut graph, &mut source, 1, "molar_flow");
    let inlet = symbol(&mut graph, &mut source, 2, "molar_enthalpy.point");
    let outlet = symbol(&mut graph, &mut source, 3, "molar_enthalpy.point");
    let heat = symbol(&mut graph, &mut source, 4, "energy_flow");
    let fraction = symbol(&mut graph, &mut source, 5, "mole_fraction");
    let gas = symbol(&mut graph, &mut source, 6, "gas_constant");
    let temperature = symbol(&mut graph, &mut source, 7, "temperature.point");
    let pressure = symbol(&mut graph, &mut source, 8, "pressure.absolute");
    let activation = symbol(&mut graph, &mut source, 9, "activation_energy");
    let delta = symbol(&mut graph, &mut source, 10, "temperature.difference");
    let energy_in = graph.mul(flow, inlet).expect("inlet energy");
    let energy_out = graph.mul(flow, outlet).expect("outlet energy");
    let supplied = graph.add(energy_in, heat).expect("heat supplied");
    let balance = graph.sub(supplied, energy_out).expect("heater residual");
    let component = graph.mul(flow, fraction).expect("component flow");
    let rt = graph.mul(gas, temperature).expect("RT");
    let density = graph.div(pressure, rt).expect("P/RT");
    let ratio = graph.div(activation, rt).expect("Ea/RT");
    let exponent = graph
        .insert(Opcode::Exp, Payload::None, &[ratio], None)
        .expect("exp");
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
        .expect("explicit temperature scale");
    let polynomial = graph.div(temperature, scale).expect("T/1000K");
    let alpha = graph.int_const(2).expect("dimensionless alpha");
    let change = graph.mul(alpha, delta).expect("scaled difference");
    let adjusted = graph
        .add(temperature, change)
        .expect("point plus difference");
    let mean = graph
        .insert(
            Opcode::WeightedMean,
            Payload::WeightedMean {
                pairs: vec![
                    WeightedPair {
                        weight: alpha,
                        value: temperature,
                    },
                    WeightedPair {
                        weight: alpha,
                        value: adjusted,
                    },
                ],
                normalization: WeightNormalization::DivideBySum,
                unit_sum_invariant: None,
            },
            &[],
            None,
        )
        .expect("ordered mean");
    let roots = [
        balance, energy_in, component, rt, density, exponent, polynomial, adjusted, mean,
    ];
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &roots, &source, &registry),
        Policy::Strict,
    )
    .expect("actual physical inference");
    for (root, name) in result.roots().iter().zip([
        "energy_flow",
        "energy_flow",
        "component_flow",
        "molar_energy",
        "molar_density",
        "neutral",
        "neutral",
        "temperature.point",
        "temperature.point",
    ]) {
        assert_eq!(
            result.node(*root).expect("result").quantity_type,
            ids::quantity(name)
        );
    }
    let energy = registry
        .quantity_type(result.node(result.roots()[1]).unwrap().quantity_type)
        .unwrap();
    assert_eq!(energy.key.reference_state, Some(ids::reference("standard")));
    let component = registry
        .quantity_type(result.node(result.roots()[2]).unwrap().quantity_type)
        .unwrap();
    assert_eq!(component.key.basis, Some(ids::basis("molar")));
}

#[test]
fn dimensional_compatibility_cannot_admit_different_datums_bases_or_missing_operations() {
    let standard = standard_registry().unwrap();
    let mut builder = standard.to_builder();
    for (name, unit) in [("length", "m"), ("mass", "kg")] {
        builder.kind(pse_quantity::QuantityKind {
            id: ids::kind(name),
            dimension: standard.unit(ids::unit(unit)).unwrap().dimension,
            extensive: false,
            addition_kind: pse_quantity::QuantityAdditionKind::Additive,
        });
        builder.quantity_type(pse_quantity::QuantityType {
            id: ids::quantity(name),
            key: pse_quantity::QuantityTypeKey {
                kind: ids::kind(name),
                basis: None,
                reference_state: None,
                scale_kind: pse_quantity::ScaleKind::Point,
                shape: vec![],
                subject_kind: None,
            },
            canonical_unit: ids::unit(unit),
            nominal_magnitude: None,
        });
    }
    let registry = builder.build().unwrap();
    let mut graph = ExprGraph::new();
    let mut source = Source::default();
    let absolute = symbol(&mut graph, &mut source, 1, "pressure.absolute");
    let gauge = symbol(&mut graph, &mut source, 2, "pressure.gauge");
    let length = symbol(&mut graph, &mut source, 3, "length");
    let mass = symbol(&mut graph, &mut source, 4, "mass");
    let flow = symbol(&mut graph, &mut source, 5, "molar_flow");
    let mass_basis = registry
        .quantity_type(ids::quantity("molar_flow"))
        .unwrap()
        .clone();
    let mut changed = mass_basis;
    changed.id = ids::quantity("flow.with_wrong_basis");
    changed.key.basis = Some(ids::basis("mass"));
    let mut builder = registry.to_builder();
    builder.basis(pse_quantity::Basis {
        id: ids::basis("mass"),
        kind: pse_quantity::BasisKind::Mass,
        composition_basis: None,
        rate_basis: None,
        reference_conditions: None,
    });
    builder.quantity_type(changed);
    let registry = builder.build().unwrap();
    let wrong = symbol(&mut graph, &mut source, 6, "flow.with_wrong_basis");
    for root in [
        graph.sub(absolute, gauge).unwrap(),
        graph.mul(length, mass).unwrap(),
        graph.add(flow, wrong).unwrap(),
    ] {
        assert!(
            canonicalize(
                CanonicalizeInput::new(&graph, &[root], &source, &registry),
                Policy::Strict
            )
            .is_err()
        );
    }
}
