// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared provider output envelopes (ADR-0105 §1, ADR-0120).
#![allow(
    clippy::unwrap_used,
    reason = "test fixtures report a broken precondition by panicking"
)]
use super::*;
use pse_quantity::standard::{ids, standard_registry};

/// A provider that returns fixed outputs and declares an optional envelope.
#[derive(Debug)]
struct Bounded {
    spec: ProviderSpec,
    envelope: Option<Vec<(f64, f64)>>,
    outputs: Vec<f64>,
}
impl Provider for Bounded {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn evaluate(
        &mut self,
        _: &[f64],
        request: &ProviderRequest,
        _: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        Ok(ProviderValues {
            values: request.outputs.iter().map(|&o| self.outputs[o]).collect(),
            jacobian: Vec::new(),
            hessians: Vec::new(),
        })
    }
}
impl ProviderFactory for Bounded {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn create(&self) -> Result<Box<dyn Provider>, ProviderError> {
        Ok(Box::new(Self {
            spec: self.spec.clone(),
            envelope: self.envelope.clone(),
            outputs: self.outputs.clone(),
        }))
    }
    fn envelope(&self) -> Option<Vec<(f64, f64)>> {
        self.envelope.clone()
    }
}

fn spec() -> ProviderSpec {
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let port = |n| Port {
        id: SemanticId::from_bytes([n; 16]),
        quantity,
        unit,
    };
    ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        id: SemanticId::from_bytes([9; 16]),
        revision: ContentHash::from_bytes([1; 32]),
        data: ContentHash::from_bytes([2; 32]),
        inputs: vec![port(1)],
        outputs: vec![port(2), port(3)],
        derivatives: DerivativeOrder::Value,
        smoothness: DerivativeOrder::Value,
    }
}

fn register(
    envelope: Option<Vec<(f64, f64)>>,
    outputs: Vec<f64>,
) -> Result<Registration, ProviderError> {
    let registry = standard_registry().unwrap();
    Registration::new(
        Arc::new(Bounded {
            spec: spec(),
            envelope,
            outputs,
        }),
        &registry,
    )
}

#[test]
fn provider_envelope_is_checked_against_the_contract() {
    // Providers enforce no envelope unless they declare one.
    assert_eq!(register(None, vec![0.0, 0.0]).unwrap().envelope(), None);
    let declared = vec![(0.0, 27.0), (-1.0, 1.0)];
    assert_eq!(
        register(Some(declared.clone()), vec![0.0, 0.0])
            .unwrap()
            .envelope(),
        Some(declared.as_slice())
    );
    // One closed, nonempty interval per output, or a typed contract refusal at registration.
    for wrong in [
        vec![(0.0, 1.0)],
        vec![(0.0, 1.0), (2.0, 1.0)],
        vec![(0.0, 1.0), (f64::NAN, 1.0)],
    ] {
        assert!(matches!(
            register(Some(wrong), vec![0.0, 0.0]),
            Err(ProviderError::Contract(_))
        ));
    }
    // The factory-free descriptor bound to a factory is checked the same way.
    let registry = standard_registry().unwrap();
    let descriptor = AdmittedProvider::new(spec(), &registry).unwrap();
    let factory = Arc::new(Bounded {
        spec: spec(),
        envelope: Some(vec![(0.0, 1.0)]),
        outputs: vec![0.0, 0.0],
    });
    assert!(matches!(
        Registration::bind(descriptor, factory),
        Err(ProviderError::Contract(_))
    ));
}

#[test]
fn envelope_rejects_empty_interval() {
    // Each interval must contain a real number (ADR-0120 item 5).
    let inf = f64::INFINITY;
    for empty in [(inf, inf), (-inf, -inf)] {
        assert!(matches!(
            register(Some(vec![(0.0, 1.0), empty]), vec![0.0, 0.0]),
            Err(ProviderError::Contract(_))
        ));
    }
    // Half-bounded and unbounded intervals contain reals and are admitted.
    let admitted = vec![(-inf, 0.0), (5.0, inf)];
    assert_eq!(
        register(Some(admitted.clone()), vec![0.0, 5.0])
            .unwrap()
            .envelope(),
        Some(admitted.as_slice())
    );
    assert!(register(Some(vec![(-inf, inf), (3.0, 3.0)]), vec![0.0, 3.0]).is_ok());
}

#[test]
fn envelope_violation_is_typed_contract_error() {
    let cancelled = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancelled,
        max_result_bytes: 1 << 20,
    };
    let envelope = Some(vec![(0.0, 27.0), (-1.0, 1.0)]);
    let all = ProviderRequest::all(&spec(), DerivativeOrder::Value);
    // Values inside the envelope pass through unchanged.
    let inside = register(envelope.clone(), vec![27.0, -0.5]).unwrap();
    assert_eq!(
        inside
            .worker()
            .unwrap()
            .evaluate(&[1.0], &all, &context)
            .unwrap()
            .values,
        vec![27.0, -0.5]
    );
    // The second output breaks its declared interval: a contract error naming it, never a
    // trial rejection. The registration itself admits the provider; only evaluation shows
    // the declaration is false.
    let outside = register(envelope, vec![1.0, 1.5]).unwrap();
    let mut worker = outside.worker().unwrap();
    let failure = worker.evaluate(&[1.0], &all, &context).unwrap_err();
    let ProviderError::Contract(message) = &failure else {
        panic!("expected a contract error, got {failure:?}");
    };
    assert!(message.contains("output 1"), "{message}");
    // A request for only the first output does not evaluate the broken one.
    let first = ProviderRequest {
        outputs: vec![0],
        order: DerivativeOrder::Value,
    };
    assert_eq!(
        worker.evaluate(&[1.0], &first, &context).unwrap().values,
        vec![1.0]
    );
    // A request that reorders outputs is checked against each output's own interval.
    let reordered = ProviderRequest {
        outputs: vec![1, 0],
        order: DerivativeOrder::Value,
    };
    assert!(matches!(
        worker.evaluate(&[1.0], &reordered, &context),
        Err(ProviderError::Contract(_))
    ));
    // Scoped workers are checked the same way.
    assert!(matches!(
        outside
            .worker_scoped(ExecutionScope::new(Arc::new(AtomicBool::new(false)), None))
            .unwrap()
            .evaluate(&[1.0], &all, &context),
        Err(ProviderError::Contract(_))
    ));
    // Without a declaration, the same outputs are returned unchecked.
    assert_eq!(
        register(None, vec![1.0, 1.5])
            .unwrap()
            .worker()
            .unwrap()
            .evaluate(&[1.0], &all, &context)
            .unwrap()
            .values,
        vec![1.0, 1.5]
    );
}

#[test]
fn envelope_in_provider_key() {
    let registry = standard_registry().unwrap();
    let factory = Bounded {
        spec: spec(),
        envelope: None,
        outputs: vec![0.0, 0.0],
    };
    let key = |envelope| {
        register(envelope, vec![0.0, 0.0])
            .unwrap()
            .configuration_key()
    };
    // A provider without an envelope keeps its factory's key.
    assert_eq!(key(None), factory.configuration_key());
    assert_eq!(
        Registration::new(Arc::new(factory), &registry)
            .unwrap()
            .configuration_key(),
        key(None)
    );
    // The host frames the checked envelope: declaring one, or changing any endpoint,
    // changes the key.
    let a = key(Some(vec![(0.0, 27.0), (-1.0, 1.0)]));
    assert_ne!(a, key(None));
    assert_eq!(a, key(Some(vec![(0.0, 27.0), (-1.0, 1.0)])));
    assert_ne!(a, key(Some(vec![(0.0, 27.0), (-1.0, 2.0)])));
    assert_ne!(a, key(Some(vec![(-1.0, 1.0), (0.0, 27.0)])));
    assert_ne!(
        key(Some(vec![(0.0, 0.0), (0.0, 1.0)])),
        key(Some(vec![(-0.0, 0.0), (0.0, 1.0)]))
    );
}
