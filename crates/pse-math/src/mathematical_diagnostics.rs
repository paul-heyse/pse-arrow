// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected exact blocks and claim-scoped native numerical realizations.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::CaseValues,
    guarded::{Condition, Stage},
    jets::EvaluationLimits,
    library::{self, Optimization},
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::ExecutionScope;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    coefficient::CoefficientView,
    domains::{
        float::{
            DoubleFloat, ErrorPropagatingFloat, Float, FloatLike, Real, RealBall, SingleFloat,
        },
        integer::{Integer, Z},
        rational::Rational,
        rational_polynomial::{RationalPolynomial, RationalPolynomialField},
    },
    evaluate::{OperationError, OperationExtent},
    tensors::matrix::{Matrix, RowReductionOperation},
};

pse_contract::projection_enum! {
/// Distinct claims; estimates never acquire enclosure fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticClaim {
    projection: { name: "DiagnosticClaim", rust_path: crate::mathematical_diagnostics::DiagnosticClaim, version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Ordinary binary64 realization without a new accuracy certificate.
    Ordinary => 1;
    /// Native double-float estimate at its actual 106-bit domain precision.
    DoubleFloat => 2;
    /// Multiprecision estimate at the explicitly selected binary precision.
    ChosenPrecision => 3;
    /// Native propagated error estimate, without containment certification.
    PropagatedError => 4;
    /// Exact rational-function structure under explicit original conditions.
    ExactIdentity => 5;
    /// Directed basic arithmetic enclosing the complete requested real input set.
    ValidatedEnclosure => 6;
}
}

pse_contract::projection_struct! {
/// Exact rational transport. Denominators must be nonzero; admission bounds digits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExactValue {
    projection: { name: "ExactValue", rust_path: crate::mathematical_diagnostics::ExactValue, version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };

    /// Exact signed integer numerator in decimal notation.
    pub numerator: String => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Exact nonzero integer denominator in decimal notation.
    pub denominator: String => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl ExactValue {
    /// Construct an exact integer point.
    pub fn integer(value: i64) -> Self {
        Self {
            numerator: value.to_string(),
            denominator: "1".into(),
        }
    }
    fn rational(&self, limits: DiagnosticLimits) -> Result<Rational, MathError> {
        let digits = limits.coefficient_bits / 3 + 4;
        if self.numerator.len() > digits
            || self.denominator.len() > digits
            || self.numerator.len().saturating_add(self.denominator.len()) > limits.bytes
        {
            return Err(MathError::Limit("diagnostic rational input"));
        }
        let numerator = self
            .numerator
            .parse::<Integer>()
            .map_err(|e| MathError::Contract(e.into()))?;
        let denominator = self
            .denominator
            .parse::<Integer>()
            .map_err(|e| MathError::Contract(e.into()))?;
        if denominator.is_zero() {
            return Err(MathError::Contract("zero diagnostic denominator".into()));
        }
        let value = Rational::new(numerator, denominator);
        if crate::rational_binary64::bits(&value)? > limits.coefficient_bits {
            return Err(MathError::Limit("diagnostic input bits"));
        }
        Ok(value)
    }
}

pse_contract::projection_struct! {
/// A point when `upper` is absent; otherwise an exact closed input interval.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticInput {
    projection: { name: "DiagnosticInput", rust_path: crate::mathematical_diagnostics::DiagnosticInput, version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };

    /// Original scalar coordinate identity.
    pub coordinate: SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Exact lower endpoint, or the sole point value.
    pub lower: ExactValue => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Optional exact upper endpoint; absence denotes a point.
    pub upper: Option<ExactValue> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
/// Mathematical workload limits, independent of host capacity placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticLimits {
    projection: { name: "DiagnosticLimits", rust_path: crate::mathematical_diagnostics::DiagnosticLimits, version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };

    /// Maximum admitted selected original row count.
    pub rows: usize => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
    /// Maximum admitted original unknown and parameter count.
    pub coordinates: usize => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
    /// Maximum admitted native polynomial or evaluator population.
    pub terms: usize => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
    /// Maximum native work population for an admitted pass.
    pub work: usize => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
    /// Maximum mathematical construction backing for this request.
    pub bytes: usize => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
    /// Maximum admitted exact coefficient or requested precision extent.
    pub coefficient_bits: usize => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
}
}
impl Default for DiagnosticLimits {
    fn default() -> Self {
        Self {
            rows: 32,
            coordinates: 32,
            terms: 100_000,
            work: 1_000_000,
            bytes: 64 * 1024 * 1024,
            coefficient_bits: 65_536,
        }
    }
}

pse_contract::projection_struct! {
/// Ordered original demand, with symbolic parameters retained until specialization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticRequest {
    projection: { name: "DiagnosticRequest", rust_path: crate::mathematical_diagnostics::DiagnosticRequest, version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };

    /// Ordered original selected row identities.
    pub rows: Vec<SemanticId> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Ordered coordinates solved by exact block inspection.
    pub unknowns: Vec<SemanticId> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Ordered symbolic parameters, retained until an explicit specialization.
    pub parameters: Vec<SemanticId> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Exact points or closed intervals bound to original coordinate identities.
    pub inputs: Vec<DiagnosticInput> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Actual mathematical assurance requested or delivered.
    pub claim: DiagnosticClaim => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Requested or actual binary arithmetic precision.
    pub precision: u32 => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
    /// Explicit mathematical workload admission.
    pub limits: DiagnosticLimits => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Ordered first/second derivative coordinate sequences; empty requests values only.
    #[serde(default)]
    pub derivatives: Vec<Vec<SemanticId>> => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

/// Compiler-issued proof of the original source arithmetic profile, before normalization.
/// This capability is deliberately neither serializable nor reconstructible from a result.
#[derive(Debug)]
pub struct ArithmeticSourceAdmission {
    structure: ContentHash,
    rows: Vec<SemanticId>,
}
impl ArithmeticSourceAdmission {
    /// Issue only after the compiler checks every selected original source operation.
    /// # Safety
    /// The issuer must establish exact real rational constants and basic arithmetic/
    /// integer powers only, before normalization, with no branch, provider callback,
    /// transcendental or rounded external constant. Identity and row order must refer
    /// to precisely the admitted original product, never decoded caller assertions.
    #[allow(
        unsafe_code,
        reason = "ADR-0265: compiler-only original arithmetic source admission capability"
    )]
    pub unsafe fn issue(structure: ContentHash, rows: Vec<SemanticId>) -> Self {
        Self { structure, rows }
    }
}

/// Exact consistency of the selected block under its declared conditions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExactConsistency {
    /// Full column rank with a consistent right-hand side.
    Unique,
    /// Consistent with free coordinates; no arbitrary canonical solution is asserted.
    Underdetermined,
    /// The augmented rank exceeds the coefficient rank.
    Inconsistent,
}

/// Source-linked native row-combination witness; entries multiply original rows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RowWitness {
    /// Original rows multiplied by this native row-combination witness.
    pub original_rows: Vec<SemanticId>,
    /// Exact row-combination coefficients in original row order.
    pub multipliers: Vec<String>,
    /// Whether this row combination proves a nonzero inconsistent right-hand side.
    pub inconsistent: bool,
}

/// Exact matrix projection. Generic claims apply only under all listed conditions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExactBlockResult {
    /// Exact coefficient rows in requested unknown order.
    pub matrix: Vec<Vec<String>>,
    /// Exact original extracted right-hand side.
    pub right_hand_side: Vec<String>,
    /// Exact coefficient matrix rank under all listed conditions.
    pub rank: usize,

    /// Exact rank after adjoining the right-hand side.
    pub augmented_rank: usize,
    /// Native exact consistency classification under all listed conditions.
    pub consistency: ExactConsistency,

    /// Sufficient nonzero pivot and denominator conditions for this result.
    pub nonzero_conditions: Vec<String>,
    /// Native row-reduced coefficient rows.
    pub reduced_matrix: Vec<Vec<String>>,

    /// Native row-reduced right-hand side.
    pub reduced_right_hand_side: Vec<String>,
    /// Source-linked redundant or inconsistent row combinations.
    pub row_witnesses: Vec<RowWitness>,

    /// Original unknown identities and exact recovered coordinate expressions.
    pub recovered: Vec<(SemanticId, String)>,
    /// Original-domain decision when exact specialization permits it.
    pub original_domain_valid: Option<bool>,

    /// Declared physical coordinate bounds decision when established.
    pub physical_bounds_valid: Option<bool>,
    /// Recovered coordinates satisfy original residual, domain and bound validation.
    pub reduced_evaluation: bool,
    /// This specialization consumed the retained generic recovery product.
    pub used_reduced_product: bool,
}

/// One numerical result. Exact rational endpoint strings preserve outward rounding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct NumericalDiagnosticValue {
    /// Numerical estimate text, absent for an enclosure.
    pub value: Option<String>,
    /// Propagated absolute error estimate, never an enclosure radius.
    pub absolute_error: Option<f64>,

    /// Exact lower endpoint, or the sole point value.
    pub lower: Option<String>,
    /// Optional exact upper endpoint; absence denotes a point.
    pub upper: Option<String>,
    /// The supported enclosure arithmetic produced a nonfinite ball.
    pub unbounded: bool,
}

/// One delivered numerical scalar's original row and ordered derivative axes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DiagnosticOutput {
    /// Original selected output row.
    pub row: SemanticId,
    /// Empty denotes its value; otherwise the ordered native partial derivative.
    pub derivatives: Vec<SemanticId>,
}

/// Durable mathematical projection, without supplier atoms or executable state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DiagnosticResult {
    /// Exact admitted case structure identity.
    pub structure: ContentHash,
    /// Ordered original selected row identities.
    pub rows: Vec<SemanticId>,
    /// Ordered original scalar coordinate identities.
    pub coordinates: Vec<SemanticId>,

    /// Actual mathematical assurance requested or delivered.
    pub claim: DiagnosticClaim,
    /// Requested or actual binary arithmetic precision.
    pub precision: u32,
    /// Original contributing scientific instance identities.
    pub original_sources: Vec<SemanticId>,

    /// Original source occurrence and retained domain premise.
    pub original_conditions: Vec<(SemanticId, String)>,
    /// Exact block projection when requested.
    pub exact: Option<ExactBlockResult>,

    /// Ordered numerical output projections when requested.
    pub numerical: Vec<NumericalDiagnosticValue>,
    /// Original row/derivative mapping in delivered numerical order.
    pub outputs: Vec<DiagnosticOutput>,
    /// Explicit unsupported or unestablished claim reason, if present.
    pub unsupported: Option<String>,

    /// Maintained supplier arithmetic and certification profile.
    pub supplier_profile: String,
    /// Native mathematical presentation frozen while the exact source product is owned.
    pub mathematical_views: Vec<crate::analytical::MathematicalView>,
}

struct Guard {
    source: SemanticId,
    condition: Condition,
    expression: Atom,
}

/// Request-owned exact source preparation. New numeric inputs reuse the same native
/// exact evaluator; no global cache or durable mathematical identity is introduced.
pub struct PreparedDiagnostic {
    structure: ContentHash,
    request: DiagnosticRequest,
    expressions: Vec<Atom>,
    evaluation_expressions: Vec<Atom>,
    outputs: Vec<DiagnosticOutput>,
    formals: Vec<Atom>,
    guards: Vec<Guard>,
    sources: Vec<SemanticId>,
    exact: Option<Arc<library::admission::ScalarProduct>>,
    variables: Vec<crate::binding::Variable>,
    reduction: std::sync::OnceLock<CachedReduction>,
}
struct CachedReduction {
    result: ExactBlockResult,
    conditions: Vec<Atom>,
    matrix: Vec<Vec<Atom>>,
    rhs: Vec<Atom>,
    reduced: Vec<Vec<Atom>>,
    reduced_rhs: Vec<Atom>,
    witnesses: Vec<Vec<Atom>>,
    recovered: BTreeMap<Symbol, Atom>,
}
impl std::fmt::Debug for PreparedDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedDiagnostic")
            .field("structure", &self.structure)
            .field("rows", &self.request.rows)
            .finish_non_exhaustive()
    }
}

fn check(scope: &ExecutionScope) -> Result<(), MathError> {
    scope.check().map_err(crate::error::scope_error)
}
fn operation(error: OperationError<MathError>) -> MathError {
    match error {
        OperationError::Control(error) => error,
        OperationError::Unsupported(profile) => {
            MathError::Contract(format!("unsupported diagnostic: {profile}"))
        }
        OperationError::InvalidProgram(message) => MathError::Library(message),
        OperationError::Overflow | OperationError::Allocation => {
            MathError::Limit("diagnostic native backing")
        }
        OperationError::Interrupted => MathError::Cancelled,
    }
}
fn control<'a>(
    scope: &'a ExecutionScope,
    limits: DiagnosticLimits,
) -> impl FnMut(OperationExtent) -> Result<(), MathError> + 'a {
    let mut peak = 0usize;
    move |extent| {
        check(scope)?;
        if extent.work > limits.work || extent.output_entries > limits.terms {
            return Err(MathError::Limit("diagnostic native work"));
        }
        let bytes = extent
            .peak_bytes()
            .ok_or(MathError::Limit("diagnostic native extent"))?;
        if bytes > limits.bytes {
            return Err(MathError::Limit("diagnostic native bytes"));
        }
        if bytes > peak {
            crate::construction::additional(bytes - peak)?;
            peak = bytes;
        }
        Ok(())
    }
}
fn exact_constant(atom: &Atom) -> Option<Rational> {
    match atom.as_view() {
        AtomView::Num(number) => match number.get_coeff_view() {
            CoefficientView::Natural(n, d, 0, _) if d != 0 => Some(Rational::new(n, d)),
            CoefficientView::Large(real, imaginary) if imaginary.is_zero() => Some(real.to_rat()),
            _ => None,
        },
        _ => None,
    }
}
fn substitute(atom: &Atom, substitutions: &BTreeMap<Symbol, Atom>) -> Atom {
    atom.replace_map(|node, _, output| {
        if let AtomView::Var(variable) = node
            && let Some(value) = substitutions.get(&variable.get_symbol())
        {
            output.set_from_view(&value.as_view());
        }
    })
}
/// Admit exact numeric substitution before native normalization can materialize
/// large powers. This counts a conservative rational numerator/denominator extent;
/// it does not evaluate the expression or replace the supplier interpreter.
fn admit_substitution(
    atom: &Atom,
    substitutions: &BTreeMap<Symbol, Atom>,
    limits: DiagnosticLimits,
    scope: &ExecutionScope,
) -> Result<(), MathError> {
    fn extent(
        atom: AtomView<'_>,
        substitutions: &BTreeMap<Symbol, Atom>,
        limits: DiagnosticLimits,
        scope: &ExecutionScope,
        depth: usize,
        work: &mut usize,
    ) -> Result<usize, MathError> {
        check(scope)?;
        *work = work
            .checked_add(1)
            .ok_or(MathError::Limit("exact substitution work"))?;
        if *work > limits.work || depth > 128 {
            return Err(MathError::Limit("exact substitution scope"));
        }
        let bits = match atom {
            AtomView::Num(_) => exact_constant(&atom.to_owned())
                .map(|value| crate::rational_binary64::bits(&value))
                .transpose()?
                .unwrap_or(1)
                .saturating_mul(2),
            AtomView::Var(variable) => {
                if let Some(value) = substitutions.get(&variable.get_symbol()) {
                    exact_constant(value)
                        .map(|value| crate::rational_binary64::bits(&value))
                        .transpose()?
                        .unwrap_or(1)
                        .saturating_mul(2)
                } else {
                    1
                }
            }
            AtomView::Add(add) => {
                let mut bits = 1usize;
                for argument in add.iter() {
                    bits = bits
                        .checked_add(extent(
                            argument,
                            substitutions,
                            limits,
                            scope,
                            depth + 1,
                            work,
                        )?)
                        .and_then(|bits| bits.checked_add(1))
                        .ok_or(MathError::Limit("exact substitution bits"))?;
                }
                bits
            }
            AtomView::Mul(mul) => {
                let mut bits = 1usize;
                for argument in mul.iter() {
                    bits = bits
                        .checked_add(extent(
                            argument,
                            substitutions,
                            limits,
                            scope,
                            depth + 1,
                            work,
                        )?)
                        .ok_or(MathError::Limit("exact substitution bits"))?;
                }
                bits
            }
            AtomView::Pow(power) => {
                let base = extent(
                    power.get_base(),
                    substitutions,
                    limits,
                    scope,
                    depth + 1,
                    work,
                )?;
                if let Ok(exponent) = i64::try_from(power.get_exp()) {
                    base.checked_mul(
                        usize::try_from(exponent.unsigned_abs())
                            .map_err(|_| MathError::Limit("exact substitution exponent"))?,
                    )
                    .ok_or(MathError::Limit("exact substitution bits"))?
                } else {
                    base.checked_add(extent(
                        power.get_exp(),
                        substitutions,
                        limits,
                        scope,
                        depth + 1,
                        work,
                    )?)
                    .ok_or(MathError::Limit("exact substitution bits"))?
                }
            }
            AtomView::Fun(function) => {
                let mut bits = 1usize;
                for argument in function.iter() {
                    bits = bits
                        .checked_add(extent(
                            argument,
                            substitutions,
                            limits,
                            scope,
                            depth + 1,
                            work,
                        )?)
                        .ok_or(MathError::Limit("exact substitution bits"))?;
                }
                bits
            }
        };
        if bits > limits.coefficient_bits {
            return Err(MathError::Limit("exact substitution coefficient bits"));
        }
        Ok(bits)
    }
    extent(atom.as_view(), substitutions, limits, scope, 0, &mut 0).map(|_| ())
}

fn symbol(atom: &Atom) -> Result<Symbol, MathError> {
    match atom.as_view() {
        AtomView::Var(variable) => Ok(variable.get_symbol()),
        _ => Err(MathError::Contract(
            "diagnostic coordinate is not a formal".into(),
        )),
    }
}

impl CasePlan {
    /// Admit selected original equations, source guards and one exact supplier preparation.
    pub fn prepare_mathematical_diagnostic(
        &self,
        values: &CaseValues,
        request: DiagnosticRequest,
        source_admission: Option<&ArithmeticSourceAdmission>,
        scope: &ExecutionScope,
    ) -> Result<PreparedDiagnostic, MathError> {
        check(scope)?;
        let limits = request.limits;
        let coordinates = request
            .unknowns
            .iter()
            .chain(&request.parameters)
            .copied()
            .collect::<Vec<_>>();
        let declared = self
            .structure()
            .variables()
            .iter()
            .map(|variable| variable.port.id)
            .chain(self.structure().parameters().iter().map(|port| port.id))
            .collect::<BTreeSet<_>>();
        if request.rows.is_empty()
            || request.rows.len() > limits.rows
            || coordinates.len() > limits.coordinates
            || (request.claim == DiagnosticClaim::ExactIdentity && request.unknowns.is_empty())
            || request.rows.iter().collect::<BTreeSet<_>>().len() != request.rows.len()
            || coordinates.iter().collect::<BTreeSet<_>>().len() != coordinates.len()
            || coordinates.iter().any(|id| !declared.contains(id))
            || request.unknowns.iter().any(|id| !self.structure().variables().iter()
                .any(|variable| variable.port.id == *id && !variable.fixed))
            || request
                .rows
                .iter()
                .any(|id| !self.structure().rows().iter().any(|row| row.id == *id))
            || request.precision == 0
            || request.precision as usize > limits.coefficient_bits
            || [
                limits.rows,
                limits.coordinates,
                limits.work,
                limits.terms,
                limits.bytes,
                limits.coefficient_bits,
            ]
            .contains(&0)
        {
            return Err(MathError::Contract(
                "invalid diagnostic row/coordinate/precision selection".into(),
            ));
        }
        if request.claim == DiagnosticClaim::ValidatedEnclosure
            && !source_admission.is_some_and(|admission| {
                admission.structure == self.structure().key() && admission.rows == request.rows
            })
        {
            return Err(MathError::Contract(
                "validated enclosure requires original compiler arithmetic admission".into(),
            ));
        }
        let mut expressions = crate::class_evidence::diagnostic_expressions(
            self,
            values,
            &request.rows,
            &coordinates,
            limits.work,
            limits.coefficient_bits,
            scope.cancellation(),
        )?;
        if request.claim == DiagnosticClaim::ExactIdentity {
            for (id, expression) in request.rows.iter().zip(&mut expressions) {
                let row = self
                    .structure()
                    .rows()
                    .iter()
                    .find(|row| row.id == *id)
                    .ok_or_else(|| MathError::Contract("missing exact row".into()))?;
                if !row.lower.is_finite() || row.lower != row.upper {
                    return Err(MathError::Contract(
                        "exact block requires finite original equalities".into(),
                    ));
                }
                *expression -= Atom::num(row.lower);
            }
        }
        let formals = (0..coordinates.len())
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        for formal in &formals[..request.unknowns.len()] {
            let formal_symbol = symbol(formal)?;
            if !expressions
                .iter()
                .any(|expression| expression.get_all_symbols(true).contains(&formal_symbol))
            {
                return Err(MathError::Contract(
                    "diagnostic unknown absent from selected support".into(),
                ));
            }
        }
        let coordinate_map = coordinates
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index))
            .collect::<BTreeMap<_, _>>();
        if request.derivatives.iter().any(|axes| {
            axes.is_empty()
                || axes.len() > 2
                || axes.iter().any(|axis| !coordinate_map.contains_key(axis))
        }) || request.derivatives.iter().collect::<BTreeSet<_>>().len()
            != request.derivatives.len()
            || (request.claim == DiagnosticClaim::ExactIdentity && !request.derivatives.is_empty())
        {
            return Err(MathError::Contract(
                "invalid or unsupported diagnostic derivative demand".into(),
            ));
        }
        let order = match request.derivatives.iter().map(Vec::len).max().unwrap_or(0) {
            0 => pse_kernels::DerivativeOrder::Value,
            1 => pse_kernels::DerivativeOrder::First,
            _ => pse_kernels::DerivativeOrder::Second,
        };
        let derivative_axes = request
            .derivatives
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut guards = Vec::new();
        let mut sources = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for row in &request.rows {
            for &(binding_index, contribution_index) in self.row_contributions(*row) {
                let binding = &self.structure().instances()[binding_index];
                let output = binding.contributions[contribution_index].output;
                if !visited.insert((binding_index, output)) {
                    continue;
                }
                sources.insert(binding.instance);
                let body = &self.bodies()[&binding.body];
                let local_formals = body.diagnostic_formals()?;
                if order > pse_kernels::DerivativeOrder::Value {
                    let axes = binding
                        .slots
                        .iter()
                        .enumerate()
                        .filter(|(_, binding)| derivative_axes.contains(&binding.source()))
                        .map(|(slot, _)| body.diagnostic_axis(slot))
                        .collect::<Result<Vec<_>, _>>()?;
                    if body.available_order_for_outputs(&[output], &axes)? < order {
                        return Err(MathError::Contract(
                            "selected source lacks requested diagnostic derivative order".into(),
                        ));
                    }
                }
                let stages = body.demanded_stages(&[output])?;
                // Retain exactly this original output/effect closure. Unrelated
                // multi-output inputs must not become required fixed assumptions.
                let mut needed_inputs = BTreeSet::new();
                for stage in &stages {
                    match stage {
                        Stage::Block { expressions, .. } => {
                            for expression in expressions {
                                needed_inputs.extend(
                                    expression
                                        .get_all_symbols(true)
                                        .into_iter()
                                        .filter_map(|symbol| body.input_slot(symbol)),
                                );
                            }
                        }
                        Stage::Require {
                            argument,
                            order: minimum_order,
                            ..
                        } => {
                            if *minimum_order <= order && *argument < binding.slots.len() {
                                needed_inputs.insert(*argument);
                            }
                        }
                        _ => {
                            return Err(MathError::Contract(
                                "unsupported diagnostic original branch/provider/domain stage"
                                    .into(),
                            ));
                        }
                    }
                }
                let mut slots = BTreeMap::<usize, Atom>::new();
                for slot in needed_inputs {
                    let binding = binding.slots.get(slot).ok_or_else(|| {
                        MathError::Contract("diagnostic original input layout".into())
                    })?;
                    let value = if let Some(&index) = coordinate_map.get(&binding.source()) {
                        formals[index].clone()
                    } else {
                        let value = values.scalars.get(&binding.source()).ok_or_else(|| {
                            MathError::Contract("missing diagnostic fixed input".into())
                        })?;
                        if !value.is_finite() {
                            return Err(MathError::Contract(
                                "nonfinite diagnostic fixed input".into(),
                            ));
                        }
                        crate::class_evidence::exact_binary64_atom(*value, limits.coefficient_bits)?
                    };
                    slots.insert(
                        slot,
                        value * crate::class_evidence::exact_binary64_atom(binding.scale(), limits.coefficient_bits)?
                            + crate::class_evidence::exact_binary64_atom(binding.offset(), limits.coefficient_bits)?,
                    );
                }
                for stage in stages {
                    check(scope)?;
                    match stage {
                        Stage::Block {
                            expressions,
                            outputs,
                            ..
                        } => {
                            let substitutions = slots
                                .iter()
                                .map(|(slot, value)| {
                                    Ok((
                                        symbol(local_formals.get(*slot).ok_or_else(|| {
                                            MathError::Contract("diagnostic local slot".into())
                                        })?)?,
                                        value.clone(),
                                    ))
                                })
                                .collect::<Result<BTreeMap<_, _>, MathError>>()?;
                            for (expression, slot) in expressions.iter().zip(outputs) {
                                admit_substitution(expression, &substitutions, limits, scope)?;
                                let bound = substitute(expression, &substitutions);
                                if bound.as_view().get_byte_size() > limits.bytes {
                                    return Err(MathError::Limit("diagnostic guard expression"));
                                }
                                slots.insert(slot, bound);
                            }
                        }
                        Stage::Require {
                            argument,
                            condition,
                            source,
                            order: minimum_order,
                            ..
                        } => {
                            if minimum_order > order {
                                continue;
                            }
                            let expression = slots.get(&argument).cloned().ok_or_else(|| {
                                MathError::Contract("diagnostic original guard slot".into())
                            })?;
                            guards.push(Guard {
                                source,
                                condition,
                                expression,
                            });
                        }
                        _ => {
                            return Err(MathError::Contract(
                                "unsupported diagnostic original branch/provider/domain stage"
                                    .into(),
                            ));
                        }
                    }
                }
            }
        }
        let output_count = request
            .rows
            .len()
            .checked_mul(
                request
                    .derivatives
                    .len()
                    .checked_add(1)
                    .ok_or(MathError::Limit("diagnostic derivative population"))?,
            )
            .ok_or(MathError::Limit("diagnostic derivative population"))?;
        if output_count > limits.terms {
            return Err(MathError::Limit("diagnostic derivative outputs"));
        }
        let mut evaluation_expressions = expressions.clone();
        let mut outputs = request
            .rows
            .iter()
            .map(|row| DiagnosticOutput {
                row: *row,
                derivatives: vec![],
            })
            .collect::<Vec<_>>();
        for axes in &request.derivatives {
            for (row, original) in request.rows.iter().zip(&expressions) {
                let mut expression = original.clone();
                for axis in axes {
                    check(scope)?;
                    if expression.as_view().get_byte_size() > limits.bytes {
                        return Err(MathError::Limit("diagnostic partial source bytes"));
                    }
                    expression = symbolica::analytical::derivative_with(
                        expression.as_view(),
                        &symbol(&formals[coordinate_map[axis]])?.into(),
                        &mut control(scope, limits),
                    )?;
                    check(scope)?;
                    let operations = expression.count_operations();
                    let work = operations
                        .additions
                        .checked_add(operations.multiplications)
                        .and_then(|work| work.checked_add(operations.inversions))
                        .and_then(|work| work.checked_add(operations.function_calls))
                        .ok_or(MathError::Limit("diagnostic partial work"))?;
                    if work > limits.work || expression.as_view().get_byte_size() > limits.bytes {
                        return Err(MathError::Limit("diagnostic partial construction"));
                    }
                }
                evaluation_expressions.push(expression);
                outputs.push(DiagnosticOutput {
                    row: *row,
                    derivatives: axes.clone(),
                });
            }
        }
        let exact = if request.claim == DiagnosticClaim::ExactIdentity {
            None
        } else {
            Some(library::admission::diagnostic_product(
                &evaluation_expressions,
                &formals,
                Optimization {
                    cores: scope.admitted_threads().get(),
                    ..Optimization::default()
                },
                scope,
                EvaluationLimits {
                    operations: limits.work,
                    scratch_bytes: limits.bytes,
                    ..EvaluationLimits::default()
                },
            )?)
        };
        check(scope)?;
        Ok(PreparedDiagnostic {
            structure: self.structure().key(),
            request,
            expressions,
            evaluation_expressions,
            outputs,
            formals,
            guards,
            sources: sources.into_iter().collect(),
            exact,
            variables: self.structure().variables().to_vec(),
            reduction: std::sync::OnceLock::new(),
        })
    }

    /// Perform the selected diagnostic through the prepared native product.
    pub fn mathematical_diagnostics(
        &self,
        values: &CaseValues,
        request: DiagnosticRequest,
        source_admission: Option<&ArithmeticSourceAdmission>,
        scope: &ExecutionScope,
    ) -> Result<DiagnosticResult, MathError> {
        self.prepare_mathematical_diagnostic(values, request, source_admission, scope)?
            .evaluate(scope)
    }
}

type ExactField = RationalPolynomialField<Z, u16>;
type ExactMatrix = Matrix<ExactField>;

impl PreparedDiagnostic {
    /// Bind new points/intervals while retaining the admitted exact preparation.
    pub fn evaluate_with_inputs(
        &self,
        inputs: &[DiagnosticInput],
        scope: &ExecutionScope,
    ) -> Result<DiagnosticResult, MathError> {
        check(scope)?;
        let request = &self.request;
        let mut result = DiagnosticResult {
            structure: self.structure,
            rows: request.rows.clone(),
            coordinates: request
                .unknowns
                .iter()
                .chain(&request.parameters)
                .copied()
                .collect(),
            claim: request.claim,
            precision: match request.claim {
                DiagnosticClaim::Ordinary => 53,
                DiagnosticClaim::DoubleFloat => 106,
                _ => request.precision,
            },
            original_sources: self.sources.clone(),
            original_conditions: self
                .guards
                .iter()
                .map(|guard| {
                    (
                        guard.source,
                        format!("{} is {}", guard.expression, guard.condition.description()),
                    )
                })
                .collect(),
            exact: None,
            numerical: vec![],
            outputs: self.outputs.clone(),
            unsupported: None,
            supplier_profile:
                "symbolica-3.0.1;maintained-checked-diagnostic-v1;numerica-3.0.1;real-basic-ball-v1"
                    .into(),
            mathematical_views: self
                .evaluation_expressions
                .iter()
                .map(|expression| {
                    crate::analytical::mathematical_view(
                        expression,
                        EvaluationLimits {
                            operations: request.limits.work,
                            scratch_bytes: request.limits.bytes,
                            ..EvaluationLimits::default()
                        },
                        scope,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        let mut bound = BTreeMap::new();
        for input in inputs {
            check(scope)?;
            if !result.coordinates.contains(&input.coordinate)
                || bound.contains_key(&input.coordinate)
            {
                return Err(MathError::Contract(
                    "duplicate or absent diagnostic input coordinate".into(),
                ));
            }
            let lower = input.lower.rational(request.limits)?;
            let upper = input
                .upper
                .as_ref()
                .map(|upper| upper.rational(request.limits))
                .transpose()?
                .unwrap_or_else(|| lower.clone());
            if lower > upper
                || (lower != upper && request.claim != DiagnosticClaim::ValidatedEnclosure)
            {
                return Err(MathError::Contract(
                    "unordered or unsupported diagnostic interval".into(),
                ));
            }
            if request.claim != DiagnosticClaim::ExactIdentity
                && let Some(variable) = self
                    .variables
                    .iter()
                    .find(|variable| variable.port.id == input.coordinate)
                && !coordinate_interval_permitted(variable, &lower, &upper)?
            {
                return Err(MathError::Contract(
                    "diagnostic input violates original coordinate domain or bounds".into(),
                ));
            }
            bound.insert(input.coordinate, (lower, upper));
        }
        if request.claim == DiagnosticClaim::ExactIdentity {
            if request.unknowns.iter().any(|id| bound.contains_key(id)) {
                return Err(MathError::Contract(
                    "exact-block unknowns are solved, not specialized inputs".into(),
                ));
            }
            result.exact = Some(self.exact_block(&bound, scope)?);
            check(scope)?;
            return Ok(result);
        }
        if bound.len() != result.coordinates.len() {
            return Err(MathError::Contract(
                "missing diagnostic point/interval inputs".into(),
            ));
        }
        let intervals = result
            .coordinates
            .iter()
            .map(|id| bound[id].clone())
            .collect::<Vec<_>>();
        // Original denominator/domain obligations are checked before the normalized
        // numerical product can execute, including obligations whose output canceled.
        let substitutions = self
            .formals
            .iter()
            .zip(&intervals)
            .map(|(formal, (value, _))| Ok((symbol(formal)?, Atom::num(value.clone()))))
            .collect::<Result<BTreeMap<_, _>, MathError>>()?;
        if request.claim != DiagnosticClaim::ValidatedEnclosure {
            for guard in &self.guards {
                admit_substitution(&guard.expression, &substitutions, request.limits, scope)?;
                let value = exact_constant(&substitute(&guard.expression, &substitutions))
                    .ok_or_else(|| {
                        MathError::Contract("diagnostic guard has no exact point decision".into())
                    })?;
                if !permits(guard.condition, &value) {
                    return Err(MathError::Contract(format!(
                        "original diagnostic domain violation at {}",
                        guard.source
                    )));
                }
            }
        }
        let exact = self
            .exact
            .as_ref()
            .ok_or_else(|| MathError::Contract("missing native diagnostic preparation".into()))?;
        let mut admission = control(scope, request.limits);
        let diagnostic = match request.claim {
            DiagnosticClaim::Ordinary => {
                let mut evaluator = exact
                    .evaluator
                    .try_map_coeff_with_prec(
                        &|value| {
                            crate::rational_binary64::nearest(&value.re)
                                .map_err(|error| error.to_string())
                                .and_then(|value| {
                                    if value.is_finite() {
                                        Ok(value)
                                    } else {
                                        Err("nonfinite diagnostic coefficient".into())
                                    }
                                })
                        },
                        53,
                        &mut admission,
                    )
                    .map_err(operation)?;
                let inputs = intervals
                    .iter()
                    .map(|(value, _)| {
                        crate::rational_binary64::nearest(value).and_then(|value| {
                            if value.is_finite() {
                                Ok(value)
                            } else {
                                Err(MathError::Contract("nonfinite diagnostic input".into()))
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut outputs = vec![0.0; self.outputs.len()];
                evaluator
                    .try_evaluate_with(&inputs, &mut outputs, &mut admission)
                    .map_err(operation)?;
                outputs
                    .into_iter()
                    .map(|value| estimate(value, None))
                    .collect::<Result<Vec<_>, _>>()?
            }
            DiagnosticClaim::DoubleFloat => {
                let mut evaluator = exact
                    .evaluator
                    .try_map_coeff_with_prec(
                        &|value| Ok(DoubleFloat::from(&value.re)),
                        106,
                        &mut admission,
                    )
                    .map_err(operation)?;
                let inputs = intervals
                    .iter()
                    .map(|(value, _)| DoubleFloat::from(value))
                    .collect::<Vec<_>>();
                let mut outputs = vec![DoubleFloat::new_zero(); self.outputs.len()];
                evaluator
                    .try_evaluate_with(&inputs, &mut outputs, &mut admission)
                    .map_err(operation)?;
                outputs
                    .into_iter()
                    .map(|value| estimate(value, None))
                    .collect::<Result<Vec<_>, _>>()?
            }
            DiagnosticClaim::ChosenPrecision => {
                let mut evaluator = exact
                    .evaluator
                    .try_map_coeff_with_prec(
                        &|value| Ok(value.re.to_multi_prec_float(request.precision)),
                        request.precision,
                        &mut admission,
                    )
                    .map_err(operation)?;
                let inputs = intervals
                    .iter()
                    .map(|(value, _)| value.to_multi_prec_float(request.precision))
                    .collect::<Vec<_>>();
                let mut outputs = vec![Float::new(request.precision); self.outputs.len()];
                evaluator
                    .try_evaluate_with(&inputs, &mut outputs, &mut admission)
                    .map_err(operation)?;
                outputs
                    .into_iter()
                    .map(|value| estimate(value, None))
                    .collect::<Result<Vec<_>, _>>()?
            }
            DiagnosticClaim::PropagatedError => {
                let make = |value: &Rational| {
                    ErrorPropagatingFloat::new(
                        value.to_multi_prec_float(request.precision),
                        f64::from(request.precision) * std::f64::consts::LOG10_2 - 1.0,
                    )
                };
                let mut evaluator = exact
                    .evaluator
                    .try_map_coeff_with_prec(
                        &|value| Ok(make(&value.re)),
                        request.precision,
                        &mut admission,
                    )
                    .map_err(operation)?;
                let inputs = intervals
                    .iter()
                    .map(|(value, _)| make(value))
                    .collect::<Vec<_>>();
                let mut outputs = vec![make(&Rational::zero()); self.outputs.len()];
                evaluator
                    .try_evaluate_with(&inputs, &mut outputs, &mut admission)
                    .map_err(operation)?;
                outputs
                    .into_iter()
                    .map(|value| {
                        estimate(value.get_num().clone(), Some(value.get_absolute_error()))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
            DiagnosticClaim::ValidatedEnclosure => {
                let inputs = intervals
                    .iter()
                    .map(|(lower, upper)| {
                        RealBall::from_rational_bounds(lower, upper, request.precision)
                    })
                    .collect::<Vec<_>>();
                for guard in &self.guards {
                    let guard_product = library::admission::diagnostic_product(
                        &[guard.expression.clone()],
                        &self.formals,
                        Optimization::default(),
                        scope,
                        EvaluationLimits {
                            operations: request.limits.work,
                            scratch_bytes: request.limits.bytes,
                            ..EvaluationLimits::default()
                        },
                    )?;
                    let mut evaluator = guard_product
                        .evaluator
                        .try_map_real_ball_with(request.precision, &mut admission)
                        .map_err(operation)?;
                    let mut outputs = vec![RealBall::from_rational_bounds(
                        &Rational::zero(),
                        &Rational::zero(),
                        request.precision,
                    )];
                    evaluator
                        .try_evaluate_with(&inputs, &mut outputs, &mut admission)
                        .map_err(operation)?;
                    let ball = &outputs[0];
                    let permits = match guard.condition {
                        Condition::Nonzero => {
                            ball.is_strictly_positive() || ball.is_strictly_negative()
                        }
                        Condition::Positive => ball.is_strictly_positive(),
                        Condition::Nonnegative => {
                            ball.lower_bound() >= Float::new(request.precision)
                        }
                    };
                    if !permits {
                        result.unsupported = Some(format!(
                            "original domain is singular or unestablished over input interval at {}",
                            guard.source
                        ));
                        return Ok(result);
                    }
                }
                let mut evaluator = exact
                    .evaluator
                    .try_map_real_ball_with(request.precision, &mut admission)
                    .map_err(operation)?;
                let mut outputs = vec![
                    RealBall::from_rational_bounds(
                        &Rational::zero(),
                        &Rational::zero(),
                        request.precision
                    );
                    self.outputs.len()
                ];
                evaluator
                    .try_evaluate_with(&inputs, &mut outputs, &mut admission)
                    .map_err(operation)?;
                outputs
                    .into_iter()
                    .map(|ball| {
                        if !ball.is_finite() {
                            return NumericalDiagnosticValue {
                                value: None,
                                absolute_error: None,
                                lower: None,
                                upper: None,
                                unbounded: true,
                            };
                        }
                        NumericalDiagnosticValue {
                            value: None,
                            absolute_error: None,
                            lower: Some(ball.lower_bound().to_rational().to_string()),
                            upper: Some(ball.upper_bound().to_rational().to_string()),
                            unbounded: false,
                        }
                    })
                    .collect()
            }
            DiagnosticClaim::ExactIdentity => {
                return Err(MathError::Contract("unreachable diagnostic claim".into()));
            }
        };
        check(scope)?;
        result.numerical = diagnostic;
        Ok(result)
    }
    /// Evaluate the exact original inputs retained by this request.
    pub fn evaluate(&self, scope: &ExecutionScope) -> Result<DiagnosticResult, MathError> {
        self.evaluate_with_inputs(&self.request.inputs, scope)
    }

    fn exact_block(
        &self,
        inputs: &BTreeMap<SemanticId, (Rational, Rational)>,
        scope: &ExecutionScope,
    ) -> Result<ExactBlockResult, MathError> {
        if !inputs.is_empty()
            && let Some(result) = self.reduced_specialization(inputs, scope)?
        {
            return Ok(result);
        }
        let unknown_count = self.request.unknowns.len();
        let substitutions = self
            .request
            .parameters
            .iter()
            .enumerate()
            .filter_map(|(index, id)| {
                inputs
                    .get(id)
                    .map(|(value, _)| (index + unknown_count, value))
            })
            .map(|(index, value)| Ok((symbol(&self.formals[index])?, Atom::num(value.clone()))))
            .collect::<Result<BTreeMap<_, _>, MathError>>()?;
        let equations = self
            .expressions
            .iter()
            .map(|expression| {
                admit_substitution(expression, &substitutions, self.request.limits, scope)?;
                Ok(substitute(expression, &substitutions))
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let (matrix, rhs) = symbolica::solve::system_to_matrix_with::<u16, _, _, _>(
            &equations,
            &self.formals[..unknown_count],
            &mut control(scope, self.request.limits),
        )
        .map_err(operation)?;
        check(scope)?;
        let original_matrix = (0..matrix.nrows())
            .map(|row| {
                (0..matrix.ncols())
                    .map(|column| matrix[(row as u32, column as u32)].to_string())
                    .collect()
            })
            .collect();
        let original_rhs = (0..rhs.nrows())
            .map(|row| rhs[(row as u32, 0)].to_string())
            .collect();
        // The witness augmentation is a new owned native population. Admit its
        // backing before identity/augment allocate, then widen only at safe points.
        let augmented_entries = matrix
            .nrows()
            .checked_mul(
                unknown_count
                    .checked_add(1)
                    .and_then(|columns| columns.checked_add(matrix.nrows()))
                    .ok_or(MathError::Limit("exact witness columns"))?,
            )
            .ok_or(MathError::Limit("exact witness entries"))?;
        let variables = matrix
            .row_iter()
            .flatten()
            .chain(rhs.row_iter().flatten())
            .map(|value| value.numerator.nvars().max(value.denominator.nvars()))
            .max()
            .unwrap_or(0);
        let entry_bytes = variables
            .checked_mul(2 * size_of::<u16>())
            .and_then(|bytes| {
                bytes.checked_add(
                    size_of::<RationalPolynomial<Z, u16>>() + 2 * size_of::<Integer>() + 128,
                )
            })
            .ok_or(MathError::Limit("exact witness entry backing"))?;
        let augmented_bytes = augmented_entries
            .checked_mul(entry_bytes)
            .ok_or(MathError::Limit("exact witness backing"))?;
        if augmented_bytes > self.request.limits.bytes {
            return Err(MathError::Limit("exact witness bytes"));
        }
        crate::construction::additional(augmented_bytes)?;
        let identity = Matrix::identity(matrix.nrows() as u32, matrix.field().clone());
        let mut reduced = matrix
            .augment(&rhs)
            .and_then(|matrix| matrix.augment(&identity))
            .map_err(|error| MathError::Library(error.to_string()))?;
        let limits = self.request.limits;
        let mut matrix_peak = augmented_bytes;
        let witness = reduced.row_reduce_with(
            unknown_count as u32,
            &mut |current: &ExactMatrix, pivots, work, operation| {
                check(scope)?;
                if work > limits.work {
                    return Err(MathError::Limit("exact block elimination work"));
                }
                let predicted = exact_arithmetic_admission(operation, limits)?;
                let mut terms = 0usize;
                let mut bytes = 0usize;
                for value in current.row_iter().flatten().chain(pivots) {
                    bytes = bytes
                        .checked_add(size_of::<RationalPolynomial<Z, u16>>())
                        .ok_or(MathError::Limit("exact block value backing"))?;
                    for polynomial in [&value.numerator, &value.denominator] {
                        bytes = polynomial
                            .coefficients
                            .capacity()
                            .checked_mul(size_of::<Integer>())
                            .and_then(|n| {
                                polynomial
                                    .exponents
                                    .capacity()
                                    .checked_mul(size_of::<u16>())
                                    .and_then(|e| n.checked_add(e))
                            })
                            .and_then(|n| {
                                polynomial
                                    .nvars()
                                    .checked_mul(128)
                                    .and_then(|v| n.checked_add(v))
                            })
                            .and_then(|n| bytes.checked_add(n))
                            .ok_or(MathError::Limit("exact block polynomial backing"))?;
                    }
                    terms = terms
                        .checked_add(value.numerator.nterms())
                        .and_then(|n| n.checked_add(value.denominator.nterms()))
                        .ok_or(MathError::Limit("exact block term extent"))?;
                    for integer in value
                        .numerator
                        .coefficients
                        .iter()
                        .chain(&value.denominator.coefficients)
                    {
                        let bits = integer.significant_bits() as usize;
                        if bits > limits.coefficient_bits {
                            return Err(MathError::Limit("exact block coefficient bits"));
                        }
                        bytes = bytes
                            .checked_add(bits.div_ceil(8))
                            .ok_or(MathError::Limit("exact block bytes"))?;
                    }
                }
                if terms > limits.terms || bytes > limits.bytes {
                    return Err(MathError::Limit("exact block native extent"));
                }
                let required = bytes
                    .checked_mul(8)
                    .and_then(|bytes| bytes.checked_add(predicted))
                    .ok_or(MathError::Limit("exact block simultaneous backing"))?;
                if required > limits.bytes {
                    return Err(MathError::Limit("exact block simultaneous bytes"));
                }
                if required > matrix_peak {
                    crate::construction::additional(required - matrix_peak)?;
                    matrix_peak = required;
                }
                Ok(())
            },
        )?;
        let rank = witness.rank;
        let mut conditions = witness
            .pivots
            .iter()
            .filter(|value| !value.is_constant())
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        for value in matrix.row_iter().flatten().chain(rhs.row_iter().flatten()) {
            if !value.denominator.is_constant() {
                conditions.insert(value.denominator.to_string());
            }
        }
        let mut row_witnesses = Vec::new();
        let mut inconsistent = false;
        for row in rank..matrix.nrows() {
            let nonzero_rhs = !reduced[(row as u32, unknown_count as u32)].is_zero();
            inconsistent |= nonzero_rhs;
            if nonzero_rhs && !reduced[(row as u32, unknown_count as u32)].is_constant() {
                conditions.insert(reduced[(row as u32, unknown_count as u32)].to_string());
            }
            row_witnesses.push(RowWitness {
                original_rows: self.request.rows.clone(),
                multipliers: (0..matrix.nrows())
                    .map(|column| {
                        reduced[(row as u32, (unknown_count + 1 + column) as u32)].to_string()
                    })
                    .collect(),
                inconsistent: nonzero_rhs,
            });
        }
        let consistency = if inconsistent {
            ExactConsistency::Inconsistent
        } else if rank < unknown_count {
            ExactConsistency::Underdetermined
        } else {
            ExactConsistency::Unique
        };
        let mut recovered = Vec::new();
        let mut recovered_atoms = BTreeMap::new();
        if consistency == ExactConsistency::Unique {
            for (row, &column) in witness.pivot_columns.iter().enumerate() {
                let expression = reduced[(row as u32, unknown_count as u32)].to_expression();
                recovered.push((self.request.unknowns[column], expression.to_string()));
                recovered_atoms.insert(symbol(&self.formals[column])?, expression);
            }
        }
        let mut full_substitutions = substitutions;
        full_substitutions.extend(recovered_atoms.clone());
        let (domain, bounds, recovered_valid) = self.validate_recovered(
            &equations,
            &full_substitutions,
            consistency == ExactConsistency::Unique,
            scope,
        )?;
        check(scope)?;
        let result = ExactBlockResult {
            matrix: original_matrix,
            right_hand_side: original_rhs,
            rank,
            augmented_rank: rank + usize::from(inconsistent),
            consistency,
            nonzero_conditions: conditions.into_iter().collect(),
            reduced_matrix: (0..matrix.nrows())
                .map(|row| {
                    (0..unknown_count)
                        .map(|column| reduced[(row as u32, column as u32)].to_string())
                        .collect()
                })
                .collect(),
            reduced_right_hand_side: (0..matrix.nrows())
                .map(|row| reduced[(row as u32, unknown_count as u32)].to_string())
                .collect(),
            row_witnesses,
            recovered,
            original_domain_valid: domain,
            physical_bounds_valid: bounds,
            reduced_evaluation: recovered_valid && domain == Some(true) && bounds == Some(true),
            used_reduced_product: false,
        };
        if inputs.is_empty() && result.consistency == ExactConsistency::Unique {
            let mut conditions = witness
                .pivots
                .iter()
                .flat_map(|value| {
                    [
                        value.numerator.to_expression(),
                        value.denominator.to_expression(),
                    ]
                })
                .collect::<Vec<_>>();
            conditions.extend(
                matrix
                    .row_iter()
                    .flatten()
                    .chain(rhs.row_iter().flatten())
                    .map(|value| value.denominator.to_expression()),
            );
            let cached = CachedReduction {
                result: result.clone(),
                conditions,
                matrix: (0..matrix.nrows())
                    .map(|row| {
                        (0..unknown_count)
                            .map(|column| matrix[(row as u32, column as u32)].to_expression())
                            .collect()
                    })
                    .collect(),
                rhs: (0..rhs.nrows())
                    .map(|row| rhs[(row as u32, 0)].to_expression())
                    .collect(),
                reduced: (0..matrix.nrows())
                    .map(|row| {
                        (0..unknown_count)
                            .map(|column| reduced[(row as u32, column as u32)].to_expression())
                            .collect()
                    })
                    .collect(),
                reduced_rhs: (0..matrix.nrows())
                    .map(|row| reduced[(row as u32, unknown_count as u32)].to_expression())
                    .collect(),
                witnesses: (rank..matrix.nrows())
                    .map(|row| {
                        (0..matrix.nrows())
                            .map(|column| {
                                reduced[(row as u32, (unknown_count + 1 + column) as u32)]
                                    .to_expression()
                            })
                            .collect()
                    })
                    .collect(),
                recovered: recovered_atoms,
            };
            // A racing consumer may already have published the same immutable request product.
            if self.reduction.set(cached).is_err() {
                check(scope)?;
            }
        }
        Ok(result)
    }

    fn validate_recovered(
        &self,
        equations: &[Atom],
        full_substitutions: &BTreeMap<Symbol, Atom>,
        unique: bool,
        scope: &ExecutionScope,
    ) -> Result<(Option<bool>, Option<bool>, bool), MathError> {
        for expression in equations
            .iter()
            .chain(self.guards.iter().map(|guard| &guard.expression))
        {
            admit_substitution(expression, full_substitutions, self.request.limits, scope)?;
        }
        let domain = if unique {
            let mut established = true;
            let mut valid = true;
            for guard in &self.guards {
                if let Some(value) =
                    exact_constant(&substitute(&guard.expression, full_substitutions))
                {
                    valid &= permits(guard.condition, &value);
                } else {
                    established = false;
                }
            }
            if !valid {
                Some(false)
            } else if established {
                Some(true)
            } else {
                None
            }
        } else {
            None
        };
        let bounds = if unique {
            let mut valid = true;
            let mut established = true;
            for (index, id) in self.request.unknowns.iter().enumerate() {
                let expression = full_substitutions
                    .get(&symbol(&self.formals[index])?)
                    .ok_or_else(|| MathError::Contract("missing recovered coordinate".into()))?;
                if let Some(value) = exact_constant(expression) {
                    if let Some(variable) = self
                        .variables
                        .iter()
                        .find(|variable| variable.port.id == *id)
                    {
                        valid &= coordinate_interval_permitted(variable, &value, &value)?;
                    }
                } else {
                    established = false;
                }
            }
            if !valid {
                Some(false)
            } else if established {
                Some(true)
            } else {
                None
            }
        } else {
            None
        };
        // Recovery is accepted only against the original selected residuals, never
        // solely because the reduced equations report a solution.
        let recovered_valid = unique
            && equations
                .iter()
                .all(|equation| substitute(equation, full_substitutions).is_zero());

        Ok((domain, bounds, recovered_valid))
    }

    fn reduced_specialization(
        &self,
        inputs: &BTreeMap<SemanticId, (Rational, Rational)>,
        scope: &ExecutionScope,
    ) -> Result<Option<ExactBlockResult>, MathError> {
        if self
            .request
            .parameters
            .iter()
            .any(|id| !inputs.contains_key(id))
        {
            return Ok(None);
        }
        if self.reduction.get().is_none() {
            self.exact_block(&BTreeMap::new(), scope)?;
        }
        let Some(cached) = self.reduction.get() else {
            return Ok(None);
        };
        let parameters = self
            .request
            .parameters
            .iter()
            .enumerate()
            .map(|(index, id)| {
                Ok((
                    symbol(&self.formals[self.request.unknowns.len() + index])?,
                    Atom::num(inputs[id].0.clone()),
                ))
            })
            .collect::<Result<BTreeMap<_, _>, MathError>>()?;
        for expression in cached
            .conditions
            .iter()
            .chain(cached.recovered.values())
            .chain(cached.matrix.iter().flatten())
            .chain(&cached.rhs)
            .chain(cached.reduced.iter().flatten())
            .chain(&cached.reduced_rhs)
            .chain(cached.witnesses.iter().flatten())
        {
            admit_substitution(expression, &parameters, self.request.limits, scope)?;
        }
        // Exceptional values always re-extract and reduce the original selected block.
        // A canceled denominator in a generic inverse cannot classify those values.
        for condition in &cached.conditions {
            check(scope)?;
            if !exact_constant(&substitute(condition, &parameters))
                .is_some_and(|value| !value.is_zero())
            {
                return Ok(None);
            }
        }
        let mut recovered = BTreeMap::new();
        for (coordinate, expression) in &cached.recovered {
            check(scope)?;
            recovered.insert(*coordinate, substitute(expression, &parameters));
        }
        let mut full = parameters.clone();
        full.extend(recovered.clone());
        let (domain, bounds, residual) =
            self.validate_recovered(&self.expressions, &full, true, scope)?;
        let project = |expression: &Atom| substitute(expression, &parameters).to_string();
        let mut result = cached.result.clone();
        result.matrix = cached
            .matrix
            .iter()
            .map(|row| row.iter().map(project).collect())
            .collect();
        result.right_hand_side = cached.rhs.iter().map(project).collect();
        result.reduced_matrix = cached
            .reduced
            .iter()
            .map(|row| row.iter().map(project).collect())
            .collect();
        result.reduced_right_hand_side = cached.reduced_rhs.iter().map(project).collect();
        for (witness, expressions) in result.row_witnesses.iter_mut().zip(&cached.witnesses) {
            witness.multipliers = expressions.iter().map(project).collect();
        }
        result.nonzero_conditions.clear();
        result.recovered = self
            .request
            .unknowns
            .iter()
            .enumerate()
            .map(|(index, id)| {
                Ok((
                    *id,
                    recovered
                        .get(&symbol(&self.formals[index])?)
                        .ok_or_else(|| MathError::Contract("missing reduced recovery".into()))?
                        .to_string(),
                ))
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        result.original_domain_valid = domain;
        result.physical_bounds_valid = bounds;
        result.reduced_evaluation = residual && domain == Some(true) && bounds == Some(true);
        result.used_reduced_product = true;
        check(scope)?;
        Ok(Some(result))
    }
}
/// Conservative numerator/denominator populations before the native field operation.
fn exact_arithmetic_admission(
    operation: RowReductionOperation<'_, ExactField>,
    limits: DiagnosticLimits,
) -> Result<usize, MathError> {
    let shape =
        |value: &RationalPolynomial<Z, u16>| -> Result<(usize, usize, usize, usize), MathError> {
            let bits = value
                .numerator
                .coefficients
                .iter()
                .chain(&value.denominator.coefficients)
                .map(|integer| integer.significant_bits() as usize)
                .max()
                .unwrap_or(1)
                .max(1);
            Ok((
                value.numerator.nterms().max(1),
                value.denominator.nterms().max(1),
                bits,
                [&value.numerator, &value.denominator]
                    .into_iter()
                    .flat_map(|polynomial| {
                        (0..polynomial.nterms()).map(move |term| {
                            polynomial
                                .exponents(term)
                                .iter()
                                .map(|&e| usize::from(e))
                                .sum::<usize>()
                        })
                    })
                    .max()
                    .unwrap_or(0),
            ))
        };
    let mul = |a: usize, b: usize| {
        a.checked_mul(b)
            .ok_or(MathError::Limit("exact arithmetic growth"))
    };
    let add = |a: usize, b: usize| {
        a.checked_add(b)
            .ok_or(MathError::Limit("exact arithmetic growth"))
    };
    let (terms, bits, degree) = match operation {
        RowReductionOperation::Inspect => return Ok(0),
        RowReductionOperation::Inverse(value) => {
            let (n, d, b, degree) = shape(value)?;
            (add(n, d)?, b, degree)
        }
        RowReductionOperation::Multiply(a, b) => {
            let (an, ad, ab, ae) = shape(a)?;
            let (bn, bd, bb, be) = shape(b)?;
            let terms = add(mul(an, bn)?, mul(ad, bd)?)?;
            (
                terms,
                add(add(ab, bb)?, terms.max(1).ilog2() as usize + 2)?,
                add(ae, be)?,
            )
        }
        RowReductionOperation::SubtractProduct(a, b, c) => {
            let (an, ad, ab, ae) = shape(a)?;
            let (bn, bd, bb, be) = shape(b)?;
            let (cn, cd, cb, ce) = shape(c)?;
            let terms = add(
                add(mul(mul(an, bd)?, cd)?, mul(mul(bn, cn)?, ad)?)?,
                mul(mul(ad, bd)?, cd)?,
            )?;
            (
                terms,
                add(add(add(ab, bb)?, cb)?, terms.max(1).ilog2() as usize + 3)?,
                add(add(ae, be)?, ce)?,
            )
        }
    };
    if degree > usize::from(u16::MAX) {
        return Err(MathError::Limit("exact arithmetic exponent growth"));
    }
    if terms > limits.terms || bits > limits.coefficient_bits {
        return Err(MathError::Limit("exact arithmetic predicted growth"));
    }
    let variables =
        |value: &RationalPolynomial<Z, u16>| value.numerator.nvars().max(value.denominator.nvars());
    let variables = match operation {
        RowReductionOperation::Inspect => 0,
        RowReductionOperation::Inverse(value) => variables(value),
        RowReductionOperation::Multiply(a, b) => variables(a).max(variables(b)),
        RowReductionOperation::SubtractProduct(a, b, c) => {
            variables(a).max(variables(b)).max(variables(c))
        }
    };
    let term_bytes = add(
        add(bits.div_ceil(8), 128)?,
        mul(variables, size_of::<u16>())?,
    )?;
    let bytes = mul(mul(terms, term_bytes)?, 64)?;
    if bytes > limits.bytes {
        return Err(MathError::Limit("exact arithmetic predicted bytes"));
    }
    Ok(bytes)
}

fn coordinate_interval_permitted(
    variable: &crate::binding::Variable,
    lower: &Rational,
    upper: &Rational,
) -> Result<bool, MathError> {
    use pse_model::generated::enums::ModelingVariableDomain as Domain;
    if matches!(
        variable.domain,
        Domain::Semicontinuous | Domain::Semiinteger
    ) && lower.is_zero()
        && upper.is_zero()
    {
        return Ok(true);
    }
    if let Some(bound) = variable.lower.filter(|bound| bound.is_finite())
        && lower
            < &Rational::try_from(bound).map_err(|error| MathError::Library(error.to_string()))?
    {
        return Ok(false);
    }
    if let Some(bound) = variable.upper.filter(|bound| bound.is_finite())
        && upper
            > &Rational::try_from(bound).map_err(|error| MathError::Library(error.to_string()))?
    {
        return Ok(false);
    }
    Ok(match variable.domain {
        Domain::Integer | Domain::Semiinteger => lower == upper && lower.is_integer(),
        Domain::Binary => lower == upper && (lower.is_zero() || lower.is_one()),
        Domain::Continuous | Domain::Semicontinuous => true,
    })
}

fn permits(condition: Condition, value: &Rational) -> bool {
    match condition {
        Condition::Nonzero => !value.is_zero(),
        Condition::Positive => !value.is_negative() && !value.is_zero(),
        Condition::Nonnegative => !value.is_negative(),
    }
}
fn estimate<T: Real + SingleFloat>(
    value: T,
    absolute_error: Option<f64>,
) -> Result<NumericalDiagnosticValue, MathError> {
    if !value.is_finite() || absolute_error.is_some_and(|error| !error.is_finite()) {
        return Err(MathError::CoefficientRange);
    }
    Ok(NumericalDiagnosticValue {
        value: Some(format!("{value:e}")),
        absolute_error,
        lower: None,
        upper: None,
        unbounded: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assembly::AssemblyLimits,
        binding::{
            CaseLimits, CaseStructure, Contribution, InstanceBinding, Row, SlotBinding, Target,
            Variable,
        },
        typed::{Binary, BodyBuilder, BodyLimits, TypedValue},
    };
    use pse_kernels::{DerivativeOrder, Port};
    use pse_model::generated::enums::ModelingVariableDomain;
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    fn id(value: u8) -> SemanticId {
        SemanticId::from_bytes([value; 16])
    }
    fn scope() -> ExecutionScope {
        ExecutionScope::new(pse_diagnostics::Cancellation::default(), None)
    }
    fn point(coordinate: u8, value: i64) -> DiagnosticInput {
        DiagnosticInput {
            coordinate: id(coordinate),
            lower: ExactValue::integer(value),
            upper: None,
        }
    }
    fn plan(
        inputs: usize,
        outputs: impl FnOnce(&mut BodyBuilder<'_>, &[TypedValue]) -> Vec<TypedValue>,
    ) -> CasePlan {
        plan_bounded(inputs, false, outputs)
    }
    fn plan_bounded(
        inputs: usize,
        nonnegative: bool,
        outputs: impl FnOnce(&mut BodyBuilder<'_>, &[TypedValue]) -> Vec<TypedValue>,
    ) -> CasePlan {
        let registry = standard_registry().unwrap();
        let quantity = ids::quantity("neutral");
        let port = |index| Port {
            id: id(index),
            quantity,
            unit: registry.quantity_type(quantity).unwrap().canonical_unit,
        };
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            inputs,
            BodyLimits::default(),
        )
        .unwrap();
        let values = (0..inputs)
            .map(|index| {
                builder
                    .input(index, quantity, IndexSet::new(), id(index as u8 + 30))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let outputs = outputs(&mut builder, &values);
        let body = Arc::new(builder.prepare(&outputs).unwrap());
        let body_id = ContentHash::from_bytes([7; 32]);
        let structure = CaseStructure::new(
            (0..inputs.min(2))
                .map(|index| Variable {
                    port: port(index as u8 + 1),
                    fixed: false,
                    domain: ModelingVariableDomain::Continuous,
                    lower: (index == 0 && nonnegative).then_some(0.0),
                    upper: None,
                })
                .collect(),
            (2..inputs).map(|index| port(index as u8 + 1)).collect(),
            vec![InstanceBinding {
                instance: id(20),
                body: body_id,
                slots: (0..inputs)
                    .map(|index| {
                        SlotBinding::new(&port(index as u8 + 1), &port(index as u8 + 1), &registry)
                            .unwrap()
                    })
                    .collect::<Vec<_>>()
                    .into(),
                contributions: (0..outputs.len())
                    .map(|output| Contribution {
                        output,
                        target: Target::Row(id(output as u8 + 10)),
                        scale: 1.0,
                    })
                    .collect(),
                occurrences: Default::default(),
                checked_sources: Default::default(),
                checked_members: Default::default(),
            }],
            (0..outputs.len())
                .map(|output| Row {
                    id: id(output as u8 + 10),
                    quantity,
                    lower: 0.0,
                    upper: 0.0,
                })
                .collect(),
            None,
            CaseLimits::default(),
        )
        .unwrap();
        CasePlan::prepare(
            Arc::new(structure),
            BTreeMap::from([(body_id, body)]),
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
    }
    fn request(
        claim: DiagnosticClaim,
        rows: usize,
        unknowns: usize,
        parameters: usize,
    ) -> DiagnosticRequest {
        DiagnosticRequest {
            rows: (0..rows).map(|index| id(index as u8 + 10)).collect(),
            unknowns: (0..unknowns).map(|index| id(index as u8 + 1)).collect(),
            parameters: (unknowns..unknowns + parameters)
                .map(|index| id(index as u8 + 1))
                .collect(),
            inputs: vec![],
            derivatives: vec![],
            claim,
            precision: 160,
            limits: DiagnosticLimits::default(),
        }
    }
    fn expression(value: Atom, template: &TypedValue) -> TypedValue {
        TypedValue::symbolic_fixture(value, template.physical_contract().clone(), id(90))
    }
    #[test]
    fn exact_diagnostic_accounting_specializations_and_original_row_witnesses() {
        let plan = plan_bounded(4, true, |_, input| {
            let [x, y, p, q] = input else { unreachable!() };
            vec![
                expression(
                    x.atom.as_ref().unwrap() + y.atom.as_ref().unwrap() - Atom::num(1),
                    x,
                ),
                expression(
                    x.atom.as_ref().unwrap() + p.atom.as_ref().unwrap() * y.atom.as_ref().unwrap()
                        - q.atom.as_ref().unwrap(),
                    x,
                ),
            ]
        });
        let prepared = plan
            .prepare_mathematical_diagnostic(
                &CaseValues::default(),
                request(DiagnosticClaim::ExactIdentity, 2, 2, 2),
                None,
                &scope(),
            )
            .unwrap();
        let generic = prepared.evaluate(&scope()).unwrap().exact.unwrap();
        assert_eq!(generic.rank, 2);
        assert!(!generic.nonzero_conditions.is_empty());
        let unique = prepared
            .evaluate_with_inputs(&[point(3, 2), point(4, 3)], &scope())
            .unwrap()
            .exact
            .unwrap();
        assert_eq!(unique.consistency, ExactConsistency::Unique);
        assert_eq!(
            unique.recovered,
            vec![(id(1), "-1".into()), (id(2), "2".into())]
        );
        assert_eq!(unique.original_domain_valid, Some(true));
        assert_eq!(unique.physical_bounds_valid, Some(false));
        assert!(!unique.reduced_evaluation);
        assert!(unique.used_reduced_product);
        let feasible = prepared
            .evaluate_with_inputs(&[point(3, 2), point(4, 0)], &scope())
            .unwrap()
            .exact
            .unwrap();
        assert!(feasible.used_reduced_product && feasible.reduced_evaluation);
        assert_eq!(
            feasible.recovered,
            vec![(id(1), "2".into()), (id(2), "-1".into())]
        );
        let redundant = prepared
            .evaluate_with_inputs(&[point(3, 1), point(4, 1)], &scope())
            .unwrap()
            .exact
            .unwrap();
        assert_eq!((redundant.rank, redundant.augmented_rank), (1, 1));
        assert_eq!(redundant.consistency, ExactConsistency::Underdetermined);
        assert!(!redundant.used_reduced_product);
        assert_eq!(
            redundant.row_witnesses[0].original_rows,
            vec![id(10), id(11)]
        );
        assert_eq!(redundant.row_witnesses[0].multipliers, vec!["-1", "1"]);
        let inconsistent = prepared
            .evaluate_with_inputs(&[point(3, 1), point(4, 2)], &scope())
            .unwrap()
            .exact
            .unwrap();
        assert_eq!((inconsistent.rank, inconsistent.augmented_rank), (1, 2));
        assert_eq!(inconsistent.consistency, ExactConsistency::Inconsistent);
        assert!(inconsistent.row_witnesses[0].inconsistent);
        assert!(!inconsistent.used_reduced_product);
    }
    #[test]
    fn exact_diagnostic_preserves_canceled_denominator_and_limits() {
        let plan = plan(1, |builder, input| {
            let quotient = builder
                .binary(
                    Binary::Div,
                    input[0].clone(),
                    input[0].clone(),
                    None,
                    id(80),
                )
                .unwrap();
            let sum = builder
                .binary(Binary::Add, quotient, input[0].clone(), None, id(81))
                .unwrap();
            let one = expression(Atom::num(1), &input[0]);
            vec![builder.binary(Binary::Sub, sum, one, None, id(82)).unwrap()]
        });
        let mut request = request(DiagnosticClaim::ExactIdentity, 1, 1, 0);
        let result = plan
            .mathematical_diagnostics(&CaseValues::default(), request.clone(), None, &scope())
            .unwrap();
        assert!(!result.original_conditions.is_empty());
        assert_eq!(result.exact.unwrap().original_domain_valid, Some(false));
        request.limits.work = 1;
        assert!(
            plan.mathematical_diagnostics(&CaseValues::default(), request, None, &scope())
                .is_err()
        );
        let expired = ExecutionScope::new(
            pse_diagnostics::Cancellation::default(),
            Some(Instant::now() - Duration::from_secs(1)),
        );
        assert!(
            plan.mathematical_diagnostics(
                &CaseValues::default(),
                self::request(DiagnosticClaim::ExactIdentity, 1, 1, 0),
                None,
                &expired
            )
            .is_err()
        );
    }
    #[test]
    fn diagnostic_precision_and_ball_claims_remain_distinct() {
        let plan = plan(1, |_, input| {
            vec![expression(
                input[0].atom.as_ref().unwrap().pow(2)
                    + input[0].atom.as_ref().unwrap() * Atom::num(2),
                &input[0],
            )]
        });
        for claim in [
            DiagnosticClaim::Ordinary,
            DiagnosticClaim::DoubleFloat,
            DiagnosticClaim::ChosenPrecision,
            DiagnosticClaim::PropagatedError,
        ] {
            let mut request = request(claim, 1, 1, 0);
            request.inputs = vec![point(1, 3)];
            let prepared = plan
                .prepare_mathematical_diagnostic(&CaseValues::default(), request, None, &scope())
                .unwrap();
            for x in [2, 3] {
                let result = prepared
                    .evaluate_with_inputs(&[point(1, x)], &scope())
                    .unwrap();
                let numeric = &result.numerical[0];
                assert!(numeric.lower.is_none() && numeric.upper.is_none());
                assert_eq!(
                    numeric.absolute_error.is_some(),
                    claim == DiagnosticClaim::PropagatedError
                );
                assert!(!numeric.unbounded);
            }
        }
        let mut request = request(DiagnosticClaim::ValidatedEnclosure, 1, 1, 0);
        request.inputs = vec![DiagnosticInput {
            coordinate: id(1),
            lower: ExactValue::integer(-2),
            upper: Some(ExactValue::integer(3)),
        }];
        assert!(
            plan.mathematical_diagnostics(&CaseValues::default(), request.clone(), None, &scope())
                .is_err()
        );
        // SAFETY: this fixture owns precisely the original basic arithmetic expression.
        #[allow(
            unsafe_code,
            reason = "test fixture owns the entire admitted arithmetic source"
        )]
        let admission = unsafe {
            ArithmeticSourceAdmission::issue(plan.structure().key(), request.rows.clone())
        };
        let result = plan
            .mathematical_diagnostics(&CaseValues::default(), request, Some(&admission), &scope())
            .unwrap();
        let ball = &result.numerical[0];
        let ratio = |text: &str| {
            let (n, d) = text.split_once('/').unwrap_or((text, "1"));
            Rational::new(n.parse::<Integer>().unwrap(), d.parse::<Integer>().unwrap())
        };
        // x^2+2x has analytic extrema -1 and 15 on [-2,3].
        assert!(ratio(ball.lower.as_ref().unwrap()) <= Rational::from(-1));
        assert!(ratio(ball.upper.as_ref().unwrap()) >= Rational::from(15));
        assert!(ball.value.is_none() && ball.absolute_error.is_none());
    }
    #[test]
    fn diagnostic_delivers_original_rows_and_ordered_native_partials() {
        let plan = plan(2, |_, input| {
            vec![
                expression(
                    input[0].atom.as_ref().unwrap().pow(2)
                        + input[0].atom.as_ref().unwrap() * Atom::num(2),
                    &input[0],
                ),
                input[1].clone(),
            ]
        });
        let mut request = request(DiagnosticClaim::ChosenPrecision, 1, 1, 0);
        request.inputs = vec![point(1, 3)];
        request.derivatives = vec![vec![id(1)], vec![id(1), id(1)]];
        let result = plan
            .mathematical_diagnostics(&CaseValues::default(), request, None, &scope())
            .unwrap();
        assert_eq!(
            result
                .numerical
                .iter()
                .map(|value| value.value.as_ref().unwrap().parse::<f64>().unwrap())
                .collect::<Vec<_>>(),
            vec![15.0, 8.0, 2.0]
        );
        assert_eq!(
            result
                .outputs
                .iter()
                .map(|output| output.derivatives.clone())
                .collect::<Vec<_>>(),
            vec![vec![], vec![id(1)], vec![id(1), id(1)]]
        );
        assert!(result.outputs.iter().all(|output| output.row == id(10)));
        assert_eq!(result.mathematical_views.len(), 3);
    }
    #[test]
    fn diagnostic_original_domain_guards_follow_actual_derivative_demand() {
        let plan = plan(1, |builder, input| {
            vec![
                builder
                    .unary(crate::Function::Sqrt, input[0].clone(), id(91))
                    .unwrap(),
            ]
        });
        let mut request = request(DiagnosticClaim::ChosenPrecision, 1, 1, 0);
        request.inputs = vec![point(1, 0)];
        let value = plan
            .mathematical_diagnostics(&CaseValues::default(), request.clone(), None, &scope())
            .unwrap();
        assert_eq!(
            value.numerical[0]
                .value
                .as_ref()
                .unwrap()
                .parse::<f64>()
                .unwrap(),
            0.0
        );
        request.derivatives = vec![vec![id(1)]];
        assert!(
            plan.mathematical_diagnostics(&CaseValues::default(), request, None, &scope())
                .is_err()
        );
    }
    #[test]
    fn diagnostic_chosen_precision_retains_small_residual_lost_to_binary64() {
        let plan = plan(1, |_, input| {
            vec![expression(
                input[0].atom.as_ref().unwrap().pow(2) - Atom::num(2),
                &input[0],
            )]
        });
        // x=1+2^-80: x^2-2 is -1+2^-79+2^-160. Chosen precision must
        // distinguish the exact residual from the binary64-rounded input result -1.
        let denominator = Integer::from(2).pow(80);
        let mut request = request(DiagnosticClaim::ChosenPrecision, 1, 1, 0);
        request.inputs = vec![DiagnosticInput {
            coordinate: id(1),
            lower: ExactValue {
                numerator: (&denominator + Integer::from(1)).to_string(),
                denominator: denominator.to_string(),
            },
            upper: None,
        }];
        let high = plan
            .mathematical_diagnostics(&CaseValues::default(), request.clone(), None, &scope())
            .unwrap();
        let text = high.numerical[0].value.as_ref().unwrap();
        let actual = Float::parse(text, Some(160)).unwrap();
        assert!(actual > Float::from(-1.0));
        request.claim = DiagnosticClaim::Ordinary;
        let ordinary = plan
            .mathematical_diagnostics(&CaseValues::default(), request, None, &scope())
            .unwrap();
        assert_eq!(
            ordinary.numerical[0]
                .value
                .as_ref()
                .unwrap()
                .parse::<f64>()
                .unwrap(),
            -1.0
        );
    }
    #[test]
    fn diagnostic_precision_maps_fixed_builtin_constants_at_actual_requested_precision() {
        crate::initialize().unwrap();
        let formal = library::formal(0).unwrap();
        let expression = &formal + Atom::num(1).sin();
        let scope = scope();
        let product = library::admission::diagnostic_product(
            &[expression],
            &[formal],
            Optimization::default(),
            &scope,
            EvaluationLimits::default(),
        )
        .unwrap();
        let mut evaluator = product
            .evaluator
            .try_map_coeff_with_prec(
                &|value| Ok(value.re.to_multi_prec_float(160)),
                160,
                &mut control(&scope, DiagnosticLimits::default()),
            )
            .unwrap();
        let mut output = vec![Float::new(160)];
        evaluator
            .try_evaluate_with(
                &[Float::new(160)],
                &mut output,
                &mut control(&scope, DiagnosticLimits::default()),
            )
            .unwrap();
        let realized = output[0].to_rational();
        // Independent alternating Taylor bounds for sin(1). These bounds are much
        // narrower than binary64 error and do not use the native evaluator or balls.
        let mut sum = Rational::zero();
        let mut term = Rational::one();
        let mut lower = Rational::zero();
        let mut upper = Rational::one();
        for index in 0..=14 {
            sum += &term;
            if index == 13 {
                lower = sum.clone();
            }
            if index == 14 {
                upper = sum.clone();
            }
            term = -term / Rational::from(((2 * index + 2) * (2 * index + 3)) as i64);
        }
        assert!(lower < realized && realized < upper);
    }

    #[test]
    fn diagnostic_admission_refuses_coordinate_and_exact_binding_extent_errors() {
        let plan = plan_bounded(1, true, |_, input| vec![input[0].clone()]);
        let mut demand = request(DiagnosticClaim::ChosenPrecision, 1, 1, 0);
        demand.inputs = vec![point(1, -1)];
        assert!(
            plan.mathematical_diagnostics(&CaseValues::default(), demand.clone(), None, &scope())
                .is_err()
        );
        demand.unknowns.push(id(1));
        assert!(
            plan.prepare_mathematical_diagnostic(
                &CaseValues::default(),
                demand.clone(),
                None,
                &scope()
            )
            .is_err()
        );
        demand.unknowns = vec![id(99)];
        assert!(
            plan.prepare_mathematical_diagnostic(&CaseValues::default(), demand, None, &scope())
                .is_err()
        );
        let parameter_plan = self::plan(3, |_, input| vec![input[2].clone()]);
        let mut wrongly_bound = request(DiagnosticClaim::ChosenPrecision, 1, 1, 0);
        wrongly_bound.unknowns = vec![id(3)];
        wrongly_bound.inputs = vec![point(3, 2)];
        assert!(parameter_plan.prepare_mathematical_diagnostic(&CaseValues::default(), wrongly_bound.clone(), None, &scope()).is_err());
        wrongly_bound.unknowns.clear();
        wrongly_bound.parameters = vec![id(3)];
        assert!(parameter_plan.mathematical_diagnostics(&CaseValues::default(), wrongly_bound, None, &scope()).is_ok());
        let mut limits = DiagnosticLimits::default();
        limits.coefficient_bits = 16;
        let large = ExactValue {
            numerator: "1".repeat(1_000),
            denominator: "1".into(),
        };
        assert!(matches!(
            large.rational(limits),
            Err(MathError::Limit("diagnostic rational input"))
        ));
        let formal = library::formal(0).unwrap();
        let expression = formal.pow(100_000);
        let substitutions = BTreeMap::from([(symbol(&formal).unwrap(), Atom::num(2))]);
        assert!(matches!(
            admit_substitution(
                &expression,
                &substitutions,
                DiagnosticLimits::default(),
                &scope()
            ),
            Err(MathError::Limit("exact substitution coefficient bits"))
        ));
    }

    #[test]
    fn diagnostic_native_enclosure_refuses_unsupported_and_singular_programs() {
        crate::initialize().unwrap();
        let formal = library::formal(0).unwrap();
        for expression in [
            formal.sin(),
            formal.log(),
            formal.pow(Atom::num(Rational::from((1, 2)))),
        ] {
            let product = library::admission::diagnostic_product(
                &[expression],
                &[formal.clone()],
                Optimization::default(),
                &scope(),
                EvaluationLimits::default(),
            )
            .unwrap();
            assert!(
                product
                    .evaluator
                    .try_map_real_ball_with(
                        100,
                        &mut control(&scope(), DiagnosticLimits::default())
                    )
                    .is_err()
            );
        }
        let product = library::admission::diagnostic_product(
            &[formal.pow(-1)],
            &[formal],
            Optimization::default(),
            &scope(),
            EvaluationLimits::default(),
        )
        .unwrap();
        let mut evaluator = product
            .evaluator
            .try_map_real_ball_with(100, &mut control(&scope(), DiagnosticLimits::default()))
            .unwrap();
        let mut out = vec![RealBall::from_rational_bounds(
            &Rational::zero(),
            &Rational::zero(),
            100,
        )];
        evaluator
            .try_evaluate_with(
                &[RealBall::from_rational_bounds(
                    &Rational::from(-1),
                    &Rational::one(),
                    100,
                )],
                &mut out,
                &mut control(&scope(), DiagnosticLimits::default()),
            )
            .unwrap();
        assert!(!out[0].is_finite());
        evaluator
            .try_evaluate_with(
                &[RealBall::from(2.0)],
                &mut out,
                &mut control(&scope(), DiagnosticLimits::default()),
            )
            .unwrap();
        assert_eq!(out[0].lower_bound().to_rational(), Rational::from((1, 2)));
        assert_eq!(out[0].upper_bound().to_rational(), Rational::from((1, 2)));
        out[0] = RealBall::from(73.0);
        let cancelled = scope();
        cancelled.cancellation_control().cancel();
        assert!(
            evaluator
                .try_evaluate_with(
                    &[RealBall::from(2.0)],
                    &mut out,
                    &mut control(&cancelled, DiagnosticLimits::default())
                )
                .is_err()
        );
        assert_eq!(out[0].lower_bound().to_rational(), Rational::from(73));
        assert_eq!(out[0].upper_bound().to_rational(), Rational::from(73));
    }
}
