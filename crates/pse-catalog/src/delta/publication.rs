// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact Delta publication opening. The catalog selects a publication's complete record
//! (Plan 22 X9-X12); this crate opens exactly its member versions.
use datafusion::{
    catalog::{
        CatalogProvider, CatalogProviderList, MemoryCatalogProvider, MemoryCatalogProviderList,
        MemorySchemaProvider, SchemaProvider,
    },
    common::{DataFusionError, Result, ScalarValue},
    datasource::ViewTable,
    execution::session_state::{SessionState, SessionStateBuilder},
    logical_expr::{LogicalPlanBuilder, lit},
};
use futures_util::{FutureExt, StreamExt, TryStreamExt};
use pse_relations::generated::runtime::publication_manifests;
use pse_schema::Registry;
use std::{collections::BTreeMap, sync::Arc};
type RecordedMembers = BTreeMap<
    datafusion::common::ResolvedTableReference,
    pse_schema::compatibility::VerifiedRecordedContract,
>;

/// What a reader selected: the complete publication record the catalog granted, the
/// catalog read scope of that grant, and the owner that keeps the grant alive (a reader
/// lease guard). An offline reader of an export manifest has the record and the scope
/// the export recorded, and no owner.
#[derive(Clone, Debug)]
pub struct PublicationSelection {
    /// The complete record: exact member and input versions.
    pub record: publication_manifests::Row,
    /// The catalog read scope; caches are keyed on it and bypassed without one.
    pub scope: Option<super::scope::ReadScope>,
    /// Retained for as long as any session or stream of the publication lives.
    pub owner: Option<Arc<dyn pse_engine::provider::witness::ExecutionOwner>>,
}

/// Exact selected publication and its admitted native execution environment.
/// Reads retain policies, requirements, cancellation and Arrow ownership through
/// the same boundary as SQL and compiler plans.
#[derive(Debug)]
pub struct Publication {
    record: publication_manifests::Row,
    session: pse_engine::session::EngineSession,
    recorded: RecordedMembers,
}

#[derive(Clone, Copy)]
enum Consumption {
    Current,
    Recorded,
}

impl Publication {
    /// Open exactly the member versions a catalog selection names, under the caller's
    /// native policy. Relation payloads remain lazy; opening verifies declarations and
    /// selections. The selection's owner is retained by the session.
    /// # Errors
    /// Missing versions, incompatible contracts, policy refusal or cancellation.
    pub async fn open(
        selection: PublicationSelection,
        registry: Arc<Registry>,
        factory: &pse_engine::session::EngineFactory,
        cancel: &pse_columnar::CancellationToken,
    ) -> std::result::Result<Self, crate::EngineError> {
        Self::open_consuming(selection, registry, factory, cancel).await
    }
    async fn open_consuming(
        selection: PublicationSelection,
        registry: Arc<Registry>,
        factory: &pse_engine::session::EngineFactory,
        cancel: &pse_columnar::CancellationToken,
    ) -> std::result::Result<Self, crate::EngineError> {
        super::admission::admit_profile(&selection.record, &registry)
            .map_err(pse_engine::session::engine)?;
        let publication =
            Self::open_interpreted(selection, registry, factory, cancel, Consumption::Current)
                .await?;
        if !matches!(
            publication.record.kind,
            pse_relations::generated::enums::PublicationKind::Relations
                | pse_relations::generated::enums::PublicationKind::Migration
        ) {
            publication.artifact_descriptor(cancel).await?;
        }
        Ok(publication)
    }
    /// Open verified recorded meaning for explicit transformation without current consumer admission.
    /// Storage selection and portable checks use the same interpreter as ordinary reads.
    /// # Errors
    /// Missing recorded proof, contradictory selections, policy refusal or cancellation.
    pub async fn open_recorded(
        selection: PublicationSelection,
        registry: Arc<Registry>,
        factory: &pse_engine::session::EngineFactory,
        cancel: &pse_columnar::CancellationToken,
    ) -> std::result::Result<Self, crate::EngineError> {
        Self::open_interpreted(selection, registry, factory, cancel, Consumption::Recorded).await
    }
    async fn open_interpreted(
        selection: PublicationSelection,
        registry: Arc<Registry>,
        factory: &pse_engine::session::EngineFactory,
        cancel: &pse_columnar::CancellationToken,
        consumption: Consumption,
    ) -> std::result::Result<Self, crate::EngineError> {
        cancel.checkpoint()?;
        let PublicationSelection {
            record,
            scope,
            owner,
        } = selection;
        let mut state = factory.native_state().clone();
        state.config_mut().set_extension(Arc::new(
            pse_engine::session::execution::AttemptScope::default(),
        ));
        if let Some(scope) = scope {
            scope.install(state.config_mut());
        }
        let consumer = match consumption {
            Consumption::Current => Some(registry.as_ref()),
            Consumption::Recorded => None,
        };
        let (state, recorded) = cancel
            .until_cancelled(bind_interpreted_members(
                &record.members,
                consumer,
                Arc::new(state),
            ))
            .await?
            .map_err(pse_engine::session::engine)?;
        let mut session = match consumption {
            Consumption::Current => {
                crate::selection::bind_publication(
                    &record.members,
                    &state,
                    registry,
                    factory,
                    cancel,
                )
                .await?
            }
            Consumption::Recorded => {
                crate::selection::bind_recorded_publication(
                    &record.members,
                    &state,
                    registry,
                    factory,
                    cancel,
                )
                .await?
            }
        };
        if let Some(owner) = owner {
            session.retain_owner(owner);
        }
        Ok(Self {
            record,
            session,
            recorded,
        })
    }
    /// Verified original member meaning, independent of a consumer projection.
    /// # Errors
    /// The exact qualified member is absent.
    pub fn recorded_member(
        &self,
        reference: &datafusion::common::ResolvedTableReference,
    ) -> std::result::Result<&pse_schema::compatibility::VerifiedRecordedContract, crate::EngineError>
    {
        self.recorded
            .get(reference)
            .ok_or_else(|| pse_engine::session::engine(invalid("recorded member is absent")))
    }
    pub(crate) fn recorded_members(&self) -> &RecordedMembers {
        &self.recorded
    }
    /// The complete publication record; no parallel manifest is retained.
    pub fn record(&self) -> &publication_manifests::Row {
        &self.record
    }
    /// Actual immutable execution environment over the selected native hierarchy.
    pub fn session(&self) -> &pse_engine::session::EngineSession {
        &self.session
    }
    /// Move the selected provider owners into an invocation without retaining this handle.
    pub fn into_session(self) -> pse_engine::session::EngineSession {
        self.session
    }
    /// Exact generated member the publication selects.
    /// # Errors
    /// The qualified name is outside this publication.
    pub fn member(
        &self,
        reference: &datafusion::common::ResolvedTableReference,
    ) -> std::result::Result<
        pse_relations::generated::structures::MemberDescriptor,
        crate::EngineError,
    > {
        crate::selection::selected_member(&self.session, reference)
    }
    /// Begin an owned native relation stream with common admission and requirements.
    /// Dropping the publication does not invalidate its stream or exported batches.
    /// # Errors
    /// Missing selection, policy refusal, planning, resource or cancellation failure.
    pub async fn relation_stream(
        &self,
        reference: &datafusion::common::ResolvedTableReference,
        cancel: &pse_columnar::CancellationToken,
    ) -> std::result::Result<pse_engine::session::OwnedComputationStream, crate::EngineError> {
        self.member(reference)?;
        let table = datafusion::common::TableReference::full(
            reference.catalog.clone(),
            reference.schema.clone(),
            reference.table.clone(),
        );
        let binding = self
            .session
            .bindings()
            .iter()
            .find_map(|(_, binding)| (binding.reference == table).then_some(binding))
            .ok_or_else(|| {
                pse_engine::session::engine(invalid("selected source binding is absent"))
            })?;
        if binding.relation.is_some() {
            return self.session.relation_stream(reference, cancel).await;
        }
        let plan = LogicalPlanBuilder::scan(
            table,
            datafusion::datasource::provider_as_source(binding.provider.clone()),
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(pse_engine::session::engine)?;
        self.session
            .prepare_rule_plan(plan, cancel)?
            .execute_stream(cancel)
            .await
    }
}
pub(super) async fn bind_members(
    members: &[pse_relations::generated::structures::MemberDescriptor],
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<Arc<SessionState>> {
    let (state, _) = bind_interpreted_members(members, Some(registry), state).await?;
    Ok(state)
}
async fn bind_interpreted_members(
    members: &[pse_relations::generated::structures::MemberDescriptor],
    registry: Option<&Registry>,
    state: Arc<SessionState>,
) -> Result<(Arc<SessionState>, RecordedMembers)> {
    let catalogs = Arc::new(MemoryCatalogProviderList::new());
    let limit = state
        .config()
        .get_extension::<pse_engine::cache_service::NativeCacheService>()
        .map_or(1, |service| service.policy().concurrent_loads.get());
    let slots = pse_columnar::MemoryConsumer::new("publication:member-open-slots")
        .register(&state.runtime_env().memory_pool);
    slots.try_grow(
        members
            .len()
            .checked_mul(256)
            .ok_or_else(|| invalid("member open inventory overflows"))?,
    )?;
    let opens = futures_util::stream::iter(0..members.len()).map(|ordinal| {
        let member = &members[ordinal];
        let state = state.clone();
        async move {
            let (view, recorded) = interpreted_provider(member, registry, state)
                .await
                .map_err(|error| {
                    error.context(format!(
                        "open publication member {}.{}",
                        member.schema_name, member.table_name
                    ))
                })?;
            Ok::<_, DataFusionError>((ordinal, view, recorded))
        }
        .boxed()
    });
    let mut opened = opens
        .buffer_unordered(limit)
        .try_collect::<Vec<_>>()
        .await?;
    opened.sort_unstable_by_key(|(ordinal, _, _)| *ordinal);
    let mut witnesses = BTreeMap::new();
    for (ordinal, view, recorded) in opened {
        let member = &members[ordinal];
        let reference = datafusion::common::ResolvedTableReference {
            catalog: member.catalog_name.clone().into(),
            schema: member.schema_name.clone().into(),
            table: member.table_name.clone().into(),
        };
        witnesses.insert(reference, recorded);
        let catalog = if let Some(catalog) = catalogs.catalog(&member.catalog_name) {
            catalog
        } else {
            let catalog: Arc<dyn CatalogProvider> = Arc::new(MemoryCatalogProvider::new());
            catalogs.register_catalog(member.catalog_name.clone(), Arc::clone(&catalog));
            catalog
        };
        let schema = if let Some(schema) = catalog.schema(&member.schema_name) {
            schema
        } else {
            let schema: Arc<dyn SchemaProvider> = Arc::new(MemorySchemaProvider::new());
            catalog.register_schema(&member.schema_name, Arc::clone(&schema))?;
            schema
        };
        if schema
            .register_table(member.table_name.clone(), view)?
            .is_some()
        {
            return Err(invalid(
                "publication has duplicate qualified table bindings",
            ));
        }
    }
    let state = Arc::new(
        SessionStateBuilder::new_from_existing(state.as_ref().clone())
            .with_catalog_list(catalogs)
            .build(),
    );
    verify_recorded_inventory(members, &witnesses)?;
    Ok((state, witnesses))
}

/// Open exactly the declared member slice through the common native provider.
pub(crate) async fn selected_provider(
    member: &pse_relations::generated::structures::MemberDescriptor,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<Arc<dyn datafusion::catalog::TableProvider>> {
    let (provider, _) = interpreted_provider(member, Some(registry), state).await?;
    Ok(provider)
}
async fn interpreted_provider(
    member: &pse_relations::generated::structures::MemberDescriptor,
    registry: Option<&Registry>,
    state: Arc<SessionState>,
) -> Result<(
    Arc<dyn datafusion::catalog::TableProvider>,
    pse_schema::compatibility::VerifiedRecordedContract,
)> {
    let contract = registry
        .map(|registry| {
            registry
                .relation_by_id(member.relation_id)
                .ok_or_else(|| invalid("consumer references an unknown relation"))?;
            super::contract::DeclaredCheck::new(registry, member.relation_id)
        })
        .transpose()?;
    let location =
        url::Url::parse(&member.table_uri).map_err(|e| DataFusionError::External(Box::new(e)))?;
    let (view, recorded) = super::provider::open_recorded_view(
        location,
        member.delta_version,
        contract.as_ref(),
        Some(member),
        Arc::clone(&state),
    )
    .await?;
    let view = match member
        .selection
        .selected()
        .map_err(pse_columnar::external)?
    {
        pse_relations::generated::structures::MemberDescriptorSelectionSelected::Full => view,
        pse_relations::generated::structures::MemberDescriptorSelectionSelected::Revision(
            selection,
        ) => {
            let schema = view.logical_plan().schema().as_arrow();
            let column = schema
                .field_with_name(&selection.column)
                .map_err(|_| invalid("revision selection column is absent"))?;
            if column
                .metadata()
                .get(pse_schema::arrow::KEY_EXTENSION_NAME)
                .map(String::as_str)
                != Some("pse.semantic_id")
            {
                return Err(invalid(
                    "revision selection column is not a semantic identity",
                ));
            }
            let value =
                ScalarValue::FixedSizeBinary(16, Some(selection.revision_id.as_bytes().to_vec()));
            let plan = LogicalPlanBuilder::from(view.logical_plan().clone())
                .filter(
                    datafusion::logical_expr::Expr::Column(datafusion::common::Column::from_name(
                        &selection.column,
                    ))
                    .eq(lit(value)),
                )?
                .build()?;
            ViewTable::new(plan, None)
        }
    };
    let provider = crate::cache_service::resident::selected(
        super::provider::selected_view(view),
        member,
        &state,
    )?;
    let provider: Arc<dyn datafusion::catalog::TableProvider> = if registry.is_none() {
        Arc::new(pse_engine::provider::recorded::RecordedProvider::new(
            provider,
            &recorded,
            member.relation_id,
        )?)
    } else {
        provider
    };
    Ok((provider, recorded))
}

fn verify_recorded_inventory(
    members: &[pse_relations::generated::structures::MemberDescriptor],
    witnesses: &RecordedMembers,
) -> Result<()> {
    // A recorded closure must agree with the actual selected roots in the same catalog.
    for (reference, witness) in witnesses {
        for (id, description) in &witness.contract().relations {
            let Some(selected) = members
                .iter()
                .find(|m| m.catalog_name == reference.catalog.as_ref() && m.relation_id == *id)
            else {
                continue;
            };
            let target = datafusion::common::ResolvedTableReference {
                catalog: selected.catalog_name.clone().into(),
                schema: selected.schema_name.clone().into(),
                table: selected.table_name.clone().into(),
            };
            if witnesses
                .get(&target)
                .and_then(|w| w.contract().relations.get(id))
                != Some(description)
            {
                return Err(invalid(
                    "selected member contradicts recorded support declaration",
                ));
            }
        }
    }
    Ok(())
}

/// Every exact provenance input opens under its verified recorded contract.
pub(super) async fn verify_inputs(
    inputs: &[pse_relations::generated::structures::MemberDescriptor],
    state: Arc<SessionState>,
) -> Result<()> {
    // A migration can consume a recorded predecessor absent from the current
    // registry. Candidate outputs still receive current consumer admission.
    bind_interpreted_members(inputs, None, state).await?;
    Ok(())
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
