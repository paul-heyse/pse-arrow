// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native capabilities a Cargo selection needs, from its actual unit graph.
//!
//! `pse-backend-native` declares once, in `[package.metadata.pse.native-capabilities]`,
//! which prepared providers each of its features needs. The correctness adapter's validated
//! unit graph reports which of those features a selection activates; package membership alone
//! would overstate the need. The provider implications close the result.
use crate::{
    HostError,
    correctness::{self, Resolution},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
};

/// The package whose features select prepared native providers.
pub const PACKAGE: &str = "pse-backend-native";
const KEY: &str = "native-capabilities";

fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(format!("native capabilities: {}", message.into()))
}
fn close<'a>(names: impl IntoIterator<Item = &'a str>) -> Result<BTreeSet<String>, HostError> {
    let joined = names.into_iter().collect::<Vec<_>>().join(",");
    Ok(super::providers::capabilities(&joined)?
        .into_iter()
        .map(str::to_owned)
        .collect())
}

/// The declared feature-to-capability table of [`PACKAGE`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityTable {
    features: BTreeMap<String, BTreeSet<String>>,
}

impl CapabilityTable {
    /// Read the declaration through no-deps Cargo metadata of the workspace that the same
    /// arguments select (the working directory, or their `--manifest-path`).
    pub fn read(arguments: &[OsString]) -> Result<Self, HostError> {
        let mut command = cargo_metadata::MetadataCommand::new();
        command
            .cargo_path(correctness::real_cargo().map_err(|error| invalid(error.to_string()))?)
            .no_deps()
            .other_options(vec!["--locked".into()]);
        if let Some(path) = manifest_path(arguments)? {
            command.manifest_path(path);
        }
        let metadata = command
            .exec()
            .map_err(|error| invalid(format!("Cargo metadata failed: {error}")))?;
        let package = metadata
            .workspace_packages()
            .into_iter()
            .find(|package| package.name.as_str() == PACKAGE)
            .ok_or_else(|| invalid(format!("workspace has no {PACKAGE} member")))?;
        Self::from_package(&package.features, &package.metadata)
    }

    /// Validate a declaration against the package's features: every feature except
    /// `default` is declared, `[]` meaning no provider, and every capability is known.
    pub fn from_package(
        features: &BTreeMap<String, Vec<String>>,
        metadata: &serde_json::Value,
    ) -> Result<Self, HostError> {
        let table = metadata
            .get("pse")
            .and_then(|pse| pse.get(KEY))
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                invalid(format!(
                    "{PACKAGE} declares no [package.metadata.pse.{KEY}] table"
                ))
            })?;
        let mut result = BTreeMap::new();
        for (feature, value) in table {
            if !features.contains_key(feature) || feature == "default" {
                return Err(invalid(format!(
                    "{feature} is not a declarable {PACKAGE} feature"
                )));
            }
            let names = value
                .as_array()
                .and_then(|names| {
                    names
                        .iter()
                        .map(serde_json::Value::as_str)
                        .collect::<Option<Vec<_>>>()
                })
                .ok_or_else(|| invalid(format!("{feature} must list capability names")))?;
            close(names.iter().copied())?;
            result.insert(
                feature.clone(),
                names.into_iter().map(str::to_owned).collect(),
            );
        }
        if let Some(missing) = features
            .keys()
            .find(|feature| *feature != "default" && !result.contains_key(*feature))
        {
            return Err(invalid(format!(
                "{PACKAGE} feature {missing} has no declaration; declare its capabilities or []"
            )));
        }
        Ok(Self { features: result })
    }

    /// Declared capabilities of one feature, before closure.
    pub fn declared(&self, feature: &str) -> Option<&BTreeSet<String>> {
        self.features.get(feature)
    }

    /// Every declared feature.
    pub fn features(&self) -> impl Iterator<Item = &str> {
        self.features.keys().map(String::as_str)
    }

    /// Join the features a resolved graph activates with the declaration and close them.
    pub fn requirement(&self, resolution: &Resolution) -> Result<NativeRequirement, HostError> {
        let features: BTreeSet<String> = resolution
            .activated
            .get(PACKAGE)
            .into_iter()
            .flatten()
            .filter(|feature| *feature != "default")
            .cloned()
            .collect();
        let mut names = BTreeSet::new();
        for feature in &features {
            names.extend(
                self.features
                    .get(feature)
                    .ok_or_else(|| invalid(format!("activated {feature} has no declaration")))?
                    .iter()
                    .map(String::as_str),
            );
        }
        Ok(NativeRequirement {
            capabilities: close(names)?,
            features,
        })
    }
}

/// Prepared native providers a selection needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeRequirement {
    /// [`PACKAGE`] features the selection's unit graph activates.
    pub features: BTreeSet<String>,
    /// Closed capability set: prepare and select exactly these.
    pub capabilities: BTreeSet<String>,
}

impl NativeRequirement {
    /// The `--native=` / `native prepare --capabilities` value; empty selects none.
    pub fn profile(&self) -> String {
        self.capabilities
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Derive the native requirement of a Cargo compiler selection without compiling it.
///
/// `arguments` are Cargo's (for example `check -p pse-runtime --features solver-kinsol`), as the
/// correctness adapter accepts them; its resolution, force-validation qualification included,
/// supplies the unit graph.
pub fn derive(arguments: &[OsString]) -> Result<NativeRequirement, HostError> {
    let table = CapabilityTable::read(arguments)?;
    let resolution =
        correctness::resolve(arguments).map_err(|error| invalid(error.to_string()))?;
    table.requirement(&resolution)
}

fn manifest_path(arguments: &[OsString]) -> Result<Option<OsString>, HostError> {
    let end = arguments
        .iter()
        .position(|argument| argument == "--")
        .unwrap_or(arguments.len());
    let mut iterator = arguments[..end].iter();
    while let Some(argument) = iterator.next() {
        if argument == "--manifest-path" {
            return iterator
                .next()
                .cloned()
                .map(Some)
                .ok_or_else(|| invalid("--manifest-path needs a value"));
        }
        if let Some(path) = argument
            .to_str()
            .and_then(|text| text.strip_prefix("--manifest-path="))
        {
            return Ok(Some(path.into()));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn features(names: &[&str]) -> BTreeMap<String, Vec<String>> {
        names
            .iter()
            .map(|name| ((*name).to_owned(), Vec::new()))
            .collect()
    }
    fn resolution(activated: &[&str]) -> Resolution {
        Resolution {
            arguments: Vec::new(),
            packages: Vec::new(),
            activated: [(
                PACKAGE.to_owned(),
                activated.iter().map(|name| (*name).to_owned()).collect(),
            )]
            .into(),
        }
    }
    fn set(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn capability_table_requires_every_feature_and_known_capabilities() {
        let declared = features(&["default", "a", "b"]);
        let table = CapabilityTable::from_package(
            &declared,
            &json!({"pse":{KEY:{"a":["uno"],"b":[]}}}),
        )
        .unwrap();
        let derived = table.requirement(&resolution(&["a", "default"])).unwrap();
        assert_eq!(derived.capabilities, set(&["compiler", "solver", "uno"]));
        assert_eq!(derived.profile(), "compiler,solver,uno");
        assert!(
            table
                .requirement(&resolution(&["b"]))
                .unwrap()
                .capabilities
                .is_empty()
        );
        for (metadata, message) in [
            (json!({}), "declares no"),
            (json!({"pse":{KEY:{"a":["uno"]}}}), "feature b has no declaration"),
            (json!({"pse":{KEY:{"a":[],"b":[],"c":[]}}}), "c is not a declarable"),
            (json!({"pse":{KEY:{"a":[],"b":[],"default":[]}}}), "default is not"),
            (json!({"pse":{KEY:{"a":["gpu"],"b":[]}}}), "unknown native capability"),
            (json!({"pse":{KEY:{"a":"solver","b":[]}}}), "must list"),
        ] {
            let error = CapabilityTable::from_package(&declared, &metadata).unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
        }
        let error = table.requirement(&resolution(&["undeclared"])).unwrap_err();
        assert!(error.to_string().contains("has no declaration"), "{error}");
    }

    #[test]
    fn capability_manifest_path_follows_cargo_selection() {
        let args = |values: &[&str]| values.iter().map(OsString::from).collect::<Vec<_>>();
        assert_eq!(manifest_path(&args(&["check", "-p", "x"])).unwrap(), None);
        assert_eq!(
            manifest_path(&args(&["check", "--manifest-path", "/w/Cargo.toml"])).unwrap(),
            Some("/w/Cargo.toml".into())
        );
        assert_eq!(
            manifest_path(&args(&["check", "--manifest-path=/w/Cargo.toml"])).unwrap(),
            Some("/w/Cargo.toml".into())
        );
        assert_eq!(
            manifest_path(&args(&["run", "--bin", "x", "--", "--manifest-path", "y"])).unwrap(),
            None
        );
    }

    /// Independent expectations: `pse-runtime` feature, the backend features its graph must
    /// activate and the closed capabilities build.rs needs for them.
    const EXPECTED: &[(&str, &[&str], &[&str])] = &[
        ("solver-ipopt", &["ipopt"], &["solver"]),
        ("solver-uno", &["uno"], &["compiler", "solver", "uno"]),
        ("solver-petsc", &["petsc"], &["compiler", "petsc", "solver"]),
        ("solver-highs", &["highs"], &[]),
        ("solver-pounce", &["pounce"], &[]),
        ("solver-kinsol", &["kinsol"], &["compiler", "klu"]),
        (
            "solver-root-isolation",
            &["root-isolation"],
            &["compiler", "isolation"],
        ),
        ("solver-sdp", &["sdp"], &["solver"]),
        ("solver-clarabel-pardiso", &["clarabel-pardiso"], &["solver"]),
        ("solver-diffsol", &["diffsol"], &["compiler", "klu"]),
        ("solver-idas", &["diffsol", "idas"], &["compiler", "klu"]),
        ("solver-scip", &["scip"], &["solver"]),
    ];
    fn runtime(features: Option<&str>) -> NativeRequirement {
        let mut arguments: Vec<OsString> = vec![
            "check".into(),
            "--locked".into(),
            "-p".into(),
            "pse-runtime".into(),
        ];
        if let Some(features) = features {
            arguments.extend(["--features".into(), features.into()]);
        }
        derive(&arguments).unwrap()
    }
    fn assert_feature(feature: &str) {
        let (_, backend, capabilities) = EXPECTED
            .iter()
            .find(|(name, _, _)| *name == feature)
            .unwrap();
        let derived = runtime(Some(feature));
        assert_eq!(derived.features, set(backend), "{feature}");
        assert_eq!(derived.capabilities, set(capabilities), "{feature}");
    }

    #[test]
    fn capability_expectations_cover_the_real_declaration() {
        let table = CapabilityTable::read(&[]).unwrap();
        let covered: BTreeSet<_> = EXPECTED
            .iter()
            .flat_map(|(_, backend, _)| backend.iter().copied())
            .chain(["native-solvers"])
            .collect();
        assert_eq!(table.features().collect::<BTreeSet<_>>(), covered);
    }
    #[test]
    fn capability_derived_ipopt() {
        assert_feature("solver-ipopt");
    }
    #[test]
    fn capability_derived_uno() {
        assert_feature("solver-uno");
    }
    #[test]
    fn capability_derived_petsc() {
        assert_feature("solver-petsc");
    }
    #[test]
    fn capability_derived_highs() {
        assert_feature("solver-highs");
    }
    #[test]
    fn capability_derived_pounce() {
        assert_feature("solver-pounce");
    }
    #[test]
    fn capability_derived_kinsol() {
        assert_feature("solver-kinsol");
    }
    #[test]
    fn capability_derived_root_isolation() {
        assert_feature("solver-root-isolation");
    }
    #[test]
    fn capability_derived_sdp() {
        assert_feature("solver-sdp");
    }
    #[test]
    fn capability_derived_clarabel_pardiso() {
        assert_feature("solver-clarabel-pardiso");
    }
    #[test]
    fn capability_derived_diffsol() {
        assert_feature("solver-diffsol");
    }
    #[test]
    fn capability_derived_idas() {
        assert_feature("solver-idas");
    }
    #[test]
    fn capability_derived_scip() {
        assert_feature("solver-scip");
    }
    #[test]
    fn capability_derived_native_solvers_is_the_full_set() {
        let derived = runtime(Some("native-solvers"));
        assert!(derived.features.contains("native-solvers"));
        assert_eq!(
            derived.capabilities,
            set(&["compiler", "isolation", "klu", "petsc", "solver", "uno"])
        );
    }
    #[test]
    fn capability_derived_runtime_without_solvers_needs_none() {
        let derived = runtime(None);
        assert!(derived.features.is_empty(), "{:?}", derived.features);
        assert_eq!(derived.profile(), "");
    }
}
