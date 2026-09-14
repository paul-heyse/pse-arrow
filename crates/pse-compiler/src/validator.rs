// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stage admission re-executes the actual producer and compares complete values.
mod compare;
mod fixture;
mod sources;

use crate::passes::{dag::invalid, p3::P3, p10::P10};
use crate::{BoundInput, CompilerError, ExternalInputs, InputBundle, Pass, PassContext, PolicySet};
use pse_catalog::{
    Catalog, CatalogError,
    store::membership::{AdmissionContext, SemanticValidator},
};
use pse_ids::CancellationToken;
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{PassSpec, PortSource, RelationKey},
};
use std::{collections::BTreeMap, sync::Arc};

/// Registered rule admission plus actual P3 source-to-output verification.
/// P10 and its fixed complete predecessor fixture require explicit opt-in.
#[derive(Debug)]
pub struct CompilerValidator {
    invariants: Arc<dyn SemanticValidator>,
    p10_fixture: bool,
}
impl CompilerValidator {
    /// Compose with the existing actual-row invariant validator.
    #[must_use]
    pub fn new(invariants: Arc<dyn SemanticValidator>) -> Self {
        Self {
            invariants,
            p10_fixture: false,
        }
    }
    /// Admit the complete, declared arithmetic P10 fixture and its importer.
    /// This is restricted to the fixed fixture's actual values and never invents P4–P9.
    #[must_use]
    pub fn with_p10_fixture(mut self) -> Self {
        self.p10_fixture = true;
        self
    }
    async fn stage(
        &self,
        catalog: &Catalog,
        context: &AdmissionContext,
        candidates: &BTreeMap<String, RecordBatch>,
        cancel: &CancellationToken,
    ) -> Result<(), CompilerError> {
        let registry = catalog.registry();
        let spec = context
            .stage_pass
            .and_then(|id| registry.passes().iter().find(|spec| spec.id == id))
            .ok_or_else(|| invalid("stage has no actual registered producer"))?;
        let inputs = bind(spec, registry, context)?;
        if self.p10_fixture && spec.name == "FixtureP10Inputs" && spec.version == "1" {
            return fixture::validate(catalog, spec, &inputs, candidates, cancel);
        }
        let pass: Box<dyn Pass> = match (spec.name, spec.version) {
            ("P3", "1") => Box::new(P3::new(registry)?),
            ("P10", "1") if self.p10_fixture => Box::new(P10::new(registry)?),
            _ => {
                return Err(invalid(
                    "stage has no executable semantic admission contract",
                ));
            }
        };
        let documents =
            crate::driver::inputs::documents(catalog, &inputs.rows(registry)?, cancel).await?;
        let policies = PolicySet::default();
        let external = ExternalInputs::default();
        let ctx = PassContext {
            registry,
            documents: &documents,
            policies: &policies,
            external: &external,
            cancel,
            reserver: catalog.reserver().as_ref(),
            session: None,
        };
        let expected = pass.run(&ctx, &inputs).await?;
        crate::passes::bundle::validate_output(&expected, spec, registry)?;
        for port in &spec.outputs {
            compare::rows(
                catalog,
                &port.relation,
                &expected.ports[port.port],
                std::slice::from_ref(
                    candidates
                        .get(port.port)
                        .ok_or_else(|| invalid("stage candidate output absent"))?,
                ),
                cancel,
            )?;
        }
        Ok(())
    }
}
impl SemanticValidator for CompilerValidator {
    fn validate<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants.validate(registry, rows, cancel)
    }
    fn validate_sidecar<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants.validate_sidecar(registry, rows, cancel)
    }
    fn validate_stage<'a>(
        &'a self,
        catalog: &'a Catalog,
        context: &'a AdmissionContext,
        candidates: &'a BTreeMap<String, RecordBatch>,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            self.stage(catalog, context, candidates, cancel)
                .await
                .map_err(|error| CatalogError::Semantic(Arc::new(error)))
        })
    }
    fn validate_snapshot_sources<'a>(
        &'a self,
        catalog: &'a Catalog,
        kind: pse_ids::SnapshotKind,
        context: &'a AdmissionContext,
        candidates: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            sources::validate(catalog, kind, context, candidates, cancel)
                .await
                .map_err(|error| CatalogError::Semantic(Arc::new(error)))
        })
    }
}

fn bind(
    spec: &PassSpec,
    registry: &Registry,
    context: &AdmissionContext,
) -> Result<InputBundle, CompilerError> {
    let mut inputs = InputBundle::new();
    for port in &spec.inputs {
        let relation = registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("stage input declaration missing"))?;
        let binding = context
            .parents
            .get(port.port)
            .map(|snapshot| match port.source {
                PortSource::Pinned => {
                    BoundInput::bind(Arc::clone(snapshot), relation.key, registry)
                }
                PortSource::Derived { port, .. } => {
                    BoundInput::bind_port(Arc::clone(snapshot), relation.key, port, registry)
                }
            })
            .transpose()?;
        inputs.ports.insert(port.port, binding);
    }
    inputs.validate(spec, registry)?;
    Ok(inputs)
}
