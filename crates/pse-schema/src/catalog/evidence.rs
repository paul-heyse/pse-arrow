// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Disposable tooling projections. Original host artifacts retain authority and custody.
use super::declarations::{column, enumeration, relation};
use crate::{RegistryBuilder, model::{FieldContract as T, Namespace as N, SnapshotClass as S}};
use arrow_schema::DataType as D;
/// Tooling interpretation, independent from scientific interpretation.
pub const INTERPRETATION: &str = "pse-tooling-evidence-v1";
/// Native indexes installed only in the tooling database.
pub const INDEXES: &[super::substrate::NativeIndex] = &[
    super::substrate::NativeIndex { name:"evidence_fact_membership", table:"evidence_facts", fields:&["generation","member"], unique:true },
    super::substrate::NativeIndex { name:"evidence_fact_selection", table:"evidence_facts", fields:&["generation","kind","status","member"], unique:false },
    super::substrate::NativeIndex { name:"evidence_snapshot_identity", table:"evidence_generations", fields:&["authority","codec","source_id","snapshot_digest"], unique:true },
];
pub(super) fn declare(b: &mut RegistryBuilder) {
    enumeration(b,"EvidenceCoverage",["complete","partial"]);
    enumeration(b,"EvidenceGenerationState",["staged","active"]);
    let text=|| T::native(D::Utf8);
    let count=|| T::native(D::UInt64);
    relation(b,N::Runtime,"evidence_interpretations",S::Derived,&["key"],vec![
        column("key",text()),column("interpretation",text()),column("schema_digest",text()),column("issuing_store",text()),column("service_instance_id",text()),column("active_generation",text()).optional(),
    ],"Tooling-only compatibility and issuing identity. Active generation grants no raw artifact lifetime or scientific authority.");
    relation(b,N::Runtime,"evidence_generations",S::Derived,&["key"],vec![
        column("key",text()),column("authority",text()),column("codec",text()),column("source_id",text()),column("snapshot_digest",text()),column("content_digest",text()),column("coverage",T::enumeration("EvidenceCoverage")),column("expected_members",count()),column("state",T::enumeration("EvidenceGenerationState")),
    ],"An immutable authentic-source snapshot identity and reconciled compact projection. Staged imports remain distinct from complete activated generations; partial source coverage is explicit.");
    relation(b,N::Runtime,"evidence_facts",S::Derived,&["key"],vec![
        column("key",text()),column("generation",text()),column("member",text()),column("kind",text()),column("status",text()),column("crate_name",text()).optional(),column("test_id",text()).optional(),column("detail_ref",text()).optional(),
    ],"Compact owner-adapted fact. Original status and membership remain explicit; detail references are descriptive and grant neither file custody nor protection.");
}
