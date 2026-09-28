// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable formulation changes retaining original-space meaning and lineage.
use super::*;

/// One relaxed constraint with an independently observable original equation.
#[derive(Clone, Debug, PartialEq)]
pub struct Elastic {
    /// Original equation and physical provenance before slack insertion.
    pub original: Row,
    /// Nonnegative auxiliary variables, ordered positive then negative for equalities.
    pub slacks: Vec<SemanticId>,
    /// Dimensionless L1 penalty; normalization uses a positive authored physical nominal.
    pub penalty: Expr,
}
/// A physical homotopy parameter; progress belongs to the attempt, never this declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Continuation {
    /// Physical value at the start of the homotopy.
    pub start: Value,
    /// Physical value at the end of the homotopy.
    pub end: Value,
    /// Source of the continuation declaration.
    pub lineage: Lineage,
}
impl Continuation {
    /// Interpolate canonical physical values. Endpoints remain exact and failed steps do not mutate them.
    pub fn at(&self, fraction: f64) -> Result<Value> {
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(invalid(
                self.lineage.declaration,
                "continuation fraction outside [0,1]",
            ));
        }
        if fraction == 0.0 {
            return Ok(self.start.clone());
        }
        if fraction == 1.0 {
            return Ok(self.end.clone());
        }
        let Value::Number { quantity, .. } = self.start else {
            return Err(invalid(
                self.lineage.declaration,
                "physical continuation endpoint required",
            ));
        };
        let a = self.start.scalar(self.lineage.declaration)?;
        let b = self.end.scalar(self.lineage.declaration)?;
        let value = (1.0 - fraction) * a + fraction * b;
        if !value.is_finite() {
            return Err(invalid(
                self.lineage.declaration,
                "nonfinite continuation point",
            ));
        }
        Ok(Value::Number {
            quantity,
            bits: value.to_bits(),
        })
    }
}
impl Engine<'_, '_> {
    pub(super) fn transformation(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        if let Some(v) = &row.value.relaxation {
            for (target, ty, local) in
                self.annotation_targets(instance, &v.target, env, at, false)?
            {
                let nominal = self.eval(at, &local, &v.nominal, Some(&ty))?;
                if nominal.scalar(at)? <= 0.0 {
                    return Err(invalid(at, "elastic nominal must be positive"));
                }
                let lineage = self.lineage(instance, row, &[at]);
                if self
                    .relaxations
                    .insert(target, (ty, nominal, lineage))
                    .is_some()
                {
                    return Err(invalid(at, "competing relaxation policies"));
                }
            }
        }
        if let Some(v) = &row.value.continuation {
            for (target, ty, local) in
                self.annotation_targets(instance, &v.target, env, at, false)?
            {
                let start = self.eval(at, &local, &v.start, Some(&ty))?;
                let end = self.eval(at, &local, &v.end, Some(&ty))?;
                let lineage = self.lineage(instance, row, &[at]);
                if self
                    .model
                    .continuation
                    .insert(
                        target,
                        Continuation {
                            start,
                            end,
                            lineage,
                        },
                    )
                    .is_some()
                {
                    return Err(invalid(at, "competing continuation endpoints"));
                }
            }
        }
        Ok(())
    }
    pub(super) fn select_formulation(&mut self, policy: &Formulation) -> Result<()> {
        if policy
            .omitted
            .len()
            .checked_add(policy.elastic.len())
            .is_none_or(|n| n > self.limits.items)
        {
            return Err(ModelingError::Budget("formulation extent".into()));
        }
        for target in policy.omitted.iter().chain(policy.elastic.keys()) {
            self.checkpoint()?;
            let row = self
                .model
                .equations
                .iter()
                .find(|r| r.id == *target)
                .ok_or_else(|| invalid(*target, "formulation equation not selected"))?;
            let mut owner = Some(row.lineage.instance);
            while let Some(id) = owner {
                if self
                    .model
                    .implicit
                    .get(&id)
                    .is_some_and(Realization::is_nested)
                {
                    return Err(invalid(
                        *target,
                        "analysis overlay cannot change an inner solver residual",
                    ));
                }
                owner = self.model.instances.get(&id).and_then(|i| i.parent);
            }
            if policy.omitted.contains(target) && policy.elastic.contains_key(target) {
                return Err(invalid(*target, "cannot both omit and relax one equation"));
            }
            if self.relaxations.contains_key(target) {
                return Err(invalid(
                    *target,
                    "analysis overlay conflicts with an authored relaxation",
                ));
            }
        }
        self.model
            .equations
            .retain(|r| !policy.omitted.contains(&r.id));
        self.model
            .annotations
            .retain(|a| !policy.omitted.contains(&a.target));
        for instance in self.model.instances.values_mut() {
            instance.rows.retain(|r| !policy.omitted.contains(r));
        }
        for (target, nominal) in &policy.elastic {
            let Value::Number { quantity, bits } = nominal else {
                return Err(invalid(
                    *target,
                    "elastic nominal requires a physical scalar",
                ));
            };
            let value = f64::from_bits(*bits);
            if !value.is_finite() || value <= 0. {
                return Err(invalid(
                    *target,
                    "elastic nominal must be finite and positive",
                ));
            }
            let row = self
                .model
                .equations
                .iter()
                .find(|r| r.id == *target)
                .ok_or_else(|| invalid(*target, "elastic source absent"))?;
            self.relaxations.insert(
                *target,
                (
                    Type::Quantity(pse_quantity::scheme::Scheme::Concrete(*quantity)),
                    nominal.clone(),
                    row.lineage.clone(),
                ),
            );
        }
        Ok(())
    }
    pub(super) fn apply_relaxations(&mut self) -> Result<()> {
        let relaxations = std::mem::take(&mut self.relaxations);
        for (target, (ty, nominal, lineage)) in relaxations {
            self.checkpoint()?;
            let index = self
                .model
                .equations
                .iter()
                .position(|r| r.id == target)
                .ok_or_else(|| invalid(lineage.declaration, "relaxed equation not selected"))?;
            let original = self.model.equations[index].clone();
            let EquationKind::Relation { lhs, sense, rhs } = &original.equation.kind else {
                return Err(invalid(
                    target,
                    "elastic target must be a selected relation",
                ));
            };
            let mut residual = binary(BinaryOp::Sub, lhs.clone(), rhs.clone());
            let count = if *sense == EquationSense::Eq { 2 } else { 1 };
            self.reserve(count)?;
            let mut slacks = Vec::new();
            let mut penalty = None;
            let zero = self.typed_zero(&ty, lineage.declaration)?;
            let nominal_id = pse_ids::named_id(target, "elastic-nominal");
            let mut nominal_lineage = lineage.clone();
            nominal_lineage.path = format!("{}.elastic_nominal", original.lineage.path);
            self.reserve(1)?;
            self.model.symbols.insert(
                nominal_id,
                Symbol {
                    id: nominal_id,
                    ty: ty.clone(),
                    role: Kind::Parameter,
                    domain: Domain::Continuous,
                    expression: None,
                    initial: Some(nominal),
                    lineage: nominal_lineage,
                },
            );
            let denominator = symbol_expr(nominal_id);
            let Type::Quantity(s) = &ty else {
                return Err(invalid(target, "elastic physical residual required"));
            };
            let quantity = s
                .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
                .map_err(|e| invalid(target, e.to_string()))?;
            for ordinal in 0..count {
                let id = pse_ids::named_id(target, &format!("elastic-slack-{ordinal}"));
                let mut provenance = lineage.clone();
                provenance.path = format!("{}.slack_{ordinal}", original.lineage.path);
                self.model.symbols.insert(
                    id,
                    Symbol {
                        id,
                        ty: ty.clone(),
                        role: Kind::Variable,
                        domain: Domain::Continuous,
                        expression: None,
                        initial: Some(Value::Number {
                            bits: 0.0f64.to_bits(),
                            quantity,
                        }),
                        lineage: provenance.clone(),
                    },
                );
                self.model.annotations.push(crate::annotation::Annotation {
                    target: id,
                    value: crate::annotation::AnnotationValue::Start(zero.clone()),
                    lineage: provenance,
                });
                let slack = symbol_expr(id);
                let subtract =
                    (*sense == EquationSense::Le) || (*sense == EquationSense::Eq && ordinal == 0);
                residual = binary(
                    if subtract {
                        BinaryOp::Sub
                    } else {
                        BinaryOp::Add
                    },
                    residual,
                    slack.clone(),
                );
                let term = binary(BinaryOp::Div, slack, denominator.clone());
                penalty = Some(match penalty {
                    None => term,
                    Some(value) => binary(BinaryOp::Add, value, term),
                });
                slacks.push(id);
            }
            self.model.equations[index].equation = Equation {
                kind: EquationKind::Relation {
                    lhs: residual,
                    sense: *sense,
                    rhs: zero,
                },
                span: Span::default(),
            };
            self.model.elastic.insert(
                target,
                Elastic {
                    original,
                    slacks,
                    penalty: penalty.ok_or_else(|| invalid(target, "elastic penalty absent"))?,
                },
            );
        }
        Ok(())
    }
}
fn binary(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr {
        kind: ExprKind::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        span: Span::default(),
    }
}
