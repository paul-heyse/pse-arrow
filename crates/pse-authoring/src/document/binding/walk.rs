// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{BoundPath, Context, ParsedExpression, PathMeaning, paths};
use crate::document::load::contract;
use crate::{
    AuthoringError, SourceSpan,
    dsl::{self, EquationKind, ExprKind, PredicateKind},
};
use pse_ids::SemanticId;
use std::collections::BTreeSet;

pub(super) fn bind(
    expression: &ParsedExpression,
    owner: SemanticId,
    context: &Context,
    at: SourceSpan,
    indexed_by: &[String],
) -> Result<Vec<BoundPath>, AuthoringError> {
    let mut walker = Walker {
        owner,
        context,
        at,
        paths: Vec::new(),
        locals: indexed_by.iter().cloned().collect(),
    };
    if walker.locals.len() != indexed_by.len() {
        return Err(contract(Some(at), "duplicate expression index variables"));
    }
    for name in indexed_by {
        if context
            .targets
            .template_domains
            .iter()
            .filter(|domain| domain.template_id == owner && domain.name == *name)
            .count()
            != 1
        {
            return Err(contract(
                Some(at),
                "expression index variable requires its declared template domain",
            ));
        }
    }
    match expression {
        ParsedExpression::Expr(value) => walker.expr(value)?,
        ParsedExpression::Equation(value) => walker.equation(value)?,
        ParsedExpression::Predicate(value) => walker.predicate(value)?,
    }
    Ok(walker.paths)
}
struct Walker<'a> {
    owner: SemanticId,
    context: &'a Context,
    at: SourceSpan,
    paths: Vec<BoundPath>,
    locals: BTreeSet<String>,
}
impl Walker<'_> {
    fn path(&mut self, path: &dsl::Path, span: dsl::Span) -> Result<(), AuthoringError> {
        self.paths.push(paths::bind(
            path,
            span,
            self.owner,
            self.context,
            &self.locals,
            self.at,
        )?);
        for segment in &path.segments {
            for index in &segment.indices {
                self.expr(index)?;
            }
        }
        Ok(())
    }
    fn expr(&mut self, expr: &dsl::Expr) -> Result<(), AuthoringError> {
        match &expr.kind {
            ExprKind::Number(_) => {}
            ExprKind::Path(path) => self.path(path, expr.span)?,
            ExprKind::Neg(expr) => self.expr(expr)?,
            ExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs)?;
                self.expr(rhs)?;
            }
            ExprKind::Call {
                function: dsl::Function::Convert,
                args,
                ..
            } => {
                self.expr(&args[0])?;
                self.unit(&args[1])?;
            }
            ExprKind::Call { args, named, .. } => {
                for expr in args {
                    self.expr(expr)?;
                }
                for arg in named {
                    self.expr(&arg.value)?;
                }
            }
            ExprKind::Kernel { args, .. } => {
                for expr in args {
                    self.expr(expr)?;
                }
            }
            ExprKind::Reduce { binder, body, .. } => {
                self.path(&binder.domain, expr.span)?;
                let prior = self.locals.clone();
                self.locals.insert(binder.var.clone());
                if let Some(filter) = &binder.filter {
                    self.predicate(filter)?;
                }
                self.expr(body)?;
                self.locals = prior;
            }
            ExprKind::Derivative { body, wrt } => {
                self.expr(body)?;
                self.path(wrt, expr.span)?;
            }
            ExprKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                self.predicate(guard)?;
                self.expr(then)?;
                self.expr(otherwise)?;
            }
            ExprKind::Let { bindings, body } => {
                let prior = self.locals.clone();
                let mut seen = BTreeSet::new();
                for (name, expr) in bindings {
                    if !seen.insert(name) {
                        return Err(contract(Some(self.at), "duplicate let binding"));
                    }
                    self.expr(expr)?;
                    self.locals.insert(name.clone());
                }
                self.expr(body)?;
                self.locals = prior;
            }
        }
        Ok(())
    }
    fn unit(&mut self, expr: &dsl::Expr) -> Result<(), AuthoringError> {
        match &expr.kind {
            ExprKind::Path(path) => {
                let [part] = path.segments.as_slice() else {
                    return Err(contract(Some(self.at), "unit symbol must have one segment"));
                };
                if !part.indices.is_empty() {
                    return Err(contract(
                        Some(self.at),
                        "unit symbol cannot have subscripts",
                    ));
                }
                let matches = self
                    .context
                    .units
                    .iter()
                    .filter(|unit| unit.symbol == part.name)
                    .collect::<Vec<_>>();
                let [unit] = matches.as_slice() else {
                    return Err(contract(
                        Some(self.at),
                        "conversion target unit symbol is missing or ambiguous",
                    ));
                };
                self.paths.push(BoundPath {
                    span: expr.span,
                    meaning: PathMeaning::Unit {
                        unit_id: unit.unit_id,
                    },
                    segment_entities: vec![None],
                });
            }
            ExprKind::Number(number) if number.unit.is_none() => {}
            ExprKind::Neg(value) => self.unit(value)?,
            ExprKind::Binary {
                op: dsl::BinaryOp::Mul | dsl::BinaryOp::Div | dsl::BinaryOp::Pow,
                lhs,
                rhs,
            } => {
                self.unit(lhs)?;
                self.unit(rhs)?;
            }
            _ => {
                return Err(contract(
                    Some(self.at),
                    "conversion target requires a declared unit expression",
                ));
            }
        }
        Ok(())
    }
    fn predicate(&mut self, predicate: &dsl::Predicate) -> Result<(), AuthoringError> {
        match &predicate.kind {
            PredicateKind::Compare { op, lhs, rhs } => self.compare(*op, lhs, rhs)?,
            PredicateKind::In { expr, domain } => {
                if self.operand_enum(expr)?.is_some() {
                    return Err(contract(
                        Some(self.at),
                        "enum-valued scalar has no declared domain membership mapping",
                    ));
                }
                self.expr(expr)?;
                self.path(domain, predicate.span)?;
            }
            PredicateKind::And(lhs, rhs) | PredicateKind::Or(lhs, rhs) => {
                self.predicate(lhs)?;
                self.predicate(rhs)?;
            }
            PredicateKind::Not(value) => self.predicate(value)?,
            PredicateKind::Atom(value) => self.expr(value)?,
            PredicateKind::Bool(_) | PredicateKind::Null => {}
        }
        Ok(())
    }
    fn equation(&mut self, equation: &dsl::Equation) -> Result<(), AuthoringError> {
        match &equation.kind {
            EquationKind::Relation { lhs, rhs, .. } => {
                self.expr(lhs)?;
                self.expr(rhs)?;
            }
            EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                self.predicate(guard)?;
                self.equation(then)?;
                self.equation(otherwise)?;
            }
        }
        Ok(())
    }
}

impl Walker<'_> {
    fn operand_enum(&self, expr: &dsl::Expr) -> Result<Option<SemanticId>, AuthoringError> {
        let ExprKind::Path(path) = &expr.kind else {
            return Ok(None);
        };
        if path
            .segments
            .iter()
            .any(|segment| !segment.indices.is_empty())
        {
            return Ok(None);
        }
        // This probe supplies context only. Every operand is independently admitted
        // below, so failed or ambiguous ordinary binding is never accepted by the probe.
        let Ok(binding) = paths::bind(
            path,
            expr.span,
            self.owner,
            self.context,
            &self.locals,
            self.at,
        ) else {
            return Ok(None);
        };
        let identity = match binding.meaning {
            PathMeaning::Feature { template_id, name } => self
                .context
                .features
                .iter()
                .find(|row| row.template_id == template_id && row.name == name)
                .and_then(|row| row.enum_id),
            PathMeaning::Parameter { template_id, name } => self
                .context
                .params
                .iter()
                .find(|row| row.template_id == template_id && row.name == name)
                .and_then(|row| row.enum_id),
            _ => None,
        };
        if identity.is_some_and(|id| !self.context.enums.contains_key(&id)) {
            return Err(contract(
                Some(self.at),
                "operand enum_id is not a registered declaration",
            ));
        }
        Ok(identity)
    }
    fn compare(
        &mut self,
        op: dsl::CompareOp,
        lhs: &dsl::Expr,
        rhs: &dsl::Expr,
    ) -> Result<(), AuthoringError> {
        let left = self.operand_enum(lhs)?;
        let right = self.operand_enum(rhs)?;
        if left.is_some() && right.is_some() && left != right {
            return Err(contract(
                Some(self.at),
                "compared operands declare different enums",
            ));
        }
        let Some(identity) = left.or(right) else {
            self.expr(lhs)?;
            return self.expr(rhs);
        };
        if !matches!(op, dsl::CompareOp::Eq | dsl::CompareOp::NotEq) {
            return Err(contract(Some(self.at), "enum ordering is not declared"));
        }
        self.enum_operand(lhs, identity)?;
        self.enum_operand(rhs, identity)
    }
    fn enum_operand(
        &mut self,
        expr: &dsl::Expr,
        enum_id: SemanticId,
    ) -> Result<(), AuthoringError> {
        if let ExprKind::Path(path) = &expr.kind
            && let [segment] = path.segments.as_slice()
            && segment.indices.is_empty()
            && self
                .context
                .enums
                .get(&enum_id)
                .is_some_and(|members| members.contains(&segment.name))
        {
            if !paths::unbound_bare(
                &segment.name,
                self.owner,
                self.context,
                &self.locals,
                self.at,
            )? {
                return Err(contract(
                    Some(self.at),
                    "enum member spelling also resolves as a declaration or lexical value",
                ));
            }
            self.paths.push(BoundPath {
                span: expr.span,
                meaning: PathMeaning::EnumLiteral {
                    enum_id,
                    member: segment.name.clone(),
                },
                segment_entities: vec![None],
            });
            return Ok(());
        }
        if self.operand_enum(expr)? != Some(enum_id) {
            return Err(contract(
                Some(self.at),
                "operand is neither a member nor a value of the exact expected enum",
            ));
        }
        self.expr(expr)
    }
}
