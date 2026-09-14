// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-bound lowering preserves predicate/equation structure and lexical scope.

use crate::CompilerError;
use pse_authoring::{
    document::binding::{BoundPath, ParsedExpression, PathMeaning, SourceExpression},
    dsl,
};
use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::{
    BoundIndexId, DomainRef, ExprGraph, GuardRef, NodeId, Opcode, Payload, ReductionKind,
    TemplateValueKind, ValueRef, WeightNormalization, WeightedPair,
};
use pse_schema::model::Cell;
use std::collections::BTreeMap;

pub(super) struct Lowered {
    pub graph: ExprGraph,
    pub root: u64,
    pub syntax: &'static str,
    pub predicates: Vec<Vec<Cell>>,
    pub equations: Vec<Vec<Cell>>,
    pub bindings: Vec<Vec<Cell>>,
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
    context: &pse_authoring::targets::TargetContext,
    cancel: &CancellationToken,
    units: &mut super::units::Units,
    package: SemanticId,
) -> Result<Lowered, CompilerError> {
    let mut lower = Lower {
        source,
        context,
        source_id,
        offset,
        cancel,
        units,
        package,
        paths: source.paths.iter(),
        graph: ExprGraph::new(),
        locals: BTreeMap::new(),
        predicates: Vec::new(),
        equations: Vec::new(),
        bindings: Vec::new(),
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
    })
}
struct Lower<'a, 'b> {
    source: &'a SourceExpression,
    context: &'a pse_authoring::targets::TargetContext,
    source_id: SemanticId,
    offset: u64,
    cancel: &'a CancellationToken,
    units: &'b mut super::units::Units,
    package: SemanticId,
    paths: std::slice::Iter<'a, BoundPath>,
    graph: ExprGraph,
    locals: BTreeMap<String, Local>,
    predicates: Vec<Vec<Cell>>,
    equations: Vec<Vec<Cell>>,
    bindings: Vec<Vec<Cell>>,
}
impl Lower<'_, '_> {
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
        if self.graph.len() >= 65_536 {
            return Err(self.error("source expression node budget exhausted"));
        }
        match &expr.kind {
            dsl::ExprKind::Number(value) => {
                let payload = self.units.literal(self.package, value)?;
                Ok(self.graph.insert(Opcode::Const, payload, &[], None)?)
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
        let meaning = self.next_path(path)?;
        let indices = path
            .segments
            .iter()
            .flat_map(|segment| &segment.indices)
            .collect::<Vec<_>>();
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
        let continuous = match domain {
            DomainRef::Template {
                template_id,
                domain_name,
            } => self
                .context
                .template_domains
                .iter()
                .find(|row| row.template_id == *template_id && row.name == *domain_name)
                .is_some_and(|row| row.continuous),
            DomainRef::Actual(id) => self
                .context
                .domains
                .iter()
                .find(|row| row.domain_id == id.as_id())
                .is_some_and(|row| row.continuous),
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
        let (actual, template, domain_name) = domain_cells(&domain);
        self.bindings.push(vec![
            Cell::Id(self.source_id),
            Cell::Id(id.as_id()),
            Cell::Text(name.to_owned()),
            actual,
            template,
            domain_name,
            position.map_or(Cell::Null, |position| Cell::U64(u64::from(position))),
        ]);
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
        let symbols = self
            .context
            .symbols
            .iter()
            .filter(|row| row.symbol_decl_id == symbol && row.template_id == template)
            .collect::<Vec<_>>();
        let [declaration] = symbols.as_slice() else {
            return Err(self.error("indexed symbol declaration is missing or ambiguous"));
        };
        let axes = declaration.indexed_by.clone();
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
                    .find(|row| row[1] == Cell::Id(index.as_id()))
                    .ok_or_else(|| self.error("index value lacks an actual lexical binding"))?;
                if binding[3] != Cell::Null
                    || binding[4] != Cell::Id(template)
                    || binding[5] != Cell::text(axis)
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
        let mut row = vec![Cell::Null; 18];
        row[0] = Cell::Id(self.source_id);
        match &predicate.kind {
            dsl::PredicateKind::Bool(value) => {
                row[2] = Cell::Enum("boolean");
                row[3] = Cell::Bool(*value);
            }
            dsl::PredicateKind::Null => row[2] = Cell::Enum("null"),
            dsl::PredicateKind::Atom(value) => {
                row[2] = Cell::Enum("atom");
                let node = self.expr(value)?;
                row[5] = Cell::U64(self.global(node)?);
                row[12] = Cell::Enum("expression");
            }
            dsl::PredicateKind::Compare { op, lhs, rhs } => {
                row[2] = Cell::Enum("compare");
                row[4] = Cell::Enum(match op {
                    dsl::CompareOp::Eq => "eq",
                    dsl::CompareOp::NotEq => "not_eq",
                    dsl::CompareOp::Lt => "lt",
                    dsl::CompareOp::Le => "le",
                    dsl::CompareOp::Gt => "gt",
                    dsl::CompareOp::Ge => "ge",
                });
                self.predicate_operand(lhs, &mut row, true)?;
                self.predicate_operand(rhs, &mut row, false)?;
            }
            dsl::PredicateKind::In { expr, domain } => {
                row[2] = Cell::Enum("in");
                let node = self.expr(expr)?;
                row[5] = Cell::U64(self.global(node)?);
                row[12] = Cell::Enum("expression");
                let (actual, template, name) = domain_cells(&self.domain(domain)?);
                row[9] = actual;
                row[10] = template;
                row[11] = name;
            }
            dsl::PredicateKind::And(left, right) | dsl::PredicateKind::Or(left, right) => {
                row[2] = Cell::Enum(if matches!(predicate.kind, dsl::PredicateKind::And(..)) {
                    "and"
                } else {
                    "or"
                });
                row[7] = Cell::U64(self.predicate(left)?);
                row[8] = Cell::U64(self.predicate(right)?);
            }
            dsl::PredicateKind::Not(value) => {
                row[2] = Cell::Enum("not");
                row[7] = Cell::U64(self.predicate(value)?);
            }
        }
        let id = u64::try_from(self.predicates.len())
            .map_err(|_| self.error("predicate ordinal overflow"))?;
        row[1] = Cell::U64(id);
        self.predicates.push(row);
        Ok(id)
    }
    fn predicate_operand(
        &mut self,
        expr: &dsl::Expr,
        row: &mut [Cell],
        left: bool,
    ) -> Result<(), CompilerError> {
        let (value, kind, identity, member) = if left {
            (5, 12, 13, 14)
        } else {
            (6, 15, 16, 17)
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
            row[kind] = Cell::Enum("enum_literal");
            row[identity] = Cell::Id(enum_id);
            row[member] = Cell::Text(spelling);
        } else {
            let node = self.expr(expr)?;
            row[kind] = Cell::Enum("expression");
            row[value] = Cell::U64(self.global(node)?);
        }
        Ok(())
    }
    fn equation(&mut self, equation: &dsl::Equation) -> Result<u64, CompilerError> {
        self.cancel.checkpoint()?;
        let mut row = vec![
            Cell::Id(self.source_id),
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
        ];
        match &equation.kind {
            dsl::EquationKind::Relation { lhs, sense, rhs } => {
                row[2] = Cell::Enum("relation");
                row[3] = Cell::Enum(match sense {
                    dsl::EquationSense::Eq => "eq",
                    dsl::EquationSense::Le => "le",
                    dsl::EquationSense::Ge => "ge",
                });
                let left = self.expr(lhs)?;
                let right = self.expr(rhs)?;
                row[4] = Cell::U64(self.global(left)?);
                row[5] = Cell::U64(self.global(right)?);
            }
            dsl::EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                row[2] = Cell::Enum("conditional");
                row[6] = Cell::U64(self.predicate(guard)?);
                row[7] = Cell::U64(self.equation(then)?);
                row[8] = Cell::U64(self.equation(otherwise)?);
            }
        }
        let id = u64::try_from(self.equations.len())
            .map_err(|_| self.error("equation ordinal overflow"))?;
        row[1] = Cell::U64(id);
        self.equations.push(row);
        Ok(id)
    }
}
fn domain_cells(domain: &DomainRef) -> (Cell, Cell, Cell) {
    match domain {
        DomainRef::Actual(id) => (Cell::Id(id.as_id()), Cell::Null, Cell::Null),
        DomainRef::Template {
            template_id,
            domain_name,
        } => (
            Cell::Null,
            Cell::Id(*template_id),
            Cell::Text(domain_name.clone()),
        ),
    }
}
