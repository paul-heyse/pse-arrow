// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Continuously polled, single-population stream of opaque product-issued requests.
use super::{ProductReaderError, ProductReaderSelection, ProductReply, invalid};
use crate::environment::Environment;
use futures_util::{StreamExt, stream};
use pse_diagnostics::Cancellation;
use serde::Serialize;
use serde_json::value::RawValue;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Observer stop cause; none establishes completion or cancellation of observed work.
#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stop {
    /// The observer's requested sample allowance ended.
    SampleLimit,
    /// The observer's original wall-clock allowance ended.
    Deadline,
    /// Explicit observer cancellation, after the current helper settled.
    Cancelled,
    /// The product supplied no next request, including unavailable/refused evidence.
    ProductStopped,
}
/// Terminal observer facts, separate from the original product payload's scientific outcome.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Summary {
    /// Successfully emitted whole product responses.
    pub samples: usize,
    /// Original reason observation ended.
    pub stop: Stop,
}
/// Caller-selected polling allowances, independent of native workload resource policy.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Positive maximum emitted snapshots.
    pub samples: usize,
    /// Positive delay between reads; no initial delay.
    pub interval: Duration,
    /// Original absolute observer deadline.
    pub deadline: Instant,
}
struct State {
    selection: Arc<ProductReaderSelection>,
    environment: Arc<Environment>,
    correlation: String,
    next: Option<Box<RawValue>>,
    limits: Limits,
    control: Cancellation,
    count: usize,
}
enum Event {
    Sample(ProductReply),
    End(Summary),
}
/// Run one outstanding passive helper at a time, emit complete responses, then settle stop.
/// A failing renderer ends observation; no observed-work control capability exists here.
pub async fn run<Emission: Future<Output = Result<(), ProductReaderError>>>(
    selection: Arc<ProductReaderSelection>,
    environment: Arc<Environment>,
    correlation: String,
    request: Box<RawValue>,
    limits: Limits,
    control: Cancellation,
    mut emit: impl FnMut(ProductReply) -> Emission,
) -> Result<Summary, ProductReaderError> {
    if limits.samples == 0 || limits.interval.is_zero() {
        return Err(invalid(
            "follow requires positive sample and polling allowances",
        ));
    }
    let observer_control = control.clone();
    let state = State {
        selection,
        environment,
        correlation,
        next: Some(request),
        limits,
        control,
        count: 0,
    };
    let events = stream::unfold(Some(state), |state| async move {
        let mut state = state?;
        let stop = if state.control.is_cancelled() {
            Some(Stop::Cancelled)
        } else if state.count >= state.limits.samples {
            Some(Stop::SampleLimit)
        } else if state.next.is_none() {
            Some(Stop::ProductStopped)
        } else if Instant::now() >= state.limits.deadline {
            Some(Stop::Deadline)
        } else {
            None
        };
        if let Some(stop) = stop {
            return Some((
                Ok(Event::End(Summary {
                    samples: state.count,
                    stop,
                })),
                None,
            ));
        }
        if state.count > 0 {
            let next = Instant::now()
                .checked_add(state.limits.interval)
                .unwrap_or(state.limits.deadline)
                .min(state.limits.deadline);
            tokio::select! {
                _ = state.control.cancelled() => {},
                _ = tokio::time::sleep_until(tokio::time::Instant::from_std(next)) => {},
            }
            if state.control.is_cancelled() || Instant::now() >= state.limits.deadline {
                let stop = if state.control.is_cancelled() {
                    Stop::Cancelled
                } else {
                    Stop::Deadline
                };
                return Some((
                    Ok(Event::End(Summary {
                        samples: state.count,
                        stop,
                    })),
                    None,
                ));
            }
        }
        let request = state.next.take()?;
        let selected = state.selection.clone();
        let environment = state.environment.clone();
        let correlation = state.correlation.clone();
        let control = state.control.clone();
        let mut task = tokio::task::spawn_blocking(move || {
            selected.read_cancellable(&request, &correlation, &environment, &control)
        });
        let result = tokio::select! {
            result = &mut task => result,
            _ = state.control.cancelled() => { state.control.cancel(); task.await },
            _ = tokio::time::sleep_until(tokio::time::Instant::from_std(state.limits.deadline)) => { state.control.cancel(); task.await },
        };
        let result = match result {
            Ok(result) => result,
            Err(error) => Err(invalid(format!(
                "follow observer task failed after join: {error}"
            ))),
        };
        match result {
            Ok(reply) => {
                state.next = reply.next_request().map(ToOwned::to_owned);
                state.count += 1;
                Some((Ok(Event::Sample(reply)), Some(state)))
            }
            Err(ProductReaderError::Execution {
                failures,
                stdout,
                stderr,
            }) if state.control.is_cancelled()
                && stdout.is_empty()
                && stderr.is_empty()
                && failures.len() == 1
                && matches!(
                    &failures[0],
                    ProductReaderError::Supervision(pse_diagnostics::SupervisionError::Cancelled)
                ) =>
            {
                // Cancellation-owned helper failure is joined; it is not a scientific failure.
                let stop = if Instant::now() >= state.limits.deadline {
                    Stop::Deadline
                } else {
                    Stop::Cancelled
                };
                Some((
                    Ok(Event::End(Summary {
                        samples: state.count,
                        stop,
                    })),
                    None,
                ))
            }
            Err(error) => Some((Err(error), None)),
        }
    });
    futures_util::pin_mut!(events);
    let mut emitted = 0;
    while let Some(event) = events.next().await {
        match event? {
            Event::Sample(reply) => {
                tokio::select! {
                    result = emit(reply) => result?,
                    _ = observer_control.cancelled() => return Ok(Summary { samples: emitted, stop: Stop::Cancelled }),
                    _ = tokio::time::sleep_until(tokio::time::Instant::from_std(limits.deadline)) => {
                        observer_control.cancel();
                        return Ok(Summary { samples: emitted, stop: Stop::Deadline });
                    },
                }
                emitted += 1;
            }
            Event::End(mut summary) => {
                summary.samples = emitted;
                return Ok(summary);
            }
        }
    }
    Err(invalid("follow stream ended without observer settlement"))
}
