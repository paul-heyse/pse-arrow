// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::resolved::{
    Binding, Node, Reference, ResolvedPredicate,
    construction::{Rewritten, RewrittenEquation, RewrittenPredicate},
};
use pse_authoring::dsl::{Predicate, PredicateKind, ReduceKind};
/// The owning instance, the member declaration and the member's index coordinates.
pub(super) type ResolvedPath = (InstanceId, DeclarationId, Vec<(String, Value)>);
/// Exact current instance and lexical expansion environment for a rewrite.
#[derive(Clone, Copy)]
pub(crate) struct RewriteContext<'a> {
    pub(crate) instance: InstanceId,
    pub(crate) env: &'a Environment,
    pub(crate) chain: &'a [DeclarationId],
}
impl Engine<'_, '_> {
    fn set_source_origin(
        &self,
        instance: InstanceId,
        path: &Path,
        env: &Environment,
        at: DeclarationId,
    ) -> Option<&NumericalSource> {
        let [segment] = path.segments.as_slice() else {
            return None;
        };
        if self.p.functions.contains_key(&at)
            || !segment.indices.is_empty()
            || !matches!(env.get(&segment.name), Some(Value::Set(_)))
        {
            return None;
        }
        if let Some(source) = self.set_alias_sources.get(&segment.name) {
            return Some(source);
        }
        if self.source_locals.contains(&segment.name) {
            return None;
        }
        self.numerical_sources
            .get(&(instance, segment.name.clone()))
    }
    // A Set's pre-evaluated value remains authoritative. Re-evaluate only the
    // source of a scope Set that this static read actually consumes, in the
    // current consumer frame, so its selection receipt accompanies the read.
    fn restore_set_source(
        &mut self,
        instance: InstanceId,
        path: &Path,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<()> {
        let [segment] = path.segments.as_slice() else {
            return Ok(());
        };
        let Some(source) = self.set_source_origin(instance, path, env, at).cloned() else {
            return Ok(());
        };
        let key = (instance, segment.name.clone());
        if self.numerical_source_stack.contains(&key) {
            return Err(invalid(at, "recursive scope Set source"));
        }
        if self.numerical_source_stack.len() >= self.limits.depth {
            return Err(ModelingError::Budget("scope Set source depth".into()));
        }
        self.numerical_source_stack.push(key);
        let locals = std::mem::take(&mut self.source_locals);
        let aliases = std::mem::take(&mut self.set_alias_sources);
        let result = (|| {
            // A forwarded scope Set retains its original constructor origin.
            if let ExprKind::Path(path) = &source.expression.kind {
                self.restore_set_source(source.instance, path, &source.env, source.declaration)?;
            }
            let replay =
                self.eval_ast_with(source.declaration, &source.env, &source.expression, None)?;
            if Some(&replay) != env.get(&segment.name) {
                return Err(invalid(
                    at,
                    "scope Set source differs from its admitted binding",
                ));
            }
            Ok(())
        })();
        self.source_locals = locals;
        self.set_alias_sources = aliases;
        self.numerical_source_stack.pop();
        result
    }
    fn static_source_predicate(
        &mut self,
        instance: InstanceId,
        predicate: &Predicate,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<bool> {
        match &predicate.kind {
            PredicateKind::And(a, b) => Ok(self.static_source_predicate(instance, a, env, at)?
                && self.static_source_predicate(instance, b, env, at)?),
            PredicateKind::Or(a, b) => Ok(self.static_source_predicate(instance, a, env, at)?
                || self.static_source_predicate(instance, b, env, at)?),
            PredicateKind::Not(p) => Ok(!self.static_source_predicate(instance, p, env, at)?),
            PredicateKind::In { domain, .. } => {
                self.restore_set_source(instance, domain, env, at)?;
                Evaluator {
                    lexical_bases: Some(&self.lexical_bases),
                    package: self.p,
                    physical: self.c,
                    at,
                    env,
                    limit: self.limits.members,
                    stack: Vec::new(),
                    reader: self.reader,
                    selections: Some(&self.selection_collector),
                }
                .predicate(predicate)
            }
            _ => Evaluator {
                lexical_bases: Some(&self.lexical_bases),
                package: self.p,
                physical: self.c,
                at,
                env,
                limit: self.limits.members,
                stack: Vec::new(),
                reader: self.reader,
                selections: Some(&self.selection_collector),
            }
            .predicate(predicate),
        }
    }
    // Integrated coordinates are runtime inputs for numerical conditions. Their
    // admitted lower endpoint still selects topology, but cannot fold a check or
    // a numerical conditional for the entire trajectory.
    fn numerical_guard_environment<'a>(
        &self,
        env: &'a Environment,
    ) -> std::borrow::Cow<'a, Environment> {
        let runtime = |value: &Value| {
            matches!(value, Value::Coordinate { id, .. }
            if self.model.integrated.values().any(|axis| axis.coordinate == *id))
        };
        if env.values().any(runtime) {
            std::borrow::Cow::Owned(
                env.iter()
                    .filter(|(_, value)| !runtime(value))
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect(),
            )
        } else {
            std::borrow::Cow::Borrowed(env)
        }
    }
    pub(super) fn temporal_active(&self, instance: InstanceId, member: DeclarationId) -> bool {
        self.p.temporal.get(&member).is_some_and(|(policy, _, _)| {
            self.states[&instance]
                .members
                .values()
                .any(|active| active == policy)
        })
    }
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
            self.states[&instance].member_kinds[&member],
            Kind::Child | Kind::Implicit
        ) {
            return Err(invalid(at, "child instance required"));
        }
        let values = segment
            .indices
            .iter()
            .map(|index| self.eval_ast_with(at, env, index, None))
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
        let declaration = self.p.declarations.get(&member).ok_or_else(|| {
            invalid(
                member,
                format!(
                    "field context lacks member coordinate contract: owner={} member={:?}",
                    self.states[&instance].path,
                    self.states[&instance]
                        .members
                        .iter()
                        .find(|(_, id)| **id == member)
                        .map(|(name, _)| name)
                ),
            )
        })?;
        let indices = if let Some(v) = &declaration.value.binding {
            v.indices
                .iter()
                .filter(|index| {
                    index.name != crate::temporal::COORDINATE
                        || self.temporal_active(instance, member)
                })
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect::<Vec<_>>()
        } else if let Some(v) = &declaration.value.accumulator {
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else if let Some(boundary) = &declaration.value.boundary {
            boundary
                .indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else if let Some(exchange) = &declaration.value.exchange {
            exchange
                .indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else if let Some(v) = &declaration.value.state_specification {
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else if let Some(v) = &declaration.value.state_port {
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else if let Some(e) = &declaration.value.equation {
            e.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else if let Some(connection) = &declaration.value.connection {
            connection
                .indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str()))
                .collect()
        } else {
            Vec::new()
        };
        if indices.len() != values.len() {
            return Err(crate::invalid_reason(
                member,
                crate::ModelingContractReason::MemberIndexArity,
                "member index arity differs",
            ));
        }
        let admitted = self.coordinates(
            instance,
            member,
            &self.states[&instance].env,
            indices.into_iter(),
        )?;
        admitted.into_iter().find(|row| row.iter().zip(&values).all(|((_, x), y)| {
            x == y || matches!((x,y),
                (Value::Coordinate{quantity:q,bits:a,..},Value::Number{quantity:r,bits:b}) if q==r && a==b)
        })).ok_or_else(|| crate::invalid_reason(member, crate::ModelingContractReason::CoordinateOutsideMembership, "coordinate outside declared membership"))
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
                    lexical_bases: Some(&self.lexical_bases),
                    package: self.p,
                    physical: self.c,
                    at,
                    env,
                    limit: self.limits.members,
                    stack: Vec::new(),
                    reader: self.reader,
                    selections: Some(&self.selection_collector),
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
                .filter(|id| state.member_kinds.get(id) == Some(&Kind::Disjunction))
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
    ) -> Result<Rewritten> {
        if !self.field_fragment_active && self.function_symbols.is_none() {
            return self.rewrite_local_field(instance, expression, local, chain);
        }
        self.reserve(1)?;
        let at = chain
            .last()
            .copied()
            .unwrap_or(self.states[&instance].definition);
        let source = self.original_expression_source(expression, at)?;
        let in_function = self.p.functions.contains_key(&at);
        let mut env = if in_function {
            Environment::new()
        } else {
            self.states[&instance].env.clone()
        };
        env.extend(local.clone());
        let (node, kind) = match &expression.kind {
            ExprKind::Number(n) => (Node::Number(n.clone()), ExprKind::Number(n.clone())),
            ExprKind::Path(path) => {
                let text = dsl::render_path(path);
                if let Some(value) = self.lexical.get(&text) {
                    let inspection = value.clone();
                    let reference =
                        self.lexical_references.get(&text).cloned().ok_or_else(|| {
                            invalid(at, "lexical value lacks its actual typed binding")
                        })?;
                    return self.construction.pair(
                        Node::Reference(reference),
                        inspection.kind,
                        source,
                    );
                }
                if path.segments.len() == 1
                    && let Some(group) = self.indexed_arguments.get(&path.segments[0].name)
                {
                    let coordinates = path.segments[0]
                        .indices
                        .iter()
                        .map(|e| self.eval_ast_with(at, &env, e, None))
                        .collect::<Result<Vec<_>>>()?;
                    let selected = group.members.get(&coordinates).cloned().ok_or_else(|| {
                        invalid(at, "function coordinate outside actual membership")
                    })?;
                    return self.construction.pair(
                        Node::Reference(selected.reference),
                        selected.inspection.kind,
                        source,
                    );
                }
                if let Some(value) = env.get(&text) {
                    if !in_function
                        && path.segments.len() == 1
                        && path.segments[0].indices.is_empty()
                    {
                        return self.scope_parameter_expression(instance, &text, value, at, source);
                    }
                    return self
                        .scientific_value_expression(instance, path, value, at, &env, source);
                }
                // A pure function's free names belong to its definition scope. A caller
                // instance cannot capture a table, entity or constant with a same-named member.
                if in_function {
                    let value = self.eval_ast(
                        at,
                        &env,
                        &Expr {
                            kind: ExprKind::Path(path.clone()),
                            span: Span::default(),
                        },
                    )?;
                    return self
                        .scientific_value_expression(instance, path, &value, at, &env, source);
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
                        return self.symbol_value_expression(
                            owner,
                            member,
                            &coordinates,
                            id,
                            source,
                        );
                    }
                    let value = self.eval_ast(
                        at,
                        &env,
                        &Expr {
                            kind: ExprKind::Path(path.clone()),
                            span: Span::default(),
                        },
                    )?;
                    return self
                        .scientific_value_expression(instance, path, &value, at, &env, source);
                }
                let (owner, member, coordinates) =
                    self.resolve_path(instance, at, path, &env, false)?;
                let id = self.symbol(owner, member, &coordinates, chain)?;
                return self.symbol_value_expression(owner, member, &coordinates, id, source);
            }
            ExprKind::Neg(e) => {
                let value = self.rewrite(instance, e, &env, chain)?;
                (
                    Node::Neg(value.root),
                    ExprKind::Neg(Box::new(value.inspection)),
                )
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let lhs = self.rewrite(instance, lhs, &env, chain)?;
                let rhs = self.rewrite(instance, rhs, &env, chain)?;
                (
                    Node::Binary {
                        op: *op,
                        lhs: lhs.root,
                        rhs: rhs.root,
                    },
                    ExprKind::Binary {
                        op: *op,
                        lhs: Box::new(lhs.inspection),
                        rhs: Box::new(rhs.inspection),
                    },
                )
            }
            ExprKind::Call { function, args } => {
                let args = args
                    .iter()
                    .map(|arg| self.rewrite(instance, arg, &env, chain))
                    .collect::<Result<Vec<_>>>()?;
                let roots = args.iter().map(|arg| arg.root).collect();
                let inspection = args.into_iter().map(|arg| arg.inspection).collect();
                (
                    Node::Call {
                        function: *function,
                        args: roots,
                    },
                    ExprKind::Call {
                        function: *function,
                        args: inspection,
                    },
                )
            }
            ExprKind::NamedCall { name, args } => {
                if let Some(value) = self.physical_operation_call(
                    RewriteContext {
                        instance,
                        env: &env,
                        chain,
                    },
                    at,
                    &dsl::render_path(name),
                    args,
                    source.clone(),
                )? {
                    return Ok(value);
                }
                if let Some(value) = self.contextual_call(
                    RewriteContext {
                        instance,
                        env: &env,
                        chain,
                    },
                    at,
                    &dsl::render_path(name),
                    args,
                    source.clone(),
                )? {
                    return Ok(value);
                }
                let function = self.resolve_function(instance, at, name, &env)?;
                self.function_call(instance, function, args, &[], &env, chain)?
                    .into_parts()
            }
            ExprKind::Partial {
                function,
                wrt,
                args,
            } => {
                let function = self.resolve_function(instance, at, function, &env)?;
                self.function_call(instance, function, args, wrt, &env, chain)?
                    .into_parts()
            }
            ExprKind::Reduce { kind, binder, body } => {
                if *kind == ReduceKind::Integral {
                    return self.integral(
                        RewriteContext {
                            instance,
                            env: &env,
                            chain,
                        },
                        binder,
                        body,
                        None,
                        source,
                    );
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
                let mut body_types = crate::TypeScope::new(&types);
                body_types.insert(binder.var.clone(), *element.clone());
                let prototype =
                    crate::expression::infer(body, &body_types, self.p, self.c, at, None)?;
                let result_type =
                    crate::expression::infer(expression, &types, self.p, self.c, at, None)?;
                self.restore_set_source(instance, &binder.domain, &env, at)?;
                let Value::Set(values) = self.eval_ast_with(
                    at,
                    &env,
                    &Expr {
                        kind: ExprKind::Path(binder.domain.clone()),
                        span: Span::default(),
                    },
                    None,
                )?
                else {
                    return Err(invalid(at, "finite reduction set required"));
                };
                let carrier = crate::resolved::Carrier {
                    kind: crate::resolved::CarrierKind::OrderedSet,
                    element: *element.clone(),
                    members: values.clone().into(),
                };
                let mut selected_members = Vec::new();
                let saved = self.lexical.remove(&binder.var);
                let saved_reference = self.lexical_references.remove(&binder.var);
                let mut terms = Vec::new();
                for value in values {
                    let mut local = env.clone();
                    local.insert(binder.var.clone(), value.clone());
                    if let Some(filter) = &binder.filter
                        && !(Evaluator {
                            lexical_bases: Some(&self.lexical_bases),
                            package: self.p,
                            physical: self.c,
                            at,
                            env: &local,
                            limit: self.limits.members,
                            stack: Vec::new(),
                            reader: self.reader,
                            selections: Some(&self.selection_collector),
                        })
                        .predicate(filter)?
                    {
                        continue;
                    }
                    selected_members.push(value);
                    terms.push(self.rewrite(instance, body, &local, chain)?);
                }
                if let Some(saved) = saved {
                    self.lexical.insert(binder.var.clone(), saved);
                }
                if let Some(reference) = saved_reference {
                    self.lexical_references
                        .insert(binder.var.clone(), reference);
                }
                return self.finish_finite_reduction(
                    *kind,
                    finite::ReductionSelection {
                        carrier,
                        selected: selected_members.into(),
                    },
                    (prototype, result_type),
                    terms,
                    source,
                    at,
                );
            }
            ExprKind::Fold { .. } => return self.fold(instance, expression, &env, chain, source),
            ExprKind::Let { bindings, body } => {
                let saved = self.lexical.clone();
                let saved_references = self.lexical_references.clone();
                let saved_types = self.function_types.clone();
                let saved_locals = self.source_locals.clone();
                let mut alias_nodes = self.set_alias_sources.len();
                for source in self.set_alias_sources.values() {
                    source
                        .expression
                        .walk(|_| alias_nodes = alias_nodes.saturating_add(1));
                }
                self.reserve(alias_nodes)?;
                let saved_aliases = self.set_alias_sources.clone();
                let mut output = Vec::new();
                let mut resolved_bindings = Vec::new();
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
                    if !matches!(ty, Type::Quantity(_) | Type::Integer | Type::Indexed { .. }) {
                        // A plain Set alias keeps the exact source it read before
                        // the new local name shadows any scope argument.
                        let origin = if matches!(ty, Type::Set(_))
                            && let ExprKind::Path(path) = &value.kind
                        {
                            self.set_source_origin(instance, path, &env, at).cloned()
                        } else {
                            None
                        };
                        let value =
                            self.eval_ast_with(at, &env, value, Some(&ty))
                                .map_err(|e| {
                                    invalid(
                                        at,
                                        format!("{name} must be static at specialization: {e}"),
                                    )
                                })?;
                        self.lexical.remove(name);
                        self.lexical_references.remove(name);
                        env.insert(name.clone(), value);
                        self.source_locals.insert(name.clone());
                        self.set_alias_sources.remove(name);
                        if let Some(origin) = origin {
                            let mut count = 0usize;
                            origin.expression.walk(|_| count += 1);
                            self.reserve(count.saturating_add(1))?;
                            self.set_alias_sources.insert(name.clone(), origin);
                        }
                        continue;
                    }
                    let value = self.rewrite(instance, value, &env, chain)?;
                    let binding = self.construction.binding(Binding::Value(value.root))?;
                    resolved_bindings.push(binding);
                    self.lexical_references
                        .insert(name.clone(), Reference::Lexical(binding));
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
                    self.source_locals.insert(name.clone());
                    self.set_alias_sources.remove(name);
                    output.push((fresh, value.inspection));
                }
                let result = self.rewrite(instance, body, &env, chain);
                self.lexical = saved;
                self.lexical_references = saved_references;
                self.function_types = saved_types;
                self.source_locals = saved_locals;
                self.set_alias_sources = saved_aliases;
                if output.is_empty() {
                    return result;
                }
                let result = result?;
                (
                    Node::Let {
                        bindings: resolved_bindings,
                        body: result.root,
                    },
                    ExprKind::Let {
                        bindings: output,
                        body: Box::new(result.inspection),
                    },
                )
            }
            ExprKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                let static_env = self.numerical_guard_environment(&env).into_owned();
                let static_guard = self.static_source_predicate(instance, guard, &static_env, at);
                if let Ok(selected) = static_guard {
                    self.record_static_guard(guard, selected, at)?;
                    return self.rewrite(
                        instance,
                        if selected { then } else { otherwise },
                        &env,
                        chain,
                    );
                }
                let guard = self.rewrite_predicate(instance, guard, &env, chain)?;
                let branches = self
                    .rewrite(instance, then, &env, chain)
                    .and_then(|then| Ok((then, self.rewrite(instance, otherwise, &env, chain)?)));
                let (then, otherwise) = branches?;
                (
                    Node::Conditional {
                        guard: guard.root,
                        then: then.root,
                        otherwise: otherwise.root,
                    },
                    ExprKind::Conditional {
                        guard: Box::new(guard.inspection),
                        then: Box::new(then.inspection),
                        otherwise: Box::new(otherwise.inspection),
                    },
                )
            }
            ExprKind::Derivative { body, wrt } => {
                return self.derivative(
                    RewriteContext {
                        instance,
                        env: &env,
                        chain,
                    },
                    body,
                    wrt,
                    None,
                    source,
                );
            }
            ExprKind::Kernel { .. } => {
                return Err(ModelingError::Unsupported {
                    declaration: at.into(),
                    capability: "K5 external functions".into(),
                });
            }
        };
        self.pair_expression(node, kind, source, Some(expression))
    }
    pub(super) fn value_expression(
        &mut self,
        value: &Value,
        at: DeclarationId,
        source: crate::resolved::Source,
    ) -> Result<Rewritten> {
        if let Value::Coordinate { id, .. } = value
            && let Some(axis) = self
                .model
                .integrated
                .values()
                .find(|axis| axis.coordinate == *id)
        {
            let slot = self.model.symbols.slot(axis.time).ok_or_else(|| {
                invalid(at, "integrated coordinate lacks its authoritative scalar")
            })?;
            return self.construction.pair(
                Node::Reference(Reference::Symbol(slot)),
                symbol_expr(axis.time).kind,
                source,
            );
        }
        match value {
            Value::Number { bits, quantity } | Value::Coordinate { bits, quantity, .. } => self
                .constant(
                    f64::from_bits(*bits),
                    &Type::Quantity(pse_quantity::scheme::Scheme::Concrete(*quantity)),
                    at,
                    source,
                ),
            Value::Integer(value) => {
                let number = Number {
                    exact_integer: Some(i128::from(*value)),
                    value: *value as f64,
                    unit: None,
                };
                self.construction.pair(
                    Node::Number(number.clone()),
                    ExprKind::Number(number),
                    source,
                )
            }
            _ => Err(invalid(
                at,
                "non-numeric static value used in a runtime expression",
            )),
        }
    }
    pub(super) fn rewrite_equation(
        &mut self,
        instance: InstanceId,
        equation: &Equation,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<RewrittenEquation> {
        let at = chain
            .last()
            .copied()
            .unwrap_or(self.states[&instance].definition);
        match &equation.kind {
            EquationKind::Relation { lhs, sense, rhs } => {
                let lhs = self.rewrite(instance, lhs, env, chain)?;
                let rhs = self.rewrite(instance, rhs, env, chain)?;
                Ok(RewrittenEquation {
                    lhs: lhs.root,
                    sense: *sense,
                    rhs: rhs.root,
                    inspection: Equation {
                        kind: EquationKind::Relation {
                            lhs: lhs.inspection,
                            sense: *sense,
                            rhs: rhs.inspection,
                        },
                        span: equation.span,
                    },
                })
            }
            EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                let selected = Evaluator {
                    lexical_bases: Some(&self.lexical_bases),
                    package: self.p,
                    physical: self.c,
                    at,
                    env,
                    limit: self.limits.members,
                    stack: Vec::new(),
                    reader: self.reader,
                    selections: Some(&self.selection_collector),
                }
                .predicate(guard)?;
                self.record_static_guard(guard, selected, at)?;
                self.rewrite_equation(
                    instance,
                    if selected { then } else { otherwise },
                    env,
                    chain,
                )
            }
        }
    }
    pub(crate) fn rewrite_predicate(
        &mut self,
        instance: InstanceId,
        predicate: &Predicate,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<RewrittenPredicate> {
        let at = chain
            .last()
            .copied()
            .unwrap_or(self.states[&instance].definition);
        let source = self.original_predicate_source(predicate, at)?;
        let static_env = self.numerical_guard_environment(env).into_owned();
        if let Ok(value) = self.static_source_predicate(instance, predicate, &static_env, at) {
            self.record_static_guard(predicate, value, at)?;
            return self.pair_predicate(
                ResolvedPredicate::Bool(value),
                PredicateKind::Bool(value),
                source,
            );
        }
        let (node, inspection) = match &predicate.kind {
            PredicateKind::Compare { op, lhs, rhs } => {
                let lhs = self.rewrite(instance, lhs, env, chain)?;
                let rhs = self.rewrite(instance, rhs, env, chain)?;
                (
                    ResolvedPredicate::Compare {
                        op: *op,
                        lhs: lhs.root,
                        rhs: rhs.root,
                    },
                    PredicateKind::Compare {
                        op: *op,
                        lhs: Box::new(lhs.inspection),
                        rhs: Box::new(rhs.inspection),
                    },
                )
            }
            PredicateKind::Atom(expression) => {
                if matches!(&expression.kind,ExprKind::NamedCall {name,..} if name.is_ident("present"))
                {
                    let value = Evaluator {
                        lexical_bases: Some(&self.lexical_bases),
                        package: self.p,
                        physical: self.c,
                        at,
                        env: &static_env,
                        limit: self.limits.members,
                        stack: Vec::new(),
                        reader: self.reader,
                        selections: Some(&self.selection_collector),
                    }
                    .predicate(predicate)?;
                    self.record_static_guard(predicate, value, at)?;
                    return self.pair_predicate(
                        ResolvedPredicate::Bool(value),
                        PredicateKind::Bool(value),
                        source,
                    );
                }
                let expression = self.rewrite(instance, expression, env, chain)?;
                (
                    ResolvedPredicate::Atom(expression.root),
                    PredicateKind::Atom(Box::new(expression.inspection)),
                )
            }
            PredicateKind::And(lhs, rhs) => {
                let lhs = self.rewrite_predicate(instance, lhs, env, chain)?;
                let rhs = self.rewrite_predicate(instance, rhs, env, chain)?;
                (
                    ResolvedPredicate::And(lhs.root, rhs.root),
                    PredicateKind::And(Box::new(lhs.inspection), Box::new(rhs.inspection)),
                )
            }
            PredicateKind::Or(lhs, rhs) => {
                let lhs = self.rewrite_predicate(instance, lhs, env, chain)?;
                let rhs = self.rewrite_predicate(instance, rhs, env, chain)?;
                (
                    ResolvedPredicate::Or(lhs.root, rhs.root),
                    PredicateKind::Or(Box::new(lhs.inspection), Box::new(rhs.inspection)),
                )
            }
            PredicateKind::Not(value) => {
                let value = self.rewrite_predicate(instance, value, env, chain)?;
                (
                    ResolvedPredicate::Not(value.root),
                    PredicateKind::Not(Box::new(value.inspection)),
                )
            }
            PredicateKind::In { .. } => {
                return Err(invalid(
                    at,
                    "finite membership must be selected from its actual static carrier",
                ));
            }
            PredicateKind::Bool(value) => {
                (ResolvedPredicate::Bool(*value), PredicateKind::Bool(*value))
            }
            PredicateKind::Null => (ResolvedPredicate::Null, PredicateKind::Null),
        };
        self.pair_predicate(node, inspection, source)
    }
}
