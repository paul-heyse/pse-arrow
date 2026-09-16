// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pipeline driver (blueprint §14.3).
//!
//! Owns the catalog and the shared runtime, commits document bundles and runs the declared
//! pass DAG through a requested stage. `miette::Result` and the graphical reporter live
//! here and in the CLI, never in a library crate (blueprint §23.2).
//!
mod commit;
mod execution;
mod history;
pub(crate) mod inputs;
pub use commit::{CommitBase, CommitReport, CommitRequest, CommitRevisionIds};
pub use inputs::BaseReader;

use crate::{
    CompilerError, PolicySet,
    memo::Memo,
    passes::{PassStatus, StageKey, dag::StageDag},
};
use pse_catalog::{Catalog, Snapshot, session::SessionFactory};
use pse_ids::CancellationToken;
use pse_schema::Registry;
use std::{collections::BTreeMap, sync::Arc};

/// An explicit immutable pipeline request over the registered production graph.
#[derive(Clone, Debug)]
pub struct PipelineRequest {
    /// Last requested available stage.
    pub through: String,
    /// Exact admitted source tip with model/case parent context.
    pub snapshot: Arc<Snapshot>,
    /// Actual selected policy rows.
    pub policies: PolicySet,
    /// Reuse admitted results when their actual dependencies match.
    pub reuse: bool,
}
/// One completed stage and its actual outcome.
#[derive(Clone, Debug)]
pub struct StageResult {
    /// Registered pass name.
    pub pass: String,
    /// Diagnostic lookup identity; actual dependencies were also checked on reuse.
    pub key: StageKey,
    /// Admitted complete immutable output.
    pub snapshot: Arc<Snapshot>,
    /// Whether this attempt recomputed or reused.
    pub status: PassStatus,
    /// Complete admitted terminal observation of this exact attempt.
    pub record: pse_catalog::store::sidecar::SidecarArtifact,
}
/// Complete outputs of the requested available dependency closure.
#[derive(Clone, Debug, Default)]
pub struct PipelineReport {
    /// Dependency-first stage results.
    pub stages: Vec<StageResult>,
}

/// Compiler controller sharing the store's actual registry and accounted session resources.
#[derive(Debug)]
pub struct Driver {
    catalog: Arc<Catalog>,
    sessions: Arc<SessionFactory>,
    memo: Memo,
}
impl Driver {
    /// Construct the controller over one existing catalog and shared engine factory.
    /// # Errors
    /// The registry's declared stage graph is not closed and acyclic.
    pub fn new(catalog: Arc<Catalog>) -> Result<Self, CompilerError> {
        StageDag::build(catalog.registry())?;
        let sessions = Arc::clone(catalog.session_factory());
        Ok(Self {
            catalog,
            sessions,
            memo: Memo::new(64),
        })
    }
    /// Exact store registry authority.
    pub fn registry(&self) -> &Arc<Registry> {
        self.catalog.registry()
    }
    /// Execute the declared graph, checking actual input/output values before identity or reuse.
    /// # Errors
    /// Unavailable stage, undeclared read, semantic failure, budget/cancel or atomic publication failure.
    pub async fn run(
        &mut self,
        request: PipelineRequest,
        cancel: &CancellationToken,
    ) -> Result<PipelineReport, CompilerError> {
        let registry = Arc::clone(self.catalog.registry());
        let dag = StageDag::build(&registry)?;
        let mut documents = None;
        let mut stages = BTreeMap::new();
        let mut state = execution::InvocationState::default();
        let mut report = PipelineReport::default();
        for spec in dag.through(&request.through)? {
            let attempt = crate::records::Attempt::new(spec);
            let execution = async {
                if documents.is_none() {
                    let source_inputs = inputs::inventory(&request.snapshot, &registry)?;
                    documents = Some(inputs::documents(
                        &self.catalog,
                        &inputs::row_inventory(&source_inputs),
                        cancel,
                    )?);
                }
                let documents = documents.as_ref().ok_or_else(|| {
                    crate::passes::dag::invalid("pipeline source preparation omitted its owner")
                })?;
                self.execute_stage(spec, &request, &stages, documents, &mut state, cancel)
                    .await
            }
            .await;
            let work = match execution {
                Ok(work) => work,
                Err(error) => {
                    return Err(attempt
                        .failed(&self.catalog, Some(request.snapshot.snapshot_id()), error)
                        .await);
                }
            };
            let record = attempt
                .success(
                    &self.catalog,
                    crate::records::Observation {
                        status: work.status,
                        input: Some(request.snapshot.snapshot_id()),
                        output: Some(work.output.snapshot_id()),
                        engine: work.engine,
                        findings: &work.findings,
                        derivations: &work.derivations,
                        plans: &work.plans,
                    },
                    cancel,
                )
                .await?;
            if work.status == PassStatus::Ok && work.dependencies.reusable()? {
                self.catalog
                    .write_stage_hint(
                        work.key.content_hash(),
                        &work.inputs,
                        &work.output,
                        &record,
                        cancel,
                    )
                    .await
                    .map_err(|source| CompilerError::AuxiliaryHint {
                        pass_run_id: attempt.id,
                        output: work.output.snapshot_id(),
                        source: Box::new(source.into()),
                    })?;
            }
            self.memo
                .insert(work.key, work.dependencies, Arc::clone(&work.output));
            stages.insert(spec.name.to_owned(), Arc::clone(&work.output));
            report.stages.push(StageResult {
                pass: spec.name.to_owned(),
                key: work.key,
                snapshot: work.output,
                status: work.status,
                record,
            });
        }
        Ok(report)
    }
}
