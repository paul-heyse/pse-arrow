// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Reusable finite bodies have local coordinates; instance bindings retain semantic identities.
use super::*;
use std::sync::Arc;
/// Normalized finite body shared by instances with the same implementation and structure.
#[derive(Clone, Debug, PartialEq)]
pub struct DispatchBody {
    /// Local coordinate physical types and authored roles; external captures are explicit inputs.
    pub coordinates: Vec<(Type, Kind)>,
    /// Local expression members, indexed into `coordinates`.
    pub expressions: BTreeMap<usize, Expr>,
    /// Local equation rows in declaration/coordinate order.
    pub equations: Vec<Equation>,
}
fn local_name(slot: usize) -> String {
    format!("local_{slot}")
}
fn rename(expr: &mut Expr, names: &BTreeMap<String, String>) -> Result<()> {
    expr.try_walk_mut(|e| {
        if let ExprKind::Path(path) = &mut e.kind
            && path.segments.len() == 1
            && path.segments[0].indices.is_empty()
            && let Some(name) = names.get(&path.segments[0].name)
        {
            path.segments[0].name = name.clone();
        }
        Ok(())
    })?;
    expr.strip_spans();
    Ok(())
}
fn rename_equation(e: &mut Equation, names: &BTreeMap<String, String>) -> Result<()> {
    match &mut e.kind {
        EquationKind::Relation { lhs, rhs, .. } => {
            rename(lhs, names)?;
            rename(rhs, names)?;
        }
        EquationKind::Conditional { .. } => {
            return Err(invalid(
                SemanticId::NIL,
                "dispatch body retains a structural equation guard",
            ));
        }
    }
    e.strip_spans();
    Ok(())
}
impl Engine<'_, '_> {
    pub(super) fn group_bodies(&mut self) -> Result<()> {
        for (id, state) in &self.states {
            self.checkpoint()?;
            let own = state
                .symbols
                .values()
                .copied()
                .chain(
                    self.model
                        .symbols
                        .values()
                        .filter(|s| {
                            s.lineage.instance == *id && !state.symbols.values().any(|v| *v == s.id)
                        })
                        .map(|s| s.id),
                )
                .collect::<Vec<_>>();
            let mut coordinates = own.clone();
            let mut known = own.iter().copied().collect::<BTreeSet<_>>();
            let mut rows = self
                .model
                .equations
                .iter()
                .filter(|r| r.lineage.instance == *id)
                .collect::<Vec<_>>();
            rows.sort_by_key(|r| (r.lineage.declaration, r.id));
            let paths = own
                .iter()
                .filter_map(|id| self.model.symbols[id].expression.as_ref())
                .flat_map(|e| e.free_paths())
                .chain(rows.iter().flat_map(|r| r.equation.paths()));
            for path in paths {
                if path.segments.len() == 1
                    && let Some(raw) = path.segments[0].name.strip_prefix("s_")
                    && let Ok(symbol) = SemanticId::parse_hex(raw)
                    && known.insert(symbol)
                {
                    coordinates.push(symbol);
                }
            }
            let names = coordinates
                .iter()
                .enumerate()
                .map(|(slot, id)| (symbol_name(*id), local_name(slot)))
                .collect::<BTreeMap<_, _>>();
            let mut body = DispatchBody {
                coordinates: Vec::new(),
                expressions: BTreeMap::new(),
                equations: Vec::new(),
            };
            for (slot, symbol) in coordinates.iter().enumerate() {
                let symbol = &self.model.symbols[symbol];
                body.coordinates.push((
                    symbol.ty.clone(),
                    if slot < own.len() {
                        symbol.role
                    } else {
                        Kind::Parameter
                    },
                ));
                if slot < own.len()
                    && let Some(expression) = &symbol.expression
                {
                    let mut expression = expression.clone();
                    rename(&mut expression, &names)?;
                    body.expressions.insert(slot, expression);
                }
            }
            for row in &rows {
                let mut equation = row.equation.clone();
                rename_equation(&mut equation, &names)?;
                body.equations.push(equation);
            }
            let mut h = FramedHasher::new(pse_ids::Frame::ModelingDispatchBodyV4);
            h.id(&state.definition.as_id())
                .u64(body.coordinates.len() as u64);
            for (ty, role) in &body.coordinates {
                h.str(role.as_str());
                let Some(scheme) = ty.quantity_scheme() else {
                    return Err(invalid(
                        state.definition,
                        "finite body coordinate is not physical",
                    ));
                };
                let quantity = scheme
                    .resolve_contract_with_evidence(
                        self.c.quantities,
                        &BTreeMap::new(),
                        self.c.preconditions,
                    )
                    .map_err(|e| invalid(state.definition, e.to_string()))?;
                quantity.frame(&mut h);
                if let Some(refinement) = ty.physical_refinement() {
                    h.u64(1);
                    refinement.frame(&mut h);
                } else {
                    h.u64(0);
                }
            }
            h.u64(body.expressions.len() as u64);
            for (slot, expression) in &body.expressions {
                h.u64(*slot as u64).str(&dsl::render_expr(expression));
            }
            h.u64(body.equations.len() as u64);
            for equation in &body.equations {
                h.str(&dsl::render_equation(equation));
            }
            // A selected function's meaning is part of every body that calls it.
            let mut pending = BTreeSet::new();
            let mut calls = |e: &Expr| {
                e.walk(|e| match &e.kind {
                    ExprKind::NamedCall { name, .. } | ExprKind::Partial { function: name, .. } => {
                        pending.insert(name.clone());
                    }
                    _ => {}
                });
            };
            for e in body.expressions.values() {
                calls(e);
            }
            for e in &body.equations {
                if let EquationKind::Relation { lhs, rhs, .. } = &e.kind {
                    calls(lhs);
                    calls(rhs);
                }
            }
            let mut visited = BTreeSet::new();
            while let Some(name) = pending.pop_first() {
                if !visited.insert(name.clone()) {
                    continue;
                }
                if let Some(function) = self.model.functions.get(&name) {
                    h.id(&function.id.as_id());
                    if let Some(expression) = &function.body {
                        h.str(&dsl::render_expr(expression));
                        expression.walk(|e| match &e.kind {
                            ExprKind::NamedCall { name, .. }
                            | ExprKind::Partial { function: name, .. } => {
                                pending.insert(name.clone());
                            }
                            _ => {}
                        });
                    }
                }
            }
            let key = h.finish_hash();
            if let Some(group) = self.model.groups.get(&key)
                && group.body.as_ref() != &body
            {
                return Err(invalid(state.definition, "dispatch identity collision"));
            }
            let instance = self
                .model
                .instances
                .get_mut(id)
                .ok_or_else(|| invalid(*id, "dispatch instance missing"))?;
            instance.group = key;
            instance.coordinates = coordinates;
            instance.rows = rows.iter().map(|r| r.id).collect();
            self.model
                .groups
                .entry(key)
                .or_insert_with(|| DispatchGroup {
                    key,
                    definition: state.definition,
                    instances: Vec::new(),
                    body: Arc::new(body),
                })
                .instances
                .push(*id);
        }
        Ok(())
    }
}
impl SpecializedModel {
    /// Project shared normalized bodies through their instance bindings for the sole compiler.
    /// # Errors
    /// Corrupt local coordinates or an inconsistent specialization product.
    pub fn bound_bodies(
        &self,
    ) -> Result<(BTreeMap<SemanticId, Expr>, BTreeMap<SemanticId, Equation>)> {
        let mut expressions = BTreeMap::new();
        let mut equations = BTreeMap::new();
        for instance in self.instances.values() {
            let body = &self
                .groups
                .get(&instance.group)
                .ok_or_else(|| invalid(instance.id, "missing dispatch group"))?
                .body;
            if body.coordinates.len() != instance.coordinates.len()
                || body.equations.len() != instance.rows.len()
            {
                return Err(invalid(instance.id, "dispatch binding arity"));
            }
            let names = instance
                .coordinates
                .iter()
                .enumerate()
                .map(|(slot, id)| (local_name(slot), symbol_name(*id)))
                .collect();
            for (slot, expression) in &body.expressions {
                let symbol = instance
                    .coordinates
                    .get(*slot)
                    .ok_or_else(|| invalid(instance.id, "dispatch expression coordinate"))?;
                let mut expression = expression.clone();
                rename(&mut expression, &names)?;
                if expressions.insert(*symbol, expression).is_some() {
                    return Err(invalid(*symbol, "duplicate dispatch expression owner"));
                }
            }
            for (id, equation) in instance.rows.iter().zip(&body.equations) {
                let mut equation = equation.clone();
                rename_equation(&mut equation, &names)?;
                if equations.insert(*id, equation).is_some() {
                    return Err(invalid(*id, "duplicate dispatch row owner"));
                }
            }
        }
        Ok((expressions, equations))
    }
}
