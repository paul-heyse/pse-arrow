// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive discovery through the existing retention and canonical publication owners.
use super::*;
#[path = "project_preparation_wire.rs"]
mod wire;
use pse_operations::canonical_selection::{ProductHeaderCursor, ProductHeaderPage};

/// Exact specialization to inspect without preparing it.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparationDiscoveryRequest {
    /// Selected authored root.
    pub root: DeclarationId,
    /// Selected occurrence identity.
    pub instance: InstanceId,
    /// Exact specialization premises, including demand and formulation.
    #[serde(with = "wire")]
    #[schemars(with = "wire::PreparationBindings")]
    pub bindings: Bindings,
    /// Exact specialization limits.
    pub limits: Limits,
    /// Continue durable publication headers under the same immutable source.
    pub durable_after: Option<ProductHeaderCursor>,
}
/// Receiving compatibility is independent of current availability.
pub use pse_model::generated::enums::PreparationEligibility;
/// Which existing process-local owner retains the observed immutable product.
pub use pse_model::generated::enums::PreparationProductKind;
/// Compact observational metadata; contains no live product or native handle.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResidentPreparation {
    /// Original retention product class.
    pub kind: PreparationProductKind,
    /// Existing exact-key and source-premise assessment.
    pub eligibility: PreparationEligibility,
    /// Immutable insertion weight, including retained key and owner metadata.
    pub retained_bytes: u64,
    /// Original compiler-sealed body identity for body/encoding entries only.
    pub semantic_identity: Option<String>,
}
/// Bounded observation, not a cache census or a reusable preparation receipt.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparationDiscoveryReport {
    /// Exact local source parent identity inspected.
    pub revision_id: String,
    /// Selected deployment physical identity.
    pub physical: String,
    /// At most 32 resident metadata rows. Body entries are service-wide and unchecked;
    /// selected admissions/frontiers/bases are scoped to the requested root.
    /// Absence has no inferred cause.
    pub resident: Vec<ResidentPreparation>,
    /// More matching process-local entries existed during this observation.
    pub residents_truncated: bool,
    /// Original rooted publication headers, when this package has canonical origin.
    pub durable: Option<ProductHeaderPage>,
    /// Passive durable presence never establishes receiving qualification.
    pub durable_eligibility: PreparationEligibility,
}
impl SelectedRequest {
    pub(crate) fn includes_root(&self, root: DeclarationId) -> bool {
        self.roots.contains(&root)
    }
}
impl ModelingPackage {
    /// Observe existing products without admission, compilation, provider observation,
    /// evaluator construction, scientific payload hydration or recency promotion.
    pub async fn preparation_discovery(
        &self,
        services: &crate::workflow::SourceServices,
        request: &PreparationDiscoveryRequest,
        cancel: &crate::CancelSource,
    ) -> Result<PreparationDiscoveryReport, WorkflowError> {
        request.limits.validate().map_err(modeling_error)?;
        self.check_service_bindings(services.preparation())?;
        let activity = services.preparation().admit_activity()?;
        activity
            .scope(async {
                let deadline = services.preparation().math().preparation_deadline(cancel)?;
                preparation::scoped(cancel, deadline, async {
                    let selected = self.selection_request(services.preparation(), &[request.root]);
                    let (resident, residents_truncated) = services
                        .preparation()
                        .math()
                        .modeling_cache
                        .inspect_preparations(
                            &selected,
                            request,
                            &self.revision.admitted,
                            cancel,
                        )?;
                    let durable = if let Some(origin) = self.canonical_revision() {
                        let store = services.canonical_store()?;
                        let selection = store
                            .protect(origin.clone(), std::time::Duration::from_secs(60))
                            .await?;
                        let result = store
                            .product_headers(&selection, request.durable_after.as_ref())
                            .await;
                        let released = store.release(&selection).await;
                        match result {
                            Err(error) => return Err(error.into()),
                            Ok(page) => {
                                released?;
                                Some(page)
                            }
                        }
                    } else {
                        if request.durable_after.is_some() {
                            return Err(contract(
                                "durable product cursor requires a canonical source origin",
                            ));
                        }
                        None
                    };
                    Ok(PreparationDiscoveryReport {
                        revision_id: self.revision_id().into(),
                        physical: self.physical.key.to_hex(),
                        resident,
                        residents_truncated,
                        durable,
                        durable_eligibility: PreparationEligibility::Unchecked,
                    })
                })
                .await
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declarations(source: &str) -> Vec<Declaration> {
        pse_authoring::language::parse(
            source,
            SemanticId::from_bytes([91; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap()
    }

    #[derive(Debug)]
    struct DescriptorOnlyProvider {
        spec: pse_kernels::ProviderSpec,
        configuration: pse_ids::ContentHash,
    }
    impl pse_kernels::ProviderFactory for DescriptorOnlyProvider {
        fn spec(&self) -> &pse_kernels::ProviderSpec {
            &self.spec
        }
        fn configuration_key(&self) -> pse_ids::ContentHash {
            self.configuration
        }
        fn create(&self) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
            Err(pse_kernels::ProviderError::Contract(
                "passive discovery must not instantiate providers".into(),
            ))
        }
    }

    #[tokio::test]
    async fn passive_preparation_discovery_rejects_changed_provider_configuration() {
        use pse_kernels::{
            AdmittedProvider, DerivativeOrder, DerivativeSource, Port, ProviderShapes,
            ProviderSpec, Registration,
        };
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let quantity = physical.quantities.neutral_dimensionless().unwrap();
        let unit = physical
            .quantities
            .quantity_type(quantity)
            .unwrap()
            .canonical_unit;
        let port = |n| Port {
            id: SemanticId::from_bytes([n; 16]),
            quantity,
            unit,
        };
        let spec = ProviderSpec {
            shapes: ProviderShapes::default(),
            derivative_source: DerivativeSource::Analytic,
            id: SemanticId::from_bytes([92; 16]),
            revision: pse_ids::ContentHash::from_bytes([93; 32]),
            data: pse_ids::ContentHash::from_bytes([94; 32]),
            inputs: vec![port(95)],
            outputs: vec![port(96)],
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        let registration = |n| {
            Registration::bind(
                AdmittedProvider::new(spec.clone(), &physical.quantities).unwrap(),
                Arc::new(DescriptorOnlyProvider {
                    spec: spec.clone(),
                    configuration: pse_ids::ContentHash::from_bytes([n; 32]),
                }),
            )
            .unwrap()
        };
        let rows = declarations(&format!(
            "package p {{fn law(x:Scalar)->Scalar external \"law\" revision \"{}\" data \"{}\" output 0 derivatives 1 source analytic smoothness 1; def Root {{var x:Scalar; eq e:law(x)==2;}}}}",
            spec.revision, spec.data
        ));
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let first = runtime
            .modeling_package_registered(
                rows,
                physical.clone(),
                BTreeMap::from([("law".into(), registration(97))]),
            )
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let _selected = first
            .checked_selection(&runtime.preparation_services(), &[root], &cancel)
            .await
            .unwrap();
        let request = PreparationDiscoveryRequest {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            durable_after: None,
        };
        let effects = runtime.shared.math().preparation_effects();
        assert!(
            first
                .preparation_discovery(&runtime.source_services(), &request, &cancel)
                .await
                .unwrap()
                .resident
                .iter()
                .any(|row| row.eligibility == PreparationEligibility::Compatible)
        );
        let mut changed = first.clone();
        changed.providers = Arc::new(BTreeMap::from([("law".into(), registration(98))]));
        let report = changed
            .preparation_discovery(&runtime.source_services(), &request, &cancel)
            .await
            .unwrap();
        assert!(
            report
                .resident
                .iter()
                .any(|row| row.kind == PreparationProductKind::SelectedAdmission
                    && row.eligibility == PreparationEligibility::Incompatible)
        );
        assert_eq!(effects, runtime.shared.math().preparation_effects());
    }

    #[tokio::test]
    async fn passive_preparation_discovery_preserves_observed_value_and_structural_body_identities()
    {
        let runtime = crate::workflow::tests::runtime();
        let package = runtime
            .modeling_package(
                declarations(
                    "package p { def Root { param p:Scalar=1; var x:Scalar; eq a:x*x==p; } }",
                ),
                crate::workflow::tests::physical(),
            )
            .await
            .unwrap();
        let root = package
            .declarations(&runtime.preparation_services())
            .await
            .unwrap()
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let request = PreparationDiscoveryRequest {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            durable_after: None,
        };
        let cancel = crate::CancelSource::new();
        let first = package
            .prepare(
                &runtime.preparation_services(),
                root,
                request.instance,
                request.bindings.clone(),
                request.limits,
                &cancel,
            )
            .await
            .unwrap();
        let identities = |value: &ModelingPreparation| {
            value
                .compiled()
                .admitted
                .bodies
                .values()
                .filter_map(|body| body.semantic_identity())
                .collect::<std::collections::BTreeSet<_>>()
        };
        let original = identities(&first);
        assert!(!original.is_empty());
        let changed_value = package
            .with_declarations(
                &runtime.source_services(),
                declarations(
                    "package p { def Root { param p:Scalar=2; var x:Scalar; eq a:x*x==p; } }",
                ),
            )
            .await
            .unwrap();
        let value = changed_value
            .prepare(
                &runtime.preparation_services(),
                root,
                request.instance,
                request.bindings.clone(),
                request.limits,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            original,
            identities(&value),
            "actual admitted parameter slots retain semantic bodies after a value edit"
        );
        let changed_structure = changed_value
            .with_declarations(
                &runtime.source_services(),
                declarations(
                    "package p { def Root { param p:Scalar=2; var x:Scalar; eq a:x*x*x==p; } }",
                ),
            )
            .await
            .unwrap();
        let structure = changed_structure
            .prepare(
                &runtime.preparation_services(),
                root,
                request.instance,
                request.bindings.clone(),
                request.limits,
                &cancel,
            )
            .await
            .unwrap();
        let structural = identities(&structure);
        assert_ne!(
            original, structural,
            "the actual compiler establishes different mathematics"
        );
        let effects = runtime.shared.math().preparation_effects();
        let report = changed_structure
            .preparation_discovery(&runtime.source_services(), &request, &cancel)
            .await
            .unwrap();
        let observed = report
            .resident
            .iter()
            .filter(|row| row.kind == PreparationProductKind::Body)
            .filter_map(|row| row.semantic_identity.clone())
            .collect::<std::collections::BTreeSet<_>>();
        for identity in original.iter().chain(&structural) {
            assert!(observed.contains(&identity.as_id().to_hex()));
        }
        assert!(
            report
                .resident
                .iter()
                .filter(|row| row.kind == PreparationProductKind::Body)
                .all(|row| row.eligibility == PreparationEligibility::Unchecked)
        );
        assert_eq!(effects, runtime.shared.math().preparation_effects());
    }

    #[tokio::test]
    async fn passive_preparation_discovery_observes_eligibility_without_preparing_or_promoting() {
        let runtime = crate::workflow::tests::runtime();
        let source = "package p { def Root { var x:Scalar; eq a:x*x==1; } }";
        let package = runtime
            .modeling_package(declarations(source), crate::workflow::tests::physical())
            .await
            .unwrap();
        let root = package
            .declarations(&runtime.preparation_services())
            .await
            .unwrap()
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let request = PreparationDiscoveryRequest {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            durable_after: None,
        };
        let cancel = crate::CancelSource::new();
        let math = runtime.shared.math();
        let cold_effects = math.preparation_effects();
        let cold = package
            .preparation_discovery(&runtime.source_services(), &request, &cancel)
            .await
            .unwrap();
        assert!(cold.resident.is_empty());
        assert!(cold.durable.is_none());
        assert_eq!(cold.durable_eligibility, PreparationEligibility::Unchecked);
        assert_eq!(cold_effects, math.preparation_effects());
        let _prepared = package
            .prepare(
                &runtime.preparation_services(),
                root,
                request.instance,
                request.bindings.clone(),
                request.limits,
                &cancel,
            )
            .await
            .unwrap();
        let effects = math.preparation_effects();
        let before = math.modeling_cache.report(0);
        for _ in 0..3 {
            let observed = package
                .preparation_discovery(&runtime.source_services(), &request, &cancel)
                .await
                .unwrap();
            assert!(
                observed
                    .resident
                    .iter()
                    .any(|row| row.kind == PreparationProductKind::Basis
                        && row.eligibility == PreparationEligibility::Compatible)
            );
        }
        let after = math.modeling_cache.report(0);
        assert_eq!(
            (
                before.hits,
                before.misses,
                before.entries,
                before.retained_bytes
            ),
            (
                after.hits,
                after.misses,
                after.entries,
                after.retained_bytes
            )
        );
        assert_eq!(effects, math.preparation_effects());
        let changed = package
            .with_declarations(
                &runtime.source_services(),
                declarations("package p { def Root { var x:Scalar; eq a:x*x==2; } }"),
            )
            .await
            .unwrap();
        let observed = changed
            .preparation_discovery(&runtime.source_services(), &request, &cancel)
            .await
            .unwrap();
        assert!(
            observed
                .resident
                .iter()
                .any(|row| row.kind == PreparationProductKind::Basis
                    && row.eligibility == PreparationEligibility::Incompatible)
        );
        let mut changed_request = request.clone();
        changed_request.limits.depth += 1;
        let observed = package
            .preparation_discovery(&runtime.source_services(), &changed_request, &cancel)
            .await
            .unwrap();
        assert!(
            observed
                .resident
                .iter()
                .any(|row| row.kind == PreparationProductKind::Basis
                    && row.eligibility == PreparationEligibility::Incompatible)
        );
        math.clear_program_cache();
        let before = math.preparation_effects();
        let cleared = package
            .preparation_discovery(&runtime.source_services(), &request, &cancel)
            .await
            .unwrap();
        assert!(
            cleared.resident.is_empty(),
            "absence reports no invented eviction or invalidation cause"
        );
        assert_eq!(before, math.preparation_effects());
    }
}
