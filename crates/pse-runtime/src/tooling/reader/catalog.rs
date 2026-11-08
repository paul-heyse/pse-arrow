// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Product reader adapter for the canonical maintenance owner's passive catalog.

use super::canonical::{MAXIMUM_READ_MILLIS, redaction_policy_with_credentials};
use super::{
    EncodedResponse, Payload, ReaderError, encode_response, invalid, response_with_policy,
};
use pse_host::{envelope::RedactionPolicy, investigation::NativePath};
use pse_operations::canonical::{maintenance::CatalogRead, operation_scope};
use std::{
    num::NonZeroU64,
    time::{Duration, Instant},
};
pub(super) fn timeout(timeout_ms: Option<NonZeroU64>) -> Result<Duration, ReaderError> {
    let timeout_ms = timeout_ms.map_or(MAXIMUM_READ_MILLIS, NonZeroU64::get);
    if timeout_ms > MAXIMUM_READ_MILLIS {
        return Err(invalid(
            "catalog observation requires a 1..=25000ms observer allowance",
        ));
    }
    Ok(Duration::from_millis(timeout_ms))
}

pub(super) fn encode(
    correlation: String,
    state: NativePath,
    database: Option<String>,
    expected_issuer: &str,
    timeout_ms: Option<NonZeroU64>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<EncodedResponse, ReaderError> {
    let mut policy = RedactionPolicy::from_environment();
    let result = (|| {
        cancel.checkpoint()?;
        let deadline = Instant::now() + timeout(timeout_ms)?;
        let read =
            CatalogRead::from_state(&state.to_path_buf(), database.as_deref(), expected_issuer)
                .map_err(|error| ReaderError::Canonical(Box::new(error)))?;
        policy = redaction_policy_with_credentials([(
            "PSE_CANONICAL_SELECTION_PASSWORD",
            read.viewer_password_for_redaction(),
        )]);
        let executor = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        executor.block_on(operation_scope(deadline, async {
            cancel.checkpoint()?;
            let observation = read
                .inspect()
                .await
                .map_err(|error| ReaderError::Canonical(Box::new(error)))?;
            cancel.checkpoint()?;
            Ok(Payload::CanonicalCatalog {
                observation: Box::new(observation),
            })
        }))
    })();
    encode_response(response_with_policy(correlation, result, &policy))
}
