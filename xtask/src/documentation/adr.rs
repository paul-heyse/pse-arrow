// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native decision-record commands, source preservation and historical immutability.
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::disallowed_types, reason = "xtask decision-record command surface owns terminal diagnostics and dynamic tooling errors")]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};
use clap::Subcommand;
use regex::{Captures, Regex};

use super::{headings::markdown_headings, metadata::{self, Value}};

type Fields = BTreeMap<String, Value>;
const REQUIRED_KEYS: &[&str] = &["id", "title", "status", "date", "deciders", "level", "principles", "blueprint", "review", "evidence", "supersedes", "superseded-by", "revisit", "verification"];
const STATUSES: &[&str] = &["proposed", "accepted", "rejected", "deprecated", "superseded"];
const LEVELS: &[&str] = &["decision", "should-deviation", "must-gap"];
const EVIDENCE: &[&str] = &["Proposed", "Interface-checked", "Implemented", "Tested", "Measured", "Formally established"];
const GIT_REVIEW: &str = r"^git:([0-9a-f]{12,40}):([^#\s]+)(#\S+)?$";
const PERMALINK: &str = r"\]\(https://github\.com/([\w.-]+/[\w.-]+)/blob/([0-9a-f]{12,40})/([^)#\s]+)";

/// Native ADR operations. The outer xtask owns workspace selection.
#[derive(Debug, Subcommand)]
pub enum AdrCommand {
    /// Allocate a never-reused ADR number and populate the authored template.
    New {
        /// Lowercase kebab-case filename slug.
        slug: String,
        /// Imperative decision title.
        #[arg(long)]
        title: String,
    },
    /// Validate record contracts, references and accepted-record immutability.
    Lint,
    /// Publish or compare the deterministic retained-record index.
    Index {
        /// Compare without writing.
        #[arg(long)]
        check: bool,
    },
    /// Link retained predecessor/successor records and append status history.
    Supersede {
        /// Predecessor ID or number.
        old: String,
        /// Successor ID or number.
        new: String,
    },
}

/// Accumulated ADR diagnostics. History limitations remain visible as notes.
#[derive(Debug, Default)]
pub struct LintReport {
    /// Number of retained decision files examined.
    pub records: usize,
    /// Contract violations, against the zero-error target.
    pub errors: Vec<String>,
    /// Explicit limitations of available Git history.
    pub notes: Vec<String>,
}

/// Execute one ADR operation in the explicitly selected repository.
/// # Errors
/// Invalid records, inaccessible source/history, or failed publication.
pub fn execute(root: &Path, command: AdrCommand) -> Result<()> {
    match command {
        AdrCommand::New { slug, title } => {
            let path = new_record(root, &slug, &title, &today())?;
            println!("adr new: {}", path.strip_prefix(root)?.display());
            println!("next: fill the §H fields, then run `just adr-index`");
        }
        AdrCommand::Index { check } => {
            index(root, check)?;
            println!("adr index: {}", if check { "up to date" } else { "wrote docs/adr/README.md" });
        }
        AdrCommand::Lint => {
            let report = inspect(root)?;
            for note in report.notes { println!("note: {note}"); }
            for error in &report.errors { eprintln!("error: {error}"); }
            ensure!(report.errors.is_empty(), "adr lint: {} problem(s) in {} record(s)", report.errors.len(), report.records);
            println!("adr lint: {} record(s) OK", report.records);
        }
        AdrCommand::Supersede { old, new } => {
            let (before, after) = supersede(root, &old, &new, &today())?;
            println!("adr supersede: {before} -> {after}");
            println!("next: append the reason to the new record's Status history, then run `just adr-index`");
        }
    }
    Ok(())
}

fn today() -> String { chrono::Local::now().date_naive().to_string() }

fn filename_number(name: &str) -> Option<u32> {
    let (number, tail) = name.split_once('-')?;
    let slug = tail.strip_suffix(".md")?;
    (number.len() == 4 && number.bytes().all(|b| b.is_ascii_digit()) && !slug.is_empty() && slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')).then(|| number.parse().ok()).flatten()
}
fn issued_number(id: &str) -> Option<u32> {
    let number = id.strip_prefix("ADR-")?;
    (number.len() == 4 && number.bytes().all(|b| b.is_ascii_digit())).then(||number.parse().ok()).flatten()
}
fn adr_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let dir = root.join("docs/adr");
    if !dir.is_dir() { return Ok(paths); }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|s|s == "md") && !["README.md", "template.md", "register.md"].iter().any(|name|path.file_name().is_some_and(|s|s == *name)) { paths.push(path); }
    }
    paths.sort(); Ok(paths)
}
fn text_value(value: &Value) -> String {
    match value {
        Value::Null => "None".to_owned(), Value::Bool(true) => "True".to_owned(), Value::Bool(false) => "False".to_owned(),
        Value::Integer(v) => v.to_string(), Value::Unsigned(v) => v.to_string(), Value::Float(v) => v.to_string(), Value::String(v) => v.clone(),
        Value::Sequence(v) => format!("[{}]", v.iter().map(|v|match v { Value::String(s)=>format!("{s:?}"), _=>text_value(v) }).collect::<Vec<_>>().join(", ")),
        Value::Mapping(v) => format!("{{{}}}", v.iter().map(|(key,v)|format!("{key:?}: {}",text_value(v))).collect::<Vec<_>>().join(", ")),
    }
}
fn field(front: &Fields, key: &str) -> String { front.get(key).map_or_else(String::new, text_value) }
fn nonnull_field(front: &Fields, key: &str) -> String { front.get(key).filter(|v|!matches!(v,Value::Null)).map_or_else(String::new,text_value) }
fn as_list(value: Option<&Value>) -> Vec<String> { match value { None | Some(Value::Null)=>Vec::new(), Some(Value::Sequence(values))=>values.iter().map(text_value).collect(), Some(value)=>vec![text_value(value)] } }
fn record(root: &Path, path: &Path) -> Result<(Fields, String)> {
    let raw = fs::read(path)?; decode(root,path,&raw)
}
fn decode(root: &Path, path: &Path, raw: &[u8]) -> Result<(Fields,String)> {
    let relative = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
    let document = metadata::read(raw,&relative,true,Some(&relative),root)?;
    Ok((document.metadata,std::str::from_utf8(document.body)?.to_owned()))
}
fn walk_markdown(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir(){return Ok(());}
    for entry in fs::read_dir(dir)? { let entry=entry?; let path=entry.path(); if entry.file_type()?.is_dir(){walk_markdown(&path,paths)?;} else if path.extension().is_some_and(|ext|ext=="md"){paths.push(path);} }
    Ok(())
}
/// Resolve numbered section headings to their sole source owner and raw heading.
/// # Errors
/// Inaccessible sources or duplicate section identities.
pub fn section_owners(blueprint: &Path) -> Result<BTreeMap<String,(PathBuf,String)>> {
    section_owners_with_reader(blueprint, &|path| Ok(fs::read_to_string(path)?))
}
/// Resolve numbered section owners using the caller's one captured source cut.
/// Directory discovery supplies names only; interpretation reads exclusively through `reader`.
/// # Errors
/// Inaccessible sources or duplicate section identities.
pub fn section_owners_with_reader(blueprint: &Path, reader: &dyn Fn(&Path) -> Result<String>) -> Result<BTreeMap<String,(PathBuf,String)>> {
    let mut paths=vec![blueprint.to_owned()];
    let mut sections=Vec::new(); walk_markdown(&blueprint.parent().context("blueprint parent absent")?.join("sections"),&mut sections)?; sections.sort(); paths.extend(sections);
    section_owners_from_sources(&paths,reader)
}
/// Resolve the same stable section vocabulary from an explicit captured membership.
/// # Errors
/// Unavailable captured bytes or duplicate stable identities.
pub fn section_owners_from_sources(paths:&[PathBuf],reader:&dyn Fn(&Path)->Result<String>)->Result<BTreeMap<String,(PathBuf,String)>>{
    let pattern=Regex::new(r"^#{2,4}\s+(?:([0-9]+(?:\.[0-9]+)*)|([A-Z][0-9]+))\.?\s")?;
    let mut owners:BTreeMap<String,(PathBuf,String)>=BTreeMap::new();
    for path in paths { let text=reader(&path)?; for heading in markdown_headings(&text) { if let Some(captures)=pattern.captures(heading.source) { let id=captures.get(1).or_else(||captures.get(2)).context("section capture absent")?.as_str().to_owned(); if let Some((previous,_))=owners.get(&id){bail!("duplicate section §{id}: {} and {}",previous.display(),path.display());} owners.insert(id,(path.clone(),heading.source.to_owned())); } } }
    Ok(owners)
}
fn git(root: &Path, args: &[&str]) -> Result<(bool,String)> {
    let output=Command::new("git").arg("-C").arg(root).args(args).output().context("run Git for ADR history")?;
    Ok((output.status.success(),String::from_utf8(output.stdout)?))
}
fn origin_main_exists(root: &Path) -> Result<bool> { Ok(git(root,&["rev-parse","--verify","--quiet","origin/main"])?.0) }
fn issued_history(root: &Path, reference: &str) -> Result<Option<BTreeSet<u32>>> {
    if git(root,&["rev-parse","--is-shallow-repository"])?.1.trim()=="true" {return Ok(None);}
    let (ok,listing)=git(root,&["log",reference,"--no-renames","--diff-filter=A","--name-only","--format=","--","docs/adr/"])?;
    if !ok{return Ok(None);}
    Ok(Some(listing.lines().filter_map(|line|Path::new(line).file_name()?.to_str().and_then(filename_number)).collect()))
}
#[derive(PartialEq, Eq)]
enum ObjectState { Present, Absent, Unknown }
fn object_state(root: &Path, commit: &str, path: &str) -> Result<ObjectState> {
    if git(root,&["cat-file","-e",&format!("{commit}:{path}")])?.0 { return Ok(if git(root,&["merge-base","--is-ancestor",commit,"HEAD"])?.0 {ObjectState::Present}else{ObjectState::Absent}); }
    if git(root,&["rev-parse","--is-shallow-repository"])?.1.trim()=="true" && !git(root,&["cat-file","-e",&format!("{commit}^{{commit}}")])?.0 {return Ok(ObjectState::Unknown);}
    Ok(ObjectState::Absent)
}
fn same_as_baseline(root: &Path, commit: &str, path: &str) -> Result<bool> {
    let (ok,baseline)=git(root,&["rev-parse","--verify","--quiet",&format!("origin/main:{path}")])?;
    Ok(!ok || git(root,&["rev-parse","--verify","--quiet",&format!("{commit}:{path}")])?.1==baseline)
}
fn repository_slug(root: &Path) -> Result<Option<String>> {
    let url=git(root,&["remote","get-url","origin"])?.1;
    Ok(Regex::new(r"github\.com[:/]([\w.-]+/[\w.-]+?)(?:\.git)?$")?.captures(url.trim()).and_then(|captures|captures.get(1).map(|value|value.as_str().to_owned())))
}
fn review_error(root: &Path, review: &str, notes: &mut Vec<String>) -> Result<Option<String>> {
    if review.starts_with("not-required:"){return Ok(None);}
    if let Some(captures)=Regex::new(GIT_REVIEW)?.captures(review) {
        return Ok(match object_state(root,&captures[1],&captures[2])? { ObjectState::Unknown=>{notes.push(format!("history unavailable; checked the shape of {review:?} only"));None}, ObjectState::Absent=>Some(format!("review {review:?} names no file at that commit reachable from HEAD")), ObjectState::Present=>None });
    }
    if review.starts_with("git:"){return Ok(Some(format!("review {review:?} must be git:<12-40 hex commit>:<path>[#anchor]")));}
    if !root.join(review.split('#').next().unwrap_or(review)).is_file(){return Ok(Some(format!("review path {review:?} does not exist; cite a retained review or its immutable git:<commit>:<path> source")));}
    Ok(None)
}
fn relocated_review(root: &Path, before: &str, after: &str) -> Result<bool> {
    let pattern=Regex::new(GIT_REVIEW)?; let Some(captures)=pattern.captures(after) else{return Ok(false);};
    let path=&captures[2]; let same=format!("{path}{}",captures.get(3).map_or("",|v|v.as_str()))==before;
    Ok(same && !root.join(path).exists() && object_state(root,&captures[1],path)?!=ObjectState::Absent && same_as_baseline(root,&captures[1],path)?)
}
fn relocated_scenarios(root: &Path, before: Option<&Value>, after: Option<&Value>) -> Result<bool> {
    let before=as_list(before);let after=as_list(after); if before.len()!=after.len(){return Ok(false);}
    let slug=repository_slug(root)?;let pattern=Regex::new(r"^https://github\.com/([\w.-]+/[\w.-]+)/blob/([0-9a-f]{12,40})/([^#\s]+)(#\S+)?$")?;
    for (before,after) in before.iter().zip(&after) {
        if before==after {continue;}
        let Some(captures)=pattern.captures(after) else{return Ok(false);};let path=&captures[3];
        if slug.as_deref().is_some_and(|slug|slug!=&captures[1]) || before!=&format!("{path}{}",captures.get(4).map_or("",|v|v.as_str())) || root.join(path).exists() || object_state(root,&captures[2],path)?==ObjectState::Absent || !same_as_baseline(root,&captures[2],path)? {return Ok(false);}
    }
    Ok(true)
}
fn relative_path(target: &Path, base: &Path) -> String {
    let target:Vec<_>=target.components().collect();let base:Vec<_>=base.components().collect();
    let shared=target.iter().zip(&base).take_while(|(a,b)|a==b).count();
    let mut path=PathBuf::new();for _ in &base[shared..]{path.push("..");}for component in &target[shared..]{path.push(component.as_os_str());}path.to_string_lossy().replace('\\',"/")
}
fn unrelocate_links(root: &Path, text: &str, record_dir: &Path, already: &str) -> Result<(String,Vec<String>)> {
    let slug=repository_slug(root)?;let pattern=Regex::new(PERMALINK)?;let mut errors=Vec::new();let mut failure=None;
    let restored=pattern.replace_all(text,|captures:&Captures<'_>|{
        let repository=&captures[1];let commit=&captures[2];let path=&captures[3];let where_=format!("{repository}@{}:{path}",&commit[..12]);
        let check=(||->Result<()>{
            if already.contains(&captures[0]){return Ok(());}
            if slug.as_deref().is_some_and(|slug|slug!=repository){errors.push(format!("{where_} is not this repository"));}
            else if root.join(path).exists(){errors.push(format!("{where_} still exists locally; link it directly"));}
            else if object_state(root,commit,path)?==ObjectState::Absent {errors.push(format!("{where_} names no file at a commit reachable from HEAD"));}
            else if !same_as_baseline(root,commit,path)?{errors.push(format!("{where_} differs from the file the record cited"));} Ok(())
        })();
        if let Err(error)=check {failure=Some(error);}
        format!("]({}",relative_path(&root.join(path),record_dir))
    }).into_owned();
    if let Some(error)=failure{return Err(error);}Ok((restored,errors))
}
fn body_sections(body: &str) -> BTreeMap<String,String> {
    let mut sections:BTreeMap<String,Vec<&str>>=BTreeMap::from([(String::new(),Vec::new())]);let mut current=String::new();
    for line in body.lines(){if let Some(title)=line.strip_prefix("## "){current=title.trim().to_owned();sections.entry(current.clone()).or_default();}else{sections.entry(current.clone()).or_default().push(line);}}
    sections.into_iter().map(|(key,lines)|(key,lines.join("\n").trim().to_owned())).collect()
}

fn supersession_errors(records: &BTreeMap<String,Fields>, issued: Option<&BTreeSet<u32>>) -> Vec<String> {
    let mut errors=Vec::new();let highest=records.keys().filter_map(|id|issued_number(id)).max().unwrap_or(0);
    for (id,front) in records {
        for other in as_list(front.get("supersedes")) {
            if let Some(peer)=records.get(&other) {
                if nonnull_field(peer,"superseded-by")!=*id {errors.push(format!("{id}: supersedes {other}, which does not name it in superseded-by"));}
            } else {
                let retired=issued_number(&other).is_some_and(|number|issued.map_or(number<highest,|issued|issued.contains(&number)));
                if !retired {errors.push(format!("{id}: supersedes unknown record {other}"));}
            }
        }
        let by=nonnull_field(front,"superseded-by");if by.is_empty(){continue;}
        match records.get(&by) {
            None=>errors.push(format!("{id}: superseded-by {by}, which is not retained; retire the superseded record too")),
            Some(peer) if !as_list(peer.get("supersedes")).contains(id)=>errors.push(format!("{id}: superseded-by {by}, which does not name it in supersedes")),
            Some(_) if field(front,"status")!="superseded"=>errors.push(format!("{id}: has superseded-by but status is {:?}",field(front,"status"))),
            Some(_)=>{}
        }
    }
    errors
}
fn lint_highest_issued(root: &Path, paths: &[PathBuf], notes: &mut Vec<String>) -> Result<Vec<String>> {
    if !origin_main_exists(root)? {notes.push("origin/main does not exist; skipping the highest-issued check".to_owned());return Ok(Vec::new());}
    let Some(issued)=issued_history(root,"origin/main")? else {notes.push("ADR history unavailable; skipping the highest-issued check".to_owned());return Ok(Vec::new());};
    let current=paths.iter().filter_map(|path|path.file_name()?.to_str().and_then(filename_number)).max().unwrap_or(0);
    Ok(issued.last().filter(|highest|current<**highest).map(|highest|format!("ADR-{highest:04} is the highest issued record; keep it until a newer record exists so allocation cannot reuse its number")).into_iter().collect())
}
fn lint_immutability(root: &Path, paths: &[PathBuf], notes: &mut Vec<String>) -> Result<Vec<String>> {
    if !origin_main_exists(root)? {notes.push("origin/main does not exist yet; skipping the ADR immutability check".to_owned());return Ok(Vec::new());}
    let mut errors=Vec::new();
    for path in paths {
        let relative=path.strip_prefix(root)?.to_string_lossy().replace('\\',"/");
        let (ok,old)=git(root,&["show",&format!("origin/main:{relative}")])?;if !ok{continue;}
        let parsed=(||->Result<_>{Ok((decode(root,path,old.as_bytes())?,record(root,path)?))})();
        let ((before,old_body),(after,new_body))=match parsed {Ok(value)=>value,Err(error)=>{errors.push(error.to_string());continue;}};
        if field(&before,"status")=="proposed" {continue;}
        let keys:BTreeSet<_>=before.keys().chain(after.keys()).collect();
        for key in keys {
            if ["status","superseded-by"].contains(&key.as_str()){continue;}
            if key=="review" && relocated_review(root,&nonnull_field(&before,key),&nonnull_field(&after,key))? {continue;}
            if key=="scenarios" && relocated_scenarios(root,before.get(key),after.get(key))? {continue;}
            if before.get(key)!=after.get(key) {errors.push(format!("{relative}: {key} changed on an accepted record; supersede it instead"));}
        }
        let old_sections=body_sections(&old_body);let new_sections=body_sections(&new_body);
        let names:BTreeSet<_>=old_sections.keys().chain(new_sections.keys()).collect();
        for name in names {
            if old_sections.get(name)==new_sections.get(name){continue;}
            let before=old_sections.get(name).map_or("",String::as_str);
            let after=new_sections.get(name).map_or("",String::as_str);
            let (relocated,problems)=unrelocate_links(root,after,path.parent().context("record parent absent")?,before)?;
            let (original,_)=unrelocate_links(root,before,path.parent().context("record parent absent")?,before)?;
            errors.extend(problems.into_iter().map(|problem|format!("{relative}: relocated link {problem}")));
            if name=="Status history" {if !relocated.starts_with(&original){errors.push(format!("{relative}: Status history is append-only"));}}
            else if original!=relocated {let label=if name.is_empty(){"(preamble)"}else{name};errors.push(format!("{relative}: section '{label}' changed on an accepted record; supersede it instead"));}
        }
    }
    Ok(errors)
}

/// Inspect the complete retained ADR corpus without publishing or editing sources.
/// # Errors
/// Filesystem/Git failures or inaccessible section-owner declarations.
pub fn inspect(root: &Path) -> Result<LintReport> {
    let paths=adr_paths(root)?;let mut report=LintReport{records:paths.len(),..LintReport::default()};
    if paths.is_empty(){report.errors.push("adr: no records under docs/adr/".to_owned());return Ok(report);}
    let sections=section_owners(&root.join("docs/authoritative_design/blueprint.md"))?;
    let principle=Regex::new(r"^(AP|DP|PS|DM)-[0-9]{2}$")?;
    let citation=Regex::new(r"^§([0-9]+(?:\.[0-9]+)*|[A-Z][0-9]+)$")?;
    let date=Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$")?;
    let mut records=BTreeMap::new();let mut numbers=BTreeMap::<u32,usize>::new();
    for path in &paths {
        let relative=path.strip_prefix(root)?.to_string_lossy().replace('\\',"/");
        let Some(number)=path.file_name().and_then(|name|name.to_str()).and_then(filename_number) else {report.errors.push(format!("{relative}: filename must match ^\\d{{4}}-[a-z0-9-]+\\.md$"));continue;};
        *numbers.entry(number).or_default()+=1;
        let (front,_)=match record(root,path){Ok(value)=>value,Err(error)=>{report.errors.push(error.to_string());continue;}};
        let missing:Vec<_>=REQUIRED_KEYS.iter().filter(|key|!front.contains_key(**key)).copied().collect();
        if !missing.is_empty(){report.errors.push(format!("{relative}: missing front-matter keys {}",missing.join(", ")));}
        let id=field(&front,"id");match issued_number(&id) {
            None=>report.errors.push(format!("{relative}: id must be ADR-NNNN, found {id:?}")),
            Some(actual) if actual!=number=>report.errors.push(format!("{relative}: id {id} does not match filename number {number:04}")),
            Some(_)=>{records.insert(id,front.clone());}
        }
        for (key,allowed) in [("status",STATUSES),("level",LEVELS)] {let value=field(&front,key);if !allowed.contains(&value.as_str()){report.errors.push(format!("{relative}: {key} {value:?} not in {allowed:?}"));}}
        if !EVIDENCE.contains(&field(&front,"evidence").as_str()){report.errors.push(format!("{relative}: evidence {:?} is not a design principles §D label",field(&front,"evidence")));}
        if !date.is_match(&field(&front,"date")){report.errors.push(format!("{relative}: date must be YYYY-MM-DD"));}
        for value in as_list(front.get("principles")){if !principle.is_match(&value){report.errors.push(format!("{relative}: principle {value:?} must match AP-NN, DP-NN, PS-NN or legacy DM-NN"));}}
        for value in as_list(front.get("blueprint")) {
            if let Some(captures)=citation.captures(&value){if !sections.contains_key(&captures[1]){report.errors.push(format!("{relative}: blueprint citation {value} resolves to no heading in blueprint.md"));}}
            else {report.errors.push(format!("{relative}: blueprint citation {value:?} must look like §14.3 or §D1"));}
        }
        if let Some(problem)=review_error(root,&nonnull_field(&front,"review"),&mut report.notes)?{report.errors.push(format!("{relative}: {problem}"));}
        for key in ["revisit","verification"]{if nonnull_field(&front,key).trim().is_empty(){report.errors.push(format!("{relative}: {key} must state something observable"));}}
    }
    let duplicates:Vec<_>=numbers.into_iter().filter_map(|(number,count)|(count>1).then_some(number)).collect();
    if !duplicates.is_empty(){report.errors.push(format!("adr numbers are issued more than once: {duplicates:?}"));}
    report.errors.extend(supersession_errors(&records,issued_history(root,"HEAD")?.as_ref()));
    report.errors.extend(lint_immutability(root,&paths,&mut report.notes)?);
    report.errors.extend(lint_highest_issued(root,&paths,&mut report.notes)?);
    match render_index(root) {
        Ok(expected)=>{let readme=root.join("docs/adr/README.md");if !readme.is_file(){report.errors.push("docs/adr/README.md is missing; run `just adr-index`".to_owned());}else if fs::read_to_string(readme)?!=expected{report.errors.push("docs/adr/README.md is stale; run `just adr-index`".to_owned());}},
        Err(error)=>report.errors.push(format!("ADR index cannot be rendered: {error}")),
    }
    Ok(report)
}

/// Render the sole retained-record index deterministically from strict metadata.
/// # Errors
/// Invalid record metadata or inaccessible files.
pub fn render_index(root: &Path) -> Result<String> {
    let mut rows=Vec::new();
    for path in adr_paths(root)? {
        let (front,_)=record(root,&path)?;
        let display=|key|front.get(key).map_or_else(||"None".to_owned(),text_value);
        let blueprint=as_list(front.get("blueprint")).join(", ");let blueprint=if blueprint.is_empty(){"—"}else{&blueprint};
        rows.push(format!("| [{}]({}) | {} | {} | {} | {blueprint} | {} |",display("id"),path.file_name().context("record filename absent")?.to_string_lossy(),display("title"),display("status"),display("date"),display("revisit")));
    }
    Ok(format!("{README_PREAMBLE}\n| ID | Title | Status | Date | Blueprint | Revisit |\n|---|---|---|---|---|---|\n{}\n",rows.join("\n")))
}
/// Publish the ADR index, or verify it without any write in check mode.
/// # Errors
/// Stale index, invalid source or failed publication.
pub fn index(root: &Path, check: bool) -> Result<()> {
    let expected=render_index(root)?;let path=root.join("docs/adr/README.md");
    if check {ensure!(path.is_file() && fs::read_to_string(&path)?==expected,"stale generated file(s): docs/adr/README.md");}else{fs::write(path,expected)?;}
    Ok(())
}
/// Allocate above all retained and historically issued numbers; never reuse a retired ID.
/// # Errors
/// Filesystem or Git failures.
pub fn next_number(root: &Path) -> Result<u32> {
    let retained=adr_paths(root)?.into_iter().filter_map(|path|path.file_name().and_then(|v|v.to_str()).and_then(filename_number)).max().unwrap_or(0);
    Ok(retained.max(issued_history(root,"HEAD")?.and_then(|issued|issued.last().copied()).unwrap_or(0))+1)
}
fn new_record(root: &Path, slug: &str, title: &str, date: &str) -> Result<PathBuf> {
    ensure!(!slug.is_empty() && slug.bytes().all(|byte|byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte==b'-'),"slug {slug:?} must be lowercase kebab-case");
    let number=next_number(root)?;ensure!(number<=9999,"ADR four-digit number space exhausted");
    let path=root.join(format!("docs/adr/{number:04}-{slug}.md"));
    let text=fs::read_to_string(root.join("docs/adr/template.md"))?.replace("ADR-NNNN",&format!("ADR-{number:04}")).replace("TITLE",title).replace("YYYY-MM-DD",date);
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&path)?;file.write_all(text.as_bytes())?;Ok(path)
}
fn resolve(root: &Path, reference: &str) -> Result<PathBuf> {
    let digits:String=reference.chars().filter(char::is_ascii_digit).collect();let digits=&digits[digits.len().saturating_sub(4)..];let padded=format!("{digits:0>4}");
    adr_paths(root)?.into_iter().find(|path|path.file_name().is_some_and(|name|name.to_string_lossy().starts_with(&padded))).with_context(||format!("no record matching {reference:?}"))
}
fn replace_field(root: &Path, path: &Path, raw: &[u8], key: &str, value: &str) -> Result<Vec<u8>> {
    let relative=path.strip_prefix(root)?.to_string_lossy().replace('\\',"/");let document=metadata::read(raw,&relative,true,Some(&relative),root)?;
    ensure!(document.metadata.contains_key(key),"{} has no {key}: field",path.display());
    let prefix=format!("{key}:");let mut output=Vec::new();let mut found=false;
    for line in document.prefix.split_inclusive(|byte|*byte==b'\n') {
        if !found && line.starts_with(prefix.as_bytes()) {output.extend_from_slice(format!("{key}: {value}").as_bytes());output.extend_from_slice(if line.ends_with(b"\r\n"){b"\r\n"}else{b"\n"});found=true;}else{output.extend_from_slice(line);}
    }
    ensure!(found,"{} has no {key}: line",path.display());output.extend_from_slice(document.body);Ok(output)
}
fn supersede(root: &Path, before: &str, after: &str, date: &str) -> Result<(String,String)> {
    let old=resolve(root,before)?;let new=resolve(root,after)?;ensure!(old!=new,"an ADR cannot supersede itself");
    let old_raw=fs::read(&old)?;let new_raw=fs::read(&new)?;let (old_front,_)=decode(root,&old,&old_raw)?;let (new_front,_)=decode(root,&new,&new_raw)?;
    let old_id=field(&old_front,"id");let new_id=field(&new_front,"id");ensure!(issued_number(&old_id).is_some() && issued_number(&new_id).is_some(),"supersession needs valid ADR identities");
    let revised=replace_field(root,&old,&old_raw,"status","superseded")?;
    let mut revised=replace_field(root,&old,&revised,"superseded-by",&new_id)?;
    let mut predecessors:BTreeSet<_>=as_list(new_front.get("supersedes")).into_iter().collect();predecessors.insert(old_id.clone());
    let successor=replace_field(root,&new,&new_raw,"supersedes",&format!("[{}]",predecessors.into_iter().collect::<Vec<_>>().join(", ")))?;
    revised.extend_from_slice(format!("- {date} — superseded by {new_id}.\n").as_bytes());
    // Both records and all controlled edits have passed admission before publication.
    fs::write(&old,revised)?;fs::write(&new,successor)?;Ok((old_id,new_id))
}

#[cfg(test)]
#[path = "adr_tests.rs"]
mod tests;

const README_PREAMBLE: &str = r#"# Decision records

<!-- @generated by xtask adr index; do not edit -->

Each record states one decision, the architecture sections it governs, the design
principles §H fields (principles, level, evidence, revisit trigger, verification), and the
design-review finding that motivated it. A record is immutable once accepted:
it changes only by superseding it with a new record, or by a status edit and an
appended `## Status history` line. `docs/authoritative_design/README.md` routes to
the authoritative design collection; these records say *why* it reads the way it does.

This index lists the retained decisions. Records whose rationale no longer explains the
current system were retired under ADR-0096 and remain in Git history; numbers are never
reused, so gaps are expected.

Run `just adr-new <slug>` to start one and `just adr-lint` before opening the PR.
Open items that were deliberately deferred live in [`register.md`](register.md).
"#;
