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
        use pse_mathir::{DomainRef, GuardRef, MathIrError, NodeId, Opcode, TemplateValueKind, ValueRef};
        use pse_quantity::{BoundIndexId, ConversionId, DomainId, InvariantId, OperationId, QuantityTypeId, ReductionKind, UnitId, WeightNormalization, infer::BuiltInRule};
        use crate::mathir_relations::malformed;
        #[expect(clippy::needless_pass_by_value, reason = "map_err consumes the relation error at the mathematical boundary")]
        fn relation_error(error: pse_relations::RelationError) -> MathIrError { malformed(error.to_string()) }
        fn enumeration<T>(value: &str, parse: fn(&str) -> Option<T>) -> Result<T, MathIrError> { parse(value).ok_or_else(|| malformed("unknown mathematical enum member")) }
        fn guard(node: Option<u64>, source: Option<pse_ids::SemanticId>, predicate: Option<u64>) -> Result<Option<GuardRef>, MathIrError> {
            if node.is_none() && source.is_none() && predicate.is_none() { return Ok(None); }
            GuardRef::from_columns(node.map(NodeId), source, predicate).map(Some)
        }
        /// Replay one exact generated family into the domain algorithm, without a Cell model.
        pub(crate) fn replay(key: pse_schema::model::RelationKey, batch: &pse_relations::columnar::FieldCheckedBatch, sink: &mut dyn pse_mathir::relations::MathRelationSink) -> Result<(), MathIrError> {
            match (key.namespace.as_str(), key.name) { #(#arms)* _ => return Err(malformed("unregistered mathematical source family")) }
            Ok(())
        }
    })
}

fn has(spec: &RelationSpec, name: &str) -> bool {
    spec.columns.iter().any(|field| field.name() == name)
}
fn domain(spec: &RelationSpec, name: &str) -> TokenStream {
    let field = types::ident(name);
    if has(spec, "template_id") {
        quote!(DomainRef::from_columns(row.#field.map(DomainId::from_id), row.template_id, row.domain_name)?)
    } else {
        quote!(DomainRef::Actual(DomainId::from_id(row.#field)))
    }
}
fn guard(spec: &RelationSpec, filter: bool) -> TokenStream {
    let (node, source, predicate) = if filter {
        ("filter_node_id", "filter_source_id", "filter_predicate_id")
    } else {
        ("guard_node_id", "guard_source_id", "guard_predicate_id")
    };
    let nullable = spec
        .columns
        .iter()
        .find(|field| field.name() == node)
        .is_none_or(crate::model::FieldContract::nullable);
    let node = types::ident(node);
    let node = if nullable {
        quote!(row.#node)
    } else {
        quote!(Some(row.#node))
    };
    let (source, predicate) = if has(spec, source) {
        let source = types::ident(source);
        let predicate = types::ident(predicate);
        (quote!(row.#source), quote!(row.#predicate))
    } else {
        (quote!(None), quote!(None))
    };
    quote!(guard(#node, #source, #predicate)?)
}

#[expect(
    clippy::too_many_lines,
    reason = "Mechanical mapping of the complete mathematical family to its generated Arrow adapter"
)]
fn callback(name: &str, spec: &RelationSpec) -> Result<Option<TokenStream>, SchemaError> {
    let filter = guard(spec, true);
    let result = match name {
        "math_expr_nodes" => {
            quote!(sink.expr_node(NodeId(row.node_id), enumeration(row.opcode.as_str(), Opcode::parse)?, row.quantity_type_id.map(QuantityTypeId::from_id), row.scope_instance_id, row.subtree_hash)?;)
        }
        "math_expr_args" => {
            quote!(sink.expr_arg(NodeId(row.parent_node_id), row.argument_ordinal, NodeId(row.child_node_id))?;)
        }
        "math_symbol_refs" if !has(spec, "kind") => {
            quote!(sink.symbol_ref(NodeId(row.node_id), ValueRef::ActualSymbol(row.symbol_id))?;)
        }
        "math_symbol_refs" => reference(),
        "math_float_constants" => {
            quote!(sink.float_constant(NodeId(row.node_id), row.value, UnitId::from_id(row.unit_id))?;)
        }
        "math_int_constants" => quote!(sink.int_constant(NodeId(row.node_id), row.value)?;),
        "math_conditionals" => {
            let guard = guard(spec, false);
            quote!(sink.conditional(NodeId(row.node_id), #guard.ok_or_else(|| malformed("conditional guard absent"))?)?;)
        }
        "math_kernel_calls" => {
            quote!(sink.kernel_call(NodeId(row.node_id), row.kernel_binding_id, row.output_ordinal)?;)
        }
        "math_implicit_refs" => {
            quote!(sink.implicit_ref(NodeId(row.node_id), row.implicit_system_id, row.unknown_ordinal)?;)
        }
        "math_derivatives" => {
            let domain = domain(spec, "wrt_domain_id");
            quote!(sink.derivative(NodeId(row.node_id), #domain, row.order)?;)
        }
        "math_integrals" => {
            let domain = domain(spec, "domain_id");
            quote!(sink.integral(NodeId(row.node_id), #domain, BoundIndexId::from_id(row.bound_index_id), row.quadrature_policy_id, #filter)?;)
        }
        "math_broadcasts" => {
            let domain = domain(spec, "domain_id");
            quote!(sink.broadcast(NodeId(row.node_id), #domain, BoundIndexId::from_id(row.bound_index_id))?;)
        }
        "math_reductions" => {
            let domain = domain(spec, "domain_id");
            quote!(sink.reduction(NodeId(row.node_id), enumeration(row.kind.as_str(), ReductionKind::parse)?, #domain, BoundIndexId::from_id(row.bound_index_id), #filter)?;)
        }
        "math_affine" => {
            quote! { let terms = row.terms.into_iter().map(|term| (term.coefficient, NodeId(term.child_node_id))).collect::<Vec<_>>(); sink.affine(NodeId(row.node_id), row.constant, row.constant_quantity_type_id.map(QuantityTypeId::from_id), row.constant_unit_id.map(UnitId::from_id), &terms)?; }
        }
        "math_weighted_means" => {
            quote! { let pairs = row.pairs.into_iter().map(|pair| (NodeId(pair.weight_node_id), NodeId(pair.value_node_id))).collect::<Vec<_>>(); sink.weighted_mean(NodeId(row.node_id), &pairs, enumeration(row.normalization.as_str(), WeightNormalization::parse)?, row.unit_sum_invariant_id.map(InvariantId::from_id))?; }
        }
        "math_gathers" => {
            let coordinates = if has(spec, "gather_state") {
                quote!(
                    row.coordinate_map
                        .ok_or_else(|| malformed("resolved gather coordinates absent"))?
                )
            } else {
                quote!(row.coordinate_map)
            };
            let pending = if has(spec, "gather_state") {
                quote! {
                    if row.gather_state.as_str() == "pending" {
                        if row.coordinate_map.is_some() { return Err(malformed("pending gather carries resolved coordinates")); }
                        let indices = row.index_nodes.ok_or_else(|| malformed("pending gather indices absent"))?.into_iter().map(NodeId).collect::<Vec<_>>();
                        sink.pending_gather(NodeId(row.node_id), row.group_id, &indices)?; continue;
                    }
                    if row.index_nodes.is_some() { return Err(malformed("resolved gather carries pending indices")); }
                }
            } else {
                quote!()
            };
            quote! { #pending let coordinates = #coordinates.into_iter().map(|coordinate| (BoundIndexId::from_id(coordinate.bound_index_id), coordinate.position)).collect::<Vec<_>>(); sink.gather(NodeId(row.node_id), row.group_id, &coordinates)?; }
        }
        "math_piecewise_linear" => {
            quote! { let points = row.breakpoints.into_iter().map(|point| (point.x, point.y)).collect::<Vec<_>>(); sink.piecewise_linear(NodeId(row.node_id), &points, QuantityTypeId::from_id(row.input_quantity_type_id), QuantityTypeId::from_id(row.output_quantity_type_id))?; }
        }
        "math_indexed_equations" => {
            quote!(sink.indexed_equation(row.indexed_equation_id, row.owner_instance_id, row.equation_decl_id, &row.qualified_name, row.product_id, row.filter_node_id.map(NodeId), NodeId(row.body_node_id), enumeration(row.sense.as_str(), pse_mathir::equation::Sense::parse)?, row.lower_node_id.map(NodeId), row.upper_node_id.map(NodeId), row.residual_quantity_type_id.map(QuantityTypeId::from_id), row.law_instance_id, row.derivation_id)?;)
        }
        "math_free_indices" => {
            quote!(sink.free_index(row.indexed_equation_id, BoundIndexId::from_id(row.bound_index_id), DomainId::from_id(row.domain_id), row.position)?;)
        }
        "math_quantity_selections" => {
            quote! { let conversions = row.conversions.into_iter().map(|conversion| (conversion.operand, ConversionId::from_id(conversion.conversion_id))).collect::<Vec<_>>(); sink.quantity_selection(NodeId(row.node_id), row.operation_id.map(OperationId::from_id), row.builtin_rule.map(|rule| enumeration(rule.as_str(), BuiltInRule::parse)).transpose()?, &row.operand_permutation, &conversions, row.deferred_static_check)?; }
        }
        "kernel_bindings" => {
            quote! { let parameters = row.parameter_bindings.into_iter().map(|parameter| (parameter.name, parameter.symbol_id, parameter.value, parameter.unit_id.map(UnitId::from_id))).collect::<Vec<_>>(); let inputs = row.input_bindings.into_iter().map(|input| (input.name, NodeId(input.node_id))).collect::<Vec<_>>(); sink.kernel_binding(row.binding_id, row.kernel_id, row.scope_instance_id, &parameters, &inputs)?; }
        }
        "math_smooth_ops" if has(spec, "epsilon_state") => {
            quote! { if !row.eps.is_finite() || row.eps <= 0.0 { return Err(malformed("epsilon must be finite and positive")); } match (row.epsilon_state.as_str(), row.eps_unit_id) { ("coordinate", None) => { sink.smooth_op(NodeId(row.node_id), row.eps)?; }, ("pending_unit", Some(unit)) => { sink.pending_smooth_op(NodeId(row.node_id), row.eps, UnitId::from_id(unit))?; }, _ => return Err(malformed("overlapping or incomplete smoothing alternatives")) } }
        }
        "math_smooth_ops" => quote!(sink.smooth_op(NodeId(row.node_id), row.eps)?;),
        "math_unit_converts" if has(spec, "conversion_state") => {
            quote! { match (row.conversion_state.as_str(), row.scale, row.offset, row.from_unit_id) { ("pending", None, None, None) => { sink.pending_unit_convert(NodeId(row.node_id), UnitId::from_id(row.to_unit_id))?; }, ("resolved", Some(scale), Some(offset), Some(from)) => { sink.unit_convert(NodeId(row.node_id), scale, offset, UnitId::from_id(from), UnitId::from_id(row.to_unit_id))?; }, _ => return Err(malformed("overlapping or incomplete conversion alternatives")) } }
        }
        "math_unit_converts" => {
            quote!(sink.unit_convert(NodeId(row.node_id), row.scale, row.offset, UnitId::from_id(row.from_unit_id), UnitId::from_id(row.to_unit_id))?;)
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

fn reference() -> TokenStream {
    quote! {
        if row.kind.as_str() == pse_mathir::Payload::PENDING_PATH_KIND {
            if row.symbol_id.is_some() || row.template_id.is_some() || row.name.is_some() || row.domain_id.is_some() || row.bound_index_id.is_some() { return Err(malformed("path reference overlaps another alternative")); }
            let indices = row.path_index_nodes.ok_or_else(|| malformed("path index inventory absent"))?.into_iter().map(NodeId).collect::<Vec<_>>();
            sink.pending_path(NodeId(row.node_id), row.path_source_id.ok_or_else(|| malformed("path source absent"))?, row.path_id.ok_or_else(|| malformed("path ordinal absent"))?, &indices)?;
            continue;
        }
        if row.path_source_id.is_some() || row.path_id.is_some() || row.path_index_nodes.is_some() { return Err(malformed("ordinary reference carries path fields")); }
        let reference = match (row.kind.as_str(), row.symbol_id, row.template_id, row.name, row.domain_id, row.bound_index_id) {
            ("symbol", Some(id), None, None, None, None) => ValueRef::ActualSymbol(id),
            ("index", None, None, None, None, Some(id)) => ValueRef::Index(BoundIndexId::from_id(id)),
            ("domain", None, template, name, domain, None) => ValueRef::Domain(DomainRef::from_columns(domain.map(DomainId::from_id), template, name)?),
            (kind, None, Some(template_id), Some(name), None, None) => ValueRef::Template { template_id, name, kind: TemplateValueKind::ALL.into_iter().find(|value| value.as_str() == kind).ok_or_else(|| malformed("unknown template reference kind"))? },
            _ => return Err(malformed("overlapping or incomplete reference alternatives")),
        };
        sink.symbol_ref(NodeId(row.node_id), reference)?;
    }
}
