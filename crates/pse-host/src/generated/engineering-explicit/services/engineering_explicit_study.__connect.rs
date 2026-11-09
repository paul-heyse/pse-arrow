///Shorthand for `OwnedView<EngineeringPrepareExplicitStudyView<'static>>`.
pub type OwnedEngineeringPrepareExplicitStudyView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPrepareExplicitStudyView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringPreparedStudyView<'static>>`.
pub type OwnedEngineeringPreparedStudyView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPreparedStudyView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringRestoreExplicitStudyView<'static>>`.
pub type OwnedEngineeringRestoreExplicitStudyView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringRestoreExplicitStudyView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringOperationKeyView<'static>>`.
pub type OwnedEngineeringOperationKeyView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringOperationKeyView<
        'static,
    >,
>;
///Shorthand for `OwnedView<ExplicitStudyDefinitionView<'static>>`.
pub type OwnedExplicitStudyDefinitionView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::ExplicitStudyDefinitionView<
        'static,
    >,
>;
impl ::connectrpc::Encodable<
    crate::engineering_client::pb::pse::engineering::v1::EngineeringPreparedStudy,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPreparedStudyView<
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
    crate::engineering_client::pb::pse::engineering::v1::EngineeringPreparedStudy,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPreparedStudyView<
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
    crate::engineering_client::pb::pse::engineering::v1::ExplicitStudyDefinition,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::ExplicitStudyDefinitionView<
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
    crate::engineering_client::pb::pse::engineering::v1::ExplicitStudyDefinition,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::ExplicitStudyDefinitionView<
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
pub const EXPLICIT_STUDY_PREPARATION_V2_SERVICE_NAME: &str = "pse.engineering.v1.ExplicitStudyPreparationV2";
/// Static [`Spec`](::connectrpc::Spec) for the `PrepareExplicitStudy` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const EXPLICIT_STUDY_PREPARATION_V2_PREPARE_EXPLICIT_STUDY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.ExplicitStudyPreparationV2/PrepareExplicitStudy",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `RestoreExplicitStudy` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const EXPLICIT_STUDY_PREPARATION_V2_RESTORE_EXPLICIT_STUDY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.ExplicitStudyPreparationV2/RestoreExplicitStudy",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `ReadPreparedExplicitStudy` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const EXPLICIT_STUDY_PREPARATION_V2_READ_PREPARED_EXPLICIT_STUDY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.ExplicitStudyPreparationV2/ReadPreparedExplicitStudy",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for ExplicitStudyPreparationV2.
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
pub trait ExplicitStudyPreparationV2: Send + Sync + 'static {
    /// Handle the PrepareExplicitStudy RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn prepare_explicit_study<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringPrepareExplicitStudy,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::EngineeringPreparedStudy,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the RestoreExplicitStudy RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn restore_explicit_study<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringRestoreExplicitStudy,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::EngineeringPreparedStudy,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the ReadPreparedExplicitStudy RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn read_prepared_explicit_study<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::ExplicitStudyDefinition,
            > + Send + use<'a, Self>,
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
pub trait ExplicitStudyPreparationV2Ext: ExplicitStudyPreparationV2 {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: ExplicitStudyPreparationV2> ExplicitStudyPreparationV2Ext for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                EXPLICIT_STUDY_PREPARATION_V2_SERVICE_NAME,
                "PrepareExplicitStudy",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPrepareExplicitStudyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringPrepareExplicitStudy,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.prepare_explicit_study(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::EngineeringPreparedStudy,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(EXPLICIT_STUDY_PREPARATION_V2_PREPARE_EXPLICIT_STUDY_SPEC)
            .route_view(
                EXPLICIT_STUDY_PREPARATION_V2_SERVICE_NAME,
                "RestoreExplicitStudy",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringRestoreExplicitStudyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringRestoreExplicitStudy,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.restore_explicit_study(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::EngineeringPreparedStudy,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(EXPLICIT_STUDY_PREPARATION_V2_RESTORE_EXPLICIT_STUDY_SPEC)
            .route_view(
                EXPLICIT_STUDY_PREPARATION_V2_SERVICE_NAME,
                "ReadPreparedExplicitStudy",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringOperationKeyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.read_prepared_explicit_study(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::ExplicitStudyDefinition,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(EXPLICIT_STUDY_PREPARATION_V2_READ_PREPARED_EXPLICIT_STUDY_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct ExplicitStudyPreparationV2RegisterMarker;
impl<
    S: ExplicitStudyPreparationV2,
> ::connectrpc::ServiceRegister<ExplicitStudyPreparationV2RegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as ExplicitStudyPreparationV2Ext>::register(self, router)
    }
}
/// Monomorphic dispatcher for `ExplicitStudyPreparationV2`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = ExplicitStudyPreparationV2Server::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct ExplicitStudyPreparationV2Server<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: ExplicitStudyPreparationV2> ExplicitStudyPreparationV2Server<T> {
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
impl<T> Clone for ExplicitStudyPreparationV2Server<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: ExplicitStudyPreparationV2> ::connectrpc::Dispatcher
for ExplicitStudyPreparationV2Server<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path
            .strip_prefix("pse.engineering.v1.ExplicitStudyPreparationV2/")?;
        match method {
            "PrepareExplicitStudy" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(
                            EXPLICIT_STUDY_PREPARATION_V2_PREPARE_EXPLICIT_STUDY_SPEC,
                        ),
                )
            }
            "RestoreExplicitStudy" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(
                            EXPLICIT_STUDY_PREPARATION_V2_RESTORE_EXPLICIT_STUDY_SPEC,
                        ),
                )
            }
            "ReadPreparedExplicitStudy" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(
                            EXPLICIT_STUDY_PREPARATION_V2_READ_PREPARED_EXPLICIT_STUDY_SPEC,
                        ),
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
        let Some(method) = path
            .strip_prefix("pse.engineering.v1.ExplicitStudyPreparationV2/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "PrepareExplicitStudy" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringPrepareExplicitStudy,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPrepareExplicitStudyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringPrepareExplicitStudy,
                    >::from_parts(&req, &body);
                    svc.prepare_explicit_study(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::EngineeringPreparedStudy,
                        >(format)
                })
            }
            "RestoreExplicitStudy" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringRestoreExplicitStudy,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringRestoreExplicitStudyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringRestoreExplicitStudy,
                    >::from_parts(&req, &body);
                    svc.restore_explicit_study(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::EngineeringPreparedStudy,
                        >(format)
                })
            }
            "ReadPreparedExplicitStudy" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringOperationKeyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
                    >::from_parts(&req, &body);
                    svc.read_prepared_explicit_study(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::ExplicitStudyDefinition,
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
        let Some(method) = path
            .strip_prefix("pse.engineering.v1.ExplicitStudyPreparationV2/") else {
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
        let Some(method) = path
            .strip_prefix("pse.engineering.v1.ExplicitStudyPreparationV2/") else {
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
        let Some(method) = path
            .strip_prefix("pse.engineering.v1.ExplicitStudyPreparationV2/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
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
/// let client = ExplicitStudyPreparationV2Client::new(conn, config);
/// let response = client.prepare_explicit_study(request).await?;
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
/// let client = ExplicitStudyPreparationV2Client::new(http, config);
/// let response = client.prepare_explicit_study(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.prepare_explicit_study(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.prepare_explicit_study(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one
/// [`StreamMessage`](::connectrpc::StreamMessage) per received message from
/// `.message().await` — read fields zero-copy through the generated accessor
/// methods (`msg.name()`) or `.view()`, or convert with `.to_owned_message()`.
#[derive(Clone)]
pub struct ExplicitStudyPreparationV2Client<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
impl<T> ExplicitStudyPreparationV2Client<T>
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
    /// Call the PrepareExplicitStudy RPC. Sends a request to /pse.engineering.v1.ExplicitStudyPreparationV2/PrepareExplicitStudy.
    pub async fn prepare_explicit_study(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringPrepareExplicitStudy,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPreparedStudyView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.prepare_explicit_study_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the PrepareExplicitStudy RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn prepare_explicit_study_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringPrepareExplicitStudy,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPreparedStudyView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                EXPLICIT_STUDY_PREPARATION_V2_PREPARE_EXPLICIT_STUDY_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the RestoreExplicitStudy RPC. Sends a request to /pse.engineering.v1.ExplicitStudyPreparationV2/RestoreExplicitStudy.
    pub async fn restore_explicit_study(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringRestoreExplicitStudy,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPreparedStudyView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.restore_explicit_study_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the RestoreExplicitStudy RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn restore_explicit_study_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringRestoreExplicitStudy,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringPreparedStudyView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                EXPLICIT_STUDY_PREPARATION_V2_RESTORE_EXPLICIT_STUDY_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the ReadPreparedExplicitStudy RPC. Sends a request to /pse.engineering.v1.ExplicitStudyPreparationV2/ReadPreparedExplicitStudy.
    pub async fn read_prepared_explicit_study(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::ExplicitStudyDefinitionView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.read_prepared_explicit_study_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ReadPreparedExplicitStudy RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn read_prepared_explicit_study_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::ExplicitStudyDefinitionView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                EXPLICIT_STUDY_PREPARATION_V2_READ_PREPARED_EXPLICIT_STUDY_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
}
