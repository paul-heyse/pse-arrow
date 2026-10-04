// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit auxiliary distance objective over one immutable original algebraic inventory.
//! The analytic diagonal quadratic is admitted as such; no solver or numerical
//! differentiation lives here, and its optimum grants no original permission.
use crate::{
    MathError,
    derived::{Correspondence, DerivativeSupport, OriginalContract},
    index::GlobalCol,
};
use pse_ids::{ContentHash, FramedHasher};
use pse_kernels::DerivativeOrder;
use std::{collections::BTreeSet, sync::Arc};

/// Prepared physical weighted distance, expressed in the original coordinate units.
#[derive(Clone, Debug)]
pub struct LeastDeviation {
    original: Arc<OriginalContract>,
    center: Vec<f64>,
    scales: Vec<f64>,
    weights: Vec<f64>,
    free: Vec<GlobalCol>,
    held: Vec<GlobalCol>,
    curvature: Vec<f64>,
    metric_source: ContentHash,
    key: ContentHash,
}
impl LeastDeviation {
    /// Admit `0.5 * sum_free weight[i] * ((x[i]-center[i])/scale[i])^2`.
    /// Scales are positive physical characteristic extents in each original
    /// coordinate's canonical unit; weights are positive dimensionless metric entries.
    /// The center is a reference, never a valid-original-point certificate.
    /// # Errors
    /// Invalid metric/extents, incomplete or overlapping role maps, a held center
    /// outside an authored bound, or unrepresentable positive diagonal curvature.
    pub fn new(
        original: Arc<OriginalContract>,
        center: Vec<f64>,
        scales: Vec<f64>,
        weights: Vec<f64>,
        free: Vec<GlobalCol>,
        held: Vec<GlobalCol>,
        metric_source: ContentHash,
    ) -> Result<Self, MathError> {
        let n = original.coordinates().len();
        if center.len() != n
            || scales.len() != n
            || weights.len() != n
            || center.iter().any(|x| !x.is_finite())
            || scales
                .iter()
                .chain(&weights)
                .any(|x| !x.is_finite() || *x <= 0.0)
            || free.len() + held.len() != n
            || free.iter().chain(&held).any(|c| c.get() >= n)
            || free
                .iter()
                .chain(&held)
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                != n
        {
            return Err(MathError::Contract(
                "invalid least-deviation center, physical metric or held/free coordinate partition"
                    .into(),
            ));
        }
        let mut curvature = vec![0.0; n];
        for column in &free {
            let i = column.get();
            let value = weights[i] / scales[i] / scales[i];
            if !value.is_finite() || value <= 0.0 {
                return Err(MathError::CoefficientRange);
            }
            curvature[i] = value;
        }
        for column in &held {
            let i = column.get();
            let coordinate = &original.coordinates()[i];
            if center[i] < coordinate.lower || center[i] > coordinate.upper {
                return Err(MathError::Contract(
                    "held initialization coordinate violates its original interval".into(),
                ));
            }
        }
        let mut hash = FramedHasher::new(pse_ids::Frame::InitializationDeviationV1);
        original.frame(&mut hash);
        hash.hash(&metric_source)
            .str("analytic-positive-diagonal-quadratic");
        for ((c, s), w) in center.iter().zip(&scales).zip(&weights) {
            hash.f64(*c).f64(*s).f64(*w);
        }
        hash.u64(free.len() as u64);
        for column in &free {
            hash.u64(column.get() as u64);
        }
        hash.u64(held.len() as u64);
        for column in &held {
            hash.u64(column.get() as u64);
        }
        Ok(Self {
            original,
            center,
            scales,
            weights,
            free,
            held,
            curvature,
            metric_source,
            key: hash.finish_hash(),
        })
    }
    /// Complete immutable authored bounds, rows, guards, selection and objective meaning.
    pub fn original(&self) -> &Arc<OriginalContract> {
        &self.original
    }
    /// Objective/metric/role identity, distinct from original scientific identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Declared physical metric producer, including its unit/normalization admission.
    pub fn metric_source(&self) -> ContentHash {
        self.metric_source
    }
    /// Finite reference in original coordinate order.
    pub fn center(&self) -> &[f64] {
        &self.center
    }
    /// Declared physical characteristic extents in original coordinate order.
    pub fn scales(&self) -> &[f64] {
        &self.scales
    }
    /// Declared dimensionless positive weights in original coordinate order.
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }
    /// Original coordinates allowed to change.
    pub fn free(&self) -> &[GlobalCol] {
        &self.free
    }
    /// Original coordinates fixed to their reference values.
    pub fn held(&self) -> &[GlobalCol] {
        &self.held
    }
    /// Exact analytic objective curvature; held columns contribute zero on this slice.
    pub fn curvature(&self) -> &[f64] {
        &self.curvature
    }
    /// Owned product storage; the shared original inventory is charged by its owner.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + (self.center.capacity()
                + self.scales.capacity()
                + self.weights.capacity()
                + self.curvature.capacity())
                * size_of::<f64>()
            + (self.free.capacity() + self.held.capacity()) * size_of::<GlobalCol>()
    }
    /// Actual analytic objective derivative source. This does not promote constraints.
    pub fn objective_support(&self) -> DerivativeSupport {
        DerivativeSupport {
            order: DerivativeOrder::Second,
            jacobian_product: true,
            source: self.key,
        }
    }
    /// Original rows/bounds retain identity; the auxiliary objective is not the authored objective.
    pub fn correspondence(&self) -> Correspondence {
        Correspondence::ConstraintIdentity
    }
    /// Intersect original intervals with the explicitly held point, without dropping any interval.
    pub fn coordinate_bounds(&self) -> Vec<(f64, f64)> {
        let mut bounds = self
            .original
            .coordinates()
            .iter()
            .map(|c| (c.lower, c.upper))
            .collect::<Vec<_>>();
        for column in &self.held {
            let i = column.get();
            bounds[i] = (self.center[i], self.center[i]);
        }
        bounds
    }
    /// Check an actual original-coordinate point, including the immutable held roles.
    pub fn validate_point(&self, x: &[f64]) -> Result<(), MathError> {
        if x.len() != self.center.len() || x.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Contract(
                "least-deviation original point extent or nonfinite value".into(),
            ));
        }
        for (coordinate, value) in self.original.coordinates().iter().zip(x) {
            if *value < coordinate.lower || *value > coordinate.upper {
                return Err(MathError::OutsideRange {
                    source_id: coordinate.id,
                    target: coordinate.id,
                    value: *value,
                    lower: coordinate.lower.is_finite().then_some(coordinate.lower),
                    upper: coordinate.upper.is_finite().then_some(coordinate.upper),
                });
            }
        }
        if self.held.iter().any(|c| x[c.get()] != self.center[c.get()]) {
            return Err(MathError::Contract(
                "least-deviation trial changed an explicitly held coordinate".into(),
            ));
        }
        Ok(())
    }
    /// Evaluate the admitted quadratic through faer's real vector norm reduction.
    pub fn objective(&self, x: &[f64]) -> Result<f64, MathError> {
        self.validate_point(x)?;
        let values = self
            .free
            .iter()
            .map(|c| {
                let i = c.get();
                (x[i] - self.center[i]) * self.curvature[i].sqrt()
            })
            .collect::<Vec<_>>();
        if values.iter().any(|v| !v.is_finite()) {
            return Err(MathError::CoefficientRange);
        }
        let value = 0.5 * faer::col::ColRef::from_slice(&values).squared_norm_l2();
        if !value.is_finite() {
            return Err(MathError::CoefficientRange);
        }
        Ok(value)
    }
    /// Exact analytic first derivative, publishing only after every entry is finite.
    pub fn gradient(&self, x: &[f64], out: &mut [f64]) -> Result<(), MathError> {
        self.validate_point(x)?;
        if out.len() != self.center.len() {
            return Err(MathError::Contract(
                "least-deviation gradient extent".into(),
            ));
        }
        let values = x
            .iter()
            .zip(&self.center)
            .zip(&self.curvature)
            .map(|((x, c), d)| (x - c) * d)
            .collect::<Vec<_>>();
        if values.iter().any(|v| !v.is_finite()) {
            return Err(MathError::CoefficientRange);
        }
        out.copy_from_slice(&values);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        derived::{Constraint, Coordinate, OriginalObligations},
        index::{Entry, GlobalRow},
    };
    use pse_ids::SemanticId;
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    fn original() -> Arc<OriginalContract> {
        Arc::new(
            OriginalContract::new(
                hash(1),
                hash(2),
                vec![
                    Coordinate {
                        id: SemanticId::from_bytes([1; 16]),
                        lower: -2.0,
                        upper: 5.0,
                    },
                    Coordinate {
                        id: SemanticId::from_bytes([2; 16]),
                        lower: 0.0,
                        upper: 10.0,
                    },
                ],
                vec![Constraint {
                    id: SemanticId::from_bytes([3; 16]),
                    lower: -1.0,
                    upper: 2.0,
                }],
                vec![Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
                DerivativeSupport {
                    order: DerivativeOrder::First,
                    jacobian_product: true,
                    source: hash(3),
                },
                OriginalObligations {
                    guards: hash(4),
                    selection: hash(5),
                    objective: Some(hash(6)),
                },
            )
            .unwrap(),
        )
    }
    #[test]
    fn quadratic_derivatives_keep_metric_and_held_original_inventory() {
        let original = original();
        let family = LeastDeviation::new(
            original.clone(),
            vec![1.0, 3.0],
            vec![2.0, 4.0],
            vec![8.0, 2.0],
            vec![GlobalCol::new(0)],
            vec![GlobalCol::new(1)],
            hash(7),
        )
        .unwrap();
        assert_eq!(family.original().as_ref(), original.as_ref());
        assert_eq!(family.coordinate_bounds(), [(-2.0, 5.0), (3.0, 3.0)]);
        assert_eq!(family.objective_support().order, DerivativeOrder::Second);
        assert_eq!(family.original().support().order, DerivativeOrder::First);
        assert!((family.objective(&[2.0, 3.0]).unwrap() - 1.0).abs() < 1e-14);
        let mut gradient = [-99.0; 2];
        family.gradient(&[2.0, 3.0], &mut gradient).unwrap();
        assert_eq!(gradient, [2.0, 0.0]);
        assert_eq!(family.curvature(), [2.0, 0.0]);
        assert!(matches!(
            family.objective(&[2.0, 3.1]),
            Err(MathError::Contract(_))
        ));
        assert!(matches!(
            family.objective(&[5.1, 3.0]),
            Err(MathError::OutsideRange { .. })
        ));
        let changed = LeastDeviation::new(
            original,
            vec![1.0, 3.0],
            vec![2.0, 4.0],
            vec![8.0, 2.0],
            vec![GlobalCol::new(0)],
            vec![GlobalCol::new(1)],
            hash(8),
        )
        .unwrap();
        assert_ne!(family.key(), changed.key());
    }
    #[test]
    fn invalid_partition_metric_and_held_center_refuse_preparation() {
        let make = |center, scales, free, held| {
            LeastDeviation::new(
                original(),
                center,
                scales,
                vec![1.0; 2],
                free,
                held,
                hash(7),
            )
        };
        let held = make(
            vec![1.0, 3.0],
            vec![1.0; 2],
            vec![],
            vec![GlobalCol::new(0), GlobalCol::new(1)],
        )
        .unwrap();
        assert_eq!(held.objective(&[1.0, 3.0]).unwrap(), 0.0);
        assert_eq!(held.coordinate_bounds(), [(1.0, 1.0), (3.0, 3.0)]);
        assert!(
            make(
                vec![1.0, 11.0],
                vec![1.0; 2],
                vec![GlobalCol::new(0)],
                vec![GlobalCol::new(1)]
            )
            .is_err()
        );
        assert!(
            make(
                vec![1.0, 3.0],
                vec![0.0, 1.0],
                vec![GlobalCol::new(0)],
                vec![GlobalCol::new(1)]
            )
            .is_err()
        );
        assert!(
            make(
                vec![1.0, 3.0],
                vec![1.0; 2],
                vec![GlobalCol::new(0)],
                vec![GlobalCol::new(0)]
            )
            .is_err()
        );
        assert!(
            make(
                vec![1.0, 3.0],
                vec![1e-300, 1.0],
                vec![GlobalCol::new(0)],
                vec![GlobalCol::new(1)]
            )
            .is_err()
        );
    }
}
