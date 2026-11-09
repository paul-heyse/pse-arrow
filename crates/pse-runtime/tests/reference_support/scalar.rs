// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned scalar reference inputs, shared by native and independent callers.

use crate::reference_support::Result;
use pse_ids::{ContentHash, SemanticId};
use pse_model::reference_exchange::ReferenceMeaning;
use pse_runtime::{
    reference_exchange::{
        ReferenceCaseContract, ReferenceFixedInput, ReferenceInputArtifact,
        ReferenceObservableContract,
    },
    workflow::{
        PhysicalContext, PhysicalInspectionDocument, PhysicalInspectionQuery,
        PhysicalInspectionRequest,
    },
};
use sha2::{Digest as _, Sha256};
use std::{collections::BTreeMap, sync::Arc};

pub(super) struct Case<'a> {
    pub(super) name: &'a str,
    pub(super) identity: SemanticId,
    pub(super) source: &'a str,
    pub(super) configuration: ContentHash,
    pub(super) inputs: Vec<(String, f64)>,
    pub(super) outputs: Vec<(String, SemanticId)>,
}

pub(super) struct Inputs {
    pub(super) contract: ReferenceCaseContract,
    pub(super) _owner: Arc<datafusion::execution::memory_pool::MemoryReservation>,
}

pub(super) fn bind(
    case: Case<'_>,
    context: &PhysicalContext,
    originals: BTreeMap<String, Vec<u8>>,
) -> Result<Inputs> {
    let PhysicalInspectionDocument::Page {
        mut quantities,
        owner,
        ..
    } = context.describe(&PhysicalInspectionRequest {
        query: PhysicalInspectionQuery::QuantityNamed {
            name: "Scalar".into(),
        },
        offset: 0,
        limit: 1,
    })?
    else {
        return Err("scalar reference physical description is unavailable".into());
    };
    if quantities.len() != 1 {
        return Err("scalar reference physical description is absent or ambiguous".into());
    }
    let physical = quantities.remove(0);
    let artifacts = originals
        .into_iter()
        .map(|(path, content)| ReferenceInputArtifact {
            path,
            sha256: Sha256::digest(content).to_vec().into(),
        })
        .collect::<Vec<_>>();
    let source = pse_ids::preimage::hash(&serde_json::to_vec(&(
        case.source,
        artifacts
            .iter()
            .map(|artifact| (&artifact.path, artifact.sha256.as_slice()))
            .collect::<Vec<_>>(),
    ))?);
    let specification = case
        .inputs
        .into_iter()
        .map(|(key, value)| ReferenceFixedInput {
            key,
            value,
            physical: physical.clone(),
        })
        .collect();
    let observables = case
        .outputs
        .into_iter()
        .map(|(key, output)| ReferenceObservableContract {
            output,
            key,
            physical: physical.clone(),
        })
        .collect();
    Ok(Inputs {
        contract: ReferenceCaseContract {
            version: Default::default(),
            case_name: case.name.into(),
            meaning: ReferenceMeaning {
                case: case.identity,
                source,
                formulation: pse_ids::preimage::hash(case.source.as_bytes()),
                physical: context.identity(),
                configuration: case.configuration,
            },
            observables,
            specification,
            artifacts,
        },
        _owner: owner.ok_or("scalar reference description lost its allocation owner")?,
    })
}
