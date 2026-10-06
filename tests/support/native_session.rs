// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual native engine sources with declared contracts and bounded ownership.
#![allow(clippy::expect_used, reason = "isolated engine fixtures")]
use std::{collections::BTreeMap, sync::Arc};

pub(crate) fn candidate(
    registry: Arc<pse_schema::Registry>,
    batches: BTreeMap<pse_schema::model::RelationKey, arrow::array::RecordBatch>,
) -> (
    pse_engine::session::EngineSession,
    tempfile::TempDir,
    Arc<dyn pse_columnar::MemoryPool>,
) {
    let directory = tempfile::tempdir().expect("fixture spill directory");
    let fixture = pse_testkit::NativeFixture::new((256 << 20).try_into().expect("bounded pool"))
        .expect("native factory");
    let factory = fixture.into_factory();
    let pool = factory.pool().clone();
    let session = factory
        .candidate(batches, registry, &pse_columnar::CancellationToken::new())
        .expect("actual declared native inputs");
    (session, directory, pool)
}
