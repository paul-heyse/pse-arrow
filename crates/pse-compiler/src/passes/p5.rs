// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native port, topology and scope construction with typed graph/path algorithms.
mod paths;
mod ports;
mod tears;
use super::{
    native_outputs::{self, OutputRows, SourceRole, Sources},
    native_rows::workspace,
};
use crate::AlgorithmOutput;
use crate::{AlgorithmContext, AlgorithmInputs, CompilerError};
use pse_catalog::session::SnapshotSession;
use pse_relations::{
    RecordBatch,
    columnar::FieldCheckedBatch,
    generated::{inferred as i, normalized as n},
};
use pse_rules::strata::RuleInputLocation;
use pse_schema::{
    Registry,
    model::{AlgorithmSpec, RelationKey},
};
use std::collections::BTreeMap;

/// Complete native stage results and their actual source bindings.
#[derive(Debug)]
pub(crate) struct NativeStage {
    pub(crate) relations: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(crate) sources: Sources,
    pub(crate) derivations: Vec<RecordBatch>,
}

/// Registered P5 implementation over actual native source owners.
#[derive(Debug)]
pub struct P5 {
    spec: AlgorithmSpec,
}
impl P5 {
    /// Bind the complete registry declaration.
    /// # Errors
    /// P5 is undeclared.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        Ok(Self {
            spec: registry
                .algorithm("P5@1")
                .ok_or_else(|| invalid("P5 declaration absent"))?
                .clone(),
        })
    }
}
impl crate::Algorithm for P5 {
    fn spec(&self) -> &AlgorithmSpec {
        &self.spec
    }
    fn run<'a>(
        &'a self,
        ctx: &'a AlgorithmContext<'a>,
        inputs: &'a AlgorithmInputs,
    ) -> pse_catalog::provider::BoxFut<'a, Result<AlgorithmOutput, CompilerError>> {
        Box::pin(async move {
            inputs.validate(&self.spec, ctx.registry)?;
            let rows = inputs.checked_rows(ctx.registry)?;
            let sources = Sources::from_inputs(inputs, ctx.registry)?;
            let base = ctx.session;
            let output = Box::pin(evaluate(&self.spec, ctx, inputs, rows, sources, base)).await?;
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let spec = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| invalid("P5 output declaration absent"))?;
                    let batch = output
                        .relations
                        .get(&spec.key)
                        .ok_or_else(|| invalid(format!("P5 omitted {}", port.relation)))?;
                    Ok((port.port.clone(), batch.clone()))
                })
                .collect::<Result<_, CompilerError>>()?;
            Ok(AlgorithmOutput {
                outputs: ports,
                findings: Vec::new(),
                derivations: output.derivations,
                plans: Vec::new(),
            })
        })
    }
}

pub(crate) async fn evaluate(
    pass: &AlgorithmSpec,
    ctx: &AlgorithmContext<'_>,
    inputs: &AlgorithmInputs,
    mut rows: BTreeMap<RelationKey, FieldCheckedBatch>,
    mut sources: Sources,
    base: &SnapshotSession,
) -> Result<NativeStage, CompilerError> {
    let constructed = ports::construct(&rows, &sources, pass, base, ctx.cancel).await?;
    let mut relations = BTreeMap::new();
    let mut derivations = Vec::new();
    for (key, input) in constructed {
        rows.insert(key, input.checked().clone());
        relations.insert(key, input.checked().clone());
        derivations.push(input.derivations().clone().into_batch());
        sources.replace_native(key, input)?;
    }
    let session = workspace(base, &relations, ctx.cancel)?;
    let p5 = ctx
        .registry
        .algorithm("P5@1")
        .ok_or_else(|| invalid("P5 declaration absent"))?;
    let locations = sources.locations()?;
    let result =
        super::p4::execute_selected_program(p5, pass, ctx, inputs, &session, &locations).await?;
    for key in result.completed.keys() {
        let result = result.completed.relation(key)?;
        rows.insert(key, result.checked().clone());
        relations.insert(key, result.checked().clone());
        sources.replace_location(key, RuleInputLocation::Completed(result))?;
    }
    derivations.extend(result.derivations);
    let session = workspace(&session, &relations, ctx.cancel)?;
    let mut inventory = super::p4::predicates::Inventory::load(
        &rows,
        ctx.registry,
        &session,
        ctx.reserver,
        ctx.cancel,
        ctx.physical()?,
    )
    .await?;
    let mut output = OutputRows::new(ctx.registry, ctx.reserver, ctx.cancel)?;
    for (target, read) in [
        (i::path_targets::RELATION_KEY, i::instances::RELATION_KEY),
        (
            i::path_targets::RELATION_KEY,
            i::valid_index_tuples::RELATION_KEY,
        ),
        (
            i::path_targets::RELATION_KEY,
            n::config_values::RELATION_KEY,
        ),
        (
            i::path_targets::RELATION_KEY,
            n::instance_bindings::RELATION_KEY,
        ),
        (
            i::tear_candidates::RELATION_KEY,
            i::topology_edges::RELATION_KEY,
        ),
        (
            i::tear_candidates::RELATION_KEY,
            n::connections::RELATION_KEY,
        ),
    ] {
        let (port, _) = sources
            .get(&read)
            .ok_or_else(|| invalid("P5 algorithm read scope is not bound"))?;
        output.read_scope(
            target,
            SourceRole {
                relation: read,
                port: port.clone(),
            },
        )?;
    }
    let mut work = ctx.reserver.open("P5:finite-algorithms");
    paths::emit(&mut inventory, ctx, &sources, work.as_mut(), &mut output).await?;
    tears::emit(ctx, &sources, &mut inventory, work.as_mut(), &mut output).await?;
    for (key, input) in
        native_outputs::materialize(output.finish()?, &sources, pass, &session, ctx.cancel).await?
    {
        relations.insert(key, input.checked().clone());
        derivations.push(input.derivations().clone().into_batch());
        sources.replace_native(key, input)?;
    }
    Ok(NativeStage {
        relations,
        sources,
        derivations,
    })
}
fn invalid(reason: impl Into<String>) -> CompilerError {
    super::p4::invalid(reason)
}
