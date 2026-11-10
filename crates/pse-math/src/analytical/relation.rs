// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit bounded native integer-relation hypotheses, independently verified when algebraic.
use super::*;
use symbolica::domains::{
    Ring,
    algebraic::AlgebraicContext,
    float::{Float, RealLike, SingleFloat},
    integer::{Integer, IntegerRelationError},
};

pse_contract::projection_struct! {
/// Opt-in research controls. A tolerance hit is not an exact identity proof.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RelationRequest {
    projection:{name:"AnalyticalRelationRequest",rust_path:crate::analytical::RelationRequest,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Actual binary precision of the native numerical realization.
    pub precision:u32 => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Exact positive absolute numerical tolerance, smaller than one.
    pub tolerance:RationalView => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Explicit finite native iteration bound.
    pub max_iterations:usize => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Explicit absolute coefficient bound.
    pub max_coefficient:u64 => {tag:4,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
}
/// Actual native stopping reason, independently of any later exact verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationStop {
    /// Native numerical tolerance was met.
    Tolerance,
    /// The finite iteration bound exhausted; the candidate remains a hypothesis.
    IterationLimit,
    /// The actual precision could not continue native reconstruction.
    PrecisionLimit,
    /// The configured coefficient bound prevented a candidate.
    CoefficientLimit,
}
/// Proof strength, separate from native search stopping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationMeaning {
    /// Native algebraic embedding of the original exact expressions established zero.
    ExactAlgebraicZero,
    /// A numerical candidate without an exact verification proof.
    NumericalHypothesis,
    /// Native search supplied no candidate.
    Unavailable,
}
/// Owned research observation. Coefficients never enter executable modeling authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RelationObservation {
    /// Actual original research controls.
    pub request: RelationRequest,
    /// Actual native stopping reason.
    pub stop: RelationStop,
    /// Independently established mathematical strength.
    pub meaning: RelationMeaning,
    /// Original selected exact or rounded source expressions, in coefficient order.
    pub sources: Vec<MathematicalView>,
    /// Native realized values at the requested binary precision.
    pub numerical_values: Vec<String>,
    /// Candidate exact integers, including an exhaustion candidate when available.
    pub candidate: Option<Vec<String>>,
    /// Native numerical residual at the same precision; never an interval enclosure.
    pub residual: Option<String>,
    /// Native original linear combination, with original root choices retained.
    pub exact_combination: Option<MathematicalView>,
    /// Native algebraic normalization, present only when original constants embed.
    pub algebraic_normal_form: Option<MathematicalView>,
    /// Exact verification was outside the bounded radical profile or refused by
    /// the supplier. This never changes the actual numerical search result.
    pub verification_unavailable: Option<String>,
}
impl SelectedExpression {
    /// Search a bounded relation among original closed selected outputs. Exact
    /// algebraic verification is a separate native operation over those originals.
    pub fn integer_relation(
        &self,
        others: &[&Self],
        request: &RelationRequest,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        check(scope)?;
        if others.is_empty()
            || others.len() >= 16
            || request.precision < 64
            || request.precision > 4096
            || request.max_iterations == 0
            || request.max_iterations > limits.operations
            || request.max_coefficient == 0
        {
            return Err(MathError::Limit(
                "bounded research integer-relation profile",
            ));
        }
        if request
            .tolerance
            .numerator
            .len()
            .saturating_add(request.tolerance.denominator.len())
            > limits.scratch_bytes.min(4096)
        {
            return Err(MathError::Limit("research tolerance extent"));
        }
        let numerator = request
            .tolerance
            .numerator
            .parse::<Integer>()
            .map_err(|error| MathError::Contract(error.into()))?;
        let denominator = request
            .tolerance
            .denominator
            .parse::<Integer>()
            .map_err(|error| MathError::Contract(error.into()))?;
        if denominator.is_zero() {
            return Err(MathError::Contract(
                "zero research tolerance denominator".into(),
            ));
        }
        let tolerance = Rational::new(numerator, denominator);
        if tolerance.is_negative() || tolerance.is_zero() || tolerance >= 1 {
            return Err(MathError::Contract(
                "research tolerance must lie strictly between zero and one".into(),
            ));
        }
        let expressions = std::iter::once(self)
            .chain(others.iter().copied())
            .map(|source| &source.expression)
            .collect::<Vec<_>>();
        let mut values = Vec::<Float>::new();
        let mut sources = Vec::new();
        for expression in &expressions {
            bounded(expression, limits, scope)?;
            let value = expression
                .evaluate_with_prec(
                    &std::collections::HashMap::<Atom, Float>::new(),
                    request.precision,
                )
                .map_err(|error| MathError::Library(error.to_string()))?;
            if !value.is_finite() || value.is_zero() {
                return Err(MathError::Contract(
                    "research relation requires finite nonzero closed source values".into(),
                ));
            }
            // Keep native initial Hermite integer conversion inside the finite
            // research profile; finite MPFR values can have enormous exponents.
            let magnitude = RealLike::to_f64(&value).abs();
            if !(1e-32..=1e32).contains(&magnitude) {
                return Err(MathError::Limit("research realized magnitude profile"));
            }
            values.push(value);
            sources.push(mathematical_view(expression, limits, scope)?);
        }
        let backing = values
            .len()
            .checked_mul(values.len())
            .and_then(|value| value.checked_mul(request.precision as usize / 8 + 256))
            .and_then(|value| value.checked_mul(128))
            .filter(|bytes| *bytes <= limits.scratch_bytes)
            .ok_or(MathError::Limit("research native matrix backing"))?;
        crate::construction::additional(backing)?;
        let mut stopped = None;
        let mut polls = 0usize;
        let search = Integer::solve_integer_relation_with(
            &values,
            tolerance.to_multi_prec_float(request.precision),
            request.max_iterations,
            Some(Integer::from(request.max_coefficient)),
            None,
            || {
                polls = polls.saturating_add(1);
                let observation = if polls > limits.operations {
                    Err(MathError::Limit("research native work"))
                } else {
                    check(scope)
                };
                match observation {
                    Ok(()) => true,
                    Err(error) => {
                        stopped = Some(error);
                        false
                    }
                }
            },
        );
        if let Some(error) = stopped {
            return Err(error);
        }
        let (stop, candidate) = match search {
            Ok(candidate) => (RelationStop::Tolerance, Some(candidate)),
            Err(IntegerRelationError::IterationLimit(candidate)) => {
                (RelationStop::IterationLimit, Some(candidate))
            }
            Err(IntegerRelationError::PrecisionLimit) => (RelationStop::PrecisionLimit, None),
            Err(IntegerRelationError::CoefficientLimit) => (RelationStop::CoefficientLimit, None),
            Err(IntegerRelationError::Interrupted) => {
                return Err(MathError::Contract(
                    "native research interruption lost original control cause".into(),
                ));
            }
        };
        let mut expression = Atom::Zero;
        let mut residual = None;
        let mut exact_combination = None;
        let mut algebraic_normal_form = None;
        let mut verification_unavailable = None;
        let mut meaning = if candidate.is_some() {
            RelationMeaning::NumericalHypothesis
        } else {
            RelationMeaning::Unavailable
        };
        if let Some(candidate) = &candidate {
            let mut sum = Float::new(request.precision);
            for ((coefficient, original), value) in candidate.iter().zip(&expressions).zip(&values)
            {
                check(scope)?;
                expression += Atom::num(coefficient.clone()) * (*original);
                sum += value.clone()
                    * Rational::from(coefficient.clone()).to_multi_prec_float(request.precision);
            }
            residual = Some(sum.to_string());
            exact_combination = Some(mathematical_view(&expression, limits, scope)?);
            // A rounded source must never turn its numerical coincidence into an
            // algebraic proof by treating the rounded coefficient as exact.
            let mut rounded = false;
            for original in &expressions {
                original.visitor(&mut |node|{rounded|=matches!(node,AtomView::Num(number) if matches!(number.get_coeff_view(),symbolica::coefficient::CoefficientView::Float(..)));true});
            }
            if rounded {
                verification_unavailable =
                    Some("original source contains rounded coefficients".into());
            } else if !algebraic_profile(&expressions, limits, scope)? {
                verification_unavailable = Some(
                    "original source exceeds bounded exact radical verification profile".into(),
                );
            } else {
                check(scope)?;
                if let Ok(mut context) = AlgebraicContext::from_atom(expression.as_view())
                    && let Ok(value) = context.convert_atom(expression.as_view())
                {
                    if context.field().is_zero(&value) {
                        meaning = RelationMeaning::ExactAlgebraicZero;
                    }
                    algebraic_normal_form = Some(mathematical_view(
                        &context.field().element_to_atom_simplified(&value),
                        limits,
                        scope,
                    )?);
                } else {
                    verification_unavailable =
                        Some("native algebraic embedding refused the original expression".into());
                }
                check(scope)?;
            }
        }
        let observation = RelationObservation {
            request: request.clone(),
            stop,
            meaning,
            sources,
            numerical_values: values.iter().map(ToString::to_string).collect(),
            candidate: candidate
                .as_ref()
                .map(|candidate| candidate.iter().map(ToString::to_string).collect()),
            residual,
            exact_combination,
            algebraic_normal_form,
            verification_unavailable,
        };
        bounded(&expression, limits, scope)?;
        Ok(AnalyticalResult {
            observation: AnalyticalObservation::IntegerRelation(observation),
            expression,
        })
    }
}

// A finite candidate gate for the indivisible native algebraic embedding. This
// selects the research profile; the supplier alone proves the exact relation.
// At most four independent square roots (or equivalent degree16 radical tower),
// small rational radicands, bounded integer powers and128-bit exact coefficients.
// Wider algebraic research needs a controlled native extension entry first.
fn algebraic_profile(
    expressions: &[&Atom],
    limits: EvaluationLimits,
    scope: &ExecutionScope,
) -> Result<bool, MathError> {
    fn visit(
        atom: AtomView<'_>,
        radicals: &mut std::collections::HashSet<Atom>,
        degree: &mut usize,
        nodes: &mut usize,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<bool, MathError> {
        check(scope)?;
        *nodes = nodes.saturating_add(1);
        if *nodes > limits.operations.min(128) {
            return Ok(false);
        }
        match atom {
            AtomView::Num(_) => Ok(Rational::try_from(atom).is_ok_and(|value| {
                crate::rational_binary64::bits(&value).is_ok_and(|bits| bits <= 128)
            })),
            AtomView::Add(add) => {
                for value in add.iter() {
                    if !visit(value, radicals, degree, nodes, limits, scope)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            AtomView::Mul(mul) => {
                for value in mul.iter() {
                    if !visit(value, radicals, degree, nodes, limits, scope)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            AtomView::Pow(power) => {
                let Ok(exponent) = Rational::try_from(power.get_exp()) else {
                    return Ok(false);
                };
                let Some(numerator) = exponent.numerator_ref().to_i64() else {
                    return Ok(false);
                };
                let Some(denominator) = exponent.denominator_ref().to_i64() else {
                    return Ok(false);
                };
                if numerator.unsigned_abs() > 16 || !(1..=8).contains(&denominator) {
                    return Ok(false);
                }
                if denominator > 1 && radicals.insert(atom.to_owned()) {
                    *degree = degree.saturating_mul(denominator as usize);
                    if *degree > 16 {
                        return Ok(false);
                    }
                    let Ok(radicand) = Rational::try_from(power.get_base()) else {
                        return Ok(false);
                    };
                    if crate::rational_binary64::bits(&radicand)? > 20 {
                        return Ok(false);
                    }
                }
                visit(power.get_base(), radicals, degree, nodes, limits, scope)
            }
            _ => Ok(false),
        }
    }
    let mut radicals = std::collections::HashSet::new();
    let mut degree = 1;
    let mut nodes = 0;
    for expression in expressions {
        if !visit(
            expression.as_view(),
            &mut radicals,
            &mut degree,
            &mut nodes,
            limits,
            scope,
        )? {
            return Ok(false);
        }
    }
    let bytes = degree
        .checked_mul(degree)
        .and_then(|value| value.checked_mul(128))
        .and_then(|value| value.checked_mul(16))
        .filter(|bytes| *bytes <= limits.scratch_bytes)
        .ok_or(MathError::Limit("research algebraic verification backing"))?;
    crate::construction::additional(bytes)?;
    Ok(true)
}
