// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive discovery through the existing retention and canonical publication owners.
use super::*;
use pse_operations::canonical_selection::{ProductHeaderCursor, ProductHeaderPage};

pse_contract::projection_struct! {
/// Exact specialization to inspect without preparing it.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparationDiscoveryRequest {
    projection: { name: "PreparationDiscoveryRequest", rust_path: crate::workflow::PreparationDiscoveryRequest, version: 2, direction: pse_contract::boundary_projection::Direction::Input, reserved_tags: [], reserved_names: [] };
    /// Selected authored root.
    pub root: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Input, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Selected occurrence identity.
    pub instance: InstanceId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Input, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Exact specialization premises, including demand and formulation.
    pub bindings: Bindings => { tag: 3, direction: pse_contract::boundary_projection::Direction::Input, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Exact specialization limits.
    pub limits: Limits => { tag: 4, direction: pse_contract::boundary_projection::Direction::Input, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Continue durable publication headers under the same immutable source.
    pub durable_after: Option<ProductHeaderCursor> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Input, rules: [] };
}
}
/// Receiving compatibility is independent of current availability.
pub use pse_model::generated::enums::PreparationEligibility;
/// Which existing process-local owner retains the observed immutable product.
pub use pse_model::generated::enums::PreparationProductKind;
pse_contract::projection_struct! {
/// Compact observational metadata; contains no live product or native handle.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResidentPreparation {
    projection: { name: "ResidentPreparation", rust_path: crate::workflow::ResidentPreparation, version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Original retention product class.
    pub kind: PreparationProductKind => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Existing exact-key and source-premise assessment.
    pub eligibility: PreparationEligibility => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Immutable insertion weight, including retained key and owner metadata.
    pub retained_bytes: u64 => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original compiler-sealed body identity for body/encoding entries only.
    pub semantic_identity: Option<String> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Bounded observation, not a cache census or a reusable preparation receipt.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparationDiscoveryReport {
    projection: { name: "PreparationDiscoveryReport", rust_path: crate::workflow::PreparationDiscoveryReport, version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Exact local source parent identity inspected.
    pub revision_id: String => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Selected deployment physical identity.
    pub physical: String => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// At most 32 resident metadata rows. Body entries are service-wide and unchecked;
    /// selected admissions/frontiers/bases are scoped to the requested root.
    /// Absence has no inferred cause.
    pub resident: Vec<ResidentPreparation> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [pse_contract::boundary_projection::LocalRule::Length { min: 0, max: PreparationDiscoveryReport::MAX_RESIDENT_ROWS as u64 }] };
    /// More matching process-local entries existed during this observation.
    pub residents_truncated: bool => { tag: 4, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original rooted publication headers, when this package has canonical origin.
    pub durable: Option<ProductHeaderPage> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Passive durable presence never establishes receiving qualification.
    pub durable_eligibility: PreparationEligibility => { tag: 6, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
}
}
impl PreparationDiscoveryReport {
    /// One bound shared by producer selection and outgoing local constraints.
    pub const MAX_RESIDENT_ROWS: usize = 32;
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

    #[test]
    fn preparation_completion_projection_retains_original_scope_and_shared_bound() {
        use pse_contract::boundary_projection::{
            ProjectionGraph, Shape, TypeRef, ValidateProjection,
        };
        let graph = ProjectionGraph::of::<PreparationDiscoveryReport>().unwrap();
        let Shape::Record(fields) = &graph.definitions()["PreparationDiscoveryReport"].shape else {
            panic!("original completion owner")
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| (field.name.as_str(), field.tag))
                .collect::<Vec<_>>(),
            [
                ("revision_id", 1),
                ("physical", 2),
                ("resident", 3),
                ("residents_truncated", 4),
                ("durable", 5),
                ("durable_eligibility", 6),
            ]
        );
        assert_eq!(
            fields[4].ty,
            TypeRef::Optional(Box::new(TypeRef::Named("ProductHeaderPage".into())))
        );
        let Shape::Record(header) = &graph.definitions()["ProductHeader"].shape else {
            panic!("original durable provenance")
        };
        assert_eq!(
            header
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            [
                "key",
                "revision",
                "producer",
                "producer_class",
                "interpretation",
                "descriptor",
                "content",
                "receipt",
            ]
        );
        let Shape::StringEnum(eligibility) = &graph.definitions()["PreparationEligibility"].shape
        else {
            panic!("actual source vocabulary")
        };
        assert_eq!(eligibility, &["compatible", "incompatible", "unchecked"]);
        let resident = ResidentPreparation {
            kind: PreparationProductKind::Body,
            eligibility: PreparationEligibility::Unchecked,
            retained_bytes: 1,
            semantic_identity: None,
        };
        let mut report = PreparationDiscoveryReport {
            revision_id: "exact-local-parent".into(),
            physical: "exact-physical-basis".into(),
            resident: vec![resident.clone(); PreparationDiscoveryReport::MAX_RESIDENT_ROWS],
            residents_truncated: true,
            durable: None,
            durable_eligibility: PreparationEligibility::Unchecked,
        };
        report.validate_projection().unwrap();
        report.resident.push(resident);
        assert!(
            matches!(report.validate_projection(), Err(pse_contract::boundary_projection::ValidationError::Rule { path, rule: "length", .. }) if path == "resident")
        );
    }

    #[test]
    fn preparation_discovery_v2_uses_actual_structural_owner_and_preserves_fact_refusal() {
        use pse_modeling::specialize::Value;
        let root = DeclarationId::from_bytes([0x31; 16]);
        let quantity = pse_quantity::QuantityTypeId::from_bytes([0x32; 16]);
        let mut bindings = Bindings::default();
        bindings.arguments.insert(
            "values".into(),
            Value::Tuple(vec![
                Value::Number {
                    bits: 0x8000_0000_0000_0000,
                    quantity,
                },
                Value::Missing,
                Value::Set(vec![Value::Integer(7)]),
                Value::Row {
                    table: root,
                    names: vec!["z".into(), "a".into()].into(),
                    fields: vec![Value::Text("first".into()), Value::Text("second".into())].into(),
                },
            ]),
        );
        bindings.facts.insert(
            pse_modeling::analysis::Fact::Stage("warm".into()),
            Value::Boolean(true),
        );
        let request = PreparationDiscoveryRequest {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings,
            limits: Limits::default(),
            durable_after: None,
        };
        let bytes = crate::boundary_documents::Wire::FlatV2
            .to_vec(&request)
            .unwrap();
        let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["document_version"], 2);
        let tuple = &json["bindings"]["arguments"]["values"]["Tuple"];
        assert_eq!(
            tuple[0]["Number"]["bits"],
            serde_json::json!(0x8000_0000_0000_0000_u64)
        );
        assert_eq!(
            tuple[0]["Number"]["quantity"],
            serde_json::json!("32323232323232323232323232323232")
        );
        assert_eq!(tuple[1], "Missing");
        assert_eq!(tuple[2]["Set"][0]["Integer"], 7);
        assert_eq!(tuple[3]["Row"]["names"], serde_json::json!(["z", "a"]));
        assert_eq!(
            json["bindings"]["facts"][0][0],
            serde_json::json!({"Stage": "warm"})
        );
        let decoded: PreparationDiscoveryRequest = crate::boundary_documents::Wire::FlatV2
            .decode_slice(&bytes)
            .unwrap();
        assert_eq!(decoded.bindings, request.bindings);
        json["document_version"] = serde_json::json!(1);
        assert!(
            crate::boundary_documents::Wire::FlatV2
                .decode_slice::<PreparationDiscoveryRequest>(&serde_json::to_vec(&json).unwrap())
                .is_err()
        );
        json["document_version"] = serde_json::json!(2);
        let duplicate = json["bindings"]["facts"][0].clone();
        json["bindings"]["facts"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        let refusal = crate::boundary_documents::Wire::FlatV2
            .decode_slice::<PreparationDiscoveryRequest>(&serde_json::to_vec(&json).unwrap())
            .unwrap_err();
        assert!(
            refusal
                .inner()
                .to_string()
                .contains("duplicate structural fact")
        );
        assert_eq!(
            refusal.location().category,
            pse_model::document::DecodeCategory::Data
        );
    }

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
