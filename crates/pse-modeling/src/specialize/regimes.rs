// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Alternative equations share their implicit owner's variables, physical score and criterion.
use super::*;
use crate::annotation::Annotation;

#[derive(Clone, Debug, PartialEq)]
pub struct Regime {
    pub id: SemanticId,
    pub declaration: DeclarationId,
    pub equations: Vec<Row>,
    pub annotations: Vec<Annotation>,
    pub eligibility: dsl::Predicate,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RegimeSelection {
    pub criterion: Expr,
    pub tolerance: Expr,
    pub quantity: pse_quantity::QuantityTypeId,
    pub alternatives: Vec<Regime>,
}
impl Engine<'_, '_> {
    pub(super) fn regimes(
        &mut self,
        instance: InstanceId,
        scope: &Declaration,
        members: &BTreeMap<String, DeclarationId>,
        env: &Environment,
    ) -> Result<()> {
        let rows = members
            .values()
            .filter(|id| self.p.declarations[id].value.kind == Kind::Regime)
            .map(|id| self.p.declarations[id].clone())
            .collect::<Vec<_>>();
        let selection = scope
            .value
            .scope
            .as_ref()
            .and_then(|s| s.selection.as_ref());
        if rows.is_empty() && selection.is_none() {
            return Ok(());
        }
        let Some(selection) = selection else {
            return Err(invalid(
                scope.declaration_id,
                "implicit alternatives require a selection criterion",
            ));
        };
        if rows.is_empty()
            || self
                .model
                .equations
                .iter()
                .any(|r| r.lineage.instance == instance)
        {
            return Err(invalid(
                scope.declaration_id,
                "selected implicit blocks require alternatives and no common residual equations",
            ));
        }
        let criterion = self.rewrite(
            instance,
            &dsl::parse_expr(&selection.criterion)
                .map_err(|e| invalid(scope.declaration_id, e.to_string()))?,
            env,
            &[scope.declaration_id],
        )?;
        let tolerance = self.rewrite(
            instance,
            &dsl::parse_expr(&selection.tolerance)
                .map_err(|e| invalid(scope.declaration_id, e.to_string()))?,
            env,
            &[scope.declaration_id],
        )?;
        let types = self
            .model
            .symbols
            .iter()
            .map(|(id, s)| (symbol_name(*id), s.ty.clone()))
            .collect();
        let Type::Quantity(quantity) = crate::expression::infer(
            &criterion,
            &types,
            &self.model.function_contracts(self.p),
            self.c,
            scope.declaration_id,
            None,
        )?
        else {
            return Err(invalid(
                scope.declaration_id,
                "physical regime score required",
            ));
        };
        let quantity = quantity
            .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
            .map_err(|e| invalid(scope.declaration_id, e.to_string()))?;
        let mut alternatives = Vec::new();
        for regime in rows {
            self.checkpoint()?;
            let at = regime.declaration_id;
            let children = self.p.children.get(&at).cloned().unwrap_or_default();
            if children.iter().any(|id| {
                !matches!(
                    self.p.declarations[id].value.kind,
                    Kind::Equation | Kind::Annotation
                )
            }) {
                return Err(invalid(
                    at,
                    "regimes declare alternative equations and numerical annotations over shared unknowns",
                ));
            }
            let previous = self.states[&instance].members.clone();
            for child in &children {
                let row = &self.p.declarations[child];
                if let Some(shared) = previous.get(&row.name)
                    && !(row.value.kind == Kind::Annotation
                        && self.p.declarations[shared].value.kind == Kind::Annotation)
                {
                    return Err(invalid(
                        *child,
                        "regime declaration shadows a shared implicit member",
                    ));
                }
                self.states
                    .get_mut(&instance)
                    .ok_or_else(|| invalid(instance, "regime owner absent"))?
                    .members
                    .insert(row.name.clone(), *child);
            }
            let eligible = regime
                .value
                .scope
                .as_ref()
                .and_then(|s| s.eligibility.as_deref())
                .unwrap_or("true");
            let eligibility = self.rewrite_predicate(
                instance,
                &dsl::parse_predicate(eligible).map_err(|e| invalid(at, e.to_string()))?,
                env,
                &[at],
            )?;
            let mut equations = Vec::new();
            let annotation_start = self.model.annotations.len();
            for child in children {
                let row = self.p.declarations[&child].clone();
                if let Some(e) = &row.value.equation {
                    for coordinates in self.coordinates(
                        child,
                        env,
                        e.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        let equation = dsl::parse_equation(&e.expression)
                            .map_err(|e| invalid(child, e.to_string()))?;
                        let equation = self.rewrite_equation(
                            instance,
                            &equation,
                            &coordinates_env(env, &coordinates),
                            &[child],
                        )?;
                        if !matches!(
                            equation.kind,
                            EquationKind::Relation {
                                sense: dsl::EquationSense::Eq,
                                ..
                            }
                        ) {
                            return Err(invalid(child, "regime residuals must be equalities"));
                        }
                        self.reserve(1)?;
                        equations.push(Row {
                            id: member_id(instance, child, &coordinates),
                            equation,
                            lineage: self.lineage(instance, &row, &[child]),
                        });
                    }
                } else {
                    self.annotation(instance, &row, env)?;
                }
            }
            let annotations = self.model.annotations.split_off(annotation_start);
            if annotations.iter().any(|a| {
                !matches!(
                    a.value,
                    crate::annotation::AnnotationValue::Start(_)
                        | crate::annotation::AnnotationValue::Bounds(..)
                        | crate::annotation::AnnotationValue::Nominal(_)
                        | crate::annotation::AnnotationValue::Scale(_)
                )
            }) {
                return Err(invalid(
                    at,
                    "regime annotations are starts, bounds, nominals and row scaling",
                ));
            }
            self.states
                .get_mut(&instance)
                .ok_or_else(|| invalid(instance, "regime owner absent"))?
                .members = previous;
            alternatives.push(Regime {
                id: member_id(instance, at, &[]),
                declaration: at,
                equations,
                annotations,
                eligibility,
            });
        }
        let unknowns = self
            .model
            .symbols
            .values()
            .filter(|s| {
                s.lineage.instance == instance && s.role == Kind::Variable && s.expression.is_none()
            })
            .count();
        if unknowns == 0 || alternatives.iter().any(|r| r.equations.len() != unknowns) {
            return Err(invalid(
                scope.declaration_id,
                "each regime must define the same nonempty square unknown system",
            ));
        }
        self.model.regimes.insert(
            instance,
            RegimeSelection {
                criterion,
                tolerance,
                quantity,
                alternatives,
            },
        );
        Ok(())
    }
}
