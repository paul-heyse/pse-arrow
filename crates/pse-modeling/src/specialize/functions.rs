// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite function arguments become explicit scalar formals before library differentiation.
use super::*;

impl Engine<'_, '_> {
    /// Finish the checker product after every finite function, transfer and reference
    /// wrapper has been synthesized. Rewritten syntax and actual scalar formals replace
    /// generic source obligations; no stale source binder is carried into lowering.
    pub(super) fn admit_function_occurrences(&mut self) -> Result<()> {
        let contracts = self.model.function_contracts(self.p);
        for function in self.model.functions.values_mut() {
            // Specialization replaces the consumed lexical reduced-law formal with a
            // direct call. Give that call the same operation-owned numerical result
            // view as the original formal, only in this reconstruction's body.
            let body_contracts = function.physical_operation.as_ref().and_then(|operation| {
                if !matches!(operation, crate::PhysicalOperation::Reconstruction { .. }) {
                    return None;
                }
                let mut body_contracts = contracts.clone();
                for called in body_contracts.functions.values_mut() {
                    let numerical = operation.body_arguments(&[(
                        String::new(),
                        Type::Function {
                            arguments: called.arguments.clone(),
                            result: Box::new(called.result.clone()),
                        },
                    )]);
                    if let Some((_, Type::Function { result, .. })) = numerical.into_iter().next() {
                        called.result = *result;
                    }
                }
                Some(body_contracts)
            });
            let contracts = body_contracts.as_ref().unwrap_or(&contracts);
            let recorder = crate::expression::admission::AdmissionRecorder::default();
            let context = TypeContext {
                admissions: Some(&recorder),
                formula_authority: matches!(
                    function.physical_operation,
                    Some(crate::PhysicalOperation::Response { .. })
                )
                .then(|| pse_quantity::PhysicalFormulaAuthority::response(function.id.as_id())),
                quantities: self.c.quantities,
                preconditions: self.c.preconditions,
                scope: self.c.scope,
            };
            let arguments = function
                .physical_operation
                .as_ref()
                .map_or_else(
                    || function.arguments.clone(),
                    |operation| operation.body_arguments(&function.arguments),
                )
                .into_iter()
                .collect();
            if let Some(body) = &function.body {
                let actual = crate::expression::infer(
                    body,
                    &arguments,
                    contracts,
                    &context,
                    function.id,
                    if function.physical_operation.is_some() {
                        None
                    } else {
                        Some(&function.result)
                    },
                )?;
                if let Some(operation) = &function.physical_operation {
                    operation.admit_result(&actual, &function.result, &context, function.id)?;
                }
            }
            for predicate in function
                .validity
                .iter()
                .chain(function.envelopes.iter().map(|guard| &guard.predicate))
                .chain(
                    function
                        .applicability_uses
                        .iter()
                        .flat_map(|usage| usage.predicates.iter()),
                )
            {
                crate::expression::predicate(
                    predicate,
                    &arguments,
                    contracts,
                    &context,
                    function.id,
                )?;
            }
            for usage in &function.applicability_uses {
                for input in &usage.inputs {
                    crate::expression::infer(
                        input,
                        &arguments,
                        contracts,
                        &context,
                        function.id,
                        None,
                    )?;
                }
            }
            let formula_authority = context.formula_authority.clone();
            let mut admissions = recorder
                .into_inner()
                .remove(&function.id)
                .unwrap_or_default();
            if let Some(reduction) = &function.reduction {
                let request = pse_quantity::infer::OpRequest::FiniteReduce {
                    kind: reduction.kind,
                    domain: reduction.domain,
                };
                let resolved = crate::expression::admission::admit_operation(
                    &request,
                    std::slice::from_ref(&reduction.prototype),
                    self.c.quantities,
                    self.c.preconditions,
                    formula_authority.as_ref(),
                    function.id,
                )?;
                admissions.insert(
                    crate::expression::admission::ExpressionOccurrence::finite_reduction(
                        function.id,
                    ),
                    crate::expression::admission::PhysicalAdmission::checked(
                        &request,
                        vec![pse_quantity::scheme::Scheme::from_contract(
                            reduction.prototype.clone(),
                        )],
                        Some(resolved),
                        function.id,
                    )?,
                );
            }
            function.physical_admissions = admissions;
        }
        Ok(())
    }
    pub(super) fn resolve_function(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        name: &str,
        env: &Environment,
    ) -> Result<DeclarationId> {
        self.resolve_function_binding(instance, at, name, env, &mut BTreeSet::new())
    }
    fn resolve_function_binding(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        name: &str,
        env: &Environment,
        stack: &mut BTreeSet<(InstanceId, DeclarationId, Vec<SemanticId>)>,
    ) -> Result<DeclarationId> {
        if let Some(Value::Function(id)) = env.get(name) {
            return Ok(*id);
        }
        if self.p.functions.contains_key(&at) {
            return match self.eval(at, env, name, None)? {
                Value::Function(id) => Ok(id),
                _ => Err(invalid(
                    at,
                    "function definition scope requires a pure function",
                )),
            };
        }
        let expression = dsl::parse_expr(name).map_err(|error| invalid(at, error.to_string()))?;
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
                let source = row
                    .value
                    .binding
                    .as_ref()
                    .and_then(|binding| binding.expression.as_deref())
                    .ok_or_else(|| {
                        invalid(member, "function parameter requires an implementation")
                    })?;
                let local = coordinates_env(&self.states[&owner].env, &coordinates);
                let result = self.resolve_function_binding(owner, member, source, &local, stack)?;
                stack.remove(&key);
                self.compatible(&Value::Function(result), &self.p.types[&member], member)?;
                return Ok(result);
            }
        }
        if let Some(id) = self.states[&instance]
            .members
            .get(name)
            .copied()
            .or_else(|| self.p.resolve(at, name))
            .filter(|id| self.p.functions.contains_key(id))
        {
            return Ok(id);
        }
        if let Value::Function(id) = self
            .eval(at, env, name, None)
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
    ) -> Result<BTreeMap<String, Type>> {
        let mut types = self.p.named_types(at);
        let mut owners = Vec::new();
        let mut owner = Some(at);
        while let Some(id) = owner {
            owners.push(id);
            owner = self.p.declarations[&id].parent_id;
        }
        let mut variables = BTreeSet::new();
        for id in owners.into_iter().rev() {
            if let Some(scope) = &self.p.declarations[&id].value.scope {
                variables.extend(scope.type_parameters.iter().cloned());
                for arg in &scope.parameters {
                    types.insert(
                        arg.name.clone(),
                        self.c.resolve(&arg.r#type, &variables, &types, at)?,
                    );
                }
            }
        }
        if let Some(function) = self.p.functions.get(&at) {
            types.extend(function.arguments.iter().cloned());
        }
        for (name, value) in env {
            if let Some(ty) = value_type(value) {
                types.insert(name.clone(), ty);
            }
        }
        types.extend(self.function_types.clone());
        Ok(types)
    }

    pub(super) fn constant(&mut self, value: f64, ty: &Type, at: DeclarationId) -> Result<Expr> {
        let Type::Quantity(scheme) = ty else {
            return Err(invalid(at, "physical constant type required"));
        };
        let quantity = scheme
            .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
            .map_err(|e| invalid(at, e.to_string()))?;
        let unit = self
            .c
            .quantities
            .quantity_type(quantity)
            .and_then(|q| self.c.quantities.unit_product(q.canonical_unit))
            .map_err(|e| invalid(at, e.to_string()))?;
        let body = Expr {
            kind: ExprKind::Number(Number {
                value,
                exact_integer: None,
                unit: Some(unit),
            }),
            span: Span::default(),
        };
        let mut h = FramedHasher::new(pse_ids::Frame::ModelingTypedConstantV1);
        h.id(&quantity.as_id()).u64(value.to_bits());
        // A lowered function is a synthesized function declaration.
        let id = DeclarationId::from(h.finish_id());
        let name = format!("f_{}", id.as_id().to_hex());
        self.model
            .functions
            .entry(name.clone())
            .or_insert(crate::Function {
                applicability: Vec::new(),
                applicability_uses: Vec::new(),
                physical_admissions: BTreeMap::new(),
                reduction: None,
                physical_operation: None,
                validity: None,
                envelopes: Vec::new(),
                validity_reads: crate::envelope::Reads::default(),
                external: None,
                continuity: None,
                id,
                variables: BTreeSet::new(),
                arguments: vec![],
                result: Type::Quantity(pse_quantity::scheme::Scheme::Concrete(quantity)),
                body: Some(body),
            });
        Ok(Expr {
            kind: ExprKind::NamedCall { name, args: vec![] },
            span: Span::default(),
        })
    }
    /// Resolve an indexed actual without inventing coordinates from observed values.
    pub(super) fn argument_group(
        &mut self,
        instance: InstanceId,
        path: &Path,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Vec<(Vec<Value>, Expr)>> {
        let (first, rest) = path
            .segments
            .split_first()
            .ok_or_else(|| invalid(instance, "empty indexed argument"))?;
        if rest.is_empty()
            && first.indices.is_empty()
            && let Some(values) = self.indexed_arguments.get(&first.name)
        {
            return Ok(values.clone());
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
            );
        }
        let state = &self.states[&instance];
        if !state.members.contains_key(&first.name)
            && !state.children.keys().any(|(name, _)| name == &first.name)
            && self.p.declarations[&state.definition].value.kind == Kind::Implicit
            && let Some(parent) = state.parent
        {
            return self.argument_group(parent, path, env, chain);
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
            member,
            &state.env,
            binding
                .indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str())),
        )?;
        coordinates
            .into_iter()
            .map(|coordinates| {
                let symbol = self.symbol(instance, member, &coordinates, chain)?;
                Ok((
                    coordinates.into_iter().map(|(_, value)| value).collect(),
                    symbol_expr(symbol),
                ))
            })
            .collect()
    }

    pub(super) fn function_call(
        &mut self,
        instance: InstanceId,
        function: DeclarationId,
        args: &[Expr],
        wrt: &[Path],
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<ExprKind> {
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
        let types = self.source_types(at, env)?;
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
        for ((_, formal), expr) in contract.arguments.iter().zip(args) {
            let actual = crate::expression::infer(expr, &types, self.p, self.c, at, Some(formal))?;
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
        let mut selectors = BTreeMap::new();
        let mut identity = FramedHasher::new(pse_ids::Frame::ModelingFiniteFunctionV8);
        identity.id(&function.as_id());
        let selection_scope = self.selection_collector.scope();
        for quantity in substitution.values() {
            quantity.frame(&mut identity);
        }
        for ((name, ty), expr) in contract.arguments.iter().zip(args) {
            let start = actual.len();
            let mut bind = |coordinates: Vec<Value>, expr: Expr, ty: Type| {
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
                selectors.insert((name.clone(), coordinates.clone()), path);
                actual.push(expr);
                formals.push((formal, ty));
                (coordinates, reference)
            };
            match ty {
                Type::Quantity(_) | Type::RefinedQuantity { .. } => {
                    let expr = self.rewrite(instance, expr, env, chain)?;
                    lexical.insert(name.clone(), bind(vec![], expr, ty.clone()).1);
                }
                Type::Indexed { element, axes } => {
                    if element.quantity_scheme().is_none() {
                        return Err(invalid(
                            function,
                            "indexed function values must be physical",
                        ));
                    }
                    let values = match &expr.kind {
                        ExprKind::Path(path) => self.argument_group(instance, path, env, chain)?,
                        ExprKind::NamedCall { name, args } => {
                            self.coordinate_group(instance, at, name, args, env, chain)?
                        }
                        _ => {
                            return Err(invalid(
                                function,
                                "indexed argument must name an admitted group or coordinate-map slot",
                            ));
                        }
                    };
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
                    identity.u64(values.len() as u64);
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
                            coordinate.frame(&mut identity);
                        }
                        flattened.push(bind(coordinates, expr, *element.clone()));
                    }
                    indexed.insert(name.clone(), flattened);
                }
                Type::Function { .. } => {
                    let selected =
                        self.resolve_function(instance, at, &dsl::render_expr(expr), env)?;
                    if let Some(operation) = &mut contract.physical_operation {
                        operation.bind_response_witness(name, selected, self.p, at)?;
                    }
                    let value = Value::Function(selected);
                    self.compatible(&value, ty, at)?;
                    value.frame(&mut identity);
                    statics.insert(name.clone(), value);
                }
                _ => {
                    let value = self
                        .eval(
                            chain.last().copied().unwrap_or(function),
                            env,
                            &dsl::render_expr(expr),
                            Some(ty),
                        )
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
                        })?;
                    value.frame(&mut identity);
                    statics.insert(name.clone(), value);
                }
            }
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
                        self.eval(
                            chain.last().copied().unwrap_or(function),
                            env,
                            &dsl::render_expr(e),
                            None,
                        )
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
        let saved_lexical = std::mem::replace(&mut self.lexical, lexical);
        let saved_indexed = std::mem::replace(&mut self.indexed_arguments, indexed);
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
                package: self.p,
                physical: self.c,
                at: function,
                env: &statics,
                limit: self.limits.members,
                stack: vec![],
                reader: self.reader,
                selections: Some(&self.selection_collector),
            }
            .text(&dsl::render_expr(&external.output), Some(&Type::Integer))?;
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
            identity
                .str(&external.implementation)
                .hash(&external.revision)
                .hash(&external.data)
                .u64(output as u64)
                .str(external.derivative_source.as_str())
                .u64(u64::from(external.derivatives))
                .u64(u64::from(external.smoothness));
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
        let guards =
            self.specialize_guards(instance, function, &contract, &statics);
        let applicability_uses = self.specialize_applicability(instance, &contract, &statics);
        self.function_stack.pop();
        self.lexical = saved_lexical;
        self.indexed_arguments = saved_indexed;
        self.local_serial = saved_serial;
        self.function_types = saved_types;
        let applicability_uses = applicability_uses?;
        let mut body = body?;
        let mut validity = validity?;
        let guards = guards?;
        if let Some(p) = &mut validity {
            p.strip_spans();
            identity.str("validity").str(&dsl::render_predicate(p));
            validity_reads.frame(&mut identity);
        }
        identity.u64(guards.len() as u64);
        for guard in &guards {
            identity
                .str("envelope")
                .id(&guard.envelope.owner.as_id())
                .str(&dsl::render_predicate(&guard.predicate));
            guard.reads.frame(&mut identity);
        }
        identity.u64(applicability_uses.len() as u64);
        for usage in &applicability_uses {
            usage.frame(&mut identity);
        }
        if let Some(body) = &mut body {
            body.strip_spans();
            identity.str(&dsl::render_expr(body));
        }
        if let Some(order) = contract.continuity {
            identity.str("piecewise").u64(u64::from(order));
        }
        if let Some(operation) = &contract.physical_operation {
            operation.frame(&mut identity);
        }
        identity
            .str("physical-signature-v1")
            .u64(formals.len() as u64);
        for (name, ty) in &formals {
            identity.str(name);
            if let Some(quantity) = ty.quantity_scheme() {
                quantity.frame(&mut identity);
            }
            identity.bool(ty.physical_refinement().is_some());
            if let Some(refinement) = ty.physical_refinement() {
                refinement.frame(&mut identity);
            }
        }
        if let Some(quantity) = contract.result.quantity_scheme() {
            quantity.frame(&mut identity);
        }
        identity.bool(contract.result.physical_refinement().is_some());
        if let Some(refinement) = contract.result.physical_refinement() {
            refinement.frame(&mut identity);
        }
        let consumed_selections = selection_scope.finish();
        crate::scientific_selection::frame(&consumed_selections, &mut identity);
        let id = identity.finish_id();
        let name = format!("f_{}", id.to_hex());
        let function = crate::Function {
            applicability: Vec::new(),
            applicability_uses,
            physical_admissions: BTreeMap::new(),
            reduction: None,
            physical_operation: contract.physical_operation,
            validity,
            envelopes: guards,
            validity_reads,
            external: contract.external,
            continuity: contract.continuity,
            id: contract.id,
            variables: contract.variables,
            arguments: formals,
            result: contract.result,
            body,
        };
        if let Some(previous) = self.model.functions.get(&name)
            && previous != &function
        {
            return Err(invalid(id, "function specialization identity collision"));
        }
        self.model.functions.insert(name.clone(), function);
        Ok(if selected.is_empty() {
            ExprKind::NamedCall { name, args: actual }
        } else {
            ExprKind::Partial {
                function: name,
                wrt: selected,
                args: actual,
            }
        })
    }
}

impl SpecializedModel {
    /// Checked source signatures augmented with finite scalar specializations for admission.
    pub fn function_contracts(&self, package: &CheckedPackage) -> CheckedPackage {
        let mut package = package.clone();
        for (name, function) in &self.functions {
            if let Some(id) = name
                .strip_prefix("f_")
                .and_then(|id| SemanticId::parse_hex(id).ok())
                .map(DeclarationId::from)
            {
                package.functions.insert(id, function.clone());
                package.lowered_functions.insert(id);
            }
        }
        package
    }
}
