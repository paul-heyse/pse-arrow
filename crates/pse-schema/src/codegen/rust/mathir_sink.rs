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
        use pse_mathir::{MathIrError, NodeId, Opcode, Payload, relations::{MathRelationSink, InputBinding, ParameterBinding}}; use pse_schema::math::{Sense};
        use pse_quantity::{BoundIndexId, ConversionId, DomainId, InvariantId, OperationId, QuantityTypeId, UnitId, infer::BuiltInRule};
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
            let encoded = super::mathir_value::encode(spec)?;
            prelude.extend(quote! {
                if let Payload::Affine { terms, .. } = payload
                    && !terms.iter().map(|term| term.child).eq(children.iter().copied()) {
                    return Err(malformed("affine coefficients must follow the ordered child list"));
                }
            });
            prelude.extend(quote!(crate::mathir_relations::check_payload_family(payload, matches!(self.family, Family::Normalized { .. }), !matches!(self.family, Family::Compiled))?;));
            set!("payload", encoded);
            set!(
                "children",
                quote!(
                    children
                        .iter()
                        .map(|child| self.node(*child))
                        .collect::<Result<_, MathIrError>>()?
                )
            );
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
                    "constraint",
                    super::mathir_value::equation_constraint(spec)?,
                ),
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
            set!("position", quote!(i64::from(position)));
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
            set!(
                "operand_permutation",
                quote!(permutation.iter().copied().map(i64::from).collect())
            );
            set!(
                "conversions",
                quote!(conversions.iter().map(|(operand, conversion)| #item { operand: i64::from(*operand), conversion_id: conversion.as_id() }).collect())
            );
            set!("deferred_static_check", quote!(deferred_static_check));
        }
        "kernel_binding" => {
            let parameter = super::mathir_value::kernel_parameter(spec)?;
            let input = nested(spec, "input_bindings");
            set!("binding_id", quote!(binding));
            set!("kernel_id", quote!(kernel));
            set!("scope_instance_id", quote!(scope));
            set!(
                "parameter_bindings",
                quote!(parameters.iter().map(|(name,symbol,value,unit)| Ok(#parameter)).collect::<Result<_, MathIrError>>()?)
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
        "expr_node" => quote!(
            size_of_val(children)
                .checked_add(payload.allocation_extent()?)
                .ok_or_else(|| malformed("whole-node allocation extent overflow"))?
        ),
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
