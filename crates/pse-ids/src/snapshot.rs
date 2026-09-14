// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse.snapshot.v2`: naming a snapshot's membership without self-reference
//! (blueprint §5.3 step 7, ADR-0045).
//!
//! A snapshot ID is a plain BLAKE3 hash over a framed list of what the snapshot *contains*
//! and what it *derives from*. Three properties follow from the frame layout and are the
//! reason it is written out here rather than left to a serializer:
//!
//! - **Order is not membership.** Parents sort by role bytes and members by
//!   `(port, namespace, relation_id, schema_version)`, so the order a caller happens to
//!   assemble them in cannot change the ID.
//! - **Absence is a fact.** Required empty relations appear explicitly; an omitted member
//!   is a different snapshot, not the same one with less to say.
//! - **A snapshot never contains itself.** Revision catalogs, mutable refs, stage catalogs
//!   and pass-attempt records are `sidecar` and stay out of membership, which is what makes
//!   the ID computable at all.

use crate::error::SnapshotError;
use crate::frame::FrameSink;
use crate::id::{ContentHash, LogicalHash, SchemaVersion, SemanticId, SnapshotId};

/// The frozen frame version string; the first component of every snapshot preimage.
pub const SNAPSHOT_PROFILE: &str = "pse.snapshot.v2";

/// The role a parent must carry for a case snapshot to name the model it overlays.
const MODEL_PARENT_ROLE: &str = "model";

/// What a snapshot is (blueprint §5.3 step 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SnapshotKind {
    /// Structural authored and reference contracts: the model.
    Model,
    /// §6.10 case and observation relations, overlaying a model.
    Case,
    /// One pass stage's complete declared output ports.
    Stage,
    /// One run's runtime relations.
    Run,
}

impl SnapshotKind {
    /// The lowercase spelling that enters the frame.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Case => "case",
            Self::Stage => "stage",
            Self::Run => "run",
        }
    }
}

impl std::fmt::Display for SnapshotKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A semantic parent: what this snapshot was derived from, under the role that binding had.
///
/// A case names its model as `model` and its overlays under their own names; a stage names
/// its input bindings. The role is part of the frame, so rebinding the same parent to a
/// different port is a different snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotParent {
    /// The input binding this parent was bound to.
    pub role: String,
    /// The parent snapshot.
    pub snapshot_id: SnapshotId,
}

/// One complete relation artifact in a snapshot.
///
/// A member is always a *complete* relation: any row-selection transformation is declared
/// before snapshot assembly, never folded into membership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotMember {
    /// The port under which this relation is published; see [`model_port_name`].
    pub port: String,
    /// The relation's namespace.
    pub namespace: String,
    /// The relation's registry identity.
    pub relation_id: SemanticId,
    /// The relation's declared schema version.
    pub schema_version: SchemaVersion,
    /// The relation's logical content identity (blueprint §5.3 step 6).
    pub logical_hash: LogicalHash,
}

/// Everything `snapshot_id` hashes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotFrame {
    /// The registry fingerprint the members were declared under.
    pub registry_fingerprint: ContentHash,
    /// What kind of snapshot this is.
    pub kind: SnapshotKind,
    /// Semantic parents, in any order; the frame sorts them.
    pub parents: Vec<SnapshotParent>,
    /// Members, in any order; the frame sorts them.
    pub members: Vec<SnapshotMember>,
}

/// The member sort key: `(port, namespace, relation_id, schema_version)` (§5.3 step 7).
fn member_key(member: &SnapshotMember) -> (&[u8], &[u8], &[u8; SemanticId::WIDTH], u32) {
    (
        member.port.as_bytes(),
        member.namespace.as_bytes(),
        member.relation_id.as_bytes(),
        member.schema_version.0,
    )
}

/// Writes the `pse.snapshot.v2` preimage of `frame` into `sink`.
fn write_frame<S: FrameSink>(frame: &SnapshotFrame, sink: &mut S) -> Result<(), SnapshotError> {
    let mut parents: Vec<&SnapshotParent> = frame.parents.iter().collect();
    parents.sort_by(|left, right| left.role.as_bytes().cmp(right.role.as_bytes()));
    for pair in parents.windows(2) {
        if let [left, right] = pair
            && left.role == right.role
        {
            return Err(SnapshotError::DuplicateParentRole {
                role: left.role.clone(),
            });
        }
    }

    let mut members: Vec<&SnapshotMember> = frame.members.iter().collect();
    members.sort_by(|left, right| member_key(left).cmp(&member_key(right)));
    for pair in members.windows(2) {
        if let [left, right] = pair
            && left.port == right.port
        {
            return Err(SnapshotError::DuplicateMemberPort {
                port: left.port.clone(),
            });
        }
    }

    if frame.kind == SnapshotKind::Case
        && !parents
            .iter()
            .any(|parent| parent.role == MODEL_PARENT_ROLE)
    {
        return Err(SnapshotError::MissingModelParent);
    }

    sink.put_len_prefixed(SNAPSHOT_PROFILE.as_bytes());
    sink.put_fixed(frame.registry_fingerprint.as_bytes());
    sink.put_len_prefixed(frame.kind.as_str().as_bytes());

    sink.put_u64_le(u64::try_from(parents.len()).unwrap_or(u64::MAX));
    for parent in parents {
        sink.put_len_prefixed(parent.role.as_bytes());
        sink.put_fixed(parent.snapshot_id.0.as_bytes());
    }

    sink.put_u64_le(u64::try_from(members.len()).unwrap_or(u64::MAX));
    for member in members {
        sink.put_len_prefixed(member.port.as_bytes());
        sink.put_len_prefixed(member.namespace.as_bytes());
        sink.put_fixed(member.relation_id.as_bytes());
        sink.put_u32_le(member.schema_version.0);
        sink.put_fixed(member.logical_hash.0.as_bytes());
    }

    Ok(())
}

/// Names a snapshot by its membership.
///
/// # Errors
///
/// - [`SnapshotError::DuplicateParentRole`] and [`SnapshotError::DuplicateMemberPort`]:
///   a repeated role or port would make the frame ambiguous, and silently keeping one of
///   the two would make the ID depend on assembly order.
/// - [`SnapshotError::MissingModelParent`]: a case snapshot that does not name its model
///   is not a case.
///
/// ```
/// use pse_ids::{ContentHash, SnapshotFrame, SnapshotKind, snapshot_id};
///
/// let frame = SnapshotFrame {
///     registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
///     kind: SnapshotKind::Model,
///     parents: Vec::new(),
///     members: Vec::new(),
/// };
/// // An empty model snapshot is a snapshot; emptiness is a fact, not an omission.
/// assert!(snapshot_id(&frame).is_ok());
/// ```
pub fn snapshot_id(frame: &SnapshotFrame) -> Result<SnapshotId, SnapshotError> {
    let mut hasher = blake3::Hasher::new();
    write_frame(frame, &mut hasher)?;
    Ok(SnapshotId(ContentHash::from_bytes(
        *hasher.finalize().as_bytes(),
    )))
}

/// The `pse.snapshot.v2` preimage of `frame`, for a fixture that needs to see the bytes.
///
/// # Errors
///
/// The [`snapshot_id`] errors; the preimage of an invalid frame is not built.
pub fn snapshot_preimage(frame: &SnapshotFrame) -> Result<Vec<u8>, SnapshotError> {
    let mut bytes: Vec<u8> = Vec::new();
    write_frame(frame, &mut bytes)?;
    Ok(bytes)
}

/// The port name of a model or case member: `<namespace>/<32 lowercase hex digits>`
/// (blueprint §5.3 step 7).
///
/// The relation *ID*, not its name, because a relation rename must not change a port and
/// therefore must not change a snapshot ID.
///
/// ```
/// use pse_ids::{model_port_name, SemanticId};
///
/// assert_eq!(
///     model_port_name("authored", SemanticId::from_bytes([0xab; 16])),
///     "authored/abababababababababababababababab",
/// );
/// ```
pub fn model_port_name(namespace: &str, relation_id: SemanticId) -> String {
    format!("{namespace}/{}", relation_id.to_hex())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn member(port: &str, namespace: &str, tag: u8, version: u32) -> SnapshotMember {
        SnapshotMember {
            port: port.to_owned(),
            namespace: namespace.to_owned(),
            relation_id: SemanticId::from_bytes([tag; 16]),
            schema_version: SchemaVersion(version),
            logical_hash: LogicalHash(ContentHash::from_bytes([tag; 32])),
        }
    }

    fn parent(role: &str, tag: u8) -> SnapshotParent {
        SnapshotParent {
            role: role.to_owned(),
            snapshot_id: SnapshotId(ContentHash::from_bytes([tag; 32])),
        }
    }

    fn model_frame(members: Vec<SnapshotMember>) -> SnapshotFrame {
        SnapshotFrame {
            registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
            kind: SnapshotKind::Model,
            parents: Vec::new(),
            members,
        }
    }

    #[test]
    fn the_kinds_spell_themselves() {
        assert_eq!(SnapshotKind::Model.as_str(), "model");
        assert_eq!(SnapshotKind::Case.to_string(), "case");
        assert_eq!(SnapshotKind::Stage.as_str(), "stage");
        assert_eq!(SnapshotKind::Run.as_str(), "run");
    }

    #[test]
    fn a_port_name_is_a_namespace_and_a_relation_id() {
        assert_eq!(
            model_port_name("reference", SemanticId::NIL),
            format!("reference/{}", "0".repeat(32))
        );
    }

    #[test]
    fn the_preimage_starts_with_the_frozen_profile_string() {
        let preimage = snapshot_preimage(&model_frame(Vec::new()));
        let Ok(preimage) = preimage else {
            panic!("an empty model frame was refused");
        };
        let mut expected: Vec<u8> = Vec::new();
        expected.put_len_prefixed(SNAPSHOT_PROFILE.as_bytes());
        assert_eq!(preimage.get(..expected.len()), Some(expected.as_slice()));
        // version ‖ fingerprint ‖ kind ‖ parent count ‖ member count
        assert_eq!(preimage.len(), 8 + 15 + 32 + 8 + 5 + 8 + 8);
    }

    #[test]
    fn member_order_is_not_membership() {
        let ordered = model_frame(vec![
            member("authored/a", "authored", 1, 1),
            member("authored/b", "authored", 2, 1),
        ]);
        let reversed = model_frame(vec![
            member("authored/b", "authored", 2, 1),
            member("authored/a", "authored", 1, 1),
        ]);
        assert_eq!(snapshot_id(&ordered), snapshot_id(&reversed));
    }

    #[test]
    fn parent_order_is_not_membership_either() {
        let make = |parents: Vec<SnapshotParent>| SnapshotFrame {
            registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
            kind: SnapshotKind::Case,
            parents,
            members: vec![member("case/a", "case", 1, 1)],
        };
        assert_eq!(
            snapshot_id(&make(vec![parent("model", 3), parent("overlay", 4)])),
            snapshot_id(&make(vec![parent("overlay", 4), parent("model", 3)]))
        );
    }

    #[test]
    fn an_absent_member_is_a_different_snapshot() {
        let full = model_frame(vec![
            member("authored/a", "authored", 1, 1),
            member("authored/b", "authored", 2, 1),
        ]);
        let partial = model_frame(vec![member("authored/a", "authored", 1, 1)]);
        assert_ne!(snapshot_id(&full), snapshot_id(&partial));
    }

    #[test]
    fn every_framed_component_changes_the_identity() {
        let base = SnapshotFrame {
            registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
            kind: SnapshotKind::Case,
            parents: vec![parent("model", 3)],
            members: vec![member("case/a", "case", 1, 1)],
        };
        let baseline = snapshot_id(&base);

        let mut fingerprint = base.clone();
        fingerprint.registry_fingerprint = ContentHash::from_bytes([0x12; 32]);
        assert_ne!(snapshot_id(&fingerprint), baseline);

        let mut kind = base.clone();
        kind.kind = SnapshotKind::Stage;
        assert_ne!(snapshot_id(&kind), baseline);

        let mut role = base.clone();
        role.parents = vec![parent("model", 3), parent("overlay", 3)];
        assert_ne!(snapshot_id(&role), baseline);

        let mut version = base.clone();
        version.members = vec![member("case/a", "case", 1, 2)];
        assert_ne!(snapshot_id(&version), baseline);

        let mut namespace = base.clone();
        namespace.members = vec![member("case/a", "compiled", 1, 1)];
        assert_ne!(snapshot_id(&namespace), baseline);

        let mut content = base.clone();
        content.members = vec![SnapshotMember {
            logical_hash: LogicalHash(ContentHash::from_bytes([0x99; 32])),
            ..member("case/a", "case", 1, 1)
        }];
        assert_ne!(snapshot_id(&content), baseline);
    }

    #[test]
    fn a_split_port_name_cannot_masquerade_as_another() {
        // Length prefixes, so ("ab", "c") and ("a", "bc") are different members.
        let left = model_frame(vec![member("ab", "c", 1, 1)]);
        let right = model_frame(vec![member("a", "bc", 1, 1)]);
        assert_ne!(snapshot_id(&left), snapshot_id(&right));
    }

    #[test]
    fn duplicate_roles_and_ports_are_rejected() {
        let duplicate_parent = SnapshotFrame {
            registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
            kind: SnapshotKind::Stage,
            parents: vec![parent("input", 3), parent("input", 4)],
            members: Vec::new(),
        };
        assert_eq!(
            snapshot_id(&duplicate_parent),
            Err(SnapshotError::DuplicateParentRole {
                role: "input".to_owned()
            })
        );

        let duplicate_member = model_frame(vec![
            member("authored/a", "authored", 1, 1),
            member("authored/a", "authored", 2, 1),
        ]);
        assert_eq!(
            snapshot_id(&duplicate_member),
            Err(SnapshotError::DuplicateMemberPort {
                port: "authored/a".to_owned()
            })
        );
    }

    #[test]
    fn a_case_must_name_the_model_it_overlays() {
        let orphan = SnapshotFrame {
            registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
            kind: SnapshotKind::Case,
            parents: vec![parent("overlay", 4)],
            members: Vec::new(),
        };
        assert_eq!(snapshot_id(&orphan), Err(SnapshotError::MissingModelParent));

        let parented = SnapshotFrame {
            parents: vec![parent("model", 3), parent("overlay", 4)],
            ..orphan
        };
        assert!(snapshot_id(&parented).is_ok());
    }

    #[test]
    fn only_a_case_requires_a_model_parent() {
        for kind in [SnapshotKind::Model, SnapshotKind::Stage, SnapshotKind::Run] {
            let frame = SnapshotFrame {
                registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
                kind,
                parents: Vec::new(),
                members: Vec::new(),
            };
            assert!(snapshot_id(&frame).is_ok(), "{kind} was refused");
        }
    }

    proptest! {
        /// Permuting members and parents never changes the snapshot ID.
        #[test]
        fn permutation_does_not_change_the_identity(
            tags in proptest::collection::vec(0_u8..64, 1..8),
            roles in proptest::collection::vec(0_u8..64, 0..4),
            permutation_seed in 0_usize..32,
        ) {
            let mut members: Vec<SnapshotMember> = Vec::new();
            for tag in &tags {
                let port = format!("authored/{tag}");
                if members.iter().all(|existing| existing.port != port) {
                    members.push(member(&port, "authored", *tag, 1));
                }
            }
            let mut parents: Vec<SnapshotParent> = Vec::new();
            for tag in &roles {
                let role = format!("in{tag}");
                if parents.iter().all(|existing| existing.role != role) {
                    parents.push(parent(&role, *tag));
                }
            }

            let frame = SnapshotFrame {
                registry_fingerprint: ContentHash::from_bytes([0x11; 32]),
                kind: SnapshotKind::Stage,
                parents: parents.clone(),
                members: members.clone(),
            };

            // A cheap deterministic shuffle: rotate by the seed.
            let member_rotation = permutation_seed % members.len().max(1);
            members.rotate_left(member_rotation);
            if !parents.is_empty() {
                let parent_rotation = permutation_seed % parents.len();
                parents.rotate_left(parent_rotation);
            }
            let shuffled = SnapshotFrame { parents, members, ..frame.clone() };

            prop_assert_eq!(snapshot_id(&frame), snapshot_id(&shuffled));
        }

        /// A repeated port is refused however the members are ordered.
        #[test]
        fn a_repeated_port_is_always_refused(extra in 0_u8..64) {
            let frame = model_frame(vec![
                member("authored/x", "authored", 1, 1),
                member("authored/x", "authored", extra, 2),
            ]);
            prop_assert_eq!(
                snapshot_id(&frame),
                Err(SnapshotError::DuplicateMemberPort { port: "authored/x".to_owned() })
            );
        }
    }
}
