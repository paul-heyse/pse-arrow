// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A snapshot as the session sees it: a validated manifest plus the relations it names,
//! already decoded and semantically admitted (blueprint §5.4, §20.1).
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
use pse_ids::{EncodingChecksum, SnapshotId};

use crate::contract::RelationContract;
use crate::store::manifest::{Manifest, RelationMember};

/// Immutable admitted manifest metadata; detached clones retain their shared allocation.
pub type OwnedManifest = crate::store::control::OwnedControl<Manifest>;

/// Where the bytes behind a snapshot came from (blueprint §20.1).
///
/// This records provenance. Both paths currently decode and validate actual content,
/// complete membership and registered semantics before comparing their identities;
/// ownership provenance does not bypass admission.
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
    pub(crate) contract: Arc<RelationContract>,
    /// The relation's complete content, in one primary-key-sorted batch.
    pub(crate) batch: pse_relations::columnar::FieldCheckedBatch,
    /// The manifest member this was loaded from.
    pub(crate) member: crate::store::control::OwnedControl<RelationMember>,
}

impl LoadedRelation {
    /// The registry-bound contract whose rows passed catalog admission.
    pub fn contract(&self) -> &Arc<RelationContract> {
        &self.contract
    }
    /// The admitted, sorted rows. Buffer clones retain the reservation owner.
    pub fn batch(&self) -> &RecordBatch {
        self.batch.batch()
    }
    /// The retained field construction/admission result, without another value scan.
    pub fn checked(&self) -> &pse_relations::columnar::FieldCheckedBatch {
        &self.batch
    }
    /// The independently checked manifest claims for this relation.
    pub fn member(&self) -> &RelationMember {
        &self.member
    }
    /// The number of rows the batch holds.
    ///
    /// Read from the batch rather than the manifest so that a caller comparing the two
    /// is comparing two independent statements (§20.2 validates row counts).
    #[must_use]
    pub fn rows(&self) -> usize {
        self.batch().num_rows()
    }
}

/// The exact immutable manifest, independent of its logical membership identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManifestRef {
    /// Logical membership identity, verified after semantic admission.
    pub snapshot_id: SnapshotId,
    /// Exact encoded manifest to reopen for reproducibility.
    pub manifest_checksum: EncodingChecksum,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestRefWire {
    snapshot_id: pse_ids::ContentHash,
    manifest_checksum: pse_ids::ContentHash,
}
impl serde::Serialize for ManifestRef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ManifestRefWire {
            snapshot_id: self.snapshot_id.0,
            manifest_checksum: self.manifest_checksum.0,
        }
        .serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for ManifestRef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = ManifestRefWire::deserialize(deserializer)?;
        Ok(Self {
            snapshot_id: SnapshotId(value.snapshot_id),
            manifest_checksum: EncodingChecksum(value.manifest_checksum),
        })
    }
}

/// A pinned, fully loaded snapshot (blueprint §5.4).
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// The validated manifest.
    pub(crate) manifest: OwnedManifest,
    /// Every complete member, keyed by its actual declared manifest/output port.
    pub(crate) relations: Arc<BTreeMap<String, Arc<LoadedRelation>>>,
    /// Exact immutable manifest pinned at admission.
    pub(crate) manifest_ref: ManifestRef,
    /// Admitted semantic context retained for contextual validation and reproduction.
    pub(crate) parents: Arc<BTreeMap<String, Arc<Snapshot>>>,
    /// Catalog admission provenance, compared by allocation identity, never a digest.
    pub(crate) admission: Arc<crate::store::open::CatalogContext>,
    /// Registered producer checked during stage admission.
    pub(crate) stage_pass: Option<pse_ids::SemanticId>,
    /// Actual auxiliary producer inputs admitted with this snapshot.
    pub(crate) invocation: Option<crate::store::invocation::OwnedInvocation>,
}

impl Snapshot {
    /// Exact selected policy owners and engine settings used by the producer.
    pub fn invocation(&self) -> Option<&crate::store::invocation::OwnedInvocation> {
        self.invocation.as_ref()
    }
    /// The exact manifest object that produced this admitted handle.
    pub const fn manifest_ref(&self) -> ManifestRef {
        self.manifest_ref
    }
    /// Manifest claims checked against the decoded relations.
    pub fn manifest(&self) -> &OwnedManifest {
        &self.manifest
    }
    /// The complete admitted relation set, keyed by exact manifest/output port.
    pub fn relations(&self) -> &BTreeMap<String, Arc<LoadedRelation>> {
        &self.relations
    }
    /// Semantic parent handles, with their exact manifest references retained.
    pub fn parents(&self) -> &BTreeMap<String, Arc<Snapshot>> {
        &self.parents
    }
    /// The exact registered stage producer, if this is a stage snapshot.
    pub const fn stage_pass(&self) -> Option<pse_ids::SemanticId> {
        self.stage_pass
    }
    /// The snapshot's membership identity, as the manifest records it.
    #[must_use]
    pub fn snapshot_id(&self) -> SnapshotId {
        self.manifest.snapshot_id
    }

    /// One uniquely named relation, or `None` when absent or present under several ports.
    /// Stage consumers must use [`Self::relation_port`] when a producer has repeated
    /// output schemas; relation names cannot choose between distinct actual outputs.
    #[must_use]
    pub fn relation(&self, namespace: &str, name: &str) -> Option<&Arc<LoadedRelation>> {
        let mut matches = self.relations.values().filter(|relation| {
            relation.member.namespace == namespace && relation.member.name == name
        });
        let found = matches.next()?;
        matches.next().is_none().then_some(found)
    }

    /// One exact declared output port, retaining its distinct complete relation values.
    pub fn relation_port(&self, port: &str) -> Option<&Arc<LoadedRelation>> {
        self.relations.get(port)
    }

    /// The namespaces present, in UTF-8 byte order.
    #[must_use]
    pub fn namespaces(&self) -> Vec<&str> {
        let mut found: Vec<&str> = self
            .relations
            .values()
            .map(|relation| relation.member.namespace.as_str())
            .collect();
        found.sort_unstable();
        found.dedup();
        found
    }

    /// The relation names in one namespace, in UTF-8 byte order.
    #[must_use]
    pub fn relation_names(&self, namespace: &str) -> Vec<&str> {
        let mut names: Vec<_> = self
            .relations
            .values()
            .filter(|relation| relation.member.namespace == namespace)
            .map(|relation| relation.member.name.as_str())
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }
}

#[cfg(test)]
mod tests {
    use datafusion::arrow::array::{Int64Array, StringArray};
    use pse_ids::{
        CANON_VERSION, ContentHash, EncodingChecksum, LogicalHash, SNAPSHOT_PROFILE, SchemaVersion,
        SemanticId, SnapshotKind, SnapshotParent,
    };

    use super::*;
    use crate::contract::EncodingPolicy;
    use crate::store::layout::{EncodingFormat, relation_path};
    use crate::store::manifest::{CompilerRef, EncodingRecord, MANIFEST_VERSION, ToolchainRef};

    fn owned<T>(value: T) -> crate::store::control::OwnedControl<T> {
        use pse_ids::MemoryReserver;
        let budget = pse_ids::FixedBudget::new(1 << 20);
        let mut reservation = budget.open("snapshot fixture metadata");
        reservation.try_grow(1 << 20).expect("fixture reservation");
        crate::store::control::OwnedControl::new(value, pse_ids::ReservationLease::new(reservation))
    }

    fn loaded(namespace: &str, name: &str, tag: u8) -> Arc<LoadedRelation> {
        use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
        let mut registry = pse_schema::RegistryBuilder::new();
        let namespace_value = match namespace {
            "compiled" => Namespace::Compiled,
            "authored" => Namespace::Authored,
            _ => panic!("fixture namespace"),
        };
        registry.declare_relation(
            RelationDecl::new(
                namespace_value,
                match name {
                    "math_equations" => "math_equations",
                    "math_expr_nodes" => "math_expr_nodes",
                    "units" => "units",
                    _ => panic!("fixture relation"),
                },
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "snapshot fixture",
            )
            .pk(&["id"])
            .columns(vec![
                FieldContract::key(
                    "id",
                    FieldContract::native(datafusion::arrow::datatypes::DataType::Utf8),
                    "identity",
                ),
                FieldContract::payload(
                    "ordinal",
                    FieldContract::native(datafusion::arrow::datatypes::DataType::Int64),
                    "ordinal",
                ),
            ]),
        );
        let registry = registry.build().expect("fixture declarations");
        let spec = registry
            .relation(&format!("{namespace}.{name}"))
            .expect("fixture relation");
        let contract = RelationContract::from_spec(&registry, spec, EncodingPolicy::IpcFile)
            .expect("fixture contract");
        let batch = RecordBatch::try_new(
            Arc::clone(&contract.canonical.schema),
            vec![
                Arc::new(StringArray::from(vec!["a", "b"])),
                Arc::new(Int64Array::from(vec![1_i64, 2])),
            ],
        )
        .expect("fixture arrays");
        let batch = pse_relations::columnar::FieldCheckedBatch::admit(&registry, spec, batch)
            .expect("fixture admission");
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
            member: owned(RelationMember {
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
            }),
        })
    }

    fn snapshot() -> Snapshot {
        let mut relations: BTreeMap<String, Arc<LoadedRelation>> = BTreeMap::new();
        for (namespace, name, tag) in [
            ("compiled", "math_equations", 0x21_u8),
            ("compiled", "math_expr_nodes", 0x22),
            ("authored", "units", 0x23),
        ] {
            relations.insert(format!("{namespace}/{name}"), loaded(namespace, name, tag));
        }
        let manifest = Manifest {
            admission_binding: None,
            manifest_version: MANIFEST_VERSION.to_owned(),
            snapshot_kind: SnapshotKind::Stage,
            snapshot_id: SnapshotId(ContentHash::from_bytes([0x77; 32])),
            membership_profile: SNAPSHOT_PROFILE.to_owned(),
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            schema_registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
            relations: relations
                .values()
                .map(|loaded| loaded.member.as_ref().clone())
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
            manifest_ref: ManifestRef {
                snapshot_id: manifest.snapshot_id,
                manifest_checksum: EncodingChecksum(ContentHash::NIL),
            },
            manifest: owned(manifest),
            relations: Arc::new(relations),
            parents: Arc::new(BTreeMap::new()),
            admission: crate::Catalog::open(
                Arc::new(object_store::memory::InMemory::new()),
                Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
                TrustLevel::Owned,
                Arc::new(crate::store::clock::SystemClock),
                Arc::new(
                    crate::session::SessionFactory::new(
                        Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
                        pse_ids::FixedBudget::new(32 << 20),
                        crate::ExecutionSettings::default(),
                        crate::ThreadBudget {
                            pool_threads: std::num::NonZeroUsize::MIN,
                            target_partitions: std::num::NonZeroUsize::MIN,
                        },
                        crate::session::native_engine_profile(),
                    )
                    .unwrap(),
                ),
            )
            .admission,
            stage_pass: None,
            invocation: None,
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
