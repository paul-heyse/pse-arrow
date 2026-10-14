// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Opaque sealed file effects. Scientific codecs and interpretation remain with
//! Runtime; only an attached catalog receipt acknowledges durable publication.

use crate::{canonical::{CanonicalError, CanonicalStore, bounded_query}, canonical_codec as codec, canonical_execution::AttemptFence};
use pse_ids::{Frame, FramedHasher};
use std::{fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, sync::Arc};
use surrealdb::types::{Object, Value};

#[cfg(all(test,feature="canonical-tests"))]
#[derive(Clone)]
struct EffectPause { kind:&'static str, ready:Arc<tokio::sync::Notify>, resume:Arc<tokio::sync::Notify> }
#[cfg(all(test,feature="canonical-tests"))]
tokio::task_local! { static FILE_EFFECT_PAUSE: EffectPause; }
#[cfg(all(test,feature="canonical-tests"))]
async fn effect_pause(kind:&str){if let Ok(pause)=FILE_EFFECT_PAUSE.try_with(Clone::clone){if pause.kind==kind{pause.ready.notify_one();pause.resume.notified().await;}}}

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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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
pub(crate) struct ProcessIdentity {boot:String,pid_namespace:String,pid:u32,start:String,invocation:Option<String>,unit:Option<String>,group:Option<String>,group_inode:Option<u64>}
impl ProcessIdentity {
    fn current()->Result<Self,CanonicalError>{
        let boot=std::fs::read_to_string("/proc/sys/kernel/random/boot_id").map_err(configuration)?.trim().to_owned();
        let process=std::fs::read_to_string("/proc/self/stat").map_err(configuration)?;
        let (_,tail)=process.rsplit_once(')').ok_or_else(||configuration("process stat identity malformed"))?;
        let start=tail.split_whitespace().nth(19).ok_or_else(||configuration("process start identity absent"))?.to_owned();
        let pid_namespace=std::fs::read_link("/proc/self/ns/pid").map_err(configuration)?.to_string_lossy().into_owned();
        let mut identity=Self{boot,pid_namespace,pid:std::process::id(),start,invocation:None,unit:None,group:None,group_inode:None};
        let cgroups=std::fs::read_to_string("/proc/self/cgroup").map_err(configuration)?;
        if let Some(group)=cgroups.lines().find_map(|line|line.strip_prefix("0::")) {
            if let Some(unit)=Path::new(group).components().filter_map(|part|part.as_os_str().to_str()).find(|part|(part.starts_with("pse-surreal-worker-")||part.starts_with("pse-native-"))&&(part.ends_with(".scope")||part.ends_with(".service"))) {
                let output=std::process::Command::new("systemctl").args(["--user","show",unit,"--property=InvocationID,ControlGroup","--no-pager"]).output().map_err(configuration)?;
                if !output.status.success(){return Err(configuration("managed file owner invocation unavailable"));}
                let fields=String::from_utf8(output.stdout).map_err(configuration)?;
                let invocation=fields.lines().find_map(|line|line.strip_prefix("InvocationID=")).ok_or_else(||configuration("managed file owner invocation absent"))?;
                let control=fields.lines().find_map(|line|line.strip_prefix("ControlGroup=")).ok_or_else(||configuration("managed file owner control group absent"))?;
                if invocation.len()!=32||!invocation.bytes().all(|b|b.is_ascii_hexdigit())||!(group==control||group.starts_with(&format!("{control}/"))){return Err(configuration("managed file owner invocation differs"));}
                use std::os::unix::fs::MetadataExt;
                let inode=std::fs::metadata(Path::new("/sys/fs/cgroup").join(control.trim_start_matches('/'))).map_err(configuration)?.ino();
                identity.invocation=Some(invocation.into());identity.unit=Some(unit.into());identity.group=Some(control.into());identity.group_inode=Some(inode);
            }
        }
        Ok(identity)
    }
    fn require_current(&self)->Result<(),CanonicalError>{if self.pid!=std::process::id(){return Err(configuration("inherited file capability requires independent process registration"));}Ok(())}
}
#[derive(Debug)]
pub(crate) struct FileSession {
    pub(crate) incarnation:String,
    root:PathBuf,
    process:ProcessIdentity,
    owner:tokio::sync::Mutex<Option<OwnerRegistration>>,
}
#[derive(Debug)]
struct OwnerRegistration {
    row:Object,
    acknowledged:bool,
}
impl FileSession {
    pub(crate) fn from_state(state:&Path,database:&str)->Result<Self,CanonicalError>{
        #[derive(serde::Deserialize)] struct Placement{payload_incarnation:String}
        let config=std::fs::read(state.join("config.json")).map_err(configuration)?;
        let placement:Placement=serde_json::from_slice(&config).map_err(configuration)?;
        uuid::Uuid::parse_str(&placement.payload_incarnation).map_err(configuration)?;
        Ok(Self{root:state.join("payloads").join(&placement.payload_incarnation).join(database),incarnation:placement.payload_incarnation,process:ProcessIdentity::current()?,owner:tokio::sync::Mutex::new(None)})
    }
    #[cfg(test)]
    pub(crate) fn wire_control()->Result<Self,CanonicalError>{Ok(Self{incarnation:uuid::Uuid::new_v4().to_string(),root:std::env::temp_dir().join(format!("pse-wire-control-no-file-effects-{}",uuid::Uuid::new_v4())),process:ProcessIdentity::current()?,owner:tokio::sync::Mutex::new(None)})}
    pub(crate) fn require_current(&self)->Result<(),CanonicalError>{self.process.require_current()}
    // Pending original rows also belong to physical teardown. An absent server
    // row must become a drained tombstone before any delayed registration lands.
    pub(crate) async fn registered_owner(&self)->Option<Object>{self.owner.lock().await.as_ref().map(|owner|owner.row.clone())}
    pub(crate) fn path(&self,instance:&str)->Result<PathBuf,CanonicalError>{uuid::Uuid::parse_str(instance).map_err(configuration)?;Ok(self.root.join("instances").join(format!("{instance}.sealed")))}
}
fn configuration(error:impl std::fmt::Display)->CanonicalError{CanonicalError::Configuration(error.to_string())}
fn last_object(response:&mut surrealdb::IndexedResults)->Result<Object,CanonicalError>{let index=response.num_statements().saturating_sub(2);response.take::<Option<Object>>(index)?.ok_or(CanonicalError::IncompleteResponse)}
fn text(row:&mut Object,name:&'static str)->Result<String,CanonicalError>{Ok(codec::decode_string(codec::required(row,name)?)?)}

impl CanonicalStore {
    pub(crate) async fn registered_file_owner(&self)->Result<String,CanonicalError>{
        self.files.require_current()?;
        let mut owner=self.files.owner.lock().await;
        if owner.is_none(){
            // Retain the exact original row before first dispatch. Cancellation
            // drops only this mutex guard; retry and physical drain keep the row.
            let process=ProcessIdentity::current()?;
            let mut row=Object::new();
            row.insert("key",uuid::Uuid::new_v4().to_string());row.insert("issuing_store",self.issuing_store()?.to_owned());row.insert("storage_incarnation",self.files.incarnation.clone());row.insert("boot",process.boot.clone());row.insert("pid_namespace",process.pid_namespace.clone());row.insert("pid",codec::encode_uint(u64::from(process.pid))?);row.insert("start",process.start.clone());row.insert("drained",false);if let Some(invocation)=&process.invocation{row.insert("invocation",invocation.clone());}
            if let Some(unit)=&process.unit{row.insert("unit",unit.clone());}if let Some(group)=&process.group{row.insert("group",group.clone());}if let Some(inode)=process.group_inode{row.insert("group_inode",codec::encode_uint(inode)?);}
            *owner=Some(OwnerRegistration{row,acknowledged:false});
        }
        let pending=owner.as_mut().ok_or(CanonicalError::IncompleteResponse)?;
        let key=codec::decode_string(pending.row.get("key").cloned().ok_or(CanonicalError::IncompleteResponse)?)?;
        if !pending.acknowledged {
            #[cfg(all(test,feature="canonical-tests"))]
            effect_pause("register").await;
            let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::register_owner($pse_rpc_expires_at,$row); COMMIT;").bind(("row",pending.row.clone()))).await?;
            let mut saved=last_object(&mut response)?;saved.remove("id");if saved!=pending.row{return Err(CanonicalError::OperationReused);}
            pending.acknowledged=true;
        }
        Ok(key)
    }
    /// Reserve original result publication before file creation. Repeating the
    /// same intent selects its original physical instance without new science.
    pub async fn reserve_result_file(&self,fence:&AttemptFence,operation:&str,request_digest:&str)->Result<FilePublicationIntent,CanonicalError>{
        self.reserve_file("result",fence.attempt(),operation,request_digest,Some(fence)).await
    }
    pub(crate) async fn reserve_artifact_file(&self,artifact:&str,operation:&str,request_digest:&str)->Result<FilePublicationIntent,CanonicalError>{self.reserve_file("artifact",artifact,operation,request_digest,None).await}
    async fn reserve_file(&self,kind:&str,selection:&str,operation:&str,request_digest:&str,fence:Option<&AttemptFence>)->Result<FilePublicationIntent,CanonicalError>{
        let activity=self.admit_activity()?;
        activity.scope(crate::canonical::transport::within_clock(crate::canonical::transport::original_deadline(crate::canonical::REQUEST_TIMEOUT),async {
        self.ensure_writes()?;let owner=self.registered_file_owner().await?;
        if operation.is_empty()||operation.len()>4096||request_digest.len()!=64{return Err(configuration("file original intent malformed"));}
        let mut row=Object::new();row.insert("key",uuid::Uuid::new_v4().to_string());row.insert("intent",operation.to_owned());row.insert("request_digest",request_digest.to_owned());row.insert("owner",owner.clone());row.insert("owner_kind",kind.to_owned());row.insert("selection",selection.to_owned());row.insert("issuing_store",self.issuing_store()?.to_owned());row.insert("storage_incarnation",self.files.incarnation.clone());row.insert("digest",String::new());row.insert("payload_bytes",codec::encode_uint(0)?);row.insert("encoding",String::new());row.insert("batch_count",codec::encode_uint(0)?);row.insert("schema",String::new());row.insert("interpretation",crate::generated::surreal::INTERPRETATION);row.insert("state","pending");
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::reserve($pse_rpc_expires_at,$row,$run,$attempt,$generation); COMMIT;").bind(("row",row)).bind(("run",fence.map(|f|Value::String(f.run().to_owned())).unwrap_or(Value::None))).bind(("attempt",fence.map(|f|Value::String(f.attempt().to_owned())).unwrap_or(Value::None))).bind(("generation",fence.map(|f|codec::encode_uint(f.generation())).transpose()?.unwrap_or(Value::None)))).await?;
        let mut saved=last_object(&mut response)?;
        let original=FilePublicationIntent{instance:text(&mut saved,"key")?,intent:text(&mut saved,"intent")?,request_digest:text(&mut saved,"request_digest")?,owner:text(&mut saved,"owner")?,storage_incarnation:text(&mut saved,"storage_incarnation")?};
        if original.owner!=owner {self.recover_file_intent(&original).await}else{Ok(original)}
        })).await
    }
    /// Recover only custody of the original file effect after authenticated old
    /// owner drain. Original writer/creation permission is never renewed.
    pub async fn recover_file_intent(&self,original:&FilePublicationIntent)->Result<FilePublicationIntent,CanonicalError>{
        let activity=self.admit_activity()?;
        activity.scope(crate::canonical::transport::within_clock(crate::canonical::transport::original_deadline(crate::canonical::REQUEST_TIMEOUT),async {
        self.files.require_current()?;
        let mut inspection=bounded_query(self.db.query("RETURN SELECT * FROM ONLY type::record('canonical_file_instances',$key);").bind(("key",original.instance.clone()))).await?;
        let mut inspected=inspection.take::<Option<Object>>(0)?.ok_or_else(||configuration("original physical intent unavailable"))?;
        if text(&mut inspected,"intent")?!=original.intent||text(&mut inspected,"request_digest")?!=original.request_digest||text(&mut inspected,"storage_incarnation")?!=self.files.incarnation{return Err(CanonicalError::OperationReused);}
        let owner=self.registered_file_owner().await?;
        let mut expected=Object::new();expected.insert("instance",original.instance.clone());expected.insert("intent",original.intent.clone());expected.insert("request_digest",original.request_digest.clone());expected.insert("owner",original.owner.clone());
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::recover_intent($pse_rpc_expires_at,$expected,$owner,$incarnation); COMMIT;").bind(("expected",expected)).bind(("owner",owner.clone())).bind(("incarnation",self.files.incarnation.clone()))).await?;
        let mut saved=last_object(&mut response)?;
        let recovered=FilePublicationIntent{instance:text(&mut saved,"key")?,intent:text(&mut saved,"intent")?,request_digest:text(&mut saved,"request_digest")?,owner,storage_incarnation:self.files.incarnation.clone()};
        let lock=lock_instance(&self.files.root,&recovered.instance).await?;
        self.physical_intent_open(&recovered).await?;
        settle_published_alias(&self.files.root,&recovered.instance,&lock)?;
        Ok(recovered)
        })).await
    }
    async fn physical_intent_open(&self,intent:&FilePublicationIntent)->Result<Object,CanonicalError>{
        let mut expected=Object::new();expected.insert("instance",intent.instance.clone());expected.insert("intent",intent.intent.clone());expected.insert("request_digest",intent.request_digest.clone());
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::intent_open($pse_rpc_expires_at,$expected,$owner,$incarnation); COMMIT;").bind(("expected",expected)).bind(("owner",intent.owner.clone())).bind(("incarnation",self.files.incarnation.clone()))).await?;last_object(&mut response)
    }
    /// Complete an opaque bounded native file after original intent reservation.
    /// Runtime validates native IPC/schema before this boundary; Operations
    /// verifies storage bytes, flush and no-replace publication before sealing.
    pub async fn seal_file(&self,intent:&FilePublicationIntent,bytes:&[u8],encoding:&str,batch_count:u64,schema:&str)->Result<SealedFile,CanonicalError>{
        let activity=self.admit_activity()?;
        activity.scope(crate::canonical::transport::within_clock(crate::canonical::transport::original_deadline(crate::canonical::REQUEST_TIMEOUT),async {
        self.files.require_current()?; self.ensure_writes()?;
        self.physical_intent_open(intent).await?;
        if intent.storage_incarnation!=self.files.incarnation || intent.owner!=self.registered_file_owner().await?{return Err(configuration("file intent actual owner differs"));}
        let descriptor=FileDescriptor{instance:intent.instance.clone(),digest:crate::canonical_execution::result_payload_digest(bytes),bytes:bytes.len() as u64,encoding:encoding.into(),batch_count,schema:schema.into(),interpretation:crate::generated::surreal::INTERPRETATION.into()};descriptor.validate()?;
        if encoding==OPAQUE_JSON {let _:serde_json::Value=serde_json::from_slice(bytes).map_err(configuration)?;}
        let lock=lock_instance(&self.files.root,&descriptor.instance).await?;
        let mut original=self.physical_intent_open(intent).await?;
        if original.get("state")!=Some(&Value::String("pending".into())) {
            if text(&mut original,"digest")?!=descriptor.digest||codec::decode_uint(codec::required(&mut original,"payload_bytes")?)?!=descriptor.bytes||text(&mut original,"encoding")?!=descriptor.encoding||codec::decode_uint(codec::required(&mut original,"batch_count")?)?!=descriptor.batch_count||text(&mut original,"schema")?!=descriptor.schema||text(&mut original,"interpretation")?!=descriptor.interpretation{return Err(CanonicalError::OperationReused);}
            if !self.files.path(&descriptor.instance)?.try_exists().map_err(configuration)?{return Err(configuration("original sealed file is missing; immutable physical paths cannot be recreated"));}
        }
        publish_locked(&self.files.root,&descriptor,bytes,&lock)?;
        #[cfg(all(test,feature="canonical-tests"))]
        effect_pause("seal").await;
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::seal($pse_rpc_expires_at,$key,$owner,$incarnation,$descriptor); COMMIT;").bind(("key",intent.instance.clone())).bind(("owner",intent.owner.clone())).bind(("incarnation",self.files.incarnation.clone())).bind(("descriptor",descriptor.encode()?))).await?;
        let saved=last_object(&mut response)?;
        if saved.get("digest")!=Some(&Value::String(descriptor.digest.clone())){return Err(CanonicalError::OperationReused);}
        Ok(SealedFile{descriptor,intent:intent.clone()})
        })).await
    }
    pub(crate) fn open_physical_file(&self,descriptor:FileDescriptor,owner:Arc<dyn std::fmt::Debug+Send+Sync>)->Result<ProtectedFile,CanonicalError>{
        self.files.require_current()?; descriptor.validate()?;
        let mut file=open_regular(&self.files.path(&descriptor.instance)?)?;require_single_link(&file)?;validate_file(&mut file,&descriptor)?;
        Ok(ProtectedFile{file,descriptor,_owner:owner,process:self.files.process.clone()})
    }
    /// Settle only this original unpublished storage intent after actual owner
    /// drain and original writer/creation closure. Unknown remains protected.
    pub async fn settle_file_intent(&self,original:&FilePublicationIntent)->Result<bool,CanonicalError>{
        let activity=self.admit_activity()?;
        activity.scope(crate::canonical::transport::within_clock(crate::canonical::transport::original_deadline(crate::canonical::REQUEST_TIMEOUT),async {
        self.files.require_current()?;
        let mut expected=Object::new();expected.insert("instance",original.instance.clone());expected.insert("intent",original.intent.clone());expected.insert("request_digest",original.request_digest.clone());expected.insert("owner",original.owner.clone());
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::settle_intent($pse_rpc_expires_at,$expected,$incarnation); COMMIT;").bind(("expected",expected)).bind(("incarnation",self.files.incarnation.clone()))).await?;
        let index=response.num_statements().saturating_sub(2);response.take::<Option<bool>>(index)?.ok_or(CanonicalError::IncompleteResponse)
        })).await
    }
    /// Resume an exact admitted collection. Physical unlink follows irreversible
    /// catalog collecting state; a lost final reply cannot permit ABA reuse.
    pub async fn collect_file(&self,instance:&str)->Result<(),CanonicalError>{
        let activity=self.admit_activity()?;
        activity.scope(crate::canonical::transport::within_clock(crate::canonical::transport::original_deadline(crate::canonical::REQUEST_TIMEOUT),async {
        self.files.require_current()?;let path=self.files.path(instance)?;
        let mut inspection=bounded_query(self.db.query("RETURN SELECT key,state,storage_incarnation FROM ONLY type::record('canonical_file_instances',$key);").bind(("key",instance.to_owned()))).await?;
        let mut inspected=inspection.take::<Option<Object>>(0)?.ok_or_else(||configuration("physical collection instance unavailable"))?;
        if text(&mut inspected,"key")?!=instance||text(&mut inspected,"storage_incarnation")?!=self.files.incarnation{return Err(CanonicalError::OperationReused);}
        let state=text(&mut inspected,"state")?;if state=="collected"{return Ok(());}if !["attached","abandoned","collecting"].contains(&state.as_str()){return Err(configuration("original file publication remains unresolved"));}
        self.registered_file_owner().await?;
        let _lock=lock_instance(&self.files.root,instance).await?;
        let mut response=bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::collect($pse_rpc_expires_at,$key,$incarnation); COMMIT;").bind(("key",instance.to_owned())).bind(("incarnation",self.files.incarnation.clone()))).await?;
        let row=last_object(&mut response)?;if row.get("state")==Some(&Value::String("collected".into())){return Ok(());}
        #[cfg(all(test,feature="canonical-tests"))]
        effect_pause("collect").await;
        match std::fs::remove_file(&path){Ok(())=>File::open(path.parent().ok_or_else(||configuration("file parent absent"))?).and_then(|f|f.sync_all()).map_err(configuration)?,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},Err(error)=>return Err(configuration(error))}
        let pending=self.files.root.join(".pending").join(format!("{instance}.pending"));
        match std::fs::remove_file(&pending){Ok(())=>File::open(pending.parent().ok_or_else(||configuration("staging parent absent"))?).and_then(|f|f.sync_all()).map_err(configuration)?,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},Err(error)=>return Err(configuration(error))}
        bounded_query(self.db.query("BEGIN; RETURN fn::pse_files_v1::collected($pse_rpc_expires_at,$key,$incarnation); COMMIT;").bind(("key",instance.to_owned())).bind(("incarnation",self.files.incarnation.clone()))).await?;Ok(())
        })).await
    }
}
/// Verify one offline inventory member without database access or effects.
/// The selected path must name a regular file directly; symlinks are refused.
pub fn verify_payload_file(path:&Path,descriptor:&FileDescriptor)->Result<(),CanonicalError>{descriptor.validate()?;let mut file=open_regular(path)?;require_single_link(&file)?;validate_file(&mut file,descriptor)}
fn require_single_link(file:&File)->Result<(),CanonicalError>{use std::os::unix::fs::MetadataExt;if file.metadata().map_err(configuration)?.nlink()!=1{return Err(configuration("sealed instance retains unknown physical alias"));}Ok(())}
fn open_regular(path:&Path)->Result<File,CanonicalError>{
    use std::os::unix::fs::OpenOptionsExt;
    let file=OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC).open(path).map_err(configuration)?;
    if !file.metadata().map_err(configuration)?.is_file(){return Err(configuration("sealed payload is not a regular file"));}Ok(file)
}
fn validate_file(file:&mut File,descriptor:&FileDescriptor)->Result<(),CanonicalError>{
    if file.metadata().map_err(configuration)?.len()!=descriptor.bytes{return Err(configuration("sealed file length differs"));}
    let mut hash=FramedHasher::new(Frame::CanonicalPayloadV1);hash.str("pse.execution.result-payload.v1");hash.part_reader(descriptor.bytes,file,&||Ok(())).map_err(configuration)?;
    if hash.finish_hash().to_hex()!=descriptor.digest{return Err(configuration("sealed file integrity differs"));}
    file.seek(SeekFrom::Start(0)).map_err(configuration)?;Ok(())
}
// This unique lock path survives collection. Deleting it would let a late
// waiter and a new opener lock different inodes for one physical instance.
struct InstanceLock { _file:File, instance:String }
fn instance_lock_file(root:&Path,instance:&str)->Result<File,CanonicalError>{
    uuid::Uuid::parse_str(instance).map_err(configuration)?;
    let directory=root.join(".locks");std::fs::create_dir_all(&directory).map_err(configuration)?;
    use std::os::unix::fs::OpenOptionsExt;
    let file=OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC).open(directory.join(format!("{instance}.lock"))).map_err(configuration)?;
    if !file.metadata().map_err(configuration)?.is_file(){return Err(configuration("physical instance lock is not a regular file"));}
    require_single_link(&file)?;file.sync_all().map_err(configuration)?;File::open(directory).and_then(|file|file.sync_all()).map_err(configuration)?;Ok(file)
}
async fn lock_instance(root:&Path,instance:&str)->Result<InstanceLock,CanonicalError>{
    let file=instance_lock_file(root,instance)?;
    let deadline=crate::canonical::transport::original_deadline(crate::canonical::REQUEST_TIMEOUT);
    loop {match file.try_lock(){Ok(())=>return Ok(InstanceLock{_file:file,instance:instance.into()}),Err(std::fs::TryLockError::WouldBlock)=>{if tokio::time::Instant::now()>=deadline{return Err(CanonicalError::Timeout);}tokio::time::sleep(std::time::Duration::from_millis(5)).await;},Err(std::fs::TryLockError::Error(error))=>return Err(configuration(error))}}
}
#[cfg(test)]
fn publish(root:&Path,descriptor:&FileDescriptor,bytes:&[u8])->Result<(),CanonicalError>{let file=instance_lock_file(root,&descriptor.instance)?;file.lock().map_err(configuration)?;let lock=InstanceLock{_file:file,instance:descriptor.instance.clone()};publish_locked(root,descriptor,bytes,&lock)}
fn settle_published_alias(root:&Path,instance:&str,lock:&InstanceLock)->Result<(),CanonicalError>{
    if lock.instance!=instance{return Err(configuration("physical instance exclusion differs"));}
    uuid::Uuid::parse_str(instance).map_err(configuration)?;
    let sealed=root.join("instances").join(format!("{instance}.sealed"));
    if !sealed.try_exists().map_err(configuration)?{return Ok(());}
    let final_file=open_regular(&sealed)?;
    let leftover=root.join(".pending").join(format!("{instance}.pending"));
    match std::fs::symlink_metadata(&leftover){Ok(metadata)=>{
        use std::os::unix::fs::MetadataExt;
        let original=final_file.metadata().map_err(configuration)?;
        if !metadata.is_file()||metadata.dev()!=original.dev()||metadata.ino()!=original.ino(){return Err(configuration("original staging inode differs from sealed publication"));}
        std::fs::remove_file(&leftover).map_err(configuration)?;File::open(leftover.parent().ok_or_else(||configuration("staging parent absent"))?).and_then(|file|file.sync_all()).map_err(configuration)?;
    },Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},Err(error)=>return Err(configuration(error))}
    use std::os::unix::fs::MetadataExt;
    if final_file.metadata().map_err(configuration)?.nlink()!=1{return Err(configuration("sealed instance retains unknown physical alias"));}Ok(())
}
fn publish_locked(root:&Path,descriptor:&FileDescriptor,bytes:&[u8],lock:&InstanceLock)->Result<(),CanonicalError>{
    descriptor.validate()?;
    if lock.instance!=descriptor.instance{return Err(configuration("physical instance exclusion differs"));}
    let pending=root.join(".pending");let sealed=root.join("instances");
    std::fs::create_dir_all(&pending).map_err(configuration)?;std::fs::create_dir_all(&sealed).map_err(configuration)?;
    let final_path=sealed.join(format!("{}.sealed",descriptor.instance));
    if final_path.try_exists().map_err(configuration)? {
        let mut file=open_regular(&final_path)?;validate_file(&mut file,descriptor)?;
        let mut scratch=[0u8;65536];let mut checked=0usize;
        while checked<bytes.len(){let width=(bytes.len()-checked).min(scratch.len());file.read_exact(&mut scratch[..width]).map_err(configuration)?;if scratch[..width]!=bytes[checked..checked+width]{return Err(CanonicalError::OperationReused);}checked+=width;}
        settle_published_alias(root,&descriptor.instance,lock)?;
        return Ok(());
    }
    let staging=pending.join(format!("{}.pending",descriptor.instance));
    use std::os::unix::fs::OpenOptionsExt;
    let mut file=match OpenOptions::new().write(true).read(true).create_new(true).mode(0o600).open(&staging){Ok(file)=>file,Err(error) if error.kind()==std::io::ErrorKind::AlreadyExists=>OpenOptions::new().read(true).write(true).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC).open(&staging).map_err(configuration)?,Err(error)=>return Err(configuration(error))};
    let written=file.metadata().map_err(configuration)?.len();
    if written>bytes.len() as u64{return Err(CanonicalError::OperationReused);}
    let mut buffer=[0u8;65536];let mut checked=0usize;
    while checked<written as usize {let width=(written as usize-checked).min(buffer.len());file.read_exact(&mut buffer[..width]).map_err(configuration)?;if buffer[..width]!=bytes[checked..checked+width]{return Err(CanonicalError::OperationReused);}checked+=width;}
    file.write_all(&bytes[written as usize..]).map_err(configuration)?;file.sync_all().map_err(configuration)?;file.seek(SeekFrom::Start(0)).map_err(configuration)?;validate_file(&mut file,descriptor)?;
    // hard_link is an atomic no-replace publication on the supported local Linux filesystem.
    std::fs::hard_link(&staging,&final_path).map_err(configuration)?;
    File::open(&sealed).and_then(|f|f.sync_all()).map_err(configuration)?;
    std::fs::remove_file(staging).map_err(configuration)?;File::open(pending).and_then(|f|f.sync_all()).map_err(configuration)?;require_single_link(&file)?;Ok(())
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
    fn original_partial_staging_is_not_replaced_or_advertised_complete(){let root=std::env::temp_dir().join(format!("pse-partial-control-{}",uuid::Uuid::new_v4()));let bytes=b"[1,2,3]";let file=descriptor(bytes);std::fs::create_dir_all(root.join(".pending")).unwrap();let staging=root.join(".pending").join(format!("{}.pending",file.instance));std::fs::write(&staging,b"[9,").unwrap();assert!(publish(&root,&file,bytes).is_err());assert_eq!(std::fs::read(staging).unwrap(),b"[9,");assert!(!root.join("instances").join(format!("{}.sealed",file.instance)).exists());std::fs::remove_dir_all(root).unwrap();}
    #[test]
    fn exact_original_staging_prefix_resumes_in_same_inode(){
        use std::os::unix::fs::MetadataExt;
        let root=std::env::temp_dir().join(format!("pse-resume-control-{}",uuid::Uuid::new_v4()));let bytes=b"[1,2,3]";let file=descriptor(bytes);
        std::fs::create_dir_all(root.join(".pending")).unwrap();let staging=root.join(".pending").join(format!("{}.pending",file.instance));std::fs::write(&staging,b"[1,").unwrap();let inode=std::fs::metadata(&staging).unwrap().ino();
        publish(&root,&file,bytes).unwrap();let sealed=root.join("instances").join(format!("{}.sealed",file.instance));assert_eq!(std::fs::metadata(&sealed).unwrap().ino(),inode);verify_payload_file(&sealed,&file).unwrap();std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn offline_verifier_refuses_symlink_even_when_target_bytes_match(){
        let root=std::env::temp_dir().join(format!("pse-nofollow-control-{}",uuid::Uuid::new_v4()));let bytes=b"[1,2]";let file=descriptor(bytes);publish(&root,&file,bytes).unwrap();let path=root.join("instances").join(format!("{}.sealed",file.instance));let link=root.join("selected-link");std::os::unix::fs::symlink(path, &link).unwrap();assert!(verify_payload_file(&link,&file).is_err());std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[allow(unsafe_code, reason="actual fork control checks inherited handle refusal before IO and exits child without Rust teardown")]
    fn actual_fork_child_cannot_read_inherited_protected_file(){
        let root=std::env::temp_dir().join(format!("pse-fork-control-{}",uuid::Uuid::new_v4()));let bytes=b"[1,2]";let descriptor=descriptor(bytes);publish(&root,&descriptor,bytes).unwrap();
        let mut file=ProtectedFile{file:open_regular(&root.join("instances").join(format!("{}.sealed",descriptor.instance))).unwrap(),descriptor,_owner:Arc::new(()),process:ProcessIdentity::current().unwrap()};
        // SAFETY: no application locks or asynchronous tasks cross this test's
        // fork; the child checks cached PID before IO and uses _exit directly.
        let child=unsafe{libc::fork()};assert!(child>=0);
        if child==0 {let mut byte=[0u8;1];let refused=file.read(&mut byte).is_err();unsafe{libc::_exit(if refused{0}else{1})};}
        let mut status=0;assert_eq!(unsafe{libc::waitpid(child,&mut status,0)},child);assert_eq!(status,0);drop(file);std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn link_before_unlink_crash_settles_only_exact_original_alias(){
        use std::os::unix::fs::MetadataExt;
        let root=std::env::temp_dir().join(format!("pse-link-crash-control-{}",uuid::Uuid::new_v4()));let bytes=b"[1,2]";let file=descriptor(bytes);
        let pending=root.join(".pending");let sealed=root.join("instances");std::fs::create_dir_all(&pending).unwrap();std::fs::create_dir_all(&sealed).unwrap();
        let source=pending.join(format!("{}.pending",file.instance));let target=sealed.join(format!("{}.sealed",file.instance));std::fs::write(&source,bytes).unwrap();File::open(&source).unwrap().sync_all().unwrap();std::fs::hard_link(&source,&target).unwrap();assert_eq!(std::fs::metadata(&target).unwrap().nlink(),2);assert!(verify_payload_file(&target,&file).is_err());
        publish(&root,&file,bytes).unwrap();assert!(!source.exists());assert_eq!(std::fs::metadata(&target).unwrap().nlink(),1);verify_payload_file(&target,&file).unwrap();
        std::fs::write(&source,bytes).unwrap();let locked=instance_lock_file(&root,&file.instance).unwrap();locked.lock().unwrap();let lock=InstanceLock{_file:locked,instance:file.instance.clone()};assert!(settle_published_alias(&root,&file.instance,&lock).is_err());drop(lock);assert!(source.exists());std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn concurrent_original_file_replays_serialize_equal_and_different_content(){
        for equal in [true,false] {
            let root=std::env::temp_dir().join(format!("pse-concurrent-file-{}",uuid::Uuid::new_v4()));
            let first=Arc::new(vec![b'1';2*1024*1024]);let second=Arc::new(if equal{first.as_ref().clone()}else{vec![b'9';first.len()]});
            let mut original=descriptor(first.as_slice());original.encoding=OPAQUE_BYTES.into();let mut changed=original.clone();changed.digest=crate::canonical_execution::result_payload_digest(second.as_slice());
            let start=Arc::new(std::sync::Barrier::new(3));let left_root=root.clone();let left_start=start.clone();let left_descriptor=original.clone();let left_bytes=first.clone();
            let left=std::thread::spawn(move||{left_start.wait();publish(&left_root,&left_descriptor,left_bytes.as_slice())});
            let right_root=root.clone();let right_start=start.clone();let right_descriptor=changed.clone();let right_bytes=second.clone();let right=std::thread::spawn(move||{right_start.wait();publish(&right_root,&right_descriptor,right_bytes.as_slice())});
            start.wait();let left=left.join().unwrap();let right=right.join().unwrap();
            let path=root.join("instances").join(format!("{}.sealed",original.instance));
            if equal{assert!(left.is_ok()&&right.is_ok());verify_payload_file(&path,&original).unwrap();}else{assert_ne!(left.is_ok(),right.is_ok());let expected=if left.is_ok(){&original}else{&changed};verify_payload_file(&path,expected).unwrap();}
            let observed=std::fs::read(path).unwrap();assert!(observed==*first||observed==*second);assert!(root.join(".locks").join(format!("{}.lock",original.instance)).exists());std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn second_creator_cannot_open_writable_staging_during_original_seal(){
        let root=std::env::temp_dir().join(format!("pse-exclusive-stage-{}",uuid::Uuid::new_v4()));let bytes=b"[1,2,3]";let original=descriptor(bytes);let file=instance_lock_file(&root,&original.instance).unwrap();file.lock().unwrap();let lock=InstanceLock{_file:file,instance:original.instance.clone()};
        std::fs::create_dir_all(root.join(".pending")).unwrap();std::fs::write(root.join(".pending").join(format!("{}.pending",original.instance)),[]).unwrap();
        let competing=instance_lock_file(&root,&original.instance).unwrap();assert!(matches!(competing.try_lock(),Err(std::fs::TryLockError::WouldBlock)));
        let mut other=original.clone();other.digest=crate::canonical_execution::result_payload_digest(b"[9,8,7]");let worker_root=root.clone();let worker=std::thread::spawn(move||publish(&worker_root,&other,b"[9,8,7]"));
        publish_locked(&root,&original,bytes,&lock).unwrap();drop(lock);assert!(worker.join().unwrap().is_err());verify_payload_file(&root.join("instances").join(format!("{}.sealed",original.instance)),&original).unwrap();std::fs::remove_dir_all(root).unwrap();
    }

}

impl CanonicalStore {
    pub(crate) async fn admit_manifest_ticket(&self,kind:&str,selection:&str,result:Option<(&str,&str,&str)>,protection:&str)->Result<(),CanonicalError>{
        let owner=self.registered_file_owner().await?;let mut row=Object::new();row.insert("key",protection.to_owned());row.insert("kind",kind.to_owned());row.insert("selection",selection.to_owned());row.insert("owner",owner);row.insert("storage_incarnation",self.files.incarnation.clone());row.insert("released",false);
        if let Some((run,attempt,manifest))=result{row.insert("run",run.to_owned());row.insert("attempt",attempt.to_owned());row.insert("manifest",manifest.to_owned());}
        let mut response=self.protected_query(&format!("file-ticket:{protection}"),crate::canonical::QueryEffect::Mutation,"canonical_files::admit_ticket",||Ok(self.db.query("BEGIN; RETURN fn::pse_files_v1::ticket($pse_rpc_expires_at,$row); COMMIT;").bind(("row",row.clone())))).await?;
        let mut saved=last_object(&mut response)?;saved.remove("id");if saved!=row{return Err(CanonicalError::OperationReused);}Ok(())
    }
    pub(crate) async fn file_descriptor(&self,batch:&pse_model::generated::runtime::canonical_result_batches::Row)->Result<FileDescriptor,CanonicalError>{
        let descriptor=self.descriptor_for_instance(&batch.file_instance).await?;
        if descriptor.digest!=batch.file_digest||descriptor.bytes!=batch.payload_bytes||descriptor.schema!=batch.schema||descriptor.encoding!=batch.encoding{return Err(configuration("logical file descriptor differs from sealed instance"));}
        FileBatch{file:descriptor.clone(),index:batch.file_batch_index}.validate()?;Ok(descriptor)
    }
    pub(crate) async fn descriptor_for_instance(&self,instance:&str)->Result<FileDescriptor,CanonicalError>{
        self.files.require_current()?;
        let mut response=bounded_query(self.db.query("BEGIN; LET $file=SELECT * FROM ONLY type::record('canonical_file_instances',$key) FOR UPDATE; IF $file=NONE OR $file.storage_incarnation!=$incarnation OR $file.state!='attached' { THROW 'admitted file physical instance unavailable'; }; RETURN $file; COMMIT;").bind(("key",instance.to_owned())).bind(("incarnation",self.files.incarnation.clone()))).await?;
        let mut row=last_object(&mut response)?;
        let descriptor=FileDescriptor{instance:text(&mut row,"key")?,digest:text(&mut row,"digest")?,bytes:codec::decode_uint(codec::required(&mut row,"payload_bytes")?)?,encoding:text(&mut row,"encoding")?,batch_count:codec::decode_uint(codec::required(&mut row,"batch_count")?)?,schema:text(&mut row,"schema")?,interpretation:text(&mut row,"interpretation")?};descriptor.validate()?;Ok(descriptor)
    }
}

#[cfg(all(test,feature="canonical-tests"))]
#[allow(clippy::unwrap_used,clippy::expect_used,unsafe_code,reason="UUID-isolated actual catalog controls supply their synthetic original scientific outcomes")]
mod catalog_controls {
    use super::*;
    use crate::{canonical::{CanonicalOptions,Revision},canonical_execution::{RunRequest,ResultPayload,SeedEligibility,TerminalClass}};
    async fn fixture()->(CanonicalStore,Revision){let state=std::env::var("PSE_SURREAL_STATE").expect("explicit file lifecycle fixture state required");let mut options=CanonicalOptions::from_state(Path::new(&state)).unwrap();options.database=format!("canonical_test_file_{}",uuid::Uuid::new_v4().simple());let store=crate::testing::canonical_fixture_with_options(&options,true).unwrap();let revision=store.edit("problem",None,"source",&[]).await.unwrap();(store,revision)}
    async fn run(store:&CanonicalStore,revision:Revision)->AttemptFence{store.begin_run(&RunRequest{key:"file-run".into(),revision,sources:vec![],request:vec![1],source_selection:vec![2],attestation:vec![3]}).await.unwrap();store.claim_run("file-run","claim","worker",std::time::Duration::from_secs(60)).await.unwrap()}
    #[tokio::test]
    async fn canonical_files_strong_ticket_survives_expired_grant_and_shared_file_retirement_both_orders(){
        let (store,revision)=fixture().await;let fence=run(&store,revision).await;
        let bytes=b"[1,2]";let intent=store.reserve_result_file(&fence,"original-file-intent",&crate::canonical_execution::result_payload_digest(bytes)).await.unwrap();let file=store.seal_file(&intent,bytes,OPAQUE_JSON,0,"").await.unwrap();
        for ordinal in 0..2{store.append_result_batch(&fence,&format!("append-{ordinal}"),"values",ordinal,ResultPayload::from_sealed(file.clone(),0).unwrap(),1).await.unwrap();}
        let closed=store.close_result_ingestion(&fence,"close").await.unwrap();let manifest=store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this fixture owns the exact synthetic closed observations.
        unsafe{store.seal_attempt(&manifest,"terminal",TerminalClass::Succeeded,&[1],SeedEligibility::Unavailable).await}.unwrap();
        let read=store.read_results(fence.run(),fence.attempt()).await.unwrap();let set=read.sets()[0].key.clone();
        bounded_query(store.db.query("UPDATE ONLY type::record('canonical_protections',$key) SET expires_at=0;").bind(("key",read.protected_selection().key().to_owned()))).await.unwrap();
        let descriptors=store.result_file_batch_descriptors(&read,&set,&[0,1]).await.unwrap();assert_eq!(descriptors.len(),2);assert_eq!(descriptors[0].file_instance,descriptors[1].file_instance);
        let mut unread=store.open_result_file(&read,&descriptors[1]).await.unwrap();drop(read);
        let mut observed=Vec::new();unread.read_to_end(&mut observed).unwrap();assert_eq!(observed,bytes);
        assert!(store.forget_run_results(fence.run()).await.is_err());assert!(store.collect_file(file.descriptor().instance.as_str()).await.is_err());
        drop(unread);store.result_read_drain.drain().await.unwrap();store.forget_run_results(fence.run()).await.unwrap();
        loop{if store.reclaim_result_page(fence.run()).await.unwrap().complete{break;}}
        store.collect_file(&file.descriptor().instance).await.unwrap();assert!(!store.files.path(&file.descriptor().instance).unwrap().exists());
        assert!(store.seal_file(&intent,bytes,OPAQUE_JSON,0,"").await.is_err());assert!(store.recover_file_intent(&intent).await.is_err());
        assert!(!store.files.path(intent.instance()).unwrap().exists());assert!(!store.files.root.join(".pending").join(format!("{}.pending",intent.instance())).exists());
        let peer=CanonicalStore::connect(&{let mut options=CanonicalOptions::from_state(store.deployment_state()).unwrap();options.database=store.database().to_owned();options}).await.unwrap();peer.open().await.unwrap();assert!(peer.read_results(fence.run(),fence.attempt()).await.unwrap_err().to_string().contains("retired"));
        // Admission already acquired its source selection before the retired
        // result guard refused it. Join that exact automatic release queue.
        peer.result_read_drain.drain().await.unwrap();peer.disconnect().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_pending_owner_cancel_before_dispatch_disconnect_fences_late_registration(){
        let (store,revision)=fixture().await;let fence=run(&store,revision).await;
        let pause=EffectPause{kind:"register",ready:Arc::new(tokio::sync::Notify::new()),resume:Arc::new(tokio::sync::Notify::new())};let caller=store.clone();let gate=pause.clone();
        let pending=tokio::spawn(FILE_EFFECT_PAUSE.scope(gate,async move{caller.reserve_result_file(&fence,"cancel-before-owner-dispatch",&crate::canonical_execution::result_payload_digest(b"[1]")).await}));
        tokio::time::timeout(std::time::Duration::from_secs(30),pause.ready.notified()).await.unwrap();pending.abort();assert!(pending.await.unwrap_err().is_cancelled());
        let original=store.files.registered_owner().await.unwrap();let key=codec::decode_string(original.get("key").cloned().unwrap()).unwrap();assert!(!store.files.owner.lock().await.as_ref().unwrap().acknowledged);
        let mut options=CanonicalOptions::from_state(store.deployment_state()).unwrap();options.database=store.database().to_owned();let peer=CanonicalStore::connect(&options).await.unwrap();peer.open().await.unwrap();
        let mut before=bounded_query(peer.db.query("RETURN SELECT * FROM ONLY type::record('canonical_file_owners',$key);").bind(("key",key.clone()))).await.unwrap();assert!(before.take::<Option<Object>>(0).unwrap().is_none());
        bounded_query(peer.db.query("BEGIN; RETURN fn::pse_files_v1::freeze($pse_rpc_expires_at,$incarnation); COMMIT;").bind(("incarnation",store.files.incarnation.clone()))).await.unwrap();
        store.disconnect().await.unwrap();
        let mut after=bounded_query(peer.db.query("RETURN SELECT * FROM ONLY type::record('canonical_file_owners',$key); RETURN SELECT key FROM canonical_file_owners; RETURN SELECT key FROM canonical_file_instances;").bind(("key",key.clone()))).await.unwrap();let mut receipt=after.take::<Option<Object>>(0).unwrap().unwrap();assert_eq!(receipt.get("drained"),Some(&Value::Bool(true)));receipt.remove("id");receipt.insert("drained",false);assert_eq!(receipt,original);assert_eq!(after.take::<Vec<Object>>(1).unwrap().len(),1);assert!(after.take::<Vec<Object>>(2).unwrap().is_empty());
        bounded_query(peer.db.query("BEGIN; RETURN fn::pse_files_v1::thaw($pse_rpc_expires_at,$incarnation); COMMIT;").bind(("incarnation",store.files.incarnation.clone()))).await.unwrap();
        assert!(bounded_query(peer.db.query("BEGIN; RETURN fn::pse_files_v1::register_owner($pse_rpc_expires_at,$row); COMMIT;").bind(("row",original))).await.is_err());
        peer.disconnect().await.unwrap();store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn canonical_files_unknown_collection_and_forged_intent_leave_no_physical_residue(){
        let (store,_revision)=fixture().await;assert!(!store.files.root.exists());
        let original=FilePublicationIntent{instance:uuid::Uuid::new_v4().to_string(),intent:"forged-original".into(),request_digest:crate::canonical_execution::result_payload_digest(b"[1]"),owner:uuid::Uuid::new_v4().to_string(),storage_incarnation:store.files.incarnation.clone()};
        assert!(store.collect_file(original.instance()).await.is_err());
        assert!(store.seal_file(&original,b"[1]",OPAQUE_JSON,0,"").await.is_err());
        assert!(store.recover_file_intent(&original).await.is_err());
        assert!(!store.files.root.exists());assert!(store.files.owner.lock().await.is_none());
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_files_unresolved_intent_blocks_collection_and_original_owner_cannot_recover_live(){
        let (store,revision)=fixture().await;let fence=run(&store,revision).await;let bytes=b"[1]";
        let intent=store.reserve_result_file(&fence,"unresolved-file",&crate::canonical_execution::result_payload_digest(bytes)).await.unwrap();store.seal_file(&intent,bytes,OPAQUE_JSON,0,"").await.unwrap();assert!(store.collect_file(intent.instance()).await.is_err());
        let other=CanonicalStore::connect(&{let mut options=CanonicalOptions::from_state(store.deployment_state()).unwrap();options.database=store.database().to_owned();options}).await.unwrap();other.open().await.unwrap();assert!(other.recover_file_intent(&intent).await.is_err());other.disconnect().await.unwrap();
        assert!(store.settle_file_intent(&intent).await.is_err());store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_file_effects_hold_physical_drain_across_filesystem_and_catalog_completion(){
        let (store,revision)=fixture().await;let fence=run(&store,revision).await;let bytes=b"[1,2,3]";
        let intent=store.reserve_result_file(&fence,"paused-original",&crate::canonical_execution::result_payload_digest(bytes)).await.unwrap();
        let pause=EffectPause{kind:"seal",ready:Arc::new(tokio::sync::Notify::new()),resume:Arc::new(tokio::sync::Notify::new())};
        let publisher=store.clone();let submitted=intent.clone();let gate=pause.clone();
        let pending=tokio::spawn(FILE_EFFECT_PAUSE.scope(gate,async move{publisher.seal_file(&submitted,bytes,OPAQUE_JSON,0,"").await}));
        tokio::time::timeout(std::time::Duration::from_secs(30),pause.ready.notified()).await.unwrap();
        assert!(store.files.path(intent.instance()).unwrap().exists());
        assert!(matches!(store.disconnect().await,Err(CanonicalError::ConnectionBusy{..})));
        store.close();assert!(matches!(store.seal_file(&intent,bytes,OPAQUE_JSON,0,"").await,Err(CanonicalError::CallerClosed)));
        pause.resume.notify_one();let sealed=pending.await.unwrap().unwrap();verify_payload_file(&store.files.path(intent.instance()).unwrap(),sealed.descriptor()).unwrap();store.disconnect().await.unwrap();
        let mut options=CanonicalOptions::from_state(store.deployment_state()).unwrap();options.database=store.database().to_owned();let peer=CanonicalStore::connect(&options).await.unwrap();peer.open().await.unwrap();
        peer.close_result_ingestion(&fence,"close-original").await.unwrap();assert!(peer.settle_file_intent(&intent).await.unwrap());
        let pause=EffectPause{kind:"collect",ready:Arc::new(tokio::sync::Notify::new()),resume:Arc::new(tokio::sync::Notify::new())};let collector=peer.clone();let instance=intent.instance().to_owned();let gate=pause.clone();
        let pending=tokio::spawn(FILE_EFFECT_PAUSE.scope(gate,async move{collector.collect_file(&instance).await}));
        tokio::time::timeout(std::time::Duration::from_secs(30),pause.ready.notified()).await.unwrap();assert!(peer.files.path(intent.instance()).unwrap().exists());assert!(matches!(peer.disconnect().await,Err(CanonicalError::ConnectionBusy{..})));
        peer.close();assert!(matches!(peer.collect_file(intent.instance()).await,Err(CanonicalError::CallerClosed)));pause.resume.notify_one();pending.await.unwrap().unwrap();assert!(!peer.files.path(intent.instance()).unwrap().exists());peer.disconnect().await.unwrap();store.remove_isolated_fixture().await.unwrap();
    }

}
