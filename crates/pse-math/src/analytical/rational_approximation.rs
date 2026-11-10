// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native formal Padé products; no executable approximation or interval certificate.
use super::*;
use std::sync::Arc;
use symbolica::{
    coefficient::CoefficientView,
    domains::{
        integer::Z,
        rational::Q,
        rational_polynomial::{RationalPolynomial, RationalPolynomialField},
    },
    poly::{PolyVariable, polynomial::MultivariatePolynomial},
};

impl SelectedExpression {
    /// Construct a bounded formal Padé product from the original exact expansion.
    /// The returned denominator is a retained mathematical condition; this does
    /// not authorize replacing the original executable output.
    pub fn rational_approximation(
        &self,
        coordinate: usize,
        point: &Atom,
        numerator_degree: u32,
        denominator_degree: u32,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        let degree = numerator_degree
            .checked_add(denominator_degree)
            .ok_or(MathError::Limit("Padé total degree"))?;
        if degree as usize > limits.derivative_components {
            return Err(MathError::Limit("Padé total degree"));
        }
        bounded(&self.expression, limits, scope)?;
        bounded(point, limits, scope)?;
        let axis = self.axis(coordinate)?;
        if point.contains(axis.as_view()) {
            return Err(MathError::Contract(
                "Padé expansion point depends on its coordinate".into(),
            ));
        }
        self.check_exact_point(&axis, point, limits, scope)?;
        let series = controlled_series(
            &self.expression,
            &axis,
            point,
            SeriesDepth::absolute(degree),
            limits,
            scope,
        )?;
        let order = degree
            .checked_add(1)
            .ok_or(MathError::Limit("Padé congruence order"))?;
        if !self.expression.is_zero()
            && matches!(series.precision(),SeriesPrecision::Finite{absolute,..} if absolute<Rational::from(order))
        {
            return Err(MathError::Contract(
                "native series does not cover the Padé congruence order".into(),
            ));
        }
        let field = RationalPolynomialField::<_, u32>::new(Z);
        let variables = Arc::new(vec![PolyVariable::Symbol(
            axis.get_symbol()
                .ok_or_else(|| MathError::Contract("Padé coordinate".into()))?,
        )]);
        let mut polynomial =
            MultivariatePolynomial::<_, u32>::new(&field, Some(order as usize), variables);
        for (exponent, coefficient) in series.terms() {
            check(scope)?;
            if !exponent.denominator_ref().is_one() || exponent.is_negative() {
                return Err(MathError::Contract(
                    "Padé profile requires a regular integral-power series".into(),
                ));
            }
            let exponent = exponent
                .numerator_ref()
                .to_string()
                .parse::<u32>()
                .map_err(|_| MathError::Limit("Padé exponent"))?;
            if exponent >= order {
                continue;
            }
            bounded(coefficient, limits, scope)?;
            let mut floating = false;
            coefficient.visitor(&mut |node|{if matches!(node,AtomView::Num(number) if matches!(number.get_coeff_view(),CoefficientView::Float(..))){floating=true;}true});
            if floating {
                return Err(MathError::Contract(
                    "Padé exact profile refuses floating coefficients".into(),
                ));
            }
            let coefficient: RationalPolynomial<_, u32> = coefficient
                .try_to_rational_polynomial(&Q, &Z, None)
                .map_err(|error| MathError::Library(error.to_string()))?;
            polynomial.append_monomial(coefficient, &[exponent]);
        }
        check(scope)?;
        let (numerator, denominator) = polynomial
            .rational_approximant_univariate_with(
                numerator_degree,
                denominator_degree,
                &mut operation_control(limits, scope),
            )?
            .ok_or_else(|| {
                MathError::Contract(
                    "native Padé construction found no requested approximant".into(),
                )
            })?;
        if numerator.degree(0) > numerator_degree
            || denominator.degree(0) > denominator_degree
            || denominator.is_zero()
            || !(&(&denominator * &polynomial) - &numerator)
                .mod_var(0, order)
                .is_zero()
        {
            return Err(MathError::Contract(
                "native Padé output did not satisfy the requested degree and formal congruence"
                    .into(),
            ));
        }
        let shift = &axis - point;
        let numerator = numerator
            .to_expression_with_coeff_map(|_, value, out| *out = value.to_expression())
            .replace(axis.clone())
            .with(shift.clone());
        let denominator = denominator
            .to_expression_with_coeff_map(|_, value, out| *out = value.to_expression())
            .replace(axis.clone())
            .with(shift);
        // Clear coefficient-field denominators in the native rational abstraction.
        // Otherwise a common scale can display removable parameter poles (e.g.
        // powers of the linear coefficient for exp(a*x)) that the product lacks.
        let normalized: RationalPolynomial<_, u32> = (numerator / denominator)
            .try_to_rational_polynomial(&Q, &Z, None)
            .map_err(|error| MathError::Library(error.to_string()))?;
        let numerator = normalized.numerator.to_expression();
        let denominator = normalized.denominator.to_expression();
        let observation = AnalyticalObservation::RationalApproximation {
            coordinate,
            expansion_point: mathematical_view(point, limits, scope)?,
            numerator_degree,
            denominator_degree,
            numerator: mathematical_view(&numerator, limits, scope)?,
            denominator: mathematical_view(&denominator, limits, scope)?,
            formal_order: (&Rational::from(order)).into(),
        };
        let expression = numerator / denominator;
        bounded(&expression, limits, scope)?;
        Ok(AnalyticalResult {
            observation,
            expression,
        })
    }
}
