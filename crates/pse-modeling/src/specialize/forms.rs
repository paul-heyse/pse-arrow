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
use crate::resolved::{
    Binding, Node, NodeIx, Reference, Source,
    construction::{Rewritten, RewrittenEquation},
};
use pse_authoring::dsl::Proposition;
use pse_model::{
    forms::{LogicOperand, NativeConstraint},
    generated::enums::{
        ModelingRealizationPolicy as Policy, ModelingStructuralRequirement, NativeConstraintForm,
    },
};

macro_rules! form_sym {
    ($engine:expr,$id:expr) => {{
        let id = $id;
        let source = $engine.form_source()?;
        $engine.selected_scalar(id, source)?
    }};
}
macro_rules! form_bin {
    ($engine:expr,$op:expr,$lhs:expr,$rhs:expr $(,)?) => {{
        let lhs = $lhs;
        let rhs = $rhs;
        let source = $engine.form_source()?;
        $engine.generated_binary($op, lhs, rhs, source)?
    }};
}
macro_rules! form_neg {
    ($engine:expr,$value:expr) => {{
        let value = $value;
        let source = $engine.form_source()?;
        $engine.generated_neg(value, source)?
    }};
}
macro_rules! form_sum {
    ($engine:expr,$values:expr) => {{
        let values = $values;
        let source = $engine.form_source()?;
        $engine.form_balanced_sum(values, source)?
    }};
}
macro_rules! form_zero {
    ($engine:expr,$ty:expr,$at:expr) => {{
        let ty = $ty;
        let source = $engine.form_source()?;
        $engine.constant(0.0, ty, $at, source)?
    }};
}
macro_rules! form_row {
    ($engine:expr,$id:expr,$lhs:expr,$sense:expr,$rhs:expr,$lineage:expr $(,)?) => {{
        let id = $id;
        let lhs = $lhs;
        let sense = $sense;
        let rhs = $rhs;
        let lineage = $lineage;
        $engine.push_row(id, lhs, sense, rhs, lineage)
    }};
}
macro_rules! logic_row {
    ($lowering:expr,$engine:expr,$lhs:expr,$sense:expr,$rhs:expr $(,)?) => {{
        let lhs = $lhs;
        let sense = $sense;
        let rhs = $rhs;
        $lowering.row($engine, lhs, sense, rhs)
    }};
}
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
    BigM(Expr),
    DerivedBigM(f64),
    Hull(Option<f64>),
    /// The authored smoothing function and width expression of a complementarity.
    Smooth {
        function: Path,
        width: Expr,
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
    lhs: Rewritten,
    sense: EquationSense,
    rhs: Rewritten,
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
    fn with_form_source<T>(
        &mut self,
        at: DeclarationId,
        apply: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.form_sources.push(Source {
            field: crate::expression::occurrences::OccurrenceKey {
                declaration: at,
                role: "derived.constraint_form".into(),
                position: 0,
            },
            span: Span::default(),
            coordinate: None,
        });
        let result = apply(self);
        self.form_sources.pop();
        result
    }
    fn form_source(&self) -> Result<Source> {
        self.form_sources.last().cloned().ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "form construction has no actual original source owner",
            )
        })
    }
    fn form_balanced_sum(
        &mut self,
        mut values: Vec<Rewritten>,
        source: Source,
    ) -> Result<Option<Rewritten>> {
        if values.is_empty() {
            return Ok(None);
        }
        while values.len() > 1 {
            let mut next = Vec::with_capacity(values.len().div_ceil(2));
            let mut previous = values.into_iter();
            while let Some(lhs) = previous.next() {
                next.push(if let Some(rhs) = previous.next() {
                    self.generated_binary(BinaryOp::Add, lhs, rhs, source.clone())?
                } else {
                    lhs
                });
            }
            values = next;
        }
        Ok(values.pop())
    }

    /// Register a realization declaration whose target is a constraint form or disjunction.
    pub(super) fn register_form_realization(
        &mut self,
        instance: InstanceId,
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
            (Policy::Smooth, Some(_)) => Realized::Smooth {
                function: match &self
                    .p
                    .expression_at(declaration, "realization.function", 0)?
                    .kind
                {
                    ExprKind::Path(path) => path.clone(),
                    _ => {
                        return Err(invalid(
                            declaration,
                            "smoothing function requires a checked path",
                        ));
                    }
                },
                width: self
                    .p
                    .expression_at(declaration, "realization.argument", 0)?
                    .clone(),
            },
            (Policy::BigM, Some(_)) => Realized::BigM(
                self.p
                    .expression_at(declaration, "realization.argument", 0)?
                    .clone(),
            ),
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
            .insert((instance, target), (realized, declaration))
            .is_some()
        {
            return Err(invalid(declaration, "competing form realizations"));
        }
        Ok(())
    }
    fn realization(
        &self,
        form: Form,
        instance: InstanceId,
        source: DeclarationId,
    ) -> Result<Realized> {
        let realized = match self.form_realizations.get(&(instance, source)) {
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
                definition: None,
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
                definition: None,
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
        let zero = form_zero!(self, &ty, at);
        self.reserve(1)?;
        self.model.symbols.insert(
            id,
            Symbol {
                definition: None,
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
            value: crate::annotation::AnnotationValue::Start(zero.inspection),
            roots: crate::annotation::AnnotationRoots::Scalar { value: zero.root },
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
                definition: None,
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
        lhs: Rewritten,
        sense: EquationSense,
        rhs: Rewritten,
        lineage: Lineage,
    ) -> Result<SemanticId> {
        self.reserve(1)?;
        self.retain_row(
            id,
            Self::generated_relation(lhs, sense, rhs, Span::default()),
            lineage,
        )?;
        Ok(id)
    }
    fn type_of(
        &self,
        expression: &Rewritten,
        _contracts: &dyn crate::FunctionAdmission,
        _at: DeclarationId,
    ) -> Result<Type> {
        self.constructed_type(expression.root)
    }
    /// Resolve an authored member path to an existing binary variable.
    fn binary_operand(
        &mut self,
        instance: InstanceId,
        role: &str,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<SemanticId> {
        let expression = self.p.expression_at(at, role, 0)?.clone();
        let expression = self.with_original_expression(at, role, 0, &expression, |engine| {
            engine.rewrite(instance, &expression, env, &[at])
        })?;
        self.symbol_of_root(expression.root)
            .filter(|id| {
                self.model
                    .symbols
                    .get(id)
                    .is_some_and(|s| s.role == Kind::Variable && s.domain == Domain::Binary)
            })
            .ok_or_else(|| invalid(at, format!("{role} must name a binary variable")))
    }
    fn binary_operand_ast(
        &mut self,
        instance: InstanceId,
        expression: &Expr,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<SemanticId> {
        let expression = self.rewrite(instance, expression, env, &[at])?;
        self.symbol_of_root(expression.root)
            .filter(|id| {
                self.model
                    .symbols
                    .get(id)
                    .is_some_and(|s| s.role == Kind::Variable && s.domain == Domain::Binary)
            })
            .ok_or_else(|| {
                invalid(
                    at,
                    format!(
                        "{} must name a binary variable",
                        dsl::render_expr(&expression.inspection)
                    ),
                )
            })
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
        equation: RewrittenEquation,
        env: &Environment,
    ) -> Result<()> {
        self.with_form_source(row.declaration_id, |engine| {
            engine.indicator_equation_inner(instance, row, coordinates, equation, env)
        })
    }
    fn indicator_equation_inner(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        coordinates: &[(String, Value)],
        equation: RewrittenEquation,
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
        let variable = self.binary_operand(instance, "equation.condition.variable", &local, at)?;
        let (lhs, sense, rhs) = match equation.inspection.kind {
            EquationKind::Relation { lhs, sense, rhs } => (
                Rewritten {
                    root: equation.lhs,
                    inspection: lhs,
                },
                sense,
                Rewritten {
                    root: equation.rhs,
                    inspection: rhs,
                },
            ),
            _ => return Err(invalid(at, "an indicator constraint is a relation")),
        };
        let id = member_id(instance, at, coordinates);
        let lineage = self.lineage(instance, row, &[at]);
        let realized = self.realization(Form::Indicator, instance, at)?;
        if realized == Realized::Other(Policy::Indicator) {
            let row = form_row!(self, id, lhs, sense, rhs, lineage)?;
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
            form_bin!(
                self,
                BinaryOp::Sub,
                form_sym!(self, one),
                form_sym!(self, variable)
            )
        } else {
            form_sym!(self, variable)
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
        lhs: Rewritten,
        sense: EquationSense,
        rhs: Rewritten,
        relax: Rewritten,
        realized: &Realized,
        lineage: Lineage,
        env: &Environment,
    ) -> Result<(Vec<SemanticId>, Vec<SemanticId>)> {
        let residual = form_bin!(self, BinaryOp::Sub, lhs, rhs);
        let ty = self.type_of(&residual, &self.model.function_contracts(self.p), source)?;
        let mut variables = Vec::new();
        let (residual, upper, lower) = match realized {
            Realized::BigM(expression) => {
                let owner = self
                    .form_realizations
                    .get(&(instance, source))
                    .map(|(_, owner)| *owner)
                    .ok_or_else(|| {
                        invalid(source, "authored big-M lacks its actual realization owner")
                    })?;
                let m = self.with_original_expression(
                    owner,
                    "realization.argument",
                    0,
                    expression,
                    |engine| engine.rewrite(instance, expression, env, &[owner]),
                )?;
                if self.constructed_type(m.root)? != ty {
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
                            definition: Some(m.root),
                            id: m_id,
                            ty: ty.clone(),
                            role: Kind::Let,
                            domain: Domain::Continuous,
                            expression: Some(m.inspection),
                            initial: None,
                            lineage: self.derived_lineage(&lineage, "big_m"),
                        },
                    );
                }
                variables.push(m_id);
                (
                    residual,
                    form_sym!(self, m_id),
                    form_neg!(self, form_sym!(self, m_id)),
                )
            }
            Realized::DerivedBigM(margin) => {
                let residual_id = pse_ids::named_id(id, "disjunct-residual");
                self.reserve(1)?;
                self.model.symbols.insert(
                    residual_id,
                    Symbol {
                        definition: Some(residual.root),
                        id: residual_id,
                        ty: ty.clone(),
                        role: Kind::Let,
                        domain: Domain::Continuous,
                        expression: Some(residual.inspection),
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
                let mut side = |upper: bool, engine: &mut Self| -> Result<Rewritten> {
                    let needed = if upper {
                        sense != EquationSense::Ge
                    } else {
                        sense != EquationSense::Le
                    };
                    if !needed {
                        return Ok(form_sym!(engine, residual_id));
                    }
                    let id = bound(upper, engine)?;
                    variables.push(id);
                    Ok(form_sym!(engine, id))
                };
                let up = side(true, self)?;
                let down = side(false, self)?;
                (form_sym!(self, residual_id), up, down)
            }
            _ => return Err(invalid(source, "big-M realization expected")),
        };
        let mut rows = Vec::new();
        if matches!(sense, EquationSense::Le | EquationSense::Eq) {
            rows.push(form_row!(
                self,
                pse_ids::named_id(id, "big-m-upper-row"),
                residual.clone(),
                EquationSense::Le,
                form_bin!(self, BinaryOp::Mul, upper, relax.clone()),
                lineage.clone(),
            )?);
        }
        if matches!(sense, EquationSense::Ge | EquationSense::Eq) {
            rows.push(form_row!(
                self,
                pse_ids::named_id(id, "big-m-lower-row"),
                residual,
                EquationSense::Ge,
                form_bin!(self, BinaryOp::Mul, lower, relax),
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
        self.with_form_source(declaration, |engine| {
            engine.disjunction_inner(instance, declaration, env, parent)
        })
    }
    fn disjunction_inner(
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
                    self.register_form_realization(instance, target, child)?;
                }
            }
        }
        let realized = self.realization(Form::Disjunction, instance, declaration)?;
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
        let selected = form_sum!(
            self,
            indicators
                .iter()
                .map(|id| Ok(form_sym!(self, *id)))
                .collect::<Result<Vec<_>>>()?
        )
        .ok_or_else(|| invalid(declaration, "disjunction without alternatives"))?;
        let mut rows = vec![form_row!(
            self,
            pse_ids::named_id(base, "select-one"),
            selected,
            EquationSense::Eq,
            form_sym!(self, parent.unwrap_or(one)),
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
                        let row = form_row!(self, r.id, r.lhs, r.sense, r.rhs, r.lineage)?;
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
                        let relax = form_bin!(
                            self,
                            BinaryOp::Sub,
                            form_sym!(self, one),
                            form_sym!(self, *indicator)
                        );
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
                        instance,
                        child,
                        env,
                        e.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        let equation = self.rewrite_equation_field(
                            instance,
                            child,
                            "equation.expression",
                            0,
                            &coordinates_env(env, &coordinates),
                            &[child],
                        )?;
                        let (lhs, sense, rhs) = match equation.inspection.kind {
                            EquationKind::Relation { lhs, sense, rhs } => (
                                Rewritten {
                                    root: equation.lhs,
                                    inspection: lhs,
                                },
                                sense,
                                Rewritten {
                                    root: equation.rhs,
                                    inspection: rhs,
                                },
                            ),
                            _ => return Err(invalid(child, "a disjunct row is a relation")),
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
                                        lhs: form_sym!(self,annotation.target),
                                        sense,
                                        rhs:Rewritten {root:match annotation.roots {crate::annotation::AnnotationRoots::Bounds {lower,upper}=>if suffix=="lower" {lower} else {upper},_=>return Err(invalid(child,"disjunct bound lacks its actual annotation roots"))},inspection:bound},
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
        let mut variables = BTreeSet::new();
        for row in disjuncts.iter().flatten() {
            self.collect_variables(&row.lhs, &mut variables)?;
            self.collect_variables(&row.rhs, &mut variables)?;
        }
        let mut rows = Vec::new();
        let mut derived = Vec::new();
        let mut copies = vec![BTreeMap::<SemanticId, Rewritten>::new(); indicators.len()];
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
            zeros.insert(*x, form_sym!(self, zero));
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
                copies[k].insert(*x, form_sym!(self, copy));
                parts.push(form_sym!(self, copy));
                rows.push(form_row!(
                    self,
                    pse_ids::named_id(copy, "hull-upper"),
                    form_sym!(self, copy),
                    EquationSense::Le,
                    form_bin!(
                        self,
                        BinaryOp::Mul,
                        form_sym!(self, upper),
                        form_sym!(self, *indicator)
                    ),
                    lineage.clone(),
                )?);
                rows.push(form_row!(
                    self,
                    pse_ids::named_id(copy, "hull-lower"),
                    form_sym!(self, copy),
                    EquationSense::Ge,
                    form_bin!(
                        self,
                        BinaryOp::Mul,
                        form_sym!(self, lower),
                        form_sym!(self, *indicator)
                    ),
                    lineage.clone(),
                )?);
            }
            let parts = form_sum!(self, parts)
                .ok_or_else(|| invalid(source, "hull without alternatives"))?;
            let link = pse_ids::named_id(base, &format!("hull-link-{}", x.to_hex()));
            match parent {
                None => rows.push(form_row!(
                    self,
                    link,
                    form_sym!(self, *x),
                    EquationSense::Eq,
                    parts,
                    lineage.clone(),
                )?),
                Some(owner) => {
                    // Inside an unselected owner the copies vanish and x keeps its own box.
                    let gap = form_bin!(self, BinaryOp::Sub, form_sym!(self, *x), parts);
                    for (suffix, sense, bound) in [
                        ("upper", EquationSense::Le, upper),
                        ("lower", EquationSense::Ge, lower),
                    ] {
                        rows.push(form_row!(
                            self,
                            pse_ids::named_id(link, suffix),
                            gap.clone(),
                            sense,
                            form_bin!(
                                self,
                                BinaryOp::Sub,
                                form_sym!(self, bound),
                                form_bin!(
                                    self,
                                    BinaryOp::Mul,
                                    form_sym!(self, bound),
                                    form_sym!(self, owner)
                                ),
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
                let g = form_bin!(self, BinaryOp::Sub, r.lhs, r.rhs);
                let ty = self.type_of(&g, &self.model.function_contracts(self.p), source)?;
                let zero = form_zero!(self, &ty, source);
                let mut memo = BTreeMap::new();
                let lhs = match self.classify(&g, &variables, &mut memo)? {
                    Class::Constant | Class::Affine => {
                        // Homogenize the affine row: g(v) − g(0) + g(0)·y.
                        let at_copy = self.substitute(&g, map, &variables, source)?;
                        let at_zero = self.substitute(&g, &zeros, &variables, source)?;
                        form_bin!(
                            self,
                            BinaryOp::Add,
                            form_bin!(self, BinaryOp::Sub, at_copy, at_zero.clone()),
                            form_bin!(self, BinaryOp::Mul, at_zero, form_sym!(self, *indicator)),
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
                        let scale = form_bin!(
                            self,
                            BinaryOp::Add,
                            form_sym!(self, *indicator),
                            form_sym!(self, eps)
                        );
                        let scaled = map
                            .iter()
                            .map(|(x, copy)| {
                                Ok((
                                    *x,
                                    form_bin!(self, BinaryOp::Div, copy.clone(), scale.clone()),
                                ))
                            })
                            .collect::<Result<BTreeMap<_, _>>>()?;
                        form_bin!(
                            self,
                            BinaryOp::Mul,
                            scale,
                            self.substitute(&g, &scaled, &variables, source)?,
                        )
                    }
                };
                rows.push(form_row!(self, r.id, lhs, r.sense, zero, r.lineage)?);
            }
        }
        Ok((rows, derived, perspective))
    }
    /// Original numerical and guard variables reached through actual member definitions.
    fn collect_variables(
        &self,
        expression: &Rewritten,
        out: &mut BTreeSet<SemanticId>,
    ) -> Result<()> {
        let reads = self
            .construction
            .reads(&self.model.symbols, expression.root)?;
        for id in reads.numeric.into_iter().chain(reads.guards) {
            if self
                .model
                .symbols
                .get(&id)
                .is_some_and(|symbol| symbol.role == Kind::Variable)
            {
                out.insert(id);
            }
        }
        Ok(())
    }
    fn depends_root(&self, root: NodeIx, variables: &BTreeSet<SemanticId>) -> Result<bool> {
        let reads = self.construction.reads(&self.model.symbols, root)?;
        Ok(!reads.numeric.is_disjoint(variables) || !reads.guards.is_disjoint(variables))
    }
    fn classify(
        &self,
        expression: &Rewritten,
        variables: &BTreeSet<SemanticId>,
        memo: &mut BTreeMap<NodeIx, Class>,
    ) -> Result<Class> {
        self.classify_root(expression.root, variables, memo)
    }
    fn classify_root(
        &self,
        root: NodeIx,
        variables: &BTreeSet<SemanticId>,
        memo: &mut BTreeMap<NodeIx, Class>,
    ) -> Result<Class> {
        if let Some(class) = memo.get(&root) {
            return Ok(*class);
        }
        memo.insert(root, Class::Nonlinear);
        let class = match self.construction.get(root).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "form dependence operation is outside its typed owner",
            )
        })? {
            Node::Number(_) => Class::Constant,
            Node::Reference(Reference::Symbol(slot)) => {
                let symbol = self.model.symbols.at(*slot).ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "form dependence scalar is outside its actual owner",
                    )
                })?;
                if variables.contains(&symbol.id) {
                    Class::Affine
                } else if let Some(body) = symbol.definition {
                    self.classify_root(body, variables, memo)?
                } else {
                    Class::Constant
                }
            }
            Node::Reference(Reference::Lexical(binding)) => {
                match self.construction.binding_at(*binding)? {
                    Binding::Value(value) => self.classify_root(*value, variables, memo)?,
                    _ => Class::Constant,
                }
            }
            Node::Neg(value)
            | Node::Evidence { value, .. }
            | Node::Sequence { value, .. }
            | Node::Let { body: value, .. } => self.classify_root(*value, variables, memo)?,
            Node::Binary { op, lhs, rhs } => {
                let a = self.classify_root(*lhs, variables, memo)?;
                let b = self.classify_root(*rhs, variables, memo)?;
                match op {
                    BinaryOp::Add | BinaryOp::Sub => a.max(b),
                    BinaryOp::Mul if a == Class::Constant => b,
                    BinaryOp::Mul if b == Class::Constant => a,
                    BinaryOp::Div if b == Class::Constant => a,
                    _ if a == Class::Constant && b == Class::Constant => Class::Constant,
                    _ => Class::Nonlinear,
                }
            }
            Node::Apply { function, args }
                if self
                    .construction
                    .function_at(*function)?
                    .product
                    .reduction
                    .as_ref()
                    .is_some_and(|reduction| {
                        reduction.kind == pse_quantity::ReductionKind::Sum
                    }) =>
            {
                let mut class = Class::Constant;
                for arg in args {
                    class = class.max(self.classify_root(*arg, variables, memo)?);
                }
                class
            }
            _ if self.depends_root(root, variables)? => Class::Nonlinear,
            _ => Class::Constant,
        };
        memo.insert(root, class);
        Ok(class)
    }
    fn substitute(
        &mut self,
        expression: &Rewritten,
        map: &BTreeMap<SemanticId, Rewritten>,
        _variables: &BTreeSet<SemanticId>,
        at: DeclarationId,
    ) -> Result<Rewritten> {
        let replacements = map.iter().map(|(id, value)| (*id, value.root)).collect();
        Ok(Rewritten {
            root: self.substitute_root(expression.root, &replacements, at)?,
            inspection: expression.inspection.clone(),
        })
    }

    /// `sos1`/`sos2 name[i in s]: x[i] weight w[i];`
    pub(super) fn ordered_set(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        self.with_form_source(row.declaration_id, |engine| {
            engine.ordered_set_inner(instance, row, env)
        })
    }
    fn ordered_set_inner(
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
            instance,
            at,
            env,
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str())),
        )? {
            let local = coordinates_env(env, &coordinates);
            let member = self.variable_operand(instance, "ordered_set.member", &local, at)?;
            let weight = self
                .eval_field(at, &local, "ordered_set.weight", 0, None)?
                .scalar(at)?;
            members.push((member, weight));
        }
        members.sort_by(|a, b| a.1.total_cmp(&b.1));
        if members.windows(2).any(|w| w[0].1 >= w[1].1) || members.iter().any(|m| !m.1.is_finite())
        {
            return Err(invalid(at, "ordered set weights are finite and distinct"));
        }
        let realized = self.realization(Form::OrderedSet, instance, at)?;
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
        role: &str,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<SemanticId> {
        let expression = self.p.expression_at(at, role, 0)?.clone();
        let expression = self.with_original_expression(at, role, 0, &expression, |engine| {
            engine.rewrite(instance, &expression, env, &[at])
        })?;
        self.symbol_of_root(expression.root)
            .filter(|id| {
                self.model
                    .symbols
                    .get(id)
                    .is_some_and(|s| s.role == Kind::Variable && s.definition.is_none())
            })
            .ok_or_else(|| invalid(at, format!("{role} must name a variable")))
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
        if let Some(total) = form_sum!(
            self,
            z.iter()
                .map(|id| Ok(form_sym!(self, *id)))
                .collect::<Result<Vec<_>>>()?
        ) {
            rows.push(form_row!(
                self,
                pse_ids::named_id(base, "select-at-most-one"),
                total,
                EquationSense::Le,
                form_sym!(self, one),
                lineage.clone(),
            )?);
        }
        for (i, member) in members.iter().enumerate() {
            // The selectors that allow member i to be nonzero.
            let allowed = match form {
                NativeConstraintForm::Sos2 => [i.checked_sub(1), (i < selectors).then_some(i)]
                    .into_iter()
                    .flatten()
                    .map(|k| Ok(form_sym!(self, z[k])))
                    .collect::<Result<Vec<_>>>()?,
                _ => vec![form_sym!(self, z[i])],
            };
            let Some(allowed) = form_sum!(self, allowed) else {
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
        switch: Rewritten,
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
            rows.push(form_row!(
                self,
                pse_ids::named_id(base, &format!("switched-{}-{}", upper, member.to_hex())),
                form_sym!(self, member),
                sense,
                form_bin!(self, BinaryOp::Mul, form_sym!(self, bound), switch.clone()),
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
        self.with_form_source(row.declaration_id, |engine| {
            engine.cardinality_inner(instance, row, env)
        })
    }
    fn cardinality_inner(
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
        let count = self
            .eval_field(at, env, "cardinality.count", 0, None)?
            .scalar(at)?;
        if !(count.is_finite()
            && count >= 0.0
            && count.fract() == 0.0
            && count <= f64::from(u32::MAX))
        {
            return Err(invalid(at, "a cardinality count is a nonnegative integer"));
        }
        let mut members = Vec::new();
        for coordinates in self.coordinates(
            instance,
            at,
            env,
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str())),
        )? {
            let local = coordinates_env(env, &coordinates);
            members.push(self.variable_operand(instance, "cardinality.member", &local, at)?);
        }
        let realized = self.realization(Form::Cardinality, instance, at)?;
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
        let limit = form_sym!(self, self.indicator_constant(instance, count, at)?);
        if binary {
            // Over binaries the count is one exact linear row on every backend.
            let total = form_sum!(
                self,
                members
                    .iter()
                    .map(|id| Ok(form_sym!(self, *id)))
                    .collect::<Result<Vec<_>>>()?
            )
            .ok_or_else(|| invalid(at, "cardinality without members"))?;
            let row = form_row!(
                self,
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
            selectors.push(form_sym!(self, z));
            let switch = form_sym!(self, z);
            let (rows_k, bounds) =
                self.switched_bounds(at, base, *member, switch, Policy::Linear, &lineage)?;
            rows.extend(rows_k);
            variables.extend(bounds);
        }
        let total =
            form_sum!(self, selectors).ok_or_else(|| invalid(at, "cardinality without members"))?;
        rows.push(form_row!(
            self,
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
    pub(super) fn piecewise(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        self.with_form_source(row.declaration_id, |engine| {
            engine.piecewise_inner(instance, row, env)
        })
    }
    fn piecewise_inner(
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
        let rewrite = |engine: &mut Self, role: &str, local: &Environment| -> Result<Rewritten> {
            engine.rewrite_field(instance, at, role, 0, local, &[at])
        };
        let output = rewrite(self, "piecewise.output", env)?;
        let input = rewrite(self, "piecewise.input", env)?;
        let mut points = Vec::new();
        for coordinates in self.coordinates(
            instance,
            at,
            env,
            v.indices
                .iter()
                .map(|i| (i.name.as_str(), i.domain.as_str())),
        )? {
            let local = coordinates_env(env, &coordinates);
            points.push((
                rewrite(self, "piecewise.abscissa", &local)?,
                rewrite(self, "piecewise.ordinate", &local)?,
            ));
        }
        if points.len() < 2 {
            return Err(invalid(
                at,
                "a piecewise function needs two or more breakpoints",
            ));
        }
        let realized = self.realization(Form::Piecewise, instance, at)?;
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
                rows.push(form_row!(
                    self,
                    pse_ids::named_id(w, "after"),
                    form_sym!(self, deltas[j + 1]),
                    EquationSense::Le,
                    form_sym!(self, w),
                    lineage.clone(),
                )?);
                rows.push(form_row!(
                    self,
                    pse_ids::named_id(w, "before"),
                    form_sym!(self, w),
                    EquationSense::Le,
                    form_sym!(self, deltas[j]),
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
                    terms.push(form_bin!(
                        self,
                        BinaryOp::Mul,
                        form_sym!(self, *delta),
                        form_bin!(self, BinaryOp::Sub, coordinate(j + 1), coordinate(j)),
                    ));
                }
                let total = form_sum!(self, terms)
                    .ok_or_else(|| invalid(at, "piecewise without breakpoints"))?;
                rows.push(form_row!(
                    self,
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
            let total = form_sum!(
                self,
                weights
                    .iter()
                    .map(|id| Ok(form_sym!(self, *id)))
                    .collect::<Result<Vec<_>>>()?
            )
            .ok_or_else(|| invalid(at, "piecewise without breakpoints"))?;
            rows.push(form_row!(
                self,
                pse_ids::named_id(base, "convex"),
                total,
                EquationSense::Eq,
                form_sym!(self, one),
                lineage.clone(),
            )?);
            for (name, target, pick) in [("input", input, 0usize), ("output", output, 1)] {
                let terms = weights
                    .iter()
                    .enumerate()
                    .map(|(k, w)| {
                        Ok(form_bin!(
                            self,
                            BinaryOp::Mul,
                            form_sym!(self, *w),
                            if pick == 0 {
                                points[k].0.clone()
                            } else {
                                points[k].1.clone()
                            },
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let combination = form_sum!(self, terms)
                    .ok_or_else(|| invalid(at, "piecewise without breakpoints"))?;
                rows.push(form_row!(
                    self,
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
                let chosen = form_sum!(
                    self,
                    segments
                        .iter()
                        .map(|id| Ok(form_sym!(self, *id)))
                        .collect::<Result<Vec<_>>>()?
                )
                .ok_or_else(|| invalid(at, "piecewise without segments"))?;
                rows.push(form_row!(
                    self,
                    pse_ids::named_id(base, "one-segment"),
                    chosen,
                    EquationSense::Eq,
                    form_sym!(self, one),
                    lineage.clone(),
                )?);
                for (k, weight) in weights.iter().enumerate() {
                    let ends = [k.checked_sub(1), (k < n - 1).then_some(k)]
                        .into_iter()
                        .flatten()
                        .map(|j| Ok(form_sym!(self, segments[j])))
                        .collect::<Result<Vec<_>>>()?;
                    let ends = form_sum!(self, ends)
                        .ok_or_else(|| invalid(at, "piecewise without segments"))?;
                    rows.push(form_row!(
                        self,
                        pse_ids::named_id(*weight, "adjacent"),
                        form_sym!(self, *weight),
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
        self.with_form_source(row.declaration_id, |engine| {
            engine.complementarity_inner(instance, row, coordinates, env)
        })
    }
    fn complementarity_inner(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        coordinates: &[(String, Value)],
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let _v = row
            .value
            .complementarity
            .clone()
            .ok_or_else(|| invalid(at, "complementarity payload"))?;
        let realized = self.realization(Form::Complementarity, instance, at)?;
        let local = coordinates_env(env, coordinates);
        let base = member_id(instance, at, coordinates);
        let lineage = self.lineage(instance, row, &[at]);
        let member = |engine: &mut Self, role: &str| -> Result<(Rewritten, Type)> {
            let expression = engine.rewrite_field(instance, at, role, 0, &local, &[at])?;
            let contracts = engine.model.function_contracts(engine.p);
            let ty = engine.type_of(&expression, &contracts, at)?;
            Ok((expression, ty))
        };
        let (first, first_ty) = member(self, "complementarity.first")?;
        let (second, second_ty) = member(self, "complementarity.second")?;
        let mut rows = Vec::new();
        let mut variables = Vec::new();
        let nonnegative = |engine: &mut Self,
                           rows: &mut Vec<SemanticId>,
                           name: &str,
                           value: &Rewritten,
                           ty: &Type|
         -> Result<()> {
            let zero = form_zero!(engine, ty, at);
            rows.push(form_row!(
                engine,
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
                let owner = self
                    .form_realizations
                    .get(&(instance, at))
                    .map(|(_, owner)| *owner)
                    .ok_or_else(|| {
                        invalid(at, "smoothing function lacks its actual realization owner")
                    })?;
                let width_value = self.with_original_expression(
                    owner,
                    "realization.argument",
                    0,
                    width,
                    |engine| engine.rewrite(instance, width, &local, &[owner]),
                )?;
                let width_type = self.constructed_type(width_value.root)?;
                let function = self.resolve_function(instance, owner, function, &local)?;
                let prepared = [
                    functions::PreparedArgument::Scalar {
                        value: first.clone(),
                        ty: first_ty.clone(),
                    },
                    functions::PreparedArgument::Scalar {
                        value: second.clone(),
                        ty: second_ty.clone(),
                    },
                    functions::PreparedArgument::Scalar {
                        value: width_value,
                        ty: width_type,
                    },
                ];
                let smoothed = self.function_call_prepared(
                    instance,
                    function,
                    &prepared,
                    self.form_source()?,
                )?;
                let ty = self.type_of(&smoothed, &self.model.function_contracts(self.p), at)?;
                let zero = form_zero!(self, &ty, at);
                rows.push(form_row!(
                    self,
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
                    rows.push(form_row!(
                        self,
                        pse_ids::named_id(base, &format!("complementarity-{name}")),
                        form_sym!(self, slack),
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
                let product = form_bin!(self, BinaryOp::Mul, first, second);
                let ty = self.type_of(&product, &self.model.function_contracts(self.p), at)?;
                let zero = form_zero!(self, &ty, at);
                rows.push(form_row!(
                    self,
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
        self.with_form_source(row.declaration_id, |engine| {
            engine.logic_inner(instance, row, coordinates, env)
        })
    }
    fn logic_inner(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        coordinates: &[(String, Value)],
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let proposition = self.p.proposition_at(at)?.clone();
        let realized = self.realization(Form::Logic, instance, at)?;
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
        self.with_original_logic(at, "logic.proposition", 0, &proposition, |engine| {
            lowering.assert(engine, &proposition)
        })?;
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
        lhs: Rewritten,
        sense: EquationSense,
        rhs: Rewritten,
    ) -> Result<()> {
        self.serial += 1;
        let id = pse_ids::named_id(self.base, &format!("logic-row-{}", self.serial));
        self.rows.push(form_row!(
            engine,
            id,
            lhs,
            sense,
            rhs,
            self.lineage.clone()
        )?);
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
            Proposition::Atom(expression) => (
                engine.binary_operand_ast(self.instance, expression, &self.env, self.at)?,
                false,
            ),
            Proposition::Not(inner) => {
                let (variable, negated) = self.literal(engine, inner)?;
                (variable, !negated)
            }
            other => (self.define(engine, other)?, false),
        })
    }
    fn value(
        &self,
        engine: &mut Engine<'_, '_>,
        (variable, negated): (SemanticId, bool),
    ) -> Result<Rewritten> {
        Ok(if negated {
            form_bin!(
                engine,
                BinaryOp::Sub,
                form_sym!(engine, self.one),
                form_sym!(engine, variable)
            )
        } else {
            form_sym!(engine, variable)
        })
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
        let values = operands
            .iter()
            .map(|o| self.value(engine, *o))
            .collect::<Result<Vec<_>>>()?;
        let total = form_sum!(engine, values.clone())
            .ok_or_else(|| invalid(self.at, "empty proposition"))?;
        match form {
            NativeConstraintForm::And => {
                for value in &values {
                    logic_row!(
                        self,
                        engine,
                        form_sym!(engine, z),
                        EquationSense::Le,
                        value.clone()
                    )?;
                }
                let slack = form_sym!(
                    engine,
                    engine.indicator_constant(self.instance, (values.len() - 1) as f64, self.at,)?
                );
                logic_row!(
                    self,
                    engine,
                    form_sym!(engine, z),
                    EquationSense::Ge,
                    form_bin!(engine, BinaryOp::Sub, total, slack),
                )?;
            }
            NativeConstraintForm::Or => {
                for value in &values {
                    logic_row!(
                        self,
                        engine,
                        form_sym!(engine, z),
                        EquationSense::Ge,
                        value.clone()
                    )?;
                }
                logic_row!(self, engine, form_sym!(engine, z), EquationSense::Le, total)?;
            }
            _ => {
                let (a, b) = (values[0].clone(), values[1].clone());
                logic_row!(
                    self,
                    engine,
                    form_sym!(engine, z),
                    EquationSense::Ge,
                    form_bin!(engine, BinaryOp::Sub, a.clone(), b.clone()),
                )?;
                logic_row!(
                    self,
                    engine,
                    form_sym!(engine, z),
                    EquationSense::Ge,
                    form_bin!(engine, BinaryOp::Sub, b.clone(), a.clone()),
                )?;
                logic_row!(self, engine, form_sym!(engine, z), EquationSense::Le, total)?;
                let two = form_sym!(
                    engine,
                    engine.indicator_constant(self.instance, 2.0, self.at)?
                );
                logic_row!(
                    self,
                    engine,
                    form_sym!(engine, z),
                    EquationSense::Le,
                    form_bin!(
                        engine,
                        BinaryOp::Sub,
                        form_bin!(engine, BinaryOp::Sub, two, a),
                        b
                    ),
                )?;
            }
        }
        Ok(z)
    }
    /// Assert a proposition true with the fewest auxiliary variables.
    fn assert(&mut self, engine: &mut Engine<'_, '_>, p: &Proposition) -> Result<()> {
        let one = form_sym!(engine, self.one);
        match p {
            Proposition::And(ps) => {
                for q in ps {
                    self.assert(engine, q)?;
                }
                Ok(())
            }
            Proposition::Exactly(count, ps) => {
                let k = engine
                    .eval_ast(self.at, &self.env, count)?
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
                    .map(|o| self.value(engine, o))
                    .collect::<Result<Vec<_>>>()?;
                let total = form_sum!(engine, values)
                    .ok_or_else(|| invalid(self.at, "empty proposition"))?;
                let count = form_sym!(
                    engine,
                    engine.indicator_constant(self.instance, k, self.at)?
                );
                logic_row!(self, engine, total, EquationSense::Eq, count)
            }
            _ if self.native => {
                let (variable, negated) = self.literal(engine, p)?;
                let value = self.value(engine, (variable, negated))?;
                logic_row!(self, engine, value, EquationSense::Eq, one)
            }
            Proposition::Or(ps) => {
                let values = self
                    .operands(engine, ps)?
                    .into_iter()
                    .map(|o| self.value(engine, o))
                    .collect::<Result<Vec<_>>>()?;
                let total = form_sum!(engine, values)
                    .ok_or_else(|| invalid(self.at, "empty proposition"))?;
                logic_row!(self, engine, total, EquationSense::Ge, one)
            }
            Proposition::Implies(a, b) => {
                let a = self.literal(engine, a)?;
                let b = self.literal(engine, b)?;
                let (a, b) = (self.value(engine, a)?, self.value(engine, b)?);
                logic_row!(self, engine, a, EquationSense::Le, b)
            }
            Proposition::Xor(a, b) => {
                let a = self.literal(engine, a)?;
                let b = self.literal(engine, b)?;
                let total = form_bin!(
                    engine,
                    BinaryOp::Add,
                    self.value(engine, a)?,
                    self.value(engine, b)?
                );
                logic_row!(self, engine, total, EquationSense::Eq, one)
            }
            Proposition::Atom(_) | Proposition::Not(_) => {
                let literal = self.literal(engine, p)?;
                let value = self.value(engine, literal)?;
                logic_row!(self, engine, value, EquationSense::Eq, one)
            }
        }
    }
}
