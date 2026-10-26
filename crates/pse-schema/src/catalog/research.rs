// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable research organization over original admitted source and result owners.
use super::declarations::{column, canonical_relation as relation};
use super::substrate::NativeIndex;
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
/// Bounded project discovery without decoding measurement payloads.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex {
        name: "research_dataset_project",
        table: "canonical_research_datasets",
        fields: &["project", "key"],
        unique: false,
    },
    NativeIndex {
        name: "research_selection_dataset",
        table: "canonical_research_selections",
        fields: &["dataset", "key"],
        unique: false,
    },
];
pub(super) fn declare(b: &mut RegistryBuilder) {
    let text = || T::native(D::Utf8);
    let bytes = || T::native(D::Binary);
    for (name, fields, doc) in [
        (
            "canonical_research_datasets",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ResearchDataset),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision),
                column("physical_context", text()),
                column("context", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectContext),
                column("parent", text()).with_canonical_key(crate::model::CanonicalKeyKind::ResearchDataset).optional(),
                column("role", text()),
                column("members", bytes()),
                column("request", bytes()),
            ],
            "Immutable version referencing admitted typed data objects and exact context. Members contain identities and attribute selections, never a competing measurement payload.",
        ),
        (
            "canonical_research_selections",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ResearchSelection),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project),
                column("dataset", text()).with_canonical_key(crate::model::CanonicalKeyKind::ResearchDataset),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision),
                column("physical_context", text()),
                column("context", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectContext),
                column("fit", text()),
                column("selection", bytes()),
                column("request", bytes()),
            ],
            "Frozen original fit observation and calibration selection; exclusions, reasons, original-to-derived correspondence and uncertainty treatment reference admitted attributes.",
        ),
        (
            "canonical_research_publications",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ResearchPublication),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project),
                column("selection", text()).with_canonical_key(crate::model::CanonicalKeyKind::ResearchSelection),
                column("run", text()).with_canonical_key(crate::model::CanonicalKeyKind::Run),
                column("attempt", text()).with_canonical_key(crate::model::CanonicalKeyKind::Attempt),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision),
                column("parent", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision),
                column("mapping", bytes()),
                column("request", bytes()),
            ],
            "Atomic source publication receipt referencing original qualified fitted cells and admitted target source. No fitted values are copied into this receipt.",
        ),
    ] {
        relation(b, N::Runtime, name, S::Derived, &["key"], fields, doc);
    }
}
