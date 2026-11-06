// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Response decode policy follows the complete encoded allowance of the selected operation.

use super::*;

const DEFAULT_MESSAGE_BYTES: usize = 4 << 20;
const DEFAULT_ELEMENT_BYTES: usize = 32 << 20;
use super::super::DOCUMENT_RESPONSE_BYTES as DOCUMENT_BYTES;

pub(super) fn config(uri: http::Uri) -> connectrpc::client::ClientConfig {
    connectrpc::client::ClientConfig::new(uri)
        .with_protocol(connectrpc::Protocol::Grpc)
        .with_default_max_message_size(DEFAULT_MESSAGE_BYTES)
        .with_default_element_memory_limit(DEFAULT_ELEMENT_BYTES)
}

fn options(maximum_bytes: usize) -> connectrpc::client::CallOptions {
    // AccountedResponse enforces this ceiling on the complete encoded message:
    // metadata is already included, not an extra allowance for an IPC/body field.
    // Retain the supplier's finite 32 MiB view-element profile independently of
    // wire size. This bounds repeated/map/unknown-field amplification; it is not
    // an assertion that every legal nested document fits that decode profile.
    connectrpc::client::CallOptions::default()
        .with_max_message_size(maximum_bytes)
        .with_element_memory_limit(DEFAULT_ELEMENT_BYTES)
}

macro_rules! fixed_response {
    ($method:ident, $with_options:ident, $request:ident, $view:ident, $doc:literal) => {
        #[doc = $doc]
        /// The finite 32 MiB view-element profile independently bounds collection amplification.
        /// # Errors
        /// Request validation, transport and complete response decode refusals propagate.
        pub async fn $method(
            &self,
            request: pb::$request,
        ) -> Result<
            connectrpc::client::UnaryResponse<buffa::OwnedView<pb::__buffa::view::$view<'static>>>,
            ConnectError,
        > {
            adoption::validate(&request)?;
            self.inner
                .$with_options(request, options(DOCUMENT_BYTES))
                .await
        }
    };
}

macro_rules! selected_response {
    ($method:ident, $with_options:ident, $request:ident, $view:ident, $maximum:expr, $doc:literal) => {
        #[doc = $doc]
        /// The finite 32 MiB view-element profile independently bounds collection amplification.
        /// # Errors
        /// Invalid original allowances, transport and complete response decode refusals propagate.
        pub async fn $method(
            &self,
            request: pb::$request,
        ) -> Result<
            connectrpc::client::UnaryResponse<buffa::OwnedView<pb::__buffa::view::$view<'static>>>,
            ConnectError,
        > {
            adoption::validate(&request)?;
            let maximum_bytes: u64 = ($maximum)(&request)?;
            let maximum_bytes = usize::try_from(maximum_bytes).map_err(|_| {
                ConnectError::invalid_argument("response allowance exceeds addressable memory")
            })?;
            self.inner
                .$with_options(request, options(maximum_bytes))
                .await
        }
    };
}

impl EngineeringClient {
    fixed_response!(
        describe_contract,
        describe_contract_with_options,
        EngineeringDescribeRequest,
        EngineeringContractView,
        "Read the source-owned contract within its complete 16 MiB producer allowance."
    );
    fixed_response!(
        browse_studies,
        browse_studies_with_options,
        StudyWorkbenchRequest,
        StudyWorkbenchPageView,
        "Read the bounded workbench page within its complete 16 MiB producer allowance."
    );
    fixed_response!(
        read_prepared_study,
        read_prepared_study_with_options,
        EngineeringOperationKey,
        StudyDefinitionView,
        "Read the original prepared study within its complete 16 MiB producer allowance."
    );
    selected_response!(
        read_result,
        read_result_with_options,
        EngineeringReadResult,
        EngineeringResultPageView,
        |request: &pb::EngineeringReadResult| Ok::<_, ConnectError>(request.maximum_bytes),
        "Decode the complete selected result using its original request allowance."
    );
    selected_response!(
        read_study_result,
        read_study_result_with_options,
        EngineeringReadStudyResult,
        EngineeringStudyResultPageView,
        |request: &pb::EngineeringReadStudyResult| Ok::<_, ConnectError>(request.maximum_bytes),
        "Decode the complete selected study result using its original request allowance."
    );
    selected_response!(
        read_source_file,
        read_source_file_with_options,
        EngineeringReadSourceFile,
        OriginalSourceFileView,
        |request: &pb::EngineeringReadSourceFile| {
            request
                .request
                .as_option()
                .map(|request| request.maximum_bytes)
                .ok_or_else(|| {
                    ConnectError::invalid_argument("original source-file request is required")
                })
        },
        "Decode the exact original source file using its complete request allowance."
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower_service::Service as _;

    async fn client(
        router: connectrpc::Router,
    ) -> (
        EngineeringClient,
        tempfile::TempDir,
        tokio::task::JoinHandle<()>,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("responses.sock");
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let service = connectrpc::ConnectRpcService::new(router);
            let service = hyper::service::service_fn(move |request| {
                let mut service = service.clone();
                service.call(request)
            });
            hyper::server::conn::http2::Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(hyper_util::rt::TokioIo::new(socket), service)
                .await
                .unwrap();
        });
        let config = config("http://localhost".parse().unwrap());
        let connection =
            connectrpc::client::Http2Connection::connect_unix(path, config.base_uri().clone())
                .await
                .unwrap()
                .shared(32);
        (
            EngineeringClient {
                inner: bindings::EngineeringServiceClient::new(connection, config),
            },
            directory,
            server,
        )
    }

    async fn finish_server(server: tokio::task::JoinHandle<()>) {
        server.abort();
        if let Err(error) = server.await {
            assert!(error.is_cancelled(), "response fixture failed: {error}");
        }
    }

    #[tokio::test]
    async fn engineering_response_limits_preserve_selected_large_pages_and_complete_ceiling() {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let router = connectrpc::Router::new()
            .route::<_, pb::EngineeringReadResult, pb::EngineeringResultPage>(
                "pse.engineering.v1.EngineeringService",
                "ReadResult",
                connectrpc::handler::handler_fn(move |_, _: pb::EngineeringReadResult| {
                    observed.fetch_add(1, Ordering::SeqCst);
                    async {
                        Ok(connectrpc::Response::new(pb::EngineeringResultPage {
                            run: vec![1; 16],
                            relation: "runtime.solve_variables".into(),
                            ipc: vec![42; 5 << 20],
                            ..Default::default()
                        }))
                    }
                }),
            );
        let (client, _directory, server) = client(router).await;
        let mut request = pb::EngineeringReadResult {
            operation: "original-run".into(),
            relation: "runtime.solve_variables".into(),
            maximum_bytes: 6 << 20,
            run: vec![1; 16],
            attempt: vec![2; 16],
            end: 1,
            ..Default::default()
        };
        assert_eq!(
            client
                .read_result(request.clone())
                .await
                .unwrap()
                .into_owned()
                .ipc
                .len(),
            5 << 20
        );
        assert_eq!(
            client
                .generated()
                .read_result(request.clone())
                .await
                .unwrap_err()
                .code,
            connectrpc::ErrorCode::ResourceExhausted
        );
        request.maximum_bytes = 5 << 20;
        assert_eq!(
            client.read_result(request.clone()).await.unwrap_err().code,
            connectrpc::ErrorCode::ResourceExhausted,
            "the complete allowance includes response metadata, not just the IPC field"
        );
        request.maximum_bytes = (DOCUMENT_BYTES + 1) as u64;
        assert_eq!(
            client.read_result(request).await.unwrap_err().code,
            connectrpc::ErrorCode::InvalidArgument
        );
        assert_eq!(
            calls.load(Ordering::SeqCst),
            3,
            "invalid allowance refuses before dispatch"
        );
        drop(client);
        finish_server(server).await;
    }

    #[tokio::test]
    async fn engineering_response_limits_bound_document_bytes_and_element_amplification() {
        let count = DEFAULT_ELEMENT_BYTES
            / size_of::<pb::__buffa::view::EngineeringMethodContractView<'static>>()
            + 1;
        let calls = Arc::new(AtomicUsize::new(0));
        let router = connectrpc::Router::new()
            .route::<_, pb::EngineeringDescribeRequest, pb::EngineeringContract>(
                "pse.engineering.v1.EngineeringService",
                "DescribeContract",
                connectrpc::handler::handler_fn(move |_, _: pb::EngineeringDescribeRequest| {
                    let index = calls.fetch_add(1, Ordering::SeqCst);
                    async move {
                        let document = match index {
                            0 => pb::EngineeringContract {
                                descriptor: vec![42; 5 << 20],
                                ..Default::default()
                            },
                            1 => pb::EngineeringContract {
                                descriptor: vec![42; DOCUMENT_BYTES + 1],
                                ..Default::default()
                            },
                            _ => pb::EngineeringContract {
                                methods: vec![pb::EngineeringMethodContract::default(); count],
                                ..Default::default()
                            },
                        };
                        Ok(connectrpc::Response::new(document))
                    }
                }),
            );
        let (client, _directory, server) = client(router).await;
        assert_eq!(
            client
                .describe_contract(pb::EngineeringDescribeRequest::default())
                .await
                .unwrap()
                .into_owned()
                .descriptor
                .len(),
            5 << 20
        );
        assert_eq!(
            client
                .describe_contract(pb::EngineeringDescribeRequest::default())
                .await
                .unwrap_err()
                .code,
            connectrpc::ErrorCode::ResourceExhausted
        );
        let error = client
            .describe_contract(pb::EngineeringDescribeRequest::default())
            .await
            .unwrap_err();
        assert_eq!(error.code, connectrpc::ErrorCode::ResourceExhausted);
        assert!(
            error
                .message
                .as_deref()
                .is_some_and(|message| message.contains("element")),
            "tiny repeated records must hit the independent element budget: {error}"
        );
        drop(client);
        finish_server(server).await;
    }
}
