// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Request-owned proof that retained documentation spans nominate exact current source.
use super::{ApiError, Artifact, invalid};
use crate::{
    environment::Environment,
    semantic::{Configuration, NativePackageContext},
    source_read::SourceText,
};
use std::{
    collections::BTreeMap,
    ops::Range,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Verify an artifact and source universe once, then map requested original item spans.
#[derive(Debug)]
pub struct SourceBridge {
    artifact: Arc<Artifact>,
    root: PathBuf,
    package_root: PathBuf,
    native: NativePackageContext,
    routes: super::query::SourceRoutes,
    files: BTreeMap<PathBuf, String>,
    sources: Mutex<BTreeMap<PathBuf, SourceText>>,
    qualification: Mutex<super::query::QualificationContext>,
    dependency_sources: BTreeMap<PathBuf, String>,
}
impl SourceBridge {
    /// Check an external declaration against this same original public surface.
    pub(crate) fn validate_definition(
        &self,
        item: &str,
        evidence: &super::query::DefinitionEvidence,
        definition: &Artifact,
    ) -> Result<(), ApiError> {
        self.qualification
            .lock()
            .map_err(|_| invalid("API qualification owner panicked"))?
            .validate(item, evidence, definition)
    }
    /// Compare captured Cargo/HIR context from the same loaded native world; never run Cargo.
    pub fn prepare(
        artifact: Arc<Artifact>,
        configuration: &Configuration,
        native_packages: &[NativePackageContext],
        environment: &Environment,
    ) -> Result<Self, ApiError> {
        let original = super::retained(&artifact.path)?;
        if original.sha256 != artifact.sha256 || original.context != artifact.context {
            return Err(invalid("API artifact or acquisition context changed"));
        }
        let root = artifact.context.source_root.canonicalize()?;
        let project = configuration.project.canonicalize()?;
        let manifest = artifact.context.manifest.canonicalize()?;
        let qualification = super::query::QualificationContext::new(&artifact)?;
        let closure = qualification.closure();
        let mut matching = Vec::new();
        for native in native_packages {
            if native.package_id != artifact.context.package_id
                || native.features != artifact.context.features
                || native.target.as_deref() != Some(artifact.context.target.as_str())
                || native.role != closure.root.role
            {
                continue;
            }
            if native.manifest.canonicalize()? != manifest
                || native.library_root.canonicalize()? != closure.root.library_root
            {
                continue;
            }
            if !project.starts_with(native.workspace_root.canonicalize()?) {
                continue;
            }
            matching.push(native);
        }
        let [native] = matching.as_slice() else {
            return Err(invalid(
                "API package/library/features/target has no unique equivalent profile in the selected native world",
            ));
        };
        let native = (*native).clone();
        verify_dependency_profiles(closure, &native)?;
        let dependency_sources = verify_dependency_sources(&artifact.context, &native)?;
        let environment = super::extraction_environment(&root, environment)?;
        if !artifact
            .context
            .source
            .recorded_environment_matches(&environment)
        {
            return Err(invalid(
                "API build-input environment differs from the selected working environment",
            ));
        }
        if !artifact
            .context
            .source
            .recorded_sources_match(&root)
            .map_err(|error| invalid(error.to_string()))?
        {
            return Err(invalid(
                "API acquisition source/configuration inputs changed",
            ));
        }
        let package_root = manifest
            .parent()
            .ok_or_else(|| invalid("API package manifest has no parent"))?
            .to_path_buf();
        if native.source_root.canonicalize()? != package_root {
            return Err(invalid(
                "native package source root does not match the original API package",
            ));
        }
        let mut files = Vec::new();
        super::source_files(&package_root, Path::new(""), &mut files)?;
        if super::source_files_digest(&files) != artifact.context.package_source_sha256 {
            return Err(invalid("API package source bytes or membership changed"));
        }
        let files = files
            .into_iter()
            .map(|(path, bytes)| (package_root.join(path), crate::probe::digest(&bytes)))
            .collect();
        let routes = super::query::SourceRoutes::new(&artifact.document);
        Ok(Self {
            artifact,
            root,
            package_root,
            native,
            routes,
            files,
            sources: Mutex::new(BTreeMap::new()),
            qualification: Mutex::new(qualification),
            dependency_sources,
        })
    }
    /// Nominate only the artifact's original source span; native resolution establishes identity.
    pub fn source(
        &self,
        item: &str,
        public_path: Option<&str>,
    ) -> Result<(SourceText, Range<usize>), ApiError> {
        let id = rustdoc_types::Id(
            item.parse()
                .map_err(|_| invalid("API item ID is not an artifact-local integer"))?,
        );
        self.routes.validate(id, public_path)?;
        let declaration = self
            .artifact
            .document
            .index
            .get(&id)
            .ok_or_else(|| invalid("API item is absent from its retained artifact"))?;
        let direct = declaration.span.as_ref().filter(|span| {
            self.root
                .join(&span.filename)
                .starts_with(&self.package_root)
        });
        let span = if let Some(span) = direct {
            span
        } else {
            self.routes
                .reexport_span(&self.artifact.document, id, public_path)?
                .ok_or_else(|| {
                    invalid("API declaration has no exact original package source/reexport span")
                })?
        };
        let path = self.root.join(&span.filename).canonicalize()?;
        if !path.starts_with(&self.package_root) {
            return Err(invalid(
                "API span is not an original file in the captured package",
            ));
        }
        let mut sources = self
            .sources
            .lock()
            .map_err(|_| invalid("API source bridge cache owner panicked"))?;
        if let std::collections::btree_map::Entry::Vacant(entry) = sources.entry(path.clone()) {
            let source = SourceText::new(path.clone(), std::fs::read_to_string(&path)?)
                .map_err(|error| invalid(error.to_string()))?;
            if self
                .files
                .get(&path)
                .is_none_or(|hash| hash != source.identity().source_sha256())
            {
                return Err(invalid(
                    "API span source differs from its verified original source snapshot",
                ));
            }
            if self
                .native
                .source_hashes
                .get(&path)
                .is_none_or(|hash| hash != source.identity().source_sha256())
            {
                return Err(invalid(
                    "API span bytes are not identical to the selected native world's captured source",
                ));
            }
            entry.insert(source);
        }
        let source = sources
            .get(&path)
            .ok_or_else(|| invalid("verified API source snapshot absent"))?;
        let start = offset(source.text(), span.begin)?;
        let end = offset(source.text(), span.end)?;
        if end < start {
            return Err(invalid("API source span is reversed"));
        }
        Ok((source.clone(), start..end))
    }
    /// A late mutation invalidates final coverage after any earlier positive nominations.
    pub fn verify_unchanged(&self, environment: &Environment) -> Result<(), ApiError> {
        self.qualification
            .lock()
            .map_err(|_| invalid("API qualification owner panicked"))?
            .verify_unchanged()?;
        for (root, expected) in &self.dependency_sources {
            let mut files = Vec::new();
            super::source_files(root, Path::new(""), &mut files)?;
            if super::source_files_digest(&files) != *expected {
                return Err(invalid(
                    "original dependency package bytes changed during native refinement",
                ));
            }
        }
        let artifact = super::retained(&self.artifact.path)?;
        if artifact.sha256 != self.artifact.sha256 || artifact.context != self.artifact.context {
            return Err(invalid("API artifact/context changed during refinement"));
        }
        let environment = super::extraction_environment(&self.root, environment)?;
        if !self
            .artifact
            .context
            .source
            .recorded_environment_matches(&environment)
            || !self
                .artifact
                .context
                .source
                .recorded_sources_match(&self.root)
                .map_err(|error| invalid(error.to_string()))?
        {
            return Err(invalid(
                "API source/configuration changed during refinement",
            ));
        }
        let mut files = Vec::new();
        super::source_files(&self.package_root, Path::new(""), &mut files)?;
        if super::source_files_digest(&files) != self.artifact.context.package_source_sha256 {
            return Err(invalid("API package source changed during refinement"));
        }
        Ok(())
    }
}

fn verify_dependency_sources(
    context: &super::Context,
    native: &NativePackageContext,
) -> Result<BTreeMap<PathBuf, String>, ApiError> {
    let mut roots = BTreeMap::new();
    // Sysroot crates were explicitly excluded by the native Cargo projection; only
    // actual captured Cargo dependency roots carry these package-byte obligations.
    for dependency in &native.dependencies {
        let Some(super::PackageSourceEvidence::Recorded { sha256 }) =
            context.dependency_sources.get(&dependency.package_id)
        else {
            return Err(invalid(
                "original dependency package bytes were not recorded; native context equivalence remains unknown",
            ));
        };
        let root = dependency.source_root.canonicalize()?;
        if let Some(previous) = roots.get(&root) {
            if previous != sha256 {
                return Err(invalid(
                    "native dependency root has ambiguous original source evidence",
                ));
            }
            continue;
        }
        let mut files = Vec::new();
        super::source_files(&root, Path::new(""), &mut files)?;
        if super::source_files_digest(&files) != *sha256 {
            return Err(invalid(
                "original documentation dependency bytes differ from the selected native world's source root",
            ));
        }
        roots.insert(root, sha256.clone());
    }
    Ok(roots)
}

fn verify_dependency_profiles(
    closure: &super::DocumentationClosure,
    native: &NativePackageContext,
) -> Result<(), ApiError> {
    if !native.dependency_issues.is_empty() {
        return Err(invalid(format!(
            "native dependency profile closure is incomplete: {}",
            native.dependency_issues.join("; ")
        )));
    }
    let mut actual = std::collections::BTreeSet::new();
    for profile in &native.dependencies {
        if profile.role != "lib" {
            return Err(invalid(
                "native host/proc-macro/test dependency profile cannot establish ordinary API library equivalence",
            ));
        }
        let target = profile
            .target
            .clone()
            .ok_or_else(|| invalid("native dependency target is not captured"))?;
        actual.insert(super::DocumentationLibraryProfile {
            package_id: profile.package_id.clone(),
            library_root: profile.library_root.canonicalize()?,
            features: profile.features.clone(),
            target,
            role: profile.role.clone(),
        });
    }
    // Builtin sysroot exclusions are explicitly typed by the native producer. They are not
    // Cargo package units. A Cargo-declared std/build-std unit remains in the expected set
    // and cannot disappear through a crate-name shortcut here.
    if actual != closure.dependencies {
        let missing = closure.dependencies.difference(&actual).collect::<Vec<_>>();
        let additional = actual.difference(&closure.dependencies).collect::<Vec<_>>();
        return Err(invalid(format!(
            "API transitive Cargo library profiles differ from the selected native world; original-only={missing:?}; native-only={additional:?}"
        )));
    }
    Ok(())
}

fn offset(text: &str, (line, column): (usize, usize)) -> Result<usize, ApiError> {
    if line == 0 || column == 0 {
        return Err(invalid("rustdoc source coordinates must be one-based"));
    }
    let mut start = 0;
    for (index, part) in text.split_inclusive('\n').enumerate() {
        if index + 1 == line {
            let offset = part
                .char_indices()
                .map(|(offset, _)| offset)
                .chain(std::iter::once(part.len()))
                .nth(column - 1)
                .ok_or_else(|| invalid("API source column is outside its original line"))?;
            return Ok(start + offset);
        }
        start += part.len();
    }
    if line == text.bytes().filter(|byte| *byte == b'\n').count() + 1 && column == 1 {
        return Ok(text.len());
    }
    Err(invalid("API source line is outside its original input"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn acquired_cargo_documentation_closure_retains_deep_effective_feature_profiles() {
        use std::{fs, process::Command};
        let sandbox = tempfile::tempdir().unwrap();
        let root = sandbox.path().join("a");
        let state = sandbox.path().join("state");
        for path in ["src", ".cargo", "middle/src", "middle/leaf/src"] {
            fs::create_dir_all(root.join(path)).unwrap();
        }
        fs::write(root.join("Cargo.toml"),"[package]\nname='profile-a'\nversion='0.1.0'\nedition='2024'\n[workspace]\nexclude=['middle']\n[dependencies]\nprofile-middle={path='middle'}\n").unwrap();
        fs::write(root.join("middle/Cargo.toml"),"[package]\nname='profile-middle'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nprofile-leaf={path='leaf',default-features=false}\n").unwrap();
        fs::write(root.join("middle/leaf/Cargo.toml"),"[package]\nname='profile-leaf'\nversion='0.1.0'\nedition='2024'\n[features]\nwide=[]\n").unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn run() -> profile_middle::Wire { profile_middle::run() }\n",
        )
        .unwrap();
        fs::write(
            root.join("middle/src/lib.rs"),
            "pub use profile_leaf::Wire; pub fn run() -> Wire { 1 }\n",
        )
        .unwrap();
        fs::write(root.join("middle/leaf/src/lib.rs"),"#[cfg(feature=\"wide\")] pub type Wire=u64; #[cfg(not(feature=\"wide\"))] pub type Wire=u8;\n").unwrap();
        fs::write(root.join("Cargo.lock"),"version=4\n[[package]]\nname=\"profile-a\"\nversion=\"0.1.0\"\ndependencies=[\"profile-middle\"]\n[[package]]\nname=\"profile-middle\"\nversion=\"0.1.0\"\ndependencies=[\"profile-leaf\"]\n[[package]]\nname=\"profile-leaf\"\nversion=\"0.1.0\"\n").unwrap();
        fs::write(root.join(".cargo/config.toml"), "").unwrap();
        fs::write(
            root.join("rust-toolchain.toml"),
            include_str!("../../../../rust-toolchain.toml"),
        )
        .unwrap();
        let environment = std::env::vars_os().collect();
        let snapshot = crate::dependency::capture(&root, &environment).unwrap();
        let acquisition =
            super::super::prepare(&root, &state, &snapshot, "profile-a", None, &environment)
                .unwrap();
        let output = Command::new(&acquisition.command[0])
            .args(&acquisition.command[1..])
            .current_dir(&acquisition.cwd)
            .env_clear()
            .envs(&acquisition.environment)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let artifact = super::super::finish(&acquisition, &snapshot).unwrap();
        let qualification = super::super::query::QualificationContext::new(&artifact).unwrap();
        let closure = qualification.closure();
        let id = |name: &str| {
            snapshot
                .metadata
                .packages
                .iter()
                .find(|package| package.name == name)
                .unwrap()
                .id
                .to_string()
        };
        assert_eq!(closure.root.package_id, id("profile-a"));
        assert_eq!(closure.dependencies.len(), 2);
        for (name, path) in [
            ("profile-middle", "middle/src/lib.rs"),
            ("profile-leaf", "middle/leaf/src/lib.rs"),
        ] {
            let profile = closure
                .dependencies
                .iter()
                .find(|profile| profile.package_id == id(name))
                .unwrap();
            assert!(profile.features.is_empty());
            assert_eq!(profile.library_root, root.join(path));
            assert_eq!(profile.target, artifact.context.target);
            assert_eq!(profile.role, "lib");
        }
    }
    #[test]
    fn native_rustdoc_spans_keep_one_based_unicode_scalar_columns() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("lib.rs");
        let text = "// αβ\n\npub fn unicode_é() {}\nmod inner { /* λ */ pub fn later() {} }\nmod hidden { pub fn real() {} }\npub use hidden::real as Exported;\npub use hidden::real as Other;\n";
        std::fs::write(&path, text).unwrap();
        let output = std::process::Command::new("rustdoc")
            .args([
                "-Z",
                "unstable-options",
                "--output-format",
                "json",
                "--document-private-items",
                "--crate-name",
                "span_probe",
                "--edition",
                "2024",
            ])
            .arg(&path)
            .arg("-o")
            .arg(root.path().join("doc"))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let document =
            super::super::decode(&std::fs::read(root.path().join("doc/span_probe.json")).unwrap())
                .unwrap();
        let later = document
            .index
            .values()
            .find(|item| item.name.as_deref() == Some("later"))
            .unwrap()
            .span
            .as_ref()
            .unwrap();
        assert_eq!(later.begin, (4, 21));
        let range = offset(text, later.begin).unwrap()..offset(text, later.end).unwrap();
        assert_eq!(&text[range], "pub fn later() {}");
        let unicode = document
            .index
            .values()
            .find(|item| item.name.as_deref() == Some("unicode_é"))
            .unwrap()
            .span
            .as_ref()
            .unwrap();
        let range = offset(text, unicode.begin).unwrap()..offset(text, unicode.end).unwrap();
        assert_eq!(&text[range], "pub fn unicode_é() {}");
        assert!(offset(text, (1, 0)).is_err());
        assert!(offset(text, (99, 1)).is_err());
        assert_eq!(offset("αβ", (1, 1)).unwrap(), 0);
        assert_eq!(offset("αβ", (1, 2)).unwrap(), 2);
        assert_eq!(offset("αβ", (1, 3)).unwrap(), 4);
        assert!(offset("αβ", (1, 4)).is_err());
        assert!(offset("αβ", (0, 1)).is_err());
        assert_eq!(offset("α\n", (2, 1)).unwrap(), 3);
        let real = document
            .index
            .values()
            .find(|item| item.name.as_deref() == Some("real"))
            .unwrap();
        let routes = super::super::query::SourceRoutes::new(&document);
        for alias in ["Exported", "Other"] {
            let span = routes
                .reexport_span(&document, real.id, Some(&format!("span_probe::{alias}")))
                .unwrap()
                .unwrap();
            let range = offset(text, span.begin).unwrap()..offset(text, span.end).unwrap();
            assert_eq!(&text[range], format!("pub use hidden::real as {alias};"));
        }
        assert!(routes.reexport_span(&document, real.id, None).is_err());
        assert!(
            routes
                .validate(real.id, Some("span_probe::not_exported"))
                .is_err()
        );
    }
}
