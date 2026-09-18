// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generate conversions between complete Arrow node values and bounded math algorithms.

use super::{error, types};
use crate::{SchemaError, model::RelationSpec};
use arrow_schema::DataType;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;

fn record(
    spec: &RelationSpec,
    path: &[&str],
    values: Vec<(&str, TokenStream)>,
) -> Result<TokenStream, SchemaError> {
    let first = path
        .first()
        .ok_or_else(|| error("empty math value path".into()))?;
    let mut field = spec
        .columns
        .iter()
        .find(|field| field.name() == *first)
        .ok_or_else(|| error(format!("missing math field {first}")))?
        .field()
        .clone();
    let mut stem = format!(
        "{}{}Field{}",
        types::pascal(spec.key.namespace.as_str()),
        types::pascal(spec.key.name),
        types::pascal(first)
    );
    for name in &path[1..] {
        field = match (field.data_type(), *name) {
            (DataType::List(child), "item") => (**child).clone(),
            (DataType::Struct(fields), name) => fields
                .iter()
                .find(|field| field.name() == name)
                .map(|field| (**field).clone())
                .ok_or_else(|| error(format!("missing math child {name}")))?,
            _ => {
                return Err(error(
                    "math value path is not a declared struct/list".into(),
                ));
            }
        };
        stem.push_str(&types::pascal(name));
    }
    let DataType::Struct(fields) = field.data_type() else {
        return Err(error("math value constructor needs a struct".into()));
    };
    let mut values: BTreeMap<_, _> = values.into_iter().collect();
    let fields = fields
        .iter()
        .map(|field| {
            let name = types::ident(field.name());
            let value = match values.remove(field.name().as_str()) {
                Some(value) => value,
                None if field.is_nullable() => quote!(None),
                None => {
                    return Err(error(format!(
                        "missing required math value {}",
                        field.name()
                    )));
                }
            };
            Ok(quote!(#name: #value))
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    if !values.is_empty() {
        return Err(error("math value contains undeclared fields".into()));
    }
    let name = types::ident(&stem);
    let namespace = types::ident(spec.key.namespace.as_str());
    let relation = types::ident(spec.key.name);
    Ok(quote!(pse_relations::generated::#namespace::#relation::#name { #(#fields),* }))
}
fn at(path: &[&str], child: &str) -> Vec<String> {
    path.iter()
        .copied()
        .chain([child])
        .map(str::to_owned)
        .collect()
}
fn tagged(
    spec: &RelationSpec,
    path: &[&str],
    tag: &str,
    fields: Vec<(&str, TokenStream)>,
) -> Result<TokenStream, SchemaError> {
    let child = at(path, tag);
    let child = child.iter().map(String::as_str).collect::<Vec<_>>();
    let value = record(spec, &child, fields)?;
    record(
        spec,
        path,
        vec![
            ("kind", quote!(#tag.parse().map_err(adapter_error)?)),
            (tag, quote!(Some(#value))),
        ],
    )
}
fn domain(spec: &RelationSpec, path: &[&str]) -> Result<TokenStream, SchemaError> {
    let actual = tagged(
        spec,
        path,
        "actual",
        vec![("domain_id", quote!(id.as_id()))],
    )?;
    let template = tagged(
        spec,
        path,
        "template",
        vec![
            ("template_id", quote!(*template_id)),
            ("name", quote!(domain_name.clone())),
        ],
    )?;
    Ok(
        quote!(match domain { pse_mathir::DomainRef::Actual(id) => #actual, pse_mathir::DomainRef::Template { template_id, domain_name } => #template }),
    )
}
fn guard(spec: &RelationSpec, path: &[&str]) -> Result<TokenStream, SchemaError> {
    let math = tagged(
        spec,
        path,
        "math",
        vec![("node_id", quote!(self.node(*node)?))],
    )?;
    let predicate = tagged(
        spec,
        path,
        "predicate",
        vec![
            ("source_id", quote!(*source_id)),
            ("predicate_id", quote!(*predicate_id)),
        ],
    )?;
    Ok(
        quote!(match guard { pse_mathir::GuardRef::Math(node) => #math, pse_mathir::GuardRef::Predicate { source_id, predicate_id } => #predicate }),
    )
}
fn optional_guard(spec: &RelationSpec, path: &[&str]) -> Result<TokenStream, SchemaError> {
    let value = guard(spec, path)?;
    Ok(quote!(match filter { Some(guard) => Some(#value), None => None }))
}
fn reference(spec: &RelationSpec) -> Result<TokenStream, SchemaError> {
    let path = &["payload", "symbol", "reference"];
    let symbol = tagged(spec, path, "symbol", vec![("symbol_id", quote!(*id))])?;
    let template = tagged(
        spec,
        path,
        "template",
        vec![
            ("template_id", quote!(*template_id)),
            (
                "member_kind",
                quote!(kind.as_str().parse().map_err(adapter_error)?),
            ),
            ("name", quote!(name.clone())),
        ],
    )?;
    let inner = domain(spec, &["payload", "symbol", "reference", "domain", "value"])?;
    let domain = tagged(spec, path, "domain", vec![("value", inner)])?;
    let index = tagged(
        spec,
        path,
        "index",
        vec![("bound_index_id", quote!(id.as_id()))],
    )?;
    Ok(quote!(match symbol {
        pse_mathir::ValueRef::ActualSymbol(id) => #symbol,
        pse_mathir::ValueRef::Template { template_id, kind, name } => #template,
        pse_mathir::ValueRef::Domain(domain) => #domain,
        pse_mathir::ValueRef::Index(id) => #index,
    }))
}

pub(super) fn encode(spec: &RelationSpec) -> Result<TokenStream, SchemaError> {
    let mut arms = Vec::new();
    macro_rules! arm {
        ($tag:literal, $pattern:expr, $( $field:literal => $value:expr ),* $(,)?) => {{
            let value = tagged(spec, &["payload"], $tag, vec![$(($field, $value)),*])?;
            let pattern = $pattern;
            arms.push(quote!(#pattern => #value));
        }};
    }
    let none = record(
        spec,
        &["payload"],
        vec![("kind", quote!("none".parse().map_err(adapter_error)?))],
    )?;
    arms.push(quote!(Payload::None => #none));
    arm!("symbol", quote!(Payload::SymbolRef { symbol }), "reference" => reference(spec)?);
    arm!("float", quote!(Payload::FloatConst { value, unit }), "value" => quote!(*value), "unit_id" => quote!(unit.as_id()));
    arm!("integer", quote!(Payload::IntConst { value }), "value" => quote!(*value));
    arm!("affine", quote!(Payload::Affine { constant, constant_quantity_type, constant_unit, terms }),
        "constant" => quote!(*constant), "constant_quantity_type_id" => quote!(constant_quantity_type.map(QuantityTypeId::as_id)), "constant_unit_id" => quote!(constant_unit.map(UnitId::as_id)),
        "coefficients" => quote!(terms.iter().map(|term| term.coefficient).collect()));
    let pair = record(
        spec,
        &["payload", "weighted_mean", "pairs", "item"],
        vec![
            ("weight_node_id", quote!(self.node(pair.weight)?)),
            ("value_node_id", quote!(self.node(pair.value)?)),
        ],
    )?;
    arm!("weighted_mean", quote!(Payload::WeightedMean { pairs, normalization, unit_sum_invariant }),
        "pairs" => quote!(pairs.iter().map(|pair| Ok(#pair)).collect::<Result<_, MathIrError>>()?), "normalization" => quote!(normalization.as_str().parse().map_err(adapter_error)?), "unit_sum_invariant_id" => quote!(unit_sum_invariant.map(InvariantId::as_id)));
    arm!("reduction", quote!(Payload::Reduction { kind, domain, bound_index, filter }),
        "reduction_kind" => quote!(kind.as_str().parse().map_err(adapter_error)?), "domain" => domain(spec, &["payload", "reduction", "domain"])?, "bound_index_id" => quote!(bound_index.as_id()), "filter" => optional_guard(spec, &["payload", "reduction", "filter"])?);
    let coordinate = record(
        spec,
        &["payload", "gather", "coordinates", "item"],
        vec![
            ("bound_index_id", quote!(id.as_id())),
            ("position", quote!(i64::from(*position))),
        ],
    )?;
    arm!("gather", quote!(Payload::Gather { group, coordinate_map }), "group_id" => quote!(*group), "coordinates" => quote!(coordinate_map.iter().map(|(id, position)| #coordinate).collect()));
    arm!("pending_gather", quote!(Payload::PendingGather { group, indices }), "group_id" => quote!(*group), "indices" => quote!(indices.iter().map(|id| self.node(*id)).collect::<Result<_, _>>()?));
    arm!("pending_path", quote!(Payload::PendingPath { source_id, path_id, indices }), "source_id" => quote!(*source_id), "path_id" => quote!(*path_id), "indices" => quote!(indices.iter().map(|id| self.node(*id)).collect::<Result<_, _>>()?));
    arm!("broadcast", quote!(Payload::Broadcast { domain, bound_index }), "domain" => domain(spec, &["payload", "broadcast", "domain"])?, "bound_index_id" => quote!(bound_index.as_id()));
    arm!("derivative", quote!(Payload::Derivative { wrt_domain: domain, order }), "domain" => domain(spec, &["payload", "derivative", "domain"])?, "order" => quote!(i64::from(*order)));
    arm!("integral", quote!(Payload::Integral { domain, bound_index, quadrature_policy, filter }), "domain" => domain(spec, &["payload", "integral", "domain"])?, "bound_index_id" => quote!(bound_index.as_id()), "quadrature_policy_id" => quote!(*quadrature_policy), "filter" => optional_guard(spec, &["payload", "integral", "filter"])?);
    arm!("smooth", quote!(Payload::SmoothOp { eps }), "eps" => quote!(*eps));
    arm!("pending_smooth", quote!(Payload::PendingSmoothOp { eps, unit }), "eps" => quote!(*eps), "unit_id" => quote!(unit.as_id()));
    arm!("conditional", quote!(Payload::Conditional { guard }), "guard" => guard(spec, &["payload", "conditional", "guard"])?);
    arm!("kernel_call", quote!(Payload::KernelCall { kernel_binding, output_ordinal }), "binding_id" => quote!(*kernel_binding), "output_ordinal" => quote!(i64::from(*output_ordinal)));
    arm!("implicit_ref", quote!(Payload::ImplicitRef { implicit_system, unknown_ordinal }), "system_id" => quote!(*implicit_system), "unknown_ordinal" => quote!(i64::from(*unknown_ordinal)));
    arm!("unit_convert", quote!(Payload::UnitConvert(pse_quantity::UnitConvertSpec { scale, offset, from, to })), "scale" => quote!(*scale), "offset" => quote!(*offset), "from_unit_id" => quote!(from.as_id()), "to_unit_id" => quote!(to.as_id()));
    arm!("pending_unit_convert", quote!(Payload::PendingUnitConvert { to }), "to_unit_id" => quote!(to.as_id()));
    let point = record(
        spec,
        &["payload", "piecewise_linear", "breakpoints", "item"],
        vec![("x", quote!(*x)), ("y", quote!(*y))],
    )?;
    arm!("piecewise_linear", quote!(Payload::PiecewiseLinear { breakpoints, input, output }), "breakpoints" => quote!(breakpoints.iter().map(|(x, y)| #point).collect()), "input_quantity_type_id" => quote!(input.as_id()), "output_quantity_type_id" => quote!(output.as_id()));
    Ok(
        quote!(match payload { #(#arms,)* _ => return Err(malformed("unsupported mathematical payload")) }),
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive mechanical conversion of the declared payload alternatives"
)]
pub(super) fn decode() -> TokenStream {
    quote!({
        macro_rules! required {
            ($value:expr) => {
                $value.ok_or_else(|| malformed("selected mathematical value arm is absent"))?
            };
        }
        macro_rules! ordinal {
            ($value:expr) => {{
                let value = $value;
                if value < 0 {
                    return Err(malformed("negative mathematical ordinal"));
                }
                value
            }};
        }
        macro_rules! domain {
            ($value:expr) => {{
                let value = $value;
                match value.kind.as_str() {
                    "actual" => {
                        DomainRef::Actual(DomainId::from_id(required!(value.actual).domain_id))
                    }
                    "template" => {
                        let value = required!(value.template);
                        DomainRef::Template {
                            template_id: value.template_id,
                            domain_name: value.name,
                        }
                    }
                    _ => return Err(malformed("unknown mathematical domain alternative")),
                }
            }};
        }
        macro_rules! guard {
            ($value:expr) => {{
                let value = $value;
                match value.kind.as_str() {
                    "math" => GuardRef::Math(NodeId(ordinal!(required!(value.math).node_id))),
                    "predicate" => {
                        let value = required!(value.predicate);
                        GuardRef::Predicate {
                            source_id: value.source_id,
                            predicate_id: ordinal!(value.predicate_id),
                        }
                    }
                    _ => return Err(malformed("unknown mathematical guard alternative")),
                }
            }};
        }
        let value = row.payload;
        match value.kind.as_str() {
            "none" => Payload::None,
            "symbol" => {
                let value = required!(value.symbol).reference;
                let symbol = match value.kind.as_str() {
                    "symbol" => ValueRef::ActualSymbol(required!(value.symbol).symbol_id),
                    "template" => {
                        let value = required!(value.template);
                        ValueRef::Template {
                            template_id: value.template_id,
                            kind: TemplateValueKind::ALL
                                .into_iter()
                                .find(|kind| kind.as_str() == value.member_kind.as_str())
                                .ok_or_else(|| malformed("invalid template member kind"))?,
                            name: value.name,
                        }
                    }
                    "domain" => ValueRef::Domain(domain!(required!(value.domain).value)),
                    "index" => ValueRef::Index(BoundIndexId::from_id(
                        required!(value.index).bound_index_id,
                    )),
                    _ => return Err(malformed("unknown mathematical reference alternative")),
                };
                Payload::SymbolRef { symbol }
            }
            "float" => {
                let value = required!(value.float);
                Payload::FloatConst {
                    value: value.value,
                    unit: UnitId::from_id(value.unit_id),
                }
            }
            "integer" => Payload::IntConst {
                value: required!(value.integer).value,
            },
            "affine" => {
                let value = required!(value.affine);
                if value.coefficients.len() != row.children.len() {
                    return Err(malformed("affine coefficient/child cardinality differs"));
                }
                Payload::Affine {
                    constant: value.constant,
                    constant_quantity_type: value
                        .constant_quantity_type_id
                        .map(QuantityTypeId::from_id),
                    constant_unit: value.constant_unit_id.map(UnitId::from_id),
                    terms: value
                        .coefficients
                        .into_iter()
                        .zip(&row.children)
                        .map(|(coefficient, child)| {
                            Ok(pse_mathir::AffineTerm {
                                coefficient,
                                child: NodeId(ordinal!(*child)),
                            })
                        })
                        .collect::<Result<_, MathIrError>>()?,
                }
            }
            "weighted_mean" => {
                let value = required!(value.weighted_mean);
                Payload::WeightedMean {
                    pairs: value
                        .pairs
                        .into_iter()
                        .map(|pair| {
                            Ok(pse_mathir::WeightedPair {
                                weight: NodeId(ordinal!(pair.weight_node_id)),
                                value: NodeId(ordinal!(pair.value_node_id)),
                            })
                        })
                        .collect::<Result<_, MathIrError>>()?,
                    normalization: enumeration(
                        value.normalization.as_str(),
                        WeightNormalization::parse,
                    )?,
                    unit_sum_invariant: value.unit_sum_invariant_id.map(InvariantId::from_id),
                }
            }
            "reduction" => {
                let value = required!(value.reduction);
                Payload::Reduction {
                    kind: enumeration(value.reduction_kind.as_str(), ReductionKind::parse)?,
                    domain: domain!(value.domain),
                    bound_index: BoundIndexId::from_id(value.bound_index_id),
                    filter: value.filter.map(|value| Ok(guard!(value))).transpose()?,
                }
            }
            "gather" => {
                let value = required!(value.gather);
                Payload::Gather {
                    group: value.group_id,
                    coordinate_map: value
                        .coordinates
                        .into_iter()
                        .map(|value| {
                            Ok((
                                BoundIndexId::from_id(value.bound_index_id),
                                u16::try_from(value.position)
                                    .map_err(|_| malformed("coordinate width"))?,
                            ))
                        })
                        .collect::<Result<_, MathIrError>>()?,
                }
            }
            "pending_gather" => {
                let value = required!(value.pending_gather);
                Payload::PendingGather {
                    group: value.group_id,
                    indices: value
                        .indices
                        .into_iter()
                        .map(|id| Ok(NodeId(ordinal!(id))))
                        .collect::<Result<_, MathIrError>>()?,
                }
            }
            "pending_path" => {
                let value = required!(value.pending_path);
                Payload::PendingPath {
                    source_id: value.source_id,
                    path_id: ordinal!(value.path_id),
                    indices: value
                        .indices
                        .into_iter()
                        .map(|id| Ok(NodeId(ordinal!(id))))
                        .collect::<Result<_, MathIrError>>()?,
                }
            }
            "broadcast" => {
                let value = required!(value.broadcast);
                Payload::Broadcast {
                    domain: domain!(value.domain),
                    bound_index: BoundIndexId::from_id(value.bound_index_id),
                }
            }
            "derivative" => {
                let value = required!(value.derivative);
                Payload::Derivative {
                    wrt_domain: domain!(value.domain),
                    order: u8::try_from(value.order)
                        .map_err(|_| malformed("derivative order width"))?,
                }
            }
            "integral" => {
                let value = required!(value.integral);
                Payload::Integral {
                    domain: domain!(value.domain),
                    bound_index: BoundIndexId::from_id(value.bound_index_id),
                    quadrature_policy: value.quadrature_policy_id,
                    filter: value.filter.map(|value| Ok(guard!(value))).transpose()?,
                }
            }
            "smooth" => Payload::SmoothOp {
                eps: required!(value.smooth).eps,
            },
            "pending_smooth" => {
                let value = required!(value.pending_smooth);
                Payload::PendingSmoothOp {
                    eps: value.eps,
                    unit: UnitId::from_id(value.unit_id),
                }
            }
            "conditional" => Payload::Conditional {
                guard: guard!(required!(value.conditional).guard),
            },
            "kernel_call" => {
                let value = required!(value.kernel_call);
                Payload::KernelCall {
                    kernel_binding: value.binding_id,
                    output_ordinal: u16::try_from(value.output_ordinal)
                        .map_err(|_| malformed("kernel output width"))?,
                }
            }
            "implicit_ref" => {
                let value = required!(value.implicit_ref);
                Payload::ImplicitRef {
                    implicit_system: value.system_id,
                    unknown_ordinal: u16::try_from(value.unknown_ordinal)
                        .map_err(|_| malformed("implicit unknown width"))?,
                }
            }
            "unit_convert" => {
                let value = required!(value.unit_convert);
                Payload::UnitConvert(pse_quantity::UnitConvertSpec {
                    scale: value.scale,
                    offset: value.offset,
                    from: UnitId::from_id(value.from_unit_id),
                    to: UnitId::from_id(value.to_unit_id),
                })
            }
            "pending_unit_convert" => Payload::PendingUnitConvert {
                to: UnitId::from_id(required!(value.pending_unit_convert).to_unit_id),
            },
            "piecewise_linear" => {
                let value = required!(value.piecewise_linear);
                Payload::PiecewiseLinear {
                    breakpoints: value
                        .breakpoints
                        .into_iter()
                        .map(|point| (point.x, point.y))
                        .collect(),
                    input: QuantityTypeId::from_id(value.input_quantity_type_id),
                    output: QuantityTypeId::from_id(value.output_quantity_type_id),
                }
            }
            _ => return Err(malformed("unknown mathematical payload alternative")),
        }
    })
}

/// One declared equation value, with no optional-bound combinations in storage.
pub(super) fn equation_constraint(spec: &RelationSpec) -> Result<TokenStream, SchemaError> {
    let single = record(
        spec,
        &["constraint", "single"],
        vec![("node_id", quote!(self.node(target)?))],
    )?;
    let range = record(
        spec,
        &["constraint", "range"],
        vec![
            ("lower_node_id", quote!(self.node(lower)?)),
            ("upper_node_id", quote!(self.node(upper)?)),
        ],
    )?;
    let value = record(
        spec,
        &["constraint"],
        vec![
            (
                "kind",
                quote!(sense.as_str().parse().map_err(adapter_error)?),
            ),
            ("single", quote!(selected_single)),
            ("range", quote!(selected_range)),
        ],
    )?;
    Ok(quote!({
        let (selected_single, selected_range) = match (sense, lower, upper) {
            (Sense::Range, Some(lower), Some(upper)) => (None, Some(#range)),
            (Sense::Ge | Sense::Eq | Sense::Definition, Some(target), None)
            | (Sense::Le | Sense::Eq | Sense::Definition, None, Some(target)) => (Some(#single), None),
            _ => return Err(malformed("equation bound shape disagrees with its sense")),
        };
        #value
    }))
}

/// Generate the sole native parameter selection from the bounded algorithm input.
pub(super) fn kernel_parameter(spec: &RelationSpec) -> Result<TokenStream, SchemaError> {
    let symbol = record(
        spec,
        &["parameter_bindings", "item", "binding", "symbol"],
        vec![("symbol_id", quote!(bound_symbol))],
    )?;
    let literal = record(
        spec,
        &["parameter_bindings", "item", "binding", "literal"],
        vec![
            ("value", quote!(bound_value)),
            ("unit_id", quote!(bound_unit.as_id())),
        ],
    )?;
    let binding = record(
        spec,
        &["parameter_bindings", "item", "binding"],
        vec![
            ("kind", quote!(kind.parse().map_err(adapter_error)?)),
            ("symbol", quote!(selected_symbol)),
            ("literal", quote!(selected_literal)),
        ],
    )?;
    let item = record(
        spec,
        &["parameter_bindings", "item"],
        vec![("name", quote!(name.clone())), ("binding", binding)],
    )?;
    Ok(quote!({
        let (kind, selected_symbol, selected_literal) = match (*symbol, *value, *unit) {
            (Some(bound_symbol), None, None) => ("symbol", Some(#symbol), None),
            (None, Some(bound_value), Some(bound_unit)) if bound_value.is_finite() => ("literal", None, Some(#literal)),
            _ => return Err(malformed("kernel parameter requires a symbol or finite literal with unit")),
        };
        #item
    }))
}
