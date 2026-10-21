// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual submitted foreign lease through retained native storage, not a mock grant.
use super::*;
use pse_backend_native::{dynamics::*, solve::Execution};
use pse_ids::{ContentHash, SemanticId};
use pse_math::index::{OriginalCol, OriginalRow};

#[derive(Debug)]
struct Decay {
    contract: Contract,
    layout: Arc<PreparedLayout>,
    initial_calls: usize,
}
impl Decay {
    fn new() -> Result<Self, pse_backend_native::ProblemError> {
        let id = |n| SemanticId::from_bytes([n; 16]);
        Ok(Self {
            contract: Contract {
                identity: ContentHash::from_bytes([98; 32]),
                states: vec![id(1)],
                differential: vec![true],
                parameters: vec![id(2)],
                outputs: vec![id(3)],
                quadratures: vec![],
                balances: vec![],
                events: vec![vec![]],
                signs: vec![StateSign::Free],
                derivatives: pse_kernels::DerivativeOrder::First,
            },
            layout: Arc::new(PreparedLayout::new(
                1,
                std::iter::once(SupportEntry::new(OriginalRow::new(0), OriginalCol::new(0))),
                128 << 20,
            )?),
            initial_calls: 0,
        })
    }
}
impl Oracle for Decay {
    fn contract(&self) -> &Contract {
        &self.contract
    }
    fn prepared_layout(
        &self,
        _: usize,
    ) -> Result<Arc<PreparedLayout>, pse_backend_native::ProblemError> {
        Ok(self.layout.clone())
    }
    fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
        match function {
            Function::Initial => vec![SupportEntry::new(OriginalRow::new(0), OriginalCol::new(1))],
            Function::Rhs | Function::Output => {
                vec![SupportEntry::new(OriginalRow::new(0), OriginalCol::new(0))]
            }
            _ => vec![],
        }
    }
    fn evaluate(
        &mut self,
        _: usize,
        function: Function,
        _: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, pse_backend_native::ProblemError> {
        let (values, column, partial) = match function {
            Function::Initial => {
                self.initial_calls += 1;
                (vec![p[0]], 1, 1.)
            }
            Function::Rhs => (vec![-x[0]], 0, -1.),
            Function::Output => (vec![x[0]], 0, 1.),
            Function::Roots => (vec![], 0, 0.),
            _ => {
                return Err(pse_backend_native::ProblemError::unsupported(
                    "retained lease fixture role",
                ));
            }
        };
        let jacobian = if derivatives {
            let entries = if values.is_empty() {
                vec![]
            } else {
                vec![faer::sparse::Triplet::new(0, column, partial)]
            };
            Some(
                Arc::new(
                    faer::sparse::SparseColMat::try_new_from_triplets(values.len(), 2, &entries)
                        .map_err(|error| {
                            pse_backend_native::ProblemError::internal(format!(
                                "fixture partial matrix: {error:?}"
                            ))
                        })?,
                )
                .into(),
            )
        } else {
            None
        };
        Ok(Evaluation { values, jacobian })
    }
}
#[tokio::test]
async fn actual_submitted_foreign_lease_plateaus_and_outlives_task_until_idas_storage_drops() {
    let foreign = 128 << 20;
    let (service, _cache) = super::super::tests::service_with_policy(
        256 << 20,
        super::super::MathPolicy {
            foreign_bytes: foreign,
            ..Default::default()
        },
    );
    let baseline = service.pool.reserved();
    let inner = service.clone();
    let handle = service
        .submit_with_storage(
            1,
            0,
            Submission::ephemeral(),
            move |flag, progress, storage| {
                assert_eq!(storage.size(), foreign);
                let scope = pse_kernels::ExecutionScope::new(flag.clone(), None);
                let task = crate::math::strategy::admission::TaskAdmission::new(
                    pse_model::strategy::WorkLimits {
                        attempts: 1,
                        evaluations: None,
                        iterations: None,
                        factorizations: None,
                        proof_steps: None,
                    },
                    scope.clone(),
                    Some(inner.pool.clone()),
                    false,
                );
                task.retain_storage_allowance(storage.clone())?;
                let task_weak = Arc::downgrade(&task);
                let mut execution = Execution::within(
                    flag,
                    &pse_backend_native::solve::Controls::default(),
                    scope,
                )?;
                execution.progress = progress;
                execution.memory = Some(storage.size());
                execution.work_admission = Some(task.clone());
                let held = inner.pool.reserved();
                let mut workspace = IdasWorkspace::default();
                let snapshot = pse_backend_native::execution::Snapshot::observe(
                    &pse_backend_native::execution::LINKED,
                );
                let profile = Profile {
                    method: Method::Idas,
                    end: 0.2,
                    samples: vec![0., 0.1, 0.2],
                    rtol: 0.001,
                    atol: vec![0.001],
                    sensitivity: DynamicSensitivity::Forward,
                    parameter_scales: vec![1.],
                    ..Default::default()
                };
                let mut source = Decay::new()?;
                let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
                let operation = hub.start("actual retained IDAS lease");
                let context = operation.context();
                let _entered = context.enter();
                for parameter in [1., 2., 0.5, 1.5] {
                    let report = integrate_with_workspace_observed(
                        &mut source,
                        &profile,
                        &[parameter],
                        &execution,
                        &snapshot,
                        &mut workspace,
                    )?;
                    assert_eq!(
                        report.termination,
                        Termination::Completed,
                        "{:?}",
                        report.error
                    );
                    let last = report.samples.last().unwrap();
                    assert!(
                        (last.state[0] - parameter * (-profile.end).exp()).abs()
                            <= 0.001 * (1. + parameter)
                    );
                    assert!((last.state_sensitivities[0] - (-profile.end).exp()).abs() <= 0.001);
                    drop(report);
                    assert_eq!(
                        inner.pool.reserved(),
                        held,
                        "retained calls share one actual prepaid envelope"
                    );
                    let before = source.initial_calls;
                    let mut other = IdasWorkspace::default();
                    let refused = integrate_with_workspace_observed(
                        &mut source,
                        &profile,
                        &[parameter],
                        &execution,
                        &snapshot,
                        &mut other,
                    )?;
                    assert_ne!(refused.termination, Termination::Completed);
                    assert!(
                        refused.error.is_some(),
                        "occupied allowance must refuse fresh fallback explicitly"
                    );
                    assert_eq!(
                        source.initial_calls, before,
                        "the occupied-owner refusal precedes Initial source effects"
                    );
                    assert_eq!(inner.pool.reserved(), held);
                }
                let counts = operation
                    .reader()
                    .snapshot(std::time::Duration::from_secs(60))
                    .metrics;
                assert!(
                    counts
                        .iter()
                        .any(|metric| metric.name == "idas.workspace.reuses"
                            && matches!(
                                metric.cumulative,
                                pse_diagnostics::observation::Value::Count(3)
                            ))
                );
                drop(execution);
                drop(task);
                drop(storage);
                assert!(
                    task_weak.upgrade().is_none(),
                    "native storage must not retain a completed work ledger"
                );
                assert_eq!(
                    inner.pool.reserved(),
                    held,
                    "the actual native owner retains detached foreign coverage"
                );
                drop(workspace);
                assert_eq!(inner.pool.reserved(), held - foreign);
                Ok(((), 0))
            },
        )
        .unwrap();
    let ((), result) = handle.finish().await.unwrap();
    drop(result);
    assert_eq!(service.pool.reserved(), baseline);
}
