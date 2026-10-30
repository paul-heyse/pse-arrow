// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One configured connection and its protected typed page through encoding and joined drain.
use super::{EncodedResponse, Payload, ReaderError, encode_response, response};
use crate::workflow::{
    AttemptObservationRequest, RetainedDataServices, StudyObservationCursor,
    StudyObservationRequest,
};
use datafusion::execution::{
    disk_manager::{DiskManagerBuilder, DiskManagerMode},
    memory_pool::{GreedyMemoryPool, MemoryPool},
    runtime_env::RuntimeEnvBuilder,
    session_state::SessionStateBuilder,
};
use pse_engine::session::{BoundModel, EngineFactory};
use pse_operations::canonical::{CanonicalOptions, CanonicalStore};
use std::{
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

/// Explicit existing canonical deployment and this observer's finite read allocation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalReadSelection {
    /// Existing private deployment state; this route never starts its configured database.
    pub state: PathBuf,
    /// Existing selected database, absent to retain the configured selection.
    pub database: Option<String>,
    /// Explicit read allocation shared by native metadata, validation and progress decoding.
    pub memory_bytes: NonZeroUsize,
    /// One original observation/drain allowance, 1..=25000ms, below the transport deadline.
    pub timeout_ms: u64,
}
impl CanonicalReadSelection {
    pub(super) fn validate(&self) -> Result<(), ReaderError> {
        if !self.state.is_absolute() || !(1..=25_000).contains(&self.timeout_ms) {
            return Err(super::invalid(
                "canonical read requires absolute existing state and a 1..=25000ms observer allowance",
            ));
        }
        Ok(())
    }
}
pub(super) enum Read {
    Study {
        request: StudyObservationRequest,
        cursor: Option<StudyObservationCursor>,
    },
    Attempt {
        request: AttemptObservationRequest,
    },
}
fn engine(error: pse_engine::EngineError) -> ReaderError {
    ReaderError::Workflow(Box::new(error.into()))
}
fn workflow(error: crate::workflow::WorkflowError) -> ReaderError {
    ReaderError::Workflow(Box::new(error))
}
fn canonical(error: pse_operations::canonical::CanonicalError) -> ReaderError {
    ReaderError::Canonical(Box::new(error))
}
pub(super) fn model(bytes: NonZeroUsize) -> Result<BoundModel, ReaderError> {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(bytes.get()));
    let environment = RuntimeEnvBuilder::new()
        .with_memory_pool(pool.clone())
        .with_disk_manager_builder(
            DiskManagerBuilder::default().with_mode(DiskManagerMode::Disabled),
        )
        .build_arc()
        .map_err(|error| engine(error.into()))?;
    let factory = Arc::new(
        EngineFactory::from_builder(
            environment,
            pool,
            "passive-retained-observation",
            SessionStateBuilder::new_with_default_features(),
        )
        .with_observation(pse_engine::session::assurance::ObservationPolicy::Contract),
    );
    factory
        .bind_model(
            pse_schema::shared_registry()
                .map_err(|error| engine(pse_engine::EngineError::Semantic(Arc::new(error))))?,
        )
        .map_err(engine)
}
pub(super) fn encode(
    correlation: String,
    selection: CanonicalReadSelection,
    read: Read,
) -> Result<EncodedResponse, ReaderError> {
    let deadline = Instant::now() + Duration::from_millis(selection.timeout_ms);
    let result = (|| {
        let options = CanonicalOptions::from_state_for_database(
            &selection.state,
            selection.database.as_deref(),
        )
        .map_err(canonical)?;
        let model = model(selection.memory_bytes)?;
        let executor = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        executor.block_on(pse_operations::canonical::operation_scope(
            deadline,
            async {
                let store = CanonicalStore::connect(&options).await.map_err(canonical)?;
                let operation = async {
                    store.open().await.map_err(canonical)?;
                    let services =
                        RetainedDataServices::new(store.clone(), model).map_err(workflow)?;
                    let output = async {
                        let payload = match read {
                            Read::Study { request, cursor } => {
                                let mut observer = services
                                    .observe_study(request, cursor)
                                    .await
                                    .map_err(workflow)?;
                                let observed = observer.poll().await.map_err(workflow);
                                observer.close();
                                Payload::Study {
                                    page: Box::new(observed?),
                                }
                            }
                            Read::Attempt { request } => Payload::Attempt {
                                page: Box::new(
                                    services.observe_attempt(request).await.map_err(workflow)?,
                                ),
                            },
                        };
                        let owned = response(correlation.clone(), Ok(payload));
                        // The original page's protected read/capture/allocation is live here.
                        // No detached generic JSON copy replaces that custody boundary.
                        let encoded = encode_response(&owned);
                        drop(owned);
                        encoded
                    }
                    .await;
                    // The typed page was encoded and dropped before closing admission.
                    services.close();
                    drop(services);
                    output
                }
                .await;
                let cleanup = store
                    .drain_owned_connection_until(tokio::time::Instant::from_std(deadline))
                    .await
                    .map_err(canonical);
                match (operation, cleanup) {
                    (result, Ok(())) => result,
                    (Ok(_), Err(error)) => Err(error),
                    (Err(operation), Err(cleanup)) => Err(ReaderError::Settlement {
                        operation: Box::new(operation),
                        cleanup: Box::new(cleanup),
                    }),
                }
            },
        ))
    })();
    match result {
        Ok(output) => Ok(output),
        Err(error) => encode_response(&response(correlation, Err(error))),
    }
}
