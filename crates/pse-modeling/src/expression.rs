// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source expression checking; executable arithmetic remains in pse-math.
use crate::{
    DeclarationId, Result,
    check::CheckedPackage,
    invalid,
    types::{Type, TypeContext},
};
use pse_authoring::dsl::{self, BinaryOp, Expr, ExprKind, Path, Predicate, PredicateKind};
use pse_quantity::{
    IndexSet, Ratio,
    infer::{Chain, Exponent, OpRequest, Operand},
    literal::LiteralContext,
    scheme::{Scheme, Substitution},
};
use std::collections::{BTreeMap, BTreeSet};

/// Referenced source paths in deterministic lexical traversal order.
pub fn references(expression: &Expr) -> BTreeSet<String> {
    expression
        .free_paths()
        .iter()
        .map(|p| dsl::render_path(p))
        .collect()
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
    if let Type::Quantity(s) = ty {
        Ok(s.clone())
    } else {
        Err(invalid(
            at,
            "physical expression requires quantity operands",
        ))
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
    request: OpRequest<'_>,
    values: &[Scheme],
    context: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<Option<Type>> {
    let Some(ids) = values
        .iter()
        .map(|v| concrete(v, context))
        .collect::<Option<Vec<_>>>()
    else {
        return Ok(None);
    };
    let indices = IndexSet::new();
    let operands = ids
        .iter()
        .map(|id| Operand {
            quantity_type: *id,
            indices: &indices,
        })
        .collect::<Vec<_>>();
    // A product, quotient or exact power is a two-factor multiplicative chain (ADR-0124).
    let chain = match (&request, operands.as_slice()) {
        (OpRequest::Mul, [a, b]) => Some(Chain::Mul(
            Box::new(Chain::Leaf(*a)),
            Box::new(Chain::Leaf(*b)),
        )),
        (OpRequest::Div, [a, b]) => Some(Chain::Div(
            Box::new(Chain::Leaf(*a)),
            Box::new(Chain::Leaf(*b)),
        )),
        (
            OpRequest::Pow {
                exponent: Exponent::Rational(exponent),
            },
            [a, b],
        ) => Some(Chain::Pow {
            base: Box::new(Chain::Leaf(*a)),
            exponent: *exponent,
            power: *b,
        }),
        _ => None,
    };
    let inferred = match chain {
        Some(chain) => {
            pse_quantity::infer::infer_chain(&chain, context.quantities, context.preconditions)
        }
        None => pse_quantity::infer::infer_with_evidence(
            &request,
            &operands,
            context.quantities,
            context.preconditions,
        ),
    }
    .map_err(|e| invalid(at, e.to_string()))?;
    Ok(Some(Type::Quantity(Scheme::Concrete(inferred.result))))
}
/// One node of a maximal product, quotient and exact-power subtree (ADR-0124).
enum Node {
    Leaf(Type),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Pow(Box<Node>, Ratio, Type),
}
/// A product, a quotient or an exact power belongs to a multiplicative chain.
pub(crate) fn chain_operation(op: BinaryOp, rhs: &Expr) -> bool {
    match op {
        BinaryOp::Mul | BinaryOp::Div => true,
        BinaryOp::Pow => rational(rhs).is_some(),
        BinaryOp::Add | BinaryOp::Sub => false,
    }
}
fn chain_node(
    expr: &Expr,
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    context: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<Node> {
    match &expr.kind {
        ExprKind::Binary { op, lhs, rhs } if chain_operation(*op, rhs) => {
            let left = Box::new(chain_node(lhs, env, p, context, at)?);
            Ok(match op {
                BinaryOp::Pow => Node::Pow(
                    left,
                    rational(rhs).ok_or_else(|| invalid(at, "exact exponent"))?,
                    infer(rhs, env, p, context, at, None)?,
                ),
                BinaryOp::Div => Node::Div(left, Box::new(chain_node(rhs, env, p, context, at)?)),
                _ => Node::Mul(left, Box::new(chain_node(rhs, env, p, context, at)?)),
            })
        }
        _ => Ok(Node::Leaf(infer(expr, env, p, context, at, None)?)),
    }
}
impl Node {
    /// The factors, without the exponents of powers, and whether every exponent is whole.
    fn factors<'a>(&'a self, out: &mut Vec<&'a Type>) -> bool {
        match self {
            Self::Leaf(ty) => {
                out.push(ty);
                true
            }
            Self::Mul(a, b) | Self::Div(a, b) => a.factors(out) & b.factors(out),
            Self::Pow(a, exponent, _) => a.factors(out) && exponent.is_integer(),
        }
    }
    /// The quantity chain when every factor has a concrete complete type.
    fn concrete<'a>(
        &self,
        context: &TypeContext<'_>,
        indices: &'a IndexSet,
    ) -> Option<Chain<'a>> {
        let operand = |ty: &Type| match ty {
            Type::Quantity(s) => concrete(s, context).map(|quantity_type| Operand {
                quantity_type,
                indices,
            }),
            _ => None,
        };
        Some(match self {
            Self::Leaf(ty) => Chain::Leaf(operand(ty)?),
            Self::Mul(a, b) => Chain::Mul(
                Box::new(a.concrete(context, indices)?),
                Box::new(b.concrete(context, indices)?),
            ),
            Self::Div(a, b) => Chain::Div(
                Box::new(a.concrete(context, indices)?),
                Box::new(b.concrete(context, indices)?),
            ),
            Self::Pow(a, exponent, power) => Chain::Pow {
                base: Box::new(a.concrete(context, indices)?),
                exponent: *exponent,
                power: operand(power)?,
            },
        })
    }
    /// A concrete subtree resolves as one chain; a polymorphic one keeps its scheme with
    /// the generic simplifications of a single operation.
    fn scheme(&self, context: &TypeContext<'_>, at: DeclarationId) -> Result<Scheme> {
        let indices = IndexSet::new();
        if let Some(chain) = self.concrete(context, &indices) {
            let inferred =
                pse_quantity::infer::infer_chain(&chain, context.quantities, context.preconditions)
                    .map_err(|e| invalid(at, e.to_string()))?;
            return Ok(Scheme::Concrete(inferred.result));
        }
        let neutral = context
            .quantities
            .neutral_dimensionless()
            .map(Scheme::Concrete);
        Ok(match self {
            Self::Leaf(ty) => scheme(ty, at)?,
            Self::Mul(a, b) => {
                let (a, b) = (a.scheme(context, at)?, b.scheme(context, at)?);
                if Some(&b) == neutral.as_ref() {
                    a
                } else if Some(&a) == neutral.as_ref() {
                    b
                } else if a == b {
                    Scheme::Power(
                        Box::new(a),
                        Ratio::new(2, 1).map_err(|e| invalid(at, e.to_string()))?,
                    )
                } else {
                    Scheme::Product(Box::new(a), Box::new(b))
                }
            }
            Self::Div(a, b) => {
                let (a, b) = (a.scheme(context, at)?, b.scheme(context, at)?);
                if Some(&b) == neutral.as_ref() {
                    a
                } else if a == b {
                    // A generic normalization constrains its eventual quotient to the
                    // neutral type. Concrete calls still lower the original division
                    // through the physical registry; this is not an operation rule.
                    neutral.ok_or_else(|| invalid(at, "neutral normalization type absent"))?
                } else {
                    Scheme::Quotient(Box::new(a), Box::new(b))
                }
            }
            Self::Pow(a, exponent, _) => Scheme::Power(Box::new(a.scheme(context, at)?), *exponent),
        })
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
    let q = |s| Type::Quantity(s);
    match &expr.kind {
        ExprKind::Number(n) => {
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
                    .and_then(|t| {
                        if let Type::Quantity(s) = t {
                            concrete(s, context)
                        } else {
                            None
                        }
                    })
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
            Ok(q(Scheme::Concrete(id)))
        }
        ExprKind::Path(path) => path_type(path, env, p, context, at),
        ExprKind::Neg(e) => {
            let ty = infer(e, env, p, context, at, expected)?;
            if ty == Type::Integer {
                return Ok(ty);
            }
            let a = scheme(&ty, at)?;
            Ok(physical_op(OpRequest::Neg, std::slice::from_ref(&a), context, at)?.unwrap_or(q(a)))
        }
        ExprKind::Binary { op, lhs, rhs }
            if chain_operation(*op, rhs) && expected != Some(&Type::Integer) =>
        {
            // A maximal product, quotient and exact-power subtree is typed as one
            // multiplicative chain (ADR-0124). Its factors get no expected type.
            let node = chain_node(expr, env, p, context, at)?;
            let mut factors = Vec::new();
            let whole = node.factors(&mut factors);
            if factors.iter().any(|ty| **ty == Type::Integer) {
                return if whole && factors.iter().all(|ty| **ty == Type::Integer) {
                    Ok(Type::Integer)
                } else {
                    Err(invalid(at, "integer operand type"))
                };
            }
            Ok(q(node.scheme(context, at)?))
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
            let ta = q(a.clone());
            let b = scheme(
                &infer(
                    rhs,
                    env,
                    p,
                    context,
                    at,
                    if matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                        Some(&ta)
                    } else {
                        None
                    },
                )?,
                at,
            )?;
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
            let operands = vec![a.clone(), b.clone()];
            if let Some(ty) = physical_op(request, &operands, context, at)? {
                return Ok(ty);
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
            Ok(q(result))
        }
        ExprKind::Call { function, args } => {
            let values = args
                .iter()
                .map(|e| infer(e, env, p, context, at, None).and_then(|t| scheme(&t, at)))
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
            if let Some(ty) = physical_op(request, &values, context, at)? {
                return Ok(ty);
            }
            match (function.as_str(), &values[0]) {
                ("sqrt", Scheme::Power(value, power))
                    if *power == Ratio::new(2, 1).map_err(|e| invalid(at, e.to_string()))? =>
                {
                    Ok(q(*value.clone()))
                }
                ("abs", value) => Ok(q(value.clone())),
                _ => Err(invalid(
                    at,
                    "primitive requires a concrete eligible physical type",
                )),
            }
        }
        ExprKind::NamedCall { name, args } if name == "keys" => {
            if args.len() != 1 {
                return Err(invalid(at, "keys arity"));
            }
            let ExprKind::Path(path) = &args[0].kind else {
                return Err(invalid(at, "keys requires a table declaration"));
            };
            let id = p
                .resolve(at, &dsl::render_path(path))
                .ok_or_else(|| invalid(at, "unknown table"))?;
            let table = p
                .tables
                .get(&id)
                .ok_or_else(|| invalid(at, "keys requires a table"))?;
            Ok(Type::Set(Box::new(if table.keys.len() == 1 {
                table.keys[0].clone()
            } else {
                Type::Tuple(table.keys.clone())
            })))
        }
        ExprKind::NamedCall { name, args } if matches!(name.as_str(), "union" | "product") => {
            if args.len() != 2 {
                return Err(invalid(at, "binary set operation arity"));
            }
            let Type::Set(a) = infer(&args[0], env, p, context, at, None)? else {
                return Err(invalid(at, "set required"));
            };
            let Type::Set(b) = infer(&args[1], env, p, context, at, None)? else {
                return Err(invalid(at, "set required"));
            };
            if name == "product" {
                Ok(Type::Set(Box::new(Type::Tuple(vec![*a, *b]))))
            } else if a == b {
                Ok(Type::Set(a))
            } else {
                Err(invalid(at, "union element types differ"))
            }
        }
        ExprKind::NamedCall { name, args } if name == "at" => {
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
        ExprKind::NamedCall { name, args } if name == "present" => {
            if args.len() != 1
                || !matches!(
                    infer(&args[0], env, p, context, at, None)?,
                    Type::Optional(_)
                )
            {
                return Err(invalid(at, "present requires an optional value"));
            }
            Ok(Type::Boolean)
        }
        ExprKind::NamedCall { name, args } if name == "size" => {
            if args.len() != 1
                || !matches!(infer(&args[0], env, p, context, at, None)?, Type::Set(_))
            {
                return Err(invalid(at, "size requires a finite set"));
            }
            Ok(Type::Integer)
        }
        ExprKind::NamedCall { name, args }
            if matches!(name.as_str(), "implements" | "provides") =>
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
            if name == "implements"
                && !p
                    .resolve(at, &dsl::render_path(path))
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
            if !matches!(ty, Type::Quantity(_) | Type::Integer) {
                return Err(invalid(
                    at,
                    "fold requires scalar physical or integer values",
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
            Ok(
                physical_op(OpRequest::Div, &[value.clone(), axis.clone()], context, at)?
                    .unwrap_or(Type::Quantity(Scheme::Quotient(
                        Box::new(value),
                        Box::new(axis),
                    ))),
            )
        }
        ExprKind::Kernel { .. } => Err(invalid(
            at,
            "external functions require the declared K5 contract",
        )),
    }
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
    name: &str,
    args: &[Expr],
    wrt: &[Path],
    env: &BTreeMap<String, Type>,
    p: &CheckedPackage,
    context: &TypeContext<'_>,
    at: DeclarationId,
) -> Result<Type> {
    let indirect;
    let f = if let Some(function) = p.resolve(at, name).and_then(|id| p.functions.get(&id)) {
        function
    } else {
        let expression = dsl::parse_expr(name).map_err(|e| invalid(at, e.to_string()))?;
        let ExprKind::Path(path) = expression.kind else {
            return Err(invalid(at, "function reference path required"));
        };
        let Type::Function { arguments, result } = path_type(&path, env, p, context, at)? else {
            return Err(invalid(at, "reference is not a function"));
        };
        indirect = crate::Function {
            reduction: None,
            validity: None,
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
    if args.len() != f.arguments.len() {
        return Err(invalid(at, "function arity"));
    }
    let actual = args
        .iter()
        .zip(&f.arguments)
        .map(|(expr, (_, formal))| infer(expr, env, p, context, at, Some(formal)))
        .collect::<Result<Vec<_>>>()?;
    let normalize = |s: Scheme| {
        s.resolve_with_evidence(
            context.quantities,
            &Substitution::new(),
            context.preconditions,
        )
        .map(Scheme::Concrete)
        .unwrap_or(s)
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
            let actual = normalize(actual.clone());
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
                if normalize(expected) != normalize(actual.clone()) {
                    return Err(invalid(
                        at,
                        "function complete physical argument type differs",
                    ));
                }
            }
            (formal, actual) if formal == actual => {}
            _ => return Err(invalid(at, "function argument type")),
        }
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
                    if !matches!(infer(index,env,p,context,at,None)?,Type::Entity(id)|Type::Enum(id) if id==*kind)
                    {
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
    Ok(Type::Quantity(normalize(result)))
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
            if infer(expr, env, p, context, at, Some(&element))? != *element {
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
fn declaration_environment(
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
                env.insert(
                    a.name.clone(),
                    context.resolve(&a.r#type, &vars, &env, id)?,
                );
            }
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
    let typed = |source: &str, env: &BTreeMap<String, Type>| -> Result<Type> {
        let e = dsl::parse_expr(source).map_err(|e| invalid(id, e.to_string()))?;
        infer(&e, env, p, context, id, None)
    };
    let indicator = || crate::indicator_type(context.quantities, id);
    if let Some(condition) = row
        .value
        .equation
        .as_ref()
        .and_then(|e| e.condition.as_ref())
        && typed(&condition.variable, env)? != indicator()?
    {
        return Err(invalid(
            id,
            "an indicator condition names an indicator variable",
        ));
    }
    if let Some(set) = &row.value.ordered_set
        && !matches!(typed(&set.member, env)?, Type::Quantity(_))
    {
        return Err(invalid(id, "ordered set members are physical variables"));
    }
    if let Some(c) = &row.value.cardinality
        && !matches!(typed(&c.member, env)?, Type::Quantity(_))
    {
        return Err(invalid(id, "cardinality members are physical variables"));
    }
    if let Some(f) = &row.value.piecewise {
        let [index] = f.indices.as_slice() else {
            return Err(invalid(id, "a piecewise function has one breakpoint index"));
        };
        let mut local = outer.clone();
        let domain = dsl::parse_expr(&index.domain).map_err(|e| invalid(id, e.to_string()))?;
        let Type::Set(element) = infer(&domain, outer, p, context, id, None)? else {
            return Err(invalid(id, "breakpoints range over a finite set"));
        };
        local.insert(index.name.clone(), *element);
        if typed(&f.input, outer)? != typed(&f.abscissa, &local)?
            || typed(&f.output, outer)? != typed(&f.ordinate, &local)?
        {
            return Err(invalid(
                id,
                "breakpoints have the input and output physical types",
            ));
        }
    }
    if let Some(l) = &row.value.logic {
        crate::logic::parse(&l.proposition).map_err(|e| invalid(id, e))?;
    }
    if let Some(c) = &row.value.complementarity
        && [&c.first, &c.second]
            .into_iter()
            .map(|member| typed(member, env))
            .collect::<Result<Vec<_>>>()?
            .iter()
            .any(|ty| !matches!(ty, Type::Quantity(_)))
    {
        return Err(invalid(id, "complementarity members are physical"));
    }
    Ok(())
}
pub(crate) fn check_all(p: &CheckedPackage, context: &TypeContext<'_>) -> Result<()> {
    for (id, row) in &p.declarations {
        let mut env = declaration_environment(p, context, *id)?;
        let mut ancestors = Vec::new();
        let mut cursor = row.parent_id;
        while let Some(owner) = cursor {
            ancestors.push(owner);
            cursor = p.declarations[&owner].parent_id;
        }
        for owner in ancestors.into_iter().rev() {
            if let Some(guard) = &p.declarations[&owner].value.guard {
                let predicate = dsl::parse_predicate(&guard.predicate)
                    .map_err(|e| invalid(owner, e.to_string()))?;
                env = refine(&predicate, &env, p, context, owner)?;
            }
        }
        let indices = row
            .value
            .binding
            .as_ref()
            .map(|b| {
                b.indices
                    .iter()
                    .map(|i| (&i.name, &i.domain))
                    .collect::<Vec<_>>()
            })
            .or_else(|| {
                row.value
                    .equation
                    .as_ref()
                    .map(|v| v.indices.iter().map(|i| (&i.name, &i.domain)).collect())
            })
            .or_else(|| {
                row.value
                    .accumulator
                    .as_ref()
                    .map(|v| v.indices.iter().map(|i| (&i.name, &i.domain)).collect())
            })
            .or_else(|| {
                row.value
                    .contribution
                    .as_ref()
                    .map(|v| v.indices.iter().map(|i| (&i.name, &i.domain)).collect())
            })
            .or_else(|| {
                row.value
                    .ordered_set
                    .as_ref()
                    .map(|v| v.indices.iter().map(|i| (&i.name, &i.domain)).collect())
            })
            .or_else(|| {
                row.value
                    .cardinality
                    .as_ref()
                    .map(|v| v.indices.iter().map(|i| (&i.name, &i.domain)).collect())
            })
            .or_else(|| {
                row.value
                    .logic
                    .as_ref()
                    .map(|v| v.indices.iter().map(|i| (&i.name, &i.domain)).collect())
            })
            .or_else(|| {
                row.value
                    .complementarity
                    .as_ref()
                    .map(|v| v.indices.iter().map(|i| (&i.name, &i.domain)).collect())
            })
            .unwrap_or_default();
        let outer = env.clone();
        for (name, domain) in indices {
            let expr = dsl::parse_expr(domain).map_err(|e| invalid(*id, e.to_string()))?;
            let (Type::Set(element) | Type::Continuous(_, element)) =
                infer(&expr, &env, p, context, *id, None)?
            else {
                return Err(invalid(*id, "indexed member requires a finite set"));
            };
            env.insert(name.clone(), *element);
        }
        check_forms(row, *id, &outer, &env, p, context)?;
        if let Some(axis) = &row.value.continuous {
            let Type::Continuous(_, target) = &p.types[id] else {
                return Err(invalid(*id, "continuous type absent"));
            };
            for source in [&axis.lower, &axis.upper] {
                let e = dsl::parse_expr(source).map_err(|e| invalid(*id, e.to_string()))?;
                if infer(&e, &env, p, context, *id, Some(target))? != **target {
                    return Err(invalid(*id, "continuous bound type differs"));
                }
            }
        }
        if let Some(scope) = &row.value.scope {
            if let Some(selection) = &scope.selection {
                let score = dsl::parse_expr(&selection.criterion)
                    .map_err(|e| invalid(*id, e.to_string()))?;
                let Type::Quantity(quantity) = infer(&score, &env, p, context, *id, None)? else {
                    return Err(invalid(
                        *id,
                        "regime score requires a complete physical quantity",
                    ));
                };
                let tolerance = dsl::parse_expr(&selection.tolerance)
                    .map_err(|e| invalid(*id, e.to_string()))?;
                let expected = Type::Quantity(Scheme::Delta(Box::new(quantity)));
                if infer(&tolerance, &env, p, context, *id, Some(&expected))? != expected {
                    // Concrete types and delta schemes can name the same physical type.
                    let actual = infer(&tolerance, &env, p, context, *id, Some(&expected))?;
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
            if let Some(eligible) = &scope.eligibility {
                predicate(
                    &dsl::parse_predicate(eligible).map_err(|e| invalid(*id, e.to_string()))?,
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
            if grid.scheme != "integrated" {
                crate::continuous::scheme(p, *id, &grid.scheme)?;
            }
            for source in [&grid.elements, &grid.order] {
                let e = dsl::parse_expr(source).map_err(|e| invalid(*id, e.to_string()))?;
                if infer(&e, &env, p, context, *id, Some(&Type::Integer))? != Type::Integer {
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
            let ty = crate::annotation::target_type(p, context, *id, &v.target, &env)?;
            let e = dsl::parse_expr(&v.nominal).map_err(|e| invalid(*id, e.to_string()))?;
            if infer(&e, &env, p, context, *id, Some(&ty))? != ty {
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
            let ty = crate::annotation::target_type(p, context, *id, &v.target, &env)?;
            if !matches!(ty, Type::Quantity(_)) {
                return Err(invalid(*id, "continuation cannot change structural facts"));
            }
            for text in [&v.start, &v.end] {
                let e = dsl::parse_expr(text).map_err(|e| invalid(*id, e.to_string()))?;
                if infer(&e, &env, p, context, *id, Some(&ty))? != ty {
                    return Err(invalid(*id, "continuation endpoint physical type differs"));
                }
            }
        }
        if let Some(f) = p.functions.get(id) {
            env.extend(f.arguments.iter().cloned());
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
            if let Some(validity) = &f.validity {
                predicate(validity, &env, p, context, *id)?;
            }
            let zero = dsl::parse_expr("0").map_err(|e| invalid(*id, e.to_string()))?;
            let dependency_expression = if let Some(validity) = &f.validity {
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
                let mut dependencies = dependency_expression;
                dependencies.try_walk_mut(|node| -> Result<()> {
                    let name = match &node.kind {
                        ExprKind::NamedCall { name, .. }
                        | ExprKind::Partial { function: name, .. } => name,
                        _ => return Ok(()),
                    };
                    if p.resolve(*id, name)
                        .is_some_and(|id| p.functions.contains_key(&id))
                        || matches!(
                            name.as_str(),
                            "keys"
                                | "at"
                                | "size"
                                | "present"
                                | "implements"
                                | "provides"
                                | "union"
                                | "product"
                        )
                    {
                        return Ok(());
                    }
                    let callee = dsl::parse_expr(name).map_err(|e| invalid(*id, e.to_string()))?;
                    node.kind = ExprKind::Binary {
                        op: BinaryOp::Add,
                        lhs: Box::new(callee),
                        rhs: Box::new(node.clone()),
                    };
                    Ok(())
                })?;
                for path in dependencies.free_paths() {
                    let explicit = path
                        .segments
                        .first()
                        .is_some_and(|s| f.arguments.iter().any(|(name, _)| *name == s.name));
                    let immutable = (1..=path.segments.len())
                        .rev()
                        .find_map(|end| {
                            let name = path.segments[..end]
                                .iter()
                                .map(|s| s.name.as_str())
                                .collect::<Vec<_>>()
                                .join(".");
                            p.resolve(*id, &name)
                        })
                        .is_some_and(|value| {
                            use pse_model::generated::enums::ModelingDeclarationKind as K;
                            let declaration = &p.declarations[&value];
                            matches!(
                                declaration.value.kind,
                                K::Entity | K::Set | K::Table | K::Enum
                            ) && declaration.parent_id.is_some_and(|parent| {
                                p.declarations[&parent].value.kind == K::Package
                            })
                        });
                    if !explicit && !immutable {
                        return Err(invalid(
                            *id,
                            "pure function requires explicit runtime arguments or immutable package data",
                        ));
                    }
                }
            }
            if let Some(body) = &f.body {
                let actual = infer(body, &env, p, context, *id, Some(&f.result))?;
                if actual != f.result {
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
            && matches!(expected, Type::Quantity(_))
        {
            for source in b.expression.iter() {
                let expr = dsl::parse_expr(source).map_err(|e| invalid(*id, e.to_string()))?;
                let actual = infer(&expr, &env, p, context, *id, Some(expected))?;
                if &actual != expected {
                    return Err(invalid(*id, "member physical type differs"));
                }
            }
        }
        if let Some(a) = &row.value.annotation {
            // Whole-family annotations bind the target's coordinates just like
            // specialization does; their expressions are scalar at each coordinate.
            let mut env = env.clone();
            let target_declaration =
                crate::annotation::target_declaration(p, context, *id, &a.target, &env)?;
            if let Some(target) = target_declaration {
                let declaration = &p.declarations[&target];
                let indices = declaration
                    .value
                    .binding
                    .as_ref()
                    .map(|b| {
                        b.indices
                            .iter()
                            .map(|i| (&i.name, &i.domain))
                            .collect::<Vec<_>>()
                    })
                    .or_else(|| {
                        declaration
                            .value
                            .equation
                            .as_ref()
                            .map(|e| e.indices.iter().map(|i| (&i.name, &i.domain)).collect())
                    })
                    .or_else(|| {
                        declaration
                            .value
                            .accumulator
                            .as_ref()
                            .map(|a| a.indices.iter().map(|i| (&i.name, &i.domain)).collect())
                    })
                    .unwrap_or_default();
                let mut target_env = declaration_environment(p, context, target)?;
                for (name, domain) in indices {
                    let expression =
                        dsl::parse_expr(domain).map_err(|e| invalid(*id, e.to_string()))?;
                    let (Type::Set(element) | Type::Continuous(_, element)) =
                        infer(&expression, &target_env, p, context, target, None)?
                    else {
                        return Err(invalid(*id, "annotation index domain must be a set"));
                    };
                    target_env.insert(name.clone(), *element.clone());
                    env.insert(name.clone(), *element);
                }
            }
            let target = crate::annotation::target_type(p, context, *id, &a.target, &env)?;
            use crate::annotation::{AnnotationKind as Kind, Shape};
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
            if !matches!(target, Type::Quantity(_)) {
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
                Shape::Objective => {
                    objective_members(a, &target, &env, p, context, *id)?;
                    continue;
                }
                Shape::Expressions(count) => count,
                Shape::Label => {
                    crate::annotation::label(&a.arguments[0], *id)?;
                    0
                }
                Shape::Scheme => 0,
                Shape::Predicate => {
                    predicate(
                        &dsl::parse_predicate(&a.arguments[0])
                            .map_err(|e| invalid(*id, e.to_string()))?,
                        &env,
                        p,
                        context,
                        *id,
                    )?;
                    0
                }
            };
            for argument in &a.arguments[..numeric] {
                let expression =
                    dsl::parse_expr(argument).map_err(|e| invalid(*id, e.to_string()))?;
                if infer(&expression, &env, p, context, *id, Some(&target))? != target {
                    return Err(invalid(*id, "annotation type differs from target"));
                }
            }
        }
        if let Some(expectation) = &row.value.expectation {
            let actual =
                dsl::parse_expr(&expectation.actual).map_err(|e| invalid(*id, e.to_string()))?;
            let expected =
                dsl::parse_expr(&expectation.expected).map_err(|e| invalid(*id, e.to_string()))?;
            let actual = infer(&actual, &env, p, context, *id, None)?;
            if infer(&expected, &env, p, context, *id, Some(&actual))? != actual {
                return Err(invalid(*id, "test expectation types differ"));
            }
            let tolerance =
                dsl::parse_expr(&expectation.tolerance).map_err(|e| invalid(*id, e.to_string()))?;
            let delta = if let Type::Quantity(s) = &actual {
                Type::Quantity(Scheme::Delta(Box::new(s.clone())))
            } else {
                return Err(invalid(*id, "test expectation requires quantities"));
            };
            let Type::Quantity(delta) = delta else {
                return Err(invalid(*id, "test tolerance must be physical"));
            };
            let delta = Type::Quantity(Scheme::Concrete(
                delta
                    .resolve_with_evidence(
                        context.quantities,
                        &Substitution::new(),
                        context.preconditions,
                    )
                    .map_err(|e| invalid(*id, e.to_string()))?,
            ));
            if infer(&tolerance, &env, p, context, *id, Some(&delta))? != delta {
                return Err(invalid(*id, "expectation tolerance type differs"));
            }
            if let Some(relative) = &expectation.relative_tolerance {
                let relative =
                    dsl::parse_expr(relative).map_err(|e| invalid(*id, e.to_string()))?;
                let scalar = Type::Quantity(Scheme::Concrete(
                    context.quantities.neutral_dimensionless().ok_or_else(|| {
                        invalid(*id, "relative tolerance requires a neutral scalar type")
                    })?,
                ));
                if infer(&relative, &env, p, context, *id, Some(&scalar))? != scalar {
                    return Err(invalid(*id, "relative tolerance must be dimensionless"));
                }
            }
        }
        let check_eq = |source: &str| -> Result<()> {
            let e = dsl::parse_equation(source).map_err(|e| invalid(*id, e.to_string()))?;
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
            eq(&e, &env, p, context, *id)
        };
        if let Some(e) = &row.value.equation {
            check_eq(&e.expression)?;
        }
        if let Some(e) = row
            .value
            .binding
            .as_ref()
            .and_then(|b| b.defined_by.as_ref())
        {
            check_eq(e)?;
        }
        for source in row
            .value
            .guard
            .as_ref()
            .map(|g| &g.predicate)
            .into_iter()
            .chain(row.value.requirement.as_ref().map(|r| &r.predicate))
        {
            let pred = dsl::parse_predicate(source).map_err(|e| invalid(*id, e.to_string()))?;
            predicate(&pred, &env, p, context, *id)?;
        }
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
    if path.segments.iter().all(|s| s.indices.is_empty())
        && let Some(ty) = env.get(&qualified)
    {
        // A declared indexed member still requires coordinates; lexical values do not.
        if !p.resolve(at, &qualified).is_some_and(|id| {
            p.declarations[&id]
                .value
                .binding
                .as_ref()
                .is_some_and(|b| !b.indices.is_empty())
        }) {
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
        let declaration = p.resolve(at, &prefix);
        if let Some(ty) = env
            .get(&prefix)
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
            for (index, expected) in segment.indices.iter().zip(&table.keys) {
                if infer(index, env, p, c, at, Some(expected))? != *expected {
                    return Err(invalid(at, "table key type"));
                }
            }
            ty = if table.optional {
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
                let actual = infer(index, env, p, c, at, None)?;
                if !matches!(actual,Type::Entity(id)|Type::Enum(id) if id==*kind) {
                    return Err(invalid(at, "index kind differs"));
                }
            }
            ty = *element.clone();
            declaration = None;
        } else if let Some(id) = declaration {
            let indices = p.declarations[&id]
                .value
                .binding
                .as_ref()
                .map(|b| {
                    b.indices
                        .iter()
                        .map(|i| (&i.name, &i.domain))
                        .collect::<Vec<_>>()
                })
                .or_else(|| {
                    p.declarations[&id]
                        .value
                        .accumulator
                        .as_ref()
                        .map(|a| a.indices.iter().map(|i| (&i.name, &i.domain)).collect())
                })
                .unwrap_or_default();
            if segment.indices.is_empty() && !indices.is_empty() {
                let mut axes = Vec::new();
                let mut local = declaration_environment(p, c, id)?;
                for (name, domain) in &indices {
                    let domain = dsl::parse_expr(domain).map_err(|e| invalid(id, e.to_string()))?;
                    let (Type::Set(element) | Type::Continuous(_, element)) =
                        infer(&domain, &local, p, c, id, None)?
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
            if indices.len() != segment.indices.len() {
                return Err(invalid(at, "member index arity"));
            }
            let mut local = declaration_environment(p, c, id)?;
            for (index, (name, domain)) in segment.indices.iter().zip(indices) {
                let domain = dsl::parse_expr(domain).map_err(|e| invalid(id, e.to_string()))?;
                let (Type::Set(element) | Type::Continuous(_, element)) =
                    infer(&domain, &local, p, c, id, None)?
                else {
                    return Err(invalid(id, "index domain must be a set"));
                };
                if infer(index, env, p, c, at, Some(&element))? != *element {
                    return Err(invalid(at, "index member kind differs"));
                }
                local.insert(name.clone(), *element);
            }
        } else if !segment.indices.is_empty() {
            return Err(invalid(at, "scalar member cannot be indexed"));
        }
    }
    Ok(ty)
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
                .is_some_and(|e| e.members.iter().any(|m| m == name)) =>
        {
            Ok((Type::Enum(*id), None))
        }
        Type::Row(id) => p
            .tables
            .get(id)
            .and_then(|t| t.columns.iter().find(|(n, _)| n == name))
            .map(|(_, ty)| (ty.clone(), None))
            .ok_or_else(|| invalid(at, "unknown table column")),
        Type::Entity(id) | Type::Definition(id) | Type::Interface(id) => {
            let member = p
                .declared_member(*id, name)
                .ok_or_else(|| invalid(at, format!("type has no member {name}")))?;
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
                if name == "present"
                    && args.len() == 1
                    && let ExprKind::Path(path) = &args[0].kind
                    && let Type::Optional(inner) = infer(&args[0], &env, p, c, at, None)?
                {
                    env.insert(dsl::render_path(path), *inner);
                }
                if name == "implements"
                    && args.len() == 2
                    && let (ExprKind::Path(target), ExprKind::Path(contract)) =
                        (&args[0].kind, &args[1].kind)
                {
                    let id = p
                        .resolve(at, &dsl::render_path(contract))
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
    for (source, expected) in [
        (&members.weight, &scalar),
        (&members.normalization, target),
        (&members.absolute_tolerance, &difference),
        (&members.relative_tolerance, &scalar),
    ] {
        if let Some(source) = source {
            let expression = dsl::parse_expr(source).map_err(|e| invalid(id, e.to_string()))?;
            if infer(&expression, env, p, context, id, Some(expected))? != *expected {
                return Err(invalid(id, "objective member type differs"));
            }
        }
    }
    Ok(())
}
