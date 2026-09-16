// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! External stage rows are checked against the catalog's actual registered operation.

use super::membership::{AdmissionContext, refused};
use crate::{Catalog, CatalogError};
use pse_ids::CancellationToken;
use pse_relations::RecordBatch;
use std::collections::BTreeMap;

pub(super) async fn validate(
    catalog: &Catalog,
    context: &AdmissionContext,
    candidates: &BTreeMap<String, RecordBatch>,
    cancel: &CancellationToken,
) -> Result<(), CatalogError> {
    if context
        .invocation
        .as_ref()
        .and_then(|invocation| invocation.engine.as_ref())
        .is_none()
    {
        return Err(refused("external operation has no captured native engine"));
    }
    let producer = context.stage_pass.and_then(|pass| {
        catalog
            .validator
            .as_ref()
            .and_then(|validator| validator.stage_producer(pass))
    });
    if producer.is_none() {
        return Err(refused(
            "stage admission requires an executable validator for the actual registered producer",
        ));
    }
    let prepared = catalog.prepare_production(context.clone(), None, cancel)?;
    let spec = prepared.spec().clone();
    let expected = prepared.execute(cancel).await?;
    for port in &spec.outputs {
        let actual = candidates
            .get(port.port)
            .ok_or_else(|| refused("external stage output port is absent"))?;
        let relation = catalog
            .registry()
            .relation(&port.relation)
            .ok_or_else(|| refused("output declaration is absent"))?;
        let contract = crate::RelationContract::from_spec(
            catalog.registry(),
            relation,
            crate::EncodingPolicy::IpcFile,
        )?;
        let sorted = |batch: &RecordBatch| {
            pse_ids::canonicalize(
                &contract.canonical,
                std::slice::from_ref(batch),
                catalog.reserver().as_ref(),
                pse_ids::CanonicalizeOptions {
                    keep_sorted: true,
                    keep_preimage: false,
                    cancel: Some(cancel.clone()),
                    envelope: catalog.limits().envelope,
                },
            )?
            .sorted
            .ok_or_else(|| refused("native value ordering omitted its Arrow result"))
        };
        // Complete normalized values and multiplicities establish correspondence.
        // Hashes do not certify that the registered operation produced these rows.
        if sorted(expected.outputs()[port.port].batch())? != sorted(actual)? {
            return Err(refused(&format!(
                "actual stage rows differ from executed producer: {}",
                port.relation
            )));
        }
    }
    Ok(())
}
