// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stored golden fixtures compare fully admitted values, not hash certificates.

use anyhow::{Context, Result, bail, ensure};
use clap::ValueEnum;
use pse_authoring::{ParseBudget, document::load_package};
use pse_catalog::{Snapshot, store::membership::AdmissionContext};
use pse_compiler::{
    ExternalInputs, PolicySet,
    driver::{CommitRequest, CommitRevisionIds, Driver, PipelineRequest},
};
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use std::{collections::BTreeMap, path::Path, sync::Arc};

mod environment;
mod observation;
mod p10;
use environment::Environment;
use observation::{Index, Observation, StoredSnapshot};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum Name {
    Registry,
    #[value(name = "minimal_explicit")]
    MinimalExplicit,
}
impl Name {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Registry => "registry",
            Self::MinimalExplicit => "minimal_explicit",
        }
    }
}

pub(crate) fn run(root: &Path, name: Name, check: bool) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    runtime.block_on(run_async(root, name, check))
}

async fn run_async(root: &Path, name: Name, check: bool) -> Result<()> {
    let parent = root.join("tests/golden");
    std::fs::create_dir_all(&parent)?;
    let scratch = tempfile::tempdir_in(&parent)?;
    let fresh = scratch.path().join(name.as_str());
    std::fs::create_dir(&fresh)?;
    println!("golden {}: publishing complete fixture", name.as_str());
    publish(root, &fresh, name).await?;
    println!(
        "golden {}: reopening and querying fresh store",
        name.as_str()
    );
    let actual = observe(&fresh).await?;
    let destination = parent.join(name.as_str());
    if check {
        println!("golden {}: reopening committed store", name.as_str());
        let expected: Observation = read_json(&destination.join("values.json"))?;
        let stored = observe(&destination).await?;
        compare(&expected, &stored).context("stored golden differs from its decoded values")?;
        compare(&stored, &actual).context("fresh execution differs from the stored golden")?;
        println!(
            "golden {}: admitted stored and fresh rows, sources and SQL agree",
            name.as_str()
        );
    } else {
        write_json(&fresh.join("values.json"), &actual)?;
        let backup = scratch.path().join("previous");
        if destination.exists() {
            std::fs::rename(&destination, &backup)?;
        }
        if let Err(error) = std::fs::rename(&fresh, &destination) {
            if backup.exists() {
                std::fs::rename(&backup, &destination)?;
            }
            return Err(error.into());
        }
        println!(
            "golden {}: published and reopened {} snapshots",
            name.as_str(),
            actual.snapshots.len()
        );
    }
    Ok(())
}

async fn publish(root: &Path, path: &Path, name: Name) -> Result<()> {
    let environment = Environment::new(path)?;
    let mut driver = Driver::new(
        Arc::clone(&environment.catalog),
        Arc::clone(&environment.sessions),
    )?;
    let documents = match name {
        Name::Registry => vec![],
        Name::MinimalExplicit => vec![load_package(
            &root.join("tests/fixtures/packages/minimal_explicit"),
            &environment.registry,
            ParseBudget::default(),
        )?],
    };
    let sources = documents
        .iter()
        .flat_map(|bundle| {
            bundle.documents.iter().map(move |document| {
                (
                    format!("{}/{}", bundle.package.package_id, document.path),
                    document.text.clone(),
                )
            })
        })
        .collect();
    let committed = driver
        .commit(
            CommitRequest {
                reference: pse_catalog::RefName::parse("fixture")?,
                revision_ids: Some(CommitRevisionIds {
                    model: id(0x100)?,
                    case: id(0x101)?,
                }),
                base: None,
                documents,
                header: authored::change_sets::Row {
                    change_set_id: id(0x102)?,
                    base_revision_id: SemanticId::NIL,
                    author: "golden fixture".to_owned(),
                    message: "admitted source fixture".to_owned(),
                    created_at: 1_000_000_000,
                },
                changes: None,
            },
            &environment.cancel,
        )
        .await?;
    ensure!(
        committed.validation.error_count == 0,
        "P2 findings: {:?}",
        committed.validation.findings
    );
    let model = committed.model.context("successful commit omitted model")?;
    let case = committed.tip.context("successful commit omitted case")?;
    let mut index = Index {
        snapshots: Vec::new(),
        sources,
    };
    index
        .snapshots
        .push(StoredSnapshot::new("model", &model, &[])?);
    index
        .snapshots
        .push(StoredSnapshot::new("case", &case, &[("model", &model)])?);
    if matches!(name, Name::MinimalExplicit) {
        println!("golden minimal_explicit: publishing complete P10 fixture boundary");
        p10::write(path).await?;
        println!("golden minimal_explicit: running P3 on committed package and case");
        let report = driver
            .run(
                PipelineRequest {
                    through: "P3".to_owned(),
                    snapshot: Arc::clone(&case),
                    policies: PolicySet::default(),
                    external_bindings: ExternalInputs::default(),
                    fixture_mode: false,
                    reuse: false,
                },
                &environment.cancel,
            )
            .await?;
        ensure!(report.stages.len() == 1, "expected one complete P3 stage");
        index.snapshots.push(StoredSnapshot::new(
            "P3",
            &report.stages[0].snapshot,
            &[("model", &model), ("case", &case)],
        )?);
    }
    write_json(&path.join("store-index.json"), &index)
}

fn id(ordinal: u16) -> Result<SemanticId> {
    Ok(pse_authoring::ids::parse_id(
        &format!("01991d6a13a07000800000000000{ordinal:04x}"),
        pse_authoring::SourceSpan::head(SemanticId::NIL),
    )?)
}

async fn observe(path: &Path) -> Result<Observation> {
    let index: Index = read_json(&path.join("store-index.json"))?;
    let environment = Environment::new(path)?;
    let mut opened = BTreeMap::<String, Arc<Snapshot>>::new();
    let mut observation = Observation {
        snapshots: BTreeMap::new(),
        identities: BTreeMap::new(),
        queries: BTreeMap::new(),
        sources: index.sources,
        p10: p10::observe(path).await?,
    };
    for stored in index.snapshots {
        let context = AdmissionContext {
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
        observation.snapshots.insert(
            stored.label.clone(),
            observation::rows(&snapshot, &environment.registry)?,
        );
        observation
            .identities
            .insert(stored.label.clone(), observation::identity(&snapshot));
        opened.insert(stored.label, snapshot);
    }
    observation::query(&environment, &opened, &mut observation).await?;
    observation::sources(&environment, &opened, &observation.sources).await?;
    observation::normalization_budget_control(&opened, &environment.registry)?;
    Ok(observation)
}

fn compare(expected: &Observation, actual: &Observation) -> Result<()> {
    if expected != actual {
        for (name, rows) in &expected.snapshots {
            if actual.snapshots.get(name) != Some(rows) {
                bail!("complete admitted row values or membership differ in {name}");
            }
        }
        bail!("snapshot inventory, query results or original source bytes differ");
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_comparison_refuses_changed_actual_values_under_unchanged_identity() {
        let expected = Observation {
            snapshots: BTreeMap::from([(
                "model".to_owned(),
                BTreeMap::from([(
                    "authored.fixture".to_owned(),
                    vec![vec![pse_schema::model::Cell::I64(3).literal_spec()]],
                )]),
            )]),
            identities: BTreeMap::new(),
            queries: BTreeMap::new(),
            sources: BTreeMap::new(),
            p10: None,
        };
        let mut actual = expected.clone();
        actual
            .snapshots
            .get_mut("model")
            .unwrap()
            .get_mut("authored.fixture")
            .unwrap()[0][0] = pse_schema::model::Cell::I64(4).literal_spec();
        assert_eq!(expected.identities, actual.identities);
        assert!(compare(&expected, &actual).is_err());
    }
}
