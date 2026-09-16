// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Sparse symbolic differentiation of actual native expression bindings.
use crate::{NumericsError, error::unsupported};
use datafusion::{
    common::{
        Column, ScalarValue,
        tree_node::{TreeNode, TreeNodeRecursion},
    },
    functions::math::{self, expr_fn as math_expr},
    logical_expr::{
        Expr, Operator, Volatility,
        expr::{Case, ScalarFunction},
        lit, when,
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct Jet {
    pub(crate) value: Expr,
    pub(crate) gradient: BTreeMap<usize, Expr>,
}

pub(crate) fn compile(
    expr: &Expr,
    variables: &BTreeMap<Column, usize>,
    depth: usize,
) -> Result<Jet, NumericsError> {
    if depth > 64 {
        return Err(unsupported("numerical expression exceeds depth 64"));
    }
    if expr
        .column_refs()
        .iter()
        .all(|column| !variables.contains_key(*column))
    {
        let mut immutable = true;
        expr.apply(|node| {
            if let Expr::ScalarFunction(function) = node {
                immutable &= function.func.signature().volatility == Volatility::Immutable;
            }
            Ok(TreeNodeRecursion::Continue)
        })
        .map_err(NumericsError::preparation)?;
        if !immutable {
            return Err(unsupported("fixed numerical expressions must be immutable"));
        }
        // Arbitrary native parameter transformations need no derivative rule.
        // Native physical preparation still checks types and executable support.
        return Ok(Jet {
            value: expr.clone(),
            gradient: BTreeMap::new(),
        });
    }
    let next = |expr: &Expr| compile(expr, variables, depth + 1);
    let gradient = match expr {
        Expr::Alias(alias) => return next(&alias.expr),
        Expr::Literal(ScalarValue::Float64(Some(value)), _) if value.is_finite() => BTreeMap::new(),
        Expr::Column(column) => variables
            .get(column)
            .map(|index| (*index, lit(1.0_f64)))
            .into_iter()
            .collect(),
        Expr::Negative(child) => next(child)?
            .gradient
            .into_iter()
            .map(|(index, value)| (index, -value))
            .collect(),
        Expr::BinaryExpr(binary) => {
            let left = next(&binary.left)?;
            let right = next(&binary.right)?;
            binary_gradient(&left, &right, binary.op)?
        }
        Expr::ScalarFunction(function) => {
            let args = function
                .args
                .iter()
                .map(next)
                .collect::<Result<Vec<_>, _>>()?;
            function_gradient(function, &args)?
        }
        Expr::Case(case) => return case_gradient(case, variables, depth),
        other => {
            return Err(unsupported(format!(
                "{} requires an explicit numerical lowering",
                other.variant_name()
            )));
        }
    };
    Ok(Jet {
        value: expr.clone(),
        gradient,
    })
}

fn binary_gradient(
    left: &Jet,
    right: &Jet,
    op: Operator,
) -> Result<BTreeMap<usize, Expr>, NumericsError> {
    if !matches!(
        op,
        Operator::Plus | Operator::Minus | Operator::Multiply | Operator::Divide
    ) {
        return Err(unsupported(format!("derivative of native operator {op}")));
    }
    let keys = left
        .gradient
        .keys()
        .chain(right.gradient.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    Ok(keys
        .into_iter()
        .filter_map(|key| {
            let a = left.gradient.get(&key).cloned();
            let b = right.gradient.get(&key).cloned();
            let (a, b) = match op {
                Operator::Multiply => (
                    a.map(|d| d * right.value.clone()),
                    b.map(|d| left.value.clone() * d),
                ),
                Operator::Divide => (
                    a.map(|d| d / right.value.clone()),
                    b.map(|d| {
                        (left.value.clone() * d) / (right.value.clone() * right.value.clone())
                    }),
                ),
                _ => (a, b),
            };
            let subtract = matches!(op, Operator::Minus | Operator::Divide);
            combine(a, b, subtract).map(|value| (key, value))
        })
        .collect())
}
fn combine(a: Option<Expr>, b: Option<Expr>, subtract: bool) -> Option<Expr> {
    match (a, b, subtract) {
        (Some(a), Some(b), false) => Some(a + b),
        (Some(a), Some(b), true) => Some(a - b),
        (Some(a), None, _) => Some(a),
        (None, Some(b), false) => Some(b),
        (None, Some(b), true) => Some(-b),
        (None, None, _) => None,
    }
}
fn function_gradient(
    function: &ScalarFunction,
    args: &[Jet],
) -> Result<BTreeMap<usize, Expr>, NumericsError> {
    if function.func.signature().volatility != Volatility::Immutable {
        return Err(unsupported("numerical functions must be immutable"));
    }
    // Parameter-only functions need no derivative implementation. They still own
    // their real UDF, execute normally, and undergo finite-value checks.
    if args.iter().all(|arg| arg.gradient.is_empty()) {
        return Ok(BTreeMap::new());
    }
    if function.func.as_ref() == math::power().as_ref() {
        return power_gradient(args);
    }
    let [arg] = args else {
        return Err(unsupported(format!(
            "derivative binding for {}",
            function.func.name()
        )));
    };
    let local = unary_derivative(function, arg.value.clone())?;
    Ok(arg
        .gradient
        .iter()
        .map(|(index, value)| (*index, local.clone() * value.clone()))
        .collect())
}
fn unary_derivative(function: &ScalarFunction, x: Expr) -> Result<Expr, NumericsError> {
    let actual = function.func.as_ref();
    let one = || lit(1.0_f64);
    let square = || x.clone() * x.clone();
    let value = if actual == math::exp().as_ref() {
        math_expr::exp(x)
    } else if actual == math::ln().as_ref() {
        one() / x
    } else if actual == math::log10().as_ref() {
        one() / (x * lit(std::f64::consts::LN_10))
    } else if actual == math::sqrt().as_ref() {
        one() / (lit(2.0_f64) * math_expr::sqrt(x))
    } else if actual == math::sin().as_ref() {
        math_expr::cos(x)
    } else if actual == math::cos().as_ref() {
        -math_expr::sin(x)
    } else if actual == math::tan().as_ref() {
        one() / (math_expr::cos(x.clone()) * math_expr::cos(x))
    } else if actual == math::asin().as_ref() {
        one() / math_expr::sqrt(one() - square())
    } else if actual == math::acos().as_ref() {
        -(one() / math_expr::sqrt(one() - square()))
    } else if actual == math::atan().as_ref() {
        one() / (one() + square())
    } else if actual == math::sinh().as_ref() {
        math_expr::cosh(x)
    } else if actual == math::cosh().as_ref() {
        math_expr::sinh(x)
    } else if actual == math::tanh().as_ref() {
        one() - (math_expr::tanh(x.clone()) * math_expr::tanh(x))
    } else if actual == math::abs().as_ref() {
        when(x.clone().gt(lit(0.0_f64)), one())
            .when(x.lt(lit(0.0_f64)), lit(-1.0_f64))
            .otherwise(lit(f64::NAN))
            .map_err(NumericsError::preparation)?
    } else {
        return Err(unsupported(format!(
            "derivative binding for actual function {}",
            function.func.name()
        )));
    };
    Ok(value)
}
fn power_gradient(args: &[Jet]) -> Result<BTreeMap<usize, Expr>, NumericsError> {
    let [base, exponent] = args else {
        return Err(unsupported("power arity"));
    };
    let constant = match exponent.value.as_literal().and_then(|value| {
        value
            .cast_to(&datafusion::arrow::datatypes::DataType::Float64)
            .ok()
    }) {
        Some(ScalarValue::Float64(value)) => value,
        _ => None,
    };
    let keys = base
        .gradient
        .keys()
        .chain(exponent.gradient.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    Ok(keys
        .into_iter()
        .filter_map(|index| {
            let left = base.gradient.get(&index).and_then(|d| {
                if constant == Some(0.0) {
                    None
                } else if constant == Some(1.0) {
                    Some(d.clone())
                } else {
                    Some(
                        (exponent.value.clone()
                            * math_expr::power(
                                base.value.clone(),
                                exponent.value.clone() - lit(1.0_f64),
                            ))
                            * d.clone(),
                    )
                }
            });
            let right = exponent.gradient.get(&index).map(|d| {
                (math_expr::power(base.value.clone(), exponent.value.clone())
                    * math_expr::ln(base.value.clone()))
                    * d.clone()
            });
            combine(left, right, false).map(|value| (index, value))
        })
        .collect())
}
fn case_gradient(
    case: &Case,
    variables: &BTreeMap<Column, usize>,
    depth: usize,
) -> Result<Jet, NumericsError> {
    if case.expr.is_some() || case.else_expr.is_none() {
        return Err(unsupported(
            "numerical CASE requires Boolean guards and an explicit ELSE",
        ));
    }
    let mut branches = vec![];
    for (guard, value) in &case.when_then_expr {
        guard
            .apply(|node| {
                if let Expr::ScalarFunction(function) = node
                    && function.func.signature().volatility != Volatility::Immutable
                {
                    return Err(datafusion::common::DataFusionError::NotImplemented(
                        "numerical CASE guards must be immutable".into(),
                    ));
                }
                Ok(TreeNodeRecursion::Continue)
            })
            .map_err(NumericsError::preparation)?;
        if guard
            .column_refs()
            .iter()
            .any(|column| variables.contains_key(*column))
        {
            return Err(unsupported(
                "variable-dependent CASE needs an explicit nonsmooth derivative policy",
            ));
        }
        branches.push((guard.clone(), compile(value, variables, depth + 1)?));
    }
    let fallback = compile(
        case.else_expr
            .as_ref()
            .ok_or_else(|| unsupported("CASE ELSE absent"))?,
        variables,
        depth + 1,
    )?;
    let mut keys = fallback.gradient.keys().copied().collect::<BTreeSet<_>>();
    for (_, branch) in &branches {
        keys.extend(branch.gradient.keys().copied());
    }
    let gradient = keys
        .into_iter()
        .map(|index| {
            let expr = Expr::Case(Case {
                expr: None,
                when_then_expr: branches
                    .iter()
                    .map(|(guard, branch)| {
                        (
                            guard.clone(),
                            Box::new(
                                branch
                                    .gradient
                                    .get(&index)
                                    .cloned()
                                    .unwrap_or_else(|| lit(0.0_f64)),
                            ),
                        )
                    })
                    .collect(),
                else_expr: Some(Box::new(
                    fallback
                        .gradient
                        .get(&index)
                        .cloned()
                        .unwrap_or_else(|| lit(0.0_f64)),
                )),
            });
            (index, expr)
        })
        .collect();
    Ok(Jet {
        value: Expr::Case(case.clone()),
        gradient,
    })
}
