// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Verified bulk updates, exact granted witnesses and uncertain-mutation retirement.
use super::*;
use crate::{
    execution::{Retained, SessionUpdate},
    solver_tests::{id, stamp},
};
use faer::sparse::{SparseColMat, Triplet};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
struct Storage {
    live: usize,
    peak: usize,
    denied: bool,
}
#[derive(Debug, Default)]
struct Admission(Arc<Mutex<Storage>>);
struct Grant {
    storage: Arc<Mutex<Storage>>,
    bytes: usize,
}
impl Drop for Grant {
    fn drop(&mut self) {
        self.storage.lock().unwrap().live -= self.bytes;
    }
}
impl WorkAdmission for Admission {
    fn admit(&self, _: WorkEvidence) -> Result<(), ProblemError> {
        Ok(())
    }
    fn observe(&self, _: WorkEvidence) -> Result<(), ProblemError> {
        Ok(())
    }
    fn reserve_retained_storage(
        &self,
        _: &str,
        bytes: usize,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, ProblemError> {
        let mut state = self.0.lock().unwrap();
        if state.denied {
            return Ok(None);
        }
        state.live += bytes;
        state.peak = state.peak.max(state.live);
        Ok(Some(Arc::new(Grant {
            storage: self.0.clone(),
            bytes,
        })))
    }
}
fn execution() -> Execution {
    Execution::new(Default::default(), &Controls::default())
}
fn granted() -> (Execution, Arc<Admission>) {
    let admission = Arc::new(Admission::default());
    let mut execution = execution();
    execution.work_admission = Some(admission.clone());
    (execution, admission)
}
fn problem() -> CoefficientProblem {
    let mut contract = crate::solver_tests::contract();
    contract.variables = (0..2)
        .map(|c| crate::Variable {
            id: id(c + 1),
            lower: 0.,
            upper: 3.,
        })
        .collect();
    contract.rows = vec![id(3)];
    CoefficientProblem {
        contract,
        objective: vec![-1., -2.],
        objective_constant: 7.,
        sense: ObjectiveSense::Minimize,
        domains: vec![ModelingVariableDomain::Continuous; 2],
        assumptions: stamp(Backend::Highs).data,
        constraints: SparseColMat::try_new_from_triplets(
            1,
            2,
            &[Triplet::new(0, 0, 1.), Triplet::new(0, 1, 1.)],
        )
        .unwrap(),
        hessian: None,
        bounds: vec![(f64::NEG_INFINITY, 4.)],
        objectives: vec![],
    }
}
fn new(p: &CoefficientProblem, execution: &Execution) -> Session {
    let certificate = p
        .hessian
        .as_ref()
        .map(|h| crate::solver_tests::certify(h, 1.));
    Session::new(
        p,
        certificate
            .as_ref()
            .map(|c| -> &dyn pse_math::convexity::QuadraticEvidence { c }),
        stamp(Backend::Highs),
        execution,
    )
    .unwrap()
}
fn update(
    s: &mut Session,
    p: &CoefficientProblem,
    execution: &Execution,
) -> Result<SessionUpdate, ProblemError> {
    let certificate = p
        .hessian
        .as_ref()
        .map(|h| crate::solver_tests::certify(h, 1.));
    s.update(
        p,
        certificate
            .as_ref()
            .map(|c| -> &dyn pse_math::convexity::QuadraticEvidence { c }),
        stamp(Backend::Highs),
        execution,
    )
}
fn solve(
    s: &mut Session,
    p: &CoefficientProblem,
    controls: &Controls,
) -> Result<SolveReport, ProblemError> {
    s.solve(
        p,
        &pse_math::normalization::Normalization::identity(2, 1),
        controls,
        &ResolvedAccuracy::verification(),
        &Settings::default(),
        Execution::new(Default::default(), controls),
        &Tolerances {
            variables: vec![1e-8; 2],
            rows: vec![1e-8],
            integrality: 1e-8,
        },
        None,
    )
}
fn equivalent_to_fresh(s: &mut Session, p: &CoefficientProblem) {
    verify_upload(s.model().unwrap(), p).unwrap();
    let actual = solve(s, p, &Controls::default()).unwrap();
    // Fresh native construction under the same thread's sole scheduler owner. A second
    // Session would incorrectly acquire nested lifecycle ownership on this thread.
    let retained = s.model.take();
    s.model = Some(upload(p).unwrap());
    let expected = solve(s, p, &Controls::default()).unwrap();
    s.model = retained;
    assert_eq!(actual.termination.category, Termination::Success);
    assert_eq!(actual.termination.category, expected.termination.category);
    let (actual, expected) = (actual.candidate.unwrap(), expected.candidate.unwrap());
    assert!((actual.objective.unwrap() - expected.objective.unwrap()).abs() < 1e-7);
    assert!(
        actual
            .primal
            .iter()
            .zip(&expected.primal)
            .all(|(a, b)| (a - b).abs() < 1e-7)
    );
}

#[test]
fn highs_bulk_fallback_and_granted_deltas_match_fresh_originals() {
    for grant in [false, true] {
        let (granted, _) = granted();
        let execution = if grant { granted } else { execution() };
        let mut p = problem();
        let mut s = new(&p, &execution);
        assert_eq!(s.uploaded.is_some(), grant);
        // Cost, column bound, row bound, single coefficient and both zero directions.
        for step in 0..7 {
            match step {
                0 => p.objective[0] = -3.,
                1 => p.contract.variables[0].upper = 2.,
                2 => p.bounds[0].1 = 3.,
                3 => p.constraints.val_mut()[0] = 2.,
                4 => p.constraints.val_mut()[0] = 0.,
                5 => p.constraints.val_mut()[0] = 1.,
                _ => {
                    p.sense = ObjectiveSense::Maximize;
                    p.objective_constant = -4.;
                    p.objective = vec![3., 1.];
                    p.contract.variables[1].upper = 2.;
                    p.bounds[0] = (0., 2.5);
                    p.constraints.val_mut().copy_from_slice(&[2., 3.]);
                }
            }
            test_mutations::reset(None);
            assert_eq!(
                update(&mut s, &p, &execution).unwrap(),
                SessionUpdate::Compatible
            );
            assert_eq!(test_mutations::count("update costs"), 1);
            assert_eq!(test_mutations::count("update column bounds"), 1);
            assert_eq!(test_mutations::count("update row bounds"), 1);
            assert_eq!(
                test_mutations::count("update coefficient"),
                if grant {
                    if step < 3 {
                        0
                    } else if step == 6 {
                        2
                    } else {
                        1
                    }
                } else {
                    2
                }
            );
            assert_eq!(
                test_mutations::count("exact Hessian upload"),
                usize::from(!grant)
            );
            equivalent_to_fresh(&mut s, &p);
        }
    }
}

#[test]
fn highs_witness_tracks_hessian_values_structure_and_presence() {
    let (execution, _) = granted();
    let mut p = problem();
    let mut s = new(&p, &execution);
    for (column, value) in [
        (Some(0), 2.),
        (Some(0), 2.),
        (Some(0), 4.),
        (Some(1), 4.),
        (None, 0.),
    ] {
        let previous = p.hessian.clone();
        p.hessian = column.map(|c| {
            SparseColMat::try_new_from_triplets(2, 2, &[Triplet::new(c, c, value)]).unwrap()
        });
        let unchanged = match (&previous, &p.hessian) {
            (Some(a), Some(b)) => {
                a.col_ptr() == b.col_ptr() && a.row_idx() == b.row_idx() && a.val() == b.val()
            }
            (None, None) => true,
            _ => false,
        };
        test_mutations::reset(None);
        assert_eq!(
            update(&mut s, &p, &execution).unwrap(),
            SessionUpdate::Compatible
        );
        assert_eq!(
            test_mutations::count("exact Hessian upload"),
            usize::from(!unchanged)
        );
        equivalent_to_fresh(&mut s, &p);
    }
}

#[test]
fn highs_witness_uses_exact_bits_and_releases_each_grant() {
    let (execution, admission) = granted();
    let mut p = problem();
    p.constraints.val_mut()[0] = 0.;
    let mut s = new(&p, &execution);
    let bytes = admission.0.lock().unwrap().live;
    assert!(bytes > 0);
    p.constraints.val_mut()[0] = -0.;
    test_mutations::reset(None);
    update(&mut s, &p, &execution).unwrap();
    assert_eq!(test_mutations::count("update coefficient"), 1);
    assert_eq!(admission.0.lock().unwrap().live, bytes);
    assert_eq!(admission.0.lock().unwrap().peak, 2 * bytes);
    assert_eq!(s.uploaded.as_ref().unwrap().a[0], (-0.0f64).to_bits());
    // Capacity refusal retires the old premise after a complete update.
    admission.0.lock().unwrap().denied = true;
    test_mutations::reset(None);
    update(&mut s, &p, &execution).unwrap();
    assert_eq!(test_mutations::count("update coefficient"), 2);
    assert!(s.uploaded.is_none());
    assert_eq!(admission.0.lock().unwrap().live, 0);
    admission.0.lock().unwrap().denied = false;
    update(&mut s, &p, &execution).unwrap();
    let owner = Arc::downgrade(&admission);
    let storage = admission.0.clone();
    drop(execution);
    drop(admission);
    assert!(
        owner.upgrade().is_none(),
        "witness cannot retain a task admission oracle"
    );
    assert!(storage.lock().unwrap().live > 0);
    drop(s);
    assert_eq!(storage.lock().unwrap().live, 0);
}

#[test]
fn highs_incompatible_update_has_no_effects_or_new_grant() {
    let (execution, admission) = granted();
    let p = problem();
    let mut s = new(&p, &execution);
    let bytes = admission.0.lock().unwrap().live;
    let mut changed = p.clone();
    changed.constraints =
        SparseColMat::try_new_from_triplets(1, 2, &[Triplet::new(0, 1, 1.)]).unwrap();
    test_mutations::reset(None);
    assert_eq!(
        update(&mut s, &changed, &execution).unwrap(),
        SessionUpdate::Incompatible
    );
    assert_eq!(test_mutations::count("update objective sense"), 0);
    assert_eq!(admission.0.lock().unwrap().peak, bytes);
    verify_upload(s.model().unwrap(), &p).unwrap();
    let mut incompatible = stamp(Backend::Highs);
    incompatible.profile = pse_ids::ContentHash::from_bytes([99; 32]);
    assert_eq!(
        s.update(&p, None, incompatible, &execution).unwrap(),
        SessionUpdate::Incompatible
    );
}

#[test]
fn highs_partial_mutation_and_verification_failure_destroy_retained_state_even_require_reuse() {
    for operation in [
        "update costs",
        "update column bounds",
        "update coefficient",
        "update row bounds",
        "update offset",
        "full model readback",
    ] {
        let (execution, admission) = granted();
        let mut p = problem();
        let mut retained = Retained::default();
        retained
            .session(
                Backend::Highs,
                ReusePolicy::AllowRebuild,
                |_: &mut Session| Ok(SessionUpdate::Compatible),
                || Ok(new(&p, &execution)),
            )
            .unwrap();
        p.objective[0] = -3.;
        p.constraints.val_mut()[0] = 2.;
        test_mutations::reset(Some(operation));
        let error = retained
            .session(
                Backend::Highs,
                ReusePolicy::RequireReuse,
                |s: &mut Session| update(s, &p, &execution),
                || panic!("RequireReuse must not rebuild after mutation failure"),
            )
            .unwrap_err();
        assert!(
            matches!(error, ProblemError::Contract(ref message) if message == &format!("injected HiGHS {operation} failure"))
        );
        assert!(retained.is_empty());
        assert_eq!(admission.0.lock().unwrap().live, 0);
        assert!(test_mutations::count("update objective sense") > 0);
    }
}

#[test]
fn highs_binary_bulk_bounds_remain_clipped() {
    let mut p = problem();
    p.domains[0] = ModelingVariableDomain::Binary;
    p.contract.variables[0].lower = -10.;
    p.contract.variables[0].upper = 10.;
    let mut s = new(&p, &execution());
    p.contract.variables[0].lower = -3.;
    p.contract.variables[0].upper = 4.;
    update(&mut s, &p, &execution()).unwrap();
    equivalent_to_fresh(&mut s, &p);
}

#[test]
fn highs_diagnostic_options_restore_and_failed_restore_invalidates() {
    for fail in [
        None,
        Some("restore diagnostic time limit"),
        Some("restore diagnostic IIS time limit"),
        Some("restore diagnostic IIS strategy"),
    ] {
        let p = problem();
        let mut s = new(&p, &execution());
        let before = option_snapshot(s.model().unwrap().as_ptr()).unwrap().0;
        test_mutations::reset(fail);
        let result = s.diagnose(
            &p,
            &diagnostics::Request {
                iis: true,
                ..Default::default()
            },
            &execution(),
        );
        match fail {
            Some(operation) => {
                assert!(result.unwrap_err().to_string().contains(operation));
                assert!(s.model.is_none());
                assert!(s.uploaded.is_none());
            }
            None => {
                result.unwrap();
                let after = option_snapshot(s.model().unwrap().as_ptr()).unwrap().0;
                for key in ["time_limit", "iis_time_limit", "iis_strategy"] {
                    assert_eq!(before[key], after[key]);
                }
                verify_upload(s.model().unwrap(), &p).unwrap();
            }
        }
    }
}

#[test]
fn highs_lexicographic_restore_and_early_solve_failure_invalidate() {
    for fail in [
        None,
        Some("restore costs"),
        Some("restore linear objectives"),
        Some("full model readback"),
    ] {
        let mut p = problem();
        p.objectives = vec![
            crate::LinearObjective {
                coefficients: p.objective.clone(),
                constant: p.objective_constant,
                sense: p.sense,
                degradation: Some(pse_math::binding::Degradation {
                    absolute: 0.,
                    relative: 0.,
                }),
            },
            crate::LinearObjective {
                coefficients: vec![1., 0.],
                constant: 11.,
                sense: ObjectiveSense::Maximize,
                degradation: None,
            },
        ];
        let mut s = new(&p, &execution());
        test_mutations::reset(fail);
        let result = solve(&mut s, &p, &Controls::default());
        if let Some(operation) = fail {
            assert!(result.unwrap_err().to_string().contains(operation));
            assert!(s.model.is_none());
        } else {
            let report = result.unwrap();
            assert_eq!(
                report.metrics["lexicographic.objectives"],
                Metric::Integer(2)
            );
            let candidate = report.candidate.unwrap();
            assert!((candidate.primal[0] - 1.).abs() < 1e-8);
            assert!((candidate.primal[1] - 3.).abs() < 1e-8);
            verify_upload(s.model().unwrap(), &p).unwrap();
            assert_eq!(test_mutations::count("restore linear objectives"), 1);
            let controls = Controls {
                options: BTreeMap::from([("unknown_c5_option".into(), OptionValue::Bool(true))]),
                ..Controls::default()
            };
            assert!(
                solve(&mut s, &p, &controls)
                    .unwrap_err()
                    .to_string()
                    .contains("unknown_c5_option")
            );
            assert!(s.model.is_none());
        }
    }
}

#[test]
fn highs_independent_reset_keeps_verified_model_and_clears_request_state() {
    for fail in [
        None,
        Some("reset independent solver state"),
        Some("reset independent options"),
    ] {
        let (execution, admission) = granted();
        let p = problem();
        let mut s = new(&p, &execution);
        let controls = Controls {
            options: BTreeMap::from([
                ("presolve".into(), OptionValue::Text("off".into())),
                ("random_seed".into(), OptionValue::Integer(123)),
            ]),
            ..Controls::default()
        };
        solve(&mut s, &p, &controls).unwrap();
        s.pending_sparse = Some(BTreeMap::from([(p.contract.variables[0].id, 1.)]));
        let before = admission.0.lock().unwrap().live;
        test_mutations::reset(fail);
        let result = s.reset_for_new_application();
        if let Some(operation) = fail {
            assert!(result.unwrap_err().to_string().contains(operation));
            assert!(s.model.is_none());
            assert_eq!(admission.0.lock().unwrap().live, 0);
        } else {
            result.unwrap();
            assert!(s.pending_sparse.is_none());
            assert!(s.cut_pool.is_none());
            assert_eq!(admission.0.lock().unwrap().live, before);
            verify_upload(s.model().unwrap(), &p).unwrap();
            let (effective, defaults) = option_snapshot(s.model().unwrap().as_ptr()).unwrap();
            assert_eq!(effective, defaults);
            equivalent_to_fresh(&mut s, &p);
        }
    }
}
