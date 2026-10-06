// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded exact affine correspondence for SCIP's actual rational optimum.
use super::*;
use pse_math::{
    binding::ObjectiveSense,
    factorable::{Constant, FactorableProgram, Fidelity, Node, ProjectedVariable, Rational},
    implicit::ProofInterval,
};

const COLUMNS: usize = 16;
const ROWS: usize = 32;
const NODES: usize = 256;
const BITS: u64 = 4096;

#[derive(Clone, Debug, PartialEq)]
struct Upload {
    id: SemanticId,
    terms: Vec<(usize, f64)>,
    constant: f64,
    lower: f64,
    upper: f64,
}
/// A producer-issued exact Optimal receipt, never an accuracy claim by itself.
/// Its private upload inventory must match the original exact-real Value program.
#[derive(Clone, Debug, PartialEq)]
pub struct ExactObjectiveTransport {
    structure: ContentHash,
    values: BTreeMap<SemanticId, u64>,
    variables: Vec<ProjectedVariable>,
    rows: Vec<Upload>,
    objective: Vec<(usize, f64)>,
    constant: f64,
    sense: ObjectiveSense,
    point: Vec<u64>,
    optimum: String,
    external_offset: f64,
    validity: ContentHash,
}
fn bounded(value: Rational) -> Option<Rational> {
    (value.numerator_ref().significant_bits() <= BITS
        && value.denominator_ref().significant_bits() <= BITS)
        .then_some(value)
}
fn binary(value: f64) -> Option<Rational> {
    Rational::try_from(value).ok().and_then(bounded)
}
fn parse(text: &str) -> Option<Rational> {
    if text.is_empty() || text.len() > 255 {
        return None;
    }
    let (num, den) = text.split_once('/').unwrap_or((text, "1"));
    let template = Rational::from(0);
    let mut numerator = template.numerator();
    let mut denominator = template.denominator();
    numerator.clone_from(&num.parse().ok()?);
    denominator.clone_from(&den.parse().ok()?);
    if denominator <= template.numerator() {
        return None;
    }
    bounded(Rational::from((numerator, denominator)))
}
fn interval(value: &Rational) -> Option<ProofInterval> {
    let center = value.to_f64();
    let exact = binary(center)?;
    let lower = if exact > *value {
        center.next_down()
    } else {
        center
    };
    let upper = if exact < *value {
        center.next_up()
    } else {
        center
    };
    (lower.is_finite() && upper.is_finite() && binary(lower)? <= *value && binary(upper)? >= *value)
        .then_some(ProofInterval { lower, upper })
}
// This checks finite affine upload arithmetic using library rationals. It is not
// differentiation, an interval evaluator, or a solver. Non-affine nodes refuse.
fn forms(
    program: &FactorableProgram,
    execution: &Execution,
) -> Result<Option<Vec<Vec<Rational>>>, ProblemError> {
    let n = program.variables.len();
    if n == 0 || n > COLUMNS || program.rows.len() > ROWS || program.nodes.len() > NODES {
        return Ok(None);
    }
    let mut forms: Vec<Vec<Rational>> = Vec::with_capacity(program.nodes.len());
    for node in &program.nodes {
        execution.check()?;
        let mut form = vec![Rational::from(0); n + 1];
        match node {
            Node::Var(j) if *j < n => form[*j] = Rational::from(1),
            Node::Const(value) => {
                let value = match value {
                    Constant::Rational(q) => bounded(q.clone()),
                    Constant::Float(v) => binary(*v),
                };
                let Some(value) = value else {
                    return Ok(None);
                };
                form[n] = value;
            }
            Node::Sum(children) => {
                for child in children {
                    let Some(source) = forms.get(*child) else {
                        return Ok(None);
                    };
                    for (target, source) in form.iter_mut().zip(source) {
                        let Some(sum) = bounded(&*target + source) else {
                            return Ok(None);
                        };
                        *target = sum;
                    }
                }
            }
            Node::Product(children) => {
                form[n] = Rational::from(1);
                for child in children {
                    let Some(source) = forms.get(*child) else {
                        return Ok(None);
                    };
                    let form_constant = form[..n].iter().all(Rational::is_zero);
                    let source_constant = source[..n].iter().all(Rational::is_zero);
                    if !form_constant && !source_constant {
                        return Ok(None);
                    }
                    let (scalar, multiplicand) = if form_constant {
                        (&form[n], source)
                    } else {
                        (&source[n], &form)
                    };
                    let Some(product) = multiplicand
                        .iter()
                        .map(|q| bounded(q * scalar))
                        .collect::<Option<Vec<_>>>()
                    else {
                        return Ok(None);
                    };
                    form = product;
                }
            }
            _ => return Ok(None),
        }
        forms.push(form);
    }
    Ok(Some(forms))
}
fn agrees(form: &[Rational], terms: &[(usize, f64)], constant: f64) -> bool {
    let Some((last, coefficients)) = form.split_last() else {
        return false;
    };
    if binary(constant).as_ref() != Some(last) {
        return false;
    }
    let mut upload = vec![Rational::from(0); coefficients.len()];
    for (j, value) in terms {
        let (Some(target), Some(value)) = (upload.get_mut(*j), binary(*value)) else {
            return false;
        };
        let Some(sum) = bounded(&*target + &value) else {
            return false;
        };
        *target = sum;
    }
    coefficients == upload
}
impl ExactObjectiveTransport {
    #[cfg(feature = "scip")]
    pub(crate) fn issue(
        plan: &crate::execution::factorable::Plan<'_>,
        point: &[f64],
        optimum: &str,
        external_offset: f64,
    ) -> Option<Self> {
        let p = plan.program;
        if p.variables.is_empty()
            || p.variables.len() > COLUMNS
            || p.rows.len() > ROWS
            || p.nodes.len() > NODES
            || p.values.len() > NODES
            || !p.obligations.is_empty()
            || !p.auxiliaries.is_empty()
            || !p.native.is_empty()
            || !p.implicit.is_empty()
            || !plan.semi.is_empty()
            || plan.fidelity != Fidelity::Exact
            || plan.constraints.len() != p.rows.len()
            || point.len() != p.variables.len()
            || point.iter().any(|v| !v.is_finite())
            || p.variables.iter().zip(&plan.boxes).any(|(v, b)| {
                let expected =
                    if v.domain == pse_model::generated::enums::ModelingVariableDomain::Binary {
                        (v.lower.max(0.), v.upper.min(1.))
                    } else {
                        (v.lower, v.upper)
                    };
                expected != *b
            })
        {
            return None;
        }
        let (node, sense) = plan.objective?;
        let objective = plan.affine.get(node)?.as_ref()?;
        let mut rows = Vec::with_capacity(p.rows.len());
        for (row, constraint) in p.rows.iter().zip(&plan.constraints) {
            if constraint.condition.is_some()
                || row.expression
                    != match constraint.expression {
                        crate::execution::factorable::Expression::Node(node) => Some(node),
                        _ => None,
                    }
                || row.lower != constraint.lower
                || row.upper != constraint.upper
            {
                return None;
            }
            let f = plan.form(constraint.expression)?;
            // SCIP receives sides after binary64 subtraction. Demand exact transport.
            for bound in [row.lower, row.upper].into_iter().filter(|b| b.is_finite()) {
                if binary(bound - f.constant)? != bounded(binary(bound)? - binary(f.constant)?)? {
                    return None;
                }
            }
            rows.push(Upload {
                id: row.id,
                terms: f.terms.clone(),
                constant: f.constant,
                lower: row.lower,
                upper: row.upper,
            });
        }
        let total = bounded(parse(optimum)? + binary(external_offset)?)?;
        interval(&total)?;
        let mut validity = pse_ids::FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        validity
            .str("scip-exact-optimal-affine-upload")
            .hash(&p.key)
            .str(optimum)
            .u64(external_offset.to_bits());
        for v in point {
            validity.u64(v.to_bits());
        }
        Some(Self {
            structure: p.structure,
            values: p.values.clone(),
            variables: p.variables.clone(),
            rows,
            objective: objective.terms.clone(),
            constant: objective.constant,
            sense,
            point: point.iter().map(|v| v.to_bits()).collect(),
            optimum: optimum.into(),
            external_offset,
            validity: validity.finish_hash(),
        })
    }
    /// Finite scratch allowance for exact coefficient correspondence, not evaluator work.
    pub fn correspondence_bytes(&self) -> usize {
        NODES * (self.variables.len() + 1) * 4096
    }
    /// Native objective sense actually proved.
    pub fn sense(&self) -> ObjectiveSense {
        self.sense
    }
    /// Receipt identity, supplemented by the actual original exact-real program.
    pub fn validity(&self) -> ContentHash {
        self.validity
    }
    /// Verify the exact original coefficient, domain, bound and fixed-value transport.
    /// # Errors
    /// Actual cancellation/deadline failure; unsupported correspondence returns None.
    pub fn original_interval(
        &self,
        original: &FactorableProgram,
        point: &[f64],
        execution: &Execution,
    ) -> Result<Option<ProofInterval>, ProblemError> {
        execution.check()?;
        if original.structure != self.structure
            || original.values != self.values
            || original.variables != self.variables
            || original.rows.len() != self.rows.len()
            || point
                .iter()
                .map(|v| v.to_bits())
                .ne(self.point.iter().copied())
            || !original.obligations.is_empty()
            || !original.native.is_empty()
            || !original.auxiliaries.is_empty()
            || !original.implicit.is_empty()
            || original.fidelity() != Fidelity::Exact
        {
            return Ok(None);
        }
        let Some(forms) = forms(original, execution)? else {
            return Ok(None);
        };
        for (row, upload) in original.rows.iter().zip(&self.rows) {
            let Some(form) = row.expression.and_then(|node| forms.get(node)) else {
                return Ok(None);
            };
            if row.id != upload.id
                || row.lower != upload.lower
                || row.upper != upload.upper
                || !agrees(form, &upload.terms, upload.constant)
            {
                return Ok(None);
            }
        }
        let Some(objective) = &original.objective else {
            return Ok(None);
        };
        let Some(form) = objective.expression.and_then(|node| forms.get(node)) else {
            return Ok(None);
        };
        if objective.sense != self.sense || !agrees(form, &self.objective, self.constant) {
            return Ok(None);
        }
        Ok(parse(&self.optimum)
            .and_then(|q| bounded(q + binary(self.external_offset)?))
            .and_then(|q| interval(&q)))
    }
}
