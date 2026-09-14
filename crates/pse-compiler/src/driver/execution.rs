// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stage reuse and publication retain the same actual producer admission boundary.
use super::Driver;
use crate::{
    CompilerError, InputBundle, PassContext, PassOutput, PassStatus, StageKey,
    passes::bundle::validate_output,
};
use pse_catalog::{
    Snapshot,
    store::{stage::StageInputs, stage_context::StageContext},
};
use pse_ids::CancellationToken;
use pse_schema::{Registry, model::PassSpec};
use std::{collections::BTreeMap, sync::Arc};

impl Driver {
    pub(super) async fn validate_preconditions(
        &self,
        spec: &PassSpec,
        inputs: &InputBundle,
        registry: &Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<(), CompilerError> {
        if spec.preconditions.is_empty() {
            return Ok(());
        }
        let rows = inputs.rows(registry)?;
        let validation = self
            .sessions
            .candidate(rows.clone(), Arc::clone(registry), cancel)?;
        let checked = crate::passes::p2::validate(&rows, &validation, registry, cancel).await?;
        if checked.error_count != 0 {
            return Err(pse_rules::RuleError::InvariantViolations {
                count: checked.error_count,
                findings: checked.findings,
            }
            .into());
        }
        Ok(())
    }

    pub(super) async fn lookup_durable(
        &self,
        key: StageKey,
        spec: &PassSpec,
        inputs: &StageInputs,
        context: &StageContext,
        cancel: &CancellationToken,
    ) -> Result<Option<Arc<Snapshot>>, CompilerError> {
        let exact_inputs = inputs
            .iter()
            .map(|(port, input)| {
                (
                    port.clone(),
                    input.as_ref().map(|snapshot| snapshot.manifest_ref()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        for hint in self.catalog.stage_hints(key.content_hash(), cancel).await? {
            if hint.pass_id == spec.id
                && hint.inputs == exact_inputs
                && hint.context.as_ref() == Some(context)
            {
                let (snapshot, _) = self.catalog.open_stage_hint(&hint, inputs, cancel).await?;
                return Ok(Some(snapshot));
            }
        }
        Ok(None)
    }

    pub(super) async fn execute_stage(
        &mut self,
        spec: &PassSpec,
        request: &super::PipelineRequest,
        stages: &BTreeMap<String, Arc<Snapshot>>,
        cancel: &CancellationToken,
    ) -> Result<CompletedStage, CompilerError> {
        cancel.checkpoint()?;
        let registry = Arc::clone(self.registry());
        let pinned = super::inputs::inventory(&request.snapshot, &registry)?;
        let actual = super::inputs::row_inventory(&pinned);
        let documents = super::inputs::documents(&self.catalog, &actual, cancel).await?;
        let inputs =
            super::inputs::bind(spec, &registry, &pinned, stages, &request.external_bindings)?;
        let session = if spec.executes_plans {
            Some(
                self.sessions
                    .candidate(inputs.rows(&registry)?, Arc::clone(&registry), cancel)?,
            )
        } else {
            None
        };
        self.validate_preconditions(spec, &inputs, &registry, cancel)
            .await?;
        let ctx = PassContext {
            registry: &registry,
            documents: &documents,
            policies: &request.policies,
            external: &request.external_bindings,
            cancel,
            reserver: self.sessions.reserver().as_ref(),
            session: session.as_ref(),
        };
        let key = crate::passes::key::stage_key(spec, &inputs, &ctx)?;
        let context = crate::memo::context::capture(&ctx, spec.executes_plans)?;
        let stage_inputs = inputs
            .ports
            .iter()
            .map(|(port, input)| {
                (
                    port.to_string(),
                    input.as_ref().map(|input| Arc::clone(input.snapshot())),
                )
            })
            .collect();
        let dependencies = crate::memo::Dependencies::capture(&inputs, &ctx, spec.executes_plans)?;
        let mut cached = if request.reuse {
            self.memo.lookup(key, &dependencies)?
        } else {
            None
        };
        if request.reuse && cached.is_none() {
            cached = self
                .lookup_durable(key, spec, &stage_inputs, &context.value, cancel)
                .await?;
        }
        let (output, status, findings) = if let Some(output) = cached {
            (output, PassStatus::Reused, vec![])
        } else {
            let result = self.passes.get(spec.name)?.run(&ctx, &inputs).await?;
            let output = self.admit_output(spec, &inputs, &result, &ctx).await?;
            (output, PassStatus::Ok, result.findings)
        };
        Ok(CompletedStage {
            key,
            inputs: stage_inputs,
            context,
            dependencies,
            output,
            status,
            findings,
            engine: session
                .as_ref()
                .map(pse_catalog::session::SnapshotSession::profile_hash),
        })
    }

    async fn admit_output(
        &self,
        spec: &PassSpec,
        inputs: &InputBundle,
        output: &PassOutput,
        ctx: &PassContext<'_>,
    ) -> Result<Arc<Snapshot>, CompilerError> {
        if output.record.status == PassStatus::Failed {
            return Err(CompilerError::Postcondition {
                pass: spec.name.to_owned(),
                findings: output.findings.clone(),
            });
        }
        if output.record.status == PassStatus::Cancelled {
            return Err(CompilerError::Cancelled {
                findings: output.findings.clone(),
            });
        }
        if output.record.status != PassStatus::Ok {
            return Err(crate::passes::dag::invalid(
                "only the driver may claim reuse",
            ));
        }
        let result = async {
            validate_output(output, spec, ctx.registry)?;
            self.publish_output(spec, inputs, output, ctx.cancel).await
        }
        .await;
        result.map_err(|source| CompilerError::PassFailure {
            source: Box::new(source),
            findings: output.findings.clone(),
        })
    }
}

pub(super) struct CompletedStage {
    pub(super) key: StageKey,
    pub(super) inputs: StageInputs,
    pub(super) context: crate::memo::context::OwnedContext,
    pub(super) dependencies: crate::memo::Dependencies,
    pub(super) output: Arc<Snapshot>,
    pub(super) status: PassStatus,
    pub(super) findings: Vec<pse_relations::RecordBatch>,
    pub(super) engine: Option<pse_ids::ContentHash>,
}
