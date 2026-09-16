// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Snapshot inspection is a projection of the common provider binding inventory.

use super::{SessionFactory, SnapshotSession, snapshot_session::engine};
use crate::{
    CatalogError, Snapshot,
    provider::{
        binding::{BindingKey, TableBinding},
        table::RelationTable,
    },
};
use datafusion::{
    catalog::TableProvider,
    common::TableReference,
    datasource::provider_as_source,
    logical_expr::{LogicalPlan, LogicalPlanBuilder},
};
use pse_ids::CancellationToken;
use std::sync::Arc;

impl SessionFactory {
    /// Bind every exact snapshot member to a native port and unambiguous relation alias.
    /// Repeated relation outputs remain independently addressable through their ports.
    /// # Errors
    /// Registry mismatch, provider admission, cancellation or resource refusal.
    pub fn inspect_snapshot(
        &self,
        snapshot: &Arc<Snapshot>,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        let registry = Arc::clone(&snapshot.admission.registry);
        let mut session = self.candidate(
            std::collections::BTreeMap::new(),
            Arc::clone(&registry),
            cancel,
        )?;
        for (port, relation) in snapshot.relations() {
            cancel.checkpoint()?;
            let spec = registry
                .relation_by_id(relation.contract().canonical.relation_id)
                .ok_or_else(|| invalid("member declaration absent"))?;
            let provider: Arc<dyn TableProvider> = Arc::new(RelationTable::from_port(
                Arc::clone(snapshot),
                port,
                &registry,
            )?);
            let binding = TableBinding::new(
                TableReference::full("snapshot", "ports", port.clone()),
                provider,
                Some(spec.key),
                Some(relation.checked().clone()),
            );
            session
                .bindings
                .insert(BindingKey::SnapshotPort(port.clone()), binding.clone())
                .map_err(engine)?;
            if snapshot
                .relation(spec.key.namespace.as_str(), spec.key.name)
                .is_some()
            {
                let alias = TableBinding {
                    reference: TableReference::full(
                        "model",
                        spec.key.namespace.as_str(),
                        spec.key.name,
                    ),
                    ..binding
                };
                session
                    .bindings
                    .insert(BindingKey::Relation(spec.key), alias)
                    .map_err(engine)?;
            }
        }
        Ok(session.with_purpose(pse_schema::model::provider::OperationPurpose::Inspect))
    }
}

impl SnapshotSession {
    /// Exact relation/port inventory; this lookup performs no row work.
    pub fn inspection_tables(&self) -> Vec<(String, String)> {
        self.bindings
            .iter()
            .filter_map(|(key, binding)| {
                let BindingKey::SnapshotPort(port) = key else {
                    return None;
                };
                binding.relation.map(|relation| {
                    (
                        format!("{}.{}", relation.namespace.as_str(), relation.name),
                        port.clone(),
                    )
                })
            })
            .collect()
    }

    pub(crate) fn inspection_plan(
        &self,
        name: &str,
        port: Option<&str>,
    ) -> Result<LogicalPlan, CatalogError> {
        let TableReference::Partial { schema, table } = TableReference::parse_str(name) else {
            return Err(invalid(
                "table names require namespace and relation components",
            ));
        };
        let mut members = self.bindings.iter().filter(|(key, binding)| {
            let BindingKey::SnapshotPort(bound_port) = key else {
                return false;
            };
            port.is_none_or(|port| port == bound_port)
                && binding.relation.is_some_and(|key| {
                    key.namespace.as_str() == schema.as_ref() && key.name == table.as_ref()
                })
        });
        let (_, binding) = members
            .next()
            .ok_or_else(|| invalid("table or selected port is absent"))?;
        if members.next().is_some() {
            return Err(invalid("table is ambiguous; select an exact output port"));
        }
        let plan = LogicalPlanBuilder::scan(
            binding.reference.clone(),
            provider_as_source(Arc::clone(&binding.provider)),
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
        let spec = binding
            .relation
            .and_then(|key| self.registry.relation_by_key(key))
            .ok_or_else(|| invalid("member declaration is absent"))?;
        super::output::declare_relation_output(plan, &self.registry, spec).map_err(engine)
    }
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "provider.inspection".into(),
        reason: reason.into(),
    }
}
