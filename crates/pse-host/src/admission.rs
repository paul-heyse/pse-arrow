// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite host admission, coordinated across checkouts against actual manager lifetimes.
//!
//! Ordinary work has no reservation row, memory cap or owner-count limit. Its admission
//! increments the shared revision so exclusive snapshots cannot miss an admitted gate.
use crate::{HostError, HostManager, LaunchIntent, OwnedUnit, UnitObservation, environment::Environment};
use crate::manager::{FiniteParentRequest, ResourceControls, ResidentServiceFacts, UnitBinding};
use pse_diagnostics::process_facts::{ProcessIdentity, ProcessRead, ProcessReader};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};

const GIB: u64 = 1 << 30;
const MARKER: &str = "PSE_HOST_ALLOCATION";
fn invalid(message: impl Into<String>) -> HostError { HostError::Ownership(message.into()) }
fn json(error: impl std::fmt::Display) -> HostError { invalid(format!("finite admission record: {error}")) }
fn checked_gib(value: u64) -> Result<u64, HostError> { value.checked_mul(GIB).ok_or_else(|| invalid("capacity declaration overflow")) }
fn value<'a>(environment: &'a Environment, key: &str) -> Result<Option<&'a str>, HostError> {
    environment.get(OsStr::new(key)).map(|value| value.to_str().ok_or_else(|| invalid(format!("{key} must be UTF-8")))).transpose()
}

/// Validate a named host role without loading configuration or starting helpers.
pub fn validate_class(class:&str)->Result<(),HostError> {
    if matches!(class,"build" | "light" | "compile" | "functional" | "wide" | "timing" | "reference" | "exclusive" | "exclusive-observer" | "compiler-cache" | "store-functional" | "store-timing") { Ok(()) }
    else { Err(invalid("unknown resource class")) }
}
fn slice_syntax(slice:&str)->bool {
    slice.ends_with(".slice") && slice.len()>6 && slice.bytes().all(|byte|byte.is_ascii_alphanumeric() || matches!(byte,b'-' | b'_' | b'.'))
}
/// Reject malformed explicit caller declarations before activation, without
/// consulting personal selection, policy files, host capacity or the manager.
pub fn validate_request(environment:&Environment)->Result<(),HostError> {
    if value(environment,"PSE_BUILD_MODE")?.is_some_and(|mode|!matches!(mode,"bounded" | "unbounded")) { return Err(invalid("PSE_BUILD_MODE must be bounded or unbounded")); }
    if let Some(class)=value(environment,"PSE_RESOURCE_CLASS")? { validate_class(class)?; }
    if let Some(request)=value(environment,"PSE_MEMORY_MAX")? { finite_bytes(request,100)?; }
    if value(environment,"PSE_SLICE")?.is_some_and(|slice| !slice.is_empty() && slice!="none" && !slice_syntax(slice)) { return Err(invalid("PSE_SLICE must name a slice or none")); }
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
struct Policy {
    version: u32,
    aggregate_gib: u64,
    non_agent_headroom_gib: u64,
    pressure_guard_gib: u64,
    admission_seconds: u64,
    functional_cores: Vec<u32>,
    timing_cores: Vec<u32>,
    light: Lane,
    #[serde(rename = "compiler-cache")]
    compiler_cache: Lane,
    compile: Lane,
    functional: Lane,
    wide: Lane,
    timing: Lane,
    exclusive: Lane,
    #[serde(rename = "exclusive-observer")]
    exclusive_observer: Lane,
    production: Production,
}
#[derive(Clone, Debug, Deserialize)]
struct Lane {
    #[serde(default)] slots: u64,
    #[serde(default)] memory_gib: u64,
    #[serde(default)] slot_gib: u64,
    #[serde(default)] store_gib: u64,
    #[serde(default)] store_slots: u64,
    #[serde(default)] light_gib: u64,
}
#[derive(Clone, Debug, Deserialize)]
struct Production { version: u32, profiles: Vec<ProductionProfile> }
#[derive(Clone, Debug, Deserialize)]
struct ProductionProfile { name: String, total_memory_bytes: u64, native_worker_memory_bytes: u64 }

/// Exact finite declaration; lane slots are independent of the physical aggregate ceiling.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Profile {
    /// Selected class, after explicit request widening.
    pub name: String,
    /// Workload envelope in bytes, zero only for ordinary work.
    pub memory: u64,
    /// Actual admission lane.
    pub lane: String,
    /// Declared finite lane charge, zero for ordinary work.
    pub slots: u64,
    /// Configured physical cores, expanded to logical siblings only for finite controls.
    pub cores: Vec<u32>,
    /// Exclusive heavy-work ownership.
    pub exclusive: bool,
    /// Aggregate ancestor ceiling, including resident roles for exclusive/reference work.
    pub aggregate: u64,
    /// This allocation's ceiling including its resident roles.
    pub allocation: u64,
}
impl Profile {
    /// Ordinary builds do not take numerical host reservations.
    pub fn ordinary(&self) -> bool { self.name == "build" && self.memory == 0 }
    /// Enforced workload controls. Ordinary work adds none.
    pub fn controls(&self) -> Result<ResourceControls, HostError> {
        if self.ordinary() { return Ok(ResourceControls::default()); }
        let cpus = logical_cpus(&self.cores)?;
        let mut mask = vec![0_u8; cpus.iter().copied().max().map_or(0, |cpu| cpu as usize / 8 + 1)];
        for cpu in &cpus { mask[*cpu as usize / 8] |= 1 << (*cpu % 8); }
        Ok(ResourceControls {
            memory_max: Some(self.memory),
            memory_swap_max: Some(0),
            cpu_quota_per_sec_usec: Some(u64::try_from(cpus.len()).map_err(json)? * 1_000_000),
            allowed_cpus: mask,
        })
    }
}

/// Validated policy and caller selection, before any manager effect.
#[derive(Clone, Debug)]
pub struct Selection { policy: Policy, profile: Profile, slice: String, explicit_class: bool }
impl Selection {
    /// Command selection wins over caller environment, then recipe/default and local mode.
    pub fn load(root: &Path, environment: &Environment, explicit: Option<&str>, recipe: Option<&str>) -> Result<Self, HostError> {
        let policy: Policy = toml::from_str(&fs::read_to_string(root.join(".config/agent-capacity.toml"))?).map_err(json)?;
        let total = meminfo("MemTotal")?;
        Self::from_policy(policy, environment, explicit, recipe, total)
    }
    fn from_policy(policy: Policy, environment: &Environment, explicit: Option<&str>, recipe: Option<&str>, total: u64) -> Result<Self, HostError> {
        validate_request(environment)?;
        if policy.version != 1 || policy.production.version != 1 || policy.admission_seconds == 0
            || policy.functional.slots == 0 || policy.light.slots == 0
            || policy.functional.store_slots == 0 || policy.timing.store_slots == 0
            || policy.functional_cores.is_empty() || policy.timing_cores.is_empty()
        { return Err(invalid("invalid host capacity policy")); }
        let mode = value(environment,"PSE_BUILD_MODE")?.unwrap_or("bounded");
        if !matches!(mode,"bounded" | "unbounded") { return Err(invalid("PSE_BUILD_MODE must be bounded or unbounded")); }
        let request = value(environment,"PSE_MEMORY_MAX")?;
        let direct = explicit.or(value(environment,"PSE_RESOURCE_CLASS")?);
        let mut name = direct.or(recipe).unwrap_or("functional").to_owned();
        validate_class(&name)?;
        let timing = value(environment,"PSE_BUILD_MEASUREMENT_TIMING")?;
        if timing.is_some() {
            if direct.is_some_and(|name| !matches!(name,"timing" | "reference" | "exclusive-observer")) {
                return Err(invalid("timing measurement requires its explicit isolated resource class"));
            }
            if direct.is_none() { name = "timing".into(); }
        } else if direct.is_none() && mode == "unbounded" && request.is_none()
            && matches!(name.as_str(),"light" | "compile" | "functional" | "wide" | "exclusive" | "build")
        { name = "build".into(); }
        let slice = value(environment,"PSE_SLICE")?.unwrap_or("pse.slice");
        if name != "build" && slice != "pse.slice" && (!slice.starts_with("pse-") || !slice_syntax(slice)) {
            return Err(invalid("finite placement must preserve the declared pse.slice aggregate ancestry"));
        }
        let amount = request.map(|request| finite_bytes(request,total)).transpose()?;
        let all_cores = policy.functional_cores.iter().chain(&policy.timing_cores).copied().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        if name == "build" {
            if amount.is_some() { return Err(invalid("unbounded build cannot take PSE_MEMORY_MAX; select a finite class")); }
            return Ok(Self { policy, profile: Profile { name, memory:0,lane:"build".into(),slots:0,cores:all_cores,exclusive:false,aggregate:0,allocation:0 }, slice:slice.to_owned(), explicit_class:explicit.is_some() });
        }
        if matches!(name.as_str(),"light" | "compile" | "functional" | "wide" | "exclusive") {
            if let Some(amount) = amount {
                if amount > checked_gib(policy.wide.slot_gib)? { name = "exclusive".into(); }
                else if amount > checked_gib(policy.functional.slot_gib)? { name = "wide".into(); }
                else if name == "light" && amount > checked_gib(policy.light.memory_gib)? { name = "functional".into(); }
            }
        }
        let (declared,lane,slots,cores,exclusive) = match name.as_str() {
            "light" => (checked_gib(policy.light.memory_gib)?,"light",1,policy.functional_cores.clone(),false),
            "compiler-cache" => (checked_gib(policy.compiler_cache.memory_gib)?,"light",1,policy.functional_cores.clone(),false),
            "compile" => (checked_gib(policy.compile.memory_gib)?,"functional",1,policy.functional_cores.clone(),false),
            "functional" => (checked_gib(policy.functional.slot_gib)?,"functional",1,policy.functional_cores.clone(),false),
            "wide" => (checked_gib(policy.wide.slot_gib)?,"functional",2,policy.functional_cores.clone(),false),
            "timing" => (checked_gib(policy.timing.slot_gib)?,"timing",1,policy.timing_cores.clone(),false),
            "store-functional" => (checked_gib(policy.functional.store_gib)?,"store-functional",1,policy.functional_cores.clone(),false),
            "store-timing" => (checked_gib(policy.timing.store_gib)?,"store-timing",1,policy.timing_cores.clone(),false),
            "exclusive" => (checked_gib(policy.exclusive.slot_gib)?,"exclusive",2,all_cores,false),
            "exclusive-observer" => (checked_gib(policy.exclusive_observer.slot_gib)?,"exclusive",2,all_cores,true),
            "reference" => {
                let reference = policy.production.profiles.iter().find(|profile| profile.name == "reference").ok_or_else(|| invalid("reference production profile absent"))?;
                (reference.native_worker_memory_bytes,"exclusive",2,all_cores,true)
            }
            _ => return Err(invalid("unknown resource class")),
        };
        let exclusive = exclusive || name == "exclusive";
        let memory = amount.unwrap_or(declared);
        if memory == 0 { return Err(invalid("finite profile has no memory ceiling")); }
        if matches!(name.as_str(),"compiler-cache" | "reference") && memory != declared { return Err(invalid("selected role requires its exact declared memory envelope")); }
        if name.starts_with("store-") && memory > declared { return Err(invalid("resident store request exceeds its declared lane")); }
        let combined = policy.production.profiles.iter().find(|profile| profile.name == name);
        let resident = checked_gib(policy.exclusive.store_gib.checked_add(policy.exclusive.light_gib).ok_or_else(|| invalid("resident envelope overflow"))?)?;
        let allocation = if matches!(name.as_str(),"reference" | "exclusive-observer") { combined.ok_or_else(|| invalid("combined production profile absent"))?.total_memory_bytes }
            else if exclusive { memory.checked_add(resident).ok_or_else(|| invalid("allocation envelope overflow"))? } else { memory };
        let aggregate = checked_gib(policy.aggregate_gib)?.max(allocation);
        let maximum = total.checked_sub(checked_gib(policy.non_agent_headroom_gib)?).ok_or_else(|| invalid("non-agent headroom exceeds memory"))?;
        if aggregate > maximum { return Err(invalid("requested aggregate exceeds host memory after non-agent allowance")); }
        Ok(Self { policy, profile: Profile {name,memory,lane:lane.into(),slots,cores,exclusive,aggregate,allocation}, slice:slice.to_owned(), explicit_class:explicit.is_some() })
    }
    /// Select child environment before spawning its closed gate. These strings
    /// grant no authority until actual reservation and original scope binding.
    pub fn apply_pending_environment(&self,environment:&mut Environment,workload:&LaunchIntent) {
        if self.profile.ordinary() {
            environment.remove(OsStr::new(MARKER));
            environment.remove(OsStr::new("PSE_RESOURCE_CLASS"));
        } else {
            environment.insert(MARKER.into(),workload.unit().into());
            environment.insert("PSE_RESOURCE_CLASS".into(),self.profile.name.clone().into());
        }
    }
    /// Selected profile, including exact lane charges.
    pub fn profile(&self) -> &Profile { &self.profile }
    /// Configured finite parent. Ordinary foreground placement retains its actual parent.
    pub fn slice(&self) -> &str { &self.slice }
    /// Policy admission duration; callers intersect it with their original startup deadline.
    pub fn admission_timeout(&self) -> Duration { Duration::from_secs(self.policy.admission_seconds) }
    fn conflict(&self, rows: impl Iterator<Item = Profile>, ordinary_live: bool) -> Option<String> {
        let entries = rows.collect::<Vec<_>>();
        if (self.profile.exclusive || self.profile.lane == "timing" || self.profile.lane == "store-timing") && ordinary_live {
            return Some("ordinary workload remains physically present".into());
        }
        let timing = |lane:&str| matches!(lane,"timing" | "store-timing");
        let competing = |lane:&str| matches!(lane,"functional" | "store-functional" | "build" | "exclusive");
        if entries.iter().any(|entry| timing(&self.profile.lane) && competing(&entry.lane) || competing(&self.profile.lane) && timing(&entry.lane)) { return Some("isolated timing owner".into()); }
        if entries.iter().any(|entry| self.profile.exclusive && entry.lane != "light" || self.profile.lane != "light" && entry.exclusive) { return Some("exclusive heavy-work owner".into()); }
        if self.profile.ordinary() { return None; }
        let Some(used) = entries.iter().filter(|entry| entry.lane == self.profile.lane)
            .try_fold(0_u64, |used,entry| used.checked_add(entry.slots))
        else { return Some("finite slot charge overflow".into()); };
        let capacity = match self.profile.lane.as_str() {
            "light" => self.policy.light.slots,
            "functional" => self.policy.functional.slots,
            "store-functional" => self.policy.functional.store_slots,
            "store-timing" => self.policy.timing.store_slots,
            "timing" => 1,
            _ => 2,
        };
        (used.checked_add(self.profile.slots).is_none_or(|total| total > capacity)).then(|| format!("{} slots occupied",self.profile.lane))
    }
}

fn finite_bytes(value:&str,total:u64)->Result<u64,HostError> {
    let upper=value.trim().to_ascii_uppercase();
    let digits=upper.bytes().take_while(u8::is_ascii_digit).count();
    if digits==0 { return Err(invalid("PSE_MEMORY_MAX must be a positive finite byte declaration")); }
    let count=upper[..digits].parse::<u64>().map_err(json)?;
    let suffix=&upper[digits..];
    let result=match suffix {
        "%" => (u128::from(count)*u128::from(total)/100).try_into().ok(),
        "" => Some(count),
        "K" => count.checked_mul(1<<10), "M" => count.checked_mul(1<<20),
        "G" => count.checked_mul(GIB), "T" => count.checked_mul(1<<40), _ => None,
    }.filter(|amount|*amount>0);
    result.ok_or_else(||invalid("PSE_MEMORY_MAX must be a positive finite byte declaration"))
}
fn meminfo(key:&str)->Result<u64,HostError> {
    let contents=fs::read_to_string("/proc/meminfo")?;
    let row=contents.lines().find_map(|line|line.strip_prefix(&format!("{key}:"))).ok_or_else(||invalid("kernel memory fact absent"))?;
    let mut fields=row.split_whitespace();
    let value=fields.next().ok_or_else(||invalid("kernel memory value absent"))?.parse::<u64>().map_err(json)?;
    if fields.next()!=Some("kB") || fields.next().is_some() { return Err(invalid("kernel memory unit unavailable")); }
    value.checked_mul(1024).ok_or_else(||invalid("kernel memory value overflow"))
}
fn logical_cpus(physical:&[u32])->Result<Vec<u32>,HostError> {
    let mut result=BTreeSet::new();
    for core in physical {
        let text=fs::read_to_string(format!("/sys/devices/system/cpu/cpu{core}/topology/thread_siblings_list"))?;
        let mut siblings=BTreeSet::new();
        for span in text.trim().split(',') {
            let (first,last)=span.split_once('-').unwrap_or((span,span));
            let first=first.parse::<u32>().map_err(json)?;
            let last=last.parse::<u32>().map_err(json)?;
            if last<first || last-first>65535 { return Err(invalid("invalid CPU sibling topology")); }
            siblings.extend(first..=last);
        }
        if siblings.first()!=Some(core) { return Err(invalid("host topology differs from declared physical cores")); }
        result.extend(siblings);
    }
    Ok(result.into_iter().collect())
}

/// Read-only physical capacity input from a separately owned original resident.
/// It has no stop/ref/release method and is never serialized into our journal.
#[derive(Clone,Debug)]
pub(crate) struct BorrowedResidentCharge { facts:ResidentServiceFacts, profile:Profile }
impl BorrowedResidentCharge {
    pub(crate) fn facts(&self)->&ResidentServiceFacts { &self.facts }
    pub(crate) fn matches(&self,unit:&UnitObservation,device:u64,inode:u64)->bool {
        self.facts.observation.unit==unit.unit && self.facts.binding.invocation==unit.invocation
            && self.facts.binding.group==unit.control_group && self.facts.binding.device==device && self.facts.binding.inode==inode
    }
    fn already_charged(&self,journal:&Journal)->bool {
        journal.finite.values().any(|row|row.profile==self.profile && row.units.iter().any(|unit|unit.intent.unit()==self.facts.observation.unit && unit.binding.as_ref()==Some(&self.facts.binding)))
    }
}
#[derive(Deserialize)]
struct LegacyLedger { version:u32, owners:BTreeMap<String,serde_json::Value> }
#[derive(Clone,Deserialize)]
struct LegacyResidentAllocation {
    boot:String,
    class:String,
    lane:String,
    memory:u64,
    slots:u64,
    cores:Vec<u32>,
    exclusive:bool,
    units:BTreeMap<String,LegacyUnitBinding>,
}
#[derive(Clone,Deserialize)]
struct LegacyUnitBinding { group:String, inode:u64, invocation:String }
#[derive(Clone,Deserialize)]
struct LegacyCacheRecord {
    version:u32,
    unit:String,
    generation:String,
    record:PathBuf,
    allocation:BTreeMap<String,String>,
    startup_inputs:BTreeMap<String,String>,
}
fn read_private_json<T:serde::de::DeserializeOwned>(path:&Path)->Result<T,HostError> {
    let parent=path.parent().ok_or_else(||invalid("resident metadata parent absent"))?;
    private_directory(parent,false)?;
    let file=OpenOptions::new().read(true).custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32).open(path)?;
    private_file(&file)?;
    if file.metadata()?.len()>16*1024*1024 { return Err(invalid("resident metadata exceeds bounded envelope")); }
    serde_json::from_reader(file).map_err(json)
}
fn resident_launch_record(facts:&ResidentServiceFacts,cache_root:&Path)->Result<(PathBuf,String),HostError> {
    let [executable,script,mode,record]=facts.arguments.as_slice() else { return Err(invalid("resident cache launch is not the original serve protocol")); };
    let path=PathBuf::from(record);
    let relative=path.strip_prefix(cache_root).map_err(|_|invalid("resident record is outside its configured source metadata root"))?;
    let components=relative.components().collect::<Vec<_>>();
    if components.len()!=3 || !components.iter().all(|component|matches!(component,Component::Normal(_))) || path.file_name()!=Some(OsStr::new("service.json"))
        || executable!=&facts.executable || mode!="--pse-cache-serve" || Path::new(script).file_name()!=Some(OsStr::new("compiler_cache.py"))
        || Path::new(script).parent().and_then(Path::file_name)!=Some(OsStr::new("scripts"))
    { return Err(invalid("resident launch declaration does not identify its original cache role")); }
    let mut fragment=OpenOptions::new().read(true).open(&facts.fragment)?;
    let meta=fragment.metadata()?;
    if !meta.is_file() || meta.uid()!=rustix::process::geteuid().as_raw() || meta.mode() & 0o022!=0 || facts.fragment.file_name()!=Some(OsStr::new(&facts.observation.unit)) {
        return Err(invalid("resident configured unit provenance is unavailable or mutable"));
    }
    // The manager's original argv remains authoritative; this file is only
    // supported installed provenance, not a replacement INI policy interpreter.
    let mut text=String::new();
    io::Read::read_to_string(&mut fragment,&mut text)?;
    if !text.lines().any(|line|line.trim()=="[Service]") { return Err(invalid("resident installed service declaration absent")); }
    Ok((path,script.clone()))
}
fn validate_resident_declaration(facts:&ResidentServiceFacts,record:&LegacyCacheRecord,row:&LegacyResidentAllocation,path:&Path,script:&str,boot:&str,profile:&Profile)->Result<(),HostError> {
    let original=row.units.get(&facts.observation.unit).ok_or_else(||invalid("resident original unit binding absent"))?;
    let invocation=original.invocation.as_bytes().chunks_exact(2).map(|pair|std::str::from_utf8(pair).ok().and_then(|pair|u8::from_str_radix(pair,16).ok())).collect::<Option<Vec<_>>>();
    let source=record.startup_inputs.get(script);
    if record.version!=1 || record.record!=path || record.unit!=facts.observation.unit
        || record.generation.len()!=32 || !record.generation.bytes().all(|byte|byte.is_ascii_hexdigit())
        || record.unit!=format!("pse-compiler-cache-{}.service",record.generation)
        || path.parent().and_then(Path::file_name)!=Some(OsStr::new(&record.generation))
        || record.allocation.get("PSE_RESOURCE_CLASS").map(String::as_str)!=Some("compiler-cache")
        || source.is_none_or(|hash|hash.len()!=64 || !hash.bytes().all(|byte|byte.is_ascii_hexdigit()))
        || row.boot!=boot || row.class!="compiler-cache" || row.lane!="light" || row.exclusive || row.slots!=1
        || row.memory!=profile.memory || row.cores!=profile.cores || facts.memory_max!=profile.memory || facts.memory_swap_max!=0
        || facts.kernel_memory_max!=profile.memory || facts.kernel_memory_swap_max!=0
        || original.group!=facts.binding.group || original.inode!=facts.binding.inode || original.invocation.len()!=32 || invocation.as_ref()!=Some(&facts.binding.invocation)
    { return Err(invalid("resident role declaration differs from original manager/kernel/capacity facts")); }
    let allocation=record.allocation.get(MARKER).ok_or_else(||invalid("resident allocation marker absent"))?;
    let nonce=Path::new(allocation).file_name().and_then(OsStr::to_str).ok_or_else(||invalid("resident allocation identity absent"))?;
    if facts.slice!=format!("pse-allocation-{nonce}.slice") { return Err(invalid("resident original allocation slice differs")); }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct RegisteredUnit { intent: LaunchIntent, binding: Option<UnitBinding>, controller: bool }
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct Reservation {
    boot: String,
    manager: String,
    caller: ProcessIdentity,
    // An unbound gated child cannot fork or exec raw work: its private gate
    // remains closed until the original workload binding is committed here.
    gated_child: ProcessIdentity,
    profile: Profile,
    units: Vec<RegisteredUnit>,
    released: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Journal { version: u32, revision: u64, finite: BTreeMap<String,Reservation> }
impl Default for Journal {
    fn default()->Self { Self {version:1, revision:0, finite:BTreeMap::new()} }
}
impl Journal {
    fn advance(&mut self)->Result<(),HostError> {
        self.revision=self.revision.checked_add(1).ok_or_else(||invalid("admission revision exhausted"))?;
        Ok(())
    }
    fn validate(&self)->Result<(),HostError> {
        if self.version!=1 { return Err(invalid("unsupported finite admission journal")); }
        for (nonce,row) in &self.finite {
            if nonce.is_empty() || nonce.bytes().any(|byte|!byte.is_ascii_alphanumeric() && !matches!(byte,b'-' | b'_' | b'.'))
                || row.profile.ordinary() || row.profile.memory==0 || row.profile.slots==0
                || row.units.is_empty() || row.manager.is_empty() || row.boot!=row.caller.boot_id || row.boot!=row.gated_child.boot_id
                || !row.units.iter().any(|unit|unit.intent.unit()==nonce && !unit.controller && unit.intent.unit().ends_with(".scope"))
                || row.units.iter().any(|unit| !unit.intent.unit().ends_with(".scope") && !unit.intent.unit().ends_with(".service") && !unit.intent.unit().ends_with(".slice"))
            { return Err(invalid("invalid finite reservation identity or charge")); }
        }
        Ok(())
    }
}

/// One per-user runtime coordinator, independent of checkout location.
#[derive(Clone, Debug)]
pub struct Coordinator { directory: PathBuf, declared_parent: PathBuf, legacy_state: Option<PathBuf> }
impl Coordinator {
    /// Open only the explicitly selected user runtime directory, with private state.
    pub fn open(root:&Path, environment:&Environment)->Result<Self,HostError> {
        let runtime=environment.get(OsStr::new("XDG_RUNTIME_DIR")).ok_or_else(||invalid("XDG_RUNTIME_DIR is required for shared host admission"))?;
        let runtime=PathBuf::from(runtime);
        if !runtime.is_absolute() { return Err(invalid("XDG_RUNTIME_DIR must be absolute")); }
        private_directory(&runtime,false)?;
        let directory=runtime.join("pse-host-admission");
        private_directory(&directory,true)?;
        let legacy_state=environment.get(OsStr::new("XDG_STATE_HOME")).map(PathBuf::from)
            .or_else(||environment.get(OsStr::new("HOME")).map(|home|PathBuf::from(home).join(".local/state")));
        Ok(Self {directory,declared_parent:root.join(".config/systemd/pse.slice"),legacy_state})
    }
    /// Open the same shared runtime for ordinary exclusion without policy/config loading.
    pub fn ordinary(environment:&Environment)->Result<Self,HostError> {
        Self::open(Path::new(""),environment)
    }
    async fn lock(&self,name:&str,deadline:Instant)->Result<File,HostError> {
        let file=OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32).open(self.directory.join(name))?;
        private_file(&file)?;
        loop {
            if Instant::now()>=deadline { return Err(HostError::Deadline("shared admission lock".into())); }
            match rustix::fs::flock(&file,rustix::fs::FlockOperation::NonBlockingLockExclusive) {
                Ok(())=>return Ok(file),
                Err(rustix::io::Errno::WOULDBLOCK)=>tokio::time::sleep_until((Instant::now()+Duration::from_millis(5)).min(deadline).into()).await,
                Err(error)=>return Err(io::Error::from(error).into()),
            }
        }
    }
    fn read(&self)->Result<Journal,HostError> {
        let path=self.directory.join("finite.json");
        let file=match OpenOptions::new().read(true).custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32).open(path) {
            Ok(file)=>file,
            Err(error) if error.kind()==io::ErrorKind::NotFound=>return Ok(Journal::default()),
            Err(error)=>return Err(error.into()),
        };
        private_file(&file)?;
        if file.metadata()?.len()>16*1024*1024 { return Err(invalid("finite journal exceeds its bounded envelope")); }
        let state:Journal=serde_json::from_reader(file).map_err(json)?;
        state.validate()?;
        Ok(state)
    }
    fn write(&self,state:&Journal)->Result<(),HostError> {
        state.validate()?;
        let mut file=tempfile::NamedTempFile::new_in(&self.directory)?;
        serde_json::to_writer(file.as_file_mut(),state).map_err(json)?;
        file.as_file_mut().flush()?;
        file.as_file().sync_all()?;
        file.persist(self.directory.join("finite.json")).map_err(|error|error.error)?;
        File::open(&self.directory)?.sync_all()?;
        Ok(())
    }
    fn parent_proof(&self)->Result<Option<crate::manager::ParentProof>,HostError> {
        let file=match OpenOptions::new().read(true).custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32).open(self.directory.join("parent.json")) {
            Ok(file)=>file,
            Err(error) if error.kind()==io::ErrorKind::NotFound=>return Ok(None),
            Err(error)=>return Err(error.into()),
        };
        private_file(&file)?;
        if file.metadata()?.len()>65536 { return Err(invalid("parent provenance exceeds bounded envelope")); }
        Ok(Some(serde_json::from_reader(file).map_err(json)?))
    }
    fn write_parent_proof(&self,proof:&crate::manager::ParentProof)->Result<(),HostError> {
        let mut file=tempfile::NamedTempFile::new_in(&self.directory)?;
        serde_json::to_writer(file.as_file_mut(),proof).map_err(json)?;
        file.as_file_mut().flush()?;
        file.as_file().sync_all()?;
        file.persist(self.directory.join("parent.json")).map_err(|error|error.error)?;
        Ok(())
    }
    async fn snapshot(&self,deadline:Instant)->Result<Journal,HostError> {
        let _guard=self.lock("admission.lock",deadline).await?;
        self.read()
    }
    async fn commit(&self,expected:u64,removed:&BTreeSet<String>,insert:Option<(&str,&Reservation)>,ordinary:bool,deadline:Instant)->Result<bool,HostError> {
        let _guard=self.lock("admission.lock",deadline).await?;
        let mut current=self.read()?;
        if current.revision!=expected { return Ok(false); }
        for nonce in removed { current.finite.remove(nonce); }
        if let Some((nonce,row))=insert {
            if current.finite.contains_key(nonce) { return Err(invalid("original finite launch is already registered; do not replay")); }
            current.finite.insert(nonce.to_owned(),row.clone());
        }
        if ordinary || insert.is_some() || !removed.is_empty() {
            current.advance()?;
            self.write(&current)?;
        }
        Ok(true)
    }
    async fn retire_released<F,Fut>(&self,nonce:&str,deadline:Instant,mut observe:F)->Result<bool,HostError>
    where F:FnMut(Journal)->Fut,Fut:Future<Output=Result<BTreeSet<String>,HostError>> {
        loop {
            let snapshot=self.snapshot(deadline).await?;
            if !snapshot.finite.contains_key(nonce) { return Ok(true); }
            let revision=snapshot.revision;
            let reclaimed=observe(snapshot).await?;
            if Instant::now()>=deadline { return Err(HostError::Deadline("original finite release reconciliation".into())); }
            if !reclaimed.contains(nonce) { return Ok(false); }
            if self.commit(revision,&BTreeSet::from([nonce.to_owned()]),None,false,deadline).await? { return Ok(true); }
            // An unrelated admission/release changed the journal while physical
            // facts were read. Obtain fresh facts under the same original clock.
        }
    }
    async fn borrowed_residents(&self,manager:&HostManager,selection:&Selection,deadline:Instant)->Result<Vec<BorrowedResidentCharge>,HostError> {
        let candidates=tokio::time::timeout_at(deadline.into(),manager.resident_cache_candidates()).await.map_err(|_|HostError::Deadline("resident capacity observation".into()))??;
        if candidates.is_empty() { return Ok(Vec::new()); }
        let state=self.legacy_state.as_ref().ok_or_else(||invalid("live legacy resident has no configured original metadata root"))?;
        if !state.is_absolute() { return Err(invalid("legacy resident observation state root must be absolute")); }
        let root=state.join("pse-arrow");
        let ledger:LegacyLedger=read_private_json(&root.join("host-admission/allocations.json"))?;
        if ledger.version!=1 { return Err(invalid("unsupported original resident allocation metadata")); }
        let resident_selection=Selection::from_policy(selection.policy.clone(),&Environment::new(),Some("compiler-cache"),None,meminfo("MemTotal")?)?;
        let mut result=Vec::new();
        let mut seen=BTreeSet::new();
        for facts in candidates {
            if !seen.insert(facts.observation.unit.clone()) { continue; }
            let (record_path,script)=resident_launch_record(&facts,&root.join("compiler-cache"))?;
            let record:LegacyCacheRecord=read_private_json(&record_path)?;
            let allocation=Path::new(record.allocation.get(MARKER).ok_or_else(||invalid("resident allocation declaration absent"))?);
            if allocation.parent()!=Some(root.join("host-admission").as_path()) { return Err(invalid("resident allocation source is outside its configured original owner")); }
            let nonce=allocation.file_name().and_then(OsStr::to_str).ok_or_else(||invalid("resident allocation nonce absent"))?;
            let row_value=ledger.owners.get(nonce).ok_or_else(||invalid("resident original role declaration absent"))?;
            let row:LegacyResidentAllocation=serde_json::from_value(row_value.clone()).map_err(json)?;
            validate_resident_declaration(&facts,&record,&row,&record_path,&script,manager.boot_id(),&resident_selection.profile)?;
            tokio::time::timeout_at(deadline.into(),manager.validate_resident(&facts)).await.map_err(|_|HostError::Deadline("resident original identity recheck".into()))??;
            result.push(BorrowedResidentCharge {facts,profile:resident_selection.profile.clone()});
        }
        Ok(result)
    }
    async fn reclaimed(&self,manager:&HostManager,state:&Journal,deadline:Instant)->Result<BTreeSet<String>,HostError> {
        let mut removed=BTreeSet::new();
        for (nonce,row) in &state.finite {
            if row.boot!=manager.boot_id() { removed.insert(nonce.clone()); continue; }
            if row.manager!=manager.identity_key() { continue; }
            if !row.released && caller_present(&row.caller) { continue; }
            let unbound=row.units.iter().find(|unit|unit.intent.unit()==nonce).is_none_or(|unit|unit.binding.is_none());
            if unbound && caller_present(&row.gated_child) { continue; }
            let mut drained=true;
            for unit in &row.units {
                if row.released && unit.controller { continue; }
                let result=tokio::time::timeout_at(deadline.into(),manager.original_drained(&unit.intent,unit.binding.as_ref())).await;
                if !matches!(result,Ok(Ok(true))) { drained=false; break; }
            }
            if drained { removed.insert(nonce.clone()); }
        }
        Ok(removed)
    }
    /// Reserve finite demand before dispatching its original scopes. An ordinary
    /// selection must use `admit_ordinary` after its gate is visible to the manager.
    pub async fn admit(&self,manager:&HostManager,selection:&Selection,owner:&LaunchIntent,workload:&LaunchIntent,child_pid:u32,deadline:Instant)->Result<Permit,HostError> {
        if selection.profile.ordinary() { return Err(invalid("ordinary admission requires an observed gated workload")); }
        let slice=LaunchIntent::fresh_allocation(&selection.slice)?;
        let caller=caller_identity()?;
        let gated_child=process_identity(child_pid)?;
        if gated_child.boot_id!=manager.boot_id() || gated_child.pid_namespace!=caller.pid_namespace || gated_child.pid==caller.pid {
            return Err(invalid("gated child does not have a distinct authenticated original process identity"));
        }
        let nonce=workload.unit().to_owned();
        let units=vec![RegisteredUnit {intent:owner.clone(),binding:None,controller:true},RegisteredUnit {intent:workload.clone(),binding:None,controller:false},RegisteredUnit {intent:slice.clone(),binding:None,controller:false}];
        let proposed=Reservation {boot:manager.boot_id().to_owned(),manager:manager.identity_key(),caller,gated_child,profile:selection.profile.clone(),units,released:false};
        let deadline=deadline.min(Instant::now()+selection.admission_timeout());
        let mut reason="capacity".to_owned();
        while Instant::now()<deadline {
            let snapshot=self.snapshot(deadline).await?;
            let removed=self.reclaimed(manager,&snapshot,deadline).await?;
            let finite=snapshot.finite.iter().filter(|(nonce,_)|!removed.contains(*nonce)).flat_map(|(_,row)|row.units.iter().map(|unit|unit.intent.unit().to_owned())).collect::<Vec<_>>();
            let ordinary=tokio::time::timeout_at(deadline.into(),manager.ordinary_workloads(&[owner.unit().to_owned(),workload.unit().to_owned()],&finite)).await.map_err(|_|HostError::Deadline("ordinary workload exclusion snapshot".into()))??;
            let borrowed=self.borrowed_residents(manager,selection,deadline).await?;
            if selection.profile.lane!="light" && meminfo("MemAvailable")?<checked_gib(selection.policy.pressure_guard_gib)? { reason="host MemAvailable below finite startup guard".into(); }
            else {
                let conflict=selection.conflict(snapshot.finite.iter().filter(|(nonce,_)|!removed.contains(*nonce)).map(|(_,row)|row.profile.clone())
                    .chain(borrowed.iter().filter(|resident|!resident.already_charged(&snapshot)).map(|resident|resident.profile.clone())),ordinary);
                if let Some(conflict)=conflict {
                    reason=conflict;
                    if !self.commit(snapshot.revision,&removed,None,false,deadline).await? { continue; }
                } else if self.commit(snapshot.revision,&removed,Some((&nonce,&proposed)),false,deadline).await? {
                    return Ok(Permit {coordinator:self.clone(),selection:Some(selection.clone()),profile:selection.profile.clone(),nonce:Some(nonce),slice:Some(slice),slice_owner:None,borrowed:false});
                } else { continue; }

            }
            tokio::time::sleep_until((Instant::now()+Duration::from_millis(50)).min(deadline).into()).await;
        }
        Err(HostError::Deadline(format!("finite admission: {reason}")))
    }
    /// Check live finite exclusions for an already attached, unreleased gate.
    /// Revision publication is deliberately not an ordinary reservation row.
    pub async fn admit_ordinary(&self,manager:&HostManager,workload:&UnitObservation,deadline:Instant)->Result<(),HostError> {
        let current=tokio::time::timeout_at(deadline.into(),manager.observe(&workload.unit)).await.map_err(|_|HostError::Deadline("ordinary gate identity".into()))??;
        let native=tokio::time::timeout_at(deadline.into(),manager.native_workload(&current)).await.map_err(|_|HostError::Deadline("ordinary gate role".into()))??;
        if current.invocation!=workload.invocation || current.control_group!=workload.control_group || !native {
            return Err(invalid("ordinary gate no longer matches its original native workload"));
        }
        loop {
            let snapshot=self.snapshot(deadline).await?;
            let removed=self.reclaimed(manager,&snapshot,deadline).await?;
            let conflict=snapshot.finite.iter().filter(|(nonce,_)|!removed.contains(*nonce))
                .any(|(_,row)|row.profile.exclusive || matches!(row.profile.lane.as_str(),"timing" | "store-timing"));
            if !self.commit(snapshot.revision,&removed,None,!conflict,deadline).await? { continue; }
            if conflict { return Err(invalid("ordinary admission refused by live exclusive/timing owner")); }
            return Ok(());
        }
    }
    /// Reuse only a finite reservation containing this actual original workload.
    pub async fn inherit(&self,environment:&Environment,manager:&HostManager,selection:Option<&Selection>,current:&UnitObservation,deadline:Instant)->Result<Option<Permit>,HostError> {
        let Some(marker)=value(environment,MARKER)? else { return Ok(None); };
        let actual=tokio::time::timeout_at(deadline.into(),manager.current_unit()).await.map_err(|_|HostError::Deadline("inherited finite identity".into()))??;
        if actual.unit!=current.unit || actual.invocation!=current.invocation || actual.control_group!=current.control_group { return Err(invalid("inherited workload changed during admission")); }
        let snapshot=self.snapshot(deadline).await?;
        let row=snapshot.finite.get(marker).ok_or_else(||invalid("inherited finite allocation absent"))?;
        if row.boot!=manager.boot_id() || row.manager!=manager.identity_key() || row.released { return Err(invalid("inherited finite allocation is stale or released")); }
        let registered=row.units.iter().find(|unit|!unit.controller && !unit.intent.unit().ends_with(".slice") && unit.binding.as_ref().is_some_and(|binding|binding.invocation==actual.invocation && unit.intent.unit()==actual.unit && binding.group==actual.control_group))
            .ok_or_else(||invalid("finite allocation marker does not match actual original workload membership"))?;
        let binding=registered.binding.as_ref().ok_or_else(||invalid("inherited original binding absent"))?;
        tokio::time::timeout_at(deadline.into(),manager.validate_binding(&registered.intent,binding)).await.map_err(|_|HostError::Deadline("inherited original cgroup binding".into()))??;
        if selection.is_some_and(|selection| !selection.profile.ordinary() && (selection.profile.memory>row.profile.memory || selection.profile.exclusive && !row.profile.exclusive || !selection.profile.cores.iter().all(|core|row.profile.cores.contains(core)))) {
            return Err(invalid("nested finite demand exceeds the authenticated enclosing allocation"));
        }
        let requested_bytes = if let Some(request)=value(environment,"PSE_MEMORY_MAX")? {
            Some(finite_bytes(request,meminfo("MemTotal")?)?)
        } else { selection.filter(|selection|!selection.profile.ordinary() && (selection.explicit_class || selection.profile.name!=row.profile.name)).map(|selection|selection.profile.memory) };
        if let Some(requested)=requested_bytes {
            if requested>row.profile.memory { return Err(invalid("nested memory request exceeds its admitted reservation")); }
            enclosing_ceiling(&actual.control_group,requested)?;
        }
        let requested=row.units.iter().find_map(|unit|unit.intent.unit().ends_with(".slice").then(||unit.intent.clone())).ok_or_else(||invalid("inherited allocation ancestor absent"))?;
        Ok(Some(Permit {coordinator:self.clone(),selection:selection.cloned(),profile:row.profile.clone(),nonce:Some(marker.to_owned()),slice:Some(requested),slice_owner:None,borrowed:true}))
    }
}

/// Held finite responsibility or authenticated borrowing. Drop does not release capacity.
#[must_use="finite admission requires explicit physical settlement"]
#[derive(Debug)]
pub struct Permit {
    coordinator: Coordinator,
    selection: Option<Selection>,
    profile: Profile,
    nonce: Option<String>,
    slice: Option<LaunchIntent>,
    slice_owner: Option<OwnedUnit>,
    borrowed: bool,
}
impl Permit {
    /// Whether this request reused an existing physically authenticated allocation.
    pub fn borrowed(&self)->bool { self.borrowed }
    /// Selected original finite profile, or ordinary profile without numerical charge.
    pub fn profile(&self)->&Profile { &self.profile }
    /// Place the workload and owned resident roles below this finite ancestor.
    /// The controller retains its authenticated prior slice and conservative overhead charge.
    pub fn allocation_slice(&self)->Option<&str> { self.slice.as_ref().map(LaunchIntent::unit) }
    /// Propagate only actual finite reservation identity; ordinary work adds no marker.
    pub fn apply_environment(&self,environment:&mut Environment) {
        if let Some(nonce)=self.nonce.as_ref() { environment.insert(MARKER.into(),nonce.into()); environment.insert("PSE_RESOURCE_CLASS".into(),self.profile.name.clone().into()); }
        else { environment.remove(OsStr::new(MARKER)); environment.remove(OsStr::new("PSE_RESOURCE_CLASS")); }
    }
    /// Configure the declared parent under its independent control lock and build
    /// this private reference-owned slice. No admission lock spans manager calls.
    pub async fn prepare(&mut self,manager:&HostManager,deadline:Instant)->Result<(),HostError> {
        if self.borrowed || self.slice.is_none() { return Ok(()); }
        if self.nonce.is_none() { return Err(invalid("finite allocation capability was already relinquished")); }
        if self.slice_owner.is_some() { return Err(invalid("original finite slice already prepared")); }
        let selection=self.selection.as_ref().ok_or_else(||invalid("owned finite selection absent"))?;
        let _configuration=self.coordinator.lock("parent-configuration.lock",deadline).await?;
        let proof=self.coordinator.parent_proof()?;
        let snapshot=self.coordinator.snapshot(deadline).await?;
        let known=snapshot.finite.values().filter(|row|row.manager==manager.identity_key() && row.boot==manager.boot_id())
            .flat_map(|row|row.units.iter().map(|unit|unit.intent.clone())).collect::<Vec<_>>();
        let borrowed=self.coordinator.borrowed_residents(manager,selection,deadline).await?;
        let proof=manager.ensure_finite_parent(FiniteParentRequest {
            selected:&selection.slice,aggregate:self.profile.aggregate,declared:&self.coordinator.declared_parent,proof:proof.as_ref(),known:&known,borrowed:&borrowed,
            isolated:self.profile.exclusive || matches!(self.profile.lane.as_str(),"timing" | "store-timing"),
        },deadline).await?;
        if let Some(proof)=proof { self.coordinator.write_parent_proof(&proof)?; }
        let mut controls=self.profile.controls()?;
        controls.memory_max=Some(self.profile.allocation);
        let slice=manager.start_slice(self.slice.as_ref().ok_or_else(||invalid("finite slice intent absent"))?,&controls,deadline).await?;
        self.slice_owner=Some(slice);
        self.bind(self.slice_owner.as_ref().ok_or_else(||invalid("original slice handle absent"))?,deadline).await

    }
    /// Bind original invocation and kernel identity before releasing an exec gate.
    pub async fn bind(&self,unit:&OwnedUnit,deadline:Instant)->Result<(),HostError> {
        if self.borrowed { return Err(invalid("borrowed finite capability cannot register another owner")); }
        let Some(nonce)=self.nonce.as_ref() else { return Ok(()); };
        let binding=tokio::time::timeout_at(deadline.into(),unit.binding()).await.map_err(|_|HostError::Deadline("original finite binding".into()))??;
        let _guard=self.coordinator.lock("admission.lock",deadline).await?;
        let mut current=self.coordinator.read()?;
        let row=current.finite.get_mut(nonce).ok_or_else(||invalid("finite reservation no longer live"))?;
        if row.released { return Err(invalid("finite reservation already released")); }
        let registered=row.units.iter_mut().find(|registered|registered.intent==*unit.intent()).ok_or_else(||invalid("unit was not registered before dispatch"))?;
        if registered.binding.as_ref().is_some_and(|original|original!=&binding) { return Err(invalid("registered original unit identity changed")); }
        registered.binding=Some(binding);
        current.advance()?;
        self.coordinator.write(&current)
    }
    /// Attempt every original physical settlement before deleting its finite charge.
    /// A false result retains the reservation; borrowed allocations remain owned elsewhere.
    pub async fn release(&mut self,manager:&HostManager,deadline:Instant)->Result<bool,HostError> {
        if self.borrowed || self.nonce.is_none() { return Ok(true); }
        if self.slice_owner.is_none() {
            if let Some(intent)=self.slice.as_ref() { self.slice_owner=manager.reconcile_if_present(intent,deadline).await?; }
        }
        if let Some(slice)=self.slice_owner.as_mut() { slice.stop_and_drain(deadline).await?; self.slice_owner=None; }
        let nonce=self.nonce.as_ref().ok_or_else(||invalid("finite release identity absent"))?;
        {
            let _guard=self.coordinator.lock("admission.lock",deadline).await?;
            let mut state=self.coordinator.read()?;
            let Some(row)=state.finite.get_mut(nonce) else { self.nonce=None; return Ok(true); };
            row.released=true;
            state.advance()?;
            self.coordinator.write(&state)?;
        }
        let coordinator=&self.coordinator;
        let released=coordinator.retire_released(nonce,deadline,|state|async move {coordinator.reclaimed(manager,&state,deadline).await}).await?;
        if released { self.nonce=None; }
        Ok(released)
    }
}
fn enclosing_ceiling(group:&str,requested:u64)->Result<(),HostError> {
    let relative=group.strip_prefix('/').filter(|group|!group.is_empty()).ok_or_else(||invalid("enclosing finite cgroup is unavailable"))?;
    if Path::new(relative).components().any(|component|!matches!(component,Component::Normal(_))) { return Err(invalid("invalid enclosing finite cgroup")); }
    let mut path=Path::new("/sys/fs/cgroup").join(relative);
    loop {
        if path==Path::new("/sys/fs/cgroup") { break; }
        let value=fs::read_to_string(path.join("memory.max"))?;
        if value.trim()!="max" && value.trim().parse::<u64>().ok().is_none_or(|limit|limit<requested) {
            return Err(invalid("nested demand exceeds its actual enclosing role or ancestor ceiling"));
        }
        if path==Path::new("/sys/fs/cgroup") { break; }
        if !path.pop() { return Err(invalid("enclosing finite hierarchy root absent")); }
    }
    Ok(())
}

fn private_directory(path:&Path,create:bool)->Result<(),HostError> {
    if path.components().any(|component|!matches!(component,Component::RootDir | Component::Normal(_))) { return Err(invalid("invalid private runtime path")); }
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) if meta.file_type().is_symlink()=>return Err(invalid("runtime admission cannot traverse symlinks")),
            Ok(_)=>{},
            Err(error) if create && error.kind()==io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error.into()),
        }
    }
    if create { match fs::create_dir(path) {Ok(())=>fs::set_permissions(path,fs::Permissions::from_mode(0o700))?,Err(error) if error.kind()==io::ErrorKind::AlreadyExists=>{},Err(error)=>return Err(error.into())} }
    let meta=fs::metadata(path)?;
    if !meta.is_dir() || meta.uid()!=rustix::process::geteuid().as_raw() || meta.mode() & 0o077!=0 { return Err(invalid("runtime admission directory must be user-owned mode0700")); }
    Ok(())
}
fn private_file(file:&File)->Result<(),HostError> {
    let meta=file.metadata()?;
    if !meta.is_file() || meta.uid()!=rustix::process::geteuid().as_raw() || meta.mode() & 0o077!=0 || meta.nlink()!=1 { return Err(invalid("unsafe admission state file")); }
    Ok(())
}
fn caller_identity()->Result<ProcessIdentity,HostError> { process_identity(std::process::id()) }
fn process_identity(pid:u32)->Result<ProcessIdentity,HostError> {
    match ProcessReader::default().process(pid) {
        ProcessRead::Present(facts)=>Ok(facts.identity),
        _=>Err(invalid("admission caller process identity unavailable")),
    }
}
fn caller_present(identity:&ProcessIdentity)->bool {
    match ProcessReader::default().process(identity.pid) {
        ProcessRead::Present(facts)=>facts.identity==*identity,
        ProcessRead::Absent=>false,
        _=>true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy()->Policy { toml::from_str(include_str!("../../../.config/agent-capacity.toml")).unwrap() }
    fn selected(class:Option<&str>,request:Option<&str>,mode:&str)->Selection {
        let mut env=Environment::from([("PSE_BUILD_MODE".into(),mode.into())]);
        if let Some(request)=request { env.insert("PSE_MEMORY_MAX".into(),request.into()); }
        Selection::from_policy(policy(),&env,class,Some("compile"),256*GIB).unwrap()
    }
    fn fixture()->(tempfile::TempDir,Environment,Coordinator) {
        let runtime=tempfile::tempdir().unwrap();
        fs::set_permissions(runtime.path(),fs::Permissions::from_mode(0o700)).unwrap();
        let mut environment=Environment::from([("XDG_RUNTIME_DIR".into(),runtime.path().as_os_str().into())]);
        for key in ["HOME","XDG_STATE_HOME"] { if let Some(value)=std::env::var_os(key) { environment.insert(key.into(),value); } }
        let coordinator=Coordinator::open(Path::new("/first-checkout"),&environment).unwrap();
        (runtime,environment,coordinator)
    }
    #[test]
    fn borrowed_resident_requires_original_metadata_identity_and_actual_resource_bounds() {
        let directory=tempfile::tempdir().unwrap();
        let generation="11111111111111111111111111111111";
        let nonce="22222222222222222222222222222222";
        let unit=format!("pse-compiler-cache-{generation}.service");
        let cache_root=directory.path().join("compiler-cache");
        let path=cache_root.join("source").join(generation).join("service.json");
        let script="/original/scripts/compiler_cache.py";
        let profile=selected(Some("compiler-cache"),None,"bounded").profile;
        let binding=UnitBinding {invocation:vec![1;16],group:format!("/user.slice/pse.slice/pse-allocation-{nonce}.slice/{unit}"),device:31,inode:42};
        let process=ProcessIdentity {boot_id:"original-boot".into(),pid_namespace:"pid:[1]".into(),pid:7,start_ticks:9};
        let fragment=directory.path().join(&unit);
        fs::write(&fragment,"[Unit]\nDescription=Owned PSE compiler cache\n[Service]\n").unwrap();
        fs::set_permissions(&fragment,fs::Permissions::from_mode(0o644)).unwrap();
        let facts=ResidentServiceFacts {
            observation:UnitObservation {unit:unit.clone(),invocation:binding.invocation.clone(),active_state:"active".into(),sub_state:"running".into(),main_pid:7,exit_code:0,exit_status:0,control_group:binding.group.clone()},
            binding:binding.clone(),executable:"/python".into(),arguments:vec!["/python".into(),script.into(),"--pse-cache-serve".into(),path.to_str().unwrap().into()],fragment,slice:format!("pse-allocation-{nonce}.slice"),memory_max:8*GIB,memory_swap_max:0,kernel_memory_max:8*GIB,kernel_memory_swap_max:0,process,
        };
        let record=LegacyCacheRecord {version:1,unit:unit.clone(),generation:generation.into(),record:path.clone(),allocation:BTreeMap::from([(MARKER.into(),format!("/state/host-admission/{nonce}")),("PSE_RESOURCE_CLASS".into(),"compiler-cache".into())]),startup_inputs:BTreeMap::from([(script.into(),"a".repeat(64))])};
        let row=LegacyResidentAllocation {boot:"original-boot".into(),class:"compiler-cache".into(),lane:"light".into(),memory:8*GIB,slots:1,cores:profile.cores.clone(),exclusive:false,units:BTreeMap::from([(unit.clone(),LegacyUnitBinding {group:binding.group.clone(),inode:42,invocation:"01".repeat(16)})])};
        assert!(resident_launch_record(&facts,&cache_root).is_ok());
        assert!(validate_resident_declaration(&facts,&record,&row,&path,script,"original-boot",&profile).is_ok());
        let mut changed=row.clone(); changed.slots=2;
        assert!(validate_resident_declaration(&facts,&record,&changed,&path,script,"original-boot",&profile).is_err());
        let mut changed=row.clone(); changed.units.get_mut(&unit).unwrap().invocation="02".repeat(16);
        assert!(validate_resident_declaration(&facts,&record,&changed,&path,script,"original-boot",&profile).is_err());
        let mut changed=row.clone(); changed.class="functional".into();
        assert!(validate_resident_declaration(&facts,&record,&changed,&path,script,"original-boot",&profile).is_err());
        let mut changed=facts.clone(); changed.kernel_memory_max=16*GIB;
        assert!(validate_resident_declaration(&changed,&record,&row,&path,script,"original-boot",&profile).is_err());
        let mut changed=facts.clone(); changed.memory_swap_max=u64::MAX;
        assert!(validate_resident_declaration(&changed,&record,&row,&path,script,"original-boot",&profile).is_err());
        let mut changed=facts.clone(); changed.arguments[2]="--other-server".into();
        assert!(resident_launch_record(&changed,&cache_root).is_err());
        fs::set_permissions(&facts.fragment,fs::Permissions::from_mode(0o664)).unwrap();
        assert!(resident_launch_record(&facts,&cache_root).is_err());
        let borrowed=BorrowedResidentCharge {facts,profile};
        let light=selected(Some("light"),None,"bounded");
        assert!(light.conflict(vec![light.profile.clone();2].into_iter().chain(std::iter::once(borrowed.profile.clone())),false).is_none());
        assert!(light.conflict(vec![light.profile.clone();3].into_iter().chain(std::iter::once(borrowed.profile.clone())),false).is_some());
        let functional=selected(Some("functional"),None,"bounded");
        assert!(functional.conflict(std::iter::once(borrowed.profile.clone()),false).is_none());
        assert!(!borrowed.already_charged(&Journal::default()));
    }
    #[test]
    fn ordinary_work_has_no_slots_caps_or_pressure_selection() {
        let selection=selected(None,None,"unbounded");
        assert!(selection.profile.ordinary());
        assert_eq!(selection.profile.slots,0);
        let controls=selection.profile.controls().unwrap();
        assert!(controls.memory_max.is_none());
        assert!(controls.allowed_cpus.is_empty());
        assert_eq!(selection.conflict(std::iter::empty(),true),None);
        let light_hint=Selection::from_policy(policy(),&Environment::from([("PSE_BUILD_MODE".into(),"unbounded".into())]),None,Some("light"),256*GIB).unwrap();
        assert!(light_hint.profile.ordinary());
        assert_eq!(selected(Some("light"),None,"unbounded").profile.memory,2*GIB);
        assert_eq!(selected(Some("compile"),None,"unbounded").profile.memory,24*GIB);
        assert_eq!(selected(Some("timing"),None,"unbounded").profile.lane,"timing");
    }
    #[test]
    fn finite_widening_keeps_declared_lane_charges_and_coherent_envelopes() {
        let wide=selected(Some("functional"),Some("64G"),"unbounded");
        assert_eq!((&*wide.profile.name,wide.profile.memory,wide.profile.slots),("wide",64*GIB,2));
        let exclusive=selected(Some("functional"),Some("128G"),"unbounded");
        assert_eq!((&*exclusive.profile.name,exclusive.profile.memory,exclusive.profile.slots),("exclusive",128*GIB,2));
        assert_eq!(exclusive.profile.allocation,152*GIB);
        let default=selected(Some("exclusive"),None,"unbounded");
        assert_eq!(default.profile.aggregate,168*GIB);
        assert_eq!(finite_bytes("50%",256*GIB).unwrap(),128*GIB);
        for invalid in ["", "0", "max", "-1G", "infinity", "1.5G", "1GB", "18446744073709551615T"] {
            assert!(finite_bytes(invalid,256*GIB).is_err(),"{invalid}");
        }
    }
    #[test]
    fn reference_and_observer_keep_host_and_production_meanings_distinct() {
        let reference=selected(Some("reference"),None,"bounded");
        assert_eq!((reference.profile.memory,reference.profile.allocation,reference.profile.aggregate),(140*GIB,160*GIB,160*GIB));
        let observer=selected(Some("exclusive-observer"),None,"bounded");
        let production=observer.policy.production.profiles.iter().find(|profile|profile.name=="exclusive-observer").unwrap();
        assert_eq!(observer.profile.memory,144*GIB);
        assert_eq!(production.native_worker_memory_bytes,72*GIB);
        assert_eq!((observer.profile.allocation,observer.profile.aggregate),(production.total_memory_bytes,160*GIB));
        let env=Environment::from([("PSE_MEMORY_MAX".into(),"64G".into())]);
        assert!(Selection::from_policy(policy(),&env,Some("reference"),None,256*GIB).is_err());
    }
    #[test]
    fn slots_preserve_declared_concurrency_and_scientific_exclusions() {
        let functional=selected(Some("functional"),None,"bounded");
        let rows=vec![functional.profile.clone();16];
        assert!(functional.conflict(rows.clone().into_iter(),false).is_some());
        assert!(functional.conflict(rows[..15].iter().cloned(),false).is_none());
        let ordinary=selected(None,None,"unbounded");
        assert!(ordinary.conflict(rows.into_iter(),false).is_none());
        for class in ["timing","reference","exclusive","exclusive-observer"] {
            let isolated=selected(Some(class),None,"bounded");
            assert!(isolated.conflict(std::iter::empty(),true).is_some());
            assert!(ordinary.conflict(std::iter::once(isolated.profile),false).is_some());
        }
        let light=selected(Some("light"),None,"bounded");
        let exclusive=selected(Some("exclusive"),None,"bounded");
        assert!(light.conflict(std::iter::once(exclusive.profile),true).is_none());
        let store=selected(Some("store-functional"),None,"bounded");
        assert!(store.conflict(vec![store.profile.clone();3].into_iter(),false).is_some());
    }
    #[test]
    fn finite_cpu_controls_expand_declared_physical_siblings() {
        let selection=selected(Some("compile"),None,"bounded");
        let controls=selection.profile.controls().unwrap();
        let cpus=logical_cpus(&selection.profile.cores).unwrap();
        assert_eq!(controls.cpu_quota_per_sec_usec,Some(cpus.len() as u64*1_000_000));
        assert_eq!(controls.memory_max,Some(24*GIB));
        assert_eq!(controls.memory_swap_max,Some(0));
        for cpu in cpus { assert_ne!(controls.allowed_cpus[cpu as usize/8] & (1<<(cpu%8)),0); }
    }
    #[tokio::test]
    async fn runtime_lock_is_shared_across_checkouts_and_deadline_bound() {
        let (_runtime,environment,first)=fixture();
        let second=Coordinator::open(Path::new("/second-checkout"),&environment).unwrap();
        assert_eq!(first.directory,second.directory);
        let held=first.lock("admission.lock",Instant::now()+Duration::from_secs(1)).await.unwrap();
        let conflict=second.lock("admission.lock",Instant::now()+Duration::from_millis(20)).await;
        assert!(matches!(conflict,Err(HostError::Deadline(_))));
        drop(held);
        assert!(second.lock("admission.lock",Instant::now()+Duration::from_secs(1)).await.is_ok());
    }
    #[tokio::test]
    async fn release_retries_unrelated_commits_and_rechecks_physical_receipt() {
        use std::sync::atomic::{AtomicU64,Ordering};
        let (_runtime,_environment,coordinator)=fixture();
        let intent=LaunchIntent::fresh_scope().unwrap();
        let nonce=intent.unit().to_owned();
        let caller=caller_identity().unwrap();
        let row=Reservation {boot:caller.boot_id.clone(),manager:"recorded-manager".into(),caller:caller.clone(),gated_child:caller,profile:selected(Some("light"),None,"bounded").profile,units:vec![RegisteredUnit {intent,binding:Some(UnitBinding {invocation:vec![1;16],group:String::new(),device:0,inode:0}),controller:false}],released:true};
        let deadline=Instant::now()+Duration::from_secs(1);
        assert!(coordinator.commit(0,&BTreeSet::new(),Some((&nonce,&row)),false,deadline).await.unwrap());
        let receipts=AtomicU64::new(0);
        let released=coordinator.retire_released(&nonce,deadline,|snapshot| {
            let attempt=receipts.fetch_add(1,Ordering::Relaxed);
            let coordinator=&coordinator;
            let nonce=&nonce;
            async move {
                if attempt==0 {
                    // Real ordinary commit races the recorded original physical
                    // drain receipt, invalidating only this snapshot's revision.
                    assert!(coordinator.commit(snapshot.revision,&BTreeSet::new(),None,true,deadline).await?);
                }
                Ok(BTreeSet::from([nonce.clone()]))
            }
        }).await.unwrap();
        assert!(released);
        assert_eq!(receipts.load(Ordering::Relaxed),2);
        assert!(!coordinator.snapshot(deadline).await.unwrap().finite.contains_key(&nonce));
    }
    #[tokio::test]
    async fn revision_prevents_a_stale_exclusive_snapshot_from_ignoring_ordinary_commit() {
        let (_runtime,_environment,coordinator)=fixture();
        let deadline=Instant::now()+Duration::from_secs(1);
        let snapshot=coordinator.snapshot(deadline).await.unwrap();
        assert!(coordinator.commit(snapshot.revision,&BTreeSet::new(),None,true,deadline).await.unwrap());
        // This is the same CAS used after the manager scan in finite admission.
        assert!(!coordinator.commit(snapshot.revision,&BTreeSet::new(),None,true,deadline).await.unwrap());
        let after=coordinator.snapshot(deadline).await.unwrap();
        assert_ne!(snapshot.revision,after.revision);
        assert!(after.finite.is_empty(),"ordinary commit created a reservation");
    }
    #[test]
    fn runtime_state_refuses_symlinks_public_modes_and_malformed_journals() {
        let (_runtime,_environment,coordinator)=fixture();
        fs::write(coordinator.directory.join("finite.json"),b"{broken").unwrap();
        fs::set_permissions(coordinator.directory.join("finite.json"),fs::Permissions::from_mode(0o600)).unwrap();
        assert!(coordinator.read().is_err());
        fs::remove_file(coordinator.directory.join("finite.json")).unwrap();
        let outside=tempfile::NamedTempFile::new().unwrap();
        std::os::unix::fs::symlink(outside.path(),coordinator.directory.join("finite.json")).unwrap();
        assert!(coordinator.read().is_err());
        fs::set_permissions(&coordinator.directory,fs::Permissions::from_mode(0o755)).unwrap();
        assert!(private_directory(&coordinator.directory,false).is_err());
    }
    #[tokio::test]
    async fn real_manager_predispatch_reservation_is_retained_until_explicit_settlement() {
        let (_runtime,environment,coordinator)=fixture();
        let manager=HostManager::connect_user(None).await.unwrap();
        let selection=selected(Some("light"),None,"bounded");
        let owner=LaunchIntent::fresh_scope().unwrap();
        let workload=LaunchIntent::fresh_scope().unwrap();
        let deadline=Instant::now()+Duration::from_secs(5);
        let mut child=std::process::Command::new("/bin/cat")
            .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null()).spawn().unwrap();
        let admitted=coordinator.admit(&manager,&selection,&owner,&workload,child.id(),deadline).await;
        let checked:Result<(),HostError> = async {
            let permit=admitted.as_ref().map_err(|error|invalid(error.to_string()))?;
            let peer=Coordinator::open(Path::new("/second-checkout"),&environment)?;
            let snapshot=peer.snapshot(deadline).await?;
            if snapshot.finite.len()!=1 || !peer.reclaimed(&manager,&snapshot,deadline).await?.is_empty() {
                return Err(invalid("live original gated child did not retain its reservation"));
            }
            if permit.profile().memory!=2*GIB { return Err(invalid("finite fixture charge differs")); }
            Ok(())
        }.await;
        let mut permit=admitted.ok();
        let retained=if let Some(permit)=permit.as_mut() { permit.release(&manager,deadline).await } else { Ok(false) };
        // EOF of this native pipe-blocked fixture ends the original child without
        // launching raw work. Reap before asking admission to settle its charge.
        drop(child.stdin.take());
        let reaped=child.wait();
        let released=if let Some(permit)=permit.as_mut() { permit.release(&manager,deadline).await } else { Ok(false) };
        let final_state=coordinator.snapshot(deadline).await;
        assert!(checked.is_ok(),"{checked:?}");
        assert!(matches!(retained,Ok(false)),"{retained:?}");
        assert!(reaped.is_ok(),"{reaped:?}");
        assert!(matches!(released,Ok(true)),"{released:?}");
        assert!(final_state.unwrap().finite.is_empty());
    }
    #[tokio::test]
    async fn real_populated_group_keeps_charge_after_lost_caller_and_explicit_release() {
        let (_runtime,_environment,coordinator)=fixture();
        let manager=HostManager::connect_user(None).await.unwrap();
        let current=manager.current_unit().await.unwrap();
        let group=Path::new("/sys/fs/cgroup").join(current.control_group.trim_start_matches('/'));
        let meta=fs::metadata(&group).unwrap();
        let mut caller=caller_identity().unwrap();
        caller.start_ticks=caller.start_ticks.wrapping_add(1);
        let intent=LaunchIntent::fresh_scope().unwrap();
        let nonce=intent.unit().to_owned();
        let row=Reservation {
            boot:manager.boot_id().into(),manager:manager.identity_key(),caller,gated_child:caller_identity().unwrap(),profile:selected(Some("light"),None,"bounded").profile,
            units:vec![RegisteredUnit {intent,binding:Some(UnitBinding {invocation:current.invocation,group:current.control_group,device:meta.dev(),inode:meta.ino()}),controller:false}],released:true,
        };
        let mut snapshot=Journal::default();
        snapshot.finite.insert(nonce,row);
        let removed=coordinator.reclaimed(&manager,&snapshot,Instant::now()+Duration::from_secs(5)).await.unwrap();
        assert!(removed.is_empty(),"leader/reference loss erased actual populated descendants");
    }
    #[tokio::test]
    async fn unknown_manager_generation_retains_finite_charge() {
        let (_runtime,_environment,coordinator)=fixture();
        let manager=HostManager::connect_user(None).await.unwrap();
        let row=Reservation {boot:manager.boot_id().into(),manager:"recorded-different-manager".into(),caller:caller_identity().unwrap(),gated_child:caller_identity().unwrap(),profile:selected(Some("light"),None,"bounded").profile,units:vec![RegisteredUnit {intent:LaunchIntent::fresh_scope().unwrap(),binding:None,controller:false}],released:true};
        let mut snapshot=Journal::default();
        snapshot.finite.insert("unknown-manager".into(),row);
        assert!(coordinator.reclaimed(&manager,&snapshot,Instant::now()+Duration::from_secs(5)).await.unwrap().is_empty());
    }
}
