// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded provisional simulation output. Subscriber ownership, append ACK and
//! joined scientific completion are separate from the live native history owner.
use super::{ModelingSimulation, RunResult, WorkflowError, contract, relation};
use pse_backend_native::{
    ProblemError,
    dynamics::{Sample, SampleRetention},
};
use pse_model::generated::identities::RunId;
use pse_relations::columnar::{Collection, FieldCheckedBatch};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Explicit finite queue and terminal payload demand, bound before dispatch.
#[derive(Clone, Copy, Debug)]
pub struct LiveSimulationOptions {
    /// Maximum pending events at each producer/subscriber boundary.
    pub capacity: usize,
    /// Whether the terminal native report also retains the complete trajectory.
    pub retention: SampleRetention,
}
impl LiveSimulationOptions {
    pub(super) fn admit(
        self,
        profile: &pse_backend_native::dynamics::Profile,
    ) -> Result<(), WorkflowError> {
        if self.capacity == 0 || self.capacity > 1024 {
            return Err(contract("live simulation queue capacity must be 1..=1024"));
        }
        if self.retention == SampleRetention::Compact
            && profile.sensitivity != pse_backend_native::dynamics::DynamicSensitivity::None
        {
            return Err(crate::math::MathRuntimeError::from(ProblemError::Unsupported(
                "compact live transport delivers physical values; explicitly requested sample sensitivities require Full retention".into(),
            )).into());
        }
        Ok(())
    }
}

/// Owned checked physical rows with the original accuracy-attempt locator.
#[derive(Clone, Debug)]
pub struct LiveSampleBatch {
    /// Execution identity, independent of model/content equality.
    pub run_id: RunId,
    /// Canonical native attempt key, absent for an ephemeral run.
    pub canonical_attempt: Option<String>,
    /// Actual accuracy retry, never relabeled as the selected final attempt.
    pub accuracy_attempt: u64,
    /// Original completed sample ordinal within this accuracy attempt.
    pub ordinal: usize,
    /// Provisional physical output rows. Completion grants no final permission here.
    pub table: FieldCheckedBatch,
    _owner: Arc<pse_columnar::AllocationLease>,
}

/// Computed output and actual recording settlement have different variants.
#[derive(Debug)]
pub enum LiveSimulationEvent {
    /// Completed provisional sample; no durable ACK or final scientific claim.
    Sample(LiveSampleBatch),
    /// Actual native attempt completion and original-check summary.
    AttemptFinished(Arc<LiveSampleAttempt>),
    /// Accuracy producer has selected this attempt; joined completion is still separate.
    Selected {
        /// Actual selected accuracy pass, after all requested refinement decisions.
        accuracy_attempt: u64,
    },
    /// Actual append ACK covers this many original live-relation rows.
    Recorded {
        /// Acknowledged prefix extent in original live-relation row order.
        rows: u64,
    },
    /// An append or prefix close could not be established; prior ACKs remain facts.
    RecordingUnknown(Arc<WorkflowError>),
    /// All provisional rows were acknowledged and the live relation producer closed.
    RecordingClosed,
    /// Joined result includes native drain, scientific completion and recording outcome.
    Joined(Arc<RunResult>),
}

/// A retained attempt summary carries its own charge after the native owner returns.
#[derive(Debug)]
pub struct LiveSampleAttempt {
    /// Actual attempt/check/coverage facts; no append or scientific acceptance is inferred.
    pub summary: super::SimulationSampleAttempt,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl std::ops::Deref for LiveSampleAttempt {
    type Target = super::SimulationSampleAttempt;
    fn deref(&self) -> &Self::Target {
        &self.summary
    }
}

/// One subscription. Dropping it releases only its queue and escaped-buffer ownership.
#[derive(Debug)]
pub struct LiveSimulationCursor {
    receiver: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<LiveSimulationEvent>>,
    completion: Option<tokio::sync::watch::Receiver<Option<Arc<RunResult>>>>,
    joined: std::sync::atomic::AtomicBool,
    closed: pse_diagnostics::Cancellation,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl LiveSimulationCursor {
    pub(super) fn bind_completion(
        &mut self,
        completion: tokio::sync::watch::Receiver<Option<Arc<RunResult>>>,
    ) {
        self.completion = Some(completion);
    }
    /// Read the next event; None means this subscription drained, not scientific success.
    pub async fn next(&self) -> Option<LiveSimulationEvent> {
        let mut receiver = self.receiver.lock().await;
        tokio::select! {
            biased;
            _ = self.closed.cancelled() => {
                receiver.close();
                while receiver.try_recv().is_ok() {}
                None
            }
            event = receiver.recv() => {
                if event.is_some() { return event; }
                // Settlement never waits for a terminal queue slot. The run's existing
                // watch owns both early refusals and cancellation after native drain.
                if self.joined.swap(true, std::sync::atomic::Ordering::AcqRel) { return None; }
                let mut completion = self.completion.clone()?;
                loop {
                    if let Some(result) = completion.borrow().clone() {
                        return Some(LiveSimulationEvent::Joined(result));
                    }
                    tokio::select! {
                        biased;
                        _ = self.closed.cancelled() => return None,
                        changed = completion.changed() => { if changed.is_err() { return None; } }
                    }
                }
            },
        }
    }
    /// Detach this subscription, releasing unread buffers without cancelling science.
    pub fn close(&self) {
        self.closed.cancel();
        if let Ok(mut receiver) = self.receiver.try_lock() {
            receiver.close();
            while receiver.try_recv().is_ok() {}
        }
    }
}

/// Selected immutable physical lineage, checked once against the complete original
/// identity inventories before native dispatch. Original admitted ownership remains
/// attached; per-sample encoding reads only this ordered projection.
#[derive(Clone, Copy, Debug)]
pub(super) struct LiveOutputLineage {
    pub symbol: pse_ids::SemanticId,
    pub quantity: pse_ids::SemanticId,
    pub unit: pse_ids::SemanticId,
}
pub(super) struct LiveProjection {
    pub outputs: Vec<LiveOutputLineage>,
    pub modes: Vec<pse_ids::ContentHash>,
    pub source: pse_math::SharedAllocation<pse_compiler::workspace::AdmittedModeling>,
    pub _owner: Arc<pse_columnar::AllocationLease>,
}

/// The supervisor moves this owner intact. Fields drop in declaration order, so
/// the shared channel-envelope grant outlives both channel handles and projection.
pub(super) struct LiveChannel {
    pub receiver: tokio::sync::mpsc::Receiver<LiveCapture>,
    pub subscriber: tokio::sync::mpsc::Sender<LiveSimulationEvent>,
    pub projection: Result<LiveProjection, Arc<WorkflowError>>,
    _owner: Arc<pse_columnar::AllocationLease>,
}

/// Run-owned native callback payload. Only selected physical values cross this
/// boundary; native states, sensitivities and complete trajectories stay with
/// their original owner. The supervision lane owns checked relation encoding.
pub(super) enum LiveCapture {
    Sample {
        run_id: RunId,
        canonical_attempt: Option<String>,
        accuracy_attempt: u64,
        ordinal: usize,
        mode: usize,
        time: f64,
        outputs: Vec<f64>,
        owner: Arc<pse_columnar::AllocationLease>,
    },
    Event(LiveSimulationEvent),
}
impl LiveCapture {
    pub(super) fn encode(
        self,
        prepared: &ModelingSimulation,
        projection: &LiveProjection,
        cancel: &pse_diagnostics::Cancellation,
        deadline: Instant,
    ) -> Result<LiveSimulationEvent, WorkflowError> {
        encoding_checkpoint(cancel, deadline)?;
        let Self::Sample {
            run_id,
            canonical_attempt,
            accuracy_attempt,
            ordinal,
            mode,
            time,
            outputs,
            owner,
        } = self
        else {
            let Self::Event(event) = self else {
                return Err(contract("live capture variant absent"));
            };
            return Ok(event);
        };
        let runtime = &prepared.runtime;
        if !pse_math::SharedAllocation::ptr_eq(
            &projection.source,
            &prepared.model().compiled().admitted,
        ) {
            return Err(contract(
                "live projection belongs to another admitted source",
            ));
        }
        let mode_identity = *projection
            .modes
            .get(mode)
            .ok_or_else(|| contract("live trajectory mode absent"))?;
        if outputs.len() != projection.outputs.len() {
            return Err(contract("live trajectory output extent"));
        }
        let mode =
            i64::try_from(mode).map_err(|_| contract("live trajectory mode ordinal extent"))?;
        let attempt = i64::try_from(accuracy_attempt)
            .map_err(|_| contract("live accuracy attempt ordinal extent"))?;
        let sample = i64::try_from(ordinal).map_err(|_| contract("live sample ordinal extent"))?;
        let extent = outputs
            .len()
            .checked_mul(size_of::<
                pse_model::generated::runtime::live_simulation_samples::Row,
            >())
            .and_then(|bytes| bytes.checked_add(size_of::<LiveSampleBatch>() + 1024))
            .ok_or_else(|| contract("live physical sample extent"))?;
        let working = runtime
            .native()
            .reserve("simulation:live-row-working", extent)?;
        let pool = runtime.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let validation = runtime.validation_context();
        let mut columns = Collection::new(runtime.registry(), &pool, &token, &validation);
        columns
            .ensure::<pse_model::generated::runtime::live_simulation_samples::Row>()
            .map_err(relation)?;
        for (column, lineage) in projection.outputs.iter().enumerate() {
            encoding_checkpoint(cancel, deadline)?;
            columns
                .push(
                    pse_model::generated::runtime::live_simulation_samples::Row {
                        run_id,
                        mode,
                        mode_identity,
                        accuracy_attempt: attempt,
                        sample,
                        time,
                        symbol_id: lineage.symbol,
                        quantity_id: lineage.quantity,
                        unit_id: lineage.unit,
                        value: outputs[column],
                    },
                )
                .map_err(relation)?;
        }
        let table = columns
            .finish()
            .map_err(relation)?
            .pop_first()
            .map(|(_, batch)| batch)
            .ok_or_else(|| contract("live physical relation absent"))?;
        encoding_checkpoint(cancel, deadline)?;
        drop(outputs);
        drop(owner);
        Ok(LiveSimulationEvent::Sample(LiveSampleBatch {
            run_id,
            canonical_attempt,
            accuracy_attempt,
            ordinal,
            table,
            _owner: working,
        }))
    }
}

pub(super) fn encoding_checkpoint(
    cancel: &pse_diagnostics::Cancellation,
    deadline: Instant,
) -> Result<(), WorkflowError> {
    if cancel.is_cancelled() {
        return Err(crate::math::MathRuntimeError::Cancelled.into());
    }
    if Instant::now() >= deadline {
        return Err(crate::math::MathRuntimeError::Solve(ProblemError::Limit {
            kind: pse_backend_native::LimitKind::Time,
            detail: "live encoding original deadline".into(),
        })
        .into());
    }
    Ok(())
}

pub(super) struct LiveDispatch {
    pub observer: Box<dyn super::SimulationSampleObserver>,
    pub channel: LiveChannel,
    pub cursor: LiveSimulationCursor,
    pub retention: SampleRetention,
}

impl LiveDispatch {
    pub(super) fn new(
        prepared: ModelingSimulation,
        options: LiveSimulationOptions,
        run_id: RunId,
        canonical_attempt: Option<String>,
        cancel: &pse_diagnostics::Cancellation,
        deadline: Instant,
    ) -> Result<Self, WorkflowError> {
        // Canonical launch settles preparation refusals through its existing
        // joined-result path; a failed projection never launches native work.
        let projection = prepared.live_projection(cancel, deadline).map_err(Arc::new);
        let owner = prepared.runtime.native().reserve(
            "simulation:live-queues",
            options
                .capacity
                .checked_mul(size_of::<LiveCapture>() + size_of::<LiveSimulationEvent>() + 256)
                .and_then(|bytes| bytes.checked_add(size_of::<Self>() + size_of::<LiveChannel>()))
                .ok_or_else(|| contract("live queue extent"))?,
        )?;
        let (sender, receiver) = tokio::sync::mpsc::channel(options.capacity);
        let (subscriber, public) = tokio::sync::mpsc::channel(options.capacity);
        Ok(Self {
            observer: Box::new(Observer {
                prepared,
                sender,
                run_id,
                canonical_attempt,
                _owner: owner.clone(),
            }),
            channel: LiveChannel {
                receiver,
                subscriber,
                projection,
                _owner: owner.clone(),
            },
            retention: options.retention,
            cursor: LiveSimulationCursor {
                receiver: tokio::sync::Mutex::new(public),
                completion: None,
                joined: Default::default(),
                closed: Default::default(),
                _owner: owner,
            },
        })
    }
}

struct Observer {
    prepared: ModelingSimulation,
    sender: tokio::sync::mpsc::Sender<LiveCapture>,
    run_id: RunId,
    canonical_attempt: Option<String>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl Observer {
    fn send(
        &self,
        mut event: LiveCapture,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<(), ProblemError> {
        execution.check()?;
        match self.sender.try_send(event) {
            Ok(()) => Ok(()),
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => Err(ProblemError::internal(
                "live simulation recording supervisor disappeared",
            )),
            Err(tokio::sync::mpsc::error::TrySendError::Full(returned)) => {
                event = returned;
                super::with_simulation_sample_wait(execution, || {
                    loop {
                        execution.check()?;
                        match self.sender.try_send(event) {
                            Ok(()) => return Ok(()),
                            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                return Err(ProblemError::internal(
                                    "live simulation recording supervisor disappeared",
                                ));
                            }
                            Err(tokio::sync::mpsc::error::TrySendError::Full(returned)) => {
                                event = returned;
                                std::thread::sleep(Duration::from_millis(1));
                            }
                        }
                    }
                })
            }
        }
    }
}

impl super::SimulationSampleObserver for Observer {
    fn sample(
        &mut self,
        accuracy_attempt: u64,
        ordinal: usize,
        sample: &Sample,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<(), ProblemError> {
        execution.check()?;
        let extent = sample
            .outputs
            .len()
            .checked_mul(size_of::<f64>())
            .and_then(|bytes| bytes.checked_add(size_of::<LiveCapture>()))
            .and_then(|bytes| {
                bytes.checked_add(self.canonical_attempt.as_ref().map_or(0, String::len))
            })
            .ok_or_else(|| ProblemError::memory("live capture extent"))?;
        let owner = self
            .prepared
            .runtime
            .native()
            .reserve("simulation:live-capture", extent)
            .map_err(crate::math::MathRuntimeError::into_problem)?;
        let mut outputs = Vec::new();
        outputs
            .try_reserve_exact(sample.outputs.len())
            .map_err(|_| ProblemError::memory("live capture allocation refused"))?;
        outputs.extend_from_slice(&sample.outputs);
        self.send(
            LiveCapture::Sample {
                run_id: self.run_id,
                canonical_attempt: self.canonical_attempt.clone(),
                accuracy_attempt,
                ordinal,
                mode: sample.mode,
                time: sample.time,
                outputs,
                owner,
            },
            execution,
        )
    }
    fn attempt_finished(
        &mut self,
        summary: &super::SimulationSampleAttempt,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<(), ProblemError> {
        let bytes = summary
            .coverage
            .points
            .len()
            .checked_mul(size_of::<(f64, usize)>())
            .and_then(|bytes| bytes.checked_add(size_of::<LiveSampleAttempt>() + 64))
            .ok_or_else(|| ProblemError::memory("live coverage summary extent"))?;
        let owner = self
            .prepared
            .runtime
            .native()
            .reserve("simulation:live-coverage-summary", bytes)
            .map_err(crate::math::MathRuntimeError::into_problem)?;
        self.send(
            LiveCapture::Event(LiveSimulationEvent::AttemptFinished(Arc::new(
                LiveSampleAttempt {
                    summary: summary.clone(),
                    _owner: owner,
                },
            ))),
            execution,
        )
    }
    fn finished(
        &mut self,
        selected_attempt: u64,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<(), ProblemError> {
        self.send(
            LiveCapture::Event(LiveSimulationEvent::Selected {
                accuracy_attempt: selected_attempt,
            }),
            execution,
        )
    }
}

/// Backpressure checks the original clock/cancellation. A dropped subscription is
/// ordinary detachment; recording and native drain remain owned by the supervisor.
pub(super) async fn deliver(
    subscriber: &tokio::sync::mpsc::Sender<LiveSimulationEvent>,
    event: LiveSimulationEvent,
    cancel: &pse_diagnostics::Cancellation,
    deadline: Instant,
) {
    let sent = subscriber.send(event);
    tokio::pin!(sent);
    loop {
        tokio::select! {
            _ = &mut sent => return,
            _ = tokio::time::sleep(Duration::from_millis(10)) => {
                if cancel.flag().load(std::sync::atomic::Ordering::Acquire) || Instant::now() >= deadline { return; }
            }
        }
    }
}

#[cfg(all(test, feature = "solver-diffsol"))]
mod tests {
    use super::*;
    use pse_backend_native::dynamics::Method;

    #[test]
    fn compact_refuses_requested_sensitivity_payload_before_live_dispatch() {
        use pse_backend_native::dynamics::{DynamicSensitivity, Profile};
        let options = LiveSimulationOptions {
            capacity: 1,
            retention: SampleRetention::Compact,
        };
        assert!(options.admit(&Profile::default()).is_ok());
        for sensitivity in [DynamicSensitivity::Forward, DynamicSensitivity::Adjoint] {
            let profile = Profile {
                sensitivity,
                ..Default::default()
            };
            assert!(matches!(
                options.admit(&profile),
                Err(WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                    ProblemError::Unsupported(_)
                )))
            ));
            assert!(
                LiveSimulationOptions {
                    retention: SampleRetention::Full,
                    ..options
                }
                .admit(&profile)
                .is_ok()
            );
        }
    }

    async fn prepared() -> ModelingSimulation {
        prepared_on(crate::workflow::tests::runtime_with_workspace(32 << 20)).await
    }
    async fn prepared_on(runtime: super::super::Runtime) -> ModelingSimulation {
        prepared_with_time_limit(runtime, Duration::from_secs(300)).await
    }
    async fn prepared_with_time_limit(
        runtime: super::super::Runtime,
        time_limit: Duration,
    ) -> ModelingSimulation {
        prepared_with_inventory(runtime, time_limit, 0).await
    }
    #[expect(
        clippy::unwrap_used,
        reason = "the fixture must contain valid authored dynamics and prepare its declared Root"
    )]
    async fn prepared_with_inventory(
        runtime: super::super::Runtime,
        time_limit: Duration,
        unrelated: usize,
    ) -> ModelingSimulation {
        let mut source = String::from(
            "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == -x[i]/1{s}; eq initial: x[0{s}] == 1{s}; annotation check x(x[i] > 0{s}); ",
        );
        for ordinal in 0..unrelated {
            source.push_str(&format!("let spare{ordinal}: Time = 1{{s}}; annotation check spare{ordinal}(spare{ordinal} > 0{{s}}); "));
        }
        source.push_str("} }");
        let rows = pse_authoring::language::parse(
            &source,
            pse_ids::SemanticId::from_bytes([175; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        package
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                crate::workflow::tests::compiler_profile(),
                pse_backend_native::dynamics::Profile {
                    method: Method::Diffsol,
                    end: 1.,
                    samples: vec![0., 0.25, 0.5, 0.75, 1.],
                    atol: vec![1e-7],
                    rtol: 1e-7,
                    time_limit,
                    ..Default::default()
                },
                pse_kernels::DerivativeOrder::First,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn live_capture_keeps_only_selected_values_and_defers_checked_encoding() {
        let prepared = prepared().await;
        let options = LiveSimulationOptions {
            capacity: 1,
            retention: SampleRetention::Compact,
        };
        let run_id = pse_operations::mint_id();
        let mut dispatch = LiveDispatch::new(
            prepared.clone(),
            options,
            run_id,
            Some("attempt".into()),
            &Default::default(),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        let execution =
            pse_backend_native::solve::Execution::new(Default::default(), &Default::default());
        let sample = Sample {
            mode: 0,
            time: 0.25,
            outputs: vec![0.75],
            state: vec![9.; 4096],
            integrals: vec![8.; 4096],
            state_sensitivities: vec![7.; 4096],
            output_sensitivities: vec![6.; 4096],
        };
        dispatch.observer.sample(3, 2, &sample, &execution).unwrap();
        let capture = dispatch.channel.receiver.recv().await.unwrap();
        let LiveCapture::Sample {
            outputs,
            accuracy_attempt,
            ordinal,
            owner,
            ..
        } = &capture
        else {
            panic!("sample capture absent")
        };
        assert_eq!(outputs, &[0.75]);
        assert_eq!((*accuracy_attempt, *ordinal), (3, 2));
        assert!(
            owner.size() < 1024,
            "native state and sensitivity inventory cannot cross compact capture"
        );
        let LiveSimulationEvent::Sample(batch) = capture
            .encode(
                &prepared,
                dispatch.channel.projection.as_ref().unwrap(),
                &Default::default(),
                Instant::now() + Duration::from_secs(10),
            )
            .unwrap()
        else {
            panic!("checked sample absent")
        };
        assert_eq!(batch.run_id, run_id);
        assert_eq!(batch.canonical_attempt.as_deref(), Some("attempt"));
        assert_eq!((batch.accuracy_attempt, batch.ordinal), (3, 2));
        assert_eq!(batch.table.batch().num_rows(), 1);
    }

    #[tokio::test]
    async fn live_capture_invalid_mode_and_output_extent_refuse_in_supervision() {
        let prepared = prepared().await;
        let execution =
            pse_backend_native::solve::Execution::new(Default::default(), &Default::default());
        for (mode, outputs) in [(usize::MAX, vec![1.]), (0, Vec::new()), (0, vec![1., 2.])] {
            let mut dispatch = LiveDispatch::new(
                prepared.clone(),
                LiveSimulationOptions {
                    capacity: 1,
                    retention: SampleRetention::Compact,
                },
                pse_operations::mint_id(),
                None,
                &Default::default(),
                Instant::now() + Duration::from_secs(10),
            )
            .unwrap();
            let sample = Sample {
                mode,
                time: 0.,
                outputs,
                state: Vec::new(),
                integrals: Vec::new(),
                state_sensitivities: Vec::new(),
                output_sensitivities: Vec::new(),
            };
            dispatch.observer.sample(0, 0, &sample, &execution).unwrap();
            assert!(
                dispatch
                    .channel
                    .receiver
                    .recv()
                    .await
                    .unwrap()
                    .encode(
                        &prepared,
                        dispatch.channel.projection.as_ref().unwrap(),
                        &Default::default(),
                        Instant::now() + Duration::from_secs(10)
                    )
                    .is_err()
            );
        }
    }

    #[tokio::test]
    async fn live_queue_grant_outlives_dropped_cursor_and_finished_native_observer() {
        let prepared = prepared().await;
        let dispatch = LiveDispatch::new(
            prepared,
            LiveSimulationOptions {
                capacity: 1,
                retention: SampleRetention::Compact,
            },
            pse_operations::mint_id(),
            None,
            &Default::default(),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        let LiveDispatch {
            mut observer,
            mut channel,
            cursor,
            ..
        } = dispatch;
        let grant = Arc::downgrade(&channel._owner);
        let execution =
            pse_backend_native::solve::Execution::new(Default::default(), &Default::default());
        observer
            .sample(
                0,
                0,
                &Sample {
                    mode: 0,
                    time: 0.,
                    outputs: vec![1.],
                    state: Vec::new(),
                    integrals: Vec::new(),
                    state_sensitivities: Vec::new(),
                    output_sensitivities: Vec::new(),
                },
                &execution,
            )
            .unwrap();
        drop(cursor);
        drop(observer);
        assert_eq!(
            channel.receiver.len(),
            1,
            "a captured sample remains after native completion"
        );
        assert!(
            grant.upgrade().is_some(),
            "supervisor must own both channel envelopes while draining"
        );
        drop(channel.receiver.recv().await.unwrap());
        assert!(
            grant.upgrade().is_some(),
            "empty but live channels retain their envelopes"
        );
        drop(channel);
        assert!(
            grant.upgrade().is_none(),
            "channel teardown releases the last envelope grant"
        );
    }

    #[tokio::test]
    async fn live_cursor_drop_joins_native_and_releases_supervisor_queue_grant() {
        let prepared = prepared().await;
        let (handle, cursor) = prepared
            .start_live(LiveSimulationOptions {
                capacity: 1,
                retention: SampleRetention::Compact,
            })
            .unwrap();
        let grant = Arc::downgrade(&cursor._owner);
        drop(cursor);
        let result = tokio::time::timeout(Duration::from_secs(10), handle.wait())
            .await
            .unwrap()
            .unwrap();
        assert!(
            result.report().is_ok(),
            "dropping the subscription cannot cancel science"
        );
        assert!(
            grant.upgrade().is_none(),
            "joined native work and channel teardown release their shared grant"
        );
    }

    #[tokio::test]
    async fn live_projection_selected_encoding_charge_is_independent_of_original_inventory() {
        let mut extents = Vec::new();
        let mut inventories = Vec::new();
        for unrelated in [0, 64] {
            let prepared = prepared_with_inventory(
                crate::workflow::tests::runtime_with_workspace(32 << 20),
                Duration::from_secs(300),
                unrelated,
            )
            .await;
            inventories.push(prepared.model().compiled().admitted.case().rows().len());
            let mut dispatch = LiveDispatch::new(
                prepared.clone(),
                LiveSimulationOptions {
                    capacity: 1,
                    retention: SampleRetention::Compact,
                },
                pse_operations::mint_id(),
                None,
                &Default::default(),
                Instant::now() + Duration::from_secs(10),
            )
            .unwrap();
            let projection = dispatch.channel.projection.as_ref().unwrap();
            assert_eq!(
                projection.outputs.len(),
                1,
                "unrelated authored members are not selected native outputs"
            );
            let projection_extent = projection._owner.size();
            let execution =
                pse_backend_native::solve::Execution::new(Default::default(), &Default::default());
            for ordinal in 0..3 {
                dispatch
                    .observer
                    .sample(
                        0,
                        ordinal,
                        &Sample {
                            mode: 0,
                            time: ordinal as f64 * 0.25,
                            outputs: vec![1.],
                            state: Vec::new(),
                            integrals: Vec::new(),
                            state_sensitivities: Vec::new(),
                            output_sensitivities: Vec::new(),
                        },
                        &execution,
                    )
                    .unwrap();
                let capture = dispatch.channel.receiver.recv().await.unwrap();
                let LiveSimulationEvent::Sample(batch) = capture
                    .encode(
                        &prepared,
                        projection,
                        &Default::default(),
                        Instant::now() + Duration::from_secs(10),
                    )
                    .unwrap()
                else {
                    panic!("checked sample absent")
                };
                assert_eq!(batch.table.batch().num_rows(), 1);
                assert_eq!(batch.ordinal, ordinal);
                extents.push((projection_extent, batch._owner.size()));
            }
        }
        assert!(
            inventories[1] >= inventories[0] + 64,
            "fixture must exercise the original broad physical inventory"
        );
        assert!(
            extents.iter().all(|extent| *extent == extents[0]),
            "only selected lineage and rows remain charged for repeated encoding"
        );
    }

    #[tokio::test]
    async fn live_projection_preparation_preserves_original_stop_scope() {
        let prepared = prepared().await;
        let cancel = pse_diagnostics::Cancellation::default();
        cancel.cancel();
        assert!(matches!(
            prepared.live_projection(&cancel, Instant::now() + Duration::from_secs(10)),
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
        assert!(matches!(
            prepared.live_projection(&Default::default(), Instant::now()),
            Err(WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    ..
                }
            )))
        ));
    }

    #[test]
    fn live_encoding_keeps_original_cancellation_and_deadline() {
        let cancel = pse_diagnostics::Cancellation::default();
        let deadline = Instant::now() + Duration::from_secs(10);
        assert!(encoding_checkpoint(&cancel, deadline).is_ok());
        cancel.cancel();
        assert!(matches!(
            encoding_checkpoint(&cancel, deadline),
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
        assert!(matches!(
            encoding_checkpoint(&Default::default(), Instant::now()),
            Err(WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    ..
                }
            )))
        ));
    }

    #[tokio::test]
    async fn live_samples_compact_and_full_keep_original_ordinals_checks_and_escaped_buffers() {
        let prepared = prepared().await;
        let mut endpoints = Vec::new();
        for retention in [SampleRetention::Full, SampleRetention::Compact] {
            let (handle, cursor) = prepared
                .start_live(LiveSimulationOptions {
                    capacity: 1,
                    retention,
                })
                .unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
            assert!(
                handle.result().is_none(),
                "bounded unread output must apply backpressure"
            );
            let mut ordinal = 0;
            let mut escaped = Vec::new();
            let mut original_checked = false;
            loop {
                let event = tokio::time::timeout(Duration::from_secs(10), cursor.next())
                    .await
                    .unwrap()
                    .unwrap();
                match event {
                    LiveSimulationEvent::Sample(batch) => {
                        assert_eq!(batch.run_id, handle.run_id());
                        assert_eq!(batch.accuracy_attempt, 0);
                        assert_eq!(batch.ordinal, ordinal);
                        assert!(batch.canonical_attempt.is_none());
                        assert!(batch.table.batch().num_rows() > 0);
                        escaped.push(batch.table);
                        ordinal += 1;
                    }
                    LiveSimulationEvent::AttemptFinished(summary) => {
                        assert_eq!(summary.coverage.produced(), 5);
                        assert!(summary.checks_complete && summary.checks_satisfied);
                        assert!(summary.original_checks > 0);
                        original_checked = true;
                    }
                    LiveSimulationEvent::Selected { accuracy_attempt } => {
                        assert_eq!(accuracy_attempt, 0)
                    }
                    LiveSimulationEvent::Joined(result) => {
                        let crate::workflow::RunReport::Simulation(trajectory) =
                            result.report().unwrap()
                        else {
                            panic!("simulation report missing")
                        };
                        assert!(trajectory.checks_complete());
                        assert_eq!(trajectory.report().coverage.produced(), 5);
                        assert_eq!(
                            trajectory.report().samples.len(),
                            if retention == SampleRetention::Full {
                                5
                            } else {
                                0
                            }
                        );
                        endpoints.push(
                            trajectory
                                .report()
                                .endpoint
                                .as_ref()
                                .unwrap()
                                .point
                                .state
                                .clone(),
                        );
                        break;
                    }
                    _ => panic!("ephemeral run cannot manufacture recording acknowledgement"),
                }
            }
            assert!(original_checked);
            assert_eq!(ordinal, 5);
            cursor.close();
            drop(cursor);
            drop(handle);
            assert_eq!(escaped.len(), 5);
            assert!(escaped.iter().all(|batch| batch.batch().num_rows() > 0));
        }
        assert_eq!(endpoints[0], endpoints[1]);
    }

    #[tokio::test]
    async fn live_subscription_close_detaches_and_backpressure_cancel_joins_native_owner() {
        let prepared = prepared().await;
        let options = LiveSimulationOptions {
            capacity: 1,
            retention: SampleRetention::Compact,
        };
        let (handle, cursor) = prepared.start_live(options).unwrap();
        cursor.close();
        assert!(cursor.next().await.is_none());
        let result = tokio::time::timeout(Duration::from_secs(10), handle.wait())
            .await
            .unwrap()
            .unwrap();
        assert!(result.report().is_ok(), "detachment cannot cancel science");
        let (handle, cursor) = prepared.start_live(options).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        handle.cancel();
        let settled = tokio::time::timeout(Duration::from_secs(10), handle.wait())
            .await
            .unwrap()
            .unwrap();
        assert!(
            handle.result().is_some(),
            "cancelled native owner joined before publication"
        );
        loop {
            if let LiveSimulationEvent::Joined(result) =
                tokio::time::timeout(Duration::from_secs(10), cursor.next())
                    .await
                    .unwrap()
                    .unwrap()
            {
                assert!(Arc::ptr_eq(&result, &settled));
                break;
            }
        }
        assert!(
            cursor.next().await.is_none(),
            "one terminal result after a full cancelled queue"
        );
        cursor.close();
    }

    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_live_expired_admission_yields_one_joined_refusal() {
        let runtime = crate::workflow::durable_tests::durable_runtime();
        let prepared = prepared_with_time_limit(runtime.clone(), Duration::from_nanos(1)).await;
        let (handle, cursor) = prepared
            .start_live(LiveSimulationOptions {
                capacity: 1,
                retention: SampleRetention::Compact,
            })
            .unwrap();
        let settled = tokio::time::timeout(Duration::from_secs(30), handle.wait())
            .await
            .unwrap()
            .unwrap();
        assert!(!settled.usable());
        let event = tokio::time::timeout(Duration::from_secs(30), cursor.next())
            .await
            .unwrap()
            .unwrap();
        let LiveSimulationEvent::Joined(result) = event else {
            panic!("expired admission must settle before native samples")
        };
        assert!(Arc::ptr_eq(&result, &settled));
        assert!(cursor.next().await.is_none());
        cursor.close();
        drop(cursor);
        drop(result);
        drop(settled);
        drop(handle);
        drop(prepared);
        runtime.clear_program_cache();
        runtime
            .canonical()
            .unwrap()
            .store()
            .remove_isolated_fixture()
            .await
            .unwrap();
    }

    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_live_sample_acknowledgments_close_the_original_attempt_prefix() {
        let runtime = crate::workflow::durable_tests::durable_runtime();
        let prepared = prepared_on(runtime.clone()).await;
        let (handle, cursor) = prepared
            .start_live(LiveSimulationOptions {
                capacity: 1,
                retention: SampleRetention::Compact,
            })
            .unwrap();
        let canonical_attempt = handle.canonical_attempt_key().unwrap().to_owned();
        let mut computed_rows = 0_u64;
        let mut acknowledged_rows = 0;
        let mut closed = false;
        let result = loop {
            match tokio::time::timeout(Duration::from_secs(30), cursor.next())
                .await
                .unwrap()
                .unwrap()
            {
                LiveSimulationEvent::Sample(batch) => {
                    assert_eq!(
                        batch.canonical_attempt.as_deref(),
                        Some(canonical_attempt.as_str())
                    );
                    computed_rows += batch.table.batch().num_rows() as u64;
                }
                LiveSimulationEvent::Recorded { rows } => {
                    assert!(rows > acknowledged_rows && rows <= computed_rows);
                    acknowledged_rows = rows;
                }
                LiveSimulationEvent::RecordingClosed => {
                    assert_eq!(acknowledged_rows, computed_rows);
                    closed = true;
                }
                LiveSimulationEvent::RecordingUnknown(error) => {
                    panic!("recording failure: {error}")
                }
                LiveSimulationEvent::Joined(result) => break result,
                _ => {}
            }
        };
        assert!(closed);
        let super::super::RunDurability::Durable(record) = result.durability() else {
            panic!("durable result missing")
        };
        assert!(record.attempt.is_ok(), "{:?}", record.attempt);
        let sets = pse_operations::canonical_execution::decode_result_descriptors(
            record.manifest.as_ref().unwrap(),
        )
        .unwrap();
        let live = sets
            .iter()
            .find(|set| {
                set.name
                    == pse_relations::generated::runtime::live_simulation_samples::RELATION_ID
                        .to_string()
            })
            .unwrap();
        assert_eq!(live.row_count, acknowledged_rows);
        assert_eq!(live.batch_count, 5);
        let super::super::RunReport::Simulation(trajectory) = result.report().unwrap() else {
            panic!("simulation missing")
        };
        assert!(trajectory.checks_complete());
        assert!(trajectory.report().samples.is_empty());
        cursor.close();
        drop(cursor);
        drop(result);
        drop(handle);
        drop(prepared);
        runtime.clear_program_cache();
        runtime
            .canonical()
            .unwrap()
            .store()
            .remove_isolated_fixture()
            .await
            .unwrap();
    }
}
