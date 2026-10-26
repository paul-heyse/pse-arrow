// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Contained independent reader and consumed-byte/import-copy boundaries.
#![allow(
    clippy::unreachable,
    reason = "Explicit fixture discriminants require the selected project and scientific member shapes"
)]
use super::*;
use crate::workflow;

async fn bundle() -> (tempfile::TempDir, ProjectManifest) {
    let runtime = workflow::tests::runtime();
    let cancel = crate::CancelSource::new();
    let (physical_documents, documents) = workflow::tests::sources(
        "package algebraic {def Fixed {param x:Scalar=2; eq fixed:x==2;}}",
    );
    let physical = Box::pin(runtime.physical_from_sources(&physical_documents, &cancel))
        .await
        .unwrap();
    let package = Box::pin(runtime.package_from_sources(&[documents], physical.clone(), &cancel))
        .await
        .unwrap();
    let replay = package.replay_source().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("members")).unwrap();
    let mut manifest = ProjectManifest {
        version: Version,
        origin: ProjectSelection {
            issuing_store: "unavailable-origin".into(),
            project: "original-project".into(),
            reference: workflow::ProjectReference::Branch { key: "main".into() },
            problem: "original-problem".into(),
            revision: "original-revision".into(),
            source_identity: package.revision.identity(),
            physical_context: physical.identity(),
        },
        sources: vec![ProjectSourceClosure {
            revision: "original-revision".into(),
            source_identity: Some(package.revision.identity().to_string()),
            physical_context: Some(physical.identity().to_string()),
            documents: vec![],
            replay_source: Some("source".into()),
            physical_documents: vec![],
        }],
        source_revisions: vec![ProjectSourceRevisionMetadata {
            key: "original-revision".into(),
            problem: "original-problem".into(),
            sequence: 1,
            parent: None,
            operation: "original-revision".into(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        }],
        result_metadata: vec![],
        datasets: vec![],
        research_selections: vec![],
        publications: vec![],
        contexts: vec![],
        studies: vec![],
        reproducers: vec![],
        providers: vec![],
        artifact_origins: vec![],
        members: vec![],
        maximum_copy_bytes: 64 << 20,
        omissions: vec![],
        offline_complete: true,
        executable_complete: true,
        descriptive_only: false,
    };
    fn add(root: &Path, manifest: &mut ProjectManifest, id: String, codec: &str, bytes: &[u8]) {
        let relative_path = format!("members/{}.bin", manifest.members.len());
        std::fs::write(root.join(&relative_path), bytes).unwrap();
        manifest.members.push(ProjectBundleMember {
            id,
            relative_path,
            codec: codec.into(),
            copied_bytes: bytes.len() as u64,
            digest: pse_ids::encoding_checksum(bytes).content_hash().to_hex(),
            result: None,
            artifact: None,
            artifact_metadata: None,
        });
    }
    add(
        root.path(),
        &mut manifest,
        "source".into(),
        "replay-source.v1",
        &serde_json::to_vec(&replay).unwrap(),
    );
    for (i, (path, bytes)) in physical_documents.into_iter().enumerate() {
        let id = format!("physical-{i}");
        add(
            root.path(),
            &mut manifest,
            id.clone(),
            "authored-document.v1",
            &bytes,
        );
        manifest.sources[0]
            .physical_documents
            .push(ProjectDocumentMember {
                package: 0,
                path,
                member: id,
            });
    }
    std::fs::write(
        root.path().join("project.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    (root, manifest)
}
fn save(root: &Path, manifest: &ProjectManifest) {
    std::fs::write(
        root.join("project.json"),
        serde_json::to_vec(manifest).unwrap(),
    )
    .unwrap();
}

#[tokio::test]
async fn project_portability_result_ipc_reuses_original_lineage_and_finalization_validation() {
    use datafusion::arrow::{
        datatypes::{Field, Schema},
        ipc::writer::StreamWriter,
    };
    let (root, mut manifest) = bundle().await;
    let metadata = std::collections::HashMap::from([
        ("pse.canonical.run".into(), "original-run".into()),
        ("pse.canonical.attempt".into(), "original-attempt".into()),
        ("pse.canonical.relation".into(), "original-relation".into()),
        ("pse.canonical.relation-version".into(), "1".into()),
        ("pse.canonical.row-start".into(), "0".into()),
        ("pse.canonical.row-end".into(), "0".into()),
        ("pse.canonical.terminal-class".into(), "completed".into()),
    ]);
    let schema = Schema::new_with_metadata(Vec::<Field>::new(), metadata);
    let mut bytes = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut bytes, &schema).unwrap();
        writer.finish().unwrap();
    }
    let path = "members/scientific-result.arrow";
    std::fs::write(root.path().join(path), &bytes).unwrap();
    manifest.members.push(ProjectBundleMember {
        id: "selected-result".into(),
        relative_path: path.into(),
        codec: "arrow-ipc-stream.v1".into(),
        copied_bytes: bytes.len() as u64,
        digest: pse_ids::encoding_checksum(&bytes).content_hash().to_hex(),
        result: Some(workflow::PackResultSelection {
            run: "original-run".into(),
            attempt: "original-attempt".into(),
            relation: "original-relation".into(),
            schema_version: 1,
            row_start: 0,
            row_end: 0,
            field: None,
            terminal_class: "completed".into(),
            predicate: None,
            usability: workflow::PackScientificState::Unknown,
        }),
        artifact: None,
        artifact_metadata: None,
    });
    save(root.path(), &manifest);
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_ok());
    let original = manifest.clone();
    for qualifier in [
        "run",
        "attempt",
        "relation",
        "version",
        "range",
        "terminal",
        "field",
        "predicate",
    ] {
        let mut changed = original.clone();
        let result = changed.members.last_mut().unwrap().result.as_mut().unwrap();
        match qualifier {
            "run" => result.run = "another-run".into(),
            "attempt" => result.attempt = "another-attempt".into(),
            "relation" => result.relation = "another-relation".into(),
            "version" => result.schema_version = 2,
            "range" => result.row_end = 1,
            "terminal" => result.terminal_class = "failed".into(),
            "field" => result.field = Some("another-field".into()),
            "predicate" => {
                result.predicate = Some(workflow::PackResultPredicate {
                    output: "another-output".into(),
                    partition: "p".into(),
                    minimum_bits: None,
                    maximum_bits: None,
                    missing: None,
                })
            }
            _ => unreachable!(),
        }
        save(root.path(), &changed);
        assert!(
            ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
            "checksummed IPC must retain exact original {qualifier}"
        );
    }
    let mut wrong_codec = original.clone();
    wrong_codec.members.last_mut().unwrap().codec = "authored-document.v1".into();
    save(root.path(), &wrong_codec);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "a result cannot bypass lineage checks by relabelling its codec"
    );
    let mut unknown = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut unknown, &Schema::empty()).unwrap();
        writer.finish().unwrap();
    }
    std::fs::write(root.path().join(path), &unknown).unwrap();
    let mut no_lineage = original;
    let member = no_lineage.members.last_mut().unwrap();
    member.copied_bytes = unknown.len() as u64;
    member.digest = pse_ids::encoding_checksum(&unknown).content_hash().to_hex();
    save(root.path(), &no_lineage);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "a finalized checksummed stream without original scientific lineage is unknown"
    );
    let mut unfinished = bytes;
    unfinished.truncate(unfinished.len() - 8);
    std::fs::write(root.path().join(path), &unfinished).unwrap();
    let member = manifest.members.last_mut().unwrap();
    member.copied_bytes = unfinished.len() as u64;
    member.digest = pse_ids::encoding_checksum(&unfinished)
        .content_hash()
        .to_hex();
    save(root.path(), &manifest);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "a matching byte checksum cannot replace the IPC terminal marker"
    );
}

#[tokio::test]
async fn project_portability_offline_reader_uses_only_contained_bytes_and_detects_later_mutation() {
    let (root, manifest) = bundle().await;
    // This has no canonical deployment handle or reachable original service.
    let reader = ProjectBundleReader::open(root.path(), 16 << 20).unwrap();
    assert_eq!(reader.inspect().origin.issuing_store, "unavailable-origin");
    let original = reader.member_bytes("source", 16 << 20).unwrap();
    assert!(!original.is_empty());
    let path = root.path().join(&manifest.members[0].relative_path);
    let mut changed = original.clone();
    changed[0] ^= 1;
    std::fs::write(path, changed).unwrap();
    assert!(
        reader.member_bytes("source", 16 << 20).is_err(),
        "cached inspection cannot authenticate subsequently changed consumed bytes"
    );
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_err());
}
#[tokio::test]
async fn project_portability_missing_required_closure_unknown_codec_and_escaped_path_refuse_complete_manifest()
 {
    let (root, manifest) = bundle().await;
    let mut missing = manifest.clone();
    missing.members.remove(0);
    save(root.path(), &missing);
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_err());
    let mut no_replay = manifest.clone();
    no_replay.sources[0].replay_source = None;
    save(root.path(), &no_replay);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "required executable source cannot omit its replay member"
    );
    no_replay.descriptive_only = true;
    no_replay.executable_complete = false;
    no_replay.sources[0].physical_documents.clear();
    save(root.path(), &no_replay);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "descriptive source omission must be explicit and execution-required"
    );
    let mut unknown = manifest.clone();
    unknown.members[0].codec = "guessed-codec.v99".into();
    save(root.path(), &unknown);
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_err());
    let mut escape = manifest;
    escape.members[0].relative_path = "members/../../outside".into();
    save(root.path(), &escape);
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_err());
}
#[tokio::test]
async fn project_portability_symlink_escape_and_interrupted_archive_copy_refuse() {
    let (root, manifest) = bundle().await;
    let reader = ProjectBundleReader::open(root.path(), 16 << 20).unwrap();
    let cancel = crate::CancelSource::new();
    cancel.cancel();
    assert!(reader.archive(&cancel).is_err());
    assert!(!root.path().join(".project-manifest").exists());
    let path = root.path().join(&manifest.members[0].relative_path);
    std::fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(root.path().join("project.json"), &path).unwrap();
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_err());
    assert!(reader.member_bytes("source", 16 << 20).is_err());
}
#[tokio::test]
async fn project_portability_unsupported_provider_is_inspectable_and_never_executable() {
    let (root, mut manifest) = bundle().await;
    manifest.providers.push(ProjectProviderRequirement {
        name: "unavailable-provider".into(),
        identity: "original-required-factory".into(),
        supported: false,
    });
    manifest.executable_complete = false;
    save(root.path(), &manifest);
    let reader = ProjectBundleReader::open(root.path(), 16 << 20).unwrap();
    assert!(!reader.inspect().executable_complete);
    let runtime = workflow::tests::runtime();
    assert!(
        Box::pin(runtime.project_import_issue(&reader, "refused", "main"))
            .await
            .is_err()
    );
    manifest.executable_complete = true;
    save(root.path(), &manifest);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "unsupported provider cannot be advertised as complete executable closure"
    );
}

#[tokio::test]
async fn project_portability_selected_reproducer_executes_only_on_explicit_owner_request() {
    let runtime = workflow::tests::runtime();
    let cancel = crate::CancelSource::new();
    // Reuse the reproducer owner's actual supported pre-handle diagnostic fixture and exact predicate.
    let command = workflow::reproducer::tests::text_command();
    let first = Box::pin(runtime.diagnose_documents(command.operation.clone(), &cancel))
        .await
        .unwrap();
    let original = first.failure.unwrap();
    let extracted =
        workflow::DiagnosticReproducer::extract_text(command, Some(original), None).unwrap();
    let (root, mut manifest) = bundle().await;
    manifest.reproducers.push(extracted.reproducer.clone());
    save(root.path(), &manifest);
    let (reader, counts) =
        crate::math::counted(async { ProjectBundleReader::open(root.path(), 16 << 20) }).await;
    assert_eq!(
        counts,
        Default::default(),
        "opening the bundle cannot reconstruct or execute its selected reproducer"
    );
    let reader = reader.unwrap();
    let request = reader.reproducer(0).unwrap();
    assert_eq!(
        serde_json::to_value(&request.predicate).unwrap(),
        serde_json::to_value(&extracted.reproducer.predicate).unwrap()
    );
    let reproduced = Box::pin(runtime.run_reproducer(request, &cancel))
        .await
        .unwrap();
    assert_ne!(
        reproduced.operation, first.operation,
        "reproduction is a new explicitly requested operation"
    );
    assert_eq!(
        reproduced.status,
        workflow::ReproductionStatus::OriginalFailureReproduced
    );
    assert!(reproduced.failure.is_some());
    assert!(
        reproduced.run_id.is_none(),
        "the original selected failure occurred before any scientific run handle"
    );
}

#[test]
fn project_portability_interrupted_final_manifest_publication_leaves_only_partial_staging() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("members")).unwrap();
    std::fs::write(root.path().join("members/copied.bin"), b"copied bytes").unwrap();
    let result = portable_payload::publish_document(
        root.path(),
        ".project-manifest",
        "project.json",
        b"complete manifest must not appear",
        || {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "interrupted final publication",
            ))
        },
    );
    assert!(result.is_err());
    assert!(!root.path().join("project.json").exists());
    assert!(
        root.path().join(".project-manifest").exists(),
        "staged bytes are honest partial output"
    );
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_err());
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn project_portability_expired_original_recovery_keeps_exact_receipt_and_fresh_destination() {
    let runtime = workflow::durable_tests::durable_runtime();
    let cancel = crate::CancelSource::new();
    let (root, _) = bundle().await;
    let reader = ProjectBundleReader::open(root.path(), 16 << 20).unwrap();
    let intent = Box::pin(runtime.project_import_issue(&reader, "Recovered import", "main"))
        .await
        .unwrap();
    let local = runtime
        .clone()
        .with_durability(workflow::DurabilitySelection::Ephemeral)
        .unwrap();
    let physical_documents = reader.physical_documents().unwrap();
    let physical = Box::pin(local.physical_from_sources(&physical_documents, &cancel))
        .await
        .unwrap();
    let admitted = reader
        .replay_source()
        .unwrap()
        .reconstruct(&local.source_services(), physical, BTreeMap::new(), &cancel)
        .unwrap();
    let source =
        Box::pin(admitted.publish_portable_import(&runtime.source_services(), &intent, &cancel))
            .await
            .unwrap();
    let revision = source.canonical_revision().unwrap().clone();
    let store = runtime.canonical_store().unwrap();
    // An authenticated original occurrence that never began copying can already be expired.
    // Its original identity is issued before any effect; no live header/clock is edited.
    let bytes = b"interrupted original copy".to_vec();
    let mut original = store
        .new_artifact(
            &revision,
            "project-import-evidence",
            encode(&(1u8, intent.bundle_digest()), 131072).unwrap(),
            1,
            bytes.len() as u64,
        )
        .await
        .unwrap();
    original.creation_expires_at = 1;
    original.key = format!(
        "pse.artifact.v1:{}:{}:{}",
        original.primary_authority, original.creation_nonce, original.creation_expires_at
    );
    let blocks = vec![pse_operations::canonical_artifacts::artifact_block(
        &original.key,
        0,
        "project-import-evidence",
        bytes,
        intent.bundle_digest(),
        0,
    )];
    pse_operations::canonical_artifacts::seal_artifact_request(
        &mut original,
        std::slice::from_ref(&revision),
        &[],
        &blocks,
        "project",
        intent.operation(),
    );
    store
        .project_import_bind_artifact(&intent, &original)
        .await
        .unwrap();
    assert!(
        !runtime
            .project_import_status(&intent)
            .await
            .unwrap()
            .unwrap()
            .project_complete
    );
    assert!(
        Box::pin(runtime.project_import(&reader, &intent, &cancel))
            .await
            .is_err(),
        "an expired original is not silently renewed"
    );
    let collected = Box::pin(runtime.project_import_recover(&intent, &original, &cancel))
        .await
        .unwrap();
    assert_eq!(
        collected.state,
        ProjectImportRecoveryState::OriginalCollected
    );
    assert_eq!(collected.import.retired_originals, vec![original.clone()]);
    assert!(!collected.import.project_complete);
    assert!(collected.import.artifact_intent.is_none());
    let imported = Box::pin(runtime.project_import(&reader, &intent, &cancel))
        .await
        .unwrap();
    assert!(imported.project_complete);
    assert_ne!(imported.artifact_intent.as_ref().unwrap().key, original.key);
    assert_eq!(imported.project, intent.destinations().0);
    assert_eq!(imported.revision, intent.operation());
    // Retrying a lost recovery reply returns its old receipt and cannot abort the newer active occurrence.
    let old_reply = Box::pin(runtime.project_import_recover(&intent, &original, &cancel))
        .await
        .unwrap();
    assert_eq!(
        old_reply.state,
        ProjectImportRecoveryState::OriginalCollected
    );
    assert!(old_reply.import.project_complete);
    assert_eq!(old_reply.import.artifact_intent, imported.artifact_intent);
    let duplicate = Box::pin(runtime.project_import(&reader, &intent, &cancel))
        .await
        .unwrap();
    assert_eq!(duplicate.project, imported.project);
    std::fs::remove_dir_all(root.path()).unwrap();
    let reopened = Box::pin(runtime.project_import_open(&intent, &cancel))
        .await
        .unwrap();
    assert_eq!(
        reopened.canonical_revision().unwrap().key,
        imported.revision,
        "current copied artifact/source ownership is independent of supplied bundle lifetime"
    );
    let repeated = Box::pin(runtime.project_import(&reader, &intent, &cancel))
        .await
        .unwrap();
    assert_eq!(
        repeated.project, imported.project,
        "lost completed reply settles before missing original pack preflight"
    );
    store.remove_isolated_fixture().await.unwrap();
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn project_portability_active_original_settles_after_expiry_without_copy_or_original_pack() {
    let runtime = workflow::durable_tests::durable_runtime();
    let cancel = crate::CancelSource::new();
    let (root, _) = bundle().await;
    let reader = ProjectBundleReader::open(root.path(), 16 << 20).unwrap();
    let intent = Box::pin(runtime.project_import_issue(&reader, "Lost activation reply", "main"))
        .await
        .unwrap();
    let local = runtime
        .clone()
        .with_durability(workflow::DurabilitySelection::Ephemeral)
        .unwrap();
    let physical_documents = reader.physical_documents().unwrap();
    let physical = Box::pin(local.physical_from_sources(&physical_documents, &cancel))
        .await
        .unwrap();
    let admitted = reader
        .replay_source()
        .unwrap()
        .reconstruct(&local.source_services(), physical, BTreeMap::new(), &cancel)
        .unwrap();
    let source =
        Box::pin(admitted.publish_portable_import(&runtime.source_services(), &intent, &cancel))
            .await
            .unwrap();
    let revision = source.canonical_revision().unwrap().clone();
    let store = runtime.canonical_store().unwrap();
    let mut archive = reader.archive(&cancel).unwrap();
    let mut bytes = Vec::new();
    archive.read_to_end(&mut bytes).unwrap();
    let chunk = pse_operations::canonical_files::FILE_TARGET_BYTES;
    let mut original = store
        .new_artifact(
            &revision,
            "project-import-evidence",
            encode(&(1u8, intent.bundle_digest()), 131072).unwrap(),
            bytes.len().div_ceil(chunk) as u64,
            bytes.len() as u64,
        )
        .await
        .unwrap();
    // Shorten an unissued fixture's original allowance before framing/binding it; never renew an issued clock.
    original.creation_expires_at -= 57_000_000;
    original.key = format!(
        "pse.artifact.v1:{}:{}:{}",
        original.primary_authority, original.creation_nonce, original.creation_expires_at
    );
    let blocks = bytes
        .chunks(chunk)
        .enumerate()
        .map(|(i, b)| {
            pse_operations::canonical_artifacts::artifact_block(
                &original.key,
                i as u64,
                "project-import-evidence",
                b.to_vec(),
                intent.bundle_digest(),
                0,
            )
        })
        .collect::<Vec<_>>();
    pse_operations::canonical_artifacts::seal_artifact_request(
        &mut original,
        std::slice::from_ref(&revision),
        &[],
        &blocks,
        "project",
        intent.operation(),
    );
    store
        .project_import_bind_artifact(&intent, &original)
        .await
        .unwrap();
    store
        .persist_artifact(
            &original,
            std::slice::from_ref(&revision),
            &[],
            &blocks,
            "project",
            intent.operation(),
        )
        .await
        .unwrap();
    assert!(
        !runtime
            .project_import_status(&intent)
            .await
            .unwrap()
            .unwrap()
            .project_complete,
        "simulate reply loss between original artifact activation and project finish"
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while store
        .artifact_creation_status(&original)
        .await
        .unwrap()
        .window_open
    {
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    std::fs::remove_dir_all(root.path()).unwrap();
    let completed = Box::pin(runtime.project_import(&reader, &intent, &cancel))
        .await
        .unwrap();
    assert!(completed.project_complete);
    assert_eq!(completed.artifact_intent, Some(original.clone()));
    assert!(completed.retired_originals.is_empty());
    let reopened = Box::pin(runtime.project_import_open(&intent, &cancel))
        .await
        .unwrap();
    assert_eq!(
        reopened.canonical_revision().unwrap().key,
        intent.operation()
    );
    let recovered = Box::pin(runtime.project_import_recover(&intent, &original, &cancel))
        .await
        .unwrap();
    assert_eq!(
        recovered.state,
        ProjectImportRecoveryState::ProjectComplete,
        "active exact originals are finished, never aborted"
    );
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn project_portability_artifact_original_codec_digest_and_scientific_extent_survive_copy() {
    let (root, mut manifest) = bundle().await;
    let block = pse_operations::canonical_artifacts::artifact_block(
        "origin-artifact",
        0,
        "origin-fitted-cells",
        b"{\"estimate\":11}".to_vec(),
        "original-encoding-provenance",
        1,
    );
    let original = pse_operations::canonical_artifacts::Artifact {
        key: block.artifact.clone(),
        kind: "original-copied-evidence".into(),
        revision: manifest.origin.revision.clone(),
        primary_problem: manifest.origin.problem.clone(),
        primary_authority: "descriptive-origin-authority".into(),
        creation_nonce: "original-nonce".into(),
        creation_expires_at: 1,
        creation_request_digest: "original-request".into(),
        interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        provenance: vec![1].into(),
        block_count: 1,
        payload_bytes: block.payload.len() as u64,
        active: true,
        retiring: false,
    };
    manifest.artifact_origins.push(original);
    let relative_path = "members/evidence.bin".to_owned();
    std::fs::write(root.path().join(&relative_path), block.payload.as_slice()).unwrap();
    manifest.members.push(ProjectBundleMember {
        id: "original-evidence".into(),
        relative_path,
        codec: "artifact-block.v1".into(),
        copied_bytes: block.payload.len() as u64,
        digest: pse_ids::encoding_checksum(block.payload.as_slice())
            .content_hash()
            .to_hex(),
        result: None,
        artifact: Some((block.artifact.clone(), 0)),
        artifact_metadata: Some(ProjectArtifactBlockMetadata {
            relation: block.relation.clone(),
            payload_digest: block.digest.clone(),
            original_digest: block.original_digest.clone(),
            rows: block.rows,
            encoding: block.encoding().into(),
            schema: block.schema().into(),
        }),
    });
    save(root.path(), &manifest);
    let reader = ProjectBundleReader::open(root.path(), 16 << 20).unwrap();
    let metadata = reader
        .inspect()
        .members
        .last()
        .unwrap()
        .artifact_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.rows, 1);
    assert_eq!(metadata.relation, "origin-fitted-cells");
    assert_eq!(metadata.original_digest, "original-encoding-provenance");
    assert_eq!(metadata.payload_digest, block.digest);
    assert_eq!(
        reader.member_bytes("original-evidence", 1024).unwrap(),
        block.payload.as_slice()
    );
    let full = manifest.clone();
    let mut missing = full.clone();
    missing.members.pop();
    save(root.path(), &missing);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "complete artifact cannot lose its required ordinal"
    );
    let mut duplicate = full.clone();
    let mut extra = duplicate.members.last().unwrap().clone();
    extra.id = "duplicate-original-coordinate".into();
    extra.relative_path = "members/duplicate-evidence.bin".into();
    std::fs::write(
        root.path().join(&extra.relative_path),
        block.payload.as_slice(),
    )
    .unwrap();
    duplicate.members.push(extra);
    save(root.path(), &duplicate);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "distinct member IDs cannot duplicate an original artifact ordinal"
    );
    let mut extent = full.clone();
    extent.artifact_origins[0].payload_bytes += 1;
    save(root.path(), &extent);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "copied bytes must cover the exact original artifact extent"
    );
    let mut headers = full.clone();
    headers
        .artifact_origins
        .push(headers.artifact_origins[0].clone());
    save(root.path(), &headers);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20).is_err(),
        "an artifact header has one authoritative selection"
    );
    let mut descriptive = missing;
    descriptive.descriptive_only = true;
    descriptive.executable_complete = false;
    descriptive.omissions.push(ProjectOmission {
        selection: "artifact:origin-artifact".into(),
        reason: "explicit metadata-only evidence selection".into(),
        required_for_execution: true,
    });
    save(root.path(), &descriptive);
    assert!(
        ProjectBundleReader::open(root.path(), 16 << 20)
            .unwrap()
            .inspect()
            .descriptive_only,
        "only explicit descriptive omission permits a header without copied artifact blocks"
    );
    let metadata = manifest
        .members
        .last_mut()
        .unwrap()
        .artifact_metadata
        .as_mut()
        .unwrap();
    metadata.payload_digest = "00".repeat(32);
    save(root.path(), &manifest);
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_err());
    let metadata = manifest
        .members
        .last_mut()
        .unwrap()
        .artifact_metadata
        .as_mut()
        .unwrap();
    metadata.payload_digest = block.digest;
    metadata.encoding = "unknown-evidence.v99".into();
    save(root.path(), &manifest);
    assert!(ProjectBundleReader::open(root.path(), 16 << 20).is_err());
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn project_portability_descriptive_export_never_hydrates_missing_selected_artifact_payloads()
{
    let runtime = workflow::durable_tests::durable_runtime();
    let cancel = crate::CancelSource::new();
    let (physical_documents, documents) =
        workflow::tests::sources("package algebraic {def Fixed {param x:Scalar=2;eq fixed:x==2;}}");
    let physical = Box::pin(runtime.physical_from_sources(&physical_documents, &cancel))
        .await
        .unwrap();
    let source = Box::pin(runtime.package_from_sources(&[documents], physical.clone(), &cancel))
        .await
        .unwrap();
    let store = runtime.canonical_store().unwrap();
    store
        .create_project(
            "create-descriptive",
            "descriptive-project",
            "Metadata survives unavailable bytes",
        )
        .await
        .unwrap();
    let project = store.project("descriptive-project").await.unwrap().unwrap();
    Box::pin(runtime.project_fork(
        &project,
        "descriptive-main",
        "descriptive-source",
        "descriptive-fork",
        &source,
    ))
    .await
    .unwrap();
    let selection = Box::pin(runtime.project_select(
        &workflow::ProjectSelectRequest {
            version: Version,
            reference: workflow::ProjectReference::Branch {
                key: "descriptive-main".into(),
            },
        },
        physical.clone(),
        &cancel,
    ))
    .await
    .unwrap();
    // Source objects live in the original canonical source-block owner, independently
    // from FileTier. Create actual selected artifact bytes before withholding files.
    let revision = store.revision(&selection.revision).await.unwrap().unwrap();
    let evidence = b"original selected artifact evidence".to_vec();
    let mut artifact = store
        .new_artifact(
            &revision,
            "descriptive-test-evidence",
            vec![],
            1,
            evidence.len() as u64,
        )
        .await
        .unwrap();
    let blocks = vec![pse_operations::canonical_artifacts::artifact_block(
        &artifact.key,
        0,
        "descriptive-test-evidence",
        evidence.clone(),
        "original-evidence",
        0,
    )];
    pse_operations::canonical_artifacts::seal_artifact_request(
        &mut artifact,
        std::slice::from_ref(&revision),
        &[],
        &blocks,
        "project",
        &selection.project,
    );
    let artifact = store
        .persist_artifact(
            &artifact,
            std::slice::from_ref(&revision),
            &[],
            &blocks,
            "project",
            &selection.project,
        )
        .await
        .unwrap();
    let readable = store.read_artifact(&artifact.key).await.unwrap();
    assert_eq!(
        readable.block(0).await.unwrap().payload.as_slice(),
        evidence
    );
    drop(readable);
    let mut request = ProjectExportRequest {
        version: Version,
        selection: selection.clone(),
        research_selections: vec![],
        publications: vec![],
        studies: vec![],
        results: vec![],
        artifacts: vec![artifact.key.clone()],
        reproducers: vec![],
        descriptive_only: false,
        omissions: vec![],
        maximum_output_bytes: 16 << 20,
        maximum_document_bytes: 16 << 20,
    };
    let output = tempfile::tempdir().unwrap();
    let complete_path = output.path().join("must-refuse");
    let metadata_path = output.path().join("metadata");
    let manifest=store.with_unavailable_fixture_payloads(async {
        let unavailable=store.read_artifact(&artifact.key).await.unwrap();assert!(unavailable.block(0).await.is_err(),"the selected artifact's original sealed bytes are genuinely absent");drop(unavailable);
        assert!(Box::pin(runtime.project_export(&complete_path,&request,physical.clone(),&cancel)).await.is_err(),"executable export cannot silently reinterpret missing selected artifact bytes as metadata");
        assert!(!complete_path.join("project.json").exists());
        assert!(Box::pin(runtime.project_export_metadata(&metadata_path,&request,&cancel)).await.is_err(),"metadata path requires explicit descriptive selection");
        request.descriptive_only=true;
        let explanation=Box::pin(runtime.project_closure_metadata(&request,&cancel)).await.unwrap();assert!(explanation.manifest.members.is_empty());
        Box::pin(runtime.project_export_metadata(&metadata_path,&request,&cancel)).await.unwrap()
    }).await.unwrap();
    let restored = store.read_artifact(&artifact.key).await.unwrap();
    assert_eq!(
        restored.block(0).await.unwrap().payload.as_slice(),
        evidence
    );
    drop(restored);
    assert!(manifest.offline_complete);
    assert!(!manifest.executable_complete);
    assert!(manifest.members.is_empty());
    assert_eq!(manifest.origin.revision, selection.revision);
    assert!(manifest.sources[0].replay_source.is_none());
    assert!(
        manifest.sources[0].source_identity.is_none(),
        "a selector alone cannot authenticate a scientific source hash without readmission"
    );
    assert_eq!(manifest.source_revisions[0].problem, selection.problem);
    assert!(manifest.omissions.iter().any(|o| o.selection
        == format!("source:{}", selection.revision)
        && o.required_for_execution));
    assert_eq!(manifest.artifact_origins[0].key, artifact.key);
    assert!(
        manifest.omissions.iter().any(
            |o| o.selection == format!("artifact:{}", artifact.key) && o.required_for_execution
        )
    );
    store.close();
    assert!(
        store.project("descriptive-project").await.is_err(),
        "original source caller is unavailable during independent inspection"
    );
    let reader = ProjectBundleReader::open(&metadata_path, 16 << 20).unwrap();
    assert!(reader.inspect().descriptive_only);
    assert!(
        Box::pin(runtime.project_import_issue(&reader, "must-refuse", "main"))
            .await
            .is_err(),
        "descriptive closure is never editable source admission"
    );
    drop(reader);
    drop(source);
    drop(runtime);
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn project_portability_selected_multi_revision_exchange_reuses_original_scientific_admission() {
    let runtime = workflow::durable_tests::durable_runtime();
    let cancel = crate::CancelSource::new();
    let (physical_documents, _) = workflow::tests::sources(
        "package algebraic {def Fixed {param x:Scalar=2;eq fixed:x==2;}}",
    );
    let physical = Box::pin(runtime.physical_from_sources(&physical_documents, &cancel))
        .await
        .unwrap();
    let store = runtime.canonical_store().unwrap();
    store.create_project("create-exchange", "exchange-project", "Exact selected exchange")
        .await.unwrap();
    // Keep only original identities/checksums, rather than retaining the fixture's packages
    // or encoded bodies while the production traversal reopens its selected revisions.
    let mut expected = BTreeMap::new();
    let mut historical = None;
    for (branch, operation, value) in [
        ("history", "a-history", 2),
        ("unselected", "m-unselected", 7),
        ("origin", "z-origin", 3),
    ] {
        let (_, documents) = workflow::tests::sources(&format!(
            "package algebraic {{def Fixed {{param x:Scalar={value};eq fixed:x=={value};}}}}"
        ));
        let source = Box::pin(runtime.package_from_sources(&[documents], physical.clone(), &cancel))
            .await.unwrap();
        let replay = serde_json::to_vec(&source.replay_source().unwrap()).unwrap();
        let identity = source.revision.identity();
        let replay_digest = pse_ids::encoding_checksum(&replay).content_hash().to_hex();
        let project = store.project("exchange-project").await.unwrap().unwrap();
        let revision = Box::pin(runtime.project_fork(
            &project, branch, &format!("problem-{branch}"), operation, &source,
        )).await.unwrap();
        if branch == "history" { historical = Some(revision.clone()); }
        if branch != "unselected" { expected.insert(revision.key.clone(), (identity, replay_digest)); }
    }
    let selection = Box::pin(runtime.project_select(
        &workflow::ProjectSelectRequest {
            version: Version,
            reference: workflow::ProjectReference::Branch { key: "origin".into() },
        }, physical.clone(), &cancel,
    )).await.unwrap();
    let historical = historical.unwrap();
    let evidence = b"selected historical evidence";
    let mut artifact = store.new_artifact(
        &historical, "exchange-evidence", vec![], 1, evidence.len() as u64,
    ).await.unwrap();
    let blocks = vec![pse_operations::canonical_artifacts::artifact_block(
        &artifact.key, 0, "exchange-evidence", evidence.to_vec(), "original-history", 0,
    )];
    pse_operations::canonical_artifacts::seal_artifact_request(
        &mut artifact, std::slice::from_ref(&historical), &[], &blocks, "project", &selection.project,
    );
    let artifact = store.persist_artifact(
        &artifact, std::slice::from_ref(&historical), &[], &blocks, "project", &selection.project,
    ).await.unwrap();
    let mut request = ProjectExportRequest {
        version: Version, selection, research_selections: vec![], publications: vec![],
        studies: vec![], results: vec![], artifacts: vec![artifact.key.clone()], reproducers: vec![],
        descriptive_only: false, omissions: vec![], maximum_output_bytes: 16 << 20,
        maximum_document_bytes: 16 << 20,
    };
    let explanation = Box::pin(runtime.project_closure(&request, physical.clone(), &cancel))
        .await.unwrap();
    assert!(!explanation.result_bytes_known);
    assert_eq!(explanation.manifest.sources.iter().map(|s| &s.revision).collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(), "unselected project forks cannot broaden exact closure");
    let output = tempfile::tempdir().unwrap();
    let first = output.path().join("first");
    let manifest = Box::pin(runtime.project_export(&first, &request, physical.clone(), &cancel))
        .await.unwrap();
    assert!(manifest.executable_complete && manifest.offline_complete);
    assert!(manifest.providers.is_empty(), "this fixture has no external execution premises");
    assert_eq!(explanation.known_source_bytes, manifest.members.iter()
        .filter(|m| m.artifact.is_none()).map(|m| m.copied_bytes).sum::<u64>());
    let reader = ProjectBundleReader::open(&first, request.maximum_document_bytes).unwrap();
    // Admit every exported physical/model pair through the receiving scientific pipeline.
    // A contained checksum alone does not establish either scientific identity.
    let receiver = workflow::tests::runtime();
    for (ordinal, source) in manifest.sources.iter().enumerate() {
        assert_eq!(source.replay_source.as_deref(), Some(format!("source-{ordinal}").as_str()));
        let expected = &expected[&source.revision];
        let replay_member = manifest.members.iter()
            .find(|m| Some(m.id.as_str()) == source.replay_source.as_deref()).unwrap();
        assert_eq!(replay_member.digest, expected.1);
        let replay: workflow::ReplaySourceDocument = serde_json::from_slice(
            &reader.member_bytes(&replay_member.id, request.maximum_document_bytes).unwrap(),
        ).unwrap();
        let copied = reader.source_documents(source).unwrap();
        assert_eq!(copied.physical, physical_documents,
            "shared physical documents retain exact bytes in each source closure");
        let receiving_physical = Box::pin(receiver.physical_from_sources(&copied.physical, &cancel))
            .await.unwrap();
        assert_eq!(receiving_physical.identity(), physical.identity());
        let admitted = replay.reconstruct(&receiver.source_services(), receiving_physical,
            BTreeMap::new(), &cancel).unwrap();
        assert_eq!(admitted.revision.identity(), expected.0);
        for document in source.documents.iter().chain(&source.physical_documents) {
            let member = manifest.members.iter().find(|m| m.id == document.member).unwrap();
            let bytes = reader.member_bytes(&member.id, request.maximum_document_bytes).unwrap();
            assert_eq!(pse_ids::encoding_checksum(&bytes).content_hash().to_hex(), member.digest);
        }
    }
    let artifact_member = manifest.members.iter().find(|m| m.artifact.is_some()).unwrap();
    assert_eq!(reader.member_bytes(&artifact_member.id, request.maximum_document_bytes).unwrap(), evidence);
    assert_eq!(artifact_member.artifact.as_ref().unwrap(), &(artifact.key.clone(), 0));
    assert_eq!(artifact_member.artifact_metadata.as_ref().unwrap().original_digest, "original-history");
    let second = output.path().join("second");
    let repeated = Box::pin(runtime.project_export(&second, &request, physical.clone(), &cancel))
        .await.unwrap();
    assert_eq!(serde_json::to_vec(&manifest).unwrap(), serde_json::to_vec(&repeated).unwrap(),
        "origin-first traversal preserves deterministic sorted source IDs and copied checksums");

    request.maximum_output_bytes = explanation.known_source_bytes - 1;
    let exhausted = output.path().join("exhausted");
    assert!(Box::pin(runtime.project_export(&exhausted, &request, physical.clone(), &cancel)).await.is_err());
    assert!(!exhausted.join("project.json").exists());
    assert!(ProjectBundleReader::open(&exhausted, request.maximum_document_bytes).is_err());
    request.maximum_output_bytes = 16 << 20;
    let interrupted = output.path().join("interrupted");
    let interrupted_cancel = crate::CancelSource::new();
    let finished = std::sync::atomic::AtomicBool::new(false);
    let export = async {
        let result = Box::pin(runtime.project_export(
            &interrupted, &request, physical.clone(), &interrupted_cancel,
        )).await;
        finished.store(true, std::sync::atomic::Ordering::Release);
        result
    };
    let interrupt_after_effect = async {
        while !finished.load(std::sync::atomic::Ordering::Acquire) {
            if interrupted.join(".staging/source-1").exists() {
                interrupted_cancel.cancel();
                return true;
            }
            tokio::task::yield_now().await;
        }
        false
    };
    let (result, interrupted_after_effect) = tokio::join!(export, interrupt_after_effect);
    assert!(interrupted_after_effect, "cancellation must exercise an actual staged source effect");
    assert!(result.is_err());
    assert!(!interrupted.join("project.json").exists());
    assert!(ProjectBundleReader::open(&interrupted, request.maximum_document_bytes).is_err());
    drop(reader);
    store.remove_isolated_fixture().await.unwrap();
}

#[test]
fn source_sinks_share_exact_receipts_and_refuse_before_copying() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("bundle");
    let token = pse_columnar::CancellationToken::new();
    let mut count = SourceSink::new(None, 9, 8);
    let mut export = SourceSink::new(Some(&destination), 9, 8);
    for (id, bytes) in [("one", b"abc".as_slice()), ("two", b"defghi".as_slice())] {
        count.emit(id, bytes, &token).unwrap();
        export.emit(id, bytes, &token).unwrap();
    }
    assert_eq!(count.bytes, 9);
    assert_eq!(count.receipts, export.receipts);
    for (id, bytes, digest) in &export.receipts {
        let stored = std::fs::read(destination.join(".staging").join(id)).unwrap();
        assert_eq!(stored.len() as u64, *bytes);
        assert_eq!(&pse_ids::encoding_checksum(&stored).content_hash().to_hex(), digest);
    }
    assert!(count.emit("over", b"x", &token).is_err());
    assert!(export.emit("over", b"x", &token).is_err());
    assert!(!destination.join(".staging/over").exists());
    assert!(!destination.join("project.json").exists());
    let untouched = root.path().join("untouched");
    let mut short = SourceSink::new(Some(&untouched), 2, 8);
    assert_eq!(short.document_limit(), 2);
    assert!(short.emit("one", b"abc", &token).is_err());
    assert!(!untouched.exists());
}

#[test]
fn source_sink_cancellation_leaves_no_published_or_copied_unit() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("bundle");
    let token = pse_columnar::CancellationToken::new();
    token.cancel();
    let mut sink = SourceSink::new(Some(&destination), 10, 10);
    assert!(sink.emit("one", b"abc", &token).is_err());
    assert!(sink.receipts.is_empty());
    assert!(!destination.exists());
}
