// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{MathRows, malformed};
use pse_authoring::SourceSpan;
use pse_ids::{ContentHash, SemanticId};
use pse_mathir::relations::{InputBinding, MathRelationSink, ParameterBinding};
use pse_mathir::{DomainRef, GuardRef, MathIrError, NodeId, Opcode, ValueRef};
use pse_quantity::{
    BoundIndexId, ConversionId, DomainId, InvariantId, OperationId, QuantityTypeId, ReductionKind,
    UnitId, WeightNormalization, infer::BuiltInRule,
};
use pse_schema::{Registry, model::Cell};
use std::collections::BTreeMap;

/// A mathematical storage family and its explicit normalization provenance.
#[derive(Clone, Debug)]
pub enum Family {
    /// Fully resolved compiled mathematical relations.
    Compiled,
    /// Parsed source family; offsets make graph-local ordinals unique within that family.
    Normalized {
        /// Declared expression family spelling.
        prefix: &'static str,
        /// Global ordinal offset for this source graph.
        offset: u64,
        /// Source derivation identity.
        derivation: SemanticId,
        /// Original authored field range.
        source_span: SourceSpan,
    },
}

/// Primitive callback adapter; every row is projected by its registered field names.
#[derive(Debug)]
pub struct RelationSink<'a> {
    registry: &'a Registry,
    family: Family,
    rows: MathRows,
}
impl<'a> RelationSink<'a> {
    /// Start an empty projection; semantic graph admission occurs before emission.
    #[must_use]
    pub fn new(registry: &'a Registry, family: Family) -> Self {
        Self {
            registry,
            family,
            rows: MathRows::new(),
        }
    }
    /// Move the exact rows to the caller for budgeted batch construction and admission.
    #[must_use]
    pub fn into_rows(self) -> MathRows {
        self.rows
    }
    fn node(&self, node: NodeId) -> Result<Cell, MathIrError> {
        let offset = match self.family {
            Family::Compiled => 0,
            Family::Normalized { offset, .. } => offset,
        };
        node.0
            .checked_add(offset)
            .map(Cell::U64)
            .ok_or_else(|| malformed("math ordinal overflow"))
    }
    fn optional_node(&self, node: Option<NodeId>) -> Result<Cell, MathIrError> {
        node.map_or(Ok(Cell::Null), |node| self.node(node))
    }
    fn write(&mut self, source: &str, fields: Vec<(&str, Cell)>) -> Result<(), MathIrError> {
        let name = match &self.family {
            Family::Compiled => format!("compiled.{source}"),
            Family::Normalized { prefix, .. } => {
                let target = pse_schema::catalog::expr_family::target_name(prefix, source)
                    .ok_or_else(|| malformed(format!("{source} has no normalized projection")))?;
                format!("normalized.{target}")
            }
        };
        let spec = self
            .registry
            .relation(&name)
            .ok_or_else(|| malformed(format!("undeclared {name}")))?;
        let count = fields.len();
        let mut fields = fields.into_iter().collect::<BTreeMap<_, _>>();
        if fields.len() != count {
            return Err(malformed("duplicate callback field"));
        }
        if source == "math_smooth_ops" {
            validate_smoothing(&fields, matches!(self.family, Family::Normalized { .. }))?;
        }
        let row = spec
            .columns
            .iter()
            .map(|column| {
                fields.remove(column.name).ok_or_else(|| {
                    malformed(format!("missing callback field {}.{}", name, column.name))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !fields.is_empty() {
            return Err(malformed(format!("extra callback fields for {name}")));
        }
        self.rows.entry(spec.key).or_default().push(row);
        Ok(())
    }
    fn domain_fields(
        &self,
        domain: DomainRef,
        actual: &'static str,
    ) -> Result<Vec<(&'static str, Cell)>, MathIrError> {
        let (id, template, name) = match domain {
            DomainRef::Actual(id) => (Cell::Id(id.as_id()), Cell::Null, Cell::Null),
            DomainRef::Template {
                template_id,
                domain_name,
            } => {
                if matches!(self.family, Family::Compiled) {
                    return Err(malformed("unresolved template domain in compiled output"));
                }
                (Cell::Null, Cell::Id(template_id), Cell::Text(domain_name))
            }
        };
        let mut fields = vec![(actual, id)];
        if matches!(self.family, Family::Normalized { .. }) {
            fields.extend([("template_id", template), ("domain_name", name)]);
        }
        Ok(fields)
    }
    fn guard_fields(
        &self,
        guard: Option<GuardRef>,
        filter: bool,
    ) -> Result<Vec<(&'static str, Cell)>, MathIrError> {
        let (actual, source, predicate) = if filter {
            ("filter_node_id", "filter_source_id", "filter_predicate_id")
        } else {
            ("guard_node_id", "guard_source_id", "guard_predicate_id")
        };
        let (node, source_id, predicate_id) = match guard {
            None => (Cell::Null, Cell::Null, Cell::Null),
            Some(GuardRef::Math(node)) => (self.node(node)?, Cell::Null, Cell::Null),
            Some(GuardRef::Predicate {
                source_id,
                predicate_id,
            }) => {
                if matches!(self.family, Family::Compiled) {
                    return Err(malformed("unresolved predicate in compiled output"));
                }
                (Cell::Null, Cell::Id(source_id), Cell::U64(predicate_id))
            }
        };
        let mut fields = vec![(actual, node)];
        if matches!(self.family, Family::Normalized { .. }) {
            fields.extend([(source, source_id), (predicate, predicate_id)]);
        }
        Ok(fields)
    }
}
fn opt_id<T: Into<SemanticId>>(value: Option<T>) -> Cell {
    value.map_or(Cell::Null, |value| Cell::Id(value.into()))
}
fn text(value: impl Into<String>) -> Cell {
    Cell::Text(value.into())
}

fn validate_smoothing(fields: &BTreeMap<&str, Cell>, normalized: bool) -> Result<(), MathIrError> {
    if !matches!(fields.get("eps"), Some(Cell::F64(value)) if value.is_finite() && *value > 0.0) {
        return Err(malformed("smoothing epsilon must be positive and finite"));
    }
    let state = (fields.get("epsilon_state"), fields.get("eps_unit_id"));
    if normalized {
        if !matches!(
            state,
            (Some(Cell::Enum("coordinate")), Some(Cell::Null))
                | (Some(Cell::Enum("pending_unit")), Some(Cell::Id(_)))
        ) {
            return Err(malformed(
                "normalized smoothing state and unit are not one complete alternative",
            ));
        }
    } else if state != (None, None) {
        return Err(malformed("compiled smoothing tolerance must be resolved"));
    }
    Ok(())
}
impl MathRelationSink for RelationSink<'_> {
    fn expr_node(
        &mut self,
        node: NodeId,
        opcode: Opcode,
        quantity: Option<QuantityTypeId>,
        scope: Option<SemanticId>,
        hash: ContentHash,
    ) -> Result<(), MathIrError> {
        let mut fields = vec![
            ("node_id", self.node(node)?),
            ("opcode", Cell::Enum(opcode.as_str())),
            ("quantity_type_id", opt_id(quantity)),
            ("scope_instance_id", opt_id(scope)),
            ("subtree_hash", Cell::Hash(hash)),
        ];
        if let Family::Normalized {
            derivation,
            source_span,
            ..
        } = self.family
        {
            fields.extend([
                ("derivation_id", Cell::Id(derivation)),
                (
                    "source_span",
                    Cell::Struct(vec![
                        Cell::Id(source_span.document_id),
                        Cell::U64(u64::from(source_span.start)),
                        Cell::U64(u64::from(source_span.end)),
                    ]),
                ),
            ]);
        }
        self.write("math_expr_nodes", fields)
    }
    fn expr_arg(&mut self, parent: NodeId, ordinal: u16, child: NodeId) -> Result<(), MathIrError> {
        self.write(
            "math_expr_args",
            vec![
                ("parent_node_id", self.node(parent)?),
                ("argument_ordinal", Cell::U64(u64::from(ordinal))),
                ("child_node_id", self.node(child)?),
            ],
        )
    }
    fn symbol_ref(&mut self, node: NodeId, symbol: ValueRef) -> Result<(), MathIrError> {
        let mut fields = vec![("node_id", self.node(node)?)];
        if matches!(self.family, Family::Compiled) {
            let ValueRef::ActualSymbol(symbol) = symbol else {
                return Err(malformed(
                    "unresolved declaration reference in compiled output",
                ));
            };
            fields.push(("symbol_id", Cell::Id(symbol)));
        } else {
            let (kind, symbol, template, name, domain, index) = match symbol {
                ValueRef::ActualSymbol(id) => (
                    "symbol",
                    Cell::Id(id),
                    Cell::Null,
                    Cell::Null,
                    Cell::Null,
                    Cell::Null,
                ),
                ValueRef::Template {
                    template_id,
                    kind,
                    name,
                } => (
                    kind.as_str(),
                    Cell::Null,
                    Cell::Id(template_id),
                    text(name),
                    Cell::Null,
                    Cell::Null,
                ),
                ValueRef::Domain(DomainRef::Actual(id)) => (
                    "domain",
                    Cell::Null,
                    Cell::Null,
                    Cell::Null,
                    Cell::Id(id.as_id()),
                    Cell::Null,
                ),
                ValueRef::Domain(DomainRef::Template {
                    template_id,
                    domain_name,
                }) => (
                    "domain",
                    Cell::Null,
                    Cell::Id(template_id),
                    text(domain_name),
                    Cell::Null,
                    Cell::Null,
                ),
                ValueRef::Index(id) => (
                    "index",
                    Cell::Null,
                    Cell::Null,
                    Cell::Null,
                    Cell::Null,
                    Cell::Id(id.as_id()),
                ),
            };
            fields.extend([
                ("symbol_id", symbol),
                ("kind", Cell::Enum(kind)),
                ("template_id", template),
                ("name", name),
                ("domain_id", domain),
                ("bound_index_id", index),
            ]);
        }
        self.write("math_symbol_refs", fields)
    }
    fn float_constant(
        &mut self,
        node: NodeId,
        value: f64,
        unit: UnitId,
    ) -> Result<(), MathIrError> {
        self.write(
            "math_float_constants",
            vec![
                ("node_id", self.node(node)?),
                ("value", Cell::F64(value)),
                ("unit_id", Cell::Id(unit.as_id())),
            ],
        )
    }
    fn int_constant(&mut self, node: NodeId, value: i64) -> Result<(), MathIrError> {
        self.write(
            "math_int_constants",
            vec![("node_id", self.node(node)?), ("value", Cell::I64(value))],
        )
    }
    fn affine(
        &mut self,
        node: NodeId,
        constant: f64,
        quantity: Option<QuantityTypeId>,
        unit: Option<UnitId>,
        terms: &[(f64, NodeId)],
    ) -> Result<(), MathIrError> {
        let terms = terms
            .iter()
            .map(|(coefficient, child)| {
                Ok(Cell::Struct(vec![
                    Cell::F64(*coefficient),
                    self.node(*child)?,
                ]))
            })
            .collect::<Result<Vec<_>, MathIrError>>()?;
        self.write(
            "math_affine",
            vec![
                ("node_id", self.node(node)?),
                ("constant", Cell::F64(constant)),
                ("constant_quantity_type_id", opt_id(quantity)),
                ("constant_unit_id", opt_id(unit)),
                ("terms", Cell::List(terms)),
            ],
        )
    }
    fn weighted_mean(
        &mut self,
        node: NodeId,
        pairs: &[(NodeId, NodeId)],
        normalization: WeightNormalization,
        certificate: Option<InvariantId>,
    ) -> Result<(), MathIrError> {
        let pairs = pairs
            .iter()
            .map(|(weight, value)| Ok(Cell::Struct(vec![self.node(*weight)?, self.node(*value)?])))
            .collect::<Result<Vec<_>, MathIrError>>()?;
        self.write(
            "math_weighted_means",
            vec![
                ("node_id", self.node(node)?),
                ("pairs", Cell::List(pairs)),
                ("normalization", Cell::Enum(normalization.as_str())),
                ("unit_sum_invariant_id", opt_id(certificate)),
            ],
        )
    }
    fn reduction(
        &mut self,
        node: NodeId,
        kind: ReductionKind,
        domain: DomainRef,
        bound_index: BoundIndexId,
        filter: Option<GuardRef>,
    ) -> Result<(), MathIrError> {
        let mut fields = vec![
            ("node_id", self.node(node)?),
            ("kind", Cell::Enum(kind.as_str())),
            ("bound_index_id", Cell::Id(bound_index.as_id())),
        ];
        fields.extend(self.domain_fields(domain, "domain_id")?);
        fields.extend(self.guard_fields(filter, true)?);
        self.write("math_reductions", fields)
    }
    fn gather(
        &mut self,
        node: NodeId,
        group: SemanticId,
        coordinates: &[(BoundIndexId, u16)],
    ) -> Result<(), MathIrError> {
        let mut fields = vec![
            ("node_id", self.node(node)?),
            ("group_id", Cell::Id(group)),
            (
                "coordinate_map",
                Cell::List(
                    coordinates
                        .iter()
                        .map(|(index, position)| {
                            Cell::Struct(vec![
                                Cell::Id(index.as_id()),
                                Cell::U64(u64::from(*position)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ];
        if matches!(self.family, Family::Normalized { .. }) {
            fields.extend([
                ("gather_state", Cell::Enum("resolved")),
                ("index_nodes", Cell::Null),
            ]);
        }
        self.write("math_gathers", fields)
    }
    fn pending_gather(
        &mut self,
        node: NodeId,
        group: SemanticId,
        indices: &[NodeId],
    ) -> Result<(), MathIrError> {
        if matches!(self.family, Family::Compiled) {
            return Err(malformed("pending Gather cannot enter compiled relations"));
        }
        self.write(
            "math_gathers",
            vec![
                ("node_id", self.node(node)?),
                ("group_id", Cell::Id(group)),
                ("coordinate_map", Cell::Null),
                ("gather_state", Cell::Enum("pending")),
                (
                    "index_nodes",
                    Cell::List(
                        indices
                            .iter()
                            .map(|node| self.node(*node))
                            .collect::<Result<_, _>>()?,
                    ),
                ),
            ],
        )
    }

    fn broadcast(
        &mut self,
        node: NodeId,
        domain: DomainRef,
        bound_index: BoundIndexId,
    ) -> Result<(), MathIrError> {
        let mut fields = vec![
            ("node_id", self.node(node)?),
            ("bound_index_id", Cell::Id(bound_index.as_id())),
        ];
        fields.extend(self.domain_fields(domain, "domain_id")?);
        self.write("math_broadcasts", fields)
    }
    fn derivative(
        &mut self,
        node: NodeId,
        domain: DomainRef,
        order: u8,
    ) -> Result<(), MathIrError> {
        let mut fields = vec![
            ("node_id", self.node(node)?),
            ("order", Cell::U64(u64::from(order))),
        ];
        fields.extend(self.domain_fields(domain, "wrt_domain_id")?);
        self.write("math_derivatives", fields)
    }
    fn integral(
        &mut self,
        node: NodeId,
        domain: DomainRef,
        bound_index: BoundIndexId,
        policy: Option<SemanticId>,
        filter: Option<GuardRef>,
    ) -> Result<(), MathIrError> {
        let mut fields = vec![
            ("node_id", self.node(node)?),
            ("bound_index_id", Cell::Id(bound_index.as_id())),
            ("quadrature_policy_id", opt_id(policy)),
        ];
        fields.extend(self.domain_fields(domain, "domain_id")?);
        fields.extend(self.guard_fields(filter, true)?);
        self.write("math_integrals", fields)
    }
    fn smooth_op(&mut self, node: NodeId, eps: f64) -> Result<(), MathIrError> {
        let mut fields = vec![("node_id", self.node(node)?), ("eps", Cell::F64(eps))];
        if matches!(self.family, Family::Normalized { .. }) {
            fields.extend([
                ("epsilon_state", Cell::Enum("coordinate")),
                ("eps_unit_id", Cell::Null),
            ]);
        }
        self.write("math_smooth_ops", fields)
    }
    fn pending_smooth_op(
        &mut self,
        node: NodeId,
        eps: f64,
        unit: UnitId,
    ) -> Result<(), MathIrError> {
        if matches!(self.family, Family::Compiled) {
            return Err(malformed(
                "unresolved smoothing tolerance in compiled output",
            ));
        }
        self.write(
            "math_smooth_ops",
            vec![
                ("node_id", self.node(node)?),
                ("eps", Cell::F64(eps)),
                ("epsilon_state", Cell::Enum("pending_unit")),
                ("eps_unit_id", Cell::Id(unit.as_id())),
            ],
        )
    }
    fn conditional(&mut self, node: NodeId, guard: GuardRef) -> Result<(), MathIrError> {
        let mut fields = vec![("node_id", self.node(node)?)];
        fields.extend(self.guard_fields(Some(guard), false)?);
        self.write("math_conditionals", fields)
    }
    fn kernel_call(
        &mut self,
        node: NodeId,
        binding: SemanticId,
        output: u16,
    ) -> Result<(), MathIrError> {
        self.write(
            "math_kernel_calls",
            vec![
                ("node_id", self.node(node)?),
                ("kernel_binding_id", Cell::Id(binding)),
                ("output_ordinal", Cell::U64(u64::from(output))),
            ],
        )
    }
    fn implicit_ref(
        &mut self,
        node: NodeId,
        system: SemanticId,
        unknown: u16,
    ) -> Result<(), MathIrError> {
        self.write(
            "math_implicit_refs",
            vec![
                ("node_id", self.node(node)?),
                ("implicit_system_id", Cell::Id(system)),
                ("unknown_ordinal", Cell::U64(u64::from(unknown))),
            ],
        )
    }
    fn unit_convert(
        &mut self,
        node: NodeId,
        scale: f64,
        offset: f64,
        from: UnitId,
        to: UnitId,
    ) -> Result<(), MathIrError> {
        let mut fields = vec![
            ("node_id", self.node(node)?),
            ("scale", Cell::F64(scale)),
            ("offset", Cell::F64(offset)),
            ("from_unit_id", Cell::Id(from.as_id())),
            ("to_unit_id", Cell::Id(to.as_id())),
        ];
        if matches!(self.family, Family::Normalized { .. }) {
            fields.push(("conversion_state", Cell::Enum("resolved")));
        }
        self.write("math_unit_converts", fields)
    }
    fn pending_unit_convert(&mut self, node: NodeId, to: UnitId) -> Result<(), MathIrError> {
        if !matches!(self.family, Family::Normalized { .. }) {
            return Err(malformed(
                "compiled conversion requires admitted complete coefficients",
            ));
        }
        self.write(
            "math_unit_converts",
            vec![
                ("node_id", self.node(node)?),
                ("scale", Cell::Null),
                ("offset", Cell::Null),
                ("from_unit_id", Cell::Null),
                ("to_unit_id", Cell::Id(to.as_id())),
                ("conversion_state", Cell::Enum("pending")),
            ],
        )
    }
    fn piecewise_linear(
        &mut self,
        node: NodeId,
        points: &[(f64, f64)],
        input: QuantityTypeId,
        output: QuantityTypeId,
    ) -> Result<(), MathIrError> {
        self.write(
            "math_piecewise_linear",
            vec![
                ("node_id", self.node(node)?),
                (
                    "breakpoints",
                    Cell::List(
                        points
                            .iter()
                            .map(|(x, y)| Cell::Struct(vec![Cell::F64(*x), Cell::F64(*y)]))
                            .collect(),
                    ),
                ),
                ("input_quantity_type_id", Cell::Id(input.as_id())),
                ("output_quantity_type_id", Cell::Id(output.as_id())),
            ],
        )
    }
    fn indexed_equation(
        &mut self,
        id: SemanticId,
        owner: SemanticId,
        declaration: Option<SemanticId>,
        name: &str,
        product: Option<SemanticId>,
        filter: Option<NodeId>,
        body: NodeId,
        sense: pse_mathir::equation::Sense,
        lower: Option<NodeId>,
        upper: Option<NodeId>,
        residual: Option<QuantityTypeId>,
        law: Option<SemanticId>,
        derivation: SemanticId,
    ) -> Result<(), MathIrError> {
        self.write(
            "math_indexed_equations",
            vec![
                ("indexed_equation_id", Cell::Id(id)),
                ("owner_instance_id", Cell::Id(owner)),
                ("equation_decl_id", opt_id(declaration)),
                ("qualified_name", text(name)),
                ("product_id", opt_id(product)),
                ("filter_node_id", self.optional_node(filter)?),
                ("body_node_id", self.node(body)?),
                ("sense", Cell::Enum(sense.as_str())),
                ("lower_node_id", self.optional_node(lower)?),
                ("upper_node_id", self.optional_node(upper)?),
                ("residual_quantity_type_id", opt_id(residual)),
                ("law_instance_id", opt_id(law)),
                ("derivation_id", Cell::Id(derivation)),
            ],
        )
    }
    fn free_index(
        &mut self,
        equation: SemanticId,
        binder: BoundIndexId,
        domain: DomainId,
        position: u16,
    ) -> Result<(), MathIrError> {
        self.write(
            "math_free_indices",
            vec![
                ("indexed_equation_id", Cell::Id(equation)),
                ("bound_index_id", Cell::Id(binder.as_id())),
                ("domain_id", Cell::Id(domain.as_id())),
                ("position", Cell::U64(u64::from(position))),
            ],
        )
    }
    fn quantity_selection(
        &mut self,
        node: NodeId,
        operation: Option<OperationId>,
        builtin: Option<BuiltInRule>,
        permutation: &[u16],
        conversions: &[(u16, ConversionId)],
        deferred_static_check: bool,
    ) -> Result<(), MathIrError> {
        self.write(
            "math_quantity_selections",
            vec![
                ("node_id", self.node(node)?),
                ("operation_id", opt_id(operation)),
                (
                    "builtin_rule",
                    builtin.map_or(Cell::Null, |value| Cell::Enum(value.as_str())),
                ),
                (
                    "operand_permutation",
                    Cell::List(
                        permutation
                            .iter()
                            .copied()
                            .map(|value| Cell::U64(u64::from(value)))
                            .collect(),
                    ),
                ),
                (
                    "conversions",
                    Cell::List(
                        conversions
                            .iter()
                            .map(|(operand, id)| {
                                Cell::Struct(vec![
                                    Cell::U64(u64::from(*operand)),
                                    Cell::Id(id.as_id()),
                                ])
                            })
                            .collect(),
                    ),
                ),
                ("deferred_static_check", Cell::Bool(deferred_static_check)),
            ],
        )
    }
    fn kernel_binding(
        &mut self,
        binding: SemanticId,
        kernel: SemanticId,
        scope: SemanticId,
        parameters: &[ParameterBinding],
        inputs: &[InputBinding],
    ) -> Result<(), MathIrError> {
        self.write(
            "kernel_bindings",
            vec![
                ("binding_id", Cell::Id(binding)),
                ("kernel_id", Cell::Id(kernel)),
                ("scope_instance_id", Cell::Id(scope)),
                (
                    "parameter_bindings",
                    Cell::List(
                        parameters
                            .iter()
                            .map(|(name, symbol, value, unit)| {
                                Cell::Struct(vec![
                                    text(name),
                                    opt_id(*symbol),
                                    value.map_or(Cell::Null, Cell::F64),
                                    opt_id(*unit),
                                ])
                            })
                            .collect(),
                    ),
                ),
                (
                    "input_bindings",
                    Cell::List(
                        inputs
                            .iter()
                            .map(|(name, node)| {
                                Ok(Cell::Struct(vec![text(name), self.node(*node)?]))
                            })
                            .collect::<Result<Vec<_>, MathIrError>>()?,
                    ),
                ),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, SemanticId, validate_smoothing};
    use std::collections::BTreeMap;

    #[test]
    fn smoothing_rows_require_complete_state_and_unit_pairs_and_positive_finite_values() {
        let mut fields = BTreeMap::from([
            ("eps", Cell::F64(2.0)),
            ("epsilon_state", Cell::Enum("coordinate")),
            ("eps_unit_id", Cell::Null),
        ]);
        assert!(validate_smoothing(&fields, true).is_ok());
        fields.insert("eps_unit_id", Cell::Id(SemanticId::from_bytes([1; 16])));
        assert!(validate_smoothing(&fields, true).is_err());
        fields.insert("epsilon_state", Cell::Enum("pending_unit"));
        assert!(validate_smoothing(&fields, true).is_ok());
        assert!(validate_smoothing(&fields, false).is_err());
        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            fields.insert("eps", Cell::F64(invalid));
            assert!(validate_smoothing(&fields, true).is_err());
        }
        fields.insert("eps", Cell::F64(2.0));
        fields.insert("eps_unit_id", Cell::Null);
        assert!(validate_smoothing(&fields, true).is_err());
        fields.remove("epsilon_state");
        fields.remove("eps_unit_id");
        assert!(validate_smoothing(&fields, false).is_ok());
        assert!(validate_smoothing(&fields, true).is_err());
    }
}
