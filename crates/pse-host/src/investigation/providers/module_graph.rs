// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored module routing, without a compiler, Cargo execution or filename-name guesses.
use super::super::*;
use super::{ProviderInput, selection};
use crate::source_read::SourceRequest;
use ra_ap_syntax::{
    AstNode as _, Edition, SourceFile,
    ast::{self, HasAttrs as _, HasModuleItem as _, HasName as _},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use tokio_util::sync::CancellationToken;

/// One written target/module route. Conditional routes are never active-cfg claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenModuleContext {
    /// Authored out-of-line ancestors; in-file ancestors remain with the native entry.
    pub modules: Vec<String>,
    /// Manifest-qualified target identity, preserving lib/bin ambiguity.
    pub target: String,
    /// At least one route or target has unresolved configuration conditions.
    pub conditional: bool,
}

/// Request-local plain facts; native AST handles never cross file workers.
#[derive(Debug, Default)]
pub struct ModuleGraph {
    contexts: BTreeMap<PathBuf, Vec<WrittenModuleContext>>,
    diagnostics: Vec<QueryDiagnostic>,
    issue_targets: BTreeMap<String, BTreeSet<usize>>,
}
impl ModuleGraph {
    /// Derive only actual authored/default target roots and native module declarations.
    pub fn discover(
        root: &Path,
        selector: &SourceSelector,
        token: &CancellationToken,
    ) -> Result<Self, Error> {
        let mut universe = selector.clone();
        universe.paths.clear();
        universe.packages.clear();
        universe.globs.clear();
        universe.exclude.clear();
        universe.types.clear();
        universe.tracked = None;
        universe.changed = false;
        universe.include_dependencies = false;
        if let SourceKind::Revision { revision } = &universe.kind {
            let repository =
                gix::open(root).map_err(|error| Error::Io(std::io::Error::other(error)))?;
            universe.kind = SourceKind::Revision {
                revision: repository
                    .rev_parse_single(revision.as_str())
                    .map_err(|error| Error::Io(std::io::Error::other(error)))?
                    .to_string(),
            };
        }
        let sources = selection::select_prepared_context(root, &universe, token)?;
        let available = sources
            .iter()
            .map(|source| {
                source
                    .path()
                    .strip_prefix(root)
                    .unwrap_or(source.path())
                    .to_path_buf()
            })
            .collect::<BTreeSet<_>>();
        let mut graph = Self::default();
        let mut loader = Loader {
            root,
            kind: &universe.kind,
            token,
            available: &available,
            graph: &mut graph,
            stack: Vec::new(),
        };
        for source in sources.iter().filter(|source| {
            source
                .path()
                .file_name()
                .is_some_and(|name| name == "Cargo.toml")
        }) {
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let input = selection::acquire_cancellable(root, source, token)?;
            let manifest: toml::Value = toml::from_str(
                std::str::from_utf8(&input.bytes)
                    .map_err(|error| Error::Invalid(error.to_string()))?,
            )
            .map_err(|error| Error::Invalid(error.to_string()))?;
            let Some(package) = manifest.get("package") else {
                continue;
            };
            let Some(name) = package.get("name").and_then(toml::Value::as_str) else {
                continue;
            };
            if !selector.packages.is_empty()
                && !selector.packages.iter().any(|selected| selected == name)
            {
                continue;
            }
            let directory = input.path.parent().unwrap_or(Path::new(""));
            let mut targets = BTreeMap::<(PathBuf, String), bool>::new();
            if let Some(lib) = manifest.get("lib") {
                let path = directory.join(
                    lib.get("path")
                        .and_then(toml::Value::as_str)
                        .unwrap_or("src/lib.rs"),
                );
                let target = lib
                    .get("name")
                    .and_then(toml::Value::as_str)
                    .unwrap_or(name)
                    .replace('-', "_");
                targets.insert(
                    (path, format!("{}:lib:{target}", input.path.display())),
                    lib.get("required-features").is_some(),
                );
            } else if package.get("autolib").and_then(toml::Value::as_bool) != Some(false)
                && loader.exists(&directory.join("src/lib.rs"))
            {
                targets.insert(
                    (
                        directory.join("src/lib.rs"),
                        format!("{}:lib:{}", input.path.display(), name.replace('-', "_")),
                    ),
                    false,
                );
            }
            if package.get("autobins").and_then(toml::Value::as_bool) != Some(false) {
                if loader.exists(&directory.join("src/main.rs")) {
                    targets.insert(
                        (
                            directory.join("src/main.rs"),
                            format!("{}:bin:{name}", input.path.display()),
                        ),
                        false,
                    );
                }
                let bins = directory.join("src/bin");
                for path in &available {
                    if let Ok(relative) = path.strip_prefix(&bins) {
                        let parts = relative.components().count();
                        let target = if parts == 1
                            && path.extension().is_some_and(|extension| extension == "rs")
                        {
                            path.file_stem()
                        } else if parts == 2
                            && path.file_name().is_some_and(|name| name == "main.rs")
                        {
                            relative.parent().and_then(Path::file_name)
                        } else {
                            None
                        };
                        if let Some(target) = target.and_then(|target| target.to_str()) {
                            targets.insert(
                                (
                                    path.clone(),
                                    format!("{}:bin:{target}", input.path.display()),
                                ),
                                false,
                            );
                        }
                    }
                }
            }
            for bin in manifest
                .get("bin")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
            {
                let target = bin
                    .get("name")
                    .and_then(toml::Value::as_str)
                    .unwrap_or(name);
                let paths = if let Some(path) = bin.get("path").and_then(toml::Value::as_str) {
                    vec![directory.join(path)]
                } else {
                    let mut paths = vec![
                        directory.join(format!("src/bin/{target}.rs")),
                        directory.join(format!("src/bin/{target}/main.rs")),
                    ];
                    if target == name {
                        paths.push(directory.join("src/main.rs"));
                    }
                    paths
                        .into_iter()
                        .filter(|path| loader.exists(path))
                        .collect()
                };
                if paths.is_empty() {
                    loader.issue(
                        &format!("{}:bin:{target}", input.path.display()),
                        format!("authored bin target {target} has no discoverable source root"),
                    );
                }
                for path in paths {
                    targets.insert(
                        (path, format!("{}:bin:{target}", input.path.display())),
                        bin.get("required-features").is_some(),
                    );
                }
            }
            for ((path, target), conditional) in targets {
                loader.file(
                    path,
                    WrittenModuleContext {
                        modules: Vec::new(),
                        target,
                        conditional,
                    },
                    true,
                )?;
            }
        }
        Ok(graph)
    }
    /// Multiple written routes are retained; an absent route does not invent a module name.
    pub fn contexts(&self, path: &Path) -> &[WrittenModuleContext] {
        self.contexts.get(path).map_or(&[], Vec::as_slice)
    }
    /// Structural uncertainty is explicit alongside any valid written routes.
    pub fn diagnostics(&self) -> &[QueryDiagnostic] {
        &self.diagnostics
    }
    /// Absence of graph uncertainty is not a compiler or active-cfg assertion.
    pub fn complete(&self) -> bool {
        self.diagnostics.is_empty()
    }
    /// A selected file's written target routes do not inherit unrelated crate failures.
    pub fn diagnostics_for(&self, path: &Path) -> impl Iterator<Item = &QueryDiagnostic> {
        let indices = self
            .contexts(path)
            .iter()
            .filter_map(|context| self.issue_targets.get(&context.target))
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>();
        self.diagnostics
            .iter()
            .enumerate()
            .filter_map(move |(index, diagnostic)| indices.contains(&index).then_some(diagnostic))
    }
    /// Completeness of established written routes for this source, without active-cfg claims.
    pub fn complete_for(&self, path: &Path) -> bool {
        self.diagnostics_for(path).next().is_none()
    }
}

struct Loader<'a> {
    root: &'a Path,
    kind: &'a SourceKind,
    token: &'a CancellationToken,
    available: &'a BTreeSet<PathBuf>,
    graph: &'a mut ModuleGraph,
    stack: Vec<PathBuf>,
}
impl Loader<'_> {
    fn exists(&self, path: &Path) -> bool {
        let path = normalize(path);
        if *self.kind == SourceKind::Working {
            self.root.join(path).is_file()
        } else {
            self.available.contains(&path)
        }
    }
    fn issue(&mut self, target: &str, message: String) {
        self.graph
            .issue_targets
            .entry(target.into())
            .or_default()
            .insert(self.graph.diagnostics.len());
        self.graph.diagnostics.push(QueryDiagnostic {
            code: "search.written_module_context".into(),
            message,
            source: None,
        });
    }
    fn acquire(&self, path: &Path) -> Result<ProviderInput, Error> {
        let source = match self.kind {
            SourceKind::Working => SourceRequest::Working {
                path: self.root.join(path),
            },
            SourceKind::Index => SourceRequest::Index {
                repository: self.root.into(),
                path: path.into(),
            },
            SourceKind::Revision { revision } => SourceRequest::Blob {
                repository: self.root.into(),
                path: path.into(),
                revision: revision.clone(),
            },
        };
        selection::acquire_cancellable(self.root, &source, self.token)
    }
    fn file(
        &mut self,
        path: PathBuf,
        context: WrittenModuleContext,
        root_file: bool,
    ) -> Result<(), Error> {
        let path = normalize(&path);
        if self.token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if self.stack.contains(&path) {
            self.issue(
                &context.target,
                format!("cyclic written module source route at {path:?}"),
            );
            return Ok(());
        }
        let contexts = self.graph.contexts.entry(path.clone()).or_default();
        if contexts.contains(&context) {
            return Ok(());
        }
        contexts.push(context.clone());
        let input = match self.acquire(&path) {
            Ok(input) => input,
            Err(Error::Cancelled) => return Err(Error::Cancelled),
            Err(error) => {
                self.issue(
                    &context.target,
                    format!("module source {path:?} unavailable: {error}"),
                );
                return Ok(());
            }
        };
        let text = match std::str::from_utf8(&input.bytes) {
            Ok(text) => text,
            Err(error) => {
                self.issue(
                    &context.target,
                    format!("module source {path:?} is not UTF-8: {error}"),
                );
                return Ok(());
            }
        };
        let parsed = SourceFile::parse(text, Edition::Edition2024);
        if !parsed.errors().is_empty() {
            self.issue(
                &context.target,
                format!("native parser recovered written module source {path:?}"),
            );
        }
        let parent = path.parent().unwrap_or(Path::new(""));
        let mod_rs = root_file || path.file_name().is_some_and(|name| name == "mod.rs");
        let directory = if mod_rs {
            parent.to_path_buf()
        } else {
            parent.join(path.file_stem().unwrap_or_default())
        };
        self.stack.push(path.clone());
        self.items(parsed.tree().items(), &context, &directory, parent, false)?;
        self.stack.pop();
        Ok(())
    }
    fn items(
        &mut self,
        items: impl Iterator<Item = ast::Item>,
        context: &WrittenModuleContext,
        directory: &Path,
        file_parent: &Path,
        inline_body: bool,
    ) -> Result<(), Error> {
        for item in items {
            if self.token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let ast::Item::Module(module) = item else {
                continue;
            };
            let Some(name) = module.name().map(crate::source_read::identifier_name) else {
                self.issue(
                    &context.target,
                    "module declaration has no written name".into(),
                );
                continue;
            };
            let mut next = context.clone();
            next.modules.push(name.clone());
            let mut paths = Vec::new();
            let mut unsupported_path = false;
            let mut conditional_path = false;
            for attribute in module.attrs() {
                match attribute.simple_name().as_deref() {
                    Some("cfg") => next.conditional = true,
                    Some("cfg_attr") => {
                        next.conditional = true;
                        for meta in attribute
                            .syntax()
                            .descendants()
                            .filter_map(ast::KeyValueMeta::cast)
                        {
                            if meta
                                .path()
                                .and_then(|path| path.as_single_name_ref())
                                .is_some_and(|name| name.text() == "path")
                            {
                                conditional_path = true;
                                if let Some(ast::Expr::Literal(literal)) = meta.expr() {
                                    if let ast::LiteralKind::String(value) = literal.kind() {
                                        if let Ok(value) = value.value() {
                                            paths.push(PathBuf::from(value.as_ref()));
                                            continue;
                                        }
                                    }
                                }
                                unsupported_path = true;
                            }
                        }
                    }
                    Some("path") => {
                        if let Some(ast::Meta::KeyValueMeta(meta)) = attribute.meta() {
                            if let Some(ast::Expr::Literal(literal)) = meta.expr() {
                                if let ast::LiteralKind::String(value) = literal.kind() {
                                    if let Ok(value) = value.value() {
                                        paths.push(PathBuf::from(value.as_ref()));
                                        continue;
                                    }
                                }
                            }
                        }
                        unsupported_path = true;
                    }
                    _ => {}
                }
            }
            if unsupported_path {
                self.issue(
                    &context.target,
                    format!(
                        "conditional/nonliteral module path for {} remains unresolved",
                        next.modules.join("::")
                    ),
                );
            }
            if let Some(inline) = module.item_list() {
                if paths.is_empty() || conditional_path {
                    self.items(
                        inline.items(),
                        &next,
                        &directory.join(&name),
                        file_parent,
                        true,
                    )?;
                }
                for path in paths {
                    self.items(
                        inline.items(),
                        &next,
                        &directory.join(path),
                        file_parent,
                        true,
                    )?;
                }
            } else {
                let mut candidates = Vec::new();
                if paths.is_empty() || conditional_path {
                    candidates.extend(
                        [
                            directory.join(format!("{name}.rs")),
                            directory.join(&name).join("mod.rs"),
                        ]
                        .into_iter()
                        .filter(|path| self.exists(path)),
                    );
                }
                candidates.extend(paths.into_iter().map(|path| {
                    if inline_body {
                        directory.join(path)
                    } else {
                        file_parent.join(path)
                    }
                }));
                candidates.sort();
                candidates.dedup();
                if candidates.len() != 1 {
                    self.issue(
                        &context.target,
                        format!(
                            "module {} has {} written source candidates",
                            next.modules.join("::"),
                            candidates.len()
                        ),
                    );
                }
                for candidate in candidates {
                    self.file(candidate, next.clone(), false)?;
                }
            }
        }
        Ok(())
    }
}

fn normalize(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir if normalized.file_name().is_some_and(|name| name != "..") => {
                normalized.pop();
            }
            Component::ParentDir if normalized.has_root() => {}
            part => normalized.push(part.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};
    #[test]
    fn raw_module_names_use_real_paths_and_unrelated_targets_do_not_poison_narrow_queries() {
        let root = tempfile::tempdir().unwrap();
        for package in ["good", "broken"] {
            fs::create_dir_all(root.path().join(package).join("src")).unwrap();
            fs::write(
                root.path().join(package).join("Cargo.toml"),
                format!("[package]\nname='{package}'\nversion='0.0.0'\nedition='2024'\n"),
            )
            .unwrap();
        }
        fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nmembers=['good','broken']\nresolver='3'\n",
        )
        .unwrap();
        fs::write(
            root.path().join("good/src/lib.rs"),
            "mod r#type; pub struct Writer;\n",
        )
        .unwrap();
        fs::write(
            root.path().join("good/src/type.rs"),
            "pub fn r#match() {}\n",
        )
        .unwrap();
        fs::write(root.path().join("broken/src/lib.rs"), "mod missing;\n").unwrap();
        let graph = ModuleGraph::discover(
            root.path(),
            &SourceSelector::default(),
            &CancellationToken::new(),
        )
        .unwrap();
        assert!(!graph.complete());
        assert!(graph.complete_for(Path::new("good/src/type.rs")));
        assert!(!graph.complete_for(Path::new("broken/src/lib.rs")));
        assert_eq!(
            graph.contexts(Path::new("good/src/type.rs"))[0].modules,
            ["type"]
        );
        for (path, name) in [
            ("good/src/lib.rs", "Writer"),
            ("good/src/type.rs", "type::match"),
        ] {
            let mut request = Request {
                predicate: Predicate::Symbol(SymbolQuery {
                    name: name.into(),
                    matching: NameMatch::Exact,
                    ..Default::default()
                }),
                ..Default::default()
            };
            request.sources.paths.push(path.into());
            let summary = super::super::run(
                root.path(),
                &request,
                &CancellationToken::new(),
                &mut |_| Ok(()),
            )
            .unwrap();
            assert!(summary.complete, "{summary:?}");
            assert_eq!(summary.matches, 1);
            assert_eq!(summary.unknown, 0);
        }
    }
    #[test]
    fn written_roots_paths_inline_modules_and_multiple_routes_keep_native_names() {
        let root = tempfile::tempdir().unwrap();
        for directory in ["src/control", "src/alt", "src/inline"] {
            fs::create_dir_all(root.path().join(directory)).unwrap();
        }
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname='written'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        for (path, text) in [
            (
                "src/lib.rs",
                "mod control; #[path = \"alt/custom.rs\"] mod renamed; mod inline { #[path = \"nested.rs\"] mod leaf; } #[cfg(unix)] #[path=\"shared.rs\"] mod first; #[cfg(windows)] #[path=\"shared.rs\"] mod second;",
            ),
            (
                "src/control.rs",
                "pub fn run() {} mod child; #[path=\"relative.rs\"] mod selected;",
            ),
            ("src/control/child.rs", "pub fn child() {}"),
            ("src/relative.rs", "pub fn relative() {}"),
            ("src/alt/custom.rs", "pub fn alternate() {}"),
            ("src/inline/nested.rs", "pub fn nested() {}"),
            ("src/shared.rs", "pub fn shared() {}"),
        ] {
            fs::write(root.path().join(path), text).unwrap();
        }
        let graph = ModuleGraph::discover(
            root.path(),
            &SourceSelector::default(),
            &CancellationToken::new(),
        )
        .unwrap();
        assert!(graph.complete(), "{:?}", graph.diagnostics());
        assert_eq!(
            graph.contexts(Path::new("src/control.rs"))[0].modules,
            ["control"]
        );
        assert_eq!(
            graph.contexts(Path::new("src/control/child.rs"))[0].modules,
            ["control", "child"]
        );
        assert_eq!(
            graph.contexts(Path::new("src/relative.rs"))[0].modules,
            ["control", "selected"]
        );
        assert_eq!(
            graph.contexts(Path::new("src/alt/custom.rs"))[0].modules,
            ["renamed"]
        );
        assert_eq!(
            graph.contexts(Path::new("src/inline/nested.rs"))[0].modules,
            ["inline", "leaf"]
        );
        let shared = graph.contexts(Path::new("src/shared.rs"));
        assert_eq!(shared.len(), 2);
        assert!(shared.iter().all(|context| context.conditional));
        assert!(graph.contexts(Path::new("src/unattached.rs")).is_empty());
        for args in [
            vec!["init", "-q"],
            vec!["add", "."],
            vec![
                "-c",
                "user.name=A",
                "-c",
                "user.email=a@example.invalid",
                "commit",
                "-qm",
                "native modules",
            ],
        ] {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root.path())
                    .status()
                    .unwrap()
                    .success()
            );
        }
        fs::remove_file(root.path().join("src/control.rs")).unwrap();
        for kind in [
            SourceKind::Index,
            SourceKind::Revision {
                revision: "HEAD".into(),
            },
        ] {
            let selector = SourceSelector {
                kind,
                ..Default::default()
            };
            let graph =
                ModuleGraph::discover(root.path(), &selector, &CancellationToken::new()).unwrap();
            assert_eq!(
                graph.contexts(Path::new("src/control.rs"))[0].modules,
                ["control"]
            );
        }
    }
    #[test]
    fn ambiguous_and_cyclic_authored_routes_retain_positives_with_unknown_coverage() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src/dual")).unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname='ambiguous'\nversion='0.1.0'\n",
        )
        .unwrap();
        fs::write(
            root.path().join("src/lib.rs"),
            "mod dual; #[path=\"lib.rs\"] mod cycle;",
        )
        .unwrap();
        for path in ["src/dual.rs", "src/dual/mod.rs"] {
            fs::write(root.path().join(path), "pub fn witness() {}").unwrap();
        }
        let graph = ModuleGraph::discover(
            root.path(),
            &SourceSelector::default(),
            &CancellationToken::new(),
        )
        .unwrap();
        assert!(!graph.complete());
        assert_eq!(
            graph.contexts(Path::new("src/dual.rs"))[0].modules,
            ["dual"]
        );
        assert_eq!(
            graph.contexts(Path::new("src/dual/mod.rs"))[0].modules,
            ["dual"]
        );
    }
    #[test]
    fn authored_nonstandard_targets_and_conditional_paths_do_not_guess_from_filenames() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        fs::create_dir(root.path().join("bin")).unwrap();
        fs::write(root.path().join("Cargo.toml"),"[package]\nname='targets'\nversion='0.1.0'\nautolib=false\nautobins=false\n[lib]\npath='engine.rs'\n[[bin]]\nname='worker'\npath='bin/start.rs'\n").unwrap();
        for (path, text) in [
            (
                "engine.rs",
                "#[cfg_attr(feature=\"alternate\",path=\"alternate.rs\")] mod selected;",
            ),
            ("selected.rs", "pub fn default_written() {}"),
            ("alternate.rs", "pub fn alternate_written() {}"),
            ("bin/start.rs", "mod tasks;"),
            ("bin/tasks.rs", "pub fn run() {}"),
            ("src/main.rs", "fn unused() {}"),
            ("src/lib.rs", "fn unused() {}"),
        ] {
            fs::write(root.path().join(path), text).unwrap();
        }
        let graph = ModuleGraph::discover(
            root.path(),
            &SourceSelector::default(),
            &CancellationToken::new(),
        )
        .unwrap();
        assert_eq!(
            graph.contexts(Path::new("bin/tasks.rs"))[0].modules,
            ["tasks"]
        );
        assert!(
            graph.contexts(Path::new("bin/tasks.rs"))[0]
                .target
                .ends_with("bin:worker")
        );
        assert!(graph.contexts(Path::new("src/main.rs")).is_empty());
        assert!(graph.contexts(Path::new("src/lib.rs")).is_empty());
        for path in ["selected.rs", "alternate.rs"] {
            let context = &graph.contexts(Path::new(path))[0];
            assert_eq!(context.modules, ["selected"]);
            assert!(context.conditional);
        }
        assert!(
            !graph.complete(),
            "conditional alternatives remain written possibilities"
        );
    }
}
