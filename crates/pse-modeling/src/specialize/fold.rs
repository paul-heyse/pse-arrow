// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite folds become shared lexical bindings; the source supplies every operation.
use super::*;

impl Engine<'_, '_> {
    pub(super) fn fold(
        &mut self,
        instance: InstanceId,
        expression: &Expr,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Expr> {
        let ExprKind::Fold {
            accumulator,
            item,
            binder,
            value,
            step,
        } = &expression.kind
        else {
            return Err(invalid(instance, "fold syntax required"));
        };
        let at = chain
            .last()
            .copied()
            .unwrap_or(self.states[&instance].definition);
        let ty = crate::expression::infer(
            expression,
            &self.source_types(at, env)?,
            self.p,
            self.c,
            at,
            None,
        )?;
        let Value::Set(members) = self.eval(at, env, &dsl::render_path(&binder.domain), None)?
        else {
            return Err(invalid(at, "fold requires finite membership"));
        };
        let saved = self.lexical.clone();
        let saved_types = self.function_types.clone();
        self.lexical.remove(&binder.var);
        let result = (|| {
            let mut bindings = Vec::new();
            let mut previous = None;
            for member in members {
                self.reserve(2)?;
                let mut local = env.clone();
                local.insert(binder.var.clone(), member);
                if let Some(filter) = &binder.filter
                    && !(Evaluator {
                        package: self.p,
                        physical: self.c,
                        at,
                        env: &local,
                        limit: self.limits.members,
                        stack: Vec::new(),
                    })
                    .predicate(filter)?
                {
                    continue;
                }
                self.lexical = saved.clone();
                self.lexical.remove(&binder.var);
                self.function_types = saved_types.clone();
                let current = self.rewrite(instance, value, &local, chain)?;
                self.local_serial += 1;
                let current_name = format!("__fold_value_{}", self.local_serial);
                bindings.push((current_name.clone(), current));
                let current = path_expression(&current_name);
                let next = if let Some(prior) = previous {
                    self.function_types.insert(accumulator.clone(), ty.clone());
                    self.function_types.insert(item.clone(), ty.clone());
                    self.lexical.insert(accumulator.clone(), prior);
                    self.lexical.insert(item.clone(), current);
                    let next = self.rewrite(instance, step, &local, chain)?;
                    self.local_serial += 1;
                    let name = format!("__fold_step_{}", self.local_serial);
                    bindings.push((name.clone(), next));
                    path_expression(&name)
                } else {
                    current
                };
                previous = Some(next);
            }
            let body = previous
                .ok_or_else(|| invalid(at, "fold requires at least one selected member"))?;
            Ok(Expr {
                kind: ExprKind::Let {
                    bindings,
                    body: Box::new(body),
                },
                span: Span::default(),
            })
        })();
        self.lexical = saved;
        self.function_types = saved_types;
        result
    }
}
fn path_expression(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Path(Path {
            segments: vec![PathSegment {
                name: name.into(),
                indices: vec![],
            }],
        }),
        span: Span::default(),
    }
}
