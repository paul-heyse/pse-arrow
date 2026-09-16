// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual selected policy owners and the engine context used by a producer.

use std::{collections::BTreeMap, sync::Arc};

use pse_ids::{MemoryReserver, SemanticId};

use super::control::{self, OwnedControl};
use crate::{CatalogError, Snapshot, session::SessionSemantics};

/// One selected row in an actual admitted policy relation.
#[derive(Clone, Debug)]
pub struct PolicyInput {
    /// The selected semantic key, not the relation's content hash.
    pub policy_id: SemanticId,
    /// Actual immutable owner of the selected policy relation.
    pub snapshot: Arc<Snapshot>,
    /// Exact member port in that snapshot.
    pub port: String,
}

/// Complete auxiliary inputs for a single producing invocation.
#[derive(Debug)]
pub struct InvocationContext {
    /// Exact committed source owner from which the invocation's original documents
    /// were loaded. This is independent of the pass's selected relational input ports.
    pub document_source: Option<Arc<Snapshot>>,
    /// Every explicitly selected policy, including selections yielding equal outputs.
    pub policies: BTreeMap<String, PolicyInput>,
    /// Actual sealed engine configuration; absent for a producer with no native plans.
    pub engine: Option<SessionSemantics>,
}

/// Shared invocation values and the lease for their retained allocation.
pub type OwnedInvocation = OwnedControl<InvocationContext>;

impl InvocationContext {
    /// Retain exact admitted policy handles and engine values, reserving before cloning.
    ///
    /// # Errors
    /// Invalid member selection, addressable-size overflow or allocation refusal.
    pub fn capture<'a>(
        policies: impl Iterator<Item = (&'a str, SemanticId, &'a Arc<Snapshot>, &'a str)> + Clone,
        document_source: Option<&Arc<Snapshot>>,
        engine: Option<&SessionSemantics>,
        reserver: &dyn MemoryReserver,
    ) -> Result<OwnedInvocation, CatalogError> {
        let slots =
            super::stage_owned::map_extent::<String, PolicyInput>(policies.clone().count())?;
        let bytes = policies.clone().try_fold(
            control::add(512, slots)?,
            |bytes, (role, _, snapshot, port)| {
                if !snapshot.relations().contains_key(port) {
                    return Err(super::verify::admission(
                        "invocation",
                        "policy member is absent",
                    ));
                }
                control::add(bytes, control::add(role.len(), port.len())?)
            },
        )?;
        let mut reservation = reserver.open("store:invocation");
        reservation.try_grow(control::add(bytes, engine_extent(engine)?)?)?;
        let mut bound = BTreeMap::new();
        for (role, policy_id, snapshot, port) in policies {
            if bound
                .insert(
                    role.to_owned(),
                    PolicyInput {
                        policy_id,
                        snapshot: Arc::clone(snapshot),
                        port: port.to_owned(),
                    },
                )
                .is_some()
            {
                return Err(super::verify::admission(
                    "invocation",
                    "duplicate selected policy role",
                ));
            }
        }
        Ok(OwnedControl::new(
            Self {
                document_source: document_source.cloned(),
                policies: bound,
                engine: engine.cloned(),
            },
            pse_ids::ReservationLease::new(reservation),
        ))
    }
}

pub(super) fn engine_extent(engine: Option<&SessionSemantics>) -> Result<usize, CatalogError> {
    engine.map_or(Ok(0), SessionSemantics::heap_extent)
}

/// Borrowed engine inputs, counted before their owned semantic snapshot is cloned.
pub(crate) fn engine_inputs_extent(
    engine_version_bytes: usize,
    arrow_version_bytes: usize,
    profile: &crate::session::EngineProfile,
    settings: &BTreeMap<String, Option<String>>,
    functions: &BTreeMap<String, Vec<String>>,
) -> Result<usize, CatalogError> {
    super::stage_owned::engine_parts_extent(
        engine_version_bytes,
        arrow_version_bytes,
        profile,
        settings,
        functions,
    )
}
