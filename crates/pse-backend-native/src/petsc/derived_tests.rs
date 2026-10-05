// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded native mechanism controls share the one terminal process lifetime test.
use super::*;
use pse_ids::ContentHash;
use pse_math::{
    derived::{
        Constraint, Coordinate, Correspondence, DerivativeSupport, DerivedFamily, MassBinding,
        MassOracle, MassStructure, OriginalContract, OriginalObligations,
    },
    index::{Addend, Entry, GlobalCol, GlobalRow},
    sparse::AssemblyMatrix,
};
use std::sync::Arc;
fn hash(n: u8) -> ContentHash {
    ContentHash::from_bytes([n; 32])
}
#[derive(Debug)]
struct Root {
    contract: crate::OracleContract,
    matrix: faer::sparse::SparseColMat<usize, f64>,
    calls: usize,
}
impl Root {
    fn new(n: usize) -> Self {
        let contract = crate::OracleContract {
            identity: hash(60),
            variables: (0..n)
                .map(|i| crate::Variable {
                    id: crate::solver_tests::id(i as u8 + 1),
                    lower: f64::NEG_INFINITY,
                    upper: f64::INFINITY,
                })
                .collect(),
            rows: (0..n)
                .map(|i| crate::solver_tests::id(i as u8 + 10))
                .collect(),
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let matrix = faer::sparse::SparseColMat::try_new_from_triplets(
            n,
            n,
            &if n == 1 {
                vec![faer::sparse::Triplet::new(0, 0, 1.0)]
            } else {
                vec![
                    faer::sparse::Triplet::new(0, 0, 2.0),
                    faer::sparse::Triplet::new(0, 1, 0.25),
                    faer::sparse::Triplet::new(1, 0, 0.25),
                    faer::sparse::Triplet::new(1, 1, 2.0),
                ]
            },
        )
        .unwrap();
        Self {
            contract,
            matrix,
            calls: 0,
        }
    }
    fn original(&self) -> Arc<OriginalContract> {
        let mut edges = Vec::new();
        for col in 0..self.matrix.ncols() {
            for row in self.matrix.symbolic().row_idx_of_col(col) {
                edges.push(Entry::new(GlobalRow::new(row), GlobalCol::new(col)));
            }
        }
        Arc::new(
            OriginalContract::new(
                self.contract.identity,
                hash(61),
                self.contract
                    .variables
                    .iter()
                    .map(|v| Coordinate {
                        id: v.id,
                        lower: v.lower,
                        upper: v.upper,
                    })
                    .collect(),
                self.contract
                    .rows
                    .iter()
                    .map(|id| Constraint {
                        id: *id,
                        lower: 0.0,
                        upper: 0.0,
                    })
                    .collect(),
                edges,
                DerivativeSupport {
                    order: pse_kernels::DerivativeOrder::First,
                    jacobian_product: true,
                    source: hash(62),
                },
                OriginalObligations {
                    guards: hash(63),
                    selection: hash(64),
                    objective: None,
                },
            )
            .unwrap(),
        )
    }
}
impl NleOracle for Root {
    fn contract(&self) -> &crate::OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.matrix.symbolic()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.calls += 1;
        if x.len() == 1 {
            out[0] = x[0] - 1.0;
        } else {
            out[0] = 2.0 * x[0] + 0.25 * x[1] - 2.5;
            out[1] = 0.25 * x[0] + 2.0 * x[1] - 4.25;
        }
        Ok(())
    }
    fn jacobian(&mut self, _x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(self.matrix.val());
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        _x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        if direction.len() == 1 {
            out[0] = direction[0];
        } else {
            out[0] = 2.0 * direction[0] + 0.25 * direction[1];
            out[1] = 0.25 * direction[0] + 2.0 * direction[1];
        }
        Ok(())
    }
}

#[derive(Debug)]
struct UnstableRoot {
    base: Root,
}
impl UnstableRoot {
    fn new() -> Self {
        let mut base = Root::new(2);
        base.contract.identity = hash(74);
        base.matrix = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 1.0),
                faer::sparse::Triplet::new(1, 0, 2.0),
                faer::sparse::Triplet::new(0, 1, 2.0),
                faer::sparse::Triplet::new(1, 1, 1.0),
            ],
        )
        .unwrap();
        Self { base }
    }
    fn original(&self) -> Arc<OriginalContract> {
        let inventory = self.base.original();
        Arc::new(
            OriginalContract::new(
                self.base.contract.identity,
                hash(75),
                inventory.coordinates().to_vec(),
                inventory.constraints().to_vec(),
                inventory.incidence().to_vec(),
                DerivativeSupport {
                    source: hash(76),
                    ..inventory.support()
                },
                OriginalObligations {
                    guards: hash(77),
                    selection: hash(78),
                    objective: None,
                },
            )
            .unwrap(),
        )
    }
}
impl NleOracle for UnstableRoot {
    fn contract(&self) -> &crate::OracleContract {
        &self.base.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.base.matrix.symbolic()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.base.calls += 1;
        out[0] = x[0] + 2.0 * x[1];
        out[1] = 2.0 * x[0] + x[1];
        Ok(())
    }
    fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(self.base.matrix.val());
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        _: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out[0] = direction[0] + 2.0 * direction[1];
        out[1] = 2.0 * direction[0] + direction[1];
        Ok(())
    }
}
#[derive(Debug)]
struct Domain {
    source: ContentHash,
    nonpositive: bool,
    calls: usize,
}
impl Domain {
    fn new() -> Self {
        Self {
            source: hash(63),
            nonpositive: false,
            calls: 0,
        }
    }
}
impl DomainOracle for Domain {
    fn source(&self) -> ContentHash {
        self.source
    }
    fn check(&mut self, x: &[f64]) -> Result<(), ProblemError> {
        self.calls += 1;
        if self.nonpositive && x.iter().any(|v| *v > 0.0) {
            Err(pse_math::MathError::Domain {
                source_id: crate::solver_tests::id(1),
                requirement: "nonpositive test domain",
            }
            .into())
        } else {
            Ok(())
        }
    }
}
fn matrix(value: f64) -> AssemblyMatrix {
    let mut matrix = AssemblyMatrix::new(
        1,
        1,
        &[Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
        i32::MAX as usize,
    )
    .unwrap();
    matrix.add(Addend::new(0), value).unwrap();
    matrix
}
#[derive(Debug)]
struct Refresh {
    calls: usize,
}
impl FrozenMassRefresh for Refresh {
    fn source(&self) -> ContentHash {
        hash(65)
    }
    fn realization(&self) -> ContentHash {
        hash(66)
    }
    fn at_anchor(&mut self, point: &[f64]) -> Result<AssemblyMatrix, ProblemError> {
        assert!(point[0] >= 0.0);
        self.calls += 1;
        Ok(matrix(2.0))
    }
}
#[derive(Debug)]
struct VariableMass {
    structure: MassStructure,
    derivatives: Arc<std::sync::atomic::AtomicUsize>,
}
impl MassOracle for VariableMass {
    type Error = ProblemError;
    fn structure(&self) -> &MassStructure {
        &self.structure
    }
    fn realization(&self) -> ContentHash {
        hash(67)
    }
    fn apply(&mut self, x: &[f64], v: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = (1.0 + x[0] * x[0]) * v[0];
        Ok(())
    }
    fn derivative_action(
        &mut self,
        x: &[f64],
        v: &[f64],
        velocity: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.derivatives
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        out[0] = 2.0 * x[0] * v[0] * velocity[0];
        Ok(())
    }
}
fn execution(controls: &Controls) -> Execution {
    let mut e = Execution::new(Arc::default(), controls);
    e.memory = Some(1 << 20);
    e
}
fn tolerances(n: usize) -> Tolerances {
    Tolerances {
        variables: vec![1e-8; n],
        rows: vec![1e-8; n],
        integrality: 1e-8,
    }
}
pub(super) fn exercise_declared_flow_and_blocks() {
    let controls = Controls {
        iterations: 30,
        ..Default::default()
    };
    let accuracy = ResolvedAccuracy {
        native_scaling: false,
        feasibility: 1e-10,
        ..ResolvedAccuracy::verification()
    };
    let compatibility = crate::solver_tests::stamp(Backend::Petsc);
    let settings = Settings {
        method: Method::PseudoTransient,
        pseudo: Some(crate::settings::petsc::PseudoTime {
            initial: 0.1,
            growth: 1.5,
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut root = Root::new(1);
    let original = root.original();
    let incidence = vec![Entry::new(GlobalRow::new(0), GlobalCol::new(0))];
    let family = Arc::new(
        DerivedFamily::shifted_pseudo_time(
            original.clone(),
            vec![GlobalCol::new(0)],
            1.0,
            MassStructure::Frozen {
                incidence: incidence.clone(),
            },
        )
        .unwrap(),
    );
    let mut domain = Domain::new();
    let mut refresh = Refresh { calls: 0 };
    let mut flow = ArtificialFlow::new(
        family.clone(),
        &mut root,
        MassBinding::Frozen(matrix(2.0)),
        &mut domain,
    )
    .unwrap()
    .with_frozen_refresh(&mut refresh)
    .unwrap();
    let report = solve_flow(
        &mut flow,
        &[0.0],
        &settings,
        FlowLimits {
            steps: 100,
            artificial_time: 1e6,
        },
        SolveRequest {
            controls: &controls,
            accuracy: &accuracy,
            execution: execution(&controls),
            tolerances: &tolerances(1),
            warm: None,
            compatibility: &compatibility,
        },
    )
    .unwrap();
    assert_eq!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    assert_eq!(report.termination.name, "TS_CONVERGED_USER");
    assert!(report.quality.as_ref().unwrap().feasible());
    assert!(report.candidate.as_ref().unwrap().primal[0] > 0.99999999);
    assert_eq!(report.provenance["mass.scope"], "accepted-anchor");
    drop(flow);
    assert!(refresh.calls > 0);
    assert_eq!(
        report.metrics["TSGetStepNumber"],
        Metric::Integer(refresh.calls as i64)
    );

    // A concrete state-dependent mass action and DM contribution are both consumed.
    let derivatives = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let structure = MassStructure::StateDependent {
        source: hash(68),
        incidence: incidence.clone(),
        derivative_incidence: incidence.clone(),
    };
    let family = Arc::new(
        DerivedFamily::shifted_pseudo_time(
            original.clone(),
            vec![GlobalCol::new(0)],
            1.0,
            structure.clone(),
        )
        .unwrap(),
    );
    let mut root = Root::new(1);
    let mut domain = Domain::new();
    let mut flow = ArtificialFlow::new(
        family,
        &mut root,
        MassBinding::StateDependent(Box::new(VariableMass {
            structure,
            derivatives: derivatives.clone(),
        })),
        &mut domain,
    )
    .unwrap();
    flow::assert_actual_mass_action(&mut flow, &execution(&controls));
    let report = solve_flow(
        &mut flow,
        &[0.0],
        &settings,
        FlowLimits {
            steps: 100,
            artificial_time: 1e6,
        },
        SolveRequest {
            controls: &controls,
            accuracy: &accuracy,
            execution: execution(&controls),
            tolerances: &tolerances(1),
            warm: None,
            compatibility: &compatibility,
        },
    )
    .unwrap();
    assert_eq!(report.termination.category, Termination::Success);
    assert!(report.quality.unwrap().feasible());
    assert!(derivatives.load(std::sync::atomic::Ordering::Relaxed) > 0);
    drop(flow);

    // Failed stages can shrink below TSAdapt's declared floor; the public pre-stage
    // checkpoint stops before another inner solve. Refresh is never called on rejection.
    let family = Arc::new(
        DerivedFamily::shifted_pseudo_time(
            original.clone(),
            vec![GlobalCol::new(0)],
            1.0,
            MassStructure::Frozen {
                incidence: incidence.clone(),
            },
        )
        .unwrap(),
    );
    let mut root = Root::new(1);
    let mut domain = Domain {
        nonpositive: true,
        ..Domain::new()
    };
    let mut refresh = Refresh { calls: 0 };
    let mut flow = ArtificialFlow::new(
        family.clone(),
        &mut root,
        MassBinding::Frozen(matrix(2.0)),
        &mut domain,
    )
    .unwrap()
    .with_frozen_refresh(&mut refresh)
    .unwrap();
    let settings = Settings {
        pseudo: Some(crate::settings::petsc::PseudoTime {
            initial: 0.01,
            minimum: 0.01,
            growth: 1.1,
            ..Default::default()
        }),
        ..settings
    };
    let limited = Controls {
        iterations: 2,
        ..controls.clone()
    };
    let report = solve_flow(
        &mut flow,
        &[0.0],
        &settings,
        FlowLimits {
            steps: 4,
            artificial_time: 1.0,
        },
        SolveRequest {
            controls: &limited,
            accuracy: &accuracy,
            execution: execution(&limited),
            tolerances: &tolerances(1),
            warm: None,
            compatibility: &compatibility,
        },
    )
    .unwrap();
    assert_eq!(
        report.termination.category,
        Termination::Limit,
        "{report:?}"
    );
    assert_eq!(report.termination.name, "PETSC_ERR_USER");
    assert!(matches!(
        report.callback_failure(),
        Some(ProblemError::Limit {
            kind: crate::LimitKind::Work,
            ..
        })
    ));
    assert!(report.quality.is_none() && report.warm_start.is_none());
    drop(flow);
    assert_eq!(refresh.calls, 0);

    let mut root = Root::new(1);
    let mut domain = Domain::new();
    let mut flow = ArtificialFlow::new(
        family,
        &mut root,
        MassBinding::Frozen(matrix(2.0)),
        &mut domain,
    )
    .unwrap();
    let mut e = execution(&controls);
    e.memory = Some(1);
    assert!(matches!(
        solve_flow(
            &mut flow,
            &[0.0],
            &settings,
            FlowLimits {
                steps: 2,
                artificial_time: 1.0
            },
            SolveRequest {
                controls: &controls,
                accuracy: &accuracy,
                execution: e,
                tolerances: &tolerances(1),
                warm: None,
                compatibility: &compatibility
            },
        ),
        Err(ProblemError::Limit {
            kind: crate::LimitKind::Memory,
            ..
        })
    ));
    drop(flow);
    assert_eq!(
        root.calls, 0,
        "shape rejection precedes oracle/native allocations"
    );

    exercise_unstable_flow(&controls, &accuracy, &compatibility);
    exercise_blocks(&controls, &accuracy, &compatibility);
}

fn exercise_unstable_flow(
    controls: &Controls,
    accuracy: &ResolvedAccuracy,
    compatibility: &Compatibility,
) {
    let mut root = UnstableRoot::new();
    let original = root.original();
    let incidence = vec![
        Entry::new(GlobalRow::new(0), GlobalCol::new(0)),
        Entry::new(GlobalRow::new(1), GlobalCol::new(1)),
    ];
    let family = Arc::new(
        DerivedFamily::shifted_pseudo_time(
            original.clone(),
            vec![GlobalCol::new(0), GlobalCol::new(1)],
            1.0,
            MassStructure::Frozen {
                incidence: incidence.clone(),
            },
        )
        .unwrap(),
    );
    assert_eq!(family.correspondence(), Correspondence::Approximate);
    let mut mass = AssemblyMatrix::new(2, 2, &incidence, i32::MAX as usize).unwrap();
    mass.add(Addend::new(0), 1.0).unwrap();
    mass.add(Addend::new(1), 1.0).unwrap();
    let mut domain = Domain {
        source: original.obligations().guards,
        ..Domain::new()
    };
    let mut flow = ArtificialFlow::new(
        family.clone(),
        &mut root,
        MassBinding::Frozen(mass),
        &mut domain,
    )
    .unwrap();
    let binding = flow.binding_key();
    let settings = Settings {
        method: Method::PseudoTransient,
        pseudo: Some(crate::settings::petsc::PseudoTime {
            initial: 0.1,
            maximum: 0.2,
            ..Default::default()
        }),
        ..Default::default()
    };
    // J[1,-1] = -[1,-1], so M=I and sign +1 make the artificial flow
    // unstable about the original root zero. Small finite steps cannot grant a root.
    let report = solve_flow(
        &mut flow,
        &[0.1, -0.1],
        &settings,
        FlowLimits {
            steps: 2,
            artificial_time: 1.0,
        },
        SolveRequest {
            controls,
            accuracy,
            execution: execution(controls),
            tolerances: &tolerances(2),
            warm: None,
            compatibility,
        },
    )
    .unwrap();
    assert!(matches!(report.metrics["TSGetStepNumber"], Metric::Integer(n) if n > 0 && n <= 2));
    assert!(report.evidence.work.evaluations.is_some_and(|n| n > 0));
    assert_ne!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    assert_eq!(report.termination.assurance, Assurance::None);
    assert_eq!(report.qualification, Qualification::Unqualified);
    assert!(!report.quality.as_ref().unwrap().feasible());
    assert_eq!(report.provenance["family"], family.key().to_hex());
    assert_eq!(
        report.provenance["normalization"],
        original.normalization().to_hex()
    );
    assert_eq!(
        report.provenance["guards"],
        original.obligations().guards.to_hex()
    );
    assert_eq!(report.provenance["mass.scope"], "fixed-trajectory");
    assert_eq!(report.provenance["mass.initial_binding"], binding.to_hex());
    assert_eq!(report.provenance["mass.final_binding"], binding.to_hex());
    drop(flow);
    assert!(root.base.calls > 0 && domain.calls > 0);
}
#[derive(Debug)]
struct Local {
    family: Arc<DerivedFamily>,
    index: usize,
    calls: usize,
    fail: bool,
    ghost_observed: bool,
}
impl BlockOracle for Local {
    fn family(&self) -> &Arc<DerivedFamily> {
        &self.family
    }
    fn realization(&self) -> ContentHash {
        hash(70)
    }
    fn residual(
        &mut self,
        local: &[f64],
        ghosts: GhostCoordinates<'_>,
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.calls += 1;
        assert_eq!(local.len(), 1);
        assert_eq!(ghosts.columns.len(), 2);
        assert_eq!(ghosts.values[self.index], local[0]);
        self.ghost_observed = true;
        if self.fail {
            out[0] = 100.0;
            return Err(pse_math::MathError::Domain {
                source_id: crate::solver_tests::id(self.index as u8 + 1),
                requirement: "declared block domain contradiction",
            }
            .into());
        }
        let rhs = if self.index == 0 { 2.5 } else { 4.25 };
        out[0] = 2.0 * local[0] + 0.25 * ghosts.values[1 - self.index] - rhs;
        Ok(())
    }
    fn jacobian(
        &mut self,
        _local: &[f64],
        _ghosts: GhostCoordinates<'_>,
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out[0] = 2.0;
        Ok(())
    }
}
fn exercise_blocks(
    controls: &Controls,
    accuracy: &ResolvedAccuracy,
    compatibility: &Compatibility,
) {
    let mut root = Root::new(2);
    let original = root.original();
    let a = Arc::new(
        DerivedFamily::block_subsystem(
            original.clone(),
            vec![GlobalCol::new(0)],
            vec![GlobalRow::new(0)],
        )
        .unwrap(),
    );
    let b = Arc::new(
        DerivedFamily::block_subsystem(original, vec![GlobalCol::new(1)], vec![GlobalRow::new(1)])
            .unwrap(),
    );
    let mut first = Local {
        family: a.clone(),
        index: 0,
        calls: 0,
        fail: false,
        ghost_observed: false,
    };
    let mut second = Local {
        family: b.clone(),
        index: 1,
        calls: 0,
        fail: false,
        ghost_observed: false,
    };
    let mut domain = Domain::new();
    let witness = DomainWitness {
        source: domain.source(),
        declaration: hash(69),
    };
    assert!(matches!(
        BlockSpec::new(
            &mut first,
            vec![GlobalCol::new(0)],
            DomainWitness {
                source: hash(90),
                ..witness
            }
        ),
        Err(ProblemError::Contract(_))
    ));
    assert!(matches!(
        BlockComposition::new(
            &mut root,
            &mut domain,
            vec![BlockSpec::new(&mut first, vec![GlobalCol::new(0)], witness).unwrap()]
        ),
        Err(ProblemError::Contract(_))
    ));
    assert_eq!(
        root.calls, 0,
        "refused incomplete composition has no native/oracle trajectory"
    );
    let mut composition = BlockComposition::new(
        &mut root,
        &mut domain,
        vec![
            BlockSpec::new(&mut first, vec![GlobalCol::new(0)], witness).unwrap(),
            BlockSpec::new(&mut second, vec![GlobalCol::new(1)], witness).unwrap(),
        ],
    )
    .unwrap();
    let settings = Settings {
        method: Method::NonlinearAdditiveSchwarz,
        schwarz: Some(Default::default()),
        ..Default::default()
    };
    let report = solve_blocks(
        &mut composition,
        &[0.0, 0.0],
        &settings,
        SolveRequest {
            controls,
            accuracy,
            execution: execution(controls),
            tolerances: &tolerances(2),
            warm: None,
            compatibility,
        },
    )
    .unwrap();
    assert_eq!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    assert!(report.quality.as_ref().unwrap().feasible());
    let point = &report.candidate.as_ref().unwrap().primal;
    assert!((point[0] - 1.0).abs() < 1e-8 && (point[1] - 2.0).abs() < 1e-8);
    assert!(report.evidence.work.iterations.is_some_and(|n| n > 0));
    drop(composition);
    assert!(first.ghost_observed && second.ghost_observed);
    for failed_index in 0..2 {
        let mut root = Root::new(2);
        let mut domain = Domain::new();
        let mut first = Local {
            family: a.clone(),
            index: 0,
            calls: 0,
            fail: failed_index == 0,
            ghost_observed: false,
        };
        let mut second = Local {
            family: b.clone(),
            index: 1,
            calls: 0,
            fail: failed_index == 1,
            ghost_observed: false,
        };
        let mut composition = BlockComposition::new(
            &mut root,
            &mut domain,
            vec![
                BlockSpec::new(&mut first, vec![GlobalCol::new(0)], witness).unwrap(),
                BlockSpec::new(&mut second, vec![GlobalCol::new(1)], witness).unwrap(),
            ],
        )
        .unwrap();
        let report = solve_blocks(
            &mut composition,
            &[0.0, 0.0],
            &settings,
            SolveRequest {
                controls,
                accuracy,
                execution: execution(controls),
                tolerances: &tolerances(2),
                warm: None,
                compatibility,
            },
        )
        .unwrap();
        assert_eq!(report.termination.category, Termination::Evaluation);
        assert!(report.evidence.callback.terminal_failure);
        assert_eq!(report.termination.name, "PETSC_ERR_USER");
        assert!(matches!(
            report.callback_failure(),
            Some(ProblemError::Math(pse_math::MathError::Domain {
                requirement: "declared block domain contradiction",
                ..
            }))
        ));
        assert!(report.quality.is_none() && report.warm_start.is_none());
        drop(composition);
        if failed_index == 0 {
            assert_eq!(first.calls, 1);
            assert_eq!(
                second.calls, 0,
                "terminal block latch prevents other callbacks"
            );
        } else {
            assert_eq!(second.calls, 1);
        }
        assert_eq!(
            root.calls, 2,
            "no final original validation after terminal block latch"
        );
    }
    exercise_guard_only_external_slots(controls, accuracy, compatibility);
}

#[derive(Debug)]
struct GuardRoot {
    base: Root,
}
impl GuardRoot {
    fn new() -> Self {
        let mut base = Root::new(2);
        base.matrix = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 1.0),
                faer::sparse::Triplet::new(1, 1, 1.0),
            ],
        )
        .unwrap();
        Self { base }
    }
}
impl NleOracle for GuardRoot {
    fn contract(&self) -> &crate::OracleContract {
        &self.base.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.base.matrix.symbolic()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if x[1] <= 0.0 {
            return Err(pse_math::MathError::Domain {
                source_id: crate::solver_tests::id(2),
                requirement: "guard-only external must be positive",
            }
            .into());
        }
        out.copy_from_slice(&[x[0] - 1.0, x[1] - 2.0]);
        Ok(())
    }
    fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(self.base.matrix.val());
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        _: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out.copy_from_slice(direction);
        Ok(())
    }
}
#[derive(Debug)]
struct GuardLocal {
    family: Arc<DerivedFamily>,
    index: usize,
    extras: Vec<GlobalCol>,
    observed: Vec<f64>,
}
impl BlockOracle for GuardLocal {
    fn family(&self) -> &Arc<DerivedFamily> {
        &self.family
    }
    fn realization(&self) -> ContentHash {
        hash(72)
    }
    fn external_coordinates(&self) -> Vec<GlobalCol> {
        self.extras.clone()
    }
    fn residual(
        &mut self,
        local: &[f64],
        ghosts: GhostCoordinates<'_>,
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        if self.index == 0 {
            let slot = ghosts.columns.binary_search(&GlobalCol::new(1)).unwrap();
            let value = ghosts.values[slot];
            if value <= 0.0 {
                return Err(pse_math::MathError::Domain {
                    source_id: crate::solver_tests::id(2),
                    requirement: "guard-only external must be positive",
                }
                .into());
            }
            self.observed.push(value);
        }
        out[0] = local[0] - (self.index as f64 + 1.0);
        Ok(())
    }
    fn jacobian(
        &mut self,
        _: &[f64],
        _: GhostCoordinates<'_>,
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out[0] = 1.0;
        Ok(())
    }
}
fn exercise_guard_only_external_slots(
    controls: &Controls,
    accuracy: &ResolvedAccuracy,
    compatibility: &Compatibility,
) {
    let mut root = GuardRoot::new();
    let original = root.base.original();
    let a = Arc::new(
        DerivedFamily::block_subsystem(
            original.clone(),
            vec![GlobalCol::new(0)],
            vec![GlobalRow::new(0)],
        )
        .unwrap(),
    );
    let b = Arc::new(
        DerivedFamily::block_subsystem(original, vec![GlobalCol::new(1)], vec![GlobalRow::new(1)])
            .unwrap(),
    );
    assert!(
        a.external_incidence().is_empty(),
        "guard input is independent of equation derivative incidence"
    );
    let mut first = GuardLocal {
        family: a,
        index: 0,
        extras: vec![GlobalCol::new(2)],
        observed: Vec::new(),
    };
    let mut second = GuardLocal {
        family: b,
        index: 1,
        extras: Vec::new(),
        observed: Vec::new(),
    };
    let mut domain = Domain::new();
    let witness = DomainWitness {
        source: domain.source(),
        declaration: hash(73),
    };
    assert!(matches!(
        BlockSpec::new(&mut first, vec![GlobalCol::new(0)], witness),
        Err(ProblemError::Contract(_))
    ));
    first.extras = vec![GlobalCol::new(1), GlobalCol::new(1)];
    assert!(BlockSpec::new(&mut first, vec![GlobalCol::new(0)], witness).is_err());
    first.extras = vec![GlobalCol::new(1)];
    let mut composition = BlockComposition::new(
        &mut root,
        &mut domain,
        vec![
            BlockSpec::new(&mut first, vec![GlobalCol::new(0)], witness).unwrap(),
            BlockSpec::new(&mut second, vec![GlobalCol::new(1)], witness).unwrap(),
        ],
    )
    .unwrap();
    let settings = Settings {
        method: Method::NonlinearAdditiveSchwarz,
        schwarz: Some(crate::settings::petsc::Schwarz {
            damping: 0.75,
            ..Default::default()
        }),
        ..Default::default()
    };
    let report = solve_blocks(
        &mut composition,
        &[0.0, 1.0],
        &settings,
        SolveRequest {
            controls,
            accuracy,
            execution: execution(controls),
            tolerances: &tolerances(2),
            warm: None,
            compatibility,
        },
    )
    .unwrap();
    // The finite damped attempt can stop after child tolerances have been met;
    // its actual limited status does not grant auxiliary or original permission.
    assert_eq!(
        report.termination.category,
        Termination::IterationLimit,
        "{report:?}"
    );
    assert!(!report.evidence.callback.terminal_failure);
    assert!(
        report
            .candidate
            .as_ref()
            .unwrap()
            .primal
            .iter()
            .zip([1.0, 2.0])
            .all(|(actual, expected)| (actual - expected).abs() < 1e-8)
    );
    drop(composition);
    assert!(first.observed.contains(&1.0));
    assert!(
        first.observed.iter().any(|v| (*v - 2.0).abs() < 1e-8),
        "actual guard scatter must refresh the outer coordinate"
    );
}
