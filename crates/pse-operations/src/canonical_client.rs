// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One bounded canonical protocol client. Supplier crates own protobuf, values
//! and typed errors; this module owns request policy and physical completion.
use std::{
    borrow::Cow,
    future::IntoFuture,
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use surrealdb_protocol::proto::rpc::v1::surreal_db_service_client::SurrealDbServiceClient;
use surrealdb_protocol::proto::{rpc::v1 as rpc, v1 as proto};
use surrealdb_types::{Error, Object, SurrealValue, Value, prost::Message};
use tokio::sync::{Notify, Semaphore};
#[path = "canonical_client/allocation.rs"]
mod allocation;
#[path = "canonical_client/custody.rs"]
mod custody;
#[path = "canonical_client/errors.rs"]
mod errors;
#[path = "canonical_client/outbound.rs"]
mod outbound;
#[path = "canonical_client/owned.rs"]
mod owned;
#[path = "canonical_client/request.rs"]
mod request;
#[path = "canonical_client/results.rs"]
mod results;
#[path = "canonical_client/wire.rs"]
mod wire;
// The generator's prettyplease output owns formatting for this wire catalogue.
#[rustfmt::skip]
#[path = "generated/grpc_wire_schema.rs"]
mod wire_schema;
pub use allocation::ResponseAllocation;
pub use custody::RetainedRequest;
use owned::{Driver, OwnedHttp2, REQUEST};
pub use owned::{REQUEST_SPAN_NAME, REQUEST_SPAN_TARGET, is_request_span};
pub use request::{DispatchState, RequestContext, ResponseLimits};
pub use results::{IndexedResults, QueryResult, QueryStatisticsObserver, Stats};
type EngineResult<T> = Result<T, Error>;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = EngineResult<T>> + Send + 'a>>;

/// Root credentials, never rendered by Debug.
#[derive(Clone)]
pub struct Root {
    /// Authenticated account name.
    pub username: String,
    /// Secret supplied directly to the native signin RPC.
    pub password: String,
}
/// Database access credentials, never rendered by Debug.
#[derive(Clone)]
pub struct Database {
    /// Existing namespace.
    pub namespace: String,
    /// Existing database.
    pub database: String,
    /// Authenticated account name.
    pub username: String,
    /// Secret supplied directly to the native signin RPC.
    pub password: String,
}
impl std::fmt::Debug for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RootCredentials(redacted)")
    }
}
impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DatabaseCredentials(redacted)")
    }
}
/// The two credential classes used by canonical and development-store callers.
#[derive(Clone, Debug)]
pub enum Credentials {
    /// Root administrative account.
    Root(Root),
    /// Database-scoped account.
    Database(Database),
}
impl From<Root> for Credentials {
    fn from(value: Root) -> Self {
        Self::Root(value)
    }
}
impl From<Database> for Credentials {
    fn from(value: Database) -> Self {
        Self::Database(value)
    }
}
#[derive(Clone, Debug)]
enum Setup {
    Signin(Credentials),
    Use {
        namespace: Option<String>,
        database: Option<String>,
    },
}

struct Generation {
    client: SurrealDbServiceClient<OwnedHttp2>,
    driver: Arc<Driver>,
    session: uuid::Uuid,
    request_bytes: usize,
    users: AtomicUsize,
    idle: Notify,
}
impl Generation {
    async fn connect(
        address: SocketAddr,
        message_bytes: usize,
        context: &RequestContext,
    ) -> EngineResult<Arc<Self>> {
        let driver = Driver::new();
        let opening = async {
            let url = url::Url::parse(&format!("grpc://{address}")).map_err(connection)?;
            let transport = OwnedHttp2::connect(&url, message_bytes, driver.clone()).await?;
            let mut client = SurrealDbServiceClient::new(transport)
                .max_decoding_message_size(message_bytes)
                .max_encoding_message_size(message_bytes);
            let capabilities = client
                .get_capabilities(rpc::GetCapabilitiesRequest {
                    context: None,
                    client: Some(rpc::ClientInfo {
                        name: "pse-arrow".into(),
                        version: env!("CARGO_PKG_VERSION").into(),
                        platform: "rust".into(),
                        metadata: Vec::new(),
                    }),
                })
                .await
                .map_err(errors::status_to_error)?
                .into_inner()
                .capabilities
                .ok_or_else(|| connection("Server omitted capabilities"))?;
            let advertised = capabilities
                .limits
                .and_then(|limits| usize::try_from(limits.max_message_bytes).ok())
                .filter(|bytes| *bytes > 0);
            let request_bytes = advertised.map_or(message_bytes, |bytes| bytes.min(message_bytes));
            client = client.max_encoding_message_size(request_bytes);
            let session = client
                .attach_session(rpc::AttachSessionRequest { context: None })
                .await
                .map_err(errors::status_to_error)?
                .into_inner()
                .session
                .ok_or_else(|| connection("Server omitted attached session"))?
                .to_uuid()
                .map_err(connection)?;
            Ok(Arc::new(Self {
                client,
                driver: driver.clone(),
                session,
                request_bytes,
                users: AtomicUsize::new(0),
                idle: Notify::new(),
            }))
        };
        let result = tokio::select! {biased;_=context.interrupted()=>Err(context.interruption_error()),result=opening=>result};
        let result = match result {
            Ok(generation) => {
                tokio::select! {biased;_=context.interrupted()=>Err(context.interruption_error()),_=context.outbound_drained()=>Ok(generation)}
            }
            Err(error) => Err(error),
        };
        if result.is_err() {
            driver.shutdown().await;
        } else {
            driver.finish_setup();
        }
        result
    }
    fn context(&self) -> EngineResult<rpc::RequestContext> {
        let deadline = REQUEST
            .try_with(RequestContext::deadline)
            .map_err(|_| Error::configuration("Request has no original clock".into(), None))?;
        let timeout = proto::Duration::try_from(deadline.saturating_duration_since(Instant::now()))
            .map_err(connection)?;
        Ok(rpc::RequestContext {
            session: Some(proto::Uuid::from_uuid(self.session)),
            transaction: None,
            timeout: Some(timeout),
        })
    }
    fn checked<M: Message>(&self, request: M) -> EngineResult<M> {
        if request.encoded_len() > self.request_bytes {
            return Err(Error::configuration(
                "Native request exceeds the pinned message envelope".into(),
                None,
            ));
        }
        Ok(request)
    }
    async fn apply(&self, setup: &Setup) -> EngineResult<()> {
        match setup {
            Setup::Signin(credentials) => {
                let (namespace, database, username, password) = match credentials {
                    Credentials::Root(root) => (
                        String::new(),
                        String::new(),
                        root.username.clone(),
                        root.password.clone(),
                    ),
                    Credentials::Database(account) => (
                        account.namespace.clone(),
                        account.database.clone(),
                        account.username.clone(),
                        account.password.clone(),
                    ),
                };
                let response = self
                    .client
                    .clone()
                    .signin(self.checked(rpc::SigninRequest {
                        context: Some(self.context()?),
                        access_method: Some(rpc::AccessMethod {
                            method: Some(rpc::access_method::Method::User(rpc::UserCredentials {
                                namespace,
                                database,
                                username,
                                password,
                                access: String::new(),
                            })),
                        }),
                    })?)
                    .await
                    .map_err(errors::status_to_error)?
                    .into_inner();
                if response.tokens.is_none() {
                    return Err(connection("Server omitted signin tokens"));
                }
                Ok(())
            }
            Setup::Use {
                namespace,
                database,
            } => {
                let nullable = |value: String| rpc::NullableString {
                    value: Some(rpc::nullable_string::Value::Some(value)),
                };
                self.client
                    .clone()
                    .r#use(self.checked(rpc::UseRequest {
                        context: Some(self.context()?),
                        namespace: namespace.clone().map(nullable),
                        database: database.clone().map(nullable),
                    })?)
                    .await
                    .map_err(errors::status_to_error)?;
                Ok(())
            }
        }
    }
    async fn detach(&self) -> EngineResult<()> {
        self.client
            .clone()
            .detach_session(self.checked(rpc::DetachSessionRequest {
                context: Some(self.context()?),
            })?)
            .await
            .map_err(errors::status_to_error)?;
        Ok(())
    }
    async fn drained(&self) {
        loop {
            let wake = self.idle.notified();
            tokio::pin!(wake);
            wake.as_mut().enable();
            if self.users.load(Ordering::Acquire) == 0 {
                return;
            }
            wake.await;
        }
    }
}
struct GenerationUse(Arc<Generation>);
impl Drop for GenerationUse {
    fn drop(&mut self) {
        self.0.users.fetch_sub(1, Ordering::AcqRel);
        self.0.idle.notify_waiters();
    }
}
struct State {
    address: SocketAddr,
    message_bytes: usize,
    generation: tokio::sync::Mutex<Arc<Generation>>,
    acknowledged: Mutex<Vec<Setup>>,
    setup: Arc<tokio::sync::Mutex<()>>,
    admission: Mutex<bool>,
    close_drained: AtomicBool,
    close_wake: Notify,
    active: AtomicUsize,
    idle: Notify,
    application: Arc<Semaphore>,
    control: Arc<Semaphore>,
    queued: Arc<Semaphore>,
    queued_control: Arc<Semaphore>,
}
impl State {
    async fn ready(&self, context: &RequestContext) -> EngineResult<GenerationUse> {
        let mut current = tokio::select! {biased;_=context.interrupted()=>return Err(context.interruption_error()),current=self.generation.lock()=>current};
        if *self
            .admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            return Err(connection("Canonical connection is closing"));
        }
        if !current.driver.accepting() {
            if context.was_dispatched() {
                return Err(context.interruption_error());
            }
            current.driver.shutdown().await;
            // Join issued work before taking its final acknowledged setup log.
            current.drained().await;
            let log = self
                .acknowledged
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let setup = RequestContext::control(
                context
                    .deadline()
                    .min(Instant::now() + Duration::from_secs(5)),
            )
            .inherit_request_ownership(context);
            let fresh = REQUEST
                .scope(
                    setup.clone(),
                    Generation::connect(self.address, self.message_bytes, &setup),
                )
                .await?;
            *current = fresh.clone();
            let restore_deadline = setup.deadline();
            let restoring = REQUEST.scope(setup.clone(), async {
                for entry in &log {
                    fresh.apply(entry).await?;
                    setup.outbound_drained().await;
                }
                Ok(())
            });
            let restored = tokio::select! {biased;_=context.interrupted()=>Err(context.interruption_error()),_=tokio::time::sleep_until(restore_deadline.into())=>Err(connection("Canonical restoration deadline expired")),result=restoring=>result};
            if let Err(error) = restored {
                fresh.driver.shutdown().await;
                return Err(error);
            }
            if *self
                .admission
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
            {
                fresh.driver.shutdown().await;
                return Err(connection("Canonical connection is closing"));
            }
        }
        current.users.fetch_add(1, Ordering::AcqRel);
        Ok(GenerationUse(current.clone()))
    }
}
struct Active(Arc<State>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::AcqRel);
        self.0.idle.notify_waiters();
    }
}
struct Drain(RequestContext);
impl Drop for Drain {
    fn drop(&mut self) {
        self.0.release_lifetime_after_drain();
        self.0.release_payload_after_drain();
    }
}
struct Cancel(RequestContext);
impl Drop for Cancel {
    fn drop(&mut self) {
        if self.0.dispatch_state() != DispatchState::Completed {
            self.0.cancel();
        }
    }
}

/// One physical connection, with one bounded acknowledged session.
#[derive(Clone)]
pub struct Client {
    state: Arc<State>,
}
impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanonicalProtocolClient")
            .field("endpoint", &self.state.address)
            .finish_non_exhaustive()
    }
}
impl Client {
    /// Connect an authenticated-loopback transport under the enclosing clock.
    pub async fn connect(address: SocketAddr, message_bytes: usize) -> EngineResult<Self> {
        if !address.ip().is_loopback() || message_bytes == 0 {
            return Err(Error::configuration(
                "Bounded canonical transport requires loopback and a positive envelope".into(),
                None,
            ));
        }
        let deadline = super::original_deadline(Duration::from_secs(5)).into_std();
        let context = RequestContext::control(deadline);
        let _cancel = Cancel(context.clone());
        let task = context.clone();
        let (send, receive) = tokio::sync::oneshot::channel();
        tokio::spawn(owned::observe("connect", async move {
            let result = REQUEST
                .scope(
                    task.clone(),
                    Generation::connect(address, message_bytes, &task),
                )
                .await;
            let result = result.map(|generation| Self {
                state: Arc::new(State {
                    address,
                    message_bytes,
                    generation: tokio::sync::Mutex::new(generation),
                    acknowledged: Mutex::new(Vec::new()),
                    setup: Arc::new(tokio::sync::Mutex::new(())),
                    admission: Mutex::new(false),
                    close_drained: AtomicBool::new(false),
                    close_wake: Notify::new(),
                    active: AtomicUsize::new(0),
                    idle: Notify::new(),
                    application: Arc::new(Semaphore::new(32)),
                    control: Arc::new(Semaphore::new(2)),
                    queued: Arc::new(Semaphore::new(32)),
                    queued_control: Arc::new(Semaphore::new(32)),
                }),
            });
            if result.is_ok() {
                task.mark_completed();
            }
            if let Err(Ok(client)) = send.send(result) {
                let _ = client.disconnect().await;
            }
        }));
        tokio::select! {biased;_=context.interrupted()=>Err(context.interruption_error()),result=receive=>result.map_err(|_|connection("Canonical connection owner failed"))?}
    }

    /// Build one complete script. No effect occurs until the builder is polled.
    pub fn query<'a>(&self, query: impl Into<Cow<'a, str>>) -> Query<'a> {
        Query {
            client: self.clone(),
            query: query.into(),
            variables: Ok(Object::new()),
            guarded_bindings: std::collections::BTreeSet::new(),
            context: None,
            observer: None,
            statement_window: None,
            #[cfg(test)]
            transaction: None,
            cleanup: request::UndispatchedPayloadDrain::caller(None),
        }
    }
    /// Authenticate using one of the actually consumed credential classes.
    pub fn signin(&self, credentials: impl Into<Credentials>) -> SetupRequest {
        SetupRequest {
            client: self.clone(),
            setup: Setup::Signin(credentials.into()),
            context: None,
        }
    }
    /// Select a namespace, optionally followed by database selection.
    pub fn use_ns(&self, namespace: &str) -> UseRequest {
        UseRequest {
            client: self.clone(),
            namespace: namespace.into(),
            database: None,
            context: None,
        }
    }
    /// Select one existing record through the same bounded query owner.
    pub fn select<R: SurrealValue>(&self, resource: (&str, &str)) -> Select<R> {
        let record = surrealdb_types::RecordId::new(resource.0.to_owned(), resource.1.to_owned());
        Select {
            query: self
                .query("SELECT * FROM ONLY $pse_record")
                .bind(("pse_record", record)),
            marker: std::marker::PhantomData,
        }
    }
    async fn execute<T, F, Q>(&self, context: RequestContext, operation: F) -> EngineResult<T>
    where
        T: Send + 'static,
        F: FnOnce(Arc<Generation>) -> Q + Send + 'static,
        Q: Future<Output = EngineResult<T>> + Send + 'static,
    {
        let _undispatched = request::UndispatchedPayloadDrain::caller(Some(context.clone()));
        let _cancel = Cancel(context.clone());
        let _receiver_payload = context.payload_owner().map(|owner| owner.retain());
        let state = self.state.clone();
        let (waiting, immediate, active) = {
            let closed = state
                .admission
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if *closed {
                return Err(connection("Canonical connection is closing"));
            }
            let semaphore = if context.is_control() {
                state.control.clone()
            } else {
                state.application.clone()
            };
            let immediate = semaphore.try_acquire_owned().ok();
            let waiting = if immediate.is_none() {
                Some(
                    (if context.is_control() {
                        state.queued_control.clone()
                    } else {
                        state.queued.clone()
                    })
                    .try_acquire_owned()
                    .map_err(|_| connection("Bounded canonical admission queue full"))?,
                )
            } else {
                None
            };
            context.mark_enqueued();
            state.active.fetch_add(1, Ordering::AcqRel);
            (waiting, immediate, Active(state.clone()))
        };
        let task = context.clone();
        let (send, receive) = tokio::sync::oneshot::channel();
        tokio::spawn(owned::observe("bounded", async move {
            let _active = active;
            let _drain = Drain(task.clone());
            let _payload = task.payload_owner().map(|owner| owner.retain_driver());
            let _lifetime = task.lifetime_owner().map(|owner| owner.retain());
            let semaphore = if task.is_control() {
                state.control.clone()
            } else {
                state.application.clone()
            };
            let permit = match immediate {
                Some(permit) => Ok(permit),
                None => {
                    tokio::select! {biased;_=task.interrupted()=>{let _=send.send(Err(task.interruption_error()));return;},permit=semaphore.acquire_owned()=>permit}
                }
            };
            let Ok(_permit) = permit else {
                return;
            };
            drop(waiting);
            let generation = match state.ready(&task).await {
                Ok(generation) => generation,
                Err(error) => {
                    let _ = send.send(Err(error));
                    return;
                }
            };
            let driver = generation.0.driver.clone();
            let outcome = {
                let future = REQUEST.scope(task.clone(), operation(generation.0.clone()));
                tokio::pin!(future);
                tokio::select! {biased;
                    _=task.interrupted()=>{
                        if task.was_dispatched()&&Instant::now()<task.deadline(){tokio::select!{biased;_=tokio::time::sleep_until(task.deadline().into())=>None,_=driver.stopping()=>None,result=&mut future=>Some(result)}}else{None}
                    },
                    _=driver.stopping()=>None,
                    result=&mut future=>Some(result),
                }
            };
            let outcome = match outcome {
                Some(result) => {
                    if result
                        .as_ref()
                        .is_err_and(|error| error.is_connection() || error.is_serialization())
                    {
                        driver.shutdown().await;
                    } else if task.was_dispatched() {
                        task.mark_completed();
                    }
                    result
                }
                None => {
                    if task.was_dispatched() {
                        driver.shutdown().await;
                    }
                    Err(task.interruption_error())
                }
            };
            // Publish the known response once. Physical outbound work may still
            // own bytes under peer backpressure: its original deadline and task
            // owner retain admission, generation, actual grants and read guards.
            let _ = send.send(outcome);
            if !task.outbound_is_drained() {
                tokio::select! {biased;
                    _=tokio::time::sleep_until(task.deadline().into())=>driver.shutdown().await,
                    _=driver.stopping()=>driver.shutdown().await,
                    _=task.outbound_drained()=>{},
                }
            }
            drop(generation);
        }));
        tokio::select! {biased;_=context.interrupted()=>Err(context.interruption_error()),result=receive=>result.map_err(|_|connection("Canonical request owner failed"))?}
    }
    /// Seal new requests and join one retained close round. Abandoning a waiter
    /// cannot restart its original detach clock or strand physical driver shutdown.
    pub async fn disconnect(&self) -> EngineResult<()> {
        let start = {
            let mut closing = self
                .state
                .admission
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if *closing {
                false
            } else {
                *closing = true;
                true
            }
        };
        if start {
            let state = self.state.clone();
            let deadline = super::original_deadline(Duration::from_secs(5)).into_std();
            tokio::spawn(owned::observe("disconnect", async move {
                let generation = state.generation.lock().await.clone();
                if generation.driver.begin_close() && !generation.driver.closed() {
                    // A failed or expired detach never prevents actual socket drain.
                    let _ = tokio::time::timeout_at(
                        deadline.into(),
                        REQUEST.scope(RequestContext::control(deadline), generation.detach()),
                    )
                    .await;
                }
                generation.driver.shutdown().await;
                loop {
                    let wake = state.idle.notified();
                    tokio::pin!(wake);
                    wake.as_mut().enable();
                    if state.active.load(Ordering::Acquire) == 0 {
                        break;
                    }
                    wake.await;
                }
                state.close_drained.store(true, Ordering::Release);
                state.close_wake.notify_waiters();
            }));
        }
        loop {
            let wake = self.state.close_wake.notified();
            tokio::pin!(wake);
            wake.as_mut().enable();
            if self.state.close_drained.load(Ordering::Acquire) {
                return Ok(());
            }
            wake.await;
        }
    }
}
fn connection(error: impl std::fmt::Display) -> Error {
    Error::connection(error.to_string(), None)
}

/// One acknowledged setup RPC, serialized through its retained performing owner.
#[derive(Debug)]
pub struct SetupRequest {
    client: Client,
    setup: Setup,
    context: Option<RequestContext>,
}
impl SetupRequest {
    /// Supply the original control clock and explicit request custody.
    pub fn request_context(mut self, context: RequestContext) -> Self {
        self.context = Some(context);
        self
    }
}
/// A namespace selection that may add a database to the same native USE request.
#[derive(Debug)]
pub struct UseRequest {
    client: Client,
    namespace: String,
    database: Option<String>,
    context: Option<RequestContext>,
}
impl UseRequest {
    /// Add database selection without permitting it on an authentication request.
    pub fn use_db(mut self, database: &str) -> Self {
        self.database = Some(database.into());
        self
    }
    /// Supply the original control clock and explicit request custody.
    pub fn request_context(mut self, context: RequestContext) -> Self {
        self.context = Some(context);
        self
    }
}
impl IntoFuture for UseRequest {
    type Output = EngineResult<()>;
    type IntoFuture = BoxFuture<'static, ()>;
    fn into_future(self) -> Self::IntoFuture {
        SetupRequest {
            client: self.client,
            setup: Setup::Use {
                namespace: Some(self.namespace),
                database: self.database,
            },
            context: self.context,
        }
        .into_future()
    }
}
impl IntoFuture for SetupRequest {
    type Output = EngineResult<()>;
    type IntoFuture = BoxFuture<'static, ()>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let context = self.context.ok_or_else(|| {
                Error::configuration("Setup requires an original request context".into(), None)
            })?;
            let state = self.client.state.clone();
            self.client
                .execute(context, move |generation| async move {
                    let _serial = state.setup.lock().await;
                    if state
                        .acknowledged
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .len()
                        >= 63
                    {
                        return Err(Error::configuration(
                            "Bounded acknowledged setup limit reached".into(),
                            None,
                        ));
                    }
                    generation.apply(&self.setup).await?;
                    state
                        .acknowledged
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(self.setup);
                    Ok(())
                })
                .await
        })
    }
}

/// A single bounded query builder; it carries bytes before its cleanup guard.
pub struct Query<'a> {
    client: Client,
    query: Cow<'a, str>,
    variables: EngineResult<Object>,
    guarded_bindings: std::collections::BTreeSet<String>,
    context: Option<RequestContext>,
    observer: Option<Arc<dyn QueryStatisticsObserver>>,
    statement_window: Option<(usize, usize)>,
    #[cfg(test)]
    transaction: Option<(uuid::Uuid, uuid::Uuid)>,
    cleanup: request::UndispatchedPayloadDrain,
}
impl std::fmt::Debug for Query<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanonicalQuery")
            .field("client", &self.client)
            .field("has_context", &self.context.is_some())
            .finish_non_exhaustive()
    }
}
impl<'a> Query<'a> {
    /// Bind typed supplier values without interpreting application policy.
    pub fn bind(mut self, variables: impl SurrealValue) -> Self {
        let guarded = &self.guarded_bindings;
        let insert = |current: &mut Object, key: String, value: Value| -> EngineResult<()> {
            if guarded.contains(&key) {
                return Err(Error::configuration(
                    format!("Typed script authority binding {key} cannot be overwritten"),
                    None,
                ));
            }
            current.insert(key, value);
            Ok(())
        };
        self.variables = self.variables.and_then(|mut current| {
            match variables.into_value() {
                Value::Object(values) => {
                    for (key, value) in values {
                        insert(&mut current, key, value)?;
                    }
                }
                Value::Array(values) => {
                    let mut values = values.into_iter();
                    while let Some(key) = values.next() {
                        let value = values.next().ok_or_else(|| {
                            Error::validation(
                                "Variable bindings require key/value pairs".into(),
                                None,
                            )
                        })?;
                        let key = key.into_string().map_err(|_| {
                            Error::validation("Variable key must be a string".into(), None)
                        })?;
                        insert(&mut current, key, value)?;
                    }
                }
                _ => {
                    return Err(Error::validation(
                        "Variable bindings must be an object or key/value pairs".into(),
                        None,
                    ));
                }
            }
            Ok(current)
        });
        self
    }
    /// Attach bindings supplied by the typed envelope once, before ordinary business binds.
    pub(crate) fn bind_guarded(mut self, bindings: Object) -> Self {
        let guarded = bindings.keys().cloned().collect::<Vec<_>>();
        self = self.bind(bindings);
        self.guarded_bindings.extend(guarded);
        self
    }
    #[cfg(test)]
    pub(crate) fn payload_leaf_count(&self) -> usize {
        self.context
            .as_ref()
            .map_or(0, RequestContext::payload_leaf_count)
    }
    /// Replace the clock while retaining existing explicit allocation/lifetime custody.
    pub fn request_context(self, context: RequestContext) -> Self {
        let context = match &self.context {
            Some(old) => context.inherit_request_ownership(old),
            None => context,
        };
        self.replace_composed_context(context)
    }
    /// Apply the endpoint's preflight to the already issued lifecycle and owners.
    pub(crate) fn response_limits(mut self, limits: ResponseLimits) -> Self {
        let Some(context) = self.context.clone() else {
            self.variables = Err(Error::configuration(
                "Metadata limits require an original request context".into(),
                None,
            ));
            return self;
        };
        self.replace_composed_context(context.with_response_limits(limits))
    }
    #[cfg(test)]
    pub(crate) fn context_snapshot(&self) -> Option<RequestContext> {
        self.context.clone()
    }
    #[cfg(test)]
    pub(crate) fn shares_lifecycle(&self, original: &RequestContext) -> bool {
        self.context
            .as_ref()
            .is_some_and(|context| context.same_lifecycle(original))
    }
    // Internal owners were composed from this exact context already. Merging
    // them back into their source would multiply the same old drain leaves.
    fn replace_composed_context(mut self, context: RequestContext) -> Self {
        self.cleanup.update_context(Some(context.clone()));
        self.context = Some(context);
        self
    }
    /// Retain the actual prepaid response reservation through native delivery.
    pub fn payload_owner(self, owner: impl Into<ResponseAllocation>) -> Self {
        let owner = owner.into();
        let context = self.context.clone().unwrap_or_else(|| {
            RequestContext::new(super::original_deadline(super::REQUEST_TIMEOUT).into_std())
        });
        self.replace_composed_context(owner.apply(context))
    }
    /// Retain an existing scientific staging lease through actual request drain.
    /// It grants no decoded-response allowance or execution authority.
    pub(crate) fn scientific_payload_owner(
        self,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Self {
        let context = self.context.clone().unwrap_or_else(|| {
            RequestContext::new(super::original_deadline(super::REQUEST_TIMEOUT).into_std())
        });
        self.replace_composed_context(context.with_scientific_payload_owner(owner))
    }
    /// Retain each independent staging lease without an erased aggregate owner.
    pub(crate) fn scientific_payload_owners(
        mut self,
        owners: Vec<Arc<pse_columnar::AllocationLease>>,
    ) -> Self {
        for owner in owners {
            self = self.scientific_payload_owner(owner);
        }
        self
    }
    /// Retain an issued immutable selection through its guarded request.
    pub(crate) fn retain_selection(self, selection: crate::canonical::ProtectedSelection) -> Self {
        match self.context.clone() {
            Some(context) => {
                self.replace_composed_context(context.with_selection_custody(selection))
            }
            None => self.replace_composed_context(
                RequestContext::new(super::original_deadline(super::REQUEST_TIMEOUT).into_std())
                    .with_selection_custody(selection),
            ),
        }
    }
    /// Install one caller observation sink; it supplies no execution authority.
    pub fn observe_statistics(mut self, observer: Arc<dyn QueryStatisticsObserver>) -> Self {
        self.observer = Some(observer);
        self
    }
    /// An owner-added transaction envelope preserves the caller's original statement slots.
    /// Complete response errors, including failed COMMIT, are checked before projection.
    pub(crate) fn statement_window(mut self, prefix: usize, suffix: usize) -> Self {
        self.statement_window = Some((prefix, suffix));
        self
    }
    pub(crate) fn refuse(mut self, message: &str) -> Self {
        self.variables = Err(Error::configuration(message.to_owned(), None));
        self
    }
    /// Own query text so the builder can cross the caller's borrow boundary.
    pub fn into_owned(self) -> Query<'static> {
        Query {
            client: self.client,
            query: Cow::Owned(self.query.into_owned()),
            variables: self.variables,
            guarded_bindings: self.guarded_bindings,
            context: self.context,
            observer: self.observer,
            statement_window: self.statement_window,
            #[cfg(test)]
            transaction: self.transaction,
            cleanup: self.cleanup,
        }
    }
}
impl<'a> IntoFuture for Query<'a> {
    type Output = EngineResult<IndexedResults>;
    type IntoFuture = BoxFuture<'a, IndexedResults>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut builder = self;
            // Declare the clearable custody guard before taking byte-owning fields,
            // so lexical cancellation drops the fields before publishing their drain.
            let _cleanup = std::mem::replace(
                &mut builder.cleanup,
                request::UndispatchedPayloadDrain::caller(None),
            );
            let Self {
                client,
                query,
                variables,
                guarded_bindings: _,
                context,
                observer,
                statement_window,
                #[cfg(test)]
                transaction,
                cleanup: _,
            } = builder;
            let context = context.ok_or_else(|| {
                Error::configuration("Query requires an original request context".into(), None)
            })?;
            let limit = context.response_limits();
            let query = query
                .trim_end_matches(|c: char| c == ';' || c.is_whitespace())
                .to_owned();
            let statement_limit = query
                .len()
                .min(1 << 20)
                .min(client.state.message_bytes / results::statement_slot_bytes())
                .min(limit.map_or(usize::MAX, |limit| {
                    limit
                        .nodes
                        .min(limit.container_items)
                        .min(limit.wire_bytes / results::statement_slot_bytes())
                }));
            let result = client
                .execute(context, move |generation| async move {
                    let variables: proto::Variables = variables?
                        .into_iter()
                        .map(|(key, value)| {
                            proto::Value::try_from(value)
                                .map(|value| (key, value))
                                .map_err(connection)
                        })
                        .collect::<EngineResult<Vec<_>>>()?
                        .into_iter()
                        .collect();
                    let native_context = generation.context()?;
                    #[cfg(test)]
                    let native_context = match transaction {
                        Some((transaction, session)) => {
                            if session != generation.session {
                                return Err(connection(
                                    "Held test transaction lost its original connection",
                                ));
                            }
                            rpc::RequestContext {
                                transaction: Some(proto::Uuid::from_uuid(transaction)),
                                ..native_context
                            }
                        }
                        None => native_context,
                    };
                    let request = generation.checked(rpc::QueryRequest {
                        context: Some(native_context),
                        query,
                        variables: Some(variables),
                        accepted_encodings: Vec::new(),
                        max_batch_records: 0,
                    })?;
                    let stream = generation
                        .client
                        .clone()
                        .query(request)
                        .await
                        .map_err(errors::status_to_error)?
                        .into_inner();
                    results::collect(stream, statement_limit).await
                })
                .await;
            if let Some(observer) = observer {
                match &result {
                    Ok(response) => response.observe(observer.as_ref()),
                    Err(_) => observer.request_failed(),
                }
            }
            match statement_window {
                Some((prefix, suffix)) => result.and_then(|result| result.statement_window(prefix, suffix)),
                None => result,
            }
        })
    }
}

/// Typed single-record selection under explicit original request options.
pub struct Select<R> {
    query: Query<'static>,
    marker: std::marker::PhantomData<fn() -> R>,
}
impl<R> std::fmt::Debug for Select<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CanonicalSelect").field(&self.query).finish()
    }
}
impl<R> Select<R> {
    pub(crate) fn from_query(query: Query<'static>) -> Self {
        Self { query, marker: std::marker::PhantomData }
    }
    /// Supply original deadline and request custody.
    pub fn request_context(mut self, context: RequestContext) -> Self {
        self.query = self.query.request_context(context);
        self
    }
}
impl<R: SurrealValue + 'static> IntoFuture for Select<R>
where
    usize: QueryResult<R>,
{
    type Output = EngineResult<R>;
    type IntoFuture = BoxFuture<'static, R>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut result = self.query.await?;
            result.take(0)
        })
    }
}

#[cfg(test)]
#[path = "canonical_client/held_transaction.rs"]
mod held_transaction;
#[cfg(all(test, feature = "canonical-tests"))]
pub(crate) use held_transaction::HeldTransaction;
