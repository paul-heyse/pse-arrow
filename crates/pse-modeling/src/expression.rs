// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source expression checking; executable arithmetic remains in pse-math.
pub mod admission;
pub mod occurrences;

use crate::{
    DeclarationId, Result,
    check::CheckedPackage,
    invalid,
    types::{Type, TypeContext},
};
use pse_authoring::dsl::{self, BinaryOp, Expr, ExprKind, Path, Predicate, PredicateKind};
use pse_quantity::{
    Ratio,
    infer::{Exponent, OpRequest},
    literal::LiteralContext,
    scheme::{Scheme, Substitution},
};
use std::collections::{BTreeMap, BTreeSet};

/// Referenced source paths in deterministic lexical traversal order.
pub fn references(expression: &Expr) -> BTreeSet<String> {
    source_paths(expression)
        .iter()
        .map(|p| dsl::render_path(p))
        .collect()
}
/// Source references exclude intrinsic callees while retaining equally named value paths.
pub(crate) fn source_paths(expression: &Expr) -> Vec<&Path> {
    let mut intrinsics = Vec::new();
    expression.walk(|node| {
        if let ExprKind::NamedCall { name, .. } = &node.kind
            && intrinsic_callee(name)
        {
            intrinsics.push(std::ptr::from_ref(name));
        }
    });
    expression
        .free_paths()
        .into_iter()
        .filter(|path| !intrinsics.iter().any(|callee| std::ptr::eq(*callee, *path)))
        .collect()
}
pub(crate) fn intrinsic_callee(path: &Path) -> bool {
    matches!(
        path.ident(),
        Some(
            "applicability_interval"
                | "selection_closure"
                | "selection_compatible"
                | "set_of"
                | "tuple"
                | "require_present"
                | "keys"
                | "at"
                | "size"
                | "present"
                | "implements"
                | "provides"
                | "union"
                | "product"
                | "reconstruct"
                | "transfer"
                | "reorient"
                | "reflect"
        )
    )
}
/// Visit source paths, including actual argument and index expressions.
pub fn walk(expression: &Expr, visit: &mut impl FnMut(&Path)) {
    match &expression.kind {
        ExprKind::Path(p) => {
            visit(p);
            for s in &p.segments {
                for i in &s.indices {
                    walk(i, visit);
                }
            }
        }
        ExprKind::Neg(e) => walk(e, visit),
        ExprKind::Binary { lhs, rhs, .. } => {
            walk(lhs, visit);
            walk(rhs, visit);
        }
        ExprKind::Call { args, .. }
        | ExprKind::NamedCall { args, .. }
        | ExprKind::Partial { args, .. }
        | ExprKind::Kernel { args, .. } => {
            for e in args {
                walk(e, visit);
            }
        }
        ExprKind::Reduce { body, binder, .. } => {
            walk(body, visit);
            if let Some(p) = &binder.filter {
                walk_predicate(p, visit);
            }
        }
        ExprKind::Fold {
            binder,
            value,
            step,
            ..
        } => {
            walk(value, visit);
            walk(step, visit);
            if let Some(filter) = &binder.filter {
                walk_predicate(filter, visit);
            }
        }
        ExprKind::Derivative { body, .. } => walk(body, visit),
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            walk_predicate(guard, visit);
            walk(then, visit);
            walk(otherwise, visit);
        }
        ExprKind::Let { bindings, body } => {
            for (_, e) in bindings {
                walk(e, visit);
            }
            walk(body, visit);
        }
        ExprKind::Number(_) => {}
    }
}
fn walk_predicate(p: &Predicate, visit: &mut impl FnMut(&Path)) {
    match &p.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            walk(lhs, visit);
            walk(rhs, visit);
        }
        PredicateKind::In { expr, .. } | PredicateKind::Atom(expr) => walk(expr, visit),
        PredicateKind::And(a, b) | PredicateKind::Or(a, b) => {
            walk_predicate(a, visit);
            walk_predicate(b, visit);
        }
        PredicateKind::Not(p) => walk_predicate(p, visit),
        _ => {}
    }
}
fn scheme(ty: &Type, at: DeclarationId) -> Result<Scheme> {
    if let Some(s) = ty.quantity_scheme() {
        Ok(s.clone())
    } else {
        Err(invalid(
            at,
            "physical expression requires quantity operands",
        ))
    }
}
fn with_physical_refinement(ty: Type, refinement: Option<crate::PhysicalRefinement>) -> Type {
    match (ty, refinement) {
        (Type::Quantity(quantity), Some(refinement)) => Type::RefinedQuantity {
            quantity,
            refinement,
        },
        (ty, _) => ty,
    }
}
fn concrete(s: &Scheme, context: &TypeContext<'_>) -> Option<pse_quantity::QuantityTypeId> {
    s.resolve_with_evidence(
        context.quantities,
        &Substitution::new(),
        context.preconditions,
    )
    .ok()
}
fn delta(s: Scheme) -> Scheme {
    match s {
        Scheme::Delta(_) => s,
        _ => Scheme::Delta(Box::new(s)),
    }
}
fn rational(e: &Expr) -> Option<Ratio> {
    match &e.kind {
        ExprKind::Number(n)
            if n.unit.is_none()
                && n.value.fract() == 0.0
                && n.value.abs() <= f64::from(i16::MAX) =>
        {
            Ratio::new(n.value as i32, 1).ok()
        }
        ExprKind::Neg(e) => {
            let v = rational(e)?;
            Ratio::new(-i32::from(v.num()), i32::from(v.den())).ok()
        }
        ExprKind::Binary {
            op: BinaryOp::Div,
            lhs,
            rhs,
        } => {
            let a = rational(lhs)?;
            let b = rational(rhs)?;
            Ratio::new(
                i32::from(a.num()) * i32::from(b.den()),
                i32::from(a.den()) * i32::from(b.num()),
            )
            .ok()
        }
        _ => None,
    }
}
fn physical_op(
    expression: &Expr,
    request: OpRequest<'_>,
    values: &[Scheme],
    context: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<Option<Type>> {
    if values.iter().any(|value| !value.is_closed()) {
        if let Some(recorder) = context.admissions {
            recorder.record(
                expression,
                at,
                admission::PhysicalAdmission::checked(&request, values.to_vec(), None, at)?,
            )?;
        }
        return Ok(None);
    }
    let schemes = values.to_vec();
    let values = values
        .iter()
        .map(|value| {
            value
                .resolve_contract_with_evidence(
                    context.quantities,
                    &Substitution::new(),
                    context.preconditions,
                )
                .map_err(|error| invalid(at, error.to_string()))
        })
        .collect::<Result<Vec<_>>>()?;
    let admitted = pse_quantity::resolved::infer_in_context(
        &request,
        &values,
        None,
        context.quantities,
        context.preconditions,
        context.formula_authority.as_ref(),
    )
    .map_err(|error| invalid(at, error.to_string()))?;
    if let Some(recorder) = context.admissions {
        recorder.record(
            expression,
            at,
            admission::PhysicalAdmission::checked(&request, schemes, Some(admitted.clone()), at)?,
        )?;
    }
    Ok(Some(Type::Quantity(Scheme::from_contract(admitted.result))))
}
/// A product, a quotient or an exact power belongs to a multiplicative chain.
pub(crate) fn chain_operation(op: BinaryOp, rhs: &Expr) -> bool {
    match op {
        BinaryOp::Mul | BinaryOp::Div => true,
        BinaryOp::Pow => rational(rhs).is_some(),
        BinaryOp::Add | BinaryOp::Sub => false,
    }
}
/// Check one expression with actual lexical variables and optional expected physical type.
/// # Errors
/// Undefined references, physical mismatch, invalid arguments or unsupported semantics.
pub fn infer(
    expr: &Expr,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    context: &TypeContext<'_>,
    at: DeclarationId,
    expected: Option<&Type>,
) -> Result<Type> {
    if let Some(recorder) = context.admissions {
        recorder.enter(expr);
    }
    let result = infer_expression(expr, env, p, context, at, expected);
    if let Some(recorder) = context.admissions {
        recorder.leave();
    }
    result
}
fn infer_expression(
    expr: &Expr,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    context: &TypeContext<'_>,
    at: DeclarationId,
    expected: Option<&Type>,
) -> Result<Type> {
    let q = |s| Type::Quantity(s);
    match &expr.kind {
        ExprKind::Number(n) => number_type(n, context, at, expected),
        // Plan 23 D0: `missing` states absence where an optional value is expected.
        ExprKind::Path(_) if is_missing(expr) => match expected {
            Some(Type::Optional(inner)) => Ok(Type::Optional(inner.clone())),
            Some(ty) => Ok(Type::Optional(Box::new(ty.clone()))),
            None => Err(invalid(
                at,
                "missing states absence where an optional value is expected",
            )),
        },
        ExprKind::Path(path) => path_type(path, env, p, context, at),
        ExprKind::Neg(e) => {
            let ty = infer(e, env, p, context, at, expected)?;
            if ty == Type::Integer {
                return Ok(ty);
            }
            let refinement = crate::contextual::arithmetic_refinement(
                &OpRequest::Neg,
                std::slice::from_ref(&ty),
                context,
                at,
            )?;
            let a = scheme(&ty, at)?;
            let result = physical_op(expr, OpRequest::Neg, std::slice::from_ref(&a), context, at)?
                .unwrap_or(q(a));
            Ok(with_physical_refinement(result, refinement))
        }
        ExprKind::Binary { op, lhs, rhs } => {
            // A product, quotient or power does not give either operand its
            // result's physical type. Only additive and exact-integer contexts
            // propagate that expectation, as in typed mathematical lowering.
            let operand_expected = if matches!(op, BinaryOp::Add | BinaryOp::Sub)
                || expected == Some(&Type::Integer)
            {
                expected
            } else {
                None
            };
            let left = infer(lhs, env, p, context, at, operand_expected)?;
            if left == Type::Integer {
                if infer(rhs, env, p, context, at, Some(&Type::Integer))? != Type::Integer {
                    return Err(invalid(at, "integer operand type"));
                }
                return Ok(Type::Integer);
            }
            let a = scheme(&left, at)?;
            let right = infer(
                rhs,
                env,
                p,
                context,
                at,
                if matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                    Some(&left)
                } else {
                    None
                },
            )?;
            let b = scheme(&right, at)?;
            let request = match op {
                BinaryOp::Add => OpRequest::Add,
                BinaryOp::Sub => OpRequest::Sub,
                BinaryOp::Mul => OpRequest::Mul,
                BinaryOp::Div => OpRequest::Div,
                BinaryOp::Pow => OpRequest::Pow {
                    exponent: rational(rhs).map_or(Exponent::Symbolic, Exponent::Rational),
                },
            };
            // The exact exponent is a request fact; its quantity remains a second
            // operand so the authoritative operation checks dimensionlessness.
            let refinement =
                crate::contextual::arithmetic_refinement(&request, &[left, right], context, at)?;
            let operands = vec![a.clone(), b.clone()];
            if let Some(ty) = physical_op(expr, request, &operands, context, at)? {
                return Ok(with_physical_refinement(ty, refinement));
            }
            let neutral = context
                .quantities
                .neutral_dimensionless()
                .map(Scheme::Concrete);
            let result = match op {
                BinaryOp::Sub if a == b => delta(a),
                BinaryOp::Add if a == b => a,
                BinaryOp::Add | BinaryOp::Sub if b == delta(a.clone()) => a,
                BinaryOp::Add if a == delta(b.clone()) => b,
                BinaryOp::Add | BinaryOp::Sub => {
                    return Err(invalid(at, "incompatible polymorphic additive operands"));
                }
                BinaryOp::Mul if Some(&b) == neutral.as_ref() => a,
                BinaryOp::Mul if Some(&a) == neutral.as_ref() => b,
                BinaryOp::Div if Some(&b) == neutral.as_ref() => a,
                // A generic normalization constrains its eventual quotient to the
                // neutral type. Concrete calls still lower the original division
                // through the physical registry; this is not an operation rule.
                BinaryOp::Div if a == b => {
                    neutral.ok_or_else(|| invalid(at, "neutral normalization type absent"))?
                }
                BinaryOp::Mul if a == b => Scheme::Power(
                    Box::new(a),
                    Ratio::new(2, 1).map_err(|e| invalid(at, e.to_string()))?,
                ),
                BinaryOp::Mul => Scheme::Product(Box::new(a), Box::new(b)),
                BinaryOp::Div => Scheme::Quotient(Box::new(a), Box::new(b)),
                BinaryOp::Pow => Scheme::Power(
                    Box::new(a),
                    rational(rhs)
                        .ok_or_else(|| invalid(at, "polymorphic exponent must be exact"))?,
                ),
            };
            Ok(with_physical_refinement(q(result), refinement))
        }
        ExprKind::Call { function, args } => {
            let types = args
                .iter()
                .map(|e| infer(e, env, p, context, at, None))
                .collect::<Result<Vec<_>>>()?;
            let values = types
                .iter()
                .map(|ty| scheme(ty, at))
                .collect::<Result<Vec<_>>>()?;
            let request = match function.as_str() {
                "sqrt" => OpRequest::Sqrt,
                "abs" => OpRequest::Abs,
                "exp" => OpRequest::Transcendental(pse_quantity::Opcode::Exp),
                "log" => OpRequest::Transcendental(pse_quantity::Opcode::Log),
                "sin" => OpRequest::Transcendental(pse_quantity::Opcode::Sin),
                "cos" => OpRequest::Transcendental(pse_quantity::Opcode::Cos),
                _ => {
                    return Err(invalid(
                        at,
                        "nonprimitive function requires a package declaration",
                    ));
                }
            };
            if values.len() != 1 {
                return Err(invalid(at, "primitive function arity"));
            }
            let refinement =
                crate::contextual::arithmetic_refinement(&request, &types, context, at)?;
            if let Some(ty) = physical_op(expr, request, &values, context, at)? {
                return Ok(with_physical_refinement(ty, refinement));
            }
            match (function.as_str(), &values[0]) {
                ("sqrt", Scheme::Power(value, power))
                    if *power == Ratio::new(2, 1).map_err(|e| invalid(at, e.to_string()))? =>
                {
                    Ok(q(*value.clone()))
                }
                ("abs", value) => Ok(with_physical_refinement(q(value.clone()), refinement)),
                _ => Err(invalid(
                    at,
                    "primitive requires a concrete eligible physical type",
                )),
            }
        }
        ExprKind::NamedCall { name, args } if name.is_ident("applicability_interval") => {
            if args.len() != 2 {
                return Err(invalid(
                    at,
                    "applicability_interval requires two endpoint claim applications",
                ));
            }
            for arg in args {
                if infer(arg, env, p, context, at, Some(&Type::Applicability))?
                    != Type::Applicability
                {
                    return Err(invalid(
                        at,
                        "interval coverage requires typed applicability claims",
                    ));
                }
            }
            Ok(Type::Applicability)
        }
        ExprKind::NamedCall { name, args } if name.is_ident("tuple") => {
            let expected_items = match expected {
                Some(Type::Tuple(items)) => Some(items),
                _ => None,
            };
            if expected_items.is_some_and(|items| items.len() != args.len()) {
                return Err(invalid(at, "tuple constructor arity"));
            }
            let mut types = Vec::new();
            for (i, arg) in args.iter().enumerate() {
                let formal = expected_items.map(|items| &items[i]);
                let actual = infer(arg, env, p, context, at, formal)?;
                if let Some(formal) = formal {
                    if !crate::scientific_selection::accepts_type(&actual, formal, p) {
                        return Err(invalid(at, "tuple coordinate type"));
                    }
                    types.push(formal.clone());
                } else {
                    types.push(actual);
                }
            }
            Ok(Type::Tuple(types))
        }
        ExprKind::NamedCall { name, args } if name.is_ident("set_of") => {
            let formal = match expected {
                Some(Type::Set(element)) => Some(element.as_ref()),
                _ => None,
            };
            let mut element = formal.cloned();
            for arg in args {
                let actual = infer(arg, env, p, context, at, element.as_ref())?;
                if let Some(chosen) = &element {
                    if !crate::scientific_selection::accepts_type(&actual, chosen, p) {
                        if crate::scientific_selection::accepts_type(chosen, &actual, p)
                            && formal.is_none()
                        {
                            element = Some(actual);
                        } else {
                            return Err(invalid(at, "set constructor member type"));
                        }
                    }
                } else {
                    element = Some(actual);
                }
            }
            element
                .map(|element| Type::Set(Box::new(element)))
                .ok_or_else(|| invalid(at, "empty set_of requires an expected set type"))
        }
        ExprKind::NamedCall { name, args } if name.is_ident("selection_compatible") => {
            if args.len() != 3 {
                return Err(invalid(at, "selection_compatible arity"));
            }
            let Type::Set(element) = infer(&args[0], env, p, context, at, None)? else {
                return Err(invalid(at, "selection records are a finite set"));
            };
            let scope = infer(&args[1], env, p, context, at, None)?;
            let Type::Function { arguments, result } = infer(&args[2], env, p, context, at, None)?
            else {
                return Err(invalid(at, "selection slot projection function required"));
            };
            if !matches!(element.as_ref(), Type::Entity(_))
                || arguments.len() != 2
                || arguments[0].1 != *element
                || arguments[1].1 != scope
                || !matches!(*result, Type::Tuple(_))
            {
                return Err(invalid(
                    at,
                    "selection slot projection takes the record and context and returns a typed tuple",
                ));
            }
            Ok(Type::Boolean)
        }
        ExprKind::NamedCall { name, args } if name.is_ident("selection_closure") => {
            if args.len() != 3 {
                return Err(invalid(
                    at,
                    "selection_closure requires roots, context and dependencies",
                ));
            }
            let Type::Set(element) = infer(&args[0], env, p, context, at, None)? else {
                return Err(invalid(at, "selection roots are a finite set"));
            };
            if !matches!(element.as_ref(), Type::Entity(_)) {
                return Err(invalid(at, "selection records are entity references"));
            }
            let scope = infer(&args[1], env, p, context, at, None)?;
            let Type::Function { arguments, result } = infer(&args[2], env, p, context, at, None)?
            else {
                return Err(invalid(
                    at,
                    "selection dependencies are an authored function",
                ));
            };
            if arguments.len() != 2
                || arguments[0].1 != *element
                || arguments[1].1 != scope
                || *result != Type::Set(element.clone())
            {
                return Err(invalid(
                    at,
                    "selection dependency function takes the record and context and returns the same record set",
                ));
            }
            Ok(Type::Set(element))
        }
        ExprKind::NamedCall { name, args } if name.is_ident("keys") => {
            if args.len() != 1 {
                return Err(invalid(at, "keys arity"));
            }
            let ExprKind::Path(path) = &args[0].kind else {
                return Err(invalid(at, "keys requires a table declaration"));
            };
            let id = p
                .resolve_segments(at, &path.segments)
                .filter(|_| {
                    path.segments
                        .iter()
                        .all(|segment| segment.indices.is_empty())
                })
                .ok_or_else(|| invalid(at, "unknown table"))?;
            let table = p
                .tables
                .get(&id)
                .ok_or_else(|| invalid(at, "keys requires a table"))?;
            Ok(Type::Set(Box::new(if table.keys.len() == 1 {
                table.keys[0].ty.clone()
            } else {
                Type::Tuple(table.keys.iter().map(|k| k.ty.clone()).collect())
            })))
        }
        ExprKind::NamedCall { name, args }
            if matches!(name.ident().unwrap_or(""), "union" | "product") =>
        {
            if args.len() != 2 {
                return Err(invalid(at, "binary set operation arity"));
            }
            let Type::Set(a) = infer(&args[0], env, p, context, at, None)? else {
                return Err(invalid(at, "set required"));
            };
            let Type::Set(b) = infer(&args[1], env, p, context, at, None)? else {
                return Err(invalid(at, "set required"));
            };
            if name.is_ident("product") {
                Ok(Type::Set(Box::new(Type::Tuple(vec![*a, *b]))))
            } else if a == b {
                Ok(Type::Set(a))
            } else {
                Err(invalid(at, "union element types differ"))
            }
        }
        ExprKind::NamedCall { name, args } if name.is_ident("at") => {
            if args.len() != 2 {
                return Err(invalid(at, "tuple coordinate arity"));
            }
            let Type::Tuple(values) = infer(&args[0], env, p, context, at, None)? else {
                return Err(invalid(at, "tuple required"));
            };
            let ExprKind::Number(n) = &args[1].kind else {
                return Err(invalid(at, "tuple coordinate must be a constant integer"));
            };
            if n.unit.is_some()
                || n.value < 0.
                || n.value.fract() != 0.
                || n.value >= values.len() as f64
            {
                return Err(invalid(at, "tuple coordinate out of bounds"));
            }
            Ok(values[n.value as usize].clone())
        }
        ExprKind::NamedCall { name, args } if name.is_ident("require_present") => {
            if args.len() != 1 {
                return Err(invalid(at, "require_present takes one optional value"));
            }
            let Type::Optional(inner) = infer(&args[0], env, p, context, at, None)? else {
                return Err(invalid(at, "require_present requires an optional value"));
            };
            Ok(*inner)
        }
        ExprKind::NamedCall { name, args } if name.is_ident("present") => {
            if args.len() != 1
                || !matches!(
                    infer(&args[0], env, p, context, at, None)?,
                    Type::Optional(_)
                )
            {
                return Err(invalid(
                    at,
                    format!(
                        "present requires an optional value: {} has {:?}",
                        args.first().map(dsl::render_expr).unwrap_or_default(),
                        args.first()
                            .map(|arg| infer(arg, env, p, context, at, None))
                            .transpose()?
                    ),
                ));
            }
            Ok(Type::Boolean)
        }
        ExprKind::NamedCall { name, args } if name.is_ident("size") => {
            if args.len() != 1
                || !matches!(infer(&args[0], env, p, context, at, None)?, Type::Set(_))
            {
                return Err(invalid(at, "size requires a finite set"));
            }
            Ok(Type::Integer)
        }
        ExprKind::NamedCall { name, args }
            if matches!(name.ident().unwrap_or(""), "implements" | "provides") =>
        {
            if args.len() != 2 {
                return Err(invalid(at, "capability query arity"));
            }
            let target = infer(&args[0], env, p, context, at, None)?;
            if !matches!(target, Type::Definition(_) | Type::Interface(_)) {
                return Err(invalid(at, "capability query requires a definition"));
            }
            let ExprKind::Path(path) = &args[1].kind else {
                return Err(invalid(at, "capability name required"));
            };
            if name.is_ident("implements")
                && !p
                    .resolve_segments(at, &path.segments)
                    .filter(|_| {
                        path.segments
                            .iter()
                            .all(|segment| segment.indices.is_empty())
                    })
                    .is_some_and(|id| matches!(p.types.get(&id), Some(Type::Interface(_))))
            {
                return Err(invalid(at, "interface name required"));
            }
            Ok(Type::Boolean)
        }
        ExprKind::NamedCall { name, args } => call(name, args, &[], env, p, context, at),
        ExprKind::Partial {
            function,
            wrt,
            args,
        } => call(function, args, wrt, env, p, context, at),
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            predicate(guard, env, p, context, at)?;
            let refined = refine(guard, env, p, context, at)?;
            // A branch stating absence makes the conditional optional in the other's type.
            if is_missing(then) || is_missing(otherwise) {
                let inner = match expected {
                    Some(Type::Optional(inner)) => Some(inner.as_ref()),
                    other => other,
                };
                let ty = if is_missing(then) {
                    infer(otherwise, env, p, context, at, inner)?
                } else {
                    infer(then, &refined, p, context, at, inner)?
                };
                return Ok(match ty {
                    Type::Optional(inner) => Type::Optional(inner),
                    other => Type::Optional(Box::new(other)),
                });
            }
            let a = infer(then, &refined, p, context, at, expected)?;
            let b = infer(otherwise, env, p, context, at, Some(&a))?;
            if a != b {
                return Err(invalid(at, "conditional physical branches differ"));
            }
            Ok(a)
        }
        ExprKind::Let { bindings, body } => {
            let mut env = env.clone();
            for (n, e) in bindings {
                let ty = infer(e, &env, p, context, at, None)?;
                env.insert(n.clone(), ty);
            }
            infer(body, &env, p, context, at, expected)
        }
        ExprKind::Reduce { kind, body, binder } => {
            let domain = path_type(&binder.domain, env, p, context, at)?;
            let (Type::Set(element) | Type::Continuous(_, element)) = &domain else {
                return Err(invalid(at, "reduction domain must be a set"));
            };
            let mut env = env.clone();
            env.insert(binder.var.clone(), *element.clone());
            if let Some(filter) = &binder.filter {
                predicate(filter, &env, p, context, at)?;
            }
            let ty = infer(body, &env, p, context, at, None)?;
            if *kind == dsl::ReduceKind::Integral {
                let Type::Continuous(_, coordinate) = domain else {
                    return Err(invalid(at, "integral requires a continuous domain"));
                };
                let (Type::Quantity(value), Type::Quantity(axis)) = (ty, *coordinate) else {
                    return Err(invalid(at, "physical integral required"));
                };
                let axis = Scheme::Delta(Box::new(axis));
                return Ok(physical_op(
                    expr,
                    OpRequest::Mul,
                    &[value.clone(), axis.clone()],
                    context,
                    at,
                )?
                .unwrap_or(Type::Quantity(Scheme::Product(
                    Box::new(value),
                    Box::new(axis),
                ))));
            }
            if let Type::Quantity(s) = &ty {
                let (kind, domain) = finite_reduction(*kind, element, at)?;
                return Ok(physical_op(
                    expr,
                    OpRequest::FiniteReduce { kind, domain },
                    std::slice::from_ref(s),
                    context,
                    at,
                )?
                .unwrap_or(ty));
            }
            Ok(ty)
        }
        ExprKind::Fold {
            accumulator,
            item,
            binder,
            value,
            step,
        } => {
            if accumulator == item || accumulator == &binder.var || item == &binder.var {
                return Err(invalid(at, "fold binders must be distinct"));
            }
            let Type::Set(element) = path_type(&binder.domain, env, p, context, at)? else {
                return Err(invalid(at, "fold requires a finite set"));
            };
            let mut local = env.clone();
            local.insert(binder.var.clone(), *element);
            if let Some(filter) = &binder.filter {
                predicate(filter, &local, p, context, at)?;
            }
            let ty = infer(value, &local, p, context, at, expected)?;
            if !matches!(ty, Type::Quantity(_) | Type::Integer | Type::Set(_)) {
                return Err(invalid(
                    at,
                    "fold requires scalar physical, integer or finite set values",
                ));
            }
            local.insert(accumulator.clone(), ty.clone());
            local.insert(item.clone(), ty.clone());
            if infer(step, &local, p, context, at, Some(&ty))? != ty {
                return Err(invalid(
                    at,
                    "fold step changes the accumulator's complete type",
                ));
            }
            Ok(ty)
        }
        ExprKind::Derivative { body, wrt } => {
            let Type::Quantity(value) = infer(body, env, p, context, at, None)? else {
                return Err(invalid(at, "derivative body must be physical"));
            };
            let Type::Quantity(axis) = path_type(wrt, env, p, context, at)? else {
                return Err(invalid(at, "derivative coordinate must be physical"));
            };
            let value = Scheme::Delta(Box::new(value));
            let axis = Scheme::Delta(Box::new(axis));
            Ok(physical_op(
                expr,
                OpRequest::Div,
                &[value.clone(), axis.clone()],
                context,
                at,
            )?
            .unwrap_or(Type::Quantity(Scheme::Quotient(
                Box::new(value),
                Box::new(axis),
            ))))
        }
        ExprKind::Kernel { .. } => Err(invalid(
            at,
            "external functions require the declared K5 contract",
        )),
    }
}
/// Whether an expression is the literal `missing`, explicit absence (Plan 23 D0).
fn is_missing(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Path(path)
        if path.segments.len() == 1 && path.segments[0].name == "missing" && path.segments[0].indices.is_empty())
}
/// The type of a number literal: an exact integer where one is expected, otherwise the
/// quantity type its unit resolves to in the expected context, or the neutral dimensionless
/// type without a unit. Expressions and data cells read literals through this one rule.
pub(crate) fn number_type(
    n: &dsl::Number,
    context: &TypeContext<'_>,
    at: DeclarationId,
    expected: Option<&Type>,
) -> Result<Type> {
    if n.unit.is_none() && expected == Some(&Type::Integer) && n.value.fract() == 0.0 {
        return Ok(Type::Integer);
    }
    let id = if let Some(unit) = &n.unit {
        // Composed from atomic factors; no composite spelling is looked up whole.
        let unit = context
            .quantities
            .compose(unit)
            .map_err(|e| invalid(at, format!("unit {{{unit}}}: {e}")))?;
        let literal_context = expected
            .and_then(Type::quantity_scheme)
            .and_then(|s| concrete(s, context))
            .map_or(LiteralContext::Free, |quantity_type| {
                LiteralContext::Explicit { quantity_type }
            });
        pse_quantity::literal::resolve_literal(&unit, literal_context, context.quantities)
            .map_err(|e| invalid(at, e.to_string()))?
    } else {
        context
            .quantities
            .neutral_dimensionless()
            .ok_or_else(|| invalid(at, "no neutral physical type"))?
    };
    let ty = Type::Quantity(Scheme::Concrete(id));
    Ok(with_physical_refinement(
        ty,
        expected
            .and_then(Type::physical_refinement)
            .filter(|role| matches!(role, crate::PhysicalRefinement::Transfer { .. }))
            .cloned(),
    ))
}
pub(crate) fn finite_reduction(
    kind: dsl::ReduceKind,
    element: &Type,
    at: DeclarationId,
) -> Result<(
    pse_quantity::ReductionKind,
    Option<pse_quantity::EntityKindId>,
)> {
    let kind = match kind {
        dsl::ReduceKind::Sum => pse_quantity::ReductionKind::Sum,
        dsl::ReduceKind::Prod => pse_quantity::ReductionKind::Prod,
        dsl::ReduceKind::Integral => {
            return Err(invalid(at, "continuous integral is not a finite reduction"));
        }
    };
    let domain = if let Type::Entity(kind) = element {
        Some(pse_quantity::EntityKindId::from_id(kind.as_id()))
    } else {
        None
    };
    Ok((kind, domain))
}
fn call(
    name: &Path,
    args: &[Expr],
    wrt: &[Path],
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    context: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<Type> {
    if name.is_ident("reconstruct") {
        if !wrt.is_empty() {
            return Err(invalid(
                at,
                "differentiate the reconstructed physical function",
            ));
        }
        let (family, arguments) = args.split_first().ok_or_else(|| {
            invalid(
                at,
                "reconstruct requires a declared family and selected law",
            )
        })?;
        let ExprKind::Path(path) = &family.kind else {
            return Err(invalid(
                at,
                "reconstruct requires a declared reconstruction family",
            ));
        };
        let selected = p
            .resolve_segments(at, &path.segments)
            .filter(|id| matches!(p.types.get(id), Some(Type::Reconstruction { .. })))
            .ok_or_else(|| invalid(at, "reconstruct requires a declared reconstruction family"))?;
        return call(
            &p.declaration_path(selected),
            arguments,
            &[],
            env,
            p,
            context,
            at,
        );
    }
    if let Some(ty) =
        crate::contextual::call_type(name.ident().unwrap_or(""), args, env, p, context, at)?
    {
        if !wrt.is_empty() {
            return Err(invalid(
                at,
                "differentiate the composed physical function, not a contextual intrinsic",
            ));
        }
        return Ok(ty);
    }
    let indirect;
    let lexical_formal = p.functions.get(&at).is_some_and(|function| {
        function
            .arguments
            .iter()
            .any(|(argument, _)| Some(argument.as_str()) == name.ident())
    });
    let f = if let Some(function) = (!lexical_formal
        && name
            .segments
            .iter()
            .all(|segment| segment.indices.is_empty()))
    .then(|| p.resolve_segments(at, &name.segments))
    .flatten()
    .and_then(|id| p.functions.get(&id))
    {
        function
    } else {
        let Type::Function { arguments, result } = path_type(name, env, p, context, at)? else {
            return Err(invalid(at, "reference is not a function"));
        };
        indirect = crate::Function {
            applicability: Vec::new(),
            applicability_uses: Vec::new(),
            prerequisites: Vec::new(),
            physical_admissions: BTreeMap::new(),
            reduction: None,
            physical_operation: None,
            validity: None,
            envelopes: Vec::new(),
            validity_reads: crate::envelope::Reads::default(),
            external: None,
            continuity: None,
            id: at,
            variables: BTreeSet::new(),
            arguments,
            result: *result,
            body: None,
        };
        &indirect
    };
    let indexed_slot = p
        .declarations
        .get(&f.id)
        .and_then(|row| row.value.coordinate_slot.as_ref())
        .filter(|slot| {
            !slot.indices.is_empty() && args.len() + slot.indices.len() == f.arguments.len()
        });
    if let Some(slot) = indexed_slot {
        if !wrt.is_empty() {
            return Err(invalid(
                at,
                "an indexed coordinate group is differentiated through its physical reconstruction",
            ));
        }
        for (expr, (_, formal)) in args.iter().zip(&f.arguments) {
            if infer(expr, env, p, context, at, Some(formal))? != *formal {
                return Err(invalid(at, "coordinate map argument contract differs"));
            }
        }
        let axes = f.arguments[args.len()..]
            .iter()
            .map(|(_, ty)| match ty {
                Type::Entity(id) | Type::Enum(id) => Ok(*id),
                _ => Err(invalid(
                    at,
                    "coordinate slot group has invalid coordinate kind",
                )),
            })
            .collect::<Result<Vec<_>>>()?;
        if axes.len() != slot.indices.len() {
            return Err(invalid(at, "coordinate slot group arity"));
        }
        return Ok(Type::Indexed {
            element: Box::new(f.result.clone()),
            axes,
        });
    }
    if args.len() != f.arguments.len() {
        return Err(invalid(
            at,
            format!(
                "function {name} expects {} arguments, received {}",
                f.arguments.len(),
                args.len()
            ),
        ));
    }
    let actual = args
        .iter()
        .zip(&f.arguments)
        .map(|(expr, (_, formal))| infer(expr, env, p, context, at, Some(formal)))
        .collect::<Result<Vec<_>>>()?;
    let normalize = |s: Scheme| -> Result<Scheme> {
        if !s.is_closed() {
            return Ok(s);
        }
        s.resolve_contract_with_evidence(
            context.quantities,
            &Substitution::new(),
            context.preconditions,
        )
        .map(Scheme::from_contract)
        .map_err(|e| invalid(at, e.to_string()))
    };
    fn leaves<'a>(
        formal: &'a Type,
        actual: &'a Type,
        out: &mut Vec<(&'a Type, &'a Type)>,
        at: DeclarationId,
    ) -> Result<()> {
        match (formal, actual) {
            (
                Type::Indexed {
                    element: a,
                    axes: x,
                },
                Type::Indexed {
                    element: b,
                    axes: y,
                },
            ) if x == y => leaves(a, b, out, at),
            (Type::Indexed { .. }, _) => Err(invalid(at, "indexed argument kind or shape differs")),
            _ => {
                out.push((formal, actual));
                Ok(())
            }
        }
    }
    let mut pairs = Vec::new();
    for ((_, formal), actual) in f.arguments.iter().zip(&actual) {
        leaves(formal, actual, &mut pairs, at)?;
    }
    let mut bindings = BTreeMap::<String, Scheme>::new();
    for (formal, actual) in &pairs {
        if let (Type::Quantity(Scheme::Variable(name)), Type::Quantity(actual)) = (formal, actual) {
            let actual = normalize(actual.clone())?;
            if let Some(previous) = bindings.insert(name.clone(), actual.clone())
                && previous != actual
            {
                return Err(invalid(
                    at,
                    "conflicting complete polymorphic argument types",
                ));
            }
        }
    }
    for (formal, actual) in pairs {
        match (formal, actual) {
            (Type::Quantity(formal), Type::Quantity(actual)) => {
                let expected = formal
                    .substitute(&bindings)
                    .map_err(|e| invalid(at, e.to_string()))?;
                if normalize(expected)? != normalize(actual.clone())? {
                    return Err(invalid(
                        at,
                        "function complete physical argument type differs",
                    ));
                }
            }
            // An entity of a refined kind is an argument of every kind it refines.
            (formal, actual) if p.subsumes(formal, actual) => {}
            _ => return Err(invalid(at, "function argument type")),
        }
    }
    if wrt.is_empty() && f.result.quantity_scheme().is_none() {
        return Ok(f.result.clone());
    }
    let mut result = scheme(&f.result, at)?;
    for path in wrt {
        let first = path
            .segments
            .first()
            .ok_or_else(|| invalid(at, "empty differentiation argument"))?;
        let arg = f
            .arguments
            .iter()
            .find(|(name, _)| name == &first.name)
            .ok_or_else(|| invalid(at, "partial must name an explicit formal argument"))?;
        if path.segments.len() != 1 {
            return Err(invalid(at, "partial must select an explicit formal"));
        }
        let ty = match &arg.1 {
            Type::Indexed { element, axes } => {
                if axes.len() != first.indices.len() {
                    return Err(invalid(at, "partial coordinate arity"));
                }
                for (index, kind) in first.indices.iter().zip(axes) {
                    if !coordinate_of(&infer(index, env, p, context, at, None)?, *kind, p) {
                        return Err(invalid(at, "partial coordinate kind"));
                    }
                }
                element.as_ref()
            }
            other if first.indices.is_empty() => other,
            _ => return Err(invalid(at, "scalar partial cannot be indexed")),
        };
        result = Scheme::Quotient(Box::new(delta(result)), Box::new(delta(scheme(ty, at)?)));
    }
    let result = result
        .substitute(&bindings)
        .map_err(|e| invalid(at, e.to_string()))?;
    let result = Type::Quantity(normalize(result)?);
    if wrt.is_empty() {
        Ok(with_physical_refinement(
            result,
            f.result.physical_refinement().cloned(),
        ))
    } else if f.result.physical_refinement().is_some()
        || f.arguments
            .iter()
            .any(|(_, ty)| ty.physical_refinement().is_some())
    {
        Err(invalid(
            at,
            "differentiate the reconstructed physical function rather than nominal reduced coordinates",
        ))
    } else {
        Ok(result)
    }
}
/// Check an admission predicate and complete comparison contracts.
pub(crate) fn predicate(
    v: &Predicate,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    context: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<()> {
    match &v.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            let a = infer(lhs, env, p, context, at, None)?;
            let b = infer(rhs, env, p, context, at, Some(&a))?;
            if a != b {
                return Err(invalid(at, "comparison types differ"));
            }
        }
        PredicateKind::And(a, b) | PredicateKind::Or(a, b) => {
            predicate(a, env, p, context, at)?;
            predicate(b, env, p, context, at)?;
        }
        PredicateKind::Not(a) => predicate(a, env, p, context, at)?,
        PredicateKind::Atom(e) => {
            if infer(e, env, p, context, at, None)? != Type::Boolean {
                return Err(invalid(at, "Boolean predicate required"));
            }
        }
        PredicateKind::In { expr, domain } => {
            let Type::Set(element) = path_type(domain, env, p, context, at)? else {
                return Err(invalid(at, "membership domain must be a set"));
            };
            if !p.subsumes(&element, &infer(expr, env, p, context, at, Some(&element))?) {
                return Err(invalid(at, "membership element type differs"));
            }
        }
        PredicateKind::Bool(_) => {}
        PredicateKind::Null => {
            return Err(invalid(at, "unknown predicate cannot establish admission"));
        }
    }
    Ok(())
}
pub(crate) fn declaration_environment(
    p: &CheckedPackage,
    context: &TypeContext<'_>,
    id: DeclarationId,
) -> Result<BTreeMap<String, Type>> {
    let mut env = p.named_types(id);
    let mut owner = Some(id);
    let mut vars = BTreeSet::new();
    let mut chain = Vec::new();
    while let Some(current) = owner {
        chain.push(current);
        owner = p.declarations[&current].parent_id;
    }
    for current in chain.into_iter().rev() {
        if let Some(scope) = &p.declarations[&current].value.scope {
            vars.extend(scope.type_parameters.iter().cloned());
            for a in &scope.parameters {
                env.insert(a.name.clone(), context.resolve(&a.r#type, &vars, &env, id)?);
            }
        }
        if let Some(map) = &p.declarations[&current].value.coordinate_map {
            for argument in &map.arguments {
                let ty = context.resolve(&argument.r#type, &vars, &env, current)?;
                env.insert(argument.name.clone(), ty);
            }
        }
        if let Some(function) = p.functions.get(&current) {
            env.extend(function.arguments.iter().cloned());
        }
    }
    Ok(env)
}
/// Physical contracts of ADR-0104 declarations; realizations are admitted at specialization.
fn check_forms(
    row: &crate::Declaration,
    id: DeclarationId,
    outer: &BTreeMap<String, Type>,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    context: &TypeContext<'_>,
) -> Result<()> {
    let typed = |role: &str, env: &BTreeMap<String, Type>| -> Result<Type> {
        let e = p.expression_at(id, role, 0)?;
        infer(e, env, p, context, id, None)
    };
    let indicator = || crate::indicator_type(context.quantities, id);
    if let Some(_condition) = row
        .value
        .equation
        .as_ref()
        .and_then(|e| e.condition.as_ref())
        && typed("equation.condition.variable", env)? != indicator()?
    {
        return Err(invalid(
            id,
            "an indicator condition names an indicator variable",
        ));
    }
    if let Some(_set) = &row.value.ordered_set
        && !matches!(typed("ordered_set.member", env)?, Type::Quantity(_))
    {
        return Err(invalid(id, "ordered set members are physical variables"));
    }
    if let Some(_c) = &row.value.cardinality
        && !matches!(typed("cardinality.member", env)?, Type::Quantity(_))
    {
        return Err(invalid(id, "cardinality members are physical variables"));
    }
    if let Some(f) = &row.value.piecewise {
        let [index] = f.indices.as_slice() else {
            return Err(invalid(id, "a piecewise function has one breakpoint index"));
        };
        let mut local = outer.clone();
        let Type::Set(element) = index_domain_type(
            p.static_at(id, "piecewise.indices.domain", 0)?,
            outer,
            p,
            context,
            id,
        )?
        else {
            return Err(invalid(id, "breakpoints range over a finite set"));
        };
        local.insert(index.name.clone(), *element);
        if typed("piecewise.input", outer)? != typed("piecewise.abscissa", &local)?
            || typed("piecewise.output", outer)? != typed("piecewise.ordinate", &local)?
        {
            return Err(invalid(
                id,
                "breakpoints have the input and output physical types",
            ));
        }
    }
    if row.value.logic.is_some() {
        p.proposition_at(id)?;
    }
    if let Some(_c) = &row.value.complementarity
        && ["complementarity.first", "complementarity.second"]
            .into_iter()
            .map(|role| typed(role, env))
            .collect::<Result<Vec<_>>>()?
            .iter()
            .any(|ty| !matches!(ty, Type::Quantity(_)))
    {
        return Err(invalid(id, "complementarity members are physical"));
    }
    Ok(())
}
pub(crate) fn check_all(p: &mut CheckedPackage, context: &TypeContext<'_>) -> Result<()> {
    let recorder = admission::AdmissionRecorder::default();
    let physical_context = TypeContext {
        admissions: Some(&recorder),
        formula_authority: context.formula_authority.clone(),
        quantities: context.quantities,
        preconditions: context.preconditions,
        scope: context.scope,
    };
    check_declarations(p, &physical_context)?;
    p.physical_admissions = recorder.into_inner();
    for (id, function) in &mut p.functions {
        function.physical_admissions = p.physical_admissions.get(id).cloned().unwrap_or_default();
    }
    Ok(())
}
fn check_declarations(p: &CheckedPackage, context: &TypeContext<'_>) -> Result<()> {
    for (id, row) in &p.declarations {
        let mut env = declaration_environment(p, context, *id)?;
        let mut ancestors = Vec::new();
        let mut cursor = row.parent_id;
        while let Some(owner) = cursor {
            ancestors.push(owner);
            cursor = p.declarations[&owner].parent_id;
        }
        for owner in ancestors.into_iter().rev() {
            if let Some(_guard) = &p.declarations[&owner].value.guard {
                let predicate = p.predicate_at(owner, "guard.predicate", 0)?;
                env = refine(predicate, &env, p, context, owner)?;
            }
        }
        let indices = member_indices(row);
        let outer = env.clone();
        for (position, (name, _domain)) in indices.into_iter().enumerate() {
            let (Type::Set(element) | Type::Continuous(_, element)) =
                member_index_type(*id, position, &env, p, context)?
            else {
                return Err(invalid(*id, "indexed member requires a finite set"));
            };
            env.insert(name.clone(), *element);
        }
        check_forms(row, *id, &outer, &env, p, context)?;
        check_process_contracts(row, *id, &env, p, context)?;
        if let Some(_axis) = &row.value.continuous {
            let Type::Continuous(_, target) = &p.types[id] else {
                return Err(invalid(*id, "continuous type absent"));
            };
            for role in ["continuous.lower", "continuous.upper"] {
                let e = p.expression_at(*id, role, 0)?;
                if infer(e, &env, p, context, *id, Some(target))? != **target {
                    return Err(invalid(*id, "continuous bound type differs"));
                }
            }
        }
        if let Some(scope) = &row.value.scope {
            if let Some(_selection) = &scope.selection {
                let score = p.expression_at(*id, "scope.selection.criterion", 0)?;
                let Type::Quantity(quantity) = infer(score, &env, p, context, *id, None)? else {
                    return Err(invalid(
                        *id,
                        "regime score requires a complete physical quantity",
                    ));
                };
                let tolerance = p.expression_at(*id, "scope.selection.tolerance", 0)?;
                let expected = Type::Quantity(Scheme::Delta(Box::new(quantity)));
                if infer(tolerance, &env, p, context, *id, Some(&expected))? != expected {
                    // Concrete types and delta schemes can name the same physical type.
                    let actual = infer(tolerance, &env, p, context, *id, Some(&expected))?;
                    let (Type::Quantity(a), Type::Quantity(b)) = (actual, expected) else {
                        return Err(invalid(*id, "regime tie tolerance units"));
                    };
                    if a.resolve_with_evidence(
                        context.quantities,
                        &BTreeMap::new(),
                        context.preconditions,
                    )
                    .map_err(|e| invalid(*id, e.to_string()))?
                        != b.resolve_with_evidence(
                            context.quantities,
                            &BTreeMap::new(),
                            context.preconditions,
                        )
                        .map_err(|e| invalid(*id, e.to_string()))?
                    {
                        return Err(invalid(*id, "regime tie tolerance units"));
                    }
                }
            }
            if let Some(_eligible) = &scope.eligibility {
                predicate(
                    p.predicate_at(*id, "scope.eligibility", 0)?,
                    &env,
                    p,
                    context,
                    *id,
                )?;
            }
        }
        if let Some(scheme) = &row.value.difference_scheme {
            crate::continuous::Scheme::Difference(scheme).validate(*id)?;
        }
        if let Some(scheme) = &row.value.collocation_scheme {
            crate::continuous::Scheme::Collocation(scheme).validate(*id)?;
        }
        if let Some(grid) = &row.value.discretization {
            let target = p
                .resolve(*id, &grid.target)
                .ok_or_else(|| invalid(*id, "discretization target absent"))?;
            if !matches!(p.types.get(&target), Some(Type::Continuous(..))) {
                return Err(invalid(*id, "discretization target must be continuous"));
            }
            if !matches!(grid.scheme.as_str(), "integrated" | "stationary") {
                crate::continuous::scheme(p, *id, &grid.scheme)?;
            }
            for role in ["discretization.elements", "discretization.order"] {
                let e = p.expression_at(*id, role, 0)?;
                if infer(e, &env, p, context, *id, Some(&Type::Integer))? != Type::Integer {
                    return Err(invalid(*id, "mesh sizes must be exact integers"));
                }
            }
        }
        if let Some(policy) = &row.value.realization {
            use pse_model::generated::enums::ModelingDeclarationKind as K;
            let target = p
                .resolve(*id, &policy.target)
                .ok_or_else(|| invalid(*id, "realization target absent"))?;
            let target = &p.declarations[&target];
            if target.value.kind == K::Implicit {
                crate::specialize::Realization::from_contract(policy, *id)?;
            } else if !(matches!(
                target.value.kind,
                K::Disjunction
                    | K::Sos1
                    | K::Sos2
                    | K::Atmost
                    | K::Atleast
                    | K::Exactly
                    | K::Piecewise
                    | K::Logic
                    | K::Complementarity
            ) || target
                .value
                .equation
                .as_ref()
                .is_some_and(|e| e.condition.is_some()))
            {
                // ADR-0104: forms and disjunctions admit their realizations at specialization.
                return Err(invalid(
                    *id,
                    "realization target must be an implicit block, a constraint form or a disjunction",
                ));
            }
        }
        if let Some(v) = &row.value.relaxation {
            let target = p
                .resolve(*id, &v.target)
                .ok_or_else(|| invalid(*id, "elastic target absent"))?;
            if p.declarations[&target].value.equation.is_none() {
                return Err(invalid(*id, "elastic target must be an equation"));
            }
            let ty = crate::annotation::target_type(
                p,
                context,
                *id,
                p.expression_at(*id, "relaxation.target", 0)?,
                &env,
            )?;
            let e = p.expression_at(*id, "relaxation.nominal", 0)?;
            if infer(e, &env, p, context, *id, Some(&ty))? != ty {
                return Err(invalid(*id, "elastic nominal must have residual units"));
            }
        }
        if let Some(v) = &row.value.continuation {
            let target = p
                .resolve(*id, &v.target)
                .ok_or_else(|| invalid(*id, "continuation target absent"))?;
            if p.declarations[&target].value.kind
                != pse_model::generated::enums::ModelingDeclarationKind::Parameter
            {
                return Err(invalid(
                    *id,
                    "continuation target must be a physical runtime parameter",
                ));
            }
            let ty = crate::annotation::target_type(
                p,
                context,
                *id,
                p.expression_at(*id, "continuation.target", 0)?,
                &env,
            )?;
            if !matches!(ty, Type::Quantity(_)) {
                return Err(invalid(*id, "continuation cannot change structural facts"));
            }
            for role in ["continuation.start", "continuation.end"] {
                let e = p.expression_at(*id, role, 0)?;
                if infer(e, &env, p, context, *id, Some(&ty))? != ty {
                    return Err(invalid(*id, "continuation endpoint physical type differs"));
                }
            }
        }
        if let Some(f) = p.functions.get(id) {
            let physical_context = TypeContext {
                admissions: context.admissions,
                formula_authority: matches!(
                    f.physical_operation,
                    Some(crate::PhysicalOperation::Response { .. })
                )
                .then(|| pse_quantity::PhysicalFormulaAuthority::response(id.as_id())),
                quantities: context.quantities,
                preconditions: context.preconditions,
                scope: context.scope,
            };
            let context = &physical_context;
            env.extend(f.physical_operation.as_ref().map_or_else(
                || f.arguments.clone(),
                |operation| operation.body_arguments(&f.arguments),
            ));
            if let Some(external) = &f.external {
                if f.body.is_some() || f.continuity.is_some() {
                    return Err(invalid(
                        *id,
                        "external function cannot also own an expression body or symbolic continuity claim",
                    ));
                }
                if infer(
                    &external.output,
                    &env,
                    p,
                    context,
                    *id,
                    Some(&Type::Integer),
                )? != Type::Integer
                {
                    return Err(invalid(
                        *id,
                        "external output selector requires an exact integer",
                    ));
                }
            }
            // The form layer's predicate and the data layer's generated guards take one
            // domain-predicate path (ADR-0123 Outcome 4).
            let domain = f
                .validity
                .iter()
                .chain(f.envelopes.iter().map(|guard| &guard.predicate))
                .cloned()
                .reduce(|a, b| Predicate {
                    kind: PredicateKind::And(Box::new(a), Box::new(b)),
                    span: Default::default(),
                });
            if let Some(validity) = &f.validity {
                predicate(validity, &env, p, context, *id)?;
            }
            for guard in &f.envelopes {
                predicate(&guard.predicate, &env, p, context, *id)?;
            }
            let zero = Expr {
                kind: ExprKind::Number(dsl::Number {
                    value: 0.,
                    exact_integer: Some(0),
                    unit: None,
                }),
                span: Default::default(),
            };
            let dependency_expression = if let Some(validity) = &domain {
                Expr {
                    kind: ExprKind::Conditional {
                        guard: Box::new(validity.clone()),
                        then: Box::new(f.body.clone().unwrap_or_else(|| zero.clone())),
                        otherwise: Box::new(zero),
                    },
                    span: Default::default(),
                }
            } else {
                f.body.clone().unwrap_or(zero)
            };
            {
                // Runtime state is explicit. Immutable package data may be referenced
                // through the admitted lexical/import closure, in bodies and domains alike.
                let dependencies = dependency_expression;
                for path in source_paths(&dependencies) {
                    if path.segments.len() == 1
                        && path.segments[0].indices.is_empty()
                        && matches!(path.segments[0].name.as_str(), "missing" | "true" | "false")
                    {
                        continue;
                    }
                    let explicit = path
                        .segments
                        .first()
                        .is_some_and(|s| f.arguments.iter().any(|(name, _)| *name == s.name));
                    let immutable = (1..=path.segments.len())
                        .rev()
                        .find_map(|end| p.resolve_segments(*id, &path.segments[..end]))
                        .is_some_and(|value| {
                            use pse_model::generated::enums::ModelingDeclarationKind as K;
                            let declaration = &p.declarations[&value];
                            (p.functions.contains_key(&value) || matches!(
                                declaration.value.kind,
                                K::Entity
                                    | K::Set
                                    | K::Table
                                    | K::Enum
                                    | K::EntityKind
                                    | K::Constant
                                    | K::Reconstruction
                            )) && declaration.parent_id.is_some_and(|parent| {
                                let owner = &p.declarations[&parent];
                                // Reconstructed bodies call their map's checked slots.
                                // Their immutable authority is the package-owned map.
                                owner.value.kind == K::Package
                                    || (matches!(p.functions.get(&value).and_then(|function| function.physical_operation.as_ref()), Some(crate::PhysicalOperation::Coordinate { map, .. }) if *map == parent)
                                        && owner.parent_id.is_some_and(|package| p.declarations[&package].value.kind == K::Package))
                            })
                        });
                    let physical = (1..=path.segments.len()).any(|end| {
                        let name = path.segments[..end]
                            .iter()
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                            .join(".");
                        p.physical_name(*id, &name).is_some()
                    });
                    if !explicit && !immutable && !physical {
                        return Err(invalid(
                            *id,
                            format!(
                                "pure function requires explicit runtime arguments or immutable package data: {}",
                                dsl::render_path(path)
                            ),
                        ));
                    }
                }
            }
            if let Some(body) = &f.body {
                let actual = infer(
                    body,
                    &env,
                    p,
                    context,
                    *id,
                    if f.physical_operation.is_some() {
                        None
                    } else {
                        Some(&f.result)
                    },
                )?;
                if let Some(operation) = &f.physical_operation {
                    operation.admit_result(&actual, &f.result, context, *id)?;
                } else if actual != f.result {
                    return Err(invalid(
                        *id,
                        format!(
                            "function result differs: expected {:?}, got {:?}",
                            f.result, actual
                        ),
                    ));
                }
            }
        }
        if let Some(b) = &row.value.binding
            && let Some(expected) = p.types.get(id)
            && expected.quantity_scheme().is_some()
        {
            for _source in b.expression.iter() {
                let expr = p.expression_at(*id, "binding.expression", 0)?;
                let actual = infer(expr, &env, p, context, *id, Some(expected))?;
                if &actual != expected {
                    return Err(invalid(
                        *id,
                        format!(
                            "member physical type differs: expected {expected:?}, got {actual:?}"
                        ),
                    ));
                }
            }
        }
        if let Some(a) = &row.value.annotation {
            // Whole-family annotations bind the target's coordinates just like
            // specialization does; their expressions are scalar at each coordinate.
            use crate::annotation::{AnnotationKind as Kind, Shape};
            if a.kind == Kind::EngineeringRule {
                // This package marker names an existing checked constant. It has no
                // expression target; specialization resolves its stable constant identity.
                continue;
            }
            let mut env = env.clone();
            let target_declaration = crate::annotation::target_declaration(
                p,
                context,
                *id,
                p.expression_at(*id, "annotation.target", 0)?,
                &env,
            )?;
            if crate::annotation::shape(a, *id)? == Shape::Connectivity {
                crate::annotation::connectivity_limits(a, *id)?;
                let target = match target_declaration {
                    Some(target) => target,
                    None => indexed_declaration_reference(
                        p.expression_at(*id, "annotation.target", 0)?,
                        &env,
                        p,
                        context,
                        *id,
                    )?,
                };
                use pse_model::generated::enums::ModelingDeclarationKind as K;
                if !matches!(p.declarations[&target].value.kind, K::Port | K::StatePort) {
                    return Err(invalid(*id, "connectivity target must be a declared port"));
                }
                continue;
            }
            if let Some(target) = target_declaration {
                let declaration = &p.declarations[&target];
                let indices = member_indices(declaration);
                let mut target_env = declaration_environment(p, context, target)?;
                for (position, (name, _domain)) in indices.into_iter().enumerate() {
                    let (Type::Set(element) | Type::Continuous(_, element)) =
                        member_index_type(target, position, &target_env, p, context)?
                    else {
                        return Err(invalid(*id, "annotation index domain must be a set"));
                    };
                    target_env.insert(name.clone(), *element.clone());
                    env.insert(name.clone(), *element);
                }
            }
            let target = crate::annotation::target_type(
                p,
                context,
                *id,
                p.expression_at(*id, "annotation.target", 0)?,
                &env,
            )?;
            let shape = crate::annotation::shape(a, *id)?;
            let equation_target = target_declaration
                .is_some_and(|target| p.declarations[&target].value.equation.is_some());
            match a.kind {
                Kind::Scale if !equation_target => {
                    return Err(invalid(*id, "term scaling requires an equation target"));
                }
                Kind::Start | Kind::Bounds | Kind::Valid | Kind::Objective if equation_target => {
                    return Err(invalid(*id, "this annotation requires a value target"));
                }
                _ => {}
            }
            if target.quantity_scheme().is_none() {
                return Err(invalid(*id, "annotation requires a physical member"));
            }
            let numeric = match shape {
                Shape::Connectivity => {
                    crate::annotation::connectivity_limits(a, *id)?;
                    if target_declaration.is_some_and(|target| {
                        p.declarations[&target].value.kind
                            != pse_model::generated::enums::ModelingDeclarationKind::Port
                    }) {
                        return Err(invalid(*id, "connectivity target must be a declared port"));
                    }
                    continue;
                }
                Shape::AccuracyGoal => {
                    let goal = a
                        .accuracy_goal
                        .as_ref()
                        .ok_or_else(|| invalid(*id, "accuracy goal members"))?;
                    let difference = Type::Quantity(Scheme::Concrete(
                        Scheme::Delta(Box::new(
                            target
                                .quantity_scheme()
                                .ok_or_else(|| invalid(*id, "accuracy goal target is physical"))?
                                .clone(),
                        ))
                        .resolve_with_evidence(
                            context.quantities,
                            &Substitution::new(),
                            context.preconditions,
                        )
                        .map_err(|error| invalid(*id, error.to_string()))?,
                    ));
                    for (role, expression, expected) in [
                        ("time", &goal.time, None),
                        ("resolution", &goal.resolution, Some(&difference)),
                        ("criterion_lower", &goal.criterion_lower, Some(&target)),
                        ("criterion_upper", &goal.criterion_upper, Some(&target)),
                    ] {
                        let Some(_expression) = expression else {
                            continue;
                        };
                        let expression =
                            p.expression_at(*id, &format!("annotation.accuracy_goal.{role}"), 0)?;
                        let actual = infer(expression, &env, p, context, *id, expected)?;
                        if actual.quantity_scheme().is_none()
                            || expected.is_some_and(|expected| actual != *expected)
                        {
                            return Err(invalid(
                                *id,
                                "accuracy goal expression has an incompatible physical type",
                            ));
                        }
                    }
                    0
                }
                Shape::EngineeringScale => {
                    let _scale = a
                        .engineering_scale
                        .as_ref()
                        .ok_or_else(|| invalid(*id, "engineering scale members"))?;
                    let difference = Type::Quantity(Scheme::Concrete(
                        Scheme::Delta(Box::new(
                            target
                                .quantity_scheme()
                                .ok_or_else(|| {
                                    invalid(*id, "engineering scale target is physical")
                                })?
                                .clone(),
                        ))
                        .resolve_with_evidence(
                            context.quantities,
                            &Substitution::new(),
                            context.preconditions,
                        )
                        .map_err(|error| invalid(*id, error.to_string()))?,
                    ));
                    let expression =
                        p.expression_at(*id, "annotation.engineering_scale.value", 0)?;
                    if infer(expression, &env, p, context, *id, Some(&difference))? != difference {
                        return Err(invalid(
                            *id,
                            "engineering scale differs from target magnitude type",
                        ));
                    }
                    0
                }
                Shape::EngineeringDefault | Shape::EngineeringRule => 0,
                Shape::Objective => {
                    objective_members(a, &target, &env, p, context, *id)?;
                    continue;
                }
                Shape::Expressions(count) => count,
                Shape::Label => {
                    crate::annotation::label(p.static_at(*id, "annotation.arguments", 0)?, *id)?;
                    0
                }
                Shape::Scheme => 0,
                Shape::Predicate => {
                    predicate(
                        p.predicate_at(*id, "annotation.arguments", 0)?,
                        &env,
                        p,
                        context,
                        *id,
                    )?;
                    0
                }
            };
            for position in 0..numeric {
                let expression = p.expression_at(*id, "annotation.arguments", position)?;
                if infer(expression, &env, p, context, *id, Some(&target))? != target {
                    return Err(invalid(*id, "annotation type differs from target"));
                }
            }
        }
        if let Some(expectation) = &row.value.expectation {
            let actual = p.expression_at(*id, "expectation.actual", 0)?;
            let expected = p.expression_at(*id, "expectation.expected", 0)?;
            let actual = infer(actual, &env, p, context, *id, None)?;
            if infer(expected, &env, p, context, *id, Some(&actual))? != actual {
                return Err(invalid(*id, "test expectation types differ"));
            }
            let tolerance = p.expression_at(*id, "expectation.tolerance", 0)?;
            let delta = Scheme::Delta(Box::new(
                actual
                    .quantity_scheme()
                    .ok_or_else(|| invalid(*id, "test expectation requires quantities"))?
                    .clone(),
            ));
            let delta = Type::Quantity(Scheme::Concrete(
                delta
                    .resolve_with_evidence(
                        context.quantities,
                        &Substitution::new(),
                        context.preconditions,
                    )
                    .map_err(|e| invalid(*id, e.to_string()))?,
            ));
            if infer(tolerance, &env, p, context, *id, Some(&delta))? != delta {
                return Err(invalid(*id, "expectation tolerance type differs"));
            }
            if let Some(_relative) = &expectation.relative_tolerance {
                let relative = p.expression_at(*id, "expectation.relative_tolerance", 0)?;
                let scalar = Type::Quantity(Scheme::Concrete(
                    context.quantities.neutral_dimensionless().ok_or_else(|| {
                        invalid(*id, "relative tolerance requires a neutral scalar type")
                    })?,
                ));
                if infer(relative, &env, p, context, *id, Some(&scalar))? != scalar {
                    return Err(invalid(*id, "relative tolerance must be dimensionless"));
                }
            }
        }
        let check_eq = |role: &str| -> Result<()> {
            let e = p.equation_at(*id, role, 0)?;
            fn eq(
                e: &dsl::Equation,
                env: &BTreeMap<String, Type>,
                p: &CheckedPackage,
                c: &TypeContext<'_>,
                id: DeclarationId,
            ) -> Result<()> {
                match &e.kind {
                    dsl::EquationKind::Relation { lhs, rhs, .. } => {
                        let a = infer(lhs, env, p, c, id, None)?;
                        let b = infer(rhs, env, p, c, id, Some(&a))?;
                        if a != b {
                            return Err(invalid(id, "equation side physical types differ"));
                        }
                    }
                    dsl::EquationKind::Conditional {
                        guard,
                        then,
                        otherwise,
                    } => {
                        predicate(guard, env, p, c, id)?;
                        eq(then, env, p, c, id)?;
                        eq(otherwise, env, p, c, id)?;
                    }
                }
                Ok(())
            }
            eq(e, &env, p, context, *id)
        };
        if let Some(_e) = &row.value.equation {
            check_eq("equation.expression")?;
        }
        if let Some(_e) = row
            .value
            .binding
            .as_ref()
            .and_then(|b| b.defined_by.as_ref())
        {
            check_eq("binding.defined_by")?;
        }
        // A kind's requirement binds its entity and attributes; entity admission types it
        // (Plan 23 D0).
        let kind_requirement = row
            .parent_id
            .is_some_and(|parent| p.kinds.contains_key(&parent));
        for (role, present) in [
            ("guard.predicate", row.value.guard.is_some()),
            (
                "requirement.predicate",
                row.value.requirement.is_some() && !kind_requirement,
            ),
        ] {
            if present {
                predicate(p.predicate_at(*id, role, 0)?, &env, p, context, *id)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn member_indices(row: &crate::Declaration) -> Vec<(&String, &String)> {
    macro_rules! indices {
        ($field:ident) => {
            if let Some(value) = &row.value.$field {
                return value.indices.iter().map(|i| (&i.name, &i.domain)).collect();
            }
        };
    }
    indices!(binding);
    indices!(equation);
    indices!(accumulator);
    indices!(contribution);
    indices!(boundary);
    indices!(exchange);
    indices!(coordinate_slot);
    indices!(ordered_set);
    indices!(cardinality);
    indices!(logic);
    indices!(complementarity);
    indices!(state_specification);
    indices!(state_port);
    indices!(inventory_balance);
    indices!(connection);
    Vec::new()
}

fn member_index_type(
    declaration: DeclarationId,
    position: usize,
    env: &BTreeMap<String, Type>,
    package: &CheckedPackage,
    context: &TypeContext<'_>,
) -> Result<Type> {
    match crate::temporal::index_domain(package, declaration, position)? {
        crate::temporal::IndexDomain::Temporal { policy, axis } => package
            .types
            .get(&axis)
            .cloned()
            .ok_or_else(|| invalid(policy, "resolved temporal axis type absent")),
        crate::temporal::IndexDomain::Authored { position } => index_domain_type(
            package.static_at(
                declaration,
                &member_index_role(&package.declarations[&declaration]),
                position,
            )?,
            env,
            package,
            context,
            declaration,
        ),
    }
}

pub(crate) fn member_index_role(row: &crate::Declaration) -> String {
    macro_rules! role { ($($field:ident),+) => { $(if row.value.$field.is_some() { return concat!(stringify!($field), ".indices.domain").into(); })+ }; }
    role!(
        binding,
        equation,
        accumulator,
        contribution,
        boundary,
        exchange,
        coordinate_slot,
        ordered_set,
        cardinality,
        piecewise,
        logic,
        complementarity,
        state_specification,
        state_port,
        inventory_balance,
        connection
    );
    String::new()
}

pub(crate) fn member_index_names(row: &crate::Declaration) -> Vec<&str> {
    macro_rules! names { ($($field:ident),+) => { $(if let Some(value) = &row.value.$field { return value.indices.iter().map(|index| index.name.as_str()).collect(); })+ }; }
    names!(
        binding,
        equation,
        accumulator,
        contribution,
        boundary,
        exchange,
        coordinate_slot,
        ordered_set,
        cardinality,
        piecewise,
        logic,
        complementarity,
        state_specification,
        state_port,
        inventory_balance,
        connection
    );
    Vec::new()
}

pub(crate) fn index_domain_type(
    value: &pse_authoring::language::StaticValue,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<Type> {
    fn syntax(
        value: &pse_authoring::language::StaticValue,
        env: &BTreeMap<String, Type>,
        p: &CheckedPackage,
        c: &TypeContext<'_>,
        at: DeclarationId,
    ) -> Result<Type> {
        use pse_authoring::language::StaticValue as S;
        match value {
            S::Expression(expression) => infer(expression, env, p, c, at, None),
            S::Text(_) => Ok(Type::Text),
            S::Tuple(values) => Ok(Type::Tuple(
                values
                    .iter()
                    .map(|value| syntax(value, env, p, c, at))
                    .collect::<Result<Vec<_>>>()?,
            )),
            S::Set(values) => {
                let first = values.first().ok_or_else(|| {
                    invalid(
                        at,
                        "empty index membership requires a typed set declaration",
                    )
                })?;
                let element = syntax(first, env, p, c, at)?;
                for value in &values[1..] {
                    let actual = syntax(value, env, p, c, at)?;
                    if !p.subsumes(&element, &actual) {
                        return Err(invalid(at, "index membership element types differ"));
                    }
                }
                Ok(Type::Set(Box::new(element)))
            }
            S::Comprehension {
                body,
                bindings,
                filter,
            } => {
                let mut local = env.clone();
                for (name, domain) in bindings {
                    let Type::Set(element) = syntax(domain, &local, p, c, at)? else {
                        return Err(invalid(at, "comprehension index domain must be a set"));
                    };
                    local.insert(name.clone(), *element);
                }
                if let Some(filter) = filter {
                    predicate(filter, &local, p, c, at)?;
                }
                Ok(Type::Set(Box::new(syntax(body, &local, p, c, at)?)))
            }
            S::Apply { name, .. } => p
                .resolve(at, name)
                .and_then(|id| p.types.get(&id))
                .cloned()
                .ok_or_else(|| invalid(at, "unknown index domain constructor")),
        }
    }
    syntax(value, env, p, c, at)
}

fn process_index_environment(
    indices: impl IntoIterator<Item = (String, String)>,
    role: &str,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<BTreeMap<String, Type>> {
    let mut local = env.clone();
    let mut bound = member_indices(&p.declarations[&at])
        .into_iter()
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    for (position, (name, _domain)) in indices.into_iter().enumerate() {
        if !bound.insert(name.clone()) {
            return Err(invalid(
                at,
                "process slot index shadows an existing binding",
            ));
        }
        let Type::Set(element) =
            index_domain_type(p.static_at(at, role, position)?, &local, p, c, at)?
        else {
            return Err(invalid(at, "process slot indices require finite sets"));
        };
        local.insert(name, *element);
    }
    Ok(local)
}

fn same_process_type(
    actual: &Type,
    expected: &Type,
    c: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<bool> {
    if actual == expected {
        return Ok(true);
    }
    let (Some(actual), Some(expected)) = (actual.quantity_scheme(), expected.quantity_scheme())
    else {
        return Ok(false);
    };
    let actual = actual
        .resolve_contract_with_evidence(c.quantities, &Substitution::new(), c.preconditions)
        .map_err(|e| invalid(at, e.to_string()))?;
    let expected = expected
        .resolve_contract_with_evidence(c.quantities, &Substitution::new(), c.preconditions)
        .map_err(|e| invalid(at, e.to_string()))?;
    Ok(actual.same_meaning(&expected))
}

fn process_value(
    expression: &Expr,
    expected: &Type,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<()> {
    let actual = infer(expression, env, p, c, at, Some(expected))?;
    if !same_process_type(&actual, expected, c, at)? {
        return Err(invalid(
            at,
            "process expression physical type differs from its contract",
        ));
    }
    Ok(())
}

fn check_process_contracts(
    row: &crate::Declaration,
    at: DeclarationId,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    c: &TypeContext<'_>,
) -> Result<()> {
    if let Some(state) = &row.value.state_specification {
        let mut ancestors = BTreeSet::from([at]);
        let mut owner = at;
        let mut local = env.clone();
        while let Some(_source) = p.declarations[&owner]
            .value
            .state_specification
            .as_ref()
            .and_then(|state| state.extends.as_ref())
        {
            let base = indexed_declaration_reference(
                p.expression_at(owner, "state_specification.extends", 0)?,
                &local,
                p,
                c,
                owner,
            )?;
            if p.declarations[&base].value.state_specification.is_none() {
                return Err(invalid(
                    at,
                    "state extension requires a state specification",
                ));
            }
            if !ancestors.insert(base) {
                return Err(invalid(at, "recursive state specification extension"));
            }
            local = declaration_environment(p, c, base)?;
            for (position, (name, _domain)) in member_indices(&p.declarations[&base])
                .into_iter()
                .enumerate()
            {
                let (Type::Set(element) | Type::Continuous(_, element)) =
                    member_index_type(base, position, &local, p, c)?
                else {
                    return Err(invalid(base, "state index requires a set"));
                };
                local.insert(name.clone(), *element);
            }
            owner = base;
        }
        process_value(
            p.expression_at(at, "state_specification.supplied", 0)?,
            &Type::Boolean,
            env,
            p,
            c,
            at,
        )?;
        let mut names = BTreeSet::new();
        for (position, slot) in state.coordinates.iter().enumerate() {
            if !names.insert(slot.name.as_str()) {
                return Err(invalid(at, "duplicate state coordinate"));
            }
            let local = process_index_environment(
                slot.indices
                    .iter()
                    .map(|i| (i.name.clone(), i.domain.clone())),
                &format!("state_specification.coordinates.{position}.indices.domain"),
                env,
                p,
                c,
                at,
            )?;
            let target = infer(
                p.expression_at(at, "state_specification.coordinates.target", position)?,
                &local,
                p,
                c,
                at,
                None,
            )?;
            if target.quantity_scheme().is_none() {
                return Err(invalid(
                    at,
                    "state coordinates require physical numeric values",
                ));
            }
        }
        names.clear();
        for (position, slot) in state.reconstructions.iter().enumerate() {
            if !names.insert(slot.name.as_str()) {
                return Err(invalid(at, "duplicate state reconstruction"));
            }
            let local = process_index_environment(
                slot.indices
                    .iter()
                    .map(|i| (i.name.clone(), i.domain.clone())),
                &format!("state_specification.reconstructions.{position}.indices.domain"),
                env,
                p,
                c,
                at,
            )?;
            fn relation(
                e: &dsl::Equation,
                tolerance: &Expr,
                env: &BTreeMap<String, Type>,
                p: &CheckedPackage,
                c: &TypeContext<'_>,
                at: DeclarationId,
            ) -> Result<()> {
                match &e.kind {
                    dsl::EquationKind::Relation { lhs, rhs, .. } => {
                        let left = infer(lhs, env, p, c, at, None)?;
                        let right = infer(rhs, env, p, c, at, Some(&left))?;
                        if !same_process_type(&left, &right, c, at)? {
                            return Err(invalid(
                                at,
                                "state reconstruction side physical types differ",
                            ));
                        }
                        let tolerance_type = Type::Quantity(delta(scheme(&left, at)?));
                        process_value(tolerance, &tolerance_type, env, p, c, at)
                    }
                    dsl::EquationKind::Conditional {
                        guard,
                        then,
                        otherwise,
                    } => {
                        predicate(guard, env, p, c, at)?;
                        relation(then, tolerance, env, p, c, at)?;
                        relation(otherwise, tolerance, env, p, c, at)
                    }
                }
            }
            relation(
                p.equation_at(at, "state_specification.reconstructions.equation", position)?,
                p.expression_at(
                    at,
                    "state_specification.reconstructions.tolerance",
                    position,
                )?,
                &local,
                p,
                c,
                at,
            )?;
        }
        names.clear();
        for (position, slot) in state.transports.iter().enumerate() {
            if !names.insert(slot.name.as_str()) {
                return Err(invalid(at, "duplicate state transported observation"));
            }
            let local = process_index_environment(
                slot.indices
                    .iter()
                    .map(|i| (i.name.clone(), i.domain.clone())),
                &format!("state_specification.transports.{position}.indices.domain"),
                env,
                p,
                c,
                at,
            )?;
            let target = infer(
                p.expression_at(at, "state_specification.transports.expression", position)?,
                &local,
                p,
                c,
                at,
                None,
            )?;
            let tolerance_type = Type::Quantity(delta(scheme(&target, at)?));
            process_value(
                p.expression_at(at, "state_specification.transports.tolerance", position)?,
                &tolerance_type,
                &local,
                p,
                c,
                at,
            )?;
        }
    }
    if let Some(_port) = &row.value.state_port {
        let target = indexed_declaration_reference(
            p.expression_at(at, "state_port.specification", 0)?,
            env,
            p,
            c,
            at,
        )?;
        if p.declarations[&target].value.state_specification.is_none() {
            return Err(invalid(at, "state port requires a state specification"));
        }
    }
    if let Some(balance) = &row.value.inventory_balance {
        let expected = p
            .types
            .get(&at)
            .ok_or_else(|| invalid(at, "inventory contract type absent"))?;
        process_value(
            p.expression_at(at, "inventory_balance.inventory", 0)?,
            expected,
            env,
            p,
            c,
            at,
        )?;
        let axis = infer(
            p.expression_at(at, "inventory_balance.axis", 0)?,
            env,
            p,
            c,
            at,
            None,
        )?;
        let (Type::Continuous(_, axis) | Type::Set(axis)) = axis else {
            return Err(invalid(at, "inventory balance requires a continuous axis"));
        };
        let inventory = scheme(expected, at)?;
        let flux = Type::Quantity(Scheme::Quotient(
            Box::new(delta(inventory.clone())),
            Box::new(delta(scheme(&axis, at)?)),
        ));
        process_value(
            p.expression_at(at, "inventory_balance.flux", 0)?,
            &flux,
            env,
            p,
            c,
            at,
        )?;
        let difference = Type::Quantity(delta(inventory));
        process_value(
            p.expression_at(at, "inventory_balance.tolerance", 0)?,
            &difference,
            env,
            p,
            c,
            at,
        )?;
        let mut events = BTreeSet::new();
        for (position, _transfer) in balance.transfers.iter().enumerate() {
            let event = p.expression_at(at, "inventory_balance.transfers.event", position)?;
            let target = indexed_declaration_reference(event, env, p, c, at)?;
            let ExprKind::Path(path) = &event.kind else {
                return Err(invalid(
                    at,
                    "inventory event transfer requires a guard member path",
                ));
            };
            if infer(event, env, p, c, at, None)?
                .quantity_scheme()
                .is_none()
            {
                return Err(invalid(
                    at,
                    "inventory event guard requires a physical numeric member",
                ));
            }
            if !events.insert((target, dsl::render_path(path))) {
                return Err(invalid(at, "duplicate inventory event transfer"));
            }
            process_value(
                p.expression_at(at, "inventory_balance.transfers.expression", position)?,
                &difference,
                env,
                p,
                c,
                at,
            )?;
        }
    }
    if let Some(_connection) = &row.value.connection {
        let endpoints = ["connection.from", "connection.to"].map(|role| {
            indexed_declaration_reference(p.expression_at(at, role, 0)?, env, p, c, at)
        });
        let from = endpoints[0].as_ref().map_err(Clone::clone)?;
        let to = endpoints[1].as_ref().map_err(Clone::clone)?;
        let source = &p.declarations[from].value;
        let target = &p.declarations[to].value;
        // Legacy aggregate instance ports still use their existing specialization owner.
        if source.state_port.is_some() != target.state_port.is_some() {
            return Err(invalid(
                at,
                "connection endpoints require the same port contract family",
            ));
        }
    }
    Ok(())
}

/// Resolve a declaration reference that is consumed by a contextual intrinsic rather
/// than an ordinary value type, validating each actual coordinate in its path.
pub(crate) fn indexed_declaration_reference(
    expression: &Expr,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<DeclarationId> {
    let ExprKind::Path(path) = &expression.kind else {
        return Err(invalid(
            at,
            "a contextual declaration reference requires a member path",
        ));
    };
    let mut prefix = String::new();
    let mut selected = None;
    let mut current_type = None;
    for (position, segment) in path.segments.iter().enumerate() {
        if !prefix.is_empty() {
            prefix.push('.');
        }
        prefix.push_str(&segment.name);
        let id = if let Some(ty) = &current_type {
            member_declaration(ty, &segment.name, p, at)?
        } else if let Some(id) = p.resolve_segments(at, &path.segments[..=position]) {
            id
        } else if let Some(ty) = env.get(&prefix).filter(|_| segment.indices.is_empty()) {
            current_type = Some(ty.clone());
            continue;
        } else {
            return Err(invalid(at, "unknown contextual declaration reference"));
        };
        check_member_indices(
            segment,
            member_indices(&p.declarations[&id]),
            env,
            p,
            c,
            at,
            id,
        )?;
        current_type = env.get(&prefix).or_else(|| p.types.get(&id)).cloned();
        if current_type.is_none()
            && p.declarations[&id].value.kind
                == pse_model::generated::enums::ModelingDeclarationKind::Child
        {
            // An unannotated child still has a checked constructor occurrence.
            // Resolve only its nominal source contract; selected arguments and
            // instantiated/indexed port identities remain specialization obligations.
            use pse_authoring::language::StaticValue;
            let constructor = match p.static_at(id, "binding.expression", 0)? {
                StaticValue::Apply { name, .. } => p.resolve(id, name),
                StaticValue::Expression(Expr {
                    kind: ExprKind::NamedCall { name, .. },
                    ..
                }) => p.resolve_segments(id, &name.segments),
                _ => None,
            };
            if let Some(target) = constructor {
                current_type = Some(Type::Definition(p.preset_definition(target)?));
            }
        }
        selected = Some(id);
    }
    selected.ok_or_else(|| invalid(at, "empty contextual declaration reference"))
}
fn check_member_indices(
    segment: &dsl::PathSegment,
    indices: Vec<(&String, &String)>,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
    id: DeclarationId,
) -> Result<()> {
    if indices.len() != segment.indices.len() {
        return Err(invalid(at, "member index arity"));
    }
    let mut local = declaration_environment(p, c, id)?;
    for (position, (index, (name, _domain))) in segment.indices.iter().zip(indices).enumerate() {
        let (Type::Set(element) | Type::Continuous(_, element)) =
            member_index_type(id, position, &local, p, c)?
        else {
            return Err(invalid(id, "index domain must be a set"));
        };
        if !p.subsumes(&element, &infer(index, env, p, c, at, Some(&element))?) {
            return Err(invalid(at, "index member kind differs"));
        }
        local.insert(name.clone(), *element);
    }
    Ok(())
}
fn path_type(
    path: &Path,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<Type> {
    let text = dsl::render_path(path);
    if let Some(ty) = crate::analysis::type_of(&text) {
        return Ok(ty);
    }
    if matches!(text.as_str(), "true" | "false") {
        return Ok(Type::Boolean);
    }
    let first = path
        .segments
        .first()
        .ok_or_else(|| invalid(at, "empty path"))?;
    let qualified = path
        .segments
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join(".");
    if path.segments.len() == 1
        && path.segments.iter().all(|s| s.indices.is_empty())
        && let Some(ty) = env.get(&qualified)
    {
        // A declared indexed member still requires coordinates; lexical values do not.
        if !p
            .resolve_segments(at, &path.segments)
            .is_some_and(|id| !member_indices(&p.declarations[&id]).is_empty())
        {
            return Ok(ty.clone());
        }
    }
    // Package scopes have no value type. Resolve their first typed declaration
    // before traversing indexed values and row members. A lexical value still
    // takes precedence, and resolution retains explicit-import visibility.
    let mut prefix = String::new();
    let mut root = None;
    for (position, segment) in path.segments.iter().enumerate() {
        if position > 0 {
            prefix.push('.');
        }
        prefix.push_str(&segment.name);
        let declaration = p.resolve_segments(at, &path.segments[..=position]);
        if let Some(ty) = env
            .get(&prefix)
            .filter(|_| {
                position == 0 || declaration.is_some() || p.physical_name(at, &prefix).is_some()
            })
            .cloned()
            .or_else(|| declaration.and_then(|id| p.types.get(&id).cloned()))
        {
            root = Some((position, declaration, ty));
            break;
        }
        if !segment.indices.is_empty() {
            break;
        }
    }
    let (start, mut declaration, mut ty) =
        root.ok_or_else(|| invalid(at, format!("unknown member {}", first.name)))?;
    for (position, segment) in path.segments.iter().enumerate().skip(start) {
        let prefix = dsl::render_path(&Path {
            segments: path.segments[..=position].to_vec(),
        });
        if let Some(refined) = env.get(&prefix)
            && (position > 0 || !segment.indices.is_empty())
            && !p
                .resolve_segments(at, &path.segments[..=position])
                .is_some_and(|id| {
                    segment.indices.is_empty() && !member_indices(&p.declarations[&id]).is_empty()
                })
        {
            ty = refined.clone();
            declaration = None;
            continue;
        }
        if position > start {
            let (next, id) = member_type(&ty, &segment.name, p, at)?;
            ty = next;
            declaration = id;
        }
        if let Type::Table(id) = ty {
            let table = p
                .tables
                .get(&id)
                .ok_or_else(|| invalid(id, "table not admitted"))?;
            if segment.indices.len() != table.keys.len() {
                return Err(invalid(at, "table key arity"));
            }
            for (index, key) in segment.indices.iter().zip(&table.keys) {
                if !p.subsumes(&key.ty, &infer(index, env, p, c, at, Some(&key.ty))?) {
                    return Err(invalid(at, "table key type"));
                }
            }
            ty = if table.optional() {
                Type::Optional(Box::new(table.result.clone()))
            } else {
                table.result.clone()
            };
            declaration = None;
        } else if let Type::Indexed { element, axes } = &ty {
            if segment.indices.is_empty() {
                continue;
            }
            if segment.indices.len() != axes.len() {
                return Err(invalid(at, "indexed member arity"));
            }
            for (index, kind) in segment.indices.iter().zip(axes) {
                if !coordinate_of(&infer(index, env, p, c, at, None)?, *kind, p) {
                    return Err(invalid(at, "index kind differs"));
                }
            }
            ty = *element.clone();
            declaration = None;
        } else if let Some(kind) = declaration.filter(|id| p.kinds.contains_key(id))
            && !segment.indices.is_empty()
        {
            // ADR-0123 Outcome 2: `kind[keys]` names a row of a keyed kind by the
            // key-declaring kind's keys in order; trailing keys with a default may be omitted.
            let (_, keys) = p
                .keys(kind)
                .ok_or_else(|| invalid(at, format!("kind {} has no keys", segment.name)))?;
            if segment.indices.len() > keys.len()
                || keys[segment.indices.len()..].iter().any(|k| k.2.is_none())
            {
                return Err(invalid(
                    at,
                    format!("a {} row is looked up by its keys", segment.name),
                ));
            }
            for (index, (_, expected, _)) in segment.indices.iter().zip(&keys) {
                if !p.subsumes(expected, &infer(index, env, p, c, at, Some(expected))?) {
                    return Err(invalid(at, format!("{} key type", segment.name)));
                }
            }
            declaration = None;
        } else if let Some(id) = declaration {
            let indices = member_indices(&p.declarations[&id]);
            if segment.indices.is_empty() && !indices.is_empty() {
                let mut axes = Vec::new();
                let mut local = declaration_environment(p, c, id)?;
                for (position, (name, _domain)) in indices.iter().enumerate() {
                    let (Type::Set(element) | Type::Continuous(_, element)) =
                        member_index_type(id, position, &local, p, c)?
                    else {
                        return Err(invalid(id, "index domain must be a set"));
                    };
                    let kind = match element.as_ref() {
                        Type::Entity(id) | Type::Enum(id) => *id,
                        _ => {
                            return Err(invalid(
                                id,
                                "indexed physical argument requires declared coordinate kinds",
                            ));
                        }
                    };
                    axes.push(kind);
                    local.insert((*name).clone(), *element);
                }
                ty = Type::Indexed {
                    element: Box::new(ty),
                    axes,
                };
                continue;
            }
            check_member_indices(segment, indices, env, p, c, at, id)?;
        } else if !segment.indices.is_empty() {
            return Err(invalid(at, "scalar member cannot be indexed"));
        }
    }
    Ok(ty)
}
/// Whether a value of type `actual` indexes an axis of `kind`: an enumeration member, or
/// an entity of the kind or a refinement.
fn coordinate_of(actual: &Type, kind: DeclarationId, p: &CheckedPackage) -> bool {
    match actual {
        Type::Enum(id) => *id == kind,
        Type::Entity(id) => p.refines(*id, kind),
        _ => false,
    }
}
fn member_declaration(
    ty: &Type,
    name: &str,
    p: &CheckedPackage,
    at: DeclarationId,
) -> Result<DeclarationId> {
    let (Type::Entity(id) | Type::Definition(id) | Type::Interface(id)) = ty else {
        return Err(invalid(at, "value has no declaration members"));
    };
    p.declared_member(*id, name)
        .ok_or_else(|| invalid(at, format!("type has no member {name}")))
}
fn member_type(
    ty: &Type,
    name: &str,
    p: &CheckedPackage,
    at: DeclarationId,
) -> Result<(Type, Option<DeclarationId>)> {
    match ty {
        Type::Enum(id)
            if p.declarations[id]
                .value
                .enumeration
                .as_ref()
                .is_some_and(|e| e.members.iter().any(|m| m.name == name)) =>
        {
            Ok((Type::Enum(*id), None))
        }
        Type::Row(id) => p
            .tables
            .get(id)
            .and_then(|t| t.columns.iter().find(|c| c.name == name))
            .map(|c| (c.ty.clone(), None))
            .ok_or_else(|| invalid(at, "unknown table column")),
        Type::Entity(_) | Type::Definition(_) | Type::Interface(_) => {
            let member = member_declaration(ty, name, p, at)?;
            p.types
                .get(&member)
                .cloned()
                .map(|ty| (ty, Some(member)))
                .ok_or_else(|| invalid(at, "member type absent"))
        }
        // ADR-0123 Outcome 6: a reference state's typed conditions.
        Type::ReferenceState if matches!(name, "temperature" | "pressure") => Ok((
            Type::Quantity(Scheme::Concrete(p.reference_attribute_type(name, at)?)),
            None,
        )),
        Type::Optional(_) => Err(invalid(
            at,
            "optional value must be guarded before member access",
        )),
        _ => Err(invalid(at, "value has no named members")),
    }
}

fn refine(
    guard: &Predicate,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<BTreeMap<String, Type>> {
    let mut env = env.clone();
    match &guard.kind {
        PredicateKind::And(a, b) => {
            env = refine(a, &env, p, c, at)?;
            env = refine(b, &env, p, c, at)?;
        }
        PredicateKind::Atom(expression) => {
            if let ExprKind::NamedCall { name, args } = &expression.kind {
                if name.is_ident("present")
                    && args.len() == 1
                    && let ExprKind::Path(path) = &args[0].kind
                    && let Type::Optional(inner) = infer(&args[0], &env, p, c, at, None)?
                {
                    env.insert(dsl::render_path(path), *inner);
                }
                if name.is_ident("implements")
                    && args.len() == 2
                    && let (ExprKind::Path(target), ExprKind::Path(contract)) =
                        (&args[0].kind, &args[1].kind)
                {
                    let id = p
                        .resolve_segments(at, &contract.segments)
                        .filter(|_| {
                            contract
                                .segments
                                .iter()
                                .all(|segment| segment.indices.is_empty())
                        })
                        .ok_or_else(|| invalid(at, "interface refinement"))?;
                    if !matches!(p.types.get(&id), Some(Type::Interface(_))) {
                        return Err(invalid(at, "interface refinement requires an interface"));
                    }
                    env.insert(dsl::render_path(target), Type::Interface(id));
                }
            }
        }
        _ => {}
    }
    Ok(env)
}

/// Conservative owned syntax storage for compiler memo and worker admission.
pub fn retained_bytes(expression: &Expr) -> usize {
    crate::extent::expression(expression)
}

/// Static typing of an objective annotation's members (ADR-0111): the weight and relative
/// tolerance are dimensionless, the normalization has the target's type, and the absolute
/// tolerance is a difference of the member's term, the target or its dimensionless
/// quotient by the normalization.
fn objective_members(
    a: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueAnnotation,
    target: &Type,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    context: &TypeContext<'_>,
    id: DeclarationId,
) -> Result<()> {
    let members = a
        .objective
        .as_ref()
        .filter(|_| a.arguments.is_empty())
        .ok_or_else(|| invalid(id, "an objective annotation carries its typed members"))?;
    let scalar = Type::Quantity(Scheme::Concrete(
        context
            .quantities
            .neutral_dimensionless()
            .ok_or_else(|| invalid(id, "objective members need a neutral scalar type"))?,
    ));
    let term = if members.normalization.is_some() {
        &scalar
    } else {
        target
    };
    let Type::Quantity(scheme) = term else {
        return Err(invalid(id, "objective requires a physical member"));
    };
    let difference = Type::Quantity(Scheme::Concrete(
        Scheme::Delta(Box::new(scheme.clone()))
            .resolve_with_evidence(
                context.quantities,
                &Substitution::new(),
                context.preconditions,
            )
            .map_err(|e| invalid(id, e.to_string()))?,
    ));
    for (role, source, expected) in [
        ("annotation.objective.weight", &members.weight, &scalar),
        (
            "annotation.objective.normalization",
            &members.normalization,
            target,
        ),
        (
            "annotation.objective.absolute_tolerance",
            &members.absolute_tolerance,
            &difference,
        ),
        (
            "annotation.objective.relative_tolerance",
            &members.relative_tolerance,
            &scalar,
        ),
    ] {
        if let Some(_source) = source {
            let expression = p.expression_at(id, role, 0)?;
            if infer(expression, env, p, context, id, Some(expected))? != *expected {
                return Err(invalid(id, "objective member type differs"));
            }
        }
    }
    Ok(())
}
