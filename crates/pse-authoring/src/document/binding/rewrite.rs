// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{BoundPath, ParsedExpression, SourceExpression};
use crate::document::load::contract;
use crate::{
    AuthoringError,
    dsl::{self, EquationKind, ExprKind, PredicateKind},
};
use pse_ids::SemanticId;

/// Re-render only when a checked identity-bound segment refers to the renamed entity.
/// # Errors
/// The complete bound path inventory must match the parsed traversal exactly.
pub fn rename(
    expression: &SourceExpression,
    entity: SemanticId,
    name: &str,
) -> Result<Option<String>, AuthoringError> {
    let mut parsed = expression.parsed.clone();
    let mut walker = Walker {
        paths: expression.paths.iter(),
        entity,
        name,
        changed: false,
    };
    match &mut parsed {
        ParsedExpression::Expr(value) => walker.expr(value)?,
        ParsedExpression::Equation(value) => walker.equation(value)?,
        ParsedExpression::Predicate(value) => walker.predicate(value)?,
    }
    if walker.paths.next().is_some() {
        return Err(contract(
            Some(expression.source_span),
            "unused checked source bindings",
        ));
    }
    if !walker.changed {
        return Ok(None);
    }
    Ok(Some(match parsed {
        ParsedExpression::Expr(value) => dsl::render_expr(&value),
        ParsedExpression::Equation(value) => dsl::render_equation(&value),
        ParsedExpression::Predicate(value) => dsl::render_predicate(&value),
    }))
}
struct Walker<'a> {
    paths: std::slice::Iter<'a, BoundPath>,
    entity: SemanticId,
    name: &'a str,
    changed: bool,
}
impl Walker<'_> {
    fn path(&mut self, path: &mut dsl::Path) -> Result<(), AuthoringError> {
        let binding = self
            .paths
            .next()
            .ok_or_else(|| contract(None, "missing checked source binding"))?;
        if binding.segment_entities.len() != path.segments.len() {
            return Err(contract(None, "source binding path shape differs"));
        }
        for (part, entity) in path.segments.iter_mut().zip(&binding.segment_entities) {
            if *entity == Some(self.entity) {
                self.name.clone_into(&mut part.name);
                self.changed = true;
            }
        }
        for segment in &mut path.segments {
            for index in &mut segment.indices {
                self.expr(index)?;
            }
        }
        Ok(())
    }
    fn expr(&mut self, expr: &mut dsl::Expr) -> Result<(), AuthoringError> {
        match &mut expr.kind {
            ExprKind::Number(_) => {}
            ExprKind::Path(path) => self.path(path)?,
            ExprKind::Neg(expr) => self.expr(expr)?,
            ExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs)?;
                self.expr(rhs)?;
            }
            ExprKind::Call { args, named, .. } => {
                for expr in args {
                    self.expr(expr)?;
                }
                for arg in named {
                    self.expr(&mut arg.value)?;
                }
            }
            ExprKind::Kernel { args, .. } => {
                for expr in args {
                    self.expr(expr)?;
                }
            }
            ExprKind::Reduce { binder, body, .. } => {
                self.path(&mut binder.domain)?;
                if let Some(filter) = &mut binder.filter {
                    self.predicate(filter)?;
                }
                self.expr(body)?;
            }
            ExprKind::Derivative { body, wrt } => {
                self.expr(body)?;
                self.path(wrt)?;
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
                for (_, expr) in bindings {
                    self.expr(expr)?;
                }
                self.expr(body)?;
            }
        }
        Ok(())
    }
    fn predicate(&mut self, predicate: &mut dsl::Predicate) -> Result<(), AuthoringError> {
        match &mut predicate.kind {
            PredicateKind::Compare { lhs, rhs, .. } => {
                self.expr(lhs)?;
                self.expr(rhs)?;
            }
            PredicateKind::In { expr, domain } => {
                self.expr(expr)?;
                self.path(domain)?;
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
    fn equation(&mut self, equation: &mut dsl::Equation) -> Result<(), AuthoringError> {
        match &mut equation.kind {
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
