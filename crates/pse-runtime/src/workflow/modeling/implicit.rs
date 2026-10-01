// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Numerical hint meaning is selected once, then resolved on each enclosing trial.
use super::*;
use crate::math::{modeling::ModelingInner, solves::NumericalInputs};
use pse_compiler::workspace::{ModelingCaseBindings, ModelingHint, ModelingVariableState, Profile};
use pse_math::{
    MathError,
    implicit::{Configuration, HintResolver, Options, Unknown},
};
use pse_model::{
    SemanticFrame,
    generated::enums::{NumericalSource, NumericalTarget},
    numerics::NumericalPolicy,
};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Debug)]
struct TrialHints {
    identity: pse_ids::ContentHash,
    quantities: Arc<pse_quantity::QuantityRegistry>,
    /// What its requirements name: the enclosing model and case, and the block's
    /// instance (`pse_model::lineage::Solved::stage`).
    lineage: pse_model::lineage::Lineage,
    unknowns: Vec<SemanticId>,
    rows: Vec<SemanticId>,
    hints: Vec<(SemanticId, DeclarationId, ModelingHint)>,
    scales: Vec<pse_compiler::workspace::ImplicitScale>,
    values: BTreeMap<SemanticId, f64>,
    states: BTreeMap<SemanticId, ModelingVariableState>,
    targets: Vec<pse_math::numerics::TargetSpec>,
    declarations: Vec<pse_math::numerics::SourcedRequirement>,
    policy: NumericalPolicy,
    controls: pse_backend_native::solve::Controls,
}
impl HintResolver for TrialHints {
    fn identity(&self) -> pse_ids::ContentHash {
        self.identity
    }
    fn time_limit(&self) -> std::time::Duration {
        self.controls.time_limit
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.hints.len() * 128
            + self.scales.len() * 128
            + self.unknowns.len() * 256
            + self.rows.len() * 128
            + self.targets.len() * 256
            + (self.declarations.len() + self.policy.requirements.len()) * 512
    }
    fn resolve(
        &self,
        observed: &[f64],
        terms: Option<&[f64]>,
        semantic_anchor: Option<&[f64]>,
    ) -> Result<(Vec<Unknown>, Options), MathError> {
        let failure = |message: &str| MathError::Contract(message.into());
        if observed.len() != self.hints.len() {
            return Err(failure("implicit hint observation extent"));
        }
        if semantic_anchor.is_some_and(|anchor| anchor.len() != self.unknowns.len()) {
            return Err(failure("implicit semantic anchor extent"));
        }
        let selected = self
            .hints
            .iter()
            .zip(observed)
            .map(|((id, source, kind), value)| ((*id, *kind), (*source, *value)))
            .collect::<BTreeMap<_, _>>();
        let hint = |id, kind| selected.get(&(id, kind)).map(|(_, v)| *v);
        let mut declarations = self.declarations.clone();
        for ((target, kind), (source, value)) in &selected {
            if *kind == ModelingHint::Nominal {
                declarations.push(cases::requirement(
                    self.lineage,
                    *target,
                    if self.unknowns.contains(target) {
                        NumericalTarget::Variable
                    } else {
                        NumericalTarget::Row
                    },
                    *source,
                    NumericalSource::ModelHint,
                    Some(*value),
                    None,
                ));
            }
        }
        if let Some(terms) = terms {
            if terms.len() != self.scales.last().map_or(0, |s| s.terms.end) {
                return Err(failure("implicit original-term extent"));
            }
            for scale in &self.scales {
                let values = terms
                    .get(scale.terms.clone())
                    .ok_or_else(|| failure("implicit original-term range"))?;
                let value = pse_math::numerics::term_scale(scale.scheme, values)?;
                declarations.push(cases::requirement(
                    self.lineage,
                    scale.row,
                    NumericalTarget::Row,
                    scale.source,
                    NumericalSource::DerivedNominal,
                    None,
                    Some(value),
                ));
            }
        }
        let numerics = pse_math::numerics::resolve(
            &self.quantities,
            &self.targets,
            &declarations,
            &self.policy,
        )?;
        let mut bounds = Vec::with_capacity(self.unknowns.len());
        let mut start = Vec::with_capacity(self.unknowns.len());
        for (index, id) in self.unknowns.iter().enumerate() {
            let state = self.states.get(id);
            if state.is_some_and(|s| s.fixed == Some(true)) {
                return Err(failure(
                    "fixing a nested unknown requires inline realization",
                ));
            }
            let lower = state
                .and_then(|s| s.lower)
                .unwrap_or_else(|| hint(*id, ModelingHint::Lower))
                .unwrap_or(f64::NEG_INFINITY);
            let upper = state
                .and_then(|s| s.upper)
                .unwrap_or_else(|| hint(*id, ModelingHint::Upper))
                .unwrap_or(f64::INFINITY);
            let value = semantic_anchor
                .and_then(|a| a.get(index))
                .copied()
                .or_else(|| {
                    self.values
                        .get(id)
                        .copied()
                        .or_else(|| hint(*id, ModelingHint::Start))
                })
                .ok_or_else(|| failure("missing deterministic start for implicit unknown"))?;
            if !value.is_finite()
                || lower.is_nan()
                || upper.is_nan()
                || lower > upper
                || value < lower
                || value > upper
            {
                return Err(MathError::Domain {
                    source_id: *id,
                    requirement: "implicit start outside its declared interval",
                });
            }
            bounds.push(Unknown {
                id: *id,
                lower,
                upper,
            });
            start.push(value);
        }
        let target = |id, kind| {
            numerics
                .targets
                .iter()
                .find(|t| t.id == id && t.kind == kind)
                .ok_or_else(|| failure("implicit physical budget absent"))
        };
        let options = Options {
            start,
            variable_nominals: self
                .unknowns
                .iter()
                .map(|id| target(*id, NumericalTarget::Variable).map(|t| t.nominal))
                .collect::<Result<_, _>>()?,
            variable_tolerance: self
                .unknowns
                .iter()
                .map(|id| target(*id, NumericalTarget::Variable).map(|t| t.budget))
                .collect::<Result<_, _>>()?,
            residual_tolerance: self
                .rows
                .iter()
                .map(|id| target(*id, NumericalTarget::Row).map(|t| t.budget))
                .collect::<Result<_, _>>()?,
            iterations: self.controls.iterations,
            time_limit: self.controls.time_limit,
            derivative_tolerance: self.policy.kkt.stationarity,
        };
        Ok((bounds, options))
    }
}
/// Original residual definitions of every implicit block a factorable export uses in
/// place of its realization (ADR-0105 §1). A case bound on an unknown replaces that
/// endpoint's bound hint, exactly as the trial resolver does for the evaluator, so the
/// exported interval is the one evaluation enforces.
pub(super) fn factorable_definitions(
    model: &ModelingPreparation,
    case: &ModelingCaseBindings,
) -> BTreeMap<pse_kernels::ProviderKey, pse_math::factorable::ImplicitDefinition> {
    let product = model.compiled();
    let states = case
        .variables
        .iter()
        .filter_map(|(path, state)| product.model.paths.get(path).map(|id| (*id, state)))
        .collect::<BTreeMap<_, _>>();
    product
        .admitted
        .implicit
        .values()
        .filter_map(|inner| {
            let (key, mut definition) = inner.factorable_definition()?;
            for (j, id) in inner.unknowns.iter().enumerate() {
                let Some(state) = states.get(id) else {
                    continue;
                };
                if let Some(lower) = state.lower {
                    definition.unknowns[j].0 = lower.unwrap_or(f64::NEG_INFINITY);
                    if let Some(bounds) = definition.bounds.as_mut() {
                        bounds.lower[j] = None;
                    }
                }
                if let Some(upper) = state.upper {
                    definition.unknowns[j].1 = upper.unwrap_or(f64::INFINITY);
                    if let Some(bounds) = definition.bounds.as_mut() {
                        bounds.upper[j] = None;
                    }
                }
            }
            Some((key, definition))
        })
        .collect()
}
/// The authored block instance an implicit stage solves; a generated rate system is no
/// authored instance (`pse_model::lineage::Solved::stage`).
fn stage_instance(inner: &pse_compiler::workspace::AdmittedImplicit) -> Option<InstanceId> {
    (inner.algorithm != pse_compiler::workspace::ImplicitAlgorithm::AffineRates)
        .then(|| InstanceId::from_id(inner.descriptor.spec().id))
}
impl ModelingPackage {
    #[expect(
        clippy::too_many_arguments,
        reason = "nested hints resolve from the model, case, numerical inputs, policy and controls under the compiler profile, cancellation and row selection"
    )]
    pub(super) async fn inner_registrations(
        &self,
        model: ModelingPreparation,
        case: &ModelingCaseBindings,
        numerical: &NumericalInputs,
        policy: &NumericalPolicy,
        controls: &pse_backend_native::solve::Controls,
        requested_output: pse_kernels::DerivativeOrder,
        compiler: Profile,
        cancel: &crate::CancelSource,
        rows: Option<&BTreeSet<SemanticId>>,
    ) -> Result<BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>, WorkflowError> {
        let product = model.compiled();
        if product.admitted.implicit.is_empty() {
            return Ok(self.registrations());
        }
        controls
            .validate()
            .map_err(crate::math::MathRuntimeError::from)?;
        let provider_demands = product
            .admitted
            .provider_demands_for(rows, requested_output)
            .map_err(crate::math::MathRuntimeError::from)?;
        let mut inputs = Vec::new();
        for inner in product
            .admitted
            .implicit_order_for(rows)
            .map_err(crate::math::MathRuntimeError::from)?
        {
            let mut configurations = BTreeMap::new();
            for residual in &inner.residuals {
                let belongs =
                    |id: &SemanticId| inner.unknowns.contains(id) || residual.rows.contains(id);
                let mut targets = inner
                    .descriptor
                    .spec()
                    .outputs
                    .iter()
                    .map(|p| pse_math::numerics::TargetSpec {
                        id: p.id,
                        kind: NumericalTarget::Variable,
                        quantity: p.quantity,
                        unit: p.unit,
                        integer: false,
                        declared_tolerance: None,
                    })
                    .collect::<Vec<_>>();
                for (id, quantity) in residual.rows.iter().zip(&residual.body.quantities) {
                    targets.push(pse_math::numerics::TargetSpec {
                        id: *id,
                        kind: NumericalTarget::Row,
                        quantity: *quantity,
                        unit: self
                            .quantities
                            .quantity_type(*quantity)
                            .map_err(|e| contract(e.to_string()))?
                            .canonical_unit,
                        integer: false,
                        declared_tolerance: None,
                    });
                }
                let mut policy = policy.clone();
                policy.requirements.retain(|r| belongs(&r.target_id));
                let declarations = numerical
                    .declarations
                    .iter()
                    .filter(|r| belongs(&r.declaration.target_id))
                    .cloned()
                    .collect::<Vec<_>>();
                let mut values = case
                    .values
                    .iter()
                    .filter_map(|(p, v)| {
                        product
                            .model
                            .paths
                            .get(p)
                            .filter(|id| inner.unknowns.contains(id))
                            .map(|id| (*id, *v))
                    })
                    .collect::<BTreeMap<_, _>>();
                // Affine elimination evaluates at a deterministic zero reference; this is
                // an algorithm coordinate, not a scientific initial condition or a warm start.
                if inner.algorithm == pse_compiler::workspace::ImplicitAlgorithm::AffineRates {
                    for id in &inner.unknowns {
                        values.entry(*id).or_insert(0.);
                    }
                }
                let states = case
                    .variables
                    .iter()
                    .filter_map(|(p, v)| {
                        product
                            .model
                            .paths
                            .get(p)
                            .filter(|id| inner.unknowns.contains(id))
                            .map(|id| (*id, v.clone()))
                    })
                    .collect::<BTreeMap<_, _>>();
                if states.values().any(|state| state.fixed == Some(true)) {
                    return Err(contract(
                        "fixing a nested unknown requires inline realization",
                    ));
                }
                if inner.selection.anchors.is_none()
                    && inner.unknowns.iter().any(|id| {
                        !values.contains_key(id)
                            && !residual.hint_targets.iter().any(|(target, _, kind)| {
                                target == id && *kind == ModelingHint::Start
                            })
                    })
                {
                    return Err(contract("missing deterministic start for implicit unknown"));
                }
                let mut hash =
                    pse_ids::FramedHasher::new(pse_ids::Frame::ModelingImplicitTrialHintsV1);
                hash.hash(&inner.descriptor.spec().identity())
                    .hash(&residual.body.spec.physical)
                    .id(&residual.id)
                    .hash(&policy.key());
                for r in &declarations {
                    r.source.frame(&mut hash);
                    r.declaration.frame(&mut hash);
                }
                for (id, value) in &values {
                    hash.id(id).u64(value.to_bits());
                }
                for (id, state) in &states {
                    hash.id(id).u64(match state.fixed {
                        None => 0,
                        Some(false) => 1,
                        Some(true) => 2,
                    });
                    for bound in [state.lower, state.upper] {
                        hash.bool(bound.is_some());
                        if let Some(value) = bound {
                            hash.bool(value.is_some());
                            if let Some(value) = value {
                                hash.u64(value.to_bits());
                            }
                        }
                    }
                }
                hash.u64(controls.iterations as u64)
                    .u64(controls.time_limit.as_secs())
                    .u64(controls.time_limit.subsec_nanos() as u64);
                configurations.insert(
                    residual.id,
                    Configuration::Hints(Arc::new(TrialHints {
                        identity: hash.finish_hash(),
                        quantities: self.quantities.clone(),
                        lineage: model.solved().stage(stage_instance(&inner)),
                        unknowns: inner.unknowns.clone(),
                        rows: residual.rows.clone(),
                        hints: residual.hint_targets.clone(),
                        scales: residual.scales.clone(),
                        values,
                        states,
                        targets,
                        declarations,
                        policy,
                        controls: controls.clone(),
                    })),
                );
            }
            inputs.push(ModelingInner {
                admitted: inner,
                configurations,
            });
        }
        Ok(self
            .runtime
            .shared
            .math()
            .modeling_inner_providers(
                inputs,
                self.accelerators.clone(),
                self.registrations(),
                provider_demands,
                compiler,
                cancel,
            )
            .await?)
    }
}

#[cfg(test)]
#[cfg(feature = "solver-kinsol")]
mod tests {
    use super::*;
    use pse_compiler::workspace::ModelingOutput;
    use pse_math::binding::CaseValues;
    /// A definition root specialized as its own instance.
    fn solved(root: SemanticId) -> pse_model::lineage::Solved {
        pse_model::lineage::Solved::new(
            DeclarationId::from_id(root),
            pse_model::generated::enums::ModelingDeclarationKind::Definition,
            InstanceId::from_id(root),
        )
    }
    #[test]
    fn kernel_nested_original_term_nominals_follow_numerical_precedence() {
        let physical = super::super::super::tests::physical();
        let quantity = physical.quantities.neutral_dimensionless().unwrap();
        let unit = physical
            .quantities
            .quantity_type(quantity)
            .unwrap()
            .canonical_unit;
        let x = SemanticId::from_bytes([171; 16]);
        let row = SemanticId::from_bytes([172; 16]);
        let source = SemanticId::from_bytes([173; 16]);
        let declaration = DeclarationId::from(source);
        let mut resolver = TrialHints {
            identity: pse_ids::ContentHash::from_bytes([0; 32]),
            quantities: physical.quantities,
            lineage: solved(source).stage(Some(InstanceId::from_id(source))),
            unknowns: vec![x],
            rows: vec![row],
            hints: vec![(x, declaration, ModelingHint::Nominal)],
            scales: vec![pse_compiler::workspace::ImplicitScale {
                row,
                source: declaration,
                scheme: pse_model::generated::enums::ConstraintScalingScheme::InverseSum,
                terms: 0..3,
            }],
            values: BTreeMap::from([(x, 1.)]),
            states: BTreeMap::new(),
            targets: vec![(x, NumericalTarget::Variable), (row, NumericalTarget::Row)]
                .into_iter()
                .map(|(id, kind)| pse_math::numerics::TargetSpec {
                    id,
                    kind,
                    quantity,
                    unit,
                    integer: false,
                    declared_tolerance: None,
                })
                .collect(),
            declarations: vec![],
            policy: NumericalPolicy::default(),
            controls: Default::default(),
        };
        let (_, options) = resolver.resolve(&[3.], Some(&[9., -4., 0.]), None).unwrap();
        assert_eq!(options.variable_nominals, vec![3.]);
        assert!((options.residual_tolerance[0] - 13e-8).abs() < 1e-20);
        resolver.declarations.push(cases::requirement(
            solved(source).stage(Some(InstanceId::from_id(source))),
            row,
            NumericalTarget::Row,
            DeclarationId::from(pse_ids::named_id(source, "override")),
            NumericalSource::Model,
            Some(7.),
            None,
        ));
        let (_, options) = resolver.resolve(&[3.], Some(&[9., -4., 0.]), None).unwrap();
        assert!((options.residual_tolerance[0] - 7e-8).abs() < 1e-20);
        assert!(
            resolver
                .resolve(&[3.], Some(&[f64::NAN, 4., 0.]), None)
                .is_err()
        );
        resolver.values.clear();
        let (_, anchored) = resolver
            .resolve(&[3.], Some(&[9., -4., 0.]), Some(&[4.]))
            .unwrap();
        assert_eq!(anchored.start, vec![4.]);
        assert_eq!(anchored.variable_nominals, vec![3.]);
        assert!(resolver.resolve(&[3.], Some(&[9., -4., 0.]), None).is_err());
        assert!(
            resolver
                .resolve(&[3.], Some(&[9., -4., 0.]), Some(&[]))
                .is_err()
        );
    }
    #[tokio::test]
    async fn kernel_nested_observations_ignore_undemanded_missing_starts() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let source = "package p { def Root { var x:Scalar; implicit missing {var y:Scalar; eq root:y==2;} realize r on missing using nested; eq independent:x==1; eq dependent:missing.y==x; } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).unwrap();
        let cancel = crate::CancelSource::new();
        let compiler = super::super::super::tests::compiler_profile();
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = model.compiled();
        let row = |name: &str| {
            product
                .model
                .equations
                .iter()
                .find(|r| r.lineage.path.rsplit('.').next() == Some(name))
                .unwrap()
                .id
        };
        let independent = row("independent");
        let dependent = row("dependent");
        assert_eq!(product.admitted.inputs.len(), 1);
        let point = CaseValues {
            scalars: BTreeMap::from([(product.admitted.inputs[0], 3.)]),
        };
        assert!(
            product
                .admitted
                .implicit_order_for(Some(&BTreeSet::from([independent])))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            product
                .admitted
                .implicit_order_for(Some(&BTreeSet::from([dependent])))
                .unwrap()
                .len(),
            1
        );
        let observed = package
            .observe(
                model.clone(),
                BTreeSet::from([independent]),
                point.clone(),
                compiler,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(observed[&independent], 2.);
        assert!(
            package
                .observe(model, BTreeSet::from([dependent]), point, compiler, &cancel)
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn kernel_nested_hints_order_siblings_before_outer_starts() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let source = "package p { def Root { implicit a {var y:Scalar; eq e:y==2; annotation start y(1);} realize ra on a using nested; implicit b {var z:Scalar; eq e:z==a.y+1; annotation start z(a.y); annotation bounds z(0,10);} realize rb on b using nested; var x:Scalar; eq pin:x==b.z; annotation start x(b.z); } }";
        for (text, succeeds) in [
            (source.to_string(), true),
            (
                source.replace("annotation start y(1)", "annotation start y(b.z)"),
                false,
            ),
        ] {
            let rows = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = rt.modeling_package(rows, physical.clone()).unwrap();
            let compiler = super::super::super::tests::compiler_profile();
            let cancel = crate::CancelSource::new();
            let mut solver = super::super::super::tests::profile();
            solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
                pse_backend_native::solve::Backend::Kinsol,
            );
            let result = package
                .prepare_solve(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    pse_kernels::DerivativeOrder::Second,
                    compiler,
                    solver,
                    NumericalInputs::default(),
                    &cancel,
                )
                .await;
            if !succeeds {
                assert!(result.is_err());
                continue;
            }
            let prepared = result.unwrap();
            let x = prepared.model.model.compiled().admitted.inputs[0];
            assert!((prepared.model.values.scalars[&x] - 3.).abs() < 1e-8);
            let result = package
                .solve_case(prepared, compiler, &cancel)
                .await
                .unwrap();
            assert!(result.accepted, "{:?}", result.validation_error);
        }
    }
    #[tokio::test]
    async fn kernel_nested_value_observation_propagates_native_demand_without_promoting_hints() {
        use pse_kernels::DerivativeOrder::{First, Second, Value};
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows=pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; implicit numerical select operational(w=1) settings(\"native.kinsol.v1\") {var w:Scalar;eq e:w*w==1;annotation bounds w(0.5,1.5);}realize n on numerical using nested;implicit outer {var y:Scalar;implicit child select branch(z>=0) {var z:Scalar;eq e:z*z==x;annotation start z(sqrt(x));annotation bounds z(0.1,10);}realize c on child using nested;eq e:y+child.z==x;annotation start y(numerical.w);annotation bounds y(0.1,10);}realize o on outer using nested;eq pin:outer.y==2;} }",
            SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default(),
        ).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = model.compiled();
        let pin = product
            .model
            .equations
            .iter()
            .find(|r| r.lineage.path.rsplit('.').next() == Some("pin"))
            .unwrap()
            .id;
        let selected = BTreeSet::from([pin]);
        let ordered = product
            .admitted
            .implicit_order_for(Some(&selected))
            .unwrap();
        assert_eq!(ordered.len(), 3);
        let numerical = ordered
            .iter()
            .find(|inner| {
                matches!(
                    inner.selection.meaning,
                    pse_compiler::workspace::ImplicitMeaning::Operational(_)
                )
            })
            .unwrap();
        let parent = ordered
            .iter()
            .find(|inner| !inner.residuals[0].body.math.providers().is_empty())
            .unwrap();
        let child = ordered
            .iter()
            .find(|inner| {
                inner.descriptor.spec().id != numerical.descriptor.spec().id
                    && inner.descriptor.spec().id != parent.descriptor.spec().id
            })
            .unwrap();
        assert_eq!(numerical.descriptor.spec().derivatives, Value);
        assert_eq!(child.descriptor.spec().derivatives, Second);
        let seeds = product
            .admitted
            .provider_demands_for(Some(&selected), Value)
            .unwrap();
        assert_eq!(seeds[&parent.descriptor.spec().key()], Value);
        // Hint-only inputs may be captured by the original case body's provider call.
        // Their presence is valid; the requested order must remain Value.
        assert!(
            seeds
                .get(&numerical.descriptor.spec().key())
                .is_none_or(|order| *order == Value)
        );
        // This calls the production MathService reverse walk. The native parent needs
        // first partials in its residual even when the original observation asks for values.
        let providers = package
            .inner_registrations(
                model.clone(),
                &ModelingCaseBindings::default(),
                &NumericalInputs::default(),
                &NumericalPolicy::default(),
                &pse_backend_native::solve::Controls::default(),
                Value,
                compiler,
                &cancel,
                Some(&selected),
            )
            .await
            .unwrap();
        assert_eq!(
            providers[&parent.descriptor.spec().key()]
                .spec()
                .derivatives,
            Value
        );
        assert_eq!(
            providers[&child.descriptor.spec().key()].spec().derivatives,
            First
        );
        assert_eq!(
            providers[&numerical.descriptor.spec().key()]
                .spec()
                .derivatives,
            Value
        );
        let point = CaseValues {
            scalars: BTreeMap::from([(product.admitted.inputs[0], 4.)]),
        };
        let observed = package
            .observe_registered(model, selected, point, compiler, providers, &cancel)
            .await
            .unwrap();
        assert!(observed[&pin].abs() < 1e-7, "{:?}", observed[&pin]);
    }
    #[tokio::test]
    async fn kernel_regime_selection_executes_branch_hints_and_refuses_ties() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows=pse_authoring::language::parse(
            "package p { def Root { param target:Scalar; implicit roots select minimum((y-target)*(y-target), 1e-8) { var y:Scalar; regime negative eligible(y<0) { eq root:y == -1; annotation start y(-0.5); annotation bounds y(-2,-0.1); } regime positive eligible(y>0) { eq root:y == 1; annotation start y(0.5); annotation bounds y(0.1,2); } } realize r on roots using nested; annotation report roots.y(\"selected\"); } }",
            SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = model.compiled();
        let target = product.admitted.inputs[0];
        let y = product.admitted.implicit.values().next().unwrap().unknowns[0];
        let row = ModelingOutput::Member(y).row_id();
        let inner = product.admitted.implicit.values().next().unwrap();
        assert_eq!(inner.residuals.len(), 2);
        assert_eq!(
            inner.descriptor.spec().derivatives,
            pse_kernels::DerivativeOrder::Second
        );
        assert!(inner.residuals.iter().all(|r| r.assessment.is_some()));
        for (point, expected) in [(-2., -1.), (2., 1.)] {
            let values = CaseValues {
                scalars: BTreeMap::from([(target, point)]),
            };
            let result = package
                .observe(
                    model.clone(),
                    BTreeSet::from([row]),
                    values,
                    compiler,
                    &cancel,
                )
                .await
                .unwrap();
            assert!((result[&row] - expected).abs() < 1e-8);
        }
        let error = package
            .observe(
                model.clone(),
                BTreeSet::from([row]),
                CaseValues {
                    scalars: BTreeMap::from([(target, 0.)]),
                },
                compiler,
                &cancel,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("tie"), "{error}");
    }
    /// A nested regime selection binds its derivatives to the regime of the start (M3).
    /// The outer Ipopt steps toward the other regime; every trial that crosses is refused
    /// as a recoverable trial, and the solve report counts the crossings by the outer
    /// iteration in which they occurred.
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn nested_stage_reports_regime_crossings_per_outer_iteration() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows=pse_authoring::language::parse(
            "package p { def Root { var target:Scalar; var s:Scalar; implicit roots select minimum((y-target)*(y-target), 1e-8) { var y:Scalar; regime negative eligible(y<0) { eq root:y == -1; annotation start y(-0.5); annotation bounds y(-2,-0.1); } regime positive eligible(y>0) { eq root:y == 1; annotation start y(0.5); annotation bounds y(0.1,2); } } realize r on roots using nested; eq link:s == roots.y; eq pin:target == 2; annotation start target(-2); annotation start s(-1); } }",
            SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let mut solver = super::super::super::tests::profile();
        // Presolve would fix `target` from the pin and start Ipopt across the switch.
        solver.presolve = pse_backend_native::presolve::Policy::Off;
        solver.controls.iterations = 20;
        // Every refused trial is also an event: retain all of them with the iterations.
        solver.controls.history = 4096;
        solver.intent = pse_backend_native::solve::SolveIntent::FeasiblePoint;
        solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Ipopt,
        );
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                pse_kernels::DerivativeOrder::Second,
                compiler,
                solver,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let result = package
            .solve_case(prepared, compiler, &cancel)
            .await
            .unwrap();
        let crate::math::solves::Outcome::Native(report) = &result.outcome else {
            panic!("expected a native outer solve: {:?}", result.outcome);
        };
        // Reaching `target == 2` crosses from the bound negative regime: refused, never
        // continued across the switch.
        assert!(!result.accepted);
        let callback = report.evidence.callback;
        assert!(callback.regime_crossings > 0, "{:?}", report.metrics);
        assert!(callback.trial_rejections >= callback.regime_crossings);
        use pse_backend_native::solve::Metric;
        assert_eq!(
            report.metrics["callback.regime_crossings"],
            Metric::Integer(i64::try_from(callback.regime_crossings).unwrap())
        );
        // Every outer iteration's progress event carries its own crossings.
        assert_eq!(report.dropped_events, 0);
        let by_iteration = report
            .events
            .iter()
            .filter(|e| e.phase == "ipopt.iteration")
            .map(
                |e| match (&e.values["iteration"], &e.values["regime.crossings"]) {
                    (Metric::Integer(iteration), Metric::Integer(crossings)) => {
                        (*iteration, *crossings)
                    }
                    other => panic!("{other:?}"),
                },
            )
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            by_iteration.keys().copied().collect::<Vec<_>>(),
            (0..=20).collect::<Vec<_>>()
        );
        // The start binds the regime in iteration 0; later iterations' steps cross.
        assert_eq!(by_iteration[&0], 0);
        assert!(by_iteration[&1] > 0, "{by_iteration:?}");
        assert!(
            by_iteration.values().sum::<i64>() <= i64::try_from(callback.regime_crossings).unwrap()
        );
    }
    #[tokio::test]
    async fn kernel_nested_stage_composes_child_residuals_and_second_derivatives() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows=pse_authoring::language::parse(
            "package p { def Root { var x: Scalar; implicit outer { var y: Scalar; implicit child select branch(z>=0) { var z: Scalar; eq residual: z*z == x; annotation start z(sqrt(x)); annotation bounds z(sqrt(x)-0.01,sqrt(x)+0.01); annotation nominal z(sqrt(x)); annotation scale residual(inverseSum); } realize c on child using nested; eq residual: y+child.z == x; annotation start y(1); annotation bounds y(0.1,100); } realize p on outer using nested; eq pin: outer.y == 2; annotation start x(3); annotation report outer.child.z(\"child\"); } }",
            SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let mut solver = super::super::super::tests::profile();
        solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Kinsol,
        );
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                pse_kernels::DerivativeOrder::Second,
                compiler,
                solver,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = prepared.model.model.compiled();
        let ordered = product.admitted.implicit_order().unwrap();
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[1].residuals[0].body.math.providers().len(), 1);
        assert_eq!(
            ordered[1].residuals[0].body.math.providers()[0].id,
            ordered[0].descriptor.spec().id
        );
        assert_eq!(
            product.model.instances[&InstanceId::from(ordered[0].descriptor.spec().id)].parent,
            Some(InstanceId::from(ordered[1].descriptor.spec().id))
        );
        assert_eq!(prepared.model.case.compiled().facts.variables, 1);
        let x = product.admitted.inputs[0];
        let mut values = prepared.model.values.clone();
        values.scalars.insert(x, 4.);
        let assembly = rt
            .shared
            .math()
            .assemble(prepared.model.case.clone())
            .await
            .unwrap();
        let (jacobian, hessian) = rt
            .shared
            .math()
            .with_worker(
                assembly,
                prepared.providers.clone(),
                &cancel,
                move |worker| {
                    let jacobian = worker.jacobian(&values)?.val().to_vec();
                    let hessian = worker.hessian(&values, 0., &[1.])?.val().to_vec();
                    Ok((jacobian, hessian))
                },
            )
            .await
            .unwrap();
        assert_eq!(jacobian.len(), 1);
        assert_eq!(hessian.len(), 1);
        assert!((jacobian[0] - 0.75).abs() < 1e-6, "{jacobian:?}");
        assert!((hessian[0] - 0.03125).abs() < 1e-6, "{hessian:?}");
        let result = package
            .solve_case(prepared, compiler, &cancel)
            .await
            .unwrap();
        assert!(result.accepted, "{:?}", result.validation_error);
        assert!((result.values.scalars[&x] - 4.).abs() < 1e-6);
        assert!(
            (result
                .reports
                .iter()
                .find(|r| r.label == "child")
                .unwrap()
                .value
                - 2.)
                .abs()
                < 1e-6
        );
    }
    #[tokio::test]
    async fn kernel_nested_stage_runs_on_outer_worker_and_uses_authored_hints() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows=pse_authoring::language::parse(
            "package p { def Root { var x: Scalar; implicit root select branch(y>=0) { var y: Scalar; eq residual: y*y == x; annotation scale residual(inverseSum); annotation start y(1); annotation bounds y(0.5,3); annotation valid y(0.5,3); annotation nominal y(2); } realize policy on root using nested; eq pin: root.y == 2; annotation start x(2); annotation report root.y(\"root\"); } }",
            SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).unwrap();
        let mut solver = super::super::super::tests::profile();
        solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Kinsol,
        );
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                pse_kernels::DerivativeOrder::Second,
                compiler,
                solver.clone(),
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(prepared.model.case.compiled().facts.variables, 1);
        let x = prepared.model.model.compiled().admitted.inputs[0];
        let original_identity = prepared.solve.request_identity().unwrap();
        let mut case = ModelingCaseBindings::default();
        case.values.insert("root.y".into(), 1.5);
        let changed = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                case,
                pse_kernels::DerivativeOrder::Second,
                compiler,
                solver.clone(),
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_ne!(original_identity, changed.solve.request_identity().unwrap());
        assert_ne!(
            prepared.solve.compatibility().unwrap().data,
            changed.solve.compatibility().unwrap().data
        );
        let result = package
            .solve_case(prepared, compiler, &cancel)
            .await
            .unwrap();
        assert!(result.accepted, "{result:?}");
        assert!((result.values.scalars[&x] - 4.).abs() < 1e-6);
        assert!(
            (result
                .reports
                .iter()
                .find(|r| r.label == "root")
                .unwrap()
                .value
                - 2.)
                .abs()
                < 1e-7
        );
        // A case start cannot fix an inner coordinate while leaving its equation hidden.
        let case = ModelingCaseBindings {
            values: BTreeMap::new(),
            variables: BTreeMap::from([(
                "root.y".into(),
                ModelingVariableState {
                    fixed: Some(true),
                    ..Default::default()
                },
            )]),
        };
        assert!(
            package
                .prepare_solve(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    case,
                    pse_kernels::DerivativeOrder::Second,
                    compiler,
                    solver,
                    NumericalInputs::default(),
                    &cancel
                )
                .await
                .unwrap_err()
                .to_string()
                .contains("inline realization")
        );
    }
}
