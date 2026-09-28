// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Minting store identities (ADR-0114 Outcome 13).

use pse_ids::SemanticId;
use uuid::Uuid;

/// Mint a new identity on the runtime's `UUIDv7` path, as the typed id the caller names
/// (an `AttemptId`, a `JobId`, ...) or as a bare [`SemanticId`]. The store itself mints no
/// domain identity.
pub fn mint_id<T: From<SemanticId>>() -> T {
    T::from(SemanticId::from_bytes(*Uuid::now_v7().as_bytes()))
}

#[cfg(test)]
mod ids_unit {
    use super::*;
    use pse_model::generated::identities::AttemptId;

    #[test]
    fn minted_ids_are_version_seven_and_typed() {
        let id: SemanticId = mint_id();
        assert_eq!(Uuid::from_bytes(*id.as_bytes()).get_version_num(), 7);
        let attempt: AttemptId = mint_id();
        assert_ne!(attempt.as_id(), id);
    }
}
