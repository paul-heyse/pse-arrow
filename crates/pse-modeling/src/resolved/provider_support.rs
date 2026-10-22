// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact provider result boundaries; existing original arithmetic is never reinferred.
use super::*;
impl PhysicalSupport {
    pub(in crate::resolved) fn substitute_providers(
        &self,
        syntax: &ResolvedSyntax,
        inputs: &[SymbolIx],
        issued: &[(
            NodeIx,
            SymbolIx,
            pse_quantity::QuantityTypeId,
            Vec<pse_quantity::QuantityTypeId>,
        )],
        output_count: usize,
        context: &crate::TypeContext<'_>,
    ) -> Result<Self> {
        let ((nodes, order), premises) = pse_quantity::premises::capture(
            context.quantities,
            context.preconditions,
            || {
                if !self.observe(context.quantities, context.preconditions)? {
                    return Err(invalid(
                        SemanticId::NIL,
                        "supplier substitution has stale physical premises",
                    ));
                }
                let mut nodes = self.nodes.clone();
                let mut order = self.order.clone();
                for (root, symbol, quantity, ports) in issued {
                    let args = match syntax.nodes.get(*root) {
                        Some(Node::Provider { args, .. }) => args,
                        _ => {
                            return Err(invalid(
                                SemanticId::NIL,
                                "supplier issued root changed its typed provider role",
                            ));
                        }
                    };
                    if args.len() != ports.len() {
                        return Err(invalid(
                            SemanticId::NIL,
                            "supplier ordered input extent changed",
                        ));
                    }
                    for (arg, quantity) in args.iter().zip(ports) {
                        let contract = &nodes
                            .get(arg)
                            .ok_or_else(|| {
                                invalid(
                                    SemanticId::NIL,
                                    "supplier actual input lacks its original physical contract",
                                )
                            })?
                            .contract;
                        pse_quantity::admission::require_same_contract(
                            contract
                                .require_named()
                                .map_err(|error| invalid(SemanticId::NIL, error.to_string()))?,
                            *quantity,
                            context.quantities,
                        )
                        .map_err(|error| invalid(SemanticId::NIL, error.to_string()))?;
                    }
                    let declared = syntax.symbols.at(*symbol).ok_or_else(|| {
                        invalid(
                            SemanticId::NIL,
                            "supplier original output declaration is absent",
                        )
                    })?;
                    let target = self
                        .inputs
                        .iter()
                        .find(|(slot, _)| slot == symbol)
                        .map(|(_, contract)| contract.clone())
                        .ok_or_else(|| {
                            invalid(
                                SemanticId::NIL,
                                "supplier original independent output contract is absent",
                            )
                        })?;
                    let source = ResolvedPhysicalContract::named(
                        *quantity,
                        pse_quantity::IndexSet::new(),
                        context.quantities,
                    )
                    .map_err(|error| invalid(SemanticId::NIL, error.to_string()))?;
                    let result = OutputSupport::prepare(&source, &target, context.quantities)
                        .map_err(|error| invalid(SemanticId::NIL, error.to_string()))?;
                    nodes.insert(
                        *root,
                        NodeSupport {
                            ty: declared.ty.clone(),
                            contract: target.clone(),
                            operation: None,
                            literal: None,
                            boundary: Some(result),
                            call: None,
                            range: None,
                            exponent: None,
                        },
                    );
                    order.push(*root);
                    // The original scalar remains the nominal owner. Its value now has an
                    // explicit admitted definition boundary rather than an executable input.
                    for (node, value) in &mut nodes {
                        if matches!(syntax.nodes.get(*node),Some(Node::Reference(Reference::Symbol(slot))) if slot==symbol)
                        {
                            if value.ty != declared.ty || value.contract != target {
                                return Err(invalid(
                                    SemanticId::NIL,
                                    "supplier substitution changes the original scalar contract",
                                ));
                            }
                            value.boundary = Some(
                                OutputSupport::prepare(&target, &target, context.quantities)
                                    .map_err(|error| invalid(SemanticId::NIL, error.to_string()))?,
                            );
                        }
                    }
                }
                Ok::<_, crate::ModelingError>((nodes, order))
            },
        )?;
        let value = Self {
            syntax: syntax.identity()?,
            nodes,
            order,
            outputs: self
                .outputs
                .get(..output_count)
                .ok_or_else(|| invalid(SemanticId::NIL, "supplier original output extent changed"))?
                .to_vec(),
            inputs: self
                .inputs
                .iter()
                .filter(|(slot, _)| inputs.contains(slot))
                .cloned()
                .collect(),
            premises: PhysicalEvidence::capture(premises.ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "supplier substitution lacks a real physical capture",
                )
            })?),
            boolean: self.boolean.clone(),
            reduction: self.reduction.clone(),
            reduction_output: self.reduction_output.clone(),
        };
        Ok(value)
    }
}
