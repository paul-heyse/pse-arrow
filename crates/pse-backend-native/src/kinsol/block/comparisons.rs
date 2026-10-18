// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Discriminating controls for explicit existing profiles; these do not select a default.
use super::super::*;
use crate::{NleOracle, OracleContract, Variable};

#[derive(Clone, Debug)]
struct MatrixRoot {
    contract: OracleContract,
    matrix: faer::sparse::SparseColMat<usize, f64>,
    nonlinear: f64,
}
impl MatrixRoot {
    fn new(case: usize) -> Self {
        let n = 8usize;
        let mut entries = Vec::new();
        for column in 0..n {
            for row in 0..n {
                let value = match case {
                    0 => {
                        if row == column {
                            4.
                        } else if row.abs_diff(column) == 1 {
                            0.01
                        } else {
                            0.
                        }
                    }
                    1 => {
                        if row == column {
                            3.
                        } else {
                            1.
                        }
                    }
                    2 => {
                        if row == column {
                            2.
                        } else if column == row + 1 {
                            20.
                        } else if row == column + 1 {
                            0.001
                        } else {
                            0.
                        }
                    }
                    _ => {
                        if row == column {
                            4.
                        } else if row == (column + 1) % n || column == (row + 1) % n {
                            1.
                        } else {
                            0.
                        }
                    }
                };
                if value != 0. {
                    entries.push(faer::sparse::Triplet::new(row, column, value));
                }
            }
        }
        Self {
            contract: OracleContract {
                identity: pse_ids::ContentHash::from_bytes([case as u8 + 81; 32]),
                variables: (0..n)
                    .map(|i| Variable {
                        id: crate::solver_tests::id(i as u8 + 1),
                        lower: f64::NEG_INFINITY,
                        upper: f64::INFINITY,
                    })
                    .collect(),
                rows: (0..n)
                    .map(|i| crate::solver_tests::id(i as u8 + 21))
                    .collect(),
                derivatives: pse_kernels::DerivativeOrder::First,
                smoothness: pse_kernels::DerivativeOrder::First,
            },
            matrix: faer::sparse::SparseColMat::try_new_from_triplets(n, n, &entries).unwrap(),
            nonlinear: if case == 1 { 0.1 } else { 0. },
        }
    }
}
impl NleOracle for MatrixRoot {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.matrix.symbolic()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(0.);
        for (column, &value) in x.iter().enumerate() {
            for index in self.matrix.symbolic().col_range(column) {
                out[self.matrix.symbolic().row_idx()[index]] +=
                    self.matrix.val()[index] * (value - 1.);
            }
        }
        for (out, x) in out.iter_mut().zip(x) {
            *out += self.nonlinear * (x - 1.).powi(3);
        }
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(self.matrix.val());
        for (column, &value) in x.iter().enumerate() {
            for index in self.matrix.symbolic().col_range(column) {
                if self.matrix.symbolic().row_idx()[index] == column {
                    out[index] += 3. * self.nonlinear * (value - 1.).powi(2);
                }
            }
        }
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out.fill(0.);
        for (column, &seed) in direction.iter().enumerate() {
            for index in self.matrix.symbolic().col_range(column) {
                out[self.matrix.symbolic().row_idx()[index]] += self.matrix.val()[index] * seed;
            }
        }
        for i in 0..x.len() {
            out[i] += 3. * self.nonlinear * (x[i] - 1.).powi(2) * direction[i];
        }
        Ok(())
    }
}
#[test]
fn existing_root_profiles_preserve_original_quality_across_coupling_and_fill_controls() {
    let n = 8usize;
    let tolerances = Tolerances {
        variables: vec![1e-8; n],
        rows: vec![1e-8; n],
        integrality: 1e-8,
    };
    let normalization = pse_math::normalization::Normalization::identity(n, n);
    let accuracy =
        ResolvedAccuracy::resolve(&Default::default(), &tolerances, &normalization).unwrap();
    let controls = Controls::default();
    for case in 0..4 {
        for start in [0., 1.5] {
            for setup_interval in [1, 10] {
                for preconditioned in [false, true] {
                    let linear = if preconditioned {
                        Linear::Spgmr {
                            dimension: pse_model::scalars::PositiveCount::try_new(n).unwrap(),
                        }
                    } else {
                        Linear::Klu
                    };
                    let method = Method {
                        linear,
                        preconditioner: if preconditioned {
                            Preconditioner::BlockFactor
                        } else {
                            Preconditioner::None
                        },
                        setup_interval,
                        ..Default::default()
                    };
                    let settings = Settings::from_policy(
                        method,
                        &tolerances,
                        &normalization,
                        accuracy.feasibility,
                    );
                    let mut execution = crate::solver_tests::execution();
                    execution.memory = Some(128 << 20);
                    let stamp = crate::solver_tests::stamp(Backend::Kinsol);
                    let started = std::time::Instant::now();
                    let mut session = Session::new(
                        Function::Equations(Box::new(MatrixRoot::new(case))),
                        settings,
                        execution.clone(),
                        stamp,
                    )
                    .unwrap();
                    let report = session
                        .solve(
                            &vec![start; n],
                            &controls,
                            &accuracy,
                            execution,
                            &tolerances,
                            None,
                        )
                        .unwrap();
                    assert!(
                        report.quality.as_ref().is_some_and(|q| q.feasible()),
                        "case={case} block={preconditioned} start={start} setup={setup_interval}: {report:?}"
                    );
                    eprintln!(
                        "profile-control case={case} block={preconditioned} start={start} setup={setup_interval} elapsed_us={} work={:?} block_factor={:?} block_solve={:?}",
                        started.elapsed().as_micros(),
                        report.evidence.work,
                        report.metrics.get("preconditioner.block.factor_calls"),
                        report.metrics.get("preconditioner.block.solve_calls")
                    );
                }
            }
        }
    }
}
#[test]
fn gilbert_peierls_symbolic_reuse_is_distinct_from_markowitz_pivoting() {
    for case in 0..4 {
        let source = MatrixRoot::new(case);
        let n = source.matrix.ncols();
        let mut matrix = feral::SparseColMatrix {
            m: n,
            col_ptr: source.matrix.symbolic().col_ptr().to_vec(),
            row_idx: source.matrix.symbolic().row_idx().to_vec(),
            values: source.matrix.val().to_vec(),
        };
        let symbolic = feral::SparseLuSymbolic::analyze(&matrix).unwrap();
        for pass in 0..2 {
            if pass == 1 {
                for column in 0..n {
                    for index in matrix.col_ptr[column]..matrix.col_ptr[column + 1] {
                        if matrix.row_idx[index] == column {
                            matrix.values[index] += 0.5;
                        }
                    }
                }
            }
            for gp in [false, true] {
                let mut factor = feral::SparseLu::factor(
                    &matrix,
                    &symbolic,
                    feral::LuParams {
                        pivoting: if gp {
                            feral::LuPivoting::GilbertPeierls
                        } else {
                            feral::LuPivoting::Markowitz
                        },
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(factor.used_markowitz(), !gp);
                let mut rhs = vec![0.; n];
                for column in 0..n {
                    for index in matrix.col_ptr[column]..matrix.col_ptr[column + 1] {
                        rhs[matrix.row_idx[index]] += matrix.values[index];
                    }
                }
                factor.ftran(&mut rhs).unwrap();
                assert!(rhs.iter().all(|value| (value - 1.).abs() < 1e-7));
                eprintln!(
                    "factor-control case={case} pass={pass} gp={gp} fill={} growth={}",
                    factor.factor_nnz(),
                    factor.growth()
                );
            }
        }
    }
}
#[cfg(feature = "petsc")]
#[test]
fn existing_petsc_gmres_ilu_preserves_same_original_coupling_controls() {
    let n = 8usize;
    let tolerances = Tolerances {
        variables: vec![1e-8; n],
        rows: vec![1e-8; n],
        integrality: 1e-8,
    };
    let accuracy = ResolvedAccuracy {
        native_scaling: false,
        ..ResolvedAccuracy::verification()
    };
    let controls = Controls::default();
    let settings = crate::petsc::Settings::default();
    for case in 0..4 {
        for start in [0., 1.5] {
            let mut execution = crate::solver_tests::execution();
            execution.memory = Some(128 << 20);
            let mut source = MatrixRoot::new(case);
            let stamp = crate::solver_tests::stamp(Backend::Petsc);
            let report = crate::petsc::solve(
                &mut source,
                &vec![start; n],
                &settings,
                crate::petsc::SolveRequest {
                    controls: &controls,
                    accuracy: &accuracy,
                    execution,
                    tolerances: &tolerances,
                    warm: None,
                    compatibility: &stamp,
                },
            )
            .unwrap();
            assert!(
                report.quality.as_ref().is_some_and(|q| q.feasible()),
                "case={case} start={start}: {report:?}"
            );
            eprintln!(
                "petsc-profile-control case={case} start={start} work={:?}",
                report.evidence.work
            );
        }
    }
}

#[test]
fn regular_full_operator_does_not_qualify_a_zero_a11_schur_split() {
    let matrix = feral::SparseColMatrix {
        m: 2,
        col_ptr: vec![0, 2, 4],
        row_idx: vec![0, 1, 0, 1],
        values: vec![0., 1., 1., 0.],
    };
    let mut full = feral::SparseLu::factor_markowitz(&matrix, Default::default()).unwrap();
    let mut rhs = [2., 3.];
    full.ftran(&mut rhs).unwrap();
    assert_eq!(rhs, [3., 2.]);
    let a11 = feral::SparseColMatrix {
        m: 1,
        col_ptr: vec![0, 1],
        row_idx: vec![0],
        values: vec![0.],
    };
    assert!(feral::SparseLu::factor_markowitz(&a11, Default::default()).is_err());
}
