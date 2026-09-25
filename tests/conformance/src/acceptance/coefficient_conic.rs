// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::fixtures::near;
use pse_backend_native::{quality::Tolerances, solve::*, *};
use pse_ids::{ContentHash, SemanticId};
use std::sync::{Arc, atomic::AtomicBool};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn contract(rows: usize) -> OracleContract {
    OracleContract {
        identity: ContentHash::from_bytes([1; 32]),
        variables: vec![Variable {
            id: id(1),
            lower: 0.,
            upper: 10.,
        }],
        rows: (0..rows).map(|i| id(10 + i as u8)).collect(),
        derivatives: pse_kernels::DerivativeOrder::Second,
        smoothness: pse_kernels::DerivativeOrder::Second,
    }
}
fn stamp(backend: Backend) -> Compatibility {
    Compatibility {
        layout: ContentHash::from_bytes([1; 32]),
        data: ContentHash::from_bytes([2; 32]),
        backend,
    }
}
fn execution(c: &Controls) -> Execution {
    Execution::new(Arc::new(AtomicBool::new(false)), c)
}
#[test]
fn coefficient_conic() {
    use faer::sparse::{SparseColMat, Triplet};
    use pse_math::binding::{ObjectiveSense, VariableDomain};
    let controls = Controls::default();
    for domain in [
        VariableDomain::Continuous,
        VariableDomain::Integer,
        VariableDomain::Binary,
    ] {
        let p = CoefficientProblem {
            contract: contract(1),
            objective: vec![-1.],
            objective_constant: 3.,
            sense: ObjectiveSense::Minimize,
            domains: vec![domain],
            assumptions: stamp(Backend::Highs).data,
            constraints: SparseColMat::try_new_from_triplets(1, 1, &[Triplet::new(0, 0, 1.)])
                .unwrap(),
            hessian: None,
            bounds: vec![(0., 2.5)],
        };
        let tolerances = Tolerances {
            variables: vec![1e-7],
            rows: vec![1e-7],
            integrality: 1e-7,
        };
        let mut session = highs::Session::new(&p, None, stamp(Backend::Highs)).unwrap();
        let mut r = session
            .solve(
                &p,
                &controls,
                highs::Method::Choose,
                execution(&controls),
                &tolerances,
                None,
            )
            .unwrap();
        assert_eq!(r.termination.assurance, Assurance::None, "{r:?}");
        quality::qualify(&mut r, &controls.accuracy);
        assert_eq!(r.termination.category, Termination::Success, "{r:?}");
        assert_eq!(r.termination.assurance, Assurance::NativeOptimal, "{r:?}");
        assert!(r.quality.as_ref().unwrap().feasible());
        near(
            r.candidate.as_ref().unwrap().primal[0],
            match domain {
                VariableDomain::Continuous => 2.5,
                VariableDomain::Integer => 2.,
                _ => 1.,
            },
            1e-7,
        );
        assert!(!r.metrics.is_empty());
    }
    use clarabel::{algebra::CscMatrix, solver::SupportedConeT::*};
    // Every advertised cone has an analytic boundary optimum, independent of the solver.
    for (cone, rhs, row, expected) in [
        (ZeroConeT(1), vec![0.], 0, 0.),
        (PSDTriangleConeT(2), vec![0., 0., 1.], 0, 0.),
        (NonnegativeConeT(1), vec![0.], 0, 0.),
        (SecondOrderConeT(3), vec![0., 3., 4.], 0, 5.),
        (ExponentialConeT(), vec![0., 1., 0.], 2, 1.),
        (PowerConeT(0.5), vec![0., 1., 1.], 0, 1.),
        (GenPowerConeT(vec![0.5, 0.5], 1), vec![0., 1., 1.], 0, 1.),
    ] {
        let m = rhs.len();
        let p = ConicProblem {
            contract: contract(m),
            quadratic: CscMatrix::zeros((1, 1)),
            objective: vec![1.],
            constraints: CscMatrix::new(m, 1, vec![0, 1], vec![row], vec![-1.]),
            rhs,
            cones: vec![cone],
            objective_constant: 0.,
        };
        let zero = SparseColMat::try_new_from_triplets(1, 1, &[]).unwrap();
        let certificate =
            GramCertificate::new(&zero, 1., &faer::Mat::zeros(0, 1), &[], 10).unwrap();
        let mut session = conic::Session::new(
            &p,
            &certificate,
            &controls,
            Default::default(),
            conic::Mode::ReusableData,
            stamp(Backend::Clarabel),
        )
        .unwrap();
        let mut r = session
            .solve(
                &p,
                &controls,
                Default::default(),
                execution(&controls),
                &Tolerances {
                    variables: vec![1e-6],
                    rows: vec![1e-6; m],
                    integrality: 1e-7,
                },
            )
            .unwrap();
        assert_eq!(r.termination.assurance, Assurance::None, "{r:?}");
        quality::qualify(&mut r, &controls.accuracy);
        assert_eq!(r.termination.category, Termination::Success, "{r:?}");
        assert_eq!(r.termination.assurance, Assurance::NativeOptimal, "{r:?}");
        assert!(r.quality.as_ref().unwrap().feasible(), "{r:?}");
        near(r.candidate.as_ref().unwrap().primal[0], expected, 2e-5);
    }
    // convexity/integrality are semantic eligibility, never an inferred fallback.
    let q = SparseColMat::try_new_from_triplets(1, 1, &[Triplet::new(0, 0, 2.)]).unwrap();
    let certificate =
        GramCertificate::new(&q, 1., &faer::Mat::from_fn(1, 1, |_, _| 1.), &[2.], 10).unwrap();
    let mut p = CoefficientProblem {
        contract: contract(0),
        objective: vec![-4.],
        objective_constant: 4.,
        sense: ObjectiveSense::Minimize,
        domains: vec![VariableDomain::Continuous],
        assumptions: stamp(Backend::Highs).data,
        constraints: SparseColMat::try_new_from_triplets(0, 1, &[]).unwrap(),
        hessian: Some(q),
        bounds: vec![],
    };
    let mut r = highs::Session::new(&p, Some(&certificate), stamp(Backend::Highs))
        .unwrap()
        .solve(
            &p,
            &controls,
            highs::Method::Choose,
            execution(&controls),
            &Tolerances {
                variables: vec![1e-6],
                rows: vec![],
                integrality: 1e-7,
            },
            None,
        )
        .unwrap();
    assert_eq!(r.termination.assurance, Assurance::None, "{r:?}");
    quality::qualify(&mut r, &controls.accuracy);
    assert_eq!(r.termination.category, Termination::Success, "{r:?}");
    // Native QP regularization can satisfy native stopping while missing the
    // requested original objective gap. The default candidate is only feasible.
    assert_eq!(r.qualification, Qualification::Feasible, "{r:?}");
    assert!(
        matches!(r.metrics.get("primal_dual_objective_error"), Some(Metric::Real(v)) if *v > controls.accuracy.gap_relative)
    );
    near(r.candidate.unwrap().primal[0], 2., 1e-5);
    let mut precise = controls.clone();
    precise
        .options
        .insert("qp_regularization_value".into(), OptionValue::Real(1e-12));
    let mut r = highs::Session::new(&p, Some(&certificate), stamp(Backend::Highs))
        .unwrap()
        .solve(
            &p,
            &precise,
            highs::Method::Choose,
            execution(&precise),
            &Tolerances {
                variables: vec![1e-6],
                rows: vec![],
                integrality: 1e-7,
            },
            None,
        )
        .unwrap();
    quality::qualify(&mut r, &precise.accuracy);
    assert_eq!(
        r.qualification,
        Qualification::OptimalWithinTolerance,
        "{r:?}"
    );
    near(r.candidate.unwrap().primal[0], 2., 1e-8);
    p.domains[0] = VariableDomain::Integer;
    assert!(highs::Session::new(&p, Some(&certificate), stamp(Backend::Highs)).is_err());
    p.domains[0] = VariableDomain::Continuous;
    p.hessian.as_mut().unwrap().val_mut()[0] = -2.;
    assert!(highs::Session::new(&p, Some(&certificate), stamp(Backend::Highs)).is_err());
}

#[test]
fn weighted_native_tears_match_exhaustive_acyclic_decisions() {
    use pse_structural::{flowsheet::*, projection::GraphLimits};
    use std::collections::BTreeSet;
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let mut nodes = (0..3)
        .map(|i| Node {
            id: id(i + 1),
            ports: vec![],
        })
        .collect::<Vec<_>>();
    let mut connections = vec![];
    for (i, (from, to)) in [(0, 1), (0, 1), (1, 2), (2, 0)].into_iter().enumerate() {
        let a = id(60 + i as u8 * 2);
        let b = id(61 + i as u8 * 2);
        nodes[from].ports.push(pse_kernels::Port {
            id: a,
            quantity,
            unit,
        });
        nodes[to].ports.push(pse_kernels::Port {
            id: b,
            quantity,
            unit,
        });
        connections.push(Connection {
            id: id(20 + i as u8),
            from: nodes[from].id,
            to: nodes[to].id,
            decision: id(40 + i as u8),
            bindings: vec![(a, b)],
        });
    }
    for policy in [Policy::Free, Policy::Mandatory, Policy::Forbidden] {
        let decisions = [1., 2., 2., 3.]
            .into_iter()
            .enumerate()
            .map(|(i, cost)| Decision {
                id: id(40 + i as u8),
                cost,
                policy: if i == 0 { policy } else { Policy::Free },
            })
            .collect();
        let graph = FlowGraph::admit(
            Declaration {
                nodes: nodes.clone(),
                connections: connections.clone(),
                decisions,
            },
            &registry,
            GraphLimits {
                nodes: 10,
                edges: 10,
            },
        )
        .unwrap();
        let best = (0..16)
            .filter_map(|mask| {
                let chosen = (0..4)
                    .filter(|i| mask & (1 << i) != 0)
                    .map(|i| id(40 + i))
                    .collect::<BTreeSet<_>>();
                graph.witness(&chosen).ok().map(|_| {
                    graph
                        .declaration()
                        .decisions
                        .iter()
                        .filter(|d| chosen.contains(&d.id))
                        .map(|d| d.cost)
                        .sum::<f64>()
                })
            })
            .fold(f64::INFINITY, f64::min);
        let controls = Controls::default();
        let (tear, report) = tears::solve(&graph, &controls, execution(&controls)).unwrap();
        assert_eq!(report.termination.category, Termination::Success);
        let tear = tear.unwrap();
        near(tear.cost, best, 1e-8);
        assert!(graph.witness(&tear.decisions).is_ok());
        assert_eq!(tear.order.len(), 3);
    }
}
