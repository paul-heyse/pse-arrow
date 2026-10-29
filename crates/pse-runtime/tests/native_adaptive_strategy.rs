// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! External native construction of the retained EGObox actor and original correction.
//! Requires solver-kinsol and explicit correctness force-validation.
//! Resume means further calls to the same actor, never reconstruction from a snapshot.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "public scientific fixture and independent analytic assertions fail directly"
)]

#[path = "reference_support/mod.rs"]
mod reference_support;

use futures_util::FutureExt;
use pse_backend_native::{presolve::PolicyKind, solve::{Backend, SolveIntent}, ProblemError};
use pse_ids::{ContentHash, Frame, FramedHasher};
use pse_kernels::ExecutionScope;
use pse_model::strategy::{BranchPolicy, StartOrigin};
use pse_relations::{columnar::RelationRow, generated::runtime::solve_variables};
use pse_runtime::{
    CancelSource,
    math::{
        MathRuntimeError,
        settings::SolveSettings,
        surrogate::{CorrelationSpec, FidelityCorrespondence, FidelityEvaluator, RegressionSpec,
            SurrogateOptions, ThetaTuning},
    },
    workflow::{ModelingAnalysis, ModelingSolvePreparation},
};
use reference_support::{NativeContext, Result};
use std::{panic::AssertUnwindSafe, sync::{Arc, atomic::{AtomicUsize, Ordering}},
    time::{Duration, Instant}};

const SOURCE: &str = "package adaptive { def Root { param p:Scalar=4.4; var x:Scalar;
    eq root:x*x==p; annotation start x(2); }}";
const APPROXIMATION: &str = "declared approximate root merit (x-2.02)^2 in Scalar coordinates";

fn source_hash(source: &str) -> ContentHash {
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str(source);
    hash.finish_hash()
}

#[derive(Debug)]
struct ApproximateRoot {
    correspondence: FidelityCorrespondence,
    calls: Arc<AtomicUsize>,
    fail_at: Option<usize>,
}
impl FidelityEvaluator for ApproximateRoot {
    type Error = ProblemError;
    fn correspondence(&self) -> &FidelityCorrespondence { &self.correspondence }
    fn evaluate(&mut self, point: &[f64], outputs: &mut [f64]) -> std::result::Result<(), ProblemError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if self.fail_at == Some(call) {
            return Err(ProblemError::Math(pse_math::MathError::Domain {
                source_id: self.correspondence.coordinates[0],
                requirement: "declared approximate-root evaluation refused",
            }));
        }
        outputs[0] = (point[0]-2.02).powi(2);
        Ok(())
    }
}

async fn target(context: &NativeContext) -> Result<ModelingSolvePreparation> {
    let package = context.package(SOURCE).await?;
    let services = context.runtime.preparation_services();
    let root = package.declarations(&services).await?.iter()
        .find(|row| row.name == "Root").unwrap().declaration_id;
    let mut settings = SolveSettings {
        backend: Some(Backend::Kinsol), intent: SolveIntent::Root,
        presolve: PolicyKind::Off, ..Default::default()
    };
    settings.composition.recovery.push(StartOrigin::Surrogate);
    let request = ModelingAnalysis {
        root, instance: pse_modeling::specialize::root_instance(root),
        bindings: Default::default(), limits: Default::default(), case: Default::default(),
        order: pse_kernels::DerivativeOrder::First, compiler: Default::default(),
        solver: settings.profile()?, numerical: Default::default(),
    };
    Ok(Box::pin(package.prepare_analysis(&services, &request, &CancelSource::new())).await?)
}

fn correspondence(target: &ModelingSolvePreparation) -> Result<FidelityCorrespondence> {
    let plan = &target.model.case.compiled().plan;
    let [coordinate] = plan.columns() else { panic!("one actual original root coordinate") };
    let [row] = plan.structure().rows() else { panic!("one actual original root equation") };
    let identity = target.solve.original_identity()?;
    Ok(FidelityCorrespondence {
        original_target: identity, model_target: identity,
        source: source_hash(SOURCE), fidelity: source_hash(APPROXIMATION),
        coordinates: vec![*coordinate], outputs: vec![row.id],
    })
}

fn options(context: &NativeContext) -> SurrogateOptions {
    SurrogateOptions {
        bounds: vec![(1.,3.)], initial: vec![vec![1.],vec![2.02],vec![3.]],
        iterations: 2, evaluations: 20, infill_starts: 1,
        regression: RegressionSpec::CONSTANT, correlation: CorrelationSpec::MATERN52,
        theta: ThetaTuning::Fixed(vec![0.1].into()), gp_starts: 0, gp_evaluations: 5,
        seed: 74, phase_steps: (1,1), radius: 0.2, contraction: 0.5,
        threads: 1, stack_bytes: context.shared.budget().math.stack_bytes,
        foreign_bytes: 1 << 20, time: Duration::from_secs(30),
    }
}

fn scope() -> ExecutionScope {
    ExecutionScope::new(pse_diagnostics::Cancellation::default(),
        Some(Instant::now()+Duration::from_secs(30)))
}

fn finish(outcome: std::thread::Result<Result<()>>, closed: Result<()>) -> Result<()> {
    match outcome {
        Ok(result) => { closed?; result }
        Err(panic) => { let _ = closed; std::panic::resume_unwind(panic) }
    }
}

#[tokio::test]
async fn native_actor_resumes_actual_history_then_original_root_corrector_grants_permission() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let mut target = target(&context).await?;
        let correspondence = correspondence(&target)?;
        let variable = correspondence.coordinates[0];
        let calls = Arc::new(AtomicUsize::new(0));
        let cancel = CancelSource::new();
        let scope = scope();
        let service = context.runtime.native();
        let handle = service.prepare_surrogate(ApproximateRoot {
            correspondence: correspondence.clone(), calls: calls.clone(), fail_at: None,
        }, options(&context), scope.clone(), &cancel).await?;
        assert_eq!(calls.load(Ordering::SeqCst), 0, "preparation does not invent observations");
        assert!(handle.proposal()?.is_none());
        assert!(handle.retained_bytes() > 0);
        let first = service.advance_surrogate(&handle, &cancel).await?;
        assert!(first.state.initialized && !first.state.disabled);
        assert_eq!(first.state.iteration, 0);
        assert_eq!(first.work.initializations, 1);
        assert_eq!(first.work.evaluations, 3);
        let history = handle.snapshot(&pse_columnar::CancellationToken::new())?;
        assert!(history.retained_bytes() > 0);
        assert_eq!(history.state().unwrap().iter, 0);
        assert!(history.state().unwrap().rng.is_some());
        let cancelled_snapshot = pse_columnar::CancellationToken::new();
        cancelled_snapshot.cancel();
        assert!(matches!(handle.snapshot(&cancelled_snapshot), Err(MathRuntimeError::Cancelled)));

        // A cancelled snapshot is independent of the retained task. Advancing
        // again consumes its actual RNG/training state rather than replaying DOE.
        let resumed = service.advance_surrogate(&handle, &cancel).await?;
        assert_eq!(resumed.state.iteration, 1);
        assert_eq!(resumed.work.initializations, 1);
        assert_eq!(resumed.work.iterations, 1);
        assert_eq!(resumed.work.evaluations, calls.load(Ordering::SeqCst) as u64);
        assert!(resumed.work.evaluations > first.work.evaluations);
        assert_eq!(history.state().unwrap().iter, 0, "owned history remains a point-in-time copy");
        let trained = handle.snapshot(&pse_columnar::CancellationToken::new())?;
        let state = trained.state().unwrap();
        assert_eq!(state.iter, 1);
        let (inputs, outputs, _) = state.surrogate.data.as_ref().unwrap();
        assert!(inputs.nrows() > 3);
        assert_eq!(inputs.nrows(), outputs.nrows());
        for row in 0..inputs.nrows() {
            let expected = (inputs[[row,0]]-2.02).powi(2);
            assert!((outputs[[row,0]]-expected).abs() <= 1e-12,
                "every completed training observation is the actual declared fidelity");
        }
        let statistical = resumed.proposal.unwrap();
        assert_eq!(statistical.task, handle.key());
        assert_eq!(statistical.original_target, correspondence.original_target);
        assert_eq!(statistical.fidelity, correspondence.fidelity);
        let proposal = target.solve.surrogate_start(&statistical, &correspondence,
            handle.key(), BranchPolicy::any_qualified())?;
        assert!(proposal.source().accuracy.is_none(), "statistical merit grants no numerical accuracy");
        let screened = service.screen_start(target.solve.clone(), proposal,
            BranchPolicy::any_qualified(), scope, &cancel).await?;
        target.solve = target.solve.with_screened_start(&screened)?;
        let run = target.start(&context.runtime.execution_services())?.wait().await?;
        assert!(run.usable(), "{:?}", run.report());
        let variables = solve_variables::Row::rows(&run.table("runtime.solve_variables")?)?;
        let corrected = variables.iter().find(|row| row.symbol_id == variable).unwrap();
        let x = corrected.value.unwrap();
        let allowance = corrected.tolerance.unwrap();
        assert!(allowance.is_finite() && allowance > 0.);
        assert!((x-4.4_f64.sqrt()).abs() <= allowance,
            "original corrector must solve x*x=4.4: x={x}, allowance={allowance}");
        assert!((x-2.02).abs() > 0.01, "the approximate root cannot become original truth");

        let entered = calls.load(Ordering::SeqCst);
        handle.cancel();
        assert!(matches!(service.advance_surrogate(&handle, &cancel).await,
            Err(MathRuntimeError::Cancelled)));
        assert_eq!(calls.load(Ordering::SeqCst), entered);
        assert_eq!(handle.work()?.evaluations, resumed.work.evaluations);
        drop(handle);
        assert_eq!(trained.state().unwrap().iter, 1, "owned snapshot survives actor destruction");
        Ok(())
    }).catch_unwind().await;
    finish(outcome, context.close().await)
}

#[tokio::test]
async fn native_actor_retains_failed_callback_work_without_imputed_observation_or_retry() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let target = target(&context).await?;
        let correspondence = correspondence(&target)?;
        let calls = Arc::new(AtomicUsize::new(0));
        let cancel = CancelSource::new();
        let service = context.runtime.native();
        let handle = service.prepare_surrogate(ApproximateRoot {
            correspondence, calls: calls.clone(), fail_at: Some(2),
        }, options(&context), scope(), &cancel).await?;
        let error = service.advance_surrogate(&handle, &cancel).await.unwrap_err();
        assert!(matches!(error, MathRuntimeError::Solve(ProblemError::Math(
            pse_math::MathError::Domain { requirement: "declared approximate-root evaluation refused", .. }
        ))), "original typed callback cause must survive: {error:?}");
        let work = handle.work()?;
        assert_eq!(work.evaluations, 2, "include the failed callback, exclude unentered DOE points");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(work.initializations, 1);
        assert_eq!(work.iterations, 0);
        assert!(handle.proposal()?.is_none(), "failed initialization cannot publish an imputed optimum");
        let history = handle.snapshot(&pse_columnar::CancellationToken::new())?;
        assert!(history.state().is_none(), "no complete library training state was produced");
        assert!(service.advance_surrogate(&handle, &cancel).await.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 2, "terminal failures do not silently retry or impute");
        assert_eq!(handle.work()?.evaluations, 2);
        drop(handle);
        Ok(())
    }).catch_unwind().await;
    finish(outcome, context.close().await)
}
