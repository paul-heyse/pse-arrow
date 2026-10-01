// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
#[test]
fn implicit_requirements_separate_c1_inner_minimum_and_selector_stability() {
    use DerivativeOrder::{First, Second, Value};
    let first = DerivativeRequirements::new(First, First, First, First, First).unwrap();
    assert_eq!(first.residual_compilation, First);
    assert_eq!(first.output_available(), First);
    assert!(
        DerivativeRequirements::new(First, First, First, First, Second)
            .unwrap_err()
            .to_string()
            .contains("residual derivative")
    );
    assert!(
        DerivativeRequirements::new(Second, Second, Value, First, First)
            .unwrap_err()
            .to_string()
            .contains("selector neighborhood")
    );
    assert_eq!(
        DerivativeRequirements::new(Second, Second, Value, First, Value)
            .unwrap()
            .residual_compilation,
        First
    );
    assert_eq!(
        DerivativeRequirements::new(Second, Second, Second, First, Second)
            .unwrap()
            .residual_compilation,
        Second
    );
    assert!(
        DerivativeRequirements::new(Value, Second, Second, First, Value)
            .unwrap_err()
            .to_string()
            .contains("residual derivative")
    );
}

#[test]
fn bound_provider_keeps_admitted_meaning_and_refuses_uncompiled_orders() {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let admitted = AdmittedProvider::new(
        ProviderSpec {
            shapes: ProviderShapes::default(),
            derivative_source: DerivativeSource::Analytic,
            id: SemanticId::from_bytes([1; 16]),
            revision: ContentHash::from_bytes([2; 32]),
            data: ContentHash::from_bytes([3; 32]),
            inputs: vec![],
            outputs: vec![Port {
                id: SemanticId::from_bytes([4; 16]),
                quantity,
                unit: registry.quantity_type(quantity).unwrap().canonical_unit,
            }],
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        },
        &registry,
    )
    .unwrap();
    let bound = admitted.restrict_order(DerivativeOrder::First).unwrap();
    assert_ne!(bound.spec().key(), admitted.spec().key());
    bound
        .spec()
        .check_bound(admitted.spec(), DerivativeOrder::First)
        .unwrap();
    assert!(
        bound
            .spec()
            .check_bound(admitted.spec(), DerivativeOrder::Second)
            .unwrap_err()
            .to_string()
            .contains("bound derivative capability")
    );
    assert!(bound.restrict_order(DerivativeOrder::Second).is_err());
    let mut changed = bound.spec().clone();
    changed.revision = ContentHash::from_bytes([5; 32]);
    assert!(
        changed
            .check_bound(admitted.spec(), DerivativeOrder::Value)
            .unwrap_err()
            .to_string()
            .contains("changed its contract")
    );
    let mut widened = bound.spec().clone();
    widened.derivatives = DerivativeOrder::Second;
    assert!(
        widened
            .check_bound(bound.spec(), DerivativeOrder::Value)
            .is_err()
    );
}
