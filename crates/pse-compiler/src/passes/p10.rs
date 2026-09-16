// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P10 binds native stage inputs to physical graph algorithms and native root mapping.
mod contracts;
mod production;

use crate::{CompilerError, InputBundle, PassContext};
use pse_catalog::computation::ProducedStage;
use pse_schema::{Registry, model::PassSpec};

/// Physical compilation from the actual registered inputs and explicit context.
#[derive(Debug)]
pub struct P10 {
    spec: PassSpec,
}
impl P10 {
    /// Binds the production physical compilation declaration.
    /// # Errors
    /// P10 or one of its required mathematical context ports is undeclared.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        let spec = registry
            .pass("P10@1")
            .ok_or_else(|| invalid("P10 pass is not declared"))?
            .clone();
        for required in [
            "reference.math_context",
            "compiled.expression_roots",
            "compiled.expression_root_indices",
            "inferred.math_expr_nodes",
        ] {
            if !spec
                .inputs
                .iter()
                .any(|port| port.relation == required && port.required)
            {
                return Err(invalid(format!("P10 lacks required context {required}")));
            }
        }
        Ok(Self { spec })
    }
}
impl crate::Pass for P10 {
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
            let mut output = production::run(ctx, inputs, &self.spec).await?;
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let spec = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| invalid("P10 output contract absent"))?;
                    let batch = output
                        .remove(&spec.key)
                        .ok_or_else(|| invalid(format!("P10 output {} missing", spec.key)))?;
                    Ok((port.port.to_owned(), batch))
                })
                .collect::<Result<_, CompilerError>>()?;
            if !output.is_empty() {
                return Err(invalid("P10 constructed undeclared outputs"));
            }
            Ok(ProducedStage {
                outputs: ports,
                findings: vec![],
                derivations: vec![],
                plans: Vec::new(),
            })
        })
    }
}
fn invalid(detail: impl Into<String>) -> CompilerError {
    pse_mathir::MathIrError::Malformed {
        node: None,
        detail: detail.into(),
    }
    .into()
}
