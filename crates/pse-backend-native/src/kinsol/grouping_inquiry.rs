// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent original blocks; complete costs are observations, never policy cutoffs.
use super::*;
use faer::sparse::{SparseColMat, Triplet};
use std::{sync::atomic::AtomicBool, time::Instant};

#[derive(Debug)]
struct Blocks {
    contract: OracleContract,
    matrix: SparseColMat<usize, f64>,
    structure: crate::routing::Structure,
    target: Vec<f64>,
    singular: bool,
}
impl Blocks {
    fn new(sizes: &[usize], offset: usize, dense: bool, generation: u8, singular: bool) -> Self {
        let n = sizes.iter().sum();
        let id = |row, i: usize| {
            let mut bytes = ((offset + i) as u128).to_le_bytes();
            bytes[15] = u8::from(row);
            pse_ids::SemanticId::from_bytes(bytes)
        };
        let mut contract = crate::solver_tests::contract();
        contract.variables = (0..n)
            .map(|i| crate::Variable {
                id: id(false, i),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            })
            .collect();
        contract.rows = (0..n).map(|i| id(true, i)).collect();
        let mut entries = Vec::new();
        let mut start = 0;
        for &size in sizes {
            for col in 0..size {
                for row in 0..size {
                    if row == col || dense || row.abs_diff(col) == 1 {
                        let value = if row == col {
                            3. + f64::from(generation) * 0.25
                        } else {
                            0.1 / size as f64
                        };
                        entries.push(Triplet::new(start + row, start + col, value));
                    }
                }
            }
            start += size;
        }
        let matrix = SparseColMat::try_new_from_triplets(n, n, &entries).unwrap();
        let structure = crate::structural::oracle_structure_with_cancel(
            &contract,
            matrix.symbolic(),
            &vec![(0., 0.); n],
            false,
            &AtomicBool::new(false),
        )
        .unwrap();
        let target = (0..n)
            .map(|i| (offset + i) as f64 * 0.01 + f64::from(generation) * 0.1)
            .collect();
        Self {
            contract,
            matrix,
            structure,
            target,
            singular,
        }
    }
    fn residual_at(&self, x: &[f64], out: &mut [f64]) {
        if self.singular {
            out.fill(1.);
            return;
        }
        for (out, (x, target)) in out.iter_mut().zip(x.iter().zip(&self.target)) {
            *out = 0.1 * (x - target).powi(3);
        }
        for col in 0..x.len() {
            for entry in self.matrix.col_ptr()[col]..self.matrix.col_ptr()[col + 1] {
                out[self.matrix.row_idx()[entry]] +=
                    self.matrix.val()[entry] * (x[col] - self.target[col]);
            }
        }
    }
}
impl NleOracle for Blocks {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        Some(&self.structure.witness)
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.residual_at(x, out);
        Ok(())
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.matrix.symbolic()
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        for col in 0..x.len() {
            for (entry, row) in (self.matrix.col_ptr()[col]..self.matrix.col_ptr()[col + 1])
                .zip(self.matrix.symbolic().row_idx_of_col(col))
            {
                out[entry] = if self.singular {
                    0.
                } else {
                    self.matrix.val()[entry]
                        + if row == col {
                            0.3 * (x[col] - self.target[col]).powi(2)
                        } else {
                            0.
                        }
                };
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
        if !self.singular {
            for col in 0..x.len() {
                out[col] += 0.3 * (x[col] - self.target[col]).powi(2) * direction[col];
                for entry in self.matrix.col_ptr()[col]..self.matrix.col_ptr()[col + 1] {
                    out[self.matrix.row_idx()[entry]] += self.matrix.val()[entry] * direction[col];
                }
            }
        }
        Ok(())
    }
}
fn config(
    n: usize,
    linear: Linear,
    preconditioner: Preconditioner,
    interval: u32,
) -> (Settings, Tolerances) {
    let tolerances = Tolerances {
        variables: vec![0.001; n],
        rows: vec![0.001; n],
        integrality: 0.001,
    };
    let settings = Settings::from_policy(
        Method {
            linear,
            preconditioner,
            setup_interval: interval,
            ..Method::default()
        },
        &tolerances,
        &pse_math::normalization::Normalization::identity(n, n),
        0.001,
    );
    (settings, tolerances)
}
fn execution() -> Execution {
    let mut execution = crate::solver_tests::execution();
    execution.memory = Some(64 << 20);
    execution
}
fn stamp(generation: u8) -> Compatibility {
    let mut stamp = crate::solver_tests::stamp(Backend::Kinsol);
    stamp.data = pse_ids::ContentHash::from_bytes([generation; 32]);
    stamp
}
fn solve(
    session: &mut Session,
    initial: &[f64],
    tolerances: &Tolerances,
) -> Result<SolveReport, ProblemError> {
    session.solve(
        initial,
        &Controls::default(),
        &ResolvedAccuracy::from_policy(&Default::default(), 0.001).unwrap(),
        execution(),
        tolerances,
        None,
    )
}
fn verify(
    report: &SolveReport,
    sizes: &[usize],
    offset: usize,
    dense: bool,
    generation: u8,
) -> f64 {
    assert_eq!(report.termination.category, Termination::Success);
    assert!(report.quality.as_ref().unwrap().feasible());
    let x = &report.candidate.as_ref().unwrap().primal;
    let difference = x
        .iter()
        .enumerate()
        .map(|(i, value)| value - ((offset + i) as f64 * 0.01 + f64::from(generation) * 0.1))
        .collect::<Vec<_>>();
    let mut maximum: f64 = 0.;
    let mut start = 0;
    for &size in sizes {
        for row in 0..size {
            let d = difference[start + row];
            let residual = (3. + f64::from(generation) * 0.25) * d
                + 0.1 * d.powi(3)
                + (0..size)
                    .filter(|&col| col != row && (dense || row.abs_diff(col) == 1))
                    .map(|col| 0.1 / size as f64 * difference[start + col])
                    .sum::<f64>();
            maximum = maximum.max(residual.abs());
        }
        start += size;
    }
    assert!(maximum <= 0.001, "independent original residual {maximum}");
    assert!(difference.iter().all(|d| d.abs() <= 0.001));
    maximum
}
#[test]
fn independent_grouping_complete_costs_keep_original_roots_and_fresh_setup() {
    use pse_model::scalars::PositiveCount;
    let routes = [
        (Linear::Klu, Preconditioner::None),
        (
            Linear::Dense {
                limit: PositiveCount::try_new(128).unwrap(),
            },
            Preconditioner::None,
        ),
        (
            Linear::Spgmr {
                dimension: PositiveCount::try_new(20).unwrap(),
            },
            Preconditioner::Jacobi,
        ),
        (
            Linear::Spgmr {
                dimension: PositiveCount::try_new(20).unwrap(),
            },
            Preconditioner::BlockFactor,
        ),
    ];
    for (name, sizes, dense) in [
        ("tiny", vec![1; 8], false),
        ("dense", vec![6; 4], true),
        ("banded", vec![12; 4], false),
        ("skewed", vec![1, 2, 3, 24], false),
    ] {
        for grouped in [false, true] {
            for (linear, preconditioner) in routes {
                for interval in [1, 10] {
                    let total = Instant::now();
                    let grouping = Instant::now();
                    let chunks = if grouped {
                        vec![sizes.clone()]
                    } else {
                        sizes.iter().map(|&n| vec![n]).collect()
                    };
                    let grouping_ns = grouping.elapsed().as_nanos();
                    let mut preparation_ns = 0;
                    let mut solve_ns = 0;
                    let mut retained = 0;
                    let mut retained_peak = 0;
                    let mut offset = 0;
                    let mut maximum: f64 = 0.;
                    let mut setups = Vec::new();
                    for chunk in chunks {
                        let prepared = Instant::now();
                        let source = Blocks::new(&chunk, offset, dense, 0, false);
                        let n = source.target.len();
                        let initial = source.target.iter().map(|x| x + 0.75).collect::<Vec<_>>();
                        let (settings, tolerances) = config(n, linear, preconditioner, interval);
                        let mut session = Session::new(
                            Function::Equations(Box::new(source)),
                            settings.clone(),
                            execution(),
                            stamp(0),
                        )
                        .unwrap();
                        preparation_ns += prepared.elapsed().as_nanos();
                        let started = Instant::now();
                        let report = solve(&mut session, &initial, &tolerances).unwrap();
                        solve_ns += started.elapsed().as_nanos();
                        maximum = maximum.max(verify(&report, &chunk, offset, dense, 0));
                        let bytes = session.retained_bytes();
                        retained += bytes;
                        retained_peak = retained_peak.max(bytes);
                        setups.push(report.metrics.get("klu.numeric_factor.completed").cloned());
                        let replacement = Instant::now();
                        session
                            .replace(
                                Function::Equations(Box::new(Blocks::new(
                                    &chunk, offset, dense, 1, false,
                                ))),
                                settings.clone(),
                                stamp(1),
                            )
                            .unwrap();
                        preparation_ns += replacement.elapsed().as_nanos();
                        let started = Instant::now();
                        let report = solve(&mut session, &initial, &tolerances).unwrap();
                        solve_ns += started.elapsed().as_nanos();
                        maximum = maximum.max(verify(&report, &chunk, offset, dense, 1));
                        if linear == Linear::Klu {
                            assert!(
                                matches!(report.metrics.get("klu.numeric_factor.completed"), Some(Metric::Integer(n)) if *n >= 1)
                            );
                        }
                        offset += n;
                    }
                    eprintln!(
                        "grouping shape={name} grouped={grouped} linear={linear:?} preconditioner={preconditioner:?} interval={interval} grouping_ns={grouping_ns} preparation_ns={preparation_ns} solve_ns={solve_ns} complete_with_independent_check_ns={} session_retained_sum={retained} session_retained_peak={retained_peak} numeric_factors={setups:?} original_max={maximum}",
                        total.elapsed().as_nanos()
                    );
                }
            }
        }
    }
}
#[test]
fn grouped_singular_attempt_does_not_reuse_numeric_setup_on_recovery() {
    for linear in [
        Linear::Klu,
        Linear::Dense {
            limit: pse_model::scalars::PositiveCount::try_new(32).unwrap(),
        },
    ] {
        let sizes = [2, 3];
        let n = 5;
        let (settings, tolerances) = config(n, linear, Preconditioner::None, 10);
        let mut session = Session::new(
            Function::Equations(Box::new(Blocks::new(&sizes, 0, true, 0, false))),
            settings.clone(),
            execution(),
            stamp(0),
        )
        .unwrap();
        let initial = vec![0.75; n];
        verify(
            &solve(&mut session, &initial, &tolerances).unwrap(),
            &sizes,
            0,
            true,
            0,
        );
        session
            .replace(
                Function::Equations(Box::new(Blocks::new(&sizes, 0, true, 1, true))),
                settings.clone(),
                stamp(1),
            )
            .unwrap();
        if let Ok(report) = solve(&mut session, &initial, &tolerances) {
            assert!(!report.quality.as_ref().is_some_and(Quality::feasible));
        }
        session
            .replace(
                Function::Equations(Box::new(Blocks::new(&sizes, 0, true, 2, false))),
                settings,
                stamp(2),
            )
            .unwrap();
        let report = solve(&mut session, &initial, &tolerances).unwrap();
        verify(&report, &sizes, 0, true, 2);
        if linear == Linear::Klu {
            assert!(
                matches!(report.metrics.get("klu.numeric_factor.completed"), Some(Metric::Integer(n)) if *n >= 1)
            );
        }
    }
}
