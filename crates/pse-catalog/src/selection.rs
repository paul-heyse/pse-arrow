// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact durable source selections; generic engine execution carries opaque witnesses.
use datafusion::common::{ResolvedTableReference, TableReference};
use pse_columnar::CancellationToken;
use pse_engine::{
    EngineError, EngineFactory, EngineSession,
    provider::witness::{InputWitness, SourceWitness},
};
use pse_relations::generated::runtime::publications;
use std::{collections::BTreeMap, sync::Arc};
type Member = publications::RuntimePublicationsFieldMembersItem;
#[derive(Debug)]
struct Selected(Member);
impl InputWitness for Selected {
    fn reference(&self) -> TableReference {
        TableReference::full(
            self.0.catalog_name.clone(),
            self.0.schema_name.clone(),
            self.0.table_name.clone(),
        )
    }
    fn value(&self) -> &dyn std::any::Any {
        &self.0
    }
    fn equivalent(&self, other: &dyn InputWitness) -> bool {
        other.value().downcast_ref::<Member>() == Some(&self.0)
    }
    fn descriptor(&self) -> datafusion::common::Result<Vec<u8>> {
        serde_json::to_vec(&self.0)
            .map_err(|error| datafusion::common::DataFusionError::External(Box::new(error)))
    }
}
/// Retain exact selected metadata as opaque engine evidence.
pub fn witness(member: Member) -> SourceWitness {
    SourceWitness::new(Arc::new(Selected(member)))
}
/// Resolve an admitted source's actual selected member.
/// # Errors
/// The source is absent or is not an exact publication selection.
pub fn selected_member(
    session: &EngineSession,
    reference: &ResolvedTableReference,
) -> Result<Member, EngineError> {
    session
        .source_witness(reference)?
        .value::<Member>()
        .cloned()
        .ok_or_else(|| invalid("source is not a selected publication member"))
}
/// Exact publication inputs consumed by a plan, including empty input dependencies.
/// # Errors
/// A source dependency is absent or lacks a publication selection.
pub fn selected_dependencies(
    session: &EngineSession,
    plan: &datafusion::logical_expr::LogicalPlan,
    cancel: &CancellationToken,
) -> Result<Vec<Member>, EngineError> {
    session
        .source_dependencies(plan, cancel)?
        .iter()
        .map(|w| {
            w.value::<Member>()
                .cloned()
                .ok_or_else(|| invalid("dependency is not a selected publication member"))
        })
        .collect()
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "publication.selection".into(),
        reason: reason.into(),
    }
}
fn engine(error: datafusion::common::DataFusionError) -> EngineError {
    pse_engine::session::engine(error)
}
fn table_reference(reference: &ResolvedTableReference) -> TableReference {
    TableReference::full(
        reference.catalog.clone(),
        reference.schema.clone(),
        reference.table.clone(),
    )
}
pub(crate) async fn bind_publication(
    record: &publications::Row,
    state: &datafusion::execution::session_state::SessionState,
    registry: Arc<pse_schema::Registry>,
    factory: &EngineFactory,
    cancel: &CancellationToken,
) -> Result<EngineSession, EngineError> {
    use pse_engine::provider::binding::{BindingKey, TableBinding};
    let mut session = factory.candidate(BTreeMap::new(), registry, cancel)?;
    let mut unique = BTreeMap::new();
    for member in &record.members {
        cancel.checkpoint()?;
        let reference = ResolvedTableReference {
            catalog: member.catalog_name.clone().into(),
            schema: member.schema_name.clone().into(),
            table: member.table_name.clone().into(),
        };
        let provider = state
            .catalog_list()
            .catalog(&reference.catalog)
            .and_then(|catalog| catalog.schema(&reference.schema))
            .ok_or_else(|| invalid("selected member namespace is absent"))?
            .table(&reference.table)
            .await
            .map_err(engine)?
            .ok_or_else(|| invalid("selected member provider is absent"))?;
        let spec = session
            .registry()
            .relation_by_id(member.relation_id)
            .filter(|spec| {
                i64::from(spec.key.version) == member.relation_version
                    && spec.fingerprint == member.contract_fingerprint
            })
            .ok_or_else(|| invalid("selected member declaration differs"))?;
        let schema = pse_schema::arrow::relation_schema(session.registry(), spec)
            .map_err(pse_relations::RelationError::from)?;
        if provider.schema().fields() != schema.fields() {
            return Err(invalid(
                "selected Delta fields differ from their declaration",
            ));
        }
        let reference = table_reference(&reference);
        let mut binding = TableBinding::new(reference.clone(), provider, Some(spec.key), None);
        binding.dependencies =
            pse_engine::session::facts::view_dependencies(&binding.provider).map_err(engine)?;
        binding.witness = Some(witness(member.clone()));
        // Repeated declarations in separate roles remain fully qualified. An
        // unambiguous declaration also supports generated relation-key consumers.
        unique
            .entry(spec.key)
            .and_modify(|value| *value = None)
            .or_insert_with(|| Some(binding.clone()));
        session
            .bind_source(BindingKey::Native(reference), binding)
            .map_err(engine)?;
    }
    for (key, binding) in unique
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key, value)))
    {
        session
            .bind_source(BindingKey::Relation(key), binding)
            .map_err(engine)?;
    }
    Ok(session)
}
