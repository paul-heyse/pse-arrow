// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_kernels::{
    DerivativeSource, EvaluationContext, Port, PreparedProviderRequest, Provider, ProviderBinding,
    ProviderBindings, ProviderDemand, ProviderError, ProviderOutput, ProviderRuntime,
    ProviderShapes, ProviderSpec,
};
use pse_quantity::standard::{StandardInvariantChecker, ids, standard_registry};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn fixture() -> (
    QuantityRegistry,
    pse_kernels::AdmittedProvider,
    ResolvedPhysicalContract,
) {
    let original = standard_registry().unwrap();
    let mut declarations = original.to_builder();
    let mut unit = original.unit(ids::unit("K")).unwrap().clone();
    unit.id = id(81).into();
    unit.symbol = "twiceK_provider".into();
    unit.scale_to_canonical = 2.0;
    declarations.unit(unit.clone());
    let registry = declarations.build().unwrap();
    let quantity = ids::quantity("temperature.point");
    let spec = ProviderSpec {
        id: id(82),
        revision: ContentHash::from_bytes([83; 32]),
        data: ContentHash::from_bytes([84; 32]),
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        inputs: vec![Port {
            id: id(85),
            quantity,
            unit: unit.id,
        }],
        outputs: vec![Port {
            id: id(86),
            quantity,
            unit: ids::unit("degC"),
        }],
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    };
    let descriptor = pse_kernels::AdmittedProvider::new(spec, &registry).unwrap();
    let contract = ResolvedPhysicalContract::named(quantity, IndexSet::new(), &registry).unwrap();
    (registry, descriptor, contract)
}
#[derive(Debug)]
struct Identity {
    calls: Arc<AtomicUsize>,
}
impl Provider for Identity {
    fn supports(&self, demand: &ProviderDemand) -> bool {
        matches!(demand, ProviderDemand::CompleteJets)
    }
    fn fill(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        _runtime: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        request.check_context(context)?;
        self.calls.fetch_add(1, Ordering::Relaxed);
        if inputs[0] < 0.0 {
            return Err(ProviderError::Trial(
                "original identity trial domain".into(),
            ));
        }
        output.values.fill(inputs[0]);
        output.jacobian.fill(1.0);
        output.hessians.fill(0.0);
        Ok(())
    }
}
fn body(
    registry: &QuantityRegistry,
    call: &PreparedProviderCall,
    source: SemanticId,
    ordinary: bool,
) -> Arc<PreparedBody> {
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let input = builder
        .input_admitted(0, call.inputs()[0].clone(), source)
        .unwrap();
    let values = if ordinary {
        builder.provider_partial(call.descriptor(), &[input], call.partial(), source)
    } else {
        builder.provider_partial_admitted(call, &[input], source)
    }
    .unwrap();
    Arc::new(builder.prepare(&values).unwrap())
}
fn evaluate(
    prepared: &Arc<PreparedBody>,
    descriptor: &pse_kernels::AdmittedProvider,
    input: f64,
    calls: Arc<AtomicUsize>,
) -> Result<crate::guarded::Evaluation, MathError> {
    let cancel = Arc::new(AtomicBool::new(false));
    let operation = pse_diagnostics::Cancellation::default();
    let compiled = prepared.compile(
        &[0],
        &[],
        DerivativeOrder::Value,
        Optimization::default(),
        crate::jets::EvaluationLimits::default(),
        &pse_kernels::ExecutionScope::new(operation, None),
    )?;
    let mut providers = ProviderBindings::from([(
        descriptor.spec().key(),
        ProviderBinding::new(descriptor.spec().clone(), Box::new(Identity { calls })),
    )]);
    compiled
        .worker()
        .evaluate(&[input], DerivativeOrder::Value, &mut providers, &cancel)
}
#[test]
fn prepared_provider_value_and_partial_preserve_exact_affine_units_and_worker_behavior() {
    let (registry, descriptor, input) = fixture();
    let calls = Arc::new(AtomicUsize::new(0));
    for (partial, expected) in [(&[][..], 428.15), (&[0][..], 0.5)] {
        let prepared = PreparedProviderCall::prepare(
            &descriptor,
            std::slice::from_ref(&input),
            partial,
            &registry,
            &StandardInvariantChecker,
        )
        .unwrap();
        let admitted = body(&registry, &prepared, id(1), false);
        let ordinary = body(&registry, &prepared, id(2), true);
        let first = evaluate(&admitted, &descriptor, 310.0, calls.clone()).unwrap();
        let old = evaluate(&ordinary, &descriptor, 310.0, calls.clone()).unwrap();
        assert!((first.values[0] - expected).abs() < 1e-12);
        assert_eq!(first.values, old.values);
        assert!(evaluate(&admitted, &descriptor, -10.0, calls.clone()).is_err());
        assert_eq!(
            evaluate(&admitted, &descriptor, 310.0, calls.clone())
                .unwrap()
                .values,
            first.values
        );
        assert_eq!(
            PreparedProviderCallRecord::capture(&prepared),
            PreparedProviderCallRecord::capture(
                &PreparedProviderCall::prepare(
                    &descriptor,
                    std::slice::from_ref(&input),
                    partial,
                    &registry,
                    &StandardInvariantChecker
                )
                .unwrap()
            ),
            "prepared physical identity has no source occurrence input"
        );
    }
    assert!(calls.load(Ordering::Relaxed) >= 8);
}
#[test]
fn prepared_provider_refuses_actual_contract_mismatch_before_any_worker() {
    let (registry, descriptor, input) = fixture();
    let prepared = PreparedProviderCall::prepare(
        &descriptor,
        &[input],
        &[],
        &registry,
        &StandardInvariantChecker,
    )
    .unwrap();
    let wrong =
        ResolvedPhysicalContract::named(ids::quantity("neutral"), IndexSet::new(), &registry)
            .unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let value = builder.input_admitted(0, wrong, id(3)).unwrap();
    assert!(
        builder
            .provider_partial_admitted(&prepared, &[value], id(4))
            .is_err()
    );
}
#[allow(
    unsafe_code,
    reason = "private immutable scientific record fixture supplies exact qualified authority; never exposed publicly"
)]
fn replay(
    record: &PreparedProviderCallRecord,
    retained: &PreparedProviderCallRecord,
) -> Result<PreparedProviderCall, MathError> {
    let (_, receipts) =
        pse_quantity::resolved::receipts::capture(|| Ok::<_, MathError>(())).unwrap();
    let payload =
        serde_json::to_vec(&serde_json::json!({"record":retained,"receipts":&receipts})).unwrap();
    // SAFETY: private owner-produced immutable unit fixture, including deliberately altered records.
    let permit = unsafe {
        pse_ids::scientific_replay::QualifiedScientificRecipe::from_canonical_product(
            payload.clone(),
            pse_ids::scientific_replay::admitted_recipe_payload_hash(&payload),
            ContentHash::from_bytes([1; 32]),
            ContentHash::from_bytes([2; 32]),
        )
    }
    .unwrap();
    pse_quantity::resolved::receipts::replay(&receipts, permit.records(), || record.restore())
}
#[test]
fn prepared_provider_record_requires_exact_qualified_membership_and_restores_all_nested_facts() {
    let (registry, descriptor, input) = fixture();
    let prepared = PreparedProviderCall::prepare(
        &descriptor,
        &[input],
        &[0],
        &registry,
        &StandardInvariantChecker,
    )
    .unwrap();
    let record = PreparedProviderCallRecord::capture(&prepared);
    assert!(record.restore().is_err());
    let restored = replay(&record, &record).unwrap();
    assert_eq!(identity(&restored), identity(&prepared));
    assert_eq!(
        evaluate(
            &body(&registry, &restored, id(5), false),
            &descriptor,
            310.0,
            Arc::new(AtomicUsize::new(0))
        )
        .unwrap()
        .values,
        [0.5]
    );
    let mut altered = record.clone();
    altered.partial.clear();
    assert!(replay(&altered, &record).is_err());
    assert!(replay(&altered, &altered).is_err());
    let mut unknown = serde_json::to_value(&record).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("fabricated_owner".into(), serde_json::json!(1));
    assert!(serde_json::from_value::<PreparedProviderCallRecord>(unknown).is_err());
}
