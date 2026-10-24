// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite kernel projection into the established physically checked math boundary.
use super::*;
use petgraph::{algo::toposort, graph::DiGraph};
use pse_authoring::dsl::{self, EquationSense};
use pse_modeling::specialize::symbol_name;
use pse_quantity::scheme::Substitution;
use std::collections::BTreeSet;
mod conformance;
mod dependencies;
mod derived;
mod factorable;
mod grouped;
mod implicit;
mod policy;
pub use policy::{
    AnnotationObservation, CompilerView, NumericalAnnotation, NumericalAnnotationMode,
    NumericalAnnotations, NumericalScale, NumericalStartSource, NumericalStartValues,
    ScalingObservations, ViewChoice, ViewPolicy, ViewReason,
};
#[cfg(test)]
#[path = "executable/projection_tests.rs"]
mod projection_tests;
mod solve;
pub use conformance::{ModelingExpectationResult, ModelingPointChecks, ModelingValidityResult};
pub use derived::{Derivation, Derived};
pub use implicit::{
    AdmittedImplicit, ImplicitAlgorithm, ImplicitCapabilities, ImplicitMeaning,
    ImplicitProgramRequest, ImplicitPrograms, ImplicitScale, ImplicitSelection,
    SelectionEquivalence, SelectionNeighborhood,
};
pub use solve::{
    BoundLookup, BoundLookupKey, BoundStructure, ModelingCaseBindings, ModelingOccurrenceIdentity,
    ModelingVariableState,
};

/// Numerical observation purpose. These do not add equations or fix variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// Classify the already issued predicate against the selected physical objective.
    /// The side is an existing typed root; no spelling or source syntax supplies identity.
    pub(crate) fn classify(
        model: &SpecializedModel,
        objective: Option<(SemanticId, pse_math::binding::ObjectiveSense)>,
        predicate: pse_modeling::resolved::PredicateIx,
    ) -> Option<(Self, pse_modeling::resolved::NodeIx)> {
        use dsl::CompareOp as Op;
        use pse_math::binding::ObjectiveSense as Sense;
        use pse_modeling::resolved::{Node, Reference, ResolvedPredicate};
        let (target, sense) = objective?;
        let ResolvedPredicate::Compare { op, lhs, rhs } =
            model.resolved.predicates.get(predicate)?
        else {
            return None;
        };
        let is_objective = |root| {
            matches!(model.resolved.nodes.get(root), Some(Node::Reference(Reference::Symbol(slot)))
                if model.symbols.at(*slot).is_some_and(|symbol| symbol.id == target))
        };
        let (op, side) = match (is_objective(*lhs), is_objective(*rhs)) {
            (true, false) => (*op, *rhs),
            (false, true) => (
                match op {
                    Op::Lt => Op::Gt,
                    Op::Le => Op::Ge,
                    Op::Gt => Op::Lt,
                    Op::Ge => Op::Le,
                    other => *other,
                },
                *lhs,
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
        decision_free(model, side).then_some((bound, side))
    }
}
/// Follow the model's exact typed numerical and guard dependencies. Defined members keep
/// their nominal boundary; only independent parameter leaves are decision-free.
fn decision_free(model: &SpecializedModel, root: pse_modeling::resolved::NodeIx) -> bool {
    use pse_model::generated::enums::ModelingDeclarationKind as Kind;
    use pse_modeling::resolved::{RegionOutput, RegionRequest};
    let outputs = [RegionOutput::Value(root)];
    let Ok(reads) = model.resolved.reads(RegionRequest {
        outputs: &outputs,
        validity: &[],
    }) else {
        return false;
    };
    reads.numeric.union(&reads.guards).all(|id| {
        model
            .symbols
            .get(id)
            .is_some_and(|symbol| symbol.definition.is_some() || symbol.role == Kind::Parameter)
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
#[derive(Clone, Debug)]
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
    supplier_topology: Arc<once_cell::sync::OnceCell<implicit::SupplierTopology>>,
    supplier_topology_bytes: usize,
    /// Auxiliary original term rows and signs, outside public observable ordering.
    pub term_outputs: BTreeMap<SemanticId, Vec<(SemanticId, f64)>>,
}
impl PartialEq for AdmittedModeling {
    fn eq(&self, other: &Self) -> bool {
        self.inputs == other.inputs
            && self.outputs == other.outputs
            && self.bodies == other.bodies
            && self.case == other.case
            && self.implicit == other.implicit
            && self.term_outputs == other.term_outputs
    }
}
#[derive(Clone, Debug, PartialEq)]
struct Projection {
    model: Arc<SpecializedModel>,
    regions: Vec<pse_modeling::resolved::SelectedRegion>,
    physical_scope: Arc<PhysicalScope>,
    annotation_scales: Vec<NumericalScale>,
    annotation_plans: Option<Arc<policy::AnnotationPlans>>,
    /// Authored objective members grouped into levels (ADR-0111): priority, weight,
    /// normalization, and each level's tolerances and generated bound.
    objectives: pse_modeling::specialize::Objectives,
    body_limits: BodyLimits,
    /// Original additive conservation terms assembled into their semantic row.
    conservation: BTreeMap<SemanticId, Vec<(SemanticId, f64)>>,
    nonnegative: BTreeSet<SemanticId>,
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
    quantities: Vec<QuantityTypeId>,
    declarations: Vec<DeclarationId>,
    implicit: Vec<implicit::Projection>,
    original: Option<Arc<OriginalProjection>>,
}
#[derive(Clone, Debug, PartialEq)]
struct OriginalProjection {
    projection: Projection,
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
    use pse_modeling::annotation::{AnnotationRoots as Roots, AnnotationValue as A};
    use pse_modeling::resolved::{Node, Reference, RegionOutput, RegionRequest};

    let model = observed_specialization(db, catalog, request)?;
    let registry = inventory.quantities(db);
    let selected_contracts = function_contracts(db, catalog, *request.root(db))?;
    let package = model.function_contracts(&selected_contracts);
    let context = package.context();
    let rows = model
        .equations
        .iter()
        .chain(model.regimes.values().flat_map(|regime| {
            regime
                .alternatives
                .iter()
                .flat_map(|alternative| &alternative.equations)
        }))
        .collect::<Vec<_>>();
    let rows_by_id = rows
        .iter()
        .map(|row| (row.id, *row))
        .collect::<BTreeMap<_, _>>();
    let contributions_by_id = model
        .closures
        .values()
        .flat_map(|closure| {
            closure
                .terms
                .iter()
                .map(move |term| ((closure.id, term.id), term))
        })
        .collect::<BTreeMap<_, _>>();
    let mut symbols_by_path = BTreeMap::new();
    for symbol in model.symbols.values() {
        symbols_by_path
            .entry(symbol.lineage.path.as_str())
            .or_insert(symbol.id);
    }
    let rate_ids = model
        .derivatives
        .values()
        .map(|derivative| derivative.rate)
        .collect::<BTreeSet<_>>();
    let annotations = model
        .annotations
        .iter()
        .chain(model.regimes.values().flat_map(|regime| {
            regime
                .alternatives
                .iter()
                .flat_map(|alternative| &alternative.annotations)
        }))
        .collect::<Vec<_>>();

    // The producer owns the actual member range and its reached guard closure. Preserve
    // the original ambiguity refusal, without constructing or re-inferring its predicates.
    let mut validity_targets = BTreeSet::new();
    for annotation in &model.annotations {
        if let A::Valid { layer, .. } = &annotation.value
            && *layer == pse_model::generated::enums::ModelingValidityLayer::Closure
            && !validity_targets.insert(annotation.target)
        {
            return Err(CompileError::Missing(
                "competing validity intervals for a model member".into(),
            ));
        }
    }

    let mut p = Projection {
        model: model.clone(),
        regions: Vec::new(),
        physical_scope: physical_scope(db, catalog, *request.root(db))?,
        annotation_scales: Vec::new(),
        annotation_plans: None,
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
                let row = pse_modeling::derived::Relationship::ClosureConservation {
                    closure: closure.id,
                }
                .identity();
                (rows_by_id.contains_key(&row) && !model.elastic.contains_key(&row)).then(|| {
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
        nonnegative: model
            .elastic
            .values()
            .flat_map(|elastic| elastic.slacks.iter().copied())
            .chain(model.nonnegative.iter().copied())
            .collect(),
        inputs: Vec::new(),
        free: BTreeMap::new(),
        unit_interval: model.unit_interval.clone(),
        native: model.native.clone(),
        requirements: model.requirements.clone(),
        formals: Vec::new(),
        outputs: Vec::new(),
        quantities: Vec::new(),
        declarations: Vec::new(),
        implicit: Vec::new(),
        original: None,
    };
    let mut dependencies = dependencies::Dependencies::new(
        model
            .symbols
            .keys()
            .copied()
            .chain(rate_ids.iter().copied())
            .collect::<BTreeSet<_>>(),
    );
    let members = model
        .symbols
        .iter()
        .filter_map(|(id, symbol)| symbol.definition.map(|_| *id))
        .collect::<BTreeSet<_>>();
    for (id, symbol) in &model.symbols {
        if let Some(root) = symbol.definition {
            if symbol.ty.quantity_scheme().is_none() {
                return Err(CompileError::Missing(
                    "nonphysical expression member".into(),
                ));
            }
            let outputs = [RegionOutput::Value(root)];
            let reads = model.resolved.reads(RegionRequest {
                outputs: &outputs,
                validity: &[],
            })?;
            dependencies.numerical(*id, reads.numeric);
            dependencies.guard(*id, reads.guards);
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
                .map_err(|error| CompileError::Missing(error.to_string()))?;
            if symbol.role == pse_model::generated::enums::ModelingDeclarationKind::Variable {
                if symbol.domain.requires_pure_number()
                    && registry
                        .discrete_category(quantity)
                        .map_err(|error| CompileError::Missing(error.to_string()))?
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
    let order = dependencies.order(&members)?;
    for row in &rows {
        let relation = *model
            .equation_roots
            .get(&row.id)
            .ok_or_else(|| CompileError::Missing("bound dispatch equation".into()))?;
        let region = model.prepare_region(
            RegionRequest {
                outputs: &[RegionOutput::RelationResidual(relation)],
                validity: &[],
            },
            &context,
        )?;
        dependencies.numerical(row.id, region.reads.numeric.iter().copied());
        dependencies.guard(row.id, region.reads.guards.iter().copied());
        projection_push_region(
            &mut p,
            ModelingOutput::Equation {
                id: row.id,
                sense: relation.sense,
            },
            region,
        )?;
    }

    // Original rate-system selection remains square, equality-only and whole-system.
    let dynamic_consumers = dependencies.consumers(rate_ids.iter().copied(), db);
    let dynamic_rows = rows
        .iter()
        .filter_map(|row| {
            dynamic_consumers
                .contains(&row.id)
                .then(|| {
                    model
                        .equation_roots
                        .get(&row.id)
                        .map(|relation| (row.id, relation.sense))
                })
                .flatten()
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
        projection_push_region(
            &mut p,
            ModelingOutput::DynamicRate {
                state: derivative.state,
                equations: dynamic_rows.iter().map(|(id, _)| *id).collect(),
            },
            model.prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::Value(model.symbol_root(derivative.rate)?)],
                    validity: &[],
                },
                &context,
            )?,
        )?;
        let direct = |root| {
            matches!(model.resolved.nodes.get(root), Some(Node::Reference(Reference::Symbol(slot)))
                if model.symbols.at(*slot).is_some_and(|symbol| symbol.id == derivative.state))
        };
        let mut initial = None;
        for id in &model.initial_equations {
            let relation = model
                .equation_roots
                .get(id)
                .ok_or_else(|| CompileError::Missing("bound initial equation".into()))?;
            if relation.sense != EquationSense::Eq {
                continue;
            }
            let value = if direct(relation.lhs) {
                Some(relation.rhs)
            } else if direct(relation.rhs) {
                Some(relation.lhs)
            } else {
                None
            };
            if let Some(value) = value
                && initial.replace((*id, value)).is_some()
            {
                return Err(CompileError::Missing(
                    "multiple initial conditions for one state".into(),
                ));
            }
        }
        if let Some((equation, root)) = initial {
            projection_push_region(
                &mut p,
                ModelingOutput::InitialState {
                    state: derivative.state,
                    equation,
                },
                model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(root)],
                        validity: &[],
                    },
                    &context,
                )?,
            )?;
        }
    }

    let instance_path = &model.instances[request.instance(db)].path;
    let mut observed = request
        .bindings(db)
        .demand
        .iter()
        .filter_map(|path| {
            model.paths.get(path).copied().or_else(|| {
                let full_path = format!("{instance_path}.{path}");
                symbols_by_path.get(full_path.as_str()).copied()
            })
        })
        .collect::<BTreeSet<_>>();
    observed.extend(model.fixtures.values().flat_map(|fixture| {
        fixture
            .specifications
            .values()
            .map(|specification| specification.target)
    }));
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
    observed.extend(model.ports.values().map(|port| port.symbol));
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
        observed.extend(model.symbols.values().filter_map(|symbol| {
            (symbol.role == pse_model::generated::enums::ModelingDeclarationKind::Variable
                && symbol.definition.is_none())
            .then_some(symbol.id)
        }));
        observed.extend(model.integrals.values().map(|integral| integral.integrand));
    }
    let input_members = p
        .inputs
        .iter()
        .filter(|id| observed.contains(id) && !rate_ids.contains(id))
        .copied()
        .collect::<Vec<_>>();
    for id in input_members
        .into_iter()
        .chain(order.into_iter().filter(|id| observed.contains(id)))
    {
        projection_push_region(
            &mut p,
            ModelingOutput::Member(id),
            model.prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::Value(model.symbol_root(id)?)],
                    validity: &[],
                },
                &context,
            )?,
        )?;
    }
    for level in &model.objectives.levels {
        if let Some(bound) = &level.bound {
            projection_push_region(
                &mut p,
                ModelingOutput::LevelBound {
                    row: bound.row,
                    parameter: bound.parameter,
                    sense: match level.sense {
                        pse_modeling::annotation::ObjectiveSense::Minimize => EquationSense::Le,
                        pse_modeling::annotation::ObjectiveSense::Maximize => EquationSense::Ge,
                    },
                },
                model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(model.symbol_root(bound.parameter)?)],
                        validity: &[],
                    },
                    &context,
                )?,
            )?;
        }
    }
    for balance in model.inventory_balances.values() {
        projection_push_region(
            &mut p,
            ModelingOutput::Inventory(balance.id),
            model.prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::Value(balance.inventory_root)],
                    validity: &[],
                },
                &context,
            )?,
        )?;
        for (event, root) in &balance.transfer_roots {
            projection_push_region(
                &mut p,
                ModelingOutput::InventoryTransfer {
                    balance: balance.id,
                    event: *event,
                },
                model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(*root)],
                        validity: &[],
                    },
                    &context,
                )?,
            )?;
        }
    }
    for closure in model.closures.values() {
        for term in &closure.terms {
            let root = *model
                .contribution_roots
                .get(&term.id)
                .ok_or_else(|| CompileError::Missing("original contribution root".into()))?;
            projection_push_region(
                &mut p,
                ModelingOutput::Contribution {
                    accumulator: closure.id,
                    contribution: term.id,
                },
                model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(root)],
                        validity: &[],
                    },
                    &context,
                )?,
            )?;
        }
    }
    for (id, elastic) in &model.elastic {
        projection_push_region(
            &mut p,
            ModelingOutput::OriginalEquation(*id),
            model.prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::RelationResidual(elastic.original_relation)],
                    validity: &[],
                },
                &context,
            )?,
        )?;
        projection_push_region(
            &mut p,
            ModelingOutput::Penalty(*id),
            model.prepare_region(
                RegionRequest {
                    outputs: &[RegionOutput::Value(elastic.penalty_root)],
                    validity: &[],
                },
                &context,
            )?,
        )?;
    }
    let objective = p.objective();
    for annotation in &annotations {
        let mut push = |kind, output| -> Result<()> {
            projection_push_region(
                &mut p,
                ModelingOutput::Hint {
                    target: annotation.target,
                    declaration: annotation.lineage.declaration,
                    kind,
                },
                model.prepare_region(
                    RegionRequest {
                        outputs: &[output],
                        validity: &[],
                    },
                    &context,
                )?,
            )
        };
        match (&annotation.value, &annotation.roots) {
            (A::Start(_), Roots::Scalar { value }) => {
                push(ModelingHint::Start, RegionOutput::Value(*value))?
            }
            (A::Nominal(_), Roots::Scalar { value }) => {
                push(ModelingHint::Nominal, RegionOutput::Value(*value))?
            }
            (A::Bounds(..), Roots::Bounds { lower, upper }) => {
                push(ModelingHint::Lower, RegionOutput::Value(*lower))?;
                push(ModelingHint::Upper, RegionOutput::Value(*upper))?;
            }
            (A::Valid { .. }, Roots::Valid { lower, upper }) => {
                push(ModelingHint::ValidLower, RegionOutput::Value(*lower))?;
                push(ModelingHint::ValidUpper, RegionOutput::Value(*upper))?;
            }
            (A::Check(_), Roots::Check { predicate }) => {
                if let Some((bound, side)) = ObjectiveBound::classify(&model, objective, *predicate)
                {
                    push(
                        ModelingHint::ObjectiveBound(bound),
                        RegionOutput::Value(side),
                    )?;
                }
                push(
                    ModelingHint::Check,
                    RegionOutput::PredicateIndicator(*predicate),
                )?;
            }
            (
                A::AccuracyGoal(_),
                Roots::Accuracy {
                    time,
                    resolution,
                    criterion_lower,
                    criterion_upper,
                },
            ) => {
                for (kind, root) in [
                    (ModelingHint::AccuracyGoalTime, time),
                    (ModelingHint::AccuracyGoalResolution, resolution),
                    (ModelingHint::AccuracyGoalLower, criterion_lower),
                    (ModelingHint::AccuracyGoalUpper, criterion_upper),
                ] {
                    if let Some(root) = root {
                        push(kind, RegionOutput::Value(*root))?;
                    }
                }
            }
            (A::EngineeringScale(_), Roots::Engineering { value }) => push(
                ModelingHint::EngineeringScaleValue,
                RegionOutput::Value(*value),
            )?,
            (
                A::EngineeringDefault { .. } | A::Scale(_) | A::Report(_) | A::Objective(_),
                Roots::None,
            ) => {}
            _ => {
                return Err(CompileError::Missing(
                    "annotation executable roots do not match its declared role".into(),
                ));
            }
        }
    }
    let mut projected_members = p
        .outputs
        .iter()
        .filter_map(|output| match output {
            ModelingOutput::Member(id) => Some(*id),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for annotation in &model.annotations {
        if matches!(annotation.value, A::Report(_) | A::Valid { .. })
            && model.symbols.contains_key(&annotation.target)
            && projected_members.insert(annotation.target)
        {
            projection_push_region(
                &mut p,
                ModelingOutput::Member(annotation.target),
                model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(model.symbol_root(annotation.target)?)],
                        validity: &[],
                    },
                    &context,
                )?,
            )?;
        }
    }
    // Parent selection order is the original public output order. The producer alone
    // splits the admitted original residual into ordered signed conditioning terms.
    let equations = p
        .outputs
        .iter()
        .filter_map(|output| match output {
            ModelingOutput::Equation { id, .. } if !model.elastic.contains_key(id) => Some(
                model
                    .equation_roots
                    .get(id)
                    .map(|relation| (*id, *relation))
                    .ok_or_else(|| CompileError::Missing("term parent equation".into())),
            ),
            ModelingOutput::OriginalEquation(id) => {
                Some(Ok((*id, model.elastic[id].original_relation)))
            }
            _ => None,
        })
        .collect::<Result<Vec<_>>>()?;
    for (equation, relation) in equations {
        for term in model.prepare_relation_terms(relation, &context)? {
            projection_push_region(
                &mut p,
                ModelingOutput::Term {
                    equation,
                    ordinal: term.ordinal,
                    negative: term.negative,
                },
                term.region,
            )?;
        }
    }
    for test in model.expectations.values() {
        for (component, root) in [
            (ModelingTestValue::Actual, test.roots.actual),
            (ModelingTestValue::Expected, test.roots.expected),
            (ModelingTestValue::Tolerance, test.roots.tolerance),
            (
                ModelingTestValue::RelativeTolerance,
                test.roots.relative_tolerance,
            ),
        ] {
            projection_push_region(
                &mut p,
                ModelingOutput::Test {
                    id: test.id,
                    component,
                },
                model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(root)],
                        validity: &[],
                    },
                    &context,
                )?,
            )?;
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
            | ModelingOutput::Penalty(id) => rows_by_id.get(id).map(|row| row.lineage.declaration),
            ModelingOutput::ConditionalBoundary(id)
            | ModelingOutput::Member(id)
            | ModelingOutput::LevelBound { parameter: id, .. } => model
                .symbols
                .get(id)
                .map(|symbol| symbol.lineage.declaration),
            ModelingOutput::Contribution {
                accumulator,
                contribution,
            } => contributions_by_id
                .get(&(*accumulator, *contribution))
                .map(|term| term.lineage.declaration),
        }
        .ok_or_else(|| CompileError::Missing("output lineage".into()))?;
        p.declarations.push(declaration);
    }
    p.annotation_scales = annotations.iter().filter_map(|annotation| {
        let A::Scale(scheme) = annotation.value else { return None; };
        Some(NumericalScale {
            target: annotation.target, declaration: annotation.lineage.declaration, scheme,
            observations: ScalingObservations::Rows(p.outputs.iter().filter(|output| {
                matches!(output, ModelingOutput::Term { equation, .. } if *equation == annotation.target)
            }).map(ModelingOutput::row_id).collect()), nested: false,
        })
    }).collect();
    let original = OriginalProjection {
        projection: p.clone(),
    };
    implicit::project(
        db,
        &model,
        registry,
        inventory.preconditions(db).as_ref(),
        &mut p,
    )?;
    if !p.implicit.is_empty() {
        p.original = Some(Arc::new(original));
    }
    finish_projection(db, &mut p, registry)?;
    Ok(Arc::new(p))
}

/// Projection bookkeeping consumes the actual producer's output boundary, never
/// a caller-provided expected physical type or a second source-expression admission.
fn projection_push_region(
    projection: &mut Projection,
    output: ModelingOutput,
    region: pse_modeling::resolved::SelectedRegion,
) -> Result<()> {
    let root = *region
        .syntax
        .outputs
        .first()
        .ok_or_else(|| CompileError::Missing("selected projection output".into()))?;
    if region.syntax.outputs.len() != 1 {
        return Err(CompileError::Missing(
            "projection region must have one ordered output".into(),
        ));
    }
    let quantity = region
        .support
        .output(0)?
        .apply_contract(&region.support.node(root)?.contract)
        .map_err(MathError::from)?
        .0
        .require_named()
        .map_err(MathError::from)?;
    projection.outputs.push(output);
    projection.quantities.push(quantity);
    projection.regions.push(region);
    Ok(())
}

fn finish_projection(
    db: &dyn CompilerDb,
    p: &mut Projection,
    registry: &QuantityRegistry,
) -> Result<()> {
    if p.outputs.len() != p.regions.len() || p.outputs.len() != p.quantities.len() {
        return Err(CompileError::Missing(
            "typed projection output extent".into(),
        ));
    }
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
                .get(
                    &pse_modeling::derived::Relationship::ContributionOutput {
                        contribution: *term,
                    }
                    .identity(),
                )
                .is_some_and(|physical| {
                    physical.key == ledger.key && physical.canonical_unit == ledger.canonical_unit
                })
        })
    });
    let constant_bounds = p
        .outputs
        .iter()
        .zip(&p.regions)
        .filter(|(output, region)| {
            matches!(
                output,
                ModelingOutput::Hint {
                    kind: ModelingHint::Lower | ModelingHint::Upper,
                    ..
                }
            ) && region.reads.numeric.is_empty()
                && region.reads.guards.is_empty()
        })
        .map(|(output, _)| output.row_id())
        .collect();
    p.annotation_plans = Some(policy::projection_annotations(db, p, &constant_bounds));
    Ok(())
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
        let projected =
            salsa::Cancelled::catch(|| projection(&self.db, self.inventory, catalog, request))
                .map_err(|_| compiler_cancelled(&self.db))??;
        let working_bytes = projection_body_bytes(&projected);
        self.admit_live_root(projected.outputs.len(), working_bytes)?;
        let result =
            salsa::Cancelled::catch(|| admitted(&self.db, self.inventory, catalog, request))
                .map_err(|_| compiler_cancelled(&self.db))
                .and_then(std::convert::identity);
        if let Ok(product) = &result {
            let bodies = product
                .bodies
                .values()
                .map(|body| {
                    (
                        grouped::memo_key(body),
                        body.descriptor_bytes()
                            .saturating_add(body.math.retained_bytes()),
                    )
                })
                .collect();
            self.retain_live_root(bodies, working_bytes);
        }
        self.trim_queries()?;
        result
    }
}

fn frame_physical_scope(hash: &mut FramedHasher, scope: &PhysicalScope) {
    hash.bool(scope.package.is_some());
    if let Some(package) = &scope.package {
        hash.str(package);
    }
    hash.bool(scope.documents.is_some());
    if let Some(documents) = &scope.documents {
        hash.u64(documents.len() as u64);
        for document in documents {
            hash.id(document);
        }
    }
}
fn complete_context(
    registry: &QuantityRegistry,
    facts: &PhysicalPreconditions,
    scope: &PhysicalScope,
) -> ContentHash {
    let mut hash = FramedHasher::new(pse_ids::Frame::MathPhysicalPremisesV1);
    hash.str("complete-compiler-product")
        .hash(&crate::physical_identity::complete(registry, facts));
    frame_physical_scope(&mut hash, scope);
    hash.finish_hash()
}
fn consumed_context(
    premises: &pse_quantity::premises::PhysicalPremises,
    scope: &PhysicalScope,
) -> ContentHash {
    let mut hash = FramedHasher::new(pse_ids::Frame::MathPhysicalPremisesV1);
    hash.str("compiler-product")
        .hash(&crate::physical_identity::consumed(premises));
    frame_physical_scope(&mut hash, scope);
    hash.finish_hash()
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
    physical_premises: Option<Arc<pse_quantity::premises::PhysicalPremises>>,
    environment: ContentHash,
    providers: BTreeMap<ContentHash, pse_kernels::AdmittedProvider>,
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
            + self
                .physical_premises
                .as_ref()
                .map_or(0, |premises| premises.retained_bytes())
            + self.semantic.descriptor_bytes()
            + self.model.retained_bytes()
            + projection_body_bytes(&self.projection)
            + self.primary.retained_bytes()
            + self.original.as_ref().map_or(0, |(projection, plan)| {
                projection_body_bytes(projection) + plan.retained_bytes()
            })
            + self.providers.len()
                * (size_of::<(ContentHash, pse_kernels::AdmittedProvider)>() + 128)
    }
}

/// Finite kernel preparation and post-specialization structural evidence.
#[derive(Clone, Debug)]
pub struct PreparedModeling {
    context: CompilerContext,
    physical: ContentHash,
    semantic: SemanticModeling,
    projection: Arc<Projection>,
    view_choices: Arc<policy::ViewChoices>,
    occurrences: Arc<BTreeMap<SemanticId, Vec<Occurrence>>>,
    occurrence_identity: Arc<ModelingOccurrenceIdentity>,
    lookup_structure: ContentHash,
    solver_rows: Arc<BTreeSet<SemanticId>>,
    owned_implicit_view: Option<pse_math::SharedAllocation<AdmittedModeling>>,
    original: Option<(
        Arc<Projection>,
        pse_math::SharedAllocation<AdmittedModeling>,
    )>,
    portable: Arc<[Arc<AdmittedBody>]>,
    original_portable: Option<Arc<[Arc<AdmittedBody>]>>,
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
    /// Pure typed numerical instructions for the requested analysis mode.
    /// # Errors
    /// An unpublished/incomplete projection has no annotation product.
    pub fn numerical_annotations(
        &self,
        mode: NumericalAnnotationMode,
    ) -> Result<&NumericalAnnotations> {
        self.projection
            .annotation_plans
            .as_ref()
            .and_then(|plans| plans.get(&mode))
            .map(AsRef::as_ref)
            .ok_or_else(|| CompileError::Missing("compiler numerical annotation product".into()))
    }
    /// Compiler-owned view selection, carrying the source/policy premise that permits it.
    /// # Errors
    /// The retained immutable view inventory is incomplete.
    pub fn compiler_view(&self, policy: ViewPolicy) -> Result<(Self, Arc<ViewChoice>)> {
        let choice = self
            .view_choices
            .get(&(self.original.is_some(), policy))
            .cloned()
            .ok_or_else(|| CompileError::Missing("compiler view policy product".into()))?;
        let model = match choice.view {
            CompilerView::Effective => self.clone(),
            CompilerView::Original => self
                .original_equations()
                .ok_or_else(|| CompileError::Missing("selected original-equation view".into()))?,
        };
        Ok((model, choice))
    }
    /// Resolver-issued consumed physical interpretation of this mathematical product.
    #[doc(hidden)]
    pub fn consumed_physical_identity(&self) -> ContentHash {
        self.physical
    }
    /// Current admitted owners for a fresh receiving numerical shell.
    #[doc(hidden)]
    pub fn compiler_context(&self) -> CompilerContext {
        self.context.clone()
    }
    /// Complete portable body inventory, independent of which pure queries executed.
    /// Primary and original views share one entry per sealed semantic identity; direct
    /// implicit bodies are retained by their systems and have no invented portable key.
    pub fn portable_bodies(&self) -> impl Iterator<Item = &Arc<AdmittedBody>> {
        self.portable.iter()
    }
    /// Original coordinates and equations for eligible selected suppliers. Other
    /// selected operations retain their existing nested guards, regimes and descriptors.
    pub fn original_equations(&self) -> Option<Self> {
        let (projection, admitted) = self.original.as_ref()?;
        let mut model = self.clone();
        model.projection = projection.clone();
        model.admitted = admitted.clone();
        model.lookup_structure = admitted.case.key();
        model.solver_rows = Arc::new(
            admitted
                .outputs
                .iter()
                .filter_map(|o| o.constraint().map(|(id, _)| id))
                .collect(),
        );
        model.owned_implicit_view = None;
        model.original = None;
        model.portable = self.original_portable.as_ref()?.clone();
        model.original_portable = None;
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
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("compiler.preparation_demands", "count", 1);
            context.phase("compiler.plan_modeling")
        });
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        self.db.cancel = cancel;
        let (catalog, request) = self.modeling_request(root, instance, bindings, limits)?;
        let result = salsa::Cancelled::catch(|| {
            let (mut frontier, premises) = pse_quantity::premises::capture(
                &self.inputs.quantities,
                &self.inputs.preconditions,
                || -> Result<_> {
                    let model = pse_quantity::premises::without_recording(|| {
                        observed_specialization(&self.db, catalog, request)
                    })?;
                    let semantic = SemanticModeling::admit(
                        model.clone().into(),
                        self.inventory.quantities(&self.db),
                        self.inventory.preconditions(&self.db),
                    )?;
                    let projection = pse_quantity::premises::without_recording(|| {
                        projection(&self.db, self.inventory, catalog, request)
                    })?;
                    let primary =
                        grouped::plan(&self.db, self.inventory, &projection, BTreeMap::new())?;
                    let promoted: BTreeSet<_> = primary
                        .implicit
                        .values()
                        .filter(|supplier| {
                            supplier.selection.neighborhood_evidence
                                != SelectionNeighborhood::Unestablished
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
                            implicit::project(
                                &self.db,
                                &remaining,
                                &self.inputs.quantities,
                                self.inputs.preconditions.as_ref(),
                                &mut p,
                            )?;
                            finish_projection(&self.db, &mut p, &self.inputs.quantities)?;
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
                    let providers = if primary.implicit.is_empty() {
                        let mut providers = primary.providers();
                        if let Some((_, original)) = &original {
                            providers.extend(original.providers());
                        }
                        providers
                    } else {
                        self.inventory
                            .providers(&self.db)
                            .values()
                            .map(|provider| {
                                (
                                    provider.descriptor.spec().identity(),
                                    provider.descriptor.clone(),
                                )
                            })
                            .collect()
                    };
                    let physical = complete_context(
                        &self.inputs.quantities,
                        &self.inputs.preconditions,
                        &projection.physical_scope,
                    );
                    Ok(ModelingPreparationFrontier {
                        semantic,
                        model,
                        projection,
                        primary,
                        original,
                        physical_premises: None,
                        physical,
                        environment: *self.inventory.environment(&self.db),
                        providers,
                    })
                },
            )?;
            if let Some(premises) = premises {
                frontier.physical =
                    consumed_context(&premises, &frontier.projection.physical_scope);
                frontier.physical_premises = Some(Arc::new(premises));
            } else {
                // Conservative physical routes retain the complete provider interpretation
                // as well, including unused registrations in this current context.
                frontier.providers = self
                    .inputs
                    .providers
                    .values()
                    .map(|provider| {
                        (
                            provider.descriptor.spec().identity(),
                            provider.descriptor.clone(),
                        )
                    })
                    .collect();
            }
            Ok(frontier)
        })
        .map_err(|_| compiler_cancelled(&self.db))
        .and_then(std::convert::identity);
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
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("compiler.completion_demands", "count", 1);
            context.phase("compiler.complete_modeling")
        });
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        self.db.cancel = cancel;
        let live_keys = frontier.body_requests().collect::<BTreeSet<_>>();
        let frontier_bytes = frontier.retained_bytes();
        // These extents belong to the complete owned frontier, independently of
        // resident query history. An oversize incoming root cannot be made to fit
        // by discarding a generation, and must not initiate a rotation.
        if !self.live_root_fits(
            live_keys.len().max(self.limits.query_values),
            frontier_bytes,
        ) {
            return Err(CompileError::Limit("live root working set"));
        }
        let (mut capacity, mut count, mut working) =
            self.live_root_working_set(live_keys.len(), frontier_bytes);
        let mut rotated = false;
        if !self.live_root_fits(count.max(capacity), working) {
            // Only this route holds no generation-local request/catalog handles.
            // Rebuild reconstructs the current admitted revision and immutable
            // inputs; escaped products and attached pure-body retention survive.
            // Preserve the current environment check before a rebuild can refresh
            // that inventory from the initialized library environment.
            if frontier.environment != *self.inventory.environment(&self.db) {
                return Err(CompileError::from(MathError::Contract(
                    "modeling frontier has a different immutable compiler inventory".into(),
                )));
            }
            self.rebuild(self.inputs.clone())?;
            rotated = true;
            (capacity, count, working) =
                self.live_root_working_set(live_keys.len(), frontier_bytes);
        }
        // One receiving generation at most. Required source/context extent and
        // the entire incoming frontier still face the unchanged finite policy.
        if !self.live_root_fits(count.max(capacity), working) {
            return Err(CompileError::Limit("live root working set"));
        }
        self.accept_live_root(capacity, working);
        // Completion consumes immutable owned instructions. Allocation owners may
        // contain interior accounting state, but cancellation cannot expose a partly
        // constructed product; non-cancellation panics are rethrown by Salsa.
        let result = salsa::Cancelled::catch(std::panic::AssertUnwindSafe(|| {
            let (mut product, premises) = pse_quantity::premises::capture(
                &self.inputs.quantities,
                &self.inputs.preconditions,
                || -> Result<_> {
                    checkpoint(&self.db);
                    let providers = self
                        .inventory
                        .providers(&self.db)
                        .values()
                        .map(|provider| {
                            (provider.descriptor.spec().identity(), &provider.descriptor)
                        })
                        .collect::<BTreeMap<_, _>>();
                    let compatible = match &frontier.physical_premises {
                        Some(premises) => {
                            premises.observe(&self.inputs.quantities, &self.inputs.preconditions)
                        }
                        None => {
                            pse_quantity::premises::unsupported();
                            frontier.physical
                                == complete_context(
                                    &self.inputs.quantities,
                                    &self.inputs.preconditions,
                                    &frontier.projection.physical_scope,
                                )
                        }
                    };
                    if !compatible
                        || frontier.environment != *self.inventory.environment(&self.db)
                        || frontier.physical_premises.is_none()
                            && frontier.providers.len() != providers.len()
                        || !frontier.providers.iter().all(|(identity, expected)| {
                            providers
                                .get(identity)
                                .is_some_and(|actual| actual.spec() == expected.spec())
                        })
                    {
                        return Err(CompileError::from(MathError::Contract(
                            "modeling frontier has a different immutable compiler inventory".into(),
                        )));
                    }
                    let admitted = grouped::complete(&self.db, self.inventory, frontier.primary)?;
                    let original: Option<(
                        Arc<Projection>,
                        pse_math::SharedAllocation<AdmittedModeling>,
                    )> = frontier
                        .original
                        .map(|(projection, plan)| -> Result<_> {
                            Ok((
                                projection,
                                grouped::complete(&self.db, self.inventory, plan)?.into(),
                            ))
                        })
                        .transpose()?;
                    for view in std::iter::once(admitted.as_ref())
                        .chain(original.iter().map(|(_, view)| view.as_ref()))
                    {
                        CasePlan::check_physical_bindings(
                            &view.case,
                            &view
                                .bodies
                                .iter()
                                .map(|(key, body)| (*key, body.math.clone()))
                                .collect(),
                            &self.inputs.quantities,
                            self.db.cancel(),
                        )?;
                    }
                    checkpoint(&self.db);
                    let original_portable = original.as_ref().map(
                        |(_, view): &(_, pse_math::SharedAllocation<AdmittedModeling>)| {
                            portable_inventory(view.bodies.values())
                        },
                    );
                    let portable = portable_inventory(
                        admitted
                            .bodies
                            .values()
                            .chain(original.iter().flat_map(|(_, view)| view.bodies.values())),
                    );
                    let occurrences =
                        Arc::new(PreparedModeling::collect_occurrences(&frontier.model));
                    let occurrence_identity = Arc::new(
                        frontier
                            .model
                            .source_occurrences()
                            .iter()
                            .map(|(key, value)| {
                                (
                                    key.declaration,
                                    key.role.clone(),
                                    key.position,
                                    value.source,
                                )
                            })
                            .collect(),
                    );
                    let lookup_structure = admitted.case.key();
                    let solver_rows = Arc::new(
                        admitted
                            .outputs
                            .iter()
                            .filter_map(|o| o.constraint().map(|(id, _)| id))
                            .collect(),
                    );
                    let view_choices = policy::view_choices(
                        &self.db,
                        &admitted,
                        original.as_ref().map(|(_, view)| view.as_ref()),
                        &frontier.model,
                    );
                    Ok(PreparedModeling {
                        context: self.inputs.clone(),
                        physical: complete_context(
                            &self.inputs.quantities,
                            &self.inputs.preconditions,
                            &frontier.projection.physical_scope,
                        ),
                        lookup_structure,
                        solver_rows,
                        occurrences,
                        occurrence_identity,
                        semantic: frontier.semantic,
                        view_choices,
                        projection: frontier.projection,
                        original,
                        portable,
                        original_portable,
                        owned_implicit_view: None,
                        model: frontier.model.into(),
                        admitted: admitted.into(),
                    })
                },
            )?;
            if let Some(premises) = premises {
                product.physical = consumed_context(&premises, &product.projection.physical_scope);
            }
            Ok(product)
        }))
        .map_err(|_| compiler_cancelled(&self.db))
        .and_then(std::convert::identity);
        if let Ok(product) = &result {
            let bodies = product
                .admitted
                .bodies
                .values()
                .chain(
                    product
                        .original
                        .iter()
                        .flat_map(|(_, view)| view.bodies.values()),
                )
                .map(|body| {
                    (
                        grouped::memo_key(body),
                        body.descriptor_bytes()
                            .saturating_add(body.math.retained_bytes()),
                    )
                })
                .collect();
            self.retain_live_root(bodies, frontier_bytes);
        }
        if rotated {
            self.db.trigger_lru_eviction();
            // A cancellation or typed admission refusal remains the actual
            // failure even if its partial query keys exceed retention policy.
            if result.is_ok() && self.retention_exceeded() {
                return Err(CompileError::Limit("live root working set"));
            }
        } else {
            self.trim_queries()?;
        }
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
            + context_extent(&self.context)
            + self.semantic.descriptor_bytes()
            + policy::choices_bytes(&self.view_choices)
            + size_of_val(self.portable.as_ref())
            + 32
            + self
                .original_portable
                .as_ref()
                .map_or(0, |inventory| size_of_val(inventory.as_ref()) + 32)
            + self.model.retained_bytes()
            + self
                .occurrences
                .values()
                .map(|values| 128 + values.capacity() * size_of::<Occurrence>())
                .sum::<usize>()
            + self.solver_rows.len() * 64
            + self
                .occurrence_identity
                .iter()
                .map(|(_, role, _, _)| 128 + role.capacity())
                .sum::<usize>()
            + projection_heap(&Ok(self.projection.clone()))
            + admitted_allocation_bytes(&self.admitted)
            + admitted_owner_attachment_bytes(&self.admitted)
            + self.original.as_ref().map_or(0, |(p, a)| {
                projection_heap(&Ok(p.clone())) + admitted_allocation_bytes(a)
            })
    }
}

fn portable_inventory<'a>(
    bodies: impl Iterator<Item = &'a Arc<AdmittedBody>>,
) -> Arc<[Arc<AdmittedBody>]> {
    bodies
        .filter_map(|body| {
            body.semantic_identity()
                .map(|identity| (identity, body.clone()))
        })
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect::<Vec<_>>()
        .into()
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
        use pse_modeling::derived::Relationship;
        use pse_modeling::resolved::{RegionOutput, RegionRequest};
        let mut p = model.projection.as_ref().clone();
        let members = p
            .outputs
            .iter()
            .enumerate()
            .filter_map(|(index, output)| match output {
                ModelingOutput::Member(id) => {
                    Some((*id, (p.quantities[index], p.declarations[index])))
                }
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let context = TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &self.inputs.quantities,
            preconditions: &self.inputs.preconditions,
            scope: &p.physical_scope,
        };
        let selected = symbols
            .iter()
            .map(|symbol| {
                let (quantity, declaration) = members.get(symbol).copied().ok_or_else(|| {
                    CompileError::Missing("conditional boundary member projection absent".into())
                })?;
                let region = p.model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::ConditionalBoundary(*symbol)],
                        validity: &[],
                    },
                    &context,
                )?;
                let parameter = region
                    .derivations
                    .require(Relationship::ConditionalBoundaryValue { symbol: *symbol })?;
                Ok((*symbol, parameter, quantity, declaration, region))
            })
            .collect::<Result<Vec<_>>>()?;
        p.objectives = Default::default();
        p.conservation.clear();
        p.outputs.clear();
        p.regions.clear();
        p.quantities.clear();
        p.declarations.clear();
        for (symbol, parameter, quantity, declaration, region) in selected {
            p.inputs.push(parameter);
            p.formals.push(Formal {
                path: symbol_name(parameter),
                quantity,
            });
            projection_push_region(&mut p, ModelingOutput::ConditionalBoundary(symbol), region)?;
            p.declarations.push(declaration);
        }
        let admitted = grouped::admit(&self.db, self.inventory, &p)?;
        let plan = Arc::new(CasePlan::prepare_functions(
            &admitted.case,
            |key| admitted.bodies.get(&key).map(|body| body.math.clone()),
            &p.outputs
                .iter()
                .map(ModelingOutput::row_id)
                .collect::<Vec<_>>(),
            coordinates,
            &self.inputs.quantities,
            DerivativeOrder::First,
            profile.assembly,
            cancel,
        )?);
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
        .saturating_add(p.model.retained_bytes())
        .saturating_add(
            p.annotation_plans
                .as_ref()
                .map_or(0, |plans| policy::plans_bytes(plans)),
        )
        .saturating_add(p.annotation_scales.capacity() * size_of::<NumericalScale>())
        .saturating_add(
            p.annotation_scales
                .iter()
                .map(|scale| match &scale.observations {
                    ScalingObservations::Rows(rows) => rows.capacity() * size_of::<SemanticId>(),
                    ScalingObservations::Range(_) => 0,
                })
                .sum::<usize>(),
        )
        .saturating_add(
            p.objectives.members.capacity()
                * (size_of::<pse_modeling::specialize::ObjectiveMember>() + 256),
        )
        .saturating_add(
            p.objectives.levels.capacity()
                * (size_of::<pse_modeling::specialize::ObjectiveLevel>() + 64),
        )
        .saturating_add(
            p.conservation
                .values()
                .map(|terms| 128 + terms.capacity() * size_of::<(SemanticId, f64)>())
                .sum::<usize>(),
        )
        .saturating_add(p.inputs.capacity() * size_of::<SemanticId>())
        .saturating_add((p.free.len() + p.unit_interval.len() + p.nonnegative.len()) * 128)
        .saturating_add(
            p.native
                .iter()
                .map(|constraint| {
                    64 + constraint.identities().len() * (size_of::<SemanticId>() + 16)
                })
                .sum::<usize>(),
        )
        .saturating_add(p.formals.capacity() * size_of::<Formal>())
        .saturating_add(
            p.formals
                .iter()
                .map(|formal| formal.path.capacity())
                .sum::<usize>(),
        )
        .saturating_add(p.outputs.capacity() * size_of::<ModelingOutput>())
        .saturating_add(
            p.outputs
                .iter()
                .map(|output| match output {
                    ModelingOutput::DynamicRate { equations, .. } => {
                        equations.capacity() * size_of::<SemanticId>()
                    }
                    _ => 0,
                })
                .sum::<usize>(),
        )
        .saturating_add(p.regions.capacity() * size_of::<pse_modeling::resolved::SelectedRegion>())
        .saturating_add(
            p.regions
                .iter()
                .map(|region| {
                    region
                        .syntax
                        .retained_bytes()
                        .saturating_add(region.support.retained_bytes())
                        .saturating_add(
                            region.bindings.capacity()
                                * size_of::<pse_modeling::resolved::RegionBinding>(),
                        )
                        .saturating_add(
                            region.inputs.capacity()
                                * size_of::<pse_modeling::resolved::SymbolIx>(),
                        )
                        .saturating_add(
                            region
                                .attribution
                                .nodes
                                .values()
                                .map(|source| {
                                    size_of::<pse_modeling::resolved::Source>()
                                        + source.field.role.capacity()
                                        + 96
                                })
                                .sum::<usize>(),
                        )
                        .saturating_add(
                            region
                                .attribution
                                .predicates
                                .values()
                                .map(|source| {
                                    size_of::<pse_modeling::resolved::Source>()
                                        + source.field.role.capacity()
                                        + 96
                                })
                                .sum::<usize>(),
                        )
                        .saturating_add(
                            (region.reads.numeric.len()
                                + region.reads.guards.len()
                                + region.checked_members.len()
                                + region.checked_sources.len())
                                * 160,
                        )
                        .saturating_add(
                            region
                                .occurrences
                                .values()
                                .map(|source| {
                                    size_of::<pse_model::diagnostic::SourceLocation>()
                                        + 128
                                        + source.path.capacity()
                                        + source.name.as_ref().map_or(0, String::capacity)
                                        + source
                                            .occurrence_role
                                            .as_ref()
                                            .map_or(0, String::capacity)
                                })
                                .sum::<usize>(),
                        )
                        .saturating_add(region.derivations.retained_bytes())
                })
                .sum::<usize>(),
        )
        .saturating_add(
            p.original
                .as_ref()
                .map_or(0, |raw| projection_body_bytes(&raw.projection)),
        )
        .saturating_add(
            p.implicit
                .iter()
                .map(implicit::Projection::retained_bytes)
                .sum::<usize>(),
        )
        .saturating_add(p.declarations.capacity() * size_of::<DeclarationId>())
        .saturating_add(p.quantities.capacity() * size_of::<QuantityTypeId>())
}

fn admitted_allocation_bytes(p: &AdmittedModeling) -> usize {
    size_of::<AdmittedModeling>()
        + p.supplier_topology_bytes
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
                    + (i.checked_members.len() + i.checked_sources.len())
                        * (size_of::<(SemanticId, SemanticId)>() + 96)
                    + i.occurrences
                        .values()
                        .map(|source| {
                            size_of::<pse_model::diagnostic::SourceLocation>()
                                + 128
                                + source.heap_bytes()
                        })
                        .sum::<usize>()
                    + size_of_val(i.slots.as_slice())
                    + size_of_val(i.contributions.as_slice())
            })
            .sum::<usize>()
        + p.case.coordinate_allocation().1
        + p.case.selection_bytes()
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
pub(in crate::workspace) fn configure(db: &mut CompilerDatabase, n: usize) {
    projection::set_lru_capacity(db, n);
    admitted::set_lru_capacity(db, n);
    grouped::configure(db, n);
    policy::configure(db, n);
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
