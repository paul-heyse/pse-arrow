// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The actual `MathRelationSink` API projected into declared generated Arrow rows.

use super::{error, types};
use crate::{
    Registry, SchemaError,
    model::{Namespace, RelationSpec},
};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};
use syn::visit_mut::VisitMut;

pub(super) fn render(registry: &Registry) -> Result<TokenStream, SchemaError> {
    // Signatures come from the existing Arrow-free API, not a second method inventory.
    let api = syn::parse_file(include_str!("../../../../pse-mathir/src/relations/mod.rs"))
        .map_err(|cause| error(cause.to_string()))?;
    let api = api
        .items
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Trait(item) if item.ident == "MathRelationSink" => Some(item),
            _ => None,
        })
        .ok_or_else(|| error("MathRelationSink API disappeared".into()))?;
    let mut methods = Vec::new();
    for item in api.items {
        let syn::TraitItem::Fn(method) = item else {
            return Err(error("unsupported MathRelationSink associated item".into()));
        };
        let mut signature = method.sig;
        Signature.visit_signature_mut(&mut signature);
        let method = signature.ident.to_string();
        let source = source(&method)?;
        let mut arms = Vec::new();
        let mut covered = BTreeSet::new();
        for namespace in [
            Namespace::Compiled,
            Namespace::Inferred,
            Namespace::Normalized,
        ] {
            let prefixes: &[&str] = if namespace == Namespace::Normalized {
                &["template", "instance", "display", "contribution", "guard"]
            } else {
                &[""]
            };
            for prefix in prefixes {
                let name = if namespace == Namespace::Normalized {
                    let Some(name) = crate::catalog::expr_family::target_name(prefix, source)
                    else {
                        continue;
                    };
                    name
                } else {
                    source
                };
                let spec = registry
                    .relation(&format!("{namespace}.{name}"))
                    .ok_or_else(|| error(format!("missing math output {namespace}.{name}")))?;
                if !supported(&method, spec) {
                    continue;
                }
                let body = row(&method, spec)?;
                let pattern = match namespace {
                    Namespace::Compiled => quote!(Family::Compiled),
                    Namespace::Inferred => quote!(Family::Inferred),
                    _ => quote!(Family::Normalized { prefix: #prefix, .. }),
                };
                arms.push(quote!(#pattern => { #body }));
                covered.insert(namespace);
            }
        }
        let mut unavailable = vec![quote!(Family::Normalized { .. })];
        if !covered.contains(&Namespace::Compiled) {
            unavailable.push(quote!(Family::Compiled));
        }
        if !covered.contains(&Namespace::Inferred) {
            unavailable.push(quote!(Family::Inferred));
        }
        methods.push(quote! { #signature { match self.family.clone() { #(#arms,)* #(#unavailable)|* => Err(malformed("mathematical callback is unavailable in this declared storage family")) } } });
    }
    Ok(quote! {
        //! Existing mathematical callbacks write direct generated columns.
        use pse_ids::{ContentHash, SemanticId};
        use pse_mathir::{MathIrError, NodeId, Opcode, relations::{MathRelationSink, InputBinding, ParameterBinding}}; use pse_schema::math::{Sense};
        use pse_quantity::{BoundIndexId, ConversionId, DomainId, InvariantId, OperationId, QuantityTypeId, ReductionKind, UnitId, WeightNormalization, infer::BuiltInRule};
        use crate::mathir_relations::{Family, RelationSink, malformed, sink::adapter_error};
        impl MathRelationSink for RelationSink<'_> { #(#methods)* }
    })
}

struct Signature;
impl VisitMut for Signature {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Some(first) = path.segments.first_mut()
            && first.ident == "crate"
        {
            first.ident = types::ident("pse_mathir");
        }
        syn::visit_mut::visit_path_mut(self, path);
    }
    fn visit_pat_ident_mut(&mut self, pattern: &mut syn::PatIdent) {
        let name = pattern.ident.to_string();
        if let Some(name) = name.strip_prefix('_') {
            pattern.ident = types::ident(name);
        }
    }
}
fn source(method: &str) -> Result<&'static str, SchemaError> {
    Ok(match method {
        "expr_node" => "math_expr_nodes",
        "expr_arg" => "math_expr_args",
        "symbol_ref" | "pending_path" => "math_symbol_refs",
        "float_constant" => "math_float_constants",
        "int_constant" => "math_int_constants",
        "affine" => "math_affine",
        "weighted_mean" => "math_weighted_means",
        "reduction" => "math_reductions",
        "gather" | "pending_gather" => "math_gathers",
        "broadcast" => "math_broadcasts",
        "derivative" => "math_derivatives",
        "integral" => "math_integrals",
        "smooth_op" | "pending_smooth_op" => "math_smooth_ops",
        "conditional" => "math_conditionals",
        "kernel_call" => "math_kernel_calls",
        "implicit_ref" => "math_implicit_refs",
        "unit_convert" | "pending_unit_convert" => "math_unit_converts",
        "piecewise_linear" => "math_piecewise_linear",
        "indexed_equation" => "math_indexed_equations",
        "free_index" => "math_free_indices",
        "quantity_selection" => "math_quantity_selections",
        "kernel_binding" => "kernel_bindings",
        _ => {
            return Err(error(format!(
                "new MathRelationSink callback {method} needs its declared projection"
            )));
        }
    })
}
fn has(spec: &RelationSpec, field: &str) -> bool {
    spec.columns.iter().any(|column| column.name() == field)
}
fn optional(spec: &RelationSpec, field: &str, value: TokenStream) -> TokenStream {
    if spec
        .columns
        .iter()
        .any(|column| column.name() == field && column.nullable())
    {
        quote!(Some(#value))
    } else {
        value
    }
}
fn supported(method: &str, spec: &RelationSpec) -> bool {
    match method {
        "pending_path" => has(spec, "path_id"),
        "pending_gather" => has(spec, "gather_state"),
        "pending_smooth_op" => has(spec, "epsilon_state"),
        "pending_unit_convert" => has(spec, "conversion_state"),
        _ => true,
    }
}
fn nested(spec: &RelationSpec, field: &str) -> TokenStream {
    let namespace = types::ident(spec.key.namespace.as_str());
    let module = types::ident(spec.key.name);
    let name = types::ident(&format!(
        "{}{}Field{}Item",
        types::pascal(spec.key.namespace.as_str()),
        types::pascal(spec.key.name),
        types::pascal(field)
    ));
    quote!(pse_relations::generated::#namespace::#module::#name)
}

#[expect(
    clippy::too_many_lines,
    reason = "Mechanical mapping of the complete mathematical family to its generated Arrow adapter"
)]
fn row(method: &str, spec: &RelationSpec) -> Result<TokenStream, SchemaError> {
    let mut fields = BTreeMap::<&str, TokenStream>::new();
    let mut prelude = TokenStream::new();
    if has(spec, "node_id") {
        fields.insert("node_id", quote!(self.node(node)?));
    }
    macro_rules! set {
        ($name:literal, $value:expr) => {
            fields.insert($name, $value);
        };
    }
    match method {
        "expr_node" => {
            set!(
                "opcode",
                quote!(opcode.as_str().parse().map_err(adapter_error)?)
            );
            set!(
                "quantity_type_id",
                quote!(quantity_type.map(QuantityTypeId::as_id))
            );
            set!("scope_instance_id", quote!(scope));
            set!("subtree_hash", quote!(hash));
            if has(spec, "source_span") {
                prelude.extend(quote!(let (derivation, span) = self.provenance()?;));
                set!("derivation_id", quote!(derivation));
                set!(
                    "source_span",
                    quote!(pse_relations::generated::extension_values::SourceSpan {
                        document_id: span.document_id,
                        start: i64::from(span.start),
                        end: i64::from(span.end)
                    })
                );
            }
        }
        "expr_arg" => {
            set!("parent_node_id", quote!(self.node(parent)?));
            set!("argument_ordinal", quote!(ordinal));
            set!("child_node_id", quote!(self.node(child)?));
        }
        "symbol_ref" if !has(spec, "kind") => {
            prelude.extend(quote!(let pse_mathir::ValueRef::ActualSymbol(symbol) = symbol else { return Err(malformed("unresolved declaration reference in instantiated output")); };));
            set!("symbol_id", quote!(symbol));
        }
        "symbol_ref" => {
            prelude.extend(quote!(let (kind, symbol, template, name, domain, index) = Self::reference_parts(symbol);));
            for (name, value) in [
                ("kind", quote!(kind.parse().map_err(adapter_error)?)),
                ("symbol_id", quote!(symbol)),
                ("template_id", quote!(template)),
                ("name", quote!(name)),
                ("domain_id", quote!(domain)),
                ("bound_index_id", quote!(index)),
                ("path_source_id", quote!(None)),
                ("path_id", quote!(None)),
                ("path_index_nodes", quote!(None)),
            ] {
                fields.insert(name, value);
            }
        }
        "pending_path" => {
            for name in [
                "symbol_id",
                "template_id",
                "name",
                "domain_id",
                "bound_index_id",
            ] {
                fields.insert(name, quote!(None));
            }
            set!(
                "kind",
                quote!(
                    pse_schema::math::PENDING_PATH_KIND
                        .parse()
                        .map_err(adapter_error)?
                )
            );
            set!("path_source_id", quote!(Some(source_id)));
            set!("path_id", quote!(Some(path_id)));
            set!(
                "path_index_nodes",
                quote!(Some(
                    indices
                        .iter()
                        .map(|node| self.node(*node))
                        .collect::<Result<_, _>>()?
                ))
            );
        }
        "float_constant" => {
            set!("value", quote!(value));
            set!("unit_id", quote!(unit.as_id()));
        }
        "int_constant" => {
            set!("value", quote!(value));
        }
        "affine" => {
            let item = nested(spec, "terms");
            set!("constant", quote!(constant));
            set!(
                "constant_quantity_type_id",
                quote!(constant_quantity_type.map(QuantityTypeId::as_id))
            );
            set!("constant_unit_id", quote!(constant_unit.map(UnitId::as_id)));
            set!(
                "terms",
                quote!(terms.iter().map(|(coefficient, child)| Ok(#item { coefficient: *coefficient, child_node_id: self.node(*child)? })).collect::<Result<_, MathIrError>>()?)
            );
        }
        "weighted_mean" => {
            let item = nested(spec, "pairs");
            set!(
                "pairs",
                quote!(pairs.iter().map(|(weight, value)| Ok(#item { weight_node_id: self.node(*weight)?, value_node_id: self.node(*value)? })).collect::<Result<_, MathIrError>>()?)
            );
            set!(
                "normalization",
                quote!(normalization.as_str().parse().map_err(adapter_error)?)
            );
            set!(
                "unit_sum_invariant_id",
                quote!(certificate.map(InvariantId::as_id))
            );
        }
        "reduction" | "integral" | "broadcast" | "derivative" => {
            prelude.extend(quote!(let (actual, template, name) = self.domain_parts(domain)?;));
            let actual = if has(spec, "template_id") {
                quote!(actual)
            } else {
                quote!(actual.ok_or_else(|| malformed("actual domain absent"))?)
            };
            fields.insert(
                if method == "derivative" {
                    "wrt_domain_id"
                } else {
                    "domain_id"
                },
                actual,
            );
            if has(spec, "template_id") {
                set!("template_id", quote!(template));
                set!("domain_name", quote!(name));
            } else {
                prelude.extend(quote!(let _ = (template, name);));
            }
            if method == "derivative" {
                set!("order", quote!(order));
            } else {
                set!("bound_index_id", quote!(bound_index.as_id()));
            }
            if method == "integral" {
                set!("quadrature_policy_id", quote!(policy));
            }
            if method == "reduction" {
                set!(
                    "kind",
                    quote!(kind.as_str().parse().map_err(adapter_error)?)
                );
            }
            if matches!(method, "reduction" | "integral") {
                add_guard(spec, true, &quote!(filter), &mut fields, &mut prelude);
            }
        }
        "gather" => {
            let item = nested(spec, "coordinate_map");
            set!("group_id", quote!(group));
            fields.insert("coordinate_map", optional(spec, "coordinate_map", quote!(coordinates.iter().map(|(index, position)| #item { bound_index_id: index.as_id(), position: *position }).collect())));
            if has(spec, "gather_state") {
                set!(
                    "gather_state",
                    quote!("resolved".parse().map_err(adapter_error)?)
                );
                set!("index_nodes", quote!(None));
            }
        }
        "pending_gather" => {
            set!("group_id", quote!(group));
            set!("coordinate_map", quote!(None));
            set!(
                "gather_state",
                quote!("pending".parse().map_err(adapter_error)?)
            );
            set!(
                "index_nodes",
                quote!(Some(
                    indices
                        .iter()
                        .map(|node| self.node(*node))
                        .collect::<Result<_, _>>()?
                ))
            );
        }
        "smooth_op" | "pending_smooth_op" => {
            prelude.extend(quote!(if !eps.is_finite() || eps <= 0.0 {
                return Err(malformed("smoothing epsilon must be finite and positive"));
            }));
            set!("eps", quote!(eps));
            if has(spec, "epsilon_state") {
                let pending = method == "pending_smooth_op";
                let state = if pending {
                    "pending_unit"
                } else {
                    "coordinate"
                };
                set!(
                    "epsilon_state",
                    quote!(#state.parse().map_err(adapter_error)?)
                );
                set!(
                    "eps_unit_id",
                    if pending {
                        quote!(Some(unit.as_id()))
                    } else {
                        quote!(None)
                    }
                );
            }
        }
        "conditional" => add_guard(spec, false, &quote!(Some(guard)), &mut fields, &mut prelude),
        "kernel_call" => {
            set!("kernel_binding_id", quote!(binding));
            set!("output_ordinal", quote!(output));
        }
        "implicit_ref" => {
            set!("implicit_system_id", quote!(system));
            set!("unknown_ordinal", quote!(unknown));
        }
        "unit_convert" => {
            set!("scale", optional(spec, "scale", quote!(scale)));
            set!("offset", optional(spec, "offset", quote!(offset)));
            set!(
                "from_unit_id",
                optional(spec, "from_unit_id", quote!(from.as_id()))
            );
            set!("to_unit_id", quote!(to.as_id()));
            if has(spec, "conversion_state") {
                set!(
                    "conversion_state",
                    quote!("resolved".parse().map_err(adapter_error)?)
                );
            }
        }
        "pending_unit_convert" => {
            for field in ["scale", "offset", "from_unit_id"] {
                fields.insert(field, quote!(None));
            }
            set!("to_unit_id", quote!(to.as_id()));
            set!(
                "conversion_state",
                quote!("pending".parse().map_err(adapter_error)?)
            );
        }
        "piecewise_linear" => {
            let item = nested(spec, "breakpoints");
            set!(
                "breakpoints",
                quote!(points.iter().map(|(x,y)| #item { x:*x, y:*y }).collect())
            );
            set!("input_quantity_type_id", quote!(input.as_id()));
            set!("output_quantity_type_id", quote!(output.as_id()));
        }
        "indexed_equation" => {
            for (name, value) in [
                ("indexed_equation_id", quote!(id)),
                ("owner_instance_id", quote!(owner)),
                ("equation_decl_id", quote!(declaration)),
                ("qualified_name", quote!(name.to_owned())),
                ("product_id", quote!(product)),
                ("filter_node_id", quote!(self.optional_node(filter)?)),
                ("body_node_id", quote!(self.node(body)?)),
                (
                    "sense",
                    quote!(sense.as_str().parse().map_err(adapter_error)?),
                ),
                ("lower_node_id", quote!(self.optional_node(lower)?)),
                ("upper_node_id", quote!(self.optional_node(upper)?)),
                (
                    "residual_quantity_type_id",
                    quote!(residual.map(QuantityTypeId::as_id)),
                ),
                ("law_instance_id", quote!(law)),
                ("derivation_id", quote!(derivation)),
            ] {
                fields.insert(name, value);
            }
        }
        "free_index" => {
            set!("indexed_equation_id", quote!(equation));
            set!("bound_index_id", quote!(binder.as_id()));
            set!("domain_id", quote!(domain.as_id()));
            set!("position", quote!(position));
        }
        "quantity_selection" => {
            let item = nested(spec, "conversions");
            set!("operation_id", quote!(operation.map(OperationId::as_id)));
            set!(
                "builtin_rule",
                quote!(
                    builtin
                        .map(|value| value.as_str().parse().map_err(adapter_error))
                        .transpose()?
                )
            );
            set!("operand_permutation", quote!(permutation.to_vec()));
            set!(
                "conversions",
                quote!(conversions.iter().map(|(operand, conversion)| #item { operand: *operand, conversion_id: conversion.as_id() }).collect())
            );
            set!("deferred_static_check", quote!(deferred_static_check));
        }
        "kernel_binding" => {
            let parameter = nested(spec, "parameter_bindings");
            let input = nested(spec, "input_bindings");
            set!("binding_id", quote!(binding));
            set!("kernel_id", quote!(kernel));
            set!("scope_instance_id", quote!(scope));
            set!(
                "parameter_bindings",
                quote!(parameters.iter().map(|(name,symbol,value,unit)| #parameter { name:name.clone(), symbol_id:*symbol, value:*value, unit_id:unit.map(UnitId::as_id) }).collect())
            );
            set!(
                "input_bindings",
                quote!(inputs.iter().map(|(name,node)| Ok(#input { name:name.clone(), node_id:self.node(*node)? })).collect::<Result<_, MathIrError>>()?)
            );
        }
        _ => {
            return Err(error(format!(
                "missing mathematical output projection for {method}"
            )));
        }
    }
    let row = spec
        .columns
        .iter()
        .map(|column| {
            let field = types::ident(column.name());
            let value = fields.remove(column.name()).ok_or_else(|| {
                error(format!(
                    "{method} omits declared {}.{}",
                    spec.key,
                    column.name()
                ))
            })?;
            Ok(
                if value.to_string().trim_start_matches("r#") == column.name() {
                    quote!(#field)
                } else {
                    quote!(#field: #value)
                },
            )
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    if !fields.is_empty() {
        return Err(error(format!(
            "{method} adds undeclared {} fields",
            spec.key
        )));
    }
    let namespace = types::ident(spec.key.namespace.as_str());
    let module = types::ident(spec.key.name);
    let payload = payload_extent(method);
    Ok(
        quote! { let _row_allocation = self.reserve_row(#payload)?; #prelude let row = pse_relations::generated::#namespace::#module::Row { #(#row),* }; self.columns.push(row).map_err(adapter_error) },
    )
}

fn payload_extent(method: &str) -> TokenStream {
    match method {
        "pending_path" | "pending_gather" => quote!(size_of_val(indices)),
        "affine" => quote!(size_of_val(terms)),
        "weighted_mean" => quote!(size_of_val(pairs)),
        "gather" => quote!(size_of_val(coordinates)),
        "piecewise_linear" => quote!(size_of_val(points)),
        "indexed_equation" => quote!(name.len()),
        "quantity_selection" => quote!(
            size_of_val(permutation)
                .checked_add(size_of_val(conversions))
                .ok_or_else(|| malformed("quantity selection extent overflow"))?
        ),
        "kernel_binding" => quote!({
            let bytes = size_of_val(parameters)
                .checked_add(size_of_val(inputs))
                .ok_or_else(|| malformed("kernel binding extent overflow"))?;
            parameters
                .iter()
                .map(|(name, ..)| name.len())
                .chain(inputs.iter().map(|(name, ..)| name.len()))
                .try_fold(bytes, usize::checked_add)
                .ok_or_else(|| malformed("kernel binding text extent overflow"))?
        }),
        _ => quote!(0usize),
    }
}

fn add_guard(
    spec: &RelationSpec,
    filter: bool,
    value: &TokenStream,
    fields: &mut BTreeMap<&'static str, TokenStream>,
    prelude: &mut TokenStream,
) {
    let (node, source, predicate) = if filter {
        ("filter_node_id", "filter_source_id", "filter_predicate_id")
    } else {
        ("guard_node_id", "guard_source_id", "guard_predicate_id")
    };
    prelude.extend(
        quote!(let (guard_node, guard_source, guard_predicate) = self.guard_parts(#value)?;),
    );
    fields.insert(
        node,
        if spec
            .columns
            .iter()
            .any(|column| column.name() == node && column.nullable())
        {
            quote!(guard_node)
        } else {
            quote!(guard_node.ok_or_else(|| malformed("required actual guard absent"))?)
        },
    );
    if has(spec, source) {
        fields.insert(source, quote!(guard_source));
        fields.insert(predicate, quote!(guard_predicate));
    } else {
        prelude.extend(quote!(let _ = (guard_source, guard_predicate);));
    }
}
