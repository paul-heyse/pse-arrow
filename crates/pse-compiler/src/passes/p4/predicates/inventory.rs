// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed arguments retained from the same native projection as their source keys.
mod plans;
use super::invalid;
use crate::{
    CompilerError,
    passes::native_rows::{AlgorithmInputs, Keyed},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_mathir::NodeId;
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::{inferred as i, normalized as n},
};
use pse_schema::{Registry, model::RelationKey};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) type Origin = (RelationKey, pse_ids::ContentHash);
pub(crate) type Support = BTreeSet<Origin>;
pub(crate) type Inputs = BTreeMap<RelationKey, FieldCheckedBatch>;

pub(crate) struct Inventory<'a> {
    pub registry: &'a Registry,
    pub physical: &'a pse_quantity::QuantityRegistry,
    pub session: SnapshotSession,
    pub arguments: AlgorithmInputs,
    pub sources: Vec<Keyed<n::expression_sources::Row>>,
    pub predicates: Vec<Keyed<n::predicate_nodes::Row>>,
    pub instances: Vec<Keyed<n::instance_bindings::Row>>,
    pub indices: Vec<Keyed<n::expression_index_bindings::Row>>,
    pub domains: Vec<Keyed<n::domains::Row>>,
    pub members: Vec<Keyed<n::domain_members::Row>>,
    pub domain_bindings: Vec<Keyed<n::instance_domain_bindings::Row>>,
    pub paths: Vec<Keyed<n::expression_paths::Row>>,
    pub products: Vec<Keyed<n::domain_products::Row>>,
    pub tuples: Vec<Keyed<n::candidate_index_tuples::Row>>,
    pub submodels: Vec<Keyed<n::template_submodels::Row>>,
    pub symbols: Vec<Keyed<n::template_symbols::Row>>,
    pub parameters: Vec<Keyed<n::template_params::Row>>,
    pub features: Vec<Keyed<n::template_features::Row>>,
    pub ports: Vec<Keyed<n::template_ports::Row>>,
    pub configuration: Vec<Keyed<n::config_values::Row>>,
    pub feature_values: Vec<Keyed<i::instance_features::Row>>,
    pub node_origins: BTreeMap<(String, NodeId), Support>,
}
impl<'a> Inventory<'a> {
    pub(crate) async fn rows<T: RelationRow>(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<Vec<Keyed<T>>, CompilerError> {
        plans::rows(&mut self.arguments, &self.session, self.registry, cancel).await
    }
    pub(crate) async fn load(
        inputs: &Inputs,
        registry: &'a Registry,
        session: &SnapshotSession,
        reserver: &dyn MemoryReserver,
        cancel: &CancellationToken,
        physical: &'a crate::quantity_relations::PhysicalInventory,
    ) -> Result<Self, CompilerError> {
        let session = session.with_checked_role_inputs(
            inputs
                .iter()
                .map(|(key, input)| (key.qualified_name(), input.clone()))
                .collect(),
            cancel,
        )?;
        let mut arguments = AlgorithmInputs::new(reserver, "semantic:typed-predicate-arguments");
        let result = Self {
            sources: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            predicates: plans::available(inputs, &mut arguments, &session, registry, cancel)
                .await?,
            instances: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            indices: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            domains: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            members: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            domain_bindings: plans::available(inputs, &mut arguments, &session, registry, cancel)
                .await?,
            paths: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            products: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            tuples: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            submodels: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            symbols: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            parameters: plans::available(inputs, &mut arguments, &session, registry, cancel)
                .await?,
            features: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            ports: plans::available(inputs, &mut arguments, &session, registry, cancel).await?,
            configuration: plans::available(inputs, &mut arguments, &session, registry, cancel)
                .await?,
            feature_values: plans::available(inputs, &mut arguments, &session, registry, cancel)
                .await?,
            node_origins: plans::node_origins(inputs, &mut arguments, &session, registry, cancel)
                .await?,
            registry,
            physical: physical.quantities(),
            session,
            arguments,
        };
        Ok(result)
    }
    pub(crate) fn origin<T: RelationRow>(&self, row: &Keyed<T>) -> Result<Origin, CompilerError> {
        Ok((T::relation(self.registry)?.key, row.key))
    }
    pub(crate) fn source(
        &self,
        id: SemanticId,
    ) -> Result<&Keyed<n::expression_sources::Row>, CompilerError> {
        unique(
            &self.sources,
            |row| row.source_id == id,
            "expression source",
        )
    }
    pub(crate) fn instance(
        &self,
        id: SemanticId,
    ) -> Result<&Keyed<n::instance_bindings::Row>, CompilerError> {
        unique(
            &self.instances,
            |row| row.instance_id == id,
            "prospective instance",
        )
    }
    pub(crate) fn binder(
        &self,
        source: SemanticId,
        binder: SemanticId,
    ) -> Result<&Keyed<n::expression_index_bindings::Row>, CompilerError> {
        unique(
            &self.indices,
            |row| row.source_id == source && row.bound_index_id == binder,
            "source binder",
        )
    }
    pub(crate) fn owner(
        &self,
        instance: SemanticId,
        source: SemanticId,
    ) -> Result<&Keyed<n::instance_bindings::Row>, CompilerError> {
        let owner = self.instance(instance)?;
        let source = &self.source(source)?.row;
        if source.owner_instance_id.is_some_and(|id| id != instance)
            || source
                .owner_template_id
                .is_some_and(|id| id != owner.row.template_id)
            || (source.owner_instance_id.is_none() && source.owner_template_id.is_none())
        {
            return Err(invalid("source owner differs from the actual instance"));
        }
        Ok(owner)
    }
    pub(crate) fn domain(
        &self,
        instance: &n::instance_bindings::Row,
        domain: Option<SemanticId>,
        template: Option<SemanticId>,
        name: Option<&str>,
        support: &mut Support,
    ) -> Result<SemanticId, CompilerError> {
        if let Some(domain) = domain {
            return Ok(domain);
        }
        if template != Some(instance.template_id) {
            return Err(invalid("source domain belongs to another template"));
        }
        let name = name.ok_or_else(|| invalid("source domain name absent"))?;
        let binding = unique(
            &self.domain_bindings,
            |row| row.instance_id == instance.instance_id && row.domain_name == name,
            "instance domain binding",
        )?;
        support.insert(self.origin(binding)?);
        Ok(binding.row.domain_id)
    }
    pub(crate) fn members(
        &self,
        domain: SemanticId,
        support: &mut Support,
    ) -> Result<Vec<&Keyed<n::domain_members::Row>>, CompilerError> {
        let declaration = unique(
            &self.domains,
            |row| row.domain_id == domain,
            "finite domain",
        )?;
        support.insert(self.origin(declaration)?);
        if declaration.row.continuous {
            return Err(invalid("continuous predicate domain requires P11"));
        }
        let mut members = self
            .members
            .iter()
            .filter(|row| row.row.domain_id == domain)
            .collect::<Vec<_>>();
        members.sort_by_key(|row| row.row.ordinal);
        let mut ordinals = BTreeSet::new();
        let mut identities = BTreeSet::new();
        for member in &members {
            if !ordinals.insert(member.row.ordinal) || !identities.insert(member.row.member_id) {
                return Err(invalid("domain repeats an identity or semantic ordinal"));
            }
        }
        Ok(members)
    }
    pub(crate) fn node_support(
        &self,
        source: SemanticId,
        node: NodeId,
        support: &mut Support,
    ) -> Result<(), CompilerError> {
        let family = &self.source(source)?.row.family;
        let origins = self
            .node_origins
            .get(&(family.as_str().to_owned(), node))
            .ok_or_else(|| invalid("scalar node lacks actual native source occurrences"))?;
        support.extend(origins.iter().copied());
        Ok(())
    }
}
pub(crate) fn unique<'a, T>(
    rows: &'a [Keyed<T>],
    predicate: impl Fn(&T) -> bool,
    name: &str,
) -> Result<&'a Keyed<T>, CompilerError> {
    let mut rows = rows.iter().filter(|row| predicate(&row.row));
    let row = rows
        .next()
        .ok_or_else(|| invalid(format!("{name} exact binding absent")))?;
    if rows.next().is_some() {
        return Err(invalid(format!("{name} exact binding ambiguous")));
    }
    Ok(row)
}
