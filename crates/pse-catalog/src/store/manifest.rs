// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse.manifest.v2`: the physical envelope around the explicit semantic membership of
//! blueprint §5.3 step 7 (blueprint §20.2, ADR-0044, ADR-0045, ADR-0049).
//!
//! The registry's `ManifestSpec` generates every wire field and nested object
//! (ADR-0060). This module supplies existing native codecs and independent envelope
//! admission. Unknown fields are refused recursively; hashes retain their explicit
//! algorithm prefix and role types. A decoded envelope is not an admitted snapshot.
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
use crate::store::layout::{EncodingFormat, evidence_path, is_sidecar_path, relation_path};

use crate::generated::manifest::ManifestSemanticParentsItem as ParentWire;
/// Generated wire declarations retain the existing catalog API names.
pub use crate::generated::manifest::{
    MANIFEST_VERSION, Manifest, ManifestAdmissionBinding as AdmissionBindingRef,
    ManifestCompiler as CompilerRef, ManifestCompilerPassesItem as PassRef,
    ManifestEngineProfile as EngineProfileRef, ManifestEvidenceItem as EvidenceRecord,
    ManifestKernelsItem as KernelRef, ManifestNumericalPolicy as NumericalPolicyRef,
    ManifestPackagesItem as PackageRef, ManifestRelationsItem as RelationMember,
    ManifestRelationsItemEncodingsItem as EncodingRecord, ManifestToolchain as ToolchainRef,
};

/// `blake3:<64 hex>`, the textual form of every hash in the manifest.
pub(crate) mod hash_text {
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
        pub(crate) mod $module {
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
pub(crate) mod semantic_id_text {
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
pub(crate) mod schema_version_number {
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
pub(crate) mod snapshot_kind_text {
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
pub(crate) mod encoding_format_text {
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

/// `Vec<SnapshotParent>` as an array of `{role, snapshot_id}` objects.
pub(crate) mod parents_wire {
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

    /// Checks the manifest envelope and its self-consistency (blueprint §20.2).
    ///
    /// Checks format versions, the registry fingerprint, member uniqueness, storage
    /// paths, evidence noncanonicality and the recomputed `snapshot_id`. These are
    /// envelope checks only: equality of an identity does not validate decoded relation
    /// values, keys, references, required membership or stage preconditions. The store
    /// must establish those contracts separately before exposing a snapshot.
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
        if !super::clock::valid_rfc3339_utc(&self.created_at) {
            return Err(CatalogError::ManifestInvalid {
                reason: "created_at must be a calendar-valid RFC3339 UTC timestamp".to_owned(),
            });
        }
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
        if self.toolchain.canonicalization != pse_ids::CANON_VERSION {
            return Err(CatalogError::UnknownVersion {
                field: "toolchain.canonicalization".to_owned(),
                value: self.toolchain.canonicalization.clone(),
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

    /// Ports are unique. Model/case relation triples are unique; a stage may declare
    /// distinct output ports with the same schema, each carrying its own complete rows.
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

        if self.snapshot_kind == SnapshotKind::Stage {
            return Ok(());
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
            let expected = evidence_path(&evidence.encoding_checksum);
            if evidence.path != expected.as_ref() {
                return Err(CatalogError::ManifestInvalid {
                    reason: format!(
                        "evidence `{}` stores `{}`, but its checksum-derived layout is `{expected}`",
                        evidence.kind, evidence.path
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
    /// [`Manifest::validate`] for envelope consistency; it does not admit relation data.
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
            admission_binding: Some(AdmissionBindingRef {
                encoding_checksum: EncodingChecksum(ContentHash::from_bytes([0x59; 32])),
            }),
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
            engine_profile: Some(EngineProfileRef {
                engine_profile_id: SemanticId::from_bytes([0x51; 16]),
                content_hash: ContentHash::from_bytes([0x52; 32]),
            }),
            numerical_policy: Some(NumericalPolicyRef {
                policy_id: SemanticId::from_bytes([0x53; 16]),
                content_hash: ContentHash::from_bytes([0x54; 32]),
            }),
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
    fn actual_utc_calendar_validity_is_checked_even_with_consistent_hashes() {
        for value in [
            "0001-01-01T00:00:00Z",
            "2000-02-29T23:59:59.1Z",
            "2024-02-29T00:00:00.123456789Z",
        ] {
            let mut manifest = fixture();
            manifest.created_at = value.to_owned();
            rehashed(&mut manifest);
            manifest
                .validate(fingerprint())
                .expect("calendar-valid UTC");
        }
        for value in [
            "",
            "not a timestamp",
            "0000-01-01T00:00:00Z",
            "2100-02-29T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:60:00Z",
            "2026-01-01T00:00:60Z",
            "2026-01-01T00:00:00.Z",
            "2026-01-01T00:00:00.1234567890Z",
            "2026-01-01T00:00:00+00:00",
            "2026-01-01T00:00:00.αZ",
        ] {
            let mut manifest = fixture();
            manifest.created_at = value.to_owned();
            rehashed(&mut manifest);
            assert!(
                matches!(
                    manifest.validate(fingerprint()),
                    Err(CatalogError::ManifestInvalid { .. })
                ),
                "{value}"
            );
        }
    }

    /// Recompute the membership identity after tampering, then round-trip the envelope
    /// so refusal exercises a decodable manifest carrying a consistent identity.
    fn rehashed(manifest: &mut Manifest) {
        manifest.snapshot_id =
            snapshot_id(&manifest.frame()).expect("the tampered frame is nameable");
        let bytes = manifest
            .encode()
            .expect("the tampered envelope is encodable");
        *manifest = Manifest::decode(&bytes).expect("the tampered envelope decodes");
    }

    #[test]
    fn the_manifest_uses_registry_declared_artifact_field_names() {
        let json = serde_json::to_value(fixture()).expect("manifest encodes");
        assert!(json["engine_profile"].get("engine_profile_id").is_some());
        assert!(json["numerical_policy"].get("policy_id").is_some());
        assert!(json["engine_profile"].get("id").is_none());
        assert!(json["numerical_policy"].get("id").is_none());
        let mut wrong = json;
        let value = wrong["engine_profile"]["engine_profile_id"].take();
        wrong["engine_profile"]["id"] = value;
        assert!(Manifest::decode(&serde_json::to_vec(&wrong).expect("JSON encodes")).is_err());
    }

    /// A populated fixture exercises every registry-declared branch; this is checked
    /// recursively, so a nested wire-field rename cannot hide behind a top-level match.
    fn assert_manifest_type(
        value: &serde_json::Value,
        ty: &pse_schema::model::ManifestType,
        path: &str,
    ) {
        use pse_schema::model::ManifestType;
        match ty {
            ManifestType::Text | ManifestType::Timestamp => assert!(value.is_string(), "{path}"),
            ManifestType::U32 => assert!(
                value.as_u64().is_some_and(|v| u32::try_from(v).is_ok()),
                "{path}"
            ),
            ManifestType::U64 => assert!(value.as_u64().is_some(), "{path}"),
            ManifestType::Bool => assert!(value.is_boolean(), "{path}"),
            ManifestType::Id => assert!(
                value
                    .as_str()
                    .is_some_and(|v| SemanticId::parse_hex(v).is_ok()),
                "{path}"
            ),
            ManifestType::Hash => assert!(
                value
                    .as_str()
                    .is_some_and(|v| ContentHash::parse_prefixed(v).is_ok()),
                "{path}"
            ),
            ManifestType::List(inner) => {
                let values = value.as_array().expect("list shape must match registry");
                assert!(
                    !values.is_empty(),
                    "the parity fixture must exercise {path}"
                );
                for (ordinal, value) in values.iter().enumerate() {
                    assert_manifest_type(value, inner, &format!("{path}/{ordinal}"));
                }
            }
            ManifestType::Struct(fields) => {
                let object = value.as_object().expect("object shape must match registry");
                assert_eq!(
                    object.len(),
                    fields.len(),
                    "{path}: extra or missing wire fields"
                );
                for field in fields {
                    let value = object
                        .get(field.name)
                        .expect("registry-declared field must exist");
                    assert_manifest_type(value, &field.ty, &format!("{path}/{}", field.name));
                }
            }
            ManifestType::Optional(inner) => {
                if !value.is_null() {
                    assert_manifest_type(value, inner, path);
                }
            }
        }
    }

    #[test]
    fn rust_manifest_matches_the_recursive_registry_declaration() {
        let registry = pse_schema::registry().expect("registry is valid");
        let spec = registry.manifest().expect("manifest is declared");
        let manifest = fixture();
        assert_eq!(manifest.manifest_version, spec.version);
        assert_eq!(manifest.membership_profile, spec.membership_profile);
        let value = serde_json::to_value(&manifest).expect("manifest encodes");
        assert_manifest_type(
            &value,
            &pse_schema::model::ManifestType::Struct(spec.fields().to_vec()),
            "manifest",
        );
        let mut optional_absent = manifest;
        optional_absent.engine_profile = None;
        optional_absent.numerical_policy = None;
        let value = serde_json::to_value(&optional_absent).expect("manifest encodes");
        assert_manifest_type(
            &value,
            &pse_schema::model::ManifestType::Struct(spec.fields().to_vec()),
            "manifest",
        );
        assert_eq!(
            Manifest::decode(&serde_json::to_vec(&value).expect("JSON encodes"))
                .expect("optional fields decode"),
            optional_absent
        );
    }

    fn malformed_objects(
        complete: &serde_json::Value,
        ty: &pse_schema::model::ManifestType,
        path: &str,
    ) {
        use pse_schema::model::ManifestType;
        match ty {
            ManifestType::Struct(fields) => {
                let mut extra = complete.clone();
                extra.pointer_mut(path).expect("fixture object")["unknown_future_member"] =
                    serde_json::Value::Bool(true);
                assert!(
                    Manifest::decode(&serde_json::to_vec(&extra).expect("JSON")).is_err(),
                    "unknown field at {path}"
                );
                for field in fields {
                    let mut absent = complete.clone();
                    absent
                        .pointer_mut(path)
                        .expect("fixture object")
                        .as_object_mut()
                        .expect("object")
                        .remove(field.name);
                    let decoded = Manifest::decode(&serde_json::to_vec(&absent).expect("JSON"));
                    assert_eq!(
                        decoded.is_ok(),
                        matches!(field.ty, ManifestType::Optional(_)),
                        "absence at {path}/{}",
                        field.name
                    );
                    malformed_objects(complete, &field.ty, &format!("{path}/{}", field.name));
                }
            }
            ManifestType::List(inner) => {
                let values = complete
                    .pointer(path)
                    .expect("fixture list")
                    .as_array()
                    .expect("list");
                for index in 0..values.len() {
                    malformed_objects(complete, inner, &format!("{path}/{index}"));
                }
            }
            ManifestType::Optional(inner) => malformed_objects(complete, inner, path),
            _ => {}
        }
    }

    #[test]
    fn generated_nested_objects_refuse_unknown_and_missing_required_fields() {
        let registry = pse_schema::registry().expect("registry");
        let spec = registry.manifest().expect("manifest");
        let value = serde_json::to_value(fixture()).expect("populated wire fixture");
        malformed_objects(
            &value,
            &pse_schema::model::ManifestType::Struct(spec.fields().to_vec()),
            "",
        );
        let mut without_optionals = fixture();
        without_optionals.engine_profile = None;
        without_optionals.numerical_policy = None;
        let text = String::from_utf8(without_optionals.encode().expect("wire")).expect("UTF-8");
        assert!(text.contains("\"engine_profile\":null"));
        assert!(text.contains("\"numerical_policy\":null"));
    }

    #[test]
    fn an_unknown_canonicalization_is_refused_with_consistent_hashes() {
        let mut manifest = fixture();
        manifest.toolchain.canonicalization = "pse.canon.v999".to_owned();
        rehashed(&mut manifest);
        assert!(matches!(
            manifest.validate(fingerprint()),
            Err(CatalogError::UnknownVersion { ref field, .. })
                if field == "toolchain.canonicalization"
        ));
    }

    #[test]
    fn evidence_must_have_its_own_checksum_path_despite_consistent_hashes() {
        for path in [
            "evidence/wrong",
            "relations/authored/fake.arrow",
            "../outside",
            "/evidence/absolute",
        ] {
            let mut manifest = fixture();
            manifest.evidence[0].path = path.to_owned();
            rehashed(&mut manifest);
            assert!(matches!(
                manifest.validate(fingerprint()),
                Err(CatalogError::ManifestInvalid { .. })
            ));
        }
    }

    #[test]
    fn missing_relation_encodings_fail_despite_consistent_hashes() {
        let mut manifest = fixture();
        manifest.relations[0].encodings.clear();
        rehashed(&mut manifest);
        assert!(
            matches!(manifest.validate(fingerprint()), Err(CatalogError::ManifestInvalid { ref reason }) if reason.contains("no stored encoding"))
        );
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
        manifest.snapshot_kind = SnapshotKind::Model;
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
    fn stage_ports_can_share_a_relation_contract() {
        let mut manifest = fixture();
        let mut second = manifest.relations[0].clone();
        second.port = "other_declared_output".to_owned();
        manifest.relations.push(second);
        rehashed(&mut manifest);
        manifest
            .validate(fingerprint())
            .expect("distinct stage ports");
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
