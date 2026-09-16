// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stage reuse and publication retain the same actual producer admission boundary.
use super::Driver;
use crate::{CompilerError, InputBundle, PassContext, PassStatus, StageKey};
use pse_catalog::{
    Snapshot,
    store::{
        invocation::{InvocationContext, OwnedInvocation},
        stage::StageInputs,
    },
};
use pse_ids::CancellationToken;
use pse_schema::{Registry, model::PassSpec};
use std::{collections::BTreeMap, sync::Arc};

/// Owners shared only within one immutable pipeline request.
#[derive(Default)]
pub(super) struct InvocationState {
    physical: Arc<crate::validator::physical::PhysicalInventories>,
    invocation: Option<OwnedInvocation>,
}

impl InvocationState {
    fn capture(
        &mut self,
        request: &super::PipelineRequest,
        ctx: &PassContext<'_>,
    ) -> Result<OwnedInvocation, CompilerError> {
        let slot = &mut self.invocation;
        if let Some(invocation) = slot {
            if invocation
                .engine
                .as_ref()
                .is_none_or(|engine| !ctx.session.matches_semantic_inputs(engine))
            {
                return Err(crate::passes::dag::invalid(
                    "pipeline request changed its sealed engine semantics",
                ));
            }
            return Ok(invocation.clone());
        }
        ctx.cancel.checkpoint()?;
        let mut work = ctx.reserver.open("compiler:invocation-engine-inputs");
        work.try_grow(ctx.session.semantic_inputs_extent()?)
            .map_err(pse_ids::CanonError::from)?;
        let engine = ctx.session.semantic_inputs();
        let invocation = InvocationContext::capture(
            request.policies.0.iter().map(|(role, policy)| {
                (
                    role.as_str(),
                    policy.policy_id,
                    policy.input.snapshot(),
                    policy.input.relation().member().port.as_str(),
                )
            }),
            Some(&request.snapshot),
            Some(&engine),
            ctx.reserver,
        )?;
        *slot = Some(invocation.clone());
        Ok(invocation)
    }
}

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
        let checked_inputs = inputs.checked_rows(registry)?;
        let rows = checked_inputs
            .iter()
            .map(|(key, input)| (*key, input.batch().clone()))
            .collect();
        let validation =
            self.sessions
                .candidate_checked(checked_inputs, Arc::clone(registry), cancel)?;
        let checked = crate::passes::p2::validate(&rows, &validation, registry, cancel).await?;
        if checked.error_count() != 0 {
            return Err(pse_rules::RuleError::InvariantViolations {
                count: checked.error_count(),
                findings: checked.into_findings(),
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
        invocation: &InvocationContext,
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
            if hint.pass_id == spec.id && hint.inputs == exact_inputs {
                let (snapshot, _) = self.catalog.open_stage_hint(&hint, inputs, cancel).await?;
                if matches_invocation(
                    &snapshot,
                    inputs,
                    invocation,
                    self.registry(),
                    self.catalog.reserver().as_ref(),
                    cancel,
                )? {
                    return Ok(Some(snapshot));
                }
            }
        }
        Ok(None)
    }

    pub(super) async fn execute_stage(
        &mut self,
        spec: &PassSpec,
        request: &super::PipelineRequest,
        stages: &BTreeMap<String, Arc<Snapshot>>,
        documents: &pse_authoring::document::OwnedDocumentSet,
        state: &mut InvocationState,
        cancel: &CancellationToken,
    ) -> Result<CompletedStage, CompilerError> {
        cancel.checkpoint()?;
        let started = std::time::Instant::now();
        tracing::info!(pass = spec.name, "stage invocation started");
        let registry = Arc::clone(self.registry());
        let pinned = super::inputs::inventory(&request.snapshot, &registry)?;
        let inputs = super::inputs::bind(spec, &registry, &pinned, stages)?;
        let session = self
            .sessions
            .candidate_checked_ports(inputs.checked_ports(), Arc::clone(&registry), cancel)?
            .with_purpose(pse_schema::model::provider::OperationPurpose::Construct);
        self.validate_preconditions(spec, &inputs, &registry, cancel)
            .await?;
        let ctx = PassContext {
            physical: None,
            registry: &registry,
            documents,
            policies: &request.policies,
            cancel,
            reserver: self.sessions.reserver().as_ref(),
            session: &session,
        };
        let key = crate::passes::key::stage_key(spec, &inputs, &ctx)?;
        let invocation = state.capture(request, &ctx)?;
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
        let dependencies = crate::memo::Dependencies::capture(&inputs, &ctx)?;
        let mut cached = if request.reuse {
            self.memo.lookup(key, &dependencies, ctx.reserver, cancel)?
        } else {
            None
        };
        if request.reuse && cached.is_none() {
            cached = self
                .lookup_durable(key, spec, &stage_inputs, &invocation, cancel)
                .await?;
        }
        let (output, status, findings, derivations, plans) = if let Some(output) = cached {
            (output, PassStatus::Reused, vec![], vec![], vec![])
        } else {
            let context = pse_catalog::store::membership::AdmissionContext {
                traversal: Arc::default(),
                invocation: Some(invocation),
                parents: inputs
                    .ports
                    .iter()
                    .filter_map(|(role, input)| {
                        input
                            .as_ref()
                            .map(|input| (role.to_string(), Arc::clone(input.snapshot())))
                    })
                    .collect(),
                stage_pass: Some(spec.id),
            };
            let arguments = crate::validator::InvocationArguments {
                physical: Arc::clone(&state.physical),
                documents: documents.clone(),
                policies: request.policies.clone(),
                session: session.clone(),
            };
            let prepared =
                self.catalog
                    .prepare_production(context, Some(Arc::new(arguments)), cancel)?;
            let completed = prepared.execute(cancel).await?;
            tracing::info!(
                pass = spec.name,
                phase = "admission",
                elapsed_seconds = started.elapsed().as_secs_f64(),
                "stage obligations completed"
            );
            let published = self.catalog.publish_production(completed, cancel).await?;
            tracing::info!(
                pass = spec.name,
                phase = "publication",
                elapsed_seconds = started.elapsed().as_secs_f64(),
                "stage published"
            );
            (
                published.snapshot,
                PassStatus::Ok,
                published.findings,
                published.derivations,
                published.plans,
            )
        };
        Ok(CompletedStage {
            key,
            inputs: stage_inputs,
            dependencies,
            output,
            status,
            findings,
            derivations,
            plans,
            engine: Some(session.profile_hash()),
        })
    }
}

pub(super) struct CompletedStage {
    pub(super) key: StageKey,
    pub(super) inputs: StageInputs,
    pub(super) dependencies: crate::memo::Dependencies,
    pub(super) output: Arc<Snapshot>,
    pub(super) status: PassStatus,
    pub(super) findings: Vec<pse_relations::RecordBatch>,
    pub(super) derivations: Vec<pse_relations::RecordBatch>,
    pub(super) plans: Vec<pse_catalog::session::PlanObservation>,
    pub(super) engine: Option<pse_ids::ContentHash>,
}

fn matches_invocation(
    snapshot: &Snapshot,
    inputs: &StageInputs,
    expected: &InvocationContext,
    registry: &Registry,
    reserver: &dyn pse_ids::MemoryReserver,
    cancel: &CancellationToken,
) -> Result<bool, CompilerError> {
    let Some(actual) = snapshot.invocation() else {
        return Ok(false);
    };
    if !crate::memo::snapshots::same_invocation(actual, expected, registry, reserver, cancel)? {
        return Ok(false);
    }
    for (role, expected) in inputs {
        match (snapshot.parents().get(role), expected) {
            (None, None) => {}
            (Some(actual), Some(expected)) => {
                if !crate::memo::snapshots::same_snapshot(
                    actual, expected, registry, reserver, cancel,
                )? {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
    }
    Ok(true)
}
