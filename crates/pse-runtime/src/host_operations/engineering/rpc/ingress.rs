// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Collective RPC memory admission precedes library body retention and decoding.
use bytes::Bytes;
use connectrpc::{ConnectError, ConnectRpcBody, ConnectRpcService};
use datafusion::execution::memory_pool::{MemoryConsumer, MemoryPool, MemoryReservation};
use std::{
    convert::Infallible,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower_service::Service as _;

const WIRE_BYTES: usize = 16 << 20;
const ELEMENT_BYTES: usize = 32 << 20;
// Cover compressed/decompressed collection growth, retained view wire bytes,
// native owned copies, and their transient encoding; four element extents cover
// view/owned collection capacities and native map allocation growth. The actual
// native request's independent grant takes custody before this permit can drop.
const REQUEST_EXTENT: usize = 8 * WIRE_BYTES + 4 * ELEMENT_BYTES + (64 << 10);

pub(super) fn limits() -> connectrpc::Limits {
    connectrpc::Limits::default()
        .with_max_request_body_size(WIRE_BYTES)
        .with_max_message_size(WIRE_BYTES)
        .with_element_memory_limit(ELEMENT_BYTES)
}

pub(super) struct ResponseBody {
    body: Pin<Box<ConnectRpcBody>>,
    _owner: Option<Arc<MemoryReservation>>,
}
impl http_body::Body for ResponseBody {
    type Data = Bytes;
    type Error = Infallible;
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Bytes>, Infallible>>> {
        self.get_mut().body.as_mut().poll_frame(cx)
    }
    fn is_end_stream(&self) -> bool {
        self.body.is_end_stream()
    }
    fn size_hint(&self) -> http_body::SizeHint {
        self.body.size_hint()
    }
}

pub(super) async fn call<D, B>(
    mut service: ConnectRpcService<D>,
    pool: Arc<dyn MemoryPool>,
    request: http::Request<B>,
) -> Result<http::Response<ResponseBody>, Infallible>
where
    D: connectrpc::Dispatcher,
    B: http_body::Body<Data = Bytes> + Send + 'static,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    let binary_grpc = request.method() == http::Method::POST
        && request.version() == http::Version::HTTP_2
        && connectrpc::Protocol::detect(request.headers()).is_some_and(|protocol| {
            protocol.protocol == connectrpc::Protocol::Grpc
                && protocol.codec_format == connectrpc::CodecFormat::Proto
        });
    if !binary_grpc {
        return Ok(ConnectError::unimplemented(
            "local engineering operations require binary gRPC POST over HTTP/2",
        )
        .into_http_response(request.headers())
        .map(|body| ResponseBody {
            body: Box::pin(body),
            _owner: None,
        }));
    }
    let reservation = MemoryConsumer::new("engineering:rpc-ingress").register(&pool);
    // Refuse without polling the body or constructing the library decode future.
    // The pinned library chooses the protocol's actual error/trailer framing.
    if reservation.try_grow(REQUEST_EXTENT).is_err() {
        return Ok(ConnectError::resource_exhausted(
            "shared engineering RPC ingress allowance exhausted",
        )
        .into_http_response(request.headers())
        .map(|body| ResponseBody {
            body: Box::pin(body),
            _owner: None,
        }));
    }
    let owner = Arc::new(reservation);
    let response = service.call(request).await?;
    Ok(response.map(|body| ResponseBody {
        body: Box::pin(body),
        _owner: Some(owner),
    }))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "bounded pool and actual protocol refusal controls"
    )]
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct ObservedBody {
        reads: Arc<AtomicUsize>,
        frame: Option<Bytes>,
    }
    impl http_body::Body for ObservedBody {
        type Data = Bytes;
        type Error = Infallible;
        fn poll_frame(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<http_body::Frame<Bytes>, Infallible>>> {
            let body = self.get_mut();
            body.reads.fetch_add(1, Ordering::SeqCst);
            Poll::Ready(
                body.frame
                    .take()
                    .map(|bytes| Ok(http_body::Frame::data(bytes))),
            )
        }
    }
    fn request(reads: Arc<AtomicUsize>) -> http::Request<ObservedBody> {
        http::Request::builder()
            .method("POST")
            .version(http::Version::HTTP_2)
            .uri("/pse.engineering.v1.EngineeringService/DescribeContract")
            .header(http::header::CONTENT_TYPE, "application/grpc+proto")
            .body(ObservedBody {
                reads,
                frame: Some(Bytes::from_static(&[0, 0, 0, 0, 0])),
            })
            .unwrap()
    }

    fn service(calls: Arc<AtomicUsize>) -> ConnectRpcService<connectrpc::Router> {
        use super::super::generated::pb::pse::engineering::v1 as pb;
        let router = connectrpc::Router::new()
            .route::<_, pb::EngineeringDescribeRequest, pb::EngineeringContract>(
                "pse.engineering.v1.EngineeringService",
                "DescribeContract",
                connectrpc::handler::handler_fn(move |_, _: pb::EngineeringDescribeRequest| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    async { Ok(connectrpc::Response::new(pb::EngineeringContract::default())) }
                }),
            );
        ConnectRpcService::new(router).with_limits(limits())
    }

    #[tokio::test]
    async fn engineering_ingress_rejects_other_protocols_before_body_allocation_or_dispatch() {
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service(calls.clone());
        let pool: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(REQUEST_EXTENT),
        );
        for (method, content_type, version) in [
            (
                http::Method::POST,
                Some("application/json"),
                http::Version::HTTP_2,
            ),
            (
                http::Method::POST,
                Some("application/proto"),
                http::Version::HTTP_2,
            ),
            (
                http::Method::POST,
                Some("application/connect+proto"),
                http::Version::HTTP_2,
            ),
            (
                http::Method::POST,
                Some("application/grpc+json"),
                http::Version::HTTP_2,
            ),
            (
                http::Method::POST,
                Some("application/grpc-web+proto"),
                http::Version::HTTP_2,
            ),
            (
                http::Method::POST,
                Some("application/grpc-web-text+proto"),
                http::Version::HTTP_2,
            ),
            (http::Method::GET, None, http::Version::HTTP_2),
            (
                http::Method::PUT,
                Some("application/grpc+proto"),
                http::Version::HTTP_2,
            ),
            (
                http::Method::POST,
                Some("application/grpc+proto"),
                http::Version::HTTP_11,
            ),
            (http::Method::POST, None, http::Version::HTTP_2),
            (
                http::Method::POST,
                Some("application/octet-stream"),
                http::Version::HTTP_2,
            ),
        ] {
            let reads = Arc::new(AtomicUsize::new(0));
            let mut request = request(reads.clone());
            *request.method_mut() = method;
            *request.version_mut() = version;
            request.headers_mut().remove(http::header::CONTENT_TYPE);
            if let Some(content_type) = content_type {
                request
                    .headers_mut()
                    .insert(http::header::CONTENT_TYPE, content_type.parse().unwrap());
            }
            let response = call(service.clone(), pool.clone(), request).await.unwrap();
            assert_eq!(reads.load(Ordering::SeqCst), 0);
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            assert_eq!(
                pool.reserved(),
                0,
                "refused protocol cannot retain an ingress grant"
            );
            assert!(response.body()._owner.is_none());
        }
        let empty: Arc<dyn MemoryPool> =
            Arc::new(datafusion::execution::memory_pool::GreedyMemoryPool::new(0));
        let mut unsupported = request(Arc::new(AtomicUsize::new(0)));
        unsupported.headers_mut().insert(
            http::header::CONTENT_TYPE,
            "application/grpc+json".parse().unwrap(),
        );
        let refused = call(service, empty, unsupported).await.unwrap();
        assert_eq!(
            refused.headers()["grpc-status"],
            "12",
            "protocol refusal precedes pool exhaustion"
        );
    }

    #[tokio::test]
    async fn engineering_ingress_accepts_library_binary_grpc_content_type_forms() {
        use http_body_util::BodyExt as _;
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service(calls.clone());
        let pool: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(REQUEST_EXTENT),
        );
        for content_type in [
            "application/grpc",
            "application/grpc+proto",
            "application/grpc+proto; charset=utf-8",
        ] {
            let reads = Arc::new(AtomicUsize::new(0));
            let mut request = request(reads.clone());
            request
                .headers_mut()
                .insert(http::header::CONTENT_TYPE, content_type.parse().unwrap());
            let response = call(service.clone(), pool.clone(), request).await.unwrap();
            assert_eq!(response.status(), http::StatusCode::OK);
            assert!(reads.load(Ordering::SeqCst) > 0);
            assert_eq!(pool.reserved(), REQUEST_EXTENT);
            let collected = response.into_body().collect().await.unwrap();
            assert_eq!(collected.trailers().unwrap()["grpc-status"], "0");
            drop(collected);
            assert_eq!(pool.reserved(), 0);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn engineering_ingress_collectively_refuses_before_body_decode_and_retains_response_custody()
     {
        let pool: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(REQUEST_EXTENT * 2),
        );
        let first_reads = Arc::new(AtomicUsize::new(0));
        let second_reads = Arc::new(AtomicUsize::new(0));
        let refused_reads = Arc::new(AtomicUsize::new(0));
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service(calls.clone());
        let (first, second, refused) = tokio::join!(
            call(service.clone(), pool.clone(), request(first_reads.clone())),
            call(service.clone(), pool.clone(), request(second_reads.clone())),
            call(
                service.clone(),
                pool.clone(),
                request(refused_reads.clone())
            ),
        );
        let (first, second, refused) = (first.unwrap(), second.unwrap(), refused.unwrap());
        assert_eq!(first.status(), http::StatusCode::OK);
        assert_eq!(second.status(), http::StatusCode::OK);
        assert_eq!(refused.headers()["grpc-status"], "8");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            refused_reads.load(Ordering::SeqCst),
            0,
            "pool refusal must not read a request body"
        );
        assert!(first_reads.load(Ordering::SeqCst) > 0);
        assert!(second_reads.load(Ordering::SeqCst) > 0);
        assert_eq!(pool.reserved(), REQUEST_EXTENT * 2);
        drop(first);
        assert_eq!(pool.reserved(), REQUEST_EXTENT);
        let admitted = call(service, pool.clone(), request(first_reads))
            .await
            .unwrap();
        assert_eq!(admitted.status(), http::StatusCode::OK);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        drop(second);
        drop(admitted);
        drop(refused);
        assert_eq!(pool.reserved(), 0);
    }

    #[tokio::test]
    async fn abandoned_h2_upload_releases_ingress_and_preserves_the_connection() {
        use buffa::Message as _;
        use futures_util::{StreamExt as _, stream};
        use http_body::Frame;
        use http_body_util::{BodyExt as _, Full, StreamBody};
        use hyper_util::rt::{TokioExecutor, TokioIo};
        use std::time::Duration;

        let runtime = crate::workflow::tests::runtime();
        let shared = runtime.shared.clone();
        let engine = super::super::EngineeringService::new(runtime, shared, 8).unwrap();
        let pool = engine.ingress_pool();
        let baseline = pool.reserved();
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("engineering.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let stop = crate::CancelSource::new();
        let serving = super::super::serve(
            listener,
            engine,
            super::super::LocalAccessPolicy {
                uid: rustix::process::geteuid().as_raw(),
                gid: None,
                maximum_connections: 1,
                connection_drain: Duration::from_secs(5),
            },
            stop.clone(),
        );
        let stop_after_reply = stop.clone();
        let exercise = tokio::spawn(async move {
            // The timeout is a test watchdog. Each transition below is driven by
            // actual body polling, native admission, or original settlement.
            tokio::time::timeout(Duration::from_secs(10), async {
                let peer = tokio::net::UnixStream::connect(socket)
                    .await
                    .unwrap()
                    .into_std()
                    .unwrap();
                let retained_peer = peer.try_clone().unwrap();
                let peer = tokio::net::UnixStream::from_std(peer).unwrap();
                let (mut client, connection) =
                    hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(peer))
                        .await
                        .unwrap();
                let exchange = async move {
                    let request = |body| {
                        http::Request::post("http://localhost/pse.engineering.v1.EngineeringService/DescribeContract")
                            .version(http::Version::HTTP_2)
                            .header(http::header::HOST, "localhost")
                            .header(http::header::CONTENT_TYPE, "application/grpc+proto")
                            .header("te", "trailers")
                            .body(body)
                            .unwrap()
                    };
                    let (started, sent) = tokio::sync::oneshot::channel();
                    let partial = stream::once(async move {
                        started.send(()).unwrap();
                        Ok::<_, Infallible>(Frame::data(Bytes::from_static(&[0, 0])))
                    })
                    .chain(stream::pending());
                    client.ready().await.unwrap();
                    let mut upload = Box::pin(
                        client.send_request(request(StreamBody::new(partial).boxed_unsync())),
                    );
                    tokio::select! {
                        reply = &mut upload => panic!("incomplete upload unexpectedly replied: {reply:?}"),
                        () = async {
                            sent.await.unwrap();
                            while pool.reserved() < baseline + REQUEST_EXTENT {
                                tokio::task::yield_now().await;
                            }
                        } => {}
                    }
                    // Hyper's response-future cancellation resets this one stream.
                    // Require release before issuing anything else on the same peer.
                    drop(upload);
                    while pool.reserved() != baseline {
                        tokio::task::yield_now().await;
                    }

                    let body = Full::new(Bytes::from_static(&[0, 0, 0, 0, 0])).boxed_unsync();
                    client.ready().await.unwrap();
                    let response = client.send_request(request(body)).await.unwrap();
                    let collected = response.into_body().collect().await.unwrap();
                    assert_eq!(collected.trailers().unwrap()["grpc-status"], "0");
                    let frame = collected.to_bytes();
                    assert_eq!(frame[0], 0);
                    let length = u32::from_be_bytes(frame[1..5].try_into().unwrap()) as usize;
                    assert_eq!(length, frame.len() - 5);
                    let contract = super::super::pb::EngineeringContract::decode_from_slice(&frame[5..])
                        .unwrap();
                    assert_eq!(contract.descriptor.as_slice(), super::super::generated::DESCRIPTORS);
                    // The server sends graceful shutdown while the original
                    // client still owns and polls this connection.
                    stop_after_reply.cancel();
                    client
                };
                let (client, driver) = tokio::join!(exchange, connection);
                driver.unwrap();
                drop(client);
                // The public driver can finish before the server's last writes.
                // Signal writer EOF but keep the original receive half through
                // server join so those writes still have a live socket peer.
                retained_peer.shutdown(std::net::Shutdown::Write).unwrap();
                retained_peer
            })
            .await
            .expect("the H2 upload, subsequent reply and original driver must settle")
        });
        let joined = async {
            let exercise = exercise.await;
            stop.cancel();
            exercise
        };
        let (result, service) = tokio::join!(joined, serving);
        service.unwrap();
        drop(result.unwrap());
    }
}
