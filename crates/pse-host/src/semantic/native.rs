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
use ra_ap_ide_db::base_db::salsa;
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
mod importability;

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

fn preparation_target(configuration: &Configuration) -> Result<String> {
    let explicit_target =
        configuration.target.is_some() || std::env::var_os("CARGO_BUILD_TARGET").is_some();
    let target = if let Some(target) = &configuration.target {
        Some(target.clone())
    } else if let Some(target) = std::env::var_os("CARGO_BUILD_TARGET") {
        Some(
            target
                .into_string()
                .map_err(|_| Error::Unavailable("Cargo target environment is not UTF-8".into()))?,
        )
    } else {
        // Ask Cargo for its merged file configuration. This fixed read belongs
        // only to explicit preparation; no analysis query runs a tool.
        let output = std::process::Command::new("cargo")
            .args([
                "-Z",
                "unstable-options",
                "config",
                "get",
                "--format",
                "json",
            ])
            .current_dir(&configuration.project)
            .output()?;
        if !output.status.success() {
            return Err(Error::Unavailable(format!(
                "effective Cargo target observation exited {}",
                output.status
            )));
        }
        let configuration: Value = serde_json::from_slice(&output.stdout)?;
        match configuration.pointer("/build/target") {
            None | Some(Value::Null) => None,
            Some(Value::String(target)) => Some(target.clone()),
            Some(Value::Array(targets)) if targets.len() == 1 => Some(
                targets[0]
                    .as_str()
                    .ok_or_else(|| Error::Unavailable("Cargo target entry is not a string".into()))?
                    .to_owned(),
            ),
            _ => return Err(Error::Unavailable(
                "select one explicit native target for a multi-target/custom Cargo configuration"
                    .into(),
            )),
        }
    };
    if let Some(target) = target {
        if target.is_empty() {
            return Err(Error::Unavailable("effective Cargo target is empty".into()));
        }
        if target.ends_with(".json") {
            if !explicit_target {
                return Err(Error::Unavailable("select an explicit custom target JSON path; the merged Cargo view does not preserve its defining-file path origin".into()));
            }
            let path = configuration.project.join(target).canonicalize()?;
            return path
                .into_os_string()
                .into_string()
                .map_err(|_| Error::Unavailable("custom native target path is not UTF-8".into()));
        }
        return Ok(target);
    }
    let sysroot = ra_ap_project_model::Sysroot::empty();
    let targets = ra_ap_project_model::toolchain_info::target_tuple::get(
        ra_ap_project_model::toolchain_info::QueryConfig::Rustc(&sysroot, &configuration.project),
        None,
        &Default::default(),
    )
    .map_err(super::engine::protocol)?;
    if targets.len() != 1 {
        return Err(Error::Unavailable(
            "native compiler host target is ambiguous".into(),
        ));
    }
    targets
        .into_iter()
        .next()
        .ok_or_else(|| Error::Unavailable("native compiler host target is unavailable".into()))
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
    packages: Arc<Vec<super::NativePackageContext>>,
    package_issue: Option<String>,
    actual_target: String,
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
        let target = preparation_target(&configuration)?;
        let sysroot_src = installed_sysroot_sources(&configuration)?;
        let cargo = ra_ap_project_model::CargoConfig {
            features: ra_ap_project_model::CargoFeatures::Selected {
                features: configuration.features.clone(),
                no_default_features: configuration.no_default_features,
            },
            target: Some(target.clone()),
            sysroot: Some(ra_ap_project_model::RustLibSource::Discover),
            // Discover-with-src-override never invokes the library's rustup
            // component-install fallback. Preparation only observes installed inputs.
            sysroot_src: Some(sysroot_src),
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
        let native_root =
            ra_ap_vfs::AbsPathBuf::try_from(configuration.project.to_str().ok_or_else(|| {
                Error::Unavailable("native Cargo project path must be UTF-8".into())
            })?)
            .map_err(|_| Error::Unavailable("native Cargo project path is not absolute".into()))?;
        let manifest = ra_ap_project_model::ProjectManifest::discover_single(&native_root)
            .map_err(super::engine::protocol)?;
        let workspace = ra_ap_project_model::ProjectWorkspace::load(manifest, &cargo, &|_| {})
            .map_err(super::engine::protocol)?;
        let mut package_models = Vec::new();
        let package_issue;
        if let ra_ap_project_model::ProjectWorkspaceKind::Cargo { cargo, error, .. } =
            &workspace.kind
        {
            package_issue = error.as_ref().map(ToString::to_string);
            for package in cargo.packages() {
                let package = &cargo[package];
                for target_id in &package.targets {
                    let library = &cargo[*target_id];
                    let ra_ap_project_model::TargetKind::Lib { is_proc_macro } = library.kind
                    else {
                        continue;
                    };
                    let manifest = PathBuf::from(package.manifest.as_str());
                    package_models.push(super::NativePackageContext {
                        package_id: package.id.to_string(),
                        source_root: manifest.parent().ok_or(Error::Stale)?.to_owned(),
                        manifest,
                        workspace_root: PathBuf::from(cargo.workspace_root().as_str()),
                        library_root: PathBuf::from(library.root.as_str()),
                        features: BTreeSet::new(),
                        target: if is_proc_macro {
                            None
                        } else {
                            Some(target.clone())
                        },
                        role: if is_proc_macro { "proc-macro" } else { "lib" }.into(),
                        source_hashes: BTreeMap::new(),
                        dependencies: Vec::new(),
                        dependency_issues: Vec::new(),
                        dependency_exclusions: BTreeSet::new(),
                    });
                }
            }
        } else {
            package_issue = Some("prepared project is not a Cargo workspace".into());
        }
        let metadata_inputs = package_models
            .iter()
            .map(|package| package.manifest.clone())
            .collect::<BTreeSet<_>>();
        let metadata_hashes = metadata_inputs
            .iter()
            .map(|path| Ok((path.clone(), super::engine::file_digest(path)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        context.bind_snapshot_sources(&metadata_inputs)?;
        let (db, vfs, _) = ra_ap_load_cargo::load_workspace(workspace, &cargo.extra_env, &loading)
            .map_err(|error| Error::Unavailable(format!("native Cargo/VFS loading: {error:#}")))?;
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
        let crates = ra_ap_hir::Crate::all(&db);
        let mut packages = Vec::new();
        for model in package_models {
            for krate in &crates {
                if paths
                    .get(&krate.root_file(&db))
                    .is_none_or(|root| root != &model.library_root)
                {
                    continue;
                }
                let mut package = model.clone();
                package.features = krate
                    .cfg(&db)
                    .get_cfg_values("feature")
                    .map(|value| value.as_str().to_owned())
                    .collect();
                if krate
                    .cfg(&db)
                    .get_cfg_keys()
                    .any(|key| key.as_str() == "test")
                {
                    package.role = "lib-test".into();
                }
                package.source_hashes = digests
                    .iter()
                    .filter(|(path, _)| path.starts_with(&package.source_root))
                    .map(|(path, hash)| (path.clone(), hash.clone()))
                    .collect();
                packages.push((*krate, package));
            }
        }
        let mut profiles =
            std::collections::HashMap::<_, BTreeSet<super::NativeLibraryProfile>>::new();
        for (krate, package) in &packages {
            profiles
                .entry(*krate)
                .or_default()
                .insert(super::NativeLibraryProfile {
                    package_id: package.package_id.clone(),
                    manifest: package.manifest.clone(),
                    source_root: package.source_root.clone(),
                    library_root: package.library_root.clone(),
                    features: package.features.clone(),
                    target: package.target.clone(),
                    role: package.role.clone(),
                });
        }
        for (krate, package) in &mut packages {
            if profiles
                .get(krate)
                .is_none_or(|candidates| candidates.len() != 1)
            {
                package.dependency_issues.push(
                    "selected native root maps to multiple Cargo library package profiles".into(),
                );
            }
            let mut pending = krate
                .dependencies(&db)
                .into_iter()
                .map(|dependency| dependency.krate)
                .collect::<Vec<_>>();
            let mut visited = std::collections::HashSet::new();
            let mut dependencies = BTreeSet::new();
            while let Some(dependency) = pending.pop() {
                if !visited.insert(dependency) || dependency == *krate {
                    continue;
                }
                if dependency.is_builtin(&db) {
                    package
                        .dependency_exclusions
                        .insert(super::NativeDependencyExclusion::Sysroot);
                    continue;
                }
                pending.extend(
                    dependency
                        .dependencies(&db)
                        .into_iter()
                        .map(|dependency| dependency.krate),
                );
                if let Some(profile) = profiles
                    .get(&dependency)
                    .filter(|profiles| profiles.len() == 1)
                    .and_then(BTreeSet::first)
                {
                    if profile.role != "lib" || profile.target.is_none() {
                        package.dependency_issues.push(format!(
                            "native dependency {} has an unsupported host/proc-macro/test profile",
                            profile.package_id
                        ));
                    }
                    dependencies.insert(profile.clone());
                } else {
                    package.dependency_issues.push(format!("non-sysroot native dependency at {:?} has no unique captured Cargo library profile",paths.get(&dependency.root_file(&db))));
                }
            }
            package.dependencies = dependencies.into_iter().collect();
            package.dependency_issues.sort();
            package.dependency_issues.dedup();
        }
        let packages = packages.into_iter().map(|(_, package)| package).collect();
        for (path, original) in metadata_hashes {
            if super::engine::file_digest(&path)? != original {
                return Err(Error::Stale);
            }
        }
        let mut inputs = digests.keys().cloned().collect::<BTreeSet<_>>();
        inputs.extend(metadata_inputs);
        if target.ends_with(".json") {
            inputs.insert(PathBuf::from(&target));
        }
        context.bind_snapshot_sources(&inputs)?;
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
            packages: Arc::new(packages),
            package_issue,
            actual_target: target,
        })
    }
    pub(super) fn package_context(&self, package_id: &str) -> Result<Value> {
        self.context.check()?;
        if let Some(issue) = &self.package_issue {
            return Err(Error::Unavailable(format!(
                "native Cargo package context is incomplete: {issue}"
            )));
        }
        let packages = self
            .packages
            .iter()
            .filter(|package| package.package_id == package_id)
            .cloned()
            .collect::<Vec<_>>();
        if packages.is_empty() {
            return Err(Error::Unavailable(
                "requested library package is not part of the prepared native crate graph".into(),
            ));
        }
        serde_json::to_value(super::PackageContextAnswer {
            session: self.analysis_context(),
            generation: 1,
            packages,
        })
        .map_err(Error::from)
    }
    fn analysis_context(&self) -> AnalysisContext {
        AnalysisContext {backend:BACKEND.into(),configuration:self.configuration.clone(),position_encoding:"utf-8".into(),backend_status:Some(json!({"quiescent":true,"immutable":true,"actual_target":self.actual_target,"unicode_identifier_version":unicode_ident::UNICODE_VERSION,"unicode_properties_version":unicode_properties::UNICODE_VERSION})),preparation:"explicit immutable Cargo/VFS snapshot; checking/build scripts/proc macros disabled; source/configuration changes require stop and explicit reprepare".into()}
    }
    pub(super) fn status(&self) -> Result<super::SessionStatus> {
        let current = self.context.check();
        Ok(super::SessionStatus {
            configuration: self.configuration.clone(),
            backend: BACKEND.into(),
            position_encoding: "utf-8".into(),
            generation: 1,
            server_capabilities: json!({"nativeSnapshot":true,"typedFacts":true,"referenceRoles":true,"workspaceSymbolProvider":true,"sourceApplication":false,"procMacros":false}),
            backend_status: self.analysis_context().backend_status,
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
        let inputs = self
            .world
            .lock()
            .map_err(super::engine::protocol)
            .map(|world| {
                let (db, cancellation) = world.db.request_snapshot();
                (
                    db,
                    cancellation,
                    world.files.clone(),
                    world.paths.clone(),
                    world.digests.clone(),
                )
            });
        let native_cancel = inputs
            .as_ref()
            .ok()
            .map(|(_, cancellation, _, _, _)| cancellation.clone());
        let context = self.context.clone();
        let session = self.analysis_context();
        let active = self.active.clone();
        let task = spawn_cpu(token, move |token| {
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
            let (db, _cancellation, files, paths, digests) = inputs?;
            let result = native_result(salsa::Cancelled::catch(std::panic::AssertUnwindSafe(
                || query_world(&db, &files, &paths, &digests, session, query, &token),
            )))??;
            context.check()?;
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            serde_json::to_value(result).map_err(Error::from)
        });
        match native_cancel {
            Some(cancel) => task.with_cancel_hook(move || cancel.cancel()),
            None => task,
        }
    }
    pub(super) fn start_symbols(
        &self,
        query: WorkspaceQuery,
        token: CancellationToken,
    ) -> CpuTask<Result<Value>> {
        let inputs = self
            .world
            .lock()
            .map_err(super::engine::protocol)
            .map(|world| {
                let (db, cancellation) = world.db.request_snapshot();
                (db, cancellation, world.paths.clone(), world.digests.clone())
            });
        let native_cancel = inputs
            .as_ref()
            .ok()
            .map(|(_, cancellation, _, _)| cancellation.clone());
        let context = self.context.clone();
        let session = self.analysis_context();
        let active = self.active.clone();
        let task = spawn_cpu(token, move |token| {
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
            let (db, _cancellation, paths, digests) = inputs?;
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let host = ra_ap_ide::AnalysisHost::with_database(db);
            let mut native = native_result(
                host.analysis()
                    .symbol_search(symbol_query(&query), query.limit),
            )?;
            // The native libs switch replaces the workspace domain. Widening
            // therefore requires both queries against this same immutable DB.
            if query.include_dependencies {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                let mut libraries = symbol_query(&query);
                libraries.libs();
                native.extend(native_result(
                    host.analysis().symbol_search(libraries, query.limit),
                )?);
            }
            let mut rows = Vec::new();
            let mut seen = BTreeSet::new();
            for target in native {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                let Some(path) = paths.get(&target.file_id) else {
                    continue;
                };
                let text = std::fs::read_to_string(path)?;
                let location = target.focus_range.unwrap_or(target.full_range);
                if !seen.insert((
                    path.clone(),
                    u32::from(location.start()),
                    u32::from(location.end()),
                )) {
                    continue;
                }
                let start = utf8_position(&text, usize::from(location.start()))?;
                let end = utf8_position(&text, usize::from(location.end()))?;
                rows.push(json!({"name":target.name.as_str(),"kind":target.kind.map(|kind|format!("{kind:?}")),"containerName":target.container_name.map(|name|name.to_string()),"source_sha256":digests.get(path),"location":{"uri":super::engine::uri(path)?,"range":{"start":start,"end":end}}}));
            }
            context.check()?;
            serde_json::to_value(WorkspaceAnswer {session,generation:1,symbols:Section {category:"workspace_symbols".into(),evidence:Value::Array(rows),omitted:0,handles:Vec::new()},limitations:vec!["Immutable native snapshot with explicit source/configuration guards; native fuzzy search is provider-ranked and not an uncapped whole-program proof.".into()]}).map_err(Error::from)
        });
        match native_cancel {
            Some(cancel) => task.with_cancel_hook(move || cancel.cancel()),
            None => task,
        }
    }
}

fn native_result<T>(result: std::result::Result<T, salsa::Cancelled>) -> Result<T> {
    result.map_err(|cause| match cause {
        salsa::Cancelled::Local => Error::Cancelled,
        salsa::Cancelled::PendingWrite => Error::Stale,
        _ => Error::Protocol(format!(
            "native computation failed independently of requested cancellation: {cause:?}"
        )),
    })
}

fn symbol_query(query: &WorkspaceQuery) -> ra_ap_ide::Query {
    let mut native = ra_ap_ide::Query::new(query.query.clone());
    match query.matching {
        super::NativeNameMode::Fuzzy => native.fuzzy(),
        super::NativeNameMode::Exact => {
            native.exact();
            native.case_sensitive();
        }
        super::NativeNameMode::Prefix => {
            native.prefix();
            native.case_sensitive();
        }
    }
    native
}

fn installed_sysroot_sources(configuration: &Configuration) -> Result<ra_ap_vfs::AbsPathBuf> {
    let output = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .current_dir(&configuration.project)
        .output()?;
    if !output.status.success() {
        return Err(Error::Unavailable("could not observe the selected compiler sysroot; prepare the pinned toolchain explicitly".into()));
    }
    let root = std::str::from_utf8(&output.stdout)
        .map_err(super::engine::protocol)?
        .trim_end_matches(['\n', '\r']);
    let source = Path::new(root).join("lib/rustlib/src/rust/library");
    if !source.join("core/src/lib.rs").is_file() {
        return Err(Error::Unavailable("the selected compiler has no installed rust-src; install that pinned toolchain component explicitly before native analysis preparation".into()));
    }
    let source = source.canonicalize()?;
    ra_ap_vfs::AbsPathBuf::try_from(
        source
            .to_str()
            .ok_or_else(|| Error::Unavailable("installed rust-src path is not UTF-8".into()))?,
    )
    .map_err(|_| Error::Unavailable("installed rust-src path is not absolute".into()))
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
            let mut modules=sema.file_to_module_defs(id);
            let module=modules.next().ok_or_else(||Error::Unavailable("typed snapshot source is inactive in the selected native context".into()))?;
            if modules.next().is_some(){return Err(Error::Unavailable("typed snapshot facts need a unique native module context; this source belongs to multiple selected crate/module contexts".into()));}
            let parsed=sema.parse(ra_ap_hir::EditionedFileId::current_edition(db,id));
            let node=parsed.syntax().token_at_offset(position.offset).right_biased().and_then(|token|token.parent()).ok_or_else(||Error::Unavailable("no native syntax at selected position".into()))?;
            let expression=node.ancestors().find_map(ast::Expr::cast);
            let display_target=module.krate(db).to_display_target(db);
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
            let definitions=native_result(analysis.goto_definition(position,&ra_ap_ide::GotoDefinitionConfig {ra_fixture:fixture}))?.map(|definitions|definitions.info).unwrap_or_default();
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
        Kind::Importable {target}=>{
            let file=target.file.canonicalize()?;
            let source=crate::source_read::SourceRequest::Working {path:file.clone()}.acquire()?;
            if digests.get(&file).is_none_or(|hash|hash!=&target.source_sha256)||source.identity().source_sha256()!=target.source_sha256{return Err(Error::Stale);}
            let anchor=crate::source_read::semantic_anchor(source.text(),target.byte_start..target.byte_end).ok_or_else(||Error::Unavailable("importability target has no unique native declaration name".into()))?;
            let target_file=*files.get(&file).ok_or_else(||Error::Unavailable("importability target is outside the prepared native VFS".into()))?;
            let target_position=ra_ap_ide::FilePosition {file_id:target_file,offset:native_ra_syntax::TextSize::from(u32::try_from(anchor).map_err(|_|Error::Unavailable("native importability offset is unsupported".into()))?)};
            let fact=importability::inspect(db,target_position,position,token)?;
            linked_paths.insert(file);
            sections.push(Section {category:"importability".into(),evidence:serde_json::to_value(fact)?,omitted:0,handles:Vec::new()});
        },
        Kind::Refs=>{
            let analysis=ra_ap_ide::AnalysisHost::with_database(db.clone()).analysis();
            let mut fixture=ra_ap_ide::RaFixtureConfig::default();fixture.disable_ra_fixture=true;
            let config=ra_ap_ide::FindAllRefsConfig {search_scope:None,ra_fixture:fixture,exclude_imports:false,exclude_tests:false};
            let results=native_result(analysis.find_all_refs(position,&config))?.unwrap_or_default();
            let mut total=0usize;
            for(entity,result)in results.into_iter().enumerate(){
                for(id,usages)in result.references{
                    for(range,category)in usages{
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
            let native=native_result(analysis.goto_implementation(&ra_ap_ide::GotoImplementationConfig {filter_adjacent_derive_implementations:false},position))?;
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
            if section.category == "importability" {
                return section.evidence["importable"].is_boolean()
                    || section.evidence["visibility"].is_boolean();
            }
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
    const SOURCE: &str = "trait Act { fn same(&self)->u32; }\nstruct A;\nstruct B;\nimpl Act for A { fn same(&self)->u32 { 1 } }\nimpl Act for B { fn same(&self)->u32 { 2 } }\nfn generic<T: Act>(value:T) { let _ = value.same(); }\nfn operations() { let mut a=A; let b=B; let _=a.same(); let _=b.same(); let _: &A = &mut a; let mut n=1; n=2; let _=n; }\npub fn original()->u32 { 7 }\npub use original as facade;\nfn indirect(){let _=facade();}\nmod api { pub fn unused(){} fn hidden(){} fn internal_destination(){} }\nmod consumer { pub fn destination(){} pub fn unused(){} }\nfn standard(){let values=std::vec::Vec::<u32>::new();let optional=Some(7u32);let _=values.len();let _=optional.is_some();}\n";

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
    async fn cold_ide_symbol_cancellation_settles_and_preserves_the_world() {
        let (root, configuration, file) = fixture();
        let engine = Engine::start(root.path(), configuration).unwrap();
        let mut cold = engine.start_symbols(
            WorkspaceQuery {
                query: "Vec".into(),
                matching: super::super::NativeNameMode::Exact,
                limit: usize::MAX,
                include_dependencies: true,
                if_generation: Some(1),
            },
            CancellationToken::new(),
        );
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
        while engine.active.load(Ordering::SeqCst) == 0 {
            assert!(
                tokio::time::Instant::now() < deadline,
                "actual cold native operation never entered its CPU owner"
            );
            tokio::select! {
                result=cold.settle()=>panic!("cold native operation completed before cancellation could be exercised: {result:?}"),
                _=tokio::time::sleep(std::time::Duration::from_millis(1))=>{},
            }
        }
        // This proves actual coarse ownership on a cold world, not the precise
        // internal indexing instruction at which cancellation is delivered.
        // Barrier-controlled library tests separately prove nested query cancellation.
        let mut sibling = engine.start_query(
            query(&file, "original()->", Kind::Explain),
            CancellationToken::new(),
        );
        cold.cancel();
        let cancelled = tokio::time::timeout(std::time::Duration::from_secs(30), cold.settle())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(cancelled, Err(Error::Cancelled)), "{cancelled:?}");
        assert!(
            sibling.settle().await.unwrap().is_ok(),
            "another native request must remain independent"
        );
        let fresh = engine
            .start_symbols(
                WorkspaceQuery {
                    query: "operations".into(),
                    matching: super::super::NativeNameMode::Exact,
                    limit: 20,
                    include_dependencies: false,
                    if_generation: Some(1),
                },
                CancellationToken::new(),
            )
            .settle()
            .await
            .unwrap()
            .unwrap();
        assert!(
            fresh["symbols"]["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|symbol| symbol["name"] == "operations")
        );
        let status = engine.status().unwrap();
        assert!(status.context_current);
        assert_eq!(status.active_requests, 0);
    }

    #[test]
    fn captured_package_context_keeps_transitive_effective_features_and_sysroot_projection() {
        let (root, configuration, _file) = fixture();
        std::fs::create_dir_all(root.path().join("middle/src")).unwrap();
        std::fs::create_dir_all(root.path().join("leaf/src")).unwrap();
        std::fs::write(root.path().join("Cargo.toml"),"[package]\nname='native-analysis-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n[dependencies]\nprofile-middle={path='middle'}\n").unwrap();
        std::fs::write(root.path().join("middle/Cargo.toml"),"[package]\nname='profile-middle'\nversion='0.0.0'\nedition='2024'\n[dependencies]\nprofile-leaf={path='../leaf',features=['wide']}\n").unwrap();
        std::fs::write(
            root.path().join("middle/src/lib.rs"),
            "pub use profile_leaf::Value;\n",
        )
        .unwrap();
        std::fs::write(root.path().join("leaf/Cargo.toml"),"[package]\nname='profile-leaf'\nversion='0.0.0'\nedition='2024'\n[features]\nwide=[]\n").unwrap();
        std::fs::write(root.path().join("leaf/src/lib.rs"),"#[cfg(feature=\"wide\")] pub type Value=u64;\n#[cfg(not(feature=\"wide\"))] pub type Value=u32;\n").unwrap();
        std::fs::write(root.path().join("Cargo.lock"),"version=4\n[[package]]\nname='native-analysis-control'\nversion='0.0.0'\ndependencies=['profile-middle']\n[[package]]\nname='profile-middle'\nversion='0.0.0'\ndependencies=['profile-leaf']\n[[package]]\nname='profile-leaf'\nversion='0.0.0'\n").unwrap();
        let engine = Engine::start(root.path(), configuration).unwrap();
        let package = engine
            .packages
            .iter()
            .find(|package| {
                package.manifest == root.path().join("Cargo.toml") && package.role == "lib"
            })
            .unwrap();
        assert!(
            package.features.is_empty(),
            "the root's own features do not reveal its dependency context"
        );
        assert!(
            package.dependency_issues.is_empty(),
            "{:?}",
            package.dependency_issues
        );
        assert_eq!(
            package.dependencies.len(),
            2,
            "same-world transitive closure retains middle and leaf"
        );
        let leaf = package
            .dependencies
            .iter()
            .find(|profile| profile.manifest == root.path().join("leaf/Cargo.toml"))
            .unwrap();
        assert_eq!(leaf.features, BTreeSet::from(["wide".into()]));
        assert_eq!(leaf.role, "lib");
        assert_eq!(leaf.target.as_deref(), Some(engine.actual_target.as_str()));
        assert!(
            package
                .dependency_exclusions
                .contains(&super::super::NativeDependencyExclusion::Sysroot),
            "only actual builtin sysroot edges have a successful typed exclusion"
        );
        let original = engine.package_context(&package.package_id).unwrap();
        assert_eq!(
            original["packages"][0]["dependencies"]
                .as_array()
                .unwrap()
                .len(),
            2,
            "the native session transports the same captured closure without Cargo on read"
        );
        std::fs::write(root.path().join("leaf/Cargo.toml"),"[package]\nname='profile-leaf'\nversion='0.0.0'\nedition='2024'\n[features]\nwide=[]\nchanged=[]\n").unwrap();
        assert!(
            matches!(
                engine.package_context(&package.package_id),
                Err(Error::Stale)
            ),
            "mutating a transitive dependency manifest invalidates the captured world"
        );
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
        assert!(
            first["session"]["configuration"]["target"].is_null(),
            "requested configuration remains unchanged"
        );
        assert!(
            first["session"]["backend_status"]["actual_target"]
                .as_str()
                .is_some_and(|target| !target.is_empty()),
            "the actual loaded target is a separately retained native premise"
        );
        let direct = answer(&engine, query(&file, "original()->", Kind::Explain)).await;
        let facade = answer(&engine, query(&file, "facade()", Kind::Explain)).await;
        assert_eq!(
            direct["facts"]["target"]["location"], facade["facts"]["target"]["location"],
            "plain calls through reexports must retain the actual native declaration identity"
        );
        assert_eq!(facade["facts"]["target"]["name"], "original");
        let vector = answer(&engine, query(&file, "values.len", Kind::Explain)).await;
        assert_eq!(
            vector["facts"]["original"]["contains_unknown"], false,
            "installed std/core must participate in the same native world: {vector}"
        );
        assert!(
            vector["facts"]["original"]["display"]
                .as_str()
                .is_some_and(|value| value.contains("Vec") && value.contains("u32"))
        );
        assert_eq!(vector["facts"]["target"]["name"], "len");
        assert!(
            vector["facts"]["target"]["location"]["path"]
                .as_str()
                .is_some_and(|path| path.contains("rust/library"))
        );
        let optional = answer(&engine, query(&file, "optional.is_some", Kind::Explain)).await;
        assert_eq!(optional["facts"]["original"]["contains_unknown"], false);
        assert_eq!(optional["facts"]["target"]["name"], "is_some");
        let outline = crate::source_read::rust(SOURCE, crate::source_read::RustEdition::E2024);
        let target = |name: &str| {
            let declaration = outline
                .entries
                .iter()
                .find(|entry| {
                    entry.path.last().is_some_and(|part| part == name) && entry.kind == "FN"
                })
                .unwrap();
            super::super::ImportTarget {
                file: file.clone(),
                byte_start: declaration.range.byte_start,
                byte_end: declaration.range.byte_end,
                source_sha256: crate::source_read::source_digest(SOURCE),
            }
        };
        let destination = |name: &str, kind| {
            let mut query = query(&file, &format!("fn {name}("), kind);
            query.column += 3;
            query
        };
        let public = answer(
            &engine,
            destination(
                "destination",
                Kind::Importable {
                    target: target("unused"),
                },
            ),
        )
        .await;
        let public = &public["sections"][0]["evidence"];
        assert_eq!(public["visibility"], true);
        assert_eq!(public["importable"], true);
        assert!(public["path"].as_str().is_some());
        let hidden = answer(
            &engine,
            destination(
                "destination",
                Kind::Importable {
                    target: target("hidden"),
                },
            ),
        )
        .await;
        assert_eq!(hidden["sections"][0]["evidence"]["importable"], false);
        let internal = answer(
            &engine,
            destination(
                "internal_destination",
                Kind::Importable {
                    target: target("hidden"),
                },
            ),
        )
        .await;
        assert_eq!(
            internal["sections"][0]["evidence"]["importable"], true,
            "private item remains visible from its actual own module"
        );
        let associated = answer(
            &engine,
            destination(
                "destination",
                Kind::Importable {
                    target: target("same"),
                },
            ),
        )
        .await;
        assert!(
            associated["sections"][0]["evidence"]["importable"].is_null(),
            "associated item naming is not ordinary module importability"
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
            adjustment["facts"]["adjusted"]["is_mutable_reference"], false,
            "native &mut A to &A adjustment: {adjustment}"
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
                        matching: super::super::NativeNameMode::Exact,
                        limit: 20,
                        include_dependencies: false,
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
        let libraries = engine
            .start_symbols(
                WorkspaceQuery {
                    query: "Vec".into(),
                    matching: super::super::NativeNameMode::Exact,
                    limit: 100,
                    include_dependencies: true,
                    if_generation: Some(1),
                },
                CancellationToken::new(),
            )
            .settle()
            .await
            .unwrap()
            .unwrap();
        assert!(
            libraries["symbols"]["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|symbol| symbol["name"] == "Vec"
                    && symbol["location"]["uri"]
                        .as_str()
                        .is_some_and(|uri| uri.contains("rust/library"))),
            "explicit dependency discovery must include the same world's installed standard library: {libraries}"
        );
        let qualified = engine
            .start_symbols(
                WorkspaceQuery {
                    query: "api::unused".into(),
                    matching: super::super::NativeNameMode::Exact,
                    limit: 100,
                    include_dependencies: false,
                    if_generation: Some(1),
                },
                CancellationToken::new(),
            )
            .settle()
            .await
            .unwrap()
            .unwrap();
        let qualified = qualified["symbols"]["evidence"].as_array().unwrap();
        assert_eq!(
            qualified.len(),
            1,
            "native module qualification excludes the unrelated same basename"
        );
        assert_eq!(qualified[0]["name"], "unused");
        assert_eq!(
            qualified[0]["location"]["range"]["start"]["line"],
            SOURCE[..SOURCE.find("mod api").unwrap()]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
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
