// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native-client controls for the supervisor's offline recovery journey.

use anyhow::{Context, Result, ensure};
use pse_model::generated::runtime::canonical_products::Row as Product;
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore, ObjectEdit, ObjectVersion},
    canonical_selection::SelectedRead,
    generated::surreal::INTERPRETATION,
};
use std::{path::Path, sync::Arc, time::Duration};

/// Retain an unissued, sealed request outside the backup, then replay it through
/// the public creation API on the restored physical store. Authority is checked
/// before the creation window, so an expired request cannot mask this refusal.
pub(crate) async fn creation_authority(state: &Path, intent: &Path, seed: bool) -> Result<()> {
    use pse_operations::canonical_analyses::seal_analysis_request;
    let store = CanonicalStore::connect(&CanonicalOptions::from_state(state)?).await?;
    store.open().await?;
    let revision = store
        .revision(FIRST)
        .await?
        .context("creation source missing")?;
    let mut fresh = store
        .new_analysis(
            &revision,
            "recovery-authority.v1",
            vec![],
            "empty".into(),
            0,
            0,
        )
        .await?;
    seal_analysis_request(&mut fresh, std::slice::from_ref(&revision), &[], &[], &[]);
    if seed {
        let native = pse_operations::generated::surreal::encode_canonical_analyses(&fresh)?;
        std::fs::write(intent, serde_json::to_vec(&native)?)?;
    } else {
        let stale = pse_operations::generated::surreal::decode_canonical_analyses(
            serde_json::from_slice(&std::fs::read(intent)?)?,
        )?;
        ensure!(
            stale.primary_authority != fresh.primary_authority,
            "restored physical creation authority was not rotated"
        );
        let refused = store
            .persist_analysis(&stale, std::slice::from_ref(&revision), &[], &[], &[])
            .await;
        let error = refused
            .err()
            .context("stale physical creation intent was accepted")?;
        ensure!(
            format!("{error:?}").contains("analysis primary authority unavailable"),
            "stale creation refused for a different reason: {error}"
        );
        let admitted = store
            .persist_analysis(&fresh, &[revision], &[], &[], &[])
            .await?;
        ensure!(
            admitted.active,
            "fresh physical creation authority did not activate"
        );
    }
    Ok(())
}

const PROBLEM: &str = "native-recovery-control";
const FIRST: &str = "native-recovery-first";
const SECOND: &str = "native-recovery-second";
const LARGE_BYTES: usize = 5 * 1024 * 1024 + 17;

const ANALYSIS_RECEIPT: &str = "recovery-retained-analysis";
const ANALYSIS_RECEIPT_VERSION: &str = "recovery-retained-analysis-v1";
const ARTIFACT_BYTES: &[u8] =
    br#"{"minus_zero_bits":"8000000000000000","source":"historical-first","unit":"dimensionless"}"#;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedScience {
    analysis: Vec<u8>,
    artifact: Vec<u8>,
}

fn analysis_graph(
    header: &pse_operations::canonical_analyses::Analysis,
    artifact: &str,
) -> (
    Vec<pse_operations::canonical_analyses::AnalysisNode>,
    Vec<pse_operations::canonical_analyses::AnalysisEdge>,
) {
    use pse_operations::canonical_analyses::{AnalysisEdge, AnalysisNode};
    let source = format!("{}:node:0", header.key);
    let target = format!("{}:node:1", header.key);
    let nodes = vec![
        AnalysisNode {
            key: source.clone(),
            analysis: header.key.clone(),
            semantic: "historical-x".into(),
            kind: "variable".into(),
        },
        AnalysisNode {
            key: target.clone(),
            analysis: header.key.clone(),
            semantic: "historical-result".into(),
            kind: "result".into(),
        },
    ];
    let edges = vec![AnalysisEdge {
        key: format!("{}:edge:0", header.key),
        analysis: header.key.clone(),
        source,
        target,
        kind: "recorded-evidence".into(),
        evidence_artifact: Some(artifact.into()),
        evidence_block: Some(0),
        evidence_row: None,
        evidence_field: None,
    }];
    (nodes, edges)
}

async fn seed_retained_science(
    store: &CanonicalStore,
    revision: &pse_operations::canonical::Revision,
) -> Result<Vec<u8>> {
    use pse_operations::{
        canonical_analyses::seal_analysis_request,
        canonical_artifacts::{artifact_block, seal_artifact_request},
    };
    let mut artifact = store
        .new_artifact(
            revision,
            "recovery-json-capsule",
            vec![0, 255, 1],
            1,
            u64::try_from(ARTIFACT_BYTES.len())?,
        )
        .await?;
    let blocks = vec![artifact_block(
        &artifact.key,
        0,
        "historical-exact-bits",
        ARTIFACT_BYTES.to_vec(),
        "recovery-original-json-v1",
        1,
    )];
    seal_artifact_request(
        &mut artifact,
        std::slice::from_ref(revision),
        &[],
        &blocks,
        "user",
        "recovery-fixture",
    );
    let artifact_intent = artifact.clone();
    let artifact = store
        .persist_artifact(
            &artifact,
            std::slice::from_ref(revision),
            &[],
            &blocks,
            "user",
            "recovery-fixture",
        )
        .await?;
    let input = store.read_artifact(&artifact.key).await?;
    let mut analysis = store
        .new_analysis(
            revision,
            "recovery-retained-graph.v1",
            vec![0, 255, 1],
            blocks[0].digest.clone(),
            2,
            1,
        )
        .await?;
    let (nodes, edges) = analysis_graph(&analysis, &artifact.key);
    seal_analysis_request(
        &mut analysis,
        std::slice::from_ref(revision),
        std::slice::from_ref(&input),
        &nodes,
        &edges,
    );
    let analysis = store
        .persist_analysis(
            &analysis,
            std::slice::from_ref(revision),
            std::slice::from_ref(&input),
            &nodes,
            &edges,
        )
        .await?;
    input.release().await?;
    store
        .release_artifact(&artifact.key, "user", "recovery-fixture")
        .await?;
    ensure!(
        analysis.active && artifact.active,
        "retained science was not admitted"
    );
    let refusal = store
        .collect_artifact(&artifact_intent)
        .await
        .err()
        .context("analysis future artifact input had no retained obligation")?;
    ensure!(
        format!("{refusal:?}").contains("artifact has retained obligations"),
        "artifact collection refused for a different reason: {refusal}"
    );
    Ok(serde_json::to_vec(&RetainedScience {
        analysis: serde_json::to_vec(
            &pse_operations::generated::surreal::encode_canonical_analyses(&analysis)?,
        )?,
        artifact: serde_json::to_vec(
            &pse_operations::generated::surreal::encode_canonical_artifacts(&artifact)?,
        )?,
    })?)
}

async fn verify_retained_science(
    store: &CanonicalStore,
    read: &mut SelectedRead,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
) -> Result<()> {
    let members = store
        .resolve_names(read, "root", &[ANALYSIS_RECEIPT.into()])
        .await?;
    ensure!(members.len() == 1, "retained analysis receipt missing");
    let extent = store
        .selected_object_extent(read.selection(), &members[0].version)
        .await?
        .context("retained analysis receipt extent missing")?;
    let reservation = pse_columnar::MemoryConsumer::new("recovery:analysis-receipt").register(pool);
    reservation.try_grow(
        extent
            .checked_mul(2)
            .and_then(|bytes| {
                bytes.checked_add(2 * pse_operations::canonical_staging::SOURCE_BLOCK_BYTES)
            })
            .context("retained analysis receipt extent overflow")?,
    )?;
    let receipt = store
        .selected_object(read.selection(), &members[0].version)
        .await?
        .context("retained analysis receipt missing")?;
    let receipt: RetainedScience = serde_json::from_slice(receipt.payload.as_slice())?;
    let expected_analysis = pse_operations::generated::surreal::decode_canonical_analyses(
        serde_json::from_slice(&receipt.analysis)?,
    )?;
    let expected_artifact = pse_operations::generated::surreal::decode_canonical_artifacts(
        serde_json::from_slice(&receipt.artifact)?,
    )?;
    let analysis = store.analysis(&expected_analysis.key).await?;
    ensure!(
        analysis == expected_analysis,
        "admitted analysis changed during recovery"
    );
    let (nodes, edges) = analysis_graph(&analysis, &expected_artifact.key);
    ensure!(
        store.analysis_node_page(&analysis.key, None).await? == nodes,
        "retained analysis nodes changed"
    );
    ensure!(
        store.analysis_edge_page(&analysis.key, None).await? == edges,
        "retained analysis edges changed"
    );
    ensure!(
        store
            .analysis_node_page(&analysis.key, Some(&nodes[1].key))
            .await?
            .is_empty(),
        "retained analysis acquired extra nodes"
    );
    ensure!(
        store
            .analysis_edge_page(&analysis.key, Some(&edges[0].key))
            .await?
            .is_empty(),
        "retained analysis acquired extra edges"
    );
    let artifact = store.read_artifact(&expected_artifact.key).await?;
    ensure!(
        artifact.header() == &expected_artifact,
        "independent artifact metadata changed"
    );
    {
        let block = artifact.block(0).await?;
        ensure!(
            block.payload.as_slice() == ARTIFACT_BYTES,
            "analysis future artifact input changed"
        );
        ensure!(
            block.original_digest == "recovery-original-json-v1" && block.rows == 1,
            "artifact scientific coordinates changed"
        );
    }
    artifact.release().await?;
    Ok(())
}

async fn read_product(
    store: &CanonicalStore,
    read: &mut SelectedRead,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
) -> Result<
    Option<(
        pse_operations::canonical_selection::ReusableProduct,
        pse_columnar::MemoryReservation,
    )>,
> {
    let mut after = String::new();
    while let Some(candidate) = store
        .product_candidate(
            read.selection(),
            b"exact-recovery-result",
            "recovery-control.v1",
            &after,
        )
        .await?
    {
        after = candidate.key().to_owned();
        let reservation = pse_columnar::MemoryConsumer::new("recovery:product").register(pool);
        reservation.try_grow(candidate.retained_bytes())?;
        if let Some(product) = store.qualify_product(read, &candidate).await? {
            return Ok(Some((product, reservation)));
        }
    }
    Ok(None)
}

fn object(logical: &str, version: &str, payload: Vec<u8>) -> ObjectEdit {
    ObjectEdit {
        logical: logical.into(),
        scope: "root".into(),
        name: logical.into(),
        version: Some(ObjectVersion {
            key: version.into(),
            logical: logical.into(),
            kind: "recovery-control".into(),
            payload: payload.into(),
            interpretation: INTERPRETATION.into(),
        }),
        references: Vec::new(),
    }
}

pub(crate) async fn run(state: &Path, seed: bool) -> Result<()> {
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(128 * 1024 * 1024));
    let store = CanonicalStore::connect(&CanonicalOptions::from_state(state)?).await?;
    if seed {
        store.create().await?;
    }
    store.open().await?;
    let first_edits = vec![
        object(
            "x",
            "recovery-x-first",
            (-0.0f64).to_bits().to_be_bytes().to_vec(),
        ),
        object("bank", "recovery-large", vec![0xa7; LARGE_BYTES]),
    ];
    if seed {
        let first = store.edit(PROBLEM, None, FIRST, &first_edits).await?;
        let mut read = SelectedRead::new(
            store
                .protect(first.clone(), Duration::from_secs(60))
                .await?,
        );
        store
            .resolve_names(
                &mut read,
                "root",
                &["x".into(), "bank".into(), "absent".into()],
            )
            .await?;
        store
            .publish_product(
                &read,
                Product {
                    key: "recovery-description".into(),
                    problem: PROBLEM.into(),
                    revision: FIRST.into(),
                    request: b"exact-recovery-result".to_vec().into(),
                    payload: 2.0f64.to_bits().to_be_bytes().to_vec().into(),
                    dependencies: Vec::new().into(),
                    producer: "recovery-control.v1".into(),
                    interpretation: INTERPRETATION.into(),
                },
            )
            .await?;
        store.release(read.selection()).await?;
        let analysis_receipt = seed_retained_science(&store, &first).await?;
        store
            .edit(
                PROBLEM,
                Some(FIRST),
                SECOND,
                &[
                    object(
                        "x",
                        "recovery-x-second",
                        3.0f64.to_bits().to_be_bytes().to_vec(),
                    ),
                    object(ANALYSIS_RECEIPT, ANALYSIS_RECEIPT_VERSION, analysis_receipt),
                ],
            )
            .await?;
        ensure!(
            store.edit(PROBLEM, None, FIRST, &first_edits).await?.key == FIRST,
            "lost-ack replay did not return the immutable receipt"
        );
    }
    let first = store
        .revision(FIRST)
        .await?
        .context("historical receipt missing")?;
    let second = store
        .revision(SECOND)
        .await?
        .context("head receipt missing")?;
    ensure!(
        first.sequence == 1 && second.sequence == 2 && second.parent.as_deref() == Some(FIRST),
        "revision lineage changed"
    );
    let mut read = SelectedRead::new(store.protect(first, Duration::from_secs(60)).await?);
    let members = store
        .resolve_names(&mut read, "root", &["x".into(), "bank".into()])
        .await?;
    ensure!(members.len() == 2, "historical selected source missing");
    for member in members {
        let extent = store
            .selected_object_extent(read.selection(), &member.version)
            .await?
            .context("payload extent missing")?;
        let reservation = pse_columnar::MemoryConsumer::new("recovery:source").register(&pool);
        let bytes = extent
            .checked_mul(2)
            .and_then(|n| n.checked_add(2 * pse_operations::canonical_staging::SOURCE_BLOCK_BYTES))
            .context("source extent overflow")?;
        reservation.try_grow(bytes)?;
        let saved = store
            .selected_object(read.selection(), &member.version)
            .await?
            .context("payload missing")?;
        match member.name.as_str() {
            "x" => ensure!(
                saved.payload.as_slice() == (-0.0f64).to_bits().to_be_bytes(),
                "scientific bits changed"
            ),
            "bank" => ensure!(
                saved.payload.len() == LARGE_BYTES
                    && saved.payload.iter().all(|byte| *byte == 0xa7),
                "staged payload changed"
            ),
            _ => unreachable!("only two selected names"),
        }
    }
    {
        let (product, _reservation) = read_product(&store, &mut read, &pool)
            .await?
            .context("retained product missing")?;
        ensure!(
            product.payload() == 2.0f64.to_bits().to_be_bytes(),
            "recorded numerical value changed"
        );
    }
    store.release(read.selection()).await?;
    let mut changed = SelectedRead::new(store.protect(second, Duration::from_secs(60)).await?);
    ensure!(
        read_product(&store, &mut changed, &pool).await?.is_none(),
        "changed dependency incorrectly reused the old description"
    );
    verify_retained_science(&store, &mut changed, &pool).await?;
    store.release(changed.selection()).await?;
    if !seed {
        // Gated restore permits protection bookkeeping and reads, while authoring remains closed.
        ensure!(
            matches!(
                store
                    .edit(PROBLEM, Some(SECOND), "forbidden-restore-write", &[])
                    .await,
                Err(pse_operations::canonical::CanonicalError::Quiesced)
            ),
            "validation admitted an authoring mutation"
        );
    }
    store.disconnect().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_creation_intent_retains_exact_native_codec_fields() -> Result<()> {
        let intent = pse_operations::canonical_analyses::Analysis {
            key: "pse.analysis.v2:original-authority:nonce:123456".into(),
            revision: FIRST.into(),
            method: "recovery-authority.v1".into(),
            configuration: vec![0, 255, 1].into(),
            input_digest: "empty".into(),
            primary_problem: PROBLEM.into(),
            primary_authority: "original-authority".into(),
            creation_nonce: "exact-original-nonce".into(),
            creation_request_digest: "sealed-original-request".into(),
            creation_expires_at: 123456,
            interpretation: INTERPRETATION.into(),
            node_count: 0,
            edge_count: 0,
            active: false,
            retiring: false,
        };
        let native = pse_operations::generated::surreal::encode_canonical_analyses(&intent)?;
        let bytes = serde_json::to_vec(&native)?;
        let decoded = pse_operations::generated::surreal::decode_canonical_analyses(
            serde_json::from_slice(&bytes)?,
        )?;
        ensure!(
            decoded == intent,
            "saved creation intent was reconstructed differently"
        );
        Ok(())
    }
}

/// Seed an explicit two-file terminal partial observation for the installed
/// Python signal control. Each ordinary file seal + append ACK is a demanded
/// flush boundary. Runtime's append queue is private and deliberately not made
/// into a production fixture API.
#[cfg(feature = "package-fixtures")]
#[allow(unsafe_code, reason = "explicit control asserts only its own partial observation closure")]
pub(crate) async fn seed_signal_results(state: &Path) -> Result<serde_json::Value> {
    use pse_operations::canonical_execution::{
        ResultPayload, RunRequest, SeedEligibility, TerminalClass,
        result_batch_key, result_payload_digest, result_set_key,
    };
    use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
    ensure!(cfg!(feature = "force-validate"), "signal fixture requires force-validate");
    let options = CanonicalOptions::from_state(state)?;
    let store = CanonicalStore::connect(&options).await?;
    store.create().await?;
    store.open().await?;
    let run_id = pse_operations::mint_id::<pse_model::generated::identities::RunId>();
    let run = format!("signal-results:{run_id}");
    let revision = store.edit(&run, None, &format!("{run}:source"), &[]).await?;
    store.begin_run(&RunRequest {
        key: run.clone(), revision, sources: vec![],
        request: b"explicit-two-file-signal-control-v1".to_vec(),
        source_selection: vec![], attestation: vec![],
    }).await?;
    let fence = store.claim_run(&run, &format!("{run}:claim"), "signal-control", Duration::from_secs(60)).await?;
    let registry = pse_schema::shared_registry()?;
    let context = datafusion::execution::context::SessionContext::new();
    let validation = pse_relations::validate::ValidationContext::new(
        &registry, pse_engine::validation::NativeValidation(context.state()),
    );
    let pool: Arc<dyn pse_columnar::MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(256 << 20));
    let name = solve_metrics::RELATION_ID.to_string();
    let set = result_set_key(fence.attempt(), &name);
    let mut files = Vec::new();
    for ordinal in 0..2_u64 {
        let owner = Arc::new(pse_columnar::MemoryConsumer::new("control:signal-result-file").register(&pool));
        owner.try_grow(4 * pse_operations::canonical_files::FILE_BYTES)?;
        let mut builder = solve_metrics::Builder::with_registry(&registry, 1, &validation)?;
        builder.push(solve_metrics::Row {
            run_id, step: ordinal as i64, namespace: "signal-control".into(),
            name: "sealed-observation".into(), kind: NativeMetricKind::Real,
            real: Some(if ordinal == 0 { -0.0 } else { 2.0 }),
            integer: None, boolean: None, text: None, unavailable: None,
        })?;
        let batch = builder.finish()?;
        let bytes = pse_runtime::workflow::result_files::encode_result_file(batch.batch())?;
        let intent = store.reserve_result_file(&fence, &format!("{run}:signal-file:{ordinal}"), &result_payload_digest(&bytes)).await?;
        let file = store.seal_file(&intent, &bytes, pse_operations::canonical_files::ARROW_IPC_FILE, 1, &name).await?;
        let payload = ResultPayload::from_sealed_owned(file.clone(), 0, owner)?;
        let key = result_batch_key(fence.attempt(), &set, ordinal);
        let block = pse_model::generated::runtime::canonical_result_blocks::Row {
            key: key.clone(), batch: key, result_set: set.clone(), output: name.clone(),
            partition: "0".into(), ordinal, start: ordinal, end: ordinal + 1,
            rows: 1, columns: batch.batch().num_columns() as u64,
            coordinate_min: None, coordinate_max: None,
            payload_bytes: payload.byte_len(), payload_digest: payload.digest().into(),
            interpretation: INTERPRETATION.into(),
        };
        store.append_result_block(&fence, &format!("{run}:signal-append:{ordinal}"), &name, ordinal, payload, 1, &block).await?;
        files.push(file.descriptor().instance.clone());
    }
    let closed = store.close_result_ingestion(&fence, &format!("{run}:signal-close")).await?;
    let manifest = store.reconcile_closed_attempt(&closed).await?;
    // SAFETY: this explicit control owns two registry-valid observations, asserts
    // only partial availability, and grants no successful solve or seed meaning.
    unsafe {
        store.seal_attempt(&manifest, &format!("{run}:signal-partial"), TerminalClass::Partial,
            b"explicit signal-control partial observations", SeedEligibility::Unavailable).await?;
    }
    let read = store.read_results(&run, fence.attempt()).await?;
    let page = store.result_block_page(&read, &set, &name, "0", 0, 2, None).await?;
    ensure!(page.len() == 2 && page[0].ordinal == 0 && page[1].ordinal == 1
        && page[0].end == page[1].start, "first page lacks consecutive distinct-file blocks");
    let descriptors = store.result_file_batch_descriptors(&read, &set, &[0, 1]).await?;
    ensure!(descriptors.len() == 2 && descriptors[0].file_instance != descriptors[1].file_instance
        && descriptors.iter().zip(&files).all(|(batch, file)| &batch.file_instance == file),
        "signal control did not seal two actual file instances");
    drop(read);
    // Dropping the final read schedules its authenticated protection release.
    // Disconnect deliberately refuses while that original activity is live.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        match store.disconnect().await {
            Ok(()) => break,
            Err(pse_operations::canonical::CanonicalError::ConnectionBusy { .. })
                if tokio::time::Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(serde_json::json!({
        "version": 1, "event": "signal-results-seeded", "state": state,
        "database": options.database, "run": run, "attempt": fence.attempt(),
        "relation": "runtime.solve_metrics", "terminal_class": "partial",
        "first_page_ordinals": [0, 1], "file_instances": files,
    }))
}
