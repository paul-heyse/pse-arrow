// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual executor, socket, copied-buffer and outbound-admission witnesses.
use super::super::{Client, DispatchState};
use super::*;
use http_body_util::{BodyExt, StreamBody};
use hyper::rt::Executor as _;
use pse_columnar::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
use std::{
    io,
    sync::{Condvar, atomic::AtomicUsize},
    time::{Duration, Instant},
};
use surrealdb_protocol::proto::{rpc::v1 as rpc, v1 as proto};
use surrealdb_types::prost::Message;

fn owned_context(pool: &Arc<dyn MemoryPool>, bytes: usize, deadline: Instant) -> RequestContext {
    let grant = MemoryConsumer::new("physical:outbound").register(pool);
    grant.try_grow(bytes).unwrap();
    let metadata = MemoryConsumer::new("physical:issued-read").register(pool);
    metadata
        .try_grow(crate::canonical::MetadataResponse::Protection.allocation_bytes())
        .unwrap();
    let selection = crate::canonical::ProtectedSelection::test_only(
        crate::canonical::Revision {
            key: "revision".into(),
            problem: "problem".into(),
            sequence: 1,
            parent: None,
            operation: "operation".into(),
            request: Vec::new().into(),
            interpretation: "test".into(),
        },
        "issued-read",
    );
    selection.retain_metadata_owner(Arc::new(metadata)).unwrap();
    RequestContext::new(deadline)
        .with_payload_owner(Arc::new(grant))
        .with_retained_request(selection)
}

struct BlockingDrop {
    entered: Option<tokio::sync::oneshot::Sender<()>>,
    release: Arc<(Mutex<bool>, Condvar)>,
    finished: Arc<AtomicBool>,
}
impl Drop for BlockingDrop {
    fn drop(&mut self) {
        if let Some(entered) = self.entered.take() {
            let _ = entered.send(());
        }
        let (lock, changed) = &*self.release;
        let mut released = lock.lock().unwrap();
        while !*released {
            released = changed.wait(released).unwrap();
        }
        self.finished.store(true, Ordering::Release);
    }
}
// Two workers are the fixture for one deliberately blocked native destructor
// plus its independent releaser; this does not constrain host test concurrency.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_joins_blocked_child_destructors_after_its_waiter_is_abandoned() {
    let metadata = crate::canonical::MetadataResponse::Protection.allocation_bytes();
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(metadata + 1));
    let context = owned_context(&pool, 1, Instant::now() + Duration::from_secs(5));
    let driver = REQUEST
        .scope(context.clone(), async { Driver::new() })
        .await;
    let (started, ready) = tokio::sync::oneshot::channel();
    let (entered, blocked) = tokio::sync::oneshot::channel();
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let finished = Arc::new(AtomicBool::new(false));
    let witness = BlockingDrop {
        entered: Some(entered),
        release: release.clone(),
        finished: finished.clone(),
    };
    OwnedExecutor(driver.tasks.clone()).execute(async move {
        let _witness = witness;
        started.send(()).unwrap();
        std::future::pending::<()>().await;
    });
    ready.await.unwrap();
    context.release_payload_after_drain();
    context.release_lifetime_after_drain();
    let closing = driver.clone();
    let waiter = tokio::spawn(async move {
        closing.shutdown().await;
    });
    blocked.await.unwrap();
    assert!(!driver.drained());
    assert_eq!(pool.reserved(), metadata + 1);
    waiter.abort();
    assert!(waiter.await.unwrap_err().is_cancelled());
    let mut successor = Box::pin(driver.shutdown());
    assert!(futures_util::poll!(successor.as_mut()).is_pending());
    {
        let (lock, changed) = &*release;
        *lock.lock().unwrap() = true;
        changed.notify_all();
    }
    tokio::time::timeout(Duration::from_secs(2), successor)
        .await
        .unwrap();
    assert!(finished.load(Ordering::Acquire));
    assert!(driver.drained());
    assert_eq!(pool.reserved(), 0);
}

struct DropCount(Arc<AtomicUsize>);
impl Drop for DropCount {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn executor_spawn_racing_its_seal_joins_every_admitted_or_rejected_child() {
    let driver = Driver::new();
    let count = Arc::new(AtomicUsize::new(0));
    let (started, ready) = tokio::sync::oneshot::channel();
    let executor = OwnedExecutor(driver.tasks.clone());
    let drops = count.clone();
    driver.tasks.spawn(async move {
        let mut started = Some(started);
        for _ in 0..128 {
            let witness = DropCount(drops.clone());
            executor.execute(async move {
                let _witness = witness;
                std::future::pending::<()>().await;
            });
            if let Some(started) = started.take() {
                let _ = started.send(());
            }
            std::thread::yield_now();
        }
    });
    ready.await.unwrap();
    driver.shutdown().await;
    assert_eq!(count.load(Ordering::Acquire), 128);
    assert!(driver.drained());
}

#[derive(Default)]
struct FlushGate {
    open: AtomicBool,
    waker: futures_util::task::AtomicWaker,
}
impl FlushGate {
    fn release(&self) {
        self.open.store(true, Ordering::Release);
        self.waker.wake();
    }
}
struct WitnessIo {
    inner: Option<TokioIo<tokio::net::TcpStream>>,
    gate: Arc<FlushGate>,
    dropped: Arc<AtomicBool>,
}
impl Drop for WitnessIo {
    fn drop(&mut self) {
        drop(self.inner.take());
        self.dropped.store(true, Ordering::Release);
    }
}
impl hyper::rt::Read for WitnessIo {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(self.inner.as_mut().unwrap()).poll_read(cx, buf)
    }
}
impl hyper::rt::Write for WitnessIo {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(self.inner.as_mut().unwrap()).poll_write(cx, buf)
    }
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(self.inner.as_mut().unwrap()).poll_write_vectored(cx, buf)
    }
    fn is_write_vectored(&self) -> bool {
        self.inner.as_ref().unwrap().is_write_vectored()
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.gate.waker.register(cx.waker());
        if !self.gate.open.load(Ordering::Acquire) {
            return Poll::Pending;
        }
        Pin::new(self.inner.as_mut().unwrap()).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(self.inner.as_mut().unwrap()).poll_shutdown(cx)
    }
}
struct SingleBody {
    bytes: Option<Bytes>,
    dropped: Arc<AtomicBool>,
}
impl Body for SingleBody {
    type Data = Bytes;
    type Error = io::Error;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, io::Error>>> {
        Poll::Ready(self.bytes.take().map(|bytes| Ok(Frame::data(bytes))))
    }
    fn is_end_stream(&self) -> bool {
        self.bytes.is_none()
    }
}
impl Drop for SingleBody {
    fn drop(&mut self) {
        drop(self.bytes.take());
        self.dropped.store(true, Ordering::Release);
    }
}

#[tokio::test]
async fn actual_io_flush_and_whole_task_join_cover_copied_suffix_and_chained_upload() {
    for bytes in [5, 64, 256 * 1024] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let incoming = Arc::new(Mutex::new(Vec::new()));
        let held = incoming.clone();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let service =
                hyper::service::service_fn(move |request: http::Request<hyper::body::Incoming>| {
                    let (send, receive) =
                        tokio::sync::mpsc::unbounded_channel::<Result<Frame<Bytes>, io::Error>>();
                    held.lock().unwrap().push((request.into_body(), send));
                    async move {
                        Ok::<_, std::convert::Infallible>(http::Response::new(StreamBody::new(
                            futures_util::stream::unfold(receive, |mut receive| async move {
                                receive.recv().await.map(|frame| (frame, receive))
                            }),
                        )))
                    }
                });
            let _ = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
                .initial_stream_window_size(1024)
                .initial_connection_window_size(64 << 10)
                .serve_connection(TokioIo::new(socket), service)
                .await;
        });
        let gate = Arc::new(FlushGate::default());
        gate.release();
        let socket_dropped = Arc::new(AtomicBool::new(false));
        let socket = tokio::net::TcpStream::connect(address).await.unwrap();
        let driver = Driver::new();
        let mut transport = OwnedHttp2::handshake(
            WitnessIo {
                inner: Some(TokioIo::new(socket)),
                gate: gate.clone(),
                dropped: socket_dropped.clone(),
            },
            address.to_string().parse().unwrap(),
            1 << 20,
            driver.clone(),
        )
        .await
        .unwrap();
        // Complete the initial HTTP/2 exchange before gating a DATA flush.
        let OwnedHttp2::Owned { sender, .. } = &mut transport;
        futures_util::future::poll_fn(|cx| sender.poll_ready(cx))
            .await
            .unwrap();
        let warm = sender
            .send_request(
                http::Request::builder()
                    .uri(format!("http://{address}/warmup"))
                    .body(OutboundBody::new(
                        tonic::body::Body::empty(),
                        None,
                        driver.tasks.io.clone(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let metadata = crate::canonical::MetadataResponse::Protection.allocation_bytes();
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(metadata + bytes));
        let context = owned_context(&pool, bytes, Instant::now() + Duration::from_secs(5));
        let body_dropped = Arc::new(AtomicBool::new(false));
        let consumed = Arc::new(AtomicBool::new(false));
        let observed = consumed.clone();
        let flush_gate = gate.clone();
        driver.tasks.io.on_consumption(move || {
            flush_gate.open.store(false, Ordering::Release);
            observed.store(true, Ordering::Release);
        });
        let body = OutboundBody::new(
            tonic::body::Body::new(SingleBody {
                bytes: Some(if bytes == 5 {
                    Bytes::from_static(b"\0\0\0\0\0")
                } else {
                    Bytes::from(vec![7; bytes])
                }),
                dropped: body_dropped.clone(),
            }),
            Some(&context),
            driver.tasks.io.clone(),
        );
        let OwnedHttp2::Owned { sender, .. } = &mut transport;
        futures_util::future::poll_fn(|cx| sender.poll_ready(cx))
            .await
            .unwrap();
        let response = sender
            .send_request(
                http::Request::builder()
                    .uri(format!("http://{address}/physical-upload"))
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while !consumed.load(Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        if bytes <= 64 {
            tokio::time::timeout(Duration::from_secs(2), async {
                while driver.tasks.io.pending_count() == 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
        }
        context.release_payload_after_drain();
        context.release_lifetime_after_drain();
        assert_eq!(
            pool.reserved(),
            metadata + bytes,
            "headers do not discharge outbound copies"
        );
        assert!(!socket_dropped.load(Ordering::Acquire));
        gate.release();
        if bytes <= 64 {
            tokio::time::timeout(Duration::from_secs(2), context.outbound_drained())
                .await
                .unwrap();
            assert_eq!(
                pool.reserved(),
                0,
                "small copied bytes retire only after actual underlying flush"
            );
            assert!(
                !driver.closed(),
                "successful flush keeps the healthy connection usable"
            );
        } else {
            assert!(
                tokio::time::timeout(Duration::from_millis(50), context.outbound_drained())
                    .await
                    .is_err(),
                "unread stream retains a chained upload beyond its advertised receive window"
            );
        }
        driver.shutdown().await;
        assert!(driver.drained());
        assert!(socket_dropped.load(Ordering::Acquire));
        assert!(body_dropped.load(Ordering::Acquire));
        assert_eq!(pool.reserved(), 0);
        drop(response);
        drop(warm);
        drop(transport);
        incoming.lock().unwrap().clear();
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
    }
}

fn grpc_frame(message: impl Message) -> Vec<u8> {
    let payload = message.encode_to_vec();
    let mut bytes = vec![0];
    bytes.extend(u32::try_from(payload.len()).unwrap().to_be_bytes());
    bytes.extend(payload);
    bytes
}
fn acknowledged_query() -> Vec<u8> {
    let mut bytes = grpc_frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::Begin(rpc::QueryBegin {
            query_id: Some(proto::Uuid::from_uuid(uuid::Uuid::new_v4())),
            statement_count: 1,
        })),
    });
    bytes.extend(grpc_frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::Batch(rpc::QueryBatchFrame {
            query_index: 0,
            kind: rpc::QueryResponseKind::Single as i32,
            payload: Some(rpc::query_batch_frame::Payload::Values(rpc::ValueBatch {
                values: vec![proto::Value {
                    value: Some(proto::value::Value::Bool(true)),
                }],
            })),
            ..Default::default()
        })),
    }));
    bytes.extend(grpc_frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::End(rpc::QueryEnd {
            result_count: 1,
            ..Default::default()
        })),
    }));
    bytes
}
#[tokio::test]
async fn early_query_ack_keeps_physical_admission_bounded_and_original_deadline_releases_uploads() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let incoming = Arc::new(Mutex::new(Vec::new()));
    let held = incoming.clone();
    let queries = Arc::new(AtomicUsize::new(0));
    let received = queries.clone();
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let service =
            hyper::service::service_fn(move |request: http::Request<hyper::body::Incoming>| {
                let held = held.clone();
                let received = received.clone();
                async move {
                    let method = request.uri().path().rsplit('/').next().unwrap().to_owned();
                    let bytes = match method.as_str() {
                        "Query" => {
                            held.lock().unwrap().push(request.into_body());
                            received.fetch_add(1, Ordering::AcqRel);
                            acknowledged_query()
                        }
                        "DetachSession" => {
                            held.lock().unwrap().push(request.into_body());
                            grpc_frame(rpc::DetachSessionResponse {})
                        }
                        method => {
                            request.into_body().collect().await.unwrap();
                            match method {
                                "GetCapabilities" => grpc_frame(rpc::GetCapabilitiesResponse {
                                    capabilities: Some(rpc::ServerCapabilities {
                                        server_version: "3.3.4".into(),
                                        ..Default::default()
                                    }),
                                }),
                                "AttachSession" => grpc_frame(rpc::AttachSessionResponse {
                                    session: Some(proto::Uuid::from_uuid(uuid::Uuid::new_v4())),
                                    ..Default::default()
                                }),
                                _ => panic!("unexpected fixture RPC"),
                            }
                        }
                    };
                    let mut trailers = http::HeaderMap::new();
                    trailers.insert("grpc-status", "0".parse().unwrap());
                    let body = StreamBody::new(futures_util::stream::iter([
                        Ok::<_, io::Error>(Frame::data(Bytes::from(bytes))),
                        Ok(Frame::trailers(trailers)),
                    ]));
                    Ok::<_, std::convert::Infallible>(
                        http::Response::builder()
                            .header("content-type", "application/grpc")
                            .body(body)
                            .unwrap(),
                    )
                }
            });
        let _ = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
            .initial_stream_window_size(1024)
            .initial_connection_window_size(64 << 10)
            .serve_connection(TokioIo::new(socket), service)
            .await;
    });
    let client = Client::connect(address, 1 << 20).await.unwrap();
    let payload = 256 * 1024;
    let metadata = crate::canonical::MetadataResponse::Protection.allocation_bytes();
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(32 * (payload * 4 + metadata)));
    let deadline = Instant::now() + Duration::from_secs(3);
    let query = format!("RETURN '{}'", "x".repeat(payload));
    let mut historical = Vec::new();
    for _ in 0..32 {
        let context = owned_context(&pool, payload * 4, deadline);
        let mut acknowledged = client
            .query(query.clone())
            .request_context(context.clone())
            .await
            .unwrap();
        assert_eq!(acknowledged.take::<Option<bool>>(0).unwrap(), Some(true));
        historical.push(context);
    }
    assert_eq!(queries.load(Ordering::Acquire), 32);
    assert!(
        historical
            .iter()
            .all(|context| !context.outbound_is_drained()),
        "every acknowledged upload must still have physical work beyond the peer's receive window"
    );
    assert_eq!(
        client.state.application.available_permits(),
        0,
        "all application admission remains owned by the 32 actual uploads"
    );
    let queued = RequestContext::new(Instant::now() + Duration::from_millis(100));
    assert!(
        client
            .query(query)
            .request_context(queued.clone())
            .await
            .is_err()
    );
    assert_eq!(queued.dispatch_state(), DispatchState::NotDispatched);
    assert_eq!(
        queries.load(Ordering::Acquire),
        32,
        "early ACK cannot free another physical request slot"
    );
    assert!(pool.reserved() > 0);
    tokio::time::timeout(Duration::from_secs(4), async {
        while pool.reserved() != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(
        historical
            .iter()
            .all(|context| context.dispatch_state() == DispatchState::Completed),
        "known acknowledgements stay known when the later physical deadline closes uploads"
    );
    client.disconnect().await.unwrap();
    incoming.lock().unwrap().clear();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}
