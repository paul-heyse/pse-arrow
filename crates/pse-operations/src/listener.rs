// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One `LISTEN` connection per [`Store`](crate::Store) (Plan 22 X8; ADR-0114 Outcome 26).
//!
//! A task owns a dedicated connection (`application_name = pse-operations-listener`),
//! drives it with `Connection::poll_message`, issues `LISTEN` on the cancellation and
//! progress channels, and broadcasts every notification. A notification only shortens
//! latency: whatever was sent while the connection was down is gone. So after every
//! successful `LISTEN` the task increments its generation and broadcasts
//! [`Event::Resync`], and a watcher re-reads its authority on a matching notification,
//! on `Resync` and when it lagged behind the broadcast. A lost connection is re-established
//! with capped exponential backoff. The task ends with the store: dropping the listener
//! aborts it.

use std::time::Duration;

use futures::StreamExt as _;
use pse_ids::SemanticId;
use tokio::sync::{broadcast, watch};
use tokio::task::JoinHandle;
use tokio_postgres::AsyncMessage;
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::attempts::AttemptId;
use crate::error::{DriverError, OperationsError, Target};

/// The `application_name` of the listener's connection.
pub const LISTENER_APPLICATION: &str = "pse-operations-listener";

/// The channels the listener serves.
const LISTEN: &str = "LISTEN pse_ops_cancel; LISTEN pse_ops_progress";

/// Reconnection backoff bounds.
const BACKOFF_MIN: Duration = Duration::from_millis(50);
const BACKOFF_MAX: Duration = Duration::from_secs(5);

/// Broadcast capacity; a watcher further behind resynchronizes.
const CAPACITY: usize = 1024;

/// A channel the store notifies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Channel {
    /// [`crate::cancellation::CANCEL_CHANNEL`].
    Cancel,
    /// [`crate::streams::PROGRESS_CHANNEL`].
    Progress,
}

impl Channel {
    fn of(name: &str) -> Option<Self> {
        match name {
            crate::cancellation::CANCEL_CHANNEL => Some(Self::Cancel),
            crate::streams::PROGRESS_CHANNEL => Some(Self::Progress),
            _ => None,
        }
    }
}

/// What the listener broadcasts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    /// A notification naming an attempt.
    Notification {
        /// Its channel.
        channel: Channel,
        /// The attempt its payload names.
        attempt: AttemptId,
    },
    /// `LISTEN` is (again) in effect; notifications sent before may be lost, so every
    /// watcher re-reads its authority.
    Resync,
}

/// The listener task of one store.
#[derive(Debug)]
pub(crate) struct Listener {
    /// Weak: the task holds the only strong sender, so the channel closes when it ends.
    events: broadcast::WeakSender<Event>,
    generation: watch::Receiver<u64>,
    task: JoinHandle<()>,
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Every event after the listener's `LISTEN` was in effect.
#[derive(Debug)]
pub(crate) struct Subscription {
    events: broadcast::Receiver<Event>,
    target: Target,
}

impl Subscription {
    /// The next event. A watcher that fell behind the broadcast is told to resynchronize.
    ///
    /// # Errors
    /// [`OperationsError::Unavailable`] once the listener stopped with its store.
    pub(crate) async fn next(&mut self) -> Result<Event, OperationsError> {
        match self.events.recv().await {
            Ok(event) => Ok(event),
            Err(broadcast::error::RecvError::Lagged(_)) => Ok(Event::Resync),
            Err(broadcast::error::RecvError::Closed) => Err(stopped(&self.target)),
        }
    }
}

fn stopped(target: &Target) -> OperationsError {
    OperationsError::Unavailable {
        target: target.clone(),
        source: DriverError::new(std::io::Error::other("the store's listener has stopped")),
    }
}

impl Listener {
    /// Start the listener task on its own connection.
    pub(crate) fn spawn(config: &tokio_postgres::Config, tls: &MakeRustlsConnect) -> Self {
        let mut config = config.clone();
        config.application_name(LISTENER_APPLICATION);
        let (events, _) = broadcast::channel(CAPACITY);
        let (generation, receiver) = watch::channel(0);
        let weak = events.downgrade();
        let task = tokio::spawn(run(config, tls.clone(), events, generation));
        Self {
            events: weak,
            generation: receiver,
            task,
        }
    }

    /// Subscribe, then wait until `LISTEN` is in effect: a watcher reads its authority
    /// only after this returns (listen, then inspect).
    ///
    /// # Errors
    /// [`OperationsError::Unavailable`] when `LISTEN` is not in effect within `timeout`
    /// or the listener stopped.
    pub(crate) async fn subscribe(
        &self,
        target: &Target,
        timeout: Duration,
    ) -> Result<Subscription, OperationsError> {
        let events = self
            .events
            .upgrade()
            .ok_or_else(|| stopped(target))?
            .subscribe();
        let mut generation = self.generation.clone();
        match tokio::time::timeout(timeout, generation.wait_for(|listened| *listened > 0)).await {
            Ok(Ok(_)) => Ok(Subscription {
                events,
                target: target.clone(),
            }),
            Ok(Err(_)) => Err(stopped(target)),
            Err(_) => Err(OperationsError::Unavailable {
                target: target.clone(),
                source: DriverError::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("LISTEN was not in effect within {timeout:?}"),
                )),
            }),
        }
    }

    /// How many times `LISTEN` took effect: 1 after the first connection, one more after
    /// every reconnection.
    #[cfg(test)]
    pub(crate) fn generation(&self) -> u64 {
        *self.generation.borrow()
    }

    /// Stop the task; subscribers see the listener as unavailable.
    pub(crate) fn stop(&self) {
        self.task.abort();
    }
}

/// Forward one asynchronous message; anything but a well-formed store notification is
/// ignored.
fn forward(events: &broadcast::Sender<Event>, message: &AsyncMessage) {
    if let AsyncMessage::Notification(notification) = message
        && let Some(channel) = Channel::of(notification.channel())
        && let Ok(id) = SemanticId::parse_hex(notification.payload())
    {
        // No subscriber is not an error: nobody is watching right now.
        let _ = events.send(Event::Notification {
            channel,
            attempt: AttemptId::from_id(id),
        });
    }
}

async fn run(
    config: tokio_postgres::Config,
    tls: MakeRustlsConnect,
    events: broadcast::Sender<Event>,
    generation: watch::Sender<u64>,
) {
    let mut backoff = BACKOFF_MIN;
    loop {
        if let Ok((client, mut connection)) = config.connect(tls.clone()).await {
            // The connection makes progress only while polled: its messages are the
            // notifications, and polling them also completes `LISTEN`.
            let mut messages =
                futures::stream::poll_fn(move |cx| connection.poll_message(cx)).fuse();
            let listen = client.batch_execute(LISTEN);
            tokio::pin!(listen);
            let listened = loop {
                tokio::select! {
                    result = &mut listen => break result.is_ok(),
                    message = messages.next() => match message {
                        Some(Ok(message)) => forward(&events, &message),
                        Some(Err(_)) | None => break false,
                    },
                }
            };
            if listened {
                backoff = BACKOFF_MIN;
                generation.send_modify(|listened| *listened += 1);
                let _ = events.send(Event::Resync);
                while let Some(Ok(message)) = messages.next().await {
                    forward(&events, &message);
                }
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = backoff.saturating_mul(2).min(BACKOFF_MAX);
    }
}

#[cfg(test)]
mod listener_unit {
    use super::*;

    #[test]
    fn store_channels_are_recognized() {
        assert_eq!(
            Channel::of(crate::cancellation::CANCEL_CHANNEL),
            Some(Channel::Cancel)
        );
        assert_eq!(
            Channel::of(crate::streams::PROGRESS_CHANNEL),
            Some(Channel::Progress)
        );
        assert_eq!(Channel::of(crate::jobs::JOBS_CHANNEL), None);
        assert!(LISTEN.contains(crate::cancellation::CANCEL_CHANNEL));
        assert!(LISTEN.contains(crate::streams::PROGRESS_CHANNEL));
    }
}
