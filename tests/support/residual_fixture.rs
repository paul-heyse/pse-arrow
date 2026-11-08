// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared construction of hand-authored residual test inputs.
#![allow(
    clippy::expect_used,
    reason = "each fixture prerequisite reports its own failure"
)]

use super::math_fixture_api::{
    SourceBindings, SourceBody,
    guarded::{CompiledBody, PreparedBody},
    implicit::ResidualBodies,
    jets::EvaluationLimits,
    library::Optimization,
};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use std::sync::Arc;

pub(crate) fn source_body_for_test(
    instance: SemanticId,
    prepared: &PreparedBody,
    compiled: Arc<CompiledBody>,
) -> SourceBody {
    assert!(prepared.checked_members().is_empty());
    assert!(prepared.checked_sources().is_empty());
    assert!(prepared.occurrence_tokens().is_empty());
    let bindings = SourceBindings::new(
        instance,
        prepared,
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .expect("handcrafted fixture source bindings");
    SourceBody::new(compiled, bindings).expect("handcrafted fixture compiled source body")
}

pub(super) fn residual_bodies(
    parameter: &SourceBody,
    unknown_axes: &[usize],
    unknown_order: DerivativeOrder,
    unknown_options: (Optimization, EvaluationLimits),
    source_options: (Optimization, EvaluationLimits),
) -> ResidualBodies {
    let control = pse_diagnostics::Cancellation::new(false);
    let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
    let compile = |axes, order, (optimization, limits)| {
        let compiled = parameter
            .prepared_support()
            .incidence(axes, &control.flag())
            .expect("fixture derivative axes")
            .at_order(order)
            .expect("fixture derivative order")
            .compile(optimization, limits, &scope)
            .expect("fixture body compilation");
        SourceBody::new(Arc::new(compiled), parameter.bindings().clone())
            .expect("fixture source bindings")
    };
    ResidualBodies {
        unknown: compile(unknown_axes, unknown_order, unknown_options),
        source: compile(&[], DerivativeOrder::Value, source_options),
    }
}
