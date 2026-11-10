// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original selected analytical products, ready for the shared result publisher.
use super::*;
use crate::workflow::document_extent;
use pse_compiler::workspace::{AnalyticalOperation, AnalyticalSource, CompiledAnalyticalResult};
use pse_ids::ContentHash;
use pse_math::analytical::AnalyticalObservation;
/// Owned original observation and its native allocation owner. Mathematical text
/// and physical/source context are frozen here; readers never rerun this producer.
#[derive(Clone, Debug)]
pub struct AuthoredAnalyticalResult {
    /// Exact immutable producing source revision.
    pub source_revision: ContentHash,
    /// Actual original authored package versions.
    pub source_versions: BTreeMap<String, String>,
    /// Actual original mathematical demand.
    pub operation: AnalyticalOperation,
    /// Immutable original admitted source or its complete owned projection.
    pub source: AnalyticalSource,
    /// Frozen mathematical observation produced once.
    pub observation: AnalyticalObservation,
    native: Arc<pse_columnar::Leased<CompiledAnalyticalResult>>,
    _projection_owner: Arc<dyn pse_columnar::PayloadOwner>,
}
impl AuthoredAnalyticalResult {
    /// Borrow the native mathematical product and its retained original allocation owner.
    pub fn native(&self) -> &CompiledAnalyticalResult {
        &self.native
    }
}
impl ModelingPackage {
    /// Select through the same checked source/frontier owners as ordinary modeling.
    /// The request names an original member and coordinates, never an Atom parser.
    #[expect(
        clippy::too_many_arguments,
        reason = "source specialization, selected analytical demand and limits, and the original execution owner remain explicit"
    )]
    pub async fn analyze_authored(
        &self,
        services: &crate::workflow::PreparationServices,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        member: SemanticId,
        operation: AnalyticalOperation,
        analytical_limits: pse_math::jets::EvaluationLimits,
        cancel: &crate::CancelSource,
    ) -> Result<AuthoredAnalyticalResult, WorkflowError> {
        limits.validate().map_err(modeling_error)?;
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let driver = services.math().preparation_driver(cancel)?;
                let deadline = services.math().preparation_deadline(&driver)?;
                let selected = self
                    .checked_selection_scoped(services, &[root], &driver, deadline)
                    .await?;
                let source_revision = selected.admission.revision.identity().as_id();

                let key = services.math().basis_key(
                    &selected.admission,
                    root,
                    instance,
                    bindings,
                    limits,
                )?;
                let workspace = self.pure_workspace(services)?;
                let frontier = services
                    .math()
                    .plan_frontier_keyed(
                        key,
                        workspace.clone(),
                        selected.admission.revision.clone(),
                        &driver,
                        deadline,
                    )
                    .await?;
                let native = services
                    .math()
                    .analytical_member_for(
                        workspace,
                        frontier,
                        member,
                        operation.clone(),
                        analytical_limits,
                        &driver,
                    )
                    .await?;
                let projection_bytes = document_extent(
                    &(
                        &native.operation,
                        &native.source,
                        &native.observation,
                        selected.admission.versions.as_ref(),
                    ),
                    usize::MAX,
                )?
                .checked_mul(128)
                .and_then(|bytes| bytes.checked_add(size_of::<AuthoredAnalyticalResult>()))
                .ok_or_else(|| {
                    WorkflowError::Resource("analytical owned source projection extent".into())
                })?;
                let projection_owner = services
                    .math()
                    .reserve("analytical:owned-source-projection", projection_bytes)?;
                Ok(AuthoredAnalyticalResult {
                    source_revision,
                    source_versions: selected.admission.versions.as_ref().clone(),
                    operation,
                    source: native.source.clone(),
                    observation: native.observation.clone(),
                    native,
                    _projection_owner: projection_owner,
                })
            })
            .await
    }
}
