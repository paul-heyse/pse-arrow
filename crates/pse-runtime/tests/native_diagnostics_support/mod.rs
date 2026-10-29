// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Test-only admitted physical fixture with an explicit centimetre representation.
use crate::reference_support::{NativeContext, Result, root};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_ids::SemanticId;
use pse_runtime::{CancelSource, workflow::ModelingPackage};
use std::collections::BTreeMap;

pub(crate) const CENTIMETRE: SemanticId = SemanticId::from_bytes([245;16]);

pub(crate) async fn length_package(context: &NativeContext, source: &str) -> Result<ModelingPackage> {
    let fixtures = root().join("tests/fixtures/packages");
    let physical_root = fixtures.join("physical-primitives");
    let mut physical = ["package.toml", "materials/physical.yaml", "materials/time.yaml"]
        .into_iter().map(|name| Ok((name.to_owned(), std::fs::read(physical_root.join(name))?)))
        .collect::<Result<BTreeMap<_,_>>>()?;
    let text = std::str::from_utf8(&physical["materials/physical.yaml"])?
        .lines().filter(|line| !line.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n");
    let mut document: serde_json::Value = serde_json::from_str(&text)?;
    let units = document["units"].as_array_mut().unwrap();
    let mut centimetre = units[0].clone();
    centimetre["unit_id"] = serde_json::to_value(CENTIMETRE)?;
    centimetre["symbol"] = "cm".into();
    centimetre["name"] = "centimetre".into();
    centimetre["scale_to_canonical"] = 0.01.into();
    units.push(centimetre);
    physical.insert("materials/physical.yaml".into(), serde_json::to_vec(&document)?);
    let manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))?
        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
        .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
    let modeling = BTreeMap::from([("package.toml".into(), manifest.into_bytes()),
        ("models/diagnostic.pse".into(), source.as_bytes().to_vec())]);
    let pool = context.shared.pool();
    let token = pse_columnar::CancellationToken::new();
    let validation = context.runtime.validation_context();
    let load = |documents: &BTreeMap<String, Vec<u8>>| -> Result<OwnedDocumentSet> {
        Ok(OwnedDocumentSet::try_from_bundles(vec![load_package_documents_owned(
            documents, context.runtime.registry(), Default::default(), &pool, &token, &validation,
        )?], &pool, &token)?)
    };
    let physical = Box::pin(context.runtime.physical_from_documents(&load(&physical)?, &token)).await?;
    Ok(Box::pin(context.runtime.modeling_from_documents(&load(&modeling)?, physical, &CancelSource::new())).await?)
}
