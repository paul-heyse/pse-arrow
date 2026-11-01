///Shorthand for `OwnedView<RequestView<'static>>`.
pub type OwnedRequestView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::RequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<ObservationView<'static>>`.
pub type OwnedObservationView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::ObservationView<
        'static,
    >,
>;
///Shorthand for `OwnedView<ControlView<'static>>`.
pub type OwnedControlView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::ControlView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EventView<'static>>`.
pub type OwnedEventView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::EventView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EmptyView<'static>>`.
pub type OwnedEmptyView = ::buffa::view::OwnedView<
    ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
>;
///Shorthand for `OwnedView<NativeSessionStatusView<'static>>`.
pub type OwnedNativeSessionStatusView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeSessionStatusView<
        'static,
    >,
>;
///Shorthand for `OwnedView<NativePackageRequestView<'static>>`.
pub type OwnedNativePackageRequestView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativePackageRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<NativePackageReplyView<'static>>`.
pub type OwnedNativePackageReplyView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativePackageReplyView<
        'static,
    >,
>;
///Shorthand for `OwnedView<NativeControlView<'static>>`.
pub type OwnedNativeControlView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeControlView<
        'static,
    >,
>;
///Shorthand for `OwnedView<NativeEventView<'static>>`.
pub type OwnedNativeEventView = ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeEventView<
        'static,
    >,
>;
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Observation,
>
for crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::ObservationView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Observation,
>
for ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::ObservationView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
    /// An `OwnedView` still holds the buffer it was decoded from, so
    /// its large fields can be handed to the response body by
    /// reference count instead of copied. The bare view impl above
    /// cannot do this: it has borrows but no buffer to name.
    fn encode_segments(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::connectrpc::EncodedBody, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body_segments(
            self.reborrow(),
            self.bytes(),
            codec,
        )
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Event,
>
for crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::EventView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Event,
>
for ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::EventView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
    /// An `OwnedView` still holds the buffer it was decoded from, so
    /// its large fields can be handed to the response body by
    /// reference count instead of copied. The bare view impl above
    /// cannot do this: it has borrows but no buffer to name.
    fn encode_segments(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::connectrpc::EncodedBody, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body_segments(
            self.reborrow(),
            self.bytes(),
            codec,
        )
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus,
>
for crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeSessionStatusView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus,
>
for ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeSessionStatusView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
    /// An `OwnedView` still holds the buffer it was decoded from, so
    /// its large fields can be handed to the response body by
    /// reference count instead of copied. The bare view impl above
    /// cannot do this: it has borrows but no buffer to name.
    fn encode_segments(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::connectrpc::EncodedBody, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body_segments(
            self.reborrow(),
            self.bytes(),
            codec,
        )
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageReply,
>
for crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativePackageReplyView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageReply,
>
for ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativePackageReplyView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
    /// An `OwnedView` still holds the buffer it was decoded from, so
    /// its large fields can be handed to the response body by
    /// reference count instead of copied. The bare view impl above
    /// cannot do this: it has borrows but no buffer to name.
    fn encode_segments(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::connectrpc::EncodedBody, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body_segments(
            self.reborrow(),
            self.bytes(),
            codec,
        )
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeEvent,
>
for crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeEventView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeEvent,
>
for ::buffa::view::OwnedView<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeEventView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
    /// An `OwnedView` still holds the buffer it was decoded from, so
    /// its large fields can be handed to the response body by
    /// reference count instead of copied. The bare view impl above
    /// cannot do this: it has borrows but no buffer to name.
    fn encode_segments(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::connectrpc::EncodedBody, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body_segments(
            self.reborrow(),
            self.bytes(),
            codec,
        )
    }
}
/// Full service name for this service.
pub const INVESTIGATION_SERVICE_NAME: &str = "pse.code_intelligence.v1.Investigation";
/// Static [`Spec`](::connectrpc::Spec) for the `Inspect` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const INVESTIGATION_INSPECT_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.code_intelligence.v1.Investigation/Inspect",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `Run` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const INVESTIGATION_RUN_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.code_intelligence.v1.Investigation/Run",
        ::connectrpc::StreamType::BidiStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for Investigation.
///
/// # Implementing handlers
///
/// Implement methods with plain `async fn`; the returned future satisfies
/// the `Send` bound automatically.
///
/// **Unary and server-streaming requests** arrive as
/// [`ServiceRequest<'_, Req>`](::connectrpc::ServiceRequest): a zero-copy
/// view of the request plus its body, valid for the duration of the call.
/// Fields are read directly (`request.name` is a `&str` into the decoded
/// buffer) and the borrow may be held across `.await` points. Anything
/// that must outlive the call — `tokio::spawn`, channels, server state,
/// or data captured by a returned response stream — takes owned data:
/// call `request.to_owned_message()` (or copy the specific fields)
/// first.
///
/// **Client-streaming and bidi requests** arrive as
/// [`InboundStream<Req>`](::connectrpc::InboundStream) — a
/// `ServiceStream` of [`StreamMessage`](::connectrpc::StreamMessage)s.
/// Each item owns its decoded buffer and is `Send + 'static`, so items
/// can be buffered or moved into spawned tasks; read fields zero-copy
/// through the generated accessor methods (`item.name()`) or `.view()`,
/// convert with `.to_owned_message()`, or yield an item back unchanged —
/// `StreamMessage<M>` implements `Encodable<M>`.
///
/// Request types resolved through `extern_path` (e.g. well-known types
/// from another crate) use the same wrappers; the crate that owns the
/// type must be generated with buffa ≥ 0.9.0 and views enabled so the
/// backing `HasMessageView` impl exists.
///
/// The `impl Encodable<Out>` return bound accepts the owned `Out`, the
/// generated `OutView<'_>` / `OwnedOutView`,
/// [`MaybeBorrowed`](::connectrpc::MaybeBorrowed), or
/// [`PreEncoded`](::connectrpc::PreEncoded) for handlers that encode a
/// non-`'static` view internally and pass the bytes across the handler
/// boundary. View bodies are not emitted for output types mapped via
/// `extern_path` (the impl would be an orphan); return owned for
/// WKT/extern outputs.
///
/// Server-streaming and bidi-streaming methods return
/// `ServiceStream<impl Encodable<Out> + Send + use<Self>>`. The
/// `use<Self>` precise-capturing clause excludes `&self`'s lifetime and
/// the request's lifetime (unary methods use `use<'a, Self>` and may
/// borrow from `&self`), so stream items must be `'static` and cannot
/// borrow from the request. To stream view-encoded data, encode each
/// item inside the stream body and yield
/// [`PreEncoded`](::connectrpc::PreEncoded) — see its `# Streaming
/// example` doc.
#[allow(clippy::type_complexity, reason = "Pinned protobuf generator emits portable schema spelling and implementation scaffolding")]
pub trait Investigation: Send + Sync + 'static {
    /// Handle the Inspect RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn inspect<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Request,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Observation,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the Run RPC.
    ///
    /// Each `requests` item is a [`StreamMessage`](::connectrpc::StreamMessage):
    /// it owns its buffer, is `Send + 'static`, and exposes zero-copy
    /// accessor methods (`item.name()`), `.view()`, and
    /// `.to_owned_message()`.
    fn run(
        &self,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::InboundStream<
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Control,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Event,
                > + Send + use<Self>,
            >,
        >,
    > + Send;
}
/// Extension trait for registering a service implementation with a Router.
///
/// This trait is automatically implemented for all types that implement the service trait.
/// Prefer [`Router::add_service`](::connectrpc::Router::add_service) for
/// top-down registration; `register` remains available for compatibility
/// and cases where the service-first call shape is more convenient.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// let service = Arc::new(MyServiceImpl);
/// let router = service.register(Router::new());
/// ```
pub trait InvestigationExt: Investigation {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: Investigation> InvestigationExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                INVESTIGATION_SERVICE_NAME,
                "Inspect",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::RequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Request,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.inspect(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Observation,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(INVESTIGATION_INSPECT_SPEC)
            .route_view_bidi_stream::<
                _,
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Event,
            >(
                INVESTIGATION_SERVICE_NAME,
                "Run",
                ::connectrpc::view_bidi_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |ctx, req| {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let req = ::connectrpc::dispatcher::codegen::into_stream_messages::<
                                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Control,
                            >(req);
                            svc.run(ctx, req).await
                        }
                    }
                }),
            )
            .with_spec(INVESTIGATION_RUN_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct InvestigationRegisterMarker;
impl<S: Investigation> ::connectrpc::ServiceRegister<InvestigationRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as InvestigationExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `Investigation`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = InvestigationServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct InvestigationServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: Investigation> InvestigationServer<T> {
    /// Wrap a service implementation in a monomorphic dispatcher.
    pub fn new(service: T) -> Self {
        Self {
            inner: ::std::sync::Arc::new(service),
        }
    }
    /// Wrap an already-`Arc`'d service implementation.
    pub fn from_arc(inner: ::std::sync::Arc<T>) -> Self {
        Self { inner }
    }
}
impl<T> Clone for InvestigationServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: Investigation> ::connectrpc::Dispatcher for InvestigationServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("pse.code_intelligence.v1.Investigation/")?;
        match method {
            "Inspect" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INVESTIGATION_INSPECT_SPEC),
                )
            }
            "Run" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::bidi_streaming()
                        .with_spec(INVESTIGATION_RUN_SPEC),
                )
            }
            _ => None,
        }
    }
    fn call_unary(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::Payload,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("pse.code_intelligence.v1.Investigation/")
        else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Inspect" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Request,
                    >(request.encoded()?, format)?;
                    let req: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::RequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Request,
                    >::from_parts(&req, &body);
                    svc.inspect(ctx, req)
                        .await?
                        .encode::<
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Observation,
                        >(format)
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_server_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::buffa::bytes::Bytes,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("pse.code_intelligence.v1.Investigation/")
        else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
    fn call_client_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("pse.code_intelligence.v1.Investigation/")
        else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_bidi_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("pse.code_intelligence.v1.Investigation/")
        else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            "Run" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let req_stream = ::connectrpc::dispatcher::codegen::decode_message_request_stream::<
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Control,
                    >(requests, format, ctx.decode_options().clone());
                    let resp = svc.run(ctx, req_stream).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Event,
                                _,
                                _,
                            >(s, format)),
                    )
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
}
/// Client for this service.
///
/// Generic over `T: ClientTransport`. For **gRPC** (HTTP/2), use
/// `Http2Connection` — it has honest `poll_ready` and composes with
/// `tower::balance` for multi-connection load balancing. For **Connect
/// over HTTP/1.1** (or unknown protocol), use `HttpClient`.
///
/// # Example (gRPC / HTTP/2)
///
/// ```rust,ignore
/// use connectrpc::client::{Http2Connection, ClientConfig};
/// use connectrpc::Protocol;
///
/// let uri: http::Uri = "http://localhost:8080".parse()?;
/// let conn = Http2Connection::connect_plaintext(uri.clone()).await?.shared(1024);
/// let config = ClientConfig::new(uri).with_protocol(Protocol::Grpc);
///
/// let client = InvestigationClient::new(conn, config);
/// let response = client.inspect(request).await?;
/// ```
///
/// # Example (Connect / HTTP/1.1 or ALPN)
///
/// ```rust,ignore
/// use connectrpc::client::{HttpClient, ClientConfig};
///
/// let http = HttpClient::plaintext();  // cleartext http:// only
/// let config = ClientConfig::new("http://localhost:8080".parse()?);
///
/// let client = InvestigationClient::new(http, config);
/// let response = client.inspect(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.inspect(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.inspect(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one
/// [`StreamMessage`](::connectrpc::StreamMessage) per received message from
/// `.message().await` — read fields zero-copy through the generated accessor
/// methods (`msg.name()`) or `.view()`, or convert with `.to_owned_message()`.
#[derive(Clone)]
pub struct InvestigationClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
impl<T> InvestigationClient<T>
where
    T: ::connectrpc::client::ClientTransport,
    <T::ResponseBody as ::connectrpc::http_body::Body>::Error: ::std::fmt::Display,
{
    /// Create a new client with the given transport and configuration.
    pub fn new(transport: T, config: ::connectrpc::client::ClientConfig) -> Self {
        Self { transport, config }
    }
    /// Get the client configuration.
    pub fn config(&self) -> &::connectrpc::client::ClientConfig {
        &self.config
    }
    /// Get a mutable reference to the client configuration.
    pub fn config_mut(&mut self) -> &mut ::connectrpc::client::ClientConfig {
        &mut self.config
    }
    /// Call the Inspect RPC. Sends a request to /pse.code_intelligence.v1.Investigation/Inspect.
    pub async fn inspect(
        &self,
        request: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Request,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::ObservationView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.inspect_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Inspect RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn inspect_with_options(
        &self,
        request: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Request,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::ObservationView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INVESTIGATION_INSPECT_SPEC.with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the Run RPC. Sends a request to /pse.code_intelligence.v1.Investigation/Run.
    pub async fn run(
        &self,
    ) -> Result<
        ::connectrpc::client::BidiStream<
            T::ResponseBody,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Control,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::EventView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.run_with_options(::connectrpc::client::CallOptions::default()).await
    }
    /// Call the Run RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn run_with_options(
        &self,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::BidiStream<
            T::ResponseBody,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::Control,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::EventView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_bidi_stream(
                &self.transport,
                &self.config,
                INVESTIGATION_RUN_SPEC.with_origin(::connectrpc::SpecOrigin::Client),
                options,
            )
            .await
    }
}
/// Full service name for this service.
pub const NATIVE_SEMANTIC_SERVICE_NAME: &str = "pse.code_intelligence.v1.NativeSemantic";
/// Static [`Spec`](::connectrpc::Spec) for the `Status` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const NATIVE_SEMANTIC_STATUS_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.code_intelligence.v1.NativeSemantic/Status",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `PackageContext` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const NATIVE_SEMANTIC_PACKAGE_CONTEXT_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.code_intelligence.v1.NativeSemantic/PackageContext",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `Capabilities` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const NATIVE_SEMANTIC_CAPABILITIES_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.code_intelligence.v1.NativeSemantic/Capabilities",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `Stop` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const NATIVE_SEMANTIC_STOP_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.code_intelligence.v1.NativeSemantic/Stop",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `Run` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const NATIVE_SEMANTIC_RUN_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.code_intelligence.v1.NativeSemantic/Run",
        ::connectrpc::StreamType::BidiStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for NativeSemantic.
///
/// # Implementing handlers
///
/// Implement methods with plain `async fn`; the returned future satisfies
/// the `Send` bound automatically.
///
/// **Unary and server-streaming requests** arrive as
/// [`ServiceRequest<'_, Req>`](::connectrpc::ServiceRequest): a zero-copy
/// view of the request plus its body, valid for the duration of the call.
/// Fields are read directly (`request.name` is a `&str` into the decoded
/// buffer) and the borrow may be held across `.await` points. Anything
/// that must outlive the call — `tokio::spawn`, channels, server state,
/// or data captured by a returned response stream — takes owned data:
/// call `request.to_owned_message()` (or copy the specific fields)
/// first.
///
/// **Client-streaming and bidi requests** arrive as
/// [`InboundStream<Req>`](::connectrpc::InboundStream) — a
/// `ServiceStream` of [`StreamMessage`](::connectrpc::StreamMessage)s.
/// Each item owns its decoded buffer and is `Send + 'static`, so items
/// can be buffered or moved into spawned tasks; read fields zero-copy
/// through the generated accessor methods (`item.name()`) or `.view()`,
/// convert with `.to_owned_message()`, or yield an item back unchanged —
/// `StreamMessage<M>` implements `Encodable<M>`.
///
/// Request types resolved through `extern_path` (e.g. well-known types
/// from another crate) use the same wrappers; the crate that owns the
/// type must be generated with buffa ≥ 0.9.0 and views enabled so the
/// backing `HasMessageView` impl exists.
///
/// The `impl Encodable<Out>` return bound accepts the owned `Out`, the
/// generated `OutView<'_>` / `OwnedOutView`,
/// [`MaybeBorrowed`](::connectrpc::MaybeBorrowed), or
/// [`PreEncoded`](::connectrpc::PreEncoded) for handlers that encode a
/// non-`'static` view internally and pass the bytes across the handler
/// boundary. View bodies are not emitted for output types mapped via
/// `extern_path` (the impl would be an orphan); return owned for
/// WKT/extern outputs.
///
/// Server-streaming and bidi-streaming methods return
/// `ServiceStream<impl Encodable<Out> + Send + use<Self>>`. The
/// `use<Self>` precise-capturing clause excludes `&self`'s lifetime and
/// the request's lifetime (unary methods use `use<'a, Self>` and may
/// borrow from `&self`), so stream items must be `'static` and cannot
/// borrow from the request. To stream view-encoded data, encode each
/// item inside the stream body and yield
/// [`PreEncoded`](::connectrpc::PreEncoded) — see its `# Streaming
/// example` doc.
#[allow(clippy::type_complexity, reason = "Pinned protobuf generator emits portable schema spelling and implementation scaffolding")]
pub trait NativeSemantic: Send + Sync + 'static {
    /// Handle the Status RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn status<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, ::buffa_types::google::protobuf::Empty>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the PackageContext RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn package_context<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageReply,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the Capabilities RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn capabilities<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, ::buffa_types::google::protobuf::Empty>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the Stop RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn stop<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, ::buffa_types::google::protobuf::Empty>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                ::buffa_types::google::protobuf::Empty,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the Run RPC.
    ///
    /// Each `requests` item is a [`StreamMessage`](::connectrpc::StreamMessage):
    /// it owns its buffer, is `Send + 'static`, and exposes zero-copy
    /// accessor methods (`item.name()`), `.view()`, and
    /// `.to_owned_message()`.
    fn run(
        &self,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::InboundStream<
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeControl,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeEvent,
                > + Send + use<Self>,
            >,
        >,
    > + Send;
}
/// Extension trait for registering a service implementation with a Router.
///
/// This trait is automatically implemented for all types that implement the service trait.
/// Prefer [`Router::add_service`](::connectrpc::Router::add_service) for
/// top-down registration; `register` remains available for compatibility
/// and cases where the service-first call shape is more convenient.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// let service = Arc::new(MyServiceImpl);
/// let router = service.register(Router::new());
/// ```
pub trait NativeSemanticExt: NativeSemantic {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: NativeSemantic> NativeSemanticExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                NATIVE_SEMANTIC_SERVICE_NAME,
                "Status",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            ::buffa_types::google::protobuf::__buffa::view::EmptyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                ::buffa_types::google::protobuf::Empty,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.status(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(NATIVE_SEMANTIC_STATUS_SPEC)
            .route_view(
                NATIVE_SEMANTIC_SERVICE_NAME,
                "PackageContext",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativePackageRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.package_context(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageReply,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(NATIVE_SEMANTIC_PACKAGE_CONTEXT_SPEC)
            .route_view(
                NATIVE_SEMANTIC_SERVICE_NAME,
                "Capabilities",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            ::buffa_types::google::protobuf::__buffa::view::EmptyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                ::buffa_types::google::protobuf::Empty,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.capabilities(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(NATIVE_SEMANTIC_CAPABILITIES_SPEC)
            .route_view(
                NATIVE_SEMANTIC_SERVICE_NAME,
                "Stop",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            ::buffa_types::google::protobuf::__buffa::view::EmptyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                ::buffa_types::google::protobuf::Empty,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.stop(ctx, sreq)
                                .await?
                                .encode::<::buffa_types::google::protobuf::Empty>(format)
                        }
                    })
                },
            )
            .with_spec(NATIVE_SEMANTIC_STOP_SPEC)
            .route_view_bidi_stream::<
                _,
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeEvent,
            >(
                NATIVE_SEMANTIC_SERVICE_NAME,
                "Run",
                ::connectrpc::view_bidi_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |ctx, req| {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let req = ::connectrpc::dispatcher::codegen::into_stream_messages::<
                                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeControl,
                            >(req);
                            svc.run(ctx, req).await
                        }
                    }
                }),
            )
            .with_spec(NATIVE_SEMANTIC_RUN_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct NativeSemanticRegisterMarker;
impl<S: NativeSemantic> ::connectrpc::ServiceRegister<NativeSemanticRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as NativeSemanticExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `NativeSemantic`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = NativeSemanticServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct NativeSemanticServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: NativeSemantic> NativeSemanticServer<T> {
    /// Wrap a service implementation in a monomorphic dispatcher.
    pub fn new(service: T) -> Self {
        Self {
            inner: ::std::sync::Arc::new(service),
        }
    }
    /// Wrap an already-`Arc`'d service implementation.
    pub fn from_arc(inner: ::std::sync::Arc<T>) -> Self {
        Self { inner }
    }
}
impl<T> Clone for NativeSemanticServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: NativeSemantic> ::connectrpc::Dispatcher for NativeSemanticServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("pse.code_intelligence.v1.NativeSemantic/")?;
        match method {
            "Status" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(NATIVE_SEMANTIC_STATUS_SPEC),
                )
            }
            "PackageContext" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(NATIVE_SEMANTIC_PACKAGE_CONTEXT_SPEC),
                )
            }
            "Capabilities" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(NATIVE_SEMANTIC_CAPABILITIES_SPEC),
                )
            }
            "Stop" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(NATIVE_SEMANTIC_STOP_SPEC),
                )
            }
            "Run" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::bidi_streaming()
                        .with_spec(NATIVE_SEMANTIC_RUN_SPEC),
                )
            }
            _ => None,
        }
    }
    fn call_unary(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::Payload,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("pse.code_intelligence.v1.NativeSemantic/")
        else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Status" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        ::buffa_types::google::protobuf::Empty,
                    >(request.encoded()?, format)?;
                    let req: ::buffa_types::google::protobuf::__buffa::view::EmptyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        ::buffa_types::google::protobuf::Empty,
                    >::from_parts(&req, &body);
                    svc.status(ctx, req)
                        .await?
                        .encode::<
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus,
                        >(format)
                })
            }
            "PackageContext" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativePackageRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageRequest,
                    >::from_parts(&req, &body);
                    svc.package_context(ctx, req)
                        .await?
                        .encode::<
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageReply,
                        >(format)
                })
            }
            "Capabilities" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        ::buffa_types::google::protobuf::Empty,
                    >(request.encoded()?, format)?;
                    let req: ::buffa_types::google::protobuf::__buffa::view::EmptyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        ::buffa_types::google::protobuf::Empty,
                    >::from_parts(&req, &body);
                    svc.capabilities(ctx, req)
                        .await?
                        .encode::<
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeSessionStatus,
                        >(format)
                })
            }
            "Stop" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        ::buffa_types::google::protobuf::Empty,
                    >(request.encoded()?, format)?;
                    let req: ::buffa_types::google::protobuf::__buffa::view::EmptyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        ::buffa_types::google::protobuf::Empty,
                    >::from_parts(&req, &body);
                    svc.stop(ctx, req)
                        .await?
                        .encode::<::buffa_types::google::protobuf::Empty>(format)
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_server_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::buffa::bytes::Bytes,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("pse.code_intelligence.v1.NativeSemantic/")
        else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
    fn call_client_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("pse.code_intelligence.v1.NativeSemantic/")
        else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_bidi_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("pse.code_intelligence.v1.NativeSemantic/")
        else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            "Run" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let req_stream = ::connectrpc::dispatcher::codegen::decode_message_request_stream::<
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeControl,
                    >(requests, format, ctx.decode_options().clone());
                    let resp = svc.run(ctx, req_stream).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeEvent,
                                _,
                                _,
                            >(s, format)),
                    )
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
}
/// Client for this service.
///
/// Generic over `T: ClientTransport`. For **gRPC** (HTTP/2), use
/// `Http2Connection` — it has honest `poll_ready` and composes with
/// `tower::balance` for multi-connection load balancing. For **Connect
/// over HTTP/1.1** (or unknown protocol), use `HttpClient`.
///
/// # Example (gRPC / HTTP/2)
///
/// ```rust,ignore
/// use connectrpc::client::{Http2Connection, ClientConfig};
/// use connectrpc::Protocol;
///
/// let uri: http::Uri = "http://localhost:8080".parse()?;
/// let conn = Http2Connection::connect_plaintext(uri.clone()).await?.shared(1024);
/// let config = ClientConfig::new(uri).with_protocol(Protocol::Grpc);
///
/// let client = NativeSemanticClient::new(conn, config);
/// let response = client.status(request).await?;
/// ```
///
/// # Example (Connect / HTTP/1.1 or ALPN)
///
/// ```rust,ignore
/// use connectrpc::client::{HttpClient, ClientConfig};
///
/// let http = HttpClient::plaintext();  // cleartext http:// only
/// let config = ClientConfig::new("http://localhost:8080".parse()?);
///
/// let client = NativeSemanticClient::new(http, config);
/// let response = client.status(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.status(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.status(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one
/// [`StreamMessage`](::connectrpc::StreamMessage) per received message from
/// `.message().await` — read fields zero-copy through the generated accessor
/// methods (`msg.name()`) or `.view()`, or convert with `.to_owned_message()`.
#[derive(Clone)]
pub struct NativeSemanticClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
impl<T> NativeSemanticClient<T>
where
    T: ::connectrpc::client::ClientTransport,
    <T::ResponseBody as ::connectrpc::http_body::Body>::Error: ::std::fmt::Display,
{
    /// Create a new client with the given transport and configuration.
    pub fn new(transport: T, config: ::connectrpc::client::ClientConfig) -> Self {
        Self { transport, config }
    }
    /// Get the client configuration.
    pub fn config(&self) -> &::connectrpc::client::ClientConfig {
        &self.config
    }
    /// Get a mutable reference to the client configuration.
    pub fn config_mut(&mut self) -> &mut ::connectrpc::client::ClientConfig {
        &mut self.config
    }
    /// Call the Status RPC. Sends a request to /pse.code_intelligence.v1.NativeSemantic/Status.
    pub async fn status(
        &self,
        request: ::buffa_types::google::protobuf::Empty,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeSessionStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.status_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Status RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn status_with_options(
        &self,
        request: ::buffa_types::google::protobuf::Empty,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeSessionStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                NATIVE_SEMANTIC_STATUS_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the PackageContext RPC. Sends a request to /pse.code_intelligence.v1.NativeSemantic/PackageContext.
    pub async fn package_context(
        &self,
        request: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativePackageReplyView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.package_context_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the PackageContext RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn package_context_with_options(
        &self,
        request: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePackageRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativePackageReplyView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                NATIVE_SEMANTIC_PACKAGE_CONTEXT_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the Capabilities RPC. Sends a request to /pse.code_intelligence.v1.NativeSemantic/Capabilities.
    pub async fn capabilities(
        &self,
        request: ::buffa_types::google::protobuf::Empty,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeSessionStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.capabilities_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the Capabilities RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn capabilities_with_options(
        &self,
        request: ::buffa_types::google::protobuf::Empty,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeSessionStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                NATIVE_SEMANTIC_CAPABILITIES_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the Stop RPC. Sends a request to /pse.code_intelligence.v1.NativeSemantic/Stop.
    pub async fn stop(
        &self,
        request: ::buffa_types::google::protobuf::Empty,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.stop_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Stop RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn stop_with_options(
        &self,
        request: ::buffa_types::google::protobuf::Empty,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                NATIVE_SEMANTIC_STOP_SPEC.with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the Run RPC. Sends a request to /pse.code_intelligence.v1.NativeSemantic/Run.
    pub async fn run(
        &self,
    ) -> Result<
        ::connectrpc::client::BidiStream<
            T::ResponseBody,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeControl,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeEventView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.run_with_options(::connectrpc::client::CallOptions::default()).await
    }
    /// Call the Run RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn run_with_options(
        &self,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::BidiStream<
            T::ResponseBody,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeControl,
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::view::NativeEventView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_bidi_stream(
                &self.transport,
                &self.config,
                NATIVE_SEMANTIC_RUN_SPEC.with_origin(::connectrpc::SpecOrigin::Client),
                options,
            )
            .await
    }
}
