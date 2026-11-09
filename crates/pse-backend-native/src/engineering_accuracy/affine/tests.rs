// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    quality::Tolerances,
    solve::Controls,
    square_response::{SquareScope, point_key},
};
use pse_ids::SemanticId;
use pse_kernels::{DerivativeOrder, Port};
use pse_math::{
    assembly::AssemblyLimits,
    binding::{CaseLimits, CaseStructure, Contribution, InstanceBinding, Row, SlotBinding, Target},
    normalization::Normalization,
    typed::{BodyBuilder, BodyLimits},
};
use pse_model::{
    engineering_accuracy::{AccuracyGoal, BoundGoal},
    generated::enums::{
        AccuracyGoalSubject, AccuracyGoalUse, AccuracyObservation, NumericalSource, NumericalTarget,
    },
    strategy::{AccuracyClass, SemanticProductKey},
};
use pse_quantity::{
    IndexSet,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::{collections::BTreeMap, sync::Arc};

fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn hash(n: u8) -> ContentHash {
    ContentHash::from_bytes([n; 32])
}
fn row(entries: &[(usize, f64)]) -> Option<AffineRow> {
    Some(AffineRow {
        entries: entries.iter().copied().collect(),
        constant: 0.0,
    })
}
fn anchor(rows: &[Option<AffineRow>], n: usize) -> Vec<bool> {
    anchored(rows, n, &execution()).unwrap()
}

#[test]
fn affine_accuracy_provenance_requires_unary_anchor_and_keeps_moving_dependents() {
    assert_eq!(
        anchor(
            &[
                row(&[(0, 1.)]),
                row(&[(0, -2.), (1, 1.)]),
                row(&[(1, -3.), (2, 1.)])
            ],
            4
        ),
        [true, true, true, false]
    );
    assert_eq!(
        anchor(&[row(&[(0, 1.), (1, -1.)]), row(&[(0, -1.), (1, 1.)])], 2),
        [false, false]
    );
    assert_eq!(
        anchor(&[row(&[(0, 0.)]), row(&[(0, -1.), (1, 1.)])], 2),
        [false, false]
    );
    // A source row with three dependent coordinates is outside this narrow
    // closure, even when later numerical elimination can cancel a term.
    assert_eq!(
        anchor(&[row(&[(0, 1.)]), row(&[(0, 1.), (1, 1.), (2, -1.)])], 3),
        [true, false, false]
    );
}

struct Fixture {
    plan: CasePlan,
    values: CaseValues,
    facts: Facts,
    square: SquareScope,
    point: Vec<f64>,
    observed: Vec<f64>,
    normalization: Normalization,
    tolerances: Tolerances,
    key: SemanticProductKey,
}
#[derive(Clone, Copy)]
enum Source {
    Anchored,
    UnanchoredCycle,
    BoundsOnly,
    FirstGuard,
}
impl Fixture {
    fn new() -> Self {
        Self::source(Source::Anchored)
    }
    fn source(source: Source) -> Self {
        use pse_math::typed::Binary;
        let registry = standard_registry().unwrap();
        let quantity = ids::quantity("neutral");
        let ports = (1..=4)
            .map(|n| Port {
                id: id(n),
                quantity,
                unit: registry.quantity_type(quantity).unwrap().canonical_unit,
            })
            .collect::<Vec<_>>();
        let mut builder = BodyBuilder::new(
            pse_math::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            4,
            BodyLimits::default(),
        )
        .unwrap();
        let inputs = (0..4)
            .map(|i| builder.input(i, quantity, IndexSet::new(), id(20)).unwrap())
            .collect::<Vec<_>>();
        let chain = builder
            .binary(
                Binary::Sub,
                inputs[1].clone(),
                inputs[0].clone(),
                None,
                id(21),
            )
            .unwrap();
        let first = match source {
            Source::UnanchoredCycle => builder
                .binary(
                    Binary::Sub,
                    inputs[0].clone(),
                    inputs[1].clone(),
                    None,
                    id(25),
                )
                .unwrap(),
            Source::BoundsOnly => builder
                .binary(
                    Binary::Mul,
                    inputs[0].clone(),
                    inputs[0].clone(),
                    None,
                    id(25),
                )
                .unwrap(),
            Source::Anchored | Source::FirstGuard => inputs[0].clone(),
        };
        let yy = builder
            .binary(
                Binary::Mul,
                inputs[2].clone(),
                inputs[2].clone(),
                None,
                id(22),
            )
            .unwrap();
        let dependence = if matches!(source, Source::FirstGuard) {
            let sqrt = builder
                .unary(pse_math::Function::Sqrt, inputs[0].clone(), id(26))
                .unwrap();
            builder.bind(sqrt).unwrap()
        } else {
            inputs[0].clone()
        };
        let nonlinear = builder
            .binary(Binary::Add, yy, dependence, None, id(23))
            .unwrap();
        let nonlinear = builder
            .binary(Binary::Sub, nonlinear, inputs[3].clone(), None, id(24))
            .unwrap();
        let body = Arc::new(builder.prepare(&[first, chain, nonlinear]).unwrap());
        let structure = Arc::new(
            CaseStructure::new(
                ports[..3]
                    .iter()
                    .map(|port| pse_math::binding::Variable {
                        port: port.clone(),
                        fixed: false,
                        domain: pse_model::generated::enums::ModelingVariableDomain::Continuous,
                        lower: Some(0.),
                        upper: Some(
                            if matches!(source, Source::BoundsOnly) && port.id == id(1) {
                                0.
                            } else {
                                10.
                            },
                        ),
                    })
                    .collect(),
                vec![ports[3].clone()],
                vec![InstanceBinding {
                    occurrences: Default::default(),
                    checked_sources: Default::default(),
                    checked_members: Default::default(),
                    instance: id(30),
                    body: hash(30),
                    slots: ports
                        .iter()
                        .map(|port| SlotBinding::new(port, port, &registry).unwrap())
                        .collect::<Vec<_>>()
                        .into(),
                    contributions: (0..3)
                        .map(|r| Contribution {
                            output: r,
                            target: Target::Row(id(101 + u8::try_from(r).unwrap())),
                            scale: 1.0,
                        })
                        .collect(),
                }],
                (0..3)
                    .map(|r| Row {
                        id: id(101 + r),
                        quantity,
                        lower: 0.,
                        upper: 0.,
                    })
                    .collect(),
                None,
                CaseLimits::default(),
            )
            .unwrap(),
        );
        let cancel = pse_diagnostics::Cancellation::default();
        let plan = CasePlan::prepare(
            structure,
            BTreeMap::from([(hash(30), body)]),
            &registry,
            DerivativeOrder::First,
            AssemblyLimits::default(),
            &cancel.flag(),
        )
        .unwrap();
        assert_eq!(plan.columns(), &[id(1), id(2), id(3)]);
        let point = if matches!(source, Source::Anchored) {
            vec![0.001, 0.001, 2.0]
        } else {
            vec![0., 0., 2.]
        };
        let values = CaseValues {
            scalars: BTreeMap::from([
                (id(1), point[0]),
                (id(2), point[1]),
                (id(3), point[2]),
                (id(4), 4.0),
            ]),
        };
        let base = plan
            .presolve_domain_facts(&values, 1 << 24, &cancel.flag())
            .unwrap();
        let facts = plan
            .affine_row_facts(&values, &base, 1 << 24, &cancel.flag())
            .unwrap();
        assert_eq!(
            facts.affine[0].is_some(),
            !matches!(source, Source::BoundsOnly)
        );
        assert!(facts.affine[1].is_some());
        assert!(facts.affine[2].is_none());
        let square = SquareScope::admit(
            &crate::assembled::contract(&plan),
            plan.jacobian_pattern(),
            &[(0., 0.); 3],
            None,
        )
        .unwrap();
        let normalization = Normalization::identity(3, 3);
        let key = SemanticProductKey {
            structure: plan.structure().key(),
            binding: hash(1),
            numerical_policy: Some(hash(2)),
            normalization: Some(normalization.key()),
            point: Some(point_key(&point)),
            parameters: Some(hash(3)),
            derivation: None,
            branch: Some(hash(4)),
            accuracy: Some(hash(5)),
        };
        let observed = if matches!(source, Source::Anchored) {
            vec![0.001, 0., 0.001]
        } else {
            vec![0.; 3]
        };
        Self {
            plan,
            values,
            facts,
            square,
            point,
            observed,
            normalization,
            tolerances: Tolerances {
                variables: vec![0.01, 0.01, 1e-8],
                rows: vec![0.01; 3],
                integrality: 1e-8,
            },
            key,
        }
    }
    fn request(&self) -> SparseRequest<'_> {
        SparseRequest {
            scope: &self.square,
            point: &self.point,
            values: &self.observed,
            tolerances: &self.tolerances,
            normalization: &self.normalization,
            key: self.key,
            bytes: 1 << 20,
        }
    }
    fn frame(&self, execution: &Execution) -> Result<SquareAccuracyFrame<'_>, Withheld> {
        SquareAccuracyFrame::admit(
            self.request(),
            &self.plan,
            &self.values,
            &self.facts,
            execution,
        )
    }
    fn at_point(point: [f64; 3]) -> Self {
        let mut fixture = Self::new();
        fixture.point = point.to_vec();
        for (column, value) in fixture.plan.columns().iter().zip(point) {
            fixture.values.scalars.insert(*column, value);
        }
        let fixed = point[2] * point[2];
        fixture.values.scalars.insert(id(4), fixed);
        fixture.observed = vec![point[0], point[1] - point[0], (fixed + point[0]) - fixed];
        let cancel = pse_diagnostics::Cancellation::default();
        let base = fixture
            .plan
            .presolve_domain_facts(&fixture.values, 1 << 24, &cancel.flag())
            .unwrap();
        fixture.facts = fixture
            .plan
            .affine_row_facts(&fixture.values, &base, 1 << 24, &cancel.flag())
            .unwrap();
        fixture.key.point = Some(point_key(&fixture.point));
        fixture
    }
    fn jacobian(&self) -> SparseColMat<usize, f64> {
        let columns = [[1., -1., 1.], [0., 1., 0.], [0., 0., 2. * self.point[2]]];
        let pattern = self.plan.jacobian_pattern();
        let mut values = Vec::new();
        for (c, column) in columns.iter().enumerate() {
            for r in pattern.row_idx_of_col(c) {
                values.push(column[r]);
            }
        }
        SparseColMat::new(pattern.to_owned().unwrap(), values)
    }
    fn goal(&self) -> BoundGoal {
        BoundGoal {
            declaration: AccuracyGoal {
                goal_id: id(50).into(),
                model_id: None,
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: id(51),
                target_kind: NumericalTarget::Observable,
                quantity_id: id(52),
                unit_id: id(53),
                subject: AccuracyGoalSubject::SelectedOutput,
                observation: AccuracyObservation::Steady,
                time: None,
                resolution: Some(0.0001),
                criterion_lower: None,
                criterion_upper: None,
                required_class: AccuracyClass::Estimated,
                use_policy: AccuracyGoalUse::Assess,
                refine: true,
                source: NumericalSource::Analysis,
                priority: 0,
                provenance: "q=z+y, x=0, z=x, y*y+x=4 original-equation oracle".into(),
            },
            source: self.key,
            product: hash(50),
            normalization: hash(51),
        }
    }
}
fn execution() -> Execution {
    Execution::new(
        pse_diagnostics::Cancellation::default(),
        &Controls::default(),
    )
}

#[test]
fn affine_accuracy_boundary_chain_keeps_original_defect_uncertainty_and_row_demands() {
    let f = Fixture::new();
    let execution = execution();
    assert!(matches!(
        SparseFactor::check(&f.request()),
        Err(Withheld::Neighborhood(_))
    ));
    let factor = f
        .frame(&execution)
        .unwrap()
        .prepare(|| Ok(f.jacobian()), &execution)
        .unwrap();
    let goal = f.goal();
    let mut output = SquareOutput {
        goal: &goal,
        product: goal.product,
        normalization: goal.normalization,
        derivative_source: f.key,
        value: 2.001,
        gradient: &[0., 1., 1.],
        gradient_uncertainty: Some(&[0., 0., 0.]),
        uncertainty: Some(0.),
    };
    let validity = SquareValidity {
        source: f.key,
        witness: hash(60),
    };
    let residual = SquareResidual {
        source: f.key,
        values: &f.observed,
        residual_uncertainty: Some(&[0.; 3]),
        jacobian_uncertainty: Some(&[0.; 9]),
    };
    let estimate = factor.estimate(residual, validity, &[output], &execution);
    let correction = estimate.correction.as_ref().unwrap();
    assert!((correction[0] + 0.001).abs() < 1e-15);
    assert!((correction[1] + 0.001).abs() < 1e-15);
    assert!(correction[2].abs() < 1e-15);
    let evidence = estimate.outputs[0].as_ref().unwrap();
    assert_eq!(evidence.accuracy.class, AccuracyClass::Estimated);
    assert!((evidence.accuracy.error.unwrap() - 0.001).abs() < 1e-12);
    let demand = estimate.work_demands[0].as_ref().unwrap();
    // The original anchor and nonlinear rows have nonzero defects. The
    // exactly satisfied copy row contributes no row-work demand, even though
    // it remains in the factor and its adjoint has a nonzero multiplier.
    assert_eq!(
        demand.rows.iter().map(|row| row.row).collect::<Vec<_>>(),
        [id(101), id(103)]
    );
    // Dropping the original anchor defect would incorrectly move the nonlinear
    // coordinate: -J^-1 (0,0,.001) has dy=-.00025 rather than zero.
    let without_anchor = factor.estimate(
        SquareResidual {
            values: &[0., 0., 0.001],
            ..residual
        },
        validity,
        &[output],
        &execution,
    );
    assert!((without_anchor.correction.unwrap()[2] + 0.00025).abs() < 1e-15);
    let uncertain = factor.estimate(
        SquareResidual {
            residual_uncertainty: Some(&[1e-5, 0., 0.]),
            ..residual
        },
        validity,
        &[output],
        &execution,
    );
    assert!(
        uncertain.outputs[0]
            .as_ref()
            .unwrap()
            .accuracy
            .error
            .unwrap()
            > evidence.accuracy.error.unwrap() + 7e-6
    );
    assert!(
        uncertain.work_demands[0].as_ref().unwrap().rows[0].physical_allowance
            < demand.rows[0].physical_allowance
    );
    output.gradient_uncertainty = None;
    assert!(matches!(
        factor
            .estimate(residual, validity, &[output], &execution)
            .outputs[0],
        Err(pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty)
    ));
    output.gradient_uncertainty = Some(&[0.; 3]);
    let missing = factor.estimate(
        SquareResidual {
            jacobian_uncertainty: None,
            ..residual
        },
        validity,
        &[output],
        &execution,
    );
    assert!(matches!(
        missing.outputs[0],
        Err(pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty)
    ));
    assert!(missing.action.is_none());
    let stale = factor.estimate(
        SquareResidual {
            source: SemanticProductKey {
                point: Some(hash(99)),
                ..f.key
            },
            ..residual
        },
        validity,
        &[output],
        &execution,
    );
    assert!(matches!(
        stale.outputs[0],
        Err(pse_model::generated::enums::AccuracyUnavailableReason::InvalidValidity)
    ));
    assert!(stale.action.is_none());
}

#[test]
fn affine_accuracy_equal_bounds_do_not_seed_library_constant_provenance() {
    let f = Fixture::source(Source::BoundsOnly);
    assert!(f.facts.matches(&f.plan, &f.values));
    let original = crate::assembled::contract(&f.plan);
    assert_eq!(
        (original.variables[0].lower, original.variables[0].upper),
        (0., 0.)
    );
    assert!(f.facts.affine[0].is_none()); // x*x=0 is not a unary affine anchor.
    let evaluations = std::cell::Cell::new(0);
    let result = prepare_counted(&f, f.request(), &execution(), &evaluations);
    assert!(matches!(result, Err(Withheld::Neighborhood(_))));
    assert_eq!(evaluations.get(), 0);
}

#[test]
fn affine_accuracy_full_frame_refuses_unanchored_source_cycle_before_evaluation() {
    let f = Fixture::source(Source::UnanchoredCycle);
    assert!(f.facts.matches(&f.plan, &f.values));
    assert_eq!(f.facts.affine[0].as_ref().unwrap().entries.len(), 2);
    assert_eq!(f.facts.affine[1].as_ref().unwrap().entries.len(), 2);
    let evaluations = std::cell::Cell::new(0);
    let result = prepare_counted(&f, f.request(), &execution(), &evaluations);
    assert!(matches!(result, Err(Withheld::Neighborhood(_))));
    assert_eq!(evaluations.get(), 0);
}

#[test]
fn affine_accuracy_refuses_stale_consumed_values_and_point() {
    let mut f = Fixture::new();
    let execution = execution();
    f.values.scalars.insert(id(4), 5.0);
    assert!(matches!(f.frame(&execution), Err(Withheld::Structural(_))));
    f.values.scalars.insert(id(4), 4.0);
    f.values.scalars.insert(id(1), 0.002);
    assert!(matches!(f.frame(&execution), Err(Withheld::Structural(_))));
    f.values.scalars.insert(id(1), 0.001);
    f.key.point = Some(hash(99));
    assert!(matches!(f.frame(&execution), Err(Withheld::Numerical(_))));
}

#[test]
fn affine_accuracy_thin_positive_interior_is_private_and_independent_of_physical_tolerance() {
    let mut f = Fixture::at_point([0.001, 0.001, 1e-10]);
    // The anchored coordinates satisfy even the public margin requirement.
    // Only the genuinely moving coordinate has a smaller positive margin.
    f.tolerances.variables = vec![1e-4, 1e-4, 1e-12];
    assert!(SparseFactor::check(&f.request()).is_ok());
    f.tolerances.variables[2] = 1e-8;
    assert!(matches!(
        SparseFactor::check(&f.request()),
        Err(Withheld::Neighborhood(_))
    ));
    let evaluations = std::cell::Cell::new(0);
    let factor = prepare_counted(&f, f.request(), &execution(), &evaluations);
    assert!(
        factor.is_ok(),
        "strict positive interior must admit the private factor: {factor:?}"
    );
    assert_eq!(evaluations.get(), 1);
    // Increasing the physical budget also leaves private pointwise eligibility
    // intact; it does not relax the unchanged public margin admission.
    f.tolerances.variables[2] = 3.0;
    assert!(f.frame(&execution()).is_ok());
    assert!(matches!(
        SparseFactor::check(&f.request()),
        Err(Withheld::Neighborhood(_))
    ));
}

#[test]
fn affine_accuracy_moving_bounds_and_tolerance_accepted_outside_refuse_before_evaluation() {
    for value in [0., 10., -1e-10, 10. + 1e-10] {
        let f = Fixture::at_point([0.001, 0.001, value]);
        let evaluations = std::cell::Cell::new(0);
        let result = prepare_counted(&f, f.request(), &execution(), &evaluations);
        let Err(Withheld::Neighborhood(detail)) = result else {
            panic!("moving coordinate {value:?} must lack strict interior: {result:?}");
        };
        assert!(detail.contains(&format!("coordinate={}", id(3))));
        for field in [
            "value=",
            "lower=",
            "upper=",
            "tolerance=",
            "lower_margin=",
            "upper_margin=",
        ] {
            assert!(
                detail.contains(field),
                "missing original bound detail {field}: {detail}"
            );
        }
        assert_eq!(evaluations.get(), 0);
    }
}

#[test]
fn affine_accuracy_original_physical_infeasibility_is_separate_and_includes_constants() {
    // Source-anchored constants can lie just outside the bound within their
    // physical tolerance; genuinely moving coordinates cannot.
    assert!(
        Fixture::at_point([-0.001, -0.001, 2.])
            .frame(&execution())
            .is_ok()
    );
    for point in [[-0.02, -0.02, 2.], [0.001, 0.001, -1e-4]] {
        let mut f = Fixture::at_point(point);
        // Keep row acceptance independent of the coordinate-bound failure.
        f.tolerances.rows = vec![0.1; 3];
        let evaluations = std::cell::Cell::new(0);
        let result = prepare_counted(&f, f.request(), &execution(), &evaluations);
        let Err(Withheld::Infeasible(detail)) = result else {
            panic!("original physical infeasibility must win: {result:?}");
        };
        let offenders = if point[0] < 0. {
            vec![id(1), id(2)]
        } else {
            vec![id(3)]
        };
        for offender in offenders {
            assert!(detail.contains(&format!("coordinate={offender}")));
        }
        assert_eq!(evaluations.get(), 0);
    }
}

fn prepare_counted<'a>(
    f: &'a Fixture,
    request: SparseRequest<'a>,
    execution: &Execution,
    evaluations: &std::cell::Cell<usize>,
) -> Result<SquareAccuracyFactor, Withheld> {
    SquareAccuracyFrame::admit(request, &f.plan, &f.values, &f.facts, execution)?.prepare(
        || {
            evaluations.set(evaluations.get() + 1);
            Ok(f.jacobian())
        },
        execution,
    )
}

#[test]
fn affine_accuracy_full_frame_refuses_original_support_mismatch_before_evaluation() {
    use faer::sparse::{Pair, SymbolicSparseColMat};
    let f = Fixture::new();
    // A complete but different original support still has a perfect matching.
    // Identity, original coordinates, values and row bounds all remain equal.
    let pairs = (0..3)
        .flat_map(|r| (0..3).map(move |c| Pair::new(r, c)))
        .collect::<Vec<_>>();
    let (pattern, _) = SymbolicSparseColMat::try_new_from_indices(3, 3, &pairs).unwrap();
    let square = SquareScope::admit(
        &crate::assembled::contract(&f.plan),
        pattern.as_ref(),
        &[(0., 0.); 3],
        None,
    )
    .unwrap();
    let evaluations = std::cell::Cell::new(0);
    let result = prepare_counted(
        &f,
        SparseRequest {
            scope: &square,
            ..f.request()
        },
        &execution(),
        &evaluations,
    );
    assert!(matches!(result, Err(Withheld::Structural(_))));
    assert_eq!(evaluations.get(), 0);
}

#[derive(Debug, Default)]
struct Admission {
    strict_storage: bool,
    storage_calls: std::sync::atomic::AtomicUsize,
    factor_admissions: std::sync::atomic::AtomicU64,
    factor_observations: std::sync::atomic::AtomicU64,
}
impl crate::solve::WorkAdmission for Admission {
    fn admit_storage(
        &self,
        scope: crate::solve::NativeStorageScope,
        _: &str,
        _: usize,
        opaque: bool,
    ) -> Result<(), ProblemError> {
        self.storage_calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if self.strict_storage && scope == crate::solve::NativeStorageScope::Linear && opaque {
            return Err(ProblemError::Unsupported(
                "strict output accuracy requires complete foreign storage coverage".into(),
            ));
        }
        Ok(())
    }
    fn admit(&self, work: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
        self.factor_admissions.fetch_add(
            work.factorizations.unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
        Ok(())
    }
    fn observe(&self, work: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
        self.factor_observations.fetch_add(
            work.factorizations.unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
        Ok(())
    }
}

#[test]
fn affine_accuracy_shortage_cancellation_and_strict_storage_refuse_before_evaluation() {
    use std::sync::atomic::Ordering::Relaxed;
    let f = Fixture::new();
    let evaluations = std::cell::Cell::new(0);
    let mut run = execution();
    let admission = Arc::new(Admission::default());
    run.work_admission = Some(admission.clone());
    let required = SparseFactor::allowance(3).unwrap()
        + crate::presolve::roots::allowance(3, f.plan.jacobian_pattern().compute_nnz()).unwrap();
    let short = prepare_counted(
        &f,
        SparseRequest {
            bytes: required - 1,
            ..f.request()
        },
        &run,
        &evaluations,
    );
    assert!(matches!(short, Err(Withheld::Memory)));
    run.cancel.store(true, Relaxed);
    let cancelled = prepare_counted(&f, f.request(), &run, &evaluations);
    assert!(
        matches!(cancelled, Err(Withheld::Cause(cause)) if matches!(&*cause, ProblemError::Cancelled))
    );
    assert_eq!(evaluations.get(), 0);
    assert_eq!(admission.factor_admissions.load(Relaxed), 0);
    assert_eq!(admission.factor_observations.load(Relaxed), 0);

    let mut run = execution();
    let strict = Arc::new(Admission {
        strict_storage: true,
        ..Default::default()
    });
    run.work_admission = Some(strict.clone());
    let refused = prepare_counted(&f, f.request(), &run, &evaluations);
    assert!(
        matches!(refused, Err(Withheld::Cause(cause)) if matches!(&*cause, ProblemError::Unsupported(_)))
    );
    assert_eq!(strict.storage_calls.load(Relaxed), 1);
    assert_eq!(strict.factor_admissions.load(Relaxed), 0);
    assert_eq!(strict.factor_observations.load(Relaxed), 0);
    assert_eq!(evaluations.get(), 0);

    // An admitted frame does not permit evaluation after its execution stops.
    let run = execution();
    let frame = f.frame(&run).unwrap();
    run.cancel.store(true, Relaxed);
    let cancelled = frame.prepare(
        || {
            evaluations.set(evaluations.get() + 1);
            Ok(f.jacobian())
        },
        &run,
    );
    assert!(
        matches!(cancelled, Err(Withheld::Cause(cause)) if matches!(&*cause, ProblemError::Cancelled))
    );
    assert_eq!(evaluations.get(), 0);
}

#[test]
fn affine_accuracy_exempt_anchor_keeps_original_first_guard_and_zero_factor_work() {
    use std::sync::atomic::Ordering::Relaxed;
    let f = Fixture::source(Source::FirstGuard);
    let cancellation = pse_diagnostics::Cancellation::default();
    let assembly = Arc::new(
        Arc::new(f.plan.clone())
            .compile(
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancellation.clone(), None),
            )
            .unwrap(),
    );
    let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), cancellation);
    // sqrt(0) is valid at Value. Its original First operation requires x>0,
    // even though the source equality x=0 qualifies for the bound exemption.
    assert_eq!(worker.constraints(&f.values).unwrap(), &[0., 0., 0.]);
    let mut run = execution();
    let admission = Arc::new(Admission::default());
    run.work_admission = Some(admission.clone());
    let frame = f.frame(&run).unwrap();
    let calls = std::cell::Cell::new(0);
    let result = frame.prepare(
        || {
            calls.set(calls.get() + 1);
            worker
                .jacobian(&f.values)
                .cloned()
                .map_err(ProblemError::Math)
        },
        &run,
    );
    let Err(Withheld::Cause(cause)) = result else {
        panic!("the original First guard must withhold the factor");
    };
    let ProblemError::Math(error) = cause.as_ref() else {
        panic!("expected the original mathematical guard cause: {cause}");
    };
    fn assert_original_domain(error: &pse_math::MathError) {
        match error {
            pse_math::MathError::Instance {
                instance, cause, ..
            } => {
                assert_eq!(*instance, id(30));
                assert_original_domain(cause);
            }
            pse_math::MathError::Domain {
                source_id,
                requirement,
            } => {
                assert_eq!(*source_id, id(26));
                assert_eq!(*requirement, "positive");
            }
            other => panic!("expected sqrt's original positive First domain: {other}"),
        }
    }
    assert_original_domain(error);
    assert_eq!(calls.get(), 1);
    assert_eq!(admission.factor_admissions.load(Relaxed), 0);
    assert_eq!(admission.factor_observations.load(Relaxed), 0);
}
