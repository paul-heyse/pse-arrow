// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared structures shared by every analysis of one package (A6): solver views and
//! value-independent observation programs, each keyed on its complete structural
//! identity (DP-09). A value-only change finds its structure here and rebinds values; only
//! a structural change prepares again.
use super::*;
use crate::math::{ExecutableCase, modeling::ModelingCasePreparation};
use pse_compiler::workspace::{ModelingVariableState, Profile};
use pse_kernels::DerivativeOrder;
use pse_math::binding::CaseValues;
use std::{collections::BTreeSet, sync::Arc};
impl ModelingPackage {
    /// The solver view of `model` under `states`, bound to `values`. The first request for a
    /// structure prepares it; every later one rebinds values onto it (A6). The returned
    /// values are completed with the view's derived realization parameters (ADR-0104). The
    /// bound tightenings belong to `states`, not to the shared view (ADR-0103 item 4).
    #[cfg(test)]
    pub(in crate::workflow) async fn bound_case(
        &self,
        model: &ModelingPreparation,
        values: CaseValues,
        states: &BTreeMap<SemanticId, ModelingVariableState>,
        order: DerivativeOrder,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingCasePreparation, WorkflowError> {
        self.bound_case_with_current_view(model, values, states, order, compiler, None, cancel)
            .await
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the borrowed current view has independent lane ownership from fresh case inputs"
    )]
    pub(in crate::workflow) async fn bound_case_with_current_view(
        &self,
        model: &ModelingPreparation,
        values: CaseValues,
        states: &BTreeMap<SemanticId, ModelingVariableState>,
        order: DerivativeOrder,
        compiler: Profile,
        current_view: Option<&mut crate::workflow::study_operations::CurrentNumericalView>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingCasePreparation, WorkflowError> {
        self.bound_case_with_lookup_reservation(
            model,
            values,
            states,
            order,
            compiler,
            current_view,
            cancel,
            |bytes| {
                self.runtime
                    .shared
                    .math()
                    .reserve("math:bound-lookup", bytes)
            },
        )
        .await
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the optional retention reservation is an independent effect from scientific case preparation"
    )]
    async fn bound_case_with_lookup_reservation(
        &self,
        model: &ModelingPreparation,
        values: CaseValues,
        states: &BTreeMap<SemanticId, ModelingVariableState>,
        order: DerivativeOrder,
        compiler: Profile,
        mut current_view: Option<&mut crate::workflow::study_operations::CurrentNumericalView>,
        cancel: &crate::CancelSource,
        reserve: impl FnOnce(
            usize,
        ) -> Result<
            Arc<pse_columnar::AllocationLease>,
            crate::math::MathRuntimeError,
        > + Send,
    ) -> Result<ModelingCasePreparation, WorkflowError> {
        let preparation_driver = self.runtime.shared.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let product = model.compiled();
                let lookup = product
                    .bound_lookup(
                        states,
                        order,
                        compiler,
                        &product.consumed_physical_identity(),
                    )
                    .map_err(crate::math::MathRuntimeError::from)?;
                let key = lookup.key.clone();
                let tightenings = lookup.tightenings.clone();
                let identity = product.occurrence_identity();
                let service = self.runtime.shared.math();
                let generation = service.modeling_cache.generation();
                // The explicit active lane owns one current view independently of
                // optional cross-lane lookup retention. State/domain admission above
                // still runs for every point, including a current-view hit.
                let current = current_view.as_deref_mut().and_then(|slot| {
                    if slot.as_ref().is_some_and(|view| view.key == key) {
                        slot.clone()
                    } else {
                        // Drop the incompatible lane owner before replacement work.
                        *slot = None;
                        None
                    }
                });
                let cached = current.or_else(|| service.modeling_cache.bound_solver(key.clone()));
                let (case, retained) = match cached {
                    Some(cached) => {
                        let case = service.rebind(&cached.view, values.clone(), cancel).await?;
                        if cached.identity == identity
                            && case.compiled().has_modeling_context(product)
                        {
                            (case, Some(cached))
                        } else {
                            // Equal occurrences under new physical owners still need
                            // the receiving shell; ordinary active views stay shared.
                            (service.attribute_modeling_view(&case, model)?, None)
                        }
                    }
                    None => {
                        let bound = product
                            .expand_bound_lookup(lookup)
                            .map_err(crate::math::MathRuntimeError::from)?;
                        (
                            service
                                .prepare_modeling_view(
                                    self.numerical_workspace()?,
                                    model.clone(),
                                    bound.structure,
                                    values.clone(),
                                    order,
                                    compiler,
                                    cancel,
                                )
                                .await?,
                            None,
                        )
                    }
                };
                let retained = if let Some(retained) = retained {
                    Some(retained)
                } else {
                    let bytes = key.retained_bytes()
                        + identity.capacity()
                            * size_of::<(DeclarationId, String, usize, pse_authoring::SourceSpan)>(
                            )
                        + identity
                            .iter()
                            .map(|(_, role, _, _)| role.capacity())
                            .sum::<usize>()
                        + 1024;
                    // A lane's metadata is part of its admitted live working set.
                    // Without a lane, lookup retention remains wholly optional.
                    let owner = match reserve(bytes) {
                        Ok(owner) => Some(owner),
                        Err(error) if current_view.is_some() => return Err(error.into()),
                        Err(_) => None,
                    };
                    owner.map(|owner| {
                        let retained = Arc::new(crate::math::retention::RetainedBoundSolver {
                            key: key.clone(),
                            view: case.clone(),
                            identity,
                            _owner: owner,
                        });
                        service.modeling_cache.retain_bound_solver(
                            generation,
                            key,
                            retained.clone(),
                        );
                        retained
                    })
                };
                if let Some(slot) = current_view {
                    *slot = retained;
                }
                Ok(ModelingCasePreparation {
                    model: model.clone(),
                    values: case.compiled().complete(&values),
                    case,
                    tightenings,
                })
            })
            .await
    }
    /// The parametric program of the solver view `prepared` over `parameters`, in request
    /// order (Plan 22 S1), compiled once per view and parameter selection and cached with
    /// the view (A6). Every declared parameter of the view's structure is admitted.
    pub(in crate::workflow) async fn parametric_program(
        &self,
        model: &ModelingPreparation,
        prepared: &ModelingCasePreparation,
        parameters: &[SemanticId],
        order: DerivativeOrder,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<ExecutableCase>, WorkflowError> {
        let preparation_driver = self.runtime.shared.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = self.runtime.admit_activity()?;
        activity.scope(async {
        let structure = prepared.case.compiled().plan.structure();
        let declared: BTreeSet<_> = structure.parameters().iter().map(|p| p.id).collect();
        if let Some(unknown) = parameters.iter().find(|p| !declared.contains(p)) {
            return Err(contract(format!(
                "sensitivity parameter {unknown} is not a declared parameter of the solved case"
            )));
        }
        let generation = self.runtime.shared.math().modeling_cache.generation();
        let product = model.compiled();
        let view = product.view_key(structure, order, compiler, &product.consumed_physical_identity());
        let key = pse_compiler::workspace::PreparedModeling::parametric_key(&view, parameters);
        if let Some(program) = self.runtime.shared.math().modeling_cache.program(key, true) {
            return Ok(program);
        }
        let program = self
            .runtime
            .shared
            .math()
            .prepare_modeling_parametric(
                self.numerical_workspace()?,
                prepared.case.clone(),
                parameters.to_vec(),
                order,
                compiler,
                cancel,
            )
            .await?;
        self.runtime.shared.math().modeling_cache.retain_program(
            generation,
            key,
            program.clone(),
            true,
        );
        Ok(program)

        }).await
    }
    /// The value-independent program observing `rows` of `model`, compiled once per
    /// structure.
    pub(in crate::workflow) async fn observation_program(
        &self,
        model: &ModelingPreparation,
        rows: &BTreeSet<SemanticId>,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<ExecutableCase>, WorkflowError> {
        let preparation_driver = self.runtime.shared.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let generation = self.runtime.shared.math().modeling_cache.generation();
                let product = model.compiled();
                let structure = product
                    .observation_structure(rows)
                    .map_err(crate::math::MathRuntimeError::from)?;
                let key = product.view_key(
                    &structure,
                    DerivativeOrder::Value,
                    compiler,
                    &product.consumed_physical_identity(),
                );
                if let Some(program) = self
                    .runtime
                    .shared
                    .math()
                    .modeling_cache
                    .program(key, false)
                {
                    return Ok(program);
                }
                let program = self
                    .runtime
                    .shared
                    .math()
                    .prepare_modeling_observations(
                        self.numerical_workspace()?,
                        model.clone(),
                        structure,
                        compiler,
                        cancel,
                    )
                    .await?;
                self.runtime.shared.math().modeling_cache.retain_program(
                    generation,
                    key,
                    program.clone(),
                    false,
                );
                Ok(program)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::{self, ObservationHub, Policy, Value};

    async fn current_view_fixture(
        runtime: &Runtime,
        source: &str,
    ) -> (ModelingPackage, ModelingPreparation, SemanticId) {
        let declarations = pse_authoring::language::parse(
            source,
            SemanticId::from_bytes([145; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        (package, model, x)
    }

    #[tokio::test]
    async fn lane_current_view_survives_trim_rebinds_values_and_replaces_complete_key() {
        let runtime = crate::workflow::tests::runtime();
        let (package, model, x) =
            current_view_fixture(&runtime, "package p {def Root {var x:Scalar; eq e:x==2;}} ")
                .await;
        let cancel = crate::CancelSource::new();
        let compiler = crate::workflow::tests::compiler_profile();
        let service = runtime.shared.math();
        let mut current = None;
        let before = service.preparations().views;
        let mut first = None;
        for value in [3., 5., 7., 9.] {
            service.clear_program_cache();
            let prepared = package
                .bound_case_with_current_view(
                    &model,
                    CaseValues {
                        scalars: BTreeMap::from([(x, value)]),
                    },
                    &BTreeMap::new(),
                    DerivativeOrder::First,
                    compiler,
                    Some(&mut current),
                    &cancel,
                )
                .await
                .unwrap();
            assert_eq!(prepared.values.scalars[&x], value);
            if let Some(plan) = &first {
                assert!(Arc::ptr_eq(plan, &prepared.case.compiled().plan));
            } else {
                first = Some(prepared.case.compiled().plan.clone());
            }
        }
        assert_eq!(
            service.preparations().views - before,
            1,
            "four compatible values prepare one view despite four explicit lookup trims"
        );
        let invalid = BTreeMap::from([(
            x,
            ModelingVariableState {
                lower: Some(Some(9.)),
                upper: Some(Some(1.)),
                ..Default::default()
            },
        )]);
        assert!(
            package
                .bound_case_with_current_view(
                    &model,
                    CaseValues {
                        scalars: BTreeMap::from([(x, 4.)])
                    },
                    &invalid,
                    DerivativeOrder::First,
                    compiler,
                    Some(&mut current),
                    &cancel
                )
                .await
                .is_err()
        );
        let mut changed_compiler = compiler;
        changed_compiler.class_proof_work += 1;
        let changed_states = BTreeMap::from([(
            x,
            ModelingVariableState {
                lower: Some(Some(-10.)),
                ..Default::default()
            },
        )]);
        let mut changed_context = package.clone();
        // Deliberate key-level control: identical typed inventory under another
        // selected physical identity must never match the old numerical view.
        changed_context.physical.key = pse_ids::ContentHash::from_bytes([99; 32]);
        let (changed_structure, changed_model, changed_x) =
            current_view_fixture(&runtime, "package p {def Root {var x:Scalar; eq e:x==3;}} ")
                .await;
        for (source, product, member, states, order, profile) in [
            (
                &package,
                &model,
                x,
                BTreeMap::new(),
                DerivativeOrder::Second,
                compiler,
            ),
            (
                &package,
                &model,
                x,
                BTreeMap::new(),
                DerivativeOrder::Second,
                changed_compiler,
            ),
            (
                &package,
                &model,
                x,
                changed_states,
                DerivativeOrder::Second,
                changed_compiler,
            ),
            (
                &changed_context,
                &model,
                x,
                BTreeMap::new(),
                DerivativeOrder::Second,
                changed_compiler,
            ),
            (
                &changed_structure,
                &changed_model,
                changed_x,
                BTreeMap::new(),
                DerivativeOrder::Second,
                changed_compiler,
            ),
        ] {
            service.clear_program_cache();
            let previous = Arc::downgrade(current.as_ref().unwrap());
            let previous_key = current.as_ref().unwrap().key.clone();
            source
                .bound_case_with_current_view(
                    product,
                    CaseValues {
                        scalars: BTreeMap::from([(member, 4.)]),
                    },
                    &states,
                    order,
                    profile,
                    Some(&mut current),
                    &cancel,
                )
                .await
                .unwrap();
            assert_ne!(current.as_ref().unwrap().key, previous_key);
            assert!(
                previous.upgrade().is_none(),
                "replacement retains no historical slot"
            );
        }
        service.clear_program_cache();
        let departure = Arc::downgrade(current.as_ref().unwrap());
        drop(current);
        assert!(departure.upgrade().is_none());
    }

    #[tokio::test]
    async fn concurrent_lane_views_share_one_lookup_metadata_owner_with_fresh_values() {
        let runtime = crate::workflow::tests::runtime();
        let (package, model, x) =
            current_view_fixture(&runtime, "package p {def Root {var x:Scalar; eq e:x==2;}} ")
                .await;
        let cancel = crate::CancelSource::new();
        let compiler = crate::workflow::tests::compiler_profile();
        let states = BTreeMap::new();
        package
            .bound_case(
                &model,
                CaseValues {
                    scalars: BTreeMap::from([(x, 1.)]),
                },
                &states,
                DerivativeOrder::First,
                compiler,
                &cancel,
            )
            .await
            .unwrap();
        let (mut left, mut right) = (None, None);
        let (a, b) = tokio::join!(
            package.bound_case_with_current_view(
                &model,
                CaseValues {
                    scalars: BTreeMap::from([(x, 3.)])
                },
                &states,
                DerivativeOrder::First,
                compiler,
                Some(&mut left),
                &cancel
            ),
            package.bound_case_with_current_view(
                &model,
                CaseValues {
                    scalars: BTreeMap::from([(x, 5.)])
                },
                &states,
                DerivativeOrder::First,
                compiler,
                Some(&mut right),
                &cancel
            ),
        );
        let (a, b) = (a.unwrap(), b.unwrap());
        assert_eq!((a.values.scalars[&x], b.values.scalars[&x]), (3., 5.));
        assert!(Arc::ptr_eq(left.as_ref().unwrap(), right.as_ref().unwrap()));
        assert!(Arc::ptr_eq(
            &a.case.compiled().plan,
            &b.case.compiled().plan
        ));
        runtime.shared.math().clear_program_cache();
        let shared = Arc::downgrade(left.as_ref().unwrap());
        drop(left);
        assert!(shared.upgrade().is_some());
        drop(right);
        assert!(shared.upgrade().is_none());
    }

    #[tokio::test]
    async fn lane_current_view_reattributes_changed_source_occurrences() {
        let runtime = crate::workflow::tests::runtime();
        let (package, model, x) =
            current_view_fixture(&runtime, "package p {def Root {var x:Scalar; eq e:x==2;}} ")
                .await;
        let (revised, next_model, next_x) = current_view_fixture(
            &runtime,
            "package p {def Root {var x:Scalar;       eq e:x==2;}} ",
        )
        .await;
        let cancel = crate::CancelSource::new();
        let compiler = crate::workflow::tests::compiler_profile();
        let mut current = None;
        let first = package
            .bound_case_with_current_view(
                &model,
                CaseValues {
                    scalars: BTreeMap::from([(x, 3.)]),
                },
                &BTreeMap::new(),
                DerivativeOrder::First,
                compiler,
                Some(&mut current),
                &cancel,
            )
            .await
            .unwrap();
        let key = current.as_ref().unwrap().key.clone();
        let identity = current.as_ref().unwrap().identity.clone();
        runtime.shared.math().clear_program_cache();
        let next = revised
            .bound_case_with_current_view(
                &next_model,
                CaseValues {
                    scalars: BTreeMap::from([(next_x, 5.)]),
                },
                &BTreeMap::new(),
                DerivativeOrder::First,
                compiler,
                Some(&mut current),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(current.as_ref().unwrap().key, key);
        assert_ne!(current.as_ref().unwrap().identity, identity);
        assert_eq!(
            current.as_ref().unwrap().identity,
            next_model.compiled().occurrence_identity()
        );
        assert!(!Arc::ptr_eq(
            &first.case.compiled().plan,
            &next.case.compiled().plan
        ), "the receiving source has its own validated plan shell");
        assert_eq!(
            first.case.compiled().plan.allocation_identity()[1..],
            next.case.compiled().plan.allocation_identity()[1..],
            "numerical layouts remain shared across source reattribution"
        );
        for (key, body) in first.case.compiled().plan.bodies() {
            assert!(Arc::ptr_eq(body, &next.case.compiled().plan.bodies()[key]));
        }
        for binding in next.case.compiled().plan.structure().instances() {
            let actual = next_model.compiled().admitted.case().instances().iter()
                .find(|actual| actual.instance == binding.instance).unwrap();
            assert_eq!(binding.checked_members, actual.checked_members);
            assert_eq!(binding.checked_sources, actual.checked_sources);
            assert_eq!(binding.occurrences, actual.occurrences);
            assert!(!binding.occurrences.is_empty());
            assert!(binding.occurrences.values().all(|location|
                location.document == Some(SemanticId::from_bytes([145; 16]))));
        }
        assert_eq!(
            next.case.compiled().occurrences.allocation_identity(),
            Arc::as_ptr(&next_model.compiled().shared_occurrences()) as usize
        );
        assert_eq!(next.values.scalars[&next_x], 5.);
    }

    #[tokio::test]
    async fn denied_bound_lookup_retention_returns_current_case_without_cache_entry() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let runtime = crate::workflow::tests::runtime();
        let declarations = pse_authoring::language::parse(
            "package p {def Root {var x:Scalar; eq e:x==2;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
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
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        let compiler = crate::workflow::tests::compiler_profile();
        let states = BTreeMap::new();
        let key = model
            .compiled()
            .bound_lookup(
                &states,
                DerivativeOrder::First,
                compiler,
                &package.physical.key,
            )
            .unwrap()
            .key;
        let service = runtime.shared.math();
        assert!(service.modeling_cache.bound_solver(key.clone()).is_none());
        let denials = AtomicUsize::new(0);
        for value in [3., 5.] {
            let current = package
                .bound_case_with_lookup_reservation(
                    &model,
                    CaseValues {
                        scalars: BTreeMap::from([(x, value)]),
                    },
                    &states,
                    DerivativeOrder::First,
                    compiler,
                    None,
                    &cancel,
                    |_| {
                        denials.fetch_add(1, Ordering::Relaxed);
                        Err(crate::math::MathRuntimeError::Limit(
                            "test:lookup-retention-denied",
                        ))
                    },
                )
                .await
                .unwrap();
            assert_eq!(current.values.scalars[&x], value);
            assert_eq!(current.case.compiled().plan.structure().rows().len(), 1);
            assert!(service.modeling_cache.bound_solver(key.clone()).is_none());
        }
        assert_eq!(
            denials.load(Ordering::Relaxed),
            2,
            "each completed case reached the denied optional reservation"
        );
        let mut lane = None;
        let error = package
            .bound_case_with_lookup_reservation(
                &model,
                CaseValues {
                    scalars: BTreeMap::from([(x, 7.)]),
                },
                &states,
                DerivativeOrder::First,
                compiler,
                Some(&mut lane),
                &cancel,
                |_| {
                    Err(crate::math::MathRuntimeError::Limit(
                        "test:lane-metadata-denied",
                    ))
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            WorkflowError::Math(crate::math::MathRuntimeError::Limit(
                "test:lane-metadata-denied"
            ))
        ));
        assert!(
            lane.is_none(),
            "active ownership is not published without its metadata charge"
        );
    }

    #[tokio::test]
    async fn warm_bound_lookup_expands_once_rebinds_values_and_rejects_invalid_bounds() {
        let runtime = crate::workflow::tests::runtime();
        let source = "package p {def Root {var x:Scalar; eq e:x==2;}}";
        let declarations = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
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
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        let compiler = crate::workflow::tests::compiler_profile();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("bound.lookup.hit");
        let activity = runtime.admit_activity().unwrap();
        activity
            .scope(observation::scope(Some(operation.context()), async {
                let first = package
                    .bound_case(
                        &model,
                        CaseValues {
                            scalars: BTreeMap::from([(x, 3.)]),
                        },
                        &BTreeMap::new(),
                        DerivativeOrder::First,
                        compiler,
                        &cancel,
                    )
                    .await
                    .unwrap();
                let second = package
                    .bound_case(
                        &model,
                        CaseValues {
                            scalars: BTreeMap::from([(x, 5.)]),
                        },
                        &BTreeMap::new(),
                        DerivativeOrder::First,
                        compiler,
                        &cancel,
                    )
                    .await
                    .unwrap();
                assert!(Arc::ptr_eq(
                    &first.case.compiled().plan,
                    &second.case.compiled().plan
                ));
                assert_eq!(first.values.scalars[&x], 3.);
                assert_eq!(second.values.scalars[&x], 5.);
                assert_eq!(
                    first.case.compiled().occurrences.allocation_identity(),
                    second.case.compiled().occurrences.allocation_identity()
                );
                let invalid = BTreeMap::from([(
                    x,
                    ModelingVariableState {
                        lower: Some(Some(9.)),
                        upper: Some(Some(1.)),
                        ..Default::default()
                    },
                )]);
                assert!(
                    package
                        .bound_case(
                            &model,
                            second.values.clone(),
                            &invalid,
                            DerivativeOrder::First,
                            compiler,
                            &cancel
                        )
                        .await
                        .is_err()
                );
            }))
            .await;
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "compiler.bound_structures")
                .unwrap()
                .cumulative,
            Value::Count(2),
            "one valid cold expansion and one invalid-bound validation, with no warm-hit expansion"
        );
    }

    #[tokio::test]
    async fn selected_observation_cache_carries_one_structure_and_rebinds_actual_values() {
        let runtime = crate::workflow::tests::runtime();
        let declarations = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; eq e:x==2; } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
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
        let row = model
            .compiled()
            .model
            .equations
            .iter()
            .find(|equation| equation.lineage.path == "Root.e")
            .unwrap()
            .id;
        let rows = BTreeSet::from([row]);
        let compiler = crate::workflow::tests::compiler_profile();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("selected.observation.cache");
        let activity = runtime.admit_activity().unwrap();
        activity
            .scope(observation::scope(Some(operation.context()), async {
                let first = package
                    .observation_program(&model, &rows, compiler, &cancel)
                    .await
                    .unwrap();
                let second = package
                    .observation_program(&model, &rows, compiler, &cancel)
                    .await
                    .unwrap();
                assert!(Arc::ptr_eq(&first, &second));
            }))
            .await;
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let structures = snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == "compiler.observation_structures")
            .unwrap();
        assert_eq!(
            structures.cumulative,
            Value::Count(2),
            "each key lookup constructs one structure; a miss must carry that same structure into preparation"
        );
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path == "Root.x")
            .unwrap()
            .id;
        for (value, expected) in [(3., 1.), (5., 3.)] {
            let observed = package
                .observe_registered(
                    model.clone(),
                    rows.clone(),
                    CaseValues {
                        scalars: BTreeMap::from([(x, value)]),
                    },
                    compiler,
                    BTreeMap::new(),
                    &cancel,
                )
                .await
                .unwrap();
            assert_eq!(observed[&row], expected);
        }
    }
}
