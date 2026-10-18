// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::{
    Function, MathError,
    jets::{EvaluationLimits, ProductKind, ProductValues},
    library::Optimization,
    typed::{Binary, BodyBuilder, BodyLimits},
};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_quantity::{
    IndexSet,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::collections::BTreeMap;

fn typed_sqrt_weighted_products(options: Optimization) {
    let registry = standard_registry().unwrap();
    let source = SemanticId::from_bytes([91; 16]);
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let x = builder
        .input(
            0,
            ids::quantity("neutral"),
            IndexSet::new(),
            SemanticId::NIL,
        )
        .unwrap();
    let u = builder.unary(Function::Sqrt, x.clone(), source).unwrap();
    // An authored shared binding keeps sqrt as an actual intermediate block.
    let u = builder.bind(u).unwrap();
    let square = builder
        .binary(Binary::Mul, u.clone(), u, None, SemanticId::NIL)
        .unwrap();
    let quadratic = builder
        .binary(Binary::Mul, x.clone(), x, None, SemanticId::NIL)
        .unwrap();
    let output = builder
        .binary(Binary::Add, square, quadratic, None, SemanticId::NIL)
        .unwrap();
    let body = builder.prepare(&[output]).unwrap();
    let cancel = pse_diagnostics::Cancellation::new(false);
    let support = body
        .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
        .unwrap();

    for (kind, values) in [
        (
            ProductKind::WeightedHessian,
            ProductValues::WeightedHessian { weights: &[1.] },
        ),
        (
            ProductKind::WeightedHessianAction,
            ProductValues::WeightedHessianAction {
                weights: &[1.],
                direction: &[1.],
            },
        ),
    ] {
        let compiled = support
            .compile_product(
                kind,
                options,
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        assert_eq!(compiled.product_len(), 1);
        assert!(compiled.census().any(|entry| entry.direct_weighted));
        let mut worker = compiled.worker();
        let mut providers = pse_kernels::ProviderBindings::new();
        let mut result = [93.];

        for expected_unchanged in [93., 2.] {
            let error = worker
                .evaluate_product_into(&[0.], values, &mut providers, &cancel, &mut result)
                .unwrap_err();
            assert!(matches!(
                error,
                MathError::Domain { source_id, requirement: "positive" } if source_id == source
            ));
            assert_eq!(result, [expected_unchanged]);

            // sqrt's intermediate second derivative overflows here. The requested
            // curvature of u²+x² is nevertheless finite, and its original guards hold.
            worker
                .evaluate_product_into(&[1e-300], values, &mut providers, &cancel, &mut result)
                .unwrap();
            assert_eq!(result, [2.]);
        }
    }
}

#[test]
fn typed_sqrt_guard_survives_interpreted_weighted_product_lowering() {
    typed_sqrt_weighted_products(Optimization::default());
}

#[cfg(feature = "compiled-evaluator")]
#[test]
fn typed_sqrt_guard_survives_compiled_weighted_product_lowering() {
    typed_sqrt_weighted_products(Optimization {
        evaluator: crate::library::Evaluator::CompiledScalar {
            module_bytes: 1 << 20,
        },
        ..Default::default()
    });
}
