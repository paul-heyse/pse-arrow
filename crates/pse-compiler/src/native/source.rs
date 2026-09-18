// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source parsing/target resolution is a finite native algorithm over document rows.
use crate::{
    Algorithm, AlgorithmContext, AlgorithmInputs, AlgorithmOutput, CompilerError, passes::invalid,
};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{Registry, model::AlgorithmSpec};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct SourceProjection {
    spec: AlgorithmSpec,
}
impl SourceProjection {
    pub(super) fn new(registry: &Registry) -> Result<Self, CompilerError> {
        let spec = registry
            .algorithm("source@1")
            .ok_or_else(|| invalid("source projection declaration absent"))?
            .clone();
        Ok(Self { spec })
    }
}
impl Algorithm for SourceProjection {
    fn spec(&self) -> &AlgorithmSpec {
        &self.spec
    }
    fn requires_physical(&self) -> bool {
        false
    }
    fn run<'a>(
        &'a self,
        ctx: &'a AlgorithmContext<'a>,
        _: &'a AlgorithmInputs,
    ) -> pse_catalog::provider::BoxFut<'a, Result<AlgorithmOutput, CompilerError>> {
        Box::pin(async move {
            let mut rows =
                pse_authoring::p1::project(ctx.documents, ctx.session, ctx.cancel).await?;
            for (key, batch) in pse_relations::registry_relations::materialize(ctx.registry)? {
                let spec = ctx
                    .registry
                    .relation_by_key(key)
                    .ok_or_else(|| invalid("registry reflection declaration absent"))?;
                let checked = FieldCheckedBatch::admit(ctx.registry, spec, batch)?;
                if rows.insert(spec.id, checked).is_some() {
                    return Err(invalid(
                        "source documents cannot override registry reflection",
                    ));
                }
            }
            let mut outputs = BTreeMap::new();
            for port in &self.spec.outputs {
                let spec = ctx
                    .registry
                    .relation(&port.relation)
                    .ok_or_else(|| invalid("source output undeclared"))?;
                let value = match rows.remove(&spec.id) {
                    Some(value) => value,
                    None => FieldCheckedBatch::concat_reserved(
                        ctx.registry,
                        spec,
                        &[],
                        ctx.reserver,
                        ctx.cancel,
                    )?,
                };
                outputs.insert(port.port.clone(), value);
            }
            Ok(AlgorithmOutput {
                outputs,
                findings: vec![],
                derivations: vec![],
                plans: vec![],
            })
        })
    }
}
