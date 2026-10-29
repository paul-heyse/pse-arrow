// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use flate2::{Compression, write::GzEncoder};
use std::io::Write;
use std::os::unix::fs::symlink;

fn spec() -> ToolSpec {
    ToolSpec { version:"test".into(),url:"https://example.invalid/release".into(),sha256:"0".repeat(64),binary_sha256:None,member:"tool".into(),archive:Some("tar.gz".into()),prefix:None,max_archive_bytes:None,max_binary_bytes:None }
}
fn tar_file(directory:&Path,entries:&[(&str,&[u8])],link:bool)->PathBuf {
    let path=directory.join("release.tar.gz");let gzip=GzEncoder::new(File::create(&path).unwrap(),Compression::fast());let mut builder=tar::Builder::new(gzip);
    for (name,bytes) in entries {let mut header=tar::Header::new_gnu();header.set_size(bytes.len() as u64);header.set_mode(0o755);header.set_cksum();builder.append_data(&mut header,*name,*bytes).unwrap();}
    if link {let mut header=tar::Header::new_gnu();header.set_entry_type(tar::EntryType::Symlink);header.set_mode(0o755);header.set_size(0);header.set_cksum();builder.append_link(&mut header,"release/alias","/outside").unwrap();}
    builder.into_inner().unwrap().finish().unwrap();path
}
#[test]
fn tool_pins_and_paths_retain_existing_layout_and_cargo_selections(){
    let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");let temporary=tempfile::tempdir().unwrap();let env=Environment::from([("HOME".into(),temporary.path().into())]);
    let rr=binary_path(&root,Tool::Rr,&env).unwrap();assert!(rr.ends_with(".local/share/pse/tools/rr/5.10.0/bin/rr"));assert!(binary_path(&root,Tool::TraceProcessor,&env).unwrap().ends_with("pse/tools/perfetto/58.2/trace_processor_shell"));
    let selected=cargo_tool_specs(&root,&["cargo-nextest".into(),"other-tool".into()]).unwrap();assert_eq!(selected,["cargo-nextest@=0.9.146","other-tool"]);assert!(Tool::parse("cargo-tools").is_err());
    assert!(verify(&root,Tool::Rr,&env).is_err());assert!(!temporary.path().join(".local").exists(),"passive verification created tool directories");
}
#[test]
fn selected_archive_binary_limits_hashes_and_changed_install_refuse(){
    let directory=tempfile::tempdir().unwrap();let archive=tar_file(directory.path(),&[("release/tool",b"original executable")],false);let mut selected=spec();selected.sha256=digest(&archive).unwrap();selected.binary_sha256=Some(Sha256::digest(b"original executable").iter().map(|byte|format!("{byte:02x}")).collect());
    let target=directory.path().join("installed");let stage=tempfile::tempdir_in(directory.path()).unwrap();selected.max_archive_bytes=Some("1".into());assert!(publish(&archive,&target,&selected,stage.path()).is_err());assert!(!target.exists());
    selected.max_archive_bytes=None;selected.max_binary_bytes=Some("1".into());assert!(publish(&archive,&target,&selected,stage.path()).is_err());assert!(!target.exists());
    selected.max_binary_bytes=None;let stage=tempfile::tempdir_in(directory.path()).unwrap();publish(&archive,&target,&selected,stage.path()).unwrap();assert_eq!(fs::read(&target).unwrap(),b"original executable");verify_at(&target,&selected).unwrap();
    fs::write(&target,b"changed").unwrap();assert!(verify_at(&target,&selected).is_err());
}
#[test]
fn ambiguous_tar_binary_never_publishes(){
    let directory=tempfile::tempdir().unwrap();let archive=tar_file(directory.path(),&[("first/tool",b"first"),("second/tool",b"second")],false);let mut selected=spec();selected.sha256=digest(&archive).unwrap();let stage=tempfile::tempdir_in(directory.path()).unwrap();let target=directory.path().join("installed");assert!(publish(&archive,&target,&selected,stage.path()).is_err());assert!(!target.exists());
}
#[test]
fn portable_rr_retains_runtime_and_detects_mutation(){
    let directory=tempfile::tempdir().unwrap();let names=[("release/bin/rr",b"rr".as_slice()),("release/bin/rr_exec_stub",b"stub".as_slice()),("release/lib/rr/librrpreload.so",b"preload".as_slice()),("release/lib/rr/librrpage.so",b"page".as_slice()),("release/share/rr/core.xml",b"xml".as_slice())];let archive=tar_file(directory.path(),&names,false);let mut selected=spec();selected.archive=Some("portable-tar".into());selected.prefix=Some("release".into());selected.member="bin/rr".into();selected.sha256=digest(&archive).unwrap();let stage=tempfile::tempdir_in(directory.path()).unwrap();let target=directory.path().join("installed/bin/rr");publish(&archive,&target,&selected,stage.path()).unwrap();verify_at(&target,&selected).unwrap();assert!(directory.path().join("installed/share/rr/core.xml").is_file());fs::write(directory.path().join("installed/lib/rr/librrpage.so"),b"changed").unwrap();assert!(verify_at(&target,&selected).is_err());
}
#[test]
fn portable_links_incomplete_runtime_and_path_aliases_refuse(){
    let directory=tempfile::tempdir().unwrap();let mut selected=spec();selected.archive=Some("portable-tar".into());selected.prefix=Some("release".into());selected.member="bin/rr".into();
    for link in [false,true]{let archive=tar_file(directory.path(),&[("release/bin/rr",b"rr")],link);selected.sha256=digest(&archive).unwrap();let stage=tempfile::tempdir_in(directory.path()).unwrap();assert!(publish(&archive,&directory.path().join("installed/bin/rr"),&selected,stage.path()).is_err());assert!(!directory.path().join("installed").exists());}
    let real=directory.path().join("real");fs::create_dir(&real).unwrap();let alias=directory.path().join("alias");symlink(&real,&alias).unwrap();assert!(no_symlinks(&alias.join("future-tool")).is_err());
}
#[test]
fn zip_member_allowance_and_nonregular_entry_refuse(){
    let directory=tempfile::tempdir().unwrap();let path=directory.path().join("archive.zip");let mut writer=zip::ZipWriter::new(File::create(&path).unwrap());writer.start_file("release/tool",zip::write::SimpleFileOptions::default().unix_permissions(0o755)).unwrap();writer.write_all(b"tool").unwrap();writer.finish().unwrap();
    let mut selected=spec();selected.archive=None;selected.sha256=digest(&path).unwrap();selected.max_binary_bytes=Some("3".into());let stage=tempfile::tempdir_in(directory.path()).unwrap();assert!(publish(&path,&directory.path().join("installed"),&selected,stage.path()).is_err());
    selected.max_binary_bytes=None;let stage=tempfile::tempdir_in(directory.path()).unwrap();publish(&path,&directory.path().join("installed"),&selected,stage.path()).unwrap();assert_eq!(fs::read(directory.path().join("installed")).unwrap(),b"tool");
    let path=directory.path().join("link.zip");let mut writer=zip::ZipWriter::new(File::create(&path).unwrap());writer.add_symlink("release/tool","/outside",zip::write::SimpleFileOptions::default()).unwrap();writer.finish().unwrap();selected.sha256=digest(&path).unwrap();let stage=tempfile::tempdir_in(directory.path()).unwrap();assert!(publish(&path,&directory.path().join("other"),&selected,stage.path()).is_err());
}
#[test]
fn publication_never_overwrites_an_incompatible_existing_tool(){
    let directory=tempfile::tempdir().unwrap();let archive=tar_file(directory.path(),&[("release/tool",b"new tool")],false);let mut selected=spec();selected.sha256=digest(&archive).unwrap();let target=directory.path().join("installed");fs::write(&target,b"existing").unwrap();fs::set_permissions(&target,fs::Permissions::from_mode(0o755)).unwrap();let stage=tempfile::tempdir_in(directory.path()).unwrap();assert!(publish(&archive,&target,&selected,stage.path()).is_err());assert_eq!(fs::read(&target).unwrap(),b"existing");
}

#[test]
fn unpinned_binary_requires_exact_archive_provenance_and_rejects_forged_receipt(){
    let directory=tempfile::tempdir().unwrap();let archive=tar_file(directory.path(),&[("release/tool",b"pinned archive member")],false);let mut selected=spec();selected.sha256=digest(&archive).unwrap();let target=directory.path().join("installed");
    fs::write(&target,b"arbitrary executable").unwrap();fs::set_permissions(&target,fs::Permissions::from_mode(0o755)).unwrap();assert!(verify_at(&target,&selected).is_err());
    let (source,receipt)=provenance_paths(&target).unwrap();fs::copy(&archive,&source).unwrap();let forged=BinaryReceipt{schema_version:1,version:selected.version.clone(),archive_sha256:selected.sha256.clone(),binary_sha256:digest(&target).unwrap(),source:selected.url.clone(),member:selected.member.clone()};fs::write(&receipt,serde_json::to_vec(&forged).unwrap()).unwrap();assert!(verify_at(&target,&selected).is_err(),"a matching self-asserted executable hash did not originate in the pinned archive");
    fs::remove_file(&target).unwrap();fs::remove_file(&receipt).unwrap();fs::remove_file(&source).unwrap();let stage=tempfile::tempdir_in(directory.path()).unwrap();publish(&archive,&target,&selected,stage.path()).unwrap();verify_at(&target,&selected).unwrap();
    fs::write(&target,b"same user replaced tool").unwrap();assert!(verify_at(&target,&selected).is_err());
}
#[test]
fn passive_provenance_check_refuses_archive_replacement_without_repair(){
    let directory=tempfile::tempdir().unwrap();let archive=tar_file(directory.path(),&[("release/tool",b"selected")],false);let mut selected=spec();selected.sha256=digest(&archive).unwrap();let target=directory.path().join("installed");let stage=tempfile::tempdir_in(directory.path()).unwrap();publish(&archive,&target,&selected,stage.path()).unwrap();let (source,receipt)=provenance_paths(&target).unwrap();let before=fs::read(&receipt).unwrap();fs::write(&source,b"changed archive").unwrap();assert!(verify_at(&target,&selected).is_err());assert_eq!(fs::read(&receipt).unwrap(),before);assert_eq!(fs::read(&source).unwrap(),b"changed archive");
}

#[test]
fn oversized_unrelated_tar_member_is_bounded_before_selected_binary_in_both_paths(){
    let directory=tempfile::tempdir().unwrap();let archive=directory.path().join("release.tar.gz");let gzip=GzEncoder::new(File::create(&archive).unwrap(),Compression::fast());let mut builder=tar::Builder::new(gzip);
    let mut header=tar::Header::new_gnu();let oversized=(64_u64<<20)+2;header.set_size(oversized);header.set_mode(0o644);header.set_cksum();builder.append_data(&mut header,"release/unrelated",std::io::repeat(0).take(oversized)).unwrap();
    let mut header=tar::Header::new_gnu();header.set_size(1);header.set_mode(0o755);header.set_cksum();builder.append_data(&mut header,"release/tool",b"x".as_slice()).unwrap();builder.into_inner().unwrap().finish().unwrap();
    let mut selected=spec();selected.sha256=digest(&archive).unwrap();selected.max_binary_bytes=Some("1".into());let stage=tempfile::tempdir_in(directory.path()).unwrap();let target=directory.path().join("installed");let error=publish(&archive,&target,&selected,stage.path()).unwrap_err();assert!(error.to_string().contains("expanded allowance"),"{error}");assert!(!target.exists());let error=archive_binary_digest(&archive,&selected).unwrap_err();assert!(error.to_string().contains("expanded allowance"),"{error}");
}
