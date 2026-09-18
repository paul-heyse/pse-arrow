// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P6 constructs native finite keys, then executes the declared demand rules.
mod framing;
use super::{native_outputs::Sources, native_rows::workspace};
use crate::AlgorithmOutput;
use crate::{AlgorithmContext, AlgorithmInputs, CompilerError};
use pse_schema::{Registry, model::AlgorithmSpec};
use std::collections::BTreeMap;

/// Finite demand closure over exact immutable sources and declared methods.
#[derive(Debug)]
pub struct P6 {
    spec: AlgorithmSpec,
}
impl P6 {
    /// Bind the complete registered P6 contract.
    /// # Errors
    /// P6 is undeclared.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        Ok(Self {
            spec: registry
                .algorithm("P6@1")
                .ok_or_else(|| super::p4::invalid("P6 declaration missing"))?
                .clone(),
        })
    }
}
impl crate::Algorithm for P6 {
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
            let base = ctx.session;
            let mut sources = Sources::from_inputs(inputs, ctx.registry)?;
            let framed = Box::pin(framing::construct(
                &self.spec,
                &inputs.checked_rows(ctx.registry)?,
                &sources,
                ctx,
            ))
            .await?;
            let mut rows = BTreeMap::new();
            let mut derivations = Vec::new();
            for (key, input) in framed {
                rows.insert(key, input.checked().clone());
                derivations.push(input.derivations().clone().into_batch());
                sources.replace_native(key, input)?;
            }
            let session = workspace(base, &rows, ctx.cancel)?;
            let result = super::p4::execute_selected_program(
                &self.spec,
                &self.spec,
                ctx,
                inputs,
                &session,
                &sources.locations()?,
            )
            .await?;
            for key in result.completed.keys() {
                rows.insert(key, result.completed.relation(key)?.checked().clone());
            }
            derivations.extend(result.derivations);
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let spec = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| super::p4::invalid("P6 output contract absent"))?;
                    Ok((
                        port.port.clone(),
                        rows.get(&spec.key)
                            .ok_or_else(|| {
                                super::p4::invalid(format!("P6 omitted {}", port.relation))
                            })?
                            .clone(),
                    ))
                })
                .collect::<Result<_, CompilerError>>()?;
            Ok(AlgorithmOutput {
                outputs: ports,
                findings: Vec::new(),
                derivations,
                plans: Vec::new(),
            })
        })
    }
}
