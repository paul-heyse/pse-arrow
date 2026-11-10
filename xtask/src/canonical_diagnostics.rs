// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Inspection consumes the production-owned query and existing connection drain.

use anyhow::Result;
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore},
    canonical_studies::StudyCandidatesQuery,
};
use std::path::Path;

pub(crate) fn inspect(
    state: &Path,
    database: Option<&str>,
    study: &str,
    after: Option<u64>,
    experiment: bool,
    describe_only: bool,
) -> Result<()> {
    let result = if describe_only {
        StudyCandidatesQuery::new(study, after)?.description()
    } else {
        let options = CanonicalOptions::from_state_for_database(state, database)?;
        let executor = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let hub = pse_diagnostics::observation::ObservationHub::new(
            pse_diagnostics::observation::Policy::default(),
        );
        let operation = hub.start("canonical-query-inspection");
        let reader = operation.reader();
        let observed = executor.block_on(pse_diagnostics::observation::scope(
            Some(operation.context()),
            async {
                let store = CanonicalStore::connect(&options).await?;
                let result = async {
                    store.open().await?;
                    let mut result = store
                        .inspect_study_candidates(study, after, experiment)
                        .await?;
                    result["module"] = store.inspect_study_module().await?;
                    Ok::<_, pse_operations::canonical::CanonicalError>(result)
                }
                .await;
                let drained = store.disconnect().await;
                let output = result?;
                drained?;
                Ok::<_, pse_operations::canonical::CanonicalError>(output)
            },
        ));
        operation.finish(if observed.is_ok() {
            pse_diagnostics::observation::Outcome::Completed
        } else {
            pse_diagnostics::observation::Outcome::Failed
        });
        let mut result = observed?;
        result["statistics"] =
            serde_json::to_value(reader.snapshot(std::time::Duration::from_secs(60)))?;
        result
    };
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

/// One operator consumer of the semantic product, with optional independent native plan output.
pub(crate) fn graph(
    state: &Path,
    database: Option<&str>,
    request: &Path,
    explain: bool,
    experiment: bool,
    paths: bool,
    describe_only: bool,
) -> Result<()> {
    anyhow::ensure!(
        std::fs::metadata(request)?.len() <= 64 * 1024,
        "graph request exceeds its input bound"
    );
    let bytes = std::fs::read(request)?;
    if paths {
        let request: pse_operations::canonical_explanations::CanonicalGraphPathsRequest =
            pse_model::document::decode_versioned::<_, 1>(&bytes)?;
        if describe_only {
            println!("{}", serde_json::to_string(&request.description()?)?);
            return Ok(());
        }
        let options = CanonicalOptions::from_state_for_database(state, database)?;
        let executor = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let output = executor.block_on(async {
            let store = CanonicalStore::connect(&options).await?;
            let result = async {
                store.open().await?;
                let paths = if explain || experiment {
                    None
                } else {
                    let mut read = store.begin_graph_paths(&request).await?;
                    let mut pages = Vec::new();
                    while let Some(page) = read.next_page()? {
                        pages.push(page);
                    }
                    Some(pages)
                };
                let plan = if explain || experiment {
                    Some(store.inspect_graph_paths(&request, experiment).await?)
                } else {
                    None
                };
                Ok::<_, pse_operations::canonical::CanonicalError>(
                    serde_json::json!({"paths":paths,"physical":plan}),
                )
            }
            .await;
            let drain = store.disconnect().await;
            let result = result?;
            drain?;
            Ok::<_, pse_operations::canonical::CanonicalError>(result)
        })?;
        println!("{}", serde_json::to_string(&output)?);
        return Ok(());
    }
    let request: pse_operations::canonical_explanations::CanonicalGraphRequest =
        pse_model::document::decode_versioned::<_, 1>(&bytes)?;
    if describe_only {
        println!("{}", serde_json::to_string(&request.description()?)?);
        return Ok(());
    }
    let options = CanonicalOptions::from_state_for_database(state, database)?;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result = executor.block_on(async {
        let store = CanonicalStore::connect(&options).await?;
        let result = async {
            store.open().await?;
            let neighborhood = if explain || experiment {
                None
            } else {
                Some(store.graph_explanation(&request).await?)
            };
            let plan = if explain || experiment {
                Some(
                    store
                        .inspect_graph_explanation(&request, experiment)
                        .await?,
                )
            } else {
                None
            };
            Ok::<_, pse_operations::canonical::CanonicalError>(
                serde_json::json!({"neighborhood":neighborhood,"physical":plan}),
            )
        }
        .await;
        let drained = store.disconnect().await;
        let output = result?;
        drained?;
        Ok::<_, pse_operations::canonical::CanonicalError>(output)
    })?;
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
