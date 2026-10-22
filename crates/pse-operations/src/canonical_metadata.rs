// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Endpoint-derived transport/intermediate metadata extents, distinct from IPC.

use pse_schema::catalog::substrate as wire;

/// Canonical metadata layouts. The wire preflight enforces these bounds before
/// SurrealDB allocates owned Values; typed codecs retain their semantic checks.
#[derive(Clone, Copy, Debug)]
pub enum MetadataResponse {
    /// Release/renew transactions return at most the guard and protection rows,
    /// statement acknowledgements, and bounded errors; they carry no payloads.
    Protection,
    /// At most two header rows, each below 32 declared fields. Run headers carry
    /// request/source-selection bytes (up to the transport limit together).
    Header,
    /// Header/manifest/protection selection, including 256 CBOR descriptors.
    Selection,
    /// A 64-row descriptor or analysis page, each below 32 declared fields.
    Page,
    /// A 64-row block page, including two optional exact IEEE coordinate values
    /// per row. Each native Float64 wrapper carries eight bytes of original bits.
    BlockPage,
    /// One protected page of at most 64 blocks and 64 exact batch descriptors.
    /// The two arrays share one aggregate wire/allocation allowance.
    JoinedBlockPage,
    /// A study page shares at most 128 immediate-premise rows between 64
    /// candidates plus one notification source, with at most 65 point identities in each membership list.
    /// Encoded scope admission also bounds this graph before it leaves the server.
    StudyScopePage,
    /// A 64-row scalar/dense output page, including encoded registry keys.
    OutputPage,
}
impl MetadataResponse {
    /// Limits include 32 statement envelopes of at most eight fields, errors,
    /// and the root envelope. Keys are counted separately from their values.
    pub fn limits(self) -> surrealdb::opt::ResponseLimits {
        let rows = match self {
            Self::Protection | Self::Header | Self::Selection => 2,
            Self::Page | Self::BlockPage => crate::canonical_results::PAGE,
            Self::JoinedBlockPage => 2 * crate::canonical_results::PAGE,
            Self::StudyScopePage => 128,
            Self::OutputPage => 3 * crate::canonical_results::PAGE,
        };
        let nodes = if matches!(self, Self::StudyScopePage) {
            // Shared row maps, candidate maps, membership lists and cursor/fence
            // maps; each map's keys consume nodes as well as its values.
            18_000
        } else {
            512 + rows * 32 * 2
        };
        let strings =
            (rows * 32 * wire::METADATA_ID_BYTES + 512 * 64).min(wire::RESULT_MESSAGE_BYTES);
        let blobs = match self {
            Self::Protection | Self::Page => 0,
            Self::BlockPage | Self::JoinedBlockPage => {
                crate::canonical_results::PAGE * (pse_schema::catalog::substrate::RESULT_STATISTICS_BYTES
                    + 2 * size_of::<f64>())
            }
            Self::Header | Self::Selection | Self::OutputPage | Self::StudyScopePage => wire::RESULT_MESSAGE_BYTES,
        };
        surrealdb::opt::ResponseLimits {
            // Native protobuf messages and repeated slots need at most 128 bytes
            // per admitted node beyond copied strings for this scalar ACK layout.
            wire_bytes: if matches!(self, Self::Protection) {
                nodes * 128 + strings
            } else {
                wire::RESULT_MESSAGE_BYTES
            },
            nodes,
            depth: 16,
            container_items: match self {
                Self::Protection => 32,
                Self::Selection => crate::canonical_execution::RESULT_SETS,
                Self::StudyScopePage => 128,
                // source-selection readers admit up to 65 sources plus a sentinel.
                _ => 66,
            },
            string_bytes: strings,
            string_length: wire::METADATA_ID_BYTES,
            blob_bytes: blobs,
            blob_length: match self {
                Self::BlockPage | Self::JoinedBlockPage => wire::RESULT_STATISTICS_BYTES,
                Self::StudyScopePage => 128 * 1024,
                _ => crate::canonical::PAYLOAD_BYTES,
            },
        }
    }
    /// Raw frame plus four simultaneous owned/intermediate representations.
    /// 256 bytes per visited value/key covers Value slots, geometric Vec growth,
    /// BTreeMap nodes and typed row/index copies at the pinned SDK. String/blob
    /// occurrences are separately prepaid, even for repeated wire offsets.
    pub fn allocation_bytes(self) -> usize {
        let limits = self.limits();
        limits.wire_bytes
            + 4 * (limits.nodes * 256 + limits.string_bytes + limits.blob_bytes)
            + 64 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_layouts_fit_the_declared_row_field_envelope() {
        let registry = pse_schema::registry().unwrap();
        for name in [
            "canonical_runs",
            "canonical_attempts",
            "canonical_result_manifests",
            "canonical_result_blocks",
            "canonical_result_batches",
            "canonical_result_block_outputs",
            "canonical_file_instances",
            "canonical_artifacts",
            "canonical_artifact_blocks",
            "canonical_analyses",
            "canonical_analysis_nodes",
            "canonical_analysis_edges",
        ] {
            let relation = registry.relation(&format!("runtime.{name}")).unwrap();
            assert!(
                relation.columns.len() < 32,
                "{name}: fields plus native id must fit metadata preflight"
            );
        }
    }
    #[test]
    fn metadata_allowances_are_separate_from_physical_files() {
        for profile in [
            MetadataResponse::Protection,
            MetadataResponse::Header,
            MetadataResponse::Selection,
            MetadataResponse::Page,
            MetadataResponse::BlockPage,
            MetadataResponse::JoinedBlockPage,
            MetadataResponse::StudyScopePage,
            MetadataResponse::OutputPage,
        ] {
            assert!(profile.allocation_bytes() < 2 * wire::RESULT_BLOCK_BYTES);
            assert!(profile.limits().nodes >= 512 + 2 * 32 * 2);
        }
        assert_eq!(MetadataResponse::Page.limits().blob_bytes, 0);
        assert_eq!(
            MetadataResponse::BlockPage.limits().blob_bytes,
            crate::canonical_results::PAGE
                * (pse_schema::catalog::substrate::RESULT_STATISTICS_BYTES + 2 * size_of::<f64>())
        );
        assert_eq!(
            MetadataResponse::BlockPage.limits().blob_length,
            pse_schema::catalog::substrate::RESULT_STATISTICS_BYTES
        );
        assert_eq!(MetadataResponse::Protection.limits().blob_bytes, 0);
        assert_eq!(MetadataResponse::JoinedBlockPage.limits().wire_bytes, wire::RESULT_MESSAGE_BYTES);
        assert_eq!(MetadataResponse::StudyScopePage.limits().container_items, 128);
        assert!(MetadataResponse::JoinedBlockPage.limits().nodes > MetadataResponse::BlockPage.limits().nodes);
        assert!(
            MetadataResponse::Protection.allocation_bytes()
                < MetadataResponse::Header.allocation_bytes()
        );
    }
}
