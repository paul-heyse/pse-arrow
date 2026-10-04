// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Case/test data binds existing symbols; it never adds equations or changes model fixedness.
use super::*;
use pse_model::generated::{
    authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixture as Contract,
    enums::ModelingFixtureBinding as Binding,
};
/// Bindings of one independent symbol, in canonical units; unset parts keep the model's.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureValue {
    /// Bound variable or parameter.
    pub target: SemanticId,
    /// Specified value.
    pub value: Option<f64>,
    /// Whether the fixture fixes (`true`) or frees (`false`) the variable.
    pub fixed: Option<bool>,
    /// Lower bound override.
    pub lower: Option<f64>,
    /// Upper bound override.
    pub upper: Option<f64>,
}
/// One authored case or test fixture of a model instance.
#[derive(Clone, Debug, PartialEq)]
pub struct Fixture {
    /// Declaration that authored the fixture.
    pub declaration: DeclarationId,
    /// Degrees of freedom the fixture expects after its bindings.
    pub expected_degrees_of_freedom: i64,
    /// Bindings keyed by their model path.
    pub specifications: BTreeMap<String, FixtureValue>,
    /// The source entity the fixture's expected values come from (ADR-0123 Outcome 5).
    pub oracle: Option<DeclarationId>,
    /// Admitted temporal route, with its Rust-owned default applied.
    pub route: pse_model::generated::enums::ModelingAnalysisRoute,
    /// Independent admitted authored procedure.
    pub procedure: pse_model::generated::enums::ModelingProcedure,
    /// Actual endpoint requirement, with an event member resolved before native work.
    pub endpoint: FixtureEndpoint,
    /// Declared solve intent (ADR-0119 Outcome 1); `None` leaves it to the runtime policy.
    pub intent: Option<pse_model::generated::enums::NativeSolveIntent>,
    /// Initialization stages to run, in order.
    pub stages: Vec<String>,
    /// Authored initialization settings.
    pub initialization: Option<pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixtureInitialization>,
    /// Integration samples and tolerances for the integrated route.
    pub integration: Option<IntegrationFixture>,
    /// Same-layout modes in order, the first starting the integration; empty for one
    /// smooth mode (ADR-0119 Outcome 3).
    pub modes: Vec<FixtureMode>,
    /// The shooting method and nodes of a shooting fixture.
    pub shooting: Option<ShootingFixture>,
    /// Failure the fixture expects instead of a result, its lineage resolved (Plan 23 H5).
    pub expected_failure: Option<ExpectedFailure>,
    /// Numerical diagnostic findings the fixture expects at its solved point (Plan 23 CT-S13).
    pub diagnostics: Vec<FixtureDiagnostic>,
}
/// Endpoint obligation admitted against this fixture's resolved event inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixtureEndpoint {
    /// Every requested fixed-horizon obligation remains required.
    FixedHorizon,
    /// The permitted terminal event, identified by its guard member.
    DeclaredTerminalEvent(SemanticId),
}
/// A numerical diagnostic finding a fixture expects: its rule and the members it names,
/// each resolved once to the equations or variables of every coordinate of the member.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureDiagnostic {
    /// The diagnostics rule, such as a near-parallel or rank-deficiency finding.
    pub rule: pse_diagnostics::DiagnosticRule,
    /// Each named member: its authored path and its equation or variable identities.
    pub members: Vec<(String, BTreeSet<SemanticId>)>,
}
/// The failure a fixture expects instead of a result (Plan 23 H5): its typed class and its
/// lineage, resolved once to identities. A failure is the expected one only when both agree;
/// a class alone never matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedFailure {
    /// The boundary class of the failure.
    pub class: pse_model::diagnostic::BoundaryClass,
    /// What the failure concerns.
    pub lineage: ExpectedLineage,
}
/// What an expected failure concerns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpectedLineage {
    /// A refused evidence claim; mathematical domains keep their separate lineage.
    Applicability {
        /// Declared form, data or closure evidence scope.
        layer: pse_model::generated::enums::ModelingValidityLayer,
        /// Outside a region or unknown evidence, independent of policy.
        outcome: pse_model::generated::enums::ModelingApplicabilityOutcome,
        /// Named evidence declaration.
        claim: DeclarationId,
        /// Scientific form that demanded it.
        form: DeclarationId,
        /// Exact selected records.
        sets: Vec<SemanticId>,
        /// Claim input names, matched as an exact set.
        variables: Vec<String>,
    },
    /// A rejected validity predicate (ADR-0123 Outcome 4): its layer; of a form or data
    /// layer predicate, the form, the parameter sets whose values bound it and the positions
    /// of the form's arguments it constrains; of a closure range, the members it bounds.
    /// Each list is sorted.
    Validity {
        /// Form, data or closure layer.
        layer: pse_model::generated::enums::ModelingValidityLayer,
        /// The form's declaration; none on the closure layer.
        form: Option<DeclarationId>,
        /// Table-row or entity identities.
        sets: Vec<SemanticId>,
        /// Positions among the form's declared arguments.
        variables: Vec<u32>,
        /// The members a closure range bounds.
        members: Vec<SemanticId>,
    },
    /// The members, variables or equations of the fixture's model, that a structural refusal
    /// or a diagnostic finding names.
    Members(BTreeSet<SemanticId>),
}
impl ExpectedFailure {
    /// Whether `observed` is this failure: the same class, and a validity rejection with
    /// exactly this lineage, or a finding naming every expected member.
    pub fn matches(&self, observed: &pse_model::diagnostic::BoundaryDiagnostic) -> bool {
        observed.class == self.class
            && match &self.lineage {
                ExpectedLineage::Applicability {
                    layer,
                    outcome,
                    claim,
                    form,
                    sets,
                    variables,
                } => observed.applicability.iter().any(|o| {
                    let mut records = o.claim.records.clone();
                    records.sort_unstable();
                    let mut inputs = o.inputs.iter().map(|i| i.name.clone()).collect::<Vec<_>>();
                    inputs.sort();
                    !o.admitted
                        && o.required
                        && o.claim.id == Some(claim.as_id())
                        && o.claim.form == form.as_id()
                        && o.claim.layer == *layer
                        && o.outcome == *outcome
                        && records == *sets
                        && inputs == *variables
                }),
                ExpectedLineage::Validity {
                    layer,
                    form,
                    sets,
                    variables,
                    members,
                } => observed.validity.as_ref().is_some_and(|lineage| {
                    let sorted = |mut v: Vec<SemanticId>| {
                        v.sort_unstable();
                        v
                    };
                    let mut observed_variables = lineage.variables.clone();
                    observed_variables.sort_unstable();
                    lineage.layer == *layer
                        && lineage.form == form.map(DeclarationId::as_id)
                        && sorted(lineage.sets.clone()) == *sets
                        && observed_variables == *variables
                        && sorted(lineage.members.clone()) == *members
                }),
                ExpectedLineage::Members(members) => members
                    .iter()
                    .all(|member| observed.sources.contains(member)),
            }
    }
}
/// Authored integration samples expressed in the admitted axis's canonical unit.
#[derive(Clone, Debug, PartialEq)]
pub struct IntegrationFixture {
    /// Strictly increasing sample times within the axis; the last one ends the horizon.
    pub samples: Vec<f64>,
    /// Positive initial step.
    pub initial_step: f64,
    /// Relative integration tolerance.
    pub relative_tolerance: f64,
    /// Absolute tolerance of every normalized state coordinate.
    pub normalized_absolute_tolerance: f64,
    /// Relative tolerance of the terminal quadratures, when there are any.
    pub quadrature_relative_tolerance: Option<f64>,
    /// Canonical absolute tolerance of each integral, keyed by the integral.
    pub quadratures: BTreeMap<SemanticId, f64>,
    /// Piecewise-constant inputs, in declaration order (ADR-0119 Outcome 2).
    pub schedules: Vec<ScheduleFixture>,
}
/// One authored input held piecewise constant: each interval's value is its own
/// integration parameter, so sensitivities stay live across every change (I6).
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleFixture {
    /// The scheduled independent parameter or variable.
    pub target: SemanticId,
    /// Strictly increasing change times after the axis's lower bound and up to the last
    /// sample, in the axis's canonical unit.
    pub times: Vec<f64>,
    /// One canonical value per interval: `times.len() + 1`, the first from the start.
    /// A control's values are the shooting starting guesses.
    pub values: Vec<f64>,
    /// Present when the schedule is held free: each interval value is a shooting control.
    pub control: Option<ScheduleControl>,
}
/// The canonical bounds of a schedule held free; unbounded where absent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScheduleControl {
    /// Lower bound of every interval value.
    pub lower: Option<f64>,
    /// Upper bound of every interval value.
    pub upper: Option<f64>,
}
/// A shooting fixture's method and, for multiple shooting, its inner nodes in the axis's
/// canonical unit (ADR-0110 Outcome 5).
#[derive(Clone, Debug, PartialEq)]
pub struct ShootingFixture {
    /// Single or multiple shooting.
    pub method: pse_model::generated::enums::ShootingMethod,
    /// Strictly increasing inner node times inside the horizon; empty for single shooting.
    pub nodes: Vec<f64>,
}
/// One authored same-layout mode: the Boolean facts that select its `when` variants and
/// stage overrides, and the events active in it (ADR-0119 Outcome 3).
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureMode {
    /// Name, unique within the fixture.
    pub name: String,
    /// Structural facts bound while the mode is active.
    pub facts: BTreeMap<crate::analysis::Fact, bool>,
    /// Events active in the mode, in declaration order.
    pub events: Vec<FixtureEvent>,
}
/// A zero crossing of an authored scalar member.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureEvent {
    /// Authored event name: the guard member path as declared by the fixture.
    pub name: String,
    /// The member whose zero crossing triggers the event.
    pub guard: SemanticId,
    /// The crossings that trigger it.
    pub direction: pse_model::generated::enums::EventDirection,
    /// Positive absolute guard tolerance for ambiguity detection, in the guard's canonical
    /// unit.
    pub tolerance: f64,
    /// Each reset member and the member whose value it takes, of exactly its type.
    pub reset: BTreeMap<SemanticId, SemanticId>,
    /// The successor mode's position; `None` stops the integration at the event.
    pub next: Option<usize>,
}
impl Engine<'_, '_> {
    /// The fixture's modes and events, resolved to members (ADR-0119 Outcome 3). Only the
    /// integrated route admits them: the steady and simultaneous routes refuse them.
    fn fixture_modes(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        contract: &Contract,
        env: &Environment,
    ) -> Result<Vec<FixtureMode>> {
        if contract.modes.is_empty() {
            return Ok(Vec::new());
        }
        let route = crate::analysis::route(&self.facts)?;
        if route != crate::analysis::Route::Integrated {
            return Err(invalid(
                at,
                format!(
                    "authored modes and events need the integrated route; the {} route refuses them",
                    route.as_str()
                ),
            ));
        }
        let member = |engine: &mut Self, expression: &Expr| {
            let targets = engine.annotation_targets(instance, expression, env, at, false)?;
            match targets.as_slice() {
                [(id, ty, local)] if engine.model.symbols.contains_key(id) => {
                    Ok((*id, ty.clone(), local.clone()))
                }
                _ => Err(invalid(at, "an event names one scalar member")),
            }
        };
        let mut modes = Vec::with_capacity(contract.modes.len());
        let mut event_ordinal = 0;
        for (mode_position, mode) in contract.modes.iter().enumerate() {
            let mut events = Vec::with_capacity(mode.events.len());
            for (event_position, event) in mode.events.iter().enumerate() {
                let expression =
                    self.p
                        .expression_at(at, "scope.fixture.modes.events.guard", event_ordinal)?;

                let ExprKind::Path(path) = &expression.kind else {
                    return Err(invalid(at, "an event guard names a member path"));
                };
                let name = dsl::render_path(path);
                let (guard, ty, local) = member(self, &expression.clone())?;
                if !matches!(ty, Type::Quantity(_)) {
                    return Err(invalid(at, "an event guard requires a physical type"));
                }
                let tolerance = self
                    .eval_field(
                        at,
                        &local,
                        "scope.fixture.modes.events.tolerance",
                        event_ordinal,
                        Some(&ty),
                    )?
                    .scalar(at)?;
                if !tolerance.is_finite() || tolerance <= 0. {
                    return Err(invalid(at, "an event tolerance is positive and finite"));
                }
                event_ordinal += 1;
                let mut reset = BTreeMap::new();
                for (reset_position, _assignment) in event.reset.iter().enumerate() {
                    let (target, ..) = member(self, &self.p.expression_at(at, &format!("scope.fixture.modes.{mode_position}.events.{event_position}.reset.{reset_position}.target"), 0)?.clone())?;
                    let (value, ..) = member(self, &self.p.expression_at(at, &format!("scope.fixture.modes.{mode_position}.events.{event_position}.reset.{reset_position}.expression"), 0)?.clone())?;
                    if self.model.symbols[&target].ty != self.model.symbols[&value].ty
                        || reset.insert(target, value).is_some()
                    {
                        return Err(invalid(
                            at,
                            "event resets assign unique members from members of exactly their physical type",
                        ));
                    }
                }
                let next = event
                    .next
                    .as_ref()
                    .map(|next| {
                        contract
                            .modes
                            .iter()
                            .position(|m| m.name == *next)
                            .ok_or_else(|| invalid(at, "event successor mode absent"))
                    })
                    .transpose()?;
                events.push(FixtureEvent {
                    name,
                    guard,
                    direction: event.direction,
                    tolerance,
                    reset,
                    next,
                });
            }
            self.reserve(1 + events.len())?;
            modes.push(FixtureMode {
                name: mode.name.clone(),
                facts: mode
                    .facts
                    .iter()
                    .map(|f| {
                        crate::analysis::Fact::new(f.namespace, &f.name)
                            .filter(crate::analysis::Fact::is_structural)
                            .map(|fact| (fact, f.value))
                            .ok_or_else(|| invalid(at, "a mode binds stage facts only"))
                    })
                    .collect::<Result<_>>()?,
                events,
            });
        }
        Ok(modes)
    }
    pub(super) fn fixture(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        contract: &Contract,
        oracle: Option<DeclarationId>,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let modes = self.fixture_modes(instance, at, contract, env)?;
        let mut nodes = Vec::new();
        let integration = if let Some(data) = &contract.integration {
            let axis = self.model.integrated.values().next().ok_or_else(|| {
                invalid(
                    at,
                    "integrated fixture requires the integrated analysis route and a time axis",
                )
            })?;
            let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(axis.quantity));
            let lower = axis.lower;
            let upper = axis.upper;
            let samples = data
                .samples
                .iter()
                .enumerate()
                .map(|(position, _)| {
                    self.eval_field(
                        at,
                        env,
                        "scope.fixture.integration.samples",
                        position,
                        Some(&ty),
                    )?
                    .scalar(at)
                })
                .collect::<Result<Vec<_>>>()?;
            let initial_step = self
                .eval_field(
                    at,
                    env,
                    "scope.fixture.integration.initial_step",
                    0,
                    Some(&ty),
                )?
                .scalar(at)?;
            if !initial_step.is_finite()
                || initial_step <= 0.
                || samples
                    .iter()
                    .any(|s| !s.is_finite() || *s < lower || *s > upper)
                || samples.windows(2).any(|s| s[0] >= s[1])
            {
                return Err(invalid(
                    at,
                    "integration samples must increase within the declared axis and the step must be positive",
                ));
            }
            let mut quadratures = BTreeMap::new();
            for (position, _entry) in data.quadratures.iter().enumerate() {
                let targets = self.annotation_targets(
                    instance,
                    &self
                        .p
                        .expression_at(
                            at,
                            "scope.fixture.integration.quadratures.target",
                            position,
                        )?
                        .clone(),
                    env,
                    at,
                    false,
                )?;
                let [(target, ty, local)] = targets.as_slice() else {
                    return Err(invalid(at, "one terminal integral per tolerance"));
                };
                // Authored lets retain their identity for diagnostics. Follow only
                // exact symbol aliases to the integrated coordinate; a sum or scaled
                // expression cannot define the integrator's absolute tolerance.
                let target = self
                    .model
                    .integral_of(*target)
                    .ok_or_else(|| invalid(at, "quadrature tolerance target is not an integral"))?;
                let value = self
                    .eval_field(
                        at,
                        local,
                        "scope.fixture.integration.quadratures.absolute_tolerance",
                        position,
                        Some(ty),
                    )?
                    .scalar(at)?;
                if !value.is_finite() || value <= 0. || quadratures.insert(target, value).is_some()
                {
                    return Err(invalid(
                        at,
                        "quadrature tolerances must be unique positive physical values",
                    ));
                }
            }
            if quadratures.keys().ne(self.model.integrals.keys()) {
                return Err(invalid(
                    at,
                    "each integrated quadrature requires an explicit physical tolerance",
                ));
            }
            let end = samples.last().copied().unwrap_or(lower);
            let mut schedules = Vec::with_capacity(data.schedules.len());
            for (schedule, entry) in data.schedules.iter().enumerate() {
                let targets = self.annotation_targets(
                    instance,
                    &self
                        .p
                        .expression_at(
                            at,
                            &format!("scope.fixture.integration.schedules.{schedule}.target"),
                            0,
                        )?
                        .clone(),
                    env,
                    at,
                    false,
                )?;
                let [(target, target_ty, local)] = targets.as_slice() else {
                    return Err(invalid(at, "a scheduled input names one indexed scalar"));
                };
                if !self.model.symbols.get(target).is_some_and(|symbol| {
                    symbol.expression.is_none()
                        && matches!(symbol.role, Kind::Variable | Kind::Parameter)
                        && matches!(target_ty, Type::Quantity(_))
                }) {
                    return Err(invalid(
                        at,
                        "a scheduled input must be an independent physical variable or parameter",
                    ));
                }
                if schedules
                    .iter()
                    .any(|s: &ScheduleFixture| s.target == *target)
                {
                    return Err(invalid(at, "one schedule per input"));
                }
                let times = entry
                    .times
                    .iter()
                    .enumerate()
                    .map(|(position, _)| {
                        self.eval_field(
                            at,
                            env,
                            &format!("scope.fixture.integration.schedules.{schedule}.times"),
                            position,
                            Some(&ty),
                        )?
                        .scalar(at)
                    })
                    .collect::<Result<Vec<_>>>()?;
                let values = entry
                    .values
                    .iter()
                    .enumerate()
                    .map(|(position, _)| {
                        self.eval_field(
                            at,
                            local,
                            &format!("scope.fixture.integration.schedules.{schedule}.values"),
                            position,
                            Some(target_ty),
                        )?
                        .scalar(at)
                    })
                    .collect::<Result<Vec<_>>>()?;
                if times.is_empty()
                    || values.len() != times.len() + 1
                    || times
                        .iter()
                        .any(|t| !t.is_finite() || *t <= lower || *t > end)
                    || times.windows(2).any(|w| w[0] >= w[1])
                    || values.iter().any(|v| !v.is_finite())
                {
                    return Err(invalid(
                        at,
                        "schedule changes must increase after the axis's lower bound up to the last sample, with one finite value per interval",
                    ));
                }
                // A schedule held free is a shooting control within its bounds, starting
                // from its authored values (ADR-0110 Outcome 5).
                let control = if entry.free {
                    let bound = |role: &str, text: &Option<String>| {
                        text.as_ref()
                            .map(|_| {
                                self.eval_field(
                                    at,
                                    local,
                                    &format!(
                                        "scope.fixture.integration.schedules.{schedule}.{role}"
                                    ),
                                    0,
                                    Some(target_ty),
                                )?
                                .scalar(at)
                            })
                            .transpose()
                    };
                    let control = ScheduleControl {
                        lower: bound("lower", &entry.lower)?,
                        upper: bound("upper", &entry.upper)?,
                    };
                    let lower = control.lower.unwrap_or(f64::NEG_INFINITY);
                    let upper = control.upper.unwrap_or(f64::INFINITY);
                    if lower.is_nan()
                        || upper.is_nan()
                        || lower > upper
                        || values.iter().any(|v| *v < lower || *v > upper)
                    {
                        return Err(invalid(
                            at,
                            "a free schedule's bounds are ordered and hold its starting values",
                        ));
                    }
                    Some(control)
                } else {
                    None
                };
                self.reserve(1)?;
                schedules.push(ScheduleFixture {
                    target: *target,
                    times,
                    values,
                    control,
                });
            }
            nodes = contract
                .shooting
                .iter()
                .flat_map(|s| &s.nodes)
                .enumerate()
                .map(|(position, _)| {
                    self.eval_field(at, env, "scope.fixture.shooting.nodes", position, Some(&ty))?
                        .scalar(at)
                })
                .collect::<Result<Vec<_>>>()?;
            if nodes.windows(2).any(|w| w[0] >= w[1])
                || nodes
                    .iter()
                    .any(|t| !t.is_finite() || *t <= lower || *t >= end)
            {
                return Err(invalid(
                    at,
                    "multiple shooting nodes increase strictly inside the horizon",
                ));
            }
            Some(IntegrationFixture {
                samples,
                quadratures,
                schedules,
                quadrature_relative_tolerance: data.quadrature_relative_tolerance,
                initial_step,
                relative_tolerance: data.relative_tolerance,
                normalized_absolute_tolerance: data.normalized_absolute_tolerance,
            })
        } else {
            None
        };
        // Expected diagnostics name members by path; each resolves once to every coordinate
        // of an equation or variable member of the fixture's instance.
        let mut diagnostics = Vec::with_capacity(contract.diagnostics.len());
        for (diagnostic, expected) in contract.diagnostics.iter().enumerate() {
            if expected.members.is_empty() {
                return Err(invalid(
                    at,
                    "an expected diagnostic names its rule and members",
                ));
            }
            let mut members = Vec::with_capacity(expected.members.len());
            for (position, path) in expected.members.iter().enumerate() {
                let ids = self
                    .annotation_targets(
                        instance,
                        &self
                            .p
                            .expression_at(
                                at,
                                &format!("scope.fixture.diagnostics.{diagnostic}.members"),
                                position,
                            )?
                            .clone(),
                        env,
                        at,
                        false,
                    )?
                    .into_iter()
                    .map(|(id, ..)| id)
                    .filter(|id| {
                        self.model.symbols.contains_key(id)
                            || self.model.equations.iter().any(|row| row.id == *id)
                    })
                    .collect::<BTreeSet<_>>();
                if ids.is_empty() {
                    return Err(invalid(
                        at,
                        format!("expected diagnostic member {path} names no equation or variable"),
                    ));
                }
                members.push((path.clone(), ids));
            }
            self.reserve(1 + members.len())?;
            diagnostics.push(FixtureDiagnostic {
                rule: expected.rule,
                members,
            });
        }
        let mut specifications = BTreeMap::<String, FixtureValue>::new();
        let mut seen = BTreeSet::new();
        for (position, s) in contract.specifications.iter().enumerate() {
            let targets = self.annotation_targets(
                instance,
                &self
                    .p
                    .expression_at(at, "scope.fixture.specifications.target", position)?
                    .clone(),
                env,
                at,
                false,
            )?;
            if targets.len() != 1 {
                return Err(invalid(
                    at,
                    "fixture specifications require one indexed scalar path",
                ));
            }
            let (target, _, local) = targets
                .into_iter()
                .next()
                .ok_or_else(|| invalid(at, "empty fixture target"))?;
            let symbol = self.model.symbols.get(&target).ok_or_else(|| {
                invalid(at, "fixture target must be an independent physical symbol")
            })?;
            if symbol.expression.is_some()
                || !matches!(symbol.role, Kind::Variable | Kind::Parameter)
            {
                return Err(invalid(
                    at,
                    "fixture target must be an independent variable or parameter",
                ));
            }
            if s.kind != Binding::Value && symbol.role != Kind::Variable {
                return Err(invalid(at, "fixture fixedness/bounds require a variable"));
            }
            let ty = symbol.ty.clone();
            let quantity = ty
                .quantity_scheme()
                .ok_or_else(|| invalid(at, "fixture requires a physical target"))?
                .clone();
            let key = if s.kind == Binding::Free {
                Binding::Fix
            } else {
                s.kind
            };
            if !seen.insert((target, key)) {
                return Err(invalid(at, "duplicate fixture specification"));
            }
            if s.kind == Binding::Free && s.expression.is_some() {
                return Err(invalid(at, "free fixture binding takes no value"));
            }
            let value = s
                .expression
                .as_ref()
                .map(|_| {
                    if ty.physical_refinement().is_some() {
                        let expression = self
                            .p
                            .expression_at(at, "scope.fixture.specifications.expression", position)?
                            .clone();
                        let expression = self.rewrite(instance, &expression, &local, &[at])?;
                        let types = self
                            .model
                            .symbols
                            .iter()
                            .map(|(id, symbol)| (symbol_name(*id), symbol.ty.clone()))
                            .collect();
                        if crate::expression::infer(
                            &expression,
                            &types,
                            &self.model.function_contracts(self.p),
                            self.c,
                            at,
                            Some(&ty),
                        )? != ty
                        {
                            return Err(invalid(at, "fixture physical type differs from target"));
                        }
                    }
                    // Fixture data changes this existing symbol's numeric binding;
                    // its admitted nominal owner remains on the symbol, not Value.
                    self.eval_field(
                        at,
                        &local,
                        "scope.fixture.specifications.expression",
                        position,
                        Some(&Type::Quantity(quantity)),
                    )?
                    .scalar(at)
                })
                .transpose()?;
            if matches!(s.kind, Binding::Value | Binding::Lower | Binding::Upper) && value.is_none()
            {
                return Err(invalid(at, "fixture binding requires a physical value"));
            }
            if value.is_some_and(|v| !v.is_finite()) {
                return Err(invalid(at, "nonfinite fixture value"));
            }
            if s.kind == Binding::Fix && value.is_some() && !seen.insert((target, Binding::Value)) {
                return Err(invalid(at, "duplicate fixture value through aliased paths"));
            }
            let path = if self.states[&instance].parent.is_none() {
                s.target.clone()
            } else {
                format!("{}.{}", self.states[&instance].path, s.target)
            };
            self.model.paths.insert(path.clone(), target);
            let entry = specifications.entry(path).or_insert(FixtureValue {
                target,
                value: None,
                fixed: None,
                lower: None,
                upper: None,
            });
            match s.kind {
                Binding::Value => {
                    if entry.value.is_some() {
                        return Err(invalid(at, "duplicate fixture value"));
                    }
                    entry.value = value;
                }
                Binding::Fix => {
                    entry.fixed = Some(true);
                    if value.is_some() {
                        if entry.value.is_some() {
                            return Err(invalid(at, "duplicate fixture value"));
                        }
                        entry.value = value;
                    }
                }
                Binding::Free => entry.fixed = Some(false),
                Binding::Lower => entry.lower = value,
                Binding::Upper => entry.upper = value,
            }
        }
        for s in specifications.values() {
            if s.lower.zip(s.upper).is_some_and(|(a, b)| a > b) {
                return Err(invalid(at, "reversed fixture bounds"));
            }
        }
        let expected_failure = contract
            .expected_failure
            .as_ref()
            .map(|expected| self.expected_failure(instance, at, expected, env))
            .transpose()?;
        self.reserve(specifications.len() + 1)?;
        let declared = crate::analysis::declared_policy(at, Some(contract))?;
        let endpoint = match contract.endpoint.as_ref().and_then(|e| e.event.as_ref()) {
            None => FixtureEndpoint::FixedHorizon,
            Some(name) => {
                let selected = modes
                    .iter()
                    .flat_map(|m| &m.events)
                    .filter(|e| &e.name == name)
                    .collect::<Vec<_>>();
                let [event] = selected.as_slice() else {
                    return Err(invalid(
                        at,
                        "endpoint names exactly one declared terminal event",
                    ));
                };
                if event.next.is_some() || !event.reset.is_empty() {
                    return Err(invalid(
                        at,
                        "declared terminal endpoint cannot reset or select a successor mode",
                    ));
                }
                FixtureEndpoint::DeclaredTerminalEvent(event.guard)
            }
        };
        self.model.fixtures.insert(
            instance,
            Fixture {
                declaration: at,
                expected_degrees_of_freedom: contract.degrees_of_freedom,
                specifications,
                oracle,
                route: declared.route,
                procedure: declared.procedure,
                endpoint,
                intent: contract.intent,
                stages: contract.stages.clone(),
                initialization: contract.initialization.clone(),
                integration,
                modes,
                shooting: contract.shooting.as_ref().map(|s| ShootingFixture {
                    method: s.method,
                    nodes,
                }),
                expected_failure,
                diagnostics,
            },
        );
        Ok(())
    }
    /// Resolve an expected failure's lineage in the fixture's scope (Plan 23 H5): a form to
    /// its function, its arguments to their positions and its parameter sets, evaluated
    /// statically, to their identities; members to the variables and equations they name.
    fn expected_failure(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        expected: &pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixtureExpectedFailure,
        env: &Environment,
    ) -> Result<ExpectedFailure> {
        let lineage = if let Some(applicability) = &expected.applicability {
            let claim = self
                .p
                .resolve(at, &applicability.claim)
                .filter(|id| self.p.declarations[id].value.applicability.is_some())
                .ok_or_else(|| invalid(at, "expected applicability claim is not a declaration"))?;
            let form = self
                .p
                .resolve(at, &applicability.form)
                .filter(|id| self.p.functions.contains_key(id))
                .ok_or_else(|| invalid(at, "expected applicability form is not a function"))?;
            let mut sets = applicability
                .sets
                .iter()
                .enumerate()
                .map(|(position, _)| {
                    let value = self.eval_field(
                        at,
                        env,
                        "scope.fixture.expected_failure.applicability.sets",
                        position,
                        None,
                    )?;
                    self.p
                        .set_identity(&value)
                        .ok_or_else(|| invalid(at, "expected applicability set is not a record"))
                })
                .collect::<Result<Vec<_>>>()?;
            sets.sort_unstable();
            let mut variables = applicability.variables.clone();
            variables.sort();
            if variables.iter().any(|name| {
                !self.p.functions[&claim]
                    .arguments
                    .iter()
                    .any(|(formal, ty)| formal == name && ty.quantity_scheme().is_some())
            }) {
                return Err(invalid(
                    at,
                    "expected applicability variable is not a numerical claim argument",
                ));
            }
            ExpectedLineage::Applicability {
                layer: applicability.layer,
                outcome: applicability.outcome,
                claim,
                form,
                sets,
                variables,
            }
        } else if let Some(validity) = &expected.validity {
            // A closure range names the members it bounds (Plan 23 H5).
            let Some(named) = &validity.form else {
                let mut members = Vec::new();
                for (position, path) in validity.variables.iter().enumerate() {
                    members.extend(self.members(
                        instance,
                        at,
                        path,
                        "scope.fixture.expected_failure.validity.variables",
                        position,
                        env,
                    )?);
                }
                members.sort_unstable();
                members.dedup();
                return Ok(ExpectedFailure {
                    class: expected.class,
                    lineage: ExpectedLineage::Validity {
                        layer: validity.layer,
                        form: None,
                        sets: Vec::new(),
                        variables: Vec::new(),
                        members,
                    },
                });
            };
            let form = self
                .p
                .resolve(at, named)
                .filter(|id| self.p.functions.contains_key(id))
                .ok_or_else(|| {
                    invalid(
                        at,
                        format!("expected failure names form {named}, which is not a function"),
                    )
                })?;
            let arguments = &self.p.functions[&form].arguments;
            let mut variables = validity
                .variables
                .iter()
                .map(|name| {
                    arguments
                        .iter()
                        .position(|(argument, _)| argument == name)
                        .map(|position| position as u32)
                        .ok_or_else(|| {
                            invalid(
                                at,
                                format!(
                                    "expected failure names {name}, which is not an argument of form {named}"
                                ),
                            )
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            variables.sort_unstable();
            let mut sets = validity
                .sets
                .iter()
                .enumerate().map(|(position, text)| {
                    let value = self.eval_field(at, env, "scope.fixture.expected_failure.validity.sets", position, None)?;
                    self.p.set_identity(&value).ok_or_else(|| {
                        invalid(
                            at,
                            format!("expected failure names set {text}, which is not a table row or an entity"),
                        )
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            sets.sort_unstable();
            ExpectedLineage::Validity {
                layer: validity.layer,
                form: Some(form),
                sets,
                variables,
                members: Vec::new(),
            }
        } else {
            let mut members = BTreeSet::new();
            for (position, path) in expected.members.iter().enumerate() {
                members.extend(self.members(
                    instance,
                    at,
                    path,
                    "scope.fixture.expected_failure.members",
                    position,
                    env,
                )?);
            }
            ExpectedLineage::Members(members)
        };
        Ok(ExpectedFailure {
            class: expected.class,
            lineage,
        })
    }
    /// Variables, equations, or the exact equality rows of a named connection.
    fn members(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        path: &str,
        role: &str,
        position: usize,
        env: &Environment,
    ) -> Result<Vec<SemanticId>> {
        let expression = self.p.expression_at(at, role, position)?.clone();
        if let ExprKind::Path(member_path) = &expression.kind
            && let Some((last, prefix)) = member_path.segments.split_last()
        {
            let mut owner = instance;
            for segment in prefix {
                owner = if segment.name == "parent" && segment.indices.is_empty() {
                    self.states[&owner]
                        .parent
                        .ok_or_else(|| invalid(at, "root has no parent"))?
                } else {
                    self.child_instance(owner, at, segment, env)?
                };
            }
            if let Some(member) = self.states[&owner].members.get(&last.name).copied()
                && self.p.declarations[&member].value.kind == Kind::Connection
            {
                let occurrence = if last.indices.is_empty() {
                    None
                } else {
                    let (owner, member, coordinates) =
                        self.resolve_path(instance, at, member_path, env, true)?;
                    Some(self.connection_occurrence_id(
                        owner,
                        &self.p.declarations[&member],
                        &coordinates,
                    ))
                };
                let rows = self
                    .model
                    .connections
                    .values()
                    .filter(|connection| {
                        connection.lineage.instance == owner
                            && connection.lineage.declaration == member
                            && occurrence.is_none_or(|id| connection.id == id)
                    })
                    .flat_map(|connection| connection.rows.iter().copied())
                    .collect::<Vec<_>>();
                if rows.is_empty() {
                    return Err(invalid(
                        at,
                        format!(
                            "expected failure names connection {path}, which has no active equality rows"
                        ),
                    ));
                }
                return Ok(rows);
            }
        }
        let targets = self.annotation_targets(instance, &expression, env, at, false)?;
        if targets.is_empty() {
            return Err(invalid(
                at,
                format!("expected failure names member {path}, which the fixture's model lacks"),
            ));
        }
        Ok(targets.into_iter().map(|(id, ..)| id).collect())
    }
}
