// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure observations select an execution-only capability before symbolic preparation.
use super::*;
use pse_compiler::workspace::Profile;
use pse_kernels::DerivativeOrder;
use pse_math::{binding::CaseValues, guarded::Evaluation};

impl ModelingPackage {
    /// Observe an authored pure selection without first constructing a source
    /// mathematical body. A protected numerical hit survives producer retirement.
    #[expect(
        clippy::too_many_arguments,
        reason = "the authored specialization, ordered numerical demand, actual values and current operation remain explicit"
    )]
    pub async fn observe_authored(
        &self,
        services: &crate::workflow::SourceServices,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        rows: Vec<SemanticId>,
        coordinates: Vec<SemanticId>,
        order: DerivativeOrder,
        values: CaseValues,
        profile: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<Evaluation>>, WorkflowError> {
        let preparation = services.preparation();
        self.check_service_bindings(preparation)?;
        let activity = preparation.admit_activity()?;
        activity
            .scope(async {
                let artifact = self
                    .numerical_observation_program(
                        preparation,
                        Some(services),
                        root,
                        instance,
                        bindings,
                        limits,
                        rows,
                        coordinates,
                        order,
                        profile,
                        cancel,
                    )
                    .await?
                    .ok_or_else(|| {
                        contract(
                            "selected observation requires a source-bearing/provider capability",
                        )
                    })?;
                Ok(preparation
                    .math()
                    .evaluate_numerical_observation(artifact, values, cancel)
                    .await?)
            })
            .await
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one original source selection issues a passive request and an independently retained optional miss recipe"
    )]
    pub(super) async fn numerical_observation_program(
        &self,
        services: &crate::workflow::PreparationServices,
        source_services: Option<&crate::workflow::SourceServices>,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        rows: Vec<SemanticId>,
        coordinates: Vec<SemanticId>,
        order: DerivativeOrder,
        profile: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Option<Arc<crate::math::NumericalArtifact>>, WorkflowError> {
        limits.validate().map_err(modeling_error)?;
        self.check_service_bindings(services)?;
        let driver = services.math().preparation_driver(cancel)?;
        let deadline = services.math().preparation_deadline(&driver)?;
        let selected = self
            .checked_selection_scoped(services, &[root], &driver, deadline)
            .await?;
        let math = services.math();
        let key = math.basis_key(&selected.admission, root, instance, bindings, limits)?;
        let workspace = self.pure_workspace(services)?;
        let frontier = math
            .plan_frontier_keyed(
                key,
                workspace.clone(),
                selected.admission.revision.clone(),
                &driver,
                deadline,
            )
            .await?;
        let (prepared, _request_owner) = math
            .numerical_observation_request_for(
                workspace.clone(),
                frontier,
                rows,
                coordinates,
                order,
                profile,
                &driver,
            )
            .await?;
        let control = pse_diagnostics::Cancellation::default();
        let Some((request, recipe)) = prepared else {
            return Ok(None);
        };

        let mut protected = portable_preparation::ProtectedPreparation::open(
            self,
            source_services,
            math,
            &selected.admission.versions,
            &driver,
            deadline,
        )
        .await?;
        let result = portable_preparation::run(&driver, &control, async {
            let received = if let Some((store, selection, namespace)) =
                protected.as_mut().and_then(|current| current.receiving())
            {
                crate::math::portable::reuse_execution_current(
                    math,
                    store,
                    selection,
                    namespace,
                    request.clone(),
                    &control.flag(),
                )
                .await
                .map_err(crate::math::portable::runtime_error)?
            } else {
                None
            };
            let publish = received.is_none();
            let artifact = math
                .numerical_artifact_for(workspace, request, recipe, received, &driver)
                .await?;
            if publish
                && let Some((store, selection, namespace)) =
                    protected.as_ref().and_then(|current| current.publication())
            {
                canonical::checkpoint(&driver)?;
                crate::math::portable::publish_execution_current(
                    math,
                    store,
                    selection,
                    namespace,
                    &artifact.execution,
                    deadline.into(),
                )
                .await
                .map_err(crate::math::portable::runtime_error)?;
            }
            Ok(Some(artifact))
        })
        .await;
        portable_preparation::finish(protected, result).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn authored_numerical_observation_executes_selected_second_and_reuses_runtime_artifact() {
        let runtime = crate::workflow::tests::runtime();
        let declarations = pse_authoring::language::parse(
            "package p {def Root {var x:Scalar;var y:Scalar;eq a:log1p(x)==0;eq b:magnitude(x,y)==0;}}",
            SemanticId::from_bytes([153;16]), pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        ).unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let instance = pse_modeling::specialize::root_instance(root);
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let source = runtime.source_services();
        let services = source.preparation();
        let driver = crate::CancelSource::new();
        let mut bindings = Bindings::default();
        bindings.demand = vec!["x".into(), "y".into()];
        let model = package
            .prepare(
                services,
                root,
                instance,
                bindings.clone(),
                Limits::default(),
                &driver,
            )
            .await
            .unwrap();
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path == "Root.x")
            .unwrap()
            .id;
        let y = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path == "Root.y")
            .unwrap()
            .id;
        let rows = ["Root.a", "Root.b"]
            .into_iter()
            .map(|path| {
                model
                    .compiled()
                    .model
                    .equations
                    .iter()
                    .find(|row| row.lineage.path == path)
                    .unwrap()
                    .id
            })
            .collect::<Vec<_>>();
        drop(model);
        services.math().clear_program_cache();
        let profile = crate::workflow::tests::compiler_profile();
        let program = package
            .numerical_observation_program(
                services,
                None,
                root,
                instance,
                bindings.clone(),
                Limits::default(),
                rows.clone(),
                vec![y, x],
                DerivativeOrder::Second,
                profile,
                &driver,
            )
            .await
            .unwrap()
            .unwrap();
        let again = package
            .numerical_observation_program(
                services,
                None,
                root,
                instance,
                bindings.clone(),
                Limits::default(),
                rows.clone(),
                vec![y, x],
                DerivativeOrder::Second,
                profile,
                &driver,
            )
            .await
            .unwrap()
            .unwrap();
        assert!(Arc::ptr_eq(&program, &again));
        let values = CaseValues {
            scalars: BTreeMap::from([(x, 3.), (y, 4.)]),
        };
        let result = services
            .math()
            .evaluate_numerical_observation(program, values, &driver)
            .await
            .unwrap();
        assert!((result.values[0] - 4_f64.ln()).abs() < 2e-15);
        assert_eq!(result.values[1], 5.);
        for (actual, expected) in result.jacobian.iter().zip([0., 0.25, 0.8, 0.6]) {
            assert!((actual - expected).abs() < 2e-15);
        }
        for (actual, expected) in result
            .hessians
            .iter()
            .zip([0., 0., 0., -0.0625, 0.072, -0.096, -0.096, 0.128])
        {
            assert!((actual - expected).abs() < 2e-15);
        }
        let invalid = CaseValues {
            scalars: BTreeMap::from([(x, -1.), (y, 4.)]),
        };
        assert!(
            package
                .observe_authored(
                    &source,
                    root,
                    instance,
                    bindings,
                    Limits::default(),
                    rows,
                    Vec::new(),
                    DerivativeOrder::Value,
                    invalid,
                    profile,
                    &driver
                )
                .await
                .is_err()
        );
    }
}
