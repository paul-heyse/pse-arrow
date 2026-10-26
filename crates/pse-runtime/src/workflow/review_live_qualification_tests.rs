// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One actual native callback remains owned while copied review is inspected/cancelled.
#![cfg(feature = "solver-kinsol")]
use super::*;
use crate::tooling::{
    frozen_inspection::{
        FrozenEvidenceSet, FrozenInspectionRequest, RecordedWindow, inspect_frozen,
    },
    inspection::InspectionRequest,
    review_analysis::{AnalysisRequest, analyze},
};
use pse_backend_native::{
    presolve,
    solve::{Backend, SolverSelection},
};
use pse_columnar::CancellationToken;
use pse_diagnostics::observation::{OperationId, ProducerRole};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{
    AdmittedProvider, DerivativeOrder, DerivativeSource, EvaluationContext, ExecutionScope, Port,
    PreparedProviderRequest, Provider, ProviderBinding, ProviderError, ProviderFactory,
    ProviderOutput, ProviderRuntime, ProviderShapes, ProviderSpec, Registration,
};
use pse_relations::{columnar::RelationRow, generated::runtime::solve_variables};
use std::sync::{
    Arc, Condvar, Mutex, Weak,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::Duration;

#[derive(Debug)]
struct CallbackGate {
    entered: tokio::sync::Notify,
    released: Mutex<bool>,
    changed: Condvar,
    armed: AtomicBool,
    fills: AtomicUsize,
    native: Weak<crate::math::MathService>,
    free_cpu: usize,
    free_jobs: usize,
    actual: Mutex<Option<(OperationId, Arc<AtomicBool>)>>,
}
impl CallbackGate {
    fn release(&self) {
        *self
            .released
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        self.changed.notify_all();
    }
    fn enter(&self, scope: &ExecutionScope) -> Result<(), ProviderError> {
        let native = self
            .native
            .upgrade()
            .ok_or_else(|| ProviderError::Contract("native owner absent".into()))?;
        let admitted = native.test_admission_snapshot();
        if admitted.1 >= self.free_cpu || admitted.2 >= self.free_jobs {
            return Err(ProviderError::Contract(
                "gate was reached without actual native CPU/job admission".into(),
            ));
        }
        let context = pse_diagnostics::observation::current().ok_or_else(|| {
            ProviderError::Contract("actual callback observation owner absent".into())
        })?;
        *self.actual.lock().map_err(|_| {
            ProviderError::Contract("live callback evidence lock poisoned".into())
        })? = Some((context.id(), scope.cancellation().clone()));
        self.entered.notify_one();
        let mut released = self
            .released
            .lock()
            .map_err(|_| ProviderError::Contract("live callback release lock poisoned".into()))?;
        while !*released {
            released = self.changed.wait(released).map_err(|_| {
                ProviderError::Contract("live callback release wait poisoned".into())
            })?;
        }
        scope.check()
    }
}
struct ReleaseCallback(Arc<CallbackGate>);
impl Drop for ReleaseCallback {
    fn drop(&mut self) {
        self.0.release();
    }
}
#[derive(Debug)]
struct IdentityFactory {
    spec: ProviderSpec,
    gate: Arc<CallbackGate>,
}
impl ProviderFactory for IdentityFactory {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn create(&self) -> Result<ProviderBinding, ProviderError> {
        Err(ProviderError::Contract(
            "the live control requires the original scoped native worker".into(),
        ))
    }
    fn create_scoped(&self, scope: ExecutionScope) -> Result<ProviderBinding, ProviderError> {
        scope.check()?;
        Ok(ProviderBinding::new(
            self.spec.clone(),
            Box::new(IdentityProvider {
                gate: self.gate.clone(),
                scope,
            }),
        ))
    }
}
#[derive(Debug)]
struct IdentityProvider {
    gate: Arc<CallbackGate>,
    scope: ExecutionScope,
}
impl Provider for IdentityProvider {
    fn fill(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        _runtime: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        request.check_context(context)?;
        self.scope.check()?;
        let ordinal = self.gate.fills.fetch_add(1, Ordering::AcqRel);
        // The first native callback completes a real checkpoint. The second
        // remains blocked; admission/preparation cannot enter an unarmed gate.
        if self.gate.armed.load(Ordering::Acquire) && ordinal == 1 {
            self.gate.enter(&self.scope)?;
        }
        output.values.fill(inputs[0]);
        output.jacobian.fill(1.);
        output.hessians.fill(0.);
        Ok(())
    }
}

#[tokio::test]
async fn live_review_of_gated_native_root_cancels_only_copied_reader_then_joins_original() {
    let runtime = tests::runtime();
    let baseline = runtime.native().test_admission_snapshot();
    let gate = Arc::new(CallbackGate {
        entered: Default::default(),
        released: Mutex::new(false),
        changed: Condvar::new(),
        armed: AtomicBool::new(false),
        fills: AtomicUsize::new(0),
        native: Arc::downgrade(runtime.native()),
        free_cpu: baseline.1,
        free_jobs: baseline.2,
        actual: Mutex::new(None),
    });
    let _release = ReleaseCallback(gate.clone());
    let physical = tests::physical();
    let quantity = physical.quantities.neutral_dimensionless().unwrap();
    let unit = physical
        .quantities
        .quantity_type(quantity)
        .unwrap()
        .canonical_unit;
    let port = |n| Port {
        id: SemanticId::from_bytes([n; 16]),
        quantity,
        unit,
    };
    let spec = ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        id: tests::id(91),
        revision: ContentHash::from_bytes([92; 32]),
        data: ContentHash::from_bytes([93; 32]),
        inputs: vec![port(94)],
        outputs: vec![port(95)],
        derivatives: DerivativeOrder::First,
        smoothness: DerivativeOrder::First,
    };
    let registration = Registration::bind(
        AdmittedProvider::new(spec.clone(), &physical.quantities).unwrap(),
        Arc::new(IdentityFactory {
            spec: spec.clone(),
            gate: gate.clone(),
        }),
    )
    .unwrap();
    let rows = pse_authoring::language::parse(&format!(
        "package review {{ fn law(x:Scalar)->Scalar external \"law\" revision \"{}\" data \"{}\" output 0 derivatives 1 source analytic smoothness 1; def Root {{var x:Scalar; eq original:law(x)==3; annotation start x(0);}} }}", spec.revision, spec.data,
    ), SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default()).unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package_registered(
            rows,
            physical,
            std::collections::BTreeMap::from([("law".into(), registration)]),
        )
        .await
        .unwrap();
    let mut solver = tests::profile();
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    solver.presolve = presolve::Policy::Off;
    solver.controls.time_limit = Duration::from_secs(10);
    assert_eq!(solver.numerics.engineering_relative_fraction, 0.001);
    let analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Default::default(),
        limits: Default::default(),
        case: Default::default(),
        order: DerivativeOrder::First,
        compiler: tests::compiler_profile(),
        solver,
        numerical: Default::default(),
    };
    let source_cancel = crate::CancelSource::new();
    let prepared = package
        .prepare_analysis(&runtime.preparation_services(), &analysis, &source_cancel)
        .await
        .unwrap();
    assert_eq!(
        gate.fills.load(Ordering::Acquire),
        0,
        "preparation must not execute the external law"
    );
    gate.armed.store(true, Ordering::Release);
    let handle = runtime
        .start_modeling(vec![prepared], false, &source_cancel)
        .await
        .unwrap();
    let entered = tokio::time::timeout(Duration::from_secs(5), gate.entered.notified()).await;
    // Catch assertion failures only to guarantee release AND original native join
    // before reporting them. This does not turn a failed observation into a pass.
    let reviewed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        entered.unwrap();
        assert!(handle.result().is_none());
        let (operation, native_cancel) = gate.actual.lock().unwrap().clone().unwrap();
        assert!(!native_cancel.load(Ordering::Acquire));
        let admitted = runtime.native().test_admission_snapshot();
        assert!(admitted.1 < baseline.1 && admitted.2 < baseline.2);
        let reader = runtime.observations().hub().reader(operation).unwrap();
        let snapshot = reader.snapshot(Duration::from_secs(60));
        assert!(snapshot.finished_at.is_none());
        assert!(snapshot.outcome.is_none());
        assert!(
            snapshot
                .producers
                .iter()
                .any(|producer| producer.current.is_some()
                    && matches!(
                        producer.descriptor.role,
                        ProducerRole::NativeCallback
                            | ProducerRole::NativeIteration
                            | ProducerRole::OriginalOracle
                            | ProducerRole::Evaluator
                    ))
        );
        let original = serde_json::to_vec(&snapshot).unwrap();
        let set = FrozenEvidenceSet {
            operations: vec![snapshot],
            ..Default::default()
        };
        let inspected = inspect_frozen(
            &set,
            FrozenInspectionRequest {
                version: pse_model::document::Version,
                inspection: InspectionRequest::new(operation),
                window: RecordedWindow::Recorded,
            },
        )
        .unwrap();
        assert_eq!(
            serde_json::to_vec(inspected.inspection.operation.as_ref().unwrap()).unwrap(),
            original
        );
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20));
        let request = AnalysisRequest::new(operation);
        let copied_reader = CancellationToken::new();
        let before = gate.fills.load(Ordering::Acquire);
        let document = analyze(&set, &request, &pool, &copied_reader).unwrap();
        assert_eq!(document.document().request.operation, operation);
        copied_reader.cancel();
        assert!(analyze(&set, &request, &pool, &copied_reader).is_err());
        assert!(!source_cancel.token().is_cancelled());
        assert!(!native_cancel.load(Ordering::Acquire));
        assert_eq!(gate.fills.load(Ordering::Acquire), before);
        assert!(handle.result().is_none());
        let still_owned = runtime.native().test_admission_snapshot();
        assert_eq!((still_owned.1, still_owned.2), (admitted.1, admitted.2));
    }));
    gate.release();
    let result = tokio::time::timeout(Duration::from_secs(10), handle.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(result.usable(), "{:?}", result.report());
    let rows =
        solve_variables::Row::rows(&result.table("runtime.solve_variables").unwrap()).unwrap();
    assert_eq!(rows.len(), 1);
    assert!((rows[0].value.unwrap() - 3.).abs() <= 0.001 * 3.);
    if let Err(failure) = reviewed {
        std::panic::resume_unwind(failure);
    }
}
