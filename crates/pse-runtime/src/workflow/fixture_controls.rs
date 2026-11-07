// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent observations and fixed mutations for isolated native test stores.

use std::{collections::BTreeMap, path::PathBuf};

use pse_operations::{
    canonical::{CanonicalError, CanonicalStore, Revision},
    canonical_codec as codec,
    canonical_execution::{AttemptFence, PayloadLocator, result_batch_key, result_payload_locator,
        result_set_key, validate_result_batch},
    generated::surreal as wire,
    native_fixture::FixtureSelection,
    transport::client::IndexedResults,
};

fn invalid(error: impl std::fmt::Display) -> CanonicalError {
    CanonicalError::Configuration(error.to_string())
}

pub(crate) async fn source_problem_head(
    store: &CanonicalStore,
    problem: &str,
) -> Result<Option<pse_model::generated::runtime::canonical_problems::Row>, CanonicalError> {
    let mut response = FixtureSelection::selected()?.query(store,
        "SELECT * FROM ONLY type::record('canonical_problems',$problem);",
        ("problem", problem.to_owned()),
    ).await?;
    response.take::<Option<_>>(0)?.map(wire::decode_canonical_problems).transpose().map_err(Into::into)
}

const PENDING_BATCH: &str = "RETURN SELECT * FROM ONLY type::record('canonical_result_batches',$key) WHERE attempt=$attempt AND result_set=$set;";

fn pending_batch_bindings(
    fence: &AttemptFence,
    output: &str,
    ordinal: u64,
) -> BTreeMap<String, String> {
    let set = result_set_key(fence.attempt(), output);
    let key = result_batch_key(fence.attempt(), &set, ordinal);
    BTreeMap::from([
        ("key".into(), key),
        ("attempt".into(), fence.attempt().to_owned()),
        ("set".into(), set),
    ])
}

fn decode_pending_batch(
    mut response: IndexedResults,
) -> Result<Option<pse_operations::canonical_execution::ResultBatch>, CanonicalError> {
    response.take::<Option<_>>(0)?.map(wire::decode_canonical_result_batches).transpose().map_err(Into::into)
}

async fn pending_batch(
    store: &CanonicalStore,
    fence: &AttemptFence,
    output: &str,
    ordinal: u64,
) -> Result<Option<pse_operations::canonical_execution::ResultBatch>, CanonicalError> {
    decode_pending_batch(FixtureSelection::selected()?.query(store, PENDING_BATCH,
        pending_batch_bindings(fence, output, ordinal),
    ).await?)
}

#[cfg(feature = "native-service")]
pub(crate) async fn pending_owned_result_batch_rows(
    service: &pse_operations::native_service::NativeCanonicalService,
    fence: &AttemptFence,
    output: &str,
    ordinal: u64,
) -> Result<Option<u64>, pse_operations::native_service::NativeServiceError> {
    let response = service.fixture_query(PENDING_BATCH,
        pending_batch_bindings(fence, output, ordinal),
    ).await?;
    Ok(decode_pending_batch(response)?.map(|batch| batch.row_count))
}

pub(crate) async fn pending_result_batch_rows(
    store: &CanonicalStore,
    fence: &AttemptFence,
    output: &str,
    ordinal: u64,
) -> Result<Option<u64>, CanonicalError> {
    Ok(pending_batch(store, fence, output, ordinal).await?.map(|batch| batch.row_count))
}

pub(crate) async fn pending_result_payload_coordinates(
    store: &CanonicalStore,
    fence: &AttemptFence,
    output: &str,
    ordinal: u64,
) -> Result<Option<(PayloadLocator, String)>, CanonicalError> {
    pending_batch(store, fence, output, ordinal).await?.map(|batch| {
        validate_result_batch(&batch)?;
        Ok((result_payload_locator(&batch)?, batch.schema))
    }).transpose()
}

pub(crate) async fn result_payload_counts(
    store: &CanonicalStore,
    fence: &AttemptFence,
) -> Result<(u64, u64), CanonicalError> {
    let mut response = FixtureSelection::selected()?.query(store,
        "RETURN {inline:<decimal>array::len((SELECT key FROM canonical_inline_payloads WHERE attempt=$attempt)),sealed:<decimal>array::len((SELECT key FROM canonical_file_instances WHERE owner_kind='result' AND selection=$attempt))};",
        ("attempt", fence.attempt().to_owned()),
    ).await?;
    let mut row = response.take::<Option<_>>(0)?.ok_or(CanonicalError::IncompleteResponse)?;
    Ok((codec::decode_uint(codec::required(&mut row, "inline")?)?,
        codec::decode_uint(codec::required(&mut row, "sealed")?)?))
}

pub(crate) async fn expire_acknowledged_attempt(
    store: &CanonicalStore,
    fence: &AttemptFence,
) -> Result<(), CanonicalError> {
    FixtureSelection::selected()?.expire_acknowledged_attempt(store, fence).await
}

pub(crate) async fn invalidate_terminal_study_outcome(
    store: &CanonicalStore,
    study: &str,
    occurrence: pse_model::study::OccurrenceKey,
) -> Result<(), CanonicalError> {
    FixtureSelection::selected()?.query(store,
        "LET $study = SELECT * FROM ONLY type::record('canonical_studies',$study);
        IF $study = NONE OR !$study.terminal { THROW 'PSE_REFUSAL:fixture_study_is_not_terminal'; };
        UPDATE ONLY type::record('canonical_study_points',$point) SET outcome=encoding::base64::decode('/w==');",
        ("study", study.to_owned(), "point",
            pse_operations::canonical_studies::point_key(study, occurrence)),
    ).await?;
    Ok(())
}

pub(crate) async fn invalidate_prepared_revision(
    store: &CanonicalStore,
    revision: &Revision,
) -> Result<(), CanonicalError> {
    FixtureSelection::selected()?.query(store,
        "DELETE ONLY type::record('canonical_revisions',$key) WHERE problem=$problem AND sequence=$sequence;",
        ("key", revision.key.clone(), "problem", revision.problem.clone(),
            "sequence", codec::encode_uint(revision.sequence)?),
    ).await?;
    Ok(())
}

pub(crate) async fn corrupt_product_block(
    store: &CanonicalStore,
    key: &str,
) -> Result<(), CanonicalError> {
    FixtureSelection::selected()?.query(store,
        "LET $association = SELECT * FROM ONLY type::record('canonical_product_associations',$key);
        IF $association = NONE { THROW 'PSE_REFUSAL:fixture_product_missing'; };
        UPDATE ONLY type::record('canonical_payload_blocks',$association.content + ':0') SET payload=encoding::base64::decode('AA==');",
        ("key", key.to_owned()),
    ).await?;
    Ok(())
}

pub(crate) async fn with_unavailable_fixture_payloads<F: Future>(
    store: &CanonicalStore,
    future: F,
) -> Result<F::Output, CanonicalError> {
    let original = FixtureSelection::selected()?.payload_instances(store)?;
    if !std::fs::symlink_metadata(&original).map_err(invalid)?.file_type().is_dir()
        || std::fs::read_dir(&original).map_err(invalid)?.next().transpose().map_err(invalid)?.is_none()
    {
        return Err(invalid("fixture requires nonempty published file instances"));
    }
    struct Restore {
        original: PathBuf,
        withheld: PathBuf,
        restored: bool,
    }
    impl Restore {
        fn restore(&mut self) -> Result<(), CanonicalError> {
            match std::fs::remove_dir(&self.original) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(invalid(error)),
            }
            std::fs::rename(&self.withheld, &self.original).map_err(invalid)?;
            self.restored = true;
            Ok(())
        }
    }
    impl Drop for Restore {
        fn drop(&mut self) {
            if !self.restored {
                let _ = self.restore();
            }
        }
    }
    let withheld = original.with_file_name(format!(".withheld-{}", uuid::Uuid::new_v4()));
    std::fs::rename(&original, &withheld).map_err(invalid)?;
    let mut restore = Restore { original, withheld, restored: false };
    std::fs::create_dir(&restore.original).map_err(invalid)?;
    let outcome = future.await;
    restore.restore()?;
    Ok(outcome)
}
