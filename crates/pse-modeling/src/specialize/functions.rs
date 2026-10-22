// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite function arguments become explicit scalar formals before library differentiation.
use super::rewrite::RewriteContext;
use super::*;
use crate::resolved::{
    FunctionProduct, Node, Reference, Source, SymbolIx,
    construction::{Construction, Rewritten, RewrittenKind, ScalarSelection},
    functions::{
        ArgumentLayout, EvidenceUse, FunctionEnvelope, FunctionExternal, SelectionMeaning,
    },
};
use std::sync::Arc;
/// Generated callers pass actual admitted values directly to the one selector algorithm.
#[derive(Clone, Debug)]
pub(super) enum PreparedArgument {
    Scalar { value: Rewritten, ty: Type },
    Static { value: Value, ty: Type },
    Indexed { value: ActualGroup, ty: Type },
}
impl PreparedArgument {
    fn ty(&self) -> &Type {
        match self {
            Self::Scalar { ty, .. } | Self::Static { ty, .. } | Self::Indexed { ty, .. } => ty,
        }
    }
    fn value(&self) -> Option<&Value> {
        match self {
            Self::Static { value, .. } => Some(value),
            Self::Scalar { .. } | Self::Indexed { .. } => None,
        }
    }
}
#[derive(Clone, Debug)]
pub(super) struct IndexedSelection {
    pub(super) members: BTreeMap<Vec<Value>, ScalarSelection>,
    pub(super) domains: Vec<(usize, Vec<Value>, crate::resolved::Carrier)>,
}
#[derive(Clone, Debug)]
pub(super) struct ActualGroup {
    pub(super) members: Vec<(Vec<Value>, Rewritten)>,
    pub(super) domains: Vec<(usize, Vec<Value>, crate::resolved::Carrier)>,
}

impl Engine<'_, '_> {
    pub(super) fn resolve_function(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        name: &Path,
        env: &Environment,
    ) -> Result<DeclarationId> {
        self.resolve_function_binding(instance, at, name, env, &mut BTreeSet::new())
    }
    fn resolve_function_binding(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        name: &Path,
        env: &Environment,
        stack: &mut BTreeSet<(InstanceId, DeclarationId, Vec<SemanticId>)>,
    ) -> Result<DeclarationId> {
        let rendered = dsl::render_path(name);
        if let Some(Value::Function(id)) = env.get(&rendered) {
            return Ok(*id);
        }
        if self.p.functions.contains_key(&at) {
            return match self.eval_ast(
                at,
                env,
                &Expr {
                    kind: ExprKind::Path(name.clone()),
                    span: Span::default(),
                },
            )? {
                Value::Function(id) => Ok(id),
                _ => Err(invalid(
                    at,
                    "function definition scope requires a pure function",
                )),
            };
        }
        let expression = Expr {
            kind: ExprKind::Path(name.clone()),
            span: Span::default(),
        };
        if let ExprKind::Path(path) = &expression.kind
            && let Ok((owner, member, coordinates)) =
                self.resolve_path(instance, at, path, env, false)
        {
            if self.p.functions.contains_key(&member) {
                return Ok(member);
            }
            let row = &self.p.declarations[&member];
            if row.value.kind == Kind::Parameter
                && matches!(self.p.types.get(&member), Some(Type::Function { .. }))
            {
                let key = (
                    owner,
                    member,
                    coordinates
                        .iter()
                        .map(|(_, value)| value.identity())
                        .collect(),
                );
                if stack.len() >= self.limits.depth || !stack.insert(key.clone()) {
                    return Err(invalid(member, "recursive function parameter binding"));
                }
                let ExprKind::Path(source) =
                    &self.p.expression_at(member, "binding.expression", 0)?.kind
                else {
                    return Err(invalid(
                        member,
                        "function parameter requires a checked path implementation",
                    ));
                };
                let local = coordinates_env(&self.states[&owner].env, &coordinates);
                let result = self.resolve_function_binding(owner, member, source, &local, stack)?;
                stack.remove(&key);
                self.compatible(&Value::Function(result), &self.p.types[&member], member)?;
                return Ok(result);
            }
        }
        if let Some(id) = self.states[&instance]
            .members
            .get(&rendered)
            .copied()
            .or_else(|| self.p.resolve_segments(at, &name.segments))
            .filter(|id| self.p.functions.contains_key(id))
        {
            return Ok(id);
        }
        if let Value::Function(id) = self
            .eval_ast(at, env, &expression)
            .map_err(|e| invalid(at, format!("function {name}: {e}")))?
        {
            return Ok(id);
        }
        Err(invalid(at, "selected value is not a pure function"))
    }
    pub(super) fn source_types(
        &self,
        at: DeclarationId,
        env: &Environment,
    ) -> Result<crate::TypeScope<'_>> {
        let base = self.lexical_bases.source(self.p, self.c, at)?;
        let mut lexical = self.p.named_types(at);
        lexical.extend(base.iter().map(|(name, ty)| (name.clone(), ty.clone())));
        let mut types = crate::TypeScope::owned(lexical).with_upper(&self.function_types);
        types.extend(
            env.iter()
                .filter_map(|(name, value)| value_type(value).map(|ty| (name.clone(), ty))),
        );
        Ok(types)
    }

    /// Canonical physical constant emitted directly with its actual declared literal contract.
    pub(super) fn constant(
        &mut self,
        value: f64,
        ty: &Type,
        at: DeclarationId,
        source: Source,
    ) -> Result<Rewritten> {
        let quantity = ty
            .quantity_scheme()
            .ok_or_else(|| invalid(at, "physical constant type required"))?
            .resolve_contract_with_evidence(
                self.c.quantities,
                &BTreeMap::new(),
                self.c.preconditions,
            )
            .map_err(|error| invalid(at, error.to_string()))?;
        let named = quantity
            .require_named()
            .map_err(|error| invalid(at, error.to_string()))?;
        let unit = self
            .c
            .quantities
            .quantity_type(named)
            .and_then(|quantity| self.c.quantities.unit_product(quantity.canonical_unit))
            .map_err(|error| invalid(at, error.to_string()))?;
        let number = Number {
            value,
            exact_integer: None,
            unit: Some(unit),
        };
        let result = self.construction.pair(
            Node::Number(number.clone()),
            ExprKind::Number(number),
            source,
        )?;
        self.construction.literal(result.root, ty.clone())?;
        Ok(result)
    }
    /// Resolve an indexed actual without inventing coordinates from observed values.
    pub(super) fn argument_group(
        &mut self,
        instance: InstanceId,
        path: &Path,
        env: &Environment,
        chain: &[DeclarationId],
        source: Source,
    ) -> Result<ActualGroup> {
        let (first, rest) = path
            .segments
            .split_first()
            .ok_or_else(|| invalid(instance, "empty indexed argument"))?;
        if rest.is_empty()
            && first.indices.is_empty()
            && let Some(values) = self.indexed_arguments.get(&first.name)
        {
            let values = values.clone();
            let members = values
                .members
                .into_iter()
                .map(|(tuple, selected)| {
                    Ok((
                        tuple,
                        self.construction.pair(
                            Node::Reference(selected.reference),
                            selected.inspection.kind,
                            source.clone(),
                        )?,
                    ))
                })
                .collect::<Result<_>>()?;
            return Ok(ActualGroup {
                members,
                domains: values.domains,
            });
        }
        if first.name == "parent" {
            let parent = self.states[&instance]
                .parent
                .ok_or_else(|| invalid(instance, "root has no parent"))?;
            return self.argument_group(
                parent,
                &Path {
                    segments: rest.to_vec(),
                },
                env,
                chain,
                source,
            );
        }
        let state = &self.states[&instance];
        if !state.members.contains_key(&first.name)
            && !state.children.keys().any(|(name, _)| name == &first.name)
            && self.p.declarations[&state.definition].value.kind == Kind::Implicit
            && let Some(parent) = state.parent
        {
            return self.argument_group(parent, path, env, chain, source);
        }
        if !rest.is_empty() {
            let child = self.child_instance(
                instance,
                chain.last().copied().unwrap_or(state.definition),
                first,
                env,
            )?;
            return self.argument_group(
                child,
                &Path {
                    segments: rest.to_vec(),
                },
                env,
                chain,
                source,
            );
        }
        if !first.indices.is_empty() {
            return Err(invalid(instance, "whole indexed argument required"));
        }
        let member = state
            .members
            .get(&first.name)
            .copied()
            .ok_or_else(|| invalid(instance, "indexed argument member absent"))?;
        let binding = self.p.declarations[&member]
            .value
            .binding
            .as_ref()
            .ok_or_else(|| invalid(member, "indexed physical member required"))?;
        let coordinates = self.coordinates(
            instance,
            member,
            &state.env,
            binding
                .indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str())),
        )?;
        let family = self.finite_family(
            instance,
            member,
            &crate::expression::member_index_role(&self.p.declarations[&member]),
            &state.env,
        );
        let domains = self.finite_collector.actual_domains(&family)?;
        let members = coordinates
            .into_iter()
            .map(|coordinates| {
                let symbol = self.symbol(instance, member, &coordinates, chain)?;
                let expression = self.symbol_value_expression(
                    instance,
                    member,
                    &coordinates,
                    symbol,
                    source.clone(),
                )?;
                Ok((
                    coordinates.into_iter().map(|(_, value)| value).collect(),
                    expression,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(ActualGroup { members, domains })
    }

    pub(super) fn function_call(
        &mut self,
        instance: InstanceId,
        function: DeclarationId,
        args: &[Expr],
        wrt: &[Path],
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<RewrittenKind> {
        self.selected_function_call(
            RewriteContext {
                instance,
                env,
                chain,
            },
            function,
            args,
            wrt,
            None,
        )
    }
    pub(super) fn function_call_prepared(
        &mut self,
        instance: InstanceId,
        function: DeclarationId,
        arguments: &[PreparedArgument],
        mut source: Source,
    ) -> Result<Rewritten> {
        source.coordinate = None;
        let inspections = arguments
            .iter()
            .enumerate()
            .map(|(ordinal, argument)| match argument {
                PreparedArgument::Scalar { value, .. } => value.inspection.clone(),
                PreparedArgument::Static { .. } | PreparedArgument::Indexed { .. } => Expr {
                    kind: ExprKind::Path(Path::single(format!("__prepared_static_{ordinal}"))),
                    span: source.span,
                },
            })
            .collect::<Vec<_>>();
        let selected = self.selected_function_call(
            RewriteContext {
                instance,
                env: &Environment::new(),
                chain: &[function],
            },
            function,
            &inspections,
            &[],
            Some(arguments),
        )?;
        self.construction
            .pair(selected.node, selected.inspection, source)
    }
    fn selected_function_call(
        &mut self,
        rewrite: RewriteContext<'_>,
        function: DeclarationId,
        args: &[Expr],
        wrt: &[Path],
        prepared: Option<&[PreparedArgument]>,
    ) -> Result<RewrittenKind> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
        if prepared.is_some_and(|arguments| arguments.len() != args.len()) {
            return Err(invalid(function, "prepared function actual arity differs"));
        }
        self.reserve(1)?;
        if self.function_stack.len() >= self.limits.depth {
            return Err(ModelingError::Budget("function expansion depth".into()));
        }
        if self.function_stack.contains(&function) {
            return Err(invalid(function, "recursive function expansion"));
        }
        let mut contract = self
            .p
            .functions
            .get(&function)
            .cloned()
            .ok_or_else(|| invalid(function, "call does not name a function"))?;
        if contract.arguments.len() != args.len() {
            return Err(invalid(function, "function argument count"));
        }
        let at = chain.last().copied().unwrap_or(function);
        let types = prepared
            .is_none()
            .then(|| self.source_types(at, env))
            .transpose()?;
        let mut substitution = BTreeMap::new();
        fn bind(
            formal: &Type,
            actual: &Type,
            c: &TypeContext<'_>,
            bindings: &mut pse_quantity::scheme::Substitution,
            at: DeclarationId,
        ) -> Result<()> {
            match (formal, actual) {
                (Type::Quantity(formal), Type::Quantity(actual)) => {
                    let actual = actual
                        .resolve_contract_with_evidence(
                            c.quantities,
                            &BTreeMap::new(),
                            c.preconditions,
                        )
                        .map_err(|e| invalid(at, e.to_string()))?;
                    formal
                        .bind_contract_with_evidence(
                            &actual,
                            c.quantities,
                            bindings,
                            c.preconditions,
                        )
                        .map_err(|e| invalid(at, e.to_string()))
                }
                (
                    Type::Indexed {
                        element: a,
                        axes: x,
                    },
                    Type::Indexed {
                        element: b,
                        axes: y,
                    },
                ) if x == y => bind(a, b, c, bindings, at),
                (a, b) if a == b => Ok(()),
                // A static entity argument is checked on its value below: a call through a
                // `Ref` may pass a refinement of the declared kind (ADR-0123 Outcome 2).
                (Type::Entity(_), Type::Entity(_)) => Ok(()),
                _ => Err(invalid(at, "function actual type differs")),
            }
        }
        for (ordinal, ((_, formal), expr)) in contract.arguments.iter().zip(args).enumerate() {
            let actual = if let Some(prepared) = prepared {
                prepared[ordinal].ty().clone()
            } else {
                crate::expression::infer(
                    expr,
                    types
                        .as_ref()
                        .ok_or_else(|| invalid(at, "source call lacks its argument type scope"))?,
                    self.p,
                    self.c,
                    at,
                    Some(formal),
                )?
            };
            bind(formal, &actual, self.c, &mut substitution, at)?;
        }
        fn instantiate(
            ty: &mut Type,
            c: &TypeContext<'_>,
            bindings: &pse_quantity::scheme::Substitution,
            at: DeclarationId,
        ) -> Result<()> {
            match ty {
                Type::Quantity(scheme)
                | Type::RefinedQuantity {
                    quantity: scheme, ..
                } => {
                    *scheme = pse_quantity::scheme::Scheme::from_contract(
                        scheme
                            .resolve_contract_with_evidence(c.quantities, bindings, c.preconditions)
                            .map_err(|e| invalid(at, e.to_string()))?,
                    )
                }
                Type::Indexed { element, .. } => instantiate(element, c, bindings, at)?,
                _ => {}
            }
            Ok(())
        }
        for (_, ty) in &mut contract.arguments {
            instantiate(ty, self.c, &substitution, function)?;
        }
        instantiate(&mut contract.result, self.c, &substitution, function)?;
        contract.variables.clear();
        let mut actual = Vec::new();
        let mut formals = Vec::new();
        let mut lexical = BTreeMap::new();
        let mut indexed = BTreeMap::new();
        let mut statics = Environment::new();
        let mut lexical_references = BTreeMap::new();
        let mut argument_layouts = Vec::new();
        let mut selectors = BTreeMap::new();

        let selection_scope = self.selection_collector.scope();

        for (ordinal, ((name, ty), expr)) in contract.arguments.iter().zip(args).enumerate() {
            let start = actual.len();
            let mut argument_layout = ArgumentLayout {
                ty: ty.clone(),
                start,
                len: 0,
                tuples: vec![],
                domains: vec![],
                static_value: None,
            };
            let mut bind = |coordinates: Vec<Value>, expr: Rewritten, ty: Type| {
                let formal = format!("arg_{}", formals.len());
                let path = Path {
                    segments: vec![PathSegment {
                        name: formal.clone(),
                        indices: vec![],
                    }],
                };
                let reference = Expr {
                    kind: ExprKind::Path(path.clone()),
                    span: Span::default(),
                };
                let ordinal = formals.len();
                selectors.insert((name.clone(), coordinates.clone()), (path, ordinal));
                actual.push(expr);
                formals.push((formal, ty));
                (
                    coordinates,
                    ScalarSelection {
                        reference: Reference::Symbol(SymbolIx::from(ordinal)),
                        inspection: reference,
                    },
                )
            };
            match ty {
                Type::Quantity(_) | Type::RefinedQuantity { .. } => {
                    let expr = if let Some(prepared) = prepared {
                        let PreparedArgument::Scalar { value, .. } = &prepared[ordinal] else {
                            return Err(invalid(
                                at,
                                "physical formal requires its actual prepared scalar",
                            ));
                        };
                        value.clone()
                    } else {
                        self.rewrite(instance, expr, env, chain)?
                    };
                    let selected = bind(vec![], expr, ty.clone()).1;
                    lexical.insert(name.clone(), selected.inspection);
                    lexical_references.insert(name.clone(), selected.reference);
                }
                Type::Indexed { element, axes } => {
                    if element.quantity_scheme().is_none() {
                        return Err(invalid(
                            function,
                            "indexed function values must be physical",
                        ));
                    }
                    let values = if let Some(prepared) = prepared {
                        let PreparedArgument::Indexed { value, .. } = &prepared[ordinal] else {
                            return Err(invalid(
                                at,
                                "indexed formal requires its actual prepared group",
                            ));
                        };
                        value.clone()
                    } else {
                        match &expr.kind {
                            ExprKind::Path(path) => self.argument_group(
                                instance,
                                path,
                                env,
                                chain,
                                self.original_expression_source(expr, at)?,
                            )?,
                            ExprKind::NamedCall { name, args } => self.coordinate_group(
                                RewriteContext {
                                    instance,
                                    env,
                                    chain,
                                },
                                at,
                                name,
                                args,
                                self.original_expression_source(expr, at)?,
                            )?,
                            _ => {
                                return Err(invalid(
                                    function,
                                    "indexed argument must name an admitted group or coordinate-map slot",
                                ));
                            }
                        }
                    };
                    argument_layout.domains = values.domains;
                    argument_layout.tuples = values
                        .members
                        .iter()
                        .map(|(tuple, _)| tuple.clone())
                        .collect();
                    let values = values.members;
                    if let Some(external) = &mut contract.external {
                        external.shapes.push(crate::external::ArgumentShape {
                            argument: pse_ids::named_id(function.as_id(), name),
                            axes: axes.iter().map(|axis| axis.as_id()).collect(),
                            coordinates: values
                                .iter()
                                .map(|(v, _)| v.iter().map(Value::identity).collect())
                                .collect(),
                            start,
                        });
                    }

                    let mut flattened = Vec::new();
                    for (coordinates, expr) in values {
                        if coordinates.len() != axes.len() {
                            return Err(invalid(function, "indexed actual arity"));
                        }
                        for (coordinate, axis) in coordinates.iter().zip(axes) {
                            if !matches!(value_type(coordinate), Some(Type::Entity(id) | Type::Enum(id)) if id == *axis)
                            {
                                return Err(invalid(function, "indexed actual kind"));
                            }
                        }
                        flattened.push(bind(coordinates, expr, *element.clone()));
                    }
                    indexed.insert(
                        name.clone(),
                        IndexedSelection {
                            members: flattened.into_iter().collect(),
                            domains: argument_layout.domains.clone(),
                        },
                    );
                }
                Type::Function { .. } => {
                    let selected = if let Some(prepared) = prepared {
                        let Some(Value::Function(selected)) = prepared[ordinal].value() else {
                            return Err(invalid(
                                at,
                                "function formal requires its actual prepared callable",
                            ));
                        };
                        *selected
                    } else {
                        self.resolve_function(
                            instance,
                            at,
                            match &expr.kind {
                                ExprKind::Path(path) => path,
                                _ => {
                                    return Err(invalid(
                                        at,
                                        "function argument requires a checked path",
                                    ));
                                }
                            },
                            env,
                        )?
                    };
                    if let Some(operation) = &mut contract.physical_operation {
                        operation.bind_response_witness(name, selected, self.p, at)?;
                    }
                    let value = Value::Function(selected);
                    self.compatible(&value, ty, at)?;
                    argument_layout.static_value = Some(value.clone());
                    statics.insert(name.clone(), value);
                }
                _ => {
                    let value = if let Some(prepared) = prepared {
                        let value = prepared[ordinal]
                            .value()
                            .ok_or_else(|| invalid(at, "static formal received a runtime scalar"))?
                            .clone();
                        self.compatible(&value, ty, at)?;
                        value
                    } else {
                        self.eval_ast_with(
                            chain.last().copied().unwrap_or(function),
                            env,
                            expr,
                            Some(ty))
                        .map_err(|e| match ty {
                            // ADR-0123 Outcome 2: a `Ref` is static at specialization.
                            Type::Entity(_) => invalid(
                                at,
                                format!(
                                    "Ref argument {name} of {} must be static at specialization: {e}",
                                    self.p
                                        .declarations
                                        .get(&function)
                                        .map_or("a function", |row| row.name.as_str())
                                ),
                            ),
                            _ => e,
                        })?
                    };
                    argument_layout.static_value = Some(value.clone());
                    statics.insert(name.clone(), value);
                }
            }
            argument_layout.len = actual.len() - start;
            argument_layouts.push(argument_layout);
        }
        let selected = wrt
            .iter()
            .map(|path| {
                if path.segments.len() != 1 {
                    return Err(invalid(
                        function,
                        "partial must select an explicit argument",
                    ));
                }
                let segment = &path.segments[0];
                let coordinates = segment
                    .indices
                    .iter()
                    .map(|e| {
                        self.eval_ast_with(chain.last().copied().unwrap_or(function), env, e, None)
                    })
                    .collect::<Result<Vec<_>>>()?;
                selectors
                    .get(&(segment.name.clone(), coordinates))
                    .cloned()
                    .ok_or_else(|| {
                        invalid(
                            function,
                            "partial coordinate is outside the actual argument",
                        )
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        let permissions = self.evidence_permissions(instance)?;
        let selection_receipts = selection_scope.finish();
        let selected_records = selection_receipts
            .values()
            .flat_map(|closure| {
                closure
                    .roots
                    .iter()
                    .chain(std::iter::once(&closure.context))
                    .chain(&closure.records)
                    .chain(closure.edges.keys())
                    .chain(closure.edges.values().flatten())
            })
            .cloned()
            .collect::<Vec<_>>();
        let premises = prelude::PreludePremises::select(
            self.p,
            &prelude::PreludeBindings {
                definition: function,
                root: self.root,
                fixture: matches!(self.reader, crate::provenance::Reader::Fixture),
                arguments: statics.clone(),
                scope: Environment::from([(
                    "fragment.selection-records".into(),
                    Value::Set(selected_records),
                )]),
                inherited: Environment::new(),
                facts: Environment::new(),
                members: self.limits.members,
            },
        )?;
        let input = Arc::new(function_fragments::FunctionFragmentInput {
            package: premises.package,
            selection: premises.selection,
            recipe: Arc::new(function_fragments::FunctionFragmentRecipe {
                function,
                instance,
                contract,
                formals,
                lexical,
                indexed,
                lexical_references,
                permissions,
                substitution,
                statics,
                argument_layouts,
                selections: selection_receipts,
                root: self.root,
                fixture: matches!(self.reader, crate::provenance::Reader::Fixture),
                limits: self.limits,
                function_stack: self.function_stack.clone(),
            }),
        });
        let fragment = if let Some(fragments) = self.fragments {
            fragments.function_fragment(input)?
        } else {
            Arc::new(function_fragments::derive(
                &input,
                None,
                self.cancel,
                self.discretizer,
            )?)
        };
        self.reserve(fragment.construction_items)?;
        for (relationship, _) in fragment.derivations.iter() {
            self.model.derivations.publish(*relationship)?;
        }
        for (occurrence, closure) in &fragment.selections {
            self.selection_collector
                .record(occurrence.clone(), closure.clone());
        }
        self.finite_collector.merge(&fragment.finite)?;
        for (name, inspection) in &fragment.inspections {
            if self
                .model
                .functions
                .get(name)
                .is_some_and(|prior| prior != inspection)
            {
                return Err(invalid(
                    function,
                    "typed fragment inspection name has conflicting original source meaning",
                ));
            }
            self.model
                .functions
                .insert(name.clone(), inspection.clone());
        }
        self.original_guards
            .extend(fragment.original_guards.clone());
        let product = fragment.product.clone();
        let name = fragment.name.clone();
        let function = self.construction.function(product)?;
        let roots = actual.iter().map(|arg| arg.root).collect();
        let args = actual.into_iter().map(|arg| arg.inspection).collect();
        Ok(if selected.is_empty() {
            RewrittenKind {
                node: Node::Apply {
                    function,
                    args: roots,
                },
                inspection: ExprKind::NamedCall {
                    name: Path::single(name),
                    args,
                },
            }
        } else {
            RewrittenKind {
                node: Node::Partial {
                    function,
                    wrt: selected.iter().map(|(_, ordinal)| *ordinal).collect(),
                    args: roots,
                },
                inspection: ExprKind::Partial {
                    function: Path::single(name),
                    wrt: selected.into_iter().map(|(path, _)| path).collect(),
                    args,
                },
            }
        })
    }
    pub(super) fn build_function_fragment(
        &mut self,
        input: &function_fragments::FunctionFragmentInput,
    ) -> Result<(Arc<FunctionProduct>, String)> {
        let function = input.recipe.function;
        let instance = input.recipe.instance;
        let mut contract = input.recipe.contract.clone();
        let formals = input.recipe.formals.clone();
        let lexical = input.recipe.lexical.clone();
        let indexed = input.recipe.indexed.clone();
        let lexical_references = input.recipe.lexical_references.clone();
        let substitution = input.recipe.substitution.clone();
        let statics = input.recipe.statics.clone();
        let argument_layouts = input.recipe.argument_layouts.clone();
        let selection_scope = self.selection_collector.scope();
        for (occurrence, closure) in &input.recipe.selections {
            self.selection_collector
                .record(occurrence.clone(), closure.clone());
        }
        let saved_lexical = std::mem::replace(&mut self.lexical, lexical);
        let saved_indexed = std::mem::replace(&mut self.indexed_arguments, indexed);
        let saved_references = std::mem::replace(&mut self.lexical_references, lexical_references);
        let saved_substitutions = std::mem::replace(&mut self.function_substitutions, substitution);
        let saved_construction =
            std::mem::replace(&mut self.construction, Construction::new(self.limits.items));
        let body_arguments = contract.physical_operation.as_ref().map_or_else(
            || formals.clone(),
            |operation| operation.body_arguments(&formals),
        );
        let mut symbols = Symbols::default();
        let mut formal_slots = Vec::new();
        let mut outputs = Vec::new();
        for (ordinal, (name, ty)) in body_arguments.iter().enumerate() {
            let mut identity = FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1);
            identity.str("local-formal").u64(ordinal as u64);
            let slot = SymbolIx::from(ordinal);
            symbols.push(Symbol {
                id: identity.finish_id(),
                ty: ty.clone(),
                role: Kind::Variable,
                domain: Domain::Continuous,
                definition: None,
                expression: None,
                initial: None,
                lineage: Lineage {
                    declaration: function,
                    instance: SemanticId::NIL.into(),
                    path: name.clone(),
                    demand: vec![],
                    default_owner: None,
                    is_override: false,
                    presets: vec![],
                },
            });
            formal_slots.push(slot);
            outputs.push(self.construction.node(
                Node::Reference(Reference::Symbol(slot)),
                Source {
                    field: crate::expression::occurrences::OccurrenceKey {
                        declaration: function,
                        role: "function.arguments".into(),
                        position: ordinal,
                    },
                    span: Span::default(),
                    coordinate: None,
                },
            )?);
        }
        let saved_symbols = self.function_symbols.replace(symbols);
        let original_scope =
            construction::OriginalScope::function(&contract, self.p, self.limits.items)?;
        self.originals.push(Arc::new(original_scope));
        // Local numbering belongs to the function body, independent of other call sites.
        let saved_serial = std::mem::replace(&mut self.local_serial, 0);
        let saved_types = std::mem::replace(
            &mut self.function_types,
            contract
                .physical_operation
                .as_ref()
                .map_or_else(
                    || contract.arguments.clone(),
                    |operation| operation.body_arguments(&contract.arguments),
                )
                .into_iter()
                .collect(),
        );
        self.function_stack.push(function);
        let body = if let Some(external) = &mut contract.external {
            let output = Evaluator {
                lexical_bases: Some(&self.lexical_bases),
                package: self.p,
                physical: self.c,
                at: function,
                env: &statics,
                limit: self.limits.members,
                stack: vec![],
                reader: self.reader,
                selections: Some(&self.selection_collector),
            }
            .expr(&external.output, Some(&Type::Integer), 0)?;
            let Value::Integer(output) = output else {
                return Err(invalid(
                    function,
                    "external output ordinal requires an exact integer",
                ));
            };
            if !(0..=4095).contains(&output) {
                return Err(invalid(
                    function,
                    "external output ordinal outside capacity",
                ));
            }
            external.output = Expr {
                kind: ExprKind::Number(Number {
                    value: output as f64,
                    exact_integer: Some(i128::from(output)),
                    unit: None,
                }),
                span: Span::default(),
            };
            Ok(None)
        } else {
            contract
                .body
                .as_ref()
                .ok_or_else(|| invalid(function, "function has no selected body"))
                .and_then(|body| self.rewrite(instance, body, &statics, &[function]))
                .map(Some)
        };
        let validity = contract
            .validity
            .as_ref()
            .map(|p| self.rewrite_predicate(instance, p, &statics, &[function]))
            .transpose();
        // What the form layer reads is the lineage of its rejection (Plan 23 H5).
        let validity_reads = contract
            .validity
            .as_ref()
            .map(|p| crate::envelope::Reads::of(self.p, &contract, p, &statics))
            .unwrap_or_default();
        // The data layer: guards specialized in the frame under the consumer's policy
        // (ADR-0123 Outcome 4).
        let guards = self.specialize_guards(instance, function, &contract, &statics);
        let applicability_uses = self.specialize_applicability(instance, &contract, &statics);
        self.function_stack.pop();
        self.originals.pop();
        self.lexical = saved_lexical;
        self.lexical_references = saved_references;
        self.function_substitutions = saved_substitutions;
        self.indexed_arguments = saved_indexed;
        self.local_serial = saved_serial;
        self.function_types = saved_types;
        let mut completed = std::mem::replace(&mut self.construction, saved_construction);
        let symbols = self
            .function_symbols
            .take()
            .ok_or_else(|| invalid(function, "selected function lost its local scalar owner"))?;
        self.function_symbols = saved_symbols;
        let applicability_uses = applicability_uses?;
        let body = body?;
        let validity = validity?;
        let guards = guards?;
        let body_root = body.as_ref().map(|value| value.root);
        if let Some(root) = body_root {
            outputs.push(root);
        }
        let mut validity_roots = validity
            .iter()
            .map(|predicate| predicate.root)
            .collect::<Vec<_>>();
        validity_roots.extend(guards.iter().map(|guard| guard.root));
        let envelopes = guards
            .iter()
            .map(|guard| FunctionEnvelope {
                owner: guard.inspection.envelope.owner,
                ty: guard.inspection.envelope.ty.clone(),
                predicate: guard.root,
                reads: guard.inspection.reads.clone(),
            })
            .collect();
        let mut applicability = Vec::new();
        for usage in &applicability_uses {
            outputs.extend(usage.inputs.iter().map(|input| input.root));
            validity_roots.extend(usage.predicates.iter().map(|predicate| predicate.root));
            applicability.push(EvidenceUse {
                node: usage.node.clone(),
                predicates: usage
                    .predicates
                    .iter()
                    .map(|predicate| predicate.root)
                    .collect(),
                inputs: usage.inputs.iter().map(|input| input.root).collect(),
            });
        }
        let external = contract
            .external
            .as_ref()
            .map(|external| {
                let ExprKind::Number(output) = &external.output.kind else {
                    return Err(invalid(
                        function,
                        "selected external output is not an exact ordinal",
                    ));
                };
                let output = output
                    .integer()
                    .and_then(|value| usize::try_from(value).ok())
                    .filter(|value| *value <= 4095)
                    .ok_or_else(|| {
                        invalid(
                            function,
                            "selected external output ordinal outside capacity",
                        )
                    })?;
                Ok(FunctionExternal {
                    implementation: external.implementation.clone(),
                    revision: external.revision,
                    data: external.data,
                    output,
                    derivative_source: external.derivative_source,
                    derivatives: external.derivatives,
                    smoothness: external.smoothness,
                    shapes: external.shapes.clone(),
                })
            })
            .transpose()?;
        let (admissions, literals) = completed.take_support();
        let physical_operation = contract
            .physical_operation
            .as_ref()
            .map(|operation| {
                crate::resolved::operation::OperationMeaning::capture(
                    operation,
                    crate::resolved::operation::OperationRoots::None,
                    function,
                )
            })
            .transpose()?;
        let syntax = completed.finish(symbols, outputs, validity_roots)?;
        let (physical, output_boundary, body_inputs) =
            crate::resolved::PhysicalSupport::prepare_function(
                &syntax,
                self.c,
                crate::resolved::FunctionSupportRequest {
                    body: body_root,
                    result: &contract.result,
                    literals: &literals,
                    operation: contract.physical_operation.as_ref(),
                    at: function,
                    obligations: &admissions,
                    reduction: None,
                    signature: &formals.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>(),
                    formals: &formal_slots,
                    meaning: physical_operation.as_ref(),
                },
            )?;
        for input in body_inputs.iter().flatten() {
            self.model.derivations.publish(input.relationship)?;
        }
        for relationship in physical.result_views() {
            self.model.derivations.publish(relationship)?;
        }
        let product = FunctionProduct {
            identity: ContentHash::from_bytes([0; 32]),
            has_applicability_effects: false,
            declaration: function,
            syntax,
            physical: Arc::new(physical),
            formals: formal_slots,
            signature: formals.iter().map(|(_, ty)| ty.clone()).collect(),
            arguments: argument_layouts,
            selections: SelectionMeaning::capture(selection_scope.finish(), self.p)?,
            literals,
            result: contract.result.clone(),
            body: body_root,
            validity: validity.as_ref().map(|predicate| predicate.root),
            envelopes,
            applicability,
            prerequisites: contract.prerequisites.clone(),
            admissions,
            reduction_admission: None,
            reduction: None,
            reduction_selection: None,
            physical_operation,
            output_boundary,
            body_inputs,
            validity_reads: validity_reads.clone(),
            external,
            continuity: contract.continuity,
        }
        .seal()?;
        let name = format!(
            "f_{}_inspection_{}_{}",
            product.identity.to_hex(),
            function.as_id(),
            input.recipe.inspection_context().to_hex()
        );
        self.model.functions.insert(
            name.clone(),
            crate::Function {
                applicability: vec![],
                applicability_uses: applicability_uses
                    .into_iter()
                    .map(|usage| usage.inspection())
                    .collect(),
                prerequisites: contract.prerequisites,
                physical_admissions: BTreeMap::new(),
                reduction: None,
                physical_operation: contract.physical_operation,
                validity: validity.map(|predicate| predicate.inspection),
                envelopes: guards.into_iter().map(|guard| guard.inspection).collect(),
                validity_reads,
                external: contract.external,
                continuity: contract.continuity,
                id: contract.id,
                variables: contract.variables,
                arguments: formals,
                result: contract.result,
                body: body.map(|value| value.inspection),
            },
        );
        Ok((product, name))
    }
}

impl SpecializedModel {
    /// Checked source signatures augmented with finite scalar specializations for admission.
    pub fn function_contracts<'a>(
        &'a self,
        package: &'a CheckedPackage,
    ) -> crate::FunctionContracts<'a> {
        crate::FunctionContracts::new(package, &self.functions)
    }
}

impl PartialEq for IndexedSelection {
    fn eq(&self, other: &Self) -> bool {
        self.domains == other.domains
            && self.members.len() == other.members.len()
            && self
                .members
                .iter()
                .zip(&other.members)
                .all(|((left, a), (right, b))| {
                    left == right && a.reference == b.reference && a.inspection == b.inspection
                })
    }
}
