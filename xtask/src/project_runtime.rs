// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One typed operator adapter to existing source, admission and preparation owners.
use anyhow::{Context, Result, ensure};
use clap::Args;
use pse_operations::{canonical::CanonicalStore, canonical_projects::{Project,Branch,Draft,ProjectPublication,DraftVersion}, canonical_contexts::{ContextCut,ContextSelection}};
use pse_runtime::{SharedRuntime,CancelSource,workflow::{Runtime,CanonicalDeployment,OuterAttestation,ModelingPackage,PhysicalContext,ProjectDraftDocuments,ProjectDraftSaveIntent,ProjectMergeIntent,ProjectMergeResolution,PreparationDiscoveryRequest}};
use serde::{Deserialize,Serialize};
use std::{collections::BTreeMap,path::PathBuf,sync::Arc,time::Instant,io::Read};

#[derive(Debug,Args)]
pub(crate) struct RuntimeArgs {
    /// Versioned typed action JSON. Scientific selection uses exact original revisions.
    #[arg(long)] request:PathBuf,
    /// Accounted runtime capacity, independent of the environment's OS build policy.
    #[arg(long,default_value_t=16usize*1024*1024*1024)] memory_limit_bytes:usize,
    #[arg(long,default_value_t=16)] threads:usize,
    #[arg(long)] spill_dir:PathBuf,
    #[arg(long,default_value_t=16u64*1024*1024*1024)] max_spill_bytes:u64,
    #[arg(long,default_value_t=8192)] batch_size:usize,
}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    version:pse_model::document::Version<1>,
    /// Required only for scientific source loading/admission; ordinary authored documents.
    physical:Option<ProjectDraftDocuments>,
    action:Action,
}
#[derive(Debug,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
enum Action {
    Import { documents:ProjectDraftDocuments },
    Load { revision:String },
    Fork { project:Project,branch:String,problem:String,operation:String,source:String },
    DraftRead { key:String },
    DraftIssue { operation:String,project:Project,branch:Branch,key:String,base:String,expected:Option<u64>,input:PathBuf },
    DraftSave { intent:ProjectDraftSaveIntent,input:PathBuf },
    DraftAdmit { operation:String,key:String },
    DraftAdmitVersion { version:DraftVersion },
    PublicationIssue { project:Project,branch:Branch,expected:String,draft:Option<Draft>,operation:String },
    DraftPublish { intent:ProjectPublication },
    Compare { left:String,right:String,#[serde(default)] source_only:bool },
    SourceDetail { revision:String,logical:Vec<String> },
    MergePreview { base:String,target:String,proposal:String,resolutions:Vec<ProjectMergeResolution> },
    MergeIssue { publication:ProjectPublication,base:String,target:String,proposal:String,resolutions:Vec<ProjectMergeResolution> },
    MergeDraftIssue { publication:ProjectPublication,base:String,target:String,version:DraftVersion,resolutions:Vec<ProjectMergeResolution> },
    MergePublish { intent:ProjectMergeIntent },
    UndoIssue { publication:ProjectPublication,target:String,restore:String },
    ContextRecord { operation:String,project:Project,scope:String,revision:String,effective_from_us:i64,effective_to_us:Option<i64>,corrects:Option<String>,deleted:bool },
    ContextCut { project:String },
    ContextSelect { cut:ContextCut,scope:String,effective_at_us:i64 },
    ContextLoad { selection:ContextSelection },
    PreparationDiscovery { revision:String,request:PreparationDiscoveryRequest },
}
macro_rules! field {
    (memory_limit_bytes,$args:ident)=>{$args.memory_limit_bytes};
    (threads,$args:ident)=>{$args.threads};
    (spill_dir,$args:ident)=>{$args.spill_dir.clone()};
    (max_spill_bytes,$args:ident)=>{$args.max_spill_bytes};
    (batch_size,$args:ident)=>{$args.batch_size};
    ($name:ident,$args:ident)=>{None};
}
fn settings(args:&RuntimeArgs)->Result<pse_runtime::ResourceBudget>{
    macro_rules! input { ($budget:ident; $($name:ident:$ty:ty,$hint:literal $(,$default:expr)? =>$out:ty,$read:expr;)*)=>{
        pse_runtime::settings::EngineSettingsInput{$($name:field!($name,args),)*}
    }; }
    Ok(pse_runtime::engine_settings_fields!(input,unused).resolve(None)?)
}
fn value<T:Serialize>(value:T)->Result<serde_json::Value>{Ok(serde_json::to_value(value)?)}
fn package(value:&ModelingPackage)->Result<serde_json::Value>{
    #[derive(Serialize)] struct Selected<'a>{revision_id:&'a str,canonical_revision:Option<&'a str>,canonical_problem:Option<&'a str>,context:Option<&'a ContextSelection>}
    self::value(Selected{revision_id:value.revision_id(),canonical_revision:value.canonical_revision().map(|r|r.key.as_str()),canonical_problem:value.canonical_revision().map(|r|r.problem.as_str()),context:value.project_context()})
}
fn documents(runtime:&Runtime,shared:&SharedRuntime,input:&ProjectDraftDocuments,cancel:&CancelSource)->Result<pse_admission::authoring_driver::document::OwnedDocumentSet>{
    let registry=pse_schema::shared_registry()?;let pool=shared.pool();let token=cancel.token();let validation=runtime.validation_context();
    let bundles=input.packages.iter().map(|sources|pse_admission::authoring_driver::document::load_package_documents_owned(sources,&registry,Default::default(),&pool,&token,&validation)).collect::<Result<Vec<_>,_>>()?;
    Ok(pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(bundles,&pool,&token)?)
}
async fn selected(runtime:&Runtime,store:&CanonicalStore,key:&str,physical:&Option<PhysicalContext>)->Result<ModelingPackage>{
    let revision=store.revision(key).await?.context("exact source revision is unavailable")?;
    Ok(runtime.modeling_revision(revision,physical.clone().context("physical documents required for scientific action")?,BTreeMap::new()).await?)
}
fn require_physical(physical:&Option<PhysicalContext>)->Result<PhysicalContext>{physical.clone().context("physical documents required for scientific action")}
fn bytes(path:&std::path::Path,limit:usize)->Result<Vec<u8>>{
    let file=std::fs::File::open(path)?;ensure!(file.metadata()?.is_file(),"regular body file required");
    let limit=u64::try_from(limit)?.checked_add(1).context("body transport allowance overflow")?;
    let mut content=Vec::new();file.take(limit).read_to_end(&mut content)?;ensure!(u64::try_from(content.len())?<limit,"body exceeds runtime transport allowance");Ok(content)
}
pub(crate) async fn run(store:&CanonicalStore,args:RuntimeArgs,deadline:Instant)->Result<serde_json::Value>{
    let request:Request=super::project_tools::read_json(&args.request)?;
    let _version=request.version;
    let budget=settings(&args)?;
    let shared=SharedRuntime::build(budget)?;
    let registry=pse_schema::shared_registry()?;
    let sessions=Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
    let mut artifact=std::fs::File::open(std::env::current_exe()?)?;let mut digest=pse_ids::EncodingHasher::new();std::io::copy(&mut artifact,&mut digest)?;
    let runtime=Runtime::from_shared(shared.clone(),sessions.bind_model(registry)?,CanonicalDeployment::new(store.clone(),OuterAttestation{source:None,build:digest.finish().content_hash()},None))?;
    let cancel=CancelSource::new().with_deadline(Some(deadline));
    let signal_cancel=cancel.clone();let signal=tokio::spawn(async move{if tokio::signal::ctrl_c().await.is_ok(){signal_cancel.cancel();}});
    let result:Result<serde_json::Value>=async {
        let physical=if let Some(input)=request.physical {Some(runtime.physical_from_documents(&documents(&runtime,&shared,&input,&cancel)?,&cancel.token()).await?)}else{None};
        match request.action {
            Action::Import{documents:input}=>package(&runtime.modeling_from_documents(&documents(&runtime,&shared,&input,&cancel)?,require_physical(&physical)?,&cancel).await?),
            Action::Load{revision}=>package(&selected(&runtime,store,&revision,&physical).await?),
            Action::Fork{project,branch,problem,operation,source}=>{let source=selected(&runtime,store,&source,&physical).await?;value(runtime.project_fork(&project,&branch,&problem,&operation,&source).await?.key)},
            Action::DraftRead{key}=>value(runtime.project_draft_bytes(&key,&cancel).await?.to_vec()),
            Action::DraftIssue{operation,project,branch,key,base,expected,input}=>{let content=bytes(&input,args.memory_limit_bytes/4)?;value(runtime.project_draft_save_intent(&operation,&project,&branch,&key,&base,expected,&content).await?)},
            Action::DraftSave{intent,input}=>{let content=bytes(&input,args.memory_limit_bytes/4)?;value(runtime.project_save_draft_intent(&intent,&content).await?)},
            Action::DraftAdmit{operation,key}=>package(&runtime.project_admit_draft(&operation,&key,require_physical(&physical)?,&cancel).await?),
            Action::DraftAdmitVersion{version}=>package(&runtime.project_admit_draft_version(&version,require_physical(&physical)?,&cancel).await?),
            Action::PublicationIssue{project,branch,expected,draft,operation}=>value(store.project_publication_intent(&project,&branch,&expected,draft.as_ref(),&operation)?),
            Action::DraftPublish{intent}=>package(&runtime.project_publish_draft(&intent,require_physical(&physical)?,&cancel).await?),
            Action::Compare{left,right,source_only}=>{let left=selected(&runtime,store,&left,&physical).await?;let right=selected(&runtime,store,&right,&physical).await?;if source_only{value(left.project_compare_source(&right,&cancel).await?)}else{value(left.project_compare(&right,&cancel).await?)}},
            Action::SourceDetail{revision,logical}=>{let source=selected(&runtime,store,&revision,&physical).await?;value(source.project_source_detail(&logical,&cancel).await?)},
            Action::MergePreview{base,target,proposal,resolutions}=>{let base=selected(&runtime,store,&base,&physical).await?;let target=selected(&runtime,store,&target,&physical).await?;let proposal=selected(&runtime,store,&proposal,&physical).await?;value(runtime.project_merge_preview(&base,&target,&proposal,&resolutions,&cancel).await?)},
            Action::MergeIssue{publication,base,target,proposal,resolutions}=>{let base=selected(&runtime,store,&base,&physical).await?;let target=selected(&runtime,store,&target,&physical).await?;let proposal=selected(&runtime,store,&proposal,&physical).await?;value(runtime.project_merge_intent(publication,&base,&target,&proposal,resolutions,&cancel).await?)},
            Action::MergeDraftIssue{publication,base,target,version,resolutions}=>{let base=selected(&runtime,store,&base,&physical).await?;let target=selected(&runtime,store,&target,&physical).await?;value(runtime.project_merge_draft_intent(publication,&base,&target,&version,resolutions,&cancel).await?)},
            Action::MergePublish{intent}=>package(&runtime.project_publish_merge(&intent,require_physical(&physical)?,&cancel).await?),
            Action::UndoIssue{publication,target,restore}=>{let target=selected(&runtime,store,&target,&physical).await?;let restore=selected(&runtime,store,&restore,&physical).await?;value(runtime.project_undo_intent(publication,&target,&restore,&cancel).await?)},
            Action::ContextRecord{operation,project,scope,revision,effective_from_us,effective_to_us,corrects,deleted}=>{let source=selected(&runtime,store,&revision,&physical).await?;value(runtime.project_record_context(&operation,&project,&scope,&source,effective_from_us,effective_to_us,corrects.as_deref(),deleted).await?)},
            Action::ContextCut{project}=>value(store.project_context_cut(&project).await?),
            Action::ContextSelect{cut,scope,effective_at_us}=>value(store.select_project_context(&cut,&scope,effective_at_us).await?),
            Action::ContextLoad{selection}=>package(&runtime.project_context_package(&selection,require_physical(&physical)?,&cancel).await?),
            Action::PreparationDiscovery{revision,request}=>{let source=selected(&runtime,store,&revision,&physical).await?;value(source.preparation_discovery(&request,&cancel).await?)},
        }
    }.await;
    signal.abort();let _=signal.await;
    runtime.close();shared.math().shutdown_native_lanes().await;drop(runtime);drop(shared);result
}


#[cfg(all(test,feature="canonical-tests"))]
mod tests {
    use super::*;
    #[tokio::test]
    async fn project_cli_runtime_import_fork_compare_and_passive_discovery() {
        let store=pse_operations::testing::canonical_fixture_store().unwrap();
        let directory=tempfile::tempdir().unwrap();
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let physical_root=root.join("tests/fixtures/packages/physical-primitives");
        let physical=ProjectDraftDocuments{version:Default::default(),packages:vec![["package.toml","materials/physical.yaml","materials/time.yaml"].into_iter().map(|path|(path.into(),std::fs::read(physical_root.join(path)).unwrap())).collect()]};
        let mut source=std::fs::read_to_string(root.join("tests/fixtures/packages/minimal_named/package.toml")).unwrap();
        source=source.replace("dependencies = []",r#"dependencies = [{ name = "physical-primitives", package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#);
        let docs=ProjectDraftDocuments{version:Default::default(),packages:vec![BTreeMap::from([("package.toml".into(),source.into_bytes()),("models/root.pse".into(),b"package minimal_named {param k:Scalar=1; def Root {var x:Scalar; eq e:x*x==k;}}".to_vec())])]};
        let execute=async |action:Action| {
            let path=directory.path().join("request.json");
            std::fs::write(&path,serde_json::to_vec(&Request{version:Default::default(),physical:Some(physical.clone()),action}).unwrap()).unwrap();
            run(&store,RuntimeArgs{request:path,memory_limit_bytes:8usize<<30,threads:2,spill_dir:directory.path().into(),max_spill_bytes:1<<30,batch_size:8192},Instant::now()+std::time::Duration::from_secs(60)).await.unwrap()
        };
        let imported=execute(Action::Import{documents:docs}).await;
        let original=imported["canonical_revision"].as_str().unwrap().to_owned();
        store.create_project("cli-create","cli-project","CLI integration").await.unwrap();
        let project=store.project("cli-project").await.unwrap().unwrap();
        let fork=execute(Action::Fork{project,branch:"cli-main".into(),problem:"cli-problem".into(),operation:"cli-fork".into(),source:original.clone()}).await;
        let fork=fork.as_str().unwrap().to_owned();
        let compared=execute(Action::Compare{left:original,right:fork.clone(),source_only:false}).await;
        assert!(compared["semantic_context_matches"].as_bool().unwrap());
        assert!(compared["changes"].as_array().unwrap().is_empty());
        let revision=store.revision(&fork).await.unwrap().unwrap();
        let pin=store.protect(revision,std::time::Duration::from_secs(60)).await.unwrap();
        let members=store.membership_page(&pin,None,"").await.unwrap();
        let root=pse_ids::SemanticId::parse_hex(members.iter().find(|row|row.name=="Root").unwrap().logical.strip_prefix("modeling:").unwrap()).unwrap();
        store.release(&pin).await.unwrap();
        let discovery=execute(Action::PreparationDiscovery{revision:fork.clone(),request:PreparationDiscoveryRequest{
            root:pse_model::generated::identities::DeclarationId::from_id(root),instance:pse_model::generated::identities::InstanceId::from_id(root),bindings:Default::default(),limits:Default::default(),durable_after:None,
        }}).await;
        assert_eq!(discovery["durable_eligibility"],"unchecked");
        assert!(discovery["resident"].as_array().unwrap().is_empty());
        let loaded=execute(Action::Load{revision:fork}).await;
        assert_eq!(loaded["canonical_problem"],"cli-problem");
        let meta=store.project_metadata(pse_operations::canonical_projects::ProjectMetadataKind::Projects,"",None).await.unwrap();
        assert_eq!(meta.projects.len(),1);
        let frozen=meta.projects[0].clone();
        store.project_navigate("cli-rename",&frozen,&pse_operations::canonical_projects::ProjectNavigation::ProjectTitle{name:"CLI renamed".into()}).await.unwrap();
        assert_eq!(store.project("cli-project").await.unwrap().unwrap().title,"CLI renamed");
        store.remove_isolated_fixture().await.unwrap();
    }
}
