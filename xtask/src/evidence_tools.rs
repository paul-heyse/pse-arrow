// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit tooling-only evidence lifecycle. Opening never initializes scientific state.
use anyhow::{Context,Result};
use clap::{Args,Subcommand};
use pse_operations::evidence::{self,EvidenceAccess,EvidenceQuery,EvidenceSnapshot,EvidenceStore,ToolingOptions};
use std::{path::{Path,PathBuf},time::{Duration,Instant}};
#[derive(Debug, Args)]
pub(crate) struct EvidenceArgs {
    /// Selected service state; only its private tooling configuration is opened normally.
    #[arg(long)] state:PathBuf,
    /// Remaining original command allowance, shared by every stage and retry.
    #[arg(long,default_value_t=30000)] timeout_ms:u64,
    #[command(subcommand)] command:EvidenceCommand,
}
#[derive(Debug, Subcommand)]
enum EvidenceCommand {
    /// Explicit root-only initialization of the disposable tooling database.
    Init,
    /// Attest tooling interpretation and report the imported source cut.
    Status,
    /// Import original-owner adapted typed metadata, then activate reconciled membership.
    Import { #[arg(long)] input:PathBuf },
    /// Query a bounded page from the current complete imported generation.
    Query { #[arg(long)] generation:Option<String>, #[arg(long,default_value_t=64)] limit:usize,#[arg(long)] kind:Option<String>,#[arg(long)] status:Option<String>,#[arg(long)] cursor:Option<PathBuf> },
    /// Discover bounded prior activated and interrupted source cuts.
    Generations { #[arg(long,default_value_t=64)] limit:usize, #[arg(long)] after:Option<String> },
    /// Explicitly drop and reinitialize the derived index; original owners are unchanged.
    Rebuild,
    /// Explicitly drop only the selected disposable database.
    Drop,
}
fn root_credentials(state:&Path)->Result<(String,String)>{
    #[derive(serde::Deserialize)]struct Deployment{credentials_file:String}
    #[derive(serde::Deserialize)]struct RootCredentials{username:String,password:String}
    fn read(path:&Path)->Result<Vec<u8>>{let m=std::fs::symlink_metadata(path)?;anyhow::ensure!(m.is_file()&&!m.file_type().is_symlink()&&m.len()<=262144,"bounded real credentials required");#[cfg(unix)]{use std::os::unix::fs::MetadataExt;anyhow::ensure!(m.mode()&0o077==0,"credentials must be private");}Ok(std::fs::read(path)?)}
    let d:Deployment=serde_json::from_slice(&read(&state.join("config.json"))?)?;
    let path=Path::new(&d.credentials_file);let path=if path.is_absolute(){path.to_owned()}else{state.join(path)};anyhow::ensure!(path.parent()==Some(state),"service credentials must be a direct private file");
    let c:RootCredentials=serde_json::from_slice(&read(&path)?)?;Ok((c.username,c.password))
}
fn activity_guard(state:&Path,options:&ToolingOptions)->Result<std::fs::File>{
    use std::os::unix::fs::{FileExt,MetadataExt};
    let fd=std::env::var("PSE_TOOLING_ACTIVITY_FD").context("run through just evidence so its actual service activity remains owned")?.parse::<u32>()?;
    anyhow::ensure!(fd>=3,"invalid inherited service activity descriptor");
    let link=PathBuf::from(format!("/proc/self/fd/{fd}"));let path=std::fs::read_link(&link)?;
    let expected=state.canonicalize()?.join(".tooling/.activities");
    let stem=path.file_stem().and_then(|v|v.to_str()).context("invalid activity name")?;
    anyhow::ensure!(path.parent()==Some(expected.as_path())&&path.extension().is_some_and(|v|v=="json")&&stem.len()==32&&stem.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)),"foreign inherited activity path");
    let inherited=std::fs::File::open(link)?;let m=inherited.metadata()?;let actual=std::fs::metadata(&path)?;let uid=std::fs::metadata("/proc/self")?.uid();
    anyhow::ensure!(m.is_file()&&m.dev()==actual.dev()&&m.ino()==actual.ino()&&m.mode()&0o077==0&&m.uid()==uid&&m.len()<=65536,"invalid inherited activity ownership");
    let mut bytes=vec![0;m.len().try_into()?];inherited.read_exact_at(&mut bytes,0)?;
    #[derive(serde::Deserialize)]#[serde(deny_unknown_fields)]struct Activity{version:u32,service_instance_id:String}
    let record:Activity=serde_json::from_slice(&bytes)?;
    anyhow::ensure!(record.version==1&&record.service_instance_id==options.service_instance_id&&std::env::var("PSE_TOOLING_SERVICE_INSTANCE_ID").ok().as_deref()==Some(options.service_instance_id.as_str()),"foreign service activity issuing identity");
    let probe=std::fs::File::open(path)?;
    anyhow::ensure!(matches!(probe.try_lock(),Err(std::fs::TryLockError::WouldBlock)),"activity descriptor does not retain its original lifetime lock");
    Ok(inherited)
}
pub(crate) fn run(args:EvidenceArgs)->Result<()>{
    anyhow::ensure!(args.timeout_ms>0,"positive original allowance required");
    let deadline=Instant::now().checked_add(Duration::from_millis(args.timeout_ms)).context("original allowance overflow")?;
    let options=ToolingOptions::from_state(&args.state)?;
    let _activity=activity_guard(&args.state,&options)?;
    let rt=tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let output=rt.block_on(evidence::within_allowance(deadline,async{
        match args.command {
            EvidenceCommand::Init=>{let (user,password)=root_credentials(&args.state).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))?;EvidenceStore::initialize(&options,&user,&password).await?;Ok(serde_json::json!({"initialized":true}))},
            EvidenceCommand::Drop|EvidenceCommand::Rebuild=>{let rebuild=matches!(args.command,EvidenceCommand::Rebuild);let (user,password)=root_credentials(&args.state).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))?;EvidenceStore::drop_database(&options,&user,&password).await?;if rebuild{EvidenceStore::initialize(&options,&user,&password).await?;}Ok(serde_json::json!({"dropped":true,"rebuilt":rebuild}))},
            command=>{
                let role=if matches!(command,EvidenceCommand::Import{..}){EvidenceAccess::Importer}else{EvidenceAccess::Reader};
                let store=EvidenceStore::open(&options,role).await?;
                let result=async{match command{
                    EvidenceCommand::Status=>store.status().await,
                    EvidenceCommand::Import{input}=>{let metadata=std::fs::metadata(&input).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))?;if !metadata.is_file()||metadata.len()>4*1024*1024{return Err(evidence::EvidenceError::Configuration("bounded snapshot file required".into()));}let bytes=std::fs::read(input).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))?;let snapshot:EvidenceSnapshot=serde_json::from_slice(&bytes).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))?;serde_json::to_value(store.import(&snapshot).await?).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))},
                    EvidenceCommand::Generations{limit,after}=>store.generations(after.as_deref(),limit).await,
                    EvidenceCommand::Query{generation,limit,kind,status,cursor}=>{let cursor=cursor.map(|path|{let m=std::fs::metadata(&path).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))?;if !m.is_file()||m.len()>16384{return Err(evidence::EvidenceError::Configuration("bounded cursor file required".into()));}let bytes=std::fs::read(path).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))?;serde_json::from_slice(&bytes).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))}).transpose()?;serde_json::to_value(store.select(&EvidenceQuery{generation,kind,status,limit:Some(limit),cursor}).await?).map_err(|e|evidence::EvidenceError::Configuration(e.to_string()))},
                    _=>Err(evidence::EvidenceError::Configuration("unsupported command".into())),
                }}.await;
                let drained=store.disconnect().await;let output=result?;drained?;Ok(output)
            }
        }
    }))?;
    println!("{}",serde_json::to_string(&output)?);Ok(())
}
