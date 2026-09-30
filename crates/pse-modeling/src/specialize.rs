// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded finite specialization. No solver, store, query runtime or native library startup.
mod continuous;
mod contextual;
mod physical_operations;
mod envelopes;
mod fold;
mod functions;
mod group;
mod rewrite;
mod transformations;
pub use transformations::{Continuation, Elastic};
pub(crate) mod value;
use crate::{
    CheckedPackage, Declaration, ModelingError, Result, Selected, Type, TypeContext, invalid,
};
pub use group::DispatchBody;
use pse_authoring::dsl::{
    self, BinaryOp, Equation, EquationKind, EquationSense, Expr, ExprKind, Number, Path,
    PathSegment, Span,
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_model::generated::enums::{
    ModelingAccumulatorMode as Mode, ModelingContributionRole as Role,
    ModelingDeclarationKind as Kind, ModelingVariableDomain as Domain,
};
pub use pse_model::generated::identities::{DeclarationId, InstanceId};
use std::collections::{BTreeMap, BTreeSet};
pub use value::{Environment, Value};
mod fixture;
pub use fixture::{
    ExpectedFailure, ExpectedLineage, Fixture, FixtureDiagnostic, FixtureEvent, FixtureMode,
    FixtureValue, IntegrationFixture, ScheduleControl, ScheduleFixture, ShootingFixture,
};
mod regimes;
pub use regimes::{Regime, RegimeSelection};
mod forms;
pub use forms::{DEFAULT_BIG_M_MARGIN, Derived, DerivedRule, Equivalence, Lowering};
mod objectives;
pub use objectives::{ObjectiveBound, ObjectiveLevel, ObjectiveMember, Objectives};
use value::{Evaluator, value_type};

/// Explicit limits checked before expansion/allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Maximum nested definitions.
    pub depth: usize,
    /// Maximum instances, indexed members and generated equations.
    pub items: usize,
    /// Maximum members of one derived finite set/product.
    pub members: usize,
    /// Optional operation/occurrence allowance for each lowered mathematical body.
    /// None uses the mathematical backend's default, independently of mesh extent.
    pub body_occurrences: Option<usize>,
    /// Optional formal-slot allowance (inputs plus stage results) for each lowered
    /// mathematical body; the process-global formal pool extends up to it, and a body
    /// needing more is refused with its required and available slots. None uses the
    /// mathematical backend's default, the initially registered pool chunk.
    pub body_slots: Option<usize>,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            depth: 64,
            items: 100_000,
            members: 100_000,
            body_occurrences: None,
            body_slots: None,
        }
    }
}
/// The realization mechanism is distinct from a registered accelerator reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Realization {
    /// Keep the original equations in the enclosing model.
    Inline,
    /// Use the explicitly supplied generic native root capability.
    Nested,
    /// Use the named registration, which must recognize the residual form.
    Accelerated(String),
}
impl Realization {
    pub(crate) fn from_contract(
        v: &pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueRealization,
        at: DeclarationId,
    ) -> Result<Self> {
        use pse_model::generated::enums::ModelingRealizationPolicy as P;
        match (v.policy, v.accelerator.as_deref()) {
            (P::Inline, None) => Ok(Self::Inline),
            (P::Nested, None) => Ok(Self::Nested),
            (P::Accelerated, Some(id)) if !id.is_empty() && id.len() <= 256 => {
                Ok(Self::Accelerated(id.into()))
            }
            _ => Err(invalid(
                at,
                "accelerated realization requires exactly one bounded capability reference",
            )),
        }
    }
    pub(crate) fn is_nested(&self) -> bool {
        !matches!(self, Self::Inline)
    }
}
/// Root structural bindings and demands. Numeric runtime values have a separate owner.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bindings {
    /// Immutable equation omission and elastic policy, applied after source specialization.
    pub formulation: Formulation,
    /// Explicit root arguments override presets and definition defaults.
    pub arguments: Environment,
    /// Explicitly published ancestor scope values.
    pub scope: Environment,
    /// Reserved compile-time facts: the analysis route, the objective level and the
    /// selected stages (ADR-0123 Outcome 1).
    pub facts: BTreeMap<crate::analysis::Fact, Value>,
    /// Additional member paths requested by the caller.
    pub demand: Vec<String>,
}
/// Generic analysis overlay over selected semantic equations. Science remains in source.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Formulation {
    /// Remove these outer equations from this attempt only.
    pub omitted: BTreeSet<SemanticId>,
    /// Positive canonical physical nominal for each independently relaxed outer row.
    pub elastic: BTreeMap<SemanticId, Value>,
}
/// Instantiated member ownership and the path used for inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lineage {
    /// Authored declaration.
    pub declaration: DeclarationId,
    /// Actual instance.
    pub instance: InstanceId,
    /// Current human-readable path; never an executable identity.
    pub path: String,
    /// Source demand chain, from requesting content to this member.
    pub demand: Vec<DeclarationId>,
    /// Default's originating interface, if inherited.
    pub default_owner: Option<DeclarationId>,
    /// An explicit default override.
    pub is_override: bool,
    /// Ordered presets applied on the path to this definition instance.
    pub presets: Vec<DeclarationId>,
}
/// A generated scalar at the finite mathematics boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    /// Stable instantiated identity.
    pub id: SemanticId,
    /// Complete physical type.
    pub ty: Type,
    /// Declared role: variable, parameter, let or alias.
    pub role: Kind,
    /// Declared decision domain of a variable (ADR-0103); continuous for every other symbol.
    pub domain: Domain,
    /// Definition for a demanded expression member.
    pub expression: Option<Expr>,
    /// Value binding is kept outside the body identity.
    pub initial: Option<Value>,
    /// Source ownership.
    pub lineage: Lineage,
}
/// One admitted finite equation.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// Stable equation identity.
    pub id: SemanticId,
    /// Resolved expression, never rendered/reparsed for lowering.
    pub equation: Equation,
    /// Source attribution and demand explanation.
    pub lineage: Lineage,
}
/// One original contribution retained for independent physical closure.
#[derive(Clone, Debug, PartialEq)]
pub struct Contribution {
    /// Stable contribution identity.
    pub id: SemanticId,
    /// Signed role, independent of the algebraically optimized residual.
    pub role: Role,
    /// Resolved magnitude.
    pub expression: Expr,
    /// Source attribution.
    pub lineage: Lineage,
}
/// Raw accounting inputs, separate from the compiled residual.
#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    /// Accumulator member identity.
    pub id: SemanticId,
    /// Conservation or accounting semantics.
    pub mode: Mode,
    /// Complete physical type.
    pub ty: Type,
    /// Actual energy boundary consumed by directed transfer contributions.
    pub boundary: Option<crate::BoundaryRef>,
    /// Positive declared tolerance in canonical units.
    pub tolerance: Value,
    /// Unoptimized terms.
    pub terms: Vec<Contribution>,
    /// Source attribution.
    pub lineage: Lineage,
}
/// One actual definition instance, sharing executable structure with its dispatch group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    /// Stable identity.
    pub id: InstanceId,
    /// Lexical model owner; hierarchy is never inferred from a display path.
    pub parent: Option<InstanceId>,
    /// Definition identity.
    pub definition: DeclarationId,
    /// Present names mapped to stable member declarations for inspection.
    pub members: BTreeMap<String, DeclarationId>,
    /// Initialization stages from this instance's effective inherited contract.
    pub stages: BTreeSet<String>,
    /// Body specialization identity.
    pub group: ContentHash,
    /// Semantic coordinates bound to the shared body.
    pub coordinates: Vec<SemanticId>,
    /// Semantic rows bound to the shared body.
    pub rows: Vec<SemanticId>,
    /// Human-readable path.
    pub path: String,
}
/// Selected implementations grouped independently of per-instance coefficient values.
#[derive(Clone, Debug, PartialEq)]
pub struct DispatchGroup {
    /// Complete specialization identity.
    pub key: ContentHash,
    /// Reused definition.
    pub definition: DeclarationId,
    /// Shared normalized finite body.
    pub body: std::sync::Arc<DispatchBody>,
    /// Actual instances.
    pub instances: Vec<InstanceId>,
}
/// A checked source test, evaluated by the ordinary math owner.
#[derive(Clone, Debug, PartialEq)]
pub struct Expectation {
    /// Test identity.
    pub id: SemanticId,
    /// Observed expression.
    pub actual: Expr,
    /// Reference expression.
    pub expected: Expr,
    /// Absolute tolerance expression, in the actual expression's type.
    pub tolerance: Expr,
    /// Dimensionless tolerance relative to the expected value.
    pub relative_tolerance: Expr,
    /// Inferred physical type of the actual expression.
    pub ty: Type,
    /// Source of the test.
    pub lineage: Lineage,
}
/// Finite specialization product and its independent inspection/closure views.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpecializedModel {
    /// Alternative residual sets over one implicit block's shared unknowns, keyed by
    /// the block's instance.
    pub regimes: BTreeMap<InstanceId, RegimeSelection>,
    /// Data-authored case fixtures resolved to canonical physical scalar values, keyed by
    /// the fixture's instance.
    pub fixtures: BTreeMap<InstanceId, Fixture>,
    /// Integrated time axes, keyed by their domain member; at most one per model.
    pub integrated: BTreeMap<SemanticId, crate::continuous::IntegratedAxis>,
    /// Time derivatives of differential states, keyed by the state.
    pub derivatives: BTreeMap<SemanticId, crate::continuous::IntegratedDerivative>,
    /// Definite integrals over an integrated axis, keyed by their result.
    pub integrals: BTreeMap<SemanticId, crate::continuous::IntegratedIntegral>,
    /// Original constraints explicitly applied at the integrated domain lower endpoint.
    pub initial_equations: BTreeSet<SemanticId>,
    /// Checked source tests, keyed by test identity.
    pub expectations: BTreeMap<SemanticId, Expectation>,
    /// Explicit caller-requested symbol paths resolved through lexical and indexed semantics.
    pub paths: BTreeMap<String, SemanticId>,
    /// Original rows and normalized penalties of immutable elastic variants.
    pub elastic: BTreeMap<SemanticId, Elastic>,
    /// Explicit physical parameter continuation endpoints, separate from body identity.
    pub continuation: BTreeMap<SemanticId, Continuation>,
    /// Continuous domains realized before indexed members are enumerated.
    pub meshes: BTreeMap<SemanticId, crate::continuous::Mesh>,
    /// Selected implicit policies by block instance; original inline residuals remain
    /// available for inspection.
    pub implicit: BTreeMap<InstanceId, Realization>,
    /// Instantiated variables, values and demanded expression members.
    pub symbols: BTreeMap<SemanticId, Symbol>,
    /// Generated equations.
    pub equations: Vec<Row>,
    /// Raw physical closure plans.
    pub closures: BTreeMap<SemanticId, Closure>,
    /// Explicit local exchanges between distinct instantiated boundaries.
    pub exchanges: BTreeMap<SemanticId, crate::contextual::PairedExchange>,
    /// Definition instances.
    pub instances: BTreeMap<InstanceId, Instance>,
    /// Shared implementation groups.
    pub groups: BTreeMap<ContentHash, DispatchGroup>,
    /// Port identities refer to existing variables, never copies.
    pub ports: BTreeMap<SemanticId, Port>,
    /// Authored incidence limits, independent of scientific port names.
    pub connectivity: BTreeMap<SemanticId, crate::annotation::Connectivity>,
    /// Original directed connection occurrences.
    pub connections: BTreeMap<SemanticId, Connection>,
    /// Selected pure function bodies keyed by resolved semantic call identity.
    pub functions: BTreeMap<String, crate::Function>,
    /// Typed authored annotations and their instantiated owner, and the data envelopes
    /// whose consumer selected extrapolation, observed at the members they guard.
    pub annotations: Vec<crate::annotation::Annotation>,
    /// Data envelopes whose consumer selected extrapolation, observed at static arguments
    /// (ADR-0123 Outcome 4).
    pub observations: Vec<crate::envelope::StaticObservation>,
    /// Named lowerings of constraint forms and disjunctions, inner-first (ADR-0104).
    pub lowerings: Vec<Lowering>,
    /// Constraints left to a backend's native handlers; routing refuses them elsewhere.
    pub native: Vec<pse_model::forms::NativeConstraint>,
    /// Parameters the case box determines at preparation.
    pub derived: BTreeMap<SemanticId, Derived>,
    /// Kernel-derived continuous variables confined to the unit interval.
    pub unit_interval: BTreeSet<SemanticId>,
    /// Authored objective members grouped into lexicographic levels (ADR-0111).
    pub objectives: Objectives,
    /// Kernel-derived continuous variables confined to [0, ∞), such as the slack columns
    /// of a disjunctive complementarity.
    pub nonnegative: BTreeSet<SemanticId>,
    /// Requirements the lowerings place on the solve route (ADR-0104 §5).
    pub requirements: BTreeSet<pse_model::generated::enums::ModelingStructuralRequirement>,
}
/// A declared port aliases an existing physical coordinate without losing its owner.
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    /// Stable instantiated port identity, distinct from its aliased symbol.
    pub id: SemanticId,
    /// Original scalar symbol or expression member.
    pub symbol: SemanticId,
    /// Port declaration and owning instance.
    pub lineage: Lineage,
}
/// Explicit directed topology retained separately from the equality it generates.
#[derive(Clone, Debug, PartialEq)]
pub struct Connection {
    /// Connection occurrence and corresponding generated equality identity.
    pub id: SemanticId,
    /// Source port identity.
    pub from: SemanticId,
    /// Destination port identity.
    pub to: SemanticId,
    /// Original connection declaration.
    pub lineage: Lineage,
}
#[derive(Clone)]
struct State {
    presets: Vec<DeclarationId>,
    definition: DeclarationId,
    parent: Option<InstanceId>,
    env: Environment,
    scope: Environment,
    path: String,
    members: BTreeMap<String, DeclarationId>,
    children: BTreeMap<(String, Vec<SemanticId>), InstanceId>,
    symbols: BTreeMap<(DeclarationId, Vec<SemanticId>), SemanticId>,
    stack: Vec<DeclarationId>,
    /// The data layer's extrapolation policy for this instance and the declaration that
    /// selected it: the nearest instance that selects one decides (ADR-0123 Outcome 4).
    extrapolation: Option<(
        pse_model::generated::enums::ExtrapolationPolicy,
        DeclarationId,
    )>,
}
/// One deferred equation occurrence: its instance, declaration, coordinates and scope.
#[derive(Clone)]
struct ReplicatedEquation {
    instance: InstanceId,
    member: DeclarationId,
    coordinates: Vec<(String, Value)>,
    env: Environment,
}
pub(crate) struct Engine<'a, 'b> {
    pub(crate) p: &'a CheckedPackage,
    pub(crate) c: &'a TypeContext<'b>,
    limits: Limits,
    pub(crate) model: SpecializedModel,
    states: BTreeMap<InstanceId, State>,
    /// Time-independent numerical parameter owner of each temporal descendant.
    temporal_owners: BTreeMap<InstanceId, InstanceId>,
    temporal_axis: Option<SemanticId>,
    stack: Vec<DeclarationId>,
    count: usize,
    lexical: BTreeMap<String, Expr>,
    indexed_arguments: BTreeMap<String, Vec<(Vec<Value>, Expr)>>,
    function_stack: Vec<DeclarationId>,
    function_types: BTreeMap<String, Type>,
    preset_stack: Vec<DeclarationId>,
    local_serial: usize,
    continuity_done: BTreeSet<SemanticId>,
    /// Equation occurrences differentiating along a coordinate at which their instance
    /// was replicated; their stencils read sibling replicas, so they are realized once
    /// every replica exists.
    replicated: Vec<ReplicatedEquation>,
    relaxations: BTreeMap<SemanticId, (Type, Value, Lineage)>,
    /// Form realizations by the instance and form they realize: every instance of a
    /// definition realizes its own occurrences of that definition's forms.
    form_realizations: BTreeMap<(InstanceId, DeclarationId), (forms::Realized, DeclarationId)>,
    facts: Environment,
    objective_level: Option<usize>,
    cancel: &'a dyn Fn() -> bool,
    discretizer: &'a dyn crate::continuous::Discretizer,
    /// The root's reader: a test fixture reads any admitted data; any other root reads no
    /// test-only data (ADR-0123 Outcome 5).
    reader: crate::provenance::Reader<'static>,
    /// Data-layer observations the enclosing function bodies pass on to their call sites,
    /// innermost last (ADR-0123 Outcome 4).
    lifted: Vec<Vec<envelopes::Lift>>,
    /// Dynamic conditional branches enclosing the expression being rewritten.
    branches: usize,
    /// Extrapolating data envelopes observed at members, keyed by member and envelope.
    observed: BTreeMap<(SemanticId, DeclarationId), envelopes::Resolved>,
    /// Extrapolating data envelopes observed at static arguments, keyed by envelope and value.
    observed_static: BTreeMap<(DeclarationId, u64), envelopes::Resolved>,
}

/// The identity a declared analysis gives its root instance: the root declaration's own.
///
/// A root declaration and the instance it becomes are different entities that share one
/// identity value by this rule; nested instances derive their identities from their
/// parent instance and member declaration.
pub const fn root_instance(root: DeclarationId) -> InstanceId {
    InstanceId::from_id(root.as_id())
}

/// Instantiate a checked finite root with explicit structural inputs.
///
/// `root` is the declaration to instantiate and `instance` the identity of the instance
/// it becomes; the two are different entities, so passing one for the other does not
/// compile:
///
/// ```no_run
/// # use pse_modeling::specialize::{Bindings, DeclarationId, InstanceId, Limits, specialize};
/// # fn run(package: &pse_modeling::CheckedPackage, root: DeclarationId, instance: InstanceId) {
/// let _ = specialize(package, root, instance, &Bindings::default(), Limits::default());
/// # }
/// ```
///
/// ```compile_fail
/// # use pse_modeling::specialize::{Bindings, DeclarationId, InstanceId, Limits, specialize};
/// # fn run(package: &pse_modeling::CheckedPackage, root: DeclarationId, instance: InstanceId) {
/// let _ = specialize(package, instance, root, &Bindings::default(), Limits::default());
/// # }
/// ```
/// # Errors
/// Missing bindings, recursion, invalid physical composition, exhausted bounds or a future capability.
pub fn specialize(
    package: &CheckedPackage,
    root: DeclarationId,
    instance: InstanceId,
    bindings: &Bindings,
    limits: Limits,
) -> Result<SpecializedModel> {
    specialize_cancellable(package, root, instance, bindings, limits, &|| false)
}
/// Specialize with cooperative caller cancellation at bounded expansion points.
/// # Errors
/// The same semantic errors as `specialize`, or transient cancellation.
pub fn specialize_cancellable(
    package: &CheckedPackage,
    root: DeclarationId,
    instance: InstanceId,
    bindings: &Bindings,
    limits: Limits,
    cancel: &dyn Fn() -> bool,
) -> Result<SpecializedModel> {
    specialize_with_discretizer(
        package,
        root,
        instance,
        bindings,
        limits,
        cancel,
        &crate::continuous::FiniteDifference,
    )
}
/// Specialize using an admitted numerical library for continuous element construction.
/// # Errors
/// Admission, realization, expansion and cancellation errors are returned before publication.
pub fn specialize_with_discretizer(
    package: &CheckedPackage,
    root: DeclarationId,
    instance: InstanceId,
    bindings: &Bindings,
    limits: Limits,
    cancel: &dyn Fn() -> bool,
    discretizer: &dyn crate::continuous::Discretizer,
) -> Result<SpecializedModel> {
    if bindings
        .arguments
        .keys()
        .any(|k| crate::analysis::Fact::namespace_of(k).is_some())
    {
        return Err(invalid(
            root,
            "analysis, objective and stage selections belong to typed facts, not definition arguments",
        ));
    }
    // The objective level selects a transformation after instantiation; it is not an
    // ambient fact any definition reads.
    let ambient = crate::analysis::facts(&bindings.facts)?;
    let context = package.context();
    let mut engine = Engine {
        p: package,
        c: &context,
        limits,
        model: SpecializedModel::default(),
        states: BTreeMap::new(),
        temporal_owners: BTreeMap::new(),
        temporal_axis: None,
        stack: Vec::new(),
        count: 0,
        lexical: BTreeMap::new(),
        indexed_arguments: BTreeMap::new(),
        function_stack: Vec::new(),
        function_types: BTreeMap::new(),
        preset_stack: Vec::new(),
        local_serial: 0,
        continuity_done: BTreeSet::new(),
        replicated: Vec::new(),
        relaxations: BTreeMap::new(),
        form_realizations: BTreeMap::new(),
        facts: ambient.clone(),
        objective_level: crate::analysis::objective_level(&bindings.facts)?,
        discretizer,
        cancel,
        reader: crate::provenance::Reader::of(package, root),
        lifted: Vec::new(),
        branches: 0,
        observed: BTreeMap::new(),
        observed_static: BTreeMap::new(),
    };
    engine.checkpoint()?;
    let mut args = bindings.arguments.clone();
    args.extend(ambient);
    engine.instantiate(
        root,
        instance,
        None,
        package
            .declarations
            .get(&root)
            .ok_or_else(|| invalid(root, "root absent"))?
            .name
            .clone(),
        args,
        bindings.scope.clone(),
    )?;
    engine.replicated_equations()?;
    for name in &bindings.demand {
        let expr = dsl::parse_expr(name).map_err(|e| invalid(root, e.to_string()))?;
        let resolved = engine.rewrite(instance, &expr, &Environment::new(), &[root])?;
        if let Some(id) = symbol_reference(&resolved) {
            engine.model.paths.insert(name.clone(), id);
        }
    }
    for id in engine.model.regimes.keys() {
        if !engine
            .model
            .implicit
            .get(id)
            .is_some_and(Realization::is_nested)
        {
            return Err(invalid(
                engine.model.instances[id].definition,
                "regime selection requires an owning nested realization",
            ));
        }
    }
    engine.check_connectivity()?;
    engine.publish_observations()?;
    engine.finish(&bindings.formulation)?;
    Ok(engine.model)
}
impl Engine<'_, '_> {
    /// The owner and declaration of an equation member reached through child segments, or
    /// `None` when the path names no equation member.
    fn equation_member(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        prefix: &[PathSegment],
        name: &str,
        env: &Environment,
    ) -> Option<(InstanceId, DeclarationId)> {
        let mut owner = instance;
        for segment in prefix {
            owner = if segment.name == "parent" && segment.indices.is_empty() {
                self.states[&owner].parent?
            } else {
                self.child_instance(owner, at, segment, env).ok()?
            };
        }
        let member = self.states[&owner].members.get(name).copied()?;
        self.p.declarations[&member]
            .value
            .equation
            .is_some()
            .then_some((owner, member))
    }
    pub(crate) fn annotation_targets(
        &mut self,
        instance: InstanceId,
        source: &str,
        env: &Environment,
        at: DeclarationId,
        ports: bool,
    ) -> Result<Vec<(SemanticId, Type, Environment)>> {
        let expression = dsl::parse_expr(source).map_err(|e| invalid(at, e.to_string()))?;
        // One coordinate of an indexed equation member names that equation row.
        if let ExprKind::Path(path) = &expression.kind
            && let Some((last, prefix)) = path.segments.split_last()
            && !last.indices.is_empty()
            && let Some((owner, member)) =
                self.equation_member(instance, at, prefix, &last.name, env)
        {
            let values = last
                .indices
                .iter()
                .map(|index| self.eval(at, env, &dsl::render_expr(index), None))
                .collect::<Result<Vec<_>>>()?;
            let coordinates = self.member_coordinates(owner, member, values)?;
            let row = self.p.declarations[&member].clone();
            let ty = crate::annotation::target_type(
                self.p,
                self.c,
                member,
                &row.name,
                &self.source_types(member, &self.states[&owner].env)?,
            )?;
            let local = coordinates_env(env, &coordinates);
            let equation = row
                .value
                .equation
                .as_ref()
                .ok_or_else(|| invalid(at, "equation payload"))?;
            let equation = dsl::parse_equation(&equation.expression)
                .map_err(|e| invalid(at, e.to_string()))?;
            if !self.equation_defined(&equation, &local)? {
                return Ok(Vec::new());
            }
            return Ok(vec![(member_id(owner, member, &coordinates), ty, local)]);
        }
        let mut owner = instance;
        let mut family = None;
        if let ExprKind::Path(path) = &expression.kind
            && let Some((last, prefix)) = path.segments.split_last()
            && last.indices.is_empty()
        {
            for segment in prefix {
                if segment.name == "parent" && segment.indices.is_empty() {
                    owner = self.states[&owner]
                        .parent
                        .ok_or_else(|| invalid(at, "root has no parent"))?;
                    continue;
                }
                owner = self.child_instance(owner, at, segment, env)?;
            }
            family = self.states[&owner].members.get(&last.name).copied();
        }
        if let Some(member) = family {
            let row = self.p.declarations[&member].clone();
            let indices = if let Some(e) = &row.value.equation {
                e.indices
                    .iter()
                    .map(|i| (i.name.as_str(), i.domain.as_str()))
                    .collect::<Vec<_>>()
            } else if let Some(b) = &row.value.binding {
                b.indices
                    .iter()
                    .map(|i| (i.name.as_str(), i.domain.as_str()))
                    .collect()
            } else if let Some(a) = &row.value.accumulator {
                a.indices
                    .iter()
                    .map(|i| (i.name.as_str(), i.domain.as_str()))
                    .collect()
            } else {
                Vec::new()
            };
            let coordinates = self.coordinates(
                member,
                &self.states[&owner].env.clone(),
                indices.into_iter(),
            )?;
            let ty = crate::annotation::target_type(
                self.p,
                self.c,
                member,
                &row.name,
                &self.source_types(member, &self.states[&owner].env)?,
            )?;
            let mut targets = Vec::new();
            for coordinates in coordinates {
                let local = coordinates_env(env, &coordinates);
                let target = if ports {
                    if row.value.kind != Kind::Port {
                        return Err(invalid(at, "connectivity target must be a declared port"));
                    }
                    member_id(owner, member, &coordinates)
                } else if let Some(e) = &row.value.equation {
                    let equation = dsl::parse_equation(&e.expression)
                        .map_err(|e| invalid(at, e.to_string()))?;
                    if !self.equation_defined(&equation, &local)? {
                        continue;
                    }
                    member_id(owner, member, &coordinates)
                } else {
                    self.symbol(owner, member, &coordinates, &[at])?
                };
                targets.push((target, ty.clone(), local));
            }
            return Ok(targets);
        }
        if ports {
            let ExprKind::Path(path) = &expression.kind else {
                return Err(invalid(at, "connectivity target must name a port"));
            };
            let (owner, member, coordinates) = self.resolve_path(instance, at, path, env, true)?;
            if self.p.declarations[&member].value.kind != Kind::Port {
                return Err(invalid(at, "connectivity target must be a declared port"));
            }
            return Ok(vec![(
                member_id(owner, member, &coordinates),
                self.p.types[&member].clone(),
                env.clone(),
            )]);
        }
        let expression = self.rewrite(instance, &expression, env, &[at])?;
        let target = symbol_reference(&expression)
            .ok_or_else(|| invalid(at, "annotation must target a member or equation"))?;
        Ok(vec![(
            target,
            self.model.symbols[&target].ty.clone(),
            env.clone(),
        )])
    }
    fn checkpoint(&self) -> Result<()> {
        if (self.cancel)() {
            Err(ModelingError::Cancelled)
        } else {
            Ok(())
        }
    }
    pub(crate) fn reserve(&mut self, amount: usize) -> Result<()> {
        self.checkpoint()?;
        self.count = self
            .count
            .checked_add(amount)
            .ok_or_else(|| ModelingError::Budget("item overflow".into()))?;
        if self.count > self.limits.items {
            return Err(ModelingError::Budget(format!(
                "specialized item count: required {}, allowed {}",
                self.count, self.limits.items
            )));
        }
        Ok(())
    }
    pub(crate) fn eval(
        &self,
        at: DeclarationId,
        env: &Environment,
        text: &str,
        expected: Option<&Type>,
    ) -> Result<Value> {
        self.checkpoint()?;
        Evaluator {
            package: self.p,
            physical: self.c,
            at,
            env,
            limit: self.limits.members,
            stack: Vec::new(),
            reader: self.reader,
        }
        .text(text, expected)
    }
    fn predicate(&self, at: DeclarationId, env: &Environment, text: &str) -> Result<bool> {
        let p = dsl::parse_predicate(text).map_err(|e| invalid(at, e.to_string()))?;
        Evaluator {
            package: self.p,
            physical: self.c,
            at,
            env,
            limit: self.limits.members,
            stack: Vec::new(),
            reader: self.reader,
        }
        .predicate(&p)
    }
    pub(crate) fn lineage(
        &self,
        instance: InstanceId,
        row: &Declaration,
        chain: &[DeclarationId],
    ) -> Lineage {
        Lineage {
            declaration: row.declaration_id,
            instance,
            path: format!("{}.{}", self.states[&instance].path, row.name),
            demand: chain.to_vec(),
            default_owner: row
                .parent_id
                .filter(|id| self.p.declarations[id].value.kind == Kind::Interface),
            is_override: row.is_override,
            presets: self.states[&instance].presets.clone(),
        }
    }
    fn instantiate(
        &mut self,
        definition: DeclarationId,
        id: InstanceId,
        parent: Option<InstanceId>,
        path: String,
        arguments: Environment,
        scope: Environment,
    ) -> Result<()> {
        if self.stack.len() >= self.limits.depth {
            return Err(ModelingError::Budget("instance depth".into()));
        }
        if self.stack.contains(&definition) {
            return Err(invalid(definition, "recursive child construction"));
        }
        let row = self
            .p
            .declarations
            .get(&definition)
            .ok_or_else(|| invalid(definition, "missing definition"))?
            .clone();
        if let Some(binding) = &row.value.binding
            && row.value.kind == Kind::Preset
        {
            let expr = binding
                .expression
                .as_ref()
                .ok_or_else(|| invalid(definition, "preset requires application"))?;
            let value = self.eval(definition, &arguments, expr, None)?;
            let Value::Definition {
                id: target,
                mut bindings,
            } = value
            else {
                return Err(invalid(definition, "preset target"));
            };
            bindings.extend(arguments);
            self.stack.push(definition);
            self.preset_stack.push(definition);
            let result = self.instantiate(target, id, parent, path, bindings, scope);
            self.preset_stack.pop();
            self.stack.pop();
            return result;
        }
        let contract = match row
            .value
            .selected()
            .map_err(|e| invalid(definition, e.to_string()))?
        {
            Selected::Definition(contract)
            | Selected::Test(contract)
            | Selected::Case(contract)
            | Selected::Implicit(contract) => contract,
            _ => {
                return Err(invalid(
                    definition,
                    "root/child must be a definition, case or test",
                ));
            }
        };
        self.reserve(1)?;
        self.stack.push(definition);
        let mut env = if row.value.kind == Kind::Implicit {
            parent
                .and_then(|owner| self.states.get(&owner))
                .map(|state| state.env.clone())
                .unwrap_or_default()
        } else {
            Environment::new()
        };
        // Implicit scopes share their enclosing structural context. Their own
        // declarations still shadow ancestor values, including numerical members.
        for name in self
            .p
            .members
            .get(&definition)
            .into_iter()
            .flat_map(|m| m.keys())
        {
            env.remove(name);
        }
        env.extend(self.facts.clone());
        for (n, v) in &scope {
            env.insert(format!("scope.{n}"), v.clone());
        }
        let (_, mut env) = Evaluator {
            package: self.p,
            physical: self.c,
            at: definition,
            env: &env,
            limit: self.limits.members,
            stack: Vec::new(),
            reader: self.reader,
        }
        .definition_environment(definition, &arguments, env.clone())?;
        let mut members = self.p.members.get(&definition).cloned().unwrap_or_default();
        let available_stages = members
            .values()
            .filter(|id| self.p.declarations[id].value.kind == Kind::Stage)
            .map(|id| self.p.declarations[id].name.clone())
            .collect::<BTreeSet<_>>();
        let stages = members
            .values()
            .filter(|id| {
                self.p.declarations[id].value.kind == Kind::Stage
                    && env.get(
                        &crate::analysis::Fact::Stage(self.p.declarations[id].name.clone()).path(),
                    ) == Some(&Value::Boolean(true))
            })
            .count();
        if stages > 1 {
            return Err(invalid(
                definition,
                "multiple initialization stages selected in one definition",
            ));
        }
        let mut conditional = Vec::new();
        for member in members.values().copied().collect::<Vec<_>>() {
            self.active_members(member, &env, &mut conditional)?;
        }
        members.retain(|_, id| {
            !matches!(self.p.declarations[id].value.kind, Kind::When | Kind::Stage)
        });
        for child in conditional {
            let r = &self.p.declarations[&child];
            if members.insert(r.name.clone(), child).is_some() && !r.is_override {
                return Err(invalid(child, "active guarded member collision"));
            }
        }
        self.realize_domains(id, &path, &members, &mut env)?;
        let mut published = scope;
        for member in members.values() {
            let r = &self.p.declarations[member];
            if r.value.kind == Kind::ScopeValue {
                let b = r
                    .value
                    .binding
                    .as_ref()
                    .ok_or_else(|| invalid(*member, "scope payload"))?;
                let source = b
                    .expression
                    .as_ref()
                    .ok_or_else(|| invalid(*member, "scope value"))?;
                published.insert(
                    r.name.clone(),
                    self.eval(*member, &env, source, self.p.types.get(member))?,
                );
            }
        }
        // ADR-0123 Outcome 4: the data layer's policy is the one this definition, test or case
        // selects, else its parent instance's; none selects reject.
        let extrapolation = members
            .values()
            .find_map(|member| {
                self.p.declarations[member]
                    .value
                    .extrapolation
                    .as_ref()
                    .filter(|s| s.layer == pse_model::generated::enums::ModelingValidityLayer::Data)
                    .map(|s| (s.policy, *member))
            })
            .or_else(|| {
                parent
                    .and_then(|owner| self.states.get(&owner))
                    .and_then(|state| state.extrapolation)
            });
        self.states.insert(
            id,
            State {
                presets: self.preset_stack.clone(),
                definition,
                parent,
                env: env.clone(),
                scope: published,
                path: path.clone(),
                members: members.clone(),
                children: BTreeMap::new(),
                symbols: BTreeMap::new(),
                stack: Vec::new(),
                extrapolation,
            },
        );
        self.model.instances.insert(
            id,
            Instance {
                id,
                parent,
                definition,
                members: members.clone(),
                group: ContentHash::from_bytes([0; 32]),
                stages: available_stages,
                coordinates: Vec::new(),
                rows: Vec::new(),
                path,
            },
        );
        // Accumulators exist before descendants can contribute to them.
        for member in members.values() {
            let r = self.p.declarations[member].clone();
            if let Some(a) = &r.value.accumulator {
                for coordinates in self.coordinates(
                    *member,
                    &env,
                    a.indices
                        .iter()
                        .map(|i| (i.name.as_str(), i.domain.as_str())),
                )? {
                    let key = member_id(id, *member, &coordinates);
                    self.reserve(1)?;
                    let ty = self.p.types[member].clone();
                    let tolerance = self.eval(
                        *member,
                        &coordinates_env(&env, &coordinates),
                        &a.tolerance,
                        Some(&ty),
                    )?;
                    if tolerance.scalar(*member)? <= 0.0 {
                        return Err(invalid(*member, "positive closure tolerance required"));
                    }
                    let boundary = a.boundary.as_ref().map(|path| self.resolve_boundary(id, *member, path, &coordinates_env(&env, &coordinates))).transpose()?;
                    self.model.closures.insert(
                        key,
                        Closure {
                            id: key,
                            mode: a.mode,
                            ty,
                            boundary,
                            tolerance,
                            terms: Vec::new(),
                            lineage: self.lineage(id, &r, &[]),
                        },
                    );
                }
            }
        }
        for member in members.values() {
            let r = self.p.declarations[member].clone();
            if matches!(r.value.kind, Kind::Parameter | Kind::Variable)
                && self.p.types.get(member).is_some_and(|ty| ty.quantity_scheme().is_some())
                && r.value
                    .binding
                    .as_ref()
                    .is_some_and(|b| b.defined_by.is_none())
            {
                let b = r
                    .value
                    .binding
                    .as_ref()
                    .ok_or_else(|| invalid(*member, "symbol payload"))?;
                for coordinates in self.coordinates(
                    *member,
                    &env,
                    b.indices
                        .iter()
                        .map(|i| (i.name.as_str(), i.domain.as_str())),
                )? {
                    self.symbol(id, *member, &coordinates, &[])?;
                }
            }
        }
        for member in members.values() {
            let r = self.p.declarations[member].clone();
            if r.value.kind == Kind::Child {
                let b = r
                    .value
                    .binding
                    .as_ref()
                    .ok_or_else(|| invalid(*member, "child payload"))?;
                for coordinates in self.coordinates(
                    *member,
                    &env,
                    b.indices
                        .iter()
                        .filter(|index| {
                            index.name != crate::temporal::COORDINATE
                                || self.temporal_active(id, *member)
                        })
                        .map(|i| (i.name.as_str(), i.domain.as_str())),
                )? {
                    let value = self.eval(
                        *member,
                        &coordinates_env(&env, &coordinates),
                        b.expression
                            .as_deref()
                            .ok_or_else(|| invalid(*member, "child implementation"))?,
                        None,
                    )?;
                    let Value::Definition {
                        id: target,
                        mut bindings,
                    } = value
                    else {
                        return Err(invalid(*member, "child requires a definition"));
                    };
                    let temporal = self
                        .p
                        .temporal
                        .get(member)
                        .filter(|_| self.temporal_active(id, *member));
                    if let Some((policy, _, argument)) = temporal {
                        let point = coordinates
                            .first()
                            .map(|(_, point)| point.clone())
                            .ok_or_else(|| invalid(*policy, "temporal coordinate absent"))?;
                        let (mesh, _) = self.coordinate_mesh(&point)?;
                        let axis = mesh.id;
                        if self.temporal_axis.is_some_and(|selected| selected != axis) {
                            return Err(invalid(
                                *policy,
                                "one analysis-owned temporal axis is required",
                            ));
                        }
                        self.temporal_axis = Some(axis);
                        if bindings.contains_key(argument) {
                            return Err(invalid(
                                *policy,
                                "the analysis owns the instantaneous time argument",
                            ));
                        }
                        let definition = self.p.preset_definition(target)?;
                        let parameter = self.p.declarations[&definition]
                            .value
                            .scope
                            .as_ref()
                            .and_then(|scope| {
                                scope
                                    .parameters
                                    .iter()
                                    .find(|parameter| parameter.name == *argument)
                            })
                            .ok_or_else(|| {
                                invalid(*policy, "temporal constructor argument absent")
                            })?;
                        let expected = self.c.resolve(
                            &parameter.r#type,
                            &BTreeSet::new(),
                            &self.p.named_types(definition),
                            definition,
                        )?;
                        let supplied = Value::Set(vec![point]);
                        if !matches!(expected, Type::Set(_))
                            || !value::conforms(&supplied, &expected, self.p)
                        {
                            return Err(invalid(
                                *policy,
                                "temporal argument must be a set of the axis's physical time quantity",
                            ));
                        }
                        bindings.insert(argument.clone(), supplied);
                    }
                    if let Some(ty) = self.p.types.get(member) {
                        self.compatible(
                            &Value::Definition {
                                id: target,
                                bindings: bindings.clone(),
                            },
                            ty,
                            *member,
                        )?;
                    }
                    let child = InstanceId::from(member_id(id, *member, &coordinates));
                    if temporal.is_some() || self.temporal_owners.contains_key(&id) {
                        let parent = self.temporal_owners.get(&id).copied().unwrap_or(id);
                        let spatial = if temporal.is_some() {
                            &coordinates[1..]
                        } else {
                            &coordinates[..]
                        };
                        self.temporal_owners
                            .insert(child, InstanceId::from(member_id(parent, *member, spatial)));
                    }
                    let ids = coordinates
                        .iter()
                        .map(|(_, v)| v.identity())
                        .collect::<Vec<_>>();
                    self.states
                        .get_mut(&id)
                        .ok_or_else(|| invalid(id, "instance"))?
                        .children
                        .insert((r.name.clone(), ids), child);
                    let scope = self.states[&id].scope.clone();
                    self.instantiate(
                        target,
                        child,
                        Some(id),
                        format!("{}.{}", self.states[&id].path, r.name),
                        bindings,
                        scope,
                    )?;
                }
            }
        }
        let mut realizations = BTreeMap::new();
        for member in members.values() {
            if let Some(policy) = &self.p.declarations[member].value.realization {
                let target = self
                    .p
                    .resolve(*member, &policy.target)
                    .ok_or_else(|| invalid(*member, "implicit target absent"))?;
                if self.p.declarations[&target].value.kind != Kind::Implicit {
                    self.register_form_realization(id, target, *member)?;
                    continue;
                }
                if realizations
                    .insert(target, Realization::from_contract(policy, *member)?)
                    .is_some()
                {
                    return Err(invalid(*member, "competing implicit realization policies"));
                }
            }
        }
        for member in members.values() {
            let row = &self.p.declarations[member];
            if row.value.kind == Kind::Implicit {
                let policy = realizations
                    .get(member)
                    .cloned()
                    .unwrap_or(Realization::Inline);
                let child = InstanceId::from(member_id(id, *member, &[]));
                if let Some(owner) = self.temporal_owners.get(&id).copied() {
                    self.temporal_owners
                        .insert(child, InstanceId::from(member_id(owner, *member, &[])));
                }
                let name = row.name.clone();
                self.states
                    .get_mut(&id)
                    .ok_or_else(|| invalid(id, "implicit parent absent"))?
                    .children
                    .insert((name.clone(), Vec::new()), child);
                let scope = self.states[&id].scope.clone();
                self.instantiate(
                    *member,
                    child,
                    Some(id),
                    format!("{}.{}", self.states[&id].path, name),
                    Environment::new(),
                    scope,
                )?;
                self.model.implicit.insert(child, policy.clone());
                if self.model.regimes.contains_key(&child) && !policy.is_nested() {
                    return Err(invalid(
                        *member,
                        "regime selection requires nested realization; inline selection is not a smooth equation system",
                    ));
                }
            }
        }
        // Child instances now exist. Pair actual endpoints before any equation uses reflection.
        for member in members.values() {
            let row = self.p.declarations[member].clone();
            if let Some(exchange) = &row.value.exchange {
                for coordinates in self.coordinates(*member, &env, exchange.indices.iter().map(|i| (i.name.as_str(), i.domain.as_str())))? {
                    self.reserve(1)?;
                    let local = coordinates_env(&env, &coordinates);
                    let from = self.resolve_boundary(id, *member, &exchange.from, &local)?;
                    let to = self.resolve_boundary(id, *member, &exchange.to, &local)?;
                    let pair = crate::contextual::PairedExchange::admit(*member, from, to)?;
                    self.model.exchanges.insert(member_id(id, *member, &coordinates), pair);
                }
            }
        }
        for member in members.values() {
            let r = self.p.declarations[member].clone();
            match r
                .value
                .selected()
                .map_err(|e| invalid(*member, e.to_string()))?
            {
                Selected::Equation(e) => {
                    for coordinates in self.coordinates(
                        *member,
                        &env,
                        e.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        let equation = dsl::parse_equation(&e.expression)
                            .map_err(|e| invalid(*member, e.to_string()))?;
                        if self.differentiates_replicas(
                            id,
                            &equation,
                            &coordinates_env(&env, &coordinates),
                        ) {
                            self.replicated.push(ReplicatedEquation {
                                instance: id,
                                member: *member,
                                coordinates,
                                env: env.clone(),
                            });
                            continue;
                        }
                        self.equation(id, *member, &equation, &coordinates, &env)?;
                    }
                }
                Selected::Sos1(_) | Selected::Sos2(_) => self.ordered_set(id, &r, &env)?,
                Selected::Atmost(_) | Selected::Atleast(_) | Selected::Exactly(_) => {
                    self.cardinality(id, &r, &env)?;
                }
                Selected::Piecewise(_) => self.piecewise(id, &r, &env)?,
                Selected::Logic(l) => {
                    for coordinates in self.coordinates(
                        *member,
                        &env,
                        l.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        self.logic(id, &r, &coordinates, &env)?;
                    }
                }
                Selected::Complementarity(c) => {
                    for coordinates in self.coordinates(
                        *member,
                        &env,
                        c.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        self.complementarity(id, &r, &coordinates, &env)?;
                    }
                }
                Selected::Disjunction(_) => self.disjunction(id, *member, &env, None)?,
                Selected::Requirement(r) => {
                    if !self.predicate(*member, &env, &r.predicate)? {
                        return Err(invalid(*member, &r.message));
                    }
                }
                Selected::Port(b) => {
                    for coordinates in self.coordinates(
                        *member,
                        &env,
                        b.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        let expr = dsl::parse_expr(
                            b.expression
                                .as_deref()
                                .ok_or_else(|| invalid(*member, "port needs target"))?,
                        )
                        .map_err(|e| invalid(*member, e.to_string()))?;
                        let target = self.rewrite(
                            id,
                            &expr,
                            &coordinates_env(&env, &coordinates),
                            &[*member],
                        )?;
                        let symbol = symbol_reference(&target)
                            .ok_or_else(|| invalid(*member, "port must refer to a symbol"))?;
                        if self.model.symbols[&symbol].ty != self.p.types[member] {
                            return Err(invalid(*member, "port physical contract differs"));
                        }
                        let port = member_id(id, *member, &coordinates);
                        self.model.ports.insert(
                            port,
                            Port {
                                id: port,
                                symbol,
                                lineage: self.lineage(id, &r, &[*member]),
                            },
                        );
                    }
                }
                Selected::Contribution(c) => {
                    for coordinates in self.coordinates(
                        *member,
                        &env,
                        c.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        self.contribution(id, &r, &coordinates)?;
                    }
                }
                Selected::Connection(c) => {
                    let a =
                        dsl::parse_expr(&c.from).map_err(|e| invalid(*member, e.to_string()))?;
                    let b = dsl::parse_expr(&c.to).map_err(|e| invalid(*member, e.to_string()))?;
                    let endpoint = |e: &Expr| -> Result<SemanticId> {
                        let ExprKind::Path(path) = &e.kind else {
                            return Err(invalid(
                                *member,
                                "connection endpoint must name a declared port",
                            ));
                        };
                        let (owner, declaration, coordinates) =
                            self.resolve_path(id, *member, path, &env, true)?;
                        if self.p.declarations[&declaration].value.kind != Kind::Port {
                            return Err(invalid(
                                *member,
                                "connection endpoint must name a declared port",
                            ));
                        }
                        Ok(member_id(owner, declaration, &coordinates))
                    };
                    let from = endpoint(&a)?;
                    let to = endpoint(&b)?;
                    let a = self.rewrite(id, &a, &env, &[*member])?;
                    let b = self.rewrite(id, &b, &env, &[*member])?;
                    let sa = symbol_reference(&a)
                        .ok_or_else(|| invalid(*member, "connection symbol"))?;
                    let sb = symbol_reference(&b)
                        .ok_or_else(|| invalid(*member, "connection symbol"))?;
                    if self.model.symbols[&sa].ty != self.model.symbols[&sb].ty {
                        return Err(invalid(*member, "connection physical types differ"));
                    }
                    let connection = member_id(id, *member, &[]);
                    self.model.connections.insert(
                        connection,
                        Connection {
                            id: connection,
                            from,
                            to,
                            lineage: self.lineage(id, &r, &[*member]),
                        },
                    );
                    self.model.equations.push(Row {
                        id: connection,
                        equation: Equation {
                            kind: EquationKind::Relation {
                                lhs: a,
                                sense: EquationSense::Eq,
                                rhs: b,
                            },
                            span: Span::default(),
                        },
                        lineage: self.lineage(id, &r, &[*member]),
                    });
                }
                Selected::Expectation(test) => {
                    let mut rewrite = |source: &str| -> Result<Expr> {
                        self.rewrite(
                            id,
                            &dsl::parse_expr(source)
                                .map_err(|e| invalid(*member, e.to_string()))?,
                            &env,
                            &[*member],
                        )
                    };
                    let actual = rewrite(&test.actual)?;
                    let expected = rewrite(&test.expected)?;
                    let tolerance = rewrite(&test.tolerance)?;
                    let relative_tolerance =
                        rewrite(test.relative_tolerance.as_deref().unwrap_or("0"))?;
                    let types = self
                        .model
                        .symbols
                        .iter()
                        .map(|(id, s)| (symbol_name(*id), s.ty.clone()))
                        .collect();
                    let ty = crate::expression::infer(
                        &actual,
                        &types,
                        &self.model.function_contracts(self.p),
                        self.c,
                        *member,
                        None,
                    )?;
                    let test_id = member_id(id, *member, &[]);
                    self.reserve(1)?;
                    self.model.expectations.insert(
                        test_id,
                        Expectation {
                            id: test_id,
                            actual,
                            expected,
                            tolerance,
                            relative_tolerance,
                            ty,
                            lineage: self.lineage(id, &r, &[*member]),
                        },
                    );
                }
                Selected::Annotation(_) => self.annotation(id, &r, &env)?,
                Selected::Relaxation(_) | Selected::Continuation(_) => {
                    self.transformation(id, &r, &env)?
                }
                _ => {}
            }
        }
        if let Some(fixture) = &contract.fixture {
            self.fixture(id, &row, fixture, self.p.oracle(row.declaration_id), &env)?;
        }
        self.regimes(id, &row, &members, &env)?;
        self.stack.pop();
        Ok(())
    }
    /// Realize one equation occurrence: its definedness, initial-endpoint role and row.
    fn equation(
        &mut self,
        id: InstanceId,
        member: DeclarationId,
        equation: &Equation,
        coordinates: &[(String, Value)],
        env: &Environment,
    ) -> Result<()> {
        let r = self.p.declarations[&member].clone();
        let Selected::Equation(e) = r
            .value
            .selected()
            .map_err(|e| invalid(member, e.to_string()))?
        else {
            return Err(invalid(member, "equation payload"));
        };
        let local = coordinates_env(env, coordinates);
        if !self.equation_defined(equation, &local)? {
            return Ok(());
        }
        if self.initial_equation(id, member, equation, &local)? {
            self.model
                .initial_equations
                .insert(member_id(id, member, coordinates));
        }
        let equation = self.rewrite_equation(id, equation, &local, &[member])?;
        if e.condition.is_some() {
            return self.indicator_equation(id, &r, coordinates, equation, env);
        }
        self.reserve(1)?;
        self.model.equations.push(Row {
            id: member_id(id, member, coordinates),
            equation,
            lineage: self.lineage(id, &r, &[member]),
        });
        Ok(())
    }
    /// Realize the equations deferred until every replica of their instances exists.
    fn replicated_equations(&mut self) -> Result<()> {
        for occurrence in std::mem::take(&mut self.replicated) {
            let Selected::Equation(e) = self.p.declarations[&occurrence.member]
                .value
                .selected()
                .map_err(|e| invalid(occurrence.member, e.to_string()))?
            else {
                return Err(invalid(occurrence.member, "equation payload"));
            };
            let equation = dsl::parse_equation(&e.expression)
                .map_err(|e| invalid(occurrence.member, e.to_string()))?;
            self.equation(
                occurrence.instance,
                occurrence.member,
                &equation,
                &occurrence.coordinates,
                &occurrence.env,
            )?;
        }
        Ok(())
    }
    fn active_members(
        &self,
        id: DeclarationId,
        env: &Environment,
        out: &mut Vec<DeclarationId>,
    ) -> Result<()> {
        let row = &self.p.declarations[&id];
        if row.value.guard.is_some() || row.value.kind == Kind::Stage {
            let active = if let Some(guard) = &row.value.guard {
                self.predicate(id, env, &guard.predicate)?
            } else {
                env.get(&crate::analysis::Fact::Stage(row.name.clone()).path())
                    == Some(&Value::Boolean(true))
            };
            if active {
                for child in self.p.children.get(&id).into_iter().flatten() {
                    if self.p.declarations[child].value.kind == Kind::When {
                        self.active_members(*child, env, out)?;
                    } else {
                        out.push(*child);
                    }
                }
            }
        }
        Ok(())
    }
    fn compatible(&self, value: &Value, ty: &Type, at: DeclarationId) -> Result<()> {
        if value::conforms(value, ty, self.p) {
            Ok(())
        } else {
            Err(invalid(at, "binding complete type differs"))
        }
    }

    fn coordinates<'a>(
        &self,
        at: DeclarationId,
        env: &Environment,
        indices: impl Iterator<Item = (&'a str, &'a str)>,
    ) -> Result<Vec<Vec<(String, Value)>>> {
        let mut output = vec![Vec::new()];
        for (name, domain) in indices {
            let mut next = Vec::new();
            for row in output {
                self.checkpoint()?;
                let local = coordinates_env(env, &row);
                let Value::Set(values) = self.eval(at, &local, domain, None)? else {
                    return Err(invalid(at, "finite domain required"));
                };
                if next
                    .len()
                    .checked_add(values.len())
                    .is_none_or(|n| n > self.limits.members)
                {
                    return Err(ModelingError::Budget("indexed extent".into()));
                }
                for value in values {
                    let mut row = row.clone();
                    row.push((name.into(), value));
                    next.push(row);
                }
            }
            output = next;
        }
        Ok(output)
    }
    fn symbol(
        &mut self,
        instance: InstanceId,
        member: DeclarationId,
        coordinates: &[(String, Value)],
        chain: &[DeclarationId],
    ) -> Result<SemanticId> {
        // A member definition owns a lexical scope. A demand cannot capture locals of its caller.
        let saved = std::mem::take(&mut self.lexical);
        let indexed = std::mem::take(&mut self.indexed_arguments);
        let types = std::mem::take(&mut self.function_types);
        let result = self.symbol_member(instance, member, coordinates, chain);
        self.lexical = saved;
        self.indexed_arguments = indexed;
        self.function_types = types;
        result
    }
    fn symbol_member(
        &mut self,
        instance: InstanceId,
        member: DeclarationId,
        coordinates: &[(String, Value)],
        chain: &[DeclarationId],
    ) -> Result<SemanticId> {
        let row = self.p.declarations[&member].clone();
        let key = (
            member,
            coordinates
                .iter()
                .map(|(_, v)| v.identity())
                .collect::<Vec<_>>(),
        );
        if let Some(id) = self.states[&instance].symbols.get(&key) {
            if self.states[&instance].stack.contains(&member)
                && matches!(row.value.kind, Kind::Let | Kind::Alias)
            {
                return Err(invalid(member, "recursive lazy alias/let demand"));
            }
            return Ok(*id);
        }
        if let Some(accumulator) = &row.value.accumulator {
            if accumulator.mode != Mode::Accounting {
                return Err(invalid(
                    member,
                    "a conservation accumulator is an equation, not an accounting value",
                ));
            }
            let id = member_id(instance, member, coordinates);
            let closure = self
                .model
                .closures
                .get(&id)
                .ok_or_else(|| invalid(member, "accounting coordinate absent"))?;
            self.model.symbols.entry(id).or_insert_with(|| Symbol {
                id,
                ty: closure.ty.clone(),
                role: Kind::Let,
                domain: Domain::Continuous,
                expression: None,
                initial: None,
                lineage: closure.lineage.clone(),
            });
            self.states
                .get_mut(&instance)
                .ok_or_else(|| invalid(member, "accounting owner absent"))?
                .symbols
                .insert(key, id);
            return Ok(id);
        }
        if row.value.kind == Kind::Alternative {
            // ADR-0104: an alternative is selected by its own binary indicator.
            self.reserve(1)?;
            let id = member_id(instance, member, coordinates);
            let ty = self
                .p
                .types
                .get(&member)
                .cloned()
                .ok_or_else(|| invalid(member, "alternative indicator type absent"))?;
            let zero = self.typed_zero(&ty, member)?;
            let mut lineage = self.lineage(instance, &row, &[member]);
            // Name the indicator by its disjunction path, as authors reference it.
            let mut names = Vec::new();
            let mut cursor = Some(member);
            while let Some(id) = cursor.filter(|id| {
                matches!(
                    self.p.declarations[id].value.kind,
                    Kind::Alternative | Kind::Disjunction
                )
            }) {
                names.push(self.p.declarations[&id].name.clone());
                cursor = self.p.declarations[&id].parent_id;
            }
            names.reverse();
            lineage.path = format!("{}.{}", self.states[&instance].path, names.join("."));
            self.model.symbols.insert(
                id,
                Symbol {
                    id,
                    ty,
                    role: Kind::Variable,
                    domain: Domain::Binary,
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
            self.states
                .get_mut(&instance)
                .ok_or_else(|| invalid(instance, "instance missing"))?
                .symbols
                .insert(key, id);
            return Ok(id);
        }
        let b = row
            .value
            .binding
            .as_ref()
            .ok_or_else(|| invalid(member, "demand target is not a symbol"))?;
        if row.value.kind == Kind::Port {
            let expr = dsl::parse_expr(
                b.expression
                    .as_deref()
                    .ok_or_else(|| invalid(member, "port target"))?,
            )
            .map_err(|e| invalid(member, e.to_string()))?;
            let env = coordinates_env(&self.states[&instance].env, coordinates);
            let target = self.rewrite(instance, &expr, &env, chain)?;
            return symbol_reference(&target)
                .ok_or_else(|| invalid(member, "port target must name a symbol"));
        }
        let owner = if row.value.kind == Kind::Parameter && b.indices.is_empty() {
            self.temporal_owners
                .get(&instance)
                .copied()
                .unwrap_or(instance)
        } else {
            instance
        };
        let id = member_id(owner, member, coordinates);
        let ty = self
            .p
            .types
            .get(&member)
            .cloned()
            .ok_or_else(|| invalid(member, "demanded member type absent"))?;
        let env = coordinates_env(&self.states[&instance].env, coordinates);
        let ty = self.bind_physical_owner(&ty, owner, &env, member)?;
        if owner != instance
            && let Some(existing) = self.model.symbols.get(&id)
        {
            let initial = b
                .expression
                .as_ref()
                .map(|source| self.eval(member, &env, source, Some(&ty)))
                .transpose()?;
            if existing.ty != ty || existing.initial != initial {
                return Err(invalid(
                    member,
                    "scalar temporal parameter defaults must agree; time changes belong to an analysis schedule",
                ));
            }
            self.states
                .get_mut(&instance)
                .ok_or_else(|| invalid(instance, "temporal instance absent"))?
                .symbols
                .insert(key, id);
            return Ok(id);
        }
        self.reserve(1)?;
        let mut demand = chain.to_vec();
        demand.push(member);
        // ADR-0103: a variable carries its declared domain; every other symbol is continuous.
        let domain = if row.value.kind == Kind::Variable {
            b.domain
                .ok_or_else(|| invalid(member, "variable declares no domain"))?
        } else {
            Domain::Continuous
        };
        self.model.symbols.insert(
            id,
            Symbol {
                id,
                ty: ty.clone(),
                role: row.value.kind,
                domain,
                expression: None,
                initial: None,
                lineage: self.lineage(instance, &row, &demand),
            },
        );
        let state = self
            .states
            .get_mut(&instance)
            .ok_or_else(|| invalid(instance, "instance missing"))?;
        state.symbols.insert(key, id);
        state.stack.push(member);
        if let Some(source) = &b.expression {
            if row.value.kind == Kind::Parameter {
                let value = self.eval(member, &env, source, Some(&ty))?;
                self.compatible(&value, &ty, member)?;
                self.model
                    .symbols
                    .get_mut(&id)
                    .ok_or_else(|| invalid(id, "symbol missing"))?
                    .initial = Some(value);
            } else if matches!(row.value.kind, Kind::Let | Kind::Alias) {
                let expr = dsl::parse_expr(source).map_err(|e| invalid(member, e.to_string()))?;
                let expr = self.rewrite(instance, &expr, &env, &demand)?;
                self.model
                    .symbols
                    .get_mut(&id)
                    .ok_or_else(|| invalid(id, "symbol missing"))?
                    .expression = Some(expr);
            }
        }
        if let Some(source) = &b.defined_by {
            let equation =
                dsl::parse_equation(source).map_err(|e| invalid(member, e.to_string()))?;
            let equation = self.rewrite_equation(instance, &equation, &env, &demand)?;
            self.model.equations.push(Row {
                id: pse_ids::named_id(id, "defining-equation"),
                equation,
                lineage: self.lineage(instance, &row, &demand),
            });
        }
        self.states
            .get_mut(&instance)
            .ok_or_else(|| invalid(instance, "instance missing"))?
            .stack
            .pop();
        Ok(id)
    }
    fn contribution(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        coordinates: &[(String, Value)],
    ) -> Result<()> {
        let id = row.declaration_id;
        let c = row
            .value
            .contribution
            .as_ref()
            .ok_or_else(|| invalid(id, "contribution payload"))?;
        let env = coordinates_env(&self.states[&instance].env, coordinates);
        let path = dsl::parse_expr(&c.target).map_err(|e| invalid(id, e.to_string()))?;
        let ExprKind::Path(path) = path.kind else {
            return Err(invalid(id, "accumulator path required"));
        };
        let (owner, member, coords) = self.resolve_path(instance, id, &path, &env, true)?;
        let target = member_id(owner, member, &coords);
        let ty = self
            .model
            .closures
            .get(&target)
            .ok_or_else(|| invalid(id, "accumulator target absent"))?
            .ty
            .clone();
        let expression = dsl::parse_expr(&c.expression).map_err(|e| invalid(id, e.to_string()))?;
        let mut expression = self.rewrite(instance, &expression, &env, &[id])?;
        let types = self
            .model
            .symbols
            .iter()
            .map(|(id, s)| (symbol_name(*id), s.ty.clone()))
            .collect::<BTreeMap<_, _>>();
        let actual = crate::expression::infer(
            &expression,
            &types,
            &self.model.function_contracts(self.p),
            self.c,
            id,
            if c.role == Role::Directed {None}else{Some(&ty)},
        )?;
        if c.role == Role::Directed {
            let boundary=self.model.closures[&target].boundary.as_ref().ok_or_else(||invalid(id,"directed contribution requires an explicit accumulator boundary"))?;
            let refinement=actual.physical_refinement().ok_or_else(||invalid(id,"directed contribution requires a transfer"))?;
            refinement.contribution(boundary,false,id)?;
            let payload=Type::Quantity(actual.quantity_scheme().ok_or_else(||invalid(id,"transfer requires physical payload"))?.clone());
            let contracts=[&ty,&payload].iter().map(|value|value.quantity_scheme().ok_or_else(||invalid(id,"ledger requires physical payload"))?.resolve_contract_with_evidence(self.c.quantities,&BTreeMap::new(),self.c.preconditions).map_err(|error|invalid(id,error.to_string()))).collect::<Result<Vec<_>>>()?;
            let expected=contracts[0].require_named().map_err(|error|invalid(id,error.to_string()))?;
            pse_quantity::resolved::infer_operation(&pse_quantity::infer::OpRequest::Add,&contracts,Some(expected),self.c.quantities,self.c.preconditions).map_err(|error|invalid(id,format!("directed transfer does not belong in this physical ledger: {error}")))?;
            expression=self.physical_transfer_function(id,actual.clone(),payload,crate::PhysicalOperation::TransferMagnitude {source:refinement.clone()},1,expression)?;
        } else if actual.physical_refinement().is_some() {
            return Err(invalid(id,"a directed transfer must be consumed with role directed exactly once"));
        } else if actual != ty {
            return Err(invalid(id, "contribution physical contract differs"));
        }
        let term = Contribution {
            id: member_id(instance, id, coordinates),
            role: c.role,
            expression,
            lineage: self.lineage(instance, row, &[id]),
        };
        self.model
            .closures
            .get_mut(&target)
            .ok_or_else(|| invalid(id, "accumulator missing"))?
            .terms
            .push(term);
        Ok(())
    }
    fn finish(&mut self, formulation: &Formulation) -> Result<()> {
        for closure in self.model.closures.values() {
            if closure.mode == Mode::Conservation && closure.terms.is_empty() {
                return Err(invalid(
                    closure.lineage.declaration,
                    "conservation member has no contributions",
                ));
            }
            let mut ids = BTreeSet::new();
            for term in &closure.terms {
                if !ids.insert(term.id) {
                    return Err(invalid(term.lineage.declaration, "duplicate contribution"));
                }
            }
        }
        for closure in self.model.closures.values() {
            let zero = self.typed_zero(&closure.ty, closure.lineage.declaration)?;
            let mut sum = zero.clone();
            for term in &closure.terms {
                let negative = term.sign() < 0.0;
                sum = Expr {
                    kind: ExprKind::Binary {
                        op: if negative {
                            BinaryOp::Sub
                        } else {
                            BinaryOp::Add
                        },
                        lhs: Box::new(sum),
                        rhs: Box::new(term.expression.clone()),
                    },
                    span: Span::default(),
                };
            }
            if closure.mode == Mode::Conservation {
                self.model.equations.push(Row {
                    id: pse_ids::named_id(closure.id, "conservation"),
                    equation: Equation {
                        kind: EquationKind::Relation {
                            lhs: sum,
                            sense: EquationSense::Eq,
                            rhs: zero,
                        },
                        span: Span::default(),
                    },
                    lineage: closure.lineage.clone(),
                });
            } else {
                self.model.symbols.insert(
                    closure.id,
                    Symbol {
                        id: closure.id,
                        ty: closure.ty.clone(),
                        role: Kind::Let,
                        domain: Domain::Continuous,
                        expression: Some(sum),
                        initial: None,
                        lineage: closure.lineage.clone(),
                    },
                );
            }
        }
        self.validate_contextual_equations()?;
        self.select_formulation(formulation)?;
        self.apply_relaxations()?;
        self.admit_objectives()?;
        self.group_bodies()?;
        self.model.equations.sort_by_key(|r| r.id);
        Ok(())
    }
    fn typed_zero(&self, ty: &Type, at: DeclarationId) -> Result<Expr> {
        let s=ty.quantity_scheme().ok_or_else(||invalid(at,"accumulator requires quantity"))?;
        let id = s
            .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
            .map_err(|e| invalid(at, e.to_string()))?;
        let q = self
            .c
            .quantities
            .quantity_type(id)
            .map_err(|e| invalid(at, e.to_string()))?;
        let unit = self
            .c
            .quantities
            .unit_product(q.canonical_unit)
            .map_err(|e| invalid(at, e.to_string()))?;
        Ok(Expr {
            kind: ExprKind::Number(Number {
                exact_integer: None,
                value: 0.0,
                unit: Some(unit),
            }),
            span: Span::default(),
        })
    }
}
fn member_id(
    instance: InstanceId,
    declaration: DeclarationId,
    coordinates: &[(String, Value)],
) -> SemanticId {
    let mut h = FramedHasher::new(pse_ids::Frame::ModelingMemberV1);
    h.id(&instance.as_id()).id(&declaration.as_id());
    for (_, value) in coordinates {
        h.id(&value.identity());
    }
    h.finish_id()
}
fn coordinates_env(env: &Environment, coordinates: &[(String, Value)]) -> Environment {
    let mut env = env.clone();
    env.extend(coordinates.iter().cloned());
    env
}
/// Stable private formal name; public diagnostics use lineage instead.
pub fn symbol_name(id: SemanticId) -> String {
    format!("s_{}", id.to_hex())
}
fn symbol_expr(id: SemanticId) -> Expr {
    Expr {
        kind: ExprKind::Path(Path {
            segments: vec![PathSegment {
                name: symbol_name(id),
                indices: Vec::new(),
            }],
        }),
        span: Span::default(),
    }
}
pub(crate) fn symbol_reference(expr: &Expr) -> Option<SemanticId> {
    let ExprKind::Path(p) = &expr.kind else {
        return None;
    };
    if p.segments.len() != 1 {
        return None;
    }
    SemanticId::parse_hex(p.segments[0].name.strip_prefix("s_")?).ok()
}

/// Independent assessment from original contribution magnitudes in canonical units.
#[derive(Clone, Debug, PartialEq)]
pub struct ClosureAssessment {
    /// Original accumulator identity.
    pub accumulator: SemanticId,
    /// Sum of original signed magnitudes, independently of the equation residual.
    pub net: f64,
    /// Authored absolute tolerance in the accumulator's canonical unit.
    pub tolerance: f64,
    /// Conservation closure; accounting totals have no zero-closure obligation.
    pub satisfied: Option<bool>,
}
impl Contribution {
    /// Sign prescribed by the authored role; directed transfers already consume Into.
    pub fn sign(&self) -> f64 {
        if matches!(
            self.role,
            Role::Outflow | Role::Consumption | Role::Accumulation | Role::Negative
        )
        {
            -1.0
        } else {
            1.0
        }
    }
}
impl SpecializedModel {
    /// The declared integral a member names: the integral itself, or the one reached by
    /// following exact symbol aliases. A sum or scaled expression names none.
    pub fn integral_of(&self, member: SemanticId) -> Option<SemanticId> {
        let mut target = member;
        let mut visited = BTreeSet::new();
        while !self.integrals.contains_key(&target) {
            if !visited.insert(target) {
                return None;
            }
            target = self
                .symbols
                .get(&target)
                .and_then(|symbol| symbol.expression.as_ref())
                .and_then(symbol_reference)?;
        }
        Some(target)
    }
    /// The typed refusal of a derived parameter's realization, naming its subject (ADR-0104).
    pub fn realization_refusal(
        &self,
        parameter: SemanticId,
        subject: SemanticId,
        reason: crate::RealizationRefusal,
    ) -> ModelingError {
        let derived = self.derived.get(&parameter);
        let source = derived.map_or(parameter, |d| d.source.as_id());
        let path = |id: &SemanticId| {
            self.symbols
                .get(id)
                .map(|s| s.lineage.path.clone())
                .or_else(|| {
                    self.lowerings
                        .iter()
                        .find(|l| l.source.as_id() == *id)
                        .and_then(|l| l.rows.first())
                        .and_then(|row| self.equations.iter().find(|r| r.id == *row))
                        .map(|r| r.lineage.path.clone())
                })
                .unwrap_or_else(|| id.to_string())
        };
        ModelingError::Realization {
            declaration: source,
            form: self
                .symbols
                .get(&parameter)
                .map(|s| {
                    s.lineage
                        .path
                        .rsplit_once('.')
                        .map_or(s.lineage.path.clone(), |(head, _)| head.to_owned())
                })
                .unwrap_or_else(|| path(&source)),
            subject: path(&subject),
            realization: derived.map_or(
                pse_model::generated::enums::ModelingRealizationPolicy::DerivedBigM,
                |d| d.realization,
            ),
            reason,
        }
    }
    /// The typed refusal of a variable's declared domain, naming the variable (ADR-0103).
    pub fn domain_refusal(
        &self,
        variable: SemanticId,
        analysis: crate::DomainAnalysis,
        reason: crate::DomainRefusal,
    ) -> ModelingError {
        let symbol = self.symbols.get(&variable);
        ModelingError::Domain {
            variable,
            declaration: symbol.map_or(variable, |s| s.lineage.declaration.as_id()),
            path: symbol.map_or_else(|| variable.to_string(), |s| s.lineage.path.clone()),
            domain: symbol.map_or(Domain::Continuous, |s| s.domain),
            analysis,
            reason,
        }
    }
    /// The record of an inward bound tightening of a variable, naming it (ADR-0103 item 4).
    pub fn domain_tightening(
        &self,
        variable: SemanticId,
        specified: [f64; 2],
        tightened: [f64; 2],
    ) -> crate::DomainTightening {
        let symbol = self.symbols.get(&variable);
        crate::DomainTightening {
            variable,
            declaration: symbol.map_or(variable, |s| s.lineage.declaration.as_id()),
            path: symbol.map_or_else(|| variable.to_string(), |s| s.lineage.path.clone()),
            domain: symbol.map_or(Domain::Continuous, |s| s.domain),
            specified,
            tightened,
        }
    }
    /// An analysis that cannot decide discrete variables admits them only when the case
    /// fixes them (ADR-0103 item 6). Refuses the first free discrete variable.
    /// # Errors
    /// [`ModelingError::Domain`] naming the variable and the analysis.
    pub fn require_fixed_discrete(
        &self,
        free: impl IntoIterator<Item = SemanticId>,
        analysis: crate::DomainAnalysis,
    ) -> Result<()> {
        match free
            .into_iter()
            .find(|id| self.symbols.get(id).is_some_and(|s| s.domain.is_discrete()))
        {
            Some(id) => Err(self.domain_refusal(id, analysis, crate::DomainRefusal::Free)),
            None => Ok(()),
        }
    }
    /// Assess physical closure from evaluated original terms, never optimized residual values.
    /// # Errors
    /// Missing or nonfinite original magnitudes, an invalid tolerance or a nonfinite sum.
    pub fn assess_closure(
        &self,
        magnitudes: &BTreeMap<SemanticId, f64>,
    ) -> Result<Vec<ClosureAssessment>> {
        self.assess_closures(magnitudes, &self.closures.keys().copied().collect())
    }
    /// Assess exactly the declared accounting subjects selected by a consumer.
    pub fn assess_closures(
        &self,
        magnitudes: &BTreeMap<SemanticId, f64>,
        selected: &BTreeSet<SemanticId>,
    ) -> Result<Vec<ClosureAssessment>> {
        if selected.iter().any(|id| !self.closures.contains_key(id)) {
            return Err(invalid(SemanticId::NIL, "unknown selected closure"));
        }
        self.closures
            .values()
            .filter(|closure| selected.contains(&closure.id))
            .map(|closure| {
                let mut net = 0.0;
                for term in &closure.terms {
                    let magnitude = magnitudes
                        .get(&term.id)
                        .copied()
                        .filter(|v| v.is_finite())
                        .ok_or_else(|| {
                            invalid(
                                term.lineage.declaration,
                                "missing or nonfinite original closure magnitude",
                            )
                        })?;
                    net += term.sign() * magnitude;
                }
                if !net.is_finite() {
                    return Err(invalid(
                        closure.lineage.declaration,
                        "nonfinite closure total",
                    ));
                }
                let tolerance = closure.tolerance.scalar(closure.lineage.declaration)?;
                if !tolerance.is_finite() || tolerance <= 0.0 {
                    return Err(invalid(
                        closure.lineage.declaration,
                        "positive finite closure tolerance required",
                    ));
                }
                Ok(ClosureAssessment {
                    accumulator: closure.id,
                    net,
                    tolerance,
                    satisfied: (closure.mode == Mode::Conservation)
                        .then_some(net.abs() <= tolerance),
                })
            })
            .collect()
    }
}
