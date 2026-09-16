// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conservation and equality expansion from actual classified indexed descriptors.
mod contexts;
mod expansion;
mod graph;
mod inventory;
mod outputs;
use super::{native_outputs::Sources, native_rows::workspace};
use crate::{CompilerError, InputBundle, PassContext};
use pse_catalog::computation::ProducedStage;
use pse_rules::{RuleError, strata::RuleInputLocation};
use pse_schema::{Registry, model::PassSpec};

/// Registered P8 implementation; participation decisions are declared `RulePlan`s.
#[derive(Debug)]
pub struct P8 {
    spec: PassSpec,
}
impl P8 {
    /// Bind the complete registry declaration.
    /// # Errors
    /// P8 is undeclared.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        Ok(Self {
            spec: registry
                .pass("P8@1")
                .ok_or_else(|| failure("P8 declaration absent"))?
                .clone(),
        })
    }
}
impl crate::Pass for P8 {
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
            let base = ctx.session;
            let mut actual = inputs.checked_rows(ctx.registry)?;
            let mut sources = Sources::from_inputs(inputs, ctx.registry)?;
            let constructed = contexts::build(&self.spec, ctx, inputs).await?;
            let mut derivations = Vec::new();
            let mut checked = std::collections::BTreeMap::new();
            for (key, input) in constructed {
                actual.insert(key, input.checked().clone());
                checked.insert(key, input.checked().clone());
                derivations.push(input.derivations().clone().into_batch());
                sources.replace_native(key, input)?;
            }
            let session = workspace(base, &checked, ctx.cancel)?;
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
                let input = result.completed.relation(key)?;
                actual.insert(key, input.checked().clone());
                checked.insert(key, input.checked().clone());
                sources.replace_location(key, RuleInputLocation::Completed(input))?;
            }
            derivations.extend(result.derivations);
            let session = workspace(&session, &checked, ctx.cancel)?;
            let expanded = Box::pin(expansion::expand(
                &actual, &sources, &session, ctx, &self.spec,
            ))
            .await?;
            derivations.extend(expanded.derivations);
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let relation = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| failure("P8 output contract absent"))?;
                    let batch = expanded.rows.get(&relation.key).ok_or_else(|| {
                        failure(format!("P8 output {} not produced", port.relation))
                    })?;
                    Ok((port.port.to_owned(), batch.clone()))
                })
                .collect::<Result<_, RuleError>>()?;
            Ok(ProducedStage {
                outputs: ports,
                findings: vec![],
                derivations,
                plans: Vec::new(),
            })
        })
    }
}
pub(super) fn failure(message: impl Into<String>) -> RuleError {
    RuleError::Internal {
        what: message.into(),
    }
}
