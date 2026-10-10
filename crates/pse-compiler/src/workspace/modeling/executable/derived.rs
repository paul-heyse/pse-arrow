// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Parameters a bound case structure determines (ADR-0104): hull and linear bounds from
//! finite variable bounds, and derived big-Ms from the library's FBBT enclosure of each
//! disjunct row over the case box, extended to zero and widened outward.
//!
//! The rules and the enclosure program depend on the structure only and are prepared with
//! the view. Their values are values: they depend on the fixed and parameter values the
//! derivation consumed, so a value rebind recomputes them only when one of those changed,
//! and the value-dependent products see them like any other parameter (A6).
use super::*;
use pse_modeling::{ModelingError, RealizationRefusal, specialize::DerivedRule};
use std::borrow::Cow;

/// How one derived parameter is obtained under a bound structure.
#[derive(Clone, Debug)]
enum Rule {
    /// A free variable's finite case bound, fixed by the structure.
    Bound(f64),
    /// The value of a fixed variable or parameter.
    Value(SemanticId),
    /// One side of the enclosure of row `row` of the enclosure program.
    Extremum {
        /// Row index in the enclosure program.
        row: usize,
        /// Supremum rather than infimum.
        upper: bool,
        /// Relative outward widening.
        margin: f64,
    },
}
/// One derived parameter with its refusal, resolved from the model at preparation.
#[derive(Clone, Debug)]
struct Parameter {
    id: SemanticId,
    rule: Rule,
    /// The typed realization refusal naming the rule's subject; its reason is set on use.
    refusal: ModelingError,
}
impl Parameter {
    fn refuse(&self, reason: RealizationRefusal) -> CompileError {
        let mut error = self.refusal.clone();
        if let ModelingError::Realization { reason: slot, .. } = &mut error {
            *slot = reason;
        }
        error.into()
    }
}
/// The value-order program of the derived big-M rows over the case box.
#[derive(Clone, Debug)]
struct Enclosure {
    plan: Arc<CasePlan>,
    /// Free-column box of the program; a semi domain's box includes its zero branch.
    lower: Vec<f64>,
    upper: Vec<f64>,
    /// Identities the program binds; no other value is among its inputs.
    scope: BTreeSet<SemanticId>,
}
/// The derived-parameter rules of one bound structure, shared by every value rebind.
#[derive(Clone, Debug, Default)]
pub struct Derivation {
    parameters: Vec<Parameter>,
    enclosure: Option<Enclosure>,
}
/// Derived parameter values under one binding, with the values they consumed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Derived {
    /// Values in canonical units, by parameter.
    pub values: BTreeMap<SemanticId, f64>,
    /// Fixed and parameter values the derivation consumed, bitwise.
    consumed: BTreeMap<SemanticId, u64>,
}
impl Derived {
    /// Whether every value the derivation consumed is unchanged in `values`; derived
    /// parameters present in `values` are not among them.
    pub fn matches(&self, values: &CaseValues) -> bool {
        self.consumed
            .iter()
            .all(|(id, bits)| values.scalars.get(id).map(|v| v.to_bits()) == Some(*bits))
    }
    /// `values` with the derived parameters of this binding; stale derived entries in
    /// `values` are replaced.
    pub fn complete<'a>(&self, values: &'a CaseValues) -> Cow<'a, CaseValues> {
        if self.values.is_empty() {
            return Cow::Borrowed(values);
        }
        let mut completed = values.clone();
        completed
            .scalars
            .extend(self.values.iter().map(|(id, v)| (*id, *v)));
        Cow::Owned(completed)
    }
    /// Known retained payload.
    pub fn retained_bytes(&self) -> usize {
        (self.values.len() + self.consumed.len()) * (size_of::<(SemanticId, u64)>() + 32)
    }
}
impl Derivation {
    /// Enclosure rebuilding has a distinct projection lifetime/population.
    pub(super) fn has_enclosure(&self) -> bool {
        self.enclosure.is_some()
    }
    /// Resolve the derived parameters of `model` against a bound `structure`: a free
    /// variable's case bound is structural and must be finite; a fixed variable's value and
    /// every big-M enclosure are taken per binding ([`Self::derive`]).
    ///
    /// # Errors
    /// A free variable without the finite bound a lowering needs, an unknown derived row,
    /// or a failed enclosure program.
    pub(super) fn new(
        model: &PreparedModeling,
        structure: &CaseStructure,
        quantities: &QuantityRegistry,
        assembly: AssemblyLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self> {
        let variables = structure.variables();
        let mut parameters = Vec::new();
        let mut extrema = Vec::new();
        for (id, parameter) in &model.model.derived {
            match parameter.rule {
                DerivedRule::Bound { variable, upper } => {
                    let refusal = model.model.realization_refusal(
                        *id,
                        variable,
                        RealizationRefusal::InfiniteBound,
                    );
                    let rule = match variables.iter().find(|v| v.port.id == variable) {
                        Some(v) if !v.fixed => Rule::Bound(
                            if upper { v.upper } else { v.lower }
                                .filter(|b| b.is_finite())
                                .ok_or_else(|| CompileError::from(refusal.clone()))?,
                        ),
                        _ => Rule::Value(variable),
                    };
                    parameters.push(Parameter {
                        id: *id,
                        rule,
                        refusal,
                    });
                }
                DerivedRule::Extremum {
                    expression,
                    upper,
                    margin,
                } => extrema.push((*id, expression, upper, margin)),
            }
        }
        if extrema.is_empty() {
            return Ok(Self {
                parameters,
                enclosure: None,
            });
        }
        let rows = extrema
            .iter()
            .map(|(_, expression, _, _)| ModelingOutput::Member(*expression).row_id())
            .collect::<BTreeSet<_>>();
        let observed = model.observation_structure_over(&rows, variables)?;
        let scope = observed
            .parameters()
            .iter()
            .map(|p| p.id)
            .chain(observed.variables().iter().map(|v| v.port.id))
            .collect::<BTreeSet<_>>();
        let plan = Arc::new(CasePlan::prepare(
            observed,
            model
                .admitted
                .bodies
                .iter()
                .map(|(k, b)| (*k, b.math.clone()))
                .collect(),
            quantities,
            DerivativeOrder::Value,
            assembly,
            cancel,
        )?);
        let (lower, upper) = plan
            .columns()
            .iter()
            .map(|id| column_box(plan.structure().variables(), *id))
            .unzip();
        for (id, expression, upper, margin) in extrema {
            let row = ModelingOutput::Member(expression).row_id();
            let row = plan
                .structure()
                .rows()
                .iter()
                .position(|r| r.id == row)
                .ok_or_else(|| CompileError::Missing("derived big-M row".into()))?;
            parameters.push(Parameter {
                id,
                rule: Rule::Extremum { row, upper, margin },
                refusal: model.model.realization_refusal(
                    id,
                    expression,
                    RealizationRefusal::IncompleteInterval,
                ),
            });
        }
        Ok(Self {
            parameters,
            enclosure: Some(Enclosure {
                plan,
                lower,
                upper,
                scope,
            }),
        })
    }
    /// The derived parameter values under `values`, with the values they consumed.
    ///
    /// # Errors
    /// A missing or nonfinite fixed bound source, a row whose enclosure is not
    /// FBBT-complete or not finite over the box, a failed projection or cancellation.
    pub fn derive(&self, values: &CaseValues, cancel: &Arc<AtomicBool>) -> Result<Derived> {
        let mut derived = Derived::default();
        let facts = match &self.enclosure {
            Some(e) => {
                let scoped = CaseValues {
                    scalars: values
                        .scalars
                        .iter()
                        .filter(|(id, _)| e.scope.contains(id))
                        .map(|(id, v)| (*id, *v))
                        .collect(),
                };
                let facts = e.plan.presolve_facts(&scoped, 100_000, cancel)?;
                derived
                    .consumed
                    .extend(facts.values.iter().map(|(id, b)| (*id, *b)));
                Some((e, facts))
            }
            None => None,
        };
        for p in &self.parameters {
            let value = match p.rule {
                Rule::Bound(value) => value,
                Rule::Value(source) => {
                    let value = values
                        .scalars
                        .get(&source)
                        .copied()
                        .filter(|v| v.is_finite())
                        .ok_or_else(|| p.refuse(RealizationRefusal::InfiniteBound))?;
                    derived.consumed.insert(source, value.to_bits());
                    value
                }
                Rule::Extremum { row, upper, margin } => {
                    let (e, facts) = facts
                        .as_ref()
                        .ok_or_else(|| CompileError::Missing("derived big-M program".into()))?;
                    let (lo, hi) = facts
                        .row_enclosure(row, &e.lower, &e.upper)?
                        .ok_or_else(|| p.refuse(RealizationRefusal::IncompleteInterval))?;
                    widened(lo, hi, upper, margin)
                        .ok_or_else(|| p.refuse(RealizationRefusal::UnboundedInterval))?
                }
            };
            derived.values.insert(p.id, value);
        }
        Ok(derived)
    }
    /// Known retained payload; the enclosure program is shared by every rebind.
    pub fn retained_bytes(&self) -> usize {
        self.parameters.capacity() * (size_of::<Parameter>() + 64)
            + self.enclosure.as_ref().map_or(0, |e| {
                e.plan.retained_bytes()
                    + (e.lower.capacity() + e.upper.capacity()) * size_of::<f64>()
                    + e.scope.len() * (size_of::<SemanticId>() + 32)
            })
    }
}
/// The box of one enclosure column; a semi domain's zero branch lies outside its active
/// interval.
fn column_box(variables: pse_math::binding::VariableView<'_>, id: SemanticId) -> (f64, f64) {
    let v = variables.iter().find(|v| v.port.id == id);
    let l = v.and_then(|v| v.lower).unwrap_or(f64::NEG_INFINITY);
    let u = v.and_then(|v| v.upper).unwrap_or(f64::INFINITY);
    if v.is_some_and(|v| v.domain.is_semi()) {
        (l.min(0.0), u.max(0.0))
    } else {
        (l, u)
    }
}
/// One side of a derived big-M from the enclosure `[lo, hi]`, or `None` when that side is
/// not finite. The relaxation must admit the zero residual of an inactive row, and widening
/// is outward only, so a derived M never cuts a feasible point (T15). A side within the
/// enclosure's own rounding resolution of zero is zero, never a subnormal; any other side
/// of the already outward-rounded library enclosure is widened by the margin and one ULP.
fn widened(lo: f64, hi: f64, upper: bool, margin: f64) -> Option<f64> {
    let value = if upper { hi.max(0.0) } else { lo.min(0.0) };
    if !value.is_finite() {
        return None;
    }
    let resolution = [lo, hi]
        .into_iter()
        .filter(|v| v.is_finite())
        .fold(0.0_f64, |a, v| a.max(v.abs()))
        * f64::EPSILON;
    if value.abs() <= resolution {
        return Some(0.0);
    }
    let widened = value + value.abs() * margin * if upper { 1.0 } else { -1.0 };
    Some(if upper {
        widened.next_up()
    } else {
        widened.next_down()
    })
}
