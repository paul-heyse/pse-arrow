// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Oriented arclength equation families. Native libraries own correction;
//! these equations preserve original guards and scalar parameter actions.
use crate::{
    MathError,
    derived::{Constraint, Coordinate, OriginalContract},
    index::{Entry, GlobalCol, GlobalRow},
    normalization::Normalization,
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::DerivativeOrder;
use std::sync::Arc;

/// Explicit external parameter, with physical units supplied by its preparation owner.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    /// Semantic parameter coordinate.
    pub id: SemanticId,
    /// Closed physical lower limit.
    pub lower: f64,
    /// Closed physical upper limit.
    pub upper: f64,
    /// Positive physical coordinate nominal.
    pub scale: f64,
    /// Actual parameter program/provider identity.
    pub source: ContentHash,
    /// The supplier admits its first parameter action; Value does not imply this.
    pub action: bool,
    /// Complete original rows structurally dependent on the parameter.
    pub rows: Vec<GlobalRow>,
}
/// Mechanical augmented column correspondence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Column {
    /// Original state coordinate.
    State(GlobalCol),
    /// The external scalar parameter, appended after original states.
    Parameter,
}
/// Mechanical augmented row correspondence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// Original equation row.
    Original(GlobalRow),
    /// The dimensionless oriented hyperplane, appended after original rows.
    Hyperplane,
}
/// Immutable scalar arclength family; no numerical rank or connectedness claim.
#[derive(Clone, Debug)]
pub struct ArclengthFamily {
    original: Arc<OriginalContract>,
    parameter: Parameter,
    hyperplane: SemanticId,
    scales: Normalization,
    incidence: Vec<Entry<GlobalRow, GlobalCol>>,
    key: ContentHash,
}
impl ArclengthFamily {
    /// Prepare original equations plus one hyperplane. Structural parameter and
    /// hyperplane edges remain present even at numerical zero and Value support.
    /// # Errors
    /// Non-square/nonzero-equality originals, objective interpretation, duplicate
    /// coordinates, invalid parameter support/interval or normalization mismatch.
    pub fn new(
        original: Arc<OriginalContract>,
        mut parameter: Parameter,
        hyperplane: SemanticId,
        scales: Normalization,
    ) -> Result<Self, MathError> {
        let n = original.coordinates().len();
        scales.validate(n, n)?;
        if original.constraints().len() != n
            || original
                .constraints()
                .iter()
                .any(|r| r.lower != 0. || r.upper != 0.)
            || original.obligations().objective.is_some()
            || original.normalization() != scales.key()
            || original.coordinates().iter().any(|c| c.id == parameter.id)
            || original.constraints().iter().any(|r| r.id == hyperplane)
            || !parameter.lower.is_finite()
            || !parameter.upper.is_finite()
            || parameter.lower >= parameter.upper
            || !parameter.scale.is_finite()
            || parameter.scale <= 0.
            || parameter.rows.iter().any(|r| r.get() >= n)
        {
            return Err(MathError::Contract("arclength requires original square zero equations, distinct explicit maps, finite scalar interval and matching positive physical scales".into()));
        }
        parameter.rows.sort_unstable();
        parameter.rows.dedup();
        let mut incidence = original.incidence().to_vec();
        incidence.extend(
            parameter
                .rows
                .iter()
                .map(|r| Entry::new(*r, GlobalCol::new(n))),
        );
        incidence.extend((0..=n).map(|c| Entry::new(GlobalRow::new(n), GlobalCol::new(c))));
        incidence.sort_unstable();
        incidence.dedup();
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("scalar-oriented-arclength");
        original.frame(&mut h);
        h.id(&parameter.id)
            .f64(parameter.lower)
            .f64(parameter.upper)
            .f64(parameter.scale)
            .hash(&parameter.source)
            .bool(parameter.action)
            .id(&hyperplane)
            .hash(&scales.key())
            .u64(parameter.rows.len() as u64);
        for row in &parameter.rows {
            h.u64(row.get() as u64);
        }
        Ok(Self {
            original,
            parameter,
            hyperplane,
            scales,
            incidence,
            key: h.finish_hash(),
        })
    }
    /// Complete consumed family structure/source/normalization identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Original equations, bounds, guards and selected meaning.
    pub fn original(&self) -> &OriginalContract {
        &self.original
    }
    /// Explicit scalar coordinate and actual parameter derivative support.
    pub fn parameter(&self) -> &Parameter {
        &self.parameter
    }
    /// Positive physical state/row nominals; parameter nominal is separate.
    pub fn scales(&self) -> &Normalization {
        &self.scales
    }
    /// Full mechanical augmented structural edges, independent of numerical support.
    pub fn incidence(&self) -> &[Entry<GlobalRow, GlobalCol>] {
        &self.incidence
    }
    /// Explicit original state columns followed by the parameter column.
    pub fn column_map(&self) -> Vec<Column> {
        (0..self.original.coordinates().len())
            .map(|c| Column::State(GlobalCol::new(c)))
            .chain([Column::Parameter])
            .collect()
    }
    /// Explicit original rows followed by the hyperplane row.
    pub fn row_map(&self) -> Vec<Row> {
        (0..self.original.constraints().len())
            .map(|r| Row::Original(GlobalRow::new(r)))
            .chain([Row::Hyperplane])
            .collect()
    }
    /// Physical original state bounds followed by the declared parameter interval.
    pub fn coordinates(&self) -> Vec<Coordinate> {
        self.original
            .coordinates()
            .iter()
            .cloned()
            .chain([Coordinate {
                id: self.parameter.id,
                lower: self.parameter.lower,
                upper: self.parameter.upper,
            }])
            .collect()
    }
    /// Original physical rows plus a dimensionless zero hyperplane equation.
    pub fn constraints(&self) -> Vec<Constraint> {
        self.original
            .constraints()
            .iter()
            .cloned()
            .chain([Constraint {
                id: self.hyperplane,
                lower: 0.,
                upper: 0.,
            }])
            .collect()
    }
    /// State and parameter coordinate nominals in augmented column order.
    pub fn coordinate_scales(&self) -> Vec<f64> {
        self.scales
            .variables
            .iter()
            .copied()
            .chain([self.parameter.scale])
            .collect()
    }
    /// Original physical row nominals plus the dimensionless hyperplane nominal.
    pub fn row_scales(&self) -> Vec<f64> {
        self.scales.rows.iter().copied().chain([1.]).collect()
    }
    /// Whether both actual state and scalar parameter First actions are admitted.
    pub fn action_available(&self) -> bool {
        self.original.support().order >= DerivativeOrder::First
            && self.original.support().jacobian_product
            && self.parameter.action
    }
    /// Bind a physical predictor and its explicitly oriented unit normalized tangent.
    /// The source binds the accepted path/sheet, transport, point and tangent evidence.
    /// # Errors
    /// Supplier mismatch, unavailable First actions, invalid/exterior predictor or
    /// tangent shape, norm or source correspondence.
    pub fn bind<O: ParameterizedOracle>(
        self: &Arc<Self>,
        oracle: O,
        predictor: Vec<f64>,
        tangent: Vec<f64>,
        source: ContentHash,
    ) -> Result<BoundArclength<O>, MathError> {
        if oracle.contract() != self.original.as_ref()
            || oracle.parameter() != &self.parameter
            || !self.action_available()
        {
            return Err(MathError::Contract(
                "arclength supplier/support differs from its admitted family".into(),
            ));
        }
        self.validate_point(&predictor)?;
        if tangent.len() != predictor.len()
            || tangent.iter().any(|v| !v.is_finite())
            || (tangent.iter().map(|v| v * v).sum::<f64>() - 1.).abs()
                > 256. * predictor.len() as f64 * f64::EPSILON
        {
            return Err(MathError::Contract(
                "arclength hyperplane needs a unit oriented normalized tangent".into(),
            ));
        }
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("oriented-arclength-hyperplane")
            .hash(&self.key)
            .hash(&source)
            .u64(predictor.len() as u64);
        for v in predictor.iter().chain(&tangent) {
            h.f64(*v);
        }
        Ok(BoundArclength {
            family: self.clone(),
            oracle,
            predictor,
            tangent,
            source,
            key: h.finish_hash(),
        })
    }
    /// Check the declared physical coordinate intervals; the supplier still owns guards.
    pub fn validate_point(&self, point: &[f64]) -> Result<(), MathError> {
        if point.len() != self.original.coordinates().len() + 1
            || point.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "arclength physical point shape or nonfinite values".into(),
            ));
        }
        for (v, c) in point.iter().zip(self.coordinates()) {
            if *v < c.lower || *v > c.upper {
                return Err(MathError::OutsideRange {
                    source_id: c.id,
                    target: c.id,
                    value: *v,
                    lower: Some(c.lower),
                    upper: Some(c.upper),
                });
            }
        }
        Ok(())
    }
}
/// Actual parameterized original equations. These operations retain supplier errors,
/// including recoverable domains and terminal scope/provider failures.
pub trait ParameterizedOracle: std::fmt::Debug {
    /// Original owning error universe.
    type Error: From<MathError>;
    /// Complete original state/equation inventory.
    fn contract(&self) -> &OriginalContract;
    /// Exact parameter declaration consumed by the supplier.
    fn parameter(&self) -> &Parameter;
    /// Original guarded physical equation values at this scalar parameter.
    fn values(&mut self, x: &[f64], parameter: f64, out: &mut [f64]) -> Result<(), Self::Error>;
    /// Actual admitted physical state partial action at fixed parameter.
    fn state_action(
        &mut self,
        x: &[f64],
        parameter: f64,
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), Self::Error>;
    /// Actual admitted physical parameter partial action at fixed state.
    fn parameter_action(
        &mut self,
        x: &[f64],
        parameter: f64,
        direction: f64,
        out: &mut [f64],
    ) -> Result<(), Self::Error>;
}
/// Realized hyperplane family; evaluation does not choose steps or solve equations.
#[derive(Debug)]
pub struct BoundArclength<O> {
    family: Arc<ArclengthFamily>,
    oracle: O,
    predictor: Vec<f64>,
    tangent: Vec<f64>,
    source: ContentHash,
    key: ContentHash,
}
impl<O: ParameterizedOracle> BoundArclength<O> {
    /// Actual family with its original reconstruction and guard obligations.
    pub fn family(&self) -> &Arc<ArclengthFamily> {
        &self.family
    }
    /// Consumed predictor/orientation source identity.
    pub fn source(&self) -> ContentHash {
        self.source
    }
    /// Full bound equation identity; rebinding a predictor changes this key.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Original physical predictor, with parameter last.
    pub fn predictor(&self) -> &[f64] {
        &self.predictor
    }
    /// Unit tangent in normalized augmented coordinates.
    pub fn tangent(&self) -> &[f64] {
        &self.tangent
    }
    /// Return the exact supplier; terminal original assessment remains its owner.
    pub fn into_oracle(self) -> O {
        self.oracle
    }
    /// Original physical equations plus the dimensionless hyperplane equation.
    /// # Errors
    /// Original guards/provider failures, invalid physical coordinates or output shape.
    pub fn residual(&mut self, point: &[f64], out: &mut [f64]) -> Result<(), O::Error> {
        self.family.validate_point(point)?;
        let n = point.len() - 1;
        if out.len() != point.len() {
            return Err(MathError::Contract("arclength residual shape".into()).into());
        }
        let mut result = vec![0.; point.len()];
        self.oracle
            .values(&point[..n], point[n], &mut result[..n])?;
        result[n] = point
            .iter()
            .zip(&self.predictor)
            .zip(&self.tangent)
            .zip(self.family.coordinate_scales())
            .map(|(((x, p), t), s)| t * (x - p) / s)
            .sum();
        finite(&result)?;
        out.copy_from_slice(&result);
        Ok(())
    }
    /// Apply the actual physical derivative; outputs publish only after both supplier
    /// actions and the border succeed. No derivative comes from Value support.
    /// # Errors
    /// Original action failures, invalid coordinates/direction or shape.
    pub fn action(
        &mut self,
        point: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), O::Error> {
        self.family.validate_point(point)?;
        if direction.len() != point.len() || out.len() != point.len() {
            return Err(MathError::Contract("arclength action shape".into()).into());
        }
        finite(direction)?;
        let n = point.len() - 1;
        let mut result = vec![0.; point.len()];
        let mut partial = vec![0.; n];
        self.oracle
            .state_action(&point[..n], point[n], &direction[..n], &mut result[..n])?;
        self.oracle
            .parameter_action(&point[..n], point[n], direction[n], &mut partial)?;
        for (v, p) in result[..n].iter_mut().zip(partial) {
            *v += p;
        }
        result[n] = direction
            .iter()
            .zip(&self.tangent)
            .zip(self.family.coordinate_scales())
            .map(|((v, t), s)| t * v / s)
            .sum();
        finite(&result)?;
        out.copy_from_slice(&result);
        Ok(())
    }
    /// Project back into original physical states and the external parameter. This is
    /// equation correspondence, never original feasibility/stationarity permission.
    pub fn reconstruct(&self, point: &[f64]) -> Result<(Vec<f64>, f64), MathError> {
        self.family.validate_point(point)?;
        let n = point.len() - 1;
        Ok((point[..n].to_vec(), point[n]))
    }
}
fn finite(values: &[f64]) -> Result<(), MathError> {
    if values.iter().any(|v| !v.is_finite()) {
        Err(MathError::Contract(
            "nonfinite arclength supplier/action".into(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derived::{DerivativeSupport, OriginalObligations};
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    #[derive(Debug)]
    struct Quadratic {
        original: Arc<OriginalContract>,
        parameter: Parameter,
        fail_parameter: bool,
    }
    impl ParameterizedOracle for Quadratic {
        type Error = MathError;
        fn contract(&self) -> &OriginalContract {
            &self.original
        }
        fn parameter(&self) -> &Parameter {
            &self.parameter
        }
        fn values(&mut self, x: &[f64], p: f64, out: &mut [f64]) -> Result<(), MathError> {
            out[0] = x[0] * x[0] - p;
            Ok(())
        }
        fn state_action(
            &mut self,
            x: &[f64],
            _: f64,
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), MathError> {
            out[0] = 2. * x[0] * v[0];
            Ok(())
        }
        fn parameter_action(
            &mut self,
            _: &[f64],
            _: f64,
            v: f64,
            out: &mut [f64],
        ) -> Result<(), MathError> {
            if self.fail_parameter {
                return Err(MathError::Domain {
                    source_id: id(9),
                    requirement: "scripted original parameter domain",
                });
            }
            out[0] = -v;
            Ok(())
        }
    }
    fn family(action: bool) -> Arc<ArclengthFamily> {
        let scales = Normalization {
            variables: vec![2.],
            rows: vec![5.],
            objective: 1.,
        };
        let original = Arc::new(
            OriginalContract::new(
                hash(1),
                scales.key(),
                vec![Coordinate {
                    id: id(1),
                    lower: -10.,
                    upper: 10.,
                }],
                vec![Constraint {
                    id: id(2),
                    lower: 0.,
                    upper: 0.,
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
                    objective: None,
                },
            )
            .unwrap(),
        );
        let parameter = Parameter {
            id: id(6),
            lower: -10.,
            upper: 10.,
            scale: 3.,
            source: hash(7),
            action,
            rows: vec![GlobalRow::new(0)],
        };
        Arc::new(ArclengthFamily::new(original, parameter, id(8), scales).unwrap())
    }
    fn oracle(f: &ArclengthFamily, fail_parameter: bool) -> Quadratic {
        Quadratic {
            original: f.original.clone(),
            parameter: f.parameter.clone(),
            fail_parameter,
        }
    }
    #[test]
    fn physical_action_hyperplane_and_mechanical_incidence_preserve_original_equations() {
        let f = family(true);
        let t = std::f64::consts::FRAC_1_SQRT_2;
        assert_eq!(
            f.column_map(),
            [Column::State(GlobalCol::new(0)), Column::Parameter]
        );
        assert_eq!(
            f.row_map(),
            [Row::Original(GlobalRow::new(0)), Row::Hyperplane]
        );
        assert_eq!(f.incidence().len(), 4);
        assert_eq!(f.constraints()[0], f.original.constraints()[0]);
        let mut bound = f
            .bind(oracle(&f, false), vec![1., 1.], vec![t, t], hash(10))
            .unwrap();
        let mut out = [9.; 2];
        bound.residual(&[2., 2.], &mut out).unwrap();
        assert_eq!(out[0], 2.);
        assert!((out[1] - t * (0.5 + 1. / 3.)).abs() < 1e-14);
        bound.action(&[2., 2.], &[4., 6.], &mut out).unwrap();
        assert_eq!(out[0], 10.);
        assert!((out[1] - 4. * t).abs() < 1e-14);
        assert_eq!(bound.reconstruct(&[2., 2.]).unwrap(), (vec![2.], 2.));
        let key = bound.key();
        let changed = f
            .bind(oracle(&f, false), vec![1., 1.], vec![t, t], hash(11))
            .unwrap();
        assert_ne!(key, changed.key());
        assert_eq!(changed.family().key(), f.key());
    }
    #[test]
    fn value_structure_cannot_invent_parameter_action_and_failure_is_transactional() {
        let f = family(false);
        assert_eq!(f.incidence().len(), 4);
        assert!(!f.action_available());
        assert!(matches!(
            f.bind(oracle(&f, false), vec![1., 1.], vec![1., 0.], hash(10)),
            Err(MathError::Contract(_))
        ));
        let f = family(true);
        let mut bound = f
            .bind(oracle(&f, true), vec![1., 1.], vec![1., 0.], hash(10))
            .unwrap();
        let mut out = [17.; 2];
        assert!(
            matches!(bound.action(&[1.,1.],&[1.,1.],&mut out),Err(MathError::Domain {source_id,..}) if source_id==id(9))
        );
        assert_eq!(out, [17.; 2]);
        assert!(
            matches!(bound.residual(&[1.,11.],&mut out),Err(MathError::OutsideRange {target,..}) if target==id(6))
        );
        assert_eq!(out, [17.; 2]);
    }
    #[test]
    fn parameter_source_interval_scale_and_oriented_binding_are_consumed() {
        let f = family(true);
        let mut parameter = f.parameter.clone();
        parameter.source = hash(17);
        let changed =
            ArclengthFamily::new(f.original.clone(), parameter, id(8), f.scales.clone()).unwrap();
        assert_ne!(changed.key(), f.key());
        assert!(
            f.bind(oracle(&f, false), vec![1., 1.], vec![2., 0.], hash(10))
                .is_err()
        );
        let forward = f
            .bind(oracle(&f, false), vec![1., 1.], vec![1., 0.], hash(10))
            .unwrap();
        let reverse = f
            .bind(oracle(&f, false), vec![1., 1.], vec![-1., 0.], hash(10))
            .unwrap();
        assert_ne!(forward.key(), reverse.key());
    }
}
