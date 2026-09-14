// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Accounted immutable metadata; public wire DTOs remain caller-owned values.
use crate::CatalogError;
use pse_ids::{MemoryReserver, Reservation, ReservationLease};
use serde::{Serialize, Serializer};
use std::{ops::Deref, sync::Arc};

#[derive(Debug)]
struct Retained<T> {
    value: T,
    _lease: Arc<ReservationLease>,
}
/// Immutable catalog metadata whose clones retain its actual data and reservation.
/// Cloning the borrowed underlying DTO explicitly creates caller-owned allocation.
#[derive(Debug)]
pub struct OwnedControl<T>(Arc<Retained<T>>);
impl<T> Clone for OwnedControl<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl<T> OwnedControl<T> {
    pub(crate) fn new(value: T, lease: Arc<ReservationLease>) -> Self {
        Self(Arc::new(Retained {
            value,
            _lease: lease,
        }))
    }
}
impl<T> Deref for OwnedControl<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0.value
    }
}
impl<T> AsRef<T> for OwnedControl<T> {
    fn as_ref(&self) -> &T {
        &self.0.value
    }
}
impl<T: PartialEq> PartialEq for OwnedControl<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}
impl<T: Eq> Eq for OwnedControl<T> {}
impl<T: Serialize> Serialize for OwnedControl<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.value.serialize(serializer)
    }
}

/// Count only allocation-driving byte/container extents; serde remains the parser.
pub(super) fn decode_reservation(
    bytes: &[u8],
    reserver: &dyn MemoryReserver,
    owner: &str,
) -> Result<Box<dyn Reservation>, CatalogError> {
    let mut quoted = false;
    let mut escaped = false;
    let mut nodes = 1usize;
    for &byte in bytes {
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else if byte == b'"' {
            quoted = true;
        } else if matches!(byte, b'{' | b'[' | b',') {
            nodes = add(nodes, 1)?;
        }
    }
    let size = add(mul(bytes.len(), 4)?, mul(nodes, 1024)?)?;
    let mut reservation = reserver.open(owner);
    reservation.try_grow(size)?;
    Ok(reservation)
}
pub(super) fn retain(
    mut reservation: Box<dyn Reservation>,
    bytes: usize,
) -> Result<Arc<ReservationLease>, CatalogError> {
    if bytes > reservation.size() {
        return Err(super::encode::overflow());
    }
    reservation.shrink(reservation.size() - bytes);
    Ok(ReservationLease::new(reservation))
}
pub(super) fn manifest_extent(value: &super::manifest::Manifest) -> Result<usize, CatalogError> {
    let mut bytes = add(size_of::<super::manifest::Manifest>(), 64)?;
    for text in [
        &value.manifest_version,
        &value.membership_profile,
        &value.created_at,
        &value.compiler.version,
        &value.toolchain.canonicalization,
    ] {
        bytes = add(bytes, text.capacity())?;
    }
    bytes = add(bytes, vector(&value.relations)?)?;
    for member in &value.relations {
        bytes = add(bytes, super::stage_owned::member_extent(member)?)?;
    }
    bytes = add(bytes, vector(&value.packages)?)?;
    for package in &value.packages {
        bytes = add(bytes, package.version.capacity())?;
    }
    bytes = add(bytes, vector(&value.compiler.passes)?)?;
    for pass in &value.compiler.passes {
        bytes = add(bytes, pass.version.capacity())?;
    }
    bytes = add(bytes, vector(&value.kernels)?)?;
    for kernel in &value.kernels {
        bytes = add(bytes, kernel.version.capacity())?;
    }
    bytes = add(bytes, vector(&value.semantic_parents)?)?;
    for parent in &value.semantic_parents {
        bytes = add(bytes, parent.role.capacity())?;
    }
    bytes = add(bytes, vector(&value.evidence)?)?;
    for evidence in &value.evidence {
        for text in [&evidence.kind, &evidence.codec_version, &evidence.path] {
            bytes = add(bytes, text.capacity())?;
        }
    }
    Ok(bytes)
}
pub(super) fn revision_extent(value: &super::sidecar::RevisionRef) -> Result<usize, CatalogError> {
    add(
        size_of::<super::sidecar::RevisionRef>() + 64,
        super::stage_owned::member_extent(value.artifact.member())?,
    )
}
pub(super) fn vector<T>(value: &Vec<T>) -> Result<usize, CatalogError> {
    mul(value.capacity(), size_of::<T>())
}
pub(super) fn add(left: usize, right: usize) -> Result<usize, CatalogError> {
    left.checked_add(right).ok_or_else(super::encode::overflow)
}
pub(super) fn mul(left: usize, right: usize) -> Result<usize, CatalogError> {
    left.checked_mul(right).ok_or_else(super::encode::overflow)
}

pub(super) fn snapshot_extent(value: &super::manifest::Manifest) -> Result<usize, CatalogError> {
    let rows = super::stage_owned::map_extent::<String, Arc<crate::LoadedRelation>>(
        value.relations.len(),
    )?;
    let parents = super::stage_owned::map_extent::<String, Arc<crate::Snapshot>>(
        value.semantic_parents.len(),
    )?;
    add(mul(manifest_extent(value)?, 2)?, add(rows, parents)?)
}

pub(super) fn own_member(
    member: super::manifest::RelationMember,
    reservation: Box<dyn Reservation>,
) -> Result<OwnedControl<super::manifest::RelationMember>, CatalogError> {
    let retained = add(mul(super::stage_owned::member_extent(&member)?, 3)?, 1024)?;
    Ok(OwnedControl::new(member, retain(reservation, retained)?))
}
