// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Composition of admitted suppliers, without manufacturing a selected root sheet.
use crate::{
    MathError,
    derived::{
        DerivativeSupport, OriginalContract, ReconstructionAdmission, ReconstructionContract,
        ReconstructionObservation, ReconstructionOracle, RefinementLimits,
    },
    index::{Entry, GlobalCol, GlobalRow},
};
use pse_ids::{ContentHash, FramedHasher};
use pse_kernels::DerivativeOrder;
use pse_model::strategy::{AccuracyClass, AccuracyDemand, AccuracyEvidence};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Actual worker-local suppliers in prerequisite order. Every supplier retains its
/// original local contract, sheet admission and error observation.
#[derive(Debug)]
pub struct CompositeReconstruction<R> {
    contract: Arc<ReconstructionContract>,
    suppliers: Vec<R>,
    columns: Vec<Vec<Option<usize>>>,
}

impl<R: ReconstructionOracle> CompositeReconstruction<R> {
    /// Bind a compiler-derived global projection to actual local supplier workers.
    pub fn new(
        contract: Arc<ReconstructionContract>,
        suppliers: Vec<R>,
    ) -> Result<Self, MathError> {
        let expected = Self::prepare_contract(
            Arc::new(contract.original().clone()),
            contract.retained().to_vec(),
            &suppliers
                .iter()
                .map(|s| Arc::new(s.contract().clone()))
                .collect::<Vec<_>>(),
        )?;
        if expected.as_ref() != contract.as_ref() {
            return Err(MathError::Contract(
                "composite supplier contract mismatch".into(),
            ));
        }
        let columns = suppliers
            .iter()
            .map(|supplier| local_columns(contract.original(), supplier.contract()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            contract,
            suppliers,
            columns,
        })
    }
    /// Compose exact projections of the original physical scales. A smaller local
    /// normalization identity does not by itself prove correspondence.
    pub fn new_normalized(
        contract: Arc<ReconstructionContract>,
        suppliers: Vec<R>,
        normalization: &crate::normalization::Normalization,
        local: &[crate::normalization::Normalization],
    ) -> Result<Self, MathError> {
        let contracts = suppliers
            .iter()
            .map(|s| Arc::new(s.contract().clone()))
            .collect::<Vec<_>>();
        let expected = Self::prepare_contract_normalized(
            Arc::new(contract.original().clone()),
            contract.retained().to_vec(),
            &contracts,
            normalization,
            local,
        )?;
        if expected.as_ref() != contract.as_ref() {
            return Err(MathError::Contract(
                "composite normalized contract mismatch".into(),
            ));
        }
        let columns = suppliers
            .iter()
            .map(|s| local_columns(contract.original(), s.contract()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            contract,
            suppliers,
            columns,
        })
    }
    /// Validate source-coordinate scale correspondence before admitting local errors.
    pub fn prepare_contract_normalized(
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        suppliers: &[Arc<ReconstructionContract>],
        normalization: &crate::normalization::Normalization,
        local: &[crate::normalization::Normalization],
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        normalization.validate(original.coordinates().len(), original.constraints().len())?;
        if normalization.key() != original.normalization() || local.len() != suppliers.len() {
            return Err(MathError::Contract(
                "composite original normalization correspondence".into(),
            ));
        }
        for (supplier, scales) in suppliers.iter().zip(local) {
            scales.validate(
                supplier.original().coordinates().len(),
                supplier.original().constraints().len(),
            )?;
            let columns = local_columns(&original, supplier)?;
            if scales.key() != supplier.original().normalization()
                || scales.objective != normalization.objective
                || columns.iter().enumerate().any(|(i, c)| {
                    c.is_some_and(|c| scales.variables[i] != normalization.variables[c])
                })
                || supplier
                    .original()
                    .constraints()
                    .iter()
                    .enumerate()
                    .any(|(i, row)| {
                        original
                            .constraints()
                            .iter()
                            .position(|r| r == row)
                            .is_none_or(|r| scales.rows[i] != normalization.rows[r])
                    })
            {
                return Err(MathError::Contract(
                    "composite local physical scales differ from original".into(),
                ));
            }
        }
        Self::prepare_contract_inner(original, retained, suppliers, true)
    }
    /// Derive global identity, row coverage and chain support from admitted local
    /// contracts. Matching is inherited; this supplies no new regularity proof.
    pub fn prepare_contract(
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        suppliers: &[Arc<ReconstructionContract>],
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        Self::prepare_contract_inner(original, retained, suppliers, false)
    }
    fn prepare_contract_inner(
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        suppliers: &[Arc<ReconstructionContract>],
        normalized: bool,
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        if suppliers.is_empty() {
            return Err(MathError::Contract("empty reconstruction suppliers".into()));
        }
        let mut dependencies: BTreeMap<usize, BTreeSet<usize>> = retained
            .iter()
            .enumerate()
            .map(|(local, global)| (global.get(), BTreeSet::from([local])))
            .collect();
        let mut eliminated = BTreeSet::new();
        let mut source = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
        let mut validity = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
        source.str("composite-admitted-reconstruction");
        validity.str("composite-admitted-validity");
        let mut order = DerivativeOrder::First;
        let mut action = true;
        for supplier in suppliers {
            let columns = local_columns(&original, supplier)?;
            if !normalized && supplier.original().normalization() != original.normalization() {
                return Err(MathError::Contract(
                    "composite normalization mismatch".into(),
                ));
            }
            let inputs: Vec<_> = supplier
                .retained()
                .iter()
                .map(|c| columns[c.get()])
                .collect();
            if inputs
                .iter()
                .flatten()
                .any(|c| !dependencies.contains_key(c))
            {
                return Err(MathError::Contract(
                    "composite predecessor input unavailable".into(),
                ));
            }
            for (local, global) in columns
                .iter()
                .enumerate()
                .filter(|(local, _)| !supplier.retained().contains(&GlobalCol::new(*local)))
            {
                let global = global.ok_or_else(|| {
                    MathError::Contract(
                        "fixed supplier coordinate must remain a retained input".into(),
                    )
                })?;
                if dependencies.contains_key(&global) {
                    return Err(MathError::Contract(
                        "overlapping composite supplier output".into(),
                    ));
                }
                let support = supplier
                    .incidence()
                    .iter()
                    .filter(|edge| edge.row.get() == local)
                    .flat_map(|edge| {
                        inputs[edge.col.get()]
                            .into_iter()
                            .flat_map(|input| dependencies[&input].iter().copied())
                    })
                    .collect();
                dependencies.insert(global, support);
            }
            for row in supplier.eliminated() {
                let constraint = &supplier.original().constraints()[row.get()];
                let global = original
                    .constraints()
                    .iter()
                    .position(|c| c == constraint)
                    .ok_or_else(|| {
                        MathError::Contract("composite original row correspondence".into())
                    })?;
                if !eliminated.insert(GlobalRow::new(global)) {
                    return Err(MathError::Contract(
                        "overlapping composite eliminated row".into(),
                    ));
                }
            }
            source.hash(&supplier.key());
            validity.hash(&supplier.validity());
            order = order.min(supplier.support().order);
            action &= supplier.support().jacobian_product;
        }
        if dependencies.len() != original.coordinates().len() {
            return Err(MathError::Contract(
                "incomplete composite coordinate reconstruction".into(),
            ));
        }
        let incidence = dependencies
            .into_iter()
            .flat_map(|(row, cols)| {
                cols.into_iter()
                    .map(move |col| Entry::new(GlobalCol::new(row), GlobalCol::new(col)))
            })
            .collect();
        let source = source.finish_hash();
        let eliminated: Vec<_> = eliminated.into_iter().collect();
        let offsets: Vec<_> = eliminated
            .iter()
            .map(|r| (*r, original.constraints()[r.get()].lower))
            .collect();
        Ok(Arc::new(ReconstructionContract::new_with_offsets(
            original,
            crate::derived::ReconstructionProducer {
                source,
                validity: validity.finish_hash(),
                support: DerivativeSupport {
                    order,
                    jacobian_product: action,
                    source,
                },
            },
            retained,
            eliminated,
            incidence,
            &offsets,
        )?))
    }
    fn evaluate(
        &mut self,
        x: &[f64],
        direction: Option<&[f64]>,
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<ReconstructionObservation, R::Error> {
        refinement.validate()?;
        demand
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        if demand.normalization != self.contract.original().normalization()
            || x.len() != self.contract.retained().len()
            || x.iter().any(|v| !v.is_finite())
            || direction.is_some_and(|v| v.len() != x.len() || v.iter().any(|v| !v.is_finite()))
        {
            return Err(
                MathError::Contract("composite reduced point/action extent/value".into()).into(),
            );
        }
        let n = self.contract.original().coordinates().len();
        let operations = self
            .suppliers
            .len()
            .checked_mul(if direction.is_some() { 2 } else { 1 })
            .ok_or(MathError::Limit("composite refinement operations"))?;
        let local_refinement = RefinementLimits {
            rounds: refinement.rounds / operations,
            proof_cells: refinement.proof_cells / operations as u64,
        };
        local_refinement.validate()?;
        let mut values = vec![0.0; n];
        let mut actions = vec![0.0; n];
        let mut point_errors = vec![0.0; n];
        let mut action_errors = vec![0.0; n];
        for (i, column) in self.contract.retained().iter().enumerate() {
            values[column.get()] = x[i];
            actions[column.get()] = direction.map_or(0.0, |v| v[i]);
        }
        let mut class = AccuracyClass::Certified;
        for (supplier, columns) in self.suppliers.iter_mut().zip(&self.columns) {
            let inputs: Vec<_> = supplier
                .contract()
                .retained()
                .iter()
                .map(|c| columns[c.get()])
                .collect();
            let local_x: Vec<_> = inputs
                .iter()
                .zip(supplier.contract().retained())
                .map(|(c, local)| {
                    c.map_or(
                        supplier.contract().original().coordinates()[local.get()].lower,
                        |c| values[c],
                    )
                })
                .collect();
            let local_v: Vec<_> = inputs
                .iter()
                .map(|c| c.map_or(0.0, |c| actions[c]))
                .collect();
            let incoming_point: Vec<_> = inputs
                .iter()
                .map(|c| c.map_or(0.0, |c| point_errors[c]))
                .collect();
            let incoming_action: Vec<_> = inputs
                .iter()
                .map(|c| c.map_or(0.0, |c| action_errors[c]))
                .collect();
            if !supplier.supports_uncertainty()
                && incoming_point
                    .iter()
                    .chain(&incoming_action)
                    .any(|e| *e != 0.0)
            {
                return Err(MathError::Refinement {
                    product: demand.product,
                    source_key: self.contract.source(),
                    validity: self.contract.validity(),
                    reason: crate::derived::RefinementRefusal::Unavailable(
                        crate::implicit::SelectionProofRefusal::Unsupported,
                    ),
                }
                .into());
            }
            supplier.admit(&local_x)?;
            let product = |action: bool| {
                let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
                h.str(if action {
                    "composite-consumed-action"
                } else {
                    "composite-consumed-point"
                })
                .hash(&demand.product)
                .hash(&supplier.contract().key())
                .hash(&supplier.realization());
                for (value, error) in local_x.iter().zip(&incoming_point) {
                    h.f64(*value).f64(*error);
                }
                h.str(match class {
                    AccuracyClass::Certified => "certified",
                    _ => "estimated",
                });
                if action {
                    for (value, error) in local_v.iter().zip(&incoming_action) {
                        h.f64(*value).f64(*error);
                    }
                }
                h.finish_hash()
            };
            let local = AccuracyDemand {
                product: product(false),
                normalization: supplier.contract().original().normalization(),
                ..*demand
            };
            let local_action = AccuracyDemand {
                product: product(true),
                ..local
            };
            let point = supplier.uncertain(
                &local_x,
                None,
                crate::derived::ReconstructionUncertainty {
                    point: &incoming_point,
                    action: None,
                    class,
                },
                &local,
                local_refinement,
            );
            let point = consumed(point, supplier, &self.contract, demand)?;
            if !point.accuracy.satisfies(&local)
                || point.values.len() != columns.len()
                || point.values.iter().any(|v| !v.is_finite())
            {
                return Err(
                    MathError::Contract("composite point accuracy/extent refusal".into()).into(),
                );
            }
            let observation = match direction {
                Some(_) => {
                    let result = supplier.uncertain(
                        &local_x,
                        Some(&local_v),
                        crate::derived::ReconstructionUncertainty {
                            point: &incoming_point,
                            action: Some(&incoming_action),
                            class,
                        },
                        &local_action,
                        local_refinement,
                    );
                    consumed(result, supplier, &self.contract, demand)?
                }
                None => point.clone(),
            };
            let consumed = if direction.is_some() {
                &local_action
            } else {
                &local
            };
            if !observation.accuracy.satisfies(consumed)
                || observation.values.len() != columns.len()
                || observation.values.iter().any(|v| !v.is_finite())
            {
                return Err(MathError::Contract(
                    "composite consumed accuracy/extent refusal".into(),
                )
                .into());
            }
            for (input, local_col) in supplier.contract().retained().iter().enumerate() {
                if point.values[local_col.get()] != local_x[input]
                    || direction.is_some() && observation.values[local_col.get()] != local_v[input]
                {
                    return Err(MathError::Contract(
                        "composite supplier retained identity refusal".into(),
                    )
                    .into());
                }
            }
            if point.accuracy.class != AccuracyClass::Certified
                || observation.accuracy.class != AccuracyClass::Certified
            {
                class = AccuracyClass::Estimated;
            }
            for (local_col, global) in columns.iter().enumerate() {
                if supplier
                    .contract()
                    .retained()
                    .contains(&GlobalCol::new(local_col))
                {
                    continue;
                }
                let global =
                    global.ok_or_else(|| MathError::Contract("unmapped supplier output".into()))?;
                values[global] = point.values[local_col];
                actions[global] = observation.values[local_col];
                point_errors[global] = point.accuracy.error.unwrap_or(f64::INFINITY);
                action_errors[global] = observation.accuracy.error.unwrap_or(f64::INFINITY);
            }
        }
        let error = if direction.is_some() {
            action_errors
        } else {
            point_errors
        }
        .into_iter()
        .fold(0.0_f64, f64::max);
        let accuracy = AccuracyEvidence {
            product: demand.product,
            normalization: demand.normalization,
            class,
            error: Some(error),
        };
        if !accuracy.satisfies(demand) {
            return Err(MathError::Contract("composite final accuracy refusal".into()).into());
        }
        Ok(ReconstructionObservation {
            values: if direction.is_some() { actions } else { values },
            accuracy,
        })
    }
}
impl<R: ReconstructionOracle> ReconstructionOracle for CompositeReconstruction<R> {
    type Error = R::Error;
    fn contract(&self) -> &ReconstructionContract {
        &self.contract
    }
    fn realization(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
        h.str("composite-actual-suppliers")
            .hash(&self.contract.key());
        for supplier in &self.suppliers {
            h.hash(&supplier.realization());
        }
        h.finish_hash()
    }
    fn admit(&mut self, x: &[f64]) -> Result<ReconstructionAdmission, Self::Error> {
        if x.len() != self.contract.retained().len() || x.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Contract("composite input admission".into()).into());
        }
        let mut values = vec![0.0; self.contract.original().coordinates().len()];
        for (column, value) in self.contract.retained().iter().zip(x) {
            values[column.get()] = *value;
        }
        // Establish every actual selected sheet before the enclosing consumer captures
        // its product identity. Children use their predecessors' evaluated points,
        // without treating these admission readouts as accuracy evidence.
        for (supplier, columns) in self.suppliers.iter_mut().zip(&self.columns) {
            let local_x: Vec<_> = supplier
                .contract()
                .retained()
                .iter()
                .map(|c| {
                    columns[c.get()].map_or(
                        supplier.contract().original().coordinates()[c.get()].lower,
                        |global| values[global],
                    )
                })
                .collect();
            let admitted = supplier.admit(&local_x)?;
            if admitted.values.len() != columns.len()
                || admitted.values.iter().any(|v| !v.is_finite())
            {
                return Err(MathError::Contract(
                    "composite admitted point extent/value refusal".into(),
                )
                .into());
            }
            for (column, value) in supplier.contract().retained().iter().zip(&local_x) {
                if admitted.values[column.get()] != *value {
                    return Err(MathError::Contract(
                        "composite admitted retained identity refusal".into(),
                    )
                    .into());
                }
            }
            for (local, global) in columns.iter().enumerate() {
                if let Some(global) = global {
                    values[*global] = admitted.values[local];
                }
            }
        }
        Ok(ReconstructionAdmission { values })
    }
    fn point(
        &mut self,
        x: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<ReconstructionObservation, Self::Error> {
        self.evaluate(x, None, demand, refinement)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<ReconstructionObservation, Self::Error> {
        self.evaluate(x, Some(direction), demand, refinement)
    }
}
fn local_columns(
    original: &OriginalContract,
    supplier: &ReconstructionContract,
) -> Result<Vec<Option<usize>>, MathError> {
    supplier
        .original()
        .coordinates()
        .iter()
        .enumerate()
        .map(|(local, coordinate)| {
            original
                .coordinates()
                .iter()
                .position(|c| c == coordinate)
                .map(Some)
                .or_else(|| {
                    (coordinate.lower.is_finite()
                        && coordinate.lower.to_bits() == coordinate.upper.to_bits()
                        && supplier.retained().contains(&GlobalCol::new(local)))
                    .then_some(None)
                })
                .ok_or_else(|| {
                    MathError::Contract("composite original coordinate correspondence".into())
                })
        })
        .collect()
}
fn consumed<R: ReconstructionOracle>(
    result: Result<ReconstructionObservation, R::Error>,
    supplier: &R,
    contract: &ReconstructionContract,
    demand: &AccuracyDemand,
) -> Result<ReconstructionObservation, R::Error> {
    match result {
        Err(error) => match supplier.refinement_refusal() {
            Some(reason) => Err(MathError::Refinement {
                product: demand.product,
                source_key: contract.source(),
                validity: contract.validity(),
                reason,
            }
            .into()),
            None => Err(error),
        },
        success => success,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derived::{Constraint, Coordinate, DerivedFamily, Oracle, OriginalObligations};
    use std::{cell::Cell, rc::Rc};
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    fn id(n: u8) -> pse_ids::SemanticId {
        pse_ids::SemanticId::from_bytes([n; 16])
    }
    fn original() -> Arc<OriginalContract> {
        Arc::new(
            OriginalContract::new(
                hash(1),
                hash(2),
                (1..=3)
                    .map(|n| Coordinate {
                        id: id(n),
                        lower: -100.0,
                        upper: 100.0,
                    })
                    .collect(),
                (1..=2)
                    .map(|n| Constraint {
                        id: id(n + 10),
                        lower: 0.0,
                        upper: 0.0,
                    })
                    .collect(),
                vec![],
                DerivativeSupport {
                    order: DerivativeOrder::First,
                    jacobian_product: true,
                    source: hash(3),
                },
                OriginalObligations {
                    guards: hash(4),
                    selection: hash(5),
                    objective: None,
                },
            )
            .unwrap(),
        )
    }
    #[derive(Debug)]
    struct Linear {
        contract: Arc<ReconstructionContract>,
        error: f64,
        admitted: Rc<Cell<Option<f64>>>,
    }
    impl ReconstructionOracle for Linear {
        type Error = MathError;
        fn contract(&self) -> &ReconstructionContract {
            &self.contract
        }
        fn realization(&self) -> ContentHash {
            let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
            h.hash(&hash(9));
            match self.admitted.get() {
                Some(anchor) => {
                    h.bool(true).f64(anchor);
                }
                None => {
                    h.bool(false);
                }
            }
            h.finish_hash()
        }
        fn admit(&mut self, x: &[f64]) -> Result<ReconstructionAdmission, MathError> {
            if self.admitted.get().is_none() {
                self.admitted.set(Some(x[0]));
            }
            Ok(ReconstructionAdmission {
                values: vec![x[0], 2.0 * x[0]],
            })
        }
        fn point(
            &mut self,
            x: &[f64],
            demand: &AccuracyDemand,
            _: RefinementLimits,
        ) -> Result<ReconstructionObservation, MathError> {
            Ok(ReconstructionObservation {
                values: vec![x[0], 2.0 * x[0]],
                accuracy: AccuracyEvidence {
                    product: demand.product,
                    normalization: demand.normalization,
                    class: AccuracyClass::Certified,
                    error: Some(self.error),
                },
            })
        }
        fn jacobian_product(
            &mut self,
            _: &[f64],
            v: &[f64],
            demand: &AccuracyDemand,
            limits: RefinementLimits,
        ) -> Result<ReconstructionObservation, MathError> {
            self.point(v, demand, limits)
        }
    }
    #[derive(Debug)]
    struct UniformLinear(Linear);
    impl ReconstructionOracle for UniformLinear {
        type Error = MathError;
        fn contract(&self) -> &ReconstructionContract {
            self.0.contract()
        }
        fn realization(&self) -> ContentHash {
            self.0.realization()
        }
        fn supports_uncertainty(&self) -> bool {
            true
        }
        fn admit(&mut self, x: &[f64]) -> Result<ReconstructionAdmission, MathError> {
            self.0.admit(x)
        }
        fn point(
            &mut self,
            x: &[f64],
            d: &AccuracyDemand,
            r: RefinementLimits,
        ) -> Result<ReconstructionObservation, MathError> {
            self.0.point(x, d, r)
        }
        fn jacobian_product(
            &mut self,
            x: &[f64],
            v: &[f64],
            d: &AccuracyDemand,
            r: RefinementLimits,
        ) -> Result<ReconstructionObservation, MathError> {
            self.0.jacobian_product(x, v, d, r)
        }
        fn uncertain(
            &mut self,
            x: &[f64],
            direction: Option<&[f64]>,
            incoming: crate::derived::ReconstructionUncertainty<'_>,
            d: &AccuracyDemand,
            r: RefinementLimits,
        ) -> Result<ReconstructionObservation, MathError> {
            let mut result = self.0.point(direction.unwrap_or(x), d, r)?;
            // The exact authored map z=2x has a uniform amplification 2 with no
            // state dependence; derivative actions inherit only direction error.
            result.accuracy.error = Some(
                self.0.error
                    + 2.0 * direction.map_or(incoming.point[0], |_| incoming.action.unwrap()[0]),
            );
            result.accuracy.class = incoming.class;
            Ok(result)
        }
    }
    fn supplier(
        original: &Arc<OriginalContract>,
        input: usize,
        output: usize,
        row: usize,
        error: f64,
    ) -> Linear {
        let local = Arc::new(
            OriginalContract::new(
                hash((output + 20) as u8),
                original.normalization(),
                vec![
                    original.coordinates()[input].clone(),
                    original.coordinates()[output].clone(),
                ],
                vec![original.constraints()[row].clone()],
                vec![],
                original.support(),
                original.obligations(),
            )
            .unwrap(),
        );
        let contract = Arc::new(
            ReconstructionContract::new(
                local,
                hash((output + 30) as u8),
                hash(7),
                vec![0.into()],
                vec![0.into()],
                vec![
                    Entry::new(0.into(), 0.into()),
                    Entry::new(1.into(), 0.into()),
                ],
                original.support(),
            )
            .unwrap(),
        );
        Linear {
            contract,
            error,
            admitted: Rc::new(Cell::new(None)),
        }
    }
    #[derive(Debug)]
    struct LinearOriginal {
        contract: Arc<OriginalContract>,
        chain: bool,
    }
    impl Oracle for LinearOriginal {
        type Error = MathError;
        fn contract(&self) -> &OriginalContract {
            &self.contract
        }
        fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), MathError> {
            out[0] = x[1] - 2.0 * x[0];
            out[1] = x[2] - 2.0 * x[usize::from(self.chain)];
            Ok(())
        }
        fn jacobian_product(
            &mut self,
            _: &[f64],
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), MathError> {
            self.values(v, out)
        }
    }
    #[test]
    fn composite_admission_binds_actual_child_sheets_before_consumed_demand() {
        for chain in [false, true] {
            let original = original();
            let workers = vec![
                supplier(&original, 0, 1, 0, 0.0),
                supplier(&original, usize::from(chain), 2, 1, 0.0),
            ];
            let admissions: Vec<_> = workers.iter().map(|w| w.admitted.clone()).collect();
            let contract = CompositeReconstruction::<Linear>::prepare_contract(
                original.clone(),
                vec![0.into()],
                &workers
                    .iter()
                    .map(|s| s.contract.clone())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            let family =
                Arc::new(DerivedFamily::reduced_space(original.clone(), contract.clone()).unwrap());
            let composite = CompositeReconstruction::new(contract, workers).unwrap();
            let mut bound = family
                .bind_reduced(
                    LinearOriginal {
                        contract: original.clone(),
                        chain,
                    },
                    composite,
                    hash(40),
                )
                .unwrap();
            let unbound = bound.key();
            bound.prepare_reconstruction(&[3.0]).unwrap();
            assert_eq!(admissions[0].get(), Some(3.0));
            // The dependent child must bind at the actual reconstructed predecessor,
            // rather than a zero placeholder or the outer retained point.
            assert_eq!(admissions[1].get(), Some(if chain { 6.0 } else { 3.0 }));
            assert_ne!(bound.key(), unbound);
            let admitted = bound.key();
            let point_demand = AccuracyDemand {
                product: bound.point_product(&[3.0]).unwrap().key().unwrap(),
                normalization: original.normalization(),
                allowance: 1e-9,
                class: AccuracyClass::Certified,
            };
            let action_demand = AccuracyDemand {
                product: bound.action_product(&[3.0], &[1.0]).unwrap().key().unwrap(),
                ..point_demand
            };
            let limits = RefinementLimits {
                rounds: 4,
                proof_cells: 64,
            };
            let point = bound.reconstruct(&[3.0], &point_demand, limits).unwrap();
            assert_eq!(point.values, [3.0, 6.0, if chain { 12.0 } else { 6.0 }]);
            let (rows, action, accuracy) = bound
                .original_composition_product(&[3.0], &[1.0], &point_demand, &action_demand, limits)
                .unwrap();
            assert_eq!(rows, [0.0, 0.0]);
            assert_eq!(action.values, [1.0, 2.0, if chain { 4.0 } else { 2.0 }]);
            assert!(accuracy.point.satisfies(&point_demand));
            assert!(accuracy.action.unwrap().satisfies(&action_demand));
            assert_eq!(bound.key(), admitted);
        }
    }
    #[test]
    fn composite_disjoint_and_exact_chain_actions_consume_actual_accuracy() {
        for chain in [false, true] {
            let original = original();
            let workers = vec![
                supplier(&original, 0, 1, 0, 0.0),
                supplier(&original, usize::from(chain), 2, 1, 0.0),
            ];
            let contract = CompositeReconstruction::<Linear>::prepare_contract(
                original.clone(),
                vec![0.into()],
                &workers
                    .iter()
                    .map(|s| s.contract.clone())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            let mut composite = CompositeReconstruction::new(contract, workers).unwrap();
            let demand = AccuracyDemand {
                product: hash(10),
                normalization: original.normalization(),
                allowance: 1e-9,
                class: AccuracyClass::Certified,
            };
            let limits = RefinementLimits {
                rounds: 4,
                proof_cells: 64,
            };
            let point = composite.point(&[3.0], &demand, limits).unwrap();
            assert_eq!(point.values, [3.0, 6.0, if chain { 12.0 } else { 6.0 }]);
            let action = composite
                .jacobian_product(&[3.0], &[1.0], &demand, limits)
                .unwrap();
            assert_eq!(action.values, [1.0, 2.0, if chain { 4.0 } else { 2.0 }]);
            assert!(point.accuracy.satisfies(&demand));
            assert!(action.accuracy.satisfies(&demand));
        }
    }
    #[test]
    fn composite_uncertain_chain_refuses_exact_input_enclosure_transport() {
        let original = original();
        let workers = vec![
            supplier(&original, 0, 1, 0, 1e-10),
            supplier(&original, 1, 2, 1, 0.0),
        ];
        let contract = CompositeReconstruction::<Linear>::prepare_contract(
            original.clone(),
            vec![0.into()],
            &workers
                .iter()
                .map(|s| s.contract.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let mut composite = CompositeReconstruction::new(contract, workers).unwrap();
        let demand = AccuracyDemand {
            product: hash(10),
            normalization: original.normalization(),
            allowance: 1e-9,
            class: AccuracyClass::Certified,
        };
        assert!(matches!(
            composite.point(
                &[3.0],
                &demand,
                RefinementLimits {
                    rounds: 4,
                    proof_cells: 64
                }
            ),
            Err(MathError::Refinement {
                reason: crate::derived::RefinementRefusal::Unavailable(_),
                ..
            })
        ));
    }
    #[test]
    fn composite_uniform_supplier_consumes_and_amplifies_actual_predecessor_error() {
        let original = original();
        let workers = vec![
            UniformLinear(supplier(&original, 0, 1, 0, 1e-10)),
            UniformLinear(supplier(&original, 1, 2, 1, 0.0)),
        ];
        let contract = CompositeReconstruction::<UniformLinear>::prepare_contract(
            original.clone(),
            vec![0.into()],
            &workers
                .iter()
                .map(|s| s.0.contract.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let mut composite = CompositeReconstruction::new(contract, workers).unwrap();
        let demand = AccuracyDemand {
            product: hash(10),
            normalization: original.normalization(),
            allowance: 1e-9,
            class: AccuracyClass::Certified,
        };
        let r = RefinementLimits {
            rounds: 4,
            proof_cells: 64,
        };
        let point = composite.point(&[3.0], &demand, r).unwrap();
        assert_eq!(point.values, [3.0, 6.0, 12.0]);
        assert_eq!(point.accuracy.error, Some(2e-10));
        let action = composite
            .jacobian_product(&[3.0], &[1.0], &demand, r)
            .unwrap();
        assert_eq!(action.values, [1.0, 2.0, 4.0]);
        assert_eq!(action.accuracy.error, Some(2e-10));
    }
}
