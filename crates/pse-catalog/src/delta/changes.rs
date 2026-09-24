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
use pse_columnar::CancellationToken;
use pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItemSelectionSelected as Selected;
use std::sync::Arc;

/// Prepare changes after `after_version`, through this publication's selected
/// member version (inclusive). Each row contains `change` (typed native change
/// identity, version, key and timestamp) and `value` (the complete declared row).
/// Updates retain distinct preimage and postimage events; no positional pairing.
/// Revision selections filter both images through the same native predicate.
/// # Errors
/// Unselected member, invalid window, lost history, changed contract, policy
/// refusal or unsupported native CDF. Callers may conservatively recompute.
pub async fn prepare_changes(
    session: &pse_engine::EngineSession,
    reference: &ResolvedTableReference,
    after_version: i64,
    cancel: &CancellationToken,
) -> std::result::Result<pse_engine::session::PreparedComputation, crate::EngineError> {
    let (session, plan) = change_plan(session, reference, after_version, cancel)
        .await
        .map_err(pse_engine::session::engine)?;
    session.prepare(plan, cancel)
}

/// Project all declared values from both CDF images through the existing lossless codec.
/// The compiler extracts generated primary keys; image order carries no pairing meaning.
/// # Errors
/// Same qualification, policy and execution refusals as `prepare_changes`.
pub async fn prepare_change_values(
    session: &pse_engine::EngineSession,
    reference: &ResolvedTableReference,
    after: i64,
    cancel: &CancellationToken,
) -> std::result::Result<pse_engine::session::PreparedComputation, crate::EngineError> {
    let member = crate::selection::selected_member(session, reference)?;
    let (session, plan) = change_plan(session, reference, after, cancel)
        .await
        .map_err(pse_engine::session::engine)?;
    let spec = session
        .registry()
        .relation_by_id(member.relation_id)
        .ok_or_else(|| pse_engine::session::engine(invalid("CDF relation absent")))?;
    let fields = spec.columns.iter().map(|field| {
        datafusion::functions::core::expr_fn::get_field(
            Expr::Column(Column::from_name("value")),
            field.name(),
        )
        .alias(field.name())
    });
    let plan = LogicalPlanBuilder::from(plan).project(fields)?.build()?;
    let plan =
        pse_engine::session::output::declare_relation_output(plan, session.registry(), spec)?;
    session.prepare(plan, cancel)
}

/// Fall back only for positively identified unavailable CDF data/capabilities.
/// Resource, cancellation, permission, corrupt data and unknown infrastructure errors propagate.
pub fn cdf_unavailable(error: &crate::EngineError) -> bool {
    let pse_engine::EngineError::Engine(error) = error else {
        return false;
    };
    !error.failures().is_empty()
        && error.failures().iter().all(|failure| {
            let mut cause = Some(failure.leaf_cause());
            while let Some(error) = cause {
                if error
                    .downcast_ref::<deltalake::DeltaTableError>()
                    .is_some_and(|error| {
                        matches!(
                            error,
                            deltalake::DeltaTableError::ChangeDataNotRecorded { .. }
                                | deltalake::DeltaTableError::ChangeDataNotEnabled { .. }
                                | deltalake::DeltaTableError::UnsupportedColumnMapping { .. }
                                | deltalake::DeltaTableError::SchemaMismatch { .. }
                                | deltalake::DeltaTableError::InvalidVersion(_)
                        )
                    })
                    || error
                        .downcast_ref::<object_store::Error>()
                        .is_some_and(|error| {
                            matches!(
                                error,
                                object_store::Error::NotFound { .. }
                                    | object_store::Error::NotImplemented { .. }
                            )
                        })
                {
                    return true;
                }
                cause = error.source();
            }
            false
        })
}

pub(crate) async fn change_plan(
    session: &pse_engine::EngineSession,
    reference: &ResolvedTableReference,
    after: i64,
    cancel: &CancellationToken,
) -> Result<(
    pse_engine::EngineSession,
    datafusion::logical_expr::LogicalPlan,
)> {
    let member = crate::selection::selected_member(session, reference).map_err(external)?;
    let end = super::provider::delta_version(member.delta_version)?;
    let after = super::provider::delta_version(after)?;
    if after > end {
        return Err(invalid(
            "CDF predecessor is newer than the selected version",
        ));
    }
    if after == end {
        return empty_changes(session, reference, &member);
    }
    let location = url::Url::parse(&member.table_uri)
        .map_err(|error| DataFusionError::External(Box::new(error)))?;
    let lease = super::lease::read(&location, cancel).await?;
    let state = session.bound_state().map_err(external)?;
    let state = super::provider::bind_cache_state(&location, &Arc::new(state), lease.as_deref())?;
    let contract = super::contract::DeclaredCheck::new(session.registry(), member.relation_id)?;
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
    {
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
    let mut session = session
        .with_provider(raw_name.clone(), Arc::clone(&provider), cancel)
        .map_err(external)?;
    if let Some(lease) = lease {
        session.retain_owner(lease);
    }
    let input = LogicalPlanBuilder::scan(raw_name, provider_as_source(provider), None)?.build()?;
    let mut plan = contract.layout().decode_with_native_tail(input)?;
    if let Selected::Revision(selection) = member
        .selection
        .selected()
        .map_err(pse_columnar::external)?
    {
        plan = LogicalPlanBuilder::from(plan)
            .filter(Expr::Column(Column::from_name(&selection.column)).eq(lit(
                ScalarValue::FixedSizeBinary(16, Some(selection.revision_id.as_bytes().to_vec())),
            )))?
            .build()?;
    }
    let plan = normalize(&session, &member, plan)?;
    Ok((session, plan))
}
fn empty_changes(
    session: &pse_engine::EngineSession,
    reference: &ResolvedTableReference,
    member: &pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItem,
) -> Result<(
    pse_engine::EngineSession,
    datafusion::logical_expr::LogicalPlan,
)> {
    let plan = session
        .relation_plan(reference)
        .map_err(external)?
        .plan()
        .clone();
    let mut fields = plan
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect::<Vec<_>>();
    fields.extend([
        lit(ScalarValue::Utf8(None)).alias("_change_type"),
        lit(ScalarValue::UInt64(None)).alias("_commit_version"),
        lit(ScalarValue::TimestampMillisecond(None, None)).alias("_commit_timestamp"),
    ]);
    let plan = LogicalPlanBuilder::from(plan)
        .limit(0, Some(0))?
        .project(fields)?
        .build()?;
    Ok((session.clone(), normalize(session, member, plan)?))
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
        let actions = super::actions::read(table, version, state, cancel).await?;
        for action in actions.iter() {
            if let Action::Metadata(metadata) = action? {
                for (key, value) in contract.properties() {
                    if metadata.configuration().get(key) != Some(value) {
                        return Err(DataFusionError::External(Box::new(
                            deltalake::DeltaTableError::SchemaMismatch {
                                msg: "CDF window crosses a different declared table contract"
                                    .into(),
                            },
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

fn normalize(
    session: &pse_engine::session::EngineSession,
    member: &pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItem,
    plan: datafusion::logical_expr::LogicalPlan,
) -> Result<datafusion::logical_expr::LogicalPlan> {
    let spec = session
        .registry()
        .relation_by_id(member.relation_id)
        .ok_or_else(|| invalid("CDF relation declaration is absent"))?;
    let column = |name: &str| Expr::Column(Column::from_name(name));
    let key = pse_relations::identity::key(
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

#[cfg(test)]
mod durability_unit {
    use super::*;
    #[test]
    fn fallback_is_limited_to_missing_history_and_supported_capability_refusals() {
        let unavailable = pse_engine::session::engine(DataFusionError::External(Box::new(
            deltalake::DeltaTableError::ChangeDataNotEnabled { version: 4 },
        )));
        assert!(cdf_unavailable(&unavailable));
        assert!(!cdf_unavailable(&pse_engine::EngineError::Cancelled));
        assert!(!cdf_unavailable(&pse_engine::EngineError::Admission {
            path: "table".into(),
            reason: "permission denied".into()
        }));
        assert!(!cdf_unavailable(&pse_engine::session::engine(
            DataFusionError::ResourcesExhausted("memory".into())
        )));
        assert!(!cdf_unavailable(&pse_engine::session::engine(
            DataFusionError::Execution("unknown I/O fault".into())
        )));
    }
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

#[cfg(test)]
mod tests;
