// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
#[test]
fn implicit_requirements_separate_c1_inner_minimum_and_selector_stability() {
    use DerivativeOrder::{First, Second, Value};
    let first = DerivativeRequirements::new(First, First, First, First, First).unwrap();
    assert_eq!(first.residual_compilation, First);
    assert_eq!(first.output_available(), First);
    assert!(matches!(
        DerivativeRequirements::new(First, First, First, First, Second).unwrap_err(),
        ProviderError::DerivativeUnavailable {
            capability: DerivativeCapability::Residual,
            requested: Second,
            available: First,
            ..
        }
    ));
    assert!(matches!(
        DerivativeRequirements::new(Second, Second, Value, First, First).unwrap_err(),
        ProviderError::DerivativeUnavailable {
            capability: DerivativeCapability::SelectorNeighborhood,
            requested: First,
            available: Value,
            ..
        }
    ));
    assert!(matches!(
        DerivativeRequirements::new(Second, First, Second, First, Second).unwrap_err(),
        ProviderError::DerivativeUnavailable {
            capability: DerivativeCapability::OutputSmoothness,
            requested: Second,
            available: First,
            ..
        }
    ));
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
    assert!(matches!(
        DerivativeRequirements::new(Value, Second, Second, First, Value).unwrap_err(),
        ProviderError::DerivativeUnavailable {
            capability: DerivativeCapability::Residual,
            requested: First,
            available: Value,
            ..
        }
    ));
}

#[test]
fn derivative_capability_refusal_keeps_orders_and_differs_from_contract_and_trial() {
    use pse_diagnostics::DiagnosticStage;
    use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection, Observation};
    let member = SemanticId::from_bytes([9; 16]);
    let error = ProviderError::DerivativeUnavailable {
        capability: DerivativeCapability::SelectorNeighborhood,
        requested: DerivativeOrder::First,
        available: DerivativeOrder::Value,
        members: vec![member],
    };
    let diagnostic = error.boundary_diagnostic(DiagnosticStage::Modeling);
    assert_eq!(diagnostic.class, BoundaryClass::Unsupported);
    assert_eq!(diagnostic.stage, DiagnosticStage::Modeling);
    assert_eq!(diagnostic.sources, vec![member]);
    assert!(matches!(
        diagnostic.observations.get("requested_derivative_order"),
        Some(Observation::Integer(1))
    ));
    assert!(matches!(
        diagnostic.observations.get("available_derivative_order"),
        Some(Observation::Integer(0))
    ));
    assert!(!error.recoverable());
    for (error, expected) in [
        (
            ProviderError::Contract("invalid registration".into()),
            BoundaryClass::InvalidModel,
        ),
        (
            ProviderError::Trial("trial outside domain".into()),
            BoundaryClass::TrialRejected,
        ),
        (
            ProviderError::Limit("allocation"),
            BoundaryClass::ResourceLimit,
        ),
        (
            ProviderError::Terminal("worker failed".into()),
            BoundaryClass::Infrastructure,
        ),
    ] {
        assert_eq!(
            error.boundary_diagnostic(DiagnosticStage::Modeling).class,
            expected
        );
    }
}

#[test]
fn bound_provider_keeps_admitted_meaning_and_refuses_uncompiled_orders() {
    use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection};
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
    let error = changed
        .check_bound(admitted.spec(), DerivativeOrder::Value)
        .unwrap_err();
    assert!(matches!(error, ProviderError::Contract(_)));
    assert_eq!(
        error
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Modeling)
            .class,
        BoundaryClass::InvalidModel
    );
    let mut wrong_quantity = bound.spec().clone();
    wrong_quantity.outputs[0].quantity = QuantityTypeId::from_id(SemanticId::from_bytes([99; 16]));
    let error = wrong_quantity.validate(&registry).unwrap_err();
    assert!(matches!(error, ProviderError::Contract(_)));
    assert_eq!(
        error
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Modeling)
            .class,
        BoundaryClass::InvalidModel
    );
    let mut widened = bound.spec().clone();
    widened.derivatives = DerivativeOrder::Second;
    assert!(
        widened
            .check_bound(bound.spec(), DerivativeOrder::Value)
            .is_err()
    );
}
