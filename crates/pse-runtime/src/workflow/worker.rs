// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded native study candidate scheduling over exact canonical sources.
use super::{
    Durability, Operations, RunDurability, Runtime, WorkflowError, contract,
    durable::{DurableAttempt, DurableRecord},
};
use futures_util::{StreamExt, stream::FuturesUnordered};
use pse_admission::authoring_driver::document::{
    OwnedDocumentSet, load_package_documents_owned, package_checksum,
};
use pse_ids::{ContentHash, SemanticId};
use pse_model::generated::{
    enums::AttemptState,
    identities::{AttemptId, RunId},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncReadExt};
mod publication;
pub(in crate::workflow) use publication::Handoff as PublicationHandoff;
#[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
pub(in crate::workflow) use publication::observed as observed_scientific_publication;
#[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
pub(in crate::workflow) use publication::observed_idle;

// Keep construction temporaries out of the frame that polls the phase, and
// prevent nested phase polling from being inlined into one large stack frame.
// These boundaries affect workflow orchestration, not native numerical kernels.
#[inline(never)]
pub(super) fn boxed_worker_action<F, Fut>(action: F) -> std::pin::Pin<Box<Fut>>
where
    F: FnOnce() -> Fut,
    Fut: Future,
{
    Box::pin(action())
}

#[inline(never)]
pub(super) fn poll_worker_action<F: Future + ?Sized>(
    work: std::pin::Pin<&mut F>,
    context: &mut std::task::Context<'_>,
) -> std::task::Poll<F::Output> {
    work.poll(context)
}

/// Exact canonical physical source receipt, separate from compiler physical context identity.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PhysicalSource {
    /// Exact immutable revision in its declared canonical source kind.
    pub revision: String,
    /// Source interpretation checksum, distinct from the compiler context identity.
    pub identity: ContentHash,
}
/// One bounded worker scheduling action; no native reports accumulate in the supervisor.
#[derive(Clone, Debug)]
pub enum Processed {
    /// No currently dispatchable or recoverable occurrence was found.
    Idle,
    /// One bounded derived readiness page advanced; this is no scientific action.
    Maintenance {
        /// Exact study whose derived readiness maintenance progressed.
        study: String,
        /// Actual points or outgoing edges examined in the bounded page.
        examined: u64,
    },
    /// One claimed occurrence produced its retained execution receipt.
    Ran {
        /// Exact canonical occurrence key.
        point: String,
        /// Retained completion or observable persistence failure.
        record: Arc<DurableRecord>,
    },
    /// Shared policy settled one occurrence without native work.
    Settled {
        /// Exact canonical occurrence key.
        point: String,
    },
    /// Every occurrence settled and the effect-free parent summary was admitted.
    Finalized {
        /// Exact canonical study key.
        study: String,
    },
}
impl Processed {
    /// Actual retained attempt state; effect-free actions have no attempt.
    pub fn state(&self) -> Option<AttemptState> {
        match self {
            Self::Ran { record, .. } => record.completion.as_ref().map(|c| c.state),
            _ => None,
        }
    }
}
/// Finite polling and optional total-work bound; live service retains only a count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkerSettings {
    /// Finite delay between discovery iterations.
    pub poll: Duration,
    /// Finite interval between bounded assigned-attempt recovery passes.
    pub recovery: Duration,
    /// Optional maximum number of processed actions.
    pub maximum_actions: Option<usize>,
    /// Maximum concurrent case lanes; defaults to the runtime CPU and population allocation.
    pub maximum_in_flight: Option<usize>,
    /// Finish when a complete scoped discovery pass has no work.
    pub until_idle: bool,
}
impl Default for WorkerSettings {
    fn default() -> Self {
        Self {
            poll: Duration::from_millis(500),
            recovery: Duration::from_secs(30),
            maximum_actions: None,
            maximum_in_flight: None,
            until_idle: false,
        }
    }
}
/// Local preparation ownership only. Canonical claims remain the distributed authority.
#[derive(Default)]
struct CandidateGroup(Arc<Mutex<BTreeSet<String>>>);
struct CandidateTurn {
    group: Arc<Mutex<BTreeSet<String>>>,
    key: String,
}
impl CandidateGroup {
    fn acquire(&self, key: &str) -> Option<CandidateTurn> {
        let mut active = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !active.insert(key.to_owned()) {
            return None;
        }
        Some(CandidateTurn {
            group: self.0.clone(),
            key: key.to_owned(),
        })
    }
}
impl Drop for CandidateTurn {
    fn drop(&mut self) {
        self.group
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.key);
    }
}
async fn receiver_output(stream: impl AsyncRead + Unpin) -> Result<Vec<u8>, WorkflowError> {
    let mut bytes = Vec::new();
    stream
        .take(64 * 1024 + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|error| contract(format!("managed primary response: {error}")))?;
    if bytes.len() > 64 * 1024 {
        return Err(contract(
            "managed primary readiness response exceeds its bound",
        ));
    }
    Ok(bytes)
}

async fn receiver_response(
    mut command: tokio::process::Command,
) -> Result<(std::process::ExitStatus, Vec<u8>, Vec<u8>), WorkflowError> {
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .map_err(|error| contract(format!("managed primary control: {error}")))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| contract("managed primary output pipe absent"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| contract("managed primary error pipe absent"))?;
    let output = async { tokio::try_join!(receiver_output(stdout), receiver_output(stderr)) };
    tokio::pin!(output);
    tokio::select! {
        output = &mut output => match output {
            Ok((stdout, stderr)) => {
                let status = child.wait().await.map_err(|error| contract(format!("managed primary control: {error}")))?;
                Ok((status, stdout, stderr))
            }
            Err(error) => {
                // Bounded-output refusal still joins the actual control process.
                child.kill().await.map_err(|cause| contract(format!("managed primary control drain: {cause}")))?;
                Err(error)
            }
        },
        status = child.wait() => {
            let status = status.map_err(|error| contract(format!("managed primary control: {error}")))?;
            let (stdout, stderr) = output.await?;
            Ok((status, stdout, stderr))
        },
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PrimaryScope {
    unit: String,
    group: String,
    invocation: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PrimaryOwner {
    scope: PrimaryScope,
    scope_inode: u64,
}

/// One startup-admitted receiver lifetime; a replacement cannot become this owner.
pub(super) enum PrimaryObservation {
    Live,
    Drained,
    OwnerChanged,
}

pub(super) struct ManagedPrimary {
    selected: super::NativeExecutionSelection,
    owner: PrimaryOwner,
}
impl ManagedPrimary {
    /// Observe the selected lifetime without granting resource-release authority.
    pub(super) async fn observe(&self) -> Result<PrimaryObservation, WorkflowError> {
        // The startup-selected immutable supervisor observes the original lifetime,
        // even if the next native selection changes. No worker bytes are reopened.
        let (receiver, _) = self.selected.managed_primary()?;
        let mut command = tokio::process::Command::new(&receiver.supervisor_executable);
        command
            .arg(&receiver.supervisor_script)
            .arg("observe-primary")
            .arg("--state")
            .arg(self.selected.receiver_state())
            .arg("--canonical-database")
            .arg(self.selected.database())
            .arg("--primary-owner")
            .arg(serde_json::to_string(&self.owner).map_err(|error| contract(error.to_string()))?);
        let (status, stdout, _) = receiver_response(command).await?;
        // A failed/unknown observation grants no release authority.
        if !status.success() {
            return Ok(PrimaryObservation::Live);
        }
        #[derive(serde::Deserialize)]
        struct Observation {
            canonical_database: String,
            primary_owner: PrimaryOwner,
            phase: String,
        }
        let observed: Observation = serde_json::from_slice(&stdout)
            .map_err(|error| contract(format!("managed primary observation: {error}")))?;
        if observed.canonical_database != self.selected.database()
            || observed.primary_owner.scope.unit != self.owner.scope.unit
            || observed.primary_owner.scope.group != self.owner.scope.group
            || observed.primary_owner.scope.invocation != self.owner.scope.invocation
            || observed.primary_owner.scope_inode != self.owner.scope_inode
        {
            return Err(contract("managed primary observation identity differs"));
        }
        Ok(match observed.phase.as_str() {
            "complete" => PrimaryObservation::Drained,
            "mismatch" => PrimaryObservation::OwnerChanged,
            _ => PrimaryObservation::Live,
        })
    }
}
#[cfg(test)]
mod receiver_tests {
    use super::*;
    #[tokio::test]
    async fn primary_response_bound_refuses_a_stream_without_an_end() {
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            receiver_output(tokio::io::repeat(0)),
        )
        .await
        .unwrap();
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn primary_response_retains_exact_bounded_readiness_bytes() {
        let bytes = b"{\"ready\":true}";
        assert_eq!(receiver_output(bytes.as_slice()).await.unwrap(), bytes);
    }
    #[tokio::test]
    async fn primary_response_output_refusal_reaps_its_actual_control() {
        let directory = tempfile::tempdir().unwrap();
        let pid = directory.path().join("pid");
        let mut command = tokio::process::Command::new("/bin/sh");
        command
            .arg("-c")
            .arg("echo $$ > \"$1\"; while :; do printf '0123456789abcdef'; done")
            .arg("control")
            .arg(&pid);
        let result = tokio::time::timeout(Duration::from_secs(5), receiver_response(command))
            .await
            .unwrap();
        assert!(result.is_err());
        let pid: u32 = std::fs::read_to_string(pid)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert!(
            !std::path::Path::new(&format!("/proc/{pid}")).exists(),
            "refused control was reaped before returning"
        );
    }
}
// Existing service loop, isolated from canonical discovery for focused
// scheduling controls. The lane itself still owns all claims and effects.
#[cfg(test)]
async fn serve_lanes<F, Fut>(
    settings: WorkerSettings,
    width: usize,
    stop: &crate::CancelSource,
    mut work: F,
) -> Result<usize, WorkflowError>
where
    F: FnMut(bool, crate::CancelSource) -> Fut,
    Fut: Future<Output = Result<Processed, WorkflowError>>,
{
    serve_owned_lanes(
        settings,
        vec![(); width],
        width,
        stop,
        move |(), recovery, drain, _publication| {
            let future = work(recovery, drain);
            async move { ((), future.await) }
        },
    )
    .await
    .0
}
/// The actual lane state is returned after every drained action, including errors.
async fn serve_owned_lanes<S, F, Fut>(
    settings: WorkerSettings,
    available: Vec<S>,
    width: usize,
    stop: &crate::CancelSource,
    mut work: F,
) -> (Result<usize, WorkflowError>, Vec<S>)
where
    F: FnMut(S, bool, crate::CancelSource, PublicationHandoff) -> Fut,
    Fut: Future<Output = (S, Result<Processed, WorkflowError>)>,
{
    async fn tagged<S, Fut: Future<Output = (S, Result<Processed, WorkflowError>)>>(
        lane: usize,
        future: Fut,
    ) -> (usize, S, Result<Processed, WorkflowError>) {
        let (owner, result) = future.await;
        (lane, owner, result)
    }
    let mut available = available.into_iter().enumerate().collect::<Vec<_>>();
    let total = available.len();
    let mut states = (0..total)
        .map(|_| publication::State::Idle)
        .collect::<Vec<_>>();
    let mut waiting = std::collections::VecDeque::with_capacity(total);
    let (ready, mut readiness) = tokio::sync::mpsc::channel::<publication::Ready>(total.max(1));
    let mut count = 0usize;
    let mut last_recovery = None;
    let drain = crate::CancelSource::new();
    while !stop.token().is_cancelled() && settings.maximum_actions.is_none_or(|limit| count < limit)
    {
        let mut pending = FuturesUnordered::new();
        let mut failure = None;
        let mut running = 0usize;
        let mut publishing = 0usize;
        let mut progressed = false;
        let mut refill = true;
        loop {
            // Only acknowledgement of a joined scientific owner releases a
            // Running slot. A full publication stage keeps its source owner in
            // Waiting and applies finite backpressure without another claim.
            while publishing < width && !drain.token().is_cancelled() {
                let Some(lane) = waiting.pop_front() else {
                    break;
                };
                if let publication::State::Waiting(ack) =
                    std::mem::replace(&mut states[lane], publication::State::Publishing)
                {
                    if ack.is_closed() {
                        // The cancelled owner still occupies its Running slot
                        // until its existing terminal cleanup actually returns.
                        states[lane] = publication::State::Running;
                    } else {
                        running -= 1;
                        publishing += 1;
                        let _ = ack.send(());
                    }
                }
            }
            while refill
                && running < width
                && !drain.token().is_cancelled()
                && settings
                    .maximum_actions
                    .is_none_or(|limit| count.checked_add(pending.len()).is_some_and(|n| n < limit))
            {
                let Some((lane, owner)) = available.pop() else {
                    break;
                };
                states[lane] = publication::State::Running;
                running += 1;
                let recovery = take_recovery(
                    &mut last_recovery,
                    tokio::time::Instant::now(),
                    settings.recovery,
                );
                let handoff = PublicationHandoff::new(lane, ready.clone(), drain.clone());
                pending.push(tagged(lane, work(owner, recovery, drain.clone(), handoff)));
            }
            if pending.is_empty() {
                break;
            }
            enum Event<S> {
                Ready(publication::Ready),
                Done(usize, S, Result<Processed, WorkflowError>),
                Poll,
                Stop,
            }
            let event = tokio::select! {
                biased;
                ()=stop.cancelled(),if !drain.token().is_cancelled()=>Event::Stop,
                Some(ready)=readiness.recv()=>Event::Ready(ready),
                Some((lane,owner,result))=pending.next()=>Event::Done(lane,owner,result),
                ()=tokio::time::sleep(settings.poll),if !refill && running<width
                    && !available.is_empty() && !drain.token().is_cancelled()=>Event::Poll,
            };
            match event {
                Event::Stop => {
                    drain.cancel();
                    refill = false;
                }
                Event::Poll => refill = true,
                Event::Ready(ready) => {
                    if let Some(state) = states.get_mut(ready.lane) {
                        if matches!(state, publication::State::Running) {
                            *state = publication::State::Waiting(ready.acknowledged);
                            waiting.push_back(ready.lane);
                            // Running work has completed. A distinct idle lane
                            // may refill once this handoff obtains publication.
                            refill = true;
                        } else {
                            failure.get_or_insert_with(|| {
                                contract("worker duplicate publication handoff")
                            });
                            drain.cancel();
                        }
                    } else {
                        failure.get_or_insert_with(|| contract("worker publication lane absent"));
                        drain.cancel();
                    }
                }
                Event::Done(lane, owner, result) => {
                    match std::mem::replace(&mut states[lane], publication::State::Idle) {
                        publication::State::Publishing => publishing -= 1,
                        publication::State::Running => running -= 1,
                        publication::State::Waiting(_) => {
                            running -= 1;
                            waiting.retain(|waiting_lane| *waiting_lane != lane);
                        }
                        publication::State::Idle => {
                            failure.get_or_insert_with(|| contract("worker idle lane completed"));
                            drain.cancel();
                        }
                    }
                    available.push((lane, owner));
                    refill = matches!(&result,Ok(value) if !matches!(value,Processed::Idle));
                    match result {
                        Ok(Processed::Idle) => {
                            #[cfg(all(
                                test,
                                feature = "canonical-tests",
                                feature = "solver-ipopt"
                            ))]
                            publication::idle();
                        }
                        Ok(Processed::Maintenance { .. }) => progressed = true,
                        Ok(_) => {
                            progressed = true;
                            match count.checked_add(1) {
                                Some(next) => count = next,
                                None => {
                                    failure.get_or_insert_with(|| {
                                        contract("worker processed count overflow")
                                    });
                                    drain.cancel();
                                }
                            }
                        }
                        Err(error) => {
                            if failure.is_none() {
                                failure = Some(error);
                            }
                            drain.cancel();
                        }
                    }
                }
            }
        }
        if let Some(error) = failure {
            return (
                Err(error),
                available.into_iter().map(|(_, owner)| owner).collect(),
            );
        }
        if !progressed {
            if settings.until_idle {
                break;
            }
            tokio::select! {_=tokio::time::sleep(settings.poll)=>{},()=stop.cancelled()=>{}}
        }
    }
    (
        Ok(count),
        available.into_iter().map(|(_, owner)| owner).collect(),
    )
}

fn take_recovery(
    last: &mut Option<tokio::time::Instant>,
    now: tokio::time::Instant,
    interval: Duration,
) -> bool {
    if last.is_none_or(|previous| now.duration_since(previous) >= interval) {
        *last = Some(now);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod lane_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn worker_refills_after_progress_even_when_another_lane_was_idle() {
        let stop = crate::CancelSource::new();
        let (release, wait) = tokio::sync::oneshot::channel();
        let mut release = Some(release);
        let mut wait = Some(wait);
        let mut issued = 0;
        let work = serve_lanes(
            WorkerSettings {
                maximum_actions: Some(3),
                until_idle: true,
                ..WorkerSettings::default()
            },
            3,
            &stop,
            |_, _| {
                let lane = issued;
                issued += 1;
                let wait = (lane == 0).then(|| wait.take().unwrap());
                let release = (lane == 3).then(|| release.take().unwrap());
                async move {
                    if let Some(wait) = wait {
                        wait.await.unwrap();
                    }
                    if let Some(release) = release {
                        release.send(()).unwrap();
                    }
                    Ok(if lane == 1 {
                        Processed::Idle
                    } else {
                        Processed::Settled {
                            point: lane.to_string(),
                        }
                    })
                }
            },
        );
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), work)
                .await
                .unwrap()
                .unwrap(),
            3
        );
        assert_eq!(issued, 4, "idle lane did not consume an action");
    }

    #[tokio::test]
    async fn mandatory_readiness_progress_preserves_until_idle_and_action_budget() {
        let stop = crate::CancelSource::new();
        let mut issued = 0;
        let count = serve_lanes(
            WorkerSettings {
                maximum_actions: Some(1),
                until_idle: true,
                ..WorkerSettings::default()
            },
            1,
            &stop,
            |_, _| {
                issued += 1;
                let action = issued;
                async move {
                    Ok(if action < 4 {
                        Processed::Maintenance {
                            study: "bounded".into(),
                            examined: 64,
                        }
                    } else {
                        Processed::Settled {
                            point: "ready".into(),
                        }
                    })
                }
            },
        )
        .await
        .unwrap();
        assert_eq!((count, issued), (1, 4));
    }
    #[tokio::test]
    async fn publication_backpressure_keeps_two_owned_lanes_and_drains_cancelled_handoff() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let stop = crate::CancelSource::new();
        let started = Arc::new(AtomicUsize::new(0));
        let changed = Arc::new(tokio::sync::Notify::new());
        let gate = Arc::new(tokio::sync::Notify::new());
        let publishing = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicUsize::new(0));
        let cancelled_changed = Arc::new(tokio::sync::Notify::new());
        let serving = serve_owned_lanes(WorkerSettings::default(), vec![7_u64, 11], 1, &stop, {
            let started = started.clone();
            let changed = changed.clone();
            let gate = gate.clone();
            let publishing = publishing.clone();
            let cancelled = cancelled.clone();
            let cancelled_changed = cancelled_changed.clone();
            move |owner, _, _, handoff| {
                let started = started.clone();
                let changed = changed.clone();
                let gate = gate.clone();
                let publishing = publishing.clone();
                let cancelled = cancelled.clone();
                let cancelled_changed = cancelled_changed.clone();
                async move {
                    let ordinal = started.fetch_add(1, Ordering::AcqRel);
                    changed.notify_one();
                    if !handoff.ready().await.unwrap() {
                        cancelled.fetch_add(1, Ordering::AcqRel);
                        cancelled_changed.notify_one();
                        return (
                            owner,
                            Ok(Processed::Settled {
                                point: format!("cancelled-{owner}"),
                            }),
                        );
                    }
                    assert_eq!(
                        publishing.fetch_add(1, Ordering::AcqRel),
                        0,
                        "publication width stays one"
                    );
                    if ordinal == 0 {
                        gate.notified().await;
                    }
                    publishing.fetch_sub(1, Ordering::AcqRel);
                    (
                        owner,
                        Ok(Processed::Settled {
                            point: format!("owned-{owner}"),
                        }),
                    )
                }
            }
        });
        let control = async {
            let reached = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    let notified = changed.notified();
                    if started.load(Ordering::Acquire) == 2 {
                        break;
                    }
                    notified.await;
                }
            })
            .await
            .is_ok();
            stop.cancel();
            let cancelled_while_full = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    let notified = cancelled_changed.notified();
                    if cancelled.load(Ordering::Acquire) == 1 {
                        break;
                    }
                    notified.await;
                }
            })
            .await
            .is_ok();
            let retained = publishing.load(Ordering::Acquire);
            gate.notify_one();
            (reached, cancelled_while_full, retained)
        };
        let ((result, mut owners), (reached, cancelled_while_full, retained)) =
            tokio::join!(serving, control);
        assert!(
            reached,
            "second science proceeds while first publication owns its gate"
        );
        assert!(
            cancelled_while_full,
            "the unadmitted owner settles cancellation before the held publisher releases"
        );
        assert_eq!(
            retained, 1,
            "cancellation neither bypasses capacity nor drops the original publisher"
        );
        assert_eq!(
            started.load(Ordering::Acquire),
            2,
            "no third owner escapes backpressure"
        );
        assert_eq!(result.unwrap(), 2);
        owners.sort();
        assert_eq!(owners, vec![7, 11]);
        assert_eq!(publishing.load(Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn worker_polls_new_arrivals_while_an_older_publication_remains_held() {
        use std::sync::atomic::AtomicBool;
        let stop = crate::CancelSource::new();
        let arrived = Arc::new(AtomicBool::new(false));
        let issued = Arc::new(AtomicUsize::new(0));
        let scientific = Arc::new(AtomicUsize::new(0));
        let changed = Arc::new(tokio::sync::Notify::new());
        let gate = Arc::new(tokio::sync::Notify::new());
        let serving = serve_owned_lanes(
            WorkerSettings {
                poll: Duration::from_millis(1),
                ..Default::default()
            },
            vec![7_u64, 11],
            1,
            &stop,
            {
                let arrived = arrived.clone();
                let issued = issued.clone();
                let scientific = scientific.clone();
                let changed = changed.clone();
                let gate = gate.clone();
                move |owner, _, _, handoff| {
                    let ordinal = issued.fetch_add(1, Ordering::AcqRel);
                    let has_work = ordinal == 0 || arrived.swap(false, Ordering::AcqRel);
                    let scientific = scientific.clone();
                    let changed = changed.clone();
                    let gate = gate.clone();
                    async move {
                        if !has_work {
                            changed.notify_one();
                            return (owner, Ok(Processed::Idle));
                        }
                        scientific.fetch_add(1, Ordering::AcqRel);
                        changed.notify_one();
                        if handoff.ready().await.unwrap() && ordinal == 0 {
                            gate.notified().await;
                        }
                        (
                            owner,
                            Ok(Processed::Settled {
                                point: format!("owned-{owner}"),
                            }),
                        )
                    }
                }
            },
        );
        let control = async {
            let idle = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    let notified = changed.notified();
                    if issued.load(Ordering::Acquire) >= 2 {
                        break;
                    }
                    notified.await;
                }
            })
            .await
            .is_ok();
            arrived.store(true, Ordering::Release);
            let new_science = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    let notified = changed.notified();
                    if scientific.load(Ordering::Acquire) >= 2 {
                        break;
                    }
                    notified.await;
                }
            })
            .await
            .is_ok();
            stop.cancel();
            gate.notify_one();
            (idle, new_science)
        };
        let ((result, mut owners), (idle, new_science)) = tokio::join!(serving, control);
        assert!(
            idle && new_science,
            "late independent work is discovered without waiting for the first publication"
        );
        assert_eq!(result.unwrap(), 2);
        owners.sort();
        assert_eq!(owners, vec![7, 11]);
    }

    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn lost_publication_ack_keeps_original_failure_and_separate_durable_uncertainty() {
        use crate::workflow::{RunReport, RunRequest, RunResult};
        let runtime = super::super::durable_tests::durable_runtime();
        for report_failed in [false, true] {
            let prior = Arc::new(contract("original scientific or completion failure"));
            let mut result = RunResult::joined(
                RunId::from_id(SemanticId::NIL),
                runtime.clone(),
                RunRequest::Modeling(vec![]),
                None,
                if report_failed {
                    Err(prior.clone())
                } else {
                    Ok(RunReport::Modeling(vec![]))
                },
            );
            result.original_completion_captured = true;
            result.completion = Err(prior.clone());
            let mut attempt = DurableAttempt::new(operations(&runtime).unwrap()).unwrap();
            let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
            attempt.set_worker_publication(Some(PublicationHandoff::new(
                0,
                sender,
                crate::CancelSource::new(),
            )));
            let (result, ()) = tokio::join!(result.finished(Some(attempt), false), async {
                let ready = receiver.recv().await.unwrap();
                drop(ready.acknowledged);
            });
            assert!(
                !result.invocation_cancelled,
                "a lost ACK is not cancellation evidence"
            );
            assert!(Arc::ptr_eq(result.completion.as_ref().unwrap_err(), &prior));
            if report_failed {
                assert!(Arc::ptr_eq(result.report.as_ref().unwrap_err(), &prior));
            } else {
                assert!(result.report.is_ok());
            }
            let RunDurability::Durable(record) = result.durability else {
                panic!("durable failure absent")
            };
            let cleanup = record.attempt.as_ref().unwrap_err();
            assert!(
                !Arc::ptr_eq(cleanup, &prior),
                "cleanup failure remains independently observable"
            );
            assert!(
                cleanup
                    .to_string()
                    .contains("capacity acknowledgement unavailable")
            );
            assert!(
                record.manifest.is_none()
                    && record.completion.is_none()
                    && record.solutions.is_empty()
            );
        }
        runtime.shared.math().shutdown_native_lanes().await;
    }

    #[tokio::test]
    async fn worker_returns_actual_lane_owner_after_failure_and_drain() {
        let stop = crate::CancelSource::new();
        let (result, owners) = serve_owned_lanes(
            WorkerSettings::default(),
            vec![7_u64, 11],
            2,
            &stop,
            |owner, _, drain, _publication| async move {
                if owner == 11 {
                    (owner, Err(contract("owned lane failure")))
                } else {
                    drain.cancelled().await;
                    (owner, Ok(Processed::Idle))
                }
            },
        )
        .await;
        assert!(result.is_err());
        let mut owners = owners;
        owners.sort();
        assert_eq!(owners, vec![7, 11]);
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn periodic_recovery_rotates_past_an_earlier_study_that_stays_busy() {
        use pse_model::study::{OccurrenceKey, PointPolicy, SeedNeed, StartPolicy};
        use pse_operations::{canonical_execution::RunRequest, canonical_studies::NewOccurrence};
        let runtime = crate::workflow::durable_tests::durable_runtime();
        let store = runtime.canonical_store().unwrap();
        let revision = store
            .edit("recovery-rotation", None, "source", &[])
            .await
            .unwrap();
        let request = |key: &str| RunRequest {
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        for name in ["a-busy-recovery", "z-later-recovery"] {
            let point = NewOccurrence {
                policy: PointPolicy {
                    key: OccurrenceKey(0),
                    dependencies: vec![],
                    start: StartPolicy::Fresh,
                    seed_need: SeedNeed::NotNeeded,
                    attempt_limit: 1,
                },
                descriptor: vec![42],
                run: request(&format!("{name}-point")),
            };
            store
                .create_study(
                    name,
                    &request(&format!("{name}-parent")),
                    &[],
                    &[point],
                    &|| false,
                )
                .await
                .unwrap()
                .unwrap();
        }
        let position = tokio::sync::Mutex::new(None);
        let cancel = crate::CancelSource::new();
        let mut visited = Vec::new();
        for _ in 0..4 {
            runtime
                .recover_next_study(&position, None, &cancel)
                .await
                .unwrap();
            visited.push(position.lock().await.clone().unwrap());
            assert!(
                !store
                    .study_candidates("a-busy-recovery", None)
                    .await
                    .unwrap()
                    .is_empty(),
                "earlier study remains dispatchable throughout independent recovery"
            );
        }
        assert_eq!(
            visited,
            [
                "a-busy-recovery",
                "z-later-recovery",
                "a-busy-recovery",
                "z-later-recovery"
            ]
        );
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn periodic_recovery_preserves_local_cancelled_owner_until_turn_release() {
        use pse_model::study::{OccurrenceKey, PointPolicy, SeedNeed, StartPolicy};
        use pse_operations::{canonical_execution::RunRequest, canonical_studies::NewOccurrence};
        let runtime = crate::workflow::durable_tests::durable_runtime();
        let store = runtime.canonical_store().unwrap();
        let revision = store
            .edit("local-recovery", None, "source", &[])
            .await
            .unwrap();
        let request = |key: &str| RunRequest {
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        let occurrence = NewOccurrence {
            policy: PointPolicy {
                key: OccurrenceKey(0),
                dependencies: vec![],
                start: StartPolicy::Fresh,
                seed_need: SeedNeed::NotNeeded,
                attempt_limit: 1,
            },
            descriptor: vec![42],
            run: request("local-recovery-point"),
        };
        store
            .create_study(
                "local-recovery-study",
                &request("local-recovery-parent"),
                &[],
                &[occurrence],
                &|| false,
            )
            .await
            .unwrap()
            .unwrap();
        let key =
            pse_operations::canonical_studies::point_key("local-recovery-study", OccurrenceKey(0));
        let group = CandidateGroup::default();
        let owner = group.acquire(&key).unwrap();
        let scope = store.study_scope(&key).await.unwrap();
        let claim = store
            .claim_study_point(
                &scope,
                None,
                "local-recovery-claim",
                "owner",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        store.cancel_study("local-recovery-study").await.unwrap();
        let position = tokio::sync::Mutex::new(None);
        let cancel = crate::CancelSource::new();
        let action = runtime
            .recover_next_study(&position, Some(&group), &cancel)
            .await
            .unwrap();
        assert!(!matches!(action, Some(Processed::Settled { .. })));
        let before = store
            .canonical_attempt(claim.fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(!before.terminal() && !before.closed());
        let run = store
            .canonical_run(claim.fence.run())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(run.current_generation, claim.fence.generation());
        assert!(
            store
                .canonical_study_point(&key)
                .await
                .unwrap()
                .unwrap()
                .assigned
        );
        // A different worker has no local turn information. Cancellation still
        // does not authorize it to fence the unexpired native owner.
        let independent_group = CandidateGroup::default();
        *position.lock().await = None;
        let action = runtime
            .recover_next_study(&position, Some(&independent_group), &cancel)
            .await
            .unwrap();
        assert!(!matches!(action, Some(Processed::Settled { .. })));
        assert!(
            !store
                .canonical_attempt(claim.fence.attempt())
                .await
                .unwrap()
                .unwrap()
                .closed()
        );
        assert_eq!(
            store
                .canonical_run(claim.fence.run())
                .await
                .unwrap()
                .unwrap()
                .current_generation,
            claim.fence.generation()
        );
        // Expiry permits independent crash recovery, but this worker still knows
        // its own native lane has not joined and must retain that exact turn.
        pse_operations::testing::expire_acknowledged_attempt(store, &claim.fence)
            .await
            .unwrap();
        *position.lock().await = None;
        let action = runtime
            .recover_next_study(&position, Some(&group), &cancel)
            .await
            .unwrap();
        assert!(!matches!(action, Some(Processed::Settled { .. })));
        assert!(
            !store
                .canonical_attempt(claim.fence.attempt())
                .await
                .unwrap()
                .unwrap()
                .closed()
        );
        // The real worker drops this turn only after its issued native owner and
        // finish/observation have joined. A departed owner then permits recovery.
        drop(owner);
        *position.lock().await = None;
        assert!(
            matches!(runtime.recover_next_study(&position, Some(&group), &cancel).await.unwrap(),
            Some(Processed::Settled { point }) if point == key)
        );
        let terminal = store
            .canonical_attempt(claim.fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(terminal.terminal());
        assert_eq!(
            terminal.outcome.as_ref().map(|value| value.as_str()),
            Some("cancelled")
        );
        let point = store.canonical_study_point(&key).await.unwrap().unwrap();
        assert!(point.settled && !point.assigned);
        let facts = pse_operations::canonical_studies::point_facts(&point).unwrap();
        assert!(!facts.native_started && !facts.scientific.usable);
    }
    #[tokio::test]
    async fn worker_refill_respects_total_action_limit() {
        let stop = crate::CancelSource::new();
        let mut issued = 0;
        let count = serve_lanes(
            WorkerSettings {
                maximum_actions: Some(5),
                ..WorkerSettings::default()
            },
            4,
            &stop,
            |_, _| {
                issued += 1;
                async {
                    Ok(Processed::Settled {
                        point: "bounded".into(),
                    })
                }
            },
        )
        .await
        .unwrap();
        assert_eq!((count, issued), (5, 5));
    }

    #[tokio::test]
    async fn worker_failure_drains_all_issued_lanes_before_returning() {
        let stop = crate::CancelSource::new();
        let drained = Arc::new(AtomicUsize::new(0));
        let mut issued = 0;
        let result = serve_lanes(WorkerSettings::default(), 2, &stop, |_, drain| {
            let lane = issued;
            issued += 1;
            let drained = drained.clone();
            async move {
                if lane == 0 {
                    return Err(contract("deliberate lane failure"));
                }
                drain.cancelled().await;
                drained.fetch_add(1, Ordering::SeqCst);
                Ok(Processed::Idle)
            }
        })
        .await;
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("deliberate lane failure")
        );
        assert_eq!(issued, 2);
        assert_eq!(drained.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn worker_stop_drains_every_issued_lane() {
        let stop = crate::CancelSource::new();
        let drained = Arc::new(AtomicUsize::new(0));
        let (started, notice) = tokio::sync::oneshot::channel();
        let mut started = Some(started);
        let mut issued = 0;
        let work = serve_lanes(WorkerSettings::default(), 2, &stop, |_, drain| {
            issued += 1;
            if issued == 2 {
                started.take().unwrap().send(()).unwrap();
            }
            let drained = drained.clone();
            async move {
                drain.cancelled().await;
                drained.fetch_add(1, Ordering::SeqCst);
                Ok(Processed::Idle)
            }
        });
        let (count, ()) = tokio::join!(work, async {
            notice.await.unwrap();
            stop.cancel();
        });
        assert_eq!(count.unwrap(), 0);
        assert_eq!(issued, 2);
        assert_eq!(drained.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn worker_continuous_refill_keeps_recovery_due_without_a_round_barrier() {
        let now = tokio::time::Instant::now();
        let interval = Duration::from_secs(30);
        let mut last = None;
        assert!(take_recovery(&mut last, now, interval));
        assert!(!take_recovery(
            &mut last,
            now + Duration::from_secs(29),
            interval
        ));
        assert!(take_recovery(&mut last, now + interval, interval));
        assert!(!take_recovery(&mut last, now + interval, interval));
    }
}

fn operations(runtime: &Runtime) -> Result<&Operations, WorkflowError> {
    match &runtime.durability {
        Durability::Durable(operations) => Ok(operations),
        Durability::Ephemeral => Err(contract("canonical study worker requires durable runtime")),
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEntry {
    path: String,
    chunks: u64,
    bytes: u64,
    digest: String,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhysicalManifest {
    version: u8,
    identity: ContentHash,
    documents: Vec<SourceEntry>,
}
impl Operations {
    /// Store exact authored document bytes in bounded canonical objects.
    pub async fn put_sources(
        &self,
        sources: &BTreeMap<String, Vec<u8>>,
    ) -> Result<PhysicalSource, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let identity = package_checksum(sources);
                let problem = format!("physical:documents:{identity}");
                let reservation =
                    pse_columnar::MemoryConsumer::new("canonical:source-package-publication")
                        .register(&self.pool);
                let (bytes, chunks) = sources
                    .values()
                    .try_fold((0_usize, 1_usize), |(bytes, chunks), source| {
                        Some((
                            bytes.checked_add(source.len())?,
                            chunks.checked_add(source.len().div_ceil(
                                pse_operations::canonical_execution::RESULT_BATCH_BYTES,
                            ))?,
                        ))
                    })
                    .ok_or_else(|| contract("physical package extent overflow"))?;
                if chunks > pse_operations::canonical_staging::SOURCE_PACKAGE_EDITS {
                    return Err(contract("physical package exceeds source edit admission"));
                }
                reservation
                    .try_grow(
                        bytes
                            .checked_mul(4)
                            .and_then(|bytes| {
                                chunks
                                    .checked_mul(16384)
                                    .and_then(|metadata| bytes.checked_add(metadata))
                            })
                            .ok_or_else(|| contract("physical package allocation overflow"))?,
                    )
                    .map_err(pse_engine::EngineError::from)?;
                let mut edits = Vec::with_capacity(chunks);
                let mut documents = Vec::new();
                for (path, bytes) in sources {
                    let chunks = bytes
                        .len()
                        .div_ceil(pse_operations::canonical_execution::RESULT_BATCH_BYTES)
                        as u64;
                    let path_key = pse_ids::document::of(
                        pse_ids::Frame::CanonicalPayloadV1,
                        &("physical.document.path.v1", path),
                    )
                    .map_err(|e| contract(e.to_string()))?;
                    for (ordinal, payload) in bytes
                        .chunks(pse_operations::canonical_execution::RESULT_BATCH_BYTES)
                        .enumerate()
                    {
                        let logical = format!("document:{path_key}:{ordinal:020}");
                        let edit = super::durable::source_edit(
                            logical.clone(),
                            "physical:documents".into(),
                            logical,
                            "physical:documents:bytes:v1",
                            payload.to_vec(),
                        );
                        edits.push(edit);
                    }
                    documents.push(SourceEntry {
                        path: path.clone(),
                        chunks,
                        bytes: bytes.len() as u64,
                        digest: pse_operations::canonical_execution::result_payload_digest(bytes),
                    });
                }
                let payload = serde_json::to_vec(&PhysicalManifest {
                    version: 1,
                    identity,
                    documents,
                })
                .map_err(|e| contract(e.to_string()))?;
                if payload.len() > 128 * 1024 {
                    return Err(contract(
                        "physical document manifest exceeds metadata bound",
                    ));
                }
                let edit = super::durable::source_edit(
                    "manifest".into(),
                    "physical:documents".into(),
                    "manifest".into(),
                    "physical:documents:manifest:v1",
                    payload,
                );
                edits.push(edit);
                let revision = self
                    .store()
                    .edit(
                        &problem,
                        None,
                        &format!("physical:{identity}:package"),
                        &edits,
                    )
                    .await?;
                Ok(PhysicalSource {
                    revision: revision.key,
                    identity,
                })
            })
            .await
    }
    /// Load only the selected protected document-kind revision, preserving original bytes.
    pub async fn sources(
        &self,
        source: &PhysicalSource,
    ) -> Result<Arc<pse_columnar::Leased<BTreeMap<String, Vec<u8>>>>, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let revision = self
                    .store()
                    .revision(&source.revision)
                    .await?
                    .ok_or_else(|| contract("physical source revision absent"))?;
                if revision.problem != format!("physical:documents:{}", source.identity) {
                    return Err(contract("physical document receipt interpretation differs"));
                }
                let protection = self
                    .store()
                    .protect(revision, Duration::from_secs(3600))
                    .await?;
                let mut read = pse_operations::canonical_selection::SelectedRead::new(protection);
                let result = async {
                    let scratch = datafusion::execution::memory_pool::MemoryConsumer::new(
                        "canonical:physical-documents",
                    )
                    .register(&self.pool);
                    scratch
                        .try_grow(
                            pse_operations::canonical_staging::SELECTED_OBJECT_HEADER_SCRATCH
                                + 2 * pse_operations::canonical_execution::RESULT_BATCH_BYTES,
                        )
                        .map_err(pse_engine::EngineError::from)?;
                    let memberships = self
                        .store()
                        .resolve_logicals(&mut read, &["manifest".into()])
                        .await?;
                    let member = memberships
                        .first()
                        .ok_or_else(|| contract("physical document manifest absent"))?;
                    let object = self
                        .store()
                        .selected_object(read.selection(), &member.version)
                        .await?
                        .ok_or_else(|| contract("physical document manifest object absent"))?;
                    if object.kind != "physical:documents:manifest:v1"
                        || object.payload.len()
                            > pse_operations::canonical_execution::EXECUTION_METADATA_BYTES
                    {
                        return Err(contract("physical source kind differs"));
                    }
                    let manifest: PhysicalManifest =
                        serde_json::from_slice(object.payload.as_slice())
                            .map_err(|e| contract(e.to_string()))?;
                    if manifest.version != 1 || manifest.identity != source.identity {
                        return Err(contract(
                            "physical document manifest interpretation differs",
                        ));
                    }
                    let bytes = manifest
                        .documents
                        .iter()
                        .try_fold(0_usize, |sum, document| {
                            usize::try_from(document.bytes)
                                .ok()
                                .and_then(|bytes| sum.checked_add(bytes))
                        })
                        .ok_or_else(|| contract("physical source declared extent overflow"))?;
                    let reservation =
                        pse_columnar::MemoryConsumer::new("canonical:physical-document-buffers")
                            .register(&self.pool);
                    reservation
                        .try_grow(
                            bytes
                                .checked_mul(4)
                                .and_then(|bytes| {
                                    object
                                        .payload
                                        .len()
                                        .checked_mul(8)
                                        .and_then(|metadata| bytes.checked_add(metadata))
                                })
                                .ok_or_else(|| {
                                    contract("physical source allocation extent overflow")
                                })?,
                        )
                        .map_err(pse_engine::EngineError::from)?;
                    if manifest
                        .documents
                        .windows(2)
                        .any(|pair| pair[0].path >= pair[1].path)
                    {
                        return Err(contract("physical document manifest paths differ"));
                    }
                    let mut sources = BTreeMap::new();
                    let mut requests = Vec::new();
                    for document in &manifest.documents {
                        if document.chunks
                            != document.bytes.div_ceil(
                                pse_operations::canonical_execution::RESULT_BATCH_BYTES as u64,
                            )
                        {
                            return Err(contract("physical document coverage differs"));
                        }
                        if document.chunks
                            > pse_operations::canonical_staging::SOURCE_PACKAGE_EDITS as u64
                        {
                            return Err(contract("physical document chunk admission exceeded"));
                        }
                        let path_key = pse_ids::document::of(
                            pse_ids::Frame::CanonicalPayloadV1,
                            &("physical.document.path.v1", &document.path),
                        )
                        .map_err(|e| contract(e.to_string()))?;
                        let mut bytes = Vec::new();
                        bytes
                            .try_reserve_exact(
                                usize::try_from(document.bytes)
                                    .map_err(|_| contract("physical document extent"))?,
                            )
                            .map_err(|_| contract("physical document allocation refused"))?;
                        sources.insert(document.path.clone(), bytes);
                        for ordinal in 0..document.chunks {
                            requests.push((
                                &document.path,
                                format!("document:{path_key}:{ordinal:020}"),
                                (document.bytes
                                    - ordinal
                                        * pse_operations::canonical_execution::RESULT_BATCH_BYTES
                                            as u64)
                                    .min(
                                        pse_operations::canonical_execution::RESULT_BATCH_BYTES
                                            as u64,
                                    ) as usize,
                            ));
                            if requests.len()
                                > pse_operations::canonical_staging::SOURCE_PACKAGE_EDITS
                            {
                                return Err(contract("physical package chunk admission exceeded"));
                            }
                        }
                    }
                    for group in
                        requests.chunks(pse_operations::canonical_staging::SELECTED_OBJECT_BATCH)
                    {
                        let logicals = group
                            .iter()
                            .map(|(_, logical, _)| logical.clone())
                            .collect::<Vec<_>>();
                        let members = self.store().resolve_logicals(&mut read, &logicals).await?;
                        let versions = logicals
                            .iter()
                            .map(|logical| {
                                members
                                    .iter()
                                    .find(|member| &member.logical == logical)
                                    .map(|member| member.version.clone())
                                    .ok_or_else(|| contract("physical document chunk absent"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let batch = self
                            .store()
                            .selected_object_batch(read.selection(), &versions)
                            .await?;
                        let extents = batch
                            .extents()
                            .map(|(_, extent)| extent)
                            .collect::<Vec<_>>();
                        if extents
                            .iter()
                            .zip(group)
                            .any(|(extent, (_, _, expected))| extent != expected)
                        {
                            return Err(contract(
                                "physical document chunk manifest extent differs",
                            ));
                        }
                        let mut payloads = extents
                            .iter()
                            .map(|extent| {
                                let mut bytes = Vec::new();
                                bytes
                                    .try_reserve_exact(*extent)
                                    .map_err(|_| contract("physical chunk allocation refused"))?;
                                Ok(bytes)
                            })
                            .collect::<Result<Vec<_>, WorkflowError>>()?;
                        while let Some(object) = payloads
                            .iter()
                            .zip(&extents)
                            .position(|(payload, extent)| payload.len() < *extent)
                        {
                            let ordinal = (payloads[object].len()
                                / pse_operations::canonical_staging::SOURCE_BLOCK_BYTES)
                                as u64;
                            for (object, block) in self
                                .store()
                                .selected_object_blocks(&batch, object, ordinal)
                                .await?
                            {
                                if block.ordinal as usize
                                    * pse_operations::canonical_staging::SOURCE_BLOCK_BYTES
                                    != payloads[object].len()
                                {
                                    return Err(contract("physical grouped chunk order differs"));
                                }
                                payloads[object].extend_from_slice(block.payload.as_slice());
                            }
                        }
                        let objects = self
                            .store()
                            .finish_selected_object_batch(batch, payloads)
                            .await?;
                        for (object, (path, logical, length)) in objects.into_iter().zip(group) {
                            if object.kind != "physical:documents:bytes:v1"
                                || &object.logical != logical
                                || object.payload.len() != *length
                            {
                                return Err(contract(
                                    "physical document chunk kind/extent differs",
                                ));
                            }
                            sources
                                .get_mut(*path)
                                .ok_or_else(|| contract("physical document inventory differs"))?
                                .extend_from_slice(object.payload.as_slice());
                        }
                    }
                    drop(requests);
                    for document in manifest.documents {
                        let bytes = sources
                            .get(&document.path)
                            .ok_or_else(|| contract("physical document absent"))?;
                        if bytes.len() as u64 != document.bytes
                            || pse_operations::canonical_execution::result_payload_digest(bytes)
                                != document.digest
                        {
                            return Err(contract(
                                "physical document original bytes digest differs",
                            ));
                        }
                    }
                    if package_checksum(&sources) != source.identity {
                        return Err(contract("physical document package checksum differs"));
                    }
                    Ok::<_, WorkflowError>(Arc::new(pse_columnar::Leased::new(
                        Arc::new(sources),
                        pse_columnar::AllocationLease::new(reservation),
                    )))
                }
                .await;
                self.store().release(read.selection()).await?;
                result
            })
            .await
    }
}

impl Runtime {
    /// Activate or reuse the deployment's one verified primary group. The receiving
    /// observer has no native assistance lane and must fit its separate placement.
    pub(super) async fn ensure_managed_primary(
        &self,
        cancel: &crate::CancelSource,
    ) -> Result<ManagedPrimary, WorkflowError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        if cancel.token().is_cancelled() {
            return Err(crate::math::MathRuntimeError::Cancelled.into());
        }
        let store = operations(self)?.store();
        let selected = super::NativeExecutionSelection::from_store(store)?;
        let (receiver, profile) = selected.managed_primary()?;
        if self.shared.budget().memory_limit_bytes.get() > profile.observer_memory_bytes {
            return Err(contract(
                "durable observer pool exceeds its managed observer allocation",
            ));
        }
        if cancel.token().is_cancelled() {
            return Err(crate::math::MathRuntimeError::Cancelled.into());
        }
        let mut command = tokio::process::Command::new(&receiver.supervisor_executable);
        command
            .arg(&receiver.supervisor_script)
            .arg("ensure-primary")
            .arg("--state")
            .arg(selected.receiver_state())
            .arg("--canonical-database")
            .arg(store.database())
            .arg("--observer-pid")
            .arg(std::process::id().to_string());
        let response = receiver_response(command);
        let (status, stdout, stderr) = tokio::select! {
            output = tokio::time::timeout(Duration::from_millis(profile.admission_wait_ms), response) =>
                output.map_err(|_| contract("managed primary readiness deadline expired"))?
                    ?,
            () = cancel.cancelled() => return Err(crate::math::MathRuntimeError::Cancelled.into()),
        };
        if !status.success() {
            return Err(contract(format!(
                "managed primary startup: {}",
                String::from_utf8_lossy(&stderr)
            )));
        }
        #[derive(serde::Deserialize)]
        struct Ready {
            ready: bool,
            canonical_database: String,
            primary_owner: PrimaryOwner,
            diagnostics: Option<serde_json::Value>,
        }
        let ready: Ready = serde_json::from_slice(&stdout)
            .map_err(|error| contract(format!("managed primary readiness: {error}")))?;
        if !ready.ready || ready.canonical_database != selected.database() {
            return Err(contract("managed primary is not ready"));
        }
        selected.require_current()?;
        self.observations().observe_receiver_ready(selected.receiver_state(), selected.database(), ready.diagnostics);
        Ok(ManagedPrimary { selected, owner: ready.primary_owner })

        }).await
    }
    /// A definite decision rejection can demand a fresh shared-policy decision.
    /// Never replay an effect-free mutation after uncertain completion.
    pub(super) async fn settle_current_study_candidate(
        &self,
        scope: &pse_operations::canonical_studies::StudyScope,
        action: &pse_model::study::PointAction,
    ) -> Result<Option<pse_operations::canonical_studies::StudyPoint>, WorkflowError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        let deadline = tokio::time::Instant::now() + pse_operations::canonical::REQUEST_TIMEOUT;
        tokio::time::timeout_at(deadline, async {
            match self.settle_study_candidate(scope, action, None).await {
                Ok(point) => Ok(Some(point)),
                Err(error) => {
                    if !matches!(&error, WorkflowError::Canonical(error) if error.needs_fresh_premise()) {
                        return Err(error);
                    }
                    let current = self.canonical_store()?.study_scope(&scope.point().key).await?;
                    if current.point().revision != scope.point().revision
                        || current.point().assigned != scope.point().assigned
                        || current.point().settled != scope.point().settled
                        || current.generation() != scope.generation()
                        || current.cancelled() != scope.cancelled()
                        || current.predecessors().iter().map(|point| (&point.key, point.revision))
                            .ne(scope.predecessors().iter().map(|point| (&point.key, point.revision)))
                    {
                        // The server refused this old read set. The caller must
                        // rediscover the candidate and recompute its policy.
                        Ok(None)
                    } else {
                        Err(error)
                    }
                }
            }
        }).await.map_err(|_| WorkflowError::Canonical(pse_operations::canonical::CanonicalError::Timeout))?

        }).await
    }

    /// Claim a structurally ready candidate under its exact consumed revisions.
    pub async fn work_once(&self) -> Result<Processed, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async { Ok(self.work_once_with_result().await?.0) })
            .await
    }
    /// Return one drained native result, whose owner remains retained by this view.
    pub async fn work_once_with_result(
        &self,
    ) -> Result<(Processed, Option<Arc<super::RunResult>>), WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let mut lane = super::study::StudyCandidateLane::new();
                let recovery = tokio::sync::Mutex::new(None);
                let result = self
                    .work_once_scoped(
                        true,
                        &recovery,
                        None,
                        &mut lane,
                        &crate::CancelSource::new(),
                        &mut None,
                    )
                    .await;
                lane.close().await?;
                result
            })
            .await
    }
    async fn work_once_scoped(
        &self,
        recovery: bool,
        recovery_position: &tokio::sync::Mutex<Option<String>>,
        group: Option<&CandidateGroup>,
        lane: &mut super::study::StudyCandidateLane,
        cancel: &crate::CancelSource,
        publication: &mut Option<PublicationHandoff>,
    ) -> Result<(Processed, Option<Arc<super::RunResult>>), WorkflowError> {
        let _observation = pse_diagnostics::observation::phase("managed.worker.work_once");
        let operations = operations(self)?;
        match operations.store().check_write_admission() {
            Ok(()) => {}
            Err(pse_operations::canonical::CanonicalError::Quiesced) => {
                return Ok((Processed::Idle, None));
            }
            Err(error) => return Err(error.into()),
        }
        let mut recovery_maintenance = None;
        if recovery
            && !cancel.token().is_cancelled()
            && let Some(action) = self
                .worker_action(|| self.recover_next_study(recovery_position, group, cancel))
                .await?
        {
            if matches!(action, Processed::Maintenance { .. }) {
                // Bounded repair can leave notifications for discovery to
                // advance. Returning here would repeatedly verify that same
                // pending queue without ever dispatching an eligible point.
                recovery_maintenance = Some(action);
            } else {
                return Ok((action, None));
            }
        }
        let mut after = None;
        loop {
            if cancel.token().is_cancelled() {
                return Ok((Processed::Idle, None));
            }
            let studies = operations.store().study_page(after.as_deref()).await?;
            if studies.is_empty() {
                return Ok((recovery_maintenance.unwrap_or(Processed::Idle), None));
            }
            for study in studies {
                after = Some(study.key.clone());
                let readiness = operations
                    .store()
                    .advance_study_readiness(&study.key)
                    .await?;
                let maintenance = readiness.examined > 0 || readiness.pending;
                let examined = readiness.examined;
                let mut point_after = None;
                loop {
                    let page = operations
                        .store()
                        .study_candidates(&study.key, point_after)
                        .await?;
                    if page.is_empty() {
                        break;
                    }
                    for candidate in page {
                        point_after = Some(candidate.ordinal);
                        if cancel.token().is_cancelled() {
                            return Ok((Processed::Idle, None));
                        }
                        let _turn = match group {
                            Some(group) => match group.acquire(&candidate.key) {
                                Some(turn) => Some(turn),
                                None => continue,
                            },
                            None => None,
                        };
                        let action_lane = &mut *lane;
                        let action_publication = &mut *publication;
                        if let Some(result) = self
                            .worker_action(move || {
                                self.observe_study_candidate(
                                    operations,
                                    candidate,
                                    action_lane,
                                    cancel,
                                    action_publication,
                                )
                            })
                            .await?
                        {
                            return Ok(result);
                        }
                    }
                }
                let final_key = format!("finalize:{}", study.key);
                let _turn = match group {
                    Some(group) => match group.acquire(&final_key) {
                        Some(turn) => Some(turn),
                        None => continue,
                    },
                    None => None,
                };
                if self
                    .worker_action(|| self.finalize_canonical_study(&study.key))
                    .await?
                {
                    return Ok((Processed::Finalized { study: study.key }, None));
                }
                if maintenance {
                    return Ok((
                        Processed::Maintenance {
                            study: study.key,
                            examined,
                        },
                        None,
                    ));
                }
            }
        }
    }
    /// Observe actual candidate work, including preparation before a claim exists.
    /// Empty discovery polls create no operation and cannot evict actual attempts.
    async fn observe_study_candidate(
        &self,
        operations: &Operations,
        candidate: pse_operations::canonical_studies::ScopedStudyPoint,
        lane: &mut super::study::StudyCandidateLane,
        cancel: &crate::CancelSource,
        publication: &mut Option<PublicationHandoff>,
    ) -> Result<Option<(Processed, Option<Arc<super::RunResult>>)>, WorkflowError> {
        use pse_diagnostics::observation::{Outcome, Relationship, current, scope};
        let operation = current()
            .map(|context| context.child("managed.worker.candidate", Relationship::Nested));
        let context = operation.as_ref().map(|operation| operation.context());
        let result = scope(
            context,
            self.worker_action(|| {
                self.work_study_candidate(operations, candidate, lane, cancel, publication)
            }),
        )
        .await;
        if let Some(operation) = operation {
            operation.finish(if result.is_ok() {
                Outcome::Completed
            } else {
                Outcome::Failed
            });
        }
        result
    }

    /// One candidate's existing policy/claim/execution path. None resumes discovery;
    /// Some returns the same processed action to the worker scheduler.
    async fn work_study_candidate(
        &self,
        operations: &Operations,
        candidate: pse_operations::canonical_studies::ScopedStudyPoint,
        lane: &mut super::study::StudyCandidateLane,
        cancel: &crate::CancelSource,
        publication: &mut Option<PublicationHandoff>,
    ) -> Result<Option<(Processed, Option<Arc<super::RunResult>>)>, WorkflowError> {
        use pse_model::study::{ActionKind, SeedAvailability, SeedFact, StartPolicy, WaitReason};
        if let Some(context) = pse_diagnostics::observation::current() {
            context.source_ref("study.occurrence", candidate.key.as_bytes());
        }
        let scope = operations.store().study_scope(&candidate.key).await?;
        let policy = scope.point().policy()?;
        let unresolved = match &policy.start {
            StartPolicy::Fresh => None,
            StartPolicy::Explicit { role, .. } => Some(SeedFact {
                role: *role,
                availability: SeedAvailability::Unresolved,
            }),
            StartPolicy::Continuation(edge) => Some(SeedFact {
                role: edge.role,
                availability: SeedAvailability::Unresolved,
            }),
        };
        let preliminary = scope.action(unresolved)?;
        match &preliminary.kind {
            ActionKind::Wait(WaitReason::ScientificAdmission) => {
                if self.admit_study_candidate(&scope, lane, cancel).await? {
                    return Ok(Some((
                        Processed::Maintenance {
                            study: scope.point().study.clone(),
                            examined: 1,
                        },
                        None,
                    )));
                }
                return Ok(None);
            }
            ActionKind::Wait(WaitReason::SeedResolution { .. }) | ActionKind::Start(_) => {}
            ActionKind::Refuse(_) | ActionKind::Cancel => {
                if self
                    .settle_current_study_candidate(&scope, &preliminary)
                    .await?
                    .is_none()
                {
                    return Ok(None);
                }
                return Ok(Some((
                    Processed::Settled {
                        point: candidate.key,
                    },
                    None,
                )));
            }
            ActionKind::Wait(_) | ActionKind::Reconcile => return Ok(None),
        }
        let mut prepared = self
            .prepare_study_candidate_in_lane(&scope, lane, cancel)
            .await?;
        let action = scope.action(prepared.seed.clone())?;
        if !matches!(action.kind, ActionKind::Start(_)) {
            match action.kind {
                ActionKind::Refuse(_) | ActionKind::Cancel => {
                    if self
                        .settle_current_study_candidate(&scope, &action)
                        .await?
                        .is_none()
                    {
                        return Ok(None);
                    }
                    return Ok(Some((
                        Processed::Settled {
                            point: candidate.key,
                        },
                        None,
                    )));
                }
                _ => return Ok(None),
            }
        }
        let claim_operation = format!(
            "study-claim:{}:{}:{}",
            candidate.key,
            scope.point().revision,
            operations.worker()
        );
        fn phase_bytes<F, Fut>(_: &F) -> usize
        where
            F: FnOnce(pse_operations::canonical_studies::StudyClaim) -> Fut,
            Fut: Future,
        {
            size_of::<Fut>()
        }
        let launch_extent = prepared
            .operation
            .start_attempt_extent(self, cancel)
            .ok_or_else(|| contract("worker launch storage extent"))?;
        let diagnostics = prepared.diagnostics.take();
        let seed = prepared.seed.clone();
        let execute = move |claim: pse_operations::canonical_studies::StudyClaim| async move {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.source_ref("canonical.run", claim.fence.run().as_bytes());
                context.source_ref("canonical.attempt", claim.fence.attempt().as_bytes());
                context.source_ref("study.occurrence", claim.point.key.as_bytes());
                context.count("study.claimed", "occurrences", 1);
            }
            let run_key = claim
                .fence
                .run()
                .strip_prefix("run:")
                .ok_or_else(|| contract("scientific run lineage key absent"))?;
            let run = RunId::from_id(
                SemanticId::parse_hex(run_key).map_err(|e| contract(e.to_string()))?,
            );
            let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
            hash.str("scientific.attempt.lineage.v1")
                .str(claim.fence.attempt());
            let attempt_id = AttemptId::from_id(hash.finish_id());
            let mut attempt =
                DurableAttempt::claimed(operations, claim.fence.clone(), run, attempt_id)?;
            attempt.set_worker_publication(publication.take());
            attempt.set_output(prepared.output.clone());
            let stop = cancel.clone();
            if let Err(error) = attempt.start(Arc::new(move || stop.cancel())).await {
                let record = attempt.abandon(&Arc::new(error)).await;
                return Ok(Some((
                    Processed::Ran {
                        point: claim.point.key,
                        record: Arc::new(record),
                    },
                    None,
                )));
            }
            let point = match operations.store().mark_study_started(&claim).await {
                Ok(point) => point,
                Err(error) => {
                    return Ok(Some((
                        Processed::Ran {
                            point: claim.point.key,
                            record: Arc::new(attempt.abandon(&Arc::new(error.into())).await),
                        },
                        None,
                    )));
                }
            };
            if let Err(error) = self.apply_study_prediction(&mut prepared, cancel).await {
                let record = attempt.abandon(&Arc::new(error)).await;
                self.recover_study_point(&point.key).await?;
                return Ok(Some((
                    Processed::Ran {
                        point: point.key,
                        record: Arc::new(record),
                    },
                    None,
                )));
            }
            if let Err(error) = operations
                .store()
                .renew_attempt(&claim.fence, operations.policy().lease)
                .await
            {
                let error = Arc::new(WorkflowError::Canonical(error));
                let cancelled = attempt.cancelled_refusal(&error);
                let record = attempt.abandon(&error).await;
                // Drain the owner on every refusal. Only an acknowledged
                // exact cancellation can settle without native entry.
                if !cancelled {
                    return Err(WorkflowError::Shared(error));
                }
                record
                    .attempt
                    .as_ref()
                    .map_err(|error| WorkflowError::Shared(error.clone()))?;
                self.recover_study_point(&point.key).await?;
                return Ok(Some((
                    Processed::Ran {
                        point: point.key,
                        record: Arc::new(record),
                    },
                    None,
                )));
            }
            let held = lane
                .scope()
                .ok_or_else(|| contract("worker native lane source permission absent"))?;
            let result = held.during(cancel, async {
            let mut launch: futures_util::future::BoxFuture<'_, _> = boxed_worker_action(|| prepared.operation.start_attempt(self, cancel, attempt));
            let handle = futures_util::future::poll_fn(|context| poll_worker_action(launch.as_mut(), context)).await?;
            tokio::select! {result=handle.wait()=>result,()=cancel.cancelled()=>{handle.cancel();handle.wait().await}}
        }).await?;
            let record = match result.durability() {
                RunDurability::Durable(record) => record.clone(),
                RunDurability::Ephemeral => {
                    return Err(contract("claimed native study attempt was not retained"));
                }
            };
            self.record_study_attempt(&point, &claim.start, &record)
                .await?;
            Ok(Some((
                Processed::Ran {
                    point: point.key,
                    record,
                },
                Some(result),
            )))
        };
        let extent = phase_bytes(&execute)
            .checked_add(launch_extent)
            .and_then(|bytes| bytes.checked_add(128))
            .ok_or_else(|| contract("worker claimed phase storage extent"))?;
        // Both claimed phase and selected launch branch are prepaid before the claim.
        let _launch_owner = self.shared.math().reserve("study:worker-launch", extent)?;
        let claim = {
            let _claim = pse_diagnostics::observation::phase("managed.worker.claim_acquire");
            operations
                .store()
                .claim_study_point(
                    &scope,
                    seed,
                    &claim_operation,
                    operations.worker(),
                    operations.policy().lease,
                )
                .await
        };
        let claim = match claim {
            Ok(claim) => claim,
            Err(error) => {
                let current = operations.store().study_scope(&candidate.key).await?;
                if current.point().revision != scope.point().revision
                    || current.cancelled() != scope.cancelled()
                    || current
                        .predecessors()
                        .iter()
                        .map(|p| (&p.key, p.revision))
                        .ne(scope.predecessors().iter().map(|p| (&p.key, p.revision)))
                {
                    return Ok(None);
                }
                return Err(error.into());
            }
        };
        let detail = diagnostics.as_ref().and_then(|selection| {
            super::study_diagnostics::begin_child(
                selection,
                &claim.point.study,
                claim.fence.run(),
                claim.fence.attempt(),
            )
        });
        let detail_context = detail.as_ref().map(crate::capture::DetailSession::context);
        let original_context = pse_diagnostics::observation::current();
        let mut execution: futures_util::future::BoxFuture<'_, _> =
            boxed_worker_action(|| execute(claim));
        let outcome = pse_diagnostics::observation::scope(
            detail_context.or_else(|| original_context.clone()),
            futures_util::future::poll_fn(|context| {
                poll_worker_action(execution.as_mut(), context)
            }),
        )
        .await;
        // Prediction, native execution, publication and their owned drains finish first.
        // Optional collection failures never replace the scientific outcome.
        super::study_diagnostics::finish_child(detail, original_context);
        outcome
    }

    /// A separate rotating cursor gives later active studies bounded recovery
    /// service even while scientific discovery repeatedly returns an earlier point.
    async fn recover_next_study(
        &self,
        position: &tokio::sync::Mutex<Option<String>>,
        group: Option<&CandidateGroup>,
        cancel: &crate::CancelSource,
    ) -> Result<Option<Processed>, WorkflowError> {
        let store = operations(self)?.store();
        let mut position = position.lock().await;
        let mut page = store.study_page(position.as_deref()).await?;
        if page.is_empty() && position.is_some() {
            page = store.study_page(None).await?;
        }
        let Some(study) = page.into_iter().next() else {
            *position = None;
            return Ok(None);
        };
        *position = Some(study.key.clone());
        if cancel.token().is_cancelled() {
            return Ok(None);
        }
        let repair = store.repair_study_readiness(&study.key).await?;
        // Active native width is bounded; one assigned page makes progress without
        // a hidden graph drain. Completed recoveries leave this indexed page.
        let assigned = store.study_assigned_page(&study.key, None).await?;
        for point in assigned {
            if cancel.token().is_cancelled() {
                break;
            }
            // A local lane retains this turn through native drain and terminal
            // observation. Cancellation closes new entry, but maintenance must
            // not revoke that same owner's finish fence while it is draining.
            let _turn = match group {
                Some(group) => match group.acquire(&point.key) {
                    Some(turn) => Some(turn),
                    None => continue,
                },
                None => None,
            };
            if self.recover_study_point(&point.key).await? {
                return Ok(Some(Processed::Settled { point: point.key }));
            }
        }
        Ok(repair.pending.then_some(Processed::Maintenance {
            study: study.key,
            examined: repair.examined,
        }))
    }
    /// Isolate a concrete phase's layout while charging its actual heap owner.
    async fn worker_action<T, F, Fut>(&self, action: F) -> Result<T, WorkflowError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, WorkflowError>> + Send,
    {
        fn action_bytes<F, Fut>(_: &F) -> usize
        where
            F: FnOnce() -> Fut,
            Fut: Future,
        {
            size_of::<Fut>()
        }
        let _action_owner = self
            .shared
            .math()
            .reserve("study:worker-action", action_bytes(&action))?;
        let mut work: futures_util::future::BoxFuture<'_, _> = boxed_worker_action(action);
        futures_util::future::poll_fn(|context| poll_worker_action(work.as_mut(), context)).await
    }
    async fn group_lane(
        &self,
        recovery: bool,
        recovery_position: &tokio::sync::Mutex<Option<String>>,
        group: &CandidateGroup,
        lane: &mut super::study::StudyCandidateLane,
        stop: &crate::CancelSource,
        mut publication: Option<PublicationHandoff>,
    ) -> Result<Processed, WorkflowError> {
        let cancel = crate::CancelSource::new();
        let operation_cancel = &cancel;
        let action = move || async move {
            let lane = lane;
            self.work_once_scoped(
                recovery,
                recovery_position,
                Some(group),
                lane,
                operation_cancel,
                &mut publication,
            )
            .await
        };
        let work = self.worker_action(action);
        tokio::pin!(work);
        let result = tokio::select! {
            result = &mut work => result,
            () = stop.cancelled() => {
                cancel.cancel();
                // Issued canonical effects and native owners must finish their own drain.
                work.await
            }
        };
        Ok(result?.0)
    }
    /// Serve one bounded group on the shared runtime, draining every issued lane on stop
    /// or failure. Completed native owners are released as each lane finishes.
    pub async fn serve(
        &self,
        settings: WorkerSettings,
        stop: &crate::CancelSource,
    ) -> Result<usize, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                if settings.poll.is_zero() || settings.recovery.is_zero() {
                    return Err(contract(
                        "worker polling and recovery intervals must be positive",
                    ));
                }
                let allocation = self
                    .shared
                    .math()
                    .cores()
                    .min(self.shared.budget().math.jobs)
                    .max(1);
                let width = settings.maximum_in_flight.unwrap_or(allocation);
                if width == 0 || width > allocation {
                    return Err(contract("worker case lanes exceed the runtime allocation"));
                }
                let group = CandidateGroup::default();
                let group = &group;
                let recovery_position = tokio::sync::Mutex::new(None);
                let recovery_position = &recovery_position;
                let work = move |mut lane: super::study::StudyCandidateLane,
                                 recovery,
                                 drain,
                                 publication| async move {
                    let result = self
                        .group_lane(
                            recovery,
                            recovery_position,
                            group,
                            &mut lane,
                            &drain,
                            Some(publication),
                        )
                        .await;
                    (lane, result)
                };
                // Infer the exact nested future extent without constructing one
                // on the main stack. Charge it before active future allocation,
                // as well as the idle owner vector. The pinned FuturesUnordered
                // task/Arc metadata fits in 128 bytes per boxed handle; its shared
                // queue/stub fits in 256.
                fn future_bytes<S, F, Fut>(_: &F) -> usize
                where
                    F: Fn(S, bool, crate::CancelSource, PublicationHandoff) -> Fut,
                    Fut: Future,
                {
                    size_of::<Fut>()
                }
                let future_bytes = future_bytes(&work);
                let lanes = width
                    .checked_mul(2)
                    .ok_or_else(|| contract("worker publication lane extent"))?;
                let lane_bytes = lanes
                    .checked_mul(size_of::<super::study::StudyCandidateLane>() + future_bytes + 128)
                    .and_then(|bytes| bytes.checked_add(256))
                    .and_then(|bytes| {
                        publication::coordination_bytes(lanes)
                            .ok()
                            .and_then(|coord| bytes.checked_add(coord))
                    })
                    .ok_or_else(|| contract("worker lane and publication coordination extent"))?;
                let _owner = self
                    .shared
                    .math()
                    .reserve("study:worker-input-lanes", lane_bytes)?;
                let owners = (0..lanes)
                    .map(|_| super::study::StudyCandidateLane::new())
                    .collect();
                let (result, mut owners) = serve_owned_lanes(
                    settings,
                    owners,
                    width,
                    stop,
                    move |lane, recovery, drain, publication| {
                        // A lane's nested preparation/execution future is large.
                        // The scheduler transfers and polls this heap handle,
                        // retaining the exact owner until its awaited drain ends.
                        let future: futures_util::future::BoxFuture<'_, _> =
                            boxed_worker_action(|| work(lane, recovery, drain, publication));
                        future
                    },
                )
                .await;
                let mut close_failure = None;
                for lane in &mut owners {
                    if let Err(error) = lane.close().await
                        && close_failure.is_none()
                    {
                        close_failure = Some(error);
                    }
                }
                if let Some(error) = close_failure {
                    return Err(error);
                }
                result
            })
            .await
    }
    /// Shared document physical admission stays with its scientific owner.
    pub(crate) async fn physical_from_sources(
        &self,
        sources: &BTreeMap<String, Vec<u8>>,
        cancel: &crate::CancelSource,
    ) -> Result<super::PhysicalContext, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let validation = self.admission_context();
                let pool = self.shared.pool();
                let token = cancel.token();
                let bundle = load_package_documents_owned(
                    sources,
                    self.registry(),
                    pse_authoring::ParseBudget::default(),
                    &pool,
                    &token,
                    &validation,
                )?;
                let documents = OwnedDocumentSet::try_from_bundles(vec![bundle], &pool, &token)?;
                self.physical_from_documents(&documents, &token).await
            })
            .await
    }
    pub(super) async fn package_from_sources(
        &self,
        bundles: &[BTreeMap<String, Vec<u8>>],
        physical: super::PhysicalContext,
        cancel: &crate::CancelSource,
    ) -> Result<super::ModelingPackage, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let token = cancel.token();
                token.checkpoint().map_err(pse_engine::EngineError::from)?;
                // Source bytes and load order define the complete immutable admission closure.
                // The cache owns only admitted values; every attempt has its own mutable workspace.
                let service = self.shared.math();
                let generation = service.modeling_cache.generation();
                let mut framed =
                    pse_ids::FramedHasher::new(pse_ids::Frame::ModelingPackageAdmissionV1);
                framed.u64(bundles.len() as u64).hash(&physical.key)
            // Local cache authority is the exact immutable validation assembly and registry.
            // Its owner is retained with the admission, preventing pointer reuse. These
            // process-local slots never enter source/scientific/operational identities.
            .u64(Arc::as_ptr(self.sessions()) as usize as u64)
            .u64(Arc::as_ptr(self.registry()) as usize as u64);
                for sources in bundles {
                    token.checkpoint().map_err(pse_engine::EngineError::from)?;
                    framed.hash(&package_checksum(sources));
                }
                match &physical.package {
                    Some(package) => {
                        use pse_model::SemanticFrame;
                        framed.u64(1);
                        package.header.frame(&mut framed);
                    }
                    None => {
                        framed.u64(0);
                    }
                }
                let identity = pse_ids::roles::AdmittedClosureHash::from_id(framed.finish_hash());
                if let Some(admitted) = service.modeling_cache.package(identity) {
                    return self.package_from_admission(admitted, physical);
                }
                let validation = self.admission_context();
                let pool = self.shared.pool();
                let bundles = bundles
                    .iter()
                    .map(|sources| {
                        load_package_documents_owned(
                            sources,
                            self.registry(),
                            pse_authoring::ParseBudget::default(),
                            &pool,
                            &token,
                            &validation,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &token)?;
                token.checkpoint().map_err(pse_engine::EngineError::from)?;
                // This publishes the canonical source revision; await its owner completely.
                let package = self
                    .modeling_from_documents(&documents, physical, cancel)
                    .await?;
                token.checkpoint().map_err(pse_engine::EngineError::from)?;
                service
                    .modeling_cache
                    .retain_package(generation, identity, package.admission()?);
                Ok(package)
            })
            .await
    }
}
