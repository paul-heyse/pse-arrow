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
    /// Source-selected original-term scaling instructions.
    pub scales: Arc<[NumericalScale]>,
    /// Highest-precedence source first.
    pub start_precedence: Vec<NumericalStartSource>,
    /// Explicit endpoint overrides replace authored bounds, including explicit absence.
    pub bounds_overrides_first: bool,
}
impl NumericalAnnotations {
    /// Positive known descriptor storage, excluding independently owned mathematics.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.hints.len() * size_of::<NumericalAnnotation>()
            + self.hints.iter().map(|hint| hint.reads.len() * 128).sum::<usize>()
            + self.scales.len() * size_of::<NumericalScale>()
            + self.scales.iter().map(|scale| match &scale.observations { ScalingObservations::Rows(rows) => rows.capacity() * size_of::<SemanticId>(), ScalingObservations::Range(_) => 0 }).sum::<usize>()
            + self.start_precedence.capacity() * size_of::<NumericalStartSource>()
    }
    /// Outer-bound/nominal consumers exclude nested-owned coordinates.
    pub fn outer_hints(&self) -> impl Iterator<Item = &NumericalAnnotation> { self.hints.iter().filter(|hint| self.mode != NumericalAnnotationMode::Steady || !hint.nested) }
    /// Steady conditioning and nested residual scaling keep their existing owners.
    /// Integrated mode uses physical nominal/accuracy budgets, not steady row conditioning.
    pub fn outer_scales(&self) -> impl Iterator<Item = &NumericalScale> { self.scales.iter().filter(|scale| match self.mode { NumericalAnnotationMode::Steady => !scale.nested, NumericalAnnotationMode::Implicit => true, NumericalAnnotationMode::Dynamic => false }) }
    /// Only dynamic, originally member-free bounds can establish a constant sign.
    pub fn constant_sign_bounds(&self) -> impl Iterator<Item = &NumericalAnnotation> { self.hints.iter().filter(|hint| self.mode == NumericalAnnotationMode::Dynamic && hint.constant_bound && matches!(hint.kind, ModelingHint::Lower | ModelingHint::Upper)) }
    /// Apply the compiler's mode-specific precedence to fresh evaluated values.
    /// Finite/domain checks remain at the current numerical receiving boundary.
    pub fn select_start(&self, values: NumericalStartValues) -> Option<(f64, NumericalStartSource)> {
        self.start_precedence.iter().find_map(|source| {
            let value = match source { NumericalStartSource::InitialCondition => values.initial_condition, NumericalStartSource::SemanticAnchor => values.semantic_anchor, NumericalStartSource::SuppliedValue => values.supplied_value, NumericalStartSource::AuthoredHint => values.authored_hint };
            value.map(|value| (value, *source))
        })
    }
    /// Explicit endpoint absence removes an authored bound; an unspecified endpoint inherits it.
    pub fn effective_bound(&self, endpoint_override: Option<Option<f64>>, authored: Option<f64>) -> Option<f64> {
        endpoint_override.unwrap_or(authored)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct AnnotationFacts {
    pub mode: NumericalAnnotationMode,
    pub hints: Arc<[NumericalAnnotation]>,
    pub scales: Arc<[NumericalScale]>,
}
#[salsa::interned(heap_size = annotation_key_heap)]
struct AnnotationKey<'db> { facts: Arc<AnnotationFacts> }
fn annotation_key_heap((facts,): &(Arc<AnnotationFacts>,)) -> usize {
    size_of::<AnnotationFacts>() + facts.hints.len() * size_of::<NumericalAnnotation>()
        + facts.hints.iter().map(|hint| hint.reads.len() * 128).sum::<usize>()
        + facts.scales.len() * size_of::<NumericalScale>()
        + facts.scales.iter().map(|scale| match &scale.observations { ScalingObservations::Rows(rows) => rows.capacity() * size_of::<SemanticId>(), ScalingObservations::Range(_) => 0 }).sum::<usize>()
}
#[salsa::tracked(returns(clone),lru=256,heap_size=annotation_heap)]
fn annotations(db: &dyn CompilerDb, key: AnnotationKey<'_>) -> Arc<NumericalAnnotations> {
    checkpoint(db);
    let facts = key.facts(db);
    use NumericalStartSource as S;
    Arc::new(NumericalAnnotations {
        mode: facts.mode, hints: facts.hints.clone(), scales: facts.scales.clone(),
        start_precedence: match facts.mode {
            NumericalAnnotationMode::Steady => vec![S::SuppliedValue, S::AuthoredHint],
            NumericalAnnotationMode::Implicit => vec![S::SemanticAnchor, S::SuppliedValue, S::AuthoredHint],
            NumericalAnnotationMode::Dynamic => vec![S::InitialCondition, S::SuppliedValue, S::AuthoredHint],
        },
        bounds_overrides_first: true,
    })
}
fn annotation_heap(value: &Arc<NumericalAnnotations>) -> usize { value.retained_bytes() }
pub(super) fn lower(db: &dyn CompilerDb, facts: AnnotationFacts) -> Arc<NumericalAnnotations> {
    annotations(db, AnnotationKey::new(db, Arc::new(facts)))
}
pub(super) type AnnotationPlans = BTreeMap<NumericalAnnotationMode, Arc<NumericalAnnotations>>;
pub(super) fn projection_annotations(db: &dyn CompilerDb, p: &Projection, constant_bounds: &BTreeSet<SemanticId>) -> Arc<AnnotationPlans> {
    let nested = p.implicit.iter().flat_map(|system| system.annotation_inventory()).collect::<BTreeSet<_>>();
    let variables = p.free.keys().copied().chain(p.implicit.iter().flat_map(|system| system.annotation_variables())).collect::<BTreeSet<_>>();
    let equations = p.outputs.iter().filter_map(|output| match output { ModelingOutput::Equation { id, .. } => Some(*id), _ => None }).collect::<BTreeSet<_>>();
    let hints: Arc<[NumericalAnnotation]> = p.outputs.iter().zip(&p.expressions).zip(&p.quantities).filter_map(|((output, expression), quantity)| {
        let ModelingOutput::Hint { target, declaration, kind } = output else { return None; };
        Some(NumericalAnnotation { target: *target, declaration: *declaration, kind: *kind, quantity: *quantity,
            observation: AnnotationObservation::Row(output.row_id()),
            target_kind: if variables.contains(target) { NumericalTarget::Variable } else if equations.contains(target) { NumericalTarget::Row } else { NumericalTarget::Observable },
            nested: nested.contains(target), constant_bound: constant_bounds.contains(&output.row_id()),
            reads: expression.free_paths().into_iter().filter_map(dependencies::symbol).collect(),
        })
    }).collect();
    let scales: Arc<[NumericalScale]> = p.annotation_scales.iter().cloned().map(|mut scale| { scale.nested = nested.contains(&scale.target); scale }).collect();
    let mut plans = BTreeMap::new();
    for mode in [NumericalAnnotationMode::Steady, NumericalAnnotationMode::Implicit, NumericalAnnotationMode::Dynamic] {
        plans.insert(mode, lower(db, AnnotationFacts { mode, hints: hints.clone(), scales: scales.clone() }));
    }
    Arc::new(plans)
}
pub(super) fn plans_bytes(plans: &AnnotationPlans) -> usize {
    plans.len() * 128 + plans.values().map(|plan| plan.retained_bytes()).sum::<usize>()
}
pub(super) fn choices_bytes(choices: &ViewChoices) -> usize {
    choices.len() * 128 + choices.values().map(view_heap).sum::<usize>()
}

/// Actual consumed solve-policy inputs governing original-view promotion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
/// Selected compiler-owned view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerView { /// Keep the admitted operation view.
    Effective, /// Use the compiler-prepared original equations.
    Original }
/// Explicit premise supporting the choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewReason { /// No original-space consumer requested promotion.
    NoPromotionRequest, /// Sensitivity retains the operation view.
    Sensitivity, /// No supplier has established, unrestricted non-relation meaning.
    NoEligibleSupplier, /// The compiler supplied no original-equation view.
    NoOriginalView, /// The requested consumer and eligible supplier permit promotion.
    EligiblePromotion }
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
struct ViewKey<'db> { policy: ViewPolicy, original_available: bool, suppliers: Arc<BTreeSet<SemanticId>>, source_accuracy: bool }
fn view_key_heap((_, _, suppliers, _): &(ViewPolicy, bool, Arc<BTreeSet<SemanticId>>, bool)) -> usize { suppliers.len() * 128 }
#[salsa::tracked(returns(clone),lru=256,heap_size=view_heap)]
fn choose_view(db: &dyn CompilerDb, key: ViewKey<'_>) -> Arc<ViewChoice> {
    checkpoint(db);
    let policy = *key.policy(db);
    let reason = if policy.sensitivity { ViewReason::Sensitivity }
        else if !(policy.automatic_composition || policy.reconstruction || policy.declared_accuracy || *key.source_accuracy(db)) { ViewReason::NoPromotionRequest }
        else if key.suppliers(db).is_empty() { ViewReason::NoEligibleSupplier }
        else if !*key.original_available(db) { ViewReason::NoOriginalView }
        else { ViewReason::EligiblePromotion };
    Arc::new(ViewChoice { policy, view: if reason == ViewReason::EligiblePromotion { CompilerView::Original } else { CompilerView::Effective }, reason, eligible_suppliers: key.suppliers(db).clone(), source_accuracy: *key.source_accuracy(db) })
}
fn view_heap(value: &Arc<ViewChoice>) -> usize { size_of::<ViewChoice>() + value.eligible_suppliers.len() * 128 }
pub(super) type ViewChoices = BTreeMap<(bool, ViewPolicy), Arc<ViewChoice>>;
pub(super) fn view_choices(db: &dyn CompilerDb, admitted: &AdmittedModeling, original_view: Option<&AdmittedModeling>, model: &SpecializedModel) -> Arc<ViewChoices> {
    let source_accuracy = model.annotations.iter().any(|annotation| matches!(annotation.value, pse_modeling::annotation::AnnotationValue::AccuracyGoal(_)));
    let mut choices = BTreeMap::new();
    // All finite policy combinations are cheap descriptors. Prepared consumers
    // select one without acquiring another mutable compiler workspace.
    for original in [false, true].into_iter().filter(|original| !*original || original_view.is_some()) {
        let source = if original { admitted } else { original_view.unwrap_or(admitted) };
        let suppliers = Arc::new(source.implicit_systems().filter(|supplier| supplier.selection.neighborhood_evidence != SelectionNeighborhood::Unestablished && !matches!(supplier.selection.meaning, ImplicitMeaning::Relation) && supplier.selection.restriction.is_none()).map(|supplier| supplier.descriptor.spec().id).collect::<BTreeSet<_>>());
        for bits in 0..16 {
            let policy = ViewPolicy { automatic_composition: bits & 1 != 0, reconstruction: bits & 2 != 0, declared_accuracy: bits & 4 != 0, sensitivity: bits & 8 != 0 };
            choices.insert((original, policy), choose_view(db, ViewKey::new(db, policy, original, suppliers.clone(), source_accuracy)));
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
        CompilerWorkspace::new(crate::authored_transfer_tests::context(), WorkspaceLimits::default()).unwrap()
    }
    #[test]
    fn view_choice_is_keyed_by_consumed_policy_and_complete_supplier_premise() {
        let workspace = workspace();
        for available in [false, true] {
            for eligible in [false, true] {
                for bits in 0..16 {
                    let policy = ViewPolicy { automatic_composition: bits & 1 != 0, reconstruction: bits & 2 != 0, declared_accuracy: bits & 4 != 0, sensitivity: bits & 8 != 0 };
                    let suppliers = if eligible { BTreeSet::from([SemanticId::from_bytes([7; 16])]) } else { BTreeSet::new() };
                    let key = ViewKey::new(&workspace.db, policy, available, Arc::new(suppliers.clone()), false);
                    let choice = choose_view(&workspace.db, key);
                    let expected = if policy.sensitivity { ViewReason::Sensitivity } else if !(policy.automatic_composition || policy.reconstruction || policy.declared_accuracy) { ViewReason::NoPromotionRequest } else if !eligible { ViewReason::NoEligibleSupplier } else if !available { ViewReason::NoOriginalView } else { ViewReason::EligiblePromotion };
                    assert_eq!(choice.reason, expected);
                    assert_eq!(choice.eligible_suppliers.as_ref(), &suppliers);
                    assert_eq!(choice.view == CompilerView::Original, expected == ViewReason::EligiblePromotion);
                    assert!(Arc::ptr_eq(&choice, &choose_view(&workspace.db, key)));
                }
            }
        }
        let policy = ViewPolicy { automatic_composition: false, reconstruction: false, declared_accuracy: false, sensitivity: false };
        let source_goal = choose_view(&workspace.db, ViewKey::new(&workspace.db, policy, true, Arc::new(BTreeSet::from([SemanticId::from_bytes([7; 16])])), true));
        assert_eq!(source_goal.view, CompilerView::Original);
        assert!(source_goal.source_accuracy);
    }
    #[test]
    fn annotation_modes_preserve_source_types_scaling_and_original_constant_bound_rule() {
        let rows = crate::authored_transfer_tests::rows("package p {def Root {var x:Scalar; let limit:Scalar=0; eq row:x==1; annotation bounds x(limit,2); annotation start x(0); annotation nominal x(1); annotation scale row(inverseSum);}}");
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = workspace();
        workspace.publish_modeling(rows, PhysicalScope::default()).unwrap();
        let product = workspace.prepare_modeling_cancellable(root, pse_modeling::specialize::root_instance(root), Bindings::default(), Limits::default(), Arc::new(AtomicBool::new(false))).unwrap();
        let steady = product.numerical_annotations(NumericalAnnotationMode::Steady).unwrap();
        let dynamic = product.numerical_annotations(NumericalAnnotationMode::Dynamic).unwrap();
        assert!(Arc::ptr_eq(&steady.hints, &dynamic.hints));
        assert!(Arc::ptr_eq(&steady.scales, &dynamic.scales));
        assert_eq!(steady.start_precedence, [NumericalStartSource::SuppliedValue, NumericalStartSource::AuthoredHint]);
        assert_eq!(dynamic.start_precedence, [NumericalStartSource::InitialCondition, NumericalStartSource::SuppliedValue, NumericalStartSource::AuthoredHint]);
        let lower = dynamic.hints.iter().find(|hint| hint.kind == ModelingHint::Lower).unwrap();
        let upper = dynamic.hints.iter().find(|hint| hint.kind == ModelingHint::Upper).unwrap();
        assert!(!lower.constant_bound, "an authored member reference remains nonconstant after constant-member folding");
        assert!(upper.constant_bound);
        assert_eq!(lower.declaration, upper.declaration);
        assert_eq!(lower.target_kind, NumericalTarget::Variable);
        assert_eq!(steady.scales.len(), 1);
        assert_eq!(steady.outer_scales().count(), 1);
        assert_eq!(dynamic.outer_scales().count(), 0, "integrated mode retains physical budgets rather than steady conditioning");
        assert!(matches!(&steady.scales[0].observations, ScalingObservations::Rows(rows) if !rows.is_empty()));
        assert!(steady.bounds_overrides_first && dynamic.bounds_overrides_first);
        let starts = NumericalStartValues { initial_condition: Some(4.0), semantic_anchor: Some(3.0), supplied_value: Some(2.0), authored_hint: Some(1.0) };
        assert_eq!(steady.select_start(starts), Some((2.0, NumericalStartSource::SuppliedValue)));
        assert_eq!(dynamic.select_start(starts), Some((4.0, NumericalStartSource::InitialCondition)));
        assert_eq!(dynamic.constant_sign_bounds().count(), 1);
        assert_eq!(steady.constant_sign_bounds().count(), 0);
        assert_eq!(steady.effective_bound(Some(None), Some(1.0)), None);
        assert_eq!(steady.effective_bound(None, Some(1.0)), Some(1.0));
        assert_eq!(steady.effective_bound(Some(Some(2.0)), Some(1.0)), Some(2.0));
    }
    #[test]
    fn nested_residual_annotation_product_preserves_anchor_precedence_and_hint_positions() {
        let rows = crate::authored_transfer_tests::rows("package p {def Root {param p:Scalar=2; implicit inner {var y:Scalar; eq e:y==p; annotation start y(1); annotation nominal y(2); annotation bounds y(0,4);} realize r on inner using nested; var x:Scalar; eq e:x==inner.y;}}");
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = workspace();
        workspace.publish_modeling(rows, PhysicalScope::default()).unwrap();
        let product = workspace.prepare_modeling_cancellable(root, pse_modeling::specialize::root_instance(root), Bindings::default(), Limits::default(), Arc::new(AtomicBool::new(false))).unwrap();
        let inner = product.admitted.implicit_systems().next().unwrap();
        for residual in &inner.residuals {
            let plan = &residual.numerical_annotations;
            assert_eq!(plan.mode, NumericalAnnotationMode::Implicit);
            assert_eq!(plan.select_start(NumericalStartValues { semantic_anchor: Some(3.0), supplied_value: Some(2.0), authored_hint: Some(1.0), ..NumericalStartValues::default() }), Some((3.0, NumericalStartSource::SemanticAnchor)));
            assert_eq!(plan.start_precedence, [NumericalStartSource::SemanticAnchor, NumericalStartSource::SuppliedValue, NumericalStartSource::AuthoredHint]);
            assert_eq!(plan.hints.len(), residual.hint_targets.len());
            for (position, hint) in plan.hints.iter().enumerate() {
                assert_eq!(hint.observation, AnnotationObservation::Position(position));
                assert_eq!(hint.target_kind, NumericalTarget::Variable);
                assert!(!hint.nested);
            }
        }
    }
}
