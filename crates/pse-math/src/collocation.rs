// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Orthogonal collocation using Symbolica's exact polynomials and certified real roots.
use crate::MathError;
use symbolica::{
    domains::rational::{Q, Rational},
    poly::univariate::UnivariatePolynomial,
};

/// Maximum supported element order bounds exact polynomial and root-isolation work.
pub const MAX_ORDER: usize = 16;
/// Dimensionless element data consumed by finite realization.
#[derive(Clone, Debug, PartialEq)]
pub struct Stencil {
    /// Includes both element boundaries.
    pub nodes: Vec<f64>,
    /// Derivative rows at collocation nodes only.
    pub derivative: Vec<Vec<(usize, f64)>>,
    /// Integrals of cardinal functions on the collocation nodes (left boundary excluded).
    pub integral: Vec<f64>,
    /// Legendre's right boundary interpolation from the left boundary and interior nodes.
    pub endpoint: Option<Vec<f64>>,
}
/// Construct one bounded element. Root-finding, polynomial arithmetic, differentiation,
/// and integration are library-owned. No Newton iteration or coefficient tables are stored.
/// # Errors
/// Uninitialized mathematics, unsupported order, invalid root counts or nonfinite weights.
pub fn element(order: usize, alpha: f64, beta: f64, right: bool) -> Result<Stencil, MathError> {
    if !(1..=MAX_ORDER).contains(&order) {
        return Err(MathError::Limit("collocation order 1..=16"));
    }
    let formal = crate::library::formal(0)?;
    let symbolica::atom::AtomView::Var(v) = formal.as_view() else {
        return Err(MathError::Contract("formal must be a symbol".into()));
    };
    let variable = std::sync::Arc::new(symbolica::poly::PolyVariable::Symbol(v.get_symbol()));
    let poly = |c: Vec<Rational>| UnivariatePolynomial::from_coefficients(&Q, c, variable.clone());
    let one = poly(vec![1.into()]);
    let mut nodes = vec![0.0];
    nodes.extend(jacobi_roots(order - usize::from(right), alpha, beta)?);
    nodes.push(1.0);
    if nodes.windows(2).any(|v| !v[1].is_finite() || v[0] >= v[1]) {
        return Err(MathError::Library("unordered collocation nodes".into()));
    }
    let rational = nodes
        .iter()
        .map(|v| Rational::try_from(*v).map_err(|e| MathError::Library(e.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    let cardinal = |indices: &[usize], j: usize| {
        let mut p = one.clone();
        for k in indices.iter().copied().filter(|k| *k != j) {
            p = (p * &poly(vec![-rational[k].clone(), 1.into()]))
                .div_coeff(&(&rational[j] - &rational[k]));
        }
        p
    };
    let coordinates = (0..=order).collect::<Vec<_>>();
    let collocation = (1..=order).collect::<Vec<_>>();
    let mut result = Stencil {
        derivative: vec![vec![]; nodes.len()],
        integral: vec![0.0; nodes.len()],
        nodes,
        endpoint: None,
    };
    let mut endpoint = vec![];
    for j in &coordinates {
        let p = cardinal(&coordinates, *j);
        let derivative = p.derivative();
        for i in &collocation {
            result.derivative[*i].push((*j, derivative.evaluate(&rational[*i]).to_f64()));
        }
        endpoint.push(p.evaluate(&Rational::from(1)).to_f64());
    }
    for j in &collocation {
        let primitive = cardinal(&collocation, *j).integrate();
        result.integral[*j] = (&primitive.evaluate(&Rational::from(1))
            - &primitive.evaluate(&Rational::from(0)))
            .to_f64();
    }
    if !right {
        result.endpoint = Some(endpoint);
    }
    if result
        .derivative
        .iter()
        .flatten()
        .any(|(_, v)| !v.is_finite())
        || result.integral.iter().any(|v| !v.is_finite())
        || result.endpoint.iter().flatten().any(|v| !v.is_finite())
    {
        return Err(MathError::Library(
            "nonfinite collocation coefficients".into(),
        ));
    }
    Ok(result)
}

/// Roots of the shifted Jacobi polynomial `P_n^(alpha,beta)(2*x-1)`.
/// The family parameters are caller data, not a named-method dispatch. Exact
/// polynomial construction follows NIST DLMF 18.5.8 (<https://dlmf.nist.gov/18.5.E8>);
/// Symbolica owns polynomial arithmetic and certified real-root isolation.
/// # Errors
/// Orders above the resource bound, parameters outside the orthogonality domain,
/// or roots that cannot be represented as distinct interior floating coordinates.
pub fn jacobi_roots(order: usize, alpha: f64, beta: f64) -> Result<Vec<f64>, MathError> {
    if order > MAX_ORDER {
        return Err(MathError::Limit("Jacobi order 0..=16"));
    }
    if !alpha.is_finite() || !beta.is_finite() || alpha <= -1.0 || beta <= -1.0 {
        return Err(MathError::Contract(
            "Jacobi parameters must be finite and greater than -1".into(),
        ));
    }
    if order == 0 {
        return Ok(Vec::new());
    }
    let alpha = Rational::try_from(alpha).map_err(|e| MathError::Library(e.to_string()))?;
    let beta = Rational::try_from(beta).map_err(|e| MathError::Library(e.to_string()))?;
    let formal = crate::library::formal(0)?;
    let symbolica::atom::AtomView::Var(v) = formal.as_view() else {
        return Err(MathError::Contract("formal must be a symbol".into()));
    };
    let variable = std::sync::Arc::new(symbolica::poly::PolyVariable::Symbol(v.get_symbol()));
    let poly = |c| UnivariatePolynomial::from_coefficients(&Q, c, variable.clone());
    let x = poly(vec![0.into(), 1.into()]);
    let shifted = poly(vec![(-1).into(), 1.into()]);
    let binomial = |value: &Rational, n: usize| {
        (0..n).fold(Rational::from(1), |result, k| {
            result * (value - &Rational::from(k as i64)) / Rational::from((k + 1) as i64)
        })
    };
    let n = Rational::from(order as i64);
    let mut polynomial = poly(vec![0.into()]);
    for k in 0..=order {
        let coefficient = binomial(&(&n + &alpha), k) * binomial(&(&n + &beta), order - k);
        polynomial = polynomial + (shifted.pow(order - k) * &x.pow(k)).mul_coeff(&coefficient);
    }
    let roots = polynomial.isolate_real_roots();
    if roots.len() != order || roots.iter().any(|(_, m)| *m != 1) {
        return Err(MathError::Library(
            "Jacobi polynomial requires simple real roots".into(),
        ));
    }
    let tolerance = Rational::from((1_i64, 1_i64 << 54));
    let mut nodes = roots
        .into_iter()
        .map(|(root, _)| root.refined(&tolerance).enclosure().center().re.to_f64())
        .collect::<Vec<_>>();
    nodes.sort_by(f64::total_cmp);
    if nodes
        .iter()
        .any(|v| !v.is_finite() || *v <= 0.0 || *v >= 1.0)
        || nodes.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(MathError::Library(
            "Jacobi roots are not distinct interior coordinates".into(),
        ));
    }
    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jacobi_root_parameters_are_data_and_domains_are_checked() {
        crate::initialize().unwrap();
        let p = jacobi_roots(1, 0.25, 1.5).unwrap();
        assert!((p[0] - 2.5 / 3.75).abs() < 1e-14);
        let p = jacobi_roots(2, 0.0, 0.0).unwrap();
        let d = (1.0_f64 / 3.0).sqrt();
        assert!((p[0] - (1.0 - d) / 2.0).abs() < 1e-14);
        assert!((p[1] - (1.0 + d) / 2.0).abs() < 1e-14);
        assert!(jacobi_roots(0, 0.0, 0.0).unwrap().is_empty());
        for (n, a, b) in [(17, 0.0, 0.0), (1, -1.0, 0.0), (1, 0.0, f64::NAN)] {
            assert!(jacobi_roots(n, a, b).is_err());
        }
    }
    #[test]
    fn collocation_differentiates_and_integrates_polynomials() {
        crate::initialize().unwrap();
        for (alpha, beta, right) in [(1.0, 0.0, true), (0.0, 0.0, false)] {
            for order in 1..=5 {
                let s = element(order, alpha, beta, right).unwrap();
                for degree in 0..=order {
                    for (i, row) in s
                        .derivative
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| !r.is_empty())
                    {
                        let value = row
                            .iter()
                            .map(|(j, w)| w * s.nodes[*j].powi(degree as i32))
                            .sum::<f64>();
                        let expected = if degree == 0 {
                            0.0
                        } else {
                            degree as f64 * s.nodes[i].powi(degree as i32 - 1)
                        };
                        assert!(
                            (value - expected).abs() < 1e-10,
                            "{alpha} {beta} {right} {order} {degree}: {value} != {expected}"
                        );
                    }
                }
                let exact_degree = if right { 2 * order - 2 } else { 2 * order - 1 };
                for degree in 0..=exact_degree {
                    let value = s
                        .nodes
                        .iter()
                        .zip(&s.integral)
                        .map(|(x, w)| w * x.powi(degree as i32))
                        .sum::<f64>();
                    assert!((value - 1.0 / (degree + 1) as f64).abs() < 1e-12);
                }
                if let Some(weights) = s.endpoint {
                    for degree in 0..=order {
                        assert!(
                            (weights
                                .iter()
                                .zip(&s.nodes)
                                .map(|(w, x)| w * x.powi(degree as i32))
                                .sum::<f64>()
                                - 1.0)
                                .abs()
                                < 1e-10
                        );
                    }
                }
            }
        }
    }
}
