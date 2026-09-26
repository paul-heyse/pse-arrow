// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Current-format artifact admission. Hashes index complete immutable descriptors.
use crate::{ModelError, SemanticFrame, generated::runtime::artifact_descriptors as wire};
use pse_ids::{ContentHash, FramedHasher};
use std::collections::BTreeSet;

/// A completely checked descriptor. Payload loading must use its exact selections.
#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactDescriptor(wire::Row);
impl ArtifactDescriptor {
    /// Canonicalize a newly produced descriptor and assign its framed identity.
    /// # Errors
    /// Unknown format, malformed members, duplicate roots/assumptions or empty product.
    pub fn create(mut row: wire::Row) -> Result<Self, ModelError> {
        row.requested_relations.sort_unstable();
        row.release_members.sort_by(|a, b| {
            (&a.catalog_name, &a.schema_name, &a.table_name).cmp(&(
                &b.catalog_name,
                &b.schema_name,
                &b.table_name,
            ))
        });
        row.value_assumptions.sort_by(|a, b| a.name.cmp(&b.name));
        row.artifact_id = identity(&row);
        Self::admit(row)
    }
    /// Read the current format without migration, normalization or latest lookup.
    /// # Errors
    /// Invalid or noncanonical complete validity descriptor.
    pub fn admit(row: wire::Row) -> Result<Self, ModelError> {
        if row.descriptor_version != 2 {
            return Err(ModelError::MigrationRequired {
                version: row.descriptor_version,
                supported: 2,
            });
        }
        if (row.profile == crate::generated::enums::PublicationKind::Run
            && row.reconstruction != crate::generated::enums::ArtifactReconstruction::None)
            || row.requested_relations.is_empty()
            || !row.requested_relations.windows(2).all(|w| w[0] < w[1])
            || row.artifact_id != identity(&row)
            || !row
                .value_assumptions
                .windows(2)
                .all(|w| w[0].name < w[1].name)
            || row.value_assumptions.iter().any(|v| v.name.is_empty())
        {
            return Err(crate::malformed(
                "incompatible or noncanonical artifact descriptor",
            ));
        }
        let mut roles = BTreeSet::new();
        let mut previous = None;
        for member in &row.release_members {
            let role = (
                &member.catalog_name,
                &member.schema_name,
                &member.table_name,
            );
            if member.delta_version < 0
                || member.relation_version <= 0
                || member.table_uri.is_empty()
                || member.catalog_name.is_empty()
                || member.schema_name.is_empty()
                || member.table_name.is_empty()
                || previous.is_some_and(|old| old >= role)
                || !roles.insert(role)
            {
                return Err(crate::malformed("invalid exact artifact member vector"));
            }
            member.selection.selected()?;
            previous = Some(role);
        }
        Ok(Self(row))
    }
    /// Complete registry-generated values; no mutable access can retarget a payload.
    pub fn row(&self) -> &wire::Row {
        &self.0
    }
    /// Compare all validity inputs, not just a digest supplied by a cache or caller.
    /// # Errors
    /// Any source, implementation, target, profile or value assumption differs.
    pub fn require_same(&self, expected: &Self) -> Result<(), ModelError> {
        if self == expected {
            Ok(())
        } else {
            Err(crate::malformed(
                "artifact validity differs from the requested exact descriptor",
            ))
        }
    }
}
fn identity(row: &wire::Row) -> ContentHash {
    let mut hash = FramedHasher::new("pse:artifact-descriptor:v1");
    row.descriptor_version.frame(&mut hash);
    row.profile.frame(&mut hash);
    row.profile_contract.frame(&mut hash);
    row.requested_relations.frame(&mut hash);
    row.release_id.frame(&mut hash);
    row.release_members.frame(&mut hash);
    row.semantic_identity.frame(&mut hash);
    row.implementation.frame(&mut hash);
    row.target_contract.frame(&mut hash);
    row.value_assumptions.frame(&mut hash);
    row.reconstruction.frame(&mut hash);
    hash.finish_hash()
}

#[cfg(test)]
mod durability_unit {
    use super::*;
    use crate::generated::enums::{ArtifactReconstruction, PublicationKind};
    fn row() -> wire::Row {
        let hash = ContentHash::from_bytes([1; 32]);
        wire::Row {
            artifact_id: hash,
            descriptor_version: 2,
            profile: PublicationKind::Model,
            profile_contract: hash,
            requested_relations: vec![pse_ids::SemanticId::from_bytes([2; 16])],
            release_id: hash,
            release_members: vec![],
            semantic_identity: hash,
            implementation: wire::RuntimeArtifactDescriptorsFieldImplementation {
                source: hash,
                build: hash,
                registry: hash,
                algorithms: hash,
            },
            target_contract: hash,
            value_assumptions: vec![],
            reconstruction: ArtifactReconstruction::ExactRelease,
        }
    }
    #[test]
    fn full_descriptor_comparison_refuses_changed_validity_and_old_formats() {
        let original = ArtifactDescriptor::create(row()).unwrap();
        assert!(ArtifactDescriptor::admit(original.row().clone()).is_ok());
        for field in 0..5 {
            let mut changed = row();
            let hash = ContentHash::from_bytes([3; 32]);
            match field {
                0 => changed.implementation.build = hash,
                1 => changed.implementation.algorithms = hash,
                2 => changed.target_contract = hash,
                3 => changed.release_id = hash,
                _ => changed.semantic_identity = hash,
            }
            let other = ArtifactDescriptor::create(changed).unwrap();
            assert!(original.require_same(&other).is_err());
        }
        let mut old = original.row().clone();
        old.descriptor_version = 0;
        assert!(ArtifactDescriptor::admit(old).is_err());
        let mut forged = original.row().clone();
        forged.semantic_identity = ContentHash::from_bytes([9; 32]);
        assert!(ArtifactDescriptor::admit(forged).is_err());
        let mut duplicate = row();
        duplicate
            .requested_relations
            .push(duplicate.requested_relations[0]);
        assert!(ArtifactDescriptor::create(duplicate).is_err());
    }
}
