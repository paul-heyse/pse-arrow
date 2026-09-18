// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Capability and malformed-problem admission without a solver installation.
#![allow(
    clippy::unwrap_used,
    reason = "small native planning contract fixtures"
)]
use datafusion::{
    common::Column,
    logical_expr::{LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_backend_native::{
    HessianPolicy, SolveOptions, Variable,
    native::{Constraint, Solve},
};

fn plan(
    lower: Option<f64>,
    upper: Option<f64>,
    scale: f64,
) -> Result<LogicalPlan, pse_backend_native::NativeError> {
    let input = LogicalPlanBuilder::empty(true)
        .project(vec![lit(1.0_f64).alias("x"), scenario()])
        .unwrap()
        .build()
        .unwrap();
    Solve::plan(
        pse_ids::SemanticId::from_bytes([90; 16]),
        input,
        vec![Variable {
            column: Column::from_name("x"),
            lower,
            upper,
            scale,
        }],
        vec![Constraint {
            expression: col("x"),
            lower: Some(0.0),
            upper: Some(0.0),
            scale: 1.0,
        }],
        lit(0.0),
        SolveOptions {
            hessian: HessianPolicy::LimitedMemory,
            max_iterations: 10,
            tolerance: 1e-8,
            max_wall_seconds: 1.0,
        },
    )
}
#[test]
fn invalid_case_bounds_and_scales_are_refused_before_planning() {
    for (lower, upper, scale) in [
        (Some(2.0), Some(1.0), 1.0),
        (Some(f64::NAN), None, 1.0),
        (None, Some(f64::INFINITY), 1.0),
        (None, None, 0.0),
        (None, None, f64::NAN),
        (Some(f64::MAX), None, 1.0),
    ] {
        assert!(plan(lower, upper, scale).is_err());
    }
    assert!(plan(None, None, 1.0).is_ok());
}

#[cfg(not(feature = "ipopt"))]
#[tokio::test]
async fn unavailable_native_capability_is_explicit_and_never_selects_another_backend() {
    use datafusion::execution::runtime_env::RuntimeEnv;
    use pse_backend_native::native::NativeSolverPlanner;
    use pse_catalog::session::{
        ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
        planner::UnifiedPlanner,
    };
    use pse_ids::{CancellationToken, FixedBudget};
    use std::{collections::BTreeMap, sync::Arc};
    let cancel = CancellationToken::new();
    let session = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap()
    .with_query_planner(Arc::new(UnifiedPlanner::new(vec![Arc::new(
        NativeSolverPlanner::new(1.try_into().unwrap(), (8 << 20).try_into().unwrap()),
    )])))
    .candidate(
        BTreeMap::new(),
        Arc::new(pse_schema::catalog::assemble().unwrap()),
        &cancel,
    )
    .unwrap();
    let error = session
        .prepare(plan(None, None, 1.0).unwrap(), &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("requires the ipopt build capability"),
        "{error}"
    );
}

fn scenario() -> datafusion::logical_expr::Expr {
    let schema = pse_relations::generated::runtime::numerical_evaluations::schema().unwrap();
    let field = schema.field_with_name("scenario_id").unwrap();
    datafusion::logical_expr::Expr::Literal(
        datafusion::common::ScalarValue::FixedSizeBinary(16, Some(vec![1; 16])),
        Some(field.metadata().into()),
    )
    .alias("scenario_id")
}
