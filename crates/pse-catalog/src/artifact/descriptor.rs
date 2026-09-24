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
    generated::runtime::{artifact_descriptors as wire, publications},
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
        let row = descriptor.row();
        if self.fresh_observations
            && row.reconstruction != pse_relations::generated::enums::ArtifactReconstruction::None
        {
            return Err(invalid(
                "observed or effectful products cannot claim pure reconstruction",
            ));
        }
        if row.profile_contract != self.session.registry().fingerprint() {
            return Err(invalid("product descriptor registry differs"));
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
            return Err(invalid(
                "product omits a required or explicitly requested relation",
            ));
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
        header: &publications::Row,
        cancel: &CancellationToken,
    ) -> Result<(), EngineError> {
        let PublicationSelection::Product(descriptor) = &self.publication else {
            return if header.kind == pse_relations::generated::enums::PublicationKind::Relations {
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
            let member = crate::selection::selected_member(&self.session, &reference)?;
            if selected.relation_id != member.relation_id
                || selected.relation_version != member.relation_version
                || selected.contract_fingerprint != member.contract_fingerprint
                || selected.table_uri != member.table_uri
                || selected.delta_version != member.delta_version
                || selected.selection.kind != member.selection.kind
                || selected
                    .selection
                    .revision
                    .as_ref()
                    .map(|v| (&v.column, v.revision_id))
                    != member
                        .selection
                        .revision
                        .as_ref()
                        .map(|v| (&v.column, v.revision_id))
            {
                return Err(invalid(
                    "descriptor release vector differs from the bound exact input",
                ));
            }
        }
        for output in self.outputs.values() {
            for member in
                crate::selection::selected_dependencies(&self.session, &output.plan, cancel)?
            {
                if !row.release_members.iter().any(|selected| {
                    selected.catalog_name == member.catalog_name
                        && selected.schema_name == member.schema_name
                        && selected.table_name == member.table_name
                        && selected.relation_id == member.relation_id
                        && selected.relation_version == member.relation_version
                        && selected.contract_fingerprint == member.contract_fingerprint
                        && selected.table_uri == member.table_uri
                        && selected.delta_version == member.delta_version
                        && selected.selection.kind == member.selection.kind
                        && selected
                            .selection
                            .revision
                            .as_ref()
                            .map(|r| (&r.column, r.revision_id))
                            == member
                                .selection
                                .revision
                                .as_ref()
                                .map(|r| (&r.column, r.revision_id))
                }) {
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
        if row.profile_contract != self.session().registry().fingerprint() {
            return Err(invalid("artifact registry contract is incompatible"));
        }
        let mut roots: BTreeSet<_> = row.requested_relations.iter().copied().collect();
        roots.extend(
            self.session()
                .registry()
                .artifact_profile(row.profile.as_str())
                .ok_or_else(|| invalid("unknown artifact profile"))?,
        );
        let required = pse_schema::product::support_closure(self.session().registry(), &roots)
            .map_err(pse_relations::RelationError::from)?;
        let present = self
            .record()
            .members
            .iter()
            .map(|member| member.relation_id)
            .collect::<BTreeSet<_>>();
        if !required.is_subset(&present) {
            return Err(invalid("artifact support closure is incomplete"));
        }
        Ok(Arc::new(Leased::new(
            Arc::new(actual),
            AllocationLease::new(reserve),
        )))
    }
}

#[cfg(test)]
mod durability_unit;
