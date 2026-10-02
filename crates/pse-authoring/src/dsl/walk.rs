// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Structural traversal and equality use the actual syntax tree, never fingerprints.

use super::ast::{Equation, EquationKind, Expr, ExprKind, Path, Predicate, PredicateKind, Span};

impl Expr {
    /// Visit this expression and every expression child in source order.
    pub fn walk(&self, mut visitor: impl FnMut(&Self)) {
        walk_expr(self, &mut visitor);
    }
    /// Transform every expression child, then its parent, without rendering or reparsing.
    /// # Errors
    /// The visitor's first failure is returned unchanged.
    pub fn try_walk_mut<E>(
        &mut self,
        mut visitor: impl FnMut(&mut Self) -> Result<(), E>,
    ) -> Result<(), E> {
        mutate_expr(self, &mut visitor)
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
    /// Visit every expression in the predicate, including calls and indexed coordinates.
    pub fn walk_expressions(&self, mut visitor: impl FnMut(&Expr)) {
        walk_predicate(self, &mut visitor);
    }
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
        ExprKind::Call { args, .. } => {
            for arg in args {
                walk_expr(arg, visitor);
            }
        }
        ExprKind::NamedCall { name, args } => {
            for segment in &name.segments {
                for index in &segment.indices {
                    walk_expr(index, visitor);
                }
            }
            for arg in args {
                walk_expr(arg, visitor);
            }
        }
        ExprKind::Kernel { args, .. } => {
            for arg in args {
                walk_expr(arg, visitor);
            }
        }
        ExprKind::Partial {
            function,
            args,
            wrt,
        } => {
            for segment in &function.segments {
                for index in &segment.indices {
                    walk_expr(index, visitor);
                }
            }
            for a in args {
                walk_expr(a, visitor);
            }
            for p in wrt {
                walk_path(p, visitor);
            }
        }
        ExprKind::Reduce { binder, body, .. } => {
            walk_path(&binder.domain, visitor);
            if let Some(filter) = &binder.filter {
                walk_predicate(filter, visitor);
            }
            walk_expr(body, visitor);
        }
        ExprKind::Fold {
            binder,
            value,
            step,
            ..
        } => {
            walk_path(&binder.domain, visitor);
            if let Some(filter) = &binder.filter {
                walk_predicate(filter, visitor);
            }
            walk_expr(value, visitor);
            walk_expr(step, visitor);
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
        ExprKind::Call { args, .. } => {
            for arg in args {
                strip_expr(arg);
            }
        }
        ExprKind::NamedCall { name, args } => {
            for segment in &mut name.segments {
                for index in &mut segment.indices {
                    strip_expr(index);
                }
            }
            for arg in args {
                strip_expr(arg);
            }
        }
        ExprKind::Kernel { args, .. } => {
            for arg in args {
                strip_expr(arg);
            }
        }
        ExprKind::Partial {
            function,
            args,
            wrt,
        } => {
            for segment in &mut function.segments {
                for index in &mut segment.indices {
                    strip_expr(index);
                }
            }
            for a in args {
                strip_expr(a);
            }
            for p in wrt {
                strip_path(p);
            }
        }
        ExprKind::Reduce { binder, body, .. } => {
            strip_path(&mut binder.domain);
            if let Some(filter) = &mut binder.filter {
                strip_predicate(filter);
            }
            strip_expr(body);
        }
        ExprKind::Fold {
            binder,
            value,
            step,
            ..
        } => {
            strip_path(&mut binder.domain);
            if let Some(filter) = &mut binder.filter {
                strip_predicate(filter);
            }
            strip_expr(value);
            strip_expr(step);
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
        ExprKind::Call { args, .. } => {
            for arg in args {
                paths_expr(arg, paths);
            }
        }
        ExprKind::NamedCall { name, args } => {
            paths_path(name, paths);
            for arg in args {
                paths_expr(arg, paths);
            }
        }
        ExprKind::Kernel { args, .. } => {
            for arg in args {
                paths_expr(arg, paths);
            }
        }
        ExprKind::Partial {
            function,
            args,
            wrt,
        } => {
            paths_path(function, paths);
            for a in args {
                paths_expr(a, paths);
            }
            for p in wrt {
                for segment in &p.segments {
                    for i in &segment.indices {
                        paths_expr(i, paths);
                    }
                }
            }
        }
        ExprKind::Reduce { binder, body, .. } => {
            paths_path(&binder.domain, paths);
            if let Some(filter) = &binder.filter {
                paths_predicate(filter, paths);
            }
            paths_expr(body, paths);
        }
        ExprKind::Fold {
            binder,
            value,
            step,
            ..
        } => {
            paths_path(&binder.domain, paths);
            if let Some(filter) = &binder.filter {
                paths_predicate(filter, paths);
            }
            paths_expr(value, paths);
            paths_expr(step, paths);
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

fn mutate_path<E>(p: &mut Path, f: &mut impl FnMut(&mut Expr) -> Result<(), E>) -> Result<(), E> {
    for segment in &mut p.segments {
        for index in &mut segment.indices {
            mutate_expr(index, f)?;
        }
    }
    Ok(())
}
fn mutate_predicate<E>(
    p: &mut Predicate,
    f: &mut impl FnMut(&mut Expr) -> Result<(), E>,
) -> Result<(), E> {
    match &mut p.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            mutate_expr(lhs, f)?;
            mutate_expr(rhs, f)?;
        }
        PredicateKind::In { expr, domain } => {
            mutate_expr(expr, f)?;
            mutate_path(domain, f)?;
        }
        PredicateKind::Atom(e) => mutate_expr(e, f)?,
        PredicateKind::And(a, b) | PredicateKind::Or(a, b) => {
            mutate_predicate(a, f)?;
            mutate_predicate(b, f)?;
        }
        PredicateKind::Not(p) => mutate_predicate(p, f)?,
        _ => {}
    }
    Ok(())
}
fn mutate_expr<E>(e: &mut Expr, f: &mut impl FnMut(&mut Expr) -> Result<(), E>) -> Result<(), E> {
    match &mut e.kind {
        ExprKind::Path(p) => mutate_path(p, f)?,
        ExprKind::Number(_) => {}
        ExprKind::Neg(e) => mutate_expr(e, f)?,
        ExprKind::Binary { lhs, rhs, .. } => {
            mutate_expr(lhs, f)?;
            mutate_expr(rhs, f)?;
        }
        ExprKind::NamedCall { name, args } => {
            for segment in &mut name.segments {
                for index in &mut segment.indices {
                    mutate_expr(index, f)?;
                }
            }
            for arg in args {
                mutate_expr(arg, f)?;
            }
        }
        ExprKind::Call { args, .. } | ExprKind::Kernel { args, .. } => {
            for a in args {
                mutate_expr(a, f)?;
            }
        }
        ExprKind::Partial {
            function,
            args,
            wrt,
        } => {
            for segment in &mut function.segments {
                for index in &mut segment.indices {
                    mutate_expr(index, f)?;
                }
            }
            for a in args {
                mutate_expr(a, f)?;
            }
            for p in wrt {
                mutate_path(p, f)?;
            }
        }
        ExprKind::Reduce { binder, body, .. } => {
            mutate_path(&mut binder.domain, f)?;
            if let Some(p) = &mut binder.filter {
                mutate_predicate(p, f)?;
            }
            mutate_expr(body, f)?;
        }
        ExprKind::Fold {
            binder,
            value,
            step,
            ..
        } => {
            mutate_path(&mut binder.domain, f)?;
            if let Some(filter) = &mut binder.filter {
                mutate_predicate(filter, f)?;
            }
            mutate_expr(value, f)?;
            mutate_expr(step, f)?;
        }
        ExprKind::Derivative { body, wrt } => {
            mutate_expr(body, f)?;
            mutate_path(wrt, f)?;
        }
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            mutate_predicate(guard, f)?;
            mutate_expr(then, f)?;
            mutate_expr(otherwise, f)?;
        }
        ExprKind::Let { bindings, body } => {
            for (_, v) in bindings {
                mutate_expr(v, f)?;
            }
            mutate_expr(body, f)?;
        }
    }
    f(e)
}

impl Expr {
    /// Paths that are free in this expression, respecting sequential local bindings.
    /// Includes typed callee paths; partial argument selectors only contribute indices.
    pub fn free_paths(&self) -> Vec<&Path> {
        let mut output = Vec::new();
        free_expr(self, &std::collections::BTreeSet::new(), &mut output);
        output
    }
}
fn free_path<'a>(p: &'a Path, bound: &std::collections::BTreeSet<String>, out: &mut Vec<&'a Path>) {
    if p.segments.first().is_some_and(|s| !bound.contains(&s.name)) {
        out.push(p);
    }
    for s in &p.segments {
        for i in &s.indices {
            free_expr(i, bound, out);
        }
    }
}
fn free_predicate<'a>(
    p: &'a Predicate,
    bound: &std::collections::BTreeSet<String>,
    out: &mut Vec<&'a Path>,
) {
    match &p.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            free_expr(lhs, bound, out);
            free_expr(rhs, bound, out);
        }
        PredicateKind::In { expr, domain } => {
            free_expr(expr, bound, out);
            free_path(domain, bound, out);
        }
        PredicateKind::Atom(e) => free_expr(e, bound, out),
        PredicateKind::And(a, b) | PredicateKind::Or(a, b) => {
            free_predicate(a, bound, out);
            free_predicate(b, bound, out);
        }
        PredicateKind::Not(p) => free_predicate(p, bound, out),
        _ => {}
    }
}
fn free_expr<'a>(e: &'a Expr, bound: &std::collections::BTreeSet<String>, out: &mut Vec<&'a Path>) {
    match &e.kind {
        ExprKind::Path(p) => free_path(p, bound, out),
        ExprKind::Number(_) => {}
        ExprKind::Neg(e) => free_expr(e, bound, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            free_expr(lhs, bound, out);
            free_expr(rhs, bound, out);
        }
        ExprKind::NamedCall { name, args } => {
            free_path(name, bound, out);
            for arg in args {
                free_expr(arg, bound, out);
            }
        }
        ExprKind::Call { args, .. } | ExprKind::Kernel { args, .. } => {
            for a in args {
                free_expr(a, bound, out);
            }
        }
        ExprKind::Partial {
            function,
            args,
            wrt,
        } => {
            free_path(function, bound, out);
            for a in args {
                free_expr(a, bound, out);
            }
            for p in wrt {
                for segment in &p.segments {
                    for i in &segment.indices {
                        free_expr(i, bound, out);
                    }
                }
            }
        }
        ExprKind::Let { bindings, body } => {
            let mut local = bound.clone();
            for (name, value) in bindings {
                free_expr(value, &local, out);
                local.insert(name.clone());
            }
            free_expr(body, &local, out);
        }
        ExprKind::Reduce { binder, body, .. } => {
            free_path(&binder.domain, bound, out);
            let mut local = bound.clone();
            local.insert(binder.var.clone());
            if let Some(p) = &binder.filter {
                free_predicate(p, &local, out);
            }
            free_expr(body, &local, out);
        }
        ExprKind::Fold {
            accumulator,
            item,
            binder,
            value,
            step,
        } => {
            free_path(&binder.domain, bound, out);
            let mut local = bound.clone();
            local.insert(binder.var.clone());
            if let Some(filter) = &binder.filter {
                free_predicate(filter, &local, out);
            }
            free_expr(value, &local, out);
            local.insert(accumulator.clone());
            local.insert(item.clone());
            free_expr(step, &local, out);
        }
        ExprKind::Derivative { body, wrt } => {
            free_expr(body, bound, out);
            free_path(wrt, bound, out);
        }
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            free_predicate(guard, bound, out);
            free_expr(then, bound, out);
            free_expr(otherwise, bound, out);
        }
    }
}
