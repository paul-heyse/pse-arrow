// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Structural traversal and equality use the actual syntax tree, never fingerprints.

use super::ast::{Equation, EquationKind, Expr, ExprKind, Path, Predicate, PredicateKind, Span};

impl Expr {
    /// Visit this expression and every expression child in source order.
    pub fn walk(&self, mut visitor: impl FnMut(&Self)) {
        walk_expr(self, &mut visitor);
    }
    /// Every member/domain path, including paths inside subscripts and predicates.
    pub fn paths(&self) -> Vec<&Path> {
        let mut paths = Vec::new();
        paths_expr(self, &mut paths);
        paths
    }
    /// Compare the complete structure, ignoring spans and retaining exact float bits.
    pub fn structural_eq(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        left.strip_spans();
        right.strip_spans();
        left == right
    }
    /// Remove source positions without changing syntax or payload values.
    pub fn strip_spans(&mut self) {
        strip_expr(self);
    }
}

impl Predicate {
    /// Compare syntax and exact literal values while ignoring source positions.
    pub fn structural_eq(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        strip_predicate(&mut left);
        strip_predicate(&mut right);
        left == right
    }
    /// Remove source positions recursively.
    pub fn strip_spans(&mut self) {
        strip_predicate(self);
    }
    /// Every member/domain path referenced by the predicate.
    pub fn paths(&self) -> Vec<&Path> {
        let mut paths = Vec::new();
        paths_predicate(self, &mut paths);
        paths
    }
}

impl Equation {
    /// Compare both sides, senses and conditional branches while ignoring spans.
    pub fn structural_eq(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        left.strip_spans();
        right.strip_spans();
        left == right
    }
    /// Remove source positions recursively.
    pub fn strip_spans(&mut self) {
        self.span = Span::default();
        match &mut self.kind {
            EquationKind::Relation { lhs, rhs, .. } => {
                strip_expr(lhs);
                strip_expr(rhs);
            }
            EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                strip_predicate(guard);
                then.strip_spans();
                otherwise.strip_spans();
            }
        }
    }
    /// Every member/domain path referenced by either side or a branch guard.
    pub fn paths(&self) -> Vec<&Path> {
        let mut paths = Vec::new();
        paths_equation(self, &mut paths);
        paths
    }
}

fn walk_expr(expr: &Expr, visitor: &mut dyn FnMut(&Expr)) {
    visitor(expr);
    match &expr.kind {
        ExprKind::Number(_) => {}
        ExprKind::Path(path) => walk_path(path, visitor),
        ExprKind::Neg(inner) => walk_expr(inner, visitor),
        ExprKind::Binary { lhs, rhs, .. } => {
            walk_expr(lhs, visitor);
            walk_expr(rhs, visitor);
        }
        ExprKind::Call { args, named, .. } => {
            for arg in args {
                walk_expr(arg, visitor);
            }
            for arg in named {
                walk_expr(&arg.value, visitor);
            }
        }
        ExprKind::Kernel { args, .. } => {
            for arg in args {
                walk_expr(arg, visitor);
            }
        }
        ExprKind::Reduce { binder, body, .. } => {
            walk_path(&binder.domain, visitor);
            if let Some(filter) = &binder.filter {
                walk_predicate(filter, visitor);
            }
            walk_expr(body, visitor);
        }
        ExprKind::Derivative { body, wrt } => {
            walk_expr(body, visitor);
            walk_path(wrt, visitor);
        }
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            walk_predicate(guard, visitor);
            walk_expr(then, visitor);
            walk_expr(otherwise, visitor);
        }
        ExprKind::Let { bindings, body } => {
            walk_expr(body, visitor);
            for (_, value) in bindings {
                walk_expr(value, visitor);
            }
        }
    }
}
fn walk_path(path: &Path, visitor: &mut dyn FnMut(&Expr)) {
    for segment in &path.segments {
        for index in &segment.indices {
            walk_expr(index, visitor);
        }
    }
}
fn walk_predicate(predicate: &Predicate, visitor: &mut dyn FnMut(&Expr)) {
    match &predicate.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            walk_expr(lhs, visitor);
            walk_expr(rhs, visitor);
        }
        PredicateKind::In { expr, domain } => {
            walk_expr(expr, visitor);
            walk_path(domain, visitor);
        }
        PredicateKind::And(lhs, rhs) | PredicateKind::Or(lhs, rhs) => {
            walk_predicate(lhs, visitor);
            walk_predicate(rhs, visitor);
        }
        PredicateKind::Not(inner) => walk_predicate(inner, visitor),
        PredicateKind::Atom(expr) => walk_expr(expr, visitor),
        PredicateKind::Bool(_) | PredicateKind::Null => {}
    }
}
fn strip_expr(expr: &mut Expr) {
    expr.span = Span::default();
    match &mut expr.kind {
        ExprKind::Number(_) => {}
        ExprKind::Path(path) => strip_path(path),
        ExprKind::Neg(inner) => strip_expr(inner),
        ExprKind::Binary { lhs, rhs, .. } => {
            strip_expr(lhs);
            strip_expr(rhs);
        }
        ExprKind::Call { args, named, .. } => {
            for arg in args {
                strip_expr(arg);
            }
            for arg in named {
                strip_expr(&mut arg.value);
            }
        }
        ExprKind::Kernel { args, .. } => {
            for arg in args {
                strip_expr(arg);
            }
        }
        ExprKind::Reduce { binder, body, .. } => {
            strip_path(&mut binder.domain);
            if let Some(filter) = &mut binder.filter {
                strip_predicate(filter);
            }
            strip_expr(body);
        }
        ExprKind::Derivative { body, wrt } => {
            strip_expr(body);
            strip_path(wrt);
        }
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            strip_predicate(guard);
            strip_expr(then);
            strip_expr(otherwise);
        }
        ExprKind::Let { bindings, body } => {
            strip_expr(body);
            for (_, value) in bindings {
                strip_expr(value);
            }
        }
    }
}
fn strip_path(path: &mut Path) {
    for segment in &mut path.segments {
        for index in &mut segment.indices {
            strip_expr(index);
        }
    }
}
fn strip_predicate(predicate: &mut Predicate) {
    predicate.span = Span::default();
    match &mut predicate.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            strip_expr(lhs);
            strip_expr(rhs);
        }
        PredicateKind::In { expr, domain } => {
            strip_expr(expr);
            strip_path(domain);
        }
        PredicateKind::And(lhs, rhs) | PredicateKind::Or(lhs, rhs) => {
            strip_predicate(lhs);
            strip_predicate(rhs);
        }
        PredicateKind::Not(inner) => strip_predicate(inner),
        PredicateKind::Atom(expr) => strip_expr(expr),
        PredicateKind::Bool(_) | PredicateKind::Null => {}
    }
}
fn paths_path<'a>(path: &'a Path, paths: &mut Vec<&'a Path>) {
    paths.push(path);
    for segment in &path.segments {
        for index in &segment.indices {
            paths_expr(index, paths);
        }
    }
}
fn paths_expr<'a>(expr: &'a Expr, paths: &mut Vec<&'a Path>) {
    match &expr.kind {
        ExprKind::Number(_) => {}
        ExprKind::Path(path) => paths_path(path, paths),
        ExprKind::Neg(inner) => paths_expr(inner, paths),
        ExprKind::Binary { lhs, rhs, .. } => {
            paths_expr(lhs, paths);
            paths_expr(rhs, paths);
        }
        ExprKind::Call { args, named, .. } => {
            for arg in args {
                paths_expr(arg, paths);
            }
            for arg in named {
                paths_expr(&arg.value, paths);
            }
        }
        ExprKind::Kernel { args, .. } => {
            for arg in args {
                paths_expr(arg, paths);
            }
        }
        ExprKind::Reduce { binder, body, .. } => {
            paths_path(&binder.domain, paths);
            if let Some(filter) = &binder.filter {
                paths_predicate(filter, paths);
            }
            paths_expr(body, paths);
        }
        ExprKind::Derivative { body, wrt } => {
            paths_expr(body, paths);
            paths_path(wrt, paths);
        }
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            paths_predicate(guard, paths);
            paths_expr(then, paths);
            paths_expr(otherwise, paths);
        }
        ExprKind::Let { bindings, body } => {
            paths_expr(body, paths);
            for (_, value) in bindings {
                paths_expr(value, paths);
            }
        }
    }
}
fn paths_predicate<'a>(predicate: &'a Predicate, paths: &mut Vec<&'a Path>) {
    match &predicate.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            paths_expr(lhs, paths);
            paths_expr(rhs, paths);
        }
        PredicateKind::In { expr, domain } => {
            paths_expr(expr, paths);
            paths_path(domain, paths);
        }
        PredicateKind::And(lhs, rhs) | PredicateKind::Or(lhs, rhs) => {
            paths_predicate(lhs, paths);
            paths_predicate(rhs, paths);
        }
        PredicateKind::Not(inner) => paths_predicate(inner, paths),
        PredicateKind::Atom(expr) => paths_expr(expr, paths),
        PredicateKind::Bool(_) | PredicateKind::Null => {}
    }
}
fn paths_equation<'a>(equation: &'a Equation, paths: &mut Vec<&'a Path>) {
    match &equation.kind {
        EquationKind::Relation { lhs, rhs, .. } => {
            paths_expr(lhs, paths);
            paths_expr(rhs, paths);
        }
        EquationKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            paths_predicate(guard, paths);
            paths_equation(then, paths);
            paths_equation(otherwise, paths);
        }
    }
}
