// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Focused owned-transport context/lifetime controls. Real server and module parentage is
//! deliberately established by the integration journey, not these controls.
use super::*;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::{SdkTracerProvider, SpanData, SpanExporter};
use tracing_subscriber::prelude::*;

#[derive(Clone, Debug, Default)]
struct Recorded(Arc<Mutex<Vec<SpanData>>>);
impl SpanExporter for Recorded {
    async fn export(&self, batch: Vec<SpanData>) -> opentelemetry_sdk::error::OTelSdkResult {
        self.0.lock().unwrap().extend(batch);
        Ok(())
    }
}
fn subscriber() -> (SdkTracerProvider, tracing::Dispatch, Recorded) {
    let recorded = Recorded::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(recorded.clone())
        .build();
    let subscriber = tracing_subscriber::registry().with(
        tracing_opentelemetry::layer().with_tracer(provider.tracer("canonical-transport-control")),
    );
    (provider, tracing::Dispatch::new(subscriber), recorded)
}
fn parent(trace: &str, span: &str, sampled: &str) -> opentelemetry::Context {
    let fields = std::collections::HashMap::from([
        ("traceparent".into(), format!("00-{trace}-{span}-{sampled}")),
        ("tracestate".into(), "vendor=independent".into()),
    ]);
    TraceContextPropagator::new().extract_with_context(&opentelemetry::Context::new(), &fields)
}
fn attribute(span: &SpanData, key: &str) -> Option<String> {
    span.attributes
        .iter()
        .find(|value| value.key.as_str() == key)
        .map(|value| value.value.to_string())
}

#[cfg_attr(test, tokio::test)]
async fn spawned_siblings_keep_independent_parent_and_wire_carrier() {
    let (provider, dispatch, recorded) = subscriber();
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut tasks = Vec::new();
    for (id, ancestor) in [
        ("11111111111111111111111111111111", "aaaaaaaaaaaaaaaa"),
        ("22222222222222222222222222222222", "bbbbbbbbbbbbbbbb"),
    ] {
        let barrier = barrier.clone();
        let dispatch = dispatch.clone();
        tasks.push(tokio::spawn(async move {
            let parent = parent(id, ancestor, "01");
            async move {
                let bounded = RequestTrace::new("bounded");
                tokio::spawn(bounded.future(async move {
                    barrier.wait().await;
                    tokio::task::yield_now().await;
                    let rpc = RequestTrace::new("grpc");
                    let expected = rpc.context.span().span_context().clone();
                    let mut headers = http::HeaderMap::new();
                    rpc.inject(&mut headers);
                    assert_eq!(
                        headers["traceparent"],
                        format!("00-{id}-{}-01", expected.span_id())
                    );
                    assert_eq!(headers["tracestate"], "vendor=independent");
                    assert!(!headers.contains_key("baggage"));
                    (id, ancestor, expected.span_id())
                }))
                .await
                .unwrap()
            }
            .with_context(parent)
            .with_subscriber(dispatch)
            .await
        }));
    }
    for task in tasks {
        let (id, ancestor, rpc_id) = task.await.unwrap();
        provider.force_flush().unwrap();
        let spans = recorded.0.lock().unwrap();
        let bounded = spans
            .iter()
            .find(|span| {
                span.span_context.trace_id().to_string() == id
                    && attribute(span, "operation").as_deref() == Some("bounded")
            })
            .unwrap();
        let rpc = spans
            .iter()
            .find(|span| span.span_context.span_id() == rpc_id)
            .unwrap();
        assert_eq!(bounded.parent_span_id.to_string(), ancestor);
        assert!(bounded.parent_span_is_remote);
        assert_eq!(rpc.parent_span_id, bounded.span_context.span_id());
        assert!(!rpc.parent_span_is_remote);
    }
    provider.shutdown().unwrap();
}

#[cfg_attr(test, tokio::test)]
async fn authentic_tracing_parent_precedes_unrelated_ambient_api_context() {
    let (provider, dispatch, recorded) = subscriber();
    let span = tracing::dispatcher::with_default(&dispatch, || {
        let span = tracing::info_span!(parent: None, "caller");
        span.set_parent(parent(
            "33333333333333333333333333333333",
            "cccccccccccccccc",
            "01",
        ))
        .unwrap();
        span
    });
    let caller = span.context().span().span_context().clone();
    let expected_caller = caller.clone();
    async move {
        tokio::spawn(RequestTrace::new("bounded").future(async move {
            let rpc = RequestTrace::new("grpc");
            assert_eq!(
                rpc.context.span().span_context().trace_id(),
                expected_caller.trace_id()
            );
        }))
        .await
        .unwrap();
    }
    .instrument(span)
    .with_context(parent(
        "44444444444444444444444444444444",
        "dddddddddddddddd",
        "01",
    ))
    .with_subscriber(dispatch)
    .await;
    provider.force_flush().unwrap();
    let spans = recorded.0.lock().unwrap();
    let bounded = spans
        .iter()
        .find(|span| attribute(span, "operation").as_deref() == Some("bounded"))
        .unwrap();
    assert_eq!(bounded.parent_span_id, caller.span_id());
    drop(spans);
    provider.shutdown().unwrap();
}

#[cfg_attr(test, tokio::test)]
async fn missing_context_removes_stale_carrier_without_inventing_parent() {
    async {
        let trace = RequestTrace::new("grpc");
        let mut headers = http::HeaderMap::new();
        headers.insert("traceparent", "stale".parse().unwrap());
        headers.insert("tracestate", "vendor=stale".parse().unwrap());
        trace.inject(&mut headers);
        assert!(!headers.contains_key("traceparent"));
        assert!(!headers.contains_key("tracestate"));
    }
    .with_context(opentelemetry::Context::new())
    .with_subscriber(tracing::Dispatch::new(
        tracing::subscriber::NoSubscriber::default(),
    ))
    .await;
}

#[cfg_attr(test, tokio::test)]
async fn api_only_unsampled_context_preserves_flags_and_state() {
    async {
        let trace = RequestTrace::new("grpc");
        let mut headers = http::HeaderMap::new();
        trace.inject(&mut headers);
        assert_eq!(
            headers["traceparent"],
            "00-77777777777777777777777777777777-1212121212121212-00"
        );
        assert_eq!(headers["tracestate"], "vendor=independent");
    }
    .with_context(parent(
        "77777777777777777777777777777777",
        "1212121212121212",
        "00",
    ))
    .with_subscriber(tracing::Dispatch::new(
        tracing::subscriber::NoSubscriber::default(),
    ))
    .await;
}

struct ProbeBody {
    frames: tokio::sync::mpsc::UnboundedReceiver<Result<Frame<Bytes>, std::io::Error>>,
    observed: Arc<Mutex<Vec<opentelemetry::trace::SpanContext>>>,
}
impl Body for ProbeBody {
    type Data = Bytes;
    type Error = std::io::Error;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        self.observed.lock().unwrap().push(
            opentelemetry::Context::current()
                .span()
                .span_context()
                .clone(),
        );
        self.frames.poll_recv(cx)
    }
}
impl Drop for ProbeBody {
    fn drop(&mut self) {
        self.observed.lock().unwrap().push(
            opentelemetry::Context::current()
                .span()
                .span_context()
                .clone(),
        );
    }
}

#[cfg_attr(test, tokio::test)]
async fn response_body_retains_span_and_installs_its_context_on_poll_error_and_drop() {
    for terminal in ["complete", "error", "dropped"] {
        let (provider, dispatch, recorded) = subscriber();
        let observed = Arc::new(Mutex::new(Vec::new()));
        let (send, receive) = tokio::sync::mpsc::unbounded_channel();
        let (mut body, expected) = async {
            let trace = RequestTrace::new("grpc");
            let expected = trace.context.span().span_context().clone();
            (
                ObservedBody::new(
                    ProbeBody {
                        frames: receive,
                        observed: observed.clone(),
                    },
                    trace,
                ),
                expected,
            )
        }
        .with_context(parent(
            "55555555555555555555555555555555",
            "eeeeeeeeeeeeeeee",
            "01",
        ))
        .with_subscriber(dispatch)
        .await;
        // Headers have returned, but the actual body still owns the RPC span.
        provider.force_flush().unwrap();
        assert!(recorded.0.lock().unwrap().is_empty());
        let mut cx = Context::from_waker(futures_util::task::noop_waker_ref());
        assert!(Pin::new(&mut body).poll_frame(&mut cx).is_pending());
        send.send(Ok(Frame::data(Bytes::from_static(b"protobuf"))))
            .unwrap();
        assert!(matches!(
            Pin::new(&mut body).poll_frame(&mut cx),
            Poll::Ready(Some(Ok(_)))
        ));
        match terminal {
            "complete" => {
                send.send(Ok(Frame::trailers(http::HeaderMap::new())))
                    .unwrap();
            }
            "error" => {
                send.send(Err(std::io::Error::other("stream failure")))
                    .unwrap();
            }
            _ => {}
        }
        if terminal != "dropped" {
            let _ = Pin::new(&mut body).poll_frame(&mut cx);
        }
        drop(body);
        provider.force_flush().unwrap();
        let spans = recorded.0.lock().unwrap();
        assert_eq!(spans.len(), 1);
        assert_eq!(attribute(&spans[0], "response").as_deref(), Some(terminal));
        assert!(
            observed
                .lock()
                .unwrap()
                .iter()
                .all(|context| context.span_id() == expected.span_id()
                    && context.trace_id() == expected.trace_id())
        );
        assert!(
            !opentelemetry::Context::current()
                .span()
                .span_context()
                .is_valid()
        );
        drop(spans);
        provider.shutdown().unwrap();
    }
}

#[cfg_attr(test, tokio::test)]
async fn query_owner_outlives_transport_until_decode_finishes() {
    let (provider, dispatch, recorded) = subscriber();
    let (started, ready) = tokio::sync::oneshot::channel();
    let (finish, decoded) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(
        async move {
            observe("query", async move {
                let transport = RequestTrace::new("grpc");
                drop(transport);
                started.send(()).unwrap();
                decoded.await.unwrap();
            })
            .await
        }
        .with_context(parent(
            "66666666666666666666666666666666",
            "ffffffffffffffff",
            "01",
        ))
        .with_subscriber(dispatch),
    );
    ready.await.unwrap();
    provider.force_flush().unwrap();
    assert_eq!(recorded.0.lock().unwrap().len(), 1);
    assert_eq!(
        attribute(&recorded.0.lock().unwrap()[0], "operation").as_deref(),
        Some("grpc")
    );
    finish.send(()).unwrap();
    task.await.unwrap();
    provider.force_flush().unwrap();
    let spans = recorded.0.lock().unwrap();
    let query = spans
        .iter()
        .find(|span| attribute(span, "operation").as_deref() == Some("query"))
        .unwrap();
    let transport = spans
        .iter()
        .find(|span| attribute(span, "operation").as_deref() == Some("grpc"))
        .unwrap();
    assert_eq!(transport.parent_span_id, query.span_context.span_id());
    drop(spans);
    provider.shutdown().unwrap();
}

async fn real_http2_route(terminal: &'static str) {
    let (provider, dispatch, recorded) = subscriber();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (requests, mut received) = tokio::sync::mpsc::unbounded_channel();
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let service =
            hyper::service::service_fn(move |request: http::Request<hyper::body::Incoming>| {
                let (send, frames) = tokio::sync::mpsc::unbounded_channel();
                requests.send((request.headers().clone(), send)).unwrap();
                async move {
                    Ok::<_, std::convert::Infallible>(
                        http::Response::builder()
                            .header("content-type", "application/grpc")
                            .body(ProbeBody {
                                frames,
                                observed: Arc::default(),
                            })
                            .unwrap(),
                    )
                }
            });
        let _ = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
            .serve_connection(TokioIo::new(socket), service)
            .await;
    });
    let url = url::Url::parse(&format!("grpc://{address}")).unwrap();
    let driver = Driver::new();
    let transport = OwnedHttp2::connect(&url, 1024, driver.clone())
        .await
        .unwrap();
    let mut responses = Vec::new();
    for (id, ancestor) in [
        ("88888888888888888888888888888888", "1313131313131313"),
        ("99999999999999999999999999999999", "1414141414141414"),
    ] {
        let mut transport = transport.clone();
        let dispatch = dispatch.clone();
        responses.push(tokio::spawn(async move {
            let request_context =
                RequestContext::new(std::time::Instant::now() + std::time::Duration::from_secs(10));
            let response = REQUEST
                .scope(request_context.clone(), async {
                    futures_util::future::poll_fn(|cx| transport.poll_ready(cx))
                        .await
                        .unwrap();
                    let request = http::Request::builder()
                        .uri(format!(
                            "http://{address}/surrealdb.rpc.v1.SurrealDBService/Query"
                        ))
                        .header("content-type", "application/grpc")
                        .body(tonic::body::Body::empty())
                        .unwrap();
                    transport.call(request).await.unwrap()
                })
                .with_context(parent(id, ancestor, "01"))
                .with_subscriber(dispatch)
                .await;
            (id, ancestor, response.into_body(), request_context)
        }));
    }
    let mut on_wire = std::collections::HashMap::new();
    for _ in 0..2 {
        let (headers, frames) =
            tokio::time::timeout(std::time::Duration::from_secs(5), received.recv())
                .await
                .unwrap()
                .unwrap();
        let carrier = headers["traceparent"].to_str().unwrap();
        let id = carrier[3..35].to_owned();
        let span = carrier[36..52].to_owned();
        assert_eq!(headers["tracestate"], "vendor=independent");
        assert_eq!(&carrier[53..], "01");
        assert!(on_wire.insert(id, (span, frames)).is_none());
    }
    for response in responses {
        let (id, ancestor, mut body, context) =
            tokio::time::timeout(std::time::Duration::from_secs(5), response)
                .await
                .unwrap()
                .unwrap();
        let (span_id, frames) = on_wire.remove(id).unwrap();
        provider.force_flush().unwrap();
        assert!(
            !recorded
                .0
                .lock()
                .unwrap()
                .iter()
                .any(|span| span.span_context.span_id().to_string() == span_id)
        );
        // The owned route validates this complete empty protobuf message before
        // tonic can decode. The peer deliberately retains trailers for later.
        frames
            .send(Ok(Frame::data(Bytes::from_static(b"\0\0\0\0\0"))))
            .unwrap();
        let first = futures_util::future::poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await;
        assert!(matches!(first, Some(Ok(_))));
        assert_eq!(
            context.dispatch_state(),
            super::super::DispatchState::Dispatched
        );
        match terminal {
            "complete" => {
                let mut trailers = http::HeaderMap::new();
                trailers.insert("grpc-status", "0".parse().unwrap());
                frames.send(Ok(Frame::trailers(trailers))).unwrap();
                while let Some(frame) =
                    futures_util::future::poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await
                {
                    frame.unwrap();
                }
            }
            "error" => {
                frames
                    .send(Err(std::io::Error::other("peer stream failure")))
                    .unwrap();
                assert!(
                    futures_util::future::poll_fn(|cx| Pin::new(&mut body).poll_frame(cx))
                        .await
                        .unwrap()
                        .is_err()
                );
            }
            "dropped" => {
                context.cancel();
            }
            other => panic!("unexpected response scenario: {other}"),
        }
        drop(body);
        // Neither headers nor telemetry completion/cancellation can settle the
        // native response: only the canonical client's completed decode owner can do so.
        assert_eq!(
            context.dispatch_state(),
            super::super::DispatchState::Dispatched
        );
        provider.force_flush().unwrap();
        let spans = recorded.0.lock().unwrap();
        let rpc = spans
            .iter()
            .find(|span| span.span_context.span_id().to_string() == span_id)
            .unwrap();
        assert_eq!(rpc.span_context.trace_id().to_string(), id);
        assert_eq!(rpc.parent_span_id.to_string(), ancestor);
        assert_eq!(attribute(rpc, "response").as_deref(), Some(terminal));
    }
    driver.shutdown().await;
    assert!(driver.drained());
    drop(transport);
    server.abort();
    let _ = server.await;
    provider.shutdown().unwrap();
}

#[cfg_attr(test, tokio::test)]
async fn actual_owned_http2_headers_stream_error_cancel_and_siblings() {
    for terminal in ["complete", "error", "dropped"] {
        real_http2_route(terminal).await;
    }
}
