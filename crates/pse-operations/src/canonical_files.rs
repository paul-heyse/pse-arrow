// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Opaque sealed file effects. Scientific codecs and interpretation remain with
//! Runtime; only an attached catalog receipt acknowledges durable publication.

use crate::{canonical::{CanonicalError, CanonicalStore, bounded_query}, canonical_codec as codec, canonical_execution::AttemptFence};
use pse_ids::{Frame, FramedHasher};
use std::{fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, sync::Arc};
use surrealdb::types::{Object, Value};

/// Completed file packing target, independent of RPC metadata limits.
pub const FILE_TARGET_BYTES: usize = pse_schema::catalog::substrate::RESULT_FILE_TARGET_BYTES;
/// Maximum physical file size, including native format metadata.
pub const FILE_BYTES: usize = pse_schema::catalog::substrate::RESULT_BLOCK_BYTES;
/// Native completed uncompressed Arrow IPC file encoding.
pub const ARROW_IPC_FILE: &str = "arrow-ipc-file";
/// Original JSON bytes, never a substituted binary seed format.
pub const OPAQUE_JSON: &str = "opaque-json";
/// Explicit owner-coded non-IPC opaque content.
pub const OPAQUE_BYTES: &str = "opaque-bytes";

/// Root-independent exact physical identity; restore changes placement only.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileDescriptor {
    /// Unique physical instance, never reused even for byte-equal content.
    pub instance: String,
    /// Exact bytes in the existing result-payload digest domain.
    pub digest: String,
    /// Complete physical extent.
    pub bytes: u64,
    /// Explicit native IPC, JSON or opaque encoding.
    pub encoding: String,
    /// Native IPC batch count; opaque files have zero.
    pub batch_count: u64,
    /// Codec-owner schema identity; opaque encodings use an empty string.
    pub schema: String,
    /// Original selected scientific interpretation.
    pub interpretation: String,
}
impl FileDescriptor {
    /// Validate physical coordinates, without claiming scientific admission.
    pub fn validate(&self) -> Result<(), CanonicalError> {
        uuid::Uuid::parse_str(&self.instance).map_err(configuration)?;
        if self.digest.len()!=64 || !self.digest.bytes().all(|b|b.is_ascii_hexdigit()) || self.bytes==0 || self.bytes>FILE_BYTES as u64 || self.interpretation!=crate::generated::surreal::INTERPRETATION {
            return Err(configuration("sealed file identity or extent invalid"));
        }
        match self.encoding.as_str() {
            ARROW_IPC_FILE if self.batch_count>0 && !self.schema.is_empty()=>Ok(()),
            OPAQUE_JSON|OPAQUE_BYTES if self.batch_count==0 && self.schema.is_empty()=>Ok(()),
            _=>Err(configuration("sealed file encoding coordinates invalid")),
        }
    }
    pub(crate) fn encode(&self)->Result<Object,CanonicalError>{
        self.validate()?;
        let mut row=Object::new();
        row.insert("instance",self.instance.clone()); row.insert("digest",self.digest.clone());
        row.insert("bytes",codec::encode_uint(self.bytes)?); row.insert("encoding",self.encoding.clone());
        row.insert("batch_count",codec::encode_uint(self.batch_count)?); row.insert("schema",self.schema.clone()); row.insert("interpretation",self.interpretation.clone()); Ok(row)
    }
}

/// One logical batch in an admitted immutable physical file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileBatch {
    /// Exact complete file descriptor.
    pub file: FileDescriptor,
    /// Native batch index; opaque content uses zero.
    pub index: u64,
}
impl FileBatch {
    /// Refuse out-of-file native indexes and nonzero opaque indexes.
    pub fn validate(&self)->Result<(),CanonicalError>{
        self.file.validate()?;
        if (self.file.encoding==ARROW_IPC_FILE && self.index>=self.file.batch_count) || (self.file.encoding!=ARROW_IPC_FILE && self.index!=0){return Err(configuration("logical batch outside sealed file"));} Ok(())
    }
}

/// Reserved original effect, issued before creating or writing a file.
#[derive(Clone, Debug)]
pub struct FilePublicationIntent {
    instance:String,
    intent:String,
    request_digest:String,
    owner:String,
    storage_incarnation:String,
}
impl FilePublicationIntent {
    /// Never-reused physical instance chosen by the original catalog reservation.
    pub fn instance(&self)->&str{&self.instance}
}
/// Locally complete and catalog-sealed file; this is not an AppendAck.
#[derive(Clone, Debug)]
pub struct SealedFile {
    descriptor:FileDescriptor,
    intent:FilePublicationIntent,
}
impl SealedFile {
    /// Physical content binding, independent of current selected root.
    pub fn descriptor(&self)->&FileDescriptor{&self.descriptor}
    /// Reference one logical member without copying physical bytes.
    pub fn batch(&self,index:u64)->Result<FileBatch,CanonicalError>{let batch=FileBatch{file:self.descriptor.clone(),index};batch.validate()?;Ok(batch)}
    pub(crate) fn original_intent(&self)->&FilePublicationIntent{&self.intent}
}

/// File handle accompanied by its exact read lifetime. Consumers that take only
/// the native file retain their ResultRead clone through every future lazy read.
#[derive(Debug)]
pub struct ProtectedFile {
    file:File,
    descriptor:FileDescriptor,
    _owner:Arc<dyn std::fmt::Debug+Send+Sync>,
    process:ProcessIdentity,
}
impl ProtectedFile {
    /// Exact complete physical descriptor.
    pub fn descriptor(&self)->&FileDescriptor{&self.descriptor}

}
impl Read for ProtectedFile{fn read(&mut self,bytes:&mut[u8])->std::io::Result<usize>{self.process.require_current().map_err(std::io::Error::other)?;self.file.read(bytes)}}
impl Seek for ProtectedFile{fn seek(&mut self,position:SeekFrom)->std::io::Result<u64>{self.process.require_current().map_err(std::io::Error::other)?;self.file.seek(position)}}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessIdentity {boot:String,pid_namespace:String,pid:u32,start:String,invocation:Option<String>}
impl ProcessIdentity {
    fn current()->Result<Self,CanonicalError>{
        let boot=std::fs::read_to_string("/proc/sys/kernel/random/boot_id").map_err(configuration)?.trim().to_owned();
        let process=std::fs::read_to_string("/proc/self/stat").map_err(configuration)?;
        let (_,tail)=process.rsplit_once(')').ok_or_else(||configuration("process stat identity malformed"))?;
        let start=tail.split_whitespace().nth(19).ok_or_else(||configuration("process start identity absent"))?.to_owned();
        let pid_namespace=std::fs::read_link("/proc/self/ns/pid").map_err(configuration)?.to_string_lossy().into_owned();
        Ok(Self{boot,pid_namespace,pid:std::process::id(),start,invocation:std::env::var("INVOCATION_ID").ok()})
    }
    fn require_current(&self)->Result<(),CanonicalError>{if *self!=Self::current()?{return Err(configuration("inherited file capability requires independent process registration"));}Ok(())}
}
#[derive(Debug)]
pub(crate) struct FileSession {
    pub(crate) incarnation:String,
    root:PathBuf,
    process:ProcessIdentity,
    owner:tokio::sync::Mutex<Option<String>>,
}
impl FileSession {
    pub(crate) fn from_state(state:&Path,database:&str)->Result<Self,CanonicalError>{
        #[derive(serde::Deserialize)] struct Placement{payload_incarnation:String}
        let config=std::fs::read(state.join("config.json")).map_err(configuration)?;
        let placement:Placement=serde_json::from_slice(&config).map_err(configuration)?;
        uuid::Uuid::parse_str(&placement.payload_incarnation).map_err(configuration)?;
        Ok(Self{root:state.join("payloads").join(&placement.payload_incarnation).join(database),incarnation:placement.payload_incarnation,process:ProcessIdentity::current()?,owner:tokio::sync::Mutex::new(None)})
    }
    pub(crate) fn require_current(&self)->Result<(),CanonicalError>{self.process.require_current()}
    fn path(&self,instance:&str)->Result<PathBuf,CanonicalError>{uuid::Uuid::parse_str(instance).map_err(configuration)?;Ok(self.root.join("instances").join(format!("{instance}.sealed")))}
}
fn configuration(error:impl std::fmt::Display)->CanonicalError{CanonicalError::Configuration(error.to_string())}
fn last_object(response:&mut surrealdb::IndexedResults)->Result<Object,CanonicalError>{let index=response.num_statements().saturating_sub(2);response.take::<Option<Object>>(index)?.ok_or(CanonicalError::IncompleteResponse)}
fn text(row:&mut Object,name:&'static str)->Result<String,CanonicalError>{Ok(codec::decode_string(codec::required(row,name)?)?)}

impl CanonicalStore {
    pub(crate) async fn registered_file_owner(&self)->Result<String,CanonicalError>{
        self.files.require_current()?;
        let mut owner=self.files.owner.lock().await;
        if let Some(owner)=owner.as_ref(){return Ok(owner.clone());}
        let key=uuid::Uuid::new_v4().to_string(); let process=&self.files.process; let mut row=Object::new();
        row.insert("key",key.clone());row.insert("issuing_store",self.issuing_store()?.to_owned());row.insert("storage_incarnation",self.files.incarnation.clone());row.insert("boot",process.boot.clone());row.insert("pid_namespace",process.pid_namespace.clone());row.insert("pid",codec::encode_uint(u64::from(process.pid))?);row.insert("start",process.start.clone());row.insert("drained",false);if let Some(invocation)=&process.invocation{row.insert("invocation",invocation.clone());}
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::register_owner($pse_rpc_expires_at,$row); COMMIT;").bind(("row",row.clone()))).await?;
        let mut saved=last_object(&mut response)?;saved.remove("id");if saved!=row{return Err(CanonicalError::OperationReused);} *owner=Some(key.clone());Ok(key)
    }
    /// Reserve original result publication before file creation. Repeating the
    /// same intent selects its original physical instance without new science.
    pub async fn reserve_result_file(&self,fence:&AttemptFence,operation:&str,request_digest:&str)->Result<FilePublicationIntent,CanonicalError>{
        self.reserve_file("result",fence.attempt(),operation,request_digest,Some(fence)).await
    }
    pub(crate) async fn reserve_artifact_file(&self,artifact:&str,operation:&str,request_digest:&str)->Result<FilePublicationIntent,CanonicalError>{self.reserve_file("artifact",artifact,operation,request_digest,None).await}
    async fn reserve_file(&self,kind:&str,selection:&str,operation:&str,request_digest:&str,fence:Option<&AttemptFence>)->Result<FilePublicationIntent,CanonicalError>{
        self.ensure_writes()?;let owner=self.registered_file_owner().await?;
        if operation.is_empty()||operation.len()>4096||request_digest.len()!=64{return Err(configuration("file original intent malformed"));}
        let mut row=Object::new();row.insert("key",uuid::Uuid::new_v4().to_string());row.insert("intent",operation.to_owned());row.insert("request_digest",request_digest.to_owned());row.insert("owner",owner);row.insert("owner_kind",kind.to_owned());row.insert("selection",selection.to_owned());row.insert("issuing_store",self.issuing_store()?.to_owned());row.insert("storage_incarnation",self.files.incarnation.clone());row.insert("digest",String::new());row.insert("payload_bytes",codec::encode_uint(0)?);row.insert("encoding",String::new());row.insert("batch_count",codec::encode_uint(0)?);row.insert("schema",String::new());row.insert("interpretation",crate::generated::surreal::INTERPRETATION);row.insert("state","pending");
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::reserve($pse_rpc_expires_at,$row,$run,$attempt,$generation); COMMIT;").bind(("row",row)).bind(("run",fence.map(|f|Value::String(f.run().to_owned())).unwrap_or(Value::None))).bind(("attempt",fence.map(|f|Value::String(f.attempt().to_owned())).unwrap_or(Value::None))).bind(("generation",fence.map(|f|codec::encode_uint(f.generation())).transpose()?.unwrap_or(Value::None)))).await?;
        let mut saved=last_object(&mut response)?;
        Ok(FilePublicationIntent{instance:text(&mut saved,"key")?,intent:text(&mut saved,"intent")?,request_digest:text(&mut saved,"request_digest")?,owner:text(&mut saved,"owner")?,storage_incarnation:text(&mut saved,"storage_incarnation")?})
    }
    /// Complete an opaque bounded native file after original intent reservation.
    /// Runtime validates native IPC/schema before this boundary; Operations
    /// verifies storage bytes, flush and no-replace publication before sealing.
    pub async fn seal_file(&self,intent:&FilePublicationIntent,bytes:&[u8],encoding:&str,batch_count:u64,schema:&str)->Result<SealedFile,CanonicalError>{
        self.files.require_current()?; self.ensure_writes()?;
        if intent.storage_incarnation!=self.files.incarnation || intent.owner!=self.registered_file_owner().await?{return Err(configuration("file intent actual owner differs"));}
        let descriptor=FileDescriptor{instance:intent.instance.clone(),digest:crate::canonical_execution::result_payload_digest(bytes),bytes:bytes.len() as u64,encoding:encoding.into(),batch_count,schema:schema.into(),interpretation:crate::generated::surreal::INTERPRETATION.into()};descriptor.validate()?;
        if encoding==OPAQUE_JSON {let _:serde_json::Value=serde_json::from_slice(bytes).map_err(configuration)?;}
        publish(&self.files.root,&descriptor,bytes)?;
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::seal($pse_rpc_expires_at,$key,$owner,$incarnation,$descriptor); COMMIT;").bind(("key",intent.instance.clone())).bind(("owner",intent.owner.clone())).bind(("incarnation",self.files.incarnation.clone())).bind(("descriptor",descriptor.encode()?))).await?;
        let saved=last_object(&mut response)?;
        if saved.get("digest")!=Some(&Value::String(descriptor.digest.clone())){return Err(CanonicalError::OperationReused);}
        Ok(SealedFile{descriptor,intent:intent.clone()})
    }
    pub(crate) fn open_physical_file(&self,descriptor:FileDescriptor,owner:Arc<dyn std::fmt::Debug+Send+Sync>)->Result<ProtectedFile,CanonicalError>{
        self.files.require_current()?; descriptor.validate()?;
        let mut file=File::open(self.files.path(&descriptor.instance)?).map_err(configuration)?;validate_file(&mut file,&descriptor)?;
        Ok(ProtectedFile{file,descriptor,_owner:owner,process:self.files.process.clone()})
    }
    /// Resume an exact admitted collection. Physical unlink follows irreversible
    /// catalog collecting state; a lost final reply cannot permit ABA reuse.
    pub async fn collect_file(&self,instance:&str)->Result<(),CanonicalError>{
        self.files.require_current()?;let path=self.files.path(instance)?;
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::collect($pse_rpc_expires_at,$key,$incarnation); COMMIT;").bind(("key",instance.to_owned())).bind(("incarnation",self.files.incarnation.clone()))).await?;
        let row=last_object(&mut response)?;if row.get("state")==Some(&Value::String("collected".into())){return Ok(());}
        match std::fs::remove_file(&path){Ok(())=>File::open(path.parent().ok_or_else(||configuration("file parent absent"))?).and_then(|f|f.sync_all()).map_err(configuration)?,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},Err(error)=>return Err(configuration(error))}
        bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::collected($pse_rpc_expires_at,$key,$incarnation); COMMIT;").bind(("key",instance.to_owned())).bind(("incarnation",self.files.incarnation.clone()))).await?;Ok(())
    }
}
fn validate_file(file:&mut File,descriptor:&FileDescriptor)->Result<(),CanonicalError>{
    if file.metadata().map_err(configuration)?.len()!=descriptor.bytes{return Err(configuration("sealed file length differs"));}
    let mut hash=FramedHasher::new(Frame::CanonicalPayloadV1);hash.str("pse.execution.result-payload.v1");hash.part_reader(descriptor.bytes,file,&||Ok(())).map_err(configuration)?;
    if hash.finish_hash().to_hex()!=descriptor.digest{return Err(configuration("sealed file integrity differs"));}
    file.seek(SeekFrom::Start(0)).map_err(configuration)?;Ok(())
}
fn publish(root:&Path,descriptor:&FileDescriptor,bytes:&[u8])->Result<(),CanonicalError>{
    descriptor.validate()?;
    let pending=root.join(".pending");let sealed=root.join("instances");
    std::fs::create_dir_all(&pending).map_err(configuration)?;std::fs::create_dir_all(&sealed).map_err(configuration)?;
    let final_path=sealed.join(format!("{}.sealed",descriptor.instance));
    if final_path.try_exists().map_err(configuration)?{let mut file=File::open(final_path).map_err(configuration)?;validate_file(&mut file,descriptor)?;let mut original=vec![0;bytes.len()];file.read_exact(&mut original).map_err(configuration)?;if original!=bytes{return Err(CanonicalError::OperationReused);}return Ok(());}
    let staging=pending.join(format!("{}.pending",descriptor.instance));
    let mut file=OpenOptions::new().write(true).read(true).create_new(true).open(&staging).map_err(configuration)?;
    file.write_all(bytes).map_err(configuration)?;file.sync_all().map_err(configuration)?;file.seek(SeekFrom::Start(0)).map_err(configuration)?;validate_file(&mut file,descriptor)?;
    // hard_link is an atomic no-replace publication on the supported local Linux filesystem.
    std::fs::hard_link(&staging,&final_path).map_err(configuration)?;
    File::open(&sealed).and_then(|f|f.sync_all()).map_err(configuration)?;
    std::fs::remove_file(staging).map_err(configuration)?;File::open(pending).and_then(|f|f.sync_all()).map_err(configuration)?;Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason="isolated file protocol controls assert exact failures")]
mod tests {
    use super::*;
    fn descriptor(bytes:&[u8])->FileDescriptor{FileDescriptor{instance:uuid::Uuid::new_v4().to_string(),digest:crate::canonical_execution::result_payload_digest(bytes),bytes:bytes.len() as u64,encoding:OPAQUE_JSON.into(),batch_count:0,schema:String::new(),interpretation:crate::generated::surreal::INTERPRETATION.into()}}
    #[test]
    fn sealed_publication_never_replaces_original_physical_bytes(){
        let root=std::env::temp_dir().join(format!("pse-sealed-control-{}",uuid::Uuid::new_v4()));
        let original=b"[1,2,3]";let first=descriptor(original);publish(&root,&first,original).unwrap();publish(&root,&first,original).unwrap();
        let mut changed=descriptor(b"[9,8,7]");changed.instance=first.instance.clone();assert!(publish(&root,&changed,b"[9,8,7]").is_err());
        assert_eq!(std::fs::read(root.join("instances").join(format!("{}.sealed",first.instance))).unwrap(),original);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn sealed_file_corruption_and_missing_native_coordinates_are_refused(){
        let root=std::env::temp_dir().join(format!("pse-integrity-control-{}",uuid::Uuid::new_v4()));let bytes=b"[1,2]";let file=descriptor(bytes);publish(&root,&file,bytes).unwrap();let path=root.join("instances").join(format!("{}.sealed",file.instance));std::fs::write(&path,b"[1,9]").unwrap();assert!(validate_file(&mut File::open(path).unwrap(),&file).is_err());
        assert!(FileBatch{file:file.clone(),index:1}.validate().is_err());let mut invalid=file;invalid.encoding=ARROW_IPC_FILE.into();assert!(invalid.validate().is_err());std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn inherited_actual_process_identity_cannot_use_file_capability(){let mut identity=ProcessIdentity::current().unwrap();identity.pid=identity.pid.wrapping_add(1);assert!(identity.require_current().is_err());}
    #[test]
    fn original_partial_staging_is_not_replaced_or_advertised_complete(){let root=std::env::temp_dir().join(format!("pse-partial-control-{}",uuid::Uuid::new_v4()));let bytes=b"[1,2,3]";let file=descriptor(bytes);std::fs::create_dir_all(root.join(".pending")).unwrap();let staging=root.join(".pending").join(format!("{}.pending",file.instance));std::fs::write(&staging,b"[1,").unwrap();assert!(publish(&root,&file,bytes).is_err());assert_eq!(std::fs::read(staging).unwrap(),b"[1,");assert!(!root.join("instances").join(format!("{}.sealed",file.instance)).exists());std::fs::remove_dir_all(root).unwrap();}
}
