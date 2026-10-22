// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Versioned conformance campaign documents; fixture policy remains authored authority.
use super::{PreparationSettings, WorkflowError, contract};
use pse_model::{document::Version, generated::enums::{ConformanceExecution, ConformancePresolve, NativeBackend, NativeSolveIntent}};
use serde::{Serialize, Deserialize};

/// Execution settings shared by all selected authored fixtures in one campaign.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunSettings {
    /// Standalone settings interpretation.
    pub version: Version<1>,
    /// Shared accounted execution-memory capacity.
    pub memory_limit_bytes: usize,
    /// Absence retains the runtime's worker allocation policy.
    pub math_worker_bytes: Option<usize>,
    /// Absence retains the runtime's native-job capacity.
    pub math_jobs: Option<usize>,
    /// Admission-only wait when no enclosing deadline exists.
    pub math_admission_wait_ms: Option<u64>,
    /// Finite preparation allowance, when explicitly supplied.
    pub math_preparation_limit_ms: Option<u64>,
    /// Shared execution threads.
    #[serde(default = "one")]
    pub threads: usize,
    /// Per-fixture elapsed allowance; authored fixture policy remains authoritative.
    #[serde(default = "time_limit")]
    pub time_limit_seconds: f64,
    /// Requested operation class where a fixture supplies no override.
    #[serde(default = "root")]
    pub intent: NativeSolveIntent,
    /// Automatic selection is represented by absence.
    pub backend: Option<NativeBackend>,
    /// Conformance's supported presolve choice.
    #[serde(default = "presolve")]
    pub presolve: ConformancePresolve,
    /// Maximum admitted fixture inventory.
    #[serde(default = "fixtures")]
    pub maximum_fixtures: usize,
    /// Maximum authored checks.
    #[serde(default = "checks")]
    pub maximum_checks: usize,
    /// Complete preparation controls supersede the optional expansion overrides.
    pub preparation: Option<PreparationSettings>,
    /// Optional expansion item bound.
    pub expansion_items: Option<usize>,
    /// Optional expansion member bound.
    pub expansion_members: Option<usize>,
    /// Optional expansion depth bound.
    pub expansion_depth: Option<usize>,
    /// Optional semantic-body occurrence bound.
    pub body_occurrences: Option<usize>,
    /// Optional semantic-body slot bound.
    pub body_slots: Option<usize>,
    /// Maximum derivative diagnostic cells.
    #[serde(default = "cells")]
    pub derivative_cells: usize,
    /// Diagnostic finite-difference perturbation.
    #[serde(default = "step")]
    pub derivative_step: f64,
    /// Diagnostic relative tolerance.
    #[serde(default = "tolerance")]
    pub derivative_tolerance: f64,
    /// Diagnostic threshold profile path, relative to the manifest at ingress.
    pub diagnostics: Option<String>,
}
fn one() -> usize { 1 }
fn time_limit() -> f64 { 600.0 }
fn root() -> NativeSolveIntent { NativeSolveIntent::Root }
fn presolve() -> ConformancePresolve { ConformancePresolve::Auto }
fn fixtures() -> usize { 1024 }
fn checks() -> usize { 16384 }
fn cells() -> usize { 100000 }
fn step() -> f64 { 1e-6 }
fn tolerance() -> f64 { 1e-4 }
fn native() -> ConformanceExecution { ConformanceExecution::Native }
/// A package closure selected once within a conformance campaign.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConformanceRun {
    /// Standalone run-selection interpretation.
    pub version: Version<1>,
    /// Unique file-stem name used for campaign reports.
    pub name: String,
    /// Root package relative to the campaign manifest.
    pub package: String,
    /// Physical source package relative to the manifest.
    pub physical: String,
    /// Ordered dependency package roots.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Pure runs do not start solver services.
    #[serde(default = "native")]
    pub execution: ConformanceExecution,
}
/// One campaign's shared settings and ordered run inventory.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConformanceManifest {
    /// Campaign interpretation, admitted before nested settings.
    pub version: Version<1>,
    /// Settings applied through the ordinary runtime conformance entry.
    pub settings: RunSettings,
    /// Every selected closure is executed once in this order.
    pub runs: Vec<ConformanceRun>,
}
impl ConformanceManifest {
    /// Load a strict current TOML campaign and resolve its diagnostic profile path.
    /// # Errors
    /// Unsupported version, malformed settings, duplicate/invalid names, missing roots
    /// or a missing diagnostic profile. No fixture or solver is started.
    pub fn load(path: &std::path::Path) -> Result<Self, WorkflowError> {
        use std::io::Read as _;
        const MAXIMUM_BYTES: u64 = 16 << 20;
        let file = std::fs::File::open(path).map_err(|error| contract(format!("{}: {error}", path.display())))?;
        let mut bytes = Vec::new();
        file.take(MAXIMUM_BYTES + 1).read_to_end(&mut bytes).map_err(|error| contract(error.to_string()))?;
        if bytes.len() as u64 > MAXIMUM_BYTES { return Err(contract("conformance manifest extent")); }
        let text = std::str::from_utf8(&bytes).map_err(|error| contract(error.to_string()))?;
        #[derive(Deserialize)]
        struct Header { version: u32 }
        let header: Header = toml::from_str(text).map_err(|error| contract(error.to_string()))?;
        if header.version != 1 { return Err(contract(format!("unsupported conformance manifest version {} (current: 1)", header.version))); }
        let manifest: Self = toml::from_str(text).map_err(|error| contract(error.to_string()))?;
        manifest.admit_paths(path)
    }
    /// Admit campaign membership under its explicit manifest-relative source context.
    /// # Errors
    /// Empty/duplicate/invalid names or unavailable source/profile paths.
    pub fn admit_paths(self, path: &std::path::Path) -> Result<Self, WorkflowError> {
        let mut manifest = self;
        let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let mut names = std::collections::BTreeSet::new();
        if manifest.runs.is_empty() { return Err(contract("conformance runs must be nonempty")); }
        for run in &manifest.runs {
            if run.name.is_empty() || !run.name.bytes().all(|ch| ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-') || !names.insert(&run.name) {
                return Err(contract("conformance runs must be uniquely named by file stems"));
            }
            for root in std::iter::once(&run.package).chain(std::iter::once(&run.physical)).chain(&run.dependencies) {
                if !parent.join(root).join("package.toml").is_file() {
                    return Err(contract(format!("run {} names {root}, which has no package.toml", run.name)));
                }
            }
        }
        if let Some(profile) = &manifest.settings.diagnostics {
            let resolved = parent.join(profile);
            if !resolved.is_file() { return Err(contract(format!("diagnostics names {}, which is not a file", resolved.display()))); }
            manifest.settings.diagnostics = Some(resolved.to_str().ok_or_else(|| contract("diagnostic profile path is not UTF-8"))?.into());
        }
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_defaults_and_versions_follow_the_rust_owner() {
        let settings: RunSettings = serde_json::from_str(r#"{"version":1,"memory_limit_bytes":1024}"#).unwrap();
        assert_eq!(settings.threads, 1);
        assert_eq!(settings.time_limit_seconds, 600.0);
        assert_eq!(settings.maximum_checks, 16384);
        assert_eq!(settings.derivative_step, 1e-6);
        assert_eq!(settings.presolve, ConformancePresolve::Auto);
        assert!(settings.preparation.is_none());
        assert!(serde_json::from_str::<RunSettings>(r#"{"memory_limit_bytes":1024}"#).is_err());
        assert!(serde_json::from_str::<RunSettings>(r#"{"version":2,"memory_limit_bytes":1024}"#).is_err());
        assert!(serde_json::from_str::<RunSettings>(r#"{"version":1,"memory_limit_bytes":1024,"unknown":true}"#).is_err());
    }
    #[test]
    fn manifest_refuses_version_before_current_nested_settings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("campaign.toml");
        std::fs::write(&path, "version = 2\n[settings]\nmemory_limit_bytes = 'not current'\n").unwrap();
        let error = ConformanceManifest::load(&path).unwrap_err().to_string();
        assert!(error.contains("unsupported conformance manifest version 2"), "{error}");
    }
    #[test]
    fn manifest_context_admission_preserves_relative_paths_and_names() {
        let directory = tempfile::tempdir().unwrap();
        for name in ["model", "physical"] {
            std::fs::create_dir(directory.path().join(name)).unwrap();
            std::fs::write(directory.path().join(name).join("package.toml"), "").unwrap();
        }
        std::fs::write(directory.path().join("diagnostics.json"), "{}").unwrap();
        let path = directory.path().join("campaign.toml");
        let source = "version = 1\n[settings]\nversion = 1\nmemory_limit_bytes = 1024\ndiagnostics = 'diagnostics.json'\n[[runs]]\nversion = 1\nname = 'first'\npackage = 'model'\nphysical = 'physical'\nexecution = 'pure'\n";
        std::fs::write(&path, source).unwrap();
        let admitted = ConformanceManifest::load(&path).unwrap();
        assert_eq!(admitted.runs[0].execution, ConformanceExecution::Pure);
        assert_eq!(admitted.runs[0].package, "model");
        assert_eq!(admitted.settings.diagnostics.as_deref(), directory.path().join("diagnostics.json").to_str());
        std::fs::write(&path, format!("{source}[[runs]]\nversion = 1\nname = 'first'\npackage = 'model'\nphysical = 'physical'\n")).unwrap();
        assert!(ConformanceManifest::load(&path).unwrap_err().to_string().contains("uniquely"));
        std::fs::write(&path, source.replace("package = 'model'", "package = 'missing'")).unwrap();
        assert!(ConformanceManifest::load(&path).unwrap_err().to_string().contains("package.toml"));
    }
}
