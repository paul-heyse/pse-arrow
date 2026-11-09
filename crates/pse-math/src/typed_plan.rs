// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Atom-free scheduling for the subset with a source-determined stage layout.
use super::*;

/// Only distinctions that can be established without normalization. An unknown
/// value can feed an arithmetic block but cannot decide whether to elide one.
#[derive(Clone, Copy, Debug)]
pub(super) enum Shape {
    Formal,
    Number(f64),
    Unknown,
}

impl Shape {
    pub(super) fn scaled(self, scale: f64) -> Self {
        if scale == 1.0 { self } else { Self::Unknown }
    }
}

pub(super) fn unsupported() -> MathError {
    MathError::Contract("construction schedule requires normalized arithmetic".into())
}

/// Non-executable correspondence for one selected source walk. Blocks carry their
/// source and destinations, not expressions. The original sealed arithmetic and
/// its symbolic dependencies still need independent custody and admission.
#[derive(Debug)]
pub struct ConstructionPlan {
    stages: Vec<Stage>,
    slots: usize,
    input_quantities: Vec<Option<QuantityTypeId>>,
    outputs: Vec<usize>,
    quantities: Vec<QuantityTypeId>,
    effects: Vec<BTreeSet<usize>>,
    occurrences: usize,
}

impl ConstructionPlan {
    /// Compare the source-determined schedule to the producer's actual guarded
    /// schedule, including exact slots, lineage, branch order and output effects.
    /// This check confers no authority to restore or execute a symbolic payload.
    pub fn verify_producer(&self, body: &PreparedBody) -> Result<(), MathError> {
        if self.slots != body.slot_count()
            || self.input_quantities.as_slice() != body.input_quantities()
            || self.quantities.as_slice() != body.output_quantities()
            || self.outputs != body.outputs
            || Some(&self.effects) != body.output_effects.as_ref()
            || self.occurrences != body.occurrence_count()
            || !stages_match(&self.stages, &body.stages)
        {
            return Err(MathError::Contract(
                "source schedule differs from original guarded stage correspondence".into(),
            ));
        }
        Ok(())
    }
}

fn stages_match(planned: &[Stage], original: &[Stage]) -> bool {
    planned.len() == original.len()
        && planned.iter().zip(original).all(|(plan, original)| match (plan, original) {
            (
                Stage::Block { outputs: a, source: sa, .. },
                Stage::Block { outputs: b, source: sb, expressions },
            ) => a == b && sa == sb && b.len() == expressions.len(),
            (
                Stage::Branch { continuity: ca, comparison: oa, left: la, right: ra, then: ta, otherwise: ea },
                Stage::Branch { continuity: cb, comparison: ob, left: lb, right: rb, then: tb, otherwise: eb },
            ) => ca == cb && oa == ob && la == lb && ra == rb && stages_match(ta, tb) && stages_match(ea, eb),
            (
                Stage::Domain { stages: sa, argument: aa, token: ta, lineage: la },
                Stage::Domain { stages: sb, argument: ab, token: tb, lineage: lb },
            ) => aa == ab && ta == tb && la == lb && stages_match(sa, sb),
            (
                Stage::Applicability { stages: sa, predicates: pa, inputs: ia, token: ta, plan: ea },
                Stage::Applicability { stages: sb, predicates: pb, inputs: ib, token: tb, plan: eb },
            ) => pa == pb && ia == ib && ta == tb && ea == eb && stages_match(sa, sb),
            (Stage::Require { .. }, Stage::Require { .. }) => plan == original,
            _ => false,
        })
}

impl<'a> BodyBuilder<'a> {
    /// Select source-only scheduling. The same typed constructors consume the
    /// original physical and proof receipts; nothing may create a Symbolica atom.
    /// Ambiguous normalized shapes, provider equality and partial schedules refuse.
    pub fn planning(
        registry: &'a QuantityRegistry,
        checker: &'a dyn InvariantChecker,
        inputs: usize,
        limits: BodyLimits,
    ) -> Result<Self, MathError> {
        let mut builder = Self::new_inner(None, registry, checker, inputs, limits)?;
        builder.planning = true;
        Ok(builder)
    }

    pub(super) fn atom_with(
        &self,
        create: impl FnOnce() -> Result<Atom, MathError>,
    ) -> Result<Option<Atom>, MathError> {
        if self.planning { Ok(None) } else { create().map(Some) }
    }

    pub(super) fn symbolic_context(&self) -> Result<&'a crate::SymbolicContext, MathError> {
        self.context.ok_or_else(unsupported)
    }

    /// Close the same ordered output layout without preparing symbolic support.
    pub fn finish_plan(mut self, values: &[TypedValue]) -> Result<ConstructionPlan, MathError> {
        if !self.planning {
            return Err(MathError::Contract("expected a source-only construction plan".into()));
        }
        let outputs = values
            .iter()
            .map(|value| self.materialize(None, value.source))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ConstructionPlan {
            stages: self.stages,
            slots: self.next_slot,
            input_quantities: self.input_quantities,
            outputs,
            quantities: values.iter().map(TypedValue::quantity).collect::<Result<_, _>>()?,
            effects: values.iter().map(|value| value.effects.clone()).collect(),
            occurrences: self.occurrences,
        })
    }
}
