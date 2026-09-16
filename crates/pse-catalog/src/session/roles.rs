// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit source roles preserve distinct immutable instances of one relation.
use super::snapshot_session::engine;
use super::{SessionFactory, SnapshotSession, candidate::CandidateTable};
use crate::{
    CatalogError,
    provider::binding::{BindingKey, TableBinding},
};
use datafusion::{
    arrow::array::RecordBatch,
    catalog::TableProvider,
    common::TableReference,
    datasource::provider_as_source,
    logical_expr::{LogicalPlan, LogicalPlanBuilder, TableSource},
};
use pse_ids::CancellationToken;
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, sync::Arc};

impl SessionFactory {
    /// Bind every input port to its own immutable provider. An unambiguous relation
    /// also gets a relation-qualified alias to that same provider; repeated schemas
    /// remain accessible only through their named roles.
    /// # Errors
    /// A role/declaration mismatch, cancellation or resource failure.
    pub fn candidate_checked_ports(
        &self,
        rows: BTreeMap<String, pse_relations::columnar::FieldCheckedBatch>,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        let mut session = self.candidate_checked_roles(rows, registry, cancel)?;
        let mut unique = BTreeMap::new();
        for (slot, binding) in session.bindings.iter() {
            if !matches!(slot, BindingKey::Input(_)) {
                continue;
            }
            cancel.checkpoint()?;
            let key = binding
                .relation
                .ok_or_else(|| invalid("input relation absent"))?;
            unique
                .entry((key.namespace, key.name))
                .and_modify(|provider| *provider = None)
                .or_insert_with(|| Some((key, binding.clone())));
        }
        for (key, mut binding) in unique.into_values().flatten() {
            binding.reference = TableReference::full("workspace", key.namespace.as_str(), key.name);
            session
                .bindings
                .insert(BindingKey::Relation(key), binding)
                .map_err(engine)?;
        }
        Ok(session)
    }
    /// Bind generated or previously admitted fields without repeating value scans.
    /// # Errors
    /// A role/declaration mismatch, cancellation or resource failure.
    pub fn candidate_checked_roles(
        &self,
        rows: BTreeMap<String, pse_relations::columnar::FieldCheckedBatch>,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        self.candidate(BTreeMap::new(), registry, cancel)?
            .with_checked_role_inputs(rows, cancel)
    }
    /// Bind physical candidates by role. PK/FK obligations remain unestablished,
    /// so diagnostics see duplicates and missing references in the actual data.
    /// # Errors
    /// Invalid physical values/fields, source roles, cancellation or configuration.
    pub fn candidate_roles(
        &self,
        rows: BTreeMap<String, (RelationKey, RecordBatch)>,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        self.candidate(BTreeMap::new(), registry, cancel)?
            .with_role_inputs(rows, cancel)
    }
}

impl SnapshotSession {
    /// Add generated fields by their declared relation while retaining immutable owners.
    /// No established local value predicate is repeated at this binding boundary.
    /// # Errors
    /// Attempted replacement, declaration mismatch, cancellation or resources.
    pub fn with_checked_workspace(
        &self,
        rows: BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        for (key, input) in rows {
            cancel.checkpoint()?;
            if result.bindings.relation(key).is_some() {
                return Err(invalid("a workspace cannot replace a pinned provider"));
            }
            let spec = result
                .registry
                .relation_by_key(key)
                .ok_or_else(|| invalid("workspace declaration is absent"))?;
            input.check_declaration(&result.registry, spec)?;
            let input = input.retained(result.reserver.as_ref(), cancel)?;
            let table: Arc<dyn TableProvider> = Arc::new(CandidateTable {
                input: input.clone(),
            });
            result
                .bindings
                .insert(
                    BindingKey::Relation(key),
                    TableBinding::new(
                        TableReference::full("workspace", key.namespace.as_str(), key.name),
                        table,
                        Some(key),
                        Some(input),
                    ),
                )
                .map_err(engine)?;
        }
        Ok(result)
    }
    /// Add independent checked inputs, including several roles for one relation.
    /// Values and Arrow owners are retained; keys/FKs remain unestablished.
    /// # Errors
    /// Empty/replaced role, mismatched declaration, cancellation or resources.
    pub fn with_checked_role_inputs(
        &self,
        rows: BTreeMap<String, pse_relations::columnar::FieldCheckedBatch>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        for (role, input) in rows {
            cancel.checkpoint()?;
            if role.is_empty() || result.bindings.input(&role).is_some() {
                return Err(invalid(&format!(
                    "input role {role:?} is empty or already bound"
                )));
            }
            let spec = result
                .registry
                .relation_by_id(input.relation_id())
                .ok_or_else(|| invalid("role input declaration is absent"))?;
            input.check_declaration(&result.registry, spec)?;
            let key = spec.key;
            let input = input.retained(result.reserver.as_ref(), cancel)?;
            let table: Arc<dyn TableProvider> = Arc::new(CandidateTable {
                input: input.clone(),
            });
            result
                .bindings
                .insert(
                    BindingKey::Input(role.clone()),
                    TableBinding::new(
                        TableReference::full("roles", "inputs", role),
                        table,
                        Some(key),
                        Some(input),
                    ),
                )
                .map_err(engine)?;
        }
        Ok(result)
    }
    /// Add immutable candidates under explicit, non-replacing role names.
    /// # Errors
    /// Duplicate/empty role, undeclared schema, raw candidate admission or resources.
    pub fn with_role_inputs(
        &self,
        rows: BTreeMap<String, (RelationKey, RecordBatch)>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        for (role, (key, batch)) in rows {
            cancel.checkpoint()?;
            if role.is_empty() || result.bindings.input(&role).is_some() {
                return Err(invalid(&format!(
                    "input role {role:?} is empty or already bound"
                )));
            }
            let spec = result
                .registry
                .relation_by_key(key)
                .ok_or_else(|| invalid("role input declaration is absent"))?;
            let batch =
                pse_ids::owned_buffer::copy_batch(&batch, result.reserver.as_ref(), cancel)?;
            let input =
                pse_relations::columnar::FieldCheckedBatch::admit(&result.registry, spec, batch)?;
            let table: Arc<dyn TableProvider> = Arc::new(CandidateTable {
                input: input.clone(),
            });
            result
                .bindings
                .insert(
                    BindingKey::Input(role.clone()),
                    TableBinding::new(
                        TableReference::full("roles", "inputs", role),
                        table,
                        Some(key),
                        Some(input),
                    ),
                )
                .map_err(engine)?;
        }
        Ok(result)
    }
    /// Actual immutable provider for a named input role.
    /// # Errors
    /// The role is absent from the frozen input inventory.
    pub fn role_source(&self, role: &str) -> Result<Arc<dyn TableSource>, CatalogError> {
        self.bindings
            .input(role)
            .map(|binding| provider_as_source(Arc::clone(&binding.provider)))
            .ok_or_else(|| invalid("input role is absent"))
    }
    /// Native scan retaining the actual role provider and its complete declared schema.
    /// # Errors
    /// Absent role or native schema binding failure.
    pub fn scan_role(&self, role: &str) -> Result<LogicalPlan, CatalogError> {
        let binding = self
            .bindings
            .input(role)
            .ok_or_else(|| invalid("input role is absent"))?;
        LogicalPlanBuilder::scan(binding.reference.clone(), self.role_source(role)?, None)
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)
    }
    /// Complete named role inventory; an empty relation still has a binding.
    pub fn input_roles(&self) -> impl Iterator<Item = (&str, RelationKey)> + '_ {
        self.bindings
            .iter()
            .filter_map(|(slot, binding)| match slot {
                BindingKey::Input(role) => binding.relation.map(|key| (role.as_str(), key)),
                _ => None,
            })
    }
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "session.input_roles".to_owned(),
        reason: reason.to_owned(),
    }
}
