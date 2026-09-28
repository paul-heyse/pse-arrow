// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared provider output envelopes (ADR-0105 §1).
#![allow(
    clippy::unwrap_used,
    reason = "test fixtures report a broken precondition by panicking"
)]
use super::*;
use pse_quantity::standard::{ids, standard_registry};

#[derive(Debug)]
struct Bounded(ProviderSpec, Option<Vec<(f64, f64)>>);
impl Provider for Bounded {
    fn spec(&self) -> &ProviderSpec {
        &self.0
    }
    fn evaluate(
        &mut self,
        _: &[f64],
        _: &ProviderRequest,
        _: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        Err(ProviderError::Trial("not evaluated".into()))
    }
}
impl ProviderFactory for Bounded {
    fn spec(&self) -> &ProviderSpec {
        &self.0
    }
    fn create(&self) -> Result<Box<dyn Provider>, ProviderError> {
        Ok(Box::new(Self(self.0.clone(), self.1.clone())))
    }
    fn envelope(&self) -> Option<Vec<(f64, f64)>> {
        self.1.clone()
    }
}

#[test]
fn provider_envelope_is_checked_against_the_contract() {
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let port = |n| Port {
        id: SemanticId::from_bytes([n; 16]),
        quantity,
        unit,
    };
    let spec = ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        id: SemanticId::from_bytes([9; 16]),
        revision: ContentHash::from_bytes([1; 32]),
        data: ContentHash::from_bytes([2; 32]),
        inputs: vec![port(1)],
        outputs: vec![port(2), port(3)],
        derivatives: DerivativeOrder::Value,
        smoothness: DerivativeOrder::Value,
    };
    let registration = |envelope| {
        Registration::new(
            std::sync::Arc::new(Bounded(spec.clone(), envelope)),
            &registry,
        )
        .unwrap()
    };
    // Providers enforce no envelope unless they declare one.
    assert_eq!(registration(None).envelope().unwrap(), None);
    let declared = vec![(0.0, 27.0), (-1.0, 1.0)];
    assert_eq!(
        registration(Some(declared.clone())).envelope().unwrap(),
        Some(declared)
    );
    // One closed, nonempty interval per output, or a typed contract refusal.
    for wrong in [
        vec![(0.0, 1.0)],
        vec![(0.0, 1.0), (2.0, 1.0)],
        vec![(0.0, 1.0), (f64::NAN, 1.0)],
    ] {
        assert!(matches!(
            registration(Some(wrong)).envelope(),
            Err(ProviderError::Contract(_))
        ));
    }
}
