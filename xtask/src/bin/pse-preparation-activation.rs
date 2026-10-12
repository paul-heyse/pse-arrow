// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native codec/effect bridge. Invoke only through scripts/preparation_activation.py.
#![allow(
    clippy::print_stdout,
    reason = "the private supervisor pipe carries bounded native failure control"
)]
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
const OPERATION_BYTES: usize = 512 * 1024 * 1024;
// Three bounded non-Arrow owners: preparation, compact archive index, IPC scratch.
// The remaining128MiB stays available to actual native registry validation.
const OWNED_BYTES: usize = 384 * 1024 * 1024;
#[derive(Debug, Parser)]
struct Cli {
    state: PathBuf,
    archive: PathBuf,
    #[cfg(feature = "canonical-activation-fixtures")]
    #[arg(long)]
    private_fixture_seed: bool,
    #[cfg(feature = "canonical-activation-fixtures")]
    #[arg(long, requires = "private_fixture_seed")]
    private_fixture_nonpreparation: bool,
    #[cfg(feature = "canonical-activation-fixtures")]
    #[arg(long, requires = "private_fixture_seed")]
    private_fixture_history: bool,
    #[cfg(feature = "canonical-activation-fixtures")]
    #[arg(long, conflicts_with = "private_fixture_seed")]
    private_fixture_read_history: bool,
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    let nonzero = |value| {
        std::num::NonZeroUsize::new(value)
            .ok_or_else(|| anyhow::anyhow!("positive activation resource bound required"))
    };
    let resources = pse_engine::resources::EngineResources::build(
        nonzero(OPERATION_BYTES)?,
        nonzero(8)?,
        std::env::temp_dir(),
        1 << 20,
        pse_engine::cache_service::CacheBudget::disabled(1024),
    )?;
    let pool = resources.pool.clone();
    let factory = Arc::new(pse_engine::session::EngineFactory::new(
        resources.runtime.clone(),
        pool.clone(),
        pse_engine::session::ExecutionSettings::default(),
        pse_engine::session::ThreadBudget {
            pool_threads: nonzero(2)?,
            target_partitions: nonzero(1)?,
        },
        pse_engine::session::native_engine_profile(),
    )?);
    let model = factory.bind_model(pse_schema::shared_registry()?)?;
    let allocation =
        pse_columnar::MemoryConsumer::new("offline-preparation:complete-operation").register(&pool);
    allocation.try_grow(OWNED_BYTES)?;
    let allocation = Arc::new(allocation);
    let rebuilder =
        pse_runtime::workflow::HistoricalResultRebuilder::new(model, allocation.clone())?;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    #[cfg(feature = "canonical-activation-fixtures")]
    let result = if cli.private_fixture_read_history {
        executor.block_on(read_private_history(&cli.state, &rebuilder))
    } else if cli.private_fixture_seed {
        let producer = |trajectory, attempt: &str, ordinal, origin| {
            rebuilder.fixture_block(trajectory, attempt, ordinal, origin)
        };
        executor.block_on(
            pse_operations::canonical_preparation_activation_effect::seed_private_fixture(
                &cli.state,
                allocation.clone(),
                cli.private_fixture_nonpreparation,
                if cli.private_fixture_history {
                    Some(&producer)
                } else {
                    None
                },
            ),
        )
    } else {
        executor.block_on(
            pse_operations::canonical_preparation_activation_effect::run(
                &cli.state,
                &cli.archive,
                allocation.clone(),
                &rebuilder,
            ),
        )
    };
    #[cfg(not(feature = "canonical-activation-fixtures"))]
    let result = executor.block_on(
        pse_operations::canonical_preparation_activation_effect::run(
            &cli.state,
            &cli.archive,
            allocation.clone(),
            &rebuilder,
        ),
    );
    // Keep the prepaid owner through driver/runtime drain on failure as well.
    drop(executor);
    drop(allocation);
    if let Err(error) = &result {
        println!(
            "{}",
            serde_json::json!({"event":"failed","reason":error.to_string()})
        );
    }
    result?;
    Ok(())
}

#[cfg(feature = "canonical-activation-fixtures")]
async fn read_private_history(
    state: &std::path::Path,
    rebuilder: &pse_runtime::workflow::HistoricalResultRebuilder,
) -> Result<(), pse_operations::canonical::CanonicalError> {
    use pse_operations::{
        canonical::{CanonicalError, CanonicalOptions, CanonicalStore},
        canonical_execution::execution_attempt_key,
        canonical_preparation_activation::HistoricalResultRebuilder as _,
    };
    let fail = |message: &str| CanonicalError::Configuration(message.into());
    let store = CanonicalStore::connect(&CanonicalOptions::from_state(state)?).await?;
    store.open().await?;
    let attempt = execution_attempt_key("original-run", "sealed-original");
    let read = store
        .read_results("original-run", &attempt, std::time::Duration::from_secs(60))
        .await?;
    if read.run().interpretation != "pse.substrate.v3"
        || read.attempt().generation != 4
        || read.sets().len() != 2
    {
        return Err(fail("historical public read provenance differs"));
    }
    for trajectory in [false, true] {
        let (_, expected, _) = rebuilder.fixture_block(trajectory, &attempt, 0, 0)?;
        let set = read
            .sets()
            .iter()
            .find(|set| set.name == expected.output)
            .ok_or_else(|| fail("historical public set absent"))?;
        let output = if trajectory {
            format!(
                "{}:{}:value",
                set.name,
                pse_ids::SemanticId::from_bytes([78; 16])
            )
        } else {
            format!("{}:{}", set.name, pse_ids::SemanticId::from_bytes([78; 16]))
        };
        let partition = if trajectory { "0" } else { "step:4000000001" };
        let page = store
            .result_output_page(
                &read, &set.key, &output, partition, 0, 1, None, trajectory, None, None, None,
            )
            .await?;
        let metadata = page
            .blocks
            .first()
            .cloned()
            .ok_or_else(|| fail("historical public block absent"))?;
        let block = store.result_block(&read, metadata).await?;
        let projection = rebuilder.rebuild(&set.name, &block.batch, &block.metadata)?;
        if trajectory {
            if page.outputs.len() != 1
                || page.outputs[0].start != 0
                || page.outputs[0].end != 4096
                || projection.outputs[0].output != page.outputs[0].output
            {
                return Err(fail("historical dense public read differs"));
            }
        } else {
            let locator = page
                .rows
                .first()
                .ok_or_else(|| fail("historical scalar public locator absent"))?;
            if projection.rows.iter().find(|row| row.offset == 0) != Some(locator)
                || locator.step != Some(4_000_000_001)
            {
                return Err(fail(
                    "historical scalar public read differs from original IPC",
                ));
            }
        }
    }
    for (run, operation) in [
        ("original-run", "failed-original"),
        ("failed-only-original-run", "failed-only-original"),
        ("retired-original-run", "retired-original-claim"),
    ] {
        if store
            .read_results(
                run,
                &execution_attempt_key(run, operation),
                std::time::Duration::from_secs(60),
            )
            .await
            .is_ok()
        {
            return Err(fail(
                "incomplete/retired historical selection gained read permission",
            ));
        }
    }
    for (operation, class) in [
        ("partial-original", "partial"),
        ("cancelled-original", "cancelled"),
    ] {
        let previous = store
            .read_results(
                "original-run",
                &execution_attempt_key("original-run", operation),
                std::time::Duration::from_secs(60),
            )
            .await?;
        if previous.attempt().outcome.as_deref() != Some(class) || previous.sets().len() != 1 {
            return Err(fail("historical prior attempt terminal class changed"));
        }
    }
    drop(read);
    store.disconnect().await?;
    println!(
        "{}",
        serde_json::json!({"event":"complete","historical_public_reader":"original-ipc-verified","incomplete":"ineligible","retired":"ineligible"})
    );
    Ok(())
}
