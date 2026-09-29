// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Multi-objective admission and the `objective_bounds` transformation (ADR-0111, DP-08).
//!
//! Authored objective members are grouped into lexicographic levels by priority. A level
//! value is the weighted sum Σ sᵢ·gᵢ of its members' terms: gᵢ is the member itself, or the
//! generated quotient of the member by a generated parameter holding its declared
//! normalization, typed by the package's quantity operations; the scale sᵢ is the member's
//! weight, negated when its sense opposes the level's. Every term of a sum of several is
//! neutral dimensionless (PS-01), so a level value is a pure number unless the level is
//! one member, which keeps its term's type.
//!
//! A staged step selects one level with the `objective.level` fact. The named
//! transformation `objective_bounds` then bounds every earlier level by a generated
//! parameter β, sense-adjusted, so a sequence of K levels prepares at most K structures
//! and rebinds only the β values (improvement I15). Without the fact every level is kept,
//! for a native lexicographic solve.
use super::*;
use crate::ObjectiveRefusal as Refusal;
use crate::annotation::{AnnotationValue, ObjectiveDeclaration, ObjectiveSense};
use pse_quantity::QuantityTypeId;

/// The authored objectives of a specialized model, grouped into levels (ADR-0111).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Objectives {
    /// Every authored member, in level order and then instantiation order.
    pub members: Vec<ObjectiveMember>,
    /// Levels in optimization order: ascending priority.
    pub levels: Vec<ObjectiveLevel>,
    /// The level a staged step optimizes, selected by the `objective.level` fact; every
    /// earlier level then carries its generated bound.
    pub selected: Option<usize>,
}
impl Objectives {
    /// The level a solve of this structure optimizes: the selected level, or the only one.
    /// `None` without objectives, and for several levels without a selection, which only
    /// a native lexicographic route optimizes together.
    pub fn solved(&self) -> Option<&ObjectiveLevel> {
        match (self.selected, self.levels.as_slice()) {
            (Some(level), levels) => levels.get(level),
            (None, [level]) => Some(level),
            _ => None,
        }
    }
    /// The members of one level.
    pub fn members_of<'a>(
        &'a self,
        level: &'a ObjectiveLevel,
    ) -> impl Iterator<Item = &'a ObjectiveMember> + 'a {
        level.members.iter().map(|&m| &self.members[m])
    }
}
/// One authored objective member.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectiveMember {
    /// The scalar member whose value is the objective.
    pub target: SemanticId,
    /// Authored orientation.
    pub sense: ObjectiveSense,
    /// Authored priority; `None` when every objective shares one undeclared level.
    pub priority: Option<i64>,
    /// Positive dimensionless weight: authored, or one.
    pub weight: f64,
    /// The generated parameter holding the declared normalization, a positive value of
    /// the target's type, if declared.
    pub normalization: Option<SemanticId>,
    /// The symbol entering the level value: the target, or the generated expression
    /// member `target / normalization`.
    pub term: SemanticId,
    /// The factor the term enters its level value with: the weight, negated when the
    /// member's sense opposes the level's.
    pub scale: f64,
    /// Position of the member's level in [`Objectives::levels`].
    pub level: usize,
    /// The objective annotation and its instance.
    pub lineage: Lineage,
}
/// One lexicographic level; its value is Σ scaleᵢ·termᵢ over its members.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectiveLevel {
    /// Shared authored priority of the members; `None` when undeclared.
    pub priority: Option<i64>,
    /// Positions of the members in [`Objectives::members`].
    pub members: Vec<usize>,
    /// Orientation of the level value: the members' common sense, or minimization of the
    /// signed sum when their senses differ.
    pub sense: ObjectiveSense,
    /// Canonical physical type of the level value: neutral dimensionless, unless the level
    /// is one member, which keeps its term's type.
    pub quantity: QuantityTypeId,
    /// Absolute degradation tolerance in canonical units of the level value.
    pub absolute_tolerance: Option<f64>,
    /// Dimensionless degradation tolerance relative to the level optimum.
    pub relative_tolerance: Option<f64>,
    /// The generated bound of this level while a later level is optimized.
    pub bound: Option<ObjectiveBound>,
}
/// The bound an earlier level keeps while a later level is optimized: the row
/// `value − β <= 0` for a minimized level and `value − β >= 0` for a maximized one. The
/// engine binds β = f* + max(abs, rel·|f*|) (minimized) or f* − max(abs, rel·|f*|)
/// (maximized) from the level optimum f*; the row is structure, β a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectiveBound {
    /// Generated parameter β of the level type, without a default value.
    pub parameter: SemanticId,
    /// Identity of the generated inequality row.
    pub row: SemanticId,
}

impl Engine<'_, '_> {
    /// Group the objective annotations into levels and apply `objective_bounds` for a
    /// selected level.
    pub(super) fn admit_objectives(&mut self) -> Result<()> {
        let declared = self
            .model
            .annotations
            .iter()
            .filter_map(|a| match &a.value {
                AnnotationValue::Objective(o) => Some((a.target, o.clone(), a.lineage.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        let Some((_, _, first)) = declared.first() else {
            if self.objective_level.is_some() {
                return Err(refuse(SemanticId::NIL, Refusal::UnknownLevel));
            }
            return Ok(());
        };
        let first = first.declaration.as_id();
        // Several objectives need a priority on each, or one shared level with weights.
        let prioritized = declared
            .iter()
            .filter(|(_, o, _)| o.priority.is_some())
            .count();
        if declared.len() > 1
            && prioritized != declared.len()
            && (prioritized != 0 || declared.iter().any(|(_, o, _)| o.weight.is_none()))
        {
            return Err(refuse(first, Refusal::CompetingWithoutPriority));
        }
        let mut grouped = BTreeMap::<Option<i64>, Vec<Declared>>::new();
        for member in declared {
            grouped.entry(member.1.priority).or_default().push(member);
        }
        let count = grouped.len();
        let mut objectives = Objectives::default();
        for (position, (priority, members)) in grouped.into_iter().enumerate() {
            let level = self.admit_level(position, priority, &members, &mut objectives)?;
            // Every level that bounds a later one declares both tolerances.
            if position + 1 < count
                && (level.absolute_tolerance.is_none() || level.relative_tolerance.is_none())
            {
                return Err(refuse(
                    members[0].2.declaration.as_id(),
                    Refusal::MissingTolerance,
                ));
            }
            objectives.levels.push(level);
        }
        if let Some(selected) = self.objective_level {
            if selected >= objectives.levels.len() {
                return Err(refuse(first, Refusal::UnknownLevel));
            }
            for position in 0..selected {
                let bound = self.objective_bound(&objectives, position)?;
                objectives.levels[position].bound = Some(bound);
            }
            objectives.selected = Some(selected);
        }
        self.model.objectives = objectives;
        Ok(())
    }
    /// One level: its members' scales, its sense, type and tolerances.
    fn admit_level(
        &mut self,
        position: usize,
        priority: Option<i64>,
        members: &[Declared],
        objectives: &mut Objectives,
    ) -> Result<ObjectiveLevel> {
        let at = members[0].2.declaration;
        let neutral = self
            .c
            .quantities
            .neutral_dimensionless()
            .ok_or_else(|| invalid(at, "objective levels need a neutral scalar type"))?;
        let several = members.len() > 1;
        let senses = members
            .iter()
            .map(|(_, o, _)| o.sense)
            .collect::<BTreeSet<_>>();
        let sense = match senses.into_iter().collect::<Vec<_>>().as_slice() {
            [sense] => *sense,
            _ => ObjectiveSense::Minimize,
        };
        let mut quantity = neutral;
        let first_member = objectives.members.len();
        for (target, declaration, lineage) in members {
            if several && declaration.weight.is_none() {
                return Err(refuse(lineage.declaration.as_id(), Refusal::MissingWeight));
            }
            let (normalization, term) = match &declaration.normalization {
                Some(value) => {
                    let (parameter, term) = self.normalized_term(*target, value, lineage)?;
                    (Some(parameter), term)
                }
                None => (None, *target),
            };
            let term_quantity = self.objective_quantity(term, lineage.declaration)?;
            if several && term_quantity != neutral {
                return Err(refuse(lineage.declaration.as_id(), Refusal::DimensionalSum));
            }
            if !several {
                quantity = term_quantity;
            }
            let weight = declaration.weight.unwrap_or(1.0);
            objectives.members.push(ObjectiveMember {
                target: *target,
                sense: declaration.sense,
                priority,
                weight,
                normalization,
                term,
                scale: if declaration.sense == sense {
                    weight
                } else {
                    -weight
                },
                level: position,
                lineage: lineage.clone(),
            });
        }
        let absolute_tolerance = agreed(
            members.iter().map(|(_, o, _)| {
                o.absolute_tolerance
                    .as_ref()
                    .map(|v| v.scalar(at))
                    .transpose()
            }),
            at,
        )?;
        let relative_tolerance =
            agreed(members.iter().map(|(_, o, _)| Ok(o.relative_tolerance)), at)?;
        Ok(ObjectiveLevel {
            priority,
            members: (first_member..objectives.members.len()).collect(),
            sense,
            quantity,
            absolute_tolerance,
            relative_tolerance,
            bound: None,
        })
    }
    /// `objective_bounds` for one earlier level: the parameter β of the level type, and
    /// the identity of the row bounding the level value by it.
    fn objective_bound(
        &mut self,
        objectives: &Objectives,
        position: usize,
    ) -> Result<ObjectiveBound> {
        let level = &objectives.levels[position];
        let lead = &objectives.members[level.members[0]];
        // A staged level needs a positive tolerance; it is certainly zero when both
        // declared tolerances are zero (ADR-0111 item 3).
        if level.absolute_tolerance == Some(0.0) && level.relative_tolerance == Some(0.0) {
            return Err(refuse(
                lead.lineage.declaration.as_id(),
                Refusal::ZeroTolerance,
            ));
        }
        let seed = pse_ids::named_id(lead.target, "objective-level");
        let parameter = pse_ids::named_id(seed, "objective-bound");
        let mut provenance = lead.lineage.clone();
        provenance.path = format!("{}.objective_bound", lead.lineage.path);
        self.reserve(1)?;
        self.model.symbols.insert(
            parameter,
            Symbol {
                id: parameter,
                ty: Type::Quantity(pse_quantity::scheme::Scheme::Concrete(level.quantity)),
                role: Kind::Parameter,
                domain: Domain::Continuous,
                expression: None,
                initial: None,
                lineage: provenance,
            },
        );
        Ok(ObjectiveBound {
            parameter,
            row: pse_ids::named_id(seed, "objective-bound-row"),
        })
    }
    /// The generated normalization parameter of a member and its quotient term, typed by
    /// the package's quantity operations: a member kind without a declared ratio to
    /// itself cannot be normalized.
    fn normalized_term(
        &mut self,
        target: SemanticId,
        normalization: &Value,
        lineage: &Lineage,
    ) -> Result<(SemanticId, SemanticId)> {
        let at = lineage.declaration;
        let parameter = pse_ids::named_id(target, "objective-normalization");
        let term = pse_ids::named_id(target, "objective-term");
        let target_ty = self.model.symbols[&target].ty.clone();
        let mut provenance = lineage.clone();
        provenance.path = format!("{}.objective_normalization", lineage.path);
        self.reserve(2)?;
        self.model.symbols.insert(
            parameter,
            Symbol {
                id: parameter,
                ty: target_ty,
                role: Kind::Parameter,
                domain: Domain::Continuous,
                expression: None,
                initial: Some(normalization.clone()),
                lineage: provenance.clone(),
            },
        );
        let expression = Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Div,
                lhs: Box::new(symbol_expr(target)),
                rhs: Box::new(symbol_expr(parameter)),
            },
            span: Span::default(),
        };
        let types = self
            .model
            .symbols
            .iter()
            .map(|(id, s)| (symbol_name(*id), s.ty.clone()))
            .collect::<BTreeMap<_, _>>();
        let ty = crate::expression::infer(
            &expression,
            &types,
            &self.model.function_contracts(self.p),
            self.c,
            at,
            None,
        )?;
        provenance.path = format!("{}.objective_term", lineage.path);
        self.model.symbols.insert(
            term,
            Symbol {
                id: term,
                ty,
                role: Kind::Let,
                domain: Domain::Continuous,
                expression: Some(expression),
                initial: None,
                lineage: provenance,
            },
        );
        Ok((parameter, term))
    }
    fn objective_quantity(&self, symbol: SemanticId, at: DeclarationId) -> Result<QuantityTypeId> {
        let Type::Quantity(scheme) = &self.model.symbols[&symbol].ty else {
            return Err(invalid(at, "objective requires a physical member"));
        };
        scheme
            .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
            .map_err(|e| invalid(at, e.to_string()))
    }
}
/// An objective member as authored: target, evaluated declaration and lineage.
type Declared = (SemanticId, ObjectiveDeclaration, Lineage);
/// The one tolerance the members of a level declare, if any.
fn agreed(
    values: impl Iterator<Item = Result<Option<f64>>>,
    at: DeclarationId,
) -> Result<Option<f64>> {
    let mut agreed = None;
    for value in values {
        if let Some(value) = value? {
            if agreed.is_some_and(|a: f64| a.to_bits() != value.to_bits()) {
                return Err(refuse(at.as_id(), Refusal::ConflictingTolerance));
            }
            agreed = Some(value);
        }
    }
    Ok(agreed)
}
fn refuse(declaration: SemanticId, reason: Refusal) -> ModelingError {
    ModelingError::Objective {
        declaration,
        reason,
    }
}
