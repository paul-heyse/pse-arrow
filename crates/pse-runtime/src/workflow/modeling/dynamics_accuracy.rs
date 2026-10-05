// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Goal-directed paired trajectories live inside the original admitted worker task.
use super::*;
use pse_math::engineering_accuracy::GoalResult;
use pse_model::generated::enums::{
    AccuracyGoalStatus, AccuracyGoalSubject, AccuracyObservation,
    AccuracyUnavailableReason as Unavailable, NumericalAccuracyClass,
};
use pse_model::{
    HeapUsage,
    engineering_accuracy::{AccuracyGoal, BoundGoal},
    strategy::SemanticProductKey,
};

pub(super) fn task_admission(
    scope: pse_kernels::ExecutionScope,
) -> Arc<crate::math::strategy::admission::TaskAdmission> {
    use pse_model::strategy::WorkLimits;
    crate::math::strategy::admission::TaskAdmission::new(
        WorkLimits {
            attempts: 3,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
        scope,
        None,
        false,
    )
}

pub(super) fn complete_dynamic_attempt(
    admission: &crate::math::strategy::admission::TaskAdmission,
    key: ContentHash,
    occurrence: u64,
) -> Result<pse_model::strategy::WorkCharge, pse_backend_native::ProblemError> {
    use pse_model::strategy::{Phase, Scope, WorkCharge, WorkObservation};
    let mut owner = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
    owner.str("dynamic-accuracy-native-attempt").hash(&key).u64(occurrence);
    admission.complete_charge(WorkCharge {
        phase: Phase::Native,
        scope: Scope::Task,
        charging_owner: owner.finish_hash(),
        observed: WorkObservation {
            attempts: 1,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    })
}

pub(super) fn complete_dynamic_assessment(
    admission: &crate::math::strategy::admission::TaskAdmission,
    owner: ContentHash,
    evaluations: Option<u64>,
) -> Result<pse_model::strategy::WorkCharge, pse_backend_native::ProblemError> {
    use pse_model::strategy::{Phase, Scope, WorkCharge, WorkObservation};
    admission.complete_charge(WorkCharge {
        phase: Phase::Assessment,
        scope: Scope::Task,
        charging_owner: owner,
        observed: WorkObservation {
            attempts: 0,
            evaluations,
            iterations: Some(0),
            factorizations: Some(0),
            proof_steps: Some(0),
        },
    })
}

pub(super) fn retained_bytes(rows: &[GoalResult]) -> usize {
    rows.len()
        .saturating_mul(size_of::<GoalResult>())
        .saturating_add(
            rows.iter()
                .map(|r| {
                    r.goal.provenance.capacity()
                        + r.evidence.as_ref().map_or(0, |e| e.limitation.capacity())
                })
                .sum::<usize>(),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::generated::{enums::AccuracyGoalUse, identities::AccuracyGoalId};
    fn goal(observation: AccuracyObservation) -> AccuracyGoal {
        AccuracyGoal {
            goal_id: AccuracyGoalId::from_id(SemanticId::from_bytes([2; 16])),
            model_id: None,
            case_id: None,
            instance_id: None,
            fit_id: None,
            target_id: SemanticId::from_bytes([3; 16]),
            target_kind: NumericalTarget::Variable,
            quantity_id: SemanticId::from_bytes([4; 16]),
            unit_id: SemanticId::from_bytes([5; 16]),
            subject: AccuracyGoalSubject::SelectedOutput,
            observation,
            time: (observation == AccuracyObservation::Sample).then_some(0.5),
            resolution: Some(1.),
            criterion_lower: None,
            criterion_upper: None,
            required_class: NumericalAccuracyClass::Estimated,
            use_policy: AccuracyGoalUse::Assess,
            refine: true,
            source: NumericalSource::Analysis,
            priority: 0,
            provenance: "test physical decision".into(),
        }
    }
    fn sample(time: f64, mode: usize) -> native::Sample {
        native::Sample {
            mode,
            time,
            state: vec![1.],
            outputs: vec![2.],
            integrals: vec![3.],
            state_sensitivities: Vec::new(),
            output_sensitivities: Vec::new(),
        }
    }
    fn report() -> native::Report {
        native::Report {
            termination: native::Termination::Completed,
            completed_time: 1.,
            endpoint: Some(native::TrajectoryEndpoint {
                point: sample(1., 0),
                event: None,
                input_columns: Vec::new(),
                inputs: Vec::new(),
            }),
            requested_initial: vec![1.],
            consistent_initial: vec![1.],
            samples: vec![sample(0.5, 0)],
            events: Vec::new(),
            conservation: Vec::new(),
            statistics: Vec::new(),
            error: None,
            progress: Vec::new(),
            dropped_progress: 0,
        }
    }
    #[test]
    fn goal_accuracy_dynamic_contrast_changes_only_actual_work_controls() {
        let profile = native::Profile {
            rtol: 0.02,
            atol: vec![0.01],
            out_rtol: Some(0.03),
            out_atol: vec![0.04],
            schedule: vec![native::ScheduledInput {
                parameter: 0,
                times: vec![0.5],
            }],
            ..Default::default()
        };
        let contrast = contrasting_profile(&profile, 0.5).unwrap();
        assert_eq!(contrast.rtol, 0.01);
        assert_eq!(contrast.atol, vec![0.005]);
        assert_eq!(contrast.out_rtol, Some(0.015));
        assert_eq!(contrast.out_atol, vec![0.02]);
        assert_eq!(
            native::settings_identity(&profile),
            native::settings_identity(&contrast)
        );
        assert_ne!(controls_identity(&profile), controls_identity(&contrast));
        let original = native::profile_json(&profile);
        let next = native::profile_json(&contrast);
        for field in [
            "numerics",
            "schedule",
            "samples",
            "endpoint",
            "initialization",
            "native",
            "max_steps",
        ] {
            assert_eq!(original[field], next[field], "{field}");
        }
        assert_eq!(
            contrasting_profile(&profile, 1.).unwrap_err(),
            Unavailable::PrecisionLimit
        );
        let tiny = native::Profile {
            rtol: f64::from_bits(1),
            ..Default::default()
        };
        assert_eq!(
            contrasting_profile(&tiny, 0.5).unwrap_err(),
            Unavailable::PrecisionLimit
        );
    }
    #[test]
    fn goal_accuracy_dynamic_requires_actual_observation_and_correspondence() {
        let base = report();
        let mut other = report();
        let selected = goal(AccuracyObservation::Sample);
        assert_eq!(observation(&selected, &base).unwrap().time, 0.5);
        let mut absent = selected.clone();
        absent.time = Some(0.49);
        assert_eq!(
            observation(&absent, &base).unwrap_err(),
            Unavailable::MissingObservation
        );
        assert_eq!(
            observation(&goal(AccuracyObservation::Integrated), &base)
                .unwrap()
                .integrals,
            vec![3.]
        );
        assert!(
            correspondence_shape(
                &trajectory_shape(&base), &trajectory_shape(&other),
                ObservationPoint { mode: base.samples[0].mode, time: base.samples[0].time },
                ObservationPoint { mode: other.samples[0].mode, time: other.samples[0].time },
                selected.observation
            )
            .is_ok()
        );
        other.samples[0].mode = 1;
        assert_eq!(
            correspondence_shape(
                &trajectory_shape(&base), &trajectory_shape(&other),
                ObservationPoint { mode: base.samples[0].mode, time: base.samples[0].time },
                ObservationPoint { mode: other.samples[0].mode, time: other.samples[0].time },
                selected.observation
            )
            .unwrap_err(),
            Unavailable::BranchAmbiguity
        );
        other = report();
        other.events.push(native::EventRecord {
            event: Some(SemanticId::from_bytes([6; 16])),
            time: 0.25,
            before: vec![1.],
            after: Some(vec![2.]),
        });
        assert_eq!(
            correspondence_shape(
                &trajectory_shape(&base), &trajectory_shape(&other),
                ObservationPoint { mode: base.samples[0].mode, time: base.samples[0].time },
                ObservationPoint { mode: other.samples[0].mode, time: other.samples[0].time },
                selected.observation
            )
            .unwrap_err(),
            Unavailable::BranchAmbiguity
        );
        assert!(representational_floor(1.).unwrap() > 0.);
        assert!(representational_floor(0.).unwrap() > 0.);
    }
    #[test]
    fn goal_refinement_strategy_dynamic_uses_actual_demand_and_stops_at_boundary() {
        let hash = ContentHash::from_bytes([1; 32]);
        let mut declaration = goal(AccuracyObservation::Endpoint);
        declaration.resolution = Some(0.3);
        let bound = BoundGoal {
            declaration,
            product: hash,
            normalization: hash,
            source: SemanticProductKey {
                structure: hash,
                binding: hash,
                numerical_policy: Some(hash),
                normalization: Some(hash),
                point: Some(hash),
                parameters: None,
                derivation: None,
                branch: Some(hash),
                accuracy: None,
            },
        };
        let mut pair = pse_backend_native::engineering_accuracy::Comparison {
            source: bound.source,
            base: 4.,
            contrasting: 5.,
            correspondence: hash,
            base_controls: hash,
            contrasting_controls: ContentHash::from_bytes([2; 32]),
            base_arithmetic: hash,
            contrasting_arithmetic: hash,
            base_uncertainty: 0.,
            contrasting_uncertainty: 0.,
            resolution_floor: 1e-14,
        };
        let evidence =
            pse_backend_native::engineering_accuracy::dynamic_comparison(&bound, pair).unwrap();
        pair.base_uncertainty = f64::MAX * 0.75;
        pair.contrasting_uncertainty = f64::MAX * 0.75;
        assert_eq!(
            pse_backend_native::engineering_accuracy::dynamic_comparison(&bound, pair)
                .unwrap_err(),
            Unavailable::Nonfinite,
        );
        let result = GoalResult::assess(&bound, Some(evidence));
        assert_eq!(result.classification.status, AccuracyGoalStatus::Unresolved);
        let ratio = refinement_ratio(std::slice::from_ref(&result))
            .unwrap()
            .unwrap();
        assert!((ratio - 0.3).abs() < 1e-12);
        assert_ne!(ratio, 0.5);
        let mut violated_bound = bound.clone();
        violated_bound.declaration.criterion_upper = Some(2.);
        let assessed_violation = GoalResult::assess(&violated_bound, result.evidence.clone());
        assert_eq!(assessed_violation.classification.criterion,
            pse_model::generated::enums::AccuracyCriterionStatus::Violated);
        assert_eq!(assessed_violation.classification.status, AccuracyGoalStatus::Unresolved);
        assert!(refinement_ratio(std::slice::from_ref(&assessed_violation)).unwrap().is_some());
        violated_bound.declaration.use_policy = AccuracyGoalUse::RequireSatisfied;
        let required_violation = GoalResult::assess(&violated_bound, result.evidence.clone());
        assert_eq!(refinement_ratio(&[required_violation]).unwrap(), None);
        let mut exact_boundary = result.clone();
        exact_boundary.goal.resolution = None;
        exact_boundary.goal.criterion_upper = Some(4.);
        assert_eq!(
            refinement_ratio(&[exact_boundary]).unwrap_err(),
            Unavailable::Boundary
        );
        let mut disabled = result;
        disabled.goal.refine = false;
        assert_eq!(refinement_ratio(&[disabled]).unwrap(), None);
    }
    #[test]
    fn dynamic_accuracy_uses_live_schedule_and_endpoint_input_receipts() {
        let parameter = SemanticId::from_bytes([7; 16]);
        let state = SemanticId::from_bytes([8; 16]);
        let output = SemanticId::from_bytes([9; 16]);
        let contract = native::Contract {
            quadratures: vec![],
            balances: vec![],
            identity: ContentHash::from_bytes([10; 32]),
            states: vec![state],
            differential: vec![true],
            parameters: vec![parameter],
            outputs: vec![output],
            events: vec![vec![]],
            signs: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
        };
        let profile = native::Profile {
            schedule: vec![native::ScheduledInput { parameter: 0, times: vec![0.5] }],
            ..Default::default()
        };
        let integration = [2., -1.];
        let base = report();
        let sample = sample(0.5, 0);
        assert_eq!(
            point_parameters(&profile, &contract, &integration, &base, &sample, AccuracyObservation::Sample).unwrap(),
            [-1.],
            "a scheduled sample reads the expanded integration vector exactly once"
        );
        let mut endpoint_report = report();
        let endpoint = endpoint_report.endpoint.as_mut().unwrap();
        endpoint.point.time = 0.5;
        endpoint.input_columns = vec![0];
        endpoint.inputs = vec![2.];
        let point = endpoint.point.clone();
        assert_eq!(
            point_parameters(&profile, &contract, &integration, &endpoint_report, &point, AccuracyObservation::Endpoint).unwrap(),
            [2.],
            "the endpoint keeps its captured pre-change input receipt"
        );
        endpoint_report.endpoint.as_mut().unwrap().input_columns = vec![1];
        assert!(matches!(
            point_parameters(&profile, &contract, &integration, &endpoint_report, &point, AccuracyObservation::Endpoint),
            Err(pse_backend_native::ProblemError::Contract(_))
        ));
    }

    #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
    #[test]
    fn dynamic_accuracy_comparator_keeps_terminal_causes() {
        use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic, DiagnosticRule};
        use pse_diagnostics::DiagnosticStage;
        let checks = || super::checks::SampleChecks {
            rows: Vec::new(), reports: Vec::new(), complete: true, error: None,
        };
        let scope = pse_kernels::ExecutionScope::new(
            Arc::new(std::sync::atomic::AtomicBool::new(false)), None,
        );
        for (termination, expected) in [
            (native::Termination::Cancelled, "cancelled"),
            (native::Termination::TimeLimit, "time"),
            (native::Termination::StepLimit, "work"),
            (native::Termination::EventLimit, "work"),
            (native::Termination::Failed, "internal"),
            (native::Termination::Panic, "internal"),
        ] {
            let mut report = report();
            report.termination = termination;
            let error = ModelingSimulation::comparator_failure(&mut report, &checks(), &scope).unwrap_err();
            match (expected, error) {
                ("cancelled", pse_backend_native::ProblemError::Cancelled) => {}
                ("time", pse_backend_native::ProblemError::Limit { kind: pse_backend_native::LimitKind::Time, .. }) => {}
                ("work", pse_backend_native::ProblemError::Limit { kind: pse_backend_native::LimitKind::Work, .. }) => {}
                ("internal", pse_backend_native::ProblemError::Internal(_)) => {}
                (_, other) => panic!("{termination:?} lost its terminal class: {other:?}"),
            }
        }
        let mut completed_report = report();
        let mut bad_checks = checks();
        bad_checks.error = Some(BoundaryDiagnostic::new(
            BoundaryClass::Internal,
            DiagnosticStage::ModelingDynamic,
            [],
            DiagnosticRule::WorkflowUnclassified,
        ));
        assert!(matches!(
            ModelingSimulation::comparator_failure(&mut completed_report, &bad_checks, &scope),
            Err(pse_backend_native::ProblemError::Provider(pse_kernels::ProviderError::Nested { .. }))
        ), "a completed trajectory with a source check error must not become an optional unavailable result");
        completed_report.termination = native::Termination::StepLimit;
        assert!(matches!(
            ModelingSimulation::comparator_failure(&mut completed_report, &bad_checks, &scope),
            Err(pse_backend_native::ProblemError::Limit { kind: pse_backend_native::LimitKind::Work, .. })
        ), "a terminal native limit takes precedence over the retained check error");
        let mut error_report = report();
        error_report.error = Some(pse_backend_native::ProblemError::Unsupported(
            "selected dynamic operation is unsupported".into(),
        ));
        assert!(matches!(
            ModelingSimulation::comparator_failure(&mut error_report, &checks(), &scope),
            Err(pse_backend_native::ProblemError::Unsupported(_))
        ), "a structured source error remains available for optional-failure classification");
    }
}

/// The contrast changes integration work controls only. Physical goals/checks, method,
/// schedule, initialization and library nonlinear controls retain their meanings.
fn contrasting_profile(
    profile: &native::Profile,
    ratio: f64,
) -> Result<native::Profile, Unavailable> {
    if !ratio.is_finite() || ratio <= 0. || ratio >= 1. {
        return Err(Unavailable::PrecisionLimit);
    }
    let mut next = profile.clone();
    let tighten = |value: f64| {
        let next = value * ratio;
        if !next.is_finite() || next <= 0. || next >= value {
            Err(Unavailable::PrecisionLimit)
        } else {
            Ok(next)
        }
    };
    next.rtol = tighten(profile.rtol)?;
    next.atol = profile
        .atol
        .iter()
        .map(|v| tighten(*v))
        .collect::<Result<_, _>>()?;
    next.out_rtol = profile.out_rtol.map(tighten).transpose()?;
    next.out_atol = profile
        .out_atol
        .iter()
        .map(|v| tighten(*v))
        .collect::<Result<_, _>>()?;
    Ok(next)
}

/// An observation cannot borrow the nearest sample or substitute a state for an integral.
fn observation<'a>(
    goal: &AccuracyGoal,
    report: &'a native::Report,
) -> Result<&'a native::Sample, Unavailable> {
    match goal.observation {
        AccuracyObservation::Sample => report
            .samples
            .iter()
            .find(|s| Some(s.time) == goal.time)
            .ok_or(Unavailable::MissingObservation),
        AccuracyObservation::Endpoint | AccuracyObservation::Integrated => report
            .endpoint
            .as_ref()
            .map(|e| &e.point)
            .ok_or(Unavailable::MissingObservation),
        AccuracyObservation::Steady => Err(Unavailable::UnsupportedObservation),
    }
}

/// Compare actual event meanings and mode sequences. Event times may differ numerically;
/// they are retained in the witness, never silently treated as a fixed-time sample.
fn correspondence_shape(
    base: &TrajectoryShape,
    other: &TrajectoryShape,
    at_base: ObservationPoint,
    at_other: ObservationPoint,
    observation: AccuracyObservation,
) -> Result<ContentHash, Unavailable> {
    if base.termination != other.termination
        || base.events.len() != other.events.len()
        || base.events.iter().zip(&other.events).any(|(a, b)| a.0 != b.0 || a.2 != b.2)
        || base.endpoint_event != other.endpoint_event
        || base.samples.len() != other.samples.len()
        || base.samples.iter().zip(&other.samples).any(|(a, b)| a.0 != b.0 || a.1 != b.1)
        || at_base.mode != at_other.mode
        || observation == AccuracyObservation::Sample && at_base.time != at_other.time
    {
        return Err(Unavailable::BranchAmbiguity);
    }
    let mut h = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
    h.str("actual-dynamic-correspondence")
        .str(observation.as_str())
        .part(&at_base.time.to_bits().to_le_bytes())
        .part(&at_other.time.to_bits().to_le_bytes())
        .u64(at_base.mode as u64);
    for (a, b) in base.events.iter().zip(&other.events) {
        h.bool(a.0.is_some());
        if let Some(id) = a.0 {
            h.part(id.as_bytes());
        }
        h.part(&a.1.to_bits().to_le_bytes())
            .part(&b.1.to_bits().to_le_bytes())
            .bool(a.2);
    }
    Ok(h.finish_hash())
}

fn point_identity(point: &native::Sample) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
    h.str("actual-dynamic-observation")
        .u64(point.mode as u64)
        .part(&point.time.to_bits().to_le_bytes());
    for cells in [&point.state, &point.outputs, &point.integrals] {
        h.u64(cells.len() as u64);
        for v in cells {
            h.part(&v.to_bits().to_le_bytes());
        }
    }
    h.finish_hash()
}

fn controls_identity(profile: &native::Profile) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
    h.str("actual-dynamic-work-controls")
        .str(&native::profile_json(profile).to_string());
    h.finish_hash()
}

fn representational_floor(value: f64) -> Result<f64, Unavailable> {
    if !value.is_finite() {
        return Err(Unavailable::Nonfinite);
    }
    let floor = (value.next_up() - value)
        .abs()
        .max((value - value.next_down()).abs());
    if floor.is_finite() && floor > 0. {
        Ok(floor)
    } else {
        Err(Unavailable::PrecisionLimit)
    }
}

enum GoalBuildError {
    Unavailable(Unavailable),
    Failure(pse_backend_native::ProblemError),
}
impl From<Unavailable> for GoalBuildError {
    fn from(reason: Unavailable) -> Self { Self::Unavailable(reason) }
}
impl From<pse_backend_native::ProblemError> for GoalBuildError {
    fn from(error: pse_backend_native::ProblemError) -> Self { Self::Failure(error) }
}

#[derive(Clone, Copy)]
struct ObservationPoint { mode: usize, time: f64 }
struct TrajectoryShape {
    termination: native::Termination,
    events: Vec<(Option<SemanticId>, f64, bool)>,
    endpoint_event: Option<SemanticId>,
    samples: Vec<(f64, usize)>,
}
fn trajectory_shape(report: &native::Report) -> TrajectoryShape {
    TrajectoryShape {
        termination: report.termination,
        events: report.events.iter().map(|event| (event.event, event.time, event.after.is_some())).collect(),
        endpoint_event: report.endpoint.as_ref().and_then(|endpoint| endpoint.event),
        samples: report.samples.iter().map(|sample| (sample.time, sample.mode)).collect(),
    }
}
pub(super) fn function_provider_ids(
    program: &FunctionProgram,
    selected_rows: &BTreeSet<SemanticId>,
) -> Result<Vec<SemanticId>, pse_backend_native::ProblemError> {
    let assembly = &program.case.assembly;
    let mut providers = BTreeSet::new();
    for binding in assembly.structure().instances() {
        let outputs = binding.contributions.iter().filter_map(|contribution| {
            matches!(contribution.target, pse_math::binding::Target::Row(row) if selected_rows.contains(&row))
                .then_some(contribution.output)
        }).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        if outputs.is_empty() { continue; }
        let body = assembly.bodies().get(&binding.body)
            .ok_or_else(|| pse_backend_native::ProblemError::Math(
                pse_math::MathError::Contract("dynamic provider demand body is absent".into()),
            ))?;
        let demanded = body.provider_demands_for_outputs(&outputs, DerivativeOrder::Value)?;
        providers.extend(body.providers().iter().filter(|provider| demanded.contains_key(&provider.key()))
            .map(|provider| provider.id));
    }
    Ok(providers.into_iter().collect())
}
/// Selected semantic output inputs from the library's retained dependency projection.
/// This keeps a provider-free `Output` wrapper such as `q(x)` linked to the native
/// trajectory source for `x`; a provider-free `q(t)` remains independent of that source.
fn selected_output_dependencies(
    selected_plan: &pse_math::assembly::CasePlan,
    row: SemanticId,
    cancel: &Arc<std::sync::atomic::AtomicBool>,
) -> Result<BTreeSet<SemanticId>, pse_backend_native::ProblemError> {
    use pse_math::binding::Target;

    let dependencies = selected_plan.dependencies(cancel)?;
    let selected = dependencies.iter().filter(|dependency| {
        matches!(dependency.target, Target::Row(target) if target == row)
    }).collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(pse_backend_native::ProblemError::Math(
            pse_math::MathError::Contract("dynamic selected output dependency projection is absent".into()),
        ));
    }
    Ok(selected.into_iter().flat_map(|dependency| dependency.execution.iter().copied()).collect())
}

/// Bound selected provider/dependency inspection by the existing per-task reservation.
/// The library projection can retain at most one semantic source per formal slot per
/// selected contribution, plus the selected provider map.
fn source_inspection_bytes(programs: &[FunctionProgram], mode: usize) -> usize {
    programs.iter().filter(|program| program.mode == mode
        && matches!(program.function, Function::Initial | Function::Rhs | Function::Output))
        .fold(0usize, |total, program| {
            let plan = &program.case.assembly;
            let bytes = plan.structure().instances().iter().fold(0usize, |sum, binding| {
                let slots = binding.slots.len();
                let outputs = binding.contributions.len();
                let provider_count = plan.bodies().get(&binding.body).map_or(0, |body| body.providers().len());
                let dependency = outputs.saturating_mul(
                    size_of::<pse_math::assembly::ContributionDependencies>()
                        .saturating_add(slots.saturating_mul(size_of::<SemanticId>() + 32)),
                );
                let providers = outputs.saturating_mul(size_of::<usize>())
                    .saturating_add(provider_count.saturating_mul(128));
                sum.saturating_add(dependency).saturating_add(providers)
            });
            total.saturating_add(bytes)
        })
}

/// Bind contract inputs at the exact reported point. Endpoint values and columns are the
/// native owner's live receipt, which may precede a scheduled change at the terminal time.
/// Samples read the active right-continuous schedule from the expanded integration vector.
fn point_parameters(
    profile: &native::Profile,
    contract: &native::Contract,
    integration: &[f64],
    report: &native::Report,
    point: &native::Sample,
    observation: AccuracyObservation,
) -> Result<Vec<f64>, pse_backend_native::ProblemError> {
    let expected = match observation {
        AccuracyObservation::Endpoint => {
            let endpoint = report
                .endpoint
                .as_ref()
                .filter(|endpoint| point_identity(&endpoint.point) == point_identity(point))
                .ok_or_else(|| pse_backend_native::ProblemError::Contract(
                    "endpoint accuracy lacks its live input receipt".into(),
                ))?;
            let width = profile.integration_width(contract.parameters.len());
            if endpoint.input_columns.len() != contract.parameters.len()
                || endpoint.inputs.len() != contract.parameters.len()
                || endpoint.input_columns.iter().any(|column| *column >= width)
                || endpoint.input_columns.iter().zip(&endpoint.inputs).any(|(column, input)| {
                    integration.get(*column).is_none_or(|live| live.to_bits() != input.to_bits())
                })
            {
                return Err(pse_backend_native::ProblemError::Contract(
                    "endpoint input receipt does not match its captured integration columns".into(),
                ));
            }
            endpoint.inputs.clone()
        }
        AccuracyObservation::Sample => profile.parameters_at(integration, point.time),
        AccuracyObservation::Integrated | AccuracyObservation::Steady => {
            return Err(pse_backend_native::ProblemError::Unsupported(
                "this observation has no point parameter binding".into(),
            ));
        }
    };
    if expected.len() != contract.parameters.len() || expected.iter().any(|value| !value.is_finite()) {
        return Err(pse_backend_native::ProblemError::Contract(
            "dynamic observation input values do not match its contract".into(),
        ));
    }
    Ok(expected)
}

pub(super) fn function_row_ids(program: &FunctionProgram) -> BTreeSet<SemanticId> {
    let rows = program.case.assembly.structure().rows();
    program.rows.iter().filter_map(|index| rows.get(*index).map(|row| row.id)).collect()
}
struct GoalPoint {
    bound: BoundGoal,
    value: f64,
    uncertainty: f64,
    arithmetic: ContentHash,
    point: ObservationPoint,
}
struct CapturedTrajectory {
    shape: TrajectoryShape,
    goals: Vec<Result<GoalPoint, Unavailable>>,
}

fn interval_add(left: pse_math::implicit::ProofInterval, right: pse_math::implicit::ProofInterval)
    -> Option<pse_math::implicit::ProofInterval> {
    let lower = left.lower + right.lower;
    let upper = left.upper + right.upper;
    (lower.is_finite() && upper.is_finite()).then_some(pse_math::implicit::ProofInterval {
        lower: lower.next_down(), upper: upper.next_up(),
    })
}
fn interval_mul(left: f64, right: f64) -> Option<pse_math::implicit::ProofInterval> {
    let value = left * right;
    value.is_finite().then_some(pse_math::implicit::ProofInterval {
        lower: value.next_down(), upper: value.next_up(),
    })
}
fn interval_product(
    left: pse_math::implicit::ProofInterval,
    right: pse_math::implicit::ProofInterval,
) -> Option<pse_math::implicit::ProofInterval> {
    let products = [
        left.lower * right.lower,
        left.lower * right.upper,
        left.upper * right.lower,
        left.upper * right.upper,
    ];
    if products.iter().any(|value| !value.is_finite()) {
        return None;
    }
    let lower = products.iter().copied().fold(f64::INFINITY, f64::min);
    let upper = products.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some(pse_math::implicit::ProofInterval {
        lower: lower.next_down(),
        upper: upper.next_up(),
    })
}
fn coordinate_interval(
    native: f64,
    scale: f64,
    offset: f64,
) -> Option<pse_math::implicit::ProofInterval> {
    interval_mul(native, scale).and_then(|scaled| {
        interval_add(
            scaled,
            pse_math::implicit::ProofInterval {
                lower: offset,
                upper: offset,
            },
        )
    })
}
fn time_interval(
    time: f64,
    origin: f64,
    scale: f64,
) -> Option<pse_math::implicit::ProofInterval> {
    let difference = interval_add(
        pse_math::implicit::ProofInterval {
            lower: time,
            upper: time,
        },
        pse_math::implicit::ProofInterval {
            lower: -origin,
            upper: -origin,
        },
    )?;
    let reciprocal = 1.0 / scale;
    if !reciprocal.is_finite() || reciprocal == 0. {
        return None;
    }
    interval_product(
        difference,
        pse_math::implicit::ProofInterval {
            lower: reciprocal.next_down(),
            upper: reciprocal.next_up(),
        },
    )
}
fn interval_error(interval: pse_math::implicit::ProofInterval, observed: f64) -> Option<f64> {
    let error = (interval.lower - observed).abs().max((interval.upper - observed).abs());
    (error.is_finite() && error >= 0.).then_some(if error == 0. { 0. } else { error.next_up() })
}

#[cfg(test)]
fn point_stages() -> &'static std::sync::Mutex<BTreeMap<SemanticId, String>> {
    use std::sync::{Mutex, OnceLock};
    static STAGES: OnceLock<Mutex<BTreeMap<SemanticId, String>>> = OnceLock::new();
    STAGES.get_or_init(|| Mutex::new(BTreeMap::new()))
}
#[cfg(test)]
fn record_point_stage(target: SemanticId, stage: String) {
    point_stages().lock().unwrap().insert(target, stage);
}
#[cfg(not(test))]
fn record_point_stage(_target: SemanticId, _stage: String) {}
#[cfg(test)]
pub(super) fn recorded_point_stage(target: SemanticId) -> Option<String> {
    point_stages().lock().unwrap().get(&target).cloned()
}

#[cfg(not(feature = "solver-root-isolation"))]
impl ModelingSimulation {
    fn point_uncertainty(&self, _goal: &AccuracyGoal, _goal_index: usize, _report: &native::Report, _point: &native::Sample,
        _profile: &native::Profile, _occurrence: u64,
        _scope: &pse_kernels::ExecutionScope, _budget: &Arc<crate::math::WorkerBudget>,
        _charges: &mut Vec<pse_model::strategy::WorkCharge>)
        -> Result<Option<(f64, ContentHash)>, pse_backend_native::ProblemError> { Ok(None) }
}

#[cfg(feature = "solver-root-isolation")]
impl ModelingSimulation {
    /// Same-point authored expression enclosure for a selected scalar output. Endpoint
    /// input values come from the endpoint receipt; sample schedule inputs are immutable
    /// profile bindings. Integrated goals remain unavailable until the integrator can
    /// report accumulated callback and arithmetic uncertainty.
    fn point_uncertainty(&self, goal: &AccuracyGoal, goal_index: usize, report: &native::Report, point: &native::Sample,
        profile: &native::Profile, occurrence: u64,
        scope: &pse_kernels::ExecutionScope, budget: &Arc<crate::math::WorkerBudget>,
        charges: &mut Vec<pse_model::strategy::WorkCharge>)
        -> Result<Option<(f64, ContentHash)>, pse_backend_native::ProblemError> {
        use pse_backend_native::root_isolation::{Ibex, PointArithmeticEvidence};
        use pse_kernels::DerivativeOrder;

        record_point_stage(goal.target_id, "entered point arithmetic".into());
        scope.check().map_err(pse_backend_native::ProblemError::from)?;
        if goal.observation == AccuracyObservation::Integrated || goal.target_kind == NumericalTarget::Objective {
            record_point_stage(goal.target_id, "observation/target has no point arithmetic route".into());
            return Ok(None);
        }
        let available = budget.capacity().saturating_sub(budget.used());
        // Acquire the existing math-pool lease before provider and dependency inspection
        // allocate selected maps or the library's semantic dependency projection.
        let _pool_reservation = self.runtime.shared.math()
            .reserve("math:dynamic-point-arithmetic", available)
            .map_err(|_| pse_backend_native::ProblemError::memory("dynamic point arithmetic reservation"))?;
        let source_bytes = source_inspection_bytes(&self.programs, point.mode);
        let _source_charge = budget.charge(source_bytes)
            .map_err(|_| pse_backend_native::ProblemError::memory("dynamic point arithmetic source dependencies"))?;
        if goal.target_kind == NumericalTarget::Variable {
            // Only a provider actually referenced by this mode's initial/rate program
            // can contribute opaque uncertainty to the native state trajectory.
            for program in self.programs.iter().filter(|program| program.mode == point.mode
                && matches!(program.function, Function::Initial | Function::Rhs)) {
                if !function_provider_ids(program, &function_row_ids(program))?.is_empty() {
                    record_point_stage(goal.target_id, "state depends on opaque initial/rate provider".into());
                    return Ok(None);
                }
            }
            let index = self.contract.states.iter().position(|id| *id == goal.target_id)
                .ok_or_else(|| pse_backend_native::ProblemError::Unsupported("dynamic accuracy state target is not a state".into()))?;
            let binding = self.coordinates.state.get(index)
                .ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic state coordinate binding absent".into()))?;
            let native_value = *point.state.get(index)
                .ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic state observation absent".into()))?;
            let observed = native_value * binding.scale + binding.offset;
            let product = interval_mul(native_value, binding.scale)
                .and_then(|interval| interval_add(interval, pse_math::implicit::ProofInterval {
                    lower: binding.offset, upper: binding.offset,
                }));
            let Some(error) = product.and_then(|interval| interval_error(interval, observed)) else { return Ok(None); };
            let mut witness = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
            witness.str("dynamic-state-coordinate-arithmetic").id(&goal.target_id)
                .part(&native_value.to_bits().to_le_bytes())
                .part(&binding.scale.to_bits().to_le_bytes())
                .part(&binding.offset.to_bits().to_le_bytes());
            return Ok(Some((error, witness.finish_hash())));
        }
        if goal.target_kind != NumericalTarget::Observable {
            record_point_stage(goal.target_id, "target is not a selected scalar observable".into());
            return Ok(None);
        }
        let row_id = ModelingOutput::Member(goal.target_id).row_id();
        let Some(program) = self.programs.iter().find(|program| program.mode == point.mode
            && program.function == Function::Output
            && program.rows.iter().any(|index| program.case.assembly.structure().rows().get(*index)
                .is_some_and(|row| row.id == row_id))) else {
            record_point_stage(goal.target_id, "selected output function program is absent".into());
            return Ok(None);
        };
        // A provider registration matters only when the selected expression consumes it,
        // directly or through a state supplied by an opaque initial/rate provider.
        if !function_provider_ids(program, &BTreeSet::from([row_id]))?.is_empty() {
            record_point_stage(goal.target_id, "selected output consumes opaque provider".into());
            return Ok(None);
        }
        let selected_plan_bytes = program.case.assembly.allocation_bytes();
        // A selected plan may be rebuilt once after its retained semantic dependency
        // projection identifies bound dynamic inputs (notably time) absent from the
        // original solver derivative-coordinate list.
        let selected_plan_reservation = selected_plan_bytes.checked_mul(2)
            .ok_or_else(|| pse_backend_native::ProblemError::memory("dynamic selected output projection extent"))?;
        let _selected_plan_charge = budget.charge(selected_plan_reservation)
            .map_err(|_| pse_backend_native::ProblemError::memory("dynamic selected output projection"))?;
        let mut selected_plan = program.case.assembly.functions(
            &[row_id], program.case.assembly.columns().to_vec(), &self.quantities,
            DerivativeOrder::Value, scope.cancellation(),
        )?;
        let dependencies = selected_output_dependencies(&selected_plan, row_id, scope.cancellation())?;
        let states = self.coordinates.state.iter().map(|binding| binding.id).collect::<BTreeSet<_>>();
        let uses_state = dependencies.iter().any(|source| states.contains(source));
        let mut coordinates = selected_plan.columns().to_vec();
        let declared = selected_plan.structure().variables().iter().map(|variable| variable.port.id)
            .chain(selected_plan.structure().parameters().iter().map(|parameter| parameter.id))
            .collect::<BTreeSet<_>>();
        let dynamic_sources = std::iter::once(self.coordinates.time)
            .chain(self.coordinates.state.iter().map(|binding| binding.id))
            .chain(self.coordinates.parameters.iter().map(|binding| binding.id))
            .collect::<BTreeSet<_>>();
        for source in dynamic_sources.intersection(&dependencies).copied() {
            if !declared.contains(&source) && !coordinates.contains(&source) {
                record_point_stage(goal.target_id, format!(
                    "selected point dependency {source} is not a declared CasePlan coordinate"));
                return Ok(None);
            }
            if declared.contains(&source) && !coordinates.contains(&source) {
                coordinates.push(source);
            }
        }
        if coordinates != selected_plan.columns() {
            selected_plan = program.case.assembly.functions(
                &[row_id], coordinates, &self.quantities,
                DerivativeOrder::Value, scope.cancellation(),
            )?;
        }
        let dependencies = selected_output_dependencies(&selected_plan, row_id, scope.cancellation())?;
        let uses_state = dependencies.iter().any(|source| states.contains(source));
        record_point_stage(goal.target_id, format!(
            "selected plan: inputs={}, output uses state={uses_state}, dynamic inputs={:?}",
            selected_plan.columns().len(), dependencies.intersection(&dynamic_sources).collect::<Vec<_>>()));
        if uses_state {
            for source in self.programs.iter().filter(|source| source.mode == point.mode
                && matches!(source.function, Function::Initial | Function::Rhs)) {
                if !function_provider_ids(source, &function_row_ids(source))?.is_empty() {
                    record_point_stage(goal.target_id, "selected output depends on state from opaque initial/rate provider".into());
                    return Ok(None);
                }
            }
        }
        let Some(local) = program.rows.iter().position(|index| program.case.assembly.structure().rows()
            .get(*index).is_some_and(|row| row.id == row_id)) else { return Ok(None); };
        let base_values = &self.modes.get(point.mode)
            .ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic mode for accuracy point absent".into()))?
            .context.values;
        let plan = &program.case.assembly;
        let available = budget.capacity().saturating_sub(budget.used());
        let context_bytes = base_values.scalars.len()
            .saturating_mul(size_of::<SemanticId>() + size_of::<f64>() + 24)
            .saturating_add(selected_plan.columns().len().saturating_mul(
                size_of::<SemanticId>() + size_of::<f64>() + size_of::<pse_math::implicit::ProofInterval>() + 32,
            ));
        let _context_charge = budget.charge(context_bytes)
            .map_err(|_| pse_backend_native::ProblemError::memory("dynamic point arithmetic inputs"))?;
        // Point arithmetic consumes the same selected compiler projection used by the
        // supplier-dependency guard above, never the runtime program's sibling rows.
        // Earlier source and selected-plan charges are already reflected in `available`.
        let graph_available = available.saturating_sub(context_bytes);
        let node_limit = self.profile.max_cells.min(graph_available / 256);
        if node_limit == 0 {
            record_point_stage(goal.target_id, "no point projection node budget".into());
            return Ok(None);
        }
        let projection_charge = budget.charge(graph_available)
            .map_err(|_| pse_backend_native::ProblemError::memory("dynamic point arithmetic projection"))?;
        // The lease above covers point construction, projection, IBEX workspace and
        // retained enclosures before any temporary allocation is made.
        let mut values = base_values.clone();
        let mut coordinate_regions = BTreeMap::new();
        let normalized_time = (point.time - self.coordinates.time_origin) / self.coordinates.time_scale;
        values.scalars.insert(self.coordinates.time, normalized_time);
        let Some(time_region) = time_interval(point.time, self.coordinates.time_origin, self.coordinates.time_scale) else {
            return Ok(None);
        };
        coordinate_regions.insert(self.coordinates.time, time_region);
        for (binding, native_value) in self.coordinates.state.iter().zip(&point.state) {
            values.scalars.insert(binding.id, native_value * binding.scale + binding.offset);
            let Some(region) = coordinate_interval(*native_value, binding.scale, binding.offset) else {
                return Ok(None);
            };
            coordinate_regions.insert(binding.id, region);
        }
        let parameters = point_parameters(
            profile,
            &self.contract,
            &self.parameters,
            report,
            point,
            goal.observation,
        )?;
        for (binding, value) in self.coordinates.parameters.iter().zip(parameters) {
            values.scalars.insert(binding.id, value * binding.scale + binding.offset);
            let Some(region) = coordinate_interval(value, binding.scale, binding.offset) else {
                return Ok(None);
            };
            coordinate_regions.insert(binding.id, region);
        }
        let point_values = selected_plan.columns().iter().map(|id| values.scalars.get(id).copied())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic authored accuracy point is missing a bound column".into()))?;
        let point_regions = selected_plan.columns().iter().zip(&point_values).map(|(id, value)| {
            coordinate_regions.get(id).copied().or_else(|| {
                (value.is_finite()).then_some(pse_math::implicit::ProofInterval { lower: *value, upper: *value })
            })
        }).collect::<Option<Vec<_>>>().ok_or_else(||
            pse_backend_native::ProblemError::Contract("dynamic authored accuracy box is incomplete".into()))?;
        let projection = selected_plan.point_arithmetic_program_for_order(
            &values, DerivativeOrder::Value, node_limit, scope.cancellation(),
        )
            .map_err(|error| match error {
                pse_math::factorable::FactorableError::Math(error) => pse_backend_native::ProblemError::from(error),
                _ => pse_backend_native::ProblemError::Unsupported("dynamic expression has no bounded point arithmetic projection".into()),
            })?;
        let Some(projection) = projection else {
            record_point_stage(goal.target_id, format!(
                "factorable point projection returned None; selected inputs={}", selected_plan.columns().len()));
            return Ok(None);
        };
        drop(projection_charge);
        if projection.graph.inputs != point_values.len() {
            record_point_stage(goal.target_id, format!(
                "point projection input mismatch: graph={}, values={}", projection.graph.inputs, point_values.len()));
            return Ok(None);
        }
        let owner = budget.charge(projection.retained_bytes())
            .map_err(|_| pse_backend_native::ProblemError::memory("dynamic point arithmetic source"))?;
        let workspace = Ibex.point_arithmetic_workspace_bytes_for_order(&projection, DerivativeOrder::Value)?;
        let _workspace = budget.charge(workspace)
            .map_err(|_| pse_backend_native::ProblemError::memory("dynamic point arithmetic workspace"))?;
        let value_bytes = projection.graph.residuals.len()
            .checked_mul(size_of::<pse_math::implicit::ProofInterval>())
            .ok_or_else(|| pse_backend_native::ProblemError::memory("dynamic point arithmetic values"))?;
        let row_bytes = projection.rows.len()
            .checked_mul(size_of::<pse_math::factorable::PointArithmeticRow>())
            .ok_or_else(|| pse_backend_native::ProblemError::memory("dynamic point arithmetic rows"))?;
        let retained = value_bytes
            .checked_add(row_bytes)
            .ok_or_else(|| pse_backend_native::ProblemError::memory("dynamic point arithmetic result"))?;
        let _result = budget.charge(retained)
            .map_err(|_| pse_backend_native::ProblemError::memory("dynamic point arithmetic result"))?;
        if projection.retained_bytes().saturating_add(workspace).saturating_add(retained) > graph_available {
            return Err(pse_backend_native::ProblemError::memory("dynamic point arithmetic reservation"));
        }
        let mut execution = pse_backend_native::solve::Execution::within(
            scope.cancellation().clone(), &pse_backend_native::solve::Controls::default(), scope.clone())?;
        if let Some(admission) = budget.admission() {
            execution.work_admission = Some(admission.clone());
        }
        let mut assessment_owner = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        assessment_owner.str("dynamic-point-arithmetic-assessment")
            .hash(&self.key).id(&goal.target_id).u64(goal_index as u64).hash(&point_identity(point))
            .hash(&controls_identity(profile)).u64(occurrence);
        let assessment_owner = assessment_owner.finish_hash();
        let arithmetic = Ibex.enclose_arithmetic_box_values_with_execution(
            &projection, &point_regions, workspace, &execution,
        );
        record_point_stage(goal.target_id, "IBEX value enclosure invoked".into());
        let actual_work = arithmetic.as_ref().ok().map(|evidence| match evidence {
            PointArithmeticEvidence::Enclosed { work, .. }
            | PointArithmeticEvidence::Incomplete { work, .. }
            | PointArithmeticEvidence::Interrupted { work } => *work,
        });
        let evaluations = actual_work.and_then(|work| {
            work.guard_evaluations.checked_add(work.value_evaluations)?
                .checked_add(work.jacobian_evaluations)?.checked_add(work.hessian_evaluations)
        });
        if let Some(admission) = budget.admission() {
            charges.push(complete_dynamic_assessment(&admission, assessment_owner, evaluations)?);
        }
        let evidence = arithmetic?;
        scope.check().map_err(pse_backend_native::ProblemError::from)?;
        let (enclosed, witness_key) = match evidence {
            PointArithmeticEvidence::Enclosed { values, .. } => (values, projection.graph.identity()),
            PointArithmeticEvidence::Interrupted { .. } => return Err(pse_backend_native::ProblemError::Cancelled),
            PointArithmeticEvidence::Incomplete { reason, .. } => {
                record_point_stage(goal.target_id, format!("IBEX enclosure incomplete: {reason:?}"));
                use pse_math::implicit::SelectionProofRefusal as R;
                return match reason {
                    R::Resource => Err(pse_backend_native::ProblemError::memory("dynamic point arithmetic limit")),
                    R::Boundary => Ok(None),
                    R::Unsupported | R::Chart | R::Coverage => Ok(None),
                };
            }
        };
        let row = selected_plan.structure().rows().iter().position(|row| row.id == row_id)
            .ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic selected output row absent".into()))?;
        let output = projection.rows.get(row).map(|row| row.value)
            .ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic output arithmetic row absent".into()))?;
        let raw = *enclosed.get(output).ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic output interval absent".into()))?;
        if !raw.valid() { return Ok(None); }
        let scale = program.scales.get(local).copied().ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic output scale absent".into()))?;
        let offset = program.offsets.get(local).copied().ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic output offset absent".into()))?;
        let scaled = interval_mul(raw.lower, scale).zip(interval_mul(raw.upper, scale)).and_then(|(left, right)| {
            let bounds = pse_math::implicit::ProofInterval {
                lower: left.lower.min(right.lower), upper: left.upper.max(right.upper),
            };
            interval_add(bounds, pse_math::implicit::ProofInterval { lower: offset, upper: offset })
        });
        let observed = self.dynamic_goal_value(goal, point).ok_or_else(|| pse_backend_native::ProblemError::Contract("dynamic output observation absent".into()))?;
        let Some(error) = scaled.and_then(|interval| interval_error(interval, observed)) else { return Ok(None); };
        let mut witness = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        witness.str("dynamic-authored-point-arithmetic").hash(&projection.key).hash(&witness_key)
            .id(&goal.target_id).part(&(row as u64).to_le_bytes());
        drop(owner);
        Ok(Some((error, witness.finish_hash())))
    }
}

/// Later controls come from actual unresolved output demands, never another fixed ladder.
fn refinement_ratio(results: &[GoalResult]) -> Result<Option<f64>, Unavailable> {
    if results
        .iter()
        .any(|r| r.classification.status == AccuracyGoalStatus::Violated
            || r.goal.use_policy == pse_model::generated::enums::AccuracyGoalUse::RequireSatisfied
                && r.classification.criterion == pse_model::generated::enums::AccuracyCriterionStatus::Violated)
    {
        return Ok(None);
    }
    let mut ratio: Option<f64> = None;
    for row in results.iter().filter(|r| {
        r.classification.status == AccuracyGoalStatus::Unresolved
            && r.goal.refine
            && r.evidence.is_some()
    }) {
        let e = row.evidence.as_ref().ok_or(Unavailable::MissingEvidence)?;
        let goal = BoundGoal {
            declaration: row.goal.clone(),
            source: e.source,
            product: e.accuracy.product,
            normalization: e.accuracy.normalization,
        };
        let value = e.value.ok_or(Unavailable::MissingObservation)?;
        let allowance = pse_math::engineering_accuracy::refinement_allowance(&goal, value)
            .ok_or(Unavailable::Boundary)?;
        let error = row
            .classification
            .error
            .ok_or(Unavailable::MissingEvidence)?;
        let demand = allowance / error;
        if !demand.is_finite() || demand <= 0. || demand >= 1. {
            return Err(Unavailable::PrecisionLimit);
        }
        ratio = Some(ratio.map_or(demand, |prior| prior.min(demand)));
    }
    Ok(ratio)
}

#[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
impl ModelingSimulation {
    fn optional_accuracy_failure(&self, error: pse_backend_native::ProblemError)
        -> Result<Vec<GoalResult>, crate::math::MathRuntimeError> {
        if !crate::math::solves::engineering_accuracy::optional_failure(&error) {
            return Err(error.into());
        }
        let retained = error.retained_bytes();
        let cause = pse_model::diagnostic::DiagnosticCause::new(error);
        let bytes = retained.saturating_add(cause.allocation_overhead())
            .saturating_add(size_of::<pse_model::diagnostic::DiagnosticCause>());
        let owner = self.runtime.shared.math().reserve("math:dynamic-accuracy-failure", bytes)?;
        let failure = pse_math::engineering_accuracy::GoalFailure::new(cause, bytes).with_owner(owner);
        Ok(self.numerics().policy.goals.iter().map(|goal| {
            let mut result = GoalResult::unavailable(goal.clone(), Unavailable::EvaluatorUncertainty);
            result.failure = Some(failure.clone());
            result
        }).collect())
    }

    fn original_accuracy_admission(
        &self,
        report: &native::Report,
        checks: &checks::SampleChecks,
    ) -> bool {
        let coverage = report.assess_endpoint(&self.profile);
        let mut original = self.numerics().policy.clone();
        original.goals.clear();
        crate::workflow::numerics::complete(
            crate::workflow::numerics::trajectory_use(report, coverage.satisfied),
            crate::workflow::numerics::CompletionEvidence {
                checks: &checks.rows,
                checks_complete: checks.complete && checks.error.is_none(),
                required_closure: self.required_closure_checks(),
                endpoint_satisfied: Some(coverage.satisfied),
                coverage_complete: coverage.prefix_complete,
                accuracy: &[],
            },
            &original,
        )
        .permits_use()
    }

    fn comparator_failure(
        report: &mut native::Report,
        checks: &checks::SampleChecks,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<Option<Unavailable>, pse_backend_native::ProblemError> {
        scope.check().map_err(pse_backend_native::ProblemError::from)?;
        match report.termination {
            native::Termination::Completed | native::Termination::Event => {
                if let Some(error) = report.error.take() {
                    return Err(error);
                }
                if let Some(error) = &checks.error {
                    Err(pse_backend_native::ProblemError::Provider(
                        pse_kernels::ProviderError::Nested {
                            cause: pse_model::diagnostic::DiagnosticCause::new(error.clone()),
                            retained: size_of::<pse_model::diagnostic::BoundaryDiagnostic>()
                                .checked_add(error.heap_bytes())
                                .ok_or_else(|| pse_backend_native::ProblemError::memory("dynamic check diagnostic extent"))?,
                        },
                    ))
                } else {
                    Ok(None)
                }
            }
            native::Termination::Cancelled => {
                Err(pse_backend_native::ProblemError::Cancelled)
            }
            native::Termination::TimeLimit => {
                Err(pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    detail: "dynamic accuracy comparator time limit".into(),
                })
            }
            native::Termination::StepLimit | native::Termination::EventLimit => {
                Err(pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Work,
                    detail: "dynamic accuracy comparator step or event limit".into(),
                })
            }
            native::Termination::Failed => Err(report.error.take()
                .unwrap_or_else(|| pse_backend_native::ProblemError::Internal(
                    "dynamic accuracy comparator failed without a structured cause".into(),
                ))),
            native::Termination::Panic => {
                let error = report.error.take();
                if let Some(error) = error.filter(|error|
                    !crate::math::solves::engineering_accuracy::optional_failure(error))
                {
                    Err(error)
                } else {
                    Err(pse_backend_native::ProblemError::Internal(
                        "dynamic accuracy comparator panicked".into(),
                    ))
                }
            }
        }
    }

    fn dynamic_goal(
        &self,
        goal: &AccuracyGoal,
        point: &native::Sample,
    ) -> Result<(BoundGoal, f64), Unavailable> {
        if goal.subject != AccuracyGoalSubject::SelectedOutput {
            return Err(Unavailable::UnsupportedObservation);
        }
        if goal.required_class == NumericalAccuracyClass::Certified {
            return Err(Unavailable::InsufficientStrength);
        }
        let symbol = self
            .model()
            .compiled()
            .model
            .symbols
            .get(&goal.target_id)
            .ok_or(Unavailable::Unsupported)?;
        let pse_modeling::Type::Quantity(q) = &symbol.ty else {
            return Err(Unavailable::Unsupported);
        };
        let quantity = q
            .resolve(&self.quantities, &BTreeMap::new())
            .map_err(|_| Unavailable::Unsupported)?;
        let unit = self
            .quantities
            .quantity_type(quantity)
            .map_err(|_| Unavailable::Unsupported)?
            .canonical_unit;
        let targets = [pse_math::numerics::TargetSpec {
            id: goal.target_id,
            kind: goal.target_kind,
            quantity,
            unit,
            integer: false,
            declared_tolerance: None,
        }];
        let declaration = pse_math::engineering_accuracy::bind_goals(
            &self.quantities,
            &targets,
            std::slice::from_ref(goal),
            &[],
        )
        .map_err(|_| Unavailable::Unsupported)?
        .into_iter()
        .next()
        .ok_or(Unavailable::Unsupported)?;
        let value = match (goal.observation, goal.target_kind) {
            (AccuracyObservation::Integrated, _) => {
                // The trajectory owns a cumulative flux result, but currently exposes no
                // bound on accumulated callback/roundoff error. An endpoint flux enclosure
                // cannot stand in for that integral error.
                return Err(Unavailable::EvaluatorUncertainty);
            }
            (_, NumericalTarget::Variable) => {
                let index = self.contract.states.iter().position(|id| *id == goal.target_id)
                    .ok_or(Unavailable::UnsupportedObservation)?;
                let binding = self.coordinates.state.get(index).ok_or(Unavailable::MissingObservation)?;
                point.state.get(index).copied().ok_or(Unavailable::MissingObservation)?
                    * binding.scale + binding.offset
            }
            (_, NumericalTarget::Observable) => {
                let output = ModelingOutput::Member(goal.target_id).row_id();
                let i = self.contract.outputs.iter().position(|id| *id == output)
                    .ok_or(Unavailable::UnsupportedObservation)?;
                point.outputs.get(i).copied().ok_or(Unavailable::MissingObservation)?
            }
            _ => return Err(Unavailable::UnsupportedObservation),
        };
        let point_key = point_identity(point);
        let mut parameters = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        parameters.str("dynamic-consumed-parameters");
        for v in &self.parameters {
            parameters.part(&v.to_bits().to_le_bytes());
        }
        let parameters = parameters.finish_hash();
        let mut product = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        product
            .str("dynamic-output-error")
            .hash(&self.contract.identity)
            .part(goal.target_id.as_bytes())
            .str(goal.observation.as_str())
            .hash(&point_key);
        let product = product.finish_hash();
        let mut normalization = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        normalization
            .str("physical-output-error-unit")
            .part(quantity.as_id().as_bytes())
            .part(unit.as_id().as_bytes());
        let normalization = normalization.finish_hash();
        Ok((
            BoundGoal {
                declaration,
                product,
                normalization,
                source: SemanticProductKey {
                    structure: self.contract.identity,
                    binding: self.key,
                    numerical_policy: Some(self.numerics().key),
                    normalization: Some(normalization),
                    point: Some(point_key),
                    parameters: Some(parameters),
                    derivation: None,
                    branch: None,
                    accuracy: None,
                },
            },
            value,
        ))
    }

    fn dynamic_goal_value(&self, goal: &AccuracyGoal, point: &native::Sample) -> Option<f64> {
        match (goal.observation, goal.target_kind) {
            (AccuracyObservation::Integrated, _) => None,
            (_, NumericalTarget::Variable) => {
                let index = self.contract.states.iter().position(|id| *id == goal.target_id)?;
                let binding = self.coordinates.state.get(index)?;
                Some(*point.state.get(index)? * binding.scale + binding.offset)
            }
            (_, NumericalTarget::Observable) => {
                let id = ModelingOutput::Member(goal.target_id).row_id();
                let index = self.contract.outputs.iter().position(|candidate| *candidate == id)?;
                point.outputs.get(index).copied()
            }
            _ => None,
        }
    }

    fn capture_dynamic_goals(&self, report: &native::Report, profile: &native::Profile,
        occurrence: u64, scope: &pse_kernels::ExecutionScope,
        budget: &Arc<crate::math::WorkerBudget>, charges: &mut Vec<pse_model::strategy::WorkCharge>)
        -> Result<CapturedTrajectory, pse_backend_native::ProblemError> {
        let mut goals = Vec::with_capacity(self.numerics().policy.goals.len());
        for (goal_index, goal) in self.numerics().policy.goals.iter().enumerate() {
            let captured = (|| -> Result<GoalPoint, GoalBuildError> {
                let point = observation(goal, report)?;
                let (bound, value) = self.dynamic_goal(goal, point)?;
                let Some((uncertainty, arithmetic)) = self.point_uncertainty(
                    goal, goal_index, report, point, profile, occurrence, scope, budget, charges,
                )? else {
                    return Err(Unavailable::EvaluatorUncertainty.into());
                };
                Ok(GoalPoint { bound, value, uncertainty, arithmetic,
                    point: ObservationPoint { mode: point.mode, time: point.time } })
            })();
            goals.push(match captured {
                Ok(point) => Ok(point),
                Err(GoalBuildError::Unavailable(reason)) => Err(reason),
                Err(GoalBuildError::Failure(error)) => return Err(error),
            });
        }
        Ok(CapturedTrajectory { shape: trajectory_shape(report), goals })
    }

    fn paired_accuracy(&self, base: &CapturedTrajectory, contrasting: &CapturedTrajectory,
        base_controls: ContentHash, contrasting_controls: ContentHash)
        -> Vec<GoalResult> {
        self.numerics().policy.goals.iter().zip(&base.goals).zip(&contrasting.goals)
            .map(|((goal, left), right)| {
                let result = (|| -> Result<GoalResult, Unavailable> {
                    let left = left.as_ref().map_err(|reason| *reason)?;
                    let right = right.as_ref().map_err(|reason| *reason)?;
                    let receipt = correspondence_shape(&base.shape, &contrasting.shape,
                        left.point, right.point, goal.observation)?;
                    let mut bound = left.bound.clone();
                    bound.source.branch = Some(receipt);
                    let pair = pse_backend_native::engineering_accuracy::Comparison {
                        source: bound.source, base: left.value, contrasting: right.value,
                        correspondence: receipt, base_controls, contrasting_controls,
                        base_arithmetic: left.arithmetic, contrasting_arithmetic: right.arithmetic,
                        base_uncertainty: left.uncertainty, contrasting_uncertainty: right.uncertainty,
                        resolution_floor: representational_floor(left.value)?.max(representational_floor(right.value)?),
                    };
                    let mut evidence = pse_backend_native::engineering_accuracy::dynamic_comparison(&bound, pair)?;
                    evidence.limitation.push_str("; direct authored output, original checks on both trajectories; native max_steps is per integration; native counters remain report-owned; shared deadline and memory admission");
                    Ok(GoalResult::assess(&bound, Some(evidence)))
                })();
                result.unwrap_or_else(|reason| GoalResult::unavailable(goal.clone(), reason))
            }).collect()
    }

    pub(super) fn assess_dynamic_accuracy(
        &self,
        run_id: RunId,
        mut report: native::Report,
        mut checks: checks::SampleChecks,
        scope: &pse_kernels::ExecutionScope,
        flag: native::Cancellation,
        progress: Arc<pse_backend_native::solve::Progress>,
        admission: Option<Arc<crate::math::strategy::admission::TaskAdmission>>,
        mut charges: Vec<pse_model::strategy::WorkCharge>,
    ) -> Result<
        (native::Report, checks::SampleChecks, Vec<GoalResult>),
        crate::math::MathRuntimeError,
    > {
        let (mut report, checks, accuracy) = self.assess_dynamic_accuracy_inner(
            run_id, report, checks, scope, flag, progress, admission, &mut charges,
        )?;
        if !charges.is_empty() {
            report.statistics.push(serde_json::json!({ "accuracy_work_charges": charges }));
        }
        Ok((report, checks, accuracy))
    }

    fn assess_dynamic_accuracy_inner(
        &self,
        run_id: RunId,
        mut report: native::Report,
        mut checks: checks::SampleChecks,
        scope: &pse_kernels::ExecutionScope,
        flag: native::Cancellation,
        progress: Arc<pse_backend_native::solve::Progress>,
        admission: Option<Arc<crate::math::strategy::admission::TaskAdmission>>,
        charges: &mut Vec<pse_model::strategy::WorkCharge>,
    ) -> Result<
        (native::Report, checks::SampleChecks, Vec<GoalResult>),
        crate::math::MathRuntimeError,
    > {
        let goals = &self.numerics().policy.goals;
        if goals.is_empty() {
            return Ok((report, checks, Vec::new()));
        }
        let admission = admission.ok_or_else(|| pse_backend_native::ProblemError::Internal(
            "dynamic accuracy has no task admission".into(),
        ))?;
        let unavailable = |reason| {
            goals
                .iter()
                .cloned()
                .map(|g| GoalResult::unavailable(g, reason))
                .collect()
        };
        if !self.original_accuracy_admission(&report, &checks) {
            return Ok((report, checks, unavailable(Unavailable::Failed)));
        }
        let budget = crate::math::WorkerBudget::new(self.bytes)
            .with_admission(admission.clone());
        let base_capture = match self.capture_dynamic_goals(&report, &self.profile, 0, scope, &budget, charges) {
            Ok(captured) => captured,
            Err(error) if crate::math::solves::engineering_accuracy::optional_failure(&error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
            Err(error) => return Err(error.into()),
        };
        if base_capture.goals.iter().all(Result::is_err) {
            let rows = goals.iter().zip(&base_capture.goals).map(|(goal, result)| {
                GoalResult::unavailable(
                    goal.clone(),
                    result.as_ref().err().copied().unwrap_or(Unavailable::MissingEvidence),
                )
            }).collect();
            return Ok((report, checks, rows));
        }
        let contrast = match contrasting_profile(&self.profile, 0.5) {
            Ok(p) => p,
            Err(reason) => return Ok((report, checks, unavailable(reason))),
        };
        // Reserve transient native/report/check storage before constructing another worker.
        // One lease serves all goals. It is dropped before the final report escapes.
        let _comparator = self
            .runtime
            .shared
            .math()
            .reserve("math:dynamic-goal-comparator", self.bytes)?;
        let started = std::time::Instant::now();
        let integrate = |profile: &native::Profile, occurrence: i64, ratio: f64|
            -> Result<(Result<native::Report, pse_backend_native::ProblemError>, pse_model::strategy::WorkCharge), pse_backend_native::ProblemError> {
            progress.push(pse_backend_native::solve::Event {
                phase: "accuracy.dynamic.comparator".into(),
                elapsed: started.elapsed(),
                values: BTreeMap::from([
                    (
                        "occurrence".into(),
                        pse_backend_native::solve::Metric::Integer(occurrence),
                    ),
                    (
                        "control_ratio".into(),
                        pse_backend_native::solve::Metric::Real(ratio),
                    ),
                ]),
                incumbent: None,
            });
            admission.reserve_attempt()?;
            let mut worker = self.worker(scope.clone())?;
            let outcome = native::integrate_with_progress_observed(
                &mut worker,
                profile,
                &self.parameters,
                flag.clone(),
                progress.clone(),
                &self.snapshot,
            );
            drop(worker);
            let charge = complete_dynamic_attempt(&admission, self.key, occurrence as u64)?;
            Ok((outcome, charge))
        };
        let (other_outcome, other_charge) = match integrate(&contrast, 1, 0.5) {
            Ok(result) => result,
            Err(error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
        };
        charges.push(other_charge);
        let mut other = match other_outcome {
            Ok(report) => report,
            Err(error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
        };
        let other_checks = self.check_samples(run_id, &other, &self.parameters, scope);
        let comparator_result = Self::comparator_failure(&mut other, &other_checks, scope);
        let comparator_reason = match comparator_result {
            Ok(reason) => reason,
            Err(error) if crate::math::solves::engineering_accuracy::optional_failure(&error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
            Err(error) => return Err(error.into()),
        };
        if let Some(reason) = comparator_reason {
            return Ok((report, checks, unavailable(reason)));
        }
        if !self.original_accuracy_admission(&other, &other_checks) {
            return Ok((report, checks, unavailable(Unavailable::Failed)));
        }
        let base_controls = controls_identity(&self.profile);
        let contrast_controls = controls_identity(&contrast);
        let contrast_capture = match self.capture_dynamic_goals(&other, &contrast, 1, scope, &budget, charges) {
            Ok(captured) => captured,
            Err(error) if crate::math::solves::engineering_accuracy::optional_failure(&error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
            Err(error) => return Err(error.into()),
        };
        let mut results = self.paired_accuracy(&base_capture, &contrast_capture, base_controls, contrast_controls);
        report
            .statistics
            .push(serde_json::json!({ "accuracy_occurrence": 1,
            "controls": native::profile_json(&contrast), "native_statistics": &other.statistics }));
        drop(other_checks);
        drop(other);
        let ratio = match refinement_ratio(&results) {
            Ok(Some(ratio)) => ratio,
            Ok(None) => return Ok((report, checks, results)),
            Err(reason) => {
                for r in &mut results {
                    if r.goal.refine && r.classification.status == AccuracyGoalStatus::Unresolved {
                        r.classification.unavailable = Some(reason);
                    }
                }
                return Ok((report, checks, results));
            }
        };
        let refined_controls = match contrasting_profile(&contrast, ratio) {
            Ok(p) => p,
            Err(reason) => {
                for r in &mut results {
                    if r.goal.refine && r.classification.status == AccuracyGoalStatus::Unresolved {
                        r.classification.unavailable = Some(reason);
                    }
                }
                return Ok((report, checks, results));
            }
        };
        // Retain the admitted result until its replacement has passed original checks.
        let (refined_outcome, refined_charge) = match integrate(&refined_controls, 2, ratio) {
            Ok(result) => result,
            Err(error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
        };
        charges.push(refined_charge);
        let mut refined = match refined_outcome {
            Ok(report) => report,
            Err(error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
        };
        let refined_checks = self.check_samples(run_id, &refined, &self.parameters, scope);
        let comparator_result = Self::comparator_failure(&mut refined, &refined_checks, scope);
        let comparator_reason = match comparator_result {
            Ok(reason) => reason,
            Err(error) if crate::math::solves::engineering_accuracy::optional_failure(&error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
            Err(error) => return Err(error.into()),
        };
        if let Some(reason) = comparator_reason {
            for row in &mut results {
                if row.classification.status == AccuracyGoalStatus::Unresolved {
                    row.classification.unavailable = Some(reason);
                }
            }
            return Ok((report, checks, results));
        }
        if !self.original_accuracy_admission(&refined, &refined_checks) {
            for row in &mut results {
                if row.classification.status == AccuracyGoalStatus::Unresolved {
                    row.classification.unavailable = Some(Unavailable::Failed);
                }
            }
            return Ok((report, checks, results));
        }
        let refined_capture = match self.capture_dynamic_goals(&refined, &refined_controls, 2, scope, &budget, charges) {
            Ok(captured) => captured,
            Err(error) if crate::math::solves::engineering_accuracy::optional_failure(&error) => {
                let rows = self.optional_accuracy_failure(error)?;
                return Ok((report, checks, rows));
            }
            Err(error) => return Err(error.into()),
        };
        let mut refined_results = self.paired_accuracy(
            &refined_capture,
            &contrast_capture,
            controls_identity(&refined_controls),
            contrast_controls,
        );
        for (row, prior) in refined_results.iter_mut().zip(&results) {
            if row.goal.refine && row.classification.status == AccuracyGoalStatus::Unresolved
                && row.evidence.is_some() && prior.evidence.is_some()
            {
                row.classification.unavailable = Some(
                    if row
                        .classification
                        .error
                        .zip(prior.classification.error)
                        .is_some_and(|(next, previous)| next < previous)
                    {
                        Unavailable::Budget
                    } else {
                        Unavailable::Nonprogress
                    },
                );
            }
            if let Some(e) = &mut row.evidence {
                e.limitation.push_str("; one shared binary contrast and one demand-directed refinement; no convergence-order assertion");
            }
        }
        let prior_statistics = std::mem::take(&mut report.statistics);
        report = refined;
        report.statistics.push(serde_json::json!({ "accuracy_original_run": true,
            "controls": native::profile_json(&self.profile), "native_statistics": prior_statistics }));
        report
            .statistics
            .push(serde_json::json!({ "accuracy_occurrence": 2,
            "control_ratio": ratio, "controls": native::profile_json(&refined_controls) }));
        checks = refined_checks;
        Ok((report, checks, refined_results))
    }
}
