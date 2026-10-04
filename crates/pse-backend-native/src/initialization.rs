// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native NLP lowering of an explicitly prepared auxiliary initialization objective.
//! Original suppliers own constraints/guards/selection; native libraries own solves.
use crate::{
    DerivativeFacts, NlpOracle, OracleContract, ProblemError, Variable, derived::NlpBridge,
};
use pse_kernels::{DerivativeOrder, ExecutionScope};
use pse_math::{
    index::{Addend, Entry, GlobalCol, GlobalRow},
    initialization::LeastDeviation,
    sparse::AssemblyMatrix,
};
use std::sync::Arc;

/// Preparation-owned correspondence between one frozen scientific contract and its
/// actual compiled/native supplier. Their identities need not be equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalNlpBinding {
    /// Full original scientific contract identity.
    pub original: pse_ids::ContentHash,
    /// Actual native supplier contract identity.
    pub source: pse_ids::ContentHash,
    /// Frozen preparation identity, including fixed-time values/parameters and source admission.
    pub preparation: pse_ids::ContentHash,
}

/// Complete original NLP rows with the admitted least-deviation objective and held roles.
#[derive(Debug)]
pub struct LeastDeviationOracle {
    family: Arc<LeastDeviation>,
    source: Box<dyn NlpOracle>,
    contract: OracleContract,
    scope: ExecutionScope,
    hessian: Option<AssemblyMatrix>,
    source_hessian_entries: usize,
    binding: OriginalNlpBinding,
    normalization: Option<pse_math::normalization::Normalization>,
}

impl LeastDeviationOracle {
    /// Validate the actual original source before any native solver acquires it.
    /// Exact Second is admitted only when original constraints actually supply it.
    /// # Errors
    /// Original/source inventory or support mismatch, invalid sparse shape, cancellation,
    /// or a pattern beyond the explicitly supplied finite native layout allowance.
    pub fn new(
        source: Box<dyn NlpOracle>,
        family: Arc<LeastDeviation>,
        binding: OriginalNlpBinding,
        scope: ExecutionScope,
        max_entries: usize,
    ) -> Result<Self, ProblemError> {
        scope.check().map_err(ProblemError::Provider)?;
        let original = family.original();
        if binding.original != original.identity()
            || binding.source != source.contract().identity
            || source
                .normalization()
                .is_some_and(|n| n.key() != original.normalization())
        {
            return Err(ProblemError::Contract(
                "least-deviation frozen original/source binding or normalization mismatch".into(),
            ));
        }
        if original.support().order < DerivativeOrder::First {
            return Err(ProblemError::Unsupported(
                "least-deviation constraints require actual First support".into(),
            ));
        }
        let normalization = source.normalization().cloned().map(|mut n| {
            n.objective = 1.0;
            n
        });
        let source = NlpBridge::new(source, original.clone())?.into_original();
        crate::validate_nlp(source.as_ref(), DerivativeOrder::First)?;
        let n = original.coordinates().len();
        let m = original.constraints().len();
        if n > max_entries
            || m > max_entries
            || source.jacobian_pattern().compute_nnz() > max_entries
        {
            return Err(ProblemError::memory(
                "least-deviation native sparse layout allowance",
            ));
        }
        let mut edges = Vec::new();
        let jac = source.jacobian_pattern();
        for col in 0..n {
            for row in jac.row_idx_of_col(col) {
                edges.push(Entry::new(GlobalRow::new(row), GlobalCol::new(col)));
            }
        }
        edges.sort_unstable();
        if edges != original.incidence() {
            return Err(ProblemError::Contract(
                "least-deviation actual Jacobian differs from original all-branch incidence".into(),
            ));
        }
        let exact = original.support().order >= DerivativeOrder::Second
            && source.contract().derivatives >= DerivativeOrder::Second
            && source.contract().smoothness >= DerivativeOrder::Second
            && source.hessian_pattern().is_some();
        let (hessian, source_hessian_entries) = if exact {
            crate::validate_nlp(source.as_ref(), DerivativeOrder::Second)?;
            let pattern = source.hessian_pattern().ok_or_else(|| {
                ProblemError::Contract("least-deviation source Hessian vanished".into())
            })?;
            let mut entries = Vec::new();
            for col in 0..n {
                for row in pattern.row_idx_of_col(col) {
                    if row < col {
                        return Err(ProblemError::Contract(
                            "least-deviation source Hessian must use the canonical lower triangle"
                                .into(),
                        ));
                    }
                    entries.push(Entry::new(GlobalRow::new(row), GlobalCol::new(col)));
                }
            }
            let count = entries.len();
            for col in family.free() {
                entries.push(Entry::new(GlobalRow::new(col.get()), *col));
            }
            (
                Some(AssemblyMatrix::new(n, n, &entries, max_entries)?),
                count,
            )
        } else {
            (None, 0)
        };
        let bounds = family.coordinate_bounds();
        let contract = OracleContract {
            identity: Self::contract_identity(&family, binding),
            variables: original
                .coordinates()
                .iter()
                .zip(bounds)
                .map(|(c, (lower, upper))| Variable {
                    id: c.id,
                    lower,
                    upper,
                })
                .collect(),
            rows: original.constraints().iter().map(|r| r.id).collect(),
            derivatives: if exact {
                DerivativeOrder::Second
            } else {
                DerivativeOrder::First
            },
            smoothness: source.contract().smoothness.min(if exact {
                DerivativeOrder::Second
            } else {
                DerivativeOrder::First
            }),
        };
        scope.check().map_err(ProblemError::Provider)?;
        Ok(Self {
            family,
            source,
            contract,
            scope,
            hessian,
            source_hessian_entries,
            binding,
            normalization,
        })
    }
    /// Identity of the actual auxiliary oracle consumed by structural preparation.
    /// This frames its mathematical family and frozen source binding once.
    pub fn contract_identity(
        family: &LeastDeviation,
        binding: OriginalNlpBinding,
    ) -> pse_ids::ContentHash {
        let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        identity
            .str("initialization-deviation-original-source")
            .hash(&family.key())
            .hash(&binding.source)
            .hash(&binding.preparation);
        identity.finish_hash()
    }
    /// Mathematical auxiliary product; original scientific identity/obligations remain inside it.
    pub fn family(&self) -> &Arc<LeastDeviation> {
        &self.family
    }
    /// Explicit frozen correspondence supplied by the original preparation owner.
    pub fn source_binding(&self) -> OriginalNlpBinding {
        self.binding
    }
    /// Additional adapter storage; original supplier and shared mathematical product
    /// remain charged by their preparation owners.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.contract.variables.capacity() * size_of::<Variable>()
            + self.contract.rows.capacity() * size_of::<pse_ids::SemanticId>()
            + self
                .hessian
                .as_ref()
                .map_or(0, AssemblyMatrix::retained_bytes)
            + self.normalization.as_ref().map_or(0, |n| {
                (n.variables.capacity() + n.rows.capacity()) * size_of::<f64>()
            })
    }
    /// Recover the unchanged original supplier for independent original assessment/correction.
    pub fn into_original(self) -> Box<dyn NlpOracle> {
        self.source
    }
    fn checkpoint(&self, x: &[f64]) -> Result<(), ProblemError> {
        self.scope.check().map_err(ProblemError::Provider)?;
        self.family.validate_point(x)?;
        Ok(())
    }
    fn original_values(&mut self, x: &[f64]) -> Result<Vec<f64>, ProblemError> {
        self.checkpoint(x)?;
        let mut values = vec![0.0; self.family.original().constraints().len()];
        self.source.constraints(x, &mut values)?;
        self.scope.check().map_err(ProblemError::Provider)?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite original initialization constraints",
            ));
        }
        Ok(values)
    }
}
impl NlpOracle for LeastDeviationOracle {
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        self.source.structural_analysis()
    }
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        self.normalization.as_ref()
    }
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        self.source.constraint_sources()
    }
    // Original objective presolve facts do not prove the replacement objective.
    fn derivative_facts(&self) -> DerivativeFacts {
        let facts = self.source.derivative_facts();
        DerivativeFacts {
            gradient_constant: false,
            jacobian_constant: facts.jacobian_constant,
            hessian_constant: self.hessian.is_some() && facts.jacobian_constant,
        }
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.source.jacobian_pattern()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        self.hessian.as_ref().map(|h| h.matrix().symbolic())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        self.source.constraint_bounds()
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.original_values(x)?;
        let value = self.family.objective(x)?;
        self.scope.check().map_err(ProblemError::Provider)?;
        Ok(value)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if out.len() != self.contract.rows.len() {
            return Err(ProblemError::Contract(
                "least-deviation constraint output extent".into(),
            ));
        }
        let values = self.original_values(x)?;
        out.copy_from_slice(&values);
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if out.len() != self.contract.variables.len() {
            return Err(ProblemError::Contract(
                "least-deviation gradient output extent".into(),
            ));
        }
        self.original_values(x)?;
        let mut values = vec![0.0; out.len()];
        self.family.gradient(x, &mut values)?;
        self.scope.check().map_err(ProblemError::Provider)?;
        out.copy_from_slice(&values);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if out.len() != self.source.jacobian_pattern().compute_nnz() {
            return Err(ProblemError::Contract(
                "least-deviation Jacobian output extent".into(),
            ));
        }
        self.original_values(x)?;
        let mut values = vec![0.0; out.len()];
        self.source.jacobian(x, &mut values)?;
        self.scope.check().map_err(ProblemError::Provider)?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite original initialization Jacobian",
            ));
        }
        out.copy_from_slice(&values);
        Ok(())
    }
    fn hessian(
        &mut self,
        x: &[f64],
        weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let Some(pattern) = &self.hessian else {
            return Err(ProblemError::Unsupported(
                "least-deviation source constraints provide no exact Second profile".into(),
            ));
        };
        if !weight.is_finite()
            || multipliers.len() != self.contract.rows.len()
            || multipliers.iter().any(|v| !v.is_finite())
            || out.len() != pattern.matrix().val().len()
        {
            return Err(ProblemError::Contract(
                "least-deviation Lagrangian Hessian weight or extent".into(),
            ));
        }
        self.original_values(x)?;
        let mut values = vec![0.0; self.source_hessian_entries];
        // The authored objective is retained for original assessment but contributes
        // no curvature to this explicitly auxiliary weighted-distance objective.
        self.source.hessian(x, 0.0, multipliers, &mut values)?;
        self.scope.check().map_err(ProblemError::Provider)?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite original initialization Hessian",
            ));
        }
        let mut matrix = self
            .hessian
            .as_ref()
            .ok_or_else(|| {
                ProblemError::Contract("least-deviation Hessian profile changed".into())
            })?
            .clone();
        matrix.clear();
        for (i, value) in values.into_iter().enumerate() {
            matrix.add(Addend::new(i), value)?;
        }
        for (i, column) in self.family.free().iter().enumerate() {
            let value = weight * self.family.curvature()[column.get()];
            if !value.is_finite() {
                return Err(ProblemError::numerical(
                    "least-deviation weighted objective curvature overflow",
                ));
            }
            matrix.add(Addend::new(self.source_hessian_entries + i), value)?;
        }
        self.scope.check().map_err(ProblemError::Provider)?;
        if matrix.matrix().val().iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "least-deviation Lagrangian curvature sum overflow",
            ));
        }
        out.copy_from_slice(matrix.matrix().val());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::{ContentHash, SemanticId};
    use pse_kernels::ProviderError;
    use pse_math::{
        MathError,
        derived::{
            Constraint, Coordinate, DerivativeSupport, OriginalContract, OriginalObligations,
        },
    };
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        time::{Duration, Instant},
    };
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn original(order: DerivativeOrder) -> Arc<OriginalContract> {
        Arc::new(
            OriginalContract::new(
                hash(1),
                hash(2),
                vec![
                    Coordinate {
                        id: id(1),
                        lower: -2.0,
                        upper: 5.0,
                    },
                    Coordinate {
                        id: id(2),
                        lower: 0.0,
                        upper: 10.0,
                    },
                ],
                vec![Constraint {
                    id: id(3),
                    lower: 0.0,
                    upper: 100.0,
                }],
                vec![
                    Entry::new(GlobalRow::new(0), GlobalCol::new(0)),
                    Entry::new(GlobalRow::new(0), GlobalCol::new(1)),
                ],
                DerivativeSupport {
                    order,
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
    fn family(order: DerivativeOrder) -> Arc<LeastDeviation> {
        Arc::new(
            LeastDeviation::new(
                original(order),
                vec![1.0, 1.0],
                vec![2.0, 4.0],
                vec![8.0, 2.0],
                vec![GlobalCol::new(0)],
                vec![GlobalCol::new(1)],
                hash(7),
            )
            .unwrap(),
        )
    }
    #[derive(Debug)]
    struct Source {
        contract: OracleContract,
        jac: AssemblyMatrix,
        hess: Option<AssemblyMatrix>,
        guard_failure: bool,
        cancel: Option<Arc<AtomicBool>>,
        delay: bool,
        normalization: Option<pse_math::normalization::Normalization>,
    }
    impl Source {
        fn new(order: DerivativeOrder) -> Self {
            let original = original(order);
            Self {
                contract: OracleContract {
                    identity: original.identity(),
                    variables: original
                        .coordinates()
                        .iter()
                        .map(|c| Variable {
                            id: c.id,
                            lower: c.lower,
                            upper: c.upper,
                        })
                        .collect(),
                    rows: vec![id(3)],
                    derivatives: order,
                    smoothness: order,
                },
                jac: AssemblyMatrix::new(1, 2, original.incidence(), 10).unwrap(),
                hess: (order >= DerivativeOrder::Second).then(|| {
                    AssemblyMatrix::new(
                        2,
                        2,
                        &[
                            Entry::new(GlobalRow::new(0), GlobalCol::new(0)),
                            Entry::new(GlobalRow::new(1), GlobalCol::new(0)),
                            Entry::new(GlobalRow::new(1), GlobalCol::new(1)),
                        ],
                        10,
                    )
                    .unwrap()
                }),
                guard_failure: false,
                cancel: None,
                delay: false,
                normalization: None,
            }
        }
    }
    impl NlpOracle for Source {
        fn contract(&self) -> &OracleContract {
            &self.contract
        }
        fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
            self.normalization.as_ref()
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.jac.matrix().symbolic()
        }
        fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
            self.hess.as_ref().map(|h| h.matrix().symbolic())
        }
        fn constraint_bounds(&self) -> &[(f64, f64)] {
            &[(0.0, 100.0)]
        }
        fn objective(&mut self, _: &[f64]) -> Result<f64, ProblemError> {
            panic!("authored objective must remain excluded from auxiliary objective")
        }
        fn gradient(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
            panic!("authored objective gradient must remain excluded")
        }
        fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = x[0] * x[0] + x[1] * x[1] + 3.0 * x[0] * x[1];
            if let Some(cancel) = &self.cancel {
                cancel.store(true, Ordering::Release);
            }
            if self.delay {
                std::thread::sleep(Duration::from_millis(150));
            }
            if self.guard_failure {
                return Err(MathError::OutsideRange {
                    source_id: id(8),
                    target: id(9),
                    value: x[0],
                    lower: Some(3.0),
                    upper: None,
                }
                .into());
            }
            Ok(())
        }
        fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out.copy_from_slice(&[2.0 * x[0] + 3.0 * x[1], 2.0 * x[1] + 3.0 * x[0]]);
            Ok(())
        }
        fn hessian(
            &mut self,
            _: &[f64],
            weight: f64,
            multipliers: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            assert_eq!(weight, 0.0);
            assert!(self.hess.is_some());
            out.copy_from_slice(&[
                2.0 * multipliers[0],
                3.0 * multipliers[0],
                2.0 * multipliers[0],
            ]);
            Ok(())
        }
    }
    fn binding() -> OriginalNlpBinding {
        OriginalNlpBinding {
            original: hash(1),
            source: hash(1),
            preparation: hash(10),
        }
    }
    fn scope() -> ExecutionScope {
        ExecutionScope::new(Arc::new(AtomicBool::new(false)), None)
    }
    #[test]
    fn auxiliary_dimensionless_objective_keeps_original_coordinate_and_row_scales() {
        let normalization = pse_math::normalization::Normalization {
            variables: vec![2.0, 4.0],
            rows: vec![3.0],
            objective: 7.0,
        };
        let base = original(DerivativeOrder::First);
        let physical = Arc::new(
            OriginalContract::new(
                base.identity(),
                normalization.key(),
                base.coordinates().to_vec(),
                base.constraints().to_vec(),
                base.incidence().to_vec(),
                base.support(),
                base.obligations(),
            )
            .unwrap(),
        );
        let family = Arc::new(
            LeastDeviation::new(
                physical,
                vec![1.0, 1.0],
                vec![2.0, 4.0],
                vec![8.0, 2.0],
                vec![GlobalCol::new(0)],
                vec![GlobalCol::new(1)],
                hash(7),
            )
            .unwrap(),
        );
        let mut source = Source::new(DerivativeOrder::First);
        source.normalization = Some(normalization.clone());
        let oracle =
            LeastDeviationOracle::new(Box::new(source), family.clone(), binding(), scope(), 10)
                .unwrap();
        assert_eq!(
            oracle.contract().identity,
            LeastDeviationOracle::contract_identity(&family, binding())
        );
        let auxiliary = oracle.normalization().unwrap();
        assert_eq!(auxiliary.variables, normalization.variables);
        assert_eq!(auxiliary.rows, normalization.rows);
        assert_eq!(auxiliary.objective, 1.0);
        assert_eq!(
            oracle.into_original().normalization().unwrap(),
            &normalization
        );
    }
    #[test]
    fn explicit_frozen_binding_keeps_richer_original_identity_distinct_from_native_source() {
        let native_original = original(DerivativeOrder::First);
        let rich = Arc::new(
            OriginalContract::new(
                hash(30),
                native_original.normalization(),
                native_original.coordinates().to_vec(),
                native_original.constraints().to_vec(),
                native_original.incidence().to_vec(),
                native_original.support(),
                native_original.obligations(),
            )
            .unwrap(),
        );
        let family = Arc::new(
            LeastDeviation::new(
                rich,
                vec![1.0, 1.0],
                vec![2.0, 4.0],
                vec![8.0, 2.0],
                vec![GlobalCol::new(0)],
                vec![GlobalCol::new(1)],
                hash(7),
            )
            .unwrap(),
        );
        let binding = OriginalNlpBinding {
            original: hash(30),
            source: hash(1),
            preparation: hash(10),
        };
        let oracle = LeastDeviationOracle::new(
            Box::new(Source::new(DerivativeOrder::First)),
            family.clone(),
            binding,
            scope(),
            10,
        )
        .unwrap();
        assert_eq!(oracle.source_binding(), binding);
        assert_eq!(oracle.family().original().identity(), hash(30));
        let changed = LeastDeviationOracle::new(
            Box::new(Source::new(DerivativeOrder::First)),
            family,
            OriginalNlpBinding {
                preparation: hash(11),
                ..binding
            },
            scope(),
            10,
        )
        .unwrap();
        assert_ne!(oracle.contract().identity, changed.contract().identity);
    }
    #[test]
    fn auxiliary_derivatives_preserve_original_rows_and_exact_constraint_curvature() {
        let family = family(DerivativeOrder::Second);
        let mut oracle = LeastDeviationOracle::new(
            Box::new(Source::new(DerivativeOrder::Second)),
            family.clone(),
            binding(),
            scope(),
            10,
        )
        .unwrap();
        assert_ne!(oracle.contract().identity, family.key());
        assert_eq!(oracle.source_binding(), binding());
        assert_eq!(oracle.contract().variables[1].lower, 1.0);
        assert_eq!(oracle.contract().variables[1].upper, 1.0);
        assert_eq!(oracle.constraint_bounds(), [(0.0, 100.0)]);
        assert!((oracle.objective(&[2.0, 1.0]).unwrap() - 1.0).abs() < 1e-14);
        let mut gradient = [99.0; 2];
        oracle.gradient(&[2.0, 1.0], &mut gradient).unwrap();
        assert_eq!(gradient, [2.0, 0.0]);
        let mut constraint = [99.0];
        oracle.constraints(&[2.0, 1.0], &mut constraint).unwrap();
        assert_eq!(constraint, [11.0]);
        let mut jac = [99.0; 2];
        oracle.jacobian(&[2.0, 1.0], &mut jac).unwrap();
        assert_eq!(jac, [7.0, 8.0]);
        let mut hess = [99.0; 3];
        oracle.hessian(&[2.0, 1.0], 2.0, &[5.0], &mut hess).unwrap();
        assert_eq!(hess, [14.0, 15.0, 10.0]);
        assert_eq!(oracle.into_original().contract().identity, hash(1));
    }
    #[test]
    fn first_constraints_never_gain_fabricated_second_and_preparation_checks_source() {
        let family = family(DerivativeOrder::First);
        let mut oracle = LeastDeviationOracle::new(
            Box::new(Source::new(DerivativeOrder::First)),
            family.clone(),
            binding(),
            scope(),
            10,
        )
        .unwrap();
        assert_eq!(oracle.contract().derivatives, DerivativeOrder::First);
        assert!(oracle.hessian_pattern().is_none());
        let mut hess = [99.0; 3];
        assert!(matches!(
            oracle.hessian(&[2.0, 1.0], 1.0, &[1.0], &mut hess),
            Err(ProblemError::Unsupported(_))
        ));
        assert_eq!(hess, [99.0; 3]);
        let mut source = Source::new(DerivativeOrder::First);
        source.contract.variables[0].id = id(20);
        assert!(matches!(
            LeastDeviationOracle::new(Box::new(source), family.clone(), binding(), scope(), 10),
            Err(ProblemError::Contract(_))
        ));
        let mut source = Source::new(DerivativeOrder::First);
        source.contract.identity = hash(20);
        assert!(matches!(
            LeastDeviationOracle::new(Box::new(source), family.clone(), binding(), scope(), 10),
            Err(ProblemError::Contract(_))
        ));
        let mut source = Source::new(DerivativeOrder::First);
        source.jac = AssemblyMatrix::new(
            1,
            2,
            &[Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
            10,
        )
        .unwrap();
        assert!(matches!(
            LeastDeviationOracle::new(Box::new(source), family.clone(), binding(), scope(), 10),
            Err(ProblemError::Contract(_))
        ));
        assert!(
            LeastDeviationOracle::new(
                Box::new(Source::new(DerivativeOrder::First)),
                family,
                binding(),
                scope(),
                1
            )
            .is_err()
        );
    }
    #[test]
    fn held_bounds_and_original_guard_failure_publish_no_partial_products() {
        let mut source = Source::new(DerivativeOrder::First);
        source.guard_failure = true;
        let mut oracle = LeastDeviationOracle::new(
            Box::new(source),
            family(DerivativeOrder::First),
            binding(),
            scope(),
            10,
        )
        .unwrap();
        let mut output = [99.0; 2];
        assert!(
            matches!(oracle.gradient(&[2.0,1.0],&mut output),Err(ProblemError::Math(MathError::OutsideRange{source_id,target,..})) if source_id==id(8)&&target==id(9))
        );
        assert_eq!(output, [99.0; 2]);
        assert!(matches!(
            oracle.gradient(&[2.0, 1.1], &mut output),
            Err(ProblemError::Math(MathError::Contract(_)))
        ));
        assert_eq!(output, [99.0; 2]);
        assert!(
            matches!(oracle.gradient(&[5.1,1.0],&mut output),Err(ProblemError::Math(MathError::OutsideRange{source_id,..})) if source_id==id(1))
        );
        assert_eq!(output, [99.0; 2]);
    }
    #[test]
    fn original_scope_cancellation_and_late_supplier_exit_stop_atomic_publication() {
        let cancel = Arc::new(AtomicBool::new(true));
        assert!(matches!(
            LeastDeviationOracle::new(
                Box::new(Source::new(DerivativeOrder::First)),
                family(DerivativeOrder::First),
                binding(),
                ExecutionScope::new(cancel.clone(), None),
                10
            ),
            Err(ProblemError::Provider(ProviderError::Cancelled))
        ));
        cancel.store(false, Ordering::Release);
        let mut source = Source::new(DerivativeOrder::First);
        source.cancel = Some(cancel.clone());
        let mut oracle = LeastDeviationOracle::new(
            Box::new(source),
            family(DerivativeOrder::First),
            binding(),
            ExecutionScope::new(cancel.clone(), None),
            10,
        )
        .unwrap();
        let mut output = [99.0];
        assert!(matches!(
            oracle.constraints(&[2.0, 1.0], &mut output),
            Err(ProblemError::Provider(ProviderError::Cancelled))
        ));
        assert_eq!(output, [99.0]);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut source = Source::new(DerivativeOrder::First);
        source.delay = true;
        let mut oracle = LeastDeviationOracle::new(
            Box::new(source),
            family(DerivativeOrder::First),
            binding(),
            ExecutionScope::new(
                cancel.clone(),
                Some(Instant::now() + Duration::from_millis(100)),
            ),
            10,
        )
        .unwrap();
        assert!(matches!(
            oracle.constraints(&[2.0, 1.0], &mut output),
            Err(ProblemError::Provider(ProviderError::Deadline))
        ));
        assert_eq!(output, [99.0]);
        assert!(!cancel.load(Ordering::Acquire));
    }
}
