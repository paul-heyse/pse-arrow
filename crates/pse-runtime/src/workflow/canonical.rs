// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deployment composition for the mandatory canonical scientific substrate.

use crate::math::portable::QualifiedProducer;
use pse_ids::ContentHash;
use pse_operations::canonical::CanonicalStore;

/// Available deployment attestation. It describes the executable containing the
/// library; it is deliberately distinct from relevant compilation producer keys.
#[derive(Clone, Copy, Debug)]
pub struct OuterAttestation {
    /// Complete source observation, absent for an installation without its authored checkout.
    pub source: Option<ContentHash>,
    /// Observed build/deployed artifact context recorded by the composition root.
    pub build: ContentHash,
}

/// One configured canonical store and independently qualified reuse producer.
#[derive(Clone, Debug)]
pub struct CanonicalDeployment {
    store: CanonicalStore,
    attestation: OuterAttestation,
    producer: Option<QualifiedProducer>,
}
impl CanonicalDeployment {
    /// Attach an already opened, interpretation-checked canonical database.
    /// An incomplete producer permits ordinary admission while refusing persisted reuse.
    pub fn new(
        store: CanonicalStore,
        attestation: OuterAttestation,
        producer: Option<QualifiedProducer>,
    ) -> Self {
        Self {
            store,
            attestation,
            producer,
        }
    }
    /// The sole canonical scientific storage boundary.
    pub fn store(&self) -> &CanonicalStore {
        &self.store
    }
    /// Eligible relevant mathematical producer, when deployment inputs were qualified.
    pub fn producer(&self) -> Option<&QualifiedProducer> {
        self.producer.as_ref()
    }
    /// Complete executable provenance used by result lineage and publications.
    pub const fn attestation(&self) -> OuterAttestation {
        self.attestation
    }
}
impl super::Runtime {
    /// Canonical scientific storage shared by authoring, preparation and retained queries.
    pub fn canonical_store(&self) -> &CanonicalStore {
        self.canonical.store()
    }
}
