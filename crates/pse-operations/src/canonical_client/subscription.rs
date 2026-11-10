// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed-principal subscriptions own a separate attached native generation.
use super::{
    Client, Credentials, EngineResult, REQUEST, RequestContext, ResponseLimits, connection, errors,
    owned, proto, rpc,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use surrealdb_types::Object;
use tokio::sync::{Notify, watch};

/// A notification invalidates an as-of read; it is not an event history or write receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubscriptionNotice {
    /// Registration is attached and notifications are being drained.
    Active {
        /// Monotone local invalidation revision, not a durable feed cursor.
        revision: u64,
    },
    /// Continuity or authority is unavailable; an identified resnapshot is required.
    Unavailable,
    /// The original owner explicitly closed this subscription.
    Closed,
}
struct Closing {
    requested: AtomicBool,
    wake: Notify,
}
impl Closing {
    fn request(&self) {
        self.requested.store(true, Ordering::Release);
        self.wake.notify_waiters();
    }
    async fn requested(&self) {
        loop {
            let wake = self.wake.notified();
            tokio::pin!(wake);
            wake.as_mut().enable();
            if self.requested.load(Ordering::Acquire) {
                return;
            }
            wake.await;
        }
    }
}
/// Caller-held observation ownership. Drop requests cleanup; close awaits its actual result.
pub struct Subscription {
    client: Client,
    state: watch::Receiver<SubscriptionNotice>,
    done: watch::Receiver<Option<EngineResult<()>>>,
    closing: Arc<Closing>,
}
impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSubscription")
            .field("state", &*self.state.borrow())
            .finish_non_exhaustive()
    }
}
impl Subscription {
    pub(crate) fn client(&self) -> &Client {
        &self.client
    }
    /// Latest bounded invalidation state. Silence does not certify currentness or authority.
    pub fn notice(&self) -> SubscriptionNotice {
        let notice = self.state.borrow().clone();
        if matches!(notice, SubscriptionNotice::Active { .. }) && self.state.has_changed().is_err()
        {
            SubscriptionNotice::Unavailable
        } else {
            notice
        }
    }
    /// Mark the current local dirty revision seen immediately before an authoritative read.
    pub fn acknowledge(&mut self) -> SubscriptionNotice {
        self.state.borrow_and_update().clone()
    }
    /// Read an identified snapshot on this exact fixed principal/session.
    /// SQL comes from the caller's checked query owner; the explicit readonly
    /// transaction and complete error projection cannot grant write authority.
    pub async fn read_snapshot(
        &self,
        body: String,
        expression: String,
        variables: Object,
        context: RequestContext,
    ) -> EngineResult<super::IndexedResults> {
        self.client
            .query(format!(
                "BEGIN READ ONLY ISOLATION LEVEL SNAPSHOT; {body}\nRETURN {expression}; COMMIT;"
            ))
            .bind(variables)
            .request_context(context)
            .operation_return()
            .await
    }
    /// Wait for a coalesced invalidation or loss of continuity.
    pub async fn changed(&mut self) -> SubscriptionNotice {
        if self.state.changed().await.is_err() {
            return SubscriptionNotice::Unavailable;
        }
        self.notice()
    }
    /// Kill the original registration and await session/physical-driver settlement.
    /// # Errors
    /// Native kill/detach failed; physical disconnect is still attempted and joined.
    pub async fn close(&mut self) -> EngineResult<()> {
        self.closing.request();
        loop {
            if let Some(result) = self.done.borrow().clone() {
                return result;
            }
            self.done
                .changed()
                .await
                .map_err(|_| connection("Subscription owner ended without settlement"))?;
        }
    }
}
impl Drop for Subscription {
    fn drop(&mut self) {
        self.closing.request();
    }
}

struct Opening {
    context: RequestContext,
    accepted: bool,
}
impl Drop for Opening {
    fn drop(&mut self) {
        if !self.accepted {
            self.context.cancel();
        }
    }
}

impl Client {
    /// Open a fixed Viewer generation; callers retain no mutable session handle.
    /// Register before reading the identified snapshot and continuously drain thereafter.
    /// # Errors
    /// Setup/registration/Begin validation fails under the original observation deadline.
    pub async fn subscribe_fixed_viewer(
        &self,
        credentials: Credentials,
        namespace: String,
        database: String,
        query: String,
        variables: Object,
        context: RequestContext,
    ) -> EngineResult<Subscription> {
        let mut opening = Opening {
            context: context.clone(),
            accepted: false,
        };
        let address = self.state.address;
        let bytes = self.state.message_bytes;
        let (state_send, state) = watch::channel(SubscriptionNotice::Unavailable);
        let (done_send, done) = watch::channel(None);
        let closing = Arc::new(Closing {
            requested: AtomicBool::new(false),
            wake: Notify::new(),
        });
        let owner_close = closing.clone();
        let (ready_send, ready) = tokio::sync::oneshot::channel();
        let limits = ResponseLimits {
            wire_bytes: 64 << 10,
            nodes: 1024,
            depth: 20,
            container_items: 128,
            string_bytes: 32 << 10,
            string_length: 16 << 10,
            blob_bytes: 0,
            blob_length: 0,
        };
        let task = context.clone().with_response_limits(limits);
        tokio::spawn(owned::observe("subscription", async move {
            let lifetime = task.lifetime_owner().map(|owner| owner.retain());
            let payload = task.payload_owner().map(|owner| owner.retain_driver());
            let ready_send = Arc::new(std::sync::Mutex::new(Some(ready_send)));
            let begin_send = ready_send.clone();
            let result=async {
                let connect=RequestContext::control(task.deadline().min(Instant::now()+Duration::from_secs(5))).inherit_request_ownership(&task);
                let client=tokio::select! {biased;_=task.interrupted()=>return Err(task.interruption_error()),result=Client::connect_context(address,bytes,false,connect)=>result?};
                let result=async {
                    client.signin(credentials.clone()).request_context(RequestContext::control(task.deadline())).await?;
                    client.use_ns(&namespace).use_db(&database).request_context(RequestContext::control(task.deadline())).await?;
                    let operation_context=task.clone();
                    let snapshot_client=client.clone();
                    client.execute(task.clone(),move|generation| async move {
                        let variables:proto::Variables=variables.into_iter().map(|(key,value)|proto::Value::try_from(value).map(|value|(key,value)).map_err(connection)).collect::<EngineResult<Vec<_>>>()?.into_iter().collect();
                        let request=generation.checked(rpc::SubscribeRequest{context:Some(generation.context()?),resume_from:None,subscribe_to:Some(rpc::subscribe_request::SubscribeTo::Query(rpc::LiveQueryRegistration{query,variables:Some(variables)}))})?;
                        let mut stream=generation.client.clone().subscribe(request).await.map_err(errors::status_to_error)?.into_inner();
                        let begin=match stream.message().await.map_err(errors::status_to_error)?.and_then(|value|value.frame) {
                            Some(rpc::subscribe_response::Frame::Begin(begin))=>begin,
                            _=>return Err(connection("Subscription omitted its Begin frame")),
                        };
                        let live=begin.live_query_id.ok_or_else(||connection("Subscription omitted live query identity"))?;
                        let _subscription=begin.subscription_id.ok_or_else(||connection("Subscription omitted attached identity"))?.to_uuid().map_err(connection)?;
                        live.to_uuid().map_err(connection)?;
                        state_send.send_replace(SubscriptionNotice::Active{revision:0});
                        if begin_send.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take().is_some_and(|send|send.send(Ok(snapshot_client)).is_err()) {owner_close.request();}
                        let mut revision=0u64;
                        let outcome=loop {
                            let message=tokio::select!{biased;
                                _=owner_close.requested()=>break Ok(()),
                                _=operation_context.interrupted()=>break Err(operation_context.interruption_error()),
                                message=stream.message()=>message.map_err(errors::status_to_error),
                            };
                            match message {
                                Ok(Some(rpc::SubscribeResponse{frame:Some(rpc::subscribe_response::Frame::Notification(notification))}))=>{
                                    if notification.live_query_id.as_ref()!=Some(&live) {break Err(connection("Notification belongs to another registration"));}
                                    revision=revision.checked_add(1).ok_or_else(||connection("Subscription revision overflow"))?;
                                    state_send.send_replace(SubscriptionNotice::Active{revision});
                                },
                                Ok(Some(rpc::SubscribeResponse{frame:Some(rpc::subscribe_response::Frame::Error(error))}))=>break Err(errors::proto_error(error)),
                                Ok(_)=>break Err(connection("Subscription continuity ended")),
                                Err(error)=>break Err(error),
                            }
                        };
                        state_send.send_replace(if owner_close.requested.load(Ordering::Acquire){SubscriptionNotice::Closed}else{SubscriptionNotice::Unavailable});
                        // A new bounded control context retains the same original
                        // deadline; cancellation never renews observation authority.
                        let close_deadline=operation_context.deadline().min(Instant::now()+Duration::from_secs(5));
                        let control=RequestContext::control(close_deadline).inherit_request_ownership(&operation_context);
                        let killed=REQUEST.scope(control,async {
                            tokio::time::timeout_at(close_deadline.into(),generation.client.clone().kill(generation.checked(rpc::KillRequest{context:Some(generation.context()?),live_query_id:Some(live)})?)).await.map_err(|_|connection("Subscription kill deadline expired"))?.map_err(errors::status_to_error)?;
                            Ok(())
                        }).await;
                        drop(stream);
                        outcome.and(killed)
                    }).await
                }.await;
                // Always join the dedicated physical generation, including setup,
                // Begin, Kill and permission failures. No ordinary sibling is cut.
                let disconnected=client.disconnect_at(task.deadline()).await;
                result.and(disconnected)
            }.await;
            if let Some(send) = ready_send
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
            {
                let _ = send
                    .send(Err(result.clone().err().unwrap_or_else(|| {
                        connection("Subscription closed before attachment")
                    })));
            }
            drop(payload);
            drop(lifetime);
            task.release_payload_after_drain();
            task.release_lifetime_after_drain();
            done_send.send_replace(Some(result));
        }));
        let client = tokio::select! {biased;_=context.interrupted()=>return Err(context.interruption_error()),ready=ready=>ready.map_err(|_|connection("Subscription owner failed before attachment"))??};
        opening.accepted = true;
        Ok(Subscription {
            client,
            state,
            done,
            closing,
        })
    }
}
