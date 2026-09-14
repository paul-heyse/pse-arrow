// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An actual admitted fixture producer precedes ordinary Driver execution of P10.

use super::{
    Environment,
    observation::{Index, StoredSnapshot},
};
use anyhow::{Context, Result, ensure};
use pse_catalog::{
    EncodingPolicy, RelationContract, Snapshot,
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_compiler::{
    ExternalInputs, PolicySet,
    driver::{Driver, PipelineRequest},
    passes::p10::{P10, fixture},
};
use pse_ids::{SnapshotKind, model_port_name};
use pse_relations::RecordBatch;
use pse_schema::model::{Cell, RelationKey, SnapshotClass};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, sync::Arc};

type Rows = BTreeMap<String, Vec<Vec<String>>>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Observation {
    snapshots: BTreeMap<String, Rows>,
    identities: BTreeMap<String, super::observation::SnapshotIdentity>,
    query: Vec<Vec<String>>,
    sources: BTreeMap<String, String>,
}

pub(super) async fn write(path: &Path) -> Result<()> {
    let path = path.join("p10-fixture");
    std::fs::create_dir(&path)?;
    let environment = Environment::with_registry(&path, Arc::new(fixture::registry()?))?;
    let (inputs, _) = fixture::arithmetic_inputs(&environment.registry)?;
    let model = publish_model(&environment).await?;
    let importer = publish_importer(&environment, &inputs, &model).await?;
    let mut driver = Driver::new(
        Arc::clone(&environment.catalog),
        Arc::clone(&environment.sessions),
    )?;
    driver.register(Arc::new(P10::new(&environment.registry)?))?;
    let report = driver
        .run(
            PipelineRequest {
                through: "P10".to_owned(),
                snapshot: Arc::clone(&importer),
                policies: PolicySet::default(),
                external_bindings: ExternalInputs::default(),
                fixture_mode: true,
                reuse: false,
            },
            &environment.cancel,
        )
        .await?;
    ensure!(
        report.stages.len() == 1 && report.stages[0].pass == "P10",
        "P10 fixture must execute exactly its declared closure"
    );
    let package = fixture::physical_package(&environment.registry)?;
    let index = Index {
        sources: package
            .documents
            .into_iter()
            .map(|document| {
                (
                    format!("{}/{}", package.package.package_id, document.path),
                    document.text,
                )
            })
            .collect(),
        snapshots: vec![
            StoredSnapshot::new("model", &model, &[])?,
            StoredSnapshot::new("importer", &importer, &[("model", &model)])?,
            StoredSnapshot::new(
                "P10",
                &report.stages[0].snapshot,
                &[("model", &model), ("importer", &importer)],
            )?,
        ],
    };
    super::write_json(&path.join("store-index.json"), &index)
}

async fn publish_model(environment: &Environment) -> Result<Arc<Snapshot>> {
    let registry = &environment.registry;
    let inputs = fixture::model_inputs(registry)?;
    let package = fixture::physical_package(registry)?;
    for document in &package.documents {
        environment
            .catalog
            .put_document(document.id, document.text.as_bytes(), &environment.cancel)
            .await?;
    }
    let context = AdmissionContext::default();
    let mut relations = BTreeMap::new();
    for spec in registry
        .relations()
        .iter()
        .filter(|spec| spec.snapshot_class == SnapshotClass::Model)
    {
        let batch = inputs
            .get(&spec.key)
            .context("complete Model fixture omitted a relation")?
            .clone();
        relations.insert(
            model_port_name(spec.key.namespace.as_str(), spec.id),
            RelationDraft {
                contract: Arc::new(RelationContract::from_spec(
                    registry,
                    spec,
                    EncodingPolicy::IpcFile,
                )?),
                batches: vec![batch],
            },
        );
    }
    let manifest = environment
        .catalog
        .manifest_template(SnapshotKind::Model, &context)?;
    Ok(environment
        .catalog
        .publish_bundle(
            BundleDraft {
                manifest,
                relations,
                context,
            },
            &environment.cancel,
        )
        .await?)
}

async fn publish_importer(
    environment: &Environment,
    inputs: &BTreeMap<RelationKey, RecordBatch>,
    model: &Arc<Snapshot>,
) -> Result<Arc<Snapshot>> {
    let registry = &environment.registry;
    let spec = registry
        .pass("FixtureP10Inputs@1")
        .context("fixture importer undeclared")?;
    let context = AdmissionContext {
        stage_pass: Some(spec.id),
        parents: spec
            .inputs
            .iter()
            .map(|port| (port.port.to_owned(), Arc::clone(model)))
            .collect(),
    };
    let relations = spec
        .outputs
        .iter()
        .map(|port| {
            let relation = registry
                .relation(&port.relation)
                .context("importer relation undeclared")?;
            let batch = inputs
                .get(&relation.key)
                .context("complete predecessor fixture omitted port")?;
            Ok((
                port.port.to_owned(),
                RelationDraft {
                    contract: Arc::new(RelationContract::from_spec(
                        registry,
                        relation,
                        EncodingPolicy::IpcFile,
                    )?),
                    batches: vec![batch.clone()],
                },
            ))
        })
        .collect::<Result<_>>()?;
    let manifest = environment
        .catalog
        .manifest_template(SnapshotKind::Stage, &context)?;
    Ok(environment
        .catalog
        .publish_bundle(
            BundleDraft {
                manifest,
                relations,
                context,
            },
            &environment.cancel,
        )
        .await?)
}

pub(super) async fn observe(path: &Path) -> Result<Option<Observation>> {
    let path = path.join("p10-fixture");
    if !path.exists() {
        return Ok(None);
    }
    let index: Index = super::read_json(&path.join("store-index.json"))?;
    let environment = Environment::with_registry(&path, Arc::new(fixture::registry()?))?;
    let mut opened = BTreeMap::<String, Arc<Snapshot>>::new();
    let mut snapshots = BTreeMap::new();
    let mut identities = BTreeMap::new();
    for stored in index.snapshots {
        let context = AdmissionContext {
            stage_pass: stored.stage_pass,
            parents: stored
                .parents
                .iter()
                .map(|(role, label)| {
                    Ok((
                        role.clone(),
                        Arc::clone(opened.get(label).context("fixture parent missing")?),
                    ))
                })
                .collect::<Result<_>>()?,
        };
        let snapshot = environment
            .catalog
            .read_manifest(stored.reference, &context, &environment.cancel)
            .await?;
        snapshots.insert(
            stored.label.clone(),
            super::observation::rows(&snapshot, &environment.registry)?,
        );
        identities.insert(
            stored.label.clone(),
            super::observation::identity(&snapshot),
        );
        opened.insert(stored.label, snapshot);
    }
    super::observation::sources(&environment, &opened, &index.sources).await?;
    let snapshot = opened.get("P10").context("P10 output missing")?;
    let session = environment.sessions.open_session(
        vec![Arc::clone(snapshot)],
        Arc::clone(&environment.registry),
        &environment.cancel,
    )?;
    let batches = session
        .sql(
            "SELECT * FROM compiled.math_int_constants ORDER BY node_id",
            &environment.cancel,
        )
        .await?;
    let rows = batches
        .iter()
        .map(|batch| pse_relations::cells::decode_columns(&environment.registry, batch))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    ensure!(
        rows.len() == 1 && rows[0].get(1) == Some(&Cell::I64(3)),
        "P10 Driver output must contain actual folded integer three"
    );
    let query = rows
        .into_iter()
        .map(|row| row.iter().map(Cell::literal_spec).collect())
        .collect();
    Ok(Some(Observation {
        snapshots,
        identities,
        query,
        sources: index.sources,
    }))
}
