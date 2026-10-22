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
            .worker_threads(2)
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
) -> Result<()> {
    anyhow::ensure!(
        std::fs::metadata(request)?.len() <= 64 * 1024,
        "graph request exceeds its input bound"
    );
    let request: pse_operations::canonical_explanations::CanonicalGraphRequest =
        serde_json::from_slice(&std::fs::read(request)?)?;
    let options = CanonicalOptions::from_state_for_database(state, database)?;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    let result = executor.block_on(async {
        let store = CanonicalStore::connect(&options).await?;
        let result = async {
            let neighborhood = store.graph_explanation(&request).await?;
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
