// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit test-support fixture: stream original IPC, never derive fake indexes.
use super::*;
use crate::canonical_execution::{ResultSetDescriptor, execution_attempt_key};
use pse_ids::FramedHasher;
use pse_model::generated::runtime::{
    canonical_attempts::Row as Attempt, canonical_result_manifests::Row as Manifest,
    canonical_result_sets::Row as Set, canonical_roots::Row as Root, canonical_runs::Row as Run,
};
pub(super) async fn seed(
    client: &mut OfflineClient,
    producer: &FixtureProducer<'_>,
) -> Result<(), CanonicalError> {
    let run = "original-run";
    let attempt = execution_attempt_key(run, "sealed-original");
    let failed = execution_attempt_key(run, "failed-original");
    let partial = execution_attempt_key(run, "partial-original");
    let cancelled = execution_attempt_key(run, "cancelled-original");
    let retry_run = "failed-only-original-run";
    let retry = execution_attempt_key(retry_run, "failed-only-original");
    macro_rules! save {
        ($table:literal,$row:expr,$encode:path) => {{
            let row = $row;
            client
                .compare_create($table, &row.key, $encode(&row)?)
                .await?;
        }};
    }
    macro_rules! save_relation {
        ($table:literal,$row:expr,$encode:path) => {{
            let row = $row;
            client
                .compare_create_relation($table, &row.key, $encode(&row)?)
                .await?;
        }};
    }
    save!(
        "canonical_guards",
        pse_model::generated::runtime::canonical_guards::Row {
            key: "retention:fixture-problem".into(),
            generation: 1,
            incarnation: Some("original-retention-owner".into()),
            analysis_creation_closed_through: None,
            artifact_creation_closed_through: None,
            protection_closed_through: None
        },
        wire::encode_canonical_guards
    );
    save!(
        "canonical_runs",
        Run {
            key: run.into(),
            problem: "fixture-problem".into(),
            revision: "original-revision".into(),
            source_sequence: 1,
            sequence: 1,
            request: b"original-run-request".to_vec().into(),
            source_selection: b"original-source-selection".to_vec().into(),
            attestation: b"original-attestation".to_vec().into(),
            interpretation: "pse.substrate.v3".into(),
            current_generation: 4,
            current_attempt: Some(attempt.clone()),
            cancelled: false,
            terminal_attempt: Some(attempt.clone()),
            terminal_class: Some("succeeded".into())
        },
        wire::encode_canonical_runs
    );
    save!(
        "canonical_roots",
        Root {
            key: format!("run:{run}"),
            problem: "fixture-problem".into(),
            revision: "original-revision".into(),
            sequence: 1,
            owner_kind: "run".into(),
            owner: run.into()
        },
        wire::encode_canonical_roots
    );
    save_relation!(
        "canonical_run_sources",
        pse_model::generated::runtime::canonical_run_sources::Row {
            key: format!("{run}:original-revision"),
            run: run.into(),
            revision: "original-revision".into()
        },
        wire::encode_canonical_run_sources
    );
    save_relation!(
        "canonical_problem_runs",
        pse_model::generated::runtime::canonical_problem_runs::Row {
            key: "fixture-problem:1".into(),
            problem: "fixture-problem".into(),
            run: run.into()
        },
        wire::encode_canonical_problem_runs
    );
    for (key, generation, outcome, manifest) in [
        (&partial, 1, "partial", Some(partial.clone())),
        (&cancelled, 2, "cancelled", Some(cancelled.clone())),
        (&failed, 3, "failed", None),
        (&attempt, 4, "succeeded", Some(attempt.clone())),
    ] {
        save!(
            "canonical_attempts",
            Attempt {
                key: key.clone(),
                run: run.into(),
                generation,
                claim_operation: format!("claim-{generation}"),
                request: format!("original-claim-{generation}").into_bytes().into(),
                worker: "original-worker".into(),
                expires_at: 0,
                ingestion_open: false,
                closed: true,
                close_generation: Some(generation),
                terminal: true,
                outcome: Some(outcome.into()),
                closed_manifest: manifest,
                completion: None,
                seed_manifest: None,
                finalization_study: None
            },
            wire::encode_canonical_attempts
        );
    }
    save!(
        "canonical_runs",
        Run {
            key: retry_run.into(),
            problem: "fixture-problem".into(),
            revision: "original-revision".into(),
            source_sequence: 1,
            sequence: 2,
            request: b"failed-only-request".to_vec().into(),
            source_selection: b"failed-only-selection".to_vec().into(),
            attestation: b"failed-only-attestation".to_vec().into(),
            interpretation: "pse.substrate.v3".into(),
            current_generation: 1,
            current_attempt: Some(retry.clone()),
            cancelled: false,
            terminal_attempt: Some(retry.clone()),
            terminal_class: Some("failed".into())
        },
        wire::encode_canonical_runs
    );
    save!(
        "canonical_roots",
        Root {
            key: format!("run:{retry_run}"),
            problem: "fixture-problem".into(),
            revision: "original-revision".into(),
            sequence: 1,
            owner_kind: "run".into(),
            owner: retry_run.into()
        },
        wire::encode_canonical_roots
    );
    save!(
        "canonical_attempts",
        Attempt {
            key: retry.clone(),
            run: retry_run.into(),
            generation: 1,
            claim_operation: "failed-only-original".into(),
            request: b"failed-only-claim".to_vec().into(),
            worker: "original-worker".into(),
            expires_at: 0,
            ingestion_open: false,
            closed: true,
            close_generation: Some(1),
            terminal: true,
            outcome: Some("failed".into()),
            closed_manifest: None,
            completion: None,
            seed_manifest: None,
            finalization_study: None
        },
        wire::encode_canonical_attempts
    );
    let mut total = 0_u64;
    let mut descriptors: BTreeMap<String, Vec<ResultSetDescriptor>> = BTreeMap::new();
    for (attempt_key, trajectory, count, complete) in [
        (&attempt, false, 80, true),
        (&attempt, true, 4, true),
        (&failed, false, 1, false),
        (&partial, false, 1, true),
        (&cancelled, false, 1, true),
        (&retry, false, 1, false),
    ] {
        let mut digest = FramedHasher::new(Frame::CanonicalPayloadV1);
        digest.str("pse.execution.batch-selection.v1");
        let mut origin = 0;
        let mut set = None;
        for ordinal in 0..count {
            let (batch, block, outputs) = producer(trajectory, attempt_key, ordinal, origin)?;
            total = total
                .checked_add(batch.payload.as_slice().len() as u64)
                .ok_or(CanonicalError::PayloadLimit)?;
            if set.as_ref().is_some_and(|key| key != &batch.result_set) {
                return Err(invalid("fixture producer changed relation"));
            }
            set = Some(batch.result_set.clone());
            origin += batch.row_count;
            digest
                .str(&batch.key)
                .str(&batch.digest)
                .u64(batch.row_count);
            save!(
                "canonical_result_batches",
                batch,
                wire::encode_canonical_result_batches
            );
            save!(
                "canonical_result_blocks",
                block.clone(),
                wire::encode_canonical_result_blocks
            );
            for output in outputs {
                save!(
                    "canonical_result_block_outputs",
                    output,
                    wire::encode_canonical_result_block_outputs
                );
            }
            if ordinal + 1 == count {
                let set = Set {
                    key: set.clone().ok_or_else(|| invalid("fixture set absent"))?,
                    attempt: attempt_key.clone(),
                    name: block.output,
                    interpretation: "pse.substrate.v3".into(),
                    next_ordinal: count,
                    row_count: origin,
                };
                if complete {
                    descriptors
                        .entry(attempt_key.clone())
                        .or_default()
                        .push(ResultSetDescriptor {
                            key: set.key.clone(),
                            name: set.name.clone(),
                            batch_count: count,
                            row_count: origin,
                            batches_digest: digest.clone().finish_hash().to_hex(),
                        });
                }
                save!(
                    "canonical_result_sets",
                    set,
                    wire::encode_canonical_result_sets
                );
            }
        }
    }
    if total <= 16 * 1024 * 1024 {
        return Err(invalid(
            "general fixture did not exceed old whole-history snapshot bound",
        ));
    }
    for (owner, mut descriptors) in descriptors {
        descriptors.sort_by(|left, right| left.key.cmp(&right.key));
        let bytes = serde_json::to_vec(&descriptors).map_err(io)?;
        let mut digest = FramedHasher::new(Frame::CanonicalPayloadV1);
        digest.str("pse.execution.result-manifest.v1").part(&bytes);
        let generation = if owner == partial {
            1
        } else if owner == cancelled {
            2
        } else {
            4
        };
        save!(
            "canonical_result_manifests",
            Manifest {
                key: owner.clone(),
                attempt: owner,
                generation,
                digest: digest.finish_hash().to_hex(),
                descriptors: bytes.into()
            },
            wire::encode_canonical_result_manifests
        );
    }
    save!(
        "canonical_execution_operations",
        pse_model::generated::runtime::canonical_execution_operations::Row {
            key: "original-closed-receipt".into(),
            run: run.into(),
            attempt: Some(attempt.clone()),
            kind: "seal".into(),
            request: b"original-seal-request".to_vec().into(),
            result: b"original-seal-result".to_vec().into()
        },
        wire::encode_canonical_execution_operations
    );
    save!(
        "canonical_studies",
        pse_model::generated::runtime::canonical_studies::Row {
            key: "original-study".into(),
            run: run.into(),
            problem: "fixture-problem".into(),
            revision: "original-revision".into(),
            metadata: b"original-study-metadata".to_vec().into(),
            interpretation: "pse.substrate.v3".into(),
            point_count: 1,
            next_ordinal: 1,
            active: true,
            cancelled: false,
            generation: 7,
            terminal: true
        },
        wire::encode_canonical_studies
    );
    save!(
        "canonical_study_points",
        pse_model::generated::runtime::canonical_study_points::Row {
            key: "original-study:0".into(),
            study: "original-study".into(),
            ordinal: 0,
            occurrence: 13,
            seed_solution: None,
            policy: b"original-policy".to_vec().into(),
            descriptor: b"original-descriptor".to_vec().into(),
            facts: b"original-facts".to_vec().into(),
            outcome: Some(b"original-outcome".to_vec().into()),
            run: run.into(),
            revision: 1,
            assigned: true,
            settled: true,
            attempt: Some(attempt.clone()),
            start: Some(b"original-start".to_vec().into())
        },
        wire::encode_canonical_study_points
    );
    save!(
        "canonical_analyses",
        pse_model::generated::runtime::canonical_analyses::Row {
            key: "original-analysis".into(),
            revision: "original-revision".into(),
            method: "original-method".into(),
            configuration: b"original-analysis-config".to_vec().into(),
            input_digest: "original-input-digest".into(),
            primary_problem: "fixture-problem".into(),
            primary_authority: "original-retention-owner".into(),
            creation_nonce: "original-nonce".into(),
            creation_request_digest: "original-creation".into(),
            creation_expires_at: 0,
            interpretation: "pse.substrate.v3".into(),
            node_count: 1,
            edge_count: 0,
            active: true,
            retiring: false
        },
        wire::encode_canonical_analyses
    );
    save!(
        "canonical_analysis_nodes",
        pse_model::generated::runtime::canonical_analysis_nodes::Row {
            key: "original-analysis:node".into(),
            analysis: "original-analysis".into(),
            semantic: "original-semantic".into(),
            kind: "original-kind".into()
        },
        wire::encode_canonical_analysis_nodes
    );
    save!(
        "canonical_artifacts",
        pse_model::generated::runtime::canonical_artifacts::Row {
            key: "original-artifact".into(), kind:"analysis".into(),
            revision:"original-revision".into(), primary_problem:"fixture-problem".into(),
            primary_authority:"original-retention-owner".into(), creation_nonce:"original-artifact-nonce".into(),
            creation_expires_at:0,creation_request_digest:"original-artifact-request".into(),
            interpretation:"pse.substrate.v3".into(),provenance:vec![1].into(),block_count:1,payload_bytes:1,active:true,retiring:false
        },
        wire::encode_canonical_artifacts
    );
    save_relation!(
        "canonical_analysis_inputs",
        pse_model::generated::runtime::canonical_analysis_inputs::Row {
            key: "original-analysis:input".into(),
            analysis: "original-analysis".into(),
            artifact: "original-artifact".into()
        },
        wire::encode_canonical_analysis_inputs
    );
    save!(
        "canonical_roots",
        Root {
            key: "analysis:original-analysis".into(),
            problem: "fixture-problem".into(),
            revision: "original-revision".into(),
            sequence: 1,
            owner_kind: "analysis".into(),
            owner: "original-analysis".into()
        },
        wire::encode_canonical_roots
    );
    let retired = "retired-original-run";
    let retired_attempt = execution_attempt_key(retired, "retired-original-claim");
    save!(
        "canonical_runs",
        Run {
            key: retired.into(),
            problem: "fixture-problem".into(),
            revision: "retired-source-no-longer-retained".into(),
            source_sequence: 0,
            sequence: 0,
            request: b"retired-request".to_vec().into(),
            source_selection: b"retired-selection".to_vec().into(),
            attestation: b"retired-attestation".to_vec().into(),
            interpretation: "pse.substrate.v3".into(),
            current_generation: 2,
            current_attempt: Some(retired_attempt.clone()),
            cancelled: true,
            terminal_attempt: Some(retired_attempt.clone()),
            terminal_class: Some("cancelled".into())
        },
        wire::encode_canonical_runs
    );
    save!(
        "canonical_attempts",
        Attempt {
            key: retired_attempt.clone(),
            run: retired.into(),
            generation: 2,
            claim_operation: "retired-original-claim".into(),
            request: b"retired-claim-request".to_vec().into(),
            worker: "retired-worker".into(),
            expires_at: 0,
            ingestion_open: false,
            closed: true,
            close_generation: Some(2),
            terminal: true,
            outcome: Some("cancelled".into()),
            closed_manifest: Some(retired_attempt),
            completion: None,
            seed_manifest: None,
            finalization_study: None
        },
        wire::encode_canonical_attempts
    );
    save!(
        "canonical_result_retirements",
        pse_model::generated::runtime::canonical_result_retirements::Row {
            key: retired.into(),
            run: retired.into(),
            after_generation: 2,
            complete: true
        },
        wire::encode_canonical_result_retirements
    );
    save!(
        "canonical_study_retirements",
        pse_model::generated::runtime::canonical_study_retirements::Row {
            key: "original-study".into(),
            study: "original-study".into()
        },
        wire::encode_canonical_study_retirements
    );
    Ok(())
}
