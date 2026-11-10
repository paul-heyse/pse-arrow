// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated ConnectRPC service on authenticated, explicitly prepared local Unix custody.

use super::{
    EngineeringService, EngineeringServiceError,
    generated::{self, services as bindings},
};
use crate::CancelSource;
use connectrpc::{ConnectError, ServiceStream};
use generated::pb::pse::engineering::v1 as pb;
pub use pse_host::engineering_client::EngineeringEndpoint;
use std::sync::Arc;

mod adoption;
mod ingress;
mod analytical;
mod expectation;
mod inspection;
mod outgoing;
mod responses;
mod study;

/// Independent generated client over authenticated initial and reconnected HTTP/2 channels.
pub struct EngineeringClient {
    inner: bindings::EngineeringServiceClient<connectrpc::client::SharedHttp2Connection>,
    explicit: bindings::ExplicitStudyPreparationV2Client<connectrpc::client::SharedHttp2Connection>,
    expanded: bindings::ExpandedStudyPreparationV2Client<connectrpc::client::SharedHttp2Connection>,
    study: bindings::StudyControlV2Client<connectrpc::client::SharedHttp2Connection>,
}
impl std::fmt::Debug for EngineeringClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EngineeringClient")
            .finish_non_exhaustive()
    }
}
impl EngineeringClient {
    /// Authenticate the prepared payload before HTTP/2 negotiation, including every reconnect.
    /// # Errors
    /// Missing/replaced/stale original custody and transport failures refuse the connection.
    pub async fn connect(endpoint: EngineeringEndpoint) -> Result<Self, EngineeringServiceError> {
        use sha2::Digest as _;
        let (connection, config) = endpoint.connect().await.map_err(infrastructure)?;
        let client = Self::from_connection(connection, config);
        let response = client
            .describe_contract(pb::EngineeringDescribeRequest::default())
            .await
            .map_err(infrastructure)?;
        let described = response.view();
        if described.descriptor != generated::DESCRIPTORS
            || described.descriptor_sha256 != &sha2::Sha256::digest(described.descriptor)[..]
        {
            return Err(EngineeringServiceError::Interpretation("prepared service descriptor differs from this client's compiled source-owned contract".into()));
        }
        Ok(client)
    }

    fn from_connection(
        connection: connectrpc::client::SharedHttp2Connection,
        config: connectrpc::client::ClientConfig,
    ) -> Self {
        Self {
            inner: bindings::EngineeringServiceClient::new(connection.clone(), config.clone()),
            explicit: bindings::ExplicitStudyPreparationV2Client::new(
                connection.clone(),
                config.clone(),
            ),
            expanded: bindings::ExpandedStudyPreparationV2Client::new(
                connection.clone(),
                config.clone(),
            ),
            study: bindings::StudyControlV2Client::new(connection, config),
        }
    }
    /// Use generated bindings for ordinary bounded responses. The operation helpers
    /// select larger response limits only for their original complete-response contract.
    pub fn generated(
        &self,
    ) -> &bindings::EngineeringServiceClient<connectrpc::client::SharedHttp2Connection> {
        &self.inner
    }
    /// Generated client for explicit-point preparation and recovery.
    pub fn explicit_study(
        &self,
    ) -> &bindings::ExplicitStudyPreparationV2Client<connectrpc::client::SharedHttp2Connection>
    {
        &self.explicit
    }
    /// Generated client for source-expansion preparation and recovery.
    pub fn expanded_study(
        &self,
    ) -> &bindings::ExpandedStudyPreparationV2Client<connectrpc::client::SharedHttp2Connection>
    {
        &self.expanded
    }
    /// Generated client for shared Study execution and observations.
    pub fn study(
        &self,
    ) -> &bindings::StudyControlV2Client<connectrpc::client::SharedHttp2Connection> {
        &self.study
    }
}

/// Explicit local admission policy, evaluated against kernel credentials before RPC dispatch.
#[derive(Clone, Debug)]
pub struct LocalAccessPolicy {
    /// Admitted kernel user identity.
    pub uid: u32,
    /// Optional admitted kernel group identity.
    pub gid: Option<u32>,
    /// Finite connection retention; independent of scientific CPU placement.
    pub maximum_connections: usize,
    /// Finite transport drain after original native work has actually settled.
    pub connection_drain: std::time::Duration,
}
#[derive(Clone)]
struct KernelPeer {
    uid: u32,
    gid: u32,
    pid: Option<i32>,
}
#[derive(Clone)]
struct Implementation {
    engine: Arc<EngineeringService>,
    policy: LocalAccessPolicy,
}
impl Implementation {
    fn authenticate(&self, context: &connectrpc::RequestContext) -> Result<(), ConnectError> {
        let peer = context
            .extensions()
            .get::<KernelPeer>()
            .ok_or_else(|| ConnectError::unauthenticated("missing kernel peer credentials"))?;
        if peer.uid != self.policy.uid
            || self.policy.gid.is_some_and(|gid| gid != peer.gid)
            || peer.pid.is_none()
        {
            return Err(ConnectError::permission_denied(
                "kernel peer is outside the explicit engineering access policy",
            ));
        }
        Ok(())
    }

    fn describe_selected_contract(
        &self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringDescribeRequest>,
        explicit_study: bool,
    ) -> connectrpc::ServiceResult<AccountedResponse<pb::EngineeringContract>> {
        self.authenticate(&context)?;
        crate::engineering_projection::decode_engineering_describe_request(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        let output = if explicit_study {
            self.engine.describe_study_contract()
        } else {
            self.engine.describe_contract()
        }
        .map_err(transport)?;
        Ok(connectrpc::Response::new(AccountedResponse {
            message: crate::engineering_projection::encode_engineering_contract(output.document)
                .map_err(study::projection_error)?,
            reservation: output.owner,
            maximum_bytes: super::DOCUMENT_RESPONSE_BYTES,
        }))
    }
}

impl bindings::EngineeringService for Implementation {
    expectation::methods!();
    analytical::methods!();
    inspection::methods!();
    async fn inspect_physical<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringInspectPhysical>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::PhysicalInspectionDocument> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_inspect_physical(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        let cancel = CancelSource::new().with_deadline(context.deadline());
        let output = self
            .engine
            .inspect_physical(input, &cancel)
            .await
            .map_err(transport)?;
        let encoded = crate::preparation_projection::encode_owned_physical_inspection_document(
            output.document,
        )
        .map_err(study::projection_error)?;
        let owner: Arc<dyn pse_columnar::PayloadOwner> = Arc::new((output.owner, encoded.owner));
        Ok(connectrpc::Response::new(AccountedResponse {
            message: encoded.document,
            reservation: owner,
            maximum_bytes: 64 << 10,
        }))
    }
    async fn read_source_file<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringReadSourceFile>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::OriginalSourceFile> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_read_source_file(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        let cancel = CancelSource::new().with_deadline(context.deadline());
        let output = self
            .engine
            .read_source_file(input, &cancel)
            .await
            .map_err(transport)?;
        let message = crate::engineering_projection::encode_original_source_file(output.document)
            .map_err(study::projection_error)?;
        Ok(connectrpc::Response::new(AccountedResponse {
            message,
            reservation: output.owner,
            maximum_bytes: output.maximum_bytes,
        }))
    }
    async fn discover_preparation<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringDiscoverPreparation>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::PreparationDiscoveryReport> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_discover_preparation(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        let (package, services) = self
            .engine
            .inspection_source(input.source_operation)
            .map_err(transport)?;
        let cancel = CancelSource::new().with_deadline(context.deadline());
        let report = package
            .preparation_discovery(&services, &input.request, &cancel)
            .await
            .map_err(|error| transport(EngineeringServiceError::Owner(Box::new(error))))?;
        let report = crate::engineering_projection::encode_preparation_discovery_report(report)
            .map_err(study::projection_error)?;
        Ok(connectrpc::Response::new(outgoing::Validated(report)))
    }
    async fn edit_source<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringEditSource>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringSourceOutcome> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_edit_source(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        let outcome = self.engine.edit_source(input).await.map_err(transport)?;
        Ok(connectrpc::Response::new(outgoing::Validated(
            crate::engineering_projection::encode_engineering_source_outcome(outcome)
                .map_err(study::projection_error)?,
        )))
    }
    async fn describe_contract<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringDescribeRequest>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringContract> + Send + use<'a>,
    > {
        self.describe_selected_contract(context, request, false)
    }
    async fn admit_source<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringAdmitSource>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringSourceOutcome> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_admit_source(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        let outcome = self.engine.admit_source(input).await.map_err(transport)?;
        Ok(connectrpc::Response::new(outgoing::Validated(
            crate::engineering_projection::encode_engineering_source_outcome(outcome)
                .map_err(study::projection_error)?,
        )))
    }
    async fn source_status<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringOperationKey>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringSourceOutcome> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_operation_key(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        Ok(connectrpc::Response::new(outgoing::Validated(
            crate::engineering_projection::encode_engineering_source_outcome(
                self.engine.source_status(&input).map_err(transport)?,
            )
            .map_err(study::projection_error)?,
        )))
    }
    async fn inspect_source<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringInspectSource>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringDeclarationPage> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_inspect_source(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        let cancel = CancelSource::new().with_deadline(context.deadline());
        let page = self
            .engine
            .inspect_source(input, &cancel)
            .await
            .map_err(transport)?;
        let reservation = page.owner();
        Ok(connectrpc::Response::new(AccountedResponse {
            message: crate::engineering_projection::encode_engineering_declaration_page(
                page.as_ref().clone(),
            )
            .map_err(study::projection_error)?,
            reservation,
            maximum_bytes: 64 << 10,
        }))
    }
    async fn submit_run<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringSubmitRun>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringRunStatus> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_submit_run(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        Ok(connectrpc::Response::new(outgoing::Validated(
            crate::engineering_projection::encode_engineering_run_status(
                self.engine.submit_run(input).await.map_err(transport)?,
            )
            .map_err(study::projection_error)?,
        )))
    }
    async fn run_status<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringRunKey>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringRunStatus> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input =
            crate::engineering_projection::decode_engineering_run_key(request.to_owned_message())
                .map_err(study::projection_error)?;
        Ok(connectrpc::Response::new(outgoing::Validated(
            crate::engineering_projection::encode_engineering_run_status(
                self.engine.run_status(&input).await.map_err(transport)?,
            )
            .map_err(study::projection_error)?,
        )))
    }
    async fn cancel_run<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringRunKey>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringRunStatus> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input =
            crate::engineering_projection::decode_engineering_run_key(request.to_owned_message())
                .map_err(study::projection_error)?;
        Ok(connectrpc::Response::new(outgoing::Validated(
            crate::engineering_projection::encode_engineering_run_status(
                self.engine.cancel_run(&input).await.map_err(transport)?,
            )
            .map_err(study::projection_error)?,
        )))
    }
    async fn read_result<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringReadResult>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringResultPage> + Send + use<'a>,
    > {
        self.authenticate(&context)?;
        let input = crate::engineering_projection::decode_engineering_read_result(
            request.to_owned_message(),
        )
        .map_err(study::projection_error)?;
        let cancel = CancelSource::new().with_deadline(context.deadline());
        let output = self
            .engine
            .read_result(input, &cancel)
            .await
            .map_err(transport)?;
        Ok(connectrpc::Response::new(AccountedResponse {
            message: crate::engineering_projection::encode_engineering_result_page(output.page)
                .map_err(study::projection_error)?,
            reservation: output.reservation,
            maximum_bytes: output.maximum_bytes,
        }))
    }
    async fn observe_run(
        &self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringRunKey>,
    ) -> connectrpc::ServiceResult<
        ServiceStream<impl connectrpc::Encodable<pb::EngineeringRunStatus> + Send + use<>>,
    > {
        self.authenticate(&context)?;
        let key =
            crate::engineering_projection::decode_engineering_run_key(request.to_owned_message())
                .map_err(study::projection_error)?;
        let engine = self.engine.clone();
        let deadline = context.deadline();
        let stream = futures_util::stream::unfold(
            (engine, key, None, false),
            move |(engine, key, mut previous, done)| async move {
                if done {
                    return None;
                }
                let next = async {
                    loop {
                        check_run_observation_deadline(deadline)?;
                        // Terminal status can acquire original completion protection.
                        // Once started, preserve that read's outcome before reporting expiry.
                        let current = engine.run_status(&key).await.map_err(transport)?;
                        check_run_observation_deadline(deadline)?;
                        if previous.as_ref() != Some(&current) {
                            return Ok(current);
                        }
                        let changed = engine.observation_changed(&key);
                        match deadline {
                            Some(deadline) => tokio::time::timeout_at(deadline.into(), changed)
                                .await
                                .map_err(|_| run_observation_deadline_error())?,
                            None => changed.await,
                        }
                        .map_err(transport)?;
                    }
                };
                let current = match next.await {
                    Ok(current) => current,
                    Err(error) => return Some((Err(error), (engine, key, previous, true))),
                };
                let done = current.settled || current.failure.is_some();
                previous = Some(current.clone());
                let encoded = crate::engineering_projection::encode_engineering_run_status(current)
                    .map(outgoing::Validated)
                    .map_err(study::projection_error);
                let done = done || encoded.is_err();
                Some((encoded, (engine, key, previous, done)))
            },
        );
        let stream: ServiceStream<outgoing::Validated<pb::EngineeringRunStatus>> = Box::pin(stream);
        Ok(connectrpc::Response::new(stream))
    }
}

impl bindings::StudyControlV2 for Implementation {
    study::control_methods!();
    async fn describe_study_contract<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::EngineeringDescribeRequest>,
    ) -> connectrpc::ServiceResult<
        impl connectrpc::Encodable<pb::EngineeringContract> + Send + use<'a>,
    > {
        self.describe_selected_contract(context, request, true)
    }
}

fn check_run_observation_deadline(
    deadline: Option<std::time::Instant>,
) -> Result<(), ConnectError> {
    if deadline.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
        return Err(run_observation_deadline_error());
    }
    Ok(())
}

fn run_observation_deadline_error() -> ConnectError {
    ConnectError::deadline_exceeded(
        "observation allowance elapsed; original work remains under its retained owner",
    )
}

/// Host one prepared service, draining original work on shutdown or listener failure.
/// # Errors
/// Listener, connection join and native settlement failures remain failures.
pub async fn serve(
    listener: tokio::net::UnixListener,
    engine: Arc<EngineeringService>,
    policy: LocalAccessPolicy,
    stopping: CancelSource,
) -> Result<(), EngineeringServiceError> {
    if policy.maximum_connections == 0 || policy.connection_drain.is_zero() {
        drop(listener);
        let refusal = EngineeringServiceError::Request(
            "connection retention extent and transport drain allowance must be positive".into(),
        );
        return match engine.shutdown().await {
            Ok(()) => Err(refusal),
            Err(cleanup) => Err(EngineeringServiceError::Shutdown(vec![refusal, cleanup])),
        };
    }
    let maximum_connections = policy.maximum_connections;
    let connection_drain = policy.connection_drain;
    let implementation = Arc::new(Implementation {
        engine: engine.clone(),
        policy: policy.clone(),
    });
    let router = bindings::EngineeringServiceExt::register(
        implementation.clone(),
        connectrpc::Router::new(),
    );
    let router = bindings::ExplicitStudyPreparationV2Ext::register(implementation.clone(), router);
    let router = bindings::ExpandedStudyPreparationV2Ext::register(implementation.clone(), router);
    let router = bindings::StudyControlV2Ext::register(implementation, router);
    let service = connectrpc::ConnectRpcService::new(router).with_limits(ingress::limits());
    let ingress_pool = engine.ingress_pool();
    let mut connections = tokio::task::JoinSet::new();
    let mut failures = Vec::new();
    loop {
        tokio::select! {
            biased;
            _ = stopping.cancelled() => break,
            joined = connections.join_next(), if !connections.is_empty() => {
                if let Some(error) = joined.and_then(|joined| connection_failure(joined, false)) {
                    failures.push(error);
                }
            },
            accepted = listener.accept() => match accepted {
                Err(error) => { failures.push(infrastructure(error)); break; },
                Ok((stream, _)) => {
                    let peer = match stream.peer_cred() {
                        Ok(peer) if peer.uid() == policy.uid && policy.gid.is_none_or(|gid| gid == peer.gid()) && peer.pid().is_some() => peer,
                        _ => continue,
                    };
                    if connections.len() >= maximum_connections { continue; }
                    let peer = KernelPeer { uid: peer.uid(), gid: peer.gid(), pid: peer.pid() };
                    let service = service.clone();
                    let ingress_pool = ingress_pool.clone();
                    let stop = stopping.clone();
                    connections.spawn(async move {
                        let hyper_service = hyper::service::service_fn(move |mut request: http::Request<hyper::body::Incoming>| {
                            request.extensions_mut().insert(peer.clone());
                            ingress::call(service.clone(), ingress_pool.clone(), request)
                        });
                        let builder = hyper::server::conn::http2::Builder::new(hyper_util::rt::TokioExecutor::new());
                        let connection = builder.serve_connection(hyper_util::rt::TokioIo::new(stream), hyper_service);
                        tokio::pin!(connection);
                        tokio::select! {
                            result = &mut connection => result,
                            _ = stop.cancelled() => { connection.as_mut().graceful_shutdown(); connection.await },
                        }
                    });
                }
            },
        }
    }
    // Stop offering new kernel connections before awaiting any performing owner.
    // Already accepted HTTP/2 connections continue polling their graceful drain.
    drop(listener);
    stopping.cancel();
    if let Err(error) = engine.shutdown().await {
        failures.push(error);
    }
    let drained = tokio::time::timeout(connection_drain, async {
        while let Some(joined) = connections.join_next().await {
            if let Some(error) = connection_failure(joined, false) {
                failures.push(error);
            }
        }
    })
    .await;
    if drained.is_err() {
        // Only connection tasks remain here. Their drop cannot cancel or replace
        // the original native settlement awaited above.
        failures.push(infrastructure(
            "transport drain allowance elapsed after original native settlement",
        ));
        connections.abort_all();
        while let Some(joined) = connections.join_next().await {
            if let Some(error) = connection_failure(joined, true) {
                failures.push(error);
            }
        }
    }
    shutdown_result(failures)
}

// A task joining successfully does not establish that its Hyper connection succeeded.
// Retain original typed sources; ignore cancellation only after our explicit abort_all.
fn connection_failure<E>(
    joined: Result<Result<(), E>, tokio::task::JoinError>,
    expected_abort: bool,
) -> Option<EngineeringServiceError>
where
    E: std::error::Error + Send + Sync + 'static,
{
    match joined {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(EngineeringServiceError::Owner(Box::new(error))),
        Err(error) if expected_abort && error.is_cancelled() => None,
        Err(error) => Some(EngineeringServiceError::Owner(Box::new(error))),
    }
}

fn shutdown_result(failures: Vec<EngineeringServiceError>) -> Result<(), EngineeringServiceError> {
    if failures.is_empty() {
        Ok(())
    } else {
        Err(EngineeringServiceError::Shutdown(failures))
    }
}

fn infrastructure(error: impl std::fmt::Display) -> EngineeringServiceError {
    EngineeringServiceError::Infrastructure(error.to_string())
}
fn transport(error: EngineeringServiceError) -> ConnectError {
    let failure =
        crate::engineering_projection::encode_engineering_failure(super::session::failure(&error));
    let code = pse_diagnostics::TypedDiagnostic::diagnostic_code(&error);
    let mapped = match code {
        Some(
            pse_diagnostics::DiagnosticCode::WorkflowInput
            | pse_diagnostics::DiagnosticCode::ValidationInvariant,
        ) => ConnectError::invalid_argument(error.to_string()),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit) => {
            ConnectError::resource_exhausted(error.to_string())
        }
        Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled) => {
            ConnectError::canceled(error.to_string())
        }
        Some(
            pse_diagnostics::DiagnosticCode::WorkflowInternal
            | pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
        ) => ConnectError::internal(error.to_string()),
        _ => ConnectError::failed_precondition(error.to_string()),
    };
    let failure = match failure {
        Ok(failure) => failure,
        Err(projection) => {
            return ConnectError::internal("native engineering failure projection refused")
                .with_source(EngineeringServiceError::Shutdown(vec![
                    error,
                    EngineeringServiceError::Owner(Box::new(
                        crate::projection_diagnostics::ProjectionDiagnosticError::from(projection),
                    )),
                ]));
        }
    };
    mapped
        .with_detail(connectrpc::ErrorDetail::from_message(
            "pse.engineering.v1.EngineeringFailure",
            &failure,
        ))
        .with_source(error)
}

struct AccountedResponse<M> {
    message: M,
    reservation: Arc<dyn pse_columnar::PayloadOwner>,
    maximum_bytes: usize,
}
struct AccountedBytes {
    bytes: bytes::Bytes,
    _reservation: Arc<dyn pse_columnar::PayloadOwner>,
}
impl AsRef<[u8]> for AccountedBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl<M: buffa::Message + serde::Serialize + protovalidate_buffa::Validate> connectrpc::Encodable<M>
    for AccountedResponse<M>
{
    fn encode(&self, codec: connectrpc::CodecFormat) -> Result<bytes::Bytes, ConnectError> {
        let bytes = outgoing::encode_bounded(&self.message, codec, self.maximum_bytes)?;
        Ok(bytes::Bytes::from_owner(AccountedBytes {
            bytes,
            _reservation: self.reservation.clone(),
        }))
    }
}

#[cfg(test)]
mod connection_settlement_tests {
    use super::*;

    #[tokio::test]
    async fn connection_settlement_preserves_inner_failure_and_independent_native_shutdown() {
        let mut connections = tokio::task::JoinSet::new();
        connections.spawn(async { Ok::<_, std::io::Error>(()) });
        connections.spawn(async { Err(std::io::Error::other("original connection failure")) });
        let mut failures = vec![EngineeringServiceError::Request(
            "original native shutdown refusal".into(),
        )];
        while let Some(joined) = connections.join_next().await {
            if let Some(error) = connection_failure(joined, false) {
                failures.push(error);
            }
        }
        let EngineeringServiceError::Shutdown(failures) = shutdown_result(failures).unwrap_err()
        else {
            panic!("independent failures must remain an aggregate")
        };
        assert_eq!(failures.len(), 2);
        assert!(
            matches!(&failures[0], EngineeringServiceError::Request(message) if message == "original native shutdown refusal")
        );
        let EngineeringServiceError::Owner(source) = &failures[1] else {
            panic!("connection cause lost")
        };
        assert_eq!(
            source.downcast_ref::<std::io::Error>().unwrap().to_string(),
            "original connection failure"
        );
        assert!(
            connection_failure(
                Ok(Err(std::io::Error::other("completed before abort"))),
                true
            )
            .is_some()
        );
        assert!(shutdown_result(Vec::new()).is_ok());
    }

    #[tokio::test]
    async fn connection_settlement_retains_panics_even_after_abort_request() {
        let mut connections = tokio::task::JoinSet::<Result<(), std::io::Error>>::new();
        connections.spawn(async { panic!("original connection panic") });
        let joined = connections.join_next().await.unwrap();
        assert!(joined.as_ref().unwrap_err().is_panic());
        let EngineeringServiceError::Owner(source) = connection_failure(joined, true).unwrap()
        else {
            panic!("original task failure lost")
        };
        assert!(
            source
                .downcast_ref::<tokio::task::JoinError>()
                .unwrap()
                .is_panic()
        );
    }

    #[tokio::test]
    async fn connection_settlement_ignores_only_expected_abort_cancellation() {
        for expected_abort in [false, true] {
            let mut connections = tokio::task::JoinSet::<Result<(), std::io::Error>>::new();
            connections.spawn(std::future::pending());
            connections.abort_all();
            let joined = connections.join_next().await.unwrap();
            assert!(joined.as_ref().unwrap_err().is_cancelled());
            let failure = connection_failure(joined, expected_abort);
            assert_eq!(failure.is_none(), expected_abort);
            assert!(connections.is_empty());
        }
    }
}
