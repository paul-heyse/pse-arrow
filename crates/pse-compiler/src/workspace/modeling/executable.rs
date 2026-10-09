// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite kernel projection into the established physically checked math boundary.
use super::*;
use petgraph::{algo::toposort, graph::DiGraph};
use pse_authoring::dsl::{self, BinaryOp, EquationKind, EquationSense, Expr, ExprKind, Span};
use pse_modeling::{Type, specialize::symbol_name};
use pse_quantity::scheme::Substitution;
use std::collections::BTreeSet;
mod conformance;
mod derived;
mod factorable;
mod grouped;
mod implicit;
#[cfg(test)]
#[path = "executable/projection_tests.rs"]
mod projection_tests;
mod solve;
pub use conformance::{ModelingExpectationResult, ModelingPointChecks, ModelingValidityResult};
pub use derived::{Derivation, Derived};
pub use implicit::{
    AdmittedImplicit, ImplicitAlgorithm, ImplicitCapabilities, ImplicitMeaning, ImplicitScale,
    ImplicitSelection, SelectionEquivalence, SelectionNeighborhood,
};
pub use solve::{BoundStructure, ModelingCaseBindings, ModelingVariableState};

fn symbol_expression(id: SemanticId) -> Expr {
    Expr {
        kind: ExprKind::Path(dsl::Path::single(symbol_name(id))),
        span: Span::default(),
    }
}
fn number_expression(value: f64) -> Expr {
    Expr {
        kind: ExprKind::Number(dsl::Number {
            value,
            exact_integer: None,
            unit: None,
        }),
        span: Span::default(),
    }
}

/// Numerical observation purpose. These do not add equations or fix variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelingHint {
    /// Physical start value for initialization.
    Start,
    /// Characteristic magnitude for the numerical policy.
    Nominal,
    /// Lower bound expression.
    Lower,
    /// Upper bound expression.
    Upper,
    /// Lower endpoint of the declared validity range.
    ValidLower,
    /// Upper endpoint of the declared validity range.
    ValidUpper,
    /// Post-solve check indicator: one when the predicate holds, zero otherwise.
    Check,
    /// The decision-free side of an objective-bound check with the same target and
    /// declaration: the value the check compares the objective with (ADR-0119 Outcome 5).
    ObjectiveBound(ObjectiveBound),
    /// Sample time of an authored engineering goal.
    AccuracyGoalTime,
    /// Requested output-error resolution of an authored engineering goal.
    AccuracyGoalResolution,
    /// Inclusive lower endpoint of an authored engineering criterion.
    AccuracyGoalLower,
    /// Inclusive upper endpoint of an authored engineering criterion.
    AccuracyGoalUpper,
    /// Authored engineering characteristic magnitude.
    EngineeringScaleValue,
}
impl ModelingHint {
    /// Stable identity code for structural hashing; the unit variants keep their
    /// declaration-order codes.
    pub(crate) const fn code(self) -> u64 {
        match self {
            Self::Start => 0,
            Self::Nominal => 1,
            Self::Lower => 2,
            Self::Upper => 3,
            Self::ValidLower => 4,
            Self::ValidUpper => 5,
            Self::Check => 6,
            Self::ObjectiveBound(ObjectiveBound::Lower { strict }) => 7 + strict as u64,
            Self::ObjectiveBound(ObjectiveBound::Upper { strict }) => 9 + strict as u64,
            Self::AccuracyGoalTime => 11,
            Self::AccuracyGoalResolution => 12,
            Self::AccuracyGoalLower => 13,
            Self::AccuracyGoalUpper => 14,
            Self::EngineeringScaleValue => 15,
        }
    }
}
/// A check that bounds the objective from its optimized side (ADR-0119 Outcome 5): from
/// below when minimizing, from above when maximizing. A certified dual bound of the step
/// then establishes it over the declared box; without one it is evaluated at the point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectiveBound {
    /// `objective > c` (strict) or `objective >= c` on a minimized objective.
    Lower {
        /// The authored comparison is strict.
        strict: bool,
    },
    /// `objective < c` (strict) or `objective <= c` on a maximized objective.
    Upper {
        /// The authored comparison is strict.
        strict: bool,
    },
}
impl ObjectiveBound {
    /// Whether `objective` satisfies the check against the compared value `bound`. Given a
    /// certified dual bound as `objective`, a true result holds for every feasible point.
    pub fn holds(self, objective: f64, bound: f64) -> bool {
        match self {
            Self::Lower { strict: true } => objective > bound,
            Self::Lower { strict: false } => objective >= bound,
            Self::Upper { strict: true } => objective < bound,
            Self::Upper { strict: false } => objective <= bound,
        }
    }
    /// Classify an authored check predicate against the selected objective: a comparison
    /// of the objective member itself with a decision-free side, oriented toward the
    /// optimized side. Returns the classification and the decision-free side; anything
    /// else (another shape, the other side, an equality, a side that depends on a free
    /// decision) is a point check.
    pub(crate) fn classify(
        model: &SpecializedModel,
        objective: Option<(SemanticId, pse_math::binding::ObjectiveSense)>,
        predicate: &dsl::Predicate,
    ) -> Option<(Self, Expr)> {
        use dsl::{CompareOp as Op, PredicateKind};
        use pse_math::binding::ObjectiveSense as Sense;
        let (target, sense) = objective?;
        let PredicateKind::Compare { op, lhs, rhs } = &predicate.kind else {
            return None;
        };
        let name = symbol_name(target);
        let is_objective = |e: &Expr| {
            matches!(&e.kind, ExprKind::Path(path)
                if path.segments.len() == 1
                    && path.segments[0].indices.is_empty()
                    && path.segments[0].name == name)
        };
        // Orient the comparison as `objective op side`.
        let (op, side) = match (is_objective(lhs), is_objective(rhs)) {
            (true, false) => (*op, rhs),
            (false, true) => (
                match op {
                    Op::Lt => Op::Gt,
                    Op::Le => Op::Ge,
                    Op::Gt => Op::Lt,
                    Op::Ge => Op::Le,
                    other => *other,
                },
                lhs,
            ),
            _ => return None,
        };
        let bound = match (sense, op) {
            (Sense::Minimize, Op::Gt) => Self::Lower { strict: true },
            (Sense::Minimize, Op::Ge) => Self::Lower { strict: false },
            (Sense::Maximize, Op::Lt) => Self::Upper { strict: true },
            (Sense::Maximize, Op::Le) => Self::Upper { strict: false },
            _ => return None,
        };
        decision_free(model, side, &mut BTreeSet::new()).then(|| (bound, (**side).clone()))
    }
}
/// Whether `expression` depends on no variable of the model, following computed members
/// to their definitions. A path that names no model symbol is not decision-free.
fn decision_free(
    model: &SpecializedModel,
    expression: &Expr,
    visited: &mut BTreeSet<SemanticId>,
) -> bool {
    use pse_model::generated::enums::ModelingDeclarationKind as Kind;
    expression.paths().iter().all(|path| {
        let Some(symbol) = (path.segments.len() == 1)
            .then(|| path.segments.first())
            .flatten()
            .and_then(|segment| segment.name.strip_prefix("s_"))
            .and_then(|hex| SemanticId::parse_hex(hex).ok())
            .and_then(|id| model.symbols.get(&id))
        else {
            return false;
        };
        if !visited.insert(symbol.id) {
            return true;
        }
        match (&symbol.expression, symbol.role) {
            (Some(definition), _) => decision_free(model, definition, visited),
            (None, Kind::Parameter) => true,
            (None, _) => false,
        }
    })
}
/// Source-test observation role; actual and expected retain their physical representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelingTestValue {
    /// The observed quantity under test.
    Actual,
    /// The reference value the actual quantity is compared with.
    Expected,
    /// Absolute tolerance in the actual quantity's representation.
    Tolerance,
    /// Dimensionless tolerance relative to the expected value.
    RelativeTolerance,
}
/// Meaning of an output, independent of execution slots and source names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelingOutput {
    /// Compiler-owned supplied-boundary equality residual, distinct from its point observation.
    ConditionalBoundary(SemanticId),
    /// Original conserved inventory, independent of the generated accumulation coordinate.
    Inventory(SemanticId),
    /// Original pre-event transfer for a conserved subject and guard occurrence.
    InventoryTransfer {
        /// Stable conserved subject identity.
        balance: SemanticId,
        /// Resolved scalar guard member occurrence.
        event: SemanticId,
    },
    /// Rate expression derived from an original time-derivative equation.
    DynamicRate {
        /// Differential state whose rate this output evaluates.
        state: SemanticId,
        /// Original equations the rate expression was derived from.
        equations: Vec<SemanticId>,
    },
    /// Differential initial value derived from an explicit endpoint constraint.
    InitialState {
        /// Differential state whose initial value this output evaluates.
        state: SemanticId,
        /// Endpoint constraint the initial value was derived from.
        equation: SemanticId,
    },
    /// One component of an authored source test.
    Test {
        /// Source test identity.
        id: SemanticId,
        /// Which side of the comparison this output evaluates.
        component: ModelingTestValue,
    },
    /// Compiled annotation expression with its source and target preserved.
    Hint {
        /// Model member the annotation applies to.
        target: SemanticId,
        /// Declaration that authored the annotation.
        declaration: DeclarationId,
        /// Numerical purpose of the annotation expression.
        kind: ModelingHint,
    },
    /// Unrelaxed residual, retained independently of auxiliary variables.
    OriginalEquation(SemanticId),
    /// Dimensionless elastic objective contribution, also available for inspection.
    Penalty(SemanticId),
    /// Original additive term, before algebraic normalization; its sign is separate
    /// because negating an affine physical quantity is not a valid physical operation.
    Term {
        /// Equation the term belongs to.
        equation: SemanticId,
        /// Position of the term in the equation's original additive order.
        ordinal: usize,
        /// Whether the term enters the residual with a negative sign.
        negative: bool,
    },
    /// One equation residual in original orientation.
    Equation {
        /// Equation identity.
        id: SemanticId,
        /// Authored relation between the two sides.
        sense: EquationSense,
    },
    /// A generated objective bound of an earlier lexicographic level (ADR-0111,
    /// `objective_bounds`). The output is the bound parameter β and enters the row with
    /// factor −1; each member of the bounded level enters it with its scale, so the row
    /// holds `level value − β <= 0` for a minimized level and `>= 0` for a maximized one.
    LevelBound {
        /// Generated row identity.
        row: SemanticId,
        /// Generated bound parameter β.
        parameter: SemanticId,
        /// `Le` for a minimized level, `Ge` for a maximized one.
        sense: EquationSense,
    },
    /// One demanded expression or accounting member.
    Member(SemanticId),
    /// Original contribution magnitude for independent closure checking.
    Contribution {
        /// Accounting member that receives the contribution.
        accumulator: SemanticId,
        /// Contributing member.
        contribution: SemanticId,
    },
}
/// Finite typed mathematics with semantic input/output coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedModeling {
    /// CompilerContext in the math body's exact order. Values are supplied separately.
    pub inputs: Vec<SemanticId>,
    /// Outputs in the math body's exact order.
    pub outputs: Vec<ModelingOutput>,
    /// Existing typed math artifacts, shared by normalized consumer shape.
    pub bodies: BTreeMap<ContentHash, Arc<AdmittedBody>>,
    /// Semantic gathers and output contributions into the established case assembly.
    pub(super) case: Arc<CaseStructure>,
    /// Nested residual definitions, independent of mutable native workers.
    pub(super) implicit: BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
    /// Auxiliary original term rows and signs, outside public observable ordering.
    pub term_outputs: BTreeMap<SemanticId, Vec<(SemanticId, f64)>>,
}
#[derive(Clone, Debug, PartialEq)]
struct Projection {
    /// Authored objective members grouped into levels (ADR-0111): priority, weight,
    /// normalization, and each level's tolerances and generated bound.
    objectives: pse_modeling::specialize::Objectives,
    body_limits: BodyLimits,
    /// Original additive conservation terms assembled into their semantic row.
    conservation: BTreeMap<SemanticId, Vec<(SemanticId, f64)>>,
    nonnegative: BTreeSet<SemanticId>,
    validity: BTreeMap<String, crate::typed_math::Validity>,
    inputs: Vec<SemanticId>,
    /// Free variables with their declared domain (ADR-0103); fixed inputs are absent.
    free: BTreeMap<SemanticId, pse_model::generated::enums::ModelingVariableDomain>,
    /// Kernel-derived continuous variables confined to [0, 1] (ADR-0104).
    unit_interval: BTreeSet<SemanticId>,
    /// Constraint forms left to native handlers (ADR-0104).
    native: Vec<pse_model::forms::NativeConstraint>,
    /// Requirements the lowerings place on the solve route (ADR-0104 §5).
    requirements: BTreeSet<pse_model::generated::enums::ModelingStructuralRequirement>,
    formals: Vec<Formal>,
    outputs: Vec<ModelingOutput>,
    expressions: Vec<Expr>,
    functions: BTreeMap<String, pse_modeling::Function>,
    quantities: Vec<QuantityTypeId>,
    local_quantities: BTreeMap<String, QuantityTypeId>,
    declarations: Vec<DeclarationId>,
    implicit: Vec<implicit::Projection>,
    original: Option<Arc<OriginalProjection>>,
}
#[derive(Clone, Debug, PartialEq)]
struct OriginalProjection {
    projection: Projection,
    bindings: Vec<(String, Expr)>,
}
impl Projection {
    /// The objective member whose value the case structure optimizes, with its
    /// orientation: the solved level when it is one member entering with scale one. A
    /// weighted or normalized level has no member equal to its value.
    fn objective(&self) -> Option<(SemanticId, pse_math::binding::ObjectiveSense)> {
        let level = self.objectives.solved()?;
        let [member] = self.objectives.members_of(level).collect::<Vec<_>>()[..] else {
            return None;
        };
        (member.scale == 1.0 && member.term == member.target)
            .then_some((member.target, objective_sense(level.sense)))
    }
}
/// The case structure's orientation of an authored objective sense.
fn objective_sense(
    sense: pse_modeling::annotation::ObjectiveSense,
) -> pse_math::binding::ObjectiveSense {
    match sense {
        pse_modeling::annotation::ObjectiveSense::Minimize => {
            pse_math::binding::ObjectiveSense::Minimize
        }
        pse_modeling::annotation::ObjectiveSense::Maximize => {
            pse_math::binding::ObjectiveSense::Maximize
        }
    }
}
#[salsa::tracked(returns(clone),lru=64,heap_size=projection_heap)]
fn projection(
    db: &dyn CompilerDb,
    inventory: Inventory,
    catalog: Catalog,
    request: Request,
) -> Result<Arc<Projection>> {
    let model = specialized(db, catalog, request)?;
    let registry = inventory.quantities(db);
    let (bound_expressions, mut bound_equations) = model.bound_bodies()?;
    let rows = model
        .equations
        .iter()
        .chain(
            model
                .regimes
                .values()
                .flat_map(|r| r.alternatives.iter().flat_map(|a| &a.equations)),
        )
        .collect::<Vec<_>>();
    for alternative in model.regimes.values().flat_map(|r| &r.alternatives) {
        for row in &alternative.equations {
            bound_equations.insert(row.id, row.equation.clone());
        }
    }
    let annotations = model
        .annotations
        .iter()
        .chain(
            model
                .regimes
                .values()
                .flat_map(|r| r.alternatives.iter().flat_map(|a| &a.annotations)),
        )
        .collect::<Vec<_>>();
    let mut validity = BTreeMap::new();
    let mut validity_targets = BTreeSet::new();
    for a in &model.annotations {
        if let pse_modeling::annotation::AnnotationValue::Valid {
            lower,
            upper,
            layer,
            ..
        } = &a.value
        {
            // An annotated hard range is unconditional; one closure range owns each member.
            if *layer == pse_model::generated::enums::ModelingValidityLayer::Closure
                && !validity_targets.insert(a.target)
            {
                return Err(CompileError::Missing(
                    "competing validity intervals for a model member".into(),
                ));
            }
            {
                validity.insert(
                    symbol_name(a.target),
                    crate::typed_math::Validity {
                        lower: lower.clone(),
                        upper: upper.clone(),
                        source: a.lineage.declaration.into(),
                        target: a.target,
                    },
                );
            }
        }
    }
    let mut p = Projection {
        objectives: model.objectives.clone(),
        body_limits: BodyLimits {
            occurrences: request
                .limits(db)
                .body_occurrences
                .unwrap_or_else(|| BodyLimits::default().occurrences),
            slots: request
                .limits(db)
                .body_slots
                .unwrap_or_else(|| BodyLimits::default().slots),
        },
        conservation: model
            .closures
            .values()
            .filter_map(|closure| {
                let row = pse_ids::named_id(closure.id, "conservation");
                (rows.iter().any(|r| r.id == row) && !model.elastic.contains_key(&row)).then(|| {
                    (
                        row,
                        closure
                            .terms
                            .iter()
                            .map(|term| (term.id, term.sign()))
                            .collect(),
                    )
                })
            })
            .collect(),
        validity,
        nonnegative: model
            .elastic
            .values()
            .flat_map(|r| r.slacks.iter().copied())
            .chain(model.nonnegative.iter().copied())
            .collect(),
        inputs: vec![],
        free: BTreeMap::new(),
        unit_interval: model.unit_interval.clone(),
        native: model.native.clone(),
        requirements: model.requirements.clone(),
        formals: vec![],
        outputs: vec![],
        expressions: vec![],
        functions: model.functions.clone(),
        quantities: Vec::new(),
        local_quantities: BTreeMap::new(),
        declarations: Vec::new(),
        implicit: Vec::new(),
        original: None,
    };
    let mut graph = DiGraph::<SemanticId, ()>::new();
    let nodes = model
        .symbols
        .iter()
        .filter(|(_, s)| s.expression.is_some())
        .map(|(id, _)| (*id, graph.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    for (id, symbol) in &model.symbols {
        if let Some(expression) = bound_expressions.get(id) {
            let Some(scheme) = symbol.ty.quantity_scheme() else {
                return Err(CompileError::Missing(
                    "nonphysical expression member".into(),
                ));
            };
            p.local_quantities.insert(
                symbol_name(*id),
                scheme
                    .resolve_with_evidence(
                        registry,
                        &Substitution::new(),
                        inventory.preconditions(db).as_ref(),
                    )
                    .map_err(|e| CompileError::Missing(e.to_string()))?,
            );
            for path in expression.paths() {
                if path.segments.len() == 1
                    && let Some(name) = path.segments[0].name.strip_prefix("s_")
                    && let Ok(dependency) = SemanticId::parse_hex(name)
                    && let Some(node) = nodes.get(&dependency)
                {
                    graph.add_edge(*node, nodes[id], ());
                }
            }
        } else {
            let Some(scheme) = symbol.ty.quantity_scheme() else {
                return Err(CompileError::Missing("nonphysical runtime symbol".into()));
            };
            let quantity = scheme
                .resolve_with_evidence(
                    registry,
                    &Substitution::new(),
                    inventory.preconditions(db).as_ref(),
                )
                .map_err(|e| CompileError::Missing(e.to_string()))?;
            if symbol.role == pse_model::generated::enums::ModelingDeclarationKind::Variable {
                // PS-01: an integer or binary decision is a dimensionless count or indicator.
                if symbol.domain.requires_pure_number()
                    && registry
                        .discrete_category(quantity)
                        .map_err(|e| CompileError::Missing(e.to_string()))?
                        .is_none()
                {
                    return Err(model
                        .domain_refusal(
                            *id,
                            pse_modeling::DomainAnalysis::Preparation,
                            pse_modeling::DomainRefusal::QuantityKind,
                        )
                        .into());
                }
                p.free.insert(*id, symbol.domain);
            }
            p.inputs.push(*id);
            p.formals.push(Formal {
                path: symbol_name(*id),
                quantity,
            });
        }
    }
    let order = toposort(&graph, None)
        .map_err(|_| CompileError::Missing("cyclic expression members".into()))?;
    let mut bindings = order
        .iter()
        .map(|node| {
            let id = graph[*node];
            let expression = bound_expressions
                .get(&id)
                .cloned()
                .ok_or_else(|| CompileError::Missing("expression member".into()))?;
            Ok((symbol_name(id), expression))
        })
        .collect::<Result<Vec<_>>>()?;
    for row in &rows {
        let equation = bound_equations
            .get(&row.id)
            .ok_or_else(|| CompileError::Missing("bound dispatch equation".into()))?;
        let EquationKind::Relation { lhs, rhs, sense } = &equation.kind else {
            return Err(CompileError::Missing("unspecialized equation guard".into()));
        };
        p.outputs.push(ModelingOutput::Equation {
            id: row.id,
            sense: *sense,
        });
        p.expressions.push(Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Sub,
                lhs: Box::new(lhs.clone()),
                rhs: Box::new(rhs.clone()),
            },
            span: Span::default(),
        });
    }
    // The derivative system is solved as one guarded affine residual. Every rate
    // retains the whole source equation set; equation ordering never pairs states by accident.
    let rate_names = model
        .derivatives
        .values()
        .map(|d| symbol_name(d.rate))
        .collect::<BTreeSet<_>>();
    let dynamic_rows = rows
        .iter()
        .filter_map(|row| {
            let EquationKind::Relation { lhs, rhs, sense } = &bound_equations[&row.id].kind else {
                return None;
            };
            let mut needed = lhs
                .free_paths()
                .into_iter()
                .chain(rhs.free_paths())
                .filter(|&p| p.segments.len() == 1)
                .map(|p| p.segments[0].name.clone())
                .collect::<BTreeSet<_>>();
            loop {
                let before = needed.len();
                for (name, expression) in &bindings {
                    if needed.contains(name) {
                        needed.extend(
                            expression
                                .free_paths()
                                .into_iter()
                                .filter(|&p| p.segments.len() == 1)
                                .map(|p| p.segments[0].name.clone()),
                        );
                    }
                }
                if needed.len() == before {
                    break;
                }
            }
            needed
                .iter()
                .any(|p| rate_names.contains(p))
                .then_some((row.id, *sense))
        })
        .collect::<Vec<_>>();
    if dynamic_rows.len() != model.derivatives.len()
        || dynamic_rows
            .iter()
            .any(|(_, sense)| *sense != EquationSense::Eq)
    {
        return Err(CompileError::Missing(
            "integrated derivatives require a square equality system".into(),
        ));
    }
    for derivative in model.derivatives.values() {
        p.outputs.push(ModelingOutput::DynamicRate {
            state: derivative.state,
            equations: dynamic_rows.iter().map(|(id, _)| *id).collect(),
        });
        p.expressions.push(symbol_expression(derivative.rate));
        let state_name = symbol_name(derivative.state);
        let mut initial = None;
        for id in &model.initial_equations {
            let EquationKind::Relation {
                lhs,
                rhs,
                sense: EquationSense::Eq,
            } = &bound_equations[id].kind
            else {
                continue;
            };
            let direct =
                |e: &Expr| matches!(&e.kind,ExprKind::Path(p) if dsl::render_path(p)==state_name);
            let expression = if direct(lhs) {
                Some(rhs)
            } else if direct(rhs) {
                Some(lhs)
            } else {
                None
            };
            if let Some(expression) = expression
                && initial.replace((*id, expression.clone())).is_some()
            {
                return Err(CompileError::Missing(
                    "multiple initial conditions for one state".into(),
                ));
            }
        }
        if let Some((equation, expression)) = initial {
            p.outputs.push(ModelingOutput::InitialState {
                state: derivative.state,
                equation,
            });
            p.expressions.push(expression);
        }
    }
    // Numerical observation is a demand, not an automatic consequence of retaining
    // a source declaration. Original rows, checks and objective terms remain mandatory.
    let instance_path = &model.instances[request.instance(db)].path;
    let mut observed = request
        .bindings(db)
        .demand
        .iter()
        .filter_map(|path| {
            model.paths.get(path).copied().or_else(|| {
                // Specialization may inline a computed member to a finite call rather
                // than a symbol reference. Its exact authored instance path still
                // identifies the explicitly demanded member; suffix matching would
                // accidentally admit unrelated child consumers.
                let full_path = format!("{instance_path}.{path}");
                model
                    .symbols
                    .values()
                    .find_map(|symbol| (symbol.lineage.path == full_path).then_some(symbol.id))
            })
        })
        .collect::<BTreeSet<_>>();
    // The enclosure of each derived Big-M is an actual preparation consumer, even
    // when the caller has not requested its source member as a report observation.
    observed.extend(
        model
            .derived
            .values()
            .filter_map(|parameter| match parameter.rule {
                pse_modeling::specialize::DerivedRule::Extremum { expression, .. } => {
                    Some(expression)
                }
                pse_modeling::specialize::DerivedRule::Bound { .. } => None,
            }),
    );
    observed.extend(
        model
            .objectives
            .members
            .iter()
            .flat_map(|member| [member.target, member.term]),
    );
    observed.extend(model.annotations.iter().filter_map(|annotation| {
        model
            .symbols
            .contains_key(&annotation.target)
            .then_some(annotation.target)
    }));
    // Conditional boundary factories consume physical port coordinates explicitly.
    observed.extend(model.ports.values().map(|port| port.symbol));
    // Declared hybrid events consume their zero-crossing guards and physical reset
    // values. Retaining only the declaration leaves no callable trial function.
    for event in model
        .fixtures
        .values()
        .flat_map(|fixture| &fixture.modes)
        .flat_map(|mode| &mode.events)
    {
        observed.insert(event.guard);
        observed.extend(
            event
                .reset
                .iter()
                .flat_map(|(target, value)| [*target, *value]),
        );
    }
    if !model.integrated.is_empty() {
        // Integration consumes original trial coordinates and quadrature integrands
        // whether or not a report/start annotation observes them. Generated coordinates
        // have semantic identities and need no caller-authored path spelling.
        observed.extend(model.symbols.values().filter_map(|symbol| {
            (symbol.role == pse_model::generated::enums::ModelingDeclarationKind::Variable
                && symbol.expression.is_none())
            .then_some(symbol.id)
        }));
        observed.extend(model.integrals.values().map(|integral| integral.integrand));
    }
    for id in p
        .inputs
        .iter()
        .filter(|id| observed.contains(id) && !model.derivatives.values().any(|d| d.rate == **id))
    {
        p.outputs.push(ModelingOutput::Member(*id));
        p.expressions.push(symbol_expression(*id));
    }
    for node in order {
        let id = graph[node];
        if !observed.contains(&id) {
            continue;
        }
        p.outputs.push(ModelingOutput::Member(id));
        p.expressions.push(symbol_expression(id));
    }
    // `objective_bounds` (ADR-0111): each bounded level's row, assembled from β and the
    // level's member outputs.
    for level in &model.objectives.levels {
        if let Some(bound) = &level.bound {
            p.outputs.push(ModelingOutput::LevelBound {
                row: bound.row,
                parameter: bound.parameter,
                sense: match level.sense {
                    pse_modeling::annotation::ObjectiveSense::Minimize => EquationSense::Le,
                    pse_modeling::annotation::ObjectiveSense::Maximize => EquationSense::Ge,
                },
            });
            p.expressions.push(symbol_expression(bound.parameter));
        }
    }
    for balance in model.inventory_balances.values() {
        p.outputs.push(ModelingOutput::Inventory(balance.id));
        p.expressions.push(balance.inventory.clone());
        for (event, expression) in &balance.transfers {
            p.outputs.push(ModelingOutput::InventoryTransfer {
                balance: balance.id,
                event: *event,
            });
            p.expressions.push(expression.clone());
        }
    }
    for closure in model.closures.values() {
        for term in &closure.terms {
            p.outputs.push(ModelingOutput::Contribution {
                accumulator: closure.id,
                contribution: term.id,
            });
            p.expressions.push(term.expression.clone());
        }
    }
    for (id, elastic) in &model.elastic {
        let EquationKind::Relation { lhs, rhs, .. } = &elastic.original.equation.kind else {
            return Err(CompileError::Missing("original elastic relation".into()));
        };
        p.outputs.push(ModelingOutput::OriginalEquation(*id));
        p.expressions.push(Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Sub,
                lhs: Box::new(lhs.clone()),
                rhs: Box::new(rhs.clone()),
            },
            span: Span::default(),
        });
        p.outputs.push(ModelingOutput::Penalty(*id));
        p.expressions.push(elastic.penalty.clone());
    }
    use pse_modeling::annotation::AnnotationValue as A;
    let objective = p.objective();
    for annotation in &annotations {
        let mut push = |kind, expression: Expr| {
            p.outputs.push(ModelingOutput::Hint {
                target: annotation.target,
                declaration: annotation.lineage.declaration,
                kind,
            });
            p.expressions.push(expression);
        };
        match &annotation.value {
            A::Start(e) => push(ModelingHint::Start, e.clone()),
            A::Nominal(e) => push(ModelingHint::Nominal, e.clone()),
            A::Bounds(l, u) => {
                push(ModelingHint::Lower, l.clone());
                push(ModelingHint::Upper, u.clone());
            }
            A::Valid { lower, upper, .. } => {
                push(ModelingHint::ValidLower, lower.clone());
                push(ModelingHint::ValidUpper, upper.clone());
            }
            A::Check(predicate) => {
                // The typed classification travels as the compared side's own output.
                if let Some((bound, side)) = ObjectiveBound::classify(&model, objective, predicate)
                {
                    push(ModelingHint::ObjectiveBound(bound), side);
                }
                push(
                    ModelingHint::Check,
                    Expr {
                        kind: ExprKind::Conditional {
                            guard: Box::new(predicate.clone()),
                            then: Box::new(number_expression(1.0)),
                            otherwise: Box::new(number_expression(0.0)),
                        },
                        span: Span::default(),
                    },
                );
            }
            A::AccuracyGoal(goal) => {
                if let Some(time) = &goal.time {
                    push(ModelingHint::AccuracyGoalTime, time.clone());
                }
                if let Some(resolution) = &goal.resolution {
                    push(ModelingHint::AccuracyGoalResolution, resolution.clone());
                }
                if let Some(lower) = &goal.criterion_lower {
                    push(ModelingHint::AccuracyGoalLower, lower.clone());
                }
                if let Some(upper) = &goal.criterion_upper {
                    push(ModelingHint::AccuracyGoalUpper, upper.clone());
                }
            }
            A::EngineeringScale(scale) => {
                push(ModelingHint::EngineeringScaleValue, scale.value.clone());
            }
            A::EngineeringDefault { .. } => {}
            A::Scale(_) | A::Report(_) | A::Objective(_) => {}
        }
    }
    for a in &model.annotations {
        if matches!(
            a.value,
            pse_modeling::annotation::AnnotationValue::Report(_)
                | pse_modeling::annotation::AnnotationValue::Valid { .. }
        ) && model.symbols.contains_key(&a.target)
            && !p
                .outputs
                .iter()
                .any(|o| matches!(o,ModelingOutput::Member(id) if *id==a.target))
        {
            p.outputs.push(ModelingOutput::Member(a.target));
            p.expressions.push(symbol_expression(a.target));
        }
    }
    let equations = p
        .outputs
        .iter()
        .zip(&p.expressions)
        .filter_map(|(o, e)| match o {
            ModelingOutput::Equation { id, .. } if !model.elastic.contains_key(id) => {
                Some((*id, e.clone()))
            }
            ModelingOutput::OriginalEquation(id) => Some((*id, e.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    fn additive(e: &Expr, negative: bool, terms: &mut Vec<(bool, Expr)>) {
        match &e.kind {
            ExprKind::Binary {
                op: BinaryOp::Add,
                lhs,
                rhs,
            } => {
                additive(lhs, negative, terms);
                additive(rhs, negative, terms);
            }
            ExprKind::Binary {
                op: BinaryOp::Sub,
                lhs,
                rhs,
            } => {
                additive(lhs, negative, terms);
                additive(rhs, !negative, terms);
            }
            ExprKind::Neg(value) => additive(value, !negative, terms),
            _ => terms.push((negative, e.clone())),
        }
    }
    for (equation, expression) in equations {
        let mut terms = vec![];
        additive(&expression, false, &mut terms);
        for (ordinal, (negative, expression)) in terms.into_iter().enumerate() {
            p.outputs.push(ModelingOutput::Term {
                equation,
                ordinal,
                negative,
            });
            p.expressions.push(expression);
        }
    }
    for test in model.expectations.values() {
        for (component, expression) in [
            (ModelingTestValue::Actual, &test.actual),
            (ModelingTestValue::Expected, &test.expected),
            (ModelingTestValue::Tolerance, &test.tolerance),
            (
                ModelingTestValue::RelativeTolerance,
                &test.relative_tolerance,
            ),
        ] {
            p.outputs.push(ModelingOutput::Test {
                id: test.id,
                component,
            });
            p.expressions.push(expression.clone());
        }
    }
    for output in &p.outputs {
        let declaration = match output {
            ModelingOutput::Inventory(id)
            | ModelingOutput::InventoryTransfer { balance: id, .. } => {
                Some(model.inventory_balances[id].lineage.declaration)
            }
            ModelingOutput::Test { id, .. } => Some(model.expectations[id].lineage.declaration),
            ModelingOutput::Hint { declaration, .. } => Some(*declaration),
            ModelingOutput::DynamicRate { state, .. } => {
                Some(model.symbols[state].lineage.declaration)
            }
            ModelingOutput::InitialState { equation: id, .. }
            | ModelingOutput::Equation { id, .. }
            | ModelingOutput::Term { equation: id, .. }
            | ModelingOutput::OriginalEquation(id)
            | ModelingOutput::Penalty(id) => rows
                .iter()
                .find(|r| r.id == *id)
                .map(|r| r.lineage.declaration),
            ModelingOutput::ConditionalBoundary(id)
            | ModelingOutput::Member(id)
            | ModelingOutput::LevelBound { parameter: id, .. } => {
                model.symbols.get(id).map(|s| s.lineage.declaration)
            }
            ModelingOutput::Contribution {
                accumulator,
                contribution,
            } => model
                .closures
                .get(accumulator)
                .and_then(|c| c.terms.iter().find(|t| t.id == *contribution))
                .map(|c| c.lineage.declaration),
        }
        .ok_or_else(|| CompileError::Missing("output lineage".into()))?;
        p.declarations.push(declaration);
    }
    let package = model.function_contracts(catalog.checked(db));
    let context = package.context();
    let types = model
        .symbols
        .iter()
        .map(|(id, s)| (symbol_name(*id), s.ty.clone()))
        .collect();
    for (expression, output) in p.expressions.iter().zip(&p.outputs) {
        let term_type = if let ModelingOutput::Term { equation, .. } = output {
            let index = p
                .outputs
                .iter()
                .position(|o| matches!(o,ModelingOutput::Equation{id,..} if id==equation))
                .ok_or_else(|| CompileError::Missing("term parent equation".into()))?;
            Some(Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                p.quantities[index],
            )))
        } else {
            None
        };
        let hint_type = if let ModelingOutput::Hint { target, kind, .. } = output {
            if let ModelingHint::ObjectiveBound(_) = kind {
                // The compared side has the objective's physical type.
                let objective = p
                    .objective()
                    .and_then(|(objective, _)| model.symbols.get(&objective))
                    .ok_or_else(|| CompileError::Missing("objective-bound check".into()))?;
                Some(objective.ty.clone())
            } else if *kind == ModelingHint::Check {
                Some(Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                    registry
                        .neutral_dimensionless()
                        .ok_or_else(|| CompileError::Missing("dimensionless check".into()))?,
                )))
            } else if *kind == ModelingHint::AccuracyGoalTime {
                // The selected observation owner validates the physical time axis. Keep
                // the expression typed but do not compare it to the observed output type.
                None
            } else if matches!(
                *kind,
                ModelingHint::AccuracyGoalResolution | ModelingHint::EngineeringScaleValue
            ) {
                let ty = model
                    .symbols
                    .get(target)
                    .map(|symbol| &symbol.ty)
                    .or_else(|| model.closures.get(target).map(|closure| &closure.ty))
                    .or_else(|| {
                        model
                            .inventory_balances
                            .get(target)
                            .map(|balance| &balance.ty)
                    })
                    .ok_or_else(|| {
                        CompileError::Missing("goal/scale target has no physical owner".into())
                    })?;
                let scheme = ty.quantity_scheme().ok_or_else(|| {
                    CompileError::Missing("goal/scale target has no physical type".into())
                })?;
                Some(Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                    pse_quantity::scheme::Scheme::Delta(Box::new(scheme.clone()))
                        .resolve_with_evidence(
                            registry,
                            &Substitution::new(),
                            inventory.preconditions(db).as_ref(),
                        )
                        .map_err(|e| CompileError::Missing(e.to_string()))?,
                )))
            } else if let Some(symbol) = model.symbols.get(target) {
                Some(symbol.ty.clone())
            } else {
                p.outputs
                    .iter()
                    .position(|o| matches!(o,ModelingOutput::Equation{id,..} if id==target))
                    .map(|i| {
                        Type::Quantity(pse_quantity::scheme::Scheme::Concrete(p.quantities[i]))
                    })
            }
        } else {
            None
        };
        let test_type = if let ModelingOutput::Test { id, component } = output {
            let ty = &model.expectations[id].ty;
            Some(if *component == ModelingTestValue::RelativeTolerance {
                Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                    registry.neutral_dimensionless().ok_or_else(|| {
                        CompileError::Missing("relative tolerance scalar type".into())
                    })?,
                ))
            } else if *component == ModelingTestValue::Tolerance {
                let Some(q) = ty.quantity_scheme() else {
                    return Err(CompileError::Missing("test physical type".into()));
                };
                Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                    pse_quantity::scheme::Scheme::Delta(Box::new(q.clone()))
                        .resolve_with_evidence(
                            registry,
                            &Substitution::new(),
                            inventory.preconditions(db).as_ref(),
                        )
                        .map_err(|e| CompileError::Missing(e.to_string()))?,
                ))
            } else {
                ty.clone()
            })
        } else {
            None
        };
        let transfer_type = if let ModelingOutput::InventoryTransfer { balance, .. } = output {
            let scheme = model.inventory_balances[balance]
                .ty
                .quantity_scheme()
                .ok_or_else(|| CompileError::Missing("inventory transfer physical type".into()))?;
            Some(Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                pse_quantity::scheme::Scheme::Delta(Box::new(scheme.clone()))
                    .resolve_with_evidence(
                        registry,
                        &Substitution::new(),
                        inventory.preconditions(db).as_ref(),
                    )
                    .map_err(|e| CompileError::Missing(e.to_string()))?,
            )))
        } else {
            None
        };
        let expected = match output {
            ModelingOutput::Inventory(id) => Some(&model.inventory_balances[id].ty),
            ModelingOutput::InventoryTransfer { .. } => transfer_type.as_ref(),
            ModelingOutput::InitialState { state, .. } => Some(&model.symbols[state].ty),
            ModelingOutput::DynamicRate { state, .. } => {
                Some(&model.symbols[&model.derivatives[state].rate].ty)
            }
            ModelingOutput::Test { .. } => test_type.as_ref(),
            ModelingOutput::Hint { .. } => hint_type.as_ref(),
            ModelingOutput::ConditionalBoundary(_)
            | ModelingOutput::OriginalEquation(_)
            | ModelingOutput::Penalty(_) => None,
            ModelingOutput::Term { .. } => term_type.as_ref(),
            ModelingOutput::Member(id) | ModelingOutput::LevelBound { parameter: id, .. } => {
                Some(&model.symbols[id].ty)
            }
            ModelingOutput::Contribution { accumulator, .. } => {
                Some(&model.closures[accumulator].ty)
            }
            ModelingOutput::Equation { id, .. } => model
                .closures
                .values()
                .find(|closure| pse_ids::named_id(closure.id, "conservation") == *id)
                .map(|closure| &closure.ty),
        };
        let ty = pse_modeling::expression::infer(
            expression,
            &types,
            &package,
            &context,
            *request.root(db),
            expected,
        )?;
        let Some(scheme) = ty.quantity_scheme() else {
            return Err(CompileError::Missing("nonphysical output".into()));
        };
        p.quantities.push(
            scheme
                .resolve_with_evidence(
                    registry,
                    &Substitution::new(),
                    inventory.preconditions(db).as_ref(),
                )
                .map_err(|e| CompileError::Missing(e.to_string()))?,
        );
    }
    // Preserve the input to the existing lowering. Preparation replays that lowering
    // for peers whose selected meaning must remain an opaque provider operation.
    let original = OriginalProjection {
        projection: p.clone(),
        bindings: bindings.clone(),
    };
    implicit::project(&model, registry, &mut p, &mut bindings)?;
    if !p.implicit.is_empty() {
        p.original = Some(Arc::new(original));
    }
    finish_projection(&mut p, &bindings, registry)?;
    Ok(Arc::new(p))
}
fn finish_projection(
    p: &mut Projection,
    bindings: &[(String, Expr)],
    registry: &QuantityRegistry,
) -> Result<()> {
    // Scatter addition has no authority to convert between physical contracts. A
    // mixed ledger keeps its complete typed expression, whose declared operations
    // consume the individual payloads; contribution observations remain separate.
    let conservation_quantities = p
        .outputs
        .iter()
        .zip(&p.quantities)
        .filter(|(output, _)| {
            matches!(
                output,
                ModelingOutput::Equation { .. } | ModelingOutput::Contribution { .. }
            )
        })
        .map(|(output, quantity)| {
            Ok((
                output.row_id(),
                registry.quantity_type(*quantity).map_err(MathError::from)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    p.conservation.retain(|row, terms| {
        let Some(ledger) = conservation_quantities.get(row) else {
            return false;
        };
        terms.iter().all(|(term, _)| {
            conservation_quantities
                .get(&pse_ids::named_id(*term, "modeling-contribution-output"))
                .is_some_and(|physical| {
                    physical.key == ledger.key && physical.canonical_unit == ledger.canonical_unit
                })
        })
    });
    bind_members(p, bindings);
    Ok(())
}
fn bind_members(p: &mut Projection, bindings: &[(String, Expr)]) {
    let members = MemberReads::new(bindings, &p.validity);
    for expression in &mut p.expressions {
        let needed = members.closure(expression);
        if !needed.is_empty() {
            *expression = Expr {
                kind: ExprKind::Let {
                    bindings: needed.iter().map(|&i| bindings[i].clone()).collect(),
                    body: Box::new(expression.clone()),
                },
                span: Span::default(),
            };
        }
        expression.strip_spans();
    }
}
/// The expression members an output reads, transitively and through the validity guards
/// of what it reads. Each output carries only these, in their topological order: wrapping
/// every output in every member grows with the product of outputs and members.
struct MemberReads {
    index: BTreeMap<String, usize>,
    reads: Vec<Vec<usize>>,
    guards: BTreeMap<String, Vec<String>>,
}
impl MemberReads {
    fn new(
        bindings: &[(String, Expr)],
        validity: &BTreeMap<String, crate::typed_math::Validity>,
    ) -> Self {
        let names = |e: &Expr| {
            e.free_paths()
                .into_iter()
                .filter(|p| p.segments.len() == 1)
                .map(|p| p.segments[0].name.clone())
                .collect::<Vec<_>>()
        };
        let index = bindings
            .iter()
            .enumerate()
            .map(|(i, (name, _))| (name.clone(), i))
            .collect::<BTreeMap<_, _>>();
        let guards = validity
            .iter()
            .map(|(name, guard)| {
                let mut read = names(&guard.lower);
                read.extend(names(&guard.upper));
                (name.clone(), read)
            })
            .collect::<BTreeMap<_, _>>();
        let mut members = Self {
            index,
            reads: Vec::new(),
            guards,
        };
        members.reads = bindings
            .iter()
            .map(|(name, e)| {
                let mut read = names(e);
                read.push(name.clone());
                members.resolve(read)
            })
            .collect();
        members
    }
    /// Member positions a set of names reads directly, including their guards' reads.
    fn resolve(&self, names: Vec<String>) -> Vec<usize> {
        let mut out = Vec::new();
        for name in names {
            if let Some(read) = self.guards.get(&name) {
                out.extend(read.iter().filter_map(|n| self.index.get(n).copied()));
            }
            out.extend(self.index.get(&name).copied());
        }
        out
    }
    /// The positions of every member the expression reads, in topological order.
    fn closure(&self, expression: &Expr) -> BTreeSet<usize> {
        let mut needed = BTreeSet::new();
        let mut pending = self.resolve(
            expression
                .free_paths()
                .into_iter()
                .filter(|p| p.segments.len() == 1)
                .map(|p| p.segments[0].name.clone())
                .collect(),
        );
        while let Some(i) = pending.pop() {
            if needed.insert(i) {
                pending.extend(&self.reads[i]);
            }
        }
        needed
    }
}
#[salsa::tracked(returns(clone),lru=64,heap_size=admitted_heap)]
fn admitted(
    db: &dyn CompilerDb,
    inventory: Inventory,
    catalog: Catalog,
    request: Request,
) -> Result<Arc<AdmittedModeling>> {
    checkpoint(db);
    let p = projection(db, inventory, catalog, request)?;
    grouped::admit(db, inventory, &p)
}
impl CompilerWorkspace {
    /// Project a finite specialization through the existing typed math admission path.
    /// # Errors
    /// An invalid package, unsupported execution construct, physical error or resource limit.
    pub fn admit_modeling(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
    ) -> Result<Arc<AdmittedModeling>> {
        self.db.cancel = Arc::new(AtomicBool::new(false));
        let (catalog, request) = self.modeling_request(root, instance, bindings, limits)?;
        let result =
            salsa::Cancelled::catch(|| admitted(&self.db, self.inventory, catalog, request))
                .map_err(|_| compiler_cancelled(&self.db))?;
        self.trim_queries()?;
        result
    }
}

/// Complete owned planning product for one exact, instance-qualified request.
/// Normalized consumer bodies are requested here and materialized only by completion.
/// Direct nested implicit mathematics remains retained without a portable semantic identity.
#[derive(Clone, Debug)]
pub struct ModelingPreparationFrontier {
    semantic: SemanticModeling,
    model: Arc<SpecializedModel>,
    projection: Arc<Projection>,
    primary: grouped::PlannedModeling,
    original: Option<(Arc<Projection>, grouped::PlannedModeling)>,
    physical: ContentHash,
    environment: ContentHash,
    providers: BTreeMap<String, (ContentHash, usize)>,
}
impl ModelingPreparationFrontier {
    /// Every compiler-issued portable body dependency, including the original view.
    /// Repeated consumer uses of the same complete identity issue one request.
    pub fn body_requests(&self) -> impl Iterator<Item = pse_ids::roles::SemanticBodyHash> {
        self.primary
            .body_requests()
            .chain(
                self.original
                    .iter()
                    .flat_map(|(_, plan)| plan.body_requests()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
    }
    /// Conservative owned planning bytes, including retained direct implicit products.
    pub fn retained_bytes(&self) -> usize {
        2 * size_of::<Self>()
            + 256
            + self.semantic.descriptor_bytes()
            + self.model.retained_bytes()
            + projection_body_bytes(&self.projection)
            + self.primary.retained_bytes()
            + self.original.as_ref().map_or(0, |(projection, plan)| {
                projection_body_bytes(projection) + plan.retained_bytes()
            })
            + self
                .providers
                .keys()
                .map(|name| name.capacity() + 128)
                .sum::<usize>()
    }
}

/// Finite kernel preparation and post-specialization structural evidence.
#[derive(Clone, Debug)]
pub struct PreparedModeling {
    semantic: SemanticModeling,
    projection: Arc<Projection>,
    owned_implicit_view: Option<pse_math::SharedAllocation<AdmittedModeling>>,
    original: Option<(
        Arc<Projection>,
        pse_math::SharedAllocation<AdmittedModeling>,
    )>,
    /// Instantiated members, demand chains, values and original closure terms.
    pub model: pse_math::SharedAllocation<SpecializedModel>,
    /// Typed finite math with stable semantic input/output coordinates.
    pub admitted: pse_math::SharedAllocation<AdmittedModeling>,
}
impl AdmittedModeling {
    /// Borrow the admitted case inventory without exporting its unowned Arc.
    pub fn case(&self) -> &CaseStructure {
        &self.case
    }
    /// Borrow nested scientific descriptors; runtime aliases inherit their view owner.
    pub fn implicit_systems(&self) -> impl Iterator<Item = &AdmittedImplicit> {
        self.implicit.values().map(AsRef::as_ref)
    }
}
impl PreparedModeling {
    /// Complete portable body inventory, independent of which pure queries executed.
    /// Primary and original views share one entry per sealed semantic identity; direct
    /// implicit bodies are retained by their systems and have no invented portable key.
    pub fn portable_bodies(&self) -> impl Iterator<Item = &Arc<AdmittedBody>> {
        self.admitted
            .bodies
            .values()
            .chain(
                self.original
                    .iter()
                    .flat_map(|(_, admitted)| admitted.bodies.values()),
            )
            .filter_map(|body| body.semantic_identity().map(|identity| (identity, body)))
            .collect::<BTreeMap<_, _>>()
            .into_values()
    }
    /// Original coordinates and equations for eligible selected suppliers. Other
    /// selected operations retain their existing nested guards, regimes and descriptors.
    pub fn original_equations(&self) -> Option<Self> {
        let (projection, admitted) = self.original.as_ref()?;
        let mut model = self.clone();
        model.projection = projection.clone();
        model.admitted = admitted.clone();
        model.owned_implicit_view = None;
        model.original = None;
        Some(model)
    }
    /// Semantic process meaning with no dependency on numerical projection.
    pub fn semantic(&self) -> SemanticModeling {
        self.semantic.clone()
    }
    /// Child residual providers before consumers, retaining this admitted view's owner.
    /// # Errors
    /// A cyclic admitted provider dependency.
    pub fn implicit_order(&self) -> Result<Vec<pse_math::SharedAllocation<AdmittedImplicit>>> {
        self.implicit_order_for(None)
    }
    /// Retain only nested providers reachable from selected observation rows.
    /// # Errors
    /// A cyclic provider dependency or missing selected body.
    pub fn implicit_order_for(
        &self,
        rows: Option<&BTreeSet<SemanticId>>,
    ) -> Result<Vec<pse_math::SharedAllocation<AdmittedImplicit>>> {
        Ok(self
            .admitted
            .implicit_order_for(rows)?
            .into_iter()
            .map(|value| self.admitted.share_child(value))
            .collect())
    }
    /// Attach the runtime admission owner to independently escaping model/view aliases.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        let attached = self
            .owned_implicit_view
            .as_ref()
            .is_some_and(|view| pse_math::SharedAllocation::ptr_eq(view, &self.admitted));
        if !attached && !self.admitted.implicit.is_empty() {
            let mut admitted = self.admitted.as_ref().clone();
            for implicit in admitted.implicit.values_mut() {
                *implicit = Arc::new(implicit.as_ref().clone().with_owner(owner.clone()));
            }
            self.admitted = self.admitted.share_child(Arc::new(admitted));
            self.owned_implicit_view = Some(self.admitted.clone());
        }
        self.model = self.model.with_owner(owner.clone());
        self.semantic = self.semantic.with_owner(owner.clone());
        if let Some((_, original)) = &mut self.original {
            let mut view = original.as_ref().clone();
            for implicit in view.implicit.values_mut() {
                *implicit = Arc::new(implicit.as_ref().clone().with_owner(owner.clone()));
            }
            *original = original
                .share_child(Arc::new(view))
                .with_owner(owner.clone());
        }
        self.admitted = self.admitted.with_owner(owner);
        self
    }
}
impl CompilerWorkspace {
    /// Plan an exact request without materializing its normalized consumer bodies.
    /// The returned product owns its model, projection, implicit products and body requests.
    /// # Errors
    /// Invalid semantics, physical admission, resource limits or cancellation.
    pub fn plan_modeling_cancellable(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: Arc<AtomicBool>,
    ) -> Result<ModelingPreparationFrontier> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        let (catalog, request) = self.modeling_request(root, instance, bindings, limits)?;
        self.db.cancel = cancel;
        let result = salsa::Cancelled::catch(|| {
            let model = specialized(&self.db, catalog, request)?;
            let semantic = SemanticModeling::admit(
                model.clone().into(),
                self.inventory.quantities(&self.db),
                self.inventory.preconditions(&self.db),
            )?;
            let projection = projection(&self.db, self.inventory, catalog, request)?;
            let primary = grouped::plan(&self.db, self.inventory, &projection, BTreeMap::new())?;
            let promoted: BTreeSet<_> = primary
                .implicit
                .values()
                .filter(|supplier| {
                    supplier.selection.neighborhood_evidence != SelectionNeighborhood::Unestablished
                        && !matches!(supplier.selection.meaning, ImplicitMeaning::Relation)
                        && supplier.selection.restriction.is_none()
                        && supplier.residuals.len() == 1
                        && supplier.residuals[0].assessment.is_none()
                        && matches!(
                            supplier.algorithm,
                            ImplicitAlgorithm::Native | ImplicitAlgorithm::Accelerator(_)
                        )
                })
                .map(|supplier| supplier.descriptor.spec().id)
                .collect();
            let original = projection
                .original
                .as_ref()
                .filter(|_| !promoted.is_empty())
                .map(|raw| -> Result<_> {
                    let mut remaining = model.as_ref().clone();
                    remaining
                        .implicit
                        .retain(|instance, _| !promoted.contains(&instance.as_id()));
                    let mut p = raw.projection.clone();
                    let mut bindings = raw.bindings.clone();
                    implicit::project(&remaining, &self.inputs.quantities, &mut p, &mut bindings)?;
                    finish_projection(&mut p, &bindings, &self.inputs.quantities)?;
                    let supplied = primary
                        .implicit
                        .iter()
                        .filter(|(id, _)| promoted.contains(id))
                        .map(|(id, supplier)| (*id, supplier.clone()))
                        .collect();
                    let source = grouped::plan(&self.db, self.inventory, &p, supplied)?;
                    Ok((Arc::new(p), source))
                })
                .transpose()?;
            checkpoint(&self.db);
            Ok(ModelingPreparationFrontier {
                semantic,
                model,
                projection,
                primary,
                original,
                physical: physical_identity(
                    self.inventory.quantities(&self.db),
                    self.inventory.preconditions(&self.db),
                ),
                environment: *self.inventory.environment(&self.db),
                providers: self
                    .inventory
                    .providers(&self.db)
                    .iter()
                    .map(|(name, provider)| {
                        (
                            name.clone(),
                            (provider.descriptor.spec().identity(), provider.output),
                        )
                    })
                    .collect(),
            })
        })
        .map_err(|_| compiler_cancelled(&self.db))?;
        self.trim_queries()?;
        result
    }

    /// Materialize the frontier's exact normalized requests through pure body retention.
    /// A rotated workspace may receive the owned frontier if its immutable inventory agrees.
    /// # Errors
    /// Changed context, invalid math, resource limits or retryable cancellation.
    pub fn complete_modeling_cancellable(
        &mut self,
        frontier: ModelingPreparationFrontier,
        cancel: Arc<AtomicBool>,
    ) -> Result<PreparedModeling> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        self.db.cancel = cancel;
        // Completion consumes immutable owned instructions. Allocation owners may
        // contain interior accounting state, but cancellation cannot expose a partly
        // constructed product; non-cancellation panics are rethrown by Salsa.
        let result = salsa::Cancelled::catch(std::panic::AssertUnwindSafe(|| {
            checkpoint(&self.db);
            let providers = self.inventory.providers(&self.db);
            if frontier.physical
                != physical_identity(
                    self.inventory.quantities(&self.db),
                    self.inventory.preconditions(&self.db),
                )
                || frontier.environment != *self.inventory.environment(&self.db)
                || frontier.providers.len() != providers.len()
                || !frontier.providers.iter().all(|(name, identity)| {
                    providers.get(name).is_some_and(|provider| {
                        *identity == (provider.descriptor.spec().identity(), provider.output)
                    })
                })
            {
                return Err(CompileError::from(MathError::Contract(
                    "modeling frontier has a different immutable compiler inventory".into(),
                )));
            }
            let admitted = grouped::complete(&self.db, self.inventory, frontier.primary)?;
            let original = frontier
                .original
                .map(|(projection, plan)| -> Result<_> {
                    Ok((
                        projection,
                        grouped::complete(&self.db, self.inventory, plan)?.into(),
                    ))
                })
                .transpose()?;
            checkpoint(&self.db);
            Ok(PreparedModeling {
                semantic: frontier.semantic,
                projection: frontier.projection,
                original,
                owned_implicit_view: None,
                model: frontier.model.into(),
                admitted: admitted.into(),
            })
        }))
        .map_err(|_| compiler_cancelled(&self.db))?;
        self.trim_queries()?;
        result
    }

    /// Prepare the exact request through the same owned planning and completion route.
    /// # Errors
    /// Invalid semantics, physical admission, resource limits or cancellation.
    pub fn prepare_modeling_cancellable(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: Arc<AtomicBool>,
    ) -> Result<PreparedModeling> {
        let frontier =
            self.plan_modeling_cancellable(root, instance, bindings, limits, cancel.clone())?;
        self.complete_modeling_cancellable(frontier, cancel)
    }
}

impl PreparedModeling {
    /// Conservative known product bytes, which the runtime charges for as long as the
    /// product is retained.
    pub fn retained_bytes(&self) -> usize {
        2 * size_of::<Self>()
            + 256
            + self.semantic.descriptor_bytes()
            + self.model.retained_bytes()
            + projection_heap(&Ok(self.projection.clone()))
            + admitted_allocation_bytes(&self.admitted)
            + admitted_owner_attachment_bytes(&self.admitted)
            + self.original.as_ref().map_or(0, |(p, a)| {
                projection_heap(&Ok(p.clone())) + admitted_allocation_bytes(a)
            })
    }
}

impl CompilerWorkspace {
    /// Compile supplied-boundary differences through the existing typed consumer owner.
    pub(super) fn prepare_conditional_boundary_functions(
        &self,
        model: &PreparedModeling,
        symbols: &BTreeSet<SemanticId>,
        coordinates: Vec<SemanticId>,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedFunctions> {
        let mut p = model.projection.as_ref().clone();
        let mut selected = Vec::new();
        for symbol in symbols {
            let index = p
                .outputs
                .iter()
                .position(|o| matches!(o, ModelingOutput::Member(id) if id == symbol))
                .ok_or_else(|| {
                    CompileError::Missing("conditional boundary member projection absent".into())
                })?;
            let quantity = p.quantities[index];
            let point = pse_quantity::ResolvedPhysicalContract::named(
                quantity,
                pse_quantity::IndexSet::default(),
                &self.inputs.quantities,
            )
            .map_err(MathError::from)?;
            let admission = pse_quantity::resolved::infer_operation(
                &pse_quantity::infer::OpRequest::Sub,
                &[point.clone(), point],
                None,
                &self.inputs.quantities,
                &pse_quantity::infer::NoInvariantFacts,
            )
            .map_err(MathError::from)?;
            let difference = admission.result.require_named().map_err(MathError::from)?;
            if admission.operand_scales != [1.0, 1.0] || admission.result_scale != 1.0 {
                return Err(CompileError::Missing(
                    "conditional boundary requires canonical difference coordinates".into(),
                ));
            }
            let parameter = pse_ids::named_id(*symbol, "conditional-boundary-value");
            let path = symbol_name(parameter);
            p.inputs.push(parameter);
            p.formals.push(Formal {
                path: path.clone(),
                quantity,
            });
            p.local_quantities.insert(path.clone(), quantity);
            let rhs = Expr {
                span: p.expressions[index].span,
                kind: ExprKind::Path(dsl::Path {
                    segments: vec![dsl::PathSegment {
                        name: path,
                        indices: Vec::new(),
                    }],
                }),
            };
            let expression = Expr {
                span: p.expressions[index].span,
                kind: ExprKind::Binary {
                    op: BinaryOp::Sub,
                    lhs: Box::new(p.expressions[index].clone()),
                    rhs: Box::new(rhs),
                },
            };
            selected.push((
                ModelingOutput::ConditionalBoundary(*symbol),
                expression,
                difference,
                p.declarations[index],
            ));
        }
        p.objectives = Default::default();
        p.conservation.clear();
        p.outputs = selected.iter().map(|x| x.0.clone()).collect();
        p.expressions = selected.iter().map(|x| x.1.clone()).collect();
        p.quantities = selected.iter().map(|x| x.2).collect();
        p.declarations = selected.iter().map(|x| x.3).collect();
        let admitted = grouped::admit(&self.db, self.inventory, &p)?;
        let source = admitted.plan(
            &self.inputs.quantities,
            DerivativeOrder::Value,
            profile.assembly,
            cancel,
        )?;
        let plan = Arc::new(
            source.functions(
                &p.outputs
                    .iter()
                    .map(ModelingOutput::row_id)
                    .collect::<Vec<_>>(),
                coordinates,
                &self.inputs.quantities,
                DerivativeOrder::First,
                cancel,
            )?,
        );
        Ok(PreparedFunctions {
            artifacts: artifact_requests(&plan, profile, self.inventory.environment(&self.db)),
            plan,
        })
    }
}

fn projection_heap(value: &Result<Arc<Projection>>) -> usize {
    value.as_ref().map_or(0, |p| projection_body_bytes(p))
}
fn projection_body_bytes(p: &Projection) -> usize {
    size_of::<Projection>()
        + p.objectives.members.capacity()
            * (size_of::<pse_modeling::specialize::ObjectiveMember>() + 256)
        + p.objectives.levels.capacity()
            * (size_of::<pse_modeling::specialize::ObjectiveLevel>() + 64)
        + p.conservation
            .values()
            .map(|terms| 128 + terms.capacity() * size_of::<(SemanticId, f64)>())
            .sum::<usize>()
        + p.inputs.capacity() * size_of::<SemanticId>()
        + p.free.len() * (size_of::<SemanticId>() + 64)
        + p.unit_interval.len() * (size_of::<SemanticId>() + 64)
        + p.native
            .iter()
            .map(|c| 64 + c.identities().len() * (size_of::<SemanticId>() + 16))
            .sum::<usize>()
        + p.validity
            .iter()
            .map(|(name, v)| {
                name.capacity()
                    + 128
                    + pse_modeling::expression::retained_bytes(&v.lower)
                    + pse_modeling::expression::retained_bytes(&v.upper)
            })
            .sum::<usize>()
        + p.nonnegative.len() * (size_of::<SemanticId>() + 64)
        + p.formals.capacity() * size_of::<Formal>()
        + p.formals.iter().map(|f| f.path.capacity()).sum::<usize>()
        + p.outputs.capacity() * size_of::<ModelingOutput>()
        + p.outputs
            .iter()
            .map(|o| match o {
                ModelingOutput::DynamicRate { equations, .. } => {
                    equations.capacity() * size_of::<SemanticId>()
                }
                _ => 0,
            })
            .sum::<usize>()
        + p.expressions.capacity() * size_of::<Expr>()
        + p.expressions
            .iter()
            .map(pse_modeling::expression::retained_bytes)
            .sum::<usize>()
        + p.functions
            .iter()
            .map(|(n, f)| n.capacity() + f.retained_bytes() + 64)
            .sum::<usize>()
        + p.original.as_ref().map_or(0, |raw| {
            projection_body_bytes(&raw.projection)
                + raw.bindings.capacity() * size_of::<(String, Expr)>()
                + raw
                    .bindings
                    .iter()
                    .map(|(name, expression)| {
                        name.capacity() + pse_modeling::expression::retained_bytes(expression)
                    })
                    .sum::<usize>()
        })
        + p.implicit
            .iter()
            .map(implicit::Projection::retained_bytes)
            .sum::<usize>()
        + p.declarations.capacity() * size_of::<DeclarationId>()
        + p.quantities.capacity() * size_of::<QuantityTypeId>()
        + p.local_quantities
            .keys()
            .map(|n| n.capacity() + 128)
            .sum::<usize>()
}

fn admitted_allocation_bytes(p: &AdmittedModeling) -> usize {
    size_of::<AdmittedModeling>()
        + p.inputs.capacity() * size_of::<SemanticId>()
        + p.outputs.capacity() * size_of::<ModelingOutput>()
        + p.outputs
            .iter()
            .map(|o| match o {
                ModelingOutput::DynamicRate { equations, .. } => {
                    equations.capacity() * size_of::<SemanticId>()
                }
                _ => 0,
            })
            .sum::<usize>()
        + p.implicit
            .values()
            .map(|v| v.retained_bytes())
            .sum::<usize>()
        + p.term_outputs
            .values()
            .map(|v| size_of_val(v.as_slice()) + 128)
            .sum::<usize>()
        + p.bodies
            .values()
            .map(|b| b.math.retained_bytes() + b.descriptor_bytes())
            .sum::<usize>()
        + p.case
            .instances()
            .iter()
            .map(|i| {
                size_of_val(i)
                    + i.checked_members.len() * (size_of::<(SemanticId, SemanticId)>() + 96)
                    + size_of_val(i.slots.as_slice())
                    + size_of_val(i.contributions.as_slice())
            })
            .sum::<usize>()
        + size_of_val(p.case.variables())
        + size_of_val(p.case.parameters())
        + size_of_val(p.case.rows())
}
fn admitted_heap(value: &Result<Arc<AdmittedModeling>>) -> usize {
    value.as_ref().map_or(0, |p| admitted_allocation_bytes(p))
}
fn admitted_owner_attachment_bytes(p: &AdmittedModeling) -> usize {
    if p.implicit.is_empty() {
        return 0;
    }
    size_of::<AdmittedModeling>()
        + 128
        + p.inputs.len() * size_of::<SemanticId>()
        + p.outputs.len() * size_of::<ModelingOutput>()
        + p.outputs
            .iter()
            .map(|o| match o {
                ModelingOutput::DynamicRate { equations, .. } => {
                    equations.len() * size_of::<SemanticId>()
                }
                _ => 0,
            })
            .sum::<usize>()
        + p.bodies.len() * (size_of::<(ContentHash, Arc<AdmittedBody>)>() + 128)
        + p.term_outputs
            .values()
            .map(|v| {
                size_of::<(SemanticId, Vec<(SemanticId, f64)>)>()
                    + 128
                    + v.len() * size_of::<(SemanticId, f64)>()
            })
            .sum::<usize>()
        + p.implicit.len() * (size_of::<(SemanticId, Arc<AdmittedImplicit>)>() + 128)
        + p.implicit
            .values()
            .map(|v| v.owner_attachment_bytes())
            .sum::<usize>()
}
pub(super) fn configure(db: &mut CompilerDatabase, n: usize) {
    projection::set_lru_capacity(db, n);
    admitted::set_lru_capacity(db, n);
    grouped::configure(db, n);
}

impl PreparedModeling {
    /// Assess independently evaluated original contributions. Equation residuals are not closure evidence.
    /// # Errors
    /// Wrong output extent, missing original terms, nonfinite physical observations,
    /// or missing/invalid caller-resolved physical closure budgets.
    pub fn assess_closure(
        &self,
        outputs: &[f64],
        budgets: &BTreeMap<SemanticId, f64>,
    ) -> Result<Vec<pse_modeling::specialize::ClosureAssessment>> {
        if outputs.len() != self.admitted.outputs.len() {
            return Err(CompileError::Missing("modeling output extent".into()));
        }
        let magnitudes = self
            .admitted
            .outputs
            .iter()
            .zip(outputs)
            .filter_map(|(meaning, value)| match meaning {
                ModelingOutput::Contribution { contribution, .. } => Some((*contribution, *value)),
                _ => None,
            })
            .collect();
        Ok(self.model.assess_closure(&magnitudes, budgets)?)
    }
}
