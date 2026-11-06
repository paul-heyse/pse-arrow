// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-point arithmetic uncertainty uses the library's exact-real interval DAG.
use super::*;

pub(crate) struct ArithmeticUncertainty {
    pub residual: Vec<f64>,
    pub jacobian: Vec<f64>,
    pub witness: pse_ids::ContentHash,
    _owner: crate::math::jobs::WorkerCharge,
}

pub(super) enum ArithmeticAssessment {
    Available(ArithmeticUncertainty),
    Unavailable(pse_math::engineering_accuracy::GoalFailure),
}

// Producer unavailability is distinct from the outer terminal runtime error.
type PointAssessment = Result<ArithmeticUncertainty, ArithmeticUnavailable>;

/// Original causes remain typed; a native refusal does not identify a particular
/// source guard unless the native adapter actually supplies that attribution.
#[derive(Debug, thiserror::Error)]
enum ArithmeticUnavailable {
    #[cfg(not(feature = "solver-root-isolation"))]
    #[error("original point arithmetic requires the root-isolation capability")]
    Capability,
    #[cfg(feature = "solver-root-isolation")]
    #[error(
        "original point arithmetic projection has zero node allowance (available_bytes={available})"
    )]
    ProjectionAllowance { available: usize },
    #[cfg(feature = "solver-root-isolation")]
    #[error(transparent)]
    Projection(pse_math::factorable::PointArithmeticUnavailable),
    #[cfg(feature = "solver-root-isolation")]
    #[error("original point arithmetic projection failed: {0}")]
    Factorable(#[source] pse_math::factorable::FactorableError),
    #[cfg(feature = "solver-root-isolation")]
    #[error("original point arithmetic producer unavailable: {refusal}")]
    Profile {
        #[source]
        refusal: native::root_isolation::ArithmeticProfileRefusal,
    },
    #[cfg(feature = "solver-root-isolation")]
    #[error(
        "original point native arithmetic incomplete: reason={reason:?}, actual_work={work:?}; failed source guard attribution is unavailable from the native adapter"
    )]
    Native {
        reason: pse_math::implicit::SelectionProofRefusal,
        work: native::root_isolation::PointArithmeticWork,
    },
    #[cfg(feature = "solver-root-isolation")]
    #[error(
        "original point arithmetic row {row} has no shared finite equality residual output (lower={lower:?}, upper={upper:?})"
    )]
    ResidualMap {
        row: usize,
        lower: Option<usize>,
        upper: Option<usize>,
    },
    #[cfg(feature = "solver-root-isolation")]
    #[error(
        "original point arithmetic {product} enclosure unusable at row={row}, column={column:?}, output={output}: {reason:?}"
    )]
    Enclosure {
        product: &'static str,
        row: usize,
        column: Option<usize>,
        output: usize,
        reason: EnclosureUnavailable,
    },
}
#[cfg(feature = "solver-root-isolation")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnclosureUnavailable {
    MissingOutput,
    InvalidInterval,
    NonfiniteRadius,
}

pse_diagnostics::impl_diagnostic! {
    ArithmeticUnavailable,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::CompileMath) },
    forward(this) { match this {
        #[cfg(feature = "solver-root-isolation")]
        Self::Projection(error) => Some(error),
        #[cfg(feature = "solver-root-isolation")]
        Self::Factorable(error) => Some(error),
        #[cfg(feature = "solver-root-isolation")]
        Self::Profile { refusal } => Some(refusal),
        _ => None,
    } },
    help(_this) { None }, related(_this) { None },
    source(this) { match this {
        #[cfg(feature = "solver-root-isolation")]
        Self::Projection(error) => Some(error),
        #[cfg(feature = "solver-root-isolation")]
        Self::Factorable(error) => Some(error),
        #[cfg(feature = "solver-root-isolation")]
        Self::Profile { refusal } => Some(refusal),
        _ => None,
    } },
    facts(this) {
        #[cfg(feature = "solver-root-isolation")]
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        #[cfg(not(feature = "solver-root-isolation"))]
        let facts = pse_diagnostics::DiagnosticFacts::default();
        #[cfg(feature = "solver-root-isolation")]
        if let Self::Native { reason, work } = this {
            use pse_diagnostics::DiagnosticObservation as O;
            facts.observe("arithmetic_stage", O::Text("native_arithmetic".into()));
            facts.observe("native_reason", O::Text(format!("{reason:?}")));
            facts.observe("failed_guard_attribution", O::Missing);
            for (key, value) in [("guard_evaluations", work.guard_evaluations),
                ("value_evaluations", work.value_evaluations),
                ("jacobian_evaluations", work.jacobian_evaluations),
                ("hessian_evaluations", work.hessian_evaluations)] {
                facts.observe(key, i64::try_from(value).map_or_else(|_| O::Text(value.to_string()), O::Integer));
            }
        }
        let _ = this;
        facts
    }
}
impl pse_model::diagnostic::DiagnosticProjection for ArithmeticUnavailable {}

fn retain_unavailable(
    error: ArithmeticUnavailable,
    budget: &Arc<WorkerBudget>,
) -> Result<ArithmeticAssessment, ProblemError> {
    let bytes = size_of::<ArithmeticUnavailable>();
    #[cfg(feature = "solver-root-isolation")]
    let bytes = match &error {
        ArithmeticUnavailable::Projection(reason) => bytes.saturating_add(
            reason
                .retained_bytes()
                .saturating_sub(size_of::<pse_math::factorable::PointArithmeticUnavailable>()),
        ),
        ArithmeticUnavailable::Factorable(pse_math::factorable::FactorableError::Math(error)) => {
            bytes.saturating_add(
                error
                    .retained_bytes()
                    .saturating_sub(size_of::<pse_math::MathError>()),
            )
        }
        _ => bytes,
    };
    let cause = pse_model::diagnostic::DiagnosticCause::new(error);
    Ok(ArithmeticAssessment::Unavailable(retained_cause(
        cause, bytes, budget,
    )?))
}

fn retain_assessment(
    assessment: PointAssessment,
    execution: &Execution,
    budget: &Arc<WorkerBudget>,
) -> Result<ArithmeticAssessment, ProblemError> {
    // Required task stops outrank optional diagnostic retention.
    execution.check()?;
    match assessment {
        Ok(arithmetic) => Ok(ArithmeticAssessment::Available(arithmetic)),
        Err(error) => retain_unavailable(error, budget),
    }
}

#[cfg(feature = "solver-root-isolation")]
fn projection_failure(
    error: pse_math::factorable::FactorableError,
) -> Result<ArithmeticUnavailable, ProblemError> {
    match error {
        pse_math::factorable::FactorableError::Math(error) => Err(error.into()),
        error => Ok(ArithmeticUnavailable::Factorable(error)),
    }
}

#[cfg(feature = "solver-root-isolation")]
fn enclosure_radius(
    interval: Option<&pse_math::implicit::ProofInterval>,
    value: f64,
) -> Result<f64, EnclosureUnavailable> {
    let interval = interval.ok_or(EnclosureUnavailable::MissingOutput)?;
    if !interval.valid() {
        return Err(EnclosureUnavailable::InvalidInterval);
    }
    let radius = (interval.lower - value)
        .abs()
        .max((interval.upper - value).abs());
    let radius = if radius == 0. { 0. } else { radius.next_up() };
    if radius.is_finite() && radius >= 0. {
        Ok(radius)
    } else {
        Err(EnclosureUnavailable::NonfiniteRadius)
    }
}

struct PointEnclosure<'a> {
    plan: &'a pse_math::assembly::CasePlan,
    values: &'a CaseValues,
    point: &'a [f64],
    residual: &'a [f64],
    jacobian: &'a faer::sparse::SparseColMat<usize, f64>,
    execution: &'a Execution,
    budget: &'a Arc<WorkerBudget>,
    selected_values: bool,
}

#[cfg(not(feature = "solver-root-isolation"))]
fn enclose_point(input: PointEnclosure<'_>) -> Result<PointAssessment, ProblemError> {
    input.execution.check()?;
    let _ = (
        input.plan,
        input.values,
        input.point,
        input.residual,
        input.jacobian,
        input.execution,
        input.budget,
        input.selected_values,
    );
    Ok(Err(ArithmeticUnavailable::Capability))
}

#[cfg(feature = "solver-root-isolation")]
fn enclose_point(input: PointEnclosure<'_>) -> Result<PointAssessment, ProblemError> {
    let PointEnclosure {
        plan,
        values,
        point,
        residual,
        jacobian,
        execution,
        budget,
        selected_values,
    } = input;
    use native::root_isolation::{Ibex, PointArithmeticEvidence};
    execution.check()?;
    let available = budget.capacity().saturating_sub(budget.used());
    let limit = available / 256;
    if limit == 0 {
        return Ok(Err(ArithmeticUnavailable::ProjectionAllowance {
            available,
        }));
    }
    // The existing factorable projection limit bounds construction under the worker
    // allowance. Replace that temporary construction charge with its actual product.
    let projection_charge = budget
        .charge(available)
        .map_err(MathRuntimeError::into_problem)?;
    let program = match plan.assess_point_arithmetic_program_for_order(
        values,
        pse_kernels::DerivativeOrder::First,
        limit,
        &execution.cancel,
    ) {
        Ok(pse_math::factorable::PointArithmeticAssessment::Available(program)) => program,
        Ok(pse_math::factorable::PointArithmeticAssessment::Unavailable(reason)) => {
            return Ok(Err(ArithmeticUnavailable::Projection(reason)));
        }
        Err(error) => return Ok(Err(projection_failure(error)?)),
    };
    drop(projection_charge);
    let _program_charge = budget
        .charge(program.retained_bytes())
        .map_err(MathRuntimeError::into_problem)?;
    if program.graph.inputs != point.len()
        || program.rows.len() != residual.len()
        || jacobian.nrows() != residual.len()
        || jacobian.ncols() != point.len()
    {
        return Err(ProblemError::Contract(
            "original point arithmetic inventory mismatch".into(),
        ));
    }
    let ibex = Ibex::default();
    let inventory =
        match ibex.point_arithmetic_inventory(&program, pse_kernels::DerivativeOrder::First) {
            Ok(inventory) => inventory,
            Err(refusal) => {
                execution.check()?;
                return Ok(Err(ArithmeticUnavailable::Profile { refusal }));
            }
        };
    // Source occurrences and executable predicates have different populations.
    // Reserve the bounded layout before interning guards or measuring closures;
    // the prepared owner keeps this charge through the original native operation.
    let preparation_owner = budget
        .charge(inventory.preparation_bytes())
        .map_err(MathRuntimeError::into_problem)?;
    let prepared =
        match ibex.prepare_point_arithmetic(inventory, Arc::new(preparation_owner), execution)? {
            Ok(prepared) => prepared,
            Err(refusal) => {
                execution.check()?;
                return Ok(Err(ArithmeticUnavailable::Profile { refusal }));
            }
        };
    let workspace = prepared.native_workspace_bytes();
    let entries = residual
        .len()
        .checked_mul(point.len())
        .ok_or_else(|| ProblemError::memory("point arithmetic uncertainty extent"))?;
    let native_entries = point
        .len()
        .checked_add(1)
        .and_then(|n| program.graph.residuals.len().checked_mul(n))
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(|| ProblemError::memory("point arithmetic enclosure extent"))?;
    let retained_bytes = entries
        .checked_add(residual.len())
        .and_then(|n| n.checked_mul(size_of::<f64>()))
        .ok_or_else(|| ProblemError::memory("point arithmetic uncertainty extent"))?;
    let owner = budget
        .charge(retained_bytes)
        .map_err(MathRuntimeError::into_problem)?;
    let scratch_bytes = native_entries
        .checked_add(entries)
        .and_then(|n| n.checked_mul(size_of::<f64>()))
        .ok_or_else(|| ProblemError::memory("point arithmetic enclosure scratch extent"))?;
    let _scratch_charge = budget
        .charge(scratch_bytes)
        .map_err(MathRuntimeError::into_problem)?;
    let _workspace_charge = budget
        .charge(workspace)
        .map_err(MathRuntimeError::into_problem)?;
    let enclosure = ibex
        .enclose_prepared_point_arithmetic_with_execution(&prepared, point, workspace, execution)?;
    execution.check()?;
    let (enclosed_values, enclosed_jacobian) = match enclosure {
        PointArithmeticEvidence::Enclosed {
            values, jacobian, ..
        } => (values, jacobian),
        PointArithmeticEvidence::Interrupted { .. } => return Err(ProblemError::Cancelled),
        PointArithmeticEvidence::Incomplete { reason, work } => {
            return Ok(Err(ArithmeticUnavailable::Native { reason, work }));
        }
    };
    let mut dense = vec![0.; entries];
    for (column, _) in point.iter().enumerate() {
        for entry in jacobian.symbolic().col_range(column) {
            dense[jacobian.symbolic().row_idx()[entry] * point.len() + column] =
                jacobian.val()[entry];
        }
    }
    let mut residual_uncertainty = Vec::with_capacity(residual.len());
    let mut jacobian_uncertainty = Vec::with_capacity(entries);
    for (row, projection) in program.rows.iter().enumerate() {
        let Some(output) = (if selected_values {
            Some(projection.value)
        } else {
            projection
                .lower_residual
                .filter(|_| projection.lower_residual == projection.upper_residual)
        }) else {
            return Ok(Err(ArithmeticUnavailable::ResidualMap {
                row,
                lower: projection.lower_residual,
                upper: projection.upper_residual,
            }));
        };
        let error = match enclosure_radius(enclosed_values.get(output), residual[row]) {
            Ok(error) => error,
            Err(reason) => {
                return Ok(Err(ArithmeticUnavailable::Enclosure {
                    product: "residual",
                    row,
                    column: None,
                    output,
                    reason,
                }));
            }
        };
        residual_uncertainty.push(error);
        for column in 0..point.len() {
            let output = projection.value * point.len() + column;
            let error = match enclosure_radius(
                enclosed_jacobian.get(output),
                dense[row * point.len() + column],
            ) {
                Ok(error) => error,
                Err(reason) => {
                    return Ok(Err(ArithmeticUnavailable::Enclosure {
                        product: "jacobian",
                        row,
                        column: Some(column),
                        output,
                        reason,
                    }));
                }
            };
            jacobian_uncertainty.push(error);
        }
    }
    let mut witness = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
    witness
        .str("point-arithmetic-projection-and-expression")
        .hash(&program.key)
        .hash(&program.graph.identity());
    Ok(Ok(ArithmeticUncertainty {
        residual: residual_uncertainty,
        jacobian: jacobian_uncertainty,
        witness: witness.finish_hash(),
        _owner: owner,
    }))
}

/// Equality residual arithmetic, including original bound subtraction.
pub(super) fn original_point(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    point: &[f64],
    residual: &[f64],
    jacobian: &faer::sparse::SparseColMat<usize, f64>,
    execution: &Execution,
    budget: &Arc<WorkerBudget>,
) -> Result<ArithmeticAssessment, ProblemError> {
    let assessment = enclose_point(PointEnclosure {
        plan,
        values,
        point,
        residual,
        jacobian,
        execution,
        budget,
        selected_values: false,
    })?;
    retain_assessment(assessment, execution, budget)
}

#[cfg(all(test, feature = "solver-root-isolation"))]
mod refusal_tests {
    use super::*;
    use pse_diagnostics::{DiagnosticObservation as O, TypedDiagnostic};
    use pse_math::factorable::{FactorableError, Fidelity, PointArithmeticUnavailable};
    use pse_math::implicit::{ProofInterval, SelectionProofRefusal};

    fn failure(assessment: ArithmeticAssessment) -> pse_math::engineering_accuracy::GoalFailure {
        match assessment {
            ArithmeticAssessment::Unavailable(failure) => failure,
            ArithmeticAssessment::Available(_) => panic!("expected retained refusal"),
        }
    }

    fn arithmetic_profile_refusal() -> native::root_isolation::ArithmeticProfileRefusal {
        use pse_math::factorable::{Node, PointArithmeticProgram, RootIsolationProgram};
        let program = PointArithmeticProgram {
            key: pse_ids::ContentHash::from_bytes([17; 32]),
            rows: vec![],
            objective: None,
            graph: RootIsolationProgram {
                inputs: 1,
                nodes: vec![Node::Var(0)],
                residuals: vec![0; 513],
                eligibility: vec![],
                criterion: [0, 0],
                obligations: vec![],
                derivative_obligations: vec![],
            },
        };
        native::root_isolation::Ibex::default()
            .point_arithmetic_preflight(&program, pse_kernels::DerivativeOrder::First)
            .unwrap_err()
    }

    #[test]
    fn arithmetic_profile_unavailable_retains_original_typed_cause_and_zero_work() {
        let refusal = arithmetic_profile_refusal();
        let budget = WorkerBudget::new(4096);
        let execution = Execution::new(Cancellation::new(false), &Controls::default());
        let retained = failure(
            retain_assessment(
                Err(ArithmeticUnavailable::Profile { refusal }),
                &execution,
                &budget,
            )
            .unwrap(),
        );
        let original = retained
            .cause
            .as_error()
            .downcast_ref::<ArithmeticUnavailable>()
            .unwrap();
        assert!(
            matches!(original, ArithmeticUnavailable::Profile { refusal: actual } if *actual == refusal)
        );
        let source = std::error::Error::source(original)
            .unwrap()
            .downcast_ref::<native::root_isolation::ArithmeticProfileRefusal>()
            .unwrap();
        assert_eq!(*source, refusal);
        let facts = source.diagnostic_facts();
        assert!(facts.sources.is_empty());
        assert!(
            matches!(&facts.observations["arithmetic_scope"], O::Text(value) if value == "optional_original_point_arithmetic")
        );
        assert!(
            matches!(&facts.observations["arithmetic_stage"], O::Text(value) if value == "producer_preflight")
        );
        assert!(
            matches!(&facts.observations["profile_stage"], O::Text(value) if value == "source_inventory")
        );
        assert!(
            matches!(&facts.observations["profile_metric"], O::Text(value) if value == "residual_outputs")
        );
        assert!(matches!(
            facts.observations["profile_observed"],
            O::Integer(513)
        ));
        assert!(matches!(
            facts.observations["profile_limit"],
            O::Integer(512)
        ));
        assert!(matches!(
            facts.observations["native_not_attempted"],
            O::Boolean(true)
        ));
        for metric in [
            "guard_evaluations",
            "value_evaluations",
            "jacobian_evaluations",
            "hessian_evaluations",
        ] {
            assert!(matches!(facts.observations[metric], O::Integer(0)));
        }
        assert_eq!(budget.used(), retained.bytes);
        let shared = retained.clone();
        drop(retained);
        assert_eq!(budget.used(), shared.bytes);
        drop(shared);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn arithmetic_profile_refusal_keeps_actual_budget_and_execution_failures_hard() {
        let refusal = arithmetic_profile_refusal();
        let empty = WorkerBudget::new(0);
        let cancellation = Cancellation::new(false);
        let execution = Execution::new(cancellation.clone(), &Controls::default());
        assert!(
            retain_assessment(
                Err(ArithmeticUnavailable::Profile { refusal }),
                &execution,
                &empty,
            )
            .is_err(),
            "an actual charge refusal cannot become optional producer unavailability"
        );
        assert_eq!(empty.used(), 0);
        cancellation.cancel();
        assert!(matches!(
            retain_assessment(
                Err(ArithmeticUnavailable::Profile { refusal }),
                &execution,
                &empty,
            ),
            Err(ProblemError::Cancelled)
        ));
        assert_eq!(empty.used(), 0);
        assert!(!optional_failure(&ProblemError::Math(
            pse_math::MathError::Limit("enclosing actual grant")
        )));
    }

    #[test]
    fn native_refusal_preserves_actual_reason_work_and_missing_attribution() {
        let work = native::root_isolation::PointArithmeticWork {
            guard_evaluations: 3,
            value_evaluations: 1,
            jacobian_evaluations: 0,
            hessian_evaluations: 0,
        };
        for reason in [
            SelectionProofRefusal::Unsupported,
            SelectionProofRefusal::Resource,
            SelectionProofRefusal::Boundary,
        ] {
            let budget = WorkerBudget::new(4096);
            let retained = failure(
                retain_unavailable(ArithmeticUnavailable::Native { reason, work }, &budget)
                    .unwrap(),
            );
            let original = retained
                .cause
                .as_error()
                .downcast_ref::<ArithmeticUnavailable>()
                .unwrap();
            assert!(
                matches!(original, ArithmeticUnavailable::Native { reason: actual_reason, work: actual_work } if *actual_reason == reason && *actual_work == work)
            );
            let facts = original.diagnostic_facts();
            assert!(
                facts.sources.is_empty(),
                "a native status has no invented guard attribution"
            );
            assert!(matches!(
                facts.observations["failed_guard_attribution"],
                O::Missing
            ));
            assert!(matches!(
                facts.observations["guard_evaluations"],
                O::Integer(3)
            ));
            assert!(matches!(
                facts.observations["value_evaluations"],
                O::Integer(1)
            ));
            assert_eq!(budget.used(), retained.bytes);
            let shared = retained.clone();
            drop(retained);
            assert_eq!(budget.used(), shared.bytes);
            drop(shared);
            assert_eq!(budget.used(), 0);
        }
    }

    #[test]
    fn projection_refusal_retains_original_error_and_required_failures_take_precedence() {
        let budget = WorkerBudget::new(4096);
        let original = FactorableError::DisjunctiveBranch {
            instance: pse_ids::SemanticId::from_bytes([7; 16]),
        };
        let retained =
            failure(retain_unavailable(projection_failure(original).unwrap(), &budget).unwrap());
        let cause = retained
            .cause
            .as_error()
            .downcast_ref::<ArithmeticUnavailable>()
            .unwrap();
        let source = std::error::Error::source(cause)
            .unwrap()
            .downcast_ref::<FactorableError>()
            .unwrap();
        assert!(
            matches!(source, FactorableError::DisjunctiveBranch { instance } if *instance == pse_ids::SemanticId::from_bytes([7; 16]))
        );
        assert!(matches!(
            projection_failure(FactorableError::Math(pse_math::MathError::Cancelled)),
            Err(ProblemError::Math(pse_math::MathError::Cancelled))
        ));
        let contract = projection_failure(FactorableError::Math(pse_math::MathError::Contract(
            "required source invariant".into(),
        )))
        .unwrap_err();
        assert!(!optional_failure(&contract));
        let exhausted = WorkerBudget::new(0);
        assert!(
            retain_unavailable(
                ArithmeticUnavailable::Projection(PointArithmeticUnavailable::Fidelity {
                    fidelity: Fidelity::Relaxed
                }),
                &exhausted
            )
            .is_err()
        );
        assert_eq!(exhausted.used(), 0);

        let cancellation = Cancellation::new(false);
        let execution = Execution::new(cancellation.clone(), &Controls::default());
        cancellation.cancel();
        let empty = WorkerBudget::new(0);
        assert!(matches!(
            retain_assessment(
                Err(ArithmeticUnavailable::ProjectionAllowance { available: 0 }),
                &execution,
                &empty
            ),
            Err(ProblemError::Cancelled)
        ));
        assert_eq!(
            empty.used(),
            0,
            "cancellation precedes optional failure retention"
        );
    }

    #[test]
    fn missing_and_unusable_enclosures_remain_distinct_from_projection_and_native_refusal() {
        assert_eq!(
            enclosure_radius(None, 1.),
            Err(EnclosureUnavailable::MissingOutput)
        );
        assert_eq!(
            enclosure_radius(
                Some(&ProofInterval {
                    lower: 2.,
                    upper: 1.
                }),
                1.
            ),
            Err(EnclosureUnavailable::InvalidInterval)
        );
        assert_eq!(
            enclosure_radius(
                Some(&ProofInterval {
                    lower: -f64::MAX,
                    upper: f64::MAX
                }),
                f64::MAX
            ),
            Err(EnclosureUnavailable::NonfiniteRadius)
        );
        assert_eq!(
            enclosure_radius(
                Some(&ProofInterval {
                    lower: 1.,
                    upper: 1.
                }),
                1.
            ),
            Ok(0.)
        );
        let map = ArithmeticUnavailable::ResidualMap {
            row: 2,
            lower: Some(4),
            upper: Some(5),
        };
        let budget = WorkerBudget::new(4096);
        let retained = failure(retain_unavailable(map, &budget).unwrap());
        assert!(matches!(
            retained
                .cause
                .as_error()
                .downcast_ref::<ArithmeticUnavailable>(),
            Some(ArithmeticUnavailable::ResidualMap {
                row: 2,
                lower: Some(4),
                upper: Some(5)
            })
        ));
    }
}
/// Selected Member values need no bound residual; First uncertainty uses their value output.
pub(crate) fn selected_point(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    point: &[f64],
    outputs: &[f64],
    jacobian: &faer::sparse::SparseColMat<usize, f64>,
    execution: &Execution,
    budget: &Arc<WorkerBudget>,
) -> Result<Option<ArithmeticUncertainty>, ProblemError> {
    // Selected-output capability discovery intentionally consumes availability only;
    // it has no GoalResult destination in which to retain an optional refusal.
    Ok(
        match enclose_point(PointEnclosure {
            plan,
            values,
            point,
            residual: outputs,
            jacobian,
            execution,
            budget,
            selected_values: true,
        })? {
            Ok(arithmetic) => Some(arithmetic),
            Err(_) => None,
        },
    )
}
