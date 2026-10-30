// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Preparation requirements from the selected backend unit's declared feature set.

use crate::HostError;
use serde_json::Value;
use std::collections::BTreeSet;

/// Join Cargo package metadata with actual backend unit features, then close capabilities.
///
/// The caller supplies `Package::metadata` for pse-backend-native and features captured from
/// the selected compiler unit. A package name or a workspace-wide feature union is insufficient.
/// This operation neither selects providers nor prepares them.
pub fn requirements_from_backend_features(
    metadata: &Value,
    features: &[String],
) -> Result<BTreeSet<String>, HostError> {
    let declarations = metadata
        .pointer("/pse/native-capabilities")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            HostError::Ownership("backend native-capabilities metadata is missing".into())
        })?;
    let mut requested = BTreeSet::new();
    for feature in features {
        let Some(value) = declarations.get(feature) else {
            continue;
        };
        let capabilities = value.as_array().ok_or_else(|| {
            HostError::Ownership(format!(
                "invalid capability declaration for feature {feature}"
            ))
        })?;
        for capability in capabilities {
            let name = capability
                .as_str()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| {
                    HostError::Ownership(format!("invalid capability name for feature {feature}"))
                })?;
            requested.insert(name);
        }
    }
    let joined = requested.into_iter().collect::<Vec<_>>().join(",");
    Ok(super::providers::capabilities(&joined)?
        .into_iter()
        .map(str::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> Value {
        let manifest: toml::Value =
            toml::from_str(include_str!("../../../pse-backend-native/Cargo.toml"))
                .expect("backend manifest");
        serde_json::to_value(&manifest["package"]["metadata"]).expect("metadata")
    }

    #[test]
    fn declared_backend_features_use_provider_owned_closure() {
        for (feature, expected) in [
            ("ipopt", "solver"),
            ("scip", "solver"),
            ("sdp", "solver"),
            ("clarabel-pardiso", "solver"),
            ("kinsol", "compiler,klu"),
            ("idas", "compiler,klu"),
            ("diffsol", "compiler,klu"),
            ("root-isolation", "compiler,isolation"),
            ("uno", "compiler,solver,uno"),
            ("petsc", "compiler,petsc,solver"),
        ] {
            let actual = requirements_from_backend_features(&metadata(), &[feature.into()])
                .expect("requirements");
            assert_eq!(
                actual,
                expected.split(',').map(str::to_owned).collect(),
                "{feature}"
            );
        }
    }

    #[test]
    fn pure_selection_needs_no_provider_and_full_selection_deduplicates() {
        assert!(
            requirements_from_backend_features(&metadata(), &[])
                .expect("pure")
                .is_empty()
        );
        assert!(
            requirements_from_backend_features(&metadata(), &["highs".into(), "pounce".into()])
                .expect("independent libraries")
                .is_empty()
        );
        let full: Vec<_> = metadata()
            .pointer("/pse/native-capabilities")
            .expect("table")
            .as_object()
            .expect("object")
            .keys()
            .cloned()
            .collect();
        assert_eq!(
            requirements_from_backend_features(&metadata(), &full).expect("all"),
            ["compiler", "solver", "klu", "isolation", "uno", "petsc"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
    }

    #[test]
    fn missing_malformed_or_unknown_selected_capability_refuses() {
        assert!(requirements_from_backend_features(&Value::Null, &[]).is_err());
        for value in [
            serde_json::json!("solver"),
            serde_json::json!([3]),
            serde_json::json!([""]),
            serde_json::json!(["invented"]),
        ] {
            let data = serde_json::json!({"pse":{"native-capabilities":{"feature":value}}});
            assert!(requirements_from_backend_features(&data, &["feature".into()]).is_err());
        }
    }

    #[test]
    fn actual_cargo_units_derive_each_backend_capability_without_building() {
        for (feature, expected) in [
            ("ipopt", "solver"),
            ("scip", "solver"),
            ("sdp", "solver"),
            ("clarabel-pardiso", "solver"),
            ("kinsol", "compiler,klu"),
            ("idas", "compiler,klu"),
            ("diffsol", "compiler,klu"),
            ("root-isolation", "compiler,isolation"),
            ("uno", "compiler,solver,uno"),
            ("petsc", "compiler,petsc,solver"),
        ] {
            let qualified = format!("pse-backend-native/{feature}");
            let arguments: Vec<_> = [
                "check",
                "--locked",
                "-p",
                "pse-backend-native",
                "--lib",
                "--no-default-features",
                "--features",
                &qualified,
            ]
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect();
            let actual =
                crate::correctness::resolve(&arguments).expect("actual no-build Cargo graph");
            assert_eq!(
                actual.native_capabilities,
                expected.split(',').map(str::to_owned).collect(),
                "{feature}"
            );
        }
    }

    #[test]
    fn actual_runtime_solver_and_pure_units_preserve_arrow_admission() {
        let base: Vec<_> = [
            "check",
            "--locked",
            "-p",
            "pse-runtime",
            "--lib",
            "--no-default-features",
        ]
        .into_iter()
        .map(std::ffi::OsString::from)
        .collect();
        let pure = crate::correctness::resolve(&base).expect("pure runtime graph");
        assert!(pure.native_capabilities.is_empty());
        let leaves: Vec<_> = pure
            .units
            .iter()
            .filter(|unit| unit.package_name == "arrow-array")
            .collect();
        assert!(!leaves.is_empty(), "the real selection consumes Arrow");
        assert!(
            leaves
                .iter()
                .all(|unit| unit.features.contains("force_validate"))
        );
        let mut solver = base;
        solver.extend(["--features".into(), "pse-runtime/native-solvers".into()]);
        assert_eq!(
            crate::correctness::resolve(&solver)
                .expect("full native graph")
                .native_capabilities,
            ["compiler", "solver", "klu", "isolation", "uno", "petsc"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
    }
}
