// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected exact mathematical observations. Physical/source selection belongs to the
//! compiler; this owner retains the original guarded body and never parses formulas.
use crate::{
    MathError,
    guarded::{Condition, PreparedBody, Stage},
    jets::EvaluationLimits,
};
use pse_ids::SemanticId;
use pse_kernels::{DerivativeOrder, ExecutionScope};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub use symbolica::poly::series::SeriesDepth;
use symbolica::{
    atom::{Atom, AtomCore, AtomView},
    domains::rational::Rational,
    poly::series::SeriesPrecision,
    printer::PrintOptions,
};

/// Output text is a native mathematical view, never a codec or identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MathematicalView {
    /// Namespace-preserving native Symbolica file notation.
    pub symbolica: String,
    /// Namespace-preserving native LaTeX presentation.
    pub latex: String,
    /// Namespace-preserving native Typst presentation.
    pub typst: String,
}
/// Exact rational exponent/order, with arbitrary precision decimal integer components.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RationalView {
    /// Exact numerator or its native mathematical view.
    pub numerator: String,
    /// Exact nonzero denominator or its native mathematical view.
    pub denominator: String,
}
impl From<&Rational> for RationalView {
    fn from(value: &Rational) -> Self {
        Self {
            numerator: value.numerator_ref().to_string(),
            denominator: value.denominator_ref().to_string(),
        }
    }
}
/// The request's formal depth and the returned native remainder are separate facts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SeriesObservation {
    /// Original independent input ordinal.
    pub coordinate: usize,
    /// Original exact expansion point.
    pub expansion_point: MathematicalView,
    /// Requested truncation depth, independently of the returned remainder.
    pub requested_depth: RationalView,
    /// Whether the requested depth was relative to the leading term.
    pub relative_depth: bool,
    /// Native relative remainder order; absent for infinite precision.
    pub relative_order: Option<RationalView>,
    /// Native absolute remainder order; absent for infinite precision.
    pub absolute_order: Option<RationalView>,
    /// Native leading exponent, before the remainder.
    pub trailing_exponent: RationalView,
    /// Native common denominator of the represented exponents.
    pub ramification: usize,
    /// Exact polynomial termination, established from the original expression.
    pub exactly_terminating: bool,
    /// Native exact terms, including their original exponents.
    pub coefficients: Vec<SeriesCoefficient>,
    /// Native polynomial part, without interpreting the remainder as zero.
    pub polynomial: MathematicalView,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// One owned term, index label or original source correspondence.
pub struct SeriesCoefficient {
    /// Exact exponent of this term.
    pub exponent: RationalView,
    /// Exact native coefficient.
    pub coefficient: MathematicalView,
}
/// Exact normalization is distinct from an empirical applicability assertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IdentityMeaning {
    /// Exact normalization established zero.
    Zero,
    /// Exact rational normalization established a nonzero rational function.
    Nonzero,
    /// Native normalization did not establish either conclusion.
    Inconclusive,
}
/// Owned bounded observations that a producer may persist under original result custody.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnalyticalObservation {
    /// Repeated exact differentiation in original coordinate order.
    Derivative {
        /// Ordered original independent input ordinals.
        coordinates: Vec<usize>,
        /// Native exact mathematical result.
        result: MathematicalView,
    },
    /// Formal native series with explicit precision and remainder meaning.
    Series(SeriesObservation),
    /// Exact normalization comparison under the original premises.
    Identity {
        /// Strength of the exact normalization conclusion.
        meaning: IdentityMeaning,
        /// Native normalized difference; no empirical applicability assertion.
        difference: MathematicalView,
    },
    /// A changed mathematical approximation; no interval error or execution permission.
    RationalApproximation {
        /// Original independent input ordinal.
        coordinate: usize,
        /// Original exact expansion point.
        expansion_point: MathematicalView,
        /// Requested maximum numerator degree.
        numerator_degree: u32,
        /// Requested maximum denominator degree.
        denominator_degree: u32,
        /// Exact numerator or its native mathematical view.
        numerator: MathematicalView,
        /// Exact nonzero denominator or its native mathematical view.
        denominator: MathematicalView,
        /// Order through which the formal congruence was checked.
        formal_order: RationalView,
    },
    /// Original finite contraction and its native canonical form.
    Indexed(IndexedObservation),
}

/// Compiler-issued indexed coordinate; physical external identities stay fixed.
#[derive(Clone, Debug)]
pub enum TensorAxis {
    /// An original bound contraction coordinate.
    Dummy {
        /// Original source binder identity.
        binder: SemanticId,
        /// Typed index-group identity, preserved across compared expressions.
        group: SemanticId,
    },
    /// A fixed external coordinate that canonicalization cannot rename.
    External {
        /// Fixed external physical index identity.
        identity: SemanticId,
        /// Typed index-group identity, preserved across compared expressions.
        group: SemanticId,
    },
}
/// One selected physical family occurrence in the original constitutive expression.
#[derive(Clone, Debug)]
pub struct TensorOccurrence {
    /// Original selected tensor family identity.
    pub family: SemanticId,
    /// Original ordered axes.
    pub axes: Vec<TensorAxis>,
}
/// Complete common dummy alphabet supplied for compared expressions.
#[derive(Clone, Copy, Debug)]
pub struct IndexAlphabet {
    /// Typed index-group identity, preserved across compared expressions.
    pub group: SemanticId,
    /// Number of labels in the complete common dummy alphabet.
    pub labels: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// One owned term, index label or original source correspondence.
pub struct IndexLabel {
    /// Namespace-preserving native input or returned index label.
    pub label: String,
    /// Typed index-group identity, preserved across compared expressions.
    pub group: SemanticId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// One owned term, index label or original source correspondence.
pub struct DummyCorrespondence {
    /// Original source binder identity.
    pub binder: SemanticId,
    /// Original binder correspondence to the supplier input alphabet.
    pub input: IndexLabel,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexedObservation {
    /// Original source context or original mathematical view.
    pub original: MathematicalView,
    /// Native canonical mathematical view.
    pub canonical: MathematicalView,
    /// Returned fixed external indices.
    pub external_indices: Vec<IndexLabel>,
    /// Returned native dummy indices, without a claimed binder permutation.
    pub dummy_indices: Vec<IndexLabel>,
    /// Original binders to the supplier INPUT alphabet, never a canonical permutation.
    pub original_to_input: Vec<DummyCorrespondence>,
}
fn named_atom(name: String) -> Result<Atom, MathError> {
    use symbolica::atom::{NamespacedSymbol, SymbolAttribute, SymbolBuilder};
    let name = NamespacedSymbol::try_from(name.as_str()).map_err(MathError::Library)?;
    SymbolBuilder::new(name)
        .with_attributes(Vec::<SymbolAttribute>::new())
        .build()
        .map(Atom::var)
        .map_err(|error| MathError::Library(error.to_string()))
}
/// Native tensor canonicalization over a compiler-issued finite projection. This
/// comparison says nothing about physical interchangeability of its source instances.
pub fn canonicalize_contraction(
    factors: &[TensorOccurrence],
    alphabets: &[IndexAlphabet],
    limits: EvaluationLimits,
    scope: &ExecutionScope,
) -> Result<AnalyticalResult, MathError> {
    check(scope)?;
    if factors.is_empty() || factors.len() > limits.operations {
        return Err(MathError::Limit("indexed analytical factor extent"));
    }
    let mut pools = BTreeMap::<SemanticId, Vec<Atom>>::new();
    for alphabet in alphabets {
        if alphabet.labels == 0
            || alphabet.labels > limits.derivative_components
            || pools.contains_key(&alphabet.group)
        {
            return Err(MathError::Contract(
                "invalid complete dummy index alphabet".into(),
            ));
        }
        let labels = (0..alphabet.labels)
            .map(|ordinal| named_atom(format!("pse_math_index::g{}_d{ordinal}", alphabet.group)))
            .collect::<Result<Vec<_>, _>>()?;
        pools.insert(alphabet.group, labels);
    }
    let mut mapping = BTreeMap::<(SemanticId, SemanticId), Atom>::new();
    let mut original_mapping = BTreeMap::<Atom, Atom>::new();
    let mut correspondence = Vec::new();
    let mut indices = Vec::<(Atom, SemanticId)>::new();
    for (group, pool) in &pools {
        indices.extend(pool.iter().cloned().map(|label| (label, *group)));
    }
    let mut original = Atom::num(1);
    for factor in factors {
        let head = named_atom(format!("pse_math_tensor::f{}", factor.family))?
            .get_symbol()
            .ok_or_else(|| MathError::Contract("tensor family symbol".into()))?;
        let mut call = symbolica::atom::FunctionBuilder::new(head);
        for axis in &factor.axes {
            let index = match axis {
                TensorAxis::Dummy { binder, group } => {
                    let key = (*binder, *group);
                    let input = if let Some(input) = mapping.get(&key) {
                        input.clone()
                    } else {
                        let ordinal = mapping
                            .keys()
                            .filter(|(_, candidate)| candidate == group)
                            .count();
                        let input = pools
                            .get(group)
                            .and_then(|pool| pool.get(ordinal))
                            .ok_or_else(|| {
                                MathError::Contract(
                                    "dummy group exceeds supplied common index alphabet".into(),
                                )
                            })?
                            .clone();
                        mapping.insert(key, input.clone());
                        correspondence.push(DummyCorrespondence {
                            binder: *binder,
                            input: IndexLabel {
                                label: input.printer(PrintOptions::file()).to_string(),
                                group: *group,
                            },
                        });
                        input
                    };
                    let original = named_atom(format!("pse_math_original_index::b{}", binder))?;
                    original_mapping.insert(original.clone(), input);
                    original
                }
                TensorAxis::External { identity, group } => {
                    let index = named_atom(format!("pse_math_external_index::e{}", identity))?;
                    if !indices.iter().any(|(existing, _)| existing == &index) {
                        indices.push((index.clone(), *group));
                    }
                    index
                }
            };
            call = call.add_arg(index);
        }
        original *= call.finish();
    }
    bounded(&original, limits, scope)?;
    let input = original.replace_map(|node, _, out| {
        if let Some(input) = original_mapping.get(&node.to_owned()) {
            out.set_from_view(&input.as_view());
        }
    });
    let canonical = input
        .canonize_tensors(indices)
        .map_err(|error| MathError::Library(error.to_string()))?;
    bounded(&canonical.canonical_form, limits, scope)?;
    let labels = |indices: Vec<(Atom, SemanticId)>| {
        indices
            .into_iter()
            .map(|(atom, group)| IndexLabel {
                label: atom.printer(PrintOptions::file()).to_string(),
                group,
            })
            .collect()
    };
    let observation = IndexedObservation {
        original: mathematical_view(&original, limits, scope)?,
        canonical: mathematical_view(&canonical.canonical_form, limits, scope)?,
        external_indices: labels(canonical.external_indices),
        dummy_indices: labels(canonical.dummy_indices),
        original_to_input: correspondence,
    };
    Ok(AnalyticalResult {
        observation: AnalyticalObservation::Indexed(observation),
        expression: canonical.canonical_form,
    })
}
/// Original native domain/control premise, retained independently of normalized math.
#[derive(Clone, Debug)]
pub struct Requirement {
    /// Original domain argument.
    pub argument: Atom,
    /// Original domain condition.
    pub condition: Condition,
    /// Original derivative order governed by this condition.
    pub order: DerivativeOrder,
    /// Original typed condition source.
    pub source: SemanticId,
}
/// Original typed guard context is observation only; it grants no admission on readback.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GuardObservation {
    /// Original numeric domain requirement.
    Requirement {
        /// Original typed condition source.
        source: SemanticId,
        /// Original domain condition.
        condition: String,
        /// Original governed derivative order.
        derivative_order: String,
        /// Original domain argument.
        argument: MathematicalView,
    },
    /// Original typed domain lineage.
    Domain {
        /// Original source context or original mathematical view.
        original: serde_json::Value,
    },
    /// Original empirical applicability plan.
    Applicability {
        /// Original source context or original mathematical view.
        original: serde_json::Value,
    },
}
/// A selected pure mathematical output with its complete original guard owner.
#[derive(Clone, Debug)]
pub struct SelectedExpression {
    /// Original source context or original mathematical view.
    original: PreparedBody,
    output: usize,
    expression: Atom,
    requirements: Vec<Requirement>,
    /// Native alias definitions preserve source sharing until an operation needs an Atom.
    definitions: Vec<(Atom, Atom)>,
    guard_context: Vec<GuardObservation>,
}
/// A native mathematical answer and its owned publication projection.
#[derive(Clone, Debug)]
pub struct AnalyticalResult {
    /// Owned serializable observation, separate from native execution identity.
    pub observation: AnalyticalObservation,
    expression: Atom,
}
impl AnalyticalResult {
    /// Borrow the retained native mathematical expression.
    pub fn expression(&self) -> &Atom {
        &self.expression
    }
    /// Report retained native expression storage.
    pub fn retained_bytes(&self) -> usize {
        self.expression.as_view().get_byte_size() + size_of::<Self>()
    }
}
fn check(scope: &ExecutionScope) -> Result<(), MathError> {
    scope.check().map_err(crate::error::scope_error)
}
fn bounded(atom: &Atom, limits: EvaluationLimits, scope: &ExecutionScope) -> Result<(), MathError> {
    check(scope)?;
    limits.check()?;
    let mut nodes = 0usize;
    let mut exceeded = false;
    atom.visitor(&mut |_| {
        nodes = nodes.saturating_add(1);
        exceeded |= nodes > limits.operations;
        !exceeded
    });
    if exceeded {
        return Err(MathError::Limit("analytical expression operations"));
    }
    if atom.as_view().get_byte_size() > limits.scratch_bytes {
        return Err(MathError::Limit("analytical expression bytes"));
    }
    check(scope)
}
/// Native printers with namespaces retained; callers own physical labels and text layout.
pub fn mathematical_view(
    atom: &Atom,
    limits: EvaluationLimits,
    scope: &ExecutionScope,
) -> Result<MathematicalView, MathError> {
    bounded(atom, limits, scope)?;
    let options = |mut options: PrintOptions| {
        options.hide_all_namespaces = false;
        options
    };
    let view = MathematicalView {
        symbolica: atom.printer(PrintOptions::file()).to_string(),
        latex: atom.printer(options(PrintOptions::latex())).to_string(),
        typst: atom.printer(options(PrintOptions::typst())).to_string(),
    };
    let bytes = view
        .symbolica
        .len()
        .checked_add(view.latex.len())
        .and_then(|n| n.checked_add(view.typst.len()))
        .ok_or(MathError::Limit("analytical presentation bytes"))?;
    if bytes > limits.scratch_bytes {
        return Err(MathError::Limit("analytical presentation bytes"));
    }
    crate::construction::additional(bytes)?;
    check(scope)?;
    Ok(view)
}
/// Replace only project-owned exact stable mathematical heads. Their numerical profile
/// and every original domain obligation remain on `SelectedExpression::original_body`.
fn exact_heads(atom: &Atom) -> Result<Atom, MathError> {
    let mut unsupported = false;
    let result = atom.replace_map_bottom_up(|node, _, out| {
        let AtomView::Fun(function) = node else {
            return;
        };
        let Some(operation) = crate::stable::identify(function.get_symbol()) else {
            return;
        };
        use crate::stable::Operation;
        let args = function.iter().map(|a| a.to_owned()).collect::<Vec<_>>();
        let value = match (operation, args.as_slice()) {
            (Operation::Log1p, [x]) => (x + Atom::num(1)).log(),
            (Operation::Log1pFirst, [x]) => (x + Atom::num(1)).pow(-1),
            (Operation::Log1pSecond, [x]) => -(x + Atom::num(1)).pow(-2),
            (Operation::LogRatio, [x, r]) => x.log() - r.log(),
            (Operation::LogRatioX, [x, _]) => x.pow(-1),
            (Operation::LogRatioReference, [_, r]) => -r.pow(-1),
            (Operation::LogRatioXX, [x, _]) => -x.pow(-2),
            (Operation::LogRatioXReference, [_, _]) => Atom::Zero,
            (Operation::LogRatioReferenceReference, [_, r]) => r.pow(-2),
            _ => {
                unsupported = true;
                return;
            }
        };
        **out = value;
    });
    if unsupported {
        return Err(MathError::Contract(
            "selected stable head has no exact analytical projection".into(),
        ));
    }
    Ok(result)
}
impl PreparedBody {
    /// Observe one admitted scalar output without reconstructing source or dropping
    /// control barriers. Unsupported whole regions refuse before a result is issued.
    pub fn analytical_expression(
        &self,
        output: usize,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<SelectedExpression, MathError> {
        check(scope)?;
        let stages = self.demanded_stages(&[output])?;
        let mut definitions = Vec::new();
        let mut requirements = Vec::new();
        let mut guard_context = Vec::new();
        fn inspect(
            stages: &[Stage],
            definitions: &mut Vec<(Atom, Atom)>,
            requirements: &mut Vec<Requirement>,
            guard_context: &mut Vec<GuardObservation>,
            limits: EvaluationLimits,
            scope: &ExecutionScope,
        ) -> Result<(), MathError> {
            for stage in stages {
                check(scope)?;
                match stage {
                    Stage::Block {
                        expressions,
                        outputs,
                        ..
                    } => {
                        for (expression, slot) in expressions.iter().zip(outputs) {
                            bounded(expression, limits, scope)?;
                            definitions
                                .push((crate::library::formal(*slot)?, exact_heads(expression)?));
                        }
                    }
                    Stage::Require {
                        argument,
                        condition,
                        order,
                        source,
                        ..
                    } => requirements.push(Requirement {
                        argument: crate::library::formal(*argument)?,
                        condition: *condition,
                        order: *order,
                        source: *source,
                    }),
                    Stage::Domain { lineage, .. } => guard_context.push(GuardObservation::Domain {
                        original: serde_json::to_value(lineage)
                            .map_err(|error| MathError::Library(error.to_string()))?,
                    }),
                    Stage::Applicability { plan, .. } => {
                        guard_context.push(GuardObservation::Applicability {
                            original: serde_json::to_value(plan)
                                .map_err(|error| MathError::Library(error.to_string()))?,
                        })
                    }
                    Stage::Branch { .. } => {
                        return Err(MathError::Contract(
                            "selected analytical whole region contains a branch".into(),
                        ));
                    }
                    Stage::Provider { .. } => {
                        return Err(MathError::Contract(
                            "selected analytical whole region contains a provider".into(),
                        ));
                    }
                }
            }
            Ok(())
        }
        inspect(
            &stages,
            &mut definitions,
            &mut requirements,
            &mut guard_context,
            limits,
            scope,
        )?;
        let expression = self.expression(output).ok_or_else(|| {
            MathError::Contract(
                "selected analytical output exceeds exact scalar projection support".into(),
            )
        })?;
        let expression = exact_heads(expression)?;
        bounded(&expression, limits, scope)?;
        // Original requirements may mention stage results; substitute definitions in
        // original order using the same native simultaneous map used for value aliases.
        let mut aliases = BTreeMap::<Atom, Atom>::new();
        for (lhs, rhs) in &definitions {
            let rhs = rhs.replace_map(|node, _, out| {
                if let Some(value) = aliases.get(&node.to_owned()) {
                    out.set_from_view(&value.as_view());
                }
            });
            bounded(&rhs, limits, scope)?;
            aliases.insert(lhs.clone(), rhs);
        }
        for requirement in &mut requirements {
            if let Some(value) = aliases.get(&requirement.argument) {
                requirement.argument = value.clone();
            }
            guard_context.push(GuardObservation::Requirement {
                source: requirement.source,
                condition: requirement.condition.description().into(),
                derivative_order: format!("{:?}", requirement.order),
                argument: mathematical_view(&requirement.argument, limits, scope)?,
            });
        }
        Ok(SelectedExpression {
            original: self.clone(),
            output,
            expression,
            requirements,
            definitions,
            guard_context,
        })
    }
}
impl SelectedExpression {
    /// Borrow the complete original guarded execution body.
    pub fn original_body(&self) -> &PreparedBody {
        &self.original
    }
    /// Return the original selected output ordinal.
    pub fn output(&self) -> usize {
        self.output
    }
    /// Borrow the retained native mathematical expression.
    pub fn expression(&self) -> &Atom {
        &self.expression
    }
    /// Borrow original numeric domain obligations.
    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }
    /// Borrow native source aliases in original execution order.
    pub fn definitions(&self) -> &[(Atom, Atom)] {
        &self.definitions
    }
    /// Borrow the complete original observation-only guard context.
    pub fn guard_context(&self) -> &[GuardObservation] {
        &self.guard_context
    }
    fn axis(&self, coordinate: usize) -> Result<Atom, MathError> {
        let axis = crate::library::formal(coordinate)?;
        if self
            .original
            .input_slot(
                axis.get_symbol()
                    .ok_or_else(|| MathError::Contract("analytical coordinate symbol".into()))?,
            )
            .is_none()
        {
            return Err(MathError::Contract(
                "analytical coordinate is not an original selected input".into(),
            ));
        }
        Ok(axis)
    }
    /// Differentiate in the requested original coordinate order.
    pub fn derivative(
        &self,
        coordinates: &[usize],
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        if coordinates.is_empty() || coordinates.len() > limits.derivative_components {
            return Err(MathError::Limit("analytical derivative order"));
        }
        let mut expression = self.expression.clone();
        for coordinate in coordinates {
            bounded(&expression, limits, scope)?;
            expression =
                expression.derivative(self.axis(*coordinate)?.get_symbol().ok_or_else(|| {
                    MathError::Contract("analytical derivative coordinate".into())
                })?);
        }
        let result = mathematical_view(&expression, limits, scope)?;
        Ok(AnalyticalResult {
            observation: AnalyticalObservation::Derivative {
                coordinates: coordinates.to_vec(),
                result,
            },
            expression,
        })
    }
    /// Expand using native exact series semantics and retain its actual remainder.
    pub fn series(
        &self,
        coordinate: usize,
        point: &Atom,
        depth: SeriesDepth,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        bounded(&self.expression, limits, scope)?;
        bounded(point, limits, scope)?;
        let requested = match &depth {
            SeriesDepth::Absolute(value) | SeriesDepth::Relative(value) => value,
        };
        if requested.is_negative()
            || requested > &Rational::from(limits.derivative_components as u64)
        {
            return Err(MathError::Limit("analytical series depth"));
        }
        let axis = self.axis(coordinate)?;
        if point.contains(axis.as_view()) {
            return Err(MathError::Contract(
                "analytical expansion point depends on its coordinate".into(),
            ));
        }
        let series = self
            .expression
            .series(
                axis.get_symbol()
                    .ok_or_else(|| MathError::Contract("analytical series coordinate".into()))?,
                point,
                depth.clone(),
            )
            .map_err(|error| MathError::Library(error.to_string()))?;
        let expression = series.to_atom();
        bounded(&expression, limits, scope)?;
        let (relative_order, absolute_order) = match series.precision() {
            SeriesPrecision::Finite { relative, absolute } => (Some(relative), Some(absolute)),
            SeriesPrecision::Infinite => (None, None),
        };
        // Native polynomial conversion leaves nonpolynomial parts as coefficients.
        // Only coordinate-independent coefficients establish exact finite termination.
        let polynomial = self.expression.to_polynomial_in_vars::<u32>(axis.clone());
        let exactly_terminating = polynomial
            .coefficients
            .iter()
            .all(|coefficient| !coefficient.contains(axis.as_view()))
            && absolute_order
                .as_ref()
                .is_none_or(|order| Rational::from(polynomial.degree(0)) < *order);
        let coefficients = series
            .terms()
            .map(|(exponent, coefficient)| {
                Ok(SeriesCoefficient {
                    exponent: (&exponent).into(),
                    coefficient: mathematical_view(coefficient, limits, scope)?,
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let observation = SeriesObservation {
            coordinate,
            expansion_point: mathematical_view(point, limits, scope)?,
            requested_depth: requested.into(),
            relative_depth: matches!(depth, SeriesDepth::Relative(_)),
            relative_order: relative_order.as_ref().map(Into::into),
            absolute_order: absolute_order.as_ref().map(Into::into),
            trailing_exponent: (&series.get_trailing_exponent()).into(),
            ramification: series.get_ramification(),
            exactly_terminating,
            coefficients,
            polynomial: mathematical_view(&expression, limits, scope)?,
        };
        Ok(AnalyticalResult {
            observation: AnalyticalObservation::Series(observation),
            expression,
        })
    }
    /// Normalize the exact difference under both original premise sets.
    pub fn identity(
        &self,
        other: &Self,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        self.identity_derivative(other, &[], limits, scope)
    }
    /// Compare the selected derivative with another original exact output.
    pub fn identity_derivative(
        &self,
        other: &Self,
        coordinates: &[usize],
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        bounded(&self.expression, limits, scope)?;
        bounded(&other.expression, limits, scope)?;
        let left = if coordinates.is_empty() {
            self.expression.clone()
        } else {
            self.derivative(coordinates, limits, scope)?.expression
        };
        let difference = (&left - &other.expression).expand().cancel();
        bounded(&difference, limits, scope)?;
        let mut rational = true;
        difference.visitor(&mut |node| {
            match node {
                AtomView::Fun(_) => rational = false,
                AtomView::Pow(power) if i64::try_from(power.get_exp()).is_err() => rational = false,
                AtomView::Num(number)
                    if matches!(
                        number.get_coeff_view(),
                        symbolica::coefficient::CoefficientView::Float(..)
                    ) =>
                {
                    rational = false
                }
                _ => {}
            }
            true
        });
        let meaning = if difference.is_zero() {
            IdentityMeaning::Zero
        } else if rational {
            IdentityMeaning::Nonzero
        } else {
            IdentityMeaning::Inconclusive
        };
        Ok(AnalyticalResult {
            observation: AnalyticalObservation::Identity {
                meaning,
                difference: mathematical_view(&difference, limits, scope)?,
            },
            expression: difference,
        })
    }
}

#[cfg(test)]
mod tests;
