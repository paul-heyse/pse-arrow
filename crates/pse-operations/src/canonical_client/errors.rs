// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native typed error reconstruction under the response's borrowed wire limits.
use super::{EngineResult, owned::REQUEST};
use surrealdb_protocol::proto::v1 as proto;
use surrealdb_types::{Error, Object, SerializationError, Value, prost::Message};
fn deserialization_error(error: impl std::fmt::Display) -> Error {
    Error::serialization(error.to_string(), SerializationError::Deserialization)
}
pub(super) fn status_to_error(status: tonic::Status) -> Error {
    // Unary RPC failures travel in trailers. The selected native server carries
    // the same typed protobuf error as query frames in this explicit binary
    // metadata field; a status code/message alone never implies a reason.
    if let Some(encoded) = status.metadata().get_bin("surreal-error-bin") {
        const LIMIT: usize = 8 << 10;
        let decode = || -> EngineResult<Error> {
            if encoded.as_encoded_bytes().len() > LIMIT.div_ceil(3) * 4 {
                return Err(deserialization_error(
                    "native error metadata exceeds header envelope",
                ));
            }
            let bytes = encoded.to_bytes().map_err(deserialization_error)?;
            if bytes.len() > LIMIT {
                return Err(deserialization_error(
                    "native error metadata exceeds header envelope",
                ));
            }
            let limits = REQUEST
                .try_with(|ctx| ctx.response_limits())
                .ok()
                .flatten()
                .unwrap_or(super::request::ResponseLimits {
                    wire_bytes: LIMIT,
                    nodes: 1024,
                    depth: 16,
                    container_items: 128,
                    string_bytes: LIMIT,
                    string_length: LIMIT,
                    blob_bytes: LIMIT,
                    blob_length: LIMIT,
                });
            super::wire::Counts::default()
                .check(&bytes, Some("SurrealError"), limits)
                .map_err(deserialization_error)?;
            let error = proto::SurrealError::decode(bytes).map_err(deserialization_error)?;
            Ok(proto_error(error))
        };
        return decode().unwrap_or_else(|error| error);
    }
    use tonic::Code;
    let message = status.message().to_string();
    match status.code() {
        Code::InvalidArgument | Code::OutOfRange => Error::validation(message, None),
        Code::Unimplemented => Error::configuration(message, None),
        // A request the server executed and could not carry out.
        Code::DeadlineExceeded | Code::Aborted => Error::query(message, None),
        // `FailedPrecondition` covers two unlike outcomes. An import whose
        // statements ran and failed reports it, and the database holds part of
        // a dump; so do the guards that refuse a request before executing
        // anything, which leave no state. Only the second is marked, and only
        // by a server new enough to mark it — so an unmarked status is read as
        // the first, which is the safe way to be wrong: it tells a caller state
        // may have been written when none was, rather than the reverse.
        Code::FailedPrecondition => {
            if status
                .metadata()
                .contains_key("surreal-rejected-before-execution")
            {
                Error::validation(message, None)
            } else {
                Error::query(message, None)
            }
        }
        Code::PermissionDenied | Code::Unauthenticated => Error::not_allowed(message, None),
        Code::NotFound => Error::not_found(message, None),
        Code::AlreadyExists => Error::already_exists(message, None),
        Code::Unavailable | Code::Cancelled | Code::Unknown => Error::connection(message, None),
        _ => Error::internal(message),
    }
}

fn kind_str(kind: proto::ErrorKind) -> &'static str {
    use proto::ErrorKind;
    match kind {
        ErrorKind::Validation => "Validation",
        ErrorKind::Configuration => "Configuration",
        ErrorKind::Query => "Query",
        ErrorKind::Serialization => "Serialization",
        ErrorKind::NotAllowed => "NotAllowed",
        ErrorKind::NotFound => "NotFound",
        ErrorKind::AlreadyExists => "AlreadyExists",
        ErrorKind::Connection => "Connection",
        ErrorKind::Thrown => "Thrown",
        ErrorKind::Context => "Context",
        ErrorKind::Internal | ErrorKind::Unspecified => "Internal",
    }
}

/// Rebuilds the value a typed detail deserializes from.
///
/// Each level of a detail serializes as `{ kind, details }`, and the wire
/// splits one level of that open: `ErrorDetails.kind` names the reason and
/// `ErrorDetails.content` carries whatever nests below it. Putting the two back
/// into a single object is what lets the shared taxonomy parse it, however deep
/// the nesting goes.
fn proto_details(details: proto::ErrorDetails) -> Option<Value> {
    if details.kind.is_empty() {
        return None;
    }
    let mut object = Object::new();
    object.insert("kind".to_string(), Value::String(details.kind));
    // A payload this build cannot decode is left out rather than failing the
    // whole reason. A reason that needs its payload -- `TimedOut` and its
    // duration -- then falls back to the bare kind when the taxonomy parses
    // this, and one that does not, such as `TransactionConflict`, is unaffected.
    if let Some(content) = details
        .content
        .and_then(|content| Value::try_from(content).ok())
    {
        object.insert("details".to_string(), content);
    }
    Some(Value::Object(object))
}

/// Maps a structured error carried on the wire, preserving its kind, message,
/// reason, retryability and cause chain.
///
/// The reason is rebuilt into the typed detail supplied by surrealdb-types, so
/// `query_details()` and its siblings answer the same here as on any other
/// transport. Leaving it out would push callers back onto the message, which
/// the protocol states is not part of the contract and will be reworded.
pub(super) fn proto_error(error: proto::SurrealError) -> Error {
    let kind = error.kind_or_internal();
    let cause = error.cause.map(|cause| proto_error(*cause));
    let details = error.details.and_then(proto_details);
    // `SurrealError.retry` is not rebuilt into anything. It says the operation
    // may be retried and deliberately says nothing about why, so there is no
    // reason to derive from it -- and `Error` has nowhere to hold retryability
    // apart from the reason, since a field for it would change the shape every
    // transport serialises this type in. A conflict still reaches callers as
    // `QueryError::TransactionConflict`, which this server pairs with the hint,
    // so a retry loop has the typed reason to branch on either way. A peer that
    // sends a hint with some other reason, or with none, has its retryability
    // dropped here rather than turned into a reason it did not state.
    let mapped = Error::from_wire(error.message, Some(kind_str(kind)), details);
    match cause {
        Some(cause) => mapped.with_cause(cause),
        None => mapped,
    }
}
