// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authenticated gRPC/HTTP2 hosting and operation-local conversations.
use super::{
    Error, Request, Response, Result,
    service::ServiceEngine,
    wire,
    workers::{CancelOnDrop, Workers},
};
use crate::{
    code_intelligence_contracts::{pb::pse::code_intelligence::v1 as pb, services as bindings},
    envelope::CliInvocationId,
    environment::Environment,
};
use buffa::MessageView as _;
use connectrpc::{ConnectError, ServiceStream};
use futures_util::StreamExt as _;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::Instrument as _;

/// Identity comes from the kernel at accept, never from RPC metadata.
#[derive(Clone, Debug)]
pub(crate) struct PeerContext {
    pub(crate) uid: u32,
    pub(crate) gid: u32,
    pub(crate) pid: Option<i32>,
}

#[derive(Clone)]
pub(crate) struct AuthenticatedConnector {
    pub(crate) socket: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) invocation: CliInvocationId,
    pub(crate) environment: Environment,
}
impl tower_service::Service<http::Uri> for AuthenticatedConnector {
    type Response = hyper_util::rt::TokioIo<tokio::net::UnixStream>;
    type Error = std::io::Error;
    type Future = std::pin::Pin<Box<dyn Future<Output = std::io::Result<Self::Response>> + Send>>;
    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: http::Uri) -> Self::Future {
        let connector = self.clone();
        Box::pin(async move {
            let stream = tokio::net::UnixStream::connect(&connector.socket).await?;
            let peer = stream.peer_cred()?;
            let pid = peer
                .pid()
                .and_then(|pid| u32::try_from(pid).ok())
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "kernel helper PID unavailable",
                    )
                })?;
            // This connector is retained by ConnectRPC's Reconnect owner: every
            // initial connection and reconnect verifies custody before h2 handshake.
            crate::managed::supervisor::authenticate_payload_peer(
                &connector.root,
                &connector.invocation,
                pid,
                &connector.environment,
            )
            .await
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::PermissionDenied, error))?;
            Ok(hyper_util::rt::TokioIo::new(stream))
        })
    }
}

pub(super) fn config() -> Result<connectrpc::client::ClientConfig> {
    Ok(connectrpc::client::ClientConfig::new(
        "http://localhost"
            .parse()
            .map_err(super::engine::protocol)?,
    )
    .with_protocol(connectrpc::Protocol::Grpc))
}
pub(crate) fn transport_error(error: ConnectError) -> Error {
    use base64::Engine as _;
    use buffa::Message as _;
    for detail in &error.details {
        let name = detail
            .type_url
            .strip_prefix("type.googleapis.com/")
            .unwrap_or(&detail.type_url);
        if name == "pse.code_intelligence.v1.Failure" {
            if let Some(value) = &detail.value {
                let decoded = base64::engine::general_purpose::STANDARD_NO_PAD
                    .decode(value)
                    .or_else(|_| base64::engine::general_purpose::STANDARD.decode(value));
                if let Ok(bytes) = decoded
                    && let Ok(failure) = pb::Failure::decode_from_slice(&bytes)
                {
                    return wire::failure_from_pb(failure).map_or_else(
                        |error| error,
                        |failure| Error::Refusal(retain_rpc_details(failure, &error)),
                    );
                }
            }
            return Error::Transport("malformed typed semantic RPC failure detail".into());
        }
    }
    if error.code == connectrpc::ErrorCode::InvalidArgument {
        Error::Refusal(retain_rpc_details(
            super::Failure::from_error(&Error::InvalidRequest(error.to_string())),
            &error,
        ))
    } else {
        Error::Transport(format!("local semantic RPC: {error}"))
    }
}

fn validation_error(error: protovalidate_buffa::ValidationError) -> ConnectError {
    let category = if error.compile_error.is_some()
        || error.runtime_error.is_some()
        || error.violations.is_empty()
    {
        Error::Validator("request validator could not execute".into())
    } else {
        Error::InvalidRequest("request constraint rejected".into())
    };
    let diagnostic =
        crate::envelope::RedactionPolicy::from_environment().redact(&error.to_string());
    let mut failure = super::Failure::from_error(&category);
    failure.backend_data = Some(serde_json::json!({"original_validation_diagnostic":diagnostic}));
    let transport = error.into_connect_error();
    match wire::failure_to_pb(failure) {
        Ok(failure) => transport.with_detail(connectrpc::ErrorDetail::from_message(
            "pse.code_intelligence.v1.Failure",
            &failure,
        )),
        Err(_) => transport,
    }
}
fn retain_rpc_details(mut failure: super::Failure, error: &ConnectError) -> super::Failure {
    let originals = error
        .details
        .iter()
        .filter(|detail| {
            detail
                .type_url
                .strip_prefix("type.googleapis.com/")
                .unwrap_or(&detail.type_url)
                != "pse.code_intelligence.v1.Failure"
        })
        .collect::<Vec<_>>();
    if !originals.is_empty() {
        let details = serde_json::json!(originals);
        match failure.backend_data.as_mut() {
            Some(serde_json::Value::Object(object)) => {
                object.insert("original_rpc_details".into(), details);
            }
            _ => {
                failure.backend_data = Some(
                    serde_json::json!({"original_rpc_details":details,"original_backend_data":failure.backend_data.take()}),
                );
            }
        }
    }
    failure
}
fn typed_error(error: &Error, transport: ConnectError) -> ConnectError {
    match wire::failure_to_pb(super::Failure::from_error(error)) {
        Ok(failure) => transport.with_detail(connectrpc::ErrorDetail::from_message(
            "pse.code_intelligence.v1.Failure",
            &failure,
        )),
        Err(_) => transport,
    }
}

/// Reusable authenticated connection; reconnects repeat original managed-custody verification.
pub struct Client {
    inner: bindings::NativeSemanticClient<connectrpc::client::SharedHttp2Connection>,
}
impl std::fmt::Debug for Client {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Client").finish_non_exhaustive()
    }
}
impl Client {
    /// Connect to an explicitly prepared original session without preparing another helper.
    pub async fn connect(
        root: &std::path::Path,
        environment: &Environment,
        record: &super::SessionRecord,
    ) -> Result<Self> {
        if record.version != super::SESSION_PROTOCOL_VERSION {
            return Err(Error::Unavailable("prepared semantic helper uses an incompatible protocol; stop it and explicitly prepare a new session".into()));
        }
        let connector = AuthenticatedConnector {
            socket: super::service::socket_path(root, environment, &record.invocation, false)?,
            root: root.to_owned(),
            invocation: record.invocation.clone(),
            environment: environment.clone(),
        };
        let configuration = config()?;
        let connection = connectrpc::client::Http2Connection::connect_with_connector(
            connector,
            configuration.base_uri().clone(),
        )
        .await
        .map_err(transport_error)?
        .shared(32);
        Ok(Self {
            inner: bindings::NativeSemanticClient::new(connection, configuration),
        })
    }
    /// Perform one fixed semantic operation, observing cancellation through its settled terminal.
    pub async fn request(
        &self,
        request: &Request,
        cancellation: Option<&CancellationToken>,
    ) -> Result<Response> {
        if cancellation.is_some_and(CancellationToken::is_cancelled) {
            return Err(Error::Cancelled);
        }
        let client = &self.inner;
        match request {
            Request::Status {} => Ok(Response::Status(wire::status_from_pb(
                client
                    .status(buffa_types::google::protobuf::Empty::default())
                    .await
                    .map_err(transport_error)?
                    .into_owned(),
            )?)),
            Request::Stop {} => {
                client
                    .stop(buffa_types::google::protobuf::Empty::default())
                    .await
                    .map_err(transport_error)?;
                Ok(Response::Stopped)
            }
            Request::PackageContext { package_id } => match client
                .package_context(pb::NativePackageRequest {
                    package_id: package_id.clone(),
                    ..Default::default()
                })
                .await
                .map_err(transport_error)?
                .into_owned()
                .result
            {
                Some(pb::native_package_reply::Result::Answer(answer)) => Ok(
                    Response::PackageContext(wire::package_answer_from_pb(*answer)?),
                ),
                Some(pb::native_package_reply::Result::Failure(failure)) => {
                    Err(Error::Refusal(wire::failure_from_pb(*failure)?))
                }
                None => Err(Error::Protocol("package reply has no outcome".into())),
            },
            Request::Query(_) | Request::WorkspaceSymbols(_) | Request::Graph(_) => {
                let mut conversation = client.run().await.map_err(transport_error)?;
                conversation
                    .send(pb::NativeControl {
                        action: Some(pb::native_control::Action::Start(Box::new(
                            wire::request_to_pb(request)?,
                        ))),
                        ..Default::default()
                    })
                    .await
                    .map_err(transport_error)?;
                if cancellation.is_none() {
                    conversation.close_send();
                }
                let mut requested = false;
                let mut outcome = None;
                let mut terminal = None;
                loop {
                    let event = if let Some(cancellation) =
                        cancellation.filter(|_| !requested && terminal.is_none())
                    {
                        tokio::select! {
                            result = conversation.message::<pb::NativeEvent>() => result,
                            _ = cancellation.cancelled() => {
                                requested = true;
                                conversation.send(pb::NativeControl { action: Some(pb::native_control::Action::Cancel(Default::default())), ..Default::default() }).await.map_err(|error| Error::CancellationUnobserved { cause: Box::new(transport_error(error)) })?;
                                conversation.close_send();
                                continue;
                            }
                        }
                    } else {
                        conversation.message::<pb::NativeEvent>().await
                    };
                    let event = event.map_err(|error| {
                        let cause = transport_error(error);
                        if requested {
                            Error::CancellationUnobserved {
                                cause: Box::new(cause),
                            }
                        } else {
                            cause
                        }
                    })?;
                    let Some(event) = event else {
                        break;
                    };
                    let event = event.to_owned_message();
                    if terminal.is_some() {
                        return Err(Error::Protocol("semantic event followed Terminal".into()));
                    }
                    if let Some(pb::native_event::Item::Terminal(value)) = event.item {
                        terminal = Some(value);
                    } else {
                        if outcome.is_some() {
                            return Err(Error::Protocol(
                                "multiple semantic operation results".into(),
                            ));
                        }
                        outcome = response_from_event(event)?;
                    }
                }
                let terminal = terminal.ok_or_else(|| {
                    let cause = Box::new(Error::Protocol(
                        "semantic stream closed without settled Terminal".into(),
                    ));
                    if requested {
                        Error::CancellationUnobserved { cause }
                    } else {
                        *cause
                    }
                })?;
                let outcome = outcome.ok_or_else(|| {
                    Error::Schema("semantic Terminal has no original operation outcome".into())
                })?;
                finish_run(outcome, *terminal, requested)
            }
        }
    }
}
pub(super) async fn client(
    root: &std::path::Path,
    environment: &Environment,
    record: &super::SessionRecord,
    request: &Request,
    cancellation: Option<&CancellationToken>,
) -> Result<Response> {
    Client::connect(root, environment, record)
        .await?
        .request(request, cancellation)
        .await
}

/// Validate settlement for every conversation, independently of whether this caller cancelled.
pub(crate) fn finish_run(
    outcome: Result<Response>,
    terminal: pb::Terminal,
    requested: bool,
) -> Result<Response> {
    if !terminal.completed {
        return Err(Error::Schema(
            "semantic Terminal did not establish operation completion".into(),
        ));
    }
    let completion = match terminal.completion.as_known() {
        Some(pb::terminal::Completion::CpuSettled) => super::Settlement::CpuSettled,
        Some(pb::terminal::Completion::LspReturned) => super::Settlement::LspReturned,
        Some(pb::terminal::Completion::LspDetached) => super::Settlement::LspDetached,
        _ => {
            let cause = Error::Schema("unknown or unobserved terminal settlement".into());
            return Err(if requested {
                Error::CancellationUnobserved {
                    cause: Box::new(cause),
                }
            } else {
                cause
            });
        }
    };
    let original = match terminal.original_outcome {
        Some(pb::terminal::OriginalOutcome::Succeeded(_)) => super::OriginalOutcome::Succeeded,
        Some(pb::terminal::OriginalOutcome::Failed(failure)) => {
            super::OriginalOutcome::Failed(wire::failure_from_pb(*failure)?)
        }
        None => return Err(Error::Schema("Terminal omitted original outcome".into())),
    };
    if outcome.is_ok() && matches!(&original, super::OriginalOutcome::Failed(_)) {
        return Err(Error::Schema(
            "successful semantic result contradicts failed original outcome".into(),
        ));
    }
    // A control/deadline failure can differ from the original provider outcome.
    if requested {
        Err(Error::CancelledObserved {
            completion,
            original,
        })
    } else {
        outcome
    }
}

pub(crate) struct Session {
    engine: Arc<ServiceEngine>,
    stop: CancellationToken,
    workers: Workers,
}
impl Session {
    pub(super) fn new(engine: Arc<ServiceEngine>, stop: CancellationToken) -> Self {
        Self {
            engine,
            workers: Workers::new(stop.clone()),
            stop,
        }
    }
    #[cfg(test)]
    pub(crate) fn fixture(
        fixture: Arc<crate::plan61_rpc_qualification::Fixture>,
        stop: CancellationToken,
    ) -> Self {
        Self::new(Arc::new(ServiceEngine::Fixture(fixture)), stop)
    }
    pub(crate) fn reap(&self) -> Result<()> {
        self.workers.reap()
    }
    pub(crate) async fn shutdown(&self) -> Result<()> {
        self.workers.drain().await
    }

    async fn unary(
        self: &Arc<Self>,
        request: Request,
        deadline: Option<std::time::Instant>,
    ) -> Result<Response> {
        let token = self.stop.child_token();
        let _guard = CancelOnDrop(token.clone());
        let (tx, rx) = oneshot::channel();
        let session = self.clone();
        self.workers.spawn(
            async move {
                tracing::debug!("semantic unary work admitted");
                let work = session.engine.execute(request, token.clone());
                tokio::pin!(work);
                let outcome = tokio::select! {
                    outcome = &mut work => outcome,
                    _ = deadline_elapsed(deadline) => {
                        tracing::debug!(control = "deadline", "semantic cancellation requested");
                        token.cancel();
                        work.await
                    },
                };
                tracing::debug!(
                    completion = outcome.completion.name(),
                    original_success = outcome.result.is_ok(),
                    "semantic unary work settled"
                );
                let _sent = tx.send(outcome.result);
            }
            .instrument(tracing::info_span!("semantic.rpc.unary")),
        )?;
        rx.await.map_err(super::engine::protocol)?
    }

    #[cfg(test)]
    pub(crate) fn run(
        self: &Arc<Self>,
        requests: ServiceStream<pb::NativeControl>,
    ) -> Result<ServiceStream<pb::NativeEvent>> {
        self.run_with_deadline(requests, None)
    }

    fn run_with_deadline(
        self: &Arc<Self>,
        mut requests: ServiceStream<pb::NativeControl>,
        deadline: Option<std::time::Instant>,
    ) -> Result<ServiceStream<pb::NativeEvent>> {
        let token = self.stop.child_token();
        let guard = CancelOnDrop(token.clone());
        let (tx, rx) = mpsc::channel(1);
        let session = self.clone();
        self.workers.spawn(async move {
            let expiry = deadline_elapsed(deadline);
            tokio::pin!(expiry);
            let first = tokio::select! {
                first = requests.next() => first,
                _ = token.cancelled() => return,
                _ = &mut expiry => { let _sent = tx.try_send(Err(ConnectError::deadline_exceeded("semantic request deadline elapsed before Start"))); return; },
            };
            let request = match first {
                Some(Ok(pb::NativeControl { action: Some(pb::native_control::Action::Start(request)), .. })) => {
                    use protovalidate_buffa::Validate as _;
                    match request.validate().map_err(validation_error).and_then(|()| wire::request_from_pb(*request).map_err(|error| typed_error(&error, ConnectError::invalid_argument(error.to_string())))) {
                        Ok(request) => request,
                        Err(error) => { let _sent = tx.try_send(Err(error)); return; }
                    }
                }
                Some(Err(error)) => { let _sent = tx.try_send(Err(error)); return; }
                _ => { let error = Error::InvalidRequest("exactly one Start must be first".into()); let _sent = tx.try_send(Err(typed_error(&error, ConnectError::invalid_argument(error.to_string())))); return; }
            };
            if deadline.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
                let _sent = tx.try_send(Err(ConnectError::deadline_exceeded("semantic request deadline elapsed before admission")));
                return;
            }
            tracing::debug!(operation = operation_name(&request), "semantic conversation work admitted");
            let work = session.engine.execute(request, token.clone());
            tokio::pin!(work);
            let mut open = true;
            let mut cancelled = false;
            let mut control_failure = None;
            let mut expired = false;
            let outcome = loop {
                tokio::select! {
                    next = requests.next(), if open => match next {
                        None => open = false, // A graceful request half-close permits completion.
                        Some(Ok(control)) if matches!(control.action, Some(pb::native_control::Action::Cancel(_))) => { if !cancelled { tracing::debug!(control = "cancel", "semantic cancellation requested"); } cancelled = true; token.cancel(); }
                        Some(Ok(_)) => { tracing::debug!(control = "invalid", "semantic cancellation requested"); control_failure = Some(Error::InvalidRequest("only Cancel may follow Start".into())); cancelled = true; open = false; token.cancel(); }
                        Some(Err(error)) => { tracing::debug!(control = "transport_error", "semantic cancellation requested"); control_failure = Some(transport_error(error)); cancelled = true; open = false; token.cancel(); }
                    },
                    result = &mut work => break result,
                    _ = &mut expiry, if !expired => {
                        tracing::debug!(control = "deadline", "semantic cancellation requested");
                        expired = true;
                        cancelled = true;
                        control_failure = Some(Error::Transport("semantic RPC deadline elapsed".into()));
                        token.cancel();
                    },
                    _ = tx.closed() => { tracing::debug!(control = "output_closed", "semantic cancellation requested"); token.cancel(); cancelled = true; break work.await; },
                }
            };
            tracing::debug!(completion = outcome.completion.name(), original_success = outcome.result.is_ok(), cancellation_requested = cancelled || token.is_cancelled(), "semantic conversation work settled");
            let original_failure = outcome.result.as_ref().err().map(super::Failure::from_error);
            let event = if let Some(error) = control_failure {
                failure_event(&error)
            } else {
                match outcome.result { Ok(response) => wire::response_event(response), Err(error) => Ok(failure_event(&error)) }
                    .unwrap_or_else(|error| failure_event(&error))
            };
            if !deliver(&tx, event, &session.stop).await { return; }
            let terminal = terminal(outcome.completion, cancelled || token.is_cancelled(), original_failure);
            let _delivered = deliver(&tx, terminal, &session.stop).await;
        }.instrument(tracing::info_span!("semantic.rpc.conversation")))?;
        Ok(Box::pin(futures_util::stream::unfold(
            (rx, guard),
            |(mut rx, guard)| async move { rx.recv().await.map(|item| (item, (rx, guard))) },
        )))
    }
}
async fn deadline_elapsed(deadline: Option<std::time::Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await,
        None => std::future::pending().await,
    }
}
async fn deliver(
    tx: &mpsc::Sender<std::result::Result<pb::NativeEvent, ConnectError>>,
    event: pb::NativeEvent,
    stop: &CancellationToken,
) -> bool {
    match tx.try_send(Ok(event)) {
        Ok(()) => true,
        Err(mpsc::error::TrySendError::Closed(_)) => false,
        Err(mpsc::error::TrySendError::Full(event)) => {
            tokio::select! { sent = tx.send(event) => sent.is_ok(), _ = stop.cancelled() => false }
        }
    }
}

pub(super) struct Outcome {
    pub(super) result: Result<Response>,
    pub(super) completion: Completion,
}
#[derive(Clone, Copy)]
pub(super) enum Completion {
    CpuSettled,
    LspReturned,
    LspDetached,
}
impl Completion {
    fn name(self) -> &'static str {
        match self {
            Self::CpuSettled => "cpu_settled",
            Self::LspReturned => "lsp_returned",
            Self::LspDetached => "lsp_detached",
        }
    }
}
fn operation_name(request: &Request) -> &'static str {
    match request {
        Request::Status {} => "status",
        Request::Stop {} => "stop",
        Request::Query(_) => "query",
        Request::WorkspaceSymbols(_) => "workspace_symbols",
        Request::Graph(_) => "bounded_relationships",
        Request::PackageContext { .. } => "package_context",
    }
}

fn failure_event(error: &Error) -> pb::NativeEvent {
    let failure = encoded_failure(super::Failure::from_error(error));
    pb::NativeEvent {
        item: Some(pb::native_event::Item::Failure(Box::new(failure))),
        ..Default::default()
    }
}
fn encoded_failure(failure: super::Failure) -> pb::Failure {
    wire::failure_to_pb(failure).unwrap_or_else(|error| pb::Failure {
        phase: pb::failure::Phase::Transport.into(),
        state: pb::failure::State::Failed.into(),
        code: "semantic.encoding".into(),
        message: error.to_string(),
        ..Default::default()
    })
}
fn response_from_event(event: pb::NativeEvent) -> Result<Option<Result<Response>>> {
    Ok(match event.item {
        Some(pb::native_event::Item::Query(answer)) => Some(
            wire::answer_from_pb(*answer)
                .map(Box::new)
                .map(Response::Query),
        ),
        Some(pb::native_event::Item::Workspace(answer)) => {
            Some(wire::workspace_from_pb(*answer).map(Response::WorkspaceSymbols))
        }
        Some(pb::native_event::Item::Graph(answer)) => {
            Some(wire::graph_answer_from_pb(*answer).map(Response::Graph))
        }
        Some(pb::native_event::Item::Failure(failure)) => {
            Some(Err(Error::Refusal(wire::failure_from_pb(*failure)?)))
        }
        Some(pb::native_event::Item::Terminal(_)) => None,
        None => return Err(Error::Protocol("semantic event has no item".into())),
    })
}
fn terminal(
    completion: Completion,
    cancellation_requested: bool,
    original_failure: Option<super::Failure>,
) -> pb::NativeEvent {
    let completion = match completion {
        Completion::CpuSettled => pb::terminal::Completion::CpuSettled,
        Completion::LspReturned => pb::terminal::Completion::LspReturned,
        Completion::LspDetached => pb::terminal::Completion::LspDetached,
    };
    let cancelled = original_failure
        .as_ref()
        .is_some_and(|failure| matches!(failure.state, super::FailureState::Cancelled));
    let original_outcome = match original_failure {
        Some(failure) => Some(pb::terminal::OriginalOutcome::Failed(Box::new(
            encoded_failure(failure),
        ))),
        None => Some(pb::terminal::OriginalOutcome::Succeeded(Default::default())),
    };
    pb::NativeEvent {
        item: Some(pb::native_event::Item::Terminal(Box::new(pb::Terminal {
            completed: true,
            cancelled,
            completion: completion.into(),
            cancellation_requested,
            original_outcome,
            ..Default::default()
        }))),
        ..Default::default()
    }
}

impl ServiceEngine {
    async fn execute(self: &Arc<Self>, request: Request, token: CancellationToken) -> Outcome {
        if matches!(&request, Request::Status {}) {
            let engine = self.clone();
            let task = crate::source_command::batch::spawn_cpu(token.clone(), move |token| {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                engine.status().map(Response::Status)
            });
            return Outcome {
                result: settle_cpu(task, &token).await,
                completion: Completion::CpuSettled,
            };
        }
        match self.as_ref() {
            #[cfg(test)]
            ServiceEngine::Fixture(engine) => {
                let result = settle_cpu(engine.start(request, token.clone()), &token).await;
                Outcome {
                    result,
                    completion: Completion::CpuSettled,
                }
            }
            ServiceEngine::Lsp(engine) => {
                let future = async {
                    match request {
                        Request::Status {} => engine.status().map(Response::Status),
                        Request::Stop {} => Err(Error::Protocol("stop is session-owned".into())),
                        Request::PackageContext { .. } => Err(Error::unsupported(
                            "selected LSP session has no captured native package context",
                        )),
                        Request::Query(query) => {
                            engine.query(query).await.map(Box::new).map(Response::Query)
                        }
                        Request::WorkspaceSymbols(query) => engine
                            .workspace_symbols(query)
                            .await
                            .map(Response::WorkspaceSymbols),
                        Request::Graph(_) => Err(Error::unsupported(
                            "bounded relationships require one explicitly prepared native_snapshot world",
                        )),
                    }
                };
                tokio::select! {
                    result = future => Outcome { result, completion: Completion::LspReturned },
                    _ = token.cancelled() => Outcome { result: Err(Error::Cancelled), completion: Completion::LspDetached },
                }
            }
            #[cfg(feature = "native-hir")]
            ServiceEngine::Snapshot(engine) => {
                let result = match request {
                    Request::Query(query) => {
                        settle_cpu(engine.start_query(query, token.clone()), &token)
                            .await
                            .map(Box::new)
                            .map(Response::Query)
                    }
                    Request::WorkspaceSymbols(query) => {
                        settle_cpu(engine.start_symbols(query, token.clone()), &token)
                            .await
                            .map(Response::WorkspaceSymbols)
                    }
                    Request::Graph(query) => {
                        settle_cpu(engine.start_graph(query, token.clone()), &token)
                            .await
                            .map(Response::Graph)
                    }
                    Request::PackageContext { package_id } => {
                        let engine = self.clone();
                        let task =
                            crate::source_command::batch::spawn_cpu(token.clone(), move |token| {
                                if token.is_cancelled() {
                                    return Err(Error::Cancelled);
                                }
                                match engine.as_ref() {
                                    ServiceEngine::Snapshot(engine) => engine
                                        .package_context(&package_id)
                                        .map(Response::PackageContext),
                                    ServiceEngine::Lsp(_) => {
                                        Err(Error::Protocol("package owner changed".into()))
                                    }
                                    #[cfg(test)]
                                    ServiceEngine::Fixture(_) => {
                                        Err(Error::Protocol("package owner changed".into()))
                                    }
                                }
                            });
                        settle_cpu(task, &token).await
                    }
                    Request::Status {} => engine.status().map(Response::Status),
                    Request::Stop {} => Err(Error::Protocol("stop is session-owned".into())),
                };
                Outcome {
                    result,
                    completion: Completion::CpuSettled,
                }
            }
        }
    }
}
async fn settle_cpu<T>(
    mut task: crate::source_command::batch::CpuTask<Result<T>>,
    token: &CancellationToken,
) -> Result<T> {
    tokio::select! {
        result = task.settle() => result.map_err(super::engine::protocol)?,
        _ = token.cancelled() => { task.cancel(); task.settle().await.map_err(super::engine::protocol)? }
    }
}

struct Implementation(Arc<Session>);
impl Implementation {
    async fn session_status(
        &self,
        deadline: Option<std::time::Instant>,
    ) -> std::result::Result<pb::NativeSessionStatus, ConnectError> {
        self.0
            .unary(Request::Status {}, deadline)
            .await
            .and_then(Response::into_status)
            .and_then(wire::status_to_pb)
            .map_err(|error| {
                typed_error(&error, ConnectError::failed_precondition(error.to_string()))
            })
    }
}
fn authenticated(context: &connectrpc::RequestContext) -> std::result::Result<(), ConnectError> {
    let peer = context
        .extensions()
        .get::<PeerContext>()
        .ok_or_else(|| ConnectError::unauthenticated("missing Unix credentials"))?;
    if peer.uid != rustix::process::geteuid().as_raw() {
        return Err(ConnectError::permission_denied(
            "semantic session owner differs",
        ));
    }
    let _identity = (peer.gid, peer.pid);
    Ok(())
}
#[protovalidate_buffa::connect_impl]
impl bindings::NativeSemantic for Implementation {
    async fn status<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        _: connectrpc::ServiceRequest<'_, buffa_types::google::protobuf::Empty>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::NativeSessionStatus> + Send + use<'a>,
    > {
        authenticated(&context)?;
        Ok(connectrpc::Response::new(
            self.session_status(context.deadline()).await?,
        ))
    }
    async fn capabilities<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        _: connectrpc::ServiceRequest<'_, buffa_types::google::protobuf::Empty>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::NativeSessionStatus> + Send + use<'a>,
    > {
        authenticated(&context)?;
        Ok(connectrpc::Response::new(
            self.session_status(context.deadline()).await?,
        ))
    }
    async fn stop<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        _: connectrpc::ServiceRequest<'_, buffa_types::google::protobuf::Empty>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<buffa_types::google::protobuf::Empty> + Send + use<'a>,
    > {
        authenticated(&context)?;
        self.0
            .workers
            .stop()
            .map_err(|error| ConnectError::internal(error.to_string()))?;
        Ok(connectrpc::Response::new(
            buffa_types::google::protobuf::Empty::default(),
        ))
    }
    async fn package_context<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::NativePackageRequest>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::NativePackageReply> + Send + use<'a>,
    > {
        authenticated(&context)?;
        let request = request
            .view()
            .to_owned_message()
            .map_err(|error| ConnectError::invalid_argument(error.to_string()))?;
        let response = self
            .0
            .unary(
                Request::PackageContext {
                    package_id: request.package_id,
                },
                context.deadline(),
            )
            .await;
        let result = match response
            .and_then(Response::into_package_context)
            .and_then(wire::package_answer_to_pb)
        {
            Ok(answer) => pb::native_package_reply::Result::Answer(Box::new(answer)),
            Err(error) => pb::native_package_reply::Result::Failure(Box::new(
                wire::failure_to_pb(super::Failure::from_error(&error))
                    .map_err(|error| ConnectError::internal(error.to_string()))?,
            )),
        };
        Ok(connectrpc::Response::new(pb::NativePackageReply {
            result: Some(result),
            ..Default::default()
        }))
    }
    async fn run(
        &self,
        context: connectrpc::RequestContext,
        requests: connectrpc::InboundStream<pb::NativeControl>,
    ) -> connectrpc::ServiceResult<
        ServiceStream<impl connectrpc::Encodable<pb::NativeEvent> + Send + use<>>,
    > {
        authenticated(&context)?;
        let stream = self
            .0
            .run_with_deadline(
                Box::pin(requests.map(|item| item.map(|value| value.to_owned_message()))),
                context.deadline(),
            )
            .map_err(|error| ConnectError::unavailable(error.to_string()))?;
        Ok(connectrpc::Response::new(stream))
    }
}

pub(super) async fn serve(
    listener: tokio::net::UnixListener,
    engine: Arc<ServiceEngine>,
    stopping: CancellationToken,
    mut terminate: tokio::signal::unix::Signal,
) -> Result<()> {
    use bindings::NativeSemanticExt as _;
    use tower_service::Service as _;
    let session = Arc::new(Session::new(engine.clone(), stopping.clone()));
    let router = Arc::new(Implementation(session.clone())).register(connectrpc::Router::new());
    let service = connectrpc::ConnectRpcService::new(router).with_limits(
        connectrpc::Limits::default()
            .with_max_request_body_size(64 * 1024)
            .with_max_message_size(64 * 1024)
            .with_element_memory_limit(1024 * 1024),
    );
    let mut connections = tokio::task::JoinSet::new();
    let mut check = tokio::time::interval(Duration::from_millis(250));
    let result = loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Err(error) => break Err(error.into()),
                Ok((stream, _)) => {
                    let peer = match stream.peer_cred() { Ok(peer) if peer.uid() == rustix::process::geteuid().as_raw() => peer, _ => continue };
                    let peer = PeerContext { uid: peer.uid(), gid: peer.gid(), pid: peer.pid() };
                    let service = service.clone();
                    let token = stopping.clone();
                    connections.spawn(async move {
                        let hyper_service = hyper::service::service_fn(move |mut request: http::Request<hyper::body::Incoming>| {
                            let mut service = service.clone();
                            request.extensions_mut().insert(peer.clone());
                            service.call(request)
                        });
                        let builder = hyper::server::conn::http2::Builder::new(hyper_util::rt::TokioExecutor::new());
                        let connection = builder.serve_connection(hyper_util::rt::TokioIo::new(stream), hyper_service);
                        tokio::pin!(connection);
                        tokio::select! { _ = &mut connection => {}, _ = token.cancelled() => { connection.as_mut().graceful_shutdown(); } }
                    });
                }
            },
            _ = stopping.cancelled() => break Ok(()),
            _ = terminate.recv() => break Ok(()),
            _ = tokio::signal::ctrl_c() => break Ok(()),
            Some(_) = connections.join_next(), if !connections.is_empty() => {},
            _ = check.tick() => {
                if let Err(error) = session.reap() {
                    break Err(error);
                } else if engine.driver_finished() {
                    break Err(Error::Protocol("analyzer connection terminated".into()));
                }
            }
        }
    };
    let drained = session.shutdown().await;
    stopping.cancel();
    // Connection futures may disappear; the independent worker group has already drained.
    while connections.join_next().await.is_some() {}
    result?;
    drained
}

#[cfg(test)]
pub(crate) async fn serve_fixture(
    listener: tokio::net::UnixListener,
    fixture: Arc<crate::plan61_rpc_qualification::Fixture>,
    stopping: CancellationToken,
) -> Result<()> {
    let terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    serve(
        listener,
        Arc::new(ServiceEngine::Fixture(fixture)),
        stopping,
        terminate,
    )
    .await
}
