// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Completion receipts belong to one immutable assembly and one attempt scope.
use super::{DerivativeOrder, EvaluationDemand, MathError};
use enum_map::EnumMap;

#[derive(Clone, Copy, Debug, enum_map::Enum)]
pub(super) enum AssembledProduct {
    Objective,
    Constraints,
    Gradient,
    Jacobian,
    Hessian,
    JacobianAction,
    TransposeJacobianAction,
    HessianAction,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_generation_overflow_refuses_and_revokes_without_wrapping() {
        let mut state = AssemblyCompletion::default();
        state.generations[EvaluationDemand::Objective] = u64::MAX;
        state.completed(
            AssembledProduct::Objective,
            EvaluationDemand::Objective,
            DerivativeOrder::Value,
        );
        assert!(state.reusable(
            AssembledProduct::Objective,
            EvaluationDemand::Objective,
            DerivativeOrder::Value
        ));
        assert!(matches!(
            state.replaced(EvaluationDemand::Objective),
            Err(MathError::Limit("assembled completion generation"))
        ));
        assert_eq!(state.generations[EvaluationDemand::Objective], u64::MAX);
        assert!(!state.reusable(
            AssembledProduct::Objective,
            EvaluationDemand::Objective,
            DerivativeOrder::Value
        ));
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Receipt {
    demand: EvaluationDemand,
    generation: u64,
    order: DerivativeOrder,
}
#[derive(Debug, Default)]
pub(super) struct AssemblyCompletion {
    generations: EnumMap<EvaluationDemand, u64>,
    receipts: EnumMap<AssembledProduct, Option<Receipt>>,
    pub(super) objective: f64,
    fixed_revision: Option<u64>,
    coordinates: Vec<u64>,
    point_revision: u64,
    pub(super) jacobian_point: Option<u64>,
    pub(super) hessian_point: Option<u64>,
    pub(super) hessian_weights: Vec<u64>,
    action_point: Option<u64>,
    action_direction: Vec<u64>,
    #[cfg(test)]
    pub(super) assemblies: EnumMap<AssembledProduct, u64>,
    #[cfg(test)]
    pub(super) evaluations: EnumMap<EvaluationDemand, u64>,
}
impl AssemblyCompletion {
    pub(super) fn new(coordinates: usize, rows: usize) -> Self {
        Self {
            coordinates: vec![0; coordinates],
            action_direction: vec![0; coordinates],
            hessian_weights: vec![0; rows + 1],
            ..Self::default()
        }
    }
    /// Only an immutable, finite trial issued by BoundValues carries this premise.
    /// Semantic maps remain uncontrolled inputs and use the ordinary local keys.
    pub(super) fn point(
        &mut self,
        values: super::EvalInputs<'_>,
    ) -> Result<Option<u64>, MathError> {
        let super::InputSource::Trial(bound, coordinates) = values.0 else {
            return Ok(None);
        };
        if self.fixed_revision != Some(bound.revision)
            || self
                .coordinates
                .iter()
                .zip(coordinates)
                .any(|(bits, value)| *bits != value.to_bits())
        {
            let Some(next) = self.point_revision.checked_add(1) else {
                self.invalidate();
                return Err(MathError::Limit("assembled point revision"));
            };
            self.point_revision = next;
            self.fixed_revision = Some(bound.revision);
            for (bits, value) in self.coordinates.iter_mut().zip(coordinates) {
                *bits = value.to_bits();
            }
        }
        Ok(Some(self.point_revision))
    }
    pub(super) fn same_weights(&self, objective: f64, multipliers: &[f64]) -> bool {
        self.hessian_weights
            .iter()
            .copied()
            .eq(std::iter::once(objective.to_bits())
                .chain(multipliers.iter().map(|value| value.to_bits())))
    }
    pub(super) fn set_weights(&mut self, objective: f64, multipliers: &[f64]) {
        for (bits, value) in self
            .hessian_weights
            .iter_mut()
            .zip(std::iter::once(objective).chain(multipliers.iter().copied()))
        {
            *bits = value.to_bits();
        }
    }
    pub(super) fn same_action(&self, point: Option<u64>, direction: &[f64]) -> bool {
        point.is_some()
            && self.action_point == point
            && self
                .action_direction
                .iter()
                .copied()
                .eq(direction.iter().map(|value| value.to_bits()))
    }
    pub(super) fn set_action(&mut self, point: Option<u64>, direction: &[f64]) {
        self.action_point = point;
        for (bits, value) in self.action_direction.iter_mut().zip(direction) {
            *bits = value.to_bits();
        }
    }
    pub(super) fn replaced(&mut self, demand: EvaluationDemand) -> Result<(), MathError> {
        let Some(next) = self.generations[demand].checked_add(1) else {
            self.invalidate();
            return Err(MathError::Limit("assembled completion generation"));
        };
        self.generations[demand] = next;
        Ok(())
    }
    pub(super) fn invalidate(&mut self) {
        self.receipts
            .values_mut()
            .for_each(|receipt| *receipt = None);
    }
    pub(super) fn reusable(
        &self,
        product: AssembledProduct,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) -> bool {
        self.receipts[product]
            == Some(Receipt {
                demand,
                generation: self.generations[demand],
                order,
            })
    }
    pub(super) fn begin(&mut self, product: AssembledProduct) {
        self.receipts[product] = None;
        // Both actions use one completed staging buffer.
        if matches!(
            product,
            AssembledProduct::JacobianAction | AssembledProduct::TransposeJacobianAction | AssembledProduct::HessianAction
        ) {
            self.action_point = None;
            self.receipts[AssembledProduct::JacobianAction] = None;
            self.receipts[AssembledProduct::TransposeJacobianAction] = None;
            self.receipts[AssembledProduct::HessianAction] = None;
        }
        #[cfg(test)]
        {
            self.assemblies[product] += 1;
        }
    }
    pub(super) fn forget(&mut self, product: AssembledProduct) {
        self.receipts[product] = None;
    }
    pub(super) fn completed(
        &mut self,
        product: AssembledProduct,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) {
        self.receipts[product] = Some(Receipt {
            demand,
            generation: self.generations[demand],
            order,
        });
    }
}
