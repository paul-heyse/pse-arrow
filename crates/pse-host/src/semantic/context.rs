// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Captured Cargo inputs and a terminal, preparation-only native input watch set.
use super::{Configuration, Error, Result};
use rustix::fs::inotify;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    os::unix::ffi::OsStrExt as _,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};

pub(super) struct Context {
    manifests: BTreeMap<PathBuf, Option<String>>,
    configuration_watch: Mutex<ConfigurationWatch>,
    preparation: Option<Mutex<Preparation>>,
    snapshot_sources: Option<Mutex<ConfigurationWatch>>,
}
impl Context {
    pub(super) fn capture(checkout: &Path, configuration: &Configuration) -> Result<Self> {
        let mut command = std::process::Command::new("cargo");
        command.args(["metadata", "--format-version", "1", "--locked"]);
        if !configuration.build_macros {
            command.arg("--no-deps");
        }
        if !configuration.features.is_empty() {
            command.args(["--features", &configuration.features.join(",")]);
        }
        if configuration.no_default_features {
            command.arg("--no-default-features");
        }
        let output = command.current_dir(&configuration.project).output()?;
        if !output.status.success() {
            return Err(Error::Unavailable(format!(
                "Cargo model preparation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        let metadata: Value = serde_json::from_slice(&output.stdout)?;
        let workspace = metadata["workspace_root"].as_str().ok_or_else(|| {
            Error::Unavailable("Cargo metadata omitted its effective workspace root".into())
        })?;
        let mut paths = BTreeSet::new();
        for package in metadata["packages"].as_array().into_iter().flatten() {
            if let Some(path) = package["manifest_path"].as_str() {
                paths.insert(PathBuf::from(path));
            }
        }
        for root in [Path::new(workspace), &configuration.project] {
            paths.insert(root.join("Cargo.toml"));
            paths.insert(root.join("Cargo.lock"));
        }
        // Cargo configuration discovery and rustup selection can extend above a member project.
        // Absence is captured too, so creating an overriding configuration invalidates the model.
        for root in [checkout, &configuration.project] {
            for ancestor in root.ancestors() {
                for name in [
                    "rust-toolchain",
                    "rust-toolchain.toml",
                    ".cargo/config",
                    ".cargo/config.toml",
                    "rust-analyzer.toml",
                ] {
                    paths.insert(ancestor.join(name));
                }
            }
        }
        if let Some(home) = std::env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        {
            paths.insert(home.join("config"));
            paths.insert(home.join("config.toml"));
        }
        let configuration_watch = Mutex::new(ConfigurationWatch::capture(&paths)?);
        let manifests = paths
            .into_iter()
            .map(|path| {
                let digest = super::engine::file_digest(&path)?;
                Ok((path, digest))
            })
            .collect::<Result<_>>()?;
        let preparation = (configuration.build_macros
            || configuration.backend == super::Backend::NativeSnapshot)
            .then(|| Preparation::capture(&metadata, configuration))
            .transpose()?
            .map(Mutex::new);
        let context = Self {
            manifests,
            configuration_watch,
            preparation,
            snapshot_sources: None,
        };
        // Bind metadata membership/auto-target discovery to the captured bytes rather than
        // pairing an old metadata response with manifests edited before their first digest.
        // This extra native read belongs only to explicit preparation.
        let confirmation = command.output()?;
        if !confirmation.status.success() {
            return Err(Error::Unavailable(format!(
                "Cargo model confirmation failed: {}",
                String::from_utf8_lossy(&confirmation.stderr)
            )));
        }
        if serde_json::from_slice::<Value>(&confirmation.stdout)? != metadata {
            return Err(Error::Stale);
        }
        context.check()?;
        Ok(context)
    }
    pub(super) fn check(&self) -> Result<()> {
        if let Some(sources) = &self.snapshot_sources {
            sources.lock().map_err(super::engine::protocol)?.check()?;
        }
        self.configuration_watch
            .lock()
            .map_err(super::engine::protocol)?
            .check()?;
        for (path, original) in &self.manifests {
            if super::engine::file_digest(path)? != *original {
                return Err(Error::Stale);
            }
        }
        if let Some(preparation) = &self.preparation {
            preparation
                .lock()
                .map_err(super::engine::protocol)?
                .check()?;
        }
        Ok(())
    }
    /// Reuse native input fencing for loaded dependency/sysroot sources as well as local roots.
    #[cfg(feature = "native-hir")]
    pub(super) fn bind_snapshot_sources(&mut self, paths: &BTreeSet<PathBuf>) -> Result<()> {
        self.check()?;
        self.snapshot_sources = Some(Mutex::new(ConfigurationWatch::capture(paths)?));
        self.check()
    }
    pub(super) fn seal(&self) -> Result<()> {
        if let Some(preparation) = &self.preparation {
            let mut preparation = preparation.lock().map_err(super::engine::protocol)?;
            if let Err(error) = preparation.seal() {
                preparation.stale = true;
                return Err(error);
            }
        }
        self.check()
    }
}

struct ConfigurationWatch {
    fd: rustix::fd::OwnedFd,
    directories: BTreeMap<i32, PathBuf>,
    inputs: BTreeSet<PathBuf>,
    stale: bool,
}
impl ConfigurationWatch {
    fn capture(paths: &BTreeSet<PathBuf>) -> Result<Self> {
        let mut fence = Self {
            fd: inotify::init(inotify::CreateFlags::NONBLOCK | inotify::CreateFlags::CLOEXEC)
                .map_err(std::io::Error::from)?,
            directories: BTreeMap::new(),
            inputs: paths.clone(),
            stale: false,
        };
        let mut seen = BTreeSet::new();
        for path in paths {
            // Retain both the authored name and an existing symlink target. Parent watches
            // cover atomic file replacement and creation of currently absent config paths.
            let physical = path.canonicalize();
            if let Ok(physical) = physical {
                fence.inputs.insert(physical);
            } else if std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_symlink()) {
                return Err(Error::Unavailable(
                    "effective configuration has an unresolved symlink; input freshness is unknown"
                        .into(),
                ));
            }
            let mut parent = path.parent().ok_or(Error::Stale)?;
            while !parent.is_dir() {
                parent = parent.parent().ok_or(Error::Stale)?;
            }
            let normalized = parent
                .canonicalize()?
                .join(path.strip_prefix(parent).map_err(super::engine::protocol)?);
            fence.inputs.insert(normalized);
        }
        for input in &fence.inputs {
            let mut parent = input.parent().ok_or(Error::Stale)?;
            while !parent.is_dir() {
                parent = parent.parent().ok_or(Error::Stale)?;
            }
            for ancestor in parent.ancestors() {
                let ancestor = ancestor.canonicalize()?;
                if seen.insert(ancestor.clone()) {
                    let wd = inotify::add_watch(&fence.fd, &ancestor, watch_flags())
                        .map_err(std::io::Error::from)?;
                    fence.directories.insert(wd, ancestor);
                }
            }
        }
        fence.check()?;
        Ok(fence)
    }
    fn check(&mut self) -> Result<()> {
        match native_changes(&self.fd, &self.directories, |path| {
            self.inputs.iter().any(|input| input.starts_with(path))
        }) {
            Ok(changed) => self.stale |= changed,
            Err(error) => {
                self.stale = true;
                return Err(error);
            }
        }
        if self.stale {
            Err(Error::Stale)
        } else {
            Ok(())
        }
    }
}

fn watch_flags() -> inotify::WatchFlags {
    inotify::WatchFlags::MODIFY
        | inotify::WatchFlags::ATTRIB
        | inotify::WatchFlags::CREATE
        | inotify::WatchFlags::DELETE
        | inotify::WatchFlags::MOVED_FROM
        | inotify::WatchFlags::MOVED_TO
        | inotify::WatchFlags::DELETE_SELF
        | inotify::WatchFlags::MOVE_SELF
}
fn native_changes(
    fd: &rustix::fd::OwnedFd,
    directories: &BTreeMap<i32, PathBuf>,
    relevant: impl Fn(&Path) -> bool,
) -> Result<bool> {
    let mut changed = false;
    let mut buffer = [std::mem::MaybeUninit::<u8>::uninit(); 8192];
    let mut reader = inotify::Reader::new(fd, &mut buffer);
    loop {
        match reader.next() {
            Ok(event) => {
                let path = directories.get(&event.wd()).map(|root| {
                    event.file_name().map_or_else(
                        || root.clone(),
                        |name| root.join(std::ffi::OsStr::from_bytes(name.to_bytes())),
                    )
                });
                changed |= event.events().intersects(
                    inotify::ReadFlags::QUEUE_OVERFLOW
                        | inotify::ReadFlags::IGNORED
                        | inotify::ReadFlags::UNMOUNT,
                ) || path.is_none_or(|path| relevant(&path));
            }
            Err(rustix::io::Errno::AGAIN) => return Ok(changed),
            Err(error) => return Err(std::io::Error::from(error).into()),
        }
    }
}

struct Preparation {
    fd: rustix::fd::OwnedFd,
    directories: BTreeMap<i32, PathBuf>,
    visited: BTreeSet<PathBuf>,
    expanded: BTreeSet<PathBuf>,
    excluded: Vec<PathBuf>,
    build_directory: PathBuf,
    packages: BTreeMap<String, Vec<PathBuf>>,
    started: SystemTime,
    stale: bool,
    build_outputs: bool,
}
impl Preparation {
    fn capture(metadata: &Value, configuration: &Configuration) -> Result<Self> {
        let target = metadata["target_directory"].as_str().ok_or_else(|| {
            Error::Unavailable("Cargo metadata omitted its target directory".into())
        })?;
        let target = PathBuf::from(target);
        let build_directory = metadata["build_directory"].as_str().map(PathBuf::from)
            .ok_or_else(|| Error::Unavailable("pinned Cargo metadata omitted build_directory; preparation input discovery is unknown".into()))?;
        let mut fence = Self {
            fd: inotify::init(inotify::CreateFlags::NONBLOCK | inotify::CreateFlags::CLOEXEC)
                .map_err(std::io::Error::from)?,
            directories: BTreeMap::new(),
            visited: BTreeSet::new(),
            expanded: BTreeSet::new(),
            excluded: vec![
                target.canonicalize().unwrap_or(target),
                build_directory
                    .canonicalize()
                    .unwrap_or_else(|_| build_directory.clone()),
            ],
            build_directory,
            packages: BTreeMap::new(),
            started: SystemTime::now(),
            stale: false,
            build_outputs: configuration.build_macros,
        };
        for package in metadata["packages"].as_array().into_iter().flatten() {
            let manifest = package["manifest_path"].as_str().ok_or_else(|| {
                Error::Unavailable("Cargo package omitted its manifest path".into())
            })?;
            let root = Path::new(manifest).parent().ok_or(Error::Stale)?;
            if fence.excluded(&root.canonicalize()?) {
                return Err(Error::Unavailable("Cargo output exclusions contain package sources; preparation input freshness is unknown".into()));
            }
            fence.watch_tree(root, false)?;
            if let Some(name) = package["name"].as_str() {
                fence
                    .packages
                    .entry(name.to_owned())
                    .or_default()
                    .push(root.to_owned());
            }
        }
        fence.check()?;
        Ok(fence)
    }
    fn excluded(&self, path: &Path) -> bool {
        self.excluded.iter().any(|root| path.starts_with(root))
            || path.components().any(|part| part.as_os_str() == ".git")
    }
    fn watch_tree(&mut self, path: &Path, external: bool) -> Result<()> {
        self.watch_input(path, external, true)
    }
    fn watch_input(&mut self, path: &Path, external: bool, recursive: bool) -> Result<()> {
        let path = path.canonicalize()?;
        if self.excluded(&path) {
            return if external {
                Err(Error::Unavailable(
                    "build-script rerun input overlaps Cargo/VCS output; freshness is unknown"
                        .into(),
                ))
            } else {
                Ok(())
            };
        }
        let new_watch = self.visited.insert(path.clone());
        if !new_watch && (!recursive || self.expanded.contains(&path)) {
            return Ok(());
        }
        if new_watch {
            let wd =
                inotify::add_watch(&self.fd, &path, watch_flags()).map_err(std::io::Error::from)?;
            self.directories.insert(wd, path.clone());
        }
        let metadata = std::fs::metadata(&path)?;
        // Inputs may change before their individual watch is installed, including while
        // the initial watch set is being built. Quiescence never discards that uncertainty.
        if metadata.modified()? >= self.started {
            self.stale = true;
        }
        if metadata.is_dir() && recursive {
            self.expanded.insert(path.clone());
            for entry in std::fs::read_dir(&path)? {
                let path = entry?.path();
                if self.excluded(&path) {
                    continue;
                }
                let metadata = std::fs::symlink_metadata(&path)?;
                if metadata.is_dir() || metadata.is_symlink() {
                    self.watch_tree(&path, external)?;
                } else if metadata.modified()? >= self.started {
                    self.stale = true;
                }
            }
        }
        Ok(())
    }
    fn check(&mut self) -> Result<()> {
        match native_changes(&self.fd, &self.directories, |path| !self.excluded(path)) {
            Ok(changed) => self.stale |= changed,
            Err(error) => {
                self.stale = true;
                return Err(error);
            }
        }
        if self.stale {
            Err(Error::Stale)
        } else {
            Ok(())
        }
    }
    fn seal(&mut self) -> Result<()> {
        self.check()?;
        // Restricted immutable snapshots never build; their intermediate/output
        // directory is legitimately absent. Keep source fencing without requiring
        // or discovering build-script products that this context did not execute.
        if !self.build_outputs {
            return Ok(());
        }
        // Cargo's native build-script output owns declared dynamic rerun inputs. Discovery
        // happens once at explicit preparation, never as a query-side target/source scan.
        // Cargo 1.101's pinned intermediate layout is package/hash/run/stdout. Native
        // metadata owns both output roots; target triples can come from Cargo config.
        let mut build_roots = vec![self.build_directory.join("debug/build")];
        for entry in std::fs::read_dir(&self.build_directory)? {
            let path = entry?.path();
            if path.is_dir() {
                build_roots.push(path.join("debug/build"));
            }
        }
        for root in build_roots {
            let entries = match std::fs::read_dir(root) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            for entry in entries {
                let path = entry?.path();
                let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                let Some(packages) = self.packages.get(name).cloned() else {
                    continue;
                };
                for unit in std::fs::read_dir(path)? {
                    let output = match std::fs::read_to_string(unit?.path().join("run/stdout")) {
                        Ok(output) => output,
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                        Err(error) => return Err(error.into()),
                    };
                    for line in output.lines() {
                        if let Some(input) = line
                            .strip_prefix("cargo::rerun-if-changed=")
                            .or_else(|| line.strip_prefix("cargo:rerun-if-changed="))
                        {
                            let [package] = packages.as_slice() else {
                                return Err(Error::Unavailable("Cargo build-script output has ambiguous package versions; declared input freshness is unknown".into()));
                            };
                            let input = package.join(input);
                            // Missing inputs are watched through the nearest existing ancestor.
                            let mut watch = input.as_path();
                            while !watch.exists() {
                                watch = watch.parent().ok_or(Error::Stale)?;
                            }
                            self.watch_input(watch, true, input.exists())?;
                        }
                    }
                }
            }
        }
        self.check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn effective_configuration_watch_retains_transient_edits_and_absent_path_creation() {
        let scratch = tempfile::tempdir().unwrap();
        let manifest = scratch.path().join("Cargo.toml");
        std::fs::write(&manifest, "original").unwrap();
        let config = scratch.path().join(".cargo/config.toml");
        let paths = BTreeSet::from([manifest.clone(), config.clone()]);
        let mut fence = ConfigurationWatch::capture(&paths).unwrap();
        std::fs::write(scratch.path().join("unrelated-output"), "output").unwrap();
        fence.check().unwrap();
        std::fs::write(&manifest, "transient").unwrap();
        std::fs::write(&manifest, "original").unwrap();
        assert!(matches!(fence.check(), Err(Error::Stale)));
        let mut fence = ConfigurationWatch::capture(&paths).unwrap();
        std::fs::create_dir(config.parent().unwrap()).unwrap();
        std::fs::write(config, "[build]\n").unwrap();
        assert!(matches!(fence.check(), Err(Error::Stale)));
    }

    #[test]
    fn preparation_changes_before_sealing_are_never_discarded() {
        let scratch = tempfile::tempdir().unwrap();
        let package = scratch.path().join("package");
        std::fs::create_dir(&package).unwrap();
        let source = package.join("build.rs");
        std::fs::write(&source, "fn main() {}\n").unwrap();
        let metadata = json!({"target_directory": scratch.path().join("target"), "build_directory": scratch.path().join("target"), "packages":[{
            "name":"control", "manifest_path":package.join("Cargo.toml")
        }]});
        let configuration = Configuration {
            backend: super::super::Backend::Lsp,
            project: package,
            features: vec![],
            no_default_features: false,
            target: None,
            build_macros: true,
        };
        let mut fence = Preparation::capture(&metadata, &configuration).unwrap();
        std::fs::write(&source, "fn main() { /* concurrent edit */ }\n").unwrap();
        assert!(matches!(fence.seal(), Err(Error::Stale)));
        assert!(matches!(fence.check(), Err(Error::Stale)));
    }

    #[test]
    fn native_rerun_inputs_extend_the_fixed_fence_and_target_writes_do_not() {
        let scratch = tempfile::tempdir().unwrap();
        let package = scratch.path().join("package");
        std::fs::create_dir(&package).unwrap();
        let target = package.join("target");
        std::fs::create_dir(&target).unwrap();
        let build = package.join("build-cache");
        let output = build.join("debug/build/control/1234/run");
        std::fs::create_dir_all(&output).unwrap();
        let input = scratch.path().join("shared-input");
        std::fs::write(&input, "first").unwrap();
        std::fs::write(
            output.join("stdout"),
            "cargo::rerun-if-changed=../shared-input\n",
        )
        .unwrap();
        let metadata = json!({"target_directory":target,"build_directory":build,"packages":[{
            "name":"control", "manifest_path":package.join("Cargo.toml")
        }]});
        let configuration = Configuration {
            backend: super::super::Backend::Lsp,
            project: package,
            features: vec![],
            no_default_features: false,
            target: None,
            build_macros: true,
        };
        let mut fence = Preparation::capture(&metadata, &configuration).unwrap();
        std::fs::write(target.join("ordinary-output"), "compiler output").unwrap();
        std::fs::write(build.join("ordinary-output"), "intermediate output").unwrap();
        fence.seal().unwrap();
        std::fs::write(input, "changed outside package").unwrap();
        assert!(matches!(fence.check(), Err(Error::Stale)));
    }

    #[test]
    fn missing_rerun_input_does_not_mask_existing_directory_coverage() {
        let scratch = tempfile::tempdir().unwrap();
        let package = scratch.path().join("package");
        std::fs::create_dir(&package).unwrap();
        let build = package.join("target");
        let output = build.join("debug/build/control/1234/run");
        std::fs::create_dir_all(&output).unwrap();
        let shared = scratch.path().join("shared/nested");
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::write(shared.join("input"), "first").unwrap();
        std::fs::write(
            output.join("stdout"),
            "cargo::rerun-if-changed=../shared/missing\ncargo::rerun-if-changed=../shared\n",
        )
        .unwrap();
        let metadata = json!({"target_directory":build,"build_directory":build,"packages":[{
            "name":"control", "manifest_path":package.join("Cargo.toml")
        }]});
        let configuration = Configuration {
            backend: super::super::Backend::Lsp,
            project: package,
            features: vec![],
            no_default_features: false,
            target: None,
            build_macros: true,
        };
        let mut fence = Preparation::capture(&metadata, &configuration).unwrap();
        fence.seal().unwrap();
        std::fs::write(shared.join("input"), "changed nested input").unwrap();
        assert!(matches!(fence.check(), Err(Error::Stale)));
    }

    #[test]
    fn output_roots_cannot_silently_exclude_package_sources() {
        let scratch = tempfile::tempdir().unwrap();
        let package = scratch.path().join("package");
        std::fs::create_dir(&package).unwrap();
        let metadata = json!({"target_directory":scratch.path(),"build_directory":scratch.path(),"packages":[{
            "name":"control", "manifest_path":package.join("Cargo.toml")
        }]});
        let configuration = Configuration {
            backend: super::super::Backend::Lsp,
            project: package,
            features: vec![],
            no_default_features: false,
            target: None,
            build_macros: true,
        };
        assert!(matches!(
            Preparation::capture(&metadata, &configuration),
            Err(Error::Unavailable(_))
        ));
    }
}
