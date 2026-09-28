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
mod solve;
pub use conformance::ModelingExpectationResult;
pub use derived::{Derivation, Derived};
pub use implicit::{AdmittedImplicit, ImplicitAlgorithm, ImplicitScale};
pub use solve::{ModelingCaseBindings, ModelingVariableState};

/// Numerical observation purpose. These do not add equations or fix variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelingHint {
    Start,
    Nominal,
    Lower,
    Upper,
    ValidLower,
    ValidUpper,
    Check,
}
/// Source-test observation role; actual and expected retain their physical representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelingTestValue {
    Actual,
    Expected,
    Tolerance,
    RelativeTolerance,
}
/// Meaning of an output, independent of execution slots and source names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelingOutput {
    /// Rate expression derived from an original time-derivative equation.
    DynamicRate {
        state: SemanticId,
        equations: Vec<SemanticId>,
    },
    /// Differential initial value derived from an explicit endpoint constraint.
    InitialState {
        state: SemanticId,
        equation: SemanticId,
    },
    Test {
        id: SemanticId,
        component: ModelingTestValue,
    },
    /// Compiled annotation expression with its source and target preserved.
    Hint {
        target: SemanticId,
        declaration: SemanticId,
        kind: ModelingHint,
    },
    /// Unrelaxed residual, retained independently of auxiliary variables.
    OriginalEquation(SemanticId),
    /// Dimensionless elastic objective contribution, also available for inspection.
    Penalty(SemanticId),
    /// Original additive term, before algebraic normalization; its sign is separate
    /// because negating an affine physical quantity is not a valid physical operation.
    Term {
        equation: SemanticId,
        ordinal: usize,
        negative: bool,
    },
    /// One equation residual in original orientation.
    Equation {
        id: SemanticId,
        sense: EquationSense,
    },
    /// One demanded expression or accounting member.
    Member(SemanticId),
    /// Original contribution magnitude for independent closure checking.
    Contribution {
        accumulator: SemanticId,
        contribution: SemanticId,
    },
}
/// Finite typed mathematics with semantic input/output coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedModeling {
    /// Inputs in the math body's exact order. Values are supplied separately.
    pub inputs: Vec<SemanticId>,
    /// Outputs in the math body's exact order.
    pub outputs: Vec<ModelingOutput>,
    /// Existing typed math artifacts, shared by normalized consumer shape.
    pub bodies: BTreeMap<ContentHash, Arc<AdmittedBody>>,
    /// Semantic gathers and output contributions into the established case assembly.
    pub case: Arc<pse_math::binding::CaseStructure>,
    /// Auxiliary original term rows and signs, outside public observable ordering.
    /// Nested residual definitions, independent of mutable native workers.
    pub implicit: BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
    pub term_outputs: BTreeMap<SemanticId, Vec<(SemanticId, f64)>>,
}
#[derive(Clone, Debug, PartialEq)]
struct Projection {
    objective: Option<(SemanticId, pse_math::binding::ObjectiveSense)>,
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
    formals: Vec<Formal>,
    outputs: Vec<ModelingOutput>,
    expressions: Vec<Expr>,
    functions: BTreeMap<String, pse_modeling::Function>,
    quantities: Vec<QuantityTypeId>,
    local_quantities: BTreeMap<String, QuantityTypeId>,
    declarations: Vec<SemanticId>,
    implicit: Vec<implicit::Projection>,
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
            policy,
        } = &a.value
        {
            if !validity_targets.insert(a.target) {
                return Err(CompileError::Missing(
                    "competing validity intervals for a model member".into(),
                ));
            }
            if policy == "reject" {
                validity.insert(
                    symbol_name(a.target),
                    crate::typed_math::Validity {
                        lower: lower.clone(),
                        upper: upper.clone(),
                        source: a.lineage.declaration,
                    },
                );
            }
        }
    }
    let objectives = model
        .annotations
        .iter()
        .filter_map(|a| {
            if let pse_modeling::annotation::AnnotationValue::Objective(sense) = a.value {
                Some((
                    a.target,
                    match sense {
                        pse_modeling::annotation::ObjectiveSense::Minimize => {
                            pse_math::binding::ObjectiveSense::Minimize
                        }
                        pse_modeling::annotation::ObjectiveSense::Maximize => {
                            pse_math::binding::ObjectiveSense::Maximize
                        }
                    },
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if objectives.len() > 1 {
        return Err(CompileError::Missing(
            "a selected analysis has exactly one objective".into(),
        ));
    }
    let mut p = Projection {
        objective: objectives.first().copied(),
        body_limits: BodyLimits {
            occurrences: request
                .limits(db)
                .body_occurrences
                .unwrap_or_else(|| BodyLimits::default().occurrences),
            ..BodyLimits::default()
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
            .collect(),
        inputs: vec![],
        free: BTreeMap::new(),
        unit_interval: model.unit_interval.clone(),
        native: model.native.clone(),
        formals: vec![],
        outputs: vec![],
        expressions: vec![],
        functions: model.functions.clone(),
        quantities: Vec::new(),
        local_quantities: BTreeMap::new(),
        declarations: Vec::new(),
        implicit: Vec::new(),
    };
    for f in p.functions.values_mut() {
        if let Some(body) = &mut f.body {
            body.strip_spans();
        }
    }
    let mut graph = DiGraph::<SemanticId, ()>::new();
    let nodes = model
        .symbols
        .iter()
        .filter(|(_, s)| s.expression.is_some())
        .map(|(id, _)| (*id, graph.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    for (id, symbol) in &model.symbols {
        if let Some(expression) = bound_expressions.get(id) {
            let Type::Quantity(scheme) = &symbol.ty else {
                return Err(CompileError::Missing(
                    "nonphysical expression member".into(),
                ));
            };
            p.local_quantities.insert(
                symbol_name(*id),
                scheme
                    .resolve_with_evidence(
                        &registry,
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
            let Type::Quantity(scheme) = &symbol.ty else {
                return Err(CompileError::Missing("nonphysical runtime symbol".into()));
            };
            let quantity = scheme
                .resolve_with_evidence(
                    &registry,
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
                .filter_map(|p| (p.segments.len() == 1).then(|| p.segments[0].name.clone()))
                .collect::<BTreeSet<_>>();
            loop {
                let before = needed.len();
                for (name, expression) in &bindings {
                    if needed.contains(name) {
                        needed.extend(expression.free_paths().into_iter().filter_map(|p| {
                            (p.segments.len() == 1).then(|| p.segments[0].name.clone())
                        }));
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
        p.expressions.push(
            dsl::parse_expr(&symbol_name(derivative.rate))
                .map_err(|e| CompileError::Missing(e.to_string()))?,
        );
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
            if let Some(expression) = expression {
                if initial.replace((*id, expression.clone())).is_some() {
                    return Err(CompileError::Missing(
                        "multiple initial conditions for one state".into(),
                    ));
                }
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
    // An admitted input is also an observable value. Export its identity expression
    // through the same library body, so explicit function/fit selections do not
    // require an authored alias or report annotation. Synthetic rate coordinates
    // remain internal to integrated residual construction.
    for id in p
        .inputs
        .iter()
        .filter(|id| !model.derivatives.values().any(|d| d.rate == **id))
    {
        p.outputs.push(ModelingOutput::Member(*id));
        p.expressions.push(
            dsl::parse_expr(&symbol_name(*id)).map_err(|e| CompileError::Missing(e.to_string()))?,
        );
    }
    for node in order {
        let id = graph[node];
        p.outputs.push(ModelingOutput::Member(id));
        p.expressions.push(
            dsl::parse_expr(&symbol_name(id)).map_err(|e| CompileError::Missing(e.to_string()))?,
        );
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
            A::Check(predicate) => push(
                ModelingHint::Check,
                Expr {
                    kind: ExprKind::Conditional {
                        guard: Box::new(predicate.clone()),
                        then: Box::new(
                            dsl::parse_expr("1")
                                .map_err(|e| CompileError::Missing(e.to_string()))?,
                        ),
                        otherwise: Box::new(
                            dsl::parse_expr("0")
                                .map_err(|e| CompileError::Missing(e.to_string()))?,
                        ),
                    },
                    span: Span::default(),
                },
            ),
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
            p.expressions.push(
                dsl::parse_expr(&symbol_name(a.target))
                    .map_err(|e| CompileError::Missing(e.to_string()))?,
            );
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
            ModelingOutput::Member(id) => model.symbols.get(id).map(|s| s.lineage.declaration),
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
            if *kind == ModelingHint::Check {
                Some(Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                    registry
                        .neutral_dimensionless()
                        .ok_or_else(|| CompileError::Missing("dimensionless check".into()))?,
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
                let Type::Quantity(q) = ty else {
                    return Err(CompileError::Missing("test physical type".into()));
                };
                Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                    pse_quantity::scheme::Scheme::Delta(Box::new(q.clone()))
                        .resolve_with_evidence(
                            &registry,
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
        let expected = match output {
            ModelingOutput::InitialState { state, .. } => Some(&model.symbols[state].ty),
            ModelingOutput::DynamicRate { state, .. } => {
                Some(&model.symbols[&model.derivatives[state].rate].ty)
            }
            ModelingOutput::Test { .. } => test_type.as_ref(),
            ModelingOutput::Hint { .. } => hint_type.as_ref(),
            ModelingOutput::OriginalEquation(_) | ModelingOutput::Penalty(_) => None,
            ModelingOutput::Term { .. } => term_type.as_ref(),
            ModelingOutput::Member(id) => Some(&model.symbols[id].ty),
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
        let Type::Quantity(scheme) = ty else {
            return Err(CompileError::Missing("nonphysical output".into()));
        };
        p.quantities.push(
            scheme
                .resolve_with_evidence(
                    &registry,
                    &Substitution::new(),
                    inventory.preconditions(db).as_ref(),
                )
                .map_err(|e| CompileError::Missing(e.to_string()))?,
        );
    }
    implicit::project(&model, &registry, &mut p, &mut bindings)?;
    p.conservation.retain(|row, _| {
        p.outputs
            .iter()
            .any(|output| matches!(output, ModelingOutput::Equation {id, ..} if id == row))
    });
    for expression in &mut p.expressions {
        if !bindings.is_empty() {
            *expression = Expr {
                kind: ExprKind::Let {
                    bindings: bindings.clone(),
                    body: Box::new(expression.clone()),
                },
                span: Span::default(),
            };
        }
        expression.strip_spans();
    }
    Ok(Arc::new(p))
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
        root: SemanticId,
        instance: SemanticId,
        bindings: Bindings,
        limits: Limits,
    ) -> Result<Arc<AdmittedModeling>> {
        self.db.cancel = Arc::new(AtomicBool::new(false));
        let (catalog, request) = self.modeling_request(root, instance, bindings, limits)?;
        let result =
            salsa::Cancelled::catch(|| admitted(&self.db, self.inventory, catalog, request))
                .map_err(|_| CompileError::Cancelled)?;
        self.trim_queries()?;
        result
    }
}

/// Finite kernel preparation and post-specialization structural evidence.
#[derive(Clone, Debug)]
pub struct PreparedModeling {
    /// Instantiated members, demand chains, values and original closure terms.
    pub model: Arc<SpecializedModel>,
    /// Typed finite math with stable semantic input/output coordinates.
    pub admitted: Arc<AdmittedModeling>,
    /// Library matching and structural partition. This does not claim numerical rank.
    pub structure: Arc<StructuralAnalysis>,
}
#[salsa::tracked(returns(clone),lru=64,heap_size=structure_heap)]
fn structure(
    db: &dyn CompilerDb,
    inventory: Inventory,
    catalog: Catalog,
    request: Request,
) -> Result<Arc<StructuralAnalysis>> {
    let p = projection(db, inventory, catalog, request)?;
    let body = admitted(db, inventory, catalog, request)?;
    let mut rows = Vec::new();
    let mut edges = Vec::new();
    for (output, meaning) in p.outputs.iter().enumerate() {
        if let ModelingOutput::Equation { id, sense } = meaning {
            let (lower, upper) = match sense {
                EquationSense::Eq => (Some(0.), Some(0.)),
                EquationSense::Le => (None, Some(0.)),
                EquationSense::Ge => (Some(0.), None),
            };
            rows.push(Constraint {
                id: *id,
                lower,
                upper,
            });
            let mut columns = BTreeSet::new();
            for occurrence in body.case.instances() {
                for contribution in &occurrence.contributions {
                    if contribution.target != pse_math::binding::Target::Row(*id) {
                        continue;
                    }
                    for slot in
                        &body.bodies[&occurrence.body].math.support().first[contribution.output]
                    {
                        columns.insert(occurrence.slots[*slot].source());
                    }
                }
            }
            for column in columns {
                if p.free.contains_key(&column) {
                    edges.push(Incidence {
                        row: *id,
                        column,
                        instance: *request.instance(db),
                        output,
                    });
                }
            }
        }
    }
    let incidence = CaseIncidence::new(
        Scope::Whole(*request.instance(db)),
        rows,
        p.free.keys().copied().collect(),
        edges,
        BTreeSet::new(),
        GraphLimits {
            nodes: 200_000,
            edges: 1_000_000,
        },
    )?;
    checkpoint(db);
    let result = incidence.analyze(db.cancel());
    checkpoint(db);
    Ok(Arc::new(result?))
}
impl CompilerWorkspace {
    /// Atomically prepare typed math, current values and structural evidence in one generation.
    /// # Errors
    /// Invalid semantics, physical admission, resource limits or cancellation.
    pub fn prepare_modeling_cancellable(
        &mut self,
        root: SemanticId,
        instance: SemanticId,
        bindings: Bindings,
        limits: Limits,
        cancel: Arc<AtomicBool>,
    ) -> Result<PreparedModeling> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        let (catalog, request) = self.modeling_request(root, instance, bindings, limits)?;
        self.db.cancel = cancel;
        let result = salsa::Cancelled::catch(|| {
            let model = specialized(&self.db, catalog, request)?;
            let admitted = admitted(&self.db, self.inventory, catalog, request)?;
            let structure = structure(&self.db, self.inventory, catalog, request)?;
            Ok(PreparedModeling {
                model,
                admitted,
                structure,
            })
        })
        .map_err(|_| CompileError::Cancelled)?;
        self.trim_queries()?;
        result
    }
}

impl PreparedModeling {
    /// Conservative known product bytes; foreign library allowance is reserved by the runtime.
    pub fn retained_bytes(&self) -> usize {
        self.model.retained_bytes()
            + admitted_heap(&Ok(self.admitted.clone()))
            + structure_heap(&Ok(self.structure.clone()))
    }
}

fn projection_heap(value: &Result<Arc<Projection>>) -> usize {
    value.as_ref().map_or(0, |p| {
        size_of::<Projection>()
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
            + p.implicit
                .iter()
                .map(implicit::Projection::retained_bytes)
                .sum::<usize>()
            + p.declarations.capacity() * size_of::<SemanticId>()
            + p.quantities.capacity() * size_of::<QuantityTypeId>()
            + p.local_quantities
                .iter()
                .map(|(n, _)| n.capacity() + 128)
                .sum::<usize>()
    })
}
fn admitted_heap(value: &Result<Arc<AdmittedModeling>>) -> usize {
    value.as_ref().map_or(0, |p| {
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
                .map(|b| {
                    b.math.retained_bytes()
                        + b.quantities.capacity() * size_of::<QuantityTypeId>()
                        + b.occurrences.capacity() * size_of::<crate::typed_math::Occurrence>()
                })
                .sum::<usize>()
            + p.case
                .instances()
                .iter()
                .map(|i| {
                    size_of_val(i)
                        + size_of_val(i.slots.as_slice())
                        + size_of_val(i.contributions.as_slice())
                })
                .sum::<usize>()
            + size_of_val(p.case.variables())
            + size_of_val(p.case.parameters())
            + size_of_val(p.case.rows())
    })
}
pub(super) fn configure(db: &mut CompilerDatabase, n: usize) {
    projection::set_lru_capacity(db, n);
    admitted::set_lru_capacity(db, n);
    structure::set_lru_capacity(db, n);
}

impl PreparedModeling {
    /// Assess independently evaluated original contributions. Equation residuals are not closure evidence.
    /// # Errors
    /// Wrong output extent, missing original terms, or nonfinite physical observations.
    pub fn assess_closure(
        &self,
        outputs: &[f64],
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
        Ok(self.model.assess_closure(&magnitudes)?)
    }
}
