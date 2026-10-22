// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepare model-issued typed regions and bind their semantic coordinates into CasePlan.
use super::*;
use pse_kernels::Port;
use pse_math::binding::{Contribution, InstanceBinding, Row, SlotBinding, Variable};

/// Exact compact resolved owner. Instance attribution and actual input bindings live
/// in PlannedInstance; this reusable body holds only canonical typed coordinates.
#[derive(Clone, Debug)]
struct BodyInput {
    identity: pse_ids::roles::SemanticBodyHash,
    syntax: pse_modeling::resolved::ResolvedSyntax,
    inputs: Vec<pse_modeling::resolved::SymbolIx>,
    support: pse_modeling::resolved::PhysicalSupport,
    providers: crate::typed_math::SelectedProviders,
    physical: ContentHash,
    limits: BodyLimits,
}
impl PartialEq for BodyInput {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}
impl Eq for BodyInput {}
impl std::hash::Hash for BodyInput {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        std::hash::Hash::hash(&self.identity, h);
    }
}
#[salsa::interned(heap_size = body_key_heap)]
struct BodyKey<'db> {
    input: Arc<BodyInput>,
}
fn body_key_heap((input,): &(Arc<BodyInput>,)) -> usize {
    size_of::<BodyInput>()
        .saturating_add(input.syntax.retained_bytes())
        .saturating_add(size_of_val(input.inputs.as_slice()))
        .saturating_add(input.support.retained_bytes())
        .saturating_add(input.providers.retained_bytes())
}
#[salsa::tracked(returns(clone), lru = 64, heap_size = body_heap)]
fn semantic_body(db: &dyn CompilerDb, key: BodyKey<'_>) -> Result<Arc<AdmittedBody>> {
    checkpoint(db);
    let input = key.input(db);
    let retained = match db.body_retention() {
        Some(cache) => match cache.get(input.identity) {
            Ok(body) => body,
            Err(error) => body_refused(db, error),
        },
        None => None,
    };
    if let Some(body) = retained {
        let _reuse = tracing::info_span!(
            "pse.case.semantic_body_reuse",
            product = "body",
            derivative_order = "value",
            reused = true
        )
        .entered();
        if body.semantic_identity() != Some(input.identity) || body.spec.physical != input.physical
        {
            body_refused(db, MathError::Contract("retained body has a different immutable dependency closure for its exact resolved product".into()));
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("compiler.semantic_body_retention_reuses", "count", 1);
        }
        admit_live_body(
            db,
            input.identity,
            body.descriptor_bytes()
                .saturating_add(body.math.retained_bytes()),
        );
        return Ok(body);
    }
    let _phase = pse_diagnostics::observation::current().map(|context| {
        context.count("compiler.semantic_body_admissions", "count", 1);
        if db.body_retention().is_some() {
            context.unavailable(
                "compiler.body_retention_miss_reason",
                "retention does not report a miss cause",
            );
        } else {
            context.source_ref("compiler.body_retention", b"not attached");
        }
        context.phase("compiler.semantic_body_admission")
    });
    let admission = tracing::info_span!(
        "pse.case.semantic_body_admission",
        product = "body",
        derivative_order = "value",
        success = false
    );
    let _admission = admission.enter();
    let generation = db.body_retention().map(|cache| cache.generation());
    let body = Arc::new(materialize(input, db.cancel())?.for_semantic_identity(input.identity)?);
    checkpoint(db);
    admit_live_body(
        db,
        input.identity,
        body.descriptor_bytes()
            .saturating_add(body.math.retained_bytes()),
    );
    admission.record("success", true);
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count("compiler.semantic_body_admissions_completed", "count", 1);
    }
    if let Some(cache) = db.body_retention() {
        return match cache.retain(generation.unwrap_or(u64::MAX), input.identity, body.clone()) {
            Ok(retained) if retained.as_ref() == body.as_ref() => Ok(retained),
            Ok(_) => body_refused(
                db,
                MathError::Contract("retention changed admitted mathematical meaning".into()),
            ),
            Err(error) => body_refused(db, error),
        };
    }
    Ok(body)
}
fn materialize(input: &BodyInput, cancel: &Arc<AtomicBool>) -> Result<AdmittedBody> {
    let _phase = pse_diagnostics::observation::phase("compiler.body_materialization");
    let (registry, facts) = input.support.context()?;
    Ok(crate::typed_math::Request {
        definition: SemanticId::NIL,
        syntax: &input.syntax,
        inputs: &input.inputs,
        support: &input.support,
        providers: &input.providers,
        physical: input.physical,
        structure: input.identity.as_id(),
        limits: input.limits,
    }
    .admit(registry, facts, cancel)?)
}
fn body_heap(value: &Result<Arc<AdmittedBody>>) -> usize {
    value
        .as_ref()
        .map_or_else(CompileError::retained_bytes, |b| {
            b.descriptor_bytes() + b.math.retained_bytes()
        })
}
/// The grouped producer supplies this exact structure field to materialize().
/// An unportable fallback still occupies its input's Salsa memo. This locator is
/// capacity bookkeeping only and never grants semantic-body reuse eligibility.
pub(super) fn memo_key(body: &AdmittedBody) -> pse_ids::roles::SemanticBodyHash {
    body.semantic_identity()
        .unwrap_or_else(|| pse_ids::roles::SemanticBodyHash::from_id(body.spec.structure))
}

pub(super) fn configure(db: &mut CompilerDatabase, n: usize) {
    semantic_body::set_lru_capacity(db, n);
}

impl ModelingOutput {
    /// Stable output-row identity; presentation order is independent of assembly order.
    pub fn row_id(&self) -> SemanticId {
        use pse_model::generated::enums::ModelingAnnotationKind as Annotation;
        use pse_modeling::derived::{
            AnnotationComponent as Component, Relationship as R, TestComponent as Test,
        };
        match self {
            Self::Equation { id, .. } => *id,
            Self::LevelBound { row, .. } => *row,
            Self::ConditionalBoundary(symbol) => {
                R::ConditionalBoundaryResidual { symbol: *symbol }.identity()
            }
            Self::Inventory(balance) => {
                R::ConservedInventoryOutput { balance: *balance }.identity()
            }
            Self::InventoryTransfer { balance, event } => R::InventoryTransferOutput {
                balance: *balance,
                event: *event,
            }
            .identity(),
            Self::InitialState { state, equation } => R::InitialStateOutput {
                state: *state,
                equation: *equation,
            }
            .identity(),
            Self::DynamicRate { state, .. } => R::DynamicRateOutput { state: *state }.identity(),
            Self::Test { id, component } => R::TestOutput {
                test: *id,
                component: match component {
                    ModelingTestValue::Actual => Test::Actual,
                    ModelingTestValue::Expected => Test::Expected,
                    ModelingTestValue::Tolerance => Test::Tolerance,
                    ModelingTestValue::RelativeTolerance => Test::RelativeTolerance,
                },
            }
            .identity(),
            Self::Hint {
                target,
                declaration,
                kind,
            } => {
                let (kind, component) = match kind {
                    ModelingHint::Start => (Annotation::Start, Component::Value),
                    ModelingHint::Nominal => (Annotation::Nominal, Component::Value),
                    ModelingHint::Lower => (Annotation::Bounds, Component::Lower),
                    ModelingHint::Upper => (Annotation::Bounds, Component::Upper),
                    ModelingHint::ValidLower => (Annotation::Valid, Component::Lower),
                    ModelingHint::ValidUpper => (Annotation::Valid, Component::Upper),
                    ModelingHint::Check => (Annotation::Check, Component::Value),
                    ModelingHint::ObjectiveBound(ObjectiveBound::Lower { strict }) => (
                        Annotation::Check,
                        Component::ObjectiveLower { strict: *strict },
                    ),
                    ModelingHint::ObjectiveBound(ObjectiveBound::Upper { strict }) => (
                        Annotation::Check,
                        Component::ObjectiveUpper { strict: *strict },
                    ),
                    ModelingHint::AccuracyGoalTime => (Annotation::AccuracyGoal, Component::Time),
                    ModelingHint::AccuracyGoalResolution => {
                        (Annotation::AccuracyGoal, Component::Resolution)
                    }
                    ModelingHint::AccuracyGoalLower => {
                        (Annotation::AccuracyGoal, Component::CriterionLower)
                    }
                    ModelingHint::AccuracyGoalUpper => {
                        (Annotation::AccuracyGoal, Component::CriterionUpper)
                    }
                    ModelingHint::EngineeringScaleValue => {
                        (Annotation::EngineeringScale, Component::Value)
                    }
                };
                R::AnnotationOutput {
                    target: *target,
                    declaration: *declaration,
                    kind,
                    component,
                }
                .identity()
            }
            Self::OriginalEquation(equation) => R::OriginalEquationResidual {
                equation: *equation,
            }
            .identity(),
            Self::Penalty(equation) => R::ElasticPenalty {
                equation: *equation,
            }
            .identity(),
            Self::Term {
                equation, ordinal, ..
            } => R::OriginalTerm {
                equation: *equation,
                ordinal: *ordinal as u64,
            }
            .identity(),
            Self::Member(symbol) => R::MemberOutput { symbol: *symbol }.identity(),
            Self::Contribution { contribution, .. } => R::ContributionOutput {
                contribution: *contribution,
            }
            .identity(),
        }
    }
}
impl ModelingOutput {
    /// The selected constraint row this output is, with its sense: an equation residual,
    /// or a generated objective bound. Every other output is an observation.
    pub fn constraint(&self) -> Option<(SemanticId, EquationSense)> {
        match self {
            Self::ConditionalBoundary(_) => Some((self.row_id(), EquationSense::Eq)),
            Self::Equation { id, sense } => Some((*id, *sense)),
            Self::LevelBound { row, sense, .. } => Some((*row, *sense)),
            _ => None,
        }
    }
}
impl AdmittedModeling {
    /// Derive selected equation scales from the original term outputs at an explicit
    /// nominal point. These values never come from a simplified residual.
    /// # Errors
    /// Absent rows, invalid observations or an unrepresentable scale.
    pub fn derived_scales(
        &self,
        schemes: &BTreeMap<SemanticId, pse_model::generated::enums::ConstraintScalingScheme>,
        values: &[f64],
    ) -> Result<BTreeMap<SemanticId, f64>> {
        if values.len() != self.case.rows().len() {
            return Err(CompileError::Missing("nominal observation extent".into()));
        }
        let rows = self
            .case
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect::<BTreeMap<_, _>>();
        schemes
            .iter()
            .map(|(id, scheme)| {
                let sources = self.term_outputs.get(id).ok_or_else(|| {
                    CompileError::Missing("original terms for scaling target".into())
                })?;
                let terms = sources
                    .iter()
                    .map(|(row, sign)| {
                        rows.get(row)
                            .map(|i| sign * values[*i])
                            .ok_or_else(|| CompileError::Missing("original term row".into()))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok((*id, pse_math::numerics::term_scale(*scheme, &terms)?))
            })
            .collect()
    }
    /// Use the existing sparse assembly and derivative-demand owner.
    /// # Errors
    /// Physical mismatch, unavailable derivatives, cancellation or resource refusal.
    pub fn plan(
        &self,
        registry: &QuantityRegistry,
        order: DerivativeOrder,
        limits: AssemblyLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<CasePlan>> {
        Ok(Arc::new(CasePlan::prepare(
            self.case.clone(),
            self.bodies
                .iter()
                .map(|(k, b)| (*k, b.math.clone()))
                .collect(),
            registry,
            order,
            limits,
            cancel,
        )?))
    }
    /// Restore public semantic output order after the assembly's deterministic row ordering.
    /// # Errors
    /// A result does not match this immutable structure.
    pub fn ordered_values(&self, values: &[f64]) -> Result<Vec<f64>> {
        if values.len() != self.case.rows().len() {
            return Err(CompileError::Missing("modeling result extent".into()));
        }
        let rows = self
            .case
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect::<BTreeMap<_, _>>();
        self.outputs
            .iter()
            .map(|o| {
                rows.get(&o.row_id())
                    .map(|i| values[*i])
                    .ok_or_else(|| CompileError::Missing("modeling output row".into()))
            })
            .collect()
    }
}

/// Owned assembly instructions. No Salsa identity escapes the planning operation.
#[derive(Clone, Debug)]
pub(super) struct PlannedModeling {
    inputs: Vec<SemanticId>,
    outputs: Vec<ModelingOutput>,
    term_outputs: BTreeMap<SemanticId, Vec<(SemanticId, f64)>>,
    pub(super) implicit: BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
    instances: Vec<PlannedInstance>,
    variables: Vec<Variable>,
    parameters: Vec<Port>,
    rows: Vec<Row>,
    objective: Option<pse_math::binding::Objective>,
    levels: Vec<(
        pse_math::binding::Objective,
        Option<pse_math::binding::Degradation>,
    )>,
    native: Vec<pse_model::forms::NativeConstraint>,
    requirements: BTreeSet<pse_model::generated::enums::ModelingStructuralRequirement>,
}
#[derive(Clone, Debug)]
struct PlannedInstance {
    input: Arc<BodyInput>,
    instance: SemanticId,
    checked_members: BTreeMap<SemanticId, SemanticId>,
    checked_sources: BTreeMap<SemanticId, SemanticId>,
    occurrences: BTreeMap<SemanticId, pse_model::diagnostic::SourceLocation>,
    slots: Vec<SlotBinding>,
    contributions: Vec<Contribution>,
}
impl PlannedModeling {
    pub(super) fn body_requests(
        &self,
    ) -> impl Iterator<Item = pse_ids::roles::SemanticBodyHash> + '_ {
        self.instances
            .iter()
            .map(|instance| instance.input.identity)
    }
    pub(super) fn providers(&self) -> BTreeMap<ContentHash, pse_kernels::AdmittedProvider> {
        self.instances
            .iter()
            .flat_map(|instance| instance.input.providers.descriptors())
            .map(|(identity, descriptor)| (*identity, descriptor.clone()))
            .collect()
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + size_of_val(self.inputs.as_slice())
            + self
                .outputs
                .iter()
                .map(|output| {
                    size_of_val(output)
                        + match output {
                            ModelingOutput::DynamicRate { equations, .. } => {
                                size_of_val(equations.as_slice())
                            }
                            _ => 0,
                        }
                })
                .sum::<usize>()
            + self
                .term_outputs
                .values()
                .map(|rows| size_of_val(rows.as_slice()) + 128)
                .sum::<usize>()
            + self
                .implicit
                .values()
                .map(|supplier| supplier.retained_bytes())
                .sum::<usize>()
            + self
                .instances
                .iter()
                .map(|instance| {
                    size_of_val(instance)
                        + body_key_heap(&(instance.input.clone(),))
                        + (instance.checked_members.len() + instance.checked_sources.len())
                            * (size_of::<(SemanticId, SemanticId)>() + 96)
                        + instance
                            .occurrences
                            .values()
                            .map(|source| {
                                size_of::<pse_model::diagnostic::SourceLocation>()
                                    + 128
                                    + source.path.capacity()
                                    + source.name.as_ref().map_or(0, String::capacity)
                                    + source.occurrence_role.as_ref().map_or(0, String::capacity)
                            })
                            .sum::<usize>()
                        + size_of_val(instance.slots.as_slice())
                        + size_of_val(instance.contributions.as_slice())
                })
                .sum::<usize>()
            + size_of_val(self.variables.as_slice())
            + size_of_val(self.parameters.as_slice())
            + size_of_val(self.rows.as_slice())
            + size_of_val(self.levels.as_slice())
            + self
                .native
                .iter()
                .map(|constraint| {
                    64 + constraint.identities().len() * (size_of::<SemanticId>() + 16)
                })
                .sum::<usize>()
            + self.requirements.len() * 96
    }
}

pub(super) fn admit(
    db: &dyn CompilerDb,
    inventory: Inventory,
    p: &Projection,
) -> Result<Arc<AdmittedModeling>> {
    complete(db, inventory, plan(db, inventory, p, BTreeMap::new())?)
}

pub(super) fn plan(
    db: &dyn CompilerDb,
    inventory: Inventory,
    p: &Projection,
    supplied: BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
) -> Result<PlannedModeling> {
    let (implicit, direct_providers) = implicit::admit(db, inventory, p, supplied)?;
    let registry = inventory.quantities(db);
    let checker = inventory.preconditions(db);
    let mut ports = BTreeMap::new();
    let mut variables = vec![];
    let mut parameters = vec![];
    for (id, formal) in p.inputs.iter().zip(&p.formals) {
        let port = Port {
            id: *id,
            quantity: formal.quantity,
            unit: registry
                .quantity_type(formal.quantity)
                .map_err(MathError::from)?
                .canonical_unit,
        };
        ports.insert(*id, port.clone());
        if let Some(domain) = p.free.get(id).copied() {
            // A binary decision or a lowering's convex weight lives in the unit box;
            // cases may only narrow it.
            let unit = domain == pse_model::generated::enums::ModelingVariableDomain::Binary
                || p.unit_interval.contains(id);
            variables.push(Variable {
                port,
                fixed: false,
                domain,
                lower: (unit || p.nonnegative.contains(id)).then_some(0.0),
                upper: unit.then_some(1.0),
            });
        } else {
            parameters.push(port);
        }
    }
    let penalty = p
        .outputs
        .iter()
        .any(|o| matches!(o, ModelingOutput::Penalty(_)));
    // The solved level's members contribute their scales to the objective, and every
    // bounded level's members to its bound row (ADR-0111). Several levels without a
    // selected one are optimized together by a native lexicographic route: each level's
    // members contribute to that level's objective.
    let lexicographic = p.objectives.levels.len() > 1 && p.objectives.selected.is_none();
    let mut objective_scales = BTreeMap::<SemanticId, Vec<(Target, f64)>>::new();
    let solved = p.objectives.solved();
    for (position, level) in p.objectives.levels.iter().enumerate() {
        let target = if lexicographic {
            Some(Target::Objective(position))
        } else {
            solved
                .is_some_and(|solved| std::ptr::eq(solved, level))
                .then_some(Target::PRIMARY)
        };
        for member in p.objectives.members_of(level) {
            let targets = objective_scales.entry(member.term).or_default();
            if let Some(target) = target {
                targets.push((target, member.scale));
            }
            if let Some(bound) = &level.bound {
                targets.push((Target::Row(bound.row), member.scale));
            }
        }
    }
    let levels = if lexicographic {
        if penalty {
            return Err(CompileError::Missing(
                "elastic penalties need one objective; select a lexicographic level".into(),
            ));
        }
        let last = p.objectives.levels.len() - 1;
        p.objectives
            .levels
            .iter()
            .enumerate()
            .map(|(position, level)| {
                let degradation = match (
                    position < last,
                    level.absolute_tolerance,
                    level.relative_tolerance,
                ) {
                    (false, _, _) => None,
                    (true, Some(absolute), Some(relative)) => {
                        Some(pse_math::binding::Degradation { absolute, relative })
                    }
                    (true, _, _) => {
                        return Err(CompileError::Missing(
                            "an earlier lexicographic level declares both tolerances".into(),
                        ));
                    }
                };
                Ok((
                    pse_math::binding::Objective {
                        quantity: level.quantity,
                        sense: objective_sense(level.sense),
                    },
                    degradation,
                ))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let objective = if let Some(level) = solved {
        let quantity = level.quantity;
        if penalty && Some(quantity) != registry.neutral_dimensionless() {
            return Err(CompileError::Missing(
                "elastic penalties require an explicitly normalized dimensionless objective".into(),
            ));
        }
        Some(pse_math::binding::Objective {
            quantity,
            sense: objective_sense(level.sense),
        })
    } else if penalty {
        Some(pse_math::binding::Objective {
            quantity: registry
                .neutral_dimensionless()
                .ok_or_else(|| CompileError::Missing("dimensionless elastic penalty".into()))?,
            sense: pse_math::binding::ObjectiveSense::Minimize,
        })
    } else {
        None
    };
    let mut instances = vec![];
    let mut rows = vec![];
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: registry,
        preconditions: checker.as_ref(),
        scope: &p.physical_scope,
    };
    if let Some(observation) = pse_diagnostics::observation::current() {
        observation.count("compiler.projection_lookups", "count", 1);
        observation.count(
            "compiler.projection_lookup_entries",
            "count",
            ports.len() as u64,
        );
    }
    let mut conservation_rows = BTreeMap::<SemanticId, Vec<(SemanticId, f64)>>::new();
    for (row, terms) in &p.conservation {
        for (term, sign) in terms {
            conservation_rows
                .entry(*term)
                .or_default()
                .push((*row, *sign));
        }
    }
    for (index, (region, output)) in p.regions.iter().zip(&p.outputs).enumerate() {
        checkpoint(db);
        if let ModelingOutput::Equation {
            id,
            sense: EquationSense::Eq,
        } = output
            && p.conservation.contains_key(id)
        {
            rows.push(Row {
                id: *id,
                quantity: p.quantities[index],
                lower: 0.0,
                upper: 0.0,
            });
            continue;
        }
        let region = implicit::resolve_region(&p.model, region, &implicit, &context)?;
        let mut lookup = |name: &str| {
            provider(db, inventory, name.to_owned()).ok_or_else(|| {
                MathError::Contract(format!("unregistered external implementation {name}"))
            })
        };
        let selected_providers = crate::typed_math::prepare_selected_providers(
            &region.syntax,
            &region.support,
            &mut lookup,
            &direct_providers,
            &context,
        )?;
        let mut physical = FramedHasher::new(pse_ids::Frame::MathPhysicalPremisesV1);
        frame_physical_scope(&mut physical, &p.physical_scope);
        region.support.frame(&mut physical);
        selected_providers.frame(&mut physical);
        let physical = physical.finish_hash();
        let mut h = FramedHasher::new(pse_ids::Frame::ModelingConsumerBodyV5);
        h.hash(&region.syntax.identity()?).hash(&physical);
        region.syntax.frame_input_layout(&region.inputs, &mut h)?;
        h.u64(p.body_limits.slots as u64)
            .u64(p.body_limits.occurrences as u64);
        let identity = pse_ids::roles::SemanticBodyHash::from_id(h.finish_hash());
        let input = Arc::new(BodyInput {
            identity,
            syntax: region.syntax.clone(),
            inputs: region.inputs.clone(),
            support: region.support.clone(),
            providers: selected_providers,
            physical,
            limits: p.body_limits,
        });
        let actual = region
            .bindings
            .iter()
            .map(|binding| (binding.local, binding.actual))
            .collect::<BTreeMap<_, _>>();
        let slots = region
            .inputs
            .iter()
            .map(|slot| {
                let id = actual.get(slot).ok_or_else(|| {
                    CompileError::Missing("selected input has no actual instance binding".into())
                })?;
                let source = ports.get(id).ok_or_else(|| {
                    CompileError::Missing("selected input is absent from actual case ports".into())
                })?;
                let quantity = region
                    .support
                    .input(*slot)?
                    .require_named()
                    .map_err(MathError::from)?;
                Ok(SlotBinding::new(
                    source,
                    &Port {
                        id: *id,
                        quantity,
                        unit: registry
                            .quantity_type(quantity)
                            .map_err(MathError::from)?
                            .canonical_unit,
                    },
                    registry,
                )?)
            })
            .collect::<Result<Vec<_>>>()?;
        let checked_members = region.checked_members.clone();
        let checked_sources = region.checked_sources.clone();
        let occurrences = region.occurrences.clone();
        let (lower, upper) = match output.constraint() {
            Some((_, EquationSense::Eq)) => (0.0, 0.0),
            Some((_, EquationSense::Le)) => (f64::NEG_INFINITY, 0.0),
            Some((_, EquationSense::Ge)) => (0.0, f64::INFINITY),
            None => (f64::NEG_INFINITY, f64::INFINITY),
        };
        let id = output.row_id();
        rows.push(Row {
            id,
            quantity: p.quantities[index],
            lower,
            upper,
        });
        let mut contributions = vec![Contribution {
            output: 0,
            target: Target::Row(id),
            // β enters its bound row negatively: `level value − β`.
            scale: if matches!(output, ModelingOutput::LevelBound { .. }) {
                -1.0
            } else {
                1.0
            },
        }];
        if let ModelingOutput::Contribution { contribution, .. } = output {
            for (row, sign) in conservation_rows.get(contribution).into_iter().flatten() {
                contributions.push(Contribution {
                    output: 0,
                    target: Target::Row(*row),
                    scale: *sign,
                });
            }
        }
        if matches!(output, ModelingOutput::Penalty(_)) {
            contributions.push(Contribution {
                output: 0,
                target: Target::PRIMARY,
                scale: objective.as_ref().map_or(1.0, |o| o.sense.sign()),
            });
        }
        if let ModelingOutput::Member(member) = output {
            for (target, scale) in objective_scales.get(member).into_iter().flatten() {
                contributions.push(Contribution {
                    output: 0,
                    target: *target,
                    scale: *scale,
                });
            }
        }
        instances.push(PlannedInstance {
            input,
            instance: id,
            checked_members,
            checked_sources,
            occurrences,
            slots,
            contributions,
        });
    }
    let mut term_outputs = BTreeMap::<SemanticId, Vec<(SemanticId, f64)>>::new();
    for output in &p.outputs {
        if let ModelingOutput::Term {
            equation, negative, ..
        } = output
        {
            term_outputs
                .entry(*equation)
                .or_default()
                .push((output.row_id(), if *negative { -1.0 } else { 1.0 }));
        }
    }
    Ok(PlannedModeling {
        inputs: p.inputs.clone(),
        outputs: p
            .outputs
            .iter()
            .filter(|o| !matches!(o, ModelingOutput::Term { .. }))
            .cloned()
            .collect(),
        term_outputs,
        implicit,
        instances,
        variables,
        parameters,
        rows,
        objective,
        levels,
        native: p.native.clone(),
        requirements: p.requirements.clone(),
    })
}

pub(super) fn complete(
    db: &dyn CompilerDb,
    inventory: Inventory,
    planned: PlannedModeling,
) -> Result<Arc<AdmittedModeling>> {
    let mut bodies = BTreeMap::new();
    let mut instances = Vec::with_capacity(planned.instances.len());
    for instance in planned.instances {
        checkpoint(db);
        if !instance
            .input
            .support
            .observe(inventory.quantities(db), inventory.preconditions(db))?
            || !instance
                .input
                .providers
                .observe(inventory.quantities(db), inventory.preconditions(db))?
        {
            return Err(MathError::Contract(
                "planned resolved physical or provider premises changed before body admission"
                    .into(),
            )
            .into());
        }
        let requested = instance.input.identity;
        let key = BodyKey::new(db, instance.input);
        let body = pse_quantity::premises::without_recording(|| semantic_body(db, key)).map_err(
            |cause| MathError::Instance {
                instance: instance.instance,
                checked_members: instance.checked_members.clone(),
                checked_sources: instance.checked_sources.clone(),
                occurrences: instance.occurrences.clone(),
                cause: Box::new(MathError::Typed {
                    retained: cause.retained_bytes(),
                    cause: pse_model::diagnostic::DiagnosticCause::new(cause),
                }),
            },
        )?;
        if body.semantic_identity() != Some(requested) {
            pse_quantity::premises::unsupported();
        }
        admit_live_body(
            db,
            requested,
            body.descriptor_bytes()
                .saturating_add(body.math.retained_bytes()),
        );
        let key = body.spec.key();
        bodies.insert(key, body);
        instances.push(InstanceBinding {
            instance: instance.instance,
            body: key,
            checked_members: (instance.checked_members).into(),
            checked_sources: (instance.checked_sources).into(),
            occurrences: instance.occurrences.into(),
            slots: (instance.slots).into(),
            contributions: instance.contributions,
        });
    }
    let case_limits = CaseLimits::default();
    let case = if planned.levels.is_empty() {
        CaseStructure::new(
            planned.variables,
            planned.parameters,
            instances,
            planned.rows,
            planned.objective,
            case_limits,
        )?
    } else {
        CaseStructure::lexicographic(
            planned.variables,
            planned.parameters,
            instances,
            planned.rows,
            planned.levels,
            case_limits,
        )?
    };
    checkpoint(db);
    let case = Arc::new(
        case.with_native(planned.native)?
            .with_requirements(planned.requirements),
    );
    Ok(Arc::new(AdmittedModeling {
        supplier_topology_bytes: implicit::SupplierTopology::reservation(
            &planned.implicit,
            &bodies,
            &case,
        )?,
        supplier_topology: Arc::new(once_cell::sync::OnceCell::new()),
        inputs: planned.inputs,
        outputs: planned.outputs,
        term_outputs: planned.term_outputs,
        implicit: planned.implicit,
        bodies,
        case,
    }))
}

#[cfg(test)]
mod dependency_tests {
    use super::*;
    use crate::workspace::{CompilerWorkspace, WorkspaceLimits};
    use pse_modeling::{Bindings, Limits, specialize::root_instance};

    #[test]
    fn supplied_promoted_implicit_descriptors_precede_nested_consumer_planning() {
        let source = "package p { def Root { param p:Scalar=4; implicit outer select branch(y>=0) { var y:Scalar; implicit child {var z:Scalar;eq e:z==p;} realize c on child using nested; eq e:y*y==child.z; } realize o on outer using nested; eq result:outer.y==2; } }";
        let rows = crate::authored_transfer_tests::rows(source);
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = CompilerWorkspace::new(
            crate::authored_transfer_tests::context(),
            WorkspaceLimits::default(),
        )
        .unwrap();
        workspace
            .publish_modeling(rows, PhysicalScope::default())
            .unwrap();
        let (catalog, request) = workspace
            .modeling_request(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default(),
            )
            .unwrap();
        let projected = projection(&workspace.db, workspace.inventory, catalog, request).unwrap();
        let primary = plan(
            &workspace.db,
            workspace.inventory,
            &projected,
            BTreeMap::new(),
        )
        .unwrap();
        let (child, supplier) = primary
            .implicit
            .iter()
            .find(|(_, supplier)| supplier.selection.restriction.is_none())
            .unwrap();
        let mut consuming = projected.as_ref().clone();
        consuming
            .implicit
            .retain(|projection| projection.id != *child);
        let retained = plan(
            &workspace.db,
            workspace.inventory,
            &consuming,
            BTreeMap::from([(*child, supplier.clone())]),
        )
        .unwrap();
        assert!(
            Arc::ptr_eq(supplier, &retained.implicit[child]),
            "the promoted supplier is not readmitted"
        );
        let outer = retained
            .implicit
            .values()
            .find(|system| system.selection.restriction.is_some())
            .unwrap();
        assert!(
            outer.bodies().any(|body| body
                .math
                .providers()
                .iter()
                .any(|provider| provider.id == *child)),
            "nested admission consumes the supplied provider descriptor"
        );
        let admitted = complete(&workspace.db, workspace.inventory, retained).unwrap();
        assert!(admitted.supplier_topology.get().is_none());
        let before = admitted.as_ref().clone();
        let bytes = admitted_allocation_bytes(&admitted);
        let refused = admitted.supplier_topology.get_or_try_init(|| {
            Err::<implicit::SupplierTopology, _>(CompileError::Missing(
                "transient topology initialization refusal".into(),
            ))
        });
        assert!(refused.is_err());
        assert!(
            admitted.supplier_topology.get().is_none(),
            "failure does not poison or fill the success memo"
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = admitted.supplier_topology.get_or_try_init(
                    || -> Result<implicit::SupplierTopology> {
                        panic!("interrupted topology fill")
                    },
                );
            }))
            .is_err()
        );
        assert!(admitted.supplier_topology.get().is_none());
        let barrier = std::sync::Barrier::new(4);
        std::thread::scope(|scope| {
            let calls = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        admitted.implicit_order_for(None).unwrap().len()
                    })
                })
                .collect::<Vec<_>>();
            for call in calls {
                assert_eq!(call.join().unwrap(), 2);
            }
        });
        assert_eq!(before, *admitted, "memo state is not mathematical meaning");
        assert!(Arc::ptr_eq(
            &before.supplier_topology,
            &admitted.supplier_topology
        ));
        assert_eq!(
            bytes,
            admitted_allocation_bytes(&admitted),
            "lazy fill is already charged"
        );
        let topology = std::ptr::from_ref(admitted.supplier_topology.get().unwrap());
        assert_eq!(admitted.implicit_order_for(None).unwrap().len(), 2);
        assert_eq!(
            topology,
            std::ptr::from_ref(admitted.supplier_topology.get().unwrap())
        );
    }

    #[test]
    fn reusable_checked_members_bind_actual_inputs_and_computed_locals() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::DiagnosticProjection;
        let source = "package p { def Cell { var x:Scalar; let z:Scalar=2*x; annotation valid x(1,3); annotation valid z(3,6); eq e:z-z==0; } def Root { child a:Cell=Cell(); child b:Cell=Cell(); } }";
        let admit_source = |source: &str| {
            let rows = crate::authored_transfer_tests::rows(source);
            let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
            let mut workspace = CompilerWorkspace::new(
                crate::authored_transfer_tests::context(),
                WorkspaceLimits::default(),
            )
            .unwrap();
            workspace
                .publish_modeling(rows, PhysicalScope::default())
                .unwrap();
            let (catalog, request) = workspace
                .modeling_request(
                    root,
                    root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                )
                .unwrap();
            let projected =
                projection(&workspace.db, workspace.inventory, catalog, request).unwrap();
            let admitted = workspace
                .admit_modeling(
                    root,
                    root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                )
                .unwrap();
            (workspace, admitted, projected)
        };
        let (workspace, admitted, projected) = admit_source(source);
        let equations = admitted
            .outputs
            .iter()
            .filter_map(|output| match output {
                ModelingOutput::Equation { id, .. } => Some(*id),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let instances = admitted
            .case
            .instances()
            .iter()
            .filter(|binding| equations.contains(&binding.instance))
            .collect::<Vec<_>>();
        assert_eq!(instances.len(), 2);
        assert_eq!(
            instances[0].body, instances[1].body,
            "actual member attribution must not split shared mathematics"
        );
        assert!(Arc::ptr_eq(
            &admitted.bodies[&instances[0].body],
            &admitted.bodies[&instances[1].body]
        ));
        assert_eq!(instances[0].checked_members.len(), 2);
        assert_eq!(
            instances[0].checked_members.keys().collect::<Vec<_>>(),
            instances[1].checked_members.keys().collect::<Vec<_>>()
        );
        assert!(instances[0].checked_members.values().all(|actual| {
            !instances[1]
                .checked_members
                .values()
                .any(|other| other == actual)
        }));
        use pse_modeling::derived::{LocalRole, Relationship};
        // Tokens come from the exact physical region that admitted these original
        // ranges. Catalogue identities only verify the producer's canonical role
        // correspondence; no test-local spelling generates replacement authority.
        let ranges_by_instance = instances
            .iter()
            .map(|binding| {
                let index = projected.outputs.iter().position(|output| {
                matches!(output, ModelingOutput::Equation { id, .. } if *id == binding.instance)
            }).unwrap();
                let region = &projected.regions[index];
                assert_eq!(&*binding.checked_members, &region.checked_members);
                assert_eq!(&*binding.checked_sources, &region.checked_sources);
                assert_eq!(&*binding.occurrences, &region.occurrences);
                let layout = region.syntax.canonical_layout().unwrap();
                let mut roles = BTreeMap::new();
                for range in region.support.checked_ranges(&region.syntax).unwrap() {
                    let canonical = layout
                        .symbols
                        .iter()
                        .find(|symbol| symbol.symbol == range.symbol)
                        .unwrap();
                    assert_eq!(
                        range.member,
                        Relationship::CheckedMember {
                            role: canonical.role,
                            ordinal: canonical.ordinal as u64,
                        }
                        .identity()
                    );
                    assert_eq!(
                        range.source,
                        Relationship::CheckedValiditySource {
                            role: canonical.role,
                            ordinal: canonical.ordinal as u64,
                        }
                        .identity()
                    );
                    assert_ne!(
                        range.member, range.source,
                        "member and declaring source remain independent roles"
                    );
                    let actual = region
                        .bindings
                        .iter()
                        .find(|value| value.local == range.symbol)
                        .unwrap()
                        .actual;
                    assert_eq!(region.checked_members[&range.member], actual);
                    let original = projected.model.symbols.slot(actual).unwrap();
                    assert_eq!(
                        region.checked_sources[&range.source],
                        projected.model.member_range_sources[&original]
                    );
                    assert!(
                        roles
                            .insert(canonical.role, (range.member, range.source, actual))
                            .is_none()
                    );
                }
                assert_eq!(
                    roles.keys().copied().collect::<BTreeSet<_>>(),
                    BTreeSet::from([LocalRole::Coordinate, LocalRole::Member])
                );
                let coordinate = roles[&LocalRole::Coordinate].2;
                let local = roles[&LocalRole::Member].2;
                assert!(projected.model.symbols[&coordinate].definition.is_none());
                assert!(projected.model.symbols[&local].definition.is_some());
                assert_eq!(binding.slots.len(), 1);
                assert_eq!(binding.slots[0].source(), coordinate);
                assert_ne!(coordinate, local);
                (binding.instance, roles)
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            instances[0].checked_sources.keys().collect::<Vec<_>>(),
            instances[1].checked_sources.keys().collect::<Vec<_>>()
        );
        assert!(instances[0].checked_sources.values().all(|actual| {
            !instances[1]
                .checked_sources
                .values()
                .any(|other| other == actual)
        }));
        let cancellation = pse_diagnostics::Cancellation::default();
        let plan = admitted
            .plan(
                &workspace.inputs.quantities,
                DerivativeOrder::Second,
                AssemblyLimits::default(),
                &cancellation.flag(),
            )
            .unwrap();
        // Missing or invented attribution is rejected before evaluator construction.
        for (source_map, extra) in [(false, false), (false, true), (true, false), (true, true)] {
            let mut invalid = admitted.case.instances().to_vec();
            let invalid_binding = invalid
                .iter_mut()
                .find(|binding| binding.instance == instances[0].instance)
                .unwrap();
            let map = if source_map {
                &mut invalid_binding.checked_sources
            } else {
                &mut invalid_binding.checked_members
            };
            if extra {
                Arc::make_mut(map).insert(
                    SemanticId::from_bytes([123; 16]),
                    SemanticId::from_bytes([124; 16]),
                );
            } else {
                Arc::make_mut(map).pop_first();
            }
            let structure = Arc::new(
                CaseStructure::new(
                    admitted.case.variables().to_vec(),
                    admitted.case.parameters().to_vec(),
                    invalid,
                    admitted.case.rows().to_vec(),
                    admitted.case.objective().cloned(),
                    CaseLimits::default(),
                )
                .unwrap(),
            );
            let refused = CasePlan::prepare(
                structure,
                admitted
                    .bodies
                    .iter()
                    .map(|(key, body)| (*key, body.math.clone()))
                    .collect(),
                &workspace.inputs.quantities,
                DerivativeOrder::Second,
                AssemblyLimits::default(),
                &cancellation.flag(),
            );
            let expected = if source_map {
                "incomplete or invalid checked-source attribution"
            } else {
                "incomplete or invalid checked-member attribution"
            };
            assert!(matches!(refused, Err(MathError::Contract(message)) if message == expected));
        }
        let mut rebound = admitted.case.instances().to_vec();
        let first = rebound
            .iter_mut()
            .find(|binding| binding.instance == instances[0].instance)
            .unwrap();
        Arc::make_mut(&mut first.checked_members).insert(
            ranges_by_instance[&instances[0].instance][&LocalRole::Member].0,
            SemanticId::from_bytes([124; 16]),
        );
        let rebound = CaseStructure::new(
            admitted.case.variables().to_vec(),
            admitted.case.parameters().to_vec(),
            rebound,
            admitted.case.rows().to_vec(),
            admitted.case.objective().cloned(),
            CaseLimits::default(),
        )
        .unwrap();
        assert_ne!(
            admitted.case.key(),
            rebound.key(),
            "actual attribution belongs to the current bound view identity"
        );
        let formal_slots = admitted
            .case
            .instances()
            .iter()
            .map(|binding| binding.slots.len())
            .sum();
        let refused = CaseStructure::new(
            admitted.case.variables().to_vec(),
            admitted.case.parameters().to_vec(),
            admitted.case.instances().to_vec(),
            admitted.case.rows().to_vec(),
            admitted.case.objective().cloned(),
            CaseLimits {
                slots: formal_slots,
                ..CaseLimits::default()
            },
        );
        assert!(
            matches!(
                refused,
                Err(MathError::Limit("case slots or empty output layout"))
            ),
            "actual checked-member bindings consume the existing binding extent allowance"
        );
        let assembly = Arc::new(
            plan.compile(
                Default::default(),
                Default::default(),
                &pse_kernels::ExecutionScope::new(cancellation.clone(), None),
            )
            .unwrap(),
        );
        let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), cancellation);
        let mut values = CaseValues {
            scalars: admitted.inputs.iter().map(|id| (*id, 2.0)).collect(),
        };
        assert!(worker.constraints(&values).is_ok());
        for binding in &instances {
            let coordinate = ranges_by_instance[&binding.instance][&LocalRole::Coordinate];
            let local = ranges_by_instance[&binding.instance][&LocalRole::Member];
            // Input is within its range but the computed local is outside its own range.
            values.scalars.insert(coordinate.2, 1.0);
            let diagnostic = worker
                .constraints(&values)
                .unwrap_err()
                .boundary_diagnostic(DiagnosticStage::Evaluation);
            assert_eq!(
                diagnostic.validity.as_ref().unwrap().members,
                vec![binding.checked_members[&local.0]]
            );
            assert_eq!(diagnostic.causes[0].validity, diagnostic.validity);
            assert_eq!(
                diagnostic.validity.as_ref().unwrap().source,
                binding.checked_sources[&local.1]
            );
            assert!(
                binding
                    .checked_sources
                    .keys()
                    .all(|token| !diagnostic.sources.contains(token))
            );
            assert!(
                diagnostic
                    .sources
                    .contains(&binding.checked_members[&local.0])
            );
            assert!(
                binding
                    .checked_members
                    .keys()
                    .all(|local| !diagnostic.sources.contains(local))
            );
            values.scalars.insert(coordinate.2, 4.0);
            let diagnostic = worker
                .constraints(&values)
                .unwrap_err()
                .boundary_diagnostic(DiagnosticStage::Evaluation);
            assert_eq!(
                diagnostic.validity.as_ref().unwrap().members,
                vec![binding.checked_members[&coordinate.0]]
            );
            assert_eq!(
                diagnostic.validity.as_ref().unwrap().source,
                binding.checked_sources[&coordinate.1]
            );
            assert!(
                binding
                    .checked_sources
                    .keys()
                    .all(|token| !diagnostic.sources.contains(token))
            );
            assert!(
                diagnostic
                    .sources
                    .contains(&binding.checked_members[&coordinate.0])
            );
            assert!(
                binding
                    .checked_members
                    .keys()
                    .all(|local| !diagnostic.sources.contains(local))
            );
            values.scalars.insert(coordinate.2, 2.0);
        }
        assert!(worker.constraints(&values).is_ok());
        let (_, changed_range, _) = admit_source(&source.replace("valid z(3,6)", "valid z(3,7)"));
        let changed = changed_range
            .case
            .instances()
            .iter()
            .find(|binding| binding.instance == instances[0].instance)
            .unwrap();
        assert_ne!(instances[0].body, changed.body);
        let (_, changed_type, _) = admit_source(
            &source
                .replace("Scalar", "Time")
                .replace("valid x(1,3)", "valid x(1{s},3{s})")
                .replace("valid z(3,6)", "valid z(3{s},6{s})")
                .replace("z-z==0", "z-z==0{s}"),
        );
        let changed = changed_type
            .case
            .instances()
            .iter()
            .find(|binding| binding.instance == instances[0].instance)
            .unwrap();
        assert_ne!(instances[0].body, changed.body);
        assert_ne!(admitted.case.key(), changed_range.case.key());
    }

    #[test]
    fn checked_member_context_crosses_compiler_error_adapter() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::{DiagnosticCause, DiagnosticProjection, ValidityLineage};
        use pse_model::generated::enums::ModelingValidityLayer;
        let token = SemanticId::from_bytes([1; 16]);
        let actual = SemanticId::from_bytes([2; 16]);
        let source = SemanticId::from_bytes([3; 16]);
        let source_token = SemanticId::from_bytes([4; 16]);
        let inner = MathError::Validity(Box::new(ValidityLineage {
            layer: ModelingValidityLayer::Closure,
            source: source_token,
            form: None,
            sets: vec![],
            variables: vec![],
            members: vec![token],
        }));
        let native = MathError::Native {
            source_id: token,
            retained: inner.retained_bytes(),
            cause: DiagnosticCause::new(inner),
        };
        let compiler = CompileError::Math(Arc::new(native));
        let expected_class = compiler
            .boundary_diagnostic(DiagnosticStage::ModelingAdmission)
            .class;
        let retained = compiler.retained_bytes();
        let error = MathError::Instance {
            instance: source,
            checked_members: BTreeMap::from([(token, actual)]),
            checked_sources: BTreeMap::from([(source_token, source)]),
            occurrences: BTreeMap::new(),
            cause: Box::new(MathError::Typed {
                retained,
                cause: DiagnosticCause::new(compiler),
            }),
        };
        let projected = error.boundary_diagnostic(DiagnosticStage::ModelingAdmission);
        assert_eq!(projected.class, expected_class);
        assert_eq!(projected.validity.as_ref().unwrap().source, source);
        assert!(!projected.sources.contains(&source_token));
        assert_eq!(projected.validity.as_ref().unwrap().members, vec![actual]);
        assert!(projected.sources.contains(&token));
        let native = &projected.causes[0];
        assert_eq!(native.validity.as_ref().unwrap().members, vec![actual]);
        assert!(native.sources.contains(&token));
        let member_only = &native.causes[0];
        assert_eq!(member_only.validity.as_ref().unwrap().members, vec![actual]);
        assert!(member_only.sources.contains(&actual));
        assert!(!member_only.sources.contains(&token));
        assert!(member_only.sources.contains(&source));
    }

    #[test]
    fn overlapping_consumer_locals_keep_independent_validity_and_shared_lookup() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::DiagnosticProjection;
        let source = "package p { def Root { var x:Scalar; var y:Scalar; var limit:Scalar; let shared:Scalar=x+y; let left:Scalar=2*shared; let right:Scalar=3*shared; let unused:Scalar=log(-1); annotation valid shared(0,limit); annotation valid left(0,20); annotation valid right(0,30); eq a:left==6; eq b:right==9; } }";
        let rows = crate::authored_transfer_tests::rows(source);
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = CompilerWorkspace::new(
            crate::authored_transfer_tests::context(),
            WorkspaceLimits::default(),
        )
        .unwrap();
        workspace
            .publish_modeling(rows, PhysicalScope::default())
            .unwrap();
        let (catalog, request) = workspace
            .modeling_request(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default(),
            )
            .unwrap();
        let projected = projection(&workspace.db, workspace.inventory, catalog, request).unwrap();
        let member = |name: &str| {
            projected
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let equations = projected
            .outputs
            .iter()
            .enumerate()
            .filter_map(|(index, output)| {
                matches!(output, ModelingOutput::Equation { .. }).then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(equations.len(), 2);
        let mut actual_members = Vec::new();
        for index in equations {
            let region = &projected.regions[index];
            let actual = region
                .bindings
                .iter()
                .map(|binding| (binding.local, binding.actual))
                .collect::<BTreeMap<_, _>>();
            let inputs = region
                .inputs
                .iter()
                .map(|slot| actual[slot])
                .collect::<BTreeSet<_>>();
            assert_eq!(
                inputs,
                BTreeSet::from([member("x"), member("y"), member("limit")])
            );
            assert_eq!(
                region.inputs.len(),
                inputs.len(),
                "one input slot per actual independent coordinate"
            );
            assert!(
                region.reads.guards.contains(&member("limit")),
                "validity-only input stays in the actual guard closure"
            );
            assert!(
                !region.reads.numeric.contains(&member("limit")),
                "a guard-only endpoint must not acquire numerical authority"
            );
            let ranges = region.support.checked_ranges(&region.syntax).unwrap();
            assert_eq!(ranges.len(), 2);
            assert_eq!(region.checked_members.len(), 2);
            assert_eq!(region.checked_sources.len(), 2);
            assert!(
                ranges
                    .iter()
                    .all(|range| region.checked_members.contains_key(&range.member)
                        && region.checked_sources.contains_key(&range.source))
            );
            assert!(
                region
                    .checked_members
                    .keys()
                    .all(|token| !region.checked_sources.contains_key(token))
            );
            let members = region
                .checked_members
                .values()
                .copied()
                .collect::<BTreeSet<_>>();
            assert!(members.contains(&member("shared")));
            assert_eq!(members.len(), 2);
            assert!(
                region
                    .bindings
                    .iter()
                    .all(|binding| binding.actual != member("unused")),
                "unused invalid local has no selected binding"
            );
            actual_members.push(members);
        }
        assert!(actual_members.iter().any(
            |members| members.contains(&member("left")) && !members.contains(&member("right"))
        ));
        assert!(actual_members.iter().any(
            |members| members.contains(&member("right")) && !members.contains(&member("left"))
        ));
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("grouped-projection");
        let entered = operation.context().enter();
        let planned = plan(
            &workspace.db,
            workspace.inventory,
            &projected,
            BTreeMap::new(),
        )
        .unwrap();
        drop(entered);
        assert!(planned.instances.len() >= 2);
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == "compiler.projection_lookups")
                .unwrap()
                .cumulative,
            Value::Count(1)
        );
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == "compiler.projection_lookup_entries")
                .unwrap()
                .cumulative,
            Value::Count(projected.inputs.len() as u64)
        );
        let admitted = complete(&workspace.db, workspace.inventory, planned).unwrap();
        let cancellation = pse_diagnostics::Cancellation::default();
        let assembly = Arc::new(
            admitted
                .plan(
                    &workspace.inputs.quantities,
                    DerivativeOrder::Second,
                    AssemblyLimits::default(),
                    &cancellation.flag(),
                )
                .unwrap()
                .compile(
                    Default::default(),
                    Default::default(),
                    &pse_kernels::ExecutionScope::new(cancellation.clone(), None),
                )
                .unwrap(),
        );
        let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), cancellation);
        let mut values = CaseValues {
            scalars: BTreeMap::from([
                (member("x"), 1.0),
                (member("y"), 2.0),
                (member("limit"), 4.0),
            ]),
        };
        assert!(
            worker.constraints(&values).is_ok(),
            "the actual selected arithmetic must not evaluate unused log(-1)"
        );
        values.scalars.insert(member("limit"), 2.0);
        let diagnostic = worker
            .constraints(&values)
            .unwrap_err()
            .boundary_diagnostic(DiagnosticStage::Evaluation);
        assert_eq!(
            diagnostic.validity.as_ref().unwrap().members,
            vec![member("shared")]
        );
        assert!(
            !diagnostic
                .validity
                .as_ref()
                .unwrap()
                .members
                .contains(&member("left"))
        );
        assert!(
            !diagnostic
                .validity
                .as_ref()
                .unwrap()
                .members
                .contains(&member("right"))
        );
        drop(worker);
        workspace
            .prepare_modeling_cancellable(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
    }

    #[test]
    fn demanded_member_ranges_retain_only_their_transitive_functions() {
        let revision = ContentHash::from_bytes([31; 32]);
        let data = ContentHash::from_bytes([32; 32]);
        let source = format!(
            "package p {{ fn bottom(x:Scalar)->Scalar=1+x; fn lower(x:Scalar)->Scalar=bottom(x); fn upper(x:Scalar)->Scalar=3+x; fn unrelated(x:Scalar)->Scalar external \"unused-range-provider\" revision \"{revision}\" data \"{data}\" output 0 derivatives 2 source analytic smoothness 2; def Root {{ var x:Scalar; annotation valid x(lower(0),upper(0)); eq e:x==2; }} }}"
        );
        let rows = crate::authored_transfer_tests::rows(&source);
        let declaration = |name: &str| {
            rows.iter()
                .find(|row| row.name == name)
                .unwrap()
                .declaration_id
        };
        let expected = BTreeSet::from([
            declaration("bottom"),
            declaration("lower"),
            declaration("upper"),
        ]);
        let unrelated = declaration("unrelated");
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = CompilerWorkspace::new(
            crate::authored_transfer_tests::context(),
            WorkspaceLimits::default(),
        )
        .unwrap();
        workspace
            .publish_modeling(rows, PhysicalScope::default())
            .unwrap();
        let (catalog, request) = workspace
            .modeling_request(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default(),
            )
            .unwrap();
        let projected = projection(&workspace.db, workspace.inventory, catalog, request).unwrap();
        let ordinal = projected
            .outputs
            .iter()
            .position(|output| matches!(output, ModelingOutput::Equation { .. }))
            .unwrap();
        let region = &projected.regions[ordinal];
        assert_eq!(
            region.support.checked_ranges(&region.syntax).unwrap().len(),
            1
        );
        assert_eq!(region.checked_members.len(), 1);
        let member = projected
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        assert_eq!(
            region.checked_members.values().copied().collect::<Vec<_>>(),
            vec![member]
        );
        fn functions(
            syntax: &pse_modeling::resolved::ResolvedSyntax,
            seen: &mut BTreeSet<ContentHash>,
            declarations: &mut BTreeSet<DeclarationId>,
        ) {
            for slot in syntax.canonical_layout().unwrap().functions {
                let product = &syntax.functions[slot].product;
                declarations.insert(product.declaration);
                if seen.insert(product.identity) {
                    functions(&product.syntax, seen, declarations);
                }
            }
        }
        let mut selected = BTreeSet::new();
        functions(&region.syntax, &mut BTreeSet::new(), &mut selected);
        assert_eq!(
            selected, expected,
            "lower, its bottom dependency, and upper are reached solely through the demanded range"
        );
        assert!(!selected.contains(&unrelated));
        let planned = plan(
            &workspace.db,
            workspace.inventory,
            &projected,
            BTreeMap::new(),
        )
        .unwrap();
        for instance in &planned.instances {
            assert_eq!(instance.input.providers.descriptors().count(), 0);
            assert_eq!(instance.input.providers.calls().count(), 0);
        }
        // No registration exists for unused-range-provider. Planning, materialization,
        // and the public preparation must all succeed without selecting it.
        let admitted = complete(&workspace.db, workspace.inventory, planned).unwrap();
        let cancellation = pse_diagnostics::Cancellation::default();
        let assembly = Arc::new(
            admitted
                .plan(
                    &workspace.inputs.quantities,
                    DerivativeOrder::Second,
                    AssemblyLimits::default(),
                    &cancellation.flag(),
                )
                .unwrap()
                .compile(
                    Default::default(),
                    Default::default(),
                    &pse_kernels::ExecutionScope::new(cancellation.clone(), None),
                )
                .unwrap(),
        );
        let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), cancellation);
        let mut values = CaseValues {
            scalars: BTreeMap::from([(member, 2.0)]),
        };
        assert!(worker.constraints(&values).is_ok());
        use pse_model::diagnostic::DiagnosticProjection;
        for outside in [0.0, 4.0] {
            values.scalars.insert(member, outside);
            let diagnostic = worker
                .constraints(&values)
                .unwrap_err()
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
            assert_eq!(diagnostic.validity.as_ref().unwrap().members, vec![member]);
        }
        drop(worker);
        workspace
            .prepare_modeling_cancellable(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
    }
}
