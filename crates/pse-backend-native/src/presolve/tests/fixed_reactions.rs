// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Reactions omitted for declared fixed columns by the pinned affine wrapper.
use super::*;
use pse_math::normalization::Normalization;

#[derive(Debug)]
struct Fixed(Mixed);
impl Fixed {
    fn new(sign: f64, scaled: bool) -> Self {
        let mut source = Mixed::new();
        source.sign = sign;
        source.contract.variables = vec![
            Variable {
                id: id(1),
                lower: 2.0,
                upper: 2.0,
            },
            Variable {
                id: id(2),
                lower: 3.0,
                upper: 3.0,
            },
            Variable {
                id: id(5),
                lower: 0.0,
                upper: 10.0,
            },
            Variable {
                id: id(6),
                lower: 0.0,
                upper: 10.0,
            },
        ];
        source.bounds = vec![(7.0, 7.0), (11.0, 11.0)];
        source.facts.affine[0] = Some(AffineRow {
            entries: BTreeMap::from([(2, 1.0), (3, 1.0)]),
            constant: 3.0,
        });
        source.facts.tapes[0].ops = vec![
            Op::Var(2),
            Op::Var(3),
            Op::Add(0, 1),
            Op::Const(3.0),
            Op::Add(2, 3),
        ];
        source.facts.objective_linear = vec![true, true, false, false];
        let entries: Vec<_> = [(1, 0), (1, 1), (0, 2), (1, 2), (0, 3), (1, 3)]
            .into_iter()
            .map(|(r, c)| Entry::new(OriginalRow::new(r), OriginalCol::new(c)))
            .collect();
        source.j = AssemblyMatrix::new(2, 4, &entries, 100).unwrap();
        source.h = hessian(4, &[(2, 0), (3, 1), (2, 2), (3, 3)]);
        if scaled {
            source.normalization = Some(Normalization {
                variables: vec![10.0, 0.5, 2.0, 4.0],
                rows: vec![3.0, 7.0],
                objective: 100.0,
            });
        }
        Self(source)
    }
}
impl NlpOracle for Fixed {
    fn normalization(&self) -> Option<&Normalization> {
        self.0.normalization()
    }
    fn presolve_facts(&self) -> Option<&Facts> {
        self.0.presolve_facts()
    }
    fn contract(&self) -> &OracleContract {
        self.0.contract()
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.0.bounds
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.0.j.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        Some(self.0.h.matrix().symbolic())
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        Ok(self.0.sign * (10.0 * x[0] + 5.0 * x[1] + x[2] * x[2] + x[3] * x[3]))
    }
    fn constraints(&mut self, x: &[f64], g: &mut [f64]) -> Result<(), ProblemError> {
        g.copy_from_slice(&[x[2] + x[3] + 3.0, x[0] * x[2] + x[1] * x[3]]);
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], g: &mut [f64]) -> Result<(), ProblemError> {
        if self.0.fail {
            return Err(ProblemError::Contract("original derivative refusal".into()));
        }
        g.copy_from_slice(&[
            self.0.sign * 10.0,
            self.0.sign * 5.0,
            self.0.sign * 2.0 * x[2],
            self.0.sign * 2.0 * x[3],
        ]);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], g: &mut [f64]) -> Result<(), ProblemError> {
        g.copy_from_slice(&[x[2], x[3], 1.0, x[0], 1.0, x[1]]);
        Ok(())
    }
    fn hessian(&mut self, _: &[f64], w: f64, l: &[f64], g: &mut [f64]) -> Result<(), ProblemError> {
        g.copy_from_slice(&[l[1], l[1], 2.0 * self.0.sign * w, 2.0 * self.0.sign * w]);
        Ok(())
    }
}
fn tolerance() -> Tolerances {
    Tolerances {
        variables: vec![1e-8; 4],
        rows: vec![1e-8; 2],
        integrality: 1e-8,
    }
}
fn candidate(sign: f64) -> Candidate {
    Candidate {
        kind: CandidateKind::FinalIterate,
        primal: vec![2.0, 3.0, 1.0, 3.0],
        objective: Some(sign * 45.0),
        row_dual: Some(vec![sign * 6.0, sign * (-4.0)]),
        bound_dual: Some((vec![0.0; 4], vec![0.0; 4])),
        reduced_costs: None,
        slacks: None,
        commitment: None,
    }
}
fn manufactured(
    policy: &Policy,
    sign: f64,
    scaled: bool,
    row_error: f64,
    duals: bool,
) -> SolveReport {
    let source = Fixed::new(sign, scaled);
    let accuracy = ResolvedAccuracy::verification();
    let normalization = source
        .normalization()
        .cloned()
        .unwrap_or_else(|| Normalization::identity(4, 2))
        .with_row_budgets(&tolerance().rows, accuracy.feasibility)
        .unwrap();
    let mut pipeline = Pipeline::new(
        Box::new(source),
        &[2.0, 3.0, 1.0, 3.0],
        policy,
        &tolerance(),
        &accuracy,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    let rows: Vec<_> = pipeline.report().rows.iter().map(|r| r.get()).collect();
    let initial = pipeline.initial().to_vec();
    let mut oracle = pipeline.take_oracle().unwrap();
    let mut report = SolveReport::new(
        Backend::Ipopt,
        oracle.contract(),
        NativeTermination {
            code: 0,
            name: "manufactured-native-finalization".into(),
            message: None,
            category: Termination::Success,
            assurance: Assurance::LocalStationary,
        },
        &execution(),
    );
    let sense = if sign > 0.0 {
        ObjectiveSense::Minimize
    } else {
        ObjectiveSense::Maximize
    };
    let physical_rows = [sign * 6.0, sign * (-4.0) + row_error];
    report.candidate = Some(Candidate {
        primal: initial.clone(),
        objective: Some(oracle.objective(&initial).unwrap() * sense.sign()),
        row_dual: duals.then(|| {
            rows.iter()
                .map(|r| physical_rows[*r] * normalization.rows[*r] / normalization.objective)
                .collect()
        }),
        bound_dual: duals.then(|| (vec![0.0; initial.len()], vec![0.0; initial.len()])),
        ..candidate(sign)
    });
    pipeline
        .finish(report, &tolerance(), sense, &Analysis::NONE, UNUSED)
        .0
}

#[test]
fn eliminated_fixed_reactions_recover_physical_signs_and_units() {
    for sign in [1.0, -1.0] {
        for scaled in [false, true] {
            let report = manufactured(&Policy::Auto, sign, scaled, 0.0, true);
            let preprocessing = report.preprocessing.as_ref().unwrap();
            assert_eq!(
                preprocessing.dimensions,
                dimensions(4, 2, 1, 1),
                "{report:?}"
            );
            assert!(!preprocessing.columns.iter().any(|c| c.get() < 2));
            let c = report.candidate.as_ref().unwrap();
            assert_eq!(c.primal, vec![2.0, 3.0, 1.0, 3.0]);
            let (zl, zu) = c.bound_dual.as_ref().unwrap();
            let expected = if sign > 0.0 {
                ([6.0, 0.0], [0.0, 7.0])
            } else {
                ([0.0, 7.0], [6.0, 0.0])
            };
            for col in 0..2 {
                assert!((zl[col] - expected.0[col]).abs() < 1e-12, "{report:?}");
                assert!((zu[col] - expected.1[col]).abs() < 1e-12, "{report:?}");
            }
            assert_eq!(&zl[2..], &[0.0, 0.0]);
            assert_eq!(&zu[2..], &[0.0, 0.0]);
            for (actual, expected) in c
                .row_dual
                .as_ref()
                .unwrap()
                .iter()
                .zip([sign * 6.0, sign * (-4.0)])
            {
                assert!((actual - expected).abs() < 1e-12, "{report:?}");
            }
            assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
            assert!(
                report
                    .observation
                    .as_ref()
                    .unwrap()
                    .stationarity
                    .as_ref()
                    .unwrap()
                    .iter()
                    .all(|v| v.abs() < 1e-12),
                "{report:?}"
            );
        }
    }
}

#[test]
fn fixed_reactions_leave_off_missing_duals_and_free_residuals_unchanged() {
    let off = manufactured(&Policy::Off, 1.0, false, 0.0, true);
    let c = off.candidate.as_ref().unwrap();
    assert_eq!(c.bound_dual, Some((vec![0.0; 4], vec![0.0; 4])));
    assert_eq!(c.row_dual, Some(vec![6.0, -4.0]));
    assert_eq!(
        off.observation.as_ref().unwrap().stationarity,
        Some(vec![6.0, -7.0, 0.0, 0.0])
    );
    let missing = manufactured(&Policy::Auto, 1.0, false, 0.0, false);
    assert!(missing.candidate.as_ref().unwrap().bound_dual.is_none());
    assert!(missing.observation.as_ref().unwrap().dual_error.is_some());
    let inconsistent = manufactured(&Policy::Auto, 1.0, false, 0.25, true);
    let c = inconsistent.candidate.as_ref().unwrap();
    assert_eq!(&c.bound_dual.as_ref().unwrap().0[2..], &[0.0, 0.0]);
    assert_eq!(&c.bound_dual.as_ref().unwrap().1[2..], &[0.0, 0.0]);
    assert_eq!(c.row_dual.as_ref().unwrap()[1], -3.75);
    assert!(
        inconsistent
            .observation
            .as_ref()
            .unwrap()
            .stationarity
            .as_ref()
            .unwrap()[2..]
            .iter()
            .any(|v| v.abs() > 0.0)
    );
}

#[test]
fn fixed_reactions_preserve_unselected_and_nonfixed_duals_and_derivative_refusal() {
    let mut source = Fixed::new(1.0, false);
    let mut c = candidate(1.0);
    c.bound_dual = Some((vec![0.0, 0.0, 0.5, 0.25], vec![0.0, 0.0, 0.75, 1.0]));
    crate::quality::recover_fixed_bound_reactions(&mut source, &mut c, &[0, 2]).unwrap();
    assert_eq!(
        c.bound_dual,
        Some((vec![6.0, 0.0, 0.5, 0.25], vec![0.0, 0.0, 0.75, 1.0]))
    );
    assert_eq!(c.row_dual, Some(vec![6.0, -4.0]));
    let before = c.bound_dual.clone();
    source.0.fail = true;
    crate::quality::recover_fixed_bound_reactions(&mut source, &mut c, &[]).unwrap();
    assert_eq!(
        c.bound_dual, before,
        "no eliminated columns must request no derivative work"
    );
    crate::quality::recover_fixed_bound_reactions(&mut source, &mut c, &[0, 1])
        .expect_err("original derivative failure must survive recovery");
    assert_eq!(c.bound_dual, before);
    source.0.fail = false;
    c.bound_dual.as_mut().unwrap().0[2] = -1.0;
    let invalid = c.bound_dual.clone();
    crate::quality::recover_fixed_bound_reactions(&mut source, &mut c, &[0, 1]).unwrap();
    assert_eq!(c.bound_dual, invalid);
    let mut report = SolveReport::new(
        Backend::Ipopt,
        source.contract(),
        NativeTermination {
            code: 0,
            name: "invalid-dual-control".into(),
            message: None,
            category: Termination::Success,
            assurance: Assurance::LocalStationary,
        },
        &execution(),
    );
    report.candidate = Some(c);
    crate::quality::attach_nlp(
        &mut report,
        &mut source,
        &tolerance(),
        ObjectiveSense::Minimize,
    );
    assert!(
        report.observation.as_ref().unwrap().dual_error.is_some(),
        "{report:?}"
    );
}

#[test]
#[cfg(feature = "native-solvers")]
fn eliminated_fixed_reactions_qualify_actual_native_ipopt_original_kkt() {
    let accuracy = ResolvedAccuracy::verification();
    let mut pipeline = Pipeline::new(
        Box::new(Fixed::new(1.0, true)),
        &[2.0, 3.0, 1.0, 3.0],
        &Policy::Auto,
        &tolerance(),
        &accuracy,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    let mut oracle = pipeline.take_oracle().unwrap();
    let native = crate::ipopt::Session::new()
        .solve(
            &mut oracle,
            pipeline.initial(),
            ObjectiveSense::Minimize,
            &Controls::default(),
            &accuracy,
            &Default::default(),
            execution(),
            &pipeline.tolerances(&tolerance()),
            pipeline.warm(),
            pipeline.native_compatibility().clone(),
        )
        .unwrap();
    assert!(
        matches!(
            native.termination.category,
            Termination::Success | Termination::Acceptable
        ),
        "{native:?}"
    );
    let (mut report, _) = pipeline.finish(
        native,
        &tolerance(),
        ObjectiveSense::Minimize,
        &Analysis::NONE,
        UNUSED,
    );
    crate::quality::record_kkt(&mut report, &Normalization::identity(4, 2), &accuracy);
    crate::quality::qualify(&mut report, &accuracy);
    assert!(report.validation_failure().is_none(), "{report:?}");
    assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
    assert_eq!(
        report.termination.assurance,
        Assurance::LocalStationary,
        "{report:?}"
    );
    assert_eq!(
        report.evidence.kkt.as_ref().unwrap().stationarity,
        Some(true),
        "{report:?}"
    );
    assert_eq!(
        report.evidence.kkt.as_ref().unwrap().complementarity,
        Some(true),
        "{report:?}"
    );
}
