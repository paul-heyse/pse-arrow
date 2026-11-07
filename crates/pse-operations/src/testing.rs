// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fixed internal controls for operations unit tests.

/// Lose the local cancellation ACK after an actual commit, optionally failing
/// its original receipt readback. The command itself still uses the real owner.
pub(crate) async fn lose_control_cancel_ack<F: Future>(
    fail_readback: bool,
    future: F,
) -> F::Output {
    crate::canonical_control::CONTROL_ACK_LOST
        .scope(
            true,
            crate::canonical_control::CONTROL_READBACK_FAILED.scope(fail_readback, future),
        )
        .await
}

/// Actual stored payload owners for this exact attempt, including reservations.
/// This fixture observation never creates a payload or grants read permission.
pub(crate) async fn result_payload_counts(
    store: &crate::canonical::CanonicalStore,
    fence: &crate::canonical_execution::AttemptFence,
) -> Result<(u64, u64), crate::canonical::CanonicalError> {
    use crate::canonical_codec::{decode_uint, required};
    let mut response = crate::canonical::bounded_query(store.db.query(
        "RETURN {inline:<decimal>array::len((SELECT key FROM canonical_inline_payloads WHERE attempt=$attempt)),sealed:<decimal>array::len((SELECT key FROM canonical_file_instances WHERE owner_kind='result' AND selection=$attempt))};"
    ).bind(("attempt",fence.attempt().to_owned()))).await?;
    let mut row = response
        .take::<Option<surrealdb_types::Object>>(0)?
        .ok_or(crate::canonical::CanonicalError::IncompleteResponse)?;
    Ok((
        decode_uint(required(&mut row, "inline")?)?,
        decode_uint(required(&mut row, "sealed")?)?,
    ))
}
