// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native-client controls for the supervisor's offline recovery journey.

use anyhow::{Context, Result, ensure};
use pse_model::generated::runtime::canonical_products::Row as Product;
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore, ObjectEdit, ObjectVersion},
    canonical_selection::SelectedRead,
    generated::surreal::INTERPRETATION,
};
use std::{path::Path, sync::Arc, time::Duration};

const PROBLEM: &str = "native-recovery-control";
const FIRST: &str = "native-recovery-first";
const SECOND: &str = "native-recovery-second";
const LARGE_BYTES: usize = 5 * 1024 * 1024 + 17;

async fn read_product(
    store: &CanonicalStore,
    read: &mut SelectedRead,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
) -> Result<
    Option<(
        pse_operations::canonical_selection::ReusableProduct,
        pse_columnar::MemoryReservation,
    )>,
> {
    let mut after = String::new();
    while let Some(candidate) = store
        .product_candidate(
            read.selection(),
            b"exact-recovery-result",
            "recovery-control.v1",
            &after,
        )
        .await?
    {
        after = candidate.key().to_owned();
        let reservation = pse_columnar::MemoryConsumer::new("recovery:product").register(pool);
        reservation.try_grow(candidate.retained_bytes())?;
        if let Some(product) = store.qualify_product(read, &candidate).await? {
            return Ok(Some((product, reservation)));
        }
    }
    Ok(None)
}

fn object(logical: &str, version: &str, payload: Vec<u8>) -> ObjectEdit {
    ObjectEdit {
        logical: logical.into(),
        scope: "root".into(),
        name: logical.into(),
        version: Some(ObjectVersion {
            key: version.into(),
            logical: logical.into(),
            kind: "recovery-control".into(),
            payload: payload.into(),
            interpretation: INTERPRETATION.into(),
        }),
        references: Vec::new(),
    }
}

pub(crate) async fn run(state: &Path, seed: bool) -> Result<()> {
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(128 * 1024 * 1024));
    let store = CanonicalStore::connect(&CanonicalOptions::from_state(state)?).await?;
    if seed {
        store.create().await?;
    }
    store.open().await?;
    let first_edits = vec![
        object(
            "x",
            "recovery-x-first",
            (-0.0f64).to_bits().to_be_bytes().to_vec(),
        ),
        object("bank", "recovery-large", vec![0xa7; LARGE_BYTES]),
    ];
    if seed {
        let first = store.edit(PROBLEM, None, FIRST, &first_edits).await?;
        let mut read = SelectedRead::new(store.protect(first, Duration::from_secs(60)).await?);
        store
            .resolve_names(
                &mut read,
                "root",
                &["x".into(), "bank".into(), "absent".into()],
            )
            .await?;
        store
            .publish_product(
                &read,
                Product {
                    key: "recovery-description".into(),
                    problem: PROBLEM.into(),
                    revision: FIRST.into(),
                    request: b"exact-recovery-result".to_vec().into(),
                    payload: 2.0f64.to_bits().to_be_bytes().to_vec().into(),
                    dependencies: Vec::new().into(),
                    producer: "recovery-control.v1".into(),
                    interpretation: INTERPRETATION.into(),
                },
            )
            .await?;
        store.release(read.selection()).await?;
        store
            .edit(
                PROBLEM,
                Some(FIRST),
                SECOND,
                &[object(
                    "x",
                    "recovery-x-second",
                    3.0f64.to_bits().to_be_bytes().to_vec(),
                )],
            )
            .await?;
        ensure!(
            store.edit(PROBLEM, None, FIRST, &first_edits).await?.key == FIRST,
            "lost-ack replay did not return the immutable receipt"
        );
    }
    let first = store
        .revision(FIRST)
        .await?
        .context("historical receipt missing")?;
    let second = store
        .revision(SECOND)
        .await?
        .context("head receipt missing")?;
    ensure!(
        first.sequence == 1 && second.sequence == 2 && second.parent.as_deref() == Some(FIRST),
        "revision lineage changed"
    );
    let mut read = SelectedRead::new(store.protect(first, Duration::from_secs(60)).await?);
    let members = store
        .resolve_names(&mut read, "root", &["x".into(), "bank".into()])
        .await?;
    ensure!(members.len() == 2, "historical selected source missing");
    for member in members {
        let extent = store
            .selected_object_extent(read.selection(), &member.version)
            .await?
            .context("payload extent missing")?;
        let reservation = pse_columnar::MemoryConsumer::new("recovery:source").register(&pool);
        let bytes = extent
            .checked_mul(2)
            .and_then(|n| n.checked_add(2 * pse_operations::canonical_staging::SOURCE_BLOCK_BYTES))
            .context("source extent overflow")?;
        reservation.try_grow(bytes)?;
        let saved = store
            .selected_object(read.selection(), &member.version)
            .await?
            .context("payload missing")?;
        match member.name.as_str() {
            "x" => ensure!(
                saved.payload.as_slice() == (-0.0f64).to_bits().to_be_bytes(),
                "scientific bits changed"
            ),
            "bank" => ensure!(
                saved.payload.len() == LARGE_BYTES
                    && saved.payload.iter().all(|byte| *byte == 0xa7),
                "staged payload changed"
            ),
            _ => unreachable!("only two selected names"),
        }
    }
    {
        let (product, _reservation) = read_product(&store, &mut read, &pool)
            .await?
            .context("retained product missing")?;
        ensure!(
            product.payload() == 2.0f64.to_bits().to_be_bytes(),
            "recorded numerical value changed"
        );
    }
    store.release(read.selection()).await?;
    let mut changed = SelectedRead::new(store.protect(second, Duration::from_secs(60)).await?);
    ensure!(
        read_product(&store, &mut changed, &pool).await?.is_none(),
        "changed dependency incorrectly reused the old description"
    );
    store.release(changed.selection()).await?;
    if !seed {
        // Gated restore permits protection bookkeeping and reads, while authoring remains closed.
        ensure!(
            matches!(
                store
                    .edit(PROBLEM, Some(SECOND), "forbidden-restore-write", &[])
                    .await,
                Err(pse_operations::canonical::CanonicalError::Quiesced)
            ),
            "validation admitted an authoring mutation"
        );
    }
    Ok(())
}
