// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original compiled PETSc products use this package's existing compiler workspace.
use super::*;
use crate::math::solves::{
    SolverProfile,
    petsc::{PreparedPetsc, PreparedPetscSource},
};
use pse_ids::ContentHash;
use pse_kernels::ExecutionScope;

impl ModelingPackage {
    /// Freeze the actual complete authored root source and its compiled domain inventory.
    /// No auxiliary native status grants original scientific permission.
    ///
    /// # Errors
    /// Source revision/physical context or original selected values differ, or scoped
    /// mathematical preparation refuses support, cancellation or a resource cap.
    pub async fn prepare_petsc_source(
        &self,
        original: &ModelingSolvePreparation,
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PreparedPetscSource, WorkflowError> {
        if self.revision.identity() != original.source.revision.identity()
            || self.physical.key != original.source.physical.key
        {
            return Err(contract(
                "PETSc source must belong to this actual package revision and physical context",
            ));
        }
        let guards = original
            .solve
            .petsc_guard_inventory()
            .map_err(crate::math::MathRuntimeError::from)?;
        let source = self
            .runtime
            .native()
            .prepare_petsc_source(original.solve.clone(), guards, scope, driver)
            .await?;
        if !source.matches_case(&original.model.case, &original.model.values) {
            return Err(contract(
                "PETSc source differs from the original complete compiled case or frozen values",
            ));
        }
        Ok(source)
    }
    /// Prepare complete BTF First block products through the actual source workspace.
    /// The explicit declaration records the domain-safe block admission; every trial
    /// still executes the original full source domain supplier.
    ///
    /// # Errors
    /// Original/profile/domain binding, scoped preparation or complete block coverage fails.
    pub async fn prepare_petsc_blocks(
        &self,
        original: &ModelingSolvePreparation,
        profile: SolverProfile,
        declaration: ContentHash,
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PreparedPetsc, WorkflowError> {
        let source = self.prepare_petsc_source(original, scope, driver).await?;
        Ok(self
            .runtime
            .native()
            .prepare_petsc_blocks(
                source,
                crate::math::solves::petsc::PetscCompiler {
                    workspace: self.numerical_workspace()?,
                    profile: original.compiler,
                },
                profile,
                None,
                declaration,
                driver,
            )
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::solves::{NumericalInputs, PreparedRung};
    use pse_backend_native::{
        execution::BackendSettings,
        petsc,
        solve::{Backend, SolveIntent, SolverSelection},
    };
    use pse_kernels::DerivativeOrder;
    use pse_model::strategy::{
        MechanismKind, NumericalStrategy, Position, StartOrigin, Transition, WorkLimits,
    };
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    async fn original(text: &str) -> (ModelingPackage, ModelingSolvePreparation) {
        use crate::workflow::tests as fixture;
        let runtime = fixture::runtime_with(256 << 20, 1 << 20, 1 << 30);
        let rows = pse_authoring::language::parse(
            text,
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
        let package = runtime
            .modeling_package(rows, fixture::physical())
            .await
            .unwrap();
        let mut profile = SolverProfile {
            intent: SolveIntent::Root,
            selection: SolverSelection::Explicit(Backend::Petsc),
            backend: BackendSettings::Petsc(petsc::Settings::default()),
            ..Default::default()
        };
        profile.numerics.native_scaling = false;
        profile.controls.foreign_bytes = Some(1 << 20);
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                Default::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                profile,
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        (package, prepared)
    }
    #[tokio::test]
    async fn actual_complete_btf_and_guard_only_slots_reach_original_correction() {
        let (package,mut original)=original("package p { def Root { var x:Scalar; var y:Scalar; annotation start x(0); annotation start y(1); eq first:x+0*log(y)==1; eq second:y==2; } }").await;
        let scope = ExecutionScope::new(
            Arc::default(),
            Some(Instant::now() + Duration::from_secs(30)),
        );
        let mut profile = original.profile.clone();
        profile.backend = BackendSettings::Petsc(petsc::Settings {
            method: petsc::Method::NonlinearAdditiveSchwarz,
            schwarz: Some(Default::default()),
            ..Default::default()
        });
        let blocks = package
            .prepare_petsc_blocks(
                &original,
                profile,
                ContentHash::from_bytes([82; 32]),
                scope.clone(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(
            blocks.original_identity(),
            original.solve.original_identity().unwrap()
        );
        assert_eq!(blocks.mechanism(), MechanismKind::Block);
        let limits = WorkLimits {
            attempts: 2,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        };
        let mut strategy = NumericalStrategy::direct(original.profile.controls.start, limits);
        let mut correction = strategy.mechanisms[0].clone();
        correction.profile = Some(pse_model::strategy::ProfileRef {
            backend: Backend::Petsc,
            key: original.solve.strategy_profile().unwrap(),
        });
        correction.starts = vec![StartOrigin::Auxiliary];
        strategy.start.recovery = vec![StartOrigin::Auxiliary];
        strategy.mechanisms[0].kind = MechanismKind::Block;
        strategy.mechanisms[0].position = Position::Execution;
        strategy.mechanisms[0].profile = Some(blocks.profile_ref().unwrap());
        strategy.mechanisms[0].support = blocks.support().into_iter().collect();
        strategy.mechanisms[0].transitions = vec![Transition::Continue, Transition::Stop];
        strategy.mechanisms.push(correction);
        original.solve = original
            .solve
            .clone()
            .within_task(scope)
            .unwrap()
            .with_strategy(
                strategy,
                vec![PreparedRung::Petsc(blocks), original.solve.clone().into()],
            )
            .unwrap();
        let result = package
            .solve_case(
                original,
                crate::workflow::tests::compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert!(
            result.completion.decision.permits_use(),
            "original assessment refused: {:?}",
            result.completion.decision
        );
        let trace = result.strategy.as_ref().unwrap();
        assert!(
            trace
                .events
                .iter()
                .any(|e| e.mechanism == 0 && e.transition == Some(Transition::Continue))
        );
        assert!(
            trace
                .events
                .iter()
                .any(|e| e.mechanism == 1 && e.transition == Some(Transition::Finish)),
            "original correction alone must grant completion"
        );
    }
}
