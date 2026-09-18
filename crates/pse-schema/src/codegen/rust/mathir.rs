// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Mechanical bridges from the declared mathematical families to the one `MathIR` API.

use super::{error, types};
use crate::{
    Registry, SchemaError,
    model::{Namespace, RelationSpec},
};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn source(registry: &Registry) -> Result<TokenStream, SchemaError> {
    let mut arms = Vec::new();
    for original in registry.relations().iter().filter(|spec| {
        spec.key.namespace == Namespace::Compiled
            && spec.key.name != "math_implicit_systems"
            && (spec.key.name.starts_with("math_") || spec.key.name == "kernel_bindings")
    }) {
        let mut families = vec![
            (Namespace::Compiled, original.key.name),
            (Namespace::Inferred, original.key.name),
        ];
        for prefix in ["template", "instance", "display", "contribution", "guard"] {
            if let Some(name) = crate::catalog::expr_family::target_name(prefix, original.key.name)
            {
                families.push((Namespace::Normalized, name));
            }
        }
        for (namespace, name) in families {
            let spec = registry
                .relation(&format!("{namespace}.{name}"))
                .ok_or_else(|| error("mathematical adapter family is undeclared".into()))?;
            let namespace_name = namespace.as_str();
            let module = types::ident(namespace_name);
            let relation = types::ident(name);
            let visit = callback(original.key.name, spec)?.map_or_else(
                || quote! {
                    if !view.is_empty() {
                        return Err(malformed("grounded mathematical output is not an indexed graph source"));
                    }
                },
                |callback| quote! {
                    for index in 0..view.len() {
                        let row = view.row(index).map_err(relation_error)?;
                        #callback
                    }
                },
            );
            arms.push(quote! {
                (#namespace_name, #name) => {
                    let view = pse_relations::generated::#module::#relation::View::from_checked(batch).map_err(relation_error)?;
                    #visit
                }
            });
        }
    }
    Ok(quote! {
        //! Exact generated Arrow views feed the existing mathematical algorithm callbacks.
        use pse_mathir::{DomainRef, GuardRef, MathIrError, NodeId, Opcode, Payload, ValueRef}; use pse_schema::math::{TemplateValueKind};
        use pse_quantity::{BoundIndexId, ConversionId, DomainId, InvariantId, OperationId, QuantityTypeId, ReductionKind, UnitId, WeightNormalization, infer::BuiltInRule};
        use crate::mathir_relations::malformed;
        #[expect(clippy::needless_pass_by_value, reason = "map_err consumes the relation error at the mathematical boundary")]
        fn relation_error(error: pse_relations::RelationError) -> MathIrError { malformed(error.to_string()) }
        fn node(value: i64) -> Result<NodeId, MathIrError> { if value < 0 { Err(malformed("negative mathematical node ordinal")) } else { Ok(NodeId(value)) } }
        fn narrow(value: i64) -> Result<u16, MathIrError> { u16::try_from(value).map_err(|_| malformed("mathematical position exceeds u16")) }
        fn enumeration<T>(value: &str, parse: fn(&str) -> Option<T>) -> Result<T, MathIrError> { parse(value).ok_or_else(|| malformed("unknown mathematical enum member")) }
        /// Replay one exact generated family into the domain algorithm, without a Cell model.
        pub(crate) fn replay(key: pse_schema::model::RelationKey, batch: &pse_relations::columnar::FieldCheckedBatch, sink: &mut dyn pse_mathir::relations::MathRelationSink) -> Result<(), MathIrError> {
            match (key.namespace.as_str(), key.name) { #(#arms)* _ => return Err(malformed("unregistered mathematical source family")) }
            Ok(())
        }
    })
}

fn callback(name: &str, spec: &RelationSpec) -> Result<Option<TokenStream>, SchemaError> {
    let normalized = spec.key.namespace == Namespace::Normalized;
    let pending = spec.key.namespace != Namespace::Compiled;
    let result = match name {
        "math_expr_nodes" => {
            let payload = super::mathir_value::decode();
            quote! {
                let payload = #payload;
                crate::mathir_relations::check_payload_family(&payload, #normalized, #pending)?;
                let children = row.children.into_iter().map(node).collect::<Result<Vec<_>, _>>()?;
                sink.expr_node(node(row.node_id)?, enumeration(row.opcode.as_str(), Opcode::parse)?, &children, &payload, row.quantity_type_id.map(QuantityTypeId::from_id), row.scope_instance_id, row.subtree_hash)?;
            }
        }
        "math_indexed_equations" => {
            quote! {
                let sense = enumeration(row.constraint.kind.as_str(), pse_schema::math::Sense::parse)?;
                row.constraint.selected().map_err(|_| malformed("invalid equation constraint alternative"))?;
                let (lower, upper) = if let Some(range) = row.constraint.range {
                    (Some(node(range.lower_node_id)?), Some(node(range.upper_node_id)?))
                } else {
                    let target = node(row.constraint.single.ok_or_else(|| malformed("missing equation target"))?.node_id)?;
                    if sense == pse_schema::math::Sense::Ge { (Some(target), None) } else { (None, Some(target)) }
                };
                sink.indexed_equation(row.indexed_equation_id, row.owner_instance_id, row.equation_decl_id, &row.qualified_name, row.product_id, row.filter_node_id.map(node).transpose()?, node(row.body_node_id)?, sense, lower, upper, row.residual_quantity_type_id.map(QuantityTypeId::from_id), row.law_instance_id, row.derivation_id)?;
            }
        }
        "math_free_indices" => {
            quote!(sink.free_index(row.indexed_equation_id, BoundIndexId::from_id(row.bound_index_id), DomainId::from_id(row.domain_id), narrow(row.position)?)?;)
        }
        "math_quantity_selections" => {
            quote! { let conversions = row.conversions.into_iter().map(|conversion| Ok((narrow(conversion.operand)?, ConversionId::from_id(conversion.conversion_id)))).collect::<Result<Vec<_>, MathIrError>>()?; let permutation = row.operand_permutation.into_iter().map(narrow).collect::<Result<Vec<_>, _>>()?; sink.quantity_selection(node(row.node_id)?, row.operation_id.map(OperationId::from_id), row.builtin_rule.map(|rule| enumeration(rule.as_str(), BuiltInRule::parse)).transpose()?, &permutation, &conversions, row.deferred_static_check)?; }
        }
        "kernel_bindings" => {
            quote! { let parameters = row.parameter_bindings.into_iter().map(|parameter| {
                parameter.binding.selected().map_err(relation_error)?;
                let (symbol, value, unit) = if let Some(symbol) = parameter.binding.symbol {
                    (Some(symbol.symbol_id), None, None)
                } else {
                    let literal = parameter.binding.literal.ok_or_else(|| malformed("missing kernel literal"))?;
                    (None, Some(literal.value), Some(UnitId::from_id(literal.unit_id)))
                };
                Ok((parameter.name, symbol, value, unit))
            }).collect::<Result<Vec<_>, MathIrError>>()?; let inputs = row.input_bindings.into_iter().map(|input| Ok((input.name, node(input.node_id)?))).collect::<Result<Vec<_>, MathIrError>>()?; sink.kernel_binding(row.binding_id, row.kernel_id, row.scope_instance_id, &parameters, &inputs)?; }
        }
        "math_complementarity" | "math_dae_links" | "math_equations" | "math_objectives" => {
            return Ok(None);
        }
        _ => {
            return Err(error(format!(
                "mathematical callback missing for declared source {name}"
            )));
        }
    };
    Ok(Some(result))
}
