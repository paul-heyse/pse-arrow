// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive discovery through the existing retention and canonical publication owners.
use super::*;
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
    pub bindings: Bindings,
    /// Exact specialization limits.
    pub limits: Limits,
    /// Continue durable publication headers under the same immutable source.
    pub durable_after: Option<ProductHeaderCursor>,
}
/// Which existing process-local owner retains the observed immutable product.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PreparationProductKind {
    /// Exact local selected admission and its original source dependencies.
    SelectedAdmission,
    /// Compiler-sealed resident mathematical body; selected-request eligibility is unchecked.
    Body,
    /// Resident portable encoding; receiving eligibility is unchecked.
    EncodedBody,
    /// Planned specialization frontier.
    Frontier,
    /// Completed mathematical preparation basis.
    Basis,
}
/// Receiving compatibility is independent of current availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PreparationEligibility {
    /// Existing owner establishes exact source/context/request equality now.
    Compatible,
    /// Existing owner establishes that at least one exact premise differs.
    Incompatible,
    /// No receiving compatibility claim is established by this observation.
    Unchecked,
}
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
    pub(crate) fn includes_root(&self, root: DeclarationId) -> bool { self.roots.contains(&root) }
}
impl ModelingPackage {
    /// Observe existing products without admission, compilation, provider observation,
    /// evaluator construction, scientific payload hydration or recency promotion.
    pub async fn preparation_discovery(
        &self,
        request: &PreparationDiscoveryRequest,
        cancel: &crate::CancelSource,
    ) -> Result<PreparationDiscoveryReport, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity.scope(async {
            let deadline = self.runtime.shared.math().preparation_deadline(cancel)?;
            preparation::scoped(cancel, deadline, async {
                let selected = self.selection_request(&[request.root]);
                let (resident, residents_truncated) = self.runtime.shared.math().modeling_cache
                    .inspect_preparations(&selected, request, &self.revision.admitted, cancel)?;
                let durable = if let Some(origin) = self.canonical_revision() {
                    let store = self.runtime.canonical_store()?;
                    let selection = store.protect(origin.clone(), std::time::Duration::from_secs(60)).await?;
                    let result = store.product_headers(&selection, request.durable_after.as_ref()).await;
                    let released = store.release(&selection).await;
                    match result {
                        Err(error) => return Err(error.into()),
                        Ok(page) => { released?; Some(page) }
                    }
                } else {
                    if request.durable_after.is_some() {
                        return Err(contract("durable product cursor requires a canonical source origin"));
                    }
                    None
                };
                Ok(PreparationDiscoveryReport {
                    revision_id: self.revision_id().into(), physical: self.physical.key.to_hex(),
                    resident, residents_truncated, durable,
                    durable_eligibility: PreparationEligibility::Unchecked,
                })
            }).await
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declarations(source: &str) -> Vec<Declaration> {
        pse_authoring::language::parse(source, SemanticId::from_bytes([91; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap()
    }

    #[tokio::test]
    async fn passive_preparation_discovery_observes_eligibility_without_preparing_or_promoting() {
        let runtime = crate::workflow::tests::runtime();
        let source = "package p { def Root { var x:Scalar; eq a:x*x==1; } }";
        let package = runtime.modeling_package(declarations(source), crate::workflow::tests::physical()).await.unwrap();
        let root = package.declarations().await.unwrap().iter().find(|row| row.name == "Root").unwrap().declaration_id;
        let request = PreparationDiscoveryRequest {
            root, instance: pse_modeling::specialize::root_instance(root), bindings: Bindings::default(),
            limits: Limits::default(), durable_after: None,
        };
        let cancel = crate::CancelSource::new();
        let math = runtime.shared.math();
        let cold_effects = math.preparation_effects();
        let cold = package.preparation_discovery(&request, &cancel).await.unwrap();
        assert!(cold.resident.is_empty());
        assert!(cold.durable.is_none());
        assert_eq!(cold.durable_eligibility, PreparationEligibility::Unchecked);
        assert_eq!(cold_effects, math.preparation_effects());
        let _prepared = package.prepare(root, request.instance, request.bindings.clone(), request.limits, &cancel).await.unwrap();
        let effects = math.preparation_effects();
        let before = math.modeling_cache.report(0);
        for _ in 0..3 {
            let observed = package.preparation_discovery(&request, &cancel).await.unwrap();
            assert!(observed.resident.iter().any(|row| row.kind == PreparationProductKind::Basis && row.eligibility == PreparationEligibility::Compatible));
        }
        let after = math.modeling_cache.report(0);
        assert_eq!((before.hits, before.misses, before.entries, before.retained_bytes),
            (after.hits, after.misses, after.entries, after.retained_bytes));
        assert_eq!(effects, math.preparation_effects());
        let changed = package.with_declarations(declarations("package p { def Root { var x:Scalar; eq a:x*x==2; } }")).await.unwrap();
        let observed = changed.preparation_discovery(&request, &cancel).await.unwrap();
        assert!(observed.resident.iter().any(|row| row.kind == PreparationProductKind::Basis && row.eligibility == PreparationEligibility::Incompatible));
        let mut changed_request = request.clone();
        changed_request.limits.depth += 1;
        let observed = package.preparation_discovery(&changed_request, &cancel).await.unwrap();
        assert!(observed.resident.iter().any(|row| row.kind == PreparationProductKind::Basis && row.eligibility == PreparationEligibility::Incompatible));
        math.clear_program_cache();
        let before = math.preparation_effects();
        let cleared = package.preparation_discovery(&request, &cancel).await.unwrap();
        assert!(cleared.resident.is_empty(), "absence reports no invented eviction or invalidation cause");
        assert_eq!(before, math.preparation_effects());
    }
}
