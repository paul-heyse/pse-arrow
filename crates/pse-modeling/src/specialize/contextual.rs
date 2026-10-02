// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Instantiated physical boundary operations; numerical lowering remains ordinary functions.
use super::*;
use crate::{BoundaryRef, PhysicalOperation, PhysicalRefinement, TransferDirection};
use pse_authoring::dsl::{CompareOp, Number, Predicate, PredicateKind};

impl Engine<'_, '_> {
    pub(super) fn validate_contextual_equations(&self) -> Result<()> {
        let types = self
            .model
            .symbols
            .iter()
            .map(|(id, symbol)| (symbol_name(*id), symbol.ty.clone()))
            .collect::<BTreeMap<_, _>>();
        let functions = self.model.function_contracts(self.p);
        fn check(
            equation: &Equation,
            types: &BTreeMap<String, Type>,
            functions: &CheckedPackage,
            context: &TypeContext<'_>,
            at: DeclarationId,
            expected: Option<&Type>,
        ) -> Result<()> {
            match &equation.kind {
                EquationKind::Relation { lhs, rhs, .. } => {
                    let mut contextual = false;
                    for expression in [lhs, rhs] {
                        expression.walk(|expr| match &expr.kind {
                            ExprKind::Path(path) => {
                                if types
                                    .get(&dsl::render_path(path))
                                    .is_some_and(|ty| ty.physical_refinement().is_some())
                                {
                                    contextual = true;
                                }
                            }
                            ExprKind::NamedCall { name, .. }
                                if functions
                                    .resolve_segments(at, &name.segments)
                                    .and_then(|id| functions.functions.get(&id))
                                    .is_some_and(|f| f.result.physical_refinement().is_some()) =>
                            {
                                contextual = true;
                            }
                            _ => {}
                        });
                    }
                    if contextual {
                        let left =
                            crate::expression::infer(lhs, types, functions, context, at, expected)?;
                        let right = crate::expression::infer(
                            rhs,
                            types,
                            functions,
                            context,
                            at,
                            Some(&left),
                        )?;
                        if left != right {
                            return Err(invalid(
                                at,
                                "equation contextual owners, coordinates or physical roles differ",
                            ));
                        }
                    }
                }
                EquationKind::Conditional {
                    then, otherwise, ..
                } => {
                    check(then, types, functions, context, at, expected)?;
                    check(otherwise, types, functions, context, at, expected)?;
                }
            }
            Ok(())
        }
        // The synthesized conservation row owns the admitted accumulator type.
        // Its initial zero retains that type even when several meanings use one unit.
        let conservation_types = self
            .model
            .closures
            .values()
            .filter(|closure| closure.mode == Mode::Conservation)
            .map(|closure| (pse_ids::named_id(closure.id, "conservation"), &closure.ty))
            .collect::<BTreeMap<_, _>>();
        for row in &self.model.equations {
            check(
                &row.equation,
                &types,
                &functions,
                self.c,
                row.lineage.declaration,
                conservation_types.get(&row.id).copied(),
            )?;
        }
        Ok(())
    }
    pub(super) fn resolve_boundary(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        expression: &Expr,
        env: &Environment,
    ) -> Result<BoundaryRef> {
        let ExprKind::Path(path) = &expression.kind else {
            return Err(invalid(
                at,
                "a boundary must name its actual owner and coordinates",
            ));
        };
        let (instance, declaration, coordinates) =
            self.resolve_path(instance, at, path, env, true)?;
        if self.p.declarations[&declaration].value.kind != Kind::Boundary {
            return Err(invalid(
                at,
                "a transfer endpoint must name a boundary declaration",
            ));
        }
        Ok(BoundaryRef::Bound {
            instance,
            declaration,
            coordinates: coordinates.into_iter().map(|(_, value)| value).collect(),
        })
    }
    pub(super) fn bind_physical_owner(
        &self,
        ty: &Type,
        instance: InstanceId,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<Type> {
        let Type::RefinedQuantity {
            quantity,
            refinement:
                PhysicalRefinement::Transfer {
                    boundary,
                    direction,
                },
        } = ty
        else {
            return Ok(ty.clone());
        };
        let BoundaryRef::Declared(declaration) = boundary else {
            return Ok(ty.clone());
        };
        if !self.states[&instance]
            .members
            .values()
            .any(|member| member == declaration)
        {
            return Err(invalid(
                at,
                "transfer boundary is not a member of the actual owner",
            ));
        }
        let row = &self.p.declarations[declaration];
        let boundary = row
            .value
            .boundary
            .as_ref()
            .ok_or_else(|| invalid(at, "transfer boundary declaration absent"))?;
        let values = boundary
            .indices
            .iter()
            .map(|index| {
                env.get(&index.name)
                    .cloned()
                    .ok_or_else(|| invalid(at, "transfer lacks an actual boundary coordinate"))
            })
            .collect::<Result<Vec<_>>>()?;
        let coordinates = self.member_coordinates(instance, *declaration, values)?;
        let id = quantity
            .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
            .map_err(|error| invalid(at, error.to_string()))?;
        let physical = self
            .c
            .quantities
            .quantity_type(id)
            .map_err(|error| invalid(at, error.to_string()))?;
        let kind = self
            .c
            .quantities
            .kind(physical.key.kind)
            .map_err(|error| invalid(at, error.to_string()))?;
        if physical.key.reference_state.is_some()
            || !kind.extensive
            || kind.addition_kind != pse_quantity::QuantityAdditionKind::Additive
        {
            return Err(invalid(
                at,
                "a transfer carries a datum-free extensive quantity, not a material reference point",
            ));
        }
        Ok(Type::RefinedQuantity {
            quantity: quantity.clone(),
            refinement: PhysicalRefinement::Transfer {
                boundary: BoundaryRef::Bound {
                    instance,
                    declaration: *declaration,
                    coordinates: coordinates.into_iter().map(|(_, value)| value).collect(),
                },
                direction: *direction,
            },
        })
    }
    pub(super) fn contextual_call(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        name: &str,
        args: &[Expr],
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Option<Expr>> {
        if let Some(function) = self.p.resolve(at, name)
            && let Some(PhysicalOperation::ReferenceTranslation(descriptor)) = self
                .p
                .functions
                .get(&function)
                .and_then(|function| function.physical_operation.as_ref())
        {
            return self
                .reference_translation_call(
                    instance,
                    at,
                    function,
                    descriptor.clone(),
                    args,
                    env,
                    chain,
                )
                .map(Some);
        }
        if !matches!(name, "transfer" | "reorient" | "reflect") {
            return Ok(None);
        }
        let direction = |expression: &Expr| match dsl::render_expr(expression).as_str() {
            "Into" => Ok(TransferDirection::Into),
            "OutOf" => Ok(TransferDirection::OutOf),
            _ => Err(invalid(at, "direction must explicitly name Into or OutOf")),
        };
        let (input, target_direction) = match (name, args) {
            ("transfer", [value, _, orientation]) | ("reflect", [_, value, orientation]) => {
                (value, direction(orientation)?)
            }
            ("reorient", [value, orientation]) => (value, direction(orientation)?),
            _ => {
                return Err(invalid(
                    at,
                    "transfer(value,boundary,direction), reorient(value,direction), or reflect(exchange,value,direction) required",
                ));
            }
        };
        let input = self.rewrite(instance, input, env, chain)?;
        let types = self
            .model
            .symbols
            .iter()
            .map(|(id, symbol)| (symbol_name(*id), symbol.ty.clone()))
            .collect::<BTreeMap<_, _>>();
        let actual = crate::expression::infer(
            &input,
            &types,
            &self.model.function_contracts(self.p),
            self.c,
            at,
            None,
        )?;
        let quantity = actual
            .quantity_scheme()
            .ok_or_else(|| invalid(at, "a physical rate is required"))?
            .clone();
        let source = actual.physical_refinement().cloned();
        let (result, factor, exchange) = match name {
            "transfer" => {
                if source.is_some() {
                    return Err(invalid(
                        at,
                        "an existing transfer must use reorient or reflect",
                    ));
                }
                let id = quantity
                    .resolve_with_evidence(
                        self.c.quantities,
                        &BTreeMap::new(),
                        self.c.preconditions,
                    )
                    .map_err(|error| invalid(at, error.to_string()))?;
                let contract = self
                    .c
                    .quantities
                    .quantity_type(id)
                    .map_err(|error| invalid(at, error.to_string()))?;
                let kind = self
                    .c
                    .quantities
                    .kind(contract.key.kind)
                    .map_err(|error| invalid(at, error.to_string()))?;
                if contract.key.reference_state.is_some()
                    || !kind.extensive
                    || kind.addition_kind != pse_quantity::QuantityAdditionKind::Additive
                {
                    return Err(invalid(
                        at,
                        "a transfer requires a datum-free extensive rate",
                    ));
                }
                let boundary = self.resolve_boundary(instance, at, &args[1], env)?;
                (
                    PhysicalRefinement::Transfer {
                        boundary,
                        direction: target_direction,
                    },
                    1,
                    None,
                )
            }
            "reorient" => {
                let (result, factor) = source
                    .as_ref()
                    .ok_or_else(|| invalid(at, "reorient requires a transfer"))?
                    .reorient(target_direction, at)?;
                (result, factor, None)
            }
            "reflect" => {
                let ExprKind::Path(path) = &args[0].kind else {
                    return Err(invalid(at, "reflection names an exchange declaration"));
                };
                let (owner, declaration, coordinates) =
                    self.resolve_path(instance, at, path, env, true)?;
                let pair = self
                    .model
                    .exchanges
                    .get(&member_id(owner, declaration, &coordinates))
                    .ok_or_else(|| invalid(at, "the selected exchange is not instantiated"))?;
                let source = source
                    .as_ref()
                    .ok_or_else(|| invalid(at, "reflect requires a transfer"))?;
                let PhysicalRefinement::Transfer { boundary, .. } = source else {
                    return Err(invalid(at, "reflect requires a directed transfer"));
                };
                let target = if boundary == &pair.first {
                    &pair.second
                } else {
                    &pair.first
                };
                let (result, factor) = pair.reflect(source, target, target_direction)?;
                (result, factor, Some(pair.declaration))
            }
            _ => return Err(invalid(at, "unknown contextual operation")),
        };
        let output = Type::RefinedQuantity {
            quantity,
            refinement: result.clone(),
        };
        let operation = PhysicalOperation::Transfer {
            source,
            result,
            factor,
            exchange,
        };
        self.physical_transfer_function(at, actual, output, operation, factor, input)
            .map(Some)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the translation descriptor travels with its instance, declaration, call arguments, environment and dependency chain as independent inputs"
    )]
    fn reference_translation_call(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        function: DeclarationId,
        mut descriptor: crate::contextual::ReferenceTranslation,
        args: &[Expr],
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Expr> {
        use pse_quantity::{CanonicalConversionPlan as Canonical, scheme::Scheme};
        if args.len() != 3 {
            return Err(invalid(
                at,
                "reference translation requires value, composition and actual members",
            ));
        }
        self.reader.read(
            self.p,
            function,
            self.p.supplies_test_only(function),
            || crate::provenance::supplied_by(self.p, function),
        )?;
        let contract = &self.p.functions[&function];
        let source_types = self.source_types(at, env)?;
        for ((_, formal), actual) in contract.arguments.iter().zip(args) {
            let actual =
                crate::expression::infer(actual, &source_types, self.p, self.c, at, Some(formal))?;
            if actual != *formal {
                return Err(invalid(at, "reference translation actual contract differs"));
            }
        }
        let Value::Set(members) =
            self.eval_ast_with(at, env, &args[2], Some(&contract.arguments[2].1))?
        else {
            return Err(invalid(
                at,
                "translation requires a finite actual component set",
            ));
        };
        if members.is_empty() || members.len() != members.iter().collect::<BTreeSet<_>>().len() {
            return Err(invalid(
                at,
                "translation components must be nonempty and distinct",
            ));
        }
        let ExprKind::Path(path) = &args[1].kind else {
            return Err(invalid(
                at,
                "translation composition names an actual indexed group",
            ));
        };
        let weights = self.argument_group(instance, path, env, chain)?;
        if weights.len() != members.len()
            || weights.iter().any(|(coordinates, _)| {
                coordinates.len() != 1 || !members.contains(&coordinates[0])
            })
        {
            return Err(invalid(
                at,
                "translation composition must cover exactly its actual members",
            ));
        }
        let mut anchor_env = Environment::new();
        let mut conditions = Vec::new();
        for (name, expression, attribute) in [
            (
                "translation_temperature",
                &descriptor.temperature,
                "temperature",
            ),
            ("translation_pressure", &descriptor.pressure, "pressure"),
        ] {
            let quantity = self.p.reference_attribute_type(attribute, function)?;
            let value = self.eval_ast_with(
                function,
                &anchor_env,
                expression,
                Some(&Type::Quantity(Scheme::Concrete(quantity))),
            )?;
            let Value::Number { bits, quantity } = value else {
                return Err(invalid(
                    at,
                    "reference conditions must be physical constants",
                ));
            };
            let canonical = Canonical::canonical(self.c.quantities, quantity)
                .and_then(|plan| plan.apply(f64::from_bits(bits)))
                .map_err(|error| invalid(at, error.to_string()))?;
            conditions.push(canonical);
            anchor_env.insert(name.into(), value);
        }
        let mut selected_weights = Vec::new();
        for (coordinates, weight) in weights {
            self.reserve(1)?;
            let Value::Entity { id: member, kind } = &coordinates[0] else {
                return Err(invalid(at, "composition member must be an admitted entity"));
            };
            if !self.p.refines(*kind, descriptor.component_kind) {
                return Err(invalid(at, "composition member kind differs"));
            }
            anchor_env.insert("translation_component".into(), coordinates[0].clone());
            let mut anchors = Vec::new();
            let actual = [
                "translation_temperature",
                "translation_pressure",
                "translation_component",
            ]
            .iter()
            .map(|name| Expr {
                kind: ExprKind::Path(Path::single(*name)),
                span: Span::default(),
            })
            .collect::<Vec<_>>();
            for anchor in [descriptor.source_anchor, descriptor.target_anchor] {
                let kind =
                    self.function_call(instance, anchor, &actual, &[], &anchor_env, chain)?;
                anchors.push(Expr {
                    kind,
                    span: Span::default(),
                });
            }
            descriptor
                .anchors
                .push(crate::contextual::ReferenceAnchorPair {
                    member: member.as_id(),
                    values: [anchors[0].clone(), anchors[1].clone()],
                    temperature: conditions[0],
                    pressure: conditions[1],
                });
            selected_weights.push(weight);
        }
        pse_quantity::ReferenceTranslation::admit_context(
            self.c.quantities,
            descriptor.source,
            descriptor.target,
            conditions[0],
            conditions[1],
            &descriptor
                .anchors
                .iter()
                .map(|anchor| anchor.member)
                .collect::<Vec<_>>(),
            &descriptor.provenance,
        )
        .map_err(|error| invalid(at, error.to_string()))?;
        let Type::Indexed {
            element: weight_type,
            ..
        } = &contract.arguments[1].1
        else {
            return Err(invalid(at, "composition contract must be indexed"));
        };
        let source_mean =
            self.reference_anchor_mean(at, &descriptor, true, weight_type, &selected_weights)?;
        let target_mean =
            self.reference_anchor_mean(at, &descriptor, false, weight_type, &selected_weights)?;
        let value = self.rewrite(instance, &args[0], env, chain)?;
        let binary = |op, lhs, rhs| Expr {
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span: Span::default(),
        };
        let input = Type::Quantity(Scheme::Delta(Box::new(Scheme::Concrete(descriptor.source))));
        let output = Type::Quantity(Scheme::Delta(Box::new(Scheme::Concrete(descriptor.target))));
        // Preserve both datum-point operands at the ordinary subtraction boundary.
        // A result expectation for a difference must not retype a rewritten literal
        // point before subtracting its same-datum anchor.
        self.reserve(1)?;
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingPhysicalOperationV1);
        hash.str("reference-source-difference")
            .id(&descriptor.source.as_id());
        let difference_id = DeclarationId::from(hash.finish_id());
        let difference_name = format!("f_{}", difference_id.as_id().to_hex());
        let point = Type::Quantity(Scheme::Concrete(descriptor.source));
        self.model
            .functions
            .entry(difference_name.clone())
            .or_insert(crate::Function {
                applicability: Vec::new(),
                applicability_uses: Vec::new(),
                prerequisites: Vec::new(),
                physical_admissions: BTreeMap::new(),
                physical_operation: None,
                reduction: None,
                validity: None,
                envelopes: Vec::new(),
                validity_reads: crate::envelope::Reads::default(),
                external: None,
                continuity: None,
                id: difference_id,
                variables: BTreeSet::new(),
                arguments: vec![("value".into(), point.clone()), ("anchor".into(), point)],
                result: input.clone(),
                body: Some(binary(
                    BinaryOp::Sub,
                    Expr {
                        kind: ExprKind::Path(Path::single("value")),
                        span: Span::default(),
                    },
                    Expr {
                        kind: ExprKind::Path(Path::single("anchor")),
                        span: Span::default(),
                    },
                )),
            });
        let delta = Expr {
            kind: ExprKind::NamedCall {
                name: Path::single(difference_name),
                args: vec![value, source_mean],
            },
            span: Span::default(),
        };
        let operation = PhysicalOperation::ReferenceTranslation(descriptor);
        let converted = self.contextual_function(
            at,
            vec![("value".into(), input)],
            output,
            operation,
            &Expr {
                kind: ExprKind::Path(Path::single("value")),
                span: Span::default(),
            },
            None,
            vec![delta],
        )?;
        Ok(binary(BinaryOp::Add, converted, target_mean))
    }

    fn reference_anchor_mean(
        &mut self,
        at: DeclarationId,
        translation: &crate::contextual::ReferenceTranslation,
        source: bool,
        weight_type: &Type,
        weights: &[Expr],
    ) -> Result<Expr> {
        use pse_quantity::scheme::Scheme;
        let quantity = if source {
            translation.source
        } else {
            translation.target
        };
        let point = Type::Quantity(Scheme::Concrete(quantity));
        let position = usize::from(!source);
        let reference = translation.anchors[0].values[position].clone();
        let binary = |op, lhs, rhs| Expr {
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span: Span::default(),
        };
        let variable = |index| Expr {
            kind: ExprKind::Path(Path::single(format!("weight_{index}"))),
            span: Span::default(),
        };
        let sum = |values: Vec<Expr>| {
            values
                .into_iter()
                .reduce(|lhs, rhs| binary(BinaryOp::Add, lhs, rhs))
                .ok_or_else(|| invalid(at, "reference mean requires actual weights"))
        };
        let terms = translation.anchors[..weights.len()]
            .iter()
            .enumerate()
            .map(|(index, anchor)| {
                binary(
                    BinaryOp::Mul,
                    variable(index),
                    binary(
                        BinaryOp::Sub,
                        anchor.values[position].clone(),
                        reference.clone(),
                    ),
                )
            })
            .collect();
        let total = sum((0..weights.len()).map(variable).collect())?;
        let body = binary(
            BinaryOp::Add,
            reference,
            binary(BinaryOp::Div, sum(terms)?, total.clone()),
        );
        let zero = || Expr {
            kind: ExprKind::Number(Number {
                value: 0.,
                exact_integer: Some(0),
                unit: None,
            }),
            span: Span::default(),
        };
        let comparison = |op, lhs| Predicate {
            kind: PredicateKind::Compare {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(zero()),
            },
            span: Span::default(),
        };
        let guard = (0..weights.len())
            .map(|index| comparison(CompareOp::Ge, variable(index)))
            .chain(std::iter::once(comparison(CompareOp::Gt, total)))
            .reduce(|lhs, rhs| Predicate {
                kind: PredicateKind::And(Box::new(lhs), Box::new(rhs)),
                span: Span::default(),
            })
            .ok_or_else(|| invalid(at, "reference mean requires actual weights"))?;
        let coefficient = self
            .c
            .quantities
            .neutral_dimensionless()
            .ok_or_else(|| invalid(at, "reference mean requires a neutral coefficient"))?;
        self.contextual_function(
            at,
            (0..weights.len())
                .map(|index| (format!("weight_{index}"), weight_type.clone()))
                .collect(),
            point,
            PhysicalOperation::ReferenceAnchorMean {
                translation: translation.clone(),
                source,
                coefficient,
            },
            &body,
            Some(&guard),
            weights.to_vec(),
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the synthesized function's signature, physical operation, body, guard and actual arguments are independent inputs"
    )]
    fn contextual_function(
        &mut self,
        at: DeclarationId,
        arguments: Vec<(String, Type)>,
        result: Type,
        operation: PhysicalOperation,
        body: &Expr,
        guard: Option<&Predicate>,
        actual: Vec<Expr>,
    ) -> Result<Expr> {
        self.reserve(1)?;
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingPhysicalOperationV1);
        hash.id(&at.as_id());
        operation.frame(&mut hash);
        hash.str(&dsl::render_expr(body)).bool(guard.is_some());
        if let Some(guard) = guard {
            hash.str(&dsl::render_predicate(guard));
        }
        let id = DeclarationId::from(hash.finish_id());
        let name = format!("f_{}", id.as_id().to_hex());
        self.model
            .functions
            .entry(name.clone())
            .or_insert(crate::Function {
                applicability: Vec::new(),
                applicability_uses: Vec::new(),
                prerequisites: Vec::new(),
                physical_admissions: BTreeMap::new(),
                physical_operation: Some(operation),
                reduction: None,
                validity: guard.cloned(),
                envelopes: vec![],
                validity_reads: crate::envelope::Reads::default(),
                external: None,
                continuity: None,
                id,
                variables: BTreeSet::new(),
                arguments,
                result,
                body: Some(body.clone()),
            });
        Ok(Expr {
            kind: ExprKind::NamedCall {
                name: Path::single(name),
                args: actual,
            },
            span: Span::default(),
        })
    }
    pub(super) fn physical_transfer_function(
        &mut self,
        at: DeclarationId,
        input_type: Type,
        result: Type,
        operation: PhysicalOperation,
        factor: i8,
        value: Expr,
    ) -> Result<Expr> {
        let mut h = FramedHasher::new(pse_ids::Frame::ModelingPhysicalOperationV1);
        h.id(&at.as_id()).part(&factor.to_le_bytes());
        for ty in [&input_type, &result] {
            let quantity = ty
                .quantity_scheme()
                .ok_or_else(|| invalid(at, "physical transformation has no numeric payload"))?
                .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
                .map_err(|error| invalid(at, error.to_string()))?;
            h.id(&quantity.as_id())
                .bool(ty.physical_refinement().is_some());
            if let Some(refinement) = ty.physical_refinement() {
                refinement.frame(&mut h);
            }
        }
        match &operation {
            PhysicalOperation::Transfer { exchange, .. } => {
                h.str("transfer").bool(exchange.is_some());
                if let Some(exchange) = exchange {
                    h.id(&exchange.as_id());
                }
            }
            PhysicalOperation::TransferMagnitude { .. } => {
                h.str("contribution");
            }
            _ => return Err(invalid(at, "not a transfer operation")),
        }
        let id = DeclarationId::from(h.finish_id());
        let name = format!("f_{}", id.as_id().to_hex());
        let formal = Expr {
            kind: ExprKind::Path(Path::single("value")),
            span: Span::default(),
        };
        let body = if factor < 0 {
            Expr {
                kind: ExprKind::Neg(Box::new(formal)),
                span: Span::default(),
            }
        } else {
            formal
        };
        self.model
            .functions
            .entry(name.clone())
            .or_insert(crate::Function {
                applicability: Vec::new(),
                applicability_uses: Vec::new(),
                prerequisites: Vec::new(),
                physical_admissions: BTreeMap::new(),
                physical_operation: Some(operation),
                reduction: None,
                validity: None,
                envelopes: vec![],
                validity_reads: crate::envelope::Reads::default(),
                external: None,
                continuity: None,
                id,
                variables: BTreeSet::new(),
                arguments: vec![("value".into(), input_type)],
                result,
                body: Some(body),
            });
        Ok(Expr {
            kind: ExprKind::NamedCall {
                name: Path::single(name),
                args: vec![value],
            },
            span: Span::default(),
        })
    }
}
