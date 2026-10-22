// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact local Surrealism artifact admission and installed-module attestation.

use crate::canonical::{CanonicalError, CanonicalStore, MetadataResponse, bounded_query};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{io::Read, path::{Path, PathBuf}};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StudyModuleReceipt {
    kind:String,
    artifact:PathBuf,
    pub(crate) sha256:String,
    bytes:u64,
    sdk_version:String,
    abi:u32,
    inputs:serde_json::Map<String,serde_json::Value>,
    limits:ModuleLimits,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ModuleLimits {
    max_memory_bytes:u64,
    max_pool_size:u32,
    max_execution_time_ms:u64,
    max_kv_entries:u64,
    max_kv_value_bytes:u64,
    strict_timeout:bool,
}
struct ModuleDeclaration {abi:u32,sdk:String,limits:ModuleLimits}
impl ModuleDeclaration {
    fn read()->Result<Self,CanonicalError> {
        let manifest:toml::Value=toml::from_str(include_str!("../../pse-study-module/surrealism.toml")).map_err(invalid)?;
        let abi=manifest.get("abi").and_then(toml::Value::as_integer).and_then(|value|u32::try_from(value).ok()).ok_or_else(||invalid("declared ABI absent"))?;
        let mut capabilities=manifest.get("capabilities").and_then(toml::Value::as_table).cloned().ok_or_else(||invalid("declared capabilities absent"))?;
        for name in ["allow_scripting","allow_arbitrary_queries"] {
            if capabilities.remove(name)!=Some(toml::Value::Boolean(false)) {return Err(invalid("pure module cannot enable host side effects"));}
        }
        for name in ["allow_functions","allow_net"] {
            if capabilities.remove(name)!=Some(toml::Value::Array(Vec::new())) {return Err(invalid("pure module cannot enable host attachments"));}
        }
        let time=capabilities.remove("max_execution_time").ok_or_else(||invalid("declared execution limit absent"))?;
        capabilities.insert("max_execution_time_ms".into(),time);
        let limits:ModuleLimits=toml::Value::Table(capabilities).try_into().map_err(invalid)?;
        let workspace:toml::Value=toml::from_str(include_str!("../../../Cargo.toml")).map_err(invalid)?;
        let sdk=workspace.get("workspace").and_then(|value|value.get("dependencies")).and_then(|value|value.get("surrealism"))
            .and_then(toml::Value::as_str).and_then(|value|value.strip_prefix('=')).filter(|value|!value.is_empty()).ok_or_else(||invalid("exact workspace SDK pin absent"))?.to_owned();
        Ok(Self{abi,sdk,limits})
    }
}
fn invalid(message:impl std::fmt::Display)->CanonicalError {
    CanonicalError::Configuration(format!("study module: {message}"))
}
impl StudyModuleReceipt {
    pub(crate) fn read(state:&Path)->Result<Self,CanonicalError> {
        #[derive(Deserialize)]
        struct Selection {study_module:StudyModuleReceipt}
        let bytes=std::fs::read(state.join("config.json")).map_err(invalid)?;
        let receipt:Selection=serde_json::from_slice(&bytes).map_err(invalid)?;
        receipt.study_module.validate(state)?;
        Ok(receipt.study_module)
    }
    fn validate(&self,state:&Path)->Result<(),CanonicalError> {
        let declared=ModuleDeclaration::read()?;
        if self.kind!="pse-study-module-v1" || self.sdk_version!=declared.sdk || self.abi!=declared.abi
            || self.inputs.is_empty() || self.sha256.len()!=64
            || !self.sha256.bytes().all(|byte|byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || self.bytes==0 || self.bytes>64*1024*1024 {
            return Err(invalid("pinned artifact identity, ABI or source provenance differs"));
        }
        if self.limits!=declared.limits {
            return Err(invalid("finite pure-module limits differ"));
        }
        if !self.artifact.is_absolute() || self.artifact.file_name().and_then(|name|name.to_str())!=Some("study.surli")
            || std::fs::canonicalize(&self.artifact).map_err(invalid)?!=self.artifact {
            return Err(invalid("artifact must be the exact absolute non-symlink study.surli"));
        }
        let parent=self.artifact.parent().ok_or_else(||invalid("artifact parent absent"))?;
        let metadata=std::fs::symlink_metadata(&self.artifact).map_err(invalid)?;
        let directory=std::fs::symlink_metadata(parent).map_err(invalid)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len()!=self.bytes
            || !directory.is_dir() || directory.file_type().is_symlink() {
            return Err(invalid("artifact file, private directory or exact byte extent differs"));
        }
        #[cfg(unix)] {
            use std::os::unix::fs::MetadataExt;
            let owner=std::fs::metadata(state).map_err(invalid)?.uid();
            if metadata.mode()&0o377!=0 || metadata.nlink()!=1 || metadata.uid()!=owner
                || directory.mode()&0o077!=0 || directory.uid()!=owner {
                return Err(invalid("artifact must be private, immutable and owned by the selected state owner"));
            }
        }
        let mut file=std::fs::File::open(&self.artifact).map_err(invalid)?;
        let opened=file.metadata().map_err(invalid)?;
        #[cfg(unix)] {
            use std::os::unix::fs::MetadataExt;
            if opened.dev()!=metadata.dev() || opened.ino()!=metadata.ino() {
                return Err(invalid("artifact identity changed while opening"));
            }
        }
        let mut hash=Sha256::new();
        let mut buffer=[0_u8;64*1024];
        let mut length=0_u64;
        loop {
            let read=file.read(&mut buffer).map_err(invalid)?;
            if read==0 {break;}
            length=length.checked_add(read as u64).ok_or_else(||invalid("artifact extent overflow"))?;
            if length>self.bytes {return Err(invalid("artifact grew while hashing"));}
            hash.update(&buffer[..read]);
        }
        if length!=self.bytes || hash.finalize().iter().map(|byte|format!("{byte:02x}")).collect::<String>()!=self.sha256 {
            return Err(invalid("artifact SHA256 or actual extent differs"));
        }
        Ok(())
    }
    pub(crate) fn bucket(&self)->String {format!("pse_study_{}",self.sha256)}
    pub(crate) fn backend(&self)->Result<String,CanonicalError> {
        let parent=self.artifact.parent().and_then(Path::to_str).ok_or_else(||invalid("artifact directory must be UTF-8"))?;
        Ok(format!("file:{parent}"))
    }
    pub(crate) fn declarations(&self,overwrite:bool)->String {
        let kind=if overwrite {" OVERWRITE"} else {""};
        let bucket=self.bucket();
        format!("DEFINE BUCKET{kind} {bucket} READONLY BACKEND $study_module_backend PERMISSIONS NONE;\nDEFINE MODULE{kind} mod::pse FROM f\"{bucket}:/study.surli\" UNSIGNED PERMISSIONS FULL;\n")
    }
}
impl CanonicalStore {
    /// Install only under the exact selected maintenance owner. Publication of
    /// the schema/module pair remains a separate validated activation.
    #[cfg(feature="canonical-maintenance")]
    pub async fn install_study_module(&self)->Result<(),CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            crate::canonical::within_clock(crate::canonical::original_deadline(crate::canonical::ACTIVATION_REQUEST_TIMEOUT),async {
                let before=self.maintenance_config()?;
                let receipt=StudyModuleReceipt::read(&self.state_path)?;
                let response=bounded_query(self.db.query(format!("BEGIN; fn::pse_execution_v1::deadline($pse_rpc_expires_at); LET $marker=SELECT study_module_sha256 FROM ONLY canonical_interpretations:current FOR UPDATE; IF $marker!=NONE AND $marker.study_module_sha256!=$study_module_sha256 {{THROW 'PSE_REFUSAL:activation_predecessor_changed';}}; {} fn::pse_execution_v1::deadline($pse_rpc_expires_at); COMMIT;",receipt.declarations(true)))
                    .bind(("study_module_backend",receipt.backend()?)).bind(("study_module_sha256",receipt.sha256.clone()))).await?;
                crate::canonical::complete_response(response)?;
                if self.maintenance_config()?!=before {return Err(CanonicalError::Quiesced);}
                self.validate_study_module(&receipt).await
            }).await
        }).await
    }
    /// Catalog structure, not rendered SQL text, attests the exact immutable
    /// artifact path and only supported read-only policy export.
    pub(crate) async fn validate_study_module(&self,receipt:&StudyModuleReceipt)->Result<(),CanonicalError> {
        let mut response=bounded_query(self.db.metadata_query(MetadataResponse::Header,
            "LET $info=INFO FOR DB STRUCTURE; LET $b=$info.buckets.filter(|$row|$row.name=$bucket); LET $m=$info.modules.filter(|$row|$row.name='pse'); RETURN array::len($b)=1 AND array::len($m)=1 AND $b[0].readonly=true AND $b[0].backend=$backend AND $m[0].unsigned=true AND $m[0].executable.type='surrealism' AND $m[0].executable.bucket=$bucket AND $m[0].executable.key='study.surli' AND array::len($m[0].exports)=1 AND $m[0].exports[0].name='study::readiness' AND $m[0].exports[0].writeable=false;")
            .bind(("bucket",receipt.bucket())).bind(("backend",receipt.backend()?))).await?;
        if response.take::<Option<bool>>(0)?!=Some(true) {return Err(invalid("installed module, readonly bucket or pure export differs"));}
        Ok(())
    }
}

#[cfg(all(test,unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    struct Fixture { state:PathBuf, receipt:StudyModuleReceipt }
    impl Fixture {
        fn new()->Self {
            let state=std::env::temp_dir().join(format!("pse-study-module-{}",uuid::Uuid::new_v4()));
            std::fs::create_dir(&state).unwrap();
            let parent=state.join("module");std::fs::create_dir(&parent).unwrap();
            std::fs::set_permissions(&parent,std::fs::Permissions::from_mode(0o700)).unwrap();
            let artifact=parent.join("study.surli");let payload=b"exact immutable artifact";
            std::fs::write(&artifact,payload).unwrap();
            std::fs::set_permissions(&artifact,std::fs::Permissions::from_mode(0o400)).unwrap();
            let sha256=Sha256::digest(payload).iter().map(|byte|format!("{byte:02x}")).collect();
            let declared=ModuleDeclaration::read().unwrap();
            let receipt=StudyModuleReceipt {kind:"pse-study-module-v1".into(),artifact,sha256,bytes:payload.len() as u64,sdk_version:declared.sdk,abi:declared.abi,
                inputs:serde_json::Map::from_iter([("source".into(),serde_json::Value::String("fixture".into()))]),
                limits:declared.limits};
            Self{state,receipt}
        }
    }
    impl Drop for Fixture {fn drop(&mut self) {std::fs::remove_dir_all(&self.state).unwrap();}}
    #[test]
    fn study_module_receipt_binds_hash_private_extent_and_pinned_limits() {
        let fixture=Fixture::new();fixture.receipt.validate(&fixture.state).unwrap();
        let mut wrong=fixture.receipt.clone();wrong.sha256="0".repeat(64);assert!(wrong.validate(&fixture.state).is_err());
        let mut wrong=fixture.receipt.clone();wrong.bytes+=1;assert!(wrong.validate(&fixture.state).is_err());
        let mut wrong=fixture.receipt.clone();wrong.limits.max_pool_size+=1;assert!(wrong.validate(&fixture.state).is_err());
        let mut wrong=fixture.receipt.clone();wrong.abi=1;assert!(wrong.validate(&fixture.state).is_err());
        assert!(fixture.receipt.declarations(false).contains(&fixture.receipt.bucket()));
        assert!(fixture.receipt.declarations(true).contains("DEFINE MODULE OVERWRITE mod::pse"));
        std::fs::set_permissions(&fixture.receipt.artifact,std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(fixture.receipt.validate(&fixture.state).is_err());
    }
    #[test]
    fn study_module_receipt_rejects_symlink_hardlink_and_shared_directory() {
        let fixture=Fixture::new();let alias=fixture.state.join("alias");
        std::fs::hard_link(&fixture.receipt.artifact,&alias).unwrap();assert!(fixture.receipt.validate(&fixture.state).is_err());
        std::fs::remove_file(&alias).unwrap();
        let parent=fixture.receipt.artifact.parent().unwrap();
        std::fs::set_permissions(parent,std::fs::Permissions::from_mode(0o755)).unwrap();assert!(fixture.receipt.validate(&fixture.state).is_err());
        std::fs::set_permissions(parent,std::fs::Permissions::from_mode(0o700)).unwrap();
        std::os::unix::fs::symlink(parent,&alias).unwrap();let mut wrong=fixture.receipt.clone();wrong.artifact=alias.join("study.surli");
        assert!(wrong.validate(&fixture.state).is_err());
    }
}
