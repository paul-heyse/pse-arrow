// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-bound lowering preserves predicate/equation structure and lexical scope.

use crate::CompilerError;
use pse_authoring::{
    document::binding::{BoundPath, ParsedExpression, PathMeaning, SourceExpression},
    dsl,
};
use pse_ids::{CancellationToken, Reservation, SemanticId};
use pse_mathir::{
    BoundIndexId, DomainRef, ExprGraph, GuardRef, NodeId, Opcode, Payload, ReductionKind,
    TemplateValueKind, ValueRef, WeightNormalization, WeightedPair,
};
use pse_relations::generated::{
    enums::{EquationSyntax, PredicateComparison, PredicateKind, PredicateOperandKind, Sense},
    normalized,
};
use std::collections::BTreeMap;

pub(super) struct Lowered {
    pub graph: ExprGraph,
    pub root: u64,
    pub syntax: &'static str,
    pub predicates: Vec<normalized::predicate_nodes::Row>,
    pub equations: Vec<normalized::equation_nodes::Row>,
    pub bindings: Vec<normalized::expression_index_bindings::Row>,
    pub instance_paths: Vec<normalized::expression_paths::Row>,
    pub literal_units: BTreeMap<u64, Vec<SemanticId>>,
}
#[derive(Clone)]
enum Local {
    Value(NodeId),
    Index(BoundIndexId, DomainRef),
}

pub(super) fn lower(
    source: &SourceExpression,
    source_id: SemanticId,
    offset: u64,
    context: &pse_authoring::document::Batches,
    cancel: &CancellationToken,
    units: &mut super::units::Units<'_>,
    package: SemanticId,
    work: &mut dyn Reservation,
) -> Result<Lowered, CompilerError> {
    work.try_grow(
        source
            .indexed_by
            .len()
            .checked_mul(4096)
            .ok_or_else(|| super::invalid("source index allocation overflow"))?,
    )
    .map_err(pse_ids::CanonError::from)?;
    let mut lower = Lower {
        source,
        context,
        source_id,
        offset,
        cancel,
        units,
        package,
        work,
        paths: source.paths.iter(),
        graph: ExprGraph::new(),
        locals: BTreeMap::new(),
        predicates: Vec::new(),
        equations: Vec::new(),
        bindings: Vec::new(),
        instance_paths: Vec::new(),
        literal_units: BTreeMap::new(),
    };
    for (position, name) in source.indexed_by.iter().enumerate() {
        let domain = DomainRef::Template {
            template_id: source.owner_template_id,
            domain_name: name.clone(),
        };
        let position =
            u16::try_from(position).map_err(|_| lower.error("too many index dimensions"))?;
        lower.bind_index(name, domain, Some(position));
    }
    let (root, syntax) = match &source.parsed {
        ParsedExpression::Expr(expr) => {
            let node = lower.expr(expr)?;
            (lower.global(node)?, "expression")
        }
        ParsedExpression::Predicate(predicate) => (lower.predicate(predicate)?, "predicate"),
        ParsedExpression::Equation(equation) => (lower.equation(equation)?, "equation"),
    };
    if lower.paths.next().is_some() {
        return Err(lower.error("unused source binding after lowering"));
    }
    Ok(Lowered {
        graph: lower.graph,
        root,
        syntax,
        predicates: lower.predicates,
        equations: lower.equations,
        bindings: lower.bindings,
        instance_paths: lower.instance_paths,
        literal_units: lower.literal_units,
    })
}
struct Lower<'a, 'b, 'c> {
    source: &'a SourceExpression,
    context: &'a pse_authoring::document::Batches,
    source_id: SemanticId,
    offset: u64,
    cancel: &'a CancellationToken,
    units: &'b mut super::units::Units<'c>,
    work: &'b mut dyn Reservation,
    package: SemanticId,
    paths: std::slice::Iter<'a, BoundPath>,
    graph: ExprGraph,
    locals: BTreeMap<String, Local>,
    predicates: Vec<normalized::predicate_nodes::Row>,
    equations: Vec<normalized::equation_nodes::Row>,
    bindings: Vec<normalized::expression_index_bindings::Row>,
    instance_paths: Vec<normalized::expression_paths::Row>,
    literal_units: BTreeMap<u64, Vec<SemanticId>>,
}
impl Lower<'_, '_, '_> {
    fn error(&self, reason: &str) -> CompilerError {
        pse_authoring::AuthoringError::Contract {
            at: Some(self.source.source_span),
            reason: reason.to_owned(),
        }
        .into()
    }
    fn global(&self, node: NodeId) -> Result<u64, CompilerError> {
        node.0
            .checked_add(self.offset)
            .ok_or_else(|| self.error("normalized node ordinal overflow"))
    }
    fn expr(&mut self, expr: &dsl::Expr) -> Result<NodeId, CompilerError> {
        self.cancel.checkpoint()?;
        self.work
            .try_grow(8192)
            .map_err(pse_ids::CanonError::from)?;
        match &expr.kind {
            dsl::ExprKind::Number(value) => {
                self.units.source_uses.clear();
                let payload = self.units.literal(self.package, value)?;
                let node = self.graph.insert(Opcode::Const, payload, &[], None)?;
                let units = std::mem::take(&mut self.units.source_uses)
                    .into_iter()
                    .map(pse_quantity::UnitId::as_id)
                    .collect::<Vec<_>>();
                let global = self.global(node)?;
                self.literal_units.entry(global).or_default().extend(units);
                Ok(node)
            }
            dsl::ExprKind::Path(path) => self.path(path),
            dsl::ExprKind::Neg(value) => {
                let value = self.expr(value)?;
                Ok(self.graph.neg(value)?)
            }
            dsl::ExprKind::Binary { op, lhs, rhs } => {
                let lhs = self.expr(lhs)?;
                let rhs = self.expr(rhs)?;
                let opcode = match op {
                    dsl::BinaryOp::Add => Opcode::Add,
                    dsl::BinaryOp::Sub => Opcode::Sub,
                    dsl::BinaryOp::Mul => Opcode::Mul,
                    dsl::BinaryOp::Div => Opcode::Div,
                    dsl::BinaryOp::Pow => Opcode::Pow,
                };
                Ok(self
                    .graph
                    .insert(opcode, Payload::None, &[lhs, rhs], None)?)
            }
            dsl::ExprKind::Call {
                function,
                args,
                named,
            } => self.call(*function, args, named),
            dsl::ExprKind::Kernel { .. } => {
                Err(self.error("kernel lowering requires the later kernel binding pass"))
            }
            dsl::ExprKind::Reduce { kind, binder, body } => self.reduce(*kind, binder, body),
            dsl::ExprKind::Derivative { body, wrt } => {
                let body = self.expr(body)?;
                let domain = self.domain(wrt)?;
                self.continuous(&domain)?;
                Ok(self.graph.insert(
                    Opcode::Derivative,
                    Payload::Derivative {
                        wrt_domain: domain,
                        order: 1,
                    },
                    &[body],
                    None,
                )?)
            }
            dsl::ExprKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                let predicate_id = self.predicate(guard)?;
                let then = self.expr(then)?;
                let otherwise = self.expr(otherwise)?;
                Ok(self.graph.insert(
                    Opcode::Conditional,
                    Payload::Conditional {
                        guard: GuardRef::Predicate {
                            source_id: self.source_id,
                            predicate_id,
                        },
                    },
                    &[then, otherwise],
                    None,
                )?)
            }
            dsl::ExprKind::Let { bindings, body } => {
                let prior = self.locals.clone();
                for (name, value) in bindings {
                    let value = self.expr(value)?;
                    self.locals.insert(name.clone(), Local::Value(value));
                }
                let body = self.expr(body)?;
                self.locals = prior;
                Ok(body)
            }
        }
    }
    fn next_path(&mut self, path: &dsl::Path) -> Result<PathMeaning, CompilerError> {
        let binding = self
            .paths
            .next()
            .ok_or_else(|| self.error("missing exact path binding"))?;
        if binding.segment_entities.len() != path.segments.len() {
            return Err(self.error("path binding shape differs"));
        }
        Ok(binding.meaning.clone())
    }
    fn path(&mut self, path: &dsl::Path) -> Result<NodeId, CompilerError> {
        let span = self
            .paths
            .as_slice()
            .first()
            .ok_or_else(|| self.error("path source span missing"))?
            .span;
        let meaning = self.next_path(path)?;
        let indices = path
            .segments
            .iter()
            .flat_map(|segment| &segment.indices)
            .collect::<Vec<_>>();
        if let PathMeaning::InstancePath(binding) = meaning {
            let path_id = u64::try_from(self.instance_paths.len())
                .map_err(|_| self.error("path count overflow"))?;
            // Insert the obligation before descending so nested index paths get distinct ordinals.
            self.instance_paths.push(normalized::expression_paths::Row {
                source_id: self.source_id, path_id, root_instance_id: binding.root_instance_id,
                segments: binding.segments.into_iter().map(|segment| Ok(normalized::expression_paths::NormalizedExpressionPathsFieldSegmentsItem {
                    kind: segment.kind.as_str().parse()?, name: segment.name, index_count: segment.index_count,
                })).collect::<Result<_, pse_relations::RelationError>>()?,
                path_start: span.start, path_end: span.end,
                derivation_id: pse_ids::named_id(self.source_id, &format!("pass:P3:path:{path_id}")),
            });
            let indices = indices
                .into_iter()
                .map(|index| self.expr(index))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(self.graph.insert(
                Opcode::SymbolRef,
                Payload::PendingPath {
                    source_id: self.source_id,
                    path_id,
                    indices,
                },
                &[],
                None,
            )?);
        }
        if let PathMeaning::Local(name) = &meaning {
            if !indices.is_empty() {
                return Err(self.error("local scalar/index value cannot carry extra subscripts"));
            }
            return match self.locals.get(name).cloned() {
                Some(Local::Value(node)) => Ok(node),
                Some(Local::Index(index, _)) => Ok(self.graph.insert(
                    Opcode::SymbolRef,
                    Payload::SymbolRef {
                        symbol: ValueRef::Index(index),
                    },
                    &[],
                    None,
                )?),
                None => Err(self.error("unresolved lexical value")),
            };
        }
        if !indices.is_empty() {
            let PathMeaning::Symbol {
                symbol_id,
                template_id,
            } = meaning
            else {
                return Err(self.error("indexed read requires its actual symbol declaration"));
            };
            return self.indexed(symbol_id, template_id, &indices);
        }

        let symbol = match meaning {
            PathMeaning::Symbol { symbol_id, .. } => ValueRef::ActualSymbol(symbol_id),
            PathMeaning::Parameter { template_id, name } => ValueRef::Template {
                template_id,
                kind: TemplateValueKind::Parameter,
                name,
            },
            PathMeaning::Feature { template_id, name } => ValueRef::Template {
                template_id,
                kind: TemplateValueKind::Feature,
                name,
            },
            PathMeaning::Port { template_id, name } => ValueRef::Template {
                template_id,
                kind: TemplateValueKind::Port,
                name,
            },
            PathMeaning::Domain { template_id, name } => ValueRef::Domain(DomainRef::Template {
                template_id,
                domain_name: name,
            }),
            _ => return Err(self.error("bound entity is not a declared arithmetic value")),
        };
        Ok(self
            .graph
            .insert(Opcode::SymbolRef, Payload::SymbolRef { symbol }, &[], None)?)
    }
    fn domain(&mut self, path: &dsl::Path) -> Result<DomainRef, CompilerError> {
        if path
            .segments
            .iter()
            .any(|segment| !segment.indices.is_empty())
        {
            return Err(self.error("domain path cannot carry subscripts"));
        }
        match self.next_path(path)? {
            PathMeaning::Domain { template_id, name } => Ok(DomainRef::Template {
                template_id,
                domain_name: name,
            }),
            PathMeaning::Local(name) => {
                match self.locals.get(&name) {
                    Some(Local::Index(_, domain)) => Ok(domain.clone()),
                    _ => Err(self
                        .error("domain reference must name a declared domain or its bound index")),
                }
            }
            _ => Err(self.error("domain reference is not a declared domain")),
        }
    }
    fn continuous(&self, domain: &DomainRef) -> Result<(), CompilerError> {
        use pse_relations::generated::authored;
        let continuous = match domain {
            DomainRef::Template {
                template_id,
                domain_name,
            } => {
                let batch = self
                    .context
                    .get(&authored::template_domains::RELATION_ID)
                    .ok_or_else(|| self.error("template domain input absent"))?;
                let view = authored::template_domains::View::from_checked(batch)?;
                (0..view.len())
                    .find(|row| {
                        view.template_id_column().value(*row) == template_id.as_bytes()
                            && view.name_column().value(*row) == domain_name
                    })
                    .is_some_and(|row| view.continuous_column().value(row))
            }
            DomainRef::Actual(id) => {
                let batch = self
                    .context
                    .get(&authored::domains::RELATION_ID)
                    .ok_or_else(|| self.error("actual domain input absent"))?;
                let view = authored::domains::View::from_checked(batch)?;
                (0..view.len())
                    .find(|row| view.domain_id_column().value(*row) == id.as_id().as_bytes())
                    .is_some_and(|row| view.continuous_column().value(row))
            }
        };
        if !continuous {
            return Err(self
                .error("derivative or integral requires an actually declared continuous domain"));
        }
        Ok(())
    }
    fn bind_index(&mut self, name: &str, domain: DomainRef, position: Option<u16>) -> BoundIndexId {
        let ordinal = self.bindings.len();
        let id = BoundIndexId::from_id(pse_ids::named_id(
            self.source_id,
            &format!("pse:p3:index:v1:{ordinal}"),
        ));
        let (actual, template, domain_name) = domain_fields(&domain);
        self.bindings
            .push(normalized::expression_index_bindings::Row {
                source_id: self.source_id,
                bound_index_id: id.as_id(),
                name: name.to_owned(),
                domain_id: actual,
                template_id: template,
                domain_name,
                position,
            });
        self.locals
            .insert(name.to_owned(), Local::Index(id, domain));
        id
    }
    fn reduce(
        &mut self,
        kind: dsl::ReduceKind,
        binder: &dsl::Binder,
        body: &dsl::Expr,
    ) -> Result<NodeId, CompilerError> {
        let domain = self.domain(&binder.domain)?;
        if kind == dsl::ReduceKind::Integral {
            self.continuous(&domain)?;
        }
        let prior = self.locals.clone();
        let bound_index = self.bind_index(&binder.var, domain.clone(), None);
        let filter = binder
            .filter
            .as_ref()
            .map(|filter| self.predicate(filter))
            .transpose()?
            .map(|predicate_id| GuardRef::Predicate {
                source_id: self.source_id,
                predicate_id,
            });
        let body = self.expr(body)?;
        self.locals = prior;
        let (opcode, payload) = match kind {
            dsl::ReduceKind::Sum => (
                Opcode::SumOver,
                Payload::Reduction {
                    kind: ReductionKind::Sum,
                    domain,
                    bound_index,
                    filter,
                },
            ),
            dsl::ReduceKind::Prod => (
                Opcode::ProdOver,
                Payload::Reduction {
                    kind: ReductionKind::Prod,
                    domain,
                    bound_index,
                    filter,
                },
            ),
            dsl::ReduceKind::Integral => (
                Opcode::Integral,
                Payload::Integral {
                    domain,
                    bound_index,
                    quadrature_policy: None,
                    filter,
                },
            ),
        };
        Ok(self.graph.insert(opcode, payload, &[body], None)?)
    }
    fn indexed(
        &mut self,
        symbol: SemanticId,
        template: SemanticId,
        indices: &[&dsl::Expr],
    ) -> Result<NodeId, CompilerError> {
        use pse_relations::generated::authored;
        let batch = self
            .context
            .get(&authored::template_symbols::RELATION_ID)
            .ok_or_else(|| self.error("indexed symbol input absent"))?;
        let symbols = authored::template_symbols::View::from_checked(batch)?;
        let mut matches = (0..symbols.len()).filter(|row| {
            symbols.symbol_decl_id_column().value(*row) == symbol.as_bytes()
                && symbols.template_id_column().value(*row) == template.as_bytes()
        });
        let declaration = matches
            .next()
            .ok_or_else(|| self.error("indexed symbol declaration is missing"))?;
        if matches.next().is_some() {
            return Err(self.error("indexed symbol declaration is ambiguous"));
        }
        let axes = symbols.row(declaration)?.indexed_by;
        if axes.len() != indices.len() {
            return Err(self.error("indexed read arity differs from declared axes"));
        }
        let coordinates = indices
            .iter()
            .enumerate()
            .map(|(position, index)| {
                let dsl::ExprKind::Path(path) = &index.kind else {
                    return Ok(None);
                };
                let [part] = path.segments.as_slice() else {
                    return Ok(None);
                };
                if !part.indices.is_empty() {
                    return Ok(None);
                }
                let Some(Local::Index(binding, domain)) = self.locals.get(&part.name) else {
                    return Ok(None);
                };
                if domain
                    != &(DomainRef::Template {
                        template_id: template,
                        domain_name: axes[position].clone(),
                    })
                {
                    return Err(
                        self.error("indexed read coordinate is bound to a different declared axis")
                    );
                }
                let position = u16::try_from(position)
                    .map_err(|_| self.error("coordinate ordinal overflow"))?;
                Ok(Some((*binding, position)))
            })
            .collect::<Result<Vec<_>, CompilerError>>()?;
        let payload = if coordinates.iter().all(Option::is_some) {
            for index in indices {
                let dsl::ExprKind::Path(path) = &index.kind else {
                    return Err(self.error("coordinate changed during lowering"));
                };
                if !matches!(self.next_path(path)?, PathMeaning::Local(_)) {
                    return Err(self.error("coordinate binding differs from lexical declaration"));
                }
            }
            Payload::Gather {
                group: symbol,
                coordinate_map: coordinates.into_iter().flatten().collect(),
            }
        } else {
            let mut values = Vec::new();
            for (axis, index) in axes.iter().zip(indices) {
                let node = self.expr(index)?;
                self.check_index_axis(node, template, axis)?;
                values.push(node);
            }
            Payload::PendingGather {
                group: symbol,
                indices: values,
            }
        };
        Ok(self.graph.insert(Opcode::Gather, payload, &[], None)?)
    }
    fn check_index_axis(
        &self,
        root: NodeId,
        template: SemanticId,
        axis: &str,
    ) -> Result<(), CompilerError> {
        let mut pending = vec![root];
        let mut visited = std::collections::BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !visited.insert(id) {
                continue;
            }
            let node = self.graph.node(id)?;
            if let Payload::SymbolRef {
                symbol: ValueRef::Index(index),
            } = &node.payload
            {
                let binding = self
                    .bindings
                    .iter()
                    .find(|row| row.bound_index_id == index.as_id())
                    .ok_or_else(|| self.error("index value lacks an actual lexical binding"))?;
                if binding.domain_id.is_some()
                    || binding.template_id != Some(template)
                    || binding.domain_name.as_deref() != Some(axis)
                {
                    return Err(self.error("index expression uses a different declared axis"));
                }
            }
            pending.extend(node.payload.referenced_nodes());
            pending.extend(&node.children);
        }
        Ok(())
    }
    fn call(
        &mut self,
        function: dsl::Function,
        args: &[dsl::Expr],
        named: &[dsl::NamedArg],
    ) -> Result<NodeId, CompilerError> {
        use dsl::Function as F;
        if function == F::Broadcast {
            return self.broadcast(args);
        }
        if function == F::Convert {
            let child = self.expr(&args[0])?;
            self.unit_bindings(&args[1])?;
            let span = args[1].span;
            let spelling = self
                .source
                .text
                .get(span.start as usize..span.end as usize)
                .ok_or_else(|| self.error("conversion unit source range is invalid"))?;
            let requested = self.units.requested(self.package, spelling)?;
            return Ok(self.graph.insert(
                Opcode::UnitConvert,
                Payload::PendingUnitConvert { to: requested.id },
                &[child],
                None,
            )?);
        }
        if matches!(function, F::Min | F::Max) {
            return Err(self.error("function requires an admitted lowering contract"));
        }
        let children = args
            .iter()
            .map(|arg| self.expr(arg))
            .collect::<Result<Vec<_>, _>>()?;
        if function == F::WeightedMean {
            let pairs = children
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| WeightedPair {
                    weight: pair[0],
                    value: pair[1],
                })
                .collect();
            return Ok(self.graph.insert(
                Opcode::WeightedMean,
                Payload::WeightedMean {
                    pairs,
                    normalization: WeightNormalization::DivideBySum,
                    unit_sum_invariant: None,
                },
                &[],
                None,
            )?);
        }
        let opcode = match function {
            F::Exp => Opcode::Exp,
            F::Log => Opcode::Log,
            F::Log10 => Opcode::Log10,
            F::Sqrt => Opcode::Sqrt,
            F::Abs => Opcode::Abs,
            F::SmoothMax => Opcode::SmoothMax,
            F::SmoothMin => Opcode::SmoothMin,
            F::SmoothAbs => Opcode::SmoothAbs,
            F::SafeSqrt => Opcode::SafeSqrt,
            F::SafeLog => Opcode::SafeLog,
            F::Sin => Opcode::Sin,
            F::Cos => Opcode::Cos,
            F::Tan => Opcode::Tan,
            F::Tanh => Opcode::Tanh,
            F::Erf => Opcode::Erf,
            _ => return Err(self.error("function has no numerical opcode")),
        };
        let payload = if function.has_epsilon() {
            let [epsilon] = named else {
                return Err(self.error("smooth operation requires explicit epsilon"));
            };
            let dsl::ExprKind::Number(value) = &epsilon.value.kind else {
                return Err(self.error("epsilon must be a declared finite literal"));
            };
            if let Some(spelling) = &value.unit {
                let unit = self.units.requested(self.package, spelling)?;
                Payload::PendingSmoothOp {
                    eps: value.value,
                    unit: unit.id,
                }
            } else {
                Payload::SmoothOp { eps: value.value }
            }
        } else {
            Payload::None
        };
        Ok(self.graph.insert(opcode, payload, &children, None)?)
    }
    fn broadcast(&mut self, args: &[dsl::Expr]) -> Result<NodeId, CompilerError> {
        let [
            value,
            dsl::Expr {
                kind: dsl::ExprKind::Path(path),
                ..
            },
        ] = args
        else {
            return Err(self.error("broadcast requires a value and a lexical index"));
        };
        let child = self.expr(value)?;
        let PathMeaning::Local(name) = self.next_path(path)? else {
            return Err(self.error("broadcast index must name an actual lexical binding"));
        };
        let Some(Local::Index(bound_index, domain)) = self.locals.get(&name).cloned() else {
            return Err(self.error("broadcast index cannot be a scalar lexical value"));
        };
        Ok(self.graph.insert(
            Opcode::Broadcast,
            Payload::Broadcast {
                domain,
                bound_index,
            },
            &[child],
            None,
        )?)
    }
    fn unit_bindings(&mut self, expr: &dsl::Expr) -> Result<(), CompilerError> {
        match &expr.kind {
            dsl::ExprKind::Path(path) => {
                if !matches!(self.next_path(path)?, PathMeaning::Unit { .. }) {
                    return Err(self.error("conversion unit path binding mismatch"));
                }
            }
            dsl::ExprKind::Number(_) => {}
            dsl::ExprKind::Neg(expr) => self.unit_bindings(expr)?,
            dsl::ExprKind::Binary { lhs, rhs, .. } => {
                self.unit_bindings(lhs)?;
                self.unit_bindings(rhs)?;
            }
            _ => return Err(self.error("malformed bound unit expression")),
        }
        Ok(())
    }
    fn predicate(&mut self, predicate: &dsl::Predicate) -> Result<u64, CompilerError> {
        self.cancel.checkpoint()?;
        self.work
            .try_grow(4096)
            .map_err(pse_ids::CanonError::from)?;
        let mut row = predicate_row(self.source_id, PredicateKind::Null);
        match &predicate.kind {
            dsl::PredicateKind::Bool(value) => {
                row.kind = PredicateKind::Boolean;
                row.boolean_value = Some(*value);
            }
            dsl::PredicateKind::Null => {}
            dsl::PredicateKind::Atom(value) => {
                row.kind = PredicateKind::Atom;
                let node = self.expr(value)?;
                row.left_expr = Some(self.global(node)?);
                row.left_kind = Some(PredicateOperandKind::Expression);
            }
            dsl::PredicateKind::Compare { op, lhs, rhs } => {
                row.kind = PredicateKind::Compare;
                row.comparison = Some(match op {
                    dsl::CompareOp::Eq => PredicateComparison::Eq,
                    dsl::CompareOp::NotEq => PredicateComparison::NotEq,
                    dsl::CompareOp::Lt => PredicateComparison::Lt,
                    dsl::CompareOp::Le => PredicateComparison::Le,
                    dsl::CompareOp::Gt => PredicateComparison::Gt,
                    dsl::CompareOp::Ge => PredicateComparison::Ge,
                });
                self.predicate_operand(lhs, &mut row, true)?;
                self.predicate_operand(rhs, &mut row, false)?;
                if (row.left_kind == Some(PredicateOperandKind::EnumLiteral)
                    || row.right_kind == Some(PredicateOperandKind::EnumLiteral))
                    && !matches!(op, dsl::CompareOp::Eq | dsl::CompareOp::NotEq)
                {
                    return Err(self.error("enum ordering is not declared"));
                }
            }
            dsl::PredicateKind::In { expr, domain } => {
                row.kind = PredicateKind::In;
                let node = self.expr(expr)?;
                row.left_expr = Some(self.global(node)?);
                row.left_kind = Some(PredicateOperandKind::Expression);
                (row.domain_id, row.domain_template_id, row.domain_name) =
                    domain_fields(&self.domain(domain)?);
            }
            dsl::PredicateKind::And(left, right) | dsl::PredicateKind::Or(left, right) => {
                row.kind = if matches!(predicate.kind, dsl::PredicateKind::And(..)) {
                    PredicateKind::And
                } else {
                    PredicateKind::Or
                };
                row.left_predicate = Some(self.predicate(left)?);
                row.right_predicate = Some(self.predicate(right)?);
            }
            dsl::PredicateKind::Not(value) => {
                row.kind = PredicateKind::Not;
                row.left_predicate = Some(self.predicate(value)?);
            }
        }
        // Assign parents after children: syntax ordinals establish acyclicity at construction.
        row.predicate_id = u64::try_from(self.predicates.len())
            .map_err(|_| self.error("predicate ordinal overflow"))?;
        let id = row.predicate_id;
        self.predicates.push(row);
        Ok(id)
    }
    fn predicate_operand(
        &mut self,
        expr: &dsl::Expr,
        row: &mut normalized::predicate_nodes::Row,
        left: bool,
    ) -> Result<(), CompilerError> {
        let (value, kind, identity, member) = if left {
            (
                &mut row.left_expr,
                &mut row.left_kind,
                &mut row.left_enum_id,
                &mut row.left_enum_member,
            )
        } else {
            (
                &mut row.right_expr,
                &mut row.right_kind,
                &mut row.right_enum_id,
                &mut row.right_enum_member,
            )
        };
        if let dsl::ExprKind::Path(path) = &expr.kind
            && let Some(binding) = self.paths.as_slice().first()
            && matches!(binding.meaning, PathMeaning::EnumLiteral { .. })
        {
            let PathMeaning::EnumLiteral {
                enum_id,
                member: spelling,
            } = self.next_path(path)?
            else {
                return Err(self.error("enum literal binding changed"));
            };
            // Exact enum/member membership is owned by the actual source binder.
            *kind = Some(PredicateOperandKind::EnumLiteral);
            *identity = Some(enum_id);
            *member = Some(spelling);
        } else {
            let node = self.expr(expr)?;
            *kind = Some(PredicateOperandKind::Expression);
            *value = Some(self.global(node)?);
        }
        Ok(())
    }
    fn equation(&mut self, equation: &dsl::Equation) -> Result<u64, CompilerError> {
        self.cancel.checkpoint()?;
        self.work
            .try_grow(4096)
            .map_err(pse_ids::CanonError::from)?;
        let mut row = normalized::equation_nodes::Row {
            source_id: self.source_id,
            equation_id: 0,
            kind: EquationSyntax::Relation,
            sense: None,
            left_expr: None,
            right_expr: None,
            guard_predicate: None,
            then_equation: None,
            else_equation: None,
        };
        match &equation.kind {
            dsl::EquationKind::Relation { lhs, sense, rhs } => {
                row.sense = Some(match sense {
                    dsl::EquationSense::Eq => Sense::Eq,
                    dsl::EquationSense::Le => Sense::Le,
                    dsl::EquationSense::Ge => Sense::Ge,
                });
                let left = self.expr(lhs)?;
                let right = self.expr(rhs)?;
                row.left_expr = Some(self.global(left)?);
                row.right_expr = Some(self.global(right)?);
            }
            dsl::EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                row.kind = EquationSyntax::Conditional;
                row.guard_predicate = Some(self.predicate(guard)?);
                row.then_equation = Some(self.equation(then)?);
                row.else_equation = Some(self.equation(otherwise)?);
            }
        }
        row.equation_id = u64::try_from(self.equations.len())
            .map_err(|_| self.error("equation ordinal overflow"))?;
        let id = row.equation_id;
        self.equations.push(row);
        Ok(id)
    }
}
fn domain_fields(domain: &DomainRef) -> (Option<SemanticId>, Option<SemanticId>, Option<String>) {
    match domain {
        DomainRef::Actual(id) => (Some(id.as_id()), None, None),
        DomainRef::Template {
            template_id,
            domain_name,
        } => (None, Some(*template_id), Some(domain_name.clone())),
    }
}
pub(super) fn predicate_row(
    source_id: SemanticId,
    kind: PredicateKind,
) -> normalized::predicate_nodes::Row {
    normalized::predicate_nodes::Row {
        source_id,
        predicate_id: 0,
        kind,
        boolean_value: None,
        comparison: None,
        left_expr: None,
        right_expr: None,
        left_predicate: None,
        right_predicate: None,
        domain_id: None,
        domain_template_id: None,
        domain_name: None,
        left_kind: None,
        left_enum_id: None,
        left_enum_member: None,
        right_kind: None,
        right_enum_id: None,
        right_enum_member: None,
    }
}
