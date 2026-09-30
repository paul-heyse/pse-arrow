// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_authoring::dsl::{Predicate, PredicateKind, ReduceKind};
/// The owning instance, the member declaration and the member's index coordinates.
type ResolvedPath = (InstanceId, DeclarationId, Vec<(String, Value)>);
impl Engine<'_, '_> {
    pub(super) fn child_instance(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        segment: &PathSegment,
        env: &Environment,
    ) -> Result<InstanceId> {
        let state = &self.states[&instance];
        let member = *state
            .members
            .get(&segment.name)
            .ok_or_else(|| invalid(at, format!("unknown child {}", segment.name)))?;
        if !matches!(
            self.p.declarations[&member].value.kind,
            Kind::Child | Kind::Implicit
        ) {
            return Err(invalid(at, "child instance required"));
        }
        let values = segment
            .indices
            .iter()
            .map(|index| self.eval(at, env, &dsl::render_expr(index), None))
            .collect::<Result<Vec<_>>>()?;
        let coordinates = self.member_coordinates(instance, member, values)?;
        state
            .children
            .get(&(
                segment.name.clone(),
                coordinates.iter().map(|(_, v)| v.identity()).collect(),
            ))
            .copied()
            .ok_or_else(|| invalid(at, format!("unknown child {}", segment.name)))
    }
    pub(super) fn member_coordinates(
        &self,
        instance: InstanceId,
        member: DeclarationId,
        values: Vec<Value>,
    ) -> Result<Vec<(String, Value)>> {
        let declaration = &self.p.declarations[&member];
        let indices = if let Some(v) = &declaration.value.binding {
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect::<Vec<_>>()
        } else if let Some(v) = &declaration.value.accumulator {
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else if let Some(e) = &declaration.value.equation {
            e.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else {
            Vec::new()
        };
        if indices.len() != values.len() {
            return Err(invalid(member, "member index arity differs"));
        }
        let admitted =
            self.coordinates(member, &self.states[&instance].env, indices.into_iter())?;
        admitted.into_iter().find(|row| row.iter().zip(&values).all(|((_, x), y)| {
            x == y || matches!((x,y),
                (Value::Coordinate{quantity:q,bits:a,..},Value::Number{quantity:r,bits:b}) if q==r && a==b)
        })).ok_or_else(|| invalid(member, "coordinate outside declared membership"))
    }
    pub(super) fn resolve_path(
        &self,
        mut instance: InstanceId,
        at: DeclarationId,
        path: &Path,
        env: &Environment,
        ancestor: bool,
    ) -> Result<ResolvedPath> {
        let (first, rest) = path
            .segments
            .split_first()
            .ok_or_else(|| invalid(instance, "empty member path"))?;
        if first.name == "parent" {
            instance = self.states[&instance]
                .parent
                .ok_or_else(|| invalid(instance, "root has no parent"))?;
            return self.resolve_path(
                instance,
                at,
                &Path {
                    segments: rest.to_vec(),
                },
                env,
                ancestor,
            );
        }
        let state = &self.states[&instance];
        let values = first
            .indices
            .iter()
            .map(|e| {
                Evaluator {
                    package: self.p,
                    physical: self.c,
                    at,
                    env,
                    limit: self.limits.members,
                    stack: Vec::new(),
                    reader: self.reader,
                }
                .expr(e, None, 0)
            })
            .collect::<Result<Vec<_>>>()?;
        if !rest.is_empty() {
            // `route.left` names an alternative's binary indicator (ADR-0104); nested
            // alternatives continue through their disjunctions.
            if let Some(disjunction) = state
                .members
                .get(&first.name)
                .filter(|id| self.p.declarations[*id].value.kind == Kind::Disjunction)
            {
                let mut owner = *disjunction;
                let mut alternative = None;
                for segment in rest {
                    let child = self
                        .p
                        .children
                        .get(&owner)
                        .into_iter()
                        .flatten()
                        .copied()
                        .find(|id| {
                            self.p.declarations[id].name == segment.name
                                && matches!(
                                    self.p.declarations[id].value.kind,
                                    Kind::Alternative | Kind::Disjunction
                                )
                        })
                        .ok_or_else(|| {
                            invalid(at, format!("unknown alternative {}", segment.name))
                        })?;
                    if !segment.indices.is_empty() {
                        return Err(invalid(at, "an alternative is not indexed"));
                    }
                    alternative = (self.p.declarations[&child].value.kind == Kind::Alternative)
                        .then_some(child);
                    owner = child;
                }
                let alternative =
                    alternative.ok_or_else(|| invalid(at, "path must end at an alternative"))?;
                return Ok((instance, alternative, Vec::new()));
            }
            if state.members.contains_key(&first.name) {
                let child = self.child_instance(instance, at, first, env)?;
                return self.resolve_path(
                    child,
                    at,
                    &Path {
                        segments: rest.to_vec(),
                    },
                    env,
                    false,
                );
            }
            if ancestor && let Some(parent) = state.parent {
                return self.resolve_path(parent, at, path, env, true);
            }
            return Err(invalid(
                state.definition,
                format!("unknown child {}", first.name),
            ));
        }
        if let Some(member) = state.members.get(&first.name) {
            let coordinates = self.member_coordinates(instance, *member, values)?;
            return Ok((instance, *member, coordinates));
        }
        if ancestor && let Some(parent) = state.parent {
            return self.resolve_path(parent, at, path, env, true);
        }
        Err(invalid(
            state.definition,
            format!("unknown instantiated member {}", first.name),
        ))
    }
    pub(crate) fn rewrite(
        &mut self,
        instance: InstanceId,
        expression: &Expr,
        local: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Expr> {
        self.reserve(1)?;
        let at = chain
            .last()
            .copied()
            .unwrap_or(self.states[&instance].definition);
        let in_function = self.p.functions.contains_key(&at);
        let mut env = if in_function {
            Environment::new()
        } else {
            self.states[&instance].env.clone()
        };
        env.extend(local.clone());
        let kind = match &expression.kind {
            ExprKind::Number(n) => ExprKind::Number(n.clone()),
            ExprKind::Path(path) => {
                let text = dsl::render_path(path);
                if let Some(value) = self.lexical.get(&text) {
                    return Ok(value.clone());
                }
                if path.segments.len() == 1
                    && let Some(group) = self.indexed_arguments.get(&path.segments[0].name)
                {
                    let coordinates = path.segments[0]
                        .indices
                        .iter()
                        .map(|e| self.eval(at, &env, &dsl::render_expr(e), None))
                        .collect::<Result<Vec<_>>>()?;
                    return group
                        .iter()
                        .find(|(key, _)| *key == coordinates)
                        .map(|(_, e)| e.clone())
                        .ok_or_else(|| {
                            invalid(at, "function coordinate outside actual membership")
                        });
                }
                if let Some(value) = env.get(&text) {
                    return self.value_expression(value, at);
                }
                // A pure function's free names belong to its definition scope. A caller
                // instance cannot capture a table, entity or constant with a same-named member.
                if in_function {
                    let value = self.eval(at, &env, &text, None)?;
                    return self.value_expression(&value, at);
                }
                // Tables and attributes are immutable admitted data, never I/O from this operation.
                let first = path
                    .segments
                    .first()
                    .ok_or_else(|| invalid(at, "empty path"))?;
                if !self.states[&instance].members.contains_key(&first.name)
                    && !self.states[&instance]
                        .children
                        .keys()
                        .any(|(n, _)| n == &first.name)
                    && first.name != "parent"
                {
                    if self.p.declarations[&self.states[&instance].definition]
                        .value
                        .kind
                        == Kind::Implicit
                        && let Ok((owner, member, coordinates)) =
                            self.resolve_path(instance, at, path, &env, true)
                    {
                        let id = self.symbol(owner, member, &coordinates, chain)?;
                        return Ok(symbol_expr(id));
                    }
                    let value = self.eval(at, &env, &text, None)?;
                    return self.value_expression(&value, at);
                }
                let (owner, member, coordinates) =
                    self.resolve_path(instance, at, path, &env, false)?;
                let id = self.symbol(owner, member, &coordinates, chain)?;
                return Ok(symbol_expr(id));
            }
            ExprKind::Neg(e) => ExprKind::Neg(Box::new(self.rewrite(instance, e, &env, chain)?)),
            ExprKind::Binary { op, lhs, rhs } => ExprKind::Binary {
                op: *op,
                lhs: Box::new(self.rewrite(instance, lhs, &env, chain)?),
                rhs: Box::new(self.rewrite(instance, rhs, &env, chain)?),
            },
            ExprKind::Call { function, args } => ExprKind::Call {
                function: *function,
                args: args
                    .iter()
                    .map(|e| self.rewrite(instance, e, &env, chain))
                    .collect::<Result<_>>()?,
            },
            ExprKind::NamedCall { name, args } => {
                let function = self.resolve_function(instance, at, name, &env)?;
                self.function_call(instance, function, args, &[], &env, chain)?
            }
            ExprKind::Partial {
                function,
                wrt,
                args,
            } => {
                let function = self.resolve_function(instance, at, function, &env)?;
                self.function_call(instance, function, args, wrt, &env, chain)?
            }
            ExprKind::Reduce { kind, binder, body } => {
                if *kind == ReduceKind::Integral {
                    return self.integral(instance, binder, body, &env, chain);
                }
                let types = self.source_types(at, &env)?;
                let domain_type = crate::expression::infer(
                    &Expr {
                        kind: ExprKind::Path(binder.domain.clone()),
                        span: Span::default(),
                    },
                    &types,
                    self.p,
                    self.c,
                    at,
                    None,
                )?;
                let Type::Set(element) = domain_type else {
                    return Err(invalid(at, "finite reduction domain type required"));
                };
                let mut body_types = types.clone();
                body_types.insert(binder.var.clone(), *element.clone());
                let prototype =
                    crate::expression::infer(body, &body_types, self.p, self.c, at, None)?;
                let result_type =
                    crate::expression::infer(expression, &types, self.p, self.c, at, None)?;
                let Value::Set(values) =
                    self.eval(at, &env, &dsl::render_path(&binder.domain), None)?
                else {
                    return Err(invalid(at, "finite reduction set required"));
                };
                let saved = self.lexical.remove(&binder.var);
                let mut terms = Vec::new();
                for value in values {
                    let mut local = env.clone();
                    local.insert(binder.var.clone(), value);
                    if let Some(filter) = &binder.filter
                        && !(Evaluator {
                            package: self.p,
                            physical: self.c,
                            at,
                            env: &local,
                            limit: self.limits.members,
                            stack: Vec::new(),
                            reader: self.reader,
                        })
                        .predicate(filter)?
                    {
                        continue;
                    }
                    terms.push(self.rewrite(instance, body, &local, chain)?);
                }
                if let Some(saved) = saved {
                    self.lexical.insert(binder.var.clone(), saved);
                }
                if let Type::Quantity(scheme) = &prototype {
                    let quantity = scheme
                        .resolve_with_evidence(
                            self.c.quantities,
                            &BTreeMap::new(),
                            self.c.preconditions,
                        )
                        .map_err(|e| invalid(at, e.to_string()))?;
                    let (reduction, domain) =
                        crate::expression::finite_reduction(*kind, &element, at)?;
                    let mut hash = FramedHasher::new(pse_ids::Frame::ModelingFiniteReductionV1);
                    hash.str(kind.as_str())
                        .id(&quantity.as_id())
                        .bool(domain.is_some())
                        .u64(terms.len() as u64);
                    if let Some(domain) = domain {
                        hash.id(&domain.as_id());
                    }
                    // A lowered reduction is a synthesized function declaration.
                    let id = DeclarationId::from(hash.finish_id());
                    let name = format!("f_{}", id.as_id().to_hex());
                    self.model
                        .functions
                        .entry(name.clone())
                        .or_insert(crate::Function {
                            reduction: Some(crate::FiniteReduction {
                                kind: reduction,
                                domain,
                                prototype: quantity,
                            }),
                            validity: None,
                            envelopes: Vec::new(),
                            validity_reads: crate::envelope::Reads::default(),
                            external: None,
                            continuity: None,
                            id,
                            variables: BTreeSet::new(),
                            arguments: (0..terms.len())
                                .map(|i| (format!("term_{i}"), prototype.clone()))
                                .collect(),
                            result: result_type,
                            body: None,
                        });
                    return Ok(Expr {
                        kind: ExprKind::NamedCall { name, args: terms },
                        span: Span::default(),
                    });
                }
                let mut iter = terms.into_iter();
                let mut result = if let Some(first) = iter.next() {
                    first
                } else {
                    let ty = if *kind == ReduceKind::Prod {
                        Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                            self.c
                                .quantities
                                .neutral_dimensionless()
                                .ok_or_else(|| invalid(at, "neutral type absent"))?,
                        ))
                    } else {
                        crate::expression::infer(
                            expression,
                            &self.source_types(at, &env)?,
                            self.p,
                            self.c,
                            at,
                            None,
                        )?
                    };
                    self.constant(if *kind == ReduceKind::Prod { 1.0 } else { 0.0 }, &ty, at)?
                };
                for rhs in iter {
                    result = Expr {
                        kind: ExprKind::Binary {
                            op: if *kind == ReduceKind::Prod {
                                BinaryOp::Mul
                            } else {
                                BinaryOp::Add
                            },
                            lhs: Box::new(result),
                            rhs: Box::new(rhs),
                        },
                        span: Span::default(),
                    };
                }
                return Ok(result);
            }
            ExprKind::Fold { .. } => return self.fold(instance, expression, &env, chain),
            ExprKind::Let { bindings, body } => {
                let saved = self.lexical.clone();
                let saved_types = self.function_types.clone();
                let mut output = Vec::new();
                let mut env = env.clone();
                for (name, value) in bindings {
                    let ty = crate::expression::infer(
                        value,
                        &self.source_types(at, &env)?,
                        self.p,
                        self.c,
                        at,
                        None,
                    )?;
                    // A structural binding, a `Ref` above all, is a static value: a call
                    // through it selects its body at specialization (ADR-0123 Outcome 2).
                    if !matches!(
                        ty,
                        Type::Quantity(_) | Type::Integer | Type::Indexed { .. }
                    ) {
                        let value = self
                            .eval(at, &env, &dsl::render_expr(value), Some(&ty))
                            .map_err(|e| {
                                invalid(
                                    at,
                                    format!("{name} must be static at specialization: {e}"),
                                )
                            })?;
                        self.lexical.remove(name);
                        env.insert(name.clone(), value);
                        continue;
                    }
                    let value = self.rewrite(instance, value, &env, chain)?;
                    self.function_types.insert(name.clone(), ty);
                    self.local_serial += 1;
                    let fresh = format!("__local_{}", self.local_serial);
                    self.lexical.insert(
                        name.clone(),
                        Expr {
                            kind: ExprKind::Path(Path {
                                segments: vec![PathSegment {
                                    name: fresh.clone(),
                                    indices: vec![],
                                }],
                            }),
                            span: Span::default(),
                        },
                    );
                    output.push((fresh, value));
                }
                let result = self.rewrite(instance, body, &env, chain);
                self.lexical = saved;
                self.function_types = saved_types;
                if output.is_empty() {
                    return result;
                }
                ExprKind::Let {
                    bindings: output,
                    body: Box::new(result?),
                }
            }
            ExprKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                let static_guard = Evaluator {
                    package: self.p,
                    physical: self.c,
                    at,
                    env: &env,
                    limit: self.limits.members,
                    stack: Vec::new(),
                    reader: self.reader,
                }
                .predicate(guard);
                if let Ok(selected) = static_guard {
                    return self.rewrite(
                        instance,
                        if selected { then } else { otherwise },
                        &env,
                        chain,
                    );
                }
                let guard = self.rewrite_predicate(instance, guard, &env, chain)?;
                // A call in a branch is evaluated only where the branch is selected; the data
                // layer cannot observe it unconditionally (ADR-0123 Outcome 4).
                self.branches += 1;
                let branches = self
                    .rewrite(instance, then, &env, chain)
                    .and_then(|then| Ok((then, self.rewrite(instance, otherwise, &env, chain)?)));
                self.branches -= 1;
                let (then, otherwise) = branches?;
                ExprKind::Conditional {
                    guard: Box::new(guard),
                    then: Box::new(then),
                    otherwise: Box::new(otherwise),
                }
            }
            ExprKind::Derivative { body, wrt } => {
                return self.derivative(instance, body, wrt, &env, chain);
            }
            ExprKind::Kernel { .. } => {
                return Err(ModelingError::Unsupported {
                    declaration: at.into(),
                    capability: "K5 external functions".into(),
                });
            }
        };
        Ok(Expr {
            kind,
            span: Span::default(),
        })
    }
    fn value_expression(&mut self, value: &Value, at: DeclarationId) -> Result<Expr> {
        if let Value::Coordinate { id, .. } = value
            && let Some(axis) = self
                .model
                .integrated
                .values()
                .find(|axis| axis.coordinate == *id)
        {
            return Ok(symbol_expr(axis.time));
        }
        let number = match value {
            Value::Number { bits, quantity } | Value::Coordinate { bits, quantity, .. } => {
                return self.constant(
                    f64::from_bits(*bits),
                    &Type::Quantity(pse_quantity::scheme::Scheme::Concrete(*quantity)),
                    at,
                );
            }
            Value::Integer(v) => Number {
                exact_integer: Some(i128::from(*v)),
                value: *v as f64,
                unit: None,
            },
            _ => {
                return Err(invalid(
                    at,
                    "non-numeric static value used in a runtime expression",
                ));
            }
        };
        Ok(Expr {
            kind: ExprKind::Number(number),
            span: Span::default(),
        })
    }
    pub(super) fn rewrite_equation(
        &mut self,
        instance: InstanceId,
        e: &Equation,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Equation> {
        let kind = match &e.kind {
            EquationKind::Relation { lhs, sense, rhs } => EquationKind::Relation {
                lhs: self.rewrite(instance, lhs, env, chain)?,
                sense: *sense,
                rhs: self.rewrite(instance, rhs, env, chain)?,
            },
            EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                // As in `rewrite`: attribution falls back to the instance's definition.
                let at = chain
                    .last()
                    .copied()
                    .unwrap_or(self.states[&instance].definition);
                let selected = Evaluator {
                    package: self.p,
                    physical: self.c,
                    at,
                    env,
                    limit: self.limits.members,
                    stack: Vec::new(),
                    reader: self.reader,
                }
                .predicate(guard)?;
                return self.rewrite_equation(
                    instance,
                    if selected { then } else { otherwise },
                    env,
                    chain,
                );
            }
        };
        Ok(Equation {
            kind,
            span: Span::default(),
        })
    }
    pub(crate) fn rewrite_predicate(
        &mut self,
        instance: InstanceId,
        p: &Predicate,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Predicate> {
        let kind = match &p.kind {
            PredicateKind::Compare { op, lhs, rhs } => PredicateKind::Compare {
                op: *op,
                lhs: Box::new(self.rewrite(instance, lhs, env, chain)?),
                rhs: Box::new(self.rewrite(instance, rhs, env, chain)?),
            },
            PredicateKind::Atom(e) => {
                PredicateKind::Atom(Box::new(self.rewrite(instance, e, env, chain)?))
            }
            PredicateKind::And(a, b) => PredicateKind::And(
                Box::new(self.rewrite_predicate(instance, a, env, chain)?),
                Box::new(self.rewrite_predicate(instance, b, env, chain)?),
            ),
            PredicateKind::Or(a, b) => PredicateKind::Or(
                Box::new(self.rewrite_predicate(instance, a, env, chain)?),
                Box::new(self.rewrite_predicate(instance, b, env, chain)?),
            ),
            PredicateKind::Not(p) => {
                PredicateKind::Not(Box::new(self.rewrite_predicate(instance, p, env, chain)?))
            }
            other => other.clone(),
        };
        Ok(Predicate {
            kind,
            span: Span::default(),
        })
    }
}
