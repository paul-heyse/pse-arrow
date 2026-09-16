// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Realize actual selected method contracts through the shared finite configuration and P7 engine.
mod kernels;
mod outputs;
mod parameters;
mod selections;

use super::{native_outputs::Sources, native_rows::workspace};
use crate::{CompilerError, InputBundle, PassContext};
use pse_catalog::computation::ProducedStage;
use pse_catalog::session::SnapshotSession;
use pse_ids::SemanticId;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry,
    model::{PassSpec, RelationKey},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

type Inputs = BTreeMap<RelationKey, FieldCheckedBatch>;

/// The declared method-realization pass; it adds no per-method numerical constructors.
#[derive(Debug)]
pub struct P9 {
    spec: PassSpec,
}
impl P9 {
    /// Bind the complete actual registered pass specification.
    /// # Errors
    /// The registry does not declare P9.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        Ok(Self {
            spec: registry
                .pass("P9@1")
                .ok_or_else(|| invalid("P9 is undeclared"))?
                .clone(),
        })
    }
}
impl crate::Pass for P9 {
    fn spec(&self) -> &PassSpec {
        &self.spec
    }
    fn run<'a>(
        &'a self,
        ctx: &'a PassContext<'a>,
        inputs: &'a InputBundle,
    ) -> pse_catalog::provider::BoxFut<'a, Result<ProducedStage, CompilerError>> {
        Box::pin(async move {
            inputs.validate(&self.spec, ctx.registry)?;
            let result = Box::pin(run(&self.spec, ctx, inputs)).await?;
            let ports =
                self.spec
                    .outputs
                    .iter()
                    .map(|port| {
                        let relation = ctx
                            .registry
                            .relation(&port.relation)
                            .ok_or_else(|| invalid("P9 output undeclared"))?;
                        let batch = result.rows.get(&relation.key).ok_or_else(|| {
                            invalid(format!("P9 output {} omitted", port.relation))
                        })?;
                        Ok((port.port.to_owned(), batch.clone()))
                    })
                    .collect::<Result<_, CompilerError>>()?;
            Ok(ProducedStage {
                outputs: ports,
                findings: Vec::new(),
                derivations: result.derivations,
                plans: Vec::new(),
            })
        })
    }
}
async fn run(
    spec: &PassSpec,
    ctx: &PassContext<'_>,
    inputs: &InputBundle,
) -> Result<super::p7::RealizationOutput, CompilerError> {
    let started = Instant::now();
    let original = inputs.checked_rows(ctx.registry)?;
    let mut work = ctx.reserver.open("P9:selected-method-workspace");
    let extent = original.values().try_fold(0_usize, |sum, batch| {
        sum.checked_add(pse_ids::validation_extent(batch.batch())?)
            .ok_or_else(|| invalid("selected method input extent overflow"))
    })?;
    work.try_grow(
        extent
            .checked_mul(6)
            .ok_or_else(|| invalid("selected method input extent overflow"))?,
    )
    .map_err(pse_ids::CanonError::from)?;
    let session = ctx.session;
    let (selection, _selection_arguments) =
        selections::select(session, ctx.registry, ctx.reserver, ctx.cancel).await?;
    trace_phase(started, "selection");
    let overrides =
        super::p3::configure_selected(inputs, &selection.roots, spec, session, ctx).await?;
    trace_phase(started, "configuration");
    let inferred = Box::pin(super::p4::evaluate_augmented(spec, ctx, inputs, &overrides)).await?;
    trace_phase(started, "inference");
    let mut augmented = original.clone();
    augmented.extend(
        overrides
            .iter()
            .map(|(key, input)| (*key, input.checked().clone())),
    );
    augmented.extend(
        inferred
            .relations
            .iter()
            .map(|(key, batch)| (*key, batch.clone())),
    );
    let mut sources = inferred.sources.clone();
    let session = workspace(session, &augmented, ctx.cancel)?;
    let selected =
        selected_instances(&selection, &augmented, &sources, spec, &session, ctx).await?;
    let bindings = parameters::bind(&selection, ctx, &session, &sources).await?;
    outputs::bind_parameters(&mut augmented, &mut sources, bindings, ctx, spec, &session).await?;
    let session = workspace(&session, &augmented, ctx.cancel)?;
    outputs::bind_methods(
        &mut augmented,
        &mut sources,
        &selection,
        &session,
        ctx,
        spec,
    )
    .await?;
    let session = workspace(&session, &augmented, ctx.cancel)?;
    let (kernels, _kernel_arguments) =
        kernels::prepare(&selection, ctx, &session, &sources).await?;
    trace_phase(started, "bindings");
    let mut output = super::p7::realize(
        &augmented,
        &sources,
        ctx,
        spec,
        &session,
        Some(super::p7::SelectedRealization {
            instances: &selected,
            kernels: &kernels,
        }),
    )
    .await?;
    trace_phase(started, "mathematics");
    let replaced = overrides
        .keys()
        .chain(inferred.relations.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    output.derivations = outputs::without_relations(output.derivations, &replaced, ctx)?;
    output.rows.extend(
        overrides
            .iter()
            .map(|(key, input)| (*key, input.checked().clone())),
    );
    for key in &replaced {
        let (_, actual) = sources
            .get(key)
            .ok_or_else(|| invalid("final selected-stage source absent"))?;
        output
            .sources
            .replace_location(*key, actual.location.clone())?;
    }
    output.rows.extend(inferred.relations);
    output.derivations.extend(inferred.derivations);
    for input in overrides.values() {
        output
            .derivations
            .push(input.derivations().clone().into_batch());
    }
    outputs::provisions(&selection, &augmented, spec, ctx, &mut output, &session).await?;
    trace_phase(started, "provisions");
    Ok(output)
}
fn trace_phase(started: Instant, phase: &str) {
    tracing::info!(
        pass = "P9",
        phase,
        elapsed_seconds = started.elapsed().as_secs_f64(),
        "selected method phase completed"
    );
}
async fn selected_instances(
    selection: &selections::Selection,
    inputs: &Inputs,
    sources: &Sources,
    pass: &PassSpec,
    session: &SnapshotSession,
    ctx: &PassContext<'_>,
) -> Result<BTreeSet<SemanticId>, CompilerError> {
    use crate::passes::native_construction::{
        Plans, c, distinct, filter, join, prefix, project, union,
    };
    use datafusion::{
        arrow::array::{Array, FixedSizeBinaryArray},
        logical_expr::{JoinType, col},
    };
    let mut plans = Plans::new(inputs, sources, pass, session, ctx.cancel)?;
    let roots = selection
        .roots
        .iter()
        .map(|root| plans.sid(root.instance.instance_id))
        .collect::<Result<Vec<_>, _>>()?;
    if roots.is_empty() {
        return Ok(BTreeSet::new());
    }
    let instances = plans.scan("inferred.instances", "instance")?;
    let root_instances = filter(
        instances.clone(),
        c("instance", "instance_id").in_list(roots, false),
    )?;
    let mut frontier = plans
        .retain(project(
            root_instances,
            [c("instance", "instance_id").alias("instance_id")],
        )?)
        .await?;
    let mut seen = frontier.clone();
    loop {
        ctx.cancel.checkpoint()?;
        let next = join(
            instances.clone(),
            prefix(frontier, "parent")?,
            JoinType::Inner,
            [c("instance", "parent_instance_id").eq(c("parent", "instance_id"))],
        )?;
        let next = project(next, [c("instance", "instance_id").alias("instance_id")])?;
        let next = join(
            next,
            prefix(seen.clone(), "seen")?,
            JoinType::LeftAnti,
            [col("instance_id").eq(c("seen", "instance_id"))],
        )?;
        let (next, count) = plans.retain_with_count(distinct(next)?).await?;
        if count == 0 {
            break;
        }
        seen = plans
            .retain(distinct(union(vec![seen, next.clone()])?)?)
            .await?;
        frontier = next;
    }
    let complete = plans
        .session
        .prepare_rule_plan(seen, ctx.cancel)?
        .execute(ctx.cancel)
        .await?;
    let mut ids = BTreeSet::new();
    for batch in complete.batches() {
        let values = batch
            .column(0)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .ok_or_else(|| invalid("selected instance ID storage differs"))?;
        for row in 0..values.len() {
            if values.is_null(row) {
                return Err(invalid("selected instance ID is null"));
            }
            ids.insert(
                SemanticId::try_from_slice(values.value(row))
                    .map_err(|_| invalid("selected instance ID width differs"))?,
            );
        }
    }
    for root in &selection.roots {
        if !ids.contains(&root.instance.instance_id) {
            return Err(invalid(
                "selected method root was not configured as an actual instance",
            ));
        }
    }
    Ok(ids)
}
fn invalid(detail: impl Into<String>) -> CompilerError {
    pse_templates::TemplateError::Binding {
        instance: SemanticId::NIL,
        detail: detail.into(),
    }
    .into()
}
fn unbound_kernel(
    method_id: SemanticId,
    kernel_id: Option<SemanticId>,
    detail: impl Into<String>,
) -> CompilerError {
    CompilerError::KernelUnbound {
        method_id,
        kernel_id,
        detail: detail.into(),
    }
}

pub(crate) fn kernel_output_id(
    scope: SemanticId,
    method: SemanticId,
    ordinal: u16,
    index: &[SemanticId],
) -> SemanticId {
    pse_ids::named_id(
        scope,
        &format!(
            "pse:kernel-output:v1:{}:{ordinal}:{}",
            method.to_hex(),
            index.iter().map(|id| id.to_hex()).collect::<String>()
        ),
    )
}

pub(crate) fn kernel_binding_id(
    scope: SemanticId,
    method: SemanticId,
    index: &[SemanticId],
) -> SemanticId {
    pse_ids::named_id(
        scope,
        &format!(
            "pse:method-kernel-binding:v1:{}:{}",
            method.to_hex(),
            index.iter().map(|id| id.to_hex()).collect::<String>()
        ),
    )
}
