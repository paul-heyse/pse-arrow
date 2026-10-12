// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Transient typed value intent over one immutable declared parent.
use super::super::{AdmittedBinding, BindingTarget, PointOverlay};
use super::*;
use crate::math::solves::{NumericalInputs, SolverProfile};
use std::mem::size_of;

impl ModelingPackage {
    /// Solve a declared case with transient physically typed value replacements.
    ///
    /// Parameters and fixed coordinates replace their values; free coordinates replace
    /// their starts. The parent revision, roles, bounds and authored expressions remain
    /// authoritative. Each point resolves numerical facts and original assessment anew.
    #[expect(
        clippy::too_many_arguments,
        reason = "the declared case, typed intent, existing preparation/solver policy and cancellation are independent inputs"
    )]
    pub async fn solve_bound(
        &self,
        case: DeclarationId,
        overlay: &PointOverlay,
        compiler: pse_compiler::workspace::Profile,
        solver: SolverProfile,
        numerical: NumericalInputs,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        let prepared = self
            .prepare_bound_solve(case, overlay, compiler, solver, numerical, limits, cancel)
            .await?;
        self.solve_case(prepared, compiler, cancel).await
    }

    /// Retain a stored, already admitted binding under its own allocation owner.
    pub(in crate::workflow) fn retain_bound_source(
        &self,
        binding: &AdmittedBinding,
    ) -> Result<Arc<pse_columnar::Leased<AdmittedBinding>>, WorkflowError> {
        let extent = binding
            .entries
            .values()
            .try_fold(
                size_of::<AdmittedBinding>()
                    + size_of::<pse_columnar::Leased<AdmittedBinding>>()
                    + 8 * size_of::<usize>(),
                |total, entry| {
                    let path = match &entry.supplied {
                        BindingTarget::Path(path) => path.len(),
                        BindingTarget::Member(_) => 0,
                    };
                    total
                        .checked_add(
                            size_of::<(SemanticId, super::super::AdmittedBindingEntry)>() + 128,
                        )
                        .and_then(|bytes| bytes.checked_add(path))
                },
            )
            .ok_or_else(|| contract("retained bound source extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:retained-bound-source", extent)?;
        Ok(Arc::new(pse_columnar::Leased::new(
            Arc::new(binding.clone()),
            owner,
        )))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "typed binding uses the same declaration and existing settings as a normal solve"
    )]
    pub(in crate::workflow) async fn prepare_bound_solve(
        &self,
        case: DeclarationId,
        overlay: &PointOverlay,
        compiler: pse_compiler::workspace::Profile,
        solver: SolverProfile,
        numerical: NumericalInputs,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        cancel
            .token()
            .checkpoint()
            .map_err(pse_engine::EngineError::from)?;
        // Reserve the attribution, ordered input projection, admitted nodes and shared
        // wrappers before constructing them. The prepared specialization has its own owner.
        let extent = overlay
            .assignments
            .iter()
            .try_fold(
                size_of::<AdmittedBinding>()
                    + size_of::<pse_columnar::Leased<AdmittedBinding>>()
                    + 8 * size_of::<usize>(),
                |total, assignment| {
                    let path = match &assignment.target {
                        BindingTarget::Path(path) => path.len(),
                        BindingTarget::Member(_) => 0,
                    };
                    total
                        .checked_add(
                            size_of::<(SemanticId, super::super::AdmittedBindingEntry)>()
                                + size_of::<String>()
                                + 256,
                        )
                        .and_then(|bytes| {
                            path.checked_mul(3).and_then(|path| bytes.checked_add(path))
                        })
                },
            )
            .ok_or_else(|| contract("bound solve attribution extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:bound-source", extent)?;
        let demand = overlay
            .assignments
            .iter()
            .filter_map(|assignment| match &assignment.target {
                BindingTarget::Path(path) => Some(path.clone()),
                BindingTarget::Member(_) => None,
            })
            .collect();
        let mut execution = self
            .declared_execution_with_demand(
                case, compiler, solver, numerical, limits, demand, cancel,
            )
            .await?;
        if !matches!(execution.procedure, DeclaredProcedure::Solve) {
            return Err(contract(
                "bound solve requires the authored solve procedure",
            ));
        }
        let binding = self.admit_overlay(execution.model.compiled(), overlay)?;
        self.validate_binding(execution.model.compiled(), &binding)?;
        execution.analysis.case.members = binding.values();
        let analysis = execution.analysis;
        let resolution = self
            .resolve_prepared_case(
                execution.model,
                analysis.case,
                analysis.order,
                analysis.compiler,
                analysis.solver,
                analysis.numerical,
                cases::CaseOverrides::default(),
                false,
                cancel,
            )
            .await?;
        let mut prepared = self.finish_case(resolution, cancel).await?;
        prepared.bound_source = Some(Arc::new(pse_columnar::Leased::new(
            Arc::new(binding),
            owner,
        )));
        Ok(prepared)
    }
}

#[cfg(all(test, feature = "solver-kinsol"))]
mod tests {
    use super::*;
    use crate::workflow::tests as fixture;
    use crate::workflow::{BindingAssignment, BindingQuantity, StartSource};
    use pse_backend_native::solve::{Backend, SolverSelection};
    use pse_model::scalars::FiniteBound;

    fn rows(parameter: f64, fixed: f64, start: f64, literal: f64) -> Vec<Declaration> {
        pse_authoring::language::parse(
            &format!("package p {{ test Root fixture {{dof 0; fix f={fixed};}} {{param p:Scalar={parameter}; var f:Scalar; var x:Scalar; eq e:x==p*f+{literal}; annotation start x({start}); annotation bounds x(0,p*10); annotation report x(\"solution\"); annotation check x(x==p*f+{literal});}} }}"),
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        ).unwrap()
    }
    fn profile() -> SolverProfile {
        let mut profile = fixture::profile();
        profile.selection = SolverSelection::Explicit(Backend::Kinsol);
        profile
    }
    async fn package(
        parameter: f64,
        fixed: f64,
        start: f64,
        literal: f64,
    ) -> (ModelingPackage, DeclarationId) {
        let rows = rows(parameter, fixed, start, literal);
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        (
            fixture::runtime()
                .modeling_package(rows, fixture::physical())
                .await
                .unwrap(),
            root,
        )
    }
    fn assignment(
        package: &ModelingPackage,
        target: BindingTarget,
        value: f64,
    ) -> BindingAssignment {
        let quantity = package
            .quantities
            .quantity_types()
            .find(|q| q.name.as_deref() == Some("Scalar"))
            .unwrap();
        BindingAssignment {
            target,
            value: BindingQuantity {
                magnitude: FiniteBound::try_new(value).unwrap(),
                quantity: quantity.id.as_id(),
                unit: quantity.canonical_unit.as_id(),
            },
        }
    }
    async fn prepare(
        package: &ModelingPackage,
        root: DeclarationId,
        overlay: &PointOverlay,
    ) -> ModelingSolvePreparation {
        package
            .prepare_bound_solve(
                root,
                overlay,
                fixture::compiler_profile(),
                profile(),
                Default::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn bound_parameter_fixed_value_and_free_start_match_fresh_without_editing_parent() {
        let (parent, root) = package(3., 2., 1., 0.).await;
        let original_revision = parent.revision.identity();
        let original = prepare(&parent, root, &PointOverlay::default()).await;
        let member = |path: &str| original.model.model.compiled().model.paths[path];
        let (p, f, x) = (member("p"), member("f"), member("x"));
        let overlay = PointOverlay {
            assignments: vec![
                assignment(&parent, BindingTarget::Member(p), 4.),
                assignment(&parent, BindingTarget::Path("f".into()), 5.),
                assignment(&parent, BindingTarget::Path("x".into()), 7.),
            ],
        };
        let bound = parent
            .solve_bound(
                root,
                &overlay,
                fixture::compiler_profile(),
                profile(),
                Default::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let (fresh, fresh_root) = package(4., 5., 7., 0.).await;
        let fresh = fresh
            .solve_bound(
                fresh_root,
                &PointOverlay::default(),
                fixture::compiler_profile(),
                profile(),
                Default::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert!(bound.accepted && fresh.accepted);
        assert!((bound.values.scalars[&x] - fresh.values.scalars[&x]).abs() < 1e-8);
        assert!((bound.values.scalars[&x] - 20.).abs() < 1e-8);
        let receipt = bound.prepared.bound_source.as_ref().unwrap();
        assert_eq!(receipt.revision, original_revision);
        assert_eq!(receipt.context, parent.physical.identity());
        assert_eq!(receipt.entries.len(), 3);
        assert!(receipt.entries[&p].parameter);
        assert!(!receipt.entries[&f].parameter && !receipt.entries[&x].parameter);
        assert_eq!(
            receipt.entries[&x].supplied,
            BindingTarget::Path("x".into())
        );
        assert_eq!(
            bound.prepared.starts[&x],
            StartSource::Binding { member: x }
        );
        assert_eq!(parent.revision.identity(), original_revision);
        let reopened = prepare(&parent, root, &PointOverlay::default()).await;
        assert_eq!(reopened.model.values.scalars[&p], 3.);
        assert_eq!(reopened.model.values.scalars[&f], 2.);
        assert_eq!(reopened.model.values.scalars[&x], 1.);
        assert_eq!(
            bound
                .prepared
                .model
                .case
                .compiled()
                .plan
                .structure()
                .variables()
                .iter()
                .find(|v| v.port.id == x)
                .unwrap()
                .upper,
            Some(40.)
        );
        assert_eq!(
            reopened
                .model
                .case
                .compiled()
                .plan
                .structure()
                .variables()
                .iter()
                .find(|v| v.port.id == x)
                .unwrap()
                .upper,
            Some(30.)
        );
    }

    #[tokio::test]
    async fn bound_warm_point_reuses_structure_and_structural_literals_stay_authored() {
        let (parent, root) = package(3., 2., 1., 0.).await;
        let overlay = PointOverlay {
            assignments: vec![assignment(&parent, BindingTarget::Path("x".into()), 7.)],
        };
        let first = prepare(&parent, root, &overlay).await;
        let math = parent.runtime.shared.math();
        let before = math.preparation_effects();
        let (second, counts) = crate::math::counted(prepare(&parent, root, &overlay)).await;
        assert_eq!(counts.views, 0);
        let effects = math.preparation_effects().since(before);
        assert_eq!(effects.workspaces, 0);
        assert_eq!(effects.frontiers, 0);
        assert_eq!(effects.completions, 0);
        assert_eq!(effects.publications, 0);
        assert_eq!(effects.settlements, 1);
        assert!(Arc::ptr_eq(
            &first.model.case.compiled().plan,
            &second.model.case.compiled().plan
        ));
        assert_eq!(
            first.model.model.compiled().model.allocation_identity(),
            second.model.model.compiled().model.allocation_identity()
        );
        let (changed, changed_root) = package(3., 2., 1., 1.).await;
        let changed = changed
            .solve_bound(
                changed_root,
                &PointOverlay::default(),
                fixture::compiler_profile(),
                profile(),
                Default::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let x = changed.prepared.model.model.compiled().model.paths["x"];
        assert!((changed.values.scalars[&x] - 7.).abs() < 1e-8);
        let illegal = PointOverlay {
            assignments: vec![assignment(&parent, BindingTarget::Path("e".into()), 9.)],
        };
        assert!(
            parent
                .prepare_bound_solve(
                    root,
                    &illegal,
                    fixture::compiler_profile(),
                    profile(),
                    Default::default(),
                    Limits::default(),
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn declared_empty_nonempty_and_covered_fixture_demand_complete_and_settle_once() {
        for specifications in ["", "value unused=7;", "value p=4;"] {
            let text = format!(
                "package p {{test Root fixture {{dof 0; {specifications}}} {{param p:Scalar=3; param unused:Scalar=2; var x:Scalar; eq e:x==p; annotation start x(1);}}}}"
            );
            let rows = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                Default::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|row| row.name == "Root")
                .unwrap()
                .declaration_id;
            let package = fixture::runtime()
                .modeling_package(rows, fixture::physical())
                .await
                .unwrap();
            let math = package.runtime.shared.math();
            let before = math.preparation_effects();
            let execution = package
                .declared_execution(
                    root,
                    fixture::compiler_profile(),
                    profile(),
                    Default::default(),
                    Limits::default(),
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap();
            let effects = math.preparation_effects().since(before);
            assert_eq!(effects.frontiers, 1, "{specifications}: {effects:?}");
            assert_eq!(effects.completions, 1, "{specifications}: {effects:?}");
            assert_eq!(effects.settlements, 1, "{specifications}: {effects:?}");
            assert!(
                effects.workspaces >= 1,
                "cold admission constructs a workspace"
            );
            if specifications == "value unused=7;" {
                let target = execution.model.compiled().model.paths["unused"];
                assert!(execution.model.compiled().admitted.inputs.contains(&target));
            }
            let before = math.preparation_effects();
            let prepared = package
                .prepare_declared(&execution, &crate::CancelSource::new())
                .await
                .unwrap();
            let effects = math.preparation_effects().since(before);
            assert_eq!(effects.frontiers, 0, "retained execution: {specifications}");
            assert_eq!(
                effects.completions, 0,
                "retained execution: {specifications}"
            );
            assert_eq!(
                effects.settlements, 0,
                "retained execution: {specifications}"
            );
            assert_eq!(
                effects.publications, 0,
                "retained execution: {specifications}"
            );
            // Cold numerical observation/view compilation has its own workspace;
            // it does not plan or establish a second declared executable model.
            assert_eq!(
                prepared.model.model.compiled().model.allocation_identity(),
                execution.model.compiled().model.allocation_identity()
            );
        }
    }
}
