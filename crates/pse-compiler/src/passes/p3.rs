// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P3 constructs native normalized relations from admitted sources and typed syntax.

mod config;
pub(crate) use config::selected::{SelectedRoot, configure_selected};
mod demand_paths;
mod lower;
mod native;
use super::native_construction as plans;
mod port_paths;
pub(crate) mod products;
mod provenance;
mod seeds;
mod selectors;
mod source;
mod units;

use crate::AlgorithmOutput;
use crate::{AlgorithmContext, AlgorithmInputs, CompilerError};
use pse_authoring::document::{OwnedDocumentSet, binding::bind_sources_owned};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry,
    model::{AlgorithmSpec, RelationKey},
};
use std::collections::BTreeMap;

/// Complete checked outputs of the actual source normalization invocation.
#[derive(Debug)]
pub struct NormalizationOutput {
    /// Every declared P3 output, including explicit empty relations.
    pub rows: BTreeMap<RelationKey, FieldCheckedBatch>,
    /// Native output-to-source derivations for this invocation.
    pub derivations: FieldCheckedBatch,
}

/// Executable implementation bound to the exact registered P3 specification.
#[derive(Debug)]
pub struct P3 {
    spec: AlgorithmSpec,
}

impl P3 {
    /// Bind the declared P3 construction contract.
    /// # Errors
    /// Missing P3 declaration.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        Ok(Self {
            spec: registry
                .algorithm("P3@1")
                .ok_or_else(|| invalid("P3 declaration missing"))?
                .clone(),
        })
    }
}

impl crate::Algorithm for P3 {
    fn spec(&self) -> &AlgorithmSpec {
        &self.spec
    }
    fn run<'a>(
        &'a self,
        ctx: &'a AlgorithmContext<'a>,
        inputs: &'a AlgorithmInputs,
    ) -> pse_catalog::provider::BoxFut<'a, Result<AlgorithmOutput, CompilerError>> {
        Box::pin(async move {
            let output = Box::pin(normalize(
                inputs,
                ctx.documents,
                ctx.session,
                ctx.physical()?,
                ctx.cancel,
            ))
            .await?;
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let spec = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| invalid("P3 output undeclared"))?;
                    let batch = output
                        .rows
                        .get(&spec.key)
                        .ok_or_else(|| invalid("P3 omitted a declared output"))?;
                    Ok((port.port.clone(), batch.clone()))
                })
                .collect::<Result<_, CompilerError>>()?;
            Ok(AlgorithmOutput {
                outputs: ports,
                findings: Vec::new(),
                derivations: vec![output.derivations.into_batch()],
                plans: Vec::new(),
            })
        })
    }
}

/// Normalize exact admitted input roles and original immutable source documents.
/// Physical declarations are shared with the enclosing invocation. Native carriers
/// retain the actual computations until their complete output and evidence are exported.
/// # Errors
/// Missing or foreign source bindings, invalid syntax/configuration, semantic conflicts,
/// failed native obligations, cancellation or shared resource exhaustion.
pub async fn normalize(
    inputs: &AlgorithmInputs,
    documents: &OwnedDocumentSet,
    session: &SnapshotSession,
    physical: &crate::quantity_relations::PhysicalInventory,
    cancel: &CancellationToken,
) -> Result<NormalizationOutput, CompilerError> {
    let registry = session.registry();
    let pass = registry
        .algorithm("P3@1")
        .ok_or_else(|| invalid("P3 declaration missing"))?;
    inputs.validate(pass, registry)?;
    let checked = inputs.checked_rows(registry)?;
    let batches = checked
        .values()
        .map(|batch| (batch.relation_id(), batch.clone()))
        .collect();
    let mut construction = provenance::Construction::new(&checked, inputs, registry)?;
    construction.replace(primitives::emit(&construction.sources, pass, session, cancel).await?);

    let configuration = config::Configuration::build(
        &batches,
        &construction.sources,
        documents,
        session,
        physical,
        cancel,
    )
    .await?;
    let bindings =
        bind_sources_owned(documents, configuration.binding_batches(), session, cancel).await?;
    let syntax = source::emit(
        &bindings,
        configuration.binding_batches(),
        &construction,
        physical,
        pass,
        session,
        cancel,
    )
    .await?;
    construction.replace(syntax);
    let expressions = construction
        .output
        .get(&pse_relations::generated::normalized::expression_sources::RELATION_KEY)
        .ok_or_else(|| invalid("P3 expression source producer omitted its declaration"))?;
    let configured = configuration
        .emit(expressions.checked(), &construction.sources)
        .await?;
    let configured =
        provenance::materialize(configured.output, &construction, pass, session, cancel).await?;
    construction.replace(configured);

    construction.replace(selectors::emit(&construction.sources, pass, session, cancel).await?);
    construction.replace(port_paths::emit(&construction.sources, session, cancel).await?);
    let normalized = construction.session(session, cancel)?;
    let mut seed_inputs = checked.clone();
    seed_inputs.extend(construction.checked());
    construction.replace(
        seeds::emit(
            &seed_inputs,
            &construction.sources,
            pass,
            &normalized,
            cancel,
        )
        .await?,
    );
    construction
        .output
        .remove(&pse_relations::generated::provenance::property_read_occurrences::RELATION_KEY);
    construction.replace(
        products::emit(
            &construction.checked(),
            &construction.sources,
            pass,
            &normalized,
            cancel,
        )
        .await?,
    );

    // Empty normalized families are explicit members of this complete invocation.
    // Any nonempty construction must have supplied its own source occurrences.
    let mut empty = super::native_outputs::OutputRows::new(registry, session.reserver(), cancel)?;
    for spec in pass.outputs.iter().filter_map(|port| {
        registry
            .relation(&port.relation)
            .filter(|spec| !construction.output.contains_key(&spec.key))
    }) {
        empty.append_checked(
            &FieldCheckedBatch::concat_reserved(registry, spec, &[], session.reserver(), cancel)?,
            |_, _| Err(invalid("empty output unexpectedly has a row")),
        )?;
    }
    construction.replace(
        provenance::materialize(empty.finish()?, &construction, pass, session, cancel).await?,
    );
    let derivations = construction.evidence(registry, session.reserver(), cancel)?;
    Ok(NormalizationOutput {
        rows: construction.checked(),
        derivations,
    })
}

mod primitives;

fn invalid(reason: &str) -> CompilerError {
    pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    }
    .into()
}
