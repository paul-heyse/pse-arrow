// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse.manifest.v2`: the physical envelope around the explicit semantic membership of
//! blueprint §5.3 step 7 (blueprint §20.2, ADR-0044, ADR-0045, ADR-0049).
//!
//! # Why this is a hand-written wire type
//!
//! The manifest is a *format*, not a convenience view over some in-memory struct. Two
//! consequences follow and both are visible in this file:
//!
//! - **`deny_unknown_fields` everywhere.** §20.5 rejects unknown manifest versions rather
//!   than reading what it recognises; a field this build does not know about is a field
//!   whose meaning it cannot honour, and silently ignoring it is how a reader ends up
//!   serving a snapshot under a contract it never implemented.
//! - **Hashes are text with their algorithm attached.** `blake3:<64 hex>` rather than a
//!   bare digest, because the algorithm is part of the value (`pse-ids` says so) and a
//!   manifest outlives the build that wrote it. `pse-ids` stays serde-free, so the
//!   conversion lives here, in the crate that owns the format.
//!
//! # What is not in the manifest's own identity
//!
//! The manifest's checksum is held by the ref, never embedded in itself (§20.1), and the
//! `snapshot_id` field is *recomputed* from the membership at [`Manifest::validate`]
//! rather than trusted. Evidence, `created_at` and alternative encodings are excluded
//! from logical membership: two valid encodings of the same logical snapshot have
//! different manifest checksums and the same `snapshot_id`.

use pse_ids::{
    ContentHash, EncodingChecksum, LogicalHash, SchemaVersion, SemanticId, SnapshotFrame,
    SnapshotId, SnapshotKind, SnapshotMember, SnapshotParent, snapshot_id,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::CatalogError;
use crate::store::layout::{EncodingFormat, is_sidecar_path, relation_path};

/// The frozen manifest format version (blueprint §20.2).
pub const MANIFEST_VERSION: &str = "pse.manifest.v2";

/// `blake3:<64 hex>`, the textual form of every hash in the manifest.
mod hash_text {
    use super::{ContentHash, Deserialize, Deserializer, Serializer};

    /// Writes `blake3:<64 hex>`.
    pub(crate) fn serialize<S: Serializer>(
        value: &ContentHash,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_prefixed())
    }

    /// Reads `blake3:<64 hex>`, refusing an unprefixed digest.
    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ContentHash, D::Error> {
        let text = String::deserialize(deserializer)?;
        ContentHash::parse_prefixed(&text).map_err(serde::de::Error::custom)
    }
}

/// Declares a `#[serde(with = ...)]` module for one of the [`ContentHash`] role newtypes.
///
/// The roles are separate types on purpose (ADR-0045); they share one wire form, and a
/// macro is how they share it without the wire form being written four times.
macro_rules! hash_role_text {
    ($module:ident, $role:ident) => {
        /// The `blake3:<64 hex>` form of a hash role newtype.
        mod $module {
            use super::{Deserializer, Serializer, hash_text, $role};

            /// Writes `blake3:<64 hex>`.
            pub(crate) fn serialize<S: Serializer>(
                value: &$role,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                hash_text::serialize(&value.0, serializer)
            }

            /// Reads `blake3:<64 hex>`.
            pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
                deserializer: D,
            ) -> Result<$role, D::Error> {
                hash_text::deserialize(deserializer).map($role)
            }
        }
    };
}

hash_role_text!(logical_hash_text, LogicalHash);
hash_role_text!(encoding_checksum_text, EncodingChecksum);
hash_role_text!(snapshot_id_text, SnapshotId);

/// A [`SemanticId`] as 32 lowercase hexadecimal digits, with no algorithm prefix: an ID
/// is not a digest of anything, so `blake3:` would be a lie.
mod semantic_id_text {
    use super::{Deserialize, Deserializer, SemanticId, Serializer};

    /// Writes 32 lowercase hexadecimal digits.
    pub(crate) fn serialize<S: Serializer>(
        value: &SemanticId,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_hex())
    }

    /// Reads 32 hexadecimal digits.
    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<SemanticId, D::Error> {
        let text = String::deserialize(deserializer)?;
        SemanticId::parse_hex(&text).map_err(serde::de::Error::custom)
    }
}

/// A [`SchemaVersion`] as a JSON number, as §20.2's `"version": 1` shows it.
mod schema_version_number {
    use super::{Deserialize, Deserializer, SchemaVersion, Serializer};

    /// Writes the version as an unsigned number.
    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde's `with` module contract fixes the signature as `&T`"
    )]
    pub(crate) fn serialize<S: Serializer>(
        value: &SchemaVersion,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(value.0)
    }

    /// Reads an unsigned number.
    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<SchemaVersion, D::Error> {
        u32::deserialize(deserializer).map(SchemaVersion)
    }
}

/// A [`SnapshotKind`] as `model`, `case`, `stage` or `run`.
mod snapshot_kind_text {
    use super::{Deserialize, Deserializer, Serializer, SnapshotKind};

    /// Every kind; the parse side reads this rather than repeating the spellings.
    const ALL: [SnapshotKind; 4] = [
        SnapshotKind::Model,
        SnapshotKind::Case,
        SnapshotKind::Stage,
        SnapshotKind::Run,
    ];

    /// Writes the lowercase spelling `pse-ids` frames.
    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde's `with` module contract fixes the signature as `&T`"
    )]
    pub(crate) fn serialize<S: Serializer>(
        value: &SnapshotKind,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(value.as_str())
    }

    /// Reads one of the four spellings, refusing any other.
    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<SnapshotKind, D::Error> {
        let text = String::deserialize(deserializer)?;
        ALL.into_iter()
            .find(|kind| kind.as_str() == text)
            .ok_or_else(|| serde::de::Error::custom(format!("`{text}` is not a snapshot kind")))
    }
}

/// An [`EncodingFormat`] as `arrow_ipc_file` or `parquet`.
mod encoding_format_text {
    use super::{Deserialize, Deserializer, EncodingFormat, Serializer};

    /// Writes the spelling the format declares.
    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde's `with` module contract fixes the signature as `&T`"
    )]
    pub(crate) fn serialize<S: Serializer>(
        value: &EncodingFormat,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(value.as_str())
    }

    /// Reads a declared spelling, refusing any other.
    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<EncodingFormat, D::Error> {
        let text = String::deserialize(deserializer)?;
        EncodingFormat::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// The wire form of a [`SnapshotParent`], which `pse-ids` does not carry itself.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParentWire {
    /// The input binding this parent was bound to.
    role: String,
    /// The parent snapshot.
    #[serde(with = "snapshot_id_text")]
    snapshot_id: SnapshotId,
}

/// `Vec<SnapshotParent>` as an array of `{role, snapshot_id}` objects.
mod parents_wire {
    use super::{Deserialize, Deserializer, ParentWire, Serialize, Serializer, SnapshotParent};

    /// Writes the parents in the order the manifest holds them.
    pub(crate) fn serialize<S: Serializer>(
        value: &[SnapshotParent],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let wire: Vec<ParentWire> = value
            .iter()
            .map(|parent| ParentWire {
                role: parent.role.clone(),
                snapshot_id: parent.snapshot_id,
            })
            .collect();
        wire.serialize(serializer)
    }

    /// Reads the parents, preserving order.
    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<SnapshotParent>, D::Error> {
        let wire = Vec::<ParentWire>::deserialize(deserializer)?;
        Ok(wire
            .into_iter()
            .map(|parent| SnapshotParent {
                role: parent.role,
                snapshot_id: parent.snapshot_id,
            })
            .collect())
    }
}

/// One stored encoding of one relation (blueprint §20.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncodingRecord {
    /// Which physical encoding this is.
    #[serde(with = "encoding_format_text")]
    pub format: EncodingFormat,
    /// The writer that produced the bytes, for §20.5 compatibility decisions.
    pub writer_version: String,
    /// The checksum of the finished bytes (ADR-0045).
    #[serde(with = "encoding_checksum_text")]
    pub encoding_checksum: EncodingChecksum,
    /// The object's length, checked before the object is accepted.
    pub bytes: u64,
    /// The object-store path, which must equal the §20.1 layout for this record.
    pub path: String,
}

/// One complete relation artifact in a snapshot (blueprint §20.2).
///
/// A member is always a *complete* relation: §5.3 step 7 declares any row-selection
/// transformation before snapshot assembly, so a member cannot be a filtered view of one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationMember {
    /// The port under which this relation is published.
    pub port: String,
    /// The relation's namespace.
    pub namespace: String,
    /// The relation's registry identity.
    #[serde(with = "semantic_id_text")]
    pub relation_id: SemanticId,
    /// The relation's declared name, which a rename may change without changing the port.
    pub name: String,
    /// The relation's declared schema version.
    #[serde(with = "schema_version_number")]
    pub version: SchemaVersion,
    /// The relation's logical content identity (blueprint §5.3 step 6).
    #[serde(with = "logical_hash_text")]
    pub logical_hash: LogicalHash,
    /// The row count, which is the `Exact` statistic a scan may report (§5.4).
    pub rows: u64,
    /// Every stored encoding of this relation.
    pub encodings: Vec<EncodingRecord>,
}

/// A package the snapshot was built from (blueprint §20.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageRef {
    /// The package's identity.
    #[serde(with = "semantic_id_text")]
    pub package_id: SemanticId,
    /// The package version as authored.
    pub version: String,
    /// The logical hash of the package's content.
    #[serde(with = "logical_hash_text")]
    pub logical_hash: LogicalHash,
}

/// One compiler pass and the version of it that ran (blueprint §20.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassRef {
    /// The pass's identity.
    #[serde(with = "semantic_id_text")]
    pub pass_id: SemanticId,
    /// The pass version.
    pub version: String,
}

/// The compiler that produced the snapshot (blueprint §20.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerRef {
    /// The compiler version.
    pub version: String,
    /// Every pass that ran, in the order it ran.
    pub passes: Vec<PassRef>,
}

/// A content-addressed artifact the snapshot depends on: an engine profile or a
/// numerical policy (blueprint §20.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    /// The artifact's identity.
    #[serde(with = "semantic_id_text")]
    pub id: SemanticId,
    /// The hash of the artifact's content.
    #[serde(with = "hash_text")]
    pub content_hash: ContentHash,
}

/// The toolchain the snapshot was produced with (blueprint §20.2, §20.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainRef {
    /// The hash of the committed lockfile.
    #[serde(with = "hash_text")]
    pub lockfile_hash: ContentHash,
    /// The canonicalization contract version, `pse.canon.v2`.
    pub canonicalization: String,
}

/// One kernel implementation the snapshot depends on (blueprint §20.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KernelRef {
    /// The kernel's identity.
    #[serde(with = "semantic_id_text")]
    pub kernel_id: SemanticId,
    /// The kernel version.
    pub version: String,
    /// The digest of the implementation artifact.
    #[serde(with = "hash_text")]
    pub digest: ContentHash,
}

/// A noncanonical plan or diagnostic encoding (ADR-0044, blueprint §20.2).
///
/// `canonical` is always `false` and is written out anyway: ADR-0044 bounds what plan
/// bytes mean, and a reader that has to *infer* noncanonicality from the record's absence
/// will eventually infer wrongly and use a plan checksum as a semantic key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    /// What kind of evidence this is, such as `datafusion_proto`.
    pub kind: String,
    /// Always `false`; evidence is never canonical (ADR-0044).
    pub canonical: bool,
    /// The checksum of the stored evidence bytes.
    #[serde(with = "encoding_checksum_text")]
    pub encoding_checksum: EncodingChecksum,
    /// The codec and its version, for decoding and attribution.
    pub codec_version: String,
    /// The object-store path of the evidence object.
    pub path: String,
}

/// A `pse.manifest.v2` manifest (blueprint §20.2).
///
/// Field order is declaration order is JSON order, and that is a contract: the encoded
/// bytes are what [`crate::store::layout::manifest_path`] names, so a reordering would
/// produce a second object for the same manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::struct_field_names,
    reason = "`manifest_version` is the §20.2 wire key; renaming the field renames the JSON"
)]
pub struct Manifest {
    /// The manifest format version; must equal [`MANIFEST_VERSION`].
    pub manifest_version: String,
    /// What kind of snapshot this manifest envelopes.
    #[serde(with = "snapshot_kind_text")]
    pub snapshot_kind: SnapshotKind,
    /// The snapshot's membership identity, recomputed at [`Manifest::validate`].
    #[serde(with = "snapshot_id_text")]
    pub snapshot_id: SnapshotId,
    /// The membership frame version; must equal `pse_ids::SNAPSHOT_PROFILE`.
    pub membership_profile: String,
    /// When the manifest was written; excluded from logical membership.
    pub created_at: String,
    /// The schema registry fingerprint the members were declared under.
    #[serde(with = "hash_text")]
    pub schema_registry_fingerprint: ContentHash,
    /// The complete member set.
    pub relations: Vec<RelationMember>,
    /// The packages the snapshot was built from.
    pub packages: Vec<PackageRef>,
    /// The compiler and its passes.
    pub compiler: CompilerRef,
    /// The engine profile, when the snapshot kind requires one.
    pub engine_profile: Option<ArtifactRef>,
    /// The numerical policy, when the snapshot kind requires one.
    pub numerical_policy: Option<ArtifactRef>,
    /// The toolchain.
    pub toolchain: ToolchainRef,
    /// The kernels the snapshot depends on.
    pub kernels: Vec<KernelRef>,
    /// The semantic parents, by role.
    #[serde(with = "parents_wire")]
    pub semantic_parents: Vec<SnapshotParent>,
    /// Noncanonical evidence; excluded from logical membership (ADR-0044).
    pub evidence: Vec<EvidenceRecord>,
}

impl Manifest {
    /// The `pse.snapshot.v2` frame this manifest's membership names (§5.3 step 7).
    #[must_use]
    pub fn frame(&self) -> SnapshotFrame {
        SnapshotFrame {
            registry_fingerprint: self.schema_registry_fingerprint,
            kind: self.snapshot_kind,
            parents: self.semantic_parents.clone(),
            members: self
                .relations
                .iter()
                .map(|member| SnapshotMember {
                    port: member.port.clone(),
                    namespace: member.namespace.clone(),
                    relation_id: member.relation_id,
                    schema_version: member.version,
                    logical_hash: member.logical_hash,
                })
                .collect(),
        }
    }

    /// Checks everything §20.2 requires before a snapshot is exposed.
    ///
    /// In order: the two format versions, the registry fingerprint, port uniqueness,
    /// relation uniqueness, path containment, evidence noncanonicality, the absence of
    /// sidecar and self references, and finally the recomputed `snapshot_id`. The
    /// membership identity is recomputed rather than trusted — a manifest that carries
    /// its own name is only as trustworthy as the process that wrote it, and §20.1
    /// requires readers to "recompute its semantic snapshot identity".
    ///
    /// # Errors
    ///
    /// - [`CatalogError::UnknownVersion`] for a manifest or membership version this build
    ///   does not implement (§20.5 rejects rather than reads what it recognises).
    /// - [`CatalogError::UnknownRegistry`] when the members were declared under a
    ///   different schema registry.
    /// - [`CatalogError::ManifestInvalid`] for every structural failure, naming it.
    /// - [`CatalogError::Snapshot`] when the frame itself cannot be named.
    pub fn validate(&self, registry_fingerprint: ContentHash) -> Result<(), CatalogError> {
        if self.manifest_version != MANIFEST_VERSION {
            return Err(CatalogError::UnknownVersion {
                field: "manifest_version".to_owned(),
                value: self.manifest_version.clone(),
            });
        }
        if self.membership_profile != pse_ids::SNAPSHOT_PROFILE {
            return Err(CatalogError::UnknownVersion {
                field: "membership_profile".to_owned(),
                value: self.membership_profile.clone(),
            });
        }
        if self.schema_registry_fingerprint != registry_fingerprint {
            return Err(CatalogError::UnknownRegistry {
                fingerprint: self.schema_registry_fingerprint,
            });
        }

        self.check_member_uniqueness()?;
        self.check_paths()?;
        self.check_evidence()?;

        let recomputed = snapshot_id(&self.frame())?;
        if recomputed != self.snapshot_id {
            return Err(CatalogError::ManifestInvalid {
                reason: format!(
                    "membership names snapshot {recomputed}, but the manifest claims {}",
                    self.snapshot_id
                ),
            });
        }
        Ok(())
    }

    /// Ports and `(namespace, relation_id, version)` triples are unique.
    ///
    /// Both, not one: a repeated port makes the frame ambiguous, and a repeated relation
    /// under two ports would publish one relation twice with two identities.
    fn check_member_uniqueness(&self) -> Result<(), CatalogError> {
        let mut ports: Vec<&str> = self.relations.iter().map(|m| m.port.as_str()).collect();
        ports.sort_unstable();
        for pair in ports.windows(2) {
            if let [left, right] = pair
                && left == right
            {
                return Err(CatalogError::ManifestInvalid {
                    reason: format!("port `{left}` appears more than once"),
                });
            }
        }

        let mut triples: Vec<(&str, [u8; 16], u32)> = self
            .relations
            .iter()
            .map(|m| (m.namespace.as_str(), *m.relation_id.as_bytes(), m.version.0))
            .collect();
        triples.sort_unstable();
        for pair in triples.windows(2) {
            if let [left, right] = pair
                && left == right
            {
                return Err(CatalogError::ManifestInvalid {
                    reason: format!(
                        "relation {} in `{}` at version {} appears more than once",
                        SemanticId::from_bytes(left.1),
                        left.0,
                        left.2
                    ),
                });
            }
        }
        Ok(())
    }

    /// Every encoding path is the §20.1 path for its own record, and none is a sidecar.
    fn check_paths(&self) -> Result<(), CatalogError> {
        for member in &self.relations {
            if member.encodings.is_empty() {
                return Err(CatalogError::ManifestInvalid {
                    reason: format!("member `{}` has no stored encoding", member.port),
                });
            }
            for encoding in &member.encodings {
                let expected = relation_path(
                    &member.namespace,
                    &member.name,
                    member.version,
                    &encoding.encoding_checksum,
                    encoding.format,
                );
                if encoding.path != expected.as_ref() {
                    return Err(CatalogError::ManifestInvalid {
                        reason: format!(
                            "member `{}` stores `{}`, but its layout is `{expected}`",
                            member.port, encoding.path
                        ),
                    });
                }
            }
        }
        for evidence in &self.evidence {
            if is_sidecar_path(&evidence.path) {
                return Err(CatalogError::ManifestInvalid {
                    reason: format!(
                        "evidence `{}` lives under a sidecar directory; a manifest never \
                         references its own bookkeeping",
                        evidence.path
                    ),
                });
            }
        }
        Ok(())
    }

    /// No evidence record claims to be canonical (ADR-0044).
    fn check_evidence(&self) -> Result<(), CatalogError> {
        for evidence in &self.evidence {
            if evidence.canonical {
                return Err(CatalogError::ManifestInvalid {
                    reason: format!(
                        "evidence `{}` claims to be canonical; plan encodings never are \
                         (ADR-0044)",
                        evidence.kind
                    ),
                });
            }
        }
        Ok(())
    }

    /// The encoded manifest: compact JSON in declaration order.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Internal`]: every field of this type is JSON-encodable, so a
    /// failure here is a platform bug rather than a caller's input.
    pub fn encode(&self) -> Result<Vec<u8>, CatalogError> {
        serde_json::to_vec(self).map_err(|error| CatalogError::Internal {
            message: format!("the manifest could not be encoded: {error}"),
        })
    }

    /// Reads an encoded manifest.
    ///
    /// # Errors
    ///
    /// [`CatalogError::ManifestInvalid`] for malformed JSON, a missing field, an unknown
    /// field or an unreadable hash. Decoding proves the shape only; call
    /// [`Manifest::validate`] for the semantics.
    pub fn decode(bytes: &[u8]) -> Result<Self, CatalogError> {
        serde_json::from_slice(bytes).map_err(|error| CatalogError::ManifestInvalid {
            reason: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use pse_ids::{CANON_VERSION, SNAPSHOT_PROFILE};

    use super::*;
    use crate::store::clock::{Clock, FixedClock};

    /// The registry fingerprint the fixture declares its members under.
    fn fingerprint() -> ContentHash {
        ContentHash::from_bytes([0x11; 32])
    }

    fn checksum(byte: u8) -> EncodingChecksum {
        EncodingChecksum(ContentHash::from_bytes([byte; 32]))
    }

    fn member(namespace: &str, name: &str, tag: u8) -> RelationMember {
        let version = SchemaVersion(1);
        let encoding_checksum = checksum(tag);
        let path = relation_path(
            namespace,
            name,
            version,
            &encoding_checksum,
            EncodingFormat::ArrowIpcFile,
        );
        RelationMember {
            port: format!("{namespace}/{name}"),
            namespace: namespace.to_owned(),
            relation_id: SemanticId::from_bytes([tag; 16]),
            name: name.to_owned(),
            version,
            logical_hash: LogicalHash(ContentHash::from_bytes([tag; 32])),
            rows: u64::from(tag),
            encodings: vec![EncodingRecord {
                format: EncodingFormat::ArrowIpcFile,
                writer_version: "arrow-rs 59.3.0".to_owned(),
                encoding_checksum,
                bytes: 1_234,
                path: path.as_ref().to_owned(),
            }],
        }
    }

    /// A two-member, one-parent stage manifest, stamped by a [`FixedClock`].
    fn fixture() -> Manifest {
        let clock = FixedClock("2026-01-01T00:00:00Z".to_owned());
        let mut manifest = Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            snapshot_kind: SnapshotKind::Stage,
            snapshot_id: SnapshotId(ContentHash::NIL),
            membership_profile: SNAPSHOT_PROFILE.to_owned(),
            created_at: clock.now_rfc3339_utc(),
            schema_registry_fingerprint: fingerprint(),
            relations: vec![
                member("compiled", "math_expr_nodes", 0x21),
                member("compiled", "math_equations", 0x22),
            ],
            packages: vec![PackageRef {
                package_id: SemanticId::from_bytes([0x31; 16]),
                version: "1.2.0".to_owned(),
                logical_hash: LogicalHash(ContentHash::from_bytes([0x32; 32])),
            }],
            compiler: CompilerRef {
                version: "0.1.0".to_owned(),
                passes: vec![PassRef {
                    pass_id: SemanticId::from_bytes([0x41; 16]),
                    version: "1".to_owned(),
                }],
            },
            engine_profile: Some(ArtifactRef {
                id: SemanticId::from_bytes([0x51; 16]),
                content_hash: ContentHash::from_bytes([0x52; 32]),
            }),
            numerical_policy: None,
            toolchain: ToolchainRef {
                lockfile_hash: ContentHash::from_bytes([0x61; 32]),
                canonicalization: CANON_VERSION.to_owned(),
            },
            kernels: vec![KernelRef {
                kernel_id: SemanticId::from_bytes([0x71; 16]),
                version: "1".to_owned(),
                digest: ContentHash::from_bytes([0x72; 32]),
            }],
            semantic_parents: vec![SnapshotParent {
                role: "input.typed_math".to_owned(),
                snapshot_id: SnapshotId(ContentHash::from_bytes([0x81; 32])),
            }],
            evidence: vec![EvidenceRecord {
                kind: "datafusion_proto".to_owned(),
                canonical: false,
                encoding_checksum: checksum(0x91),
                codec_version: "datafusion-proto 55.1.0".to_owned(),
                path: format!("evidence/{}", ContentHash::from_bytes([0x91; 32]).to_hex()),
            }],
        };
        let Ok(named) = snapshot_id(&manifest.frame()) else {
            panic!("the fixture frame is nameable");
        };
        manifest.snapshot_id = named;
        manifest
    }

    #[test]
    fn a_manifest_round_trips_through_its_encoding() {
        let manifest = fixture();
        let Ok(bytes) = manifest.encode() else {
            panic!("the fixture encodes");
        };
        let decoded = Manifest::decode(&bytes);
        let Ok(decoded) = decoded else {
            panic!("the fixture decodes");
        };
        assert_eq!(decoded, manifest);
        let Ok(again) = decoded.encode() else {
            panic!("the decoded manifest encodes");
        };
        assert_eq!(again, bytes, "encoding is a function of the value");
    }

    #[test]
    fn the_encoding_is_declaration_order_and_prefixed_hashes() {
        let Ok(bytes) = fixture().encode() else {
            panic!("the fixture encodes");
        };
        let Ok(text) = String::from_utf8(bytes) else {
            panic!("JSON is UTF-8");
        };
        assert!(text.starts_with(r#"{"manifest_version":"pse.manifest.v2","snapshot_kind":"stage","snapshot_id":"blake3:"#));
        assert!(text.contains(r#""membership_profile":"pse.snapshot.v2""#));
        assert!(text.contains(r#""created_at":"2026-01-01T00:00:00Z""#));
        assert!(text.contains(r#""format":"arrow_ipc_file""#));
        assert!(text.contains(r#""version":1"#));
        assert!(text.contains(r#""canonical":false"#));
        assert!(text.ends_with('}'));
    }

    #[test]
    fn the_fixture_validates() {
        let manifest = fixture();
        assert!(manifest.validate(fingerprint()).is_ok());
    }

    #[test]
    fn a_wrong_manifest_version_is_refused_rather_than_read() {
        let mut manifest = fixture();
        manifest.manifest_version = "pse.manifest.v1".to_owned();
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::UnknownVersion { ref field, .. }) if field == "manifest_version"
        ));
    }

    #[test]
    fn a_wrong_membership_profile_is_refused() {
        let mut manifest = fixture();
        manifest.membership_profile = "pse.snapshot.v1".to_owned();
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::UnknownVersion { ref field, .. }) if field == "membership_profile"
        ));
    }

    #[test]
    fn an_unknown_registry_fingerprint_is_refused() {
        let manifest = fixture();
        let other = ContentHash::from_bytes([0x99; 32]);
        assert!(matches!(
            manifest.validate(other),
            Err(CatalogError::UnknownRegistry { .. })
        ));
    }

    #[test]
    fn a_duplicate_port_is_refused() {
        let mut manifest = fixture();
        let Some(first) = manifest.relations.first().cloned() else {
            panic!("the fixture has members");
        };
        let Some(second) = manifest.relations.get_mut(1) else {
            panic!("the fixture has two members");
        };
        second.port = first.port.clone();
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { ref reason })
                if reason.contains("appears more than once")
        ));
    }

    #[test]
    fn a_duplicate_relation_triple_is_refused() {
        let mut manifest = fixture();
        let Some(first) = manifest.relations.first().cloned() else {
            panic!("the fixture has members");
        };
        let Some(second) = manifest.relations.get_mut(1) else {
            panic!("the fixture has two members");
        };
        second.relation_id = first.relation_id;
        second.namespace = first.namespace.clone();
        second.version = first.version;
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { ref reason })
                if reason.contains("appears more than once")
        ));
    }

    #[test]
    fn a_path_that_is_not_the_declared_layout_is_refused() {
        let mut manifest = fixture();
        let Some(member) = manifest.relations.first_mut() else {
            panic!("the fixture has members");
        };
        let Some(encoding) = member.encodings.first_mut() else {
            panic!("the fixture member has an encoding");
        };
        encoding.path = "relations/compiled/somewhere_else.arrow".to_owned();
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { ref reason }) if reason.contains("its layout is")
        ));
    }

    #[test]
    fn an_encoding_whose_format_does_not_match_its_extension_is_refused() {
        // Changing only the format leaves the `.arrow` path from the layout in place.
        let mut manifest = fixture();
        let Some(member) = manifest.relations.first_mut() else {
            panic!("the fixture has members");
        };
        let Some(encoding) = member.encodings.first_mut() else {
            panic!("the fixture member has an encoding");
        };
        encoding.format = EncodingFormat::Parquet;
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { .. })
        ));
    }

    #[test]
    fn a_wrong_snapshot_id_is_refused() {
        let mut manifest = fixture();
        manifest.snapshot_id = SnapshotId(ContentHash::from_bytes([0xaa; 32]));
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { ref reason })
                if reason.contains("membership names snapshot")
        ));
    }

    #[test]
    fn a_changed_member_changes_the_membership_identity() {
        let mut manifest = fixture();
        let Some(member) = manifest.relations.first_mut() else {
            panic!("the fixture has members");
        };
        member.logical_hash = LogicalHash(ContentHash::from_bytes([0xbb; 32]));
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { .. })
        ));
    }

    #[test]
    fn canonical_evidence_is_refused() {
        let mut manifest = fixture();
        let Some(evidence) = manifest.evidence.first_mut() else {
            panic!("the fixture has evidence");
        };
        evidence.canonical = true;
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { ref reason })
                if reason.contains("claims to be canonical")
        ));
    }

    #[test]
    fn evidence_under_a_sidecar_directory_is_refused() {
        let mut manifest = fixture();
        let Some(evidence) = manifest.evidence.first_mut() else {
            panic!("the fixture has evidence");
        };
        evidence.path = "stages/aa.json".to_owned();
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { ref reason })
                if reason.contains("sidecar directory")
        ));
    }

    #[test]
    fn a_member_without_an_encoding_is_refused() {
        let mut manifest = fixture();
        let Some(member) = manifest.relations.first_mut() else {
            panic!("the fixture has members");
        };
        member.encodings.clear();
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::ManifestInvalid { ref reason })
                if reason.contains("no stored encoding")
        ));
    }

    #[test]
    fn an_unknown_json_field_is_refused_on_decode() {
        let Ok(bytes) = fixture().encode() else {
            panic!("the fixture encodes");
        };
        let Ok(text) = String::from_utf8(bytes) else {
            panic!("JSON is UTF-8");
        };
        let tampered = text.replacen('{', r#"{"tomorrows_field":1,"#, 1);
        assert!(matches!(
            Manifest::decode(tampered.as_bytes()),
            Err(CatalogError::ManifestInvalid { .. })
        ));
    }

    #[test]
    fn an_unprefixed_hash_is_refused_on_decode() {
        let Ok(bytes) = fixture().encode() else {
            panic!("the fixture encodes");
        };
        let Ok(text) = String::from_utf8(bytes) else {
            panic!("JSON is UTF-8");
        };
        let tampered = text.replace("blake3:", "");
        assert!(matches!(
            Manifest::decode(tampered.as_bytes()),
            Err(CatalogError::ManifestInvalid { .. })
        ));
    }

    #[test]
    fn an_unknown_snapshot_kind_or_format_is_refused_on_decode() {
        let Ok(bytes) = fixture().encode() else {
            panic!("the fixture encodes");
        };
        let Ok(text) = String::from_utf8(bytes) else {
            panic!("JSON is UTF-8");
        };
        for (from, to) in [
            (r#""snapshot_kind":"stage""#, r#""snapshot_kind":"draft""#),
            (r#""format":"arrow_ipc_file""#, r#""format":"feather""#),
        ] {
            let tampered = text.replace(from, to);
            assert_ne!(tampered, text, "the fixture contains `{from}`");
            assert!(
                matches!(
                    Manifest::decode(tampered.as_bytes()),
                    Err(CatalogError::ManifestInvalid { .. })
                ),
                "`{to}` should be refused"
            );
        }
    }

    #[test]
    fn the_frame_reports_what_the_members_declare() {
        let manifest = fixture();
        let frame = manifest.frame();
        assert_eq!(frame.registry_fingerprint, fingerprint());
        assert_eq!(frame.kind, SnapshotKind::Stage);
        assert_eq!(frame.members.len(), 2);
        assert_eq!(frame.parents.len(), 1);
        assert_eq!(
            frame.members.first().map(|m| m.port.as_str()),
            Some("compiled/math_expr_nodes")
        );
    }
}
