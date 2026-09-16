// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fresh current stores for Python inspection; no persisted comparison oracle.

use anyhow::{Context, Result, ensure};
use pse_catalog::{Snapshot, store::membership::AdmissionContext};
use pse_compiler::driver::{CommitRequest, CommitRevisionIds, Driver};
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use std::{collections::BTreeMap, path::Path, process::Command, sync::Arc};

pub(crate) mod environment;
mod index;
use environment::Environment;
use index::{Index, StoredSnapshot};

pub(crate) fn run(path: &Path) -> Result<()> {
    // A caller owns this fresh destination. Never replace an existing store.
    std::fs::create_dir(path)
        .with_context(|| format!("creating fresh inspection store {}", path.display()))?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    runtime.block_on(async {
        publish(path).await?;
        reopen(path).await
    })
}

pub(crate) fn python_tests(root: &Path, args: &[String]) -> Result<()> {
    let scratch = tempfile::tempdir()?;
    let path = scratch.path().join("inspection");
    run(&path)?;
    // Build once before pytest starts workers; every worker reads this store.
    let status = Command::new("uv")
        .current_dir(root)
        .args([
            "run",
            "--no-sync",
            "pytest",
            "-m",
            "unit or component",
            "-n",
            "auto",
        ])
        .args(args)
        .env("PSE_INSPECTION_STORE", &path)
        .status()
        .context("running Python tests against the fresh native store")?;
    ensure!(status.success(), "Python tests failed: {status}");
    Ok(())
}

async fn publish(path: &Path) -> Result<()> {
    let environment = Environment::new(path)?;
    let mut driver = Driver::new(Arc::clone(&environment.catalog))?;
    let committed = driver
        .commit(
            CommitRequest {
                reference: pse_catalog::RefName::parse("fixture")?,
                revision_ids: Some(CommitRevisionIds {
                    model: id(0x100)?,
                    case: id(0x101)?,
                }),
                base: None,
                documents: Vec::new(),
                header: authored::change_sets::Row {
                    change_set_id: id(0x102)?,
                    base_revision_id: SemanticId::NIL,
                    author: "inspection fixture".to_owned(),
                    message: "admitted source fixture".to_owned(),
                    created_at: 1_000_000_000,
                },
                changes: None,
            },
            &environment.cancel,
        )
        .await?;
    ensure!(
        committed.validation.error_count() == 0,
        "inspection fixture commit produced {} errors",
        committed.validation.error_count()
    );
    let model = committed.model.context("successful commit omitted model")?;
    let case = committed.tip.context("successful commit omitted case")?;
    let mut index = Index {
        snapshots: Vec::new(),
    };
    index
        .snapshots
        .push(StoredSnapshot::new("model", &model, &[])?);
    index
        .snapshots
        .push(StoredSnapshot::new("case", &case, &[("model", &model)])?);
    write_json(&path.join("store-index.json"), &index)
}

fn id(ordinal: u16) -> Result<SemanticId> {
    Ok(pse_authoring::ids::parse_id(
        &format!("01991d6a13a07000800000000000{ordinal:04x}"),
        pse_authoring::SourceSpan::head(SemanticId::NIL),
    )?)
}

async fn reopen(path: &Path) -> Result<()> {
    let index: Index = read_json(&path.join("store-index.json"))?;
    let environment = Environment::new(path)?;
    let mut opened = BTreeMap::<String, Arc<Snapshot>>::new();
    for stored in index.snapshots {
        let context = AdmissionContext {
            traversal: Arc::default(),
            invocation: None,
            parents: stored
                .parents
                .iter()
                .map(|(role, label)| {
                    Ok((
                        role.clone(),
                        Arc::clone(opened.get(label).with_context(|| {
                            format!("parent {label} must precede {}", stored.label)
                        })?),
                    ))
                })
                .collect::<Result<_>>()?,
            stage_pass: stored.stage_pass,
        };
        let snapshot = environment
            .catalog
            .read_manifest(stored.reference, &context, &environment.cancel)
            .await?;
        opened.insert(stored.label, snapshot);
    }
    println!(
        "inspection fixture: published and admitted {} current snapshots",
        opened.len()
    );
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(
        &std::fs::read(path).with_context(|| format!("reading {}", path.display()))?,
    )
    .with_context(|| format!("decoding {}", path.display()))
}
fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(std::fs::write(path, bytes)?)
}
