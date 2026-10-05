// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded finite specialization. No solver, store, query runtime or native library startup.
mod applicability;
mod contextual;
mod continuous;
mod envelopes;
mod fold;
mod functions;
mod group;
mod physical_operations;
mod process;
pub use process::{
    InventoryBalance, InventoryInitialCondition, MaterialPort, StateKey, StateSpecification,
};
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
    ExpectedFailure, ExpectedLineage, Fixture, FixtureDiagnostic, FixtureEndpoint, FixtureEvent,
    FixtureMode, FixtureValue, IntegrationFixture, ScheduleControl, ScheduleFixture,
    ShootingFixture,
};
mod regimes;
pub use regimes::{Regime, RegimeSelection, RootSelection};
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
    /// Independent physical obligation, without an additional solved equation.
    pub observation_only: bool,
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
impl Closure {
    /// Conservation and observation require an independent physical check; accounting
    /// records a signed total without asserting that the total is zero.
    pub fn requires_check(&self) -> bool {
        self.mode != Mode::Accounting
    }
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
    /// Selected checked source fields belong to this revision, independently of shared mathematics.
    source_occurrences: std::sync::Arc<
        BTreeMap<
            crate::expression::occurrences::OccurrenceKey,
            crate::expression::occurrences::CheckedExpression,
        >,
    >,
    /// Explicit function selectors for single-residual implicit occurrences.
    pub root_selections: BTreeMap<InstanceId, RootSelection>,
    /// Immutable selected records and authored closure edges in this model scope.
    pub selection_closures: crate::scientific_selection::Selections,
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
    /// Admitted independent state and transport contracts.
    pub state_specifications: BTreeMap<SemanticId, StateSpecification>,
    /// Material boundary occurrences, each aliasing one state specification.
    pub material_ports: BTreeMap<SemanticId, MaterialPort>,
    /// Original inventory/flux/event-transfer conservation meaning.
    pub inventory_balances: BTreeMap<SemanticId, InventoryBalance>,
    /// Retired coordinate initial rows remain required original consistency observations.
    pub inventory_initial_conditions: BTreeMap<SemanticId, InventoryInitialCondition>,
    /// Original directed connection occurrences.
    pub connections: BTreeMap<SemanticId, Connection>,
    /// Selected pure function bodies keyed by resolved semantic call identity.
    pub functions: BTreeMap<String, crate::Function>,
    /// Typed authored annotations and their instantiated owner.
    pub annotations: Vec<crate::annotation::Annotation>,
    /// Marked shared engineering-default constants visible in the checked package.
    /// Rule identity is the constant declaration identity; marker rows are provenance.
    pub engineering_rules: Vec<crate::annotation::EngineeringRule>,
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
    /// Independent scalar coordinate bindings in this single occurrence.
    pub bindings: Vec<(SemanticId, SemanticId)>,
    /// Equality identities generated from those independent bindings.
    pub rows: Vec<SemanticId>,
    /// Original connection declaration.
    pub lineage: Lineage,
}
#[derive(Clone)]
struct State {
    /// Only selections consumed while constructing or demanding this instance's members.
    selections: crate::scientific_selection::Selections,
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
}
/// Syntax ownership of a numerical or Set scope argument before its value is inlined.
/// Coefficients remain owned by the admitted records; this retains only their read source.
#[derive(Clone)]
struct NumericalSource {
    instance: InstanceId,
    declaration: DeclarationId,
    expression: Expr,
    env: std::sync::Arc<Environment>,
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
    selection_collector: crate::scientific_selection::Collector,
    pub(crate) p: &'a CheckedPackage,
    pub(crate) c: &'a TypeContext<'b>,
    limits: Limits,
    pub(crate) model: SpecializedModel,
    states: BTreeMap<InstanceId, State>,
    numerical_sources: BTreeMap<(InstanceId, String), NumericalSource>,
    numerical_source_stack: Vec<(InstanceId, String)>,
    source_locals: BTreeSet<String>,
    set_alias_sources: BTreeMap<String, NumericalSource>,
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
    pending_inventories: Vec<(InstanceId, Declaration, Environment)>,
    /// Original integrated initial rows captured before any inventory lowers them.
    inventory_initial_sources: BTreeMap<SemanticId, (SemanticId, Expr, Lineage)>,
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
        selection_collector: crate::scientific_selection::Collector::default(),
        p: package,
        c: &context,
        limits,
        model: SpecializedModel {
            engineering_rules: crate::annotation::engineering_rules(package)?,
            ..SpecializedModel::default()
        },
        states: BTreeMap::new(),
        numerical_sources: BTreeMap::new(),
        numerical_source_stack: Vec::new(),
        source_locals: BTreeSet::new(),
        set_alias_sources: BTreeMap::new(),
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
        pending_inventories: Vec::new(),
        inventory_initial_sources: BTreeMap::new(),
        relaxations: BTreeMap::new(),
        form_realizations: BTreeMap::new(),
        facts: ambient.clone(),
        objective_level: crate::analysis::objective_level(&bindings.facts)?,
        discretizer,
        cancel,
        reader: crate::provenance::Reader::of(package, root),
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
        let resolved = engine.with_instance_selections(instance, |engine| {
            engine.rewrite(instance, &expr, &Environment::new(), &[root])
        })?;
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
    engine.finish(&bindings.formulation)?;
    let mut selected = engine
        .model
        .instances
        .values()
        .map(|instance| instance.definition)
        .collect::<BTreeSet<_>>();
    for lineage in engine
        .model
        .symbols
        .values()
        .map(|symbol| &symbol.lineage)
        .chain(engine.model.equations.iter().map(|row| &row.lineage))
    {
        selected.insert(lineage.declaration);
        selected.extend(&lineage.demand);
        selected.extend(&lineage.presets);
        selected.extend(lineage.default_owner);
    }
    selected.extend(engine.model.functions.values().map(|function| function.id));
    engine.model.source_occurrences = std::sync::Arc::new(
        package
            .expression_occurrences()
            .filter(|(key, _)| selected.contains(&key.declaration))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    );
    engine.model.selection_closures = engine.selection_collector.into_inner();
    Ok(engine.model)
}
impl Engine<'_, '_> {
    fn with_instance_selections<T>(
        &mut self,
        instance: InstanceId,
        action: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let capture = self.selection_collector.instance_scope();
        let result = action(self);
        let selections = capture.finish();
        if let Some(state) = self.states.get_mut(&instance) {
            state.selections.extend(selections);
        }
        result
    }
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
        expression: &Expr,
        env: &Environment,
        at: DeclarationId,
        ports: bool,
    ) -> Result<Vec<(SemanticId, Type, Environment)>> {
        let expression = expression.clone();
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
                .map(|index| self.eval_ast_with(at, env, index, None))
                .collect::<Result<Vec<_>>>()?;
            let coordinates = self.member_coordinates(owner, member, values)?;
            let row = self.p.declarations[&member].clone();
            let ty = crate::annotation::target_type(
                self.p,
                self.c,
                member,
                &Expr {
                    kind: ExprKind::Path(Path {
                        segments: vec![PathSegment {
                            name: row.name.clone(),
                            indices: Vec::new(),
                        }],
                    }),
                    span: Span::default(),
                },
                &self.source_types(member, &self.states[&owner].env)?,
            )?;
            let local = coordinates_env(env, &coordinates);
            let equation = self
                .p
                .equation_at(member, "equation.expression", 0)?
                .clone();
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
            if ports && row.value.kind == Kind::StatePort {
                let indices = row
                    .value
                    .state_port
                    .as_ref()
                    .ok_or_else(|| invalid(at, "material port payload missing"))?;
                return self
                    .coordinates(
                        owner,
                        member,
                        &self.states[&owner].env,
                        indices
                            .indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )?
                    .into_iter()
                    .map(|coordinates| {
                        Ok((
                            member_id(owner, member, &coordinates),
                            Type::Boolean,
                            coordinates_env(env, &coordinates),
                        ))
                    })
                    .collect();
            }
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
                owner,
                member,
                &self.states[&owner].env.clone(),
                indices.into_iter(),
            )?;
            let ty = crate::annotation::target_type(
                self.p,
                self.c,
                member,
                &Expr {
                    kind: ExprKind::Path(Path {
                        segments: vec![PathSegment {
                            name: row.name.clone(),
                            indices: Vec::new(),
                        }],
                    }),
                    span: Span::default(),
                },
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
                } else if row.value.equation.is_some() {
                    let equation = self
                        .p
                        .equation_at(member, "equation.expression", 0)?
                        .clone();
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
            if !matches!(
                self.p.declarations[&member].value.kind,
                Kind::Port | Kind::StatePort
            ) {
                return Err(invalid(at, "connectivity target must be a declared port"));
            }
            return Ok(vec![(
                member_id(owner, member, &coordinates),
                self.p.types.get(&member).cloned().unwrap_or(Type::Boolean),
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
    pub(crate) fn eval_ast(
        &self,
        at: DeclarationId,
        env: &Environment,
        expression: &Expr,
    ) -> Result<Value> {
        self.eval_ast_with(at, env, expression, None)
    }
    pub(crate) fn eval_ast_with(
        &self,
        at: DeclarationId,
        env: &Environment,
        expression: &Expr,
        expected: Option<&Type>,
    ) -> Result<Value> {
        self.checkpoint()?;
        let value = Evaluator {
            package: self.p,
            physical: self.c,
            at,
            env,
            limit: self.limits.members,
            stack: Vec::new(),
            reader: self.reader,
            selections: Some(&self.selection_collector),
        }
        .expr(expression, expected, 0)?;
        if expected.is_some_and(|expected| !value::conforms(&value, expected, self.p)) {
            return Err(invalid(
                at,
                "static value does not satisfy its checked expected type",
            ));
        }
        Ok(value)
    }
    pub(crate) fn eval_field(
        &self,
        at: DeclarationId,
        env: &Environment,
        role: &str,
        position: usize,
        expected: Option<&Type>,
    ) -> Result<Value> {
        self.checkpoint()?;
        let occurrence = self
            .p
            .expression_occurrence(at, role, position)
            .ok_or_else(|| {
                invalid(
                    at,
                    format!("checked value occurrence absent: {role}[{position}]"),
                )
            })?;
        let mut evaluator = Evaluator {
            package: self.p,
            physical: self.c,
            at,
            env,
            limit: self.limits.members,
            stack: Vec::new(),
            reader: self.reader,
            selections: Some(&self.selection_collector),
        };
        let value = match &occurrence.syntax {
            crate::expression::occurrences::Syntax::Expression(expression) => {
                evaluator.expr(expression, expected, 0)?
            }
            crate::expression::occurrences::Syntax::Static(value) => {
                evaluator.syntax(value, expected, 0)?
            }
            _ => return Err(invalid(at, "value occurrence owns a different grammar")),
        };
        if expected.is_some_and(|expected| !value::conforms(&value, expected, self.p)) {
            return Err(invalid(
                at,
                "static value does not satisfy its checked expected type",
            ));
        }
        Ok(value)
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "one constructor application keeps its target, owner, source and the three environments explicit"
    )]
    fn remember_constructor_sources(
        &mut self,
        target: InstanceId,
        owner: InstanceId,
        at: DeclarationId,
        env: &Environment,
        supplied: &Environment,
        overridden: &Environment,
    ) -> Result<()> {
        use pse_authoring::language::StaticValue;
        let StaticValue::Apply { arguments, .. } = self.p.static_at(at, "binding.expression", 0)?
        else {
            return Ok(());
        };
        let mut source_env = None;
        for (name, argument) in arguments {
            if overridden.contains_key(name)
                || !supplied.get(name).is_some_and(|value| {
                    matches!(value, Value::Set(_))
                        || value_type(value).is_some_and(|ty| ty.quantity_scheme().is_some())
                })
            {
                continue;
            }
            if let StaticValue::Expression(expression) = argument {
                let env = if let Some(env) = &source_env {
                    std::sync::Arc::clone(env)
                } else {
                    let retained = self.numerical_source_environment(env)?;
                    source_env = Some(retained.clone());
                    retained
                };
                let mut count = 0usize;
                expression.walk(|_| count += 1);
                self.reserve(count.saturating_add(1))?;
                self.numerical_sources
                    .entry((target, name.clone()))
                    .or_insert(NumericalSource {
                        instance: owner,
                        declaration: at,
                        expression: expression.clone(),
                        env,
                    });
            }
        }
        Ok(())
    }
    fn numerical_source_environment(
        &mut self,
        env: &Environment,
    ) -> Result<std::sync::Arc<Environment>> {
        fn nodes(value: &Value) -> usize {
            let children = match value {
                Value::Definition { bindings, .. } => bindings
                    .values()
                    .map(nodes)
                    .fold(0usize, usize::saturating_add),
                Value::Set(values) | Value::Tuple(values) => {
                    values.iter().map(nodes).fold(0usize, usize::saturating_add)
                }
                Value::Row { fields, .. } => {
                    fields.iter().map(nodes).fold(0usize, usize::saturating_add)
                }
                _ => 0,
            };
            children.saturating_add(1)
        }
        self.reserve(
            env.values()
                .map(nodes)
                .fold(env.len(), usize::saturating_add),
        )?;
        Ok(std::sync::Arc::new(env.clone()))
    }
    fn inline_evidence_members(
        &mut self,
        expression: &Expr,
        visited: &mut BTreeSet<SemanticId>,
        depth: usize,
    ) -> Result<Expr> {
        if depth >= self.limits.depth {
            return Err(ModelingError::Budget(
                "numerical prerequisite member depth".into(),
            ));
        }
        let mut expression = expression.clone();
        expression.try_walk_mut(|node| -> Result<()> {
            self.reserve(1)?;
            let ExprKind::Path(path) = &node.kind else {
                return Ok(());
            };
            let [segment] = path.segments.as_slice() else {
                return Ok(());
            };
            let Some(id) = segment
                .name
                .strip_prefix("s_")
                .and_then(|id| SemanticId::parse_hex(id).ok())
            else {
                return Ok(());
            };
            let Some(definition) = self
                .model
                .symbols
                .get(&id)
                .and_then(|s| s.expression.clone())
            else {
                return Ok(());
            };
            if !visited.insert(id) {
                return Err(invalid(id, "cyclic numerical prerequisite member"));
            }
            let result = self.inline_evidence_members(&definition, visited, depth + 1);
            visited.remove(&id);
            *node = result?;
            Ok(())
        })?;
        Ok(expression)
    }
    fn has_applicability_effect(&self, expression: &Expr) -> bool {
        let mut pending = Vec::new();
        let calls = |expression: &Expr, pending: &mut Vec<String>| {
            expression.walk(|node| match &node.kind {
                ExprKind::NamedCall { name, .. } | ExprKind::Partial { function: name, .. } => {
                    pending.push(dsl::render_path(name));
                }
                _ => {}
            });
        };
        calls(expression, &mut pending);
        let mut seen = BTreeSet::new();
        while let Some(name) = pending.pop() {
            if !seen.insert(name.clone()) {
                continue;
            }
            if let Some(function) = self.model.functions.get(&name) {
                if !function.applicability_uses.is_empty() {
                    return true;
                }
                if let Some(body) = &function.body {
                    calls(body, &mut pending);
                }
            }
        }
        false
    }
    fn numerical_source_expression(
        &mut self,
        consumer: InstanceId,
        source: &NumericalSource,
        value: Expr,
        ty: Type,
    ) -> Result<Expr> {
        // The default owns its lexical environment. A numerical caller's local aliases
        // and function arguments cannot capture any name in that source.
        let lexical = std::mem::take(&mut self.lexical);
        let indexed = std::mem::take(&mut self.indexed_arguments);
        let types = std::mem::take(&mut self.function_types);
        let locals = std::mem::take(&mut self.source_locals);
        let aliases = std::mem::take(&mut self.set_alias_sources);
        let result = self.with_instance_selections(source.instance, |engine| {
            engine.rewrite(
                source.instance,
                &source.expression,
                &source.env,
                &[source.declaration],
            )
        });
        self.lexical = lexical;
        self.indexed_arguments = indexed;
        self.function_types = types;
        self.source_locals = locals;
        self.set_alias_sources = aliases;
        let prerequisite = self.inline_evidence_members(&result?, &mut BTreeSet::new(), 0)?;
        if !self.has_applicability_effect(&prerequisite) {
            return Ok(value);
        }
        let mut identity = FramedHasher::new(pse_ids::Frame::ModelingParameterReadV1);
        identity
            .id(&consumer.as_id())
            .id(&source.instance.as_id())
            .id(&source.declaration.as_id())
            .str(&dsl::render_expr(&source.expression))
            .str(&dsl::render_expr(&prerequisite))
            .str(&dsl::render_expr(&value))
            .u64(source.env.len() as u64);
        for (name, value) in source.env.iter() {
            identity.str(name);
            value.frame(&mut identity);
        }
        if let Some(scheme) = ty.quantity_scheme() {
            scheme.frame(&mut identity);
        }
        identity.bool(ty.physical_refinement().is_some());
        if let Some(refinement) = ty.physical_refinement() {
            refinement.frame(&mut identity);
        }
        identity.u64(1).u64(1);
        let id = DeclarationId::from(identity.finish_id());
        let name = format!("f_{}", id.as_id().to_hex());
        self.reserve(1)?;
        self.model
            .functions
            .entry(name.clone())
            .or_insert(crate::Function {
                applicability: Vec::new(),
                applicability_uses: Vec::new(),
                prerequisites: vec![1],
                physical_admissions: BTreeMap::new(),
                physical_operation: None,
                reduction: None,
                validity: None,
                envelopes: Vec::new(),
                validity_reads: Default::default(),
                external: None,
                continuity: None,
                id,
                variables: BTreeSet::new(),
                arguments: vec![("value".into(), ty.clone()), ("source".into(), ty.clone())],
                result: ty,
                body: Some(Expr {
                    kind: ExprKind::Path(Path::single("value")),
                    span: Span::default(),
                }),
            });
        Ok(Expr {
            kind: ExprKind::NamedCall {
                name: Path::single(name),
                args: vec![value, prerequisite],
            },
            span: Span::default(),
        })
    }
    fn scope_parameter_expression(
        &mut self,
        instance: InstanceId,
        name: &str,
        value: &Value,
        at: DeclarationId,
    ) -> Result<Expr> {
        let expression = self.value_expression(value, at)?;
        let Some(ty) = value_type(value).filter(|ty| ty.quantity_scheme().is_some()) else {
            return Ok(expression);
        };
        let key = (instance, name.to_owned());
        let Some(source) = self.numerical_sources.get(&key).cloned() else {
            return Ok(expression);
        };
        if self.numerical_source_stack.contains(&key) {
            return Err(invalid(at, "recursive numerical parameter source"));
        }
        if self.numerical_source_stack.len() >= self.limits.depth {
            return Err(ModelingError::Budget(
                "numerical parameter source depth".into(),
            ));
        }
        self.numerical_source_stack.push(key);
        let result = self.numerical_source_expression(instance, &source, expression, ty);
        self.numerical_source_stack.pop();
        result
    }
    fn symbol_value_expression(
        &mut self,
        instance: InstanceId,
        member: DeclarationId,
        coordinates: &[(String, Value)],
        symbol: SemanticId,
    ) -> Result<Expr> {
        let row = &self.p.declarations[&member];
        if matches!(row.value.kind, Kind::Let | Kind::Alias)
            && let Some(expression) = self.model.symbols[&symbol].expression.clone()
        {
            let expanded = self.inline_evidence_members(&expression, &mut BTreeSet::new(), 0)?;
            if self.has_applicability_effect(&expanded) {
                return Ok(expanded);
            }
        }
        let Some(_expression) = row
            .value
            .binding
            .as_ref()
            .and_then(|b| b.expression.as_deref())
            .filter(|_| row.value.kind == Kind::Parameter)
        else {
            return Ok(symbol_expr(symbol));
        };
        let source = NumericalSource {
            instance,
            declaration: member,
            expression: self
                .p
                .expression_at(member, "binding.expression", 0)?
                .clone(),
            env: self.numerical_source_environment(&coordinates_env(
                &self.states[&instance].env,
                coordinates,
            ))?,
        };
        self.numerical_source_expression(
            instance,
            &source,
            symbol_expr(symbol),
            self.model.symbols[&symbol].ty.clone(),
        )
    }
    fn predicate(&self, at: DeclarationId, env: &Environment, role: &str) -> Result<bool> {
        let p = self.p.predicate_at(at, role, 0)?;
        Evaluator {
            package: self.p,
            physical: self.c,
            at,
            env,
            limit: self.limits.members,
            stack: Vec::new(),
            reader: self.reader,
            selections: Some(&self.selection_collector),
        }
        .predicate(p)
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
        self.with_instance_selections(id, |engine| {
            engine.instantiate_body(definition, id, parent, path, arguments, scope)
        })
    }
    fn instantiate_body(
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
            let _expr = binding
                .expression
                .as_ref()
                .ok_or_else(|| invalid(definition, "preset requires application"))?;
            let value = self.eval_field(definition, &arguments, "binding.expression", 0, None)?;
            let Value::Definition {
                id: target,
                mut bindings,
            } = value
            else {
                return Err(invalid(definition, "preset target"));
            };
            self.remember_constructor_sources(
                id, id, definition, &arguments, &bindings, &arguments,
            )?;
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
            selections: Some(&self.selection_collector),
        }
        .definition_environment(definition, &arguments, env.clone())?;
        let mut source_env = None;
        for (position, parameter) in contract.parameters.iter().enumerate() {
            if !arguments.contains_key(&parameter.name)
                && env.get(&parameter.name).is_some_and(|value| {
                    matches!(value, Value::Set(_))
                        || value_type(value).is_some_and(|ty| ty.quantity_scheme().is_some())
                })
                && parameter.default_value.is_some()
            {
                let pse_authoring::language::StaticValue::Expression(expression) = self
                    .p
                    .static_at(definition, "scope.parameters.default_value", position)?
                else {
                    // Literal structural collections have no numerical read or
                    // selection expression to replay. Their admitted value is enough.
                    continue;
                };
                let retained_env = if let Some(env) = &source_env {
                    std::sync::Arc::clone(env)
                } else {
                    let retained = self.numerical_source_environment(&env)?;
                    source_env = Some(retained.clone());
                    retained
                };
                let mut count = 0usize;
                expression.walk(|_| count += 1);
                self.reserve(count.saturating_add(1))?;
                self.numerical_sources
                    .entry((id, parameter.name.clone()))
                    .or_insert(NumericalSource {
                        instance: id,
                        declaration: definition,
                        expression: expression.clone(),
                        env: retained_env,
                    });
            }
        }
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
                let _source = b
                    .expression
                    .as_ref()
                    .ok_or_else(|| invalid(*member, "scope value"))?;
                published.insert(
                    r.name.clone(),
                    self.eval_field(
                        *member,
                        &env,
                        "binding.expression",
                        0,
                        self.p.types.get(member),
                    )?,
                );
            }
        }
        self.states.insert(
            id,
            State {
                selections: BTreeMap::new(),
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
                    id,
                    *member,
                    &env,
                    a.indices
                        .iter()
                        .map(|i| (i.name.as_str(), i.domain.as_str())),
                )? {
                    let key = member_id(id, *member, &coordinates);
                    self.reserve(1)?;
                    let ty = self.p.types[member].clone();
                    let tolerance = self.eval_field(
                        *member,
                        &coordinates_env(&env, &coordinates),
                        "accumulator.tolerance",
                        0,
                        Some(&ty),
                    )?;
                    if tolerance.scalar(*member)? <= 0.0 {
                        return Err(invalid(*member, "positive closure tolerance required"));
                    }
                    let boundary = a
                        .boundary
                        .as_ref()
                        .map(|_| {
                            self.resolve_boundary(
                                id,
                                *member,
                                &self
                                    .p
                                    .expression_at(*member, "accumulator.boundary", 0)?
                                    .clone(),
                                &coordinates_env(&env, &coordinates),
                            )
                        })
                        .transpose()?;
                    self.model.closures.insert(
                        key,
                        Closure {
                            id: key,
                            mode: a.mode,
                            observation_only: false,
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
                && self
                    .p
                    .types
                    .get(member)
                    .is_some_and(|ty| ty.quantity_scheme().is_some())
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
                    id,
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
                    id,
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
                    let value = self.eval_field(
                        *member,
                        &coordinates_env(&env, &coordinates),
                        "binding.expression",
                        0,
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
                    self.remember_constructor_sources(
                        child,
                        id,
                        *member,
                        &coordinates_env(&env, &coordinates),
                        &bindings,
                        &Environment::new(),
                    )?;
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
                for coordinates in self.coordinates(
                    id,
                    *member,
                    &env,
                    exchange
                        .indices
                        .iter()
                        .map(|i| (i.name.as_str(), i.domain.as_str())),
                )? {
                    self.reserve(1)?;
                    let local = coordinates_env(&env, &coordinates);
                    let from = self.resolve_boundary(
                        id,
                        *member,
                        self.p.expression_at(*member, "exchange.from", 0)?,
                        &local,
                    )?;
                    let to = self.resolve_boundary(
                        id,
                        *member,
                        self.p.expression_at(*member, "exchange.to", 0)?,
                        &local,
                    )?;
                    let pair = crate::contextual::PairedExchange::admit(*member, from, to)?;
                    self.model
                        .exchanges
                        .insert(member_id(id, *member, &coordinates), pair);
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
                        id,
                        *member,
                        &env,
                        e.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        let equation = self
                            .p
                            .equation_at(*member, "equation.expression", 0)?
                            .clone();
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
                        id,
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
                        id,
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
                    if !self.predicate(*member, &env, "requirement.predicate")? {
                        return Err(invalid(*member, &r.message));
                    }
                }
                Selected::Port(b) => {
                    for coordinates in self.coordinates(
                        id,
                        *member,
                        &env,
                        b.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        let expr = self
                            .p
                            .expression_at(*member, "binding.expression", 0)?
                            .clone();
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
                        id,
                        *member,
                        &env,
                        c.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        self.contribution(id, &r, &coordinates)?;
                    }
                }
                Selected::StateSpecification(_) => {
                    self.state_specification(id, &r, &env)?;
                }
                Selected::StatePort(_) => {
                    self.material_port(id, &r, &env)?;
                }
                Selected::InventoryBalance(_) => {
                    self.pending_inventories.push((id, r.clone(), env.clone()));
                }
                Selected::Connection(_) => {
                    self.process_connection(id, &r, &env)?;
                }
                Selected::Expectation(test) => {
                    let mut rewrite = |role: &str| -> Result<Expr> {
                        self.rewrite(
                            id,
                            self.p.expression_at(*member, role, 0)?,
                            &env,
                            &[*member],
                        )
                    };
                    let actual = rewrite("expectation.actual")?;
                    let expected = rewrite("expectation.expected")?;
                    let tolerance = rewrite("expectation.tolerance")?;
                    let relative_tolerance = if test.relative_tolerance.is_some() {
                        rewrite("expectation.relative_tolerance")?
                    } else {
                        Expr {
                            kind: ExprKind::Number(Number {
                                exact_integer: Some(0),
                                value: 0.0,
                                unit: None,
                            }),
                            span: Span::default(),
                        }
                    };
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
        self.with_instance_selections(id, |engine| {
            engine.equation_body(id, member, equation, coordinates, env)
        })
    }
    fn equation_body(
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
            let Selected::Equation(_e) = self.p.declarations[&occurrence.member]
                .value
                .selected()
                .map_err(|e| invalid(occurrence.member, e.to_string()))?
            else {
                return Err(invalid(occurrence.member, "equation payload"));
            };
            let equation = self
                .p
                .equation_at(occurrence.member, "equation.expression", 0)?
                .clone();
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
            let active = if row.value.guard.is_some() {
                self.predicate(id, env, "guard.predicate")?
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
        instance: InstanceId,
        at: DeclarationId,
        env: &Environment,
        indices: impl Iterator<Item = (&'a str, &'a str)>,
    ) -> Result<Vec<Vec<(String, Value)>>> {
        let mut output = vec![Vec::new()];
        let declaration = &self.p.declarations[&at];
        let role = crate::expression::member_index_role(declaration);
        let names = crate::expression::member_index_names(declaration);
        for (name, _) in indices {
            let position = names
                .iter()
                .position(|selected| *selected == name)
                .ok_or_else(|| invalid(at, "index binder has no checked declaration occurrence"))?;
            let mut next = Vec::new();
            for row in output {
                self.checkpoint()?;
                let local = coordinates_env(env, &row);
                let domain = match crate::temporal::index_domain(self.p, at, position)? {
                    crate::temporal::IndexDomain::Authored { position } => {
                        self.eval_field(at, &local, &role, position, None)?
                    }
                    crate::temporal::IndexDomain::Temporal { policy, axis } => {
                        let mut owner = Some(instance);
                        let mut mesh = None;
                        while let Some(candidate) = owner {
                            let state = &self.states[&candidate];
                            if state.members.values().any(|member| *member == axis) {
                                mesh = self.model.meshes.get(&member_id(candidate, axis, &[]));
                                if mesh.is_some() {
                                    break;
                                }
                            }
                            owner = state.parent;
                        }
                        let mesh = mesh.ok_or_else(|| {
                            invalid(
                                policy,
                                format!(
                                    "resolved temporal axis {} has no active realization",
                                    axis.as_id().to_hex()
                                ),
                            )
                        })?;
                        Value::Set(mesh.points.clone())
                    }
                };
                let Value::Set(values) = domain else {
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
        let result = self.with_instance_selections(instance, |engine| {
            engine.symbol_member(instance, member, coordinates, chain)
        });
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
            if accumulator.mode == Mode::Conservation {
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
            let expr = self
                .p
                .expression_at(member, "binding.expression", 0)?
                .clone();
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
                .map(|_| self.eval_field(member, &env, "binding.expression", 0, Some(&ty)))
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
        if let Some(_source) = &b.expression {
            if row.value.kind == Kind::Parameter {
                let value = self.eval_field(member, &env, "binding.expression", 0, Some(&ty))?;
                self.compatible(&value, &ty, member)?;
                self.model
                    .symbols
                    .get_mut(&id)
                    .ok_or_else(|| invalid(id, "symbol missing"))?
                    .initial = Some(value);
            } else if matches!(row.value.kind, Kind::Let | Kind::Alias) {
                let expr = self
                    .p
                    .expression_at(member, "binding.expression", 0)?
                    .clone();
                let expr = self.rewrite(instance, &expr, &env, &demand)?;
                self.model
                    .symbols
                    .get_mut(&id)
                    .ok_or_else(|| invalid(id, "symbol missing"))?
                    .expression = Some(expr);
            }
        }
        if b.defined_by.is_some() {
            let equation = self.p.equation_at(member, "binding.defined_by", 0)?.clone();
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
        let path = self.p.expression_at(id, "contribution.target", 0)?.clone();
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
        let expression = self
            .p
            .expression_at(id, "contribution.expression", 0)?
            .clone();
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
            if c.role == Role::Directed {
                None
            } else {
                Some(&ty)
            },
        )?;
        if c.role == Role::Directed {
            let boundary = self.model.closures[&target]
                .boundary
                .as_ref()
                .ok_or_else(|| {
                    invalid(
                        id,
                        "directed contribution requires an explicit accumulator boundary",
                    )
                })?;
            let refinement = actual
                .physical_refinement()
                .ok_or_else(|| invalid(id, "directed contribution requires a transfer"))?;
            refinement.contribution(boundary, false, id)?;
            let payload = Type::Quantity(
                actual
                    .quantity_scheme()
                    .ok_or_else(|| invalid(id, "transfer requires physical payload"))?
                    .clone(),
            );
            let contracts = [&ty, &payload]
                .iter()
                .map(|value| {
                    value
                        .quantity_scheme()
                        .ok_or_else(|| invalid(id, "ledger requires physical payload"))?
                        .resolve_contract_with_evidence(
                            self.c.quantities,
                            &BTreeMap::new(),
                            self.c.preconditions,
                        )
                        .map_err(|error| invalid(id, error.to_string()))
                })
                .collect::<Result<Vec<_>>>()?;
            let expected = contracts[0]
                .require_named()
                .map_err(|error| invalid(id, error.to_string()))?;
            pse_quantity::resolved::infer_operation(
                &pse_quantity::infer::OpRequest::Add,
                &contracts,
                Some(expected),
                self.c.quantities,
                self.c.preconditions,
            )
            .map_err(|error| {
                invalid(
                    id,
                    format!("directed transfer does not belong in this physical ledger: {error}"),
                )
            })?;
            expression = self.physical_transfer_function(
                id,
                actual.clone(),
                payload,
                crate::PhysicalOperation::TransferMagnitude {
                    source: refinement.clone(),
                },
                1,
                expression,
            )?;
        } else if actual.physical_refinement().is_some() {
            return Err(invalid(
                id,
                "a directed transfer must be consumed with role directed exactly once",
            ));
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
        self.finish_process_states()?;
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
            if closure.mode == Mode::Conservation && !closure.observation_only {
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
            } else if matches!(closure.mode, Mode::Accounting | Mode::Observation) {
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
        if !self.pending_inventories.is_empty() {
            for source in &self.model.equations {
                if !self.model.initial_equations.contains(&source.id) {
                    continue;
                }
                if let EquationKind::Relation {
                    lhs,
                    rhs,
                    sense: EquationSense::Eq,
                } = &source.equation.kind
                {
                    for (target, value) in [(lhs, rhs), (rhs, lhs)] {
                        let Some(symbol) = symbol_reference(target) else {
                            continue;
                        };
                        if !self
                            .model
                            .symbols
                            .get(&symbol)
                            .is_some_and(|s| s.role == Kind::Variable)
                        {
                            continue;
                        }
                        if self
                            .inventory_initial_sources
                            .insert(symbol, (source.id, value.clone(), source.lineage.clone()))
                            .is_some()
                        {
                            return Err(invalid(
                                source.lineage.declaration,
                                "competing original inventory coordinate initial conditions",
                            ));
                        }
                    }
                }
            }
        }
        for (instance, declaration, env) in std::mem::take(&mut self.pending_inventories) {
            self.inventory_balance(instance, &declaration, &env)?;
        }
        self.validate_contextual_equations()?;
        self.select_formulation(formulation)?;
        self.apply_relaxations()?;
        self.admit_objectives()?;
        self.group_bodies()?;
        self.admit_function_occurrences()?;
        self.model.equations.sort_by_key(|r| r.id);
        Ok(())
    }
    fn typed_zero(&self, ty: &Type, at: DeclarationId) -> Result<Expr> {
        let s = ty
            .quantity_scheme()
            .ok_or_else(|| invalid(at, "accumulator requires quantity"))?;
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
        ) {
            -1.0
        } else {
            1.0
        }
    }
}
impl SpecializedModel {
    /// Number of required original physical closure checks, excluding accounting reports.
    pub fn required_closure_checks(&self) -> usize {
        self.closures
            .values()
            .filter(|c| c.requires_check())
            .count()
    }
    /// Borrow revision attribution under the specialized model's allocation owner.
    pub fn source_occurrences(
        &self,
    ) -> &BTreeMap<
        crate::expression::occurrences::OccurrenceKey,
        crate::expression::occurrences::CheckedExpression,
    > {
        &self.source_occurrences
    }

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
                    satisfied: closure.requires_check().then_some(net.abs() <= tolerance),
                })
            })
            .collect()
    }
}
