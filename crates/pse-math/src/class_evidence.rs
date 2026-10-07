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
use symbolica::atom::{Atom, AtomCore, Indeterminate};

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
        self.charge(1)?;
        Ok(expression.derivative(
            Indeterminate::try_from(formal.clone()).map_err(|e| MathError::Library(e.clone()))?,
        ))
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
        if !selected.is_empty() {
            let (formals, expressions, missing) = aggregate_expressions(
                self,
                values,
                Aggregation::Rows(&selected),
                &mut allowance,
                cancel,
            )?;
            for (index, row) in self.structure().rows().iter().enumerate() {
                if !selected.contains(&row.id) || missing.contains(&Target::Row(row.id)) {
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
    allowance.charge(formals.len())?;
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
    let (formals, expressions, _) = aggregate_expressions(
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
    Ok((formals, objectives, allowance.remaining))
}
#[derive(Clone, Copy)]
enum Aggregation<'a> {
    All,
    Objectives,
    Rows(&'a BTreeSet<SemanticId>),
}
type AggregatedExpressions = (Vec<Atom>, BTreeMap<Target, Atom>, BTreeSet<Target>);

fn aggregate_expressions(
    plan: &CasePlan,
    values: &CaseValues,
    selected: Aggregation<'_>,
    allowance: &mut Allowance,
    cancel: &Arc<AtomicBool>,
) -> Result<AggregatedExpressions, MathError> {
    // Combine original contributions before differentiating: individual nonlinear terms
    // may cancel across bodies, rows or objectives. Symbolica owns that normalization.
    allowance.charge(
        plan.columns()
            .len()
            .checked_mul(2)
            .ok_or(MathError::Limit("class coordinate storage"))?,
    )?;
    let formals = (0..plan.columns().len())
        .map(library::formal)
        .collect::<Result<Vec<_>, _>>()?;
    let columns: BTreeMap<_, _> = plan
        .columns()
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();
    let mut expressions = BTreeMap::<Target, Atom>::new();
    let mut missing = BTreeSet::new();
    for binding in plan.structure().instances() {
        for contribution in &binding.contributions {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let requested = match selected {
                Aggregation::All => true,
                Aggregation::Objectives => matches!(contribution.target, Target::Objective(_)),
                Aggregation::Rows(rows) => {
                    matches!(contribution.target, Target::Row(row) if rows.contains(&row))
                }
            };
            if !requested || missing.contains(&contribution.target) {
                continue;
            }
            let Some(original) = plan.bodies()[&binding.body].expression(contribution.output)
            else {
                if matches!(selected, Aggregation::Rows(_)) {
                    allowance.charge(1)?;
                    missing.insert(contribution.target);
                    expressions.remove(&contribution.target);
                    continue;
                }
                return Err(MathError::Contract(
                    "class symbolic dependency changed".into(),
                ));
            };
            allowance.charge(original.as_view().get_byte_size())?;
            let mut expression = original.clone();
            // Simultaneous replacement prevents a global coordinate from being mistaken
            // for a later body-local slot, including permuted/aliased physical bindings.
            allowance.charge(binding.slots.len())?;
            let replacements = binding
                .slots
                .iter()
                .enumerate()
                .map(|(slot, binding)| {
                    let replacement = if let Some(&column) = columns.get(&binding.source()) {
                        &formals[column] * Atom::num(binding.scale()) + Atom::num(binding.offset())
                    } else {
                        let value = values.scalars.get(&binding.source()).ok_or_else(|| {
                            MathError::Contract("missing class assumption".into())
                        })?;
                        Atom::num(binding.scale() * value + binding.offset())
                    };
                    Ok((library::formal(slot)?, replacement))
                })
                .collect::<Result<Vec<_>, MathError>>()?
                .into_iter()
                .filter(|(formal, replacement)| formal != replacement)
                .collect::<Vec<_>>();
            allowance.charge(replacements.len())?;
            // Existing replacement supports a multiple-pattern map, handled through
            // fresh inert symbols so no replacement can rebind another slot.
            let temporaries = replacements
                .iter()
                .enumerate()
                .map(|(slot, _)| library::formal(plan.columns().len() + binding.slots.len() + slot))
                .collect::<Result<Vec<_>, _>>()?;
            for ((formal, _), temporary) in replacements.iter().zip(&temporaries) {
                allowance.charge(expression.as_view().get_byte_size())?;
                expression = expression.replace(formal.clone()).with(temporary.clone());
            }
            for ((_, replacement), temporary) in replacements.iter().zip(&temporaries) {
                let extent = expression
                    .as_view()
                    .get_byte_size()
                    .checked_mul(replacement.as_view().get_byte_size().saturating_add(1))
                    .ok_or(MathError::Limit("class symbolic substitution"))?;
                allowance.charge(extent)?;
                expression = expression
                    .replace(temporary.clone())
                    .with(replacement.clone());
            }
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
            expressions.insert(
                contribution.target,
                previous + expression * Atom::num(contribution.scale),
            );
        }
    }
    Ok((formals, expressions, missing))
}

fn prove_affine(
    expression: &Atom,
    formals: &[Atom],
    allowance: &mut Allowance,
    cancel: &Arc<AtomicBool>,
) -> Result<Option<AffineRow>, MathError> {
    allowance.charge(expression.as_view().get_byte_size().saturating_mul(2))?;
    let mut affine = AffineRow {
        entries: BTreeMap::new(),
        constant: 0.0,
    };
    let mut zero = expression.clone();
    for (column, formal) in formals.iter().enumerate() {
        let derivative = allowance.derivative(expression, formal, cancel)?;
        if !derivative.is_constant() {
            return Ok(None);
        }
        let coefficient = crate::coefficients::number(&derivative, cancel)?;
        if coefficient != 0.0 {
            allowance.charge(1)?;
            affine.entries.insert(column, coefficient);
        }
        zero = zero.replace(formal.clone()).with(Atom::num(0));
    }
    affine.constant = crate::coefficients::number(&zero, cancel)?;
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
