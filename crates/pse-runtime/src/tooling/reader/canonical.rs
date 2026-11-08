// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One configured connection and its protected typed page through encoding and joined drain.
use super::{EncodedResponse, Payload, ReaderError, encode_response, response_with_policy};
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
use pse_host::envelope::RedactionPolicy;
use pse_operations::canonical::{CanonicalOptions, CanonicalStore};
use std::{
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
pub(super) mod project;

pub(super) const MAXIMUM_READ_MILLIS: u64 = 25_000;

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
        if !self.state.is_absolute() || !(1..=MAXIMUM_READ_MILLIS).contains(&self.timeout_ms) {
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

/// Redact both original configured credentials along with the operator's environment inputs.
pub fn canonical_redaction_policy(options: &CanonicalOptions) -> RedactionPolicy {
    redaction_policy_with_credentials([
        ("PSE_CANONICAL_PASSWORD", options.password.as_str()),
        (
            "PSE_CANONICAL_SELECTION_PASSWORD",
            options.selection_password.as_str(),
        ),
    ])
}

pub(super) fn redaction_policy_with_credentials<'a>(
    credentials: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> RedactionPolicy {
    let mut environment: pse_host::environment::Environment = std::env::vars_os().collect();
    for (name, value) in credentials {
        let mut name = std::ffi::OsString::from(name);
        while environment.contains_key(&name) {
            name.push("_");
        }
        environment.insert(name, value.into());
    }
    RedactionPolicy::from_inputs(&environment)
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
    cancel: &pse_columnar::CancellationToken,
) -> Result<EncodedResponse, ReaderError> {
    let deadline = Instant::now() + Duration::from_millis(selection.timeout_ms);
    let mut policy = RedactionPolicy::from_environment();
    let result = (|| {
        cancel.checkpoint()?;
        let options = CanonicalOptions::from_state_for_database(
            &selection.state,
            selection.database.as_deref(),
        )
        .map_err(canonical)?;
        policy = canonical_redaction_policy(&options);
        let model = model(selection.memory_bytes)?;
        connected(&options, deadline, cancel, async |store| {
            let services = RetainedDataServices::new(store.clone(), model).map_err(workflow)?;
            let output = async {
                cancel.checkpoint()?;
                let payload = match read {
                    Read::Study { request, cursor } => {
                        let mut observer = cancel
                            .until_cancelled(services.observe_study(request, cursor))
                            .await?
                            .map_err(workflow)?;
                        let observed = cancel
                            .until_cancelled(observer.poll())
                            .await
                            .map_err(ReaderError::from)
                            .and_then(|result| result.map_err(workflow));
                        observer.close();
                        Payload::Study {
                            page: Box::new(observed?),
                        }
                    }
                    Read::Attempt { request } => Payload::Attempt {
                        page: Box::new(
                            cancel
                                .until_cancelled(services.observe_attempt(request))
                                .await?
                                .map_err(workflow)?,
                        ),
                    },
                };
                cancel.checkpoint()?;
                // The original page's protected read/capture/allocation is live here.
                encode_response(response_with_policy(
                    correlation.clone(),
                    Ok(payload),
                    &policy,
                ))
            }
            .await;
            // The typed page was encoded and dropped before closing admission.
            services.close();
            drop(services);
            output
        })
    })();
    match result {
        Ok(output) => Ok(output),
        Err(error) => encode_response(response_with_policy(correlation, Err(error), &policy)),
    }
}

fn connected(
    options: &CanonicalOptions,
    deadline: Instant,
    cancel: &pse_columnar::CancellationToken,
    read: impl AsyncFnOnce(&CanonicalStore) -> Result<EncodedResponse, ReaderError>,
) -> Result<EncodedResponse, ReaderError> {
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    executor.block_on(pse_operations::canonical::operation_scope(
        deadline,
        async {
            cancel.checkpoint()?;
            // Connect only to the already selected deployment. Acquisition retains its
            // native clock and partial-acquisition owner until a store can be drained.
            let store = CanonicalStore::connect(options).await.map_err(canonical)?;
            let operation = async {
                cancel.checkpoint()?;
                store.open().await.map_err(canonical)?;
                read(&store).await
            }
            .await;
            let cleanup = store
                .drain_owned_connection_until(tokio::time::Instant::from_std(deadline))
                .await
                .map_err(canonical);
            match (operation, cleanup) {
                (result, Ok(())) => result,
                (Ok(output), Err(error)) => Err(output.settlement_failed(error)),
                (Err(operation), Err(cleanup)) => Err(ReaderError::Settlement {
                    operation: Box::new(operation),
                    cleanup: Box::new(cleanup),
                }),
            }
        },
    ))
}
