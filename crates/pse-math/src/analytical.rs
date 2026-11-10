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
pse_contract::projection_struct! {
/// Exact rational exponent/order, with arbitrary precision decimal integer components.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RationalView {
    projection:{name:"AnalyticalRational",rust_path:crate::analytical::RationalView,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Exact numerator or its native mathematical view.
    pub numerator: String => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Exact nonzero denominator or its native mathematical view.
    pub denominator: String => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
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
    /// Opt-in bounded numerical hypothesis with separately retained exact verification.
    IntegerRelation(RelationObservation),
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
/// Native canonicalization and original binder provenance remain separate observations.
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
    let mut fixed = BTreeMap::<Atom, SemanticId>::new();
    let mut occurrences = BTreeMap::<(SemanticId, SemanticId), usize>::new();
    for factor in factors {
        for axis in &factor.axes {
            if let TensorAxis::Dummy { binder, group } = axis {
                *occurrences.entry((*binder, *group)).or_default() += 1;
            }
        }
    }
    if occurrences.values().any(|count| *count != 2) {
        return Err(MathError::Contract(
            "contraction dummy must occur exactly twice".into(),
        ));
    }
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
                    // Fixed physical coordinates are ordinary graph labels, not
                    // renameable native index candidates, even when repeated.
                    fixed.insert(index.clone(), *group);
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
        external_indices: labels(fixed.into_iter().collect()),
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
        self.expression
            .as_view()
            .get_byte_size()
            .saturating_add(size_of::<Self>())
            .saturating_add(owned_projection_extent(&self.observation))
    }
}

/// Conservative owned backing of an already produced, atom-free projection.
/// The counting writer allocates no document; the factor covers tree nodes,
/// geometric string/vector capacity and simultaneous publication projection.
pub fn owned_projection_extent(value: &impl Serialize) -> usize {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(bytes.len());
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    if serde_json::to_writer(&mut counter, value).is_err() {
        return usize::MAX;
    }
    counter.0.saturating_mul(128)
}
// Pre-admit native rational expansion. The supplier then owns normalization;
// this counts a conservative numerator/denominator term envelope, not a second
// polynomial algorithm or a sampled identity test.
fn admit_rational_normalization(
    atom: &Atom,
    limits: EvaluationLimits,
    scope: &ExecutionScope,
) -> Result<(), MathError> {
    fn extent(
        atom: AtomView<'_>,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
        depth: usize,
    ) -> Result<(usize, usize), MathError> {
        check(scope)?;
        if depth > 128 {
            return Err(MathError::Limit("analytical normalization nesting"));
        }
        let mul = |a: usize, b: usize| {
            a.checked_mul(b)
                .filter(|v| *v <= limits.operations)
                .ok_or(MathError::Limit("analytical normalization terms"))
        };
        let value = match atom {
            AtomView::Add(add) => add.iter().try_fold((0usize, 1usize), |(n, d), arg| {
                let (an, ad) = extent(arg, limits, scope, depth + 1)?;
                Ok::<_, MathError>((
                    mul(n, ad)?
                        .checked_add(mul(an, d)?)
                        .filter(|v| *v <= limits.operations)
                        .ok_or(MathError::Limit("analytical normalization terms"))?,
                    mul(d, ad)?,
                ))
            })?,
            AtomView::Mul(product) => product.iter().try_fold((1, 1), |(n, d), arg| {
                let (an, ad) = extent(arg, limits, scope, depth + 1)?;
                Ok::<_, MathError>((mul(n, an)?, mul(d, ad)?))
            })?,
            AtomView::Pow(power) => {
                let (mut n, mut d) = extent(power.get_base(), limits, scope, depth + 1)?;
                let Ok(exponent) = i64::try_from(power.get_exp()) else {
                    return Ok((1, 1));
                };
                if exponent < 0 {
                    std::mem::swap(&mut n, &mut d);
                }
                let degree = u32::try_from(exponent.unsigned_abs())
                    .map_err(|_| MathError::Limit("analytical normalization degree"))?;
                (
                    n.checked_pow(degree)
                        .filter(|v| *v <= limits.operations)
                        .ok_or(MathError::Limit("analytical normalization terms"))?,
                    d.checked_pow(degree)
                        .filter(|v| *v <= limits.operations)
                        .ok_or(MathError::Limit("analytical normalization terms"))?,
                )
            }
            _ => (1, 1),
        };
        Ok(value)
    }
    let (n, d) = extent(atom.as_view(), limits, scope, 0)?;
    let bytes = n
        .checked_add(d)
        .and_then(|terms| terms.checked_mul(atom.as_view().get_byte_size().max(64)))
        .and_then(|bytes| bytes.checked_mul(32))
        .filter(|bytes| *bytes <= limits.scratch_bytes)
        .ok_or(MathError::Limit("analytical normalization backing"))?;
    crate::construction::additional(bytes)?;
    check(scope)
}
fn check(scope: &ExecutionScope) -> Result<(), MathError> {
    scope.check().map_err(crate::error::scope_error)
}
fn operation_control(
    limits: EvaluationLimits,
    scope: &ExecutionScope,
) -> impl FnMut(symbolica::evaluate::OperationExtent) -> Result<(), MathError> {
    let mut work = 0usize;
    let mut admitted = 0usize;
    move |extent| {
        check(scope)?;
        work = work
            .checked_add(extent.work)
            .ok_or(MathError::Limit("analytical native work overflow"))?;
        if work > limits.operations {
            return Err(MathError::Limit("analytical native work"));
        }
        let bytes = extent
            .peak_bytes()
            .ok_or(MathError::Limit("analytical native backing overflow"))?;
        if bytes > limits.scratch_bytes {
            return Err(MathError::Limit("analytical native backing"));
        }
        crate::construction::additional(bytes.saturating_sub(admitted))?;
        admitted = admitted.max(bytes);
        Ok(())
    }
}
fn controlled_series(
    expression: &Atom,
    axis: &Atom,
    point: &Atom,
    depth: SeriesDepth,
    limits: EvaluationLimits,
    scope: &ExecutionScope,
) -> Result<symbolica::poly::series::Series<symbolica::domains::atom::AtomField>, MathError> {
    use symbolica::evaluate::{OperationControl, OperationExtent};
    let mut control = operation_control(limits, scope);
    let requested = match &depth {
        SeriesDepth::Absolute(value) | SeriesDepth::Relative(value) => value,
    };
    let width = requested
        .numerator_ref()
        .to_string()
        .parse::<usize>()
        .map_err(|_| MathError::Limit("analytical series integral width"))?
        .checked_add(2)
        .ok_or(MathError::Limit("analytical series width"))?;
    let work = width
        .checked_mul(width)
        .ok_or(MathError::Limit("analytical series work"))?;
    let input = expression
        .as_view()
        .get_byte_size()
        .checked_add(point.as_view().get_byte_size())
        .ok_or(MathError::Limit("analytical series input backing"))?;
    let temporary = input
        .checked_mul(work)
        .and_then(|bytes| bytes.checked_mul(4))
        .ok_or(MathError::Limit("analytical series temporary backing"))?;
    control.check(OperationExtent {
        work,
        input_bytes: input,
        temporary_bytes: temporary,
        ..Default::default()
    })?;
    symbolica::analytical::series_with(
        expression.as_view(),
        &axis
            .get_symbol()
            .ok_or_else(|| MathError::Contract("analytical native coordinate".into()))?
            .into(),
        point.as_view(),
        depth,
        &mut control,
    )
    .map_err(|error| match error {
        symbolica::analytical::SeriesOperationError::Control(error) => error,
        symbolica::analytical::SeriesOperationError::Series(error) => {
            MathError::Library(error.to_string())
        }
    })
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
    use std::fmt::Write as _;
    struct Counter<'a> {
        bytes: usize,
        maximum: usize,
        scope: &'a ExecutionScope,
        error: Option<MathError>,
    }
    impl std::fmt::Write for Counter<'_> {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            if let Err(error) = check(self.scope) {
                self.error = Some(error);
                return Err(std::fmt::Error);
            }
            match self
                .bytes
                .checked_add(text.len())
                .filter(|bytes| *bytes <= self.maximum)
            {
                Some(bytes) => {
                    self.bytes = bytes;
                    Ok(())
                }
                None => {
                    self.error = Some(MathError::Limit("analytical presentation bytes"));
                    Err(std::fmt::Error)
                }
            }
        }
    }
    let formats = [
        PrintOptions::file(),
        options(PrintOptions::latex()),
        options(PrintOptions::typst()),
    ];
    let mut counts = [0usize; 3];
    let mut total = 0usize;
    for (index, format) in formats.iter().enumerate() {
        let mut counter = Counter {
            bytes: 0,
            maximum: limits.scratch_bytes - total,
            scope,
            error: None,
        };
        if write!(&mut counter, "{}", atom.printer(format.clone())).is_err() {
            return Err(counter.error.unwrap_or_else(|| {
                MathError::Library("native analytical printer refused formatting".into())
            }));
        }
        counts[index] = counter.bytes;
        total += counter.bytes;
    }
    // Admit complete presentation backing before allocating the returned strings.
    // Native formatting remains the sole presentation authority in both passes.
    crate::construction::additional(total)?;
    let mut text = [String::new(), String::new(), String::new()];
    for (index, format) in formats.into_iter().enumerate() {
        check(scope)?;
        text[index]
            .try_reserve_exact(counts[index])
            .map_err(|_| MathError::Limit("analytical presentation allocation"))?;
        write!(&mut text[index], "{}", atom.printer(format)).map_err(|_| {
            MathError::Library("native analytical printer refused formatting".into())
        })?;
        if text[index].len() != counts[index] {
            return Err(MathError::Contract(
                "native presentation changed during one immutable projection".into(),
            ));
        }
    }
    let [symbolica, latex, typst] = text;
    let view = MathematicalView {
        symbolica,
        latex,
        typst,
    };
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
    // Retain symbolic conditions; decide only exact rational point requirements.
    fn check_exact_point(
        &self,
        axis: &Atom,
        point: &Atom,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<(), MathError> {
        for requirement in &self.requirements {
            crate::typed::admit_exact_source_substitution(
                &requirement.argument,
                axis,
                point,
                limits,
                scope,
            )?;
            let argument = requirement
                .argument
                .replace(axis.clone())
                .with(point.clone());
            bounded(&argument, limits, scope)?;
            // Remaining original inputs keep their condition symbolic. Decide
            // exact rational points without a lossy binary64 conversion.
            if let Ok(value) = Rational::try_from(argument.as_view()) {
                let permitted = match requirement.condition {
                    Condition::Positive => !value.is_negative() && !value.is_zero(),
                    Condition::Nonnegative => !value.is_negative(),
                    Condition::Nonzero => !value.is_zero(),
                };
                if !permitted {
                    return Err(MathError::Domain {
                        source_id: requirement.source,
                        requirement: requirement.condition.description(),
                    });
                }
            }
        }
        Ok(())
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
        let mut control = operation_control(limits, scope);
        for coordinate in coordinates {
            bounded(&expression, limits, scope)?;
            let axis = self
                .axis(*coordinate)?
                .get_symbol()
                .ok_or_else(|| MathError::Contract("analytical derivative coordinate".into()))?;
            expression = symbolica::analytical::derivative_with(
                expression.as_view(),
                &axis.into(),
                &mut control,
            )?;
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
    /// Express the exact derivative in the receiving physical contract's
    /// coordinates. The scale's actual binary value is converted exactly to a
    /// native rational; this grants no new physical conversion authority.
    pub fn derivative_with_scale(
        &self,
        coordinates: &[usize],
        scale: f64,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        let mut result = self.derivative(coordinates, limits, scope)?;
        result.expression *= physical_scale(scale)?;
        let observation = AnalyticalObservation::Derivative {
            coordinates: coordinates.to_vec(),
            result: mathematical_view(&result.expression, limits, scope)?,
        };
        Ok(AnalyticalResult {
            observation,
            expression: result.expression,
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
        self.check_exact_point(&axis, point, limits, scope)?;
        let series =
            controlled_series(&self.expression, &axis, point, depth.clone(), limits, scope)?;
        let expression = series.to_atom();
        bounded(&expression, limits, scope)?;
        let (relative_order, absolute_order) = match series.precision() {
            SeriesPrecision::Finite { relative, absolute } => (Some(relative), Some(absolute)),
            SeriesPrecision::Infinite => (None, None),
        };
        // Native polynomial conversion leaves nonpolynomial parts as coefficients.
        // Only coordinate-independent coefficients establish exact finite termination.
        admit_rational_normalization(&self.expression, limits, scope)?;
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
        self.identity_derivative_with_scales(other, coordinates, 1.0, 1.0, limits, scope)
    }
    /// Compare in the single complete receiving physical representation after
    /// source-owned derivative and boundary conversions have been admitted.
    pub fn identity_derivative_with_scales(
        &self,
        other: &Self,
        coordinates: &[usize],
        left_scale: f64,
        right_scale: f64,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        self.identity_derivative_product_with_scales(
            other,
            coordinates,
            None,
            left_scale,
            right_scale,
            limits,
            scope,
        )
    }
    /// Compare an original derivative multiplied by one explicitly selected
    /// original source output, under the compiler's complete physical boundary.
    pub fn identity_derivative_product_with_scales(
        &self,
        other: &Self,
        coordinates: &[usize],
        multiplier: Option<&Self>,
        left_scale: f64,
        right_scale: f64,
        limits: EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<AnalyticalResult, MathError> {
        bounded(&self.expression, limits, scope)?;
        bounded(&other.expression, limits, scope)?;
        let mut left = if coordinates.is_empty() {
            self.expression.clone()
        } else {
            self.derivative(coordinates, limits, scope)?.expression
        };
        if let Some(multiplier) = multiplier {
            bounded(&multiplier.expression, limits, scope)?;
            left *= &multiplier.expression;
        }
        let mut difference =
            left * physical_scale(left_scale)? - &other.expression * physical_scale(right_scale)?;
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
        // Native rational conversion brings nested coefficient denominators into
        // one exact field before a nonzero claim; local cancel alone is incomplete.
        if rational {
            admit_rational_normalization(&difference, limits, scope)?;
            use symbolica::domains::{
                integer::Z, rational::Q, rational_polynomial::RationalPolynomial,
            };
            let normal: RationalPolynomial<_, u32> = difference
                .try_to_rational_polynomial(&Q, &Z, None)
                .map_err(|error| MathError::Library(error.to_string()))?;
            difference = normal.to_expression();
            bounded(&difference, limits, scope)?;
        }
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

fn physical_scale(scale: f64) -> Result<Atom, MathError> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(MathError::Contract(
            "analytical physical conversion scale must be finite and positive".into(),
        ));
    }
    Rational::try_from(scale)
        .map(Atom::num)
        .map_err(|error| MathError::Contract(error.to_string()))
}

#[cfg(test)]
mod tests;

mod rational_approximation;
mod relation;
pub use relation::{RelationMeaning, RelationObservation, RelationRequest, RelationStop};
