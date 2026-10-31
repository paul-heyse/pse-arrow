// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicitly prepared immutable HIR/IDE snapshots, isolated behind plain owner documents.
use super::{
    AnalysisContext, Backend, Configuration, Error, Kind, Query, Request, Result, Section,
    WorkspaceAnswer, WorkspaceQuery, context::Context,
};
use crate::source_command::batch::{CpuTask, spawn_cpu};
use native_ra_syntax::{
    AstNode as _,
    ast::{self, HasName as _},
};
use ra_ap_hir::{HirDisplay as _, Semantics};
use ra_ap_ide_db::base_db::SourceDatabase as _;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;

const BACKEND: &str = "ra_ap_hir/0.0.358;ra_ap_ide/0.0.358;immutable_snapshot";

/// A genuine native type observation; display is presentation, not serialized type identity.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TypeFact {
    /// Native HIR rendering, never parsed back into a type.
    pub display: String,
    /// Native reference classification.
    pub is_reference: bool,
    /// Native mutable-reference classification.
    pub is_mutable_reference: bool,
    /// Native unknown component, preventing an invented proof.
    pub contains_unknown: bool,
}
/// Source/configuration-qualified target, not a spelling-based symbol identifier.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct TargetFact {
    /// Native relationship, keeping a selected declaration distinct from a resolved call.
    pub relationship: String,
    /// Native selected declaration name.
    pub name: String,
    /// Authored exact location when the native mapping establishes one.
    pub location: Option<NativeLocation>,
    /// Mapping limitations remain explicit for macro/generated origins.
    pub precision: String,
}
/// One native original byte range.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct NativeLocation {
    /// Exact source file in the retained native world.
    pub path: PathBuf,
    /// Half-open original byte interval.
    pub start: usize,
    /// Half-open original byte interval.
    pub end: usize,
}
/// One reference and its library-owned roles; absence of a role is not runtime reachability.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ReferenceFact {
    /// Native result group; ambiguous entities are not silently merged.
    pub entity: usize,
    /// Native original range.
    pub location: NativeLocation,
    /// Native READ category.
    pub read: bool,
    /// Native WRITE category.
    pub write: bool,
}
/// A declared generic bound, distinct from arbitrary trait obligation proof.
#[derive(Debug, Serialize, JsonSchema)]
pub struct BoundFact {
    /// The actual generic parameter name.
    pub parameter: String,
    /// Actual native trait declaration names; identities remain in this snapshot context.
    pub declared_traits: Vec<String>,
}
/// Compact typed facts from one selected native expression and context.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TypedFacts {
    /// Actual expression whose type was queried, mapped to original source when unique.
    pub expression: Option<NativeLocation>,
    /// Original inferred type where the native provider supplies one.
    pub original: Option<TypeFact>,
    /// Adjusted type where the native provider performed an adjustment.
    pub adjusted: Option<TypeFact>,
    /// Actual native type equality, never display-string equality.
    pub original_equals_adjusted: Option<bool>,
    /// Actual native coercion query for the observed original/adjusted pair.
    pub original_coerces_to_adjusted: Option<bool>,
    /// Resolved explicit or implicit method target where supported.
    pub target: Option<TargetFact>,
    /// Native declared generic context.
    pub bounds: Vec<BoundFact>,
}
/// A typed native snapshot answer, not rustc acceptance or a current live-world claim.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NativeAnswer {
    /// Actual provider and preparation context.
    pub session: AnalysisContext,
    /// Exact selected working source, verified against preparation.
    pub source: crate::source_read::SourceIdentity,
    /// Native file URI.
    pub source_uri: String,
    /// Exact original byte position.
    pub source_byte_offset: usize,
    /// Immutable snapshot generation; original session invocation remains part of identity.
    pub generation: u64,
    /// Native facts where requested/supported.
    pub facts: Option<TypedFacts>,
    /// Native reference roles in the selected world.
    pub references: Vec<ReferenceFact>,
    /// Positive/unknown observations, not scientific success.
    pub availability: String,
    /// Opaque-output consumers retain ordinary sections without changing their authority.
    pub sections: Vec<Section>,
    /// Captured immutable bytes for every native linked location in this answer.
    pub location_sources: Vec<super::LocationSource>,
    /// Interpretation and supported-preparation limits.
    pub limitations: Vec<String>,
}

struct World {
    db: ra_ap_ide_db::RootDatabase,
    files: BTreeMap<PathBuf, ra_ap_vfs::FileId>,
    paths: BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    digests: BTreeMap<PathBuf, String>,
}
/// Native workers clone the immutable database, never share `Semantics` across threads.
pub(super) struct Engine {
    world: Arc<Mutex<World>>,
    context: Arc<Context>,
    configuration: Configuration,
    active: Arc<AtomicUsize>,
}
impl Engine {
    pub(super) fn start(checkout: &Path, configuration: Configuration) -> Result<Self> {
        let configuration = configuration.resolve()?;
        if configuration.backend != Backend::NativeSnapshot {
            return Err(Error::Unavailable(
                "native helper requires an explicitly selected native_snapshot backend".into(),
            ));
        }
        let mut context = Context::capture(checkout, &configuration)?;
        let cargo = ra_ap_project_model::CargoConfig {
            features: ra_ap_project_model::CargoFeatures::Selected {
                features: configuration.features.clone(),
                no_default_features: configuration.no_default_features,
            },
            target: configuration.target.clone(),
            metadata_extra_args: vec!["--locked".into()],
            ..Default::default()
        };
        let loading = ra_ap_load_cargo::LoadCargoConfig {
            load_out_dirs_from_check: false,
            with_proc_macro_server: ra_ap_load_cargo::ProcMacroServerChoice::None,
            prefill_caches: false,
            num_worker_threads: std::thread::available_parallelism()?.get(),
            proc_macro_processes: 0,
        };
        let (db, vfs, _) =
            ra_ap_load_cargo::load_workspace_at(&configuration.project, &cargo, &loading, &|_| {})
                .map_err(|error| {
                    Error::Unavailable(format!("native Cargo/VFS loading: {error:#}"))
                })?;
        let mut files = BTreeMap::new();
        let mut paths = BTreeMap::new();
        let mut digests = BTreeMap::new();
        for (id, path) in vfs.iter() {
            let Some(path) = path.as_path() else {
                continue;
            };
            let path = PathBuf::from(path.as_str());
            if !path.is_file() {
                continue;
            }
            let text = db.file_text(id).text(&db);
            let digest = crate::source_read::source_digest(text);
            if super::engine::file_digest(&path)?.as_deref() != Some(digest.as_str()) {
                return Err(Error::Stale);
            }
            files.insert(path.clone(), id);
            paths.insert(id, path.clone());
            digests.insert(path, digest);
        }
        context.bind_snapshot_sources(&digests.keys().cloned().collect::<BTreeSet<_>>())?;
        context.seal()?;
        context.check()?;
        Ok(Self {
            world: Arc::new(Mutex::new(World {
                db,
                files,
                paths,
                digests,
            })),
            context: Arc::new(context),
            configuration,
            active: Arc::new(AtomicUsize::new(0)),
        })
    }
    fn analysis_context(&self) -> AnalysisContext {
        AnalysisContext {backend:BACKEND.into(),configuration:self.configuration.clone(),position_encoding:"utf-8".into(),backend_status:Some(json!({"quiescent":true,"immutable":true})),preparation:"explicit immutable Cargo/VFS snapshot; checking/build scripts/proc macros disabled; source/configuration changes require stop and explicit reprepare".into()}
    }
    pub(super) fn status(&self) -> Result<super::SessionStatus> {
        let current = self.context.check();
        Ok(super::SessionStatus {
            configuration: self.configuration.clone(),
            backend: BACKEND.into(),
            position_encoding: "utf-8".into(),
            generation: 1,
            server_capabilities: json!({"nativeSnapshot":true,"typedFacts":true,"referenceRoles":true,"workspaceSymbolProvider":true,"sourceApplication":false,"procMacros":false}),
            backend_status: Some(json!({"quiescent":true,"immutable":true})),
            active_requests: self.active.load(Ordering::SeqCst),
            context_current: current.is_ok(),
            context_issue: current.err().map(|error| error.to_string()),
            automatic_reload_disabled: true,
            preparation_issue: None,
            preparation: self.analysis_context().preparation,
        })
    }
    pub(super) fn start_query(
        &self,
        query: Query,
        token: CancellationToken,
    ) -> CpuTask<Result<Value>> {
        let world = self.world.clone();
        let context = self.context.clone();
        let session = self.analysis_context();
        let active = self.active.clone();
        spawn_cpu(token, move |token| {
            struct Active(Arc<AtomicUsize>);
            impl Drop for Active {
                fn drop(&mut self) {
                    self.0.fetch_sub(1, Ordering::SeqCst);
                }
            }
            active.fetch_add(1, Ordering::SeqCst);
            let _active = Active(active);
            context.check()?;
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let (db, files, paths, digests) = {
                let world = world.lock().map_err(super::engine::protocol)?;
                (
                    world.db.clone(),
                    world.files.clone(),
                    world.paths.clone(),
                    world.digests.clone(),
                )
            };
            let result = query_world(&db, &files, &paths, &digests, session, query, &token)?;
            context.check()?;
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            serde_json::to_value(result).map_err(Error::from)
        })
    }
    pub(super) fn start_symbols(
        &self,
        query: WorkspaceQuery,
        token: CancellationToken,
    ) -> CpuTask<Result<Value>> {
        let world = self.world.clone();
        let context = self.context.clone();
        let session = self.analysis_context();
        let active = self.active.clone();
        spawn_cpu(token, move |token| {
            struct Active(Arc<AtomicUsize>);
            impl Drop for Active {
                fn drop(&mut self) {
                    self.0.fetch_sub(1, Ordering::SeqCst);
                }
            }
            active.fetch_add(1, Ordering::SeqCst);
            let _active = Active(active);
            context.check()?;
            if query.if_generation.is_some_and(|value| value != 1) {
                return Err(Error::Stale);
            }
            let (db, paths, digests) = {
                let world = world.lock().map_err(super::engine::protocol)?;
                (world.db.clone(), world.paths.clone(), world.digests.clone())
            };
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let host = ra_ap_ide::AnalysisHost::with_database(db);
            let native = host
                .analysis()
                .symbol_search(ra_ap_ide::Query::new(query.query), query.limit)
                .map_err(|_| Error::Cancelled)?;
            let mut rows = Vec::new();
            for target in native {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                let Some(path) = paths.get(&target.file_id) else {
                    continue;
                };
                let text = std::fs::read_to_string(path)?;
                let location = target.focus_range.unwrap_or(target.full_range);
                let start = utf8_position(&text, usize::from(location.start()))?;
                let end = utf8_position(&text, usize::from(location.end()))?;
                rows.push(json!({"name":target.name.as_str(),"kind":target.kind.map(|kind|format!("{kind:?}")),"containerName":target.container_name.map(|name|name.to_string()),"source_sha256":digests.get(path),"location":{"uri":super::engine::uri(path)?,"range":{"start":start,"end":end}}}));
            }
            context.check()?;
            serde_json::to_value(WorkspaceAnswer {session,generation:1,symbols:Section {category:"workspace_symbols".into(),evidence:Value::Array(rows),omitted:0,handles:Vec::new()},limitations:vec!["Immutable native snapshot with explicit source/configuration guards; native fuzzy search is provider-ranked and not an uncapped whole-program proof.".into()]}).map_err(Error::from)
        })
    }
}

fn utf8_position(text: &str, offset: usize) -> Result<Value> {
    let prefix = text.get(..offset).ok_or(Error::Stale)?;
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let start = prefix.rfind('\n').map_or(0, |index| index + 1);
    Ok(json!({"line":line,"character":offset-start}))
}
fn location(
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    file: ra_ap_vfs::FileId,
    range: native_ra_syntax::TextRange,
) -> Option<NativeLocation> {
    paths.get(&file).map(|path| NativeLocation {
        path: path.clone(),
        start: usize::from(range.start()),
        end: usize::from(range.end()),
    })
}
fn type_fact(
    db: &ra_ap_ide_db::RootDatabase,
    ty: &ra_ap_hir::Type<'_>,
    target: ra_ap_hir::DisplayTarget,
) -> TypeFact {
    TypeFact {
        display: ty.display(db, target).to_string(),
        is_reference: ty.is_reference(),
        is_mutable_reference: ty.is_mutable_reference(),
        contains_unknown: ty.contains_unknown(),
    }
}
fn query_world(
    db: &ra_ap_ide_db::RootDatabase,
    files: &BTreeMap<PathBuf, ra_ap_vfs::FileId>,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    digests: &BTreeMap<PathBuf, String>,
    session: AnalysisContext,
    query: Query,
    token: &CancellationToken,
) -> Result<NativeAnswer> {
    if query
        .if_generation
        .is_some_and(|generation| generation != 1)
    {
        return Err(Error::Stale);
    }
    if query.if_candidate.is_some() {
        return Err(Error::Unavailable("immutable snapshot does not issue native lazy-action fingerprints; select its supported typed operation without an unrelated candidate handle".into()));
    }
    let file = query.file.canonicalize()?;
    let source = crate::source_read::SourceRequest::Working { path: file.clone() }.acquire()?;
    let digest = source.identity().source_sha256();
    if digests.get(&file).is_none_or(|expected| expected != digest)
        || query
            .if_source
            .as_ref()
            .is_some_and(|expected| expected != digest)
    {
        return Err(Error::Stale);
    }
    let id = *files.get(&file).ok_or_else(|| {
        Error::Unavailable("selected source is outside the explicitly loaded snapshot".into())
    })?;
    let offset = crate::source_read::byte_offset(source.text(), query.line, query.column)?;
    let position = ra_ap_ide::FilePosition {
        file_id: id,
        offset: native_ra_syntax::TextSize::from(u32::try_from(offset).map_err(|_| {
            Error::Unavailable("source offset exceeds native text-size representation".into())
        })?),
    };
    let mut facts = None;
    let mut references = Vec::new();
    let mut sections = Vec::new();
    let mut linked_paths = BTreeSet::new();
    match query.kind {
        Kind::Explain=>{
            let sema=Semantics::new(db);
            if sema.file_to_module_defs(id).take(2).count()!=1{return Err(Error::Unavailable("typed snapshot facts need a unique native module context; this source is inactive or belongs to multiple selected crate/module contexts".into()));}
            let parsed=sema.parse(ra_ap_hir::EditionedFileId::current_edition(db,id));
            let node=parsed.syntax().token_at_offset(position.offset).right_biased().and_then(|token|token.parent()).ok_or_else(||Error::Unavailable("no native syntax at selected position".into()))?;
            let expression=node.ancestors().find_map(ast::Expr::cast);
            let scope=sema.scope(&node).ok_or_else(||Error::Unavailable("native scope is unavailable at selected position".into()))?;
            let display_target=scope.krate().to_display_target(db);
            let types=expression.as_ref().and_then(|expr|sema.type_of_expr(expr));
            let original=types.as_ref().map(|ty|type_fact(db,&ty.original,display_target));
            let adjusted=types.as_ref().and_then(|ty|ty.adjusted.as_ref()).map(|ty|type_fact(db,ty,display_target));
            let known=types.as_ref().is_some_and(|ty|!ty.original.contains_unknown()&&ty.adjusted.as_ref().is_none_or(|adjusted|!adjusted.contains_unknown()));
            let equal=if known {types.as_ref().and_then(|ty|ty.adjusted.as_ref().map(|adjusted|ty.original==*adjusted))}else{None};
            let coercion=if known {types.as_ref().and_then(|ty|ty.adjusted.as_ref().map(|adjusted|ty.original.could_coerce_to(db,adjusted)))}else{None};
            let method=node.ancestors().find_map(ast::MethodCallExpr::cast).and_then(|call|sema.resolve_method_call(&call));
            let implicit_bin=node.ancestors().find_map(ast::BinExpr::cast).and_then(|expr|sema.resolve_bin_expr(&expr));
            let declaration=node.ancestors().find_map(ast::Fn::cast).filter(|function|function.name().is_some_and(|name|name.syntax().text_range().contains(position.offset))).and_then(|function|sema.to_def(&function));
            let relationship=if method.is_some(){"resolved_method"}else if implicit_bin.is_some(){"resolved_operator"}else{"selected_function_declaration"};
            let function=method.or(implicit_bin).or(declaration);
            let mut target=function.map(|function|{
                let mapped=sema.source(function).and_then(|definition|sema.original_range_opt(definition.value.syntax()));
                TargetFact {relationship:relationship.into(),name:function.name(db).as_str().into(),location:mapped.and_then(|original|location(paths,original.file_id.file_id(db),original.range)),precision:if mapped.is_some(){"native_original_range"}else{"unavailable_or_non_unique_macro_origin"}.into()}
            });
            let enclosing=node.ancestors().find_map(ast::Fn::cast).and_then(|function|sema.to_def(&function));
            // IDE navigation owns plain paths, imports/reexports, type names and
            // local definitions; do not reproduce that resolver from spelling.
            let analysis=ra_ap_ide::AnalysisHost::with_database(db.clone()).analysis();
            let mut fixture=ra_ap_ide::RaFixtureConfig::default();fixture.disable_ra_fixture=true;
            let definitions=analysis.goto_definition(position,&ra_ap_ide::GotoDefinitionConfig {ra_fixture:fixture}).map_err(|_|Error::Cancelled)?.map(|definitions|definitions.info).unwrap_or_default();
            let total=definitions.len();
            let definitions=definitions.into_iter().take(query.limit).filter_map(|definition|location(paths,definition.file_id,definition.full_range).map(|location|TargetFact {relationship:"resolved_definition".into(),name:definition.name.to_string(),location:Some(location),precision:"native_original_range".into()})).collect::<Vec<_>>();
            if target.is_none()&&definitions.len()==1{target=definitions.first().cloned();}
            linked_paths.extend(definitions.iter().filter_map(|target|target.location.as_ref().map(|location|location.path.clone())));
            sections.push(Section {category:"definition".into(),omitted:total.saturating_sub(definitions.len()),evidence:serde_json::to_value(&definitions)?,handles:Vec::new()});
            let mut bounds=Vec::new();
            if let Some(function)=enclosing {
                for parameter in ra_ap_hir::GenericDef::Function(function).params(db){
                    if let ra_ap_hir::GenericParam::TypeParam(parameter)=parameter{
                        bounds.push(BoundFact {parameter:parameter.name(db).as_str().into(),declared_traits:parameter.trait_bounds(db).into_iter().map(|trait_|trait_.name(db).as_str().to_owned()).collect()});
                    }
                }
            }
            let expression=expression.as_ref().and_then(|expr|sema.original_range_opt(expr.syntax())).and_then(|mapped|location(paths,mapped.file_id.file_id(db),mapped.range));
            facts=Some(TypedFacts {expression,original,adjusted,original_equals_adjusted:equal,original_coerces_to_adjusted:coercion,target,bounds});
        }
        Kind::Refs=>{
            let analysis=ra_ap_ide::AnalysisHost::with_database(db.clone()).analysis();
            let mut fixture=ra_ap_ide::RaFixtureConfig::default();fixture.disable_ra_fixture=true;
            let config=ra_ap_ide::FindAllRefsConfig {search_scope:None,ra_fixture:fixture,exclude_imports:false,exclude_tests:false};
            let results=analysis.find_all_refs(position,&config).map_err(|_|Error::Cancelled)?.unwrap_or_default();
            let mut total=0usize;
            for (entity,result) in results.into_iter().enumerate(){
                for (id,usages) in result.references{
                    for (range,category) in usages{
                        if token.is_cancelled(){return Err(Error::Cancelled);}
                        total+=1;
                        if references.len()>=query.limit{continue;}
                        if let Some(location)=location(paths,id,range){references.push(ReferenceFact {entity,location,read:category.contains(ra_ap_ide_db::search::ReferenceCategory::READ),write:category.contains(ra_ap_ide_db::search::ReferenceCategory::WRITE)});}
                    }
                }
            }
            sections.push(Section {category:"native_reference_roles".into(),evidence:serde_json::to_value(&references)?,omitted:total.saturating_sub(references.len()),handles:Vec::new()});
        }
        Kind::Implementations=>{
            let analysis=ra_ap_ide::AnalysisHost::with_database(db.clone()).analysis();
            let native=analysis.goto_implementation(&ra_ap_ide::GotoImplementationConfig {filter_adjacent_derive_implementations:false},position).map_err(|_|Error::Cancelled)?;
            let native=native.map(|native|native.info).unwrap_or_default();
            let total=native.len();
            let targets=native.into_iter().take(query.limit).filter_map(|target|{
                location(paths,target.file_id,target.focus_range.unwrap_or(target.full_range)).map(|location|TargetFact {relationship:"implementation".into(),name:target.name.to_string(),location:Some(location),precision:"native_original_range".into()})
            }).collect::<Vec<_>>();
            linked_paths.extend(targets.iter().filter_map(|target|target.location.as_ref().map(|location|location.path.clone())));
            sections.push(Section {category:"native_implementations".into(),omitted:total.saturating_sub(targets.len()),evidence:serde_json::to_value(targets)?,handles:Vec::new()});
        }
        _=>return Err(Error::Unavailable("selected immutable native snapshot supports typed explain, reference roles and implementations; this operation belongs to a separately selected live-LSP context".into())),
    }
    let available = facts.as_ref().is_some_and(|facts| {
        facts.original.is_some() || facts.target.is_some() || !facts.bounds.is_empty()
    }) || !references.is_empty()
        || sections.iter().any(|section| {
            section
                .evidence
                .as_array()
                .is_some_and(|records| !records.is_empty())
        });
    linked_paths.extend(
        facts
            .as_ref()
            .and_then(|facts| facts.target.as_ref())
            .and_then(|target| target.location.as_ref())
            .map(|location| location.path.clone()),
    );
    linked_paths.extend(
        references
            .iter()
            .map(|reference| reference.location.path.clone()),
    );
    let location_sources = linked_paths
        .into_iter()
        .filter_map(|path| {
            digests
                .get(&path)
                .map(|source_sha256| super::LocationSource {
                    path: path.clone(),
                    source_sha256: source_sha256.clone(),
                })
        })
        .collect();
    Ok(NativeAnswer {session,source:source.identity().clone(),source_uri:super::engine::uri(&file)?,source_byte_offset:offset,generation:1,facts,references,availability:if available{"available"}else{"unknown_or_no_match_in_selected_restricted_context"}.into(),sections,location_sources,limitations:vec![
        "Immutable explicitly loaded Cargo snapshot; no automatic reloading/checking or proc-macro/build-script execution. Changed inputs require explicit stop/reprepare.".into(),
        "Native type display is presentation only. Equality/coercion fields use actual HIR types and unknown components remain unproven; arbitrary blanket-trait applicability and rustc acceptance are not established.".into(),
        "This is the selected analyzer Cargo context, not an admitted exact rustc compiler-unit identity; typed explain refuses inactive or ambiguous native module contexts.".into(),
        "Macro/generated origin can be unavailable/non-unique; no fallback range grants source-edit authority. Reference roles describe static native uses, not runtime reachability.".into(),
    ]})
}

#[derive(Debug, clap::Parser)]
#[command(
    name = "pse-native-analysis",
    about = "Explicit immutable native-analysis provider; --protocol is passive"
)]
struct Arguments {
    #[arg(long,conflicts_with_all=["root","operation","invocation","configuration"])]
    protocol: bool,
    #[arg(short = 'C', required_unless_present = "protocol")]
    root: Option<PathBuf>,
    #[arg(required_unless_present="protocol",value_parser=["__semantic-service"])]
    operation: Option<String>,
    #[arg(long, required_unless_present = "protocol")]
    invocation: Option<String>,
    #[arg(long, required_unless_present = "protocol")]
    configuration: Option<String>,
}
/// Fixed provider entry; all performing service work uses the ordinary semantic owner.
pub fn entry() -> std::process::ExitCode {
    use clap::Parser as _;
    use std::io::Write as _;
    let args = Arguments::parse();
    let result = (|| -> Result<()> {
        if args.protocol {
            let settings = schemars::generate::SchemaSettings::draft2020_12();
            let request = settings
                .clone()
                .for_deserialize()
                .into_generator()
                .into_root_schema_for::<Request>();
            let configuration = settings
                .clone()
                .for_deserialize()
                .into_generator()
                .into_root_schema_for::<Configuration>();
            let response = settings
                .clone()
                .for_serialize()
                .into_generator()
                .into_root_schema_for::<super::Reply>();
            let native = settings
                .for_serialize()
                .into_generator()
                .into_root_schema_for::<NativeAnswer>();
            serde_json::to_writer(
                std::io::stdout().lock(),
                &json!({"version":1,"tool":"native-analysis","backend":BACKEND,"request_schema":request,"configuration_schema":configuration,"response_schema":response,"native_payload_schema":native,"request_control":{"version":1,"cancel_byte":3,"reply_after_cpu_settlement":true}}),
            )?;
            std::io::stdout().write_all(b"\n")?;
            return Ok(());
        }
        let root = args
            .root
            .ok_or_else(|| Error::Protocol("missing checkout root".into()))?;
        let invocation = args
            .invocation
            .ok_or_else(|| Error::Protocol("missing original invocation".into()))?
            .try_into()
            .map_err(|error: uuid::Error| Error::Protocol(error.to_string()))?;
        let configuration = serde_json::from_str::<Configuration>(
            &args
                .configuration
                .ok_or_else(|| Error::Protocol("missing semantic configuration".into()))?,
        )?;
        if configuration.backend != Backend::NativeSnapshot {
            return Err(Error::Unavailable(
                "native helper cannot impersonate the live LSP backend".into(),
            ));
        }
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        runtime.block_on(super::serve(root, invocation, configuration))
    })();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(std::io::stderr().lock(), "{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = "trait Act { fn same(&self)->u32; }\nstruct A;\nstruct B;\nimpl Act for A { fn same(&self)->u32 { 1 } }\nimpl Act for B { fn same(&self)->u32 { 2 } }\nfn generic<T: Act>(value:T) { let _ = value.same(); }\nfn operations() { let mut a=A; let b=B; let _=a.same(); let _=b.same(); let _: &A = &mut a; let mut n=1; n=2; let _=n; }\n";

    fn fixture() -> (tempfile::TempDir, Configuration, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        std::fs::write(root.path().join("Cargo.toml"),"[package]\nname='native-analysis-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n").unwrap();
        std::fs::write(
            root.path().join("Cargo.lock"),
            "version=4\n[[package]]\nname='native-analysis-control'\nversion='0.0.0'\n",
        )
        .unwrap();
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        std::fs::copy(
            checkout.join("rust-toolchain.toml"),
            root.path().join("rust-toolchain.toml"),
        )
        .unwrap();
        let file = root.path().join("src/lib.rs");
        std::fs::write(&file, SOURCE).unwrap();
        let config = Configuration {
            backend: Backend::NativeSnapshot,
            project: root.path().into(),
            features: Vec::new(),
            no_default_features: false,
            target: None,
            build_macros: false,
        };
        (root, config, file)
    }
    fn query(file: &Path, needle: &str, kind: Kind) -> Query {
        let offset = SOURCE.find(needle).unwrap();
        let prefix = &SOURCE[..offset];
        Query {
            kind,
            file: file.into(),
            line: prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
            column: prefix.rsplit('\n').next().unwrap().chars().count() + 1,
            if_source: Some(crate::source_read::source_digest(SOURCE)),
            if_generation: Some(1),
            if_candidate: None,
            limit: 100,
            end_line: None,
        }
    }
    async fn answer(engine: &Engine, query: Query) -> Value {
        engine
            .start_query(query, CancellationToken::new())
            .settle()
            .await
            .unwrap()
            .unwrap()
    }

    #[tokio::test]
    async fn immutable_cargo_snapshot_resolves_targets_types_roles_and_rejects_stale_inputs() {
        let (root, configuration, file) = fixture();
        let engine = Engine::start(root.path(), configuration.clone()).unwrap();
        let first = answer(&engine, query(&file, "a.same", Kind::Explain)).await;
        let second = answer(&engine, query(&file, "b.same", Kind::Explain)).await;
        assert_ne!(
            first["facts"]["target"]["location"], second["facts"]["target"]["location"],
            "same spelling must resolve to distinct implementation declarations"
        );
        assert_eq!(
            first["session"]["configuration"]["backend"],
            "native_snapshot"
        );
        let generic = answer(&engine, query(&file, "value.same", Kind::Explain)).await;
        assert!(
            generic["facts"]["bounds"]
                .as_array()
                .unwrap()
                .iter()
                .any(|bound| bound["parameter"] == "T"
                    && bound["declared_traits"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|name| name == "Act"))
        );
        let adjustment = answer(&engine, query(&file, "&mut a", Kind::Explain)).await;
        assert_eq!(
            adjustment["facts"]["original"]["is_mutable_reference"],
            true
        );
        assert_eq!(
            adjustment["facts"]["adjusted"]["is_mutable_reference"],
            false
        );
        assert_eq!(adjustment["facts"]["original_equals_adjusted"], false);
        assert_eq!(adjustment["facts"]["original_coerces_to_adjusted"], true);
        let refs = answer(&engine, query(&file, "n=1", Kind::Refs)).await;
        let references = refs["references"].as_array().unwrap();
        assert!(references.iter().any(|reference| reference["read"] == true));
        assert!(
            references
                .iter()
                .any(|reference| reference["write"] == true)
        );
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        let refused = engine
            .start_query(query(&file, "n=1", Kind::Refs), cancelled.clone())
            .settle()
            .await
            .unwrap();
        assert!(matches!(refused, Err(Error::Cancelled)));
        assert!(
            engine.status().unwrap().context_current,
            "canceling a query must leave the selected context usable"
        );
        assert!(
            engine
                .start_symbols(
                    WorkspaceQuery {
                        query: "operations".into(),
                        limit: 20,
                        if_generation: Some(1)
                    },
                    CancellationToken::new()
                )
                .settle()
                .await
                .unwrap()
                .unwrap()["symbols"]["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|symbol| symbol["name"] == "operations")
        );
        assert!(matches!(
            engine
                .start_query(
                    query(&file, "a.same", Kind::Calls),
                    CancellationToken::new()
                )
                .settle()
                .await
                .unwrap(),
            Err(Error::Unavailable(_))
        ));
        let mut wrong_generation = query(&file, "a.same", Kind::Explain);
        wrong_generation.if_generation = Some(2);
        assert!(matches!(
            engine
                .start_query(wrong_generation, CancellationToken::new())
                .settle()
                .await
                .unwrap(),
            Err(Error::Stale)
        ));
        std::fs::write(&file, format!("{SOURCE}// changed source\n")).unwrap();
        assert!(matches!(
            engine
                .start_query(
                    query(&file, "a.same", Kind::Explain),
                    CancellationToken::new()
                )
                .settle()
                .await
                .unwrap(),
            Err(Error::Stale)
        ));
        assert!(!engine.status().unwrap().context_current);
        let mut macros = configuration;
        macros.build_macros = true;
        assert!(matches!(macros.resolve(), Err(Error::Unavailable(_))));
    }
}
