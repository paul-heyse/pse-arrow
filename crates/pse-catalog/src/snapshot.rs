// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A snapshot as the session sees it: a validated manifest plus the relations it names,
//! already decoded (blueprint §5.4, §20.1).
//!
//! §5.4 requires a session to pin a snapshot so that "schemas cannot change mid-query",
//! and it requires `SchemaProvider::table()` — which is `async` by signature — to perform
//! no I/O. Both follow from loading here, once, at session creation: after that a
//! [`Snapshot`] is immutable and every provider lookup is a `BTreeMap` hit.
//!
//! `BTreeMap` rather than `HashMap` because `table_names()` and `schema_names()` reach
//! output, and a hash-ordered listing is a diff that changes for no reason.

use std::collections::BTreeMap;
use std::sync::Arc;

use datafusion::arrow::array::RecordBatch;
use pse_ids::SnapshotId;

use crate::contract::RelationContract;
use crate::store::manifest::{Manifest, RelationMember};

/// Where the bytes behind a snapshot came from (blueprint §20.1).
///
/// The distinction is not paranoia, it is cost: an [`Self::Owned`] artifact was written by
/// this deployment under the single-writer protocol and needs its encoding checksum
/// verified; an [`Self::Untrusted`] one — an import or a restore — must additionally be
/// decoded, validated against the complete semantic contract and rehashed, because
/// nothing else establishes that the bytes hold the relation their name claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrustLevel {
    /// Written by this deployment's publication protocol.
    Owned,
    /// Imported or restored; the full §20.1 admission applies.
    Untrusted,
}

/// One decoded relation of a snapshot.
///
/// The batch is a single, primary-key-sorted `RecordBatch`: §5.3 step 1 orders a relation
/// by its unique primary key, and a provider that served several batches in arrival order
/// would hand a query a row order the logical hash does not describe.
#[derive(Clone, Debug)]
pub struct LoadedRelation {
    /// What the registry declares about this relation.
    pub contract: Arc<RelationContract>,
    /// The relation's complete content, in one primary-key-sorted batch.
    pub batch: RecordBatch,
    /// The manifest member this was loaded from.
    pub member: RelationMember,
}

impl LoadedRelation {
    /// The number of rows the batch holds.
    ///
    /// Read from the batch rather than the manifest so that a caller comparing the two
    /// is comparing two independent statements (§20.2 validates row counts).
    #[must_use]
    pub fn rows(&self) -> usize {
        self.batch.num_rows()
    }
}

/// A pinned, fully loaded snapshot (blueprint §5.4).
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// The validated manifest.
    pub manifest: Arc<Manifest>,
    /// Every member, keyed by `(namespace, name)`.
    pub relations: BTreeMap<(String, String), Arc<LoadedRelation>>,
}

impl Snapshot {
    /// The snapshot's membership identity, as the manifest records it.
    #[must_use]
    pub fn snapshot_id(&self) -> SnapshotId {
        self.manifest.snapshot_id
    }

    /// One relation, or `None` when the snapshot does not contain it.
    ///
    /// `None` is a complete answer, not a lookup failure: §5.3 step 7 requires a snapshot
    /// to name every member of its class explicitly, so a missing relation means the
    /// snapshot does not have one, and the caller decides whether that is an error.
    #[must_use]
    pub fn relation(&self, namespace: &str, name: &str) -> Option<&Arc<LoadedRelation>> {
        self.relations.get(&(namespace.to_owned(), name.to_owned()))
    }

    /// The namespaces present, in UTF-8 byte order.
    #[must_use]
    pub fn namespaces(&self) -> Vec<&str> {
        let mut found: Vec<&str> = self
            .relations
            .keys()
            .map(|(namespace, _)| namespace.as_str())
            .collect();
        found.dedup();
        found
    }

    /// The relation names in one namespace, in UTF-8 byte order.
    #[must_use]
    pub fn relation_names(&self, namespace: &str) -> Vec<&str> {
        self.relations
            .iter()
            .filter(|((found, _), _)| found == namespace)
            .map(|((_, name), _)| name.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap as StdBTreeMap;

    use datafusion::arrow::array::{Int64Array, StringArray};
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use pse_ids::{
        CANON_VERSION, CanonicalContract, ContentHash, EncodingChecksum, FieldPath, LogicalHash,
        SNAPSHOT_PROFILE, SchemaVersion, SemanticId, SnapshotKind, SnapshotParent,
    };

    use super::*;
    use crate::contract::EncodingPolicy;
    use crate::store::layout::{EncodingFormat, relation_path};
    use crate::store::manifest::{CompilerRef, EncodingRecord, MANIFEST_VERSION, ToolchainRef};

    fn loaded(namespace: &str, name: &str, tag: u8) -> Arc<LoadedRelation> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("ordinal", DataType::Int64, false),
        ]));
        let canonical = CanonicalContract::try_new(
            SemanticId::from_bytes([tag; 16]),
            SchemaVersion(1),
            ContentHash::from_bytes([0x11; 32]),
            Arc::clone(&schema),
            &["id"],
            &StdBTreeMap::<FieldPath, Arc<[String]>>::new(),
        );
        let Ok(canonical) = canonical else {
            panic!("the fixture schema is canonicalizable");
        };
        let contract = RelationContract::try_new(
            canonical,
            namespace,
            name,
            &["id"],
            &[],
            &[],
            EncodingPolicy::IpcFile,
        );
        let Ok(contract) = contract else {
            panic!("the fixture contract is admissible");
        };
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["a", "b"])),
                Arc::new(Int64Array::from(vec![1_i64, 2])),
            ],
        );
        let Ok(batch) = batch else {
            panic!("the fixture batch is well formed");
        };
        let version = SchemaVersion(1);
        let encoding_checksum = EncodingChecksum(ContentHash::from_bytes([tag; 32]));
        let path = relation_path(
            namespace,
            name,
            version,
            &encoding_checksum,
            EncodingFormat::ArrowIpcFile,
        );
        Arc::new(LoadedRelation {
            contract: Arc::new(contract),
            batch,
            member: RelationMember {
                port: format!("{namespace}/{name}"),
                namespace: namespace.to_owned(),
                relation_id: SemanticId::from_bytes([tag; 16]),
                name: name.to_owned(),
                version,
                logical_hash: LogicalHash(ContentHash::from_bytes([tag; 32])),
                rows: 2,
                encodings: vec![EncodingRecord {
                    format: EncodingFormat::ArrowIpcFile,
                    writer_version: "arrow-rs 59.3.0".to_owned(),
                    encoding_checksum,
                    bytes: 512,
                    path: path.as_ref().to_owned(),
                }],
            },
        })
    }

    fn snapshot() -> Snapshot {
        let mut relations: BTreeMap<(String, String), Arc<LoadedRelation>> = BTreeMap::new();
        for (namespace, name, tag) in [
            ("compiled", "math_equations", 0x21_u8),
            ("compiled", "math_expr_nodes", 0x22),
            ("authored", "units", 0x23),
        ] {
            relations.insert(
                (namespace.to_owned(), name.to_owned()),
                loaded(namespace, name, tag),
            );
        }
        let manifest = Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            snapshot_kind: SnapshotKind::Stage,
            snapshot_id: SnapshotId(ContentHash::from_bytes([0x77; 32])),
            membership_profile: SNAPSHOT_PROFILE.to_owned(),
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            schema_registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
            relations: relations
                .values()
                .map(|loaded| loaded.member.clone())
                .collect(),
            packages: Vec::new(),
            compiler: CompilerRef {
                version: "0.1.0".to_owned(),
                passes: Vec::new(),
            },
            engine_profile: None,
            numerical_policy: None,
            toolchain: ToolchainRef {
                lockfile_hash: ContentHash::from_bytes([0x61; 32]),
                canonicalization: CANON_VERSION.to_owned(),
            },
            kernels: Vec::new(),
            semantic_parents: Vec::<SnapshotParent>::new(),
            evidence: Vec::new(),
        };
        Snapshot {
            manifest: Arc::new(manifest),
            relations,
        }
    }

    #[test]
    fn a_snapshot_reports_the_identity_its_manifest_carries() {
        assert_eq!(
            snapshot().snapshot_id(),
            SnapshotId(ContentHash::from_bytes([0x77; 32]))
        );
    }

    #[test]
    fn a_relation_is_found_by_namespace_and_name() {
        let snapshot = snapshot();
        let found = snapshot.relation("compiled", "math_equations");
        assert!(found.is_some());
        assert_eq!(found.map(|relation| relation.rows()), Some(2));
        assert!(snapshot.relation("compiled", "no_such_relation").is_none());
        assert!(snapshot.relation("runtime", "math_equations").is_none());
    }

    #[test]
    fn listings_are_in_byte_order_and_not_in_insertion_order() {
        let snapshot = snapshot();
        assert_eq!(snapshot.namespaces(), vec!["authored", "compiled"]);
        assert_eq!(
            snapshot.relation_names("compiled"),
            vec!["math_equations", "math_expr_nodes"]
        );
        assert_eq!(snapshot.relation_names("authored"), vec!["units"]);
        assert!(snapshot.relation_names("runtime").is_empty());
    }

    #[test]
    fn the_trust_levels_are_distinct() {
        assert_ne!(TrustLevel::Owned, TrustLevel::Untrusted);
    }
}
