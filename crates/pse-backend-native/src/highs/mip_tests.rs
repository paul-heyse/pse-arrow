// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! MILP and LP extras (Plan 22 C2, G8): fixed-commitment LP duals, streamed incumbents, the node budget,
//! the basis-inverse, presolve and cut-pool views, and the QP regularization.
use super::*;
use crate::solver_tests::{id, stamp};
use faer::sparse::{SparseColMat, Triplet};
use std::sync::Arc;

/// `min Σ cost·x` over `x ∈ [lower, upper]` with the given domains and rows.
fn problem(
    cost: &[f64],
    columns: &[(f64, f64, ModelingVariableDomain)],
    rows: &[(Vec<(usize, f64)>, f64, f64)],
) -> CoefficientProblem {
    let mut contract = crate::solver_tests::contract();
    contract.variables = columns
        .iter()
        .enumerate()
        .map(|(j, &(lower, upper, _))| crate::Variable {
            id: id(10 + j as u8),
            lower,
            upper,
        })
        .collect();
    contract.rows = (0..rows.len()).map(|i| id(60 + i as u8)).collect();
    let entries = rows
        .iter()
        .enumerate()
        .flat_map(|(i, (terms, _, _))| terms.iter().map(move |&(j, v)| Triplet::new(i, j, v)))
        .collect::<Vec<_>>();
    CoefficientProblem {
        contract,
        objective: cost.to_vec(),
        objective_constant: 0.0,
        sense: ObjectiveSense::Minimize,
        domains: columns.iter().map(|c| c.2).collect(),
        assumptions: stamp(Backend::Highs).data,
        constraints: SparseColMat::try_new_from_triplets(rows.len(), columns.len(), &entries)
            .unwrap(),
        hessian: None,
        bounds: rows.iter().map(|r| (r.1, r.2)).collect(),
    }
}
/// The coordinates of a problem that is its own original.
fn identity(p: &CoefficientProblem) -> pse_math::normalization::Normalization {
    pse_math::normalization::Normalization::identity(
        p.contract.variables.len(),
        p.contract.rows.len(),
    )
}
fn solve(
    session: &mut Session,
    p: &CoefficientProblem,
    controls: &Controls,
    settings: &Settings,
) -> SolveReport {
    session
        .solve(
            p,
            &identity(p),
            controls,
            &ResolvedAccuracy::nominal(),
            settings,
            Execution::new(Default::default(), controls),
            &Tolerances {
                variables: vec![1e-8; p.contract.variables.len()],
                rows: vec![1e-8; p.contract.rows.len()],
                integrality: 1e-8,
            },
            None,
        )
        .unwrap()
}
fn diagnose(
    session: &mut Session,
    p: &CoefficientProblem,
    request: diagnostics::Request,
) -> diagnostics::Report {
    session
        .diagnose(
            p,
            &request,
            &Execution::new(Default::default(), &Controls::default()),
        )
        .unwrap()
}
const CONTINUOUS: ModelingVariableDomain = ModelingVariableDomain::Continuous;
const INTEGER: ModelingVariableDomain = ModelingVariableDomain::Integer;
const BINARY: ModelingVariableDomain = ModelingVariableDomain::Binary;

/// `min -x - 2y` with `x + y ≤ 3.5`, `x ≤ 0.4`, `x ∈ [0, 10]` and integer `y ∈ [0, 10]`:
/// the MIP commits to `y = 3`, and then only `x ≤ 0.4` binds.
fn commitment_milp(y: ModelingVariableDomain, y_bounds: (f64, f64)) -> CoefficientProblem {
    problem(
        &[-1.0, -2.0],
        &[(0.0, 10.0, CONTINUOUS), (y_bounds.0, y_bounds.1, y)],
        &[
            (vec![(0, 1.0), (1, 1.0)], f64::NEG_INFINITY, 3.5),
            (vec![(0, 1.0)], f64::NEG_INFINITY, 0.4),
        ],
    )
}

#[test]
fn native_ranging_fills_every_family_on_its_side() {
    use diagnostics::{RangeFamily, RangeSide};
    let p = commitment_milp(CONTINUOUS, (0.0, 10.0));
    let mut session = Session::new(&p, None, stamp(Backend::Highs)).unwrap();
    let report = solve(&mut session, &p, &Controls::default(), &Settings::default());
    assert_eq!(report.termination.category, Termination::Success);
    let evidence = diagnose(
        &mut session,
        &p,
        diagnostics::Request {
            ranging: true,
            ..Default::default()
        },
    );
    let ranging = evidence
        .ranging
        .unwrap_or_else(|| panic!("{:?}", evidence.unavailable));
    let columns: Vec<_> = p.contract.variables.iter().map(|v| v.id).collect();
    for (family, range) in &ranging {
        let ids = match family.side() {
            RangeSide::Column => &columns,
            RangeSide::Row => &p.contract.rows,
        };
        assert_eq!(&range.ids, ids, "{family}");
        assert_eq!(range.value.len(), ids.len(), "{family}");
    }
    // Each current cost and binding row bound lies inside its own family pair's range:
    // y (cost -2) is basic, and both rows bind at x = 0.4, y = 3.1.
    let (down, up) = (
        &ranging[RangeFamily::ColumnCostDown],
        &ranging[RangeFamily::ColumnCostUp],
    );
    assert!(
        down.value[1] <= -2.0 && -2.0 <= up.value[1],
        "{down:?} {up:?}"
    );
    let (down, up) = (
        &ranging[RangeFamily::RowBoundDown],
        &ranging[RangeFamily::RowBoundUp],
    );
    for (row, bound) in [(0, 3.5), (1, 0.4)] {
        assert!(
            down.value[row] <= bound + 1e-9 && bound - 1e-9 <= up.value[row],
            "{row}: {down:?} {up:?}"
        );
    }
}

#[test]
fn fixed_lp_duals_conditional_on_commitment() {
    let p = commitment_milp(INTEGER, (0.0, 10.0));
    let mut session = Session::new(&p, None, stamp(Backend::Highs)).unwrap();
    let report = solve(&mut session, &p, &Controls::default(), &Settings::default());
    assert_eq!(report.termination.category, Termination::Success);
    let candidate = report.candidate.as_ref().unwrap();
    assert!((candidate.primal[1] - 3.0).abs() < 1e-9, "{candidate:?}");
    // A MIP has no duals of its own.
    assert!(candidate.row_dual.is_none());
    let evidence = diagnose(
        &mut session,
        &p,
        diagnostics::Request {
            fixed_lp: true,
            ..Default::default()
        },
    );
    let fixed = evidence
        .fixed_lp
        .unwrap_or_else(|| panic!("{:?}", evidence.unavailable));
    // The duals are conditional on the recorded commitment y = 3.
    assert_eq!(
        fixed.commitment.columns,
        vec![(p.contract.variables[1].id, 3.0)]
    );
    assert_eq!(fixed.termination.category, Termination::Success);
    let (rows, reduced) = (fixed.row_dual.unwrap(), fixed.reduced_costs.unwrap());
    // They equal the duals of the LP with y fixed by hand at that commitment.
    let lp = commitment_milp(CONTINUOUS, (3.0, 3.0));
    drop(session);
    let mut session = Session::new(&lp, None, stamp(Backend::Highs)).unwrap();
    let manual = solve(
        &mut session,
        &lp,
        &Controls::default(),
        &Settings::default(),
    );
    let manual = manual.candidate.unwrap();
    for (a, b) in rows.iter().zip(manual.row_dual.as_ref().unwrap()) {
        assert!((a - b).abs() < 1e-9, "{rows:?} {manual:?}");
    }
    for (a, b) in reduced.iter().zip(manual.reduced_costs.as_ref().unwrap()) {
        assert!((a - b).abs() < 1e-9, "{reduced:?} {manual:?}");
    }
    // With y = 3 the row x ≤ 0.4 binds (dual -1) and x + y ≤ 3.5 does not.
    assert!(
        rows[0].abs() < 1e-9 && (rows[1] + 1.0).abs() < 1e-9,
        "{rows:?}"
    );
    assert!((fixed.objective.unwrap() - -6.4).abs() < 1e-9);
    // A continuous model has no commitment to condition on.
    let evidence = diagnose(
        &mut session,
        &lp,
        diagnostics::Request {
            fixed_lp: true,
            ..Default::default()
        },
    );
    assert!(evidence.fixed_lp.is_none());
    assert!(evidence.unavailable.contains_key("fixed_lp"));
}

/// A 0-1 knapsack: `max Σ v·x` with `Σ w·x ≤ capacity`, as a minimization.
fn knapsack() -> CoefficientProblem {
    let values = [10.0, 13.0, 7.0, 8.0, 11.0, 9.0, 12.0, 6.0, 5.0, 14.0];
    let weights = [5.0, 7.0, 4.0, 4.0, 6.0, 5.0, 7.0, 3.0, 3.0, 8.0];
    problem(
        &values.map(|v| -v),
        &[(0.0, 1.0, BINARY); 10],
        &[(
            weights.iter().copied().enumerate().collect(),
            f64::NEG_INFINITY,
            23.0,
        )],
    )
}

/// Collects every event a solve reports, as a durable stream would.
#[derive(Debug, Default)]
struct Collected(std::sync::Mutex<Vec<Event>>);
impl ProgressTap for Collected {
    fn observe(&self, event: &Event) {
        self.0.lock().unwrap().push(event.clone());
    }
}

/// HiGHS streams every improving solution as a typed incumbent (Plan 22 G8), in original
/// coordinates and units: the objective constant and the normalization's scales applied,
/// its bounds typed rather than repeated as callback metrics. The first solution is
/// captured at once; the last one equals the result.
#[test]
fn highs_incumbents_streamed() {
    // The knapsack, a continuous column worth 2 per unit on [0, 5] and a constant.
    let values = [10.0, 13.0, 7.0, 8.0, 11.0, 9.0, 12.0, 6.0, 5.0, 14.0];
    let weights = [5.0, 7.0, 4.0, 4.0, 6.0, 5.0, 7.0, 3.0, 3.0, 8.0];
    let mut cost: Vec<f64> = values.iter().map(|v| -v).collect();
    cost.push(-2.0);
    let mut columns = vec![(0.0, 1.0, BINARY); 10];
    columns.push((0.0, 5.0, CONTINUOUS));
    let mut p = problem(
        &cost,
        &columns,
        &[(
            weights.iter().copied().enumerate().collect(),
            f64::NEG_INFINITY,
            23.0,
        )],
    );
    p.objective_constant = 7.0;
    // Nontrivial coordinates: the native model scales the continuous column, the row and
    // the objective (integer columns keep unit scales).
    let n = pse_math::normalization::Normalization {
        variables: [vec![1.0; 10], vec![4.0]].concat(),
        rows: vec![4.0],
        objective: 8.0,
    };
    let (native, _) = crate::transport::coefficients(&p, &n, None).unwrap();
    let mut session = Session::new(&native, None, stamp(Backend::Highs)).unwrap();
    // An empty knapsack is a poor feasible start, so the search improves on it.
    session
        .sparse_start(
            &native,
            &native
                .contract
                .variables
                .iter()
                .map(|v| (v.id, 0.0))
                .collect(),
        )
        .unwrap();
    let controls = Controls::default();
    let tap = Arc::new(Collected::default());
    let mut execution = Execution::new(Default::default(), &controls);
    execution.progress = Arc::new(Progress::tapped(controls.history, tap.clone()));
    let mut report = session
        .solve(
            &native,
            &n,
            &controls,
            &ResolvedAccuracy::nominal(),
            &Settings::default(),
            execution,
            &Tolerances {
                variables: vec![1e-8; 11],
                rows: vec![1e-8],
                integrality: 1e-8,
            },
            None,
        )
        .unwrap();
    assert_eq!(report.termination.category, Termination::Success);
    crate::transport::recover(&mut report, &n, &p.contract).unwrap();
    let events = tap.0.lock().unwrap().clone();
    let incumbents: Vec<&IncumbentEvent> =
        events.iter().filter_map(|e| e.incumbent.as_ref()).collect();
    assert!(incumbents.len() >= 2, "{incumbents:?}");
    // One representation: incumbent events carry no callback metrics.
    assert!(
        events
            .iter()
            .filter(|e| e.incumbent.is_some())
            .all(|e| e.phase == "highs.incumbent" && e.values.is_empty())
    );
    // In order, each strictly better than the one before.
    for pair in incumbents.windows(2) {
        assert!(pair[0].seconds <= pair[1].seconds && pair[0].nodes <= pair[1].nodes);
        assert!(pair[1].objective <= pair[0].objective, "{incumbents:?}");
    }
    // The first solution is captured at once, in original coordinates; every captured
    // solution is feasible and its objective (constant included) is the event's.
    assert!(incumbents[0].primal.is_some());
    for incumbent in &incumbents {
        if let Some(primal) = &incumbent.primal {
            assert_eq!(primal.len(), 11);
            assert!(
                (p.objective_at(primal) - incumbent.objective).abs() < 1e-7,
                "{} vs {}",
                p.objective_at(primal),
                incumbent.objective
            );
            assert!(
                p.quality(
                    primal,
                    &Tolerances {
                        variables: vec![1e-7; 11],
                        rows: vec![1e-7],
                        integrality: 1e-7,
                    }
                )
                .unwrap()
                .feasible()
            );
        }
        assert!(
            incumbent
                .dual_bound
                .is_none_or(|d| d <= incumbent.objective + 1e-7)
        );
    }
    // The last incumbent is the result, under the post-solve objective convention. Its
    // capture, deferred by the throttle in so short a search, is taken when the search
    // ends: the continuous column at its upper bound, in original units.
    let last = incumbents.last().unwrap();
    let primal = last.primal.as_ref().unwrap();
    assert!((primal[10] - 5.0).abs() < 1e-7, "{primal:?}");
    let candidate = report.candidate.as_ref().unwrap();
    assert!((last.objective - candidate.objective.unwrap()).abs() < 1e-7);
    // The retained in-memory events keep the incumbents too, bounded like any event.
    assert!(report.events.iter().any(|e| e.incumbent.is_some()));
}

#[test]
fn mip_node_budget_independent() {
    let p = knapsack();
    let mut session = Session::new(&p, None, stamp(Backend::Highs)).unwrap();
    let integer = |r: &SolveReport, key: &str| match r.options.get(key) {
        Some(OptionValue::Integer(v)) => *v,
        other => panic!("{key}: {other:?}"),
    };
    // A small iteration budget leaves the node budget at the native default.
    let controls = Controls {
        iterations: 7,
        ..Controls::default()
    };
    let report = solve(&mut session, &p, &controls, &Settings::default());
    assert_eq!(integer(&report, "simplex_iteration_limit"), 7);
    assert_eq!(integer(&report, "mip_max_nodes"), i32::MAX);
    // A node budget leaves the iteration budget alone, and a retained model forgets it.
    let nodes = Settings {
        nodes: Some(5),
        ..Settings::default()
    };
    let report = solve(&mut session, &p, &Controls::default(), &nodes);
    assert_eq!(integer(&report, "mip_max_nodes"), 5);
    assert_eq!(integer(&report, "simplex_iteration_limit"), 3000);
    let report = solve(&mut session, &p, &Controls::default(), &Settings::default());
    assert_eq!(integer(&report, "mip_max_nodes"), i32::MAX);
    // The node budget is typed: raw options cannot set it and zero is refused.
    let raw = Controls {
        options: Options::from([("mip_max_nodes".into(), OptionValue::Integer(5))]),
        ..Controls::default()
    };
    assert!(
        session
            .solve(
                &p,
                &identity(&p),
                &raw,
                &ResolvedAccuracy::nominal(),
                &Settings::default(),
                Execution::new(Default::default(), &raw),
                &Tolerances {
                    variables: vec![1e-8; 10],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                None,
            )
            .is_err()
    );
    let zero = Settings {
        nodes: Some(0),
        ..Settings::default()
    };
    assert!(
        session
            .solve(
                &p,
                &identity(&p),
                &Controls::default(),
                &ResolvedAccuracy::nominal(),
                &zero,
                Execution::new(Default::default(), &Controls::default()),
                &Tolerances {
                    variables: vec![1e-8; 10],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                None,
            )
            .is_err()
    );
}

#[test]
fn basis_inverse_and_presolve_views() {
    // min x + 2y with x + y ≥ 1 and x ≤ 0.7: the unique optimum is (0.7, 0.3).
    let p = problem(
        &[1.0, 2.0],
        &[(0.0, 10.0, CONTINUOUS), (0.0, 10.0, CONTINUOUS)],
        &[
            (vec![(0, 1.0), (1, 1.0)], 1.0, f64::INFINITY),
            (vec![(0, 1.0)], f64::NEG_INFINITY, 0.7),
        ],
    );
    let mut session = Session::new(&p, None, stamp(Backend::Highs)).unwrap();
    let report = solve(
        &mut session,
        &p,
        &Controls::default(),
        &Settings {
            method: Method::Simplex,
            ..Settings::default()
        },
    );
    let x = report.candidate.as_ref().unwrap().primal.clone();
    assert!(
        (x[0] - 0.7).abs() < 1e-9 && (x[1] - 0.3).abs() < 1e-9,
        "{x:?}"
    );
    let evidence = diagnose(
        &mut session,
        &p,
        diagnostics::Request {
            basis_inverse: Some(vec![0, 1]),
            presolve: true,
            ..Default::default()
        },
    );
    let view = evidence
        .basis_inverse
        .unwrap_or_else(|| panic!("{:?}", evidence.unavailable));
    // Rebuild B from the basic variables (HiGHS's logical columns are +e_i) and check
    // that each returned row of B⁻¹ times B is the matching unit row.
    let column = |b: &diagnostics::Basic| -> Vec<f64> {
        match b {
            diagnostics::Basic::Column(id) => {
                let j = p
                    .contract
                    .variables
                    .iter()
                    .position(|v| v.id == *id)
                    .unwrap();
                let mut c = vec![0.0; 2];
                for (i, v) in p
                    .constraints
                    .row_idx_of_col(j)
                    .zip(p.constraints.val_of_col(j))
                {
                    c[i] = *v;
                }
                c
            }
            diagnostics::Basic::Row(id) => {
                let i = p.contract.rows.iter().position(|r| r == id).unwrap();
                let mut c = vec![0.0; 2];
                c[i] = 1.0;
                c
            }
        }
    };
    assert_eq!(view.basic.len(), 2);
    for (position, entries) in &view.rows {
        for (k, basic) in view.basic.iter().enumerate() {
            let b = column(basic);
            let product: f64 = entries.iter().map(|(i, v)| v * b[*i]).sum();
            let expected = if k == *position { 1.0 } else { 0.0 };
            assert!((product - expected).abs() < 1e-9, "{view:?}");
        }
    }
    // Presolve on a copy: its LP's solution postsolves to the model's optimum.
    let presolved = evidence.presolved.unwrap();
    assert!(presolved.columns <= 2 && presolved.rows <= 2);
    let postsolved = presolved.postsolved.unwrap();
    assert!(
        postsolved.iter().zip(&x).all(|(a, b)| (a - b).abs() < 1e-9),
        "{postsolved:?}"
    );
    // Positions must be distinct and in range.
    let evidence = diagnose(
        &mut session,
        &p,
        diagnostics::Request {
            basis_inverse: Some(vec![0, 0]),
            ..Default::default()
        },
    );
    assert!(evidence.unavailable.contains_key("basis_inverse"));
}

#[test]
fn cut_pool_captured_on_request() {
    // A multidimensional knapsack whose root relaxation leaves a gap for branching; the
    // cut pool is extracted after root cut generation.
    let mut state = 12345_u64;
    let mut next = |range: f64| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        1.0 + ((state >> 33) as f64 / (1u64 << 31) as f64 * range).floor()
    };
    let n = 40;
    let cost = (0..n).map(|_| -next(90.0)).collect::<Vec<_>>();
    let rows = (0..5)
        .map(|_| {
            let terms = (0..n).map(|j| (j, next(60.0))).collect::<Vec<_>>();
            let capacity = terms.iter().map(|t| t.1).sum::<f64>() / 3.0;
            (terms, f64::NEG_INFINITY, capacity.floor())
        })
        .collect::<Vec<_>>();
    let p = problem(&cost, &vec![(0.0, 1.0, BINARY); n], &rows);
    let mut session = Session::new(&p, None, stamp(Backend::Highs)).unwrap();
    let request = diagnostics::Request {
        cut_pool: true,
        ..Default::default()
    };
    // Without native presolve the MIP solver's LP has the model's columns.
    let controls = Controls {
        options: Options::from([
            ("presolve".into(), OptionValue::Text("off".into())),
            ("mip_heuristic_effort".into(), OptionValue::Real(0.0)),
            (
                "mip_heuristic_run_feasibility_jump".into(),
                OptionValue::Bool(false),
            ),
            ("mip_heuristic_run_rins".into(), OptionValue::Bool(false)),
            ("mip_heuristic_run_rens".into(), OptionValue::Bool(false)),
            (
                "mip_heuristic_run_root_reduced_cost".into(),
                OptionValue::Bool(false),
            ),
        ]),
        ..Controls::default()
    };
    let settings = Settings {
        diagnostics: request.clone(),
        ..Settings::default()
    };
    let report = solve(&mut session, &p, &controls, &settings);
    assert_eq!(report.termination.category, Termination::Success);
    let evidence = diagnose(&mut session, &p, request.clone());
    let pool = evidence
        .cut_pool
        .unwrap_or_else(|| panic!("{:?}", evidence.unavailable));
    assert_eq!(pool.columns, n);
    assert_eq!(pool.dropped, 0);
    assert!(
        pool.cuts
            .iter()
            .all(|c| c.lower <= c.upper && c.entries.iter().all(|(j, _)| *j < n))
    );
    // The pool belongs to one solve: a second request without a new capture is empty.
    let evidence = diagnose(&mut session, &p, request);
    assert!(evidence.cut_pool.is_none() && evidence.unavailable.contains_key("cut_pool"));
    // The retained model stops the kind after the requesting solve.
    let cut_kind = |r: &SolveReport| {
        r.events.iter().any(|e| {
            e.values.get("kind")
                == Some(&Metric::Integer(i64::from(
                    ffi::kHighsCallbackMipGetCutPool,
                )))
        })
    };
    assert!(cut_kind(&report));
    let again = solve(&mut session, &p, &controls, &Settings::default());
    assert!(!cut_kind(&again));
}

#[test]
fn qp_regularization_within_gap_budget() {
    // Two variables in [-10, 10]: the bounding box has ‖x‖² ≤ 200.
    let accuracy = ResolvedAccuracy {
        gap_absolute: 1e-6,
        ..ResolvedAccuracy::nominal()
    };
    let mut p = problem(
        &[0.0, 0.0],
        &[(-10.0, 10.0, CONTINUOUS), (-10.0, 10.0, CONTINUOUS)],
        &[],
    );
    assert!((qp_regularization(&p, &accuracy) - 1e-8).abs() < 1e-20);
    // Never above the native default; an unbounded coordinate counts at unit scale.
    p.contract.variables[0].upper = f64::INFINITY;
    let loose = ResolvedAccuracy {
        gap_absolute: 1.0,
        ..ResolvedAccuracy::nominal()
    };
    assert_eq!(qp_regularization(&p, &loose), 1e-7);
}
