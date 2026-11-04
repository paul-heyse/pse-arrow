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

    struct UnreadBody(Arc<AtomicUsize>);
    impl http_body::Body for UnreadBody {
        type Data = Bytes;
        type Error = Infallible;
        fn poll_frame(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<http_body::Frame<Bytes>, Infallible>>> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Poll::Ready(None)
        }
    }
    fn request(reads: Arc<AtomicUsize>) -> http::Request<UnreadBody> {
        http::Request::builder()
            .method("POST")
            .uri("/pse.engineering.v1.EngineeringService/DescribeContract")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(UnreadBody(reads))
            .unwrap()
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
        let service = ConnectRpcService::new(connectrpc::Router::new()).with_limits(limits());
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
        assert_ne!(first.status(), http::StatusCode::TOO_MANY_REQUESTS);
        assert_ne!(second.status(), http::StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(refused.status(), http::StatusCode::TOO_MANY_REQUESTS);
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
        assert_ne!(admitted.status(), http::StatusCode::TOO_MANY_REQUESTS);
        drop(second);
        drop(admitted);
        drop(refused);
        assert_eq!(pool.reserved(), 0);
    }
}
