// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Demand-owned coefficient-class and affine-row proof over original aggregated expressions.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::{CaseValues, Target},
    coefficients::Coefficients,
    library,
    presolve::{AffineRow, Facts, ObligationStatus},
};
use pse_ids::{FramedHasher, SemanticId};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Indeterminate, Symbol},
    id::Replacement,
};

/// The algebraic evidence needed to consider coefficient backends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassRequest {
    /// Affine original rows, primary objective of degree at most two, linear later objectives.
    Coefficients,
}
/// An unresolved dependency, rather than mathematical rejection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassDependency {
    /// The selected body did not retain an exact flattened symbolic expression.
    MissingSymbolicExpression {
        /// Original physical instance.
        instance: SemanticId,
        /// Original body output ordinal.
        output: usize,
    },
    /// The original selected domain obligation has not been established over its box.
    UnestablishedObligation {
        /// Original physical instance owning the obligation.
        instance: SemanticId,
    },
}
/// Actual proof or representation witness, retained with original attribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassWitness {
    /// An original aggregate row has a nonconstant symbolic first partial.
    NonAffineRow {
        /// Original aggregate row identity.
        row: SemanticId,
    },
    /// An original aggregate primary objective has a nonconstant second partial.
    NonQuadraticObjective {
        /// Original objective level.
        level: usize,
    },
    /// A later aggregate objective has a nonconstant first partial.
    NonlinearLaterObjective {
        /// Original objective level.
        level: usize,
    },
    /// An original obligation is contradicted throughout the admitted box.
    ViolatedObligation {
        /// Original physical instance owning the contradicted obligation.
        instance: SemanticId,
    },
    /// Exact symbolic coefficients cannot be represented by the numeric coefficient boundary.
    CoefficientRange,
}
/// Scientific assessment is distinct from missing evidence and representation readiness.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ClassStatus {
    /// No coefficient proof was requested.
    #[default]
    Unassessed,
    /// Original coefficient-class proof is established.
    Established,
    /// Actual scientific or domain proof excludes coefficient admission.
    RuledOut(ClassWitness),
    /// The proof still lacks explicitly named dependencies.
    Pending(Vec<ClassDependency>),
    /// Numeric representation is limited; this is not proof of nonquadratic science.
    RepresentationLimited(ClassWitness),
}
/// Immutable class evidence and the original-domain facts it consumed.
#[derive(Clone, Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "Class decisions retain complete original facts and the selected coefficient product"
)]
pub enum ClassEvidence {
    /// Coefficient representation and its source proof are ready.
    Established {
        /// Current original-domain and class facts.
        facts: Facts,
        /// Admitted numeric coefficient representation.
        coefficients: Coefficients,
    },
    /// Actual proof excludes coefficient admission.
    RuledOut {
        /// Current original-domain and class facts.
        facts: Facts,
        /// Actual scientific or domain witness.
        witness: ClassWitness,
    },
    /// Dependencies are unresolved, including opaque or bounded symbolic expressions.
    Pending {
        /// Current original-domain facts and unresolved class state.
        facts: Facts,
        /// Unresolved finite proof dependencies.
        dependencies: Vec<ClassDependency>,
    },
    /// Representation is unavailable without declaring the scientific class absent.
    RepresentationLimited {
        /// Current original-domain and retained scientific facts.
        facts: Facts,
        /// Numeric representation limit.
        witness: ClassWitness,
    },
}
impl ClassEvidence {
    /// Retained current original-space facts.
    pub fn facts(&self) -> &Facts {
        match self {
            Self::Established { facts, .. }
            | Self::RuledOut { facts, .. }
            | Self::Pending { facts, .. }
            | Self::RepresentationLimited { facts, .. } => facts,
        }
    }
    /// Authoritative assessed meaning, independent of optional Boolean fields.
    pub fn status(&self) -> ClassStatus {
        self.facts().class_status.clone()
    }
}
pub(crate) struct Allowance {
    pub(crate) remaining: usize,
}
impl Allowance {
    pub(crate) fn charge(&mut self, required: usize) -> Result<(), MathError> {
        if required > self.remaining {
            return Err(MathError::WorkLimit {
                source_id: SemanticId::NIL,
                resource: "class proof construction",
                required,
                available: self.remaining,
                components: 0,
            });
        }
        self.remaining -= required;
        Ok(())
    }
    fn derivative(
        &mut self,
        expression: &Atom,
        formal: &Atom,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Atom, MathError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        self.charge(expression.as_view().get_byte_size().saturating_add(1))?;
        let derivative = expression.derivative(
            Indeterminate::try_from(formal.clone()).map_err(|e| MathError::Library(e.clone()))?,
        );
        checkpoint(cancel)?;
        self.charge(derivative.as_view().get_byte_size())?;
        Ok(derivative)
    }
    fn number(&mut self, expression: &Atom, cancel: &Arc<AtomicBool>) -> Result<f64, MathError> {
        checkpoint(cancel)?;
        self.charge(expression.as_view().get_byte_size())?;
        let number = crate::coefficients::number(expression, cancel)?;
        self.charge(1)?;
        Ok(number)
    }
}
impl CasePlan {
    /// Prove original aggregate affine rows independently of whole-program class admission.
    /// The proof describes residuals on their original valid domain: all guards, domain
    /// obligations and tapes remain owned by the original callbacks and postsolve assessment.
    /// Missing expressions, nonlinear rows and unrepresentable coefficients withhold only
    /// their own row; stale assumptions, exhausted work and cancellation remain errors.
    pub fn affine_row_facts(
        &self,
        values: &CaseValues,
        base: &Facts,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Facts, MathError> {
        let _phase = pse_diagnostics::observation::phase("affine.preparation");
        for name in [
            "selected_rows",
            "retained_rows",
            "contributions_examined",
            "bound_output_reuses",
            "candidate_coordinates",
            "derivative_attempts",
            "nonzero_coefficients",
            "substitution_calls",
            "established_rows",
            "withheld_rows",
            "refused_requests",
            "formal_coordinates",
            "input_symbols_examined",
            "input_slots_examined",
        ] {
            observe(name, 0);
        }
        let result = self.affine_row_facts_inner(values, base, limit, cancel);
        if result.is_err() {
            observe("refused_requests", 1);
        }
        result
    }

    fn affine_row_facts_inner(
        &self,
        values: &CaseValues,
        base: &Facts,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Facts, MathError> {
        if !base.matches(self, values) {
            return Err(MathError::Contract(
                "affine row proof requires current domain facts".into(),
            ));
        }
        if base.affine.len() != self.structure().rows().len() {
            return Err(MathError::Contract(
                "affine row proof requires the original row inventory".into(),
            ));
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let mut allowance = Allowance {
            remaining: base.proof_remaining.min(limit),
        };
        allowance.charge(base.bytes())?;
        let mut facts = base.clone();
        let selected = self
            .structure()
            .rows()
            .iter()
            .zip(&base.affine)
            .filter_map(|(row, affine)| affine.is_none().then_some(row.id))
            .collect::<BTreeSet<_>>();
        allowance.charge(selected.len())?;
        observe("selected_rows", selected.len());
        observe("retained_rows", base.affine.len() - selected.len());
        if !selected.is_empty() {
            let (formals, expressions, missing) = aggregate_expressions(
                self,
                values,
                Aggregation::Rows(&selected),
                &mut allowance,
                cancel,
            )?;
            for (index, row) in self.structure().rows().iter().enumerate() {
                if !selected.contains(&row.id) {
                    continue;
                }
                if missing.contains(&Target::Row(row.id)) {
                    observe("withheld_rows", 1);
                    continue;
                }
                let expression = expressions
                    .get(&Target::Row(row.id))
                    .cloned()
                    .unwrap_or_else(|| Atom::num(0));
                facts.affine[index] =
                    match prove_affine(&expression, &formals, &mut allowance, cancel) {
                        Ok(proof) => proof,
                        Err(MathError::CoefficientRange) => None,
                        Err(error) => return Err(error),
                    };
                observe(
                    if facts.affine[index].is_some() {
                        "established_rows"
                    } else {
                        "withheld_rows"
                    },
                    1,
                );
            }
        }
        refresh_affine_key(base, &mut facts);
        facts.proof_remaining = allowance.remaining;
        Ok(facts)
    }

    /// Obtain only the requested mandatory class proof under the base projection's remainder.
    /// Resource and cancellation failures stop this request; they never become class negatives.
    pub fn class_evidence(
        &self,
        values: &CaseValues,
        base: &Facts,
        requested: ClassRequest,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<ClassEvidence, MathError> {
        match requested {
            ClassRequest::Coefficients => {}
        }
        let mut facts = shape_facts(self, values, base, limit, cancel)?;
        match facts.class_status.clone() {
            ClassStatus::RuledOut(witness) => {
                return Ok(ClassEvidence::RuledOut { facts, witness });
            }
            ClassStatus::Pending(dependencies) => {
                return Ok(ClassEvidence::Pending {
                    facts,
                    dependencies,
                });
            }
            ClassStatus::RepresentationLimited(witness) => {
                return Ok(ClassEvidence::RepresentationLimited { facts, witness });
            }
            ClassStatus::Established => {}
            ClassStatus::Unassessed => {
                return Err(MathError::Contract(
                    "mandatory class proof remained unassessed".into(),
                ));
            }
        }
        let (coefficients, remaining) = match self.coefficient_projection(
            values,
            &facts,
            facts.proof_remaining.min(limit),
            cancel,
        ) {
            Ok(product) => product,
            Err(MathError::CoefficientRange) => {
                let witness = ClassWitness::CoefficientRange;
                facts.class_status = ClassStatus::RepresentationLimited(witness.clone());
                return Ok(ClassEvidence::RepresentationLimited { facts, witness });
            }
            Err(error) => return Err(error),
        };
        facts.proof_remaining = remaining;
        Ok(ClassEvidence::Established {
            facts,
            coefficients,
        })
    }
}
pub(crate) fn shape_facts(
    plan: &CasePlan,
    values: &CaseValues,
    base: &Facts,
    limit: usize,
    cancel: &Arc<AtomicBool>,
) -> Result<Facts, MathError> {
    if !base.matches(plan, values) {
        return Err(MathError::Contract(
            "class proof requires current domain facts".into(),
        ));
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(MathError::Cancelled);
    }
    if limit == 0 {
        return Err(MathError::Limit("class proof construction"));
    }
    let mut allowance = Allowance {
        remaining: base.proof_remaining.min(limit),
    };
    allowance.charge(base.bytes())?;
    let mut facts = base.clone();
    if !matches!(base.class_status, ClassStatus::Unassessed) {
        facts.proof_remaining = allowance.remaining;
        return Ok(facts);
    }
    let mut pending = vec![];
    for binding in plan.structure().instances() {
        match base.obligations.get(&binding.instance) {
            Some(ObligationStatus::Violated) => {
                facts.class_status = ClassStatus::RuledOut(ClassWitness::ViolatedObligation {
                    instance: binding.instance,
                });
                return Ok(facts);
            }
            Some(ObligationStatus::Discharged) => {}
            Some(ObligationStatus::Unestablished) | None => {
                allowance.charge(1)?;
                pending.push(ClassDependency::UnestablishedObligation {
                    instance: binding.instance,
                });
            }
        }
        for contribution in &binding.contributions {
            if plan.bodies()[&binding.body]
                .expression(contribution.output)
                .is_none()
            {
                allowance.charge(1)?;
                pending.push(ClassDependency::MissingSymbolicExpression {
                    instance: binding.instance,
                    output: contribution.output,
                });
            }
        }
    }
    if !pending.is_empty() {
        facts.proof_remaining = allowance.remaining;
        facts.class_status = ClassStatus::Pending(pending);
        return Ok(facts);
    }
    let (formals, expressions, _) =
        aggregate_expressions(plan, values, Aggregation::All, &mut allowance, cancel)?;
    allowance.charge(plan.structure().rows().len())?;
    facts.affine = vec![None; plan.structure().rows().len()];
    let mut row_witness = None;
    for (row, target) in plan.structure().rows().iter().enumerate() {
        let expression = expressions
            .get(&Target::Row(target.id))
            .cloned()
            .unwrap_or_else(|| Atom::num(0));
        match prove_affine(&expression, &formals, &mut allowance, cancel) {
            Ok(Some(affine)) => facts.affine[row] = Some(affine),
            Ok(None) => {
                row_witness.get_or_insert(ClassWitness::NonAffineRow { row: target.id });
            }
            Err(MathError::CoefficientRange) => {
                refresh_affine_key(base, &mut facts);
                facts.proof_remaining = allowance.remaining;
                facts.class_status =
                    ClassStatus::RepresentationLimited(ClassWitness::CoefficientRange);
                return Ok(facts);
            }
            Err(error) => return Err(error),
        }
    }
    refresh_affine_key(base, &mut facts);
    if let Some(witness) = row_witness {
        facts.proof_remaining = allowance.remaining;
        facts.class_status = ClassStatus::RuledOut(witness);
        return Ok(facts);
    }
    facts.objective_degree = Some(0);
    facts.lexicographic_degree = Some(0);
    // The coefficient product still owns a full objective-column inventory. Selected-row
    // preparation never constructs this whole-program projection.
    allowance.charge(plan.columns().len())?;
    let formals = (0..plan.columns().len())
        .map(library::formal)
        .collect::<Result<Vec<_>, _>>()?;
    facts.objective_linear = vec![true; formals.len()];
    for level in 0..plan.structure().objectives().len() {
        allowance.charge(
            expressions
                .get(&Target::Objective(level))
                .map_or(0, |atom| atom.as_view().get_byte_size()),
        )?;
        let expression = expressions
            .get(&Target::Objective(level))
            .cloned()
            .unwrap_or_else(|| Atom::num(0));
        let mut degree = 0;
        for (column, formal) in formals.iter().enumerate() {
            let first = allowance.derivative(&expression, formal, cancel)?;
            if !first.is_zero() {
                degree = degree.max(1);
            }
            if level == 0 {
                facts.objective_linear[column] = first.is_constant();
            }
            if level > 0 && !first.is_constant() {
                facts.lexicographic_degree = None;
                facts.class_status =
                    ClassStatus::RuledOut(ClassWitness::NonlinearLaterObjective { level });
                facts.proof_remaining = allowance.remaining;
                return Ok(facts);
            }
            if level == 0 && !first.is_constant() {
                for other in &formals[column..] {
                    let second = allowance.derivative(&first, other, cancel)?;
                    if !second.is_constant() {
                        facts.objective_degree = None;
                        facts.class_status =
                            ClassStatus::RuledOut(ClassWitness::NonQuadraticObjective { level });
                        facts.proof_remaining = allowance.remaining;
                        return Ok(facts);
                    }
                    if !second.is_zero() {
                        degree = 2;
                    }
                }
            }
        }
        if level == 0 {
            facts.objective_degree = Some(degree);
        } else {
            facts.lexicographic_degree = facts
                .lexicographic_degree
                .map(|previous| previous.max(degree));
        }
    }
    facts.proof_remaining = allowance.remaining;
    facts.class_status = ClassStatus::Established;
    facts.curvature = None;
    Ok(facts)
}

pub(crate) fn aggregate_objectives(
    plan: &CasePlan,
    values: &CaseValues,
    limit: usize,
    cancel: &Arc<AtomicBool>,
) -> Result<(Vec<Atom>, Vec<Atom>, usize), MathError> {
    let mut allowance = Allowance { remaining: limit };
    // Original rows have already been proved and retained as affine coefficients.
    // Rebuilding them here would spend the shared allowance on discarded products.
    let (_, expressions, _) = aggregate_expressions(
        plan,
        values,
        Aggregation::Objectives,
        &mut allowance,
        cancel,
    )?;
    allowance.charge(plan.structure().objectives().len())?;
    for level in 0..plan.structure().objectives().len() {
        allowance.charge(
            expressions
                .get(&Target::Objective(level))
                .map_or(0, |atom| atom.as_view().get_byte_size()),
        )?;
    }
    let objectives = (0..plan.structure().objectives().len())
        .map(|level| {
            expressions
                .get(&Target::Objective(level))
                .cloned()
                .unwrap_or_else(|| Atom::num(0))
        })
        .collect();
    // Coefficient projection consumes this dense objective coordinate product explicitly.
    allowance.charge(plan.columns().len())?;
    let formals = (0..plan.columns().len())
        .map(library::formal)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((formals, objectives, allowance.remaining))
}
#[derive(Clone, Copy)]
enum Aggregation<'a> {
    All,
    Objectives,
    Rows(&'a BTreeSet<SemanticId>),
}
/// Explicit global identity: library symbol registration ordinals are never column indices.
#[derive(Default)]
struct FormalCoordinates {
    symbols: BTreeMap<Symbol, (usize, Atom)>,
    columns: BTreeMap<usize, Atom>,
}
impl FormalCoordinates {
    fn get(&mut self, column: usize, allowance: &mut Allowance) -> Result<Atom, MathError> {
        if let Some(formal) = self.columns.get(&column) {
            return Ok(formal.clone());
        }
        allowance.charge(2)?;
        let formal = library::formal(column)?;
        let AtomView::Var(variable) = formal.as_view() else {
            return Err(MathError::Contract(
                "class coordinate is not a variable".into(),
            ));
        };
        if self
            .symbols
            .insert(variable.get_symbol(), (column, formal.clone()))
            .is_some()
        {
            return Err(MathError::Contract(
                "duplicate class coordinate symbol".into(),
            ));
        }
        self.columns.insert(column, formal.clone());
        observe("formal_coordinates", 1);
        Ok(formal)
    }
}
type AggregatedExpressions = (FormalCoordinates, BTreeMap<Target, Atom>, BTreeSet<Target>);

fn checkpoint(cancel: &Arc<AtomicBool>) -> Result<(), MathError> {
    if cancel.load(Ordering::Relaxed) {
        Err(MathError::Cancelled)
    } else {
        Ok(())
    }
}
fn observe(name: &str, count: usize) {
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count(&format!("affine.{name}"), "count", count as u64);
    }
}

/// One library traversal over the original tree. Symbolica 3.0.1 replace_no_norm returns
/// immediately after a match and only descends into original children; inserted RHS atoms
/// are not revisited. Normalization runs once after the traversal (id.rs).
fn substitute(
    expression: &Atom,
    replacements: &[(Atom, Atom)],
    allowance: &mut Allowance,
    cancel: &Arc<AtomicBool>,
) -> Result<Atom, MathError> {
    checkpoint(cancel)?;
    if replacements.is_empty() {
        allowance.charge(expression.as_view().get_byte_size())?;
        return Ok(expression.clone());
    }
    let extent = replacements
        .iter()
        .try_fold(1usize, |bytes, (_, replacement)| {
            bytes
                .checked_add(replacement.as_view().get_byte_size())
                .ok_or(MathError::Limit("class symbolic substitution"))
        })?;
    allowance.charge(
        expression
            .as_view()
            .get_byte_size()
            .checked_mul(extent)
            .ok_or(MathError::Limit("class symbolic substitution"))?,
    )?;
    observe("substitution_calls", 1);
    let result = expression.replace_multiple(
        replacements
            .iter()
            .map(|(formal, replacement)| Replacement::new(formal.clone(), replacement.clone())),
    );
    checkpoint(cancel)?;
    allowance.charge(result.as_view().get_byte_size())?;
    Ok(result)
}

fn aggregate_expressions(
    plan: &CasePlan,
    values: &CaseValues,
    selected: Aggregation<'_>,
    allowance: &mut Allowance,
    cancel: &Arc<AtomicBool>,
) -> Result<AggregatedExpressions, MathError> {
    // Aggregate original contributions before proof: nonlinear terms may cancel across
    // different bodies, aliases and contribution scales. Symbolica owns normalization.
    let mut formals = FormalCoordinates::default();
    let mut expressions = BTreeMap::<Target, Atom>::new();
    let mut missing = BTreeSet::new();
    // This cache belongs only to this request and its consumed assumptions. Both newly
    // bound expressions and retained clones spend the same remaining allowance.
    let mut bound = BTreeMap::<(usize, usize), Atom>::new();
    let mut visit = |binding_index: usize, contribution_index: usize| -> Result<(), MathError> {
        checkpoint(cancel)?;
        observe("contributions_examined", 1);
        let binding = &plan.structure().instances()[binding_index];
        let contribution = &binding.contributions[contribution_index];
        if missing.contains(&contribution.target) {
            return Ok(());
        }
        let body = &plan.bodies()[&binding.body];
        let Some(original) = body.expression(contribution.output) else {
            if matches!(selected, Aggregation::Rows(_)) {
                allowance.charge(1)?;
                missing.insert(contribution.target);
                expressions.remove(&contribution.target);
                return Ok(());
            }
            return Err(MathError::Contract(
                "class symbolic dependency changed".into(),
            ));
        };
        let key = (binding_index, contribution.output);
        let expression = if let Some(expression) = bound.get(&key) {
            allowance.charge(expression.as_view().get_byte_size())?;
            observe("bound_output_reuses", 1);
            expression.clone()
        } else {
            // Discover candidates in this output, then resolve them through the body's
            // already admitted symbol layout. A wide multi-output binding must not cause
            // unrelated inputs or plan formals to be constructed for this expression.
            checkpoint(cancel)?;
            allowance.charge(original.as_view().get_byte_size())?;
            let symbols = original.get_all_symbols(true);
            checkpoint(cancel)?;
            allowance.charge(symbols.len())?;
            observe("input_symbols_examined", symbols.len());
            let mut inputs = symbols
                .into_iter()
                .filter_map(|symbol| body.input_slot(symbol).map(|slot| (slot, symbol)))
                .collect::<Vec<_>>();
            inputs.sort_by_key(|(slot, _)| *slot);
            allowance.charge(inputs.len())?;
            observe("input_slots_examined", inputs.len());
            let mut substitutions = Vec::with_capacity(inputs.len());
            for (slot, symbol) in inputs {
                checkpoint(cancel)?;
                let binding = binding.slots.get(slot).ok_or_else(|| {
                    MathError::Contract("class input outside binding layout".into())
                })?;
                let replacement = if let Some(&column) =
                    plan.projection_topology().columns.get(&binding.source())
                {
                    formals.get(column, allowance)? * Atom::num(binding.scale())
                        + Atom::num(binding.offset())
                } else {
                    let value = values
                        .scalars
                        .get(&binding.source())
                        .ok_or_else(|| MathError::Contract("missing class assumption".into()))?;
                    Atom::num(binding.scale() * value + binding.offset())
                };
                checkpoint(cancel)?;
                allowance.charge(replacement.as_view().get_byte_size())?;
                let formal = Atom::var(symbol);
                if formal != replacement {
                    substitutions.push((formal, replacement));
                }
            }
            let expression = substitute(original, &substitutions, allowance, cancel)?;
            allowance.charge(expression.as_view().get_byte_size().saturating_add(1))?;
            bound.insert(key, expression.clone());
            expression
        };
        let previous = expressions
            .remove(&contribution.target)
            .unwrap_or_else(|| Atom::num(0));
        allowance.charge(
            previous
                .as_view()
                .get_byte_size()
                .checked_add(expression.as_view().get_byte_size())
                .ok_or(MathError::Limit("class aggregate expression"))?,
        )?;
        checkpoint(cancel)?;
        let aggregate = previous + expression * Atom::num(contribution.scale);
        checkpoint(cancel)?;
        allowance.charge(aggregate.as_view().get_byte_size())?;
        expressions.insert(contribution.target, aggregate);
        Ok(())
    };
    match selected {
        Aggregation::Rows(rows) => {
            // Borrow the immutable original-contribution index; no unrelated bindings or
            // outputs are searched to satisfy selected row demand.
            for row in rows {
                for &(binding, contribution) in plan.row_contributions(*row) {
                    visit(binding, contribution)?;
                }
            }
        }
        Aggregation::All | Aggregation::Objectives => {
            for (binding_index, binding) in plan.structure().instances().iter().enumerate() {
                for (contribution_index, contribution) in binding.contributions.iter().enumerate() {
                    if matches!(selected, Aggregation::All)
                        || matches!(contribution.target, Target::Objective(_))
                    {
                        visit(binding_index, contribution_index)?;
                    }
                }
            }
        }
    }
    Ok((formals, expressions, missing))
}

fn prove_affine(
    expression: &Atom,
    formals: &FormalCoordinates,
    allowance: &mut Allowance,
    cancel: &Arc<AtomicBool>,
) -> Result<Option<AffineRow>, MathError> {
    checkpoint(cancel)?;
    allowance.charge(expression.as_view().get_byte_size().saturating_mul(2))?;
    // true is required for coordinates nested in rational-polynomial function variables.
    // Symbols are conservative candidates, not a claim of exact semantic dependence.
    let symbols = expression.get_all_symbols(true);
    checkpoint(cancel)?;
    allowance.charge(symbols.len())?;
    let mut candidates = symbols
        .iter()
        .filter_map(|symbol| formals.symbols.get(symbol))
        .map(|(column, formal)| (*column, formal.clone()))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(column, _)| *column);
    observe("candidate_coordinates", candidates.len());
    let mut affine = AffineRow {
        entries: BTreeMap::new(),
        constant: 0.0,
    };
    for (column, formal) in &candidates {
        observe("derivative_attempts", 1);
        let derivative = allowance.derivative(expression, formal, cancel)?;
        if !derivative.is_constant() {
            return Ok(None);
        }
        let coefficient = allowance.number(&derivative, cancel)?;
        if coefficient != 0.0 {
            allowance.charge(1)?;
            affine.entries.insert(*column, coefficient);
            observe("nonzero_coefficients", 1);
        }
    }
    let zeros = candidates
        .into_iter()
        .map(|(_, formal)| (formal, Atom::num(0)))
        .collect::<Vec<_>>();
    let zero = substitute(expression, &zeros, allowance, cancel)?;
    if !zero.is_constant() {
        return Err(MathError::Contract(
            "unmapped class coordinate or residual symbol".into(),
        ));
    }
    affine.constant = allowance.number(&zero, cancel)?;
    checkpoint(cancel)?;
    Ok(Some(affine))
}

fn refresh_affine_key(base: &Facts, facts: &mut Facts) {
    if facts.affine == base.affine {
        return;
    }
    let mut hash = FramedHasher::new(pse_ids::Frame::MathBoundFactsV3);
    hash.hash(&base.key)
        .str("original-aggregate-affine-rows-v1")
        .u64(facts.affine.len() as u64);
    for row in &facts.affine {
        hash.bool(row.is_some());
        if let Some(row) = row {
            hash.u64(row.constant.to_bits())
                .u64(row.entries.len() as u64);
            for (column, value) in &row.entries {
                hash.u64(*column as u64).u64(value.to_bits());
            }
        }
    }
    facts.key = hash.finish_hash();
}

#[cfg(test)]
mod affine_controls {
    use super::*;

    fn coordinates(columns: &[usize], allowance: &mut Allowance) -> FormalCoordinates {
        let mut coordinates = FormalCoordinates::default();
        for &column in columns {
            let _ = coordinates.get(column, allowance).unwrap();
        }
        coordinates
    }

    #[test]
    fn affine_bulk_binding_qualifies_permutations_aliases_offsets_and_inserted_formals() {
        crate::initialize().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let a = library::formal(0).unwrap();
        let b = library::formal(1).unwrap();
        let c = library::formal(2).unwrap();
        let original = &a + &b * Atom::num(2) + &c * Atom::num(3);
        let cases = [
            (
                vec![(a.clone(), b.clone()), (b.clone(), a.clone())],
                BTreeMap::from([(0, 2.0), (1, 1.0), (2, 3.0)]),
                0.0,
            ),
            (
                vec![
                    (a.clone(), &b + Atom::num(4)),
                    (b.clone(), &a * Atom::num(2)),
                    (c.clone(), a.clone()),
                ],
                BTreeMap::from([(0, 7.0), (1, 1.0)]),
                4.0,
            ),
            (
                vec![
                    (a.clone(), &b * Atom::num(2) + Atom::num(1)),
                    (b.clone(), &b * Atom::num(-3) + Atom::num(4)),
                    (c.clone(), &b + Atom::num(5)),
                ],
                BTreeMap::from([(1, -1.0)]),
                24.0,
            ),
        ];
        for (replacements, entries, constant) in cases {
            for replacements in [
                replacements.clone(),
                replacements.into_iter().rev().collect(),
            ] {
                let mut allowance = Allowance { remaining: 100_000 };
                let bound = substitute(&original, &replacements, &mut allowance, &cancel).unwrap();
                let formals = coordinates(&[0, 1, 2], &mut allowance);
                let affine = prove_affine(&bound, &formals, &mut allowance, &cancel)
                    .unwrap()
                    .unwrap();
                assert_eq!(affine.entries, entries);
                assert_eq!(affine.constant, constant);
            }
        }
    }

    #[test]
    fn affine_candidates_preserve_global_indices_and_refuse_unmapped_nested_dependence() {
        crate::initialize().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut allowance = Allowance { remaining: 100_000 };
        let formals = coordinates(&[2, 40, 91], &mut allowance);
        let x = library::formal(40).unwrap();
        let expression = &x * Atom::num(7) + Atom::num(3);
        let row = prove_affine(&expression, &formals, &mut allowance, &cancel)
            .unwrap()
            .unwrap();
        assert_eq!(row.entries, BTreeMap::from([(40, 7.0)]));
        assert_eq!(row.constant, 3.0);
        assert!(
            prove_affine(&x.sin().exp(), &formals, &mut allowance, &cancel)
                .unwrap()
                .is_none()
        );
        let absent = FormalCoordinates::default();
        assert!(matches!(
            prove_affine(&x.sin().exp(), &absent, &mut allowance, &cancel),
            Err(MathError::Contract(_))
        ));
        for constant in [0, 13] {
            let row = prove_affine(&Atom::num(constant), &absent, &mut allowance, &cancel)
                .unwrap()
                .unwrap();
            assert!(row.entries.is_empty());
            assert_eq!(row.constant, f64::from(constant));
        }
        let mut exhausted = Allowance { remaining: 0 };
        assert!(matches!(
            prove_affine(&expression, &formals, &mut exhausted, &cancel),
            Err(MathError::WorkLimit { .. })
        ));
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            prove_affine(&expression, &formals, &mut allowance, &cancel),
            Err(MathError::Cancelled)
        ));
    }
}
