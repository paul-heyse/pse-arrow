// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure numerical annotation and view decisions. Case values stay outside Salsa.
use super::*;
use pse_model::generated::enums::{ConstraintScalingScheme, NumericalTarget};

/// Numerical interpretation mode, independent of mutable evaluator state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumericalAnnotationMode {
    /// Outer algebraic/steady solve.
    Steady,
    /// One nested residual's ordered unknown and row inventory.
    Implicit,
    /// Integrated state and algebraic observations.
    Dynamic,
}
/// Actual observation location; nested hint bodies use ordered output positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnnotationObservation {
    /// Existing compiled observation row.
    Row(SemanticId),
    /// Output of the residual's common/local hint body.
    Position(usize),
}
/// Structural location of an admitted target, separate from mutable observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NumericalLocation {
    /// Outer algebraic coordinate or observation.
    Steady,
    /// Integrated state or instantaneous algebraic observation.
    Time,
    /// A terminal integral result.
    Integral,
    /// A terminal expression consuming complete integral results.
    Terminal,
    /// Original conservation or inventory obligation.
    Closure,
    /// Ordered nested residual output; unknowns have no output position.
    Nested(Option<usize>),
}
/// Semantic target role, preserving distinctions that share a numerical budget kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NumericalTargetRole {
    /// Outer steady coordinate.
    Variable,
    /// Steady constraint residual.
    Row,
    /// A physical output observation.
    Observable,
    /// Integrated differential coordinate.
    DifferentialState,
    /// Instantaneous algebraic coordinate.
    AlgebraicState,
    /// Instantaneous algebraic residual.
    AlgebraicRow,
    /// Original endpoint consistency equation.
    InitialCondition,
    /// Original differential equation retained for inspection.
    DifferentialEquation,
    /// Whole-domain integral result.
    Integral,
    /// Terminal expression consuming integral results.
    Terminal,
    /// Original physical closure.
    Closure,
    /// Original conserved-inventory obligation.
    Inventory,
    /// Ordered nested unknown.
    NestedUnknown,
    /// Ordered nested residual.
    NestedRow,
    /// Optimization objective.
    Objective,
}
/// Complete physical target fact. Values and provider evaluations never enter this product.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NumericalTargetFact {
    /// Exact target identity.
    pub id: SemanticId,
    /// Mode-specific numerical role.
    pub kind: NumericalTarget,
    /// Mode-specific semantic role; physical coordinates and observations remain distinct.
    pub role: NumericalTargetRole,
    /// Original declaration owning this target.
    pub declaration: DeclarationId,
    /// Complete physical contract.
    pub quantity: QuantityTypeId,
    /// Canonical representation of that contract.
    pub unit: pse_quantity::UnitId,
    /// Authored integer coordinate domain.
    pub integer: bool,
    /// Target location in this mode.
    pub location: NumericalLocation,
    /// Existing compiled observation, where one exists.
    pub observation: Option<AnnotationObservation>,
    /// Original transitive support; absence explicitly refuses a frozen-support claim.
    pub support: Option<BTreeSet<SemanticId>>,
    /// Target is owned by a nested numerical operation.
    pub nested: bool,
    /// Part of the mode's base numerical policy, before requested observations.
    pub selected: bool,
}
impl NumericalTargetFact {
    /// Bind no values: the runtime numerical owner receives the exact physical fact.
    pub fn specification(&self) -> pse_math::numerics::TargetSpec {
        pse_math::numerics::TargetSpec {
            id: self.id, kind: self.kind, quantity: self.quantity, unit: self.unit,
            integer: self.integer, declared_tolerance: None,
        }
    }
}
/// Explicit per-mode interpretation of an authored numerical observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericalAdmission {
    /// The mode has an exact supported target and observation.
    Supported,
    /// A separate nested operation owns this annotation.
    NotApplicable,
    /// This mode cannot honor the authored meaning.
    Refused(&'static str),
}
/// One physically typed, source-attributed numerical expression observation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NumericalAnnotation {
    /// Semantic coordinate receiving this fact.
    pub target: SemanticId,
    /// Source declaration; no rendered/native label is policy authority.
    pub declaration: DeclarationId,
    /// Existing compiler expression role.
    pub kind: ModelingHint,
    /// Physical type of the observed expression, including difference quantities.
    pub quantity: QuantityTypeId,
    /// Exact compiled observation to evaluate with current case values.
    pub observation: AnnotationObservation,
    /// Mode's numerical target classification.
    pub target_kind: NumericalTarget,
    /// Source ownership by a nested unknown/row; steady outer views exclude it.
    pub nested: bool,
    /// The bound originally read no member. Dynamic sign views alone consume this flag.
    pub constant_bound: bool,
    /// Complete selected free input/member references of this observation.
    pub reads: BTreeSet<SemanticId>,
}
impl NumericalAnnotation {
    /// Existing compiled row; nested positional bodies refuse this operation.
    pub fn row(&self) -> Result<SemanticId> {
        match self.observation {
            AnnotationObservation::Row(row) => Ok(row),
            AnnotationObservation::Position(_) => Err(CompileError::Missing("numerical observation requires a row".into())),
        }
    }
}
/// Original term observations consumed by an authored scaling rule.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScalingObservations {
    /// Ordinary original-term rows, in source order.
    Rows(Vec<SemanticId>),
    /// Ordered original-term body range for a nested residual.
    Range(std::ops::Range<usize>),
}
/// Typed scaling instruction with its declaration and original terms preserved.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NumericalScale {
    /// Original row being scaled.
    pub target: SemanticId,
    /// Source declaration.
    pub declaration: DeclarationId,
    /// Existing declared numerical scaling scheme.
    pub scheme: ConstraintScalingScheme,
    /// Actual original-term observations.
    pub observations: ScalingObservations,
    /// A steady outer view leaves this rule to its nested owner.
    pub nested: bool,
}
/// Start precedence is explicit, including legitimate mode differences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericalStartSource {
    /// A compiler-isolated dynamic endpoint equality.
    InitialCondition,
    /// A nested operation's semantic anchor.
    SemanticAnchor,
    /// Values resolved from the current case/continuation/binding overrides.
    SuppliedValue,
    /// The selected authored start observation.
    AuthoredHint,
}
/// Current evaluated candidates. These values are never part of a Salsa query key.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NumericalStartValues {
    /// Evaluated isolated dynamic endpoint.
    pub initial_condition: Option<f64>,
    /// Current nested semantic anchor.
    pub semantic_anchor: Option<f64>,
    /// Current explicit case/binding/continuation value.
    pub supplied_value: Option<f64>,
    /// Current evaluated authored start.
    pub authored_hint: Option<f64>,
}
/// Immutable instructions; numerical values and override application remain per case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericalAnnotations {
    /// Consumed interpretation mode, also part of the tracked query key.
    pub mode: NumericalAnnotationMode,
    /// Ordered source facts and observation locations.
    pub hints: Arc<[NumericalAnnotation]>,
    /// Complete mode-specific target inventory, including optional requested observations.
    pub targets: Arc<[NumericalTargetFact]>,
    /// Source-selected original-term scaling instructions.
    pub scales: Arc<[NumericalScale]>,
    /// Highest-precedence source first.
    pub start_precedence: Vec<NumericalStartSource>,
    /// Explicit endpoint overrides replace authored bounds, including explicit absence.
    pub bounds_overrides_first: bool,
}
impl NumericalAnnotations {
    /// Exact admitted target; unknown identities cannot acquire an inferred role.
    pub fn target(&self, id: SemanticId) -> Option<&NumericalTargetFact> {
        self.targets.binary_search_by_key(&id, |target| target.id).ok().and_then(|index| self.targets.get(index))
    }
    /// Selected numerical targets in this mode, retaining nested ownership.
    pub fn selected_targets(&self) -> impl Iterator<Item = &NumericalTargetFact> {
        self.targets.iter().filter(|target| target.selected && !target.nested)
    }
    /// Classify an owner-issued observation index without scanning the hint inventory.
    pub fn admission(&self, index: usize) -> NumericalAdmission {
        let Some(hint) = self.hints.get(index) else { return NumericalAdmission::Refused("annotation observation index was not issued by the compiler"); };
        if hint.nested { return NumericalAdmission::NotApplicable; }
        let Some(target) = self.target(hint.target) else { return NumericalAdmission::Refused("annotation target absent from compiler inventory"); };
        if self.mode == NumericalAnnotationMode::Dynamic && matches!(hint.kind, ModelingHint::Start | ModelingHint::Lower | ModelingHint::Upper)
            && (target.kind != NumericalTarget::Variable || !matches!(target.location, NumericalLocation::Time)) {
            return NumericalAdmission::Refused("dynamic bounds and starts require an integrated state coordinate");
        }
        NumericalAdmission::Supported
    }
    /// Reject unsupported authored meaning before numerical observation binding.
    pub fn require_supported(&self) -> Result<()> {
        for index in 0..self.hints.len() {
            if let NumericalAdmission::Refused(reason) = self.admission(index) { return Err(CompileError::Missing(reason.into())); }
        }
        Ok(())
    }
    /// Read exact owner-issued support by observation index; missing support never becomes empty.
    pub fn require_frozen(&self, index: usize, forbidden: &BTreeSet<SemanticId>) -> Result<()> {
        let hint = self.hints.get(index).ok_or_else(|| CompileError::Missing("numerical annotation support observation index".into()))?;
        if self.target(hint.target).is_none() {
            return Err(CompileError::Missing("numerical annotation target support crosswalk".into()));
        }
        if !hint.reads.is_disjoint(forbidden) {
            return Err(CompileError::Missing("accuracy support depends on a changing solve coordinate".into()));
        }
        Ok(())
    }

    /// Positive known descriptor storage, excluding independently owned mathematics.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.targets.len() * size_of::<NumericalTargetFact>()
            + self.targets.iter().map(|target| target.support.as_ref().map_or(0, |support| support.len() * 128)).sum::<usize>()
            + self.hints.len() * size_of::<NumericalAnnotation>()
            + self
                .hints
                .iter()
                .map(|hint| hint.reads.len() * 128)
                .sum::<usize>()
            + self.scales.len() * size_of::<NumericalScale>()
            + self
                .scales
                .iter()
                .map(|scale| match &scale.observations {
                    ScalingObservations::Rows(rows) => rows.capacity() * size_of::<SemanticId>(),
                    ScalingObservations::Range(_) => 0,
                })
                .sum::<usize>()
            + self.start_precedence.capacity() * size_of::<NumericalStartSource>()
    }
    /// Outer-bound/nominal consumers exclude nested-owned coordinates.
    pub fn outer_hints(&self) -> impl Iterator<Item = &NumericalAnnotation> {
        self.hints
            .iter()
            .filter(|hint| !hint.nested)
    }
    /// Steady conditioning and nested residual scaling keep their existing owners.
    /// Integrated mode uses physical nominal/accuracy budgets, not steady row conditioning.
    pub fn outer_scales(&self) -> impl Iterator<Item = &NumericalScale> {
        self.scales.iter().filter(|scale| match self.mode {
            NumericalAnnotationMode::Steady => !scale.nested,
            NumericalAnnotationMode::Implicit => true,
            NumericalAnnotationMode::Dynamic => false,
        })
    }
    /// Only dynamic, originally member-free bounds can establish a constant sign.
    pub fn constant_sign_bounds(&self) -> impl Iterator<Item = &NumericalAnnotation> {
        self.hints.iter().filter(|hint| {
            self.mode == NumericalAnnotationMode::Dynamic
                && hint.constant_bound
                && matches!(hint.kind, ModelingHint::Lower | ModelingHint::Upper)
        })
    }
    /// Apply the compiler's mode-specific precedence to fresh evaluated values.
    /// Finite/domain checks remain at the current numerical receiving boundary.
    pub fn select_start(
        &self,
        values: NumericalStartValues,
    ) -> Option<(f64, NumericalStartSource)> {
        self.start_precedence.iter().find_map(|source| {
            let value = match source {
                NumericalStartSource::InitialCondition => values.initial_condition,
                NumericalStartSource::SemanticAnchor => values.semantic_anchor,
                NumericalStartSource::SuppliedValue => values.supplied_value,
                NumericalStartSource::AuthoredHint => values.authored_hint,
            };
            value.map(|value| (value, *source))
        })
    }
    /// Explicit endpoint absence removes an authored bound; an unspecified endpoint inherits it.
    pub fn effective_bound(
        &self,
        endpoint_override: Option<Option<f64>>,
        authored: Option<f64>,
    ) -> Option<f64> {
        endpoint_override.unwrap_or(authored)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct AnnotationFacts {
    pub mode: NumericalAnnotationMode,
    pub hints: Arc<[NumericalAnnotation]>,
    /// Complete mode-specific target inventory, including optional requested observations.
    pub targets: Arc<[NumericalTargetFact]>,
    pub scales: Arc<[NumericalScale]>,
}
#[salsa::interned(heap_size = annotation_key_heap)]
struct AnnotationKey<'db> {
    facts: Arc<AnnotationFacts>,
}
fn annotation_key_heap((facts,): &(Arc<AnnotationFacts>,)) -> usize {
    size_of::<AnnotationFacts>()
        + facts.targets.len() * size_of::<NumericalTargetFact>()
        + facts.targets.iter().map(|target| target.support.as_ref().map_or(0, |support| support.len() * 128)).sum::<usize>()
        + facts.hints.len() * size_of::<NumericalAnnotation>()
        + facts
            .hints
            .iter()
            .map(|hint| hint.reads.len() * 128)
            .sum::<usize>()
        + facts.scales.len() * size_of::<NumericalScale>()
        + facts
            .scales
            .iter()
            .map(|scale| match &scale.observations {
                ScalingObservations::Rows(rows) => rows.capacity() * size_of::<SemanticId>(),
                ScalingObservations::Range(_) => 0,
            })
            .sum::<usize>()
}
#[salsa::tracked(returns(clone),lru=256,heap_size=annotation_heap)]
fn annotations(db: &dyn CompilerDb, key: AnnotationKey<'_>) -> Arc<NumericalAnnotations> {
    checkpoint(db);
    let facts = key.facts(db);
    use NumericalStartSource as S;
    Arc::new(NumericalAnnotations {
        mode: facts.mode,
        hints: facts.hints.clone(),
        targets: facts.targets.clone(),
        scales: facts.scales.clone(),
        start_precedence: match facts.mode {
            NumericalAnnotationMode::Steady => vec![S::SuppliedValue, S::AuthoredHint],
            NumericalAnnotationMode::Implicit => {
                vec![S::SemanticAnchor, S::SuppliedValue, S::AuthoredHint]
            }
            NumericalAnnotationMode::Dynamic => {
                vec![S::InitialCondition, S::SuppliedValue, S::AuthoredHint]
            }
        },
        bounds_overrides_first: true,
    })
}
fn annotation_heap(value: &Arc<NumericalAnnotations>) -> usize {
    value.retained_bytes()
}
pub(super) fn lower(db: &dyn CompilerDb, mut facts: AnnotationFacts) -> Arc<NumericalAnnotations> {
    let mut targets = facts.targets.to_vec();
    targets.sort_by_key(|target| target.id);
    facts.targets = targets.into();
    annotations(db, AnnotationKey::new(db, Arc::new(facts)))
}
pub(super) type AnnotationPlans = BTreeMap<NumericalAnnotationMode, Arc<NumericalAnnotations>>;
pub(super) fn projection_annotations(
    db: &dyn CompilerDb,
    p: &Projection,
    constant_bounds: &BTreeSet<SemanticId>,
    registry: &QuantityRegistry,
) -> Result<Arc<AnnotationPlans>> {
    let nested = p
        .implicit
        .iter()
        .flat_map(|system| system.annotation_inventory())
        .collect::<BTreeSet<_>>();
    let variables = p
        .free
        .keys()
        .copied()
        .chain(
            p.implicit
                .iter()
                .flat_map(|system| system.annotation_variables()),
        )
        .collect::<BTreeSet<_>>();
    let equations = p
        .outputs
        .iter()
        .filter_map(|output| match output {
            ModelingOutput::Equation { id, .. } => Some(*id),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let hints: Arc<[NumericalAnnotation]> = p
        .outputs
        .iter()
        .zip(&p.regions)
        .zip(&p.quantities)
        .filter_map(|((output, region), quantity)| {
            let ModelingOutput::Hint {
                target,
                declaration,
                kind,
            } = output
            else {
                return None;
            };
            Some(NumericalAnnotation {
                target: *target,
                declaration: *declaration,
                kind: *kind,
                quantity: *quantity,
                observation: AnnotationObservation::Row(output.row_id()),
                target_kind: if variables.contains(target) {
                    NumericalTarget::Variable
                } else if equations.contains(target) {
                    NumericalTarget::Row
                } else {
                    NumericalTarget::Observable
                },
                nested: nested.contains(target),
                constant_bound: constant_bounds.contains(&output.row_id()),
                reads: region
                    .reads
                    .numeric
                    .union(&region.reads.guards)
                    .copied()
                    .collect(),
            })
        })
        .collect();
    let scales: Arc<[NumericalScale]> = p
        .annotation_scales
        .iter()
        .cloned()
        .map(|mut scale| {
            scale.nested = nested.contains(&scale.target);
            scale
        })
        .collect();
    let index = ProjectionTargetIndex::new(p);
    let mut plans = BTreeMap::new();
    for mode in [
        NumericalAnnotationMode::Steady,
        NumericalAnnotationMode::Implicit,
        NumericalAnnotationMode::Dynamic,
    ] {
        let targets = projection_targets(p, mode, &nested, registry, &index)?;
        plans.insert(
            mode,
            lower(
                db,
                AnnotationFacts {
                    mode,
                    hints: hints.iter().cloned().map(|mut hint| {
                        if let Ok(position) = targets.binary_search_by_key(&hint.target, |target| target.id) {
                            let target = &targets[position];
                            hint.target_kind = target.kind;
                        }
                        hint
                    }).collect(),
                    targets,
                    scales: scales.clone(),
                },
            ),
        );
    }
    Ok(Arc::new(plans))
}
struct ProjectionTargetIndex<'a> {
    members: BTreeMap<SemanticId, (&'a ModelingOutput, &'a pse_modeling::resolved::SelectedRegion)>,
    rates: BTreeSet<SemanticId>,
    used_dynamic_rows: BTreeSet<SemanticId>,
    annotated: BTreeSet<SemanticId>,
    row_declarations: BTreeMap<SemanticId, DeclarationId>,
}
impl<'a> ProjectionTargetIndex<'a> {
    fn new(p: &'a Projection) -> Self {
        let mut used_dynamic_rows = p.model.initial_equations.clone();
        for output in &p.outputs {
            if let ModelingOutput::DynamicRate { equations, .. } = output { used_dynamic_rows.extend(equations); }
        }
        Self {
            members: p.outputs.iter().zip(&p.regions).filter_map(|(output, region)| match output { ModelingOutput::Member(id) => Some((*id, (output, region))), _ => None }).collect(),
            rates: p.model.derivatives.values().map(|derivative| derivative.rate).collect(),
            used_dynamic_rows,
            annotated: p.model.annotations.iter().filter(|annotation| matches!(annotation.value,
                pse_modeling::annotation::AnnotationValue::Nominal(_) | pse_modeling::annotation::AnnotationValue::AccuracyGoal(_)
                | pse_modeling::annotation::AnnotationValue::EngineeringScale(_) | pse_modeling::annotation::AnnotationValue::EngineeringDefault { .. }))
                .map(|annotation| annotation.target).collect(),
            row_declarations: p.model.equations.iter().map(|row| (row.id, row.lineage.declaration)).collect(),
        }
    }
}
fn projection_targets(
    p: &Projection,
    mode: NumericalAnnotationMode,
    nested: &BTreeSet<SemanticId>,
    registry: &QuantityRegistry,
    index: &ProjectionTargetIndex<'_>,
) -> Result<Arc<[NumericalTargetFact]>> {
    use pse_model::generated::enums::ModelingDeclarationKind as K;
    let dynamic = mode == NumericalAnnotationMode::Dynamic;
    let mut targets = BTreeMap::new();
    for (id, symbol) in &p.model.symbols {
        let Some(scheme) = symbol.ty.quantity_scheme() else { continue; };
        let quantity = scheme.resolve(registry, &BTreeMap::new()).map_err(|error| CompileError::Missing(error.to_string()))?;
        let variable = symbol.role == K::Variable && (!dynamic || !index.rates.contains(id));
        let integral = p.model.integrals.contains_key(id);
        let output = index.members.get(id).copied();
        let support = if let Some((_, region)) = output {
            Some(region.reads.numeric.union(&region.reads.guards).copied().collect())
        } else if let Some(definition) = symbol.definition {
            let reads = p.model.resolved.reads(pse_modeling::resolved::RegionRequest { outputs: &[pse_modeling::resolved::RegionOutput::Value(definition)], validity: &[] })?;
            Some(reads.numeric.union(&reads.guards).copied().collect())
        } else { Some(BTreeSet::from([*id])) };

        targets.insert(*id, NumericalTargetFact {
            id: *id, kind: if variable { NumericalTarget::Variable } else { NumericalTarget::Observable },
            role: if nested.contains(id) { NumericalTargetRole::NestedUnknown } else if integral { NumericalTargetRole::Integral } else if output.is_some_and(|(_, region)| region.reads.numeric.iter().any(|read| p.model.integrals.contains_key(read))) { NumericalTargetRole::Terminal } else if variable && dynamic { if p.model.derivatives.contains_key(id) { NumericalTargetRole::DifferentialState } else { NumericalTargetRole::AlgebraicState } } else if variable { NumericalTargetRole::Variable } else { NumericalTargetRole::Observable },
            declaration: symbol.lineage.declaration, quantity,
            unit: registry.quantity_type(quantity).map_err(MathError::from)?.canonical_unit,
            integer: symbol.domain.is_integer(),
            location: if integral { NumericalLocation::Integral } else if output.is_some_and(|(_, region)| region.reads.numeric.iter().any(|read| p.model.integrals.contains_key(read))) { NumericalLocation::Terminal } else if nested.contains(id) { NumericalLocation::Nested(None) } else if dynamic { NumericalLocation::Time } else { NumericalLocation::Steady },
            observation: output.map(|(o, _)| AnnotationObservation::Row(o.row_id())),
            support,
            nested: nested.contains(id),
            selected: variable && p.free.contains_key(id) || (index.annotated.contains(id) && (!variable || p.free.contains_key(id))),
        });
    }
    for (((output, region), quantity), declaration) in p.outputs.iter().zip(&p.regions).zip(&p.quantities).zip(&p.declarations) {
        let id = match output { ModelingOutput::Equation { id, .. } => *id, ModelingOutput::LevelBound { row, .. } => *row, _ => continue };
        targets.insert(id, NumericalTargetFact {
            id, kind: NumericalTarget::Row,
            role: if nested.contains(&id) { NumericalTargetRole::NestedRow } else if !dynamic { NumericalTargetRole::Row } else if p.model.initial_equations.contains(&id) { NumericalTargetRole::InitialCondition } else if index.used_dynamic_rows.contains(&id) { NumericalTargetRole::DifferentialEquation } else { NumericalTargetRole::AlgebraicRow },
            declaration: *declaration,
            quantity: *quantity, unit: registry.quantity_type(*quantity).map_err(MathError::from)?.canonical_unit,
            integer: false, location: if nested.contains(&id) { NumericalLocation::Nested(None) } else if dynamic { NumericalLocation::Time } else { NumericalLocation::Steady },
            observation: Some(AnnotationObservation::Row(output.row_id())), support: Some(region.reads.numeric.union(&region.reads.guards).copied().collect()),
            nested: nested.contains(&id), selected: !dynamic || !index.used_dynamic_rows.contains(&id),
        });
    }
    for system in &p.implicit {
        for (position, id, quantity, region) in system.annotation_rows() {
            let unit = registry.quantity_type(quantity).map_err(MathError::from)?.canonical_unit;
            let declaration = index.row_declarations.get(&id).copied().ok_or_else(|| CompileError::Missing("nested numerical row source".into()))?;
            targets.entry(id).or_insert_with(|| NumericalTargetFact {
                id, kind: NumericalTarget::Row, role: NumericalTargetRole::NestedRow,
                declaration,
                quantity, unit,
                integer:false, location: NumericalLocation::Nested(Some(position)),
                observation: Some(AnnotationObservation::Position(position)),
                support: Some(region.reads.numeric.union(&region.reads.guards).copied().collect()), nested:true, selected:false,
            });
        }
    }
    if !dynamic {
        let objective_quantity = p.objectives.solved().map(|level| level.quantity).or_else(|| p.outputs.iter().any(|o| matches!(o, ModelingOutput::Penalty(_))).then(|| registry.neutral_dimensionless()).flatten());
        if let Some(quantity) = objective_quantity {
            targets.insert(SemanticId::NIL, NumericalTargetFact {
                id: SemanticId::NIL, kind: NumericalTarget::Objective, role: NumericalTargetRole::Objective, declaration: DeclarationId::from(SemanticId::NIL), quantity,
                unit: registry.quantity_type(quantity).map_err(MathError::from)?.canonical_unit, integer: false,
                location: NumericalLocation::Steady, observation: None, support: None, nested: false, selected: true,
            });
        }
    }
    for (id, ty, declaration) in p.model.closures.values().map(|c| (c.id, &c.ty, c.lineage.declaration))
        .chain(p.model.inventory_balances.values().map(|b| (b.id, &b.ty, b.lineage.declaration))) {
        let scheme = ty.quantity_scheme().ok_or_else(|| CompileError::Missing("closure numerical target physical type".into()))?;
        let quantity = scheme.resolve(registry, &BTreeMap::new()).map_err(|error| CompileError::Missing(error.to_string()))?;
        targets.insert(id, NumericalTargetFact { id, kind: NumericalTarget::Closure, role: if p.model.inventory_balances.contains_key(&id) { NumericalTargetRole::Inventory } else { NumericalTargetRole::Closure }, declaration, quantity,
            unit: registry.quantity_type(quantity).map_err(MathError::from)?.canonical_unit, integer: false,
            location: NumericalLocation::Closure, observation: None, support: None, nested: false, selected: index.annotated.contains(&id), });
    }
    Ok(targets.into_values().collect())
}
pub(super) fn plans_bytes(plans: &AnnotationPlans) -> usize {
    plans.len() * 128
        + plans
            .values()
            .map(|plan| plan.retained_bytes())
            .sum::<usize>()
}
pub(super) fn choices_bytes(choices: &ViewChoices) -> usize {
    choices.len() * 128 + choices.values().map(view_heap).sum::<usize>()
}

/// Actual consumed solve-policy inputs governing original-view promotion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewPolicy {
    /// Automatic numerical composition was requested.
    pub automatic_composition: bool,
    /// Original-space reconstruction was requested.
    pub reconstruction: bool,
    /// Caller numerical policy declares an accuracy goal. Authored goals are read by the compiler.
    pub declared_accuracy: bool,
    /// Sensitivity preparation preserves its selected operation view.
    pub sensitivity: bool,
}
/// Structural admission for an operation view. Current scalar values remain runtime inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewRoles {
    /// Numerical targets belonging exclusively to nested operations.
    pub inner_targets: BTreeSet<SemanticId>,
    /// Hidden nested coordinates; outer request filtering consumes the same inventory.
    pub inner_unknowns: BTreeSet<SemanticId>,
    /// Suppliers whose complete unknown demand belongs to the nested view.
    pub nested_suppliers: BTreeSet<pse_kernels::ProviderKey>,
    /// Declared physical parameters whose current representations admit covariance.
    pub covariance_parameters: BTreeSet<SemanticId>,
    /// Fixed declared parameters that the selected sensitivity program may differentiate.
    pub sensitivity_parameters: BTreeSet<SemanticId>,
    /// Output columns solved by the current operation.
    pub propagation_outputs: BTreeSet<SemanticId>,
}
impl ViewRoles {
    /// The outer request cannot bind or supply requirements to a hidden nested target.
    pub fn admits_outer(&self, target: SemanticId) -> bool { !self.inner_targets.contains(&target) }
}
impl ViewPolicy {
    /// Apply request role policy to compiler-owned structural facts and the actual bound view.
    /// This inspects no case values, covariance numbers or mutable provider observations.
    pub fn roles(&self, source: &AdmittedModeling, operation: Option<&CasePlan>) -> ViewRoles {
        let inner_unknowns = source.implicit_systems().flat_map(|i| i.unknowns.iter().copied())
            .filter(|id| !source.inputs.contains(id)).collect::<BTreeSet<_>>();
        let mut inner_targets = inner_unknowns.clone();
        inner_targets.extend(source.implicit_systems().flat_map(|i| i.residuals.iter().flat_map(|r| r.rows.iter().copied()))
            .filter(|id| !source.case().rows().iter().any(|row| row.id == *id)));
        let nested_suppliers = source.implicit_systems().filter(|supplier| supplier.unknowns.iter().all(|id| inner_unknowns.contains(id)))
            .map(|supplier| supplier.descriptor.spec().key()).collect();
        let structure = operation.map_or(source.case(), CasePlan::structure);
        let covariance_parameters = structure.parameters().iter().map(|p| p.id).collect::<BTreeSet<_>>();
        let propagation_outputs = operation.map_or_else(BTreeSet::new, |plan| plan.columns().iter().copied().collect());
        let sensitivity_parameters = if self.sensitivity { covariance_parameters.difference(&propagation_outputs).copied().collect() } else { BTreeSet::new() };
        ViewRoles { inner_targets, inner_unknowns, nested_suppliers, covariance_parameters, sensitivity_parameters, propagation_outputs }
    }
}
/// Selected compiler-owned view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerView {
    /// Keep the admitted operation view.
    Effective,
    /// Use the compiler-prepared original equations.
    Original,
}
/// Explicit premise supporting the choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewReason {
    /// No original-space consumer requested promotion.
    NoPromotionRequest,
    /// Sensitivity retains the operation view.
    Sensitivity,
    /// No supplier has established, unrestricted non-relation meaning.
    NoEligibleSupplier,
    /// The compiler supplied no original-equation view.
    NoOriginalView,
    /// The requested consumer and eligible supplier permit promotion.
    EligiblePromotion,
}
/// Pure decision and the actual source-derived suppliers that permit it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewChoice {
    /// Actual consumed policy, retained for attribution.
    pub policy: ViewPolicy,
    /// Selected view.
    pub view: CompilerView,
    /// Why this view was selected.
    pub reason: ViewReason,
    /// Established, unrestricted suppliers with non-relation meaning.
    pub eligible_suppliers: Arc<BTreeSet<SemanticId>>,
    /// The admitted source itself declares an accuracy goal.
    pub source_accuracy: bool,
}
#[salsa::interned(heap_size = view_key_heap)]
struct ViewKey<'db> {
    policy: ViewPolicy,
    original_available: bool,
    suppliers: Arc<BTreeSet<SemanticId>>,
    source_accuracy: bool,
}
fn view_key_heap(
    (_, _, suppliers, _): &(ViewPolicy, bool, Arc<BTreeSet<SemanticId>>, bool),
) -> usize {
    suppliers.len() * 128
}
#[salsa::tracked(returns(clone),lru=256,heap_size=view_heap)]
fn choose_view(db: &dyn CompilerDb, key: ViewKey<'_>) -> Arc<ViewChoice> {
    checkpoint(db);
    let policy = *key.policy(db);
    let reason = if policy.sensitivity {
        ViewReason::Sensitivity
    } else if !(policy.automatic_composition
        || policy.reconstruction
        || policy.declared_accuracy
        || *key.source_accuracy(db))
    {
        ViewReason::NoPromotionRequest
    } else if key.suppliers(db).is_empty() {
        ViewReason::NoEligibleSupplier
    } else if !*key.original_available(db) {
        ViewReason::NoOriginalView
    } else {
        ViewReason::EligiblePromotion
    };
    Arc::new(ViewChoice {
        policy,
        view: if reason == ViewReason::EligiblePromotion {
            CompilerView::Original
        } else {
            CompilerView::Effective
        },
        reason,
        eligible_suppliers: key.suppliers(db).clone(),
        source_accuracy: *key.source_accuracy(db),
    })
}
fn view_heap(value: &Arc<ViewChoice>) -> usize {
    size_of::<ViewChoice>() + value.eligible_suppliers.len() * 128
}
pub(super) type ViewChoices = BTreeMap<(bool, ViewPolicy), Arc<ViewChoice>>;
pub(super) fn view_choices(
    db: &dyn CompilerDb,
    admitted: &AdmittedModeling,
    original_view: Option<&AdmittedModeling>,
    model: &SpecializedModel,
) -> Arc<ViewChoices> {
    let source_accuracy = model.annotations.iter().any(|annotation| {
        matches!(
            annotation.value,
            pse_modeling::annotation::AnnotationValue::AccuracyGoal(_)
        )
    });
    let mut choices = BTreeMap::new();
    // All finite policy combinations are cheap descriptors. Prepared consumers
    // select one without acquiring another mutable compiler workspace.
    for original in [false, true]
        .into_iter()
        .filter(|original| !*original || original_view.is_some())
    {
        let source = if original {
            admitted
        } else {
            original_view.unwrap_or(admitted)
        };
        let suppliers = Arc::new(
            source
                .implicit_systems()
                .filter(|supplier| {
                    supplier.selection.neighborhood_evidence != SelectionNeighborhood::Unestablished
                        && !matches!(supplier.selection.meaning, ImplicitMeaning::Relation)
                        && supplier.selection.restriction.is_none()
                })
                .map(|supplier| supplier.descriptor.spec().id)
                .collect::<BTreeSet<_>>(),
        );
        for bits in 0..16 {
            let policy = ViewPolicy {
                automatic_composition: bits & 1 != 0,
                reconstruction: bits & 2 != 0,
                declared_accuracy: bits & 4 != 0,
                sensitivity: bits & 8 != 0,
            };
            choices.insert(
                (original, policy),
                choose_view(
                    db,
                    ViewKey::new(db, policy, original, suppliers.clone(), source_accuracy),
                ),
            );
        }
    }
    Arc::new(choices)
}
pub(super) fn configure(db: &mut CompilerDatabase, capacity: usize) {
    annotations::set_lru_capacity(db, capacity.max(3));
    choose_view::set_lru_capacity(db, capacity.max(32));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn workspace() -> CompilerWorkspace {
        CompilerWorkspace::new(
            crate::authored_transfer_tests::context(),
            WorkspaceLimits::default(),
        )
        .unwrap()
    }
    fn prepare(source: &str) -> PreparedModeling {
        let rows = crate::authored_transfer_tests::rows(source);
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = workspace();
        workspace.publish_modeling(rows, PhysicalScope::default()).unwrap();
        workspace.prepare_modeling_cancellable(root, pse_modeling::specialize::root_instance(root), Bindings::default(), Limits::default(), Arc::new(AtomicBool::new(false))).unwrap()
    }
    #[test]
    fn numerical_inventory_transitive_support_and_unknown_crosswalk_refuse() {
        let product = prepare("package p {def Root {param p:Scalar=2; var x:Scalar; let first:Scalar=x+p; let second:Scalar=first*2; eq row:x==1; annotation engineering_scale x(kind=magnitude,value=second); annotation nominal x(3);}}");
        let plan = product.numerical_annotations(NumericalAnnotationMode::Steady).unwrap();
        let (index, hint) = plan.hints.iter().enumerate().find(|(_, hint)| hint.kind == ModelingHint::EngineeringScaleValue).unwrap();
        let x = product.model.symbols.values().find(|symbol| symbol.lineage.path.ends_with(".x")).unwrap().id;
        let first = product.model.symbols.values().find(|symbol| symbol.lineage.path.ends_with(".first")).unwrap().id;
        assert!(hint.reads.contains(&first), "derived member support must survive independent-input filtering");
        assert!(hint.reads.contains(&x), "changing coordinate dependence is transitive");
        assert!(plan.require_frozen(index, &BTreeSet::from([x])).is_err());
        assert!(plan.require_frozen(index, &BTreeSet::new()).is_ok());
        let mut unknown = plan.clone();
        let mut hints = unknown.hints.to_vec(); hints[index].target = SemanticId::from_bytes([0xf4;16]); unknown.hints = hints.into();
        assert!(unknown.require_frozen(index, &BTreeSet::new()).is_err());
        assert!(plan.require_frozen(plan.hints.len(), &BTreeSet::new()).is_err(), "an unissued observation cannot masquerade as empty support");
        let (constant_index, constant) = plan.hints.iter().enumerate().find(|(_, hint)| hint.kind == ModelingHint::Nominal).unwrap();
        assert!(constant.reads.is_empty());
        assert!(plan.require_frozen(constant_index, &BTreeSet::from([x])).is_ok());
    }
    #[test]
    fn numerical_inventory_dynamic_states_rows_integrals_and_original_inventory_are_distinct() {
        let product = prepare("package p {def Root {domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); param p:Scalar=0.5; var x[i in t]:Time; var y[i in t]:Scalar; conserve stock[i in t]:Time on t inventory x[i] flux p tolerance 1e-6{s}; eq initial:x[0{s}]==1{s}; eq algebraic[i in t]:y[i]==x[i]/1{s}; let area:Time=integral(i in t | p); let terminal:Scalar=area/1{s}; let view[i in t]:Time=2*x[i]; annotation nominal view(3{s}); annotation report area(\"integral\"); annotation report terminal(\"terminal\");}}");
        let steady = product.numerical_annotations(NumericalAnnotationMode::Steady).unwrap();
        let dynamic = product.numerical_annotations(NumericalAnnotationMode::Dynamic).unwrap();
        let state = *product.model.derivatives.keys().next().unwrap();
        let state_fact = dynamic.target(state).unwrap();
        assert_eq!(state_fact.kind, NumericalTarget::Variable);
        assert_eq!(state_fact.location, NumericalLocation::Time);
        assert_eq!(state_fact.role, NumericalTargetRole::DifferentialState);
        let inventory = *product.model.inventory_balances.keys().next().unwrap();
        assert_eq!(dynamic.target(inventory).unwrap().kind, NumericalTarget::Closure);
        assert_eq!(dynamic.target(inventory).unwrap().role, NumericalTargetRole::Inventory);
        assert_eq!(dynamic.target(inventory).unwrap().location, NumericalLocation::Closure);
        let integral = *product.model.integrals.keys().next().unwrap();
        assert_eq!(dynamic.target(integral).unwrap().location, NumericalLocation::Integral);
        let terminal = product.model.symbols.values().find(|symbol| symbol.lineage.path.ends_with(".terminal")).unwrap().id;
        assert_eq!(dynamic.target(terminal).unwrap().location, NumericalLocation::Terminal);
        for initial in &product.model.initial_equations {
            assert!(steady.target(*initial).unwrap().selected);
            assert!(!dynamic.target(*initial).unwrap().selected);
        }
        let nominal = dynamic.hints.iter().find(|hint| hint.kind == ModelingHint::Nominal).unwrap();
        let observable = dynamic.target(nominal.target).unwrap();
        assert_eq!(nominal.target_kind, NumericalTarget::Observable);
        assert!(observable.selected, "an authored observable nominal cannot silently disappear in dynamics");
        assert!(dynamic.selected_targets().any(|target| target.id == observable.id));
        assert!(dynamic.selected_targets().any(|target| target.kind == NumericalTarget::Row && target.location == NumericalLocation::Time));
    }
    #[test]
    fn numerical_view_roles_share_inner_demand_and_request_admission() {
        let product = prepare("package p {def Root {param p:Scalar=2; implicit inner {var y:Scalar; eq e:y==p; annotation start y(1); annotation nominal y(2);} realize r on inner using nested; var x:Scalar; eq e:x==inner.y;}}");
        let supplier = product.admitted.implicit_systems().next().unwrap();
        let policy = ViewPolicy { automatic_composition:false, reconstruction:false, declared_accuracy:false, sensitivity:true };
        let roles = policy.roles(&product.admitted, None);
        for unknown in &supplier.unknowns {
            assert!(roles.inner_unknowns.contains(unknown));
            assert!(!roles.admits_outer(*unknown));
            assert!(!roles.sensitivity_parameters.contains(unknown));
        }
        assert!(roles.nested_suppliers.contains(&supplier.descriptor.spec().key()));
        let parameter = product.model.symbols.values().find(|symbol| symbol.lineage.path.ends_with(".p")).unwrap().id;
        assert!(roles.covariance_parameters.contains(&parameter));
        assert!(roles.sensitivity_parameters.contains(&parameter));
        assert!(roles.propagation_outputs.is_empty());
        let structure = product.bound_structure(&BTreeMap::new()).unwrap();
        let plan = CasePlan::prepare(structure.structure, product.admitted.bodies.iter().map(|(key, body)| (*key, body.math.clone())).collect(), &product.context.quantities, DerivativeOrder::Value, Profile::default().assembly, &Arc::new(AtomicBool::new(false))).unwrap();
        let bound_roles = policy.roles(&product.admitted, Some(&plan));
        assert_eq!(bound_roles.propagation_outputs, plan.columns().iter().copied().collect());
        assert!(!bound_roles.propagation_outputs.is_empty());
        assert!(bound_roles.propagation_outputs.is_disjoint(&bound_roles.sensitivity_parameters));

        assert!(ViewPolicy { sensitivity:false, ..policy }.roles(&product.admitted, None).sensitivity_parameters.is_empty());
        let nested = &supplier.residuals[0].numerical_annotations;
        assert!(nested.targets.iter().all(|target| matches!(target.location, NumericalLocation::Nested(_))));
        assert!(nested.targets.iter().any(|target| target.kind == NumericalTarget::Row));
        assert!(nested.targets.iter().any(|target| target.kind == NumericalTarget::Variable));
    }
    #[test]
    fn numerical_inventory_refuses_unsupported_dynamic_annotations_explicitly() {
        let product = prepare("package p {def Root {domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]:Time; eq rate[i in t]:d(x[i])/di==1; eq initial:x[0{s}]==1{s}; let area:Time=integral(i in t | 1); annotation start area(1{s});}}");
        let dynamic = product.numerical_annotations(NumericalAnnotationMode::Dynamic).unwrap();
        let index = dynamic.hints.iter().position(|hint| hint.kind == ModelingHint::Start).unwrap();
        assert!(matches!(dynamic.admission(index), NumericalAdmission::Refused(_)));
        assert!(dynamic.require_supported().is_err());
    }
    #[test]
    fn nested_residual_row_nominal_has_an_exact_owner_in_every_mode() {
        let product = prepare("package p {def Root {param p:Scalar=2; implicit inner {var y:Scalar; eq e:y==p; annotation start y(1); annotation nominal e(1);} realize r on inner using nested; var x:Scalar; eq e:x==inner.y;}}");
        let residual = &product.admitted.implicit_systems().next().unwrap().residuals[0];
        let row = residual.rows[0];
        for mode in [NumericalAnnotationMode::Steady, NumericalAnnotationMode::Implicit, NumericalAnnotationMode::Dynamic] {
            let plan = product.numerical_annotations(mode).unwrap();
            let index = plan.hints.iter().position(|hint| hint.target == row && hint.kind == ModelingHint::Nominal).unwrap();
            assert_eq!(plan.admission(index), NumericalAdmission::NotApplicable);
            let fact = plan.target(row).unwrap();
            assert_eq!(fact.role, NumericalTargetRole::NestedRow);
            assert_eq!(fact.quantity, residual.numerical_annotations.target(row).unwrap().quantity);
            assert!(fact.nested);
            assert!(plan.require_supported().is_ok());
        }
        let inner = &residual.numerical_annotations;
        let index = inner.hints.iter().position(|hint| hint.target == row && hint.kind == ModelingHint::Nominal).unwrap();
        assert_eq!(inner.admission(index), NumericalAdmission::Supported);
    }
    #[test]
    fn view_choice_is_keyed_by_consumed_policy_and_complete_supplier_premise() {
        let workspace = workspace();
        for available in [false, true] {
            for eligible in [false, true] {
                for bits in 0..16 {
                    let policy = ViewPolicy {
                        automatic_composition: bits & 1 != 0,
                        reconstruction: bits & 2 != 0,
                        declared_accuracy: bits & 4 != 0,
                        sensitivity: bits & 8 != 0,
                    };
                    let suppliers = if eligible {
                        BTreeSet::from([SemanticId::from_bytes([7; 16])])
                    } else {
                        BTreeSet::new()
                    };
                    let key = ViewKey::new(
                        &workspace.db,
                        policy,
                        available,
                        Arc::new(suppliers.clone()),
                        false,
                    );
                    let choice = choose_view(&workspace.db, key);
                    let expected = if policy.sensitivity {
                        ViewReason::Sensitivity
                    } else if !(policy.automatic_composition
                        || policy.reconstruction
                        || policy.declared_accuracy)
                    {
                        ViewReason::NoPromotionRequest
                    } else if !eligible {
                        ViewReason::NoEligibleSupplier
                    } else if !available {
                        ViewReason::NoOriginalView
                    } else {
                        ViewReason::EligiblePromotion
                    };
                    assert_eq!(choice.reason, expected);
                    assert_eq!(choice.eligible_suppliers.as_ref(), &suppliers);
                    assert_eq!(
                        choice.view == CompilerView::Original,
                        expected == ViewReason::EligiblePromotion
                    );
                    assert!(Arc::ptr_eq(&choice, &choose_view(&workspace.db, key)));
                }
            }
        }
        let policy = ViewPolicy {
            automatic_composition: false,
            reconstruction: false,
            declared_accuracy: false,
            sensitivity: false,
        };
        let source_goal = choose_view(
            &workspace.db,
            ViewKey::new(
                &workspace.db,
                policy,
                true,
                Arc::new(BTreeSet::from([SemanticId::from_bytes([7; 16])])),
                true,
            ),
        );
        assert_eq!(source_goal.view, CompilerView::Original);
        assert!(source_goal.source_accuracy);
    }
    #[test]
    fn annotation_modes_preserve_source_types_scaling_and_original_constant_bound_rule() {
        let rows = crate::authored_transfer_tests::rows(
            "package p {def Root {var x:Scalar; let limit:Scalar=0; eq row:x==1; annotation bounds x(limit,2); annotation start x(0); annotation nominal x(1); annotation scale row(inverseSum);}}",
        );
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = workspace();
        workspace
            .publish_modeling(rows, PhysicalScope::default())
            .unwrap();
        let product = workspace
            .prepare_modeling_cancellable(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let steady = product
            .numerical_annotations(NumericalAnnotationMode::Steady)
            .unwrap();
        let dynamic = product
            .numerical_annotations(NumericalAnnotationMode::Dynamic)
            .unwrap();
        assert_eq!(steady.hints, dynamic.hints);
        assert!(Arc::ptr_eq(&steady.scales, &dynamic.scales));
        assert_eq!(
            steady.start_precedence,
            [
                NumericalStartSource::SuppliedValue,
                NumericalStartSource::AuthoredHint
            ]
        );
        assert_eq!(
            dynamic.start_precedence,
            [
                NumericalStartSource::InitialCondition,
                NumericalStartSource::SuppliedValue,
                NumericalStartSource::AuthoredHint
            ]
        );
        let lower = dynamic
            .hints
            .iter()
            .find(|hint| hint.kind == ModelingHint::Lower)
            .unwrap();
        let upper = dynamic
            .hints
            .iter()
            .find(|hint| hint.kind == ModelingHint::Upper)
            .unwrap();
        assert!(
            !lower.constant_bound,
            "an authored member reference remains nonconstant after constant-member folding"
        );
        assert!(upper.constant_bound);
        assert_eq!(lower.declaration, upper.declaration);
        assert_eq!(lower.target_kind, NumericalTarget::Variable);
        assert_eq!(steady.scales.len(), 1);
        assert_eq!(steady.outer_scales().count(), 1);
        assert_eq!(
            dynamic.outer_scales().count(),
            0,
            "integrated mode retains physical budgets rather than steady conditioning"
        );
        assert!(
            matches!(&steady.scales[0].observations, ScalingObservations::Rows(rows) if !rows.is_empty())
        );
        assert!(steady.bounds_overrides_first && dynamic.bounds_overrides_first);
        let starts = NumericalStartValues {
            initial_condition: Some(4.0),
            semantic_anchor: Some(3.0),
            supplied_value: Some(2.0),
            authored_hint: Some(1.0),
        };
        assert_eq!(
            steady.select_start(starts),
            Some((2.0, NumericalStartSource::SuppliedValue))
        );
        assert_eq!(
            dynamic.select_start(starts),
            Some((4.0, NumericalStartSource::InitialCondition))
        );
        assert_eq!(dynamic.constant_sign_bounds().count(), 1);
        assert_eq!(steady.constant_sign_bounds().count(), 0);
        assert_eq!(steady.effective_bound(Some(None), Some(1.0)), None);
        assert_eq!(steady.effective_bound(None, Some(1.0)), Some(1.0));
        assert_eq!(
            steady.effective_bound(Some(Some(2.0)), Some(1.0)),
            Some(2.0)
        );
    }
    #[test]
    fn nested_residual_annotation_product_preserves_anchor_precedence_and_hint_positions() {
        let rows = crate::authored_transfer_tests::rows(
            "package p {def Root {param p:Scalar=2; implicit inner {var y:Scalar; eq e:y==p; annotation start y(1); annotation nominal y(2); annotation bounds y(0,4);} realize r on inner using nested; var x:Scalar; eq e:x==inner.y;}}",
        );
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = workspace();
        workspace
            .publish_modeling(rows, PhysicalScope::default())
            .unwrap();
        let product = workspace
            .prepare_modeling_cancellable(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let inner = product.admitted.implicit_systems().next().unwrap();
        for residual in &inner.residuals {
            let plan = &residual.numerical_annotations;
            assert_eq!(plan.mode, NumericalAnnotationMode::Implicit);
            assert_eq!(
                plan.select_start(NumericalStartValues {
                    semantic_anchor: Some(3.0),
                    supplied_value: Some(2.0),
                    authored_hint: Some(1.0),
                    ..NumericalStartValues::default()
                }),
                Some((3.0, NumericalStartSource::SemanticAnchor))
            );
            assert_eq!(
                plan.start_precedence,
                [
                    NumericalStartSource::SemanticAnchor,
                    NumericalStartSource::SuppliedValue,
                    NumericalStartSource::AuthoredHint
                ]
            );
            assert_eq!(plan.hints.len(), residual.hints.as_ref().map_or(0, |body| body.quantities().len()));
            for (position, hint) in plan.hints.iter().enumerate() {
                assert_eq!(hint.observation, AnnotationObservation::Position(position));
                assert_eq!(hint.target_kind, NumericalTarget::Variable);
                assert!(!hint.nested);
            }
        }
    }
}
