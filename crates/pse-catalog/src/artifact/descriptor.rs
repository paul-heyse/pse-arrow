// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit durable products and exact descriptor loading through normal owned reads.
use super::{ArtifactPlan, PublicationSelection, RelationOutput, invalid};
use datafusion::common::{ResolvedTableReference, TableReference};
use pse_columnar::{AllocationLease, CancellationToken, Leased, MemoryConsumer};
use pse_engine::EngineError;
use pse_model::HeapUsage;
use pse_model::artifact::ArtifactDescriptor;
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{artifact_descriptors as wire, publication_manifests},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

impl ArtifactPlan {
    /// Select a complete durable product and attach its immutable generated descriptor.
    /// The retained plans still own all validation/effect dependencies.
    /// # Errors
    /// Missing outputs, mismatched registry or descriptor, duplicate descriptor binding.
    pub fn with_product(
        mut self,
        descriptor: ArtifactDescriptor,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        if matches!(self.publication, PublicationSelection::Migration(_)) {
            return Err(invalid(
                "an artifact migration must retain its declared lineage publication",
            ));
        }
        let row = descriptor.row();
        if row.profile == pse_relations::generated::enums::PublicationKind::Migration {
            return Err(invalid(
                "migration products require checked transformation and lineage admission",
            ));
        }
        if self.fresh_observations
            && row.reconstruction != pse_relations::generated::enums::ArtifactReconstruction::None
        {
            return Err(invalid(
                "observed or effectful products cannot claim pure reconstruction",
            ));
        }
        if row.profile_contract
            != pse_schema::fingerprint::semantic_profile(
                self.session.registry(),
                row.profile.as_str(),
                &row.requested_relations.iter().copied().collect(),
            )
            .map_err(pse_relations::RelationError::from)?
        {
            return Err(invalid("product descriptor registry differs"));
        }
        if row
            .profile_required_relations
            .as_ref()
            .map(|values| values.iter().copied().collect::<BTreeSet<_>>())
            != self
                .session
                .registry()
                .artifact_profile(row.profile.as_str())
                .cloned()
        {
            return Err(invalid("new product descriptor profile inventory differs"));
        }
        let required = self
            .session
            .registry()
            .artifact_profile(row.profile.as_str())
            .ok_or_else(|| invalid("unknown product profile"))?;
        let mut selected: BTreeSet<_> = row.requested_relations.iter().copied().collect();
        selected.extend(required);
        selected = pse_schema::product::support_closure(self.session.registry(), &selected)
            .map_err(pse_relations::RelationError::from)?;
        selected.remove(&wire::RELATION_ID);
        let present: BTreeSet<_> = self.outputs.values().map(|v| v.relation_id).collect();
        if !selected.is_subset(&present) {
            let missing = selected
                .difference(&present)
                .filter_map(|id| self.session.registry().relation_by_id(*id))
                .map(|spec| spec.key.qualified_name())
                .collect::<Vec<_>>();
            return Err(invalid(&format!(
                "product omits required or explicitly requested relations: {}",
                missing.join(", ")
            )));
        }
        self.outputs
            .retain(|_, value| selected.contains(&value.relation_id));
        let reserve =
            MemoryConsumer::new("catalog:descriptor-encoding").register(self.session.pool());
        reserve
            .try_grow(row.owned_bytes().saturating_mul(16).saturating_add(4096))
            .map_err(pse_engine::session::engine)?;
        let mut builder = wire::Builder::with_registry(self.session.registry(), 1)?;
        builder.push(row.clone())?;
        self.session = self.session.with_checked_workspace(
            BTreeMap::from([(wire::RELATION_KEY, builder.finish()?)]),
            cancel,
        )?;
        self.session.retain_owner(AllocationLease::new(reserve));
        let source = ResolvedTableReference {
            catalog: "workspace".into(),
            schema: "runtime".into(),
            table: wire::NAME.into(),
        };
        self.outputs.insert(
            ResolvedTableReference {
                catalog: "artifact".into(),
                schema: "runtime".into(),
                table: wire::NAME.into(),
            },
            RelationOutput {
                relation_id: wire::RELATION_ID,
                plan: pse_engine::session::output::declare_relation_output(
                    self.session.relation_plan(&source)?.plan().clone(),
                    self.session.registry(),
                    wire::spec(self.session.registry())?,
                )
                .map_err(pse_engine::session::engine)?,
            },
        );
        self.publication = PublicationSelection::Product(Arc::new(descriptor));
        Ok(self)
    }

    pub(super) fn validate_product_header(
        &self,
        header: &publication_manifests::Row,
        cancel: &CancellationToken,
    ) -> Result<(), EngineError> {
        let PublicationSelection::Product(descriptor) = &self.publication else {
            return if matches!(
                (&self.publication, header.kind),
                (
                    PublicationSelection::Relations,
                    pse_relations::generated::enums::PublicationKind::Relations
                ) | (
                    PublicationSelection::Migration(_),
                    pse_relations::generated::enums::PublicationKind::Migration
                )
            ) {
                Ok(())
            } else {
                Err(invalid(
                    "complete product publication requires its exact descriptor",
                ))
            };
        };
        let row = descriptor.row();
        if row.profile != header.kind {
            return Err(invalid("publication kind differs from descriptor"));
        }
        // The entire advertised input vector must be bound, even when an output
        // does not read a member. Unused/absent inputs are still semantic dependencies.
        for selected in &row.release_members {
            let reference = ResolvedTableReference {
                catalog: selected.catalog_name.clone().into(),
                schema: selected.schema_name.clone().into(),
                table: selected.table_name.clone().into(),
            };
            if crate::selection::selected_member(&self.session, &reference)? != *selected {
                return Err(invalid(
                    "descriptor release vector differs from the bound exact input",
                ));
            }
        }
        for output in self.outputs.values() {
            for member in
                crate::selection::selected_dependencies(&self.session, &output.plan, cancel)?
            {
                if !row.release_members.contains(&member) {
                    return Err(invalid(
                        "descriptor does not cover an exact consumed release member",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl crate::delta::publication::Publication {
    /// Load and compare the complete descriptor under current read authorization.
    /// No cache miss or mismatch is allowed to resolve a different release.
    /// # Errors
    /// Missing/duplicate descriptor, unavailable bytes, or incompatible validity inputs.
    pub async fn require_artifact(
        &self,
        expected: &ArtifactDescriptor,
        cancel: &CancellationToken,
    ) -> Result<Arc<Leased<ArtifactDescriptor>>, EngineError> {
        let actual = self.artifact_descriptor(cancel).await?;
        actual
            .require_same(expected)
            .map_err(pse_relations::RelationError::from)?;
        Ok(actual)
    }
    /// Read the current format and require a complete declared durable support closure.
    /// # Errors
    /// Missing or incompatible descriptor, contract mismatch, unavailable exact data.
    pub async fn artifact_descriptor(
        &self,
        cancel: &CancellationToken,
    ) -> Result<Arc<Leased<ArtifactDescriptor>>, EngineError> {
        let members: Vec<_> = self
            .record()
            .members
            .iter()
            .filter(|m| m.relation_id == wire::RELATION_ID)
            .collect();
        let [member] = members.as_slice() else {
            return Err(invalid("artifact requires one exact descriptor member"));
        };
        let reference = TableReference::full(
            member.catalog_name.clone(),
            member.schema_name.clone(),
            member.table_name.clone(),
        )
        .resolve("", "");
        let mut stream = self.relation_stream(&reference, cancel).await?;
        let reserve =
            MemoryConsumer::new("catalog:descriptor-decoding").register(self.session().pool());
        let mut found = None;
        while let Some(batch) = stream.next_batch(cancel).await? {
            if batch.batch().num_rows() > usize::from(found.is_none()) {
                return Err(invalid("artifact descriptor member has multiple rows"));
            }
            reserve
                .try_grow(pse_columnar::algorithm_decode_extent(batch.batch())?)
                .map_err(pse_engine::session::engine)?;
            let checked = pse_relations::columnar::FieldCheckedBatch::admit_owned(
                self.session().registry(),
                wire::spec(self.session().registry())?,
                batch,
            )?;
            for row in wire::Row::rows(&checked)? {
                let descriptor =
                    ArtifactDescriptor::admit(row).map_err(pse_relations::RelationError::from)?;
                if found.replace(descriptor).is_some() {
                    return Err(invalid("artifact descriptor member has multiple rows"));
                }
            }
        }
        let actual = found.ok_or_else(|| invalid("artifact descriptor is absent"))?;
        if actual.row().profile != self.record().kind {
            return Err(invalid("artifact descriptor profile differs from control"));
        }
        let row = actual.row();
        let requested: BTreeSet<_> = row.requested_relations.iter().copied().collect();
        let required: BTreeSet<_> = match &row.profile_required_relations {
            Some(required) => required.iter().copied().collect(),
            None if row.descriptor_version == 2 => self
                .session()
                .registry()
                .artifact_profile(row.profile.as_str())
                .cloned()
                .ok_or_else(|| {
                    invalid("unknown historical profile inventory requires migration")
                })?,
            None => return Err(invalid("recorded profile inventory is absent")),
        };
        let mut roots = requested.clone();
        roots.extend(&required);
        let mut relations = BTreeMap::new();
        for recorded in self.recorded_members().values() {
            for (id, description) in &recorded.contract().relations {
                if relations.get(id).is_some_and(|old| old != description) {
                    return Err(invalid("artifact recorded support declarations contradict"));
                }
                relations.insert(*id, description.clone());
            }
        }
        // Select only the descriptor's root closure, retaining the independently verified
        // support descriptions. The original digest proves the v2 baseline fallback.
        let all_roots = relations.keys().copied().collect();
        let all = pse_schema::compatibility::VerifiedRecordedContract::from_contract(
            pse_schema::fingerprint::SemanticContract {
                version: pse_schema::fingerprint::SEMANTIC_VERSION,
                roots: all_roots,
                relations,
            },
        )
        .map_err(pse_columnar::external)
        .map_err(pse_engine::session::engine)?;
        let recorded = all
            .select_roots(&roots)
            .map_err(pse_columnar::external)
            .map_err(pse_engine::session::engine)?;
        if row.profile_contract
            != pse_schema::fingerprint::semantic_profile_recorded(
                &recorded,
                row.profile.as_str(),
                &requested,
                &required,
            )
            .map_err(pse_relations::RelationError::from)?
        {
            return Err(invalid(
                "recorded artifact profile identity cannot be proven; explicit migration required",
            ));
        }
        let present = self
            .record()
            .members
            .iter()
            .map(|member| member.relation_id)
            .collect::<BTreeSet<_>>();
        if !recorded
            .contract()
            .relations
            .keys()
            .all(|id| present.contains(id))
        {
            return Err(invalid("artifact recorded support closure is incomplete"));
        }
        let consumer =
            pse_schema::fingerprint::SemanticContract::new(self.session().registry(), &roots)
                .map_err(pse_relations::RelationError::from)?;
        recorded
            .project(&consumer)
            .map_err(pse_columnar::external)
            .map_err(pse_engine::session::engine)?;
        Ok(Arc::new(Leased::new(
            Arc::new(actual),
            AllocationLease::new(reserve),
        )))
    }
}

#[cfg(test)]
mod durability_unit;
