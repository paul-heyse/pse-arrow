// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Contained independent reader and consumed-byte/import-copy boundaries.
use super::*;
use crate::workflow;

async fn bundle()->(tempfile::TempDir,ProjectManifest) {
    let runtime=workflow::tests::runtime();let cancel=crate::CancelSource::new();
    let (physical_documents,documents)=workflow::worker_tests::sources("package algebraic {def Fixed {param x:Scalar=2; eq fixed:x==2;}}");
    let physical=Box::pin(runtime.physical_from_sources(&physical_documents,&cancel)).await.unwrap();
    let package=Box::pin(runtime.package_from_sources(&[documents],physical.clone(),&cancel)).await.unwrap();
    let replay=package.replay_source().unwrap();
    let root=tempfile::tempdir().unwrap();std::fs::create_dir(root.path().join("members")).unwrap();
    let mut manifest=ProjectManifest{
        version:Version,origin:ProjectSelection{issuing_store:"unavailable-origin".into(),project:"original-project".into(),reference:workflow::ProjectReference::Branch{key:"main".into()},problem:"original-problem".into(),revision:"original-revision".into(),source_identity:package.revision.identity(),physical_context:physical.identity()},
        sources:vec![ProjectSourceClosure{revision:"original-revision".into(),source_identity:package.revision.identity().to_string(),physical_context:physical.identity().to_string(),documents:vec![],replay_source:"source".into(),physical_documents:vec![]}],
        datasets:vec![],research_selections:vec![],publications:vec![],contexts:vec![],studies:vec![],reproducers:vec![],providers:vec![],artifact_origins:vec![],members:vec![],maximum_copy_bytes:64<<20,omissions:vec![],offline_complete:true,executable_complete:true,descriptive_only:false,
    };
    fn add(root:&Path,manifest:&mut ProjectManifest,id:String,codec:&str,bytes:&[u8]) {
        let relative_path=format!("members/{}.bin",manifest.members.len());std::fs::write(root.join(&relative_path),bytes).unwrap();
        manifest.members.push(ProjectBundleMember{id,relative_path,codec:codec.into(),copied_bytes:bytes.len() as u64,digest:pse_ids::encoding_checksum(bytes).content_hash().to_hex(),result:None,artifact:None,artifact_metadata:None});
    }
    add(root.path(),&mut manifest,"source".into(),"replay-source.v1",&serde_json::to_vec(&replay).unwrap());
    for (i,(path,bytes)) in physical_documents.into_iter().enumerate() {
        let id=format!("physical-{i}");add(root.path(),&mut manifest,id.clone(),"authored-document.v1",&bytes);
        manifest.sources[0].physical_documents.push(ProjectDocumentMember{package:0,path,member:id});
    }
    std::fs::write(root.path().join("project.json"),serde_json::to_vec(&manifest).unwrap()).unwrap();
    (root,manifest)
}
fn save(root:&Path,manifest:&ProjectManifest) {std::fs::write(root.join("project.json"),serde_json::to_vec(manifest).unwrap()).unwrap();}

#[tokio::test]
async fn project_portability_offline_reader_uses_only_contained_bytes_and_detects_later_mutation() {
    let (root,manifest)=bundle().await;
    // This has no canonical deployment handle or reachable original service.
    let reader=ProjectBundleReader::open(root.path(),16<<20).unwrap();
    assert_eq!(reader.inspect().origin.issuing_store,"unavailable-origin");
    let original=reader.member_bytes("source",16<<20).unwrap();assert!(!original.is_empty());
    let path=root.path().join(&manifest.members[0].relative_path);
    let mut changed=original.clone();changed[0]^=1;std::fs::write(path,changed).unwrap();
    assert!(reader.member_bytes("source",16<<20).is_err(),"cached inspection cannot authenticate subsequently changed consumed bytes");
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err());
}
#[tokio::test]
async fn project_portability_missing_required_closure_unknown_codec_and_escaped_path_refuse_complete_manifest() {
    let (root,manifest)=bundle().await;
    let mut missing=manifest.clone();missing.members.remove(0);save(root.path(),&missing);
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err());
    let mut unknown=manifest.clone();unknown.members[0].codec="guessed-codec.v99".into();save(root.path(),&unknown);
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err());
    let mut escape=manifest;escape.members[0].relative_path="members/../../outside".into();save(root.path(),&escape);
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err());
}
#[tokio::test]
async fn project_portability_symlink_escape_and_interrupted_archive_copy_refuse() {
    let (root,manifest)=bundle().await;
    let reader=ProjectBundleReader::open(root.path(),16<<20).unwrap();
    let cancel=crate::CancelSource::new();cancel.cancel();
    assert!(reader.archive(&cancel).is_err());
    assert!(!root.path().join(".project-manifest").exists());
    let path=root.path().join(&manifest.members[0].relative_path);
    std::fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(root.path().join("project.json"),&path).unwrap();
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err());
    assert!(reader.member_bytes("source",16<<20).is_err());
}
#[tokio::test]
async fn project_portability_unsupported_provider_is_inspectable_and_never_executable() {
    let (root,mut manifest)=bundle().await;
    manifest.providers.push(ProjectProviderRequirement{name:"unavailable-provider".into(),identity:"original-required-factory".into(),supported:false});manifest.executable_complete=false;save(root.path(),&manifest);
    let reader=ProjectBundleReader::open(root.path(),16<<20).unwrap();
    assert!(!reader.inspect().executable_complete);
    let runtime=workflow::tests::runtime();
    assert!(Box::pin(runtime.project_import_issue(&reader,"refused","main")).await.is_err());
    manifest.executable_complete=true;save(root.path(),&manifest);
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err(),"unsupported provider cannot be advertised as complete executable closure");
}

#[tokio::test]
async fn project_portability_selected_reproducer_executes_only_on_explicit_owner_request() {
    let runtime=workflow::tests::runtime();let cancel=crate::CancelSource::new();
    // Reuse the reproducer owner's actual supported pre-handle diagnostic fixture and exact predicate.
    let command=workflow::reproducer::tests::text_command();
    let first=Box::pin(runtime.diagnose_documents(command.operation.clone(),&cancel)).await.unwrap();
    let original=first.failure.unwrap();
    let extracted=workflow::DiagnosticReproducer::extract_text(command,Some(original),None).unwrap();
    let (root,mut manifest)=bundle().await;manifest.reproducers.push(extracted.reproducer.clone());save(root.path(),&manifest);
    let (reader,counts)=crate::math::counted(async {ProjectBundleReader::open(root.path(),16<<20)}).await;
    assert_eq!(counts,Default::default(),"opening the bundle cannot reconstruct or execute its selected reproducer");
    let reader=reader.unwrap();let request=reader.reproducer(0).unwrap();
    assert_eq!(serde_json::to_value(&request.predicate).unwrap(),serde_json::to_value(&extracted.reproducer.predicate).unwrap());
    let reproduced=Box::pin(runtime.run_reproducer(request,&cancel)).await.unwrap();
    assert_ne!(reproduced.operation,first.operation,"reproduction is a new explicitly requested operation");
    assert_eq!(reproduced.status,workflow::ReproductionStatus::OriginalFailureReproduced);
    assert!(reproduced.failure.is_some());assert!(reproduced.run_id.is_none(),"the original selected failure occurred before any scientific run handle");
}

#[test]
fn project_portability_interrupted_final_manifest_publication_leaves_only_partial_staging() {
    let root=tempfile::tempdir().unwrap();std::fs::create_dir(root.path().join("members")).unwrap();
    std::fs::write(root.path().join("members/copied.bin"),b"copied bytes").unwrap();
    let result=contained_copy::publish_document(root.path(),".project-manifest","project.json",b"complete manifest must not appear",||Err(std::io::Error::new(std::io::ErrorKind::Interrupted,"interrupted final publication")));
    assert!(result.is_err());assert!(!root.path().join("project.json").exists());
    assert!(root.path().join(".project-manifest").exists(),"staged bytes are honest partial output");
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err());
}

#[cfg(feature="canonical-tests")]
#[tokio::test]
async fn project_portability_expired_original_recovery_keeps_exact_receipt_and_fresh_destination() {
    let runtime=workflow::durable_tests::durable_runtime();let cancel=crate::CancelSource::new();
    let (root,_)=bundle().await;let reader=ProjectBundleReader::open(root.path(),16<<20).unwrap();
    let intent=Box::pin(runtime.project_import_issue(&reader,"Recovered import","main")).await.unwrap();
    let local=runtime.clone().with_durability(workflow::DurabilitySelection::Ephemeral).unwrap();
    let physical_documents=reader.physical_documents().unwrap();
    let physical=Box::pin(local.physical_from_sources(&physical_documents,&cancel)).await.unwrap();
    let admitted=reader.replay_source().unwrap().reconstruct(&local,physical,BTreeMap::new(),&cancel).unwrap();
    let source=Box::pin(admitted.publish_portable_import(&runtime,&intent,&cancel)).await.unwrap();
    let revision=source.canonical_revision().unwrap().clone();let store=runtime.canonical_store().unwrap();
    // An authenticated original occurrence that never began copying can already be expired.
    // Its original identity is issued before any effect; no live header/clock is edited.
    let bytes=b"interrupted original copy".to_vec();
    let mut original=store.new_artifact(&revision,"project-import-evidence",encode(&(1u8,intent.bundle_digest()),131072).unwrap(),1,bytes.len() as u64).await.unwrap();
    original.creation_expires_at=1;
    original.key=format!("pse.artifact.v1:{}:{}:{}",original.primary_authority,original.creation_nonce,original.creation_expires_at);
    let blocks=vec![pse_operations::canonical_artifacts::artifact_block(&original.key,0,"project-import-evidence",bytes,intent.bundle_digest(),0)];
    pse_operations::canonical_artifacts::seal_artifact_request(&mut original,std::slice::from_ref(&revision),&[],&blocks,"project",intent.operation());
    store.project_import_bind_artifact(&intent,&original).await.unwrap();
    assert!(!runtime.project_import_status(&intent).await.unwrap().unwrap().project_complete);
    assert!(Box::pin(runtime.project_import(&reader,&intent,&cancel)).await.is_err(),"an expired original is not silently renewed");
    let collected=Box::pin(runtime.project_import_recover(&intent,&original,&cancel)).await.unwrap();
    assert_eq!(collected.state,ProjectImportRecoveryState::OriginalCollected);
    assert_eq!(collected.import.retired_originals,vec![original.clone()]);
    assert!(!collected.import.project_complete);assert!(collected.import.artifact_intent.is_none());
    let imported=Box::pin(runtime.project_import(&reader,&intent,&cancel)).await.unwrap();
    assert!(imported.project_complete);assert_ne!(imported.artifact_intent.as_ref().unwrap().key,original.key);
    assert_eq!(imported.project,intent.destinations().0);assert_eq!(imported.revision,intent.operation());
    // Retrying a lost recovery reply returns its old receipt and cannot abort the newer active occurrence.
    let old_reply=Box::pin(runtime.project_import_recover(&intent,&original,&cancel)).await.unwrap();
    assert_eq!(old_reply.state,ProjectImportRecoveryState::OriginalCollected);assert!(old_reply.import.project_complete);
    assert_eq!(old_reply.import.artifact_intent,imported.artifact_intent);
    let duplicate=Box::pin(runtime.project_import(&reader,&intent,&cancel)).await.unwrap();assert_eq!(duplicate.project,imported.project);
    std::fs::remove_dir_all(root.path()).unwrap();
    let reopened=Box::pin(runtime.project_import_open(&intent,&cancel)).await.unwrap();
    assert_eq!(reopened.canonical_revision().unwrap().key,imported.revision,"current copied artifact/source ownership is independent of supplied bundle lifetime");
    let repeated=Box::pin(runtime.project_import(&reader,&intent,&cancel)).await.unwrap();
    assert_eq!(repeated.project,imported.project,"lost completed reply settles before missing original pack preflight");
    store.remove_isolated_fixture().await.unwrap();
}

#[cfg(feature="canonical-tests")]
#[tokio::test]
async fn project_portability_active_original_settles_after_expiry_without_copy_or_original_pack() {
    let runtime=workflow::durable_tests::durable_runtime();let cancel=crate::CancelSource::new();
    let (root,_)=bundle().await;let reader=ProjectBundleReader::open(root.path(),16<<20).unwrap();
    let intent=Box::pin(runtime.project_import_issue(&reader,"Lost activation reply","main")).await.unwrap();
    let local=runtime.clone().with_durability(workflow::DurabilitySelection::Ephemeral).unwrap();
    let physical_documents=reader.physical_documents().unwrap();
    let physical=Box::pin(local.physical_from_sources(&physical_documents,&cancel)).await.unwrap();
    let admitted=reader.replay_source().unwrap().reconstruct(&local,physical,BTreeMap::new(),&cancel).unwrap();
    let source=Box::pin(admitted.publish_portable_import(&runtime,&intent,&cancel)).await.unwrap();
    let revision=source.canonical_revision().unwrap().clone();let store=runtime.canonical_store().unwrap();
    let mut archive=reader.archive(&cancel).unwrap();let mut bytes=Vec::new();archive.read_to_end(&mut bytes).unwrap();
    let chunk=pse_operations::canonical_files::FILE_TARGET_BYTES;
    let mut original=store.new_artifact(&revision,"project-import-evidence",encode(&(1u8,intent.bundle_digest()),131072).unwrap(),bytes.len().div_ceil(chunk) as u64,bytes.len() as u64).await.unwrap();
    // Shorten an unissued fixture's original allowance before framing/binding it; never renew an issued clock.
    original.creation_expires_at-=57_000_000;
    original.key=format!("pse.artifact.v1:{}:{}:{}",original.primary_authority,original.creation_nonce,original.creation_expires_at);
    let blocks=bytes.chunks(chunk).enumerate().map(|(i,b)|pse_operations::canonical_artifacts::artifact_block(&original.key,i as u64,"project-import-evidence",b.to_vec(),intent.bundle_digest(),0)).collect::<Vec<_>>();
    pse_operations::canonical_artifacts::seal_artifact_request(&mut original,std::slice::from_ref(&revision),&[],&blocks,"project",intent.operation());
    store.project_import_bind_artifact(&intent,&original).await.unwrap();
    store.persist_artifact(&original,std::slice::from_ref(&revision),&[],&blocks,"project",intent.operation()).await.unwrap();
    assert!(!runtime.project_import_status(&intent).await.unwrap().unwrap().project_complete,"simulate reply loss between original artifact activation and project finish");
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
    while store.artifact_creation_status(&original).await.unwrap().window_open {
        assert!(std::time::Instant::now()<deadline);tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    std::fs::remove_dir_all(root.path()).unwrap();
    let completed=Box::pin(runtime.project_import(&reader,&intent,&cancel)).await.unwrap();
    assert!(completed.project_complete);assert_eq!(completed.artifact_intent,Some(original.clone()));assert!(completed.retired_originals.is_empty());
    let reopened=Box::pin(runtime.project_import_open(&intent,&cancel)).await.unwrap();
    assert_eq!(reopened.canonical_revision().unwrap().key,intent.operation());
    let recovered=Box::pin(runtime.project_import_recover(&intent,&original,&cancel)).await.unwrap();
    assert_eq!(recovered.state,ProjectImportRecoveryState::ProjectComplete,"active exact originals are finished, never aborted");
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn project_portability_artifact_original_codec_digest_and_scientific_extent_survive_copy() {
    let (root,mut manifest)=bundle().await;
    let block=pse_operations::canonical_artifacts::artifact_block("origin-artifact",0,"origin-fitted-cells",b"{\"estimate\":11}".to_vec(),"original-encoding-provenance",1);
    let original=pse_operations::canonical_artifacts::Artifact{key:block.artifact.clone(),kind:"original-copied-evidence".into(),revision:manifest.origin.revision.clone(),primary_problem:manifest.origin.problem.clone(),primary_authority:"descriptive-origin-authority".into(),creation_nonce:"original-nonce".into(),creation_expires_at:1,creation_request_digest:"original-request".into(),interpretation:"original-registry".into(),provenance:vec![1].into(),block_count:1,payload_bytes:block.payload.len() as u64,active:true,retiring:false};
    manifest.artifact_origins.push(original);
    let relative_path="members/evidence.bin".to_owned();std::fs::write(root.path().join(&relative_path),block.payload.as_slice()).unwrap();
    manifest.members.push(ProjectBundleMember{id:"original-evidence".into(),relative_path,codec:"artifact-block.v1".into(),copied_bytes:block.payload.len() as u64,digest:pse_ids::encoding_checksum(block.payload.as_slice()).content_hash().to_hex(),result:None,artifact:Some((block.artifact.clone(),0)),artifact_metadata:Some(ProjectArtifactBlockMetadata{relation:block.relation.clone(),payload_digest:block.digest.clone(),original_digest:block.original_digest.clone(),rows:block.rows,encoding:block.encoding().into(),schema:block.schema().into()})});
    save(root.path(),&manifest);
    let reader=ProjectBundleReader::open(root.path(),16<<20).unwrap();
    let metadata=reader.inspect().members.last().unwrap().artifact_metadata.as_ref().unwrap();
    assert_eq!(metadata.rows,1);assert_eq!(metadata.relation,"origin-fitted-cells");assert_eq!(metadata.original_digest,"original-encoding-provenance");assert_eq!(metadata.payload_digest,block.digest);
    assert_eq!(reader.member_bytes("original-evidence",1024).unwrap(),block.payload.as_slice());
    let metadata=manifest.members.last_mut().unwrap().artifact_metadata.as_mut().unwrap();metadata.payload_digest="00".repeat(32);save(root.path(),&manifest);
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err());
    let metadata=manifest.members.last_mut().unwrap().artifact_metadata.as_mut().unwrap();metadata.payload_digest=block.digest;metadata.encoding="unknown-evidence.v99".into();save(root.path(),&manifest);
    assert!(ProjectBundleReader::open(root.path(),16<<20).is_err());
}
