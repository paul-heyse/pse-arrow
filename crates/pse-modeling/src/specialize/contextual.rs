// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Instantiated physical boundary operations; numerical lowering remains ordinary functions.
use super::functions::PreparedArgument;
use super::rewrite::RewriteContext;
use super::*;
use crate::resolved::{Source, construction::Rewritten, operation::OperationRoots};
use crate::{BoundaryRef, PhysicalOperation, PhysicalRefinement, TransferDirection};
use pse_authoring::dsl::{CompareOp, Number};

impl Engine<'_, '_> {
    pub(super) fn validate_contextual_equations(&self) -> Result<()> {
        // Contextual equality consumes the constructed typed relation. Inspection
        // function names and source ASTs have no receiving authority here.
        for row in &self.model.equations {
            let relation = self.model.equation_roots.get(&row.id).ok_or_else(|| {
                invalid(
                    row.lineage.declaration,
                    "contextual equation lacks its constructed relation",
                )
            })?;
            let layout = self.current_layout(&[relation.lhs, relation.rhs], &[])?;
            let mut contextual = layout.symbols.iter().any(|slot| {
                self.model
                    .symbols
                    .at(slot.symbol)
                    .is_some_and(|symbol| symbol.ty.physical_refinement().is_some())
            });
            for function in &layout.functions {
                contextual |= self
                    .construction
                    .function_at(*function)?
                    .product
                    .result
                    .physical_refinement()
                    .is_some();
            }
            if contextual {
                let left = self.constructed_type(relation.lhs)?;
                let right = self.constructed_type_with_context(relation.rhs, Some(&left))?;
                if left != right {
                    return Err(invalid(
                        row.lineage.declaration,
                        "equation contextual owners, coordinates or physical roles differ",
                    ));
                }
            }
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
        rewrite: RewriteContext<'_>,
        at: DeclarationId,
        name: &str,
        args: &[Expr],
        origin: Source,
    ) -> Result<Option<Rewritten>> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
        if let Some(function) = self.p.resolve(at, name)
            && let Some(PhysicalOperation::ReferenceTranslation(descriptor)) = self
                .p
                .functions
                .get(&function)
                .and_then(|function| function.physical_operation.as_ref())
        {
            return self
                .reference_translation_call(
                    RewriteContext {
                        instance,
                        env,
                        chain,
                    },
                    at,
                    function,
                    descriptor.clone(),
                    args,
                    origin,
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
        let actual = self.constructed_type(input.root)?;
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
        self.physical_transfer_function(at, (actual, output), operation, factor, input, origin)
            .map(Some)
    }

    fn reference_translation_call(
        &mut self,
        rewrite: RewriteContext<'_>,
        at: DeclarationId,
        function: DeclarationId,
        mut descriptor: crate::contextual::ReferenceTranslation,
        args: &[Expr],
        origin: Source,
    ) -> Result<Rewritten> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
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
        let package = self.p;
        let contract = &package.functions[&function];
        let types = self.source_types(at, env)?;
        for ((_, formal), actual) in contract.arguments.iter().zip(args) {
            if crate::expression::infer(actual, &types, self.p, self.c, at, Some(formal))?
                != *formal
            {
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
        let weights = self.argument_group(
            instance,
            path,
            env,
            chain,
            self.original_expression_source(&args[1], at)?,
        )?;
        if weights.members.len() != members.len()
            || weights
                .members
                .iter()
                .any(|(tuple, _)| tuple.len() != 1 || !members.contains(&tuple[0]))
        {
            return Err(invalid(
                at,
                "translation composition must cover exactly its actual members",
            ));
        }
        let mut anchor_env = Environment::new();
        let mut conditions = Vec::new();
        let mut states = Vec::new();
        let mut state_types = Vec::new();
        for (position, (name, expression, attribute)) in [
            (
                "translation_temperature",
                &descriptor.temperature,
                "temperature",
            ),
            ("translation_pressure", &descriptor.pressure, "pressure"),
        ]
        .into_iter()
        .enumerate()
        {
            let quantity = self.p.reference_attribute_type(attribute, function)?;
            let ty = Type::Quantity(Scheme::Concrete(quantity));
            let value = self.eval_ast_with(function, &anchor_env, expression, Some(&ty))?;
            let Value::Number { bits, quantity } = value else {
                return Err(invalid(
                    at,
                    "reference conditions must be physical constants",
                ));
            };
            conditions.push(
                Canonical::canonical(self.c.quantities, quantity)
                    .and_then(|plan| plan.apply(f64::from_bits(bits)))
                    .map_err(|error| invalid(at, error.to_string()))?,
            );
            let field = if position == 0 {
                "reference_translation.temperature"
            } else {
                "reference_translation.pressure"
            };
            let source = Source {
                field: crate::expression::occurrences::OccurrenceKey {
                    declaration: function,
                    role: field.into(),
                    position: 0,
                },
                span: expression.span,
                coordinate: None,
            };
            states.push(self.value_expression(&value, at, source)?);
            state_types.push(ty);
            anchor_env.insert(name.into(), value);
        }
        let mut selected_weights = Vec::new();
        let mut anchor_roots = Vec::new();
        for (coordinates, weight) in weights.members {
            self.reserve(1)?;
            let Value::Entity { id: member, kind } = &coordinates[0] else {
                return Err(invalid(at, "composition member must be an admitted entity"));
            };
            if !self.p.refines(*kind, descriptor.component_kind) {
                return Err(invalid(at, "composition member kind differs"));
            }
            let arguments = [
                PreparedArgument::Scalar {
                    value: states[0].clone(),
                    ty: state_types[0].clone(),
                },
                PreparedArgument::Scalar {
                    value: states[1].clone(),
                    ty: state_types[1].clone(),
                },
                PreparedArgument::Static {
                    value: coordinates[0].clone(),
                    ty: Type::Entity(*kind),
                },
            ];
            let mut anchors = Vec::new();
            for anchor in [descriptor.source_anchor, descriptor.target_anchor] {
                anchors.push(self.function_call_prepared(
                    instance,
                    anchor,
                    &arguments,
                    origin.clone(),
                )?);
            }
            anchor_roots.push([anchors[0].root, anchors[1].root]);
            descriptor
                .anchors
                .push(crate::contextual::ReferenceAnchorPair {
                    member: member.as_id(),
                    values: [anchors[0].inspection.clone(), anchors[1].inspection.clone()],
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
        let roots = || OperationRoots::Reference {
            state: [states[0].root, states[1].root],
            anchors: anchor_roots.clone(),
        };
        let source_mean = self.reference_anchor_mean(
            function,
            (&descriptor, true),
            weight_type,
            &selected_weights,
            roots(),
            origin.clone(),
        )?;
        let target_mean = self.reference_anchor_mean(
            function,
            (&descriptor, false),
            weight_type,
            &selected_weights,
            roots(),
            origin.clone(),
        )?;
        let value = self.rewrite(instance, &args[0], env, chain)?;
        let point = Type::Quantity(Scheme::Concrete(descriptor.source));
        let input = Type::Quantity(Scheme::Delta(Box::new(Scheme::Concrete(descriptor.source))));
        let output = Type::Quantity(Scheme::Delta(Box::new(Scheme::Concrete(descriptor.target))));
        let mut generated = origin.clone();
        generated.coordinate = None;
        let delta = self.constructed_function(
            constructed::ConstructedFunction {
                at: function,
                arguments: vec![("value".into(), point.clone()), ("anchor".into(), point)],
                result: input.clone(),
                operation: None,
                actual: vec![value, source_mean],
                source: generated.clone(),
                closed: OperationRoots::None,
            },
            |construction, formals, _, source| {
                Ok((
                    construction.binary(
                        BinaryOp::Sub,
                        formals[0].clone(),
                        formals[1].clone(),
                        source.clone(),
                    )?,
                    None,
                ))
            },
        )?;
        let operation = PhysicalOperation::ReferenceTranslation(descriptor);
        let converted = self.constructed_function(
            constructed::ConstructedFunction {
                at: function,
                arguments: vec![("value".into(), input)],
                result: output,
                operation: Some(operation),
                actual: vec![delta],
                source: generated,
                closed: roots(),
            },
            |_, formals, _, _| Ok((formals[0].clone(), None)),
        )?;
        self.generated_binary(BinaryOp::Add, converted, target_mean, origin)
    }
    fn reference_anchor_mean(
        &mut self,
        at: DeclarationId,
        anchor: (&crate::contextual::ReferenceTranslation, bool),
        weight_type: &Type,
        weights: &[Rewritten],
        roots: OperationRoots,
        origin: Source,
    ) -> Result<Rewritten> {
        let (translation, is_source) = anchor;
        use pse_quantity::scheme::Scheme;
        let mut origin = origin;
        origin.coordinate = None;
        let quantity = if is_source {
            translation.source
        } else {
            translation.target
        };
        let position = usize::from(!is_source);
        let point = Type::Quantity(Scheme::Concrete(quantity));
        let coefficient = self
            .c
            .quantities
            .neutral_dimensionless()
            .ok_or_else(|| invalid(at, "reference mean requires a neutral coefficient"))?;
        let operation = PhysicalOperation::ReferenceAnchorMean {
            translation: translation.clone(),
            source: is_source,
            coefficient,
        };
        let inspections = translation
            .anchors
            .iter()
            .map(|anchor| anchor.values[position].clone())
            .collect::<Vec<_>>();
        self.constructed_function(
            constructed::ConstructedFunction {
                at,
                arguments: (0..weights.len())
                    .map(|ordinal| (format!("weight_{ordinal}"), weight_type.clone()))
                    .collect(),
                result: point,
                operation: Some(operation),
                actual: weights.to_vec(),
                source: origin,
                closed: roots,
            },
            |construction, formals, imported, source| {
                if formals.is_empty() || imported.len() != 2 + 2 * formals.len() {
                    return Err(invalid(
                        at,
                        "reference mean lacks its exact finite anchor roots",
                    ));
                }
                let reference = Rewritten {
                    root: imported[2 + position],
                    inspection: inspections[0].clone(),
                };
                let mut terms = Vec::new();
                for (ordinal, weight) in formals.iter().enumerate() {
                    let anchor = Rewritten {
                        root: imported[2 + 2 * ordinal + position],
                        inspection: inspections[ordinal].clone(),
                    };
                    let delta = construction.binary(
                        BinaryOp::Sub,
                        anchor,
                        reference.clone(),
                        source.clone(),
                    )?;
                    terms.push(construction.binary(
                        BinaryOp::Mul,
                        weight.clone(),
                        delta,
                        source.clone(),
                    )?);
                }
                let total = construction.sum(formals.to_vec(), source.clone())?;
                let sum = construction.sum(terms, source.clone())?;
                let difference =
                    construction.binary(BinaryOp::Div, sum, total.clone(), source.clone())?;
                let body =
                    construction.binary(BinaryOp::Add, reference, difference, source.clone())?;
                let mut guards = Vec::new();
                for value in formals {
                    let zero = construction.number(
                        Number {
                            value: 0.0,
                            exact_integer: Some(0),
                            unit: None,
                        },
                        source.clone(),
                    )?;
                    guards.push(construction.compare(
                        CompareOp::Ge,
                        value.clone(),
                        zero,
                        source.clone(),
                    )?);
                }
                let zero = construction.number(
                    Number {
                        value: 0.0,
                        exact_integer: Some(0),
                        unit: None,
                    },
                    source.clone(),
                )?;
                guards.push(construction.compare(CompareOp::Gt, total, zero, source.clone())?);
                let mut guards = guards.into_iter();
                let mut guard = guards
                    .next()
                    .ok_or_else(|| invalid(at, "reference mean guard absent"))?;
                for next in guards {
                    guard = construction.and(guard, next, source.clone())?;
                }
                Ok((body, Some(guard)))
            },
        )
    }
    pub(super) fn physical_transfer_function(
        &mut self,
        at: DeclarationId,
        signature: (Type, Type),
        operation: PhysicalOperation,
        factor: i8,
        value: Rewritten,
        origin: Source,
    ) -> Result<Rewritten> {
        let (input_type, result) = signature;
        if !matches!(
            operation,
            PhysicalOperation::Transfer { .. } | PhysicalOperation::TransferMagnitude { .. }
        ) {
            return Err(invalid(at, "not a transfer operation"));
        }
        self.constructed_function(
            constructed::ConstructedFunction {
                at,
                arguments: vec![("value".into(), input_type)],
                result,
                operation: Some(operation),
                actual: vec![value],
                source: origin,
                closed: OperationRoots::None,
            },
            |construction, formals, _, source| {
                Ok((
                    if factor < 0 {
                        construction.neg(formals[0].clone(), source.clone())?
                    } else {
                        formals[0].clone()
                    },
                    None,
                ))
            },
        )
    }
}
