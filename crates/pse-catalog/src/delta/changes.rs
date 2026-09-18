// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded native CDF over an exact publication selection. A missing log entry
//! or changed declaration refuses the window; it never becomes an empty delta.
use datafusion::functions::core::expr_fn::named_struct;
use datafusion::{
    catalog::TableProvider,
    common::{
        Column, DataFusionError, ResolvedTableReference, Result, ScalarValue, TableReference,
    },
    datasource::provider_as_source,
    logical_expr::{Expr, LogicalPlanBuilder, lit},
};
use deltalake::{delta_datafusion::DeltaCdfTableProvider, kernel::Action};
use pse_ids::CancellationToken;
use pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItemSelectionSelected as Selected;
use std::sync::Arc;

impl crate::session::SnapshotSession {
    /// Prepare changes after `after_version`, through this publication's selected
    /// member version (inclusive). Each row contains `change` (typed native change
    /// identity, version, key and timestamp) and `value` (the complete declared row).
    /// Updates retain distinct preimage and postimage events; no positional pairing.
    /// Revision selections filter both images through the same native predicate.
    /// # Errors
    /// Unselected member, invalid window, lost history, changed contract, policy
    /// refusal or unsupported native CDF. Callers may conservatively recompute.
    pub async fn prepare_changes(
        &self,
        reference: &ResolvedTableReference,
        after_version: i64,
        cancel: &CancellationToken,
    ) -> std::result::Result<crate::session::PreparedComputation, crate::CatalogError> {
        let (session, plan) = self
            .change_plan(reference, after_version, cancel)
            .await
            .map_err(crate::session::engine)?;
        session.prepare(plan, cancel)
    }

    pub(crate) async fn change_plan(
        &self,
        reference: &ResolvedTableReference,
        after: i64,
        cancel: &CancellationToken,
    ) -> Result<(Self, datafusion::logical_expr::LogicalPlan)> {
        let member = self.selected_member(reference).map_err(external)?;
        let end = super::provider::delta_version(member.delta_version)?;
        let after = super::provider::delta_version(after)?;
        if after > end {
            return Err(invalid(
                "CDF predecessor is newer than the selected version",
            ));
        }
        let location = url::Url::parse(&member.table_uri).map_err(external)?;
        let lease = super::lease::read(&location, cancel).await?;
        let state = self.bound_state().map_err(external)?;
        let state = crate::cache_service::bind_state(&location, &Arc::new(state))?;
        let contract = super::contract::DeclaredCheck::new(self.registry(), member.relation_id)?;
        let opened = cancel
            .until_cancelled(super::provider::open_native(
                location.clone(),
                Some(end),
                crate::cache_service::snapshot::LoadRequirement::Query,
                &state,
            ))
            .await
            .map_err(external)??;
        let table = &opened.table;
        contract.verify(table)?;
        if after != end {
            let predecessor = cancel
                .until_cancelled(super::provider::open_native(
                    location,
                    Some(after),
                    crate::cache_service::snapshot::LoadRequirement::Metadata,
                    &state,
                ))
                .await
                .map_err(external)??;
            contract.verify(&predecessor.table)?;
        }
        // The lower endpoint is a selected state, not a request to fabricate CDF
        // for earlier versions. Its descriptor must match the same declaration.
        let first = after.saturating_add(1).min(end);
        verify_history(table, &contract, after, end, cancel, &state).await?;
        let provider: Arc<dyn TableProvider> = Arc::new(
            DeltaCdfTableProvider::try_new(
                table
                    .clone()
                    .scan_cdf()
                    .with_file_metadata_cache(
                        state.runtime_env().cache_manager.get_file_metadata_cache(),
                    )
                    .with_starting_version(first)
                    .with_ending_version(end),
            )
            .map_err(external)?,
        );
        let provider = super::leased::retain_reader_budget(provider, &state);
        let provider = match opened.owner {
            Some(owner) => super::leased::retain_snapshot(provider, owner),
            None => provider,
        };
        let provider = super::leased::retain(provider, lease.clone());
        let raw_name = TableReference::full(
            "pse_changes",
            reference.schema.clone(),
            reference.table.clone(),
        );
        let mut session = self
            .with_provider(raw_name.clone(), Arc::clone(&provider), cancel)
            .map_err(external)?;
        if let Some(lease) = lease {
            session.leases.push(lease);
        }
        let input =
            LogicalPlanBuilder::scan(raw_name, provider_as_source(provider), None)?.build()?;
        let mut plan = contract.layout().decode_with_native_tail(input)?;
        if let Selected::Revision(selection) = member.selection.selected().map_err(external)? {
            plan = LogicalPlanBuilder::from(plan)
                .filter(Expr::Column(Column::from_name(&selection.column)).eq(lit(
                    ScalarValue::FixedSizeBinary(
                        16,
                        Some(selection.revision_id.as_bytes().to_vec()),
                    ),
                )))?
                .build()?;
        }
        if after == end {
            plan = LogicalPlanBuilder::from(plan).limit(0, Some(0))?.build()?;
        }
        let plan = normalize(&session, &member, plan)?;
        Ok((session, plan))
    }
}
async fn verify_history(
    table: &deltalake::DeltaTable,
    contract: &super::contract::DeclaredCheck,
    after: u64,
    end: u64,
    cancel: &CancellationToken,
    state: &datafusion::execution::session_state::SessionState,
) -> Result<()> {
    for version in after..=end {
        cancel.checkpoint().map_err(external)?;
        let (entry, _decode_owner) = super::attempt::receipt_bytes(table, version, state).await?;
        for action in serde_json::Deserializer::from_slice(&entry).into_iter::<Action>() {
            if let Action::Metadata(metadata) = action.map_err(external)? {
                for (key, value) in contract.properties() {
                    if metadata.configuration().get(key) != Some(value) {
                        return Err(invalid(
                            "CDF window crosses a different declared table contract",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn normalize(
    session: &crate::session::SnapshotSession,
    member: &pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItem,
    plan: datafusion::logical_expr::LogicalPlan,
) -> Result<datafusion::logical_expr::LogicalPlan> {
    let spec = session
        .registry()
        .relation_by_id(member.relation_id)
        .ok_or_else(|| invalid("CDF relation declaration is absent"))?;
    let column = |name: &str| Expr::Column(Column::from_name(name));
    let key = crate::session::scalar::key(
        spec.id,
        spec.primary_key
            .iter()
            .map(|name| (*name, column(name)))
            .collect(),
    );
    let checked = session
        .scalar_function("pse_checked_value")
        .map_err(external)?;
    let timestamp = pse_relations::generated::runtime::change_events::spec(session.registry())
        .map_err(external)?
        .column("committed_at")
        .ok_or_else(|| invalid("CDF timestamp declaration is absent"))?
        .data_type();
    let change = named_struct(
        [
            ("table_uri", lit(member.table_uri.clone())),
            (
                "relation_id",
                lit(ScalarValue::FixedSizeBinary(
                    16,
                    Some(member.relation_id.as_bytes().to_vec()),
                )),
            ),
            (
                "contract_fingerprint",
                lit(ScalarValue::FixedSizeBinary(
                    32,
                    Some(member.contract_fingerprint.as_bytes().to_vec()),
                )),
            ),
            (
                "commit_version",
                datafusion::logical_expr::cast(
                    column("_commit_version"),
                    datafusion::arrow::datatypes::DataType::Int64,
                ),
            ),
            (
                "kind",
                datafusion::logical_expr::cast(
                    column("_change_type"),
                    datafusion::arrow::datatypes::DataType::Utf8,
                ),
            ),
            ("row_key", key),
            (
                "committed_at",
                datafusion::logical_expr::cast(column("_commit_timestamp"), timestamp),
            ),
        ]
        .into_iter()
        .flat_map(|(name, value)| {
            [
                lit(name),
                checked.call(vec![value, lit("runtime.change_events"), lit(name)]),
            ]
        })
        .collect(),
    );
    let value = named_struct(
        spec.columns
            .iter()
            .flat_map(|field| [lit(field.name()), column(field.name())])
            .collect(),
    );
    let plan = LogicalPlanBuilder::from(plan)
        .project(vec![change.alias("change"), value.alias("value")])?
        .build()?;
    Ok(plan)
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}

#[cfg(test)]
mod tests;
