// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Constraint forms and disjunctions lowered by named transformations (ADR-0104, DP-08).
//!
//! Every lowering happens here, at preparation, never during a solve. The authored
//! revision is unchanged; the lowering record names the source, the realization and its
//! declared equivalence, and every derived row and variable keeps the authored lineage.
//! A linear path works over finite bounds on any mixed-integer backend; a native path
//! keeps the authored structure as [`NativeConstraint`] metadata that routing refuses on
//! a backend without the handler.
use super::*;
use crate::logic::Proposition;
use pse_model::{
    forms::{LogicOperand, NativeConstraint},
    generated::enums::{
        ModelingRealizationPolicy as Policy, ModelingStructuralRequirement, NativeConstraintForm,
    },
};

/// The default relative margin by which a derived big-M is widened outward.
pub const DEFAULT_BIG_M_MARGIN: f64 = 1e-6;

/// Declared equivalence of a lowering (ADR-0104 §3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Equivalence {
    /// The lowered set equals the authored one over the admitted case box.
    Exact,
    /// Exact only if the authored M is valid; that validity is the author's assertion.
    AuthoredBigM,
    /// Nonlinear hull rows use the ε-perspective, an O(ε) approximation (PS-06).
    Perspective {
        /// Declared epsilon.
        epsilon: f64,
    },
    /// Left to a native constraint handler; only a backend with it may run the case.
    Native,
    /// A complementarity smoothed to `f(first, second, width) == 0` by the authored
    /// smoothing function: an O(width) approximation, continued toward zero width (PS-06).
    Smoothed,
    /// A complementarity product row for the l1 exact-penalty route: exact at a
    /// nondegenerate solution with a sufficient penalty, a labelled least-infeasible point
    /// otherwise.
    ExactPenalty,
}
/// One named transformation applied at preparation.
#[derive(Clone, Debug, PartialEq)]
pub struct Lowering {
    /// Authored form or disjunction declaration.
    pub source: DeclarationId,
    /// Owning instance.
    pub instance: InstanceId,
    /// Declared (or default) realization.
    pub realization: Policy,
    /// Declared equivalence of the lowered set.
    pub equivalence: Equivalence,
    /// Derived rows, including conditional rows left to a native handler.
    pub rows: Vec<SemanticId>,
    /// Derived variables and parameters.
    pub variables: Vec<SemanticId>,
}
/// A parameter whose value the case box determines at preparation.
#[derive(Clone, Debug, PartialEq)]
pub struct Derived {
    /// Rule computing the value.
    pub rule: DerivedRule,
    /// Authored form the value serves; refusals name it.
    pub source: DeclarationId,
    /// Declared realization of that form.
    pub realization: Policy,
}
/// How a derived parameter is computed.
#[derive(Clone, Debug, PartialEq)]
pub enum DerivedRule {
    /// The finite case bound of a variable (its value, when the case fixes it).
    Bound {
        /// Bounded variable.
        variable: SemanticId,
        /// Upper rather than lower bound.
        upper: bool,
    },
    /// The FBBT supremum (or infimum) of an expression member over the case box, extended
    /// to include zero and widened outward by a relative margin (T15).
    Extremum {
        /// Expression member whose interval is taken.
        expression: SemanticId,
        /// Supremum rather than infimum.
        upper: bool,
        /// Relative outward widening.
        margin: f64,
    },
}

/// A declared realization with its parsed argument.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Realized {
    BigM(String),
    DerivedBigM(f64),
    Hull(Option<f64>),
    /// The authored smoothing function and width expression of a complementarity.
    Smooth {
        function: String,
        width: String,
    },
    Other(Policy),
}
impl Realized {
    fn policy(&self) -> Policy {
        match self {
            Self::BigM(_) => Policy::BigM,
            Self::DerivedBigM(_) => Policy::DerivedBigM,
            Self::Hull(_) => Policy::Hull,
            Self::Smooth { .. } => Policy::Smooth,
            Self::Other(policy) => *policy,
        }
    }
}
/// The authored form families and the realizations each admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Form {
    Indicator,
    Disjunction,
    OrderedSet,
    Cardinality,
    Piecewise,
    Logic,
    Complementarity,
}
impl Form {
    fn admits(self, policy: Policy) -> bool {
        match self {
            Self::Indicator => matches!(
                policy,
                Policy::BigM | Policy::DerivedBigM | Policy::Indicator
            ),
            Self::Disjunction => matches!(
                policy,
                Policy::BigM | Policy::DerivedBigM | Policy::Hull | Policy::Indicator
            ),
            Self::OrderedSet | Self::Cardinality | Self::Logic => {
                matches!(policy, Policy::Linear | Policy::Native)
            }
            Self::Piecewise => {
                matches!(policy, Policy::Sos2 | Policy::Incremental | Policy::Native)
            }
            Self::Complementarity => matches!(
                policy,
                Policy::Smooth | Policy::PenaltyL1 | Policy::Disjunctive
            ),
        }
    }
    fn default(self) -> Option<Realized> {
        match self {
            Self::Indicator => Some(Realized::DerivedBigM(DEFAULT_BIG_M_MARGIN)),
            // The tightness of a disjunction's reformulation is the author's decision.
            Self::Disjunction => None,
            Self::OrderedSet | Self::Cardinality | Self::Logic => {
                Some(Realized::Other(Policy::Linear))
            }
            Self::Piecewise => Some(Realized::Other(Policy::Sos2)),
            // How a complementarity is approximated or enforced is the author's decision.
            Self::Complementarity => None,
        }
    }
}
/// A disjunct row before realization.
struct DisjunctRow {
    id: SemanticId,
    lhs: Expr,
    sense: EquationSense,
    rhs: Expr,
    lineage: Lineage,
}
/// Dependence of an expression on a variable set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    Constant,
    Affine,
    Nonlinear,
}

impl Engine<'_, '_> {
    /// Register a realization declaration whose target is a constraint form or disjunction.
    pub(super) fn register_form_realization(
        &mut self,
        target: DeclarationId,
        declaration: DeclarationId,
    ) -> Result<()> {
        let v = self.p.declarations[&declaration]
            .value
            .realization
            .clone()
            .ok_or_else(|| invalid(declaration, "realization payload"))?;
        if v.accelerator.is_some() {
            return Err(invalid(
                declaration,
                "a form realization names no accelerator",
            ));
        }
        let number = |text: &str| -> Result<f64> {
            text.trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| invalid(declaration, "realization argument must be a finite number"))
        };
        if v.function.is_some() != (v.policy == Policy::Smooth) {
            return Err(invalid(
                declaration,
                "a smoothing function belongs to exactly a smooth realization",
            ));
        }
        let realized = match (v.policy, v.argument.as_deref()) {
            (Policy::Smooth, Some(width)) => Realized::Smooth {
                function: v.function.clone().unwrap_or_default(),
                width: width.into(),
            },
            (Policy::BigM, Some(m)) => Realized::BigM(m.into()),
            (Policy::DerivedBigM, margin) => {
                let margin = margin
                    .map(number)
                    .transpose()?
                    .unwrap_or(DEFAULT_BIG_M_MARGIN);
                if margin < 0.0 {
                    return Err(invalid(declaration, "a big-M margin is nonnegative"));
                }
                Realized::DerivedBigM(margin)
            }
            (Policy::Hull, epsilon) => {
                let epsilon = epsilon.map(number).transpose()?;
                if epsilon.is_some_and(|e| e <= 0.0) {
                    return Err(invalid(declaration, "a hull epsilon is positive"));
                }
                Realized::Hull(epsilon)
            }
            (
                policy @ (Policy::Indicator
                | Policy::Linear
                | Policy::Native
                | Policy::Sos2
                | Policy::Incremental
                | Policy::PenaltyL1
                | Policy::Disjunctive),
                None,
            ) => Realized::Other(policy),
            _ => {
                return Err(invalid(
                    declaration,
                    "realization and argument do not form a declared realization",
                ));
            }
        };
        if self
            .form_realizations
            .insert(target, (realized, declaration))
            .is_some()
        {
            return Err(invalid(declaration, "competing form realizations"));
        }
        Ok(())
    }
    fn realization(&self, form: Form, source: DeclarationId) -> Result<Realized> {
        let realized = match self.form_realizations.get(&source) {
            Some((realized, _)) => realized.clone(),
            None => form.default().ok_or_else(|| {
                invalid(
                    source,
                    if form == Form::Complementarity {
                        "a complementarity requires a declared realization"
                    } else {
                        "a disjunction requires a declared realization"
                    },
                )
            })?,
        };
        if !form.admits(realized.policy()) {
            return Err(invalid(
                source,
                format!(
                    "{} realization does not apply to this form",
                    realized.policy().as_str()
                ),
            ));
        }
        Ok(realized)
    }
    /// The unique registry quantity typing binary decisions and convex weights.
    fn indicator_type(&self, at: DeclarationId) -> Result<Type> {
        crate::indicator_type(self.c.quantities, at)
    }
    fn quantity_of(&self, ty: &Type, at: DeclarationId) -> Result<pse_quantity::QuantityTypeId> {
        let Type::Quantity(s) = ty else {
            return Err(invalid(at, "a constraint form requires physical members"));
        };
        s.resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
            .map_err(|e| invalid(at, e.to_string()))
    }
    fn derived_lineage(&self, base: &Lineage, suffix: &str) -> Lineage {
        let mut lineage = base.clone();
        lineage.path = format!("{}.{suffix}", base.path);
        lineage
    }
    /// A parameter equal to one of the indicator quantity, shared per instance.
    fn indicator_one(&mut self, instance: InstanceId, at: DeclarationId) -> Result<SemanticId> {
        let id = pse_ids::named_id(instance.as_id(), "indicator-one");
        if self.model.symbols.contains_key(&id) {
            return Ok(id);
        }
        let ty = self.indicator_type(at)?;
        let quantity = self.quantity_of(&ty, at)?;
        self.reserve(1)?;
        let state = &self.states[&instance];
        let lineage = Lineage {
            declaration: state.definition,
            instance,
            path: format!("{}.indicator_one", state.path),
            demand: vec![at],
            default_owner: None,
            is_override: false,
            presets: state.presets.clone(),
        };
        self.model.symbols.insert(
            id,
            Symbol {
                id,
                ty,
                role: Kind::Parameter,
                domain: Domain::Continuous,
                expression: None,
                initial: Some(Value::Number {
                    quantity,
                    bits: 1.0f64.to_bits(),
                }),
                lineage,
            },
        );
        Ok(id)
    }
    /// An indicator-typed constant; a count of indicators keeps the indicator contract.
    fn indicator_constant(
        &mut self,
        instance: InstanceId,
        value: f64,
        at: DeclarationId,
    ) -> Result<SemanticId> {
        if value == 1.0 {
            return self.indicator_one(instance, at);
        }
        let ty = self.indicator_type(at)?;
        let state = &self.states[&instance];
        let lineage = Lineage {
            declaration: state.definition,
            instance,
            path: format!("{}.indicator_{value}", state.path),
            demand: vec![at],
            default_owner: None,
            is_override: false,
            presets: state.presets.clone(),
        };
        self.constant_parameter(
            pse_ids::named_id(instance.as_id(), &format!("indicator-constant-{value}")),
            ty,
            value,
            lineage,
            at,
        )
    }
    fn constant_parameter(
        &mut self,
        id: SemanticId,
        ty: Type,
        value: f64,
        lineage: Lineage,
        at: DeclarationId,
    ) -> Result<SemanticId> {
        if self.model.symbols.contains_key(&id) {
            return Ok(id);
        }
        let quantity = self.quantity_of(&ty, at)?;
        self.reserve(1)?;
        self.model.symbols.insert(
            id,
            Symbol {
                id,
                ty,
                role: Kind::Parameter,
                domain: Domain::Continuous,
                expression: None,
                initial: Some(Value::Number {
                    quantity,
                    bits: value.to_bits(),
                }),
                lineage,
            },
        );
        Ok(id)
    }
    fn derived_variable(
        &mut self,
        id: SemanticId,
        ty: Type,
        domain: Domain,
        lineage: Lineage,
        at: DeclarationId,
    ) -> Result<SemanticId> {
        let zero = self.typed_zero(&ty, at)?;
        self.reserve(1)?;
        self.model.symbols.insert(
            id,
            Symbol {
                id,
                ty,
                role: Kind::Variable,
                domain,
                expression: None,
                initial: None,
                lineage: lineage.clone(),
            },
        );
        self.model.annotations.push(crate::annotation::Annotation {
            target: id,
            value: crate::annotation::AnnotationValue::Start(zero),
            lineage,
        });
        Ok(id)
    }
    fn derived_parameter(
        &mut self,
        id: SemanticId,
        ty: Type,
        rule: DerivedRule,
        source: DeclarationId,
        realization: Policy,
        lineage: Lineage,
    ) -> Result<SemanticId> {
        if self.model.symbols.contains_key(&id) {
            return Ok(id);
        }
        self.reserve(1)?;
        self.model.symbols.insert(
            id,
            Symbol {
                id,
                ty,
                role: Kind::Parameter,
                domain: Domain::Continuous,
                expression: None,
                initial: None,
                lineage,
            },
        );
        self.model.derived.insert(
            id,
            Derived {
                rule,
                source,
                realization,
            },
        );
        Ok(id)
    }
    fn push_row(
        &mut self,
        id: SemanticId,
        lhs: Expr,
        sense: EquationSense,
        rhs: Expr,
        lineage: Lineage,
    ) -> Result<SemanticId> {
        self.reserve(1)?;
        self.model.equations.push(Row {
            id,
            equation: Equation {
                kind: EquationKind::Relation { lhs, sense, rhs },
                span: Span::default(),
            },
            lineage,
        });
        Ok(id)
    }
    fn type_of(
        &self,
        expression: &Expr,
        contracts: &CheckedPackage,
        at: DeclarationId,
    ) -> Result<Type> {
        let types = self
            .model
            .symbols
            .iter()
            .map(|(id, s)| (symbol_name(*id), s.ty.clone()))
            .collect::<BTreeMap<_, _>>();
        crate::expression::infer(expression, &types, contracts, self.c, at, None)
    }
    /// Resolve an authored member path to an existing binary variable.
    fn binary_operand(
        &mut self,
        instance: InstanceId,
        text: &str,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<SemanticId> {
        let expression = dsl::parse_expr(text).map_err(|e| invalid(at, e.to_string()))?;
        let expression = self.rewrite(instance, &expression, env, &[at])?;
        symbol_reference(&expression)
            .filter(|id| {
                self.model
                    .symbols
                    .get(id)
                    .is_some_and(|s| s.role == Kind::Variable && s.domain == Domain::Binary)
            })
            .ok_or_else(|| invalid(at, format!("{text} must name a binary variable")))
    }
    fn record(
        &mut self,
        source: DeclarationId,
        instance: InstanceId,
        realization: Policy,
        equivalence: Equivalence,
        rows: Vec<SemanticId>,
        variables: Vec<SemanticId>,
    ) {
        self.model.lowerings.push(Lowering {
            source,
            instance,
            realization,
            equivalence,
            rows,
            variables,
        });
    }

    /// `eq name when y: …` — an indicator constraint (ADR-0104 §1).
    pub(super) fn indicator_equation(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        coordinates: &[(String, Value)],
        equation: Equation,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let condition = row
            .value
            .equation
            .as_ref()
            .and_then(|e| e.condition.clone())
            .ok_or_else(|| invalid(at, "indicator condition absent"))?;
        let local = coordinates_env(env, coordinates);
        let variable = self.binary_operand(instance, &condition.variable, &local, at)?;
        let EquationKind::Relation { lhs, sense, rhs } = equation.kind else {
            return Err(invalid(at, "an indicator constraint is a relation"));
        };
        let id = member_id(instance, at, coordinates);
        let lineage = self.lineage(instance, row, &[at]);
        let realized = self.realization(Form::Indicator, at)?;
        if realized == Realized::Other(Policy::Indicator) {
            let row = self.push_row(id, lhs, sense, rhs, lineage)?;
            self.model.native.push(NativeConstraint::Indicator {
                row,
                variable,
                active: condition.active,
            });
            self.record(
                at,
                instance,
                Policy::Indicator,
                Equivalence::Native,
                vec![row],
                vec![],
            );
            return Ok(());
        }
        let one = self.indicator_one(instance, at)?;
        // The relaxation factor is zero exactly when the row must hold.
        let relax = if condition.active {
            bin(BinaryOp::Sub, sym(one), sym(variable))
        } else {
            sym(variable)
        };
        let (rows, variables) = self.relaxed(
            instance, at, id, lhs, sense, rhs, relax, &realized, lineage, env,
        )?;
        let equivalence = match realized {
            Realized::BigM(_) => Equivalence::AuthoredBigM,
            _ => Equivalence::Exact,
        };
        self.record(
            at,
            instance,
            realized.policy(),
            equivalence,
            rows,
            variables,
        );
        Ok(())
    }
    /// Big-M rows `r <= M·f` and `r >= m·f` whose factor `f` vanishes when the row holds.
    #[expect(
        clippy::too_many_arguments,
        reason = "one relaxed relation keeps its source, identity and factor explicit"
    )]
    fn relaxed(
        &mut self,
        instance: InstanceId,
        source: DeclarationId,
        id: SemanticId,
        lhs: Expr,
        sense: EquationSense,
        rhs: Expr,
        relax: Expr,
        realized: &Realized,
        lineage: Lineage,
        env: &Environment,
    ) -> Result<(Vec<SemanticId>, Vec<SemanticId>)> {
        let contracts = self.model.function_contracts(self.p);
        let residual = bin(BinaryOp::Sub, lhs, rhs);
        let ty = self.type_of(&residual, &contracts, source)?;
        let mut variables = Vec::new();
        let (residual, upper, lower) = match realized {
            Realized::BigM(text) => {
                let m = dsl::parse_expr(text).map_err(|e| invalid(source, e.to_string()))?;
                let m = self.rewrite(instance, &m, env, &[source])?;
                let types = self
                    .model
                    .symbols
                    .iter()
                    .map(|(id, s)| (symbol_name(*id), s.ty.clone()))
                    .collect::<BTreeMap<_, _>>();
                if crate::expression::infer(&m, &types, &contracts, self.c, source, Some(&ty))?
                    != ty
                {
                    return Err(invalid(
                        source,
                        "an authored big-M has the row's physical type",
                    ));
                }
                // A typed member gives the authored value its row's physical context.
                let m_id = pse_ids::named_id(id, "authored-big-m");
                if !self.model.symbols.contains_key(&m_id) {
                    self.reserve(1)?;
                    self.model.symbols.insert(
                        m_id,
                        Symbol {
                            id: m_id,
                            ty: ty.clone(),
                            role: Kind::Let,
                            domain: Domain::Continuous,
                            expression: Some(m),
                            initial: None,
                            lineage: self.derived_lineage(&lineage, "big_m"),
                        },
                    );
                }
                variables.push(m_id);
                (residual, sym(m_id), neg(sym(m_id)))
            }
            Realized::DerivedBigM(margin) => {
                let residual_id = pse_ids::named_id(id, "disjunct-residual");
                self.reserve(1)?;
                self.model.symbols.insert(
                    residual_id,
                    Symbol {
                        id: residual_id,
                        ty: ty.clone(),
                        role: Kind::Let,
                        domain: Domain::Continuous,
                        expression: Some(residual),
                        initial: None,
                        lineage: self.derived_lineage(&lineage, "residual"),
                    },
                );
                let bound = |upper: bool, engine: &mut Self| {
                    engine.derived_parameter(
                        pse_ids::named_id(id, if upper { "big-m-upper" } else { "big-m-lower" }),
                        ty.clone(),
                        DerivedRule::Extremum {
                            expression: residual_id,
                            upper,
                            margin: *margin,
                        },
                        source,
                        Policy::DerivedBigM,
                        engine.derived_lineage(
                            &lineage,
                            if upper { "big_m_upper" } else { "big_m_lower" },
                        ),
                    )
                };
                // Only the sides the row's sense needs are derived.
                variables.push(residual_id);
                let mut side = |upper: bool, engine: &mut Self| -> Result<Expr> {
                    let needed = if upper {
                        sense != EquationSense::Ge
                    } else {
                        sense != EquationSense::Le
                    };
                    if !needed {
                        return Ok(sym(residual_id));
                    }
                    let id = bound(upper, engine)?;
                    variables.push(id);
                    Ok(sym(id))
                };
                let up = side(true, self)?;
                let down = side(false, self)?;
                (sym(residual_id), up, down)
            }
            _ => return Err(invalid(source, "big-M realization expected")),
        };
        let mut rows = Vec::new();
        if matches!(sense, EquationSense::Le | EquationSense::Eq) {
            rows.push(self.push_row(
                pse_ids::named_id(id, "big-m-upper-row"),
                residual.clone(),
                EquationSense::Le,
                bin(BinaryOp::Mul, upper, relax.clone()),
                lineage.clone(),
            )?);
        }
        if matches!(sense, EquationSense::Ge | EquationSense::Eq) {
            rows.push(self.push_row(
                pse_ids::named_id(id, "big-m-lower-row"),
                residual,
                EquationSense::Ge,
                bin(BinaryOp::Mul, lower, relax),
                lineage,
            )?);
        }
        Ok((rows, variables))
    }

    /// `disjunction name { alternative … }`, lowered inner-first (ADR-0104 §1).
    pub(super) fn disjunction(
        &mut self,
        instance: InstanceId,
        declaration: DeclarationId,
        env: &Environment,
        parent: Option<SemanticId>,
    ) -> Result<()> {
        let row = self.p.declarations[&declaration].clone();
        let alternatives = self
            .p
            .children
            .get(&declaration)
            .into_iter()
            .flatten()
            .copied()
            .filter(|id| self.p.declarations[id].value.kind == Kind::Alternative)
            .collect::<Vec<_>>();
        if alternatives.len() < 2 {
            return Err(invalid(
                declaration,
                "a disjunction has at least two alternatives",
            ));
        }
        for alternative in &alternatives {
            for child in self
                .p
                .children
                .get(alternative)
                .into_iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
            {
                if let Some(policy) = &self.p.declarations[&child].value.realization {
                    let target = self
                        .p
                        .resolve(child, &policy.target)
                        .ok_or_else(|| invalid(child, "realization target absent"))?;
                    self.register_form_realization(target, child)?;
                }
            }
        }
        let realized = self.realization(Form::Disjunction, declaration)?;
        let lineage = self.lineage(instance, &row, &[declaration]);
        let base = member_id(instance, declaration, &[]);
        let mut indicators = Vec::new();
        for alternative in &alternatives {
            indicators.push(self.symbol(instance, *alternative, &[], &[declaration])?);
        }
        // Inner disjunctions first: each selects exactly one alternative while its owner is selected.
        for (alternative, indicator) in alternatives.iter().zip(&indicators) {
            for child in self
                .p
                .children
                .get(alternative)
                .into_iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
            {
                if self.p.declarations[&child].value.kind == Kind::Disjunction {
                    self.disjunction(instance, child, env, Some(*indicator))?;
                }
            }
        }
        let one = self.indicator_one(instance, declaration)?;
        let selected = sum(indicators.iter().map(|id| sym(*id)).collect())
            .ok_or_else(|| invalid(declaration, "disjunction without alternatives"))?;
        let mut rows = vec![self.push_row(
            pse_ids::named_id(base, "select-one"),
            selected,
            EquationSense::Eq,
            sym(parent.unwrap_or(one)),
            lineage.clone(),
        )?];
        let mut variables = indicators.clone();
        let mut disjuncts = Vec::new();
        for alternative in &alternatives {
            disjuncts.push(self.disjunct_rows(instance, *alternative, env)?);
        }
        let equivalence = match &realized {
            Realized::Other(Policy::Indicator) => {
                for (rows_k, indicator) in disjuncts.into_iter().zip(&indicators) {
                    for r in rows_k {
                        let row = self.push_row(r.id, r.lhs, r.sense, r.rhs, r.lineage)?;
                        self.model.native.push(NativeConstraint::Indicator {
                            row,
                            variable: *indicator,
                            active: true,
                        });
                        rows.push(row);
                    }
                }
                Equivalence::Native
            }
            Realized::BigM(_) | Realized::DerivedBigM(_) => {
                for (rows_k, indicator) in disjuncts.into_iter().zip(&indicators) {
                    for r in rows_k {
                        let relax = bin(BinaryOp::Sub, sym(one), sym(*indicator));
                        let (derived_rows, derived) = self.relaxed(
                            instance,
                            declaration,
                            r.id,
                            r.lhs,
                            r.sense,
                            r.rhs,
                            relax,
                            &realized,
                            r.lineage,
                            env,
                        )?;
                        rows.extend(derived_rows);
                        variables.extend(derived);
                    }
                }
                if matches!(realized, Realized::BigM(_)) {
                    Equivalence::AuthoredBigM
                } else {
                    Equivalence::Exact
                }
            }
            Realized::Hull(epsilon) => {
                let (hull_rows, hull_variables, perspective) = self.hull(
                    declaration,
                    base,
                    parent,
                    &indicators,
                    disjuncts,
                    *epsilon,
                    &lineage,
                )?;
                rows.extend(hull_rows);
                variables.extend(hull_variables);
                match (perspective, epsilon) {
                    (true, Some(epsilon)) => Equivalence::Perspective { epsilon: *epsilon },
                    _ => Equivalence::Exact,
                }
            }
            Realized::Smooth { .. } | Realized::Other(_) => {
                return Err(invalid(declaration, "unsupported disjunction realization"));
            }
        };
        self.record(
            declaration,
            instance,
            realized.policy(),
            equivalence,
            rows,
            variables,
        );
        Ok(())
    }
    /// An alternative's rows: equations and conditional bounds. Starts and nominals remain
    /// numerical hints; nested disjunctions and realizations are handled by the owner.
    fn disjunct_rows(
        &mut self,
        instance: InstanceId,
        alternative: DeclarationId,
        env: &Environment,
    ) -> Result<Vec<DisjunctRow>> {
        let mut rows = Vec::new();
        for child in self
            .p
            .children
            .get(&alternative)
            .into_iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>()
        {
            self.checkpoint()?;
            let declaration = self.p.declarations[&child].clone();
            match declaration.value.kind {
                Kind::Equation => {
                    let e = declaration
                        .value
                        .equation
                        .clone()
                        .ok_or_else(|| invalid(child, "equation payload"))?;
                    if e.condition.is_some() {
                        return Err(invalid(
                            child,
                            "a disjunct row is not an indicator constraint",
                        ));
                    }
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
                        let EquationKind::Relation { lhs, sense, rhs } = equation.kind else {
                            return Err(invalid(child, "a disjunct row is a relation"));
                        };
                        rows.push(DisjunctRow {
                            id: member_id(instance, child, &coordinates),
                            lhs,
                            sense,
                            rhs,
                            lineage: self.lineage(instance, &declaration, &[child]),
                        });
                    }
                }
                Kind::Annotation => {
                    let start = self.model.annotations.len();
                    self.annotation(instance, &declaration, env)?;
                    let added = self.model.annotations.split_off(start);
                    for (ordinal, annotation) in added.into_iter().enumerate() {
                        match annotation.value {
                            crate::annotation::AnnotationValue::Bounds(lower, upper) => {
                                let id = pse_ids::named_id(
                                    member_id(instance, child, &[]),
                                    &format!("bound-{ordinal}-{}", annotation.target.to_hex()),
                                );
                                for (suffix, sense, bound) in [
                                    ("lower", EquationSense::Ge, lower),
                                    ("upper", EquationSense::Le, upper),
                                ] {
                                    rows.push(DisjunctRow {
                                        id: pse_ids::named_id(id, suffix),
                                        lhs: sym(annotation.target),
                                        sense,
                                        rhs: bound,
                                        lineage: annotation.lineage.clone(),
                                    });
                                }
                            }
                            crate::annotation::AnnotationValue::Start(_)
                            | crate::annotation::AnnotationValue::Nominal(_) => {
                                self.model.annotations.push(annotation);
                            }
                            _ => {
                                return Err(invalid(
                                    child,
                                    "alternative annotations are bounds, starts and nominals",
                                ));
                            }
                        }
                    }
                }
                Kind::Disjunction | Kind::Realization => {}
                _ => {
                    return Err(invalid(
                        child,
                        "alternatives declare equations, bounds, nested disjunctions and realizations",
                    ));
                }
            }
        }
        Ok(rows)
    }
    /// Convex-hull reformulation with disaggregated variables. Affine rows are exact; other
    /// rows use the ε-perspective `(y+ε)·g(v/(y+ε))` when an epsilon is declared.
    #[expect(
        clippy::too_many_arguments,
        reason = "the hull consumes the complete disjunction and its owner in one transformation"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "disaggregation, linking, bounds and row reformulation form one transformation"
    )]
    fn hull(
        &mut self,
        source: DeclarationId,
        base: SemanticId,
        parent: Option<SemanticId>,
        indicators: &[SemanticId],
        disjuncts: Vec<Vec<DisjunctRow>>,
        epsilon: Option<f64>,
        lineage: &Lineage,
    ) -> Result<(Vec<SemanticId>, Vec<SemanticId>, bool)> {
        let contracts = self.model.function_contracts(self.p);
        let mut variables = BTreeSet::new();
        for row in disjuncts.iter().flatten() {
            self.collect_variables(&row.lhs, &mut variables);
            self.collect_variables(&row.rhs, &mut variables);
        }
        let mut rows = Vec::new();
        let mut derived = Vec::new();
        let mut copies = vec![BTreeMap::<SemanticId, Expr>::new(); indicators.len()];
        let mut zeros = BTreeMap::new();
        for x in &variables {
            let symbol = self.model.symbols[x].clone();
            let bound = |upper: bool, engine: &mut Self| {
                engine.derived_parameter(
                    pse_ids::named_id(
                        base,
                        &format!(
                            "hull-{}-{}",
                            if upper { "upper" } else { "lower" },
                            x.to_hex()
                        ),
                    ),
                    symbol.ty.clone(),
                    DerivedRule::Bound {
                        variable: *x,
                        upper,
                    },
                    source,
                    Policy::Hull,
                    engine.derived_lineage(
                        &symbol.lineage,
                        if upper { "hull_upper" } else { "hull_lower" },
                    ),
                )
            };
            let upper = bound(true, self)?;
            let lower = bound(false, self)?;
            let zero = self.constant_parameter(
                pse_ids::named_id(base, &format!("hull-zero-{}", x.to_hex())),
                symbol.ty.clone(),
                0.0,
                self.derived_lineage(&symbol.lineage, "hull_zero"),
                source,
            )?;
            zeros.insert(*x, sym(zero));
            derived.extend([upper, lower, zero]);
            let mut parts = Vec::new();
            for (k, indicator) in indicators.iter().enumerate() {
                let copy = self.derived_variable(
                    pse_ids::named_id(*indicator, &format!("hull-{}", x.to_hex())),
                    symbol.ty.clone(),
                    Domain::Continuous,
                    self.derived_lineage(&symbol.lineage, &format!("hull_{k}")),
                    source,
                )?;
                derived.push(copy);
                copies[k].insert(*x, sym(copy));
                parts.push(sym(copy));
                rows.push(self.push_row(
                    pse_ids::named_id(copy, "hull-upper"),
                    sym(copy),
                    EquationSense::Le,
                    bin(BinaryOp::Mul, sym(upper), sym(*indicator)),
                    lineage.clone(),
                )?);
                rows.push(self.push_row(
                    pse_ids::named_id(copy, "hull-lower"),
                    sym(copy),
                    EquationSense::Ge,
                    bin(BinaryOp::Mul, sym(lower), sym(*indicator)),
                    lineage.clone(),
                )?);
            }
            let parts = sum(parts).ok_or_else(|| invalid(source, "hull without alternatives"))?;
            let link = pse_ids::named_id(base, &format!("hull-link-{}", x.to_hex()));
            match parent {
                None => rows.push(self.push_row(
                    link,
                    sym(*x),
                    EquationSense::Eq,
                    parts,
                    lineage.clone(),
                )?),
                Some(owner) => {
                    // Inside an unselected owner the copies vanish and x keeps its own box.
                    let gap = bin(BinaryOp::Sub, sym(*x), parts);
                    for (suffix, sense, bound) in [
                        ("upper", EquationSense::Le, upper),
                        ("lower", EquationSense::Ge, lower),
                    ] {
                        rows.push(self.push_row(
                            pse_ids::named_id(link, suffix),
                            gap.clone(),
                            sense,
                            bin(
                                BinaryOp::Sub,
                                sym(bound),
                                bin(BinaryOp::Mul, sym(bound), sym(owner)),
                            ),
                            lineage.clone(),
                        )?);
                    }
                }
            }
        }
        let mut perspective = false;
        for ((rows_k, indicator), map) in disjuncts.into_iter().zip(indicators).zip(&copies) {
            for r in rows_k {
                let g = bin(BinaryOp::Sub, r.lhs, r.rhs);
                let ty = self.type_of(&g, &contracts, source)?;
                let zero = self.typed_zero(&ty, source)?;
                let mut memo = BTreeMap::new();
                let lhs = match self.classify(&g, &variables, &mut memo) {
                    Class::Constant | Class::Affine => {
                        // Homogenize the affine row: g(v) − g(0) + g(0)·y.
                        let at_copy = self.substitute(&g, map, &variables, source)?;
                        let at_zero = self.substitute(&g, &zeros, &variables, source)?;
                        bin(
                            BinaryOp::Add,
                            bin(BinaryOp::Sub, at_copy, at_zero.clone()),
                            bin(BinaryOp::Mul, at_zero, sym(*indicator)),
                        )
                    }
                    Class::Nonlinear => {
                        let Some(epsilon) = epsilon else {
                            return Err(ModelingError::Realization {
                                declaration: source.into(),
                                form: lineage.path.clone(),
                                subject: r.lineage.path.clone(),
                                realization: Policy::Hull,
                                reason: crate::RealizationRefusal::Nonlinear,
                            });
                        };
                        perspective = true;
                        let indicator_ty = self.indicator_type(source)?;
                        let eps = self.constant_parameter(
                            pse_ids::named_id(base, "hull-epsilon"),
                            indicator_ty,
                            epsilon,
                            self.derived_lineage(lineage, "hull_epsilon"),
                            source,
                        )?;
                        derived.push(eps);
                        let scale = bin(BinaryOp::Add, sym(*indicator), sym(eps));
                        let scaled = map
                            .iter()
                            .map(|(x, copy)| (*x, bin(BinaryOp::Div, copy.clone(), scale.clone())))
                            .collect();
                        bin(
                            BinaryOp::Mul,
                            scale,
                            self.substitute(&g, &scaled, &variables, source)?,
                        )
                    }
                };
                rows.push(self.push_row(r.id, lhs, r.sense, zero, r.lineage)?);
            }
        }
        Ok((rows, derived, perspective))
    }
    /// Independent variables an expression depends on, through expression members.
    fn collect_variables(&self, expression: &Expr, out: &mut BTreeSet<SemanticId>) {
        let mut pending = vec![expression.clone()];
        let mut seen = BTreeSet::new();
        while let Some(e) = pending.pop() {
            for path in e.paths() {
                let Some(id) = path
                    .segments
                    .first()
                    .filter(|_| path.segments.len() == 1)
                    .and_then(|s| s.name.strip_prefix("s_"))
                    .and_then(|hex| SemanticId::parse_hex(hex).ok())
                else {
                    continue;
                };
                let Some(symbol) = self.model.symbols.get(&id) else {
                    continue;
                };
                match &symbol.expression {
                    Some(body) if seen.insert(id) => pending.push(body.clone()),
                    None if symbol.role == Kind::Variable => {
                        out.insert(id);
                    }
                    _ => {}
                }
            }
        }
    }
    fn depends(&self, expression: &Expr, variables: &BTreeSet<SemanticId>) -> bool {
        let mut found = BTreeSet::new();
        self.collect_variables(expression, &mut found);
        !found.is_disjoint(variables)
    }
    /// Conservative dependence class of an expression on `variables`.
    fn classify(
        &self,
        expression: &Expr,
        variables: &BTreeSet<SemanticId>,
        memo: &mut BTreeMap<String, Class>,
    ) -> Class {
        match &expression.kind {
            ExprKind::Number(_) => Class::Constant,
            ExprKind::Path(path) => {
                let name = dsl::render_path(path);
                if let Some(class) = memo.get(&name) {
                    return *class;
                }
                let class = match symbol_reference(expression)
                    .and_then(|id| self.model.symbols.get(&id).map(|s| (id, s)))
                {
                    Some((id, _)) if variables.contains(&id) => Class::Affine,
                    Some((_, symbol)) => match &symbol.expression {
                        Some(body) => self.classify(body, variables, memo),
                        None => Class::Constant,
                    },
                    None if self.depends(expression, variables) => Class::Nonlinear,
                    None => Class::Constant,
                };
                memo.insert(name, class);
                class
            }
            ExprKind::Neg(e) => self.classify(e, variables, memo),
            ExprKind::Binary { op, lhs, rhs } => {
                let a = self.classify(lhs, variables, memo);
                let b = self.classify(rhs, variables, memo);
                match op {
                    BinaryOp::Add | BinaryOp::Sub => a.max(b),
                    BinaryOp::Mul if a == Class::Constant => b,
                    BinaryOp::Mul if b == Class::Constant => a,
                    BinaryOp::Div if b == Class::Constant => a,
                    _ if a == Class::Constant && b == Class::Constant => Class::Constant,
                    _ => Class::Nonlinear,
                }
            }
            ExprKind::NamedCall { name, args }
                if self.model.functions.get(name).is_some_and(|f| {
                    f.reduction
                        .as_ref()
                        .is_some_and(|r| r.kind == pse_quantity::ReductionKind::Sum)
                }) =>
            {
                args.iter()
                    .map(|a| self.classify(a, variables, memo))
                    .max()
                    .unwrap_or(Class::Constant)
            }
            ExprKind::Let { bindings, body } => {
                for (name, value) in bindings {
                    let class = self.classify(value, variables, memo);
                    memo.insert(name.clone(), class);
                }
                self.classify(body, variables, memo)
            }
            _ if self.depends(expression, variables) => Class::Nonlinear,
            _ => Class::Constant,
        }
    }
    /// Replace `variables` by `map`, inlining expression members that depend on them.
    fn substitute(
        &self,
        expression: &Expr,
        map: &BTreeMap<SemanticId, Expr>,
        variables: &BTreeSet<SemanticId>,
        at: DeclarationId,
    ) -> Result<Expr> {
        let kind = match &expression.kind {
            ExprKind::Number(_) => return Ok(expression.clone()),
            ExprKind::Path(_) => {
                return match symbol_reference(expression) {
                    Some(id) if map.contains_key(&id) => Ok(map[&id].clone()),
                    Some(id) => match self
                        .model
                        .symbols
                        .get(&id)
                        .and_then(|s| s.expression.as_ref())
                    {
                        Some(body) if self.depends(body, variables) => {
                            self.substitute(body, map, variables, at)
                        }
                        _ => Ok(expression.clone()),
                    },
                    None => Ok(expression.clone()),
                };
            }
            ExprKind::Neg(e) => ExprKind::Neg(Box::new(self.substitute(e, map, variables, at)?)),
            ExprKind::Binary { op, lhs, rhs } => ExprKind::Binary {
                op: *op,
                lhs: Box::new(self.substitute(lhs, map, variables, at)?),
                rhs: Box::new(self.substitute(rhs, map, variables, at)?),
            },
            ExprKind::Call { function, args } => ExprKind::Call {
                function: *function,
                args: args
                    .iter()
                    .map(|a| self.substitute(a, map, variables, at))
                    .collect::<Result<_>>()?,
            },
            ExprKind::NamedCall { name, args } => ExprKind::NamedCall {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| self.substitute(a, map, variables, at))
                    .collect::<Result<_>>()?,
            },
            ExprKind::Let { bindings, body } => ExprKind::Let {
                bindings: bindings
                    .iter()
                    .map(|(n, v)| Ok((n.clone(), self.substitute(v, map, variables, at)?)))
                    .collect::<Result<_>>()?,
                body: Box::new(self.substitute(body, map, variables, at)?),
            },
            _ if self.depends(expression, variables) => {
                return Err(invalid(
                    at,
                    "a hull disjunct row uses an expression form the hull cannot disaggregate",
                ));
            }
            _ => return Ok(expression.clone()),
        };
        Ok(Expr {
            kind,
            span: Span::default(),
        })
    }

    /// `sos1`/`sos2 name[i in s]: x[i] weight w[i];`
    pub(super) fn ordered_set(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let v = row
            .value
            .ordered_set
            .clone()
            .ok_or_else(|| invalid(at, "ordered set payload"))?;
        let form = if row.value.kind == Kind::Sos1 {
            NativeConstraintForm::Sos1
        } else {
            NativeConstraintForm::Sos2
        };
        let mut members = Vec::new();
        for coordinates in self.coordinates(
            at,
            env,
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str())),
        )? {
            let local = coordinates_env(env, &coordinates);
            let member = self.variable_operand(instance, &v.member, &local, at)?;
            let weight = self.eval(at, &local, &v.weight, None)?.scalar(at)?;
            members.push((member, weight));
        }
        members.sort_by(|a, b| a.1.total_cmp(&b.1));
        if members.windows(2).any(|w| w[0].1 >= w[1].1) || members.iter().any(|m| !m.1.is_finite())
        {
            return Err(invalid(at, "ordered set weights are finite and distinct"));
        }
        let realized = self.realization(Form::OrderedSet, at)?;
        let lineage = self.lineage(instance, row, &[at]);
        let base = member_id(instance, at, &[]);
        if realized == Realized::Other(Policy::Native) {
            self.model.native.push(NativeConstraint::Sos {
                form,
                members: members.clone(),
            });
            self.record(
                at,
                instance,
                Policy::Native,
                Equivalence::Native,
                vec![],
                vec![],
            );
            return Ok(());
        }
        let ids = members.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let (rows, variables) =
            self.linear_ordered_set(instance, at, base, &ids, form, &lineage)?;
        self.record(
            at,
            instance,
            Policy::Linear,
            Equivalence::Exact,
            rows,
            variables,
        );
        Ok(())
    }
    /// Resolve a member path to an existing independent variable.
    fn variable_operand(
        &mut self,
        instance: InstanceId,
        text: &str,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<SemanticId> {
        let expression = dsl::parse_expr(text).map_err(|e| invalid(at, e.to_string()))?;
        let expression = self.rewrite(instance, &expression, env, &[at])?;
        symbol_reference(&expression)
            .filter(|id| {
                self.model
                    .symbols
                    .get(id)
                    .is_some_and(|s| s.role == Kind::Variable && s.expression.is_none())
            })
            .ok_or_else(|| invalid(at, format!("{text} must name a variable")))
    }
    /// Member `i` may be nonzero only while its binary `z` allows: `L·z ≤ x ≤ U·z` over
    /// finite case bounds. SOS1 selects one member; SOS2 one segment of two neighbours.
    fn linear_ordered_set(
        &mut self,
        instance: InstanceId,
        source: DeclarationId,
        base: SemanticId,
        members: &[SemanticId],
        form: NativeConstraintForm,
        lineage: &Lineage,
    ) -> Result<(Vec<SemanticId>, Vec<SemanticId>)> {
        let indicator = self.indicator_type(source)?;
        let one = self.indicator_one(instance, source)?;
        let selectors = match form {
            NativeConstraintForm::Sos2 => members.len().saturating_sub(1),
            _ => members.len(),
        };
        let mut variables = Vec::new();
        let mut z = Vec::new();
        for k in 0..selectors {
            let id = self.derived_variable(
                pse_ids::named_id(base, &format!("selector-{k}")),
                indicator.clone(),
                Domain::Binary,
                self.derived_lineage(lineage, &format!("selector_{k}")),
                source,
            )?;
            z.push(id);
            variables.push(id);
        }
        let mut rows = Vec::new();
        if let Some(total) = sum(z.iter().map(|id| sym(*id)).collect()) {
            rows.push(self.push_row(
                pse_ids::named_id(base, "select-at-most-one"),
                total,
                EquationSense::Le,
                sym(one),
                lineage.clone(),
            )?);
        }
        for (i, member) in members.iter().enumerate() {
            // The selectors that allow member i to be nonzero.
            let allowed = match form {
                NativeConstraintForm::Sos2 => [i.checked_sub(1), (i < selectors).then_some(i)]
                    .into_iter()
                    .flatten()
                    .map(|k| sym(z[k]))
                    .collect::<Vec<_>>(),
                _ => vec![sym(z[i])],
            };
            let Some(allowed) = sum(allowed) else {
                continue;
            };
            let (rows_i, bounds) =
                self.switched_bounds(source, base, *member, allowed, Policy::Linear, lineage)?;
            rows.extend(rows_i);
            variables.extend(bounds);
        }
        Ok((rows, variables))
    }
    /// `L·s ≤ x ≤ U·s` with derived finite case bounds of `x`.
    fn switched_bounds(
        &mut self,
        source: DeclarationId,
        base: SemanticId,
        member: SemanticId,
        switch: Expr,
        realization: Policy,
        lineage: &Lineage,
    ) -> Result<(Vec<SemanticId>, Vec<SemanticId>)> {
        let symbol = self.model.symbols[&member].clone();
        let mut rows = Vec::new();
        let mut variables = Vec::new();
        for (upper, sense) in [(true, EquationSense::Le), (false, EquationSense::Ge)] {
            let bound = self.derived_parameter(
                pse_ids::named_id(base, &format!("bound-{}-{}", upper, member.to_hex())),
                symbol.ty.clone(),
                DerivedRule::Bound {
                    variable: member,
                    upper,
                },
                source,
                realization,
                self.derived_lineage(
                    &symbol.lineage,
                    if upper {
                        "switched_upper"
                    } else {
                        "switched_lower"
                    },
                ),
            )?;
            variables.push(bound);
            rows.push(self.push_row(
                pse_ids::named_id(base, &format!("switched-{}-{}", upper, member.to_hex())),
                sym(member),
                sense,
                bin(BinaryOp::Mul, sym(bound), switch.clone()),
                lineage.clone(),
            )?);
        }
        Ok((rows, variables))
    }

    /// `atmost`/`atleast`/`exactly name[i in s]: k of y[i];`
    pub(super) fn cardinality(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let v = row
            .value
            .cardinality
            .clone()
            .ok_or_else(|| invalid(at, "cardinality payload"))?;
        let count = self.eval(at, env, &v.count, None)?.scalar(at)?;
        if !(count.is_finite()
            && count >= 0.0
            && count.fract() == 0.0
            && count <= f64::from(u32::MAX))
        {
            return Err(invalid(at, "a cardinality count is a nonnegative integer"));
        }
        let mut members = Vec::new();
        for coordinates in self.coordinates(
            at,
            env,
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str())),
        )? {
            let local = coordinates_env(env, &coordinates);
            members.push(self.variable_operand(instance, &v.member, &local, at)?);
        }
        let realized = self.realization(Form::Cardinality, at)?;
        let lineage = self.lineage(instance, row, &[at]);
        let base = member_id(instance, at, &[]);
        let sense = match row.value.kind {
            Kind::Atmost => EquationSense::Le,
            Kind::Atleast => EquationSense::Ge,
            _ => EquationSense::Eq,
        };
        let binary = members
            .iter()
            .all(|id| self.model.symbols[id].domain == Domain::Binary);
        let limit = sym(self.indicator_constant(instance, count, at)?);
        if binary {
            // Over binaries the count is one exact linear row on every backend.
            let total = sum(members.iter().map(|id| sym(*id)).collect())
                .ok_or_else(|| invalid(at, "cardinality without members"))?;
            let row = self.push_row(
                pse_ids::named_id(base, "count"),
                total,
                sense,
                limit,
                lineage,
            )?;
            self.record(
                at,
                instance,
                realized.policy(),
                Equivalence::Exact,
                vec![row],
                vec![],
            );
            return Ok(());
        }
        if sense != EquationSense::Le {
            return Err(ModelingError::Unsupported {
                declaration: at.into(),
                capability:
                    "a lower cardinality over nonbinary members, which needs a nonzero threshold"
                        .into(),
            });
        }
        if realized == Realized::Other(Policy::Native) {
            self.model.native.push(NativeConstraint::Cardinality {
                members: members.clone(),
                bound: count as u32,
            });
            self.record(
                at,
                instance,
                Policy::Native,
                Equivalence::Native,
                vec![],
                vec![],
            );
            return Ok(());
        }
        let indicator = self.indicator_type(at)?;
        let mut rows = Vec::new();
        let mut variables = Vec::new();
        let mut selectors = Vec::new();
        for (k, member) in members.iter().enumerate() {
            let z = self.derived_variable(
                pse_ids::named_id(base, &format!("nonzero-{k}")),
                indicator.clone(),
                Domain::Binary,
                self.derived_lineage(&lineage, &format!("nonzero_{k}")),
                at,
            )?;
            variables.push(z);
            selectors.push(sym(z));
            let (rows_k, bounds) =
                self.switched_bounds(at, base, *member, sym(z), Policy::Linear, &lineage)?;
            rows.extend(rows_k);
            variables.extend(bounds);
        }
        let total = sum(selectors).ok_or_else(|| invalid(at, "cardinality without members"))?;
        rows.push(self.push_row(
            pse_ids::named_id(base, "count"),
            total,
            sense,
            limit,
            lineage,
        )?);
        self.record(
            at,
            instance,
            Policy::Linear,
            Equivalence::Exact,
            rows,
            variables,
        );
        Ok(())
    }

    /// `piecewise name[k in s]: y == x at (X[k], Y[k]);` over breakpoints in set order.
    #[expect(
        clippy::too_many_lines,
        reason = "the SOS2 and incremental formulations share breakpoint admission"
    )]
    pub(super) fn piecewise(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let v = row
            .value
            .piecewise
            .clone()
            .ok_or_else(|| invalid(at, "piecewise payload"))?;
        if v.indices.len() != 1 {
            return Err(invalid(at, "a piecewise function has one breakpoint index"));
        }
        let rewrite = |engine: &mut Self, text: &str, local: &Environment| -> Result<Expr> {
            let e = dsl::parse_expr(text).map_err(|e| invalid(at, e.to_string()))?;
            engine.rewrite(instance, &e, local, &[at])
        };
        let output = rewrite(self, &v.output, env)?;
        let input = rewrite(self, &v.input, env)?;
        let mut points = Vec::new();
        for coordinates in self.coordinates(
            at,
            env,
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str())),
        )? {
            let local = coordinates_env(env, &coordinates);
            points.push((
                rewrite(self, &v.abscissa, &local)?,
                rewrite(self, &v.ordinate, &local)?,
            ));
        }
        if points.len() < 2 {
            return Err(invalid(
                at,
                "a piecewise function needs two or more breakpoints",
            ));
        }
        let realized = self.realization(Form::Piecewise, at)?;
        let lineage = self.lineage(instance, row, &[at]);
        let base = member_id(instance, at, &[]);
        let indicator = self.indicator_type(at)?;
        let one = self.indicator_one(instance, at)?;
        let mut rows = Vec::new();
        let mut variables = Vec::new();
        let unit = |engine: &mut Self,
                    name: &str,
                    variables: &mut Vec<SemanticId>|
         -> Result<SemanticId> {
            let id = engine.derived_variable(
                pse_ids::named_id(base, name),
                indicator.clone(),
                Domain::Continuous,
                engine.derived_lineage(&lineage, &name.replace('-', "_")),
                at,
            )?;
            engine.model.unit_interval.insert(id);
            variables.push(id);
            Ok(id)
        };
        let n = points.len();
        let equivalence = if realized == Realized::Other(Policy::Incremental) {
            // x = X0 + Σ δj (Xj+1 − Xj), with δj+1 ≤ wj ≤ δj filling segments in order.
            let mut deltas = Vec::new();
            for j in 0..n - 1 {
                deltas.push(unit(self, &format!("fill-{j}"), &mut variables)?);
            }
            for j in 0..n - 2 {
                let w = self.derived_variable(
                    pse_ids::named_id(base, &format!("order-{j}")),
                    indicator.clone(),
                    Domain::Binary,
                    self.derived_lineage(&lineage, &format!("order_{j}")),
                    at,
                )?;
                variables.push(w);
                rows.push(self.push_row(
                    pse_ids::named_id(w, "after"),
                    sym(deltas[j + 1]),
                    EquationSense::Le,
                    sym(w),
                    lineage.clone(),
                )?);
                rows.push(self.push_row(
                    pse_ids::named_id(w, "before"),
                    sym(w),
                    EquationSense::Le,
                    sym(deltas[j]),
                    lineage.clone(),
                )?);
            }
            for (name, target, pick) in [("input", input, 0usize), ("output", output, 1)] {
                let coordinate = |k: usize| {
                    if pick == 0 {
                        points[k].0.clone()
                    } else {
                        points[k].1.clone()
                    }
                };
                let mut terms = vec![coordinate(0)];
                for (j, delta) in deltas.iter().enumerate() {
                    terms.push(bin(
                        BinaryOp::Mul,
                        sym(*delta),
                        bin(BinaryOp::Sub, coordinate(j + 1), coordinate(j)),
                    ));
                }
                let total =
                    sum(terms).ok_or_else(|| invalid(at, "piecewise without breakpoints"))?;
                rows.push(self.push_row(
                    pse_ids::named_id(base, name),
                    target,
                    EquationSense::Eq,
                    total,
                    lineage.clone(),
                )?);
            }
            Equivalence::Exact
        } else {
            // x = Σ λk Xk, y = Σ λk Yk, Σ λk = 1, with λ an SOS2 set.
            let mut weights = Vec::new();
            for k in 0..n {
                weights.push(unit(self, &format!("weight-{k}"), &mut variables)?);
            }
            let total = sum(weights.iter().map(|id| sym(*id)).collect())
                .ok_or_else(|| invalid(at, "piecewise without breakpoints"))?;
            rows.push(self.push_row(
                pse_ids::named_id(base, "convex"),
                total,
                EquationSense::Eq,
                sym(one),
                lineage.clone(),
            )?);
            for (name, target, pick) in [("input", input, 0usize), ("output", output, 1)] {
                let terms = weights
                    .iter()
                    .enumerate()
                    .map(|(k, w)| {
                        bin(
                            BinaryOp::Mul,
                            sym(*w),
                            if pick == 0 {
                                points[k].0.clone()
                            } else {
                                points[k].1.clone()
                            },
                        )
                    })
                    .collect();
                let combination =
                    sum(terms).ok_or_else(|| invalid(at, "piecewise without breakpoints"))?;
                rows.push(self.push_row(
                    pse_ids::named_id(base, name),
                    target,
                    EquationSense::Eq,
                    combination,
                    lineage.clone(),
                )?);
            }
            if realized == Realized::Other(Policy::Native) {
                self.model.native.push(NativeConstraint::Sos {
                    form: NativeConstraintForm::Sos2,
                    members: weights
                        .iter()
                        .enumerate()
                        .map(|(k, w)| (*w, k as f64))
                        .collect(),
                });
                Equivalence::Native
            } else {
                // SOS2 by segment binaries: exactly one segment, λk only on its ends.
                let mut segments = Vec::new();
                for j in 0..n - 1 {
                    let z = self.derived_variable(
                        pse_ids::named_id(base, &format!("segment-{j}")),
                        indicator.clone(),
                        Domain::Binary,
                        self.derived_lineage(&lineage, &format!("segment_{j}")),
                        at,
                    )?;
                    variables.push(z);
                    segments.push(z);
                }
                let chosen = sum(segments.iter().map(|id| sym(*id)).collect())
                    .ok_or_else(|| invalid(at, "piecewise without segments"))?;
                rows.push(self.push_row(
                    pse_ids::named_id(base, "one-segment"),
                    chosen,
                    EquationSense::Eq,
                    sym(one),
                    lineage.clone(),
                )?);
                for (k, weight) in weights.iter().enumerate() {
                    let ends = [k.checked_sub(1), (k < n - 1).then_some(k)]
                        .into_iter()
                        .flatten()
                        .map(|j| sym(segments[j]))
                        .collect();
                    let ends =
                        sum(ends).ok_or_else(|| invalid(at, "piecewise without segments"))?;
                    rows.push(self.push_row(
                        pse_ids::named_id(*weight, "adjacent"),
                        sym(*weight),
                        EquationSense::Le,
                        ends,
                        lineage.clone(),
                    )?);
                }
                Equivalence::Exact
            }
        };
        self.record(
            at,
            instance,
            realized.policy(),
            equivalence,
            rows,
            variables,
        );
        Ok(())
    }

    /// `complements name: (first >= 0, second >= 0);`, the pair 0 <= first ⊥ second >= 0,
    /// lowered by its declared realization (ADR-0104 §5, improvement I12). Every lowering
    /// keeps both members nonnegative:
    /// - `smooth(f, width)`: the row `f(first, second, width) == 0` with the package's
    ///   smoothing function, CHKS `(a + b − sqrt((a − b)² + width²))/2` for `smooth_min`.
    ///   It holds exactly where `first·second = width²/4` with both members positive, so
    ///   no inequality row is added and a square system stays square. A width
    ///   referencing a parameter is continued as a value;
    /// - `disjunctive`: nonnegative slack columns equal to each member, one native SOS1
    ///   over them;
    /// - `penalty(l1)`: the rows `first >= 0`, `second >= 0` and `first*second <= 0`, and
    ///   the structural requirement of the l1 exact-penalty route.
    pub(super) fn complementarity(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        coordinates: &[(String, Value)],
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let v = row
            .value
            .complementarity
            .clone()
            .ok_or_else(|| invalid(at, "complementarity payload"))?;
        let realized = self.realization(Form::Complementarity, at)?;
        let local = coordinates_env(env, coordinates);
        let base = member_id(instance, at, coordinates);
        let lineage = self.lineage(instance, row, &[at]);
        let member = |engine: &mut Self, text: &str| -> Result<(Expr, Type)> {
            let parsed = dsl::parse_expr(text).map_err(|e| invalid(at, e.to_string()))?;
            let expression = engine.rewrite(instance, &parsed, &local, &[at])?;
            let contracts = engine.model.function_contracts(engine.p);
            let ty = engine.type_of(&expression, &contracts, at)?;
            Ok((expression, ty))
        };
        let (first, first_ty) = member(self, &v.first)?;
        let (second, second_ty) = member(self, &v.second)?;
        let mut rows = Vec::new();
        let mut variables = Vec::new();
        let nonnegative = |engine: &mut Self,
                           rows: &mut Vec<SemanticId>,
                           name: &str,
                           value: &Expr,
                           ty: &Type|
         -> Result<()> {
            let zero = engine.typed_zero(ty, at)?;
            rows.push(engine.push_row(
                pse_ids::named_id(base, name),
                value.clone(),
                EquationSense::Ge,
                zero,
                lineage.clone(),
            )?);
            Ok(())
        };
        let equivalence = match &realized {
            Realized::Smooth { function, width } => {
                // The smoothing function is the package's, called as authored.
                let call = dsl::parse_expr(&format!(
                    "{function}(({}), ({}), ({width}))",
                    v.first, v.second
                ))
                .map_err(|e| invalid(at, e.to_string()))?;
                let smoothed = self.rewrite(instance, &call, &local, &[at])?;
                let contracts = self.model.function_contracts(self.p);
                let ty = self.type_of(&smoothed, &contracts, at)?;
                let zero = self.typed_zero(&ty, at)?;
                rows.push(self.push_row(
                    pse_ids::named_id(base, "complementarity-smooth"),
                    smoothed,
                    EquationSense::Eq,
                    zero,
                    lineage.clone(),
                )?);
                Equivalence::Smoothed
            }
            Realized::Other(Policy::Disjunctive) => {
                let mut slacks = Vec::new();
                for (name, value, ty) in [
                    ("first", &first, &first_ty),
                    ("second", &second, &second_ty),
                ] {
                    let slack = self.derived_variable(
                        pse_ids::named_id(base, &format!("complementarity-slack-{name}")),
                        ty.clone(),
                        Domain::Continuous,
                        self.derived_lineage(&lineage, &format!("slack_{name}")),
                        at,
                    )?;
                    self.model.nonnegative.insert(slack);
                    rows.push(self.push_row(
                        pse_ids::named_id(base, &format!("complementarity-{name}")),
                        sym(slack),
                        EquationSense::Eq,
                        value.clone(),
                        lineage.clone(),
                    )?);
                    variables.push(slack);
                    slacks.push(slack);
                }
                self.model.native.push(NativeConstraint::Sos {
                    form: NativeConstraintForm::Sos1,
                    members: slacks.iter().copied().zip([1.0, 2.0]).collect(),
                });
                Equivalence::Native
            }
            Realized::Other(Policy::PenaltyL1) => {
                nonnegative(self, &mut rows, "complementarity-first", &first, &first_ty)?;
                nonnegative(
                    self,
                    &mut rows,
                    "complementarity-second",
                    &second,
                    &second_ty,
                )?;
                let product = bin(BinaryOp::Mul, first, second);
                let contracts = self.model.function_contracts(self.p);
                let ty = self.type_of(&product, &contracts, at)?;
                let zero = self.typed_zero(&ty, at)?;
                rows.push(self.push_row(
                    pse_ids::named_id(base, "complementarity-product"),
                    product,
                    EquationSense::Le,
                    zero,
                    lineage.clone(),
                )?);
                self.model
                    .requirements
                    .insert(ModelingStructuralRequirement::L1ExactPenalty);
                Equivalence::ExactPenalty
            }
            _ => return Err(invalid(at, "complementarity realization expected")),
        };
        self.record(
            at,
            instance,
            realized.policy(),
            equivalence,
            rows,
            variables,
        );
        Ok(())
    }

    /// `logic name: proposition;` over binary variables, lowered exactly.
    pub(super) fn logic(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        coordinates: &[(String, Value)],
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let v = row
            .value
            .logic
            .clone()
            .ok_or_else(|| invalid(at, "logic payload"))?;
        let proposition = crate::logic::parse(&v.proposition).map_err(|e| invalid(at, e))?;
        let realized = self.realization(Form::Logic, at)?;
        let native = realized == Realized::Other(Policy::Native);
        let mut lowering = LogicLowering {
            instance,
            at,
            base: member_id(instance, at, coordinates),
            env: coordinates_env(env, coordinates),
            lineage: self.lineage(instance, row, &[at]),
            native,
            one: self.indicator_one(instance, at)?,
            indicator: self.indicator_type(at)?,
            serial: 0,
            rows: Vec::new(),
            variables: Vec::new(),
        };
        lowering.assert(self, &proposition)?;
        let LogicLowering {
            rows, variables, ..
        } = lowering;
        self.record(
            at,
            instance,
            realized.policy(),
            if native {
                Equivalence::Native
            } else {
                Equivalence::Exact
            },
            rows,
            variables,
        );
        Ok(())
    }
}

/// Exact linear (or native) lowering of one proposition by auxiliary resultants.
struct LogicLowering {
    instance: InstanceId,
    at: DeclarationId,
    base: SemanticId,
    env: Environment,
    lineage: Lineage,
    native: bool,
    one: SemanticId,
    indicator: Type,
    serial: usize,
    rows: Vec<SemanticId>,
    variables: Vec<SemanticId>,
}
impl LogicLowering {
    fn row(
        &mut self,
        engine: &mut Engine<'_, '_>,
        lhs: Expr,
        sense: EquationSense,
        rhs: Expr,
    ) -> Result<()> {
        self.serial += 1;
        let id = pse_ids::named_id(self.base, &format!("logic-row-{}", self.serial));
        self.rows
            .push(engine.push_row(id, lhs, sense, rhs, self.lineage.clone())?);
        Ok(())
    }
    fn resultant(&mut self, engine: &mut Engine<'_, '_>) -> Result<SemanticId> {
        self.serial += 1;
        let id = engine.derived_variable(
            pse_ids::named_id(self.base, &format!("logic-resultant-{}", self.serial)),
            self.indicator.clone(),
            Domain::Binary,
            engine.derived_lineage(&self.lineage, &format!("resultant_{}", self.serial)),
            self.at,
        )?;
        self.variables.push(id);
        Ok(id)
    }
    /// A literal: a binary variable and whether the proposition is its complement.
    fn literal(
        &mut self,
        engine: &mut Engine<'_, '_>,
        p: &Proposition,
    ) -> Result<(SemanticId, bool)> {
        Ok(match p {
            Proposition::Atom(text) => (
                engine.binary_operand(self.instance, text, &self.env, self.at)?,
                false,
            ),
            Proposition::Not(inner) => {
                let (variable, negated) = self.literal(engine, inner)?;
                (variable, !negated)
            }
            other => (self.define(engine, other)?, false),
        })
    }
    fn value(&self, (variable, negated): (SemanticId, bool)) -> Expr {
        if negated {
            bin(BinaryOp::Sub, sym(self.one), sym(variable))
        } else {
            sym(variable)
        }
    }
    fn operands(
        &mut self,
        engine: &mut Engine<'_, '_>,
        ps: &[Proposition],
    ) -> Result<Vec<(SemanticId, bool)>> {
        ps.iter().map(|p| self.literal(engine, p)).collect()
    }
    /// A resultant equal to the truth of a compound proposition.
    fn define(&mut self, engine: &mut Engine<'_, '_>, p: &Proposition) -> Result<SemanticId> {
        let (form, operands) = match p {
            Proposition::And(ps) => (NativeConstraintForm::And, self.operands(engine, ps)?),
            Proposition::Or(ps) => (NativeConstraintForm::Or, self.operands(engine, ps)?),
            Proposition::Implies(a, b) => {
                let (a, negated) = self.literal(engine, a)?;
                (
                    NativeConstraintForm::Or,
                    vec![(a, !negated), self.literal(engine, b)?],
                )
            }
            Proposition::Xor(a, b) => (
                NativeConstraintForm::Xor,
                vec![self.literal(engine, a)?, self.literal(engine, b)?],
            ),
            Proposition::Exactly(..) => {
                return Err(ModelingError::Unsupported {
                    declaration: self.at.into(),
                    capability: "exactly(k, ...) nested inside another proposition".into(),
                });
            }
            Proposition::Atom(_) | Proposition::Not(_) => {
                return Err(invalid(self.at, "a literal needs no resultant"));
            }
        };
        let z = self.resultant(engine)?;
        if self.native {
            engine.model.native.push(NativeConstraint::Logic {
                form,
                resultant: z,
                operands: operands
                    .iter()
                    .map(|(variable, negated)| LogicOperand {
                        variable: *variable,
                        negated: *negated,
                    })
                    .collect(),
            });
            return Ok(z);
        }
        let values = operands.iter().map(|o| self.value(*o)).collect::<Vec<_>>();
        let total = sum(values.clone()).ok_or_else(|| invalid(self.at, "empty proposition"))?;
        match form {
            NativeConstraintForm::And => {
                for value in &values {
                    self.row(engine, sym(z), EquationSense::Le, value.clone())?;
                }
                let slack = sym(engine.indicator_constant(
                    self.instance,
                    (values.len() - 1) as f64,
                    self.at,
                )?);
                self.row(
                    engine,
                    sym(z),
                    EquationSense::Ge,
                    bin(BinaryOp::Sub, total, slack),
                )?;
            }
            NativeConstraintForm::Or => {
                for value in &values {
                    self.row(engine, sym(z), EquationSense::Ge, value.clone())?;
                }
                self.row(engine, sym(z), EquationSense::Le, total)?;
            }
            _ => {
                let (a, b) = (values[0].clone(), values[1].clone());
                self.row(
                    engine,
                    sym(z),
                    EquationSense::Ge,
                    bin(BinaryOp::Sub, a.clone(), b.clone()),
                )?;
                self.row(
                    engine,
                    sym(z),
                    EquationSense::Ge,
                    bin(BinaryOp::Sub, b.clone(), a.clone()),
                )?;
                self.row(engine, sym(z), EquationSense::Le, total)?;
                let two = sym(engine.indicator_constant(self.instance, 2.0, self.at)?);
                self.row(
                    engine,
                    sym(z),
                    EquationSense::Le,
                    bin(BinaryOp::Sub, bin(BinaryOp::Sub, two, a), b),
                )?;
            }
        }
        Ok(z)
    }
    /// Assert a proposition true with the fewest auxiliary variables.
    fn assert(&mut self, engine: &mut Engine<'_, '_>, p: &Proposition) -> Result<()> {
        let one = sym(self.one);
        match p {
            Proposition::And(ps) => {
                for q in ps {
                    self.assert(engine, q)?;
                }
                Ok(())
            }
            Proposition::Exactly(count, ps) => {
                let k = engine
                    .eval(self.at, &self.env, count, None)?
                    .scalar(self.at)?;
                if !(k.is_finite() && k >= 0.0 && k.fract() == 0.0) {
                    return Err(invalid(
                        self.at,
                        "exactly(k, ...) needs a nonnegative integer",
                    ));
                }
                let values = self
                    .operands(engine, ps)?
                    .into_iter()
                    .map(|o| self.value(o))
                    .collect();
                let total = sum(values).ok_or_else(|| invalid(self.at, "empty proposition"))?;
                let count = sym(engine.indicator_constant(self.instance, k, self.at)?);
                self.row(engine, total, EquationSense::Eq, count)
            }
            _ if self.native => {
                let (variable, negated) = self.literal(engine, p)?;
                let value = self.value((variable, negated));
                self.row(engine, value, EquationSense::Eq, one)
            }
            Proposition::Or(ps) => {
                let values = self
                    .operands(engine, ps)?
                    .into_iter()
                    .map(|o| self.value(o))
                    .collect();
                let total = sum(values).ok_or_else(|| invalid(self.at, "empty proposition"))?;
                self.row(engine, total, EquationSense::Ge, one)
            }
            Proposition::Implies(a, b) => {
                let a = self.literal(engine, a)?;
                let b = self.literal(engine, b)?;
                let (a, b) = (self.value(a), self.value(b));
                self.row(engine, a, EquationSense::Le, b)
            }
            Proposition::Xor(a, b) => {
                let a = self.literal(engine, a)?;
                let b = self.literal(engine, b)?;
                let total = bin(BinaryOp::Add, self.value(a), self.value(b));
                self.row(engine, total, EquationSense::Eq, one)
            }
            Proposition::Atom(_) | Proposition::Not(_) => {
                let literal = self.literal(engine, p)?;
                let value = self.value(literal);
                self.row(engine, value, EquationSense::Eq, one)
            }
        }
    }
}

fn sym(id: SemanticId) -> Expr {
    symbol_expr(id)
}
fn bin(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr {
        kind: ExprKind::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        span: Span::default(),
    }
}
fn neg(value: Expr) -> Expr {
    Expr {
        kind: ExprKind::Neg(Box::new(value)),
        span: Span::default(),
    }
}
/// A balanced sum keeps generated rows shallow.
fn sum(mut terms: Vec<Expr>) -> Option<Expr> {
    while terms.len() > 1 {
        let mut next = Vec::with_capacity(terms.len().div_ceil(2));
        let mut iter = terms.into_iter();
        while let Some(a) = iter.next() {
            next.push(match iter.next() {
                Some(b) => bin(BinaryOp::Add, a, b),
                None => a,
            });
        }
        terms = next;
    }
    terms.pop()
}
