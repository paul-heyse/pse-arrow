// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked binding of the independent reference package to the declared physical source.

use pse_model::generated::{
    authored::packages::AuthoredPackagesFieldDependenciesItem, enums::ModelingVersionOperator,
    identities::PackageId, structures::VersionRequirement,
};

const PHYSICAL_PACKAGE: &str = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a";
const MODEL_PACKAGE: &str = "01991d6a-13a0-7000-8000-000000000001";

pub(super) fn named_model_manifest(
    model: &str,
    physical: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let physical: toml::Table = toml::from_str(std::str::from_utf8(physical)?)?;
    let dependency = physical
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or("reference physical package header absent")?;
    if dependency.contains_key("id")
        || dependency.get("package_id").and_then(toml::Value::as_str) != Some(PHYSICAL_PACKAGE)
        || dependency.get("version").and_then(toml::Value::as_str) != Some("1.0.0")
    {
        return Err("reference physical package identity or version differs".into());
    }
    let mut model: toml::Table = toml::from_str(model)?;
    let header = model
        .get_mut("package")
        .and_then(toml::Value::as_table_mut)
        .ok_or("reference model package header absent")?;
    if header.contains_key("package_id")
        || header.get("id").and_then(toml::Value::as_str) != Some(MODEL_PACKAGE)
        || header.get("id_policy").and_then(toml::Value::as_str) != Some("explicit")
        || !header
            .get("dependencies")
            .and_then(toml::Value::as_array)
            .is_some_and(Vec::is_empty)
    {
        return Err(
            "reference model overlay requires the original explicit package and empty dependencies"
                .into(),
        );
    }
    let physical_id = pse_ids::SemanticId::parse_hex(PHYSICAL_PACKAGE)?;
    let binding = AuthoredPackagesFieldDependenciesItem {
        package_id: PackageId::from_id(physical_id),
        version_req: VersionRequirement {
            operator: ModelingVersionOperator::Exact,
            major: 1,
            minor: 0,
            patch: 0,
        },
    };
    header.insert("id_policy".into(), toml::Value::String("named".into()));
    header.insert("dependencies".into(), toml::Value::try_from([binding])?);
    Ok(toml::to_string(&model)?.into_bytes())
}
