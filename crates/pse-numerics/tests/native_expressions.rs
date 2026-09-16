// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Analytic, finite-difference and failure oracles for reusable native numerics.
#![allow(clippy::unwrap_used, reason = "independent numerical test assertions")]
use datafusion::{
    arrow::{
        array::{Float64Array, RecordBatch},
        datatypes::{DataType, Field, Schema, SchemaRef},
    },
    common::Column,
    execution::session_state::{SessionState, SessionStateBuilder},
    functions::math::expr_fn::{exp, ln, power, sin, sqrt},
    logical_expr::{ColumnarValue, Expr, Volatility, col, create_udf, lit, when},
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_numerics::{EvaluationProgram, NumericsError};
use std::sync::Arc;

fn state() -> SessionState {
    SessionStateBuilder::new_with_default_features().build()
}
fn schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("x", DataType::Float64, false),
        Field::new("y", DataType::Float64, false),
    ]))
}
fn values(x: Vec<f64>, y: Vec<f64>) -> RecordBatch {
    RecordBatch::try_new(
        schema(),
        vec![
            Arc::new(Float64Array::from(x)),
            Arc::new(Float64Array::from(y)),
        ],
    )
    .unwrap()
}
fn value(batch: &RecordBatch, column: usize, row: usize) -> f64 {
    batch
        .column(column)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap()
        .value(row)
}
fn program(expressions: &[Expr]) -> EvaluationProgram {
    EvaluationProgram::compile(
        expressions,
        &[Column::from_name("x"), Column::from_name("y")],
        schema(),
        &state(),
        FixedBudget::new(128 << 20),
        CancellationToken::new(),
    )
    .unwrap()
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-9 * (1.0 + expected.abs()),
        "{actual} != {expected}"
    );
}

#[test]
fn nonlinear_residuals_and_sparse_jacobians_match_independent_oracles() {
    // f = x^3 + sin(y) - 7; g = exp(x)/y. Deliberately non-linear and cross-coupled.
    let program = program(&[
        power(col("x"), lit(3.0_f64)) + sin(col("y")) - lit(7.0_f64),
        exp(col("x")) / col("y"),
    ]);
    assert_eq!(program.residual_count(), 2);
    assert_eq!(
        program.jacobian_coordinates(),
        &[(0, 0), (0, 1), (1, 0), (1, 1)]
    );
    for (x, y) in [(1.2_f64, 2.3_f64), (-1.4, 0.7), (0.0, 1.0)] {
        let result = program.evaluate(&values(vec![x], vec![y])).unwrap();
        close(value(&result, 0, 0), x.powi(3) + y.sin() - 7.0);
        close(value(&result, 1, 0), x.exp() / y);
        for (column, expected) in [3.0 * x * x, y.cos(), x.exp() / y, -x.exp() / (y * y)]
            .into_iter()
            .enumerate()
        {
            close(value(&result, column + 2, 0), expected);
        }
        let h = 1e-5;
        let left = program.evaluate(&values(vec![x - h], vec![y])).unwrap();
        let right = program.evaluate(&values(vec![x + h], vec![y])).unwrap();
        close(
            value(&result, 2, 0),
            (value(&right, 0, 0) - value(&left, 0, 0)) / (2.0 * h),
        );
        close(
            value(&result, 4, 0),
            (value(&right, 1, 0) - value(&left, 1, 0)) / (2.0 * h),
        );
    }
}

#[test]
fn native_batches_keep_order_and_omit_structurally_absent_derivatives() {
    let program = program(&[ln(col("x")), lit(8.0_f64)]);
    assert_eq!(program.jacobian_coordinates(), &[(0, 0)]);
    let output = program
        .evaluate(&values(vec![1.0, 2.0, 4.0], vec![0.0; 3]))
        .unwrap();
    assert_eq!(output.num_rows(), 3);
    for (row, x) in [1.0_f64, 2.0, 4.0].into_iter().enumerate() {
        close(value(&output, 0, row), x.ln());
        close(value(&output, 1, row), 8.0);
        close(value(&output, 2, row), 1.0 / x);
    }
}

#[test]
fn every_supported_unary_binding_and_variable_power_has_a_derivative_oracle() {
    use datafusion::functions::math::expr_fn as f;
    let x = 0.4_f64;
    let cases = [
        (exp(col("x")), x.exp()),
        (ln(col("x")), 1.0 / x),
        (f::log10(col("x")), 1.0 / (x * std::f64::consts::LN_10)),
        (sqrt(col("x")), 1.0 / (2.0 * x.sqrt())),
        (sin(col("x")), x.cos()),
        (f::cos(col("x")), -x.sin()),
        (f::tan(col("x")), 1.0 / x.cos().powi(2)),
        (f::asin(col("x")), 1.0 / (1.0 - x * x).sqrt()),
        (f::acos(col("x")), -1.0 / (1.0 - x * x).sqrt()),
        (f::atan(col("x")), 1.0 / (1.0 + x * x)),
        (f::sinh(col("x")), x.cosh()),
        (f::cosh(col("x")), x.sinh()),
        (f::tanh(col("x")), 1.0 / x.cosh().powi(2)),
        (f::abs(col("x")), 1.0),
    ];
    for (expression, derivative) in cases {
        let program = program(&[expression]);
        let output = program.evaluate(&values(vec![x], vec![1.0])).unwrap();
        close(value(&output, 1, 0), derivative);
        let h = 1e-5;
        let a = program.evaluate(&values(vec![x - h], vec![1.0])).unwrap();
        let b = program.evaluate(&values(vec![x + h], vec![1.0])).unwrap();
        close(
            value(&output, 1, 0),
            (value(&b, 0, 0) - value(&a, 0, 0)) / (2.0 * h),
        );
    }
    let output = program(&[power(col("x"), col("y"))])
        .evaluate(&values(vec![2.0], vec![3.0]))
        .unwrap();
    close(value(&output, 0, 0), 8.0);
    close(value(&output, 1, 0), 12.0);
    close(value(&output, 2, 0), 8.0 * 2.0_f64.ln());
}

#[test]
fn inactive_native_case_branch_is_not_evaluated_but_nonfinite_intermediates_fail() {
    let selected = when(lit(true), power(col("x"), lit(2.0_f64)))
        .otherwise(sqrt(-col("y")))
        .unwrap();
    let output = program(&[selected])
        .evaluate(&values(vec![3.0], vec![1.0]))
        .unwrap();
    close(value(&output, 0, 0), 9.0);
    close(value(&output, 1, 0), 6.0);
    close(value(&output, 2, 0), 0.0);
    let hidden_overflow = program(&[lit(1.0_f64) / exp(col("x"))]);
    assert!(matches!(
        hidden_overflow.evaluate(&values(vec![1000.0], vec![1.0])),
        Err(NumericsError::Evaluate(_))
    ));
    assert!(
        program(&[sqrt(col("x"))])
            .evaluate(&values(vec![0.0], vec![1.0]))
            .is_err(),
        "singular derivative is not an accepted finite gradient"
    );
}

#[test]
fn derivative_admission_uses_actual_udf_and_refuses_variable_dependent_guards() {
    let impostor = create_udf(
        "exp",
        vec![DataType::Float64],
        DataType::Float64,
        Volatility::Immutable,
        Arc::new(|args: &[ColumnarValue]| Ok(args[0].clone())),
    );
    assert!(matches!(
        EvaluationProgram::compile(
            &[impostor.call(vec![col("x")])],
            &[Column::from_name("x")],
            schema(),
            &state(),
            FixedBudget::new(1 << 20),
            CancellationToken::new()
        ),
        Err(NumericsError::Unsupported { .. })
    ));
    let switch = when(col("x").gt(lit(0.0_f64)), col("x"))
        .otherwise(-col("x"))
        .unwrap();
    assert!(matches!(
        EvaluationProgram::compile(
            &[switch],
            &[Column::from_name("x")],
            schema(),
            &state(),
            FixedBudget::new(1 << 20),
            CancellationToken::new()
        ),
        Err(NumericsError::Unsupported { .. })
    ));
}

#[test]
fn cancellation_resource_refusal_and_buffer_lifetimes_are_explicit() {
    let cancel = CancellationToken::new();
    let budget = FixedBudget::new(1 << 20);
    let program = EvaluationProgram::compile(
        &[col("x") * col("x")],
        &[Column::from_name("x")],
        schema(),
        &state(),
        budget.clone(),
        cancel.clone(),
    )
    .unwrap();
    let result = program.evaluate(&values(vec![2.0], vec![3.0])).unwrap();
    let retained = result.column(0).clone();
    cancel.cancel();
    assert!(matches!(
        program.evaluate(&values(vec![2.0], vec![3.0])),
        Err(NumericsError::Resource(_))
    ));
    drop((program, result));
    assert!(budget.reserved() > 0);
    drop(retained);
    assert_eq!(budget.reserved(), 0);
    assert!(matches!(
        EvaluationProgram::compile(
            &[exp(col("x"))],
            &[Column::from_name("x")],
            schema(),
            &state(),
            FixedBudget::new(1),
            CancellationToken::new()
        ),
        Err(NumericsError::Resource(_))
    ));
}

#[test]
fn native_resource_errors_remain_resource_failures() {
    let refused = create_udf(
        "resource_refusal",
        vec![DataType::Float64],
        DataType::Float64,
        Volatility::Immutable,
        Arc::new(|_: &[ColumnarValue]| {
            Err(datafusion::common::DataFusionError::ResourcesExhausted(
                "test consumer".into(),
            ))
        }),
    );
    // Fixed parameter functions need no derivative, but their failure remains real.
    let program = EvaluationProgram::compile(
        &[refused.call(vec![col("y")])],
        &[Column::from_name("x")],
        schema(),
        &state(),
        FixedBudget::new(1 << 20),
        CancellationToken::new(),
    )
    .unwrap();
    assert!(matches!(
        program.evaluate(&values(vec![1.0], vec![2.0])),
        Err(NumericsError::NativeResource(_))
    ));
}

#[test]
fn fixed_native_transforms_need_no_derivative_binding_or_numeric_argument_restriction() {
    let length = datafusion::logical_expr::expr_fn::cast(
        datafusion::functions::unicode::expr_fn::character_length(lit("material")),
        DataType::Float64,
    );
    let output = program(&[col("x") * length, power(col("x"), lit(0i64))])
        .evaluate(&values(vec![0.0], vec![1.0]))
        .unwrap();
    close(value(&output, 0, 0), 0.0);
    close(value(&output, 1, 0), 1.0);
    close(value(&output, 2, 0), 8.0);
}

#[tokio::test]
async fn the_same_numerical_expressions_execute_as_an_ordinary_native_projection() {
    let state = state();
    let input = values(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
    let program = EvaluationProgram::compile(
        &[col("x") * col("y")],
        &[Column::from_name("x"), Column::from_name("y")],
        schema(),
        &state,
        FixedBudget::new(1 << 20),
        CancellationToken::new(),
    )
    .unwrap();
    let expected = program.evaluate(&input).unwrap();
    let context = datafusion::execution::context::SessionContext::new_with_state(state);
    let child = context.read_batch(input).unwrap().into_unoptimized_plan();
    let plan = program.logical_projection(child).unwrap();
    assert!(
        plan.display_indent()
            .to_string()
            .contains("pse_finite_numerical_value")
    );
    drop(program);
    let actual = context
        .execute_logical_plan(plan)
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(actual, [expected]);
}
