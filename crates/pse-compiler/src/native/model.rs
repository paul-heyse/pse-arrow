// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Existing process-model functionality composed as ordinary native plan values.
use super::Value;
use crate::{Algorithm, ArtifactPlan, CompilerError, RelationOutput, passes};
use datafusion::common::ResolvedTableReference;
use pse_catalog::session::{RelationPlan, SnapshotSession};
use pse_ids::CancellationToken;
use pse_schema::model::RelationKey;
use std::{collections::BTreeMap, sync::Arc};

/// Compose the existing complete process-model compiler over exact selected sources.
/// This builds a graph; it neither runs algorithms nor creates intermediate publications.
/// # Errors
/// An input is absent/ambiguous, fields differ, or native planning is refused.
pub async fn compile(
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<ArtifactPlan, CompilerError> {
    let registry = session.registry();
    let mut inputs = BTreeMap::new();
    for key in session.input_keys() {
        let reference = session.table_reference(&key)?;
        let reference = ResolvedTableReference {
            catalog: reference
                .catalog()
                .ok_or_else(|| passes::invalid("compiler source has no catalog"))?
                .into(),
            schema: reference
                .schema()
                .ok_or_else(|| passes::invalid("compiler source has no schema"))?
                .into(),
            table: reference.table().into(),
        };
        inputs.insert(
            key,
            Value::from_relation(session.relation_plan(&reference)?),
        );
    }
    compose(session, inputs, cancel, algorithms(registry)?).await
}
fn algorithms(registry: &pse_schema::Registry) -> Result<Vec<Arc<dyn Algorithm>>, CompilerError> {
    Ok(vec![
        Arc::new(passes::p3::P3::new(registry)?),
        Arc::new(passes::p4::P4::new(registry)?),
        Arc::new(passes::p5::P5::new(registry)?),
        Arc::new(passes::p6::P6::new(registry)?),
        Arc::new(passes::p7::P7::new(registry)?),
        Arc::new(passes::p8::P8::new(registry)?),
        Arc::new(passes::p9::P9::new(registry)?),
        Arc::new(passes::p10::P10::new(registry)?),
    ])
}
/// Compose source parsing directly with the current mathematical compiler.
/// All document and domain computations remain lazy native dependencies.
/// # Errors
/// Missing document contract, incomplete inputs or native planning refusal.
pub async fn from_documents(
    session: &SnapshotSession,
    documents: RelationPlan,
    cancel: &CancellationToken,
) -> Result<ArtifactPlan, CompilerError> {
    from_documents_using(session, documents, cancel, algorithms(session.registry())?).await
}
/// Compose explicit finite algorithms over the actual parsed source relation.
/// Each algorithm declares its own typed arguments, results and native obligations;
/// callers can select existing outcomes without publishing intermediate stages.
/// # Errors
/// Missing dependencies, invalid declarations or native planning refusal.
pub async fn from_documents_using(
    session: &SnapshotSession,
    documents: RelationPlan,
    cancel: &CancellationToken,
    algorithms: Vec<Arc<dyn Algorithm>>,
) -> Result<ArtifactPlan, CompilerError> {
    let source = source_values(session, documents, cancel).await?;
    compose(session, source, cancel, algorithms).await
}
/// Build a complete typed source artifact from the actual document relation.
/// # Errors
/// Invalid documents/field declarations or native planning refusal.
pub async fn source(
    session: &SnapshotSession,
    documents: RelationPlan,
    cancel: &CancellationToken,
) -> Result<ArtifactPlan, CompilerError> {
    artifact(
        session,
        source_values(session, documents, cancel).await?,
        cancel,
    )
}
async fn source_values(
    session: &SnapshotSession,
    documents: RelationPlan,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Value>, CompilerError> {
    let algorithm = Arc::new(super::source::SourceProjection::new(session.registry())?);
    super::plan(
        algorithm,
        BTreeMap::new(),
        Value::from_relation(documents),
        session,
        cancel,
    )
    .await?
    .into_values()
    .map(|value| {
        let key = session
            .registry()
            .relation_by_id(value.relation().relation_id())
            .ok_or_else(|| passes::invalid("source result declaration absent"))?
            .key;
        Ok((key, value))
    })
    .collect()
}

async fn compose(
    session: &SnapshotSession,
    mut relations: BTreeMap<RelationKey, Value>,
    cancel: &CancellationToken,
    algorithms: Vec<Arc<dyn Algorithm>>,
) -> Result<ArtifactPlan, CompilerError> {
    let registry = session.registry();
    let documents = relations
        .get(&pse_relations::generated::authored::documents::RELATION_KEY)
        .ok_or_else(|| passes::invalid("source documents absent"))?
        .clone();
    let mut outputs = relations
        .iter()
        .filter(|(_, value)| value.tuple.is_some())
        .map(|(key, value)| (*key, value.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut evidence = BTreeMap::<RelationKey, Vec<Value>>::new();
    for algorithm in algorithms {
        cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        let mut arguments = BTreeMap::new();
        for port in &algorithm.spec().inputs {
            let spec = registry
                .relation(&port.relation)
                .ok_or_else(|| passes::invalid("algorithm input undeclared"))?;
            arguments.insert(port.port.clone(), relations.get(&spec.key).cloned());
        }
        for (name, value) in
            super::plan(algorithm, arguments, documents.clone(), session, cancel).await?
        {
            let spec = registry
                .relation_by_id(value.relation().relation_id())
                .ok_or_else(|| passes::invalid("algorithm result undeclared"))?;
            if name.starts_with("__") {
                evidence.entry(spec.key).or_default().push(value);
            } else {
                relations.insert(spec.key, value.clone());
                outputs.insert(spec.key, value);
            }
        }
    }
    for (key, values) in evidence {
        let Some(first) = values.first() else {
            continue;
        };
        let plan = super::evidence::combine(
            key,
            values
                .iter()
                .map(|value| value.relation().plan().clone())
                .collect(),
            session,
            cancel,
        )?;
        outputs.insert(
            key,
            Value::from_relation(RelationPlan::derived(
                first.relation().relation_id(),
                plan,
                registry,
            )?),
        );
    }
    artifact(session, outputs, cancel)
}
fn artifact(
    session: &SnapshotSession,
    outputs: BTreeMap<RelationKey, Value>,
    cancel: &CancellationToken,
) -> Result<ArtifactPlan, CompilerError> {
    let outputs = outputs
        .into_iter()
        .map(|(key, value)| {
            (
                ResolvedTableReference {
                    catalog: "artifact".into(),
                    schema: key.namespace.as_str().into(),
                    table: key.name.into(),
                },
                RelationOutput {
                    relation_id: value.relation().relation_id(),
                    plan: value.relation().plan().clone(),
                },
            )
        })
        .collect();
    Ok(ArtifactPlan::new(session.clone(), outputs, cancel)?)
}
