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
        datasets:vec![],research_selections:vec![],publications:vec![],contexts:vec![],studies:vec![],reproducers:vec![],providers:vec![],members:vec![],maximum_copy_bytes:64<<20,omissions:vec![],offline_complete:true,executable_complete:true,descriptive_only:false,
    };
    fn add(root:&Path,manifest:&mut ProjectManifest,id:String,codec:&str,bytes:&[u8]) {
        let relative_path=format!("members/{}.bin",manifest.members.len());std::fs::write(root.join(&relative_path),bytes).unwrap();
        manifest.members.push(ProjectBundleMember{id,relative_path,codec:codec.into(),copied_bytes:bytes.len() as u64,digest:pse_ids::encoding_checksum(bytes).content_hash().to_hex(),result:None,artifact:None});
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
