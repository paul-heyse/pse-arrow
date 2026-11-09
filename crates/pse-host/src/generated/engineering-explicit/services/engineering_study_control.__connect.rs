///Shorthand for `OwnedView<EngineeringDescribeRequestView<'static>>`.
pub type OwnedEngineeringDescribeRequestView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringDescribeRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringContractView<'static>>`.
pub type OwnedEngineeringContractView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringContractView<
        'static,
    >,
>;
///Shorthand for `OwnedView<StudyWorkbenchRequestView<'static>>`.
pub type OwnedStudyWorkbenchRequestView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyWorkbenchRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<StudyWorkbenchPageView<'static>>`.
pub type OwnedStudyWorkbenchPageView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyWorkbenchPageView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringSubmitStudyView<'static>>`.
pub type OwnedEngineeringSubmitStudyView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringSubmitStudyView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringStudyStatusView<'static>>`.
pub type OwnedEngineeringStudyStatusView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringStudyKeyView<'static>>`.
pub type OwnedEngineeringStudyKeyView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyKeyView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringObserveStudyView<'static>>`.
pub type OwnedEngineeringObserveStudyView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringObserveStudyView<
        'static,
    >,
>;
///Shorthand for `OwnedView<StudyObservationPageView<'static>>`.
pub type OwnedStudyObservationPageView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyObservationPageView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringFreezeStudyCohortView<'static>>`.
pub type OwnedEngineeringFreezeStudyCohortView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringFreezeStudyCohortView<
        'static,
    >,
>;
///Shorthand for `OwnedView<StudyCohortReceiptView<'static>>`.
pub type OwnedStudyCohortReceiptView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortReceiptView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringStudyHeaderPageView<'static>>`.
pub type OwnedEngineeringStudyHeaderPageView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyHeaderPageView<
        'static,
    >,
>;
///Shorthand for `OwnedView<StudyCohortHeaderPageView<'static>>`.
pub type OwnedStudyCohortHeaderPageView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortHeaderPageView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringStudyCohortPageView<'static>>`.
pub type OwnedEngineeringStudyCohortPageView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyCohortPageView<
        'static,
    >,
>;
///Shorthand for `OwnedView<StudyCohortMemberPageView<'static>>`.
pub type OwnedStudyCohortMemberPageView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortMemberPageView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringReadStudyResultView<'static>>`.
pub type OwnedEngineeringReadStudyResultView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringReadStudyResultView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringStudyResultPageView<'static>>`.
pub type OwnedEngineeringStudyResultPageView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyResultPageView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EngineeringCohortReleaseView<'static>>`.
pub type OwnedEngineeringCohortReleaseView = ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringCohortReleaseView<
        'static,
    >,
>;
impl ::connectrpc::Encodable<
    crate::engineering_client::pb::pse::engineering::v1::EngineeringContract,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringContractView<
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
    crate::engineering_client::pb::pse::engineering::v1::EngineeringContract,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringContractView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchPage,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyWorkbenchPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchPage,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyWorkbenchPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
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
    crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyObservationPage,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyObservationPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyObservationPage,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyObservationPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyCohortReceipt,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortReceiptView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyCohortReceipt,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortReceiptView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyCohortHeaderPage,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortHeaderPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyCohortHeaderPage,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortHeaderPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyCohortMemberPage,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortMemberPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::StudyCohortMemberPage,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortMemberPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyResultPage,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyResultPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyResultPage,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyResultPageView<
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
    crate::engineering_client::pb::pse::engineering::v1::EngineeringCohortRelease,
>
for crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringCohortReleaseView<
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
    crate::engineering_client::pb::pse::engineering::v1::EngineeringCohortRelease,
>
for ::buffa::view::OwnedView<
    crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringCohortReleaseView<
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
pub const STUDY_CONTROL_V2_SERVICE_NAME: &str = "pse.engineering.v1.StudyControlV2";
/// Static [`Spec`](::connectrpc::Spec) for the `DescribeStudyContract` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_DESCRIBE_STUDY_CONTRACT_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/DescribeStudyContract",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `BrowseStudies` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_BROWSE_STUDIES_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/BrowseStudies",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `SubmitStudy` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_SUBMIT_STUDY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/SubmitStudy",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `StudyStatus` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_STUDY_STATUS_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/StudyStatus",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `CancelStudy` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_CANCEL_STUDY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/CancelStudy",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `ObserveStudy` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_OBSERVE_STUDY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/ObserveStudy",
        ::connectrpc::StreamType::ServerStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `FreezeStudyCohort` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_FREEZE_STUDY_COHORT_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/FreezeStudyCohort",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `ReadStudyHeaders` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_READ_STUDY_HEADERS_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/ReadStudyHeaders",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `ReadStudyMembers` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_READ_STUDY_MEMBERS_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/ReadStudyMembers",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `ReadStudyResult` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_READ_STUDY_RESULT_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/ReadStudyResult",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `ReleaseStudyCohort` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const STUDY_CONTROL_V2_RELEASE_STUDY_COHORT_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/pse.engineering.v1.StudyControlV2/ReleaseStudyCohort",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for StudyControlV2.
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
pub trait StudyControlV2: Send + Sync + 'static {
    /// Handle the DescribeStudyContract RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn describe_study_contract<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringDescribeRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::EngineeringContract,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the BrowseStudies RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn browse_studies<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchPage,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the SubmitStudy RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn submit_study<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringSubmitStudy,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the StudyStatus RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn study_status<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the CancelStudy RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn cancel_study<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the ObserveStudy RPC.
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call (until the response stream is returned);
    /// message fields are read directly on it (zero-copy). Data the
    /// returned stream needs must be copied out or converted via
    /// `.to_owned_message()`.
    fn observe_study(
        &self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringObserveStudy,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::engineering_client::pb::pse::engineering::v1::StudyObservationPage,
                > + Send + use<Self>,
            >,
        >,
    > + Send;
    /// Handle the FreezeStudyCohort RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn freeze_study_cohort<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringFreezeStudyCohort,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::StudyCohortReceipt,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the ReadStudyHeaders RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn read_study_headers<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyHeaderPage,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::StudyCohortHeaderPage,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the ReadStudyMembers RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn read_study_members<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyCohortPage,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::StudyCohortMemberPage,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the ReadStudyResult RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn read_study_result<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringReadStudyResult,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyResultPage,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Handle the ReleaseStudyCohort RPC.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn release_study_cohort<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::engineering_client::pb::pse::engineering::v1::EngineeringCohortRelease,
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
pub trait StudyControlV2Ext: StudyControlV2 {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: StudyControlV2> StudyControlV2Ext for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "DescribeStudyContract",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringDescribeRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringDescribeRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.describe_study_contract(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::EngineeringContract,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_DESCRIBE_STUDY_CONTRACT_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "BrowseStudies",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyWorkbenchRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.browse_studies(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchPage,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_BROWSE_STUDIES_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "SubmitStudy",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringSubmitStudyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringSubmitStudy,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.submit_study(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_SUBMIT_STUDY_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "StudyStatus",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyKeyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.study_status(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_STUDY_STATUS_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "CancelStudy",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyKeyView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.cancel_study(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_CANCEL_STUDY_SPEC)
            .route_view_server_stream::<
                _,
                _,
                crate::engineering_client::pb::pse::engineering::v1::StudyObservationPage,
            >(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "ObserveStudy",
                ::connectrpc::view_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringObserveStudyView<
                                'static,
                            >,
                        >|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringObserveStudy,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.observe_study(ctx, sreq).await
                        }
                    }
                }),
            )
            .with_spec(STUDY_CONTROL_V2_OBSERVE_STUDY_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "FreezeStudyCohort",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringFreezeStudyCohortView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringFreezeStudyCohort,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.freeze_study_cohort(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::StudyCohortReceipt,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_FREEZE_STUDY_COHORT_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "ReadStudyHeaders",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyHeaderPageView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyHeaderPage,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.read_study_headers(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::StudyCohortHeaderPage,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_READ_STUDY_HEADERS_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "ReadStudyMembers",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyCohortPageView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyCohortPage,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.read_study_members(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::StudyCohortMemberPage,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_READ_STUDY_MEMBERS_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "ReadStudyResult",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringReadStudyResultView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::engineering_client::pb::pse::engineering::v1::EngineeringReadStudyResult,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.read_study_result(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyResultPage,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_READ_STUDY_RESULT_SPEC)
            .route_view(
                STUDY_CONTROL_V2_SERVICE_NAME,
                "ReleaseStudyCohort",
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
                            svc.release_study_cohort(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::engineering_client::pb::pse::engineering::v1::EngineeringCohortRelease,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(STUDY_CONTROL_V2_RELEASE_STUDY_COHORT_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct StudyControlV2RegisterMarker;
impl<S: StudyControlV2> ::connectrpc::ServiceRegister<StudyControlV2RegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as StudyControlV2Ext>::register(self, router)
    }
}
/// Monomorphic dispatcher for `StudyControlV2`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = StudyControlV2Server::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct StudyControlV2Server<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: StudyControlV2> StudyControlV2Server<T> {
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
impl<T> Clone for StudyControlV2Server<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: StudyControlV2> ::connectrpc::Dispatcher for StudyControlV2Server<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("pse.engineering.v1.StudyControlV2/")?;
        match method {
            "DescribeStudyContract" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_DESCRIBE_STUDY_CONTRACT_SPEC),
                )
            }
            "BrowseStudies" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_BROWSE_STUDIES_SPEC),
                )
            }
            "SubmitStudy" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_SUBMIT_STUDY_SPEC),
                )
            }
            "StudyStatus" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_STUDY_STATUS_SPEC),
                )
            }
            "CancelStudy" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_CANCEL_STUDY_SPEC),
                )
            }
            "ObserveStudy" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::server_streaming()
                        .with_spec(STUDY_CONTROL_V2_OBSERVE_STUDY_SPEC),
                )
            }
            "FreezeStudyCohort" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_FREEZE_STUDY_COHORT_SPEC),
                )
            }
            "ReadStudyHeaders" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_READ_STUDY_HEADERS_SPEC),
                )
            }
            "ReadStudyMembers" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_READ_STUDY_MEMBERS_SPEC),
                )
            }
            "ReadStudyResult" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_READ_STUDY_RESULT_SPEC),
                )
            }
            "ReleaseStudyCohort" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(STUDY_CONTROL_V2_RELEASE_STUDY_COHORT_SPEC),
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
        let Some(method) = path.strip_prefix("pse.engineering.v1.StudyControlV2/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "DescribeStudyContract" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringDescribeRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringDescribeRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringDescribeRequest,
                    >::from_parts(&req, &body);
                    svc.describe_study_contract(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::EngineeringContract,
                        >(format)
                })
            }
            "BrowseStudies" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyWorkbenchRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchRequest,
                    >::from_parts(&req, &body);
                    svc.browse_studies(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchPage,
                        >(format)
                })
            }
            "SubmitStudy" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringSubmitStudy,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringSubmitStudyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringSubmitStudy,
                    >::from_parts(&req, &body);
                    svc.submit_study(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
                        >(format)
                })
            }
            "StudyStatus" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyKeyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
                    >::from_parts(&req, &body);
                    svc.study_status(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
                        >(format)
                })
            }
            "CancelStudy" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyKeyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
                    >::from_parts(&req, &body);
                    svc.cancel_study(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyStatus,
                        >(format)
                })
            }
            "FreezeStudyCohort" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringFreezeStudyCohort,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringFreezeStudyCohortView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringFreezeStudyCohort,
                    >::from_parts(&req, &body);
                    svc.freeze_study_cohort(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::StudyCohortReceipt,
                        >(format)
                })
            }
            "ReadStudyHeaders" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyHeaderPage,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyHeaderPageView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyHeaderPage,
                    >::from_parts(&req, &body);
                    svc.read_study_headers(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::StudyCohortHeaderPage,
                        >(format)
                })
            }
            "ReadStudyMembers" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyCohortPage,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyCohortPageView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyCohortPage,
                    >::from_parts(&req, &body);
                    svc.read_study_members(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::StudyCohortMemberPage,
                        >(format)
                })
            }
            "ReadStudyResult" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringReadStudyResult,
                    >(request.encoded()?, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringReadStudyResultView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringReadStudyResult,
                    >::from_parts(&req, &body);
                    svc.read_study_result(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyResultPage,
                        >(format)
                })
            }
            "ReleaseStudyCohort" => {
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
                    svc.release_study_cohort(ctx, req)
                        .await?
                        .encode::<
                            crate::engineering_client::pb::pse::engineering::v1::EngineeringCohortRelease,
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
        let Some(method) = path.strip_prefix("pse.engineering.v1.StudyControlV2/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "ObserveStudy" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringObserveStudy,
                    >(request, format)?;
                    let req: crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringObserveStudyView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::engineering_client::pb::pse::engineering::v1::EngineeringObserveStudy,
                    >::from_parts(&req, &body);
                    let resp = svc.observe_study(ctx, req).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::engineering_client::pb::pse::engineering::v1::StudyObservationPage,
                                _,
                                _,
                            >(s, format)),
                    )
                })
            }
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
        let Some(method) = path.strip_prefix("pse.engineering.v1.StudyControlV2/") else {
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
        let Some(method) = path.strip_prefix("pse.engineering.v1.StudyControlV2/") else {
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
/// let client = StudyControlV2Client::new(conn, config);
/// let response = client.describe_study_contract(request).await?;
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
/// let client = StudyControlV2Client::new(http, config);
/// let response = client.describe_study_contract(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.describe_study_contract(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.describe_study_contract(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one
/// [`StreamMessage`](::connectrpc::StreamMessage) per received message from
/// `.message().await` — read fields zero-copy through the generated accessor
/// methods (`msg.name()`) or `.view()`, or convert with `.to_owned_message()`.
#[derive(Clone)]
pub struct StudyControlV2Client<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
impl<T> StudyControlV2Client<T>
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
    /// Call the DescribeStudyContract RPC. Sends a request to /pse.engineering.v1.StudyControlV2/DescribeStudyContract.
    pub async fn describe_study_contract(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringDescribeRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringContractView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.describe_study_contract_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the DescribeStudyContract RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn describe_study_contract_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringDescribeRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringContractView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_DESCRIBE_STUDY_CONTRACT_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the BrowseStudies RPC. Sends a request to /pse.engineering.v1.StudyControlV2/BrowseStudies.
    pub async fn browse_studies(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyWorkbenchPageView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.browse_studies_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the BrowseStudies RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn browse_studies_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::StudyWorkbenchRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyWorkbenchPageView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_BROWSE_STUDIES_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the SubmitStudy RPC. Sends a request to /pse.engineering.v1.StudyControlV2/SubmitStudy.
    pub async fn submit_study(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringSubmitStudy,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.submit_study_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the SubmitStudy RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn submit_study_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringSubmitStudy,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_SUBMIT_STUDY_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the StudyStatus RPC. Sends a request to /pse.engineering.v1.StudyControlV2/StudyStatus.
    pub async fn study_status(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.study_status_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the StudyStatus RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn study_status_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_STUDY_STATUS_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the CancelStudy RPC. Sends a request to /pse.engineering.v1.StudyControlV2/CancelStudy.
    pub async fn cancel_study(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.cancel_study_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the CancelStudy RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn cancel_study_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyKey,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyStatusView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_CANCEL_STUDY_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the ObserveStudy RPC. Sends a request to /pse.engineering.v1.StudyControlV2/ObserveStudy.
    pub async fn observe_study(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringObserveStudy,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyObservationPageView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.observe_study_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ObserveStudy RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn observe_study_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringObserveStudy,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyObservationPageView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_server_stream(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_OBSERVE_STUDY_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the FreezeStudyCohort RPC. Sends a request to /pse.engineering.v1.StudyControlV2/FreezeStudyCohort.
    pub async fn freeze_study_cohort(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringFreezeStudyCohort,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortReceiptView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.freeze_study_cohort_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the FreezeStudyCohort RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn freeze_study_cohort_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringFreezeStudyCohort,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortReceiptView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_FREEZE_STUDY_COHORT_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the ReadStudyHeaders RPC. Sends a request to /pse.engineering.v1.StudyControlV2/ReadStudyHeaders.
    pub async fn read_study_headers(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyHeaderPage,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortHeaderPageView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.read_study_headers_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ReadStudyHeaders RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn read_study_headers_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyHeaderPage,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortHeaderPageView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_READ_STUDY_HEADERS_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the ReadStudyMembers RPC. Sends a request to /pse.engineering.v1.StudyControlV2/ReadStudyMembers.
    pub async fn read_study_members(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyCohortPage,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortMemberPageView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.read_study_members_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ReadStudyMembers RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn read_study_members_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringStudyCohortPage,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::StudyCohortMemberPageView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_READ_STUDY_MEMBERS_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the ReadStudyResult RPC. Sends a request to /pse.engineering.v1.StudyControlV2/ReadStudyResult.
    pub async fn read_study_result(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringReadStudyResult,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyResultPageView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.read_study_result_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ReadStudyResult RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn read_study_result_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringReadStudyResult,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringStudyResultPageView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_READ_STUDY_RESULT_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the ReleaseStudyCohort RPC. Sends a request to /pse.engineering.v1.StudyControlV2/ReleaseStudyCohort.
    pub async fn release_study_cohort(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringCohortReleaseView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.release_study_cohort_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ReleaseStudyCohort RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn release_study_cohort_with_options(
        &self,
        request: crate::engineering_client::pb::pse::engineering::v1::EngineeringOperationKey,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::engineering_client::pb::pse::engineering::v1::__buffa::view::EngineeringCohortReleaseView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                STUDY_CONTROL_V2_RELEASE_STUDY_COHORT_SPEC
                    .with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
}
