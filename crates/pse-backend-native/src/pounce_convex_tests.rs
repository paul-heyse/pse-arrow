// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! POUNCE-convex on coefficient programs through the coefficient runner's cone lowering,
//! one batch against its single solves, and its certificates (Plan 22 N5).
use crate::{
    CoefficientProblem,
    clarabel_tests::{Affine, budgets, near, problem, run, scaled},
    execution::{self, BackendSettings, Coefficients, Retained, Step},
    quality::Tolerances,
    solve::*,
    solver_tests::stamp,
};
use faer::sparse::{SparseColMat, Triplet};
use pse_math::{binding::ObjectiveSense, normalization::Normalization};
use std::sync::{Arc, atomic::AtomicBool};

/// min x² + xy + y² − 3x − 4y + 4 over x + y ≤ `rhs`, x ≥ 0.2 and x, y in [0, 5]. The
/// unconstrained minimizer (2/3, 5/3) violates the first row for `rhs` < 7/3, so the row
/// binds: at `rhs` = 2 the objective along it is x² − x − 4 + 4, least at x = 0.5,
/// y = 1.5, with multiplier 0.5 and the second row inactive.
fn qp(rhs: f64) -> (CoefficientProblem, pse_math::convexity::GramCertificate) {
    let mut p = problem(
        &[(0.0, 5.0), (0.0, 5.0)],
        &[(f64::NEG_INFINITY, rhs), (0.2, f64::INFINITY)],
        &[(0, 0, 1.0), (0, 1, 1.0), (1, 0, 1.0)],
        vec![-3.0, -4.0],
        ObjectiveSense::Minimize,
    );
    // ½·[x y]·[[2 1][1 2]]·[x y]ᵀ = x² + xy + y².
    let q = SparseColMat::try_new_from_triplets(
        2,
        2,
        &[
            Triplet::new(0, 0, 2.0),
            Triplet::new(0, 1, 1.0),
            Triplet::new(1, 0, 1.0),
            Triplet::new(1, 1, 2.0),
        ],
    )
    .unwrap();
    let proof = crate::solver_tests::certify(&q, 1.0);
    p.hessian = Some(q);
    (p, proof)
}
fn pounce_convex() -> BackendSettings {
    BackendSettings::PounceConvex(crate::settings::pounce_convex::Settings::default())
}
fn solved(r: &SolveReport) -> &Candidate {
    assert!(
        matches!(
            r.termination.category,
            Termination::Success | Termination::Acceptable
        ),
        "{r:?}"
    );
    assert!(r.quality.as_ref().unwrap().feasible(), "{r:?}");
    assert!(r.validation_failure().is_none(), "{r:?}");
    assert_eq!(
        r.qualification,
        Qualification::OptimalWithinTolerance,
        "{r:?}"
    );
    r.candidate.as_ref().unwrap()
}

/// A convex quadratic program and a mixed linear program solved by POUNCE-convex agree
/// with HiGHS on the primal solution, the objective and one authored-sense multiplier per
/// coefficient row (declared tolerance 1e-6), in scaled coordinates that differ from the
/// original ones.
#[cfg(feature = "highs")]
#[test]
fn pounce_convex_qp_matches_highs() {
    use crate::clarabel_tests::mixed_lp;
    let controls = Controls::default();
    let (q, proof) = qp(2.0);
    let lp = mixed_lp();
    let certified: Option<&dyn pse_math::convexity::QuadraticEvidence> = Some(&proof);
    for (p, evidence) in [(&q, certified), (&lp, None)] {
        let t = budgets(p, 1e-7);
        let n = scaled(p);
        let highs = run(
            p,
            evidence,
            Backend::Highs,
            &BackendSettings::Default,
            &controls,
            &n,
            &t,
        )
        .unwrap();
        let convex = run(
            p,
            evidence,
            Backend::PounceConvex,
            &pounce_convex(),
            &controls,
            &n,
            &t,
        )
        .unwrap();
        assert_eq!(convex.backend, Backend::PounceConvex);
        let (h, c) = (solved(&highs), solved(&convex));
        for (a, b) in h.primal.iter().zip(&c.primal) {
            near(*a, *b, 1e-6);
        }
        near(h.objective.unwrap(), c.objective.unwrap(), 1e-6);
        for (a, b) in h
            .row_dual
            .as_ref()
            .unwrap()
            .iter()
            .zip(c.row_dual.as_ref().unwrap())
        {
            near(*a, *b, 1e-6);
        }
        assert!(convex.provenance["native"].contains("pounce-convex"));
        assert_eq!(convex.metrics["batch"], Metric::Integer(1));
    }
}

/// The steps of one batch (Plan 22 N5): four quadratic programs differing in their row
/// bound run as one parallel batch on two threads, and each report equals the program's
/// own single solve (declared tolerance 1e-7) and records the batch it ran in. A second
/// batch of the same layout starts warm from the first batch's solutions.
#[test]
fn pounce_convex_batch_matches_single_solves() {
    let snapshot = execution::Snapshot::observe(&execution::LINKED);
    if !execution::adapter(Backend::PounceConvex).linked() {
        return;
    }
    let controls = Controls {
        threads: 2,
        ..Controls::default()
    };
    let accuracy = ResolvedAccuracy::nominal();
    let settings = pounce_convex();
    let programs: Vec<_> = [1.2, 1.6, 2.0, 2.4].into_iter().map(qp).collect();
    let budgets: Vec<Tolerances> = programs.iter().map(|(p, _)| budgets(p, 1e-7)).collect();
    let normalizations: Vec<Normalization> = programs.iter().map(|(p, _)| scaled(p)).collect();
    let constants = vec![0.0; 2];
    let singles: Vec<SolveReport> = programs
        .iter()
        .zip(&budgets)
        .zip(&normalizations)
        .map(|(((p, proof), t), n)| {
            run(
                p,
                Some(proof),
                Backend::PounceConvex,
                &settings,
                &Controls::default(),
                n,
                t,
            )
            .unwrap()
        })
        .collect();
    let mut retained = Retained::default();
    let batch = |retained: &mut Retained| {
        let mut originals: Vec<Affine<'_>> = programs.iter().map(|(p, _)| Affine(p)).collect();
        let steps = programs
            .iter()
            .zip(&budgets)
            .zip(&normalizations)
            .zip(originals.iter_mut())
            .map(|((((p, proof), t), n), original)| {
                (
                    Step {
                        snapshot: &snapshot,
                        structure: None,
                        adapter: execution::adapter(Backend::PounceConvex),
                        settings: &settings,
                        controls: &controls,
                        accuracy: &accuracy,
                        execution: Execution::new(Arc::new(AtomicBool::new(false)), &controls),
                        tolerances: t,
                        normalization: n,
                        compatibility: stamp(Backend::PounceConvex),
                        warm: None,
                    },
                    Coefficients {
                        lowered: None,
                        problem: p,
                        certificate: Some(proof),
                        row_constants: &constants,
                        row_bounds: p.bounds.clone(),
                        original,
                    },
                )
            })
            .collect();
        execution::coefficients_batch(retained, steps)
    };
    let first = batch(&mut retained);
    let second = batch(&mut retained);
    assert_eq!(first.len(), 4);
    for (k, ((single, cold), warm)) in singles.iter().zip(&first).zip(&second).enumerate() {
        let (cold, warm) = (cold.as_ref().unwrap(), warm.as_ref().unwrap());
        let s = solved(single);
        for r in [cold, warm] {
            let c = solved(r);
            for (a, b) in s.primal.iter().zip(&c.primal) {
                near(*a, *b, 1e-7);
            }
            near(s.objective.unwrap(), c.objective.unwrap(), 1e-7);
            assert_eq!(r.metrics["batch"], Metric::Integer(4), "{k}");
        }
        assert_eq!(cold.metrics["warm_started"], Metric::Bool(false), "{k}");
        assert_eq!(warm.metrics["warm_started"], Metric::Bool(true), "{k}");
    }
}

/// An infeasible convex quadratic program: POUNCE-convex's Farkas ray over the lowered
/// row and the bounds verifies in original coordinates.
#[test]
fn pounce_convex_farkas_certificate() {
    if !execution::adapter(Backend::PounceConvex).linked() {
        return;
    }
    let (mut p, proof) = qp(2.0);
    // x + y >= 20 over the box [0, 5]²: infeasible.
    p.bounds = vec![(20.0, f64::INFINITY), (0.2, f64::INFINITY)];
    let t = budgets(&p, 1e-7);
    let r = run(
        &p,
        Some(&proof),
        Backend::PounceConvex,
        &pounce_convex(),
        &Controls::default(),
        &Normalization::identity(2, 2),
        &t,
    )
    .unwrap();
    assert_eq!(r.termination.category, Termination::Infeasible, "{r:?}");
    let certificate = r.certificate.as_ref().unwrap();
    assert_eq!(certificate.kind, CertificateKind::PrimalInfeasible);
    assert!(
        certificate
            .verification
            .as_ref()
            .is_some_and(|v| v.verified),
        "{certificate:?}"
    );
    assert!(r.candidate.is_none());
}
