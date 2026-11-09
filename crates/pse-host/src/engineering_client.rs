// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent explicit Study client for an authenticated prepared local endpoint.

use crate::{envelope::CliInvocationId, environment::Environment};
use connectrpc::{
    ConnectError,
    client::{CallOptions, ClientConfig, Http2Connection, SharedHttp2Connection, UnaryResponse},
};
use sha2::{Digest as _, Sha256};
use std::path::PathBuf;

mod auth;

#[path = "generated/engineering-explicit/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
pub mod pb;
#[path = "generated/engineering-explicit/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator spelling and packaging modules")]
mod validation;
/// Generated explicit preparation and shared Study control procedures.
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, reason = "Pinned service generator surface")]
#[allow(clippy::match_single_binding, reason = "Pinned service generator emits empty streaming branches")]
pub mod services {
    include!("generated/engineering-explicit/services/pse.engineering.v1.mod.rs");
}

/// The complete selected explicit Study descriptor and import world.
pub const DESCRIPTORS: &[u8] = include_bytes!("generated/engineering-explicit/descriptor.bin");
/// Complete encoded ceiling for contract, workbench and prepared Study documents.
pub const DOCUMENT_RESPONSE_BYTES: usize = 16 << 20;
/// Separate ceiling for collection and unknown-field view allocation.
pub const VIEW_ELEMENT_MEMORY_BYTES: usize = 32 << 20;

/// Exact whole-response allowance, plus the independent view-element limit.
pub fn response_options(maximum_bytes: usize) -> CallOptions {
    CallOptions::default()
        .with_max_message_size(maximum_bytes)
        .with_element_memory_limit(VIEW_ELEMENT_MEMORY_BYTES)
}

/// Default local transport configuration used by selected and full clients.
/// # Errors
/// Refuses an invalid built-in local endpoint URI.
pub fn client_config() -> Result<ClientConfig, ConnectError> {
    let uri = "http://localhost"
        .parse()
        .map_err(|_| ConnectError::internal("invalid fixed local engineering URI"))?;
    Ok(ClientConfig::new(uri)
        .with_protocol(connectrpc::Protocol::Grpc)
        .with_default_max_message_size(4 << 20)
        .with_default_element_memory_limit(VIEW_ELEMENT_MEMORY_BYTES))
}

/// Explicitly prepared original service. Connecting never launches or rebuilds it.
#[derive(Clone, Debug)]
pub struct EngineeringEndpoint {
    /// Checkout whose managed invocation owns the service payload.
    pub root: PathBuf,
    /// Original managed service invocation.
    pub invocation: CliInvocationId,
    /// Published socket for that exact incarnation.
    pub socket: PathBuf,
    /// Approved checkout environment for original kernel membership.
    pub environment: Environment,
    /// Original payload image, reobserved on every connection.
    pub artifact: pse_buildinfo::FileObservation,
    /// Native executable attestation from the prepared original payload.
    pub build: pse_ids::ContentHash,
}

impl EngineeringEndpoint {
    /// Connect with the shared bounded policy and authenticate on every Unix connection.
    /// # Errors
    /// Transport or mismatched original endpoint custody refuses the connection.
    pub async fn connect(self) -> Result<(SharedHttp2Connection, ClientConfig), ConnectError> {
        let config = client_config()?;
        let connection = Http2Connection::connect_with_connector(
            AuthenticatedConnector(self),
            config.base_uri().clone(),
        )
        .await
        .map_err(|error| ConnectError::unavailable(error.to_string()))?
        .shared(32);
        Ok((connection, config))
    }
}

#[derive(Clone)]
struct AuthenticatedConnector(EngineeringEndpoint);
impl tower_service::Service<http::Uri> for AuthenticatedConnector {
    type Response = hyper_util::rt::TokioIo<tokio::net::UnixStream>;
    type Error = std::io::Error;
    type Future =
        std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: http::Uri) -> Self::Future {
        let endpoint = self.0.clone();
        Box::pin(async move {
            let stream = tokio::net::UnixStream::connect(&endpoint.socket).await?;
            let pid = stream
                .peer_cred()?
                .pid()
                .and_then(|pid| u32::try_from(pid).ok())
                .ok_or_else(|| denied("kernel service PID unavailable"))?;
            let original = crate::managed::supervisor::authenticate_payload_peer(
                &endpoint.root,
                &endpoint.invocation,
                pid,
                &endpoint.environment,
            )
            .await
            .map_err(|error| denied(&error.to_string()))?;
            tokio::task::spawn_blocking(move || {
                auth::authenticate_image(&original, &endpoint.artifact, endpoint.build)
            })
            .await
            .map_err(|error| denied(&format!("payload image observation failed: {error}")))??;
            Ok(hyper_util::rt::TokioIo::new(stream))
        })
    }
}

fn denied(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::PermissionDenied, message)
}

/// Typed explicit Study preparation and shared controls, independent of the performing runtime.
#[derive(Clone)]
pub struct ExplicitStudyClient {
    /// Explicit preparation and original definition recovery.
    pub preparation: services::ExplicitStudyPreparationV2Client<SharedHttp2Connection>,
    /// Shared original Study control and protected observations.
    pub control: services::StudyControlV2Client<SharedHttp2Connection>,
}

impl std::fmt::Debug for ExplicitStudyClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExplicitStudyClient")
            .finish_non_exhaustive()
    }
}

impl ExplicitStudyClient {
    /// Authenticate the endpoint and verify the exact compiled selected descriptor closure.
    /// # Errors
    /// Unavailable custody, failed discovery or an inconsistent selected descriptor refuses.
    pub async fn connect(endpoint: EngineeringEndpoint) -> Result<Self, ConnectError> {
        let (connection, config) = endpoint.connect().await?;
        let control = services::StudyControlV2Client::new(connection.clone(), config.clone());
        let response = control
            .describe_study_contract_with_options(
                pb::pse::engineering::v1::EngineeringDescribeRequest::default(),
                response_options(DOCUMENT_RESPONSE_BYTES),
            )
            .await?;
        let described = response.view();
        if described.descriptor != DESCRIPTORS
            || described.descriptor_sha256 != &Sha256::digest(described.descriptor)[..]
        {
            return Err(ConnectError::failed_precondition(
                "prepared service differs from the compiled explicit Study contract",
            ));
        }
        Ok(Self {
            preparation: services::ExplicitStudyPreparationV2Client::new(connection, config),
            control,
        })
    }

    /// Read the exact prepared points within the complete document response limit.
    /// # Errors
    /// Transport, original-operation admission or bounded decoding can refuse.
    pub async fn read_prepared_explicit_study(
        &self,
        request: pb::pse::engineering::v1::EngineeringOperationKey,
    ) -> Result<
        UnaryResponse<
            buffa::OwnedView<pb::pse::engineering::v1::ExplicitStudyDefinitionView<'static>>,
        >,
        ConnectError,
    > {
        self.preparation
            .read_prepared_explicit_study_with_options(
                request,
                response_options(DOCUMENT_RESPONSE_BYTES),
            )
            .await
    }
}
