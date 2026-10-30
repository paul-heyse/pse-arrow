// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native compatibility rules over two original, explicitly selected API artifacts.
use super::{Artifact, Context};
use crate::{HostError, diagnostic_tools, environment::Environment};
use serde::Serialize;
use std::{
    ffi::OsString,
    fs,
    os::unix::fs::DirBuilderExt as _,
    path::{Path, PathBuf},
};

/// Native performing inputs plus preserved comparison premises.
#[derive(Debug)]
pub struct Prepared {
    /// Exact verified optional tool.
    pub executable: PathBuf,
    /// Explicit existing-file inputs; neither registry lookup nor rustdoc generation is allowed.
    pub arguments: Vec<OsString>,
    /// Original and comparison context evidence for the managed invocation.
    pub artifacts: Vec<crate::envelope::ArtifactReference>,
    /// Exact native rule provider identity.
    pub version: String,
    /// Verified executable content.
    pub executable_sha256: String,
}
#[derive(Serialize)]
struct Premises<'a> {
    version: u32,
    provider: &'a str,
    provider_version: &'a str,
    release_policy: &'a str,
    baseline: &'a Context,
    baseline_sha256: &'a str,
    current: &'a Context,
    current_sha256: &'a str,
    differing_context_fields: Vec<&'static str>,
    limitations: [&'static str; 3],
}
/// Report extraction mismatches before interpreting compatibility findings.
pub fn differing_context_fields(before: &Context, after: &Context) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if before.target != after.target {
        fields.push("target");
    }
    if before.features != after.features {
        fields.push("features");
    }
    if before.rustdoc_identity != after.rustdoc_identity {
        fields.push("rustdoc_identity");
    }
    if before.extraction_flags != after.extraction_flags {
        fields.push("extraction_flags");
    }
    if before.rustdoc_format != after.rustdoc_format {
        fields.push("rustdoc_format");
    }
    if before.environment_sha256 != after.environment_sha256 {
        fields.push("environment_sha256");
    }
    fields
}
/// Freeze checked originals before a separate native reader opens them. Installation
/// is an independent explicit operation; this does not install, fetch, compile or update.
pub fn prepare(
    root: &Path,
    store: &Path,
    baseline: &Artifact,
    current: &Artifact,
    environment: &Environment,
) -> Result<Prepared, HostError> {
    let tool = diagnostic_tools::Tool::CargoSemverChecks;
    let executable = diagnostic_tools::verify(root, tool, environment).map_err(|error| {
        HostError::Ownership(format!(
            "{error}; explicitly prepare with pse diagnostic-tools install cargo-semver-checks"
        ))
    })?;
    let version = diagnostic_tools::tool_spec(root, tool)?.version;
    let executable_sha256 = diagnostic_tools::digest(&executable)?;
    let directory = store
        .join("api-comparisons")
        .join(uuid::Uuid::now_v7().to_string());
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&directory)?;
    let copy = |artifact: &Artifact, name: &str| -> Result<PathBuf, HostError> {
        let bytes = crate::test_inventory::read_private(&artifact.path)?;
        if crate::probe::digest(&bytes) != artifact.sha256 {
            return Err(HostError::Ownership(
                "API input changed before comparison capture".into(),
            ));
        }
        let path = directory.join(name);
        super::write(&path, &bytes).map_err(|error| HostError::Api(Box::new(error)))?;
        Ok(path)
    };
    let old = copy(baseline, "baseline.json")?;
    let new = copy(current, "current.json")?;
    let context_path = directory.join("comparison.json");
    let context = serde_json::to_vec_pretty(&Premises {
        version: 1, provider: tool.name(), provider_version: &version, release_policy: "patch",
        baseline: &baseline.context, baseline_sha256: &baseline.sha256,
        current: &current.context, current_sha256: &current.sha256,
        differing_context_fields: differing_context_fields(&baseline.context, &current.context),
        limitations: [
            "native compatibility findings are not a complete behavioral compatibility verdict",
            "rule findings and actual tool exit are original retained stdout/stderr; no textual rule parsing or bespoke SemVer rules",
            "public-api 0.52.2 uses rustdoc-types 0.59 and cannot establish format-61 structural additions; inspect addressed declarations separately",
        ],
    }).map_err(std::io::Error::other)?;
    super::write(&context_path, &context).map_err(|error| HostError::Api(Box::new(error)))?;
    Ok(Prepared {
        executable,
        version,
        executable_sha256,
        arguments: vec![
            "--current-rustdoc".into(),
            new.as_os_str().to_owned(),
            "--baseline-rustdoc".into(),
            old.as_os_str().to_owned(),
            "--release-type".into(),
            "patch".into(),
            "--color".into(),
            "never".into(),
        ],
        artifacts: vec![
            crate::envelope::ArtifactReference {
                kind: "api_comparison_context".into(),
                location: context_path.display().to_string(),
                digest: Some(crate::probe::digest(&context)),
            },
            crate::envelope::ArtifactReference {
                kind: "api_comparison_baseline".into(),
                location: old.display().to_string(),
                digest: Some(baseline.sha256.clone()),
            },
            crate::envelope::ArtifactReference {
                kind: "api_comparison_current".into(),
                location: new.display().to_string(),
                digest: Some(current.sha256.clone()),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    fn context() -> Context {
        let build = crate::generation::BuildContext::new(
            "test rustc".into(),
            "test cargo".into(),
            "dev".into(),
            BTreeSet::new(),
            &Environment::new(),
        );
        let source = serde_json::from_value(serde_json::json!({
            "version":1,"package_roots":[],"additional_files":[],"files":{},"context":build
        }))
        .unwrap();
        Context {
            version: 1,
            package_id: "fixture@1.0.0".into(),
            manifest: "/fixture/Cargo.toml".into(),
            package_source_sha256: "original source".into(),
            metadata_sha256: "original metadata".into(),
            features: BTreeSet::new(),
            metadata_features: BTreeSet::new(),
            unit_graph_sha256: "original graph".into(),
            source,
            target: "x86_64-unknown-linux-gnu".into(),
            extraction_flags: vec![],
            rustdoc_format: 61,
            rustdoc_identity: "test rustdoc".into(),
            environment_sha256: "original environment".into(),
        }
    }
    fn document() -> rustdoc_types::Crate {
        let id = rustdoc_types::Id(0);
        rustdoc_types::Crate {
            root: id,
            crate_version: Some("1.0.0".into()),
            includes_private: false,
            index: [(
                id,
                rustdoc_types::Item {
                    id,
                    crate_id: 0,
                    name: Some("fixture".into()),
                    span: None,
                    visibility: rustdoc_types::Visibility::Public,
                    docs: None,
                    links: Default::default(),
                    attrs: vec![],
                    deprecation: None,
                    stability: None,
                    const_stability: None,
                    inner: rustdoc_types::ItemEnum::Module(rustdoc_types::Module {
                        is_crate: true,
                        items: vec![],
                        is_stripped: false,
                    }),
                },
            )]
            .into_iter()
            .collect(),
            paths: Default::default(),
            external_crates: Default::default(),
            target: rustdoc_types::Target {
                triple: "x86_64-unknown-linux-gnu".into(),
                target_features: vec![],
            },
            format_version: 61,
        }
    }
    #[test]
    fn historical_artifact_checks_original_context_digest_and_target_before_use() {
        let directory = tempfile::tempdir().unwrap();
        let context = context();
        let bytes = serde_json::to_vec(&document()).unwrap();
        let record = super::super::ArtifactRecord {
            context_key: context.key().unwrap(),
            sha256: crate::probe::digest(&bytes),
        };
        super::super::write(
            &directory.path().join("context.json"),
            &serde_json::to_vec(&context).unwrap(),
        )
        .unwrap();
        super::super::write(
            &directory.path().join("artifact.json"),
            &serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
        let path = directory.path().join("document.json");
        super::super::write(&path, &bytes).unwrap();
        let selected = super::super::retained(&path).unwrap();
        assert_eq!(selected.context, context);
        assert_eq!(selected.sha256, record.sha256);
        assert!(differing_context_fields(&context, &selected.context).is_empty());
        let mut changed = context.clone();
        changed.features.insert("changed".into());
        changed.target = "aarch64-unknown-linux-gnu".into();
        assert_eq!(
            differing_context_fields(&context, &changed),
            ["target", "features"]
        );
        fs::remove_file(&path).unwrap();
        super::super::write(&path, b"changed original bytes").unwrap();
        assert!(
            super::super::retained(&path)
                .unwrap_err()
                .to_string()
                .contains("original bytes")
        );
    }
}
