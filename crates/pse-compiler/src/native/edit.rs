// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source edit and rename are finite native computations over actual document children.
use super::Value;
use crate::{
    Algorithm, AlgorithmContext, AlgorithmInputs, AlgorithmOutput, CompilerError, passes::invalid,
};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::authored::{document_edits, documents, rename_requests},
};
use pse_schema::{Registry, model::AlgorithmSpec};
use std::{collections::BTreeMap, sync::Arc};

/// Apply exact document before-images and replacements as a native relation value.
/// The output can feed source projection, compilation or publication directly.
/// # Errors
/// Mismatched request contract, missing inputs, or native planning refusal.
pub async fn documents_plan(
    session: &SnapshotSession,
    documents: Value,
    requests: Value,
    cancel: &CancellationToken,
) -> Result<Value, CompilerError> {
    plan(session, documents, requests, "source_edit", cancel).await
}
/// Rename exactly one explicit source identity under its expected current name.
/// The bound expressions and targets must retain their meanings after the rename.
/// # Errors
/// Mismatched request contract, missing inputs, or native planning refusal.
pub async fn rename_plan(
    session: &SnapshotSession,
    documents: Value,
    requests: Value,
    cancel: &CancellationToken,
) -> Result<Value, CompilerError> {
    plan(session, documents, requests, "source_rename", cancel).await
}
async fn plan(
    session: &SnapshotSession,
    documents: Value,
    requests: Value,
    name: &str,
    cancel: &CancellationToken,
) -> Result<Value, CompilerError> {
    let algorithm = Arc::new(Edit::new(session.registry(), name)?);
    super::plan(
        algorithm,
        BTreeMap::from([("request".into(), Some(requests))]),
        documents,
        session,
        cancel,
    )
    .await?
    .remove("documents")
    .ok_or_else(|| invalid("source operation omitted its document result"))
}
#[derive(Debug)]
struct Edit {
    spec: AlgorithmSpec,
}
impl Edit {
    fn new(registry: &Registry, name: &str) -> Result<Self, CompilerError> {
        let spec = registry
            .algorithm(name)
            .ok_or_else(|| invalid("source operation undeclared"))?
            .clone();
        Ok(Self { spec })
    }
}
impl Algorithm for Edit {
    fn spec(&self) -> &AlgorithmSpec {
        &self.spec
    }
    fn requires_physical(&self) -> bool {
        false
    }
    fn run<'a>(
        &'a self,
        ctx: &'a AlgorithmContext<'a>,
        inputs: &'a AlgorithmInputs,
    ) -> pse_catalog::provider::BoxFut<'a, Result<AlgorithmOutput, CompilerError>> {
        Box::pin(async move {
            let request = inputs
                .port("request")
                .and_then(Option::as_ref)
                .ok_or_else(|| invalid("source request absent"))?
                .relation()?
                .checked();
            let documents = if self.spec.name == "source_edit" {
                let edits = document_edits::View::from_checked(request)?
                    .rows()?
                    .into_iter()
                    .map(|row| pse_authoring::document::DocumentEdit {
                        document_id: row.document_id,
                        path: row.path,
                        before: row.before,
                        after: row.after,
                    })
                    .collect::<Vec<_>>();
                ctx.documents.edit(
                    &edits,
                    ctx.registry,
                    pse_authoring::ParseBudget::default(),
                    ctx.reserver,
                    ctx.cancel,
                )?
            } else {
                let rows = rename_requests::View::from_checked(request)?.rows()?;
                let [row] = rows.as_slice() else {
                    return Err(invalid("a rename request must select exactly one entity"));
                };
                pse_authoring::document::rename_documents(
                    ctx.documents,
                    row.entity_id,
                    &row.expected_name,
                    &row.new_name,
                    ctx.session,
                    ctx.cancel,
                )
                .await?
            };
            let parts = documents
                .bundles()
                .iter()
                .map(|bundle| {
                    bundle
                        .batches
                        .get(&documents::RELATION_ID)
                        .cloned()
                        .ok_or_else(|| invalid("parsed document relation absent"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let output = FieldCheckedBatch::concat_reserved(
                ctx.registry,
                documents::spec(ctx.registry)?,
                &parts,
                ctx.reserver,
                ctx.cancel,
            )?;
            Ok(AlgorithmOutput {
                outputs: BTreeMap::from([("documents".into(), output)]),
                findings: vec![],
                derivations: vec![],
                plans: vec![],
            })
        })
    }
}
