// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordered finite folds issue shared value bindings during the one specialization rewrite.
use super::*;
use crate::resolved::{
    Binding, Carrier, CarrierKind, Node, Reference, Source, construction::Rewritten,
};

impl Engine<'_, '_> {
    pub(super) fn fold(
        &mut self,
        instance: InstanceId,
        expression: &Expr,
        env: &Environment,
        chain: &[DeclarationId],
        source: Source,
    ) -> Result<Rewritten> {
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
        let types = self.source_types(at, env)?;
        let ty = crate::expression::infer(expression, &types, self.p, self.c, at, None)?;
        let domain = crate::expression::infer(
            &Expr {
                kind: ExprKind::Path(binder.domain.clone()),
                span: Span::default(),
            },
            &types,
            self.p,
            self.c,
            at,
            None,
        )?;
        let Type::Set(element) = domain else {
            return Err(invalid(
                at,
                "fold requires its declared finite carrier type",
            ));
        };
        let Value::Set(members) = self.eval_ast_with(
            at,
            env,
            &Expr {
                kind: ExprKind::Path(binder.domain.clone()),
                span: Span::default(),
            },
            None,
        )?
        else {
            return Err(invalid(at, "fold requires finite membership"));
        };
        let carrier = self.construction.carrier(Carrier {
            kind: CarrierKind::OrderedSet,
            element: *element,
            members: members.clone().into(),
        })?;
        let saved = self.lexical.clone();
        let saved_references = self.lexical_references.clone();
        let saved_types = self.function_types.clone();
        self.lexical.remove(&binder.var);
        self.lexical_references.remove(&binder.var);
        let result = (|| {
            let mut inspection = Vec::new();
            let mut bindings = Vec::new();
            let mut selected = Vec::new();
            let mut previous = None;
            for member in members {
                self.reserve(2)?;
                let mut local = env.clone();
                local.insert(binder.var.clone(), member.clone());
                if let Some(filter) = &binder.filter
                    && !(Evaluator {
                        lexical_bases: Some(&self.lexical_bases),
                        package: self.p,
                        physical: self.c,
                        at,
                        env: &local,
                        limit: self.limits.members,
                        stack: Vec::new(),
                        reader: self.reader,
                        selections: Some(&self.selection_collector),
                    })
                    .predicate(filter)?
                {
                    continue;
                }
                selected.push(member);
                self.lexical = saved.clone();
                self.lexical.remove(&binder.var);
                self.lexical_references = saved_references.clone();
                self.lexical_references.remove(&binder.var);
                self.function_types = saved_types.clone();
                let current = self.rewrite(instance, value, &local, chain)?;
                let current_binding = self.construction.binding(Binding::Value(current.root))?;
                bindings.push(current_binding);
                self.local_serial += 1;
                let current_name = format!("__fold_value_{}", self.local_serial);
                inspection.push((current_name.clone(), current.inspection));
                let current = path_expression(&current_name);
                let next = if let Some((prior_binding, prior)) = previous {
                    self.function_types.insert(accumulator.clone(), ty.clone());
                    self.function_types.insert(item.clone(), ty.clone());
                    self.lexical.insert(accumulator.clone(), prior);
                    self.lexical.insert(item.clone(), current);
                    self.lexical_references
                        .insert(accumulator.clone(), Reference::Lexical(prior_binding));
                    self.lexical_references
                        .insert(item.clone(), Reference::Lexical(current_binding));
                    let next = self.rewrite(instance, step, &local, chain)?;
                    let binding = self.construction.binding(Binding::Value(next.root))?;
                    bindings.push(binding);
                    self.local_serial += 1;
                    let name = format!("__fold_step_{}", self.local_serial);
                    inspection.push((name.clone(), next.inspection));
                    (binding, path_expression(&name))
                } else {
                    (current_binding, current)
                };
                previous = Some(next);
            }
            let (binding, body) = previous
                .ok_or_else(|| invalid(at, "fold requires at least one selected member"))?;
            let body_root = self
                .construction
                .node(Node::Reference(Reference::Lexical(binding)), source.clone())?;
            let result = self.construction.pair(
                Node::Let {
                    bindings,
                    body: body_root,
                },
                ExprKind::Let {
                    bindings: inspection,
                    body: Box::new(body),
                },
                source,
            )?;
            self.construction
                .finite_fold(result.root, carrier, selected, ty)?;
            Ok(result)
        })();
        self.lexical = saved;
        self.lexical_references = saved_references;
        self.function_types = saved_types;
        result
    }
}
fn path_expression(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Path(Path::single(name)),
        span: Span::default(),
    }
}
