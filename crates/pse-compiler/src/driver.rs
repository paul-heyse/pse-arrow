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
    CompilerError, ExternalInputs, PolicySet,
    memo::Memo,
    passes::{
        PassOutput, PassStatus, StageKey,
        dag::{StageDag, invalid},
        registry::PassRegistry,
    },
};
use pse_catalog::{
    Catalog, Snapshot,
    session::SessionFactory,
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_ids::{CancellationToken, SnapshotKind};
use pse_schema::{Registry, model::PassSpec};
use std::{collections::BTreeMap, sync::Arc};

/// An explicit immutable pipeline request. P10 requires a complete fixture registry.
#[derive(Clone, Debug)]
pub struct PipelineRequest {
    /// Last requested available stage.
    pub through: String,
    /// Exact admitted source tip with model/case parent context.
    pub snapshot: Arc<Snapshot>,
    /// Actual selected policy rows.
    pub policies: PolicySet,
    /// Explicit external inputs; used only when `fixture_mode` is enabled.
    pub external_bindings: ExternalInputs,
    /// Enable complete externally supplied predecessor fixtures.
    pub fixture_mode: bool,
    /// Bypass memo lookup for clean-execution comparisons.
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
    passes: PassRegistry,
    memo: Memo,
}
impl Driver {
    /// Construct the controller over one existing catalog and shared engine factory.
    /// # Errors
    /// The registry's declared stage graph is not closed and acyclic.
    pub fn new(
        catalog: Arc<Catalog>,
        sessions: Arc<SessionFactory>,
    ) -> Result<Self, CompilerError> {
        StageDag::build(catalog.registry())?;
        let mut passes = PassRegistry::new();
        passes.register(
            Arc::new(crate::passes::p3::P3::new(catalog.registry())?),
            catalog.registry(),
        )?;
        Ok(Self {
            catalog,
            sessions,
            passes,
            memo: Memo::new(64),
        })
    }
    /// Register a bounded implementation against its complete declaration.
    /// # Errors
    /// Unknown, mismatched or duplicate pass implementation.
    pub fn register(&mut self, pass: Arc<dyn crate::Pass>) -> Result<(), CompilerError> {
        self.passes.register(pass, self.catalog.registry())
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
        if !request.fixture_mode && !request.external_bindings.bindings.is_empty() {
            return Err(invalid(
                "external predecessor bindings require explicit fixture mode",
            ));
        }
        if request.through == "P10" && !request.fixture_mode {
            return Err(invalid(
                "P10 requires a complete predecessor fixture in Wave 1",
            ));
        }
        let registry = Arc::clone(self.catalog.registry());
        let dag = StageDag::build(&registry)?;
        let mut stages = BTreeMap::new();
        let mut report = PipelineReport::default();
        for spec in dag.through(&request.through)? {
            let attempt = crate::records::Attempt::new(spec);
            let work = match self.execute_stage(spec, &request, &stages, cancel).await {
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
                        plans: &[],
                    },
                    cancel,
                )
                .await?;
            if work.status == PassStatus::Ok {
                self.catalog
                    .write_stage_hint_with_context(
                        work.key.content_hash(),
                        &work.inputs,
                        &work.output,
                        &record,
                        Some(&work.context.value),
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
    async fn publish_output(
        &self,
        spec: &PassSpec,
        inputs: &crate::InputBundle,
        output: &PassOutput,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CompilerError> {
        let context = AdmissionContext {
            parents: inputs
                .ports
                .iter()
                .filter_map(|(port, input)| {
                    input
                        .as_ref()
                        .map(|input| (port.to_string(), Arc::clone(input.snapshot())))
                })
                .collect(),
            stage_pass: Some(spec.id),
        };
        let manifest = self
            .catalog
            .manifest_template(SnapshotKind::Stage, &context)?;
        let relations = spec
            .outputs
            .iter()
            .map(|port| {
                let relation = self
                    .registry()
                    .relation(&port.relation)
                    .ok_or_else(|| invalid("output relation absent"))?;
                Ok((
                    port.port.to_owned(),
                    RelationDraft {
                        contract: Arc::new(pse_catalog::RelationContract::from_spec(
                            self.registry(),
                            relation,
                            pse_catalog::EncodingPolicy::IpcFile,
                        )?),
                        batches: output.ports[port.port].clone(),
                    },
                ))
            })
            .collect::<Result<_, CompilerError>>()?;
        Ok(self
            .catalog
            .publish_bundle(
                BundleDraft {
                    manifest,
                    relations,
                    context,
                },
                cancel,
            )
            .await?)
    }
}
